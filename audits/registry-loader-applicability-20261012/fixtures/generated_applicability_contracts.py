"""Frozen applicability contracts for a supplied already-compiled registry generator.
No Cargo invocation; every case is preserved even when its contract fails.
"""
import argparse
import copy
import hashlib
import json
import os
import pathlib
import re
import subprocess
PROPERTIES = ["molar_mass", "molar_heat_capacity", "mass_density"]
FIELDS = ["temperature", "pressure", "ph", "ionic_strength", "medium"]
CASES = [{"name": "canonical", "mutation": "canonical", "refusal": False},
         {"name": "reverse_thermo", "mutation": "reverse", "refusal": False},
         {"name": "null_constraints_and_notes", "mutation": "null_notes", "refusal": False},
         {"name": "missing_conditions", "mutation": "missing", "refusal": False},
         {"name": "null_condition_phase", "mutation": "null_phase", "refusal": False},
         {"name": "excluded_conditional_foreign_records", "mutation": "foreign", "refusal": False}]
for property_name in PROPERTIES:
    for field in FIELDS:
        CASES.append({"name": property_name + "_" + field, "mutation": "constraint",
                      "property": property_name, "field": field, "refusal": True})
    CASES.append({"name": property_name + "_condition_phase_conflict", "mutation": "phase_conflict",
                  "property": property_name, "field": "phase", "refusal": True})
def digest(raw):
    return hashlib.sha256(raw).hexdigest()
def constraint(field):
    if field == "medium":
        return "water"
    bounds = {"temperature": (290.0, 310.0, "K", "temperature"),
              "pressure": (90000.0, 110000.0, "Pa", "pressure"),
              "ph": (6.0, 8.0, "1", "dimensionless"),
              "ionic_strength": (0.0, 0.1, "mol/L", "concentration")}
    lower, upper, symbol, dimension = bounds[field]
    return {"lower": lower, "upper": upper, "unit": {"symbol": symbol, "dimension": dimension}}
def selected_row(doc, property_name):
    rows = [row for row in doc["phase_thermodynamics"]
            if row["species_id"] == "MgSO4" and row["property"] == property_name]
    assert len(rows) == 1
    return rows[0]
def mutated_document(doc, case):
    probe = copy.deepcopy(doc)
    mutation = case["mutation"]
    if mutation == "reverse":
        probe["phase_thermodynamics"].reverse()
    elif mutation in {"constraint", "phase_conflict"}:
        row = selected_row(probe, case["property"])
        assert row["phase"] == "solid"
        row["quantity"]["conditions"][case["field"]] = (
            constraint(case["field"]) if mutation == "constraint" else "gas")
    elif mutation in {"null_notes", "missing", "null_phase"}:
        for property_name in PROPERTIES:
            row = selected_row(probe, property_name)
            if mutation == "missing":
                del row["quantity"]["conditions"]
            elif mutation == "null_phase":
                row["quantity"]["conditions"]["phase"] = None
            else:
                for field in FIELDS:
                    row["quantity"]["conditions"][field] = None
                row["quantity"]["conditions"]["notes"] = "Synthetic descriptive note; no applicability restriction"
    elif mutation == "foreign":
        for property_name in ["molar_heat_capacity", "mass_density"]:
            extra = copy.deepcopy(selected_row(probe, property_name))
            extra["id"] = "independent_conditional_foreign_" + property_name
            extra["phase"] = "gas"
            extra["quantity"]["conditions"]["phase"] = "gas"
            extra["quantity"]["value"] *= 2.0
            for field in FIELDS:
                extra["quantity"]["conditions"][field] = constraint(field)
            probe["phase_thermodynamics"].insert(0, extra)
    else:
        assert mutation == "canonical"
    return probe
def payload_for(doc, case):
    return json.dumps(mutated_document(doc, case), sort_keys=True, separators=(",", ":"), allow_nan=False).encode()
def expected_values(doc):
    expected = {}
    for identity in doc["identities"]:
        key = identity["id"]
        assert key not in expected
        selected = {}
        for property_name in PROPERTIES:
            rows = [row for row in doc["phase_thermodynamics"]
                    if row["species_id"] == key and row["property"] == property_name]
            assert len(rows) == 1
            selected[property_name] = rows[0]
        phase = selected["molar_mass"]["phase"]
        assert all(row["phase"] == phase for row in selected.values())
        expected[key] = {"molar_mass": selected["molar_mass"]["quantity"]["value"],
                         "heat_capacity": selected["molar_heat_capacity"]["quantity"]["value"],
                         "density": selected["mass_density"]["quantity"]["value"], "phase": phase}
    assert len(expected) == 186
    assert sum(row["heat_capacity"] == 0 for row in expected.values()) == 47
    return expected
def values(text, key):
    matches = list(re.finditer(r'key:\s*"' + re.escape(key) + r'",', text))
    assert len(matches) == 1, "missing/repeated species: " + key
    row = text[matches[0].end():].split("    SpeciesData {", 1)[0]
    fields = {}
    for field in ["molar_mass", "density", "heat_capacity"]:
        matches = re.findall(r"^\s*" + field + r":\s*([^,]+),$", row, re.M)
        assert len(matches) == 1, "missing/repeated field: " + field
        fields[field] = float(matches[0])
    matches = re.findall(r"^\s*standard_phase:\s*Phase::(\w+),$", row, re.M)
    assert len(matches) == 1
    fields["phase"] = matches[0].lower()
    return fields
def check_contract(case, exit_code, stderr, out, expected):
    if case["refusal"]:
        assert exit_code is not None and exit_code != 0, "generator admitted constrained selected record"
        error = stderr.lower()
        for token in ["mgso4", case["property"], case["field"]]:
            assert token.encode() in error, "unrelated refusal, missing " + token
        assert not (out / "species_generated.rs").exists(), "refusal emitted species output"
        return
    assert exit_code == 0, "supported input refused or timed out"
    text = (out / "species_generated.rs").read_text()
    keys = re.findall(r'^\s*key:\s*"([^"\n]+)",$', text, re.M)
    assert len(keys) == len(set(keys)) == len(expected) and set(keys) == set(expected), "species inventory changed"
    for key, wanted in expected.items():
        actual = values(text, key)
        assert actual["phase"] == wanted["phase"], "phase changed: " + key
        for field in ["molar_mass", "density", "heat_capacity"]:
            assert abs(actual[field] - wanted[field]) <= max(abs(wanted[field]), 1.0) * 1e-12, "scalar changed: " + key + "/" + field
        if wanted["heat_capacity"] == 0:
            assert actual["heat_capacity"] == 0, "zero Cp convention changed: " + key
    for name in ["solvent_generated.rs", "materials_generated.rs"]:
        assert (out / name).stat().st_size > 0, "missing generated companion: " + name

def main():
    parser = argparse.ArgumentParser()
    for option in ["binary", "registry", "out"]:
        parser.add_argument("--" + option, type=pathlib.Path, required=True)
    for option in ["binary-sha256", "registry-sha256"]:
        parser.add_argument("--" + option, required=True)
    args = parser.parse_args()
    # All admission checks happen before any binary invocation or evidence directory creation.
    if args.out.exists():
        parser.error("output directory already exists; receipts will not be overwritten")
    binary = args.binary.resolve(strict=True)
    binary_sha = digest(binary.read_bytes())
    if not re.fullmatch(r"[0-9a-f]{64}", args.binary_sha256) or binary_sha != args.binary_sha256:
        parser.error("binary SHA256 mismatch")
    raw = args.registry.read_bytes()
    registry_sha = digest(raw)
    if not re.fullmatch(r"[0-9a-f]{64}", args.registry_sha256) or registry_sha != args.registry_sha256:
        parser.error("registry SHA256 mismatch")
    freeze_path = pathlib.Path(__file__).resolve().parents[1] / "generated-freeze.json"
    freeze_raw = freeze_path.read_bytes()
    freeze = json.loads(freeze_raw)
    assert digest(pathlib.Path(__file__).read_bytes()) == freeze["fixture_sha256"], "collector differs from freeze"
    assert registry_sha == freeze["registry_sha256"], "source differs from frozen canonical registry"
    assert CASES == freeze["cases"] and len(CASES) == freeze["expected_count"], "case inventory differs from freeze"
    doc = json.loads(raw)
    expected = expected_values(doc)
    payloads = [payload_for(doc, case) for case in CASES]
    assert [digest(raw) for raw in payloads] == freeze["case_input_sha256"], "case inputs differ from freeze"
    args.out.mkdir(parents=True, exist_ok=False)
    (args.out / "collector.py").write_bytes(pathlib.Path(__file__).read_bytes())
    (args.out / "freeze.json").write_bytes(freeze_raw)
    results = []
    for definition, payload in zip(CASES, payloads):
        case = args.out / definition["name"]
        case.mkdir(exist_ok=False)
        manifest = case / "crates" / "kerotakis-core"
        manifest.mkdir(parents=True)
        data = case / "data" / "registry"
        data.mkdir(parents=True)
        out = case / "generated"
        out.mkdir()
        (data / "registry-source-v1.json").write_bytes(payload)
        code, stdout, stderr, launch_error = None, b"", b"", None
        try:
            proc = subprocess.run([str(binary)], cwd=manifest,
                env=dict(os.environ, CARGO_MANIFEST_DIR=str(manifest.resolve()), OUT_DIR=str(out.resolve())),
                capture_output=True, timeout=15)
            code, stdout, stderr = proc.returncode, proc.stdout, proc.stderr
        except subprocess.TimeoutExpired as error:
            stdout, stderr = error.stdout or b"", error.stderr or b""
            launch_error = "generator timed out after 15 seconds"
        except OSError as error:
            launch_error = str(error)
        (case / "stdout.log").write_bytes(stdout)
        (case / "stderr.log").write_bytes(stderr)
        (case / "exit-code.json").write_text(json.dumps({"exit_code": code, "launch_error": launch_error}) + "\n")
        failed_reason = launch_error
        if failed_reason is None:
            try:
                check_contract(definition, code, stderr, out, expected)
            except Exception as error:
                failed_reason = type(error).__name__ + ": " + str(error)
        generated = {path.name: digest(path.read_bytes()) for path in sorted(out.iterdir()) if path.is_file()}
        results.append({"case": definition["name"], "expected_refusal": definition["refusal"],
                        "passed": failed_reason is None, "failed_reason": failed_reason,
                        "exit_code": code, "input_sha256": digest(payload),
                        "stdout_sha256": digest(stdout), "stderr_sha256": digest(stderr),
                        "generated_output_sha256": generated,
                        "species_values_and_phases_compared": 186 if failed_reason is None and not definition["refusal"] else 0,
                        "zero_cp_rows_compared": 47 if failed_reason is None and not definition["refusal"] else 0})
    passed = sum(case["passed"] for case in results)
    report = {"total": len(results), "passed": passed, "failed": len(results) - passed, "cases": results,
              "binary_sha256": binary_sha, "registry_sha256": registry_sha,
              "collector_sha256": freeze["fixture_sha256"], "freeze_sha256": digest(freeze_raw),
              "artifact_source_scope": "Supplied hash-bound artifact execution only; this collector does not compile or independently reproduce an artifact or infer source/dependency identity from its path."}
    (args.out / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"total": len(results), "passed": passed, "failed": len(results) - passed}))
    return 0 if passed == len(results) else 1
if __name__ == "__main__":
    raise SystemExit(main())

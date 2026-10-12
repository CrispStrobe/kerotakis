#!/usr/bin/env python3
"""Execute immutable source-informed CLI controls and preserve every outcome."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent


def digest(blob):
    return hashlib.sha256(blob).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def strict_json(blob):
    def reject_constant(value):
        raise ValueError("non-JSON numeric constant: " + value)
    value = json.loads(blob, parse_constant=reject_constant)
    def finite(item):
        if isinstance(item, float):
            require(math.isfinite(item), "nonfinite JSON value")
        elif isinstance(item, dict):
            for child in item.values():
                finite(child)
        elif isinstance(item, list):
            for child in item:
                finite(child)
    finite(value)
    return value


def keys(value, expected):
    require(isinstance(value, dict) and set(value) == set(expected), "output schema differs")


def inspect_json(value, pack):
    keys(value, ["name", "species", "reactions", "elements", "reaction_details"])
    require(value["name"] == pack["name"], "pack description differs")
    require(value["species"] == len(pack["species"]) and value["reactions"] == len(pack["equations"]), "pack counts differ")
    require(value["elements"] == pack["elements"], "elements differ")
    rows = value["reaction_details"]
    require(len(rows) == len(pack["equations"]), "reaction details incomplete")
    for row, identity, equation in zip(rows, pack["reaction_ids"], pack["equations"]):
        keys(row, ["id", "equation", "total_order", "pre_exponential", "temperature_exponent", "activation_energy_j_per_mol", "rate_model", "low_pressure_pre_exponential", "pressure_points_pa", "reversible"])
        require(row["id"] == identity and row["equation"] == equation, "reaction identity/order differs")
        require(row["pre_exponential"] > 0 and row["total_order"] > 0, "invalid rate summary")
        require(isinstance(row["reversible"], bool), "reversibility is not boolean")
        require(row["rate_model"] in ["elementary", "three_body", "troe", "lindemann", "pressure_dependent_arrhenius"], "rate model differs")


def check(case, stdout, stderr, reference_json, manifest):
    contract = case["contract"]
    if contract == "refusal":
        require(not stdout, "refusal emitted a successful payload")
        message = stderr.decode()
        require(all(text in message for text in case["stderr_contains"]), "refusal diagnostic differs")
        return
    require(not stderr, "successful command emitted stderr")
    pack_id = "h2-o2-skeletal-v1" if case["input"] == "complex-note" else case["input"]
    pack = manifest["pack_expectations"][pack_id]
    if contract == "inspect_text":
        lines = stdout.decode().splitlines()
        first = f"{pack['name']}: {len(pack['species'])} species, {len(pack['equations'])} reactions; elements {', '.join(pack['elements'])}"
        require(len(lines) == 1 + len(pack["equations"]) and lines[0] == first, "text header/row count differs")
        for line, identity, equation in zip(lines[1:], pack["reaction_ids"], pack["equations"]):
            require(line.startswith(f"  {identity}  {equation}  [") and "Ea=" in line and " J/mol" in line, "text reaction identity or rate fields differ")
        return
    value = strict_json(stdout)
    if contract in ["inspect_json", "complex_note"]:
        inspect_json(value, pack)
        if contract == "complex_note":
            require(value == reference_json["inspect-h2-o2-skeletal-v1-json"], "ignored note changed full inspect JSON")
        return
    if contract == "rates":
        keys(value, ["mechanism", "volume_litres", "temperature_k", "pressure_pa", "reaction_rates", "species_rates", "rate_determining_step", "rate_determining_criterion"])
        require(value["mechanism"] == pack["name"] and value["volume_litres"] == 1 and value["temperature_k"] == 1500 and value["pressure_pa"] > 0, "rates reactor envelope differs")
        rows = value["reaction_rates"]
        require(len(rows) == len(pack["equations"]), "rates inventory incomplete")
        active = False
        for row, identity, equation in zip(rows, pack["reaction_ids"], pack["equations"]):
            keys(row, ["reaction", "equation", "forward_moles_per_litre_second", "reverse_moles_per_litre_second", "net_moles_per_litre_second"])
            require(row["reaction"] == identity and row["equation"] == equation, "rate identities differ")
            forward, reverse, net = [row[k] for k in ["forward_moles_per_litre_second", "reverse_moles_per_litre_second", "net_moles_per_litre_second"]]
            require(forward >= 0 and reverse >= 0 and math.isclose(net, forward - reverse, rel_tol=1e-12, abs_tol=1e-300), "progress accounting differs")
            active = active or net != 0
        species = value["species_rates"]
        require([row["species"] for row in species] == pack["species"], "species rate inventory/order differs")
        for row in species:
            keys(row, ["species", "net_production_moles_per_litre_second"])
        require(active and value["rate_determining_step"] is not None, "tiny active feed has no active rate")
        step = value["rate_determining_step"]
        keys(step, ["reaction", "equation", "absolute_net_moles_per_litre_second"])
        require(step["reaction"] in pack["reaction_ids"] and step["absolute_net_moles_per_litre_second"] > 0, "rate-determining candidate invalid")
        require(value["rate_determining_criterion"] == "smallest non-zero absolute net progress rate among currently active reactions", "criterion differs")
        return
    require(contract == "simulate_inert", "unknown frozen contract")
    keys(value, ["mechanism", "duration_seconds", "volume_litres", "temperature_k", "initial_pressure_pa", "final_pressure_pa", "initial_moles", "final_moles", "sample_intervals", "samples", "extents", "statistics"])
    require(value["mechanism"] == pack["name"] and value["duration_seconds"] == 0.0001 and value["volume_litres"] == 1 and value["temperature_k"] == 1500 and value["sample_intervals"] == 2, "trajectory envelope differs")
    expected = [dict(species=s, moles=0.000001 if s == "N2" else 0.0) for s in pack["species"]]
    require(value["initial_moles"] == expected and value["final_moles"] == expected and value["extents"] == [], "inert inventory changed")
    require(value["initial_pressure_pa"] > 0 and math.isclose(value["initial_pressure_pa"], value["final_pressure_pa"], rel_tol=1e-12), "inert pressure changed")
    require(len(value["samples"]) == 3, "trajectory sample count differs")
    for sample, seconds in zip(value["samples"], [0, 0.00005, 0.0001]):
        keys(sample, ["elapsed_seconds", "pressure_pa", "moles"])
        require(sample["elapsed_seconds"] == seconds and sample["moles"] == expected and math.isclose(sample["pressure_pa"], value["initial_pressure_pa"], rel_tol=1e-12), "inert sample differs")
    keys(value["statistics"], ["accepted_steps", "rejected_steps", "nonlinear_iterations", "nonlinear_failures", "depletion_events", "constrained_commits"])
    require(all(type(n) is int and n >= 0 for n in value["statistics"].values()), "solver statistics invalid")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--binary-sha256", required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--manifest-sha256", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    require(__debug__, "optimized Python is unsupported")
    manifest_bytes = (HERE / "manifest.json").read_bytes()
    require(digest(manifest_bytes) == args.manifest_sha256, "manifest SHA differs")
    manifest = json.loads(manifest_bytes)
    require(digest(Path(__file__).read_bytes()) == manifest["collector_sha256"], "collector SHA differs")
    require(len(manifest["cases"]) == manifest["expected_count"] == 15 and [c["id"] for c in manifest["cases"]] == manifest["expected_ids"] and len(set(manifest["expected_ids"])) == 15, "frozen case inventory differs")
    binary = args.binary.resolve(strict=True)
    source = args.source.resolve(strict=True)
    require(digest(binary.read_bytes()) == args.binary_sha256, "binary SHA differs")
    for path, wanted in manifest["source_sha256"].items():
        require(digest((source / path).read_bytes()) == wanted, "source bytes differ: " + path)
    payloads = {}
    for identity, entry in manifest["inputs"].items():
        blob = (source / entry["repository_path"]).read_bytes() if "repository_path" in entry else entry["text"].encode()
        require(digest(blob) == entry["sha256"], "input SHA differs: " + identity)
        payloads[identity] = blob
    require(not args.out.exists() and not args.out.is_symlink(), "output already exists")
    out = args.out.resolve()
    require(not out.is_relative_to(source), "evidence must be outside source")
    out.mkdir(parents=True, exist_ok=False)
    (out / "inputs").mkdir()
    (out / "manifest.json").write_bytes(manifest_bytes)
    (out / "collector.py").write_bytes(Path(__file__).read_bytes())
    for identity, blob in payloads.items():
        (out / "inputs" / (identity + ".yaml")).write_bytes(blob)
    results, references = [], {}
    for case in manifest["cases"]:
        case_dir = out / case["id"]
        case_dir.mkdir()
        argv = [str(binary)] + [str(out / "inputs" / (case["input"] + ".yaml")) if arg == "@input" else arg for arg in case["args"]]
        write_json(case_dir / "command.json", dict(argv=argv, cwd=str(source), timeout_seconds=case["timeout_seconds"]))
        timed_out, launch_error, code = False, None, None
        try:
            process = subprocess.run(argv, cwd=source, capture_output=True, timeout=case["timeout_seconds"])
            stdout, stderr, code = process.stdout, process.stderr, process.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr, timed_out = error.stdout or b"", error.stderr or b"", True
        except OSError as error:
            stdout, stderr, launch_error = b"", b"", str(error)
        (case_dir / "stdout.log").write_bytes(stdout)
        (case_dir / "stderr.log").write_bytes(stderr)
        write_json(case_dir / "exit.json", dict(exit_code=code, timeout=timed_out, launch_error=launch_error))
        failure = None
        try:
            require(not timed_out and launch_error is None, "command timed out or did not launch")
            require(code == case["expected_exit"], "exit differs: " + str(code))
            check(case, stdout, stderr, references, manifest)
            if case["contract"] == "inspect_json":
                references[case["id"]] = strict_json(stdout)
        except Exception as error:
            failure = type(error).__name__ + ": " + str(error)
        results.append(dict(id=case["id"], passed=failure is None, failure=failure, exit_code=code, timeout=timed_out, launch_error=launch_error, input_sha256=manifest["inputs"][case["input"]]["sha256"], stdout_sha256=digest(stdout), stderr_sha256=digest(stderr), command_sha256=digest((case_dir / "command.json").read_bytes()), exit_sha256=digest((case_dir / "exit.json").read_bytes())))
    report = dict(source=manifest["source"], source_tree=manifest["source_tree"], binary_sha256=args.binary_sha256, manifest_sha256=args.manifest_sha256, collector_sha256=manifest["collector_sha256"], total=15, passed=sum(r["passed"] for r in results), failed=sum(not r["passed"] for r in results), cases=results, accepted_integration=False, scope=manifest["scope"], limits=manifest["limits"])
    write_json(out / "results.json", report)
    print(json.dumps({k: report[k] for k in ["total", "passed", "failed", "accepted_integration"]}))
    return 0 if report["failed"] == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())

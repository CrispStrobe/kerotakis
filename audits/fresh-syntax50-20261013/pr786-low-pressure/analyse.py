#!/usr/bin/env python3
"""Read preserved CLI streams against frozen boundary acceptance controls."""
import argparse, hashlib, json, math, pathlib, sys


def strict_json(text):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError("duplicate JSON key")
            result[key] = value
        return result
    def number(raw):
        result = float(raw)
        if not math.isfinite(result):
            raise ValueError("nonfinite JSON number")
        return result
    def invalid(raw):
        raise ValueError("invalid JSON constant")
    return json.loads(text, object_pairs_hook=pairs, parse_float=number, parse_constant=invalid)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def vessel(row):
    found = [v for v in row["bench"]["vessels"] if v["id"] == 0]
    if len(found) != 1:
        raise ValueError("missing or duplicate vessel 0")
    return found[0]


def inventory(v, gas_only=False):
    totals = {}
    for portion in v["contents"]:
        amount = portion["moles"]
        if isinstance(amount, bool) or not isinstance(amount, (int, float)) or not math.isfinite(amount) or amount < 0:
            raise ValueError("invalid portion amount")
        if not gas_only or portion["phase"] == "gas":
            key = portion["species"]
            totals[key] = totals.get(key, 0.0) + amount
    return totals


def equal_inventory(before, after):
    return all(math.isclose(before.get(k, 0.0), after.get(k, 0.0), rel_tol=1e-8, abs_tol=1e-12)
               for k in before.keys() | after.keys())


def transition(rows, op):
    matches = [i for i, row in enumerate(rows) if row.get("operator", {}).get("op") == op]
    if len(matches) != 1 or matches[0] == 0:
        raise ValueError("expected one operation with a preceding bench snapshot")
    i = matches[0]
    return vessel(rows[i - 1]), vessel(rows[i]), rows[i]["events"]


def refusal(events):
    return any(e.get("event") == "not_yet_modeled" and e.get("cause") == "boundary-mismatch" for e in events)


def check(case, rows):
    if not rows:
        raise ValueError("empty CLI stream")
    checks = {}
    details = {}
    if case in ("01-sealed-magnesium.lab", "02-sealed-carbon.lab", "06-pressure-controlled-magnesium.lab", "07-swept-magnesium.lab"):
        before, after, events = transition(rows, "ignite")
        fuel = "C" if case == "02-sealed-carbon.lab" else "Mg"
        boundary = "pressure_controlled" if case.startswith("06") else "swept" if case.startswith("07") else "sealed"
        initial = inventory(before); final = inventory(after)
        checks["expected_boundary"] = after["headspace"]["boundary"] == boundary
        checks["typed_boundary_refusal"] = refusal(events)
        checks["fuel_retained"] = math.isclose(initial.get(fuel, 0), final.get(fuel, 0), rel_tol=1e-8, abs_tol=1e-12) and initial.get(fuel, 0) > 0
        checks["no_thermal_equilibrium"] = not any(e.get("event") == "thermal_equilibrium" for e in events)
        checks["no_ignition_exhaust"] = not any(e.get("event") == "gas_evolved" and e.get("moles", 0) > 1e-12 for e in events)
        if boundary != "swept":
            gases_before = inventory(before, True); gases_after = inventory(after, True)
            checks["gas_inventory_retained"] = bool(gases_before) and equal_inventory(gases_before, gases_after)
            checks["positive_pressure"] = after["pressure"] > 0
            details["gases_before"] = gases_before; details["gases_after"] = gases_after
        final_snapshot = vessel(rows[-1])
        checks["fuel_retained_through_observation"] = math.isclose(inventory(final_snapshot).get(fuel, 0), initial.get(fuel, 0), rel_tol=1e-8, abs_tol=1e-12)
        if boundary != "swept":
            checks["gas_retained_through_observation"] = equal_inventory(inventory(before, True), inventory(final_snapshot, True))
            checks["observed_pressure_positive"] = final_snapshot["pressure"] > 0
        if boundary == "pressure_controlled":
            checks["pressure_held_at_1bar"] = math.isclose(after["pressure"], 100000, rel_tol=1e-8)
        if boundary == "swept":
            checks["no_mgo_production"] = final.get("MgO", 0) <= initial.get("MgO", 0) + 1e-12
        details["fuel_before"] = initial.get(fuel, 0); details["fuel_after"] = final.get(fuel, 0)
    elif case == "03-open-magnesium.lab":
        before, after, events = transition(rows, "ignite")
        checks["open_boundary"] = after["headspace"]["boundary"] == "open"
        checks["thermal_equilibrium"] = any(e.get("event") == "thermal_equilibrium" for e in events)
        checks["no_boundary_refusal"] = not refusal(events)
        checks["mgo_formed"] = inventory(after).get("MgO", 0) > 0.0019
    elif case == "04-inert-gas-heating.lab":
        before, after, events = transition(rows, "heat")
        checks["sealed_boundary"] = after["headspace"]["boundary"] == "sealed"
        checks["gas_inventory_retained"] = bool(inventory(before, True)) and equal_inventory(inventory(before, True), inventory(after, True))
        checks["fixed_volume"] = before["headspace"]["volume"] == after["headspace"]["volume"]
        checks["warming"] = after["temperature"] > before["temperature"] > 0
        checks["positive_pressures"] = before["pressure"] > 0 and after["pressure"] > 0
        if not checks["positive_pressures"]:
            checks["pressure_temperature_ratio"] = False
        else:
            details["pressure_ratio"] = after["pressure"] / before["pressure"]
            details["temperature_ratio"] = after["temperature"] / before["temperature"]
            checks["pressure_temperature_ratio"] = math.isclose(details["pressure_ratio"], details["temperature_ratio"], rel_tol=1e-6)
        checks["no_thermal_equilibrium"] = not any(e.get("event") == "thermal_equilibrium" for e in events)
        checks["no_boundary_refusal"] = not refusal(events)
    elif case == "05-warm-methane.lab":
        before, after, events = transition(rows, "heat")
        if not (500 <= after["temperature"] < 810.15):
            return {"classification": "inconclusive", "reason": "frozen heat input missed the required temperature interval", "temperature_k": after["temperature"]}
        checks["sealed_boundary"] = after["headspace"]["boundary"] == "sealed"
        checks["below_autoignition_diagnostic"] = any(e.get("event") == "below_autoignition" and e.get("fuel") == "methane" for e in events)
        checks["gas_inventory_retained"] = bool(inventory(before, True)) and equal_inventory(inventory(before, True), inventory(after, True))
        checks["methane_retained"] = math.isclose(inventory(after).get("methane", 0), 0.01, rel_tol=1e-8)
        checks["no_thermal_equilibrium"] = not any(e.get("event") == "thermal_equilibrium" for e in events)
        checks["no_boundary_refusal"] = not refusal(events)
        details["temperature_k"] = after["temperature"]
    else:
        raise ValueError("unknown frozen control")
    return {"classification": "pass" if all(checks.values()) else "fail", "checks": checks, "details": details}


def main():
    if sys.flags.optimize:
        raise SystemExit("optimized Python would disable required checks")
    ap = argparse.ArgumentParser()
    ap.add_argument("--raw", type=pathlib.Path, required=True)
    ap.add_argument("--out", type=pathlib.Path, required=True)
    a = ap.parse_args()
    root = pathlib.Path(__file__).parent
    protocol = strict_json((root / "protocol.json").read_text())
    for name, digest in protocol["files"].items():
        assert sha(root / name) == digest, "frozen input changed"
    binding = strict_json((a.raw / "binding.json").read_text())
    assert binding["protocol_sha256"] == sha(root / "protocol.json")
    assert binding["analyzer_sha256"] == sha(pathlib.Path(__file__))
    assert binding["source_head"] == protocol["source_head"] and binding["source_tree"] == protocol["source_tree"]
    assert sha(a.raw / "artifact-review.json") == binding["artifact_review_receipt_sha256"]
    artifact = strict_json((a.raw / "artifact-review.json").read_text())
    assert artifact["all_checks_pass"] is True
    assert artifact["candidate"]["head"] == protocol["source_head"]
    assert artifact["candidate"]["tree"] == protocol["source_tree"]
    assert artifact["candidate"]["binary_sha256"] == binding["binary_sha256"]
    assert artifact["candidate"]["lock_sha256"] == binding["lock_sha256"]
    assert artifact["run"] == protocol["own_cli_workflow_run"] and artifact["build_job"] == protocol["own_cli_build_job"]
    assert artifact["tested_parents"] == [protocol["base"], protocol["source_head"]]
    assert artifact["artifact_id"] == binding["artifact_id"] and artifact["artifact_zip_sha256"] == binding["artifact_zip_sha256"]
    cases = sorted(n.removeprefix("scripts/") for n in protocol["files"] if n.endswith(".lab"))
    results = []
    for i, case in enumerate(cases, 1):
        record = strict_json((a.raw / f"{i:02d}.result.json").read_text())
        stdout = a.raw / f"{i:02d}.stdout.jsonl"; stderr = a.raw / f"{i:02d}.stderr.txt"
        assert record["case"] == "scripts/" + case and record["script_sha256"] == protocol["files"]["scripts/" + case]
        assert sha(stdout) == record["stdout_sha256"] and sha(stderr) == record["stderr_sha256"]
        if record["exit_code"] != 0 or record["timeout"] or stderr.stat().st_size:
            result = {"classification": "fail", "reason": "CLI exit, timeout or stderr prevented acceptance"}
        else:
            if stdout.stat().st_size > 4 * 1024 * 1024:
                raise ValueError("oversized stream")
            result = check(case, [strict_json(s) for s in stdout.read_text().splitlines()])
        results.append(dict(result, case=case, stdout_sha256=record["stdout_sha256"], stderr_sha256=record["stderr_sha256"]))
    report = {"cases": results, "all_pass": all(r["classification"] == "pass" for r in results),
              "source_head": binding["source_head"], "source_tree": binding["source_tree"],
              "binary_sha256": binding["binary_sha256"], "protocol_sha256": binding["protocol_sha256"],
              "analyzer_sha256": sha(pathlib.Path(__file__)), "runtime_binding_sha256": sha(a.raw / "binding.json"),
              "scope": "Source-informed narrow repair acceptance; original physical predictions remain separate.",
              "empirical_calibration": False, "closed_chemistry_implemented": False}
    with a.out.open("x") as f:
        f.write(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"all_pass": report["all_pass"], "cases": len(results)}))
    return 0 if report["all_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

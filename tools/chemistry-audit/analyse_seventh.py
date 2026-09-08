#!/usr/bin/env python3
"""Independent and metamorphic checks for seventh-fleet evidence."""
import argparse
import json
import math
from pathlib import Path
import tempfile


def close(a, b, absolute=1e-9, relative=1e-5):
    return math.isfinite(a) and math.isfinite(b) and abs(a - b) <= absolute + relative * max(abs(a), abs(b))


def analyse(directory):
    summary = json.loads((directory / "summary.json").read_text())
    records = {int(row["id"].split("-")[0]): row for row in summary["cases"]}
    if len(records) != 30 or set(records) != set(range(459, 489)):
        raise ValueError("exactly 30 distinct cases 459-488 required")
    rows = {}
    malformed = {}
    for key, record in records.items():
        rows[key], malformed[key] = [], []
        for line in (directory / record["id"] / "stdout.ndjson").read_text().splitlines():
            try:
                value = json.loads(line)
                if not isinstance(value, dict):
                    raise ValueError("row is not an object")
                rows[key].append(value)
            except ValueError as error:
                malformed[key].append(str(error))

    checks = []

    def check(name, function, kind="model-law"):
        try:
            passed, evidence = function()
        except (KeyError, IndexError, StopIteration, TypeError, ValueError, ZeroDivisionError) as error:
            passed, evidence = False, {"missing_or_invalid_output": str(error)}
        checks.append({"check": name, "kind": kind, "passed": bool(passed), "evidence": evidence})

    def events(key, name):
        return [event for row in rows[key] for event in row.get("events", []) if event["event"] == name]

    def final(key):
        return records[key]["final"]["vessels"][0]

    def before(key, operator):
        index = next(i for i, row in enumerate(rows[key]) if row.get("operator", {}).get("op") == operator)
        return next(row["bench"]["vessels"][0] for row in reversed(rows[key][:index]) if row.get("bench"))

    def amount(key, species, phase=None):
        return sum(
            portion["moles"] for portion in final(key)["contents"]
            if portion["species"] == species and (phase is None or portion["phase"] == phase)
        )

    def vessel_amount(vessel, species):
        return sum(portion["moles"] for portion in vessel["contents"] if portion["species"] == species)

    for key, record in records.items():
        def execution(key=key, record=record):
            solver_failures = events(key, "solver_failed")
            valid = (
                record["exit"] == 0 and rows[key] and not record["non_json_lines"]
                and not malformed[key] and record["final"] and not solver_failures
                and all(
                    math.isfinite(portion["moles"]) and portion["moles"] >= 0
                    for vessel in record["final"]["vessels"] for portion in vessel["contents"]
                )
            )
            return valid, {"exit": record["exit"], "rows": len(rows[key]), "malformed": malformed[key], "solver_failures": solver_failures}
        check(f"{key}: complete finite execution", execution, "execution")

    for key in range(459, 465):
        def ammonia_partition(key=key):
            total = amount(key, "NH3") + amount(key, "NH4+")
            gas = amount(key, "NH3", "gas")
            partition = events(key, "headspace_partitioned")[-1]
            test = events(key, "gas_tested")[-1]
            expected_positive = key not in (462, 463)
            valid = close(total, .005 if key == 459 else .01) and 0 < gas <= total and partition["species"] == "NH3" and test["positive"] is expected_positive
            return valid, {"total_mol": total, "gas_mol": gas, "gas_fraction": gas / total, "test": test}
        check(f"{key}: conserved finite ammonia partition and threshold-consistent litmus", ammonia_partition, "conservation")

    check("459/460: dose ordering", lambda: (amount(460, "NH3", "gas") > amount(459, "NH3", "gas"), {key: amount(key, "NH3", "gas") for key in (459, 460)}))
    check("460/461/462: water-retention ordering", lambda: (
        amount(460, "NH3", "gas") / amount(460, "NH3")
        > amount(461, "NH3", "gas") / amount(461, "NH3")
        > amount(462, "NH3", "gas") / amount(462, "NH3"),
        {key: amount(key, "NH3", "gas") / amount(key, "NH3") for key in (460, 461, 462)},
    ))
    check("463/464: temperature partition ordering", lambda: (
        amount(464, "NH3", "gas") > amount(463, "NH3", "gas"),
        {key: {"temperature_k": final(key)["temperature"], "gas_mol": amount(key, "NH3", "gas")} for key in (463, 464)},
    ))

    for key in range(465, 473):
        def ester_equilibrium(key=key):
            reactions = events(key, "org_reacted")
            initial = before(key, "react")
            extent = reactions[0]["extent"]
            n = {species: vessel_amount(initial, species) for species in ("CH3COOH", "ethanol", "ethyl_acetate", "water")}
            settled = {
                "CH3COOH": n["CH3COOH"] - extent,
                "ethanol": n["ethanol"] - extent,
                "ethyl_acetate": n["ethyl_acetate"] + extent,
                "water": n["water"] + extent,
            }
            quotient = settled["ethyl_acetate"] * settled["water"] / (settled["CH3COOH"] * settled["ethanol"])
            expected_events = 2 if key == 470 else 1
            return len(reactions) == expected_events and abs(quotient - 4.0) < 1e-4 and all(event.get("boundary") for event in reactions), {"initial": n, "settled": settled, "Q": quotient, "events": reactions}
        check(f"{key}: declared ester equilibrium", ester_equilibrium)
    check("465/468: extensive ester scaling", lambda: (
        all(close(amount(468, species), 2 * amount(465, species)) for species in ("CH3COOH", "ethanol", "ethyl_acetate", "water")),
        {species: [amount(465, species), amount(468, species)] for species in ("CH3COOH", "ethanol", "ethyl_acetate", "water")},
    ))
    check("465/469: ester feed-order invariance", lambda: (
        all(close(amount(465, species), amount(469, species)) for species in ("CH3COOH", "ethanol", "ethyl_acetate", "water")),
        {species: [amount(465, species), amount(469, species)] for species in ("CH3COOH", "ethanol", "ethyl_acetate", "water")},
    ))
    check("470: repeated equilibrium is a no-op", lambda: (
        abs(events(470, "org_reacted")[-1]["extent"]) < 1e-8,
        events(470, "org_reacted"),
    ))

    expected_tests = {473: True, 474: False, 475: True, 476: False, 477: True, 478: False, 479: True, 480: False}
    for key, expected in expected_tests.items():
        def gas_verdict(key=key, expected=expected):
            verdicts = events(key, "gas_tested")
            return len(verdicts) == 1 and verdicts[0]["positive"] is expected, verdicts
        check(f"{key}: threshold verdict is {expected}", gas_verdict, "curated-threshold")

    gas_constant = 8314.46261815324
    for key in range(481, 489):
        def ideal_pressure(key=key):
            vessel = final(key)
            if key in (486, 487):
                readings = events(key, "measured")
                return len(readings) == 2 and all(reading["unit"] == "kPa" for reading in readings) and readings[0]["value"] > 101.325 and close(readings[-1]["value"], 101.325, absolute=.01), readings
            gas = sum(portion["moles"] for portion in vessel["contents"] if portion["phase"] == "gas")
            expected = gas * gas_constant * vessel["temperature"] / vessel["headspace"]["volume"] / 1000
            reading = events(key, "measured")[-1]
            return reading["unit"] == "kPa" and close(reading["value"], expected, absolute=.01), {"measured_kpa": reading["value"], "ideal_kpa": expected}
        check(f"{key}: pressure/vent relation", ideal_pressure)
    check("481/485: extensive pressure invariance", lambda: (close(events(481, "measured")[-1]["value"], events(485, "measured")[-1]["value"], absolute=.01), [events(481, "measured")[-1], events(485, "measured")[-1]]))
    check("481/482/488: pressure dose ordering", lambda: (
        events(482, "measured")[-1]["value"] > events(481, "measured")[-1]["value"] > events(488, "measured")[-1]["value"],
        {key: events(key, "measured")[-1]["value"] for key in (481, 482, 488)},
    ))
    check("484: heating raises pressure", lambda: (
        events(484, "measured")[-1]["value"] > events(484, "measured")[0]["value"],
        events(484, "measured"),
    ))

    return {"fleet": "seventh-459-488", "cases": len(records), "checks": checks, "passed": sum(row["passed"] for row in checks), "unmet": sum(not row["passed"] for row in checks)}


def self_test():
    import seventh_batch
    cases = seventh_batch.recorder.CASES
    assert len(cases) == len({case["id"] for case in cases}) == len({case["script"] for case in cases}) == 30
    with tempfile.TemporaryDirectory(prefix="kero-seventh-negative-") as temporary:
        directory = Path(temporary)
        records = []
        for case in cases:
            source = directory / case["id"]
            source.mkdir()
            (source / "stdout.ndjson").write_text("")
            records.append({"id": case["id"], "exit": 0, "non_json_lines": [], "final": None})
        (directory / "summary.json").write_text(json.dumps({"cases": records}))
        result = analyse(directory)
        assert result["passed"] == 0 and result["unmet"] >= 30
    print(json.dumps({"distinct_inputs": 30, "missing_output_negative_control": "passed", "rejected_checks": result["unmet"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, nargs="?")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
    else:
        if args.directory is None:
            parser.error("directory required")
        result = analyse(args.directory)
        text = json.dumps(result, indent=2, allow_nan=False) + "\n"
        if args.out:
            with args.out.open("x") as output:
                output.write(text)
        print(text, end="")
        raise SystemExit(bool(result["unmet"]))

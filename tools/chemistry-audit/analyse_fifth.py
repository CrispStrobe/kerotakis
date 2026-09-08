#!/usr/bin/env python3
"""Independent fifth-fleet laws; completed contract summarized in HISTORY.md."""
import argparse
import json
import math
import pathlib
import re
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]


def close(a, b, absolute=1e-8, relative=1e-6):
    return math.isfinite(a) and math.isfinite(b) and abs(a - b) <= absolute + relative * max(abs(a), abs(b))


def analyse(directory):
    summary = json.loads((directory / "summary.json").read_text())
    records = {int(r["id"].split("-", 1)[0]): r for r in summary["cases"]}
    if len(summary["cases"]) != 36 or set(records) != set(range(159, 195)):
        raise ValueError("Exactly 36 distinct cases 159–194 are required")
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text())
    composition = {r["species_id"]: {e["element"]: e["count"]["value"] for e in r["elements"]}
                   for r in registry["compositions"]}
    rows, scripts, malformed = {}, {}, {}
    for k, record in records.items():
        source = directory / record["id"]
        rows[k], malformed[k] = [], []
        try:
            scripts[k] = (source / "experiment.lab").read_text()
            lines = (source / "stdout.ndjson").read_text().splitlines()
        except OSError as exc:
            scripts[k], lines = "", []
            malformed[k].append(str(exc))
        for line in lines:
            try:
                row = json.loads(line)
                if not isinstance(row, dict):
                    raise ValueError("NDJSON row must be an object")
                rows[k].append(row)
            except ValueError:
                malformed[k].append(line)
    checks = []

    def check(name, calculate, kind="model-law"):
        try:
            passed, evidence = calculate()
        except (KeyError, IndexError, StopIteration, TypeError, ValueError, ZeroDivisionError, OverflowError) as exc:
            passed, evidence = False, {"missing_or_invalid_output": f"{type(exc).__name__}: {exc}"}
        checks.append({"check": name, "kind": kind, "passed": bool(passed), "evidence": evidence})

    def final(k, index=0):
        return next(v for v in records[k]["final"]["vessels"] if v["id"] == index)

    def events(k, name):
        return [e for row in rows[k] for e in row.get("events", []) if e["event"] == name]

    def atoms(v, element, phase=None):
        return sum(p["moles"] * composition[p["species"]].get(element, 0)
                   for p in v["contents"] if phase is None or p["phase"] == phase)

    def amount(v, key):
        return sum(p["moles"] for p in v["contents"] if p["species"] == key)

    def feed(k):
        result = {}
        for key, n in re.findall(r"^add v\d+ (\S+) ([0-9.eE+-]+)mol$", scripts[k], re.M):
            result[key] = result.get(key, 0) + float(n)
        return result

    def before(k, op):
        index = next(i for i, row in enumerate(rows[k]) if row.get("operator", {}).get("op") == op)
        return index, next(v for row in reversed(rows[k][:index]) if "bench" in row
                           for v in row["bench"]["vessels"] if v["id"] == 0)

    for k, record in records.items():
        def execution(k=k, record=record):
            last = next(row["bench"] for row in reversed(rows[k]) if "bench" in row)
            vessels = record["final"]["vessels"]
            valid = (record["exit"] == 0 and not record["non_json_lines"] and not malformed[k]
                     and bool(scripts[k]) and bool(vessels) and last == record["final"]
                     and not events(k, "solver_failed")
                     and all(p["species"] in composition and math.isfinite(p["moles"]) and p["moles"] >= 0
                             for v in vessels for p in v["contents"]))
            return valid, {"exit": record["exit"], "solver_failures": events(k, "solver_failed"),
                           "malformed": malformed[k], "stderr": record["stderr"]}
        check(f"{k}: complete execution and finite registered inventory", execution, "execution")

        if k < 189:
            def conservation(k=k):
                expected = {}
                for key, n in feed(k).items():
                    for element, count in composition[key].items():
                        if element in {"Ag", "Cl", "Na", "Zn", "K", "Ca", "Mg"}:
                            expected[element] = expected.get(element, 0) + n * count
                if 171 <= k <= 176:
                    event = events(k, "titrated")[-1]
                    expected["K"] = expected.get("K", 0) + event["concentration"] * event["total_volume"]
                actual = {e: sum(atoms(v, e) for v in records[k]["final"]["vessels"]) for e in expected}
                return bool(expected) and all(close(actual[e], n) for e, n in expected.items()), {
                    "feed_mol_atoms": expected, "final_mol_atoms": actual}
            check(f"{k}: supplied nonvolatile element conservation", conservation, "conservation")

    def solid(k, element):
        return atoms(final(k), element, "solid")

    for k in range(159, 165):
        def capacity(k=k):
            supplied = feed(k)
            limit = min(supplied["AgNO3"], supplied["NaCl"])
            n = solid(k, "Ag")
            return .9 * limit < n <= limit + 1e-8, {"solid_Ag_mol": n, "limiting_reagent_mol": limit}
        check(f"{k}: silver chloride limiting-reagent capacity", capacity)
    for k, scale in ((160, 1), (162, 4)):
        check(f"159/{k}: silver order or extensive scaling", lambda k=k, scale=scale:
              (close(solid(159, "Ag"), solid(k, "Ag") / scale, relative=1e-4), [solid(159, "Ag"), solid(k, "Ag") / scale]))
    check("159/163: dilution cannot increase silver solid", lambda:
          (solid(163, "Ag") <= solid(159, "Ag") + 1e-8, [solid(159, "Ag"), solid(163, "Ag")]))

    for k in range(165, 170):
        def zinc_capacity(k=k):
            limit = min(feed(k)["ZnSO4"], feed(k)["KOH"] / 2)
            n = solid(k, "Zn")
            return 0 < n <= limit + 1e-8, {"solid_Zn_mol": n, "hydroxide_capacity_mol": limit}
        check(f"{k}: zinc positive precipitation within hydroxide capacity", zinc_capacity)
    for k, scale in ((166, 1), (169, 3)):
        check(f"165/{k}: zinc order or extensive scaling", lambda k=k, scale=scale:
              (close(solid(165, "Zn"), solid(k, "Zn") / scale, relative=1e-4), [solid(165, "Zn"), solid(k, "Zn") / scale]))
    check("165/167/168/170: zinc dose ordering and acid reversal", lambda:
          (0 < solid(168, "Zn") < solid(165, "Zn") < solid(167, "Zn")
           and solid(170, "Zn") < .1 * solid(165, "Zn"), {str(k): solid(k, "Zn") for k in (165, 167, 168, 170)}))

    for k in range(171, 177):
        def capacity(k=k):
            event = events(k, "titrated")[-1]
            supplied = feed(k)
            expected = 2 * supplied["H2SO4"] - supplied.get("KOH", 0)
            measured = event["total_volume"] * event["concentration"]
            return (event["titrant"] == "KOH" and event["endpoint_reached"] is True
                    and abs(event["final_ph"] - 7) < .02
                    and close(measured, expected, absolute=2e-7, relative=2e-3)), {
                        "expected_KOH_mol": expected, "delivered_KOH_mol": measured,
                        "volume_L": event["total_volume"], "pH": event["final_ph"]}
        check(f"{k}: diprotic acid endpoint and remaining equivalents", capacity)

    faraday = 6.02214076e23 * 1.602176634e-19
    for k in range(177, 183):
        def electrolysis(k=k):
            doses = re.findall(r"^electrolyse v1 ([0-9.eE+-]+)A ([0-9.eE+-]+)s$", scripts[k], re.M)
            charge = sum(float(a) * float(t) for a, t in doses)
            product = {}
            es = events(k, "electrolysed")
            for event in es:
                for pole in ("anode", "cathode"):
                    key = event[pole + "_species"]
                    product[key] = product.get(key, 0) + event[pole + "_moles"]
            return (bool(doses) and len(es) == len(doses)
                    and close(sum(e["coulombs"] for e in es), charge)
                    and close(product["H2"], charge / (2 * faraday), absolute=1e-10)
                    and close(product["O2"], charge / (4 * faraday), absolute=1e-10)), {
                        "charge_C": charge, "electrode_product_mol": product}
        check(f"{k}: independent SI Faraday yields at both electrodes", electrolysis)

        def sealed(k=k):
            index, initial = before(k, "electrolyse")
            v = final(k)
            gas = sum(p["moles"] for p in v["contents"] if p["phase"] == "gas")
            ideal = gas * 8314.46261815324 * v["temperature"] / v["headspace"]["volume"]
            differences = {e: atoms(v, e) - atoms(initial, e) for e in ("H", "O", "N", "K")}
            flows = [e for row in rows[k][index:] for e in row.get("events", [])
                     if e["event"] in ("gas_evolved", "gas_absorbed") and e["moles"] > 1e-8]
            return (v["headspace"]["boundary"] == "sealed" and gas > 0 and not flows
                    and close(v["pressure"], ideal, absolute=.001)
                    and all(close(atoms(v, e), atoms(initial, e)) for e in differences)), {
                        "atom_changes_mol": differences, "external_flows": flows,
                        "pressure_Pa": v["pressure"], "ideal_Pa": ideal}
        check(f"{k}: sealed atom inventory and actual-headspace gas law", sealed, "conservation")
    check("177/179/180/182: equal-charge, split-charge and extensive pressure controls", lambda:
          (all(close(final(177)["pressure"], final(k)["pressure"], absolute=.1, relative=1e-5)
               for k in (179, 180, 182)), {str(k): final(k)["pressure"] for k in (177, 179, 180, 182)}))

    for k, scale in ((183, 1), (184, 1), (185, 1), (186, 4), (187, 1), (188, .5)):
        def receiver(k=k, scale=scale):
            expected = {"Ca": .0006 * .4 * scale, "Cl": .0006 * .8 * scale,
                        "Mg": .0005 * .6 * scale, "Na": .0007 * .8 * scale}
            actual = {e: atoms(final(k, 4), e) for e in expected}
            return all(close(actual[e], n) for e, n in expected.items()), {"expected_mol_atoms": expected, "receiver_mol_atoms": actual}
        check(f"{k}: three-feed fractional receiver ledger", receiver, "conservation")
    check("183/184/185/186/188: grouping and concentration preserve receiver pH", lambda:
          (all(abs(final(183, 4)["solution"]["ph"] - final(k, 4)["solution"]["ph"]) < .03 for k in (184, 185, 186, 188)),
           {str(k): final(k, 4)["solution"]["ph"] for k in (183, 184, 185, 186, 188)}))

    extents = {}
    for k in range(189, 195):
        def organic(k=k):
            _index, initial = before(k, "react")
            keys = ("CH3COOH", "ethanol", "ethyl_acetate", "water")
            n = [amount(initial, key) for key in keys]
            es = [e for e in events(k, "org_reacted") if e["name"] == "esterification"]
            x = sum(e["extent"] for e in es)
            extents[k] = x
            after = [n[0] - x, n[1] - x, n[2] + x, n[3] + x]
            q = after[2] * after[3] / (after[0] * after[1])
            return (bool(es) and all(e.get("boundary") for e in es) and x > 0
                    and all(v > 0 for v in after) and abs(q - 4) < 1e-4
                    and all(close(atoms(initial, e), atoms(final(k), e)) for e in ("C", "H", "O"))), {
                        "initial_mol": dict(zip(keys, n)), "extent_mol": x, "Q": q,
                        "boundaries": [e.get("boundary") for e in es]}
        check(f"{k}: current-state organic mass action and conserved atoms", organic)
    for k, scale in ((190, 1), (191, 3), (192, 1)):
        check(f"189/{k}: organic order, scale or repeated-request idempotence", lambda k=k, scale=scale:
              (close(extents[189], extents[k] / scale, absolute=1e-9, relative=1e-4), [extents[189], extents[k] / scale]))
    check("189/193/194: independent reactant/product perturbation directions", lambda:
          (0 < extents[194] < extents[189] < extents[193], {str(k): extents[k] for k in (189, 193, 194)}))
    return {"fleet": "fifth-159-194", "binary_sha256": summary.get("binary_sha256"), "checks": checks,
            "passed": sum(c["passed"] for c in checks), "unmet": sum(not c["passed"] for c in checks)}


def self_test():
    """A wholly absent observation stream must not pass any physical check."""
    import fifth_batch
    cases = fifth_batch.recorder.CASES
    assert len(cases) == len({c["script"] for c in cases}) == 36
    with tempfile.TemporaryDirectory(prefix="kero-fifth-missing-output-") as temporary:
        directory = pathlib.Path(temporary)
        records = []
        for case in cases:
            source = directory / case["id"]
            source.mkdir()
            (source / "experiment.lab").write_text(case["script"])
            (source / "stdout.ndjson").write_text("")
            records.append({"id": case["id"], "exit": 0, "non_json_lines": [], "final": None, "stderr": ""})
        (directory / "summary.json").write_text(json.dumps({"cases": records}))
        report = analyse(directory)
        assert report["passed"] == 0 and report["unmet"] >= 36, report
    print(json.dumps({"distinct_inputs": 36, "missing_output_negative_control": "passed", "rejected_checks": report["unmet"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=pathlib.Path, nargs="?")
    parser.add_argument("--out", type=pathlib.Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
    else:
        if args.directory is None:
            parser.error("directory is required unless --self-test is given")
        report = analyse(args.directory)
        encoded = json.dumps(report, indent=2, allow_nan=False) + "\n"
        if args.out:
            with args.out.open("x") as handle:
                handle.write(encoded)
        print(encoded, end="")
        raise SystemExit(bool(report["unmet"]))

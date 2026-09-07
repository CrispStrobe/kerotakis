#!/usr/bin/env python3
"""Predeclared fourth-fleet laws and matched controls; no output goldens."""
import argparse
import json
import math
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]


def analyse(directory):
    summary = json.loads((directory / "summary.json").read_text())
    records = {int(r["id"].split("-", 1)[0]): r for r in summary["cases"]}
    if len(summary["cases"]) != 36 or set(records) != set(range(123, 159)):
        raise ValueError("Exactly 36 distinct cases 123–158 are required")
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text())
    composition = {r["species_id"]: {e["element"]: e["count"]["value"] for e in r["elements"]}
                   for r in registry["compositions"]}
    rows, scripts, malformed = {}, {}, {}
    for k, record in records.items():
        source = directory / record["id"]
        scripts[k] = (source / "experiment.lab").read_text()
        rows[k], malformed[k] = [], []
        for line in (source / "stdout.ndjson").read_text().splitlines():
            try:
                row = json.loads(line)
                if not isinstance(row, dict):
                    raise ValueError("JSON row is not an object")
                rows[k].append(row)
            except ValueError:
                malformed[k].append(line)
    checks = []

    def check(name, calculation, kind="model-law"):
        try:
            passed, evidence = calculation()
        except (KeyError, IndexError, StopIteration, TypeError, ValueError, ZeroDivisionError, OverflowError) as exc:
            passed, evidence = False, {"missing_or_invalid_output": f"{type(exc).__name__}: {exc}"}
        checks.append({"check": name, "kind": kind, "passed": bool(passed), "evidence": evidence})

    def close(a, b, absolute=1e-8, relative=1e-6):
        return math.isfinite(a) and math.isfinite(b) and abs(a - b) <= absolute + relative * max(abs(a), abs(b))

    def final(k, index=0):
        return next(v for v in records[k]["final"]["vessels"] if v["id"] == index)

    def events(k, name):
        return [e for row in rows[k] for e in row.get("events", []) if e["event"] == name]

    def amount(vessel, key):
        return sum(p["moles"] for p in vessel["contents"] if p["species"] == key)

    def atoms(vessel, element, phase=None):
        return sum(p["moles"] * composition[p["species"]].get(element, 0)
                   for p in vessel["contents"] if phase is None or p["phase"] == phase)

    def ph(k):
        return final(k)["solution"]["ph"]

    def prior(k, op):
        index = next(i for i, row in enumerate(rows[k]) if row.get("operator", {}).get("op") == op)
        return index, next(row["bench"] for row in reversed(rows[k][:index]) if "bench" in row)

    def feed(k):
        amounts = {}
        for key, n in re.findall(r"^add v\d+ (\S+) ([0-9.eE+-]+)mol$", scripts[k], re.M):
            amounts[key] = amounts.get(key, 0) + float(n)
        return amounts

    for k, record in records.items():
        check(f"{k}: execution and finite final inventory", lambda k=k, record=record:
              (record["exit"] == 0 and not record["non_json_lines"] and not malformed[k]
               and record["final"] is not None and not events(k, "solver_failed")
               and all(p["species"] in composition and math.isfinite(p["moles"]) and p["moles"] >= 0
                       for v in record["final"]["vessels"] for p in v["contents"]),
               {"exit": record["exit"], "solver_failures": events(k, "solver_failed"), "stderr": record["stderr"]}), "execution")

        def nonvolatile(k=k):
            expected = {}
            for key, n in feed(k).items():
                for element, count in composition[key].items():
                    if element in {"Na", "K", "Cl", "Ba", "Mg"}:
                        expected[element] = expected.get(element, 0) + n * count
            actual = {e: sum(atoms(v, e) for v in records[k]["final"]["vessels"]) for e in expected}
            return all(close(n, actual[e]) for e, n in expected.items()), {
                "expected_mol_atoms": expected, "actual_mol_atoms": actual}
        if any(e in composition[key] for key in feed(k) for e in ("Na", "K", "Cl", "Ba", "Mg")):
            check(f"{k}: supplied nonvolatile element inventory", nonvolatile, "conservation")

    latent = {}
    for line in (ROOT / "data/thermo/uscg-chris-still.tsv").read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        key, _tb, j_kg, grams_mol, _source = line.split()
        latent[key] = float(j_kg) * float(grams_mol) / 1e6
    vle_source = (ROOT / "crates/kerotakis-thermo/src/vle.rs").read_text()
    for key in ("water", "ethanol"):
        latent[key] = float(re.search(key.upper() + r"_HVAP_KJ_PER_MOL: f64 = ([0-9.]+)", vle_source)[1])

    for k in range(123, 129):
        def cut(k=k):
            initial = feed(k)
            receiver = {key: amount(final(k, 1), key) for key in initial}
            event = events(k, "distilled")[-1]
            valid = all(0 < receiver[key] < n and close(amount(final(k), key) + receiver[key], n)
                        for key, n in initial.items())
            valid &= len(event["components"]) == len(initial) and "approximation" in event["model"]
            valid &= "azeotropes" in event["model"] and "excluding reflux" in event["model"]
            if k in (126, 127):
                calculated = sum(receiver[key] * latent[key] for key in initial)
                valid &= close(calculated, k - 125) and close(calculated, event["energy_kj"])
            else:
                fraction = 0.1 if k == 128 else 0.12
                valid &= close(sum(receiver.values()), fraction * sum(initial.values()))
            return valid, {"initial_mol": initial, "receiver_mol": receiver, "reported_latent_kj": event["energy_kj"], "model": event["model"]}
        check(f"{k}: componentwise cut and latent/molar budget", cut)
    for other, scale in ((124, 0.1), (125, 1)):
        check(f"123/{other}: still order/scale control", lambda other=other, scale=scale:
              (all(close(amount(final(123, 1), key), amount(final(other, 1), key) / scale, relative=1e-5)
                   for key in feed(123)), {key: [amount(final(123, 1), key), amount(final(other, 1), key) / scale] for key in feed(123)}))
    check("126/127: greater latent budget withdraws more of every component", lambda:
          (all(amount(final(127, 1), key) > amount(final(126, 1), key) for key in feed(126)),
           {key: [amount(final(126, 1), key), amount(final(127, 1), key)] for key in feed(126)}))

    for k, volume, scale in ((129, .25, 1), (130, .25, 1), (131, 2.5, 10), (132, 1, 1), (133, 1, 1)):
        ideal = -math.log10(.0006 * scale / volume)
        check(f"{k}: residual strong-acid concentration law", lambda k=k, ideal=ideal:
              (abs(ph(k) - ideal) < .08, {"pH": ph(k), "ideal_pH": ideal}))
    for a, b in ((129, 130), (129, 131), (132, 133)):
        check(f"{a}/{b}: neutralization path/scale pH", lambda a=a, b=b: (abs(ph(a) - ph(b)) < .02, [ph(a), ph(b)]))
    check("129/130: acid/base order thermal state", lambda:
          (abs(final(129)["temperature"] - final(130)["temperature"]) < .02,
           [final(129)["temperature"], final(130)["temperature"]]))
    check("129/132: logarithmic residual-acid dilution", lambda:
          (abs(ph(132) - ph(129) - math.log10(4)) < .08, [ph(129), ph(132)]))
    check("134: remaining-equivalent endpoint", lambda: (6.5 < ph(134) < 7.5, {"pH": ph(134)}))

    def solid(k, element):
        return atoms(final(k), element, "solid")

    def phases(k, element):
        return [{"species": p["species"], "metal_mol_atoms": p["moles"] * composition[p["species"]].get(element, 0)}
                for p in final(k)["contents"] if p["phase"] == "solid" and composition[p["species"]].get(element, 0)]

    for k, cap in ((135, .0004), (136, .0004), (137, .004)):
        check(f"{k}: sulfate-limited barium precipitation", lambda k=k, cap=cap:
              (.9 * cap < solid(k, "Ba") <= cap + 1e-8, phases(k, "Ba")))
    for k, scale in ((136, 1), (137, 10)):
        check(f"135/{k}: precipitate order/scale control", lambda k=k, scale=scale:
              (close(solid(135, "Ba"), solid(k, "Ba") / scale, relative=1e-4), [solid(135, "Ba"), solid(k, "Ba") / scale]))
    check("138/139/140: magnesium precipitation, acid reversal and hydroxide limitation", lambda:
          (.5 * .001 < solid(138, "Mg") <= .001 + 1e-8 and solid(139, "Mg") < .1 * solid(138, "Mg")
           and 0 < solid(140, "Mg") < solid(138, "Mg") and solid(140, "Mg") <= .0005 + 1e-8,
           {str(k): phases(k, "Mg") for k in (138, 139, 140)}))

    for k in range(141, 147):
        def closed_pressure(k=k):
            vessel = final(k)
            n = sum(p["moles"] for p in vessel["contents"] if p["phase"] == "gas")
            calculated = n * 8314.46261815324 * vessel["temperature"] / vessel["headspace"]["volume"]
            return (vessel["headspace"]["boundary"] == "sealed" and n > 0
                    and close(vessel["pressure"], calculated, absolute=1e-3, relative=1e-6)), {
                        "gas_mol": n, "reported_Pa": vessel["pressure"], "ideal_Pa": calculated}
        check(f"{k}: finite-headspace ideal-gas law", closed_pressure)
        def closed_inventory(k=k):
            # Capture after sealing but before the finite gas feed: initial
            # dissolved atmosphere and trapped air are not zero inventories.
            index = next(i for i, row in enumerate(rows[k]) if row.get("operator", {}).get("op") == "seal")
            sealed = next(v for v in rows[k][index]["bench"]["vessels"] if v["id"] == 0)
            key = "CO2" if k <= 144 else "N2"
            expected = {e: feed(k).get(key, 0) * composition[key].get(e, 0) for e in ("C", "N", "O")}
            actual = {e: atoms(final(k), e) - atoms(sealed, e) for e in expected}
            flow = [e for row in rows[k][index + 1:] for e in row.get("events", [])
                    if e["event"] in ("gas_evolved", "gas_absorbed") and e["moles"] > 1e-8]
            return all(close(actual[e], n) for e, n in expected.items()) and not flow, {
                "added_mol_atoms": expected, "inventory_change_mol_atoms": actual, "unexpected_external_gas_flows": flow}
        check(f"{k}: closed-system gas atoms including initially trapped air", closed_inventory, "conservation")
    check("141–144: finite CO2 solvent/headspace/dose controls", lambda:
          (atoms(final(142), "C", "aqueous") <= atoms(final(141), "C", "aqueous") + 1e-8
           and atoms(final(143), "C", "aqueous") > atoms(final(141), "C", "aqueous")
           and atoms(final(144), "C", "aqueous") + 1e-8 >= atoms(final(141), "C", "aqueous"),
           {str(k): atoms(final(k), "C", "aqueous") for k in range(141, 145)}))
    check("145/146: dry-gas heat partition and warming", lambda:
          (final(145)["temperature"] > 298.15 and close(final(145)["temperature"], final(146)["temperature"], absolute=1e-5, relative=0)
           and close(final(145)["pressure"], final(146)["pressure"], absolute=.01, relative=1e-6),
           {str(k): {"temperature_K": final(k)["temperature"], "pressure_Pa": final(k)["pressure"]} for k in (145, 146)}))

    extents = {}
    for k in range(147, 153):
        def equilibrium(k=k):
            _index, before = prior(k, "react")
            vessel = next(v for v in before["vessels"] if v["id"] == 0)
            keys = ("CH3COOH", "ethanol", "ethyl_acetate", "water")
            initial = [amount(vessel, key) for key in keys]
            reactions = [e for e in events(k, "org_reacted") if e["name"] == "esterification"]
            x = sum(e["extent"] for e in reactions)
            extents[k] = x
            n = [initial[0] - x, initial[1] - x, initial[2] + x, initial[3] + x]
            q = n[2] * n[3] / (n[0] * n[1])
            valid = all(v > 0 for v in n) and abs(q - 4) < 1e-4
            valid &= all(close(atoms(vessel, e), atoms(final(k), e)) for e in ("C", "H", "O"))
            if k == 151:
                valid &= abs(x) < 1e-10 and all(close(amount(final(k), key), initial[i]) for i, key in enumerate(keys))
            else:
                valid &= bool(reactions) and all(e.get("boundary") for e in reactions)
                valid &= x > 0 if k in (147, 148) else x < 0
            return valid, {"pre_request_mol": dict(zip(keys, initial)), "extent_mol": x, "Q_after_signed_extent": q,
                           "boundaries": [e.get("boundary") for e in reactions]}
        check(f"{k}: current-inventory mass action, direction and conserved atoms", equilibrium)
    check("147/148: product addition suppresses forward extent", lambda:
          (0 < extents[148] < extents[147], {str(k): extents[k] for k in (147, 148)}))
    check("149/150: reverse extent extensive scaling", lambda:
          (close(extents[149], extents[150] / 7, relative=1e-4), [extents[149], extents[150] / 7]))
    check("152: water-limited reverse capacity", lambda: (-.012 < extents[152] < 0, {"extent_mol": extents[152]}))

    check("153/154: buffer dilution is not strong-acid dilution", lambda: (abs(ph(154) - ph(153)) < .1, [ph(153), ph(154)]))
    check("153/155: buffer ratio logarithm", lambda:
          (abs(ph(155) - ph(153) - math.log10(4)) < .08, {"pH_change": ph(155) - ph(153), "ideal_change": math.log10(4)}))
    for k, base, acid in ((156, .0016, .0044), (157, .0024, .0036)):
        ideal = math.log10((base / acid) / (.002 / .004))
        check(f"153/{k}: buffered equivalent-pulse law", lambda k=k, ideal=ideal:
              (abs(ph(k) - ph(153) - ideal) < .06 and (ph(k) < ph(153) if k == 156 else ph(k) > ph(153)),
               {"pH_change": ph(k) - ph(153), "ideal_ratio_change": ideal}))
    check("153/158: intensive buffer scale", lambda: (abs(ph(153) - ph(158)) < .01, [ph(153), ph(158)]))
    return {"fleet": "123–158", "binary_sha256": summary["binary_sha256"], "checks": checks,
            "passed": sum(c["passed"] for c in checks), "unmet": sum(not c["passed"] for c in checks)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=pathlib.Path)
    parser.add_argument("--out", type=pathlib.Path, required=True)
    args = parser.parse_args()
    result = analyse(args.directory)
    with args.out.open("x") as output:
        json.dump(result, output, indent=2)
        output.write("\n")
    print(f"{result['passed']} passed; {result['unmet']} unmet")
    raise SystemExit(bool(result["unmet"]))

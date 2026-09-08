#!/usr/bin/env python3
"""Law-based checks for the changed catalog, independent of rounded prose.

These checks supplement (not replace) `kero codex lint`. They inspect both
members of comparisons and inventory bounds that the catalog schema's
single-vessel pH range cannot express.
"""
import argparse
import json
import math
import pathlib
import re


def analyse(directory):
    summary = json.loads((directory / "summary.json").read_text())
    records = {record["id"]: record for record in summary["cases"]}
    required = {
        "endpoint-is-not-a-full-drop", "equilibrium-can-run-backward", "cold-from-baking-soda",
        "doubling-the-thiosulfate-doubles-the-rate", "the-acid-term-is-an-activity",
        "the-cross-disappears", "the-ten-degree-rule-and-where-it-frays",
        "two-beakers-and-one-clock", "a-salt-with-no-database", "the-acid-is-never-used-up",
        "three-components-one-cut", "equal-charge-different-clocks",
        "grouping-does-not-change-water-heat",
    }
    if not required.issubset(records):
        raise ValueError(f"Missing required catalog comparisons: {sorted(required - records.keys())}")
    checks = []
    ph = {}

    def check(name, result, evidence):
        checks.append({"check": name, "passed": bool(result), "evidence": evidence})

    for key, record in records.items():
        rows = []
        for line in (directory / key / "stdout.ndjson").read_text().splitlines():
            try:
                rows.append(json.loads(line))
            except ValueError:
                # Malformed lines remain an execution failure, not a reason
                # to discard all the other evidence in the comparison.
                continue
        events = [event for row in rows for event in row.get("events", [])]
        failures = [event for event in events if event["event"] == "solver_failed"]
        check(key + ": complete without solver failure",
              record["exit"] == 0 and not failures and not record["non_json_lines"],
              {"exit": record["exit"], "solver_failures": failures})
        vessels = record["final"]["vessels"] if record["final"] else []
        ph[key] = {str(v["id"]): (v.get("solution") or {}).get("ph") for v in vessels}
        script = (directory / key / "experiment.lab").read_text()
        if key == "three-components-one-cut":
            expected = {"water": 3.0, "methanol": 0.3, "isopropanol": 0.2}
            totals = {s: sum(p["moles"] for v in vessels for p in v["contents"]
                             if p["species"] == s) for s in expected}
            check(key + ": component conservation", len(vessels) == 2 and
                  all(abs(totals[s] - n) <= 1e-8 + 1e-6*n for s, n in expected.items()), totals)
            receiver = next((v for v in vessels if v["id"] == 1), None)
            cut = {s: sum(p["moles"] for p in receiver["contents"] if p["species"] == s)
                   for s in expected} if receiver else {}
            check(key + ": mixed receiver and requested molar cut", bool(cut) and
                  all(n > 0 for n in cut.values()) and abs(sum(cut.values()) - .7) <= 1e-8,
                  cut)
        if key == "equal-charge-different-clocks":
            faraday = 6.02214076e23 * 1.602176634e-19
            products = {}
            for event in events:
                if event["event"] == "electrolysed":
                    amounts = products.setdefault(event["vessel"], {"H2": 0.0, "O2": 0.0})
                    for pole in ("anode", "cathode"):
                        species = event[pole + "_species"]
                        amounts[species] = amounts.get(species, 0) + event[pole + "_moles"]
            check(key + ": equal and split charge obey SI Faraday law",
                  set(products) == {0, 1, 2} and all(
                      abs(amounts[s] - 12/(z*faraday)) <= 1e-10 + 1e-6*12/(z*faraday)
                      for amounts in products.values() for s, z in (("H2", 2), ("O2", 4))), products)
        if key == "grouping-does-not-change-water-heat":
            receivers = {v["id"]: v for v in vessels if v["id"] in (0, 9)}
            expected_t = 298.15 + 200/(6*75.3)
            check(key + ": both groupings conserve water and sensible energy",
                  set(receivers) == {0, 9} and all(
                      math.isfinite(v["temperature"]) and abs(v["temperature"] - expected_t) <= .005
                      and abs(sum(p["moles"] for p in v["contents"] if p["species"] == "water") - 6) <= 1e-8
                      for v in receivers.values()),
                  {"expected_temperature_K": expected_t,
                   "observed_temperature_K": {i: v["temperature"] for i, v in receivers.items()}})
        if "Na2S2O3" in script:
            acid = {}
            for vessel, n in re.findall(r"^add v(\d+) HCl ([0-9.eE+-]+)mol(?:\s|$)", script, re.M):
                index = int(vessel) - 1
                acid[index] = acid.get(index, 0.0) + float(n)
            extents = {}
            for event in events:
                if event["event"] == "reacted" and event.get("reaction") == "thiosulfate-acid":
                    index = event["vessel"]
                    extents[index] = extents.get(index, 0.0) + event["moles"]
            check(key + ": reaction extent respects supplied acid capacity",
                  all(0 <= extent <= acid.get(index, 0.0) / 2 + 1e-8
                      for index, extent in extents.items()),
                  {"supplied_acid_mol": acid, "integrated_extent_mol": extents})
            if "wait " in script:
                check(key + ": kinetic comparison actually reacts", bool(extents), extents)
        if key == "endpoint-is-not-a-full-drop":
            doses = [e for e in events if e["event"] == "titrated"]
            check(key + ": both increments reach pH target and equivalent volume",
                  len(doses) == 2 and all(e["endpoint_reached"] and
                      abs(e["final_ph"] - 7) <= 1e-4 and
                      abs(e["total_volume"] - 0.023) < 1e-6 for e in doses), doses)
            check(key + ": changing trial increment does not change delivered amount",
                  len(doses) == 2 and abs(doses[0]["total_volume"] - doses[1]["total_volume"]) < 1e-7,
                  [e["total_volume"] for e in doses])
        if key == "equilibrium-can-run-backward":
            reacted = [e for e in events if e["event"] == "org_reacted"]
            check(key + ": forward and reverse extents have opposite signs",
                  len(reacted) == 2 and reacted[0]["extent"] > 0 and reacted[1]["extent"] < 0 and
                  all(e.get("boundary") for e in reacted), reacted)
            inputs = {}
            for vessel, species, n in re.findall(r"^add v(\d+) (\S+) ([0-9.eE+-]+)mol(?:\s|$)", script, re.M):
                index = int(vessel) - 1
                amounts = inputs.setdefault(index, {})
                amounts[species] = amounts.get(species, 0.0) + float(n)
            for index, amounts in sorted(inputs.items()):
                initial = tuple(amounts.get(key, 0.0) for key in ["CH3COOH", "ethanol", "ethyl_acetate", "water"])
                event = next((e for e in reacted if e["vessel"] == index), None)
                if event is None:
                    check(key + f": vessel {index} equilibrium quotient", False, "No reaction event")
                    continue
                extent = event["extent"]
                acid, alcohol, ester, water = initial
                remaining = (acid - extent, alcohol - extent, ester + extent, water + extent)
                denominator = remaining[0] * remaining[1]
                quotient = remaining[2] * remaining[3] / denominator if denominator > 0 else None
                check(key + f": vessel {index} mass-action root from actual extent",
                      all(n >= 0 for n in remaining) and quotient is not None and abs(quotient - 4) < 1e-8,
                      {"initial_mol": initial, "extent_mol": extent, "Q": quotient})
        if key == "a-salt-with-no-database":
            salt_only = next((v for v in vessels if v["id"] == 2), None)
            check(key + ": missing ionic model cannot inherit pure-water pH",
                  salt_only is not None and salt_only.get("solution") is None and
                  any(e["event"] == "not_yet_modeled" for e in events),
                  {"salt_only_solution": salt_only.get("solution") if salt_only else "Missing vessel"})
        if key == "cold-from-baking-soda":
            temperature = vessels[0]["temperature"] if vessels else None
            gas = sum(e["moles"] for e in events if e["event"] == "gas_evolved" and e["species"] == "CO2")
            check(key + ": cooling and carbonate gas change occur together",
                  temperature is not None and temperature < 298.15 and gas > 0,
                  {"temperature_K": temperature, "CO2_mol": gas})
    return {"binary_sha256": summary["binary_sha256"], "checks": checks,
            "passed": sum(c["passed"] for c in checks), "unmet": sum(not c["passed"] for c in checks),
            "final_ph_by_entry": ph}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=pathlib.Path)
    parser.add_argument("--out", required=True, type=pathlib.Path)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("output exists; choose a fresh evidence path")
    result = analyse(args.directory)
    args.out.write_text(json.dumps(result, indent=2) + "\n")
    print(f'{result["passed"]} passed; {result["unmet"]} unmet')
    raise SystemExit(bool(result["unmet"]))

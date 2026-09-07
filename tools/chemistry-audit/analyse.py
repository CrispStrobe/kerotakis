#!/usr/bin/env python3
"""Check physical expectations, separately from process completion.

Exit 1 means an unmet expectation, including documented model gaps. This is
an exploratory audit, not a claim that existing goldens are scientific truth.
"""
import argparse
import json
import math
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directories", nargs="*", type=pathlib.Path,
                        default=[HERE / "results", HERE / "followups", HERE / "followups-50"])
    parser.add_argument("--out", type=pathlib.Path, default=HERE / "checks.json")
    args = parser.parse_args()
    rows, executions, repeats = {}, [], []
    for directory in args.directories:
        summary = json.loads((directory / "summary.json").read_text())
        for record in summary["cases"]:
            key = int(record["id"][:2])
            data = [json.loads(line) for line in
                    (directory / record["id"] / "stdout.ndjson").read_text().splitlines()]
            if key in rows:
                repeats.append({"case": key, "identical_json": rows[key] == data})
            rows[key] = data
            executions.append({"id": record["id"], "exit": record["exit"],
                               "seconds": record["seconds"], "non_json_lines": record["non_json_lines"]})
    required = set(range(1, 51))
    if set(rows) != required:
        raise SystemExit(f"Expected all 50 cases; missing {sorted(required - set(rows))}")
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text())
    molar_mass = {row["species_id"]: row["quantity"]["value"]
                  for row in registry["phase_thermodynamics"] if row["property"] == "molar_mass"}
    checks = []

    def check(name, passed, evidence):
        checks.append({"check": name, "passed": bool(passed), "evidence": evidence})

    def events(key, kind):
        return [e for row in rows[key] for e in row.get("events", []) if e["event"] == kind]

    def measurements(key, instrument):
        return [e["value"] for e in events(key, "measured") if e["instrument"] == instrument]

    def final(key, vessel=0):
        return next(v for row in reversed(rows[key]) if "bench" in row
                    for v in row["bench"]["vessels"] if v["id"] == vessel)

    def moles(key, species, phase=None, vessel=0):
        return sum(p["moles"] for p in final(key, vessel)["contents"]
                   if p["species"] == species and (phase is None or p["phase"] == phase))

    def scene_mass(row):
        return sum(v["mass_g"] for v in row["scene"]["vessels"])

    for key in [1, 49, 50]:
        ph = measurements(key, "ph_meter")
        check(f"{key:02}: water/nonionic baseline has measurable near-neutral pH",
              bool(ph) and 6.5 < ph[-1] < 7.5, ph)
    for key, sign in [(2, 1), (3, -1)]:
        ph = measurements(key, "ph_meter")
        delta = ph[1] - ph[0]
        check(f"{key:02}: dilute strong electrolyte tenfold pH shift", abs(delta - sign) < 0.03, ph)
    a, b = final(4), final(5)
    check("04/05: neutralization addition-order invariance",
          abs(a["temperature"] - b["temperature"]) < 0.001 and
          abs(a["solution"]["ph"] - b["solution"]["ph"]) < 0.001,
          {"temperature_K": [a["temperature"], b["temperature"]],
           "ph": [a["solution"]["ph"], b["solution"]["ph"]]})
    ph = measurements(6, "ph_meter")
    check("06/07: buffer resists acid", abs(ph[1] - ph[0]) < 0.15 and
          measurements(7, "ph_meter")[-1] < 3, ph)
    ph = measurements(8, "ph_meter")
    check("08: successive phosphate additions raise pH", all(a < b for a, b in zip(ph, ph[1:])), ph)
    ph = measurements(9, "ph_meter")[-1]
    check("09: ultradilute acid respects water autoionization", 6.9 < ph < 7.1, ph)
    for key, species in [(10, "AgCl"), (12, "BaSO4"), (13, "Cu(OH)2")]:
        amount = moles(key, species, "solid")
        check(f"{key:02}: major precipitate yield", 0.0019 < amount <= 0.002, amount)
    delta = abs(moles(10, "AgCl") - moles(11, "AgCl"))
    check("10/11: precipitation addition-order yield", delta < 1e-8, {"delta_mol": delta})
    amount = moles(14, "NaCl", "solid")
    check("14/15: saturation and redissolution", 0.1 < amount < 0.3 and
          moles(15, "NaCl", "solid") == 0, {"saturated_solid_mol": amount})
    check("16/17: dissolution heat signs", final(16)["temperature"] > 298.15 and
          final(17)["temperature"] < 298.15,
          {"temperatures_K": [final(16)["temperature"], final(17)["temperature"]]})
    initial = rows[18][1]["bench"]["vessels"][0]
    check("18: equal heat/cool returns approximate initial temperature",
          abs(initial["temperature"] - final(18)["temperature"]) < 0.01,
          [initial["temperature"], final(18)["temperature"]])
    check("20/21: gas responds to declared boundary", final(20)["pressure"] > 120000 and
          final(21)["pressure"] == 120000 and final(21)["headspace"]["volume"] > 0.25,
          {"sealed_Pa": final(20)["pressure"], "regulated_L": final(21)["headspace"]["volume"]})
    solid_before = next(p["moles"] for p in rows[22][2]["bench"]["vessels"][0]["contents"]
                        if p["species"] == "CaCO3")
    check("22: excess finite CO2 substantially reduces precipitate",
          moles(22, "CaCO3") < solid_before * 0.1,
          {"before_mol": solid_before, "after_mol": moles(22, "CaCO3")})
    check("23/24: displacement and reverse control", moles(23, "Cu", "solid") > 0.0029 and
          moles(24, "Zn", "solid") == 0 and moles(24, "Cu", "solid") == 0.004,
          {"Cu_plated_mol": moles(23, "Cu", "solid")})
    gas = events(25, "gas_evolved")
    check("25/26: magnesium acid and copper control", len(gas) == 1 and
          abs(gas[0]["moles"] - 0.002) < 1e-10 and moles(26, "Cu", "solid") == 0.002, gas)
    voltage = events(27, "cell_voltage")[0]["volts"]
    check("27: Daniell voltage", 1.05 < voltage < 1.15, voltage)
    e = events(28, "electrolysed")[0]
    check("28: Faraday charge and gas ratio", e["coulombs"] == 60 and
          abs(e["cathode_moles"] / e["anode_moles"] - 2) < 1e-12 and
          abs(e["cathode_moles"] - 60 / (2 * 96485.33212)) < 1e-10, e)
    check("30: dilution preserves permanganate inventory", abs(moles(30, "MnO4-") - 1e-4) < 1e-10,
          events(30, "observed"))
    check("31: filter preserves solid and dissolved silver", abs(moles(31, "AgCl") +
          moles(31, "Ag+", vessel=1) - 0.002) < 1e-9 and moles(31, "water") == 0,
          {"solid_mol": moles(31, "AgCl"), "filtrate_Ag_mol": moles(31, "Ag+", vessel=1)})
    ethanol_fraction = moles(33, "ethanol", vessel=1) / (
        moles(33, "ethanol", vessel=1) + moles(33, "water", vessel=1))
    check("33: distillation enriches ethanol and conserves it", ethanol_fraction > 0.4/4.4 and
          abs(moles(33, "ethanol") + moles(33, "ethanol", vessel=1) - 0.4) < 1e-12,
          {"receiver_ethanol_mole_fraction": ethanol_fraction})
    check("34: passive organic wait explains unavailable kinetics", bool(events(34, "not_yet_modeled")),
          {"wait_events": rows[34][3]["events"]})
    check("35: explicit organic reaction declares its boundary", bool(events(35, "org_reacted")[0].get("boundary")),
          events(35, "org_reacted"))
    check("36/37: fermentation requires yeast", not events(36, "fermented") and bool(events(37, "fermented")),
          events(37, "fermented"))
    check("37: fermentation conserves mass when all generated CO2 is retained",
          abs(scene_mass(rows[37][3]) - scene_mass(rows[37][2])) < 1e-4,
          {"mass_change_g": scene_mass(rows[37][3]) - scene_mass(rows[37][2])})
    check("38: hot MgO phase agrees with solver's liquid MgO", moles(38, "MgO", "liquid") > 0.009,
          {"T_K": final(38)["temperature"], "contents": final(38)["contents"],
           "provenance": events(38, "thermal_equilibrium")[0]["provenance"]})
    check("39: NaCl flame test preserves salt", moles(39, "NaCl") == 0.01 and bool(events(39, "flame_test")),
          events(39, "flame_test"))
    endpoint = events(42, "titrated")[0]
    check("42: endpoint is refined to the requested pH", abs(endpoint["final_ph"] - 7.0) <= 1e-4,
          {"volume_L": endpoint["total_volume"], "ph": endpoint["final_ph"]})
    for key, species, dose in [(43, "HCl", 0.1), (44, "NaOH", 0.1), (45, None, 0.5)]:
        masses = measurements(key, "balance")
        expected = dose * molar_mass[species] if species else dose
        actual = masses[1] - masses[0]
        check(f"{key}: added mass appears on balance", abs(actual - expected) < 1e-4,
              {"expected_added_g": expected, "actual_added_g": actual, "error_g": actual - expected})
    masses = measurements(48, "balance")
    expected = 0.01 * molar_mass["NaHCO3"] + 0.012 * molar_mass["CH3COOH"]
    check("48: sealed fizz weighs its additions", abs(masses[1] - masses[0] - expected) < 1e-4,
          {"expected_added_g": expected, "actual_added_g": masses[1] - masses[0]})
    if repeats:
        check("repeat: replayed cases are identical", all(r["identical_json"] for r in repeats), repeats)
    nonfinite = []

    def finite(value, path):
        if isinstance(value, float) and not math.isfinite(value):
            nonfinite.append(path)
        elif isinstance(value, dict):
            for key, child in value.items(): finite(child, path + "/" + str(key))
        elif isinstance(value, list):
            for key, child in enumerate(value): finite(child, path + "/" + str(key))

    finite(rows, "cases")
    check("output: all numeric JSON values finite", not nonfinite, nonfinite)
    check("output: every experiment completes", all(r["exit"] == 0 for r in executions),
          [r for r in executions if r["exit"] != 0])
    check("output: no solver failures", not any(events(key, "solver_failed") for key in rows),
          [e for key in rows for e in events(key, "solver_failed")])
    for key in [41, 46]:
        solution = final(key).get("solution", {})
        names = [s["name"] for s in solution.get("species", [])]
        check(f"{key}: copper/ammonia includes tetraammine speciation", "Cu(NH3)4+2" in names,
              {"native_species": names, "boundary": "a presence/coverage probe, not a validated complex yield"})
        # These feeds supply ammonia N(-3) and sulfate S(+6). The adapter
        # explicitly promises to hold these slow redox states as added. Check
        # the native state, not just the reconstructed analytical inventory.
        changed = [state for state in solution.get("redox", [])
                   if state["element"] in {"N", "S"}
                   and state["oxidation"] != {"N": -3, "S": 6}[state["element"]]
                   and state["molality"] * solution.get("solvent_kg", 0.0) > 1e-8]
        check(f"{key}: native speciation honors declared slow-redox isolation",
              not changed and bool(solution.get("redox")),
              {"unexpected_redox_states": changed,
               "routing_claim": solution.get("provenance", {}).get("routing")})
    result = {"unique_cases": len(rows), "executions": len(executions),
              "execution_records": executions, "checks": checks,
              "passed": sum(c["passed"] for c in checks),
              "unmet": sum(not c["passed"] for c in checks),
              "solver_failures": [e for key in rows for e in events(key, "solver_failed")]}
    args.out.write_text(json.dumps(result, indent=2) + "\n")
    for c in checks:
        print("PASS" if c["passed"] else "UNMET", c["check"])
    print(f"{result['passed']} passed, {result['unmet']} unmet; {len(rows)} distinct experiments")
    return bool(result["unmet"])


if __name__ == "__main__":
    raise SystemExit(main())

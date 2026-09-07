#!/usr/bin/env python3
"""Predeclared law, control and capability checks for third-fleet cases 87–122.

No previously computed answers are inputs. Checks use conservation, declared
model input data, SI charge conversion and matched experiments. A missing
result fails its check rather than causing other evidence to be discarded.
"""
import argparse
import json
import math
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]


def analyse(directory):
    summary = json.loads((directory / "summary.json").read_text())
    records = {int(r["id"].split("-", 1)[0]): r for r in summary["cases"]}
    if len(summary["cases"]) != 36 or set(records) != set(range(87, 123)):
        raise ValueError("Exactly the 36 distinct cases 87–122 are required")
    rows = {}
    for number, record in records.items():
        rows[number] = []
        for line in (directory / record["id"] / "stdout.ndjson").read_text().splitlines():
            try:
                rows[number].append(json.loads(line))
            except ValueError:
                pass  # non_json_lines makes the execution check fail
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text())
    composition = {r["species_id"]: {e["element"]: e["count"]["value"] for e in r["elements"]}
                   for r in registry["compositions"]}
    checks = []

    def check(name, predicate, evidence, kind="model-law"):
        checks.append({"check": name, "kind": kind, "passed": bool(predicate), "evidence": evidence})

    def guarded(name, calculate, kind="model-law"):
        try:
            passed, evidence = calculate()
            check(name, passed, evidence, kind)
        except (KeyError, IndexError, StopIteration, ZeroDivisionError, TypeError, ValueError) as exc:
            check(name, False, {"missing_or_invalid_output": f"{type(exc).__name__}: {exc}"}, kind)

    def final(k, index=0):
        return next(v for v in records[k]["final"]["vessels"] if v["id"] == index)

    def events(k, name):
        return [e for row in rows[k] for e in row.get("events", []) if e["event"] == name]

    def amount(k, key, index=0, phase=None):
        return sum(p["moles"] for p in final(k, index)["contents"]
                   if p["species"] == key and (phase is None or p["phase"] == phase))

    def atoms(vessel, element, phase=None):
        return sum(p["moles"] * composition[p["species"]].get(element, 0)
                   for p in vessel["contents"] if phase is None or p["phase"] == phase)

    def ph(k, index=0):
        return final(k, index)["solution"]["ph"]

    def extent(k):
        return sum(e["moles"] for e in events(k, "reacted") if e.get("reaction") == "thiosulfate-acid")

    def close(a, b, absolute=1e-8, relative=1e-6):
        return abs(a - b) <= absolute + relative * max(abs(a), abs(b))

    for k, record in records.items():
        check(f"{k}: complete execution without solver failure",
              record["exit"] == 0 and not record["non_json_lines"] and record["final"] is not None
              and not events(k, "solver_failed"),
              {"exit": record["exit"], "solver_failures": events(k, "solver_failed"), "stderr": record["stderr"]}, "execution")
        if record["final"] is None:
            continue
        check(f"{k}: finite nonnegative material",
              all(math.isfinite(p["moles"]) and p["moles"] >= 0
                  for v in record["final"]["vessels"] for p in v["contents"]), {}, "conservation")
        unknown = sorted({p["species"] for v in record["final"]["vessels"] for p in v["contents"]
                          if p["species"] not in composition})
        check(f"{k}: every output species has a composition", not unknown, unknown, "conservation")
        expected = {}
        script = (directory / record["id"] / "experiment.lab").read_text()
        for key, number in re.findall(r"^add v\d+ (\S+) ([0-9.eE+-]+)mol$", script, re.M):
            # These elements have no gas outlet in the corresponding probes.
            # C/N/S/H/O need explicit reservoir/phase accounting, not a silent
            # assumption that every open experiment is a closed system.
            for element, count in composition[key].items():
                if element in {"Na", "K", "Cl", "Ca", "Mg", "Zn", "Fe", "Cu"}:
                    expected[element] = expected.get(element, 0) + float(number) * count
        if expected and not unknown:
            actual = {element: sum(atoms(v, element) for v in record["final"]["vessels"])
                      for element in expected}
            check(f"{k}: nonvolatile element conservation", all(close(actual[e], n) for e, n in expected.items()),
                  {"expected_mol_atoms": expected, "actual_mol_atoms": actual}, "conservation")

    for k, scale in [(87, 1), (88, 1), (89, 10)]:
        def ternary(k=k, scale=scale):
            original = {"water": 3 * scale, "methanol": 0.3 * scale, "isopropanol": 0.2 * scale}
            receiver = {key: amount(k, key, 1) for key in original}
            cut = events(k, "distilled")[-1]
            valid = all(n > 0 for n in receiver.values()) and close(sum(receiver.values()), 0.7 * scale)
            valid &= all(close(amount(k, key) + receiver[key], n) for key, n in original.items())
            valid &= receiver["methanol"] / sum(receiver.values()) > 0.3 / 3.5
            valid &= len(cut["components"]) == 3 and "approximation" in cut["model"] and "azeotropes" in cut["model"]
            return valid, {"receiver_mol": receiver, "model": cut["model"]}
        guarded(f"{k}: three-component cut conservation/enrichment/disclosure", ternary)
    for other, scale in [(88, 1), (89, 10)]:
        guarded(f"87/{other}: componentwise cut order/scale invariance", lambda other=other, scale=scale:
                (all(close(amount(87, key, 1), amount(other, key, 1) / scale, relative=1e-5)
                     for key in ["water", "methanol", "isopropanol"]),
                 {key: [amount(87, key, 1), amount(other, key, 1) / scale]
                  for key in ["water", "methanol", "isopropanol"]}))

    def latent_cut():
        # Property INPUTS, not cached results. The additional rows have their
        # own reviewed USCG provenance; water remains its existing data owner.
        latent = {}
        boiling = {}
        for line in (ROOT / "data/thermo/uscg-chris-still.tsv").read_text().splitlines():
            if line.startswith("#") or not line.strip():
                continue
            key, tb, j_kg, grams_mol, _ = line.split()
            latent[key] = float(j_kg) * float(grams_mol) / 1e6
            boiling[key] = float(tb)
        source = (ROOT / "crates/kerotakis-thermo/src/vle.rs").read_text()
        latent["water"] = float(re.search(r"WATER_HVAP_KJ_PER_MOL: f64 = ([0-9.]+)", source)[1])
        cut = events(90, "distilled")[-1]
        calculated = sum(amount(90, key, 1) * h for key, h in latent.items())
        valid = close(calculated, 3.0, absolute=1e-8) and close(cut["energy_kj"], calculated)
        valid &= "excluding reflux" in cut["model"] and 0 < sum(amount(90, key, 1) for key in latent) < 3.5
        pure = events(92, "distilled")[-1]
        return valid and close(pure["at"], boiling["methanol"], absolute=1e-6), {
            "budget_kj": 3, "computed_from_condensate_kj": calculated, "reported_kj": cut["energy_kj"],
            "pure_temperature_K": pure["at"], "reference_boiling_K": boiling["methanol"]}
    guarded("90/92: latent-budget identity and pure-component boiling limit", latent_cut)
    guarded("92: pure-component cut conserves supplied methanol", lambda:
            (close(amount(92, "methanol", 1), 0.1) and close(amount(92, "methanol"), 0.3),
             {"source": amount(92, "methanol"), "receiver": amount(92, "methanol", 1)}))

    def rejected_still():
        index = next(i for i, row in enumerate(rows[91]) if row.get("operator", {}).get("op") == "distil")
        before = next(row["bench"] for row in reversed(rows[91][:index]) if "bench" in row)
        after = rows[91][index]["bench"]
        valid = [v["contents"] for v in before["vessels"]] == [v["contents"] for v in after["vessels"]]
        diagnostic = events(91, "not_yet_modeled")
        valid &= not events(91, "distilled") and any("NH3" in e["what"] and "distillation" in e["what"] for e in diagnostic)
        return valid, diagnostic
    guarded("91: unsupported aqueous volatile is atomically withheld", rejected_still, "expected-model-boundary")

    for k in [93, 94]:
        guarded(f"{k}: dilute strong-acid law", lambda k=k:
                (abs(ph(k) + math.log10(0.0001 / 0.5)) < 0.08, {"pH": ph(k), "ideal_concentration_pH": -math.log10(0.0001 / 0.5)}))
    guarded("93/94: acid identity control", lambda: (abs(ph(93) - ph(94)) < 0.03, [ph(93), ph(94)]))
    for k in [95, 96]:
        guarded(f"{k}: sealed bromine inventory", lambda k=k:
                (close(atoms(final(k), "Br"), 0.001, absolute=1e-9) and ph(k) < 4,
                 {"Br_mol_atoms": atoms(final(k), "Br"), "pH": ph(k)}), "conservation")
    guarded("95/96: headspace uptake control", lambda:
            (atoms(final(96), "Br", "aqueous") <= atoms(final(95), "Br", "aqueous") + 1e-8,
             {"small_headspace_aqueous_Br": atoms(final(95), "Br", "aqueous"),
              "large_headspace_aqueous_Br": atoms(final(96), "Br", "aqueous")}))
    guarded("97/98: finite-gas neutralization order and thermal path", lambda:
            (all(6.5 < ph(k) < 7.5 for k in [97, 98]) and abs(ph(97) - ph(98)) < 0.01
             and abs(final(97)["temperature"] - final(98)["temperature"]) < 0.01,
             {str(k): {"pH": ph(k), "temperature_K": final(k)["temperature"]} for k in [97, 98]}))

    def solid_metal(k, element):
        inventory = {p["species"]: p["moles"] * composition[p["species"]].get(element, 0)
                     for p in final(k)["contents"] if p["phase"] == "solid"
                     and composition[p["species"]].get(element, 0) > 0}
        return {"metal_mol_atoms": atoms(final(k), element, "solid"), "phase_metal_mol_atoms": inventory}

    for made, dissolved, element, maximum in [(99, 100, "Zn", 0.002), (101, 102, "Fe", 0.001)]:
        def precipitation(made=made, dissolved=dissolved, element=element, maximum=maximum):
            before, after = solid_metal(made, element), solid_metal(dissolved, element)
            return (0.5 * maximum < before["metal_mol_atoms"] <= maximum + 1e-8
                    and after["metal_mol_atoms"] < 0.1 * before["metal_mol_atoms"]), {
                        "before_acid": before, "after_acid": after,
                        "scope": "all solid phases; no assumed hydroxide/oxide/polymorph identity"}
        guarded(f"{made}/{dissolved}: precipitation capacity and acid reversal", precipitation)
    guarded("103/104: carbonate liquid-feed dilution control", lambda:
            (0 < atoms(final(103), "Ca", "solid") <= 0.001 + 1e-8
             and atoms(final(104), "Ca", "solid") < atoms(final(103), "Ca", "solid"),
             {str(k): solid_metal(k, "Ca") for k in [103, 104]}))

    faraday = 6.02214076e23 * 1.602176634e-19  # exact SI definitions N_A * e
    electrolysis = {}
    for k, charge in [(105, 12), (106, 12), (107, 24), (108, 12)]:
        def faraday_check(k=k, charge=charge):
            production = {}
            records_e = events(k, "electrolysed")
            for event in records_e:
                for pole in ["anode", "cathode"]:
                    key = event[pole + "_species"]
                    production[key] = production.get(key, 0) + event[pole + "_moles"]
            electrolysis[k] = production
            valid = close(sum(e["coulombs"] for e in records_e), charge)
            valid &= close(production["H2"], charge / (2 * faraday), absolute=1e-10, relative=1e-6)
            valid &= close(production["O2"], charge / (4 * faraday), absolute=1e-10, relative=1e-6)
            # Events count newly generated matter once, before any subsequent
            # open-gas loss or dissolution. Do not add them to final inventory:
            # that would count the same generated gas twice.
            return valid, {"charge_C": charge, "generated_mol": production, "accounting": "sum electrode production events, independent of final gas location"}
        guarded(f"{k}: Faraday charge and both electrode products", faraday_check)
    guarded("105–108: equal/split/doubled charge controls", lambda:
            (all(close(electrolysis[k][key] / factor, electrolysis[105][key], absolute=1e-10)
                 for k, factor in [(106, 1), (107, 2), (108, 1)] for key in ["H2", "O2"]), electrolysis))
    guarded("109: automatic electrode orientation is operand invariant", lambda:
            (len(events(109, "cell_voltage")) == 2
             and all(events(109, "cell_voltage")[0][key] == events(109, "cell_voltage")[1][key] for key in ["anode", "cathode"])
             and close(events(109, "cell_voltage")[0]["volts"], events(109, "cell_voltage")[1]["volts"], absolute=1e-10), events(109, "cell_voltage")))
    guarded("110: identical half-cells have no driving voltage", lambda:
            (any(abs(e["volts"]) < 1e-8 for e in events(110, "cell_voltage"))
             or any("identical" in e["why"] and "0 V" in e["why"] for e in events(110, "no_cell")),
             events(110, "cell_voltage") + events(110, "no_cell")))

    for k, capacity in [(111, 0.0005), (112, 0.005), (113, 0.0005), (114, 0.00005), (116, 0.0005)]:
        guarded(f"{k}: finite kinetic acid capacity and actual progress", lambda k=k, capacity=capacity:
                (0 < extent(k) <= capacity + 1e-8 and close(amount(k, "S"), extent(k), absolute=1e-8),
                 {"extent_mol": extent(k), "acid_capacity_mol": capacity, "S_mol": amount(k, "S")}))
    guarded("111/112: intensive kinetic scale control", lambda:
            (close(extent(111), extent(112) / 10, absolute=1e-8, relative=1e-3), [extent(111), extent(112) / 10]))
    guarded("111/113: time-partition stability screen", lambda:
            (close(extent(111), extent(113), absolute=1e-8, relative=0.01),
             {"single_step": extent(111), "five_steps": extent(113), "relative_budget": 0.01,
              "scope": "predeclared convergence screen, not exact semigroup identity with refreshed activity/temperature"}), "approximation-stability")
    guarded("115: buffered proton-consuming rate is withheld explicitly", lambda:
            (extent(115) == 0 and amount(115, "S") == 0
             and any("thiosulfate-acid" in e["what"] and "buffer" in e["what"] for e in events(115, "not_yet_modeled")),
             events(115, "not_yet_modeled")), "expected-model-boundary")

    expected_t = 298.15 + (300 - 200 + 100) / (6 * 75.3)  # existing water Cp, J/mol/K
    guarded("117/118: three-way water heat conservation and associativity", lambda:
            (abs(final(117, 4)["temperature"] - expected_t) < 0.001
             and abs(final(118, 4)["temperature"] - expected_t) < 0.001
             and abs(final(117, 4)["temperature"] - final(118, 4)["temperature"]) < 1e-5,
             {"energy_balance_K": expected_t, "left_K": final(117, 4)["temperature"], "right_K": final(118, 4)["temperature"]}))
    guarded("119: three-feed residual strong-acid law", lambda:
            (abs(ph(119, 4) + math.log10(0.001 / 0.6)) < 0.1,
             {"pH": ph(119, 4), "ideal_concentration_pH": -math.log10(0.001 / 0.6)}))
    guarded("120: serial aliquot fractions compose multiplicatively", lambda:
            (all(close(amount(120, "K+", index), expected, absolute=1e-9)
                 for index, expected in [(0, 0.008 * 0.75**2), (1, 0.008 * 0.25), (2, 0.008 * 0.75 * 0.25)]),
             {str(index): amount(120, "K+", index) for index in range(3)}))
    for k, scale in [(121, 1), (122, 0.1)]:
        def equilibrium(k=k, scale=scale):
            event = events(k, "org_reacted")[-1]
            x = event["extent"] / scale
            n = [0.01 - x, 0.02 - x, 0.005 + x, 0.015 + x]
            q = n[2] * n[3] / (n[0] * n[1])
            return all(v > 0 for v in n) and abs(q - 4) < 1e-4 and bool(event.get("boundary")), {"scaled_extent_mol": x, "nominal_feed_Q_after_extent": q, "boundary": event.get("boundary")}
        guarded(f"{k}: four-component mass-action equation", equilibrium)
    guarded("121/122: equilibrium extent scale control", lambda:
            (close(events(121, "org_reacted")[-1]["extent"], 10 * events(122, "org_reacted")[-1]["extent"], absolute=1e-8, relative=1e-4),
             {str(k): events(k, "org_reacted")[-1]["extent"] for k in [121, 122]}))
    return {"fleet": "87–122", "binary_sha256": summary["binary_sha256"], "checks": checks,
            "passed": sum(c["passed"] for c in checks), "unmet": sum(not c["passed"] for c in checks)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=pathlib.Path)
    parser.add_argument("--out", required=True, type=pathlib.Path)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("report already exists; preserve evidence and choose a new output")
    report = analyse(args.directory)
    args.out.write_text(json.dumps(report, indent=2) + "\n")
    print(f'{report["passed"]} passed; {report["unmet"]} unmet')
    raise SystemExit(bool(report["unmet"]))

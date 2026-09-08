#!/usr/bin/env python3
"""Independent law/control checks for cases 51–86; unmet checks exit nonzero.

No golden output is treated as a chemical oracle. Stoichiometry comes from
the registry, comparisons from matched controls, and bounds from conservation.
This is deliberately a discovery audit, not a claim of universal accuracy.
"""
import argparse
import json
import math
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[2]


def analyse(directory):
    summary = json.loads((directory / "summary.json").read_text())
    records = {int(r["id"][:2]): r for r in summary["cases"]}
    if set(records) != set(range(51, 87)):
        raise ValueError("All 36 distinct cases 51–86 are required")
    rows = {}
    for k, record in records.items():
        rows[k] = []
        for line in (directory / record["id"] / "stdout.ndjson").read_text().splitlines():
            try:
                rows[k].append(json.loads(line))
            except ValueError:
                # The recorder's non_json_lines makes execution fail below;
                # retain other usable evidence rather than crash the audit.
                continue
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text())
    composition = {r["species_id"]: {e["element"]: e["count"]["value"]
                                    for e in r["elements"]}
                   for r in registry["compositions"]}
    checks = []

    def check(name, predicate, evidence):
        checks.append({"check": name, "passed": bool(predicate), "evidence": evidence})

    def vessel(k, index=0):
        return next(v for row in reversed(rows[k]) if "bench" in row
                    for v in row["bench"]["vessels"] if v["id"] == index)

    def amount(k, species, phase=None, index=0):
        return sum(p["moles"] for p in vessel(k, index)["contents"]
                   if p["species"] == species and (phase is None or p["phase"] == phase))

    def events(k, event):
        return [e for row in rows[k] for e in row.get("events", []) if e["event"] == event]

    def measurements(k, instrument):
        return [e["value"] for e in events(k, "measured") if e["instrument"] == instrument]

    def ph(k):
        return measurements(k, "ph_meter")

    for k, record in records.items():
        failures = events(k, "solver_failed")
        check(f"{k}: complete execution without solver failure",
              record["exit"] == 0 and not record["non_json_lines"] and not failures,
              {"exit": record["exit"], "solver_failures": failures,
               "stderr": record["stderr"]})
        if not record["final"]:
            continue
        finite = all(math.isfinite(p["moles"]) and p["moles"] >= 0
                     for v in record["final"]["vessels"] for p in v["contents"])
        check(f"{k}: finite nonnegative final inventory", finite, {})
        # These elements have no gas outlet in these probes. H/O depend on
        # the external water/gas boundaries; N and C can be volatile, so do
        # not pretend an unqualified closed-system ledger applies to them.
        expected = {}
        script = (directory / record["id"] / "experiment.lab").read_text()
        for species, n in re.findall(r"^add v\d+ (\S+) ([0-9.eE+-]+)mol$", script, re.M):
            for element, count in composition[species].items():
                if element not in {"H", "O", "C", "N"}:
                    expected[element] = expected.get(element, 0) + float(n) * count
        actual = {element: sum(p["moles"] * composition.get(p["species"], {}).get(element, 0)
                               for v in record["final"]["vessels"] for p in v["contents"])
                  for element in expected}
        # Automatic titration supplies additional spectator ions, so its
        # input ledger needs the delivered dose and is checked separately.
        if expected and "titrate " not in script:
            check(f"{k}: nonvolatile elements conserved across all vessels",
                  all(abs(actual[e] - n) <= 1e-8 + n * 1e-6 for e, n in expected.items()),
                  {"expected_mol_atoms": expected, "actual_mol_atoms": actual})

    # A missing measurement is a failed chemical check, not an analyser crash.
    def guarded(name, calculate):
        try:
            result, evidence = calculate()
            check(name, result, evidence)
        except (KeyError, IndexError, StopIteration, ZeroDivisionError, TypeError) as exc:
            check(name, False, {"missing_output": type(exc).__name__ + ": " + str(exc)})

    guarded("51: ammonium hydrolysis is acidic", lambda: (4 < ph(51)[-1] < 7, ph(51)))
    for acid, base, label in [(52, 53, "ammonium"), (73, 74, "phosphate")]:
        guarded(f"{acid}/{base}: {label} buffer opposite small responses", lambda a=acid, b=base:
                (0 < ph(a)[0] - ph(a)[1] < 0.4 and 0 < ph(b)[1] - ph(b)[0] < 0.4,
                 {"acid": ph(a), "base": ph(b)}))
    guarded("54: bisulfate proton is neutralised", lambda:
            (ph(54)[0] < 3 and ph(54)[-1] > 6, ph(54)))
    for k in [55, 56, 68]:
        guarded(f"{k}: near-neutral final solution", lambda k=k: (6.5 < ph(k)[-1] < 7.5, ph(k)))
    guarded("57/58/85: gypsum order and dilution controls", lambda:
            (amount(57, "gypsum", "solid") > 0 and
             abs(amount(57, "gypsum", "solid") - amount(58, "gypsum", "solid")) < 1e-5 and
             amount(85, "gypsum", "solid") < amount(57, "gypsum", "solid"),
             {str(k): amount(k, "gypsum", "solid") for k in [57, 58, 85]}))
    guarded("59/60: acid reverses magnesium precipitation", lambda:
            (0.002 < amount(59, "Mg(OH)2", "solid") <= 0.003 and
             amount(60, "Mg(OH)2", "solid") < 0.1 * amount(59, "Mg(OH)2", "solid"),
             {str(k): amount(k, "Mg(OH)2", "solid") for k in [59, 60]}))
    guarded("61/62: acid consumes chalk relative to control", lambda:
            (amount(61, "CaCO3", "solid") < 0.1 * amount(62, "CaCO3", "solid"),
             {str(k): amount(k, "CaCO3", "solid") for k in [61, 62]}))
    guarded("63: dissolved glucose passes filter", lambda:
            (abs(amount(63, "glucose", index=1) - 0.005) < 1e-9 and amount(63, "glucose") < 1e-9,
             {"receiver_mol": amount(63, "glucose", index=1)}))
    guarded("64: filter separates silica and potassium", lambda:
            (amount(64, "SiO2", "solid") > 0.0049 and amount(64, "K+", index=1) > 0.0029,
             {"retained_silica": amount(64, "SiO2", "solid"), "passed_potassium": amount(64, "K+", index=1)}))
    guarded("65/66: fractional MIX operand invariance", lambda:
            (abs(vessel(65, 2)["solution"]["ph"] - vessel(66, 2)["solution"]["ph"]) < 1e-5 and
             abs(amount(65, "Mg+2", index=2) - 0.0005) < 1e-8 and
             abs(amount(66, "Mg+2", index=2) - 0.0005) < 1e-8,
             {str(k): {"ph": vessel(k, 2)["solution"]["ph"], "Mg_mol": amount(k, "Mg+2", index=2)} for k in [65, 66]}))
    for k, target in [(69, 7), (70, 5)]:
        guarded(f"{k}: downward titration refines target", lambda k=k, target=target:
                (abs(ph(k)[-1] - target) <= 1e-4, ph(k)))
    for k in [71, 72]:
        guarded(f"{k}: organic-acid neutralisation is monotone", lambda k=k:
                (len(ph(k)) >= 2 and all(a < b for a, b in zip(ph(k), ph(k)[1:])), ph(k)))
    guarded("75/76: thermal MIX is intermediate and order invariant", lambda:
            (298.15 < vessel(75, 2)["temperature"] < 308.15 and
             abs(vessel(75, 2)["temperature"] - vessel(76, 2)["temperature"]) < 1e-6,
             {str(k): vessel(k, 2)["temperature"] for k in [75, 76]}))
    for k in [77, 78, 79]:
        guarded(f"{k}: equilibrium is an explicitly bounded model", lambda k=k:
                (bool(events(k, "org_reacted")[0].get("boundary")), events(k, "org_reacted")))
    guarded("79/80: requested reverse reaction differs from passive waiting", lambda:
            (amount(79, "ethyl_acetate") < 0.009 and amount(79, "ethanol") > 0.001 and
             abs(amount(80, "ethyl_acetate") - 0.01) < 1e-8 and not events(80, "org_reacted"),
             {str(k): {"ester": amount(k, "ethyl_acetate"), "ethanol": amount(k, "ethanol")} for k in [79, 80]}))
    guarded("80: passive hydrolysis discloses its missing rate/condition model", lambda:
            (any("rate" in e.get("what", "").lower() or "kinetic" in e.get("what", "").lower()
                 for e in events(80, "not_yet_modeled")), events(80, "not_yet_modeled")))
    guarded("81/82: ionic conductivity exceeds nonionic control", lambda:
            (measurements(81, "conductivity_meter")[-1] > 100 * measurements(82, "conductivity_meter")[-1],
             {str(k): measurements(k, "conductivity_meter") for k in [81, 82]}))
    for k, species in [(83, "isopropanol"), (84, "methanol")]:
        guarded(f"{k}: alcohol distillation enriches and conserves", lambda k=k, species=species:
                (abs(amount(k, species) + amount(k, species, index=1) - 0.2) < 1e-9 and
                 amount(k, species, index=1) / (amount(k, species, index=1) + amount(k, "water", index=1)) > 0.2 / 4.2,
                 {"receiver_alcohol": amount(k, species, index=1), "receiver_water": amount(k, "water", index=1),
                  "remaining_alcohol": amount(k, species)}))
    guarded("86: ammonium nitrate dissolution cools", lambda:
            (vessel(86)["temperature"] < 298.15, vessel(86)["temperature"]))
    return {"batch": "51–86", "binary_sha256": summary["binary_sha256"],
            "checks": checks, "passed": sum(c["passed"] for c in checks),
            "unmet": sum(not c["passed"] for c in checks)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=pathlib.Path)
    parser.add_argument("--out", required=True, type=pathlib.Path)
    args = parser.parse_args()
    if args.out.exists():
        parser.error("check output already exists; choose a fresh evidence path")
    report = analyse(args.directory)
    args.out.write_text(json.dumps(report, indent=2) + "\n")
    print(f'{report["passed"]} passed; {report["unmet"]} unmet')
    raise SystemExit(bool(report["unmet"]))

"""Source-only saved-evidence checks and deliberate corruptions; no engine.

Run with an already collected polyprotic-carbonate evidence directory.
This never substitutes synthetic outputs for an actual producer execution.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from carbonate_closed_equilibrium import LawFailure, check_rows, rounded_interval

CASES = {
    "521-bicarbonate-dose-0.0005": .0005,
    "522-bicarbonate-dose-0.001": .001,
    "523-bicarbonate-dose-0.002": .002,
    "524-bicarbonate-dose-0.004": .004,
}


def corrupt_activity(rows):
    species = rows[-1]["bench"]["vessels"][0]["solution"]["species"]
    next(s for s in species if s["name"] == "HCO3-")["activity"] *= 1.05


def corrupt_charge_preserving_components(rows):
    species = rows[-1]["bench"]["vessels"][0]["solution"]["species"]
    by_name = {s["name"]: s for s in species}
    delta = .9*by_name["NaHCO3"]["molality"]
    # Preserve both the sodium and carbon censuses while changing charge.
    by_name["Na+"]["molality"] += delta
    by_name["NaHCO3"]["molality"] -= delta
    by_name["CO2"]["molality"] += delta


def corrupt_provenance(rows):
    rows[-1]["bench"]["vessels"][0]["solution"]["provenance"]["dataset"] = "unbound"


def corrupt_dose(rows):
    rows[1]["operator"]["moles"] *= 1.1


def corrupt_water_ledger(rows):
    portions = rows[-1]["bench"]["vessels"][0]["contents"]
    next(p for p in portions if p["species"] == "water")["moles"] += 1e-5


def corrupt_carbon_ledger(rows):
    portions = rows[-1]["bench"]["vessels"][0]["contents"]
    next(p for p in portions if p["species"] == "CO2(aq)")["moles"] += 1e-5


def corrupt_ph(rows):
    rows[-1]["bench"]["vessels"][0]["solution"]["ph"] += .01


def corrupt_unsupported(rows):
    rows[-1]["events"].append({"event": "not_yet_modeled", "reason": "unsupported"})


def corrupt_scope(rows):
    rows[-1]["bench"]["vessels"][0]["solution"]["scope"] = "solvent_only"


def corrupt_activity_model(rows):
    rows[-1]["bench"]["vessels"][0]["solution"]["provenance"]["model"] = "ideal"


MUTATIONS = {f.__name__: f for f in (
    corrupt_activity, corrupt_charge_preserving_components, corrupt_provenance,
    corrupt_dose, corrupt_water_ledger, corrupt_carbon_ledger, corrupt_ph,
    corrupt_unsupported, corrupt_scope, corrupt_activity_model)}


def assess_saved(directory):
    # Rounding must use known report precision despite JSON's lost zeros.
    assert rounded_interval(.00167)[0] < .0016695
    assert rounded_interval(.00167)[1] > .0016705
    results = {"mode": "saved-only; no engine", "cases": [], "mutations": []}
    for case, dose in CASES.items():
        path = directory/case/"stdout.ndjson"
        data = path.read_bytes()
        rows = [json.loads(line) for line in data.splitlines()]
        evidence = check_rows(rows, dose)
        results["cases"].append({"id": case, "sha256": hashlib.sha256(data).hexdigest(),
                                 "evidence": evidence})
        for name, mutation in MUTATIONS.items():
            changed = copy.deepcopy(rows)
            mutation(changed)
            try:
                check_rows(changed, dose)
            except LawFailure as error:
                results["mutations"].append({"case": case, "mutation": name,
                                              "rejected": True, "reason": str(error)})
            else:
                raise AssertionError(f"accepted corrupt evidence: {case}/{name}")
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("saved_family", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    results = assess_saved(args.saved_family)
    if args.report:
        with args.report.open("x") as handle:
            json.dump(results, handle, indent=2)
            handle.write("\n")
    print(f"{len(results['cases'])} saved cases accepted; "
          f"{len(results['mutations'])} corruptions rejected; no engine executed")


if __name__ == "__main__":
    main()

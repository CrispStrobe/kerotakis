"""Check captured thermal controls; baseline mode records defects without excusing repairs.

The fusion value is the pre-existing registry datum (6010 J/mol), not fitted
to these outputs. CLI checks complement full-state rollback unit tests:
the CLI stops on refusal, so its last successful snapshot cannot by itself
prove that internal state was restored.
"""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("evidence", type=Path)
parser.add_argument("--baseline", action="store_true")
parser.add_argument("--validation", type=Path, required=True)
args = parser.parse_args()
suite = Path(__file__).resolve().parent
checks = []
facts = []


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(name, passed, **details):
    checks.append(dict(name=name, passed=bool(passed), **details))


def strict(text):
    def reject(value):
        raise ValueError(value)
    return json.loads(text, parse_constant=reject)


receipt = strict((args.evidence / "execution.json").read_text())
validation = strict(args.validation.read_text())
check("validated executable provenance", validation["passed"] is True
      and all(s["exit_code"] == 0 for s in validation["stages"])
      and receipt["binary_source_commit"] == validation["commit"]
      and receipt["binary_sha256"] == validation["binary_sha256"])
for stage in validation["stages"]:
    check(f'validation {stage["name"]}: log digest',
          digest(args.validation.parent / stage["log"]) == stage["log_sha256"])
for name, expected in validation["source_hashes"].items():
    source = subprocess.check_output(["git", "show", f'{validation["commit"]}:{name}'],
                                     cwd=suite.parents[1])
    check(f"validation source: {name}", hashlib.sha256(source).hexdigest() == expected)
predictions = strict((suite / "predictions.json").read_text())
freeze = strict((suite / "freeze.json").read_text())
check("unchanged frozen predictions", digest(suite / "predictions.json")
      == freeze["predictions_sha256"] == receipt["predictions_sha256"])
check("all twelve controls in both CLI modes",
      sorted((r["id"], r["mode"]) for r in receipt["runs"])
      == sorted((p["id"], m) for p in predictions for m in ["text", "json"]))
for run in receipt["runs"]:
    for stream in ["stdout", "stderr"]:
        check(f'{run["id"]} {run["mode"]}: {stream} digest',
              digest(args.evidence / run[stream]) == run[f"{stream}_sha256"])


def water(v):
    return sum(p["moles"] for p in v["contents"] if p["species"] == "water")


def liquid(v):
    return sum(p["moles"] for p in v["contents"]
               if p["species"] == "water" and p["phase"] == "liquid")


def inventory(v):
    # Independent formulas for exactly this water/air/salt family. Analytical
    # free_proton/free_hydroxide are speciation coordinates, not extra stock.
    formulas = {
        "water": ({"H": 2, "O": 1}, 0), "N2": ({"N": 2}, 0),
        "O2": ({"O": 2}, 0), "CO2": ({"C": 1, "O": 2}, 0),
        "CO2(aq)": ({"C": 1, "O": 2}, 0), "H+": ({"H": 1}, 1),
        "HCO3-": ({"H": 1, "C": 1, "O": 3}, -1),
        "CO3-2": ({"C": 1, "O": 3}, -2), "OH-": ({"H": 1, "O": 1}, -1),
        "base_equivalents": ({"H": 1, "O": 1}, -1),
        "Na+": ({"Na": 1}, 1), "Cl-": ({"Cl": 1}, -1),
    }
    budget = {"charge": 0.0}
    for portion in v["contents"]:
        atoms, charge = formulas[portion["species"]]
        budget["charge"] += charge * portion["moles"]
        for atom, count in atoms.items():
            budget[atom] = budget.get(atom, 0.0) + count * portion["moles"]
    return budget


for prediction in predictions:
    case = prediction["id"]
    runs = [r for r in receipt["runs"] if r["id"] == case]
    rows = [strict(line) for line in (args.evidence / f"{case}.json.stdout").read_text().splitlines()]
    check(f"{case}: strict NDJSON", bool(rows) and all(isinstance(r, dict) for r in rows))
    snapshots = [r["bench"]["vessels"][0] for r in rows if "bench" in r]
    initial_index = next(i for i, r in enumerate(rows)
                         if r.get("operator", {}).get("op") == "inspect")
    initial = rows[initial_index]["bench"]["vessels"][0]
    final = snapshots[-1]
    transfers = [e for r in rows for e in r.get("events", []) if e["event"] == "energy_transferred"]
    fact = dict(id=case, exit_codes=[r["exit_code"] for r in runs],
                initial_k=initial["temperature"], final_k=final["temperature"],
                cycle_delta_k=final["temperature"]-initial["temperature"],
                partitions=[dict(heating=e["heating"], delivered_j=e["delivered_j"],
                                 sensible_j=e["sensible_j"], complete=e.get("energy_partition_complete"))
                            for e in transfers])
    facts.append(fact)
    if args.baseline:
        continue
    if case in ["T09", "T10"]:
        errors = " ".join((args.evidence / r["stderr"]).read_text() for r in runs)
        check(f"{case}: supported-domain refusal in both modes", all(r["exit_code"] != 0 for r in runs)
              and "thermal operation was not committed" in errors)
        check(f"{case}: no unsupported cooling budget published", not transfers
              and not any(r.get("operator", {}).get("op") in ["cool", "heat"] for r in rows))
        check(f"{case}: last published state precedes cooling", final == initial)
        continue
    check(f"{case}: both CLI modes succeed", all(r["exit_code"] == 0 for r in runs))
    check(f"{case}: reciprocal cycle within frozen 0.2K", abs(fact["cycle_delta_k"]) <= 0.2)
    check(f"{case}: both doses published", len(transfers) == 2)
    initial_budget = inventory(initial)
    for row in rows[initial_index:]:
        if "bench" not in row:
            continue
        current_budget = inventory(row["bench"]["vessels"][0])
        for atom in sorted(initial_budget.keys() | current_budget.keys()):
            delta = current_budget.get(atom, 0.0) - initial_budget.get(atom, 0.0)
            check(f'{case} output {row["output_sequence"]}: {atom} conserved',
                  abs(delta) <= 1e-8, inventory_delta_mol=delta)
    # Trace acid/carbonate chemistry can consume/form solvent in mixed cases;
    # strict solvent-only conservation is asserted for the pure-water controls.
    pure = case in ["T01", "T02", "T03", "T07", "T08"]
    if pure:
        check(f"{case}: water conserved through every snapshot",
              all(abs(water(v)-water(initial)) <= 1e-8 for v in snapshots))
        previous = initial
        for row in rows[initial_index:]:
            accounts = [e for e in row.get("events", []) if e["event"] == "energy_transferred"]
            for account in accounts:
                current = row["bench"]["vessels"][0]
                phases = [e for e in row.get("events", []) if e["event"] == "state_changed"]
                change = liquid(current) - liquid(previous)
                narrated_change = sum(e["moles"] * (1 if e["to"] == "liquid" else -1)
                                      for e in phases)
                check(f'{case}: {row["operator"]["op"]} phase events match committed inventory',
                      all(e["species"] == "water" and {e["from"], e["to"]} == {"solid", "liquid"}
                          for e in phases) and abs(change-narrated_change) <= 1e-8)
                latent = change * 6010.0 * (1 if account["heating"] else -1)
                check(f'{case}: {row["operator"]["op"]} partition established',
                      account.get("energy_partition_complete") is True)
                check(f'{case}: {row["operator"]["op"]} committed fusion reconciles dose',
                      abs(account["sensible_j"]+latent-account["delivered_j"])
                      <= max(1e-6, account["delivered_j"]*1e-7))
            if "bench" in row:
                previous = row["bench"]["vessels"][0]
    else:
        check(f"{case}: mixed inventory partition coverage explicit",
              all(isinstance(e.get("energy_partition_complete"), bool) for e in transfers))
    text = (args.evidence / f"{case}.text.stdout").read_text()
    if any(e.get("energy_partition_complete") is False for e in transfers):
        check(f"{case}: incomplete partition narrated", "not fully established" in text)
        check(f"{case}: no exact chemical residual asserted", "chemistry/phase=" not in text)

result = dict(mode="baseline evidence integrity" if args.baseline else "post-repair contracts",
              binary_source_commit=receipt["binary_source_commit"],
              predictions_sha256=receipt["predictions_sha256"],
              passed=all(c["passed"] for c in checks), checks=checks, cases=facts)
(args.evidence / "verification.json").write_text(json.dumps(result, indent=2)+"\n")
print(f'{len(checks)} checks; {sum(not c["passed"] for c in checks)} failed')
for c in checks:
    if not c["passed"]:
        print(c)
raise SystemExit(0 if result["passed"] else 1)

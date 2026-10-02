"""Assess the same sealed-headspace script before and after the fix."""
import json
from pathlib import Path

base = Path(__file__).resolve().parent


def inventory(name):
    rows = [json.loads(line) for line in (base / name).read_text().splitlines()]
    seal = next(row for row in rows if row["operator"]["op"] == "seal")
    filtered = next(row for row in rows if row["operator"]["op"] == "filter")
    assert all(event["event"] != "solver_failed" for row in rows for event in row["events"])

    def nitrogen(vessel):
        return sum(p["moles"] for p in vessel["contents"] if p["species"] == "N2")

    initial = nitrogen(seal["bench"]["vessels"][0])
    source, receiver = filtered["bench"]["vessels"]
    return dict(initial=initial, source=nitrogen(source), receiver=nitrogen(receiver))


before = inventory("filter-before.jsonl")
after = inventory("filter-after.jsonl")
assert before["initial"] > 0 and before["source"] == before["receiver"] == 0
assert after["initial"] > 0 and after["receiver"] == 0
assert abs(after["source"] / after["initial"] - 1) < 1e-12
print(json.dumps(dict(before=before, after=after), indent=2))

"""Check independently predicted ownership and outlet-temperature contracts."""
import json
from pathlib import Path

root = Path(__file__).resolve().parent


def rows(revision, name):
    return [
        json.loads(line)
        for line in (root / f"{revision}-{name}.json.stdout").read_text().splitlines()
        if line.startswith("{")
    ]


def transferred(revision, name):
    data = rows(revision, name)
    index = next(i for i, row in enumerate(data) if row.get("operator", {}).get("op") == "transport")
    assert all(event["event"] != "solver_failed" for row in data for event in row.get("events", []))
    return data[index - 1], data[index]


def chloride(vessel):
    return sum(p["moles"] for p in vessel["contents"] if p["species"] == "Cl-")


initial, doubled = transferred("before", "duplicate-chain")
initial_chloride = chloride(initial["bench"]["vessels"][0])
combined_chloride = sum(chloride(v) for v in doubled["bench"]["vessels"])
assert initial_chloride > 0
assert abs(combined_chloride / initial_chloride - 2) < 1e-10

execution = json.loads((root / "after-execution.json").read_text())
assert all(r["exit_code"] == (1 if r["name"] == "duplicate-chain" else 0) for r in execution["runs"])
assert all(row.get("operator", {}).get("op") != "transport" for row in rows("after", "duplicate-chain"))
refusal = (root / "after-duplicate-chain.json.stderr").read_text()
assert "appears more than once" in refusal, refusal

temperatures = {}
for revision in ["before", "after"]:
    initial, output = transferred(revision, "hot-outlet")
    source = initial["bench"]["vessels"][0]["temperature"]
    receiver = next(v for v in output["bench"]["vessels"] if v["id"] == 2)
    actual = receiver["temperature"]
    assert abs(chloride(receiver) / 0.001 - 1) < 1e-8
    temperatures[revision] = dict(source=source, receiver=actual)
    if revision == "before":
        assert source - actual > 40
    else:
        assert abs(source - actual) < 0.001

print(json.dumps(dict(duplicate_before=dict(initial=initial_chloride, combined=combined_chloride),
                      duplicate_after="refused before publication", hot_outlet=temperatures), indent=2))

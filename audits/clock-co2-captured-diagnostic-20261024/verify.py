"""Check saved bytes and reproduce the failed ledger; never execute chemistry."""
from decimal import Decimal as D, getcontext
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
getcontext().prec = 45
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
result = json.loads((ROOT / "RESULT.json").read_text())
assert sha(ROOT / "diagnostic.stdout") == result["stdout_sha256"]
assert sha(ROOT / "diagnostic.stderr") == result["stderr_sha256"]
assert sha(ROOT / "open-base-wait.kero") == result["protocol_sha256"]
assert result["source"] == "a0177cff67cf13bcf460521d5bef34d026953296"
assert result["exit_status"] == 0
rows = [json.loads(line, parse_float=D) for line in (ROOT / "diagnostic.stdout").read_text().splitlines() if line.startswith("{")]
assert len(rows) == 4
assert rows[2]["operator"] == {"op": "wait", "seconds": D(3600)}
before, after = [rows[i]["bench"]["vessels"][0] for i in [1, 2]]
# Explicit formula scope of this one saved experiment; reject any other stock.
formulas = {
    "water": ({"H": 2, "O": 1}, 0),
    "base_equivalents": ({"H": 1, "O": 1}, -1),
    "Na+": ({"Na": 1}, 1),
    "CO2": ({"C": 1, "O": 2}, 0),
    "CO2(aq)": ({"C": 1, "O": 2}, 0),
    "HCO3-": ({"C": 1, "H": 1, "O": 3}, -1),
    "CO3-2": ({"C": 1, "O": 3}, -2),
}


def ledger(vessel):
    for field in ["unresolved_materials", "surfaces", "adsorbed", "exchanges", "solid_solutions", "material_objects", "electrodes"]:
        assert not vessel.get(field), field
    elements, charge = {}, D(0)
    for portion in vessel["contents"]:
        counts, q = formulas[portion["species"]]
        n = D(portion["moles"])
        assert n.is_finite() and n >= 0
        for element, count in counts.items():
            elements[element] = elements.get(element, D(0)) + count * n
        charge += q * n
    return elements, charge


expected, expected_charge = ledger(before)
for event in rows[2]["events"]:
    assert event["event"] != "solver_failed"
    if event["event"] not in ["gas_absorbed", "gas_evolved"]:
        continue
    assert event["vessel"] == before["id"]
    n = D(event["moles"])
    assert n.is_finite() and n >= 0
    counts, q = formulas[event["species"]]
    sign = 1 if event["event"] == "gas_absorbed" else -1
    for element, count in counts.items():
        expected[element] = expected.get(element, D(0)) + sign * count * n
    expected_charge += sign * q * n
actual, actual_charge = ledger(after)
saved = json.loads((ROOT / "LEDGER-ARITHMETIC.json").read_text())
for element in expected.keys() | actual.keys():
    error = actual.get(element, D(0)) - expected.get(element, D(0))
    assert error == D(saved["physical_elements"][element]["error"])
    if element != "O":
        assert abs(error) <= D("1e-10")
assert actual_charge - expected_charge == D(saved["formal_charge"]["error"])
assert abs(actual["O"] - expected["O"]) > D("1e-10")
assert abs(actual_charge - expected_charge) > D("1e-10")
print("Verified preserved FAILURE: oxygen and formal charge violate 1e-10; C/H/Na close. No native execution or qualification.")

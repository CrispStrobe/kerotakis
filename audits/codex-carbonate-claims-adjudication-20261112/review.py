#!/usr/bin/env python3
"""Independent saved-data checks; never executes an engine or writes evidence."""
import gzip
import hashlib
import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent
BINDINGS = json.loads((ROOT / "bindings.json").read_text(encoding="utf-8"))
for name, digest in BINDINGS["files_sha256"].items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest, name
original = tomllib.loads(gzip.decompress((ROOT / "aqueous.original.toml.gz").read_bytes()).decode())
revised = tomllib.loads(gzip.decompress((ROOT / "lint-input-v2-aqueous.toml.gz").read_bytes()).decode())
before = {entry["id"]: entry for entry in original["reaction"]}
after = {entry["id"]: entry for entry in revised["reaction"]}
FORMULAS = {
    "water": {"H": 2, "O": 1}, "CO2": {"C": 1, "O": 2},
    "CO2(aq)": {"C": 1, "O": 2}, "HCl": {"H": 1, "Cl": 1},
    "NaOH": {"Na": 1, "O": 1, "H": 1},
    "NaHCO3": {"Na": 1, "H": 1, "C": 1, "O": 3},
    "HCO3-": {"H": 1, "C": 1, "O": 3, "q": -1},
    "CO3-2": {"C": 1, "O": 3, "q": -2},
    "Na+": {"Na": 1, "q": 1}, "Cl-": {"Cl": 1, "q": -1},
    "H+": {"H": 1, "q": 1}, "base_equivalents": {"H": 1, "O": 1, "q": -1},
}


def totals(portions):
    result = {key: 0.0 for key in ("H", "O", "C", "Na", "Cl", "q")}
    for portion in portions:
        for key, count in FORMULAS[portion["species"]].items():
            result[key] += count * portion["moles"]
    return result


maximum_residual = 0.0
for name in ("baking-soda-vents-carbon-dioxide", "bicarbonate-plays-both-sides",
             "the-antacid-runs-on-arithmetic"):
    path = ROOT / name
    frames = [json.loads(line) for line in gzip.decompress((path / "stdout.jsonl.gz").read_bytes()).splitlines()]
    old, new = before[name], after[name]
    assert old["setup"] == new["setup"]
    assert old["expect"]["events"] == new["expect"]["events"]
    assert old["expect"]["predict"]["answer"] == new["expect"]["predict"]["answer"]
    assert new["expect"]["ph"]["max"] - new["expect"]["ph"]["min"] < 0.4
    assert (path / "experiment.lab").read_text(encoding="utf-8") == old["setup"]["script"]
    events = [event for frame in frames for event in frame["events"]]
    assert not any(event.get("event") == "solver_failed" for event in events)
    for vessel in frames[-1]["bench"]["vessels"]:
        exports = [event for event in events if event.get("vessel") == vessel["id"]
                   and event.get("event") == "gas_evolved"]
        imports = [event for event in events if event.get("vessel") == vessel["id"]
                   and event.get("event") == "gas_absorbed"]
        expected = totals(vessel["lots"] + imports)
        observed = totals(vessel["contents"] + exports)
        residual = max(abs(observed[key] - expected[key]) for key in expected)
        maximum_residual = max(maximum_residual, residual)
        assert residual < 1e-10, (name, vessel["id"], residual)
        ph = vessel["solution"]["ph"]
        if vessel["id"] == 0:
            assert new["expect"]["ph"]["min"] <= ph <= new["expect"]["ph"]["max"]
            assert not old["expect"]["ph"]["min"] <= ph <= old["expect"]["ph"]["max"]
        print(name, "vessel", vessel["id"] + 1, "pH", ph,
              "CO2 export mol", sum(event["moles"] for event in exports),
              "element/charge maximum residual", residual)
    if name != "baking-soda-vents-carbon-dioxide":
        assert any(event.get("reason", {}).get("key") == "not-modeled.carbonate-fast-endpoint"
                   for event in events)
print("PASS: original scripts/events/answer indices retained; narrower ranges; known element/charge closure", maximum_residual)

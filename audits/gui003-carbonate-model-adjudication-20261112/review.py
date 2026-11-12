#!/usr/bin/env python3
"""Review saved evidence only; no engine execution and no output overwrites."""
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
bindings = json.loads((ROOT / "bindings.json").read_text(encoding="utf-8"))
for name, expected in bindings["files_sha256"].items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name
raw = gzip.decompress((ROOT / "native.stdout.jsonl.gz").read_bytes())
assert hashlib.sha256(raw).hexdigest() == bindings["native_stdout_uncompressed_sha256"]
frames = [json.loads(line) for line in raw.splitlines()]
by_step = {frame["step"]: frame for frame in frames}
original = json.loads((ROOT / "gui003-dom-five.original.json").read_text(encoding="utf-8"))
actual = json.loads((ROOT / "browser-first-readback.json").read_text(encoding="utf-8"))
delta = json.loads((ROOT / "eight-leaf-delta.json").read_text(encoding="utf-8"))
allowed = {(0, vessel, field) for vessel in (1, 2) for field in (
    "boiling-k", "temperature-k", "srgb", "volume-l"
)}
assert len(delta) == 8
for change in delta:
    path = change["path"]
    assert path[:2] == [0, "vessels"]
    field = path[-1]
    assert (path[0], path[2], field) in allowed
    allowed.remove((path[0], path[2], field))
    assert path == ([0, "vessels", path[2], "liquids", 0, field]
                    if field in ("srgb", "volume-l") else [0, "vessels", path[2], field])
    owner = original
    for key in path[:-1]:
        owner = owner[key]
    assert owner[path[-1]] == change["old"]
    owner[path[-1]] = change["new"]
assert not allowed and original == actual

# Analytical base_equivalents carries H/O/charge as documented in species.rs.
# Pigment is checked as an unchanged separate inventory; opaque material has no
# invented formula. The simple solvent/carbonate stocks below are fully known.
FORMULAS = {
    "water": {"H": 2, "O": 1}, "CO2": {"C": 1, "O": 2},
    "CO2(aq)": {"C": 1, "O": 2},
    "HCO3-": {"H": 1, "C": 1, "O": 3, "q": -1},
    "CO3-2": {"C": 1, "O": 3, "q": -2},
    "NaHCO3": {"Na": 1, "H": 1, "C": 1, "O": 3},
    "Na+": {"Na": 1, "q": 1}, "H+": {"H": 1, "q": 1},
    "base_equivalents": {"H": 1, "O": 1, "q": -1},
}


def totals(portions):
    result = {key: 0.0 for key in ("H", "O", "C", "Na", "q")}
    for portion in portions:
        if portion["species"] == "anthocyanin":
            continue
        for key, count in FORMULAS[portion["species"]].items():
            result[key] += count * portion["moles"]
    return result


for vessel, step in ((1, 6), (2, 12)):
    frame = by_step[step]
    stock = frame["bench"]["vessels"][vessel]
    exports = [event for f in frames if f["step"] <= step
               for event in f.get("events", [])
               if event.get("event") == "gas_evolved" and event.get("vessel") == vessel]
    expected, observed = totals(stock["lots"]), totals(stock["contents"] + exports)
    assert max(abs(observed[key] - expected[key]) for key in expected) < 1e-10
    pigment = lambda portions: sum(p["moles"] for p in portions if p["species"] == "anthocyanin")
    assert pigment(stock["lots"]) == pigment(stock["contents"])
    for repeat in (step + 1, step + 2):
        assert by_step[repeat]["bench"]["vessels"][vessel] == stock
        assert not any(e.get("event") in ("gas_evolved", "temperature_changed")
                       for e in by_step[repeat]["events"])
    scene = frame["scene"]["vessels"][vessel]
    dom = actual[0]["vessels"][vessel]
    assert dom["temperature-k"] == f'{scene["temperature_k"]:.2f}'
    assert dom["boiling-k"] == f'{scene["boiling_point_k"]:.2f}'
    assert dom["liquids"][0]["volume-l"] == f'{scene["liquid"]["volume_l"]:.6f}'
    assert dom["liquids"][0]["srgb"] == ",".join(map(str, scene["liquid"]["srgb"]))
print("PASS: eight leaves only; native/browser agreement; known H/O/C/Na/charge closure; unchanged observations")

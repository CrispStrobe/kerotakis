#!/usr/bin/env python3
"""Compare native aqueous element projections with the analytical inventory.

Uses every emitted bench row, native molality times native solvent mass, and
the registry's formula-derived element counts. No experiment identifiers or
expected outputs are encoded. Also detects internal component-name leakage.

This is a projection consistency check, not a complete chemical mass balance:
native redox totals can exclude separately conserved organic/ligand pools.
A mismatch therefore requires inspection of component scope before it is
called an atom-conservation failure. Solids, gases, surfaces, and prepared
objects are outside this aqueous-portion comparison.
"""
import argparse
import collections
import hashlib
import json
import math
from pathlib import Path
import re


ROOT = Path(__file__).resolve().parents[2]
DIGITS = "zero|one|two|three|four|five|six|seven|eight|nine"
INTERNAL = re.compile(r"[NS](?:red|ox)(?:" + DIGITS + r")+|Thiocyanate(?:[+\-]|\d)")


def strings(value, path="$"):
    if isinstance(value, str):
        yield path, value
    elif isinstance(value, dict):
        for key, item in value.items():
            yield from strings(item, f"{path}.{key}")
    elif isinstance(value, list):
        for index, item in enumerate(value):
            yield from strings(item, f"{path}[{index}]")


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def check(directories, registry, absolute_tolerance, relative_tolerance):
    registry_bytes = registry.read_bytes()
    document = json.loads(registry_bytes)
    atoms = {
        record["species_id"]: {
            element["element"]: element["count"]["value"]
            for element in record["elements"]
        }
        for record in document["compositions"]
    }
    comparisons, failures, invalid, aliases, inputs = [], [], [], [], []
    rows_checked = 0
    states = set()
    files = sorted({p.resolve() for d in directories for p in d.glob("*/stdout.ndjson")})
    if not files:
        raise ValueError("no */stdout.ndjson files found")
    for path in files:
        raw = path.read_bytes()
        inputs.append({"path": str(path), "sha256": digest(raw)})
        for line_number, line in enumerate(raw.splitlines(), 1):
            if not line.strip():
                continue
            row = json.loads(line)
            origin = {"file": str(path), "line": line_number, "step": row.get("step")}
            for field, value in strings(row):
                if INTERNAL.search(value):
                    aliases.append({**origin, "field": field, "value": value})
            rows_checked += 1
            for vessel in row.get("bench", {}).get("vessels", []):
                solution = vessel.get("solution") or {}
                redox = solution.get("redox") or []
                kg = solution.get("solvent_kg")
                if not redox:
                    continue
                where = {**origin, "vessel": vessel["id"]}
                if kg is None or not math.isfinite(kg) or kg <= 0:
                    invalid.append({**where, "reason": "missing/invalid native solvent mass"})
                    continue
                aqueous = [p for p in vessel.get("contents", []) if p["phase"] == "aqueous"]
                unknown = sorted({p["species"] for p in aqueous if p["species"] not in atoms})
                if unknown:
                    invalid.append({**where, "reason": "missing registry composition", "species": unknown})
                    continue
                native = collections.defaultdict(float)
                for entry in redox:
                    native[entry["element"]] += entry["molality"] * kg
                    states.add((entry["element"], entry["oxidation"]))
                for element, total in sorted(native.items()):
                    inventory = sum(p["moles"] * atoms[p["species"]].get(element, 0) for p in aqueous)
                    if not math.isfinite(total) or not math.isfinite(inventory):
                        invalid.append({**where, "element": element, "reason": "nonfinite native/inventory total"})
                        continue
                    error = abs(total - inventory)
                    tolerance = absolute_tolerance + relative_tolerance * abs(total)
                    passed = error <= tolerance
                    comparison = {
                        **where, "element": element, "native_moles": total,
                        "aqueous_inventory_moles": inventory, "absolute_error_mol": error,
                        "tolerance_mol": tolerance, "passed": passed,
                    }
                    comparisons.append(comparison)
                    if not passed:
                        failures.append(comparison)
    return {
        "scope": "Native redox-element aqueous totals versus formula-counted aqueous portions; separate ligand pools may require a narrower comparison.",
        "registry": {"path": str(registry.resolve()), "sha256": digest(registry_bytes)},
        "inputs": inputs,
        "absolute_tolerance_mol": absolute_tolerance,
        "relative_tolerance": relative_tolerance,
        "rows_checked": rows_checked,
        "comparisons_checked": len(comparisons),
        "comparisons_passed": sum(c["passed"] for c in comparisons),
        "states_observed": [{"element": e, "oxidation": s} for e, s in sorted(states)],
        "maximum_absolute_error_mol": max((c["absolute_error_mol"] for c in comparisons), default=0.0),
        "comparisons": comparisons, "mismatches": failures,
        "invalid_comparisons": invalid, "internal_alias_leaks": aliases,
        "passed": bool(comparisons) and not (failures or invalid or aliases),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directories", nargs="+", type=Path)
    parser.add_argument("--registry", type=Path, default=ROOT / "data/registry/registry-source-v1.json")
    parser.add_argument("--absolute-tolerance", type=float, default=1e-8)
    parser.add_argument("--relative-tolerance", type=float, default=1e-6)
    parser.add_argument("--out", type=Path, help="New report file; refuses to overwrite evidence")
    args = parser.parse_args()
    for value in (args.absolute_tolerance, args.relative_tolerance):
        if not math.isfinite(value) or value < 0:
            parser.error("tolerances must be finite and nonnegative")
    result = check(args.directories, args.registry, args.absolute_tolerance, args.relative_tolerance)
    rendered = json.dumps(result, indent=2, allow_nan=False) + "\n"
    if args.out:
        with args.out.open("x") as output:
            output.write(rendered)
    else:
        print(rendered, end="")
    raise SystemExit(0 if result["passed"] else 1)


if __name__ == "__main__":
    main()

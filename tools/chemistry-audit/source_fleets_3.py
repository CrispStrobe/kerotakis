#!/usr/bin/env python3
"""Validate or record the separately frozen v3 source-informed fleets."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import source_fleets_2 as base

MANIFEST_DIR = Path(__file__).with_name("source-fleets-3")
SCHEMA = "kerotakis-source-fleet-v3"
FIRST_ID, LAST_ID = 777, 896
FAMILIES = ("water-energy", "aqueous-selectivity", "ionic-conductivity",
            "metal-ligand", "organic-equilibrium")
EXPECTED_RELATIONS = {family: 6 for family in FAMILIES}
EXPECTED_RELATION_TOTAL = 30
EXPECTED_CHECK_TOTAL = 150


def configure() -> None:
    base.MANIFEST_DIR = MANIFEST_DIR
    base.SCHEMA = SCHEMA
    base.FIRST_ID, base.LAST_ID = FIRST_ID, LAST_ID
    base.FAMILIES = FAMILIES
    base.EXPECTED_RELATIONS = EXPECTED_RELATIONS
    base.EXPECTED_RELATION_TOTAL = EXPECTED_RELATION_TOTAL
    base.EXPECTED_CHECK_TOTAL = EXPECTED_CHECK_TOTAL


def load_manifests(directory: Path = MANIFEST_DIR):
    configure()
    return base.load_manifests(directory)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifests", type=Path, default=MANIFEST_DIR)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--family")
    parser.add_argument("--validate", action="store_true")
    args = parser.parse_args()
    manifests, cases = load_manifests(args.manifests)
    if args.family:
        manifests = [m for m in manifests if m["family"] == args.family]
        cases = [(m, c) for m, c in cases if m["family"] == args.family]
        if not manifests:
            parser.error(f"unknown family: {args.family}")
    print(json.dumps({"families": len(manifests), "cases": len(cases),
                      "distinct_scripts": len({c['script'] for _, c in cases})}))
    if args.validate:
        return
    if not args.binary or not args.out:
        parser.error("recording requires --binary and --out")
    base.record(args.binary, args.out, manifests, cases)
    summary = args.out / "summary.json"
    value = json.loads(summary.read_text())
    value["schema"] = "kerotakis-source-evidence-v3"
    summary.write_text(json.dumps(value, indent=2) + "\n")


if __name__ == "__main__":
    main()

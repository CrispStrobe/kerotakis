#!/usr/bin/env python3
"""Require one complete analysis report for every frozen source fleet."""
import argparse
import json
from pathlib import Path

import source_fleets


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    manifests, cases = source_fleets.load_manifests()
    expected = {manifest["family"]: 24 + len(manifest["relations"])
                for manifest in manifests}
    reports = {}
    for path in args.directory.rglob("*-law-checks.json"):
        family = path.name.removesuffix("-law-checks.json")
        if family in reports:
            raise ValueError(f"duplicate report: {family}")
        reports[family] = json.loads(path.read_text())
    if set(reports) != set(expected):
        raise ValueError(f"report families differ: expected {sorted(expected)}, got {sorted(reports)}")
    errors = []
    for family, report in reports.items():
        if report.get("families") != 1 or report.get("cases") != 24:
            errors.append(f"{family}: wrong family/case count")
        if len(report.get("checks", [])) != expected[family]:
            errors.append(f"{family}: wrong check count")
        if report.get("unmet") != 0:
            errors.append(f"{family}: {report.get('unmet')} unmet")
    result = {"families": len(reports), "cases": len(cases),
              "checks": sum(expected.values()), "unmet_families": errors}
    print(json.dumps(result, indent=2))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()

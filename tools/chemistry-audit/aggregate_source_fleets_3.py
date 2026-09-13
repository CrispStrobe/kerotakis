#!/usr/bin/env python3
"""Require one complete report for every frozen v3 source fleet."""
import argparse
import json
from pathlib import Path

import source_fleets_3


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    _, cases = source_fleets_3.load_manifests()
    expected = {family: 30 for family in source_fleets_3.FAMILIES}
    reports = {}
    for path in args.directory.rglob("*-law-checks.json"):
        family = path.name.removesuffix("-law-checks.json")
        if family in reports:
            raise ValueError(f"duplicate report: {family}")
        if family in expected:
            reports[family] = json.loads(path.read_text())
    errors = []
    if set(reports) != set(expected):
        errors.append(f"report families differ: {sorted(reports)}")
    commits = {report.get("commit") for report in reports.values()}
    binaries = {report.get("binary_sha256") for report in reports.values()}
    for family, report in reports.items():
        if report.get("schema") != "kerotakis-source-analysis-v3":
            errors.append(f"{family}: wrong schema")
        if report.get("cases") != 24 or len(report.get("checks", [])) != expected[family]:
            errors.append(f"{family}: wrong case/check count")
        if report.get("unmet") != 0:
            errors.append(f"{family}: {report.get('unmet')} unmet")
    if len(commits) != 1 or None in commits:
        errors.append("family reports do not share one commit")
    if len(binaries) != 1 or None in binaries:
        errors.append("family reports do not share one binary")
    result = {"schema": "kerotakis-source-aggregate-v3", "families": len(reports),
              "cases": len(cases), "relations": 30, "checks": 150,
              "commit": next(iter(commits), None),
              "binary_sha256": next(iter(binaries), None),
              "unmet_families": errors}
    print(json.dumps(result, indent=2))
    raise SystemExit(bool(errors))


if __name__ == "__main__":
    main()

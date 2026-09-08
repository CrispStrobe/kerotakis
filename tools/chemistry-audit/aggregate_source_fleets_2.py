#!/usr/bin/env python3
"""Require one complete report for every frozen v2 source fleet."""
import argparse
import json
from pathlib import Path

import source_fleets_2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path); args = parser.parse_args()
    manifests, cases = source_fleets_2.load_manifests()
    expected = {family: 24 + count for family, count in source_fleets_2.EXPECTED_RELATIONS.items()}
    reports = {}
    for path in args.directory.rglob("*-law-checks.json"):
        family = path.name.removesuffix("-law-checks.json")
        if family in reports: raise ValueError(f"duplicate report: {family}")
        if family in expected: reports[family] = json.loads(path.read_text())
    if set(reports) != set(expected):
        raise ValueError(f"report families differ: expected {sorted(expected)}, got {sorted(reports)}")
    errors = []
    commits = {report.get("commit") for report in reports.values()}
    binaries = {report.get("binary_sha256") for report in reports.values()}
    if len(commits) != 1 or None in commits: errors.append("family reports do not share one commit")
    if len(binaries) != 1 or None in binaries: errors.append("family reports do not share one binary SHA-256")
    for family, report in reports.items():
        if report.get("schema") != "kerotakis-source-analysis-v2": errors.append(f"{family}: wrong schema")
        if report.get("families") != 1 or report.get("cases") != 24: errors.append(f"{family}: wrong family/case count")
        if len(report.get("checks", [])) != expected[family]: errors.append(f"{family}: wrong check count")
        if report.get("unmet") != 0: errors.append(f"{family}: {report.get('unmet')} unmet")
    result = {"schema": "kerotakis-source-aggregate-v2", "families": len(reports),
              "cases": len(cases), "relations": sum(n-24 for n in expected.values()),
              "checks": sum(expected.values()), "commit": next(iter(commits), None),
              "binary_sha256": next(iter(binaries), None), "unmet_families": errors}
    if result["relations"] != source_fleets_2.EXPECTED_RELATION_TOTAL:
        errors.append("frozen relation total drifted")
    if result["checks"] != source_fleets_2.EXPECTED_CHECK_TOTAL:
        errors.append("frozen check total drifted")
    print(json.dumps(result, indent=2)); raise SystemExit(bool(errors))


if __name__ == "__main__": main()

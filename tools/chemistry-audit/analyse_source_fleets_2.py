#!/usr/bin/env python3
"""Apply the proven analyzer to the separately frozen v2 manifest contract."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re

import analyse_source_fleets as v1
import source_fleets_2


def analyse(evidence_dir: Path, manifest_dir: Path = source_fleets_2.MANIFEST_DIR, family=None):
    summary = json.loads((evidence_dir / "summary.json").read_text())
    if summary.get("schema") != "kerotakis-source-evidence-v2":
        raise ValueError("v2 evidence has a missing or incorrect schema")
    commit = summary.get("commit")
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}(?:[0-9a-f]{24})?", commit):
        raise ValueError("v2 evidence has an invalid commit identity")
    original = v1.source_fleets
    try:
        v1.source_fleets = source_fleets_2
        result = v1.analyse(evidence_dir, manifest_dir, family)
    finally:
        v1.source_fleets = original
    result["schema"] = "kerotakis-source-analysis-v2"
    result["commit"] = commit
    result["binary_sha256"] = summary["binary_sha256"]
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, nargs="?")
    parser.add_argument("--manifests", type=Path, default=source_fleets_2.MANIFEST_DIR)
    parser.add_argument("--family"); parser.add_argument("--out", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        # The evaluator and all hostile evidence mutations remain shared and
        # are tested by the v1 harness; v2 separately validates its manifests.
        v1.self_test(); return
    if args.directory is None: parser.error("evidence directory required")
    result = analyse(args.directory, args.manifests, args.family)
    output = json.dumps(result, indent=2, allow_nan=False) + "\n"
    if args.out:
        with args.out.open("x") as handle: handle.write(output)
    print(output, end=""); raise SystemExit(bool(result["unmet"]))


if __name__ == "__main__": main()

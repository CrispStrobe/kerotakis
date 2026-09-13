#!/usr/bin/env python3
"""Apply the shared named-relation analyzer to frozen v3 evidence."""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import analyse_source_fleets as shared
import source_fleets_3


def analyse(directory: Path, manifests: Path, family: str | None):
    summary = json.loads((directory / "summary.json").read_text())
    if summary.get("schema") != "kerotakis-source-evidence-v3":
        raise ValueError("v3 evidence has a missing or incorrect schema")
    original = shared.source_fleets
    try:
        shared.source_fleets = source_fleets_3
        result = shared.analyse(directory, manifests, family)
    finally:
        shared.source_fleets = original
    result["schema"] = "kerotakis-source-analysis-v3"
    result["commit"] = summary.get("commit")
    result["binary_sha256"] = summary.get("binary_sha256")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, nargs="?")
    parser.add_argument("--manifests", type=Path, default=source_fleets_3.MANIFEST_DIR)
    parser.add_argument("--family")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        shared.self_test()
        return
    if args.directory is None:
        parser.error("evidence directory required")
    result = analyse(args.directory, args.manifests, args.family)
    output = json.dumps(result, indent=2, allow_nan=False) + "\n"
    if args.out:
        with args.out.open("x") as handle:
            handle.write(output)
    print(output, end="")
    raise SystemExit(bool(result["unmet"]))


if __name__ == "__main__":
    main()

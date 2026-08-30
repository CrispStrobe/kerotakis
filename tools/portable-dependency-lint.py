#!/usr/bin/env python3
"""Keep native-only solver adapters out of the portable runtime closure."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT_PACKAGE = "kerotakis-wasm"
NATIVE_ONLY = "native-only"


def dependency_paths(metadata: dict[str, Any]) -> list[list[str]]:
    """Return root-to-native-only package-name paths from Cargo metadata."""
    packages = {package["id"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    roots = [
        package["id"]
        for package in packages.values()
        if package["name"] == ROOT_PACKAGE
    ]
    if len(roots) != 1:
        raise ValueError(f"expected exactly one {ROOT_PACKAGE} package, found {len(roots)}")

    violations: list[list[str]] = []
    pending: list[tuple[str, list[str]]] = [(roots[0], [])]
    visited: set[str] = set()
    while pending:
        package_id, parent_path = pending.pop()
        if package_id in visited:
            continue
        visited.add(package_id)
        package = packages[package_id]
        path = [*parent_path, package["name"]]
        package_metadata = package.get("metadata") or {}
        marker = package_metadata.get("kerotakis", {}).get("runtime")
        if marker == NATIVE_ONLY:
            violations.append(path)
            continue
        pending.extend((dependency_id, path) for dependency_id in nodes[package_id]["dependencies"])
    return violations


def cargo_metadata(repo: Path) -> dict[str, Any]:
    command = [
        "cargo",
        "metadata",
        "--format-version",
        "1",
        "--filter-platform",
        "wasm32-unknown-unknown",
    ]
    result = subprocess.run(command, cwd=repo, check=True, capture_output=True, text=True)
    return json.loads(result.stdout)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--metadata",
        type=Path,
        help="read Cargo metadata JSON from this file instead of invoking Cargo",
    )
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    metadata = json.loads(args.metadata.read_text()) if args.metadata else cargo_metadata(repo)
    violations = dependency_paths(metadata)
    if not violations:
        print(f"portable dependency lint: {ROOT_PACKAGE} closure is portable")
        return 0

    print("portable dependency lint: native-only package reached from browser runtime", file=sys.stderr)
    for path in violations:
        print(f"  {' -> '.join(path)}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

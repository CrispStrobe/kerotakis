#!/usr/bin/env python3
"""Print a reviewed three-way snapshot merge as an apply_patch patch.

Merge metadata only; never invent solver outputs. Requires unmerged index
stages and resolved catalog TOML. Any overlapping field edits are refused.
The caller reviews/applies the printed patch and CI validates Rust's export.
"""
import difflib
import json
import pathlib
import subprocess
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
NAME = "crates/kerotakis-codex/tests/golden/codex-export.json"
MISSING = object()


def merge(base, ours, theirs, path=""):
    if ours == base or ours == theirs:
        return theirs
    if theirs == base:
        return ours
    if all(isinstance(value, dict) for value in (base, ours, theirs)):
        keys = list(theirs) + [key for key in ours if key not in theirs]
        result = {}
        for key in keys:
            value = merge(base.get(key, MISSING), ours.get(key, MISSING),
                          theirs.get(key, MISSING), path + "/" + key)
            if value is not MISSING:
                result[key] = value
        return result
    if all(isinstance(value, list) for value in (base, ours, theirs)) and all(
        isinstance(row, dict) and "id" in row
        for value in (base, ours, theirs) for row in value
    ):
        maps = [{row["id"]: row for row in value} for value in (base, ours, theirs)]
        assert all(len(mapping) == len(value) for mapping, value in zip(maps, (base, ours, theirs)))
        keys = list(maps[2]) + [key for key in maps[1] if key not in maps[2]]
        return [merge(*(mapping.get(key, MISSING) for mapping in maps), path + "/" + key)
                for key in keys]
    raise ValueError("Overlapping metadata edits require review: " + path)


if __name__ == "__main__":
    versions = [json.loads(subprocess.check_output(["git", "show", f":{stage}:{NAME}"], cwd=ROOT))
                for stage in (1, 2, 3)]
    merged = merge(*versions)
    entries = [entry for file in sorted((ROOT / "codex").glob("*.toml"))
               for entry in tomllib.loads(file.read_text()).get("reaction", [])]
    by_id = {entry["id"]: entry for entry in merged["reactions"]}
    assert len(by_id) == len(entries) and set(by_id) == {entry["id"] for entry in entries}
    # main introduced a mandatory authored progress field. New audit entries
    # take that explicit metadata from TOML, not a guessed schema default.
    merged["reactions"] = [dict(id=entry["id"], progress=entry["progress"],
                                 **{key: value for key, value in by_id[entry["id"]].items()
                                    if key not in {"id", "progress"}}) for entry in entries]
    before = (ROOT / NAME).read_text()
    after = json.dumps(merged, ensure_ascii=False, indent=2) + "\n"
    diff = list(difflib.unified_diff(before.splitlines(True), after.splitlines(True)))[2:]
    print("*** Begin Patch\n*** Update File: " + str(ROOT / NAME))
    for line in diff:
        print("@@" if line.startswith("@@") else line.rstrip("\n"))
    print("*** End Patch")

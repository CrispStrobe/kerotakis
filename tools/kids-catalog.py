#!/usr/bin/env python3
"""Validate and export the stable sixty-experiment children's catalog."""

import json
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
ALLOWED_STATUS = {"computed", "partial", "boundary", "declined", "unreachable"}
ALLOWED_SAFETY = {"home", "school"}
EXPECTED_STATUS_COUNTS = {
    "computed": 45, "partial": 9, "boundary": 2, "declined": 2, "unreachable": 2,
}


def validate(document: dict, root: pathlib.Path = ROOT) -> list[dict]:
    if document.get("schema") != 1:
        raise ValueError("kids catalog schema must be 1")
    rows = document.get("experiments")
    if not isinstance(rows, list):
        raise ValueError("experiments must be an array")
    expected = [f"K{i:02d}" for i in range(1, 61)]
    ids = [row.get("id") for row in rows]
    if ids != expected:
        raise ValueError("experiments must contain K01 through K60 exactly, in order")
    counts = {status: 0 for status in ALLOWED_STATUS}
    curiosity = root / "tests" / "coverage" / "curiosity-v1"
    manifest = tomllib.loads((curiosity / "manifest.toml").read_text())
    capability_ids = {
        prompt["id"]
        for shard in manifest["shards"]
        for prompt in tomllib.loads((curiosity / shard).read_text())["prompt"]
    }
    codex_ids = {
        reaction["id"]
        for source in (root / "codex").glob("*.toml")
        for reaction in tomllib.loads(source.read_text()).get("reaction", [])
    }
    for row in rows:
        kid = row["id"]
        for field in ("title", "phenomenon"):
            if not isinstance(row.get(field), str) or not row[field].strip():
                raise ValueError(f"{kid}: {field} must be non-empty")
        status = row.get("status")
        if status not in ALLOWED_STATUS:
            raise ValueError(f"{kid}: invalid status {status!r}")
        counts[status] += 1
        if row.get("safety") not in ALLOWED_SAFETY:
            raise ValueError(f"{kid}: safety must be home or school")
        for field in ("topics", "ingredients", "apparatus"):
            values = row.get(field)
            if not isinstance(values, list) or not values or not all(isinstance(v, str) and v for v in values):
                raise ValueError(f"{kid}: {field} must be a non-empty string array")
        if status in {"partial", "boundary", "declined", "unreachable"} and not row.get("boundary"):
            raise ValueError(f"{kid}: non-computed status requires a boundary")
        lesson = row.get("lesson")
        if lesson and (not re.fullmatch(r"[a-z0-9-]+\.lab", lesson) or not (root / "lessons" / lesson).is_file()):
            raise ValueError(f"{kid}: lesson link does not exist: {lesson}")
        quest = row.get("quest")
        if quest and (not re.fullmatch(r"[a-z0-9-]+", quest) or not (root / "quests" / f"{quest}.toml").is_file()):
            raise ValueError(f"{kid}: quest link does not exist: {quest}")
        if status in {"declined", "unreachable"} and (lesson or quest):
            raise ValueError(f"{kid}: {status} rows cannot carry a launch link")
        for field, known in (("capabilities", capability_ids), ("codex", codex_ids)):
            links = row.get(field, [])
            if not isinstance(links, list) or not all(isinstance(link, str) and link in known for link in links):
                raise ValueError(f"{kid}: {field} must contain existing exact identifiers")
    if counts != EXPECTED_STATUS_COUNTS:
        raise ValueError(f"status totals drifted from the audited matrix: {counts}")
    return rows


def add_translation(document: dict, translation: dict) -> dict:
    """Merge a complete locale file into export rows without changing English."""
    locale = translation.get("locale")
    if translation.get("schema") != document.get("schema") or locale != "de":
        raise ValueError("kids translation must use schema 1 and locale de")
    source_rows = document["experiments"]
    translated = translation.get("experiments")
    expected = [row["id"] for row in source_rows]
    if not isinstance(translated, list) or [row.get("id") for row in translated] != expected:
        raise ValueError("German catalog must contain the same K01 through K60 rows in order")
    by_id = {row["id"]: row for row in translated}
    for source in source_rows:
        target = by_id[source["id"]]
        required = ["title", "phenomenon"] + (["boundary"] if source.get("boundary") else [])
        if not source.get("boundary") and "boundary" in target:
            raise ValueError(f"{source['id']}: German boundary exists without an English boundary")
        for field in required:
            value = target.get(field)
            if not isinstance(value, str) or not value.strip():
                raise ValueError(f"{source['id']}: missing German {field}")
            source[f"{field}_{locale}"] = value
    return document


def main() -> None:
    source = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "data/kids/experiments-v1.json"
    document = json.loads(source.read_text())
    validate(document)
    translation = source.with_name("experiments-de-v1.json")
    add_translation(document, json.loads(translation.read_text()))
    if len(sys.argv) > 2:
        target = pathlib.Path(sys.argv[2])
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps(document, indent=2) + "\n")
    print(f"   {len(document['experiments'])} kids experiments validated")


if __name__ == "__main__":
    main()

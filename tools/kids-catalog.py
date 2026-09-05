#!/usr/bin/env python3
"""Validate and export the stable sixty-experiment children's catalog."""

import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
ALLOWED_STATUS = {"computed", "partial", "boundary", "declined", "unreachable"}
ALLOWED_SAFETY = {"home", "school"}
EXPECTED_STATUS_COUNTS = {
    "computed": 44, "partial": 8, "boundary": 3, "declined": 2, "unreachable": 3,
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
    if counts != EXPECTED_STATUS_COUNTS:
        raise ValueError(f"status totals drifted from the audited matrix: {counts}")
    return rows


def main() -> None:
    source = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "data/kids/experiments-v1.json"
    document = json.loads(source.read_text())
    validate(document)
    if len(sys.argv) > 2:
        target = pathlib.Path(sys.argv[2])
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps(document, indent=2) + "\n")
    print(f"   {len(document['experiments'])} kids experiments validated")


if __name__ == "__main__":
    main()

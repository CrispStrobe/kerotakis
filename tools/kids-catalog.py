#!/usr/bin/env python3
"""Validate and export the stable seventy-experiment guided catalog."""

import json
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
ALLOWED_STATUS = {"computed", "partial", "boundary", "declined", "unreachable"}
ALLOWED_SAFETY = {"home", "school"}
ALLOWED_PROGRESS = {"starter", "intermediate", "advanced"}
SAFETY_DETAIL_REQUIRED = {"K03", "K19", "K35", "K41", "K54", "K61", "K63", "K64", "K65", "K66", "K67", "K68", "K69", "K70"}
STRUCTURED_PREVIEW_REQUIRED = {"K02", "K04", "K26", "K31", "K33"}
KNOWN_KITS = {"balloon-kit", "candle-kit", "paper-chromatography-kit", "filter-funnel-kit", "magnet-kit"}
REQUIRED_KIT_BY_EXPERIMENT = {
    "K02": "balloon-kit", "K04": "candle-kit", "K26": "paper-chromatography-kit",
    "K31": "magnet-kit", "K33": "filter-funnel-kit",
}
EXPECTED_STATUS_COUNTS = {
    "computed": 62, "partial": 5, "boundary": 1, "declined": 2, "unreachable": 0,
}


def validate(document: dict, root: pathlib.Path = ROOT) -> list[dict]:
    if document.get("schema") != 1:
        raise ValueError("kids catalog schema must be 1")
    rows = document.get("experiments")
    if not isinstance(rows, list):
        raise ValueError("experiments must be an array")
    expected = [f"K{i:02d}" for i in range(1, 71)]
    ids = [row.get("id") for row in rows]
    if ids != expected:
        raise ValueError("experiments must contain K01 through K70 exactly, in order")
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
        if row.get("progress") not in ALLOWED_PROGRESS:
            raise ValueError(f"{kid}: progress must be starter, intermediate, or advanced")
        if kid in SAFETY_DETAIL_REQUIRED:
            for field in ("safety_rationale", "safety_guidance"):
                if not isinstance(row.get(field), str) or not row[field].strip():
                    raise ValueError(f"{kid}: {field} must be non-empty")
        for field in ("topics", "ingredients", "apparatus"):
            values = row.get(field)
            if not isinstance(values, list) or not values or not all(isinstance(v, str) and v for v in values):
                raise ValueError(f"{kid}: {field} must be a non-empty string array")
        for field in ("procedure", "observations", "kits"):
            values = row.get(field)
            if values is not None and (not isinstance(values, list) or not values or not all(isinstance(v, str) and v.strip() for v in values)):
                raise ValueError(f"{kid}: {field} must be a non-empty string array")
        recipe = row.get("recipe")
        if recipe is not None:
            if not isinstance(recipe, list) or not recipe:
                raise ValueError(f"{kid}: recipe must be a non-empty array")
            for line in recipe:
                if not isinstance(line, dict) or line.get("ingredient") not in row["ingredients"] or not isinstance(line.get("quantity"), str) or not line["quantity"].strip():
                    raise ValueError(f"{kid}: recipe lines need a listed ingredient and quantity")
                if "preparation" in line and (not isinstance(line["preparation"], str) or not line["preparation"].strip()):
                    raise ValueError(f"{kid}: recipe preparation must be non-empty")
        if any(kit not in KNOWN_KITS for kit in row.get("kits", [])):
            raise ValueError(f"{kid}: kits must contain existing exact kit identifiers")
        if kid in STRUCTURED_PREVIEW_REQUIRED and not all(row.get(field) for field in ("recipe", "procedure", "observations", "kits")):
            raise ValueError(f"{kid}: structured recipe, procedure, observations and kit are required")
        if kid in REQUIRED_KIT_BY_EXPERIMENT and row.get("kits") != [REQUIRED_KIT_BY_EXPERIMENT[kid]]:
            raise ValueError(f"{kid}: expected exact familiar kit {REQUIRED_KIT_BY_EXPERIMENT[kid]}")
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
        raise ValueError("German catalog must contain the same K01 through K70 rows in order")
    by_id = {row["id"]: row for row in translated}
    for source in source_rows:
        target = by_id[source["id"]]
        required = ["title", "phenomenon"] + (["boundary"] if source.get("boundary") else [])
        required += [field for field in ("safety_rationale", "safety_guidance") if source.get(field)]
        if not source.get("boundary") and "boundary" in target:
            raise ValueError(f"{source['id']}: German boundary exists without an English boundary")
        for field in required:
            value = target.get(field)
            if not isinstance(value, str) or not value.strip():
                raise ValueError(f"{source['id']}: missing German {field}")
            source[f"{field}_{locale}"] = value
        for field in ("procedure", "observations"):
            if source.get(field):
                value = target.get(field)
                if not isinstance(value, list) or len(value) != len(source[field]) or not all(isinstance(v, str) and v.strip() for v in value):
                    raise ValueError(f"{source['id']}: German {field} must match the English item count")
                source[f"{field}_{locale}"] = value
        if source.get("recipe"):
            value = target.get("recipe")
            if not isinstance(value, list) or len(value) != len(source["recipe"]):
                raise ValueError(f"{source['id']}: German recipe must match the English item count")
            for original, translated_line in zip(source["recipe"], value):
                if not isinstance(translated_line, dict) or translated_line.get("ingredient") != original["ingredient"] or not isinstance(translated_line.get("quantity"), str) or not translated_line["quantity"].strip():
                    raise ValueError(f"{source['id']}: German recipe must preserve ingredients and translate quantities")
                if original.get("preparation") and (not isinstance(translated_line.get("preparation"), str) or not translated_line["preparation"].strip()):
                    raise ValueError(f"{source['id']}: German recipe must translate preparation")
            source[f"recipe_{locale}"] = value
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

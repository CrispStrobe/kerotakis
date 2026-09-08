#!/usr/bin/env python3
"""Validate or record the independently frozen source-informed v2 fleets."""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
MANIFEST_DIR = Path(__file__).with_name("source-fleets-2")
SCHEMA = "kerotakis-source-fleet-v2"
FIRST_ID, LAST_ID = 489, 776
FAMILIES = (
    "quantitative-solutions", "polyprotic-carbonate", "redox-cells",
    "gas-production-transfer", "phase-colligative", "crystallisation-boundaries",
    "separation-trains", "kinetics-boundaries", "household-biochemistry",
    "materials-transport", "analytical-observables", "thermochemical-paths",
)
EXPECTED_RELATIONS = {family: 6 for family in FAMILIES} | {
    "gas-production-transfer": 8,
    "crystallisation-boundaries": 12,
}
EXPECTED_RELATION_TOTAL = 80
EXPECTED_CHECK_TOTAL = 368
ASSERTIONS = {
    "final-inventory-equal", "final-elements-equal", "case-elements-conserved",
    "final-scalar-equal", "final-scalar-order", "event-scalar-equal",
    "event-scalar-order", "event-present", "event-boundary-present",
    "event-components-total-order",
}
SUBSTANTIVE_ASSERTIONS = ASSERTIONS - {"event-present"}
RELATION_KINDS = {"conservation", "independent-law", "metamorphic", "boundary"}
FORBIDDEN_FIELDS = {"source", "sources", "source_note", "source_notes", "citation",
                    "citations", "author", "authors", "title", "isbn", "doi", "url"}
PRIVATE_TOKEN = re.compile(r"(?:https?://|www\.|\bISBN(?:-1[03])?\b|\bdoi\s*:|10\.\d{4,9}/\S+)", re.I)


def _privacy_check(value, where="manifest"):
    if isinstance(value, dict):
        for key, child in value.items():
            if key.lower() in FORBIDDEN_FIELDS:
                raise ValueError(f"private/source field {key!r} in {where}")
            _privacy_check(child, f"{where}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _privacy_check(child, f"{where}[{index}]")
    elif isinstance(value, str) and PRIVATE_TOKEN.search(value):
        raise ValueError(f"source-identifying token in {where}")


def load_manifests(directory: Path = MANIFEST_DIR):
    paths = sorted(directory.glob("*.json"))
    manifests = [json.loads(path.read_text()) for path in paths]
    if len(manifests) != len(FAMILIES):
        raise ValueError(f"expected {len(FAMILIES)} v2 manifests, found {len(manifests)}")
    by_family = {manifest.get("family"): manifest for manifest in manifests}
    if set(by_family) != set(FAMILIES) or len(by_family) != len(manifests):
        raise ValueError("v2 manifest families differ from the frozen family set")
    cases, identifiers, scripts, relation_ids = [], set(), set(), set()
    for family in FAMILIES:
        manifest = by_family[family]
        _privacy_check(manifest, family)
        if manifest.get("schema") != SCHEMA:
            raise ValueError(f"unsupported schema in {family}")
        family_cases = manifest.get("cases")
        if not isinstance(family_cases, list) or len(family_cases) != 24:
            raise ValueError(f"{family} must contain exactly 24 cases")
        family_ids = {case.get("id") for case in family_cases if isinstance(case, dict)}
        if len(family_ids) != 24 or None in family_ids:
            raise ValueError(f"{family} has duplicate or missing case ids")
        substantive = set()
        relations = manifest.get("relations")
        if not isinstance(relations, list) or not relations:
            raise ValueError(f"{family} has no relations")
        if len(relations) != EXPECTED_RELATIONS[family]:
            raise ValueError(
                f"{family} must retain exactly {EXPECTED_RELATIONS[family]} frozen relations"
            )
        for relation in relations:
            relation_id = relation.get("id")
            if not relation_id or relation_id in relation_ids:
                raise ValueError(f"duplicate or missing relation id: {relation_id}")
            relation_ids.add(relation_id)
            assertion = relation.get("assertion")
            if assertion not in ASSERTIONS or relation.get("kind") not in RELATION_KINDS:
                raise ValueError(f"unsupported relation in {relation_id}")
            linked = relation.get("cases")
            if not isinstance(linked, list) or not linked or len(linked) != len(set(linked)):
                raise ValueError(f"relation has missing or duplicate cases: {relation_id}")
            unknown = set(linked) - family_ids
            if unknown:
                raise ValueError(f"relation {relation_id} has unknown cases: {sorted(unknown)}")
            if not isinstance(relation.get("parameters"), dict):
                raise ValueError(f"relation has no parameter object: {relation_id}")
            if assertion in SUBSTANTIVE_ASSERTIONS:
                substantive.update(linked)
        if substantive != family_ids:
            raise ValueError(f"{family} cases lack a substantive relation: {sorted(family_ids-substantive)}")
        for spec in family_cases:
            key, script = spec.get("id"), spec.get("script")
            question = spec.get("question", "").strip()
            if key in identifiers or script in scripts:
                raise ValueError(f"duplicate id or script: {key}")
            if not isinstance(script, str) or not script.strip() or not question:
                raise ValueError(f"empty question or script: {key}")
            identifiers.add(key); scripts.add(script); cases.append((manifest, spec))
    numeric = sorted(int(spec["id"].split("-", 1)[0]) for _, spec in cases)
    if numeric != list(range(FIRST_ID, LAST_ID + 1)):
        raise ValueError(f"case ids must be the exact contiguous range {FIRST_ID}-{LAST_ID}")
    if len(relation_ids) != EXPECTED_RELATION_TOTAL:
        raise ValueError(f"v2 must retain exactly {EXPECTED_RELATION_TOTAL} relations")
    return [by_family[name] for name in FAMILIES], cases


def record(binary: Path, output: Path, manifests, cases):
    if output.exists() and any(output.iterdir()):
        raise ValueError("output directory is not empty; preserve it and choose a fresh path")
    output.mkdir(parents=True, exist_ok=True)
    binary = binary.resolve()
    envelope = {
        "schema": "kerotakis-source-evidence-v2",
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "manifest_sha256": {f"{m['family']}.json": hashlib.sha256(
            (MANIFEST_DIR / f"{m['family']}.json").read_bytes()).hexdigest() for m in manifests},
        "cases": [],
    }
    for manifest, spec in cases:
        directory = output / spec["id"]; directory.mkdir()
        script_text = "register lv3\n" + spec["script"].strip() + "\ninspect\n"
        (directory / "experiment.lab").write_text(f"# {spec['question']}\n{script_text}")
        started = time.monotonic()
        try:
            process = subprocess.run([str(binary), "run", str((directory / "experiment.lab").resolve()), "--json"],
                                     cwd=directory, capture_output=True, text=True, timeout=90)
            stdout, stderr, status = process.stdout, process.stderr, process.returncode
        except subprocess.TimeoutExpired as exc:
            stdout = exc.stdout.decode() if isinstance(exc.stdout, bytes) else exc.stdout or ""
            stderr = exc.stderr.decode() if isinstance(exc.stderr, bytes) else exc.stderr or ""
            status = "timeout"
        (directory / "stdout.ndjson").write_text(stdout); (directory / "stderr.txt").write_text(stderr)
        rows, malformed = [], []
        for line in stdout.splitlines():
            try: rows.append(json.loads(line))
            except ValueError: malformed.append(line)
        events = [event for row in rows for event in row.get("events", [])]
        final = next((row["bench"] for row in reversed(rows) if "bench" in row), None)
        envelope["cases"].append({
            "id": spec["id"], "family": manifest["family"], "question": spec["question"],
            "exit": status, "seconds": round(time.monotonic()-started, 3), "steps": len(rows),
            "non_json_lines": malformed,
            "event_counts": dict(collections.Counter(e.get("event", "<missing>") for e in events)),
            "diagnostics": [e for e in events if e.get("event") in ("not_yet_modeled", "solver_failed", "hazard_warning")],
            "final": final, "stderr": stderr,
        })
        print(spec["id"], "exit=", status, "steps=", len(rows), flush=True)
    (output / "summary.json").write_text(json.dumps(envelope, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifests", type=Path, default=MANIFEST_DIR)
    parser.add_argument("--binary", type=Path); parser.add_argument("--out", type=Path)
    parser.add_argument("--family"); parser.add_argument("--validate", action="store_true")
    args = parser.parse_args(); manifests, cases = load_manifests(args.manifests)
    if args.family:
        manifests = [m for m in manifests if m["family"] == args.family]
        cases = [(m, c) for m, c in cases if m["family"] == args.family]
        if not manifests: parser.error(f"unknown family: {args.family}")
    print(json.dumps({"families": len(manifests), "cases": len(cases),
                      "distinct_scripts": len({c['script'] for _, c in cases})}))
    if args.validate: return
    if not args.binary or not args.out: parser.error("recording requires --binary and --out")
    record(args.binary, args.out, manifests, cases)


if __name__ == "__main__": main()

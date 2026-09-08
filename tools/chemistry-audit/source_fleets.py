#!/usr/bin/env python3
"""Validate, list, or record immutable source-informed fleet manifests."""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
from pathlib import Path
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
MANIFEST_DIR = Path(__file__).with_name("source-fleets")
ASSERTIONS = {
    "final-inventory-equal", "final-elements-equal", "case-elements-conserved",
    "final-scalar-equal", "final-scalar-order", "event-scalar-equal",
    "event-scalar-order", "event-present", "event-boundary-present",
}
RELATION_KINDS = {"conservation", "independent-law", "metamorphic", "boundary"}


def load_manifests(directory: Path = MANIFEST_DIR):
    manifests = [json.loads(path.read_text()) for path in sorted(directory.glob("*.json"))]
    if not manifests:
        raise ValueError(f"no manifests in {directory}")
    cases, identifiers, scripts, relation_ids = [], set(), set(), set()
    for manifest in manifests:
        if manifest.get("schema") != "kerotakis-source-fleet-v1":
            raise ValueError(f"unsupported schema in {manifest.get('family', '<unknown>')}")
        if len(manifest.get("cases", [])) != 24:
            raise ValueError(f"{manifest['family']} must contain exactly 24 cases")
        family_cases = manifest.get("cases", [])
        family_ids = {case.get("id") for case in family_cases}
        if len(family_ids) != 24 or None in family_ids:
            raise ValueError(f"{manifest['family']} has duplicate or missing case ids")
        relations = manifest.get("relations", [])
        if not relations:
            raise ValueError(f"{manifest['family']} has no relations")
        participants = set()
        for relation in relations:
            relation_id = relation.get("id")
            if not relation_id or relation_id in relation_ids:
                raise ValueError(f"duplicate or missing relation id: {relation_id}")
            relation_ids.add(relation_id)
            if relation.get("kind") not in RELATION_KINDS:
                raise ValueError(f"unsupported or missing relation kind in {relation_id}")
            if relation.get("assertion") not in ASSERTIONS:
                raise ValueError(f"unsupported assertion in {relation_id}")
            linked = relation.get("cases")
            if not isinstance(linked, list) or not linked:
                raise ValueError(f"relation has no cases: {relation_id}")
            unknown = set(linked) - family_ids
            if unknown:
                raise ValueError(f"relation {relation_id} has unknown cases: {sorted(unknown)}")
            if not isinstance(relation.get("parameters"), dict):
                raise ValueError(f"relation has no parameter object: {relation_id}")
            participants.update(linked)
        for case in family_cases:
            key, script = case["id"], case["script"]
            question = case.get("question", "").strip()
            if key in identifiers or script in scripts:
                raise ValueError(f"duplicate id or script: {key}")
            if not question or not script.strip():
                raise ValueError(f"empty question or script: {key}")
            if "http://" in question or "https://" in question or "www." in question:
                raise ValueError(f"public source link in question: {key}")
            if key not in participants:
                raise ValueError(f"case has no independent relation: {key}")
            identifiers.add(key); scripts.add(script); cases.append((manifest, case))
    numeric = sorted(int(case["id"].split("-", 1)[0]) for _, case in cases)
    if numeric != list(range(numeric[0], numeric[-1] + 1)):
        raise ValueError("case ids must form one contiguous range")
    return manifests, cases


def record(binary: Path, output: Path, manifests, cases):
    if output.exists() and any(output.iterdir()):
        raise ValueError("output directory is not empty; preserve it and choose a fresh path")
    output.mkdir(parents=True, exist_ok=True)
    binary = binary.resolve()
    envelope = {
        "schema": "kerotakis-source-evidence-v1",
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "manifest_sha256": {
            f"{manifest['family']}.json": hashlib.sha256(
                (MANIFEST_DIR / f"{manifest['family']}.json").read_bytes()).hexdigest()
            for manifest in manifests
        },
        "cases": [],
    }
    for manifest, spec in cases:
        directory = output / spec["id"]
        directory.mkdir()
        script_text = "register lv3\n" + spec["script"].strip() + "\ninspect\n"
        (directory / "experiment.lab").write_text(f"# {spec['question']}\n{script_text}")
        started = time.monotonic()
        try:
            process = subprocess.run(
                [str(binary), "run", str((directory / "experiment.lab").resolve()), "--json"],
                cwd=directory, capture_output=True, text=True, timeout=90)
            stdout, stderr, status = process.stdout, process.stderr, process.returncode
        except subprocess.TimeoutExpired as exc:
            stdout = exc.stdout.decode() if isinstance(exc.stdout, bytes) else exc.stdout or ""
            stderr = exc.stderr.decode() if isinstance(exc.stderr, bytes) else exc.stderr or ""
            status = "timeout"
        (directory / "stdout.ndjson").write_text(stdout)
        (directory / "stderr.txt").write_text(stderr)
        rows, malformed = [], []
        for line in stdout.splitlines():
            try:
                rows.append(json.loads(line))
            except ValueError:
                malformed.append(line)
        events = [event for row in rows for event in row.get("events", [])]
        final = next((row["bench"] for row in reversed(rows) if "bench" in row), None)
        envelope["cases"].append({
            "id": spec["id"], "family": manifest["family"], "question": spec["question"],
            "exit": status, "seconds": round(time.monotonic() - started, 3),
            "steps": len(rows), "non_json_lines": malformed,
            "event_counts": dict(collections.Counter(e.get("event", "<missing>") for e in events)),
            "diagnostics": [e for e in events if e.get("event") in
                            ("not_yet_modeled", "solver_failed", "hazard_warning")],
            "final": final, "stderr": stderr,
        })
        print(spec["id"], "exit=", status, "steps=", len(rows), flush=True)
    (output / "summary.json").write_text(json.dumps(envelope, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifests", type=Path, default=MANIFEST_DIR)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--family")
    parser.add_argument("--validate", action="store_true")
    args = parser.parse_args()
    manifests, cases = load_manifests(args.manifests)
    if args.family:
        manifests = [m for m in manifests if m["family"] == args.family]
        cases = [(m, c) for m, c in cases if m["family"] == args.family]
        if not manifests:
            parser.error(f"unknown family: {args.family}")
    print(json.dumps({"families": len(manifests), "cases": len(cases),
                      "distinct_scripts": len({c['script'] for _, c in cases})}))
    if args.validate:
        return
    if not args.binary or not args.out:
        parser.error("recording requires --binary and --out")
    record(args.binary, args.out, manifests, cases)


if __name__ == "__main__":
    main()

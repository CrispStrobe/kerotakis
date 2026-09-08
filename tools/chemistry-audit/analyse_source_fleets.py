#!/usr/bin/env python3
"""Analyse source-informed fleet evidence using only named, frozen relations."""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import math
from pathlib import Path
import re
import tempfile

import source_fleets

ROOT = Path(__file__).resolve().parents[2]
KINDS = {"conservation", "independent-law", "metamorphic", "boundary"}
ASSERTIONS = {
    "final-inventory-equal", "final-elements-equal", "case-elements-conserved",
    "final-scalar-equal", "final-scalar-order", "event-scalar-equal",
    "event-scalar-order", "event-present", "event-boundary-present",
    "event-components-total-order",
}


def close(a, b, *, atol=1e-9, rtol=1e-6):
    return (math.isfinite(a) and math.isfinite(b)
            and abs(a - b) <= atol + rtol * max(abs(a), abs(b)))


def _finite_tree(value):
    if isinstance(value, bool) or value is None or isinstance(value, str):
        return True
    if isinstance(value, (int, float)):
        return math.isfinite(value)
    if isinstance(value, list):
        return all(_finite_tree(item) for item in value)
    if isinstance(value, dict):
        return all(isinstance(key, str) and _finite_tree(item)
                   for key, item in value.items())
    return False


def _path(value, path):
    if not isinstance(path, list) or not path:
        raise ValueError("path must be a nonempty component list")
    for component in path:
        if isinstance(component, bool) or not isinstance(component, (str, int)):
            raise ValueError("path components must be strings or integers")
        if isinstance(component, str):
            if not isinstance(value, dict) or component not in value:
                raise ValueError(f"missing path component: {component}")
        elif not isinstance(value, list) or component < 0 or component >= len(value):
            raise ValueError(f"invalid list path component: {component}")
        value = value[component]
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise ValueError("path does not resolve to a finite scalar")
    return float(value)


def _relation_path(relation):
    if "path" in relation and "field" in relation:
        raise ValueError("specify path or field, not both")
    if "field" in relation:
        if not isinstance(relation["field"], str) or not relation["field"]:
            raise ValueError("field must be a nonempty string")
        return [relation["field"]]
    path = relation["path"]
    if isinstance(path, str):
        match = re.fullmatch(r"v([1-9][0-9]*)\.([A-Za-z_][A-Za-z0-9_]*)", path)
        if not match:
            raise ValueError("string path must have the form vN.field")
        return ["vessels", int(match.group(1)) - 1, match.group(2)]
    return path


def _events(record, name):
    return [event for row in record["rows"] for event in row.get("events", [])
            if isinstance(event, dict) and event.get("event") == name]


def _event(record, relation, occurrence=None):
    matches = _events(record, relation["event"])
    occurrence = relation.get("occurrence", "last") if occurrence is None else occurrence
    if occurrence == "last":
        return matches[-1]
    if occurrence == "first":
        return matches[0]
    if isinstance(occurrence, int) and not isinstance(occurrence, bool) and occurrence >= 0:
        return matches[occurrence]
    raise ValueError("occurrence must be first, last, or a nonnegative integer")


def _before(record, operator):
    if not isinstance(operator, str) or not operator:
        raise ValueError("before_op must be a nonempty operator name")
    index = next(index for index, row in enumerate(record["rows"])
                 if row.get("operator", {}).get("op") == operator)
    return next(row["bench"] for row in reversed(record["rows"][:index])
                if isinstance(row.get("bench"), dict))


def _vessels(bench):
    vessels = bench.get("vessels")
    if not isinstance(vessels, list) or not vessels:
        raise ValueError("bench has no vessels")
    return vessels


def _inventory(bench, relation):
    species_filter = relation.get("species")
    phase_filter = relation.get("phases")
    if species_filter is not None and not isinstance(species_filter, list):
        raise ValueError("species must be a list")
    if phase_filter is not None and not isinstance(phase_filter, list):
        raise ValueError("phases must be a list")
    result = {}
    for vessel in _vessels(bench):
        for portion in vessel.get("contents", []):
            if species_filter is not None and portion["species"] not in species_filter:
                continue
            if phase_filter is not None and portion["phase"] not in phase_filter:
                continue
            key = f"{portion['species']}|{portion['phase']}"
            result[key] = result.get(key, 0.0) + float(portion["moles"])
    return result


def _elements(bench, composition, relation):
    wanted = relation.get("elements")
    if not isinstance(wanted, list) or not wanted or not all(isinstance(e, str) for e in wanted):
        raise ValueError("elements must be a nonempty string list")
    result = {element: 0.0 for element in wanted}
    for vessel in _vessels(bench):
        for portion in vessel.get("contents", []):
            formula = composition[portion["species"]]
            for element in wanted:
                result[element] += float(portion["moles"]) * formula.get(element, 0)
    return result


def _equal_maps(values, relation):
    atol, rtol = float(relation.get("atol", 1e-9)), float(relation.get("rtol", 1e-6))
    if atol < 0 or rtol < 0 or not math.isfinite(atol) or not math.isfinite(rtol):
        raise ValueError("tolerances must be finite and nonnegative")
    scale = float(relation.get("scale", 1.0))
    if not math.isfinite(scale):
        raise ValueError("scale must be finite")
    keys = set().union(*(value.keys() for value in values))
    return all(close(scale * values[0].get(key, 0.0), value.get(key, 0.0),
                     atol=atol, rtol=rtol) for value in values[1:] for key in keys)


def _ordered(values, relation):
    direction = relation.get("direction", "increasing")
    delta = float(relation.get("min_delta", 0.0))
    if direction not in ("increasing", "decreasing") or delta < 0 or not math.isfinite(delta):
        raise ValueError("invalid direction or min_delta")
    pairs = zip(values, values[1:])
    return all((right - left >= delta if direction == "increasing" else left - right >= delta)
               for left, right in pairs)


def _relation(relation, records, composition):
    assertion = relation["assertion"]
    if assertion not in ASSERTIONS:
        raise ValueError(f"unsupported assertion: {assertion}")
    case_ids = relation["cases"]
    if not isinstance(case_ids, list) or not case_ids or len(case_ids) != len(set(case_ids)):
        raise ValueError("relation cases must be a nonempty unique list")
    selected = [records[case_id] for case_id in case_ids]
    if assertion == "final-inventory-equal":
        values = [_inventory(record["final"], relation) for record in selected]
        return _equal_maps(values, relation), values
    if assertion == "final-elements-equal":
        values = [_elements(record["final"], composition, relation) for record in selected]
        return _equal_maps(values, relation), values
    if assertion == "case-elements-conserved":
        evidence, passed = {}, True
        for record in selected:
            initial = (_before(record, relation["before_op"])
                       if "before_op" in relation else record["initial"])
            pair = [_elements(initial, composition, relation),
                    _elements(record["final"], composition, relation)]
            evidence[record["id"]] = pair
            passed &= _equal_maps(pair, relation)
        return passed, evidence
    if assertion.startswith("final-scalar-"):
        values = [_path(record["final"], _relation_path(relation)) for record in selected]
    elif assertion.startswith("event-scalar-"):
        occurrences = relation.get("occurrence")
        if isinstance(occurrences, str) and "," in occurrences:
            occurrences = [item.strip() for item in occurrences.split(",")]
            if len(occurrences) != len(selected):
                raise ValueError("comma-separated occurrence count must match cases")
        else:
            occurrences = [occurrences] * len(selected)
        values = [_path(_event(record, relation, occurrence), _relation_path(relation))
                  for record, occurrence in zip(selected, occurrences)]
    elif assertion == "event-present":
        expected = relation.get("count")
        counts = [len(_events(record, relation["event"])) for record in selected]
        if expected is not None and (isinstance(expected, bool) or not isinstance(expected, int) or expected < 0):
            raise ValueError("count must be a nonnegative integer")
        return (all(count > 0 for count in counts) if expected is None
                else all(count == expected for count in counts)), counts
    elif assertion == "event-boundary-present":
        boundaries = [_event(record, relation).get("boundary") for record in selected]
        return all(isinstance(value, str) and bool(value.strip()) for value in boundaries), boundaries
    elif assertion == "event-components-total-order":
        values = []
        for record in selected:
            components = _event(record, relation).get("components")
            if not isinstance(components, list) or not components:
                raise ValueError("event has no component inventory")
            if any(not isinstance(component, list) or len(component) != 2
                   or not isinstance(component[0], str)
                   or isinstance(component[1], bool)
                   or not isinstance(component[1], (int, float))
                   for component in components):
                raise ValueError("event has a malformed component inventory")
            values.append(sum(float(component[1]) for component in components))
    else:  # pragma: no cover - dispatch exhaustiveness guard
        raise ValueError(f"unhandled assertion: {assertion}")
    if assertion.endswith("-equal"):
        return _equal_maps([{"value": value} for value in values], relation), values
    return _ordered(values, relation), values


def _normalized_relation(relation):
    """Merge the parameter object without permitting it to override identity fields."""
    parameters = relation.get("parameters", {})
    if not isinstance(parameters, dict):
        raise ValueError("relation parameters must be an object")
    overlap = set(parameters) & {"id", "kind", "cases", "assertion", "description", "parameters"}
    if overlap:
        raise ValueError(f"reserved relation parameters: {sorted(overlap)}")
    return {**parameters, **relation}


def analyse(evidence_dir: Path, manifest_dir: Path = source_fleets.MANIFEST_DIR, family=None):
    manifests, cases = source_fleets.load_manifests(manifest_dir)
    if family:
        manifests = [manifest for manifest in manifests if manifest["family"] == family]
        cases = [(manifest, case) for manifest, case in cases if manifest["family"] == family]
        if not manifests:
            raise ValueError(f"unknown family: {family}")
    specs = {case["id"]: (manifest, case) for manifest, case in cases}
    summary_path = evidence_dir / "summary.json"
    if not summary_path.is_file():
        raise ValueError("missing summary.json")
    summary = json.loads(summary_path.read_text())
    listed = summary.get("cases")
    if not isinstance(listed, list) or len(listed) != len(specs):
        raise ValueError("summary case count does not match manifests")
    if {item.get("id") for item in listed if isinstance(item, dict)} != set(specs):
        raise ValueError("summary IDs do not match manifests exactly")
    if not isinstance(summary.get("binary_sha256"), str) or len(summary["binary_sha256"]) != 64:
        raise ValueError("missing binary SHA-256")
    expected_hashes = {f"{m['family']}.json": hashlib.sha256(
        (manifest_dir / f"{m['family']}.json").read_bytes()).hexdigest() for m in manifests}
    if summary.get("manifest_sha256") != expected_hashes:
        raise ValueError("manifest hashes do not match frozen inputs")
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text())
    composition = {row["species_id"]: {e["element"]: e["count"]["value"]
                                          for e in row["elements"]}
                   for row in registry["compositions"]}
    records, checks = {}, []
    for item in listed:
        case_id = item["id"]
        directory = evidence_dir / case_id
        errors, rows = [], []
        for filename in ("experiment.lab", "stdout.ndjson", "stderr.txt"):
            if not (directory / filename).is_file():
                errors.append(f"missing {filename}")
        script = (directory / "experiment.lab").read_text() if (directory / "experiment.lab").is_file() else ""
        stdout = (directory / "stdout.ndjson").read_text() if (directory / "stdout.ndjson").is_file() else ""
        stderr = (directory / "stderr.txt").read_text() if (directory / "stderr.txt").is_file() else ""
        for number, line in enumerate(stdout.splitlines(), 1):
            try:
                row = json.loads(line)
                if not isinstance(row, dict):
                    raise ValueError("not an object")
                rows.append(row)
            except (ValueError, TypeError) as exc:
                errors.append(f"stdout line {number}: {exc}")
        benches = [row["bench"] for row in rows if isinstance(row.get("bench"), dict)]
        events = [event for row in rows for event in row.get("events", []) if isinstance(event, dict)]
        final = item.get("final")
        if not rows or not benches or final != benches[-1]:
            errors.append("missing or inconsistent final bench")
        if item.get("exit") != 0:
            errors.append(f"exit is {item.get('exit')!r}")
        if item.get("non_json_lines"):
            errors.append("recorder reports non-JSON output")
        if item.get("steps") != len(rows):
            errors.append("recorded step count differs from stdout")
        if item.get("stderr") != stderr:
            errors.append("recorded stderr differs from evidence file")
        actual_counts = dict(collections.Counter(event.get("event", "<missing>") for event in events))
        if item.get("event_counts") != actual_counts:
            errors.append("recorded event counts differ from stdout")
        actual_diagnostics = [event for event in events if event.get("event") in
                              ("not_yet_modeled", "solver_failed", "hazard_warning")]
        if item.get("diagnostics") != actual_diagnostics:
            errors.append("recorded diagnostics differ from stdout")
        if any(event.get("event") == "solver_failed" for event in events):
            errors.append("solver_failed event")
        if not _finite_tree(rows) or not _finite_tree(final):
            errors.append("nonfinite or invalid JSON value")
        if benches:
            try:
                for bench in benches:
                    for vessel in _vessels(bench):
                        for portion in vessel.get("contents", []):
                            if portion["species"] not in composition:
                                errors.append(f"unregistered species {portion['species']}")
                            amount = portion["moles"]
                            if isinstance(amount, bool) or not isinstance(amount, (int, float)) or amount < 0:
                                errors.append("invalid or negative inventory")
            except (KeyError, TypeError, ValueError) as exc:
                errors.append(f"invalid bench: {exc}")
        manifest, spec = specs[case_id]
        expected_script = "register lv3\n" + spec["script"].strip() + "\ninspect\n"
        if script != f"# {spec['question']}\n{expected_script}":
            errors.append("experiment script does not match manifest")
        if item.get("family") != manifest["family"] or item.get("question") != spec["question"]:
            errors.append("summary metadata differs from manifest")
        records[case_id] = {"id": case_id, "rows": rows, "initial": benches[0] if benches else {},
                            "final": final if isinstance(final, dict) else {}}
        checks.append({"check": f"{case_id}: complete finite registered execution",
                       "kind": "execution", "passed": not errors, "evidence": errors})
    for manifest in manifests:
        relation_ids = set()
        family_ids = {case["id"] for case in manifest["cases"]}
        for raw_relation in manifest.get("relations", []):
            try:
                relation = _normalized_relation(raw_relation)
            except (TypeError, ValueError) as exc:
                relation = raw_relation
                normalization_error = str(exc)
            else:
                normalization_error = None
            relation_id = relation.get("id")
            errors = []
            if normalization_error:
                errors.append(normalization_error)
            if not isinstance(relation_id, str) or not relation_id or relation_id in relation_ids:
                errors.append("relation ID missing or duplicate within family")
            relation_ids.add(relation_id)
            if relation.get("kind") not in KINDS:
                errors.append(f"unsupported kind: {relation.get('kind')!r}")
            relation_cases = relation.get("cases")
            if not isinstance(relation_cases, list):
                errors.append("relation cases must be a list")
            elif any(case_id not in family_ids for case_id in relation_cases):
                errors.append("relation references a case outside its family")
            try:
                passed, evidence = _relation(relation, records, composition) if not errors else (False, errors)
            except (KeyError, IndexError, StopIteration, TypeError, ValueError,
                    ZeroDivisionError, OverflowError) as exc:
                passed, evidence = False, {"missing_or_invalid_evidence": str(exc)}
            checks.append({"check": f"{manifest['family']}:{relation_id}",
                           "kind": relation.get("kind", "invalid"), "assertion": relation.get("assertion"),
                           "passed": bool(passed), "evidence": evidence})
    passed = sum(check["passed"] for check in checks)
    return {"schema": "kerotakis-source-analysis-v1", "families": len(manifests),
            "cases": len(cases), "checks": checks, "passed": passed,
            "unmet": len(checks) - passed}


def _fixture(directory, mutation=None):
    manifests = directory / "manifests"; manifests.mkdir(parents=True)
    cases = [{"id": f"{n}-fixture", "question": f"Question {n}?",
              "script": f"add v1 water {n - 218}.0mol"} for n in range(219, 243)]
    relations = [
        {"id": "water-ledger", "kind": "conservation", "cases": [c["id"] for c in cases],
         "assertion": "case-elements-conserved", "parameters": {"elements": ["H", "O"]}},
        {"id": "same-temperature", "kind": "metamorphic", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "final-scalar-equal", "parameters": {"path": "v1.temperature"}},
        {"id": "ordered-temperature", "kind": "independent-law", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "final-scalar-order", "parameters": {"path": "v1.temperature"}},
        {"id": "same-inventory", "kind": "metamorphic", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "final-inventory-equal", "parameters": {"species": ["water"]}},
        {"id": "same-elements", "kind": "conservation", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "final-elements-equal", "parameters": {"elements": ["H", "O"]}},
        {"id": "same-reading", "kind": "metamorphic", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "event-scalar-equal", "parameters": {"event": "measured", "field": "value"}},
        {"id": "ordered-reading", "kind": "independent-law", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "event-scalar-order", "parameters": {"event": "measured", "field": "value"}},
        {"id": "ordered-components", "kind": "independent-law", "cases": [cases[0]["id"], cases[1]["id"]],
         "assertion": "event-components-total-order", "parameters": {"event": "measured"}},
        {"id": "reading-present", "kind": "boundary", "cases": [cases[0]["id"]],
         "assertion": "event-present", "parameters": {"event": "measured", "count": 1}},
        {"id": "observed", "kind": "boundary", "cases": [cases[0]["id"]],
         "assertion": "event-boundary-present", "parameters": {"event": "measured"}},
    ]
    manifest = {"schema": "kerotakis-source-fleet-v1", "family": "fixture",
                "cases": cases, "relations": relations}
    manifest_path = manifests / "fixture.json"
    manifest_path.write_text(json.dumps(manifest))
    evidence = directory / "evidence"; evidence.mkdir()
    listed = []
    for index, case in enumerate(cases):
        case_dir = evidence / case["id"]; case_dir.mkdir()
        bench = {"vessels": [{"id": 0, "temperature": 298.15, "pressure": 101325.0,
                               "contents": [{"species": "water", "phase": "liquid", "moles": 1.0}]}]}
        rows = [{"bench": bench, "events": []}, {"bench": bench, "events": [
            {"event": "measured", "value": 1.0, "components": [["water", 1.0]],
             "boundary": "fixture boundary"}]}]
        (case_dir / "experiment.lab").write_text(
            f"# {case['question']}\nregister lv3\n{case['script']}\ninspect\n")
        (case_dir / "stdout.ndjson").write_text("\n".join(json.dumps(row) for row in rows) + "\n")
        (case_dir / "stderr.txt").write_text("")
        listed.append({"id": case["id"], "family": "fixture", "question": case["question"],
                       "exit": 0, "steps": 2, "non_json_lines": [],
                       "event_counts": {"measured": 1}, "diagnostics": [],
                       "final": bench, "stderr": ""})
    summary = {"binary_sha256": "0" * 64,
               "manifest_sha256": {"fixture.json": hashlib.sha256(manifest_path.read_bytes()).hexdigest()},
               "cases": listed}
    (evidence / "summary.json").write_text(json.dumps(summary))
    if mutation:
        mutation(evidence, cases)
    return evidence, manifests


def self_test():
    tests = {}
    with tempfile.TemporaryDirectory(prefix="kero-source-analysis-") as temporary:
        root = Path(temporary)
        evidence, manifests = _fixture(root / "valid")
        result = analyse(evidence, manifests)
        assert result["unmet"] == 0, result
        tests["valid"] = "passed"
        mutations = {
            "missing-evidence": lambda e, c: (e / c[0]["id"] / "stdout.ndjson").unlink(),
            "nonfinite": lambda e, c: (e / c[0]["id"] / "stdout.ndjson").write_text(
                (e / c[0]["id"] / "stdout.ndjson").read_text().replace("298.15", "NaN")),
            "unregistered-species": lambda e, c: (e / c[0]["id"] / "stdout.ndjson").write_text(
                (e / c[0]["id"] / "stdout.ndjson").read_text().replace('"water"', '"unobtainium"')),
            "missing-boundary": lambda e, c: (e / c[0]["id"] / "stdout.ndjson").write_text(
                (e / c[0]["id"] / "stdout.ndjson").read_text().replace('"fixture boundary"', '""')),
            "altered-inventory": lambda e, c: (e / c[0]["id"] / "stdout.ndjson").write_text(
                (e / c[0]["id"] / "stdout.ndjson").read_text().replace('"moles": 1.0', '"moles": 2.0', 1)),
            "malformed-components": lambda e, c: (e / c[0]["id"] / "stdout.ndjson").write_text(
                (e / c[0]["id"] / "stdout.ndjson").read_text().replace(
                    '[["water", 1.0]]', '[["water", 1.0], ["broken"]]', 1)),
        }
        for name, mutation in mutations.items():
            evidence, manifests = _fixture(root / name, mutation)
            result = analyse(evidence, manifests)
            assert result["unmet"] > 0, (name, result)
            tests[name] = "rejected"
    print(json.dumps({"safe_assertions": sorted(ASSERTIONS), "mutation_tests": tests}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, nargs="?")
    parser.add_argument("--manifests", type=Path, default=source_fleets.MANIFEST_DIR)
    parser.add_argument("--family")
    parser.add_argument("--out", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
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

#!/usr/bin/env python3
"""Validate experiment audit records (docs/EXPERIMENT-PROTOCOL.md).

Stdlib only: implements the JSON Schema 2020-12 subset the schema uses
(type, enum, required, additionalProperties, properties, items, $ref,
pattern, minLength, minItems) plus the protocol's cross-field rules:
unique ids, verdicts refer to declared expectations, and every
ENGINE_DISCREPANCY / SCRIPT_DISCREPANCY / UNSUPPORTED has a gap.

    python3 tools/experiment-audit.py FILE.json [...]
    python3 tools/experiment-audit.py --self-test
"""
from __future__ import annotations
import argparse, json, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "docs/schemas/experiment-audit.schema.json"
EXAMPLE = ROOT / "docs/schemas/experiment-audit.example.json"
NEEDS_GAP = {"ENGINE_DISCREPANCY", "SCRIPT_DISCREPANCY", "UNSUPPORTED"}
TYPES = {"object": dict, "array": list, "string": str, "integer": int,
         "number": (int, float), "null": type(None), "boolean": bool}


def _is(value, name):
    if name in ("integer", "number") and isinstance(value, bool):
        return False
    return isinstance(value, TYPES[name])


def check(node, schema, root, path, errors):
    if "$ref" in schema:
        ref = schema["$ref"]
        target = root
        for part in ref.lstrip("#/").split("/"):
            target = target[part]
        return check(node, target, root, path, errors)
    if "type" in schema:
        names = schema["type"] if isinstance(schema["type"], list) else [schema["type"]]
        if not any(_is(node, n) for n in names):
            errors.append(f"{path}: expected {'/'.join(names)}")
            return
    if "enum" in schema and node not in schema["enum"]:
        errors.append(f"{path}: {node!r} not in {schema['enum']}")
    if isinstance(node, str):
        if len(node) < schema.get("minLength", 0):
            errors.append(f"{path}: empty")
        if "pattern" in schema and not re.search(schema["pattern"], node):
            errors.append(f"{path}: {node!r} does not match {schema['pattern']}")
    if isinstance(node, list):
        if len(node) < schema.get("minItems", 0):
            errors.append(f"{path}: needs at least {schema['minItems']} items")
        if "items" in schema:
            for i, item in enumerate(node):
                check(item, schema["items"], root, f"{path}[{i}]", errors)
    if isinstance(node, dict):
        props = schema.get("properties", {})
        for key in schema.get("required", []):
            if key not in node:
                errors.append(f"{path}: missing {key}")
        for key, value in node.items():
            if key in props:
                check(value, props[key], root, f"{path}.{key}", errors)
            elif schema.get("additionalProperties") is False:
                errors.append(f"{path}: unknown field {key}")


def rules(records, errors):
    seen = set()
    for i, rec in enumerate(records):
        if not isinstance(rec, dict):
            continue
        rid = rec.get("id", f"[{i}]")
        if rid in seen:
            errors.append(f"{rid}: duplicate id")
        seen.add(rid)
        declared = {e.get("id") for e in rec.get("expectations", []) if isinstance(e, dict)}
        gaps = [g for g in rec.get("gaps", []) if isinstance(g, dict)]
        for v in rec.get("verdicts", []):
            if not isinstance(v, dict):
                continue
            x = v.get("expectation")
            if x not in declared:
                errors.append(f"{rid}: verdict for undeclared expectation {x}")
            if v.get("verdict") in NEEDS_GAP and not gaps:
                errors.append(f"{rid}: {x} is {v.get('verdict')} but the record has no gap")
            if v.get("verdict") in NEEDS_GAP and gaps and any("expectation" in g for g in gaps) \
                    and not any(g.get("expectation") in (x, None) for g in gaps):
                errors.append(f"{rid}: {x} is {v.get('verdict')} but no gap names it")


def validate(records, schema):
    errors: list[str] = []
    check(records, schema, schema, "$", errors)
    if isinstance(records, list):
        rules(records, errors)
    return errors


def self_test(schema):
    example = json.loads(EXAMPLE.read_text(encoding="utf-8"))
    assert validate(example, schema) == [], validate(example, schema)
    bad = json.loads(json.dumps(example))
    bad[0]["gaps"] = []
    assert any("no gap" in e for e in validate(bad, schema))
    bad = json.loads(json.dumps(example))
    bad[0]["verdicts"][0]["verdict"] = "PASS"
    assert any("not in" in e for e in validate(bad, schema))
    bad = json.loads(json.dumps(example))
    bad[0]["verdicts"][0]["expectation"] = "X9"
    assert any("undeclared" in e for e in validate(bad, schema))
    bad = json.loads(json.dumps(example))
    bad[0]["manual_text"] = "copied"
    assert any("unknown field" in e for e in validate(bad, schema))
    bad = example + example
    assert any("duplicate" in e for e in validate(bad, schema))
    print("experiment-audit self-test: ok")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("files", nargs="*", type=pathlib.Path)
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args(argv)
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    if args.self_test or not args.files:
        self_test(schema)
    status = 0
    for path in args.files:
        errors = validate(json.loads(path.read_text(encoding="utf-8")), schema)
        for e in errors:
            print(f"{path}: {e}")
        status |= bool(errors)
    return status


if __name__ == "__main__":
    sys.exit(main())

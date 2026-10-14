#!/usr/bin/env python3
"""Assess saved alias CLI streams; stock balance is reported by typed events."""
import argparse
import hashlib
import json
import math
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def pairs(items):
    value = {}
    for key, item in items:
        assert key not in value, "duplicate JSON key"
        value[key] = item
    return value


def finite(value):
    result = float(value)
    assert math.isfinite(result)
    return result


def strict(text):
    def constant(value):
        raise ValueError("nonfinite JSON constant: " + value)
    return json.loads(text, object_pairs_hook=pairs, parse_float=finite, parse_constant=constant)


def analyse(raw, design):
    protocol = strict((design / "protocol.json").read_text())
    expected = strict((design / "expectations.json").read_text())
    binding = strict((raw / "binding.json").read_text())
    assert binding["protocol_sha256"] == sha(design / "protocol.json")
    hosted = strict((design / "hosted-binding.json").read_text())
    assert binding["helper_sha256"] == hosted["helper_sha256"]
    for key in ["source_head", "binary_sha256", "lock_sha256", "artifact_id", "artifact_zip_sha256"]:
        assert binding[key] == protocol[key]
    for name, digest in protocol["files_sha256"].items():
        assert sha(design / name) == digest
    cases, semantic = [], []
    for ordinal, name in enumerate(["canonical", "mixed"], 1):
        record = strict((raw / f"{ordinal:02}.result.json").read_text())
        assert record["name"] == name and record["script_sha256"] == protocol["files_sha256"][f"scripts/{name}.lab"]
        for key, value in binding.items():
            assert record[key] == value
        for suffix, key in [("stdout.jsonl", "stdout_sha256"), ("stderr.txt", "stderr_sha256")]:
            assert sha(raw / f"{ordinal:02}.{suffix}") == record[key]
        rows = [strict(line) for line in (raw / f"{ordinal:02}.stdout.jsonl").read_text().splitlines()]
        adds = [row for row in rows if row["operator"]["op"] == "add"]
        assert len(adds) == 3 and rows
        vessel = lambda row: next(v for v in row["bench"]["vessels"] if v["id"] == 0)
        stocked = [e for row in rows for e in row["events"] if e["event"] == "shelf_stocked"]
        exhausted = [e for e in adds[2]["events"] if e["event"] == "stock_exhausted"]
        final = vessel(rows[-1])
        water = [p for p in final["contents"] if p["species"] == "water"]
        overdraw = rows.index(adds[2])
        checks = {
            "exit_zero": record["exit"] == expected["both_processes_exit"] and not record["timeout"],
            "stderr_empty": not (raw / f"{ordinal:02}.stderr.txt").read_bytes(),
            "final_water_moles": sum(p["moles"] for p in water) == expected["final_water_moles"],
            "canonical_inventory": bool(water) and not any(p["species"] == "H2O" for p in final["contents"]),
            "one_canonical_bottle_creation": len(stocked) == 1 and stocked[0]["key"] == "water" and stocked[0]["unit"] == "mole" and stocked[0]["amount"] == 1.0,
            "shared_finite_stock_balance": len(exhausted) == 1 and exhausted[0]["key"] == expected["overdraw_stock_exhausted_key"] and exhausted[0]["unit"] == "mole" and exhausted[0]["remaining"] == expected["final_stock_water_moles"] and exhausted[0]["requested"] == 0.75,
            "overdraw_inventory_unchanged": vessel(adds[1]) == vessel(adds[2]),
            "later_observations_preserve_inventory": all(row["operator"]["op"] == "measure" and vessel(row) == vessel(adds[2]) for row in rows[overdraw + 1:]),
        }
        cases.append({"name": name, "classification": "pass" if all(checks.values()) else "fail", "checks": checks, "record": record,
                      "stderr": (raw / f"{ordinal:02}.stderr.txt").read_text()})
        semantic.append([{key: row[key] for key in ["operator", "events", "bench"]} for row in rows])
    return {"binding": binding, "cases": cases, "semantic_rows_equal": semantic[0] == semantic[1],
            "all_pass": all(c["classification"] == "pass" for c in cases) and semantic[0] == semantic[1],
            "stock_evidence": "CLI bench JSON exposes vessels, not the stock map. Shared finite balance is verified from shelf_stocked and stock_exhausted events; map cardinality is checked separately by the exact-source native regressions.",
            "candidate_integrated": False, "empirical_calibration": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--raw", type=Path, required=True)
    parser.add_argument("--design", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    report = analyse(args.raw, args.design)
    with args.out.open("x") as f:
        f.write(json.dumps(report, indent=2, allow_nan=False) + "\n")
    print(json.dumps({"all_pass": report["all_pass"], "cases": len(report["cases"])}))

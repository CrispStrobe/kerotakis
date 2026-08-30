#!/usr/bin/env python3
"""BRD-071 decision-spike measurement and budget evaluation.

The probe must print one JSON object containing ``trace_sha256`` and a
non-empty ``step_ms`` array.  Timing thresholds are reported but deliberately
do not affect the process exit status: shared CI runners are not performance
reference machines.  Determinism and payload budgets are stable CI gates.
"""

from __future__ import annotations

import argparse
import gzip
import json
import math
import pathlib
import statistics
import subprocess
import sys
import time
from typing import Any, Sequence

MAX_WASM_GZIP_DELTA = 750 * 1024
NATIVE_P95_STEP_MS = 4.0
NATIVE_MAX_STEP_MS = 16.7


def parse_probe(text: str) -> dict[str, Any]:
    try:
        value = json.loads(text)
    except json.JSONDecodeError as error:
        raise ValueError(f"probe did not emit JSON: {error}") from error
    if not isinstance(value, dict):
        raise ValueError("probe JSON must be an object")
    trace = value.get("trace_sha256")
    if not isinstance(trace, str) or len(trace) != 64:
        raise ValueError("trace_sha256 must be a 64-character hexadecimal digest")
    try:
        bytes.fromhex(trace)
    except ValueError as error:
        raise ValueError("trace_sha256 must be hexadecimal") from error
    steps = value.get("step_ms")
    if not isinstance(steps, list) or not steps:
        raise ValueError("step_ms must be a non-empty array")
    if any(isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) or v < 0 for v in steps):
        raise ValueError("step_ms entries must be finite non-negative numbers")
    return {"trace_sha256": trace.lower(), "step_ms": [float(v) for v in steps]}


def percentile_nearest_rank(values: Sequence[float], percentile: float) -> float:
    if not values:
        raise ValueError("cannot take a percentile of an empty sample")
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered)))
    return ordered[rank - 1]


def gzip_size(path: pathlib.Path) -> int:
    # mtime=0 makes this byte-for-byte reproducible as well as size-stable.
    return len(gzip.compress(path.read_bytes(), compresslevel=9, mtime=0))


def payload_decision(baseline: pathlib.Path, candidate: pathlib.Path) -> dict[str, Any]:
    baseline_raw = baseline.stat().st_size
    candidate_raw = candidate.stat().st_size
    baseline_gzip = gzip_size(baseline)
    candidate_gzip = gzip_size(candidate)
    delta = candidate_gzip - baseline_gzip
    return {
        "baseline_raw_bytes": baseline_raw,
        "candidate_raw_bytes": candidate_raw,
        "raw_delta_bytes": candidate_raw - baseline_raw,
        "baseline_gzip_bytes": baseline_gzip,
        "candidate_gzip_bytes": candidate_gzip,
        "gzip_delta_bytes": delta,
        "gzip_delta_limit_bytes": MAX_WASM_GZIP_DELTA,
        "pass": delta <= MAX_WASM_GZIP_DELTA,
    }


def probe_decision(runs: Sequence[dict[str, Any]], wall_ms: Sequence[float]) -> dict[str, Any]:
    if not runs:
        raise ValueError("at least one probe run is required")
    traces = [run["trace_sha256"] for run in runs]
    steps = [step for run in runs for step in run["step_ms"]]
    p95 = percentile_nearest_rank(steps, 0.95)
    maximum = max(steps)
    return {
        "runs": len(runs),
        "samples": len(steps),
        "trace_sha256": traces[0],
        "deterministic": len(set(traces)) == 1,
        "step_median_ms": statistics.median(steps),
        "step_p95_ms": p95,
        "step_max_ms": maximum,
        "wall_median_ms": statistics.median(wall_ms),
        "p95_limit_ms": NATIVE_P95_STEP_MS,
        "max_limit_ms": NATIVE_MAX_STEP_MS,
        "performance_pass": p95 <= NATIVE_P95_STEP_MS and maximum <= NATIVE_MAX_STEP_MS,
        "timing_is_advisory": True,
    }


def run_probe(command: Sequence[str], repeats: int) -> tuple[list[dict[str, Any]], list[float]]:
    runs: list[dict[str, Any]] = []
    walls: list[float] = []
    for _ in range(repeats):
        started = time.perf_counter()
        completed = subprocess.run(command, check=True, text=True, capture_output=True)
        walls.append((time.perf_counter() - started) * 1000.0)
        runs.append(parse_probe(completed.stdout))
    return runs, walls


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-wasm", required=True, type=pathlib.Path)
    parser.add_argument("--candidate-wasm", required=True, type=pathlib.Path)
    parser.add_argument("--probe-command", required=True, type=pathlib.Path)
    parser.add_argument("--probe-arg", action="append", default=[])
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args(argv)
    if args.repeats < 2:
        parser.error("--repeats must be at least 2 to test deterministic replay")

    runs, wall_ms = run_probe([str(args.probe_command), *args.probe_arg], args.repeats)
    probe = probe_decision(runs, wall_ms)
    payload = payload_decision(args.baseline_wasm, args.candidate_wasm)
    report = {
        "schema": "kerotakis.brd071-evaluation.v1",
        "thresholds": {
            "native_p95_step_ms": NATIVE_P95_STEP_MS,
            "native_max_step_ms": NATIVE_MAX_STEP_MS,
            "wasm_gzip_delta_bytes": MAX_WASM_GZIP_DELTA,
        },
        "native_probe": probe,
        "wasm_payload": payload,
        # Only reproducible outcomes gate CI. Inspect performance_pass on a
        # named reference machine when making the roadmap decision.
        "stable_gate_pass": probe["deterministic"] and payload["pass"],
    }
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 0 if report["stable_gate_pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())

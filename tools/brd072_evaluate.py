#!/usr/bin/env python3
"""Evaluate the BRD-072 fluid-visual decision spike.

The candidate probe prints one JSON object.  It must exercise intentional
particle loss and report chemistry digests on both sides of that loss.  This
makes the visual/chemistry authority boundary a measured gate rather than a
claim in the decision record.

Frame timing is classified for named reference hardware.  It is advisory by
default because shared CI runners are not performance reference machines;
determinism, phase ordering, chemistry isolation, and payload size are stable
CI gates.
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
from typing import Any, Sequence

SCHEMA = "kerotakis.brd072-decision-report.v1"
FRAME_60_FPS_MS = 1000.0 / 60.0
FRAME_30_FPS_MS = 1000.0 / 30.0
MAX_WASM_GZIP_DELTA = 1024 * 1024


def _digest(value: Any, name: str) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise ValueError(f"{name} must be a 64-character hexadecimal digest")
    try:
        bytes.fromhex(value)
    except ValueError as error:
        raise ValueError(f"{name} must be hexadecimal") from error
    return value.lower()


def parse_probe(text: str) -> dict[str, Any]:
    try:
        value = json.loads(text)
    except json.JSONDecodeError as error:
        raise ValueError(f"probe did not emit JSON: {error}") from error
    if not isinstance(value, dict):
        raise ValueError("probe JSON must be an object")

    frames = value.get("frame_ms")
    if not isinstance(frames, list) or not frames:
        raise ValueError("frame_ms must be a non-empty array")
    if any(
        isinstance(frame, bool)
        or not isinstance(frame, (int, float))
        or not math.isfinite(frame)
        or frame < 0
        for frame in frames
    ):
        raise ValueError("frame_ms entries must be finite non-negative numbers")

    particles_before = value.get("particles_before")
    particles_after = value.get("particles_after")
    for count, name in ((particles_before, "particles_before"), (particles_after, "particles_after")):
        if isinstance(count, bool) or not isinstance(count, int) or count < 0:
            raise ValueError(f"{name} must be a non-negative integer")
    if particles_after >= particles_before:
        raise ValueError("probe must exercise particle loss (particles_after < particles_before)")
    if not isinstance(value.get("phase_order_matches"), bool):
        raise ValueError("phase_order_matches must be a boolean")

    return {
        "visual_trace_sha256": _digest(value.get("visual_trace_sha256"), "visual_trace_sha256"),
        "chemistry_before_sha256": _digest(value.get("chemistry_before_sha256"), "chemistry_before_sha256"),
        "chemistry_after_loss_sha256": _digest(
            value.get("chemistry_after_loss_sha256"), "chemistry_after_loss_sha256"
        ),
        "frame_ms": [float(frame) for frame in frames],
        "particles_before": particles_before,
        "particles_after": particles_after,
        "phase_order_matches": value["phase_order_matches"],
    }


def percentile_nearest_rank(values: Sequence[float], percentile: float) -> float:
    if not values:
        raise ValueError("cannot take a percentile of an empty sample")
    ordered = sorted(values)
    return ordered[max(1, math.ceil(percentile * len(ordered))) - 1]


def gzip_size(path: pathlib.Path) -> int:
    return len(gzip.compress(path.read_bytes(), compresslevel=9, mtime=0))


def payload_decision(baseline: pathlib.Path, candidate: pathlib.Path) -> dict[str, Any]:
    baseline_raw = baseline.stat().st_size
    candidate_raw = candidate.stat().st_size
    baseline_gzip = gzip_size(baseline)
    candidate_gzip = gzip_size(candidate)
    delta = candidate_gzip - baseline_gzip
    return {
        "baseline": "lightweight-fluidScene",
        "baseline_raw_bytes": baseline_raw,
        "candidate_raw_bytes": candidate_raw,
        "raw_delta_bytes": candidate_raw - baseline_raw,
        "baseline_gzip_bytes": baseline_gzip,
        "candidate_gzip_bytes": candidate_gzip,
        "gzip_delta_bytes": delta,
        "gzip_delta_limit_bytes": MAX_WASM_GZIP_DELTA,
        "pass": delta <= MAX_WASM_GZIP_DELTA,
    }


def probe_decision(runs: Sequence[dict[str, Any]]) -> dict[str, Any]:
    if len(runs) < 2:
        raise ValueError("at least two probe runs are required")
    frames = [frame for run in runs for frame in run["frame_ms"]]
    traces = [run["visual_trace_sha256"] for run in runs]
    chemistry_isolated = all(
        run["chemistry_before_sha256"] == run["chemistry_after_loss_sha256"] for run in runs
    )
    # A replay may not silently change the authoritative starting chemistry.
    chemistry_replay_stable = len({run["chemistry_before_sha256"] for run in runs}) == 1
    p95 = percentile_nearest_rank(frames, 0.95)
    maximum = max(frames)
    return {
        "runs": len(runs),
        "samples": len(frames),
        "visual_trace_sha256": traces[0],
        "visual_deterministic": len(set(traces)) == 1,
        "chemistry_unchanged_by_particle_loss": chemistry_isolated,
        "chemistry_replay_stable": chemistry_replay_stable,
        "phase_order_matches_authority": all(run["phase_order_matches"] for run in runs),
        "particles_lost_min": min(run["particles_before"] - run["particles_after"] for run in runs),
        "frame_median_ms": statistics.median(frames),
        "frame_p95_ms": p95,
        "frame_max_ms": maximum,
        "fps_60": {"frame_budget_ms": FRAME_60_FPS_MS, "p95_pass": p95 <= FRAME_60_FPS_MS},
        "fps_30": {"frame_budget_ms": FRAME_30_FPS_MS, "p95_pass": p95 <= FRAME_30_FPS_MS},
    }


def run_probe(command: Sequence[str], repeats: int) -> list[dict[str, Any]]:
    runs = []
    for _ in range(repeats):
        completed = subprocess.run(command, check=True, text=True, capture_output=True)
        runs.append(parse_probe(completed.stdout))
    return runs


def build_report(
    runs: Sequence[dict[str, Any]],
    baseline: pathlib.Path,
    candidate: pathlib.Path,
    reference_machine: str | None = None,
) -> dict[str, Any]:
    probe = probe_decision(runs)
    payload = payload_decision(baseline, candidate)
    stable_gate = (
        probe["visual_deterministic"]
        and probe["chemistry_unchanged_by_particle_loss"]
        and probe["chemistry_replay_stable"]
        and probe["phase_order_matches_authority"]
        and payload["pass"]
    )
    timing_pass = probe["fps_30"]["p95_pass"]
    decision = "go" if stable_gate and (reference_machine is None or timing_pass) else "no-go"
    return {
        "schema": SCHEMA,
        "candidate": "salva-fluid-visual",
        "comparison": "existing-lightweight-fluidScene",
        "thresholds": {
            "fps_60_frame_ms": FRAME_60_FPS_MS,
            "fps_30_frame_ms": FRAME_30_FPS_MS,
            "wasm_gzip_delta_bytes": MAX_WASM_GZIP_DELTA,
        },
        "reference_machine": reference_machine,
        "timing_is_advisory": reference_machine is None,
        "probe": probe,
        "wasm_payload": payload,
        "stable_gate_pass": stable_gate,
        "decision": decision,
        "decision_reason": (
            "all stable gates pass; reference timing is advisory"
            if stable_gate and reference_machine is None
            else "all stable gates and the reference 30 fps budget pass"
            if stable_gate and timing_pass
            else "stable gates pass, but the named reference misses the 30 fps budget"
            if stable_gate
            else "one or more required gates failed"
        ),
    }


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline-wasm", required=True, type=pathlib.Path)
    parser.add_argument("--candidate-wasm", required=True, type=pathlib.Path)
    parser.add_argument("--probe-command", required=True, type=pathlib.Path)
    parser.add_argument("--probe-arg", action="append", default=[])
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--reference-machine")
    parser.add_argument("--output", type=pathlib.Path)
    args = parser.parse_args(argv)
    if args.repeats < 2:
        parser.error("--repeats must be at least 2")
    runs = run_probe([str(args.probe_command), *args.probe_arg], args.repeats)
    report = build_report(
        runs, args.baseline_wasm, args.candidate_wasm, args.reference_machine
    )
    rendered = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(rendered, encoding="utf-8")
    else:
        sys.stdout.write(rendered)
    return 0 if report["decision"] == "go" else 1


if __name__ == "__main__":
    raise SystemExit(main())

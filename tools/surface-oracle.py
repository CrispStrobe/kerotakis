#!/usr/bin/env python3
"""Independent, development-only check for the HFO zinc adsorption edge.

This is deliberately a small second implementation, not a runtime dependency.
It solves the intrinsic mass-action and finite-site balances from the approved
USGS ``wateq4f.dat`` constants with only the Python standard library.  It does
*not* implement PHREEQC's diffuse-layer electrostatics or its complete aqueous
speciation, so it is an oracle for the direction and approximate position of
the pH edge, not an attempted replacement for IPhreeqc.

Reaktoro is not used here.  Reaktoro 2.13 can load PHREEQC databases, but its
surface-complexation support is still an open upstream gap.  Keeping that LGPL
tool outside the repository would satisfy the distribution boundary, but it
cannot answer this particular benchmark.

The candidate file is ephemeral development output.  The script emits only
aggregate errors and provenance; do not check the per-case candidate output
into the repository.

Candidate schema::

    {
      "schema": 1,
      "benchmark": "hfo-zinc-ph-edge-v1",
      "producer": {"name": "kerotakis", "version": "<git revision>"},
      "retrieved": "YYYY-MM-DD",
      "cases": [
        {"id": "acidic", "ph": 2.1, "bound_zinc_mol": 1e-9,
         "total_zinc_mol": 1e-4}
      ]
    }

Usage::

    python3 tools/surface-oracle.py \
      --candidate /tmp/kerotakis-hfo.json \
      --database vendor/iphreeqc/database/wateq4f.dat

Only the JSON written to stdout is suitable for review as a persisted AQ-006
result.  The USGS database stays in its already-approved runtime-data lane;
this script neither copies nor exports its records.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import sys
from typing import Any


SCHEMA = 1
BENCHMARK = "hfo-zinc-ph-edge-v1"
ORACLE_VERSION = "1"
MAX_ABSOLUTE_BOUND_FRACTION_ERROR = 0.25
MAX_MEAN_BOUND_FRACTION_ERROR = 0.10

# HFO site populations used by SurfaceSites in the live benchmark.
STRONG_CAPACITY_MOL = 5.0e-6
WEAK_CAPACITY_MOL = 2.0e-4
SOLUTION_WATER_KG = 1.0
SULFATE_TOTAL_MOL = 1.0e-4

# Intrinsic constants from the approved USGS wateq4f.dat HFO model.
LOG_K_PROTONATION = 7.29
LOG_K_DEPROTONATION = -8.93
LOG_K_ZINC_STRONG = 0.99
LOG_K_ZINC_WEAK = -1.99
LOG_K_SULFATE_PROTONATED = 7.78
LOG_K_SULFATE_UNPROTONATED = 0.79


class InputError(ValueError):
    """The candidate cannot be compared without guessing its meaning."""


def _finite_number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise InputError(f"{label} must be a number")
    answer = float(value)
    if not math.isfinite(answer):
        raise InputError(f"{label} must be finite")
    return answer


def _site_distribution(
    ph: float, free_zinc: float, free_sulfate: float, capacity: float, log_k_zinc: float,
    *, sulfate_binding: bool,
) -> tuple[float, float]:
    """Return (bound zinc, bound sulfate) for one intrinsic site population."""
    protonated = 10.0 ** (LOG_K_PROTONATION - ph)
    deprotonated = 10.0 ** (LOG_K_DEPROTONATION + ph)
    zinc = 10.0 ** (log_k_zinc + ph) * free_zinc
    sulfate_h = 0.0
    sulfate_oh = 0.0
    if sulfate_binding:
        sulfate_h = 10.0 ** (LOG_K_SULFATE_PROTONATED - ph) * free_sulfate
        sulfate_oh = 10.0 ** LOG_K_SULFATE_UNPROTONATED * free_sulfate
    denominator = 1.0 + protonated + deprotonated + zinc + sulfate_h + sulfate_oh
    return capacity * zinc / denominator, capacity * (sulfate_h + sulfate_oh) / denominator


def _bound(ph: float, free_zinc: float, free_sulfate: float) -> tuple[float, float]:
    strong_zinc, _ = _site_distribution(
        ph,
        free_zinc,
        free_sulfate,
        STRONG_CAPACITY_MOL,
        LOG_K_ZINC_STRONG,
        sulfate_binding=False,
    )
    weak_zinc, weak_sulfate = _site_distribution(
        ph,
        free_zinc,
        free_sulfate,
        WEAK_CAPACITY_MOL,
        LOG_K_ZINC_WEAK,
        sulfate_binding=True,
    )
    return strong_zinc + weak_zinc, weak_sulfate


def _bisect_free(total: float, residual) -> float:
    lo = 0.0
    hi = total / SOLUTION_WATER_KG
    for _ in range(100):
        mid = 0.5 * (lo + hi)
        if residual(mid) > 0.0:
            hi = mid
        else:
            lo = mid
    return 0.5 * (lo + hi)


def oracle_bound_zinc(ph: float, total_zinc: float) -> float:
    """Solve independent Zn/S finite-site balances at a fixed pH."""
    if not 0.0 <= ph <= 14.0:
        raise InputError(f"pH must be between 0 and 14, found {ph}")
    if total_zinc <= 0.0:
        raise InputError("total_zinc_mol must be positive")

    free_zinc = total_zinc / SOLUTION_WATER_KG
    free_sulfate = SULFATE_TOTAL_MOL / SOLUTION_WATER_KG
    for _ in range(100):
        previous = (free_zinc, free_sulfate)
        free_zinc = _bisect_free(
            total_zinc,
            lambda trial: trial * SOLUTION_WATER_KG
            + _bound(ph, trial, free_sulfate)[0]
            - total_zinc,
        )
        free_sulfate = _bisect_free(
            SULFATE_TOTAL_MOL,
            lambda trial: trial * SOLUTION_WATER_KG
            + _bound(ph, free_zinc, trial)[1]
            - SULFATE_TOTAL_MOL,
        )
        scale = max(total_zinc, SULFATE_TOTAL_MOL, 1.0e-30)
        if max(abs(free_zinc - previous[0]), abs(free_sulfate - previous[1])) / scale < 1.0e-13:
            break
    else:
        raise RuntimeError("intrinsic HFO balance did not converge")
    return _bound(ph, free_zinc, free_sulfate)[0]


def validate_candidate(document: Any) -> list[dict[str, Any]]:
    if not isinstance(document, dict):
        raise InputError("candidate must be a JSON object")
    if document.get("schema") != SCHEMA:
        raise InputError(f"candidate schema must be {SCHEMA}")
    if document.get("benchmark") != BENCHMARK:
        raise InputError(f"candidate benchmark must be {BENCHMARK!r}")
    producer = document.get("producer")
    if not isinstance(producer, dict) or not producer.get("name") or not producer.get("version"):
        raise InputError("candidate producer needs non-empty name and version")
    retrieved = document.get("retrieved")
    if not isinstance(retrieved, str) or len(retrieved) != 10:
        raise InputError("candidate retrieved needs a YYYY-MM-DD date")
    cases = document.get("cases")
    if not isinstance(cases, list) or len(cases) < 3:
        raise InputError("candidate needs at least three pH cases")

    checked: list[dict[str, Any]] = []
    ids: set[str] = set()
    for index, case in enumerate(cases):
        if not isinstance(case, dict):
            raise InputError(f"cases[{index}] must be an object")
        case_id = case.get("id")
        if not isinstance(case_id, str) or not case_id:
            raise InputError(f"cases[{index}].id must be non-empty")
        if case_id in ids:
            raise InputError(f"duplicate case id {case_id!r}")
        ids.add(case_id)
        ph = _finite_number(case.get("ph"), f"cases[{index}].ph")
        bound = _finite_number(case.get("bound_zinc_mol"), f"cases[{index}].bound_zinc_mol")
        total = _finite_number(case.get("total_zinc_mol"), f"cases[{index}].total_zinc_mol")
        if not 0.0 <= ph <= 14.0:
            raise InputError(f"cases[{index}].ph must be between 0 and 14")
        if total <= 0.0 or not 0.0 <= bound <= total:
            raise InputError(f"cases[{index}] needs 0 <= bound_zinc_mol <= total_zinc_mol")
        checked.append({"id": case_id, "ph": ph, "bound": bound, "total": total})
    return checked


def summarize(document: dict[str, Any], database_sha256: str) -> dict[str, Any]:
    cases = validate_candidate(document)
    comparisons = []
    for case in cases:
        expected = oracle_bound_zinc(case["ph"], case["total"])
        comparisons.append(
            {
                "ph": case["ph"],
                "candidate_fraction": case["bound"] / case["total"],
                "oracle_fraction": expected / case["total"],
            }
        )

    errors = [abs(item["candidate_fraction"] - item["oracle_fraction"]) for item in comparisons]
    by_ph = sorted(comparisons, key=lambda item: item["ph"])
    candidate_monotonic = all(
        right["candidate_fraction"] > left["candidate_fraction"]
        for left, right in zip(by_ph, by_ph[1:])
    )
    oracle_monotonic = all(
        right["oracle_fraction"] > left["oracle_fraction"]
        for left, right in zip(by_ph, by_ph[1:])
    )

    maximum_error = max(errors)
    mean_error = sum(errors) / len(errors)
    passed = (
        maximum_error <= MAX_ABSOLUTE_BOUND_FRACTION_ERROR
        and mean_error <= MAX_MEAN_BOUND_FRACTION_ERROR
        and candidate_monotonic
        and oracle_monotonic
    )

    return {
        "schema": SCHEMA,
        "benchmark": BENCHMARK,
        "oracle": {
            "name": "Kerotakis independent intrinsic HFO mass-action balance",
            "version": ORACLE_VERSION,
            "model": "finite strong/weak sites; intrinsic acid/base, zinc, and weak-site sulfate equilibria; no diffuse-layer electrostatics or aqueous side complexes",
            "database": "USGS wateq4f.dat",
            "database_sha256": database_sha256,
        },
        "candidate": document["producer"],
        "candidate_retrieved": document["retrieved"],
        "case_count": len(cases),
        "metrics": {
            "max_absolute_bound_fraction_error": maximum_error,
            "mean_absolute_bound_fraction_error": mean_error,
            "candidate_strictly_increases_with_ph": candidate_monotonic,
            "oracle_strictly_increases_with_ph": oracle_monotonic,
            "monotonic_agreement": candidate_monotonic == oracle_monotonic,
        },
        "acceptance": {
            "max_absolute_bound_fraction_error": MAX_ABSOLUTE_BOUND_FRACTION_ERROR,
            "max_mean_bound_fraction_error": MAX_MEAN_BOUND_FRACTION_ERROR,
            "requires_both_strictly_increase_with_ph": True,
            "passed": passed,
        },
        "distributability": {
            "decision": "approved-aggregate-only",
            "reason": "project-owned calculation over already-approved USGS constants; contains only aggregate errors and verdicts, not database records or external-tool output",
        },
    }


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, required=True, help="ephemeral Kerotakis JSON")
    parser.add_argument("--database", type=Path, required=True, help="approved wateq4f.dat used by the run")
    args = parser.parse_args(argv)
    try:
        document = json.loads(args.candidate.read_text(encoding="utf-8"))
        result = summarize(document, _sha256(args.database))
    except (OSError, json.JSONDecodeError, InputError, RuntimeError) as error:
        print(f"surface oracle: {error}", file=sys.stderr)
        return 2
    json.dump(result, sys.stdout, indent=2, sort_keys=True)
    print()
    return 0 if result["acceptance"]["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())

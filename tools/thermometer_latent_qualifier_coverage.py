#!/usr/bin/env python3
"""Thermometer latent-heat qualifier coverage (po-009 round 4, 2026-11-11).

A thermometer reading in a vessel whose heat accounting skipped a latent
heat must say so ON the reading. The predicate is
`melting_range::unpaid_melts`: a solid above its sourced melting point or
the top of its sourced melting range with no enthalpy of fusion the bench
can pay. It drives both the step refusal (`unmodelled_melts`) and the
reading qualifier (`thermometer_qualifier`).

This tool checks, from source:

  readouts   every `Event::Measured` in bench.rs whose value is the vessel
             temperature (`temperature.to_celsius()`) consults
             `thermometer_qualifier` in the same arm
  ranges     every `MELTING_RANGES` row is classified
               paid       sourced lower bound + enthalpy of fusion: melted
                          and paid, reading unqualified
               qualified  otherwise: above its upper bound the reading
                          carries `measured.thermometer-latent-heat-unpaid`
  sharp      species with a sourced sharp melting point and no fusion
             enthalpy are qualified by the same predicate (not enumerated:
             the registry is generated; the predicate is shared)
  locale     the qualifier key exists in the DE and FR catalogues
  tests      every test named in TESTS exists

Writes docs/thermometer-latent-qualifier-coverage-20261111.{md,json};
`--check` fails when either is stale or a check fails.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BENCH = ROOT / "crates/kerotakis-core/src/bench.rs"
RANGES = ROOT / "crates/kerotakis-core/src/melting_range.rs"
I18N = ROOT / "crates/kerotakis-core/i18n"
TEST = ROOT / "crates/kerotakis-core/tests/thermometer_latent_qualifier.rs"
MD = ROOT / "docs/thermometer-latent-qualifier-coverage-20261111.md"
JS = ROOT / "docs/thermometer-latent-qualifier-coverage-20261111.json"
KEY = "measured.thermometer-latent-heat-unpaid"
TESTS = [
    "po009_blend_thermometer_reading_carries_the_unpaid_latent_heat_qualifier",
    "thermometer_qualifier_names_every_unpaid_polymer_and_renders_de_fr",
    "only_the_polymer_above_its_range_is_named",
    "step_refusal_and_reading_qualifier_share_one_predicate",
    "a_polymer_below_its_range_reads_unqualified",
    "a_paid_wax_melt_reads_unqualified",
    "heated_water_reads_unqualified",
    "a_melted_polymer_liquid_is_not_named",
]


def flatten(table, prefix=""):
    out = {}
    for k, v in table.items():
        key = f"{prefix}{k}"
        if isinstance(v, dict):
            out.update(flatten(v, key + "."))
        else:
            out[key] = v
    return out


def readouts():
    """Each `Instrument::X =>` arm in the Measure match that reports the
    vessel temperature, and whether it consults the qualifier."""
    src = BENCH.read_text(encoding="utf-8")
    start = src.index("Operator::Measure { vessel, instrument } =>")
    body = src[start:]
    arms = [(m.group(1), m.start()) for m in re.finditer(r"Instrument::(\w+) =>", body)]
    out = []
    for i, (name, pos) in enumerate(arms):
        end = arms[i + 1][1] if i + 1 < len(arms) else len(body)
        arm = body[pos:end]
        if "temperature.to_celsius()" in arm:
            out.append(dict(instrument=name, qualified="thermometer_qualifier" in arm))
    return out


def ranges():
    src = RANGES.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    start = src.index("pub const MELTING_RANGES")
    body = src[start:src.index("];", start)]
    rows = []
    for m in re.finditer(r"MeltingRange \{(.*?)\n    \}", body, re.S):
        b = m.group(1)
        lower = re.search(r"lower_k: ([^,]+),", b).group(1).strip()
        fusion = re.search(r"fusion_j_per_kg: ([^,]+),", b).group(1).strip()
        paid = lower.startswith("Some") and fusion.startswith("Some")
        rows.append(dict(
            species=re.search(r'species: "([^"]+)"', b).group(1),
            upper_k=float(re.search(r"upper_k: ([\d_.]+),", b).group(1).replace("_", "")),
            cls="paid" if paid else "qualified",
        ))
    shared = "unpaid_melts(vessel)" in src and src.count("unpaid_melts(vessel)") >= 2
    return rows, shared


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    reads = readouts()
    rows, shared = ranges()
    problems = []
    if not reads:
        problems.append("no temperature readout found in bench.rs")
    problems += [f"readout {r['instrument']} does not consult thermometer_qualifier"
                 for r in reads if not r["qualified"]]
    if not shared:
        problems.append("unmodelled_melts and thermometer_qualifier do not share unpaid_melts")
    for lang in ("de", "fr"):
        cat = flatten(tomllib.loads((I18N / f"{lang}.toml").read_text(encoding="utf-8")))
        if KEY not in cat:
            problems.append(f"{lang}: missing {KEY}")
    test_src = TEST.read_text(encoding="utf-8") if TEST.exists() else ""
    problems += [f"missing test {t}" for t in TESTS if f"fn {t}(" not in test_src]

    data = dict(
        generated_by="tools/thermometer_latent_qualifier_coverage.py",
        predicate="melting_range::unpaid_melts",
        qualifier_key=KEY,
        readouts=reads,
        ranges=rows,
        tests=TESTS,
    )
    md = [
        "# Thermometer latent-heat qualifier coverage (2026-11-11)", "",
        "Generated by `tools/thermometer_latent_qualifier_coverage.py --check`.", "",
        "**Rule.** A solid standing above its sourced melting point, or above the top of its sourced",
        "melting range, with no enthalpy of fusion the bench can pay, was warmed by sensible heat",
        "alone. A thermometer reading of that vessel carries the inline qualifier",
        f"`{KEY}` (EN/DE/FR) on the `measured` event itself (`note` / `note_reason`): the number",
        "overstates the warming and is not a prediction. One predicate,",
        "`melting_range::unpaid_melts`, drives both this qualifier and the step-level refusals",
        "`not-modeled.melting-range-latent-heat-unsourced` / `not-modeled.melting-not-modelled`,",
        "so they cannot disagree. Generic over every species with a sourced melting point or",
        "range; no polymer-specific branch.", "",
        "## Temperature readouts", "",
        "| instrument | consults qualifier |", "|---|---|",
    ]
    md += [f"| {r['instrument']} | {'yes' if r['qualified'] else 'NO'} |" for r in reads]
    md += ["", "## Melting-range rows", "",
           "| species | upper (K) | reading above the range |", "|---|---|---|"]
    md += [f"| {r['species']} | {r['upper_k']} | "
           f"{'unqualified (latent heat paid)' if r['cls'] == 'paid' else 'qualified'} |" for r in rows]
    md += ["", "## Why the polyolefins are qualified, not paid", "",
           "U. Gaur and B. Wunderlich, J. Phys. Chem. Ref. Data 10 (1981) 119, doi:10.1063/1.555636 (PE)",
           "and 10 (1981) 1051, doi:10.1063/1.555650 (iPP) give the enthalpy of fusion of a 100 %",
           "crystalline polymer. A real grade's latent heat is that value times its crystallinity,",
           "which this bench does not know, and the lower end of a grade's melting range is not",
           "sourced either. Paying the 100 % value would overstate the latent heat by the",
           "unknown amorphous fraction, so the reading is qualified instead.", "",
           "## Tests", ""]
    md += [f"- `crates/kerotakis-core/tests/thermometer_latent_qualifier.rs::{t}`" for t in TESTS]
    md.append("")
    text = "\n".join(md)
    js = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    if a.check:
        bad = bool(problems)
        for p in problems:
            print("problem:", p, file=sys.stderr)
        for path, want in ((MD, text), (JS, js)):
            if not path.exists() or path.read_text(encoding="utf-8") != want:
                print("stale:", path, file=sys.stderr)
                bad = True
        return 1 if bad else 0
    MD.write_text(text, encoding="utf-8")
    JS.write_text(js, encoding="utf-8")
    print(json.dumps({"readouts": reads, "ranges": len(rows), "problems": problems}))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Clock-reaction kinetics coverage: which rate constants are sourced (2026-11-11).

Reads every curated row in crates/kerotakis-core/src/kinetics.rs that takes part
in a clock reaction, extracts its pre-exponential and activation energy, computes
k(25 C), and checks the sourced rows against the primary values they cite:

  sourced     constant read from the cited primary (value and page stated)
  editorial   pre-exponential chosen to match an observable; not a measurement
  owned       a catalysed branch that stands down while another row owns its step
  instant     curated instantaneous reaction (no rate constant)

Writes docs/iodine-clock-kinetics-coverage-20261111.{md,json}; `--check` fails when
stale or when a sourced row no longer reproduces its cited 25 C value.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
KINETICS = ROOT / "crates/kerotakis-core/src/kinetics.rs"
OUT_MD = ROOT / "docs/iodine-clock-kinetics-coverage-20261111.md"
OUT_JSON = ROOT / "docs/iodine-clock-kinetics-coverage-20261111.json"
R = 8.314_462_618_153_24

LIEBHAFSKY = ("Liebhafsky & Mohammad, J. Am. Chem. Soc. 55 (1933) 3977-3986, "
              "doi:10.1021/ja01337a010; read in the archive.org scan of JACS vol. 55 "
              "(identifier dli.ernet.233065): Summary items 2 and 5, Table III")

ROWS = [
    {
        "id": "iodide-peroxide-clock",
        "term": "k1[H+][H2O2][I-] (acid term)",
        "class": "sourced",
        "k25_cited_si": 10.5 / 60.0,
        "cited": "k1 = 10.5 L2 mol-2 min-1 at 25 C; Q = 10,450 +/- 120 cal/mol (0-40 C)",
        "source": LIEBHAFSKY,
        "note": "minutes verified: Table II '50 HCl' run k'/(I-) = 6.66 = 0.69 + 10.5 x 0.567; "
                "A anchored on the 25 C value with the cited Q; Table III 0/15/40 C within +7 %",
    },
    {
        "id": "iodide-peroxide-clock-direct",
        "term": "k0[H2O2][I-] (acid-independent term)",
        "class": "sourced",
        "k25_cited_si": 0.69 / 60.0,
        "cited": "k0 = 0.69 L mol-1 min-1 at 25 C; Q = 13,400 +/- 105 cal/mol (0-50 C)",
        "source": LIEBHAFSKY + "; Q also quoted by Moelwyn-Hughes, The Kinetics of Reactions "
                  "in Solution, 2nd ed. (1947), ch. II",
        "note": "same net equation as the acid term: the source equates iodine formation with "
                "peroxide loss for both terms in acid; bounded by represented acid, so it cannot "
                "run in a neutral vessel. Table III 0/15/40 C within -10 %",
    },
    {
        "id": "peroxide-decomposition",
        "term": "I-/KI catalysed branch",
        "class": "owned",
        "k25_cited_si": None,
        "cited": "Ea(I-) = 56 +/- 3 kJ/mol (Sweeney et al., J. Chem. Educ. 2014, doi:10.1021/ed500116g)",
        "source": "CATALYST_PATHWAY_OWNERS in kinetics.rs",
        "note": "stands down while either clock row can run (represented acid present), so "
                "H2O2 + I- has one owner. The neutral catalysed pre-exponential stays editorial "
                "(0.05 M reference); it gives an extent constant about 15x Liebhafsky's k0 "
                "(oxygen-evolution series, Table III) - recorded, not changed in this lane",
    },
    {
        "id": "thiosulfate-acid",
        "term": "k[S2O3-2][H+]",
        "class": "editorial",
        "k25_cited_si": None,
        "cited": "Ea 51 kJ/mol teaching value; A fitted to the disappearing-cross time",
        "source": "existing row provenance",
        "note": "competes with the iodine sink in batch2:ki-011; unchanged",
    },
    {
        "id": "iodate-bisulfite-clock",
        "term": "k[IO3-][HSO3-]",
        "class": "editorial",
        "k25_cited_si": None,
        "cited": "Ea 48 kJ/mol textbook; A matched to the clock time",
        "source": "existing row provenance",
        "note": "Landolt clock; unchanged",
    },
]

NOT_MODELLED = [
    {
        "step": "HOI + H2O2 -> I- + O2 + H+ + H2O",
        "status": "not a separate row",
        "reason": "in acid the source's rate law shows the HOI going on to iodine "
                  "(d(I3-)/dt = -d(H2O2)/dt); in neutral vessels the catalysed decomposition row "
                  "carries it implicitly. A literature value of 29 M-1 s-1 (Shin et al., Water "
                  "Research 179 (2020) 115852) was seen only as a search-result quotation and is "
                  "not used",
    },
    {
        "step": "I2 + 2 S2O3-2 -> 2 I- + S4O6-2",
        "status": "instant",
        "reason": "curated instantaneous row (curated.rs); the clock delay is the time gross "
                  "iodine production takes to exceed the thiosulfate capacity",
    },
]


def extract(rows_src: str, rid: str):
    i = rows_src.find(f'id: "{rid}",')
    if i < 0:
        return None
    j = rows_src.find("KineticReaction {", i)
    block = rows_src[i: j if j > 0 else len(rows_src)]
    fwd = block[block.find("forward: RateExpression"):]
    a = re.search(r"pre_exponential:\s*([0-9eE._+-]+)", fwd)
    e = re.search(r"activation_energy:\s*([0-9_.]+)", fwd)
    if not (a and e):
        return None
    return float(a.group(1).replace("_", "")), float(e.group(1).replace("_", ""))


def build():
    src = KINETICS.read_text(encoding="utf-8")
    problems, out = [], []
    for row in ROWS:
        got = extract(src, row["id"])
        if got is None:
            problems.append(f"{row['id']}: row not found in kinetics.rs")
            continue
        a, ea = got
        k25 = a * math.exp(-ea / (R * 298.15))
        rec = dict(row, pre_exponential=a, activation_energy_j_mol=ea, k25_engine=float(f"{k25:.6g}"))
        if row["class"] == "sourced":
            dev = k25 / row["k25_cited_si"] - 1.0
            rec["k25_deviation"] = float(f"{dev:.3g}")
            if abs(dev) > 1e-3:
                problems.append(f"{row['id']}: k(25 C) {k25:.6g} differs from cited "
                                f"{row['k25_cited_si']:.6g}")
        out.append(rec)
    counts = {}
    for r in out:
        counts[r["class"]] = counts.get(r["class"], 0) + 1
    data = {"schema": "kerotakis/iodine-clock-kinetics-coverage-v1", "counts": counts,
            "rows": out, "not_modelled": NOT_MODELLED,
            "target": "batch2:ki-011 (hand-computed gross-iodine delay 93.9 s / 196.4 s, "
                      "tests/iodide_peroxide_two_term.rs)"}
    md = ["# Iodine-clock kinetics coverage (2026-11-11)", "",
          "Generated by `tools/iodine_clock_kinetics_coverage.py --check`.", "",
          "Law: rate = (k0 + k1[H+])[H2O2][I-] (Liebhafsky & Mohammad 1933). Time unit of the",
          "source is the minute; constants below are converted to seconds.", "",
          f"Counts: {', '.join(f'{k} {v}' for k, v in sorted(counts.items()))}.", "",
          "| row | term | class | A | Ea J/mol | k(25 C) engine | cited | source | note |",
          "|---|---|---|---|---|---|---|---|---|"]
    md += [f"| {r['id']} | {r['term']} | {r['class']} | {r['pre_exponential']:.5g} | "
           f"{r['activation_energy_j_mol']:.0f} | {r['k25_engine']:.4g} | {r['cited']} | "
           f"{r['source']} | {r['note']} |" for r in out]
    md += ["", "## Steps not carried as their own row", "",
           "| step | status | reason |", "|---|---|---|"]
    md += [f"| {n['step']} | {n['status']} | {n['reason']} |" for n in NOT_MODELLED]
    md += ["", "## batch2:ki-011 hand computation", "",
           "100 mL, 0.002 mol I-, 0.002 mol H2O2, pH 1.865 (h = 0.01365 M): k = 0.0115 + 0.175 h",
           "= 0.01389 L mol-1 s-1. Gross iodine reaches the 0.0001 mol thiosulfate capacity",
           "(5e-5 mol I2) at 93.9 s and the 0.0002 mol capacity at 196.4 s (iodide, peroxide and",
           "acid depleting). With iodide regenerated by thiosulfate: 91.5 s / 186.3 s; with the",
           "curated `thiosulfate-acid` decomposition also eating the sink: about 76 s / 135 s.", ""]
    return "\n".join(md) + "\n", json.dumps(data, indent=2, ensure_ascii=False) + "\n", problems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    md, js, problems = build()
    if problems:
        print("\n".join(problems), file=sys.stderr)
        return 1
    if a.check:
        stale = [p.name for p, t in ((OUT_MD, md), (OUT_JSON, js))
                 if not p.exists() or p.read_text(encoding="utf-8") != t]
        if stale:
            print("stale: " + ", ".join(stale) + "; rerun tools/iodine_clock_kinetics_coverage.py",
                  file=sys.stderr)
            return 1
        return 0
    OUT_MD.write_text(md, encoding="utf-8")
    OUT_JSON.write_text(js, encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())

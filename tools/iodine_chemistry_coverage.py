#!/usr/bin/env python3
"""Iodine chemistry coverage (2026-11-10).

Reads the constants of `crates/kerotakis-core/src/iodine_chemistry.rs` and
writes docs/iodine-chemistry-coverage-20261110.{md,json}; `--check` fails when
they are stale. The Rust test `iodine_chemistry_coverage_is_current` holds the
runtime constants to the JSON.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/kerotakis-core/src/iodine_chemistry.rs"
OUT_JS = ROOT / "docs/iodine-chemistry-coverage-20261110.json"
OUT_MD = ROOT / "docs/iodine-chemistry-coverage-20261110.md"

SOURCES = {
    "triiodide_k": "Palmer, Ramette & Mesmer, J. Solution Chem. 13 (1984) 673, doi:10.1007/BF00650374 "
                   "(molal constant, used as L/mol; literature range 698-770; Awtrey & Connick 1951)",
    "epsilon": "ACS Omega 9 (2024) 47350, doi:10.1021/acsomega.4c09084, PMC11603272 Table 3: "
               "I2 460 nm 722 (-0.19 %/C), I3- 460 nm 1022 (+0.48 %/C) at 25 C; replaces the "
               "unconfirmed 746/975 of Awtrey & Connick, J. Am. Chem. Soc. 73 (1951) 1842, "
               "doi:10.1021/ja01148a097, which still supplies I3- 353 nm 26400 L/mol/cm "
               "(later sources 24800-26400)",
    "e0_i2aq": "Bard, Parsons & Jordan, Standard Potentials in Aqueous Solution (IUPAC/Dekker 1985): "
               "I2(aq)/I- 0.621 V",
    "e0_fe": "halogen redox half table Fe+3/Fe+2 (llnl.dat, docs/halogen-redox-coverage-20261027)",
    "thiosulfate": "curated iodometric row (Vogel; Harris), crates/kerotakis-core/src/curated.rs",
}

MECHANISMS = [
    ("iodide salts dissolve to ions", "existing aqueous route (KI -> K+ + I-); an undissociated "
     "Na/K iodide counts as iodide in every iodine computation (halogen_redox::registry_forms)", "modelled"),
    ("I2 + I- <=> I3-", "computed split of the analytical I2(aq)/I- pools at K; not a separate "
     "inventory entry", "modelled"),
    ("iodine solubility in iodide", "capacity S0 (1 + K I / (1 + K S0)); no supersaturation flag "
     "below it; chloride gives no rise", "modelled"),
    ("I2 / I3- absorbance", "Beer-Lambert over the split with cited epsilon; I3- band width fitted "
     "to the two cited points", "modelled"),
    ("starch-iodine complex absorbance", "eye colour stays the editorial band of starch_iodine.rs; "
     "spectrophotometer refuses with not-modeled.starch-iodine-absorptivity (EN/DE/FR)", "typed refusal"),
    ("2 Fe3+ + 2 I- <=> 2 Fe2+ + I2", "solved to log K = 2 (E_Fe - E_I2aq) / 0.05916 with free I2 "
     "and I- from the triiodide split, both directions, 0-60 C, ideal activities; since 2026-11-11 "
     "read from every dissolved form (KI/NaI, FeCl3, FeSO4 ...) and generic over equilibrium-limited "
     "plain couples (docs/iodide-redox-coverage-20261111)", "modelled"),
    ("I2 + 2 S2O3^2- -> 2 I- + S4O6^2-", "existing curated limiting-reagent row, pinned by a "
     "closure test", "modelled"),
    ("AgI / AgBr solids", "registry solids lane (separate branch)", "out of scope here"),
]

TRIGGERS = [
    ("batch2:ec-008", "Fe3+/I- equilibrium"),
    ("new120:rx-004", "Fe3+/I- equilibrium; I2/I3- optics"),
    ("batch3-diverse:so-006", "triiodide capacity; I2/I3- optics"),
    ("batch2:mo-012", "I2/I3- optics"),
    ("batch2:qa-007", "I2/I3- optics (thiosulfate row already closed)"),
    ("batch2:ki-011", "I2/I3- optics; clock kinetics unchanged"),
    ("batch3-diverse:an-004", "starch complex: typed refusal; eye colour"),
    ("batch3-diverse:fo-003", "starch complex: typed refusal; eye colour"),
    ("batch3-diverse:en-001", "starch complex: typed refusal; eye colour"),
    ("new120:sp-011", "thiosulfate row; extraction solvent issue is a separate lane"),
    ("batch2:pq-004", "AgI solid: registry lane"),
    ("new120:pr-017", "AgI solid: registry lane"),
    ("batch3-diverse:an-003", "AgI/AgBr solids: registry lane"),
]


def const(src: str, name: str):
    m = re.search(rf"pub const {name}: [^=]+= ([^;]+);", src)
    if not m:
        raise SystemExit(f"{name} not found in {SRC}")
    return m.group(1).strip()


def num(text: str) -> float:
    return float(text.replace("_", ""))


def band(text: str):
    return [num(x) for x in text.strip("()").split(",")]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    src = SRC.read_text(encoding="utf-8")
    k = num(const(src, "TRIIODIDE_K"))
    e_i2 = num(const(src, "E0_I2AQ_IODIDE_V"))
    i2 = band(const(src, "I2_BAND"))
    i3 = band(const(src, "I3_BAND"))
    keys = re.findall(r'"([^"]+)"', const(src, "OPTICAL_KEYS"))
    e_fe = 0.7696
    log_k = 2 * (e_fe - e_i2) / 0.05916
    i3_at_460 = i3[1] * math.exp(-0.5 * ((460 - i3[0]) / i3[2]) ** 2)
    doc = dict(
        schema="kerotakis.iodine-chemistry-coverage.v1",
        constants=dict(triiodide_k=k, e0_i2aq_iodide_v=e_i2, e0_fe3_fe2_v=e_fe,
                       ferric_iodide_log_k=round(log_k, 4), i2_band=i2, i3_band=i3,
                       i3_epsilon_at_460_fitted=round(i3_at_460, 1)),
        optical_keys=keys,
        sources=SOURCES,
        mechanisms=[dict(mechanism=m, how=h, status=s) for m, h, s in MECHANISMS],
        triggers=[dict(id=i, covered_by=c) for i, c in TRIGGERS],
    )
    js = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"
    md = ["# Iodine chemistry coverage (2026-11-10)", "",
          "Generated by `tools/iodine_chemistry_coverage.py` from",
          "`crates/kerotakis-core/src/iodine_chemistry.rs`; `iodine_chemistry_coverage_is_current`",
          "fails when the runtime constants drift from the JSON.", "",
          "The ledger keeps dissolved iodine as one analytical pool `I2(aq)` and iodide as `I-`.",
          "Triiodide is a computed split of those pools at K, used for solubility, optics and",
          "the ferric/iodide equilibrium, so every existing row that reads `I2`/`I-` is unchanged.",
          "Activities are taken as molarities (no activity coefficients); constants are 25 °C.", "",
          "## Constants", "",
          f"- K(I2 + I- = I3-) = {k:g} L/mol",
          f"- E°(I2(aq)/I-) = {e_i2} V; E°(Fe3+/Fe2+) = {e_fe} V; log K(2 Fe3+ + 2 I- = 2 Fe2+ + I2) = {log_k:.4f}",
          f"- I2 band (centre nm, ε, σ nm) = {tuple(i2)}",
          f"- I3- band (centre nm, ε, σ nm) = {tuple(i3)}; fitted ε(460 nm) = {i3_at_460:.1f} (cited 1022)",
          f"- spectrophotometer-parameterised keys: {', '.join(keys)}", "",
          "## Sources", ""]
    md += [f"- {name}: {text}" for name, text in SOURCES.items()]
    md += ["", "## Mechanisms", "", "| mechanism | how | status |", "|---|---|---|"]
    md += [f"| {m} | {h} | {s} |" for m, h, s in MECHANISMS]
    md += ["", "## Trigger originals (evidence, not targets)", "", "| id | covered by |", "|---|---|"]
    md += [f"| {i} | {c} |" for i, c in TRIGGERS]
    md += ["", "## Known limits", "",
           "- No higher polyiodides (I5-) or iodine hydrolysis (HOI, IO-); dissolved iodine above the",
           "  triiodide capacity in very concentrated iodide is under-predicted.",
           "- Fe3+ hydrolysis and chloro-complexation are not subtracted from the analytical Fe3+",
           "  in the ferric/iodide quotient, which over-states the extent in non-acidified water.",
           "- No reaction heats are charged for the iodine rows.",
           "- The starch-iodine complex has no reviewed molar absorptivity: the instrument refuses."]
    mdtext = "\n".join(md) + "\n"
    if a.check:
        bad = [p for p, t in ((OUT_JS, js), (OUT_MD, mdtext)) if not p.exists() or p.read_text(encoding="utf-8") != t]
        if bad:
            print("stale:", *bad, file=sys.stderr)
            return 1
        return 0
    OUT_JS.write_text(js, encoding="utf-8")
    OUT_MD.write_text(mdtext, encoding="utf-8")
    print(json.dumps(doc["constants"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())

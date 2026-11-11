#!/usr/bin/env python3
"""S(IV) reductant coupling coverage (2026-11-11, KI-011).

Reads the constants, ions and oxidants from
crates/kerotakis-core/src/sulfur_iv.rs and derives the registry forms of every
S(IV) ion (the ion itself plus every neutral salt that is exactly the ion and
Na/K counter-ions, as `halogen_redox::registry_forms` does). Every form x
oxidant cell is classified:

  writer          oxidised to sulfate by `sulfur_iv` (I2 always; H2O2 inside
                  the measured acidity range when S(IV) lives <= MAX_INSTANT_S)
  halogen_writer  I2 + HSO3-/SO3-2 forms with no peroxide competing: the
                  admitted halogen_redox rows own them
  refused         typed: not-modeled.sulfite-peroxide-outside-measured-acid /
                  not-modeled.sulfite-peroxide-slow

Writes docs/sulfur-iv-coupling-coverage-20261111.{md,json}; `--check` fails
when either is stale. `sulfur_iv_coverage_is_current` pins the JSON to the
runtime.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/kerotakis-core/src/sulfur_iv.rs"
REG = ROOT / "data/registry/registry-source-v1.json"
MD = ROOT / "docs/sulfur-iv-coupling-coverage-20261111.md"
JS = ROOT / "docs/sulfur-iv-coupling-coverage-20261111.json"

CONSTANTS = ["K_I2_HSO3", "K_I3_HSO3", "K_I2_SO3", "K_I3_SO3", "MH_K1_M1_S1",
             "MH_K2_OVER_KM1_M1", "MH_DH_ACT_J_MOL", "MH_T_K", "MAX_INSTANT_S"]

SOURCES = [
    {"id": "yiin-margerum-1990",
     "citation": "B. S. Yiin and D. W. Margerum, Non-metal redox kinetics: reactions of iodine and "
                 "triiodide with sulfite and hydrogen sulfite and the hydrolysis of iodosulfate, "
                 "Inorg. Chem. 29 (1990) 1559-1564",
     "doi": "10.1021/ic00333a023",
     "values": "k(I3- + HSO3-) 1.5e7, k(I2 + HSO3-) 1.7e9, k(I3- + SO3^2-) 2.9e8, k(I2 + SO3^2-) 3.1e9 "
               "M^-1 s^-1, 25.0 C, mu 0.50 M",
     "read": "values as quoted from the paper's abstract (secondary abstract page, lookchem CAS "
             "126901-00-6); full text not readable here. DOI verified via Crossref."},
    {"id": "mcardle-hoffmann-1983",
     "citation": "J. V. McArdle and M. R. Hoffmann, Kinetics and mechanism of the oxidation of aquated "
                 "sulfur dioxide by hydrogen peroxide at low pH, J. Phys. Chem. 87 (1983) 5425-5429",
     "doi": "10.1021/j150644a024",
     "values": "k1 = (2.6 +- 0.5)e6 M^-1 s^-1, k2/k-1 = 16 +- 4 M^-1 at 15 C; dH1 = 37 +- 2 kJ/mol; "
               "pH 0.0-4.5",
     "read": "abstract, CaltechAUTHORS record 59779 (authors.library.caltech.edu/59779). DOI verified "
             "via Crossref."},
    {"id": "hoffmann-edwards-1975",
     "citation": "M. R. Hoffmann and J. O. Edwards, Kinetics of the oxidation of sulfite by hydrogen "
                 "peroxide in acidic solution, J. Phys. Chem. 79 (1975) 2096-2098",
     "doi": "10.1021/j100587a005",
     "values": "context only (earlier acid-solution law); no number used",
     "read": "DOI verified via Crossref."},
    {"id": "palmer-1984",
     "citation": "triiodide K = 698 L/mol (Palmer et al. 1984), reused from iodine_chemistry::TRIIODIDE_K",
     "doi": None, "values": "698 L/mol", "read": "already cited in iodine_chemistry.rs"},
]

LIMITATIONS = [
    "Both oxidations are treated as instantaneous on the bench clock; the peroxide path only when "
    "the S(IV) pseudo-first-order lifetime is <= MAX_INSTANT_S (a modelling bound, not a constant).",
    "The I2/I3- split used for the iodine constant is the split at the start of the coupling.",
    "Rates are written on HSO3-; the shared alpha1 fraction cancels in the competition. A direct "
    "SO2(aq) + I2 path and SO3^2- at higher pH are not separately priced.",
    "Yiin-Margerum constants are 25 C values (no temperature dependence applied); k2/k-1 is held "
    "at its 15 C value; k1 is moved with Eyring and dH1.",
    "Thiosulfate's iodine titration (curated row) runs before the S(IV) coupling inside one tail "
    "pass; their competition for iodine within a step is not resolved.",
    "SO2 booked as gas by the thiosulfate-acid law and still held by the vessel counts as S(IV) "
    "while the vessel holds liquid water; vented SO2 is gone.",
    "Released protons stay in the charge balance (halogen_redox convention); no reaction heat.",
]


def rust_consts(src: str) -> dict:
    out = {}
    for name in CONSTANTS:
        m = re.search(rf"pub const {name}: f64 = ([0-9_.eE+-]+);", src)
        if not m:
            raise SystemExit(f"constant {name} not found in {SRC}")
        out[name] = float(m.group(1).replace("_", ""))
    return out


def ions(src: str) -> list[str]:
    body = src[src.index("pub const IONS"):src.index("];", src.index("pub const IONS"))]
    return re.findall(r'key: "([^"]+)"', body)


def oxidants(src: str) -> list[str]:
    m = re.search(r"pub const OXIDANTS: &\[&str\] = &\[([^\]]*)\]", src)
    return re.findall(r'"([^"]+)"', m.group(1))


def parse(formula: str):
    m = re.match(r"^(.*?)([+-])(\d*)$", formula)
    body, charge = (formula, 0)
    if m and m.group(1):
        body = m.group(1)
        charge = (1 if m.group(2) == "+" else -1) * int(m.group(3) or 1)
    counts: dict[str, float] = {}
    for el, n in re.findall(r"([A-Z][a-z]?)(\d*)", body):
        counts[el] = counts.get(el, 0) + int(n or 1)
    return counts, charge


def registry_forms(ion: str, comps: dict[str, str]) -> list[str]:
    forms = [ion] if ion in comps else []
    ic, iq = parse(ion)
    if iq >= 0:
        return forms
    salts = []
    for key, formula in comps.items():
        if key == ion:
            continue
        fc, fq = parse(formula)
        if fq != 0:
            continue
        rest = dict(fc)
        ok = True
        for el, n in ic.items():
            if rest.pop(el, None) != n:
                ok = False
                break
        if not ok or not rest or any(el not in ("Na", "K") for el in rest):
            continue
        if sum(rest.values()) + iq != 0:
            continue
        salts.append(key)
    return forms + sorted(salts)


def build():
    src = SRC.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    reg = json.loads(REG.read_text(encoding="utf-8"))
    comps = {c["species_id"]: c["formula"] for c in reg["compositions"]}
    consts = rust_consts(src)
    oxs = oxidants(src)
    forms = []
    for ion in ions(src):
        for f in registry_forms(ion, comps):
            forms.append({"form": f, "ion": ion})
    cells = []
    for f in forms:
        for o in oxs:
            if o == "I2":
                status = "writer" if f["ion"] == "SO2" else "writer_with_peroxide_else_halogen_writer"
                refusal = None
            else:
                status = "writer_in_measured_acid_and_fast"
                refusal = ["not-modeled.sulfite-peroxide-outside-measured-acid",
                           "not-modeled.sulfite-peroxide-slow"]
            cells.append({"form": f["form"], "ion": f["ion"], "oxidant": o,
                          "status": status, "refusal_keys": refusal})
    doc = {"generated_by": "tools/sulfur_iv_coupling_coverage.py", "case": "batch2:ki-011",
           "constants": consts, "oxidants": oxs, "forms": forms, "cells": cells,
           "sources": SOURCES, "limitations": LIMITATIONS}
    md = ["# S(IV) reductant coupling coverage (2026-11-11)", "",
          "Generated by `tools/sulfur_iv_coupling_coverage.py --check` from",
          "`crates/kerotakis-core/src/sulfur_iv.rs` and the registry source document. KI-011 found",
          "the SO2 from thiosulfate's acid decomposition sitting unreacted beside iodine and hydrogen",
          "peroxide. Every S(IV) form below is now reducing capacity: iodine and peroxide oxidise it to",
          "sulfate, sharing it by their competing rate laws",
          "(`s0 = (i0 - i) + beta ln(i0/i)`, `beta = k_H h0 / k_I`).", "",
          "## Constants", "", "| name | value |", "|---|---|"]
    md += [f"| `{k}` | {v:g} |" for k, v in consts.items()]
    md += ["", "## Forms x oxidants", "", "| form | ion | oxidant | status | refusal keys |",
           "|---|---|---|---|---|"]
    for c in cells:
        md.append(f"| {c['form']} | {c['ion']} | {c['oxidant']} | {c['status']} | "
                  f"{', '.join('`%s`' % k for k in c['refusal_keys']) if c['refusal_keys'] else '—'} |")
    md += ["", "## Sources", ""]
    for s in SOURCES:
        doi = f", doi:{s['doi']}" if s["doi"] else ""
        md.append(f"- {s['citation']}{doi}. Values: {s['values']}. Read: {s['read']}")
    md += ["", "## Limitations", ""] + [f"- {x}" for x in LIMITATIONS]
    md += ["", "Refusals are typed EN/DE/FR (`crates/kerotakis-core/i18n/{de,fr}.toml`, `[not-modeled]`).",
           "The same change also emits a `reacted` event for every kinetic extent below",
           "`OBSERVABLE_MOLES` that is visible in the ledger (`clock::ledger_visible_extent`, one part per",
           "million), so peroxide or oxygen amounts never change without a named reaction."]
    return "\n".join(md) + "\n", json.dumps(doc, indent=2, ensure_ascii=False) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    md, js = build()
    if a.check:
        stale = [p for p, t in ((MD, md), (JS, js)) if not p.exists() or p.read_text(encoding="utf-8") != t]
        for p in stale:
            print("stale:", p, file=sys.stderr)
        return 1 if stale else 0
    MD.write_text(md, encoding="utf-8")
    JS.write_text(js, encoding="utf-8")
    print(json.dumps({"forms": len(json.loads(js)["forms"])}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

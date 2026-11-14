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

2026-11-11 (round 4, lane D): every S(IV) oxidant the bench names is a row of
`OXIDANT_TABLE` (reduction half, rate constants per S(IV) form, pricing); the
table is parsed here, each row's registry forms derived, and its status and
typed refusal key written beside its cited source.

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
             "MH_K2_OVER_KM1_M1", "MH_DH_ACT_J_MOL", "MH_T_K", "MAX_INSTANT_S",
             "HC_K_M2_S1", "HC_KEQ_M1", "HC_T_K", "HOBR_K_SO3", "OBR_K_SO3", "HOCL_K_SO3"]

PRICING_STATUS = {
    "competing": ("writer (closed-form iodine/peroxide competition)", None),
    "rated": ("writer when sole S(IV) oxidant and every present form has k and lives <= MAX_INSTANT_S",
              ["not-modeled.sulfite-oxidant-slow", "not-modeled.sulfite-oxidant-no-rate-for-form",
               "not-modeled.sulfite-oxidant-competition-unpriced"]),
    "stoichiometric": ("writer when sole S(IV) oxidant, instantaneous, no rate constant claimed "
                       "(typed note on every run), proton demand limited by represented acid",
                       ["not-modeled.sulfite-oxidant-no-rate-claimed",
                        "not-modeled.sulfite-oxidant-acid-limited",
                        "not-modeled.sulfite-oxidant-competition-unpriced"]),
    "halogen_writer": ("admitted halogen_redox row (instantaneous)", None),
    "product_not_in_registry": ("refused", ["not-modeled.sulfite-oxidant-product-not-in-registry"]),
    "no_reviewed_rate": ("refused", ["not-modeled.sulfite-autoxidation"]),
}

OXIDANT_SOURCES = {
    "I2": "Yiin & Margerum 1990 (doi:10.1021/ic00333a023), constants as printed in the Yiin 1990 Purdue "
          "dissertation abstract (AAI9018935)",
    "H2O2": "Hoffmann & Calvert 1985 Vol. II summary table (k 7.50e7 M^-2 s^-1, K 13 M^-1, 25 C); "
            "McArdle & Hoffmann 1983 (doi:10.1021/j150644a024) dH 37 kJ/mol and pH 0-4.5 range",
    "O3": "Hoffmann & Calvert 1985 Vol. II summary table: k0 2.40e4, k1 3.70e5, k2 1.50e9 M^-1 s^-1",
    "HBrO": "Troy & Margerum 1991 (doi:10.1021/ic00018a028), 5.0e9 M^-1 s^-1 as quoted by Liu et al. 2020 "
            "GRL (doi:10.1029/2019GL086465); a database page gives 2.3e9 (conflict, not resolved here)",
    "BrO-": "Troy & Margerum 1991 (doi:10.1021/ic00018a028), 1.0e8 M^-1 s^-1 as quoted in J. Phys. Chem. A "
            "113 (2009) 9988",
    "HClO": "Fogelman, Walker & Margerum 1989 (doi:10.1021/ic00305a002), HOCl + SO3^2- 7.6e8 M^-1 s^-1, "
            "25 C, mu 0.5 (abstract; re-quoted by Liu et al. 2020); halogen_redox FAST row",
    "ClO-": "Fogelman, Walker & Margerum 1989 (doi:10.1021/ic00305a002); halogen_redox FAST row, no k used",
    "Br2": "no rate constant found (searched Yiin/Margerum, Troy & Margerum); no rate claimed",
    "MnO4-": "no rate constant obtained (Ernst, Cyfert & Wilgocki 1992, doi:10.1002/kin.550241008, gives "
             "the basic-solution form only); no rate claimed; acid product Mn2+",
    "Cr2O7-2": "Haight et al. 1965 (doi:10.1021/ja01095a009) form only; no rate claimed; acid product Cr3+",
    "HCrO4-": "Haight et al. 1965 (doi:10.1021/ja01095a009) form only; no rate claimed; acid product Cr3+",
    "NO2-": "Oblath 1981, LBL-13026 PhD thesis, Table II: rate = k[NO2-][HSO3-][H+], k 4.0-4.5e3 "
            "M^-2 s^-1 at 22 C, mu 0.20; product hydroxylamine disulfonate (not in the registry)",
    "O2": "Hoffmann & Calvert 1985 and Martin & Hill 1987 (doi:10.1016/0004-6981(87)90361-1): metal-"
          "catalysed rates depend on catalyst, pH and ionic strength; no reviewed uncatalysed rate",
}

SOURCES = [
    {"id": "yiin-margerum-1990",
     "citation": "B. S. Yiin and D. W. Margerum, Non-metal redox kinetics: reactions of iodine and "
                 "triiodide with sulfite and hydrogen sulfite and the hydrolysis of iodosulfate, "
                 "Inorg. Chem. 29 (1990) 1559-1564",
     "doi": "10.1021/ic00333a023",
     "values": "k(I3- + HSO3-) 1.5e7, k(I2 + HSO3-) 1.7e9, k(I3- + SO3^2-) 2.9e8, k(I2 + SO3^2-) 3.1e9 "
               "M^-1 s^-1, 25.0 C, mu 0.50 M",
     "values_used_2026_11_11c": "1.4e7 (I3- + HSO3-), 1.8e9 (I2 + HSO3-), 2.8e8 (I3- + SO3^2-), "
                                "3.2e9 (I2 + SO3^2-) M^-1 s^-1, 25.0 C, mu 0.50 M, p[H+] 4.1-6.6",
     "read": "constants as printed in the abstract of B. S. Yiin, PhD dissertation, Purdue 1990 "
             "(AAI9018935, docs.lib.purdue.edu/dissertations/AAI9018935), the same work; they replace "
             "the earlier secondary-abstract reading (lookchem). DOI verified via Crossref."},
    {"id": "hoffmann-calvert-1985",
     "citation": "M. R. Hoffmann and J. G. Calvert, Chemical transformation modules for Eulerian acid "
                 "deposition models, Vol. II: The aqueous-phase chemistry, NCAR / US EPA (1985)",
     "doi": None,
     "values": "H2O2: k = 7.50E+07 M^-2 s^-1, K = 1.30E+01 M^-1; O3: k0 2.40E+04, k1 3.70E+05, "
               "k2 1.50E+09 M^-1 s^-1 (25 C)",
     "read": "archive.org OCR (_djvu.txt) of the report, rate-law summary table and ozone "
             "recommendation text; the textbook 7.45e7 is not what the table prints"},
    {"id": "liebhafsky-mohammad-1933",
     "citation": "H. A. Liebhafsky and A. Mohammad, J. Am. Chem. Soc. 55 (1933) 3977-3986",
     "doi": "10.1021/ja01337a010",
     "values": "iodine clock (same lane): k0 = 0.69 L mol^-1 min^-1 = 0.0115 M^-1 s^-1 replaces the "
               "editorial iodide/peroxide catalysis constant; lowest measured (H+) 0.00762 M (Table I, "
               "Curve II, p. 3980)",
     "read": "archive.org scan dli.ernet.233065"},
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
    "Other oxidants run only as the sole S(IV) oxidant in the vessel; with any second S(IV) oxidant "
    "(iodine, peroxide, a halogen row or another table oxidant) they are refused typed "
    "(competition unpriced) and the others proceed.",
    "Rated rows use 25 C constants without temperature dependence; stoichiometric rows claim no rate.",
    "Permanganate and dichromate are admitted only as far as represented acid covers their proton "
    "demand (acid products Mn2+ / Cr3+); neutral or basic products are refused typed.",
    "Dissolved O2 is refused only when booked aqueous; O2 in a vessel's gas phase is not dissolved O2.",
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


def resolve(token: str, consts: dict, src: str):
    token = token.strip()
    if token == "None":
        return None
    m = re.fullmatch(r"Some\((.+)\)", token)
    if not m:
        raise SystemExit(f"cannot read rate entry {token!r}")
    inner = m.group(1).strip()
    am = re.fullmatch(r"([A-Z0-9_]+)\[(\d)\]", inner)
    if am:
        arr = re.search(rf"pub const {am.group(1)}: \[f64; 3\] = \[([^\]]*)\];", src)
        return float(arr.group(1).split(",")[int(am.group(2))].replace("_", ""))
    if re.fullmatch(r"[A-Z0-9_]+", inner):
        return consts[inner]
    return float(inner.replace("_", ""))


def oxidant_table(src: str, consts: dict) -> list[dict]:
    body = src[src.index("pub const OXIDANT_TABLE"):]
    body = body[:body.index("\n];")]
    rows = []
    for m in re.finditer(r'ox\(\s*"([^"]+)",\s*&\[(.*?)\],\s*([0-9.]+),\s*([0-9.]+),\s*([0-9.]+),'
                         r'\s*\[(.*?)\],\s*Pricing::(\w+),\s*\)', body, re.S):
        key, prods, e, p, w, ks, pricing = m.groups()
        products = [(k, float(n)) for k, n in re.findall(r'\("([^"]+)",\s*([0-9.]+)\)', prods)]
        parts = [t for t in re.split(r",\s*(?![^()]*\))", ks) if t.strip()]
        k = [resolve(t, consts, src) for t in parts]
        snake = re.sub(r"(?<!^)([A-Z])", r"_\1", pricing).lower()
        rows.append({"oxidant": key, "products": products, "electrons": float(e),
                     "protons": float(p), "water": float(w), "k_so2_hso3_so3": k, "pricing": snake})
    if not rows:
        raise SystemExit("OXIDANT_TABLE not parsed")
    return rows


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
    table = []
    for row in oxidant_table(src, consts):
        status, keys = PRICING_STATUS[row["pricing"]]
        reg = registry_forms(row["oxidant"], comps)
        missing = [p for p, _ in row["products"] if p not in comps]
        if not reg:
            status = "no registry form (row kept; priced as below once the registry carries it): " + status
        elif missing:
            status = f"product {', '.join(missing)} not in registry: " + status
        table.append({**row, "registry_forms": reg, "status": status, "refusal_keys": keys,
                      "source": OXIDANT_SOURCES[row["oxidant"]]})
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
           "oxidant_table": table,
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
    md += ["", "## Every S(IV) oxidant (`OXIDANT_TABLE`)", "",
           "| oxidant | registry forms | k (SO2, HSO3-, SO3^2-) M^-1 s^-1 | pricing | status | refusal keys | source |",
           "|---|---|---|---|---|---|---|"]
    for r in table:
        ks = ", ".join("—" if v is None else f"{v:g}" for v in r["k_so2_hso3_so3"])
        md.append(f"| {r['oxidant']} | {', '.join(r['registry_forms']) or '—'} | {ks} | {r['pricing']} | "
                  f"{r['status']} | "
                  f"{', '.join('`%s`' % k for k in r['refusal_keys']) if r['refusal_keys'] else '—'} | "
                  f"{r['source']} |")
    covered = sum(1 for r in table if r["registry_forms"] and r["pricing"] in
                  ("competing", "rated", "stoichiometric", "halogen_writer"))
    refused = sum(1 for r in table if r["registry_forms"] and r["pricing"] not in
                  ("competing", "rated", "stoichiometric", "halogen_writer"))
    absent = sum(1 for r in table if not r["registry_forms"])
    md += ["", f"Registry oxidants priced: {covered}; refused typed: {refused}; table rows without a "
               f"registry form: {absent}."]
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

#!/usr/bin/env python3
"""Halogen/oxyhalogen redox coverage audit (2026-10-27).

ENUMERATES, it does not hand-list results: every reduction half-reaction
below is DERIVED by linear combination of reactions parsed out of the
vendored LLNL database (vendor/iphreeqc/database/llnl.dat, thermo.com.V8.R6
converted by USGS; US-government work, public domain like phreeqc.dat), or
taken from a cited standard potential where the vendored files define no
species for that oxidation state. Each derived half is element- and
charge-balance checked and its E deg is compared with an independent
textbook value inside an explicit tolerance.

Every oxidant half x every reductant half is then classified with the SAME
precedence as `crates/kerotakis-core/src/halogen_redox.rs`:

  self -> no_registry_form -> owned_elsewhere -> couple_rejected ->
  not_favourable ->
  equilibrium_limited -> product_not_determined -> rate_not_reviewed ->
  admitted

Usage:
  python3 tools/halogen_redox_coverage.py [--database-dir DIR] [--check]

Writes docs/halogen-redox-coverage-20261027.{md,json}. `--check` fails if
the checked-in files differ (the Rust test
`halogen_redox_coverage_is_current` checks the runtime against the JSON).
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NERNST_V = 0.05916  # (RT ln10)/F at 298.15 K, volts
FAVOURABLE_MARGIN_V = 0.30  # same as halogen_redox::COMPLETE_MARGIN_V
CROSS_CHECK_TOL_V = 0.03

# Reduction halves. `combo` lists (llnl reaction text, multiplier); the sum
# must reduce to exactly `half` (species -> coefficient, products positive,
# reactants negative, electrons as "e-"). `cited` gives the independent value.
HALVES = [
    # id, oxidised species, reduced products, n, H+ consumed, H2O produced
    dict(id="I2/I-", ox="I2", red={"I-": 2}, n=2, h=0, w=0, cited=0.5355,
         combo=None, compare_to="I3-/I-",
         source="cited: E(I2/I-) = +0.5355 V, Bard, Parsons & Jordan, Standard Potentials in Aqueous Solution (IUPAC, 1985); vendored files define no I(0) species, cross-checked against the vendored I3-/I- couple"),
    dict(id="I3-/I-", ox="I3-", red={"I-": 3}, n=2, h=0, w=0, cited=0.536,
         combo=[("3 I- + 2 H+ + 0.5 O2 = I3- + H2O", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat I3- and O2 reactions"),
    dict(id="IO3-/I-", ox="IO3-", red={"I-": 1}, n=6, h=6, w=3, cited=1.085,
         combo=[("1.5 O2 + I- = IO3-", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -1.5)],
         source="llnl.dat IO3- and O2 reactions"),
    dict(id="ClO-/Cl-", ox="ClO-", red={"Cl-": 1}, n=2, h=2, w=1, cited=1.715,
         combo=[("Cl- + 0.5 O2 = ClO-", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat ClO- and O2 reactions"),
    dict(id="HClO/Cl-", ox="HClO", red={"Cl-": 1}, n=2, h=1, w=1, cited=1.482,
         combo=[("H+ + ClO- = HClO", -1), ("Cl- + 0.5 O2 = ClO-", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat HClO, ClO- and O2 reactions"),
    dict(id="ClO3-/Cl-", ox="ClO3-", red={"Cl-": 1}, n=6, h=6, w=3, cited=1.451,
         combo=[("1.5 O2 + Cl- = ClO3-", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -1.5)],
         source="llnl.dat ClO3- and O2 reactions"),
    dict(id="Cl2/Cl-", ox="Cl2", red={"Cl-": 2}, n=2, h=0, w=0, cited=1.358,
         combo=None,
         source="cited: E(Cl2(g)/Cl-) = +1.358 V, Bard, Parsons & Jordan (IUPAC, 1985); vendored files define no Cl(0) species"),
    dict(id="Br2/Br-", ox="Br2", red={"Br-": 2}, n=2, h=0, w=0, cited=1.087,
         combo=[("2 H+ + 2 Br- + 0.5 O2 = Br2 + H2O", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat Br2 and O2 reactions"),
    dict(id="Br3-/Br-", ox="Br3-", red={"Br-": 3}, n=2, h=0, w=0, cited=1.05,
         combo=[("3 Br- + 2 H+ + 0.5 O2 = Br3- + H2O", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat Br3- and O2 reactions"),
    dict(id="BrO-/Br-", ox="BrO-", red={"Br-": 1}, n=2, h=2, w=1, cited=1.57,
         combo=[("Br- + 0.5 O2 = BrO-", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat BrO- and O2 reactions"),
    dict(id="BrO3-/Br-", ox="BrO3-", red={"Br-": 1}, n=6, h=6, w=3, cited=1.423,
         combo=[("1.5 O2 + Br- = BrO3-", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -1.5)],
         source="llnl.dat BrO3- and O2 reactions"),
    dict(id="Fe+3/Fe+2", ox="Fe+3", red={"Fe+2": 1}, n=1, h=0, w=0, cited=0.771,
         combo=[("H+ + Fe+2 + 0.25 O2 = Fe+3 + 0.5 H2O", -1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.25)],
         source="llnl.dat Fe+3 and O2 reactions"),
    dict(id="S4O6-2/S2O3-2", ox="S4O6-2", red={"S2O3-2": 2}, n=2, h=0, w=0, cited=0.08,
         combo=[("4 SO3-2 + 12 H+ + 6 e- = S4O6-2 + 6 H2O", -1),
                ("2 H+ + 2 SO3-2 = S2O3-2 + O2 + H2O", 2),
                ("2 H2O = O2 + 4 H+ + 4 e-", -2)],
         source="llnl.dat S4O6-2 (electron form), S2O3-2 and O2 reactions"),
    dict(id="SO4-2/H2SO3", ox="SO4-2", red={"H2SO3": 1}, n=2, h=4, w=1, cited=0.172,
         combo=[("SO4-2 = SO3-2 + 0.5 O2", 1), ("2 H+ + SO3-2 = H2SO3", 1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat SO3-2, H2SO3 and O2 reactions; cited E(SO4-2/H2SO3) = +0.172 V (Bard, Parsons & Jordan 1985; Harris appendix)"),
    dict(id="SO4-2/SO3-2", ox="SO4-2", red={"SO3-2": 1}, n=2, h=2, w=1, cited=None, check_via="SO4-2/H2SO3",
         combo=[("SO4-2 = SO3-2 + 0.5 O2", 1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat SO3-2 and O2 reactions; cross-checked through SO4-2/H2SO3, which shares the SO3-2 line"),
    dict(id="SO4-2/HSO3-", ox="SO4-2", red={"HSO3-": 1}, n=2, h=3, w=1, cited=None, check_via="SO4-2/H2SO3",
         combo=[("SO4-2 = SO3-2 + 0.5 O2", 1), ("SO3-2 + H+ = HSO3-", 1), ("2 H2O = O2 + 4 H+ + 4 e-", -0.5)],
         source="llnl.dat SO3-2, HSO3- and O2 reactions; cross-checked through SO4-2/H2SO3, which shares the SO3-2 line"),
]

# Oxidant halves whose OXIDISED member is an oxidant in this audit, and
# reductant halves whose REDUCED member is a reductant in it.
OXIDANTS = ["I2/I-", "I3-/I-", "IO3-/I-", "ClO-/Cl-", "HClO/Cl-", "ClO3-/Cl-",
            "Cl2/Cl-", "Br2/Br-", "Br3-/Br-", "BrO-/Br-", "BrO3-/Br-", "Fe+3/Fe+2"]
REDUCTANTS = ["S4O6-2/S2O3-2", "SO4-2/SO3-2", "SO4-2/HSO3-", "Fe+3/Fe+2", "I2/I-"]
# Reductants named by the lane with no reviewed couple in any vendored file.
UNCOUPLED_REDUCTANTS = {"ascorbic_acid": "no vendored or cited dehydroascorbate couple; the curated `C6H8O6 + I2` row owns the iodine case"}
# Oxidised product of a reductant half that thermodynamics alone does not
# select: thiosulfate goes to tetrathionate ONLY with iodine (iodometry);
# stronger oxidants take it on toward sulfate.
ONLY_WITH = {"S4O6-2/S2O3-2": ["I2/I-", "I3-/I-"]}
# Pairs that are fast and quantitative on the bench time scale, with the
# source that says so. Thermodynamics never admits a pair on its own.
FAST = {
    ("I2/I-", "S4O6-2/S2O3-2"): "iodometric titration (Vogel's Quantitative Chemical Analysis, 6th ed., iodometry; Harris, Quantitative Chemical Analysis, iodometry)",
    ("I3-/I-", "S4O6-2/S2O3-2"): "iodometric titration (Vogel; Harris)",
    ("I2/I-", "SO4-2/SO3-2"): "iodimetric determination of sulfite (Harris, Quantitative Chemical Analysis, iodimetry table; Vogel)",
    ("I2/I-", "SO4-2/HSO3-"): "iodimetric determination of sulfite/bisulfite (Harris; Vogel)",
    ("I3-/I-", "SO4-2/SO3-2"): "iodimetry (Harris)",
    ("I3-/I-", "SO4-2/HSO3-"): "iodimetry (Harris)",
    ("ClO-/Cl-", "I2/I-"): "iodometric determination of available chlorine in acidified iodide (Vogel, hypochlorite)",
    ("HClO/Cl-", "I2/I-"): "iodometric determination of available chlorine in acidified iodide (Vogel, hypochlorite)",
}
CATIONS = {"Na": 1, "K": 1}

TOKEN = re.compile(r"([A-Z][a-z]?)(\d*\.?\d*)|(\()|(\))(\d*)")


def parse_formula(text: str):
    text = text.replace("(aq)", "")
    m = re.match(r"^(.*?)([+-]\d*)?$", text)
    body, ch = m.group(1), m.group(2)
    charge = 0
    if ch:
        charge = int(ch[1:] or 1) * (1 if ch[0] == "+" else -1)
    stack = [{}]
    i = 0
    while i < len(body):
        c = body[i]
        if c == "(":
            stack.append({}); i += 1; continue
        if c == ")":
            j = i + 1
            while j < len(body) and body[j].isdigit():
                j += 1
            mult = int(body[i + 1:j] or 1)
            top = stack.pop()
            for k, v in top.items():
                stack[-1][k] = stack[-1].get(k, 0) + v * mult
            i = j; continue
        m2 = re.match(r"([A-Z][a-z]?)(\d*)", body[i:])
        if not m2:
            return None
        el, n = m2.group(1), int(m2.group(2) or 1)
        stack[-1][el] = stack[-1].get(el, 0) + n
        i += len(m2.group(0))
    return stack[0], charge


def side(text):
    out = {}
    for term in text.split(" + "):
        term = term.strip()
        m = re.match(r"^(\d*\.?\d+)\s+(\S+)$", term)
        coef, sp = (Fraction(m.group(1)), m.group(2)) if m else (Fraction(1), term)
        out[sp] = out.get(sp, 0) + coef
    return out


def load_llnl(path: Path):
    reactions = {}
    lines = path.read_text(encoding="latin-1").splitlines()
    for i, raw in enumerate(lines):
        line = raw.split("#")[0].strip()
        if "=" not in line or line.startswith("-"):
            continue
        norm = re.sub(r"\s+", " ", line)
        for j in range(i + 1, min(i + 8, len(lines))):
            t = lines[j].split("#")[0].strip()
            if t.startswith("log_k"):
                reactions.setdefault(norm, (float(t.split()[1]), i + 1))
                break
    return reactions


def balance(terms):
    el, ch = {}, Fraction(0)
    for sp, c in terms.items():
        if sp == "e-":
            ch -= c; continue
        f = parse_formula("H2O" if sp == "water" else sp)
        for k, v in f[0].items():
            el[k] = el.get(k, 0) + c * v
        ch += c * f[1]
    return all(v == 0 for v in el.values()) and ch == 0


def derive(half, llnl):
    target = {half["ox"]: Fraction(-1), "e-": Fraction(-half["n"]), "H+": Fraction(-half["h"]), "H2O": Fraction(half["w"])}
    for sp, c in half["red"].items():
        target[sp] = target.get(sp, 0) + c
    target = {k: v for k, v in target.items() if v != 0}
    assert balance(target), f"{half['id']} target half is unbalanced"
    if half["combo"] is None:
        return half["cited"], [], True
    total, logk, lines = {}, 0.0, []
    for text, mult in half["combo"]:
        key = re.sub(r"\s+", " ", text)
        if key not in llnl:
            raise SystemExit(f"llnl.dat lacks reaction {text!r}")
        k, line = llnl[key]
        lines.append(line)
        lhs, rhs = key.split(" = ")
        for sp, c in side(lhs).items():
            total[sp] = total.get(sp, 0) - c * Fraction(mult)
        for sp, c in side(rhs).items():
            total[sp] = total.get(sp, 0) + c * Fraction(mult)
        logk += k * mult
    total = {k: v for k, v in total.items() if v != 0}
    if total != target:
        raise SystemExit(f"{half['id']}: combination {total} != target {target}")
    e0 = NERNST_V * logk / half["n"]
    if half["cited"] is None:
        return e0, lines, None
    return e0, lines, abs(e0 - half["cited"]) <= CROSS_CHECK_TOL_V


def registry_forms(registry, ion):
    """Registry keys that deliver `ion` in solution: the ion itself, and
    salts whose formula minus the ion leaves only Na/K balancing its charge."""
    comps = {c["species_id"]: c["formula"] for c in registry["compositions"]}
    forms = []
    if ion in comps:
        forms.append(ion)
    ip = parse_formula(ion)
    for key, formula in sorted(comps.items()):
        if key == ion:
            continue
        p = parse_formula(formula)
        if p is None or p[1] != 0 or ip[1] >= 0:
            continue
        rest = dict(p[0])
        ok = True
        for el, n in ip[0].items():
            if rest.get(el, 0) != n:
                ok = False; break
            del rest[el]
        if not ok or not rest or any(el not in CATIONS for el in rest):
            continue
        if sum(n * CATIONS[el] for el, n in rest.items()) != -ip[1]:
            continue
        forms.append(key)
    return forms


def owned_pairs(root: Path):
    """Reactant key sets of every curated row and kinetic law."""
    sets = []
    cur = (root / "crates/kerotakis-core/src/curated.rs").read_text(encoding="utf-8")
    cur = cur.split("pub const REACTIONS")[1].split("pub const ORG_REACTIONS")[0]
    for m in re.finditer(r"reactants:\s*&\[(.*?)\]", cur, re.S):
        sets.append(set(re.findall(r'\("([^"]+)",', m.group(1))))
    kin = (root / "crates/kerotakis-core/src/kinetics.rs").read_text(encoding="utf-8")
    for m in re.finditer(r"stoichiometry:\s*&\[(.*?)\n        \],", kin, re.S):
        sets.append(set(re.findall(r'species:\s*"([^"]+)",\s*coefficient:\s*-', m.group(1))))
    return sets


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--database-dir", default=str(ROOT / "vendor/iphreeqc/database"))
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    llnl = load_llnl(Path(a.database_dir) / "llnl.dat")
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text(encoding="utf-8"))
    owned = owned_pairs(ROOT)

    halves = {}
    for h in sorted(HALVES, key=lambda h: bool(h.get("check_via") or h.get("compare_to"))):
        e0, lines, agrees = derive(h, llnl)
        if h.get("check_via"):
            agrees = halves[h["check_via"]]["cross_check"]
        if h.get("compare_to"):
            agrees = abs(e0 - halves[h["compare_to"]]["e0_v"]) <= CROSS_CHECK_TOL_V
        halves[h["id"]] = dict(cross_check=agrees, id=h["id"], oxidised=h["ox"], reduced=h["red"], electrons=h["n"],
                               protons=h["h"], water=h["w"], e0_v=round(e0, 4), check_via=h.get("check_via") or h.get("compare_to"),
                               llnl_lines=lines, cited_v=h["cited"], source=h["source"])

    halves = {h["id"]: halves[h["id"]] for h in HALVES}

    def forms_of(sp):
        return registry_forms(registry, sp)

    rows = []
    for o in OXIDANTS:
        for r in REDUCTANTS + sorted(UNCOUPLED_REDUCTANTS):
            ho = halves[o]
            if r in UNCOUPLED_REDUCTANTS:
                ox_forms = forms_of(ho["oxidised"])
                rows.append(dict(oxidant=o, reductant=r, delta_e_v=None,
                                 status="no_registry_form" if not ox_forms else "no_reviewed_couple",
                                 oxidant_forms=ox_forms, reductant_forms=[r], note=UNCOUPLED_REDUCTANTS[r]))
                continue
            hr = halves[r]
            red_sp = next(iter(hr["reduced"]))
            ox_forms, red_forms = forms_of(ho["oxidised"]), forms_of(red_sp)
            de = round(ho["e0_v"] - hr["e0_v"], 4)
            if o == r:
                status = "self"
            elif not ox_forms or not red_forms:
                status = "no_registry_form"
            elif any(s & set(ox_forms) and s & set(red_forms) for s in owned):
                status = "owned_elsewhere"
            elif not (ho["cross_check"] and hr["cross_check"]):
                status = "couple_rejected"
            elif de <= 0:
                status = "not_favourable"
            elif de < FAVOURABLE_MARGIN_V:
                status = "equilibrium_limited"
            elif r in ONLY_WITH and o not in ONLY_WITH[r]:
                status = "product_not_determined"
            elif (o, r) not in FAST:
                status = "rate_not_reviewed"
            else:
                status = "admitted"
            rows.append(dict(oxidant=o, reductant=r, delta_e_v=de, status=status,
                             oxidant_forms=ox_forms, reductant_forms=red_forms,
                             note=FAST.get((o, r), "")))

    doc = dict(schema="kerotakis.halogen-redox-coverage.v1", nernst_v=NERNST_V,
               complete_margin_v=FAVOURABLE_MARGIN_V, cross_check_tolerance_v=CROSS_CHECK_TOL_V,
               halves=list(halves.values()), pairs=rows)
    counts = {}
    for row in rows:
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    doc["counts"] = dict(sorted(counts.items()))
    js = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"

    md = ["# Halogen / oxyhalogen redox coverage (2026-10-27)", "",
          "Generated by `tools/halogen_redox_coverage.py` from `vendor/iphreeqc/database/llnl.dat`, the",
          "registry source document and the curated/kinetic reactant tables. Do not edit by hand;",
          "`halogen_redox_coverage_is_current` fails when the runtime and this table disagree.", "",
          "## Couples (reduction, 25 °C, pH 0)", "",
          "| couple | n | H⁺ | E° derived (V) | cited (V) | cross-check | llnl.dat lines | source |",
          "|---|---|---|---|---|---|---|---|"]
    for h in halves.values():
        md.append(f"| {h['id']} | {h['electrons']} | {h['protons']} | {h['e0_v']:+.4f} | {'via ' + h['check_via'] if h.get('check_via') else ('{:+.3f}'.format(h['cited_v']))} | {'pass' if h['cross_check'] else 'FAIL'} | "
                  f"{', '.join(map(str, h['llnl_lines'])) or '—'} | {h['source']} |")
    md += ["", f"Counts: {', '.join(f'{k} {v}' for k, v in doc['counts'].items())}.", "",
           "## Oxidant × reductant", "",
           "| oxidant | reductant | ΔE° (V) | status | registry oxidant forms | registry reductant forms | note |",
           "|---|---|---|---|---|---|---|"]
    for row in rows:
        de = "—" if row["delta_e_v"] is None else f"{row['delta_e_v']:+.3f}"
        md.append(f"| {row['oxidant']} | {row['reductant']} | {de} | {row['status']} | "
                  f"{', '.join(row['oxidant_forms']) or '—'} | {', '.join(row['reductant_forms']) or '—'} | {row['note']} |")
    md += ["", "Cited values are the commonly tabulated IUPAC (Bard, Parsons & Jordan 1985) / Harris-appendix",
           "potentials used ONLY as independent cross-checks of the derived values (tolerance",
           f"{CROSS_CHECK_TOL_V} V), except I2/I- and Cl2/Cl-, which have no vendored species; their",
           "edition-level provenance is pending reviewer confirmation. A FAIL keeps every pair on",
           "that couple out of the runtime (`couple_rejected`). S4O6-2/S2O3-2 fails because the",
           "llnl.dat S4O6-2 entry is written in electron form while its log_k (-38.3859) matches",
           "the O2 form in its own comment line - a convention mismatch, so it is not used.", "",
           "Status meanings: `admitted` — applied by `halogen_redox` to completion on aqueous inventory;",
           "`owned_elsewhere` — a curated row or kinetic law already owns a form of both members;",
           "`couple_rejected` — a member couple failed its cross-check against the cited value;",
           "`not_favourable` — ΔE° ≤ 0; `equilibrium_limited` — 0 < ΔE° < margin, would need an",
           "equilibrium solve, not a limiting-reagent writer; `product_not_determined` — the oxidised",
           "product of the reductant depends on the oxidant beyond thermodynamics; `rate_not_reviewed` —",
           "favourable but no reviewed source that it is fast on the bench; `no_registry_form` — the",
           "registry holds neither the ion nor a Na/K salt of it; `no_reviewed_couple` — no vendored or",
           "cited couple. Every non-admitted pair leaves inventory untouched.", ""]
    mdtext = "\n".join(md)

    out_js = ROOT / "docs/halogen-redox-coverage-20261027.json"
    out_md = ROOT / "docs/halogen-redox-coverage-20261027.md"
    if a.check:
        bad = [p for p, t in ((out_js, js), (out_md, mdtext)) if not p.exists() or p.read_text(encoding="utf-8") != t]
        if bad:
            print("stale:", *bad, file=sys.stderr); return 1
        return 0
    out_js.write_text(js, encoding="utf-8")
    out_md.write_text(mdtext, encoding="utf-8")
    print(json.dumps(doc["counts"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())

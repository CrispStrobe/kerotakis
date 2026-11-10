#!/usr/bin/env python3
"""Registry-solid coverage of the routed PHREEQC databases (2026-11-10).

Every non-gas PHASES entry of the three routed databases (wateq4f.dat,
minteq.v4.dat, pitzer.dat) is matched to the registry by the same rule
`kerotakis_phreeqc::derived::registry_solid_matching` applies: the phase's
dissolution-equation formula (first left-hand term, hydrate waters split off)
must have exactly the element counts AND the hydrate count of a registry
species whose standard phase is solid. A match is what lets the aqueous
solver pose the phase and the readback book a precipitate as that species;
an unmatched phase can be supersaturated and still precipitate nothing.

Each phase is classified:

  registry-solid      matched; the precipitate is booked as the named species
  declined            a common laboratory precipitate deliberately NOT added
                      yet, with the reason (see DECLINED below)
  bookable-unclaimed  every element has a registry ion to book against, but no
                      registry solid; mostly natural-water mineralogy
  unbookable          at least one element has no charged registry species,
                      so even a registry solid could not be fed or read back

Writes docs/registry-solid-coverage-20261110.{md,json}; `--check` fails when
they are stale. The Rust test `registry_solid_coverage_is_current`
(crates/kerotakis-phreeqc/tests/mineral_crosswalk.rs) holds the runtime
crosswalk to the JSON's registry-solid rows.

The vendored databases live in the `vendor/iphreeqc` submodule; set
KERO_PHREEQC_DATABASE_DIR to read them from another checkout of the same
submodule commit.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DATABASES = [("wateq4f", "wateq4f.dat"), ("minteq.v4", "minteq.v4.dat"), ("pitzer", "pitzer.dat")]
OUT_JS = ROOT / "docs/registry-solid-coverage-20261110.json"
OUT_MD = ROOT / "docs/registry-solid-coverage-20261110.md"

# Common laboratory precipitates the databases define that this lane did NOT
# add, keyed by the database formula. Each reason names what would have to be
# settled first; none is a claim that the solid is unimportant.
DECLINED = {
    "PbCl2": "would precipitate in `qualitative.rs::lead_chloride_stays_dissolved_when_dilute` "
             "(0.1 mol/kgw Pb, 0.3 mol/kgw Cl) unless chloride complexation keeps it under; "
             "needs an engine run before the verdict can be pinned either way",
    "PbBr2": "sparingly soluble (about 1 g/100 mL) rather than a bench precipitate; same "
             "engine-run requirement as PbCl2",
    "Ag2SO4": "about 0.8 g/100 mL soluble; only a precipitate from concentrated mixtures, and "
              "an unverified addition could change existing silver/sulfate vessels",
    "Ag2O": "no sourced density on PubChem; the brown AgNO3 + NaOH precipitate needs a "
            "citable crystal density before it can ship",
    "SrCO3": "strontianite is already owned by the aragonite-strontianite SOLID_SOLUTIONS "
             "lane (`solid_solution.rs`); a pure phase would compete with it",
    "SrSO4": "strontium sulfate joins with the strontium solid-solution lane, not here",
    "MgCO3": "magnesite does not nucleate at bench temperature; needs a kinetic floor "
             "(as hydroxylapatite has) before it may be posed",
    "CaMg(CO3)2": "dolomite: same kinetic objection as magnesite",
    "FeCO3": "siderite competes with the iron(II) hydroxide verdicts; needs an engine run",
    "MnCO3": "rhodochrosite competes with manganese hydroxide/oxide routes; needs an engine run",
    "ZnCO3": "a bench Zn2+ + carbonate mix gives a basic carbonate, not smithsonite; needs review",
    "Cu2(OH)2CO3": "malachite: the real CuSO4 + Na2CO3 product, but it competes with Cu(OH)2 "
                   "and the copper hydroxy-sulfates; needs an engine run",
    "Pb(OH)2": "competes with every lead-salt-plus-base verdict; needs an engine run",
    "Mn(OH)2": "pyrochroite: Mn(II) redox pinning is not reviewed for a new hydroxide",
    "CaHPO4:2H2O": "brushite competes with the calcium phosphate milk lane (octacalcium "
                   "phosphate); needs the milk replay before it may be posed",
}


def groups(s: str):
    stack = [{}]
    i = 0
    while i < len(s):
        c = s[i]
        if c in "([":
            stack.append({})
            i += 1
        elif c in ")]":
            i += 1
            m = re.match(r"\d*\.?\d*", s[i:]).group()
            i += len(m)
            n = float(m) if m else 1.0
            g = stack.pop()
            for k, v in g.items():
                stack[-1][k] = stack[-1].get(k, 0.0) + v * n
        else:
            m = re.match(r"([A-Z][a-z]?)(\d*\.?\d*)", s[i:])
            if not m:
                return None
            n = float(m.group(2)) if m.group(2) else 1.0
            stack[-1][m.group(1)] = stack[-1].get(m.group(1), 0.0) + n
            i += len(m.group())
    return stack[0] if len(stack) == 1 else None


def strip_charge(s: str) -> str:
    m = re.match(r"^(.*?)([+-]+\d*)$", s.strip())
    return m.group(1) if m else s.strip()


def split_hydrate(f: str):
    for sep in (":", "·"):
        if sep in f:
            base, hyd = f.split(sep, 1)
            if hyd.endswith("H2O"):
                d = re.match(r"\d*", hyd).group()
                return base, float(d) if d else 1.0
    return f, 0.0


def composition(formula: str):
    base, waters = split_hydrate(formula)
    return groups(strip_charge(base)), waters


def phases(path: Path):
    """Non-gas PHASES entries as {name: formula}, first definition wins."""
    out = {}
    section = None
    pending = None
    for raw in path.read_text(encoding="latin-1").splitlines():
        line = raw.split("#")[0]
        if not line.strip():
            continue
        first = line.split()[0]
        if line[0] not in " \t" and len(first) > 3 and all(c.isupper() or c == "_" for c in first):
            section = first
            pending = None
            continue
        if section != "PHASES":
            continue
        if line[0] not in " \t":
            pending = first
        elif "=" in line and pending:
            formula = line.split("=")[0].split(" + ")[0].strip()
            if not pending.endswith("(g)") and composition(formula)[0] is not None:
                out.setdefault(pending, formula)
            pending = None
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    db_dir = Path(os.environ.get("KERO_PHREEQC_DATABASE_DIR", ROOT / "vendor/iphreeqc/database"))
    reg = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text(encoding="utf-8"))
    phase_of = {r["species_id"]: r["phase"] for r in reg["phase_thermodynamics"]
                if r["property"] == "molar_mass"}
    charge = {c["species_id"]: c["net_charge"]["quantity"]["value"] if "quantity" in c["net_charge"]
              else c["net_charge"]["value"] for c in reg["compositions"]}
    solids = []
    bookable = {"H", "O"}
    for c in reg["compositions"]:
        comp, waters = composition(c["formula"].replace("·", ":"))
        if phase_of.get(c["species_id"]) == "solid" and comp is not None:
            solids.append((c["species_id"], comp, waters))
        if charge.get(c["species_id"]) and comp is not None:
            bookable |= set(comp)

    by_formula = {}
    for tag, file in DATABASES:
        for name, formula in phases(db_dir / file).items():
            row = by_formula.setdefault(formula, {"formula": formula, "phases": []})
            row["phases"].append({"database": tag, "phase": name})
    rows = []
    for formula, row in by_formula.items():
        comp, waters = composition(formula)
        match = next((k for k, g, w in solids if g == comp and w == waters), None)
        declined = DECLINED.get(formula)
        if match:
            cls = "registry-solid"
        elif declined:
            cls = "declined"
        elif set(comp) <= bookable:
            cls = "bookable-unclaimed"
        else:
            cls = "unbookable"
        rows.append(dict(formula=formula, registry_species=match, class_=cls,
                         reason=declined if cls == "declined" else None,
                         phases=sorted(row["phases"], key=lambda p: (p["database"], p["phase"]))))
    order = {"registry-solid": 0, "declined": 1, "bookable-unclaimed": 2, "unbookable": 3}
    rows.sort(key=lambda r: (order[r["class_"]], r["registry_species"] or "", r["formula"]))
    for r in rows:
        r["class"] = r.pop("class_")
    counts = {}
    for r in rows:
        counts[r["class"]] = counts.get(r["class"], 0) + 1
    missing_declined = sorted(set(DECLINED) - {r["formula"] for r in rows})
    if missing_declined:
        print("DECLINED names formulas no database defines:", missing_declined, file=sys.stderr)
        return 2
    doc = dict(schema="kerotakis.registry-solid-coverage.v1",
               databases=[t for t, _ in DATABASES],
               counts=dict(sorted(counts.items())),
               registry_solids_matched=sorted({r["registry_species"] for r in rows if r["registry_species"]}),
               formulas=rows)
    js = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"

    def names(r):
        return ", ".join(f"{p['database']}:`{p['phase']}`" for p in r["phases"])

    md = ["# Registry-solid coverage of the routed PHREEQC databases (2026-11-10)", "",
          "Generated by `tools/registry_solid_coverage.py` from the vendored `wateq4f.dat`,",
          "`minteq.v4.dat` and `pitzer.dat` and `data/registry/registry-source-v1.json`.",
          "`--check` fails when this file is stale, and `registry_solid_coverage_is_current`",
          "(kerotakis-phreeqc `tests/mineral_crosswalk.rs`) holds the runtime crosswalk to it.", "",
          "A database phase is matched to a registry solid by exact element counts and hydrate",
          "count, the rule `derived::registry_solid_matching` applies. Only a matched phase can",
          "precipitate: an unmatched one may be supersaturated and still deposit nothing, which is",
          "the failure the 2026-11-10 family scan found for AgBr, AgI, Ag2CO3 and BaCO3.", "",
          "This lane added eight common laboratory precipitates every one of which wateq4f and",
          "minteq.v4 already defined: AgBr (`Bromyrite`), AgI (`Iodyrite`), Ag2CO3, Ag3PO4,",
          "BaCO3 (`Witherite`), PbCO3 (`Cerussite`), PbSO4 (`Anglesite`) and PbI2. Identity is",
          "PubChem's InChIKey, molar mass is derived from IUPAC/CIAAW 2021 atomic weights, density",
          "and colour carry the citation in each row's provenance (two densities are marked pending",
          "review), and heat capacity is a Kopp's-rule estimate written out in the row. No",
          "solubility product is restated anywhere: the routed database's log K is the solubility.", "",
          "Chromates are absent from every class below for a different reason: the registry has no",
          "chromate ion, so a chromate solid could be neither fed nor read back. The `unbookable`",
          "class lists every such case.", "",
          f"Counts: {', '.join(f'{k} {v}' for k, v in doc['counts'].items())}.", "",
          "## Matched registry solids", "",
          "| registry species | formula | phases |", "|---|---|---|"]
    for r in rows:
        if r["class"] == "registry-solid":
            md.append(f"| `{r['registry_species']}` | {r['formula']} | {names(r)} |")
    md += ["", "## Common laboratory precipitates declined in this lane", "",
           "| formula | phases | why not yet |", "|---|---|---|"]
    for r in rows:
        if r["class"] == "declined":
            md.append(f"| {r['formula']} | {names(r)} | {r['reason']} |")
    for cls, title in (("bookable-unclaimed", "Bookable but unclaimed phases"),
                       ("unbookable", "Unbookable phases (an element has no registry ion)")):
        md += ["", f"## {title}", "", "| formula | phases |", "|---|---|"]
        for r in rows:
            if r["class"] == cls:
                md.append(f"| {r['formula']} | {names(r)} |")
    mdtext = "\n".join(md) + "\n"
    if a.check:
        bad = [p for p, t in ((OUT_JS, js), (OUT_MD, mdtext))
               if not p.exists() or p.read_text(encoding="utf-8") != t]
        if bad:
            print("stale:", *bad, file=sys.stderr)
            return 1
        return 0
    OUT_JS.write_text(js, encoding="utf-8")
    OUT_MD.write_text(mdtext, encoding="utf-8")
    print(json.dumps(doc["counts"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())

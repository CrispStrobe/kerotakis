#!/usr/bin/env python3
"""Coverage of two family rules (2026-11-11 lane, D3-AN-004 and SE-002).

1. Organic solid dissolution. Every registry organic molecular solid (neutral,
   C and H, non-metals only - `organic_solid_solubility::is_organic_molecular`)
   is classified:
     registry-solubility   the registry carries a reviewed figure
     reviewed-row          `organic_solid_solubility::ROWS` carries a cited series
     other-route           dissolution owned by another route (named)
     refused               `not-modeled.organic-solid-no-solubility`; a curated
                           reaction drawing on its undissolved solid in water is
                           labelled `not-modeled.solid-phase-reaction`
2. Ion partition. Every registry ion is classified by `ion_partition::ion_class`
   and whether the no-transfer rule covers it against a hydrocarbon solvent
   (fewer than LIPOPHILIC_ION_CARBONS carbons) or declines (lipophilic ion).

Writes docs/organic-solid-dissolution-coverage-20261111.{md,json} and
docs/ion-partition-coverage-20261111.{md,json}; `--check` fails when stale.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REG = ROOT / "data/registry/registry-source-v1.json"
SOL_SRC = ROOT / "crates/kerotakis-core/src/organic_solid_solubility.rs"
ION_SRC = ROOT / "crates/kerotakis-core/src/ion_partition.rs"
DOCS = ROOT / "docs"
SOL_OUT = DOCS / "organic-solid-dissolution-coverage-20261111"
ION_OUT = DOCS / "ion-partition-coverage-20261111"

NON_METALS = {"C", "H", "O", "N", "S", "P", "F", "Cl", "Br", "I"}
# Dissolution owned by a different, already-wired route. Named, not guessed.
OTHER_ROUTE = {
    "citric_acid": "speciated as Citrate by the PHREEQC derived route (kerotakis-phreeqc derived.rs)",
}

ELEMENT = re.compile(r"([A-Z][a-z]?)(\d*\.?\d*)")


def parse(formula: str):
    """Element counts and charge, enough for the registry's flat formulas."""
    body, charge = formula, 0
    m = re.match(r"^(.*?)([+-])(\d*)$", formula)
    if m and m.group(1):
        body = m.group(1)
        charge = (1 if m.group(2) == "+" else -1) * int(m.group(3) or 1)
    counts: dict[str, float] = {}
    for el, n in ELEMENT.findall(body.replace("(", "").replace(")", "")):
        counts[el] = counts.get(el, 0) + float(n or 1)
    return counts, charge


def reviewed_rows():
    src = SOL_SRC.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    rows = {}
    for m in re.finditer(r'OrganicSolubilityRow \{\s*species: "([^"]+)",\s*mole_fraction: &\[(.*?)\],\s*source:\s*"([^"]+)"', src, re.S):
        pts = [(float(t.replace("_", "")), float(x.replace("_", "")))
               for t, x in re.findall(r"\(([\d._]+),\s*([\d._]+)\)", m.group(2))]
        rows[m.group(1)] = dict(points=pts, source=m.group(3))
    return rows


def lipophilic_carbons():
    src = ION_SRC.read_text(encoding="utf-8")
    return float(re.search(r"LIPOPHILIC_ION_CARBONS: f64 = ([\d.]+);", src).group(1))


def ion_class(counts, charge):
    c, h = counts.get("C", 0), counts.get("H", 0)
    if c >= 2 or (c >= 1 and h >= 2):
        return "small organic ion"
    if len(counts) == 1 and sum(counts.values()) <= 1:
        return "monatomic cation" if charge > 0 else "monatomic anion"
    return "polyatomic inorganic ion"


def write(base: Path, md: list[str], data: dict, check: bool) -> bool:
    text_md = "\n".join(md) + "\n"
    text_json = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    md_path, json_path = base.with_suffix(".md"), base.with_suffix(".json")
    if check:
        ok = True
        for p, t in ((md_path, text_md), (json_path, text_json)):
            if not p.exists() or p.read_text(encoding="utf-8") != t:
                print("stale:", p, file=sys.stderr)
                ok = False
        return ok
    md_path.write_text(text_md, encoding="utf-8")
    json_path.write_text(text_json, encoding="utf-8")
    return True


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    reg = json.loads(REG.read_text(encoding="utf-8"))
    phase = {r["species_id"]: r["phase"] for r in reg["phase_thermodynamics"] if r["property"] == "molar_mass"}
    mass = {r["species_id"]: r["quantity"]["value"] for r in reg["phase_thermodynamics"] if r["property"] == "molar_mass"}
    density = {r["species_id"]: r["quantity"]["value"] for r in reg["phase_thermodynamics"]
               if r["property"] == "mass_density"}
    formula = {r["species_id"]: r["formula"] for r in reg["compositions"]}
    sol = {r["subject"]["id"]: r["quantity"]["value"] for r in reg["model_parameters"]
           if r["parameter"] == "aqueous-solubility-g-per-100-ml"}
    rows = reviewed_rows()

    # ---- 1. organic solids
    out = []
    for sid in sorted(formula):
        counts, charge = parse(formula[sid])
        if phase.get(sid) != "solid" or charge != 0 or "C" not in counts or "H" not in counts:
            continue
        if not set(counts) <= NON_METALS:
            continue
        if sid in sol:
            out.append(dict(species=sid, cls="registry-solubility", dissolves_by_route=True,
                            g_per_100_ml=sol[sid], source="registry model_parameters aqueous-solubility-g-per-100-ml"))
        elif sid in rows:
            t, x = rows[sid]["points"][0]
            g = x / (1 - x) * mass[sid] / mass["water"] * 100 * density["water"]
            out.append(dict(species=sid, cls="reviewed-row", dissolves_by_route=True,
                            g_per_100_ml=round(g, 3), at_k=t, source=rows[sid]["source"]))
        elif sid in OTHER_ROUTE:
            out.append(dict(species=sid, cls="other-route", dissolves_by_route=False,
                            g_per_100_ml=None, source=OTHER_ROUTE[sid]))
        else:
            out.append(dict(species=sid, cls="refused", dissolves_by_route=False, g_per_100_ml=None,
                            source="no primary measurement read: not-modeled.organic-solid-no-solubility"))
    counts_by = {}
    for r in out:
        counts_by[r["cls"]] = counts_by.get(r["cls"], 0) + 1
    md = ["# Organic solid dissolution coverage (2026-11-11)", "",
          "Generated by `tools/solid_reagent_ion_partition_coverage.py --check`. Registry organic",
          "molecular solids (neutral, C and H, non-metals only). One mechanism: a solid with a reviewed",
          "aqueous solubility (registry figure, or a cited series in `organic_solid_solubility::ROWS`)",
          "dissolves through `solve::saturation_moves` before any reaction; one without is refused with",
          "`not-modeled.organic-solid-no-solubility` (EN/DE/FR); a curated reaction that still draws on",
          "such an undissolved solid in water is labelled `not-modeled.solid-phase-reaction`.", "",
          "Reviewed rows convert a measured saturation mole fraction x with the registry's molar masses and",
          "water density: g/100 mL water = x/(1-x) * M/M_water * 100 * rho_water. Between measured",
          "temperatures x is interpolated linearly; outside it is held at the nearest end.", "",
          "Source of the new row: Shalmashi, A.; Eliassi, A. J. Chem. Eng. Data 2008, 53(6), 1332-1334,",
          "doi:10.1021/je800056h (L-ascorbic acid, water, 293-323 K, gravimetric; values as deposited in",
          "NIST ThermoML).", "",
          "Maltose row (2026-11-11, batch3-diverse:en-001): Gong, X.; Wang, C.; Zhang, L.; Qu, H.",
          "J. Chem. Eng. Data 2012, 57(11), 3264-3269, doi:10.1021/je300885g, NIST ThermoML deposit:",
          "D-(+)-maltose monohydrate in water, mass fraction w = 0.4106 / 0.4424 / 0.4792 at",
          "278.2 / 288.2 / 298.2 K (gravimetric). Converted to anhydrous-maltose mole fraction with the",
          "hydrate water counted as solvent. Other sugars: glucose, fructose and sucrose already carry a",
          "registry solubility; no other sugar is a registry solid.", "",
          f"Counts: {', '.join(f'{k} {v}' for k, v in sorted(counts_by.items()))}.", "",
          "| species | class | dissolves by route | g/100 mL | source |", "|---|---|---|---|---|"]
    for r in out:
        g = "—" if r["g_per_100_ml"] is None else f"{r['g_per_100_ml']:g}" + (f" at {r['at_k']:g} K" if "at_k" in r else "")
        md.append(f"| {r['species']} | {r['cls']} | {'yes' if r['dissolves_by_route'] else 'no'} | {g} | {r['source']} |")
    ok = write(SOL_OUT, md, dict(lane="2026-11-11 solid reagent", counts=counts_by, rows=out), a.check)

    # ---- 2. ions
    limit = lipophilic_carbons()
    ions = []
    for sid in sorted(formula):
        counts, charge = parse(formula[sid])
        if charge == 0:
            continue
        covered = counts.get("C", 0) / abs(charge) < limit
        ions.append(dict(species=sid, formula=formula[sid], ion_class=ion_class(counts, charge),
                         hydrocarbon_rule="retained in water" if covered else "declines: lipophilic ion"))
    solvents = sorted(sid for sid in formula if phase.get(sid) == "liquid"
                      and set(parse(formula[sid])[0]) == {"C", "H"} and parse(formula[sid])[1] == 0)
    by_class = {}
    for r in ions:
        by_class[r["ion_class"]] = by_class.get(r["ion_class"], 0) + 1
    md = ["# Ion partition coverage (2026-11-11)", "",
          "Generated by `tools/solid_reagent_ion_partition_coverage.py --check`. Rule (`ion_partition`):",
          "a charged solute is not extracted into a hydrocarbon solvent (registry formula only C and H)",
          "without an ion-pairing agent. It is reported in the `extracted` event as `retained` with the",
          "rule and its sources, not as a missing distribution coefficient. The rule declines for an ion",
          f"with {limit:g} or more carbons per unit charge and for every ion beside one (ion-pair extraction is possible);",
          "polar solvents keep the typed `not-modeled.extraction-without-coefficient` refusal.", "",
          "Sources: Rydberg, J.; Cox, M.; Musikas, C.; Choppin, G. R. (eds.) *Solvent Extraction",
          "Principles and Practice*, 2nd ed., Marcel Dekker, New York, 2004, ISBN 978-0-8247-5063-3",
          "(extraction of ions requires conversion to a neutral ion pair or complex). Marcus, Y.",
          "J. Chem. Soc., Faraday Trans. 1991, 87, 2995-2999, doi:10.1039/FT9918702995 (Gibbs energies of",
          "ion hydration, hundreds of kJ/mol, not repaid in an alkane).", "",
          f"Hydrocarbon solvents in the registry: {', '.join(solvents)}.", "",
          f"Ion classes: {', '.join(f'{k} {v}' for k, v in sorted(by_class.items()))}.", "",
          "| ion | formula | class | against a hydrocarbon |", "|---|---|---|---|"]
    md += [f"| {r['species']} | {r['formula']} | {r['ion_class']} | {r['hydrocarbon_rule']} |" for r in ions]
    ok = write(ION_OUT, md, dict(lane="2026-11-11 ion partition", hydrocarbon_solvents=solvents,
                                 classes=by_class, rows=ions), a.check) and ok
    if a.check:
        return 0 if ok else 1
    print(json.dumps(dict(organic=counts_by, ions=by_class)))
    return 0


if __name__ == "__main__":
    sys.exit(main())

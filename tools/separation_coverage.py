#!/usr/bin/env python3
"""Separation-verb property coverage (2026-11-09).

Lists every species for which `extract` (hexane/water) has a distribution
coefficient and every species `distil` can carry overhead, read from the
same sources the engine compiles in:

  extract  empirical           apparatus.rs PARTITION_COEFFICIENTS rows
           unifac-group-model  bench.rs partition_groups (hexane only)
  extract  liquid owner        apparatus.rs WATER_MISCIBLE_LIQUIDS: a
                               liquid-labelled portion counts as dissolved
  distil   binary-unifac-still water, ethanol (vle.rs binary still)
           uscg-chris-ideal-still  data/thermo/uscg-chris-still.tsv

Every other condensed solute is a typed refusal naming it
(`not-modeled.no-distribution-coefficient`,
`not-modeled.extraction-without-coefficient`,
`not-modeled.no-reviewed-still-properties`), never a silently empty
receiver. The `watched` section records the species whose missing data held
the separation review cases, so a future data row shows up as a diff.

Writes docs/separation-coverage-20261109.{md,json}; `--check` fails when
stale. The Rust test `separation_coverage_is_current` holds the runtime
(`bench::extraction_partition_basis`, `volatility::still_property_basis`)
to the JSON for every registry species.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/kerotakis-core/src"

# Species whose absent data produced the held separation cases; the reason
# for each is engine-observed (production run 2026-10-28), not a forecast.
WATCHED = {
    "naphthalene": "extract: no hexane/water K (no empirical row, no UNIFAC decomposition)",
    "CH3COOH": "extract: no K; distil: no reviewed vapour pressure/latent heat (vapour dimerises)",
    "CH3COO-": "extract: ion, no reviewed ion-pair partition",
    "ethyl_acetate": "extract: liquid owner not admitted (partly miscible) and no K",
    "Na+": "extract: ion, no reviewed ion-pair partition",
    "Cl-": "extract: ion, no reviewed ion-pair partition",
    "K+": "extract: ion, no reviewed ion-pair partition",
    "I-": "extract: ion, no reviewed ion-pair partition",
    "S4O6-2": "extract: ion, no reviewed ion-pair partition",
    "H+": "extract: ion, no reviewed ion-pair partition",
}


def strip_tests(text: str) -> str:
    return text.split("#[cfg(test)]")[0]


def empirical_rows():
    src = strip_tests((SRC / "apparatus.rs").read_text(encoding="utf-8"))
    block = src.split("pub const PARTITION_COEFFICIENTS")[1].split("];")[0]
    rows = []
    for chunk in block.split("PartitionCoefficient {")[1:]:
        get = lambda f: re.search(rf'\b{f}:\s*([^,\n]+)', chunk).group(1).strip().strip('"')
        rows.append(dict(solute=get("solute"), solvent=get("organic_solvent"),
                         k=float(get("k_organic_over_aqueous")),
                         reference_temperature_k=float(get("reference_temperature_k")),
                         tolerance_k=float(get("temperature_tolerance_k"))))
    return rows


def miscible_liquids():
    src = (SRC / "apparatus.rs").read_text(encoding="utf-8")
    body = re.search(r"WATER_MISCIBLE_LIQUIDS: &\[&str\] = &\[([^\]]*)\]", src).group(1)
    return re.findall(r'"([^"]+)"', body)


def unifac_species():
    src = (SRC / "bench.rs").read_text(encoding="utf-8")
    body = src.split("pub(crate) fn partition_groups(")[1].split("Some(g)")[0]
    return re.findall(r'^\s*"([^"]+)"\s*=>', body, re.M)


def chris_rows():
    rows = []
    for line in (ROOT / "data/thermo/uscg-chris-still.tsv").read_text(encoding="utf-8").splitlines():
        if line.startswith("#"):
            continue
        f = line.split()
        if len(f) == 5:
            rows.append(dict(species=f[0], boiling_k=float(f[1]), latent_j_per_kg=float(f[2])))
    return rows


def build():
    registry = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text(encoding="utf-8"))
    keys = {ident["id"] for ident in registry["identities"]}
    extract = {}
    for row in empirical_rows():
        if row["solvent"] == "hexane":
            extract[row["solute"]] = dict(basis="empirical", k=row["k"],
                                          reference_temperature_k=row["reference_temperature_k"],
                                          tolerance_k=row["tolerance_k"])
    for key in unifac_species():
        extract.setdefault(key, dict(basis="unifac-group-model"))
    distil = {"water": dict(basis="binary-unifac-still"), "ethanol": dict(basis="binary-unifac-still")}
    for row in chris_rows():
        distil.setdefault(row["species"], dict(basis="uscg-chris-ideal-still",
                                               boiling_k=row["boiling_k"],
                                               latent_j_per_kg=row["latent_j_per_kg"]))
    miscible = miscible_liquids()
    covered = sorted(set(extract) | set(distil) | set(miscible))
    species = []
    for key in covered:
        species.append({
            "species": key,
            "registered": key in keys,
            "extract_hexane": extract.get(key),
            "liquid_owner_admitted": key in miscible,
            "distil": distil.get(key),
        })
    watched = [dict(species=k, registered=k in keys,
                    extract_hexane=extract.get(k, {}).get("basis"),
                    distil=distil.get(k, {}).get("basis"), gap=v) for k, v in sorted(WATCHED.items())]
    return dict(schema="kerotakis.separation-coverage.v1", extract_solvent="hexane",
                counts=dict(extract=len(extract), distil=len(distil), liquid_owner_admitted=len(miscible)),
                species=species, watched=watched)


def markdown(doc):
    out = ["# Separation-verb property coverage (2026-11-09)", "",
           "Generated by `tools/separation_coverage.py`; `separation_coverage_is_current` fails when",
           "the runtime disagrees. `extract` moves a solute only with a distribution coefficient from",
           "this table; `distil` carries a condensed owner overhead only with properties from it.",
           "Any other solute is named in a typed not-modelled reason (EN/DE/FR) - an extraction",
           "that can move nothing dispenses no solvent and leaves the receiver unchanged, and says so.",
           "",
           f"Counts: extract {doc['counts']['extract']}, distil {doc['counts']['distil']}, "
           f"liquid owners admitted {doc['counts']['liquid_owner_admitted']}.", "",
           "| species | extract (hexane/water) | K | liquid owner admitted | distil |",
           "|---|---|---|---|---|"]
    for r in doc["species"]:
        e = r["extract_hexane"] or {}
        d = r["distil"] or {}
        out.append(f"| {r['species']} | {e.get('basis', '-')} | {e.get('k', '-')} | "
                   f"{'yes' if r['liquid_owner_admitted'] else 'no'} | {d.get('basis', '-')} |")
    out += ["", "## Watched gaps", "",
            "Species whose missing data held separation review cases. They stay typed refusals",
            "until a reviewed row is added; no value is invented here.", "",
            "| species | extract | distil | gap |", "|---|---|---|---|"]
    for w in doc["watched"]:
        out.append(f"| {w['species']} | {w['extract_hexane'] or 'missing'} | "
                   f"{w['distil'] or ('missing' if 'distil' in w['gap'] else '-')} | {w['gap']} |")
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    doc = build()
    js = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"
    md = markdown(doc)
    out_js = ROOT / "docs/separation-coverage-20261109.json"
    out_md = ROOT / "docs/separation-coverage-20261109.md"
    if a.check:
        bad = [p for p, t in ((out_js, js), (out_md, md)) if not p.exists() or p.read_text(encoding="utf-8") != t]
        if bad:
            print("stale:", *bad, file=sys.stderr)
            return 1
        return 0
    out_js.write_text(js, encoding="utf-8")
    out_md.write_text(md, encoding="utf-8")
    print(json.dumps(doc["counts"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())

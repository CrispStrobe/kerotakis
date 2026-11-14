#!/usr/bin/env python3
"""Melting-range latent-heat coverage (po-009 / th-004, 2026-11-11).

Reads `MELTING_RANGES` and `IMMISCIBLE_MELTS` from
crates/kerotakis-core/src/melting_range.rs and classifies:

  every range row
    paid      sourced lower bound AND sourced enthalpy of fusion: heating
              through the range pays it (`melting_range::settle_range_melts`)
    refused   either is missing: past the upper bound the bench says
              `not-modeled.melting-range-latent-heat-unsourced` (EN/DE/FR)

  every pair of range rows (a blend read by the melting-point apparatus)
    immiscible       sourced as immiscible in the melt: no depression claimed
    solution-law     not sourced as immiscible: the conditional depression
                     law (`instrument::MIXTURE_BOUNDARY`) is stated

It also checks that the refusal and the immiscible-blend render keys exist
in the DE and FR catalogues.

Writes docs/melting-range-latent-heat-coverage-20261111.{md,json};
`--check` fails when either is stale or a key is missing.
"""
from __future__ import annotations

import argparse
import itertools
import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/kerotakis-core/src/melting_range.rs"
I18N = ROOT / "crates/kerotakis-core/i18n"
MD = ROOT / "docs/melting-range-latent-heat-coverage-20261111.md"
JS = ROOT / "docs/melting-range-latent-heat-coverage-20261111.json"
KEYS = [
    "not-modeled.melting-range-latent-heat-unsourced",
    "event.transition-point.immiscible.lv1",
    "event.transition-point.immiscible.lv2",
    "event.transition-point.immiscible.lv3",
]


def opt_float(text: str):
    m = re.match(r"Some\(([\d_.]+)\)", text.strip())
    return float(m.group(1).replace("_", "")) if m else None


def tables():
    src = SRC.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    start = src.index("pub const MELTING_RANGES")
    body = src[start:src.index("];", start)]
    rows = []
    for m in re.finditer(r"MeltingRange \{(.*?)\n    \}", body, re.S):
        b = m.group(1)
        rows.append(dict(
            species=re.search(r'species: "([^"]+)"', b).group(1),
            lower_k=opt_float(re.search(r"lower_k: ([^,]+),", b).group(1)),
            upper_k=float(re.search(r"upper_k: ([\d_.]+),", b).group(1).replace("_", "")),
            fusion_j_per_kg=opt_float(re.search(r"fusion_j_per_kg: ([^,]+),", b).group(1)),
            crystallinity=opt_float(re.search(r"crystallinity: ([^,]+),", b).group(1)),
            fusion_ref_j_per_g=opt_float(re.search(r"fusion_ref_j_per_g: ([^,]+),", b).group(1)),
            source=re.search(r'source: "([^"]+)"', b).group(1),
        ))
    start = src.index("pub const IMMISCIBLE_MELTS")
    body = src[start:src.index("];", start)]
    pairs = []
    for m in re.finditer(r"ImmiscibleMelt \{(.*?)\}", body, re.S):
        b = m.group(1)
        pairs.append(dict(
            a=re.search(r'a: "([^"]+)"', b).group(1),
            b=re.search(r'b: "([^"]+)"', b).group(1),
            source=re.search(r'source: "([^"]+)"', b).group(1),
        ))
    start = src.index("pub const REFUSED_FUSION")
    body = src[start:src.index("];", start)]
    refused = [dict(material=m.group(1), reason=m.group(2)) for m in
               re.finditer(r'material: "([^"]+)",\s*reason: "([^"]+)"', body)]
    return rows, pairs, refused


# Every polymer or wax the registry carries, and where its melting latent
# heat lives (2026-11-11 polymer/wax lane).
REGISTRY_MATERIALS = [
    ("paraffin", "range row"), ("PE", "range row"), ("PP", "range row"),
    ("PET", "range row"), ("nylon", "range row"),
    ("hexadecane", "phase_route::FUSION_ENTHALPIES (sharp point)"),
    ("PS", "none: atactic polystyrene is amorphous, no melting endotherm"),
    ("PVA", "none: 161 J/g 100 % reference (Wunderlich via TN048) but no sourced grade crystallinity"),
]
REGISTRY_JSON = ROOT / "data/registry/registry-source-v1.json"
PHASE_ROUTE = ROOT / "crates/kerotakis-core/src/phase_route.rs"


def flatten(table, prefix=""):
    out = {}
    for k, v in table.items():
        key = f"{prefix}{k}"
        if isinstance(v, dict):
            out.update(flatten(v, key + "."))
        else:
            out[key] = v
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    rows, pairs, refused = tables()
    problems = []
    for r in rows:
        r["class"] = "paid" if r["lower_k"] is not None and r["fusion_j_per_kg"] is not None else "refused"
        xc, ref = r["crystallinity"], r["fusion_ref_j_per_g"]
        if (xc is None) != (ref is None):
            problems.append(f"{r['species']}: Xc and reference must come together")
        elif xc is not None and abs(xc * ref * 1000.0 - (r["fusion_j_per_kg"] or 0)) > 1e-6 * xc * ref * 1000.0:
            problems.append(f"{r['species']}: fusion_j_per_kg != Xc x reference")
    reg_ids = {i["id"] for i in json.loads(REGISTRY_JSON.read_text(encoding="utf-8"))["identities"]}
    fus_src = PHASE_ROUTE.read_text(encoding="utf-8")
    range_keys = {r["species"] for r in rows}
    inventory = []
    for key, where in REGISTRY_MATERIALS:
        if key not in reg_ids:
            problems.append(f"{key} not in registry")
        if where == "range row":
            paid = key in range_keys and next(r for r in rows if r["species"] == key)["class"] == "paid"
        elif where.startswith("phase_route"):
            paid = f'species: "{key}",' in fus_src
        else:
            paid = False
        inventory.append(dict(species=key, latent_heat=where, paid=paid))
    imm = {frozenset((p["a"], p["b"])): p for p in pairs}
    blends = []
    for x, y in itertools.combinations(sorted(r["species"] for r in rows), 2):
        p = imm.get(frozenset((x, y)))
        blends.append(dict(a=x, b=y, cls="immiscible" if p else "solution-law",
                           source=p["source"] if p else None))
    missing = []
    for lang in ("de", "fr"):
        cat = flatten(tomllib.loads((I18N / f"{lang}.toml").read_text(encoding="utf-8")))
        missing += [f"{lang}:{k}" for k in KEYS if k not in cat]

    data = dict(
        generated_by="tools/melting_range_latent_heat_coverage.py",
        model="liquid fraction linear from lower to upper bound; sourced enthalpy of fusion spread evenly over it",
        ranges=rows,
        registry_polymers_and_waxes=inventory,
        refused_by_policy=refused,
        blends=blends,
        refusal_key=KEYS[0],
    )
    md = [
        "# Melting-range latent-heat coverage (2026-11-11)", "",
        "Generated by `tools/melting_range_latent_heat_coverage.py --check` from",
        "`melting_range::MELTING_RANGES` and `melting_range::IMMISCIBLE_MELTS`.", "",
        "**Mechanism.** A material that melts over a sourced range pays its sourced enthalpy of fusion",
        "as it is heated through the range (`melting_range::settle_range_melts`, run by the phase-route",
        "equilibrator): the liquid fraction rises linearly from 0 at the lower bound to 1 at the upper,",
        "and the vessel's temperature and split are solved from one energy balance. Cooling back",
        "through the range releases the same heat. The linear shape is a stated model choice; only",
        "the total and the span are sourced. Where the lower bound or the enthalpy of fusion is not",
        "sourced, nothing is paid and past the upper bound the bench refuses with",
        f"`{KEYS[0]}` (EN/DE/FR), which says the thermometer omits the latent heat.", "",
        "**Blends.** The melting-point depression law holds only for components that dissolve in each",
        "other's melt (a solution or eutectic system). A blend whose components are all sourced as",
        "immiscible in the melt is reported as such (`immiscible_blend`), with no depression claimed:",
        "each component melts over its own range.", "",
        "## Range rows", "",
        "**Polymers.** A semicrystalline polymer's latent heat is that of its crystalline part,",
        "ΔH_f(grade) = X_c × ΔH_f(100 %). Each polymer row records both factors from one sourced DSC",
        "measurement of a named grade, with the reference that paper used. The grade's DSC peak is the",
        "lower end of the paid span; the upper end is the sourced equilibrium melting temperature",
        "(PE, PP) or, where none is sourced, the peak itself (PET, nylon 6,6: a plateau, the whole heat",
        "paid at one temperature, partial melts resolved by the energy balance).", "",
        "| species | lower (K) | upper (K) | X_c | ΔH_f 100 % (J/g) | ΔH_fus (J/kg) | class | source |",
        "|---|---|---|---|---|---|---|---|",
    ]
    dash = lambda v: "—" if v is None else v
    for r in rows:
        md.append(f"| {r['species']} | {r['lower_k'] if r['lower_k'] is not None else '— (unsourced)'} | "
                  f"{r['upper_k']} | {dash(r['crystallinity'])} | {dash(r['fusion_ref_j_per_g'])} | "
                  f"{r['fusion_j_per_kg'] if r['fusion_j_per_kg'] is not None else '— (unsourced)'} | "
                  f"{r['class']} | {r['source']} |")
    md += ["", "## Every polymer and wax in the registry", "", "| species | latent heat | paid |", "|---|---|---|"]
    md += [f"| {i['species']} | {i['latent_heat']} | {'yes' if i['paid'] else 'no'} |" for i in inventory]
    md += ["", "## Refused by policy (`melting_range::REFUSED_FUSION`)", "", "| material | reason |", "|---|---|"]
    md += [f"| {r['material']} | {r['reason']} |" for r in refused]
    md += ["", "## Blends of range materials", "", "| a | b | class | source |", "|---|---|---|---|"]
    md += [f"| {b['a']} | {b['b']} | {b['cls']} | {b['source'] or '—'} |" for b in blends]
    md += ["", "## Notes", "",
           "- Paraffin: 189 kJ/kg is the value Sharma et al. (2009) Table 2a list for five technical grades,",
           "  compiled from Abhat (1983), doi:10.1016/0038-092X(83)90186-X; Wikipedia's *Phase-change",
           "  material* table quotes 150-250 kJ/kg for paraffin waxes citing Kenisarin and Mahkamov (2007),",
           "  doi:10.1016/j.rser.2006.05.005. It is spread over the span of the grades",
           "  (315.15-341.15 K) because the grade in the beaker is not known.",
           "- Polymer 100 % references: PE 293, PP 207, PET 140 J/g from B. Wunderlich, *Thermal Analysis*",
           "  (Academic Press 1990) pp. 417-431, as tabulated by R. L. Blaine, TA Instruments TN048; PA66",
           "  uses the 188 J/g its grade paper used (TN048 lists 226 J/g; the choice moves X_c, not the",
           "  measured grade heat). PP's reference is disputed (165-209 J/g in use), same remark.",
           "- Sourced but not in the registry (no row until the species exists): LDPE 30.58 % x 293 J/g",
           "  (doi:10.3390/polym18202566), POM 44.3 % x 326.3 J/g (doi:10.3390/nano12050858), PCL Capa",
           "  57.4-65.0 J/g (doi:10.3390/ma19204377), sintered PTFE 39.5-43.1 % x 82 J/g",
           "  (doi:10.3390/polym17101380), stearic acid 201.0-216.5 J/g (Europe PMC PMC7490278,",
           "  PMC6515142), beeswax 137.6 J/g (doi:10.3390/ma18245521; other reports to 221 J/g, so",
           "  range-only).",
           "- Hexadecane is a pure alkane with a sharp point: its 53.36 kJ/mol (235.6 J/g; Finke et al.",
           "  1954, doi:10.1021/ja01631a005, as listed by the NIST WebBook) is paid by",
           "  `phase_route::FUSION_ENTHALPIES`, not by a range row.",
           "- A range material added to a vessel above its lower bound is still stocked solid (stocking is",
           "  owned by `phase_route::stocked_phase`); the next settle melts it and pays from the vessel's",
           "  sensible heat.", ""]
    text = "\n".join(md)
    js = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    if a.check:
        bad = False
        for path, want in ((MD, text), (JS, js)):
            if not path.exists() or path.read_text(encoding="utf-8") != want:
                print("stale:", path, file=sys.stderr)
                bad = True
        if missing:
            print("missing locale keys:", missing, file=sys.stderr)
            bad = True
        if problems:
            print("problems:", problems, file=sys.stderr)
            bad = True
        return 1 if bad else 0
    MD.write_text(text, encoding="utf-8")
    JS.write_text(js, encoding="utf-8")
    print(json.dumps({"paid": sum(r["class"] == "paid" for r in rows),
                      "refused": sum(r["class"] == "refused" for r in rows),
                      "registry_paid": sum(i["paid"] for i in inventory),
                      "registry_total": len(inventory),
                      "policy_refused": len(refused),
                      "missing_keys": missing, "problems": problems}))
    return 1 if missing or problems else 0


if __name__ == "__main__":
    sys.exit(main())

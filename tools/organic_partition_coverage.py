#!/usr/bin/env python3
"""Organic hexane/water partition coverage (2026-11-10).

Lists every solute `extract` (hexane/water) and the water-mobile-phase
chromatograph can partition through the UNIFAC group model, read from the
same sources the engine compiles in:

  decompositions   crates/kerotakis-core/src/bench.rs partition_groups
  r, q, a_mn       crates/kerotakis-thermo/src/unifac.rs approved_table
  dissolved bound  crates/kerotakis-core/src/apparatus.rs PARTLY_MISCIBLE_LIQUIDS
  miscible owners  crates/kerotakis-core/src/apparatus.rs WATER_MISCIBLE_LIQUIDS

For each solute it re-computes, in plain Python from those parsed numbers,
the infinite-dilution activity-coefficient ratio gamma_inf(water) /
gamma_inf(hexane) at 298.15 K: the mole-fraction partition ratio. The
runtime multiplies it by V_m(water) / V_m(hexane) to get the
concentration-basis K the funnel uses. The Rust test
`organic_partition_coverage_is_current` holds the runtime to these numbers.

These are UNIFAC (VLE parameter) PREDICTIONS for hexane/water, not measured
hexane/water distribution coefficients and not octanol/water log P. No
octanol value is used or converted anywhere. Ions (acetate, H+, Na+, ...)
have no decomposition: they stay in the water and are named. A weak acid is
partitioned only as its neutral form, so the extractable share follows the
speciation (pH) of the aqueous phase.

Writes docs/organic-partition-coverage-20261110.{md,json}; `--check` fails
when stale.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates/kerotakis-core/src"
THERMO = ROOT / "crates/kerotakis-thermo/src/unifac.rs"
T_K = 298.15
INFINITE_DILUTION_X = 1e-9  # mirrors lle::infinite_dilution_gamma
WATER = {16: 1}
HEXANE = {1: 2, 2: 4}

# Ionisable solutes: the neutral form is the one partitioned; the conjugate
# base named here stays aqueous. pKa is informational (the runtime reads the
# speciation the vessel already holds, it does not recompute it).
IONISABLE = {"CH3COOH": dict(conjugate_base="CH3COO-", pka=4.76,
                             pka_source="CRC Handbook, dissociation constants of organic acids, 25 C")}


def table():
    src = THERMO.read_text(encoding="utf-8").split("pub fn approved_table")[1].split("#[cfg(test)]")[0]
    groups = {}
    for chunk in src.split("UnifacGroup {")[1:]:
        get = lambda f: re.search(rf"\b{f}:\s*([^,\n]+)", chunk).group(1).strip().strip('"')
        groups[int(get("id"))] = (float(get("r")), float(get("q")), int(get("main_group")))
    inter = {}
    for chunk in src.split("InteractionParam {")[1:]:
        get = lambda f: re.search(rf"\b{f}:\s*([^,\n]+)", chunk).group(1).strip()
        inter[(int(get("m")), int(get("n")))] = float(get("a_mn"))
    return groups, inter


def decompositions():
    src = (CORE / "bench.rs").read_text(encoding="utf-8")
    body = src.split("pub(crate) fn partition_groups(")[1].split("_ => return None")[0]
    out = {}
    for m in re.finditer(r'^\s*"([^"]+)"\s*=>\s*\{(.*?)\}', body, re.M | re.S):
        out[m.group(1)] = {int(g): int(c) for g, c in re.findall(r"g\.insert\((\d+),\s*(\d+)\)", m.group(2))}
    return out


def miscible():
    src = (CORE / "apparatus.rs").read_text(encoding="utf-8")
    body = re.search(r"WATER_MISCIBLE_LIQUIDS: &\[&str\] = &\[([^\]]*)\]", src).group(1)
    return re.findall(r'"([^"]+)"', body)


def partly_miscible():
    src = (CORE / "apparatus.rs").read_text(encoding="utf-8")
    block = src.split("pub const PARTLY_MISCIBLE_LIQUIDS")[1].split("}];")[0]
    rows = {}
    for chunk in block.split("PartlyMiscibleLiquid {")[1:]:
        get = lambda f: re.search(rf"\b{f}:\s*([^,\n]+)", chunk).group(1).strip().strip('"')
        rows[get("species")] = dict(dissolved_g_per_100_g_water=float(get("dissolved_g_per_100_g_water")),
                                    reference_temperature_k=float(get("reference_temperature_k")),
                                    tolerance_k=float(get("temperature_tolerance_k")))
    return rows


def ln_gamma(groups, inter, comps, x, t):
    def a(m, n):
        if m == n:
            return 0.0
        if (m, n) not in inter:
            raise KeyError(f"missing a({m},{n})")
        return inter[(m, n)]

    r = [sum(c * groups[g][0] for g, c in d.items()) for d in comps]
    q = [sum(c * groups[g][1] for g, c in d.items()) for d in comps]
    sr = sum(ri * xi for ri, xi in zip(r, x))
    sq = sum(qi * xi for qi, xi in zip(q, x))
    phi = [ri * xi / sr for ri, xi in zip(r, x)]
    th = [qi * xi / sq for qi, xi in zip(q, x)]
    ell = [5 * (ri - qi) - (ri - 1) for ri, qi in zip(r, q)]

    def residual(counts):
        tot = sum(counts.values())
        xs = {g: v / tot for g, v in counts.items()}
        qq = sum(xs[g] * groups[g][1] for g in xs)
        theta = {g: xs[g] * groups[g][1] / qq for g in xs}
        psi = lambda m, n: math.exp(-a(groups[m][2], groups[n][2]) / t)
        return {k: groups[k][1] * (1 - math.log(sum(theta[m] * psi(m, k) for m in theta))
                                   - sum(theta[m] * psi(k, m) / sum(theta[n] * psi(n, m) for n in theta)
                                         for m in theta)) for k in theta}

    mix = {}
    for d, xi in zip(comps, x):
        for g, c in d.items():
            mix[g] = mix.get(g, 0.0) + c * xi
    gm = residual(mix)
    gp = residual(dict(comps[0]))
    i = 0
    comb = (math.log(phi[i] / x[i]) + 5 * q[i] * math.log(th[i] / phi[i]) + ell[i]
            - phi[i] / x[i] * sum(xj * lj for xj, lj in zip(x, ell)))
    return comb + sum(c * (gm[g] - gp[g]) for g, c in comps[0].items())


def gamma_inf(groups, inter, solute, solvent):
    x = INFINITE_DILUTION_X
    return math.exp(ln_gamma(groups, inter, [solute, solvent], [x, 1 - x], T_K))


def build():
    groups, inter = table()
    misc = set(miscible())
    partly = partly_miscible()
    rows = []
    for key, dec in sorted(decompositions().items()):
        ratio = gamma_inf(groups, inter, dec, WATER) / gamma_inf(groups, inter, dec, HEXANE)
        rows.append(dict(
            species=key,
            groups={str(g): c for g, c in sorted(dec.items())},
            mole_fraction_ratio_298=float(f"{ratio:.6e}"),
            liquid_owner="miscible" if key in misc else ("partly-miscible" if key in partly else "not-admitted"),
            dissolved_bound=partly.get(key),
            ionisable=IONISABLE.get(key),
        ))
    return dict(schema="kerotakis.organic-partition-coverage.v1", solvent="hexane", temperature_k=T_K,
                model="original UNIFAC infinite-dilution gamma ratio (VLE parameters); prediction, not measurement",
                octanol_proxy_used=False, species=rows)


def markdown(doc):
    out = ["# Organic hexane/water partition coverage (2026-11-10)", "",
           "Generated by `tools/organic_partition_coverage.py`; `organic_partition_coverage_is_current`",
           "fails when the runtime disagrees.", "",
           "`extract` with hexane and the water-mobile-phase chromatograph partition a neutral",
           "organic solute through the UNIFAC group model (Fredenslund, Jones & Prausnitz 1975;",
           "Gmehling, Rasmussen & Fredenslund 1982; Hansen et al. 1991 for the ACH and CCOO",
           "blocks). The ratio below is gamma_inf(water)/gamma_inf(hexane) at 298.15 K on a",
           "mole-fraction basis; the funnel multiplies it by V_m(water)/V_m(hexane) (about 0.138).",
           "",
           "**What this is not.** These are model predictions from VLE-fitted group parameters,",
           "not measured hexane/water distribution coefficients, and no octanol/water log P is",
           "used or converted. Treat the numbers as order-of-magnitude: e.g. naphthalene is",
           "predicted at log10 K about 3.8 (concentration basis), and acetic",
           "acid's dimerisation in hexane is not modelled.",
           "",
           "**Ions and weak acids.** Ions have no decomposition and stay in the water; they are",
           "named in the typed `not-modeled.extraction-without-coefficient` reason. A weak acid",
           "partitions only as its neutral form, so the extractable share follows the speciation",
           "(pH) the aqueous phase already holds: at pH well above the pKa almost nothing moves.",
           "",
           "**Liquid owners.** A liquid-labelled portion is extracted as a dissolved solute when",
           "it is fully miscible with water, or partly miscible and within the reviewed dissolved",
           "bound for the water present; otherwise it is named, never silently moved.",
           "",
           "**Chromatography of the organic layer.** The column's mobile phase is water; a vessel",
           "with no water gets the typed `not-modeled.chromatography-needs-an-aqueous-sample`",
           "reason (EN/DE/FR).", "",
           "Any solute not listed is a typed refusal naming it.", "",
           "| species | groups | gamma ratio (x basis, 298 K) | liquid owner | ionisable |",
           "|---|---|---|---|---|"]
    for r in doc["species"]:
        groups = " ".join(f"{g}x{c}" for g, c in r["groups"].items())
        owner = r["liquid_owner"]
        if r["dissolved_bound"]:
            owner += f" (<= {r['dissolved_bound']['dissolved_g_per_100_g_water']} g/100 g water)"
        ion = "-"
        if r["ionisable"]:
            ion = f"neutral form only; {r['ionisable']['conjugate_base']} stays aqueous (pKa {r['ionisable']['pka']})"
        out.append(f"| {r['species']} | {groups} | {r['mole_fraction_ratio_298']:.4e} | {owner} | {ion} |")
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    doc = build()
    js = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"
    md = markdown(doc)
    out_js = ROOT / "docs/organic-partition-coverage-20261110.json"
    out_md = ROOT / "docs/organic-partition-coverage-20261110.md"
    if a.check:
        bad = [p for p, t in ((out_js, js), (out_md, md)) if not p.exists() or p.read_text(encoding="utf-8") != t]
        if bad:
            print("stale:", *bad, file=sys.stderr)
            return 1
        return 0
    out_js.write_text(js, encoding="utf-8")
    out_md.write_text(md, encoding="utf-8")
    print(len(doc["species"]), "species")
    return 0


if __name__ == "__main__":
    sys.exit(main())

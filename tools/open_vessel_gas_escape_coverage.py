#!/usr/bin/env python3
"""Open-vessel gas escape coverage (lane E, 2026-11-11).

Reads `ESCAPE_TABLE` from crates/kerotakis-core/src/gas_escape.rs and the
Sander (2015) Henry table from crates/kerotakis-core/src/properties.rs, and
applies the same admission rule as `gas_escape::transfer_velocity_m_per_s`
at 298.15 K:

  covered   diffusivity + Henry constant + dimensionless H >= H(CO2)
            (liquid-film limit; Liss 1973 places O2/CO2 there)
  refused   `not-modeled.gas-escape-no-data` with the typed gap

Writes docs/open-vessel-gas-escape-coverage-20261111.{md,json};
`--check` fails when either is stale.
"""
from __future__ import annotations

import argparse
import json
import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/kerotakis-core/src/gas_escape.rs"
PROPS = ROOT / "crates/kerotakis-core/src/properties.rs"
CLOCK = ROOT / "crates/kerotakis-core/src/clock.rs"
MD = ROOT / "docs/open-vessel-gas-escape-coverage-20261111.md"
JS = ROOT / "docs/open-vessel-gas-escape-coverage-20261111.json"
R = 0.082057366
T = 298.15

SOURCES = [
    "k_L,ref = 5.75e-6 m/s: J. J. Cole and N. F. Caraco (1998), Limnol. Oceanogr. 43, 647-656, "
    "doi:10.4319/lo.1998.43.4.0647 (zero-wind intercept of k600, Mirror Lake). Anchored on a "
    "natural still-water surface, NOT a lab-beaker measurement; an upper bound for a beaker.",
    "Schmidt exponent -2/3 (smooth surface): B. Jaehne, K. O. Muennich, R. Boesinger, A. Dutzi, "
    "W. Huber and P. Libner (1987), J. Geophys. Res. 92(C2), 1937-1950, doi:10.1029/JC092iC02p01937 "
    "(abstract); restated by K. Nagel, K. E. Krall and B. Jaehne (2019), Ocean Sci. 15, 235-247, "
    "doi:10.5194/os-15-235-2019.",
    "Diffusivities in water, 25 C: E. L. Cussler, Diffusion: Mass Transfer in Fluid Systems, 2nd ed., "
    "Cambridge University Press 1997, ISBN 0-521-45078-0; values as quoted by Wikipedia "
    "'Mass diffusivity' (https://en.wikipedia.org/wiki/Mass_diffusivity, liquid table), read 2026-11-11.",
    "Henry constants: R. Sander (2015), Atmos. Chem. Phys. 15, 4399-4981, doi:10.5194/acp-15-4399-2015 "
    "(table already in properties.rs).",
    "Liquid-film admission: P. S. Liss (1973), Deep-Sea Res. 20, 221-238, "
    "doi:10.1016/0011-7471(73)90013-2 (abstract: O2 and CO2 exchange liquid-phase controlled).",
    "NOT sourced: a still-air gas-side velocity k_G (Hicks & Liss 1976, Tellus 28, 348-354, "
    "not readable). Gases more soluble than CO2 are therefore refused, not priced.",
    "NOT used: the 1.4e-6 m/s 'smooth-surface' figure attributed to Liss & Merlivat (1986); "
    "their smooth regime is 0.17*u10 cm/h, which is zero at no wind, so 1.4e-6 corresponds to "
    "u10 ~ 3 m/s and is not a still-air value.",
]


def table():
    src = SRC.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    body = src[src.index("pub const ESCAPE_TABLE"):src.index("];", src.index("pub const ESCAPE_TABLE"))]
    rows = []
    for m in re.finditer(r'formula: "([^"]+)",\s*diffusivity_m2_s: (None|Some\(([\d.e-]+)\))', body):
        rows.append((m.group(1), float(m.group(3)) if m.group(3) else None))
    d_co2 = float(re.search(r"pub const D_CO2_M2_S: f64 = ([\d.e-]+);", src).group(1))
    props = PROPS.read_text(encoding="utf-8")
    henry = {}
    for m in re.finditer(r'formula: "([^"]+)",\s*hcp_298: ([\d.e-]+),', props):
        henry[m.group(1)] = float(m.group(2))
    k_ref = float(re.search(r"pub const STILL_SURFACE_K_L_M_PER_S: f64 = ([\d.e-]+);",
                            CLOCK.read_text(encoding="utf-8")).group(1))
    return rows, d_co2, henry, k_ref


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    rows, d_co2, henry, k_ref = table()
    h_co2 = 1.0 / (henry["CO2"] * R * T)
    out = []
    for gas, d in rows:
        h = 1.0 / (henry[gas] * R * T) if gas in henry else None
        if d is None:
            cls, gap = "refused", "no sourced diffusivity in water"
        elif h is None:
            cls, gap = "refused", "no sourced Henry constant"
        elif h < h_co2 * (1 - 1e-12):
            cls, gap = "refused", "gas-side transfer velocity unsourced"
        else:
            cls, gap = "covered", ""
        kl = k_ref * (d / d_co2) ** (2 / 3) if cls == "covered" else None
        out.append(dict(gas=gas, cls=cls, diffusivity_m2_s=d,
                        henry_dimensionless=None if h is None else float(f"{h:.4g}"),
                        k_l_m_per_s=None if kl is None else float(f"{kl:.4g}"), gap=gap))
    counts = {"covered": sum(r["cls"] == "covered" for r in out),
              "refused": sum(r["cls"] == "refused" for r in out)}
    doc = dict(lane="open-vessel gas escape", date="2026-11-11", temperature_k=T,
               k_l_reference_m_per_s=k_ref, refusal_key="not-modeled.gas-escape-no-data",
               counts=counts, rows=out, sources=SOURCES)
    js = json.dumps(doc, indent=2, ensure_ascii=False) + "\n"
    md = ["# Open-vessel gas escape coverage (2026-11-11)", "",
          "Generated by `tools/open_vessel_gas_escape_coverage.py --check` from `gas_escape::ESCAPE_TABLE`.",
          "An open vessel releases only gas above its total pressure within a step (fizz); the rest seeps",
          "out on a `wait` at `K_L*A/V` (A = vessel mouth), `1/K_L = 1/k_L + 1/(H k_G)` taken in its",
          "liquid-film limit, `k_L = k_L,ref (D/D_CO2)^(2/3)`. Gases outside the cited liquid-film class or",
          "without data are refused with `not-modeled.gas-escape-no-data` (EN/DE/FR) on a `wait`.",
          "Today only CO2 is routed through the aqueous solver (`pending_co2_transfer_mol`); the other",
          "covered rows are priced by the law but no solver yet moves them.", "",
          f"Counts at {T} K: covered {counts['covered']}, refused {counts['refused']}.", "",
          "| gas | class | D (m²/s) | H (c_g/c_aq) | k_L (m/s) | gap |", "|---|---|---|---|---|---|"]
    for r in out:
        md.append(f"| {r['gas']} | {r['cls']} | {r['diffusivity_m2_s'] or '—'} | "
                  f"{r['henry_dimensionless'] if r['henry_dimensionless'] is not None else '—'} | "
                  f"{r['k_l_m_per_s'] or '—'} | {r['gap'] or '—'} |")
    md += ["", "## Sources", ""] + [f"- {s}" for s in SOURCES]
    text = "\n".join(md) + "\n"
    if a.check:
        bad = [p for p, t in ((MD, text), (JS, js)) if not p.exists() or p.read_text(encoding="utf-8") != t]
        for p in bad:
            print("stale:", p, file=sys.stderr)
        return 1 if bad else 0
    MD.write_text(text, encoding="utf-8")
    JS.write_text(js, encoding="utf-8")
    print(json.dumps(counts))
    return 0


if __name__ == "__main__":
    sys.exit(main())

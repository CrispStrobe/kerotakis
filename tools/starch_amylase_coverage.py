#!/usr/bin/env python3
"""Coverage of the starch / amylase / sugar family (2026-11-11, batch3-diverse:en-001).

Three rules, each read from the source it describes:

1. Enzyme time course. Every catalyst in `enzyme::FAMILIES` plus `amylase` is
   classified by the route that advances it over bench time:
     unit-definition-rate   `starch_amylase` (cited U definition + U/mg)
     bounded-activity       `enzyme_activity` (editorial envelope, fraction only)
     curated-kinetics       a `kinetics.rs` catalyst row with its own rate law
2. Starch-iodine optics: refused (`not-modeled.starch-iodine-absorptivity`),
   because the cited optical source states no reliable extinction coefficient
   exists; free I2 / I3- remain read from their cited epsilon values.
3. Sugar dissolution: points at `organic_solid_solubility` (maltose row added).

Writes docs/starch-amylase-coverage-20261111.{md,json}; `--check` fails when stale.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CORE = ROOT / "crates/kerotakis-core/src"
OUT = ROOT / "docs/starch-amylase-coverage-20261111"


def const(src: str, name: str) -> float:
    return float(re.search(rf"pub const {name}: f64 = ([\d._]+);", src).group(1).replace("_", ""))


STATES = {"Native": "native", "Gelatinised": "gelatinised", "Retrograded": "retrograded"}


def registry_starch_materials() -> list[tuple[str, float]]:
    doc = json.loads((ROOT / "data/registry/registry-source-v1.json").read_text(encoding="utf-8"))
    out: list[tuple[str, float]] = []

    def walk(o):
        if isinstance(o, dict):
            if "components" in o and "id" in o and isinstance(o["components"], list):
                for c in o["components"]:
                    if isinstance(c, dict) and c.get("species_id") == "starch":
                        out.append((o["id"], c["fraction"]["lower"]))
            for v in o.values():
                walk(v)
        elif isinstance(o, list):
            for v in o:
                walk(v)

    walk(doc)
    return out


def digestibility(amy: str) -> dict:
    consts = {m.group(1): m.group(2) for m in re.finditer(r'const (\w+): &str =\s*"([^"]+)";', amy)}
    rows = []
    for m in re.finditer(r"DigestibilityRow \{\s*botanical: \"([^\"]+)\",\s*state: StarchState::(\w+),\s*"
                         r"rds_pct: ([\d.]+),\s*digested_120_pct: ([\d.]+),\s*reported: \"([^\"]+)\",\s*"
                         r"source: (\w+),?\s*\}", amy):
        rds, d120, reported = float(m.group(3)), float(m.group(4)), m.group(5)
        parts = [p.strip() for p in reported.split("/")]
        lo = lambda x: float(x.split("-")[0])
        hi = lambda x: float(x.split("-")[-1])
        if abs(rds - lo(parts[0])) > 1e-9 or abs(d120 - round(100.0 - hi(parts[2]), 2)) > 1e-9:
            raise SystemExit(f"row {m.group(1)}/{m.group(2)} does not match its reported values")
        rows.append(dict(botanical=m.group(1), state=STATES[m.group(2)], rds_pct=rds, digested_120_pct=d120,
                         reported=reported, source=consts[m.group(6)]))
    mapping = dict(re.findall(r'\("([a-z/-]+)", (?:Some\("([a-z ]+)"\)|None)\)', amy))
    registry = registry_starch_materials()
    missing = [mid for mid, _ in registry if mid not in mapping]
    if missing:
        raise SystemExit(f"starch-bearing materials missing from MATERIAL_BOTANICAL: {missing}")
    materials = []
    for mid, frac in registry:
        b = mapping[mid] or None
        materials.append(dict(id=mid, starch_fraction=frac, botanical=b,
                              outcome=("table (" + ", ".join(r["state"] for r in rows if r["botanical"] == b) + ")")
                              if b else "refused: not-modeled.amylase-starch-fraction-no-data"))
    return dict(
        protocol=("protocol W: 4 U Sigma A3176 porcine pancreatic alpha-amylase + 4 U A. niger amyloglucosidase "
                  "(Megazyme E-AMGDF) per 10 mg starch, pH 6.0 phosphate, 37 C; RDS = digested by 20 min, "
                  "SDS = 20-120 min, RS = rest"),
        protocol_dose_u_per_g=const(amy, "PROTOCOL_DOSE_U_PER_G"),
        window=("37 C +/- 0.5 K and 0.5x-2x the protocol dose (A3176 at the >=5 U/mg lower specification); "
                "outside: not-modeled.amylase-starch-outside-source-protocol (no sourced dose or temperature law)"),
        time_course=("piecewise linear through (0,0), (20 min, RDS), (120 min, 100-RS), applied as a conditional "
                     "fraction of what is left; nothing claimed after 120 min "
                     "(not-modeled.amylase-starch-beyond-source-time); ranges use the lower digested end"),
        retrogradation=("gelatinised starch held in liquid water at 4 C for a cumulative 36 h (the cited "
                        "tables' treatment) is booked retrograded; >= 85 C re-gelatinises it"),
        caveat=("tabulated values include amyloglucosidase, which acts synergistically on granules; amylase alone "
                "is expected to digest no faster, so booked digestion is an upper-bound-like figure"),
        products=("maltose equivalents; alpha-amylase gives maltose, maltotriose (minor maltotetraose) and "
                  "alpha-limit dextrins (PLoS One 8 (2013) e59745, doi:10.1371/journal.pone.0059745); "
                  "split unsourced, typed note not-modeled.amylase-starch-product-split-unsourced"),
        iodine="the starch-iodine complex reads remaining ledger starch (starch_iodine::complex_moles)",
        rows=rows,
        materials=materials,
        counts=dict(botanical_sources=len({r["botanical"] for r in rows}), starch_materials=len(materials),
                    materials_with_row=sum(1 for m in materials if m["botanical"]),
                    materials_refused=sum(1 for m in materials if not m["botanical"])),
        not_sourced=("isolated potato, pea, wheat, rice, green-banana and sweet-potato starches have rows but no "
                     "registry material yet; native rice (figure only); whole foods (flour, dough, bread, pasta, "
                     "rice grain, potato tuber, beans, seeds, batter, popcorn, cocoa, baking powder): Englyst 1992 "
                     "(Eur J Clin Nutr 46 S33) and 1999 (AJCN 69:448) food tables not readable"),
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    amy = (CORE / "starch_amylase.rs").read_text(encoding="utf-8")
    enz = (CORE / "enzyme.rs").read_text(encoding="utf-8")
    kin = (CORE / "kinetics.rs").read_text(encoding="utf-8")
    curated = (CORE / "curated.rs").read_text(encoding="utf-8")
    ph = re.search(r"PH_RANGE: \(f64, f64\) = \(([\d.]+), ([\d.]+)\)", amy)
    source = re.search(r'pub const SOURCE: &str = "([^"]+)"', amy).group(1)
    activity = dict(
        specific_activity_u_per_mg=const(amy, "SPECIFIC_ACTIVITY_U_PER_MG"),
        unit_maltose_mg=const(amy, "UNIT_MALTOSE_MG"),
        unit_seconds=const(amy, "UNIT_SECONDS"),
        ph_range=[float(ph.group(1)), float(ph.group(2))],
        source=source,
        instant_curated_row_present='catalyst: Some("amylase")' in curated,
    )
    inact_src = re.search(r'pub const INACTIVATION_SOURCE: &str = "([^"]+)"', amy).group(1)
    heat = dict(
        model="first-order irreversible deactivation, k_d(T) = (beta/t_a) exp[(E_d/R)(1/T_opt - 1/T)]",
        t_opt_k=const(amy, "INACTIVATION_T_OPT_K"),
        e_d_j_per_mol=const(amy, "INACTIVATION_E_D_J_PER_MOL"),
        beta=const(amy, "INACTIVATION_BETA"),
        assay_minutes=const(amy, "INACTIVATION_ASSAY_MINUTES"),
        gas_constant=const(amy, "INACTIVATION_GAS_CONSTANT"),
        complete_loss_above_k=const(amy, "COMPLETE_LOSS_ABOVE_K"),
        applied="only above T_opt (falling limb); none claimed at or below T_opt",
        source=inact_src,
        corroboration=("Singh, Shandilya, Kundu, Kayastha, PLoS ONE 10 (2015) e0129203, "
                       "doi:10.1371/journal.pone.0129203, Fig 3A: WHEAT alpha-amylase unperturbed to 65 C, "
                       "4% loss at 70 C, complete loss around 85 C (different organism; not used as a constant)"),
        refusal="not-modeled.amylase-heat-inactivated",
        state="Vessel::amylase_inactive_moles (irreversible; not carried by a pour)",
        target="curiosity-v1 bio-030",
    )
    rows = [dict(enzyme="amylase", route="unit-definition-rate", module="starch_amylase",
                 refusals=["not-modeled.amylase-outside-unit-conditions",
                           "not-modeled.amylase-dose-not-convertible",
                           "not-modeled.amylase-heat-inactivated",
                           "not-modeled.amylase-starch-source-unspecified",
                           "not-modeled.amylase-starch-fraction-no-data",
                           "not-modeled.amylase-starch-outside-source-protocol",
                           "not-modeled.amylase-starch-beyond-source-time",
                           "not-modeled.amylase-starch-product-split-unsourced"])]
    blocks = re.findall(r"EnzymeProfile \{(.*?)\n    \}", enz, re.S)
    for b in blocks:
        sp = re.search(r'species: "([^"]+)"', b).group(1)
        sub = re.search(r"substrate: (Some\(SubstrateClass::\w+\)|None)", b).group(1)
        if sub == "None":
            kinetic = f'species: "{sp}"' in kin
            rows.append(dict(enzyme=sp, route="curated-kinetics" if kinetic else "none",
                             module="kinetics" if kinetic else "", refusals=[]))
        else:
            rows.append(dict(enzyme=sp, route="bounded-activity", module="enzyme_activity",
                             refusals=[]))
    native = dict(
        gelatinised_at_k=const(amy, "GELATINISED_AT_K"),
        gelatinisation_source=re.search(r'pub const GELATINISATION_SOURCE: &str = "([^"]+)"', amy).group(1),
        granule_rate_source=re.search(r'pub const NATIVE_GRANULE_SOURCE: &str = "([^"]+)"', amy).group(1),
        rule=("plain `starch` (no material recipe; generic laboratory starch of unstated plant source, "
              "insoluble in the registry) booked as gelatinised (held in liquid water at >= GELATINISED_AT_K "
              "during a clock step) is digested at the soluble-starch A3176 unit rate; plain native or "
              "retrograded starch is refused; starch from a material recipe follows the digestibility table"),
        refusal="not-modeled.amylase-starch-source-unspecified",
        state=("Vessel::starch_gelatinised_moles (re-gelatinised by >= 85 C) and Vessel::starch_digestion "
               "(retrograded pool, 4 C hold seconds, table clock, digested per lot source); not carried by a pour"),
        product=("booked as maltose equivalents (the unit counts reducing sugar as maltose); maltose, maltotriose "
                 "and alpha-limit dextrins split NOT modelled: no sourced split (typed note "
                 "not-modeled.amylase-starch-product-split-unsourced)"),
        target="batch3-diverse:en-001",
    )
    dig = digestibility(amy)
    optics = dict(
        observable="starch-iodine complex absorbance",
        status="refused",
        key="not-modeled.starch-iodine-absorptivity",
        reason=("Pesek et al., Molecules 27 (2022) 8974, doi:10.3390/molecules27248974, sect. 2.1: "
                "'reliable extinction coefficients for the amylose-iodine-iodide complex are not known'. "
                "The blue-value assay (Corn Refiners Association AMYLO.01, 620 nm) calibrates against "
                "amylose standards rather than an epsilon. No value was found that could be cited. "
                "Retried 2026-11-11: Pesek & Silaghi-Dumitrescu, Molecules 29 (2024) 641, "
                "doi:10.3390/molecules29030641 (full text) gives no coefficient; Knutson, Carbohydr. Polym. "
                "42 (2000) 65, doi:10.1016/S0144-8617(99)00126-5 and Abe, Bull. Chem. Soc. Jpn. 31 (1958) 661, "
                "doi:10.1246/bcsj.31.661 are paywalled (not read); open analytical papers report APPARENT "
                "absorptivities per iodide/iodine of 4e4 to 6e5 L/(mol cm) that depend on starch source, "
                "amylose fraction and excess-starch conditions, so none transfers to a generic 'starch' whose "
                "amylose fraction the bench does not know."),
        still_read="free I2 and I3- from their cited epsilon values (iodine_chemistry)",
    )
    data = dict(activity=activity, heat_inactivation=heat, native_granules=native, digestibility=dig,
                enzymes=rows, starch_iodine_optics=optics,
                sugars="see docs/organic-solid-dissolution-coverage-20261111.md (maltose row added)")
    md = ["# Starch / amylase / sugar coverage (2026-11-11)", "",
          "Generated by `tools/starch_amylase_coverage.py --check`. Target: batch3-diverse:en-001.", "",
          "## 1. Amylase digestion over bench time", "",
          f"Source: {source}. Read on the Sigma-Aldrich A3176 product page (mirror: Scientific Labs",
          "A3176-5MU), 2026-11-11: unit definition, specification '>=5 units/mg solid' (the page title",
          "says >=10; the lower bound is used), 'The pH range for activity is 5.5 to 8.0', 'Chloride ions",
          "are necessary for activity and stability' (disclosed, not gated).", "",
          f"Rate: maltose mg/s = enzyme mg x {activity['specific_activity_u_per_mg']} U/mg x "
          f"{activity['unit_maltose_mg']} mg / {activity['unit_seconds']} s, zero order in starch, capped by",
          "the starch present; 2 repeat units + H2O -> maltose. Inside the pH window no temperature or pH",
          "dependence is claimed. Outside it: typed refusal. Instant curated row present: "
          f"{activity['instant_curated_row_present']}.", "",
          "",
          "### Heat inactivation (bio-030)", "",
          f"Source: {heat['source']}.",
          f"Model: {heat['model']}; R = {heat['gas_constant']} J/(mol K). Applied {heat['applied']}.",
          f"Above {heat['complete_loss_above_k']} K (top of the fitted data) the enzyme is booked as completely",
          f"inactivated, not extrapolated. Corroboration: {heat['corroboration']}.",
          f"State: {heat['state']}. Typed reason: `{heat['refusal']}`.", "",
          "### Native (ungelatinised) starch granules (en-001)", "",
          f"Rule: {native['rule']}. Gelatinised at: {native['gelatinised_at_k']} K.",
          f"Gelatinisation source: {native['gelatinisation_source']}.",
          f"Why no granule rate: {native['granule_rate_source']}.",
          f"State: {native['state']}. Typed reason: `{native['refusal']}`.",
          f"Product: {native['product']}.", "",
          "| enzyme | route | module | typed refusals |", "|---|---|---|---|"]
    for r in rows:
        md.append(f"| {r['enzyme']} | {r['route']} | {r['module'] or '—'} | {', '.join(r['refusals']) or '—'} |")
    md += ["", "### Per-material, per-state digestible fractions (lane A, round 4)", "",
           f"Protocol: {dig['protocol']}.",
           f"Window: {dig['window']}. Time course: {dig['time_course']}.",
           f"Retrogradation: {dig['retrogradation']}.",
           f"Caveat: {dig['caveat']}.", f"Products: {dig['products']}.",
           f"Iodine: {dig['iodine']}.", "",
           f"Rows: {len(dig['rows'])} ({dig['counts']['botanical_sources']} botanical sources); "
           f"starch-bearing registry materials: {dig['counts']['starch_materials']}, "
           f"with a row: {dig['counts']['materials_with_row']}, refused typed: {dig['counts']['materials_refused']}.", "",
           "| botanical | state | RDS % (used) | digested by 120 min % (used) | reported RDS / SDS / RS | source |",
           "|---|---|---|---|---|---|"]
    for r in dig["rows"]:
        md.append(f"| {r['botanical']} | {r['state']} | {r['rds_pct']} | {r['digested_120_pct']} | {r['reported']} | {r['source']} |")
    md += ["", "| material (registry) | starch mass fraction | botanical row | outcome |", "|---|---|---|---|"]
    for m in dig["materials"]:
        md.append(f"| {m['id']} | {m['starch_fraction']} | {m['botanical'] or '—'} | {m['outcome']} |")
    md += ["", f"Not sourced (refused or no registry material): {dig['not_sourced']}."]
    md += ["", "## 2. Starch-iodine complex absorbance", "",
           f"Status: **{optics['status']}** (`{optics['key']}`). {optics['reason']}",
           f"Still read: {optics['still_read']}.", "",
           "## 3. Sugar dissolution", "", data["sugars"] + "."]
    text_md = "\n".join(md) + "\n"
    text_json = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
    paths = ((OUT.with_suffix(".md"), text_md), (OUT.with_suffix(".json"), text_json))
    if a.check:
        bad = [p for p, t in paths if not p.exists() or p.read_text(encoding="utf-8") != t]
        for p in bad:
            print("stale:", p, file=sys.stderr)
        return 1 if bad else 0
    for p, t in paths:
        p.write_text(t, encoding="utf-8")
    print(json.dumps({r["enzyme"]: r["route"] for r in rows}))
    return 0


if __name__ == "__main__":
    sys.exit(main())

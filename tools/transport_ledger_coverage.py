#!/usr/bin/env python3
"""Open-column ledger coverage of the `transport` verb (2026-11-11).

The default inlet is a constant-composition reservoir (the PHREEQC TRANSPORT
boundary-solution convention): it is never debited, so matter entering the
first cell comes from outside the bench. `Event::Transported` records it as
`inflow_moles` and names the `inlet`; every rendered transport line states it
as a source term (`event.transported.reservoir`) or, for a `drain` feed, as
what the feed lost (`event.transported.finite-feed`), in EN/DE/FR.

This tool lists every transport configuration, the ledger identity that holds
for it, and the test that pins it. `--check` fails when
  * a named test function no longer exists in its file,
  * a render key is missing from render.rs or from a shipped locale, or
  * docs/transport-ledger-coverage-20261111.{md,json} is stale.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT_MD = ROOT / "docs/transport-ledger-coverage-20261111.md"
OUT_JSON = ROOT / "docs/transport-ledger-coverage-20261111.json"
RENDER = ROOT / "crates/kerotakis-core/src/render.rs"
LOCALES = ["de", "fr"]
KEYS = [
    "transported.reservoir",
    "transported.finite-feed",
    "transported.inflow-none",
    "transported.inlet-unnamed",
]

CORE = "crates/kerotakis-core/tests"
NATIVE = "crates/kerotakis-phreeqc/tests"

# (configuration, inlet mode, pools in the sum, identity, closes, test file, test fn)
ROWS = [
    ("single reservoir inlet, inert column", "reservoir",
     "cells + receiver + reservoir", "after - inflow == before (per species)", "yes",
     f"{CORE}/transport_feed_ledger.rs",
     "the_default_reservoir_is_not_drained_and_its_inflow_is_recorded"),
    ("two reservoir inlets in turn (tr-006) + charcoal bed", "reservoir",
     "cells + receiver + both reservoirs + adsorbed surface pool",
     "after - sum(inflow) == before (water, dye, NaCl, carbon)", "yes",
     f"{CORE}/transport_reservoir_source_term.rs",
     "tr006_two_reservoir_ledger_closes_with_the_surface_pool"),
    ("tr-006 original, native PHREEQC stack, both commands", "reservoir",
     "ConservedLedger: contents + adsorbed + surfaces + exchanges + solid solutions",
     "after == before + inflow (N, S, Na, Cl per command)", "yes",
     f"{NATIVE}/transport_feed_triggers.rs",
     "tr006_original_closes_dye_and_bound_pool_elements_per_command"),
    ("clean-water reservoir desorbing a loaded bed", "reservoir",
     "cells + receiver + adsorbed surface pool", "after == before + inflow (dye; inflow 0)", "yes",
     f"{CORE}/transport_surface_reequilibration.rs",
     "a_clean_water_flush_desorbs_toward_the_isotherm"),
    ("dye front onto a fresh bed", "reservoir",
     "cells + receiver + reservoir + adsorbed", "after == before + inflow (dye)", "yes",
     f"{CORE}/transport_surface_reequilibration.rs",
     "a_dye_front_is_adsorbed_as_it_arrives"),
    ("reactive bed, reaction after every step (toy cementation)", "reservoir",
     "cells + receiver + deposit", "ion + deposit and bed + deposit balances", "yes",
     f"{CORE}/transport_feed_ledger.rs",
     "reactions_run_after_every_step_so_an_excess_bed_retains_the_ion"),
    ("tr-004 silver column, native", "reservoir",
     "ConservedLedger (all pools)", "after == before + inflow (Ag, Na, Cl, N)", "yes",
     f"{NATIVE}/transport_feed_triggers.rs",
     "tr004_original_records_reservoir_inflow_and_keeps_silver"),
    ("tr-008 iron bed retaining copper, native", "reservoir",
     "ConservedLedger (all pools)", "after == before + inflow (Fe, Cu, S)", "yes",
     f"{NATIVE}/transport_feed_triggers.rs",
     "tr008_original_excess_iron_bed_retains_copper_per_step"),
    ("single drained feed", "drain",
     "feed + cells + receiver", "after == before; inflow == feed loss", "yes",
     f"{CORE}/transport_feed_ledger.rs",
     "finite_feed_conserves_every_species_across_feed_cells_and_effluent"),
    ("two drained feeds in turn + charcoal bed", "drain",
     "feeds + cells + receiver + adsorbed", "after == before; inflow == feed loss", "yes",
     f"{CORE}/transport_reservoir_source_term.rs",
     "tr006_drained_feeds_close_without_a_source_term"),
    ("tr-006 drained, native", "drain",
     "ConservedLedger (all pools)", "after == before (Na, Cl)", "yes",
     f"{NATIVE}/transport_feed_triggers.rs", "tr006_drained_feed_counts_salt_once"),
    ("drained feed asked for more than it holds", "drain",
     "bench unchanged", "typed InletExhausted refusal, atomic", "refused (no partial move)",
     f"{CORE}/transport_feed_ledger.rs", "an_exhausted_feed_is_a_typed_atomic_refusal"),
    ("Courant 0", "reservoir",
     "bench unchanged", "inflow empty; line says the reservoir supplied nothing", "yes (trivially)",
     f"{CORE}/transport_reservoir_source_term.rs",
     "a_zero_courant_reservoir_reports_that_it_supplied_nothing"),
    ("source term stated at LV1/LV2/LV3 in EN/DE/FR", "both",
     "rendered line", "reservoir/finite-feed label + per-species inflow", "n/a (report)",
     f"{CORE}/transport_reservoir_source_term.rs",
     "every_register_and_locale_states_the_reservoir_source_term"),
    ("event payload names inlet; older payloads still read", "both",
     "event JSON", "`inlet`, `inflow_moles`, `drain`", "n/a (report)",
     f"{CORE}/transport_reservoir_source_term.rs",
     "the_event_payload_carries_inlet_and_inflow"),
]

OPEN = [
    ("open-headspace gas exchange during transport (e.g. CO2 with air)",
     "a second open boundary that `inflow_moles` does not record; carbon and H/O are "
     "therefore not asserted in the native closure tests"),
    ("free H+/OH- and solute-charge carried by the parcel",
     "moved with the parcel but not listed in `inflow_moles` (not species amounts)"),
    ("PHREEQC surface-complexation / exchange beds in a transport cell",
     "counted by `ConservedLedger`; no dedicated transport closure test yet beyond the "
     "native trigger scripts above"),
]


def problems() -> list[str]:
    errs = []
    for row in ROWS:
        path, fn = ROOT / row[5], row[6]
        if not path.exists() or not re.search(rf"\bfn {re.escape(fn)}\(", path.read_text("utf-8")):
            errs.append(f"missing test {row[5]}::{fn}")
    render = RENDER.read_text("utf-8")
    for key in KEYS:
        if f'"event.{key}"' not in render:
            errs.append(f"render.rs does not use event.{key}")
        for lang in LOCALES:
            data = tomllib.loads((ROOT / f"crates/kerotakis-core/i18n/{lang}.toml").read_text("utf-8"))
            if key not in data.get("event", {}):
                errs.append(f"{lang}.toml lacks event.{key}")
    return errs


def build() -> tuple[str, str]:
    md = [
        "# Transport ledger coverage (2026-11-11)", "",
        "Generated by `tools/transport_ledger_coverage.py` (`--check` in CI preflight).", "",
        "The default `transport` inlet is a **constant-composition reservoir**: it is not debited,",
        "following the boundary-solution convention of PHREEQC's TRANSPORT keyword (Parkhurst, D.L.,",
        "and Appelo, C.A.J., 2013, *Description of input and examples for PHREEQC version 3*, U.S.",
        "Geological Survey Techniques and Methods, book 6, chap. A43, doi:10.3133/tm6A43). What it",
        "supplied is the source term `inflow_moles` on the `transported` event, which now also names",
        "the `inlet`; every rendered transport line (all registers, EN/DE/FR) states it per species.",
        "`drain` opts into a finite feed whose loss equals the stated inflow.", "",
        "Closure for a reservoir run: **sum over every vessel of (contents + adsorbed surface pool)",
        "- sum of `inflow_moles` over the commands == the initial sum**. No physical constant is",
        "introduced by this lane.", "",
        "| configuration | inlet | pools summed | identity | ledger closes | test |",
        "|---|---|---|---|---|---|",
    ]
    for cfg, mode, pools, ident, closes, path, fn in ROWS:
        md.append(f"| {cfg} | {mode} | {pools} | {ident} | {closes} | `{Path(path).stem}::{fn}` |")
    md += ["", "## Not closed by the transport source term", "",
           "| term | status |", "|---|---|"]
    md += [f"| {t} | {s} |" for t, s in OPEN]
    data = {
        "date": "2026-11-11",
        "render_keys": [f"event.{k}" for k in KEYS],
        "configurations": [
            dict(configuration=c, inlet=m, pools=p, identity=i, closes=cl, test=f"{path}::{fn}")
            for c, m, p, i, cl, path, fn in ROWS
        ],
        "open_terms": [dict(term=t, status=s) for t, s in OPEN],
    }
    return "\n".join(md) + "\n", json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    errs = problems()
    for e in errs:
        print(e, file=sys.stderr)
    md, js = build()
    if args.check:
        stale = [p for p, t in ((OUT_MD, md), (OUT_JSON, js))
                 if not p.exists() or p.read_text("utf-8") != t]
        for p in stale:
            print("stale:", p, file=sys.stderr)
        return 1 if errs or stale else 0
    OUT_MD.write_text(md, "utf-8")
    OUT_JSON.write_text(js, "utf-8")
    print(f"{len(ROWS)} configurations, {len(errs)} problems")
    return 1 if errs else 0


if __name__ == "__main__":
    sys.exit(main())

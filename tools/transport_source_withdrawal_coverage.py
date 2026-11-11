#!/usr/bin/env python3
"""Source-withdrawal coverage of every inter-vessel operator (2026-11-11).

TR-006 found a `transport` inlet that supplied matter to the column and kept
it: the script default was an undebited constant-composition reservoir. The
fix makes a bench inlet a finite feed by default (`reservoir` must now be
declared). This tool keeps the family honest:

* it reads `pub enum Operator` in crates/kerotakis-core/src/ops.rs and lists
  every variant that names two or more vessels / destinations (an
  inter-vessel operator);
* every such operator must have a row below saying how its source is
  debited, and the row's closure test must exist in the named test file;
* the script parser's transport stanza must keep the finite feed as default.

Writes docs/transport-source-withdrawal-coverage-20261111.{md,json};
`--check` fails when an operator has no row, a test is missing, the parser
default regressed, or the checked-in docs are stale.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OPS = ROOT / "crates/kerotakis-core/src/ops.rs"
SCRIPT = ROOT / "crates/kerotakis-core/src/script.rs"
MD = ROOT / "docs/transport-source-withdrawal-coverage-20261111.md"
JS = ROOT / "docs/transport-source-withdrawal-coverage-20261111.json"

VESSEL_FIELDS = {"from", "to", "a", "b", "into", "inlet", "receiver", "chain",
                 "destination", "destination_if_broken", "vessel"}

CORE = "crates/kerotakis-core/tests/"
NEW = CORE + "transport_source_withdrawal.rs"
TRIG = "crates/kerotakis-phreeqc/tests/transport_feed_triggers.rs"

# operator -> (status, how the source is debited, [(test file, test fn)])
ROWS = {
    "Transport": ("debited", "finite feed by default: inlet debited by exactly the injected parcel "
                  "(`advance_from_feed`); an explicit `reservoir` is an external boundary whose "
                  "supply the event reports as `inflow_moles` (source term)",
                  [(NEW, "transport_default_debits_inlet_by_exactly_the_destination_credit"),
                   (NEW, "declared_reservoir_keeps_inlet_and_reports_exact_source_term"),
                   (NEW, "exhausted_feed_is_a_typed_localized_refusal_and_moves_nothing"),
                   (NEW, "transport_script_default_is_a_finite_feed_and_reservoir_is_explicit"),
                   (TRIG, "tr006_verbatim_inlet_is_debited_by_what_entered_the_column"),
                   (TRIG, "tr006_declared_reservoir_is_not_debited_and_states_its_source_term")]),
    "Decant": ("debited", "liquid fraction withdrawn from `from`, deposited in `to`",
               [(NEW, "decant_debits_source_by_exactly_the_destination_credit")]),
    "Filter": ("debited", "mobile phase withdrawn from `from`, solids retained",
               [(NEW, "filter_debits_source_by_exactly_the_filtrate_credit")]),
    "Magnet": ("debited", "magnetic solids removed from `from`, deposited in `to`",
               [(NEW, "magnet_debits_source_by_exactly_the_attracted_credit")]),
    "Mix": ("debited", "each source keeps (1 - fraction) of its liquid",
            [(NEW, "mix_debits_both_sources_by_exactly_the_receiver_credit")]),
    "Drain": ("debited", "lower layer withdrawn from `from`",
              [(CORE + "phase_owned_transfers.rs", "drain_ordinary_aqueous_first_keeps_both_inventories"),
               (CORE + "liquid_partition_refusals.rs", "refused_drain_creates_no_receiver_and_moves_no_material")]),
    "Distil": ("debited", "distillate withdrawn from the pot",
               [(CORE + "distil.rs", "distillation_conserves_matter")]),
    "Extract": ("debited+stated-addition", "solute withdrawn from the aqueous source; the fresh "
                "extracting solvent is an explicit stated addition (`total_solvent`)",
                [(CORE + "liquid_extraction.rs", "repeated_fresh_portions_extract_more_and_conserve_every_species")]),
    "Spill": ("debited", "fraction withdrawn from `from` into the spill surface",
              [(CORE + "spill_breakage.rs", "partial_spill_is_frame_policy_independent_and_closes_ledgers")]),
    "Impact": ("debited", "a broken vessel's contents move once to the destination",
               [(CORE + "spill_breakage.rs", "breaking_moves_contents_once_and_recovery_cannot_duplicate_stock")]),
    "RecoverSpill": ("debited", "fraction withdrawn from the spill surface",
                     [(CORE + "spill_breakage.rs", "breaking_moves_contents_once_and_recovery_cannot_duplicate_stock")]),
    "Cell": ("no-transfer", "open-circuit voltmeter: nothing moves", []),
}


def operators():
    src = OPS.read_text(encoding="utf-8")
    start = src.index("pub enum Operator")
    body = src[start:src.index("\n}\n", start)]
    found = {}
    for m in re.finditer(r"\n    ([A-Z][A-Za-z]+)\s*(\{(.*?)\}|,)", body, re.S):
        fields = set(re.findall(r"\b([a-z_]+)\s*:", m.group(3) or ""))
        found[m.group(1)] = sorted(fields & VESSEL_FIELDS)
    return {k: v for k, v in found.items() if len(v) >= 2}


def parser_default_is_finite():
    src = SCRIPT.read_text(encoding="utf-8")
    return ('[rest @ .., "reservoir"] => (rest, false)' in src
            and "rest => (rest, true)," in src)


def fn_exists(path, name):
    p = ROOT / path
    return p.exists() and re.search(r"\bfn " + re.escape(name) + r"\s*\(", p.read_text(encoding="utf-8"))


def build():
    ops = operators()
    problems = []
    if not parser_default_is_finite():
        problems.append("script transport default is no longer a finite feed")
    rows = []
    for name, fields in sorted(ops.items()):
        if name not in ROWS:
            problems.append(f"inter-vessel operator {name} {fields} has no source-withdrawal row")
            continue
        status, how, tests = ROWS[name]
        if status != "no-transfer" and not tests:
            problems.append(f"{name}: no closure test")
        for path, fn in tests:
            if not fn_exists(path, fn):
                problems.append(f"{name}: test {path}::{fn} not found")
        rows.append(dict(operator=name, vessel_fields=fields, status=status, debit=how,
                         tests=[f"{p}::{f}" for p, f in tests]))
    for name in ROWS:
        if name not in ops:
            problems.append(f"row {name} names no inter-vessel operator")
    return rows, problems


def render(rows):
    doc = {
        "date": "2026-11-11",
        "defect": "batch3-diverse:tr-006 - transport inlet supplied matter without being debited",
        "rule": "for every inter-vessel operator, source debit == destination credit per element "
                "and charge; an external boundary must be declared and reported as a source term",
        "boundary_reference": "Parkhurst, D.L., Appelo, C.A.J. (2013) Description of input and "
                              "examples for PHREEQC version 3, USGS Techniques and Methods 6-A43, "
                              "doi:10.3133/tm6A43 (TRANSPORT keyword: boundary solution 0 is a "
                              "constant-composition external reservoir)",
        "constants": "none introduced",
        "operators": rows,
    }
    md = ["# Source-withdrawal coverage of inter-vessel operators (2026-11-11)", "",
          "Generated by `tools/transport_source_withdrawal_coverage.py`; `--check` holds it.", "",
          "Defect: batch3-diverse:tr-006. The `transport` inlet was by default an undebited",
          "constant-composition reservoir, so a vessel on the bench supplied the column and",
          "kept its contents (bench totals Cl +100 %, Na +98 %, dye +100 %, water +4.43 mol).",
          "Fix: a bench inlet is a finite feed by default and is debited by exactly what",
          "enters the column; an external boundary must be declared with `reservoir`, and",
          "its supply is reported as a source term on every event. A feed that cannot",
          "supply a step is refused as a whole with `error.transport-feed-exhausted`",
          "(EN/DE/FR).", "",
          "Boundary semantics follow PHREEQC TRANSPORT (Parkhurst & Appelo 2013, USGS",
          "Techniques and Methods 6-A43, doi:10.3133/tm6A43). No physical constant is",
          "introduced.", "",
          "| operator | vessel fields | status | how the source is debited | closure tests |",
          "|---|---|---|---|---|"]
    for r in rows:
        tests = "<br>".join(f"`{t.split('/')[-1]}`" for t in r["tests"]) or "—"
        md.append(f"| {r['operator']} | {', '.join(r['vessel_fields'])} | {r['status']} | {r['debit']} | {tests} |")
    md.append("")
    md.append("Out of scope here: the serialised `Operator::Transport` payload keeps `drain` "
              "absent = reservoir so logged replays are unchanged; only the script surface "
              "changed its default.")
    md.append("")
    return "\n".join(md), json.dumps(doc, indent=2, ensure_ascii=False) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args()
    rows, problems = build()
    md, js = render(rows)
    if a.check:
        if MD.read_text(encoding="utf-8") != md or JS.read_text(encoding="utf-8") != js:
            problems.append("docs are stale: run tools/transport_source_withdrawal_coverage.py")
        for p in problems:
            print("FAIL:", p, file=sys.stderr)
        if problems:
            return 1
        print(f"ok: {len(rows)} inter-vessel operators covered")
        return 0
    MD.write_text(md, encoding="utf-8")
    JS.write_text(js, encoding="utf-8")
    for p in problems:
        print("FAIL:", p, file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

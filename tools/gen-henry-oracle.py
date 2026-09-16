#!/usr/bin/env python3
"""Tier C: extract gas-dissolution constants from the PHREEQC databases this
repository already ships, so the Henry's-law table can be checked against a
compilation it did not come from.

`crates/kerotakis-core/src/properties.rs` carries six Henry coefficients from
Sander (2015), an atmospheric-chemistry compilation. The aqueous solver runs
on the USGS PHREEQC databases, a different lineage (WATEQ / Plummer /
Parkhurst), and several of them tabulate the same gas dissolutions. So this is
not only an external oracle; it is a consistency check on a quantity the
bench has TWO answers for — `aqueous.rs` asks `henry_lookup("CO2")` for a
number while running a speciation on a database that states its own.

`provenance/upstreams.toml` clears `phreeqc-databases` as `primary`, may_touch
`runtime-data, property, thermochemistry`, under the USGS User Rights Notice.
The bytes are already vendored; this adds no source and no dependency — the
script is standard library only.

WHAT IT REFUSES TO EXTRACT, and why that matters more than what it takes.
Only a reaction written `X = X` — the gas dissolving unchanged — has a log_k
that IS a Henry constant. `llnl.dat` writes CO2(g) as `CO2 + H2O = H+ +
HCO3-` and `minteq.v4.dat` as `CO2 + H2O = 2 H+ + CO3-2`; those log_k values
fold in carbonic-acid dissociation and comparing them to an Hcp would be
wrong by orders of magnitude and would look like a finding. The parser takes
the reaction apart and skips anything that is not the identity, recording the
skip so the absence is visible rather than silent.

  usage: python3 tools/gen-henry-oracle.py \
             > crates/kerotakis-core/tests/fixtures/henry_databases.tsv

Emitted columns:
  gas  database  log_k  delta_h_j_per_mol  sha256_12
`delta_h_j_per_mol` is `na` where the database gives only an analytic
expression; the analytic form is deliberately NOT differentiated here,
because its own validity range is not stated in the file and a derivative
taken at one temperature would be a number this script invented.
"""

import hashlib
import os
import re
import sys

DB_DIR = os.path.join("vendor", "iphreeqc", "database")

# The databases the engine actually loads (`crates/kerotakis-phreeqc/src`).
# A database nothing loads would make this a check against a file nobody runs.
DATABASES = ["phreeqc.dat", "wateq4f.dat", "pitzer.dat", "llnl.dat", "minteq.v4.dat"]

# The gases `properties::HENRY_COEFFICIENTS` carries. Kept by hand: a gas
# added there without a line here fails the reverse check in
# `crates/kerotakis-core/tests/henry_oracle.rs`, which is the point.
GASES = {"CO2": "CO2(g)", "O2": "O2(g)", "N2": "N2(g)", "H2": "H2(g)", "Cl2": "Cl2(g)", "NH3": "NH3(g)"}

KCAL_J = 4184.0
KJ_J = 1000.0

BLOCK_KEYWORDS = {
    "SOLUTION_MASTER_SPECIES", "SOLUTION_SPECIES", "EXCHANGE_MASTER_SPECIES",
    "EXCHANGE_SPECIES", "SURFACE_MASTER_SPECIES", "SURFACE_SPECIES", "RATES",
    "END", "PITZER", "SIT", "LLNL_AQUEOUS_MODEL_PARAMETERS", "NAMED_EXPRESSIONS",
    "ISOTOPES", "ISOTOPE_RATIOS", "ISOTOPE_ALPHAS", "CALCULATE_VALUES",
}


def phases_block(lines):
    start = None
    for i, line in enumerate(lines):
        if line.strip().upper().split()[:1] == ["PHASES"]:
            start = i
            break
    if start is None:
        return []
    end = len(lines)
    for i in range(start + 1, len(lines)):
        head = lines[i].strip().upper().split()[:1]
        if head and head[0] in BLOCK_KEYWORDS and not lines[i][:1].isspace():
            end = i
            break
    return lines[start + 1:end]


def is_identity(reaction):
    """True only for `X = X` — the gas dissolving unchanged."""
    if "=" not in reaction:
        return False
    left, right = reaction.split("=", 1)
    return left.strip() == right.strip() and left.strip() != ""


def parse(path):
    with open(path, encoding="latin-1") as handle:
        lines = handle.read().splitlines()
    out = {}
    block = phases_block(lines)
    i = 0
    while i < len(block):
        line = block[i]
        name = line.strip()
        if line[:1] not in ("", " ", "\t") and name in GASES.values():
            i += 1
            reaction, log_k, delta_h = None, None, None
            while i < len(block) and (block[i][:1] in (" ", "\t") or not block[i].strip()):
                body = block[i].split("#", 1)[0].strip()
                i += 1
                if not body:
                    continue
                for part in body.split(";"):
                    part = part.strip()
                    if not part:
                        continue
                    low = part.lower()
                    if low.startswith(("-log_k", "log_k")):
                        log_k = float(part.split()[1])
                    elif low.startswith(("-delta_h", "delta_h", "-delta_H".lower())):
                        bits = part.split()
                        value = float(bits[1])
                        # AN ASSUMPTION, AND IT IS LOAD-BEARING ON ONE ROW.
                        # PHREEQC's `-delta_h` takes an optional unit and
                        # defaults to kJ/mol when it is omitted. Every row
                        # here states its unit EXCEPT phreeqc.dat's H2(g)
                        # (`-delta_h -3.3`), and that one row is the pinned
                        # hydrogen disagreement: read as kJ it implies 397 K,
                        # read as kcal it implies 1661 K, against the 500 K
                        # this repository ships. So the disagreement's SIZE
                        # depends on a default nobody here has checked
                        # against the PHREEQC manual. Recorded rather than
                        # asserted; it is the first thing to verify if that
                        # row is ever acted on.
                        unit = bits[2].lower() if len(bits) > 2 else "kj"
                        if unit.startswith("kcal"):
                            delta_h = value * KCAL_J
                        elif unit.startswith("cal"):
                            delta_h = value
                        else:
                            delta_h = value * KJ_J
                    elif reaction is None and "=" in part and not part.startswith("-"):
                        reaction = part
            out[name] = (reaction, log_k, delta_h)
            continue
        i += 1
    return out


def main():
    print("# generated by tools/gen-henry-oracle.py — do not edit")
    print("# gas-dissolution constants read out of the PHREEQC databases this")
    print("# repository vendors, for comparison against the Sander (2015)")
    print("# coefficients in kerotakis-core/src/properties.rs. Independent OF")
    print("# Sander's compilation and of our arithmetic; NOT independent of the")
    print("# primary measurements the two compilations may share. Only `X = X`")
    print("# reactions are taken — see the generator for why.")
    print("# columns: gas database log_k delta_h_j_per_mol sha256_12")
    skipped = []
    for db in DATABASES:
        path = os.path.join(DB_DIR, db)
        if not os.path.exists(path):
            print(f"missing database {path}", file=sys.stderr)
            return 1
        digest = hashlib.sha256(open(path, "rb").read()).hexdigest()[:12]
        found = parse(path)
        for gas, phase in sorted(GASES.items()):
            if phase not in found:
                continue
            reaction, log_k, delta_h = found[phase]
            if reaction is None or not is_identity(reaction):
                skipped.append(f"{gas}\t{db}\t{reaction}")
                continue
            if log_k is None:
                skipped.append(f"{gas}\t{db}\tno log_k")
                continue
            dh = "na" if delta_h is None else f"{delta_h:.4f}"
            print(f"{gas}\t{db}\t{log_k!r}\t{dh}\t{digest}")
    for note in skipped:
        print(f"# skipped\t{note}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

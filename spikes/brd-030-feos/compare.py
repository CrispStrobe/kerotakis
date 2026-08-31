#!/usr/bin/env python3
"""BRD-030 — join the three engines and classify every disagreement.

Reads `fixtures/engines.tsv` (kerotakis-thermo, feos, and the adapter, all
emitted by the Rust binary) and `fixtures/oracle.tsv` (the Python `thermo`
referee), joins them on (case, quantity), and gives every row a class.

The classification is the deliverable. BRD-030's definition of done forbids
averaging a discrepancy away, so each row is assigned exactly one of:

  agree                  every engine that answered agrees inside tolerance
  coverage-gap           one engine has no model for this fluid at all —
                         the largest class, and the whole argument
  our-bug                kerotakis differs from a referee running the SAME
                         model with the SAME parameters. Nothing but an
                         implementation difference can cause this.
  model-difference       kerotakis and feos differ, and each agrees with a
                         referee of its own model family. Inherent, not a bug.
  parameter-difference   two published parameter sets, same feos code, same
                         model family, different answers.
  feos-difference        feos differs from the independent referee.
  single-phase-refusal   feos's stability analysis says the feed is one
                         phase and raises; kerotakis returns beta = 0 or 1.
                         A semantics difference in the API, not a wrong number.
  range-refusal          kerotakis has the model but declines to extrapolate
                         outside its fitted Antoine range. An honest refusal.
  solver-failure         an engine that HAS the model failed to converge.
  oracle-limitation      the referee could not answer, or answered from a
                         source this project cannot rely on (DIPPR/Perry).

Writes fixtures/discrepancies.tsv and prints a Markdown summary.

Run:  python3 compare.py
"""

import collections
import math
import re
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(HERE, "fixtures")

# Tolerances, chosen per quantity and stated rather than tuned until the
# table looked good. Relative unless the name says absolute.
TOL = {
    "psat_kpa": ("rel", 0.05),
    "rho_liq_mol_m3": ("rel", 0.03),
    "dhvap_kj_mol": ("rel", 0.05),
    "tboil_c": ("abs", 1.0),
    "tcrit_k": ("rel", 0.02),
    "pcrit_kpa": ("rel", 0.05),
    "tbub_c": ("abs", 1.0),
    "y1": ("abs", 0.02),
    "vapour_fraction": ("abs", 0.02),
    # Peng-Robinson, three implementations of one equation fed byte-identical
    # Tc/Pc/omega. This started at 1e-6 on the theory that they should agree to
    # solver tolerance. The measurement corrected the theory rather than the
    # other way round: kerotakis-thermo and feos agree to 4.7e-11 — they are
    # the same equation, twice, and both right — while `thermo`'s cubic root
    # solve is only converged to ~4e-4, which flagged 21 rows as
    # disagreements that were nothing but the referee's own tolerance.
    # 1e-3 is loose enough to stop measuring `thermo`'s solver and tight
    # enough that a real root-selection bug could not hide under it.
    "z_vapour": ("rel", 1e-3),
    "molar_volume_m3_mol": ("rel", 1e-3),
    "phi": ("rel", 1e-3),
}

# A referee point drawn from Perry's/DIPPR correlations cannot be used as
# evidence in this project (BRD-031 forbids the source), so a row that would
# have been judged against one is called out rather than counted as a pass.
ENCUMBERED = "DIPPR_PERRY_8E"


def canon(case):
    """Normalise a case key across the two emitters.

    Rust's `{}` for an `f64` prints `25`, Python's prints `25.0`, so
    `acetone@25C` and `acetone@25.0C` are the same case written two ways.
    Rather than make either side format defensively, every embedded number is
    reduced to a canonical form here — the join key is derived, not trusted.
    """
    def fix(m):
        v = float(m.group(0))
        return f"{v:g}"

    return re.sub(r"-?\d+\.?\d*(?:[eE][-+]?\d+)?", fix, case)


def load(path):
    rows = {}
    with open(path) as fh:
        for line in fh:
            if not line.strip() or line.startswith("#"):
                continue
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 6 or parts[0] == "engine":
                continue
            engine, case, quantity, status, value, note = parts[:6]
            v = float(value) if status == "ok" and value else None
            rows[(engine, canon(case), quantity)] = (status, v, note)
    return rows


def close(q, a, b):
    kind, tol = TOL.get(q, ("rel", 0.05))
    if a is None or b is None:
        return None
    if kind == "abs":
        return abs(a - b) <= tol
    denom = max(abs(a), abs(b), 1e-30)
    return abs(a - b) / denom <= tol


def dev(q, a, b):
    if a is None or b is None:
        return None
    kind, _ = TOL.get(q, ("rel", 0.05))
    if kind == "abs":
        return abs(a - b)
    return abs(a - b) / max(abs(a), abs(b), 1e-30)


def main():
    eng = load(os.path.join(FIX, "engines.tsv"))
    orc = load(os.path.join(FIX, "oracle.tsv"))
    rows = {**eng, **orc}

    keys = sorted({(c, q) for (_, c, q) in rows})
    out = []
    counts = collections.Counter()

    for case, quantity in keys:
        def g(engine):
            return rows.get((engine, case, quantity), (None, None, ""))

        k_st, k_v, k_note = g("kerotakis")
        kpr_st, kpr_v, kpr_note = g("kerotakis-pr")
        f_st, f_v, f_note = g("feos-pcsaft")
        fpr_st, fpr_v, fpr_note = g("feos-pr")
        f2_st, f2_v, _ = g("feos-pcsaft-gross2002")
        ad_st, ad_v, _ = g("adapter")
        oc_st, oc_v, oc_note = g("oracle-corr")
        ok_st, ok_v, _ = g("oracle-unifac-kero")
        ou_st, ou_v, _ = g("oracle-unifac-corr")
        op_st, op_v, _ = g("oracle-pr")

        # Which pair of engines is this row about?
        if kpr_st or fpr_st:
            ours, ours_st, ours_note = kpr_v, kpr_st, kpr_note
            theirs, theirs_st = fpr_v, fpr_st
            ref, ref_st = op_v, op_st
            ref_note = "thermo.eos.PR"
            family = "peng-robinson"
        else:
            ours, ours_st, ours_note = k_v, k_st, k_note
            theirs, theirs_st = f_v, f_st
            family = "vle"
            # For binaries the like-model referee is the UNIFAC one; for
            # pure fluids it is thermo's own correlations.
            if ok_st or ou_st:
                ref, ref_st, ref_note = (
                    (ok_v, ok_st, "oracle-unifac-kero")
                    if ok_st == "ok"
                    else (ou_v, ou_st, "oracle-unifac-corr")
                )
            else:
                ref, ref_st, ref_note = oc_v, oc_st, oc_note

        cls = classify(
            quantity, ours, ours_st, ours_note, theirs, theirs_st,
            ref, ref_st, ref_note, f_v, f2_v, ou_v, ou_st,
            f_note if family == "vle" else fpr_note,
        )
        counts[cls] += 1
        out.append(
            (case, quantity, family, cls,
             fmt(ours), ours_st or "-",
             fmt(theirs), theirs_st or "-",
             fmt(ref), ref_st or "-", ref_note or "",
             fmt(f2_v), fmt(ad_v),
             fmt(dev(quantity, ours, theirs)),
             fmt(dev(quantity, theirs, ref)),
             fmt(dev(quantity, ours, ref)))
        )

    path = os.path.join(FIX, "discrepancies.tsv")
    with open(path, "w") as fh:
        fh.write("# BRD-030 three-way comparison. Generated by compare.py; do not edit.\n")
        fh.write("\t".join([
            "case", "quantity", "family", "class",
            "kerotakis", "kerotakis_status",
            "feos", "feos_status",
            "referee", "referee_status", "referee_source",
            "feos_gross2002", "adapter",
            "dev_kero_vs_feos", "dev_feos_vs_referee", "dev_kero_vs_referee",
        ]) + "\n")
        for r in out:
            fh.write("\t".join([
                r[0], r[1], r[2], r[3],
                r[4], r[5], r[6], r[7], r[8], r[9], r[10],
                r[11], r[12], r[13], r[14], r[15],
            ]) + "\n")

    print(f"# {len(out)} comparison rows -> {path}\n")
    print("| class | rows |")
    print("|---|---:|")
    for cls, n in counts.most_common():
        print(f"| `{cls}` | {n} |")
    print(f"| **total** | **{len(out)}** |")

    # The coverage gap is by far the largest class and listing it row by row
    # would bury everything else, so it is summarised by (quantity, reason)
    # and the interesting classes are listed in full.
    print("\n## `coverage-gap` — what only one engine can answer\n")
    gap = collections.Counter()
    for r in out:
        if r[3] != "coverage-gap":
            continue
        who = "feos only" if r[7] == "ok" else ("kerotakis only" if r[5] == "ok" else "neither")
        gap[(r[1], who)] += 1
    print("| quantity | answered by | rows |")
    print("|---|---|---:|")
    for (q, who), n in sorted(gap.items(), key=lambda kv: (-kv[1], kv[0])):
        print(f"| {q} | {who} | {n} |")

    interesting = ("our-bug", "feos-difference", "model-difference",
                   "parameter-difference", "solver-failure", "referee-only",
                   "single-phase-refusal", "range-refusal")
    print("\n## every disagreement that is not a coverage gap\n")
    print("| case | quantity | class | kerotakis | feos | referee | referee source |")
    print("|---|---|---|---|---|---|---|")
    shown = 0
    for r in out:
        if r[3] not in interesting:
            continue
        k = r[4] if r[5] == "ok" else f"_{r[5]}_"
        f = r[6] if r[7] == "ok" else f"_{r[7]}_"
        o = r[8] if r[9] == "ok" else f"_{r[9]}_"
        print(f"| `{r[0]}` | {r[1]} | `{r[3]}` | {k} | {f} | {o} | {r[10][:40]} |")
        shown += 1
    if not shown:
        print("| _(none)_ | | | | | | |")


def fmt(v):
    if v is None:
        return ""
    if isinstance(v, float) and not math.isfinite(v):
        return ""
    return f"{v:.6g}"


def classify(q, ours, ours_st, ours_note, theirs, theirs_st,
             ref, ref_st, ref_note, f_v, f2_v, ou_v, ou_st, theirs_note=""):
    # A case only the referee spoke about is not agreement between engines;
    # it means the corpus asked something neither engine was driven with.
    if ours_st is None and theirs_st is None:
        return "referee-only"
    # One engine emitted no row at all for this quantity, because the concept
    # does not exist in it. That is the coverage gap in its purest form.
    if ours_st is None or theirs_st is None:
        return "coverage-gap"

    # A published-parameter disagreement inside feos itself, same code.
    if f_v is not None and f2_v is not None and close(q, f_v, f2_v) is False:
        return "parameter-difference"

    missing_data = ours_st == "unavailable" and (
        "no Antoine" in (ours_note or "")
        or "no model" in (ours_note or "")
        or "not in the approved table" in (ours_note or "")
        or "no liquid-density" in (ours_note or "")
        or "no enthalpy" in (ours_note or "")
        or "curates critical properties" in (ours_note or "")
        or "no group decomposition" in (ours_note or "")
    )
    if missing_data and theirs_st == "ok":
        return "coverage-gap"
    if missing_data and theirs_st != "ok":
        return "coverage-gap"
    if ours_st == "unavailable" and "outside fitted Antoine range" in (ours_note or ""):
        # The pure-fluid path refuses to extrapolate. That is the designed
        # behaviour, and feos answering where kerotakis declines is coverage,
        # not a defect on either side.
        return "range-refusal"
    if ours_st == "unavailable" and not missing_data:
        return "solver-failure"
    if theirs_st == "unavailable":
        # feos's tp_flash raises when its stability analysis finds one phase,
        # where kerotakis reports the boundary vapour fraction. Both are
        # defensible; the difference is in the contract, not the arithmetic.
        if "No phase split" in (theirs_note or ""):
            return "single-phase-refusal"
        return "solver-failure"

    if ref_st != "ok":
        return "oracle-limitation"
    if ENCUMBERED in (ref_note or ""):
        return "oracle-limitation"

    ours_ok = close(q, ours, ref)
    theirs_ok = close(q, theirs, ref)

    if ours_st == "ok" and ours_ok is False and ref_note == "oracle-unifac-kero":
        # Same Antoine constants, same UNIFAC parameters, different code.
        return "our-bug"
    if theirs_st == "ok" and theirs_ok is False:
        # feos is off against a referee that is not its own model family;
        # only call it a feos difference when an independent UNIFAC answer
        # also disagrees with feos, i.e. two votes against one.
        if ou_st == "ok" and close(q, theirs, ou_v) is False:
            return "feos-difference"
        if ou_st != "ok":
            return "feos-difference"
    if ours_st == "ok" and theirs_st == "ok" and close(q, ours, theirs) is False:
        return "model-difference"
    if ours_st == "ok" and ours_ok is False:
        return "model-difference"
    return "agree"


if __name__ == "__main__":
    sys.exit(main())

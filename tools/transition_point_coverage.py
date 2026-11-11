#!/usr/bin/env python3
"""Reviewed transition-point coverage (2026-11-11; supersedes the 2026-11-10 table).

Every registry species' melting / boiling (or sublimation, decomposition,
dehydration) temperature, with the primary source that stands behind it, plus
the sourced melting RANGES of the materials that have no point
(`crates/kerotakis-core/src/melting_range.rs`).

The reviewed values and citations live in
`data/registry/transition-points-reviewed-20261111.json`; the registry export
reads the same citations. `--apply` rewrites the transition records of
`data/registry/registry-source-v1.json` from that file (byte-preserving
everything else). `--check` fails when

  * any value or source still cites the withdrawn `kerotakis/phase-transitions-v1`
    (or carries its "NO PRIMARY REFERENCE IS CLAIMED" notice),
  * a registry transition value or its source differs from the reviewed file,
  * a reviewed citation names no DOI and no public-domain government document,
  * the generated docs are stale.

Rows without a source are `no_data`: no value is invented.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "data/registry/registry-source-v1.json"
REVIEWED = ROOT / "data/registry/transition-points-reviewed-20261111.json"
RANGES_RS = ROOT / "crates/kerotakis-core/src/melting_range.rs"
OUT = ROOT / "docs/transition-point-coverage-20261111"
WITHDRAWN_ID = "kerotakis/phase-transitions-v1"
WITHDRAWN_TEXT = "NO PRIMARY REFERENCE IS CLAIMED"
PROPS = [  # (record id prefix, registry property, reviewed key)
    ("melting-point", "melting_temperature", "melting_k"),
    ("boiling-point", "boiling_temperature", "boiling_k"),
    ("sublimation-point", {"other": "sublimation_temperature"}, "sublimation_k"),
    ("decomposition-point", {"other": "decomposition_temperature"}, "decomposition_k"),
    ("dehydration-point", {"other": "dehydration_temperature"}, "dehydration_k"),
]
MELT = ("melting_k", "sublimation_k", "decomposition_k", "dehydration_k")
BOIL = ("boiling_k", "sublimation_k", "decomposition_k")
METHOD = ("EXP-33 transition value, re-sourced 2026-11-11: transcribed from the primary "
          "measurement or public-domain table named in the source citation")
ORIGINALS = [
    ("batch3-diverse:th-003", "tert-butanol / isopropanol mp and bp; tBuOH melting plateau"),
    ("batch3-diverse:an-007", "ethanol / isopropanol / propanone mp and bp"),
    ("batch3-diverse:th-004", "paraffin wax melting range; warning above it"),
    ("batch3-diverse:po-009", "PE / PP melting ranges; blend warning above them"),
]

sys.path.insert(0, str(Path(__file__).resolve().parent))


def split_array(text, key):
    start = text.index(f'\n  "{key}": [\n') + len(f'\n  "{key}": [\n')
    end = text.index("\n  ]", start)
    return start, end, re.split(r"(?<=\n    \}),\n(?=    \{)", text[start:end])


def chunk(obj):
    return "\n".join("    " + line for line in json.dumps(obj, indent=2, ensure_ascii=False).split("\n"))


def reviewed():
    doc = json.loads(REVIEWED.read_text(encoding="utf-8"))
    return {row["species"]: row for row in doc["rows"]}, doc


def apply():
    rows, doc = reviewed()
    text = REGISTRY.read_text(encoding="utf-8")
    # phase_thermodynamics: rewrite or drop transition records
    s, e, chunks = split_array(text, "phase_thermodynamics")
    out, cited = [], []
    for c in chunks:
        rec = json.loads(c)
        prefix = rec["id"].split("/", 1)[0]
        prop = next((p for p in PROPS if p[0] == prefix), None)
        if prop is None:
            out.append(c)
            continue
        row = rows.get(rec["species_id"])
        value = row.get(prop[2]) if row else None
        if value is None:
            continue  # unsourced: no value is invented
        rec["quantity"]["value"] = float(value)
        rec["quantity"]["source_id"] = row["source_id"]
        rec["quantity"]["method"]["detail"] = METHOD
        if row["source_id"] not in cited:
            cited.append(row["source_id"])
        out.append(chunk(rec))
    text = text[:s] + ",\n".join(out) + text[e:]
    # sources: replace the withdrawn record, at its position, by the cited
    # reviewed records in file order (the exporter pushes them the same way)
    s, e, chunks = split_array(text, "sources")
    ids = [json.loads(c)["id"] for c in chunks]
    keep = [c for c, i in zip(chunks, ids) if i != WITHDRAWN_ID and not i.startswith("literature/transition-points/")]
    at = ids.index(WITHDRAWN_ID) if WITHDRAWN_ID in ids else next(
        k for k, i in enumerate(ids) if i.startswith("literature/transition-points/"))
    before = len([i for i in ids[:at] if i != WITHDRAWN_ID and not i.startswith("literature/transition-points/")])
    new = []
    for row in doc["rows"]:
        if row["source_id"] in cited:
            new.append(chunk(dict(id=row["source_id"], citation=row["citation"], licence=row["licence"],
                                  lane="runtime", origin=row["origin"], revision=row["revision"],
                                  retrieved=row["retrieved"])))
    keep[before:before] = new
    text = text[:s] + ",\n".join(keep) + text[e:]
    REGISTRY.write_text(text, encoding="utf-8")


def parse_ranges():
    src = RANGES_RS.read_text(encoding="utf-8")
    out = []
    for m in re.finditer(r'MeltingRange \{\s*species: "([^"]+)",\s*lower_k: (?:Some\(([\d.]+)\)|None),\s*upper_k: ([\d.]+),\s*'
                         r'source: "((?:[^"\\]|\\.)*)"', src):
        out.append(dict(species=m.group(1), lower_k=float(m.group(2)) if m.group(2) else None, upper_k=float(m.group(3)),
                        source=m.group(4)))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--apply", action="store_true")
    a = ap.parse_args()
    if a.apply:
        apply()
    rows, doc = reviewed()
    reg = json.loads(REGISTRY.read_text(encoding="utf-8"))
    sources = {s["id"]: s for s in reg["sources"]}
    problems = []
    if WITHDRAWN_ID in sources:
        problems.append(f"registry still carries the withdrawn source {WITHDRAWN_ID}")
    got = {}
    for rec in reg["phase_thermodynamics"]:
        prefix = rec["id"].split("/", 1)[0]
        prop = next((p for p in PROPS if p[0] == prefix), None)
        if prop is None:
            continue
        q = rec["quantity"]
        sid = q["source_id"]
        got.setdefault(rec["species_id"], {})[prop[2]] = (q["value"], sid)
        cit = sources.get(sid, {}).get("citation", "")
        if sid == WITHDRAWN_ID or WITHDRAWN_TEXT in cit:
            problems.append(f"{rec['id']} cites the withdrawn tranche")
        row = rows.get(rec["species_id"])
        if not row or row.get(prop[2]) is None:
            problems.append(f"{rec['id']} has no reviewed value")
        elif abs(float(row[prop[2]]) - q["value"]) > 1e-9 or row["source_id"] != sid:
            problems.append(f"{rec['id']} differs from the reviewed file")
    for sp, row in rows.items():
        if row.get("kind") != "model_threshold" and not re.search(r"doi:|public domain", row["citation"]):
            problems.append(f"{sp}: citation names neither a DOI nor a public-domain document")
        if WITHDRAWN_TEXT in row["citation"] or "phase-transitions-v1" in row["citation"]:
            problems.append(f"{sp}: reviewed citation cites the withdrawn tranche")
        for _, _, key in PROPS:
            if row.get(key) is not None and got.get(sp, {}).get(key) is None:
                problems.append(f"{sp}.{key} reviewed but not in the registry")
    ranges = parse_ranges()
    for r in ranges:
        if "doi" not in r["source"]:
            problems.append(f"range {r['species']} has no DOI")

    species_rows = []
    phase = {}
    for rec in reg["phase_thermodynamics"]:
        if rec["id"].startswith("molar-mass/"):
            phase[rec["species_id"]] = rec["phase"]
    rng = {r["species"]: r for r in ranges}
    dropped = {(d["species"], d["property"]): d for d in doc.get("dropped", [])}
    for ident in reg["identities"]:
        sp = ident["id"]
        g = got.get(sp, {})
        entry = dict(species=sp, standard_phase=phase.get(sp))
        for tag, keys in (("melting", MELT), ("boiling", BOIL)):
            hit = next(((k, g[k]) for k in keys if k in g), None)
            if hit:
                k, (v, sid) = hit
                entry[tag] = dict(basis="reported", outcome=k[:-2], kelvin=v, source=sid)
            elif tag == "melting" and sp in rng:
                r = rng[sp]
                entry[tag] = dict(basis="range", lower_k=r["lower_k"], upper_k=r["upper_k"])
            else:
                d = next((dropped[(sp, k)] for k in keys if (sp, k) in dropped), None)
                entry[tag] = dict(basis="no_data", **({"withdrawn_value_dropped_k": d["old_k"]} if d else {}))
        species_rows.append(entry)
    species_rows.sort(key=lambda r: r["species"])
    counts = {}
    for r in species_rows:
        for tag in ("melting", "boiling"):
            key = f"{tag}.{r[tag]['basis']}"
            counts[key] = counts.get(key, 0) + 1
    out = dict(schema="kerotakis.transition-point-coverage.v2", counts=dict(sorted(counts.items())),
               originals=[dict(id=i, question=q) for i, q in ORIGINALS],
               sources={sp: dict(source_id=r["source_id"], citation=r["citation"]) for sp, r in sorted(rows.items())},
               ranges=ranges, dropped=doc.get("dropped", []), species=species_rows)
    js = json.dumps(out, indent=2, ensure_ascii=False) + "\n"

    def cell(t):
        if t["basis"] == "reported":
            return f"{t['outcome']} {t['kelvin']:g} K"
        if t["basis"] == "range":
            low = f"{t['lower_k']:g}" if t["lower_k"] is not None else "(unsourced)"
            return f"range {low}–{t['upper_k']:g} K"
        return "no_data" + (" (withdrawn value dropped)" if "withdrawn_value_dropped_k" in t else "")

    md = ["# Reviewed transition-point coverage (2026-11-11)", "",
          "Generated by `tools/transition_point_coverage.py` from the registry source document,",
          "`data/registry/transition-points-reviewed-20261111.json` and",
          "`crates/kerotakis-core/src/melting_range.rs`; `--check` fails when this file is stale,",
          "when any value still cites the withdrawn `kerotakis/phase-transitions-v1`, or when the",
          "registry and the reviewed file disagree.", "",
          "Every transition value the melting/boiling-point apparatus prints now carries a",
          "per-species source `literature/transition-points/<species>` naming the primary",
          "measurement (with DOI) or public-domain government table behind it. A value nobody could",
          "source was DROPPED and reads `no_data` (typed, EN/DE/FR) instead of being kept on the",
          "withdrawn tranche. Paraffin wax and the polyolefins have no melting point but a sourced",
          "melting RANGE (`PurityVerdict::MeltingRange`). A solid heated above its sourced melting",
          "point or the top of its range with no sourced enthalpy of fusion is never shown silently",
          "solid: the bench emits `not-modeled.melting-not-modelled` (EN/DE/FR). Solids with a",
          "sourced enthalpy of fusion (`phase_route::FUSION_ENTHALPIES`, e.g. tert-butanol) still",
          "melt on their plateau.", "",
          "## Trigger originals", "", "| original | question |", "|---|---|"]
    md += [f"| {i} | {q} |" for i, q in ORIGINALS]
    md += ["", f"Counts: {', '.join(f'{k} {v}' for k, v in out['counts'].items())}.", "",
           "## Melting ranges", "", "| material | range | source |", "|---|---|---|"]
    md += [f"| {r['species']} | {r['lower_k'] if r['lower_k'] is not None else '(lower end unsourced)'}–{r['upper_k']:g} K | {r['source']} |" for r in ranges]
    md += ["", "## Species", "", "| species | standard phase | melting apparatus | boiling apparatus | source |",
           "|---|---|---|---|---|"]
    for r in species_rows:
        sid = rows[r["species"]]["source_id"] if r["species"] in rows else "—"
        if rows.get(r["species"], {}).get("kind") == "model_threshold":
            sid += " (model threshold, not a literature value)"
        md.append(f"| {r['species']} | {r['standard_phase'] or '—'} | {cell(r['melting'])} | {cell(r['boiling'])} | {sid} |")
    md += ["", "## Sources", ""]
    md += [f"- **{sp}** (`{r['source_id']}`): {r['citation']}" for sp, r in sorted(rows.items())]
    if doc.get("dropped"):
        md += ["", "## Withdrawn values dropped (no primary source found)", "",
               "| species | property | withdrawn value | reason |", "|---|---|---|---|"]
        md += [f"| {d['species']} | {d['property']} | {d['old_k']:g} K | {d['reason']} |" for d in doc["dropped"]]
    mdtext = "\n".join(md) + "\n"
    out_js, out_md = OUT.with_suffix(".json"), OUT.with_suffix(".md")
    if a.check:
        bad = [str(p) for p, t in ((out_js, js), (out_md, mdtext))
               if not p.exists() or p.read_text(encoding="utf-8") != t]
        for p in problems + [f"stale: {b}" for b in bad]:
            print(p, file=sys.stderr)
        return 1 if problems or bad else 0
    for p in problems:
        print("PROBLEM:", p, file=sys.stderr)
    out_js.write_text(js, encoding="utf-8")
    out_md.write_text(mdtext, encoding="utf-8")
    print(json.dumps(out["counts"]))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())

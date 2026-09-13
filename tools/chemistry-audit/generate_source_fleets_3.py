#!/usr/bin/env python3
"""Freeze the 120-case third source-informed tranche.

Source identities and source-to-case mappings stay in the private research
repository. Public output contains original questions, scripts and generic
relations only.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "source-fleets-3"


def case(number: int, slug: str, question: str, *lines: str) -> dict:
    return {"id": f"{number}-{slug}", "question": question, "script": "\n".join(lines)}


def relation(name: str, rows: list[dict], assertion: str, **parameters: object) -> dict:
    kind = "conservation" if "conserved" in assertion else "independent-law"
    return {"id": name, "kind": kind, "assertion": assertion,
            "cases": [row["id"] for row in rows], "parameters": parameters}


def grouped(family: str, rows: list[dict], rels: list[dict]) -> dict:
    assert len(rows) == 24 and len(rels) == 6
    return {"schema": "kerotakis-source-fleet-v3", "family": family,
            "cases": rows, "relations": rels}


def water_energy(start: int) -> dict:
    rows, rels = [], []
    for group, water_ml in enumerate((100, 175, 250, 400, 600, 900)):
        block = []
        for offset, joules in enumerate((125, 250, 500, 1000)):
            number = start + group * 4 + offset
            line = f"heat v1 {joules}J" if group % 2 == 0 else f"cool v1 {joules}J"
            row = case(number, f"water-energy-{water_ml}-{joules}",
                       "Does a declared energy transfer preserve matter while moving the water state?",
                       f"add v1 water {water_ml}mL", line, "measure v1 temp")
            rows.append(row); block.append(row)
        direction = "increasing" if group % 2 == 0 else "decreasing"
        rels.append(relation(f"v3-water-energy-{group+1}", block, "event-scalar-order",
                             event="measured", field="value", occurrence="last",
                             direction=direction, min_delta=1e-6))
    return grouped("water-energy", rows, rels)


def aqueous_selectivity(start: int) -> dict:
    rows, rels = [], []
    salts = (("Na2SO4", "sulfate"), ("Na2CO3", "carbonate"))
    for group in range(6):
        block = []
        salt, label = salts[group % 2]
        for offset, dose in enumerate((0.00025, 0.0005, 0.001, 0.002)):
            number = start + group * 4 + offset
            lines = ["add v1 water 300mL", "add v1 BaCl2 0.001mol",
                     f"add v1 {salt} {dose:g}mol"]
            if group >= 2:
                lines.append(f"add v1 HCl {0.0002 * (group - 1):g}mol")
            lines.extend(("new", "filter v1 v2"))
            row = case(number, f"barium-{label}-{group+1}-{offset+1}",
                       "Do defined sulfate and carbonate controls conserve every introduced element across aqueous and solid phases?",
                       *lines)
            rows.append(row); block.append(row)
        rels.append(relation(f"v3-aqueous-selectivity-{group+1}", block,
                             "case-elements-conserved", elements=["Ba", "Cl", "Na", "S", "C", "O", "H"],
                             before_op="filter", atol=1e-10, rtol=1e-7))
    return grouped("aqueous-selectivity", rows, rels)


def ionic_conductivity(start: int) -> dict:
    rows, rels = [], []
    salts = ("NaCl", "KCl", "CaCl2")
    for group in range(6):
        block = []
        salt = salts[group % len(salts)]
        water = 150 + group * 50
        for offset, dose in enumerate((0.0001, 0.00025, 0.0005, 0.001)):
            number = start + group * 4 + offset
            row = case(number, f"conductivity-{salt.lower()}-{water}-{offset+1}",
                       "Within one dilute series, does computed conductivity rise with represented ion amount?",
                       f"add v1 water {water}mL", f"add v1 {salt} {dose:g}mol",
                       "measure v1 conductivity")
            rows.append(row); block.append(row)
        rels.append(relation(f"v3-ionic-conductivity-{group+1}", block,
                             "event-scalar-order", event="measured", field="value",
                             occurrence="last", direction="increasing", min_delta=1e-8))
    return grouped("ionic-conductivity", rows, rels)


def metal_ligand(start: int) -> dict:
    rows, rels = [], []
    for group in range(6):
        block = []
        for offset, ligand in enumerate((0.00005, 0.0001, 0.00025, 0.0005)):
            number = start + group * 4 + offset
            lines = [f"add v1 water {200 + group * 25}mL",
                     f"add v1 FeCl3 {0.0002 + group * 0.00005:g}mol",
                     f"add v1 KSCN {ligand:g}mol"]
            lines.append(f"heat v1 {25 * (group + 1)}J")
            lines.append("inspect v1")
            row = case(number, f"metal-ligand-{group+1}-{offset+1}",
                       "Which parts of a metal-ligand perturbation are computed, and where does the model state a boundary?",
                       *lines)
            rows.append(row); block.append(row)
        rels.append(relation(f"v3-metal-ligand-{group+1}", block,
                             "case-elements-conserved", elements=["Fe", "Cl", "K", "S", "C", "N", "O", "H"],
                             before_op="heat", atol=1e-10, rtol=1e-7))
    return grouped("metal-ligand", rows, rels)


def organic_equilibrium(start: int) -> dict:
    rows, rels = [], []
    for group in range(6):
        block = []
        for offset, scale in enumerate((0.5, 1.0, 1.5, 2.0)):
            number = start + group * 4 + offset
            acid = (0.004 + group * 0.001) * scale
            alcohol = (0.009 - group * 0.0005) * scale
            ester = (0.001 + group * 0.0004) * scale
            water = (0.003 + group * 0.0006) * scale
            row = case(number, f"ester-quotient-{group+1}-{offset+1}",
                       "Does the signed ideal ester equilibrium preserve C, H and O for a new initial quotient?",
                       f"add v1 CH3COOH {acid:g}mol", f"add v1 ethanol {alcohol:g}mol",
                       f"add v1 ethyl_acetate {ester:g}mol", f"add v1 water {water:g}mol",
                       "react v1 esterification")
            rows.append(row); block.append(row)
        rels.append(relation(f"v3-organic-equilibrium-{group+1}", block,
                             "case-elements-conserved", elements=["C", "H", "O"],
                             before_op="react", atol=1e-10, rtol=1e-7))
    return grouped("organic-equilibrium", rows, rels)


def manifests() -> list[dict]:
    return [water_energy(777), aqueous_selectivity(801), ionic_conductivity(825),
            metal_ligand(849), organic_equilibrium(873)]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = manifests()
    expected = {m["family"]: json.dumps(m, indent=2) + "\n" for m in generated}
    if args.check:
        stale = [name for name, text in expected.items()
                 if not (OUT / f"{name}.json").exists() or (OUT / f"{name}.json").read_text() != text]
        if stale:
            raise SystemExit(f"stale v3 manifests: {', '.join(stale)}")
        return
    OUT.mkdir(exist_ok=True)
    for name, text in expected.items():
        (OUT / f"{name}.json").write_text(text)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Deterministically freeze original source-fleet designs 489--632.

Source-to-design research remains private.  This public generator contains
only original questions, executable scripts, and machine-checkable relations.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "source-fleets-2"


def case(n: int, slug: str, question: str, *script: str) -> dict:
    return {"id": f"{n}-{slug}", "question": question, "script": "\n".join(script)}


def relation(name: str, assertion: str, rows: list[dict], **parameters: object) -> dict:
    if assertion in {"case-elements-conserved", "final-elements-equal"}:
        kind = "conservation"
    elif assertion in {"event-present", "event-boundary-present"}:
        kind = "boundary"
    elif assertion.endswith("-order"):
        kind = "independent-law"
    else:
        kind = "metamorphic"
    return {"id": name, "kind": kind, "assertion": assertion,
            "cases": [row["id"] for row in rows], "parameters": parameters}


def manifest(family: str, cases: list[dict], relations: list[dict]) -> dict:
    assert len(cases) == 24 and len({row["id"] for row in cases}) == 24
    return {"schema": "kerotakis-source-fleet-v2", "family": family,
            "cases": cases, "relations": relations}


def quantitative_solutions() -> dict:
    rows, rels = [], []
    groups = [
        (range(489, 493), (100, 200, 400, 800), "fixed-salt-dilution",
         "How does dilution change conductivity when the salt amount is fixed?",
         lambda x: (f"add v1 water {x}mL", "add v1 NaCl 0.002mol", "measure v1 conductivity"),
         "decreasing", 1e-8),
        (range(493, 497), (0.0005, 0.001, 0.002, 0.004), "salt-dose-conductivity",
         "How does sodium chloride amount change conductivity in fixed water?",
         lambda x: ("add v1 water 350mL", f"add v1 NaCl {x:g}mol", "measure v1 conductivity"),
         "increasing", 1e-8),
        (range(497, 501), (0.0002, 0.0005, 0.001, 0.002), "acid-dose-ph",
         "How does strong-acid amount change pH in the same water volume?",
         lambda x: ("add v1 water 500mL", f"add v1 HCl {x:g}mol", "measure v1 ph"),
         "decreasing", .1),
        (range(501, 505), (0.0002, 0.0005, 0.001, 0.002), "base-dose-ph",
         "How does strong-base amount change pH in the same water volume?",
         lambda x: ("add v1 water 500mL", f"add v1 NaOH {x:g}mol", "measure v1 ph"),
         "increasing", .1),
        (range(505, 509), (1, 2, 3, 4), "saline-extensive",
         "Does proportional scaling preserve saline conductivity?",
         lambda x: (f"add v1 water {250*x}mL", f"add v1 NaCl {0.001*x:g}mol", "measure v1 conductivity"),
         "equal", 0.0),
        (range(509, 513), (100, 200, 400, 800), "fixed-solute-mass",
         "Does adding solvent preserve the solute atoms before measurement?",
         lambda x: (f"add v1 water {x}mL", "add v1 KCl 0.002mol", "measure v1 balance"),
         "order", 1.0),
    ]
    for nums, values, slug, question, script, direction, delta in groups:
        group = [case(n, f"{slug}-{v:g}", question, *script(v)) for n, v in zip(nums, values)]
        rows += group
        if direction == "equal":
            rels.append(relation(f"qs-{slug}", "event-scalar-equal", group,
                                 event="measured", field="value", occurrence="last", atol=.02, rtol=2e-3))
        else:
            rels.append(relation(f"qs-{slug}", "event-scalar-order", group,
                                 event="measured", field="value", occurrence="last",
                                 direction="increasing" if direction == "order" else direction,
                                 min_delta=delta))
    return manifest("quantitative-solutions", rows, rels)


def polyprotic_carbonate() -> dict:
    rows, rels = [], []
    specs = [
        (513, "phosphoric-dose", (0.0003, .0006, .0012, .0024), "How does phosphoric-acid dose move pH?", "H3PO4", "decreasing"),
        (517, "phosphoric-dilution", (100, 200, 400, 800), "How does dilution move the pH of fixed phosphoric acid?", None, "increasing"),
        (521, "bicarbonate-dose", (.0005, .001, .002, .004), "How does bicarbonate dose move aqueous pH?", "NaHCO3", "increasing"),
        (525, "carbonate-dose", (.00025, .0005, .001, .002), "How does carbonate dose move aqueous pH?", "Na2CO3", "increasing"),
        (529, "carbonate-acid", (.0002, .0005, .001, .002), "How does acid dose move a fixed carbonate solution's pH?", "acid", "decreasing"),
        (533, "carbonate-scale", (1, 2, 3, 4), "Does proportional carbonate scaling preserve pH?", "scale", "equal"),
    ]
    for start, slug, values, question, mode, direction in specs:
        group = []
        for n, value in zip(range(start, start + 4), values):
            if mode is None:
                script = (f"add v1 water {value}mL", "add v1 H3PO4 0.001mol", "measure v1 ph")
            elif mode == "acid":
                script = ("add v1 water 300mL", "add v1 Na2CO3 0.002mol", f"add v1 HCl {value:g}mol", "measure v1 ph")
            elif mode == "scale":
                script = (f"add v1 water {200*value}mL", f"add v1 Na2CO3 {0.001*value:g}mol", "measure v1 ph")
            else:
                script = ("add v1 water 300mL", f"add v1 {mode} {value:g}mol", "measure v1 ph")
            group.append(case(n, f"{slug}-{value:g}", question, *script))
        rows += group
        assertion = "event-scalar-equal" if direction == "equal" else "event-scalar-order"
        params = dict(event="measured", field="value", occurrence="last", atol=.03, rtol=0.0) if direction == "equal" else dict(event="measured", field="value", occurrence="last", direction=direction, min_delta=.02)
        rels.append(relation(f"pc-{slug}", assertion, group, **params))
    return manifest("polyprotic-carbonate", rows, rels)


def redox_cells() -> dict:
    rows, rels = [], []
    for start, slug, values, side, direction in [
        (537, "copper-activity", (.0005, .001, .004, .008), "CuSO4", "increasing"),
        (541, "zinc-activity", (.0005, .001, .004, .008), "ZnSO4", "decreasing"),
    ]:
        group = []
        for n, value in zip(range(start, start + 4), values):
            zn, cu = (0.002, value) if side == "CuSO4" else (value, .008)
            group.append(case(n, f"{slug}-{value:g}", "How does one half-cell ion amount shift Daniell voltage?",
                "add v1 water 200mL", f"add v1 ZnSO4 {zn:g}mol", "add v1 Zn 0.05mol", "new",
                "add v2 water 200mL", f"add v2 CuSO4 {cu:g}mol", "add v2 Cu 0.05mol", "cell v1 v2"))
        rows += group
        rels.append(relation(f"rc-{slug}", "event-scalar-order", group, event="cell_voltage", field="volts", occurrence="last", direction=direction, min_delta=.0001))
    displacement = [(545, "zinc-copper", "CuSO4", "Zn"), (549, "iron-copper", "CuSO4", "Fe"),
                    (553, "copper-silver", "AgNO3", "Cu"), (557, "magnesium-copper", "CuSO4", "Mg")]
    for start, slug, salt, metal in displacement:
        group = [
            case(start, f"{slug}-salt-first", "Does feed order alter the final displacement inventory?", "add v1 water 200mL", f"add v1 {salt} 0.003mol", f"add v1 {metal} 0.004mol", "inspect v1"),
            case(start+1, f"{slug}-metal-first", "Does reversed feed order alter the final displacement inventory?", "add v1 water 200mL", f"add v1 {metal} 0.004mol", f"add v1 {salt} 0.003mol", "inspect v1"),
            case(start+2, f"{slug}-split-salt", "Does splitting the salt feed alter the final displacement inventory?", "add v1 water 200mL", f"add v1 {salt} 0.001mol", f"add v1 {metal} 0.004mol", f"add v1 {salt} 0.002mol", "inspect v1"),
            case(start+3, f"{slug}-split-metal", "Does splitting the metal feed alter the final displacement inventory?", "add v1 water 200mL", f"add v1 {metal} 0.001mol", f"add v1 {salt} 0.003mol", f"add v1 {metal} 0.003mol", "inspect v1"),
        ]
        rows += group
        rels.append(relation(f"rc-{slug}-path", "final-elements-equal", group,
                             elements=["Cu", "Zn", "Fe", "Ag", "Mg", "S", "N", "O"], atol=1e-8, rtol=1e-6))
    return manifest("redox-cells", rows, rels)


def gas_production_transfer() -> dict:
    rows, rels = [], []
    recipes = [
        (561, "carbonate-acid-dose", (.0005, .001, .002, .003), "How does acid dose change carbon dioxide production?", lambda x: ("add v1 water 250mL", "add v1 NaHCO3 0.004mol", f"add v1 HCl {x:g}mol", "inspect v1"), "gas_evolved"),
        (565, "carbonate-dose", (.001, .002, .004, .008), "How does bicarbonate dose change gas production with excess acid?", lambda x: ("add v1 water 250mL", f"add v1 NaHCO3 {x:g}mol", "add v1 HCl 0.01mol", "inspect v1"), "gas_evolved"),
        (569, "sealed-carbonate", (.001, .002, .003, .004), "Does sealed acid-carbonate chemistry retain carbon in the vessel?", lambda x: ("seal v1 750mL", "add v1 water 200mL", f"add v1 NaHCO3 {x:g}mol", "add v1 HCl 0.006mol", "measure v1 pressure"), "measured"),
        (573, "gas-amount", (.001, .002, .004, .008), "How does nitrogen amount change sealed pressure?", lambda x: ("seal v1 800mL", f"add v1 N2 {x:g}mol", "measure v1 pressure"), "measured"),
        (577, "gas-volume", (1200, 900, 600, 300), "How does headspace volume change pressure for fixed gas?", lambda x: (f"seal v1 {x}mL", "add v1 N2 0.004mol", "measure v1 pressure"), "measured"),
        (581, "gas-energy", (0, 50, 100, 150), "How does thermal input change sealed-gas pressure?", lambda x: ("seal v1 700mL", "add v1 N2 0.004mol", *((f"heat v1 {x}J",) if x else ()), "measure v1 pressure"), "measured"),
    ]
    for start, slug, values, question, script, event in recipes:
        group = [case(n, f"{slug}-{v:g}", question, *script(v)) for n, v in zip(range(start,start+4), values)]
        rows += group
        if event == "gas_evolved":
            rels.append(relation(f"gt-{slug}", "event-present", group, event=event))
            rels.append(relation(f"gt-{slug}-amount", "event-scalar-order", group,
                                 event=event, field="moles", occurrence="last",
                                 direction="increasing", min_delta=1e-5))
        else:
            direction = "increasing" if slug != "gas-volume" else "increasing"
            rels.append(relation(f"gt-{slug}", "event-scalar-order", group, event=event, field="value", occurrence="last", direction=direction, min_delta=.01))
    return manifest("gas-production-transfer", rows, rels)


def phase_colligative() -> dict:
    rows, rels = [], []
    specs = [
        (585, "water-heating", (100, 250, 500, 1000), "How does heat input move liquid-water temperature?", lambda x: ("add v1 water 250mL", f"heat v1 {x}J", "measure v1 temp"), "increasing"),
        (589, "water-cooling", (100, 250, 500, 1000), "How does removed energy move liquid-water temperature?", lambda x: ("add v1 water 250mL", f"cool v1 {x}J", "measure v1 temp"), "decreasing"),
        (593, "salt-freezing", (1,2,3,4), "Does salt-feed path preserve the final cooled solution state?", lambda x: (("add v1 water 200mL", "add v1 NaCl 0.004mol", "cool v1 25kJ", "measure v1 temp") if x == 1 else ("add v1 NaCl 0.004mol", "add v1 water 200mL", "cool v1 25kJ", "measure v1 temp") if x == 2 else ("add v1 water 200mL", "add v1 NaCl 0.001mol", "add v1 NaCl 0.003mol", "cool v1 25kJ", "measure v1 temp") if x == 3 else ("add v1 water 100mL", "add v1 NaCl 0.004mol", "add v1 water 100mL", "cool v1 25kJ", "measure v1 temp")), "path"),
        (597, "heat-partition", ((750,), (375,375), (125,250,375), (50,100,200,400)), "Does partitioning equal heat preserve final temperature?", None, "equal"),
        (601, "cool-partition", ((600,), (300,300), (100,200,300), (50,100,150,300)), "Does partitioning equal cooling preserve final temperature?", None, "equal"),
        (605, "thermal-scale", (1,2,3,4), "Does proportional water and heat scaling preserve temperature?", None, "equal"),
    ]
    for start, slug, values, question, maker, direction in specs:
        group=[]
        for n,v in zip(range(start,start+4),values):
            if maker: script=maker(v)
            elif slug == "heat-partition": script=("add v1 water 250mL", *(f"heat v1 {e}J" for e in v), "measure v1 temp")
            elif slug == "cool-partition": script=("add v1 water 250mL", *(f"cool v1 {e}J" for e in v), "measure v1 temp")
            else: script=(f"add v1 water {250*v}mL", f"heat v1 {600*v}J", "measure v1 temp")
            group.append(case(n, f"{slug}-{n-start+1}", question, *script))
        rows += group
        if slug == "salt-freezing": rels.append(relation(f"pcg-{slug}-path", "final-inventory-equal", group, species=["water", "Na+", "Cl-", "NaCl"], phases=["Liquid", "Solid"], atol=1e-8, rtol=1e-6))
        elif direction == "equal": rels.append(relation(f"pcg-{slug}", "event-scalar-equal", group, event="measured", field="value", occurrence="last", atol=.03, rtol=1e-6))
        else: rels.append(relation(f"pcg-{slug}", "event-scalar-order", group, event="measured", field="value", occurrence="last", direction=direction, min_delta=.01))
    return manifest("phase-colligative", rows, rels)


def crystallisation_boundaries() -> dict:
    rows, rels = [], []
    specs = [
        (609, "nacl-evaporation", (.1,.25,.5,.75), "How does water removal move a sodium chloride solution toward crystals?", "NaCl", .04),
        (613, "kcl-evaporation", (.1,.25,.5,.75), "How does water removal move a potassium chloride solution toward crystals?", "KCl", .04),
        (617, "nitrate-evaporation", (.1,.25,.5,.75), "How does water removal move a nitrate solution toward crystals?", "NaNO3", .04),
        (621, "nacl-loading", (.15,.3,.5,.7), "How does evaporation fraction change a concentrated sodium chloride solution?", "NaCl", None),
        (625, "kcl-loading", (.15,.3,.5,.7), "How does evaporation fraction change a concentrated potassium chloride solution?", "KCl", None),
        (629, "evaporation-partition", ((.4,),(.2,.25),(.1,.333333333333),(.05,.1,.2,.122807017544)), "Can different evaporation sequences remove the same net water fraction?", "NaCl", None),
    ]
    for start, slug, values, question, salt, fixed in specs:
        group=[]
        for n,v in zip(range(start,start+4),values):
            if slug.endswith("evaporation") and fixed is not None:
                script=("add v1 water 400mL", f"add v1 {salt} {fixed:g}mol", f"evaporate v1 {v:g}", "inspect v1")
            elif slug.endswith("loading"):
                script=("add v1 water 250mL", f"add v1 {salt} 0.025mol", f"evaporate v1 {v:g}", "inspect v1")
            else:
                script=("add v1 water 400mL", "add v1 NaCl 0.03mol", *(f"evaporate v1 {x:g}" for x in v), "inspect v1")
            group.append(case(n, f"{slug}-{n-start+1}", question, *script))
        rows += group
        rels.append(relation(f"cb-{slug}", "event-present", group, event="evaporated"))
        if slug == "evaporation-partition":
            rels.append(relation(f"cb-{slug}-inventory", "final-inventory-equal", group,
                                 species=["water", "Na+", "Cl-", "NaCl"], phases=["Liquid", "Solid"], atol=1e-8, rtol=1e-6))
        else:
            rels.append(relation(f"cb-{slug}-amount", "event-scalar-order", group,
                                 event="evaporated", field="moles", occurrence="last",
                                 direction="increasing", min_delta=1e-6))
    return manifest("crystallisation-boundaries", rows, rels)


def generated() -> dict[str, dict]:
    docs = [quantitative_solutions(), polyprotic_carbonate(), redox_cells(),
            gas_production_transfer(), phase_colligative(), crystallisation_boundaries()]
    return {doc["family"]: doc for doc in docs}


def encoded(document: dict) -> str:
    return json.dumps(document, indent=2, ensure_ascii=False) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    stale=[]
    for family, document in generated().items():
        path=OUT/f"{family}.json"; content=encoded(document)
        if args.write: path.write_text(content)
        elif not path.exists() or path.read_text()!=content: stale.append(str(path))
    if stale: raise SystemExit("stale or missing generated manifests: " + ", ".join(stale))
    print(json.dumps({"families": 6, "cases": 144, "status": "written" if args.write else "current"}))


if __name__ == "__main__": main()

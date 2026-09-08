#!/usr/bin/env python3
"""Deterministically freeze source-informed fleet manifests 219--338.

The prose and experimental designs here are original.  Source-to-design notes
are deliberately not part of this repository.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path


HERE = Path(__file__).resolve().parent
OUT = HERE / "source-fleets"


def case(number: int, slug: str, question: str, *lines: str) -> dict:
    return {
        "id": f"{number}-{slug}",
        "question": question,
        "script": "\n".join(lines),
    }


def relation(name: str, assertion: str, cases: list[dict], **parameters) -> dict:
    if assertion in {"case-elements-conserved", "final-elements-equal"}:
        kind = "conservation"
    elif assertion in {"event-present", "event-boundary-present"}:
        kind = "boundary"
    elif assertion.endswith("-order"):
        kind = "independent-law"
    else:
        kind = "metamorphic"
    return {
        "id": name,
        "kind": kind,
        "assertion": assertion,
        "cases": [item["id"] for item in cases],
        "parameters": parameters,
    }


def acid_base() -> dict:
    cases: list[dict] = []
    relations: list[dict] = []

    group = []
    for number, acid in zip(range(219, 223), (0.00025, 0.0005, 0.001, 0.002)):
        group.append(case(number, f"strong-acid-{acid:g}",
            "How does increasing strong-acid amount move the pH of a fixed water portion?",
            "add v1 water 400mL", f"add v1 HCl {acid}mol", "measure v1 ph"))
    cases += group
    relations.append(relation("ab-strong-acid-monotonic", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="decreasing", min_delta=0.20))

    group = []
    for number, water in zip(range(223, 227), (100, 200, 400, 800)):
        group.append(case(number, f"acid-dilution-{water}",
            "Does diluting a fixed strong-acid amount raise pH while retaining its atoms?",
            f"add v1 water {water}mL", "add v1 HCl 0.0012mol", "measure v1 ph"))
    cases += group
    relations += [
        relation("ab-dilution-ph-order", "event-scalar-order", group,
            event="measured", field="value", occurrence="last", direction="increasing", min_delta=0.20),
        relation("ab-dilution-elements", "case-elements-conserved", group,
            elements=["H", "Cl"], before_op="measure"),
    ]

    group = []
    for number, base in zip(range(227, 231), (0.00025, 0.0005, 0.001, 0.002)):
        group.append(case(number, f"strong-base-{base:g}",
            "How does increasing strong-base amount move the pH of a fixed water portion?",
            "add v1 water 400mL", f"add v1 NaOH {base}mol", "measure v1 ph"))
    cases += group
    relations.append(relation("ab-strong-base-monotonic", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=0.20))

    group = [
        case(231, "neutral-order-acid-first", "Does reagent order alter a neutral strong-acid/strong-base inventory?",
            "add v1 water 300mL", "add v1 HCl 0.001mol", "add v1 NaOH 0.001mol", "measure v1 ph"),
        case(232, "neutral-order-base-first", "Does reversing reagent order alter the same neutral inventory?",
            "add v1 water 300mL", "add v1 NaOH 0.001mol", "add v1 HCl 0.001mol", "measure v1 ph"),
        case(233, "neutral-split-acid", "Does splitting the acid addition alter the neutral final inventory?",
            "add v1 water 300mL", "add v1 HCl 0.0004mol", "add v1 HCl 0.0006mol", "add v1 NaOH 0.001mol", "measure v1 ph"),
        case(234, "neutral-split-base", "Does splitting the base addition alter the neutral final inventory?",
            "add v1 water 300mL", "add v1 HCl 0.001mol", "add v1 NaOH 0.0003mol", "add v1 NaOH 0.0007mol", "measure v1 ph"),
    ]
    cases += group
    relations.append(relation("ab-neutral-path-independence", "final-inventory-equal", group,
        species=["water", "H+", "OH-", "Na+", "Cl-"], phases=["Liquid"], atol=1e-9, rtol=1e-7))

    group = []
    for number, acetate in zip(range(235, 239), (0.001, 0.002, 0.004, 0.008)):
        group.append(case(number, f"acetate-buffer-{acetate:g}",
            "How does added conjugate base change the pH of a fixed weak-acid portion?",
            "add v1 water 400mL", "add v1 CH3COOH 0.004mol",
            f"add v1 NaOAc {acetate}mol", "measure v1 ph"))
    cases += group
    relations.append(relation("ab-conjugate-base-order", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=0.10))

    group = []
    for number, scale in zip(range(239, 243), (1, 2, 3, 4)):
        group.append(case(number, f"buffer-extensive-{scale}",
            "Does extensive scaling preserve the pH of an acetate buffer?",
            f"add v1 water {180 * scale}mL", f"add v1 CH3COOH {0.0018 * scale:g}mol",
            f"add v1 NaOAc {0.0018 * scale:g}mol", "measure v1 ph"))
    cases += group
    relations += [
        relation("ab-buffer-intensive", "event-scalar-equal", group,
            event="measured", field="value", occurrence="last", atol=0.03, rtol=0.0),
        relation("ab-buffer-conservation", "case-elements-conserved", group,
            elements=["C", "H", "O", "Na"], before_op="measure"),
    ]
    return manifest("acid-base-buffers", cases, relations)


def solubility() -> dict:
    cases: list[dict] = []
    relations: list[dict] = []
    group = [
        case(243, "silver-chloride-salt-first", "Does addition order change silver chloride formation?",
            "add v1 water 200mL", "add v1 NaCl 0.004mol", "add v1 AgNO3 0.004mol", "inspect v1"),
        case(244, "silver-chloride-silver-first", "Does reversing the reagents change silver chloride formation?",
            "add v1 water 200mL", "add v1 AgNO3 0.004mol", "add v1 NaCl 0.004mol", "inspect v1"),
        case(245, "silver-chloride-split-salt", "Does splitting chloride delivery change the final phase inventory?",
            "add v1 water 200mL", "add v1 NaCl 0.001mol", "add v1 AgNO3 0.004mol", "add v1 NaCl 0.003mol", "inspect v1"),
        case(246, "silver-chloride-split-silver", "Does splitting silver delivery change the final phase inventory?",
            "add v1 water 200mL", "add v1 AgNO3 0.0015mol", "add v1 NaCl 0.004mol", "add v1 AgNO3 0.0025mol", "inspect v1"),
    ]
    cases += group
    relations.append(relation("sol-agcl-path-independence", "final-inventory-equal", group,
        species=["Ag+", "Cl-", "AgCl"], phases=["Liquid", "Solid"], atol=2e-8, rtol=2e-6))

    group = []
    for number, scale in zip(range(247, 251), (1, 2, 3, 4)):
        group.append(case(number, f"silver-chloride-scale-{scale}",
            "Does silver chloride production scale with an extensive recipe?",
            f"add v1 water {100 * scale}mL", f"add v1 NaCl {0.001 * scale:g}mol",
            f"add v1 AgNO3 {0.001 * scale:g}mol", "inspect v1"))
    cases += group
    relations += [
        relation("sol-agcl-extensive-elements", "case-elements-conserved", group,
            elements=["Ag", "Cl", "Na", "N"], before_op="inspect"),
        relation("sol-agcl-precipitation", "event-present", group,
            event="precipitated", count=1),
    ]

    group = []
    for number, chloride in zip(range(251, 255), (0.001, 0.002, 0.004, 0.008)):
        group.append(case(number, f"silver-limited-chloride-{chloride:g}",
            "What changes when chloride is raised around a fixed silver amount?",
            "add v1 water 250mL", f"add v1 NaCl {chloride}mol",
            "add v1 AgNO3 0.0023mol", "inspect v1"))
    cases += group
    relations.append(relation("sol-silver-limited-conservation", "case-elements-conserved", group,
        elements=["Ag", "Cl", "Na", "N"], before_op="inspect"))

    group = []
    for number, silver in zip(range(255, 259), (0.001, 0.002, 0.004, 0.008)):
        group.append(case(number, f"chloride-limited-silver-{silver:g}",
            "What changes when silver is raised around a fixed chloride amount?",
            "add v1 water 250mL", "add v1 NaCl 0.0027mol",
            f"add v1 AgNO3 {silver}mol", "inspect v1"))
    cases += group
    relations.append(relation("sol-chloride-limited-conservation", "case-elements-conserved", group,
        elements=["Ag", "Cl", "Na", "N"], before_op="inspect"))

    group = [
        case(259, "carbonate-calcium-first", "Does addition order alter calcium carbonate precipitation?",
            "add v1 water 250mL", "add v1 CaCl2 0.003mol", "add v1 Na2CO3 0.003mol", "inspect v1"),
        case(260, "carbonate-carbonate-first", "Does reversing the reagents alter calcium carbonate precipitation?",
            "add v1 water 250mL", "add v1 Na2CO3 0.003mol", "add v1 CaCl2 0.003mol", "inspect v1"),
        case(261, "carbonate-split-calcium", "Does split calcium delivery alter the final carbonate phases?",
            "add v1 water 250mL", "add v1 CaCl2 0.001mol", "add v1 Na2CO3 0.003mol", "add v1 CaCl2 0.002mol", "inspect v1"),
        case(262, "carbonate-split-carbonate", "Does split carbonate delivery alter the final carbonate phases?",
            "add v1 water 250mL", "add v1 Na2CO3 0.001mol", "add v1 CaCl2 0.003mol", "add v1 Na2CO3 0.002mol", "inspect v1"),
    ]
    cases += group
    relations.append(relation("sol-caco3-path-independence", "final-inventory-equal", group,
        species=["Ca+2", "CO3-2", "CaCO3"], phases=["Liquid", "Solid"], atol=2e-8, rtol=2e-6))

    group = [
        case(263, "filter-agcl-reference", "Where do silver chloride and its liquid go during filtration?",
            "add v1 water 200mL", "add v1 NaCl 0.003mol", "add v1 AgNO3 0.003mol", "new", "filter v1 v2", "inspect"),
        case(264, "filter-agcl-split-feed", "Does split reagent delivery conserve the filtered system?",
            "add v1 water 200mL", "add v1 NaCl 0.001mol", "add v1 NaCl 0.002mol", "add v1 AgNO3 0.003mol", "new", "filter v1 v2", "inspect"),
        case(265, "filter-agcl-reverse-feed", "Does reversed reagent order conserve the filtered system?",
            "add v1 water 200mL", "add v1 AgNO3 0.003mol", "add v1 NaCl 0.003mol", "new", "filter v1 v2", "inspect"),
        case(266, "filter-agcl-split-silver", "Does split silver delivery conserve the filtered system?",
            "add v1 water 200mL", "add v1 AgNO3 0.001mol", "add v1 NaCl 0.003mol", "add v1 AgNO3 0.002mol", "new", "filter v1 v2", "inspect"),
    ]
    cases += group
    relations += [
        relation("sol-filter-system-equality", "final-elements-equal", group,
            elements=["Ag", "Cl", "Na", "N", "H", "O"], atol=2e-8, rtol=2e-6),
        relation("sol-filter-event", "event-present", group, event="filtered", count=1),
    ]
    return manifest("solubility-precipitation", cases, relations)


def gases() -> dict:
    cases: list[dict] = []
    relations: list[dict] = []
    group = []
    for number, amount in zip(range(267, 271), (0.001, 0.002, 0.004, 0.008)):
        group.append(case(number, f"gas-amount-{amount:g}",
            "How does gas amount change pressure at fixed sealed volume?",
            "seal v1 500mL", f"add v1 N2 {amount}mol", "measure v1 pressure"))
    cases += group
    relations.append(relation("gas-amount-pressure-order", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=1.0))

    group = []
    for number, volume in zip(range(271, 275), (1000, 750, 500, 250)):
        group.append(case(number, f"gas-volume-{volume}",
            "How does reducing sealed headspace change pressure for fixed gas amount?",
            f"seal v1 {volume}mL", "add v1 N2 0.0037mol", "measure v1 pressure"))
    cases += group
    relations.append(relation("gas-volume-pressure-order", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=1.0))

    group = []
    for number, energy in zip(range(275, 279), (0, 100, 200, 400)):
        lines = ["seal v1 500mL", "add v1 N2 0.0042mol"]
        if energy:
            lines.append(f"heat v1 {energy}J")
        lines.append("measure v1 pressure")
        group.append(case(number, f"gas-heating-{energy}",
            "How does added energy change the pressure of a fixed sealed gas?", *lines))
    cases += group
    relations.append(relation("gas-heating-pressure-order", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=1.0))

    group = [
        case(279, "gas-energy-single", "Does one energy pulse reach the same state as partitioned pulses?",
            "seal v1 500mL", "add v1 N2 0.004mol", "heat v1 300J", "measure v1 pressure"),
        case(280, "gas-energy-halves", "Do two equal energy pulses reach the same final state?",
            "seal v1 500mL", "add v1 N2 0.004mol", "heat v1 150J", "heat v1 150J", "measure v1 pressure"),
        case(281, "gas-energy-unequal", "Do unequal energy pulses with the same sum reach the same state?",
            "seal v1 500mL", "add v1 N2 0.004mol", "heat v1 80J", "heat v1 220J", "measure v1 pressure"),
        case(282, "gas-energy-three-pulses", "Do three energy pulses with the same sum reach the same state?",
            "seal v1 500mL", "add v1 N2 0.004mol", "heat v1 50J", "heat v1 100J", "heat v1 150J", "measure v1 pressure"),
    ]
    cases += group
    relations.append(relation("gas-energy-partition", "final-scalar-equal", group,
        path="v1.temperature", atol=1e-7, rtol=1e-7))

    group = []
    for number, energy in zip(range(283, 287), (50, 100, 200, 400)):
        group.append(case(number, f"gas-roundtrip-{energy}",
            "Does equal heating and cooling restore the initial sealed-gas temperature?",
            "seal v1 600mL", "add v1 N2 0.005mol", f"heat v1 {energy}J",
            f"cool v1 {energy}J", "measure v1 pressure"))
    cases += group
    relations.append(relation("gas-energy-roundtrip", "final-scalar-equal", group,
        path="v1.temperature", atol=1e-7, rtol=1e-7))

    group = []
    for number, scale in zip(range(287, 291), (1, 2, 3, 4)):
        group.append(case(number, f"gas-extensive-{scale}",
            "Does scaling gas amount, volume, and energy preserve intensive state?",
            f"seal v1 {250 * scale}mL", f"add v1 N2 {0.002 * scale:g}mol",
            f"heat v1 {75 * scale}J", "measure v1 pressure"))
    cases += group
    relations += [
        relation("gas-extensive-pressure", "event-scalar-equal", group,
            event="measured", field="value", occurrence="last", atol=2.0, rtol=2e-5),
        relation("gas-extensive-conservation", "case-elements-conserved", group,
            elements=["N"], before_op="heat"),
    ]
    return manifest("gas-boundaries", cases, relations)


def cell_script(cu_mol: float = 0.01, zn_mol: float = 0.01, reverse: bool = False) -> tuple[str, ...]:
    lines = ("add v1 water 200mL", f"add v1 ZnSO4 {zn_mol:g}mol", "add v1 Zn 0.05mol",
             "new", "add v2 water 200mL", f"add v2 CuSO4 {cu_mol:g}mol", "add v2 Cu 0.05mol")
    return lines + (("cell v2 v1",) if reverse else ("cell v1 v2",))


def electrochemistry() -> dict:
    cases: list[dict] = []
    relations: list[dict] = []
    group = []
    for number, cu in zip(range(291, 295), (0.0005, 0.001, 0.005, 0.01)):
        group.append(case(number, f"daniell-copper-{cu:g}",
            "How does copper-ion amount shift the open-circuit Daniell voltage?", *cell_script(cu_mol=cu)))
    cases += group
    relations.append(relation("ec-copper-activity-voltage", "event-scalar-order", group,
        event="cell_voltage", field="volts", occurrence="last", direction="increasing", min_delta=1e-4))

    group = []
    for number, zn in zip(range(295, 299), (0.0005, 0.001, 0.005, 0.01)):
        group.append(case(number, f"daniell-zinc-{zn:g}",
            "How does zinc-ion amount shift the open-circuit Daniell voltage?",
            *cell_script(cu_mol=0.008, zn_mol=zn)))
    cases += group
    relations.append(relation("ec-zinc-activity-voltage", "event-scalar-order", group,
        event="cell_voltage", field="volts", occurrence="last", direction="decreasing", min_delta=1e-4))

    group = [
        case(299, "electrolysis-charge-fast", "Does charge, rather than current alone, govern ideal electrolysis products?",
            "add v1 water 250mL", "add v1 KNO3 0.001mol", "electrolyse v1 0.2A 60s", "inspect v1"),
        case(300, "electrolysis-charge-slow", "Does half the current for twice the time give the same products?",
            "add v1 water 250mL", "add v1 KNO3 0.001mol", "electrolyse v1 0.1A 120s", "inspect v1"),
        case(301, "electrolysis-charge-split", "Do two equal charge intervals give the same cumulative products?",
            "add v1 water 250mL", "add v1 KNO3 0.001mol", "electrolyse v1 0.2A 30s", "electrolyse v1 0.2A 30s", "inspect v1"),
        case(302, "electrolysis-charge-unequal", "Do unequal intervals with equal total charge give the same products?",
            "add v1 water 250mL", "add v1 KNO3 0.001mol", "electrolyse v1 0.2A 20s", "electrolyse v1 0.2A 40s", "inspect v1"),
    ]
    cases += group
    relations.append(relation("ec-equal-charge-inventory", "final-inventory-equal", group,
        species=["H2", "O2", "water", "K+", "NO3-"], phases=["Liquid", "Gas"], atol=1e-9, rtol=1e-7))

    group = []
    for number, seconds in zip(range(303, 307), (15, 30, 60, 120)):
        group.append(case(number, f"electrolysis-time-{seconds}",
            "How does electrolysis duration change product inventory at fixed current?",
            "add v1 water 250mL", "add v1 Na2SO4 0.001mol",
            f"electrolyse v1 0.2A {seconds}s", "inspect v1"))
    cases += group
    relations += [
        relation("ec-time-hydrogen-order", "event-scalar-order", group,
            event="electrolysed", field="moles", occurrence="last", direction="increasing", min_delta=1e-7),
        relation("ec-time-element-conservation", "case-elements-conserved", group,
            elements=["Na", "S"], before_op="electrolyse"),
    ]

    group = []
    for number, current in zip(range(307, 311), (0.05, 0.1, 0.2, 0.4)):
        group.append(case(number, f"electrolysis-current-{current:g}",
            "How does imposed current change product inventory over fixed time?",
            "add v1 water 250mL", "add v1 Na2SO4 0.001mol",
            f"electrolyse v1 {current:g}A 75s", "inspect v1"))
    cases += group
    relations.append(relation("ec-current-hydrogen-order", "event-scalar-order", group,
        event="electrolysed", field="moles", occurrence="last", direction="increasing", min_delta=1e-7))

    group = []
    for number, scale in zip(range(311, 315), (1, 2, 3, 4)):
        group.append(case(number, f"electrolysis-extensive-{scale}",
            "Does scaling water, electrolyte, and charge preserve elemental accounting?",
            f"add v1 water {110 * scale}mL", f"add v1 KNO3 {0.00045 * scale:g}mol",
            f"electrolyse v1 0.2A {28 * scale}s", "inspect v1"))
    cases += group
    relations.append(relation("ec-extensive-conservation", "case-elements-conserved", group,
        elements=["K", "N"], before_op="electrolyse"))
    return manifest("electrochemistry", cases, relations)


def mixing_transfer() -> dict:
    cases: list[dict] = []
    relations: list[dict] = []
    group = [
        case(315, "water-mix-hot-first", "Does hot/cold input order alter the equilibrium mixing temperature?",
            "add v1 water 100mL @ 80C", "new", "add v2 water 100mL @ 20C", "new", "mix v1 1 v2 1 into v3", "measure v3 thermometer"),
        case(316, "water-mix-cold-first", "Does reversing hot/cold vessel order alter the mixing temperature?",
            "add v1 water 100mL @ 20C", "new", "add v2 water 100mL @ 80C", "new", "mix v1 1 v2 1 into v3", "measure v3 thermometer"),
        case(317, "water-mix-halves", "Does halving both source portions preserve the mixing temperature?",
            "add v1 water 200mL @ 80C", "new", "add v2 water 200mL @ 20C", "new", "mix v1 0.5 v2 0.5 into v3", "measure v3 thermometer"),
        case(318, "water-mix-extensive", "Does doubling both equal portions preserve the mixing temperature?",
            "add v1 water 200mL @ 80C", "new", "add v2 water 200mL @ 20C", "new", "mix v1 1 v2 1 into v3", "measure v3 thermometer"),
    ]
    cases += group
    relations.append(relation("mix-equal-water-temperature", "event-scalar-equal", group,
        event="measured", field="value", occurrence="last", atol=0.03, rtol=0.0))

    group = []
    for number, hot_fraction in zip(range(319, 323), (0.2, 0.4, 0.6, 0.8)):
        group.append(case(number, f"water-mix-hot-fraction-{hot_fraction:g}",
            "How does the hot-water share move the final mixing temperature?",
            "add v1 water 200mL @ 80C", "new", "add v2 water 200mL @ 20C", "new",
            f"mix v1 {hot_fraction:g} v2 {1-hot_fraction:g} into v3", "measure v3 thermometer"))
    cases += group
    relations.append(relation("mix-hot-share-order", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=1.0))

    group = [
        case(323, "salt-transfer-whole", "Does one whole transfer preserve the system's elemental inventory?",
            "add v1 water 300mL", "add v1 NaCl 0.003mol", "new", "decant v1 v2 1", "inspect"),
        case(324, "salt-transfer-two-halves", "Do two successive half transfers preserve total elemental inventory?",
            "add v1 water 300mL", "add v1 NaCl 0.003mol", "new", "decant v1 v2 0.5", "decant v1 v2 1", "inspect"),
        case(325, "salt-transfer-split-solute", "Does splitting solute addition alter conserved inventory during transfer?",
            "add v1 water 300mL", "add v1 NaCl 0.001mol", "add v1 NaCl 0.002mol", "new", "decant v1 v2 1", "inspect"),
        case(326, "salt-transfer-solute-first", "Does adding solute before solvent alter conserved inventory during transfer?",
            "add v1 NaCl 0.003mol", "add v1 water 300mL", "new", "decant v1 v2 1", "inspect"),
    ]
    cases += group
    relations.append(relation("mix-transfer-system-elements", "final-elements-equal", group,
        elements=["H", "O", "Na", "Cl"], atol=1e-8, rtol=1e-7))

    group = [
        case(327, "mix-salt-left", "Does source order alter a mixed saline inventory?",
            "add v1 water 150mL", "add v1 NaCl 0.002mol", "new", "add v2 water 250mL", "new", "mix v1 1 v2 1 into v3", "inspect"),
        case(328, "mix-salt-right", "Does moving the same salt to the other source alter mixed inventory?",
            "add v1 water 150mL", "new", "add v2 water 250mL", "add v2 NaCl 0.002mol", "new", "mix v1 1 v2 1 into v3", "inspect"),
        case(329, "mix-salt-split", "Does splitting salt between sources alter mixed inventory?",
            "add v1 water 150mL", "add v1 NaCl 0.0008mol", "new", "add v2 water 250mL", "add v2 NaCl 0.0012mol", "new", "mix v1 1 v2 1 into v3", "inspect"),
        case(330, "mix-salt-portioned", "Does pre-portioning both sources alter the same mixed inventory?",
            "add v1 water 300mL", "add v1 NaCl 0.001mol", "new", "add v2 water 500mL", "add v2 NaCl 0.003mol", "new", "mix v1 0.5 v2 0.5 into v3", "inspect"),
    ]
    cases += group
    relations.append(relation("mix-salt-path-independence", "final-inventory-equal", group,
        species=["water", "Na+", "Cl-"], phases=["Liquid"], atol=1e-8, rtol=1e-7))

    group = []
    for number, salt in zip(range(331, 335), (0.0, 0.001, 0.002, 0.004)):
        lines = ["add v1 water 200mL"]
        if salt:
            lines.append(f"add v1 NaCl {salt:g}mol")
        lines += ["new", "add v2 water 200mL", "new", "mix v1 1 v2 1 into v3", "measure v3 conductivity"]
        group.append(case(number, f"mix-conductivity-{salt:g}",
            "How does conserved electrolyte amount affect conductivity after mixing?", *lines))
    cases += group
    relations.append(relation("mix-conductivity-order", "event-scalar-order", group,
        event="measured", field="value", occurrence="last", direction="increasing", min_delta=1e-8))

    group = []
    for number, scale in zip(range(335, 339), (1, 2, 3, 4)):
        group.append(case(number, f"mix-extensive-{scale}",
            "Does extensive scaling preserve the temperature of an unequal thermal mixture?",
            f"add v1 water {50 * scale}mL @ 70C", "new", f"add v2 water {100 * scale}mL @ 25C",
            "new", "mix v1 1 v2 1 into v3", "measure v3 thermometer"))
    cases += group
    relations += [
        relation("mix-extensive-temperature", "event-scalar-equal", group,
            event="measured", field="value", occurrence="last", atol=0.03, rtol=0.0),
        relation("mix-extensive-elements", "case-elements-conserved", group,
            elements=["H", "O"], before_op="mix"),
    ]
    return manifest("mixing-transfer", cases, relations)


def manifest(family: str, cases: list[dict], relations: list[dict]) -> dict:
    assert len(cases) == 24, (family, len(cases))
    return {
        "schema": "kerotakis-source-fleet-v1",
        "family": family,
        "cases": cases,
        "relations": relations,
    }


def generated() -> dict[str, dict]:
    manifests = [acid_base(), solubility(), gases(), electrochemistry(), mixing_transfer()]
    return {item["family"]: item for item in manifests}


def encoded(document: dict) -> str:
    return json.dumps(document, indent=2, ensure_ascii=False) + "\n"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true", help="write the five manifests")
    mode.add_argument("--check", action="store_true", help="verify committed manifests are current")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    stale = []
    for family, document in generated().items():
        path = OUT / f"{family}.json"
        content = encoded(document)
        if args.write:
            path.write_text(content)
        elif not path.exists() or path.read_text() != content:
            stale.append(str(path.relative_to(HERE.parents[1])))
    if stale:
        raise SystemExit("stale or missing generated manifests: " + ", ".join(stale))
    print(json.dumps({"families": 5, "cases": 120, "status": "written" if args.write else "current"}))


if __name__ == "__main__":
    main()

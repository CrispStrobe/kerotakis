#!/usr/bin/env python3
"""Generate frozen source-informed manifests 339--458.

The prose and experiment design here are original.  This generator contains no
source references and must be run before observing engine output.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path


OUT = Path(__file__).with_name("source-fleets")
SCHEMA = "kerotakis-source-fleet-v1"


def case(number: int, slug: str, question: str, script: str) -> dict:
    return {"id": f"{number}-{slug}", "question": question, "script": script}


def relation(number: int, slug: str, assertion_name: str, cases: list[dict], description: str,
             **parameters) -> dict:
    kind = ("conservation" if assertion_name in
            ("final-elements-equal", "case-elements-conserved") else "metamorphic")
    item = {"id": f"r{number:03d}-{slug}", "kind": kind,
            "cases": [entry["id"] for entry in cases],
            "assertion": assertion_name, "description": description,
            "parameters": parameters}
    return item


def paired_family(family: str, title: str, entries: list[dict], specs: list[tuple]) -> dict:
    assert len(entries) == 24 and len(specs) == 12
    relations = []
    for offset, spec in enumerate(specs):
        slug, assertion_name, description, parameters = spec
        relations.append(relation(int(entries[0]["id"].split("-", 1)[0]),
                                  f"{offset + 1:02d}-{slug}", assertion_name,
                                  entries[2 * offset:2 * offset + 2], description,
                                  **parameters))
    return {"schema": SCHEMA, "family": family, "title": title,
            "cases": entries, "relations": relations}


def filtration_evaporation() -> dict:
    c = []
    c += [case(339, "silver-filter-chloride-first", "Does filtering preserve the complete silver and chlorine ledger after chloride is added first?", "add v1 water 0.30L\nadd v1 NaCl 0.0012mol\nadd v1 AgNO3 0.0012mol\nnew\nfilter v1 v2"),
          case(340, "silver-filter-silver-first", "Does reversing the two feeds preserve the same complete silver and chlorine ledger?", "add v1 water 0.30L\nadd v1 AgNO3 0.0012mol\nadd v1 NaCl 0.0012mol\nnew\nfilter v1 v2")]
    c += [case(341, "barium-filter-sulfate-first", "Does a sulfate-first preparation conserve barium and sulfur through filtration?", "add v1 water 0.45L\nadd v1 Na2SO4 0.0007mol\nadd v1 BaCl2 0.0011mol\nnew\nfilter v1 v2"),
          case(342, "barium-filter-barium-first", "Does feed order leave the unequal barium sulfate separation unchanged?", "add v1 water 0.45L\nadd v1 BaCl2 0.0011mol\nadd v1 Na2SO4 0.0007mol\nnew\nfilter v1 v2")]
    c += [case(343, "sand-salt-filter-unit", "How are silica and dissolved potassium chloride divided by one filtration?", "add v1 water 0.25L\nadd v1 SiO2 0.004mol\nadd v1 KCl 0.002mol\nnew\nfilter v1 v2"),
          case(344, "sand-salt-filter-triple", "Does tripling every extensive input triple both separated inventories?", "add v1 water 0.75L\nadd v1 SiO2 0.012mol\nadd v1 KCl 0.006mol\nnew\nfilter v1 v2")]
    c += [case(345, "brine-evaporation-single", "What state follows removal of forty percent of the water from dilute brine?", "add v1 water 0.40L\nadd v1 NaCl 0.030mol\nevaporate v1 0.4"),
          case(346, "brine-evaporation-partitioned", "Do two successive water-removal steps matching the same retained fraction reach the same state?", "add v1 water 0.40L\nadd v1 NaCl 0.030mol\nevaporate v1 0.2\nevaporate v1 0.25")]
    c += [case(347, "potassium-brine-half", "Does half evaporation retain all potassium and chlorine in the vessel?", "add v1 water 0.32L\nadd v1 KCl 0.045mol\nevaporate v1 0.5"),
          case(348, "potassium-brine-two-halves", "Does splitting the same retained-water fraction preserve the potassium chloride result?", "add v1 water 0.32L\nadd v1 KCl 0.045mol\nevaporate v1 0.2928932188\nevaporate v1 0.2928932188")]
    c += [case(349, "nitrate-evaporation-unit", "Does evaporation remove solvent without consuming sodium nitrate?", "add v1 water 0.20L\nadd v1 NaNO3 0.010mol\nevaporate v1 0.6"),
          case(350, "nitrate-evaporation-fourfold", "Does fourfold extensive scaling preserve phase fractions while scaling inventory?", "add v1 water 0.80L\nadd v1 NaNO3 0.040mol\nevaporate v1 0.6")]
    c += [case(351, "dilute-sugar-filter", "Does a filter pass dissolved glucose while conserving carbon?", "add v1 water 0.28L\nadd v1 glucose 0.003mol\nnew\nfilter v1 v2"),
          case(352, "dilute-sugar-filter-reordered", "Does creating the receiver before dissolving sugar change the filtered result?", "new\nadd v1 water 0.28L\nadd v1 glucose 0.003mol\nfilter v1 v2")]
    c += [case(353, "silver-filter-dilute", "Does extra solvent alter total silver and chlorine recovered across a filter?", "add v1 water 0.90L\nadd v1 AgNO3 0.0005mol\nadd v1 NaCl 0.0004mol\nnew\nfilter v1 v2"),
          case(354, "silver-filter-concentrated", "Does less solvent preserve the same total elemental ledger across filtration?", "add v1 water 0.30L\nadd v1 AgNO3 0.0005mol\nadd v1 NaCl 0.0004mol\nnew\nfilter v1 v2")]
    c += [case(355, "brine-evaporate-mild", "Does a mild evaporation retain the complete sodium and chlorine inventory?", "add v1 water 0.36L\nadd v1 NaCl 0.055mol\nevaporate v1 0.25"),
          case(356, "brine-evaporate-deep", "Does deeper evaporation retain the same nonvolatile elemental inventory?", "add v1 water 0.36L\nadd v1 NaCl 0.055mol\nevaporate v1 0.75")]
    c += [case(357, "mixed-salts-filter", "Can filtration conserve calcium, potassium, chloride and nitrate in a clear mixed solution?", "add v1 water 0.50L\nadd v1 CaCl2 0.001mol\nadd v1 KNO3 0.0015mol\nnew\nfilter v1 v2"),
          case(358, "mixed-salts-filter-feed-reverse", "Does reversing soluble-salt feeds preserve the filtered inventories?", "add v1 water 0.50L\nadd v1 KNO3 0.0015mol\nadd v1 CaCl2 0.001mol\nnew\nfilter v1 v2")]
    c += [case(359, "silica-brine-filter-then-evaporate", "What remains after retaining silica and concentrating the salt-bearing filtrate?", "add v1 water 0.30L\nadd v1 SiO2 0.003mol\nadd v1 NaCl 0.020mol\nnew\nfilter v1 v2\nevaporate v2 0.5"),
          case(360, "silica-brine-scaled-process", "Does doubling the full filter-and-evaporate process double every recovered element?", "add v1 water 0.60L\nadd v1 SiO2 0.006mol\nadd v1 NaCl 0.040mol\nnew\nfilter v1 v2\nevaporate v2 0.5")]
    c += [case(361, "barium-filter-before-drying", "Does filtering a barium sulfate preparation conserve its full elemental ledger?", "add v1 water 0.35L\nadd v1 BaCl2 0.0006mol\nadd v1 Na2SO4 0.0006mol\nnew\nfilter v1 v2"),
          case(362, "barium-filter-and-dry-filtrate", "Does drying only the filtrate leave the combined elemental ledger unchanged?", "add v1 water 0.35L\nadd v1 BaCl2 0.0006mol\nadd v1 Na2SO4 0.0006mol\nnew\nfilter v1 v2\nevaporate v2 1.0")]
    specs = [
        ("silver-feed-order", "final-elements-equal", "Feed order must preserve the complete two-vessel silver and chlorine ledger.", {"elements": ["Ag", "Cl"]}),
        ("barium-feed-order", "final-elements-equal", "Feed order must not change separated elemental totals.", {"elements": ["Ba", "S", "Cl", "Na"]}),
        ("sand-salt-scale", "final-elements-equal", "All selected elemental totals must scale by three.", {"elements": ["Si", "K", "Cl"], "scale": 3.0}),
        ("brine-step-partition", "final-inventory-equal", "Equivalent retained-water fractions must produce the same final inventory.", {"species": ["water", "NaCl"], "rtol": 1e-8, "atol": 1e-10}),
        ("potassium-step-partition", "final-inventory-equal", "Partitioning evaporation must preserve the final material state.", {"species": ["water", "KCl"], "rtol": 1e-7, "atol": 1e-10}),
        ("nitrate-scale", "final-elements-equal", "Fourfold input scale must give fourfold elemental totals.", {"elements": ["Na", "N"], "scale": 4.0}),
        ("dissolved-filter-setup", "final-inventory-equal", "Receiver creation order must not affect dissolved-solute transfer.", {"species": ["glucose", "water"]}),
        ("silver-dilution-ledger", "final-elements-equal", "Solvent dilution must not alter supplied silver or chlorine totals.", {"elements": ["Ag", "Cl"]}),
        ("evaporation-nonvolatile-ledger", "final-elements-equal", "Evaporation depth must not alter sodium or chlorine totals.", {"elements": ["Na", "Cl"]}),
        ("soluble-feed-order", "final-inventory-equal", "Soluble feed order must not affect the filtered state.", {"species": ["CaCl2", "KNO3"]}),
        ("combined-process-scale", "final-elements-equal", "The complete separation path must scale by two.", {"elements": ["Si", "Na", "Cl"], "scale": 2.0}),
        ("filtrate-drying-conservation", "final-elements-equal", "Removing water from one output must preserve combined non-hydrogen and non-oxygen totals.", {"elements": ["Ba", "S", "Cl", "Na"]}),
    ]
    return paired_family("filtration-evaporation", "Filtration and evaporation", c, specs)


def distillation() -> dict:
    mixtures = [
        ("ethanol", 0.24, "methanol", 0.08), ("methanol", 0.18, "isopropanol", 0.07),
        ("ethanol", 0.13, "isopropanol", 0.11), ("ethanol", 0.20, "methanol", 0.05),
        ("methanol", 0.14, "isopropanol", 0.09), ("ethanol", 0.16, "isopropanol", 0.06),
    ]
    c, specs = [], []
    number = 363
    for i, (a, na, b, nb) in enumerate(mixtures):
        base = f"add v1 water {3.0 + i * .2:.1f}mol\nadd v1 {a} {na}mol\nadd v1 {b} {nb}mol"
        rev = f"add v1 {b} {nb}mol\nadd v1 water {3.0 + i * .2:.1f}mol\nadd v1 {a} {na}mol"
        x = case(number, f"still-order-{i + 1}-forward", f"Does ternary still path {i + 1} conserve all volatile components?", base + "\nnew\ndistil v1 v2 0.14")
        y = case(number + 1, f"still-order-{i + 1}-reverse", f"Does reordered feed {i + 1} produce the same distillation cut?", rev + "\nnew\ndistil v1 v2 0.14")
        c += [x, y]; number += 2
        specs.append((f"feed-order-{i + 1}", "final-inventory-equal", "Feed order must not change either vessel after the same cut.", {"species": ["water", a, b], "rtol": 1e-8, "atol": 1e-10}))
    for i, (a, na, b, nb) in enumerate(mixtures):
        water = 2.8 + i * .25
        base = f"add v1 water {water}mol\nadd v1 {a} {na}mol\nadd v1 {b} {nb}mol\nnew\n"
        if i < 3:
            x = case(number, f"still-duty-{i + 1}-low", f"How much does a low latent-energy budget transfer in still trial {i + 1}?", base + f"distil v1 v2 {1.0 + i * .4}kJ stages 2")
            y = case(number + 1, f"still-duty-{i + 1}-high", f"Does a larger latent-energy budget transfer more in trial {i + 1}?", base + f"distil v1 v2 {2.0 + i * .8}kJ stages 2")
            specs.append((f"energy-monotone-{i + 1}", "event-components-total-order", "A larger latent duty must transfer more total material.", {"event": "distilled", "occurrence": "last", "direction": "increasing", "min_delta": 1e-12}))
        else:
            frac = .08 + (i - 3) * .02
            x = case(number, f"still-fraction-{i + 1}-low", f"What receiver inventory follows the smaller cut in trial {i + 1}?", base + f"distil v1 v2 {frac}")
            y = case(number + 1, f"still-fraction-{i + 1}-high", f"Does a larger requested cut transfer more material in trial {i + 1}?", base + f"distil v1 v2 {frac * 2}")
            specs.append((f"fraction-monotone-{i + 1}", "event-components-total-order", "A larger requested fraction must transfer more total material.", {"event": "distilled", "occurrence": "last", "direction": "increasing", "min_delta": 1e-12}))
        c += [x, y]; number += 2
    return paired_family("distillation", "Distillation", c, specs)


def thermal_phase() -> dict:
    c, specs = [], []
    number = 387
    for i in range(4):
        water, energy = .20 + .05 * i, 900 + 200 * i
        x = case(number, f"water-heat-single-{i + 1}", f"What temperature follows one heat delivery in calorimetry trial {i + 1}?", f"add v1 water {water}L\nheat v1 {energy}J\nmeasure v1 temp")
        y = case(number + 1, f"water-heat-split-{i + 1}", f"Does splitting the same heat delivery preserve temperature in trial {i + 1}?", f"add v1 water {water}L\nheat v1 {energy * .4}J\nheat v1 {energy * .6}J\nmeasure v1 temp")
        c += [x, y]; number += 2
        specs.append((f"heat-partition-{i + 1}", "final-scalar-equal", "Partitioning heat input must preserve final temperature.", {"path": "v1.temperature", "rtol": 1e-10, "atol": 1e-8}))
    for i in range(4):
        hot, cold = 1200 + i * 150, 500 + i * 100
        prefix = f"add v1 water {0.18 + i*.02}L\nheat v1 {hot}J\nnew\nadd v2 water {0.27 + i*.03}L\ncool v2 {cold}J\nnew\n"
        x = case(number, f"thermal-mix-forward-{i + 1}", f"What equilibrium temperature follows hot-first mixing trial {i + 1}?", prefix + "mix v1 1 v2 1 into v3\nmeasure v3 temp")
        y = case(number + 1, f"thermal-mix-reverse-{i + 1}", f"Does operand reversal preserve the mixed temperature in trial {i + 1}?", prefix + "mix v2 1 v1 1 into v3\nmeasure v3 temp")
        c += [x, y]; number += 2
        specs.append((f"mix-order-{i + 1}", "final-scalar-equal", "Mix operand order must not change receiver temperature.", {"path": "v3.temperature", "rtol": 1e-10, "atol": 1e-8}))
    for i in range(4):
        water, energy = .24 + i * .03, 400 + i * 100
        x = case(number, f"thermal-roundtrip-{i + 1}", f"Does equal heating and cooling recover the initial thermal state in trial {i + 1}?", f"add v1 water {water}L\nheat v1 {energy}J\ncool v1 {energy}J\nmeasure v1 temp")
        y = case(number + 1, f"thermal-blank-{i + 1}", f"What reference temperature does untouched water have in trial {i + 1}?", f"add v1 water {water}L\nmeasure v1 temp")
        c += [x, y]; number += 2
        specs.append((f"energy-roundtrip-{i + 1}", "final-scalar-equal", "Equal positive and negative energy transfers must recover temperature.", {"path": "v1.temperature", "rtol": 1e-10, "atol": 1e-8}))
    return paired_family("thermal-phase-paths", "Thermal and phase paths", c, specs)


def organic_equilibria() -> dict:
    feeds = [
        (.013, .009, .002, .005), (.011, .016, .003, .004), (.007, .010, .014, .012),
        (.018, .006, .001, .009), (.008, .012, .020, .007), (.021, .015, .004, .003),
    ]
    c, specs, number = [], [], 411
    for i, amounts in enumerate(feeds):
        names = ["CH3COOH", "ethanol", "ethyl_acetate", "water"]
        forward = "\n".join(f"add v1 {s} {n}mol" for s, n in zip(names, amounts))
        reverse = "\n".join(f"add v1 {s} {n}mol" for s, n in reversed(list(zip(names, amounts))))
        x = case(number, f"ester-feed-forward-{i + 1}", f"What equilibrium follows organic feed arrangement {i + 1}?", forward + "\nreact v1 esterification")
        y = case(number + 1, f"ester-feed-reverse-{i + 1}", f"Does reversing all feeds preserve organic equilibrium {i + 1}?", reverse + "\nreact v1 esterification")
        c += [x, y]; number += 2
        specs.append((f"organic-feed-order-{i + 1}", "final-inventory-equal", "Feed order must preserve the equilibrium inventory.", {"species": names, "rtol": 1e-8, "atol": 1e-10}))
    for i, amounts in enumerate(feeds):
        names = ["CH3COOH", "ethanol", "ethyl_acetate", "water"]
        base = "\n".join(f"add v1 {s} {n}mol" for s, n in zip(names, amounts))
        if i < 3:
            scaled = "\n".join(f"add v1 {s} {n * (i + 2)}mol" for s, n in zip(names, amounts))
            x = case(number, f"ester-scale-base-{i + 1}", f"What equilibrium extent follows base-scale organic trial {i + 1}?", base + "\ninspect v1\nreact v1 esterification")
            y = case(number + 1, f"ester-scale-large-{i + 1}", f"Does extensive scaling preserve the intensive equilibrium in trial {i + 1}?", scaled + "\nreact v1 esterification")
            specs.append((f"organic-scale-{i + 1}", "final-inventory-equal", "Equilibrium inventories must scale with all starting amounts.", {"species": names, "scale": float(i + 2), "rtol": 1e-8, "atol": 1e-10}))
        else:
            x = case(number, f"ester-once-{i + 1}", f"What state follows one equilibrium request in organic trial {i + 1}?", base + "\ninspect v1\nreact v1 esterification")
            y = case(number + 1, f"ester-repeat-{i + 1}", f"Does a second equilibrium request leave organic trial {i + 1} unchanged?", base + "\nreact v1 esterification\nreact v1 esterification")
            specs.append((f"organic-idempotence-{i + 1}", "final-inventory-equal", "A repeated equilibrium request must be a material no-op.", {"species": names, "rtol": 1e-8, "atol": 1e-10}))
        c += [x, y]; number += 2
    return paired_family("organic-equilibria", "Organic equilibria", c, specs)


def observables_controls() -> dict:
    c = [
        case(435, "conductivity-nitrate-once", "What conductivity is reported for a dilute potassium nitrate solution?", "add v1 water 0.40L\nadd v1 KNO3 0.00012mol\nmeasure v1 conductivity"),
        case(436, "conductivity-nitrate-repeat", "Does immediately repeating a conductivity reading leave it unchanged?", "add v1 water 0.40L\nadd v1 KNO3 0.00012mol\nmeasure v1 conductivity\nmeasure v1 conductivity"),
        case(437, "ph-acid-once", "What pH is reported for a finite dilute hydrochloric acid dose?", "add v1 water 0.55L\nadd v1 HCl 0.00011mol\nmeasure v1 ph"),
        case(438, "ph-acid-repeat", "Does an immediate second pH reading reproduce the first?", "add v1 water 0.55L\nadd v1 HCl 0.00011mol\nmeasure v1 ph\nmeasure v1 ph"),
        case(439, "temperature-water-once", "What temperature is reported after a defined heat input?", "add v1 water 0.33L\nheat v1 330J\nmeasure v1 temp"),
        case(440, "temperature-water-repeat", "Does observing temperature twice avoid changing the thermal state?", "add v1 water 0.33L\nheat v1 330J\nmeasure v1 temp\nmeasure v1 temp"),
        case(441, "pressure-nitrogen-once", "What pressure does a sealed finite nitrogen inventory establish?", "seal v1 0.42L\nadd v1 N2 0.0031mol\nmeasure v1 pressure"),
        case(442, "pressure-nitrogen-repeat", "Does repeating a pressure reading preserve the same gas state?", "seal v1 0.42L\nadd v1 N2 0.0031mol\nmeasure v1 pressure\nmeasure v1 pressure"),
        case(443, "balance-water-once", "What mass does the balance report for this water portion?", "add v1 water 0.21L\nmeasure v1 balance"),
        case(444, "balance-water-repeat", "Does a second balance reading leave mass unchanged?", "add v1 water 0.21L\nmeasure v1 balance\nmeasure v1 balance"),
        case(445, "conductivity-water-blank", "Does plain water provide an explicit conductivity blank?", "add v1 water 0.37L\nmeasure v1 conductivity"),
        case(446, "conductivity-glucose-control", "Does neutral glucose avoid impersonating a strong electrolyte?", "add v1 water 0.37L\nadd v1 glucose 0.00010mol\nmeasure v1 conductivity"),
        case(447, "look-water-before-wait", "What visible features are reported for plain water before waiting?", "add v1 water 0.29L\nlook v1"),
        case(448, "look-water-after-wait", "Does waiting alone avoid inventing a visible change in plain water?", "add v1 water 0.29L\nwait 2s\nlook v1"),
        case(449, "filter-glucose-before", "What carbon inventory is present before filtering dissolved glucose?", "add v1 water 0.31L\nadd v1 glucose 0.0009mol\nnew\nfilter v1 v2"),
        case(450, "filter-glucose-after-wait", "Does waiting before filtering invent a retained glucose solid?", "add v1 water 0.31L\nadd v1 glucose 0.0009mol\nwait 3s\nnew\nfilter v1 v2"),
        case(451, "inert-potassium-before-ignite", "What mass and temperature describe potassium chloride before an ignition request?", "add v1 KCl 0.003mol\nmeasure v1 balance\nmeasure v1 temp"),
        case(452, "inert-potassium-after-ignite", "Does an ignition request avoid consuming inert potassium chloride?", "add v1 KCl 0.003mol\nignite v1\nmeasure v1 balance\nmeasure v1 temp"),
        case(453, "dry-nitrogen-before-wait", "What pressure is observed for sealed nitrogen before waiting?", "seal v1 0.48L\nadd v1 N2 0.0036mol\nmeasure v1 pressure"),
        case(454, "dry-nitrogen-after-wait", "Does waiting without a process leave sealed nitrogen pressure unchanged?", "seal v1 0.48L\nadd v1 N2 0.0036mol\nwait 4s\nmeasure v1 pressure"),
        case(455, "water-balance-before-filter", "What total mass enters a filter when only water is present?", "add v1 water 0.26L\nnew\nmeasure v1 balance\nfilter v1 v2"),
        case(456, "water-balance-after-filter", "Does filtering plain water conserve combined vessel mass?", "add v1 water 0.26L\nnew\nfilter v1 v2\nmeasure v2 balance"),
        case(457, "glucose-ph-before-wait", "What pH is reported for a neutral glucose control?", "add v1 water 0.43L\nadd v1 glucose 0.0002mol\nmeasure v1 ph"),
        case(458, "glucose-ph-after-wait", "Does waiting alone avoid generating acidity in the glucose control?", "add v1 water 0.43L\nadd v1 glucose 0.0002mol\nwait 5s\nmeasure v1 ph"),
    ]
    specs = [
        ("conductivity-repeat", "event-scalar-equal", "Repeated conductivity readings must agree.", {"event": "measured", "field": "value", "occurrence": "first,last", "atol": 1e-12, "rtol": 1e-10}),
        ("ph-repeat", "event-scalar-equal", "Repeated pH readings must agree.", {"event": "measured", "field": "value", "occurrence": "first,last", "atol": 1e-10, "rtol": 1e-10}),
        ("temperature-repeat", "event-scalar-equal", "Repeated temperature readings must agree.", {"event": "measured", "field": "value", "occurrence": "first,last", "atol": 1e-9, "rtol": 1e-10}),
        ("pressure-repeat", "event-scalar-equal", "Repeated pressure readings must agree.", {"event": "measured", "field": "value", "occurrence": "first,last", "atol": 1e-6, "rtol": 1e-10}),
        ("balance-repeat", "event-scalar-equal", "Repeated balance readings must agree.", {"event": "measured", "field": "value", "occurrence": "first,last", "atol": 1e-9, "rtol": 1e-10}),
        ("neutral-solute-conductivity", "event-scalar-order", "Neutral glucose must not exceed the water blank as though fully ionic.", {"event": "measured", "field": "value", "occurrence": "last", "direction": "decreasing", "min_delta": 0.0}),
        ("water-wait-null", "final-inventory-equal", "Waiting must not invent material in plain water.", {"species": ["water"], "rtol": 1e-10, "atol": 1e-12}),
        ("glucose-wait-null", "final-elements-equal", "Waiting must not create a filterable carbon-bearing phase.", {"elements": ["C", "H", "O"]}),
        ("inert-ignite-null", "final-elements-equal", "An unsupported combustion path must not consume potassium chloride.", {"elements": ["K", "Cl"], "rtol": 1e-10, "atol": 1e-12}),
        ("gas-wait-null", "final-scalar-equal", "Waiting without a process must preserve sealed-gas pressure.", {"path": "v1.pressure", "rtol": 1e-10, "atol": 1e-6}),
        ("water-filter-mass", "final-elements-equal", "Filtering plain water must conserve hydrogen and oxygen across vessels.", {"elements": ["H", "O"]}),
        ("glucose-ph-wait-null", "event-scalar-equal", "Waiting alone must not alter neutral-solute pH.", {"event": "measured", "field": "value", "occurrence": "last", "rtol": 1e-10, "atol": 1e-9}),
    ]
    return paired_family("observables-negative-controls", "Observables and negative controls", c, specs)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    manifests = [filtration_evaporation(), distillation(), thermal_phase(),
                 organic_equilibria(), observables_controls()]
    OUT.mkdir(parents=True, exist_ok=True)
    stale = []
    for manifest in manifests:
        path = OUT / f"{manifest['family']}.json"
        content = json.dumps(manifest, indent=2, ensure_ascii=False) + "\n"
        if args.write:
            path.write_text(content)
        elif not path.exists() or path.read_text() != content:
            stale.append(str(path))
    if stale:
        raise SystemExit("stale or missing generated manifests: " + ", ".join(stale))
    print(json.dumps({"families": 5, "cases": 120,
                      "status": "written" if args.write else "current"}))


if __name__ == "__main__":
    main()

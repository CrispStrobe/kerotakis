#!/usr/bin/env python3
"""Freeze original source-fleet v2-B manifests, cases 633--776.

Source-to-design research stays private. Questions and scripts in these
manifests are original project text and carry no bibliographic identifiers.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "source-fleets-2"


def case(n: int, slug: str, question: str, *lines: str) -> dict:
    return {"id": f"{n}-{slug}", "question": question, "script": "\n".join(lines)}


def relation(name: str, assertion: str, cases: list[dict], **parameters) -> dict:
    kind = ("conservation" if assertion in {"case-elements-conserved", "final-elements-equal"}
            else "boundary" if assertion in {"event-present", "event-boundary-present"}
            else "independent-law" if assertion.endswith("-order") else "metamorphic")
    return {"id": name, "kind": kind, "assertion": assertion,
            "cases": [c["id"] for c in cases], "parameters": parameters}


def groups(family: str, start: int, specs: list[tuple]) -> dict:
    cases, relations = [], []
    n = start
    for suffix, assertion, params, variants in specs:
        group = []
        for slug, question, lines in variants:
            group.append(case(n, slug, question, *lines)); n += 1
        cases.extend(group)
        relations.append(relation(f"v2b-{family}-{suffix}", assertion, group, **params))
    assert len(cases) == 24 and n == start + 24
    return {"schema": "kerotakis-source-fleet-v2", "family": family,
            "cases": cases, "relations": relations}


def separation_trains() -> dict:
    specs = []
    variants = []
    for scale in (1, 2, 3, 4):
        variants.append((f"sand-brine-scale-{scale}", "Does an extensively scaled filter train retain proportional silica and salt?",
            [f"add v1 water {150*scale}mL", f"add v1 SiO2 {0.001*scale:g}mol", f"add v1 NaCl {0.004*scale:g}mol", "new", "filter v1 v2", "inspect"]))
    specs.append(("filter-scale", "case-elements-conserved", {"elements":["Si","Na","Cl"], "before_op":"filter"}, variants))
    variants=[]
    for i,(a,b) in enumerate(((0.5,None),(0.2,0.375),(0.25,1/3),(0.1,4/9))):
        lines=["add v1 water 400mL","add v1 KCl 0.025mol",f"evaporate v1 {a:g}"]
        if b is not None: lines.append(f"evaporate v1 {b:g}")
        variants.append((f"brine-path-{i+1}","Do evaporation paths with the same retained water reach the same brine state?",lines))
    specs.append(("evaporation-path", "final-inventory-equal", {"species":["water","KCl"],"atol":1e-9,"rtol":1e-7}, variants))
    variants=[]
    for i,order in enumerate((("water","ethanol"),("ethanol","water"),("water","ethanol"),("ethanol","water"))):
        amounts=(3,0.3) if i<2 else (6,0.6)
        lines=[f"add v1 {order[0]} {amounts[0 if order[0]=='water' else 1]}mol",f"add v1 {order[1]} {amounts[0 if order[1]=='water' else 1]}mol","new",f"distil v1 v2 {0.2 if i<2 else 0.1:g}"]
        variants.append((f"still-route-{i+1}","Does feed order leave the same ideal binary separation?",lines))
    specs.append(("still-ledger", "case-elements-conserved", {"elements":["C","H","O"],"before_op":"distil"}, variants))
    variants=[]
    for fraction in (.2,.4,.6,.8):
        variants.append((f"filter-decant-{fraction:g}","How does the decanted share move dissolved salt after filtration?",["add v1 water 300mL","add v1 SiO2 0.002mol","add v1 NaCl 0.006mol","new","filter v1 v2","new",f"decant v2 v3 {fraction:g}","inspect"]))
    specs.append(("decant-conservation", "case-elements-conserved", {"elements":["Si","Na","Cl"],"before_op":"filter"}, variants))
    variants=[]
    for salt in (0.001,0.002,0.004,0.008):
        variants.append((f"filter-salt-{salt:g}","Does a filter conserve dissolved nitrate while retaining silica?",["add v1 water 250mL","add v1 SiO2 0.002mol",f"add v1 NaNO3 {salt:g}mol","new","filter v1 v2","inspect"]))
    specs.append(("nitrate-ledger", "case-elements-conserved", {"elements":["Si","Na","N"],"before_op":"filter"}, variants))
    variants=[]
    for water in (200,300,400,500):
        variants.append((f"dry-filtrate-{water}","What remains when a filtered salt solution is dried?",[f"add v1 water {water}mL","add v1 SiO2 0.002mol","add v1 NaCl 0.01mol","new","filter v1 v2","evaporate v2 1","inspect"]))
    specs.append(("dry-ledger", "case-elements-conserved", {"elements":["Si","Na","Cl"],"before_op":"filter"}, variants))
    return groups("separation-trains",633,specs)


def kinetics_boundaries() -> dict:
    specs=[]
    variants=[(f"peroxide-wait-{s}","Does the current modeled clock conserve an unstirred peroxide sample across this interval?",["add v1 water 200mL","add v1 H2O2 0.01mol",f"wait {s}s","inspect v1"]) for s in (30,60,120,240)]
    specs.append(("peroxide-time","case-elements-conserved",{"elements":["H","O"],"before_op":"wait"},variants))
    variants=[(f"acid-metal-wait-{s}","Does the current modeled clock conserve zinc and chlorine across this interval?",["add v1 water 200mL","add v1 HCl 0.004mol","add v1 Zn 0.002mol",f"wait {s}s","inspect v1"]) for s in (10,30,60,120)]
    specs.append(("metal-time","case-elements-conserved",{"elements":["Zn","Cl"],"before_op":"wait"},variants))
    variants=[(f"peroxide-heat-{j}","Does the current modeled clock conserve peroxide elements after this heat input and interval?",["add v1 water 200mL","add v1 H2O2 0.01mol",f"heat v1 {j}J","wait 60s","inspect v1"]) for j in (0,100,200,300)]
    specs.append(("temperature-ledger","case-elements-conserved",{"elements":["H","O"],"before_op":"wait"},variants))
    variants=[(f"zinc-dose-{m:g}","Does the current modeled clock conserve zinc and chlorine for this starting dose?",["add v1 water 250mL","add v1 HCl 0.006mol",f"add v1 Zn {m:g}mol","wait 60s","inspect v1"]) for m in (.0005,.001,.002,.004)]
    specs.append(("zinc-dose-ledger","case-elements-conserved",{"elements":["Zn","Cl"],"before_op":"wait"},variants))
    variants=[]
    for i,pulses in enumerate(((120,),(60,60),(30,90),(20,40,60))):
        lines=["add v1 water 220mL","add v1 H2O2 0.011mol"]+[f"wait {p}s" for p in pulses]+["inspect v1"]
        variants.append((f"wait-partition-{i+1}","Within the current modeled clock, does partitioning equal elapsed time preserve the peroxide state?",lines))
    specs.append(("time-partition","final-inventory-equal",{"species":["H2O2","water","O2"],"atol":1e-9,"rtol":1e-7},variants))
    variants=[(f"acid-dose-{m:g}","Does the current modeled clock conserve zinc and chlorine for this acid dose?",["add v1 water 250mL",f"add v1 HCl {m:g}mol","add v1 Zn 0.002mol","wait 60s","inspect v1"]) for m in (.0005,.001,.002,.004)]
    specs.append(("acid-dose-ledger","case-elements-conserved",{"elements":["Zn","Cl"],"before_op":"wait"},variants))
    return groups("kinetics-boundaries",657,specs)


def household_biochemistry() -> dict:
    specs=[]
    recipes=[("glucose",.001),("glucose",.002),("sucrose",.001),("sucrose",.002)]
    variants=[(f"sugar-control-{i+1}","Does a neutral household sugar remain a conductivity control?",["add v1 water 250mL",f"add v1 {s} {m:g}mol","measure v1 conductivity"]) for i,(s,m) in enumerate(recipes)]
    specs.append(("sugar-conductivity","event-scalar-equal",{"event":"measured","field":"value","occurrence":"last","atol":1e-6,"rtol":1e-5},variants))
    variants=[(f"vinegar-dilution-{w}","How does dilution move the measured pH of a fixed vinegar amount?",[f"add v1 water {w}mL","add v1 white_vinegar_5_percent 10mL","measure v1 ph"]) for w in (50,100,200,400)]
    specs.append(("vinegar-dilution","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":.02},variants))
    variants=[(f"baking-soda-dose-{m}","How does baking-soda dose shift a fixed vinegar mixture?",["add v1 white_vinegar_5_percent 50mL",f"add v1 baking_soda {m}g","measure v1 ph","inspect v1"]) for m in (1,2,3,4)]
    specs.append(("soda-ph","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":.01},variants))
    variants=[
        ("starch-then-iodine","Does feed order change the elemental ledger of a starch-iodine test?",["add v1 water 200mL","add v1 starch 0.0004mol","add v1 iodine_solution 1mL","inspect v1"]),
        ("iodine-then-starch","Does reversing both feeds change the same test ledger?",["add v1 water 200mL","add v1 iodine_solution 1mL","add v1 starch 0.0004mol","inspect v1"]),
        ("split-starch","Does splitting starch delivery change the same test ledger?",["add v1 water 200mL","add v1 starch 0.0001mol","add v1 iodine_solution 1mL","add v1 starch 0.0003mol","inspect v1"]),
        ("split-iodine","Does splitting iodine delivery change the same test ledger?",["add v1 water 200mL","add v1 iodine_solution 0.4mL","add v1 starch 0.0004mol","add v1 iodine_solution 0.6mL","inspect v1"]),
    ]
    specs.append(("starch-routes","final-elements-equal",{"elements":["C","I","H","O"],"atol":1e-8,"rtol":1e-7},variants))
    variants=[
        ("milk-then-acid","Does feed order change the full ledger of acidified milk?",["add v1 milk 100mL","add v1 white_vinegar_5_percent 5mL","inspect v1"]),
        ("acid-then-milk","Does reversing the feeds change the same acidified-milk ledger?",["add v1 white_vinegar_5_percent 5mL","add v1 milk 100mL","inspect v1"]),
        ("split-acid-milk","Does splitting vinegar delivery change the same ledger?",["add v1 milk 100mL","add v1 white_vinegar_5_percent 2mL","add v1 white_vinegar_5_percent 3mL","inspect v1"]),
        ("split-milk-acid","Does splitting milk delivery change the same ledger?",["add v1 milk 50mL","add v1 white_vinegar_5_percent 5mL","add v1 milk 50mL","inspect v1"]),
    ]
    specs.append(("milk-acid-routes","final-elements-equal",{"elements":["C","H","O"],"atol":1e-8,"rtol":1e-7},variants))
    variants=[(f"lemon-dilution-{w}","How does water dilution change lemon-juice acidity?",[f"add v1 water {w}mL","add v1 lemon_juice 5mL","measure v1 ph"]) for w in (25,50,100,200)]
    specs.append(("lemon-dilution","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":.01},variants))
    return groups("household-biochemistry",681,specs)


def materials_transport() -> dict:
    specs=[]
    variants=[(f"salt-transfer-{f:g}","How does transferred fraction divide a salt solution while conserving its elements?",["add v1 water 300mL","add v1 NaCl 0.006mol","new",f"decant v1 v2 {f:g}","inspect"]) for f in (.2,.4,.6,.8)]
    specs.append(("salt-transfer","case-elements-conserved",{"elements":["H","O","Na","Cl"],"before_op":"decant"},variants))
    variants=[(f"sugar-transfer-{f:g}","How does transferred fraction divide sugar water without creating ions?",["add v1 water 300mL","add v1 glucose 0.004mol","new",f"decant v1 v2 {f:g}","inspect"]) for f in (.25,.5,.75,1)]
    specs.append(("sugar-transfer","case-elements-conserved",{"elements":["C","H","O"],"before_op":"decant"},variants))
    variants=[]
    for i,(a,b) in enumerate(((.5,.5),(.25,.75),(.75,.25),(1,1))):
        variants.append((f"mix-portions-{i+1}","Does mixing selected portions conserve the complete salt-water system?",["add v1 water 200mL","add v1 NaCl 0.002mol","new","add v2 water 200mL","new",f"mix v1 {a:g} v2 {b:g} into v3","inspect"]))
    specs.append(("mix-ledger","case-elements-conserved",{"elements":["H","O","Na","Cl"],"before_op":"mix"},variants))
    variants=[(f"silica-transfer-{f:g}","Does decanting leave settled silica while moving the liquid fraction?",["add v1 water 300mL","add v1 SiO2 0.003mol","new",f"decant v1 v2 {f:g}","inspect"]) for f in (.2,.4,.6,.8)]
    specs.append(("silica-ledger","case-elements-conserved",{"elements":["Si","H","O"],"before_op":"decant"},variants))
    variants=[]
    for i,parts in enumerate(((1,),(.5,1),(.25,.5,1),(.1,.2,.3,1))):
        lines=["add v1 water 300mL","add v1 KCl 0.006mol","new"]+[f"decant v1 v2 {p:g}" for p in parts]+["inspect"]
        variants.append((f"staged-transfer-{i+1}","Do staged transfers conserve potassium chloride across both vessels?",lines))
    specs.append(("staged-ledger","final-elements-equal",{"elements":["K","Cl","H","O"],"atol":1e-8,"rtol":1e-7},variants))
    variants=[(f"thermal-transfer-{f:g}","How does portion size carry heat with transferred water?",["add v1 water 300mL @ 60C","new",f"decant v1 v2 {f:g}","measure v2 thermometer"]) for f in (.2,.4,.6,.8)]
    specs.append(("thermal-intensive","event-scalar-equal",{"event":"measured","field":"value","occurrence":"last","atol":.03,"rtol":0},variants))
    return groups("materials-transport",705,specs)


def analytical_observables() -> dict:
    specs=[]
    for tag,instrument,base,amounts,direction,delta in (
        ("conductivity","conductivity","NaCl",(0,.001,.002,.004),"increasing",1e-8),
        ("acid-ph","ph","HCl",(.00025,.0005,.001,.002),"decreasing",.1),
        ("base-ph","ph","NaOH",(.00025,.0005,.001,.002),"increasing",.1)):
        variants=[]
        for m in amounts:
            lines=["add v1 water 275mL"]
            if m: lines.append(f"add v1 {base} {m:g}mol")
            lines.append(f"measure v1 {instrument}")
            variants.append((f"{tag}-{m:g}",f"How does a controlled amount change the {instrument} reading?",lines))
        specs.append((tag,"event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":direction,"min_delta":delta},variants))
    variants=[(f"balance-water-{m}","How does water volume change the balance reading?",[f"add v1 water {m}mL","measure v1 balance"]) for m in (50,100,200,400)]
    specs.append(("balance-order","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":40},variants))
    variants=[(f"thermometer-{t}","Does the thermometer retain the ordering of prepared water temperatures?",[f"add v1 water 200mL @ {t}C","measure v1 thermometer"]) for t in (10,20,40,70)]
    specs.append(("temperature-order","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":5},variants))
    variants=[(f"pressure-amount-{m:g}","How does sealed nitrogen amount change the pressure-gauge reading?",["seal v1 650mL",f"add v1 N2 {m:g}mol","measure v1 pressure"]) for m in (.001,.002,.003,.004)]
    specs.append(("pressure-order","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":1},variants))
    return groups("analytical-observables",729,specs)


def thermochemical_paths() -> dict:
    specs=[]
    variants=[]
    for i,pulses in enumerate(((400,),(200,200),(100,300),(50,150,200))):
        variants.append((f"water-heat-path-{i+1}","Does partitioning equal heat leave water at the same temperature?",["add v1 water 260mL"]+[f"heat v1 {p}J" for p in pulses]+["measure v1 thermometer"]))
    specs.append(("heat-partition","event-scalar-equal",{"event":"measured","field":"value","occurrence":"last","atol":.03,"rtol":0},variants))
    variants=[]
    for i,pulses in enumerate(((300,),(150,150),(50,250),(25,75,200))):
        variants.append((f"water-cool-path-{i+1}","Does partitioning equal cooling leave water at the same temperature?",["add v1 water 250mL @ 50C"]+[f"cool v1 {p}J" for p in pulses]+["measure v1 thermometer"]))
    specs.append(("cool-partition","event-scalar-equal",{"event":"measured","field":"value","occurrence":"last","atol":.03,"rtol":0},variants))
    variants=[(f"water-heat-dose-{j}","How does supplied energy order final water temperature?",["add v1 water 250mL",f"heat v1 {j}J","measure v1 thermometer"]) for j in (0,100,200,400)]
    specs.append(("heat-order","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":.05},variants))
    variants=[]
    for i,(hot,cold) in enumerate(((50,50),(100,100),(150,150),(200,200))):
        variants.append((f"mix-scale-{i+1}","Does extensive scaling preserve an equal-part hot-cold mixture temperature?",[f"add v1 water {hot}mL @ 70C","new",f"add v2 water {cold}mL @ 20C","new","mix v1 1 v2 1 into v3","measure v3 thermometer"]))
    specs.append(("mix-intensive","event-scalar-equal",{"event":"measured","field":"value","occurrence":"last","atol":.03,"rtol":0},variants))
    variants=[]
    for i,(h,c) in enumerate(((100,300),(150,250),(250,150),(300,100))):
        variants.append((f"hot-share-{i+1}","How does hot-water share order the resting mixture temperature?",[f"add v1 water {h}mL @ 70C","new",f"add v2 water {c}mL @ 20C","new","mix v1 1 v2 1 into v3","measure v3 thermometer"]))
    specs.append(("hot-share","event-scalar-order",{"event":"measured","field":"value","occurrence":"last","direction":"increasing","min_delta":1},variants))
    variants=[]
    for j in (50,100,200,300):
        variants.append((f"thermal-roundtrip-{j}","Does equal heating and cooling restore a water sample's temperature?",["add v1 water 250mL",f"heat v1 {j}J",f"cool v1 {j}J","measure v1 thermometer"]))
    specs.append(("roundtrip","event-scalar-equal",{"event":"measured","field":"value","occurrence":"last","atol":.03,"rtol":0},variants))
    return groups("thermochemical-paths",753,specs)


def generated() -> dict[str, dict]:
    docs=[separation_trains(),kinetics_boundaries(),household_biochemistry(),materials_transport(),analytical_observables(),thermochemical_paths()]
    ids=[int(c["id"].split("-",1)[0]) for d in docs for c in d["cases"]]
    assert ids == list(range(633,777))
    return {d["family"]:d for d in docs}


def encoded(doc: dict) -> str:
    return json.dumps(doc,indent=2,ensure_ascii=False)+"\n"


def main() -> None:
    parser=argparse.ArgumentParser(description=__doc__)
    mode=parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write",action="store_true")
    mode.add_argument("--check",action="store_true")
    args=parser.parse_args(); OUT.mkdir(parents=True,exist_ok=True); stale=[]
    for family,doc in generated().items():
        path=OUT/f"{family}.json"; content=encoded(doc)
        if args.write: path.write_text(content)
        elif not path.exists() or path.read_text()!=content: stale.append(str(path.relative_to(HERE.parents[1])))
    if stale: raise SystemExit("stale or missing generated manifests: "+", ".join(stale))
    print(json.dumps({"families":6,"cases":144,"status":"written" if args.write else "current"}))


if __name__ == "__main__": main()

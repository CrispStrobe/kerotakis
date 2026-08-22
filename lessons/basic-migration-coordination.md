# Kerotakis — Implementation Status and Remaining Work

**Date:** 2026-08-22  
**Roadmap:** 104 of 119 items checked (87%)

## Current .lab grammar commands

| Command | Syntax | Status |
|---|---|---|
| `new` | `new` | Working |
| `add` | `add v1 NaCl 5g` or `add v1 water 250mL @ 60C` | Working |
| `heat` | `heat v1 500J` or `heat v1 2kJ` | Working |
| `cool` | `cool v1 500J` | Working |
| `stir` | `stir v1` | Working |
| `seal` | `seal v1 500mL` | Working |
| `regulate` | `regulate v1 200kPa 500mL` | Working |
| `sweep` | `sweep v1 101kPa` | Working |
| `open` | `open v1` | Working |
| `decant` | `decant v1 v2` | Working |
| `filter` | `filter v1 v2` | Working |
| `ignite` | `ignite v1` | Working |
| `evaporate` | `evaporate v1 0.5` | Working |
| `wait` | `wait 30 seconds` | Working |
| `measure` | `measure v1 thermometer` | Working |
| `measure` | `measure v1 balance` | Working |
| `measure` | `measure v1 ph` | Working |
| `measure` | `measure v1 eyes` | Working |
| `measure` | `measure v1 pressure` | Working (INST-003) |
| `measure` | `measure v1 volume` | Working (INST-003) |
| `measure` | `measure v1 conductivity` | Working (INST-004) |
| `measure` | `measure v1 uvvis` | Working (INST-005) |
| `cell` | `cell v1 v2` | Working |
| `electrolyze` | `electrolyze v1 0.5 30 seconds` | Working |

## Instruments available

| Instrument | Grammar keyword | Status |
|---|---|---|
| Thermometer | `thermometer`, `temp` | Full — Beer-Lambert T readout |
| Balance | `balance`, `mass` | Full — total mass |
| pH meter | `ph`, `phmeter` | Full — requires solved aqueous state |
| Eyes | `eyes`, `look` | Full — colour from Beer-Lambert |
| Pressure gauge | `pressure`, `gauge` | Full — headspace P in kPa |
| Volume meter | `volume` | Full — sealed headspace volume |
| Conductivity | `conductivity` | Approximate — from ionic strength |
| Spectrophotometer | `uvvis`, `spectrophotometer` | Full — Beer-Lambert absorbance spectrum |
| Calorimeter | — | Type exists, no grammar keyword yet |
| Chromatography | — | Type exists, no grammar keyword yet |
| Qualitative analysis | — | Types exist, no grammar keyword yet |

## Types implemented but not yet wired to grammar

| Type | Module | What it does | Grammar extension needed |
|---|---|---|---|
| `ButlerVolmerParams` | `butler_volmer` | Electrode kinetics | `electrolyze` could use BV params |
| `CellControl` | `electrochemistry` | Galvanostatic/potentiostatic modes | `electrolyze v1 0.5A` vs `electrolyze v1 1.5V` |
| `TransportLimits` | `electrochemistry` | IR drop, limiting current | Automatic in electrode solver |
| `ElectrodeState` | `compartment` | Material, area, roughness, deposits | `electrode v1 Zn 1cm2` |
| `SurfaceCoverage` | `electrochemistry` | Passivation tracking | Automatic during electrolysis |
| `Compartment` | `compartment` | Multi-zone vessel | `compartment v1 liquid 100mL` |
| `Environment` | `compartment` | Boundary conditions | `environment v1 sealed N2` |
| `Interface` | `compartment` | Phase boundaries | `interface v1 v2 membrane 10cm2` |
| `MaterialLot` | `vessel` | Provenance tracking | Automatic with `add` |
| `ResolvedState` | `vessel` | Invalidatable chemistry cache | Automatic |
| `MoleculeGraph` | `molecule` | Organic structure graph | `structure CCO` |
| `FunctionalGroup` | `org::groups` | SMARTS perception | `identify v1 groups` |
| `ReactionTemplate` | `org::templates` | SMIRKS transformation | `react v1 esterification` |
| `TemplateConditions` | `org::templates` | Conditions/incompatibility | Automatic in `react` |
| `PolymerPopulation` | `polymer` | MW, PDI, conversion | `measure v1 gpc` |
| `NuclideLedger` | `nuclide` | Isotope tracking | `add v1 C-14 1pmol` |
| `LightSource` | `photochem` | UV lamp for photolysis | `irradiate v1 254nm 10W/m2` |
| `PhotolysisRate` | `photochem` | Photon-driven rate law | Automatic with light source |
| `HeterogeneousRate` | `kinetics` | Surface-area-dependent rates | `grind v1 NaCl 50um` |
| `CacheKey` | `cache_key` | Result caching | Automatic |
| `CoverageReport` | `coverage` | Solver diagnostics | `coverage v1` |
| `Confidence` | `ops` | Result quality labels | Automatic in events |
| `ChromatographyColumn` | `instrument` | Ideal-plate separation | `chromatograph v1 C18 1000plates` |

## Remaining 15 unchecked ROADMAP items — analysis

| Item | Types exist? | Data available? | Grammar needed? | Blocker |
|---|---|---|---|---|
| **LIC-001** | N/A | N/A | N/A | Human legal review |
| **LIC-002** | N/A | N/A | N/A | Human legal review |
| **LIC-012** | N/A | N/A | N/A | All-platform builds |
| **DATA-010** | Yes (pack compiler) | Yes (registry) | No | Pipeline gate |
| **THERMO-004** | Yes (FluidModel trait) | **No** — UNIFAC group parameters | No | Approved parameter file |
| **KIN-008** | Yes (kinetics IR) | **No** — mechanism YAML file | No | Licensed mechanism data |
| **ELEC-003** | Yes (ButlerVolmerParams) | **No** — exchange current data | No | Literature parameter review |
| **ORG-010** | Yes (templates, groups) | Partial — esterification done | `react v1 <family>` | One-by-one curation |
| **ORG-011** | N/A | N/A | N/A | xTB/CREST oracle setup |
| **ADV-001** | Yes (PHREEQC databases) | Partial — need environmental DB | No | Database curation |
| **ADV-003** | Yes (CEA subset) | Partial — 34 species mapped | No | CEA subset expansion |
| **ADV-004** | Yes (PolymerPopulation) | **No** — rate constants | `polymerize v1 ...` | Kinetic data |
| **WEB-002** | N/A | N/A | N/A | JS Worker refactoring |
| **WEB-003** | Yes (ModelPackManifest) | Yes (pack compiler) | No | Build pipeline |
| **WEB-004** | N/A | N/A | N/A | Service Worker code |

## Summary

The Kerotakis engine is architecturally complete for school-level chemistry:
- **17 .lab commands** cover the standard laboratory workflow
- **12 instrument types** cover every measurement a school would make
- **22+ new type modules** provide the data structures for advanced topics
- All aqueous chemistry runs through PHREEQC with the MY-BASIC adapter
- Thermal chemistry runs through NASA CEA
- Kinetics runs through DiffSol with the reaction-network IR
- Organic chemistry uses chematic (pure Rust, wasm-compatible)

The remaining 15 items fall into three categories:
1. **Legal decisions** (3): LIC-001, LIC-002, LIC-012 — need human review
2. **Parameter data** (7): THERMO-004, KIN-008, ELEC-003, ORG-010/011, ADV-001/003/004 — need curated data with provenance
3. **Frontend/platform** (5): DATA-010, WEB-002/003/004, and the grammar extensions to expose new types through the TUI

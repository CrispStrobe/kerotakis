# The apparatus catalog (CAP-25 handoff to the GUI workline)

The user supplied a ~70-item palette of lab apparatus that the bench
should show and manipulate: put on the shelf, clamp, pour between,
probe with instruments, heat, distil — with the effects visualised.
This file is the engine-side mapping every item needs before a pixel
is drawn: **the effects layer renders only what the engine emits**,
so each item is classed by where its behaviour comes from. All
visuals are OUR OWN — the palette that inspired the list is another
product's; its icons are not copied, only the universal lab-equipment
vocabulary it shares with every chemistry classroom.

Classes: **SKIN** = pure visual over a landed verb/event (draw it,
wire the existing tap); **PROP** = passive visual, no behaviour
needed beyond placement; **BEHAVIOR(n)** = needs the named engine
task first.

`BREADTH.md` adds the cross-cutting physical and material prerequisites:
`BRD-002/014` define named material recipes; `BRD-070` defines the authority
boundary between scene physics and chemistry; `BRD-071/072` evaluate
Rapier/Salva; and `BRD-073` owns spill, tip, drop and breakage semantics. An
apparatus row may depend on those tasks, but this catalog remains the owner of
its ports, limits, affordances and cabinet metadata.

## From catalog to in-world equipment cabinet (2026-08-26)

This file also feeds the player's supply room. The UI may present it as an
equipment store or cabinet, but it is never a real-money storefront. Story mode
uses availability, ownership, replenishment, and explicit mission budgets to
make equipment growth meaningful; Sandbox derives **all installed items as
available** and ignores unlock/quantity rules.

The cabinet has five player-facing sections:

| Section | Contents | Story lifecycle |
|---|---|---|
| Glassware | vessels, adapters, stoppers, funnels, tubing | reusable; common pieces replaceable |
| Instruments | probes, balance, spectrometer, meters, sensors | permanent unlock; optional calibration/maintenance |
| Equipment | heating, cooling, stands, pumps, columns, electrical apparatus | permanent unlock, sometimes room-gated |
| Materials & reagents | substances, samples, filters, indicators, gases | quantities/replenishment may be mission-constrained |
| Saved kits | player-authored or mission-suggested cabinet selections | convenience only; kit never executes a procedure |

Every catalog item—not only apparatus—needs one versioned metadata record:

| Field | Requirement |
|---|---|
| `id`, `kind`, `model_version` | Stable identity and compatibility/migration anchor |
| `name_key`, `description_key`, `purpose_key` | English/German locale keys; no display string as identity |
| `visual_asset`, `silhouette`, `footprint` | Own artwork, collision/layout bounds, and recognizable cabinet card |
| `surfaces`, `ports`, `sockets` | Where it can sit and what can connect/insert/clamp |
| `affordances` | Operator-manifest entries this object can invoke in each valid state |
| `requires` | Other item/port/room requirements with a localized unmet reason |
| `capacity`, `limits`, `precision` | Physical capacity, safe operating bounds, and instrument performance |
| `behavior_status` | `modeled`, `visual_only`, `locked_pending_model`, or `unsupported` |
| `confidence`, `provenance` | Same fixed vocabulary used by the engine and inspector |
| `story_access` | Unlock/replenishment policy and preview prerequisite; ignored by Sandbox |
| `tags` | Search synonyms, curriculum/codex links, age-safe terms, and category |

Interaction rules:

1. A card is never dead. It can be placed, previewed with a reason, or clearly
   marked as not yet modeled; visual-only props cannot imply chemical behavior.
2. Selecting an object filters the cabinet to compatible parts without hiding
   the full searchable catalog. Ports and valid destinations preview on hover,
   focus, or drag.
3. Placement and assembly live in the client scene graph; every physical action
   still compiles through the affordance manifest to a real operator. Decorative
   arrangement must never mutate chemistry.
4. Story unlocks reveal useful families in comprehensible steps (for example,
   balance → volumetric glassware → burette), with future items previewable.
   Sandbox never checks this graph.
5. Instruments display readings on themselves first and mirror them into the
   notebook. Detail sheets explain range, precision, calibration, compatible
   objects, and model confidence at the current register.
6. Catalog art follows the bright scientific-workshop tokens in
   `ROADMAP-GUI.md`: neutral bodies, strong silhouette, one functional accent,
   real scale cues, and no baked-in English labels. Chemical contents retain
   engine-computed appearance.

Cabinet acceptance is structural: every item below must have a catalog record
and one explicit disposition; every modeled operator must have at least one
reachable Sandbox affordance; and locale lint must fail on missing English or
German catalog strings.

## Containers
| Item | Behaviour source | Class |
|---|---|---|
| Beaker, Conical/Boiling/Round-bottom/Pear flask, Boiling tube, Test tube | vessel + all verbs (the current vessel IS these) | SKIN — glassware *kinds* already landed GUI-side; per-kind capacity/geometry data |
| Test tube / conical flask / boiling tube **with side arm** | gas take-off: headspace + Distil/GasEvolved routing | SKIN (side-arm = where the gas hose attaches) |
| Bung / stopper | `seal` verb + **Burst** (landed, CAP-25 slice 1): a stoppered gas-maker now BANGS honestly | SKIN |
| Volumetric flask | solution prep + value-claims (EXP-17): fill-to-mark = target volume | SKIN + mark-line UI |
| Displacement beaker, Eureka can | volume-by-displacement instrument (EXP-18) | BEHAVIOR(EXP-18) |
| Gas jar, tank | headspace machinery + gas tests (EXP-31) | SKIN / BEHAVIOR(EXP-31) for the tests |
| Watch glass, weight boat | balance + evaporation surface | PROP + `measure balance` tap |
| Ice bath | ThermalMode::Thermostatted (landed) | SKIN |
| Thiele tube | melting-point instrument (EXP-33) | BEHAVIOR(EXP-33) |
| Schlenk flask, Schlenk lines | inert-atmosphere handling: `sweep` verb exists (N2 purge); full Schlenk technique | SKIN over `sweep` now; anaerobic-chemistry depth later (EXP-46 Grignard wants it) |
| Calorimetry cup | calorimeter instrument (landed) | SKIN |
| Vacuum flask | thermal isolation: ThermalMode::Adiabatic (landed) | SKIN |
| Cuvette | spectrophotometer (landed, CAP-22) | SKIN |
| Wash bottle | `add vN water` micro-dose | SKIN |
| Half-cell tube | Cell verb (landed) | SKIN |
| Stemmed glass, autosampler vial, cell-culture flask | out of bench scope (bio/instrument props) | PROP |

## Filtering
| Filter funnel + (fluted) paper | `filter` verb (landed) | SKIN |
| Buchner funnel + vacuum pump | `filter` fast-path; vacuum = speed visual | SKIN (speed is presentation) |
| Separatory funnel | `drain` + computed layers + partitioning (landed) | SKIN — the crown demo |
| Sieve | mechanical separation of solids by size — particle-size data exists for colloids (EXP-32) | BEHAVIOR(EXP-32 extension) |
| Thistle tube | pour-into-sealed apparatus | SKIN |
| Evaporating dish | `evaporate` (landed) | SKIN |
| Syringe filter, SPE cartridge | instrument-prep props; SPE = chromatography kin (EXP-8 modes) | PROP / BEHAVIOR(EXP-8) |

## Metallic & Ceramic
| Retort stand/clamps/rings, burette holder/clamp, G-clamp, tripod, test-tube rack, scissor jack, bench mat, beehive stand | assembly & placement | PROP (pure scene graph) |
| Spatula | `add` solid dosing gesture | SKIN |
| Nichrome loop | **flame test** (landed event) | SKIN |
| Combustion spoon | `ignite` in a gas jar (EXP-31/35) | SKIN |
| Steel wool | Fe surface-area species + `grind`-class kinetics + rusting (EXP-34) | BEHAVIOR(EXP-34) |
| Hoffman clamp | flow control on tubing | PROP |
| Mortar + pestle | `grind` verb (landed) | SKIN |
| Spotting tile | qualitative tests (EXP-30) | BEHAVIOR(EXP-30) |
| Crucible + cover | strong heating, mass-before/after (EXP-33 hydrates, EXP-45 conservation) | SKIN |
| Fume hood | safety-screen context: hazardous vapours vent — ties the **smell/waft** verb (landed, CAP-25) | SKIN |

## Devices
| Rotary evaporator | `evaporate`/`distil` under reduced pressure | BEHAVIOR(BRD-032 pressure-dependent phase routing) |
| Vacuum pump | partner of the above + Buchner | SKIN where partnered |
| Syringe pump | timed dosing: `titrate` machinery generalised to programmed addition | BEHAVIOR(small: dosing schedule on the titrate loop) |
| Centrifuge | sedimentation: settled-solid machinery exists (drain leaves solids); spin = accelerated settling of suspensions (EXP-32) | BEHAVIOR(EXP-32 extension) |
| Piston chamber | Headspace::PressureControlled (landed — Regulate verb) | SKIN |
| Sonicator | mixing/dissolution acceleration — kinetics surface-area kin | BEHAVIOR(small, or PROP with honest no-op note) |
| Hydrothermal autoclave | sealed + temperature + **Burst** rating (landed) — an autoclave is the vessel whose rating is HIGH | SKIN + per-vessel rating datum |
| Drop counter | titrate steps (landed — the curve's x-axis) | SKIN |
| Hoffman voltameter | `electrolyse` with gas collection in graduated arms (landed chemistry; arm-volume readout) | SKIN + volume readout tap |
| Breakable glassware / spill tray | scene proposes collision/tip; engine emits break/spill and owns transferred material | BEHAVIOR(BRD-070/071/073) |

## Chromatography
| Paper, spot, chamber | EXP-8 modes (TLC/paper: Rf) — plate model landed, paper/TLC mode is data | SKIN + BEHAVIOR(EXP-8 Rf mode) |

## Storage
| Reagent bottles (glass/plastic), gas cylinder | the shelf itself + `add`; cylinder = gas source for headspace | SKIN |
| Named household/food/material stock | versioned `MaterialRecipe` expands to conserved components while cabinet retains learner-facing name | BEHAVIOR(BRD-002/014) |
| Desiccator | drying: hydrate water removal (EXP-33) | BEHAVIOR(EXP-33) |

## The tally that matters
Roughly **45 of ~70 items are SKIN or PROP today** — drawable now
against landed verbs and events, no engine work. The rest concentrate
in EXP tasks already on the registry (18, 30, 31, 32, 33, 34, 8) plus
three new small behaviours surfaced by this catalog: reduced-pressure
boiling (rotovap), programmed dosing (syringe pump), accelerated
settling (centrifuge). Slice 1 of CAP-25 landed with this file: the
**smell/waft verb** (curated odours, hazard-aware, "odourless is
data") and **Burst** (sealed glass has a limit; the explosion the GUI
draws is an engine event with the ledger exact through the bang).

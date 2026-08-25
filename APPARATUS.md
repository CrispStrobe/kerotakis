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
| Rotary evaporator | `evaporate`/`distil` under reduced pressure — pressure-dependent boiling NOT modelled | BEHAVIOR(new: reduced-pressure boiling, scope with CAP-1 thermo) |
| Vacuum pump | partner of the above + Buchner | SKIN where partnered |
| Syringe pump | timed dosing: `titrate` machinery generalised to programmed addition | BEHAVIOR(small: dosing schedule on the titrate loop) |
| Centrifuge | sedimentation: settled-solid machinery exists (drain leaves solids); spin = accelerated settling of suspensions (EXP-32) | BEHAVIOR(EXP-32 extension) |
| Piston chamber | Headspace::PressureControlled (landed — Regulate verb) | SKIN |
| Sonicator | mixing/dissolution acceleration — kinetics surface-area kin | BEHAVIOR(small, or PROP with honest no-op note) |
| Hydrothermal autoclave | sealed + temperature + **Burst** rating (landed) — an autoclave is the vessel whose rating is HIGH | SKIN + per-vessel rating datum |
| Drop counter | titrate steps (landed — the curve's x-axis) | SKIN |
| Hoffman voltameter | `electrolyse` with gas collection in graduated arms (landed chemistry; arm-volume readout) | SKIN + volume readout tap |

## Chromatography
| Paper, spot, chamber | EXP-8 modes (TLC/paper: Rf) — plate model landed, paper/TLC mode is data | SKIN + BEHAVIOR(EXP-8 Rf mode) |

## Storage
| Reagent bottles (glass/plastic), gas cylinder | the shelf itself + `add`; cylinder = gas source for headspace | SKIN |
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

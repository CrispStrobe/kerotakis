# Kerotakis — Breadth programme

> Finished work is not listed here. What landed, and what it taught us, is in
> [HISTORY.md](HISTORY.md). Task numbers are never renumbered and never reused.

Dependency-ordered work for making the bench answer the ordinary questions a
curious child or teenager asks: what happens if I mix, heat, cool, burn, crush,
drop, dissolve, separate, or inspect familiar matter?

This document is the source of truth for `BRD-*` tasks. `CAPABILITIES.md` owns
solver parity, `EXPERIMENTS.md` owns experiment and quest content,
`APPARATUS.md` owns equipment affordances, and the GUI roadmaps own rendering.
Those documents link here rather than restating task scope.

The programme does **not** promise arbitrary chemistry. No available engine can
reliably decide what every arbitrary set of reagents will do, and the project
deliberately does not ship an unrestricted synthesis oracle. Breadth comes from
three honest layers:

1. familiar substances and named materials represented as reviewed data;
2. general solvers used where their model domain applies;
3. curated reaction families with explicit substrate and condition gates.

Every uncovered branch remains a typed, visible `NotYetModelled` result. A
larger catalog must never turn absence of a model into “nothing happened.”

## Rules every BRD task inherits

- Shipped code must satisfy the permissive-only bar in `PLAN.md` and
  `deny.toml`. Re-verify the exact release and transitive dependency graph at
  adoption time; a licence named here is not a substitute for that review.
- Shipped data is CC0 or CC BY 4.0 only. CC BY-SA data may be an external or
  build-time oracle but does not enter official app-store packs. Source,
  licence, retrieval date, checksum, original citation, and per-field
  provenance are mandatory under `CONTRIBUTING.md`.
- A code licence does not license bundled parameters, force fields, reaction
  mechanisms, structures, or database records. Audit these separately.
- Network access is build-time only. Every released pack and every runtime
  engine works offline.
- A new visible number carries `Provenance`; a new parser gets a fuzz target;
  a new solver route gets conservation and relevant metamorphic invariants plus
  an independent golden or differential oracle.
- Do not bulk-import “everything.” Importers first produce a quarantine report;
  only allowlisted fields and reviewed records graduate into runtime packs.
- Each task is one branch/PR unless its scope explicitly says otherwise. An
  agent claiming a task records its ID in the PR and does not silently absorb a
  dependent task.

Large data/content parents are intentionally sliceable. Claim them as
`BRD-012.S01`, `BRD-014.S01`, `BRD-023.S01`, `BRD-031.S01`, `BRD-041.S01`,
`BRD-052.S01`, or `BRD-060.S01`; add the next zero-padded slice to the parent
status before work starts. Every slice must be independently useful, pass all
parent invariants, name its exact records/families, and stay small enough for
one review. The parent closes only when its numeric/content acceptance floor is
met. Agents may work on independent slices concurrently.

A decision gate closes with either `go` or `no-go`. On `go`, its implementation
children become available. On `no-go`, those children are marked
`not-applicable` with the decision record and downstream milestones treat that
track as honestly closed, not missing. A no-go may not remove the user outcome:
the record must name the existing/fallback route that owns it.

## Target and measurement

“Everything a young person can think of” is not a finite acceptance criterion.
The programme therefore measures a versioned **curiosity corpus** of concrete
prompts. The initial target is 500 prompts spanning at least these action
families:

| Family | Examples |
|---|---|
| Mix and dissolve | salt + water, oil + water, cola + milk, soap + grease |
| Heat and cool | boil alcohol, freeze seawater, melt wax, bake soda |
| Burn and oxidise | candle, sugar, steel wool, alcohol, fuel with too little air |
| Acids, bases and gases | vinegar + chalk, fizzy drinks, bleach, ammonia cleaner |
| Separate | filter soil, distil colored water, chromatography, crystallise salt |
| Materials | paper, wood, plastic, glass, concrete, rust, batteries, alloys |
| Food and life | bread rising, digestion, enzymes, fermentation, respiration |
| Handle and inspect | pour, spill, drop, crush, smell, view particles/molecules/crystals |

The corpus is a regression suite, not a claim that every prompt must compute.
Every prompt must end in exactly one auditable disposition: computed, curated,
qualitative model, explicit boundary, or missing task. Silent fall-through is a
failure.

## Dependency graph and delivery stages

```text
Stage B0 — measure and define matter
  BRD-000 curiosity corpus ──→ BRD-001 coverage report
  BRD-002 MaterialRecipe IR ─→ BRD-003 import/quarantine framework

Stage B1 — stock the everyday shelf
  BRD-003 ─┬→ BRD-010 PubChem importer ─┐
           ├→ BRD-011 ChEBI importer ───┴→ BRD-012 familiar pure-substance pack
           └→ BRD-013 USDA importer
  BRD-002 + BRD-012 + BRD-013 ───────────→ BRD-014 household-material packs

Stage B2 — make structure and reaction families load-bearing
  BRD-012 ─→ BRD-020 reaction-family IR ─→ BRD-021 Indigo/RDKit spike
                                            └→ BRD-022 runtime executor
  BRD-014 + BRD-022 ───────────────────────→ BRD-023 first family pack

Stage B3 — broaden general solver domains (parallel after B1)
  BRD-012 ─→ BRD-030 feos spike ─→ BRD-031 fluid parameter pack ─→ BRD-032 routing
  BRD-012 ─→ BRD-040 Cantera audit ─→ BRD-041 mechanism packs ─→ BRD-042 FFI gate

Stage B4 — life and solids
  BRD-011 + BRD-020 ─→ BRD-050 bio IR ─→ BRD-051 Rhea importer ─→ BRD-052 bio pack
  BRD-003 ─→ BRD-060 COD importer ─→ BRD-061 spglib adapter
                 BRD-060 + BRD-061 + BRD-081 ─→ BRD-062 crystal experience

Stage B5 — tactile and visual reach
  BRD-070 authority contract ─→ BRD-071 Rapier ─→ BRD-073 spill/breakage
                           └──→ BRD-072 Salva ───┘
  BRD-014 + BRD-070 ──────────→ BRD-074 gas/foam observables
  BRD-014 + spectral optics ──→ BRD-075 dye/pigment mixing
  BRD-041 + BRD-070 + BRD-071 → BRD-076 movable heat/flame tools
  BRD-000 + BRD-012 + BRD-023 → BRD-077 element coverage/table modes
  BRD-012 + BRD-080 viewer spike ─→ BRD-081 molecular/crystal viewer
  BRD-022 ─────────────────────────→ BRD-082 Ketcher authoring surface

Stage B6 — validate and graduate
  BRD-032 ─→ BRD-090 pycalphad oracle where a cleared TDB exists
  all shipped tracks ─→ BRD-100 breadth release gate
```

Stages express release order, not a ban on parallel work. Tasks with all listed
dependencies complete may proceed concurrently. `BRD-042`, `BRD-082`, and
`BRD-090` are optional gates; they cannot hold the first household pack hostage.

## Stage B0 — measurement and shared data contracts

### BRD-002 — `MaterialRecipe`: named mixtures and objects

- [ ] **Status:** in progress. **Size:** large.
  **Depends on:** current pack loader (`CAP-21`/`DATA-010`).
- **Outcome:** the data schema represents vinegar, bleach, air, milk, paper,
  steel, soil, and batteries without pretending each is a pure species.
- **Scope:** add a versioned `MaterialRecipe` containing identity/aliases,
  component amount ranges and basis, optional unresolved fractions, physical
  form, preparation/lot assumptions, allowed substitutions, model confidence,
  provenance per component, and an expansion policy. `add v1 vinegar 10mL`
  expands once into conserved vessel contents and records the recipe version
  in the event log. Solid objects may carry geometry/surface-area metadata but
  still expand into ledger-owned components.
- **Integration:** registry packs, story stock, cabinet search, safety screening,
  replay/cache keys, and `explain`. Built-in species always remain canonical
  identities; recipes cannot override them.
- **Acceptance:** exact component and mass ledgers across add/undo/replay;
  deterministic selection within declared ranges (fixed recipe or seeded
  sample, never ambient randomness); an unresolved fraction is displayed and
  conserved rather than discarded; English/German aliases resolve.
- **Out of scope:** reverse-engineering branded proprietary formulations or
  treating a nutrient panel as complete molecular composition.
- **Browser explanation slice (PR #433):** the existing shelf `(i)` receives
  engine-owned recipe confidence, basis, component ranges, unresolved matter,
  preparation, lot assumptions, and source id. Component names also join shelf
  search in both the canonical and displayed locale; every wire field is
  additive so older hosts remain usable.
- **Remaining work:** close the CLI/browser `explain` parity audit and record
  the landed slice in `HISTORY.md` after the PR is green and merged. The
  schema, runtime `add`, and finite stockroom ledger are already recorded
  there.

## Stage B1 — the everyday shelf

### BRD-012 — Familiar pure-substance pack v1

- [ ] **Status:** open; slices in flight (shipped slices S02–S05 are recorded in
  `HISTORY.md`). **Size:** large/data-heavy. **Depends on:** BRD-010 and
  BRD-011.
- **Outcome:** at least 300 reviewed identities that a school-age user is likely
  to name, including common gases, acids/bases, salts, metals, minerals, fuels,
  solvents, sugars, fats, monomers, polymers-as-populations, pigments, and
  biological small molecules.
- **Scope:** select from curiosity-corpus demand, not database popularity. Every
  record needs the minimum viable behavior matrix: identity/composition,
  available phases, add-by-mass conversion, visible appearance or explicit
  unknown, safety coverage, supported solver routes, and honest refusals for
  unsupported routes. Add aliases in English and German.
- **Integration:** PHREEQC names and CEA/thermo/mechanism availability are
  resolved at build time. A registry identity does not imply that every engine
  supports it; coverage metadata makes that distinction queryable.
- **Acceptance:** `kero species` and the GUI catalog load records without code
  edits; every new record passes identity, molar-mass, safety-totality,
  provenance, locale, and route-coverage lints; curiosity report shows the
  identity-unknown bucket materially reduced with no false “inert” answers.
- **Out of scope:** importing hundreds of thousands of database entries or
  fabricating missing thermodynamic parameters.
- **Shelf screenshot triage (2026-08-27; formula is authoritative when labels
  are truncated) — still open:**
  1. **Litmus/indicator material** with acidic/basic colour-state data — NOT
     shipped with the P0 salts (needs colour-state machinery).
  2. **P1 constrained:** nitric acid (`HNO3`) as a clearly restricted lab stock;
     “carbonic acid” as carbonated water / dissolved `CO2(aq)`, not a stable
     neat-acid bottle. Land identity/safety first, then aqueous routing.
  3. **P2 virtual-only metals:** sodium (`Na`) and potassium (`K`), gated behind
     complete water/fire safety and qualitative reaction-family coverage before
     shelf exposure.
  UI acceptance for this batch includes full-name tooltips, formula-first search,
  and English/German aliases rather than duplicate identities.

### BRD-013 — USDA FoodData Central adapter

- [ ] **Status:** open (the 15-record adapter checkpoint shipped 2026-08-30 is
  recorded in `HISTORY.md`). **Size:** medium. **Depends on:** BRD-003.
- **Source/licence:** FoodData Central CC0/public domain. Primary docs:
  <https://fdc.nal.usda.gov/api-guide/>. Branded formulations are volatile and
  incomplete; prefer Foundation Foods and stable generic records.
- **Scope:** import only components that can map honestly onto Kerotakis
  species or declared unresolved fractions: water, sugars, organic acids,
  salts/minerals, fat/protein/carbohydrate aggregate populations. Preserve the
  food description, basis, sample/release and analytical uncertainty.
- **Integration:** output `MaterialRecipe` candidates. Nutrients are not silently
  converted to unique molecules: “protein,” “fat,” “fiber,” and “ash” remain
  named aggregate components until a model explicitly handles them.
- **Acceptance:** fixtures for milk, egg, flour, juice, oil and sugar; component
  masses plus unresolved remainder reconcile to the declared serving/sample
  mass; API keys and network access never enter builds or runtime.
- **Out of scope:** nutrition advice, branded-product fidelity, flavor chemistry,
  or inferring pH/reactions from nutrient labels.
- **Remaining work:** promotion review of the quarantined snapshot (nothing is
  promoted yet); soybean oil, unsalted butter and table salt still report
  proximate conflicts rather than clean candidates.

### BRD-014 — Household and school material packs

- [ ] **Status:** open; slices in flight (shipped slices S02–S07 and related
  checkpoints are recorded in `HISTORY.md`). **Size:** large/data-heavy.
  **Depends on:** BRD-002, BRD-012, and BRD-013.
- **Outcome:** versioned packs for at least 75 familiar named materials, selected
  by BRD-000 demand.
- **Scope:** begin with air, tap/seawater, vinegar, baking powder/soda, bleach,
  ammonia cleaner, hydrogen peroxide, soap/detergent surrogate, cola/fizzy
  drink, dry/wet yeast, dish soap, hand soap, pepper, isopropanol, food dyes,
  watercolor and acrylic-paint surrogates, milk, juice, flour/dough, vegetable
  oil, candle wax, paper, wood, common plastics, glass, soil/sand/clay,
  chalk/limestone, concrete surrogate, rusted/clean iron, steel/brass/bronze,
  and common battery chemistries. Each recipe states grade/concentration
  assumptions and unresolved material. Locale-sensitive ambiguous names such
  as English “soda” and German “Soda” must ask which material was meant rather
  than silently choosing baking soda, washing soda, or a fizzy drink.
- **Integration:** safety evaluates expanded components before chemistry;
  solver routing operates on those components; the UI and narration retain the
  material name; depletion and replay use the recipe version.
- **Acceptance:** at least 150 curiosity prompts become runnable or receive a
  more specific model boundary; every recipe has a conservation test and one
  characteristic behavior test; changing recipe version invalidates cache keys.
- **Out of scope:** product endorsements, clandestine composition guessing, or
  detailed toxicology from generic recipes.
- **Remaining work:** photosynthesis, respiration-as-a-process, transpiration,
  turgor and plasmolysis mechanisms are still absent for the plant/food rows
  BRD-014.S03 gave words to (owned jointly with BRD-052).

### BRD-020 — Reaction-family intermediate representation

- [ ] **Status:** open; phase 1 (IR + chematic oracle), phase 2 (conservation
  ledger + order independence) and phase 3 (router wired into the standard
  stack) landed — recorded in `HISTORY.md`. **Size:** large. **Depends on:**
  BRD-012 and the landed kinetics/curated-reaction infrastructure.
- **Outcome:** one audited rule can apply a known transformation to structurally
  matching substrates without becoming an arbitrary predictor.
- **Scope:** versioned family records contain mapped reactant/product query,
  stoichiometry, required/forbidden functional groups, solvent/phase,
  temperature/pH/catalyst/light gates, competing-family priority, equilibrium
  or kinetic model reference, products/by-products, atom mapping, provenance,
  confidence and explicit refusal domain. Define deterministic conflict
  resolution and require the rule to name why it fired or declined.
- **Integration:** family matching occurs after safety and identity resolution,
  before generic honesty fallback. Products enter the normal vessel ledger and
  downstream PHREEQC/thermo/kinetics routes. Reuse exact stoichiometry and
  molecule conservation lint.
- **Acceptance:** serializer/schema tests, parser fuzzing, atom/charge/mass
  conservation, order independence, conflict fixtures, and a deliberately
  out-of-domain substrate that refuses rather than overgeneralizes.
- **Out of scope:** reaction planning, retrosynthesis, learned outcome
  prediction, or automatic extraction of rules from patents.
- **Remaining work:** spoken decline events (gate refusals currently reach only
  the capability report, not the event stream), gated on regenerating lesson
  goldens.

### BRD-021 — Indigo versus RDKit shipping spike

- [ ] **Status:** open/decision gate. **Size:** medium. **Depends on:** BRD-020.
- **Candidates:** Indigo (Apache-2.0) and RDKit (BSD-3-Clause). Both have
  browser-capable builds and reaction-template machinery; verify the chosen
  release, notices, wasm size, mobile builds and transitive assets. Primary
  docs: <https://lifescience.opensource.epam.com/indigo/> and
  <https://github.com/rdkit/rdkit/tree/master/Code/MinimalLib>.
- **Scope:** implement the same narrow C/wasm-facing spike for both: parse and
  canonicalize 100 structures, match 30 SMARTS queries, execute 20 mapped
  reactions, retain stereochemistry/isotopes/charges, serialize products, and
  survive malformed inputs/resource caps. Compare against current `chematic`.
- **Decision rule:** prefer the smallest engine that passes the chemistry
  corpus identically on native, browser, macOS and iOS. Keep the loser as a
  build-time differential oracle where useful. If neither passes, harden
  `chematic`; do not force an FFI adoption.
- **Acceptance:** checked-in benchmark/report with exact versions and licence
  inventory; no production dependency in this PR.
- **Out of scope:** GUI molecule drawing or general reaction prediction.

### BRD-022 — Runtime structure/reaction executor

- [ ] **Status:** blocked on decision. **Size:** large. **Depends on:** BRD-021.
- **Scope:** integrate the selected engine behind a Kerotakis-owned trait for
  canonicalization, substructure match, mapped transformation, depiction data,
  and stable error/resource-limit reporting. The trait prevents pack schemas
  from depending on toolkit-specific object formats. Cross-check official InChI
  where supported.
- **Integration:** native and wasm hosts expose identical result JSON; mobile
  builds link the same curated API surface; `kerotakis-org` retains ownership.
  No engine call may bypass reaction-family conditions or the safety pass.
- **Acceptance:** BRD-021 corpus plus differential tests against the non-selected
  toolkit; byte-stable canonical output where the format promises it; wasm and
  native limits reject adversarial structures deterministically; dependency and
  NOTICE lints pass.

### BRD-023 — Familiar organic reaction-family pack v1

- [ ] **Status:** open (BRD-023.S01 and the galvanic-corrosion, peroxide-bleach
  and alcohol-oxidation checkpoints shipped 2026-09-05; the bounded
  thermoplastic/thermoset comparison became directly runnable in the unified
  catalogue 2026-09-06 — recorded in
  `HISTORY.md`). **Size:** large/data-heavy. **Depends on:** BRD-014 and
  BRD-022.
- **Scope:** curate a first useful set driven by `EXP-36/41/42/46/50` and the
  curiosity corpus: acid/base behavior of functional groups, combustion,
  esterification/hydrolysis, alcohol oxidation, carbonyl tests/additions,
  carboxylate formation, amide hydrolysis at an honest level, addition and
  condensation polymerization exemplars, substitution/elimination only inside
  the documented selectivity matrix, and moisture-sensitive Grignard formation
  as a tightly bounded teaching case.
- **Integration:** thermodynamic/kinetic numbers use existing provenance and
  solver routes; template products are ordinary registered species or
  generated structures with an explicit property-coverage ceiling.
- **Acceptance:** at least 50 family/substrate cases and 25 negative/out-of-scope
  cases; every product is atom mapped and conserved; condition perturbation
  tests switch or suppress outcomes as documented; no unrestricted free-form
  synthesis endpoint appears in CLI, wasm, MCP or GUI.
- **Out of scope:** pharmaceuticals as a catalog objective, reaction-condition
  recommendation, yield optimization, and routes outside curriculum/household
  demand.
- **Remaining work:** the whole organic family pack beyond the shipped
  checkpoints (esterification/hydrolysis conditions on BRD-022's executor,
  carbonyl/carboxylate/amide families, polymerization exemplars, Grignard).

## Stage B3 — general thermodynamics and gas kinetics

### BRD-031 — Cleared fluid parameter pack

- [ ] **Status:** in progress; blocked on a cleared residual-fluid parameter
  pack (checkpoints through 2026-09-05 are recorded in `HISTORY.md`).
  **Size:** large/data-heavy. **Depends on:** BRD-030.
- **Scope:** curate parameters for the fluids and mixtures actually demanded by
  BRD-000/014: water, common alcohols/ketones/esters/hydrocarbons, CO2, air
  gases, ammonia, light fuels and selected refrigerants. Every parameter set
  records its original publication/data licence and model validity range.
- **Integration:** join by canonical species identity; model selection is
  explicit and inspectable. Missing binary parameters produce a named refusal
  or a labelled lower-fidelity route, never silent ideality.
- **Acceptance:** coverage matrix and one literature/oracle fixture per model
  family; no proprietary DIPPR, NIST SRD/WebBook, UNIFAC Consortium, or
  otherwise encumbered parameter enters the pack.
- **Remaining work (checkpoint plan, 2026-08-31):**
  1. BRD-031a fail-closed fluid contract — shipped.
  2. BRD-031b current-solver domain safety — shipped.
  3. BRD-031.S01 six-fluid pilot pack — the rights audit itself closed a
     runtime-promotion **no-go**; see
     `provenance/brd-031-pilot-source-audit.md`. Saturated liquid density and
     residual-EOS (PC-SAFT) parameters remain unsourceable under any accepted
     licence for every one of the nine fluids the pack now identifies.
  4. BRD-031d disposable feos adapter evidence — shipped.
  5. BRD-031e integration audit — still open; only this step unblocks BRD-032's
     residual-EOS half.

### BRD-032 — feos-backed bench routing

- [ ] **Status:** first slice shipped 2026-09-05 (adsorption, pressure-dependent
  boiling, dry-ice and liquid-nitrogen phase routes; the methyl-orange on
  activated-charcoal case became directly runnable in the unified catalogue
  2026-09-06 — recorded in
  `HISTORY.md`); the residual-EOS half remains blocked on BRD-031's uncleared
  parameter pack. **Size:** large. **Depends on:** BRD-031.
- **Scope:** route pressure-dependent boiling/condensation, flash, phase split,
  density and transport-property requests through the adapter when the exact
  parameter/model domain is present. Preserve existing UNIFAC/cubic routes as
  named alternatives and expose model disagreement through `explain`.
- **Integration:** `heat`, `cool`, `distil`, `evaporate`, `drain`, sealed
  headspace, rotovap/reduced-pressure behavior, charts, CLI and wasm.
- **Acceptance:** conservation and scale invariance; pressure monotonicity for
  boiling where valid; azeotrope/phase-split goldens; identical host results;
  BRD-000 phase-change coverage increases without weakening honest refusals.
- **Liquid-nitrogen investigation checkpoint (2026-09-06):** the coupled
  ethanol-freezing/nitrogen-boiling route is reachable through an Energy Yard
  lesson and mission. Handling hazards remain separate from reactive groups;
  Story loans liquid nitrogen only inside the mission and never awards it.
- **Remaining work:** the residual-EOS route (density, saturation pressure for
  CO2/N2/O2/hexane/ethyl acetate) stays refused until BRD-031e clears parameters.

### BRD-041 — Familiar gas/combustion mechanism packs

- [ ] **Status:** packs shipped 2026-09-05 (PRs #393, #399), routed into the
  engine the same week (PR #404) — recorded in `HISTORY.md`. Three acceptance
  items remain open, listed below. **Size:** large/data-heavy. **Depends on:**
  BRD-040 (complete).
- **Scope:** add reviewed reduced mechanisms for hydrogen/oxygen, methane,
  carbon monoxide, selected light hydrocarbon/alcohol fuel exemplars, and
  nitrogen chemistry only where the mechanism licence and educational need are
  clear. Add soot/yellow-flame narration only when backed by an explicit model;
  otherwise state that luminosity/particles are outside the gas mechanism.
- **Integration:** `ignite`, sealed/open headspace, CEA equilibrium, diffsol
  kinetics, spectrophotometer/flame appearance where computed, heat ledger,
  emissions and safety events.
- **Acceptance:** Cantera differential oracle for ignition delay/species traces
  and equilibrium endpoints; element/energy conservation; rich/lean and
  temperature/pressure metamorphic cases; bounded runtime in wasm; at least 25
  curiosity prompts graduate from missing to computed.
- **Remaining acceptance items (not met as counted):**
  1. **No Cantera differential oracle** for ignition delay or species traces —
     blocked on an extent-integrator finding (a Jacobian probe sized by total
     extent rather than by species cannot carry a radical chain through
     ignition without a component-scaled fix or the existing CVODE path).
  2. **wasm runtime is unbounded/unmeasured.**
  3. **The "≥25 prompts graduate through a reviewed reduced mechanism" floor is
     not met as counted** — only 3 BRD-041 rows were ever `missing`, and those
     closed through BRD-012.S04's CEA-equilibrium registry fix, not a
     mechanism.
- **Deliberately absent, and said so in each file:** alcohol fuel exemplars;
  nitrogen chemistry (N₂ is a diluent/collider only); soot, luminosity and
  flame colour; falloff; transport, flame structure and flame speed;
  `OH + HO₂ → H₂O + O₂` between 400–1300 K; any hydrogen-free CO→CO₂ route.

### BRD-042 — Full Cantera C-API shipping gate

- [ ] **Status:** parked — BRD-040 recorded a **no-go** on 2026-08-29. **Size:**
  extra large. **Depends on:** BRD-040 (complete) and a stable upstream C API on
  all targets.
- **BRD-040 finding:** no BRD-041 need requires the C API. The portable parser,
  diffsol, the CEA equilibrium path and the existing apparatus models cover
  every item in BRD-041's acceptance criteria; the only capability the portable
  path lacks is mixture transport, which BRD-041 does not ask for and which is a
  self-contained kinetic-theory calculation rather than a reason to link a C++
  engine. Linking Cantera would also not touch what actually blocks BRD-041,
  which is that no candidate mechanism carries a redistribution grant. Re-open
  this task only on a required capability that is genuinely infeasible in Rust.
  Reasoning in `provenance/brd-040-cantera-audit.md` § 6.
- **Scope:** compile a minimal handle-based API for desktop, wasm and mobile;
  compare binary size, startup, determinism, resource limits and answers with
  the portable Kerotakis mechanism path. Keep one engine instance per worker.
- **Acceptance:** all release targets and offline packaging pass; no C++ types
  cross the boundary; exact licence/NOTICE/SBOM; measurable capability gain
  that BRD-041 cannot reasonably supply. Otherwise record a no-go and close.
- **Out of scope:** replacing the portable path solely for solver prestige.

## Stage B4 — biochemistry and crystalline matter

### BRD-050 — Bounded biochemical reaction IR and router

- [ ] **Status:** open, with a bounded first route shipped 2026-09-05 (pH
  window, irreversible denaturation, food-carried `EnzymeSource`, and three
  fermentation metabolisms — recorded in `HISTORY.md`). **Size:** medium.
  **Depends on:** BRD-011, BRD-020 and existing reaction-network kinetics.
- **Outcome:** familiar biochemical reactions can be represented without
  pretending PHREEQC or the organic family router models a living cell.
- **Scope:** extend/compose the reaction-family IR with enzyme identity,
  compartment/environment, pH/T window, cofactors, directionality, kinetic law
  (`mass_action`, Michaelis–Menten and inhibition forms), aggregate
  macromolecule bookkeeping, and a declared abstraction level. Define a
  `Biochemical` solver route and explicit boundary against medical claims.
- **Acceptance:** catalase, amylase and fermentation exemplar networks conserve
  declared moieties/elements; enzyme is not consumed unless the model says so;
  outside pH/T/compartment refuses or applies a documented denaturation model;
  `explain` labels the curated biochemical abstraction.
- **Out of scope:** cell biology simulation, diagnosis, pharmacology, complete
  metabolism, or sequence-to-function prediction.
- **Remaining work:** there is still no reaction IR and no composable
  `Biochemical` solver route — the shipped route is two hand-written models
  with typed parameters. No Michaelis–Menten, cofactors, inhibition,
  compartments or directionality yet. Lactate has no species in any loaded
  database, so lactic/acetic fermentation still cannot close a computed pH,
  and casein remains unresolved in the milk buffer.

### BRD-051 — Rhea reaction adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-050 and BRD-003.
- **Source/licence:** Rhea CC BY 4.0; ChEBI supplies participant identities.
  Primary licence: <https://www.rhea-db.org/help/license-disclaimer>.
- **Scope:** ingest selected balanced reaction equations, directionality,
  participant ChEBI IDs and enzyme cross-references into quarantine. Map
  protonation/compartment assumptions explicitly. Do not infer rate constants,
  physiological occurrence, safety or lesson suitability from Rhea membership.
- **Acceptance:** pinned-release fixture, equation/charge conservation, stable
  ChEBI identity joins, attribution propagation, and rejection reports for
  polymers/generic participants or unsupported compartments.

### BRD-052 — Familiar biochemistry pack v1

- [ ] **Status:** open (vocabulary-only tranche via BRD-014.S03, the four rows
  closed by BRD-050's bounded route, and the unapplied respiration equation
  all shipped 2026-09-05 — recorded in `HISTORY.md`). **Size:**
  large/data/content. **Depends on:** BRD-051, BRD-012 and EXP-51's kinetic
  family.
- **Scope:** curate roughly 100 reactions/networks around starch/sugar
  digestion, catalase, lactase/protease exemplars, yeast fermentation, bread
  rising, respiration, photosynthesis as a bounded net model, acidification,
  food browning only where a defensible simplified family exists, and enzyme
  inhibition experiments. Supply parameters from primary/open sources or leave
  the reaction qualitative/equilibrium-only.
- **Integration:** material recipes from BRD-013/014, `wait`, `kero study`,
  calorimeter, gas/headspace, pH and spectrophotometer; EXP-9/14/40/47/51 quests.
- **Acceptance:** at least 30 replayable experiments with positive and negative
  controls; conservation and temperature/pH response tests; no medical or
  nutritional advice; every rate parameter is independently oracle-checked.
- **Remaining work:** photosynthesis, respiration-as-a-process, transpiration,
  turgor and plasmolysis are still absent (nine BRD-014.S03 rows run and
  answer nothing); each recipe's own `lot_assumptions` names the missing
  model.

### BRD-060 — Crystallography Open Database adapter

- [ ] **Status:** open (BRD-060.S01, the silicon/doped-silicon resistivity
  objects, shipped 2026-09-05 — recorded in `HISTORY.md`). **Size:** medium.
  **Depends on:** BRD-003.
- **Source/licence:** COD structures/data CC0. Primary source:
  <https://www.crystallography.net/cod/new.html>.
- **Scope:** ingest a small reviewed educational subset of CIF structures for
  registered substances: salts, sugar, ice polymorphs, graphite/diamond,
  metals/alloys, minerals, hydrates and selected molecular crystals. Preserve
  COD ID, original authors/citation, cell, coordinates, occupancy, temperature
  and disorder flags. Validate CIF and identity/composition before promotion.
- **Integration:** registry `structure` facet, precipitate/crystal inspection,
  codex and viewer payload. A crystal record does not supply thermodynamic
  stability or a reaction rule.
- **Acceptance:** at least 50 representative structures, deterministic
  normalized payloads, malformed/disordered fixtures, exact formula/charge
  checks where meaningful, CC0 plus scholarly attribution in provenance.
- **Remaining work:** no elemental silicon species is installed yet, and no
  carrier-density/doping model exists — n-type vs p-type and semiconductor
  junctions remain out of reach.

### BRD-061 — spglib symmetry adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-060.
- **Candidate/licence:** spglib BSD-3-Clause. Primary project:
  <https://github.com/spglib/spglib>.
- **Scope:** compile the narrow C API needed to standardize cells and report
  space group, equivalent atoms and symmetry operations. First target native;
  ship to wasm/mobile only after a compile/size spike. Define stable Rust-owned
  input/output types and resource limits.
- **Acceptance:** known NaCl, diamond, graphite, ice and calcite fixtures;
  tolerance sensitivity is surfaced; native results reproduce the upstream
  oracle; licence and target gates pass.
- **Out of scope:** predicting a crystal structure from formula.

### BRD-062 — Crystal inspection and growth experience

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-060,
  BRD-061 and BRD-081's selected viewer.
- **Scope:** add `inspect crystal`/GUI affordance showing unit cell, repeated
  lattice, coordination and symmetry at register-appropriate detail. Connect
  precipitation/recrystallisation events to the correct known structure where
  the solved phase has an exact mapping; otherwise show particles or an honest
  “structure not installed.” Crystal growth animation follows solved deposited
  amount, not a nucleation claim.
- **Acceptance:** native/web visual contract snapshots, exact phase-to-structure
  mapping, accessibility descriptions, and no structure shown for ambiguous
  polymorphs without the ambiguity being stated.

## Stage B5 — tactile physics and scientific views

### BRD-074 — Gas-to-foam observable and elephant-toothpaste slice

- [ ] **Status:** open (the first implementation slice, colour checkpoint and
  quantitative-catalysis checkpoint shipped 2026-08-27 — recorded in
  `HISTORY.md`). **Size:** medium-large. **Depends on:** BRD-014 and
  BRD-070; reuse the existing peroxide kinetics until BRD-050/052 supplies the
  richer enzyme model.
- **Outcome:** gas-forming reactions can drive bubbles or persistent foam when
  a recipe contains a declared surfactant, without treating foam as new matter.
- **Scope:** add typed gas-production-rate, trapped-gas, foam-volume/height,
  overflow, lifetime and warmth observables. Chemistry owns oxygen yield, rate
  and heat; a bounded drainage/coalescence model maps those values plus
  surfactant concentration, viscosity, vessel geometry and temperature onto a
  visual target. Reduced motion shows the same peak and final state. Food color
  is an optical passenger and may form user-chosen stripes without changing the
  rate. Ship an elephant-toothpaste experiment comparing no catalyst, hydrated
  yeast/catalase surrogate, manganese dioxide and potassium iodide where its
  distinct reaction path is installed.
- **Safety contract:** the child-facing real-world activity is 3% peroxide,
  adult supervision, fitted goggles and gloves. Concentrations above 3% remain
  explorable in simulation but are labelled restricted; 10% and above are
  never described as safe home practice. Closed-vessel and combustible-contact
  variants are vetoed or presented only as safety boundaries.
- **Acceptance:** conservation from 2 H2O2 to 2 H2O + O2; catalyst survives;
  oxygen/foam monotonicity across controlled concentration and catalyst tests;
  exothermic temperature response with an explicit enthalpy source; native/web
  event parity; visual snapshots for foam rise, overflow, color stripes and
  reduced motion; a no-soap control bubbles but does not build persistent foam.
- **Out of scope:** CFD-derived bubble-size distributions, ingestible advice,
  or claiming one yeast brand has a universal enzyme activity.
- **Remaining work:** spatially preserved, user-placed colour stripes still
  need a typed placement operator; remaining boundaries are yeast-brand/age
  calibration, irreversible denaturation history and inhibition, catalyst
  pore/BET area, adsorption and pore-scale diffusion.

### BRD-075 — Transparent dye and opaque-pigment mixing

- [ ] **Status:** open (transparent-dye, opaque-pigment, watercolor and
  acrylic-material checkpoints shipped — recorded in `HISTORY.md`). **Size:**
  medium-large. **Depends on:** BRD-014 and the existing spectral/Beer–Lambert
  path; BRD-070 for the renderer contract.
- **Candidate/licence:** `palette` (MIT OR Apache-2.0) for audited color-space
  conversion/interpolation. It does not supply chemistry or pigment constants;
  keep the dependency only if it reduces tested conversion code and passes the
  wasm/mobile size gate.
- **Scope:** distinguish transparent food color/watercolor absorption from
  opaque acrylic pigment scattering. Dyes mix by concentration, path length
  and spectra through Beer–Lambert. Paint uses a bounded Kubelka–Munk K/S model
  with curated pigment coefficients, binder/white-substrate assumptions and an
  explicit “pigment data missing” result. Never average display RGB as though it
  were a physical mixture. Track dilution, opacity, staining and unresolved
  proprietary pigment/binder fractions through `MaterialRecipe` versions.
- **Interaction:** offer side-by-side swatches, droppers/brush amounts, undo,
  arbitrary user ratios and “what should I add to move toward this color?” only
  as bounded interpolation among installed materials—not general inverse
  formulation.
- **Acceptance:** primary/secondary transparent-dye fixtures, subtractive paint
  fixtures including white/black, concentration/intensity monotonicity, order
  independence, spectral-to-sRGB oracle tests, color-vision-safe descriptions,
  and identical numeric outcomes headless/native/web.
- **Out of scope:** branded paint matching, fluorescence, drying/polymerization
  in v1, or learned image-based color prediction.
- **Remaining work:** installed pigment coefficient records beyond the four
  named surrogates, thin watercolor washes beyond the three shipped, and UI
  droppers/brushes/substrate/coverage controls.

### BRD-076 — Movable Bunsen burner and guided heat interactions

- [ ] **Status:** open (guided-control, air-collar, liquid-fuel and isopropanol
  checkpoints shipped — recorded in `HISTORY.md`). **Size:** large. **Depends
  on:** BRD-070, BRD-071 and BRD-041's combustion/oxidation mechanisms.
- **Outcome:** learners can place a burner, adjust gas and air, light/extinguish
  it, and heat or ignite nearby matter through the same authoritative engine
  used by scripts.
- **Scope:** typed place/move/valve/air-collar/ignite/extinguish operations;
  flame geometry and heat-flux field derived from fuel flow and entrained air;
  vessel/material exposure integrates energy over time. Installed combustion
  models decide ignition, sustained burning, oxygen-starved yellow flames,
  soot/CO boundaries and fuel depletion. Distance, shielding, vessel material,
  heat capacity and breakage thresholds matter. Scene motion proposes poses;
  chemistry accepts exposure and owns temperature/reaction events.
- **Guidance:** sandbox permits free placement with persistent hazard cues;
  lessons may highlight safe zones and controls but do not teleport tools or
  fake outcomes. Keyboard/touch controls and reduced-motion equivalents expose
  every operation.
- **Acceptance:** deterministic replay of pose and valve history; validated
  heat-flux tests within the declared near-field model; water heats without
  burning, ethanol/candle/paper ignite only after their installed gates,
  nonflammable controls refuse, fuel/oxygen/energy ledgers close, and moving the
  flame away stops heat transfer. Native/web parity and safety veto tests.
- **Out of scope:** using renderer pixels as collision/temperature truth, full
  turbulent flame CFD, or implying that unmodelled materials are nonflammable.
- **Remaining work:** continuous free-space pose, fuel/air collar chemistry and
  distance-dependent heat flux for the typed apparatus-state tranche; sustained
  pool-fire geometry, sealed/oxygen-starved combustion, and soot/CO remain
  unmodelled; a flame held to 70% isopropanol has no combustion thermochemistry.

### BRD-080 — Molecular viewer selection spike

- [ ] **Status:** in progress; 3Dmol.js 2.5.5 is the provisional smaller-
  adequate selection (checkpoint evidence through 2026-08-31 recorded in
  `HISTORY.md`), pending disposable Svelte and physical constrained-mobile
  acceptance. Do not ship both candidates. **Size:** small-medium. **Depends
  on:** BRD-012.
- **Candidates/licences:** 3Dmol.js (BSD) and Mol* (MIT). Primary projects:
  <https://github.com/3dmol/3Dmol.js> and
  <https://github.com/molstar/molstar>.
- **Scope:** render the same molecule, crystal/CIF, protein exemplar, cube
  orbital and short trajectory; test selection, labels, accessibility hooks,
  offline bundling, Svelte integration, mobile memory and bundle size.
- **Decision rule:** choose the smaller adequate viewer; Mol* wins only if
  macromolecular/volume capability justifies its complexity. Do not ship both.
- **Acceptance:** report and prototype behind a disposable route; exact licence
  inventory; no production dependency in the decision PR.
- **Remaining checkpoints:**
  1. BRD-080a reproducible candidate evidence — shipped.
  2. BRD-080b disposable comparison route — shipped in code/browser and
     Svelte/deployment form; **pending acceptance:** physical
     constrained-mobile RAM/GPU evidence (Playwright emulation/SwiftShader are
     explicitly not substitutes).
  3. BRD-080c audited go/no-go record — provisional selection recorded;
     remains provisional until item 2's physical-mobile check passes.
  4. BRD-081a renderer-neutral accessible core — not started, conditional on
     the go/no-go closing.

### BRD-081 — Molecular/crystal viewer integration

- [ ] **Status:** blocked on the remaining BRD-080 acceptance checks; BRD-060
  also blocks the crystal slice. **Size:** medium-large. **Depends on:**
  BRD-080 and BRD-060 for the crystal slice.
- **Scope:** create a renderer-neutral `ScientificView` contract for atoms,
  bonds, unit cells, surfaces/volumes, annotations and provenance, then adapt
  the selected viewer. It consumes registry structures and future QM assets;
  it never invents a conformer or crystal. Add plain-language and tabular
  alternatives.
- **Acceptance:** offline PWA/native packaging, molecule/crystal/orbital
  fixtures, accessible fallback, theme independence from computed chemical
  appearance, and deterministic view-state serialization.

### BRD-082 — Ketcher structure/reaction authoring surface

- [ ] **Status:** optional after the organic executor. **Size:** medium-large.
  **Depends on:** BRD-022 and BRD-081.
- **Candidate/licence:** Ketcher Apache-2.0; its standalone mode uses Indigo
  wasm. Audit all npm packages/assets/notices and avoid bundling a second
  chemistry engine if BRD-022 selected RDKit without a justified boundary.
  Primary project: <https://github.com/epam/ketcher>.
- **Scope:** Sandbox/codex-author mode for drawing or importing a molecule or
  mapped reaction. Submission goes through Kerotakis identity, safety and
  reaction-family routing. The editor may validate/draw; it cannot authorize a
  transformation or populate missing thermodynamic properties.
- **Acceptance:** offline, keyboard/touch accessible, Svelte integration,
  round-trip MOL/SDF/SMILES/RXN fixtures, identity conflict handling, and an
  unmistakable response when a valid drawing has no installed behavior model.
- **Out of scope:** exposing unrestricted reaction enumeration to learners or
  treating a drawn arrow as evidence that chemistry occurs.

## Stage B6 — build-time validation and release gate

### BRD-090 — pycalphad solid/alloy oracle

- [ ] **Status:** optional build-time oracle. **Size:** medium. **Depends on:**
  a concrete EXP/materials task and BRD-032 if coupled fluid/solid behavior is
  being checked.
- **Candidate/licence:** pycalphad code MIT. Thermodynamic database (`.tdb`)
  licences are independent and often restrictive; no calculation begins until
  a specific cleared database is recorded. Primary project:
  <https://github.com/pycalphad/pycalphad>.
- **Scope:** validate a bounded school-relevant system such as a cleared binary
  alloy phase diagram or heat-treatment transition. Persist only reviewed
  aggregate/golden values with provenance; Python remains build-time.
- **Acceptance:** independent hand/literature check, pinned environment and TDB
  checksum/licence, deterministic fixture generator, written validity range.
- **Out of scope:** scraping proprietary CALPHAD databases, a general metallurgy
  engine, or shipping Python.

### BRD-091 — OpenMM decision record (parked)

- [ ] **Status:** parked; do not implement without a new concrete requirement.
  **Size:** small decision record. **Depends on:** BRD-052 or a future molecular
  motion experiment that cannot use the current particle/kinetics models.
- **Reason:** OpenMM simulates molecular dynamics, not bond-making/breaking in
  ordinary chemistry. Its Reference/CPU pieces are MIT, while GPU platform
  pieces have different licensing; it is heavy and lacks a natural offline
  browser/mobile fit for this product. Primary licence record:
  <https://github.com/openmm/openmm/blob/master/docs-source/licenses/Licenses.txt>.
- **Gate:** an agent may reopen this only with a named experiment, force field
  and data licence, target matrix, educational observable, and proof that a
  trajectory materially teaches something the lighter particle view cannot.
- **Acceptance:** either a justified scoped successor task or a dated no-go.
- **Out of scope:** adopting molecular dynamics as a proxy for chemical
  reactivity.

### BRD-092 — CoolProp decision record (oracle/extra only)

- [ ] **Status:** parked. **Size:** small decision record. **Depends on:** a
  BRD-030 discrepancy or fluid absent from a cleared feos parameter pack.
- **Candidate/licence:** CoolProp MIT, but wrapper/platform support and fluid
  data provenance must be checked separately. Primary project:
  <https://github.com/CoolProp/CoolProp>.
- **Reason:** CoolProp offers a broad reference-quality fluid-property surface,
  but duplicates much of the intended feos role and currently has a less clean
  cross-target integration path. It is valuable as a second opinion before it
  is valuable as shipped runtime weight.
- **Gate:** prefer it as a desktop/build-time differential oracle. Shipping is
  reconsidered only if it closes a named high-demand fluid gap on every target
  at acceptable size and no feos route exists.
- **Note from BRD-030 (2026-08-30):** do **not** reach CoolProp data by way of
  feos. `feos:parameters/multiparameter/coolprop.json` carries CoolProp's
  reference-EOS coefficients with the MIT notice and the per-fluid citations
  removed; if CoolProp data is ever wanted it must come from CoolProp itself,
  with its notice, or from the primary publications. See
  `provenance/brd-030-feos-spike.md` § 4.2.
- **Acceptance:** dated comparison and explicit oracle/runtime decision.

## Completed BRD tasks

- **BRD-000** — curiosity corpus v1. Complete. See `HISTORY.md`.
- **BRD-001** — coverage classifier and report. Complete. See `HISTORY.md`.
- **BRD-003** — source adapter, quarantine, and promotion framework. Complete. See `HISTORY.md`.
- **BRD-010** — PubChem identity and approved-property adapter. Complete. See `HISTORY.md`.
- **BRD-011** — ChEBI identity and ontology adapter. Complete. See `HISTORY.md`.
- **BRD-030** — direct feos integration spike. Closed `go` (scoped). See `HISTORY.md`.
- **BRD-040** — Cantera mechanism and API audit. Complete. See `HISTORY.md`.
- **BRD-070** — scene/chemistry authority contract. Complete. See `HISTORY.md`.
- **BRD-071** — Rapier rigid-body integration. Complete; go with optional Rapier 2-D. See `HISTORY.md`.
- **BRD-072** — Salva fluid-visual integration. Complete/no-go; kept `fluidScene`. See `HISTORY.md`.
- **BRD-073** — spills, tipping, drops and breakage. Complete. See `HISTORY.md`.
- **BRD-077** — element coverage score and progressive periodic table. Complete. See `HISTORY.md`.
- **BRD-093** — permissive thermochemical-engine target gate. Closed no-go for universal runtime. See `HISTORY.md`.
- **BRD-094** — GPU fluid and volumetric-rendering decision record. Frontend spike only; backend closed no-go. See `HISTORY.md`.

### BRD-100 — Breadth release gate v1

- [ ] **Status:** final integration task. **Size:** large. **Depends on:**
  BRD-001, BRD-014, BRD-023, BRD-032, BRD-041, BRD-052, BRD-062, BRD-073 and
  BRD-081. A track whose decision gate closed `no-go` satisfies this dependency
  only when its implementation children are marked `not-applicable` and the
  documented fallback is covered by the curiosity corpus. Optional decision
  tasks need only be closed with a go/no-go record.
- **Outcome:** the curiosity corpus becomes a release-quality capability
  contract rather than a one-time audit.
- **Scope:** run all prompts and publish the disposition matrix; require zero
  silent outcomes, zero unowned gaps, zero provenance-free visible numbers,
  zero unknown safety classifications for reachable stock, and host parity for
  the supported route subset. Set per-family coverage floors from the BRD-000
  baseline only after the classifier exists; do not invent percentages now.
- **Acceptance:** full preflight, licence/SBOM/provenance/locale lints, native
  and wasm smoke corpora, accessibility checks for new views, deterministic
  reports, and a human review of every disposition that changed since baseline.
- **Out of scope:** declaring Kerotakis universal. The release report names
  boundaries and the next highest-demand missing tasks.

## Agent pickup checklist

Before starting a `BRD-*` task:

1. Verify every listed dependency is merged, not merely in progress.
2. Read the owning integration document (`CAPABILITIES.md`, `EXPERIMENTS.md`,
   `APPARATUS.md`, `ROADMAP-GUI.md`) and name any required companion task.
3. Re-verify upstream version, source licence, bundled data/parameter licence,
   target support and maintenance status; update `provenance/sources.toml`.
4. Keep quarantined inputs and generated oracle outputs out of runtime data
   paths until promotion review.
5. Add the task's acceptance tests before marking it complete, run
   `tools/preflight.sh`, and update this status plus the BRD-001 baseline.

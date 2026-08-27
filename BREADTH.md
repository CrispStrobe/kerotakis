# Kerotakis — Breadth programme

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

### BRD-000 — Curiosity corpus v1

- [x] **Status:** complete on `codex/brd-000-curiosity`. **Size:** medium.
  **Depends on:** nothing.
- **Outcome:** `tests/coverage/curiosity-v1/manifest.toml` indexes ordered TOML
  shards containing exactly 500 stable prompts,
  expected dispositions, age band, action family, and capability tags. The
  corpus contains harmless, hazardous, nonsensical, and intentionally
  unsupported questions; it is not a list of only happy paths.
- **Scope:** derive prompts from the already-audited experiment corpora, shipped
  lessons, common household/school materials, and editorially authored edge
  cases. Define stable normalization for aliases and quantities. Add a runner
  that executes runnable prompts and records typed routing outcomes without
  snapshotting prose.
- **Integration:** reuse `.lab`/replay and JSON event contracts. Map each prompt
  to `EXP-*`, `CAP-*`, and `BRD-*` tags where applicable.
- **Acceptance:** every prompt parses or has an explicit parse-boundary
  expectation; corpus lint catches duplicate normalized prompts, missing tags,
  and unowned gaps; CI can run a small smoke subset while the full report is a
  scheduled/native tier.
- **Out of scope:** claiming scientific support for all prompts or changing a
  solver to make the initial percentage look better.

### BRD-001 — Coverage classifier and report

- [x] **Status:** complete on `codex/brd-001-baseline`. **Size:** medium.
  **Depends on:** BRD-000 (complete).
- **Outcome:** `kero coverage curiosity` emits deterministic JSON and a compact
  human report with counts for `computed`, `curated`, `qualitative`,
  `boundary`, and `missing`, grouped by action, material class, age band, and
  owning task.
- **Scope:** classify from typed engine/router events, never output-string
  matching. Preserve the engine/model/dataset provenance for successful paths
  and the precise refusal reason for gaps. Add a checked-in baseline so a PR
  cannot silently turn a computation into a refusal or vice versa.
- **Acceptance:** byte-deterministic reports; a synthetic routing regression
  fails CI; the report distinguishes “substance unknown” from “substance known,
  reaction unknown.”
- **Out of scope:** a vanity percentage with unlike dispositions collapsed.
- **Implementation note (2026-08-27):** typed parser errors, solver-route
  evidence, the five-way runner, deterministic grouped reports, the
  cross-family smoke gate, and a 500-entry native baseline are shipped. The
  baseline pins owner/outcome/reason per prompt and treats seven initial solver
  failures as their own regression state; ownership and graduation rules live
  beside it in `tests/coverage/curiosity-v1/README.md`.

### BRD-002 — `MaterialRecipe`: named mixtures and objects

- [ ] **Status:** in progress. **Size:** large.
  **Depends on:** current pack loader
  (`CAP-21`/`DATA-010`).
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
- **Foundation checkpoint (2026-08-27):** the source/pack schema now carries
  versioned recipes, localized aliases, component ranges, explicit unresolved
  fractions, form/geometry, substitutions, confidence and fixed/seeded
  expansion. Validation prevents species shadowing and ambiguous material
  names; deterministic expansion conserves the requested basis amount. The
  checked-in pack contains initial 3% peroxide and 5% vinegar recipes. Runtime
  `add`, stock/safety/replay integration remains the next BRD-002 checkpoint.
- **Runtime checkpoint (2026-08-27):** `add` now resolves a material key/alias,
  converts a volume only through reviewed bulk density, pins recipe ID/version,
  basis and sample seed in the serialized operator, expands once, screens the
  complete prospective mixture, deposits canonical species, and retains an
  explicit unresolved-material ledger. Events keep both the familiar material
  identity and component amounts. Built-in and optional-pack recipes share the
  runtime registry without allowing shadowing. Remaining BRD-002 work is stock
  depletion, proportional transfer of unresolved portions, cabinet cards and
  full undo/UI coverage.

### BRD-003 — Source adapter, quarantine, and promotion framework

- [ ] **Status:** open. **Size:** large. **Depends on:** BRD-002 and the existing
  `kerotakis-data` pack compiler.
- **Outcome:** all external breadth sources use one auditable path:
  fetch/build-time snapshot → raw quarantine → field allowlist → normalized
  candidate → human-reviewed runtime pack.
- **Scope:** define adapter output, source/record/field provenance, licence lane,
  checksums, rejection reasons, identity conflict reports, units normalization,
  and reproducible snapshot manifests. No runtime HTTP. Add commands to diff an
  upstream refresh without automatically promoting changed records.
- **Acceptance:** synthetic tainted fields and incompatible licences cannot
  enter a runtime pack; two source records for one InChIKey produce a reviewable
  merge/conflict report; rebuilding from a pinned snapshot is byte-identical;
  parser fuzz target and provenance lint pass.
- **Out of scope:** a generic data lake or unattended periodic imports.

## Stage B1 — the everyday shelf

### BRD-010 — PubChem identity and approved-property adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-003.
- **Source/licence:** PubChem PUG REST and bulk records; US-government core is
  public-domain-like, but depositor annotations retain source-specific terms.
  Continue the existing per-field source allowlist; do not interpret “found in
  PubChem” as a licence. Primary docs: <https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest>
  and <https://pubchem.ncbi.nlm.nih.gov/docs/data-sources>.
- **Scope:** generalize the existing approved import into an adapter for CID,
  canonical/isomeric SMILES, Standard InChI/InChIKey, formula, charge, mass,
  depositor-neutral synonyms, and only explicitly cleared physical-property
  fields. Pin raw responses/bulk release identifiers and obey service limits.
- **Integration:** official InChI recomputes identity; `chematic` plus the
  selected BRD-022 engine cross-check structure; candidates enter quarantine,
  not `registry-source-v1.json` directly.
- **Acceptance:** a 100-record fixture covers salts, hydrates, isotopes,
  stereochemistry, mixtures incorrectly returned for names, and conflicting
  synonyms; no CAS-only or non-allowlisted annotation crosses the promotion
  boundary.

### BRD-011 — ChEBI identity and ontology adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-003.
- **Source/licence:** ChEBI CC BY 4.0, monthly/nightly versioned dumps. Primary
  docs: <https://www.ebi.ac.uk/chebi/about> and
  <https://www.ebi.ac.uk/chebi/downloads>.
- **Scope:** ingest reviewed 3-star structures and the minimum useful ontology
  slice: identity, formula, charge, mass, names/synonyms, roles, and parent
  classes needed for biochemical/material search. Keep ChEBI IDs as external
  identifiers; Standard InChIKey remains the cross-source join.
- **Integration:** source attribution flows into NOTICE/data attribution and
  `explain`; ontology roles may seed search tags but never safety or reactivity
  without a separate reviewed rule.
- **Acceptance:** pinned-release reproducibility; tautomer/protonation conflicts
  are reported rather than merged; attribution survives pack compilation; no
  biological role is converted into a reaction rule.

### BRD-012 — Familiar pure-substance pack v1

- [ ] **Status:** open. **Size:** large/data-heavy. **Depends on:** BRD-010 and
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
  are truncated):** 29 of the 39 visible entries already resolve. Add the ten
  real gaps progressively:
  1. **P0 school essentials:** ammonium chloride (`NH4Cl`), iron(III) chloride
     (`FeCl3`), sodium sulfate (`Na2SO4`), and a litmus/indicator material with
     acidic/basic colour-state data. These unlock common solubility, hydrolysis,
     crystallisation and indicator interactions using existing solver routes.
  2. **P1 constrained:** nitric acid (`HNO3`) as a clearly restricted lab stock;
     “carbonic acid” as carbonated water / dissolved `CO2(aq)`, not a stable
     neat-acid bottle. Land identity/safety first, then aqueous routing.
  3. **P2 virtual-only metals:** sodium (`Na`) and potassium (`K`), gated behind
     complete water/fire safety and qualitative reaction-family coverage before
     shelf exposure.
  4. **P2 toxic virtual-only barium salts:** barium chloride (`BaCl2`) and barium
     hydroxide (`Ba(OH)2`), gated behind soluble-barium safety and precipitation
     coverage. Never present these as household experiment supplies.
  UI acceptance for this batch includes full-name tooltips, formula-first search,
  and English/German aliases rather than duplicate identities.

### BRD-013 — USDA FoodData Central adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-003.
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

### BRD-014 — Household and school material packs

- [ ] **Status:** open. **Size:** large/data-heavy. **Depends on:** BRD-002,
  BRD-012, and BRD-013.
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
- **Checkpoint 1 implemented:** baking soda/Natron, anhydrous washing
  soda/Waschsoda, and cornstarch/Speisestärke are versioned localized material
  recipes that expand exactly to the existing `NaHCO3`, `Na2CO3`, and `starch`
  solver species. The bare word “Soda” remains intentionally unclaimed because
  its meaning changes by language and context. Baking powder waits for an
  audited acid-salt component; pepper waits for aggregate-solid appearance;
  isopropanol waits for its pure-species, volatility, safety and combustion
  routes; dyes and paints wait for BRD-075 optical material roles. These are
  prerequisite boundaries, not generic inert substitutes.
- **Checkpoint 2 implemented:** the familiar vinegar + baking-soda reaction is
  a balanced curated route with limiting-reagent stoichiometry, dissolved
  sodium acetate, and CO2 that visibly evolves from an open vessel or remains
  in a sealed headspace. Reaction heat remains explicitly unclaimed pending an
  audited enthalpy record.
- **Checkpoint 3 implemented:** liquid hand soap/Flüssigseife is distinct from
  dish detergent/Spülmittel, with its own versioned unresolved formulation and
  bounded foam-stabilisation parameters. Generic “Seife/soap” remains
  unclaimed because a solid soap bar and a liquid hand wash are not the same
  material.
- **Checkpoint 4 implemented:** native and WebAssembly shelf catalogues now
  append every built-in `MaterialRecipe` beside pure species, using its
  canonical key so taps and drags compile to the same replayable `add`
  operator as typed commands. Physical form selects household-friendly volume
  or mass amounts; component formulas feed search/periodic-table coverage;
  component hazards are combined and any unresolved fraction remains visibly
  unassessed. Optical materials reuse their computed solution swatches.

## Stage B2 — organic structure and curated reaction families

### BRD-020 — Reaction-family intermediate representation

- [ ] **Status:** open. **Size:** large. **Depends on:** BRD-012 and the landed
  kinetics/curated-reaction infrastructure.
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

- [ ] **Status:** open. **Size:** large/data-heavy. **Depends on:** BRD-014 and
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

## Stage B3 — general thermodynamics and gas kinetics

### BRD-030 — Direct feos integration spike

- [ ] **Status:** open/decision gate. **Size:** medium. **Depends on:** BRD-012
  and completed CAP-1 routing.
- **Candidate/licence:** `feos`, MIT OR Apache-2.0. It supplies PC-SAFT,
  ePC-SAFT, group-contribution/multiparameter models, phase equilibrium and
  transport calculations. Audit parameter-file provenance independently.
  Primary project: <https://github.com/feos-org/feos>.
- **Scope:** compare feos with `kerotakis-thermo` on 20 pure fluids and 20
  mixtures: density, vapor pressure, bubble/dew point, flash, enthalpy and
  critical point where applicable. Measure wasm size/time/memory and compile
  every release target. Prototype a Kerotakis adapter without changing routing.
- **Acceptance:** independent fixtures and discrepancy report; exact model and
  parameter source attached to every result; go/no-go decision names which
  existing models feos would replace, backstop, or leave alone.
- **Out of scope:** adopting feos merely because it has more models.

### BRD-031 — Cleared fluid parameter pack

- [ ] **Status:** blocked on BRD-030 go. **Size:** large/data-heavy.
  **Depends on:** BRD-030.
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

### BRD-032 — feos-backed bench routing

- [ ] **Status:** blocked. **Size:** large. **Depends on:** BRD-031.
- **Scope:** route pressure-dependent boiling/condensation, flash, phase split,
  density and transport-property requests through the adapter when the exact
  parameter/model domain is present. Preserve existing UNIFAC/cubic routes as
  named alternatives and expose model disagreement through `explain`.
- **Integration:** `heat`, `cool`, `distil`, `evaporate`, `drain`, sealed
  headspace, rotovap/reduced-pressure behavior, charts, CLI and wasm.
- **Acceptance:** conservation and scale invariance; pressure monotonicity for
  boiling where valid; azeotrope/phase-split goldens; identical host results;
  BRD-000 phase-change coverage increases without weakening honest refusals.

### BRD-040 — Cantera mechanism and API audit

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-012 and current
  Cantera-YAML/kinetics support.
- **Candidate/licence:** Cantera BSD-3-Clause. Mechanism files and their original
  provenance/licences require separate review. Primary project and licence:
  <https://github.com/Cantera/cantera> and
  <https://github.com/Cantera/cantera/blob/main/License.txt>.
- **Scope:** inventory the current parser against Cantera YAML rate-law,
  thermo, transport and reactor features; identify the smallest additional
  subset needed for familiar fuels and atmospheric examples. Audit candidate
  mechanisms before any data import. Record gaps that truly require Cantera's
  C API rather than extending the portable slice.
- **Acceptance:** feature/fixture/licence matrix, parser rejection tests for
  unsupported YAML, and an ordered list of reduced mechanisms. No runtime FFI
  or new mechanism ships in this task.

### BRD-041 — Familiar gas/combustion mechanism packs

- [ ] **Status:** open. **Size:** large/data-heavy. **Depends on:** BRD-040.
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

### BRD-042 — Full Cantera C-API shipping gate

- [ ] **Status:** optional/parked until BRD-040 proves need. **Size:** extra
  large. **Depends on:** BRD-040 and a stable upstream C API on all targets.
- **Scope:** compile a minimal handle-based API for desktop, wasm and mobile;
  compare binary size, startup, determinism, resource limits and answers with
  the portable Kerotakis mechanism path. Keep one engine instance per worker.
- **Acceptance:** all release targets and offline packaging pass; no C++ types
  cross the boundary; exact licence/NOTICE/SBOM; measurable capability gain
  that BRD-041 cannot reasonably supply. Otherwise record a no-go and close.
- **Out of scope:** replacing the portable path solely for solver prestige.

## Stage B4 — biochemistry and crystalline matter

### BRD-050 — Bounded biochemical reaction IR and router

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-011, BRD-020 and
  existing reaction-network kinetics.
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

- [ ] **Status:** open. **Size:** large/data/content. **Depends on:** BRD-051,
  BRD-012 and EXP-51's kinetic family.
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

### BRD-060 — Crystallography Open Database adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-003.
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

### BRD-070 — Scene/chemistry authority contract

- [ ] **Status:** open. **Size:** medium. **Depends on:** GUI scene graph and
  current operator/event contract.
- **Outcome:** physics can make the bench tactile without becoming a second,
  divergent chemistry simulation.
- **Scope:** document and type the one-way authority boundary. Chemistry owns
  amounts, phases, temperature, pressure and events. Scene physics proposes
  gestures/collisions/transfers; an accepted operator returns the authoritative
  state and visual target. Define spill destinations, broken-container events,
  transfer reconciliation, replay seeds, reduced-motion endpoints and
  background throttling.
- **Acceptance:** replay and host parity; visual frame rate cannot change
  transferred moles; interrupted pours reconcile exactly; reduced-motion and
  headless execution reach the same state; contract referenced from
  `ROADMAP-GUI.md` and `APPARATUS.md`.

### BRD-071 — Rapier rigid-body integration

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-070.
- **Candidate/licence:** Rapier Apache-2.0 with deterministic wasm builds.
  Primary project: <https://github.com/dimforge/rapier>.
- **Scope:** prototype glassware/apparatus collision, stacking, tipping and
  dropping against current 2-D bench needs before choosing 2-D or 3-D. Use
  catalog footprints/ports as collider sources; chemistry-breaking thresholds
  remain explicit apparatus data and engine events.
- **Acceptance:** deterministic replay on supported hosts, keyboard/touch
  equivalents, no tunnelling in the drop corpus, measured bundle/performance
  budget, and a go/no-go versus simpler local collision handling.

### BRD-072 — Salva fluid-visual integration

- [ ] **Status:** open/decision gate. **Size:** medium-large. **Depends on:**
  BRD-070; may run in parallel with BRD-071.
- **Candidate/licence:** Salva Apache-2.0, Rust SPH with viscosity, surface
  tension, multiphase fluids, wasm and optional Rapier coupling. Primary
  project: <https://github.com/dimforge/salva>.
- **Scope:** prototype three visuals: water pour, oil/water layers and viscous
  syrup. Feed density/viscosity/surface tension only from solved/provenanced
  state; map SPH particles to an already accepted transfer fraction. Compare
  with the existing lightweight `fluidScene` path on size, stability, mobile
  performance and reduced motion.
- **Acceptance:** no particle loss affects chemistry; visual phase ordering
  matches authoritative layers; 60/30 fps budgets are explicit; go/no-go
  report. If no-go, retain Salva as a reference and improve `fluidScene`.

### BRD-073 — Spills, tipping, drops and breakage

- [ ] **Status:** open. **Size:** large. **Depends on:** BRD-071 and the chosen
  outcome of BRD-072.
- **Scope:** add operator/event semantics for controlled partial pours, bench
  spills, vessel tipping, collision damage and recovery/cleanup. A broken
  vessel creates recoverable consequences and transfers its contents to a
  typed spill compartment; safety reruns against exposed/combined material.
- **Integration:** undo/replay, story inventory, disposal quests, cabinet
  replacement, Burst, accessibility and notebook evidence.
- **Acceptance:** mass/element/energy ledgers close across every failure path;
  identical chemistry with and without animations; hazardous spills emit
  precise safety events; save/load migration and undo cannot duplicate stock.

### BRD-074 — Gas-to-foam observable and elephant-toothpaste slice

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-014 and
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
- **First implementation slice (2026-08-27):** dish soap and dry yeast are
  versioned material recipes with explicit unresolved blends/biomass. The
  existing peroxide rate law owns O2 yield and catalyst choice; its interval
  now emits gas rate and an explicit 98.2 kJ-per-stoichiometric-extent heat
  source. A recipe-declared, bounded drainage/coalescence model maps O2 to
  trapped gas, foam volume/height, half-life and overflow using deterministic
  vessel geometry. With no declared surfactant, the same chemistry bubbles but
  produces no persistent foam. Remaining work: hydration/activity dependence,
  KI's distinct path, color stripes and renderer snapshots. The first guided
  `elephant-toothpaste.lab` lesson compares equal 3% peroxide/yeast charges with
  and without dish soap on one shared clock, then waits one foam half-life.
- **Colored-foam checkpoint 1 implemented:** persistent foam now carries the
  liquid mixture's engine-computed spectral sRGB and plain-language colour into
  the additive scene contract. The web vessel lightens that physical tint into
  bubble-film and overflow visuals, while older hosts still fall back to white;
  arbitrary food-colour mixtures therefore change the foam without changing
  oxygen yield or rate. Spatially preserved, user-placed stripes still require
  a typed placement operator and are not inferred from a well-mixed vessel.

### BRD-075 — Transparent dye and opaque-pigment mixing

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-014 and the
  existing spectral/Beer–Lambert path; BRD-070 for the renderer contract.
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
- **Transparent-dye checkpoint 1 implemented:** explicit 0.1% w/w dropper
  surrogates use the already curated 16-band spectra for betanin red, curcumin
  yellow and indigo-carmine blue. Their absorbances add in the existing
  Beer–Lambert/CIE pipeline, so arbitrary ratios, dilution, vessel path length,
  intensity and order-independent subtractive mixing are computed rather than
  RGB-averaged. Generic “Lebensmittelfarbe/food colouring” stays ambiguous;
  watercolor and acrylic remain blocked on the distinct scattering/pigment
  model.
- **Opaque-pigment checkpoint 1 implemented:** the shared native/wasm core now
  has a deterministic, order-independent Kubelka–Munk `K/S` mixer for an
  optically thick, diffusely lit acrylic-paint surrogate. Curated absorption
  and scattering spectra mix by amount before conversion through the same CIE
  observer as solutions; white/black bounds, subtractive blue+yellow mixing,
  order independence and explicit missing-pigment-data refusal are tested.
  Installed pigment coefficient records, thin watercolor washes, UI
  droppers/brushes and substrate/coverage controls remain separate checkpoints.
- **Transparent-watercolor checkpoint 1 implemented:** red/betanin,
  yellow/curcumin and blue/indigo-carmine watercolor washes are versioned
  school-material surrogates at 0.02% w/w. They expand to water plus the same
  reviewed chromophores as the food-color droppers, so concentration, dilution,
  path length and arbitrary-ratio mixing remain Beer–Lambert/CIE calculations.
  Generic “Wasserfarbe/watercolor” stays unclaimed, and these transparent
  washes are not presented as opaque commercial pigment pans.
- **Acrylic-material checkpoint 1 implemented:** named red, yellow, blue,
  white and black waterborne acrylic teaching surrogates carry effective
  16-band absorption/scattering roles. Shelf swatches and arbitrary-ratio
  vessel mixtures run through the shared Kubelka–Munk/CIE core; white lightens,
  blue+yellow mixes subtractively, order does not matter and acrylic is visibly
  opaque. Water, pigment and binder fractions remain explicitly surrogate or
  unresolved, generic “Acrylfarbe/acrylic paint” stays ambiguous, and no result
  claims a brand, artist pigment, wet-film gloss or dried-film match.

### BRD-076 — Movable Bunsen burner and guided heat interactions

- [ ] **Status:** open. **Size:** large. **Depends on:** BRD-070, BRD-071 and
  BRD-041's combustion/oxidation mechanisms.
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
- **Guided-control checkpoint 1 implemented:** a Bunsen burner can be deployed
  at the selected vessel/work zone, moved by selecting another zone, and given
  a 5–100% flame setting plus bounded exposure time. The first reviewed
  near-field bridge delivers at most 500 W to the vessel and compiles the
  exposure to the authoritative replayable `heat` operator; a separate “touch
  flame to contents” control compiles to `ignite`, so the engine's installed
  combustion gates—not the animation—decide whether anything burns. Continuous
  free-space pose, fuel/air collar chemistry and distance-dependent heat flux
  remain for the typed apparatus-state tranche.

### BRD-077 — Element coverage score and progressive periodic table

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-000, BRD-012 and
  BRD-023; reaction links deepen progressively as later family packs land.
- **Outcome:** selecting an element answers “what can I actually try with this?”
  while the default table stays inviting rather than presenting 118 equally
  actionable boxes.
- **Default view:** a curated **lab/curiosity table**, not the exact reduced
  main-group diagram used as inspiration. Keep familiar main-group elements and
  high-value transition metals such as Mn, Fe, Cu and Zn; omit obscure,
  synthetic and highly hazardous elements from the default even when they fit
  a neat block pattern. Preserve real group/period positions and visible gaps.
  Offer an explicit “full table” toggle with all 118 structural identities,
  remembered per user and keyboard/screen-reader accessible.
- **Coverage criterion:** compute an element-to-content index from parsed
  formulas across pure species and expanded material recipes. For each default
  element, aim for two meaningfully different familiar examples and at least
  one runnable educational interaction where chemistry supports it. One example
  may be elemental/simple and one a common compound/material; repeated salts or
  ubiquitous water do not satisfy diversity by themselves. Counts must expose
  capability level: identity-only, add/observe, property-backed, reacting, and
  lesson-backed.
- **Prioritization:** score gaps by child/teen curiosity demand, familiarity,
  solver readiness, visual/educational payoff, and safety burden. A high score
  advances a substance/reaction tranche; no quota may force an obscure,
  unsupported or dangerous bottle onto the shelf. Full-table-only elements may
  honestly say why no runnable example is installed and link to safe structural
  or nuclear context instead.
- **Interaction:** selecting a cell lists installed substances/materials that
  contain that element, then separately lists runnable reactions/lessons and
  their required co-materials. Search accepts symbol, localized element name,
  formula and common material name. Coverage badges and empty states come from
  generated registry/route data, never a parallel hand-maintained UI list.
- **Acceptance:** generated coverage report and regression fixture; default/full
  toggle snapshots at desktop/mobile/reduced motion; Fe/Cu/Zn remain reachable
  in the default view; Po/At/Fr/Ra and synthetic elements do not appear there;
  every displayed count round-trips to a real shelf key and every “runnable”
  link replays successfully through the engine.
- **Out of scope:** inventing a compound per element, implying every element is
  safe to handle, or using visual completeness as a substitute for model
  coverage.
- **First UI slice (2026-08-27):** the existing 118-element structural table is
  retained behind a remembered full-table toggle. The default curated lab table
  keeps high-value Mn/Fe/Cu/Zn and excludes Po/At/Fr/Ra/synthetic elements;
  cells show a count generated from the live shelf's parsed formulas and empty
  cells remain honest. Remaining work: include expanded material recipes,
  generate route/lesson capability levels, replace the curated symbol set with
  a reviewed data artifact, and add component/mobile accessibility snapshots.

### BRD-080 — Molecular viewer selection spike

- [ ] **Status:** open/decision gate. **Size:** small-medium. **Depends on:**
  BRD-012.
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

### BRD-081 — Molecular/crystal viewer integration

- [ ] **Status:** blocked on BRD-080. **Size:** medium-large. **Depends on:**
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
- **Acceptance:** dated comparison and explicit oracle/runtime decision.

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

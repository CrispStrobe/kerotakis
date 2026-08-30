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

- [x] **Status:** done. **Size:** large. **Depends on:** BRD-002 and the existing
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
- **Foundation checkpoint (2026-08-28):** `kerotakis-data` now exposes the
  shared offline quarantine contract: versioned SHA-256 snapshot manifests,
  deterministic candidate serialization, exact per-field provenance/licence,
  explicit field-and-licence review policies, and same-identity conflict
  reports. Review returns eligible/rejected fields and cannot mutate the
  runtime registry.
- **Review-tooling checkpoint (2026-08-28):** the offline
  `quarantine-review` binary verifies snapshot manifests against raw bytes,
  canonicalizes candidate fixtures, applies review policies, and emits a
  deterministic record/identity/field-level refresh diff. A checked-in
  synthetic fixture pins the required directory and manifest shape. These
  commands only print review artifacts; none can write a runtime pack.
  Remaining BRD-003 work is units normalization, the parser fuzz target and a
  provenance lint consumable by BRD-010/011/013/060.
- **Gate checkpoint (2026-08-29):** the remainder lands and the task closes.
  Units normalization converges 201 reviewed upstream spellings — `g·cm⁻³`,
  `g/cc`, `℃`, `deg C`, `°F`, `kcal/mol`, `J mol-1 K-1`, `mg/L`, `wt%`, `ppm`,
  `Da`, `Å`, `M⁻¹cm⁻¹` — onto the `Dimension`/`Unit` vocabulary DATA-001
  already defines, with affine temperature scales rather than bare factors. A
  spelling fixes the physical quantity, never the semantic field: `g/L` and
  `J/(mol.K)` each serve two dimensions, so the target field's declared
  dimension picks the reading and a mismatch is refused. Everything outside
  the table — bare mass, bare energy, wavenumbers, `Mg`, `ppt` — is a typed
  rejection carrying the original string; nothing falls back to
  `Dimension::Other`. A 71-case checked-in fixture pins each spelling, and
  round-trip and idempotence hold over the whole table.
  A quarantined field now carries the source's unit spelling verbatim and a
  runtime field policy may declare its dimension, so review is where an
  external spelling becomes a `Unit` and records both.
  The `quarantine` fuzz target covers the external-bytes surface — snapshot
  manifests, candidate fixtures, promotion policies, unit spellings — and
  asserts both invariants the framework rests on: canonical quarantine bytes
  are stable across a re-parse, and an unpinned snapshot is always refused.
  The promotion lint is one function (`lint_promotion`) and one command
  (`quarantine-review lint`), so BRD-010/011/013/060 call the same gate from
  either side. It refuses missing per-field source or licence, a licence
  outside the runtime data lane (in a candidate *or* in the policy that would
  admit it), raw bytes that no longer hash to the pinned manifest, candidates
  claiming a snapshot they did not come from, and eligible-field lists naming
  fields the record does not carry. `tools/provenance-lint.sh` now runs that
  gate over the checked-in fixture in both directions: the clean flow passes,
  the tainted one is refused.

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

- [ ] **Status:** open; slices in flight. **Size:** large/data-heavy.
  **Depends on:** BRD-010 and BRD-011.
  - `BRD-012.S02` — P0 school essentials plus the gated barium pair.
    Ammonium chloride, iron(III) chloride and sodium sulfate land as the
    three P0 salts named in the triage list below; barium chloride and
    barium hydroxide land as the P2 toxic virtual-only pair, gated behind
    a `ToxicSoluble` safety row and never entered as a household material
    recipe. Three supporting records ride with them because the engine
    needs them to speak at all: `NH4+` and `Ba+2` are the database master
    species dissolved ammonium and barium book back as, and `BaSO4` is the
    registry solid the Barite phase precipitates into — which is EXP-30's
    open "BaCl2 sulfate row". The litmus/indicator material of item 1 is
    NOT in this slice: it needs colour-state machinery.
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
  audited acid-salt component; pepper waits for aggregate-solid appearance.
  Isopropanol identity, 70% v/v household solution, safety and boiling-range
  volatility land in checkpoint 5 below; its combustion route still waits for
  feed thermochemistry. Dyes and paints are covered by BRD-075 optical material
  roles. These are
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
- **Checkpoint 5 implemented:** pure isopropanol is a searchable registry
  identity backed by public-domain U.S. federal property data. Localized 70%
  rubbing alcohol expands as its labelled 70/30 volume mixture with water,
  preserving the explicit additive-volume approximation. The shared safety
  surface marks its resolved alcohol component as flammable; the thermodynamics
  library supplies a range-checked NIST Antoine correlation around its normal
  boiling point. Combustion remains explicitly unmodelled until compatible feed
  thermochemistry is installed.
- **Checkpoint 6 implemented:** localized 5% household bleach and 5% ammonia
  cleaner surrogates expand into the already modelled NaOCl/NH3 aqueous
  components. Their density and concentration assumptions remain visible; no
  brand additives are guessed. Mixing the named cleaners now reaches the same
  danger warning, limiting-reagent consumption and chloramine-gas evolution as
  the pure reagents, with the warning emitted before the computed chemistry.
- **Checkpoint 7 implemented:** localized sparkling water and a bounded cola
  surrogate now expand into water plus finite CO2 doses, so the installed
  gas-liquid/headspace model—not an animation constant—computes dissolved
  carbonate, acidity, escaping fizz and sealed pressure. Cola additionally
  resolves a small phosphoric-acid fraction while keeping sweetener, caramel
  colour, caffeine and flavours as conserved unresolved mass; it makes no brand
  or nutritional claim.
- **Checkpoint 8 implemented:** `Backpulver`/baking powder is available as a
  versioned heat-activated surrogate with resolved sodium bicarbonate and
  starch plus a conserved unresolved acid-salt fraction. At room temperature
  it remains still; sufficient computed heating reaches the installed balanced
  bicarbonate-decomposition route and evolves stoichiometric CO2. Wet and
  double-acting activation remain explicitly unclaimed until the actual acid
  salts and their dissolution kinetics are installed.
- **Checkpoint 9 implemented:** localized tap-water and 3.5% seawater
  surrogates expand into installed major-ion species with explicit density and
  regional/compositional assumptions. The aqueous engine computes hardness and
  ionic-strength consequences; concentrating named seawater through the normal
  evaporation operator precipitates computed sodium chloride. Minor seawater
  ions, dissolved gases and organics remain conserved unresolved mass rather
  than being silently discarded.
- **Checkpoint 10 implemented:** ground black pepper/`Pfeffer` is a localized,
  fully conserved unresolved plant-powder recipe with a reviewed floating-grain
  role. On quiet water it persists as a visible surface layer; adding the
  existing dish-soap surrogate computes a dose-bounded central clearing,
  records a `SurfaceSpread` event, retains the resulting state in scene JSON,
  and animates the grains toward the rim in the web vessel. Soap-first order
  deliberately emits no sudden-spread event. This is the familiar classroom
  observable only—not a universal surface-tension coefficient, molecular
  surfactant model, Marangoni solver, or CFD particle trajectory.
- **Checkpoint 11 implemented:** `pepper-and-soap.lab` exposes the surface
  model as a child-facing, replayable experiment rather than a hidden command
  combination. It compares pepper-first and soap-first vessels, explains why a
  fresh surface is required, and is placed in the lesson picker's `start here`
  sequence. The lesson uses only water, pepper and dish soap and keeps the
  empirical-model boundary in its narration.
- **Checkpoint 12 implemented:** fresh/compressed baker's yeast is distinct
  from dry yeast under localized `Frischhefe`/`Presshefe` and fresh/compressed
  English names. The surrogate is 70% resolved water and 30% conserved yeast
  solids, with catalase activity scaled to equal dry solids. Because its water
  is already present, it enters the existing peroxide kinetics immediately;
  dry yeast retains its measured-time hydration ramp. Strain, age, cold-chain,
  brand and storage effects remain explicitly unresolved.
- **Checkpoint 13 implemented:** sucrose is now a canonical `C12H22O11`
  species and localized granulated table sugar/`Haushaltszucker` expands to it
  exactly. A new finite neutral-solute rung moves crystals into the aqueous
  phase only up to a declared conservative room-temperature capacity; excess
  sugar remains visibly solid and mass is conserved. The runtime record uses
  an openly curated approximation rather than redistributing NIST SRD data.
  Sucrose currently contributes dissolved-particle count but makes no pH,
  activity, dissolution-heat, caramelization, fermentation, or combustion
  claim; those remain separate reviewed checkpoints.
- **Checkpoint 14 implemented:** localized vegetable/cooking oil is a fully
  conserved unresolved household mixture rather than a fictional pure
  molecule. Its reviewed material role and representative 0.92 g/mL geometry
  parameter produce a persistent pale-yellow upper layer on water; aqueous
  food-colour optics remain confined to the lower layer. Scene volume includes
  both phases while aqueous concentration calculations still use only resolved
  solvent volume. Decanting and multi-vessel mixing now transfer unresolved
  homogeneous liquids proportionally, so the oil survives real pours and
  replay. This bounded checkpoint does not yet claim emulsions, droplet
  dynamics, oxidation, hydrolysis, combustion, or an edible-oil composition.
- **Checkpoint 15 implemented:** `oil-water-colour.lab` turns the material
  layer model into a guided child-facing density and polarity activity. It
  compares blue-water-then-oil with oil-then-red-food-colour, so learners can
  predict and see that the aqueous dye colours the lower water phase rather
  than tinting the oil. The lesson is in the curated `start here` sequence,
  carries staining/adult-supervision guidance, and names the current droplet,
  emulsion, detergent and lava-lamp boundaries instead of implying animation
  support that the engine has not computed.
- **Checkpoint 16 implemented:** dish soap now has a bounded aqueous-emulsifier
  role activated by the real timed magnetic-stir operator. Surfactant dose and
  delivered stir travel compute a finite dispersed vegetable-oil volume; scene
  JSON reallocates it from the upper oil layer into a cloudy aqueous emulsion,
  while retaining any undispersed oil above. Persistent emulsion state then
  coalesces during `wait` with a declared five-minute teaching half-life and
  emits events as the oil layer returns. A no-detergent stirred control remains
  separated. This is explicitly not a CMC, droplet-size, viscosity, brand,
  transient hand-shake, or CFD model.
- **Checkpoint 17 implemented:** the existing `oil-water-colour.lab` now
  continues into the newly installed emulsion model instead of ending with a
  stale “not modelled” boundary. Learners add dish soap, deliver a replayable
  500 rpm ten-second stir, inspect the cloudy 92%-dispersed state, then wait one
  coalescence half-life and see half the droplets return to the upper layer.
  The golden lesson contract pins all sixteen events and the conserved final
  vessel contents.
- **Checkpoint 18 implemented:** localized whole milk/`Vollmilch` is now a
  conserved household colloid rather than a fictional pure molecule. Its
  approximately 87% water enters the installed liquid model; fat, protein,
  lactose, minerals and natural variation remain one explicit unresolved milk
  fraction. A bounded opaque-colloid role computes a warm-white, dilution-
  dependent cloudiness that survives proportional pouring, while shared scene
  bookkeeping now includes unresolved homogeneous-liquid mass and visible
  volume without leaking them into aqueous concentrations. This checkpoint
  deliberately stops before acid curdling, spoilage, fermentation and the
  detergent-driven “magic milk” surface motion; those require distinct,
  testable transitions rather than decorative animations.
- **Checkpoint 19 implemented:** adding real acetic-acid inventory, including
  localized household vinegar, to the milk surrogate now computes a bounded
  acid-dose curdling response. The event and scene contract report the formed
  fraction and conserved aggregate curd-solids mass (not wet yield);
  dispersed opacity falls as milk solids join warm-white clumps over cloudy
  whey. The web vessel draws those clumps
  and animates their formation only from the typed `CurdlingChanged` event.
  Vinegar without milk and trace doses remain unchanged controls. Parameters
  are calibrated only to the familiar classroom milk-and-vinegar ratio: this
  is not a casein speciation, cheese-yield, food-safety or spoilage model.
- **Checkpoint 20 implemented:** `milk-curds.lab` makes the curdling model a
  discoverable, replayable fair test. Equal 100 mL milk portions receive 1 mL
  and 10 mL of localized 5% household vinegar: the trace-dose control remains
  dispersed while the second vessel emits the computed curdling event and
  shows curds over whey. The child-facing notes prohibit tasting lab material,
  ask for adult permission before using food, and distinguish the aggregate
  teaching model from a recipe, food-safety check or cheese-yield prediction.
  The full lesson replay also advances the intentional shared wording change
  from “colourless” to “white” for bright, fully opaque suspensions.
- **Checkpoint 21 implemented:** red, yellow and blue food-colour additions to
  the whole-milk surrogate now retain their resolved dye moles as localized
  surface spots instead of instantly tinting the whole vessel. A subsequent
  recipe-declared dish-soap dose computes a bounded spread transition and
  emits `SurfaceColourSpread`; scene JSON carries each spot's colour, relative
  amount and computed extent, and the web vessel draws event-driven coloured
  streaks with a reduced-motion fallback. Real stirring emits
  `SurfaceColourMixed`, releases the exactly conserved dye inventory into the
  existing Beer–Lambert bulk-colour path, and removes the surface geometry.
  Plain-water dye controls retain normal homogeneous optics. This observable
  is calibrated to the ACS “Colors on the Move” activity and explicitly does
  not claim CFD, a molecular milk composition, universal surface tension, or
  literal streak trajectories.
- **Checkpoint 22 implemented:** `magic-milk.lab` exposes that surface state as
  a discoverable, replayable child-facing fair test. One vessel receives three
  localized colour drops followed by dish soap and therefore emits the
  computed spread event; an equal control is stirred before soap, explicitly
  homogenizing its dyes and preventing a false rainbow event. The lesson is in
  the picker's curated `start here` sequence, warns about staining and tasting,
  and invites changes to dye and detergent dose while repeating the empirical
  model boundary.
- **Checkpoint 23 implemented:** pouring, multi-stream mixing and filtration
  now explicitly homogenize any localized food-colour surface state before
  moving resolved liquid. A proportional decant therefore splits the real dye
  inventory exactly between source and target without leaving impossible
  surface spots behind; receiving pours also disturb pre-existing spots. Each
  actual transition emits `SurfaceColourMixed`, while a zero-fraction decant
  remains a no-op. This closes the interaction lifecycle without inventing
  droplet trajectories during a pour.
- **Checkpoint 24 implemented:** dissolved table sugar and baker's yeast now
  enter a finite timed fermentation pathway instead of remaining inert. The
  conserved aggregate reaction consumes sucrose and water and produces four
  moles each of resolved ethanol and carbon dioxide per mole of sucrose;
  `GasProduced` drives real bubbles while `Fermented` reports the sugar, gas,
  alcohol, effective yeast dose and elapsed time. Dry yeast follows its
  existing hydration clock, equal-dry-solids fresh yeast is immediately
  active, and a smooth bounded temperature envelope replaces unrestricted
  extrapolation. Sugar-water and yeast-without-sugar controls remain still.
  The absolute rate is an explicit classroom surrogate informed by measurable
  baker's-yeast CO2 experiments—not a strain-growth, oxygen-switching,
  inhibition, flavour, food-safety or brewing model.
- **Checkpoint 25 implemented:** `yeast-fermentation.lab` turns the pathway
  into a discoverable three-vessel fair test on the bench's shared clock:
  sugar-water, yeast-water and sugar-plus-yeast receive equal relevant doses,
  but only the complete third condition produces computed CO2 and ethanol.
  The lesson is grouped under `rates`, warns against tasting or tightly sealing
  a real active fermentation, and suggests dose, culture form and temperature
  comparisons while preserving the biological-model boundaries.
- **Checkpoint 26 implemented:** seven localized familiar solids now preserve
  object identity while expanding exactly into installed canonical matter:
  table salt/`NaCl`, a calcium-carbonate chalk stick/`CaCO3`, magnesium ribbon,
  zinc strip, an iron-nail surrogate, copper wire and aluminium foil. The first
  six immediately inherit real dissolution, carbonate-acid or electrochemical
  routes where applicable; geometry remains explicit metadata rather than an
  invented kinetic multiplier. Aluminium foil deliberately retains the
  engine's passivation/model boundary. Epsom salt's hydrate prerequisite is
  completed in checkpoint 29. Bare `salt`/`Salz` remains unclaimed as a
  chemical class.
- **Checkpoint 27 implemented:** named 5% white vinegar plus the named
  calcium-carbonate chalk object now reaches a balanced portable reaction,
  `CaCO3 + 2 CH3COOH -> Ca2+ + 2 CH3COO- + H2O + CO2`, rather than depending
  on a PHREEQC-only strong-acid demonstration. Chalk and acid are finite,
  calcium and acetate remain resolved aqueous ions, and open/sealed vessel
  boundaries reuse the ordinary gas-evolved/gas-contained events. The
  discoverable `chalk-vinegar.lab` compares water and vinegar controls so the
  browser's existing chemistry-driven bubbles visualize only the reacting
  vessel; it does not claim a kinetic bubble-size distribution.
- **Checkpoint 28 implemented:** localized steel wool/`Stahlwolle` is a
  conserved surrogate with 98% resolved iron and a visible 2% unresolved
  alloy/coating remainder. Naming hematite (`Fe2O3`) in the runtime registry
  unlocks the existing NASA-CEA open-air Gibbs route: ignition consumes the
  iron, draws finite oxygen from the explicit atmospheric reservoir, produces
  reddish-brown iron(III) oxide, computes released energy and drives the real
  ignition visual. The fibrous form explains the familiar demonstration but
  does not yet claim a measured surface-area rate; a nail and steel wool still
  share the explicit ignition-zone threshold until surface kinetics land.
- **Checkpoint 29 implemented:** Epsom salt/`Bittersalz` is now the real dry
  hydrate epsomite, `MgSO4·7H2O`, rather than anhydrous magnesium sulfate plus
  fictional liquid water. Its PubChem identity and the shipped USGS WATEQ4F
  `Epsomite` phase are runtime data. A dispensed crystal remains one solid
  species; when the aqueous engine dissolves it, the magnesium and sulfate are
  speciated and exactly seven moles of crystal water per mole of epsomite enter
  the liquid ledger. Retail additives, dehydration, dissolution heat and grain
  size remain explicit boundaries.
- **Checkpoint 30 implemented:** concentrating a dissolved named Epsom-salt
  sample through the ordinary evaporation interaction now grows computed,
  visible epsomite crystals rather than generic `MgSO4` or a decorative solid.
  The USGS saturation model decides when precipitation begins, the hydrate
  phase rebinds seven waters per formula unit, and dissolved plus crystalline
  magnesium is conserved. Crystal habit, nucleation delay, seed crystals and
  slow real-world evaporation time remain future morphology/kinetics work.
- **Checkpoint 31 implemented:** localized iron filings/`Eisenfeilspäne` and
  quartz-rich play sand/`Spielsand` make magnetic separation a complete
  child-directed material interaction. Sand resolves 95% canonical `SiO2`
  while conserving a 5% variable-mineral remainder; filings resolve to
  magnetic `Fe`. The `magnet` tool moves only the real iron inventory into a
  receiver, leaves silica and unresolved grains behind, narrates both sides,
  and preserves mass. The existing shelf, receiver-vessel and `magnet`
  interaction expose the sequence; field strength and individual grain
  trajectories remain explicit future physics.
- **Checkpoint 32 implemented:** dilute 1% Lugol iodine solution/
  `Lugol-Lösung 1%` expands to water, retained KI and iodine. Iodide now
  supports a bounded aqueous iodine inventory instead of leaving a fictional
  crystal sediment, and cornstarch/`Speisestärke` produces a computed
  blue-black optical response from the broad 620 nm amylose-polyiodide band.
  A no-starch control remains brown; inventory bounds, phase transfer and the
  visible positive/negative result are regression-tested. Individual
  polyiodide speciation, exact binding stoichiometry, botanical amylose
  fraction and temperature-dependent helix changes remain explicit limits.
- **Checkpoint 33 implemented:** `starch-iodine-test.lab` turns the Lugol route
  into a controlled child-directed comparison. Equal water and indicator doses
  begin brown; only the vessel receiving named cornstarch becomes blue-black.
  The sequence uses localized shelf materials and ordinary `inspect` state, so
  native and web clients replay the same computed optics. Flour, potato,
  unknown white powders, heating and botanical-source comparisons remain
  withheld until their own reviewed material/temperature models exist.

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

- [x] **Status:** complete on `brd040/cantera-audit` (2026-08-29). **Size:**
  medium. **Depends on:** BRD-012 and current Cantera-YAML/kinetics support.
- **Checkpoint 2026-08-29:** full report in
  `provenance/brd-040-cantera-audit.md`; machine-readable licence verdicts in
  `provenance/sources.toml`; rejection matrix executed by
  `crates/kerotakis-core/tests/mechanism_cantera_audit.rs`. No runtime FFI and
  no mechanism file shipped, per the acceptance criterion. Four findings:
  1. **Parser bugs fixed.** Reaction orders were derived from *net*
     stoichiometry rather than each side of the equation, giving a wrong rate
     order and a mis-scaled pre-exponential wherever a species appears on both
     sides (6 of 29 reactions in Cantera's `h2o2.yaml`, 18 of 325 in
     `gri30.yaml`). The default activation-energy unit ignored the `energy` and
     `quantity` directives, misreading `units: {quantity: mol}` by 10³. Phase
     reaction selectors and named reaction sections were ignored. Unknown keys
     — including `orders`, `SRI`, `Tsang`, `negative-A` and nested `units` —
     were silently dropped; all six raw structures now refuse anything outside
     a documented allowlist.
  2. **Smallest additional subset for BRD-041 is three rate-law items** —
     reversible three-body reactions, reversible falloff (Troe and Lindemann),
     and negative activation energies — **plus one piece of document handling**:
     select a phase rather than validating every phase in the file, since real
     mechanism files pair an ideal-gas phase with a real-gas variant of the same
     species. With those, H₂/O₂, N₂/NOₓ and CH₄ + CO teaching mechanisms are
     fully expressible. PLOG is already supported and unused by them; Chebyshev,
     NASA9, explicit orders and plasma rates are not needed.
  3. **Licence verdict: every audited mechanism is oracle-only.** None of
     GRI-Mech 3.0, Ó Conaire, Boivin, the syngas sets, FFCM-1 or San Diego
     carries a redistribution grant, and Cantera states it "is not claiming to
     grant a license to" the mechanisms it ships. PLAN.md's claim that those
     files are BSD-3 redistributable is **wrong** and is corrected in this
     change. BRD-041 must author its own reduced mechanisms from primary
     literature, find a genuinely CC-licensed one, or obtain written permission.
  4. **C-API verdict: no gap; BRD-042 stays parked.** Nothing BRD-041 needs
     requires Cantera's C API, and linking it would not touch the licensing
     problem that actually blocks the mechanism packs.
- **Candidate/licence:** Cantera BSD-3-Clause **for its code only**; the shipped
  mechanism files carry no grant (audited 2026-08-29). Mechanism files and their
  original provenance/licences require separate review. Primary project and licence:
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

- [ ] **Status:** open, and now blocked on a sourcing decision rather than on
  engineering. **Size:** large/data-heavy. **Depends on:** BRD-040 (complete).
- **BRD-040 finding (2026-08-29):** *no* audited mechanism may ship as
  runtime-data — not GRI-Mech 3.0, Ó Conaire, Boivin, the syngas sets, FFCM-1 or
  San Diego. None carries a redistribution grant, and Cantera states it "is not
  claiming to grant a license to" the mechanisms it ships. Three routes remain,
  in order of preference: author project-original reduced networks from
  primary-literature rate constants with per-reaction source records (the
  pattern KIN-001…003 already uses); find a mechanism under a real open licence;
  or obtain written permission recorded as a `LicenseRef-` grant. The parser
  work is small by comparison — three rate-law items and phase selection. Full
  reasoning and the ordered candidate list: `provenance/brd-040-cantera-audit.md`.
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

- [x] **Status:** complete (2026-08-30). **Size:** medium. **Depends on:** GUI scene graph and
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
- **Evidence:** `kerotakis_core::authority` provides serde-stable typed
  proposals, replay seeds, explicit vessel/bench/tray/floor destinations,
  chemistry-owned break/spill event shapes, presentation-only motion policies,
  and receipt-driven cumulative transfer reconciliation. The executable
  `scene_authority` tests pin host serialization, different frame cadences,
  reduced-motion/headless/background endpoints, exact interruption, and
  refusal/malformed-proposal non-advancement. BRD-073 still owns emitting the
  reserved break/spill events and creating material-holding spill state.

### BRD-071 — Rapier rigid-body integration

- [x] **Status:** complete. **Size:** medium-large. **Depends on:** BRD-070.
- **Candidate/licence:** Rapier Apache-2.0 with deterministic wasm builds.
  Primary project: <https://github.com/dimforge/rapier>.
- **Scope:** prototype glassware/apparatus collision, stacking, tipping and
  dropping against current 2-D bench needs before choosing 2-D or 3-D. Use
  catalog footprints/ports as collider sources; chemistry-breaking thresholds
  remain explicit apparatus data and engine events.
- **Acceptance:** deterministic replay on supported hosts, keyboard/touch
  equivalents, no tunnelling in the drop corpus, measured bundle/performance
  budget, and a go/no-go versus simpler local collision handling.
- **Delivered tasklist:**
  - [x] isolate Rapier 2-D from chemistry authority behind versioned,
    quantized replay inputs and bounded untrusted-input limits;
  - [x] validate a six-item prototype collider/port catalog and exercise
    stacking, tipping, dropping, collision proposals and an 18-case CCD corpus;
  - [x] compile mouse, pen, touch and keyboard endpoints to identical canonical
    intents while reduced-motion/headless/background modes remain visual only;
  - [x] pin a serialized replay SHA-256 golden for Linux/macOS CI host parity;
  - [x] measure release-native timing and a retained wasm payload, with stable
    determinism/payload gates and advisory shared-runner timing thresholds.
- **Decision/evidence:** **go with optional Rapier 2-D** for tactile bench
  collision, stacking, tipping and drop proposals; retain the simpler local
  path for deployments that omit the feature. The current bench has no depth
  interaction or rendering contract, so 3-D adds cost without an accepted user
  endpoint and is a no-go for this milestone. The 20-object/360-tick probe was
  byte-identical across three runs (`efb244de…ce0ea`), measured 0.100 ms p95
  and 0.113 ms maximum per step on the reference x86_64 Linux host, and the
  conservative standalone wasm upper bound was 392,897 gzip bytes (below the
  768,000-byte gate). `tools/brd071_evaluate.py` makes the reproducible gates
  executable; CI runs the golden on both supported desktop hosts.

### BRD-072 — Salva fluid-visual integration

- [x] **Status:** complete/no-go. **Size:** medium-large. **Depends on:**
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
- **Delivered tasklist:**
  - [x] build bounded Salva 2-D prototypes for water pouring, authoritative
    oil/water layers and a high-viscosity syrup;
  - [x] accept only provenanced density, viscosity and surface-tension values,
    while keeping all coefficients and particles presentation-only;
  - [x] map the accepted BRD-070 transfer fraction independently per phase,
    and prove deliberate render-particle loss cannot alter that endpoint;
  - [x] improve `fluidScene` with accepted-fraction scaling, viscosity damping,
    surface-tension droplet sizing, deterministic event seeding and a strict
    reduced-motion/no-animation path;
  - [x] measure deterministic replay, phase order, particle-loss isolation,
    standalone wasm payload and explicit 60/30 fps thresholds.
- **Decision/evidence:** **no-go for shipping Salva in the interactive path;
  retain Salva as a reference and ship the improved lightweight `fluidScene`
  path.** Three reference runs reproduced the exact
  visual trace (`a472b73…0aeaf8`), retained authoritative chemistry through a
  forced 50% particle decimation and preserved bottom-to-top phase order. The
  standalone Salva wasm upper bound was modest at 48,990 gzip bytes, but the
  96-particle, 120-step stress frame measured 35.99 ms p95 and missed the
  explicit 33.33 ms/30 fps reference budget (and therefore 16.67 ms/60 fps).
  More decisively, its dependency closure includes archived
  `generational-arena` (`RUSTSEC-2024-0014`) and MPL-2.0 code rejected by the
  shipping licence policy; the measured prototype was therefore removed from
  the product build graph instead of weakening either gate. The existing path
  already has a 9 ms governor, economy grid, static/reduced-motion endpoint and
  no extra runtime boundary. `tools/brd072_evaluate.py` keeps the stable gates
  and named-reference timing decision executable.

### BRD-073 — Spills, tipping, drops and breakage

- [x] **Status:** complete (2026-08-30). **Size:** large. **Depends on:**
  completed BRD-071 and the closed-no-go BRD-072 outcome.
- **Scope:** add operator/event semantics for controlled partial pours, bench
  spills, vessel tipping, collision damage and recovery/cleanup. A broken
  vessel creates recoverable consequences and transfers its contents to a
  typed spill compartment; safety reruns against exposed/combined material.
- **Integration:** undo/replay, story inventory, disposal quests, cabinet
  replacement, Burst, accessibility and notebook evidence.
- **Acceptance:** mass/element/energy ledgers close across every failure path;
  identical chemistry with and without animations; hazardous spills emit
  precise safety events; save/load migration and undo cannot duplicate stock.
- **Completed tasklist:**
  - [x] authoritative typed bench/tray/floor spill compartments and cumulative
    partial-pour reconciliation;
  - [x] deterministic collision thresholds, vessel breakage, full-content
    transfer, cleanup/recovery and stable replacement-vessel identities;
  - [x] combined exposed-material safety reruns with sorted contributor species
    plus cross-location safety findings;
  - [x] mass, element and energy conservation, zero-fraction/no-break no-ops,
    animation/reduced-motion/headless endpoint parity;
  - [x] legacy-save defaults, serialized spill state, exact replay and
    snapshot-undo recovery without stock duplication;
  - [x] Burst-style incident presentation, static reduced-motion equivalent,
    accessible live status, hazard feed cards and durable notebook evidence.

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
- **Quantitative-catalysis checkpoint implemented (2026-08-27):** catalyst
  selection is no longer a boolean presence test. Effective dissolved KI
  concentration enters with the measured first-order iodide dependence;
  catalase scales with effective enzyme loading and a Michaelis–Menten
  substrate-saturation correction; MnO₂ consumes material-lot mass, density,
  particle diameter and suspended fraction to obtain nominal exposed area.
  Magnetic-stirrer tip speed supplies a bounded external mass-transfer gain.
  Thus dose, grinding and stirring now change oxygen production, foam growth,
  overflow and reaction heat through one kinetics path. Regression tests cover
  twofold initial KI/enzyme dose response, fourfold household-dose ordering in
  integrated oxygen and foam, twofold MnO₂ loading, tenfold area gain from
  grinding, bounded mixing acceleration, catalyst retention, and the complete
  household peroxide + soap + yeast/KI visual outcome. The guided
  `elephant-toothpaste-catalyst-dose.lab` comparison gives two equal 3%
  peroxide/soap vessels 0.25 g and 1 g KI on one shared ten-second clock so the
  resulting foam/overflow difference is directly visible. The mixing pass now
  transfers declared solution catalysts such as KI from solid inventory into
  the aqueous phase, emits `Dissolved`, preserves their moles, and prevents a
  dissolved catalyst from being rendered or gravity-settled as sediment.
  The real-browser CI self-test now runs that two-dose KI experiment through
  the shipped worker and UI, then asserts two rendered foam columns, visible
  out-of-glass overflow, dose ordering, and the absence of the former
  unsupported-contact warning. This closes the core-to-Wasm-to-DOM path rather
  than accepting a scene value that the child cannot see.
  Shelf clicks, periodic-table additions and reagent drops now model the
  physical dispense as explicit, replayable one-second contact ticks. A
  nonreactive addition stops after its first tick; computed bubbling or growing
  foam keeps the gesture advancing for at most ten seconds, with short visual
  pacing between scene updates. Kinetic reactions therefore blubber, rise and
  overflow after the gesture instead of requiring a child to discover the
  textual `wait` command; authored lessons and command-line scripts retain
  complete control of time.
  Dry yeast recipe components now retain material-lot provenance and their
  first liquid-contact time. Only that reviewed surrogate receives a bounded
  hydration/activity ramp (warmer water shortens its teaching time constant);
  purified catalase remains immediately available, catalyst moles remain
  conserved, and old saves without lot provenance retain their prior result.
  The correlation is explicitly editorial and does not claim universal
  activity for a yeast brand, age or batch.
  Catalase now also has a smooth high-temperature activity envelope: moderate
  warmth still accelerates the curated pathway, while very hot water suppresses
  it instead of allowing Arrhenius extrapolation to grow without limit. This is
  an instantaneous teaching envelope, not yet irreversible denaturation memory;
  the latter requires explicit exposure history and remains a boundary.
  Remaining
  boundaries are yeast-brand/age calibration, irreversible denaturation
  history and inhibition, catalyst pore/BET area, adsorption and pore-scale
  diffusion.

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
- **Air-collar checkpoint implemented:** the deployed burner now exposes a
  0–100% air collar beside its 0–100% gas/flame control. Zero flame is visibly
  extinguished and cannot heat or invoke ignition. Opening the collar moves a
  declared near-field teaching efficiency from 55% to 100%, changes the
  rendered low-air yellow flame to the open-air blue flame, and compiles the
  resulting energy to the same replayable `heat` operator. This does not claim
  fuel depletion, soot or CO; those remain dependent on typed burner fuel/air
  chemistry rather than renderer colour.
- **Liquid-fuel checkpoint 1 implemented:** touching the guided flame to an
  open vessel of ethanol now reaches CEA's separately parsed, feed-only liquid
  record, admits only the matching ethanol vapour plus named stable flame
  gases, and computes fuel depletion, CO2/water-vapour products and reaction
  energy from the bundled Apache-licensed NASA-9 data. HP remains preferred;
  when its liquid-feed bracket fails, a declared TP fallback uses the explicit
  ignition-zone temperature and says so in provenance. This does not yet claim
  sustained pool-fire geometry, sealed/oxygen-starved combustion, soot/CO, or
  isopropanol identity and volatility data.
- **Isopropanol checkpoint 1 implemented:** pure isopropanol is now a searchable
  registry identity with reviewed room-liquid properties, and household 70%
  rubbing alcohol is a localized, fixed 70/30 v/v recipe rather than a falsely
  relabelled mass mixture. The shared safety screen classifies the alcohol as a
  flammable liquid, and a NIST Antoine fit computes its volatility across the
  stated range around the normal boiling point. A flame held to the 70% aqueous
  mixture remains an explicit combustion-model boundary: the bundled CEA subset
  has no isopropanol feed thermochemistry, so this checkpoint does not fake fuel
  depletion, products, heat release or sustained burning.

### BRD-077 — Element coverage score and progressive periodic table

- [x] **Status:** complete (2026-08-30). **Size:** medium. **Depends on:** BRD-000, BRD-012 and
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
  cells remain honest. The completed slice now includes expanded recipes,
  replay-proved lesson/codex routes, capability levels, a reviewed default-view
  artifact, localized search and browser-level desktop/mobile/reduced-motion
  accessibility coverage.
- **Completion tasklist and DoDs (2026-08-30):**
  - [x] Generate a deterministic, versioned 118-entry coverage report from
    pure species and expanded material recipes. **Done when:** its reviewed
    regression fixture is stable, every example resolves to a live shelf key,
    identity-only cells remain present, and native plus wasm boundaries agree.
  - [x] Derive runnable content links from shipped lesson/codex scripts.
    **Done when:** required co-materials all resolve to the shelf, lesson kits
    are generated from source, and every advertised source passes the existing
    real-engine replay/lint gates.
  - [x] Finish the progressive table interaction. **Done when:** coverage
    levels, substance/material search, honest empty states, direct lesson and
    experiment actions, remembered lab/full modes, keyboard names, mobile
    layout and reduced-motion behavior pass focused web tests and production
    build.
  - [x] Integrate and audit both host transports. **Done when:** native/wasm
    schemas match, Fe/Cu/Zn and Po/At/Fr/Ra/synthetic inclusion rules regress,
    formatting/clippy/focused suites/full preflight pass, the PR merges without
    unrelated work, and the resulting GitHub `main` workflow is green.

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

### BRD-093 — Permissive thermochemical-engine target gate

- [x] **Status:** closed no-go for universal runtime (2026-08-30); optional
  native/build-time oracle only. **Size:** small decision record. **Depends
  on:** a named high-temperature condensed-phase experiment that the existing
  CEA path cannot represent.
- **Candidate/licence:** Thermochimica code is BSD-3-Clause. Its thermodynamic
  databases and individual CALPHAD assessments are separate works and require
  record-by-record redistribution review. EQ3/6 and OpenGeoSys are also
  BSD-3-Clause; AqEquil wraps EQ3/6; ChemEQL is MIT. Code licences alone do not
  make their databases, packaged binaries, or dependency closures shippable.
- **Target verdict:** none is a new universal Kerotakis engine. Thermochimica
  requires a Fortran toolchain plus BLAS/LAPACK and has no demonstrated
  maintained Rust-to-wasm/iOS/Android distribution path. OpenGeoSys is a large
  native THMC application rather than a bench library. EQ3/6/AqEquil and
  ChemEQL duplicate the shipped IPhreeqc aqueous domain. PhreeqcRM is useful
  only after choosing multidimensional porous-media transport, which remains
  outside the product mission.
- **Portable rule:** a core runtime model must build behind the same Rust API
  for browser wasm, Android, iOS, macOS and Windows. A native-only backend may
  be an optional acceleration or oracle, but it may not own a learner-visible
  capability or produce a result unavailable in the PWA. Tauri does not make
  arbitrary native libraries web- or mobile-portable; installed shells use a
  native Rust core while the browser uses the wasm core. Native-only workspace
  adapters declare `[package.metadata.kerotakis] runtime = "native-only"`;
  `tools/portable-dependency-lint.py` rejects any such package in the
  `kerotakis-wasm` dependency closure and runs in preflight.
- **Reopen gate:** name the experiment and educational observable; identify a
  cleared database; prove deterministic C-ABI builds on Windows, macOS,
  Android and iOS; measure a browser-wasm build or specify a portable
  Kerotakis fallback with answer-level conformance fixtures. Until then,
  Thermochimica may generate reviewed fixtures externally, like pycalphad,
  but does not enter any shipped dependency graph.
- **Immediate path:** finish the already-claimed BRD-030 `feos` spike for
  portable fluid thermodynamics and use the completed BRD-040 verdict for gas
  kinetics: extend the portable Cantera-YAML/diffsol slice; keep full Cantera
  FFI parked unless a concrete capability gap survives BRD-042's gate.

### BRD-094 — GPU fluid and volumetric-rendering decision record

- [x] **Status:** frontend WebGPU spike only; Taichi/NanoVDB backend adoption
  closed no-go (2026-08-30). **Size:** small decision record. **Depends on:**
  completed BRD-070 and BRD-072; reopen implementation only for a named visual
  that the shipped lightweight `fluidScene` cannot express.
- **Authority and placement:** chemistry continues to own amounts, phase,
  temperature, pressure and accepted transfers. GPU state is disposable
  presentation state. Run an optional accelerator beside the renderer in the
  webview so particles/textures do not cross Tauri JSON IPC; feed it the same
  bounded scene/event contract in PWA and installed shells. A deterministic
  Canvas/WebGL/lightweight fallback remains the release baseline for old
  Android WebViews, reduced motion, headless tests and absent WebGPU.
- **Taichi verdict:** Apache-2.0 and useful for native research prototypes, but
  its AOT/C-API backend matrix is not a demonstrated browser + Android + iOS +
  macOS + Windows distribution. The official C-API tutorial currently lists
  Vulkan, OpenGL, x86 and CUDA and explicitly says Metal is unsupported; it
  does not provide the claimed transparent Metal/DX12 universal binary. Python
  may generate artifacts at build time, but no Taichi runtime enters a shipped
  Kerotakis target without passing BRD-093's target gate.
- **NanoVDB verdict:** current OpenVDB/NanoVDB is Apache-2.0, not BSD-3.
  NanoVDB is a compact GPU/CPU sparse-grid representation, principally for
  read access, rendering and collision queries; its topology is static at
  runtime. It neither calculates combustion chemistry nor supplies a fluid or
  smoke solver. Consider its C99 `CNanoVDB`/`PNanoVDB` layouts only after a
  measured sparse-volume transport bottleneck exists; do not stream NanoVDB
  buffers through ordinary Tauri IPC.
- **WebGPU candidate:** `jeantimex/fluid` is MIT and demonstrates browser SPH
  plus 2-D/3-D PIC/FLIP, marching cubes, raymarching and screen-space fluid.
  Treat it as algorithm/reference code, not a drop-in dependency: it is an
  application, requires a WebGPU-capable browser, and credits/ports earlier
  implementations whose exact copied-code provenance must be audited before
  reuse. Prefer a small project-owned WGSL effect scoped to one accepted
  observable over importing the whole demo.
- **Reopen/acceptance gate:** first name the missing visual—volumetric flame,
  smoke plume, foam or a genuinely 3-D pour. Then measure it against
  BRD-072's existing 9 ms governor on the low-end Chromebook/Android floor;
  require no chemistry/particle coupling, no readback per frame, graceful
  device-loss fallback, reduced-motion equivalence, deterministic endpoint
  snapshots, shader/source licence records and identical scene semantics on
  all hosts. Visual fidelity alone cannot make WebGPU mandatory.
- **First named candidate (2026-08-30):** a procedural envelope for a live
  vessel `ignite` event. Existing magnitude and curated flame-colour inputs
  make it bounded without inventing chemistry, while the current fallback is
  only a two-path SVG flame. Do not render generic evolved gas as smoke (there
  is no soot/particulate authority), infer burning from temperature alone, or
  copy `jeantimex/fluid` WGSL: its MIT repository identifies two earlier MIT
  ports but supplies no per-file lineage map. Implement project-owned WGSL
  from published fire-rendering ideas and record that provenance explicitly.

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

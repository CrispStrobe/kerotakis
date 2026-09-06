# Kerotakis — history

What landed, and what it taught us. Newest first, one line per item:
task number, what it did, and the PR, branch or commit that carries it.

Planning lives elsewhere — `PLAN.md` holds intent and open work,
`ROADMAP-GUI.md` and `ROADMAP-Webapp.md` hold the numbered open tasks, and
`BREADTH.md`, `CAPABILITIES.md`, `EXPERIMENTS.md`, `KIDS.md`,
`OPTIMIZATION.md` own their own task registries. **Task numbers are never
renumbered and never reused**; a finished item keeps the number and the date
it had while it was open, which is why a few numbers appear twice below.

---

## 2026-09-06

- **ANIM-2** — matter and pressure: precipitate count from moles, grain size from molar volume, piston height from V=nRT/P (PR 2 of GUI-099)

### Lessons

- a visual counts only when its size, count, colour, tempo or position is a function of an engine-computed quantity; a picture of the verb drawn at a constant does not.

## 2026-09-05

**BRD**

- **BRD-012** — S02 landed P0 school salts plus the gated toxic barium pair
- **BRD-012** — S03 landed the food-chemistry identity tranche (glucose, fructose, malic/citric acid, cellulose)
- **BRD-012** — S04 landed six species (fuel-gas alkanes, helium, naphthalene, H2S) reaching the CEA equilibrium route
- **BRD-012** — S05 landed uranium as a decay-heat species, flipping th-122 to computed
- **BRD-014** — S02 landed 13 materials including mayonnaise, hand sanitiser, petrol and sugar water
- **BRD-014** — S03 landed the biology tranche including chlorophyll and nylon 6,6 species
- **BRD-014** — S04 landed dry-solid electrical resistivity for seven pure metals
- **BRD-014** — S06 landed diesel surrogate species flipping th-048 via a curated autoignition comparison
- **BRD-014** — S06 landed insulator resistivity for nine glass/ceramic/silicon materials, flipping mat-053 to computed (PR #413)
- **BRD-014** — S07 landed alkaline-battery and battery-terminal-corrosion materials, flipping mat-058/mat-071 to computed
- **BRD-014** — UV attenuation model landed, flipping bio-111 to computed after S05 first assessed the row as unclosable
- **BRD-014** — corrosion barrier checkpoint stopped stainless/galvanized/painted steel recipes rusting like bare iron, closing mat-014
- **BRD-020** — phase 3 landed FamilyRouter wired after CuratedEquilibrator with an esterification/hydrolysis pack v1 (#395)
- **BRD-023** — S01 landed PolymerHeatResponse distinguishing thermoplastic softening from thermoset charring, flipping mat-025
- **BRD-023** — alcohol-oxidation reaction added to ORG_REACTIONS, unblocking bio-064 at the parser (#412)
- **BRD-023** — galvanic-couple corrosion shipped, flipping mat-099 and five other rows to computed
- **BRD-023** — peroxide-melanin kinetic rate law shipped, closing bio-112 as curated on an editorial timescale
- **BRD-031** — identity-seam checkpoint closed: nine fluids keyed by Standard InChIKey with per-parameter provenance lint
- **BRD-031** — open-licence search found seven CC BY 4.0 PC-SAFT candidate papers but none promoted; liquid density stays unsourceable
- **BRD-032** — Langmuir adsorption isotherm shipped for methyl orange on charcoal, closing bio-103 via a new adsorbed ledger
- **BRD-032** — dry-ice species and sublimation enthalpy route shipped, closing th-026
- **BRD-032** — liquid-nitrogen cryogen route shipped coupling freeze/boil energy transfer, closing th-123
- **BRD-032** — pressure-dependent boiling routed through the BRD-031 pack by InChIKey, closing th-019/th-020
- **BRD-041** — packs wired into the engine via the slow clock, with an integrator Jacobian-probe and depletion-gate fix (PR #404)
- **BRD-041** — three project-original combustion mechanisms authored from primary literature shipped as CC BY 4.0 (PR #393, PR #399)
- **BRD-050** — bounded biochemical route shipped: pH window, irreversible denaturation, food-carried enzyme role, three fermentations
- **BRD-052** — respiration equation added to ORG_REACTIONS as an unapplied combustion-enthalpy quote, not a metabolic model
- **BRD-052** — vocabulary-only tranche gave ten biology rows their words; only bio-051 and bio-085 gained real mechanisms
- **BRD-060** — S01 landed silicon/doped_silicon resistivity objects, flipping mat-066 to computed without a doping model

**KID**

- **KID-5** — galvanic coupling extension (BRD-023): lower-E metal corrodes for both, zinc-corrosion companion entry, barrier table for passive/paint films (crates/kerotakis-core/src/corrosion.rs)

### Lessons

- a condensed-gas phase such as dry ice or liquid nitrogen must never be matched to a database mineral phase by formula alone, since its stability is a temperature threshold, not a solubility product.
- a curated reaction and a kinetic rate law answer different question shapes; a process that takes minutes needs a rate law, because a curated reaction would fire and complete on the same step the reagent is added.
- a strong base in the aqueous tail must be read from measured free_hydroxide/free_proton, not solute_charge, which conflates bicarbonate alkalinity with free base.
- BRD-041's 25-prompt acceptance floor was not met as counted: the three rows that moved closed through a registry-identity fix reaching CEA equilibrium, not through the reviewed reduced mechanism the criterion actually asks for.
- milk's diffusible mineral buffer now characterises a beaker of milk near pH 6.7, but casein stays unresolved so a computed yoghurt pH is only a lower bound, not a prediction.
- BRD-014.S06 was independently claimed the same day by two unrelated shipped tranches (insulator resistivity and diesel identity); slice numbers are never reused, so the collision is left on record for a reviewer to reconcile.

## 2026-09-04

- **BRD-001** — missing corpus rows fell from 145 to 82 of 500 with protein species identified as the largest single blocker
- **KID-11** — foam observable generalized from one hardcoded reaction id to any gas-evolving reaction with a declared surfactant
- **KID-19b** — float/sink observable lands: a solid of known density floats or settles against its liquid and look says which (lessons/float-or-sink.lab)
- twenty-six materials landed (twelve inert solids, fourteen foods/fibres), moving 63 corpus rows out of missing with no expectation-mismatch drift

### Lessons

- counting how many corpus rows mention a substance overstates the payoff of adding it; a row can die on a different substance further down its own script.
- adding a named species can make a row pass without making it answerable, turning a passing test into a lie the day its second condition still is not modelled.
- a recipe's declined-in-writing refusal is only as durable as the next editor remembering it; add an automated check that the refusal still holds.
- a bare-word alias policy already granted in one locale was denied in another for the same registry items, serving one language's learners worse for no chemistry reason.
- a verdict classifier that checks for a bystander event before checking the main result can score a chemically more accurate answer as worse than a less accurate one.
- a corpus prompt's declared parse-boundary can go stale the moment the missing species ships; a lint that re-checks the declaration catches it before anyone notices.
- a finding fixed by several separate follow-up tasks over days still needs someone to re-read and close the original row, or it sits marked wrong long after the bench answers it.
- listing a deliberately out-of-scope experiment as partial instead of declined reads as a promise nobody intends to keep and costs the next planner real time.
- a database-phase-matching function keys phases to registry solids by composition, so a message naming a phase in the database but not the registry means one species entry, not a routing project.
- a symmetric matching rule silently asserted a claim that has no direction, reporting a salt as smelling of the acid it comes from until a per-substance concentration floor replaced the fallback.
- when no vendored database defines a solid phase at all, the correct deliverable is a refusal stating the concentration and the missing phase by name, not a workaround.
- the same solubility-and-cooling mechanism gives two different real answers for two different substances, and the interesting content was in running it on a second substance for contrast.
- a verdict recorded once and never re-run drifts from true-when-written to false-while-trusted; three of twelve spot-checked rows had already changed underneath their verdict.
- a prompt that claims to distinguish N things must be checked to actually produce N different answers; only six of fifty-five comparison questions scripted both conditions.
- a material property field read by exactly one caller in the whole codebase left six carefully sourced densities silently floating and sinking nothing until the mechanism was checked, not assumed.
- cargo test's fail-fast is per test binary, so a partially-green run means nothing failed before the first failure, not that everything else passed.
- a more accurate chemistry addition can cascade through an unrelated branch: precipitating a basic sulfate lowered pH past a threshold that flipped a bystander-metal comparison off the intended answer.

## 2026-09-03

- **KID-7** — temperature-dependent solubility and Event::Supersaturated land; sucrose is the first solute with a second reviewed solubility point (lessons/rock-candy.lab)
- **KID-12** — combustion module lands for paraffin/cellulose/sucrose with a limiting-oxygen-fraction model
- **KID-13** — oobleck and dancing-raisins land as bounded physical-mixture observables (lessons/dancing-raisins.lab)
- **KID-14** — slime lands as a calibrated dose response (PVA plus borax), closing the last unrunnable script in the first thirty
- **KID-19a** — measure v1 density / hydrometer lands, reading solution density with solute volume included
- **KID-20** — apple-juice and chalcanthite recipe data errors fixed; drink-recipe acid lint added (material_recipe.rs)
- **KID-21** — three ordering-trap refusals (grind/filter/cell) gained remedy text

### Lessons

- a curated reaction sitting ahead of the aqueous solver only fires if its reactant is still spelled the way the curated equation expects, so pouring reagents in the more common order silently killed it.

## 2026-09-02

**KID**

- **KID-1** — shelf reachable from terminal: normalize_material_name treats underscore/dash/space as one separator, kero materials and kero find added
- **KID-2** — curdling dose reads total acid inventory across a Bronsted pair instead of one species key (crates/kerotakis-core/tests/milk_curdling.rs)
- **KID-3** — hazard screen slice 1 landed: components arriving from one MaterialRecipe are no longer screened against each other
- **KID-5** — rusting landed as curated kinetic reaction iron-corrosion, four-arm EXP-34 comparison passes (lessons/rusting.lab)
- **KID-6** — boiling plateau lands: latent heat paid at the boiling point before the Gibbs minimiser sees the vessel
- **KID-8** — PigmentLadder generalizes the two-form Indicator to n-form chromophores; red cabbage lands as a four-form pH ladder (lessons/cabbage-rainbow.lab)
- **KID-9** — paper chromatography Rf mode lands, sharing one partition coefficient with the existing column mode (lessons/ink-chromatography.lab)
- **KID-17** — help regrouped to name all 31 verbs in script::VERBS; kero lessons lists 37 shipped lessons

**WORLD**

- **WORLD-003** — ship runtime catalog contract; hosts answer availability from engine, never serialize it
- **WORLD-004** — ship mission schema v2; all 35 shipped v1 quests parse unchanged
- **WORLD-005** — ship objective evaluator; evaluate_claim is the single source both status and completion read
- **WORLD-006** — ship transactional mission outcomes with absolute change sets, safe retry after failed write
- **WORLD-007** — ship localized-content lint over content tables, not just t() call sites
- **WORLD-008** — encode contaminated-sample vertical slice as 3 concurrent v2 quests with byte-pinned migration

**BRD**

- **BRD-020** — phase 2 landed a conservation ledger that refuses any template application not balancing atoms and charge

**EXP**

- **EXP-44** — excess-enthalpy function generalized from water-anchored to any verified organic binary pair (kerotakis-core/tests/heat_of_mixing.rs)

### Lessons

- a model's restriction to one solvent pairing was an accident of the first verified pair, not a claim about the chemistry, once the underlying math was checked to already be pair-agnostic.
- thirteen of thirty children's-experiment scripts failed on a species name already present in the registry, so the gap was reach into working chemistry, not the chemistry itself.
- a corpus run by a stranger finds shipped claims that do not reproduce precisely because the author's own corpus never drives the closed path.
- an alias nobody could type was recorded as a missing engine capability and filed against the task that had already built it, so an unreachable name lies to the coverage report as well as blocking a learner.
- A TOML authoring trap: root keys must precede the first [[claims]] table or they silently become part of that claim, because serde's flatten and deny_unknown_fields cannot combine to catch it.
- Mission-outcome change sets must be absolute next-state values rather than deltas, so replaying a commit after a failed write is indistinguishable from making it once, and a remembered failed write merges safely into the next commit.
- A drift test pinning the runtime catalog's verb tiers against the parser's own verb table caught two verbs (remove, stock) that nothing had tiered, showing cross-module consistency needs an explicit two-way test, not just one-way coverage.
- A translation-coverage gate that scans source for literal t("..") call sites cannot catch content reached only as a variable (mission text, labels, award names); it must walk the content tables themselves and check interpolation parity, with a ratchet baseline for existing untranslated debt.
- Routing both live status display and quest completion through one evaluate_claim function keeps them from ever disagreeing, and returning stable unmet-reason tags with parameters (never prose) lets clients localize without inventing the underlying arithmetic.

## 2026-09-01

- **EXP-30** — MIX-path parity closed: oxidation-state pin, candidate-list derivation and SELECTED_OUTPUT ordering fixed so mixed solves complete in one engine call (mix.rs::mix_solves_in_one_engine_call_without_falling_back)
- **WORLD-001a** — land pure AppSave v1 codec with deterministic encode/decode and legacy migration (commit 710aeaad; run 33479564633)
- **WORLD-001b** — land atomic last-known-good repository over injectable KeyValueStorage (commit 9a585d76; run 33481140466)
- **WORLD-001c** — integrate repository at session boundary with one-way Story-to-Sandbox copy (commit 24b9ddf9; run 33482173371)
- **WORLD-001** — ship versioned AppSave envelope with independent Story/Sandbox namespaces (commits 710aeaad/9a585d76/24b9ddf9)
- **WORLD-002** — prove Story/Sandbox mode isolation and equal-input equal-chemistry conformance (commit ecccf36b; run 33485348208 (#301/#302))

### Lessons

- a failed MIX solve was silently treated as advisory and re-solved via the direct path, masking three real engine bugs behind chemistry that looked right.

## 2026-08-31

- **BRD-031** — S01 rights audit closed a runtime-promotion no-go for six-fluid PC-SAFT parameters (provenance/brd-031-pilot-source-audit.md)
- **BRD-031** — fail-closed fluid contract and current-solver domain safety merged (PR #274, PR #279)
- **BRD-080** — 3Dmol.js 2.5.5 provisionally selected over Mol* under the smaller-adequate rule (168,749 vs 1,968,375 gzip bytes)
- **EXP-44** — acetone-chloroform re-audited against the engine: chloroform is not a species and UNIFAC main group 11 is absent
- **EXP-48** — water surface tension from IAPWS R1-76 and Jurin's-law capillary rise computed and pinned against reference points (kero properties water-surface-tension)
- **GPU-6a** — bench-owned injectable metrics registry, 32-session cap evicting diagnostics never presentation
- **GPU-6b** — probe contract hardening: the evaluator validates the raw v1 artifact instead of trusting its summaries
- **GPU-6c** — `tools/test-gpu-release-tools.sh` as the single local, preflight and CI entrypoint
- **GPU-6d** — end-to-end evidence manifest joining app metrics, paired probes, asset reports and provenance hashes

### Lessons

- feos ships no parameters directory and its own repository parameters tree carries no licence statement at all, so silence there must not be read as clearance.
- a computed quantity's valid range is bounded by its narrowest input's own validity, not its own wider range.
- no CI run, simulator or unavailable adapter may be turned into physical evidence; unavailable/headless evidence is valid but cannot pass.

## 2026-08-30

**BRD**

- **BRD-013** — USDA adapter shipped over 15 pinned Foundation Foods records; salt/oil/butter flagged as proximate conflicts
- **BRD-030** — feos spike closed go (scoped): adds liquid density/critical points/gases the existing thermo stack cannot compute (brd030/feos-spike)
- **BRD-070** — scene/chemistry authority contract shipped typed proposals, replay seeds and chemistry-owned break/spill events
- **BRD-073** — spill/tipping/breakage operator semantics shipped with conservation across every failure path
- **BRD-077** — progressive periodic table shipped: generated coverage report, lab/full toggle, capability levels, accessibility
- **BRD-093** — closed no-go for a universal thermochemical engine; portable-dependency-lint enforces native-only packages
- **BRD-094** — closed no-go for Taichi/NanoVDB GPU backend adoption; frontend WebGPU spike scoped to a named ignite-flame visual

**GPU**

- **GPU-1** — WebGPU lifecycle and fail-closed fallback: structural provider, adapter and device acquisition
- **GPU-2** — dynamic environment policy honouring reduced motion and document visibility, idempotent disposal
- **GPU-3** — ignition flame uniforms mapped only from a live authoritative `ignite` effect, never from temperature
- **GPU-4a** — shader ABI and fail-closed renderer core: analytic flame shader and canonical buffer writer
- **GPU-4b** — browser compiler and bounded canvas host under one policy-owned lifecycle
- **GPU-5a** — authoritative vessel integration; +14.7 kB minified / +5.7 kB gzip over GPU-4b

**GUI**

- **GUI-090** — reactant chips, concept and safety notes, and the lv3 drop rule
- **GUI-091** — reaction-class badge and before/after temperature with a delta chip, both confidence-encoded
- **GUI-095** — `balance` as a protocol command returning coefficients and the composition matrix they null
- **GUI-097** — one shareable result card exported as deterministic dependency-free SVG or 2x PNG

**CAP**

- **CAP-9** — shipped kero fit, bounded golden-section parameter fit recovering rate constant within 3%
- **CAP-12** — extended titrate endpoint grammar to pe and colour-persists via EXP-39
- **CAP-13** — fixed chematic molfile bridge via 0D structure route, closing stereo/isotope identity gap (Opus)

**I18N**

- **I18N-2** — map-screen vocabulary closed; `tools/i18n-slug-lint.py` in preflight, 308 slugs answered
- **I18N-3** — engine vocabulary gate derives all 267 substitutable terms from registry, colours, hazards and lessons

**EXP**

- **EXP-39** — titrate verb gained potentiometric and self-indicating-colour endpoints beside the existing pH default

### Lessons

- the Salva prototype passed every functional and determinism gate but was rejected on dependency hygiene (an archived RUSTSEC-flagged crate, MPL-2.0 code) and a missed 30fps budget.
- chematic's V2000 molfile writer cannot express stereochemistry or isotopes at any version, so the fix bypassed molfiles via InChI's own 0D structure API.
- the engine already knows how to withhold an undetermined potential, but a single-redox-couple flask swept of air returned a flat republished input value instead of withholding it.
- an open flask's atmospheric oxygen couple buffers pe near a constant value regardless of titration progress, so a naive pe-target claim is satisfied on the first drop.
- no shipped PHREEQC database speciates oxalate, so classic permanganate-oxalate standardisation must ride curated reaction rows rather than the coupled solver.
- a numeric verb slot must reject non-finite values outright, since 1e999 parses to infinity which serde_json cannot serialize.
- "absent rather than guessed" has to be enforced, not intended: most of what a bench does is not a reaction, so stirring gets no class badge at all.
- a test caught that the `unknown` classification branch was unreachable, so the strictness existed only in the comment.
- a heat of mixing is `modeled`, not `computed`; a fitted UNIFAC hE and a solver result are different claims and must look different.
- marking a balancing exercise against the solver's coefficients would fail a correct multiple; mark arithmetic over the learner's own vector instead.
- `t()` falling back to its argument means an English node inside a German map fails nothing, which is why a slug lint had to derive its key set from the fields components actually read.
- a translation key for a retired concept is an error, not a spare: two German strings still asserted that stirring changes nothing after the engine learned that it does.
- the equation must be taken from anywhere in the accepted command, not from the one event that won the priority list — a curated precipitation showed none because `reaction_occurred` carries the equation and `precipitated` wins.

## 2026-08-29

- **BRD-002** — stockroom StockLedger shipped finite bottles with proportional transfer proven already correct
- **BRD-003** — gate closed with units normalization for 201 spellings, a fuzz target and lint_promotion
- **BRD-010** — PubChem adapter shipped a 100-record fixture with InChI agreement on both independent identity routes (brd010/pubchem-adapter)
- **BRD-011** — ChEBI adapter shipped release 253 pinned, reporting tautomer/protonation families rather than merging them
- **BRD-040** — Cantera audit fixed reaction-order/activation-energy parser bugs and found no mechanism carries a redistribution grant (brd040/cantera-audit)
- **BRD-040** — audit found no BRD-041 need requires the Cantera C-API, parking BRD-042 as a no-go
- **EXP-30** — seven sealed-unknown salt quest specs authored, six single unknowns plus a two-unknown capstone, no engine change needed (quests/two-white-jars.toml)
- seven quests upgraded from event-only to value claims; two new sealed-unknown quests authored (weak-acid pH, metal molar mass)
- **GUI-096** — toolbox relations carry purpose, validity and source on both bindings, English and German

### Lessons

- PubChem supplies identity only, never a promotable experimental physical property, because its licensed structured values and its CC-clean sources never coincide on one field.
- Cantera's shipped combustion mechanisms (GRI-Mech 3.0, Ó Conaire, Boivin, syngas sets, FFCM-1, San Diego) carry no redistribution grant despite an earlier PLAN.md claim they were BSD-3.
- flame temperature is intensive while combustion energy is extensive, so a temperature value-claim must read the flame itself rather than assume a calorimetric delta-T into water.
- a validity range shown only after the number has already let the mistake happen; the drawer must state purpose, validity and source first.

## 2026-08-28

- **BRD-003** — quarantine framework foundation shipped snapshot manifests, per-field provenance and conflict reports
- **BRD-003** — offline quarantine-review binary shipped for snapshot verification and refresh diffs
- **GUI-062** — instruments drawn on the bench; freestanding stations user-positionable and collision checked
- **GUI-090** — structured result card projected from the accepted command's typed events, no second engine call

## 2026-08-27

**GUI**

- **GUI-033** — apparatus palette and instrument panel; chromatography column, magnifying inspection and Geiger slices
- **GUI-059** — fourteen apparatus slices: computed transfer colour, pour motion, filtration, still, separatory funnel, magnet, settling, centrifuge, stirring, gas tests, waft, piston, evaporation, dilution, gas sweep
- **GUI-074** — bench focus controls: cabinet and journal collapse to edge rails, persisted per mode
- **GUI-075** — unobstructed vessel controls moved off the vessel body
- **GUI-077** — explicit action target: the dock names its vessel and shows its live volume, temperature and materials
- **GUI-078** — phase-colour cabinet with icon and text still the primary encoding
- **GUI-079** — first-run callout anchored inside the work surface
- **GUI-080** — honest apparatus motion: configured tools stay still until the engine is actually running
- **GUI-083a** — freely positioned instrument stations, dragged or arrow-key nudged, stored per lab mode
- **GUI-083b** — legible apparatus targets: drag grip, focus treatment, dotted sample route

**BRD**

- **BRD-001** — coverage classifier shipped a five-way disposition report and checked-in 500-entry baseline (codex/brd-001-baseline)
- **BRD-002** — MaterialRecipe schema landed with versioned recipes, aliases, ranges and unresolved fractions
- **BRD-002** — runtime add operator resolves recipes through bulk density with an explicit unresolved-material ledger
- **BRD-074** — first elephant-toothpaste slice shipped gas-rate/foam/overflow observables driven by peroxide kinetics
- **BRD-074** — quantitative-catalysis checkpoint replaced boolean catalyst presence with KI/catalase/MnO2 dose-response kinetics

**Other**

- breadth-audit findings converted into versioned BRD-000 500-prompt regression corpus with an EXP-to-BRD prerequisite map

### Lessons

- read the source vessel BEFORE the engine replaces the scene: transfer, filtration and drain visuals need the computed pre-transfer colour and layers.
- an aggregate-liquid colour can visibly drain the wrong phase; use the engine's own bottom-first layers when they exist.

## 2026-08-26

- **GUI-076** — world/home shell: Research Campus home, Story and Sandbox scoped independently, non-destructive migration
- **GUI-079** — progression-aware catalog: access-policy slice, then finite-stock dispenses consumed only by accepted transactions
- **GUI-080** — first vertical slice "the contaminated sample": case board, engine-typed outcome contracts, thermal baseline
- **GUI-059** — typed-event mapper covers twelve effect families; every scale factor names its event field

## 2026-08-25

- **EXP-30** — hydroxide precipitation matrix computes for six cations plus AgCl/CO2 tests, needing four engine repairs for Fe2+'s green hydroxide (kerotakis-phreeqc/tests/qualitative.rs)
- **EXP-31** — gas-test verbs (pop/glowing-splint/limewater/damp-litmus) landed as curated headspace tests (branch kero1/exp31-gas-tests)
- 32 quest TOML files authored across 26 EXP tasks, all passing kero quest lint (quests/)
- **GUI-058** — liquid layers in Scene JSON: additive per-layer species, volume, colour and density stacking (#30)
- **GUI-061** — volume-true fills from per-kind capacity and a volume-to-height profile (#36, kero-basic)
- **GUI-063** — in-experiment visual shelves: lessons and codex entries present their kit as a rendered strip (#36, kero-basic)
- **GUI-064** — animation of running tasks on one clamped, cancellable, reduced-motion-honest scheduler (#35, #37, #45)
- **GUI-065** — fluid dynamics as the transport layer: Eulerian stable-fluids grid plus Lagrangian surface particles
- **GUI-065a/b/c** — MAC stable-fluids core with pinned Rayleigh-Taylor, ledger-exact splash handoff, true-glass wall masks
- **GUI-066** — engine-evaluated quests: observe/answer in the protocol, 17 quests exported, QuestBar and claim cards
- **GUI-067** — instant restore via snapshot-token autosave with triple fallback
- **DATA-010** — `load_pack` end to end: a hash-verified .pack adds species to shelf and chemistry at runtime
- **WEB-003** — inventory in `hello`; PROTOCOL gains its `load_pack` row
- **GUI-060** — superseded by GUI-065: the scripted plume was replaced by the fluid-dynamics transport layer

### Lessons

- an animation may only relax toward the solver's answer, never past it; settled concentrations must converge to the engine's own layer volumes and colours.

## 2026-08-24

**CAP**

- **CAP-2** — shipped kero study one-parameter sweep runner, rayon-parallel, byte-deterministic (Fable)
- **CAP-8** — added Monte Carlo sampling to kero study with seeded percentiles and chart band series (Fable)
- **CAP-10** — wired MIX operator with three-body adiabatic mixing and a hard-water softening lesson
- **CAP-13** — adopted official MIT InChI library with build-time identity totality check (Fable)
- **CAP-13** — grew InChI-verified species tranche from 23 to 65 (Opus)
- **CAP-20** — wired 1-D transport verb onto existing CellChain solver (Opus)
- **CAP-20** — wired extract/drain/chromatograph/react verbs onto existing ungrammared physics (Fable)
- **CAP-23** — added NonAqueousEquilibrator with curated solubility and metal-inertness verdicts (Fable)
- **CAP-23** — curated permanganate-ethanol oxidation reaction with MnO2 deposition (kero1)
- **CAP-23** — curated silver metathesis reactions gated on dissolved solvent fraction only (kero1)
- **CAP-23** — grew organic solubility table from 8 to 65 rows across four solvents (kero-basic)
- **CAP-24** — added EXP-43 iodide-peroxide and iodate-bisulfite clock-kinetics rate laws
- **CAP-24** — curated EXP-13 vitamin-C iodine decolorisation with titration endpoint persistence
- **CAP-24** — curated EXP-14 amylase starch hydrolysis with enzyme gating, catalyst not consumed
- **CAP-24** — curated EXP-2 NaHCO3 thermal decomposition with temperature-gated firing
- **CAP-25** — added smell/waft verb and sealed-glassware Burst overpressure event (Fable)

**GUI**

- **GUI-015** — undo/replay/timeline as one cursor over the replayed log, plus snapshot/restore protocol commands
- **GUI-020** — lesson player: lessons walk as guided steps
- **GUI-025** — the equation strip renders reactions as balanced equations
- **GUI-026** — pour and stir shipped as SVG interactions
- **GUI-027** — utilities drawer / toolbox shipped
- **GUI-028** — voice input: a microphone drives the command bar
- **GUI-053** — concept map draws the concept DAG layered by longest prerequisite chain, with a cycle guard
- **GUI-055** — curriculum browser: all / by concept / by curriculum doors over the codex export

**EXP**

- **EXP-43** — iodide-peroxide and iodate-bisulfite Landolt clock rate laws landed with four new registry species
- **EXP-44** — excess enthalpy of mixing computed from UNIFAC temperature dependence as a vessel state function; acetone-water allowlisted, ethanol-water withheld
- **EXP-49** — nuclide ledger wired to bench chemistry: curated teaching isotopes decay inside wait, Geiger counter reads total Bq (tests/nuclear.rs)
- **EXP-50** — SN1/SN2/E1/E2 selectivity rules landed: six curated rules over two substrates with condition-flip tests (branch kero1/exp50-mechanistic-selectivity)

**OPT**

- **OPT-3** — replaced anonymous cache 5-tuple with named Rc-wrapped struct, hoisted env-flag reads (87e4608, kero1)
- **OPT-5** — replaced per-iteration Vec-of-Vec allocation in CEA Newton loop with one flat matrix (Opus)
- **OPT-9** — measured wasm-JS boundary at 139 crossings post OPT-6/7 and decided against building a fix
- **OPT-11** — landed client half of one-worker web engine, attaching IPhreeqc in-process via shared bridge

### Lessons

- a study runner surfaced that titrate delivered pure titrant by volume instead of a standard solution, correcting titration-curve semantics.
- curated data checked against handbook values found permanganate's molar absorptivity curated at 1.8x the literature value.
- a registry key can pass an identity gate for the wrong reason, as Al's stored InChIKey matched an independently-wrong hydride computation.
- evaluating a mixing state function's derivative at the vessel's own changing temperature lets the path leak into the result; anchoring to a fixed reference temperature restores path-independence.
- nucleons conserve exactly across radioactive decay but elements do not, so alpha/beta/gamma departures and the mass defect must be stated as boundaries rather than silently balanced.
- corpus audits converged from sixteen new tasks down to zero across eight collections, confirming the bench's wet-chemistry scope is finite and coverable.
- decomposing solve_once was proven move-only by diffing a whitespace-normalized multiset of the hunks against signatures and forced borrow-shape changes only.
- measuring the wasm-JS boundary after call-count fixes landed showed solver compute (24 ms/call) dominates marshalling, so a faster boundary would fix nothing.
- CEA's Newton loop allocated a fresh Vec-of-Vec every iteration inside a 60-iteration outer bisection, up to 24000 allocations per solve, fixed by hoisting one flat matrix.

## 2026-08-23

**CAP**

- **CAP-1** — wired kerotakis-thermo into the bench via the distil operator with full UNIFAC bubble point
- **CAP-3** — defined chart JSON contract and shipped CLI/PWA SVG renderer plus kero diagram txy (Fable)
- **CAP-4** — computed Pourbaix pe-pH predominance diagrams for Fe and Cu with water-stability lines (Fable)
- **CAP-11** — expanded reactive-hazard safety matrix from 4 to 142 species with CI totality test
- **CAP-12** — added titrate and dilute as first-class verbs with an auto-stepped titration curve
- **CAP-14** — turned licence policy into cargo-deny CI lint with synthetic copyleft proof
- **CAP-15** — resourced Antoine constants to Stull 1947 and added methanol/propanone/acetic-acid data (8e7e461, kero-basic)
- **CAP-16** — added temperature-coupled gamma to dew-point and flash solvers (Fable)
- **CAP-17** — added Rayleigh batch distillation and N-stage column with energy coupling (Fable)
- **CAP-18** — grew UNIFAC table to 6 main groups/30 interactions, fixed OH-CH2CO parameter bug
- **CAP-19** — built Python-thermo differential oracle for UNIFAC gamma and bubble points (Fable)
- **CAP-21** — generated species registry at build time from JSON, species.rs 1563 to 179 lines (Fable)

**OPT**

- **OPT-1** — added criterion benches to core/phreeqc/cea crates and recorded baseline medians (9a88ba7, kero1)
- **OPT-2** — added workspace release profile, wasm-opt -Oz pass, moved shared deps to workspace deps (kero1)
- **OPT-4** — indexed species registry lookup with OnceLock HashMap, replacing linear scan
- **OPT-6** — decomposed 968-line solve_once into six named move-only phases plus SolveSetup struct (e51d870, kero1)
- **OPT-7** — cached redox bisection trials, warm-started bracket, added residual-tolerance break (Fable)
- **OPT-8** — unified two chemical-formula parsers behind stoich.rs after a full differential check (11fe338, Fable)

**LIC**

- **LIC-001** — resolve store-permission text scope between LICENSE and NOTICE, pinned by test
- **LIC-002** — resolve curated-data store distribution licence/grant structure

**Other**

- 10 types (molecule graphs through electrode states) have Rust types/tests, await grammar and data
- lab grammar covers 17 commands, 8 instruments, 50 event types across 7 chemistry domains

### Lessons

- exact rational arithmetic was needed for balancer families since integer solutions cannot pass through floating point without drift.
- differential oracles catch what self-consistency tests cannot, as a chempy comparison caught a UNIFAC combinatorial bug internal tests missed.
- declining to model something must be loud, so Pourbaix cells where the engine fails render as explicitly unknown rather than being interpolated over.
- caching redox bisection trials plus warm-starting the bracket cut engine calls per equilibration from 272 to 20 without moving any test result.
- a checkbox without its acceptance evidence in the marking commit is a claim not a status, and this file was restored twice after replacements silently re-bound task numbers.
- SymEngine differentiates the ODE right-hand side symbolically once, feeding CVODE's dense Jacobian slot directly with exact f64 entries (no finite-difference approximation), while a separate sparsity-pattern pass picks the KLU sparse solver for large mechanisms.

## 2026-08-22

- **LIC-004** — inventory 13 workspace crates, 234 deps, 3 vendored dirs; all pass cargo-deny
- **LIC-005** — kero provenance lint rejects missing/ambiguous records, stale checksums, oracle leakage
- **DATA-003** — compile deterministic postcard runtime pack, 586KB JSON to 116KB, hash cd14829b
- **DATA-004** — load pack behind registry API verifying KREG magic/version/SHA-256 hash
- **DATA-005** — implement property-resolution ladder returning rung/uncertainty/validity, never naked default
- **DATA-006** — import 3 Wikidata CC0 identity crosswalks end to end through full pipeline
- **DATA-007** — import 3 compatible PubChem fields, reject 1 incompatible annotation with reason
- **DATA-008** — generate build-time PHREEQC derived indexes for 4 embedded databases
- **DATA-009** — generate reachable CEA subset matching 34 of 75 registry species to NASA-9 polynomials

## 2026-08-21

**AQ**

- **AQ-004** — boundary-aware headspace energy, owned in the shared checkout without touching the kinetics modules (codex-AQ)
- **AQ-005** — typed finite-capacity HFO surface interfaces with strong/weak site ownership and ligand-exchange water ledgers (codex-AQ)
- **AQ-006** — pH-dependent HFO adsorption oracle (codex-aq/aq-006-oracle)
- **AQ-007** — first finite-capacity cation-exchange slice (codex-aq/aq-007)
- **AQ-008** — first typed mineral solid-solution slice (codex-aq/aq-008)
- **AQ-009** — one evidence-producing PHREEQC-kinetics route (codex-aq/aq-009-phreeqc-kinetics)
- **AQ-010** — couple aqueous speciation to partial-freezing water phase to 0.05K, refuse below 252K (CI 32506920952; codex-aq/aq-010-partial-freezing)
- **AQ-011** — add 1-D cell chain with conservative first-order upwind transfer, passive tracer proven (CI 32508147378; codex-aq/aq-011-cell-chain)
- **AQ-012** — couple exchange/transport in CellChain; four-cell resin column closes calcium/sodium (CI 32509744496; codex-aq/aq-012-exchange-transport)
- **AQ-013** — couple surface/transport; four-cell HFO column matches PHREEQC TRANSPORT within tolerance (CI 32514634767; codex-aq/aq-013-surface-transport)
- **AQ-014** — publish R1 acceptance suite (5 outcomes) passing native, Wasm, cache, offline (CI 32516261610; codex-aq/aq-014-r1-acceptance)

**KIN**

- **KIN-001..003** — first generic-kinetics slice: reaction-network IR, both rate laws compiled through it, conservation lint (codex-kin/reaction-network, CI 32481885344)
- **KIN-004/005** — adaptive BDF integration over reaction extents replaces the explicit midpoint loop; `diffsol =0.16.2` audited MIT, no JIT in the portable graph (codex-kin/adaptive-integrator, CI 32484407871)
- **KIN-006** — mechanism-file front end: strict portable Cantera-YAML species and elementary Arrhenius parsing into the IR (codex-kin)
- **KIN-007** — third-body concentrations, species efficiencies, Lindemann/Troe falloff, gas-network execution over finite headspace (codex-kin)
- **KIN-008** — CLI-first runtime gas-mechanism simulation with an explicit finite sealed headspace and stable JSON reports (codex-kin)
- **KIN-009** — bounded sampled gas-mechanism trajectories at exact evenly spaced requested times (codex-kin)
- **KIN-010** — one- and two-region NASA7 thermochemistry and elementary reversible detailed-balance execution (codex-kin)
- **KIN-011** — instantaneous mechanism-rate diagnostics with an explicitly defined rate-determining candidate (codex-kin)
- **KIN-012** — pressure-dependent Arrhenius kinetics: pressure-grid validation, logarithmic interpolation, nearest-endpoint extrapolation (codex-kin)

**DATA**

- **DATA-001** — define typed registry schemas across 8 record families with units/uncertainty/source (CI 32518836256; codex-data/data-001-schema)
- **DATA-002** — export 75 current species to source records; byte-identical runtime behaviour (CI 32520759936; codex-data/data-002-export)

## 2026-08-20

- **P0** — four libFuzzer targets in `fuzz/`, including the `.lab` grammar
- **P3e** — `displacement.rs`: Nernst over computed activities, the activity series, displacement, galvanic `cell v1 v2`, hydrogen overpotential

## 2026-08-19

- **P0** — feasibility spike passed: IPhreeqc native + Emscripten wasm, four databases embedded, AgCl case identical on both

## Undated (early work, before dates were recorded here)

**ARCH**

- **ARCH-001** — freeze current lesson JSON contract and outputs as regression snapshot
- **ARCH-002** — add typed quantities for power, current, potential, area, flow, photon flux
- **ARCH-003** — introduce ConservedLedger in shadow mode, asserting mass/charge/energy agreement
- **ARCH-004** — introduce MaterialLot tracking additions/transfers independent of resolved species
- **ARCH-005** — introduce ResolvedState as invalidatable derived-state container for aqueous/thermal/phase info
- **ARCH-006** — add Compartment and Environment wrapping vessel as well-mixed liquid/solid
- **ARCH-007** — add Headspace and Interface types with migration-preserved save/log replay
- **ARCH-008** — define StateDelta so models propose ledger/phase/energy transfers, not mutate vessel
- **ARCH-009** — add transactional commit/rollback validating positivity and conservation before commit
- **ARCH-010** — define structured capability/validity reports replacing boolean applies concept
- **ARCH-011** — build first orchestrator path routing a water operation through planning to commit
- **ARCH-012** — migrate solvers one at a time; frozen corpus rerun after each migration
- **ARCH-013** — remove sequential direct mutation once every solver returns deltas
- **ARCH-014** — emit coverage manifest reporting claimed models, validity, observables per operation

**ORG**

- **ORG-001** — audit Indigo/InChI toolkit versions, keep runtime graph minimal or build-time
- **ORG-002** — define molecule graph with bond order, charge, isotope, stereochemistry, round-trip
- **ORG-003** — add canonical identity/formula derivation cross-checked against two independent tools
- **ORG-004** — add functional-group perception fuzzed via SMARTS/graph matching
- **ORG-005** — define atom-mapped transformation templates, linted before application
- **ORG-006** — implement esterification/saponification end to end across structure/kinetics/heat/separation
- **ORG-007** — cross-validate templates with RDKit oracle-only, persisting discrepancies not exports
- **ORG-008** — add conditions/incompatibility filters deciding whether a template match is claimed
- **ORG-009** — add confidence labels distinguishing computed/curated/estimated/qualitative/unsupported
- **ORG-010** — add reaction families one at a time with source audit and yield boundary
- **ORG-011** — add oracle enrichment (xTB/CREST, PySCF) as separate reviewed pipeline
- **ORG-012** — add polymer population state with conversion and molar-mass moments

**ELEC**

- **ELEC-001** — add explicit electrode/interface state serializing material, area, roughness, deposits
- **ELEC-002** — move Nernst/Faraday behaviour onto electrodes, existing cell tests unchanged
- **ELEC-003** — add reviewed kinetic parameter records for exchange-current/Tafel/overpotential
- **ELEC-004** — implement Butler-Volmer/Tafel kinetics tested at equilibrium and Tafel limits
- **ELEC-005** — add galvanostatic and potentiostatic control keeping charge/work in ledger
- **ELEC-006** — add ohmic and diffusion limits via boundary-layer model
- **ELEC-007** — add competing electrode reactions chosen from thermodynamics/kinetics/activities
- **ELEC-008** — add deposit/passivation state altering kinetics without changing elemental inventory
- **ELEC-009** — publish electrochemical acceptance cases identifying limiting mechanism per result

**INST**

- **INST-001** — define instrument contract: sampling, detection limit, calibration, uncertainty, provenance
- **INST-002** — migrate eyes, balance, thermometer, pH meter preserving deterministic ideal mode
- **INST-003** — add gas pressure/volume instruments validating ideal/non-ideal routing
- **INST-004** — add conductivity using approved mobility/conductivity data
- **INST-005** — complete UV-Vis/indicator measurements from CC BY/CC0/public-domain spectra only
- **INST-006** — add calorimetry recovering ideal enthalpy in zero-loss limit
- **INST-007** — add chromatography with ideal plates connecting peak area to recovered material
- **INST-008** — add qualitative-analysis workflows emerging from computed tests, never a scripted key

**LIC**

- **LIC-003** — define provenance/sources.toml schema for source id, licence, checksum, attribution, decision
- **LIC-006** — add cargo-deny with explicit runtime/dev licence graph policy
- **LIC-007** — generate attribution notices via cargo-about, diffed against source manifest in CI
- **LIC-008** — generate CycloneDX/SPDX SBOMs for CLI, web, iOS, Android release payloads
- **LIC-009** — define signed model-pack manifest with hash, licence, ABI, min app version
- **LIC-010** — segregate oracle jobs into separate caches/outputs, excluded from release artifacts
- **LIC-011** — add dependency/data PR checklist covering terms, shipping, database rights, removal
- **LIC-012** — audit release artifacts as golden baseline for automated payload reconciliation

**THERMO**

- **THERMO-001** — audit FeOS/vle/water-property versions and parameter files before adding dependencies
- **THERMO-002** — put existing ideal VLE behind FluidModel trait, preserving water/ethanol tests
- **THERMO-003** — add phase-specific property records with ranges and sources
- **THERMO-004** — complete UNIFAC from approved parameters only, blocking proprietary consortium table
- **THERMO-005** — implement bubble/dew and TP flash validated against ideal/pure-component limits
- **THERMO-006** — add HP and UV flashes coupling energy and phase state
- **THERMO-007** — integrate one approved equation-of-state backend with cleared parameter set
- **THERMO-008** — add liquid-liquid split validated on one allowlisted extraction case

**ADV**

- **ADV-001** — environmental pack from approved PHREEQC data for soils/treatment/weathering/acidification
- **ADV-002** — photochemistry IR with light-source state, approved spectra/cross sections/quantum yields
- **ADV-003** — materials/metallurgy pilot expanding cleared CEA subset for iron/copper process
- **ADV-004** — polymer kinetics pilot coupling one network to population moments/heat ledger
- **ADV-005** — nuclear module design defining nuclide ledger and CC0/public-domain decay source
- **ADV-006** — keep biochemistry parked pending approved data source and dedicated architecture

**APP**

- **APP-001** — add powered heat sources replacing free evaporation with power/duration/loss
- **APP-002** — add condenser and receiver connections proving conservation in distillation
- **APP-003** — add repeated ideal stages and reflux; ethanol-water azeotrope boundary holds
- **APP-004** — add separatory-funnel stages compared against repeated small extractions
- **APP-005** — add recrystallization tracking crystals, mother liquor, cooling energy

**BRD**

- **BRD-000** — curiosity corpus v1 shipped 500 stable prompts with typed dispositions and a runner (codex/brd-000-curiosity)
- **BRD-071** — decision: go with optional Rapier 2-D; deterministic replay golden and wasm payload measured under budget
- **BRD-072** — decision: no-go for shipping Salva in the interactive path; kept the improved lightweight fluidScene path
- **BRD-075** — transparent-dye and Kubelka-Munk opaque-pigment mixers shipped for food-colour/watercolour/acrylic surrogates
- **BRD-076** — guided Bunsen-burner placement/valve/air-collar controls shipped compiling to authoritative heat/ignite operators

**GUI**

- **GUI-002** — typed Event enum crosses every host as structured JSON; register prose rendered engine-side
- **GUI-005** — parse-only endpoint validates a command without executing it, on both client hosts
- **GUI-011** — bench canvas v1: SVG vessels from Scene JSON with true-volume fills, headspace, badges
- **GUI-029** — affordance manifest: `affordances.json` maps every grammar verb to a surface; conformance rejects missing and invented verbs
- **GUI-087** — toolbox relations gained purpose and validity; GUI-096 later closed the item by adding the citation

**KIN**

- **KIN-001** — define reaction-network IR with stoichiometry, rate law, catalysts, uncertainty
- **KIN-002** — compile two current rate laws into IR with identical lesson outputs
- **KIN-003** — add reaction-network conservation lint for elements, charge, sites, electrons
- **KIN-004** — audit and add DiffSol with approved permissive feature graph only
- **KIN-005** — implement adaptive implicit integration with positivity and event detection

**P2**

- **P2** — `PhreeqcEquilibrator`, weak acids and buffers from the database's own equilibria, content-addressed result cache
- **P2** — `kero prewarm` exports 9 lessons / 73 steps to 26 unique solver results in 20 KB, served bit-identically with zero engine calls
- **P2** — database routing by validity domain; carbonate chemistry with an open vessel; true speciation in the expert register
- **P2** — phosphate via minteq routing, pitzer.dat for concentrated brines, hard-water chemistry

**AQ**

- **AQ-001** — compile finite Headspace to GAS_PHASE for sealed CO2/water system
- **AQ-002** — add sealed, open-reservoir, pressure-controlled, and swept headspace boundary modes
- **AQ-003** — support inward/outward gas transfer; limewater/excess-CO2 sequence conserves carbon

**CAP**

- **CAP-5** — added named-relations layer (Arrhenius, Eyring, Nernst, H-H, Debye-Huckel) with kero calc (f0af26a)
- **CAP-6** — added water/gas property correlations with provenance and kero properties command (3e79ed2)
- **CAP-7** — replaced f64 Gaussian elimination with exact Rational64 arithmetic for balance families (94bbdb7)

**P4**

- **P4** — `kero codex lint` — the check that makes the format worth having
- **P4** — `kero serve --mcp` — the bench as an MCP server over the `--json` surface
- **P4** — codex schema and content in TOML; `codex/models.toml` in eight supersession tiers; `codex/concepts.toml` with 189 concepts

**WEB**

- **WEB-001** — ship generated pre-warmed cache with no unapproved oracle-derived material
- **WEB-002** — move both Wasm engines into one module Worker with async client commands
- **WEB-004** — make offline install atomic; required assets all cache or install fails

**ANIM**

- **ANIM-1** — thermal truth: boil held at the engine's own plateau, steam sized by moles of vapour, blackbody incandescence (PR 1 of GUI-099)
- **ANIM-3** — the three events that drew nothing: emulsion droplets, fermentation tempo, UV transmission, electrolysis on shared charge (PR 3 of GUI-099)

**CI**

- **CI-001** — build scientific artifacts once, reused across Wasm/bridge/browser/publication jobs
- **CI-002** — separate fast/full/oracle validation tiers across PR, main, scheduled jobs

**P1**

- **P1** — `kerotakis-cli` REPL + batch + `--json`; 256-case conservation proptest over mass and energy
- **P1** — bench state machine: `Bench`/`Vessel`, mutating and measuring operators, enthalpy bookkeeping, L0 on prospective state

**P2g**

- **P2g** — NASA-9 thermochemistry from CEA's `thermo.inp`; Gibbs minimiser (Gordon & McBride, Lagrange multipliers)
- **P2g** — `ThermalEquilibrator` wired into the bench; the `ignite` operator; adiabatic vessels solve enthalpy-conserving

**P3k**

- **P3k** — `codex/rates.toml`, 14 entries for the rate practicals
- **P3k** — `kinetics.rs`: rate law with Arrhenius k, time as a shared state dimension, catalysis as a lower Ea

**PERF**

- **PERF-001** — add bundle/model-pack budgets for size, memory, init, solve latency
- **PERF-002** — add node-level cache keys including model version, dataset hash, constraints

**REL**

- **REL-001** — add release gate refusing publication unless provenance/deps/SBOM/signatures all pass

### Lessons

- the EXP-33 sublimation route was athermal: it moved matter without moving energy, which a phase change must always do.
- the worst animation defect is not an absence but a constant: `steaming` gated on 368 K is wrong under vacuum, under pressure, for a salted solvent and for every non-water solvent.
- IPhreeqc under Emscripten needs `-fexceptions` at compile AND link plus `-sSTACK_SIZE=8MB` — found the hard way.
- pitzer.dat carries Latin-1 comment bytes, so databases embed with `include_bytes!`, never `include_str!`.
- loading a PHREEQC database resets the accumulated state; the wrapper documents the quirk rather than working around it silently.
- database routing by validity domain is mandatory: minteq.v4 and wateq4f disagree, and wateq4f lacks free H3PO4.
- naming a valence state in a `SOLUTION` block decouples that element in PHREEQC — it gets its own mass balance and exchanges electrons with nothing.
- a half-titrated redox couple sits at its own E°, exactly as a half-neutralised acid sits at its pKa, and it falls out of free-energy minimisation rather than being encoded.
- a narrowed bracket is not a struck balance: past the equivalence point no pe in water's stability field balances the books, so an unbalanceable beaker is refused rather than answered.
- the plausible pedagogical story (permanganate titrations use H2SO4 because chloride oxidises) died on the first experiment that could have confirmed it — the electrons are owed by the solvent, and swapping the acid changes nothing.
- oxidation-state bookkeeping is the explanation layer, not the solver; inconsistency is the detector (O = −2 on H2O2 fails, and that failure is the signal), and two unknown elements means refuse.
- an average oxidation state is honest: Fe3O4 really is +8/3, and an average is exactly what electron counting needs.
- the ten-degree rule falls out of Arrhenius rather than being applied.
- observation has a detection limit; bookkeeping stays exact, but what the bench reports as visible does not.
- derivation over tables: the equilibrator's hand-maintained constants were replaced by the database's own equilibria.
- a species with no role was not ignored but destroyed: the aqueous rebuild replaces the vessel's contents with the computed state, and the defect stayed invisible because an earlier bug meant the solver never ran on such a vessel.
- the midpoint integrator could freeze — a half-step pushing the copy's last reactant below the discard threshold returned a zero rate, costing two million substeps to advance seven milliseconds.
- `codex lint` verified ranges but not sentences, and a sentence is what the learner reads; the lint now pulls unit-carrying numbers back out of the register prose and asks whether the replay produced anything like them.
- that prose-number lint is advisory and stays advisory: a good entry legitimately quotes a literature activation energy or a textbook figure held up for contrast, and failing those would train authors to strip real content out.
- Kerotakis, not PHREEQC, owns time integration and the vessel clock; PHREEQC KINETICS is kept only as an opt-in development comparator, so numerical ownership stays with the project's own integrator.
- Reaktoro 2.13 was ruled ineligible as the pH-adsorption oracle because surface complexation with diffuse-layer electrostatics is still an open upstream gap, so a narrower project-owned mass-action oracle was used instead and scoped as an edge-direction check only.

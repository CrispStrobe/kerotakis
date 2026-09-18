# Kerotakis — Development Plan

A virtual chemistry laboratory that computes real chemistry, accompanied by a
codex of a few hundred curriculum reactions and every concept that explains
them — from nine-year-old to expert, one simulation, rendered at every
register. Offline-first, no Python at runtime, the complete apparatus of
experimental chemistry as first-class operators.

Named for the sealed reflux vessel invented by Maria the Jewess in Alexandria,
1st–3rd century CE — the first named alchemist in recorded history. A sealed dome
with a sample suspended above a heated solvent, so the vapours act on it. She also
gave us the bain-marie; the airtight seal her apparatus needed is where
"hermetically sealed" comes from; and von Soxhlet's 1879 modernisation of it is
still in working labs today — and is itself an apparatus this lab will model.
The name describes the architecture: a sealed vessel you put things into, and
reactions happen. The product's voice is a **bright scientific workshop**:
optimistic, tactile, curious, and exact enough for professional use. Story tone,
operation names, narration, and visual themes live outside the solvers; the
fresh blue/cyan/orange/violet UI palette never recolors computed chemical
appearances or changes scientific behavior.

> Every licence, build flag and API claim in this document was verified against
> upstream source, package metadata or primary legal text on **2026-08-18**.
> Items marked **wasm ✓** were compile-tested locally against
> `wasm32-unknown-unknown` on that date, not read off a README.

## How these files are used

- **PLAN.md** — intent and open work: the thesis, the architecture, the build
  order, and what is still to do. Finished items live in `HISTORY.md`.
- **ROADMAP-GUI.md / ROADMAP-Webapp.md** — numbered open tasks per area (the
  GUI, and the broader chemistry emulator). Task numbers
  (GUI-xxx, BRD-xxx, EXP-xxx, CAP-xxx, KID-xx, ARCH-xxx) are **never
  renumbered and never reused**; a completed item keeps its number and date.
- **HISTORY.md** — what landed and what was learned, newest first.

## Current experiment-expansion checkpoint (2026-09-13)

The source-informed audit programme is complete on `main`: 24 frozen sixth-
fleet cases plus 240 original source-informed scripts passed 404 execution and
relation checks on GitHub CI, keeping the deployment VPS out of the build/run
path. The first distinct-concept promotion is confined to the existing
Experiments, Codex and Missions surfaces: three guided investigations, five
Codex cards and one Mission. Learning progress is authored explicitly; source
URLs and source-to-case research mappings remain private.

The ordered semantic/catalog programme is now on `main`: catalog claims are
machine-checkable, comparisons appear in the existing script-backed Codex run
result, the second 18-concept tranche is in Experiments/Codex/Missions, and the
generic scalar/ratio event boundary is closed. PR #552 carried K64–K70, seven
Codex comparisons and four Missions; PR #555 carried exact-operation numeric
event selectors and two-sample ratios. There is no fourth learner-facing
surface.

The separately versioned source-informed v2 corpus landed through PR #560.
Main's seventh fleet already owns IDs 459–488, so this corpus is frozen at 288
cases, IDs 489–776, in twelve families of 24 with 80 named relations and 368
execution-plus-relation checks, all passing on GitHub CI. Public manifests
contain original questions and scripts but no source identities or
source-to-case mapping. The first failed evidence was retained and only
input-generation/workflow-envelope defects were repaired; no scientific
expectation or engine behavior changed. Twelve GitHub Actions shards ran at
most four in parallel from one revision-bound CLI artifact, with no build or
fleet workload on the deployment VPS.

The first review of all 80 relations landed through PR #562 and selected four
distinct concepts. Intensive
saline conductivity and neutral-sugar conductivity controls become K71–K72
Experiments with paired Codex assertions. Two-axis carbonate limiting-reagent
evidence and the registered peroxide network's time evolution become Missions
with paired Codex cards. The gas card relies on the source-fleet scalar-order
gate because the Codex adapter does not expose gas-event scalars; the peroxide
card carries native semantic assertions. The latter preserves the fleet's
elemental-conservation claim while checking that H2O2 decreases during modeled time; it
is a simulator contract, not a new empirical rate law. Crystallisation
groups are rejected because their frozen relations prove evaporation rather
than crystal formation; thermal, pressure, cell, transfer, filtration and
evaporation-path overlaps stay regression-only. The complete 288-case corpus
remains evidence even when a concept is not promoted.

The third source-informed tranche is complete at cases 777–896: five reviewed
but previously unexecuted private-research candidates become 120 original
public scripts in five 24-case families, with 30 relations and 150
execution-plus-relation checks, all passing on GitHub CI after one preserved
input-contract failure. Review promoted none: thermal, conductivity and ideal
ester relations duplicate existing teaching, while aqueous selectivity and
metal-ligand relations prove conservation rather than their distinctive
optical/chemical claim. Source identities and mappings remain private.

The second tranche is fixed at 18 concepts: seven guided Experiments K64–K70,
seven Codex comparisons, and four Missions. Its pressure, buffer, neutralisation,
precipitation, thermal and process comparisons reuse exact scripts already
passed by the 219–458 fleet. Voltage-event ordering and 2:1 equilibrium scaling
landed through PR #552 and now drive the next generic gate: exact-operation
numeric event selectors plus two-sample ratios with absolute and relative
tolerances. The CLI exposes only an explicit event-field allowlist, and the GUI
must continue to mark event and species values unavailable until its scene
contract carries them; neither may be represented by a convenient but false
state proxy. PR #555 has landed; the immutable successor corpus is the active
CI-backed work described above and in
`tools/chemistry-audit/SOURCE-FLEETS-2.md`.

## Electrochemical kinetics — next ordered work

#559 and #564–#571 provide the shared computed engine, whole-curve validation,
typed potential frames, multispecies interfacial transport, persistent
transient diffusion layers, validated local equilibria at the interface and a
Nernst–Planck boundary solver. The latter two are infrastructure, not yet a
claim that homogeneous reaction and migration are coupled throughout a
time-evolving diffusion layer. Continue in this order:

1. The steady half of this is done. A charged interfacial Faradaic flux now
   reaches `electrodiffusion::reactive_electroneutral_surface` from
   `propose_electrochemical_step`, inside the existing interfacial fixed
   point, so homogeneous speciation, migration and electroneutrality are
   solved together across a steady diffusion layer instead of being refused
   by name. The charged path also inverts the compatibility rule: the
   diffusion-only adapter needs one common `D / L` because it corrects a
   per-species Fick answer afterwards, while the boundary needs one common
   layer thickness `L` and lets diffusivities differ, because the migration
   potential carries the countercharge. What remains is the *time-evolving*
   layer: the lumped transient model has no migration term, so a charged flux
   on it still refuses. Supplying one means a transient Nernst-Planck layer,
   not a transient correction applied to a steady solve, and no effect may be
   represented by tuning an effective diffusivity.
2. Add sweep-direction hysteresis, breakdown/repassivation state and evolving
   films only with forward/reverse or time-series evidence that identifies
   their parameters. A forward polarization curve cannot identify hysteresis.
3. Add dynamic bubble coverage only after a permissive primary source supplies
   an identified evolution law. Explicit measured coverage may already scale
   active area; current alone does not determine coverage.
4. Expose partial currents, potential frame, per-species surface activities,
   transport/film limitation, parameter bounds and fit diagnostics through the
   GUI authority surface.
5. Add quantitative learner experiments only after their parameter domain and
   matched controls pass the whole-curve and conservation gates.

The 2026-09-13 integration programme follows that order and keeps every
learner-facing addition on the existing Experiments, Codex or Missions
surfaces: first audit and integrate reactive electrodiffusion; then expose the
authority fields in the GUI; verify the solvent-only aqueous characterization
that landed in #543 and close its stale follow-up;
render the already-exported Codex models and their `fails_at` boundaries;
resume immutable CI-backed source fleets and promote only distinct supported
concepts; finally produce an evidence-based worktree cleanup manifest. Heavy
fleet and workspace validation runs on GitHub Actions, not the deployment VPS.

The integration pass and its bounded diagnostic follow-up are complete, and
the solver that pass landed now has a caller rather than only a test envelope. The
last committed runtime electrochemical report is persisted on the electrode
and projected through the existing vessel authority readout: terminal and
interface potential, partial currents and record IDs, surface/bulk activities,
transport depletion, parameter frame, bounds and uncertainty, inventory limit
and solver boundary. Whole-curve fit diagnostics and resolved film resistance
are not members of that runtime report, so the GUI says they are unavailable
instead of reconstructing them. Supplying either requires a future typed
engine-report extension backed by the corresponding solve, not a client-side
estimate.

Open circuit remains zero net external current, not zero partial currents.
Never infer isolated branches from net current, digitize plots for runtime
coefficients, tune discrepancies away, or borrow parameters across materials,
surface preparations, electrolytes or hydrodynamic domains. Missing evidence,
activities, gas boundary state and overlapping parameter domains must refuse.
- **`tests/coverage/curiosity-v1/README.md`** — the corpus refresh log. It is a
  test artefact, not a planning file; leave it alone.
- **`docs/CURIOSITY-COVERAGE.md`** — the standing analysis of that corpus: the
  current numbers, which side of each unmet requirement is wrong, and the
  ranked work. Read its opening section before planning off the count: since
  2026-09-08 `expected` is a floor rather than an equality, `computed` and
  `curated` are one grade when a requirement is checked, and the count is a
  backlog rather than a regression gate — baseline drift is the gate.
- **`docs/NEXT-MEASURE.md`** — a proposal, awaiting an owner decision: what
  should measure the engine now that the corpus reads 499 of 500 and no longer
  discriminates. Costs three candidates (a numeric accuracy corpus, a harder
  prompt set, a reachability measure), recommends the first as verification
  level 5 of the ladder this file's companion already specifies, and lists
  twelve defects found while investigating.
- **`validation/` — the accuracy corpus, STARTED 2026-09-15**, on the owner's
  decision of 2026-09-14 to begin with one family, six rows, real citations
  and no gate. `validation/cases/colligative.toml` is that family: four cases,
  six anchored rows, one row deliberately left open, and **two quantities
  covered**, which is the number to publish. Rows passing is not, because it
  can be inflated by adding easy rows to a quantity already covered — so
  `quantities_covered`, `anchored_rows` and `open_rows` are declared fields
  and `kerotakis-core/tests/accuracy_corpus.rs` recomputes all three, which
  makes the anti-gaming rule mechanical rather than procedural.
  Tolerance is argued per quantity, never a flat percentage, generalising the
  only argued tolerance this repository had: a tight band on the model's own
  figure and a loose one on the world's, the outer band wide enough to survive
  an improvement and narrow enough to exclude the known-wrong predecessor,
  with every row naming both so the width can be checked rather than believed.
  Two fields are new and both earned their place on the first family.
  `transcription` says whether anyone read the number off the original — the
  answer is no on all six sources, the bibliographic records being exact while
  the values are this repository's own prose finally given its sources.
  `independent_of_path` says whether the reference shares ancestry with the
  code under test, and **three of the six rows are not independent**: they are
  osmotic coefficients compared against the class of measurement `pitzer.dat`'s
  virial coefficients are FITTED to, so they check that the fit was loaded and
  read rather than that the physics is right. The sucrose row is the most
  independent and the weakest as evidence, because its model and its reference
  agree for reasons that cancel. None of that is visible in a count.
  **The corpus is a backlog, not a gate.** Promotion waits until the
  tolerances have survived argument by someone other than their author.
  The affordability finding the step existed to produce: identifying the
  primary measurement is cheap — minutes per row against Crossref, and four of
  six sources are the original papers rather than the compilation — while
  VERIFYING the transcribed number is expensive and was not bought, because
  the primary papers are paywalled. So citations are affordable; verified
  transcriptions are a separate purchase, and the corpus records which it has.

## Localisation is modular by design

A new language (French, Chinese, Japanese) is added in one go by dropping one
`crates/kerotakis-core/i18n/<lang>.toml`, one
`web/app/src/locales/<lang>.json` (copied from `_template.json`), one
`codex/i18n/<lang>.toml` and one
`tests/coverage/curiosity-v1/i18n/<lang>.toml`, with no code change. Anything
that would require code for a new language is a defect. That includes AUTHORED
CONTENT, not only chrome: the capability explorer read as translated
while listing five hundred English questions, because its buttons were in the
bundle and its subject matter was not. `I18N.md` carries the full file list and
the rule behind it.

---

## The thesis

There is no engine that takes arbitrary reagents and computes what happens. But
the problem decomposes into six sub-problems and **five of them are solved** by
mature tools with real thermodynamic databases. The product's job is to be
truthful about which is which.

| Sub-problem | Status | Engine |
|---|---|---|
| pH, precipitation, dissolution, titration, redox, buffers | solved exactly | PHREEQC |
| Heating & igniting solids and gases — decomposition, combustion, flame T | solved exactly | Gibbs minimisation over NASA CEA data |
| What boils when, what mixes, azeotropes, distillation | solved predictively | feos + UNIFAC + cubic EOS |
| Is this mixture dangerous | solved by database | reactive-group matrix (reimplemented from NOAA's published methodology) |
| How fast, concentrations over time | solved | diffsol (+ Cantera-format mechanisms) |
| Does an arbitrary organic reaction happen | **unsolved** | curate it |

PHREEQC is the highest-value engine here and almost no educational app uses it.
It solves speciation, mineral saturation, gas partitioning, redox and ionic
strength *simultaneously* from thermodynamic data. Mix silver nitrate and table
salt and it returns AgCl precipitate, how many moles, what ions remain, and the
saturation index — derived, not hardcoded.

The second row matters: heating and burning things is half of school chemistry
(CaCO₃ → CaO + CO₂, decomposing KMnO₄, dehydrating copper sulfate, magnesium in
a flame) and neither PHREEQC (aqueous) nor VLE (liquids) touches it. NASA
open-sourced CEA under **Apache-2.0 in 2026, including its thermodynamic
database** (`data/thermo.inp`) — so a small Gibbs-energy minimiser over NASA
polynomials with pure condensed phases turns "heat it" and "ignite it" into
computed chemistry, adiabatic flame temperature included.

### The product form: one world, two ways to play

Kerotakis is an open-world laboratory exploration game built on the bench, not
a text adventure and not a menu of canned simulations. The player inhabits a
persistent lab, handles visible equipment, accepts several missions at once,
finds samples and clues, and earns access to broader apparatus and places by
demonstrating scientific capability. The story supplies motives and
consequences; the solver determines what actually happens.

There are two first-class modes over the same engine and content:

| Mode | Promise | Availability | Save behavior |
|---|---|---|---|
| **Story** | Directed but non-linear growth from a small community lab to independent research | Equipment, supplies, contacts, and places unlock through missions and discoveries | Persistent, versioned story world and progression |
| **Sandbox** | Immediate unrestricted laboratory for play, teaching, authoring, and expert work | Every installed reagent, apparatus, operation, instrument, and study surface | Separate persistent world; cannot grant or consume Story progress |

Story is the recommended first door, never a compulsory tutorial. Sandbox is
always one top-level action away and is never hidden behind account status,
completion, or payment.

The core loop is:

```text
explore a place or problem
  → choose or discover a mission
  → collect a sample and assemble a kit
  → experiment freely on the persistent bench
  → observe, measure, and record evidence
  → submit a computed result or engineered outcome
  → unlock capability, knowledge, relationship, or place
  → use it on old and new problems
```

The current playable campaign shell implements the first part of that loop as
a non-linear research campus. Discovery Hall starts with the shipped beginner
and safety missions, all of them openable in any order — the case file names
four of them as leads and the board lists the rest of the district beneath it,
rather than leaving them counted but undrawn; the first completion opens two
simultaneous routes through Matter Gardens and the Energy Yard. An active
mission can be abandoned at any time, which records nothing and returns to the
map. Later districts preview their requirements, completion is stored by stable
lesson id in the Story save, and Sandbox remains an independent full-access
world. Contacts, material rewards, and transactional
engine-evaluated mission outcomes remain roadmap work rather than decorative
claims in the interface.

While a mission is active, its field journal keeps the plain-language objective,
the exact replayable operator, optional procedural help, and engine-rendered
evidence together above the persistent bench. Completion produces a dismissible,
non-blocking debrief and records the discovery without stopping further bench
work. This is the UI bridge from the existing guided `.lab` corpus toward the
future typed, multi-objective mission evaluator; the journal does not pretend
that scripted step completion is already a fully open-ended outcome contract.

The shared material and instrument catalog is progression-aware without becoming
a game shop. Story begins with useful common stock and core handling tools,
previews exact research requirements for later equipment, and loans every
material required by an accepted mission so progression cannot deadlock an
investigation. Permanent instruments arrive through visible capability
milestones and can be placed immediately from the debrief. Sandbox ignores all
of these access decorations and exposes the complete installed registry.
Story's permanent bottles also carry finite dispense counts. A dispense becomes
inventory only after the engine accepts the material-drawing command; replay,
reload, and bench undo therefore cannot duplicate stock or consume it twice.
Missions use their own supplied kits, and a first discovery replenishes the
permanent stockroom. The notebook continues to show the real requested amount
for every dispense—the inventory layer does not fabricate conversions among
mass, volume, and amount of substance.

The opening Story chapter is a case rather than a lesson list. A cloudy field
sample branches into three concurrent evidence leads plus an optional safety
audit; all four are the shipped engine-backed investigations and keep their
stable progress ids. A persisted contact briefing introduces the problem once,
then the campus board becomes the durable overview for active and secured
evidence. This case presentation does not claim that the current procedural
`.lab` checker is already the future open-ended, multi-solution outcome judge.
The mineral-contamination lead is the first exception: it declares a typed
outcome contract and completes only when the engine emits an observable amount
of precipitated silver chloride. Sodium chloride and potassium chloride are
both supplied valid routes; adding silver chloride directly cannot satisfy the
contract because no precipitation event occurred. The remaining leads retain
their procedural player until equivalent event/state contracts are authored.

Product invariants:

1. A mission states a problem and constraints, not the one approved procedure.
2. Completion is evaluated from typed events and solved world state, not a
   clicked answer key; materially different valid solutions are welcomed.
3. Failure is recoverable and explanatory. Hazards, broken glass, spoiled
   samples, and exhausted Story supplies create consequences without corrupting
   saves or teaching arbitrary physics.
4. Progression gates tools and contexts, never the register dial, equations,
   underlying truth, accessibility features, or language.
5. At useful campaign moments there are multiple active leads; the player is
   never trapped in one corridor or forced to wait/grind.
6. The complete chemistry and apparatus registry remains available in Sandbox.
7. The first UX milestone is a tactile, readable laboratory; world breadth and
   campaign volume follow only after placing, pouring, measuring, and inspecting
   feel good with mouse, touch, and keyboard.
8. Motion is a simulation readout. Pour width and speed follow transferred
   fraction; stirring follows mixed fractions; heat shimmer and frost follow
   temperature and ΔT; flame colour follows the computed emission result; and
   glass explodes only after the engine emits its pressure-rated `Burst` event.
   Reduced-motion users receive the computed endpoint without compulsory
   movement.

The delivery sequence and acceptance criteria live in
[`ROADMAP-GUI.md`](ROADMAP-GUI.md); mission semantics in
[`EXPERIMENTS.md`](EXPERIMENTS.md); save/orchestration contracts in
[`ROADMAP-Webapp.md`](ROADMAP-Webapp.md); and cabinet metadata in
[`APPARATUS.md`](APPARATUS.md).

### The build-time principle

**"No Python" is a runtime constraint. The build machine runs anything.**
Every heavyweight scientific tool becomes a build-time oracle, exporter or
verifier in `tools/`, and only *data* ships:

| Build-time tool | Role |
|---|---|
| xtb / CREST (LGPL) | Reaction ΔG, Fukui indices, MO data, approximate reaction paths, vibrational frequencies → IR spectra |
| PySCF (Apache-2.0) | Proper DFT wavefunctions for showcase FMO reactions; cube generation |
| Multiwfn (custom, verified commercial-OK) | Wavefunction analysis where PySCF needs help; cite both required papers, pin the version |
| `thermo` (MIT, Python) | Golden-test fixture generation for `kerotakis-thermo` — thousands of reference flash/VLE results |
| Cantera (desktop Python) | Reference solutions for combustion golden tests |
| RDKit (Python) | Cross-validation of Indigo template applications and canonicalisations during curation — two independent toolkits agreeing is real QA |
| **Reaktoro** (LGPL-2.1, verified 2026-08-19 via repo metadata) | **Differential oracle for the supported parts of L2** — the modern PHREEQC-class geochemical solver (Leal, ETH), which loads our exact PHREEQC databases natively: same pitzer.dat, independent solver. Diff a corpus against `PhreeqcEquilibrator`; every disagreement is our bug, its bug, or genuinely interesting chemistry. **Boundary found in AQ-006:** Reaktoro 2.13 does not implement PHREEQC surface complexation (the upstream request remains open), so it cannot validate HFO adsorption. Build-time only, never linked; persist only approved facts or aggregate metrics, never an unreviewed fixture export. |
| ChemPy (BSD-2-Clause, verified 2026-08-19) | Cheap second opinion for textbook-level aqueous fixtures where a full geochemical solver is overkill |
| RMG (MIT — verbatim MIT text under a custom header; GitHub shows NOASSERTION) | Benson group-additivity ΔHf/S/Cp(T) for arbitrary organics — the property gap PubChem/Wikidata cannot fill for the energy balance. **Blocker verified 2026-08-19: RMG-database has no LICENSE file.** Note `JacksonBurns/rmgdb` (active RMG developer, Aug 2026): vendors the database + SQLite/YAML repackaging under MIT — but with a *personal* copyright line, and one contributor cannot relicense a ~25-year collective work; no team licensing decision is on record (searched). It does make the upstream ask concrete: "affirm the licence upstream or correct rmgdb's copyright line to the RMG Team". Until then `thermo`'s Joback estimators (MIT) carry this role and no RMG parameter ships |
| ORDerly / ORD | Validation oracle only, never ingestion: check curated conditions against literature without touching ORD's CC-BY-SA (the same oracle pattern as `thermo` and Cantera). Patent-chemistry distribution → low relevance to a school codex; third-tier |
| UniChem (EMBL-EBI) | L1 identity crosswalk keyed on Standard InChI across 25 sources. EBI adds no restrictions beyond the original owners' — but no per-source licence field in the API, and a full dump includes DrugBank/CCDC accession lists (encumbered). **Use restricted to a cleared source whitelist** (PubChem, CompTox, Rhea) |
| DeepChem (MIT, active) | Build-time featurisation/toolbox if ever needed; its bundled datasets carry their own upstream terms |
| **Reaction-QM** (code BSD-3; **data CC BY 4.0**, Zenodo DOI-pinned, *Sci Data* 2026) | **Precomputed reaction energetics** — 2.3M GFN2-xTB + 200k B3LYP-D3/TZVP reactions with IRC-validated transition states and **ΔE‡/ΔH‡/ΔG‡ already in plain CSV** (57–71 MB; the 35 GB is geometry files we can ignore). Clean provenance (PubChem-enumerated, no copyleft corpus). Cuts a large slice of the L4′ xtb pipeline: where a curated reaction appears here, its barrier is free and B3LYP-quality. Caveat: machine-enumerated, often exotic reactions, ≤10 heavy atoms, 9 elements — an *enrichment and validation* source, never a codex source; prefer the B3LYP subset for anything shown to learners |
| cclib (BSD-3, actively maintained 2026-08) | Parses QC logfiles (Gaussian, ORCA, **xtb**, +17 more; PySCF via an in-memory bridge, not a parser). `ccget`/`ccwrite` slot into a Makefile as "QC output → structured JSON". Pin 1.8.1 (2.0 is alpha). May prove unnecessary — Reaction-QM ships parsed data |
| ncsw-data (MIT, solo, untested) | **Download-only** fetcher for ~20 reaction/compound datasets — verified to ship no data and to fetch from original upstream URLs, so it launders nothing. Value is its **URL inventory**; treat as a reference, not a dependency. Warning: it is a URL map, **not a licence map** — it hands over CC-BY-SA data without a word |
| PyTorch | Weight export (safetensors/ONNX) if the ML tier ever ships |

Tool licences (incl. geomeTRIC's "BSD 3-clause Non-AI" clause and Sella's
LGPL) are recorded in `tools/` provenance files; none of it ships.
(Evaluated and parked: `rxnfp` — MIT but dormant since 2021 and USPTO-trained,
the same distribution mismatch as Molecular Transformer; at most an afternoon
of codex-lint clustering someday, not a dependency.)

---

## Architecture

### Don't build a simulator, build a bench

Model containers in states and the operations a person performs on and between
them. `filter` and `decant` produce a *second* container, so the unit of state is
a bench, not a vessel:

```
Bench  = { vessels: [Vessel], log: [Op] }
Vessel = { species: [(InChIKey, moles, phase)], T, P, V, solvent, container }

Mutating ops:   add · heat · cool · stir · wait(t) · filter → v2 · decant → v2
                distil → v2 · evaporate · titrate · electrolyse · ignite
Measuring ops:  pH probe · thermometer · balance · conductivity meter ·
                litmus / indicator paper · spectrophotometer
```

Every user action is one turn of a loop: operator → L0 safety pass → solver
router → new state + explanation. Between operators the bench re-equilibrates.
This is what makes it a *lab* rather than a reaction calculator — the pedagogy
lives in the sequence, and the sequence is free once state is explicit.

**L0 grades, it rarely refuses.** This is a pedagogical tool: for known
hazards the screen emits a **strong, precise warning first** — what forms,
what it would mean outside the simulation — and then the chemistry *proceeds
and shows exactly what would happen* (curated outcome where no solver covers
it). Mixing bleach and ammonia warns, then evolves chloramine you can watch
leave the vessel on the balance. Prohibition teaches nothing; precision plus
consequence does, and the virtual lab is the only place that is safe. The
hard `Veto` verdict exists in the type but is reserved for the
product-safety boundary ("What this will not do" — anything shading into
synthesis-oracle territory), not for curriculum hazards.

**Apparatus are operators.** Burette, distillation head, separating funnel,
pneumatic trough, Soxhlet — each is an operator (or operator + constraint set)
in `kerotakis-core`, drivable textually from day one; graphics come later.
"Never dumb down the model, only the view" applies to equipment too.

**Measuring ops are first-class and read-only.** They cost almost nothing (they
read existing solver output), they give register-appropriate precision naturally
— litmus for the nine-year-old, the same pH to three decimals for the expert —
and "measure before and after" *is* the scientific method being taught.

**The operator log is the save file.** Bench state is tiny; persisting the full
operator sequence gives undo, replay, sharing experiments as scripts, and golden
tests for free. It is also the substrate for lessons and the API contract.

**The vessel has an energy balance.** PHREEQC is isothermal — you *tell* it T.
So the loop does its own enthalpy bookkeeping: reaction ΔH (curated data or
formation enthalpies) + mixture heat capacity → new T → re-equilibrate at the
new T, iterated to convergence. Each vessel is adiabatic, isothermal or
thermostatted by mode. Without this, exothermic mixing cannot warm the beaker,
and "it got hot!" is one of the most important observables at every register.

**Solver failure is a first-class result.** PHREEQC convergence failures on odd
inputs are routine, not exceptional. The router surfaces an honest "couldn't
compute this" state rather than a wrong answer or a crash — the same honesty
rule that governs L4.

**The cache ships pre-warmed.** Content-addressed: canonical species set + T +
P (floats quantised) → result. Because lessons enumerate their reachable
states, every vessel state in the curated lessons is computed at build time and
shipped — most user actions in guided content never touch a solver even on
first run, on any device. Also the graceful-degradation story for the rare
convergence failure.

### CLI first

The engine is text-native by construction — the bench loop *is* a REPL — so the
first client is a thorough CLI, and it is not a detour:

- **It proves the thesis before any UI exists.** `--register 9|15|expert` on
  the same session output tests "one simulation, many views" in the cheapest
  possible medium. If the registers don't work in text, no UI will save them.
- **It is the test harness.** Golden tests, conservation-invariant property
  tests, fuzzing, lesson replays — all are `kero run experiment.lab --json`
  under CI, diffing outputs.
- **It is the curation tool.** `kero codex lint` validates every codex entry:
  balanced equations, provenance records, register copy at all levels, and —
  the killer check — that claimed observations match what the solvers actually
  compute.
- **It is shippable.** PHREEQC's power with a humane interface is a real
  product for the expert register, publishable as `kerotakis` on crates.io
  (claiming the name with substance). A `ratatui` TUI skin later is optional
  flavour — brass-and-verdigris terminal chemistry — but REPL + batch + JSON
  comes first.

Two guardrails: **(1)** `kerotakis-cli` consumes `kerotakis-core` through the
same public API `kerotakis-wasm` will — both are thin peers over one boundary,
and `--json` output is the API contract, snapshot-tested. The moment the CLI
reaches into internals, the boundary mobile and web depend on stops being
hardened. **(2)** CLI comfort must not defer the P0 portability spike — a CLI
on macOS proves the chemistry, not the premise that can kill the project.

### The solver stack

Numbering is real: **L0 runs first and can veto**, and each layer depends on what
the ones below resolve.

| Layer | Role | Engine | Licence |
|---|---|---|---|
| **L0** | Safety & reactivity screen — runs first, grades the state | Our reimplementation of the NOAA reactive-group matrix (see below) | ours, from public-domain methodology |
| **L1** | Species & property registry, canonical InChIKey identity | SQLite/static data + Indigo (bundles the InChI plugin) | Apache-2.0 / MIT |
| **L2** | Aqueous equilibrium — **the workhorse** | IPhreeqc + phreeqc.dat, wateq4f.dat, minteq.v4.dat, **pitzer.dat** | USGS, public domain |
| **L2g** | Gas + condensed-phase equilibrium — heat, ignite, decompose, burn | Gibbs minimiser over NASA CEA data (adopt/extend `cea-rs`, or write it) | Apache-2.0 data |
| **L3** | Phase behaviour — boiling, miscibility, azeotropes | `feos` (SAFT family, flash) + own UNIFAC + `vle-thermo` (cubics, NRTL/Wilson) + `seuif97` (water) | MIT / Apache-2.0 |
| **L3e** | Electrochemistry | Standard-potential ordering + Nernst over PHREEQC's activities, own module (`kerotakis-core/src/displacement.rs`, **built** for displacement, the activity series and Faraday's law with a stated current efficiency) | ours |
| **L4** | Reaction — propose → filter → rank → verify | curated + Indigo templates | Apache-2.0 |
| **L4′** | QM enrichment — **build time only, never in the app** | xtb / CREST / PySCF | LGPL / Apache-2.0, never shipped |
| **L5** | Kinetics & time evolution | diffsol + our rate evaluator over Cantera-format mechanisms | MIT / BSD-3 data |
| **L6** | Appearance — colour, cloudiness, flames, orbitals | curated colour data + Beer–Lambert over ε(λ) + CIE via `palette`; precomputed orbital meshes | ours / MIT |

Notes per layer:

- **L0** — no hardcoded compound pairs in the end state. The screen is a
  cascade (mirroring L4's), each layer labeled with what produced it:
  1. **Curated codex outcome** — shows exactly what forms (the current seed
     entries live here).
  2. **Computed outcome** where a solver genuinely covers it. The
     hypochlorite example this list used to give was SPIKED, answered
     negatively, recorded as PERMANENT — and the negative answer was
     wrong. It said that no PHREEQC database defines a hypochlorite species
     at all, on the strength of a search "across every `.dat` vendored with
     iphreeqc on 2026-09-04, including llnl.dat, which is the one this note
     proposed", and reported that the `ClO-` matches "are all perchlorate".
     `vendor/iphreeqc/database/llnl.dat` line 107 is
     `Cl(1)     ClO-      0         Cl`; perchlorate is `Cl(7)`, three
     lines below it; line 4493 is `H+ + ClO- = HClO`, `log_k 7.5692`. Two
     minutes of `grep -n` in this repository falsify both claims, and the
     note stood for three days because a negative answer marked "permanent"
     is one nobody re-checks. `databases::minteq_v4()` now borrows that
     protonation constant and a beaker of diluted bleach computes pH 9.8. What survives, and is worth separating from the wreckage of
     its example: the ACID–BASE half of hypochlorite is computed and the
     OXIDATION half is still curated, because a log K for a protonation
     says nothing about how strongly the anion oxidises — so the cascade's
     shape is right even where this note's confidence was not. The
     methodological lesson is the sharper one: a boundary claim is a claim,
     it reaches learners as fact, and unlike a computed number nobody ever
     re-derives it. State where you looked, and cite the line. An L2g Gibbs minimisation finding a strongly
     exothermic accessible state (high adiabatic flame T) *is* an
     energetic-mixture detector; build-time RMG/xtb ΔH_rxn plus classical
     indicators (oxygen balance, energetic functional groups) give
     CHETAH-style screening — the ASTM tool is proprietary, the methods are
     published.
  3. **Reactive-group matrix** — two computable stages: structure → groups
     is a SMARTS match (Indigo; the current key→group table is a placeholder
     for this), and group × group → **predicted consequence category**
     (toxic gas, heat, flammable gas, polymerisation — the published chart's
     cells carry these). ~70 groups squared covers millions of compound
     pairs; nothing per-pair is stored.
  4. **Per-substance GHS data** — H-codes/pictograms/signal words from
     **PubChem bulk** (no NCBI restrictions, per-record attribution) and
     **CLP Annex VI via EUR-Lex** (EU legislation, reusable), keyed by
     InChIKey into L1; concentration-awareness via the GHS's own published
     mixture-classification rules (additivity formulas, cut-offs) — a drop
     of bleach and a beaker of it get different verdicts, from arithmetic.

  **Deliberately not computed: predicted toxicity (QSAR).** Authoritative
  classification beats prediction on reliability and liability — a wrong
  "safe" is the one failure mode this product cannot have. Below all four
  layers the honest output is "outside the screened set", never a guess.

  Legal position, stated precisely: the CAMEO/CRW4 *database* is not
  redistributable (its terms fence off CAS RNs, NFPA ratings, AEGLs,
  ERPGs — none of which the design needs). The methodology's facts are not
  copyrightable under *Feist*, and the SMARTS assignment rules are ours —
  but Gorman et al. 2014 is a Wiley paper with CCPS (AIChE) co-authors, so
  **source the matrix from the NOAA Institutional Repository copy**
  (repository.library.noaa.gov/view/noaa/61941), record the derivation, and
  keep it on the counsel list. Verify the group-set generation before
  encoding (classic 43×43 vs the ~68 groups current CAMEO materials
  reference), and verify NITE/other national GHS sources' terms if used
  beyond PubChem's aggregation.
- **L1** — InChI/InChIKey has exactly one implementation, the IUPAC C library,
  **relicensed MIT with v1.07** (plain C, current 1.07.5). We get it through
  Indigo's bundled InChI plugin via the same FFI; a standalone `kerotakis-inchi`
  binding is the fallback (wasm precedent: cheminfo's `inchi-js` npm package).
- **L2** — `pitzer.dat` is only **37 KB** and public-domain like the rest; it
  unlocks brines and high ionic strength (seawater evaporation is a beautiful
  teaching sequence). Embed it. **Do not embed `sit.dat`**: generated from
  ANDRA's ThermoChimie database — non-USGS provenance; revisit only after a
  terms check. Upstream is the actively maintained `phreeqc-dev` GitHub org
  (CMake, C++14).
- **L2g** — a Gibbs minimiser over NASA polynomials is a well-understood,
  few-hundred-line solver. `cea-rs` (MIT OR Apache-2.0, wasm ✓) appeared on
  crates.io in Aug 2026 — embryonic, but proof the port is tractable and a
  candidate to adopt and extend rather than start from zero.
- **L5** — reaction networks are **stiff**; explicit Runge–Kutta will not
  integrate them. diffsol's default backends are pure Rust (nalgebra/faer);
  the `diffsl` JIT stays off for iOS and wasm. For gas kinetics we parse the
  **Cantera YAML mechanism format** (publicly documented) and evaluate rates
  ourselves: GRI-Mech-class education mechanisms need only **Arrhenius +
  three-body + Troe falloff**, plus reversible forms of the last two and
  negative activation energies (measured in BRD-040). The *format* is free to
  implement; the *mechanism files* are not free to redistribute — Cantera's
  BSD-3 licence covers its code only, and BRD-040 found no open licence for any
  candidate mechanism. See `provenance/brd-040-cantera-audit.md`.
  Precedent: KiThe and Fauconneau/combustion in Rust; Arrhenius.jl and
  ReactionMechanismSimulator.jl in Julia.
- **L6** — most of the age-9 register is *observations*, and they need a
  computation path — but sized to its data sourcing. **Default: curated sRGB
  per species + concentration-driven opacity** — 95% of the pedagogy at 5% of
  the cost, and the data is a colour word per species, not a spectrum. The
  full Beer–Lambert-over-ε(λ) + CIE path is reserved for **indicators only**,
  where the dataset is small, classical and openly published. (Full spectra
  for the interesting colours are the hard case: Cu²⁺/MnO₄⁻/Cr₂O₇²⁻ are d–d
  and charge-transfer bands; sTDA-xTB *can* compute UV-Vis at build time but
  is weakest exactly there, published spectra carry copyright, and NIST is
  off-limits per the data table.) `palette` (wasm-fine) handles the colour
  math either way; we supply the CIE integration for the indicator path.

### L4 is a cascade, not a choice

The stage that produced an answer is **shown to the user**.

1. **Propose** — curated library first. A few hundred hand-verified curriculum
   reactions with conditions, ΔH and observations covers all of school and most
   of undergraduate chemistry, and it is *correct*, which matters more than
   coverage in education. Indigo's `indigoReactionProductEnumerate` /
   `indigoTransform` (both verified present in the current flat C API)
   generalise templates across homologues.
2. **Filter** — our own SMARTS incompatibility rules, plus the L0 pass. RDKit's
   shipped `FilterCatalog` (PAINS, BRENK, NIH, ChEMBL) is a set of
   *medicinal-chemistry alerts*, not reaction-feasibility rules. The rules are ours.
3. **Rank** — surface confidence, never present a prediction as a fact.
4. **Verify** — L4′, offline.

### L4′: the QM pipeline, and what it ships

GFN2-xTB is a *verifier*, not a generator — something else proposes the
mechanism. Barriers need a supervised saddle-point search plus frequency and
IRC confirmation; that pipeline fails often and cannot sit behind a user
tapping "mix". So it runs on the build machine, and ships as data. Batching it
build-time also dissolves the LGPL-3.0 relinking conflict with App Store
distribution — the binary never ships.

What the pipeline emits per curated reaction/compound (capabilities verified
against xtb docs):

- **Energetics** — ΔG_rxn (GFN2-xTB), the honesty backbone of L4.
- **Fukui indices** — `xtb --vfukui`: per-atom condensed f⁺/f⁻/f⁰. Colouring
  atoms by the dual descriptor explains regioselectivity *before a bond forms*
  — tiny data, huge pedagogy.
- **Frontier orbitals** — HOMO/LUMO energies and MOs (`--molden`); proper DFT
  wavefunctions from **PySCF** for the showcase FMO set (Diels–Alder, SN2,
  carbonyl additions) where orbital *shape* is the lesson. Honest-labelling
  caveats: GFN2's minimal valence basis gives qualitatively sensible valence MO
  ordering/symmetry but compressed gaps and no anion/diffuse orbitals — the
  register system says which engine produced what.
- **Reaction-path frames** — `xtb --path` (RMSD-PP) gives approximate paths
  *without a transition state*: the SN2 umbrella inversion and carbonyl
  sp²→sp³ animations. True IRC only where a supervised TS search earned it.
- **IR spectra** — vibrational frequencies → synthetic spectra: licence-clean
  (computed, not scraped from NIST), powering the spectrophotometer instrument
  and "identify the unknown" lessons.

**Some of this is already computed.** Reaction-QM (CC BY 4.0, DOI-pinned) publishes IRC-validated transition states and ΔG‡ for 200k B3LYP-quality reactions in plain CSV — so for any curated reaction with a match there, the barrier is a lookup rather than a supervised saddle-point search. Our own xtb/CREST pipeline then covers what the dataset does not, which is most curriculum chemistry (it is machine-enumerated organic space, ≤10 heavy atoms). Check the dataset first, compute second.

**Orbitals ship as meshes, not cubes.** Build-time marching cubes → quantised
glTF keyframes (KHR_mesh_quantization; skip Draco — its decoder outweighs the
savings at these sizes). Verified ~60–300× smaller than cube files; the direct
ancestor is Jmol's JVXL format (400–1000× published). Two-colour signed lobes
give phase-matching visuals — [4+2]-allowed vs [2+2]-forbidden — as plain mesh
rendering, so **no chemistry viewer is required at runtime**: the app's own 3D
layer plays orbital animations like any other asset. For the web expert
register, where raw-cube interaction earns its place: **3Dmol.js** (BSD-3,
active, ~158 KB gzipped, native cube rendering with two-isovalue phase
colouring). Mol* (MIT, very active, ~10× heavier; its `alpha-orbitals`
extension computes MOs on-grid client-side) stays on the watch list; NGL is
dormant — skip.

**The runtime QM tier: Hückel, honestly labelled.** For molecules the *user*
draws, no precomputation exists. No pure-Rust Hückel implementation exists, and
tblite (LGPL Fortran) has no realistic wasm path (gfortran cannot target
wasm). Options, per the adopt-and-extend policy: port or FFI **YAeHMOP**
(extended Hückel, plain C, BSD-2-Clause, small, Landrum-maintained), or write
simple Hückel ourselves — it is essentially an eigensolver over a connectivity
matrix, which nalgebra/faer provide. Qualitative π-MO phases and energies,
labelled approximate; textbook FMO diagrams *are* Hückel theory, so the
approximation is literally the pedagogy.

### Every answer carries its provenance, and the paths are shown

An answer is not just a number. The bench records, for each result: which
**engine** produced it, which **dataset** it consulted, which **model** that
dataset applies (derived from the file's own declarations — a `PITZER`
block means specific-ion interaction; per-species `-gamma` means the WATEQ
Debye-Hückel extension), **why that path was routed** over the
alternatives, and the **literature the dataset itself cites** — those
citations sit in the data files as comments (minteq.v4 carries ~310) and we
capture them instead of stripping them.

And where more than one dataset can express the question, `kero explain`
asks **all of them** and shows the disagreement rather than asserting one
answer. Saturated brine, live:

```
v1: answered by PHREEQC (IPhreeqc, USGS) using pitzer.dat
  model:   Pitzer specific-ion-interaction model (valid at high ionic strength)
  routing: chosen because the solution is concentrated (~16 mol/kgw)
the same question, asked of every dataset:
  wateq4f.dat    pH 7.059 · I = 6.42 m · Halite 1.597 mol
    WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)
  minteq.v4.dat  pH 6.855 · I = 3.69 m · Halite 4.317 mol
  pitzer.dat     pH 6.469 · I = 6.11 m · Halite 1.908 mol
```

A dataset that *cannot* express the question (pitzer.dat has no silver)
says so and names what it lacks, rather than being skipped or answering
wrongly. The comparison should also say how much the datasets even
share: only 21 of 683 mineral phases exist in all three, so three
answers are partly answers about *different admissible solids*, not
three opinions about one activity model — a sentence the `explain`
rendering now carries, fed by `derived::phase_coverage()`, whose numbers
are pinned by a test so that a vendored-database bump forces the
sentence to be re-read rather than silently reprinted. This is the honesty rule taken to its conclusion: not "here is the
number" but "here is the number, here is what computed it, here is where
that came from, and here is what the alternatives say." It is also the
expert register's deepest layer and, for the codex, the model for how
curated entries cite their sources — DB value, computed value, and which
model produced each.

### The pedagogy is the architecture

School chemistry overfeeds facts and underteaches the models that make
facts predictable. That is not a complaint about teachers; it is what the
chemistry-education literature has documented for forty years, and this
app is unusually well placed to do something about it — because it
*computes* the level that textbooks can only assert.

**The core problem has a name.** Johnstone's triangle: chemistry lives at
three levels — the **macroscopic** (what you see), the **submicroscopic**
(particles, ions, species) and the **symbolic** (equations, formulae) —
and novices fail because instruction moves between all three at once
without saying so. A textbook shows a photograph, an equation and a
particle diagram on one page and leaves the learner to guess how they
relate.

**Our engine closes that gap structurally.** PHREEQC computes the
submicroscopic level for real: 3×10⁻⁷ mol/kgw of AgCl(aq) is not an
illustration, it is the answer. So the same vessel state can be rendered
at all three levels *simultaneously and consistently*, which is precisely
the move the literature says learners cannot make unaided.

**Two axes, not one — and we had this wrong.** An earlier version of this
plan mapped lv1/lv2/lv3 onto macroscopic/symbolic/submicroscopic. That is
an error, and worth recording as one: a speciation table with molalities
and activity coefficients is not the submicroscopic level, it is *deeper
symbolic*. It is still numbers and formulae. Under that mapping the
triangle was never closed — we had macroscopic, symbolic, and more
symbolic.

Representation and detail are **orthogonal**:

|              | **lv1** | **lv2** | **lv3** |
|---|---|---|---|
| **macroscopic** | "It went cloudy." | "A white precipitate, ~1.4 g." | mass, turbidity, computed sRGB from ε(λ) |
| **submicroscopic** | a few dots, two kinds | dots at computed ratios, pairs touching | hydration shells, ion pairs, γ as crowding |
| **symbolic** | `silver + salt → chalky stuff` | `Ag⁺ + Cl⁻ → AgCl↓`, Ksp | activities, log K, saturation indices |

Every cell is a rendering of one solved state. The submicroscopic row was
the missing vertex and is now **built** (`kero particles`,
`kerotakis-core/src/particles.rs`): the engine already computes the census
(`Ag⁺ 9.56e-6 · AgCl(aq) 3.21e-7`), so drawing dots at solved ratios is
honest in a way a textbook diagram is not — the picture is the answer, not
an artist's impression of it.

Two things a particle picture can lie about, both handled explicitly:

- **Scale.** You cannot draw 10²³ of anything, so each glyph stands for an
  amount and the amount is printed. Water is drawn sparsely on purpose,
  because a picture that is 99.9 % solvent teaches nothing about the
  solute — and the renderer says that is what it has done.
- **Omission.** A species too dilute to earn a glyph is *named* rather than
  dropped. A picture that silently leaves out AgCl(aq) teaches that the
  neutral complex is not there, which is the same silent-filter defect
  found three times elsewhere in this engine.

It also distinguishes a census drawn from solved speciation from one drawn
off the raw inventory, and says which it is: without a characterised
solution, ion pairs and complexes are unresolved and the picture is coarser
than it looks.

**Guided model-building, not discovery learning.** Pure discovery fails —
Kirschner, Sweller & Clark (2006) is the decisive review: minimally
guided instruction overloads working memory and novices flounder. This is
a real constraint on a virtual lab, because "here is a bench, try things"
is exactly the failure mode. What works is *structured* model-building:
a paradigm experiment, a model built from it, deployment on new cases,
a case that breaks it, revision — with the guidance heavy early and faded
as competence grows. Modeling Instruction (Hestenes) and POGIL are the
best-evidenced structures. The codex's `requires` graph and its
predict-then-run cycle are that shape; the free-form REPL is the *faded*
end of the scaffold, not the entry point.

**Simulation is the right medium for this specific job.** The PhET
programme's finding is that for *concept* building, good simulations often
beat real labs, because they make the invisible visible and strip the
extraneous load of glassware logistics. The real lab keeps what software
cannot honestly supply: measurement skill and the epistemic messiness of
real data. We should say that plainly rather than pretend to replace the
bench.

**Organise around a few load-bearing models, not many topics.** The
candidate spine: the particle model of matter; the mole as the bridge
between mass and count; energy in bonds and reactions; structure-property
relationships; the electron-shell/orbital family; and the acid-base and
redox model families. Everything else is an application of one of these.
Acid-base is the ideal nature-of-science vehicle because school chemistry
already contains three models of expanding scope (Arrhenius → Brønsted →
Lewis) — taught *as* successive models with explicit domains rather than
as three facts, it is philosophy of science for free. `codex/models.toml`
already carries that chain.

**Models are content, not background.** The codex carries `[[model]]`
entries beside its reactions, and the load-bearing field is `fails_at`.
A model presented without its boundary is presented as truth — which is
false, and is why the next model feels like an arbitrary replacement
rather than an answer to a problem the learner can already feel. The
particle model cannot say *why* sodium and chlorine react; knowing that is
what makes the electron model worth having.

**Nature of science has to be named out loud.** Lederman and
Abd-El-Khalick's finding is uncomfortable and load-bearing for us: simply
*doing* labs teaches essentially nothing about how science works. Students
learn what a model is, why models have domains, and how evidence
adjudicates between them only when it is taught **explicitly and
reflectively**. So the app must say the quiet part: "we are now comparing
two models of acids; each explains some things and fails on others." An
engine that silently routes between three thermodynamic databases teaches
nothing about models. One that *says* it is routing, and shows the three
answers disagreeing, teaches the central idea of the discipline.

**And we can show a model failing, live.** `kero explain` already asks
every dataset the same question and prints the disagreement: three
activity models give 1.60, 4.32 and 1.91 mol of undissolved salt, each
with its validity range stated. That is philosophy of science as a
computed result rather than a paragraph — the boundary of a model made
visible by driving the model past it. No textbook can do this; we got it
by accident, chasing honesty, and it turns out to be the pedagogical core.

**Prediction comes before observation.** Predict-observe-explain is the
best-evidenced sequence in science education, and it only works if the
prediction is *committed* first. Codex entries therefore carry an optional
`predict` block: a question, plausible options drawn from the mistakes
learners actually make, and the misconception each wrong answer reveals.
The engine is the arbiter — and because it computes rather than looks up,
a quantitative prediction can be checked, which makes calculation
load-bearing instead of ritual. Working out moles matters when the number
you derive is the number the beaker will show.

Two commitments follow. First, **distractors carry their own diagnosis**:
each wrong option names the misconception it reveals *and the next move
that puts pressure on exactly that idea* — a diagnosis without a next move
is a label, not teaching. One blanket note per question cannot do this,
because a learner who picks option 2 rarely holds the same idea as one who
picks option 3, and the conceptual-change evidence is that instruction
works by eliciting the learner's own model and confronting it. The schema
carries it (`[[reaction.expect.predict.diagnosis]]`), the lint checks the
indices, and `codex lint` reports how many distractors are diagnosed — a
count this document deliberately does not carry, having watched its own
copy go stale (rates.toml is complete and is the pattern to match;
aqueous and quantitative are the work list). Misconception
*prevalence findings* are research facts rather than copyrightable
expression, so we cite them and write our own options — AAAS Project 2061's
item bank, Taber's *Chemical Misconceptions* (RSC), Barke's *Misconceptions
in Chemistry* for the German line, Driver's *Making Sense of Secondary
Science* as the compendium, and Treagust's two-tier format. Where no
citation exists the entry says `Editorial judgement (Kerotakis)`. This is
also the *better* path, because a distractor has to match what our engine
actually computes, not what a textbook rounds to.

Second, **retrieval practice, spacing and interleaving are applied to
model-use, not trivia**. "Given these two elements, predict the bond type
and justify" is worth spacing; "what is the atomic number of vanadium" is
not. The unit of review is a prediction, which is what the `predict` block
already is. The scheduler is not ours to invent: **FSRS** is the current
state of the art, and `fsrs-rs` (BSD-3-Clause, verified 2026-08-19; the
implementation Anki ships) is pure Rust and wasm-clean — zero licence or
portability friction. Hand-rolling SM-2 would be re-solving a solved
sub-problem, which is against the thesis.

**The order is the dependency structure, not the school year.** School
years are an artefact of national administration and differ by country;
the order in which the ideas depend on each other is not. `teaching_order`
is a topological sort over `requires`. Curriculum placements stay on each
entry so a learner who needs to find their syllabus topic still can — the
app follows the subject, and meets the school where it is.

**Antecedents we are consciously working in.** Martin Wagenschein's
*exemplarisches Lernen* (1956) — teach few phenomena genetically and
deeply rather than covering everything thinly — is this manifesto seventy
years early, and it is the German tradition the project is written from.
*Chemie im Kontext* (ChiK, IPN Kiel) is the modern programme that anchors
concepts in contexts and has real evaluation behind it. Neither is a
licence question; both are prior art we should be honest about rather than
present the approach as novel.

**Where we deviate from convention, we do it knowingly and say so.** The
deviation is not novelty for its own sake: it is subordinating facts to
models, making boundaries explicit, and asking for a prediction first.
Facts are not fewer — they are *organised*, which is the condition under
which they are retained at all.

### Stoichiometry: ours, and why not ChemicalFun

ChemicalFun (thermohub, LGPL-2.1) provides the stoichiometric layer we
lacked — formula parsing, composition matrices, automatic balancing. We
build it instead of linking it, for two reasons that point the same way.

**Licence.** LGPL-2.1-*only* is incompatible with AGPL-3.0; the *or-later*
form is compatible via LGPLv3. Even where compatible, LGPL's relink
requirement is awkward for statically linked iOS and wasm store binaries,
and our store exception grants nothing about third-party code. Linking is
the problem; *running* is not.

**Cost.** Balancing is the null space of the element-count matrix, with one
extra row for charge. That is a few hundred lines against machinery we
already had — a formula parser in two places and Gaussian elimination in
the Gibbs minimiser — so the dependency buys little and costs a licence
question on every target.

`crates/kerotakis-core/src/stoich.rs` therefore does it: Unicode
subscripts and superscripts, parenthesised groups, hydrate dots, state
labels, both charge notations, and `kero balance` for the exercise. It
balances dichromate against iron(II) — 14 H⁺ and 7 H₂O — which is the
university case, not a toy.

**What the balancer does and does not do.** It solves the linear system:
one row per element, one for charge, right-hand species negated, and the
answer is the null space. That makes it exact where the system determines
an answer, and it *refuses* where the system does not, which is the more
important half:

- **Under-determined skeletons are refused, not guessed.** `C + O₂ → CO +
  CO₂` admits two independent reactions, and so does `MnO₄⁻ + H₂O₂ + H⁺ →
  Mn²⁺ + O₂ + H₂O` — permanganate can take its oxygen from the peroxide or
  from itself. Resolving those needs oxidation-state bookkeeping, which is
  chemistry the linear system does not contain.
- **It balances a skeleton; it cannot complete one.** `MnO₄⁻ + Fe²⁺ → Mn²⁺
  + Fe³⁺` has nowhere to put the oxygen, and we report that rather than
  inventing the H⁺ and H₂O a half-reaction method would add. Given them, it
  is exact: `MnO₄⁻ + 5 Fe²⁺ + 8 H⁺ → Mn²⁺ + 5 Fe³⁺ + 4 H₂O`.
- **Notation limits, all deliberate.** Element symbols are validated
  against the periodic table, so `A + B → C` is not a formula rather than
  an unbalanced one. Structural formulas (`CH₃-CH₂-OH`), SMILES, isotopes,
  free electrons and organic placeholders (`R`, `Et`) are not formulas
  either. Terms separate on a *spaced* plus, because a bare one is the
  charge in `Ag+`.
- **One notation conflict, resolved and documented.** `Ca2+` and `MnO4-`
  are the same shape with different meanings. Digits before a trailing sign
  are read as a subscript, which makes every oxyanion right at the cost of
  `Ca2+`; write `Ca+2`, `Ca²⁺` or `Ca++`. The other convention was tried
  first and silently broke permanganate.

**They remain valuable as build-time oracles**, and that use is clean:
running a program over public data does not make the output a derivative
work, which is the same reasoning behind the Python `thermo` fixtures in
P3p. Two jobs worth doing:

- **Differential testing.** Run ChemicalFun/ThermoFun over a corpus at
  build time and diff against our Gibbs minimiser and PHREEQC, checked in
  as fixtures. The conservation bug found on 2026-08-19 would have been
  caught in a day rather than by accident. For the aqueous layer
  specifically, **Reaktoro** is the stronger oracle (see the build-time
  table): it speciates from the same PHREEQC databases with an
  independent solver, so a diff isolates *our* code from the data.
- **Completeness checking.** ChemicalFun can enumerate every balanced
  reaction among a substance set; diffing that against the codex yields
  "reactions our registry could already teach but does not" — a coverage
  metric derived from chemistry rather than from a topic list.

Conditions: keep such a tool out of the build graph and out of `vendor/`
so nothing links it accidentally, and keep the *tool* licence separate from
the *dataset* licence — ThermoFun's data carries its own terms, some not
commercial-friendly, exactly as we already track for the PHREEQC databases.

ChemReaX is a closed web application. It is legitimate to consult by hand
as a chemist would; it is not a source we can automate against or
redistribute from.

### `equation` and `summary` are different claims

An entry says what happens in one of two ways, and the schema now keeps
them apart.

`equation` is a **claim about chemistry** and is enforced: it must parse and
it must conserve atoms *and* charge. Putting prose there is an error that
names `summary` as the fix.

`summary` is for entries whose point is not a reaction — a yield
calculation, a measurement, a physical change, or a deliberate null result
where nothing happens at all. It is never parsed.

This was forced by evidence rather than taste. With one field doing both
jobs, 27 of 66 entries held prose in a field documented as "balanced
equation", and no checker can tell a deliberate summary from an equation
someone got wrong. Splitting them turned an ambiguous silence into two
explicit statements, and the count is now a fact rather than an apology:
**51 balanced equations, 19 entries that describe something else.**

One temptation worth naming: an entry whose point is that *nothing reacts*
can be given an identity equation that balances trivially. That inflates
the number and teaches nothing, so those keep a summary instead.

### Declining to model something must be loud

Three bugs found on 2026-08-19 were the same bug wearing different clothes,
and the pattern is worth naming so it stops recurring.

- `ignite` on ethanol reported **"nothing ignited"**. Ethanol has no
  condensed form in the NASA data, so the thermal solver never engaged —
  the bench was reporting the absence of a model as an observation about
  the world.
- Copper sulfate plus lye reported **pH 9.88 holding 0.01 mol/L of Cu²⁺**.
  That solution cannot exist: it is grossly supersaturated against two
  solids (`Cu(OH)2`, `Tenorite`) that are *already in the databases we
  ship* but absent from our registry, so the phase could never be admitted.
- Cooling a beaker past 0 °C reported **liquid water at −7.95 °C, with a
  pH**, because nothing re-evaluates state.

In every case a *filter* — the honesty boundary that says only species we
can name may appear — behaved as a *fact*. The boundary itself is right and
should stay: a solver that reaches for an exotic carbide we cannot name
must either drop it (losing mass) or show a formula with no story attached.
The defect is that the filter was **silent**.

**The rule: every place the engine declines to model something, it says
so.** A state we cannot characterise is reported as uncharacterised, never
returned as the state. Concretely this means the honesty pass must read
back **saturation indices** rather than only compositions, so "the solution
you are looking at is supersaturated against a phase this lab cannot name"
becomes a sentence the user sees. That single change closes the general
class rather than the copper instance, and it surfaces SI — which the model
audit independently flagged as computed-but-never-displayed.

### Known gaps, written down where they can be found

- **Solid-to-solid conversion carries no heat.** The energy balance reads
  `Dissolved` and `Precipitated` events; a solid turning into a different
  solid emits neither, so slaking lime (CaO + H₂O → Ca(OH)₂, about
  −82 kJ/mol) shows the vessel *cooling*. A bucket of quicklime steams and
  is a burn hazard, and we get the sign wrong. The fix is an event for a
  consumed solid plus reaction enthalpies on phase conversions, not a
  number bolted to the registry.
- **Added gas now dissolves (resolved in AQ-003).** `add v1 CO2` is an
  amount-limited dose through an external boundary: limewater goes milky,
  continued excess CO₂ clears it again, and the event ledger separates what
  entered the condensed inventory from what vented.
- **Partial freezing is not modelled.** A freezing solution really gives ice
  plus an ever more concentrated brine down to a eutectic; we freeze the
  solvent and say the rest is not modelled.
- **`evaporate` is externally powered, and this is a decision rather than an
  oversight.** Boiling off 5.5 mol of water costs about 223 kJ — vastly more
  than any chemistry in the beaker — and the operator charges nothing for
  it. That is deliberate: `evaporate` means *you put it on a hotplate*, and
  the energy comes from outside the ledger, exactly as the heat for it would
  in a real dish. Modelling the cost without modelling the burner would be
  worse than not modelling either, because a beaker that froze itself while
  evaporating is further from the truth than one that simply does not say.

  The consequence has to be stated plainly, because it is not obvious: **the
  thermometer after an `evaporate` is not a claim.** What it shows is only
  the chemistry that happened in the beaker — crystallisation heat, mostly —
  divided by a heat capacity that shrank as the water left. That is why
  brine boiled to 99% reads 65 °C: 0.0936 mol of salt crystallising into a
  gram of remaining water, with nothing to carry the heat away. The ionic
  strength and the solid amount from that beaker are trustworthy; its
  temperature is arithmetic about a system whose largest energy term has
  been deliberately left out. No codex entry quotes a temperature after
  evaporating, and none should.

  The costed version wants a heat source as a first-class thing — a hotplate
  with a power and a duration — at which point evaporation becomes a *rate*
  rather than a fraction, and the operator changes shape. That is a real
  feature and not a patch, and it is not scheduled.

### Thermodynamic product versus kinetic product

Copper turns out to be a *better* problem than a missing phase, and it earns
a place in the plan rather than a patch.

```text
Cu(OH)2  + 2H⁺ = Cu²⁺ + 2H₂O    log_k 8.674
Tenorite + 2H⁺ = Cu²⁺ +  H₂O    log_k 7.644   (CuO)
```

Tenorite is more stable by ~1.03 log units, so **equilibrium says black
CuO while the beaker shows pale blue Cu(OH)₂ gel**. That is Ostwald's rule
of stages: the metastable phase nucleates first because it is kinetically
accessible, and heating converts blue to black — which is the classic
school demonstration.

So a Gibbs-minimising engine is *structurally* unable to reproduce one of
the most-performed reactions in school chemistry, and the reason why is
itself first-rate content. The design consequence: phases may be marked
**metastable under stated conditions**, as *data with provenance*, not as
special cases in code — the same shape as the thermal solver's 500 K
stand-down, which is the same admission (equilibrium is not the whole
story) made once already. Done properly, "heat the blue precipitate and it
turns black" becomes a **computed prediction**: raise the temperature,
lift the suppression, tenorite wins.

### The codex

A few hundred curriculum reactions, and **every concept that explains them**,
as one dataset:

- **Concept graph** — a petgraph DAG: concepts → reactions they explain,
  prerequisite edges between concepts. The difficulty ladder made explicit and
  queryable; the nigredo → albedo → citrinitas → rubedo tiers are cuts through
  this graph.
- **Reaction entries** — balanced equation, conditions, ΔH, observations
  (colour, gas, precipitate, heat, smell), provenance, register copy at every
  level, and the L4′ enrichment block (ΔG, Fukui colouring, FMO pair + gap +
  phase-match verdict, path frames, IR spectrum).
- **Flavour is data.** Operation names, narration templates, codex voice — the
  bright-workshop adventure voice and any optional story theme live in these
  files and survive UI redesigns untouched. The default tone is warm and
  curious, never babyish; lv3 may be concise without becoming a different app.
- **Markup decided early.** The codex rendering convention (register copy,
  diagrams, concept pages) is chosen during authoring, not after hundreds of
  entries exist — the renderer can come late, the format cannot.
- `kero codex lint` enforces all of it mechanically, including that claimed
  observations match solver output.

Budget curation as a chemistry-editorial role, not an engineering task. This is
the moat: nobody can scrape a well-curated pedagogical reaction set with
observations, orbital stories and register copy attached.

### Curation is verifiable, so drafting can be assisted

The no-language-model rule governs the *runtime*: narration is
deterministic templates, and that stands untouched. Authoring is a
different regime, and this project is the rare place where model-drafted
content is safe to use — because it is **mechanically verifiable before a
human reads it**. `codex lint` replays every numeric claim through the
real solvers, equations must balance atoms and charge, spine anchors must
resolve, and a model entry with an empty `fails_at` is refused. A draft
that survives all of that is wrong in at most the ways a human draft is
wrong.

That matters because the two largest open work lists are editorial and
exactly draft-then-verify shaped: **most distractors carry no diagnosis**
(`codex lint` prints the live count) and **most spine topics are
uncovered** (`kero codex gaps` prints them). Budgeting those as pure
hand-curation prices the moat wrong — the moat is the verification
machinery and the editorial judgement, not the typing.

The pipeline, honestly bounded:

- A model drafts entries, distractor diagnoses, register copy,
  translations. The lint is the first reader; the chemistry editor is the
  second and remains the gate. Nothing merges on a model's say-so.
- Model-assisted entries **say so in their provenance** — the same rule
  as `Editorial judgement (Kerotakis)`. A model's confidence is not a
  citation: misconception distractors still cite Taber, Barke, Driver or
  the AAAS bank, or are marked editorial.
- `kero serve --mcp` exposes the bench as an **MCP server** over the same
  `--json` contract the CLI already snapshot-tests, so a drafting agent
  can *run its own claims* — execute the entry's `.lab` setup, compare
  the computed numbers against its prose, iterate — before a human sees
  the draft. The CLI is text-native by construction; this is a thin
  wrapper, not a new surface.

---

## The crate stack, verified

| Crate | Licence | Status (2026-08) | Role | wasm |
|---|---|---|---|---|
| `feos` | MIT OR Apache-2.0 | active, v0.10.1 | L3 core: PC-SAFT, gc-PC-SAFT, multiparameter Helmholtz, full flash machinery. **No UNIFAC** | ✓ |
| `num-dual` | MIT OR Apache-2.0 | active | AD backbone of feos; exact fugacity/enthalpy derivatives | ✓ |
| `vle-thermo` | MIT | active, very young (May 2026) | 22+ cubic EOS, NRTL/Wilson/van Laar, Rachford–Rice flash, phase envelopes | ✓ |
| `seuif97` | MIT | active | IAPWS-IF97 water/steam — most of our solvent story | ✓ |
| `diffsol` | MIT | active | L5 stiff ODE/DAE (BDF), pure-Rust nalgebra/faer backends; `diffsl` JIT **off** for iOS/wasm | ✓ (JIT off) |
| `cea-rs` | MIT OR Apache-2.0 | embryonic (Aug 2026) | L2g seed — adopt/extend or rewrite over the same Apache-2.0 CEA data | ✓ |
| `nalgebra` | Apache-2.0 | active | Stoichiometric null-space balancing, eigensolver for Hückel, linear algebra | ✓ |
| `petgraph` | MIT OR Apache-2.0 | active | Concept graph, reaction-network DAGs, cascade routing | ✓ |
| `uom` | Apache-2.0 OR MIT | alive, ~1 release/yr | Compile-time units at `kerotakis-core` API boundaries — kills molarity-vs-molality bugs | ✓ |
| `palette` | MIT OR Apache-2.0 | active | L6 colour math (XYZ/Lab/sRGB); we supply the spectral→XYZ integration | ✓ |
| `rusqlite` ≥ 0.38 | MIT | active | Registry/codex if queryable storage wins. wasm via `sqlite-wasm-rs` (also Diesel's official wasm backend) | ✓ |
| `postcard`/`rkyv` | MIT etc. | active | Alternative for read-only bundled data: `include_bytes!`, zero-copy | ✓ |

**Evaluated and set aside** (with the reason, so we don't re-litigate):

- `purr`, `gamma`, `chemcore` — frozen at 2021 proof-of-concept state; no SMARTS
  matching, no canonicalisation, no InChI. Their author, Rich Apodaca, died in
  2024; the successor `balsa` is also dormant. Not a base to build on.
- `sundials-sys` — the upstream crate is dormant ~20 months. Our own
  `sundials-kinetics-rs` (MIT, in-tree at `sundials-kinetics-rs/`) now
  provides complete SUNDIALS 7.x bindings with CVODE/IDA/ARKode/KINSOL,
  iterative solvers, preconditioners, adjoint sensitivity, and feature-gated
  KLU. `kerotakis-sundials` wraps it as a native-only Track B oracle and
  optional desktop backend behind `advance_network_cvode()`. diffsol remains
  the portable L5 default for wasm and iOS.
- `coolprop-sys` — actively maintained but bundles **prebuilt desktop dylibs
  only**: no wasm, no iOS/Android. CoolProp's official Emscripten/JS build
  keeps it available as an optional *desktop/web extra*, not core.
- `KiThe` — hard, non-optional `reqwest`/`tokio` deps (its NIST WebBook
  scraper) fail on wasm ✗, and scraping WebBook is an SRD-licensing problem
  anyway. Fork candidate only if its equilibrium code outperforms our L2g.
- `ort` — iOS/Android static binaries exist, but the browser path is `ort-web`
  bridging onnxruntime-web: two incompatible wasm contexts. If the ML tier
  ever ships: `tract` (pure Rust, wasm-clean) for small models, `candle` /
  `burn` for larger. Deferred with the ML tier itself.
- `rdkit-rs` — dormant, needs native C++ RDKit, no wasm story. Indigo is
  primary at runtime; RDKit serves at build time (oracle table above).
  **Corrected 2026-08-23:** what lost its maintainer in 2026-04 was the
  `rdkit-js` npm/wrapper repo — RDKit's actual wasm core
  (`Code/MinimalLib`, including its C-ABI `cffi` build) lives in the core
  repo and is maintained by the core team. We consume wasm modules
  directly (the IPhreeqc pattern), so the orphaned wrapper is irrelevant
  and there is nothing to adopt. MinimalLib/cffi is hereby the recorded
  **fallback for L4's runtime** — best-in-class `RunReactants`, C ABI
  usable from Rust without `rdkit-rs` — if the self-built Indigo wasm
  with `indigoTransform`/`indigoReactionProductEnumerate` disappoints.
  Decide by compile-test when L4 arrives; heavier wasm than Indigo and a
  curated API subset are the known costs. Not re-litigated before then.
- NGL Viewer — MIT but dormant; its author moved to Mol*. Skip.

**Watch list** (young or needs-work, but filling real gaps — see policy below):

- `chematic` — pure-Rust cheminformatics with real SMARTS + VF2 substructure
  matching, canonical SMILES, 2D depiction, first-class wasm npm build. Three
  months old, bus-factor 1, self-reported RDKit parity. If it matures it
  replaces the Indigo FFI for everything except InChI; re-evaluate quarterly.
- **Cantera via its generated C API** (BSD-3-Clause code only) — BRD-040
  completed the decision on 2026-08-29: the portable Cantera-YAML/diffsol
  slice covers the educational requirement, every audited upstream mechanism
  is oracle-only for lack of a redistribution grant, and full C-API shipping
  closes no current capability gap. Upstream Emscripten work is not an
  all-target proof: the clib remains experimental and iOS/Android packaging is
  unproven. BRD-042 is therefore parked until a named capability cannot
  reasonably be implemented in the portable path; a future Cantera release by
  itself is not a reason to reopen it.
- **Thermochimica** — BSD-3-Clause code and potentially valuable for melts,
  slags and non-ideal condensed phases, but its Fortran + BLAS/LAPACK build and
  independently licensed thermodynamic data do not satisfy the browser,
  Android, iOS, macOS and Windows runtime contract. BRD-093 closes universal
  runtime adoption as no-go; it remains eligible as a native/build-time oracle
  for a named experiment with a cleared database.
- `teqp` (NIST) — **public-domain** multiparameter/GERG/SAFT EOS in C++;
  Emscripten side-module feasible. The option if L3 ever needs
  reference-quality multiparameter mixtures beyond feos.
- `GEMS3K` (PSI) — LGPL-3.0 Gibbs minimiser, markedly better than PHREEQC for
  non-ideal solid solutions and melts. **Excluded from shipping by the
  permissive-only bar (2026-08-23)** — previously "manageable for
  desktop/server", no longer: nothing copyleft links into a shipped
  artifact. Remains available as a build-time differential oracle if L2
  hits that wall.
- `mcubes` — MIT marching cubes written for electron-density meshing; young
  0.1.x. Build-time mesh pipeline first choice; vendoring the ~200-line
  algorithm is the fallback.
- **YAeHMOP** — extended Hückel, plain C, BSD-2-Clause, small,
  Landrum-maintained. Port-to-Rust or FFI candidate for the runtime Hückel tier.
- the alpha `phreeqc` npm package (Emscripten, MIT + USGS notice, Jan 2026) —
  single-maintainer alpha; not a dependency, but the existence proof for our
  P0 build and worth reading first.

**Queued by the 2026-08-19 review** (adopted when their phase arrives;
licences verified via repository metadata that day):

- `fsrs-rs` (BSD-3-Clause) — spaced-retrieval scheduler; **ships**, so its
  licence matters, and it is clean. See the retrieval-practice note in the
  pedagogy section.
- `fluent-rs` (Apache-2.0) — register narration templates; **ships**. See
  the registers section.
- Dev-tools that never ship, so their licences never propagate: `insta`
  (snapshot-test the `--json` contract with a review workflow, replacing
  hand-pinned shape tests), `cargo-mutants` (mutation testing — which
  invariants are load-bearing and which decorative), `cargo-fuzz`,
  `cargo-deny`, `cargo-about`, `release-plz`, `taplo`.

**Queued by the 2026-08-23 review** (mapped to CAP/OPT tasks in
[CAPABILITIES.md](CAPABILITIES.md) and [OPTIMIZATION.md](OPTIMIZATION.md);
licences verified against upstream that day unless marked):

- **Official IUPAC InChI ≥ 1.07.1** (MIT since 2024-08; upstream
  demonstrates an Emscripten/wasm build) — the identity layer's real
  library, vendorable native + wasm on the IPhreeqc pattern. CAP-13.
- `contour` (Apache-2.0) — marching-squares isolines/regions for
  predominance diagrams (CAP-4). Its sibling `contour-isobands` is
  AGPL-3.0 and barred by the shipping bar.
- `argmin` (MIT OR Apache-2.0, active) — pure-Rust 1-D/derivative-free
  optimization for CAP-9; rust-cv's `levenberg-marquardt` (MINPACK port)
  if multi-parameter fitting ever arrives.
- `rand` + `rand_distr` + `rand_chacha`, `statrs` (MIT/Apache-2.0) —
  seeded, named PRNG and distributions for Monte Carlo (CAP-8).
- `rayon` (MIT OR Apache-2.0) — native parallelism for studies, grids
  and the sweep harness (CAP-2); wasm stays serial.
- `num-rational`/`num-bigint` (MIT OR Apache-2.0) — exact null-space
  arithmetic for underdetermined balancing (CAP-7).
- `csv` (Unlicense OR MIT) — learner measurement import (CAP-9).
- `physical_constants` (MIT OR Apache-2.0, CODATA) — fundamental
  constants with provenance still recorded per value (CAP-6).
- `talc` (MIT) — maintained small wasm allocator (`wee_alloc` is dead);
  `mimalloc` (MIT) native CLI allocator; both adopted only on measured
  deltas (OPT-2). `lasso`/`string-interner` (MIT/Apache-2.0) if
  `SpeciesId` interning gets filed (OPT-4 follow-up).
- `poloto` or `plotters` (MIT; licences to re-verify at adoption) —
  only if hand-rolled SVG from the CAP-3 chart contract proves
  insufficient; prototype without them first.
- Vendored KaTeX (MIT) — optional lv2 equation rendering in the PWA;
  polish, not architecture.

### Adopt-and-extend policy

A tool is not disqualified because it needs work from us — forking, wasm
compiles, FFI bindings, feature-gating out bad deps — **if it fills a gap no
equally good, licence-compatible tool fills**. `cea-rs`, `chematic`, `teqp`,
YAeHMOP, the Cantera clib, a KiThe fork and the `unifac` crate's algorithm are
all in that category. A tool *is* disqualified by: a non-permissive licence
on code we'd ship (see the shipping bar), non-redistributable embedded data,
or a dead upstream *plus* an equally good maintained alternative. When we
extend, we upstream patches where the project is alive and fork visibly
where it is not.

**The shipping bar, hardened 2026-08-23.** Code that ships — linked into
any binary, wasm module or app-store build — must be MIT, Apache-2.0,
BSD-2/3-Clause, Zlib, Unlicense/CC0 or public domain (USGS). **No GPL-family
licence ships, LGPL included** — this supersedes every earlier "LGPL is
manageable on desktop" reasoning in this document. Two independent reasons:
static wasm and app-store builds have no honest relink story, and the
NOTICE §7 app-store permission can only be granted by copyright holders, so
a copyleft dependency would poison the store binaries. Build-time tools and
oracles that never link into a shipped artifact (xtb, CREST, Reaktoro,
PySCF, GEMS3K-as-oracle …) are exempt and stay recorded in `tools/`
provenance. Enforcement is a CI lint — `cargo-deny` licence allowlist,
scoped as CAP-14 in [CAPABILITIES.md](CAPABILITIES.md) — not reviewer
memory.

#### Shipping bar, hardened 2026-08-23

Shipped code is **MIT / Apache-2.0 / BSD-2-Clause / BSD-3-Clause / Zlib /
Unlicense / public-domain only**. No GPL family including LGPL.

Two reasons:

1. **No honest relink story in static wasm/store builds.** A statically linked
   LGPL library in an Emscripten `.wasm` or a store `.ipa`/`.apk` cannot be
   relinked by the user. The LGPL requires offering that capability; we cannot
   offer it honestly for these targets, so we do not ship LGPL code in them.
2. **The NOTICE §7 store permission can only be granted by copyright holders.**
   A copyleft dependency's copyright belongs to its upstream authors, who have
   not granted the store-distribution permission in NOTICE. Adding copyleft
   code to the shipped payload would poison the store binaries.

Build-time oracle tools (Reaktoro LGPL, xtb LGPL, etc.) are fine — they never
enter a distributed binary.

**Queued by the 2026-08-23 review** (adopted when their phase arrives;
licences verified via repository metadata that day):

- **IUPAC InChI library** (MIT since v1.07.1) — the official canonical chemical
  identity engine. Upstream-proven wasm build exists. Vendored on the IPhreeqc
  pattern with a CI check that every registry InChIKey recomputes and matches.
- **Cantera** (BSD-3-Clause, verified 2026-08-23) — reaction mechanism format
  and rate-law definitions, implemented independently; the code itself stays
  oracle-only. The mechanism YAML files Cantera ships are **not** covered by
  that licence and are not redistributable (BRD-040, 2026-08-29). No mechanism
  data has been imported.
- `contour` (MIT) — contour line generation for phase diagrams.
- `rayon` (MIT/Apache-2.0) — data parallelism for multi-vessel benchmarks.
- `rand_chacha` / `rand_distr` / `statrs` (MIT/Apache-2.0) — reproducible
  random number generation and statistics for Monte Carlo and stochastic
  kinetics.
- `argmin` (MIT/Apache-2.0) — numerical optimization for parameter fitting.
- `csv` (MIT/Unlicense) — CSV export for selected output.
- `num-rational` (MIT/Apache-2.0) — exact stoichiometric coefficients.
- `physical_constants` (MIT) — CODATA physical constants.
- Chart libraries: hand-roll first (the renderer-neutral `ChartObject` is
  already implemented); adopt a library only when the hand-rolled version
  proves insufficient. Explicitly **ban `plotters`' AGPL sibling**.
- `lasso` / `string-interner` (MIT/Apache-2.0) — eventual SpeciesId interning
  for the DATA-010 registry refactor.

### The UNIFAC question, precisely

No clean-licensed, maintained Rust UNIFAC exists. The `unifac` crate (frozen
2021, wasm ✓) is MIT **code** with a warning clause about its embedded
parameters — and that problem attaches to *any* implementation, including one we
write: the maintained UNIFAC Consortium tables are proprietary; the original
open-literature tables (Fredenslund, Gmehling et al., 1970s–90s journals) are
usable. So: reimplement the ~300-line algorithm (or fork the crate), source
parameters from the original publications, and record provenance per parameter.
Budget it as data curation, not coding. Acceptance test: the ethanol–water
azeotrope at 95.6% — a genuine teaching moment most simulators miss.

---

## Why Rust, and why offline works

### The deciding fact

`dart:ffi` **cannot be imported when compiling to Wasm**, and there is no unified
API for driving one native library through FFI on mobile and JS interop on web.
A Flutter app targeting web must therefore write every native integration twice.
With PHREEQC and Indigo in the stack that doubles the hardest code in the
project. Rust compiles one source to `wasm32` and to all five native targets.

### The two-track wasm/FFI strategy

**Track A — pure Rust → `wasm32-unknown-unknown` + wasm-bindgen.**
`kerotakis-core`, feos, vle-thermo, seuif97, diffsol, nalgebra, petgraph, uom,
palette, sqlite-wasm-rs. All compile-verified or wasm-proven. No experiments
needed; the same source serves all five native targets.

**Done and CI-enforced:** `kerotakis-wasm` runs the bench in a wasm runtime —
thermal chemistry computed live (magnesium ignites, burns to the oxide,
~3000 K, narrated for a nine-year-old), the species shelf exposed with
provenance, and **whole `.lab` lessons replayed** by the same grammar the CLI
uses (moved into the core, so a lesson behaves identically in both and its
pre-warmed results match exactly). Aqueous chemistry comes from the shipped
results, because a browser cannot link IPhreeqc's C++: the marquee lesson
replays with **no engine present**, and a state nobody pre-computed is
reported as a stated miss rather than guessed at. The engine is now a cargo
feature (`--no-default-features` gives the cache-only build), which is what
makes that split clean rather than a fork.

**Track B — the C/C++ engines (IPhreeqc, Indigo+InChI) → Emscripten side
modules with a thin JS bridge on web; ordinary cargo + CMake FFI on native.**
Three existence proofs de-risk the web side: the alpha `phreeqc` npm package,
EPAM's official `indigo-ketcher` wasm, and CoolProp's official Emscripten build.
Because IPhreeqc's API is string-in/value-out, the JS bridge between the wasm
modules is trivial. On native, a shipping third-party Android PHREEQC app
already proves cross-compilation. One caveat: `indigo-ketcher` exports a
*subset* of the Indigo API and may not include `indigoReactionProductEnumerate`
/ `indigoTransform` — plan on building Indigo's wasm target ourselves with those
symbols exported (the Emscripten toolchain is in their repo).

**Stretch experiment — single-module linking via wasi-sdk.** Since wasi-sdk 33,
C++ exceptions are supported opt-in (`-fwasm-exceptions`, dual libc++ builds),
and Rust's `wasm32-wasip1` officially supports linking wasi-sdk-built C/C++
static libraries: one wasm module, no JS bridge, no Emscripten. Nobody has done
it with PHREEQC (which uses exceptions internally), so it is time-boxed and
never on the critical path — Track B is already proven. Either way, exceptions
stay caught on the C++ side of the boundary; IPhreeqc's API already does.

### PHREEQC runs on a phone

IPhreeqc's C API has a complete string-in / value-out path that **never touches
the filesystem**:

```c
LoadDatabaseString(id, db)          // thermodynamic DB from a string
RunString(id, input)                // input from a string
SetSelectedOutputStringOn(id, 1)    // results to memory
GetSelectedOutputValue(id, r, c, &v)
SetOutputFileOn(id, 0)              // all file I/O off
```

It builds with **CMake**, so Android NDK, iOS and Emscripten all get proper
toolchain files. And the databases are small enough to compile into the binary:

| Database | Size | Covers |
|---|---|---|
| `phreeqc.dat` | 52 KB | Core aqueous set — most teaching chemistry |
| `wateq4f.dat` | 88 KB | Extended natural-water species |
| `minteq.v4.dat` | 316 KB | Metals, complexation, sorption |
| `pitzer.dat` | 37 KB | Brines, high ionic strength |
| **all four** | **~490 KB** (~80 KB gzipped) | an `include_str!`, not an asset pipeline |

### Workspace layout

```
kerotakis/
├── crates/
│   ├── kerotakis-core/       bench + vessel state machine, operators, energy
│   │                         balance, solver router, measurement ops, registers
│   ├── kerotakis-cli/        REPL + batch + --json; the harness, the curation
│   │                         tool, and the first shippable client
│   ├── kerotakis-phreeqc/    IPhreeqc FFI + embedded databases (L2)
│   ├── kerotakis-cea/        Gibbs minimiser over NASA CEA data (L2g)
│   ├── kerotakis-indigo/     Indigo FFI — structures, InChI, templates (L1/L4)
│   ├── kerotakis-thermo/     feos + own UNIFAC + vle-thermo + seuif97 (L3)
│   ├── kerotakis-electro/    electrolysis — Faraday + potential ordering (L3e)
│   ├── kerotakis-kinetics/   diffsol + Cantera-YAML mechanism parser (L5)
│   ├── kerotakis-appearance/ colour: curated data + Beer–Lambert + CIE (L6)
│   ├── kerotakis-safety/     reimplemented reactive-group matrix + rules (L0)
│   ├── kerotakis-huckel/     runtime qualitative MO tier (own / YAeHMOP)
│   ├── kerotakis-data/       codex + registry + pre-warmed cache, embedded
│   └── kerotakis-wasm/       wasm-bindgen surface for web (Track A)
├── tools/                    build-time pipelines: xtb/PySCF batches, orbital
│                             meshes, IR spectra, data exports, oracles,
│                             Indigo wasm build
├── lessons/                  declarative scenario files
└── app/                      UI — see the open decision below
```

`kerotakis-core` is the invariant. It compiles to `wasm32-unknown-unknown` and to
aarch64-apple-ios, aarch64-linux-android, x86_64-pc-windows-msvc and
aarch64-apple-darwin from one source.

### Testing is part of the architecture

- **Conservation invariants** — property tests asserting mass and charge balance
  across *every* operator, on random benches. Catches whole classes of bugs no
  example test finds, and it is a moat: lookup-table competitors cannot even
  state the invariant.
- **Golden tests** — textbook values: acetic-acid titration curve, AgCl Ksp,
  ethanol–water azeotrope, CaCO₃ decomposition temperature, adiabatic flame T —
  plus oracle-generated fixtures from `thermo` and Cantera (build-time Python).
- **Fuzzing PHREEQC** — random vessel states in, no crash and honest failure out.
  Still open from P0, and the plan is now concrete: `cargo-fuzz` with
  `arbitrary`-derived structured inputs over the `.lab` grammar, the
  Unicode formula parser in `stoich.rs` (subscripts, hydrate dots, two
  charge notations — a classic fuzz target), `dbindex`, and the
  vessel → PHREEQC round trip. Nightly CI job; apply to **OSS-Fuzz**
  (free continuous fuzzing, Rust supported — acceptance is not guaranteed
  for a young project, but the application costs an afternoon).
- **Metamorphic invariants** — properties conservation cannot see, each a
  router/cache bug detector: order-independence (`add A; add B` ≡
  `add B; add A` at equilibrium), dilution monotonicity (adding water
  moves pH toward 7), scale invariance (double everything → intensive
  properties unchanged), and — the thesis as a test — all three registers
  and all three representations render from **one** solved state with no
  re-solving. The kinetics work found four bugs the conservation proptest
  could not see; metamorphic relations hunt that class systematically
  instead of by accident. **First three built 2026-08-19**
  (`kerotakis-cli/tests/metamorphic.rs`, driven through the real binary and
  the `--json` contract), and the order-independence test earned its place
  on its first run: the same reagents added in a different order settled
  ~8e-5 apart in pH — and pulling that thread took five rounds of
  measurement across two sessions before the root cause fell out, every
  intermediate mechanism plausible and wrong (quantisation cancellation,
  falsified by a non-dyadic ×1.7 scaling holding at 2.5e-11; uniform
  solute rescaling, falsified by K+ carrying none of the excess; print
  precision, worth ~1e-7 against a 1.4e-2 symptom, which struck
  `GetSelectedOutputValue` from the work list; carrying moles forward,
  worth 2.8e-10). The real chain: dissolution enthalpy rode on the
  Dissolved event, no event was recorded for a phase the routed database
  cannot name — and that is not a corner: **662 of 683 mineral phases
  exist in only some of the three databases** (Sylvite is pitzer-only;
  the figures were themselves corrected once, when the word "mineral"
  in the rendered sentence exposed 24 gas phases hiding in the count) —
  so KCl cooled the beaker on one path and not the other, the two orders
  ended 0.82 K apart, and dpH/dT ≈ −0.0163/K made a temperature bug
  masquerade as a composition mystery. Enthalpy had stopped being a
  state function; fixed generically in 39c592e (the fix keys on absence
  from the routed database, not on any mineral). The residual left after
  that fix was then *rationalised* — "the solver's own 0.05 K convergence
  tolerance, worth ~8e-4 in pH" — and a tolerance was derived on top of
  the rationalisation, which is the sixth wrong turn and the most
  instructive: the residual was a third bug wearing an explanation small
  enough to be believed. Dissolved matter carries no heat capacity in
  this model, so `t0 + q/cp` destroyed sensible heat whenever speciation
  shrank the vessel's Cp; balancing enthalpy instead (c1d493c) took
  pure-salt order-independence to machine precision (2.3e-10 in pH) and
  the precipitating scenario to 1.9e-6 — and, en route, made Hess's law
  hold exactly, which turned a dead codex entry writable. The test's
  tiers are re-derived from post-fix measurements: element totals 1e-9,
  phase split 1e-8, pH and ionic strength 1e-5 — the pH tier deliberately
  below the smallest historical bug signal (7.9e-5), so a recurrence of
  any bug in this chronicle fails the suite.
- **Perturbation tests** — the same question metamorphic relations ask,
  asked across a change to an INPUT rather than across two runs of the
  same one: does a number move, in the right direction, when its stated
  cause moves — and does a quantity the bench claims is independent stay
  put? **Built 2026-09-16** (`kerotakis-cli/tests/perturbation.rs`, six
  passing cases over five mechanisms plus one failing case it found, through the real binary and the `--json`
  contract). It is the generalisation *across time* of
  `tools/curiosity-answer-invariance.py`, which asks it across *vessels*
  — a prompt that distinguishes three metals must give three answers, and
  `mat-012` gave one for as long as the corpus existed. Neither half needs
  a reference value, so nothing is sourced, cited or licensed, and nothing
  is re-blessed when a solver improves. The cases were chosen from what
  the engine *claims* — the codex entry `moles-from-mass` states outright
  that the count you weighed is not negotiable by water, and that is case
  6 — rather than from what is easy to vary. Each covers a defect shape
  this repository has already paid for by hand (the figures below are
  the measurements that motivated each threshold, not pins — every
  assertion is deliberately looser than its measurement, so a solver
  improvement moves the figure and not the test): alkalinity as a charge and
  not a portion (falls 1:1 with added acid, sodium untouched, carbon
  leaving at fixed pCO₂); buffering computed from the pair actually
  present (the same 2 mmol of acid moves plain water 122× more than an
  acetate buffer, total acetate conserved); a rate law that still reads
  its own activation energy (peroxide at 75 kJ/mol is 2.64× per ten
  degrees against the thiosulfate clock's 1.90× at 51); buoyancy as a
  comparison against whole-object bulk density (the float verdict follows
  a sugar sweep across the object's own declared 1.08 g/mL, the threshold
  read out of the scene rather than pinned); and an element arriving
  mid-solve surviving readback (magnesium poured into an already-solved
  beaker is speciated, scales with the pour, and is debited from the
  source). **Every case's doc comment states what it establishes and what
  it cannot**, and says how close its perturbation sits to the path it
  tests — case 3 is labelled the weak one in the file itself, because
  given a per-reaction Ea inside an exponential, "the steeper barrier is
  the more sensitive one" is arithmetic, and it earns its place on
  plumbing rather than physics. **Non-vacuity demonstrated, not argued**, at two
  levels. Mutating the TEST — inverting or over-tightening one assertion in
  each of the six — makes all six fail, so no assertion is decorative.
  Mutating the ENGINE is the stronger check and the one that paid: giving
  peroxide the thiosulfate clock's activation energy (75 → 51 kJ/mol, the
  shape of a rate law that lost its own Ea) fails case 3, and hard-coding
  the rendered float comparison against water rather than the liquid
  present fails case 4 — but only after case 4 was widened. The first draft
  read the scene's `position` and **passed that mutation**, because the
  buoyancy comparison is made TWICE, once for the scene and once for the
  sentence a person reads, and the mutation left the scene saying
  `floating` beside the words "is at the bottom". Reading both surfaces is
  now part of the case. The lesson generalises past this file: a mutation
  applied to the test can only show an assertion is live, while a mutation
  applied to the engine shows whether the assertion is pointed at the code
  that could actually break. Note what this suite is deliberately NOT
  sensitive to: a fourth-significant-figure error in a datum passes every
  case here by construction, because no case has a reference value to
  disagree with. It catches wiring, not calibration; the oracles catch
  calibration. Cost: ~26 s of wall clock, 24 binary invocations, no data
  and no network.
  **It found one defect on its first outing**, and the seventh case is that
  defect: the `contents` entry named `OH-` — the lv3 machine contract every
  `--json` client reads — was the solution's residual cation charge, not
  hydroxide. In an equimolar acetate buffer it read 5.17e-4 mol where a pH
  of 4.66 can hold 4.6e-10, and under a perturbation it moved 1.30x where
  hydroxide must move 2.33x; the separately-kept `free_hydroxide`, which the
  solver writes on its way out, moves by exactly 2.33x and is right. The
  ratio was 7x for bicarbonate, 1.28x for sodium acetate and 1.12x for
  sodium hydroxide — 1 only where the charge really is carried by free base,
  which is the case `Vessel::free_hydroxide`'s own doc comment warns must
  not be generalised ("Reading either as hydroxide invents a neutralisation
  that never happened, at 55.81 kJ for every mole of it"). It is the
  alkalinity defect from the other side: there a charge was mistaken for a
  portion, here a charge was published as a species.
  **Fixed 2026-09-16 by renaming the carrier**, which was the `--json`
  contract change the case was waiting on an owner to decide: the slot is
  `base_equivalents` (`species::BASE_EQUIVALENTS`), the base half of the
  analytical acid/base equivalents that close a solved vessel's H/O balance,
  and the arithmetic behind it is untouched. `OH-` is left to mean hydroxide
  — the chloralkali cell deposits caustic soda under exactly that key — and
  case 7 now asserts both halves: that nothing named for hydroxide holds
  more of it than the pH can hold, at either end of an acid sweep, and that
  the renamed key falls one mole per mole of strong acid over a fourfold
  range, which is the definition it now carries. The acid half is still
  published as `H+`, is titratable acid rather than free protons, and is the
  same shape of defect; it is recorded in the case's doc comment and in
  `species::is_acid_base_basis`, not fixed, and `kerotakis-safety` already
  declines to read it as a strong-acid bottle.
- **Mutation testing** (`cargo-mutants`) — distinguishes load-bearing
  invariants from decorative ones, which is this project's epistemics
  applied to its own test suite.
- **Lessons as tests** — every scenario file replays in CI via the operator log: no lesson may go silent, hit a solver failure, or break the `--json` contract, and the pre-warmed cache must cover them. (This test immediately caught `inspect` printing prose into the JSON stream.)
- **Snapshot tests on `--json`** — the CLI's JSON output is the API
  contract. Migrate the hand-pinned shape tests to `insta` with
  redactions for volatile fields: `cargo insta review` makes an
  intentional contract change an audited event rather than a test edit.
- **CI must enforce what the plan claims — our own standard, applied to
  us.** Two claims currently rot: the **iOS gate** (passed 2026-08-19)
  has no CI step — add
  `cargo build -p kerotakis-phreeqc --target aarch64-apple-ios` on the
  macOS runner — and there is **no wasm size budget**, though the
  offline/mobile premise depends on one: assert a byte ceiling on the
  built module so a dependency cannot silently add megabytes. Smaller:
  install `wasm-bindgen-cli` from a prebuilt binary rather than
  `cargo install`, and let Dependabot watch the actions and the iphreeqc
  submodule.

---

## Data provenance, verified

The traps are all about data, not code. Checked against primary sources
2026-08-18; the conclusions changed the plan.

| Source | Terms | Verdict for us |
|---|---|---|
| PubChem (NCBI bulk) | No NCBI restrictions, commercial OK; per-annotation source attribution expected | **Primary property + GHS source.** Keep attribution per record |
| Wikidata | CC0 | Clean supplement; coverage is thin (≈2k boiling points, ≈310 pKa) — cannot carry the load |
| NASA CEA (`github.com/nasa/cea`) | **Apache-2.0** incl. `data/thermo.inp` | **Primary thermochemistry source** for L2g and formation enthalpies |
| Cantera data files (gri30.yaml etc.) | **No licence granted** — BSD-3 covers Cantera's code only | **Not redistributable.** Cantera states it "is not claiming to grant a license to" the mechanisms it ships, and that its input files are "for illustration purposes only". GRI-Mech's own site carries a disclaimer, never a grant. Oracle-only; see `provenance/brd-040-cantera-audit.md` (BRD-040) |
| CLP Annex VI via EUR-Lex | EU legislation, reuse with acknowledgment | Harmonised GHS/CLP hazard classes — take from EUR-Lex, not ECHA dumps |
| PHREEQC databases | USGS User Rights Notice (public-domain-like, attribution) | Embed (except `sit.dat` — ThermoChimie provenance, needs a terms check) |
| CAS Common Chemistry | **CC BY-NC 4.0** | Unusable commercially. Never present CAS RNs as licensed-from-CAS data; identifiers come from PubChem/Wikidata |
| NIST WebBook / JANAF-online | **NIST SRD — copyrighted**, permission required | Do not harvest. Cite a single value with attribution if it is genuinely the source; prefer tracing to the original measurement. |
| NSRDS-NBS 37 (JANAF, 1971) | **Public domain**; no copyright notice, no SRD notice | Own row deliberately: it was a parenthesis inside the row above, which a machine reading loses or inverts. Dated, and usable. |
| NBS Circulars 461 and 500, *J. Res. NBS*, NBS Technical Notes | **US Government work, not SRD** | Cleared. The line is SRD status, not authorship — that distinction unlocked five replacements on 2026-09-13. |
| NBS thermochemical tables (Wagman et al. 1982; TN 270; Circulars 461/500) | **US Government work** — NIST Technical Series, 17 U.S.C. 105, "not subject to copyright protection in the United States" (read 2026-09-15) | **Cleared.** Cite the **Technical Note**, not *J. Phys. Chem. Ref. Data* 11 Suppl. 2: the Supplement is the same numbers in an SRD journal. `kerotakis/dissolution-enthalpies-v1` names both in one parenthesis as if they were interchangeable. |
| CODATA fundamental constants | NIST's presentation is **SRD 121**, §290e compilation copyright; but the SI defining constants are **exact by definition** and the BIPM SI Brochure is **CC BY 4.0** (both read 2026-09-15) | **Cleared for what we actually ship.** Every constant in `constants.rs` is exact by SI definition or derived from ones that are, and the crate's tests check it. A *measured* CODATA value would be a transcription out of SRD 121 and needs its own reading. Not the same product as the CODATA Key Values for Thermodynamics (Cox 1989), whose terms are unread. |
| USDA FoodData Central | **CC0 1.0** — "in the public domain and they are not copyrighted" (read 2026-09-15) | **Cleared.** Attribution requested, not required. Agrees with `provenance/sources.toml`'s vendored-bytes record for the same source. USDA **ERS** is a different product and is not covered. |
| Primary journal literature (the paper that made the measurement) | *Feist* 499 U.S. 340 (1991): facts are not copyrightable; compilation copyright reaches only selection and arrangement (read 2026-09-15) | **Cleared, and preferred to both of the others.** One number with author, title, journal, volume, year, pages and a DOI. It does **not** licence a table, figure or fitted correlation lifted whole. A route, not a body, so its `names` list is per-paper and always one paper behind. |
| CRC Handbook of Chemistry and Physics | Copyrighted commercial compilation, no reuse licence (publisher page returns 403 to an automated request; verdict rests on the absence of a grant, not on a reading) | Citable for a value with attribution. **Not** a systematic source: do not depend on it in bulk, and trace to the original measurement where one exists. |
| Merck Index | Copyrighted commercial compilation | As the CRC row. |
| Majer & Svoboda, IUPAC Chemical Data Series No. 32 (1985) | Copyrighted | As the CRC row. **No row in `provenance/upstreams.toml` yet**, so the lint cannot see it; add one when someone reads its terms. |
| IUPAC/CIAAW standard atomic weights | Free educational use and republication with attribution "without the need for formal IUPAC or CIAAW permission"; then **"For commercial use of this content please contact CIAAW Secretariat"** (read 2026-09-15) | ⚠️ **Decision required.** A grant exists, so this is not the CRC row; it stops at commercial use, so it is not a clearance. Nobody has asked the Secretariat, and *Feist* may mean nobody has to — the registry ships molar masses **computed** from ~20 element values, not a copy of the table. **152 registry citations name it**, so the open-question column reads 152 where it has always read zero. That is a dependence becoming visible, not a regression. |
| ACS classroom material (Middle School Chemistry) | **Not read** — acs.org answers an automated request with a filter page and no body (checked 2026-09-15) | ⚠️ **Decision required**, not *avoid*: the CRC verdict rests on the absence of a grant on a page that loaded, and here nothing loaded. Cited once, qualitatively. ACS **journals** (J. Chem. Educ., ACS Omega) are primary literature and are not this row. |
| FAO | Copying "for private study, research and teaching purposes, and for use in non-commercial products or services" with attribution; commercial rights on request (read 2026-09-15) | ⚠️ **Decision required.** Same shape as the CIAAW row and judged the same way. What is cited is Gay-Lussac's 1815 fermentation equation — a fact rather than FAO's expression of one, which is why the question is open rather than answered against us. |
| Voet & Voet, *Biochemistry*, 4th ed. (Wiley) | **Not read** — wiley.com returns HTTP 403 for the title page and for /en-us/permissions (checked 2026-09-15) | **As the CRC row.** Adds exactly one finding, `legacy/amylase`, whose citation carries no page and calls its own value "typical". The vocabulary had no way to say "citable once, refused in bulk", which is what the 2026-09-14 rule actually says; it gained one on 2026-09-15 (`[[citation]]` `role`, below) and this row does not move, because no page means it is not attribution. |
| CAMEO / CRW4 database | Contributed fields explicitly non-duplicable (CAS RNs, NFPA, AEGL, ERPG) | Never ship the database; reimplement the published methodology (L0 note) |
| ECHA C&L exports | IP-encumbered (CAS data named) | Avoid; use EUR-Lex / PubChem routes |
| Burcat (Third Millennium) | Free non-commercial only | Skip, or write for permission if CEA coverage falls short |
| Open Reaction Database | **CC-BY-SA 4.0** on data; ShareAlike propagates to merged datasets | **Decided: BY-SA is acceptable.** The curated dataset is published BY-SA (code stays AGPL — separate works, separate licences); an educational commons staying open is a feature, not a cost. What we do *not* accept is a licence being quietly dropped: where an upstream relabels BY-SA data as CC BY or MIT (ORDerly's Figshare deposits; the CaCS SQLite), we honour the **original** terms and say so |
| `chemicals` (Python) | MIT code aggregating CRC/NIST/Yaws/Common Chemistry data | **Dropped as a data source** — it launders the SRD and NC problems into our binary. (Its sibling `thermo` remains a build-time *oracle*, generating test fixtures, not shipped data) |
| UNIFAC parameters | Consortium tables proprietary; original journal tables usable | Source from the original publications, provenance per parameter |
| USPTO reaction data (Lowe extraction) | **CC0 — verified via the figshare API**, patent text USPTO-confirmed copyright-free, reaction facts *Feist*-uncopyrightable | The clean chain under every USPTO-derived corpus/model; relevant only to the deferred ML tier and template tooling |
| CRD — Chemical Reaction Database (van der Lingen) | **CC BY 4.0** (Figshare, verified) — 1.37M reaction SMILES from US patents + literature (1.44M as of Jan 2026) | **The best-licensed bulk reaction corpus found** — attribution only, no ShareAlike. But SMILES only (no conditions/observations), patent-chemistry distribution, semi-automated with author-acknowledged errors. Build-time **template-mining** corpus at most; never codex content |
| ORDerly benchmarks (Figshare 23298467 / 23502372) | Deposits say **CC BY 4.0** — but the data is extracted from **ORD (CC-BY-SA-4.0)**, and neither the deposits nor the JCIM paper mention ShareAlike anywhere | ⚠️ **Decision required before any use.** The data itself is attractive (≈919k/939k/691k reactions with solvents, agents, **temperature**, time, yield). If ever used: treat it as **BY-SA regardless of the depositors' claim**, attribute both ORDerly and ORD — or stay out. Note `procedure_details` is verbatim patent prose, not pedagogy. Do **not** run their toolchain (dormant, CI red since 2024-06) |

### The licence discipline becomes a lint

Everything above is enforced by prose and care, which is how the codex
worked before `codex lint` existed — and the fix is the same fix:

- **`cargo-deny`** in CI: a licence allowlist (an LGPL-2.1-only crate is
  then blocked mechanically, not by someone remembering this document)
  plus RustSec advisories.
- **`cargo-about`** generates the attribution/about screen from crate
  metadata instead of a maintained page.
**Ten defects in the prose table itself, found 2026-09-13 while converting it
to `provenance/upstreams.toml` and while re-sourcing 218 values against it.**
Recorded rather than fixed, because six of them need a verdict that is the
owner's to give. A lint can only be as good as the table it reads, and four of
these would make it silently wrong rather than merely incomplete.

*Sources the repository uses and never judged.* Two of the three below were
closed on 2026-09-15 and one is still open; the list is kept as written because
it is the record of how the gap was found.

- **The CRC Handbook had no row**, and is the most-cited refused source in the
  codebase — 88 mentions in shipped Rust when the audit began. The policy on it
  was inferred from the commercial row. A lint generated from the table as it
  stood would not have caught a single one of them. *Row added; `avoid`.*
- **The Merck Index had no row**, and is cited in shipped code. *Row added;
  `avoid`.*
- **Majer & Svoboda, IUPAC Chemical Data Series No. 32 (1985)**, has no row, and
  is cited in shipped code. **Still open.** Attempted 2026-09-15 and not added:
  the 1985 volume has no terms page of its own, and its publisher's successor
  returns HTTP 403 to an automated request, so unlike the CRC row there is not
  even an absence-of-a-grant on a page that loaded to rest a verdict on.
  Inventing terms for it is what this bullet exists to prevent.

*Rows whose shape defeats a machine reading.* Each of these survives in prose
because a human reads around it:

- **NSRDS-NBS 37 is an ALLOW source nested as a parenthesis inside an AVOID
  row.** The NIST row says do not scrape or redistribute, and then grants the
  public-domain 1971 tables inside the same cell. Any conversion either loses
  the grant or inverts the refusal. It wants its own row.
- **The NIST row bundles two sites with two different terms pages.**
- **The UNIFAC row holds two sources with opposite verdicts.**
- **`sit.dat`'s carve-out and `thermo`'s oracle permission each live only in a
  parenthesis.**
- **One retrieval date, 2026-08-18, covers twenty rows** carrying evidence from
  other dates.

*Two verdicts that are wrong rather than missing:*

- **NASA CEA's verdict needs a boundary, and this one is settled by evidence
  rather than opinion.** It is sound for gas heat capacities and formation
  enthalpies, which is what the row claims. It is NOT sound for transition
  enthalpies: `thermo.inp` fits each phase over its own interval and nothing
  constrains two fits to meet at the evaluated transition, so their difference
  at the boundary carries both residuals. That shipped a wrong enthalpy of
  fusion for silver in #586 and was corrected in #595. It is also not sound for
  vaporisation, where the ideal-gas records overshoot by 0.6–3.3 % measured
  against water, nitrogen, ethanol and methanol. CEA is a cross-check for those
  two quantities, not a source.
- **The line is SRD status, not NIST authorship, and the table does not say
  so.** NIST's own policy is that works authored by its employees are not
  subject to copyright within the United States, while the WebBook carries a
  notice governed by the Standard Reference Data Act. That distinction is what
  makes NBS Circulars 461 and 500, the *Journal of Research of the NBS* and the
  Technical Note series usable, and it unlocked five of the replacements this
  week. Left implicit, a reader refuses sources that are in fact open.

**The governing principle, stated by the owner 2026-09-14, correcting how the
2026-09-13 ruling had been applied.**

> We should be able to cite any book. Only not harvest the books by systematic
> scraping. And we should be able to trace original sources for almost all
> values, and cite those.

So the line is **bulk dependence, not attribution**. Naming a book as the
source of one value, with author, title, edition and page, is ordinary practice
and is always allowed. What is refused is leaning on a copyrighted compilation
as the systematic source of many values, which is what scraping looks like in
effect whether or not a scraper was involved. And the preferred answer is
neither: trace the measurement to the paper that made it and cite that.

**A correction, because the stricter reading did damage.** During the
2026-09-13 work "a book with no digital object identifier" was treated as
uncitable, and a world-facing comparison in the dilute-brine test was dropped
for that reason. That was wrong. A reference work without a DOI is cited by
author, title, edition and page like any other book. The dropped anchors should
be restored on that basis, and the same debt on the one-molal test beside it
closed the same way. An identifier is a convenience for the reader, never the
thing that makes a citation legitimate.

This does not weaken the 2026-09-13 ruling, which was about a different
failure: a row that said only that its value "agrees with" a refused
compilation was offering agreement in place of a source, and never named where
the number came from. That still stops.

**The instrument learned to say it on 2026-09-15.** Until then `kero provenance
upstreams` could not: `avoid` is a property of a *source*, and this rule is a
property of a *citation*, so one properly attributed citation produced a
finding for exactly the practice the rule calls ordinary. `[[excuse]]` in
`provenance/upstreams.toml` became `[[citation]]` with a **`role`** —
`mentioned` (names it, claims nothing from it), `via` (the value is a cleared
primary's, read through this source's rendering) and `claims` (the value is
this source's). An attributed `claims` — one carrying a **`locator`**, the
page or table — on a work refused for *bulk* is not a finding. Two things are
counted apart, because the rule has two clauses with two different units:
attribution is per **citation**, and dependence is per **source**, capped by a
declared `cite_at_most` that is zero everywhere until somebody writes a reason
for a number. Going over it is *one* finding for the work, not one per
citation. The honest unit for both is the **quantity**, which neither surface
can see, so both figures are lower bounds.

Two limits worth knowing before quoting any of it. The carve-out stops at
`avoid` and never reaches `permission-required`: attribution answers
compilation copyright and cannot answer "nobody granted permission", and
`legacy/liquid_nitrogen` proves it by citing the NIST WebBook with a CAS
number and a deep link — a better locator than any CRC citation in the tree —
while still being a transcription out of Standard Reference Data. And a
`locator` is checked for presence, never for truth: three rows in
`phase_route.rs` name a CRC table and admit in the same sentence that no copy
was opened. The change moved the Rust surface from 18 findings to 7 and left
the registry's 60 exactly where they were, which is the point — the 46 CRC
citations there name an edition and no page.

**Decided 2026-09-14, by the owner.** Six, with the reasoning kept short
because each was argued at the time.

1. **The silent catalogue failure is surfaced and retried.** It is why the
   shelf had nothing to know, and a deployment in that state was broken with no
   diagnostic. Check with it whether the service worker can pair a new app
   bundle with a cached older engine, since that pairing is what reproduced it
   and would make it recur after every release.
2. **The three unjudged sources are citable as commentary, not as claims**,
   which is what the codebase already does after the withdrawal work, so the
   table now describes reality rather than aspiration. Rows added above, with
   the nested public-domain source given its own row, the vendored
   thermochemistry file recorded as a cross-check rather than a source for
   transition and vaporisation enthalpies, and the SRD-versus-authorship line
   made explicit.
3. **The provenance lint is promoted to failing only when its count reaches
   zero without the denominator falling with it**, in a change that does
   nothing else.
4. **Records that name no audited source at all are counted and reported, and
   the requirement binds new records only.** More than half the registry is in
   that state; stopping the bleeding beats a sweep that would take months.
5. **The accuracy corpus starts with one family, the colligative one**, six
   rows, real citations, no gate. The first obstacle is the one already met and
   now resolved above: its reference values are citable.
6. **Every corpus row gets an expectation and boundary rows execute their
   scripts.** The headline count will fall, which is the point: a measure being
   retired should stop misreporting on its way out.

**Decided 2026-09-13, by the owner, after an audit found the rule unenforced.**
Recorded here because two of these were open questions that stalled work, and
because the audit found roughly eighty-five shipped numeric claims resting on
avoid-row sources while the lint that would have caught them was still unbuilt.

- **"Agrees with NIST SRD 69" is a CLAIM, not a citation, and it has to stop.**
  Prose mentioning such a source stays as commentary. This is what the rule
  below already said; it is restated because twenty-three places relied on the
  looser reading, and because an accuracy corpus cannot start citing anything
  while the question is open.
- **Where no openly-licensed source of equal quality exists, the capability
  shrinks rather than the claim softening.** A value may keep its number with
  the claim withdrawn, or a species may lose a curated parameter and the bench
  refuse the transition. An honest refusal beats a quiet approximation, and
  that is preferred here to inventing support.
- **A candidate source that cites an avoid-row source is not a replacement.**
  Checked on 2026-09-13: Wikidata carries no enthalpy of fusion or of
  vaporisation for ethanol at all, and its standard enthalpy of formation cites
  the CRC Handbook. The table above already says Wikidata "cannot carry the
  load"; this is the sharper reason. Trace every candidate to its own primary
  reference and record what that is. The same reasoning dropped a Python
  package as a data source, for laundering.
- **Reference values are individually cited measurements first, build-time
  oracles as the fallback.** An oracle that shares the database it is checking
  is not independent validation of that database.
- **Tolerance is argued per quantity**, never a flat percentage: a pH, a
  freezing point and a flame temperature do not deserve the same bar.
- **The accuracy work starts as a backlog, not a CI gate**, and is promoted
  only once its tolerances have survived argument. A premature gate would be
  the third check this month that read stronger than it was.
- **The number published is "quantities covered with a cited source and an
  argued tolerance", never "rows passing"**, which can be gamed by adding easy
  rows.
- **`electrodiffusion.rs` gets wired up** rather than deleted or left orphaned.

- **The provenance table above moves to machine-readable TOML**
  (per source: licence, terms URL, retrieval date, verdict, what it may
  touch) with a `kero provenance lint` that checks every entry in
  `Cargo.lock` and `vendor/` appears in the audit and nothing ships from
  an avoid-row source. The prose stays as commentary; the claims stop
  being able to go stale silently — the `equation`/`summary` move,
  applied to licensing.

  > **Built 2026-09-13** as `provenance/upstreams.toml` plus
  > `kero provenance upstreams`. The verdict vocabulary is three-way, as the
  > decisions above require: a source may support a shipped claim, may only
  > CHECK one as a build-time oracle, or may only be mentioned. A claim and a
  > comment are told apart by position, not by wording — a string on the right
  > of a `provenance`/`source`/`citation` field is a claim, a `//` comment
  > carrying the same words is commentary. Two rows this table never judged,
  > the CRC Handbook and the Merck Index, are carried there as
  > `in_plan_table = false`: between them they are cited by more values than
  > every row above put together. It REPORTS and exits zero while the count
  > comes down. `crates/kerotakis-cli/src/upstreams.rs` states, in numbers,
  > what it catches, what it cannot, and what must be true before the gate is
  > promoted to failing.
- **SPDX headers** (REUSE) on data files; a CycloneDX SBOM then falls
  out for free.

---

## Nine to expert, one simulation

Never dumb down the model, only the view. One PHREEQC result, rendered at
whatever register the reader is in. The child and the postdoc see the same numbers.

| Level | Output |
|---|---|
| **lv1** | "It went cloudy! A white solid appeared — that's a *precipitate*." |
| **lv2** | `AgNO₃ + NaCl → AgCl↓ + NaNO₃` · 0.010 mol · Ksp = 1.77 × 10⁻¹⁰ |
| **lv3** | SI(AgCl) = +2.41 · I = 0.021 m · γ(Ag⁺) = 0.857 · full selected-output |

Levels are numbers, not audiences: naming them "age 9" or "child" bakes in
an assumption about who a level is for, and the levels will multiply (a
step between equations and full numerics, say). `Register` is therefore a
`u8` with named constants, unspecified levels inherit the nearest one
below, and the codex keys its copy by `lv1`/`lv2`/`lv3`/… so adding
granularity is a data change rather than a schema change.

Registers are a presentation concern and live entirely in the UI (and the
CLI's renderer). The solver has no idea who is asking. Register copy is
generated by deterministic templates over solver output ("SI > 0 and new solid
phase → 'went cloudy'"), never by a language model — offline, reproducible,
trustworthy. The same registers apply to orbitals: age 9 gets "the electron
clouds have to match up like puzzle pieces"; the expert gets the molden file.

The template format should be **Project Fluent** (`fluent-rs`, Apache-2.0,
verified 2026-08-19) rather than ad-hoc strings: plural, case and gender
rules live in the `.ftl` data, which is what the German line will need —
ad-hoc templates fight German grammar — and it is the flavour-is-data
commitment applied to grammar. Deterministic and offline either way;
Fluent changes where the rules live, not who writes them.

### The alchemical layer earns its keep

The twelve classical operations map almost directly onto our operator list, so
the naming system *is* the difficulty ladder rather than decoration:

| Child | Modern | The Work |
|---|---|---|
| Heat it up | Thermal decomposition | Calcination |
| Let it settle | Precipitation | Coagulation |
| Boil it off | Fractional distillation | Distillation |

The four stages of the magnum opus — nigredo, albedo, citrinitas, rubedo — are
cuts through the codex concept graph, realised as lesson-file tiers.

---

## What this will not do

Worth writing down before starting, because each is where an ambitious version
quietly fails.

- **Predict arbitrary organic reactions.** Genuinely unsolved. Curate, and be
  visibly honest where we are predicting rather than knowing.
- **Mechanisms and transition states at runtime.** Quantum chemistry is
  build-time; the runtime ceiling is honestly-labelled Hückel.
- **Extremes.** Plasmas, exotic organometallics, solid-state band structure,
  high pressure. (L2g's CEA data does extend T range honestly for gases and
  simple condensed phases; database validity ranges are surfaced, not hidden.)
- **Unbounded biochemistry.** A different stack, not an accidental extension
  of the aqueous or organic routes. `BREADTH.md` now scopes a deliberately
  bounded `Biochemical` route (`BRD-050…052`) for familiar enzyme, digestion,
  fermentation, respiration and photosynthesis experiments; it still does not
  claim to model cells, medicine or complete metabolism.

A general-purpose engine that computes any reaction from first principles is also
a synthesis oracle for things we do not want it computing. Curated-first gives us
an explicit, auditable boundary — a product-safety property, much easier to
defend than a filter bolted onto a general predictor.

Optional later module, cheap and on-theme: radioactive decay chains (Bateman
equations — trivial math, public-domain nuclide data, half-lives are a
curriculum staple, and decay is the most necromantic chemistry there is).

---

## The v1.0 cut

Eight phases and thirteen crates have no floor without an explicit line.
**v1.0 in a store is:**

- P0 + P1 + P2 — the bench, the safety veto, and aqueous chemistry
  (acid–base, precipitation, titration, solubility, buffers, brines)
- a **~40-reaction codex slice** with full register copy, concept links and
  observations — curated to one curriculum's inorganic aqueous block, not
  breadth-first
- the **curated-colour sliver of L6** (colour word + sRGB per species,
  concentration-driven opacity): the age-9 register *is* observations, so
  v1.0 cannot ship without "it went cloudy" — but it needs no spectra
- **one UI** over the same `--json` contract the CLI already snapshot-tests
- all three registers — they are the product's identity, not a feature
- **the particle view**, added 2026-08-19: a submicroscopic renderer driven
  by computed speciation. Without it the product has macroscopic, symbolic
  and deeper-symbolic views and Johnstone's triangle is not closed — which
  is the one pedagogical claim the whole design rests on. It may be humble
  (2-D dots at solved ratios) but it may not be absent
- **the browser as a real bench, not a lesson player** — **built and
  published**, at <https://crispstrobe.github.io/kerotakis/>. The two
  wasm halves are wired together: `Lab.setSolver()` takes a JavaScript
  function, `web/kerotakis.mjs` backs it with the Emscripten build of
  IPhreeqc, and everything above the hook is unchanged — same routing, same
  content cache, same temperature fixed point, same parsers. The web gets
  the same answers **by the same path** rather than a second implementation
  that could drift from the one the codex was linted against, and CI proves
  it by recording the desktop build's answer to a deliberately un-warmed
  question and requiring the browser to match it to 1e-6 in pH. A bench with
  no solver attached reports `canSolve() == false` and refuses rather than
  guessing, which is the honest version of what shipped before
- **the demo demonstrates the premise** — **built, 2026-08-20**: the page
  is a PWA (manifest, icon, versioned service worker precaching the
  shell, both engines and all three databases), and the headless CI test
  now *proves* the premise rather than assuming it — first load online,
  then the server is killed and the page must boot from the worker's
  cache with the engine live and the precipitate still forming. "Turn
  off wifi, it still solves" is a test assertion. The Web-Worker half is
  deliberately not done and recorded as its own piece of work: the
  solver hook is synchronous *by design* (Rust calls JavaScript and
  waits — that synchronicity is what makes the bridge possible at all),
  so moving it off the main thread means Asyncify or SharedArrayBuffer
  plumbing, not an afternoon

Explicitly **not** in v1.0: P2g (v1.1, with `ignite`), P3p (v1.1, with
distillation), the QM/orbital layer, Hückel, lessons beyond the codex slice,
and everything ML. Each later phase extends the same bench; nothing in v1.0 is
scaffolding to be thrown away.

## Breadth build order: from solver depth to familiar matter

The original P0…P7 sequence establishes solver architecture. The companion
**[BREADTH.md](BREADTH.md)** establishes the content/integration sequence that
makes those solvers answer ordinary learner questions. Its `BRD-*` graph is
subordinate to the same state, provenance, offline and honesty invariants here.

The key architectural addition is `MaterialRecipe`: named matter such as
vinegar, milk, soil, paper, steel or a battery expands into conserved,
versioned components while retaining the name and assumptions the learner
used. Pure identities continue to join by Standard InChIKey; material recipes
never masquerade as molecules. Safety sees expanded components before solver
routing, and unresolved fractions remain visible and conserved.

The breadth sequence is:

```text
measure curiosity and typed gaps
  → define material recipes and quarantined source adapters
  → ship familiar pure-substance and household-material packs
  → add bounded, condition-gated reaction families
  → broaden feos/Cantera routes where cleared parameters exist
  → add bounded biochemistry and crystal structures
  → add tactile physics and scientific viewers over authoritative state
  → graduate against the versioned curiosity corpus
```

This does not reorder P0…P7. A `BRD-*` task may start when its explicit
dependencies and the corresponding P-stage substrate are merged. In
particular, data adapters do not wait for optional engine FFIs, and UI work
does not begin before its state/authority contract.

## Build order

Genuinely sequential — each phase is shippable on its own, each depends on the
state model the previous one hardened, and **from P1 on, the CLI is each
phase's acceptance demo**.

### P0 — Feasibility spike — done 2026-08-19

The single highest-information task; everything else was downstream of it.
The offline premise holds on web, mobile and native. See `HISTORY.md`.

### P1 — Bench state machine + energy balance + L0 + CLI

The bench state machine, enthalpy bookkeeping, the CLI and the 256-case
conservation proptest are done; see `HISTORY.md`. Open:

- [ ] Grow the seed matrix to the full published NOAA group set with
      SMARTS-driven group assignment (needs Indigo; legal sourcing per the
      L0 note)

### P2 — PHREEQC, shippable on its own

The PHREEQC equilibrator, weak acids and buffers, database routing by
validity domain, the result cache, carbonate chemistry with an open vessel,
expert-register speciation, phosphate, brines, hard water and `kero prewarm`
are done; see `HISTORY.md`. Open:

- [ ] Registry breadth continues with L1 (PubChem/Wikidata export)
- [ ] The P2 CLI **is** the "strong product on its own" claim, tested literally

### P2g — Heat and fire

NASA-9 thermochemistry, the Gibbs minimiser, `ThermalEquilibrator`, the
`ignite` operator and enthalpy-conserving adiabatic vessels are done; see
`HISTORY.md`. Open:

- [ ] Validate a wider set against build-time Cantera oracle runs

### P3 — Reordered by curriculum weight, 2026-08-19

The original P3 was VLE/UNIFAC. That is now **demoted below P3e and P3k**,
on a straightforward value-per-effort argument: distillation and azeotropes
are a sliver of school chemistry, while redox and rates are enormous
blocks, and both are *already most of the way there* underneath. Phase
behaviour returns when the school-facing layers are covered.

#### P3s — States, freezing and boiling  ← closed, 2026-09-07

All four items are done; see `HISTORY.md` for what each one taught. The
last of them is worth restating here because it is a rule rather than a
feature: **a vessel whose solvent is not a settled liquid does not report a
solution.** Ice has no pH. A beaker on the boil has no *settled* pH,
because solvent is leaving while the reading is taken and every molality
the engine solved for belongs to a composition that has already changed.
`solve::SolventState` is where that is decided, the honesty pass is where
it is said, and the readout is withdrawn in the same breath so the meter
cannot contradict the sentence.

Two boundaries the closing tranche wrote down rather than crossed, both in
`phase_route.rs`:

- **A boil is only given to a species the registry carries as a liquid.**
  `condensation_partner` can find a vapour's way back only for those, so a
  boil given to a standard-phase solid — iodine, naphthalene, molten zinc —
  would be one-way, and a transition this bench pays for has to run both
  directions. Those substances melt and do not boil.
- **No metal boils.** Zinc's 1180 K boiling point is inside a Bunsen's
  reach and zinc fume is a named hazard, so it wants its own tranche with
  its own safety row.

Open, and small:

- [ ] `paraffin` still carries no melting point. The note giving the
      reason has been corrected — it used to blame the state model for
      covering nothing but water, which has stopped being true — and the
      real obstacle is now written down instead: a candle blend spanning
      C20 to C40 softens across roughly 46–68 °C rather than melting at a
      point, and `PhaseTransitions` has five temperatures and no slot for
      a RANGE. Give it one, and the wax melts.
- [x] **The colligative relation WAS the DILUTE-solution law, used where it
      was about nine per cent optimistic** — it is now the relation that law
      is the limit of, run on the solvent's activity
      (`improve/colligative-water-activity`, 2026-09-11). One molal brine
      reads −3.44 °C against a measured −3.4 where ΔT = K_f·m said −3.72,
      and a saturated brine boils at 108.0 °C where the law said 106.1
      against a measured 108.7 — the correction changes SIGN near three
      molal, so the second end is not a rider on the first. The particle
      count never moved: it was not the problem, and the llnl NaCl ion pair
      stayed declined for the reasons written in `colligative_numbers.rs`.
      What supplies a_w, what does not and where each stops is in
      `states::SolventActivity`; `HISTORY.md` has the rest.
- [x] **A dilute brine took the ideal route, and now takes an
      ion-interaction one. Closed 2026-09-13 by asking `pitzer.dat` for the
      solvent activity in a second, lean speciation. A tenth-molal brine
      reads −0.3516 °C where Raoult's law said −0.3712.** The solvent
      activity is believed only when an ion-interaction speciation computed
      it, and the router sends a solution to `pitzer.dat` at 1 mol/kgw — so
      0.1 molal brine was answered by a Debye–Hückel dataset, whose
      reported water activity is PHREEQC's hard-coded `1 − 0.017·Σm`
      placeholder rather than a model, and Raoult's law stood in.

      **Why it was worth a second solve rather than a sentence.** Keeping
      Raoult and simply saying so was the real alternative, and the code
      already declared the route `IdealSolution`, so nothing was hidden. It
      loses on three counts. The correction is 0.020 K on a 0.352 K answer,
      which is six per cent of the quantity being reported. It is the
      SAME defect, in the same direction and from the same cause, as the
      one at one molal that was judged worth fixing on 2026-09-11 — Raoult's
      law standing in for a solvent activity — so declining it here would
      have left the bench on a modelled solvent at a molality nobody makes
      and on a mole-fraction guess at the one everybody does; a tenth molal
      is a level teaspoon of salt in half a litre. And the reason for the
      gap was never a modelling limit: `pitzer.dat` is vendored, loaded and
      routed to daily, and its Na–Cl virial coefficients (which that file
      attributes to Appelo, 2015, Appl. Geochem. 55, 62-71,
      doi:10.1016/j.apgeochem.2014.11.007) describe this solution perfectly
      well. It had simply never been asked.

      **What it does NOT buy, stated because the figure invites the wrong
      reading.** φ comes back at 0.945, and the route's accuracy at this
      dilution is capped near ±0.005 K — not by the model but by the
      readback. PHREEQC's species table prints four significant figures, so
      a_w arrives as 0.9966 rather than 0.996647, and
      φ = −ln(a_w)/(M_w·Σm) turns a 5 × 10⁻⁵ rounding in a_w into 0.013 in
      φ at 0.2 mol/kgw of particles: 1.4 per cent, all of it upward. The
      correction this buys is therefore real and is most of the gap, but it
      is not exact, and getting closer needs a_w off the wire at full
      precision, which is a change in the readback rather than in this
      route.

      **What it costs, measured rather than estimated.** `kero prewarm lessons/*.lab` replays
      every shipped lesson through the real engine, which is the most
      representative script set this repository has: **113 lessons, 1333
      steps**, plus the five R1 acceptance scenarios. On that run the
      second opinion cost **52 engine calls out of 750**, so the engine
      work grew by **7.4 %** — 698 calls before, 750 after. The difference
      is exact rather than differenced between two builds, because a
      second solve does not mutate the vessel and so cannot change which
      main solves happen; an earlier run of the same script set, with the
      feature mostly declining, measured the same 698 underneath it, which
      is the cross-check. A further **10 of the 62 solvent questions asked
      were free**, answered from the content-addressed cache — the lean
      problem earning its keep, since it drops everything about a vessel
      that does not change its solution's composition. The shipped cache
      grew by the same 52 entries, 667 to 719, about 33 kB on 955 kB. The
      whole prewarm took 40 s wall and 18.2 s of user CPU on a loaded
      four-core box; the call count is the figure to quote, because a wall
      clock measured under contention is not reproducible and a count is.

      **Is 7.4 % worth 0.020 K? Yes, and the argument is that the engine
      is not the bottleneck.** Replaying every lesson this bench ships
      costs eighteen seconds of CPU in total, and this feature is 7 % of
      that — a second and a third, once, at build time, for the whole
      corpus. On a bench step it is one extra solve on a vessel that was
      already doing one. What it buys is the dilute half of the
      colligative answer, which is the half a learner is most likely to
      meet. If the engine ever does become the bottleneck the counters are
      already there to find this again:
      `PhreeqcEquilibrator::solvent_activity_solves()`, printed by
      `kero prewarm`.

      **It is conditional, and that is deliberate.** An unconditional
      second solve would pay for solutions already on `pitzer` (the same
      solve twice), for solutions `pitzer` cannot express, for
      redox-coupled problems whose second solve would be a pe bisection,
      and for solutions too dilute for a four-significant-figure a_w to say
      anything. So it is asked only when the chemistry routed to a
      Debye–Hückel dataset, `pitzer.dat` carries every element, there is no
      gas phase, surface, exchanger, solid solution or surviving solid to
      make the posed totals not the solution's, the lean problem is not
      redox-coupled, and dissolved particles are at or above 0.05 mol/kgw.
      Below that floor a_w rounds to 1.000, φ computes as zero,
      `SolventActivity::from_speciation` rejects it, and the two routes
      differ by a few thousandths of a kelvin in any case.

      **So a vessel sometimes solves twice, and a reader can always tell
      which.** `SolutionInfo::solvent_activity` is `Some` on exactly the
      two-solve vessels and names the dataset the activity came from, which
      is not the dataset in `provenance` the pH and speciation came from;
      `Provenance::routing` says the same in prose wherever provenance is
      rendered; `kero` prints it on its own line; and
      `Transitions::activity_route()` reads `IonInteraction` rather than
      `IdealSolution`. Both directions are pinned in
      `colligative_numbers.rs`, by a tenth-molal test that asserts the
      record is there and a hundredth-molal test that asserts it is not —
      the second is what stops the trigger widening by accident into "every
      aqueous step solves twice".

      **What is deliberately NOT asserted, and why it is an open item.**
      Neither new test compares against a MEASURED freezing point or
      osmotic coefficient, and the prose above quotes none. The figures
      usually printed for this solution trace to Robinson and Stokes'
      tabulation — a book, no resolvable identifier — and the critical
      re-evaluations of it are `J. Phys. Chem. Ref. Data`, which is NIST
      Standard Reference Data and sits on the avoid row of the provenance
      table above. Every number asserted is therefore one this repository
      ships or computes, pinned as such and labelled as such. The
      world-facing anchor is left open rather than written into a comment
      with no source a gate could check. **PAID, 2026-09-15**, on the
      owner's correction that the line is bulk dependence rather than
      attribution. `colligative_numbers.rs` now carries a reference block
      naming six sources, and the two dropped anchors are made: φ against
      0.9324 and the freezing point against −0.346 °C, each with a band
      argued from the four-figure a_w rounding the file already documents
      rather than from a percentage. The one-molal case beside it is paid
      the same way. **Four of the six sources are the original
      measurements**, not the compilation — Scatchard and Prentiss 1933 for
      the cryoscopy (doi:10.1021/ja01338a003), Scatchard, Hamer and Wood
      1938 for the isopiestic osmotic coefficients and for sucrose
      (doi:10.1021/ja01279a066), Gibbard et al. 1974 for the concentrated
      boiling end (doi:10.1021/je60062a023), Young and Jones 1949 for the
      sugar's liquidus (doi:10.1021/j150474a004) — and Robinson and Stokes
      is cited as the book it is, with edition, publisher, year and table,
      because it is the tabulation the numbers actually travelled through.
      **Where the trail ended is stated in the file rather than implied**:
      the bibliographic records were resolved against Crossref and are
      exact, but the two primary papers are paywalled and no copy of the
      book was opened, so the VALUES are this repository's own prose finally
      given its sources rather than a verified transcription, and the bands
      absorb that.
      One comparison was WITHDRAWN rather than cited. The 108.7 °C this file
      and `HISTORY.md` both quote is, in this file's own words, "a
      measurement of a saturated brine", while the test it was being
      compared against builds 6.000 mol/kg — and a saturated chloride at its
      boiling point is more concentrated than that. Two solutions, not two
      answers for one. The row stays open, with Gibbard et al. named as
      where a real anchor would come from.
- [x] **Is a boil a curated route or a computed one? Measured and decided,
      2026-09-11: it stays `Curated`, and the asymmetry that prompted the
      question is on the other solver.**
      `PhaseRouteEquilibrator` declares `SolverRouteKind::Curated`, which
      was right when sublimation and hydrates were its only customers —
      there the curated record IS the answer. Now that it melts and boils,
      what it produces is arithmetic over a curated parameter, which looks
      like the shape `CombustionEquilibrator` has while that one reports
      `Computed`. Twenty corpus rows moved `computed -> curated` on this
      alone. That used to make `th-017` ("can ethanol boil before water?")
      a finding for having been answered better; since the two are one
      grade when a requirement is checked (2026-09-08) it is not, so the
      question was decided on its merits and measured on its own rather
      than as a rider on someone else's count.

      **The measurement.** Twenty-six of the corpus's five hundred rows
      have a `phase-routes` route that succeeded with at least one event.
      Those twenty-six are the whole population the declaration can move;
      nothing else in the corpus reads it. Split by the transition the
      route actually ran, from each row's `StateChanged.kind`:

      - **23 rows melt, boil, freeze or condense and nothing else**:
        `th-017 th-031 th-032 th-033 th-038 th-039 th-040 th-047 th-050
        th-052 th-053 th-054 th-055 th-056 th-057 th-065 th-066 th-114
        th-115 th-123 th-124 mat-004 mat-013`. Most are combustion rows
        where the fuel boils off as the flame heats the beaker, not
        thermal rows that set out to boil something.
      - **1 row sublimes and nothing else**: `th-026`, dry ice into water.
      - **1 row does both**: `th-097` boils water, condenses the sealed
        headspace's nitrogen and deposits its carbon dioxide.
      - **0 rows hydrate or dehydrate.** The hydrate half of this solver
        has no corpus row at all.
      - **1 row changes no phase**: `mat-025` is graded `curated` on a
        `PolymerHeated` event, because `equilibrate` also calls
        `plastics::settle`. That is a fifth customer nobody had listed.

      The two other rows tagged `sublimation` — `th-027` (iodine) and
      `th-028` (naphthalene) — are already `computed`: `phase-routes`
      reports zero events for both, because `sublimes_at` needs the
      registry to carry a sublimation point and no melting point.

      **So the population the bullet was protecting is one row, and the
      hydrate rows it named do not exist.** That was the stated reason to
      hesitate, and the measurement mostly dissolves it.

      **The counterfactual, exactly.** Declaring `Computed` would move
      twenty-four rows `curated -> computed`: all twenty-six except
      `th-057`, which keeps `curated` because `curated-reactions` also
      succeeded there, and `th-065`, which is already `qualitative` on a
      typed observation. `by_observed` would read `curated 32,
      computed 352` against today's `curated 56, computed 328`. The
      counterfactual is exact rather than estimated because no row in the
      population emits an `Inert` or `InertInSolvent` event, so
      `inert_beside_an_answer` — the one clause in `coverage.rs` where a
      route kind feeds back into an earlier branch — is not engaged for
      any of them.

      **The argument, and why the count does not settle it.** Twenty-four
      against one is not the reason to keep the label; the reason is what
      `kind` is for. The bullet asks whether it records PROVENANCE (which
      road the vessel took) or COMPOSITION (a looked-up record versus
      arithmetic over a looked-up parameter). Three things decide it for
      provenance:

      1. **The composition reading does not terminate.**
         `curated-reactions` computes an extent from a curated
         stoichiometry; `reaction-families` computes products from a
         curated pattern; a gas test compares a reading against a curated
         threshold. Arithmetic over a curated record is what a curated
         solver IS. Move `phase-routes` to `Computed` on that reading and
         there is no principled place to stop, and `Curated` empties out.
         A label that sorts everything into one bucket records nothing —
         and `by_observed` and `baseline.toml` were deliberately kept
         DESCRIPTIVE on 2026-09-08, when the two stopped being separate
         grades, precisely so that a relabelling stays reviewable. The
         composition reading would spend the property that change kept.
      2. **`kind` is declared per SOLVER, not per route.**
         `PhaseRouteEquilibrator` now has five customers — freezing and
         melting, boiling and condensation, sublimation and deposition,
         hydrates, and `plastics::settle`. No single constant
         describes all five under the composition reading. Under the
         provenance reading one constant is exactly right: every number
         the solver runs on is a curated table, in `phase_route.rs`
         (`FUSION_ENTHALPIES`, `VAPORISATION_ENTHALPIES`,
         `SUBLIMATION_ENTHALPIES`), in the registry (`melts_at`,
         `boils_at`, `sublimes_at`, `hydrate_pairs`) or in a curated
         material recipe (`MaterialRole::PolymerHeatResponse`'s
         `softens_above_k` and `chars_above_k`). Saying the melt and the
         sublimation differently
         would need a per-ROUTE kind; that buys nothing now that the two
         are one grade, and it costs the stable per-solver identity the
         drift gate reads.
      3. **The asymmetry the bullet cites is not a declaration.**
         `CombustionEquilibrator` does not declare `Computed`. It has no
         `route_kind` at all and takes the trait default (`solve.rs`,
         `fn route_kind` on `Equilibrator`). The solver is NAMED
         `"curated-combustion"`, it reads a curated `FUELS` table, and
         `phase_route.rs` says of its own latent heats that they are "a
         curated table for the same reason `combustion::FUELS` is one".
         The pair is therefore not one solver labelled strictly and one
         labelled leniently; it is one solver labelled and one never
         labelled. On the provenance rule the mislabelled member of the
         pair is combustion.

      Pinned in `direct_model_routes.rs`, by
      `route_kinds_record_where_the_numbers_came_from`, which asserts both
      halves of the pair together so neither can flip in silence and the
      next reader finds the decision beside the assertion rather than only
      here.
- [x] **`curated-combustion` reported `Computed` because nobody had given
      it a `route_kind`. Declared `Curated`, 2026-09-13, and the eight rows
      the measurement predicted are the eight that moved.** Surfaced by the
      boil question above and measured in the same pass so it did not
      arrive unmeasured: eight corpus rows have a succeeded
      `curated-combustion` route — `th-030 th-048 th-051 th-058 th-059
      bio-008 bio-009 bio-044` — all eight were `computed` by
      `computed-route`, and not one had another succeeded curated route, so
      declaring `SolverRouteKind::Curated` would move exactly those eight
      `computed -> curated` and touch nothing else. It did. The corpus is
      now 320 computed and 64 curated where it was 328 and 56, the drift
      gate reports nothing else, and no chemistry changed for any row: the
      same solver ran, released the same heat and emitted the same events,
      under a label that now says where its numbers came from.

      The provenance rule the bullet above settles on is the whole
      argument: the fuels, their stoichiometries, their autoignition
      temperatures and their heats of combustion are the curated `FUELS`
      and `GAS_AUTOIGNITION` tables in `combustion.rs`, each row carrying
      its own `provenance` string. The arithmetic that shares the oxygen
      out and turns moles into joules no more makes the road computed than
      an extent over a curated stoichiometry makes `curated-reactions`
      computed. What changed is not the reading but the fact of a
      declaration: `Computed` had been an omission wearing a decision's
      clothes, which is why the pair looked inconsistent for as long as it
      did.

      Done as its own commit and its own PR, separately from the boil
      decision, for the reason this bullet's own predecessor gave: a
      relabelling moves corpus rows, and two relabellings in one commit
      make attribution impossible. Pinned in `direct_model_routes.rs` by
      `route_kinds_record_where_the_numbers_came_from`, which asserts both
      halves of the pair together.
- [ ] No tin and no glycerol in the registry at all. Tin at 232 °C is the
      soldering-iron melting point a learner is most likely to have met.
- [ ] The latent heats live in `phase_route.rs` as curated Rust tables
      rather than in the registry, which is where the temperatures they
      pair with live. `kerotakis_data::schema::PhaseProperty` already
      declares `EnthalpyOfFusion` and `EnthalpyOfVaporisation`, both
      dimension-checked and both unused, so the schema is not what is
      stopping it — only the build script, the runtime loader, the export
      crate and their fidelity tests. Worth doing now that the claim is
      twenty-five rows rather than two. (`loader_fidelity.rs` compares
      eighteen fields and silently omits `transitions` and
      `aqueous_solubility_g_per_100_ml_at_100c`; fix that in the same
      pass, or the new fields will be unpinned the same way.)

#### P3e — Redox and electrochemistry  ← the biggest missing curriculum block

**Settled by experiment, 2026-08-19.** The question "how do we solve
`MnO₄⁻ + Fe²⁺ → ?`" has an answer that needs no reaction rules at all:
*ask the thermodynamics*. Probing IPhreeqc directly:

```text
1 mmol MnO4- into 5 mmol Fe2+ at pH 1
  pe   = 18.345    "Adjusted to redox equilibrium"
  Mn+2 = 9.999e-4     all the manganese reduced
  Fe+3 = 4.827e-3 (+1.63e-4 as FeOH+2)   all the iron oxidised

the same, with half the oxidant
  pe   = 12.674
  Mn+2 = 5.000e-4
  Fe+3 = 2.418e-3     exactly half the iron oxidised
  Fe+2 = 2.500e-3     exactly half left
```

The 5:1 stoichiometry is not encoded anywhere — it falls out of free-energy
minimisation. And the half-oxidised case lands at pe 12.67, which is the
Fe³⁺/Fe²⁺ standard potential: **a half-titrated redox couple sits at its
own E°, exactly as a half-neutralised acid sits at its pKa.** That is one
of the best results this engine can produce, and it is computed.

Two mechanics matter, both learned the hard way:

- **Naming a valence state in `SOLUTION` decouples that element.** Entering
  `Mn(7) 1e-3` and `Fe(2) 5e-3` gives a solution where *nothing reacts*:
  PHREEQC holds each valence in its own mass balance and reports pe 4 with
  the permanganate untouched. Redox needs the elements *coupled*.
- **Adding a reagent by formula is the idiom that works**, and it is what
  the bench's `add` already means: `REACTION … KMnO4` supplies K, Mn and O
  together, and PHREEQC finds the pe satisfying the electron balance.

So the build order is:

- [x] Surface pe and Eh; report the redox distribution per element; couple
      the elements through `redox_coupling` + `solve_coupled` (pe bisected
      on the electron balance); Nernst over computed activities, the
      activity series, displacement, the galvanic `cell` and the hydrogen
      overpotential (`displacement.rs`, 2026-08-20). Details and the
      boundaries each one established are in `HISTORY.md`.
- [x] Faraday's law for electrolysis: charge → moles → mass at an electrode
      (`displacement.rs`, `Operator::Electrolyse` → `Event::Electrolysed`).
      `n = Q/(z·F)` with every term read rather than assumed: Q from the
      ammeter and the clock, z from the couple the vessel actually holds.
      **Current efficiency travels with the number**, because Faraday's law
      is silent about it and a bench that prints only a mass has claimed
      100% without saying so. One loss is computed exactly — when the
      plating ion is exhausted the rest of the charge reduces water, so the
      deposit stops and hydrogen starts — and the remaining losses, which
      need electrode area and exchange current densities, are declared as a
      one-sided bound: a real electrode weighs the reported mass or less,
      never more. The electron ledger (charge in = Σ cathode product × z =
      Σ anode product × z) is a test, not a paragraph.

**Oxidation-state bookkeeping is the explanation layer, not the solver.**
It does not find the products — the free-energy minimisation does — but it
turns a computed distribution into the sentence a learner needs: manganese
fell from +7 to +2, five electrons each; iron rose from +2 to +3. It also
supplies half-reaction display and the electron count. Where the numbers
come from:

1. **Derived from the databases first.** PHREEQC's master-species block
   names oxidation states outright — `Mn(7) MnO4-`, `Fe(+3) Fe+3`,
   `N(-3) NH4+`, `C(-4) CH4` — so for aqueous species the state is *data
   with provenance*, not our opinion. `dbindex` already parses this block.
2. **Rules where the database is silent**, as a constraint solve rather
   than a lookup: F = −1, group 1 = +1, group 2 = +2, H = +1, O = −2, and
   the sum over a species equals its charge. Fix what is known, solve for
   the rest. One unknown element is then determined — possibly as a
   *fraction*, which is honest: Fe₃O₄ really is +8/3 on average, and an
   average is exactly what electron counting needs.
3. **Inconsistency is the detector, not a special case.** Applying O = −2
   to H₂O₂ gives a sum of −2 against a neutral species, and that failure
   *is* the signal that the oxygen is peroxidic. Metal hydrides announce
   themselves the same way. So the exceptions are found rather than listed.
4. **Refuse when two elements remain unknown** and no database entry
   settles it — the same discipline as the balancer refusing an
   under-determined skeleton.

What oxidation states will **not** do, stated so it is not promised: they
do not resolve a skeleton with two products of the same element in
different states. `C + O₂ → CO + CO₂` stays ambiguous however carefully the
electrons are counted, because both products are oxidations and nothing in
the bookkeeping picks a ratio. That needs either a stated ratio or, better,
the same treatment as above — let the thermodynamics decide.

#### P3k — Rates, the cheap sliver  ← **built**

`kinetics.rs`, time as a shared state dimension, catalysis as a lower Ea,
`codex/rates.toml` and the pe/Eh surfacing are done; see `HISTORY.md`.

**Integration accuracy is checked against a closed form, not against
another guess.** A first-order decay has an exact solution, so the
integrator is compared with real arithmetic. That test earned its place
immediately: it caught a 0.7% drift over two minutes that nothing else
could see, which forced the switch from explicit Euler to the midpoint
method. `tools/kinetics-oracle.py` is the second opinion for cases with no
closed form — SciPy, out of the build graph, on the ChemicalFun terms.

Four bugs that only a rate model could have exposed, all fixed:

- **Neither curated rate law conserved mass**, and the conservation
  proptest could not see it because it never issues a `wait`. Peroxide
  destroyed the water its own equation string promised; thiosulfate
  destroyed Na₂O, 62 g/mol per extent, visible on the balance. A rate law
  is a reaction and has to balance like one — there is now a test that
  audits every entry in the registry, and a second that checks the declared
  equation string against the modelled stoichiometry, because when those
  two disagree it is usually the code that is lying.

- **One unknown species disabled the whole aqueous engine.** `partition`
  returned `None` if *any* species lacked a derived role, so adding
  thiosulfate — which is in no database we ship — silently withdrew the pH
  of the acid sitting beside it. The vessel simply stopped having a
  solution.
- **And then the solver deleted it.** The aqueous rebuild replaces the
  vessel's contents with the computed state, so a species with no role was
  not ignored but *destroyed*. It had been invisible only because the
  first bug meant the solver never ran on such a vessel at all.
- **The integrator could freeze.** The midpoint method evaluates on a copy;
  once a half-step pushed the copy's last reactant below the threshold at
  which `withdraw` discards a spent portion, the midpoint rate came back
  zero and the full step applied nothing — two million substeps to advance
  the clock by seven milliseconds.

**Prose numbers are now checked too, advisorily.** `codex lint` caught
nothing when the peroxide rate constant was recalibrated, while five
entries went on quoting half-lives and extents the engine no longer
produced: ranges were verified, sentences were not, and a sentence is what
the learner reads. The lint now pulls unit-carrying numbers back *out* of
the register text — handling `4.06 × 10⁻⁷` as well as `4.06e-7`, since the
first is what the codex actually uses — and asks whether the replay
produced anything like them, at the precision the author wrote. "Near
pH 1.6" is a correct report of a computed 1.64 and is not flagged; a
percentage tolerance would have called it stale.

It is **advisory and stays advisory**, because a good entry legitimately
quotes numbers this replay did not produce: an activation energy from the
literature, a stoichiometric coefficient, a textbook figure held up for
contrast (bicarbonate's famous pH 8.3 exists in the codex precisely to be
contradicted), or another entry's result quoted for comparison. Making
those errors would train authors to strip real content out of their
writing. Currently **43 flagged across 80 entries**, most of them of
exactly those kinds.

What it is genuinely good at is the job it was built for: after changing a
curated constant, the *new* entries in that list are the sentences that
went stale.

**Honest gaps.** A salt the aqueous engine cannot speciate now dissolves
rather than sitting at the bottom of the beaker (`dissolves_without_
speciation`), and the lab says exactly what that means: it contributes
nothing to pH or ionic strength. The thiosulfate reaction treats acid as a
rate influence read from the computed pH rather than as a consumed
reactant, because the practical runs with acid in large excess and the
vessel has no proton portion to draw down.

It also fills a hole the codex already admitted: `collision-theory`
`embodied_by` nothing, and the 500 K thermal stand-down is a placeholder
for exactly this.

#### P3p — Phase behaviour (was P3)

- [ ] `feos` integration (SAFT family + flash); `vle-thermo` for cubics +
      classical activity models; `seuif97` for water
- [ ] Own UNIFAC (~300 lines) against original-literature parameter tables with
      per-parameter provenance
- [ ] Golden fixtures generated by build-time `thermo` (Python oracle)
- [ ] Acceptance: ethanol–water azeotrope at 95.6%

### Curriculum sources — verified 2026-08-19

Four parallel surveys, every licence checked against the issuing body's own
page. The headline is that **the obvious sources moved against us this
year**, and the workable path is statutory rather than licensed.

**Relicensed to NonCommercial in 2026 — no longer usable:** OpenStax
Chemistry 2e and Atoms First 2e (March 2026; the pre-relicense commit
`51f80f1a` stays CC BY 4.0 *irrevocably* and is the escape hatch), PhET
(2026-03-29, 119/119 sims; pre-cutoff pinned builds remain CC BY, and
PhET's *teacher activities* are still CC BY 4.0), OpenSciEd High School —
which is where every chemistry unit lives.

**Never usable:** CK-12 (proprietary, revocable, claims rights in *your*
derivatives), MIT OCW, Khan Academy, ChemCollective (NC **and** ND),
LibreTexts except per-page tag-filtered, New Zealand's entire Ministry
estate (BY-NC), NSW NESA (all rights reserved), BC/Ontario/Alberta,
Singapore (and Cambridge co-owns the syllabuses), Ireland (no licence at
all), Switzerland's Lehrplan 21 (**not** CC — "keinerlei Rechte
übertragen"; note the constraint is *legal, not technical*: a maintained
JSON API and a ~20 MB XML export exist, but both sit behind a signed
Nutzungsvereinbarung with the D-EDK, and the only credential-free routes
are per-competence PDFs. The cantonal adoptions are arguably amtliche
Erlasse under Art. 5 URG, which is the only path worth pursuing),
European Schools, DDC (NC+ND, and dewey.info is dead).

**NGSS is worse than NC:** not NC, but the grant is *enumerated* to states,
districts, schools, teachers and non-profits — a commercial app is not on
the list, no CC licence exists, and ownership is genuinely unclear (site
says NAP, book says Achieve 2013, Achieve has wound down).

**Genuinely usable, ranked:**

| Source | Basis | Why |
|---|---|---|
| **Norway (Udir Grep API)** | `åndsverkloven §14` — public domain | Best in Europe: læreplaner *are* forskrifter, stable codes, explicit year bands, open unauthenticated API |
| **Austria (RIS OGD API)** | `UrhG §7 freie Werke` | Lehrpläne are Verordnungen in BGBl. II; **the cleanest German-language curriculum anywhere**, and no §5(2)-style Änderungsverbot. Documents: Mittelschule `Gesetzesnummer 20007850`/Anlage 1 (idF BGBl. II 178/2025), AHS `10008568`/Anlage A (idF BGBl. II 204/2024, Unterstufe and Oberstufe share one Anlage). API gotchas: `Titel` wants the plural official Kurztitel, `Fassung.FassungVom` rejects the literal `"Heute"`, and `GeltendeFassung.wxe` pages time out — use the API. First year-mapping fact banked: **Chemie is Schulstufe 8 only** (not 7–8), with a wirtschaftskundliches-Realgymnasium exception; Oberstufe runs Kompetenzmodul 5–8 |
| **Sweden (Skolverket API)** | `URL 1960:729 §9` | Live, unauthenticated; kursplaner are författningar |
| **England (DfE)** | Open Government Licence v3.0 | Richest chemistry content of any open source |
| **Australia (ACARA v9)** | CC BY 4.0 | Excellent RDF/JSON-LD/SPARQL — but §5 mandates a fixed citation *including* an app-specific offline-cache clause, and teacher-support resources are carved out NC |
| **Scotland** | OGL v3.0 | Take the Benchmarks, not just the E&Os |
| **France (Éduscol)** | **Licence Ouverte 2.0** | Switched *from* CC BY-NC around Dec 2025 — snapshot the terms page, it has moved once. Take from éduscol, **not** education.gouv.fr, which is still NC |
| **Netherlands** | `Auteurswet art. 11` | Take the examenprogramma from wetten.overheid.nl (no conditions), not Examenblad (ND) |
| **Serlo / ZUM** | CC BY-SA | German OER; **filter per item** — Serlo's licence registry includes CC BY-ND and six state-ministry exam licences |

**Germany: Bayern first, and never the KMK.**

Verified across five states. **Bayern is the only one that grants commercial
use in writing**: the ISB's Nutzungsbedingungen state *"Die Texte der
Lehrpläne unterliegen nicht dem Urheberrechtsschutz"* and expressly carve
the Lehrplan Originaltexte **out** of the site's NC clause (which governs
"alle weiteren Inhalte" — Servicematerialien, images). It also has the best
machine-readability: stable URLs (`/fachlehrplan/gymnasium/9/chemie/ch`)
and a working unauthenticated PDF export. Attribute the ISB; take text
only, never the figures.

Then **Baden-Württemberg** (published as Amtsblatt K.u.U. Ausgabe C; its
only copyright note is inside the PDF and restricts reproducing *"des
Satzes beziehungsweise der Satzordnung"* — the typesetting, not the text,
so extracting competency statements as data and re-rendering them is
outside it; best semantic IDs of the five, and verified: there is **no**
XML/ZIP export, six `requestMode` values all fall back to PDF).
**NRW and Niedersachsen** rest on the §5 argument alone — the underlying
facts are verified (NRW: RdErl. 23.06.2019 in Amtsblatt 07-08/19 under §29
SchulG), but neither state says so in writing. **Berlin-Brandenburg** is
the only state with a real CC licence (Sek II RLP is **CC BY-ND 4.0** — not
NC, so commercial use is fine, but ND plausibly forbids restructuring into
an app data model, which makes it *less* useful than Bayern's
public-domain status); its RLP 1–10 carries no licence at all.

🚫 **The KMK Bildungsstandards are the one hard NC blocker** — their terms
prohibit *"Einspeicherung, Verarbeitung bzw. Wiedergabe von Inhalten in
Datenbanken"*, which is precisely what this app does. Source every
competency statement from a **state** plan, never from a KMK document.

And don't wait for open data: **GovData holds zero curriculum datasets**,
and no Bundesland uses Datenlizenz Deutschland (all sixteen checked).

Operationally: `WebFetch` is domain-blocked for lehrplanplus.bayern.de,
schulentwicklung.nrw.de and bildungsplaene-bw.de, but `curl` works; and
NRW's site has moved to lehrplannavigator.nrw.de.

**Go to the Länder, not the KMK.**

The decisive finding is not legal but structural: **the KMK Bildungsstandards
contain no year-by-year sequencing at all.** In the 2024 Chemie MSA document
"Jahrgangsstufe" appears twice, both in the historical introduction; the
content is organised by Kompetenzbereiche and Basiskonzepte, and sequencing
is explicitly delegated downward ("landesspezifische Ergänzungen und
Präzisierungen können vorgenommen werden"). **The year axis exists only in
the 16 Länder Lehrpläne** — which is also where the §5(1) case is
*stronger*, since e.g. NRW's Kernlehrpläne are set by Runderlass under §29
SchulG and published in the Amtsblatt.

Legally: §5(1) UrhG puts Verordnungen and Erlasse outside copyright
entirely; §5(2) covers other official works but drags in an Änderungsverbot
that is functionally ND, and the courts read Abs. 2 *narrowly*. Crucially
**§5 draws no commercial/non-commercial distinction** — NC is simply not a
feature of it. kmk.org's own Impressum does carry an NC clause that names
database ingestion specifically, but it is self-limiting (it reserves only
what the UrhG does not already permit) and an Impressum cannot re-copyright
a §5 work.

Either way the *facts* are free: "Säure-Base-Reaktionen are taught in
Jahrgangsstufe 9" is a fact, not an authored expression, and *Football
Dataco* (C-604/10) confirms a constraint-dictated sequence attracts no
copyright. Two constraints shape how we take them:

- **§4 UrhG** protects a distinctive *Auswahl oder Anordnung* even when
  every entry is reworded. So we normalise into **our own** taxonomy —
  which the CC0 spine already does — rather than mirroring any document's
  architecture.
- **§87a Datenbankherstellerrecht** protects investment in *obtaining*
  data, not in creating it (the spin-off doctrine), so a ministry authoring
  its own curriculum has a weak claim — but §87b catches "wiederholte und
  systematische" extraction, which means **harvest from official ministry
  sites, never from aggregators**.

Caveat carried openly: **no court decision or commentary addresses Lehrpläne
under §5**, and the widely-repeated claim that curricula may be used
commercially traces to no primary source. This needs a German IP lawyer's
sign-off before launch, not a confident reading of statute.

**The topic spine, and it is CC0.** `oehTopics.ttl` from
`openeduhub/oeh-metadata-vocabs` — the WirLernenOnline curriculum-topic
taxonomy, CC0 1.0 verified, German, hierarchical, with definitions.
Extracted by `tools/extract-oeh-topics.py` into `codex/concepts.toml`:
**189 chemistry topics**. It carries no year mapping, which is the honest
division of labour — topics from a CC0 vocabulary, years from Norway and
Austria, and the mapping between them is ours.

Rejected as spines after checking: EuroVoc (clean licence, ~30 chemistry
concepts, three orders of magnitude too coarse), DBpedia (real granularity
but 3.5-year-stale dumps, self-contradicting licence metadata, and depth-3
is mostly Nobel laureates and trade associations), UDC Summary (CC BY-SA
but inorganic has 2 subclasses and organic 1), ChEBI and Wikidata (entity
ontologies, not teaching topics), IEEE LOM (paywalled).

### P4 — Codex + curated reaction library + appearance

Codex schema and content, `codex/models.toml`, the 189-concept curriculum
spine, `kero codex lint` and `kero serve --mcp` are done; see `HISTORY.md`.
Open:

- [ ] **TOML schema served through `taplo`** (MIT, verified 2026-08-19):
      "a chemistry editor must write it without a build step" gets teeth —
      a JSON Schema in the taplo language server puts red squiggles in the
      editor *before* lint runs
- [ ] Indigo template application over homologues; RDKit as build-time
      cross-validator; our SMARTS incompatibility rules
- [ ] Colour data: species/precipitate/flame colours, indicator ε(λ) sets;
      Beer–Lambert + CIE integration in `kerotakis-appearance`
- [ ] ORD decision (in or out) **before** the first record is ingested
- [ ] Chemistry-editorial hire

### P5 — Real mechanism kinetics

School-level rates and electrochemistry moved forward to P3k/P3e; what is
left here is the part that genuinely needs an engine.

The `codex-kin`, `codex-AQ` and `codex-DATA` sessions of 2026-08-21 delivered
KIN-001…012, AQ-004…014 and DATA-001/002 — the reaction-network IR, adaptive
BDF integration, the Cantera-YAML front end, surface complexation, cation
exchange, solid solutions, the cell chain and the seed registry. Each is
recorded in `HISTORY.md` with its branch and CI run. Open:

- [x] Cantera-YAML mechanism parser (Arrhenius + three-body + Troe covers
      GRI-Mech-class) + rate evaluator feeding diffsol — the parser and the
      evaluator have existed since BRD-040 and the shipped packs in
      `data/mechanisms/` are Cantera YAML; **#510** (merged 2026-09-07) closed
      the last gap, the *reversible* three-body and falloff forms that are the
      dominant shape in every published file. See `HISTORY.md`
- [ ] Multi-step mechanisms, rate-determining steps, steady-state
      approximations — the university-level treatment that curated
      Arrhenius parameters cannot reach

### P6 — Build-time QM enrichment

- [ ] `tools/` pipeline batching xtb + PySCF over the codex: ΔG, Fukui
      (`--vfukui`), MOs (`--molden` / cubegen), path frames (`--path`),
      frequencies → IR spectra
- [ ] Marching cubes → quantised glTF orbital meshes (adopt `mcubes` or vendor)
- [ ] Supervised TS searches only where a barrier genuinely matters
- [ ] Output is data; no QM binary or library ships

### P7 — Lessons + runtime Hückel

- [ ] Declarative scenario format over the operator log; register narration
      hooks; nigredo → rubedo tiers as concept-graph cuts
- [ ] Every lesson replays in CI; lesson states feed the pre-warmed cache
- [ ] `kerotakis-huckel`: simple/extended Hückel for user-drawn molecules
      (own eigensolver or YAeHMOP port), always labelled approximate

### ML tier — last or never

USPTO-trained models are weakest exactly where our users are. If the tier
ever ships: **T5Chem** is the reference forward-predictor (MIT code,
**CC-BY-4.0 weights on Zenodo**, USPTO-trained on the verified-CC0 Lowe
chain, deps modernised Dec 2024) — superseding the Molecular Transformer
plan; runtime via `tract`/`candle`/`burn` (pure Rust, real wasm stories),
not `ort`. Optional download, never web-bundled, confidence always surfaced.

Evaluated and set aside (2026-08-19, licenses verified): **ReactionT5v2** —
capable and popular, but its MIT tag over ORD-trained weights is the
author's unilateral claim with the CC-BY-SA ShareAlike question simply
unaddressed (data pulled from a Drive mirror, no licence notice); not while
that stands. **mhn-react** — clean USPTO-50k template corpus, but its
LICENSE file is textually defective (mangled BSD-2 granting use but
omitting the redistribution conditions); get author confirmation before
vendoring; frozen 2023. **MolReactGen** — thesis artifact, dead 2024,
inherits mhn-react's data and its ambiguity. **The HF enzyme-interaction
model** — zero downloads, no paper/metrics/config, bare pickle checkpoint:
untouchable, and biochemistry is parked regardless.

### Performance pass — cross-cutting, gated on measurement

A full-workspace survey (2026-08-23) found the hot paths concentrated in
two places — the redox bisection around uncached engine calls in
`kerotakis-phreeqc` (worst case ~272 full PHREEQC solves per vessel
equilibration, each one a wasm↔JS round trip in the browser) and
per-iteration allocation inside the CEA Newton loop — plus free wins
nobody has claimed (no `[profile.release]`, no `wasm-opt` pass, a
`const` species table inlined into five crates). The tasks are scoped,
ordered and agent-executable in **[OPTIMIZATION.md](OPTIMIZATION.md)**,
under one rule: benchmarks land first, and no optimization merges
without a before/after number recorded next to it. Chemistry output is
the contract throughout — the one task allowed to move a converged pe
in its last digits (OPT-7) says so explicitly and must still pass every
existing test unchanged.

### Capability parity — measured against the neighbours

A capability inventory (2026-08-23) compared the shipped product
surface against ChemPy (the open-source reference for exposed
relations, properties and balancing) and against the commercial
PHREEQC-workbench class (parameter studies, Monte Carlo, predominance
diagrams, fitting). Verdict: our solver depth already exceeds both in
places, but whole finished capabilities are unreachable — the VLE
crate has no dependent, the `USER_GRAPH` plumbing feeds no chart, and
there is no user-facing sweep, diagram, or uncertainty anywhere. The
gaps worth closing are scoped as CAP-1…CAP-14 in
**[CAPABILITIES.md](CAPABILITIES.md)**, which also records what is
deliberately declined as off-mission and points at the R-stages that
already own the rest.

---

## Open follow-ups from the 2026-09-05…07 sessions

### Scoped tasks, 2026-09-15

Each is written to be picked up without re-deriving its context. None is
started. They are ordered by what they unblock, not by size.

- [ ] **Triage the records that name no audited source.** 105 of 185 registry
      citations name nothing `kero provenance upstreams` recognises, which is a
      larger population than the refused-source findings it does count. Nobody
      knows which kind they are. Sort them into three: genuinely unattributable
      because the engine derived the value, merely spelled in a way the `names`
      lists do not match, and real gaps. **Deliverable: the three counts and the
      method, as a report.** Do not fix anything. The decision of 2026-09-14 was
      to count and report these and to bind the requirement to new records only,
      so a sweep would reverse a decision rather than serve it. This is the
      measurement that says whether a sweep is ever worth buying.
- [x] **Give numeric records an uncertainty, and mark which are measured.
      DONE 2026-09-15**, on `feat/registry-uncertainty`, reported in
      **[docs/registry-uncertainty-and-measurement.md](docs/registry-uncertainty-and-measurement.md)**.
      66 molar masses gained a propagated CIAAW interval, 1 record is marked
      `measured`, and NO VALUE WAS CHANGED.
      **`measured` was empty because nobody filled it, and more precisely
      because the field was filled with the answer to a different question**:
      `method` was read as "how the value entered the registry", so 852 records
      carry `Imported("verbatim export from ... REGISTRY")`, which describes
      the export step. No export measures anything, so under that reading no
      record could ever be `measured`. It now records how the value CAME TO
      EXIST, and `measured` means the CITED SOURCE is the experiment — never
      this project, which operates no laboratory.
      **`not_reported` was doing two jobs and could do neither.** It was
      documented as "the source did not report one" and was simultaneously the
      blanket default on 1093 records whose sources nobody had opened. A new
      `unestablished` carries the absence and is the default; `not_reported`
      is now a finding, reached by reading a source, and the registry has
      bought none — which a test asserts, so the first one is bought
      deliberately.
      **A derived quantity does inherit, but the propagation rule depends on
      what the input band is.** CIAAW publishes several elements as an
      INTERVAL over natural isotopic variation rather than a value with an
      error bar, so a molar mass inherits an interval by interval arithmetic
      and NOT in quadrature; the band is widened by one electron mass per unit
      of charge because this registry stores the formula sum so a dissociation
      closes its mass balance exactly.
      **The propagation is a check as well as a claim** — the validator already
      refuses an interval that excludes its own value — and it found five molar
      masses that fail: `H2O2`, `O2` and `Na+` are quoted more coarsely than
      their derivations support, and `catalase` and `amylase` carry placeholder
      formulas. All five are left as they stand and named in a declined table.
      **The pass found something the band itself did not: the bench does not
      read the record the band is attached to.** The colligative path never
      asks the registry for water's molar mass; it carries its own copy as a
      Rust literal in thirteen places across seven files and in three
      spellings, and `constants.rs`'s 18.01528 disagrees with `states.rs`'s
      0.018015 at 1.6 ppm. Both lie INSIDE the propagated interval, which is
      what makes the disagreement legible as two representatives of one
      published range rather than as a typo. Written up as its own follow-up
      below.
      **Reach and cost:** five element rows reach 66 of 186 molar masses;
      twenty-one more rows reach the rest, an afternoon plus however many more
      coarsely-quoted values they turn up. Beyond molar mass there is no table:
      roughly 228 records are declared non-claims that should stay non-claims,
      and the ~120 with no checkable source cannot be given a band at all until
      they are re-sourced. **The uncertainty programme is blocked behind the
      sourcing programme there**, and `boiling-point/water` is the clearest
      case — its citation was withdrawn on 2026-09-13, so there is no source to
      read.
- [x] **The two water enthalpies have no registry record, and they are the only
      colligative inputs big enough to move a band.** Found 2026-09-15 in the
      uncertainty pass, closed the same day in #610. Both now have records:
      `enthalpy-of-fusion/water` and `enthalpy-of-vaporisation/water`, and
      `build.rs` generates `WATER_H_FUS` and `WATER_H_VAP` out of them, so the
      constants in `states.rs` ARE the records rather than copies beside them.
      **Fusion was the cheap half and stayed cheap**: it cites the already
      vendored `vendor/nasa-cea/thermo.inp` and carries the derivation
      `H(H2O(L), 273.15) - H(H2O(cr), 273.15)` as its method, which
      `kerotakis-cea` re-checks on every run. Its uncertainty is
      `unestablished` and NOT `not_reported`, deliberately: thermo.inp does not
      print an enthalpy of fusion at all, so there is no quantity in it that
      could have quoted a band.
      **Vaporisation was the expensive half and it was bought.** It had NO
      SOURCE AT ALL. It now cites N. S. Osborne, H. F. Stimson and D. C.
      Ginnings, *Measurements of heat capacity and heat of vaporization of
      water in the range 0° to 100° C*, J. Res. NBS **23** (1939) 197–260,
      RP1228, doi:10.6028/jres.023.008 — primary journal
      literature with a DOI, and a public-domain NBS Technical Series work,
      read on 2026-09-15 from nvlpubs. Table 13 prints
      L(100 °C) = 2256.30 int. J/g; the paper's own 1 int. J = 1.00019 abs. J
      and the registry's own 18.015 g/mol give 40 655 J/mol, which is **40 650
      to the four figures the constant has always carried, so the value did not
      move**. The table is self-checking and the check passes: γ = L + β, and
      the same row prints γ = 2257.71 and β = 1.408.
      **It is also the registry's first `not_reported` record**, which #608 said
      would only be bought with the reading in hand. The authors decline to
      quote an accuracy in terms — "this agreement must not be taken as an
      estimate of the accuracy of the results, since it takes no account of
      unknown systematic errors, which may well be larger than the accidental
      errors" — so the band is still missing and the corpus still cannot spend
      it. **What remains** is the one purchase now worth most to
      `validation/cases/colligative.toml`: a modern evaluation of the steam
      properties that bounds ΔH_vap, since the depression and the elevation
      both go as 1/ΔH and one per cent there is 88 % of the tightest model band
      in the file.
- [x] **Water's molar mass is eleven literals and no reader of the registry.**
      Found 2026-09-15 while giving the registry uncertainties, closed the same
      day in #610. The count was understated: `git grep` over `crates/*/src/`
      finds **24 occurrences**, of which 5 are prose and 1 asserts a third
      party's arithmetic, leaving **18 sites across seven files** that carried
      the bench's own copy. Three of the 18 were engine test fixtures rather
      than solvers.
      **The two spellings were not a typo AND they were not equivalent, which
      the original item got half right.** Both lie inside the CIAAW interval
      [18.01471, 18.01599], so neither is a typo. But 18.01528 is
      2 × 1.00794 + 15.9994 — the **pre-2009** IUPAC standard atomic weights —
      under a comment in `constants.rs` claiming the 2021 ones; the 2021 CIAAW
      conventional values are 1.008 and 15.999, which sum to the registry's
      18.015. So one of them was the current table and one was a superseded one
      wearing the current table's label, and that is decidable rather than a
      matter of taste.
      **The fix is a generated constant, not a runtime lookup.**
      `crates/kerotakis-core/build.rs` emits `WATER_MOLAR_MASS_G_PER_MOL` and
      `WATER_MOLAR_MASS_KG_PER_MOL` from `molar-mass/water` in the pack. That
      buys what a lookup does not: it cannot miss, so the two
      `.map_or(18.01528, |d| d.molar_mass)` fallbacks in `bench.rs` and
      `scene.rs` are deleted rather than repaired; it stays usable in a `const`
      context; and the build fails by name if the record is ever dropped
      instead of a solver quietly substituting a different number.
      **Nothing moved, and the width predicted that.** No golden, no lesson
      transcript and no corpus row changed: every site that computed with the
      number already used 18.015, and the only two occurrences of 18.01528 that
      could have reached an answer were behind a lookup that cannot miss. The
      corpus's `wired` field now reads `true` on all three molar masses, and
      `crates/kerotakis-core/tests/one_value.rs` refuses a new literal under
      any crate's `src/`.
- [ ] **Two more constants restate a registry record, and one of them raises a
      question this pass declined to answer quietly.** Found 2026-09-15 while
      closing the molar mass, which is what makes that finding a PATTERN rather
      than a case. A third instance, `LIQUID_WATER_HEAT_CAPACITY = 75.3` under
      the comment "the registry's own figure, restated here", was closed in
      #610 because it was unambiguous: a substance property, an existing record
      at the same value, and no caller at all. These two are not.
      `states::WATER_FREEZING_K = 273.15` and `WATER_BOILING_K = 373.15` are
      Rust literals beside `melting-point/water` and `boiling-point/water`,
      which carry the same two numbers. **Wiring them is NOT obviously right,
      and `validation/cases/colligative.toml` says why in its own
      `boiling-point/water` row**: 373.15 K is 100 °C *exactly*, which is a
      definition of the pre-ITS-90 temperature scale rather than a measurement
      of water, and the two stopped coinciding when ITS-90 replaced it. So the
      question is whether these constants are the model's reference points —
      in which case they are definitions and should stay literals, with a
      comment saying so — or the substance's transition temperatures, in which
      case they are the registry's and the registry's record is the one that
      needs re-sourcing first. **Whoever answers it should answer the same
      question for `melting-point/water`'s own record**, whose citation was
      withdrawn on 2026-09-13 and which therefore has no source to check the
      value against either way.
- [ ] **66 shipped uncertainty bands rest on `ciaaw`, whose verdict is
      `decision-required` — and they narrow the argument that row stands on.**
      Written 2026-09-15, revised the same day after #607 landed the row this
      item was originally opened to ask for. THE ROW EXISTS NOW AND IS BETTER
      THAN THIS ITEM WAS: it records a real grant for educational use that
      stops short of commercial use, and three open questions nobody has
      asked. Nothing here disputes it. What it needs is one correction of
      fact, because the uncertainty pass landed the same day and changed the
      thing the row's own reasoning leans on.
      That reasoning says the question may not arise at all, "since a standard
      atomic weight is an evaluated MEASUREMENT and this registry ships
      compound molar masses COMPUTED by stoichiometry from about twenty
      element values rather than a copy of any table". **That was exactly true
      before 2026-09-15 and is slightly less true after it.** The uncertainty
      pass ships two things that are not computations: five CIAAW element
      intervals written verbatim into `ATOMIC_WEIGHT_INTERVALS` in
      `crates/kerotakis-registry-export/src/lib.rs`, and four registry records
      — `C`, `graphite`, `diamond`, `activated_charcoal` — whose band IS
      carbon's published interval `[12.0096, 12.0116]` unchanged, because a
      single-atom formula's propagation is the identity. The other 62 bands
      are genuine propagations over multi-element formulas and sit exactly
      where the row's argument puts them.
      **So the Feist footing is narrower than the row records, by five element
      values and four records, and that is the whole of the correction.** It
      is not a reason to withhold the bands — the registry has cited these
      atomic weights in over a hundred citations for as long as it has
      existed, and the pass deepens a dependence rather than creating one —
      but whoever asks CIAAW's Secretariat the question that row names should
      ask it knowing the answer now covers five reproduced intervals and not
      only derived arithmetic. The cheapest alternative, if the answer comes
      back unfavourable, is small and known: drop the four single-atom bands
      and keep the 62 propagated ones.
- [ ] **Majer & Svoboda has no machine-readable row.** It is cited in shipped
      code and appears in the prose table, but `provenance/upstreams.toml` has
      no entry, so the lint cannot see it. Add one when someone reads its terms;
      do not invent terms for it. **Attempted 2026-09-15 and deliberately not
      added.** There is nothing to read: the 1985 volume is a print book with no
      terms page, Blackwell Scientific no longer exists, and its successor
      returns HTTP 403 to an automated request for both the title page and the
      permissions page. That is a weaker footing than `crc-handbook`, whose
      verdict at least rests on the absence of a grant on a page that loaded.
      The one citation that matters is `phase_route.rs`'s ethanol row, and it
      names the book only to say that the RETIRED figure traced through it — so
      when a row is added it will want a `[[citation]]` row on the Rust surface
      alongside it, `role = "mentioned"`, or the lint will condemn a withdrawal
      notice. That row already exists for the WebBook in the same string and is
      the model: `matching = "Ethanol enthalpy of vaporisation"`. **Next step:
      a human opening the Wiley permissions page in a browser.**
- [ ] **A comparison in the record is wrong, not merely stale.** The 108.7 °C
      quoted for a boiling brine in `colligative_numbers.rs` and in `HISTORY.md`
      is a measurement of a SATURATED brine, while the test builds 6.000 mol/kg
      and a saturated chloride at its boiling point is more concentrated. Two
      solutions, not two answers for one. Found 2026-09-15 and withdrawn from
      the test rather than compared against. `HISTORY.md` keeps its dated entry
      as written, so this wants a new entry recording the correction rather than
      a rewrite.
- [ ] **The Open Reaction Database, as an oracle: decided out for now, with the
      condition for revisiting.** Owner's question, 2026-09-15. Its licence is
      share-alike, which this project deliberately does not bundle, but used as
      a BUILD-TIME ORACLE that conflict does not arise: share-alike binds
      redistribution, not use, and the `oracle-only` verdict already exists for
      exactly this, with two sources in it. **The blocker is fit, not licence.**
      It is synthetic organic chemistry — reagents, solvents, catalysts,
      conditions, yields — and this bench does aqueous speciation,
      thermodynamics, phase change, combustion, school kinetics and
      electrochemistry. An oracle with almost nothing to say about the questions
      asked. Revisit if synthesis and yield enter scope; until then the oracles
      worth widening are the ones already cleared, which speak the same
      chemistry the engine does.

Recorded here so they survive the sessions that found them; each names the PR
that raised it. Nothing below is a commitment to an order.

- **Temperature-dependent Cp, the wiring half (landed #509)** — #507 landed 37
  Cp(T) records over 35 species that no ledger read; #509 moved the ~40
  `Cp·ΔT` call sites onto `enthalpy_between` and took the lesson goldens with
  it. A species with no curve keeps its 298 K constant, deliberately, so "we
  have a curve" and "we do not" stay different states of the data.

  What the integral actually moved, for the planning docs: the biggest lesson
  shift is `boiling-curve`'s third beaker, 240 kJ into 100 mL, from 600.9 °C
  to 470.2 °C (874.06 K → 743.40 K) — a fixed dose buys fewer degrees once the
  price per degree is allowed to climb. Its siblings move the same way and less
  far: 169.0 → 167.2 °C for 60 kJ, 97.0 → 96.8 °C for 30 kJ. Cooling moves the
  other way for the same reason, the liquid leg costing more per kelvin than
  the room-temperature constant so less budget survives the plateau: −71.8 →
  −70.9 °C, −23.5 → −23.2 °C. Mixes land where the enthalpies balance rather
  than where the temperatures average (50.00 → 50.01 °C), a plateau's leftover
  is now spent from the threshold over the contents the vessel has *now*
  (`luminol-temperature`'s hot beaker lands on 373.15 K instead of 9.65 K under
  it, and its cold one no longer freezes past 0 °C and melts back — one event
  fewer), and solubilities follow the shifted temperature (borax 0.0879 →
  0.0878 mol dissolved; sucrose's limit 0.6264 → 0.6271 mol, so 0.2501 → 0.2494
  mol precipitates). Off the goldens, ice chilled by 60 kJ out of 100 mL lands
  at −90.6 °C rather than −77.4: ice's own capacity falls from 2.09 to about
  1.57 J/(g·K) on the way down, and the constant was only the top of the curve.
  The last rectangle in the ledger was in `kerotakis-phreeqc`'s aqueous tail,
  where balancing two flat Cp spans instead of two areas made Hess's law
  order-dependent by 7.25e-5 K.
- **Liquid water's Cp fit is ill-conditioned for differencing** (#509) — its
  NASA-9 record is a narrow fit, 273.15 to 600 K, carrying a 1/T² term, and the
  antiderivative the ledger differences is a sum of terms of order 1.2e9 J/mol
  cancelling to −9.2e8. A double gives up about 2.6e-7 J per mole of liquid
  water per difference taken, which is why two energy round trips in #509 close
  to 1.9e-9 J and 9.9e-7 J rather than to machine epsilon, and why their bounds
  are a hundred-thousandth of a joule with the measurement written beside them.
  Ice's fit gives up 4e-11 J/mol and nitrogen's 4e-12, so this is one curve's
  conditioning and not the arithmetic. `t.ln()` is libm rather than correctly
  rounded, so the floor itself moves about one unit between platforms.
  Evaluating each interval's integral in `(T − T_mid)` rather than about zero
  would buy most of it back; it moves every golden in the last digits, so it
  wants its own change. Nothing observable depends on it — the worse of the two
  residues is 3e-9 K — but any future test that asks the ledger for an exact
  joule will meet this floor and should be told why.
- **`Vessel::heat_capacity` room-temperature residual (closed #509)** — the
  burner no longer pays room-temperature prices for a crucible at kiln
  temperature. #509 also named the term the two-line ledger never
  had — the sensible heat the CO₂ carries out — which is what takes the #488
  chalk case from 93.6 % to 99.5 %.
- **The 28 codex models are exported and never rendered** (#505) —
  `parseCodexIndex` keeps `doc.reactions` and drops `models` and `concepts`,
  so every model, including every `fails_at`, reaches the browser and is
  thrown away; there is no `Model` type in the shell. The German for them is
  correct and *ready* rather than visible. Worth its own roadmap item.
- **`curriculum[].system` renders as a raw de-hyphenated slug** — "bayern
  lehrplanplus", "england national curriculum", in both languages; four
  missing bundle keys on I18N-2's surface. It belongs with the neighbouring
  design call: `curriculum[].stage` is a *citation* of an English syllabus
  document, and rendering a citation in German would fabricate a section name
  that does not exist (#505).
- **Casein buffering is unmodelled** (#446, #508) — milk's diffusible mineral
  buffer characterises a beaker near pH 6.7 and the three calcium phosphates
  now precipitate, but casein stays unresolved, so a computed yoghurt pH is a
  lower bound and not a prediction.
- **`DidNotIgnite` is done** (#501) — the last animation-audit row the client
  could not close; recorded here only because the audit's "what the engine
  still lacks" list is where a reader will look for it.
- **Open-vessel CO₂ uptake as a rate (landed #531)** — #496 was the peer
  session's PR and is closed; the same work landed as #531.
- **Characterising a solvent-only vessel (closed #543)** (#529, from #504) — a beaker of
  plain water, or of water and a neutral molecular solute, gets no
  `SolutionInfo`: `PhreeqcEquilibrator::partition` declines when nothing with
  a derived role is dissolved. Sugar water has a pH, so this is a hole, and
  #504 tried to close it. Measured on that branch, closing it moved 69
  curiosity-corpus rows, and only 35 of them were the reason-code
  strengthening it looks like from a distance:
  - **26 rows lost a typed observation.** `aq-016…021`, `bio-007…112`,
    `mat-021…084`, `th-101` fell from `qualitative/typed-observation` to
    `computed/computed-route`. `typed_observation` in `coverage.rs` is
    computed from events alone, and neither aside-guard beside it can be
    tripped by a *computed* route — so the smell, gas test or "this does not
    dissolve" that used to be the row's answer stopped being emitted.
    `bio-042` (starch + HCl + heat) and `mat-029` (PET + NaOH + heat) are in
    that list, and the classifier's own comment names them as rows where "the
    polymer is unchanged" *is* the answer.
  - **3 rows became hard solver failures.** `aq-097`, `th-002`, `th-003` all
    cool pure water through freezing; PHREEQC is then asked to solve it and
    fails at the solution phase boundary, and the `SolverFailed` is recorded
    before the independent water-phase fallback gets to answer.
  - `mat-086` fell from computed to missing.

  None of that is reachable from the coverage classifier: it is what the
  engine emits that changed. So the order is (1) make an aqueous solve of a
  solvent-only vessel not suppress the honesty pass's typed observations,
  (2) run the phase transition before the aqueous attempt for an independent
  water inventory so freezing water never reaches a solver that cannot solve
  it, (3) then open the gate and review the remaining rows one at a time.
  PR #543 completed that order. The tests that pin the boundary —
  `native_startup_tests::successful_native_startup_retains_aqueous_computation_and_provenance`
  and `unsupported_ionic::unknown_ionic_feed_is_distinct_from_a_neutral_molecular_solute`
  — assert the boundary the product actually has, and say so in a comment.
- **19 redundant worktrees (re-audited 2026-09-13)** — the triage list is at
  `/mnt/volume1/tmp-overflow/triage-prune-list-20260907.txt`. None was deleted:
  main absorbed that work through re-authored PRs rather than cherry-picks, so
  no branch HEAD is an ancestor of `origin/main` and every branch still differs
  on at least one touched file. All 19 are now clean, inactive and without a
  recent non-build write, but each still has a patch-unique commit, so none was
  deleted. `docs/WORKTREE-AUDIT-20260913.md` records the gate and grouped
  counts; semantic comparison with the named merged PR remains mandatory.

### Scoped tasks, 2026-09-16

Written the same way as the 2026-09-15 block: each is pickupable without
re-deriving its context. Two of the six decisions of this morning are already
out with agents (the `OH-` carrier rename, the fermentation rate scaling) and
are not repeated here. These four are not started.

- [x] **Teach the perturbation generator that a terminal event ends the
      experiment. DONE 2026-09-17.** The prediction written here on
      2026-09-16 was read from source and marked "confirm by running before
      rewriting". It was confirmed by running:

          v1: headspace settled at 6.959 bar with 0.0283 mol gas
          v1: BURST at 696 kPa (glass rating ~405 kPa) — seal gone, gases vented
          ⚠ HAZARD (Danger): sealed vessel over-pressurised and burst
          v1 pressure gauge: 101.33 kPa

      Seven bar against a four-bar rating, not the twelve estimated here —
      because most of the carbon dioxide stays dissolved — but the
      conclusion holds and the engine is innocent. `Pair` now carries
      whether either run ENDED, and the causal rules decline to claim
      anything when one did. `aq-061` is withdrawn from `DOSE_INERT` with
      the run output in its place, and the `#[ignore]`d test is replaced by
      `a_bottle_that_cannot_hold_the_gas_bursts`, which asserts the burst,
      the Danger line, that the vented gas is accounted for, and the
      atmospheric reading afterwards.

      **What the episode is for.** Every observation in the original report
      was correct — the gauge did read atmospheric, the boundary was open,
      doubling the dose changed nothing — and the diagnosis was still
      wrong. An instrument that watches a single scalar will eventually
      mistake a terminal event for an unresponsive one.
- [x] **Run the remaining 70 const-table mutants. DONE 2026-09-17 (#623),
      and the survivors are now zero (#625, #654).** 70 of 70: 36 caught,
      34 survived. The split was the finding — `conductivity.rs` 23 of 28
      survived against `properties.rs` 11 of 42 — and **five conductivity
      constants were watched, which was the whole list**: H⁺, OH⁻, Na⁺, K⁺,
      Cl⁻, the ions in table salt and in the acid and base every lesson
      pours. The λ° table was verified exactly where the lessons happen to
      go. Sourcing the values rather than pinning them took the 23 to 6
      (#625) and then to **0** (#654). Recorded in
      `docs/MUTATION-SENSITIVITY.md` §8b/§8c.
- [x] **Establish whether fermentation happens at all. ANSWERED 2026-09-17
      (#621), and the answer is "for two routes, against a cited
      timescale".** The rates were editorial classroom numbers with no
      source. Two are now fitted: yeast to a wild-type maximum specific
      ethanol rate, **declared an upper bound and not a best estimate**
      because a laboratory strain in a fermenter is not a gram of active
      dry yeast; the yoghurt culture to a cited eight-hours-to-6.1%-lactose
      timescale from two papers read in full. Acetobacter and sourdough
      found no measurement at all and **inherit** the lactic constant,
      marked `editorial` rather than `derived` so a consumer can tell
      fitted from inherited without reading prose.

      **The evidence it was not fitted to pH:** `bio-069` crossed real
      yoghurt's 4.4–4.6 band from below to above (3.889 → 2.835 → 5.436)
      with nothing aiming at it. 5.44 is not a yoghurt's pH either — eight
      counter-top hours sit 18 K below the culture's optimum, which is the
      honest answer to "will this work on the kitchen counter".

      **Still open, and measured rather than estimated:** milk's missing
      casein and phosphate buffering is worth **0.66 of a pH unit**, found
      by feeding one paper's own acid into this milk and reading 3.944
      where they measured 4.6. Half of that fix is a thirty-line change;
      the other half needs numbers behind a publisher's 403.
- [x] **Wire the 44 orphan lessons into the catalogue. DONE 2026-09-17
      (#622).** All 113 `.lab` lessons are now reachable from the catalogue
      — 0 orphans — and a test asserts the invariant so the next lesson
      cannot go missing the same way. The entries were written from each
      lesson's actual transcript rather than its filename, which is what
      caught four that are genuinely `partial`, including
      `starch-iodine-test` printing an inventory where its blue-black
      belonged (#630). The headline went 208 → 252, and GUI-105 (#638) then
      put the 500 answered questions in the same index.

## Open decisions

### Waiting on the owner, 2026-09-17

Three rulings, each blocking nothing else but each answerable only by
whoever owns the product. All three came out of work that is otherwise
finished and merged.

- **The KCl fit target: basis restated, value left alone.** `FIT_SOURCE`
  called 1413 µS/cm "the IUPAC/OIML reference value" for the 0.01 **mol/kg**
  standard. OIML R 56 prints **1408.3** for its 0.01 D primary standard and
  1413 appears nowhere in it; USGS WSP 2311 via Jones & Bradshaw gives
  1408.07 (0.01 D) and 1410.75 (0.01 N). 1413 is the **0.0100 mol/L**
  figure — a real standard on a *volumetric* basis, credited here to a
  molality basis and to a body that publishes neither. **Fixed 2026-09-18**
  (owner's decision): the basis is stated correctly and the body is no
  longer credited; the VALUE is unchanged, because it is a real standard
  correctly used as a fit target. **CLOSED 2026-09-18** (owner's ruling):
  re-target on 1408.3. The word "re-target" overstates what it turned out
  to be — **no coefficient moved and none needed to.** `FIT_SQRT` and
  `FIT_LINEAR` are untouched, and the calibration ladder in
  `conductivity_sources.rs` has validated the fit against OIML's 1408.3 row
  on the correct grams-per-kg-of-solution basis since 2026-09-17. What was
  actually wrong was a **basis mismatch inside one unit test**:
  `kcl_calibration_standard_within_model_error` builds a 0.01 **mol/kgw**
  solution and asserted against the 0.0100 **mol/L** figure. It now names
  OIML's 1408.3, the model reads 1423.0 (1.04% high, where it was 0.71%
  high against 1413), and the `must_overestimate` direction is unchanged
  with more margin. **The 7% window is deliberately not narrowed** — it
  would pass at 2%, and tightening it would hand CI a genuine dilute-end
  model limitation to fail on.

- **Six conductivity constants remain unwatched, and they are unwatched
  because they are unsourced.** Zn²⁺, Fe²⁺, Fe³⁺, Al³⁺, Mn²⁺, Pb²⁺ — the six
  for which no second compilation could be reached. After the sourcing work
  the survivor list stopped being "code no test aims at" and became "data
  nobody has corroborated", which is a different problem and a better one.
  **The decision: buy a second source for the six, mark them
  `Unestablished`, or accept them as they are.** (#625)

  **ANSWERED IN TWO STEPS.** *Buy a second source* was chosen and the search
  ran on 2026-09-18 (#654): two compilations were reached outside the
  transference-number family that stops at magnesium, both too coarse to
  corroborate at the half a per cent this file works to, and **λ°(Fe³⁺) was
  taken OFF the `UNCORROBORATED` list** because both print the number
  already shipped. (Written "left `UNCORROBORATED`" until 2026-09-18,
  which reads as its own opposite.) The
  search found exactly **one outright contradiction of a shipped value**:
  both print ⅓Al³⁺ = 63 where the table shipped 61. **Ruled 2026-09-18 and
  now done: adopt 63 — λ°(Al³⁺) 183.0 → 189.0**, because the shipped 61
  traces to the CRC Handbook that `provenance/upstreams.toml` refuses as a
  systematic source and that no route reached directly, and two reachable
  compilations agreeing beat one refused one that disagrees with both. It is
  a choice between unsourced readings and not a measurement: both print two
  significant figures, and each is measurably coarser (5.6 % on Cu²⁺, 6.8 %
  on CO₃²⁻) than the 3.3 % the value moved. `UNCORROBORATED` is now **five** —
  Zn²⁺, Fe²⁺, Mn²⁺, Pb²⁺, MnO₄⁻ — with manganese and lead **contradicted by
  sources that contradict each other**, which is why neither of those moves.
  Nothing downstream changed: no lesson, golden or corpus row puts aluminium
  in solution.
- **`solution.solvent_kg` is path-dependent because the SOLVER is not
  representation-invariant.** *This replaces an earlier framing of mine that
  was wrong, and that an owner decision was taken on: I reported two sources
  — "the input water when a material add is last, the solver's `mass_H2O`
  when a salt add is last" — and the owner chose "publish the solver's
  `mass_H2O` always". Instrumenting all three sites shows **both values
  already come from `finalize_solution_info`**, so that choice is already
  what happens and does not fix anything.*

  What the probe shows: the same CaCl2 addition yields 0.0997010580 when the
  detergent is already dissolved and 0.0996909357 when it is not. PHREEQC's
  `mass_H2O` is not representation-invariant — `aqueous.rs` says precisely
  that of surface complexation — and the two orders hand the solver the same
  final state in two different representations. `ionic_strength` carries the
  1e-4 into every activity coefficient.

  **RULED 2026-09-18: pose the final state canonically before the last
  solve.** Same contents, same answer, whatever order they arrived in —
  at the cost of one extra solver call per characterisation. Accepting the
  non-invariance and declaring it on the wire was offered and not chosen,
  and so was leaving it recorded. Substituting the inventory figure was
  never available: molalities are per kg of the solver's own `mass_H2O`, so
  it would leave `n = m × kg` false by exactly the discrepancy it repaired.
  The work is scoped under *Ruled by the owner, 2026-09-18* below, including
  the instruction that if the measured cost turns out worse than "one more
  solve" the answer is to stop and report it, not to ship a slower engine
  quietly.

  **RULED AND DONE 2026-09-18** — pose it canonically; see the ticked item
  in the section below for the numbers. The reading above is right about the
  non-invariance and still short of the cause: `mass_H2O` follows the water
  the INPUT declared, and the input to the last operation is the
  *intermediate* vessel plus one reagent. The two orders have different
  intermediate vessels, so the same final contents reach the solver as two
  different questions. The vessel's own water inventory never departed.


### Ruled by the owner, 2026-09-18

Eight decisions taken in one sitting, each put as an interview question with
the rendered output it would change. Every ruling below supersedes whatever
the block above it says is "still open"; where the two disagree, this section
wins, and the older bullet stays only because deleting it would erase why the
question was asked.

- [x] **λ°(Fe³⁺) stays out of `UNCORROBORATED`, caveat in the row. RULED
      2026-09-18; already the shipped state, so nothing changes.** Two
      compilations reached by unrelated routes — Hübschmann & Links (1991)
      p. 62 and Kreshkov (1970) p. 74 — both print 68 per equivalent. Both
      print it to *two* significant figures where #625's standard for the
      other 22 ions was three, so the row cannot distinguish 204.0 from 203.
      The owner's ruling: two independent tables agreeing to the precision
      they print is real corroboration *of that precision*, and the row
      already says which precision that is. `UNCORROBORATED` is six, not
      seven, and the caveat comment in `conductivity_sources.rs` is the
      thing that makes the claim honest rather than the list length.

- [ ] **λ°(Al³⁺): adopt 189 (63 per equivalent). RULED 2026-09-18.** The
      engine ships 183.0 (61 per equivalent) and **both** new independent
      compilations contradict it at 63 — a 3.3% disagreement, the only
      outright contradiction the 2026-09-17 sourcing sweep found. Nothing was
      changed at the time, per the standing instruction to report rather than
      correct. The owner's ruling is to change it: the shipped 61 traces to
      Vanýsek/CRC, which `provenance/upstreams.toml` refuses as a systematic
      source, and two reachable compilations agreeing beats one refused one.
      **What reaches the user:** an aluminium sulfate solution reads
      2.41 → 2.46 mS/cm, +2%.
      **Scope:** `LIMITING_CONDUCTIVITY` 183.0 → 189.0; add the `Al+3`
      corroboration row to `conductivity_sources.rs`; remove `Al+3` from
      `UNCORROBORATED` (six → five); write the *risk* into the row rather
      than hide it — Kreshkov is 5.6% out on Cu²⁺ and Hübschmann 6.8% out on
      CO₃²⁻ against values this repo has already corroborated, so these are
      coarse tables and the reader is owed that.

- [ ] **The KCl fit target: re-target on OIML's 1408.3. RULED 2026-09-18.**
      The `kcl_calibration_standard_within_model_error` test builds a
      **0.01 mol/kgw** solution and compares it against **1413**, which is
      the *volumetric* (0.0100 mol/L) standard's figure. The basis-consistent
      number for the solution the test actually constructs is OIML R 56's
      **1408.3 µS/cm** for its 0.01 D primary standard — the same row
      `conductivity_sources.rs` already validates the fit against at 2%
      tolerance. So this is not a re-fit of `FIT_SQRT`/`FIT_LINEAR`: it is
      removing a basis mismatch from a test and from the prose that explains
      it. The 0.33% between the two is invisible inside the 7% window, which
      is exactly why the mismatch survived.
      **Scope:** the test's two assertions and doc comment; `FIT_SOURCE`'s
      long paragraph; `docs/MUTATION-SENSITIVITY.md` §on this constant;
      `CAPABILITIES.md:236`. The 7% window is **not** narrowed — that was
      offered and not chosen, and narrowing it risks failing CI on a genuine
      model limitation at the dilute end.

- [x] **`solution.solvent_kg`: pose the final state canonically before the
      last solve. RULED 2026-09-18; DONE 2026-09-18.** Both orders of
      `aq-023` now read `solvent_kg 0.0996939730` and `ionic_strength`
      0.2759702852 against 0.2759702847, where they read 0.0997010580 /
      0.0996909590 and 0.2759516202 / 0.2759782269 before. **1.0129e-4 apart
      became 1.0023e-13, and 9.6418e-5 became 1.8361e-9.**
      **The row does NOT stop departing, and that is a finding rather than a
      shortfall.** The perturbation rule reports the first slot that moves,
      so closing the 1e-4 uncovered a 3.76e-5 one underneath it:
      `base_equivalents` — `2·O − H` left over once every portion is booked —
      differs between the orders by 1.26e-8 mol, which is to the digit the
      same 1.25e-8 mol `contents[water]` has always differed by and that the
      same file calls agreement to one part in 4e8. One wobble in a conserved
      sum, read through 5.53 mol at 2e-9 and through 3.35e-4 mol at 3.8e-5.
      It is floating-point accumulation, not a representation, and it wants a
      stabler sum rather than another solve. `ORDER_DEPARTURES` keeps
      `aq-023` with that as its reason.
      **The answer landed on neither of the two old numbers**, which is the
      point: it is the answer to the state rather than to either route
      through it, and the value the ruling predicted was simply the
      powder-first one quoted forward.
      **What the work found, and what two earlier readings got wrong.**
      `mass_H2O` tracks the water the INPUT declared, and the input to the
      last operation is the *intermediate* vessel plus one reagent — a
      calcium chloride solution in one order and a carbonate one in the
      other, holding different shares of their hydrogen and oxygen inside
      species rather than inside water. The vessel's own inventory never
      departed at all, because `complete_basis` rebuilds the water portion
      from conserved H and O. So the fix is one extra pose of the SETTLED
      contents (`PhreeqcEquilibrator::recharacterise_canonically`), which
      replaces the four reported numbers and rebuilds nothing.
      **Why the residue is not zero, stated rather than rounded away:** the
      input writes the solvent mass to nine decimals, coarser than the
      inventory's own 4e-9 disagreement, so both orders produce the same
      input text and `solvent_kg` is exact; the element totals are written
      to twelve significant figures, finer than that residue, which is the
      whole of the remaining 1.8e-9 in the ionic strength — noise in a
      conserved sum, not a representation.
      **The cost, measured:** 8 engine calls became 9 on one ordering and 6
      became 8 on the other — at most one more per equilibration, sometimes
      none when the content-addressed cache answers the re-pose. Exactly the
      "one more solve" the ruling accepted, so there was nothing to stop
      and report. `crates/kerotakis-phreeqc/tests/order_invariance.rs` holds
      the invariance and the ceiling.

- [ ] **`ionic.rs::provenance_of`: thread a `Locale` through
      `net_ionic_for`. RULED 2026-09-18.** The sixth and last member of the
      welded-prose family — the one place still building
      `"{engine} · {dataset} · {model}"` as one English string with no
      `Locale` in scope. Emitting a `Phrase` instead was offered and not
      chosen; the owner chose the direct thread.
      **What reaches the user:** the German ionic-equation drawer reads
      `Herkunft: PHREEQC 3.7.3 · llnl.dat · Pitzer-Ionenwechselwirkung`
      instead of `... · ion interaction (Pitzer)`.
      **The hazard, named because it has bitten before:** `net_ionic_for` is
      a public signature `kerotakis-wasm` calls. #645 broke the build by
      running `cargo clippy -p kerotakis-core` instead of `--workspace`, and
      this is the same shape of change.

- [ ] **MIX and solvent-only characterisation must announce their
      provenance. RULED 2026-09-18.** Both paths write a `Provenance` record
      that no `SolutionRouted` event ever carries, so a vessel filled that
      way holds provenance in its state that never reaches a reader.
      Announcing on MIX only was offered and not chosen; both get it, under
      the same fire-on-change rule `#653` built and tested.
      **What reaches the user:** mixing two beakers narrates
      `> Gelöst mit PHREEQC 3.7.3, Datensatz llnl.dat,
      Pitzer-Ionenwechselwirkung.` after `> Die Lösungen wurden vereinigt.`
      **Watch:** solvent-only is most characterisations, so fire-on-change is
      doing the real work here — if it fires on every plain-water vessel the
      goldens will say so loudly, and that is the signal to look again at
      `Vessel::aqueous_routing_said`, not to suppress the line.

- [ ] **Milk: do the phosphate half of the buffer now, leave casein
      recorded. RULED 2026-09-18.** The measured error is 0.66 of a pH unit
      and roughly half of it is a ~30-line phosphate addition using data
      already in the repo; the other half needs a casein titration curve
      behind a publisher's 403. Waiting for both halves was offered and not
      chosen, and so was declaring milk out of scope.
      **What reaches the user:** vinegar into milk reads pH 6.7 → 5.9 where
      the measurement is 6.7 → 6.1 and today's model says 5.4 — an error of
      1.3 units becoming 0.2.
      **Non-negotiable:** the residual is *named on the wire*, not merely in
      a limits file. A half-corrected buffer that reads as a fully-corrected
      one is worse than the uncorrected one, because nobody checks a number
      that looks right.

- [ ] **GUI-093 — organise the materials shelf by chemical role — is the
      next GUI session. RULED 2026-09-18.** Chosen over GUI-092 (show the
      ionic equation derived), GUI-094 (give the vessel the room) and I18N-4
      (store the locale across reloads), which stay open and unranked.
      **What reaches the user:** the Materialschrank groups as SÄUREN /
      LAUGEN / SALZE / INDIKATOREN / LÖSUNGSMITTEL instead of one
      alphabetical run, which is also what makes the 500-capability corpus
      findable.
      **Constraint carried from the last round:** the in-app search must keep
      grepping description text — `safety_rationale`, `safety_guidance`,
      `procedure_de`, `observations_de` — not just titles, and grouping must
      not reintroduce the virtualisation's `aria-setsize`/`aria-posinset`
      bookkeeping bugs.

### From a live transcript, 2026-09-18 — the peroxide/chalk vessel

**The data gap #665 named, now counted.** The routing estimate caps what a
phase may contribute by the reviewed solubility the registry holds — and
falls back to counting it in full where there is none, on purpose, because
that is what keeps a real brine routing to Pitzer. So the cap only bites
where the data exists, and the transcript's own manganese dioxide and
silver chloride are among the solids where it does not.

Measured against `crates/kerotakis-core/tests/golden/registry.json` and
`data/registry/registry-source-v1.json` on 2026-09-18:

- **93 solid species** in the shipped registry.
- **22** carry a reviewed `aqueous-solubility-g-per-100-ml`, and they are
  mostly organics and polymers — sulfur, chalk and quartz are the only
  three that are also aqueous-database phases.
- **71 do not.** Among them, by a crude name match against the 319 phases
  in `wateq4f.dat`: Cu(OH)2, CuSO4, antlerite, atacamite, brochantite,
  chalcanthite, epsomite, gypsum, langite.

- [ ] **Give the precipitating solids a reviewed solubility, with a source
      each.** Two pieces of work, and the second is the one that is easy to
      miss:

      **The values.** Each needs a citation traceable to an original
      measurement, under the standing rule — any book may be cited, no book
      may be systematically harvested, and the original source of a value is
      what gets cited rather than the compilation that repeated it. A value
      with no reachable source is better left absent than guessed: absent is
      what the full-count fallback is *for*.

      **The crosswalk, which is a finding in its own right.** The nine above
      are what a *string* match finds. AgCl and MnO₂ are database phases too
      — under `Chlorargyrite` and `Pyrolusite`/`Birnessite` — so a registry
      key and a phase name do not compare by equality, and any honest count
      of this gap needs a mineral-name crosswalk first. Until that exists,
      "9" is a floor and not the number.


The owner pasted a Laborbuch transcript from a vessel that boiled dry
while catalase and manganese dioxide were decomposing peroxide. Four
defects in it, each verified against the source rather than inferred from
the prose.

**One thing the same day settled, so nobody redoes it:** `App.svelte` was
audited for the read-after-invalidate shape that produced #664 — a handler
that assigns to a `$state` and then reads a `$derived` (or an `{@const}`,
which is one) computed from it. 53 states, 14 deriveds, every arrow-function
handler in the markup: two candidates, both spurious on inspection. The
remove-vessel dialog was the only instance. The COMPONENTS were not audited
the same way and do not need to be for this shape — a component cannot
assign to the prop its `{@const}` derives from — but that argument is the
reason, not an assumption, and it stops holding the moment a component owns
state a sibling `{@const}` reads.

- [x] **The routing caveat prints "~71046,6 mol/kgw" one line above
      "I = 0,0004 mol/kgw".** `aqueous.rs:3089` estimates a "potential
      molality" as *(dissolved totals + 2 × equilibrium-phase moles + 2 ×
      solid-solution moles) / `problem.kgw`*, and `> 1.0` both **selects
      the dataset** and triggers the `routing.activity-model-fallback`
      caveat that quotes the number. Two things are wrong with it at once
      and they compound:

      **The denominator is an evaporating solvent.** There is no floor on
      `problem.kgw`. The transcript's vessel has boiled dry — it says so
      itself, *"das letzte Wasser ist fort"* — so the fixed solute
      inventory is being divided by a residue. 71046.6 against a numerator
      of order 0.05 mol implies a `kgw` of about 1e-6 kg, a milligram of
      water. The measured ionic strength printed directly underneath, from
      the solver that actually ran, is 0.0004.

      **The numerator counts solids that will never dissolve.** Each
      equilibrium phase contributes two ions per formula unit. In this
      vessel those phases include chalk — whose *own* message three lines
      earlier says it dissolves to 0.0013 g per 100 mL, "below anything a
      beaker would show" — and silver chloride, and manganese dioxide.
      The estimate is deliberately pessimistic ("what the solid phases
      *could* dissolve"), but counting a solid the engine has separately
      declared insoluble is not pessimism, it is a contradiction between
      two parts of the same answer.

      **Why it is not cosmetic:** `potential_molality > 1.0` picks the
      database. A vessel drying out is routed to a different activity
      model, and told it is, on the strength of a number the solver
      disagrees with by eight orders of magnitude.

      **Done in #665, and what it found.** Both faults are real and the
      arithmetic checks out: a numerator of 0.048 mol over a `kgw` of
      6.76e-7 kg — 0.68 mg of water — is 71 048 mol/kgw. The estimate is
      now bounded twice, on the value that ROUTES rather than on a printed
      copy of it: the solvent mass is floored at one millilitre (the
      largest floor that leaves `condense_supersaturated`'s documented
      1 mL probe untouched), and a phase contributes at most what the
      water present could hold, read from the same reviewed solubility
      that composes its own `Event::Inert` sentence. The transcript's
      beaker goes from **71 048 to 0.00026 mol/kgw**, beside the solver's
      measured 0.0004 — the first time the two halves of that answer
      agree. Brine is untouched: 8 mol of NaCl in a kilogram still reads
      16.0, and the 1 mL probe still hands the router 200.

      **What it does NOT close, deliberately.** The cap only bites where
      the registry has reviewed a solubility — 22 species, of which
      chalk, quartz and sulfur are the only ones that are also database
      phases. A solid it has not reviewed is still counted in full,
      because that is what keeps halite and sylvite routing a real brine
      to pitzer. So manganese dioxide and silver chloride, both in that
      vessel, still contribute their whole inventory. Closing that is
      registry data with a source behind it, not arithmetic in
      `aqueous.rs`, and it is the next thing to do here.

- [x] **Identical lines repeat on every solve step.** The same
      `Event::Inert` for chalk is emitted unconditionally at
      `solve.rs:2466` and `solve.rs:2603` — once per step, forever. The
      transcript carries the same forty-word German sentence about chalk's
      solubility more than thirty times, and "NICHT MODELLIERT:
      Silbernitrat ist mit einer Flüssigkeit in Kontakt" ten times. This
      is the same defect shape `#653` solved for provenance: fire on
      change, comparing `Phrase::shape()`, not on every tick. Whatever is
      done here should reuse that mechanism rather than invent a second
      one.

      **Done in #667, and what it found.** `Vessel::honesty_said` is the
      sibling of `aqueous_routing_said` — `#[serde(skip)]`, compared as a
      shape, holding what STANDS rather than everything ever said, so a
      solid that stops being inert and is inert again is announced again.
      Across the 113 lesson goldens, **49 changed and all 49 changed the
      same way**: 133 lines removed, no vessel state moved, no line added
      that was not a first occurrence, and not one count went up. The
      largest single reduction was `rusting`, 40 events to 30.

      **`NotYetModeled` had half a mechanism, and it was not this one.**
      `render_events_in` drops a line identical to one already in the
      batch — but only inside ONE batch and only at lv1, so thirty steps
      are thirty batches and it never saw them, and lv2/lv3 readers were
      not covered at all. It gets the same treatment at the same two
      sites.

      **No suppressed line carried a quantity that moves.** Only
      `inert.insoluble-in-water` has a measurement in it at all — the
      reviewed solubility — and of the species that can reach that branch
      (below 0.01 g/100 mL) none has a second, 100 °C entry to
      interpolate towards, so it cannot move while the sentence stands.

      **What the repetition was costing, found by eye.** `hard-water`'s
      third vessel printed the calcium-chloride apology twice and pushed
      magnesium sulfate's own first sentence out of view; with the repeat
      gone, the magnesium sulfate line appears. The noise was not only
      noise — it was crowding out news.

- [ ] **The engine says chalk dissolved and, on the next line, that it
      does not dissolve and is "still all there".** Verbatim, in order:
      *"0,000001 mol Kreide (Calciumcarbonat) gelöst"*, then *"Kreide
      (Calciumcarbonat) inert: … löst sich nicht in Wasser … Es ist noch
      vollständig vorhanden"*. PHREEQC's equilibrium dissolves a micromole;
      the `Inert` message is generated from a solubility table
      (`aqueous_solubility_at`, filtered to `< 0.01` g/100 mL) that knows
      nothing about what the solver just did. Two subsystems describing the
      same solid and disagreeing — the family of #626 and #630, the words
      against the scene. The fix is not to silence either one: it is that
      the sentence claiming "still all there" must be derived from what is
      left, not from a table lookup.

- [ ] **"Silbernitrat ist mit einer Flüssigkeit in Kontakt" in a vessel
      with no liquid.** Emitted repeatedly *after* the engine has already
      reported that the last water has gone. Whatever decides "is in
      contact with a liquid" is not reading the same state as the
      evaporation step. Smaller than the others and probably a one-line
      predicate, but it is the third place in one transcript where two
      parts of the answer contradict each other, which is the pattern
      worth naming.

### UI framework

`kerotakis-core` is the invariant either way; the CLI defers the choice
harmlessly. If web is a real target → Tauri (same Rust → wasm). If mobile UI
polish outranks web → Flutter, accepting a thinner web story. The codex markup
convention must **not** wait on this decision (P4).

### Registry/codex storage

SQLite (via `rusqlite`/`sqlite-wasm-rs`, wasm-proven) if it wants real queries;
`postcard`/`rkyv` + `include_bytes!` if read-only lookup. Decide when L1 is
built; both are wasm-clean.

---

## Governance

- **Licence:** AGPL-3.0-or-later, with an App Store / Google Play additional
  permission for binaries published by the copyright holders. See `LICENSE`
  and `NOTICE`.
- **The §7 trap, closed:** under GPLv3/AGPLv3 §7 only copyright holders can
  grant additional permissions. `CONTRIBUTING.md` therefore requires, from the
  first PR, that all contributions are licensed **AGPL-3.0-or-later + the
  store exception** (inbound = outbound including the exception — the
  Nextcloud model; Signal's CLA is the heavier alternative if needed later).
- **Data licences are tracked separately from code** — per-source provenance
  files in `kerotakis-data` and `tools/`, reproduced in the app's about screen.

---

## Name & trademark status

Cleared 2026-08-18 via TMview (aggregates USPTO, EUIPO and 70+ national registries).

- **One** KEROTAKIS mark worldwide: Argentina only, Nice class 34
  (tobacco/smokers' articles), owner HELMFELT, reg. 3470789, expires 2033-11-21.
  No conflict with software.
- **Zero** hits at USPTO, zero at EUIPO, zero in classes 9 / 41 / 42 worldwide.
- Zero hits for phonetic variants (`cerotakis`, `kerotaki`, `kerotakys`,
  `kerotaxis`, `keratakis`, stem `kerotak`) in those classes.
- All of `.com` `.app` `.dev` `.io` `.org` `.net` were unregistered; crates.io,
  npm and PyPI all free.

Outstanding:

- [ ] Register the domains — the only item here with a race condition
- [ ] Claim `kerotakis` on crates.io, npm, PyPI (the CLI publish does this with
      substance behind it)
- [ ] Mechanise releases (`release-plz`, Apache-2.0, verified 2026-08-19)
      and give each codex release a **Zenodo DOI** — the CC BY-SA dataset
      becomes citable for the education-research audience we cite, and the
      DOI timestamps the licence grant
- [ ] File classes 9 / 41 / 42 through an attorney nearer launch, once the
      goods-and-services wording is settled. What was done is a screen, not a
      clearance opinion.

# Current product directive — core laboratory UX first (2026-08-27)

Before adding or restructuring missions, examples, or progression, make the
shared Sandbox/Story laboratory feel like a real, explorable, colourful place.
The active sequence is tracked in `ROADMAP-GUI.md` GUI-070…GUI-084:

1. localized cabinet search and arbitrary capacity-aware amount/unit entry;
   Story uses the same persistent Mission set / Unlocked / All scope in both
   cabinet tabs; case supplies are temporary loans, while Sandbox exposes all;
2. continuous bench space with optional workflow guides and compact object
   placement/removal controls;
3. persistent learner notes in the laboratory journal;
4. cupboards, racks, clickable/zoomable posters and periodic table, utilities,
   and visually distinct room environments;
5. placeable stands/clamps, magnetic stirrer/hotplate, mini centrifuge, probes,
   burners, baths, cooling, filtration and connected rigs;
6. motion and visible effects driven by computed power, temperature, RPM,
   viscosity, fill, particle settling, reaction energy, gas/solid amount and
   time—never generic animation.

Existing experiment and mission content is deliberately not redesigned in this
phase; it is fitted into the improved shared workflow.

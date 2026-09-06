# The sixteen classroom experiments — audit and plan (CAP-24)

> Finished work is not listed here. What landed, and what it taught us, is in
> [HISTORY.md](HISTORY.md). Task numbers are never renumbered and never reused.

The cross-corpus breadth dependencies are now executable `BRD-*` tasks in
**[BREADTH.md](BREADTH.md)**. This file continues to own experiment/quest
semantics; it must not duplicate library-integration scope. New EXP tasks name
their required BRD task when identity, material, reaction-family, biochemical,
crystal, physical-interaction or scientific-view infrastructure is required.

Audited 2026-08-24 against the tree at that day's main. Sixteen
primary-school experiment titles (user-supplied, German) mapped to
what the engine, codex, and GUI can actually do. Verdicts are the
audit; the plan follows. Rule zero, from the request that created
this file: **the user must never have exactly one thing to do** —
these become open-world quests with nudges, not corridors.

## Verdict matrix

| # | Experiment | Engine today | Codex/lesson today | Gap class |
|---|---|---|---|---|
| 1 | Stark wie ein Magnet | no magnetism anywhere | none | NEAR: `magnetic` species property + `magnet` separation verb (the filter/drain family pattern) |
| 2 | Backstube Chemielabor | fizz route (acid+NaHCO3) works; **thermal** decomposition missing | `fizz.lab` (acid route only) | NEAR: curated thermal decomposition 2 NaHCO3 →Δ Na2CO3+H2O+CO2↑ with a temperature threshold; headspace + gas events already carry the rest |
| 3 | Klimamacher Treibhausgase | headspace gases yes; **radiative heating** missing | none | HARDER: curated IR absorbance per gas + a lamp-on-vessel heating-rate model rides `Irradiate` (photochem.rs has flux) |
| 4 | Schmutzwasser reinigen | `filter` verb exists; multi-stage = chained vessels | none dedicated | NOW: lesson/quest authoring + suspended-dirt species with appearance; turbidity rendering is GUI-side |
| 5 | Der rote Fleckenteufel | NaOCl exists; dyes and bleaching reactions missing | none | NEAR: 2–3 dye species (appearance+spectrum) + curated hypochlorite bleaching; colour-safe comparison = same bench minus the oxidant |
| 6 | Strom aus der Sonne | electrochemistry yes; **photovoltaics is semiconductor physics** | none | BOUNDARY: declined as engine chemistry — the bench must not fake a solar cell. GUI may ship a curated device widget clearly labelled as data, not computation |
| 7 | Die schäumenden Perlen | thermal modes exist; **wall heat-loss (U-value) missing**; polymer.rs has populations | none | HARDER: Newton-cooling with per-vessel insulation coefficient; then insulated-vs-bare cooling curves are computed, and `kero study` plots them |
| 8 | Absender gesucht | **chromatograph verb landed** (plate model, computed K) | `one-thing-at-a-time.lab` (alcohols, not inks) | NOW+data: ink-dye species with partition data → felt-tip separation; paper-strip rendering is GUI-side |
| 9 | Das grüne Wunder | Irradiate + photochem rates exist; no photosynthesis route | none | HARDER: curated photo-reaction (CO2+H2O+light → glucose+O2, chlorophyll-gated), glucose species; O2 headspace detection already works |
| 10 | Wie wäscht Seife? | saponification landed (`react`); γ∞ partitioning landed | `there-and-back.lab` (ester angle) | NEAR: fat/oil species + a curated emulsification demo on the partition machinery; the soap is already made on-bench |
| 11 | Wie fängt man Schall? | acoustics — not chemistry | none | BOUNDARY: declined, stated. Outside the engine's subject |
| 12 | Die Plastik Docs | density per species exists; layers logic exists | none | NEAR: PE/PP/PET/PS as data species (density, provenance) → float/sink separation in water/brine is computable today |
| 13 | Rostschutz für Lebensmittel | redox machinery yes; ascorbic acid + iodine assay missing | none | NEAR: ascorbic-acid species + curated iodine decolorisation redox + starch indicator |
| 14 | Das süße Brot | catalase sets the enzyme precedent | none | NEAR: amylase + starch + glucose/maltose species, curated enzymatic hydrolysis, Lugol colour assay |
| 15 | Das Boden-Phänomen | transport.rs CellChain (1-D column) landed | `transport-column.lab` | HARDER: clay/sand/silt as column materials with retention parameters → percolation compared per soil; the machinery is the hard part already done |
| 16 | Die sprudelnde Erfrischung | CO2 headspace + carbonate chemistry solve | `limewater.lab` | NOW: quest authoring on the existing chemistry (limewater cloudiness is computed) |

**Honest tally.** Codex today covers 1 of 16 fully (#16), 3 partially
(#2 acid-route, #8 different solutes, #10 different angle). The
engine can carry the heart of 4 today (NOW), reaches 7 more with
curated data and reactions (NEAR — agent-sized), 4 with one small new
physics model each (HARDER), and 2 are declared boundaries (#6, #11)
— the bench does not fake what it cannot compute, and each declined
entry gets a codex model-boundary note in the honesty lineage of
`when-the-lab-says-it-does-not-know`.

## The open-world layer: quests, not corridors

A lesson (.lab) is a replayable script — a corridor. These sixteen
need the opposite: a stated goal, a free bench, and nudges that fire
on what the learner actually does. Design:

- **Quest file** (TOML, beside the codex; same lint discipline): a
  goal in three registers; a set of NUDGE rules, each `when = <event
  pattern or vessel predicate>` → `say = <register-appropriate hint>`
  (fires at most once, never blocks, never takes the only next step);
  a set of COMPLETION claims — codex-style event expectations that
  may be satisfied **in any order** across the whole bench; optional
  side-quest links ("your funnel just made two layers — the
  chromatography quest can use that").
- **Quest engine** (core + CLI): subscribes to the event stream the
  codex kinds already provide; matches nudge/completion rules;
  `kero quest list/start/status`. The codex predict/diagnosis
  machinery is the pedagogical voice; the quest engine is only the
  matcher. Multiple quests run concurrently by construction.
- **GUI** (the GUI workline owns rendering): quest journal panel,
  nudge toasts driven by the same events the effects layer already
  consumes, instrument/glassware affordances per quest — the tray,
  burette, transfer tool, and effects (fire/steam/frost) exist; the
  sixteen need per-experiment additions listed in the matrix (paper
  strip for #8, lamp for #3/#9, magnet tool for #1, turbidity for
  #4, cooling-curve overlay for #7 riding the chart contract).
- **Never one thing to do:** every quest's completion set must be
  reachable by at least two orders; nudges reference alternatives;
  the shelf stays fully open during quests, and hazards stay live —
  the safety screen is part of the open world, not suspended for the
  tour.

### Campaign decision (2026-08-26): quests become world missions

The landed quest engine is the evaluator under a larger story system; it is not
the final player-facing structure. A mission connects an engine-evaluated
scientific problem to a place, a person or organization, a persistent world
change, and a meaningful capability reward. The bench stays free throughout.

```text
story arc
  ├── mission: a substantial problem with persistent consequences
  │     ├── objective claims: engine-evaluated outcomes/evidence
  │     ├── optional discoveries: events or state the player may notice
  │     └── nudges: contextual, finite, non-blocking
  └── opportunity: small ambient request, sample, anomaly, or challenge
```

Terminology in code may remain `quest` during migration. New content uses
"mission" in user-facing English and "Mission" or the context-appropriate
German translation in locale data; stable ids are never localized.

#### Mission contract

Every mission file must contain:

| Field | Meaning |
|---|---|
| `id`, `version`, `locale_key` | Stable identity, migration version, and localized copy root |
| `arc`, `chapter`, `location`, `giver` | Narrative/world placement; all optional for standalone Sandbox scenarios |
| `premise` | The problem and why it matters, rendered at all three registers |
| `start` | Samples, world facts, aliases/unknowns, allowed starting bench setup |
| `objectives` | Typed claims over events, measurements, final state, notebook evidence, or submitted sample |
| `constraints` | Safety, time, available quantity, contamination, cost, apparatus, or waste limits |
| `nudges` | One-shot contextual hints with cooldown and register-localized text |
| `discoveries` | Optional computed observations that can open leads or codex concepts |
| `outcomes` | Success, partial success, recoverable failure, and honest unsupported branches |
| `rewards` | Access, apparatus, supply source, location, relationship, research node, or story fact |
| `next` | Zero or more newly visible missions; never an enforced single successor |

Mission copy describes intent and stakes. It must not prescribe exact clicks
unless the mission explicitly teaches safe operation of unfamiliar apparatus.
Even then, the instruction is an optional demonstration that can be dismissed
and replayed.

#### Objective vocabulary

Objective evaluation remains inside the engine/protocol boundary. The initial
complete vocabulary is:

- **Observe:** produce or rule out a typed event such as gas evolution,
  precipitate, phase split, flame color, endpoint, or hazard.
- **Measure:** record a property with a named instrument and any required
  calibration/uncertainty.
- **Identify:** submit the identity or class of a sealed unknown and cite one or
  more observations from the notebook.
- **Produce:** create a target species, phase, concentration, pH, temperature,
  purity, volume, or yield within explicit tolerances.
- **Separate:** deliver named output containers meeting recovery and purity
  thresholds; multiple processes can qualify.
- **Compare:** create evidence across two or more controlled trials, not merely
  click a prediction.
- **Design:** satisfy a performance envelope such as buffer capacity, heat
  output, corrosion protection, or minimum waste.
- **Avoid/contain:** complete an outcome without forbidden hazard events,
  releases, contamination, or apparatus-limit breaches.
- **Explain:** choose or compose a claim whose referenced observations and
  quantities agree with the computed run. Explanations never replace the run.

Claims may be `required`, `one_of`, `at_least(n)`, or `optional`. The evaluator
returns structured unmet reasons so the UI can say what evidence is missing
without leaking a procedure.

#### Progression without grind

Story progression has four legible resources:

1. **Access** — rooms, field locations, suppliers, people, and sample sources.
2. **Equipment** — permanent apparatus/instrument families and lab upgrades.
3. **Supplies** — replenishable Story quantities used to make constraints
   meaningful; common educational reagents never require repetitive grinding.
4. **Research** — concepts demonstrated by evidence, opening advanced studies
   and mission leads; this is not a spendable point currency.

There are no XP bars, randomized drops, daily streaks, energy timers, loot
boxes, or premium currency. Funding, if later introduced, represents explicit
mission budgets and replacement costs and may always be disabled in classroom
settings. Sandbox ignores all progression and quantity restrictions.

#### Story spine, with lateral freedom

The initial arc is a small laboratory becoming a trusted regional research
station. Chapters describe growing capability, not locked grade levels:

1. **The First Bench** — observation, safe handling, mixtures, mass, notebook;
   earn basic glassware, balance, thermometer, and local sample access.
2. **Water Stories** — pH, dissolved material, filtration, contamination,
   gases; open the water desk, probes, filters, and field sampling.
3. **Heat and Time** — heating/cooling, rates, energy, gas handling; open the
   thermal bay, hotplate, calorimeter, and controlled vessels.
4. **Signals in Matter** — qualitative tests, unknowns, spectra, electricity;
   open the analysis room and higher-precision instruments.
5. **Separation Works** — extraction, distillation, chromatography, recovery,
   waste; open process equipment and multi-stage assemblies.
6. **Independent Research** — player-proposed targets, studies, optimization,
   and bounded advanced packs; the authored story becomes a source of problems,
   not a finish screen.

Each chapter launches with at least three visible leads serving different
styles: investigation/identification, making/design, and community/environment.
Main-arc missions may depend on a demonstrated capability but not on completing
every preceding mission. Optional discoveries can reveal shortcuts or new
branches, never mandatory hidden-object hunts.

#### First playable vertical slice: The contaminated sample

GUI-080 and the mission workline share one acceptance slice:

- A community garden reports residue and a cloudy water sample. The player can
  inspect the room, talk/read briefly, accept up to three concurrent leads, and
  bring sealed samples to the same persistent bench.
- Mission A identifies what causes the cloudiness; Mission B prepares a known
  reference solution; Mission C delivers treated water under turbidity/pH and
  waste constraints. An optional observation reveals a second source upstream.
- The cabinet begins with common glassware, stir rod, balance, pH paper, funnel,
  and filter. Completing evidence work earns a reusable digital pH probe; the
  unlock is visible in Story and was already available in Sandbox.
- Treatment must accept at least filtration-first and precipitation/settling-
  first solutions when both satisfy the same final-state claims. The story may
  react to cost, recovery, waste, or safety differences without invalidating a
  chemically correct solution.
- The slice is complete only when English and German, all three registers,
  mouse/touch/keyboard operation, save/reload, Story↔Sandbox switching, and
  reduced motion are tested end to end.

#### Mission authoring definition of done

A mission is shippable only when `kero quest lint` (or its successor) proves the
schema and locales complete; automated runs prove at least one success and every
declared failure/partial branch; at least one non-tutorial mission has two valid
solution traces; objective reasons remain stable protocol data; rewards migrate
across save versions; and Sandbox can load the mission as a standalone scenario
without importing Story progression.

## Ownership and sequencing (CAP-24 slices)

1. Quest schema + engine + `kero quest` (Fable — the hard seam).
2. NOW quests authored on existing chemistry: #4, #8, #16, #10
   (agents; each quest = TOML + any missing species data + full gate).
3. NEAR data/reaction tranches: #1, #2, #5, #12, #13, #14 (agents —
   the same curated-row + registry-pipeline discipline as CAP-23:
   safety rows, exporter canonicalisation, golden regen, SMILES where
   the molecule has one).
4. HARDER models, one per branch: #3 radiative heating, #7 U-value
   cooling, #9 photosynthesis route, #15 soil columns (Fable, with
   agent data support).
5. BOUNDARY entries #6/#11: codex model-boundary notes; GUI decides
   whether a labelled data-widget is worth shipping (its call).
6. GUI per-experiment affordances: the matrix's last column is the
   requirements list; the GUI workline schedules it in ROADMAP-GUI.md.

---

# Part 2: the aqueous virtual-lab problem collection (audit 2026-08-24)

A second corpus: ~40 classic quantitative-chemistry teaching
problems, spanning stoichiometry through analytical chemistry.
**Rule first:** no external problem texts, scenarios, or data are
ever copied or adapted into this repo, from anywhere. What is audited
here is the *capability class* of each problem; our own problems get
written against those classes from scratch. The source does not
matter and is deliberately not named.

This corpus sits closer to the engine's core than Part 1: it is
almost entirely aqueous quantitative chemistry. Several of its
sections are already *tested invariants* of our engine, not features
to build: Hess's law, order-independence of dissolution heat, and
single-counted neutralisation heat are named tests in
`kerotakis-phreeqc/tests/equilibrator.rs`; saturation-limited
dissolution and temperature-coupled solubility are solved and tested;
14 registry species carry dissolution enthalpies.

## Verdict by capability class

**NOW — the chemistry already computes; only quests are missing:**
- Dilution and solution prep (dilute verb, mix verb, molarity from
  the solved state) — HCl-class problems work end to end.
- Stock solutions from solids (dissolution + saturation are solved).
- Limiting reagents incl. precipitation routes (AgNO3+NaCl class is
  `silver-and-salt.lab` chemistry).
- All of thermochemistry: reaction enthalpy, Hess demonstrations,
  mixing-temperature ("coffee") problems ride
  `adiabatic_mix_temperature`, unknown-heat-capacity determination.
- Strong/weak acid-base pH, successive-dilution pH ladders, buffers
  (`buffer.lab`), titration curves with pKa readable at
  half-equivalence (titrate verb + curve landed).
- Ksp determination and solubility-vs-temperature for database salts.
- Redox series ordering (`spannungsreihe.lab` + displacement).
- Gravimetric AgCl analysis (filter + balance are verbs).

**NEAR — data or one small instrument away:**
- Glucose/sucrose problems: both data-species now exist. BRD-012.S03
  landed glucose (and fructose) on sucrose's finite neutral-solute rung
  with a CRC room-temperature capacity, so a glucose problem has a
  species that dissolves to a limit and leaves the excess solid. The
  photosynthesis *reaction* EXP-9 wants is still absent: the identity is
  here, the curated photo-route is not.
- Density-identification problems: a volume-displacement reading
  (graduated-cylinder instrument; solid volumes are already known
  internally from molar mass and density).
- Solution-density problems (% mass / molarity / density triangle):
  curated mixture-density correlations (ethanol-water first, CRC).
- Arsenic gravimetric class: As species ride the wateq4f database the
  engine already ships; registry rows + safety rows.

**HARDER — one bounded model each:**
- Binding-equilibrium problems (dye–macromolecule K): a curated 1:1
  association solver — small, honest, and reusable for indicator
  chemistry.
- Custom weak acids not in shipped databases (KHP-class
  standardisation): needs the custom-species route into the engine's
  input, or a curated-pKa titration path.
- Speciation-driven colour (cobalt chloride equilibrium shifts): ties
  solution speciation to rendered colour — the appearance machinery
  exists, the coupling does not.

**BOUNDARY:** none. Unlike Part 1, nothing in this corpus is outside
the bench's subject — it is all chemistry the engine either does or
can honestly grow into.

## The two cross-cutting enablers (they matter more than any row)

1. **Quantitative quest claims.** This corpus's essence is the
   numeric target: "produce 500 mL of 3.0 M ± tolerance", "determine
   Ksp to two significant figures". The CAP-24 quest engine therefore
   needs value-claims — a completion condition that reads the solved
   state (concentration, mass, temperature, pH) and checks a target
   within a stated tolerance — alongside event-claims. Grading
   precision (sig-figs) can later ride CAP-8's uncertainty machinery.
2. **Unknown reagents.** Half the collection's pedagogy is "identify
   the unknown". The bench needs sealed species: a reagent whose
   identity the UI hides behind a label ("Unknown A") while the
   engine computes it truthfully underneath — identification IS the
   quest. Needs: an aliasing layer in the quest engine + UI, never a
   change to the chemistry itself.

## Ownership additions
- Quest engine (Fable, CAP-24 slice 1) grows both enablers: value
  claims + sealed species. These unlock ~18 problems' worth of quest
  classes at once.
- NEAR data tranche (agents, after current queues): As-series rows,
  mixture-density correlations, graduated cylinder. Glucose and sucrose
  are done (BRD-012.S03 and earlier).
- HARDER models (Fable): association-K solver, custom-acid route,
  speciation-colour coupling.

---

# Part 3: the task registry (EXP numbers are stable identifiers)

The audits above become work here. Rules restated because they are
load-bearing: **ideas, concepts, and task-classes only — never any external
collection's texts, scenarios, or data**; every problem
we ship is written from scratch against the capability class. EXP
numbers are never re-bound (same law as CAP/OPT). Every task follows
the established discipline: registry pipeline for new species (safety
rows, exporter, golden regen, SMILES where molecular), full preflight
for main, claim-audit statuses with acceptance evidence.

## Infrastructure

- **EXP-0 — Quest engine** — [x] **done 2026-08-24** (Fable).
  `kerotakis-codex::quest`: TOML specs linted like the codex (a
  single-claim quest is rejected as "a corridor with a door at the
  end"), event claims on the codex's own `kind:detail` matcher, value
  claims (ph / temperature_c / mass_g / moles:<sp> / molarity:<sp>,
  target ± tolerance read from the solved state, with the
  solution-volume model stated), identify claims closing the
  sealed-unknown loop, nudges that fire exactly once and never block.
  REPL: `quest list/start/status/answer`; sealed aliases are a pure
  display layer — input unmasked before parsing, rendered lines
  re-masked, chemistry untouched. Preflight gained the "quest lint"
  step. Acceptance held: the demo quest (the-white-unknown) exercises
  every feature; two distinct command orders complete through the
  full solver stack; the unknown stays sealed in every rendered line
  until named; a wrong answer is spoken, locks nothing, and the right
  answer still completes (`tests/quest_engine.rs`, cli
  `tests/quest.rs`). Original scope line follows.
  (Fable; everything below depends on it).
  Schema (TOML beside the codex, linted), event-claim matcher,
  **value claims** (target ± tolerance read from solved state:
  concentration, mass, volume, temperature, pH), **sealed unknowns**
  (UI-side aliasing "Unknown A" over a truthfully-computed species;
  chemistry untouched), `kero quest list/start/status`, nudge rules
  (fire once, never block, never the only path), any-order
  completion. Acceptance: one quest file exercising every feature;
  two distinct completion orders proven in tests; a sealed unknown
  identified only via measurements; preflight green.

## Part-1 experiments (school kit; German titles are the map key)

- **EXP-1 Magnet** — magnetic property + `magnet` separation verb +
  recycling quest. Acceptance: mixed Fe/Cu/Al solids separate; the
  non-magnetic remainder is stated; conservation exact.
  Data landed 2026-08-24 (kero-basic): Al species added, `magnetic` bool on
  SpeciesData (Fe=true), `magnet v1 v2` verb moves ferromagnetic solids.
  **Quest authored** (kero-basic): `magnet-sorting.toml` — event claims
  on `magnet_separated` + `added`.
- **EXP-2 Backpulver** — curated thermal decomposition
  2 NaHCO3 →Δ Na2CO3 + H2O + CO2↑ (threshold ~50–100 °C stated with
  source); quest links the fizz route and the heat route as two paths
  to the same gas. Acceptance: heating dry NaHCO3 evolves CO2 into a
  sealed headspace; limewater from the existing lesson detects it.
  **Quest authored** (kero-basic): `baking-powder.toml` — acid + thermal
  CO₂ paths, event claims on `gas_evolved`, `gas_tested`, `reacted`.
  Fold-in (scenario-simulation corpus): generalises to thermal
  decompositions broadly, including CaCO3/the limestone cycle.
- **EXP-3 Treibhausgase** — per-gas IR-absorbance data + lamp
  heating-rate model on `Irradiate`; quest compares CO2 vs air vs
  water-vapour bottles. Acceptance: computed warming curves differ by
  gas with sources; `kero study` sweeps concentration.
- **EXP-4 Wasserfilter** — dirt species (suspended solid, appearance)
  + multi-stage filter quest. Acceptance: turbidity falls stage by
  stage; dissolved salt passes and the quest says why.
  **Quest authored** (kero-basic): `water-filter.toml` — event claims
  on `filtered`, `measured`, `observed`.
- **EXP-5 Fleckenteufel** — dye species + curated hypochlorite
  bleaching; quest compares oxidant vs oxidant-free wash. Acceptance:
  dye colour is bleached only with NaOCl; the colour-safe wash keeps
  it; three registers say the mechanism.
  **Data landed 2026-08-24 (kero-basic):** betanin (red, λmax 535 nm),
  curcumin (yellow, 425 nm), indigo carmine (blue, 610 nm) with
  16-band Gaussian spectra + oxidised products; 3 curated NaOCl
  bleaching reactions; colour-safe wash verified.
  **Quest authored** (kero-basic): `stain-remover.toml` — event claims
  on `reacted`, `added`, `observed`.
- **EXP-6 Photovoltaik** — codex model-boundary note ONLY (declined
  as computation); GUI decides on a labelled data widget.
- **EXP-7 Dämmung** — per-vessel U-value Newton cooling; quest
  compares insulated vs bare cooling curves via the chart contract.
  Acceptance: cooling curves computed, U stated with provenance.
- **EXP-8 Filzstift-Chromatografie** — ink-dye species with partition
  data; quest separates a black ink. Acceptance: ≥3 dyes resolve on
  the landed column; areas conserve; GUI paper-strip is GUI-side.
  Fold-in (scenario-simulation corpus): TLC (Rf), ion-exchange,
  size-exclusion and HPLC join as chromatograph modes on the same
  partition physics; ion-exchange rides the upstream EXCHANGE machinery.
- **EXP-9 Fotosynthese** — glucose species + curated photo-reaction
  (chlorophyll-gated) on photochem flux. Acceptance: O2 accumulates
  in headspace under light, not in dark; stoichiometry exact.
- **EXP-10 Seife** — fat/oil species + emulsification demo on γ∞
  partitioning; quest chains on-bench saponification into washing.
  Acceptance: fat partitions with soap present, not without.
- **EXP-11 Schall** — codex model-boundary note ONLY (declined).
- **EXP-12 Plastik** — PE/PP/PET/PS density species; float/sink
  separation quest in water/brine. Acceptance: the four sort by
  density exactly as their data say; provenance per polymer.
  **Data landed 2026-08-24 (kero-basic):** PE 0.95, PP 0.90,
  PET 1.38, PS 1.05 g/mL; registry pipeline (safety rows, golden
  regen, model parameters).
  **Quest authored** (kero-basic): `plastic-doctors.toml` — event
  claims on `layers_formed`, `observed`, `measured`.
  Fold-in: polymer-formation items from the scenario-simulation corpus
  join this task's scope.
- **EXP-13 Vitamin C** — ascorbic acid species + curated iodine
  decolorisation + starch indicator. Acceptance: titration-style
  counting of drops to endpoint works; juice-vs-water contrast.
  **Quest authored** (kero-basic): `vitamin-c.toml` — iodometric
  assay, event claims on `added`, `reacted`, `observed`, `measured`.
- **EXP-14 Amylase** — amylase/starch/maltose species + curated
  enzymatic hydrolysis + Lugol assay. Acceptance: starch negative
  after enzyme+time+warmth, positive without; the sweetness line at
  lv1 is the maltose the ledger shows.
  **DONE** (2026-08-24): curated reaction with catalyst gate, 2 new
  registry species, safety rows, maltose SMILES, 10 tests all green.
  **Quest authored** (kero-basic): `sweet-bread.toml` — enzyme
  catalysis, event claims on `added:starch`, `added:amylase`,
  `reacted`, `observed`.
- **EXP-15 Boden** — clay/sand/silt column materials with retention
  parameters on the landed CellChain. Acceptance: percolation-time
  and retention orderings match the curated data; three-column
  comparison quest.
- **EXP-16 Sprudel** — quest authoring on existing CO2/limewater
  chemistry. Acceptance: the quest completes via at least two paths
  (warming the bottle vs shaking-analogue vs acid+carbonate).
  **Quest authored** (kero-basic): `fizzy-drink.toml` — CO₂ via
  limewater, event claims on `gas_evolved`, `gas_tested`,
  `temperature_changed`, `reacted`.

## Part-2 capability classes (our own problems, written from scratch)

- **EXP-17 Solution-prep quest pack** — dilution ladders, stock from
  solids, target-molarity value-claims. Needs: EXP-0 only.
  **Quest authored** (kero-basic): `solution-prep.toml` — value claim
  on `molarity:NaCl` (target 0.1 ± 0.02), event claims on `dissolved`,
  `measured`, `diluted`. Gap: leans on event claims; T2 will add more
  value claims for the dilution ladder.
- **EXP-18 Density identification** — graduated-cylinder/displacement
  instrument + sealed-unknown metal and liquid quests. Needs: EXP-0.
  **Quest authored** (kero-basic): `density-id.toml` — sealed unknown
  (Cu behind alias "unknown-metal"), Identify claim + event claims on
  `measured`, `observed`.
- **EXP-19 Mixture-density data** — curated ethanol-water (then
  sucrose-water) density correlations with sources; unlocks
  concentration-from-density quests.
  **Ethanol-water landed 2026-08-24 (kero-basic):**
  `ethanol_water_density_g_ml(w)` in properties.rs, 5th-order fit
  to CRC Handbook 97th ed. at 20 °C, max 1.5 mg/mL residual, 6 tests.
  CLI: `kero properties ethanol-water-density w=0.4`.
  **Sucrose-water landed 2026-08-31:** `sucrose_water_density_g_ml(w)`
  in properties.rs. The published datum runs the other way — NBS
  Table 114 (Bates 1942, NBS Circular C440) gives °Brix FROM apparent
  specific gravity as a cubic — so the function INVERTS it by
  bisection rather than re-fitting a second polynomial: the cubic's
  derivative has a negative discriminant, hence is strictly
  increasing, and the published coefficients stay the only data in the
  file. ρ = d·ρ_water(20 °C, Tanaka). Valid 0–40 % w/w, the
  polynomial's own stated range; above that it refuses. Cross-checked
  against ISCOTABLES 7th ed. (independent source): agreement within
  1.0 mg/mL at 0/5/10/20/30/40 %. 6 new tests.
  CLI: `kero properties sucrose-water-density w=0.2` → 1.081028 g/mL.
  **Codex entry authored:** `sugar-syrup-by-density` in
  quantitative.toml — the balance reads 124.70 g, the syrup is
  20.05 % w/w, and all three predict options are real engine numbers
  (0.998 water, 1.081 syrup, 1.59 crystal sucrose from the registry).
- **EXP-20 Limiting-reagent pack** — precipitation and gas routes,
  predict-then-check quests with value claims.
  **Quest authored** (kero-basic): `limiting-reagent.toml` — event
  claims on `precipitated:AgCl`, `filtered`, `measured`. Gap: no value
  claim on precipitate mass yet (T2 candidate).
- **EXP-21 Thermochemistry pack** — reaction enthalpy, Hess
  three-path demonstration, mixing-temperature and unknown-heat-
  capacity quests. The engine side is DONE (tested invariants);
  this is authoring.
  **Quest authored** (kero-basic): `two-roads-one-temperature.toml` —
  Hess's law, event claims on `reacted`, `measured`, `dissolved`.
  Gap: no value claim on ΔT yet (T2 candidate for temperature_c
  target ± tolerance).
  Fold-in (commercial-simulation corpus): bomb calorimetry
  (constant-volume ΔU vs ΔH) joins this task; the sealed rigid vessel
  is landed machinery, the U-vs-H distinction is the lv3 line.
- **EXP-22 Acid-base pack** — pH ladder by successive dilution, weak
  acid problems, buffer design to a target ratio, titration-to-pKa
  with the curve read at half-equivalence.
  **Quest authored** (kero-basic): `acid-base.toml` — event claims on
  `measured`, `diluted`, `titrated`, `observed`. Gap: no value claim on
  pH yet (T2 candidate for ph target ± tolerance).
- **EXP-23 Standardisation class** — potassium hydrogen phthalate (or
  an equivalent primary-standard acid) added from primary data; the
  custom-weak-acid route into the engine; 4-significant-figure
  discipline via burette precision. HARDER.
- **EXP-24 Solubility pack** — Ksp determination, solubility-vs-T
  with predict-then-test at a third temperature (value claim).
  **Quest authored** (kero-basic): `solubility.toml` — event claims
  on `dissolved`, `precipitated`, `temperature_changed`, `measured`.
  Gap: no value claim on Ksp yet (T2 candidate).
- **EXP-25 Redox-ordering quest** — design-your-own-experiment over
  the landed displacement chemistry; completion = correct ordering
  of Cu/Mg/Zn/Pb by any valid route.
  **Quest authored** (kero-basic): `redox-ordering.toml` — event
  claims on `reacted`, `cell_voltage`, `observed`.
- **EXP-26 Gravimetric pack** — precipitate, filter, dry, weigh;
  sealed-unknown AgNO3 concentration by mass. Needs: EXP-0.
  **Quest authored** (kero-basic): `gravimetric.toml` — event claims
  on `precipitated:AgCl`, `filtered`, `measured`. Gap: no value claim
  on precipitate mass_g yet, no sealed-unknown concentration (both T2
  candidates). Fold-in: also covers the BaSO4 gravimetric route
  (sulfate variant, from the university-practical corpus).
- **EXP-27 Association-K solver** — curated 1:1 binding equilibrium
  (solver + one binding quest); reusable for indicators. HARDER.
- **EXP-28 Speciation-colour coupling** — solution colour computed
  from speciation (cobalt chloride class); appearance machinery
  exists, the coupling is the task. HARDER.
- **EXP-29 Water-quality analytical scenario** — arsenic-series
  species on the shipped wateq4f chemistry; our own scenario, our
  own wells, value-claim detection thresholds. Fold-in
  (scenario-simulation corpus): eutrophication/wastewater
  nitrate-phosphate chemistry joins this scenario family
  (PHREEQC-native water chemistry).

## Sequencing
EXP-0 first (Fable). Then NOW-tier authoring (EXP-16, 4, 21, 25, 26,
17) can fan out to agents in parallel with the NEAR data tranches
(EXP-12, 5, 13, 14, 2, 1, 19); HARDER models (EXP-3, 7, 9, 15, 23,
27, 28) one per branch after their data lands. The GUI workline reads
this file for its affordance list.

---
# Part 4: task registry continued (EXP-30 and beyond), quest status, and cross-references

Corpus audits 2026-08-24 through 2026-09-02 mapped eight further collections
(school-curriculum practical canon, guided-practical classes 9-12, a
directory of simulations, a commercial simulation set, a university
practical set, a scenario-simulation catalog, quest-authoring coverage, and
the breadth-programme handoff) against EXP-0..29. The full audit narrative,
covered/compressed lists, and per-corpus registry tallies are recorded in
`HISTORY.md`. What survives here: the new task numbers those audits
produced (below), the durable declined-with-reasons verdicts, the current
quest-authoring blocker table, the breadth-prerequisite map, and the
cross-reference to the children's corpus. Same laws as Part 3: ideas and
task-classes only, never another collection's texts/scenarios/data; EXP
numbers are never re-bound.

## EXP-30..52 (new tasks; same registry-pipeline discipline as Part 3)

- **EXP-30 — Qualitative inorganic analysis** — HARDER (breadth), partially
  landed. Hydroxide precipitation matrix (Cu²⁺/Fe³⁺/Fe²⁺/Mg²⁺/Zn²⁺/Ca²⁺ +
  NaOH), AgCl from chloride, CO₂ effervescence from carbonate, the sulfate
  row (BaSO4 via the shipped USGS database), seven sealed-unknown salt
  quests (six single + one two-unknown capstone), and the MIX-path parity
  fix all landed 2026-08-25 through 2026-09-01 (see HISTORY.md, EXP-30, for
  the engine-repair narrative). Ion roster grew via a fold-in from the
  university-practical corpus: NH4+, Na+, K+, Ca2+, Ba2+, Mg2+, Mn2+
  cations; SO3^2-, Br-, I- anions. Acceptance's six-unknowns bar is met.
  STILL OPEN: NH3/SO2 gas-test observables (no aqueous degassing path,
  NEAR engine route), a dedicated flame-test verb (NEAR — currently only an
  accidental `ignite` side effect), excess-alkali amphoterism for Zn/Al
  (NEAR, needs zincate/aluminate species), the sealed-unknown display layer
  outside the CLI (`inspect` leaks the real species; hazard chips key off
  the real name not the alias; wasm has no alias map at all — all NEAR,
  CLI/GUI).
- **EXP-31 — Gas tests** — [x] **done 2026-08-25** (kero1), branch
  `kero1/exp31-gas-tests`. Pop (H2), glowing splint (O2), limewater (CO2),
  damp litmus (NH3) as curated test actions on the headspace, each an event
  with three registers. 18 tests. Quest authored (kero-basic):
  `gas-tests.toml`.
- **EXP-32 — True solution / colloid / suspension** — particle-size
  classification + a Tyndall-scatter flag on appearance; filtration and
  settling behaviour differ by class. Acceptance: salt vs starch vs chalk
  classify correctly by computable behaviour (filter passes, scatter flag,
  settling), not by label.
- **EXP-33 — States and purity** — melting/boiling point as an
  identification instrument (ties to sealed unknowns), sublimation as a
  phase route (NH4Cl class), crystallisation with hydrate bookkeeping
  (CuSO4·5H2O). Acceptance: an unknown identified by MP/BP against registry
  data; a sublimation separation quest; a hydrate loses and regains its
  water with exact mass accounting.
- **EXP-34 — Rusting kinetics** — curated slow oxidation of iron gated on
  water AND oxygen; the classic nail-in-conditions comparison (dry / boiled
  water / salt water). Acceptance: rust forms only where both are present;
  salt accelerates via the curated rate; `kero study` sweeps conditions.
  (KID-5 pulls this task forward — see HISTORY.md and KIDS.md.)
- **EXP-35 — Combustion energetics of alcohols** — ignite + calorimetry
  comparison across methanol/ethanol (both on the shelf). Acceptance:
  enthalpy-per-mole ordering emerges from the ledger, not from a table
  shown to the learner. **Quest authored** (kero-basic): `alcohol-burn.toml`
  — event claims on `ignited`, `measured`. Gap: no value claim on
  temperature_c yet (T2 candidate for calorimetric ΔT comparison).
- **EXP-36 — Organic synthesis pack** — acetylation-class synthesis on the
  kerotakis-org SMIRKS machinery (salicylic-acid data species, template
  proven at the molecule level like esterification), with
  recrystallisation + melting-point purity check chaining EXP-33.
  Acceptance: template-proven product, yield honest, purity checked by the
  MP instrument. HARDER (org data care). Fold-in (university-practical
  corpus, shared with EXP-42): named substrates aspirin, paracetamol,
  naphthyl acetate, ethyl-propionate Fischer row (propanoic acid joins the
  data list), and the condensation (aldol/Claisen-Schmidt) class join the
  same SMIRKS-template capability.
- **EXP-37 — Spectrophotometric determination quest** — the Beer-Lambert
  machinery and permanganate calibration oracle landed with CAP-22; this is
  authoring only: calibration curve, unknown concentration by value-claim.
  Acceptance: unknown determined within tolerance from absorbance alone.
  **Quest authored** (kero-basic): `spectrophotometry.toml` — event claims
  on `measured`, `diluted`, `observed`. Gap: no value claim on
  concentration yet (needs the Spectrophotometer instrument's output to be
  readable as a quantity — T2 candidate if wired).
- **EXP-38 — Curriculum paths and progress layer** — quests tagged with
  public-curriculum labels (CBSE/ICSE/IGCSE/NGSS classes) so a learner picks
  a path; progress, study-queue nudges, and post-quest quizzes ride the
  quest engine state and the codex predict machinery. GUI owns the surface;
  the engine side is tags + state queries on EXP-0. Fold-in
  (directories/simulations corpus): balancing-equations quiz mode (`kero
  balance` + 103 codex reactions) folds in here. Fold-in
  (scenario-simulation corpus): nomenclature name-to-structure quiz rounds
  (org stack's iupac module) fold in here too. Both are authoring, not new
  engine work.
- **EXP-39 — Redox titrimetry** — [x] **done 2026-08-30**. `titrate ...
  until` gained two endpoints beside the existing pH default: potentiometric
  (`until pe <op> <value>`, reading the solver's own pe) and self-indicating
  (`until colour persists`, reading computed Beer-Lambert colour word —
  works because permanganate's own high molar absorptivity makes it
  self-indicating). Oxalic acid joined the registry as
  `dissolves_without_speciation` (no shipped PHREEQC database speciates
  oxalate — see HISTORY.md for the finding). **Known gap:** the
  potentiometric endpoint cannot yet deliver a *curve* in pe on this system
  — open to air, atmospheric O2 buffers pe flat regardless of titration
  progress; swept of air, the single manganese redox couple gives a flat
  republished input value rather than a withheld potential, unlike the
  engine's existing withhold-on-no-root behaviour
  (`redox.rs::the_equivalence_point_reports_no_potential`). Pinned by
  `a_swept_flask_reports_the_default_pe_rather_than_the_couple` (expected to
  fail loudly once fixed). Iodometry (thiosulfate/starch endpoint) folded in
  from the university-practical corpus as a third endpoint mode. Acceptance
  held: KMnO4 standardised against oxalic acid to a value-claim; endpoint
  within one drop; both endpoint modes tested.
- **EXP-40 — Biomolecule assays** — the food-test canon: reducing sugars
  (Fehling/Benedict class), proteins (Biuret), starch (Lugol, already in
  EXP-14), fats (grease-spot/emulsion tie to EXP-10). Curated test rows with
  colour outcomes over real registry species (glucose from EXP-9/17 data).
  Acceptance: sealed food-sample quests identify composition from test
  patterns. Fold-in (scenario-simulation corpus): Kjeldahl nitrogen-to-
  protein joins as this task's quantitative arm; Sudan IV joins its
  named-test rows.
- **EXP-41 — Organic qualitative analysis** — functional-group wet tests
  paired with what the org crate can already do: `perceive_groups` computes
  the groups from structure, the curated wet tests (carbonyl, alcohol, acid,
  amine classes; Lassaigne-class elemental detection for N/S/halogens)
  confirm them. Acceptance: an unknown organic narrowed by tests alone, each
  test backed by curated chemistry + the perception cross-check. HARDER.
  Fold-in (scenario-simulation corpus): ceric ammonium nitrate, azo-dye
  amine test, and litmus-for-acids join the row list.
- **EXP-42 — Preparative chemistry pack** — the classic preparations:
  double salts by crystallisation (Mohr's-salt/alum class, riding EXP-33's
  hydrate bookkeeping), gas preparation with property study (SO2 class,
  riding EXP-31's tests), amphoteric aluminium (Al + NaOH -> aluminate + H2
  beside Al + HCl), azo-dye class synthesis on the org machinery (EXP-36
  family — shares that task's fold-in of named substrates). Acceptance:
  each preparation conserves exactly, yields honestly, and its product
  survives an identification test from EXP-30/33.
- **EXP-43 — Clock kinetics** — [x] **done 2026-08-24**: two curated rate
  laws (iodide-peroxide, iodate-bisulfite Landolt), 4 new registry species,
  safety rows, SMILES, 12 tests all green. Acceptance: clock time scales
  with concentration and temperature as the rate law says; the sweep
  reproduces the classic linearisation. **Quest authored** (kero-basic):
  `iodine-clock.toml`.
- **EXP-44 — Excess enthalpy of mixing** — [x] **partially done**. Excess
  enthalpy of mixing (hᴱ) computed from UNIFAC's own temperature dependence
  as a vessel state function anchored at 25°C (see HISTORY.md for the
  path-independence finding). Acetone-water allowlisted and reproduces the
  literature S-curve; ethanol-water withheld (wrong dilute-end sign for this
  parameter set — the thermo suite pins the deviation). `total_excess_j()`
  generalised 2026-09-02 from water-anchored to any verified unordered
  organic binary (ternaries and separated layers still refused by design).
  **STILL OPEN — acetone-chloroform:** two gaps remain (audited 2026-08-31):
  (1) chloroform is not a registry species; (2) UNIFAC main group 11
  (CCl3) and its twelve directional a_mn values against every group already
  present are absent, and THERMO-004 restricts sources to the 1975
  Fredenslund/Jones/Prausnitz paper and the 1982 Gmehling revision. Both are
  now data tasks with nothing structural in front of them. Acceptance: the
  mixing-calorimetry quest shows warming for the associating pair and
  cooling for a positive-deviation pair, both computed. HARDER.
- **EXP-45 — The conservation quest** — the law of conservation of mass in
  a sealed reaction, weighed before and after. Acceptance: at least three
  reaction types (precipitation, gas-in-sealed-flask, neutralisation) each
  balance on the vessel balance to the digit. **Quest authored**
  (kero-basic): `conservation.toml`. Gap: no value claim on mass_g yet (T2
  candidate — target = initial mass ± 0.001).
- **EXP-46 — Cross-coupling template class** — modern C-C and C-N bond
  chemistry (biaryl coupling from aryl halide + boronate; amide C-N
  activation) as curated SMIRKS templates on the org machinery, with the
  catalyst as a required condition (nickel-class species present or the
  reaction refuses) — the bench books the transformation and its
  conditions; it does not simulate the catalytic cycle, and says so.
  Acceptance: two templates proven at molecule level; refusal without
  catalyst; boundary line in lv3. HARDER (org). Fold-in
  (university-practical corpus): gains the Grignard reaction —
  organometallic reagent formation gated by
  `nonaqueous::single_organic_solvent`; any water present makes the
  preparation refuse, computed rather than a scripted warning.
- **EXP-47 — Colligative pack** — the four classic colligative properties
  as one computed family: vapour-pressure lowering, boiling-point
  elevation, freezing-point depression (landed; joins the pack), osmotic
  pressure (van't Hoff) with a semipermeable membrane link between two
  vessels as the one new mechanism. Acceptance: all four scale with
  particle molality including the van't Hoff factor for electrolytes;
  membrane flow equilibrates honestly. Fold-in (scenario-simulation
  corpus): tonicity/IV-drip framing joins here.
- **EXP-48 — Interfacial properties** — surface tension and capillarity as
  curated per-liquid data with computed capillary rise. **First slice done
  2026-08-31:** water σ(T) from IAPWS R1-76, `capillary_rise_mm` computed
  via Jurin's law from that σ plus the Tanaka density (validity capped by
  density's narrower 0-40°C range, not surface tension's own 0-100°C — see
  HISTORY.md for the finding). Codex entry `warm-water-climbs-less`.
  Remaining: ethanol/hexane σ rows (cohesion/adhesion contrast) and the
  EXP-10 soap surface-tension-drop measurable, which waits on the
  surfactant species EXP-10 itself needs.
- **EXP-49 — The nuclear bench** — [x] **done 2026-08-24** (Fable), first
  slice. Curated teaching-isotope table (C-14, I-131, Rn-222, Co-60,
  Tc-99m, the Sr-90 -> Y-90 -> Zr-90 chain; NUBASE2020 data) wired via
  `NuclideLedger`: chemically inert, decays inside `wait` beside kinetics,
  Geiger counter reads total Bq. Nucleons conserve exactly across decay;
  elements do not (alpha keeps its He-4; beta/nu departures and the mass
  defect are stated boundaries — see HISTORY.md for the finding). Metastable
  Tc-99m stays distinct from Tc-99. Acceptance held (`tests/nuclear.rs`).
  Remaining: decay-series depth (Bateman), codex radioactivity concept
  family, half-life value-claim (needs an activity/half-life Quantity type
  not yet in the enum). Quest authored: `half-life.toml` (event claims
  only, pending that type).
- **EXP-50 — Mechanistic selectivity rules** — [x] **landed 2026-08-24**,
  branch `kero1/exp50-mechanistic-selectivity`. Substitution-vs-elimination
  outcome prediction (SN1/SN2/E1/E2) by substrate class, nucleophile
  strength, and temperature: 6 selectivity rules (March ch.10), 5 product
  entries, 2 substrates (bromoethane, tert-butyl bromide), 2 nucleophiles,
  80°C temperature threshold, 15 tests including 3 condition-flip and 2
  mass-conservation checks, 8 new species. Verb: `react v1 haloalkane`.
  Acceptance held: the classic condition matrix reproduces textbook
  outcomes; changing one condition flips the product and the lv3 line says
  which rule fired. Regiochemistry rules where they bind are still open.
- **EXP-51 — Enzyme kinetics** — Michaelis-Menten as a curated rate family
  with Km/Vmax and competitive vs non-competitive inhibition, riding the
  existing kinetics integrator and the catalase precedent; assayed
  spectrophotometrically (EXP-37 machinery). Acceptance: Lineweaver-Burk
  from `kero study` sweeps distinguishes the two inhibition mechanisms;
  parameters recovered within tolerance.
- **EXP-52 — Disposal and lab-practice quests** — waste routing as computed
  chemistry: neutralise before drain, never mix the oxidiser stream with
  organics, halogenated separate — the safety screen already computes the
  hazard verdicts; a curated disposal rule table turns clean-up into quests
  where wrong routing triggers the same screen that guards the bench.
  Acceptance: a clear-the-bench quest gradeable entirely by existing safety
  machinery plus the rule table.

## Declined items (all corpora, recorded with reasons)

- Computational/quantum chemistry beyond the bench's subject: 2D NMR
  interpretation, protein-ligand docking, HOMO-LUMO/MO visualisation,
  retrosynthesis planning, spectral interpretation (MS/NMR/IR/MALDI/GC-MS —
  declined across four consecutive corpora; the decline is stable).
- Reference/visualisation, GUI-workline territory, not engine chemistry:
  atomic-structure interactives (isotope builders, electron configuration,
  Bohr spectra, periodic trends, VSEPR/polarity builders), VR/multiplayer
  surfaces, Rutherford scattering and periodic-table study (the GUI's
  interactive table already serves the reference need).
- Outside the bench's subject entirely: biology/health-sciences/
  microbiology (DNA/RNA synthesis, karyotyping, cell-context osmosis — the
  physical half lives in EXP-47 — water cycle, disease scenarios, Gram
  stains, PCR, ELISA, cell culture, anatomy, ecology), physics (Newtonian
  mechanics, optics, plate tectonics, reactor physics — nuclear fission/
  fusion REACTORS declined even though nuclear DECAY is EXP-49),
  meteorology (relative humidity, dew point — weather, not the beaker,
  though its vapour-pressure heart is EXP-47's machinery).
- Water-system phase rule (triple-point P-T manipulation): parked as
  investigate-only — the states machinery covers bench pressure; full P-T
  phase-diagram control is a real model decision, not a row.
- Cement hydration (concrete lab): declined for now — real chemistry, but
  multiphase hydration kinetics is beyond the current bench; recorded so it
  isn't re-litigated silently.

## Quest authoring status

32 quest TOML files in `quests/` pass `kero quest lint`, covering 26 EXP
numbers (see each EXP entry above for its quest file and claim gaps).
Batch authored 2026-08-25, with sealed-unknown and value-claim upgrades
2026-08-29 and the seven EXP-30 salt quests the same day (see HISTORY.md
for the full authored-quest ledger and the specific value-claim targets).

Quests that could not yet be written as TOML, by blocker:

| EXP | What blocks the quest | Status |
|---|---|---|
| 3 | IR-absorbance per gas + lamp heating-rate model | HARDER (model) |
| 7 | U-value Newton cooling model | HARDER (model) |
| 8 | ink-dye species with partition coefficients | NEAR (data) |
| 9 | photosynthesis curated reaction + glucose species | HARDER (model+data) |
| 10 | fat/oil species + emulsification demo | NEAR (data) |
| 15 | clay/sand/silt column materials | HARDER (data) |
| 23 | custom weak-acid route (KHP) | HARDER (engine route) |
| 27 | 1:1 association-K solver | HARDER (model) |
| 28 | speciation→colour coupling | HARDER (model) |
| 29 | arsenic-series registry rows | NEAR (data) |
| 32 | particle-size classification + Tyndall flag | NEAR (data+model) |
| 33 | MP/BP instrument + sublimation phase route | NEAR (instrument) |
| 34 | curated slow iron oxidation (water+O2 gated) | NEAR (data) |
| 36 | org synthesis templates (salicylic acid) | HARDER (org data) |
| 38 | curriculum path tags + progress layer | infra (not a quest) |
| 39 | quest authoring only — engine endpoints landed, see EXP-39 | authoring |
| 40 | curated food-test rows (Fehling, Biuret) | NEAR (data) |
| 41 | functional-group wet tests | HARDER (data+coupling) |
| 42 | preparative chemistry data (double salts, etc.) | NEAR (data) |
| 44 | chloroform species + UNIFAC main group 11 params, see EXP-44 | HARDER (data) |
| 46 | cross-coupling SMIRKS templates | HARDER (org) |
| 47 | semipermeable membrane mechanism | HARDER (model) |
| 48 | ethanol/hexane σ rows + EXP-10 soap measurable, see EXP-48 | NEAR (data) |
| 50 | quest authoring only — selectivity rules landed, see EXP-50 | authoring |
| 51 | Michaelis-Menten rate family | HARDER (model) |
| 52 | disposal rule table | NEAR (data) |

## Breadth-programme prerequisite map (2026-08-27 handoff)

The eight-corpus audit found the demand; `BRD-000` turned it into the
versioned 500-prompt curiosity regression corpus. This mapping is normative
for agents: complete the shared BRD prerequisite once, then author multiple
EXP quests against it rather than implementing compound-pair exceptions
inside a quest.

| Experiment families | Shared breadth prerequisite |
|---|---|
| EXP-5/8/10/12/15/18/19/29/30/32/33/34/40/44 | `BRD-012` familiar substances and `BRD-014` named material recipes |
| EXP-36/41/42/46/50 | `BRD-020…023` reaction-family IR, executor and organic pack |
| EXP-2/3/7/21/31/33/35/44/48 | `BRD-030…032` fluid/phase routing where applicable; experiment-specific physics remains EXP-owned |
| EXP-2/31/35/43 | `BRD-040…041` reviewed gas/combustion mechanisms where equilibrium/curated kinetics are insufficient |
| EXP-9/14/40/47/51 | `BRD-050…052` bounded biochemical router and familiar bio pack |
| EXP-24/28/33/42 | `BRD-060…062` only when the lesson claims a real crystal structure/symmetry |
| EXP-1/4/7/10/12/15/18/32/48/52 | `BRD-070…073` only for spill/drop/fluid/physical handling; chemical endpoints remain engine-owned |
| structure, crystal, orbital and protein inspection across EXPs | `BRD-080…081`; Ketcher authoring additionally requires `BRD-082` |

Quest acceptance remains stricter than breadth coverage: a known substance
or installed visualization does not make an experiment complete. Each EXP
still needs its own observable success/failure claims, controls, model
boundary, lesson/codex content and replay tests. Conversely, `BRD-100`
cannot close while an uncovered curiosity prompt points at an unowned
EXP-level behavior.

## Registry state (current)

EXP-0..52. Yield per corpus (new task numbers produced): 16 -> 9 -> 7 -> 1
-> 3 -> 0 -> 3 across the eight audits 2026-08-24/25 (see HISTORY.md for
the audit-by-audit tallies and dates). New corpora now confirm coverage
rather than add chemistry; the build order stands: EXP-0 unlocks
everything, NOW-tier authoring and NEAR data tranches fan out behind it,
HARDER models follow one per branch.

## Cross-reference: the children's corpus

A thirty-experiment audit run from the other end (a kitchen table rather
than a curriculum) lives in [`KIDS.md`](KIDS.md), with its own stable task
prefix (`KID-*`). It found the engine in better shape than the corpus's
first pass suggested and the *reach* into it far worse (the headline
finding is recorded in HISTORY.md). Three of its findings are bugs against
work recorded here, owned by the KID numbers rather than re-bound to EXP
ones:

- **KID-2** — acid curdling never fires with the aqueous solver linked
  (`curdling::observe` reads `CH3COOH`; the solver has speciated it to
  `CH3COO-`), so `lessons/milk-curds.lab` does not demonstrate its own
  headline claim on the shipped bench (see KIDS.md, KID-2 — landed).
- **KID-3** — the L0 screen is dose-blind and screens a `MaterialRecipe`'s
  own components against each other, so 1 mL of 1% Lugol raises a
  Danger-level "can detonate" banner in the starch and vitamin-C activities
  (`EXP-13`, `EXP-14`); see KIDS.md, KID-3 (slice 1 landed, KID-3b open).
- **KID-4** — `ignite` on an unresolved material emits nothing at all (see
  KIDS.md, KID-4).

`KID-5` pulls `EXP-34` (rusting) forward and `KID-9` pulls `EXP-8`'s Rf
mode forward, because both are top-ten children's experiments rather than
tail coverage.

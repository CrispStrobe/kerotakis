# The sixteen classroom experiments — audit and plan (CAP-24)

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
- Glucose/sucrose problems: two data-species (glucose already queued
  for Part 1's photosynthesis).
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
- NEAR data tranche (agents, after current queues): glucose, sucrose,
  As-series rows, mixture-density correlations, graduated cylinder.
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
  CLI: `kero properties ethanol-water-density w=0.4`. Sucrose-water
  remains.
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
  candidates).
- **EXP-27 Association-K solver** — curated 1:1 binding equilibrium
  (solver + one binding quest); reusable for indicators. HARDER.
- **EXP-28 Speciation-colour coupling** — solution colour computed
  from speciation (cobalt chloride class); appearance machinery
  exists, the coupling is the task. HARDER.
- **EXP-29 Water-quality analytical scenario** — arsenic-series
  species on the shipped wateq4f chemistry; our own scenario, our
  own wells, value-claim detection thresholds.

## Sequencing
EXP-0 first (Fable). Then NOW-tier authoring (EXP-16, 4, 21, 25, 26,
17) can fan out to agents in parallel with the NEAR data tranches
(EXP-12, 5, 13, 14, 2, 1, 19); HARDER models (EXP-3, 7, 9, 15, 23,
27, 28) one per branch after their data lands. The GUI workline reads
this file for its affordance list.

---

# Part 4: the school-curriculum practical canon (audit 2026-08-24)

A third corpus arrived as a sprawling commercial checklist site. The
site itself contributes nothing (a thin wrapper, not named here — no
source ever is); what it points at is the **public practical canon of
the national curricula** (CBSE/ICSE, Cambridge IGCSE/A-level, NGSS) —
those are open standards, and THEY are the organizing skeleton worth
taking. Audited against EXP-0..29: most of the canon is already
covered (titration, pH, electrochemistry, chromatography, rates,
calorimetry, displacement/reactivity series, filtration/distillation,
solution prep, limiting reagents, Ksp — all EXP or landed lessons).
What follows is only what is genuinely NEW.

## New tasks (EXP numbers continue; same laws apply)

- **EXP-30 — Qualitative inorganic analysis** (the crown of the
  school analytical canon; "salt analysis"). FIRST SLICE LANDED
  (2026-08-25): the hydroxide precipitation matrix computes — Cu²⁺,
  Fe³⁺, Fe²⁺, Mg²⁺, Zn²⁺, Ca²⁺ with NaOH; AgCl from chloride;
  CO₂ effervescence from carbonate; the dilute-PbCl₂ non-verdict —
  pinned in `kerotakis-phreeqc/tests/qualitative.rs`. Getting Fe²⁺
  to give the *green* hydroxide (and not the ferric one) forced four
  engine repairs: per-database polymorph translation (Ferrihydrite ↔
  Fe(OH)3(a)), a reviewed foreign-phase injection (wateq4f has no
  ferrous hydroxide at all), state-reachability admission for
  uncoupled redox elements, and in-solve oxidation-state pinning so
  phantom redistribution stops leaking water (order-independence
  guarded by the displacement metamorphic test). SECOND SLICE
  (BRD-012.S02): the **sulfate row** — barium chloride into a sulfate
  solution precipitates barite, computed from the shipped USGS
  database's own Barite phase rather than curated, with barium and
  sulfur conserved across the solve
  (`kerotakis-phreeqc/tests/school_salts.rs`). THIRD SLICE
  (2026-08-29): the **sealed-unknown salt quests** — seven quest specs
  (six single unknowns plus a two-unknown capstone), authoring only,
  no engine change; see the gap log below for what the authoring could
  NOT reach. STILL OPEN: NH3/SO2 gas test observables (EXP-31
  overlap), a dedicated flame-test verb, excess-alkali amphoterism,
  MIX-path parity (the MIX input builder still filters phases to
  native names, so polymorph translation and foreign injection do not
  apply when two solutions are combined by fraction), and the
  sealed-unknown display layer outside the CLI. The INST-008
  `QualitativeTest`/`QualitativeResult` types exist unwired. Scope:
  the classic scheme as computed chemistry — cation tests (NaOH/NH3
  precipitation with excess behaviour), anion tests (AgNO3 halide
  series, BaCl2 sulfate, acid+limewater carbonate), flame tests (the
  `FlameTest` event already fires), each a real engine solve with a
  curated observation layer; sealed-unknown salt quests where the
  learner infers identity from test patterns. Acceptance: at least
  six unknown salts identifiable by tests alone, every test verdict
  backed by a computed solve or a curated row with provenance; wrong
  inferences get diagnosis lines in the codex predict style. HARDER
  (breadth, not depth) — the highest-value single item in this part.

  ### EXP-30 quest slice (2026-08-29) — what landed
  Seven sealed-unknown specs in `quests/`, TOML only, every hit path
  and one wrong-identification path run in the REPL before it was
  written down. Salts sealed: CuSO4, FeSO4, MgSO4, Na2CO3, FeCl3,
  BaCl2 (single unknowns) and Na2SO4 + NaCl together (capstone).
  Acceptance's "six unknown salts identifiable by tests alone" is met,
  and the multi-unknown question is **answered rather than blocked**:
  `QuestSpec.unknowns` is a map, the CLI installs every alias on the
  shelf at `quest start`, and `two-white-jars.toml` completes in both
  interleavings — no engine change was needed.

  ### EXP-30 engine gap log — classic tests NOT usable in a quest
  One line each, from running them, not from reading the source. The
  list is deliberately not shrinking: two entries below were *added*
  by this slice.
  - **NH3 from an ammonium salt** — `NH4Cl + NaOH` in an open beaker
    produces no gas event of any kind (pH goes 4.93 → 12.65 and that
    is all), so the damp-litmus test has nothing to work on and
    ammonium cannot be a sealed unknown. Blocker: no NH3 degassing
    path out of the aqueous solve. NEAR (engine route).
  - **SO2 from a sulfite** — same shape; only the *thermal* route
    (`ignite` on a dry sulfate) emits SO2, which is decomposition, not
    the acid + sulfite bench test. NEAR (engine route).
  - **Flame tests** — partly reachable, and only by accident of
    plumbing: there is no flame-test verb, the `FlameTest` event falls
    out of `ignite` when the CEA thermal solver declines to burn the
    contents, and it reports the FIRST content that carries a
    `flame_colour`. It works cleanly on dry BaCl2 (apple green),
    NaCl / Na2SO4 (bright yellow) and KCl (lilac) — `the-heavy-salt`
    claims it — but dry CuSO4, MgSO4 and FeSO4 thermally decompose
    instead (CuO/MgO/Fe2O3 + SO2) and emit no flame-test event at all,
    and Na2CO3 carries no flame colour in the registry. On a solution
    the event names the dissolved ion (`Na+`), which is correct
    chemistry but bypasses the alias mask. NEAR (a `flame <vessel>`
    verb over the registry colour, independent of combustion).
  - **Excess-alkali amphoterism (Zn, Al)** — measured, not assumed:
    0.01 mol ZnSO4 + 0.02 mol NaOH gives 0.0100 mol Zn(OH)2, and a
    further 0.10 mol NaOH returns only **0.0002 mol** to solution.
    The zincate complex is not in the candidate phase set, so the
    hydroxide does not redissolve and Zn cannot be told from Mg. This
    is why `the-bitter-salt` states its candidate list excludes zinc
    instead of pretending. NEAR (data: zincate/aluminate species).
  - **MIX parity** — unchanged and still open; every quest here adds
    reagents to one vessel rather than combining solutions by
    fraction, which is the only reason the polymorph translation holds
    throughout. HARDER (engine route).
  - **Sealed unknowns leak through `inspect`** (new) — `print_vessel`
    is the one CLI render path that does not call `mask()`, so
    `inspect v1` on a sealed BaCl2 prints "barium ion" in full. Even
    masked it would leak: `mask()` rewrites only the sealed key and
    its display name, never the ions it dissociates into. Pre-existing
    (it affects `the-white-unknown` too), out of scope for a TOML-only
    change. NEAR (CLI display layer).
  - **Sealing a hazardous species drops its hazard chip** (new) — the
    "toxic" label on soluble barium hangs off the registry KEY
    (`kerotakis_safety::hazard_labels`), and a sealed alias has no
    key, so on every surface that renders hazards the sealed jar shows
    nothing. In the CLI this is invisible either way (the REPL never
    renders hazard labels on an `add`), but it means the BaCl2
    teaching moment in `the-heavy-salt` lives in the quest prose
    rather than on the bench. Worth a decision before sealed unknowns
    reach the GUI. NEAR (safety/display).
  - **Sealed unknowns are CLI-only** — `kerotakis-wasm` exposes
    `questStart`/`questAnswer` but has no alias map and no mask, so a
    sealed-unknown quest exported to the web bench would show the real
    species on the shelf. GUI work, not authoring. NEAR (GUI).
  - **A sealed species that is also a standard reagent is ambiguous**
    — `mask()` is a global string substitution, so with BaCl2 sealed
    every line mentioning the *reagent* barium chloride would read as
    the alias. `the-heavy-salt` sidesteps it by running the sulfate
    test backwards (Na2SO4 is the reagent, the unknown supplies the
    barium), which turned out to be the better quest anyway. Authoring
    constraint, not a bug.
- **EXP-31 — Gas tests** — [x] **done 2026-08-25** (kero1), branch
  `kero1/exp31-gas-tests`. Pop (H2), glowing splint (O2), limewater
  (CO2), damp litmus (NH3) as curated test actions on the headspace,
  each an event with three registers. 18 tests: positive/negative/
  refusal paths, mass conservation, O₂-limited combustion.
  **Quest authored** (kero-basic): `gas-tests.toml` — event claims on
  `gas_evolved`, `gas_tested`.
- **EXP-32 — True solution / colloid / suspension** — particle-size
  classification + a Tyndall-scatter flag on appearance; filtration
  and settling behaviour differ by class. Acceptance: salt vs starch
  vs chalk classify correctly by computable behaviour (filter passes,
  scatter flag, settling), not by label.
- **EXP-33 — States and purity** — melting/boiling point as an
  identification instrument (ties to sealed unknowns), sublimation as
  a phase route (NH4Cl class), crystallisation with hydrate
  bookkeeping (CuSO4·5H2O). Acceptance: an unknown identified by
  MP/BP against registry data; a sublimation separation quest; a
  hydrate loses and regains its water with exact mass accounting.
- **EXP-34 — Rusting kinetics** — curated slow oxidation of iron
  gated on water AND oxygen; the classic nail-in-conditions
  comparison (dry / boiled water / salt water). Acceptance: rust
  forms only where both are present; salt accelerates via the
  curated rate; `kero study` sweeps conditions.
- **EXP-35 — Combustion energetics of alcohols** — ignite +
  calorimetry comparison across methanol/ethanol (both on the shelf).
  Acceptance: enthalpy-per-mole ordering emerges from the ledger,
  not from a table shown to the learner.
  **Quest authored** (kero-basic): `alcohol-burn.toml` — event claims
  on `ignited`, `measured`. Gap: no value claim on temperature_c yet
  (T2 candidate for calorimetric ΔT comparison).
- **EXP-36 — Organic synthesis pack** — acetylation-class synthesis
  on the kerotakis-org SMIRKS machinery (salicylic-acid data species,
  template proven at the molecule level like esterification), with
  recrystallisation + melting-point purity check chaining EXP-33.
  Acceptance: template-proven product, yield honest, purity checked
  by the MP instrument. HARDER (org data care).
- **EXP-37 — Spectrophotometric determination quest** — the
  Beer–Lambert machinery and permanganate calibration oracle landed
  with CAP-22; this is authoring only: calibration curve, unknown
  concentration by value-claim. Acceptance: unknown determined within
  tolerance from absorbance alone.
  **Quest authored** (kero-basic): `spectrophotometry.toml` — event
  claims on `measured`, `diluted`, `observed`. Gap: no value claim on
  concentration yet (needs the Spectrophotometer instrument's output to
  be readable as a quantity — T2 candidate if wired).
- **EXP-38 — Curriculum paths and progress layer** — quests tagged
  with public-curriculum labels (CBSE/ICSE/IGCSE/NGSS classes) so a
  learner picks a path; progress, study-queue nudges, and post-quest
  quizzes ride the quest engine state and the codex predict machinery
  (quizzes already exist there in substance). GUI owns the surface;
  the engine side is tags + state queries on EXP-0.

## Declined from this corpus (recorded, with reasons)
- 2D NMR interpretation, protein–ligand docking, HOMO–LUMO/MO
  visualisation, retrosynthesis planning: computational and quantum
  chemistry beyond the bench's subject — the engine computes wet
  chemistry it can stand behind, and these would be a different
  product pretending to be this one.
- VR/multiplayer surfaces: GUI-workline territory if ever; not
  engine tasks and not declined chemistry.
- Water-system phase rule (triple point P–T manipulation): parked as
  investigate-only — the states machinery covers bench pressure;
  full P–T phase-diagram control is a real model decision, not a row.

---

# Part 5: the guided-practical corpus, classes 9–12 (audit 2026-08-24)

A fourth corpus (publicly funded national virtual-lab collection; not
named, as no source is). The convergence is the finding: after
mapping every experiment against EXP-0..38, nearly all of it is
covered — pH, titration, EMF, displacement single and double,
thermochemistry and neutralisation enthalpy, saponification and soap
comparisons, esterification, alcohol oxidation (CAP-23 rung 2 IS
their oxidation practical), separations, MP/BP, sublimation,
crystallisation and hydrate water, colloids, filtration, evaporation,
conductivity-as-electrolytes, equilibrium shifts (cobalt is EXP-28;
iron–thiocyanate becomes a quest on EXP-27+28 and is noted there),
sealed-unknown identifications (bleaching powder, washing vs baking
soda are EXP-30 quests), and the thiosulfate–acid kinetics clock,
which is already a curated rate law in kinetics.rs. What follows is
only the genuinely new remainder.

## New tasks

- **EXP-39 — Redox titrimetry** — the titrate verb targets pH only;
  redox titration needs endpoint modes: self-indicating permanganate
  (colour persists past equivalence) and potentiometric (pe from the
  solver the engine already computes). Oxalic acid joins the registry
  as the classic primary standard. Acceptance: KMnO4 standardised
  against oxalic acid to a value-claim; endpoint within one drop;
  both endpoint modes tested.
- **EXP-40 — Biomolecule assays** — the food-test canon: reducing
  sugars (Fehling/Benedict class), proteins (Biuret), starch (Lugol,
  already in EXP-14), fats (grease-spot/emulsion tie to EXP-10).
  Curated test rows with colour outcomes over real registry species
  (glucose from EXP-9/17 data). Acceptance: sealed food-sample quests
  identify composition from test patterns.
- **EXP-41 — Organic qualitative analysis** — functional-group wet
  tests paired with what the org crate can already do:
  `perceive_groups` computes the groups from structure, the curated
  wet tests (carbonyl, alcohol, acid, amine classes; Lassaigne-class
  elemental detection for N/S/halogens) confirm them — the pairing
  makes every test verdict checkable against a computed perception.
  Acceptance: an unknown organic narrowed by tests alone, each test
  backed by curated chemistry + the perception cross-check. HARDER.
- **EXP-42 — Preparative chemistry pack** — the classic preparations:
  double salts by crystallisation (Mohr's-salt/alum class, riding
  EXP-33's hydrate bookkeeping), gas preparation with property study
  (SO2 class, riding EXP-31's tests), amphoteric aluminium (Al +
  NaOH → aluminate + H2 beside Al + HCl — the two-front metal),
  azo-dye class synthesis on the org machinery (EXP-36 family).
  Acceptance: each preparation conserves exactly, yields honestly,
  and its product survives an identification test from EXP-30/33.
- **EXP-43 — Clock kinetics** — the iodine-clock class (iodide +
  peroxide; iodate + sulfite) joining the landed thiosulfate clock:
  curated rate laws with the sudden visual endpoint, concentration
  and temperature sweeps via `kero study`. Acceptance: clock time
  scales with concentration and temperature as the rate law says;
  the sweep reproduces the classic linearisation.
  **DONE** (2026-08-24): two curated rate laws (iodide–peroxide,
  iodate–bisulfite Landolt), 4 new registry species, safety rows,
  SMILES, 12 tests all green.
  **Quest authored** (kero-basic): `iodine-clock.toml` — event claims
  on `added`, `reacted`, `observed`, `measured`, `temperature_changed`.
- **EXP-44 — Excess enthalpy of mixing** — the
- **EXP-44 — Excess enthalpy of mixing** — [x] **first half done
  2026-08-24** (Fable). hᴱ = −RT²·Σxᵢ·∂lnγᵢ/∂T from UNIFAC's own
  temperature dependence (central difference, step stated), wired
  into the bench as a STATE FUNCTION: the vessel stores its total Hᴱ
  and each settle releases/absorbs only the difference, so one pour
  or five reach the same temperature to machine precision — proven,
  after the first attempt failed for a real thermodynamic reason
  (evaluating hᴱ at current T lets the path leak back in; the 25 °C
  reference is the stated model choice that restores exactness). The
  honesty core is the ALLOWLIST: acetone–water applies (its derived
  curve reproduces the literature S-shape); ethanol–water is
  WITHHELD, because this parameter set inverts the dilute-end sign
  and a wrong sign taught with confidence is worse than a stated gap
  — the thermo suite pins the deviation so a parameter upgrade
  (modified-UNIFAC/T-dependent aₘₙ) reopens the question loudly.
  Remaining: the acetone–chloroform pair (needs CCl-group growth —
  agent data task), the mixing-calorimetry quest. Original scope
  follows. — the
  acetone–chloroform-class negative deviation: h^E from the
  temperature dependence of UNIFAC activity coefficients
  (h^E = -RT² Σ xᵢ ∂ln γᵢ/∂T). Needs chloroform-class groups added
  to the UNIFAC table (CAP-18 growth) with sources. Acceptance: the
  mixing calorimetry quest shows warming for the associating pair
  and cooling for a positive-deviation pair, both computed. HARDER.
- **EXP-45 — The conservation quest** — the law of conservation of
  mass in a sealed reaction, weighed before and after. Pure
  authoring: the ledger IS the engine's thesis, and this quest is
  the product stating its own soul to a learner. Acceptance: at
  least three reaction types (precipitation, gas-in-sealed-flask,
  neutralisation) each balance on the vessel balance to the digit.
  **Quest authored** (kero-basic): `conservation.toml` — sealed-vessel
  mass balance, event claims on `vessel_sealed`, `reacted`,
  `precipitated`, `measured`, `gas_evolved`. Gap: no value claim on
  mass_g yet (T2 candidate — target = initial mass ± 0.001).

## Declined from this corpus
- Rutherford scattering and periodic-table study: physics
  demonstration and reference material, not bench chemistry — the
  GUI's interactive table already serves the second.

## Registry state after four corpora
EXP-0..45. Yield per corpus: 16 → 9 → 7 — the registry is converging
on the actual span of school and early-university wet chemistry,
which is the strongest evidence yet that the bench's subject is
finite and coverable.

---

# Part 6: directories, simulations, and the university tail (2026-08-24)

A fifth paste, different in kind: mostly **directories of
collections** — catalogs pointing at other catalogs. By construction
those add nothing: their contents are the corpora already audited
above or the public curricula they index. Audited for real: the five
simulations the directory highlights, and three university organic
experiments from an academic teaching collection (no source named,
as ever). The yield curve completes: 16 → 9 → 7 → **1**.

## Where the highlighted simulations land
- Acid–base strong/weak with pH probe and conductivity: covered
  (EXP-22; both instruments landed).
- Molarity exploration: covered (EXP-17).
- Balancing-equations game: the machinery is `kero balance` (null-
  space balancer, under-determined families stated) plus 103 balanced
  codex reactions to generate rounds from — folded into **EXP-38** as
  a quiz mode: strip coefficients from a codex reaction, learner
  balances, the balancer grades, the under-determined cases become
  the advanced rounds. Authoring, not building.
- Build-a-molecule and molecule-shapes (3D/VSEPR play): GUI-workline
  territory — the wasm structure panel already parses SMILES and
  perceives groups; 3D play is its call, not engine chemistry.
- The design lesson those simulations carry (game-like exploration,
  the invisible made visible at particle level) is already CAP-24's
  open-world DNA and the GUI's landed particle view.

## The one new task
- **EXP-46 — Cross-coupling template class** — the university tail's
  substance: modern C–C and C–N bond chemistry (biaryl coupling from
  aryl halide + boronate; amide C–N activation) as curated SMIRKS
  templates on the org machinery, proven at the molecule level like
  esterification, with the **catalyst as a required condition**
  (nickel-class species present or the reaction refuses) — and the
  boundary stated in every register: the bench books the
  transformation and its conditions; it does not simulate the
  catalytic cycle, and says so. Acceptance: two templates proven at
  molecule level; refusal without catalyst; boundary line in lv3;
  green-chemistry framing left to the quest prose. HARDER (org).
- Spectroscopy-interpretation coursework at the same tail: already
  declined in Part 4 (instrument-interpretation is not wet-bench
  chemistry); the decline holds consistently.

## Registry state after five corpora
EXP-0..46, yield 16 → 9 → 7 → 1. The span is effectively closed:
new corpora now audit into coverage confirmations, GUI affordances,
and quest authoring rather than new chemistry. The build order
stands: EXP-0 unlocks everything; the tiers fan out behind it.

---

# Part 7: the commercial simulation set (~57 items; 2026-08-24)

Sixth corpus, a commercial vendor's chemistry simulations (not named).
This one leans physical-chemistry where the earlier corpora leaned
wet classical, so the yield ticks up slightly: **three** new numbers.
Yield curve: 16 → 9 → 7 → 1 → 3.

## Covered (the bulk, compressed)
Stoichiometry/dimensional analysis, balancing games, limiting
reactants, moles-and-balance (EXP-17/20/38 + `kero calc`); density
by displacement, by comparison, and as an intensive property, plus
the counterfeit-coin forensic framing (EXP-18 quests); freezing-point
of brine (landed and swept); solubility-vs-T (EXP-24); calorimetry,
specific heats, hot/cold packs on the 14 curated dissolution
enthalpies, reaction energy (EXP-21; NH4NO3 joins the data list for
the cold-pack quest); pH of household substances (EXP-22 + household
data rows); titration with indicator choice (landed —
`indicator.rs` computes colour from pH, it does not script it);
mystery-powder identification (EXP-30/40 sealed unknowns, household
variant noted); nutrient tests (EXP-40); gas laws on the piston
machinery (`sealed-gas.lab` heritage); gas-phase equilibrium shifts
by concentration and pressure (headspace + solver; authoring);
collision theory with catalyst/surface/temperature — the engine has
`effective_activation_energy` and the `grind` verb; heating curves
and melting-point apparatus (EXP-33); conduction between vessels
joins EXP-7's scope as the conducting-link variant; the marine
carbonate-saturation scenario (shell erosion) is PHREEQC home ground
— saturation indices are computed today — and joins EXP-29's
scenario family; Joule-style mechanical-heat conversion noted under
EXP-21 with the energy input booked honestly.

## New tasks
- **EXP-47 — Colligative pack** — the four classic colligative
  properties as one computed family: vapour-pressure lowering
  (Raoult machinery exists in thermo), boiling-point elevation,
  freezing-point depression (landed; joins the pack), osmotic
  pressure (van't Hoff) with a semipermeable membrane link between
  two vessels as the one new mechanism. Acceptance: all four scale
  with particle molality including the van't Hoff factor for
  electrolytes (the speciation the solver already computes is the
  particle count — no fudge factors); membrane flow equilibrates
  honestly.
- **EXP-48 — Interfacial properties** — surface tension and
  capillarity as curated per-liquid data with computed capillary
  rise; cohesion/adhesion contrasts (water vs ethanol vs hexane are
  all on the shelf). Acceptance: curated values sourced; capillary
  rise computed from them; the soap quest (EXP-10) gains the
  surface-tension drop as a measurable.
- **EXP-49 — The nuclear bench** — [x] **done 2026-08-24** (Fable),
  first slice. The teaching set (C-14, I-131, Rn-222/α, Co-60,
  Tc-99m/γ, and the real Sr-90 → Y-90 → Zr-90 chain; NUBASE2020
  half-lives and masses) lives in a curated table; `add v1 I-131
  1e-9mol` routes El-A notation to the vessel's tracer-scale
  `NuclideLedger` (chemically inert, boundary stated) with a
  radioactivity hazard warning; decay runs inside `wait` beside
  kinetics on the shared clock; the Geiger counter reads total Bq.
  The invariant is the point: elements do NOT conserve across
  `Decayed` events — nucleons do, exactly, because α parcels keep
  their He-4 in the ledger; β/ν departures and the mass defect are
  stated boundaries in every lv3 line. The metastable flag keeps
  Tc-99m distinct from Tc-99 (found by the test that would have made
  the γ transition a ledger no-op). Acceptance held: half-life
  recovered from the activity series to 3 decimals over three
  half-lives; every curated equation balances A and Z in a test that
  reads the table itself; the chain propagates; uncurated nuclides
  refuse with the shelf listed (`tests/nuclear.rs`). Remaining rungs:
  decay-series depth (Bateman), codex radioactivity concept family,
  the half-life quest (EXP-0 authoring). Original scope follows.
  `nuclide.rs` has nuclides, decay
  chains, half-lives, and activity in becquerels, built and unwired
  (CAP-22 recorded the wait; this is its task number). Scope: decay
  as first-class bench chemistry — sealed sample, activity
  measurement, half-life determination from a time series via
  `wait`, balanced nuclear equations checked the way `kero balance`
  checks chemical ones, alpha/beta/gamma bookkeeping. Acceptance:
  half-life recovered from computed activity decay within tolerance;
  nuclear equations balance by mass number and charge; the codex
  gains the radioactivity concept family. HARDER (wiring, not
  physics — the physics is in the ledger already).
  **Quest authored** (kero-basic): `half-life.toml` — event claims on
  `nuclide_spiked`, `measured`, `decayed`. Gap: no value claim yet
  (would need a half-life or activity quantity type not in the current
  Quantity enum).

## Declined from this corpus
- Biology set (DNA/RNA synthesis, karyotyping, cell-context osmosis,
  water cycle, disease scenario): not bench chemistry; the physical
  half of osmosis lives in EXP-47.
- Atomic-structure interactives (element/isotope builders, electron
  configuration, Bohr spectra, periodic trends, bond/VSEPR/polarity
  builders): reference and visualisation, GUI-workline territory —
  the engine's flame colours and curated spectra remain its honest
  spectroscopy; modelling photon-level atomic transitions is not the
  bench's subject.
- Meteorology (relative humidity, dew point): weather, not the
  beaker — though its vapour-pressure heart is EXP-47's machinery,
  which the decline note says.

## Registry state after six corpora
EXP-0..49. The scenario-framing this vendor does well (forensic
cases, role-taking) is already the quest engine's prose register —
what they script, EXP-0 makes emergent.

---

# Part 8: the university practical set (2026-08-24) — yield zero

Seventh corpus (a commercial 3D-lab vendor; not named): organic
preparations, inorganic radical tests, analytical titrimetry. Every
item audits into an existing EXP number — the first zero-yield
corpus, which is what convergence looks like when it completes.
Curve: 16 → 9 → 7 → 1 → 3 → **0**.

## Fold-ins (scope refinements, no new numbers)
- **EXP-30 gains its ion roster** from this corpus's radical tests:
  cations NH4+, Na+, K+ (flame — landed event), Ca2+, Ba2+, Mg2+,
  Mn2+; anions SO3^2-, Br-, I- joining the halide/sulfate/carbonate
  set. The acceptance's "six unknowns" now draws from this roster.
- **EXP-39 gains iodometry**: thiosulfate standardisation with the
  iodine/starch endpoint as a third endpoint mode beside
  self-indicating permanganate and potentiometric. The starch-iodine
  colour is EXP-14's indicator chemistry reused.
- **EXP-36/42 template family grows** by named substrates and one
  named class: aspirin and paracetamol (acetylation of phenol/amine
  substrates), naphthyl acetate, ethyl-propionate Fischer row
  (propanoic acid joins the data list), and the condensation class
  (mixed-aldol/Claisen–Schmidt) as one more molecule-proven SMIRKS
  template — same capability, more rows.
- **EXP-46 gains the Grignard star**: organometallic reagent
  formation with the anhydrous condition enforced by machinery we
  already have — `nonaqueous::single_organic_solvent` gates it, and
  ANY water present makes the preparation refuse with the reason.
  The classic teaching moment (moisture kills the reagent) is not a
  scripted warning here; it is the computed verdict.
- **EXP-26 notes the sulfate variant** (BaSO4 gravimetric route).
- Strong/strong and weak-base/strong-acid titrations, sulfuric-acid
  determination: covered (EXP-22/39, landed titrate machinery).
- GC/MS, NMR, IR "analysis" items: the separation half of GC is the
  landed chromatograph verb; spectral interpretation stays declined
  (Parts 4 and 6), consistently.

## Registry state after seven corpora
EXP-0..49, unchanged. A corpus that adds only rows to existing tasks
is the audit series' success condition: the capability map is stable;
what grows now is data, templates, and quests.

---

# Part 9: the scenario-simulation catalog (~180 items; 2026-08-24)

Eighth corpus, a large commercial scenario-simulation vendor (not
named). Tag inflation is the first finding: well over half the
"chemistry" items are biology, health sciences, microbiology, or
physics — those get a blanket decline as out of the bench's subject
(Gram stains, PCR, ELISA, blots, cell culture, anatomy, ecology,
evolution, Newtonian mechanics, optics, plate tectonics, reactor
physics; fission/fusion REACTORS are declined even though nuclear
DECAY is EXP-49). The chemistry core audits almost entirely into
coverage. Yield: three. Curve: 16 → 9 → 7 → 1 → 3 → 0 → **3**.

## Fold-ins worth recording (no new numbers)
- Bomb calorimetry (constant-volume, ΔU vs ΔH) joins EXP-21/35 —
  the sealed rigid vessel is landed machinery; the U-vs-H distinction
  is the lv3 line.
- CaCO3 thermal decomposition and the limestone cycle join EXP-2,
  whose scope generalises to "thermal decompositions".
- Gas thermometry to absolute zero joins the gas-law authoring set.
- Electroplating joins the landed electrolysis verb as a quest.
- TLC (Rf values), ion-exchange, size-exclusion and HPLC variants
  join EXP-8 as chromatograph modes — same partition physics, and
  the upstream EXCHANGE machinery serves the ion-exchange mode.
- Nomenclature training (name ↔ structure rounds) joins EXP-38's
  quiz modes riding the org stack's iupac module.
- Named organic tests (ceric ammonium nitrate, azo-dye amine test,
  litmus-for-acids) join EXP-41's row list; Sudan IV joins EXP-40.
- Kjeldahl nitrogen-to-protein joins EXP-40 as its quantitative arm.
- Eutrophication/wastewater nitrate-phosphate chemistry joins the
  EXP-29 scenario family — PHREEQC-native water chemistry.
- Tonicity/IV-drip framing joins EXP-47; pipetting technique and
  apparatus tours are GUI affordances; polymer-formation items join
  EXP-12's scope.
- Cement hydration (their concrete lab) is declined for now with the
  reason stated: real chemistry, but multiphase hydration kinetics
  is beyond the current bench — recorded so nobody re-litigates
  silently.
- Spectral interpretation (MS, NMR, IR, MALDI, GC-MS): declined for
  the fourth consecutive corpus. The decline is stable.

## New tasks
- **EXP-50 — Mechanistic selectivity rules** — the genuinely new
  organic capability in this corpus: substitution-vs-elimination
  outcome PREDICTION (SN1/SN2/E1/E2) by substrate class, nucleophile
  strength, and temperature, plus regiochemistry rules where they
  bind. Curated rule table with textbook provenance selecting among
  molecule-proven templates; conditions outside the table refuse out
  loud. Acceptance: the classic condition matrix reproduces textbook
  outcomes; changing one condition flips the product and the lv3
  line says which rule fired. HARDER.
  - Status (2026-08-24): branch `kero1/exp50-mechanistic-selectivity`
    landed. 6 selectivity rules (March ch.10), 5 product entries, 2
    substrates (bromoethane, tert-butyl bromide), 2 nucleophiles
    (NaOH strong, water weak), temperature threshold 80°C. 15 tests
    including 3 condition-flip tests and 2 mass-conservation checks.
    8 new species (including Br- for PHREEQC booking). Verb: `react
    v1 haloalkane`. Preflight --light clean; full workspace green.
- **EXP-51 — Enzyme kinetics** — Michaelis–Menten as a curated rate
  family with Km/Vmax and competitive vs non-competitive inhibition,
  riding the existing kinetics integrator and the catalase
  precedent; assayed spectrophotometrically (EXP-37 machinery).
  Acceptance: Lineweaver–Burk from `kero study` sweeps distinguishes
  the two inhibition mechanisms; parameters recovered within
  tolerance. (Fermentation-optimisation scenarios noted as an
  optional extension, not core.)
- **EXP-52 — Disposal and lab-practice quests** — waste routing as
  computed chemistry: neutralise before drain, never mix the
  oxidiser stream with organics, halogenated separate — the safety
  screen already computes the hazard verdicts; a curated disposal
  rule table turns clean-up into quests where wrong routing triggers
  the same screen that guards the bench. Acceptance: a clear-the-
  bench quest gradeable entirely by existing safety machinery plus
  the rule table.

## Registry state after eight corpora
EXP-0..52. The pattern holds: wet chemistry converges; what this
corpus adds beyond three tasks is scenario-framing volume — which is
quest prose, and EXP-0 is still the gate everything waits behind.

---

# Part 10: quest authoring coverage (audit 2026-08-25)

## Current state
32 quest TOML files in `quests/`, all passing `kero quest lint`.
18 authored 2026-08-25 (kero-basic); 5 pre-existing; 2 sealed-unknown
variants added 2026-08-29 with the T2 value-claim upgrades below;
7 EXP-30 salt-analysis quests added 2026-08-29 (six single sealed
unknowns and the first two-unknown capstone).

## Quests authored (by EXP number)
| EXP | File | Claim types | Gap |
|---|---|---|---|
| 1 | magnet-sorting.toml | event | — |
| 2 | baking-powder.toml | event | no value claim on gas mass |
| 4 | water-filter.toml | event | — |
| 5 | stain-remover.toml | event | — |
| 12 | plastic-doctors.toml | event | — |
| 13 | vitamin-c.toml | event | no value claim on drop count |
| 14 | sweet-bread.toml | event | — |
| 16 | fizzy-drink.toml | event | — |
| 17 | solution-prep.toml | event + value | molarity:NaCl claim exists |
| 18 | density-id.toml | event + identify | sealed unknown (Cu) |
| 18 | the-grey-ingot.toml | event + value + identify | sealed unknown (Fe); mass_g on a counted 0.1 mol addition |
| 20 | limiting-reagent.toml | event + value | mass_g on the filtered AgCl (1.147 ± 0.05 g) |
| 21 | two-roads-one-temperature.toml | event + value | temperature_c claim landed |
| 22 | acid-base.toml | event + value | buffer ph claim landed |
| 22 | the-sour-unknown.toml | event + value + identify | sealed acid (CH3COOH) told from a strong one by pH 2.88 ± 0.4 |
| 24 | solubility.toml | event + value | molarity:Na+ at saturation (6.10 ± 0.05 mol/L) |
| 25 | redox-ordering.toml | event | — |
| 26 | gravimetric.toml | event + value + identify | mass_g claim and sealed unknown landed |
| 30 | the-blue-salt.toml | event + identify | sealed CuSO4; hydroxide colour + BaSO4 |
| 30 | the-green-salt.toml | event + identify | sealed FeSO4; Fe(OH)2 with Fe(OH)3 absent |
| 30 | the-bitter-salt.toml | event + identify | sealed MgSO4; candidate list excludes Zn (no amphoterism) |
| 30 | the-fizzing-salt.toml | event + value + identify | sealed Na2CO3; ph 11.26 ± 0.6, gas_contained + limewater |
| 30 | the-rust-maker.toml | event + value + identify | sealed FeCl3; hydrolysis ph 1.92 ± 0.5, Fe(OH)3, AgCl |
| 30 | the-heavy-salt.toml | event + identify | sealed BaCl2; inverted sulfate test + flame_test (apple green) |
| 30 | two-white-jars.toml | event + identify ×2 | capstone: TWO sealed unknowns (Na2SO4 + NaCl), any interleaving |
| 31 | gas-tests.toml | event | — |
| 35 | alcohol-burn.toml | event + value | temperature_c on the ethanol flame (2496.3 ± 2.0 °C); methanol does not ignite — no CEA mapping |
| 37 | spectrophotometry.toml | event | — |
| 43 | iodine-clock.toml | event | — |
| 45 | conservation.toml | event + value | mass_g on the sealed vessel (206.06 ± 0.01 g), unmoved by the reaction |
| 49 | half-life.toml | event | needs activity quantity type (not in enum) |
| — | the-white-unknown.toml | all three | demo quest; exercises every feature |

## Quests NOT authored — blocker summary
Quests that could not be written as TOML because the engine lacks the
required chemistry, data, or model:

| EXP | What blocks the quest | Blocker class |
|---|---|---|
| 3 | IR-absorbance per gas + lamp heating-rate model | HARDER (model) |
| 7 | U-value Newton cooling model | HARDER (model) |
| 8 | ink-dye species with partition coefficients | NEAR (data) |
| 9 | photosynthesis curated reaction + glucose species | HARDER (model+data) |
| 10 | fat/oil species + emulsification demo | NEAR (data) |
| 15 | clay/sand/silt column materials | HARDER (data) |
| 19 | sucrose-water density correlation (ethanol-water landed) | NEAR (data) |
| 23 | custom weak-acid route (KHP) | HARDER (engine route) |
| 27 | 1:1 association-K solver | HARDER (model) |
| 28 | speciation→colour coupling | HARDER (model) |
| 29 | arsenic-series registry rows | NEAR (data) |
| 30 | ~~qualitative inorganic test scheme (6+ salts)~~ — **authored 2026-08-29**: 6 single unknowns + a 2-unknown capstone | done |
| 32 | particle-size classification + Tyndall flag | NEAR (data+model) |
| 33 | MP/BP instrument + sublimation phase route | NEAR (instrument) |
| 34 | curated slow iron oxidation (water+O₂ gated) | NEAR (data) |
| 36 | org synthesis templates (salicylic acid) | HARDER (org data) |
| 38 | curriculum path tags + progress layer | infra (not a quest) |
| 39 | redox titration endpoint modes | NEAR (engine extension) |
| 40 | curated food-test rows (Fehling, Biuret) | NEAR (data) |
| 41 | functional-group wet tests | HARDER (data+coupling) |
| 42 | preparative chemistry data (double salts, etc.) | NEAR (data) |
| 44 | CCl-group growth + mixing quest | HARDER (UNIFAC growth) |
| 46 | cross-coupling SMIRKS templates | HARDER (org) |
| 47 | semipermeable membrane mechanism | HARDER (model) |
| 48 | surface tension + capillarity data | NEAR (data) |
| 50 | SN1/SN2/E1/E2 selectivity rule table | HARDER (model) |
| 51 | Michaelis–Menten rate family | HARDER (model) |
| 52 | disposal rule table | NEAR (data) |

## T2 value-claim upgrades — [x] **done 2026-08-29**
All seven candidates now carry value claims, every target read off the
solved state and every band verified by a hit and a miss run in the
REPL:
1. **conservation.toml** — mass_g on v1, 206.06 ± 0.01 g (the balance's
   own readability; the four pre-seal addition orders spread by
   1.1×10⁻³ g and the reaction moves it by 1.0×10⁻⁴ g, so the doc's
   original ± 0.001 would have been a corridor)
2. **acid-base.toml** — ph on v1, 4.76 ± 0.5 (landed earlier)
3. **limiting-reagent.toml** — mass_g on v1, 1.147 ± 0.05 g
4. **gravimetric.toml** — mass_g on v1, 1.433 ± 0.15 g + sealed unknown
   (landed earlier)
5. **alcohol-burn.toml** — temperature_c on v1, 2496.3 ± 2.0 °C. The
   engine's ethanol flame temperature is intensive (0.01 mol, 0.05 mol
   and 1 g all settle there) while the energy is extensive, so the
   claim reads the flame, not a calorimetric ΔT into water: combustion
   under liquid water is declined by design, and **methanol has no CEA
   mapping at all** (`ignite` answers "no wired solver models
   combustion for them"), so the quest's methanol-vs-ethanol framing
   is not yet reachable
6. **two-roads-one-temperature.toml** — temperature_c 26.2 ± 1.5
   (landed earlier)
7. **solubility.toml** — molarity:Na⁺ on the saturated vessel,
   6.10 ± 0.05 mol/L at bench temperature (pinned by Ksp: 50 g and
   80 g in 100 mL, 120 g in 250 mL and the evaporation route all land
   inside the band; warming to 66 °C moves it to 6.39)

Two sealed-unknown variants were authored alongside:
**the-sour-unknown.toml** (a weak acid told from a strong one by the
pH of a 0.1 mol/L solution) and **the-grey-ingot.toml** (a grey metal
named from the mass of a counted 0.1 mol addition — the balance as a
molar-mass instrument, since the engine has no density quantity).

---

# Part 11: breadth-programme handoff (2026-08-27)

The eight-corpus audit found the demand; `BRD-000` turns it into the versioned
500-prompt curiosity regression corpus. The following mapping is normative for
agents: complete the shared BRD prerequisite once, then author multiple EXP
quests against it rather than implementing compound-pair exceptions inside a
quest.

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

Quest acceptance remains stricter than breadth coverage: a known substance or
installed visualization does not make an experiment complete. Each EXP still
needs its own observable success/failure claims, controls, model boundary,
lesson/codex content and replay tests. Conversely, `BRD-100` cannot close while
an uncovered curiosity prompt points at an unowned EXP-level behavior.

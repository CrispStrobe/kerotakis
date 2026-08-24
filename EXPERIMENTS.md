# The sixteen classroom experiments — audit and plan (CAP-24)

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

- **EXP-0 — Quest engine** (Fable; everything below depends on it).
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
- **EXP-2 Backpulver** — curated thermal decomposition
  2 NaHCO3 →Δ Na2CO3 + H2O + CO2↑ (threshold ~50–100 °C stated with
  source); quest links the fizz route and the heat route as two paths
  to the same gas. Acceptance: heating dry NaHCO3 evolves CO2 into a
  sealed headspace; limewater from the existing lesson detects it.
- **EXP-3 Treibhausgase** — per-gas IR-absorbance data + lamp
  heating-rate model on `Irradiate`; quest compares CO2 vs air vs
  water-vapour bottles. Acceptance: computed warming curves differ by
  gas with sources; `kero study` sweeps concentration.
- **EXP-4 Wasserfilter** — dirt species (suspended solid, appearance)
  + multi-stage filter quest. Acceptance: turbidity falls stage by
  stage; dissolved salt passes and the quest says why.
- **EXP-5 Fleckenteufel** — dye species + curated hypochlorite
  bleaching; quest compares oxidant vs oxidant-free wash. Acceptance:
  dye colour is bleached only with NaOCl; the colour-safe wash keeps
  it; three registers say the mechanism.
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
- **EXP-13 Vitamin C** — ascorbic acid species + curated iodine
  decolorisation + starch indicator. Acceptance: titration-style
  counting of drops to endpoint works; juice-vs-water contrast.
- **EXP-14 Amylase** — amylase/starch/maltose species + curated
  enzymatic hydrolysis + Lugol assay. Acceptance: starch negative
  after enzyme+time+warmth, positive without; the sweetness line at
  lv1 is the maltose the ledger shows.
- **EXP-15 Boden** — clay/sand/silt column materials with retention
  parameters on the landed CellChain. Acceptance: percolation-time
  and retention orderings match the curated data; three-column
  comparison quest.
- **EXP-16 Sprudel** — quest authoring on existing CO2/limewater
  chemistry. Acceptance: the quest completes via at least two paths
  (warming the bottle vs shaking-analogue vs acid+carbonate).

## Part-2 capability classes (our own problems, written from scratch)

- **EXP-17 Solution-prep quest pack** — dilution ladders, stock from
  solids, target-molarity value-claims. Needs: EXP-0 only.
- **EXP-18 Density identification** — graduated-cylinder/displacement
  instrument + sealed-unknown metal and liquid quests. Needs: EXP-0.
- **EXP-19 Mixture-density data** — curated ethanol-water (then
  sucrose-water) density correlations with sources; unlocks
  concentration-from-density quests.
- **EXP-20 Limiting-reagent pack** — precipitation and gas routes,
  predict-then-check quests with value claims.
- **EXP-21 Thermochemistry pack** — reaction enthalpy, Hess
  three-path demonstration, mixing-temperature and unknown-heat-
  capacity quests. The engine side is DONE (tested invariants);
  this is authoring.
- **EXP-22 Acid-base pack** — pH ladder by successive dilution, weak
  acid problems, buffer design to a target ratio, titration-to-pKa
  with the curve read at half-equivalence.
- **EXP-23 Standardisation class** — potassium hydrogen phthalate (or
  an equivalent primary-standard acid) added from primary data; the
  custom-weak-acid route into the engine; 4-significant-figure
  discipline via burette precision. HARDER.
- **EXP-24 Solubility pack** — Ksp determination, solubility-vs-T
  with predict-then-test at a third temperature (value claim).
- **EXP-25 Redox-ordering quest** — design-your-own-experiment over
  the landed displacement chemistry; completion = correct ordering
  of Cu/Mg/Zn/Pb by any valid route.
- **EXP-26 Gravimetric pack** — precipitate, filter, dry, weigh;
  sealed-unknown AgNO3 concentration by mass. Needs: EXP-0.
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
  school analytical canon; "salt analysis"). The INST-008
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
- **EXP-31 — Gas tests** — pop (H2), glowing splint (O2), limewater
  (CO2, exists), damp litmus (NH3) as curated test actions on the
  headspace, each an event with three registers. Acceptance: the four
  classic gases each identified from a genuinely evolved headspace.
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
- **EXP-44 — Excess enthalpy of mixing** — the
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
- **EXP-49 — The nuclear bench** — `nuclide.rs` has nuclides, decay
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

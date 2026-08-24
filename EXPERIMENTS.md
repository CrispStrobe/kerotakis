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

A second corpus, user-supplied: ~40 problems from a public NSF-funded
university virtual-lab collection, spanning stoichiometry through
analytical chemistry. **License boundary first:** that collection is
CC BY-NC-ND — its problem texts and scenarios must never be copied or
adapted into this repo. What is audited here is the *capability
class* of each problem; our own problems get written against those
classes from scratch.

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

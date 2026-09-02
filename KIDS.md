# Part 1: the thirty-experiment children's corpus (audit 2026-09-02)

Thirty experiments a child actually meets — the volcano, the elephant
toothpaste, the naked egg, the red-cabbage rainbow — driven through the
shipped bench as a stranger would drive it, and the register of what came
back. The audits in `EXPERIMENTS.md` were written from curricula and
product catalogs; this one starts from the other end, at a kitchen table,
and asks whether the shelf, the verbs and the docs let a newcomer *reach*
the chemistry the engine already computes.

The corpus is ours: thirty activities named from the shared public
vocabulary of children's science, written from scratch as `.lab` scripts.
No external collection's text, ordering or data is used, here or anywhere
in this file.

## How it was run

One `kero` binary built from `main`, thirty `.lab` scripts, no lesson
files, no prior knowledge beyond `README.md`, `kero --help` and the REPL's
`help` — the three surfaces a new user actually has. Each script was
written the way the docs suggest, run, and then *repaired* only where the
repair taught us something: a name that could not be typed, a verb that did
not exist, a vessel that had to be created first.

The first pass is itself a finding. **Thirteen of the thirty scripts died on
the first or second line**, every one of them on a name — `household_vinegar`,
`ground_pepper`, `dilute_Lugol_solution`, `red_cabbage_indicator`, `pva_glue`.
Eleven of those thirteen substances *are* in the registry; they simply could
not be reached from a terminal. That is finding KID-1, and it is not a
chemistry gap at all. With KID-1 landed the same unrepaired scripts run
27/30, and the three that remain are genuine content gaps.

The verdict matrix below is the *repaired* run: names corrected, so that
each row reports on the chemistry rather than on the vocabulary. It was
taken against `origin/main` at `115f0a1` and is byte-identical on two
independent builds. The scripts themselves land as shipped lessons under
KID-16, once the rows they exercise stop lying — a lesson that ships a
silent miss teaches the silence.

## Verdict classes

| Class | Meaning |
|---|---|
| **computed** | The observation a child watches for comes out of the solvers. |
| **partial** | The chemistry lands; the thing the child is actually looking at does not. |
| **honest miss** | The bench says, in words, that it cannot model this. The rule holds. |
| **silent miss** | Nothing happens and nothing says so. The rule breaks. |
| **wrong** | The bench asserts something untrue, or a shipped claim does not reproduce. |
| **unreachable** | An ingredient or verb the activity needs does not exist, or cannot be typed. |

## Verdict matrix

| # | Experiment | What the child is watching for | Verdict | What the bench did |
|---|---|---|---|---|
| K01 | Volcano (soda + vinegar) | the eruption | partial | CO₂, pH 9.70, mass and the cooling-then-warming are all computed; there is no foam and no overflow, because foam exists only on the peroxide path |
| K02 | Balloon on a bottle | the balloon filling | computed | sealed 500 mL headspace reaches 1.875 bar; `regulate` gives a real expanding boundary (766 mL) — but nothing in the docs calls that a balloon |
| K03 | Limewater breath test | milky, then clear | computed | Ca(OH)₂ → calcite → redissolution, exactly as the shipped lesson promises |
| K04 | Snuff a candle with CO₂ | the flame going out | **silent miss** | `add v1 candle_wax` reports "this part of the lab isn't awake yet"; `ignite v1` then emits **no line at all**; CO₂ blanketing a flame is unmodelled |
| K05 | Elephant toothpaste | the foam column | computed | rate law, catalase gate, oxygen yield, reaction heat, foam climbing out of the vessel. Best-in-class |
| K06 | Magic milk | colours racing | computed | `SurfaceColourSpread` fires; the stirred control correctly does not |
| K07 | Pepper runs from soap | the darting | computed | fires on the soap-second order and stays silent on the soap-first control |
| K08 | Oil and water | two layers | computed | layer forms; the dye stays in the aqueous phase |
| K09 | Lava lamp | rising blobs | partial | layers and fizz computed; blob motion is a stated non-goal |
| K10 | Density tower | three stacked liquids | partial | immiscible layering is computed; a *miscible* sugar-syrup tower has no stratification model, so vinegar correctly mixes and there is no third band |
| K11 | Dancing raisins | raisins rising and falling | unreachable | no raisin, and no model for a bubble attaching to an object and lifting it |
| K12 | Red-cabbage rainbow | pink → purple → green | unreachable | no anthocyanin, in the materials *or* the indicator table (which holds only phenolphthalein, methyl orange, bromothymol blue) |
| K13 | Invisible ink | brown writing appearing | **wrong** | no pyrolysis and no browning; worse, 5 kJ into juice-on-paper drives the vessel to **670 °C with liquid water still in the ledger**. At lv3 two `NOT MODELLED` lines name both faults honestly; at lv1 the child is told "the water is boiling — look at the steam!" and the mass does not move |
| K14 | Naked egg | the shell vanishing | computed | CaCO₃ + acetic acid to completion, pH 4.60, gas out. No egg material and no membrane, so the osmosis half is out of reach |
| K15 | Rubbery bone | the bone bending | partial | real bone is calcium phosphate; only the chalk stand-in is modelled |
| K16 | Clean a copper coin | the shine returning | partial | CuO + acid + chloride computed (blue solution, Cu(II) speciated); there is no tarnish layer on a copper object, so the child has to dose copper oxide by hand |
| K17 | Rusting race | orange rust | **silent miss** | steel wool + brine + oxygen + 24 h leaves iron untouched; the only word is "this part of the lab isn't awake yet" |
| K18 | Hot pack / cold pack | the thermometer | partial | CaCl₂ gives +36 K, computed from dissolution enthalpy — exemplary. The canonical cold pack, NH₄NO₃, is not in the registry at all; Epsom salt reads 0 K |
| K19 | Salt crystals | cubes appearing | computed | evaporation precipitates halite with the ledger exact; crystal *habit* is not drawn |
| K20 | Rock candy | crystals on cooling | **wrong** | sucrose saturation is modelled (0.5843 mol per 100 mL) but is **temperature-independent** — identical at 20, 60 and 90 °C — so the one mechanism the experiment exists to show cannot happen |
| K21 | Slime | the slime | unreachable | no poly(vinyl alcohol), no borate |
| K22 | Oobleck | liquid that goes hard | honest miss | "this part of the lab isn't awake yet"; no suspension rheology |
| K23 | Plastic from milk | curds you can mould | ~~**wrong**~~ → computed | curdling **never fired with the aqueous solver on** (KID-2, fixed 2026-09-02); `filter v1 v2` still refuses because `v2` must be created first (KID-15) |
| K24 | Sherbet | fizz on the tongue | computed | the dry mixture correctly does nothing; water starts it; pH 3.83 |
| K25 | Bath bomb | waiting for water | computed | the dry/wet contrast is the whole lesson and it lands |
| K26 | Felt-tip chromatography | the colours separating | honest miss | "nothing dissolved here has a curated UNIFAC decomposition, so the column's method is silent"; no paper/Rf mode, no dye partition data |
| K27 | Starch hunt (iodine) | blue-black vs brown | computed¹ | both vessels right — and both preceded by a Danger-level banner saying the mixture "can detonate" |
| K28 | Vitamin-C detective | blue-black going clear | computed¹ | decolourisation computed, dehydroascorbic acid in the ledger; same false banner |
| K29 | Yeast balloon | the balloon filling | computed | fermentation, ethanol, CO₂ — and then an honest **BANG** when the sealed vessel bursts. Correct, and a better lesson than the one asked for |
| K30 | Flame colours | one colour per metal | partial | Na yellow, K lilac, Sr crimson, Ba apple-green, Cu blue-green all computed. **Calcium — the one a child actually owns — reads "nothing happens."** Lithium is absent entirely |

¹ computed, but preceded by a false hazard banner. See KID-3.

**Tally at audit time: computed 13 · partial 7 · honest miss 2 · silent
miss 2 · wrong 3 · unreachable 3.** After KID-1 and KID-2: **computed 14 ·
wrong 2**, and the thirty unrepaired scripts run 27/30 instead of 17/30.

The engine is in far better shape than the corpus's first pass suggested.
What stands between a child and it is, in order: names they cannot find,
three shipped claims that do not reproduce, two silences that should be
sentences, and a hazard screen that cries wolf.

## What is missing, by class

### A. Ingredients

The registry holds 139 species and 50 named household/school materials.
The materials are the right ones for this corpus — vinegar, milk, dish
soap, yeast, cornstarch, Lugol, steel wool, candle wax, sand, chalk stick.
Three problems sit on top of them.

1. **Most English aliases cannot be typed.** `normalize_material_name`
   collapses whitespace and lowercases; it does not treat `_` and space as
   equivalent, while the `.lab` grammar splits on whitespace. So `household
   vinegar` — the *only* English alias the vinegar bottle carries — is
   unreachable by construction, and the canonical key
   `white_vinegar_5_percent` is nowhere a newcomer would look. Roughly
   thirty of the fifty recipes have this shape. `add v1 household_vinegar
   50mL` fails on `main` today.
2. **The batch surface still cannot see them.** `kero species` iterates
   `species::REGISTRY` only, and there is no `kero materials`. The browser
   bridge (`kerotakis-wasm::species`) appends `material::all()`, so the GUI
   shelf shows fifty bottles a scripted or piped session has no command to
   list.
3. **The error message closes the loop.** `unknown species or material
   'vinegar' (see 'species')` points at a command that cannot show what you
   asked about — and, crucially, *not* at `find`, which could.
4. **`find` already exists and nothing points at it.** `BRD-002` landed a
   real cabinet search in the REPL: `find vinegar` returns
   `white_vinegar_5_percent … homogeneous liquid in g`, exactly the answer
   a newcomer needs. It is mentioned once, in the `species` footer, after
   several hundred rows have scrolled past; it is absent from `kero --help`
   and from the REPL's own `help` line; and the error a newcomer actually
   hits sends them somewhere else. `cabinet.rs`'s own doc comment says a
   learner can now discover "that `vinegar` is a name the bench takes" —
   but `vinegar` is not a name the bench takes. `Essig` is.

Substances this corpus needs and the registry does not have: red cabbage /
anthocyanin, poly(vinyl alcohol) glue, borax, ammonium nitrate, egg (shell
and membrane), raisin or other dried fruit, lemon juice, gelatin, an
effervescent tablet, glycerol or honey for a density tower, lithium salt
for the flame series, casein as a named colloid fraction, and a paraffin
with combustion data rather than a purely unresolved wax.

**What the unreachable names had already cost us.** Landing KID-1 moved
exactly one row of the 500-question curiosity corpus, and it is the most
instructive row in it. `aq-120` — "can the model show an oil droplet
emulsion breaking after soap is added?" — was pinned as
`expected = "missing"`, `parse_boundary = "unknown_species"`,
`owning_task = "BRD-014"`, `tags = ["emulsion-model-gap"]`. But the
emulsion model is *landed*: with `cooking_oil` typeable, the same script
disperses the oil on `stir` and coalesces it back on `wait`, and the
corpus now reads `computed`. An alias nobody could type had been recorded
as a missing capability, filed against the task that had already built it.
A name that cannot be reached does not just block a learner; it lies to
the project's own coverage report.

### B. Mechanisms

| Missing mechanism | Costs us |
|---|---|
| Corrosion of iron in aerated brine | K17; a top-five children's experiment (`EXP-34` already on the registry) |
| Combustion of organic solids; a flame that can be starved | K04, K13; `ignite` is currently *silent* on an unresolved material |
| Latent-heat plateau at a boiling point | K13; named at lv3, invisible at lv1 |
| Temperature-dependent solubility of molecular solutes, and nucleation | K20; rock candy, supersaturation, seeding |
| Acid curdling wired to the solved ledger | K23; see KID-2 — this one is a bug, not a gap |
| Foam on any gas-evolving vessel with a declared surfactant | K01; the volcano's whole point |
| Suspension rheology; buoyancy on attached bubbles; miscible stratification | K22, K11, K10 |
| Paper/TLC mode with dye partition data (`EXP-8`) | K26 |
| Anthocyanin as a computed pH-dependent chromophore | K12 |
| Pyrolysis and Maillard browning | K13 |
| Calcium (and lithium) in the flame-colour table | K30 |
| Acetic acid in the odour table | `smell` on vinegar returns "no odour a careful waft detects" |
| Dose awareness in the L0 hazard screen | K27, K28 |

### C. Apparatus

Landed and used by this corpus: five glassware kinds (beaker, flask, tube,
cylinder, crucible), the twelve-tool apparatus palette (bunsen, stirrer,
heater, cooler, centrifuge, dilution, evaporator, electrolysis cell,
mortar, lamp, regulator, sweep) and twelve instruments. `magnet` works
exactly as `EXP-1` promises.

Missing pieces this corpus asks for, in the vocabulary of `APPARATUS.md`:

| Piece | Class | Note |
|---|---|---|
| Balloon | SKIN over `regulate` | the single most common children's gas vessel; the verb exists and nothing names it |
| Candle / spirit burner as an *object* with a flame | BEHAVIOR | needed before any "put the flame out" activity |
| Filter funnel that creates its receiver | SKIN | `filter v1 v2` fails with "no vessel v2"; a newcomer does not know `new` comes first |
| Chromatography paper strip + chamber | SKIN + BEHAVIOR(`EXP-8`) | |
| Spotting tile | BEHAVIOR(`EXP-30`) | the natural home for K27's food tests |
| Magnet | PROP card | verb landed, no cabinet card, and absent from `help` |
| Plastic bag / safe pressure vessel | SKIN over `regulate` | the child-safe alternative to K29's burst |
| Ice bath, drying rack | SKIN / PROP | |

### D. Documentation — the surface a stranger actually meets

This is where the audit was harshest, and it is the cheapest thing to fix.

- **`README.md` never names a single household material.** Every example
  is a formula key — `NaCl`, `AgNO3`, `Ca(OH)2`, `CO2` — which reads as a
  chemist's tool, not a bench with vinegar on it. There is no "first five
  minutes", no list of the 38 shipped lessons, and no statement that
  `species` and `help` are the discovery route.
- **The REPL's `help` is one screen and omits real verbs**: `magnet`,
  `smell`, `chromatograph`, `particles`, `regulate`, `sweep`, `transport`,
  `irradiate`, `distil`'s stages, and the quest commands. `kero --help`
  covers more but still omits `magnet`, `smell`, `chromatograph`,
  `particles` and `find`.
- **The lessons are invisible.** Thirty-eight `.lab` files ship; no command
  lists them, so `kero run lessons/fizz.lab` is only reachable by reading
  the repository.
- **The GUI is ahead of the terminal** on exactly the axis that matters
  here: its shelf is built from the same engine call that the CLI does not
  make.

## The task registry (KID numbers are stable identifiers)

Same laws as `EXP`/`CAP`/`OPT`/`BRD`: numbers are never re-bound, every
task carries acceptance, new species and materials go through the registry
pipeline (safety rows, exporter, golden regeneration, identity crosswalk),
and `main` moves only by PR.

### Wave 1 — the closed loops (nothing else is worth doing first)

- **KID-1 — The shelf, reachable from the terminal.** Four changes, all
  small, on top of `BRD-002`'s cabinet search rather than beside it:
  `normalize_material_name` treats `_`, `-` and space as one separator, so
  every alias a recipe advertises has a writable spelling; `kero materials`
  and `kero find <word>` expose the cabinet outside the REPL, where scripts
  and pipes live; the unknown-name error names `find` and offers the
  nearest key it actually holds; and `README.md` gains the ten household
  words that make the bench look like a bench.
  *Acceptance:* every one of the 50 recipes resolves from at least one
  whitespace-free spelling of every name it advertises, with a test that
  fails the day a recipe adds a space-only alias; `add v1 household_vinegar
  50mL` works; `kero materials` lists all 50 outside the REPL; the error on
  a misspelling suggests a real key; no two recipes and no recipe/species
  pair collide under the new normalization.
  **Landed 2026-09-02.** Measured on the thirty scripts as a newcomer first
  wrote them, with no name repaired: **17/30 ran before, 27/30 after.** The
  three that still fail are content, not naming — red cabbage and PVA/borax
  do not exist (KID-8, KID-14), and `filter v1 v2` still wants its receiver
  created first (KID-15). The curiosity corpus moved by exactly one row,
  `aq-120`, from `missing` to `computed`; `expectation_mismatches` and
  `baseline_drift` are unchanged at 151 and 0.

- **KID-2 — Curdling must fire with the solver on.** `curdling::observe`
  sums vessel contents whose species id equals the recipe's
  `acid_species` (`CH3COOH`). With the aqueous engine linked, the solver
  has already speciated that into `CH3COO-`, so the dose is always zero and
  the curdling event never fires. `crates/kerotakis-core/tests/milk_curdling.rs`
  passes because it runs the engine-free path; the shipped
  `lessons/milk-curds.lab` therefore does not demonstrate its own headline
  claim on the shipped bench.
  *Acceptance:* the lesson produces curds through the full solver stack; a
  test drives it through the same path the CLI uses, not `Bench::step`
  alone; the model reads total acid *inventory* (undissociated plus
  conjugate base) rather than one species key.
  **Landed 2026-09-02.** A Brønsted acid and its conjugate base differ only
  in hydrogen count and charge, so the dose now sums the species sharing the
  declared acid's non-hydrogen composition — which reproduces the number the
  recipe was calibrated against (10 mL of 5% vinegar is 0.008376 mol of
  acetate-equivalent whether the solver has deprotonated it or not), leaving
  the core's pinned figures untouched. `milk-curds.lab` now separates the
  ten-times vessel and leaves its control a colloid, exactly as its own
  prose promised, and `lessons_replay.rs` asserts both halves on the binary
  a reader runs. Curiosity corpus unmoved: 151 mismatches, 0 drift.
  **Stated boundary, and the follow-up it earns:** this counts acid
  *inventory*, not acidity. Sodium acetate in milk would read as a dose and
  real milk would not curdle — casein aggregates at its isoelectric point.
  Making the response pH-driven wants a reviewed pI datum on the recipe;
  that is registry work and belongs with **KID-14**, not smuggled in here.

- **KID-3 — A hazard screen that does not cry wolf.** 1 mL of 1 % Lugol
  into 100 mL of water raises "mixing a strong oxidizer with a reducing
  agent can cause a violent, potentially explosive reaction … at scale,
  such mixtures can detonate." Two faults: the L0 screen is dose-blind, and
  it screens a *recipe's own components against each other* — iodine and
  iodide are a stable pharmacy reagent, not a mixing hazard.
  *Acceptance:* components arriving from one `MaterialRecipe` expansion are
  not screened against one another; incompatibilities carry a dose or
  concentration floor below which the severity drops or the rule is silent;
  the starch and vitamin-C activities run clean; the genuinely dangerous
  pairs (bleach + acid, bleach + ammonia) still fire at household strength.

- **KID-4 — `ignite` is never silent.** Holding a flame to an unresolved
  material currently emits nothing at all. The project's own rule is that
  wherever the engine declines to model something it must say so.
  *Acceptance:* every `ignite` on every shelf item produces a line at lv1;
  a test enumerates the shelf and asserts a non-empty response for each.

### Wave 2 — the mechanisms the corpus is actually short of

- **KID-5 — Rusting.** Pull `EXP-34` forward: iron surface area, oxygen
  transport, chloride acceleration, Fe(OH)₂/Fe(OH)₃/Fe₂O₃ product routing,
  a rate slow enough to need `wait` and fast enough to see.
- **KID-6 — The boiling plateau.** Hold temperature at the (solute-shifted)
  boiling point while water leaves as steam; make the lv1 register say what
  lv3 already says when the aqueous model's 300 °C ceiling is passed.
- **KID-7 — Solubility with temperature, and crystallisation.** `s(T)` for
  molecular solutes starting with sucrose; supersaturation as a state the
  bench can report; seeded growth on `wait`.
- **KID-8 — Anthocyanin.** A red-cabbage material and an anthocyanin
  chromophore with pH-dependent spectra through the existing Beer–Lambert
  path, so the rainbow is *computed colour*, not a tinted lookup.
- **KID-9 — Paper chromatography.** `EXP-8`'s Rf mode plus partition data
  for the three shipped dyes and a black-ink surrogate.
- **KID-10 — Completions.** Calcium and lithium flame colours; acetic acid
  and ethanol odours; the missing cold-pack salt (NH₄NO₃) through the
  registry pipeline.
- **KID-11 — Foam is general.** A declared surfactant plus any gas-evolving
  reaction produces the existing foam observable, not only the peroxide
  path.
- **KID-12 — Combustion of organic solids.** Paraffin, paper and sugar with
  real combustion data; a flame that a gas blanket can starve; browning as
  a separate, honestly-bounded observable.

### Wave 3 — physical behaviours and the cabinet

- **KID-13 — Physical mixtures.** Suspension rheology (oobleck), buoyancy
  on attached bubbles (raisins), miscible stratification with a slow pour
  (density tower). Each may land as an honest bounded observable rather
  than a CFD claim, following the `magic-milk` precedent.
- **KID-14 — The children's materials pack.** PVA and borate, egg, raisin,
  lemon juice, gelatin, effervescent tablet, glycerol, tarnished copper.
- **KID-15 — The children's apparatus cards.** Balloon and plastic bag over
  `regulate`; candle as an object; a filter funnel that creates its
  receiver; paper strip and chamber; spotting tile; magnet card.

### Wave 4 — the surface

- **KID-16 — The thirty lessons and their quests.** Every row above that
  reaches *computed* becomes a shipped `.lab` lesson with a prediction and
  a stated boundary, and the strongest of them become quests.
- **KID-17 — Docs a stranger can start from.** README gains "your first
  five minutes" written entirely in household words; `kero lessons` lists
  what ships; the REPL `help` covers every verb; the unknown-name error is
  a signpost rather than a dead end.

### Sequencing

```
KID-1 ──┬── KID-17 (docs quote the new commands)
        └── KID-16 (lessons need typeable names)
KID-2, KID-3, KID-4   independent bug fixes; do them next
KID-5 … KID-12        independent of each other; KID-8 before KID-9's ink
KID-13 … KID-15       after their mechanisms land
```

Wave 1 is the whole difference between "a chemist's engine" and "a bench a
child can walk up to". Everything after it adds chemistry; only Wave 1 adds
*reachability*, and reachability is what the audit found missing.

## Part 2: the next thirty

Deferred until Wave 1 lands, and recorded here so the corpus does not
restart from zero: the second children's thirty (magnet sorting, plastic
float/sink, water filter, lemon battery, water electrolysis, copper plating
a nail, ice-cream in a bag, baking powder versus baking soda, hard water and
soap scum, a sugar-cube tower, growing copper sulfate crystals, chalk versus
vinegar rates, colour-changing lemonade, milk of magnesia, glow sticks and
temperature, a raw-egg-in-cola tooth analogue, apple browning and lemon
juice, corrosion of different metals, a homemade fire extinguisher, sun-print
paper, chalk chromatography, soap-powered boat, the leaf-and-light oxygen
test, a pH map of the kitchen, hand-warmer crystallisation, borax snowflakes,
salt versus sugar melting ice, invisible-ink variants, gas-test set, and a
mass-conservation weigh-in) — then the 12–16 slice, which is where the
titration, the rate law, the equilibrium shift and the electrochemistry the
engine already computes finally meet an audience that can use them.

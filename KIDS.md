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
| K01 | Volcano (soda + vinegar) | the eruption | ~~partial~~ → computed | CO₂, pH 9.70 and mass were always computed; the eruption is too, as of KID-11 (2026-09-04) — with washing-up liquid in the glass first, **foam 1.375 L, 49.1 cm high, 1.178 L over the rim**, and nothing at all in the no-soap control. **The earlier verdict was half true and the wrong half was the temperature** — see the finding below: in the order most people pour, the bench says the volcano gets *warmer*, and it gets colder |
| K02 | Balloon on a bottle | the balloon filling | computed | sealed 500 mL headspace reaches 1.875 bar; `regulate` gives a real expanding boundary (766 mL) — but nothing in the docs calls that a balloon |
| K03 | Limewater breath test | milky, then clear | computed | Ca(OH)₂ → calcite → redissolution, exactly as the shipped lesson promises |
| K04 | Snuff a candle with CO₂ | the flame going out | ~~silent miss~~ → computed | the last silent miss, closed by KID-12 (2026-09-03). Wax is paraffin now, it burns, and a jar over it puts it out **with 77% of the oxygen still in the jar** — the number that contradicts the sentence every child is taught. Carbon dioxide poured in first stops it lighting at all, without taking anything away |
| K05 | Elephant toothpaste | the foam column | computed | rate law, catalase gate, oxygen yield, reaction heat, foam climbing out of the vessel. Best-in-class |
| K06 | Magic milk | colours racing | computed | `SurfaceColourSpread` fires; the stirred control correctly does not |
| K07 | Pepper runs from soap | the darting | computed | fires on the soap-second order and stays silent on the soap-first control |
| K08 | Oil and water | two layers | computed | layer forms; the dye stays in the aqueous phase |
| K09 | Lava lamp | rising blobs | **declined** | layers and fizz are computed; the rising blobs are bulk fluid motion, which `BREADTH.md` places outside this engine's authority. Listed as *partial* this read as work pending. It is not pending — it is refused, and the matrix should not imply otherwise |
| K10 | Density tower | three stacked liquids | partial | immiscible layering is computed; a *miscible* sugar-syrup tower has no stratification model, so vinegar correctly mixes and there is no third band. KID-19a made the half that is data answerable — `measure v1 density` reads 1.00, 1.14 and 1.28 g/mL up a sugar ladder, which is the number a tower is built on; the slow pour that would keep them apart is still not modelled |
| K11 | Dancing raisins | raisins rising and falling | ~~unreachable~~ → computed | the last unreachable row, closed by KID-13 (2026-09-03). A raisin joins the shelf at 1.35 g/mL, and the bench computes what the experiment is actually about: attached bubbles worth **35% of the raisin's own volume** lift it out of water, and only 11% out of sugar syrup |
| K12 | Red-cabbage rainbow | pink → purple → green | ~~unreachable~~ → computed | no anthocyanin existed, in the materials *or* the indicator table (KID-8, landed 2026-09-02: five computed colours, and `red_cabbage_indicator` — the exact name this row died on — now resolves) |
| K13 | Invisible ink | brown writing appearing | **wrong** → computed | a persistent, recipe-specific lemon mark stays faint while wet, dries when its water is removed, and browns on warming. This does not generalise to caramelisation or Maillard chemistry, and ignition remains the separate cellulose-combustion operation |
| K14 | Naked egg | membrane water transfer | computed | `naked_egg` keeps its inventory behind a bounded semipermeable membrane; water moves with the osmotic gradient. Shell removal remains the separate chalk/vinegar surrogate, with no yolk, elasticity or final-equilibrium claim |
| K15 | Rubbery bone | the bone bending | partial | the guided control computes acid dissolving calcium-carbonate chalk, then states the decisive boundary: real bone is a collagen–calcium-phosphate composite, and chalk has neither collagen nor a bending observable |
| K16 | Clean a copper coin | the shine returning | ~~partial~~ → computed | closed 2026-09-04. The acid dissolves the oxide to a blue solution, and with chloride the bench now precipitates **green atacamite** — the mineral that grows on wet copper. There is still no coin: no tarnish layer, no shine returning, and no difference between a second's dip and a night's soak. What the bench has is what the liquid does |
| K17 | Rusting race | orange rust | ~~**silent miss**~~ → computed | steel wool + brine + oxygen + 24 h left iron untouched, and said only "this part of the lab isn't awake yet" (KID-5, fixed 2026-09-02: the same script now converts all of it to reddish-brown iron(III) oxide) |
| K18 | Hot pack / cold pack | the thermometer | ~~partial~~ → computed | CaCl₂ gives +36 K, computed from dissolution enthalpy. **This row was my own mistake, corrected 2026-09-03:** I reached for Epsom salt, whose dissolution is very nearly athermal, and concluded the cold pack was unreachable because NH₄NO₃ is absent. Ammonium *chloride* is on the shelf, is what school kits actually contain, and gives −13 K |
| K19 | Salt crystals | cubes appearing | computed | evaporation precipitates halite with the ledger exact; crystal *habit* is not drawn |
| K20 | Rock candy | crystals on cooling | ~~**wrong**~~ → computed | sucrose saturation was modelled but **temperature-independent** — identical at 20, 60 and 90 °C (KID-7, landed 2026-09-03: hot water now holds 487 g per 100 mL against cold water's 200, a cooled syrup reports itself supersaturated, and a seed brings it down to exactly the limit) |
| K21 | Slime | the slime | ~~unreachable~~ → computed | no poly(vinyl alcohol) and no borate existed (KID-14, landed 2026-09-03: a dose response with the crosslinker still in the ledger afterwards, because a crosslinker is not a reagent) |
| K22 | Oobleck | liquid that goes hard | ~~honest miss~~ → computed | "this part of the lab isn't awake yet" (KID-13, landed 2026-09-03: the same vessel answers differently at 60 rpm and at 600, and a thin mixture answers neither way) |
| K23 | Plastic from milk | curds you can mould | ~~**wrong**~~ → computed | curdling **never fired with the aqueous solver on** (KID-2, fixed 2026-09-02); `filter v1 v2` refused because `v2` had to be created first (KID-15, fixed 2026-09-03: a pour now brings its own jar) |
| K24 | Sherbet | fizz on the tongue | computed | the dry mixture correctly does nothing; water starts it; pH 3.83 |
| K25 | Bath bomb | waiting for water | computed | the dry/wet contrast is the whole lesson and it lands |
| K26 | Felt-tip chromatography | the colours separating | ~~honest miss~~ → computed | "nothing dissolved here has a curated UNIFAC decomposition, so the column's method is silent" (KID-9, landed 2026-09-02: a black ink now separates into three spots at Rf 0.15, 0.35 and 0.85) |
| K27 | Starch hunt (iodine) | blue-black vs brown | computed¹ | both vessels right — and both preceded by a Danger-level banner saying the mixture "can detonate" (KID-3, fixed 2026-09-02: now clean) |
| K28 | Vitamin-C detective | blue-black going clear | computed¹ | decolourisation computed, dehydroascorbic acid in the ledger; one of its two banners survives KID-3 slice 1 and is owned by KID-3b |
| K29 | Yeast balloon | the balloon filling | computed | fermentation, ethanol, CO₂ — and then an honest **BANG** when the sealed vessel bursts. Correct, and a better lesson than the one asked for |
| K30 | Flame colours | one colour per metal | ~~partial~~ → computed | Na yellow, K lilac, Sr crimson, Ba apple-green, Cu blue-green all computed, and calcium — the one a child actually owns — read "nothing happens" (KID-10, fixed 2026-09-03: brick red). Lithium is still absent, and adding it is a species rather than a datum |

¹ computed, but preceded by a false hazard banner. See KID-3.

**Tally at audit time: computed 13 · partial 7 · honest miss 2 · silent
miss 2 · wrong 3 · unreachable 3.** After KID-1, 2, 5, 6, 7, 8, 9, 10, 14,
15, 20, 21, 12, 13 and 11: **computed 25 · partial 5**. Neither a silent
miss nor an unreachable row is left in the first thirty, and the volcano —
the experiment the whole list opens with — finally erupts.

**The thirty scripts, exactly as a newcomer first wrote them, now run
30/30** — against 17/30 when the audit was taken. Not one row is still
marked *wrong*, and the last script that could not run at all was K21's
slime. What remains is six partials and nothing else: K04's candle, the
last silent miss, burns as of KID-12, and K11's raisins, the last row that
could not be reached at all, ride their bubbles as of KID-13. The second
thirty gains K48 with KID-9's fix, K50 with KID-20's,
and K23's receiver with KID-15's.

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
| ~~Corrosion of iron in aerated brine~~ | K17 — **landed as KID-5**, 2026-09-02 |
| ~~Combustion of organic solids; a flame that can be starved~~ | K04, K47 — **landed as KID-12**, 2026-09-03. Paraffin, cellulose and sucrose burn against a curated table, and a flame quits at a limiting oxygen fraction rather than at zero |
| ~~Latent-heat plateau at a boiling point~~ | K13 — **landed as KID-6**, 2026-09-02 |
| ~~Temperature-dependent solubility of molecular solutes, and nucleation~~ | K20 — **landed as KID-7**, 2026-09-03; electrolyte supersaturation (K51) stays open as KID-7b |
| Acid curdling wired to the solved ledger | K23; see KID-2 — this one is a bug, not a gap |
| Foam on any gas-evolving vessel with a declared surfactant | K01; the volcano's whole point |
| ~~Suspension rheology~~; ~~buoyancy on attached bubbles~~; miscible stratification | K22 and K11 — **landed as KID-13**, 2026-09-03; K10/K57's slow-pour stratification is the half still open |
| ~~Paper/TLC mode with dye partition data (`EXP-8`)~~ | K26, K48 — **landed as KID-9**, 2026-09-02 |
| ~~Anthocyanin as a computed pH-dependent chromophore~~ | K12 — **landed as KID-8**, 2026-09-02 |
| Pyrolysis and Maillard browning | K13 — still the only missing half of the invisible-ink row: KID-12 made the paper burn, and the brown stage before it is the part with no model |
| ~~Calcium~~ (lithium still absent) in the flame-colour table | K30 — **calcium landed as KID-10**, 2026-09-03 |
| ~~Acetic acid in the odour table~~ | it was *in* the table — `smell` on vinegar still said "no odour a careful waft detects", because the solver had turned the acid into acetate and the table is keyed on the acid. **Landed as KID-10**, 2026-09-03; the third instance of one defect |
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
  **Slice 1 landed 2026-09-02** — the self-screening half. `SafetyScreen`
  gained `assess_pour(before, after)` (defaulted to today's `assess`, so no
  other screen changes), and `ReactiveGroupScreen` drops a finding whose two
  species arrived together in one reviewed bottle and were in the vessel
  beforehand in neither case. The recipes are the authority: a pairing that
  a `MaterialRecipe` ships was reviewed when the recipe was. K27 goes from
  two Danger banners to none; bleach + ammonia, permanganate + sulfite, and
  iodine + iodide *poured separately* all still fire. Corpus unmoved.
  **KID-3b is the dose half, still open.** K28 keeps one banner — iodine
  meeting ascorbic acid is a real oxidiser/reducer pair, but 0.0005 mol of
  it in 100 mL is not "at scale, such mixtures can detonate". Two things are
  wrong there and only one is the dose: the `real_world` sentence describes
  *permanganate and sulfite* while the beaker holds iodine and vitamin C, so
  the example reads as a claim about this vessel. Fixing the wording needs
  no new number; a severity floor does, and that number has to be sourced
  rather than chosen.

- **KID-4 — `ignite` is never silent.** Holding a flame to an unresolved
  material currently emits nothing at all. The project's own rule is that
  wherever the engine declines to model something it must say so.
  *Acceptance:* every `ignite` on every shelf item produces a line at lv1;
  a test enumerates the shelf and asserts a non-empty response for each.

### Wave 2 — the mechanisms the corpus is actually short of

- **KID-5 — Rusting.** Pull `EXP-34` forward: iron surface area, oxygen
  transport, chloride acceleration, Fe(OH)₂/Fe(OH)₃/Fe₂O₃ product routing,
  a rate slow enough to need `wait` and fast enough to see.
  **Landed 2026-09-02** as a curated kinetic reaction, `iron-corrosion`,
  because rusting is three rate questions and no stoichiometric ones: it is
  slow enough that `wait` is the instrument, it *stops* when the trapped
  oxygen runs out, and salt makes it faster without being consumed.
  `lessons/rusting.lab` runs the four-arm comparison `EXP-34` asked for —
  dry, oxygen swept out, plain water, salt water — and every arm holds in a
  test on the shipped binary. Measured over one sealed day on a gram of iron
  under 500 mL of room air: dry 0, deoxygenated 0, plain water 0.0016 mol of
  oxide, salt water 0.0028 mol. The salt arm runs its oxygen down to
  0.0001 mol and the headspace falls to 0.808 bar — the water rising in the
  stoppered tube, computed rather than drawn.
  Three things had to be true before the lesson could land, and only the
  first was the rate law:
  - **The product had to be a solid the solver would leave alone.** Written
    as the school equation with Fe(OH)₃ on the right, the aqueous engine
    dissolved every mole of rust straight back into Fe(III) hydroxo
    complexes — and then said, correctly, that a real beaker would not stay
    like that. The rust vanished on the way to being seen. Written as the
    net oxidation to Fe₂O₃, with the water requirement carried by the
    reaction's locality instead of its stoichiometry, it stays.
  - **The description had to name more than one thing at the bottom.** It
    reported the single largest settled solid, so a rusting nail was "grey
    iron" and nothing else — the ledger held the rust, the words did not.
    It now names every settled solid down to a tenth of the largest.
  - **The bench had to stop calling iron unreactive while rusting it.**
    `Event::Inert`'s lv1 wording — "nothing happens … it is too unreactive
    for this" — dropped the scope its lv2 and lv3 forms carry, and became
    flatly false the moment iron started to rust. It now says what it
    actually means: nothing dissolved here swaps places with the metal.
  Two smaller things came with it. `kero study` gained an
  `amount:<species>@vN` probe, because `EXP-34`'s acceptance is that the
  study sweeps the conditions and the only answer worth sweeping for is how
  much rust — and no existing probe can see a solid. And `--vary` now says
  that it sweeps **moles** whatever unit the script line used: varying a
  line reading `add v1 Fe 1g` silently dosed one mole, 55.8 g, and produced
  a curve that looked like a rate law that had stopped responding.
  Two data errors surfaced on the way and were fixed with it, because both
  are things a learner looks straight at: iron(III) oxide was called
  **pink** (a saturation rule written for *transmitted* colour — the
  dilute-versus-concentrated permanganate case — applied to a scattering
  solid), and zinc was called **blue-green** (rgb(186, 196, 200) clears the
  absolute chroma cut-off by two while being grey by any measure of
  saturation). Rust is not pink and zinc is not blue-green, and a child
  looking at a rusted nail is the reader who notices first.
  The corpus moved by two rows: `th-067` "can iron rust without being
  heated?" and `th-068` "does salt water make iron rust faster than fresh
  water?" were both pinned `missing` and are now `computed`, so
  `expectation_mismatches` fell 151 → 149 with no baseline drift. `aq-125`
  and `th-069` stay `missing` and correctly so: neither adds oxygen, and
  galvanic protection by zinc is a coupling this reaction does not model.
  **Stated boundaries**, all in the entry's own provenance: iron *amount*
  stands in for iron *area*, so a nail and filings of the same mass rust at
  the same speed and K41's powder-versus-lump contrast is still out of
  reach; the product is the registry's anhydrous oxide rather than the
  hydrated mixture real rust is; and an open vessel still does not draw
  oxygen from the room, which is why every arm of the lesson is sealed over
  a measured amount of it.
  **Extended 2026-09-05 (BRD-023) with the coupling this entry named as
  missing.** `crates/kerotakis-core/src/corrosion.rs` resolves exactly the
  part `iron-corrosion`'s own uncertainty note stood aside from — which of
  two metals in one electrolyte is the anode — and enforces it where it
  counts, in `KineticReaction::can_run`. The lower-E° metal (read off
  `displacement::SERIES`, so the bench keeps one activity series) corrodes
  for both; a companion entry `zinc-corrosion` makes the sacrifice real, so
  the zinc is consumed and turns to zinc hydroxide while the iron beside it
  is left alone; and a barrier table carries the passive film of stainless
  steel and the paint film of painted iron, keyed on the material recipe the
  lot came from. Every verdict, positive or negative, is an
  `Event::Corroded`, so "this is not rusting, and here is what is protecting
  it" is a computed answer rather than a silence. Its own stated boundaries:
  protection is all-or-nothing because the bench has no anode-to-cathode
  area ratio, and a sacrificial zinc does not speed up to carry the iron's
  current, so a coat lasts longer here than it would in a bucket.
- **KID-6 — The boiling plateau.** Hold temperature at the (solute-shifted)
  boiling point while water leaves as steam; make the lv1 register say what
  lv3 already says when the aqueous model's 300 °C ceiling is passed.
  **Landed 2026-09-02.** Freezing and melting had paid latent heat since
  they were written; boiling announced the transition, left the water
  liquid, and let the temperature run wherever the energy put it. The same
  arithmetic as the melting branch, in the other direction, now buys vapour
  with the energy above the boiling point and holds the thermometer there
  until it has bought all of it.
  The second half was a hand-off. `kerotakis_cea::ThermalEquilibrator`
  declined a vessel holding liquid water only *below* 100 °C, so pure water
  heated past boiling went to the Gibbs minimiser instead — and that route
  emptied a 100 mL beaker on 120 kJ when boiling it dry costs 256. It now
  declines whenever liquid water is present, so the plateau is paid first
  and the minimiser only ever sees what is left. Measured, 100 mL of water:
  30 kJ → 96.99 °C and nothing gone; 60 kJ → 100.00 °C, 87.0 g; 240 kJ →
  100.00 °C, 7.2 g; 300 kJ → dry at 205 °C. Salt water holds at 103.52 °C
  and says why — the same colligative relation that salts a road, read at
  the other end of the curve.
  **The blast radius was measured, not assumed:** the frozen-behaviour
  golden moved by zero lines for the solver change, and only by the new
  `lessons/boiling-curve.lab` block. The curiosity corpus moved by two rows,
  and the first of them is the question this task exists to answer —
  `aq-102`, "does water stay at its boiling point while it changes into
  steam?", pinned `missing` and tagged `latent-heat-gap`, now `computed`.
  The second, `aq-103` (hot water poured into cold), keeps its answer of
  50.00 °C exactly and only changes which route claims it, because CEA no
  longer contests a vessel with liquid water in it.
  **Stated boundary**, in the code where it bites: the leftover sensible
  heat once the last water has gone is spread over the heat capacity the
  vessel had *before* it boiled dry, so the final temperature of a vessel
  taken past dryness is under-reported. The melting branch makes the same
  approximation in the same place; neither touches the plateau itself,
  which is the observation the curve exists for.
- **KID-7 — Solubility with temperature, and crystallisation.** `s(T)` for
  molecular solutes starting with sucrose; supersaturation as a state the
  bench can report; seeded growth on `wait`.
  **Landed 2026-09-03.** The saturation limit was one reviewed number per
  solute, read at every temperature, so hot water held no more sugar than
  cold water and the mechanism rock candy exists to show could not happen.
  `SpeciesData` gained a second reviewed point at 100 °C and the limit is
  now read at the vessel's own temperature — two points make a line, and a
  line is the whole of the temperature dependence this bench claims. Sucrose
  is the first solute with one: 200 g per 100 mL at 20 °C, 487 at 100 °C. A
  solute with only one reviewed point stays temperature-independent, and the
  model says so rather than extrapolating from a single number.
  The limit also answers in **both** directions now, and the third answer is
  the interesting one:
  - room to spare → dissolve, as before;
  - over the limit with a crystal of the same solute already present →
    crystallise onto it;
  - over the limit with nothing to grow on → **`Event::Supersaturated`**.
    Not an error and not a rounding artefact: it is the state a cooled sugar
    syrup is genuinely in, and precipitating it automatically would erase
    the experiment. Measured: 300 g of sugar dissolves completely in 100 mL
    at 95 °C, cooling reports 1.40× saturation and moves nothing, and one
    gram of sugar as a seed drops 0.253 mol out and leaves the solution at
    exactly its limit. That last number is the yield of the experiment, and
    nobody had to be told it — it is the difference between two
    solubilities.
  `lessons/rock-candy.lab` is those three states in two beakers.
  **Stated boundaries:** the line between 20 and 100 °C runs a few percent
  below sucrose's real curve in the middle of the range; crystals appear
  instantly with no size, habit or purity; and nothing nucleates
  spontaneously, so the unpredictability that makes one real jar set
  overnight and the next not is outside the model. **KID-7b** is the
  electrolyte half — K51's sodium-acetate hand warmer needs a
  sodium-acetate-trihydrate phase the shipped databases do not carry, so it
  belongs to the aqueous engine rather than to this table.
- **KID-8 — Anthocyanin.** A red-cabbage material and an anthocyanin
  chromophore with pH-dependent spectra through the existing Beer–Lambert
  path, so the rainbow is *computed colour*, not a tinted lookup.
  **Landed 2026-09-02.** The obstacle was structural rather than missing
  data. An `Indicator` in this engine is one weak acid with *two* coloured
  forms, which is the whole story for phenolphthalein and cannot be the
  story for red cabbage: five colours do not come out of two forms.
  `PigmentLadder` generalises the same Henderson–Hasselbalch idea the way a
  polyprotic acid already is elsewhere here — `n` successive pKa values give
  `n + 1` forms, the fraction in each is the standard stepwise distribution,
  and the spectrum is every form's ε(λ) mixed in those fractions. A test
  holds that a two-form ladder reproduces Henderson–Hasselbalch exactly, so
  the generalisation is provably one.
  Anthocyanin joins the registry as a four-form ladder at pKa 4.0 / 7.0 /
  11.0, and `red_cabbage_juice` joins the shelf with the aliases a learner
  reaches for. `lessons/cabbage-rainbow.lab` reads **red at pH 2, deep
  purple at 6, blue at 10, blue-green at 12, yellow at 13**.
  **The green is the argument.** No form of the pigment is green. It appears
  where the blue form and the yellow form are both present, absorbing at each
  end of the visible range and leaving a window between them — a colour in no
  table in this engine, computed from two that are. A lookup table of "cabbage
  colour versus pH" would have had to be *told* about green; this one was not.
  Corpus: `aq-046`, "can red cabbage juice estimate the pH of household
  liquids?", was pinned `missing` / `unknown-species` / `indicator-gap` — and
  the name it used was already `red_cabbage_juice`, which is the canonical key
  this task chose independently. It now reads `computed`.
  **Stated boundaries**, in the ladder's own provenance and the recipe's lot
  assumptions: red cabbage is a mixture of acylated cyanidin glycosides
  rather than one compound, so no InChIKey is asserted — the position starch
  and cellulose already take here. Only the flavylium ε is a literature
  figure; the other three are explicitly editorial, chosen so each colour
  becomes visible at the concentration a jar of cabbage water actually is.
  And the juice *reports* pH without moving it: a real extract is mildly
  acidic and buffered, and this surrogate is not.
- **KID-9 — Paper chromatography.** `EXP-8`'s Rf mode plus partition data
  for the three shipped dyes and a black-ink surrogate.
  **Landed 2026-09-02.** Rf is not a second model. A column reports *when* a
  solute leaves and a paper strip reports *how far* it went, and both are one
  partition coefficient read two ways: `Rf = Kβ/(1 + Kβ)` is the fraction of
  its time a solute spends in the moving phase. So `ElutedPeak` gained an
  `rf` beside its retention time, both computed from the same `K`, and a test
  holds that the two rank the dyes identically — the bench cannot say one
  thing on a column and another on paper.
  The blocking gap was the partition data. `K` comes from a UNIFAC group
  decomposition, which exists for exactly three species here; a food dye is a
  large glycoside or a sulfonated aromatic, and splitting one into UNIFAC
  groups would be a fiction dressed as a calculation. So four dyes carry a
  **curated** `K` instead, and the lv3 provenance line now names which
  coefficients were computed and which were reviewed — it used to claim every
  one came from UNIFAC, which was true until it wasn't.
  `black_ink` joins the shelf as three dyes in the ratio that reads as a dark
  neutral, and `lessons/ink-chromatography.lab` takes it apart: indigo
  carmine at Rf 0.15, betanin at 0.35, curcumin at 0.85, with each pure dye
  running to the same height on its own — which is what lets a strip identify
  an unknown ink.
  Corpus: `bio-104`, "can paper chromatography separate two food dyes?", was
  pinned `missing` and filed against `EXP-36` (organic synthesis); it now
  reads `computed` and is filed against `EXP-8`, which owns chromatography.
  The only change to an existing lesson's output is the added `Rf` field —
  every retention time is unchanged to the digit.
  **Stated boundary:** the four curated coefficients are *ordered*, not
  measured. They reproduce which dye runs furthest and roughly how far apart
  the spots land; a real strip's Rf depends on the solvent, the paper and the
  temperature, and none of that is claimed.
- **KID-10 — Completions.** Calcium and lithium flame colours; acetic acid
  and ethanol odours; the missing cold-pack salt (NH₄NO₃) through the
  registry pipeline.
- **KID-11 — Foam is general.** A declared surfactant plus any gas-evolving
  reaction produces the existing foam observable, not only the peroxide
  path.
  **Landed 2026-09-04.** It was literally one reaction id:

  ```rust
  if reaction.id == "peroxide-decomposition" { oxygen_moles += moles.0; }
  ```

  and `foam::advance` took a parameter called `oxygen_moles` while nothing
  in its arithmetic ever looked at which gas it was. Carbon dioxide lifts a
  soap film exactly as oxygen does. The name was the only thing claiming
  otherwise, and the name came true.

  **Widening the accumulator would not have been enough**, which is the
  part worth keeping. The two engines that make gas report it in different
  words, and a volcano is entirely the second kind:

  ```
  peroxide  0.019183 mol O2 produced             GasProduced, retained
  volcano   0.041880 mol carbon dioxide evolved  GasEvolved, gone
            0.007449 mol carbon dioxide evolved
  ```

  A volcano's carbon dioxide leaves during the *solver pass of the `add`
  step*. There is no `wait` in a volcano, so by the next time advance the
  gas is out of the ledger entirely — and `advance_vessel_time`, where the
  trap lived, only runs on `stir` and `wait`. So the trap moved to
  `step_with`, where both engines' reports are visible.

  `gas_made_this_step` combines the two views with `max`, not `+`: they are
  two views of one step, and a parcel both engines described would
  otherwise be counted twice. No shipped path reports the same parcel both
  ways today, which is exactly the kind of coincidence this file has been
  cataloguing all week, so `max` fails towards under-claiming rather than
  doubling.

  **A regression the move introduced, caught by reading the diff rather
  than by a test:** running on every operator instead of only timed ones
  meant a vessel that had foamed once re-reported its unchanged foam on
  every later `look`. The old call site got that guard for free by never
  running otherwise. No gas and no elapsed time is not an event.

  **Measured, not predicted.** The no-soap control evolves 0.049 mol of
  carbon dioxide and does nothing. The same reaction with washing-up liquid
  in the glass first reaches **foam 1.375 L, 49.1 cm high, 1.178 L over the
  rim**, and two minutes against the soap's 180-second half-life takes it
  back to 0.866 L. The lesson's prose originally said the foam would be
  "most of the way back to a liquid" after that wait; the measurement says
  a bit over a third of it goes, and the prose now says what the number
  says.

  **Stated boundaries:** bubble size, film drainage geometry, and the
  difference between a detergent film and a protein one are not modelled.
  The half-life is the recipe's own reviewed number and the trapped
  fraction is a bounded teaching value — this claims *that* it foams and
  roughly how fast it subsides, not what the foam is made of.
- **KID-12 — Combustion of organic solids.** Paraffin, paper and sugar with
  real combustion data; a flame that a gas blanket can starve; browning as
  a separate, honestly-bounded observable.
  **Landed 2026-09-03**, and it closed the last silent miss in the first
  thirty (K04) along with K47 and half of K13.

  The engine could already burn hydrogen, methane, ethanol and magnesium,
  because NASA CEA's `thermo.inp` has records for them. It could burn none
  of the three things a child actually sets fire to: `grep -nE "^C1[0-9]H"`
  over that dataset stops at naphthalene, and `charge()` declines the WHOLE
  vessel the moment one species is outside it. So a candle, a sheet of
  paper and a spoonful of sugar all reached `NotYetModeled` — honest, and
  very thin for the three commonest fires in a house.

  `kerotakis_core::combustion` is the curated complement: three fuels, each
  with a balanced complete-combustion equation, a measured heat of
  combustion and an autoignition temperature (paraffin 473 K, cellulose
  506 K — which is 451 °F, and the novel's title is a real measurement —
  sucrose 683 K). It sits **after** the CEA solver in the stack on purpose:
  where NASA's data can name every species in the vessel it should answer,
  and this table speaks only for what it cannot.

  The part that earns the module its place is not that things burn, it is
  **the limiting oxygen fraction**. A flame needs air that is rich in
  oxygen — about one part in six — not merely some oxygen. So:

  - a candle under a jar goes out with **77% of the oxygen still there**,
    which is the opposite of what "it used up the oxygen" says;
  - carbon dioxide poured in first stops the flame from starting at all,
    taking nothing away from either the wax or the oxygen — a fire
    extinguisher, as one line of arithmetic rather than a special case;
  - a nitrogen-swept vessel will not light.

  Two recipes had to be resolved first, and both had **expired reasons** of
  exactly the kind KID-20 was written to catch. `candle_wax` said "no
  component of it is an installed species", which was true until this task
  installed `paraffin` (C25H52, one representative chain length standing
  for a C20–C40 blend). `paper_sheet` said "cellulose is not in the runtime
  registry" — and cellulose had been in it for some time. Both now resolve
  (92% and 85%) and conserve the rest; both lot assumptions were rewritten
  rather than left reading as current.

  **Stated boundaries**, all of them in the fuels' own provenance: no wick,
  no melt pool, no luminous flame, no soot, no smoke, no char, no carbon
  monoxide — burning here is always complete. No burn *rate*: one `ignite`
  burns what the air allows, all at once. The 16% limit is one teaching
  number standing in for something that really depends on fuel, wick and
  jar geometry, and it does not distinguish carbon dioxide from nitrogen
  even though carbon dioxide is the better smotherer. And the sugar entry
  offers only the two ends: unchanged below 683 K, burned above it. The
  caramel in between is still missing, which is what keeps K13 a partial.

  **Two things the work found on the way.** An open burn must not heat its
  own beaker: booking the reaction energy into whatever was left in the
  vessel produced a 6089 °C beaker in the curiosity corpus (`th-058`),
  because the products — which carry the heat — had already left the
  ledger. An open flame now reports its energy and warms nothing; only a
  closed boundary, which keeps its own hot gas, can be warmed by it. And
  the new `FlameStarved` event, added to the corpus classifier's
  typed-observation list wholesale, demoted `th-051` from computed to
  qualitative; the classifier now keys on the event's `burned` field, so a
  flame that never caught is an observation and one that burned first is a
  result. Baseline drift caught both.

  **A boundary this exposed rather than created:** an insoluble solid in
  water reads as a suspension however large the lump, so 20 g of wax in a
  beaker now makes the water look like milk. A floating candle is not
  modelled, and particle size is not a thing the appearance model knows.

### Wave 3 — physical behaviours and the cabinet

- **KID-13 — Physical mixtures.** Suspension rheology (oobleck), buoyancy
  on attached bubbles (raisins), miscible stratification with a slow pour
  (density tower). Each may land as an honest bounded observable rather
  than a CFD claim, following the `magic-milk` precedent.
  **Oobleck landed 2026-09-03.** It is the only experiment on the children's
  list that is not chemistry: nothing reacts, and what changes is how the
  mixture *answers being pushed*. So the answer depends on the push. The same
  vessel, in the same state, reports "it flows like a thick liquid" at 60 rpm
  and "it goes stiff under the stirrer" at 600 — and a thin suspension
  reports neither at any speed, because the effect needs the particles packed
  close enough to jam. `lessons/oobleck.lab` holds all three cases and the
  ledger, which is exactly what went in.
  **Stated boundary:** no viscosity, no yield stress, no critical shear rate.
  The claim is only that this mixture is one of the ones that does this and
  that this stir was fast enough to notice; particle size, starch source and
  standing time all matter in a real bowl and none are represented.
  **The raisins landed 2026-09-03 too**, and with them the last
  *unreachable* row in the children's first thirty. K11 needed two things
  that did not exist: a raisin, and a model for a bubble that attaches to
  an object and lifts it.

  Nothing reacts in this experiment — the raisin is the same raisin
  afterwards — so what the bench computes is the one number the
  demonstration is about. A raisin sinks because it is denser than water,
  1.35 g/mL against 1.00, and the bubbles clinging to it lift it only when
  they add enough volume to bring the pair below the liquid's density:
  `V_gas/V > (ρ_object − ρ_liquid)/ρ_liquid`. That is **35%** in water — a
  third of the raisin's own size in gas before anything happens, which is
  why you can watch the bubbles gather for a while first — and **11%** in
  sugar syrup, where the liquid does most of the lifting. `lessons/dancing-raisins.lab`
  runs both, plus the still-water control that says nothing, because there
  is nothing to ride.

  The trigger is the gas, not a clock: this bench lets a glass of fizzy
  water go flat in one step, so the moment to say it is the moment the
  bubbles appear — which is when the fizzy water meets the raisin, and is
  also the order you would do it in a kitchen.

  **A bug the work found.** `Vessel::liquid_volume` excludes solute volume
  by design: a solution's volume is carried by its solvent. Reading a
  density straight off that gave 200 g of sugar in 100 mL of water a
  density of **2.33 g/mL** — denser than anything that has ever been
  poured, because all of the sugar's mass and none of its volume was
  counted. The solute's own density puts the volume back, and the syrup
  reads 1.27, which is what a hydrometer would say. The registry has a
  density for all 141 species, so nothing is guessed.

  The raisin also brings back a contract KID-12 had left with no user:
  it is the only recipe in the registry now carrying
  `ConservedUnresolvedSolid`, and its sugars are deliberately NOT resolved
  even though sucrose, glucose and fructose are all installed — a raisin
  is an object whose sugar stays inside it over a demonstration, and a
  recipe that dissolved it would delete the thing being watched.

  **Stated boundaries:** no period, no bubble count, no bubble size, no
  nucleation-site density, no rise velocity, no number of trips. Membership
  in the rider table is curated rather than derived from density, because a
  stone is also denser than water and gas does not lift it — the effect
  needs a surface bubbles stick to, and no recipe here describes surface
  texture.

  **Still open in KID-13:** miscible stratification (K10, K57) — a slow
  pour that would not mix.
- **KID-14 — The children's materials pack.** PVA and borate, egg, raisin,
  lemon juice, gelatin, effervescent tablet, glycerol, tarnished copper.
  **Slime landed 2026-09-03**, and with it the last of the thirty scripts
  that could not run at all. `PVA` and `Na2B4O7` join the registry as the
  polymer repeat unit and the anhydrous salt — a polymer is not a molecule,
  so no InChIKey is asserted, exactly as starch and cellulose already do —
  and `pva_glue` and `borax` join the shelf. Household borax is the
  decahydrate, so a little over half of what the box weighs is the salt
  doing the work, and the ten waters ride in the conserved remainder rather
  than being released as free water.
  The observable is a dose response, not a reaction, and that distinction is
  the lesson. Borate bridges between polymer chains keep breaking and
  re-forming, so nothing is consumed: 0.05 g of borax makes no slime, 0.25 g
  makes it, 2 g makes no more of it — and in all three vessels every gram of
  borax is still in the ledger afterwards. A test holds that last part,
  because it is what a reaction would get wrong.
  **Stated boundaries:** the response is calibrated to the familiar 50 mL-glue
  classroom ratio rather than measured, and no modulus, relaxation time or
  stringiness is claimed. Whether a real glue gels at all depends on its
  degree of hydrolysis, and poly(vinyl acetate) glues do not gel this way.
  The rest of the pack — egg, raisin, gelatin, effervescent tablet,
  glycerol, tarnished copper — is still open.
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
  **Landed 2026-09-02.** `help` is regrouped by what a learner wants to do
  and now names all 31 verbs in `script::VERBS` — `magnet`, `smell`, `test`,
  `chromatograph`, `react`, `remove`, `centrifuge`, `stock`, `particles` and
  `regulate` were landed, working and unmentioned, and the corpus lost
  experiments to each of them. A test fails the day a verb is added without
  a help line. `kero lessons` lists the thirty-seven shipped lessons by
  their own first-line titles and says how to run one. The signpost half of
  the error message landed with KID-1.
  **Still open under KID-17:** the GUI's own help dialog and the affordance
  manifest were not audited here, and `EXPERIMENTS.md`'s quoted error text
  is now stale.

### Sequencing

```
KID-1 ──┬── KID-17 (docs quote the new commands)      [KID-1 done]
        └── KID-16 (lessons need typeable names)
KID-2, KID-3, KID-4   independent bug fixes           [2 done, 3 slice 1 done]
KID-5 … KID-12        independent; KID-8 before KID-9's ink   [5, 6, 7, 8, 9, 10, 12 done]
KID-18 … KID-21       from Part 2                            [20, 21 done]
KID-13 … KID-15       after their mechanisms land           [13 part, 14 part done]
KID-18 … KID-21       from Part 2; KID-18 and KID-20 are the cheap ones
```

The three shipped claims that do not reproduce — KID-2's curds, KID-18's
Faraday's law, and T09's ester — share one shape: a model that is present
and a path to it that is closed. That is what a corpus run by a stranger
finds and a corpus written by the author cannot.

### The tail is long and flat, and one row nearly became a lie (2026-09-04)

With the naming fixed, I measured what the *remaining* unanswered rows are
blocked on — not by counting substance mentions, which is misleading (43
rows mention `water`), but by finding the first substance in each script
that does not resolve.

**106 rows, blocked on 88 distinct substances.** The largest single blocker
accounts for **three** rows. That is roughly 1.2 rows per recipe, and every
recipe needs a real sourced composition. The earlier framing of "31 food
recipes" implied a much better ratio than exists.

So the recommendation changed: **stop adding substances by blocker
frequency.** Pick the rows whose *questions* become genuinely answerable,
and leave the rest honestly missing. A row converted from `missing` to
passing without becoming answerable is worse than one left alone.

**Two rows were about to demonstrate exactly that.** `mat-080` asks *"why
are diamond and graphite so different if both are carbon?"* and its script
is `add v1 graphite 1g; measure v1 balance`. `mat-041` asks *"why can glass
be transparent although sand is opaque?"* and weighs 2 g of silica glass.
Adding the named substance would have moved both out of `missing` while
answering neither.

That is the prediction from the answer-invariance sweep arriving on
schedule: *"they become lies the day their mechanism lands: the row starts
passing, on a script that never built the second condition."*

So `mat-080` got **both** allotropes rather than the one it names — graphite
at 2.26 g/mL and diamond at 3.51 — and a script that builds and compares
them. Same element, same 1.00 g on the balance, different density and
different colour, which is as far as this bench can take the question.
`mat-041` was left alone: transparency is not something the appearance
model can express for a solid, so a species would let the script run and
not let it answer. Recorded in `diamond`'s own provenance, where a reader
meeting "white diamond" will find out why.

**Also landed here:** ammonium nitrate with its +25.7 kJ/mol dissolution
enthalpy, which is K18's cold pack — 20 g takes 100 mL of water from
25.0 °C to **9.6 °C** — and a `cold pack salt` bottle so the everyday name
resolves, since species synonyms do not survive the registry export.

### A written refusal that only luck protected (2026-09-04)

`table_salt`'s lot assumption says *"the bare words salt and Salz remain
unclaimed because they name a chemical class"*. The systematic alias pass
above had `salt` in its list.

It did not get added, and not because anything stopped it: that recipe is
built by a different helper and the edit pattern did not match. I was
relying on **remembering** the refusals, and I remembered `candle_wax` and
forgot `table_salt`.

`no_recipe_claims_a_word_it_has_declined_in_writing` now checks it. Proved
by temporarily claiming `salt` and watching it fail with the recipe's own
sentence quoted back. A refusal that lives only in prose is a refusal that
depends on whoever edits next having read it.

### Thirty rows were blocked on a word, and German already knew it (2026-09-04)

Asked where the remaining 147 unanswered questions would get their data
from, I counted rather than guessed. Of the 120 blocked on an unknown
substance, **thirty were blocked on a name** — the shelf had the thing,
filed under the word a chemist uses, and the prompt used the word a child
uses.

Whether a bare everyday word may be claimed is a real question in this
project, and it is answered against claiming elsewhere: `candle_wax`'s lot
assumption refuses bare *wax* in writing, because beeswax and paraffin are
different materials. So I measured instead of ruling:

**Thirty-five of fifty-six recipes were reachable by a bare everyday word
in German and only by a compound one in English.** `Essig`, `Hefe`,
`Milch`, `Sand`, `Natron`, `Kreide`, `Alufolie`, `Glas` — all resolve.
`vinegar`, `yeast`, `milk`, `chalk` did not. The project had already
decided a bare word is claimable. It had only done it in one language, and
the English-speaking child was worse served by the same registry.

Thirteen aliases close that gap: vinegar, milk, yeast, soap, sugar, oil,
pepper, glue, ink, bicarb, filings. Not `wax`, which is declined in
writing; not `apple` or `cabbage`, because a fruit is not its juice and a
vegetable is not its indicator — those want food recipes, which is
different work. `cola` and `sand` were refused by the registry validator as
already claimed, which is the guard doing its job.

**Fourteen rows opened. Missing 147 → 133; computed 229 → 236.** Every one
had declared `parse_boundary = unknown_species`, and every declaration
became false the moment the word resolved — the mechanism that caught
`aq-067` a section above, working at scale.

One of them then failed for a second reason the first had been hiding:
`bio-075` says `wait 7d` and the parser has no day unit. A declared
boundary masks everything behind it, which is worth knowing about the other
fifty-nine boundary rows.

### Three rows got worse by getting better (2026-09-04)

`aq-123`, `mat-057` and `th-082` went **computed → qualitative**, and not
one of them touches a new alias. K40's basic copper sulfates did it.

Precipitating antlerite releases protons, so the solution is now slightly
more acid — enough for the displacement model to add a true aside: *"iron
should dissolve in this acid by the series (driving force +0.25 V), but
hydrogen has to form on iron"*. That aside is an `Inert` event; `Inert` is
in the corpus classifier's typed-observation list; and that list is checked
**before** the branch that would have said `computed`.

The copper is still plated — `0.009967 mol copper plated out onto iron`,
unchanged. More phases modelled, a more accurate pH, one more true
sentence, and a worse score.

Every earlier instance of this pattern in this file was a row that
*happened* to answer and qualify. This one is a change that made the bench
explain more and was marked down for exactly that: the only way to have
protected the metric would have been not to model the phases. It is the
clearest evidence yet that the ordering taxes honesty rather than merely
mis-sorting it.

**The ordering is not touched here.** A commit that adds species has no
business redefining how rows are scored — the lesson of the peer session's
withdrawn #362, which applies to me exactly as it applied to them. Recorded
in the corpus README with the three rows named, and handed to whoever owns
the classifier.

### The corpus had a to-do the audit did not (2026-09-04)

K13's invisible ink could not be written down: `add v1 Zitronensaft` failed
at the shelf. Adding a lemon-juice recipe is the small half of that row —
91% water, the 4.7% citric acid its sourness is made of, a little sugar —
and it produced two findings worth more than the recipe.

**A corpus prompt was waiting for the bottle.** `aq-067`, *"Does lemon
juice neutralise a sodium bicarbonate solution?"*, carried
`parse_boundary = "unknown_species"` and the tag `material-recipe-gap`.
That is not a failing row: it is a **note that the shelf was short**,
written by whoever wanted the question asked. The moment the bottle
existed the declaration became false, and the corpus lint said so before I
had noticed:

```
prompt aq-067: declared parse_boundary Some(UnknownSpecies), observed None
```

Which is the mechanism this file has spent the week wishing it had. Four
rows here went stale and sat wrong for days because nothing re-asks a
verdict. The corpus refuses to let a declaration outlive its reason, and it
caught this one in the same minute it became untrue. `aq-067` now answers —
1.86, then 0.0471 mol of carbon dioxide, ending at 9.75 — so yes, and
rather more than neutralise.

**And a wrong-mechanism observable I nearly shipped.** Vitamin C is the one
thing a lemon is famous for, ascorbic acid is an installed species, so the
first draft resolved it. The juice then read *"colourless and very slightly
hazy"* — which real lemon juice is.

It was right for the wrong reason. No shipped database defines an
ascorbate, so the bench cannot dissolve ascorbic acid, and the haze was an
undissolved grain of vitamin C sitting in the glass. Real haze is pulp,
which this recipe does not have. That is the brine-at-1.00-g/mL shape
again: a believable number from a mechanism that is not there, and harder
to catch than an obviously wrong one because nothing looks wrong. The
vitamin C is in the conserved remainder with the reason recorded.

The same discipline applied to the pH: 4.7% citric acid gives **1.86**, and
a real lemon measures 2.2–2.4 because it carries citrate salts that buffer
it. Weakening the acid would have moved the composition to fix the number,
which is pulling the wrong end. The composition is right, the number is
right for the composition, and the gap between it and a real lemon is
written into the lot assumptions.

### A fourth stale row, and two that were never work (2026-09-04)

**K40 was closed by nobody.** Its verdict listed three complaints — a boil
at 109 °C with liquid water still in the ledger, no crystals from a cooling
saturated solution, and chalcanthite drawn white — and three separate tasks
answered them one at a time. KID-6 fixed the latent heat, KID-7 gave
cooling solutions their crystals, KID-20 gave chalcanthite its blue. Nobody
re-read the row, so it has been sitting at *partial* describing a bench
that answered it days ago.

Verified rather than assumed: 0.0884 mol of chalcanthite precipitates on
cooling with `Cu²⁺(aq) + SO₄²⁻(aq) + 5 H₂O(l) → chalcanthite(s)` printed,
and the description is blue. `lessons/blue-crystals.lab` and its replay
test now pin it, so it cannot go stale in the other direction either.

One detail worth keeping, because it is the model being right rather than
convenient: the liquid reads **black** while the copper is dissolved and
**blue** after the crystals have come out. Nothing changed colour. A strong
copper sulfate solution saturates the light path in a 4 cm beaker and you
genuinely cannot see through it; taking most of the copper out is what
makes it blue again.

**K09 and K49 are not work and should never have been listed as if they
were.** The lava lamp's rising blobs and the soap-driven boat are both bulk
fluid motion, which `BREADTH.md` places outside this engine's authority.
Their chemistry is computed — the fizz, the layers, the surface-tension
event all fire. What is missing is motion, and it is missing on purpose.

Listed as *partial* they read as a promise, and a promise nobody intends to
keep is worse than a refusal: it inflates the outstanding list, and the
next person to plan work off this file will cost themselves an afternoon
discovering the decision was made years ago. They are now **declined**.

That is a different correction from the four stale rows. A stale row said
something that had stopped being true. These two said something that was
never true — not about the bench, but about the intention.

### K16: the phase was in the database and the species was not (2026-09-04)

The copper-coin row was the one I most expected to be expensive, and it was
the cheapest of the five. The bench had been saying:

```
not yet modelled — a real beaker would not stay like this: the solution is
supersaturated against Atacamite (SI +2.8). Those phases are in
minteq.v4.dat but not in this lab's registry, so nothing can precipitate
out of it here
```

Every word of which is load-bearing. The database defines the phase. The
solver computes the saturation index. `derived::build` matches database
phases to registry solids **by composition**, so the only thing standing
between that message and a green solid was a species entry with the
formula `Cu2ClH3O3`.

One species — atacamite, Cu₂Cl(OH)₃, 213.566 g/mol, 3.76 g/mL, green — and:

```
v1: 0.0027 mol atacamite (green copper corrosion) precipitated ↓
v1: The liquid is blue and so cloudy you cannot see through it, there is
    green atacamite (green copper corrosion) at the bottom.
```

That is the chemistry of the experiment: acid takes the dull oxide off,
chloride puts the green back, and the green has a name and a formula. It is
also, incidentally, why a coin left wet in salty water goes worse than it
started.

**I got this wrong in the re-audit two sections down and have corrected it
there.** I read the message as saying the phase was in a *database* we were
not using, and called it a routing question. It says *registry*. The
distinction is exactly the one a peer session's new cause taxonomy draws —
`PhaseNotInRegistry` is in our gift, `NotInAnyDatabase` is in nobody's —
and I had the two confused while writing the section that was supposed to
stop exactly that.

**K40 is the same shape and is not closed by this.** Cooling copper sulfate
reports Langite, Antlerite and Brochantite, three copper hydroxy-sulfates,
and each needs its own registry entry with its own reviewed data. The route
is now proven; the work is four more species rather than a mechanism.

### KID-10b: an odour is a question of how much (2026-09-04)

KID-10 taught `waft` to match odour rows by Brønsted family, because
vinegar poured into water leaves `CH3COO-` in the ledger while the odour
table is keyed on `CH3COOH`, and the bench had been saying "no odour a
careful waft detects" over a beaker of vinegar. That fixed a real bug.

**It also asserted the converse, which is false.** The relation has no
direction, so the same rule made **sodium acetate smell of vinegar** and
**ammonium chloride smell of ammonia** — salts of the odorous thing
reported with exactly the confidence of the odorous thing. A peer session
found it by running the two, and it is this file's own defect class again:
a test that is symmetric standing in for a claim that is not.

The fallback is now gone rather than patched, and it has no job left. A
peer's `PROTONATION_SPLITS` keeps both members of a Brønsted pair in the
ledger, so the odorous molecule is present under its own key whenever it
is genuinely present — household vinegar carries 0.88 mol/L of
undissociated acid, a pH 8.75 acetate solution carries 7.65e-6, and both
are real entries rather than reconstructions.

What replaces it is the question the old rule never asked: **how much?**

* A gas in the headspace is not gated at all. It has already reached the
  nose.
* Anything dissolved must reach a floor that belongs to the substance, not
  to the function. Ammonia's is 1e-5 mol/L and hydrogen peroxide's is
  1e-1 — four orders apart, because you smell ammonia far below the
  concentration at which you smell peroxide, and 3% peroxide barely smells
  at all even neat.

Those floors are curated teaching values and the module says so. They are
not measured detection thresholds and no claim is made about any
individual nose. What they are is *per substance*, which is the part that
carries information: a single global threshold would have been a fudge
factor, and thirteen different ones are a small table of facts.

Verified on the four cases that matter: vinegar smells and sodium acetate
does not; ammonia solution smells and ammonium chloride does not; the ester
still smells, which was the regression risk, since `bio-102` asks whether
an ester can smell fruity when its reactants do not. Corpus drift 0 and no
lesson golden moved.

### K51 closes as a refusal, and that is the right shape (2026-09-04)

The reusable hand warmer is a sodium acetate solution held far past
saturation: click the disc, the trihydrate crystallises on the scratch, and
the heat of crystallisation is the product. This bench cooled one from
65 °C to 8 °C and **nothing happened and nothing was said**, which is the
worst of the three possible answers — worse than refusing, because a
learner cannot tell a boundary from a bug.

It cannot be fixed by a datum, and it cannot be fixed by choosing another
database. A peer session searched the `PHASES` section of every `.dat`
vendored with iphreeqc — wateq4f, minteq.v4, minteq, pitzer, sit, llnl —
and **there is not one acetate solid phase in any of them.** That is not a
shipping choice this project made; nobody's PHREEQC database carries one.
`saturation_moves` cannot help either: it works on undissociated molecular
solutes, and the aqueous engine has already split this salt into sodium and
acetate ions, so there is no `NaOAc` portion for KID-7's machinery to find.

So the refusal is the deliverable. The salt is reconstructed from its ions
— only as present as its scarcer one, so a beaker of table salt with a
little acetate in it is not a concentrated acetate solution — compared
against a curated solubility, and the bench says what it cannot do:

```
not yet modelled — the crystallisation of sodium acetate: 0.488 mol is
dissolved against a limit of 0.283 mol at this temperature, and the solid
it would crystallise as is sodium acetate trihydrate, and no PHREEQC
database vendored with this project defines any acetate solid phase at all
```

`lessons/hand-warmer.lab` exists to be read beside `borax-snowflake.lab`,
because the pair makes the point neither makes alone: two solutions past
their limit on cooling, one of which crystallises and one of which cannot,
and **the difference is not the chemistry a child sees — it is whether
anybody has written the phase down.**

One implementation note worth keeping: the refusal is emitted from both
`equilibrate` and `equilibrate_delta`. ARCH-012 requires the delta path to
say everything the direct path says, and a line like this one — which
exists purely to break a silence — is exactly what would go missing in a
host that computes deltas.

### K52: the contrast the bench draws better than the numbers do (2026-09-04)

Borax needed a solubility at two temperatures and nothing else — KID-7's
rock-candy machinery had been waiting for it since the day before. The
values carry one modelling choice, stated rather than applied quietly:
handbook solubilities are for the decahydrate Na₂B₄O₇·10H₂O, which is also
what really crystallises out of a cooling solution, and this registry has
only the anhydrous salt. So the anhydrous species stands in for the
decahydrate at the decahydrate's solubility, scaled by 201.22/381.37 — 4.7
and 52 g/100 mL become **2.5 and 27.4**. The amounts are right; the solid
carrying them is a stand-in, and the source record says so.

What makes the row worth more than a datum is what happened when the same
cooling was applied to sugar for contrast:

```
borax    25 g into cold water  → 0.0202 mol dissolved, the rest sits there
         heat to 81.8 °C       → 0.0879 mol more dissolves
         cool to 13.6 °C       → 0.0957 mol precipitated ↓
sucrose  cool from 57.5 °C     → supersaturated at 1.25× saturation,
                                  and it stays dissolved
```

One mechanism, two substances, two different answers — and they are the two
answers a kitchen gets. A borax snowflake grows overnight on its own; rock
candy sits there refusing to start until you give it a string. The lesson's
prose originally explained the contrast by quoting the two solubility
ratios, which is the smaller of the two things the bench had to say.

**A change to an existing lesson, checked rather than assumed:** the
solubility limit changed `slime.lab`, because borax now reports what
dissolved instead of only its unspeciated caveat. The gel still forms —
*"the glue in v2 stops running and starts stretching"* — and the doses
there are far below the new limit, so nothing was lost. The golden was
diffed lesson by lesson rather than in bulk, which is how that was
established rather than hoped.
### The audit had drifted from the bench (re-audit, 2026-09-04)

Before closing the remaining rows I re-ran every one of them against
today's binary rather than trusting what this file said about them. Three
were describing a bench that no longer exists, and I only looked because
the corpus sweep above had just made the point that a statement nobody
re-checks is a statement nobody can trust.

| row | this file said | the bench says today |
|---|---|---|
| K41 | unreachable — "`grind v1 CaCO3` refuses" | it grinds: *"chalk ground to 50.0 µm — about 0.221 m² surface area"*, the curated equation fires and 0.0209 mol of CO₂ comes off. Add the chalk **before** the water and the row runs |
| K52 | unreachable — "no borate in the registry" | borax landed with KID-14. It dissolves; what is missing is a solubility curve, not a species |
| K35 | **silent miss** — "emitted nothing" | it refuses in words: *"neither a metal of the series nor a dissolved metal ion, so there is nothing to be an electrode"*. Silent miss → honest miss |

An audit that has drifted is the same defect as everything else in this
file: a statement that was true when written, believed afterwards because
nothing re-asked it. The lesson is not "re-read the file" but that the
verdict matrix needs re-running whenever it is used to plan work, which is
what `tools/curiosity-answer-invariance.py` does for the corpus and what
this section does by hand for the thirty.

**The re-run also sharpened the classification of what is left.** The
remaining rows are not one problem:

* **Shelf gaps** — the experiment needs a material the shelf has not got:
  K10 (honey) and K13 (lemon juice). Each is a recipe or a species with
  reviewed data, not a model.
* **A species the registry has not got** — K16 and K40 answer with a
  boundary that names its own cause. **I misquoted it in this section when
  I first wrote it**, as "those phases are in minteq.v4.dat but not in this
  database", and called it a routing question. The message says *"not in
  this lab's **registry**"*, which is a different thing and a cheaper one:
  the loaded database defines the phase, and `derived::build` matches
  database phases to registry solids **by composition**, so the phase
  becomes available the moment a species with that formula exists. Not
  routing — one registry entry. Corrected here rather than quietly, because
  a re-audit that introduces its own inaccuracy has earned no authority.
* **One datum** — K52's borax needs a solubility at two temperatures, and
  KID-7's rock-candy machinery does the rest.
* **A bounded observable** — K32's float-or-sink needs no new data at all:
  KID-13 computes the liquid's density and KID-19a the solid's, and
  nothing says which way the object goes.
* **A mechanism each** — K35 (water electrolysis), K51 (crystallisation on
  demand for an electrolyte), K45 and K13 (browning).
* **Stated non-goals** — K09's lava-lamp blobs and K49's soap-driven boat
  are bulk fluid motion, which `BREADTH.md` places outside the engine's
  authority. They are listed as *partial* here, which reads as work
  pending. They are not pending; they are declined, and the matrix should
  say so.

### Fifty-five questions ask a comparison; six build something to compare (2026-09-04)

The audit found `mat-012` in the curiosity corpus by accident: a prompt
asking *"How can density distinguish copper, zinc, and aluminium pieces?"*
whose script weighed five grams of each on a balance. Five grams of copper,
five grams of zinc and five grams of aluminium all weigh five grams. The
row matched its own `expected`, appeared in no mismatch list, and had read
as evidence of coverage for as long as the corpus had existed.

That is the one class of defect here that never surfaces as a failure — a
**passing** row whose script cannot reach its own question — so it was
worth a sweep rather than a fix. `tools/curiosity-answer-invariance.py`
applies one rule that needs no vocabulary: **a prompt that distinguishes N
things must produce N different answers.** It is validated against
`mat-012` (its pre-fix script scores 3 vessels → 1 answer and exits 1) and
the current corpus is clean.

Getting it right took two corrections, both worth keeping:

* A first version compared whole per-vessel output and found nothing,
  because the *setup echo* differs — `+0.0787 mol copper` against
  `+0.0765 mol zinc`. What the script reads back to you about what you
  typed is not an answer. Only what the bench says back counts.
* `mat-011` (copper wires versus iron) has two subjects and one answer, and
  it is not this defect: both vessels honestly refuse, because metal
  conductivity is not modelled. A repeated refusal is an engine gap, not a
  script that misses its question, and no edit would change it.

**The thin result is the finding.** Zero violations, on eight comparison
prompts — because of 55 comparative questions in the corpus, only **six**
build two vessels to compare. "Does warm dough rise faster than cold
dough", "does crushing magnesium make it react faster", "does hot water
dissolve more sugar than cold" each name two conditions and script one.

Those are not lies today, because most are `missing`: the engine cannot
answer them either way and nothing false is claimed. They become lies the
day their mechanism lands — the row starts passing, on a script that never
built the second condition. That is exactly `mat-012`'s history, which is
why the sweep is checked in rather than run once.

### The volcano's temperature depends on which bottle you pour first (2026-09-03)

Found while checking a peer session's report that two curated reactions
could not fire. Their diagnosis was right and its scope was wrong, and
correcting the scope exposed something worse.

**The reactions are not dead. They are order-dependent.** `CuratedEquilibrator`
sits before the aqueous tail in the stack, so on the step where vinegar is
*added* the ledger still holds `CH3COOH` and the reviewed equation matches:

```
add v1 NaHCO3 5g ; add v1 white_vinegar_5_percent 50mL
  v1: NaHCO₃ + CH₃COOH → CH₃COONa + H₂O + CO₂↑          ← fires

add v1 white_vinegar_5_percent 50mL ; add v1 NaHCO3 5g
  (no equation; carbonate equilibria answer instead)      ← dead
```

Put the vinegar in first — the order most people use, and the order most
lesson scripts use — and the readback has renamed the acid to acetate
before the carbonate arrives. An intermittent bug whose workaround is "add
the solid first" is worse than a dead one: someone finds the workaround by
accident and never learns why.

**And the two routes disagree on the sign of the temperature change:**

| order | route | result |
|---|---|---|
| soda first | curated | 25.0 °C → **23.6 °C**, cools 1.4 K |
| vinegar first | aqueous | 25.0 °C → **31.6 °C**, warms 6.6 K |

Same reagents, same amounts, same net reaction. Vinegar and baking soda is
one of the very few kitchen reactions a child can *feel*, and what it does
is get cold.

The aqueous route books ≈57 kJ/mol of neutralisation heat off the acidity
it saw cancelled — 0.042 mol × 57 kJ in 100 g of water is +5.7 K, which is
the number — and books nothing for the endothermic half, the bicarbonate
breaking down and the CO₂ leaving. It is modelling H⁺ + OH⁻ → H₂O and
calling that the thermochemistry of whatever consumed the acid.

**The curated route is not right, it merely declines to be wrong.**
`curated.rs` claims no reaction enthalpy at all ("no heat effect is claimed
until a reviewed reaction enthalpy is installed"), so its −1.4 K is
dissolution enthalpies alone and lands on the correct sign partly by luck.
A route that declines to claim beats a route that claims the wrong sign,
and neither of them has the number.

**Status: the order dependence is gone, verified on main.** #351 landed on
2026-09-03 and keeps both members of a Brønsted pair in the ledger, so the
curated route survives a solve. Both orders now fire the equation and both
cool to the same 23.6 °C:

```
add v1 NaHCO3 5g ; add v1 white_vinegar_5_percent 50mL   → 25.0 → 23.55 °C
add v1 white_vinegar_5_percent 50mL ; add v1 NaHCO3 5g   → 25.0 → 23.59 °C
```

It ships with a regression test asserting the two orders agree to within
0.5 K — an assertion that needs no knowledge of the right answer, which is
exactly why it would have caught the original bug.

**The underlying defect is untouched and unowned:** any acid-consuming
reaction with no curated equation still books proton-neutralisation heat as
its whole enthalpy. The volcano is safe because it has a reviewed equation
now reachable in both orders, not because the aqueous tail learned to tell
a neutralisation from a carbonate breakdown.

This is the fifth instance this week of one defect class, and the audit is
not exempt from it. `acidity cancelled → heat` is invariant over *what*
cancelled the acidity, and it read plausibly for as long as everything that
cancelled acid happened to be a neutralisation — exactly as K01's own
verdict above read plausibly for as long as nobody poured the bottles in
the other order. The family: a curated reaction that cannot find its own
reactant after a solve; an odour matched by Brønsted family in both
directions, so sodium acetate smells of vinegar; `−solute_charge` standing
in for titratable protons; a corpus prompt whose script weighs three
five-gram pieces and asks which is denser. The shape is always the same —
**a quantity that claims to be X, is invariant over X, and is believed
because nothing has yet asked it the question that would separate the
two.**

Wave 1 is the whole difference between "a chemist's engine" and "a bench a
child can walk up to". Everything after it adds chemistry; only Wave 1 adds
*reachability*, and reachability is what the audit found missing.

---

# Part 2: the second thirty (audit 2026-09-02)

Run the same way, on the same binary, after KID-1/2/3 landed. These are the
thirty a child meets *next* — the ones with a battery, a magnet, a crystal
or a rate in them.

| # | Experiment | Verdict | What the bench did |
|---|---|---|---|
| K31 | Pull the iron out of the sand | computed | `magnet v1 v2` moves 3 g of iron and names the quartz left behind. `EXP-1` exactly as promised |
| K32 | Which plastics float? | ~~silent miss~~ → computed | closed by KID-19b (2026-09-04). Nothing was missing but the comparison: polypropylene at 0.90 g/mL **floats on top**, polystyrene at 1.05 and PET at 1.38 settle and are named where they went, and a floating solid stops counting as turbidity. The registry had all three densities the whole time |
| K33 | Build a water filter | ~~partial~~ → computed | the reviewed quartz-rich sand surrogate now makes a visible suspension; `filter` transfers clear water, retains solid SiO₂ and conserves the small dissolved share, without claiming to remove microbes or dissolved pollutants |
| K34 | Open-circuit voltage from lemon juice | ~~unreachable~~ → computed | the renamed question matches what the engine actually answers: zinc and copper in the lemon surrogate give a deterministic no-load estimate from computed pH and an explicit unit zinc-ion activity. It does not claim a powered fruit battery; geometry, electrode spacing and area, internal resistance, loaded current, power and lifetime remain outside the model |
| K35 | Split water with electricity | ~~honest miss~~ → computed | conductive water with an inert sulfate electrolyte gives hydrogen and oxygen by Faraday's law in the 2:1 amount ratio; `lessons/water-electrolysis.lab` contrasts that with pure water, which remains correctly refused as an insulator |
| K36 | Turn an iron nail copper | computed | Fe + CuSO₄ → Cu + FeSO₄, +17 K, orange copper at the bottom. Textbook |
| K37 | Why salt makes ice colder | computed | −3.19 °C from 40 g of salt, freezing-point depression solved |
| K38 | Baking powder or baking soda? | computed | the heat-activated powder resolves to its starch and carbonate and behaves differently from plain soda in cold water |
| K39 | Why soap will not lather in hard water | computed | a declared fatty-soap reagent binds Ca/Mg at 2:1 into a conserved aggregate whose mass follows the consumed soap and actual ion; lather, micelles, builders and commercial formulations remain outside the model |
| K40 | Grow blue crystals | **wrong** → computed | **the fourth stale row, and the only one closed by nobody.** Its three complaints were answered by three separate tasks and the row was never re-read: KID-6 fixed the 109 °C boil with liquid water in the ledger, KID-7 gave cooling solutions their crystals, KID-20 stopped chalcanthite being drawn white. Verified 2026-09-04 — 0.0884 mol precipitates on cooling with the net ionic equation, and the liquid goes from *black* (too concentrated to see through) to *blue* as the copper leaves it Its three greens — Langite, Antlerite, Brochantite — were added to the registry on the same day and now precipitate rather than being refused |
| K41 | Powder fizzes faster than a lump | ~~unreachable~~ → partial | **stale verdict, corrected 2026-09-04.** `grind` works when the chalk goes in before the water: "ground to 50.0 µm — about 0.221 m² surface area", and the curated equation then fires. What is still missing is the *contrast*: the acid-carbonate route carries no rate, so powder and lump fizz identically |
| K42 | Lemonade that changes colour | computed | bromothymol blue, blue → yellow, from the absorption spectrum |
| K43 | Settle a sour stomach | computed | Mg(OH)₂ neutralises and the excess stays as a solid, which is exactly why the real medicine is a suspension |
| K44 | An eggshell in cola | computed | cola surrogate reads pH 2, and only the acid it actually carries dissolves shell — an honest partial, and a better lesson than the myth |
| K45 | Stop an apple going brown | computed | a prepared cut surface browns with oxygen and time, while ascorbate inhibits the bounded visible response; enzyme turnover, texture, flavour and food safety are not claimed |
| K46 | Which metal reacts first? | computed | Mg vigorous (+23 K), Zn slower, Cu refuses with the overpotential explanation. The activity series, computed |
| K47 | A fire extinguisher in a jar | ~~silent miss~~ → computed | KID-12: the fire exists now, and the extinguisher works the way a real one does — by dilution. The wax is untouched, the oxygen is untouched, and the flame will not start, because 14% of the gas being oxygen is not enough |
| K48 | Colours climbing a chalk stick | ~~honest miss~~ → computed | same refusal as K26, and fixed with it by KID-9 |
| K49 | A boat pushed by soap | **declined** | the surface-tension event fires and is computed; nothing moves, and nothing will. A boat crossing a bowl is bulk motion under a surface-tension gradient — the same authority boundary as K09, and the same correction: refused rather than owed |
| K50 | A pH map of the kitchen | **wrong** → partial | vinegar 2.41, soda 10.02, washing soda 11.57, **lemon juice 1.86** — and apple juice still a sentence rather than a number. KID-20 gave it the malic acid its tartness is made of, and the engine says precisely why it cannot price that acidity. `lessons/kitchen-ph.lab` now makes the pair the lesson: same shelf, same kind of juice, same kind of fruit acid, and the only difference is that minteq.v4 defines a citrate and nothing anywhere defines a malate |
| K51 | A hand warmer that crystallises | **wrong** → **stated boundary** | closed 2026-09-04, and closed as a refusal rather than a feature. The bench cooled a supersaturated pouch from 65 °C to 8 °C and said *nothing at all*; it now says how far past saturation the solution is (0.488 mol against 0.283) and why the solid cannot appear. A peer session searched the `PHASES` section of **every** `.dat` vendored with iphreeqc — wateq4f, minteq.v4, minteq, pitzer, sit, llnl — and there is not one acetate solid phase anywhere. Not a shipping choice; nobody's PHREEQC database has one |
| K52 | A borax snowflake | ~~unreachable~~ → computed | the row was stale before it was fixed: borax landed with KID-14 and only wanted a solubility curve. With 2.5 g/100 mL at 20 °C and 27.4 at 100 °C, 25 g into cold water leaves most of it sitting there, heating dissolves it, and cooling returns **0.0957 mol** as solid — while the same cooling makes a sugar syrup *supersaturate* instead |
| K53 | Salt or sugar on the ice? | computed | −2 °C against +1 °C: the colligative contrast a child can feel |
| K54 | Three gases, three tests | computed | limewater goes milky and the magnesium is used up. The script did not use `test`, because the audit did not know it existed — a separate probe confirms `test v1 splint` answers "glowing splint — negative" over hydrogen, so `EXP-31` works and was invisible (KID-17) |
| K55 | Nothing is lost if nothing escapes | computed | 165 g sealed, 163 g once opened. The conservation lesson, in two numbers |
| K56 | Bubble mixture that lasts | ~~partial~~ → computed | KID-11 made foam a property of gas meeting a surfactant rather than of one reaction id, so any gas-making vessel with a declared surfactant foams and drains on the recipe's own half-life — 1.375 L falls to 0.866 L over two minutes against a 180-second half-life |
| K57 | A tower of sugar water | partial | the two solutions mix, which is correct; a slow pour that would not mix is not modelled. KID-13 gave the bench the density of a sugar solution — solute volume included, which it was not before — and KID-19a gave the learner a way to read it, so the *number* a tower would be built on is both right and askable; the layering is the part that is still missing |
| K58 | Instant snow from a powder | computed | bounded, mass-balanced sodium-polyacrylate water uptake; no swelling time, volume, texture, salinity or pH response |
| K59 | Luminol light in warm and cold water | ~~partial~~ → computed | temperature changes relative intensity and lifetime, both samples fade on the clock, and ordinary engine chemistry consumes peroxide; commercial peroxyoxalate chemistry, luminol product speciation and absolute photon yield remain explicit boundaries |
| K60 | One indicator, five jars | computed | phenolphthalein purple → colourless across the neutralisation |

**Current shipped-catalog tally (2026-09-05): computed 52 · partial 5 ·
boundary 1 · declined 2 · unreachable 0.** K33, K34 and K59 are the latest
honest promotions; the five remaining partials now link to guided evidence or
explicit model boundaries. Older audit labels in this narrative remain as
history.

## What the second thirty added to the register

- **KID-18 — a half-cell must explain what kind it is.** The ordinary
  `cell` path still requires a metal in a solution of its own ion. A lemon
  cell is now a deliberately narrower zinc/acid/copper estimate, labelled
  with its missing zinc-ion activity and load boundaries. The inert-electrode
  `electrolyse` path separately splits conductive sulfate water; pure water
  remains refused because it cannot carry the imposed current.
  *Acceptance:* the shipped lesson demonstrates Faraday's law; an inert
  electrode is something a learner can add; the refusals name what is
  missing at every register.
  **KID-19b landed 2026-09-04: the observable half**, and the acceptance
  line above is met — a solid whose density is known now floats or settles
  against the liquid it is in, and `look` says which. Polypropylene floats,
  polystyrene and PET sink, and `lessons/float-or-sink.lab` is the
  float-sink tank a recycling plant uses, in a glass.

  A floating solid also stops counting towards turbidity. Five grams of
  polypropylene made the water "so cloudy you cannot see through it",
  which is the same defect the plated-metal branch beside it was written to
  prevent: something sitting on the surface is not a suspension.

  **And it found a defect in KID-19a, one day old.** The second half of the
  experiment raises the water's density with salt so that polystyrene
  floats too. The brine reads **exactly 1.00 g/mL** — because all twelve
  ion species in the registry carry a density of exactly 1.0, a structural
  default that no provenance line even mentions. Dissolved salt adds its
  mass and an equal volume of "water", so the density never moves; real
  brine is about 1.2.

  KID-19a's own tests missed this because they used sucrose, which has a
  real measured density of 1.59 and therefore works. **A placeholder that
  produces a believable number is the hardest kind to see**, and it took a
  different experiment asking a different question to surface it. Partial
  molar volumes are not something this registry can invent, so the meter
  now answers *and says what it leaves out*: the solvent's figure is right
  and the solutes' is missing. The lesson ends on that refusal rather than
  letting the next reader try the salt trick and believe it.

- **KID-19 — density is data, not an observable.** Four polymers with
  reviewed densities float and sink nowhere. `EXP-12`'s data landed and its
  quest is authored, but nothing in the vessel picture separates them.
  *Acceptance:* a solid whose density is known settles or floats against the
  liquid it is in, and `look` says which.
  **KID-19a landed 2026-09-03: the measurement half.** The registry knew
  every density in it and there was no way to ask. `measure v1 density`
  (also spelled `hydrometer`) now reads a liquid through the solution's own
  density — solute volume included, which is the arithmetic KID-13 had to
  fix — and a single dry solid through the substance's reviewed value.

  It came from the curiosity corpus rather than from the thirty. `mat-012`
  asks *"How can density distinguish copper, zinc, and aluminium pieces?"*
  and its script weighed five grams of each on a balance: five grams of
  copper, five grams of zinc and five grams of aluminium all weigh five
  grams, so the script exercised the one measurement that cannot answer its
  own question. There was no other instrument to reach for when it was
  written. Now there is, and the three vessels read 8.96, 7.14 and 2.70
  g/mL against three identical balance readings.

  A density belongs to ONE substance, so a heap of two powders refuses and
  names both — it has a mass and a volume and no density anyone should be
  told. An empty vessel refuses for its own reason. A hydrometer floats in
  the liquid, so a liquid answers even with solids sitting in it.

  **A rounding bug this exposed.** Every lv1 instrument reading went
  through `{value:.0}`, so the density meter announced aluminium as
  "3 g/mL" — and 2.7 against 8.96 is the entire content of the row, while
  "3" against "9" is a different and worse claim. Readings below ten now
  keep one decimal, which is roughly how a person reads a real dial, and
  25 °C is unaffected.

  **Stated boundaries:** the volume of a piece is computed from the
  substance's reviewed density rather than measured by displacement, so
  this is the density of the *material* and not of the *object* — a hollow
  or bubbled piece reads the same here and floats in a real bucket. Solute
  volumes use the additive-volume approximation the rest of the bench uses.
  *Still open in KID-19:* the observable half — a solid that floats or
  sinks in the vessel picture, which is what the acceptance line above asks
  for.
- **KID-20 — the household recipes have gaps that read as lies.** Apple
  juice resolves to water and sucrose, so it is not acidic; a kitchen pH map
  reports nothing for it. Chalcanthite is drawn white. These are registry
  data errors rather than model gaps, and they are cheap.
  *Acceptance:* every recipe whose real-world identity is defined by an acid,
  a colour or a hazard carries it, with a source; a lint refuses a food or
  drink recipe with no flavour-acid component.
  **Landed 2026-09-03**, and the apple-juice half turned out to be a sharper
  finding than "a missing component". The recipe *explained* itself: it
  carried "most of apple juice's sugar is fructose and glucose, and neither
  is an installed species" and "malic acid ... is not in the registry". Both
  sentences were true when they were written. Fructose, glucose and malic
  acid are all shipped species now, each with its own reviewed solubility
  limit — so the recipe was resting on two reasons that had quietly expired,
  which is worse than resting on none, because an expired reason reads as a
  current one. The juice now resolves each sugar as itself in the cited
  proportions and carries malic acid as the acid its tartness is actually
  made of.
  It still reports no pH — but for a stated reason instead of by omission:
  *"no shipped database defines a malate species, so its two carboxylic
  protons are not in this pH ... the real solution is more acidic than it
  says."* That is the right answer. What the original recipe refused to do
  — borrow an acid the engine happens to have and compute a pH from the
  wrong molecule — is still refused, and a test holds that refusal.
  Chalcanthite carried the appearance string "deep blue crystals" and no
  sRGB, so the appearance layer fell back to its default pale grey and
  described the product of "grow blue crystals" as **white**. It is blue.
  The lint the acceptance asks for is in `material_recipe.rs`: every drink
  recipe must carry an acid component, against an explicit reviewed list of
  what counts — because deciding what a drink's acidity is made of is a
  curation act, and this is where that decision is recorded.
- **KID-21 — the grammar's order traps.** `grind` after the solid has
  dissolved, `filter` into a vessel that does not exist yet, `cell` before
  the half-cells are half-cells: three refusals that are each correct and
  none of which say what to do instead.
  *Acceptance:* a refusal that has an obvious remedy states it.
  **Landed 2026-09-03.** All three now carry the remedy:
  - `no vessel v2` → *"make it first with `new`, which creates the next free
    vessel"*;
  - `contains no solid CaCO3 to grind` → *"grinding changes a solid's
    particle size, so it has to happen before the solid dissolves, not
    after"* — which is the sentence a learner who has just watched chalk
    dissolve actually needs;
  - `one of them isn't a proper half-cell yet` → *"each side needs a metal
    standing in a solution of its own ion: zinc in zinc sulfate, copper in
    copper sulfate"*, since what the learner is missing is the definition.
  Being correct and being useful are different properties, and a refusal
  that has an obvious remedy and does not state it is only the first.
- **KID-22b — `kero study --vary` sweeps moles whatever the line said.**
  Found while calibrating KID-5: `--vary add:v1:Fe=1..2` on a line reading
  `add v1 Fe 1g` replaces the parsed amount with one *mole* of iron, 55.8 g,
  and the resulting curve looks like a rate law that has stopped responding.
  KID-5 made the unit explicit in the provenance and added a stderr warning
  when the swept line is written in grams or millilitres. The remaining
  question — whether the sweep should instead follow the line's own unit —
  is left open, because converting needs the molar mass and changing the
  meaning of an existing flag is not a calibration decision.
- **KID-22 — `react` and `test` exist and are invisible.** `react v1
  esterification` runs; `test v1 splint` runs. Neither is in `kero --help`
  or the REPL's `help`, which between them name 24 and 18 of the grammar's
  31 verbs. Folded into KID-17.

---

# Part 3: the 12–16 slice (spot audit 2026-09-02)

Twelve experiments from the older band, run to test the prediction that the
engine's shape inverts there. It does.

| # | Experiment | Verdict | What the bench did |
|---|---|---|---|
| T01 | Titrate a weak acid to a colour endpoint | computed | the burette walks, the curve is real, and when the endpoint is out of reach it says which of the two reasons it is |
| T02 | A buffer against an acid | computed | acetate buffer holds pH 4.55 while plain water crashes to 2.04 under the same dose |
| T03 | The Daniell cell | computed | 1.102 V open-circuit against E° 1.104, with the direction of electron flow and the caveat that no current is drawn |
| T04 | The common-ion effect | computed | adding calcium chloride to limewater precipitates Ca(OH)₂ out of a solution that was clear |
| T05 | Thiosulfate clock | computed | 0.0476 mol reacted in 60 s, sulfur as a solid, the rate law integrated |
| T06 | Gravimetric sulfate | computed | 2.33 g of barium sulfate, filtered and weighed, exact |
| T07 | Beer–Lambert by dilution | computed | 0.48 absorbance at 525 nm, then 0.24 after a twofold dilution |
| T08 | Gas pressure with temperature | computed | sealed headspace, 127.33 kPa, with the dissolved fraction accounted |
| T09 | Make an ester | **silent miss** | ethanol + acetic acid + sulfuric acid, heated to 108 °C for an hour, produces nothing and says nothing. `react v1 esterification` *does* run it — but the verb is in no help text, so heating the flask, which is what a student does, silently fails |
| T10 | Half-life | (not exercised — the nuclide bench rides `add`, see `EXP-49`) | |
| T11 | Solubility product, three ways | computed | AgCl to cerargyrite, and `explain` answers the same question of every dataset — including "pitzer.dat cannot express this problem (no Ag, N(5))" |
| T12 | Electroplating | computed | with the copper electrode present, 0.0833 mol of copper and a mass that moves |

**Ten of twelve computed, and the two that are not are a missing verb in a
help text and a nuclide bench I did not drive.** The engine was built for
this age band; the children's corpus was the stress test, and it is where
the work is.

So the sequencing recommendation stands and sharpens: **Wave 1 and Wave 2 of
the KID register buy far more than any new chemistry would.** The 12–16 band
needs KID-17's help text and the organic tail (`EXP-44` and friends); the
7–12 band needs eight mechanisms and a vocabulary it can reach.

## Part 4: the thirty after that

Recorded so the corpus does not restart from zero, and deliberately not
audited until Wave 2 lands — a corpus run against a bench you already know
will refuse teaches nothing.

Ages 7–12, third thirty: sun-print paper; a lemon-powered clock; the
mentos-and-cola nucleation demo; making butter by shaking cream; red-cabbage
paper strips; a solar still; growing an alum crystal; the "burning" of a
sugar cube with ash; oxygen and a candle under a jar; copper patina; blowing
up a balloon with yeast at three temperatures; a pH-driven colour clock;
milk fireworks with different fat contents; cleaning silver with foil and
salt; the density of diet versus regular cola; rusting in three atmospheres;
freezing point of sea water; how much salt a litre will hold; the smell of
esters in fruit; hard-boiled versus raw egg spin; making chalk from
limewater; a CO₂ fire snake; an onion-skin dye bath; separating salt and
sand; the electrolysis of brine; a soda-can crush; growing a stalactite from
Epsom salt; testing antacid tablets against each other; a starch-and-iodine
secret message; and a two-week limescale kettle.

Ages 12–16, first thirty: standardising permanganate; back-titration of an
antacid; a rate law by initial rates; activation energy from two
temperatures; Le Chatelier on three stresses; a solubility curve measured;
the iodine clock's order in iodide; a nickel–iron cell built from the series;
the pH of salts explained by hydrolysis; a diprotic titration to two
endpoints; conductimetric titration; an equilibrium constant measured; the
common-ion effect made quantitative; buffer capacity by titration; enthalpy
of neutralisation by calorimetry; Hess's law in three steps; a gravimetric
chloride determination; an ester synthesis with a yield; a saponification
and its soap; distillation of an azeotrope; fractional crystallisation;
paper chromatography with an Rf table; a redox titration of iron; catalysis
compared homogeneous against heterogeneous; a Beer–Lambert calibration line
and an unknown read off it; the effect of ionic strength on an equilibrium;
a Nernst plot over four dilutions; corrosion under differential aeration; an
electrolysis with a measured Faraday constant; and a full unknown-salt
identification.

The audit's own prediction, recorded now so it can be checked later: the
first list will run about half, and the second about four fifths.

## Twenty-six materials, and the four kinds of gap they exposed
*2026-09-04*

The corpus had 145 rows the engine could not reach, and 95 of them stopped at
the same place: the parser did not know the word. Not a chemistry gap — a
vocabulary gap. Eighty-eight distinct tokens, most wanted by exactly one row.

Twenty-six of them are now on the shelf: twelve inert materials (fused silica,
borosilicate, coloured glass, quartz, porcelain, glazed ceramic, pumice, clay,
stainless steel, galvanised steel, painted iron, expanded polystyrene) and
fourteen foods and fibres (apple, potato, onion, cabbage, bread, pasta, rice,
honey, butter, cream, egg white, gelatine, albumin, cotton). Sixty-three
corpus rows moved out of `missing`, against two regressions, with the
expectation-mismatch count flat at 85 throughout.

Writing them was mostly not chemistry. It was deciding, twenty-six times,
what the bench is allowed to claim. Four patterns came up often enough to
name.

**The property the material is bought for is the one in the remainder.**
Borosilicate glass is 81% silica and 19% conserved, and the conserved part is
the boria — which is the entire reason borosilicate has its own name and its
own price. Stainless steel is worse: the chromium that makes it stainless is
not installed, so the bench holds an object whose iron can be attacked and no
representation of the film that stops the attack. Any corrosion result there
is a result about plain iron wearing a stainless label. Both recipes say so.

**A bulk density that nothing reads.** Six recipes were written carrying
careful bulk densities — pumice 0.64, expanded polystyrene 0.03, cotton 0.08,
apple 0.85, potato 1.08, rice 0.85 — and six notes explaining what the bench
would therefore do: float the apple, sink the potato, float the pumice for
half the right reason. Every one of those sentences was false, and I wrote all
six before running one of them.

Then I ran `add v1 pumice 50g` to a beaker of water. Pumice sank, and the
water went opaque, because 70% of it resolves to silica and silica is 2.65.
`bulk_density` is read by exactly one caller — the raisin bubble-ride in
`buoyancy.rs` — and the general float-and-sink test reads the density of each
*species* a material resolves into. There is no material-level buoyancy at
all. The numbers are right, they are in the file, and nothing consumes them.

This is the session's own recurring defect committed by the person cataloguing
it: **a claim about a mechanism, believed because it was plausible and because
nothing asked the question that separates it from its absence.** It cost one
command to find. All six notes now say what actually happens — that the
density is recorded and does not float or sink anything — and the gap is worth
its own task: bulk density is the natural key for buoyancy and the wiring does
not exist.

**A coating reported as a fraction is not a coating.** Galvanised steel is
zinc on iron, and the recipe says 3% zinc by mass, which the bench mixes
through the object. The point of galvanising is that the zinc is on the
*outside* and corrodes first; that is a geometry argument about which metal
the liquid reaches, and these recipes have no geometry to make it with. So
the bench consumes zinc and iron together in whatever ratio the chemistry
prefers. Painted iron has the identical problem and the identical note.

**Two entries for one substance, split on what earns its resolution.** The
shelf now has both a cabbage and a red-cabbage indicator juice. The juice has
its anthocyanin resolved and changes colour with pH; the head has the same
pigment in its conserved remainder and will not. That looks like an
inconsistency and is a deliberate one: the juice is an extract made for the
purpose, and the head has the pigment locked in cells this bench cannot break
open. Adding a cabbage to acid turning nothing pink is the honest outcome.

The recurring shape underneath all four: a number that is right, sitting where
the thing that produces it is absent. Same defect class as the brine that read
exactly 1.00 g/mL and the balance that was invariant over what it weighed —
and the reason each of these recipes carries its own paragraph saying which
half is missing.

Five foods cannot do the thing they exist to demonstrate. Egg white does not
set at 65 °C, gelatine does not gel, cream does not whip, albumin does not
denature by heat or acid or alcohol or salt, and onion does not sting the
eyes. All five are protein or enzyme behaviour, no protein species is
installed, and the mass sits conserved. That is one gap wearing five names,
and it is the largest single thing standing between this shelf and the
kitchen-chemistry half of the corpus.

## Four test failures, and what each one turned out to be
*2026-09-04, later*

Running the whole workspace rather than one crate found four failing targets.
Not one of them was what it first looked like, and the sequence is worth
recording because three of the four were *caused by a previous improvement*.

**`cargo test` is fail-fast by test binary, and that hid two of them.** The
three failures sort alphabetically — `element_coverage`, `frozen_behavior`,
`registry_snapshot` — and each run stopped at the first, so each fix revealed
the next and cost another full suite. A partially-green run means "nothing
failed *before* the first failure", not "everything else passed".
`--no-fail-fast --workspace` is the default worth having.

**The safety screen refuses a species it has never been shown.** Seven had no
row: the four copper hydroxy-sulfates K40 added, graphite and diamond, and the
ammonium nitrate behind the cold pack. The first six join the
no-reactive-group arm with their reasons written down — the copper phases are
the same metal in the same oxidation state as the chalcanthite already there,
and the two carbons are allotropes of the `C` already listed. Ammonium nitrate
does not: it is a strong oxidiser and gets `OxidizerStrong`. The shelf's other
nitrates being ungrouped is not a reason to repeat that, because sodium and
potassium nitrate need a fuel and a match, and ammonium nitrate carries its
fuel in the cation.

**A more accurate chemistry broke three displacement tests, and the mechanism
is a chain.** K40 put the copper hydroxy-sulfates in the registry. A 0.1 mol/L
copper sulfate solution at pH 3.9 is supersaturated in antlerite by about 0.8
log units, so the bench now precipitates 1.11e-5 mol of it — and the net ionic
is `3 Cu²⁺ + SO₄²⁻ + 4 H₂O → antlerite + 4 H⁺`. Three consequences followed,
in order:

1. Three copper per formula unit is 3.33e-5 mol of copper parked in a solid
   before any magnesium arrives — *exactly* the shortfall in what plated out.
   Mass is conserved; the copper is not missing, it is elsewhere.
2. Those four protons dropped the pH past the threshold that makes `acid`
   true in the displacement bystander pass.
3. That flipped a branch. Silver in copper sulfate is above copper **and**
   above hydrogen; both sentences are true; the code preferred the hydrogen
   one. So the bench stopped saying the thing the beaker was set up to
   demonstrate and started saying a true aside instead.

The third is the same defect as "an aside outranking the answer" that the
classifier work was about, in a different file, arrived at from the opposite
direction. The metal-versus-metal comparison now wins wherever there is
another metal's ion to make it against; the acid sentence is for a metal in
acid with no such partner, which is the case it was written for. The
tolerances were widened to 4e-5 with the arithmetic written beside them, so
the trace stays visible rather than being tuned away.

A real beaker of copper sulfate stays clear blue — because nucleating a basic
sulfate is slow, not because it is disfavoured, which is why stock solutions
are acidified. The bench computes equilibrium and has no nucleation, so it
takes the thermodynamic answer. That is a limitation to state, not to hide.

**A prompt was being graded on its neighbour's evidence.** `aq-091` returned
`curated` from a smoke run and `computed` from a full one — same script, same
binary, deterministic in each. `last_routes` is cleared by
`SolverStack::equilibrate`, so a step that never equilibrates (`new`) leaves
the previous step's routes standing, and at the top of a script that is the
previous *prompt's*. The classifier then read a `curated` route that belonged
to a different experiment. One line to fix.

What makes it worth a paragraph is how it stayed hidden. The full corpus is
unaffected — zero rows move — because there every neighbour happened to agree.
Only the smoke subset disagreed, and only because a material added in this
branch made `aq-091` run at all. **The bug was latent for exactly as long as
the row was `missing`; closing the row is what exposed it.** Which is an
argument for closing rows that has nothing to do with the score.

## Where this leaves the kids' experiments
*2026-09-04*

First thirty: **computed 25 · partial 4 · declined 1**, unchanged by this
branch — the materials work moved corpus rows rather than verdict rows.

What the second thirty and the 12–16 list now need is a short list, and it is
mechanisms rather than substances:

- **Protein.** No protein species is installed. Egg white, gelatine, cream,
  albumin and onion are all on the shelf now and none can do the thing it is
  used to teach. Five demonstrations, one gap.
- **Enzymes beyond amylase.** `pepsin`, `lactase`, `protease`, `lipase`,
  `catalase` are named by corpus rows and only amylase has a reaction.
- **Photosynthesis.** `pondweed`, `leaf`, `chlorophyll`, `germinating_seed` —
  five rows, and a mechanism the bench has no shape for at all.
- **Browning.** Apple, potato and bread are all present and none browns; it
  needs polyphenol oxidase and a quinone, or the Maillard reaction.
- **Hydrocarbon fuels.** `methane`, `propane`, `butane`, `petrol`, `diesel`
  want species with CEA thermochemistry. The combustion machinery from KID-12
  already exists and has nothing to burn.
- **Nucleation.** Honey is supersaturated and reads grainy from the first
  moment; copper sulfate deposits antlerite a real beaker would not. Both are
  the same missing clock.

The prediction recorded earlier — first list about half, second about four
fifths — still stands and is still unchecked.

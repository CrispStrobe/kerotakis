# Part 1: the thirty-experiment children's corpus (audit 2026-09-02)

> Finished work is not listed here. What landed, and what it taught us, is in
> [HISTORY.md](HISTORY.md). Task numbers are never renumbered and never reused.

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
sentences, and a hazard screen that cries wolf. (See `HISTORY.md` for the
LESSON on what the audit found about auditing itself, in the sections that
followed this one.)

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
   hits sends them somewhere else.

Substances this corpus needs and the registry does not have: red cabbage /
anthocyanin, poly(vinyl alcohol) glue, borax, ammonium nitrate, egg (shell
and membrane), raisin or other dried fruit, lemon juice, gelatin, an
effervescent tablet, glycerol or honey for a density tower, lithium salt
for the flame series, casein as a named colloid fraction, and a paraffin
with combustion data rather than a purely unresolved wax.

**What the unreachable names had already cost us** is recorded in
`HISTORY.md` as a LESSON: landing KID-1 moved exactly one row of the
500-question curiosity corpus (`aq-120`) from `missing` to `computed` — an
alias nobody could type had been recorded as a missing engine capability,
filed against the task that had already built it.

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
| ~~Acetic acid in the odour table~~ | it was *in* the table but keyed on the acid while the solver had turned it into acetate. **Landed as KID-10**, 2026-09-03 (see KID-10b for the follow-up defect this fix itself introduced) |
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
and `main` moves only by PR. Full landing narrative for every task below
lives in `HISTORY.md`; what follows is status, acceptance, and what is
still open.

### Wave 1 — the closed loops (nothing else is worth doing first)

- **KID-1 — The shelf, reachable from the terminal.** `normalize_material_name`
  now treats `_`, `-` and space as one separator; `kero materials` and
  `kero find <word>` expose the cabinet outside the REPL; the unknown-name
  error names `find`; `README.md` gained ten household words.
  *Acceptance:* every one of the 50 recipes resolves from at least one
  whitespace-free spelling of every name it advertises, with a test that
  fails the day a recipe adds a space-only alias; `add v1 household_vinegar
  50mL` works; `kero materials` lists all 50 outside the REPL; the error on
  a misspelling suggests a real key; no two recipes collide under the new
  normalization.
  **Landed 2026-09-02.** Measured on the thirty scripts as first written:
  17/30 ran before, 27/30 after. The three that still fail are content gaps
  — red cabbage and PVA/borax do not exist (KID-8, KID-14), and `filter v1
  v2` still wants its receiver created first (KID-15) — not naming.

- **KID-2 — Curdling must fire with the solver on.** `curdling::observe`
  keyed on `CH3COOH`, which the aqueous solver had already speciated to
  `CH3COO-`, so the dose was always zero and the shipped
  `lessons/milk-curds.lab` never demonstrated its own headline claim.
  *Acceptance:* the lesson produces curds through the full solver stack; a
  test drives it through the same path the CLI uses; the model reads total
  acid *inventory* (undissociated plus conjugate base) rather than one
  species key.
  **Landed 2026-09-02.** The dose now sums species sharing the declared
  acid's non-hydrogen composition, reproducing the number the recipe was
  calibrated against.
  **Stated boundary:** this counts acid inventory, not acidity — sodium
  acetate would still read as a dose though real milk would not curdle from
  it (casein aggregates at its isoelectric point). A pH-driven response
  needs a reviewed pI datum, tracked with **KID-14**.

- **KID-3 — A hazard screen that does not cry wolf.** The L0 screen was
  dose-blind and screened a recipe's own components against each other, so
  1 mL of 1% Lugol into 100 mL of water raised a Danger-level "can
  detonate" banner in the starch and vitamin-C activities.
  *Acceptance:* components arriving from one `MaterialRecipe` expansion are
  not screened against one another; incompatibilities carry a dose or
  concentration floor; the starch and vitamin-C activities run clean; the
  genuinely dangerous pairs (bleach + acid, bleach + ammonia) still fire at
  household strength.
  **Slice 1 landed 2026-09-02** — the self-screening half
  (`SafetyScreen::assess_pour`). K27 goes from two Danger banners to none;
  bleach + ammonia, permanganate + sulfite, and iodine + iodide poured
  separately all still fire.
  **KID-3b is the dose half, still open.** K28 keeps one banner — iodine
  meeting ascorbic acid is a real oxidiser/reducer pair, but at this dose
  is nowhere near "can detonate"; the `real_world` sentence also names the
  wrong reagent pair for this vessel. Fixing the wording needs no new
  number; a severity floor does, and it must be sourced.

- **KID-4 — `ignite` is never silent.** Holding a flame to an unresolved
  material currently emits nothing at all.
  *Acceptance:* every `ignite` on every shelf item produces a line at lv1;
  a test enumerates the shelf and asserts a non-empty response for each.

### Wave 2 — the mechanisms the corpus is actually short of

- **KID-5 — Rusting.** Pull `EXP-34` forward: iron surface area, oxygen
  transport, chloride acceleration, Fe(OH)₂/Fe(OH)₃/Fe₂O₃ product routing,
  a rate slow enough to need `wait` and fast enough to see.
  **Landed 2026-09-02** as a curated kinetic reaction, `iron-corrosion`,
  running the four-arm comparison `EXP-34` asked for (dry, deoxygenated,
  plain water, salt water); every arm holds in a test on the shipped
  binary, with salt water showing the fastest oxide yield and oxygen
  drawdown. Three engine repairs were needed first (a solid product the
  aqueous solver would otherwise dissolve back out, a description that
  named only the single largest settled solid, and `Event::Inert`'s
  wording contradicting an actively-rusting vessel) — see `HISTORY.md`.
  `kero study` gained an `amount:<species>@vN` probe and `--vary` now
  states which unit it sweeps.
  **Stated boundaries:** iron *amount* stands in for iron *area* (a nail
  and filings of the same mass rust at the same speed); the product is the
  registry's anhydrous oxide rather than hydrated rust; an open vessel does
  not draw oxygen from the room.
  **Extended 2026-09-05 (BRD-023)** with two-metal galvanic coupling:
  `crates/kerotakis-core/src/corrosion.rs` makes the lower-E° metal (off
  the `displacement::SERIES`) corrode for both; a `zinc-corrosion`
  companion entry makes the sacrifice real; a barrier table carries
  stainless steel's passive film and painted iron's paint film. **Stated
  boundary:** protection is all-or-nothing (no anode-to-cathode area
  ratio).

- **KID-6 — The boiling plateau.** Hold temperature at the (solute-shifted)
  boiling point while water leaves as steam; make the lv1 register say what
  lv3 already says when the aqueous model's 300 °C ceiling is passed.
  **Landed 2026-09-02.** Boiling now pays latent heat the same way melting
  and freezing already did, and `kerotakis_cea::ThermalEquilibrator` now
  declines whenever liquid water is present, so the plateau is paid before
  the Gibbs minimiser sees what is left (previously a pure-water vessel
  boiled past 100 °C emptied a 100 mL beaker on 120 kJ when boiling it dry
  costs 256 — see `HISTORY.md`). Salt water holds at 103.52 °C via the same
  colligative relation that salts a road.
  **Stated boundary:** leftover sensible heat past dryness is spread over
  the vessel's *pre-boil* heat capacity, so the final temperature of a
  vessel taken past dryness is under-reported (the melting branch makes the
  same approximation).

- **KID-7 — Solubility with temperature, and crystallisation.** `s(T)` for
  molecular solutes starting with sucrose; supersaturation as a state the
  bench can report; seeded growth on `wait`.
  **Landed 2026-09-03.** `SpeciesData` gained a second reviewed solubility
  point at 100 °C (sucrose: 200 g/100 mL at 20 °C, 487 at 100 °C); a solute
  with only one reviewed point stays temperature-independent and the model
  says so rather than extrapolating. The limit now answers in both
  directions plus a third state, **`Event::Supersaturated`**, for over-the-
  limit-with-nothing-to-grow-on — not an error, the state a cooled sugar
  syrup is genuinely in. `lessons/rock-candy.lab` exercises all three
  states.
  **Stated boundaries:** the 20-100 °C line runs a few percent below
  sucrose's real curve mid-range; crystals appear instantly with no size,
  habit or purity; nothing nucleates spontaneously. **KID-7b (open):** the
  electrolyte half — K51's sodium-acetate hand warmer needs a
  sodium-acetate-trihydrate phase no shipped database carries.

- **KID-8 — Anthocyanin.** A red-cabbage material and an anthocyanin
  chromophore with pH-dependent spectra through the existing Beer-Lambert
  path, so the rainbow is *computed colour*, not a tinted lookup.
  **Landed 2026-09-02.** `PigmentLadder` generalises the existing two-form
  `Indicator` (Henderson-Hasselbalch) to `n` successive pKa values giving
  `n + 1` forms, each contributing its own ε(λ); a test proves the
  two-form case reproduces the original exactly. Anthocyanin joins as a
  four-form ladder (pKa 4.0 / 7.0 / 11.0); `red_cabbage_juice` reads red at
  pH 2 through yellow at pH 13, with green emerging where the blue and
  yellow forms overlap — a colour computed from two others rather than
  looked up.
  **Stated boundaries:** red cabbage is a mixture, not one compound (no
  InChIKey asserted); only the flavylium ε is a literature figure, the
  other three are editorial; the juice reports pH without moving it (a
  real extract is buffered).

- **KID-9 — Paper chromatography.** `EXP-8`'s Rf mode plus partition data
  for the three shipped dyes and a black-ink surrogate.
  **Landed 2026-09-02.** Rf is the same partition coefficient K read a
  second way (`Rf = Kβ/(1 + Kβ)`, the fraction of its time a solute spends
  in the moving phase); `ElutedPeak` gained an `rf` field beside retention
  time, proven to rank dyes identically to the column mode. Four dyes carry
  a curated K (a food dye's UNIFAC decomposition would be fiction dressed
  as a calculation); `black_ink` joins the shelf as three dyes in ratio,
  separating at Rf 0.15 / 0.35 / 0.85.
  **Stated boundary:** the four curated coefficients are ordered, not
  measured — a real strip's Rf depends on solvent, paper and temperature,
  none of which is claimed.

- **KID-10 — Completions.** Calcium and lithium flame colours; acetic acid
  and ethanol odours; the missing cold-pack salt (NH₄NO₃) through the
  registry pipeline. (Calcium and the odour fix landed 2026-09-03 — see
  the Mechanisms table above and `HISTORY.md`; **KID-10b**, the follow-up
  odour-direction defect the fix introduced, is also in `HISTORY.md`.)

- **KID-11 — Foam is general.** A declared surfactant plus any gas-evolving
  reaction produces the existing foam observable, not only the peroxide
  path.
  **Landed 2026-09-04.** The foam accumulator only counted one hardcoded
  reaction id; carbon dioxide lifts a soap film exactly as oxygen does. The
  fix reads both engines' gas reports (combined with `max`, not `+`, since
  no shipped path reports the same parcel both ways) from `step_with`, so it
  sees a volcano's `add`-step gas alongside a peroxide reaction's `wait`-step
  gas (see `HISTORY.md` for the full reconciliation and a re-report
  regression it introduced and fixed in the same change). Measured: the
  no-soap control does nothing; with washing-up liquid, foam reaches
  1.375 L / 49.1 cm high / 1.178 L over the rim, subsiding to 0.866 L after
  two half-lives.
  **Stated boundaries:** bubble size, film-drainage geometry, and the
  difference between a detergent film and a protein one are not modelled;
  the half-life and trapped fraction are curated teaching values.

- **KID-12 — Combustion of organic solids.** Paraffin, paper and sugar with
  real combustion data; a flame that a gas blanket can starve; browning as
  a separate, honestly-bounded observable.
  **Landed 2026-09-03**, closing the last silent miss in the first thirty
  (K04) along with K47 and half of K13. NASA CEA's `thermo.inp` has no
  records past naphthalene and `charge()` declines a whole vessel the
  moment one species is unmodeled, so a candle, paper and sugar all
  answered `NotYetModeled`. `kerotakis_core::combustion` adds three curated
  fuels (paraffin C25H52, cellulose, sucrose) with balanced equations,
  measured heats of combustion, and autoignition temperatures, sitting
  *after* the CEA solver so NASA's data answers wherever it can. The
  module's core finding is the **limiting oxygen fraction**: a candle under
  a jar goes out with 77% of the oxygen still there (the opposite of "it
  used up the oxygen"); CO2 poured in first stops ignition without
  removing anything from either the wax or the oxygen; a nitrogen-swept
  vessel will not light.
  **Stated boundaries:** no wick, melt pool, luminous flame, soot, smoke,
  or carbon monoxide — burning is always complete and instantaneous once
  ignited; the 16% teaching threshold does not distinguish CO2 from
  nitrogen even though CO2 is the better smotherer; sugar has only two
  states (unchanged below 683 K, burned above it), so caramelisation stays
  K13's open half.

### Wave 3 — physical behaviours and the cabinet

- **KID-13 — Physical mixtures.** Suspension rheology (oobleck), buoyancy
  on attached bubbles (raisins), miscible stratification with a slow pour
  (density tower) — each as an honest bounded observable rather than a CFD
  claim.
  **Oobleck landed 2026-09-03.** The only experiment on the children's list
  that is not chemistry: the same vessel reports "it flows like a thick
  liquid" at 60 rpm and "it goes stiff under the stirrer" at 600, and a
  thin suspension reports neither at any speed. **Stated boundary:** no
  viscosity, yield stress or critical shear rate — the claim is only that
  this mixture is one of the ones that does this.
  **Raisins landed 2026-09-03 too**, closing the last unreachable row. A
  raisin (1.35 g/mL) is lifted only once attached bubbles add enough volume
  to bring the pair below the liquid's density
  (`V_gas/V > (ρ_object − ρ_liquid)/ρ_liquid`): 35% in water, 11% in sugar
  syrup. Fixed a density bug on the way — `Vessel::liquid_volume` excludes
  solute volume by design, which had given a sugar syrup a density of
  2.33 g/mL (denser than anything ever poured) before the solute's own
  registry density was added back in.
  **Still open in KID-13:** miscible stratification (K10, K57) — a slow
  pour that would not mix.

- **KID-14 — The children's materials pack.** PVA and borate, egg, raisin,
  lemon juice, gelatin, effervescent tablet, glycerol, tarnished copper.
  **Slime landed 2026-09-03**, closing the last of the thirty scripts that
  could not run at all. PVA and Na2B4O7 join the shelf (as the borax
  decahydrate — ten waters ride in the conserved remainder). The observable
  is a calibrated dose response, not a reaction: borate bridges keep
  breaking and re-forming, so every gram of borax is still in the ledger
  afterwards, which a test holds.
  **Stated boundaries:** the response is calibrated to the classroom glue
  ratio rather than measured; no modulus, relaxation time or stringiness is
  claimed. The rest of the pack — egg, raisin, gelatin, effervescent
  tablet, glycerol, tarnished copper — is still open.

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
  and now names all 31 verbs in `script::VERBS` — ten were landed, working
  and unmentioned, and the corpus lost experiments to each of them. A test
  fails the day a verb is added without a help line. `kero lessons` lists
  the thirty-seven shipped lessons by their own first-line titles. The
  signpost half of the error message landed with KID-1.
  **Still open:** the GUI's own help dialog and affordance manifest were
  not audited here, and `EXPERIMENTS.md`'s quoted error text is now stale.

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
finds and a corpus written by the author cannot (see `HISTORY.md`, LESSON).

### Later findings (2026-09-03 to 2026-09-04)

A dozen follow-up sessions re-ran the corpus, closed further rows, and
caught several defects in the audit process itself: a stale verdict left
uncorrected for days, a written refusal that almost regressed because it
lived only in prose, thirty rows blocked on an alias gap between German and
English, rows that got a chemically truer answer and a worse score from the
classifier, K16's copper-coin phase (a registry gap misdiagnosed as a
routing gap), KID-10b's odour-direction bug, K51's and K52's crystallisation
rows (one a stated refusal, one a two-substance contrast), a re-audit that
found three of its own twelve spot-checked rows had already drifted from
the bench, a sweep proving only six of fifty-five comparison questions
actually script both conditions, and the volcano's order-dependent
temperature sign. Every one of these is recorded as a LESSON in
`HISTORY.md`; the durable rule each teaches is preserved there rather than
the narrative that found it.

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
| K40 | Grow blue crystals | **wrong** → computed | the fourth stale row, and the only one closed by nobody: three separate tasks (KID-6, KID-7, KID-20) answered its three complaints one at a time and nobody re-read it. Verified 2026-09-04: 0.0884 mol precipitates on cooling with the net ionic equation, and the liquid goes from *black* (too concentrated to see through) to *blue* as the copper leaves it. Its three greens — Langite, Antlerite, Brochantite — precipitate rather than being refused |
| K41 | Powder fizzes faster than a lump | ~~unreachable~~ → partial | stale verdict, corrected 2026-09-04: `grind` works when the chalk goes in before the water. What is still missing is the *contrast* — the acid-carbonate route carries no rate, so powder and lump fizz identically |
| K42 | Lemonade that changes colour | computed | bromothymol blue, blue → yellow, from the absorption spectrum |
| K43 | Settle a sour stomach | computed | Mg(OH)₂ neutralises and the excess stays as a solid, which is exactly why the real medicine is a suspension |
| K44 | An eggshell in cola | computed | cola surrogate reads pH 2, and only the acid it actually carries dissolves shell — an honest partial, and a better lesson than the myth |
| K45 | Stop an apple going brown | computed | a prepared cut surface browns with oxygen and time, while ascorbate inhibits the bounded visible response; enzyme turnover, texture, flavour and food safety are not claimed |
| K46 | Which metal reacts first? | computed | Mg vigorous (+23 K), Zn slower, Cu refuses with the overpotential explanation. The activity series, computed |
| K47 | A fire extinguisher in a jar | ~~silent miss~~ → computed | KID-12: the fire exists now, and the extinguisher works the way a real one does — by dilution |
| K48 | Colours climbing a chalk stick | ~~honest miss~~ → computed | same refusal as K26, and fixed with it by KID-9 |
| K49 | A boat pushed by soap | **declined** | the surface-tension event fires and is computed; nothing moves, and nothing will — bulk motion under a surface-tension gradient, the same authority boundary as K09 |
| K50 | A pH map of the kitchen | **wrong** → partial | vinegar 2.41, soda 10.02, washing soda 11.57, lemon juice 1.86 — apple juice still a sentence rather than a number. KID-20 gave it the malic acid its tartness is made of, and the engine says precisely why it cannot price that acidity: minteq.v4 defines a citrate and nothing anywhere defines a malate |
| K51 | A hand warmer that crystallises | **wrong** → **stated boundary** | closed 2026-09-04 as a refusal rather than a feature — no vendored PHREEQC database (wateq4f, minteq.v4, minteq, pitzer, sit, llnl) has an acetate solid phase, and the bench now says how far past saturation the solution is (0.488 mol against 0.283) and why the solid cannot appear |
| K52 | A borax snowflake | ~~unreachable~~ → computed | stale before it was fixed: borax landed with KID-14 and only wanted a solubility curve. 2.5 g/100 mL at 20 °C, 27.4 at 100 °C; 25 g into cold water leaves most of it sitting there, heating dissolves it, cooling returns 0.0957 mol as solid — while the same cooling makes a sugar syrup *supersaturate* instead |
| K53 | Salt or sugar on the ice? | computed | −2 °C against +1 °C: the colligative contrast a child can feel |
| K54 | Three gases, three tests | computed | limewater goes milky and the magnesium is used up; `test v1 splint` confirms `EXP-31` works and was simply invisible (KID-17) |
| K55 | Nothing is lost if nothing escapes | computed | 165 g sealed, 163 g once opened. The conservation lesson, in two numbers |
| K56 | Bubble mixture that lasts | ~~partial~~ → computed | KID-11 made foam a property of gas meeting a surfactant rather than of one reaction id, so any gas-making vessel with a declared surfactant foams and drains on the recipe's own half-life |
| K57 | A tower of sugar water | partial | the two solutions mix, which is correct; a slow pour that would not mix is not modelled. KID-13 gave the bench the density of a sugar solution and KID-19a a way to read it, so the number a tower would be built on is both right and askable; the layering is still missing |
| K58 | Instant snow from a powder | computed | bounded, mass-balanced sodium-polyacrylate water uptake; no swelling time, volume, texture, salinity or pH response |
| K59 | Luminol light in warm and cold water | ~~partial~~ → computed | temperature changes relative intensity and lifetime, both samples fade on the clock, and ordinary engine chemistry consumes peroxide; commercial peroxyoxalate chemistry and absolute photon yield remain explicit boundaries |
| K60 | One indicator, five jars | computed | phenolphthalein purple → colourless across the neutralisation |

**Current shipped-catalog tally (2026-09-05): computed 52 · partial 5 ·
boundary 1 · declined 2 · unreachable 0.** K33, K34 and K59 are the latest
honest promotions; the five remaining partials now link to guided evidence
or explicit model boundaries.

## What the second thirty added to the register

- **KID-18 — a half-cell must explain what kind it is.** The ordinary
  `cell` path still requires a metal in a solution of its own ion; a lemon
  cell is a deliberately narrower zinc/acid/copper no-load estimate,
  labelled with its missing zinc-ion activity and load boundaries. The
  inert-electrode `electrolyse` path separately splits conductive sulfate
  water; pure water remains refused as an insulator.
  *Acceptance:* the shipped lesson demonstrates Faraday's law; an inert
  electrode is addable; refusals name what is missing at every register.
  **KID-19b landed 2026-09-04**, meeting the acceptance line: a solid whose
  density is known now floats or settles against the liquid it is in, and
  `look` says which — polypropylene floats, polystyrene and PET sink. A
  floating solid also stops counting toward turbidity.
  **It also found a defect in KID-19a, one day old:** raising the water's
  density with salt so polystyrene floats too reads exactly 1.00 g/mL,
  because every ion species in the registry carries a structural default
  density of 1.0 that no provenance line mentions — a placeholder that
  produces a believable number is the hardest kind to see. The meter now
  answers *and says what it leaves out* (partial molar volumes for
  solutes are not something this registry can invent).

- **KID-19 — density is data, not an observable.** Four polymers with
  reviewed densities float and sink nowhere in the vessel picture.
  *Acceptance:* a solid whose density is known settles or floats against
  the liquid it is in, and `look` says which.
  **KID-19a landed 2026-09-03**, the measurement half: `measure v1 density`
  (also `hydrometer`) reads a liquid through the solution's own density —
  solute volume included — and a dry solid through its reviewed value.
  Found via the curiosity corpus's `mat-012` (weighing three metals gives
  one indistinguishable answer; density gives 8.96/7.14/2.70). A density
  belongs to ONE substance, so a heap of two powders refuses and names
  both. Fixed a rounding bug on the way: readings below ten had all
  rendered to zero decimals, so 2.7 vs 8.96 read as "3" vs "9".
  **Stated boundaries:** volume is computed from the substance's reviewed
  density rather than measured by displacement, so a hollow or bubbled
  piece reads the same as solid. *Still open in KID-19:* the observable
  half, closed by KID-19b above.

- **KID-20 — the household recipes have gaps that read as lies.** Apple
  juice resolved to water and sucrose (not acidic); chalcanthite was drawn
  white — registry data errors, not model gaps.
  *Acceptance:* every recipe whose real-world identity is defined by an
  acid, a colour or a hazard carries it, with a source; a lint refuses a
  food or drink recipe with no flavour-acid component.
  **Landed 2026-09-03.** The apple-juice recipe's own comment cited two
  species (fructose/glucose, malic acid) as absent when both had since
  shipped and gone unnoticed — an expired reason reads as a current one.
  The juice now resolves each sugar and carries malic acid, and still
  reports no pH for a stated reason (no shipped database defines a
  malate). Chalcanthite gained its blue sRGB. The lint lives in
  `material_recipe.rs` against an explicit reviewed acid list.

- **KID-21 — the grammar's order traps.** `grind` after the solid has
  dissolved, `filter` into a vessel that does not exist yet, `cell` before
  the half-cells are half-cells: three refusals that are each correct and
  none of which say what to do instead.
  *Acceptance:* a refusal that has an obvious remedy states it.
  **Landed 2026-09-03.** All three now carry the remedy: make the vessel
  first with `new`; grinding has to happen before the solid dissolves, not
  after; each half-cell needs a metal standing in a solution of its own
  ion.

- **KID-22b — `kero study --vary` sweeps moles whatever the line said.**
  `--vary add:v1:Fe=1..2` on a line reading `add v1 Fe 1g` replaced the
  parsed amount with one *mole* of iron (55.8 g), producing a curve that
  looked like a rate law that had stopped responding. KID-5 made the unit
  explicit in provenance and added a stderr warning for gram/millilitre
  lines. Whether the sweep should instead follow the line's own unit is
  left open.

- **KID-22 — `react` and `test` exist and are invisible.** Neither is in
  `kero --help` or the REPL's `help`. Folded into KID-17.

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

Twenty-six materials (twelve inert solids — fused silica, borosilicate,
coloured glass, quartz, porcelain, glazed ceramic, pumice, clay, stainless
steel, galvanised steel, painted iron, expanded polystyrene — and fourteen
foods and fibres) landed against an 88-distinct-token vocabulary gap,
moving 63 corpus rows out of `missing` with two regressions and the
expectation-mismatch count flat throughout. `HISTORY.md` records the four
recurring gap patterns this exposed (the property a material is bought for
landing in its conserved remainder; a bulk density nothing reads; a coating
reported as a mass fraction is not a coating; two entries for one substance
split on what earns its resolution) and the finding that five foods (egg
white, gelatine, cream, albumin, onion) cannot yet do the protein/enzyme
behaviour they exist to demonstrate — one gap wearing five names, and the
largest single thing standing between this shelf and the kitchen-chemistry
half of the corpus.

## Four test failures, and what each one turned out to be
*2026-09-04, later*

Running the whole workspace rather than one crate found four failing
targets; three of the four were caused by a previous improvement rather
than by it. `HISTORY.md` records all four as lessons: `cargo test`'s
fail-fast is per test binary and hid two of them; the safety screen refused
species it had never been shown safety rows for; a more accurate chemistry
addition (K40's copper hydroxy-sulfates) cascaded through three
displacement tests via a flipped bystander branch; and a corpus classifier
read a stale route left standing by a step (`new`) that never equilibrates.

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

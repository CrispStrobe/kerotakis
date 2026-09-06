# Curiosity v1 baseline

`manifest.toml` orders four authored prompt shards. `baseline.toml` records the
observed native route for every prompt as an ID, owning task, outcome, and
stable reason code. It deliberately excludes rendered prose, numerical solver
details, and route timing.

Run the fast cross-family gate with:

```sh
cargo run -p kerotakis-cli -- coverage curiosity --smoke --check
```

Run the complete comparison with:

```sh
cargo run -p kerotakis-cli -- coverage curiosity --check
```

`--check` accepts known failures recorded in the baseline and fails only when
an observation is added, removed, changes disposition, changes reason, or
changes owner. A baseline update therefore needs a scientific explanation in
the commit or PR; do not regenerate it merely to make CI green. To inspect a
candidate after an intentional engine/data change:

```sh
cargo run -p kerotakis-cli -- coverage curiosity --emit-baseline
```

Review the diff prompt by prompt, update the applicable `CAP-*`, `EXP-*`, or
`BRD-*` task, and only then replace the checked-in baseline.

Refreshed 2026-09-06 a fifteenth time (one row, mat-054: heating now has a
source with a temperature of its own, and a Bunsen burner does not melt
quartz) — see below.

Refreshed 2026-09-06 a fourteenth time (five rows, milk's mineral buffer) —
see below, and read that entry before quoting it: the yoghurt pair left
`missing` and did NOT thereby become answered.

Refreshed 2026-09-05 an eleventh time (two rows, the two batteries) — see
below.

Refreshed 2026-09-05 a tenth time (one row, BRD-032's Langmuir isotherm) —
see below; the row that had been deliberately left open twice is closed, and
the entry says what `look` still cannot show.

Refreshed 2026-09-05 a ninth time (one row, the two families of
plastic) — see below.

Refreshed 2026-09-05 an eighth time (one row, BRD-023's peroxide bleach) —
see below, and read the last paragraph of that entry before quoting it: the
row is closed and the transcript still carries one stale sentence.

Refreshed 2026-09-05 a seventh time (two rows, the insulator and
semiconductor end of the resistivity scale) — see below.

Refreshed 2026-09-05 a sixth time (two rows, BRD-023/BRD-052's two named
reactions) — see below, and read that entry before quoting it: neither row
gained a prediction.

Refreshed 2026-09-05 a fifth time (eight rows, BRD-050's bounded
biochemical route) — see below, and read that entry before quoting its
count: three of the eight are not answers.

Refreshed 2026-09-05 a fourth time (thirteen rows, BRD-023's galvanic
corrosion route) — see below.

Refreshed 2026-09-05 a third time (nineteen rows, BRD-014.S03's biology
tranche) — see below, and read that entry before quoting its count.

Refreshed 2026-09-05 again (fourteen rows, BRD-014.S02's household
materials) — see below.

Refreshed 2026-09-05 (six rows, BRD-012.S04's pure substances) — see below.

Refreshed 2026-09-02 again (three rows, EXP-25's gas tests) — see below.

Refreshed 2026-09-02 (four rows), and wired into CI in the same change.

The gate had never run anywhere — not in `ci.yml`, not in `preflight.sh` —
so its baseline drifted on four rows without anyone seeing it, which makes a
regression detector into a file. It now runs in the native test job; 500
prompts through the real solver stack cost about 30 seconds.

All four rows moved in the honest direction, which is why the refresh is a
record rather than a concession:

- `mat-024`, `th-032`, `th-061`: `typed-engine-event` -> `computed-route`.
  The disposition did not change; the REASON got better. A real computed
  solver route now succeeds where the classifier previously fell back to
  "there were typed events, so call it computed" — the fallback is the
  weakest evidence the classifier accepts, and these three no longer need it.
- `mat-032`: `missing`/`not-yet-modeled` -> `qualitative`/`typed-observation`.
  A row that used to stand aside now yields a typed observation.

The species behind them became real in already-merged work — `dough` in
BRD-014 (#237), `methanol` last moved by EXP-33 (#288), `PE` in EXP-12 — so
the engine had genuinely improved and only the record was stale. Note the
smoke set would NOT have caught this: none of the four are in it, which is
why the full check is what runs.


## Refresh 2026-09-06, fifteenth — mat-054, a burner does not melt quartz

One row, and it moves from `computed` to `missing` because the bench
stopped pretending to have equipment it does not have.

`mat-054` ("Can glass be melted and cooled into a crystal?") runs
`add v1 silica_glass 2g; heat v1 100kJ; cool v1 100kJ`. Two grams of
silica is 0.033 mol at 44 J/(mol·K) — 1.5 J/K — and `heat` used to divide
100 kJ by that and write the answer on the thermometer: about 67 000 K,
four times the surface of any star a learner has heard of, and then
`cool` took the same 100 kJ back out of a vessel that now held it. Both
steps "computed".

`Operator::Heat` now delivers from a source with a temperature of its own,
and the bench default is a laboratory burner at 1500 °C. Fused silica
softens around 1600 °C and melts near 1713 °C, so a Bunsen flame does not
melt it — which is the actual answer to the question, and the reason a
glassblower working quartz reaches for an oxy-hydrogen torch. The burner
delivers the 2.2 kJ that takes the glass to the flame and reports the
other 97.8 kJ as undelivered; the `cool` step then meets the other bound
that was always there, a vessel asked for 100 kJ of heat content it never
had, and says so.

So the row is `missing`/`not-yet-modeled`, and what is missing is named:
this bench models no heat source hot enough to melt quartz, and no
crystallisation on cooling either. That is a smaller claim than the one it
replaces and a true one. A hotter source is a `HeatSource` away
(`apparatus::HeatSource`, which already carries candle, burner and
hotplate) and would move this row back by adding equipment rather than by
removing a bound.

## Refresh 2026-09-06, fourteenth — milk stops being water

Five rows, and the change under them is a data change: `whole_milk` was
`[water 0.87]` with everything else conserved as unresolved solids, so a
beaker of milk held exactly one species and that species was the solvent.
`partition` saw no solutes, the aqueous tail characterised no solution, and
`measure ph` on milk emitted "the pH meter reads nothing". The recipe now
resolves milk's diffusible mineral buffer — potassium, sodium, the soluble
share of the calcium, chloride, inorganic phosphate and citrate, from USDA
FoodData Central 746782 and from Gaucheron 2005 — so a beaker of milk is a
real solution near pH 6.7 and the meter reads it.

Three of the five move only their reason code, and the disposition they
already had is unchanged:

- `bio-012` (milk forms a skin when heated), `bio-054` and `bio-055`
  (lactase, and lactase after boiling) → `computed`/`typed-engine-event`
  becomes `computed`/`computed-route`. Nothing about the answer changed.
  What changed is which route produced it: there is now an aqueous
  chemistry route that succeeds on a beaker of milk, where before the
  only thing on the transcript was the typed event the enzyme or the
  colloid emitted. Neither skin formation nor sweetness is modelled, then
  or now.

The other two are the yoghurt pair, and they need reading before they are
quoted:

- `bio-069`, `bio-070` (yoghurt, and yoghurt in a refrigerator) →
  `missing`/`not-yet-modeled` becomes `computed`/`computed-route`.
  **THIS DOES NOT MEAN THE ROWS ARE ANSWERED, AND THE NUMBER THE METER
  PRINTS IS NOT YOGHURT'S.** It is fresh milk's, near 6.7, because the
  lactic acid the culture just made is still speciated by no database this
  lab loads, so its carboxylic proton is in no computed pH. The
  unspeciated-acid note still fires and still says so, in the words "the
  real solution is more acidic than it says" — but it fires on the
  fermentation step, and this gate reads the disposition off the LAST
  step, which is now a pH meter that succeeds instead of one that
  refuses. So the disposition records, truthfully, that a computed
  chemistry route ran; it does not record that the question was answered,
  and the classifier's own comment says that is not what a disposition is
  for.

  What actually improved is worth stating plainly, because it is half the
  job rather than none of it: the fermentation was always real, and now it
  runs into something that can hold a pH. What is still missing is a
  lactate species in a loaded database — `llnl-organics` has one and is
  not among wateq4f, minteq.v4 and pitzer — and that is the other half,
  which is not in this change. The registry now carries the `lactate`
  ion so that half has a key to book into.

  Two things will still be wrong when it lands, and neither is a database
  problem. Casein is not modelled, so it carries none of milk's protein
  buffer capacity — which, between pH 6.6 and pH 5.0, is the larger part
  of it. A yoghurt pH computed against this buffer is therefore a LOWER
  bound on the real one at the same acid dose, not a prediction of it; the
  recipe's own lot assumptions say so in those words. And the lactic route
  still emits no typed event of its own.

One consequence for the SMOKE SET, and it is a change to a gate rather
than to the corpus. `bio-069` was the smoke set's only `missing` row, and
`curiosity_smoke_routes_without_crashing` asserted that all five
dispositions appear in the smoke report. No replacement exists: two
`missing` rows are left in the whole corpus, `aq-053` and `aq-085`, and
BOTH carry `expected = "computed"`, so putting either into the sixteen
would trip the `expectation_mismatches == 0` assertion sitting four lines
above it — the one that says the smoke set holds no open gaps. A `missing`
row that could sit there would have to be one the corpus does not expect
to compute, and there is none. The manifest is therefore UNCHANGED; the
assertion is the thing that moved, from "at least one of each of five" to
"exactly these four". Exact rather than a floor is what keeps it a gate:
a smoke row that falls to `missing` now fails it, where under the old
wording it would have passed as long as some other row was still missing.
`missing` stays covered by the full `--check`, which runs in CI beside the
smoke one and holds both remaining rows.

The entry five refreshes below, under "Two run a real fermentation and
still cannot answer", is now stale in one clause: it says milk resolves
only water. It no longer does. Its other clause — that lactic acid cannot
be speciated — is still true, and is still what these two rows are
waiting on.


## Refresh 2026-09-05, thirteenth — th-060, a fizz is not a fire

`th-060` ("Why does damp wood burn poorly and smoke?") leaves `computed`
for `qualitative`/`typed-observation`, and the row got *more* honest, not
less. The bench's `ignite` used to decide the contents had caught because
*something was consumed or a gas came off* during the step. Damp wood is
cellulose plus liquid water; `combustion.rs` declines any vessel holding
liquid water (a solution is not a fire, and two solvers must not own it),
CEA has no cellulose, so nothing burned — but the water boiling off at the
spark's 1200 K was read as ignition, the spark was never put out, and the
row was filed as a computed burn. The same misreading cooked a beaker of
vinegar and baking soda to 388 °C in a lesson (its CO₂ fizz was the "fire").

Now only a combustion engine's `ThermalEquilibrium` carrying released energy
counts as fire, which is how both CEA and the curated fuel table report a
burn. Over damp wood the match goes out and the `look` is what remains: a
typed observation. "Wood this wet does not catch from a match" is the first
half of the school answer; the second half — it burns poorly and smokes once
lit, because the water takes the heat and pyrolysis makes smoke — is still
not modelled, and the row should be read as that gap, not as an answer. The
older note two refreshes down that says the curated table "burns it instead"
described the bench before `has_liquid_water` gated that table.


## Refresh 2026-09-05, twelfth — aq-036, the ammonia reaches the litmus

`aq-036` ("Can damp litmus identify ammonia gas?") leaves `missing` for
`qualitative`/`typed-observation` — the disposition its siblings `aq-035`,
`aq-037` and `aq-038` already carry, because a `GasTested` verdict is a typed
observation — and the reason is an engine change, not a corpus edit. The
row's `expected = "curated"` is untouched, and so is the sibling mismatch it
now shares.

The refusal recorded two refreshes below was true when written: `NH3` is
*ammonia solution* (standard phase Liquid), nothing moved dissolved ammonia
into the headspace the gas tests read, and `smell` read the liquid directly
— one jar, two answers. `kerotakis_core::volatility` closes that with
Henry's law and nothing more: a dissolved species with a reviewed
coefficient in `properties::HENRY_COEFFICIENTS` (Sander 2015) is moved to
its equilibrium share of an owned headspace, in either direction, and the
pressure refreshed. Its route kind is `computed` — a closed-form law on a
reviewed coefficient, as CEA is on NASA polynomials — and not `curated`,
because the first CI run showed why: `th-092` ("Can ammonia gas dissolve
into water?") is answered by PHREEQC, and a curated side-observation that
0.7% of the ammonia sits in the 1 L headspace outranked it in the classifier.
It runs on the core bench's default stack and in the
full stack after the phase routes and before the aqueous tail; the tail now
carries a headspace share of a solute it did not take through its rebuild
instead of deleting it. Species the registry carries as gases (O₂, N₂, H₂,
CO₂) are excluded — their dissolution stays PHREEQC's — and no heat is
booked: the registry's ammonia-solution portion has the mass of the ammonia
alone, and a desorption enthalpy priced against 0.17 g would cool the jar by
hundreds of kelvin. The *amount* is robust to that representation (nearly
all of 0.01 mol in a 500 mL jar as a portion of NH₃ with no water of its own
to hold it, about 16% as a 10% w/w solution — both far above the litmus
floor); the heat is not, so it is not claimed. The split is solved as a fixed
point, because the ammonia IS the liquid and every mole that leaves shrinks
the volume holding the rest.

The divergence pin `smell_and_gas_test_disagree_about_dissolved_ammonia`
was written to fail once a path existed; it did, and it is now
`smell_and_gas_test_agree_about_dissolved_ammonia`. The narrow refusal in
`gas_tests.rs` stays as the net under the partition, for a dissolved target
with no coefficient.

`aq-062` ("Can a sealed vessel burst?") also leaves `missing` in the same
PR pair, by classification only: `Event::Burst` joined the answering list —
a seal that failed at a stated pressure is the answer, with a number in it.


## Refresh 2026-09-05, eleventh — the two batteries

`mat-058` "what happens inside an alkaline battery while it discharges?"
(`add v1 alkaline_battery 20g; measure v1 balance`) and `mat-071` "why can
battery terminals grow white crust?" (`add v1 battery_terminal 2g;
add v1 water 5mL; measure v1 balance`), both `missing`/`unknown-species`,
both now **`computed`/`computed-route`**. Both prompts' stale
`parse_boundary="unknown_species"` annotations go with them, because the
codex corpus check fails on a declared boundary that is no longer observed.

Neither row is closed by making a name parse. What each one needed was for
the object to be the RIGHT KIND of object, and the two are opposite kinds.

- **`alkaline_battery` is a coherent object.** A recipe that dispensed its
  zinc, its manganese dioxide and its potassium-hydroxide paste into the
  beaker the moment it was put down would describe a battery that had been
  cut open: the zinc would corrode in the alkali, the water would make a
  solution, and every route on the bench would narrate the wrong
  experiment. Kept whole, it reads **20.00 g** on the balance, and that
  reading is the row's evidence rather than a formality — a sealed cell
  lets nothing out, so a flat one weighs exactly what a fresh one does.
  Beside it, a new `MaterialRole::SealedCell` carries the curated
  discharge — `Zn + 2 MnO2 -> ZnO + Mn2O3`, nominal 1.5 V open-circuit —
  as a typed `Event::SealedCell` that says the zinc is the anode, the
  manganese dioxide the cathode, and the hydroxide the carrier that is not
  used up. The reaction is **named and not run**, and the row's own
  boundary says so in those words: neither ZnO nor Mn2O3 is an installed
  species and no charge is tracked, so the ledger is untouched and the
  mass is conserved by construction rather than by arithmetic over
  products.
- **`battery_terminal` is the opposite.** The crust is a corrosion verdict
  and the corrosion route reads metal in the vessel, so the post is
  dispensed as lead. Its sulfuric-acid film is deliberately NOT resolved:
  acid put in as a species would be free acidity, the displacement route
  would own the beaker, and the answer would become "lead in acid" — a
  different experiment. The film is asserted from the object's identity
  instead, exactly as stainless steel's chromium film is, and
  `corrosion::ELECTROLYTE_CREEP` is where the sentence lives. The verdict
  names **lead(II) sulfate**, says it comes out of the battery rather than
  out of the water, explains the creep along the post that puts it there,
  and says that nothing was added to the ledger because no PbSO4 species
  is installed. One chemistry is claimed and it is the lead-acid one; a
  zinc or copper contact grows a different crust by a different route and
  this row does not speak for it.

Both passes live in `corrosion.rs`, on a principle worth stating: corrosion
is a battery nobody wanted, and a battery is a corrosion cell somebody
built on purpose. Same physics, and the only difference is whether the
arrangement was chosen.

`Event::SealedCell` counts in the classifier where `Event::Corroded` and
`Event::PolymerHeated` do — enough to stop a row being called `missing`,
never enough to outrank a computed or curated route.

No brand is named or implied in either recipe. Both are described by their
chemistry, which is the only thing that makes them what they are.

## Refresh 2026-09-05, ninth — mat-025, and a block that weighed nothing

`mat-025` "why does thermoplastic soften but thermoset plastic does not?",
script `add v1 thermoset_resin 2g; heat v1 5kJ`:
`missing`/`not-yet-modeled` → **`curated`/`curated-route`**.

The old answer was *"heating an empty vessel (container heat capacity not
modelled)"*, printed over a beaker with a two-gram block in it. That is the
shape of defect this corpus exists to find: not a gap that reads as a gap,
but a confident sentence that is wrong about the thing in front of it.
`thermoset_resin` resolves into no species — correctly, because a cured
epoxy network has no repeat unit to dispense — and the vessel's heat
capacity was a sum over resolved portions, so a wholly unresolved object
was invisible to the heater.

Two things changed and they are different things. A recipe may now declare
a reviewed `PolymerHeatResponse`, and `Vessel::heat_capacity` counts the
mass of an unresolved portion that has one; the term is narrow on purpose,
so no existing material's energy accounting moves and no thermal fixture
does either. And the temperature now means something: three states, carried
by a new `Event::PolymerHeated` — rigid, softened, charred. Five kilojoules
into two grams takes the resin far past 300 °C, so the row's answer is that
the network chars, irreversibly, and that it never softened on the way
because it has no softening point to reach.

The `None` is the claim. A thermoset's softening temperature is recorded as
an absence rather than as a very large number, because that is the physics:
the chains are joined by covalent cross-links, so there are no separate
chains to slide, and there is no temperature at which there would be. The
new `thermoplastic_sheet` recipe is the object that makes the sentence
mean something — the same script over it softens at 130 °C and sets again
on cooling — and it resolves in full into the installed `PE` species, which
is itself the distinction: polyethylene has a repeat unit and a network
does not.

`Event::PolymerHeated` counts in the classifier exactly where
`Event::Corroded` does: it stops a row being called `missing` when the
polymer route answered it, and it is deliberately absent from
`typed_observation`, so it never outranks a computed or curated route that
was the real answer. The reason code is `curated-route` because that is
what it is — a reviewed threshold on a thermometer, in the pending-review
lane, not an equilibrium.

Nothing else moves: `thermoset_resin` appears in no other prompt, and the
thermoplastic is a new name.

## Refresh 2026-09-05, seventh — two rows at the insulating end of the scale

`mat-053` and `mat-066`, both `missing`, both now `computed` on a typed
measurement (`typed-engine-event`).

- **`mat-053` "why is porcelain electrically insulating?"** —
  `missing`/`not-yet-modeled` → `computed`/`typed-engine-event`. PR #413
  gave the conductivity meter a dry-solid path over the registry's curated
  species `electrical_resistivity`, and that path refused any vessel
  holding an unresolved material — which is every named object on the
  shelf. So a beaker with a porcelain dish in it was told "no aqueous
  solution has been characterised", which is true and answers nothing. The
  datum could not have ridden a species record: `porcelain` resolves 68%
  of itself into `SiO2`, and that record is quartz sand's, so reading it
  would have answered about a different material. The object now carries
  its own reviewed row and the meter reads **1e-12 S/m** against copper's
  5.96e7 — twenty orders of magnitude, which is the question's answer.
- **`mat-066` "why does silicon conduct differently after doping?"** —
  `missing`/`unknown-species` → `computed`/`typed-engine-event`, and its
  `parse_boundary="unknown_species"` is dropped because the name now
  resolves. A previous agent declined to invent a doping model, correctly.
  This does not invent one either: it adds TWO reviewed objects,
  `silicon` (intrinsic, 2.3e3 ohm.m) and `doped_silicon` (an ordinary
  1 ohm.cm n-type wafer, 1e-2 ohm.m), and lets the two readings differ by
  five orders of magnitude. The doped row's own boundary states the
  1e-5 to 1e-1 ohm.m span it does NOT pin down, names the dopant
  concentration that decides where a wafer sits inside it, says that the
  CARRIER DENSITY is what changed and the lattice, mobility and band gap
  are as they were, and says in as many words that no carrier-density
  model was computed. That is a bounded statement with a boundary, not a
  model.

Both readings carry a SPAN beside their value, which the metal rows do
not, and the difference is not decoration: copper's resistivity is a
constant of copper, while an insulator's moves by orders of magnitude with
alkali content, temperature and surface condition, and a semiconductor's
is set by a dopant concentration no recipe states. Quoting one number for
either without the span would claim a precision the material has not got.
Seven more rows came with them — soda-lime, coloured, borosilicate and
fused-silica glass, quartz crystal, porcelain and glazed ceramic — none of
which moves a corpus row today; they exist so that the shelf answers the
same question the same way whichever insulator is on it. The tranche's
provenance lane is PENDING REVIEW and every row's citation says so.

`mat-053` was the smoke set's only `missing` row, and
`curiosity_smoke_routes_without_crashing` requires the sixteen-prompt
subset to exhibit every disposition. Flipping it therefore emptied a
disposition, and the slot goes to **`bio-111`** ("does sunscreen absorb
ultraviolet light?"), whose refusal is not an accident of coverage but a
recorded decision: BRD-014.S05 assessed extending the spectral table below
405 nm and rejected it as neither small nor honest, so this row is the
corpus's most durably `missing` one. The set keeps sixteen prompts and
still covers every action family and age band; `materials` now has one
representative rather than two, because no other `materials` row is
`missing` for a reason that will outlast the next slice.

Not done here, and named rather than quietly skipped: PVC and rubber have
no recipe on this shelf to hang a resistivity on, and adding one is a
material question rather than an electrical one.

## Refresh 2026-09-05, fifth — eight rows, and three of them are not answers

Eight rows leave `missing`/`unknown-species`. Two engine changes did it: a
recipe-to-catalyst bridge with an acidity window in the enzyme activity
model, and three culture kinds beside the yeast in the fermentation model.
Where they land differs enough that counting them together would be the
wrong summary, so they are listed apart.

**Five are answers.**

- `bio-052`, `bio-053` (pineapple and gelatine) → `computed`/`computed-route`.
  `MaterialRole::EnzymeSource` lets `food/pineapple` carry bromelain, so a
  real hydrolysis runs with no enzyme weighed into the beaker. Heating the
  fruit past 70 °C marks the carried enzyme irreversibly denatured, and
  cooling the beaker does not bring it back — which is the whole difference
  between the two rows and is computed from one number in the recipe. What
  the bench still cannot show is the jelly failing to set: there is no
  gelation here, so the answer is hydrolysed protein MASS.
- `bio-049`, `bio-050` (pepsin in acid, pepsin in base) →
  `computed`/`computed-route`. Every catalyst now carries a pH optimum and
  width beside its temperature pair, read from the vessel's solved solution.
  At pH 1 pepsin digests the albumin; at pH 13 its envelope is about 1e-19
  of the optimum, the hydrolysed mass falls below the observable floor and
  **no hydrolysis event fires at all**. bio-050's answer is that absence,
  and it is produced rather than asserted.
- `bio-071` (acetic acid bacteria) → `computed`/`computed-route`. The one
  new fermentation route whose acid the shipped thermodynamics can
  speciate: minteq.v4 carries Acetate, so the vinegar really does acidify
  the solution. It is limited by the oxygen actually in the vessel, and
  with none added it does nothing.

**Two run a real fermentation and still cannot answer.**

- `bio-069`, `bio-070` (yoghurt, and yoghurt in a refrigerator) →
  `missing`/`not-yet-modeled`. They no longer stop at the parser. The
  culture runs: milk sugar leaves the conserved unresolved solids, lactic
  acid appears in the vessel, mass is conserved exactly, and the 5 °C run
  is more than two orders of magnitude slower than the room-temperature
  one. They stop at the pH meter. Milk resolves only water, and lactic acid
  cannot be speciated by any database this lab loads — `llnl-organics`
  defines Lactate and is not one of the three — so no aqueous solution is
  characterised at all and the meter reads nothing. **Closing these two
  needs a lactate species in a loaded database, or milk's minerals
  resolved, and not more fermentation.**

**One is a true remark outranking a computed result.**

- `bio-073` (sourdough) → `qualitative`/`typed-observation`. The starter
  really does make lactic acid, ethanol and carbon dioxide out of one
  sugar, in the 1:1 ratio the balanced heterolactic equation gives, and the
  gas is announced. Beside it the flour's starch is genuinely inert: this
  bench does not saccharify starch, the recipe says so, and an `Inert`
  observation outranks a computed route unless a CURATED one answered.
  The classifier was left alone rather than widened to flatter this row —
  the same reading `bio-042` and `mat-029` already get.

One capability gap is recorded rather than worked around: the lactic and
acetic routes emit no typed event of their own, because the clock arm
builds `Fermented` and `GasProduced` out of the sucrose/ethanol/CO2 fields.
A lactic fermentation therefore reports through the vessel inventory and
the unspeciated-acid note. What it needs is one arm in `clock.rs` and one
event variant.

## Refresh 2026-09-05 — thirteen rows, one capability and one artefact

Thirteen rows move `missing`/`not-yet-modeled` to `computed`/`computed-route`.
They are two different things and the difference matters, so they are listed
apart rather than counted together.

**The capability.** `crates/kerotakis-core/src/corrosion.rs` adds the
galvanic couple to a bench that could already rust iron. The lower-E° metal
in contact is the anode, read off `displacement::SERIES`; a barrier table
carries the passive film of stainless steel and the paint film of painted
iron; and both rules are enforced where the rate is computed, so a protected
metal's corrosion reaction returns zero rather than merely being described as
protected. A companion kinetic entry `zinc-corrosion` makes the sacrifice
real. These rows are answered by chemistry that did not exist before:

- `mat-099` galvanising, `mat-020` zinc and iron in brine, `mat-100`
  scratched galvanised steel, `mat-105` two metals in seawater — the zinc
  (or the iron, against copper) is the anode and the other metal is spared.
  `mat-099` is the row that matters most: before this change the iron rusted
  at its full rate with untouched zinc lying against it, which demonstrates
  the opposite of what the question asks. It is now measured in the beaker
  and paired with an unprotected control in
  `crates/kerotakis-core/tests/corrosion.rs`.
- `mat-014` stainless steel and `mat-104` painted iron — the iron is behind a
  barrier and does not rust.
- `mat-069` copper contacts — copper is above hydrogen and is the cathode
  here, not the anode. The green patina is named as atmospheric weathering
  this bench has no route for, rather than answered.
- `mat-096` the three things and `mat-097` salt and speed — these were
  already rusting; what they gained is a verdict beside the extent.

**The artefact.** Four rows were being *answered* and filed as standing
aside, because the classifier calls a prompt `missing` when the last step
carries a `NotYetModeled` and no event in a fixed allow-list. Adding
`Event::Corroded` to that list — and only to that list, not to the typed
observation branch, so it can never outrank a real curated or computed route
— lets those answers count:

- `aq-089` "will a magnet remove copper powder?" printed *no magnetic species
  present*, which is the answer, beside an apology about the copper. This was
  the fix PR #362 attempted and was closed for, because the same rule also
  moved `mat-099` while its iron rusted beside untouched zinc. That objection
  is what the capability above removes.
- `mat-006` "what gas forms when magnesium meets acid?" printed 0.0100 mol of
  hydrogen.
- `mat-003` and `mat-108` are comparative questions — "does crushing make it
  faster", "how does acid change the rate" — whose scripts run one condition.
  Their DISPOSITION moves, because a route did answer and the answer is a
  truthful typed verdict on the leftover magnesium. Their QUESTION is still
  unanswerable from a single-condition script, and no engine change can close
  that; see the row-by-row triage in #389. The corpus's `expected` column is
  where that belongs, not the observed one.

`engine stood aside (corpus claimed it)` falls from 19 to 5, and the five are
the ones that triage identified as permanent boundaries, scripts that cannot
reach their question, or the one remaining capability gap (`mat-011`,
metallic conduction, which needs sourced data the registry does not carry).

## What "expectation mismatch" actually counts (2026-09-02)

The summary used to print one number — 151 — and one number here cannot be
acted on, because it conflates three populations with different owners and
opposite meanings. A prompt's `expected` is a PREDICTION of what the engine
will do, and predictions age in both directions. The tool now prints the
split, and names the rows in the only column that is a backlog:

| population | count | what it means |
|---|---|---|
| engine gained | 64 | the corpus said `missing`; the engine now answers. The expectation is stale because the engine IMPROVED. Nothing is wrong. |
| engine stood aside | 19 | the corpus claimed an answer; the engine declined (`not-yet-modeled`). **The tail worth working** — but named for what was OBSERVED, because the cause is not established. See below. |
| route differs | 68 | both answer, by different roads. The author predicted one and the engine took another. |

**The 19 need the same suspicion applied to them that they applied to the
64.** Naming this column "capability absent" would assert a cause it has
not established, and early reading says these are not one thing either.
At least four sub-kinds are now known, three of them measured rather than
read:

- **The script never reaches the capability.** `aq-035`, `aq-037` and
  `aq-038` are classic gas tests written as two lines — `add v1 <gas>`
  then `test v1 <reagent>` — with no `seal`. A fresh vessel is open, so
  the gas has left before the test runs, and `gas_tests.rs` implements all
  four with sourced thresholds. Add `seal v1 500mL` and all three go
  positive immediately. The engine is right and the script is incomplete:
  it omits the step a real bench requires, which is collecting the gas
  before testing it. (Found and measured by kerotakis-e9.)
- **The engine answers, confidently, and is wrong.** `aq-036` looks like
  one of the three above and is not. Sealed, it reports "damp red litmus —
  negative", because `NH3` in this registry is AMMONIA SOLUTION (its
  registry name is literally "ammonia solution") and there is no gaseous
  ammonia species at all — so the headspace fraction is always 0.00% and
  the test reports an absence the bench never modelled. It also teaches the
  opposite of the chemistry: damp red litmus DOES identify ammonia, and
  holding it over the bottle is the school demonstration. A silent stand
  aside is bad; a confident negative on a question the bench cannot see is
  worse, because nothing about it looks like a gap. (kerotakis-e9.)
- **A genuine absence.** `mat-096`/`mat-099` want rusting over `wait 1h`,
  `mat-003` wants surface area from `grind` to change a rate, `mat-034`
  wants a polymer to dissolve, `mat-011` wants the conductivity of a solid
  metal from a meter that is Kohlrausch and wants a solution.
- **A negative the engine is right to give and wrong to explain.**
  `aq-089` asks whether a magnet removes copper powder — it does not,
  copper is not ferromagnetic, and `MagnetSeparated` carries
  `attracted`/`remained` and could say so. `aq-087` asks whether dissolved
  salt appears in a neutral-solute chromatogram — it does not, and
  `Chromatographed.outside_method` exists precisely to name what the method
  cannot see. In both the refusal is correct and `not-yet-modeled` is the
  wrong reason for it.

The 19 rows span ten owning tasks — EXP-25 (4), EXP-18 (3),
CAP-16 (3), EXP-15 (2), BRD-023 (2), and one each for CAP-11, CAP-23,
CAP-25, EXP-13, BRD-014 — so they are a distributed backlog rather than one
defect, and each belongs to the task that claimed the capability.

**This answers the queued th-030/th-059 item, and the answer is that there
is nothing to fix in the classifier.** Both rows are in the *engine gained*
column: they expect `missing` and observe `computed`, and both are owned by
BRD-014, which is exactly the tranche that gave those materials real
registry identities after the expectations were written. The reverted
classifier change of 2026-08-31 was trying to make the engine's OBSERVATION
match a stale EXPECTATION — which is backwards, and is why it downgraded 49
honest rows on the way. The expectations are what aged, not the classifier.

Whether a stale `expected` should be rewritten in bulk is a content ruling
and is deliberately not taken here: it turns on whether `expected` is a
prediction of the engine or a requirement on it, and those two readings
want opposite fixes.

## Handover item 8, and why it is not implementable yet (2026-09-02)

9f's formulation after their first attempt was reverted: *an apology counts
as missing only when it stands UNANSWERED at the end of the script.* That is
the right direction — the engine emits `NotYetModeled` both as a stand-aside
AND as a DISCLOSURE offered beside a real answer, and reading the second as
the first is what recorded a vessel that burst, with a danger hazard, as an
unanswered question.

**A rule was written, measured, and deliberately NOT shipped.** "The apology
stands alone when everything else in the final step is bookkeeping" moves
`missing` from 157 to 138 and the stood-aside column from 19 to 10. It also
looks entirely correct until the moved rows are read one at a time:

| row | final step says | verdict |
|---|---|---|
| `aq-062` | `BURST at 10872 kPa` + a danger hazard, beside a supersaturation disclosure | **true disclosure** — the question is answered |
| `aq-089` | `no magnetic species present` — copper is not ferromagnetic, which IS the answer | **true disclosure** |
| `mat-006` | `0.0100 mol hydrogen ↑` — the gas the question asks about | **true disclosure** |
| `aq-085` | `the lower layer drained` — but the question is whether IODINE partitions, and iodine's dissolution is exactly what is unmodelled | **not answered** |
| `mat-003` | hydrogen forms — but the question is whether CRUSHING changes the rate, and rate is exactly what is unmodelled | **not answered** |
| `aq-016` | `100% of the suspended particles settle` — adjacent to "will sulfur dissolve", not identical to it | **not answered** |

Roughly half. The rule cannot tell an answer to THE QUESTION from an answer
to something else that happened in the same step, so shipping it would have
made the corpus assert the engine answered questions it did not — the same
damage as the reverted change, in the opposite direction and harder to see,
because the diff shrinks the missing column and looks like progress.

**What blocks it, concretely.** `NotYetModeled { vessel, what: String }`
carries its subject as PROSE. A classifier cannot ask "is this apology about
the same thing as that answer" without matching sentences, which is the
defect class this programme has spent the day removing. Two things would
unblock it, in order:

1. `NotYetModeled` carries a machine-readable subject (a `SpeciesId`, or a
   stable reason id) alongside its prose — the same prose-to-id move as the
   catalog reasons, the unmet reasons and the safety rule ids.
2. The prompt states what it is ASKING ABOUT, so "answered" can be checked
   against the question rather than against whatever else the step emitted.
   Without this, `mat-003` and `mat-006` are indistinguishable to the
   classifier: identical events, and only the question differs.

Until then the honest position is the current one — the column counts an
observation, the sub-kinds are documented, and `aq-062`/`aq-089`/`mat-006`
are known-contaminated rows rather than silently reclassified ones.

The baseline contains no solver failures. The last two left on
2026-08-31 when the aqueous engine gained its validity boundaries —
and both rows got *better* than a refusal, because the crash had been
sitting on top of an answer. `th-022` (extreme sealed-water heating)
no longer asks PHREEQC to converge beyond llnl.dat's 300 °C
parameterisation, and the burst physics that was the question's whole
point classifies as `computed` — matching the prompt's expectation for
the first time. `th-057` (permanganate heated with ethanol) no longer
crashes the aqueous engine on a mixture that was never aqueous, and
the curated oxidation route underneath answers as `curated`; the
speciation the model cannot claim is declined, with the reason spoken.
A solver failure returning here is a regression; record it in this
table with its owning task while it is being fixed.

| Prompt | Owner | Failure route |
|---|---|---|
| _none_ | | |

The five CEA failures the initial baseline carried (`aq-106`, `th-034`,
`th-035`, `th-097`, `th-102`) were removed on 2026-08-29 after the Gibbs
solver's phase handling was repaired: condensed records are admitted only
within their data range, same-name phase records merge their intervals,
each substance's phase family joins the pool, a singular solve re-seeds
crushed carriers instead of failing, and the adiabatic bisection survives
bracket points that cannot converge. `th-097` still runs from a garbage-in
vessel temperature (the latent-heat plateau is BRD-032's open model); what
changed is that the solver now answers consistently instead of dying.

Each owner may remove its failure from the baseline only after the full prompt
executes without `SolverFailed` on both native CI platforms.


## Refresh 2026-09-02, second: EXP-25's gas tests were never collected

`aq-035`, `aq-037`, `aq-038`: `missing`/`not-yet-modeled` ->
`qualitative`/`typed-observation`. **No engine capability was added.** The
four classic gas tests — limewater, damp litmus, squeaky pop, glowing
splint — have been implemented in `kerotakis-core::gas_tests` since EXP-31,
with sourced thresholds (H2 LEL 4% NFPA, O2 relight 25% CLEAPSS). The
scripts never reached them:

```toml
script = ["add v1 CO2 0.01mol", "test v1 limewater"]
```

A fresh vessel is open, so the gas left as it formed and the test refused,
correctly, with "collect over a sealed vessel first". The corpus was asking
the engine to test gas it had already let escape. Adding `seal v1 500mL` —
the step a real bench requires, and one other prompts in this file already
use — makes all three answer positive.

So this refresh records a **corpus fix, not an engine change**, and it is
the reason these three sat in the "capability absent" column while the
capability was complete. That column has been renamed `engine_stood_aside`
(#327) precisely because it named a cause it had not established; these
three are the worked example.

`aq-036` (damp litmus / ammonia) is NOT refreshed and stays
`missing`/`not-yet-modeled`. Its script gained the same `seal`, but the
reason it stands aside changed from "the vessel is open" to the true one:
`NH3` in this registry is *ammonia solution* (standard phase Liquid, formula
NH3(aq)) and there is no gaseous ammonia species, so the headspace the test
reads is always empty. It previously answered "litmus stays red — NH3 mole
fraction 0.00% is below the detection floor", a confident negative about a
gas the bench never had a way to put in the headspace, and one that teaches
the opposite of the chemistry.

Note what that refusal must NOT say, because the obvious phrasing is false:
it is not that volatility is unmodelled. `senses::waft` walks the vessel
contents directly, so `smell v1` on the same vessel reports "sharp, pungent
ammonia". Two paths, one physical fact, opposite answers — the gas tests
read the headspace inventory and nothing transfers dissolved NH3 into it.
`gas_tests.rs::smell_and_gas_test_disagree_about_dissolved_ammonia` pins
that divergence, and is written to FAIL once a path from solution to
headspace exists, so the pin cannot outlive the gap it records.

`aq-036`'s `expected = "curated"` is deliberately left alone. Whether
`expected` is a prediction *of* the engine or a requirement *on* it is the
open design question #327 names, and settling it by editing one row would
be the threshold-move that question exists to prevent.


## Refresh 2026-09-02, third: aq-087, an empty chromatogram is a result

`aq-087` ("Will dissolved salt appear in this neutral-solute chromatography
method?"): `missing`/`not-yet-modeled` -> `computed`/`computed-route`.

The column already computed `outside` — exactly the species it cannot
separate — and then discarded it in favour of "nothing dissolved here has a
curated UNIFAC decomposition, so the column's method is silent". That
reports the ENGINE's silence rather than the COLUMN's result. The question
has a real answer: no, and here is what rode past unseparated. An empty
chromatogram is a chromatogram — the run happened and the detector saw
nothing.

**One row moved that should not have, and catching it is why this refresh is
one row rather than two.** The first version of the change also moved
`bio-104` ("Can paper chromatography separate two food dyes?"), which began
reporting that betanin and curcumin "pass with the water and are not
separated". Paper chromatography DOES separate those two dyes — it is the
classic demonstration — so that would have dressed a missing parameter set
as a confident negative, the same defect this file records for `aq-036`.

The line the fix now draws: `outside` was doing two jobs.

- **Outside the METHOD.** A partition column separates by how a *neutral*
  solute divides between two phases, so an ion does not partition however
  good the parameters get — it wants ion exchange, which this column is
  not. That is a permanent property of the method, and a result.
- **Outside the MODEL.** A neutral solute with no curated group
  decomposition is one a real column would separate. That is a gap, and it
  stays `not-yet-modeled`.

Split on charge (`is_ionic`).

**Correction, and it is a live example of why a claim needs a date.** When
this was written `bio-104` was the worked case for the second row: betanin
and curcumin had no curated decomposition, so the change left them
`missing` with a refusal naming them. #337 (KID-9) then gave those four
dyes curated partition coefficients, and on main `bio-104` is now
`computed`/`typed-engine-event`. So the sentence "bio-104 is unchanged and
still missing" was true when written and is false now, with nothing in this
change having moved.

The code is unaffected — the dyes are injectable, so the unparameterised
branch does not fire for them — and the distinction it draws is unchanged.
What is gone is its live corpus example, which for a guard is the right
state rather than a problem: the unit tests still cover both sides, and the
next neutral solute without groups will meet a refusal that tells the truth
instead of a confident negative.


## `expected` is a requirement, not a prediction (2026-09-03, owner ruling)

The field was doing two jobs with opposite fixes, and the previous section
said so and deliberately left the choice open. The owner has now ruled:
**`expected` is a REQUIREMENT** — what a prompt must eventually answer, and
by which route. What the engine actually does is the baseline's job, and the
baseline is the record that is drift-gated.

The immediate consequence is that `expected = "missing"` is incoherent.
Nothing requires a bench to stay silent. As a *prediction* it read "we do
not expect an answer yet", which was often true; as a requirement it says
the engine must refuse, which nobody wants. 196 of 500 prompts carried it,
and 64 of those were counted as mismatches against a requirement no one had
made.

So `expected` becomes optional — absent means no requirement stated, which
is the common and honest position for a question worth asking that nobody
has committed to a route for — and `lint` **refuses** `missing` at load.
Enforced in the schema rather than written down here, because a rule that
lives only in a doc comment is how the field acquired two meanings in the
first place.

No row's behaviour changes and **baseline drift is 0**, which is the useful
confirmation: `expected` never fed the baseline, so the two records really
were independent. What changes is the printed count — mismatches 148 -> 86,
now entirely the two populations that are real:

| population | count | what it means |
|---|---|---|
| engine stood aside | 15 | a required answer the engine does not give. The tail worth working. |
| route differs | 71 | both answer, by different roads. Whether the ROUTE is required is the open half. |

The `engine_gained` bucket is gone with its cause.

### Three findings from doing it, recorded because each cost a cycle

**`expected` was doing a third job nobody had documented.** Two places in
`lint` used `expected == missing` to mean "this prompt declares an
intentionally unsupported input" — gating `parse_boundary` handling. That
is what the `parse_boundary` field is for, and the checks now key on it.
A prompt whose script cannot parse also cannot be required to answer by any
route, and that is now its own stated rule.

**The shards are not consistently formatted.** `materials-handling.toml`
writes `expected="missing"` with no spaces; the other three write
`expected = "missing"`. A first pass matching the spaced form silently
missed 44 rows in that one shard. Any script written against these files by
literal match will skip it. Not reformatted here — a whitespace change
across 125 rows would bury 196 real deletions — but worth a separate pass.

**Two counts disagreed and the disagreement was the finding.** A grep said
152 and an earlier regex said 202. Chasing it rather than trusting the
smaller number is what exposed the formatting split; and the residue
reconciles exactly to the six rows #337 moved from `missing` to `computed`
(aq-046, aq-102, aq-120, bio-104, th-067, th-068), which is a better check
than either number alone.

## Baseline refresh 2026-09-03: four rows, because solid fuels now burn (KID-12)

`kerotakis_core::combustion` adds a curated complement to the CEA thermal
solver for the three solid fuels NASA's `thermo.inp` has no records for:
paraffin (candle wax), cellulose (paper) and sucrose. Four baseline rows
move, all of them from `reason_code = "typed-engine-event"` to
`"computed-route"`, and **no row changes its outcome** — every one of them
was already `computed`:

| row | question | before | after |
|---|---|---|---|
| bio-008 | Why does sugar become caramel when heated? | typed-engine-event | computed-route |
| bio-009 | Does caramelisation happen without proteins? | typed-engine-event | computed-route |
| th-030 | Will candle wax melt before it catches fire? | typed-engine-event | computed-route |
| th-058 | Can sugar burn, or does it only caramelise? | typed-engine-event | computed-route |

The scientific content of the change: each of those scripts drives its
vessel above the fuel's autoignition temperature (paraffin 473 K, cellulose
506 K, sucrose 683 K) with room air available, so a combustion reaction now
occurs where previously the vessel reached the model boundary and the
disposition came from a typed observation instead of a reaction. `bio-008`
and `bio-009` deliver 20 kJ into 10 g of sucrose, which in this bench's
lumped heat model is a ~1600 K rise: at that temperature sugar in air does
not caramelise, it burns, and the prompts' own question ("does it only
caramelise?") is answered — with the standing caveat, stated in the fuel's
provenance, that the browning chemistry *between* melting and burning is
not modelled at all.

`th-051` ("Will a candle keep burning in a sealed jar?") stays `computed`
and required a fix to keep it there. The new `FlameStarved` event was first
added to the classifier's typed-observation list wholesale, which demoted
th-051 to `qualitative` — the candle burns, runs the jar down to 16%
oxygen and stops, and that is a computed result, not an observation. The
classifier now keys on the event's `burned` field: a flame that never
caught (a carbon-dioxide-smothered one) is a typed observation; one that
burned first is computed. The demotion is the kind of thing baseline drift
exists to catch, and it caught it.

## Refresh 2026-09-03: aq-049 can finally ask its own question

`aq-049` ("What dissolved ions are present in a sodium chloride
solution?"): `qualitative`/`typed-observation` -> `computed`/`computed-route`.

The bench has always answered this exactly — `particles v1` draws the
census and names Na+ and Cl- by their own labels. No corpus script could
pose it, because `particles` was a SESSION command and the lint refuses
those:

    prompt aq-049: script line 3 is a session command, not an operator

So the prompt ran `look v1` instead, which reports colour and clarity: an
answer to a different question, and the row sat in the backlog as though
the capability were missing. It was not missing. It was unaskable.

`particles` is now an operator (`Operator::Particles`, `Event::
ParticlesCounted`), which is what it should always have been — it asks the
VESSEL what is in it, which is a bench question, not a shell one. Reading
state changes none of it, exactly like `smell`.

The reason code is worth noting: `computed-route`, the strongest one, not
the `typed-engine-event` fallback. The census is drawn from SOLVED
SPECIATION, so the aqueous route that produced it is the route being
reported — and the corpus's `expected = computed` was right all along.

The REPL and the MCP server keep their existing path: `json_particles` is
part of the MCP contract and is untouched. Both routes call the same
`particles::census`, so there is one implementation of the picture and two
ways to ask for it.

## Refresh 2026-09-03: household ammonia computes its own pH

One row, `aq-031` ("What is the pH of a household ammonia solution?"),
`missing`/`not-yet-modeled` → `computed`/**`computed-route`**. Note which
reason code: the strongest one. The pH came off the aqueous route, from the
databases' own equilibrium constants, not from a curated table.

The row had been sitting in the stood-aside backlog under a diagnosis that
was true and useless: "NH3 gets no aqueous role from `derived`". It got no
role because the oxyanion-group table had a row for ammonium and none for
ammonia, so the nitrogen fell through to the residue rules, where a bare N
is not allowed. Meanwhile both shipped databases that carry nitrogen have
always known how to speciate it — `NH4+ = NH3 + H+`, N(-3) mastered by
NH4+. Nothing was asking them. It was a one-row gap in a curated table
dressed as a modelling limit.

The answer is 11.12, against the textbook 11.13 for 0.1 M of a base with
Kb 1.8e-5. Nobody typed 11.13 anywhere.

**The row's neighbour did not move, and that is the point of the ordering.**
Group extraction is greedy and ordered, and `NH4` must be tried before
`NH3` or ammonium chloride decomposes as ammonia plus a stray proton —
booking a school reagent as a weak base plus hydrochloric acid. `aq-032`
and every other N-bearing row is untouched; the table was simulated over
all 141 registry compositions before the change and exactly one formula
moves.

**`aq-053` (bleach) is deliberately NOT refreshed.** It shares a symptom
with `aq-031` and nothing else. Every `.dat` vendored with iphreeqc was
searched by name for a hypochlorite species — `HClO`, `ClO-`, `Cl(1)`, the
word itself — on 2026-09-03. There is none, in any of them; the `ClO-` hits
are all perchlorate. `contribution_from_counts` already names hypochlorite
in the comment on the guard that rejects it, and the guard is right. That
row is a correctly refused one whose refusal is merely mute, which is a
different piece of work.

## Refresh 2026-09-03 (second): vinegar and baking soda fizz at last

`aq-059` (vinegar into a baking-soda solution) and `bio-114` (vinegar on an
eggshell) move `computed`/`computed-route` → `curated`/**`curated-route`**,
and the headline mismatch count rises from 84 to 86. **The number got worse
because the bench got better.**

Both rows name a curated reaction:

    NaHCO₃ + CH₃COOH → CH₃COONa + H₂O + CO₂↑
    CaCO₃ + 2 CH₃COOH → Ca²⁺ + 2 CH₃COO⁻ + H₂O + CO₂↑

Reviewed, carrying provenance, and until now reachable only if you built the
experiment backwards. The aqueous readback booked the whole Acetate element
total as `CH3COO-`, so once vinegar had been through a solve the acid named
in the reactant list was no longer in the vessel.

**It was order dependence, not absence, and that is worse.** `curated` runs
before the aqueous tail, so on the step where the vinegar is ADDED the
ledger still holds CH3COOH and the match succeeds — put the soda in first
and it works. Put the vinegar in first, which is the order most people and
most lesson scripts use, and the carbonate equilibria answer instead. A bug
that only bites in the common order, and whose workaround ("add the solid
first") someone would eventually find by accident and never understand.

And the two routes did not agree about the SIGN of the temperature change.
Vinegar and baking soda is one of the few kitchen reactions a child can
feel, and what it does is get cold. The aqueous route had it warming by six
and a half degrees, because it books the heat of H⁺ + OH⁻ → H₂O for whatever
consumed the acid and nothing for the endothermic half — the bicarbonate
breaking up and the gas leaving. With the split in, both orders reach the
curated route and both cool: 25.0 °C → 23.6 °C either way. (Found by
kerotakis-5f while checking the KIDS audit against this change.)

**This file said so, and nobody could read it.** The classifier checks
`succeeded(Curated)` first, so `computed-route` on a row whose entire
subject is a curated reaction was the baseline asserting, in writing, that
the reviewed equation produced no events. A reason code that encodes an
absence only if you already know the precedence order is not written down in
any useful sense. If you take one thing from this section, take that.

The arithmetic is exact, and the split is what makes it so:

    0.05 mol CH₃COOH into 100 mL  →  0.0497 CH₃COOH + 0.0003 CH₃COO⁻
    then 0.05 mol NaHCO₃          →  0.0497 mol CO₂ (curated route)
                                   +  0.0003 mol CO₂ (aqueous route)

`bio-114` balances the same way (0.0498 + 0.0002) and the shell dissolves to
its saturation residue, 0.08%, which is where a real shell in a real glass
of vinegar stops.

**`expected` is deliberately NOT changed on either row.** It is prescriptive
— it says what the bench ought to do, not what it does — and moving a
prescription in the same commit that changes the engine is how a corpus
stops being able to disagree with its own bench. Whether these two are
better answered by a reviewed equation or computed from carbonate equilibria
is a real question, and it belongs to EXP-2 and CAP-1.

#### The half of this that was not chemistry

Putting the acid back in the ledger broke magnesium in vinegar, and it is
worth recording why, because the shape recurs. `displacement` reads a
vessel's unspent acidity off `−solute_charge`. That was the whole of its
acidity for exactly as long as the readback stripped every weak acid of its
proton: 0.1 mol of acetic acid arrived as 0.1 mol of acetate ion, net charge
−0.1, and the proxy read the right number **by coincidence**. With the acid
present the same beaker reads about −1e-3 — all that is dissociated at any
instant — and the metal stopped dissolving.

A weak acid's titratable protons are not its charge. `LEDGER_ACIDS` and
`bound_protons` now count them, and spending one converts the acid to its
conjugate base so it cannot be spent twice. Ammonium is deliberately left
out: it is a titratable acid, but its pKa is 9.25 and this bench computes
thermodynamics with an overpotential gate rather than a rate, so claiming
magnesium dissolves promptly in ammonium chloride would be a statement about
speed that nothing in that module is entitled to make.

## What the first refresh held back, and why (superseded)

This section described an `Acetate` protonation split as measured, correct
and deliberately held back. It has since landed — see "Refresh 2026-09-03
(second)" above, which supersedes it.

Kept as a pointer rather than deleted, because the reason it was held is
the part worth remembering: it was not the chemistry that blocked it but
`displacement::oxidant_available`, which read a vessel's unspent acidity
off its net charge and only ever agreed with reality because a different
defect upstream was stripping weak acids of their protons first.
`tests/curated_reactants_survive_a_solve.rs` now fails if anyone adds a
curated reaction on a reactant the solver renames, so this class cannot be
introduced silently again.
## mat-012's script now asks mat-012's question (2026-09-03, KID-19a)

`mat-012` asks "How can density distinguish copper, zinc, and aluminium
pieces?" and its script weighed five grams of each on a balance. Five grams
of copper, five grams of zinc and five grams of aluminium all weigh five
grams: the script exercised the one measurement that cannot answer the
question it was written for. There was no density instrument to use
instead, so this was not an oversight when it was written.

KID-19a adds one (`measure v1 density`, also spelled `hydrometer`), and the
script now takes both readings. The three vessels answer 8.96, 7.14 and
2.70 g/mL against three identical balance readings.

**The disposition does not move, and that is deliberate.** The classifier
returns `qualitative` for any `handle_and_inspect` prompt that produces an
`Observed` or `Measured` event, so a reading with a value and a unit is
classified the same as looking at the vessel. `mat-012`'s own
`expected = "qualitative"` says the corpus author read it that way too.
Whether an instrument reading with a unit ought to count as quantitative is
a real question about what `Disposition::Qualitative` means — the enum
carries no definition — and it belongs to whoever owns the classifier, not
to a commit that adds an instrument. Nothing here changes `expected`, the
classifier, or any other row: baseline drift for this change is zero.

What changed is only that the script now performs the measurement its
question names. A prompt whose script cannot reach its own question is a
gap in the corpus that no engine work can close, and it is worth looking
for others: the question is the prescription, and the script is only an
attempt at it.

## Refresh 2026-09-03 (third): the other end of the same renaming

`aq-061` (will a sealed vinegar-and-baking-soda bottle build pressure?) and
`bio-004` (why does baking soda bubble in acidic cake batter?) move
`computed`/`computed-route` → `curated`/**`curated-route`**. Mismatches
86 → 87, for the third time today, and for the third time the number got
worse because the bench got better.

Look at what both scripts have in common:

    add v1 water 100mL ; add v1 NaHCO3 … ; add v1 CH3COOH …

Water first, then the soda. So the bicarbonate dissolves and goes through a
solve before the acid arrives, the readback books its carbon as `HCO3-`, and
the reactant named `NaHCO3` is no longer in the vessel. The reviewed
equation was unreachable — from the opposite end to the one the acetate
split fixed. **Renaming is symmetric, and the first fix only did the acid
half.**

The remedy is the one already used twice for permanganate: a second entry
written in the names the beaker actually holds. `HCO₃⁻ + CH₃COOH → CH₃COO⁻ +
H₂O + CO₂↑`, on the ion rather than the salt, with the sodium absent from
both sides because it is a spectator. Any bicarbonate reaches it, not only
bicarbonate that arrived as baking soda — which is correct: acid poured into
a bicarbonate solution fizzes however the bicarbonate got there.

`expected` is untouched on both, for the reason given twice above.

### What this pair of rows was hiding

`aq-061` seals the bottle and measures the pressure. It was reaching an
answer, from carbonate equilibria, and the answer was plausible. Nothing in
the corpus, the baseline or any test said the reviewed equation had not
fired — the reason code said so, but only to a reader who knows the
classifier checks the curated route first.

So a row can be *right* and still be evidence of nothing. Both of these
computed a number the whole time. What changed today is which model produced
it, and that was never visible in the file.

## The answer-invariance sweep (2026-09-04)

`tools/curiosity-answer-invariance.py`. Written after `mat-012` — a prompt
asking how density distinguishes copper, zinc and aluminium, whose script
weighed five grams of each on a balance — turned out to have matched its
own `expected`, appeared in no mismatch list, and read as evidence of
coverage for as long as the corpus had existed.

**The rule needs no vocabulary: a prompt that distinguishes N things must
produce N different answers.** If a script fills two or more vessels with
different things and the bench says the same thing about all of them, the
script cannot answer any question that separates them — whatever events it
emits and whatever disposition it earns.

Two refinements make it precise, and both were found by getting it wrong
first:

* **Setup echoes are not answers.** A first attempt compared whole
  per-vessel output and found nothing, because `v1: +0.0787 mol copper`
  differs from `v2: +0.0765 mol zinc`. That difference is the script
  reading back what was typed into it. Only what the bench says *back*
  counts, and with the echoes dropped `mat-012` scores 3 vessels → 1
  answer.
* **A refusal repeated is not this defect.** `mat-011` ("why are wires
  copper rather than iron?") measures conductivity on two dry metals and
  both vessels answer *"the conductivity meter reads nothing — no aqueous
  solution has been characterised"*. Two subjects, one answer — but that is
  an engine gap, already counted as `missing`, and no edit to the script
  would change it. Prompts where every vessel refuses are excluded.

The tool is validated against the instance it was written for: restoring
`mat-012`'s pre-fix script makes it exit 1 and print `3 vessels, 1 distinct
answers`; the current corpus exits 0.

### What it found, and the more interesting thing it did not

**Zero violations today**, on 8 comparison prompts. That is a thin result,
and the reason is the finding:

| | count |
|---|---|
| prompts | 500 |
| comparative question ("than", "which", "faster", "difference between"…) | 55 |
| …whose script builds two or more filled vessels | **6** |
| …whose script builds one | 49 |
| …of those single-vessel comparisons, passing today | 22 |
| any prompt with two or more filled vessels | 14 |

**Fifty-five questions ask a comparison and six of them build something to
compare.** Some of the 49 are legitimate — "does a larger spoonful leave
crystals in the same water" is one vessel by construction, and several
compare a vessel against a value rather than against another vessel. Many
are not: "does warm dough rise faster than cold dough", "does crushing
magnesium make it react faster with acid", "does hot water dissolve more
sugar than cold water" each name two conditions and script one.

Those 49 are not currently a lie, because most of them are `missing` — the
engine cannot answer them either way, so nothing false is being claimed.
They become one the moment their mechanism lands: the row will start
passing, on a script that never built the second condition. **That is
mat-012's exact history**, and it is why the sweep is checked in rather
than run once.

The 22 that pass today are the half worth reading. Sorting findings by
whether the row currently passes is the discipline this sweep is built on:
a failing row is already on somebody's list, and a passing row whose script
cannot reach its question is a false statement about coverage that nothing
else in the harness will ever contradict.

### Wiring it into the gate

Not yet. It runs each comparison prompt through the shipped binary, which
belongs beside `coverage curiosity --check` rather than in a unit test, and
folding it in properly means teaching that runner to track answers per
vessel. It exits non-zero today, so it can be added to CI as it stands
whenever someone wants it; it is checked in now so the rule is written down
where the next person writing a prompt will meet it.

## Refresh 2026-09-04 (second): the cell that needs no metal

`mat-063` (what forms at the electrodes in salt-water electrolysis) →
`computed`/`computed-route`. `mat-064` (why copper plates onto one
electrode) and `mat-110` (electrolysis to remove rust) →
`qualitative`/`typed-observation`. All three were `missing`.

**Stood aside 14 → 12, and this time by capability rather than by
reclassification.** The engine does more than it did; the measurement
followed. That is the distinction worth holding onto after #362, which
moved the number by changing what counted as an answer and was closed
unmerged for it.

The electrolyser modelled one cell: a metal standing in a solution of its
own ion. Brine needs no metal at all — two carbon rods — and the refusal
was accurate about the model and wrong about the chemistry. Now:

    add v1 water 100mL ; add v1 NaCl 0.01mol ; electrolyse v1 0.5A 30min
    -> 0.0047 mol hydrogen ↑,  0.0047 mol chlorine ↑,  pH 6.98 -> 12.86

The alkali is not a detail. That is the chloralkali process, and the caustic
soda is what the cell is for.

    add v1 water 100mL ; add v1 CuSO4 0.01mol ; electrolyse v1 0.5A 30min
    -> 0.0047 mol copper plated (0.296 g),  0.0023 mol oxygen ↑,  pH 1.44

Two questions, answered separately because they are separate: **how much**
is `n = I·t/F`, arithmetic with a constant already present for the activity
series; **what** is the activity series itself. A metal ion plates only when
it is easier to reduce than water — copper at E° +0.342 does, sodium at
−2.71 does not — and chloride is oxidised before water where there is
chloride to oxidise, which is the whole difference between the two runs
above.

Pure water still refuses, and that refusal is the answer: pure water does
not conduct, and a bench that electrolysed it would be teaching that it
does.

### `mat-110` moved and does not answer its question

Flagged here rather than quietly accepted. It asks whether electrolysis can
remove rust without dissolving the iron, and its script contains iron,
bicarbonate and water — **no rust**. What it now does is correct water
electrolysis in a bicarbonate electrolyte (0.0047 mol H₂, 0.0023 mol O₂),
which is real and is not what was asked.

That is a script that cannot reach its own question, so it belongs to the
answer-invariance sweep rather than to this refresh. It is recorded here
because a row moving out of `missing` for a good reason and a row moving out
for the wrong one look identical in the drift, and the only way to tell is
to read each one.

The 2026-09-05 bounded-electrochemistry audit now pins that distinction in
the baseline. `mat-056` receives the computed open-circuit lemon-cell voltage
it asks for. `mat-110` is an explicit boundary: water electrolysis occurring
in a rust-free script cannot establish rust-layer reduction or preservation
of a coherent iron object. Counting its gas events as the answer would inflate
coverage without delivering the requested capability.

## aq-067 was waiting for a bottle (2026-09-04)

`aq-067` — *"Does lemon juice neutralise a sodium bicarbonate solution?"* —
was declared `parse_boundary = "unknown_species"`, tagged
`material-recipe-gap`, and owned by BRD-014. It was not a failing row or a
gap in the engine: it was a **note that the shelf had no lemon juice**,
written into the corpus by whoever wanted the question asked.

K13's invisible-ink row needed the same bottle, so `lemon_juice` is now on
the shelf — 91% water, 4.7% citric acid, a little sugar. That fulfils the
note, so the declaration had to go, and the lint said so before I had
noticed:

```
prompt aq-067: declared parse_boundary Some(UnknownSpecies), observed None
```

That is the corpus working exactly as intended. A prompt that declares
*why* it cannot run is a to-do with an owner, and the lint refuses to let
the declaration outlive the reason. Compare the four stale rows in
`KIDS.md` this week, which had no such check and sat wrong for days.

**`missing`/`unknown-species` → `computed`/`computed-route`**, one row of
drift. The script now measures the pH it was always about: lemon juice at
1.86, then 0.0471 mol of carbon dioxide off, ending at 9.75 — so the answer
to the question is yes, and rather more than neutralise. `measure v1 ph`
was added to the script, because a prompt that asks whether something
neutralises and never reads a pH is the class the answer-invariance sweep
was written for.

## Bare English words, and three rows that got worse by getting better (2026-09-04)

**Thirty of the 147 `missing` rows were blocked on a name.** Not on data,
not on a mechanism — on the shelf being filed under the word a chemist
uses while the prompt used the word a child uses.

The evidence that this was an oversight rather than a decision is that
**German already had the bare words and English did not**. `Essig`, `Hefe`,
`Milch`, `Sand`, `Natron`, `Kreide` all resolve; `vinegar`, `yeast`,
`milk` did not. Thirty-five of fifty-six recipes were reachable by an
everyday word in German and only by a compound one in English. The project
had already decided a bare word is claimable; it had only done it once.

Thirteen bare English aliases now match the German: vinegar, milk, yeast,
soap, sugar, oil, pepper, glue, ink, bicarb, filings — and not `wax`,
because `candle_wax`'s own lot assumption declines it in writing, nor
`apple` or `cabbage`, because a fruit is not its juice and a vegetable is
not its indicator. `cola` and `sand` were rejected by the registry
validator as already claimed, which is the guard working.

**Fourteen rows opened**: `missing` 147 → 133, computed 229 → 236,
qualitative 45 → 52. Each had declared `parse_boundary = unknown_species`
and each declaration became false the moment the word resolved — the same
mechanism that caught `aq-067`. One of them, `bio-075`, then failed on
`wait 7d`: the parser has no day unit, and the parse boundary had been
masking a second problem behind the first. It is `168h` now, the same week
in a unit that exists.

### The three that regressed, and why they are not a mistake

`aq-123`, `mat-057` and `th-082` went **computed → qualitative**. All three
are copper-sulfate displacement or cell rows, and none of them uses a new
alias. The cause is K40's basic copper sulfates.

Precipitating antlerite releases protons — `Cu3(OH)4SO4 + 4 H+ = 3 Cu+2 +
4 H2O + SO4-2`, run backwards — so the solution is now slightly more acid
than it was. That is enough for the displacement model to add its honest
aside: *"iron should dissolve in this acid by the series (driving force
+0.25 V), but hydrogen has to form on iron"*. That aside is an `Inert`
event, `Inert` is in the classifier's typed-observation list, and that list
is checked **before** the route-based branch that would have said
`computed`.

So the chemistry is strictly better — more phases modelled, a more accurate
pH, an extra true statement — and the score is worse. The copper is still
plated: `0.009967 mol copper plated out onto iron`, unchanged.

**This is the answers-and-qualifies problem arriving from the engine's end
rather than the corpus's.** The other instances were rows that happened to
qualify; this is a change that made the bench explain *more* and be marked
down for it. The classifier ordering is the cause and it is not being
touched here: a commit that adds species has no business redefining how
rows are scored, which is the lesson of #362. Raised for whoever owns the
ordering, with these three rows as the worked example.

## The vocabulary batch, and what a stale `parse_boundary` costs

Ninety-five of the 145 `missing` rows carried reason code `unknown-species`:
the run never started, because the parser did not know a word. Counting the
distinct tokens gave 88, and the shape of that distribution is the useful
part — a handful wanted by three rows each (`silica_glass`, `gelatin`,
`apple`, `albumin`, `pondweed`), a long flat tail wanted by one. There is no
small set of additions that unblocks most of it; volume here is bought a
material at a time.

Twenty-six were added, closing 63 rows. Two things worth recording for whoever
does the next batch.

**Ask the engine, not the registry file.** The first pass diffed the corpus
tokens against `registry-source-v1.json` and reported `vinegar`, `milk`,
`soap`, `yeast`, `lemon_juice`, `flour` and `vegetable_oil` as unknown. All
seven already resolved. Material aliases do not live where that walk was
looking, so the list was wrong in the direction that wastes the most work —
it invited re-adding things that were already there. Probing the binary with
`add v1 <token> 1g` is a two-line loop and is ground truth.

**Every added material makes some `parse_boundary` declaration false, and the
lint will stop the corpus dead until they are removed.** Thirty-five rows
across two batches. This is the lint behaving exactly as designed — a
declaration is a to-do with an owner, and one that has become false is a
claim the corpus is still blocked when it is not — but it means the real unit
of work is *add the material, then sweep the declarations*, and a batch that
skips the sweep does not run at all. `kero coverage curiosity --json` names
every stale row.

**The validator's naming rules, in the order they bite.** A material's `name`
may not equal its `canonical_key`; an alias may not equal the key or another
alias case-insensitively (`Kaolin` in `de` against `kaolin` in `en` collides;
so does `Butter` against key `butter`); and a material may not shadow a
species name. Six of the twenty-six tripped one of these, all on the first
export, none of them interesting — but they are silent until export and the
error names an index into `material_recipes`, not a key, so keep a count of
where the new block starts.

## An insolubility remark that read as a claim about reactivity

Eleven species gained reviewed near-zero aqueous solubilities, and `solve.rs`
turned that into a positive `Event::Inert` — "it does not dissolve, and it is
still all there" — in place of a `NotYetModeled` apology. Seven rows improved
and seven regressed, and the regressions were not a scoring artefact. The
bench printed

    v1: starch does not react — starch does not dissolve in water: ...
    v1: 2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁

in one run. Both sentences are true and the pair is false.

The fix has two halves, and the split is the point. The engine holds back the
remark only when a curated reaction *that can fire in this vessel* consumes
the species — `curated::consumes`, built on a `fires` predicate extracted from
`CuratedSolver::applies` so the two cannot drift apart. Being a *product* does
not count, or chalk would lose its answer to reactions it is not in.

That fixes the sentence and not the row, because the remark is emitted when
starch is added and the amylase arrives on a later line: it is true when
spoken, and stale by the end of the transcript the row is graded on. So the
classifier gained the second half — an `Inert` beside a **succeeded curated
route** is an aside, not the result. Narrow for the same reason the
`Plated`/`CellVoltage` guard next to it is narrow: a computed route is not
enough. `bio-042` (starch + HCl + heat) and `mat-029` (PET + NaOH + heat) have
no curated hydrolysis, so their computed route is the acid or the base
speciating rather than the polymer doing anything, and there "the polymer is
unchanged" *is* the answer. Those two stay qualitative, which is what they
are, and they are the two accepted regressions in this branch.

Net: 63 rows out of `missing`, 2 regressed, expectation mismatches flat at 85
across every step — which is the number that would have caught a guard written
wider than its evidence.

### A rough edge left in deliberately: `0.0000 mol chalk dissolved`

Giving calcium carbonate its reviewed 0.0013 g/100 mL let the aqueous path
dissolve a real trace of it — 6.49e-6 mol — where before the run said "not yet
modelled". That is the improvement. It renders as

    v1: 0.0000 mol chalk (calcium carbonate) dissolved
    v1: chalk (calcium carbonate) does not react — chalk does not dissolve in
        water: its reviewed solubility is 0.0013 g per 100 mL, ...

and the first line is a number that says nothing, which is the defect this
branch spent its time hunting. It is left in. The event is *true* — there is
aqueous CaCO₃ in that beaker — so suppressing it would hide real inventory to
tidy a display; and the honest fix, not printing a non-zero quantity as
`0.0000`, means changing `{:.4}` in `render.rs`, which every golden in the repo
is written against. That is a formatting change with a repo-wide blast radius
and it does not belong at the end of a branch about materials. The two lines
do explain each other. Recorded here so the next person finds a known edge
rather than a fresh bug.

## A prompt was being classified on its neighbour's routes

`aq-091` came back `curated` from a smoke run and `computed` from a full run,
same script, same binary, deterministic in each. That is not a tolerance and
not nondeterminism — it is one prompt's verdict depending on which prompt
happened to run before it.

`SolverStack::equilibrate` clears `last_routes` on entry, so the field is only
ever the routes of the last step that *equilibrated*. A step that does not —
`new`, and any operator that only touches bookkeeping — leaves the previous
step's routes standing, and at the top of a script that is the previous
**prompt's** last step. `execute_prompt` then did

    routes.extend(stack.last_routes.iter().cloned());

on every step including the first, so the leading `new` of `aq-091` collected
whatever its neighbour had finished with. In the smoke run that neighbour left
`curated-reactions: Succeeded`; in the full run, `NotApplicable`. The
classifier reads `succeeded(SolverRouteKind::Curated)`, so the row was graded
`curated` on a route belonging to another prompt.

The fix is one line — clear `last_routes` before each step — and the comment
at that line is the part worth keeping, because the bug is invisible in the
place it does damage. Note what it cost to find: the *full* corpus is
unaffected (0 rows move), because there every neighbour happened to agree.
Only the smoke subset disagreed, and only because a material added in this
branch made `aq-091` run at all. **The bug was latent for as long as the row
was `missing`, and closing the row is what exposed it.**

Two consequences for anyone reading a coverage report:

- A route list is now a fact about its own prompt. Before this, a prompt whose
  script began with a non-equilibrating operator had a route list that started
  with someone else's evidence.
- The smoke gate is not merely a faster full run. It selects a different
  neighbour ordering, and that is exactly why it caught something the
  500-prompt run could not. A disagreement between the two is a signal about
  state leaking across prompts, not a flake to be retried.

## The stood-aside list, triaged row by row (2026-09-05)

The eleven remaining stood-aside rows are described as "the tail worth
working" — the ones where the corpus asked for an answer and the engine
declined. **Two of them are that.** Every row below was RUN, and the
classification is from what the bench actually printed.

### Answered already, and filed as standing aside — 4 rows

The bench gives the answer and then adds an honest caveat, and the caveat
is what the classifier sees.

| row | what it printed |
|---|---|
| `aq-062` "Can a sealed vessel burst?" | `BURST at 10854 kPa (glass rating ~405 kPa)` **plus a Danger hazard** |
| `aq-089` "Will a magnet remove copper powder?" | `no magnetic species present` — copper is not ferromagnetic, which IS the answer |
| `mat-006` "What gas forms when magnesium meets acid?" | `0.0100 mol hydrogen ↑` |
| `mat-096` "What three things does iron need to rust?" | `0.0010 mol reacted in 3600 s — 4 Fe + 3 O₂ → 2 Fe₂O₃↓` |

These cannot be fixed in the classifier, and the attempt is on record: PR
#362 moved two of them and was closed unmerged, because the same rule also
moved `mat-099`, which demonstrates the *opposite* of what it asks. What
unblocks them is the prompt saying what it is ASKING ABOUT, so "answered"
can be checked against the question instead of against whatever else the
step emitted.

### The script cannot reach the question — 3 rows

Comparative questions with single-condition scripts. **A perfect model
answers "how fast"; the question is "faster than what".**

| row | question | script builds |
|---|---|---|
| `mat-003` | does crushing make it react **faster**? | one condition, never compares |
| `mat-108` | how does acid **change** the corrosion rate? | one concentration, no baseline |
| `aq-085` | can **repeated small** extractions remove more than **one tiny** one? | one extraction |

`aq-085` is doubly blocked — iodine's dissolution is also unmodelled — but
even with iodine modelled the script could not answer it.

These need their scripts rewritten to build both conditions, which is a
prescriptive corpus change, and they are invisible to the answer-invariance
sweep because it compares ACROSS vessels and these fill one each.

### Permanent boundaries, correctly refused — 2 rows

Not to-dos. Nothing will change them and both now say why in full.

* `aq-036` — the damp-litmus test reads the headspace and there is no
  modelled path from dissolved NH₃ into it.
* `aq-053` — no `.dat` vendored with iphreeqc defines a hypochlorite
  species at all.

### Genuine capability gaps — 2 rows

* `mat-099` galvanic protection. Today the iron rusts anyway and the zinc
  does nothing. **In flight** as #387.
* `mat-011` "why are wires copper rather than iron" — the conductivity
  meter reads solutions only, and the question is about metallic
  conduction. Needs sourced data before wiring: the registry carries **no
  conductivity or resistivity property for any species**, checked.

### The number that matters

**Of eleven "gaps", two are capability gaps and one of those is already
being built.** Four rows are answered and mis-filed, three have scripts
that cannot reach their questions, two are permanent boundaries.

Anyone planning work off the stood-aside count should read this first. The
column is not a backlog of missing chemistry; it is mostly a backlog of
questions the corpus cannot yet ask precisely enough to grade.

### And the sharpest single piece of evidence in the corpus

`mat-003` and `mat-006` print **byte-identical output** — same equation,
same 0.0100 mol of hydrogen, same caveat about the leftover magnesium.
`mat-006` is answered. `mat-003` is not. The difference is entirely in the
question, and no classifier that reads events can ever see it.

## The CAP-16 cluster is not a capability gap (2026-09-04)

Three of the ten remaining stood-aside rows are filed under CAP-16, surface
area and rate, and read as a single missing capability: the bench does not
compute rates. **Building one would not move any of them.** Their questions
are comparative and their scripts build one condition:

| row | question | script |
|---|---|---|
| `mat-003` | Does crushing magnesium make it react **faster**? | grinds, never compares against unground |
| `mat-034` | Can ground powder dissolve **faster than a solid chunk**? | names two conditions, builds one |
| `mat-108` | How does acid **change** the corrosion rate? | one acid concentration, no baseline |

Every one is a single vessel. `mat-003` grinds the magnesium and then has
nothing to compare the ground sample with; a perfect kinetics model would
answer "how fast", and the question is "faster than what". `mat-034` states
both conditions in its own sentence and builds only the first.

So kinetics is **necessary but not sufficient** for these rows. Each also
needs its script rewritten to build both conditions — which is a corpus
change and a prescriptive one, and belongs with whoever owns CAP-16 rather
than to a refresh.

This is worth writing down because the mis-attribution is expensive in one
direction: it invites a large capability build on the expectation that ten
stood-aside rows become seven, and they would not.

### And the answer-invariance sweep cannot see them

`tools/curiosity-answer-invariance.py` compares what the bench says **across
vessels**: a script that fills two or more and gets one answer cannot
distinguish them. These three fill one vessel each, so the sweep is blind to
them by construction — not a flaw in it, a different shape.

The shape is: **a comparative question whose script builds a single
condition.** It is the same defect as `mat-012` — a script that cannot reach
its own question — arriving without the multi-vessel signature that makes
`mat-012` detectable. `mat-012` weighed three metals and got three identical
readings; these grind one sample and never weigh the other.

Worth a second pass over the corpus for it. The mechanical part is
findable — a script that performs a condition-changing operation (`grind`,
`heat`, a concentration choice) exactly once, in one vessel — even though
deciding whether the question is comparative needs a reader.

## 2026-09-05 — six rows, and the word that was the whole blockage

`BRD-012.S04` added six pure substances to the registry. Nothing else in
the engine changed: no solver, no classifier, no curated reaction. All six
rows had been failing at the **parser** with `unknown-species`, which means
no solver had ever seen them — the run stopped on a word.

Each row and what its new outcome actually rests on:

- **th-044 `methane`, th-045 `propane`, th-046 `butane`** —
  `missing`/`unknown-species` → `computed`/`computed-route`. `thermal.rs`
  maps registry species to NASA CEA records **by chemical composition**, and
  the vendored CEA database defines CH4, C3H8 and both butane isomers. So
  `ignite` now reaches the same Gibbs equilibrium solve that already burns
  sulfur (th-043) and paper (th-059), and the three rows compute an exhaust
  composition and an energy. What this is **not** is a combustion mechanism:
  there is no rate, no ignition delay and no flame model behind these three,
  and BRD-041's acceptance criteria are untouched by them. The record says
  `computed-route` because a computed route really did succeed; the reader
  who wants kinetics should read BRD-041, where this is stated again.

- **th-095 `helium`** — `missing`/`unknown-species` →
  `qualitative`/`typed-observation`. The vessel seals and the barometer
  reads, and the classifier files a `handle_and_inspect` prompt whose events
  are observations as qualitative — exactly as it already does for th-094's
  hydrogen. The pressure itself is computed from the amount of substance,
  which is the point of the question, and helium is the one substance on the
  shelf with no chemistry underneath it to confuse that.

- **th-028 `naphthalene`** — `missing`/`unknown-species` →
  `computed`/`typed-engine-event`. Read this one carefully, because the
  disposition flatters it. `typed-engine-event` is the classifier's weakest
  evidence: it means "no solver route claimed this vessel, but typed events
  happened, so call it computed". The question is whether mothballs sublime,
  and **the bench does not model that**. Naphthalene's slow loss at room
  temperature is a vapour-pressure phenomenon; what the species carries is a
  melting point and a boiling point, and its transition record says so in a
  boundary note. The row no longer stops at the parser. It does not answer
  the question, and closing it should not be read as if it did.

- **th-070 `hydrogen_sulfide`** — `missing`/`unknown-species` →
  `computed`/`typed-engine-event`. The same caveat, more sharply. Silver
  tarnishes because it forms Ag₂S, and there is **no sulfide ion and no
  silver sulfide on this shelf** — so nothing tarnishes. The word parses,
  the gas is weighed into the vessel, typed events follow from handling it,
  and the classifier's fallback calls that computed. The chemistry the
  question asks about is still absent, and the species' own provenance line
  records that it forms no metal sulfide.

Two of the six therefore graduated on the classifier's fallback rather than
on an answer. That is worth stating plainly rather than counting six closed
rows: `unknown-species` is a vocabulary gap, and removing it reveals what
the engine can and cannot do about the question underneath — which for
th-028 and th-070 is "not much yet". The remaining work is a sublimation
route and a sulfide species with a solid it can precipitate.

No other row moved. The full check reported exactly six drifts, no
regressions, and the expectation-mismatch count held flat at 85.


## 2026-09-05 — fourteen more rows, and two that were left alone on purpose

`BRD-014.S02` added thirteen material recipes and one alias. As with the
tranche above, no solver changed: every one of these rows had been stopping
in the parser.

Eleven reached a **computed route** — a real solver claimed the vessel:

- `bio-016` mayonnaise, `bio-017` mustard — the colloid and the emulsifier
  roles fire, and the aqueous solver speciates the salt and vinegar acid the
  recipes resolve. The **stability** in bio-016 is still a modelling choice
  and not a computation; see the recipe's own notes.
- `bio-040` jam, `bio-066` sugar water, `bio-092` red-cabbage extract,
  `bio-109` grease and soap, `bio-115` orange peel oil — solutes speciate,
  the indicator reads a pH, the detergent disperses the fat layer, the oil
  refuses to mix. All four mechanisms were already installed; the words were
  not.
- `th-047` petrol and `th-060` damp wood — combustion. Petrol resolves to
  hexane and reaches the NASA CEA equilibrium path; damp wood resolves to
  cellulose, which CEA does not carry, so `combustion.rs`'s curated fuel
  table burns it instead. The 30% water in the log is real inventory taking
  real heat, which is half the answer to "why does it burn poorly". The other
  half is smoke, and there is no pyrolysis here to make any.

Three reached `qualitative`/`typed-observation`: `bio-025` beans in a sealed
pot, `bio-101` perfume evaporating, `bio-107` hand sanitiser evaporating and
cooling. The measurement each takes is real — the sealed pressure, the
thermometer after evaporation — and the classifier files an inspection
prompt whose events are observations this way.

Two reached `computed`/`typed-engine-event`, which is the weakest evidence
the classifier accepts and should be read as "the run happened": `bio-065`
coconut fat beside vegetable oil, and `bio-105` permanent marker ink in
alcohol. Coconut fat has **no melting point on this bench** — there is no
triglyceride species — so the solid-versus-liquid difference the question is
about is a fact in a note rather than a computation.

**Two rows were deliberately not closed, and this is the important part of
the entry.** `bio-103` (activated charcoal removing a food dye) and `bio-111`
(sunscreen absorbing ultraviolet) are each one line of recipe away from
parsing. Both would then answer their question **wrongly**: there is no
adsorption model, so the charcoal would be filtered out and the dye would
still be in the beaker; and the spectral bands run 405–705 nm, so nothing can
absorb the 300 nm light bio-111 shines at it. A confident wrong answer about
what removes a dye, or about what stops ultraviolet, is worse than
`unknown-species` — the reason code at least says the bench does not know.
They stay `missing` until the models exist.

No other row moved. Fourteen drifts, no regressions, and the
expectation-mismatch count held flat at 85 again.


## 2026-09-05 — nineteen rows, and why the count is the wrong thing to quote

`BRD-014.S03` added fourteen materials and two species. `missing` falls from
59 to 41 and `computed` rises from 288 to 301, and **most of these rows did
not get an answer**. The reason codes are doing real work here and the
summary line is not, so this entry sorts them by what actually happened.

### Three rows gained a mechanism

- `bio-085` bile salts → `computed-route`. Bile salts are amphipathic
  surfactants and the bounded emulsifier role is, for once, the mechanism the
  question is about rather than a stand-in for it.
- `bio-091` alcohol extracting leaf pigment → `computed-route`. Chlorophyll
  has a reviewed ethanol solubility, so the pigment really does leave the leaf
  and enter the filtrate. What the bench does NOT do is paint the filtrate
  green: the species carries no absorption spectrum.
- `bio-051` protease and meat → `typed-engine-event`. A real hydrolysis of a
  named protein fraction runs. It is not `computed-route` because the enzyme
  model reports converted mass rather than claiming a solver route, and it is
  emphatically not tenderness: texture is collagen and this is peptide bonds.

### Eight rows run and answer nothing

`bio-086`, `bio-087`, `bio-088`, `bio-089` (photosynthesis, light intensity,
green light), `bio-096` (respiration in a sealed jar), `bio-097` (germination
in brine), `bio-098` (transpiration up a celery stalk), `bio-099` (turgor),
`bio-100` (plasmolysis).

Several of these read `computed-route`, and that is the **classifier being
right about the wrong thing**: a leaf in water with carbon dioxide dissolved
in it really does have an aqueous route, and the aqueous route really does
compute. It computes the carbonate system. It does not photosynthesise,
because nothing on this bench does. Same for the celery: the dye's solution
chemistry is computed and the dye does not move up the stalk, because there
is no stalk and no transpiration. Reading `bio-098` as "closed" would be a
mistake this file exists to prevent.

Each material's `lot_assumptions` names the missing model in capitals, so the
gap is recorded where a reader meets it rather than only here.

### Two are honest for a better reason

- `mat-025` cured thermoset → still `missing`, but the reason code changes
  from `unknown-species` to `not-yet-modeled`. **This row is not closed and
  the change is still worth making**: the bench has gone from not knowing the
  word to knowing the substance and declining to claim a softening it cannot
  compute. A cured thermoset also genuinely has no melting point — it
  decomposes — so the absence happens to be correct.
- `th-101` boiling chips → `typed-observation`. The chips are inert because
  porosity is not a property this bench has, and bumping is not modelled
  either, so there is nothing for them to prevent.

### Four more

`bio-006` cake batter and `bio-022` popcorn kernel are typed observations;
setting and bursting are both absent. `bio-090` chlorophyll is a typed
observation of a green solid, and the species' provenance says the colour is
recorded rather than derived from the two absorption bands that cause it.
`mat-037` nylon in acid and `mat-072` battery electrolyte both reach a
computed route — the acid speciates, the alkali cools — while the amide
hydrolysis and the cell reaction respectively are absent. Nylon carries no
aqueous solubility on purpose, so the engine says it does not know rather
than asserting that the polymer survives.

Nineteen drifts, no regressions, and the expectation-mismatch count is 84.


## 2026-09-05, sixth — two rows, and the last `unknown-reaction` in the corpus

`bio-064` (alcohol to vinegar) and `bio-080` (glucose plus oxygen) were the
only two prompts in all five hundred that stopped at
`ParseErrorKind::UnknownReaction`: they asked the `react` verb for a name
`curated::ORG_REACTIONS` did not carry. Both names are now on that shelf, so
both scripts run end to end and both rows leave `missing`/`unknown-reaction`.

**What they gained is an equation, not a prediction**, and the verb's own
boundary text now says that in as many words: asking for a named reaction is
the LEARNER requesting an outcome, and nothing in the bench decides that
ethanol standing in air will acetify or that glucose in a beaker will
respire. That is the whole reason these two live behind a verb rather than
behind a solver.

- `bio-064` → `alcohol-oxidation`, C2H5OH + O2 → CH3COOH + H2O. Deliberately
  the SAME equation the fermentation lane's `food/acetic-acid-bacteria`
  culture runs (#412, `CultureMetabolism::Acetic`), cited to that route
  rather than restated: two paths to vinegar that disagreed about its
  stoichiometry would be worse than one path. No organism, no rate, no
  acetaldehyde intermediate, no oxygen drawn from room air, and the oxygen
  has to be in the vessel already.
- `bio-080` → `respiration`, C6H12O6 + 6 O2 → 6 CO2 + 6 H2O. Glycolysis, the
  citric acid cycle and oxidative phosphorylation collapsed into one line.
  **There is still no cell on this bench**, and the entry three sections
  above — where `bio-096`'s respiration in a sealed jar runs and answers
  nothing — is not superseded by this one. The standard enthalpy of
  combustion of glucose, about −2803 kJ/mol as commonly tabulated with its
  provenance lane pending review, is quoted in the row's boundary and **is
  not applied**: no row in that table carries a curated reaction enthalpy, so
  the vessel's temperature does not move.

Both land on `computed`/`computed-route`, and the reason code is worth
reading carefully rather than banking. **The computed route is the aqueous
chemistry of the PRODUCTS, not the reaction.** `bio-064` makes acetic acid
and water, and minteq.v4 can speciate acetate, so a real solve follows the
verb; `bio-080` makes carbon dioxide and water, and the carbonate system
solves. The curated verb is what answered the question; the solver is what
happened next. This is the same shape as `bio-063` and unlike `bio-062`,
whose ester and water give the aqueous tail nothing to work on and which
therefore records the weaker `typed-engine-event`.

Neither row carries a SMIRKS template. `kerotakis-org` has templates for the
ester pair only, and the differential test in
`crates/kerotakis-org/tests/template_oracle.rs` proves those two rows by
name; it is not a totality check and these two are outside it. What proves
these is the atom and mass balance against the registry formulas, which the
`react` verb's own conservation test exercises — a weaker check, and this
paragraph is where that is admitted rather than hidden.

One test moved with them. `crates/kerotakis-codex/tests/curiosity_corpus.rs`
required the corpus to contain a prompt declaring each typed parser boundary,
and closing both rows left `UnknownReaction` with none. The requirement is
now asserted against the parser directly (`react v1 transmutation` must still
fail typed) rather than satisfied by leaving a row permanently broken to feed
it; keeping the row would have been the corpus serving the test.

Two drifts, no regressions.


## 2026-09-05, eighth — one row, a rate rather than a reaction

`bio-112` ("why does hydrogen peroxide bleach hair pigments?") leaves
`missing`/`unknown-species` for `qualitative`/`typed-observation`. Two
species and one rate law close it: `hair_pigment`, a 5,6-dihydroxyindole
repeat unit standing for eumelanin, `hair_pigment_ox`, its colourless
oxidised twin, and `peroxide-melanin-bleach` in the curated kinetic network.

**It is deliberately a rate law and not a curated reaction.** A curated
reaction fires inside the solver stack at the end of the step that completes
its reactant set, so it would have consumed every mole of pigment on the line
that ADDS the peroxide, and `wait 10min` would then have advanced a clock
over a finished reaction. The prompt asks why peroxide bleaches *over time*;
only a rate can answer that, and the script's own `wait` is what asks.

**The rate is an editorial classroom timescale and is not measured.** No
citable rate constant for the oxidation of melanin by hydrogen peroxide was
found, so the pre-exponential is chosen so the colour is mostly gone after
ten minutes and plainly present after one — the same calibration the
`peroxide-decomposition` and `iron-corrosion` entries admit to. The entry's
`uncertainty` note is where that is stated, and it is the one field that
would have to change if a real constant were ever installed. There is no pH
term either: salon bleaching is peroxide plus ammonia, and here a developer
and a neutral bottle bleach at the same speed.

**Both colours are recorded, not computed.** Neither species carries an
absorption spectrum, so nothing derives melanin's broadband absorbance from
anything. What changes in `look` is which SOLID the beaker names — a black
one at one minute, an off-white one at ten — and the threshold that flips the
sentence is the renderer's own rule that a settled solid is named while it is
at least a tenth of the largest heap.

The disposition is `qualitative`/`typed-observation` rather than `computed`,
and the reason is worth stating plainly: the event the classifier files on is
the bench saying the pigment does not dissolve, not the bleach. A kinetic
`Reacted` is not a solver route and is not on the classifier's answering
list, so the reason code credits the remark. The chemistry that answers the
question is the rate law.

**One stale sentence, and it is not fixed here.** The honesty pass in
`solve.rs` asks `curated::consumes` whether anything is eating an insoluble
solid before it remarks on the solid not dissolving — a guard added for
exactly this reason when starch was being digested by amylase — and it does
not ask the kinetic network. So a ten-minute transcript reads
"peroxide-melanin-bleach ran" and then "hair pigment does not dissolve in
water … It is still all there", and the second clause is stale by then. The
first half of that sentence is true and always will be; the trailing clause
is the defect. It is recorded in the species' own provenance where a reader
meets it, and fixing it is a change to `solve.rs`, which this lane did not
touch.

One drift, no regressions.


## 2026-09-05, tenth — the row that was left open twice

`bio-103` ("can activated charcoal remove a food dye from water?") is closed:
`missing`/`unknown-species` becomes `computed`/`computed-route`. It had been
left open deliberately in two earlier tranches, and the entry above dated
2026-09-05 (the fourteen-row materials refresh) states the reason in the
words this one has to answer to:

> Both would then answer their question **wrongly**: there is no adsorption
> model, so the charcoal would be filtered out and the dye would still be in
> the beaker … A confident wrong answer about what removes a dye … is worse
> than `unknown-species`.

So the species was not the fix and was never added alone.
`crates/kerotakis-core/src/adsorption.rs` carries one curated **Langmuir
isotherm** for methyl orange on activated charcoal, and what it binds lives
in a new `Vessel::adsorbed` ledger outside `contents`. That placement is the
mechanism, not an implementation detail: `filter` rewrites `contents` and
touches nothing else, so what is held on the retained carbon is retained with
it, and the filtrate carries only what was still dissolved. `Vessel::mass`
weighs the bound amount, so nothing disappears.

**The answer is partial, and the event says both halves.** One gram of carbon
at a 200 mg/g monolayer cannot hold the 327 mg of dye this script pours on
it: about three fifths comes out of solution and two fifths stays. "The
charcoal adsorbed the dye" is the sentence a demonstration wants and is the
one that misleads, so `Event::Adsorbed` reports the loading in mg/g beside
the moles still dissolved and neither can be read without the other.

**Both curated parameters are pending review.** Published monolayer
capacities for this pair spread over most of an order of magnitude with the
carbon's activation and the solution pH; 200 mg/g is a mid-range
laboratory-grade value recorded as commonly tabulated, and the affinity is
chosen so the knee of the isotherm sits below classroom concentrations.
Neither is transcribed from a positively identified paper and neither claims
to be.

**What `look v2` still cannot show, and this is the part to read before
quoting the closure.** Methyl orange's spectrum is pH-dependent, this vessel
has no electrolyte, and no shipped database speciates an azo dye — so no
solution is characterised, the renderer has no pH to pick a spectrum with,
and **the filtrate reads "colourless" whether or not the carbon did
anything**. The row is answered by the adsorption event and the ledger, not
by the colour word its script asks for. A dye that keeps its colour in an
uncharacterised solution is a separate change and is not made here.

The reason code is `computed-route` rather than `typed-observation`, and one
classifier rule moved to make that true. The honesty pass truly says the
carbon does not dissolve, and that remark used to file the row as a typed
observation and hide the isotherm behind it. The existing "an aside is extra,
not instead" rule — written for a plated metal beside a true remark about
overpotential — now also lifts an `Inert` that stands beside an `Adsorbed`.
It is the same argument: the remark is about the carbon's solubility and the
answer is about the dye's.

One drift, no regressions.

# Perturbation cases generated from the corpus scripts

`crates/kerotakis-cli/tests/corpus_perturbation.rs` is the generator. This
file is the argument for it, the measurements the thresholds were set from,
and the honest count of how far it reaches.

## Why the corpus needed this

The 500 prompts here carry a `question`, a `script` and — for 307 of them —
an `expected`. The `expected` vocabulary is
`computed | curated | qualitative | boundary`: those are four **routes**,
not four outcomes. What a green row asserts is that the engine answered by
the route somebody intended. It asserts nothing about the chemistry.

`aq-003` is the standing illustration. The question asks whether a beaker
*cools*; the script measures a thermometer; nothing anywhere records that
the temperature must fall. An engine reporting forty degrees of warming
keeps that row green forever.

What survives is the **scripts** — reagents, quantities and an instrument,
which is the expensive half of a test case and is already paid for. The
generator spends it on the one question that needs no reference value, no
licence and no external data: *does this number move, in the right
direction, when its cause moves?*

## How the generator chooses a perturbation

Each script is parsed into steps — `add <vessel> <species> <amount><unit>
[@ <T>C]` structurally, everything else verbatim — and offered to five
rules. A rule is admissible when the script's *shape* supports it and when
applying it actually produces a different script. A prompt may admit
several; the census counts each.

| rule | perturbation | claim | shape |
|---|---|---|---|
| `Order` | reverse a contiguous run of adds to one vessel | every final quantity is unchanged | invariance |
| `Scale` | double every quantity the script states | intensive unchanged, extensive doubled | invariance + direction |
| `Solvent` | double the solvent only | solute amounts unchanged, ionic strength falls | invariance + direction |
| `Ablation` | delete the last non-solvent reagent | the instrument the script picked up must move | causality |
| `Dose` | double the last non-solvent reagent | the instrument must move | direction |
| `Sibling` | run an authored dose-sibling | two loadings, two answers, ordered | **differential** |

The ordering is the argument. An **invariance** survives an engine
improvement and cannot be satisfied by a constant. A **causal** claim is the
direct cure for what is wrong with this corpus. A **directional** claim is
the weakest, because a quantity the engine multiplies through always moves.
A **differential** claim is the strongest, because a global error moves both
systems equally and cannot fake a ratio between their responses.

### Two decisions that stop a generated case being vacuous

**The baseline run is the corpus script, byte for byte.** The generator
re-renders the baseline from its own parse. If that render were not
identical to the authored line, the file would be measuring something nobody
wrote. All 943 `add` lines round-trip exactly, and
`the_generator_renders_the_corpus_back_exactly_as_written` keeps it so.

**The causal rules read the readout, not the inventory.** The first draft
compared the whole bench, which deleting a reagent satisfies trivially by
the reagent no longer being in it — the corpus's own disease in a new
costume. Keys now carry a prefix: `r.` for a readout (instrument events from
the step stream, the derived properties of the solution, the scene's words
and verdicts), `n.` for an amount, `m.` for a molality. `Ablation` and
`Dose` read only `r.`, and where the script picked up an instrument, *that
instrument's own reading* is what must move. `aq-003` measures a
thermometer; a bench reporting the room's temperature whatever is in the
beaker fails, which is exactly what `expected = "computed"` could not say.

### What the generator refuses to read

`contents["base_equivalents"]` is the solution's residual cation charge: the
base half of the analytical acid/base equivalents the aqueous tail books to
close a solved vessel's H/O balance. It was published as `contents["OH-"]`
until 2026-09-16 — recorded in `perturbation.rs` 7/7, measured 1.1e6 above
the hydroxide an acetate buffer's pH can hold — and the rename is all that
changed; the quantity is the same bookkeeping coordinate it always was. Two
rules would otherwise rest on it. `trustworthy()` excludes it by name from
any claim of the form "something moved", so no generated case can be
satisfied by a coordinate moving. `Order`, whose claim is that a number came
back the same, keeps it: an invariance over a coordinate is still a true
statement about path independence.

The exclusion is a substring match, so under the old name one entry also
reached every other key spelling `[OH-]` — the reported molality, the
particle census label. `OH-` is therefore still listed beside the new name:
narrowing the filter would widen what a generated case may rest on and move
the pinned `ABLATION_INERT` set, which is a measured change rather than part
of a rename.

## What the corpus admits, and what it does not

Computed by `the_corpus_census_is_what_is_recorded` over all five hundred
prompts and checked in as `perturbation-census.json`, so a corpus edit that
puts a script beyond the generator's reach shows up as a diff with a number
on it rather than as silence.

| | prompts |
|---|---|
| admit at least one perturbation | **430** |
| admit none | **70** |

The seventy that admit none, with the reason:

| reason | prompts |
|---|---|
| no script: an explicit product boundary, nothing to run | 60 |
| no reagent: the script only observes or manipulates | 4 |
| a single reagent, no water, and a size-bound verb: nothing can be held fixed against the perturbation | 5 |
| *(the sixtieth boundary row counts once here)* | 1 |

The sixty are the `boundary` rows. They are not a failure of the generator:
a boundary row records a deliberate product refusal and has no script by
design, so there is nothing to perturb and nothing that should be.

**So the honest answer to "does this scale" is 430 of 500, and the 70 that
do not are 60 refusals plus 10 scripts too thin to hold anything fixed.**

By rule:

| rule | scripts admitting |
|---|---|
| dose | 398 |
| scale | 365 |
| ablation | 332 |
| solvent | 233 |
| order | 174 |

`order` reaches fewest because it needs two reagents to permute; `solvent`
needs water specifically. `dose` reaches most and is worth least.

## What the generator found

### `solution.pe` is path-dependent by 12.8 units where nothing constrains it

> **FIXED 2026-09-17.** The guard existed and asked the wrong question —
> whether any element *could* carry more than one oxidation state in the
> dataset, which carbon can — so a fizzing bicarbonate solution satisfied
> it. It now also requires that some element actually **is** split. `th-100`
> reports no pe in either order; `FeCl3` still reports 18.65875 against
> 18.65716 across the same reordering, four significant figures apart.
> Withdrawn from `ORDER_DEPARTURES`;
> `a_pe_no_couple_constrains_is_not_published` pins both halves. The account
> below is left as written, because it is how the defect was found.


`th-100` — "will a smaller headspace reach more pressure from the same
generated gas?" — is bicarbonate, hydrochloric acid, a sealed 200 mL
headspace and a pressure gauge. Swap the two reagents and:

```text
                          pH        pe       ionic strength     pressure
  NaHCO3, then HCl     5.555778   12.780243     0.497017        101324.90
  HCl, then NaHCO3     5.555338   -0.055944     0.497017        101324.99
```

pH agrees to four decimal places, ionic strength to six, the pressure gauge
to a part in a million — and `pe` differs by 12.84, which is about 760 mV of
redox potential. Neither vessel contains a redox couple: `solution.redox` is
`[]` in both. The number is unconstrained, the solver returns whatever its
path left behind, and the `--json` contract publishes it as the vessel's pe
with nothing to say it means nothing.

This is the same shape as the `contents["OH-"]` defect fixed on 2026-09-16
and it was found the same way: a published quantity that does not mean what its name
says, invisible to any single run because −0.06 and 12.78 are both perfectly
plausible numbers, and obvious the moment its cause is perturbed.

### `solution.solvent_kg` carries an order-dependent residue that `contents[water]` does not

`aq-023` — calcium chloride and powdered detergent in 100 mL of water. Swap
the reagents:

```text
                       contents[water|liquid]    solution.solvent_kg
  CaCl2 first            5.5339424445 mol           0.0997010580 kg
  detergent first        5.5339424570 mol           0.0996909590 kg
```

The inventory agrees to one part in 4e8. The solvent mass the solution is
characterised against disagrees by one part in 1e4 — about 10 mg of water in
100 g. Every molality is divided by that number, so the discrepancy
propagates into the whole speciation, where the wire's four significant
figures hide it. Two surfaces of the same quantity, disagreeing; the
generator compares them because it compares everything.

### Order dependence in an open beaker is chemistry, and the test now proves it

The first two order departures were `mat-086` (limewater and carbon dioxide)
and `mat-124` (bicarbonate and vinegar), both open beakers, both carbonate.
`mat-086` measured:

```text
                        pH      CaCO3 (mol)   Ca(OH)2 left (mol)
  lime, then CO2      9.898       0.009988         none
  CO2, then lime     12.452       0.003394         0.004605
```

Two and a half pH units and a factor of three in the precipitate — a
different answer to "can carbon dioxide turn limewater cloudy?". It is also
correct: an open beaker is not a closed system, and the CO₂ poured in first
has somewhere to go before the lime arrives. **Seal the same vessel and the
two orders agree** — pH 9.109228 against 9.108648, precipitate to one part in
1e9.

`closing_the_vessel_restores_order_independence` asserts exactly that, and it
is the sharpest shape in the file: the same perturbation into two systems,
with responses that must differ by a stated factor. It converts an excuse
into a claim. If the engine ever grows a genuine path memory, this separates
it from the open-vessel effect rather than letting the atmosphere take the
blame.

### `aq-018` cannot answer its own question, and the corpus already knew

"Can a spoonful of sugar disappear into water?" — `water 100mL; sucrose 10g;
stir`. Delete the sucrose and **not one readout moves**: not pH, not ionic
strength, not the temperature, not the words the scene shows. The bench
cannot tell sugar-water from water on any surface it exposes.

The row is tagged `substance-gap`, so this is a known limitation rather than
a discovery — but it is a *smoke* prompt, `expected = "computed"`, and it has
been green for as long as the corpus has existed. The route it was answered
by is the thing the corpus checks. Whether the answer depends on the sugar is
the thing it could not.

## What each generated case establishes, and what it cannot

Every case carries this in its own doc comment; the summary is here so the
weak ones cannot be quietly counted as coverage.

| case | establishes | cannot establish | weakness |
|---|---|---|---|
| `addition_order_does_not_change_where_the_corpus_ends_up` | the state a recipe reaches does not depend on the sequence it was assembled in, across the span of chemistry the corpus covers rather than one precipitation | that the state is right — both orders could be equally wrong | **low.** The two runs are the same arithmetic in a different sequence; a solver cannot pass by construction |
| `doubling_the_corpus_leaves_every_intensive_property_alone` | no absolute size leaks into a quantity that describes the solution rather than the beaker | any value; it is homogeneity of degree zero and one and nothing else | **low**, at the cost of excluding scripts with a verb whose size the script does not state |
| `twice_the_solvent_dilutes_the_corpus_without_moving_the_amounts` | the solvent is a real quantity intensive properties are computed against, and the solubility limit is live | any value, any functional form, or which branch a given beaker should have taken | **low.** No path satisfies it without recomputing from the solvent mass |
| `the_corpus_answer_depends_on_the_reagent_the_question_is_about` | the observation is causally downstream of the reagent — and where the script picked up an instrument, *that* reading is what must move | direction, magnitude, correctness. A beaker that warmed when it should have cooled passes | **the perturbation is not weak; the claim is.** `≠` is the least you can ask. The rows that FAIL are the valuable output |
| `twice_the_reagent_moves_the_corpus_answer` | a dose response exists | anything else | **high, and declared.** Doubling a solute doubles its moles, doubles its molality, moves the ionic strength: that chain is arithmetic and this rides it. Kept only for the rows where the chain breaks — saturation, excess, a limiting reagent — and sampled at a fifth the density of the rest |
| `authored_dose_siblings_are_not_answered_alike` | the engine distinguishes two loadings of the same recipe, on the surface the script's own instrument reads | which answer is right, or that the difference has the right size or sign | **low as a shape, moderate as a claim.** Differential, so a constant or an input-ignoring engine cannot pass; still only `≠`, except where a group has three loadings and the responses must be ordered |
| `closing_the_vessel_restores_order_independence` | path dependence here is attributable to the boundary rather than to memory in the solver | that the open-vessel answer is right, or that the amount lost to the room is right | **low.** A ratio between two responses; a global error moves both |

### The one that is close to the path it tests

`twice_the_reagent_moves_the_corpus_answer` is the weak one and says so in
its own doc comment, for the reason the brief warns about: a perturbation
that varies a quantity the code multiplies through always passes. It is
generated because the rows it *cannot* move are informative — a saturated
beaker, a reagent already in excess, a limiting-reagent situation where the
second half does nothing — and those are precisely the rows an `expected`
field recording a route could never distinguish.

`the_corpus_answer_depends_on_the_reagent_the_question_is_about` was nearly
in the same position and was fixed rather than excused. Its first draft
compared the whole bench, which deleting a reagent satisfies trivially by the
reagent no longer being in the inventory. Restricting it to the readout, and
then to the script's own instrument where there is one, is what makes it a
claim about the chemistry rather than about bookkeeping.

## Reach and cost, if this were continued to all five hundred

The measurement harness already reaches the whole corpus:

```sh
KERO_PERTURBATION_SWEEP=/tmp/sweep.jsonl KERO_PERTURBATION_ALL=1 \
  cargo test -p kerotakis-cli --test corpus_perturbation -- --ignored --nocapture sweep
```

What it costs is one `kero run --json` per side of each case. Measured on
this machine, with two other agents building, a single run takes 2.1 s at
rest and 3.5–5 s under contention; the corpus's slowest scripts (an hour of
simulated waiting, a combustion) reach 19 s for the pair.

| | cases | runs | serial time at 4 s/run |
|---|---|---|---|
| the gate (the subset below) | ~75 | ~150 | ~10 min |
| every rule on every script it admits | 1502 | 3004 | ~3.3 h |

So the full corpus is a nightly job, not a gate, and the generator is built
for both: `density()` picks the gate's sample by a hash of the prompt id —
stable under corpus insertion, spread across all four shards, and
independent of anything the generator computed — while
`KERO_PERTURBATION_ALL=1` ignores it.

**The marginal cost of extending the reach is zero engineering and linear
machine time.** Nothing in the generator is per-prompt: there is no list of
handled ids, no per-script annotation, and no reference value anywhere. The
1502 cases already exist; only the CI budget decides how many of them run on
a pull request.

What would NOT scale for free is the departure lists. Each recorded row
needs a human sentence saying why it departs, and the ones found so far took
between two runs (`aq-107`, saturated) and a small investigation
(`th-100`'s pe). At the rate observed on the subset — 6 departures in ~75
cases — the full corpus would produce something like 120 rows to explain.
That, not machine time, is the real cost of going to five hundred.

## Proving the cases are not vacuous: mutate the ENGINE

Mutating a test shows that an assertion is live. Mutating the engine shows
whether it is aimed at code that could break. Only the second caught the gap
in the hand-written suite — a deliberately broken engine left the scene
saying an object floats beside a sentence saying it sank, and the first draft
of that test, which read only the scene, passed.

Each mutation below is a plausible bug, not a syntactic one: something a
person could write and a reviewer could miss.

### 1. The solvent mass becomes a constant kilogram

`crates/kerotakis-phreeqc/src/aqueous.rs`, in the partition: replace the
accumulated `kgw` with `1.0`. Every molality is then computed against a
kilogram that is not the beaker's. A single run still looks entirely
plausible — a 0.1 molal solution reads 0.1 molal, because the script happened
to use 100 mL.

**Caught, loudly.** 10 of 19 generated `Scale` cases and the whole `Solvent`
case go red:

```text
aq-032: n.v0.solvent_kg: expected 1.999998e0, got 9.999989e-1 (5.00e-1)
aq-021: n.v0.free_proton: expected 2.012122e-7, got 1.006061e-7 (5.00e-1)
bio-110: m.v0[Ca+2]: expected 1.000000e-2, got 2.000000e-2 (5.00e-1)
th-099: r.v0.pe: expected 1.278721e1, got -2.305407e-1 (1.02e0)
bio-005: m.v0[NaCO3-]: expected 2.617000e-3, got 7.996000e-3 (6.73e-1)
...
```

The signature is the giveaway and is worth reading: the departures sit at
exactly 0.500 relative where a quantity failed to double or halve, and
scatter above that where speciation then re-solved against the wrong mass.

### 2. The thermometer reports the room instead of the beaker

`crates/kerotakis-core/src/bench.rs`, in `Operator::Measure`: replace
`value: v.temperature.to_celsius()` with `value: 25.0`. Every other surface
stays right — the vessel's temperature is still computed and still correct in
`bench.vessels[].temperature`; only the instrument the script picked up lies.

This is the mutation the first draft of the ablation rule would have
survived, and it is why that rule was changed. Comparing the whole bench,
deleting a reagent still moves the inventory and the pH, so "something moved"
is satisfied and the thermometer's lie goes unnoticed. That is precisely the
shape the hand-written suite's buoyancy case was caught by: two surfaces that
could disagree, and a test that read only one of them.

**Caught — and caught by `aq-003`.** The row this whole exercise is named
after; the row whose question is whether a beaker *cools* and whose script
measures a thermometer; the row that has been green for the life of the
corpus while nothing anywhere recorded what the temperature should do:

```text
1 of 30 generated ablation cases departed with no recorded reason:
  aq-003: the instrument the script picked up did not move:
          r.thermometer#0=25.000000
```

Delete the potassium chloride and the thermometer reads 25.000000 °C.
Leave it in and the thermometer reads 25.000000 °C. On the unmutated engine
it reads 19.47 °C with the salt and 25.01 °C without it, and the case
passes.

The corpus row is still green under the mutation: it still takes a
`computed` route and a thermometer still answers it. `expected` cannot see
the difference, which is the entire argument for this file.

`twice_the_reagent_moves_the_corpus_answer` and
`authored_dose_siblings_are_not_answered_alike` both still pass under this
mutation, and that is worth saying rather than hiding: the dose subset's
only thermometer row is `th-122`, already recorded as inert for correct
physical reasons, and the sibling comparison reads the whole bench rather
than the instrument. A mutation caught by one case and missed by two is the
normal result, and it is why there is more than one case.

### What is NOT proven by mutation, and why

`Order` has no mutation here. It did not need one: it found two real
departures on the *unmutated* engine — `th-100`'s 12.84 units of pe (fixed
2026-09-17) and `aq-023`'s solvent mass — which is stronger evidence that it
is aimed at live code than any injected bug would be. The `th-100` half has
now been through the whole cycle: found by the rule, explained, fixed, and
pinned by a test that also asserts the number it must NOT withhold. `closing_the_vessel_restores_order_independence`
is in the same position: it exists because the generated `Order` rule failed
on two open-vessel carbonate rows, and it asserts the explanation.

A third mutation was written and not run, for machine time: each
`kerotakis-core` mutation costs a thirteen-minute rebuild. It is recorded
here so the next person does not have to invent it. In
`Vessel::heat_capacity_at`, replace the sum over portions with a
representative portion scaled by the total:

```rust
let total: f64 = self.contents.iter().map(|portion| portion.moles.0).sum();
let representative = self
    .contents
    .first()
    .and_then(|portion| species::lookup(&portion.species).map(|d| (portion, d)))
    .map(|(portion, data)| crate::states::heat_capacity_at(data, portion.phase, t_k))
    .unwrap_or(0.0);
representative * total + crate::plastics::unresolved_heat_capacity(self)
```

Which portion is "first" depends on the order things were added, so the
vessel acquires a memory of its own assembly — plausible in any single run,
and exactly what `Order` is pointed at. Reproducing the other two is a
one-line edit each: `kgw += …` becomes `kgw = 1.0` in
`kerotakis-phreeqc/src/aqueous.rs`, and `value: v.temperature.to_celsius()`
becomes `value: 25.0` in `kerotakis-core/src/bench.rs`.

## The measured gate

Seventy-seven generated cases in the sweep, 561 s of solver; the gate as
checked in runs ~75 of them and takes **277 s wall at three test threads**,
on a machine with two other agents working. Final run: **9 passed, 0 failed,
2 ignored** — the two `#[ignore]`s being the recorded `aq-061` seal defect
and the `sweep` harness. **The `aq-061` half was withdrawn on 2026-09-17:
the bottle bursts (6.959 bar against a 405.3 kPa rating) and the engine was
right; see `terminal_event` and
`a_bottle_that_cannot_hold_the_gas_bursts`.**

| rule | cases | held | recorded departures |
|---|---|---|---|
| order | 11 | 9 | 2 |
| scale | 19 | 15 | 4 |
| solvent | 9 | 7 | 2 |
| ablation | 30 | 29 | 1 |
| dose | 8 | 5 | 3 |
| sibling groups | 13 | 11 | 2 |

**29 of 30 ablation cases hold.** That is the headline number: for all but
one of the corpus scripts the rule reached, the instrument the script picked
up does depend on the reagent the question is about. Before this branch,
nothing anywhere asserted that for any of them.

The 14 recorded departures break down as: **2 live defects** (`th-100`'s pe,
`aq-023`'s solvent mass) plus **1 more found by `Dose`** (`aq-061`'s seal —
**withdrawn 2026-09-17, it was a burst and the engine was right**); **4 solver or wire floors**
(two phase boundaries, one last-printed-place, one convergence residue);
**5 corpus rows whose scripts cannot reach their own questions** (`aq-018`
and `bio-033`'s substance gaps, `mat-069`'s copper that never corrodes,
`bio-029`'s uncharacterised enzymes, and the two combustion sibling groups
that answer `not_yet_modeled` at both loadings); **1 correct physics the rule
does not apply to** (`th-122`: self-heating raises temperature by an amount
independent of mass); and **1 that is defect-shaped and unexplained**
(`bio-070`'s fermentation extent going as the square of the batch size).

### `bio-070` — resolved 2026-09-16, and what was underneath it

The square-of-the-batch departure was a real defect and is fixed.
`kerotakis-core/src/fermentation.rs` multiplied a culture's declared rate by
the GRAMS of culture with nothing in the denominator, so doubling a batch
doubled the first-order rate as well as the substrate it was spent on. The
total acid went 2.716e-5 → 1.086e-4 mol, a factor of **3.999060**, against
`2(1−e^−2x)/(1−e^−x)` = **3.999060** at this script's `x = kt = 4.6989e-4`.

The measured departure was 4.2607 rather than 3.9991, and the extra 6.5% was
the **observable**, not a second defect. `n.v0[lactic_acid|aqueous]` is the
undissociated acid alone — 1.67e-3 of the total at milk's pH — and four
times the acid in twice the water drops the pH by 0.0275, raising the
undissociated share by `10^0.0275 = 1.0653`. `3.999060 × 1.0653 = 4.2601`.
Under an extensive rate the two runs share a pH and the readout doubles
exactly, which it now does.

The rate now reads a concentration against a declared one-litre reference
volume, and the acid is extensive to the last bit: 3.014833e-4 → 6.029666e-4.

**The row still departs**, at 2.9e-2, on the vessel's reported CO2 partial
pressure and the bicarbonate species with it — and that departure predates
this fix and was simply two hundred times smaller than the one on top of it.
Milk poured at 5 °C and left eight hours in an OPEN vessel warms to the room
while trading carbon dioxide with the atmospheric reservoir, and
`add v1 milk 100mL @ 5C; wait 8h` **with no culture in it at all** departs by
2.89e-2 on exactly those keys. It belongs with `bio-055`: what an open vessel
exchanges with the room does not double when the beaker does.

## Where the generator was wrong, and how it found out

Worth recording, because three of the five rules as first written made false
claims, and in every case the corpus refuted them faster than review would
have:

* `Solvent` held every solute amount fixed under dilution. Refuted in three
  of its first seven rows: species amounts are not conserved under dilution,
  element totals are, and the wire does not expose element totals for a
  named material. Replaced by a disjunction.
* `Solvent` then doubled whichever liquid it found, and `bio-017` doubled
  its vegetable oil and correctly left the vinegar's ionic strength alone.
  Now it doubles water.
* `Order` reversed whole runs of additions, which put the water last — a
  reagent poured into an empty vessel is a different experiment. The
  whole-run version agreed to 1.3% on a limewater script whose two REAGENT
  orders differ by 2.55 pH units.
* `Ablation` compared the whole bench, which deleting a reagent satisfies by
  the reagent no longer being in it.
* `free_proton` was filed as a readout when it is moles. Every scale case
  failed at exactly 0.5 relative, which is what an extensive quantity looks
  like when asked to hold still — a classifier's mistake has a signature a
  chemistry bug does not.
* The observer read `steps.last()` for the bench, and `particles v1` emits a
  step with no bench in it. `aq-049` came back with zero readouts and an
  ablation case that could not fail.

A generator's own classification is as capable of being wrong as the engine
it tests. What makes it tractable is that the two fail differently: a
classifier error fails every case of its kind by the same exact factor, and
a chemistry defect fails one row by an amount nobody predicted.

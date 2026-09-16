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

`contents["OH-"]` is the solution's residual cation charge wearing
hydroxide's name — recorded in `perturbation.rs` 7/7, measured 1.1e6 high in
an acetate buffer, unfixed. Two rules would otherwise have rested on it.
`trustworthy()` excludes it by name from any claim of the form "something
moved", so no generated case can be satisfied by that defect moving.
`Order`, whose claim is that a number came back the same, keeps it: an
invariance over a wrong number is still a true statement about path
independence.

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

This is the same shape as the recorded `contents["OH-"]` defect and it was
found the same way: a published quantity that does not mean what its name
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


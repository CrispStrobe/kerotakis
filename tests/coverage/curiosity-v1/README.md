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

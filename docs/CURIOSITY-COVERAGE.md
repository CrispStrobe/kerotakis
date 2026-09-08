# Curiosity coverage: what the 500 rows actually say

A reading of the curiosity-v1 corpus as it stands on `main` today, written to
answer one question the owner keeps having to ask by hand: **do we cover more
now, and if not, what would it take?**

This is a companion to `tests/coverage/curiosity-v1/README.md`, not a
replacement. That file is the refresh *log* — 2000 lines, newest first, one
entry per baseline change, and it is where the reasoning for any individual row
lives. This file is the standing *analysis*: the shape of the remaining tail,
which side of each unmet requirement is wrong, and the order to work them in.
When the two disagree, the log is the record of what happened and this page is
the argument about what to do next; fix the argument here.

## What the count means, since 2026-09-08

`expected` is a **floor**, not an equality. The field has been declared a
requirement on the engine since #341, and a requirement is met when it is
exceeded: a row required to hand-wave that answers with a Gibbs minimisation has
over-met it, and is not a finding. `computed` and `curated` are **one grade**
when a requirement is checked, because they are ordered by provenance rather
than by quality. `boundary` is **off the scale** and compares for equality only.

The metric is therefore `unmet requirements`, and there is no second count. The
old `expectation mismatches`, which required equality on all four grades, is
gone — not reported beside this one — so any number quoted from before
2026-09-08 is under a definition that no longer exists. `coverage.rs`'s
`meets_requirement` is where the rule and its reasoning live, including the two
parts that are easy to get wrong:

- **Why `boundary` is not ranked.** It is produced not only by the
  declared-boundary short circuit but by any `Event::SafetyVeto`. If it sat
  above `qualitative`, an over-eager veto that swallowed an ordinary
  dissolution question would satisfy that row's floor and pass in silence.
  That is the worst regression this bench can have, so no ordering may bless
  it.
- **Where the merge stops.** It is in the comparator only. `by_observed` still
  counts `computed` and `curated` separately and `baseline.toml` still records
  which one each row took. #511 moved twenty rows between those two grades and
  was reviewable *because* the baseline records the distinction; merged into
  the baseline it would have been a zero-diff PR.

**This count is a backlog, not a regression gate.** A floor is blind in one
direction — a row that requires `qualitative` and computes today still meets its
floor after falling back to `qualitative`. `baseline.toml` is what catches that,
per row and at higher fidelity than any grade: it records the exact outcome
*and* the exact reason code for all 500 rows, and `--check` fails on either
moving, on every PR. No separate ratchet on the grade is needed, and one would
only duplicate a subset of that at lower fidelity. Read the two lines together:
`unmet requirements` says what work is left, `baseline drift` says whether
anything moved.

## How these numbers were obtained

Built `kerotakis-cli` from `origin/main` at `22aba0e9` plus the reviewed
liquid-liquid extraction tranche and ran
the report without `--check`, which prints the counts and the split that the CI
gate only ever reduces to an exit code:

```sh
cargo run -p kerotakis-cli -- coverage curiosity
```

The individual transcripts quoted below are from running the row's own `script`
through `kero`. Nothing here is read off the recorded baseline; the baseline is
quoted only where it agrees, which it does, exactly.

## The fresh numbers

```
curiosity curiosity-v1: 500 prompts
  computed     314
  curated      43
  qualitative  82
  boundary     60
  missing      1
  unmet requirements: 21
    engine stood aside (corpus claimed it): 0
    answered below the required grade:     21
  solver/runtime failures: 0
  baseline drift: 0
```

**`baseline drift: 0`** is the load-bearing line. The recorded route is still
the route, on every one of the 500 rows, so the gate is measuring the engine
and not a stale file.

439 of 500 rows produce an answer; 60 more are deliberate refusals that are
correct to refuse (weaponisation, medical advice, fracture mechanics the bench
does not model), each with its own reason code. One row produces nothing. On
the only reading of "covered" that is not self-flattering — *the bench either
answers or says precisely why it will not* — that is **499 of 500**.

## Did coverage move?

Before this tranche, `baseline.toml` was last touched by `57466555`, inside
**#511** (per-liquid plateaus), which moved twenty rows from `computed`/`computed-route` to
`curated`/`curated-route`. Not one of those twenty gained or lost an answer:
`qualitative` stayed at 82 and `boundary` at 60. What changed is that
`PhaseRouteEquilibrator` now succeeds where it used to decline, and it declares
itself a curated-kind solver, so the classifier files those vessels under the
curated branch instead of letting them fall through to the computed one. Under
the grade defined above that relabelling is invisible to the count, which is
the point of merging the two.

The liquid-liquid extraction tranche intentionally moves exactly one row:
`aq-085` from `missing/not-yet-modeled` to `computed/computed-route`. Its script
now asks the same-total-solvent comparison, and the typed extraction event
reports both the repeated-stage result and the one-stage control. Removing that
single reviewed baseline change leaves every other row byte-for-byte stable;
`--check` reports zero solver/runtime failures and zero drift.

## The 21: one classification question and two named gaps

An unmet requirement is a row whose observed grade is below the floor its
`expected` declares. It is not a defect count and never was, but it is now a
short enough list to read row by row, which is the whole reason for the
definition. Grouped by shape:

| required → observed | n | what the evidence says |
|---|---|---|
| computed → qualitative | 15 | filed `qualitative` by an observation short circuit that never looks at which route succeeded. **The engine's classifier is wrong**, on at least ten of them. |
| curated → qualitative | 6 | five gas tests and instrument verdicts with no route to attribute the answer to; `bio-062` is the exception and is a real curated gap. |

Twenty of the twenty-one carry reason code `typed-observation`; one
(`bio-062`, esterification) carries `qualitative-route`. The former set still
contains both classifier questions and scripts that do not express their
comparison, while `aq-036` and `bio-062` are the two chemistry gaps.

### The 53 rows that now meet their floor by exceeding it

These used to be counted. They are not defects and never were: the corpus
predicted a hand-waved answer and got a mechanism. The owner's own example is
`bio-018`, "Why do oil and vinegar separate into layers?", which requires
`qualitative` and prints:

```
v1: two layers — hexane floating on water; mixing them would raise the
    Gibbs energy, so they split
```

That is better than `qualitative`, not worse. Same for `th-042` ("will copper
burn in oxygen?", required `curated`, answered by a CEA Gibbs minimisation),
`th-043` (sulfur, same), `bio-080` (respiration), `mat-089` (acid rain on
marble) and the seven fire-suppression rows. Twenty more are rows that took the
curated road where the corpus guessed computed, or the reverse; nobody can
predict which of the two a vessel takes and nobody should be scored on it.

**Eight of them are flattered rather than met, and the floor rule swallows
that**, which is the one real cost of this definition and is recorded here so
it is not lost: `th-029`, `mat-009`, `mat-011`, `mat-088`, `mat-109`,
`mat-120`, `mat-121` and `bio-076` are `computed` only via
`typed-engine-event`, the classifier's weakest evidence — "no solver route
claimed this vessel, but typed events happened, so call it computed". Their
disposition flatters them, and under a floor they pass a `qualitative`
requirement on that basis. What still watches them is `baseline.toml`: the
reason code is part of the drift-gated record, so if any of the eight changes
evidence class, `--check` says so by name.

### Where the classifier is wrong — 15 rows

`crates/kerotakis-cli/src/coverage.rs` runs two short-circuits *before* it ever
looks at which solver route succeeded:

```rust
if typed_observation && !plated_beside_an_aside && !inert_beside_curated {
    return Ok(result(prompt, Disposition::Qualitative, "typed-observation", routes));
}
if prompt.action == ActionFamily::HandleAndInspect
    && all_events.iter().any(|e| matches!(e, Event::Observed { .. } | Event::Measured { .. }))
{
    return Ok(result(prompt, Disposition::Qualitative, "typed-observation", routes));
}
```

The second rule files **any** `handle_and_inspect` prompt that produces a
reading as qualitative, no matter what computed it. `aq-113` asks "how much does
a beaker weigh after salt is added to water?" — the mass-conservation question —
and prints three computed numbers:

```
v1: 0.1711 mol sodium chloride dissolved
v1: pH 6.84
v1: 25.0 °C → 23.4 °C
v1 balance: 109.70 g
```

109.70 g is 99.7 g of water plus 10 g of salt. The row is filed `qualitative`
while a computed-chemistry route sits in its own report marked
`succeeded, event_count: 3`. `th-086` prints `349.22 kPa` for 0.1409 mol in
1 L at 298 K — the ideal gas law, to the digit — and is filed the same way.
`aq-124` refuses to react copper with dilute acid and explains why:

```
v1: copper does not react — copper sits above hydrogen in the activity
    series (E° +0.342 V against 0.000 V for 2H⁺/H₂), so dilute acid cannot
    take its electrons.
```

Its own tag is `computed-inertness`. It is filed `qualitative`.

The corpus README already named this as an open question and correctly declined
to settle it in a commit that was adding an instrument: *"Whether an instrument
reading with a unit ought to count as quantitative is a real question about what
`Disposition::Qualitative` means — the enum carries no definition."* It is still
open, and it is now the largest single item in the tail.

### Why `computed` and `curated` are one grade

`PhaseRouteEquilibrator` consults a curated latent heat and produces
arithmetic, and declares itself `Curated`. `CombustionEquilibrator` reads an
equally curated table of heats of combustion and declares itself `Computed`.
Whichever of the two is miscategorised, the corpus cannot be expected to
predict which road a vessel takes. Under the old equality rule eleven rows were
findings for guessing wrong, seven of which had a computed-chemistry route
succeed *as well as* the curated one and simply lost a precedence race. They
are not findings now, and the PLAN item asking whether a boil is curated or
computed is decidable on its merits rather than on its effect on a score.

## The one `missing` row

### `mat-054` — "Can glass be melted and cooled into a crystal?"

No expectation is declared, so it states no requirement and cannot be unmet. It
was deliberately moved to `missing` when `heat` gained a source with a
temperature of its own, and the run shows the bound being enforced honestly:

```
v1: 100.00 kJ requested; 2.19 kJ delivered, 97.81 kJ undelivered —
    Bunsen burner, ceiling 1500 °C
```

Fused silica melts near 1713 °C, so a burner does not melt it, which is why a
glassblower working quartz reaches for an oxy-hydrogen torch. A hotter source is
genuinely one constructor away — `apparatus::HeatSource` carries `name`,
`power`, `ceiling` and `provenance`, and already has `bunsen_burner()`,
`candle()` and `hotplate()`.

**But the second half will not close, and should not be promised.** Quartz
quenched from a melt does not crystallise; it re-vitrifies. Getting cristobalite
needs an anneal held for hours in the devitrification window, which is a
time-and-nucleation model the bench does not have and is not close to having.
The truthful ceiling for this row is "the torch melts it and it cools back to a
glass" — which is the correct answer to the learner's question, arrived at by
modelling rather than by refusing. The `cool` step needs a defined surroundings
temperature too; today it walks to −273.1 °C and then says it could not.

## What to do, in order

The ranking is by rows-per-unit-effort.

### 1. Let a succeeded computed route outrank the observation short-circuits. 21 → 11.

Ten rows — `aq-111`, `aq-112`, `aq-113`, `aq-114`, `aq-115`, `aq-124`,
`th-081`, `th-090`, `mat-008`, `aq-037` — have a computed-chemistry route
recorded as `succeeded` in their own report and are filed `qualitative` anyway.

Under the equality rule the change moved seven further rows (`aq-116`,
`aq-117`, `mat-032`, `mat-073`, `mat-087`, `mat-116`, `mat-124`) from
`qualitative` to `computed`, all of which declare `expected = "qualitative"`.
The ledger there was **nine fixed and seven broken** — not ten, because
`aq-037` requires `curated` and lands on `computed`, so it is closed by the
merged grade and not by the classifier — for a net improvement of two, which
was too small to carry a change of this size. Under the floor rule the seven
are no longer breakages and the same change is worth ten rows at no cost.

Two cautions, both already measured and both in `coverage.rs`'s own comments,
because this is the change most likely to be attempted broadly and it should
not be. Guarding the whole branch with "a computed route succeeded" was tried,
moved fifteen rows and *raised* the count: for a smell or a gas test the typed
observation IS the answer, even in a beaker that also ran an aqueous solve, and
`bio-042` (starch + HCl + heat) and `mat-029` (PET + NaOH + heat) have no
curated hydrolysis at all, so "the polymer is unchanged" is their answer. The
change that is worth ten rows is the narrow one against the
`HandleAndInspect` short circuit, not a general precedence flip.

**This is not what PR #362 attempted.** #362 ("a caveat is not the absence of
an answer") added `Event::Reacted` to the `NotYetModeled` allow-list; its own
body records that the count did not change, and the corpus README records why
it was closed — the same rule also moved `mat-099`, which demonstrates the
opposite of what its question asks. That is a semantic objection about what a
row is asking, and no grade definition touches it. #362 is not unblocked by
the floor definition and should not be cited as though it were.

### 2. The two real data gaps. 11 → 9.

`aq-036`: no gaseous ammonia species, so the damp-litmus test reads an empty
headspace while `smell v1` on the same vessel reports "sharp, pungent ammonia".
The divergence is already pinned by
`gas_tests.rs::smell_and_gas_test_disagree_about_dissolved_ammonia`, a test
written to fail once a path from solution to headspace exists. Note what must
*not* happen here: giving the gas test a route would file the row `curated` and
close it on the count while the bench still reports a confident negative on a
question it cannot see, which is worse than standing aside because nothing
about it looks like a gap.

`bio-062` ("can ethanol and acetic acid form an ester?") is the other, and it
is the only row in the 21 whose reason code is `qualitative-route` rather than
`typed-observation`: a qualitative route answered, and there is no curated
esterification for it to have taken instead.

### 3. Give the gas tests and the instruments a route. 9 → 3.

`aq-035`, `aq-038`, `bio-032` print real curated verdicts — `limewater —
positive`, with sourced thresholds from `kerotakis-core::gas_tests` — that no
solver route claims, so the classifier can never file them as `curated` however
the precedence is arranged. `th-086`, `th-087`, `th-088` compute an ideal-gas
pressure in the vessel rather than in a route, with the same consequence.
Declaring these as routes closes all six, and it should declare what the
*solver* did rather than synthesising a route for every typed event, or
`routes` stops being the record of which solver claimed a vessel — and a
negative gas test would be promoted to `curated` alongside a positive one,
which is exactly the trap under `aq-036` above.

### 4. Rows whose script cannot ask their question. 3 → 0. Corpus work, no engine work.

`th-025`, `th-094`, `th-120` — the last three of the 21 that any script change
can reach — and the comparative rows `mat-003`, `mat-108`. A perfect
model answers "how fast"; these questions ask "faster than
what" and the scripts build one condition. `mat-003` and `mat-006` print
byte-identical output and only one of them is answered — the difference is
entirely in the question, and no classifier that reads events can ever see it.
This is the ceiling on what any classifier change can achieve, and it is worth
stating before anyone plans a sixth round of them. A rewritten script closes
the question without changing the classifier.

### 5. `mat-054`'s torch. One missing row, outside the unmet-requirement count.

Real engine work with an honest ceiling — see above. Worth doing for the
capability, not for the count.

---

**The remaining tail is classification, script expression and two chemistry
gaps.** Twenty of the twenty-one open rows are typed observations, including
the argument about whether an instrument reading with a unit is quantitative;
that should be settled by deciding what
`Disposition::Qualitative` IS — not by adjusting the classifier until the count
looks better, which is the failure the enum's own doc comment exists to make
harder.

# Curiosity coverage: what the 500 rows actually say

A reading of the curiosity-v1 corpus as it stands on `main` today, written to
answer one question the owner keeps having to ask by hand: **do we cover more
now, and if not, what would it take?**

This is a companion to `tests/coverage/curiosity-v1/README.md`, not a
replacement. That file is the refresh *log* — 2000 lines, newest first, one
entry per baseline change, and it is where the reasoning for any individual row
lives. This file is the standing *analysis*: the shape of the remaining tail,
which side of each mismatch is wrong, and the order to work them in. When the
two disagree, the log is the record of what happened and this page is the
argument about what to do next; fix the argument here.

## How these numbers were obtained

Built `kerotakis-cli` at `2d364e0d` (`origin/main`, the merge of #525) and ran
the report without `--check`, which prints the counts and the split that the CI
gate only ever reduces to an exit code:

```sh
cargo run -p kerotakis-cli -- coverage curiosity
```

The individual transcripts quoted below are from running the row's own `script`
through `kero` at the same commit. Nothing here is read off the recorded
baseline; the baseline is quoted only where it agrees, which it does, exactly.

## The fresh numbers

```
curiosity curiosity-v1: 500 prompts
  computed     312
  curated       43
  qualitative   82
  boundary      60
  missing        3
  expectation mismatches: 76
    engine stood aside (corpus claimed it): 2
    route differs (both answer):           74
  solver/runtime failures: 0
  baseline drift: 0
```

**`baseline drift: 0`** is the load-bearing line. The recorded route is still
the route, on every one of the 500 rows, so the gate is measuring the engine
and not a stale file.

437 of 500 rows produce an answer; 60 more are deliberate refusals that are
correct to refuse (weaponisation, medical advice, fracture mechanics the bench
does not model), each with its own reason code. Three rows produce nothing. On
the only reading of "covered" that is not self-flattering — *the bench either
answers or says precisely why it will not* — that is **497 of 500**.

## Did coverage move today?

**Yes, once, and only once.** `baseline.toml` was last touched by `57466555`,
inside **#511** (per-liquid plateaus), which moved twenty rows from
`computed`/`computed-route` to `curated`/`curated-route` and dropped
expectation mismatches from 82 to 76. Not one of those twenty gained or lost an
answer: `qualitative` stayed at 82, `boundary` at 60, `missing` at 3. What
changed is that `PhaseRouteEquilibrator` now succeeds where it used to decline,
and it declares itself a curated-kind solver, so the classifier files those
vessels under the curated branch instead of letting them fall through to the
computed one. Eight of the twenty stopped being mismatches purely because of
that relabelling.

**Everything merged after #511 moved nothing, and we know that positively
rather than by assumption.** Thirteen PRs landed after it — #514–#522, #524,
#525, #497, #512 — and every one of them passed
`cargo run -p kerotakis-cli -- coverage curiosity --check` in CI against an
unchanged `baseline.toml`. A green `--check` on an untouched baseline *is* the
statement that no row moved.

**#509 (the Cp(T) heat ledger) has landed.** Its integral heat accounting and
follow-up conservation repair are now on main. The baseline gate records the
resulting classifications; this historical census must not be read as the
current report.

Since this census, #509, #529, #530, #531, #535, #536 and #537 changed the
engine and its evidence. Run `kero coverage curiosity --check --json` for the
current count; the numbers below are the dated argument that motivated the
remaining grading and classifier work, not a live dashboard.

## The 76 mismatches: mostly a definition problem, not a defect list

An expectation mismatch is a row whose `expected` in the shard differs from the
observed outcome. It is not a defect count and never was. Grouped by shape,
with the actual succeeded solver routes from the JSON report:

| expected → observed | n | what the evidence says |
|---|---|---|
| qualitative → computed | 26 | 18 have a real succeeded route. **The expectation is wrong.** |
| computed → qualitative | 15 | 10 have a succeeded *computed-chemistry* route the classifier discarded. **The engine's classifier is wrong.** |
| computed → curated | 11 | 7 had a computed route succeed *as well*; curated simply wins the race. **The label is wrong.** |
| curated → computed | 10 | 7 answered by a better road than predicted. **The expectation is wrong.** |
| qualitative → curated | 7 | all 7 have a curated route. **The expectation is wrong.** |
| curated → qualitative | 5 | the gas tests: a real curated answer with no route to attribute it to. **The classifier is wrong.** |
| computed → missing | 2 | `aq-053`, `aq-085`. **The engine is wrong.** |

The census, so the classes visibly add up: **43** rows are not a defect in
anything (32 mis-stated expectations plus 11 rows disagreeing about a label that
carries no quality ordering), **20** are the classifier discarding evidence it
has already recorded, **2** are real engine gaps, and the remaining **11** are
honest mismatches resting on the weakest evidence the classifier accepts. Only
that last group of 13 — the 2 gaps and the 11 weak rows — is a backlog of
missing chemistry, and one number covering all seven populations cannot be acted
on, which is what the report's `expectation_split` was already trying to say.

### Where the expectation is wrong — 32 rows

The corpus predicted a hand-waved answer and got a mechanism. The owner's own
example is `bio-018`, "Why do oil and vinegar separate into layers?", which
expects `qualitative` and prints:

```
v1: two layers — hexane floating on water; mixing them would raise the
    Gibbs energy, so they split
```

That is better than `qualitative`, not worse. Same for `th-042` ("will copper
burn in oxygen?", expected `curated`, answered by a CEA Gibbs minimisation),
`th-043` (sulfur, same), `bio-080` (respiration), `mat-089` (acid rain on
marble) and the seven fire-suppression rows in `qualitative → curated`. None of
these is a defect in anything. They are a corpus that under-predicted its own
engine.

Eight rows in this group are the exception and should **not** be swept up with
the rest: `th-029`, `mat-009`, `mat-011`, `mat-088`, `mat-109`, `mat-120`,
`mat-121`, `bio-076` are `computed` only via `typed-engine-event`, the
classifier's weakest evidence — "no solver route claimed this vessel, but typed
events happened, so call it computed". Their disposition flatters them. Leave
their expectations alone; they are honest mismatches.

### Where the classifier is wrong — 20 rows

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

### Where the label is wrong — 11 rows

`computed` and `curated` are not ordered by quality and the repo has now
documented that twice from opposite directions. `PhaseRouteEquilibrator`
consults a curated latent heat and produces arithmetic, and declares itself
`Curated`. `CombustionEquilibrator` reads an equally curated table of heats of
combustion and declares itself `Computed`. Whichever of the two is
miscategorised, the corpus cannot be expected to predict which road a vessel
takes, and eleven rows are mismatches for guessing wrong. Seven of them had a
computed-chemistry route succeed *as well as* the curated one; they lost a
precedence race, nothing more.

## The three `missing` rows at the time of this census

### `aq-053` — "Does diluted bleach remain alkaline?" — closed by #530

Before #530 the bench printed:

> not yet modelled — bleach (sodium hypochlorite) is dissolved and unspeciated:
> no thermodynamic database defines a hypochlorite species — searched by name
> for HClO, ClO-, Cl(1) and the word itself across every .dat vendored with
> iphreeqc on 2026-09-04, including the ones this lab does not load, and the
> ClO- matches are all perchlorate.

**That claim was false.** #530 borrowed the reviewed hypochlorite couple from
`llnl.dat`; diluted bleach now computes near pH 9.74. The evidence that found
the defect remains below. `vendor/iphreeqc/database/llnl.dat`,
vendored in this repo, contains:

- line 107 — `Cl(1)     ClO-      0         Cl`, a `SOLUTION_MASTER_SPECIES`
  for chlorine(+I). Perchlorate is a *different* master species three lines
  later, `Cl(7)     ClO4-     0         Cl`.
- line 898 — `Cl- + 0.5 O2 = ClO-`, `log_k -15.1014`, `-delta_H 66.0361 kJ/mol`,
  with an `-analytic` expansion valid 0–300 °C.
- line 4493 — `H+ + ClO- = HClO`, `log_k 7.5692`.

That last line is the whole answer to the question. pKa(HOCl) = 7.57, so
5 mmol of NaOCl in 500 mL hydrolyses to a pH near 10 — **yes, diluted bleach
remains alkaline**, and the bench has had the constant on disk the entire time.
The search recorded in `derived.rs` looked at the vendored `.dat` files and
missed the one that has it; `UNSPECIATED_SOLUTES` and the two test comments
asserting `NotInAnyDatabase` are wrong on the facts.

This is the **same root cause as the yoghurt gap** the corpus README already
names — lactate is in `llnl-organics` and that file is not among the four this
lab loads — and `crates/kerotakis-phreeqc/src/lib.rs` already shows the cheap
pattern: borrow the single reviewed `log K` from an unloaded llnl file with its
provenance, without loading the whole database. Two constants close `aq-053`.

**What it needs:** correct the false claim (mandatory, regardless of anything
else), then either index `llnl.dat` alongside wateq4f/minteq.v4/pitzer in
`generate-dbindex.rs`, or borrow `H+ + ClO- = HClO, log_k 7.5692` the way the
lactate work does. Half a day, one row, and it removes a sentence that teaches
the opposite of the chemistry.

### `aq-085` — "Can repeated small hexane extractions remove more iodine than one tiny extraction?"

Three separate blockers, and the arithmetic is not one of them:

```rust
// crates/kerotakis-core/src/apparatus.rs
pub fn extract_repeated(
    solute_moles: f64, aqueous_volume_l: f64, organic_volume_per_stage_l: f64,
    partition_coefficient: f64, stages: usize,
) -> ExtractionResult
```

`extract_repeated` already exists and already computes exactly what the question
asks. It is reachable from no verb in the script language. What is actually
missing:

1. **A partition coefficient for the solute.** Nothing in the registry carries
   a K_D. The run says so: `iodine in contact with liquid: no wired solver
   models this dissolution/reaction`. `volatility.rs` is the *headspace*
   partition (Henry's law, gas/liquid) and does not cover liquid/liquid;
   `lle.rs` computes whether two solvents split, which it does correctly here
   ("two layers — hexane floating on water"), but says nothing about where a
   third component goes.
2. **A verb.** `drain` moves the lower layer with everything dissolved in it;
   there is no `extract` operator to reach `extract_repeated`.
3. **The script cannot ask its own question.** It builds one extraction. The
   question is comparative. This is the `mat-003`/`mat-006` defect the corpus
   README already identified and it is a corpus change, not an engine one.

### `mat-054` — "Can glass be melted and cooled into a crystal?"

No expectation is declared, so it contributes nothing to the 76. It was
deliberately moved to `missing` when `heat` gained a source with a temperature
of its own, and the run shows the bound being enforced honestly:

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

The ranking is by rows-per-unit-effort, and the first item is worth more than
everything below it combined.

### 1. Define the grades. Implemented on `improve/coverage-and-kids-next`.

Two changes to what a mismatch *means*, both of which the repo has already
argued for in prose without acting on:

- **`computed` and `curated` are one grade.** They are ordered by provenance,
  not by quality, and the repo has documented from both directions that the
  boundary is not principled. Merging them removes 21 mismatches that are
  disagreements about which of two equally good roads a vessel took.
- **`expected` is a floor, not an equality.** It is already declared to be a
  *requirement on the engine*. A requirement is met when it is exceeded. A row
  that was required to be `qualitative` and came back with a Gibbs energy has
  over-met it. This removes 33 more.

Together: **76 → 22**, and — this is the part worth pausing on — **every one of
the 22 survivors has reason code `typed-observation` or `not-yet-modeled`.**
Nothing else is left. The entire remaining tail collapses to one question and
two known gaps, which is a far better artefact to hand the next person than a
number that mixes seven populations.

It also makes the `PhaseRouteEquilibrator` question decidable on its merits.
Today, relabelling it `Computed` is net **−7** on the mismatch count (fixes
`th-017`, breaks eight rows that declare `expected = "curated"`), so the score
argues against a change that may well be correct. Under a merged grade it is
score-neutral, and can be settled as the measurement it is.

### 2. Let a succeeded computed route outrank observation short-circuits. Implemented on `improve/coverage-and-kids-next`.

Ten rows — `aq-111`, `aq-112`, `aq-113`, `aq-114`, `aq-115`, `aq-124`,
`th-081`, `th-090`, `mat-008`, `aq-037` — have a computed-chemistry route
recorded as `succeeded` in their own report and are filed `qualitative` anyway.

**This must be sequenced after step 1, and that is the whole reason step 1 comes
first.** Done today, the change moves seven further rows (`aq-116`, `aq-117`,
`mat-032`, `mat-073`, `mat-087`, `mat-116`, `mat-124`) from `qualitative` to
`computed`, all of which declare `expected = "qualitative"` — so it breaks seven
to fix ten, a net of +2, and looks like a bad trade. Under a floor rule those
seven are no longer breakages, and the same change is worth ten rows at no cost.
PR #362 tried this ordering once and was closed unmerged for exactly this
reason; the grade definition is what unblocks it.

### 3. The two real data gaps at the time. 12 → 10.

`aq-053` (hypochlorite, closed by #530) and `aq-036` (no gaseous ammonia species, so the
damp-litmus test reads an empty headspace while `smell v1` on the same vessel
reports "sharp, pungent ammonia" — the divergence is already pinned by
`gas_tests.rs::smell_and_gas_test_disagree_about_dissolved_ammonia`, a test
written to fail once a path from solution to headspace exists). The ammonia
headspace path remains the live engine gap from this pair.

### 4. Give gas tests and pressure instruments a route. Implemented on `improve/coverage-and-kids-next`.

`aq-035`, `aq-038`, `bio-032` print real curated verdicts — `limewater —
positive`, with sourced thresholds from `kerotakis-core::gas_tests` — that no
solver route claims, so the classifier can never file them as `curated` however
the precedence is arranged. `th-086`, `th-087`, `th-088` compute an ideal-gas
pressure in the vessel rather than in a route, with the same consequence.
Declaring these as routes is a small, mechanical change and closes six rows.

### 5. Rows whose script could not ask their question — scripts repaired on `improve/coverage-and-kids-next`.

The comparative rows `mat-003`, `mat-108`, and `aq-085` now build both
conditions. The first two compare matched vessels; `aq-085` spends the same
total hexane in one 0.5 mol extraction and two 0.25 mol extractions. This makes
the experiments askable without pretending the classifier can judge the
comparison from unrelated events. The iodine partition coefficient remains a
real engine gap.

### 6. `aq-085`'s partition coefficient, and `mat-054`'s torch.

Real engine work, one row each, and both are honest about their ceilings — see
above. Worth doing for the capability, not for the count.

---

**The single highest-value change is the first one, and it is not an engine
change at all.** Defining `computed` and `curated` as one grade and `expected`
as a floor turns a 76-row number that mixes seven populations into a 22-row list
where every entry is the same open question or a named gap — and it converts the
classifier fix in step 2 from a losing trade into a winning one. No row moves,
no baseline is regenerated, and the gate gets strictly harder to satisfy by
accident.

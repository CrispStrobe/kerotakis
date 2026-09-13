# The measure after the curiosity corpus

A proposal for the owner. Nothing here is implemented: no engine change, no
test, no new gate. It sets out three candidate successors to the curiosity
corpus with what each would cost, recommends one, and names the first step.
The decision is the owner's.

## 1. Why the corpus stopped discriminating

`docs/CURIOSITY-COVERAGE.md` records 499 of 500 prompts answered — 320
computed, 64 curated, 55 qualitative, 60 boundary, 1 missing. The one missing
row is `mat-054`, and it was judged unable to close honestly rather than left
undone. A measure that reads 499/500 cannot rank work any more; it has no
remaining shadow to point at.

Those first two figures were 328 and 56 yesterday. **#582 moved eight rows
from `computed` to `curated` this afternoon, and its own commit message is the
cleanest statement of this document's thesis available:**

> No chemistry changed. The same solver examines the same vessels, burns the
> same fuel, releases the same heat and emits the same events; only the label
> on the road changed.

That change was correct and well argued — `CombustionEquilibrator` was taking
the `Equilibrator` trait's default `route_kind` by omission while running
entirely on curated tables. But note what moved and what did not. Eight of the
corpus's five hundred rows changed disposition, and the bench's answers are
byte-identical. A measure whose largest movement in a day is a relabelling is
measuring labels.

`CONTRIBUTING.md` §6 already says why that matters, in the repository's own
words: "a defect can be latent for exactly as long as a corpus row is
`missing`... the `missing` column is a shadow as much as it is a to-do list."
The corpus has spent its shadow. That is success, not failure, and the
question is only what replaces it.

But the corpus also never asked the question that matters most, and this is
the more important half. It asks **whether the bench answers, and how strong
the route was**. It never asks **whether the answer is right**.

## 2. The gap, measured

Every gate this repository runs is self-referential. That is not a
characterisation, it is a count.

| Gate | Cases | What a passing row asserts |
|---|---:|---|
| Curiosity corpus (`tests/coverage/curiosity-v1`) | 500 | A route of at least the declared strength produced an answer |
| Semantic catalog assertions (`docs/SEMANTIC-ASSERTIONS.md`) | per-reaction | A relation between two values **the engine itself produced** |
| Chemistry-audit fleets (`tools/chemistry-audit/`) | 896 | A relation between two **engine runs** |
| Differential oracle (`crates/kerotakis-phreeqc/tests/oracle/expected/`) | 10 | Agreement with **stock PHREEQC** — a second implementation of the same model |
| Metamorphic invariants (`crates/kerotakis-cli/tests/metamorphic.rs`) | — | A relation between two **engine runs** |

The fleet analyser is the clearest case. `analyse_source_fleets.py` admits
exactly four relation kinds — `conservation`, `independent-law`,
`metamorphic`, `boundary` — and ten assertions, every one of which is
`…-equal` between two engine runs, `…-order` between two engine runs, or
`…-present`. `independent-law` is a misleading name: **all 61 relations of
that kind across the shipped manifests use `event-scalar-order`**, an
*ordering* between two of the bench's own results. It proves a quantity moves
the right way. It cannot prove it moves the right distance.

`docs/SEMANTIC-ASSERTIONS.md` is the same shape at the catalog layer: `kind`
is `equal`, `increasing`, `decreasing`, `conserved`, `unchanged`, or `ratio`.
There is no kind that takes a number from outside the repository.

The fleets do compare numbers, and carefully — `min_delta: 0.2` on an ordered
pH series, `atol 0.03` on an intensive buffer pH, `atol 1e-9, rtol 1e-7` on an
inventory. The numbers on both sides of every one of those comparisons come
out of this engine.

The differential oracle is the closest thing to an outside check, and it is
worth being exact about what it is: `tests/oracle/expected/*.json` holds ten
fixtures whose `oracle` field reads, for example, *"MY-BASIC adapter (values
captured from legacy oracle during development)"*. It compares the engine to a
different implementation of the same model. That catches adapter bugs. It
cannot catch a model that is wrong, because the reference shares the error.

**So: 896 frozen fleet cases, 500 corpus prompts, a per-reaction assertion
language and ten oracle fixtures, and not one of them compares a computed
value to a measured one.**

The repository already says this, in the place it matters most. The
disposition enum's own documentation
(`crates/kerotakis-codex/src/curiosity.rs:105-120`):

> **It is about the ROUTE, not about the ANSWER.** This is the load-bearing
> sentence and the one most often forgotten. A disposition says which part of
> the bench spoke. It says nothing whatever about whether the PROMPT's
> question was answered.

The corpus README states the same omission as a design decision: it
*"deliberately excludes rendered prose, numerical solver details, and route
timing."* Grepping `crates/kerotakis-cli/src/coverage.rs` — all 1,157 lines of
the actual grader — for `f64`, `abs(`, `tolerance`, `atol` or `rtol` returns
only comment prose. **There is no numeric comparison in the grader at all.**

### The brine defect, and why nothing caught it

The engine computed freezing-point depression from the dilute law
`ΔT = K_f·m`. At one molal that gives −3.72 °C; the measured value is −3.4 °C.
It was wrong by about nine per cent for as long as it existed, and it was
found by hand, by one agent comparing against published freezing-point
depressions while doing something else (`improve/colligative-water-activity`,
merged as #578).

Check the defect against each gate:

- **Conservation** passes. The dilute law conserves every element.
- **Metamorphic** passes. Two routes to the same brine agree with each other.
- **`independent-law` (ordering)** passes. `ΔT = K_f·m` is strictly monotonic
  in molality, so every more-concentrated case reads colder than every
  less-concentrated one, exactly as required.
- **Boundary** passes. The answer is inside the model's declared range.
- **Curiosity corpus** passes, and passes at the *top* grade: a computed route
  produced a number, so the row graded `computed`.

A bug nine per cent wrong sailed through all five. This is not a gap in
coverage that more of the same would close. It is the axis none of them
measures.

### It is not one incident

`HISTORY.md` line 1656 records the same class of defect, found the same way:
"curated data checked against handbook values found permanganate's molar
absorptivity curated at 1.8× the literature value." That is 80 per cent wrong,
in curated data, found by a human comparison that nothing in the repository
performs systematically.

And there is a third, which was open until this afternoon. `PLAN.md` lines
1832–1838 record that a 0.1 molal brine was still answered by a Debye–Hückel
dataset and took Raoult's law: **−0.371 °C against a measured −0.346** — about
seven per cent optimistic, in the same direction the dilute law was. It was
written down, it was understood, and nothing failed because of it.

**PR #584 is fixing it right now, and it is the best evidence in this
document, because it is the pattern repeating in real time.** The fix is
correct and carefully argued; it asks `pitzer.dat` for the solvent activity in
a second lean speciation and lands the beaker at −0.347. Look at how the
reference values travel:

- the PR is a draft whose text carries `@@TENTH@@` placeholders "that will be
  replaced with the measured numbers before this comes out of draft" — the
  measurements are being sourced **by hand, now**;
- the new test's doc comment reads "φ comes back at 0.932 against a measured
  0.9324" and "−0.371 °C against a measured −0.346";
- **neither measured value carries a citation.** No DOI, no edition, no page,
  no `source_id` — consistent with the rest of `colligative_numbers.rs`, which
  has none in 350 lines;
- and the PR adds no gate that would catch the next defect of this kind.

Four numeric defects, four hand discoveries, four hand fixes, and after each
one the measured value that proved it is stored as English in a comment. That
is not a criticism of any of the four changes. It is the argument that the
repository is missing an instrument, not diligence.

The measured reference values in all three cases live in commit messages and
in prose — `HISTORY.md` lines 124, 125, 139, 146 — where no gate can read
them. `−3.4`, `108.7`, `0.936`, `−0.346` are the numbers that mattered most to
a year of chemistry work, and they are stored as English.

## 3. Option A — a numeric accuracy corpus

Rows of the form: *this script, run on this bench, produces this quantity;
the published value is X ± t, from source S.*

### What already exists

More than the framing of the question assumes. Every ingredient is built
except one.

| Ingredient | Exists? | Where |
|---|---|---|
| Row-per-claim corpus: manifest, shards, join-by-id, count pin | yes, twice | `tests/coverage/curiosity-v1/`, `tools/chemistry-audit/source-fleets*/` |
| Row schema with value + per-observable tolerance + justification + provenance | **yes, in production** | `crates/kerotakis-phreeqc/tests/oracle/expected/*.json` |
| Per-row source field in a shipped data file | yes | `data/thermo/uscg-chris-still.tsv` |
| Source registry with licence, lane, checksum, reviewer, date | yes | `provenance/sources.toml` + `kero provenance lint` |
| Every numeric value must name a resolvable source id | yes | `crates/kerotakis-data/src/validate.rs:1312` |
| Licence-cleared reference sources, approved and in use | yes, three | `usbm-ic9429-complexes-25c`, `sander-2023-hbr-reference`, `uscg-chris-still-reference` |
| Process-isolated frozen replay in CI | yes | `.github/workflows/chemistry-audit.yml`, 896 cases |
| A worked DOI-level citation of a measured number | **once, of 1,917** | `literature/hartley-campbell-iodine-water` |
| A tolerance argued rather than picked | **once, of ~870** | `crates/kerotakis-phreeqc/tests/colligative_numbers.rs` |
| **Values whose provenance is a measurement, not a re-implementation** | **no** | — |

Four of those rows deserve detail.

**Licence-cleared reference data is a solved problem here, three times over.**
Each of the three approved reference sources has a review note beside it
(`provenance/*-review.md`, 49–97 lines) stating origin, terms, the exact fields
cleared, the fields deliberately *not* taken, and download hashes.
`provenance/uscg-chris-still-review.md` even records rejecting a wrong number
a search engine returned in favour of what the original PDF prints. **The
question "where can reference data honestly come from" has a worked answer and
a routine**: a public-domain government publication or a CC-BY paper, a review
note, a `sources.toml` entry, `decision = "approved"`. The cost is about one
review note per source, not a new policy.

The doctrine for a single measured number is written out once, in
`literature/hartley-campbell-iodine-water`:

> "H. Hartley and N. P. Campbell, 'The solubility of iodine in water',
> *J. Chem. Soc., Trans.* 93 (1908) 741-745, doi:10.1039/CT9089300741:
> 0.3393 g I₂ per litre water at 25 °C. Kerotakis converts this factual
> measurement to 0.03393 g/100 mL and releases its transcription as CC0; no
> source text or table is redistributed."

Individual factual measurements, individually transcribed, individually cited,
released CC0 — never a table copied from a compilation. Bulk compilations stay
out: `CONTRIBUTING.md` §3 makes NIST SRD/WebBook, CAS Common Chemistry, CAMEO
exports, ECHA dumps, Burcat and UNIFAC Consortium tables "a legal constraint,
not a style preference", and `PLAN.md` records dropping the Python `chemicals`
package because "it launders the SRD and NC problems into our binary." The
build-time principle clears the second supply route — `thermo` (MIT), Cantera,
ChemPy (BSD-2), Reaktoro (LGPL, build-time only) — persisting "only approved
facts or aggregate metrics, never an unreviewed fixture export."

**The row format is already in production.**
`crates/kerotakis-phreeqc/tests/oracle/expected/simple_kinetics.json`:

```json
{
  "description": "Simple first-order decay: rate=0.5*M*TIME, 0.5s step",
  "oracle": "MY-BASIC adapter (values captured from legacy oracle during development)",
  "observables": {
    "k_Decay": {
      "value": 0.7788,
      "absolute_tolerance": 0.00005,
      "notes": "Runge-Kutta integration of rate=0.5*M*TIME over 0.5s"
    }
  }
}
```

A value, a per-observable tolerance, a justification and a provenance field,
loaded by two tests, with `tools/oracle/README.md` (LIC-010) specifying the
promotion contract: *"approved oracle facts (numerical values, tolerances) are
copied to `crates/*/tests/oracle/expected/` as reviewed test fixtures."*
**The schema this option needs is written and running.** What its `oracle`
field names is a program, not a publication — and that is the whole gap.

**Execution and freezing already run in CI.** `tools/chemistry-audit/` builds
the CLI once, hashes the binary, records submodule state, and replays 896
frozen cases in process-isolated shards, retaining stdout, stderr, scripts,
final benches, hashes and law-check reports. A case is `{id, question, script}`
under schema `kerotakis-source-fleet-v1`. **Adding `expected`, `unit`,
`tolerance`, `tolerance_reason` and `source_id` to that record is the whole
data-model change.**

Also built and unused: 946 lines of external-oracle tooling
(`tools/vle-oracle.py`, `kinetics-oracle.py`, `surface-oracle.py`,
`check-properties-vs-chempy.py`, `check-relations-vs-chempy.py`) whose
comparator signature is already `check(label, kero_val, ref_val, tol,
ref_source)`, run by neither `tools/preflight.sh` nor CI.

**Per-quantity tolerance has an anchor, and it is the wrong value.**
`crates/kerotakis-core/src/instrument.rs` declares a precision per instrument
on a `Reading` struct — thermometer ±0.1 °C, balance ±0.01 g, pH meter ±0.01,
pressure gauge ±0.1, conductivity ±1.0, calorimeter ±0.01, spectrophotometer
±0.001, melting-point apparatus ±0.5. Eight quantities with a declared numeric
bar is the right *axis*. It is not the right *value*: instrument precision is
how finely the bench reads, model tolerance is how far the model may sit from
reality, and the brine error was 0.32 °C against a thermometer declaring
±0.1 °C. Set tolerance at instrument precision and nearly every row fails on
day one and the gate gets switched off; set it loose enough that everything
passes and it proves nothing. **Tolerance must be argued per quantity and per
claimed model, in the row, beside the citation.** A pH from a Debye–Hückel
dataset, a freezing point from an ion-interaction one and an adiabatic flame
temperature do not deserve the same bar.

The repository has done this exactly once, and the argument generalises.
`colligative_numbers.rs` pins one claim from both ends: a tight band on the
model's own figure (`|freezing_c + 3.44| < 0.06`) and a loose band on the
world's (`|freezing_c + 3.4| < 0.15`), with the rule stated in the file —
narrowing the outer band "would fail on a change that made the answer better;
widening it would let the dilute law back in — its −3.72 is outside this band,
which is the point of keeping the band." The width is justified physically
("published depressions for 1.000 molal NaCl sit between 3.37 and 3.44 K
depending on the source"). Across ~870 numeric band assertions in the
workspace, **this is the only tolerance that is argued rather than picked, and
it carries no citation** — no DOI, no edition, no page, in 350 lines.

**And the policy already exists.** `CAPABILITIES.md` §2 requires of every
capability task: *"new solver paths get the conservation and metamorphic
invariants … plus at least one golden test against a textbook value."* The
last clause is Option A, already mandated. Nothing counts those golden tests,
nothing gates on them, and the brine shipped without one.

### What it would cost

- **A tolerance policy, per quantity.** The genuinely hard part, and the part
  that cannot be delegated to a generator. Perhaps a day of argument for the
  first six quantities, then cheap per addition.
- **A source review per new reference source**, at the rate the three existing
  ones were done: ~70–100 lines each.
- **One new assertion kind** and its analyser branch. Small against the 1,157
  lines of `crates/kerotakis-cli/src/coverage.rs` that already exist.
- **Seeding the rows.** Every measured value already written down in prose is
  a free row: `HISTORY.md` lines 124, 125, 139, 146 alone supply four, and
  `PLAN.md` 1832–1838 supplies a fifth that fails today.

### What it would catch that nothing catches today

The brine defect, on the day it landed. The permanganate absorptivity, at
1.8×. The open 0.1-molal case, immediately. Any future change that keeps a
quantity monotonic and conserved while moving it the wrong distance — which is
the failure mode of every curated constant, every fitted parameter, and every
limiting law used outside its limit.

### What it would miss

- **Anything with no published value**, which is a large part of a teaching
  bench: qualitative observations, refusals, boundary behaviour, prose, and
  the entire interface. It supplements the corpus's qualitative and boundary
  rows; it does not replace them.
- **Systematic error shared with the reference.** If the reference value and
  the engine both come from the same upstream database, agreement proves
  nothing. The row must record where the reference came from *and that it is
  independent of the path under test* — this is the same trap `HISTORY.md`
  1657 records for identity: "a registry key can pass an identity gate for the
  wrong reason, as Al's stored InChIKey matched an independently-wrong hydride
  computation."
- **Reachability.** A perfectly accurate quantity no learner can obtain still
  scores full marks.

### How it could go wrong

**The dominant risk is a tolerance chosen to make the row pass.** That is this
repository's recurring failure mode — a gate that reads stronger than it is —
in its purest form, and it is invisible in the count. Mitigation is structural
rather than procedural: the tolerance and its *reason* are separate fields, a
row whose tolerance was widened shows that in its diff, and
`CONTRIBUTING.md` §4's existing rule — "contributions that hardcode 'expected'
results defeat the project's premise" — must be read carefully here. A
hardcoded expected result in the *engine* defeats the premise. A cited
external value in a *test* is the opposite of hardcoding: it is the only thing
that can tell the engine it is wrong. The proposal recommends saying that
distinction out loud in `CONTRIBUTING.md`, because a reviewer will otherwise
reasonably object.

Second risk: reference values transcribed wrong. The existing review notes
already handle this with download hashes and per-field selection.

### What a passing score would and would not prove

**Would prove:** for each covered quantity, on each covered script, the bench
agrees with an independently sourced published number to a stated and argued
tolerance.

**Would not prove:** that the model is right anywhere between the rows; that
the tolerances are appropriate; that the quantities not covered are accurate;
that a learner can reach any of it; or that the engine is right for the right
reason — a fitted constant and a correct mechanism can hit the same number.
A high score with three quantities covered says nothing about the fourth, so
**the count that matters is quantities covered, not rows passing.**

## 4. Option B — a harder prompt set

Grow or replace the corpus with questions the bench currently answers badly.

### What already exists

The whole pipeline. `tests/coverage/curiosity-v1/` is four authored shards
plus `manifest.toml` (`target_prompts = 500`, 16 named smoke prompts) and
`baseline.toml`, which records the observed native route per prompt as id,
owning task, outcome and stable reason code. `CuriosityPrompt` is
`deny_unknown_fields`; the loader names its shards explicitly. `--check`
accepts recorded known failures and fails on drift, `--emit-baseline`
regenerates, and `tools/curiosity-prose.py`, `tools/curiosity-index.py` and
`tools/curiosity-answer-invariance.py` already validate, index and translate
it. A fifth shard is close to free: it is a file, a manifest line, an i18n
file per language (`I18N.md`'s modularity rule), and a baseline refresh.

Marginal cost is the lowest of the three options by a wide margin.

### What it would catch that nothing catches today

Missing *capability*, faster and more cheaply than anything else here — which
is exactly what the first corpus was for, and it worked. If the goal is to
rank the next tranche of chemistry breadth, this is the instrument that has
already proved it can do that.

### What it would miss

**The same thing the first corpus missed: whether any answer is right.** The
grading asks which route answered and how strong that route was. A harder
shard graded the same way would have passed the brine defect for the same
reason the existing 500 did. Adding 200 harder questions to a measure that
cannot see a nine per cent error produces a harder measure of the same one
thing.

### How it could go wrong

**The stated risk is real and is not fully solvable.** A set written by the
process that writes the engine tests what the engine already conceives of.
The existing corpus is described in its own README as "four authored prompt
shards"; there is no independence claim anywhere, and none was needed when the
goal was breadth.

Three mitigations, honestly ranked:

1. **Anchor the questions to an external syllabus.** `EXPERIMENTS.md` EXP-38
   already plans public-curriculum labels (CBSE / ICSE / IGCSE / NGSS). A
   required-practical list from a published syllabus is an outside index of
   what a learner is supposed to be able to do, and it is not derived from
   this engine. This is the strongest available option and it is already on
   the roadmap. Note the topic *list* is factual; the syllabus document's
   prose is not ours to copy.
2. **Derive questions from reference data rather than from the bench.** Pick a
   published measured value first, then write the question it answers. The
   set's shape then comes from the literature. This is real independence — but
   note what it is: **it is Option A wearing Option B's clothes.** The
   defensible form of "a harder prompt set" turns out to be the numeric
   accuracy corpus.
3. **Author the set without the engine in context.** Cheap, and weak. It
   removes recall of the implementation, not the shared conception of what
   chemistry a bench does. Worth doing, never worth relying on.

The second failure mode is subtler and this repository has already named it.
`ROADMAP-Webapp.md` line 248: "Curriculum coverage counts topics, not
scientific capability — 'covered' does not distinguish one scripted example
from a reusable model that spans a whole family." A prompt set rewards a
scripted example exactly as much as a general model, so it can be raised by
authoring rather than by engineering.

### What a passing score would and would not prove

**Would prove:** the bench produces an answer of at least the declared
evidence grade for each listed question, and did not regress.

**Would not prove:** that any answer is numerically right; that the questions
are hard in any sense other than that this project found them hard; that the
capability generalises past the authored script; or that a learner can reach
it. And critically, the corpus's own success shows the ceiling: a set of this
kind runs out of shadow. Option B buys a second finite instrument of the
first's kind, and the day it reads 499/500 this document gets written again.

## 5. Option C — a reachability measure over the product

Can a learner actually reach each capability the engine has?

### The evidence is real, and it is worse than stated

`crates/kerotakis-core/src/electrodiffusion.rs` is **1,567 lines — 688 of
implementation and 879 of tests — with zero callers anywhere in the
repository.** Every one of its ten public symbols (`NernstPlanckDomain`,
`ReactiveNernstPlanckState`, `zero_current_junction`,
`electroneutral_surface`, `reactive_electroneutral_surface` and the rest)
returns nothing on a repo-wide grep across `crates/`, `web/`, `tools/`, `docs/`
and the root documents. The single reference to the module is its own
declaration at `crates/kerotakis-core/src/lib.rs:42`. It is invisible to
`dead_code` because it is `pub`. The word itself appears once more in the
workspace, and it is the detail that makes the case: `electrochemistry.rs:2732`
— a neighbour in the same crate — returns the error `"charged
local-equilibrium bulk needs an explicit electrodiffusion model"`. One module
refuses for want of exactly what the module beside it computes.

For contrast, files referencing each sibling module in the same crate:
`displacement` 18, `phase_route` 8, `relations` 8, `compartment` 7,
`heterogeneous` 4, **`electrodiffusion` 0.** It is the unique orphan.

`crates/kerotakis-core/src/polarization.rs` is the weaker second case: 1,010
lines, exactly one caller, a CLI fitting subcommand. It is not in the grammar
`VERBS`, not in `affordances.json`, not tiered in `catalog.rs`, and not
reachable from the web bench at all.

**Every gate in this repository passes `electrodiffusion.rs`.** The curiosity
corpus does not ask which modules answered. The fleets replay scripts, and no
script can name it. The semantic assertions are per-reaction. It compiles, its
879 lines of tests pass, and nothing observes that no learner and no script can
ever reach it.

### What already exists

More than expected, and pointed the wrong way.

**`crates/kerotakis-core/src/catalog.rs`** (601 lines) is the strongest
foundation in the repository for this. Its header reads: *"WORLD-003 — the
runtime catalog contract. One answer to 'what can this learner reach, and
why'."* It holds `APPARATUS_MILESTONES` (22 `(verb, tier)` rows),
`INSTRUMENT_MILESTONES` (14 `measure:<token>` rows) and `NOT_CABINET` (16
verbs deliberately excluded **with a written rationale each**), and it joins
them through `catalog()` out to wasm (`kerotakis-wasm/src/lib.rs:514`) and into
the browser (`web/app/src/lib/catalogProgress.ts`). A `CatalogItem.id` is a
verb, an instrument token, or a registry species key — **it is the one place
where engine capability and learner reachability share an id space.**

And it already carries the one true reverse check, at
`crates/kerotakis-core/src/catalog.rs:531`:

```rust
fn every_parsed_verb_is_either_tiered_or_deliberately_not_cabinet() {
```

with the failure text *"verbs the parser knows but nothing tiered: {untiered}
— give each a milestone, or list it in NOT_CABINET with a reason."* That is
precisely the measure this option proposes, already written, at **verb**
granularity.

Three more precedents, all reusable:

- `tools/test-protocol-conformance.mjs:585–614` — the GUI-029 sandbox-
  completeness invariant, which runs **both** directions between
  `Lab::grammar()`'s 35 verbs and `web/app/src/lib/affordances.json`'s 35
  rows, and — the part worth copying — **prints a tracked metric for known
  gaps** (`planned:GUI-033` rows are counted, not failed).
- `crates/kerotakis-phreeqc/tests/curated_reachability_at_runtime.rs:133`,
  `every_curated_reaction_is_reachable_in_every_order()` — a runtime
  capability→reachable test for one capability class, whose header records
  that it was falsified against a real bug. The methodological model.
- `data/kids/experiments-v1.json` — 77 entries joining `capabilities` (corpus
  prompt ids) → `lesson` → `codex` → `status`, and **`"unreachable"` is
  already in its status vocabulary** (currently 0 rows).

Also relevant: `crates/kerotakis-cli/tests/cabinet.rs:191`,
`help_names_every_verb_the_grammar_accepts()`, exists because KID-17 found
*"`magnet`, `smell`, `test`, `chromatograph` and `react` were all landed, all
working, and absent from every surface a reader has."* **This exact failure
mode has already been diagnosed once and fixed at one layer only.**

### The blocker

**There is no enumerable capability surface to measure against.** Capability is
scattered over at least four disjoint lists with no authority among them:

| List | Location | Count |
|---|---|---:|
| `pub const VERBS` | `crates/kerotakis-core/src/script.rs:24` | 35 |
| `pub enum Operator` | `crates/kerotakis-core/src/ops.rs:115` | ~40 |
| `pub enum Event` | `crates/kerotakis-core/src/ops.rs:788` | ~111 |
| Catalog tiers | `crates/kerotakis-core/src/catalog.rs` | 22 + 14 + 16 |

There is no `enum Capability` and no registry of solver modules. `VERBS` is
the only list treated as authoritative, and it is a strict subset: whole
modules — `electrodiffusion`, `polarization` — appear in none of the four.
**Building that list is the prerequisite, and it is the expensive part**,
because deciding what counts as one capability is a judgement call on every
line of it, and a list built by reading the code will name what the code
already names.

### What it would cost

The most of the three. The enumeration is a design task, not a mechanical one,
and it must be maintained by hand forever after or it decays into a list of
what existed when someone last wrote it down. Against that: `catalog.rs` shows
the pattern works, and `NOT_CABINET`'s "excluded with a written reason" is the
right escape hatch — an unreachable capability is allowed, but it must be
declared and defended.

### What it would catch that nothing catches today

`electrodiffusion.rs`, on the day it merged. `polarization.rs`. The five
grammar verbs (`smell`, `magnet`, `irradiate`, `centrifuge`, `discard`) that
appear in zero of the 113 lessons, and the twelve with two or fewer — nothing
reports this today. The 36-versus-34 mismatch between Rust's catalog ids and
`EQUIPMENT_CATALOGUE`'s 34 gated ids, which no test crosses languages to
check.

### What it would miss

**Whether anything reachable is correct.** This is the sharpest limitation and
it should decide the recommendation. A reachability measure would have said
nothing whatever about the brine, the permanganate, or the open 0.1-molal
case: all three were reachable, prominent, exercised by lessons, and wrong.
Reachability and accuracy are orthogonal, and this option measures the one the
repository has *not* just been burned by.

### How it could go wrong

**It rewards a thin binding.** A verb becomes "reachable" the moment one
lesson mentions it, whether or not the lesson exercises the capability or a
learner would ever find it. `ROADMAP-Webapp.md` line 248 names this exactly:
*"Curriculum coverage counts topics, not scientific capability — 'covered'
does not distinguish one scripted example from a reusable model that spans a
whole family."* A reachability count is that failure mode with a different
subject.

Second: the enumeration becomes the definition. Once `enum Capability` exists,
a capability nobody enumerated is not merely unmeasured — it is invisible, and
the measure will read 100 % while `electrodiffusion.rs`'s successor sits
outside the list.

### What a passing score would and would not prove

**Would prove:** every enumerated capability has at least one declared path to
a learner surface, or an explicit, written exclusion.

**Would not prove:** that the path is discoverable, that a learner would ever
take it, that the capability works, that its answers are right, or that the
enumeration is complete — and the last of these is the one that matters, since
the measure is exactly as good as the list it is measured against, and the
list is hand-maintained.

## 6. Recommendation — Option A, the numeric accuracy corpus

### The argument

**It is not a new measure. It is the one rung of the repository's own
validation ladder that was never built.**

`ROADMAP-Webapp.md` lines 935–945 sets out five verification levels:

1. local invariants — element, mass, charge, energy balance — **built**
2. metamorphic tests — scaling, reordering, splitting, units — **built**
3. analytic cases — closed forms, Nernst limits, lever rule — **built**
4. differential oracles — Reaktoro, Cantera, build-time Python — **built**
5. **experimental benchmarks — DOI-pinned, licence-audited measurements with
   conditions and tolerances — not built**

The engine climbed four rungs over a year. The curiosity corpus measured none
of them: it measured whether a route fired. Levels 1–4 are exactly the gates
that all passed the brine defect, and the ladder already explains why, three
lines further down:

> Agreement between two paths using the same database is useful but is not an
> independent validation of that database.

That sentence is the whole case. It is already written, in the roadmap, above
a section headed **"Validation corpus"** that specifies `validation/cases/`
field by field — initial ledger, model path expected to claim it, quantities
and tolerances, conservation expectations, **source/oracle and licence**, known
disagreement modes, and the domain over which the case may generalise.

**The design is done. It was specified, and then the engine kept deepening and
the corpus kept counting routes.**

Five supporting reasons:

1. **It is the only option that would have caught the defects this project
   actually shipped.** Three known instances: brine at 9 %, permanganate
   absorptivity at 1.8×, and the 0.1-molal case still open at 7 %. Option B
   would have missed all three. Option C would have missed all three.
2. **It would have been red for months, and nothing said so.** The 0.1-molal
   row would have failed from the day the dilute law landed until #584 merges.
   That is the value of the instrument: not that it starts red today, but that
   it would have been red the whole time the defect existed, visibly, instead
   of waiting for someone to check a number by hand while doing something
   else. A measure whose rows can be red for known, understood, argued reasons
   is the opposite of this repository's twice-hit failure mode.
3. **The machinery is mostly built.** The row schema runs in production
   (`tests/oracle/expected/*.json`); the source registry, its lint and its
   referential integrity run in preflight; the licence doctrine has a worked
   template and three approved reference sources; the fleet runner replays
   896 cases in process-isolated CI shards.
4. **It restores a shadow.** The corpus's power came from its `missing`
   column. A quantity with no cited reference value is the new `missing`, and
   there are many: of 1,917 registry numeric records, **1,093 report no
   uncertainty at all, none carries an absolute, relative or interval
   uncertainty, and not one is marked `measured`.**
5. **It makes an existing rule enforceable.** `CAPABILITIES.md` §2 already
   requires "at least one golden test against a textbook value" per new solver
   path. Nothing counts them.

### Why not the others

**Option B** is cheapest and its pipeline is complete, and it should still
happen — but as maintenance of a working instrument, not as the measure that
drives the next stretch. Graded on route strength, a harder shard reproduces
the blindness at higher difficulty. Its own best independence mechanism —
sourcing questions from published values — is Option A.

**Option C** identifies a real and embarrassing failure: 688 lines of solver
that no learner can reach, which every gate passes. But it is the most
expensive, its prerequisite (an enumerable capability surface) is a design
problem with no obvious right answer, and it measures the axis the repository
has *not* just been burned by. It is the right second move. `catalog.rs:531`
already does it at verb granularity, so the gap is narrower than it looks, and
the specific orphan is cheaper to fix directly than to build a measure for.

### What a passing Option A score would and would not prove — restated plainly

This document's own warning applies to its own recommendation. A green
accuracy corpus would prove that **the covered quantities, on the covered
scripts, agree with cited published values inside stated tolerances.** It
would prove nothing about quantities not covered, about behaviour between the
rows, about whether a tolerance was argued or chosen to fit, about
reachability, or about the model being right for the right reason.

**Therefore the number to publish is not "rows passing". It is "quantities
covered, with a cited source and an argued tolerance", against the list of
quantities the bench can compute.** A corpus reporting 40/40 rows green over
three quantities has told the owner almost nothing, and a document reporting
it as a score would be the third instance of a gate reading stronger than it
is.

### The first concrete step

**Build `validation/cases/` with one family — colligative — and six rows, and
do not wire it into CI.**

Concretely:

1. Create `validation/cases/colligative.toml` following the field list already
   written at `ROADMAP-Webapp.md` §"Validation corpus", with the row shape
   borrowed from `tests/oracle/expected/*.json`: per quantity a `value`,
   `tolerance`, `tolerance_reason`, `source_id`, and the script that produces
   it.
2. Seed it from numbers already in the repository, which cost nothing to
   source: the four in `HISTORY.md` lines 124–146 (1 molal freezing −3.4 °C,
   6 molal boiling 108.7 °C, φ = 0.936 at 1 molal, 0.1 molal freezing
   −0.346 °C), the sucrose depression and van 't Hoff factor already asserted
   in `colligative_numbers.rs`, and the two #584 is adding as this is written
   (φ = 0.9324 at a tenth molal, and the −0.346 it lands against). **Do this
   after #584 merges**, so the family is seeded from the corrected engine and
   the rows start green except where the repository knows they should not.
3. **Give each a real citation.** `colligative_numbers.rs` asserts every one of
   these against "measurement" and "textbooks print" with **no DOI, no edition,
   no page, anywhere in its 350 lines**. Closing that is the actual work of
   step one, and it is the step that proves the approach is affordable — or
   proves it is not, which is equally worth knowing before committing.
4. **Carry over that file's tolerance argument**, which is the only justified
   tolerance in the repository: a tight band on the model's own figure plus a
   loose band on the world's, the outer band "wide enough to survive an
   improvement and narrow enough to exclude the known-wrong predecessor" —
   `−3.72` sits outside `−3.4 ± 0.15`, deliberately. That rule generalises and
   should be written down as policy in step one.
5. Report it as **one quantity covered**, and let the owner see what six
   honest rows cost before deciding whether the next fifty are worth it.

One family, six rows, no gate, no engine change. If the citations turn out to
be unaffordable, that is the finding, and it is cheap.

## 7. Defects found while investigating

Noted, not fixed, as instructed.

1. **`crates/kerotakis-core/src/electrodiffusion.rs` has zero callers** — 688
   implementation lines, 879 test lines, referenced only by `pub mod` in
   `lib.rs:42`. `HISTORY.md:19` says it was "Integrated … exposing homogeneous
   mass action, migration, electroneutrality and component-flux closure in one
   focused test envelope." The test envelope is real; the integration is not.
   `PLAN.md:112` still lists integrating it as open work item 1, so the
   history and the plan disagree about the same module. The sharpest detail:
   the only other mention of the word anywhere in the workspace is an error
   string in a *neighbouring* module — `electrochemistry.rs:2732` returns
   `"charged local-equilibrium bulk needs an explicit electrodiffusion
   model"`. One module refuses for want of exactly what the orphan beside it
   computes, and nothing connects them.
2. **Avoid-list sources are cited as expected values.**
   `crates/kerotakis-cea/tests/heat_source_ceiling.rs:28-30` cites "NIST
   Chemistry WebBook SRD 69" and "NIST-JANAF table Ca-027", and 23 rows in
   `crates/kerotakis-core/src/phase_route.rs` state they agree with NIST
   Chemistry WebBook SRD 69. `CONTRIBUTING.md` §3 names NIST SRD/WebBook as a
   legal constraint. `PLAN.md:1562` promises a `kero provenance lint` that
   "checks … nothing ships from an avoid-row source"; **that check is not
   implemented** — `provenance.rs`'s `direct_licence_allowed` is an allowlist
   over declared licences only, and no avoid-list of source names exists in
   code. Worth an owner ruling on whether "agrees with" is a citation.
3. **The one tool that asks whether a script can answer its own question is
   not run.** `crates/kerotakis-codex/src/curiosity.rs:119-120` points at it
   by name — *"whether a script can reach its own question is a different
   check with a different tool"* — but
   `tools/curiosity-answer-invariance.py` (127 lines) catches the
   `mat-012` class of defect — three vessels, three different contents, one
   identical answer, a row that matched its expectation for the corpus's whole
   life. It is referenced by nothing in `tools/preflight.sh` or
   `.github/workflows/`.
4. **Stale assertions in `crates/kerotakis-cli/tests/curiosity.rs`.** Line 44
   says two rows are missing, `aq-085` and `mat-054`; `aq-085` moved to
   computed in #547. `assert_eq!(failed, 7)` at line 143 is stale against the
   current baseline.
5. **`mat-054` has no `expected` field at all**, so it asserts no requirement.
   The "499 of 500" figure counts a row that, by the corpus's own rules,
   requires nothing of the bench.
6. **12 % of the corpus never executes.** The 60 `expected = "boundary"` rows
   short-circuit at `coverage.rs:514` before the script runs, and always pass.
   `curiosity.rs:178-190` records that `bio-075` declared `unknown_species`
   while its real blocker had become an unsupported `wait 7d`.
7. **193 of 500 rows carry no `expected`**, so `meets_requirement` is never
   called for them; and the fourth fallback arm at `coverage.rs:856` files a
   row as `Computed` on *any* typed event from *any* solver with no successful
   route — 65 of the 500 baseline rows.
8. **Five grammar verbs appear in zero of the 113 lessons** — `smell`,
   `magnet`, `irradiate`, `centrifuge`, `discard` — and twelve appear in two
   or fewer. Nothing reports this. `crates/kerotakis-cli/tests/cabinet.rs:191`
   exists because KID-17 found the same class of gap at the help-text layer.
9. **`web/app/src/lib/affordances.json` marks `stock` and `extract` as
   `planned:GUI-033`**, while `ROADMAP-GUI.md:1589` lists GUI-033 among
   completed tasks.
10. **Rust catalog ids (22 apparatus + 14 instruments = 36) are never
    cross-checked against `EQUIPMENT_CATALOGUE`'s 34 gated ids.**
    `web/app/src/lib/equipmentCatalogue.test.ts:38` compares the TS list to
    other TS lists only.
11. **The three roadmap files are 6–7 days stale** relative to `PLAN.md` and
    `HISTORY.md`, which move daily: `ROADMAP-GUI.md` 40 open items (last
    touched 2026-09-07), `BREADTH.md` 27 (2026-09-07), `CAPABILITIES.md` 7
    (2026-09-06). 74 open interface and breadth items in total.
12. **Two shards use a different TOML style.** `materials-handling.toml`
    writes `id="mat-001"`; the other three write `id = "aq-001"`. The corpus
    README records that a grep against the spaced form once "silently missed
    44 rows in that one shard."

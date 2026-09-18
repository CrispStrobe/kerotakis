# Would the suite notice if the engine were wrong?

*An instrument, its first measurement, and — at equal length — what that
measurement cannot say.*

Three times in the last month a check here read stronger than it was, and each
was found by accident rather than by any instrument:

* a Nernst-Planck solver whose forty-eight point test grid stopped **exactly**
  where the solver stopped converging, so the one interesting point could not
  be asked;
* a prose lint whose headline rule reached one quest file of thirty-six;
* a buoyancy assertion that a deliberately broken engine **passed**, because it
  read the scene's position while the comparison is made twice.

The third is this file's seed, and the agent that found it drew the distinction
everything below is built on:

> **Mutating a test shows an assertion is live. Mutating the engine shows
> whether the assertion is aimed at code that could break.**

Only the second found the gap. It did it by hand, for one case. Nothing in the
tree does it systematically, so nobody knows which of the thousands of
assertions here would notice if the code underneath them changed.

This is the sibling of two instruments already in the tree and it asks a
different question from both. `tools/curiosity-answer-invariance.py` perturbs a
**vessel** and asks whether the answer moves. `crates/kerotakis-cli/tests/perturbation.rs`
perturbs an **input** and asks whether the consequence moves. This one perturbs
**the engine** and asks whether anything at all complains.

---

## 1. What was evaluated, and what was built

`cargo-mutants` is the obvious candidate, is permissively licensed, and does
the textbook thing: edit one line, rebuild the crate, run the suite, repeat.
It was rejected on cost, not on quality, and the arithmetic is worth writing
down because it is the whole reason this harness looks the way it does.

| | `cargo-mutants` | this harness |
|---|---|---|
| builds | one per mutant | one, total |
| `kerotakis-core` | 71 000 lines; a cold build of it plus the CLI test binaries is minutes on four shared cores | same build, paid once |
| 105 mutants | 105 rebuilds | 105 test runs |
| mutants killed by the compiler | counted, and they are not evidence about the tests | structurally impossible |
| reaches `const` tables | yes | no — catalogued, run separately |

So this harness uses **mutant schemata**. Every mutation site is rewritten once
into a call that consults `KERO_MUTANT` at run time and returns either the
original value or the perturbed one:

```rust
//   if volume_ml <= 0.0 {
if crate::__mutation::cond(1, (volume_ml <= crate::__mutation::num(2, 0.0))) {
```

One build, N runs. The CLI integration tests inherit the selection for free,
because they spawn `kero` as a child process and a child inherits the
environment. Three consequences, two of them advantages:

* **No mutant can be killed by a compile error.** Every mutant in the
  catalogue compiles by construction, so the score cannot be inflated by the
  type checker doing the test suite's job. This matters more than it sounds:
  in a published `cargo-mutants` run a large share of "caught" mutants are
  caught by `rustc`, and counting those is one of the ways a mutation score
  flatters a suite.
* **The cost collapses** from N builds to one build plus N test runs, which is
  what makes any measurement possible on this box at all.
* **Only expressions can be schema'd.** A number inside a `const` table is
  evaluated at compile time and cannot read an environment variable. Those
  sites are catalogued as `table` mutants, and a sample of them was run the
  slow way instead — see §8a.

The harness is `tools/mutation/mutate.py`. It instruments, runs, restores, and
reports; the instrumentation is transient and is never committed.

## 2. The surface, and why this one

The workspace is out of reach and a chosen surface is not. The surface is three
modules of `kerotakis-core`, 2 033 lines:

| module | lines | what it turns into a shipped answer |
|---|---:|---|
| `buoyancy.rs` | 198 | a liquid's density, and whether a solid floats in it |
| `conductivity.rs` | 799 | specific conductance from the solved speciation, via a measured λ° table |
| `properties.rs` | 1 036 | temperature-dependent property correlations, each with a source and a validity range |

They were chosen together because they are one **kind** of code: the readout
layer, where a curated, sourced constant becomes a number an instrument
displays. All three of the month's incidents had that shape — a displayed
quantity plausibly wired to the wrong input — and the buoyancy module is
literally the seed of this task. They are also pure Rust with no C
dependency, which keeps the one build affordable, and they carry both unit
tests and integration coverage, so the ladder in §4 has something to measure at
every rung.

What this choice gives up: the solver entry points, which are where a wrong
answer would be most expensive, and which are also where a mutant is most
likely to hang rather than fail. That is the right thing to point it at next
(§9), not first.

## 3. The operators

| operator | what it does | why |
|---|---|---|
| `branch` | negates the condition of an `if` | the classic; a branch nothing distinguishes is a branch no test reaches |
| `constant` | moves an in-code `f64` literal by +25 % (0.0 becomes 1.0) | the incidents were all numbers, not control flow |
| `table` | the same, on a literal inside a `const`/`static` item | the curated tables whose values feed shipped answers |

`0.0` becomes `1.0` rather than `0.0 × 1.25`, because zero times anything is
still zero and a mutant that changes no value proves nothing about anything.
That case is not cosmetic: in this surface most zeros are guard thresholds, and
moving a guard from zero to one is a real change in behaviour.

Deliberately absent: statement deletion, operator swaps inside expressions,
and `while` conditions. The first two need a real parser and the third invites
infinite loops. A smaller operator set that always compiles is worth more here
than a larger one that needs a human to triage its wreckage.

## 4. What counts as caught

A single mutation score is the thing to distrust, because the easiest ways to
raise it are the ways that mean least. So the harness does not produce one
number; it produces a **ladder**, and the rung a mutant dies on is the result.

Mutants are run against the rungs in this order — cheapest first, and *the
weakest evidence last on purpose*:

| rung | suite | what killing a mutant here is worth |
|---|---|---|
| `unit` | `kerotakis-core --lib` | a unit test noticed. Middling: it may be asserting a value it was blessed with. |
| `property` | `perturbation.rs`, `metamorphic.rs` | **strongest.** These assert a relation — a quantity must move when its stated cause moves — so they cannot be satisfied by blessing whatever the engine says. |
| `golden` | the curiosity corpus (`kero coverage curiosity --check`) | **weakest.** A blessed-answer corpus notices *any* digit that moves, including a correction. |
| `hang` | any rung, by timeout | the runner noticed; no assertion did. |
| `survived` | — | nothing noticed. |

The ordering is the load-bearing part. Because the corpus runs **last**, a
mutant labelled `golden` is one that *nothing but a blessed value* noticed, and
that is the fact worth reporting. The converse is a limit and is stated as one
in §6: because the unit rung runs **first**, `unit` means *at least* the unit
tier noticed — not that the property tier would not have.

## 5. The first score

**105 mutants on 2 033 lines. 88 noticed, 17 survived.** Measured 2026-09-16;
the per-mutant record is `tools/mutation/results/2026-09-16.json`.

| rung a mutant died on | mutants | share |
|---|---:|---:|
| `unit` — `kerotakis-core --lib` | 56 | 53 % |
| `integration` — nine targeted `kerotakis-core/tests/` binaries | 32 | 30 % |
| `property` — `perturbation.rs`, `metamorphic.rs` | **0** | 0 % |
| `golden` — the curiosity corpus | **0** | 0 % |
| `hang` — killed by timeout | 0 | 0 % |
| **survived** | **17** | **16 %** |

| by module | noticed | |
|---|---|---:|
| `properties.rs` | 41 / 46 | 89 % |
| `conductivity.rs` | 36 / 45 | 80 % |
| `buoyancy.rs` | 11 / 14 | 79 % |

| by operator | noticed | |
|---|---|---:|
| `branch` — negate an `if` | **32 / 32** | 100 % |
| `constant` — move an f64 by +25 % | 56 / 73 | 77 % |

### The three things this run says that a single number would have hidden

**(a) Every branch mutant was caught; every survivor is a number.** The suite
covers this surface's control flow completely and its arithmetic unevenly.
That is precisely the profile of the three incidents that commissioned this
instrument: all three were numbers, none was a branch.

**(b) The blessed-answer corpus killed nothing.** Not one mutant reached the
corpus rung and died there. Every mutant the corpus would have caught was
already caught by a cheaper rung. On this surface the corpus adds no
sensitivity at all — which is not an argument for deleting it, since it exists
to pin answers rather than to find faults, but it is an argument against ever
quoting corpus coverage as evidence that the engine is checked.

**(c) The property oracles killed nothing either.** `perturbation.rs` and
`metamorphic.rs` are six days old and are the strongest instruments in the
tree, and on this surface they caught nothing that the crate's own integration
tests had not already caught — at 47.6 s a run against 1.6 s. Read carefully:
this does **not** say those tests are weak. They ran third by cost, and four
mutants (#0, #1, #4, #5) were killed by them in the first pass, before the
integration rung existed. What it says is that on *this* surface their reach is
already covered, and that pointing the next audit at the surfaces those oracles
were actually written for is the way to find out whether they bite.

### The correction, recorded rather than quietly fixed

**The first pass of this harness scored 43 % survival, and it was wrong.**
`cargo test -p kerotakis-core --lib` runs only the tests inside `src/`.
`kerotakis-core` also has **116 integration binaries** under `tests/` — one of
them `tests/buoyancy.rs`, the dedicated suite for a module in this very
surface — and the first ladder silently excluded every one. Adding that rung
moved 32 of 45 survivors into "caught".

This is the fourth instance this month of an instrument reading differently
from what it measured, and the first one in the instrument built to find the
other three. It is left in the record because the failure mode is the lesson:
`--lib` is a silent filter, and a mutation score is only ever a score of *the
tests you actually ran*.

## 6. The survivors, classified

The raw count is 17. **The count that means anything is 8**, because a
survivor is not automatically a weak test (§7.1) and telling the cases apart
needs a human. Every one is classified below.

### Real blind spots worth acting on (8)

| # | site | what the mutant does that nothing notices |
|---|---|---|
| 178 | `properties.rs:470` | The provenance note reports `σ = {sigma_n_m * 1e3} mN/m`. Move that conversion and the **note disagrees with the value the function returns**, silently. This is the third incident's exact shape — the picture saying one thing and the sentence another — reproduced in a module that is otherwise 89 % covered. |
| 63 | `conductivity.rs:316` | `covered_charge_fraction` falls back to `1.0` when nothing is charged. Make it `1.25` and the engine reports **125 % of the charge covered**. Nothing asserts that a fraction is a fraction. |
| 76 | `conductivity.rs:418` | `span_s_per_m` returns `(1.0/upper, 1.0/lower)`. Mutant #75 — the *first* of those two — is caught; #76 is not. **One end of the reported conductance span is asserted and the other is not.** |
| 98 | `properties.rs:39` | The upper edge of a stated validity range. The module's headline claim is "outside the range the function returns `Err` — a refusal, not a guess". The refusal's boundary is where that claim lives, and moving it changes nothing any test can see. |
| 3 | `buoyancy.rs:88` | `mass_g` starts at one gram instead of zero: every liquid density reading gains a gram. For a 100 mL beaker that is a 1 % error in a number a hydrometer is supposed to agree with, and the density assertions are looser than that. |
| 56 | `conductivity.rs:292` | `kappa_us_cm` starts at 1 µS/cm: a constant offset on every Kohlrausch reading. |
| 50 | `conductivity.rs:193` | The numerator of `concentration_factor`. `.clamp(0.0, 1.0)` absorbs the change in dilute solution, so this survivor says the attenuation fit **is never asserted at a concentration where it does anything** — and its own docs cite 0.63 at 1.7 mol/kgw. |
| 55 | `conductivity.rs:281` | The `concentration_factor` reported alongside a mean-mobility estimate. A metadata field on the fallback path that no test reads. |

### Genuine, but guard thresholds at extremes nothing exercises (4)

`#2` and `#7` (`buoyancy.rs`, a sub-millilitre vessel), `#79` (a vessel holding
more than a litre), `#82` (a portion of more than one unit). In each case the
*branch* is covered — the paired branch mutant was caught — and only the
threshold is not.

### Semantically null: the mutation changes no behaviour (3)

`#67` and `#73` widen a 0.01 K tolerance on "is this at 25 °C" to 0.0125 K.
`#157` moves a bisection convergence epsilon from 1e-15 to 1.25e-15. No test
could distinguish these, and no test should have to. **They are the reason a
raw survivor count must never be quoted as a blind-spot count.**

### Cannot be told apart by this instrument (1)

`#181`: the default contact angle for `capillary-rise` when `theta` is not
supplied, 0° becoming 1°. That is a 0.015 % change in a cosine — too small to
detect — *and* it may be that no caller ever omits `theta`, in which case the
line is dead. A +25 % operator cannot separate "untested" from "dead" here,
and saying which would take reading the callers, not running the harness.

### Nothing here was fixed

Per the brief these are recorded, not repaired: each fix is a new test, not a
line, and a test written by the agent that chose the mutant is a test written
to pass.


## 7. What the number cannot tell you

This repository has three incidents of a gate reading stronger than it was, so
the limits are part of the deliverable rather than a later discovery.

1. **A survivor is not automatically a weak test.** It is one of three things,
   and telling them apart needs a human: the tests are blind here; the line is
   dead; or the mutation was semantically null (a `+25 %` on a value that is
   subsequently normalised away, a branch both sides of which do the same
   thing). Every survivor is classified by hand in §6 into one of the
   three, and the count of *genuine* blind spots is smaller than the count of
   survivors. **Quote the classified number, never the raw survivor count.**
2. **The score is bounded by the surface, and the surface was chosen.** It says
   nothing about the solver, the corpus router, the kinetics integrator, or the
   web layer. A score of 80 % on 2 033 lines is not a statement about 71 000.
3. **The score is bounded by the operators.** Three operators that always
   compile cannot express the defect that started this: a comparison made
   against the wrong *object* is a data-flow error, and no arithmetic
   perturbation of a constant produces it. **This instrument would not have
   found the buoyancy bug it was commissioned by.** It finds a different and
   larger class — assertions aimed at code that cannot break — and that is the
   claim it is allowed to make.
4. **`table` mutants are not in the §5 score.** The λ° table and the property
   correlations' coefficients are `const` data, which schemata cannot reach.
   Seven of the 77 were run the slow way (§8a) and three survived, so the 84 %
   in §5 is a figure for the code around the curated numbers and not for the
   curated numbers themselves. **The two scores must never be added together
   or quoted as one.**
5. **The rung is the cheapest that noticed, not the strongest.** See §4.
6. **The instrumented tree is not the shipped tree.** Every site becomes a
   function call. The baseline run — the whole ladder, instrumented, with no
   mutant selected — must be green before any mutant means anything, and the
   harness refuses to score if it is not. But `#[inline]` on a call the
   optimiser can fold is not a proof of identical behaviour, only good
   evidence.
7. **It measures noticing, not correctness.** A suite that catches every
   mutant can still be asserting the wrong thing, in unison, everywhere.

## 8. Cost, and whether this belongs in the gate

Measured on the box this ran on: four cores, shared with another agent's build
for most of the run.

| | measured |
|---|---|
| build `kerotakis-core --lib` cold | ~30 min (heavily contended) |
| build the CLI test binaries | 4 m 43 s |
| build nine targeted `kerotakis-core` integration binaries | 48 s |
| **rebuild after instrumenting** (core + CLI) | **1 m 27 s** — paid once |
| clean run of each rung | 4.4 s / 1.6 s / 47.6 s / 11.8 s |
| **105 mutants, test execution** | **53 min** |
| per mutant: caught at the first rung | 5.0 s |
| per mutant: survives to the bottom | 59.6 s |
| per mutant: mean | 29.9 s |

**What the schema saved, measured rather than asserted.** A recompile-per-mutant
tool must rebuild `kerotakis-core` and relink every test binary in the ladder
for each mutant — fourteen binaries here. That cycle did not have to be
estimated: the seven `table` mutants in §8a have to be run exactly that way,
and their builds averaged **164 s** (157.9–171.9 s, tightly clustered). With a
mean 30 s of testing on top, a recompile-per-mutant pass over the same 105
mutants is **≈ 5 h 40 m**, against **one 1 m 27 s build plus 53 minutes of
running ≈ 55 minutes**. A **6.2× saving**, and the difference between a
measurement that happened today and one that did not. This is why
`cargo-mutants` was evaluated and not adopted: the tool is fine, the
arithmetic is not.

### Gate or audit

**A periodic audit.** Not a gate. A 53-minute pass on 2 033 lines of a 71 000
line crate cannot sit in front of a pull request, and making it faster by
shrinking the surface would produce a number too small to mean anything.

Three shapes that *are* affordable, in increasing ambition:

1. **A regression gate on the recorded survivors, not on the score.** The eight
   blind spots in §6 are a fixed list of eight mutant ids. Re-running them is
   eight ids against the two cheap rungs — **under a minute**, once the tree is
   built. When somebody writes the test that closes one, that mutant must flip
   from `survived` to `caught` and stay there. This is the only part of this
   work that belongs anywhere near CI.
2. **An audit when a module gains an oracle.** §5(c) is the argument: a new
   oracle suite claims a surface, and this is the instrument that says whether
   the claim bites. Run it against the surface the oracle was written for, on
   the day the oracle merges, when somebody still remembers what it was for.
3. **A quarterly pass on a rotating surface**, one crate-module family at a
   time, recorded the way this one is.

## 8a. The `const` tables: a sample, run the slow way

77 of the 182 catalogued sites are numbers inside `const` items — the λ° table
of measured limiting molar conductivities, the Bradley–Pitzer and Korson
coefficients, the IAPWS surface-tension constants. A `const` is evaluated by
the compiler and cannot read an environment variable, so **schemata cannot
reach them and they are not in the §5 score.** This is the most uncomfortable
gap in this run, because a curated sourced constant feeding a shipped answer is
exactly the kind of value the three incidents were about.

They can only be run by rebuilding, at 164 s of build a cycle. Seven were run
that way — about 27 minutes — rather than leaving the question blank. **Three
of the seven survived**, and which three is the point:

| # | constant | outcome |
|---|---|---|
| 14 | λ°(H⁺) = 349.65 | caught (integration) |
| 43 | `FIT_SQRT = 0.5324` | caught (unit) |
| 160 | `IAPWS_ST_B = 235.8` mN/m | caught (unit) |
| 169 | `STANDARD_GRAVITY = 9.806_65` | caught (unit) |
| **23** | **λ°(Ba²⁺) = 127.2** | **survived** |
| **41** | **λ°(MnO₄⁻) = 61.3** | **survived** |
| **125** | **Henry `c_kelvin = 2400.0`** | **survived** |

**The λ° table is verified for the ion that appears in every test and not for
the ions that do not.** Hydrogen is pinned; barium and permanganate can each be
a quarter wrong and the conductivity meter will report it without a murmur.
The table's own doc comment says "the λ° table is measured data, not theory" —
and a measured value that no test can distinguish from a wrong one is, for the
suite's purposes, not distinguishable from a placeholder. That is the same
shape as the finding recorded in `buoyancy.rs`'s own comments about ion
densities: *a placeholder that produces a believable number is the hardest kind
to see.*

This is a 7-of-77 sample chosen to contrast a common ion with rare ones, so it
is an illustration, not a rate. The rate is what §9.1 proposes measuring.

**One of those three survivors was closed on 2026-09-17 and the other two were
not**, and the split is the one §8c ends on. λ°(Ba²⁺) is now checked
against an independent compilation; **λ°(MnO₄⁻) is not, because permanganate is
one of the seven ions no second source could be reached for**, and the Henry
coefficient is in `properties.rs` and was not part of that work at all. Nothing
has been written that could distinguish #41 or #125 from a value a quarter
wrong.

## 8b. The `const` tables, all of them: the sample was right

§8a ran seven of the 77 and called itself an illustration rather than a rate.
The rate is now measured. **70 mutants, 36 caught, 34 survived**, about four
and a half hours of rebuilds. The split is the finding:

| file | run | caught | survived |
|---|---|---|---|
| `conductivity.rs` | 28 | 5 | **23 (82%)** |
| `properties.rs` | 42 | 31 | 11 (26%) |

**Five of the 28 conductivity constants are watched, and here is the whole
list of them:** λ°(H⁺), λ°(OH⁻), λ°(Na⁺), λ°(K⁺), λ°(Cl⁻) — and `FIT_LINEAR`.
The first five are the ions in table salt and in the acid and base every
lesson pours. Everything else in the table can be a quarter wrong and the
conductivity meter reports it without a murmur: lithium, ammonium, silver,
calcium, magnesium, strontium, copper, zinc, iron(II), iron(III), aluminium,
manganese, lead, bromide, iodide, fluoride, nitrate, perchlorate,
bicarbonate, carbonate, sulfate, and both concentration limits.

§8a guessed this from three rare ions. At full size it is not a tendency but
the structure of the table: **the λ° table is verified exactly where the
lessons happen to go.** A learner who dissolves Epsom salt is reading a
number with no test behind it, and the reading looks precisely as confident
as the one for sodium chloride.

`properties.rs` inverts it — 74% caught, because Henry's-law constants and
the Korson and Bradley–Pitzer coefficients feed relations the suite already
checks. The eleven survivors there are mostly `c_kelvin` temperature
coefficients for gases no lesson dissolves, which is the same shape again.

### What this run cost that the sample did not

Two SIGKILLs under memory pressure, and each one **left a falsified constant
live in the worktree**. A table mutant edits a `const` literal, rebuilds,
runs the tiers, and restores the line in a `finally`; a kill skips the
`finally`. Both times it was a Bradley–Pitzer coefficient sitting 25% wrong
on disk, caught in `git status` and reverted. The finishing run wrapped the
harness in `trap 'git checkout -- <files>' EXIT HUP INT TERM`, which is the
guard this operator has always needed and should carry itself: a mutation
harness is the one tool here whose normal operation makes the engine wrong,
and "wrong until I choose to fix it" is not safe on a shared machine.

The run was also interrupted mid-way and finished in two halves. The halves
were joined **only after checking that the catalogue reproduces its ids** —
all 62 verdicts from the first half match their catalogue entry's file and
line exactly. Without that check, appending under regenerated numbering could
have filed a verdict against the wrong constant, which is a worse outcome
than an unfinished measurement.

## 8c. What closing the conductivity survivors took

§8b measured the gap. This section is what was done about the 23 in
`conductivity.rs`, what it closed, and — at equal length — what it could
not.

### What was done about it, and why not the obvious thing

The obvious repair is 23 assertions. The owner refused it in one sentence — *a
test written to kill a mutant asserts only that a number is the number it is* —
and asked instead for the values to be traced to sources and for **one test
that covers the class**. `CONTRIBUTING.md` §4 already says why, and says it as
the deliberate opposite of the engine's own rule: an engine carrying the answer
it should compute has stopped computing, while a test carrying a *cited
external measurement* is doing the one thing the engine cannot do for itself.

`crates/kerotakis-core/tests/conductivity_sources.rs` is that test file. Three
instruments, strongest first:

1. **The OIML calibration ladder.** Six potassium chloride reference
   solutions, spanning 0.001 to 1 mol/kgw, from OIML R 56 (1981) — Jones and
   Bradshaw's 1933 primaries as corrected to the absolute ohm, and
   Shedlovsky's 1932 secondaries. This is not anyone's compilation; it is what
   a conductivity meter is calibrated against, and it exercises the λ° table,
   the fitted attenuation and the mol/L ≈ mol/kgw approximation in one number.
   The model reads +0.44 %, +0.52 %, +0.71 %, +1.09 % and +0.52 % high across
   the five dilute rungs, and **−6.4 % at the 1 D standard** — where two of the
   module's admitted approximations have stopped holding at once. The test
   allows ten times more error at that rung and flips the direction it
   requires, which is the cost of those two admissions, measured.
2. **The table against two independent compilations.** 21 of the 28 ions
   against USGS Water-Supply Paper 2311 (a public-domain United States
   Government report, itself naming Harned and Owen p. 231 and Franks p. 178,
   and printing its own international-to-absolute-ohm conversion) and a
   Sartorius handbook read at one remove. Neither is the compilation the
   shipped table was transcribed from, so agreement is corroboration and not a
   round trip — the same distinction `LAMBDA_SOURCE` already makes at length
   about Nernst-Einstein and `phreeqc.dat`.
3. **Relations that are physics** — Grotthuss hopping puts H⁺ and OH⁻ beyond
   twice the fastest ordinary ion per unit charge, and hydration reverses the
   alkali ordering so that Li⁺ < Na⁺ < K⁺.

Plus the two boundaries, each tied to the last measurement that supports it
rather than to itself: the dilute limit is bracketed at ±1 % around the 0.1 D
primary standard it coincides with, and the fitted range is asserted to end
above the most concentrated solution `FIT_SOURCE` says the two coefficients
were fitted to.

### What it did not close, which is the finding

**Seven ions have no second source that could be reached at all**: Zn²⁺, Fe²⁺,
Fe³⁺, Al³⁺, Mn²⁺, Pb²⁺ and MnO₄⁻. The USGS table does not carry them; the
Sartorius table does not carry them; Wikipedia's Adamson-sourced table does not
carry them. What does carry them is Vanýsek's table in the CRC Handbook, which
is the source the shipped values name and which `provenance/upstreams.toml`
refuses as a systematic source. So six of the 23 survivors survive **because
this repository has one source for those numbers and no way to check it**, and
that is a more useful thing to know than a mutation score.

They are listed by name in `UNCORROBORATED` in the test file, and the class
test asserts the list is exactly right: every shipped λ° is either checked
against an independent compilation or written down as resting on one. A new
ion cannot be added in silence, and finding a source for one of these seven is
a one-line deletion.

### The re-run: 17 of 23

The same 23 ids, the same catalogue, run again on 2026-09-17 —
`tools/mutation/results/2026-09-17-conductivity-rerun.json`, and on a GitHub
runner rather than here, for the reason this section's last part gives.

| | mutants | |
|---|---:|---|
| caught by `the_lambda_table_agrees_with_independent_compilations` | 15 | the corroborated ions |
| caught by `the_dilute_limit_is_the_last_primary_standard` | 1 | `DILUTE_LIMIT_MOLAL` |
| caught by `past_the_last_fitted_measurement_the_estimate_admits_extrapolating` | 1 | `FITTED_LIMIT_MOLAL` |
| **survived** | **6** | Zn²⁺, Fe²⁺, Fe³⁺, Al³⁺, Mn²⁺, Pb²⁺ |

**17 of 23, and the six that survive are exactly the six the test file names as
uncorroborated.** That correspondence is the useful part of the number: the
instrument and the provenance record agree about which values are unchecked,
so the survivor list is not a list of untested code but a list of *unsourced
data*, and it will shrink only when somebody finds a second source — not when
somebody writes another assertion.

Two of the caught ones are worth separating from the other fifteen. λ°(SO₄²⁻)
is caught twice, by the table comparison and independently by the Grotthuss
relation, because a quarter more sulfate mobility would put an ordinary ion
within a factor of two of the hydroxide. And the two range boundaries are
caught by tests that do not mention their values at all: each is bracketed
against the measurement that justifies it — the 0.1 D primary standard, and
the most concentrated solution the correction was fitted to.

### The run that reported 23 of 23, and why that number is not in this document

The first CI pass said **23 caught, 0 survived**. It was wrong, and it was
wrong in this instrument's own characteristic way.

Six mutants — the six above — were recorded as caught by the `cli-property`
rung, with three real test names under them, parsed out of a real `failures:`
block. The rung had taken **0.1 seconds**. It takes 34 at baseline.

`run_tier` set `TMPDIR` to `/mnt/volume1/tmp-overflow/kero-build` when the
caller had not set one: this project's development box's overflow directory,
hardcoded into the tool. On a GitHub runner that path does not exist,
`std::env::temp_dir()` inside `metamorphic.rs` returned it anyway,
`create_dir_all` failed with `PermissionDenied`, and all three metamorphic
tests panicked in their first statement — which looks, in a results file that
records only names, exactly like three assertions firing.

**That is the fifth instance in this document of a check reading stronger than
it was, and the first that would have been published as a result.** What found
it was arithmetic, not an assertion: 0.1 s is not a suite noticing something.
What proved it was a change made in response — a failing tier now keeps its
exit code and the tail of its output, so a verdict carries the evidence under
it and "an assertion fired" can be told from "the rung could not start".

The lesson generalises past this bug, and it is the same one §4 makes about
the corpus rung: **a mutation harness cannot distinguish a test that noticed
from a test that could not run, unless it is built to.** A false NEGATIVE in
this instrument — a mutant wrongly called survived — costs an investigation. A
false POSITIVE costs a claim, and claims are what this document is for.

### A value that disagrees with its source, reported and not changed

`FIT_SOURCE` says: *"1413 µS/cm for the 0.01 mol/kg KCl calibration standard is
the IUPAC/OIML reference value, a metrological convention rather than anyone's
compilation"*. OIML R 56 was read in full on 2026-09-17. **It prints 0.14083
S/m — 1408.3 µS/cm — for its 0.01 D primary standard, and 1413 appears nowhere
in it.** USGS WSP 2311's table 1, reporting the same Jones and Bradshaw data,
gives 1408.07 µS/cm for 0.01 D and 1410.75 for 0.01 N.

1413 is not invented: it is very close to the conductivity of a 0.0100 **mol/L**
KCl solution, which contains about 0.3 % more salt per kilogram of solution
than the 0.01 D standard does, and which is what a bottle of calibration fluid
sold as "1413 µS/cm" contains. So the number is a real standard value on a
volumetric basis, attributed here to a molality basis and to an organisation
whose published value is a different number.

**It is not changed.** The rule is that a value disagreeing with its source is
a finding to report, not a line to edit — this project has been burned by a
value corrected in the wrong direction — and the existing unit test
`kcl_calibration_standard_within_model_error` asserts against 1413 with a 7 %
window that the OIML figure also sits inside. Whoever fixes it should decide
whether the citation or the basis is what moves.

### What the provenance audit says about all this

`kero provenance upstreams` moves from **67 findings over 397 value-bound
strings to 68 over 405** — eight new citations and exactly one new finding,
which is the Sartorius handbook declared `claims` with no locator because no
copy of it was opened. That is the honest cost of corroborating five ions
through a reproduction of a book, and it is recorded as a finding rather than
argued away. The OIML rows (six) and the Wikipedia route (one) are reported
apart from the headline, as `decision-required` and `oracle-only` respectively.

**The vocabulary was missing a word and this is where it showed.** The role
that fits "read at one remove" is `via`, and `via` is defined as a value read
through a *refused* source's rendering of a *cleared* one. Here it is the other
way round: the work behind the value has no licence and the route — a CC BY-SA
encyclopaedia article — is clear. The lint said so itself, refusing the first
draft of the row with *"upstream 'sartorius-elektroanalytik' is
decision-required, which refuses nothing — the judgement judges nothing"*. The
citation therefore says the true, smaller thing (`claims`, the value IS the
handbook's) and names the route in prose. Worth a word in the vocabulary, not
a fudge in the row.

**A blind spot found on the way, reported and not repaired.**
`conductivity.rs`'s `LAMBDA_SOURCE` — the string that says the entire λ° table
came from the CRC Handbook, which this repository refuses as a systematic
source — **does not appear in that lint's output at all.** The scanner finds a
citation by looking for a FIELD named `source` (or `provenance`, `citation`, …)
followed by a string literal, and `const LAMBDA_SOURCE: &str = "…"` puts a type
where the quote should be. The same is true of `FIT_SOURCE`. Making them
visible would add findings to a reviewed count, which is the owner's call and
not a test file's; the new test file declines to add a second invisible one and
says why in the code.

### The two operator hazards §8b named, closed in the tool

**A table mutant lives in the working tree**, and §8b records what that cost:
two SIGKILLs under memory pressure, two Bradley–Pitzer coefficients left 25 %
wrong on disk, and a `trap 'git checkout -- <files>' EXIT HUP INT TERM` around
the finishing run. §8b also says where that guard belongs — *"the guard this
operator has always needed and should carry itself"* — so it now does.
`mutate.py` writes
`.mutation-state/table-live.json` *before* the edit and clears it after the
restore; every later invocation begins by putting back whatever that marker
still describes, loudly, and `mutate.py recover` does only that. SIGTERM,
SIGHUP and SIGINT are also caught now, so for every kill but SIGKILL the
existing `finally` simply runs.

**`run-table` needs a pristine copy** of each file in
`.mutation-state/orig/`, used only to turn a recorded byte offset into a line
and column. A fresh checkout has none, which is what made §8b's run awkward to
move anywhere else. It now seeds that copy from the working tree when git
agrees the file is unmodified, and refuses — rather than guessing — when it
does not.

### And the run moved off the box

`.github/workflows/mutation-tables.yml` runs the rebuild-per-mutant path on a
runner: catalogue, a green baseline on every rung before any verdict counts,
the ids as an input, and a step that fails the job if a falsified constant is
still on disk when it ends. `workflow_dispatch`, never `pull_request` — §8's
argument against putting this in front of a pull request has not changed, and
the reason it exists at all is that a 4½-hour rebuild loop on a box with a
gigabyte of free memory is a run that gets killed rather than a run that
finishes.

## 8d. Buying a second source for the six, 2026-09-18

§8c ended on six survivors that survive because **this repository has one
compilation for those numbers and no way to check it**. The owner's ruling was
to buy a second source for them. This section is what a day of looking bought:
**one ion corroborated, two disagreements to put in front of a human, one pair
of ions where the two second sources contradict each other — and the last six
conductivity survivors closed, 6 of 6 caught on a re-run.**

### The result, per ion, because they differ

Per equivalent, as all three tables print them.

| ion | shipped | Kreshkov 1970 p. 74 | Hübschmann & Links 1991 p. 62 | outcome |
|---|---:|---:|---:|---|
| **Fe³⁺** | 68.0 | **68** | **68** | **(1) CORROBORATED — left the list** |
| **Al³⁺** | 61.0 | **63** | **63** | **(2) both disagree, +3.3 %** |
| **Fe²⁺** | 54.0 | **53.5** | **53.5** | **(2) both disagree, −0.9 %** |
| Mn²⁺ | 53.5 | 53.5 | **50** | the two sources differ by 7.0 % |
| Pb²⁺ | 71.0 | **70** | **65** | the two sources differ by 7.7 % |
| Zn²⁺ | 52.8 | absent | **53** | reached at last; 2 s.f., cannot settle |
| MnO₄⁻ | 61.3 | absent | 61 | −0.5 %, just outside tolerance |

`UNCORROBORATED` goes from **seven to six**. λ°(Fe³⁺) leaves it.

### The one that left, and the caveat that rides with it

Two compilations, reached by two unrelated routes, neither of them the CRC
Handbook, both print **⅓Fe³⁺ = 68** — exactly the shipped 204.0/3. That is
what corroboration is, and iron(III) now has it.

**The caveat is in the `CORROBORATED` row itself and is not hidden:** both
print two significant figures, so "68" is anything in [67.5, 68.5] and this
cannot distinguish 204.0 from 203. What it establishes is the thing
`UNCORROBORATED` exists to deny — that the value rests on one compilation. It
no longer does.

### Why the other six are still uncorroborated, measured not asserted

Both new sources carry ions this repository has **already corroborated twice
over**, against USGS WSP 2311 and a Sartorius handbook. That overlap is not
decoration — it is the instrument. **Calibrate the source before trusting it.**

| source | overlap | inside 0.5 % | worst |
|---|---:|---:|---|
| Kreshkov 1970 | 21 | 18 | Cu²⁺ 113.2 vs 107.2, **+5.6 %** |
| Hübschmann & Links 1991 | 21 | 17 | CO₃²⁻ 148.0 vs 138.6, **+6.8 %** |

A source whose error on ions we *can* check runs to 6 % cannot settle an ion
we cannot check to 0.5 %. So the six get a **bound** rather than a
confirmation, and §2b of `conductivity_sources.rs` asserts exactly that — with
each bound *measured from its own calibration*, so it cannot be tuned to pass.

### Two values that disagree, reported and not changed

Both are stronger than a lone disagreement because **both coarse sources print
the same number independently**:

* **⅓Al³⁺ = 63** against the shipped 61 — molar **189 against 183, +3.3 %**
* **½Fe²⁺ = 53.5** against the shipped 54.0 — molar **107 against 108, −0.9 %**

**Neither is applied.** #586 corrected a silver ΔH_fus in the wrong direction
and #595 had to undo it. Here the case against acting is more specific than
that general caution: **both gaps are smaller than the same sources' own worst
error on ions two other compilations agree about.** Under their measured
fidelity these are not evidence that 183 and 108 are wrong; they are evidence
that two lineages print different numbers and none of the three has been traced
to a measurement. That trace is what would settle it.

### And two where the second sources contradict each other

* **½Mn²⁺**: Kreshkov 53.5 (equal to the shipped value to the digit),
  Hübschmann & Links **50** — 7.0 % apart
* **½Pb²⁺**: Kreshkov **70**, Hübschmann & Links **65**, shipped 71.0 — 7.7 %
  apart

Where two independent compilations cannot agree with each other, **neither
adjudicates a third value.** Both are recorded in `CONTRADICTED` in the test
file, which is asserted to be exactly right so a third cannot appear in
silence — the same guarantee `UNCORROBORATED` gives.

This also lands on §8c's own open question about Sartorius. Hübschmann & Links
carries all five ions the Sartorius row was the only route to, agrees on
silver and barium, and puts **carbonate at 148.0 against 138.6 — 6.8 %**. So
two of the twenty-two "corroborated" ions are corroborated by one source and
contradicted by another, which was not true when §8c was written.

### The survivor count: 6 of 6 caught, 0 survived

The bounds are far too loose to corroborate and nowhere near too loose to
notice a quarter. `a_quarter_wrong_would_fall_outside_the_measured_bound`
asserts exactly that, for **every** ion still in `UNCORROBORATED`, in both
directions — so the claim is checked **without a falsified constant ever
touching the disk**, the hazard §8b's two SIGKILLs left in the worktree.

It was written here as a prediction and then run. Same six ids, same
catalogue, on a runner — `tools/mutation/results/2026-09-18-conductivity-rerun.json`:

| | mutants | |
|---|---:|---|
| caught by `a_coarse_source_that_contradicts_a_shipped_value_is_named_and_counted` | 6 | all of them |
| *also* caught by `a_quarter_wrong_would_fall_outside_the_measured_bound` | 5 | every one but Al³⁺ |
| *also* caught by `the_lambda_table_agrees_with_independent_compilations` | 1 | Fe³⁺, the ion that left the list |
| **survived** | **0** | |

**§8b's 23 conductivity survivors are now 0.** The path was 23 → 6 (#625,
by sourcing 22 values) → 0 (here, by sourcing the last six). Not one of the
forty-odd assertions that closed them says that a number is the number it is;
every one compares the table against something measured outside this
repository.

**AND THE REASON THIS NUMBER IS BELIEVED, given §8c's own false 23-of-23.**
That failure looked like a rung taking 0.1 s when it takes 34. These rungs
take 0.2 s, which is the same shape — so the verdicts were read rather than
counted. Each carries `exit_code: 101`, a real `failing:` list naming real
tests, and an `output_tail` showing the neighbouring `buoyancy.rs` binary
running its five tests green and `conductivity_sources.rs` running its twelve
before one failed. The builds took 55 s each. It is a fast rung because these
tests are arithmetic over two `const` tables, not a rung that never ran.

**What it does NOT say** is that the six values are right. Five of them are
still in `UNCORROBORATED`, two of them are contradicted by one of the two
sources that reach them, and two more disagree with both. A mutation score of
100 % on this file now means the tree would notice a quarter — nothing more,
and §8d exists so that the difference stays legible.

### What was read, and what refused

Read in full on 2026-09-18, and **none of these carries any of the six**:

* **International Critical Tables VI (1929)** p. 230 table 3, "Ion
  conductances at 18 °C" — public domain, read from the Internet Archive. Of
  the six it has only ½Pb²⁺ = 61, **at 18 °C**; its own temperature
  coefficient would carry that to ≈71 at 25 °C, a 15 % extrapolation that
  settles nothing at 0.5 %.
* **Glasstone, *An Introduction to Electrochemistry* (1942)** table XIII p.
  56 — twenty-two ions, ends at ½Mg²⁺ = 53.06. Cites MacInnes (1939) p. 342.
* **Lange's *Handbook of Chemistry*, 7th ed. (1949)** p. 1417, attributing its
  table to Johnston, *J. Am. Chem. Soc.* 31 (1909) 1010 — fifteen rows; nine
  ion labels survive the scan and none is one of the six.
* **MacInnes, *The Principles of Electrochemistry* (1939)** ch. 18 — the
  scan's OCR could not resolve the table; read through Glasstone's
  reproduction of it rather than off the page, and recorded that way.
* **USGS WSP 2254** (Hem, 1985) — a chapter on conductance and no λ° table.
* **hydrochemistry.eu** (Appelo) — confirms in the author's own words that
  PHREEQC's specific conductance is computed from diffusion coefficients and
  checked against the Handbook. That is the round trip `LAMBDA_SOURCE` already
  refuses, now confirmed from the other end.
* **French Wikipedia's list of ionic molar conductivities** — prints all six
  to the digit, and its bibliography is Vanýsek, CRC Handbook 87th ed. **p.
  5-78**. Two printings of one table are one source, so this corroborates
  nothing; what it does do is pin the shipped table's provenance to a page.

**Not reached**, and recorded so the next attempt starts further on:

* **Robinson and Stokes, *Electrolyte Solutions*; Harned and Owen; Conway,
  *Electrochemical Data*** — all three are on the Internet Archive as
  lending-restricted items whose full-text search endpoint answers **HTTP
  403**. CHECKED AND NOT READ, in the words #600 had to be corrected into.
* **Johnston (1909)** itself — acs.org unreachable, as `acs-education`
  already records.
* The two books behind the one table that did reach all seven — **Hübschmann
  and Links (1991)** and **Milazzo (1952)**. Read at one remove through the
  German Wikipedia article that reproduces the table and names them, on the
  CC BY-SA route #625 already cleared for Sartorius. **No copy of either was
  opened**, which is why both citations declare `claims` with no locator and
  are findings by construction.

### The structural finding, which is the part worth keeping

Every precise λ° compilation in this family — USGS/Harned & Owen, Sartorius,
Glasstone, MacInnes, Lange's/Johnston — obtains an ion's λ° by splitting a
measured Λ° with a **transference number**, and every one of them stops around
magnesium. Transference numbers of that precision were never measured for
salts that hydrolyse, and Zn²⁺, Fe²⁺, Fe³⁺, Al³⁺, Mn²⁺ and Pb²⁺ all hydrolyse.

**The six are not absent from the good tables by accident. They are excluded
by the method the good tables use**, and they survive only in coarse handbook
tables that round to two or three figures — which is exactly why the two found
here disagree with each other by seven per cent about manganese and lead. So
looking harder in that family will not find them. What would settle these six
is the conductance literature for the individual salts, which is where "trace
original sources for almost all values, and cite those" points, and which is
behind the same publisher wall that stopped §8c.


## 9. What to point it at next

1. ~~**The rest of this surface's `const` tables.**~~ **DONE 2026-09-17, §8b.**
   70 mutants, 36 caught, 34 survived. §8a's guess was right and then some:
   23 of the 28 conductivity constants are unwatched. The owner's decision on
   what to do about them was **not** to pin each one — a test written to kill
   a mutant asserts only that a number is the number it is — but to trace the
   values to their sources and cover the class with a cited external
   measurement, which is what `CONTRIBUTING.md` §4 says a test is for. **That
   was done the same day and §8c is the account of it: 17 of the 23 were then
   caught, and the six that were not were the six ions this repository had one
   source for and no way to check. §8d closed those six on 2026-09-18 — 23 of
   23 — though five of them are still `UNCORROBORATED`, which is the
   distinction §8d exists to keep legible.**
2. **The surfaces the new oracles were written for** — adsorption,
   electrochemistry, polarization. §5(c) found that `perturbation.rs` and
   `metamorphic.rs` killed nothing here that was not already dead. That is a
   statement about this surface, and the only way to turn it into a statement
   about those instruments is to point the harness where they aim.
3. **The readback path in `vessel.rs` and `solve.rs`**, where the
   element-dropped-on-readback defect lived. It is a bigger surface and
   mutants there are far likelier to hang than to fail, which is why the
   timeout rung exists and why it should not be first.
4. **A fourth operator: swap a variable for another of the same type.** All
   three incidents were data-flow errors — a comparison made against the wrong
   *object*, not the wrong *number* — and §7.3 admits no operator here can
   express one. That operator needs a real parser, and it is the single change
   that would make this instrument able to find the bug it was commissioned by.


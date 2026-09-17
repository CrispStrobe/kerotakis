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

## 9. What to point it at next

1. ~~**The rest of this surface's `const` tables.**~~ **DONE 2026-09-17, §8b.**
   70 mutants, 36 caught, 34 survived. §8a's guess was right and then some:
   23 of the 28 conductivity constants are unwatched. The owner's decision on
   what to do about them was **not** to pin each one — a test written to kill
   a mutant asserts only that a number is the number it is — but to trace the
   values to their sources and cover the class with a cited external
   measurement, which is what `CONTRIBUTING.md` §4 says a test is for.
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


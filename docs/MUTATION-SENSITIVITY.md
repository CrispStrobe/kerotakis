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
  sites are catalogued as `table` mutants and are honest gaps in this run —
  see §7.

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

*Filled in from `results.json`; see §8 for the cost of producing it.*

## 6. What the number cannot tell you

This repository has three incidents of a gate reading stronger than it was, so
the limits are part of the deliverable rather than a later discovery.

1. **A survivor is not automatically a weak test.** It is one of three things,
   and telling them apart needs a human: the tests are blind here; the line is
   dead; or the mutation was semantically null (a `+25 %` on a value that is
   subsequently normalised away, a branch both sides of which do the same
   thing). Every survivor reported in §5 is classified by hand into one of the
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
4. **`table` mutants are not in the score.** The λ° table and the property
   correlations' coefficients are `const` data, which schemata cannot reach.
   They are catalogued and counted, and a subset was run the slow way; the
   rest are an unmeasured part of the very surface this run chose. That is the
   most uncomfortable limit here, because a curated table is exactly the kind
   of value the incidents were about.
5. **The rung is the cheapest that noticed, not the strongest.** See §4.
6. **The instrumented tree is not the shipped tree.** Every site becomes a
   function call. The baseline run — the whole ladder, instrumented, with no
   mutant selected — must be green before any mutant means anything, and the
   harness refuses to score if it is not. But `#[inline]` on a call the
   optimiser can fold is not a proof of identical behaviour, only good
   evidence.
7. **It measures noticing, not correctness.** A suite that catches every
   mutant can still be asserting the wrong thing, in unison, everywhere.

## 7. Cost, and whether this belongs in the gate

*Filled in from the measured run.*

## 8. What to point it at next

*Filled in.*

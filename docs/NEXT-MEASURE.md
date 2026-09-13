# The measure after the curiosity corpus

A proposal for the owner. Nothing here is implemented: no engine change, no
test, no new gate. It sets out three candidate successors to the curiosity
corpus with what each would cost, recommends one, and names the first step.
The decision is the owner's.

## 1. Why the corpus stopped discriminating

`docs/CURIOSITY-COVERAGE.md` records 499 of 500 prompts answered — 328
computed, 56 curated, 55 qualitative, 60 boundary, 1 missing. The one missing
row is `mat-054`, and it was judged unable to close honestly rather than left
undone. A measure that reads 499/500 cannot rank work any more; it has no
remaining shadow to point at.

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
| Source-informed fleets (`tools/chemistry-audit/source-fleets*`) | 856 | A relation between two **engine runs** |

The fleet analyser is the clearest case. `analyse_source_fleets.py` admits
exactly four relation kinds — `conservation`, `independent-law`,
`metamorphic`, `boundary` — and ten assertions, every one of which is
`…-equal` between two engine runs, `…-order` between two engine runs, or
`…-present`. `independent-law` is a misleading name: at line 381 it resolves
to `final-scalar-order`, an *ordering* between two of the bench's own results.
It proves a quantity moves the right way. It cannot prove it moves the right
distance.

`docs/SEMANTIC-ASSERTIONS.md` is the same shape at the catalog layer: `kind`
is `equal`, `increasing`, `decreasing`, `conserved`, `unchanged`, or `ratio`.
There is no kind that takes a number from outside the repository.

**So: 856 frozen fleet cases, 500 corpus prompts, and a per-reaction assertion
language, and not one of them compares a computed value to a published one.**

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

And there is a third, still open. `PLAN.md` lines 1832–1838 record that a 0.1
molal brine is still answered by a Debye–Hückel dataset and takes Raoult's
law: **−0.371 °C against a measured −0.346** — about seven per cent
optimistic, in the same direction the dilute law was. It is written down. It
is understood. Nothing fails because of it.

The measured reference values in all three cases live in commit messages and
in prose — `HISTORY.md` lines 124, 125, 139, 146 — where no gate can read
them. `−3.4`, `108.7`, `0.936`, `−0.346` are the numbers that mattered most to
a year of chemistry work, and they are stored as English.


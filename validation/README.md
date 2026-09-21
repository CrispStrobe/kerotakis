# The accuracy corpus

Rows of the form: *this script, run on this bench, produces this quantity;
the published value is X ± t, from source S.*

This is level five of the verification ladder in
[`ROADMAP-Webapp.md`](../ROADMAP-Webapp.md) — "experimental benchmarks …
with conditions and tolerances" — specified in 2026-08 and unbuilt until now.
It exists because the curiosity corpus stopped discriminating: it reads
essentially complete, and a corpus that nearly everything passes cannot tell
the owner what is wrong next. [`docs/NEXT-MEASURE.md`](../docs/NEXT-MEASURE.md)
argued for this successor and specified the first step.

**It is one family and six rows, and it is a backlog rather than a gate.**

## What is here

| | |
|---|---|
| `cases/colligative.toml` | four cases, seven anchored rows, no open row |
| `../crates/kerotakis-core/tests/accuracy_corpus.rs` | the structural lint — no engine, no accuracy gate |

**Quantities covered: 3** — `freezing_point_depression`,
`osmotic_coefficient` and, since 2026-09-21, `boiling_point_elevation`.

The boiling row closed the way the open ones are meant to: somebody read the
source. It had been open because 108.7 °C was believed to be a measurement of
a *saturated* brine while the case builds 6.000 mol/kg, so the two were held
to be answers about different solutions. Richmond's 1893 table says his
saturated solution is 28.7 per cent and boils at 109.5 °C; **108.7 °C is his
26.0 per cent row, which is 6.012 mol/kg.** The number had been a measurement
of very nearly this beaker the whole time, and the premise that withdrew it
was wrong.

Its band is the widest in the file at ±1.0 K, and deliberately: Richmond
states his results run about 1 °C above the only earlier series he names, and
that disagreement — not his tenth-of-a-degree reading — is what the world
knows here. **A model sitting inside a band that wide is weak evidence, and
the row says so in its own `tolerance_reason`.**

## The three rules this corpus is built on

### 1. Tolerance is argued per quantity, never a flat percentage

A pH, a freezing point and an adiabatic flame temperature do not deserve the
same bar. Every row carries **two** bands, generalising the only argued
tolerance this repository had before the corpus existed:

- **`model_tolerance`** — tight, on the bench's *own* figure. A pin. It moves
  when the engine, the dataset or the relation moves, and it is meant to. It
  is not a claim about the world.
- **`world_tolerance`** — loose, on the published measurement, set by the rule
  *wide enough to survive an improvement, narrow enough to exclude the
  known-wrong predecessor*. Every row names both the improvement and the
  predecessor, so the width can be checked rather than believed.

A `tolerance` and its `tolerance_reason` are **separate fields** on purpose.
The dominant risk in a corpus like this is a tolerance widened to make its own
row pass, and that is invisible in a count — but visible in a diff, if the
number can move without the argument moving with it.

Instrument precision is the right *axis* and the wrong *value*.
`instrument.rs` declares a thermometer at ±0.1 °C; model tolerance is how far
the model may sit from reality, instrument precision is how finely the bench
reads, and the 2026-09-11 brine error was 0.32 °C.

### 2. The published number is quantities covered, not rows passing

Rows passing can be inflated by adding easy rows to a quantity already
covered. `quantities_covered`, `anchored_rows` and `open_rows` are declared
fields, and the structural test recomputes all three from the rows, so a
declaration cannot drift away from what it counts. Adding five easy rows moves
`anchored_rows` and leaves `quantities_covered` exactly where it was.

### 3. No continuous-integration gate

Promotion waits until the tolerances have survived argument by someone other
than their author. A gate added on the day its tolerances were invented would
be the third check this month that read stronger than it was.

Six of the rows *are* asserted today, as ordinary tests in
`kerotakis-phreeqc/tests/colligative_numbers.rs`, and CI runs those. What is
not gated is the corpus **as a measure**.

## What the model bands rest on (added 2026-09-15)

`cases/colligative.toml` now carries `registry_input` rows: the registry
records this family's model values are computed from, each with the
uncertainty that record carries, whether the bench actually reads it, and what
it contributes to the bands.

The reason they could not be written on the day the corpus was is the reason
they matter. A model band is a claim about how far the bench may sit from
reality, and part of that distance is the uncertainty the bench's own inputs
carry — but every numeric record in the registry reported no uncertainty at
all, so that term could be named and never quantified. `PLAN.md`'s scoped task
*Give numeric records an uncertainty* closed enough of that to write the term
down, and the finding is reported in
[`../docs/registry-uncertainty-and-measurement.md`](../docs/registry-uncertainty-and-measurement.md).

The original input audit found unwired solvent constants. That finding is
historical: the follow-up [One value for the solvent](../docs/one-value-for-the-solvent.md)
wired water's molar mass and latent heats through build-time registry constants.
The current `registry_input` rows, not the earlier audit snapshot, describe
what the running path consumes.

- **The three molar-mass records are now `wired = true`.** Water's molar
  mass reaches the colligative quantities. NaCl and sucrose have empty
  `reaches` lists because these case ledgers specify solute moles, not grams;
  their molar masses do not enter these particular calculations.
- **The bounded molar-mass inputs do not control the bands.** Their
  contributions remain small compared with the documented rounding and
  routing terms. Wiring the solvent record changed no expected value or band.
- **The dominant latent heats now have registry records and sources, but
  still lack usable uncertainty widths.** Fusion is derived from vendored
  NASA polynomials (`unestablished` uncertainty). Vaporisation is traced to
  Osborne, Stimson and Ginnings (1939), whose source declines to estimate
  accuracy (`not_reported`). Traceability does not supply an uncertainty
  band or independent holdout validation; the corpus cannot spend one.

`kerotakis-core/tests/accuracy_corpus.rs` checks every declared input against
the shipped registry — value and band — so a number that moves there and not
here fails rather than drifting, and refuses a row that declares itself unwired
while claiming to reach a quantity.

## Two fields this repository has not had before

**`transcription`** — whether anyone actually read the number off the
original. For this family the answer is *no* on every source: the
bibliographic records were verified against Crossref and are exact, but both
primary papers are paywalled and no copy of the compilation was opened, so the
values are this repository's own prose finally given its sources. That is a
much better claim than the bare "by measurement" it replaces and a weaker one
than a verified transcription, and the difference belongs in a field rather
than in a reader's assumption.

**`independent_of_path`** — whether the reference shares ancestry with the
code under test. It earns its place immediately. Three of the six rows are
osmotic coefficients, and `pitzer.dat`'s Na–Cl virial coefficients are *fitted
to osmotic-coefficient measurements*; those rows check that the fit was
loaded, routed and read correctly — worth checking, since exactly that was
broken before 2026-09-13 — but they are not evidence about the physics. The
cryoscopic rows are the stronger ones, because a freezing point measured
directly is a different experiment from an isopiestic one.

The sucrose row is the most independent and the **weakest as evidence**: no
database carries sucrose, so there is no fitted parameter to share ancestry
with, but the bench's declared 2 % ideal shortfall and the sugar's own 2 %
non-ideality nearly cancel, so a pass there is close to uninformative. None of
this is visible in a count of rows passing.

## What a passing score would and would not prove

**Would prove:** for each covered quantity, on each covered script, the bench
agrees with an independently sourced published number to a stated and argued
tolerance.

**Would not prove:** that the model is right anywhere *between* the rows; that
the tolerances are appropriate; that the quantities not covered are accurate;
that a learner can reach any of it; that the engine is right for the *right
reason*; or that the reference values were transcribed correctly.

The full text lives in `proves` and `does_not_prove` inside the corpus file
itself, where anyone quoting a score will meet it. The structural test asserts
that the second is longer than the first — and records that if it ever is not,
that is a finding rather than a tidying opportunity.

## Citing sources

The governing principle, from the owner, 2026-09-14:

> We should be able to cite any book. Only not harvest the books per
> systematic scraping. And we should be able to trace original sources for
> almost all values, and cite those.

So: **naming a book as the source of one value — author, title, edition,
publisher, year, page or table — is ordinary practice and always allowed.**
What is refused is leaning on a copyrighted compilation as the systematic
source of many values. Better than either is tracing the measurement to the
paper that made it, which is why four of this family's six sources are the
original measurements and the compilation is cited beside them as the route
the numbers actually travelled.

An identifier is a convenience for the reader, never what makes a citation
legitimate. A book with no digital object identifier is not uncitable, and
treating one as uncitable on 2026-09-13 cost this family two of its rows for a
day. The structural test encodes the corrected rule: a paper must carry a
resolvable identifier, a compilation must carry its edition and publisher
instead.

What still stops, unchanged, is a row saying only that its value *agrees with*
a compilation. That offers agreement in place of an origin and never says
where the number came from.

## Adding a row

1. Put the case in `cases/<family>.toml` with the field list from
   `ROADMAP-Webapp.md` § *Validation corpus*: ledger, apparatus, boundary
   conditions, expected model path, conservation expectations, what it may
   generalise over, and its known failure modes.
2. Give each quantity a `value`, both bands, a `tolerance_reason` that names
   what the band excludes, a `source_id`, and `independent_of_path`.
3. Declare the source with `kind`, `access`, `transcription` and `licence`.
   If you read the number off the original, say so — nobody has yet.
4. Update `quantities_covered`, `anchored_rows` and `open_rows`. The
   structural test will tell you if you got them wrong.
5. Leave `gate = false`. Promoting the corpus is an owner decision.

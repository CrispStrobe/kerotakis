# Giving numeric records an uncertainty, and marking which are measured

Status: 2026-09-15. A scoped task from `PLAN.md`. The deliverable is a pattern
proven on a handful of records, three judgements stated in the open, and an
honest count of how far the pattern reaches — not a sweep of 1917 records.

## The starting state

| uncertainty kind | records | | method kind | records |
|---|---|---|---|---|
| `not_reported` | 1093 | | `imported` | 852 |
| `exact` | 824 | | `derived` | 848 |
| absolute / relative / interval | **0** | | `curated` | 111 |
| | | | `editorial` | 106 |
| | | | **`measured`** | **0** |
| | | | `calculated` | 0 |

1917 numeric records. Not one carried a band, and not one said a number had
been measured. **The registry could not distinguish a number somebody measured
from a number somebody assumed.**

## Why `measured` was empty

**Nobody filled it — and more precisely, the field was filled with the answer
to a different question.**

`method` was documented as "how the value entered the registry", and the
export took that literally: 852 records carry
`Imported("verbatim export from kerotakis_core::species::REGISTRY")`. That is
a true statement about the *export step*. No export measures anything, so
under that reading no record could ever be `measured`, however the value came
to exist. The vocabulary was not missing; the question it answered was.

It is not that nothing is measured. At least one value in the registry is a
primary measurement with a resolvable identifier —
`literature/hartley-campbell-iodine-water`, the 1908 solubility of iodine in
water — and it was labelled `curated` with the detail
*"0.3393 g/L converted exactly to 0.03393 g/100 mL"*, which describes the
unit conversion and says nothing about the origin.

`Method`'s doc comment now says the field records **how the value came to
exist, not how it reached this file**, and that is the whole repair.

## The three judgements

### 1. What an uncertainty means when the source gives none

**A record says which of two quite different things is true, and it never
converts its own last digit into a band.**

The two things were one thing until today. `NotReported` was documented as
"the source did not report one" *and* was the default the export stamped on
everything, so 1093 records asserted something about sources nobody had
opened. They are different claims with different costs to close:

- `Unestablished` — **nobody has established one.** Not a claim about the
  source: the absence of one. It is now the default, and it covers both "the
  source has not been read for a band" and "no source is identified that could
  be read". It deliberately does not separate those two, because the citation
  already does and [`registry-unattributed-census.md`](registry-unattributed-census.md)
  has counted them.
- `NotReported` — **the source was read and quotes no uncertainty.** A
  finding, reached only by reading a source.

**What a record must not do is manufacture a band from its own typography.**
Much of the older literature this bench cites quotes a number and nothing
else. The number of digits somebody typed is a fact about the typist, not
about the measurement, so a value of `0.997 g/mL` does not get `± 0.0005`; it
gets a statement that nobody has established what band it carries.

The registry has bought **zero** `NotReported` records so far, and that is the
honest reading: no source behind a shipped number has been opened and read for
a band. A test asserts the count is zero, so the day one is bought it is
bought deliberately, with the reading in hand.

### 2. Whether `measured` means the project measured it, or the source did

**The cited source. This project operates no laboratory, and the field never
claims otherwise.**

`Measured` says `source_id` points at the experiment the number came out of,
rather than at a table the number was copied from. That is the distinction
worth having — a value traceable to the determination that produced it, versus
one traceable to a compilation that repeated it — and it is exactly the line
this registry already draws between `literature/…` and the curated tranches.

**Whether anybody here has checked the number against the source's printed
table is a separate question, and it is recorded in the same vocabulary the
accuracy corpus already uses.** `validation/README.md` draws this line with its
`transcription` field: identifying a primary measurement is cheap — minutes per
row against Crossref — while verifying the transcribed number is a separate
purchase that was costed and not bought, because the papers are paywalled.
There is no second vocabulary for it here. `aqueous-solubility/I2` says in its
method detail that the experiment is Hartley and Campbell's, that the
conversion is exact, and that nobody has re-opened the paper.

That last point has a consequence that is worth stating rather than smoothing
over: **the one `measured` record still carries `unestablished` as its band.**
Being able to say a number came out of an experiment and being able to say
what that experiment quoted beside it are two purchases, and only the first has
been made.

### 3. Whether a derived quantity inherits an uncertainty from its inputs

**Yes, it propagates — but the propagation rule depends on what the input band
*is*, and getting that wrong would be worse than not propagating.**

Every species citation carrying a molar mass says the same thing: *"M from
IUPAC/CIAAW 2021 atomic weights"*. None of those records said what that
implied about its own precision, and it implies something specific and
published. For an element whose isotopic composition varies measurably between
normal terrestrial materials, **CIAAW does not publish a value with an error
bar at all — it publishes an interval**, and the number everyone quotes is a
conventional representative of it. Hydrogen, carbon, oxygen and chlorine are
four such elements; sodium is mononuclidic and gets `22.98976928(2)` instead.

So a molar mass inherits an **interval**, propagated by **interval arithmetic
and not in quadrature**. Adding two isotopic-variation intervals in quadrature
would narrow a range that is not narrow — it is a spread across real samples,
not an error in anybody's determination, and there is nothing random about it
to average away. A measurement uncertainty would propagate the other way.
Mixing the two rules is the mistake this judgement exists to avoid.

**The table was cross-checked against something already on this disk.** The
vendored PHREEQC database `wateq4f.dat` declares its own element masses, from
an older IUPAC revision and quoted coarsely: C 12.0111, Cl 35.453, H 1.008, Na
22.9898, O 16.00. Three of the five land inside the intervals here, which is
the confirmation that was wanted. The other two land just outside — and for
the reason the declined table records rather than because the intervals are
wrong: 16.00 and 22.9898 are rounded to two and four decimals, and an interval
narrower than a value's own rounding excludes it. The check is not a substitute
for CIAAW's own publication, and it is not offered as the citation; it is the
only independent numbers available offline, and they agree.

One wrinkle had to be decided and is written into the propagation: **this
registry stores an ion's molar mass as the sum of its neutral atoms.** That is
a bookkeeping choice, and a defensible one — it makes `H2O → H+ + OH-` close
its mass balance exactly (1.008 + 17.007 = 18.015), which the physical ion
masses do not. Rather than assert either reading, the propagated band is
widened by one electron mass per unit of charge so that it holds both.

## What changed, and on what basis

| | before | after |
|---|---|---|
| `exact` | 824 | 824 |
| `interval` | 0 | **66** |
| `unestablished` | — | **1027** |
| `not_reported` | 1093 | **0** |
| `measured` | 0 | **1** |
| `imported` | 852 | 790 |
| `derived` | 848 | 910 |
| **values changed** | | **0** |

Every one of the 1093 records that claimed a source reported no uncertainty
now either carries a real band (66) or says nobody has established one
(1027). `not_reported` is empty, and empty is the honest reading: no source
behind a shipped number has been opened and read for a band.

The 66 intervals are not typed in. They are computed at export time from a
five-row table of CIAAW element intervals, summed over each species' own
parsed formula. **The table is the unit of purchase**: five rows are there
because they are the elements the colligative family needs — H, C and O for
water and sucrose, Na and Cl for the brine — and the reach is then whatever
those five rows happen to cover.

## The propagation is a check, and it found five defects

The schema validator already required a record's value to lie inside its own
interval, so attaching a derived interval tests the value as well as
describing it. Five molar masses fail, and the exporter refuses both of the
things that would hide it: it will not widen a band until the value fits, and
it will not change the value. Each has to be named in
`MOLAR_MASS_INTERVAL_DECLINED` with its reason.

| species | value | its own derivation | what it is |
|---|---|---|---|
| `H2O2` | 34.01 | [34.013740, 34.015760] | quoted to four significant figures where the derivation carries six; sits 0.0037 g/mol **below** the lowest molar mass hydrogen peroxide can have |
| `O2` | 31.998 | [31.998060, 31.999540] | the same, 6e-5 below — a fifth of the last digit quoted |
| `Na+` | 22.99 | [22.989220, 22.989770] | the same, in the other direction, and the sharpest: sodium is mononuclidic, so CIAAW knows its weight to 2e-8 and 22.99 is two decimal places where seven are supported |
| `catalase` | 240000 | — | formula is `C`, a placeholder for a protein this bench counts but does not build |
| `amylase` | 55000 | — | the same |

**None is repaired here.** Sourcing a number and changing one are separate
decisions — the rule this registry already applies to the dissolution
tranche's KMnO4 and NaHCO3 rows — and rounding 34.01 up to 34.0147 would be
the second decision taken quietly under cover of the first. Three of the five
are a single phenomenon worth naming: **a value quoted more coarsely than the
quantity it represents**, which is invisible until something computes what the
quoted digits are allowed to be.

## What a tolerance in the accuracy corpus can now do

`validation/cases/colligative.toml` argues a tolerance per quantity, itemising
what each band is spent on. It could name a term it could not quantify: the
uncertainty the bench's own inputs carry. Every registry record reported
nothing, so the term was a gap in every argument in the file.

The corpus now carries `registry_input` rows — the records each model value is
computed from, with their bands and what they contribute — cross-checked
against the shipped registry by `accuracy_corpus.rs`, so a value or band that
moves in the registry and not in the corpus fails rather than drifts.

**Three things came out of writing them, and the second is not what the rows
were written to carry.**

### 1. The term can be quantified

Water's molar mass is the input every row in this family runs on — in the
kilograms of solvent the ledger converts to, and again in the osmotic rows
through φ = −ln(a_w)/(M_w·Σm). Its band is 7.1e-5 of the value, which is
6.7e-5 in an osmotic coefficient near 0.945 and about 0.025 mK in the
tenth-molal depression: **0.2 % and 0.6 % of the two tightest model bands in
the file.** Small is the useful answer. It says those bands are set by the
rounding and routing arguments each row already makes, which is what those
arguments assumed without being able to show it.

### 2. THE BENCH DOES NOT READ THE RECORD THE BAND IS ATTACHED TO

Attaching the band and then trying to spend it found this. The colligative
path never asks the registry for water's molar mass. It carries its own copy,
as a Rust literal, in **thirteen places across seven files and in three
spellings**:

| | |
|---|---|
| `states.rs` | `WATER_MOLAR_MASS_KG = 0.018_015` |
| `aqueous.rs`, `displacement.rs` | `18.015` |
| `solve.rs`, `particles.rs`, `sweep.rs` | `0.018_015` inline, nine times — six of them in `solve.rs` |
| `constants.rs` | `WATER_MOLAR_MASS = 18.015_28`, commented *"IUPAC 2021 atomic weights"* |
| `bench.rs` | falls back to `18.01528` where a registry lookup misses |

`states.rs`'s own comment reads *"M_w is the registry's own molar mass of
water, 0.018015 kg/mol"* — beside its private copy of it.

So the 66 bands are **claims about the registry and not yet about the bench**,
and a tolerance argued against one would have been arguing against a number
the bench never sees. The corpus records that per row in a new `wired` field
rather than leaving a reader to assume, and the structural test refuses a row
that declares itself unwired while claiming to reach a quantity.

The interval is what makes the disagreement *legible*, which is worth saying
for itself: 18.015 and 18.01528 differ by 1.6 ppm and **both lie inside
[18.01471, 18.01599]**. They are two conventional representatives of one
published range, not one right number and one wrong one. Without the interval
the only available readings were "identical enough" and "somebody made a
typo"; neither is what is going on. This is the shape of a defect this project
has already paid for once — one quantity, two derivations, nothing keeping
them equal — and wiring the record in is worth doing for that reason rather
than for the width.

### 3. The largest model input in the family is not a registry record at all

Both cryoscopic rows reach their answer through water's enthalpy of fusion and
both boiling rows through its enthalpy of vaporisation. Those are
`WATER_H_FUS` and `WATER_H_VAP` in `crates/kerotakis-core/src/states.rs` —
constants with **no registry record**, so they cannot carry an uncertainty in
the schema that has one, and nothing in the corpus quantifies them.
`WATER_H_VAP` is additionally the constant whose own comment says NO SOURCE IS
CLAIMED FOR THIS NUMBER.

**And this is the term that would actually matter.** The relation the bench
computes with is 1/T_f = 1/T_f° − (R/ΔH_fus)·ln a_w, so the depression goes as
1/ΔH_fus: a one per cent uncertainty in the enthalpy is a one per cent
uncertainty in the answer. That is 3.5 mK on the tenth-molal row against its
4 mK model band — **88 % of it** — and 34 mK on the one-molal row against its
60 mK band, **57 %**. Compare the 0.2 % and 0.6 % the molar mass would
contribute. The one input in this family big enough to move a band is the one
with no record, no band, and in the boiling case no source, and nobody can say
whether one per cent is pessimistic or optimistic because nothing has been
established either way.

That inverts the priority the bounded numbers suggest. Giving `WATER_H_FUS`
and `WATER_H_VAP` registry records is worth more to this corpus than the
remaining twenty-one rows of the atomic-weight table.

## How far the pattern reaches, and what the rest costs

**Molar mass — 66 of 186 today, for five table rows.**

| | |
|---|---|
| species the five-element table reaches | 71 |
| of those, given an interval | **66** |
| of those, declined as findings | 5 |
| species it does not reach | 115 |
| elements needed to reach them | **21 more rows** |

The remaining 21 rows are ordinary CIAAW entries and the work is one row each,
checked automatically: the validator refuses any interval that excludes its
value, so extending the table cannot quietly claim a band. The realistic cost
is **an afternoon for the table and an unknown number of findings** — the
five-element table turned up five, so twenty-one more rows should be expected
to turn up a similar crop of coarsely-quoted values, each of which is a
decision somebody has to make rather than a line somebody has to type.

**Beyond molar mass, the pattern does not extend by table.** The 1027
`unestablished` records divide into populations whose costs are not
comparable, counted here over the whole registry rather than over the 487 the
census covered:

| | records | what closing it takes |
|---|---|---|
| declared non-claims — a value of 0.0 meaning "not modelled" or "colourless", or the documented 1.0 g/mL density placeholder | **191** | **nothing.** These do not want a band; they want to keep saying they are not claims. The schema has no better home for them than `unestablished`, which is a real finding and the census's territory rather than this pass's |
| curated or editorial tranches — teaching values the project chose | **202** | **nothing readable.** A declared editorial constant has no source to open. A band here would have to be argued, not read, and that is a different kind of work |
| imported legacy values naming a source | **628** | **one reading each**, where the citation names something openable. This is where `not_reported` gets its first occupant, and the cheapest candidate is `literature/hartley-campbell-iodine-water`: already identified, identifier resolvable, published 1908 |

Cutting across those three, the census counted separately: **roughly 120
numbers whose citation names nothing a reader could check at all.** Those need
a re-sourcing, not a reading. **The uncertainty programme is blocked behind
the sourcing programme there**, and no amount of this kind of work unblocks
it.

`boiling-point/water` is the clearest instance, and it sits in the curated row
rather than the legacy one. The phase-transition tranche *had* a citation and
it was withdrawn on 2026-09-13, when the handbook it named turned out to sit
on the provenance audit's commercial row. There is no source to read for a
band because the source was deliberately given up. No table and no propagation
reaches it.

### And the cheapest useful purchase is none of the above

The reach numbers above rank the work by how many records it touches, and that
ranking is wrong for the corpus that asked for this. **Two numbers with no
record at all are worth more than the twenty-one remaining table rows**:
`WATER_H_FUS` and `WATER_H_VAP`, because the depression goes as 1/ΔH and a one
per cent uncertainty there consumes 88 % of the tightest model band in the
colligative family, against the 0.2 % a molar mass contributes.

`WATER_H_FUS` is the cheap half: it is already derived from the vendored
`vendor/nasa-cea/thermo.inp`, and `kerotakis-cea` re-derives it from the
shipped file on every run — so it needs a registry record and its existing
source written down, not a search. `WATER_H_VAP` is the expensive half, and
its own comment already says why: NASA CEA was checked and rejected because
its gas records are ideal-gas and the 228 J/mol gap is steam's non-ideality,
so restoring support needs a primary measurement with an identifier. Both are
recorded as follow-ups in `PLAN.md`.

## What was deliberately not done

- **No value was changed.** Five are recorded as outside their own derivations
  and left exactly as they stand.
- **No band was invented.** Nothing got a `± half the last digit`.
- **No sweep.** 1027 records still say nobody has established their
  uncertainty, which is what is true of them.
- **`provenance/upstreams.toml` was not touched** — another session owns it.

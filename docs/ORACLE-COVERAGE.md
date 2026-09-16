# Build-time oracle coverage

*Tier C of the three-tier plan in [`EXPERIMENTS.md`](../EXPERIMENTS.md)
("How the expectations get checked: three tiers"). Tier A is perturbation,
tier B is authored expectations; this file is about the middle one —
external oracles, widened to whatever they can legitimately reach, so that
tier B stays small.*

This file answers one question per row: **what is this oracle independent
of?** The rule it is written against is
[`ROADMAP-Webapp.md`](../ROADMAP-Webapp.md)'s, in that document's own words:

> Agreement between two paths using the same database is useful but is not an
> independent validation of that database. Record shared ancestry in
> provenance.

An oracle sharing a database with the path it checks proves the code reads
the database. That is worth having, and it must not be written down as more
than that. Two recent findings are why the sentence is repeated here rather
than assumed: three of the colligative accuracy family's six rows turned out
not to be independent of the path they test, because the coefficients being
checked had been fitted to the measurements used as the reference; and a
proposed corroboration was rejected because at infinite dilution a tracer
diffusion coefficient *is* a conductivity measurement in different units, so
the agreement was a round trip.

**The number that matters is quantities covered with a stated independence,
not checks passing.** A count of passing checks can be raised by adding easy
ones — which is why `validation/cases/colligative.toml` publishes
`quantities_covered` and recomputes it from its own rows, and why the two
counts below are stated separately per quantity rather than summed.

---

## 1. What was wired before this document

Surveyed rather than assumed. Of the eight oracle tools and fixture sets in
the repository, **one** was consulted by a test.

| Oracle | Wired? | Reaches |
|---|---|---|
| `tools/gen-thermo-fixtures.py` → `crates/kerotakis-thermo/tests/thermo_oracle.rs` | **yes** | UNIFAC γ and bubble points, ethanol–water only |
| `crates/kerotakis-phreeqc/tests/oracle/expected/*.json` (10 fixtures) | yes | the MY-BASIC interpreter, not chemistry — its `oracle` field names *"values captured from legacy oracle during development"* |
| `tools/check-properties-vs-chempy.py` (135 lines) | no | `kero properties` water density / viscosity / permittivity / Henry |
| `tools/check-relations-vs-chempy.py` (221 lines) | no | `kero calc` Arrhenius, Eyring, Nernst, Henderson–Hasselbalch, Debye–Hückel, van 't Hoff, ionic strength |
| `tools/kinetics-oracle.py` (108 lines) | no | the kinetics integrator against SciPy |
| `tools/surface-oracle.py` (299 lines) | no | the HFO zinc pH edge, second implementation from `wateq4f.dat` |
| `tools/vle-oracle.py` (183 lines) | no | ethanol–water VLE **via thermo's compound database** — see §5 |
| `tools/fixtures/properties-chempy.txt`, `relations-chempy.txt`, `vle-ethanol-water.json` | no | committed oracle *output* that nothing reads |

So the honest before-picture: **one wired oracle, covering two quantities on
one binary mixture**, plus a second fixture set that checks an interpreter.
Everything else is a script somebody would have to remember to run, or a
transcript from a day in August.

## 2. What the wired oracle was missing

`crates/kerotakis-thermo/src/unifac.rs::approved_table()` curates ten
subgroups and thirty ordered interaction parameters. Ethanol–water reaches
main groups 1 (CH2), 5 (OH) and 7 (H2O) only. **Six of the thirty parameters
and four of the ten subgroups moved a number; twenty-four parameters and six
subgroups had no check of any kind.**

That is measurable rather than rhetorical. Five one-digit edits in the fourth
significant figure of `approved_table()` — a(9,20), a(6,20), a(6,1), a(20,7)
and r(C) — **all pass the pre-widening oracle and all fail the widened one**.

## 3. Quantities covered now

Each row states the independence claim. None of the UNIFAC families is
independent of the parameter publication, because both sides read
Fredenslund 1975 and Gmehling 1982; that is stated in every one rather than
left for a reader to work out.

### The `thermo` differential oracle (`crates/kerotakis-thermo/tests/thermo_oracle.rs`)

| Quantity | Reach | Independent of | NOT independent of | Consumed by |
|---|---|---|---|---|
| activity coefficient γ(x,T) | 17 binaries, 254 rows; every curated subgroup and every curated interaction parameter moves a row | our combinatorial + residual arithmetic, our group bookkeeping, **our transcription of r, q and a_mn** | the UNIFAC parameter publication | everything below, plus `vle.rs`'s flash |
| excess enthalpy hᴱ | 17 binaries, 85 rows — **new** | all of the above **and our differentiation scheme**: ours is a 1 K central difference, thermo's is analytic | the parameters; and the larger fact that VLE-fitted parameters give a qualitative hᴱ | `kerotakis-core::hmix` — the temperature change a learner feels on mixing |
| infinite-dilution γ∞ | 8 solute/solvent pairs, 16 rows — **new** | our arithmetic and our 1e-9 stand-in for "infinitely dilute" | the parameters, and the group decomposition the consumer feeds in (see §6) | `bench.rs::partition_k` — a separating funnel's partition coefficient |
| bubble point (T, y) | 5 binaries, 27 rows compared and 8 correctly refused as out of range | our flash solver, its bracketing and its γ(T) loop | **the Antoine constants**, duplicated into the generator on purpose | the still in `bench.rs`, `ethanol_water_still` |

Before: **two quantities, one binary.** After: **four quantities, seventeen
binaries**, three of the four new or newly consumed-and-checked.

### The Henry's-law oracle (`crates/kerotakis-core/tests/henry_oracle.rs`) — new

`properties::HENRY_COEFFICIENTS` carries six gases from Sander (2015), an
atmospheric-chemistry compilation. The aqueous solver runs on the USGS
PHREEQC databases — a different lineage — and four of the vendored files
tabulate the same dissolutions. `tools/gen-henry-oracle.py` reads them out.

This one is **more than an oracle**: `kerotakis-phreeqc::aqueous` asks
`henry_lookup("CO2")` for a solubility while solving a speciation on a
database that states its own, so a disagreement is the bench holding two
answers to the same question.

| Quantity | Reach | Independent of | NOT independent of |
|---|---|---|---|
| Henry solubility H<sub>cp</sub>(298 K) | **5 of 6 shipped gases** (CO2, O2, N2, H2, NH3), 12 database rows | Sander's compilation entirely — a different field, different compilers, different files | the primary measurements the two compilations may share. Where they agree to 0.1% (CO2 does), shared ancestry is the likely explanation |
| temperature coefficient −Δ<sub>sol</sub>H/R | **3 of 6 gases** (CO2, O2, NH3) | as above | as above |

Cl<sub>2</sub> is **unreachable** and the test says so in a field the build
checks: no vendored database writes chlorine's dissolution as `Cl2 = Cl2`,
only as a hydrolysis-plus-redox reaction whose log K is a different quantity.
Chlorine's 9.2e-2 mol/(L·atm) and its 2500 K rest on Sander alone.

Two rows **disagree materially and are pinned rather than banded away**:

- **H₂'s temperature coefficient.** `phreeqc.dat` implies 397 K, `wateq4f.dat`
  implies 885 K, we ship Sander's 500 K. The two USGS files disagree with
  *each other* by a factor of 2.2.
- **N₂'s temperature coefficient.** `wateq4f.dat` implies 683 K against our
  1300 K; `phreeqc.dat` states no enthalpy for nitrogen at all.

Neither is load-bearing today — the gas the bench watches dissolve and escape
is carbon dioxide — but a lesson that warmed a bottle of soda water and asked
about dissolved nitrogen would be resting on an uncorroborated number.

A band that admits a 77% disagreement is not a check, so those three rows
carry a recorded figure instead, asserted to 1 percentage point. Both sides
are static tables; the gap is exactly reproducible, and a silent improvement
should be as noticeable as a silent regression.

## 4. Tolerances, and why these widths

Following `validation/cases/colligative.toml`'s rule — argued per quantity,
never a flat percentage — and its mechanism: the width and the reason are
separate fields so that widening one shows in a diff without the other
moving.

- **γ, γ∞: 1e-4 relative.** Two implementations of the same closed-form
  expression in double precision; anything looser would stop being a
  transcription check.
- **hᴱ: 0.05 J/mol absolute, 1e-4 relative.** Absolute *as well as* relative
  because hᴱ crosses zero and a relative band alone is meaningless at the
  crossing. The width is set by what the method costs: the worst
  disagreement over 85 rows is **0.0061 J/mol**, which is what a 1 K central
  difference costs on this model, and the band is eight times that — enough
  to absorb a float-order rearrangement, not enough to absorb a change of ΔT
  or of method. For scale it is 0.03% of the acetone–water minimum and a
  thousandth of what `excess.rs` calls "exothermic mid-range".
- **Bubble point: 0.05 °C and 5e-4 in y**, unchanged from the one binary this
  family used to cover.
- **Henry: per gas, set by the spread between the USGS databases themselves**,
  because nothing tighter than that spread is a meaningful claim about a
  compiled value; and narrow enough to exclude a transposed digit in the
  shipped coefficient. CO2 gets 1.5% (the files spread 0.7%), N2 and H2 get
  ~12% (the files spread 18% and 12%), and the wide ones say in their own
  reason text that they are wide because the references disagree.

## 5. What is NOT reachable, and why

Stated because a coverage claim without its complement is half a claim.

- **Four of the six curated Antoine constant sets** (isopropanol, methanol,
  propanone, ethanoic acid) — and the two for water and ethanol. The bubble
  family checks the *solver*, not the constants: the generator duplicates
  them deliberately so drift shows up. Checking the constants themselves
  needs a vapour-pressure source, and the obvious one is barred — see below.
- **`tools/vle-oracle.py` is out of policy and must not be wired.** It calls
  `thermo.VaporPressure(CASRN=...)`, which is a lookup in thermo's compound
  database, backed by the `chemicals` package. `provenance/upstreams.toml`
  marks `chemicals-python` **`avoid`**, and the `thermo-python` row that
  clears thermo at all says exactly what it clears: *"only the numbers it
  PREDICTS from our own inputs may be compared against."* The generator is
  bound by that sentence and says so in its docstring. The unwired script has
  never produced a committed fixture, so nothing rests on it — but it is a
  loaded foot-gun and this is the note.
- **Chlorine's Henry constants**, as above.
- **The water–hexane binodal.** `lle::water_hexane_lle` is consumed by
  `solve.rs`, and its only test asserts `x < 0.15` on one side and `x > 0.85`
  on the other. The binodal is determined entirely by γ(x), and γ(x) for that
  pair **is** now oracle-checked, down to x = 1e-6 in both corners. What is
  not checked is our binodal solver — the step from γ to a tie line. An
  independent common-tangent solve was attempted for this change and
  abandoned: it is a genuinely awkward two-dimensional root find for a pair
  this asymmetric, and a fragile oracle is worse than a stated gap.
- **`ethanol_water_density_g_ml` and `sucrose_water_density_g_ml`**
  (`properties.rs`) are polynomial fits to the CRC Handbook, which
  `upstreams.toml` marks `avoid`. No oracle here touches them.

## 6. Oracles that would check a number nothing consumes

The brief for this work said to prefer numbers something consumes, after a
change found the engine was not reading the registry records whose
uncertainty was being argued over. Two candidates were examined and
**deliberately not wired**:

- **`crates/kerotakis-thermo/src/eos.rs` — Peng–Robinson.** 175 lines,
  `z_vapour`, `molar_volume` and `fugacity_coefficient`, with critical
  constants for water, ethanol and nitrogen. `git grep` finds **no caller
  outside its own unit tests**, in `crates/`, `web/` or `tools/`. An oracle
  against thermo's own PR implementation would be cheap and would check
  nothing a learner or a script can reach. It is the `electrodiffusion.rs`
  shape `docs/NEXT-MEASURE.md` §5 describes, on a smaller module.
- **`crates/kerotakis-thermo/src/phase_diagram.rs`.** 94 lines, referenced
  only by its own `pub mod` line — not even by a test.

Recorded here rather than acted on: making them reachable is a capability
decision, not an oracle one.

## 7. A finding the widening surfaced

`kerotakis-core/src/bench.rs::partition_groups` decomposes **methanol as
CH3 + OH**. Original UNIFAC assigns methanol the single CH3OH subgroup —
which is what `approved_table()`'s own comment says that subgroup is *for*:

> Main group 6: CH3OH (methanol, treated as a single group because the
> hydroxyl bonded directly to the methyl has distinct interaction behaviour
> from the general OH group)

The two assignments give γ∞(water)/γ∞(hexane) of **0.1070 and 0.0841**, 27%
apart, and that ratio is the partition coefficient the bench reports when a
learner extracts methanol from water with hexane. Both are pinned in the
fixture and a test asserts they are still apart, so the choice cannot go back
to being invisible. **Changing it is not this change**: it moves a bench
answer, and the decision belongs with whoever owns the partition path.

## 8. Cost of continuing

- **γ, hᴱ, γ∞ on a new binary: free.** A line in `BINARIES`, a decomposition
  in two hand-kept tables, a regeneration. The parameter table is fully
  exercised, so further binaries buy consumer relevance rather than
  parameter coverage.
- **A new quantity from thermo that predicts from our own inputs:** an hour.
  `GE`, `SE` and `CpE` are available on the same object and none is consumed
  today.
- **The water–hexane binodal:** perhaps half a day, and the value is real
  because the current test is a bound rather than a number.
- **Wiring `tools/check-properties-vs-chempy.py`:** needs ChemPy in the
  upstream audit table, which it is not in today despite two checked-in tools
  importing it (see `provenance/upstreams.toml`). Its water correlations are
  genuinely independent implementations of Tanaka 2001, Korson 1969 and
  Bradley–Pitzer 1979; of those three, only viscosity is read by the solver.
- **Wiring `tools/surface-oracle.py`:** the most valuable unwired oracle in
  the repository — a second implementation of HFO surface complexation from
  the same approved USGS constants, so it is independent of IPhreeqc's
  numerics though not of its database. It needs the CLI built and a candidate
  JSON, which is why it was not done here rather than because it is not worth
  doing.

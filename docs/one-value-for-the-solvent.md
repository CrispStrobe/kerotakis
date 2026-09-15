# One value for the solvent, and where it comes from

Status: 2026-09-15. The follow-up to
[`registry-uncertainty-and-measurement.md`](registry-uncertainty-and-measurement.md),
whose section 2 found that the bench does not read the records whose bands it
had just written, and whose section 3 found that the biggest input of all had
no record to read.

## Read the second half first

Two defects were found together and they are **not** the same size. The
arithmetic says so, and it is not close:

| | if it carried 1 % uncertainty | share of the tightest model band in `colligative.toml` |
|---|---|---|
| **enthalpy of fusion** | 3.5 mK on the tenth-molal depression | **88 %** |
| molar mass | 0.025 mK | 0.6 % |

The freezing depression goes as 1/ΔH_fus, so a relative uncertainty in the
enthalpy is the *same* relative uncertainty in the answer. A molar mass enters
linearly and its published interval is 7.1e-5 wide. **Two and a half orders of
magnitude separate them.**

So the governing half of this change is that **water's two latent heats are
registry records for the first time, and the enthalpy of vaporisation has a
primary source for the first time at all**. That is what decides what an
accuracy tolerance in this family can claim.

The molar-mass half is a real defect and worth closing — one quantity written
four ways across eighteen sites, with a fallback that could have substituted a
different number — but **nothing it touched moved, and nothing was ever going
to.** That is the honest weight to give it, and it is why it is second here.

## The two latent heats

Neither had a registry record. They are the terms that dominate every band in
`validation/cases/colligative.toml`: the depression goes as 1/ΔH_fus, so one
per cent there is 88 % of the tightest model band in the file, against the
0.2 % a molar mass contributes.

### Fusion — a record, and an honest `unestablished`

`enthalpy-of-fusion/water` = 6010 J/mol, sourced to the already-vendored
`vendor/nasa-cea/thermo.inp` (Apache-2.0) as
`H(H2O(L), 273.15 K) − H(H2O(cr), 273.15 K)` = 6009.9 J/mol.
`kerotakis-cea`'s `latent_heats_are_the_vendored_file` re-derives it from the
shipped file on every run.

Its uncertainty is `unestablished` and **deliberately not `not_reported`**:
thermo.inp does not print an enthalpy of fusion at all — the value is a
difference of two fitted polynomials — so there is no quantity in that file
for it to have quoted a band on. Saying the source was read and reported none
would be a finding about a claim the source never made.

### Vaporisation — a primary source, for the first time, and the registry's first `not_reported`

`WATER_H_VAP` had **no source at all**; its own comment said so in capitals
after a CRC citation was withdrawn on 2026-09-13.

It now cites **N. S. Osborne, H. F. Stimson and D. C. Ginnings,
*Measurements of heat capacity and heat of vaporization of water in the range
0° to 100° C*, J. Res. NBS 23 (1939) 197–260, RP1228,
doi:10.6028/jres.023.008** — primary journal literature with a DOI, which is
the `primary-literature` route of `provenance/upstreams.toml` rather than a
compilation row, and separately a United States Government work in the NBS
Technical Series and so public domain. Read 2026-09-15 from `nvlpubs.nist.gov`;
the DOI was confirmed against Crossref, which returns volume 23, page 197,
August 1939 for that title.

**What it prints.** Table 13, *Formulation of data on heat of vaporization*,
row 100 °C, column *Heat of vaporization, L*: **2256.30 international joules
per gram**. The row is identified beyond doubt by its last column — the
specific volume of saturated vapour, 1673.0 cm³/g. The table is self-checking
and the check passes: the paper defines γ = L + β and prints γ = 2257.71 and
β = 1.408 on the same row; 2257.71 − 1.408 = 2256.30.

**The arithmetic to J/mol.** The paper states its own electrical-unit
conversion, 1 int. J = 1.00019 abs. J, giving 2256.73 J/g. Multiplied by the
registry's own 18.015 g/mol, that is **40 655 J/mol** — which is **40 650 to
the four significant figures this constant has always carried**.

**The value did not move.** Sourcing a number and changing one are separate
decisions, and here only the first was needed.

**And it is the registry's first `not_reported` record.** #608 emptied that
kind and asserted the count was zero, saying the day one was bought it would be
bought deliberately with the reading in hand. This is that day. The authors
decline to quote an accuracy, in terms:

> this agreement must not be taken as an estimate of the accuracy of the
> results, since it takes no account of unknown systematic errors, which may
> well be larger than the accidental errors.

So the corpus still cannot spend a band on the term that dominates it. **What
the sourcing bought is traceability, not a width**, and the remaining purchase
is now precisely nameable: a modern evaluation of the steam properties.

## The smaller half: the defect in the molar mass


`git grep` over `crates/*/src/` on the tip this branched from finds **24
occurrences**. Five are prose and one asserts a third party's arithmetic,
leaving **18 sites across seven files** carrying the bench's own copy of water's
molar mass, in **four spellings** that between them hid **two different
numbers**.

| spelling | sites | where |
|---|---|---|
| `0.018_015` | 9 | `solve.rs` ×6, `particles.rs`, `states.rs`, `sweep.rs` |
| `0.018015` | 1 | `sweep.rs` — the *other* site in the same file |
| `18.015` | 3 | `aqueous.rs`, `displacement.rs`, `clock.rs` |
| `18.015_28` / `18.01528` | 5 | `constants.rs`; as a **fallback** in `bench.rs` and `scene.rs`; and in two test fixtures, `scene.rs` and `swelling.rs` |

`sweep.rs` is the one file that disagreed with **itself** — the same quantity
spelled two ways ninety lines apart, which is the whole defect in miniature.

Sixteen of the eighteen never consulted the registry at all. Two consulted it
and then fell back to a *different number* if the lookup missed. `states.rs`
carried a comment reading *"M_w is the registry's own molar mass of water,
0.018015 kg/mol"* directly beside its private copy of it.

## The two spellings, and what "deciding" between them does and does not mean

**They are indistinguishable at any precision this bench resolves.** 18.015 and
18.01528 differ by 1.6 ppm, and **both lie inside** the interval CIAAW publishes
for water's molar mass, [18.01471, 18.01599]. Neither is measurably wrong;
neither could be shown wrong by any measurement this project could make. **No
verdict on accuracy is offered here, and none is available.** This is one
quantity written four ways, and the reason to unify it is that nothing kept the
four equal — the same shape as the native-versus-wasm split this project has
already paid for once — not that a wrong number was reaching an answer.

**What *is* decidable is which table each came from**, and that settles which
one the engine should carry without settling which is truer:

```
2 × 1.00794 + 15.9994 = 18.01528   ← pre-2009 IUPAC standard atomic weights
2 × 1.008   + 15.999  = 18.015     ← IUPAC/CIAAW 2021 conventional values
```

`constants.rs` read `WATER_MOLAR_MASS = 18.015_28` under the comment *"IUPAC
2021 atomic weights"*. The number is not those weights; it is the values they
superseded. **The label was wrong even though the number is not.** 18.015 is
what the registry record holds, what 2021 actually gives, and what every site
that computed with the value already used, so the engine reads the registry's —
a statement about single-sourcing rather than about accuracy.

It also closes the bench's own mass balance exactly: `H2O → H⁺ + OH⁻` is
1.008 + 17.007 = 18.015 on the convention this registry stores ion masses under.

**The interval is what made any of this readable.** Without it the available
readings were "identical enough" and "somebody made a typo"; with it the
question becomes *which table did each come from*, which has an answer, while
the accuracy question is visibly closed — two representatives of one published
range.

## What now reads the registry, and what legitimately does not

### Reads it — 19 sites

`crates/kerotakis-core/build.rs` generates four constants out of
`data/registry/registry-source-v1.json`:

| constant | record |
|---|---|
| `WATER_MOLAR_MASS_G_PER_MOL` | `molar-mass/water` |
| `WATER_MOLAR_MASS_KG_PER_MOL` | the same value ÷ 1000, not re-entered |
| `WATER_ENTHALPY_OF_FUSION_J_PER_MOL` | `enthalpy-of-fusion/water` |
| `WATER_ENTHALPY_OF_VAPORISATION_J_PER_MOL` | `enthalpy-of-vaporisation/water` |
| `WATER_LIQUID_HEAT_CAPACITY_J_PER_MOL_K` | `heat-capacity/water` |

All eighteen molar-mass sites now read the first two, plus the three
latent-heat and heat-capacity constants in `states.rs`, plus one site in
`kerotakis-phreeqc`'s tests where a fixture's grams-to-moles conversion used
the other spelling.

### Why a generated constant and not a runtime lookup

The two `.map_or(18.01528, |d| d.molar_mass)` fallbacks are the interesting
case, and the honest answers to *"what if the lookup misses?"* were **refuse**
or **say why the fallback is acceptable**. Neither is needed, because the
question is answerable at build time: `REGISTRY` is generated from the pack,
built-in keys always win a pack collision, and `build.rs` already panics by
name if a species has no molar mass. So a lookup for `water` cannot miss.

A generated constant buys three things a runtime lookup does not:

1. **It cannot miss**, so no site needs a fallback and the silent
   disagreement has nowhere to live.
2. **It stays usable in a `const` context**, which a lookup is not.
3. **The build fails by name** if the record is dropped from the pack,
   instead of a solver quietly substituting a different number.

`crates/kerotakis-core/tests/one_value.rs` pins the generator against a runtime
`species::lookup`, so the two cannot come apart.

### Legitimately does not read it

| site | why |
|---|---|
| `kerotakis-phreeqc/tests/basic_dialect_oracle.rs`, `my_basic_preview.rs` | they assert **IPhreeqc's own** GFW of H₂O. That number is the vendored engine's, from its database's element masses, and replacing it with ours would stop the test checking anything. |
| `kerotakis-org/src/lib.rs` | asserts the vendored `chematic` descriptor recomputes 18.015 from a formula. A **third party's** arithmetic; reading our constant makes it tautological. Allowlisted by name in `one_value.rs`. |
| `kerotakis-core/tests/states.rs`, `ambient_heat.rs`; `kerotakis-phreeqc/tests/{surface_transport, milk_buffer, titration_refinement, colligative_numbers}.rs` | they recompute an **expected** value by hand. A test that derives its expectation from the constant under test is not a test. |
| `kerotakis-thermo/tests/vle.rs` | a fixture passing ethanol's and water's molar masses to a pure mass-fraction helper. `kerotakis-thermo` does not depend on the registry, and this argument is an input to the helper rather than a claim about water. |
| `kerotakis-data/tests/{chebi_adapter, pubchem_adapter, registry_parity, phase_properties}.rs` | pinned expectations that an adapter or the pack **produces** 18.015. The literal is the assertion. |
| `kerotakis-registry-export/src/lib.rs` | the file that **authors** the records. Its citation prose quotes the number — including the arithmetic that converts 2256.30 int. J/g into a molar enthalpy — and writing it down is this file's job. Allowlisted by name. |
| `data/cea/reachable-subset.json`, `vendor/nasa-cea/thermo.inp` | NASA's file declares `18.0152800` in every H₂O record. It is **their** number in **their** file and must stay. |

## What moved

**Nothing.** No golden, no lesson transcript, no corpus row, no shipped value.

That is the result the arithmetic predicted, and the reason is worth stating
rather than assuming: **every site that actually computed with the number
already used 18.015.** The only two occurrences of 18.01528 that could have
reached an answer were fallbacks behind a lookup that cannot miss, and
`constants::WATER_MOLAR_MASS` — the one place the mislabelled number was stated
outright — had **no callers at all**.

So the second spelling never reached an answer, and even if it had, the two are
1.6 ppm apart and the goldens do not carry that digit. **A change that moves
nothing is the evidence here, not the disappointment.** What was closed is a
mechanism that could have let two numbers drift apart unnoticed, on a quantity
where nobody was watching because nobody could tell the two apart.

## Is this a case or a pattern?

**A pattern. Four instances, three closed here.**

| constant | registry record | same value? | disposition |
|---|---|---|---|
| `constants::WATER_MOLAR_MASS = 18.015_28` | `molar-mass/water` = 18.015 | **no — 1.6 ppm apart, both inside the interval** | generated; the second spelling is gone |
| `states::LIQUID_WATER_HEAT_CAPACITY = 75.3` | `heat-capacity/water` = 75.3 | yes | generated |
| `states::WATER_FREEZING_K = 273.15` | `melting-point/water` = 273.15 | yes | **left, with a follow-up** |
| `states::WATER_BOILING_K = 373.15` | `boiling-point/water` = 373.15 | yes | **left, with a follow-up** |

The heat capacity is the one that proves it is a pattern rather than an
accident, because it carries **the same self-aware comment**: *"The registry's
own figure, restated here so the three phases read as one set rather than two
constants and a lookup."* Restating a record is how both defects started — a
restatement is a copy that has been told it is not one. It
also had, like the molar mass, **no callers**: `constant_heat_capacity_in`
already returns the registry value for the liquid branch, so nothing would have
noticed the two coming apart.

The two transition temperatures are **deliberately not wired**, and the reason
is a question this pass should not answer quietly. The corpus's own
`boiling-point/water` row raises it: 373.15 K is 100 °C *exactly*, which is a
definition of the pre-ITS-90 temperature scale rather than a measurement of
water, and the two stopped coinciding when ITS-90 replaced it. So either these
constants are the model's reference points — definitions, which should stay
literals with a comment saying so — or they are the substance's transition
temperatures, in which case the registry owns them and `melting-point/water`'s
withdrawn citation has to be replaced before anything is wired to it. Recorded
in `PLAN.md` as a follow-up with both halves of the question stated.

## Does this touch the open atomic-weight question?

`provenance/upstreams.toml`'s `ciaaw` row is `decision-required`, and #608
recorded a correction to the argument it rests on: that argument says the
question may not arise because this registry ships **computed** compound masses
rather than a copy of any table, and #608 made that slightly less true by
writing five element intervals into source verbatim and giving four
single-atom records a published interval unchanged.

**This change does not make that worse, and it is not this pass's to resolve.**
Nothing here adds a verbatim element value, an interval, or a table row. It
copies no atomic weight. What it does is put the *existing* computed mass behind
one name instead of eighteen, which moves the dependence in the direction that
argument prefers rather than against it.

One interaction is worth naming rather than leaving for a reader to find: the
new `enthalpy-of-vaporisation/water` record's derivation **uses** the molar
mass, because the 1939 measurement is per gram and the bench's unit is per
mole. That is a computation on a computed mass, which is the side of the line
the `ciaaw` row's own reasoning already puts such things on — but it is a new
dependant, and if that row's answer ever comes back unfavourable this record's
arithmetic is one of the things that would have to be re-read. Recorded, not
resolved.

## What was deliberately not done

- **No value was changed.** Not the molar mass, which was already the
  registry's everywhere it was computed with; not the vaporisation enthalpy,
  whose source supports the four figures it already carried.
- **No band was invented.** The one source read for a band declines to give
  one, and that refusal is recorded as a finding rather than smoothed into an
  estimate.
- **No fallback was kept.** A fallback to a different number is a silent
  disagreement waiting for the day the lookup misses; the fix was to make the
  lookup unable to miss, not to make the fallback agree.
- **The two transition temperatures were not wired**, because whether they
  should be is a real question with two defensible answers.

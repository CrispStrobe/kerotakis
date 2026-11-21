# Reaction sweep, 2026-11-20: 30 new everyday and school-lab reactions

## Inventory first: how many prepared reactions exist?

Far more than the ~440 "qualified originals" suggest. That number counts
reviewed qualifications against the fixed 468-case target, not scripts.

| Source | Scripts | Executed before this sweep? |
| --- | ---: | --- |
| Fixed target (original 147 + first120 + batch2 101 + batch3 100) | 468 | yes, all screened once natively; 436 currently qualified |
| Curriculum 200 (common-engine) | 200 | yes, hosted native screen (668 total with the above) |
| Reaction dataset 200 | 200 | yes, all executed once |
| Curiosity corpus (`tests/coverage/curiosity-v1`) | 500 prompts | yes, on every CI run (`kero coverage curiosity --check`) |
| Lessons | 113 | yes, on every CI run |
| Contract/follow-up/control scripts in audits | several hundred | yes, each with saved outputs |
| **Distinct `.lab` scripts in the audit tree** | **1,624** (2,141 files) | no unexecuted batch was found |

So: yes, well over 2,000 prepared reactions exist, but no batch of
never-executed prepared scripts was found. Each batch above has saved
execution results. That is why this sweep wrote 30 new reactions.

## Method

Criteria for all 30 scripts were written down before any run (kind:
gas evolved, reactant consumed, precipitate present, temperature change,
pH band). Each script was posted once to the production REST API on
revision `ad21c6de` (an ancestor of current main). Verdicts were read
from the final bench contents and the full event stream.

## Verdicts

| ID | Reaction | Verdict | Final T (°C) | Unmet criterion |
| --- | --- | --- | ---: | --- |
| S01 | Mg ribbon in dilute HCl | works | 30.5 | |
| S02 | Zinc granules in dilute H2SO4 | fails | 25.0 | no H2, Zn unchanged |
| S03 | Aluminium foil in NaOH solution | fails | 29.2 | no H2, Al unchanged, **no event at all** |
| S04 | Iron filings in CuSO4 | works | 27.8 | |
| S05 | Zinc in CuSO4 (warms) | works | 28.9 | |
| S06 | Pb(NO3)2 + KI golden precipitate | works | 24.9 | |
| S07 | AgNO3 + NaCl | works | 25.1 | |
| S08 | CuSO4 + NaOH | works | 25.5 | |
| S09 | FeCl3 + NaOH | works | 25.0 | |
| S10 | BaCl2 + Na2CO3 | works | 25.1 | |
| S11 | CaCl2 + Na2CO3 | works | 25.5 | |
| S12 | Hard-water softening with washing soda, filtered | works | 25.1 | |
| S13 | Baking soda + citric acid | works | 24.5 | |
| S14 | Vinegar + baking soda | works | 23.1 | |
| S15 | Marble chips + HCl | works | 25.1 | |
| S16 | HCl + NaOH neutralisation | works | 37.0 | |
| S17 | H2O2 + MnO2 | works | 27.1 | |
| S18 | Elephant toothpaste (H2O2 + KI + soap) | works | 30.0 | |
| S19 | NH4Cl + NaOH, warmed | fails | 37.4 (froze to −1.3 first) | no NH3 gas; **heat wrong by ~600 kJ/mol** |
| S20 | Limewater + CO2 | works | 25.3 | |
| S21 | CuO + warm H2SO4 | works | 46.8 | |
| S22 | Magnesium burning | works | 2512 (adiabatic) | |
| S23 | Iron wool rusting over 72 h | works | 25.0 | |
| S24 | Brine electrolysis | works | 24.2 | |
| S25 | CuSO4 electrolysis | works | 26.8 | |
| S26 | Copper carbonate thermal decomposition | fails | – | refused: `CuCO3` unknown species |
| S27 | Sugar + concentrated H2SO4 | fails | 25.0 | typed model-boundary refusal (concentrated acid medium) |
| S28 | Fischer esterification | works | 25.0 | |
| S29 | NaOH dissolving | works | 35.7 | |
| S30 | NH4NO3 dissolving | works | 18.8 | |

**Score: 24 work, 0 partly, 6 fail.**

## Gaps grouped by root cause

### G1. Wrong thermo basis for derived aqueous species (missing thermo consistency) — FIXED in this PR

Symptom (S19 and follow-up probes): `NH4Cl + NaOH` in 50 mL of water
cooled the beaker by 26 K and froze part of it. The real reaction is
almost thermoneutral (`NH4+ + OH- -> NH3 + H2O`, ΔH = −3.6 kJ/mol from
NBS/NIST standard enthalpies of formation; the NaOH itself warms the water
by about 2 K). In the other direction, `NH3 + HCl` heated 50 mL by 40 K
instead of about 7 K.

Root cause: each aqueous species was priced with the `delta_h` of the
reaction that *defines* it. That number is relative to whatever is on the
reaction's left-hand side, which is a master species only sometimes.
wateq4f and phreeqc.dat define `NH4+` from `NO3-` by an eight-electron
reduction (−782.6 kJ/mol) and then `NH3` from `NH4+` (+52.2). Read raw,
the two members of one acid–base pair sat 835 kJ/mol apart.

The same error affects every family whose root is a secondary (redox)
master:

- ammonia/ammonium (wateq4f, phreeqc.dat);
- every ferric complex defined from `Fe+3`, which is itself defined from
  `Fe+2` (wateq4f, phreeqc.dat: `FeOH+2`, `FeCl+2`, `FeSO4+`, `Fe(OH)2+`
  and others);
- every ferrous complex in minteq.v4, where `Fe+3` is the master and
  `Fe+2` is defined from it;
- sulfides defined from `H2S`/`HS-`, which are defined from sulfate.

An offline count over the vendored databases: wateq4f 51, phreeqc.dat 22
and minteq.v4 261 species with a stated enthalpy were priced against an
inconsistent basis.

Fix: the database index now chains every species' enthalpy back to the
primary basis (`species_basis_h_kj`). That is the reaction `delta_h` plus
the basis enthalpies of the other species in the reaction, with `H+`,
`H2O` and `e-` at zero (PHREEQC's convention). A species whose chain
reaches a non-master with no stated enthalpy now declines by name. Heats
*within* one family do not change, by construction. Only the inconsistent
cross-root heats move. No new number was introduced. Every value still
comes from the vendored database.

Tests: `derived_species_are_priced_against_the_primary_basis` checks
NH4+/NH3 against the literature on wateq4f and minteq.v4. It also checks
that the stated reaction heat is recovered exactly for NH3/NH4+, FeOH+2,
FeCl+2 and HS-/H2S.

A second symptom stays open and is linked to G1: while the beaker was
wrongly freezing, the same step emitted "the reaction heat of this step is
not modelled … the temperature stays where the step began" for water
after a phase change. That sentence contradicts the temperature event
before it. It should disappear now that the false cooling is gone, but the
wording rule itself was not changed here.

### G2. Metals outside the activity series are silent (missing route, no typed refusal) — OPEN

S03: aluminium in 0.5 M NaOH produced no event at all. The metal stayed
unchanged and no hydrogen formed. `displacement::SERIES` lists only Ag,
Cu, Pb, Fe, Zn and Mg, and the per-metal sentences (`bystanders`) run only
when a series metal is present. No other pass names an unlisted elemental
metal that sits in a characterised solution. The same holds for Sn, Ni and
other metals outside the six.

The amphoteric route itself (`2 Al + 2 OH- + 6 H2O -> 2 Al(OH)4- + 3 H2`)
needs a curated rate. The aluminate species exists in the vendored
databases, but this lab has no reviewed rate. The minimum fix is a typed
`not_yet_modeled` sentence for any elemental metal outside the series. It
needs DE/FR phrase entries and a curiosity-baseline refresh by the
generator on CI, so it is left as a separate change.

### G3. Zinc in dilute acid is blocked by a hydrogen-overpotential rule (refusal arguably too strict) — OPEN

S02: zinc in 0.1 M H2SO4 is reported `inert` with a computed reason
("kinetically blocked … overpotential"). Pure zinc does react slowly with
dilute acid, and school zinc visibly fizzes. The rule is deliberate,
explained and typed. Changing it needs a reviewed rate, not a threshold
tweak. The same rule covers other high-overpotential metals.

### G4. Missing species: basic copper carbonate / malachite, Al2O3 (missing species) — OPEN

S26 was refused at parse: `CuCO3`, `Cu2(OH)2CO3`, `malachite` and `Al2O3`
are all unknown. Adding them needs reviewed formation enthalpies, density
and molar mass with primary citations. Malachite has NBS values. None was
added without that review.

### G5. Concentrated-acid media (model boundary) — OPEN, correct typed refusal

S27 (sugar + concentrated H2SO4) is refused with a typed model-boundary
reason: no validated activity model exists for a concentrated mineral-acid
medium. This is the honest answer. A dehydration route would need its own
curated reaction and its own heat.

### G6. Ammonia gas does not leave a warmed open beaker (missing gas event) — OPEN, re-check after G1

S19 kept all ammonia dissolved. The false freezing came first, so the
gas-release question has to be re-checked once G1 is deployed.

## Fixes

| Gap class | Change |
| --- | --- |
| G1 thermo basis | this PR |
| G2–G6 | open, reasons above |

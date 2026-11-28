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

## 2026-11-28 sweep

Thirty own-design experiments in families earlier sweeps had not covered:
redox titrations, gas collection and gas tests, electrolysis, organic
tests, silver and lead halides, thermal decomposition of carbonates and
nitrates, flame tests, buffers, solubility against temperature, and
indicators. Expectations were written before the scripts were run. The
binary was a release CLI from the gate build of the nitrogen-oxides branch,
close to current main. Tally: 23 pass, 3 fail, 3 gap, 1 bug.

Passes include the starch-iodine colour, Fehling and Tollens with glucose,
AgCl/AgBr/AgI/PbI2 precipitates, AgCl redissolving in ammonia, electrolysis
of CuSO4, acidified water and brine, Daniell cell EMF, CaCO3/ZnCO3/NaHCO3
decomposition, flame colours, acetate and ammonia buffers, peroxide
titration with permanganate, and H2/O2/CO2/NH3 tests over a sealed
headspace.

Gaps, most severe first:

### S1. Titration endpoint "until colour persists" fires early (wrong quantity) — FIXED (fix/titrate-endpoint-20261128: persists now needs a titrant-derived colour; new `until colour disappears`; I2/S2O3 reaches 20 mL with or without starch. Fe(II)/MnO4- completed by fix/permanganate-inert-20261128, see S1a and S1b)

Titrating acidified iron(II) with 0.02 M permanganate stopped after the
first 1 mL. About 25 mL was expected, and 2.48 mmol Fe(II) was still
present. Titrating iodine with thiosulfate stopped at 11 mL with 0.45 mmol
I2 left when starch was present. The same titration without starch
correctly stopped at 20 mL. The detector behaves like "the colour stopped
changing between steps", not "the titrant's colour persists". A
decolourising endpoint (`until colour disappears`) is also missing.
Generic fix: judge the endpoint from the excess-titrant species, or from a
colour change against the pre-endpoint colour, and keep the rule in one
place for all titrants. Acceptance: Fe(II)/MnO4- endpoint within 1 mL of
stoichiometry. I2/S2O3 with starch gives the same volume as without it.

### S1a. Excess permanganate oxidises water in an open acidic flask (wrong chemistry) — FIXED (fix/permanganate-inert-20261128)

Past equivalence the electron balance of Fe/Mn has its root above pe 17,
the top of the coupled search. The coupled solve refused, and the fallback
uncoupled solve fixed pe from atmospheric O2 with manganese unpinned (it was
in two states). PHREEQC then reduced every MnO4- to Mn(II) by oxidising
water, and the oxygen made was booked nowhere. Real permanganate does not
oxidise water on a bench timescale. Generic fix: a table of water-inert
oxidants (`WATER_INERT_OXIDANTS` in aqueous.rs: Mn(VII) 1.507 V, Cr(VI)
1.36 V, sourced). When one of them is the excess, the search continues
above the water ceiling to the oxidant's own standard pe + 4, with the O(0)
master held from forming (a separate engine instance, the same pinning
device as the fast-redox state pins). Manganate and other unlisted states
keep the old ceiling. An above-ceiling answer that leaves a coupled element
in a solid is refused (see S1c). The equivalence point now earns its own
pe (18.6 in the redox tests), between the iron and permanganate couples. Acceptance: the 0.5 mL titration stops at 25 mL
with MnO4- left and Fe(II) consumed. Below equivalence, MnO4- still reacts
completely with Fe(II) (negative control).

### S1b. Thermal fixed point fails at 1 mL titration steps (numerical) — FIXED (fix/permanganate-inert-20261128)

At 1 mL steps the same titration failed at 17 mL with "coupled aqueous
temperature did not converge within 64 passes" (at 0.5 mL steps it failed
near 5 mL). Raising the cap would only hide the cause. The pe search
stopped as soon as the electron residual was below 1e-4 of the inventory
(OPT-7). The accepted root then depended on the search path, and the warm
start changed that path on every thermal pass. The reaction heat in that
slack moved T by about 1e-4 K between passes, which never meets the
1e-6 K tolerance. The same residual was also booked into the next step's
electron target, so the ledger drifted. Fix: locate a bracketed root to
1e-11 of the inventory using Illinois false position (a few evaluations),
write the trial pe with 12 decimals, and keep the verdict tolerance
unchanged. Acceptance: the 1 mL titration reaches the endpoint.

### S1c. The coupled electron balance ignores electrons held in solids (bug) — OPEN

`oxidation_sum` reads dissolved oxidation-state totals only. When a solve
puts a coupled element into a posed phase, that solid's electrons are not
counted, and the pe search compensates. Example: Fe(OH)3 from an
acid-starved flask with 0.005 mol HCl, 0.005 mol FeSO4 and 0.0015 mol
KMnO4. There the found root left 0.635 mmol MnO4- against an expected
0.5 mmol. The above-ceiling search of S1a refuses such an answer and the
beaker is shown as added. The undercount below pe 17 predates this sweep
and is not fixed. Generic fix: add phase moles × state to the sum, using
each phase's oxidation-state composition from its dissolution reaction.

### S2. Sodium nitrate decomposes to sodium metal, with no oxygen (bug) — FIXED (fix/nitrate-decomp-20261128)

Heating NaNO3 to 1335 °C reports NaNO2 and then 0.0117 mol liquid sodium.
No O2 or other gas is booked for either step. Real behaviour is
2 NaNO3 → 2 NaNO2 + O2. NaNO2 decomposes further only to Na2O and nitrogen
oxides, never to the metal. Generic fix: a nitrate-decomposition table
row per cation class, with the gas products and a no-metal-product guard.
Acceptance: about 5.9 mmol O2 from 1 g NaNO3, and no Na(l).

### S3. Heavy-metal nitrates do not decompose (missing reaction) — FIXED (fix/nitrate-decomp-20261128)

Cu(NO3)2 held at 1500 °C stays intact, with 0 kJ to chemistry. The
anhydrous salt is also drawn "white". Expected:
2 Cu(NO3)2 → 2 CuO + 4 NO2 + O2, black solid, brown gas. This could share
the S2 table: an alkali row gives nitrite + O2, and a row for other metals
gives oxide + NO2 + O2. Each needs reviewed onset temperatures.

Fix (S2 and S3): `kerotakis_core::nitrate_decomposition` holds one row per
cation class (group 1 except Li → nitrite + O2; noble metals → metal + NO2 +
O2; every other metal → oxide + NO2 + O2) and one cited onset and reaction
enthalpy per salt, from Stern, J. Phys. Chem. Ref. Data 1 (1972) 747 and the
registry's reviewed decomposition points: NaNO3 873 K, KNO3 923 K,
Cu(NO3)2 500 K, AgNO3 513 K, Pb(NO3)2 539 K. The reaction is paid from the
heat the vessel holds above the onset. The Gibbs route no longer maps the
nitrates and nitrites, and no longer offers an alkali metal as a product
unless the charge already holds that metal. KNO2 joins the registry.
Cu(NO3)2 is drawn blue-green (word-colour table). Pb(NO3)2 is refused with a
typed note because PbO is not yet a registry species.

### S4. No haloform (iodoform) route (missing reaction and species) — FIXED (fix/haloform-indicators-20261128)

Ethanol with I2/KI/NaOH, warmed, gives no reaction and no CHI3. CHI3 is
not a species. Generic fix: a haloform family row (CH3CO– or CH3CH(OH)–
substrates, X2/OH-) plus a CHI3 registry row (yellow solid, sourced data).

### S5. Methyl orange has no acid-base colour; bromothymol blue colours off (wrong observation) — FIXED (fix/haloform-indicators-20261128)

Methyl orange stays "red" at pH 12.2, and the notice says it has no aqueous
mapping. Bromothymol blue is drawn orange at pH 1.75 and purple at pH 12.2,
where yellow and blue are expected. Generic fix: indicator table rows with
pKa and the two-form colours, using the existing phenolphthalein-style
indicator route.

Fix (S4 and S5): `kerotakis_core::haloform` holds one row per substrate
(propanone, propan-2-ol, ethanol), each a balanced overall equation checked
for atoms and charge; warm (editorial 50 °C onset) alkaline I2 gives CHI3 as
a yellow solid plus acetate (or, from ethanol, formic acid held molecular).
Methanol has no row and stays negative. CHI3 joins the registry (PubChem CID
6374, CIAAW mass, CRC/HSDB density 4.008 g/mL, HSDB colour "yellow", Kopp
Cp). Indicators: `Indicator::form_word` names the sourced acid colour below
the CRC transition range and the base colour above it, for every two-colour
indicator in the table; inside the range the computed hue is kept. Methyl
orange's pKa 3.47 now cites Sandberg et al., J. Phys. Chem. 76 (1972) 4023.
Still open: methyl orange keeps its "ionic solute without an aqueous
mapping" notice (its sulfonate is not speciated), and the mixed-band colour
of an overdosed dye is still the computed hue.

### S6. No KNO3 crystallisation on cooling; cooling passes the freezing boundary (missing species) — OPEN

25 g KNO3 in 20 g water stays fully dissolved down to −67.9 °C, still
"colourless and clear". A typed notice says that no validated KNO3 phase
exists. The −21.1 °C partial-freezing boundary is announced, but the
temperature then keeps falling with the liquid unchanged. Fix: a reviewed
KNO3 solubility phase (a data row). Separately, `cool` should stop or
refuse at the freezing boundary it announces.

### S7. Minor observations — OPEN

- HCl added to sealed plain water raises the "acid and carbonate fizz"
  hazard. The only carbonate present is dissolved air CO2, so the hazard
  trigger needs a threshold.
- 0.05 mol CuSO4 in 50 mL water (1 M) precipitates 8.4 mmol chalcanthite,
  although 1 M is below its solubility (about 1.4 M).
- When NaHCO3 is heated, −1.44 kJ "went into chemistry". The sign is
  inconsistent with an endothermic decomposition.
- AgBr is drawn "yellow" rather than cream. Hot ZnO is not drawn yellow.
- Gas tests on an open beaker answer "nothing to test, collect over a
  sealed vessel". This is correct and typed, and is noted here only so that
  scripts seal first.

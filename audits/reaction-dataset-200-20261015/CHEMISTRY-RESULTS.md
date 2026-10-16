# What the 200 reaction probes actually established

All 200 original scripts were executed through both CLI and production API.
They agree for every case: 92 completed and 108 stopped at an unknown identity.
That agreement establishes execution consistency, not expected chemistry.

The retrospective review of all saved results is in
[JSON](chemistry-screening.json) and [CSV](chemistry-screening.csv):

| Qualitative screen | Cases |
| --- | ---: |
| Principal source observation appears | 13 |
| Some expected behavior, with a material gap | 11 |
| Target outcome not demonstrated | 16 |
| Explicit model limitation | 20 |
| Protocol or interpretation inconclusive | 32 |
| Unknown input identity stops execution | 108 |

These categories direct repairs. They are not calibrated scientific pass counts;
protocol and product causes can overlap. The supplied records have no procedures,
and the generated probes do not supply every missing experimental condition.
Original scripts, observations and earlier blind forecasts remain unchanged.

## Complete-reagent controls resolve several apparent pH defects

The adapter interprets non-H/O analytical totals through PHREEQC charge-adjusted
pH. Bare NH4+, carbonate or H+/OH− additions are therefore not interchangeable
with complete physical reagent stocks. Treating them as such produced misleading
interpretations of the original screen.

Twelve separate protocols were frozen before execution through the same API and
engine. [Their design](stock-followups-20261015/design.json) records doses,
order, boundary and forecasts; [their assessment](stock-followups-20261015/assessment.json)
binds every script and response hash. All twelve completed. Nine show their
principal qualitative response, one reaches an explicit model limit, and two
retain missing target chemistry.

| Complete-reagent control | Observed result |
| --- | --- |
| Water | pH 6.9974 |
| HCl | pH 2.0381 |
| NaOH | pH 11.9517 |
| Equal HCl and NaOH | pH returns to 6.9927; temperature increases |
| NH4Cl | Acidic, pH 5.6637 |
| Na2CO3 | Alkaline, pH 10.9725 |
| CaCO3 plus HCl | Carbonate consumed and CO2 evolved |
| CuO plus HCl | Oxide consumed and dissolved copper represented |
| CO2 plus NaOH | Carbonate/bicarbonate formed; alkaline endpoint |
| Zinc plus HCl | Zinc remains; overpotential/model limits disclosed |
| AgCl plus NH3 | Silver ammine complexes absent; dissolution enhancement absent |
| Acidified KMnO4 plus H2O2 | Permanganate retained; coupled redox reaction absent |

Thus the original neutralisation, ammonium, carbonate, oxide/acid and CO2/base
screens do not justify rewriting the established stock chemistry. Bare-basis
inputs still need an honest reporting boundary. The stock controls establish no
empirical rate, heat, optical or exact-yield calibration.

## Ordered repairs

Each lane must preserve the original evidence, start from current main, add
positive and negative controls, pass its focused acceptance and applicable full
chemistry fleets, then use normal protected auto-merge. Do not replace a failed
probe in place or count a warning as implemented chemistry.

1. **Explain incomplete ionic inputs.** [PR796](https://github.com/CrispStrobe/kerotakis/pull/796)
   withholds solvent-only readings for explicit bare acid/base coordinates and
   discloses uncompensated analytical-component charge. Preserve complete-stock
   predictions, inventory, typed interfaces and counterion recovery. Eight
   regression groups are prepared; hosted execution is pending. A future physical
   ion-input convention needs separate versioning, water and energy accounting.
2. **Select silver–ammonia capability.** The source candidate is
   [PR798](https://github.com/CrispStrobe/kerotakis/pull/798), with four focused
   groups awaiting hosted execution. Its named limitation preserves the distinction
   between equilibrium speciation and canonical free-ligand availability. Inspect
   `crates/kerotakis-phreeqc/src/aqueous.rs` database selection. The default dataset
   lacks silver ammines; vendored `minteq.v4.dat` already contains sourced
   AgNH3+ and Ag(NH3)2+ equilibria. Select that capability for silver with reduced
   nitrogen, preserving nitrate-only controls and explicit provenance. Require
   finite-ligand dissolution enhancement, acid-protonation suppression, complete
   Ag/N/Cl and charge accounting, and repeat stability. Do not require complete
   dissolution at the declared finite ammonia dose.
3. **Establish shared post-speciation and thermal ownership, then add the endpoint.**
   The [implementation lanes](../../docs/acid-permanganate-peroxide-lane.md)
   explain why a fresh-pH reaction cannot run before the aqueous tail and why a
   temporary thermal snapshot swap can reprice unknown reaction heat on a later
   phase pass. Implement the shared native/browser transaction and rollback first.
   Freeze an endpoint
   contract for `2MnO4− + 5H2O2 + 6H+ → 2Mn2+ + 5O2 + 8H2O`. Require actual acidic
   conditions, finite acid/reactant limits, spectator conservation, molecular and
   dissolved input agreement, and open/sealed oxygen accounting. Do not globally
   couple oxygen/water into fast redox, guess a rate or invent heat. Neutral/basic
   chemistry requires distinct product contracts and remains outside this lane.
4. **Resolve identity refusals individually.** The 108 refusals contain 80
   distinct first blocked tokens, listed in the screening JSON. For each, classify
   a proven spelling alias, a missing species/material, a phase encoding, or a
   schematic entity. Preserve formula, charge, isomer, phase and stock identity.
   Add paired parser/finite-bottle controls for aliases; require sourced properties
   and explicit model scope for new species. An electron, catalyst or generic
   peptide cannot be repaired by silently dispensing a different chemical.
5. **Make incomplete protocols executable experiments.** Prioritize zinc/acid,
   organic photochemistry, electrolysis, gas synthesis and biological records.
   Freeze explicit medium, catalyst, temperature, boundary and activation controls
   separately; retain a named limitation where these are unavailable. Replace
   arbitrary heating assumptions only in new protocols. A sealed vessel bursting
   and later reacting in air does not validate closed-vessel combustion.
6. **Audit remaining quantitative/observable gaps.** Reproduce rank 126 with a
   complete ferrous stock before attributing apparent solid/solution double
   counting to conservation. Compare analytical inventory against solution
   molalities using actual solvent mass and complex stoichiometry. Follow the
   [finite KNO3 lane](../../docs/kno3-finite-solubility-lane.md), missing AgBr
   precipitation, FeSCN and precipitate spectra, and equilibrium readback churn
   with separate conservation, phase and appearance controls. Missing spectra
   are not evidence that a reaction did not occur.

The watercolor and vessel-animation work has its own accepted implementation;
its acceptance does not close these chemical coverage lanes. The production API
still reports its pinned engine revision, so later merges do not silently change
these saved results.

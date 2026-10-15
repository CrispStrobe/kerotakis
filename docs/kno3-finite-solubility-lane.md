# Finite salt solubility: reproduced failure and repair lanes

The unchanged nitrate control now fails on pinned merged source `d791d306`:
all 20 g KNO3 dissolves in 50 g water at 20.004 °C, leaving no solid. The hot
sample also dissolves all 20 g at 79.955 °C. Read the [authenticated assessment
and raw observations](../audits/fresh-syntax50-20261013/kno3-current-main/assessment.json).
Original forecasts and earlier outcomes remain immutable. This is a reproduced
physical failure; a successful process exit does not close it.

## Source diagnosis

The following locations refer to `d791d306`; refresh them before editing.

`crates/kerotakis-phreeqc/src/derived.rs` indexes WATEQ4F, MINTEQ and Pitzer
around lines 747–751. None of these active databases defines Niter/KNO3.
Mineral admission around 903–914 therefore does not apply. The bounded-solid
guard around 968–974 requires a reviewed aqueous-solubility record; KNO3 has
none. It falls through to `Dissolves`.

`crates/kerotakis-phreeqc/src/aqueous.rs` then contributes its entire potassium
and N(5) inventory to solution totals around 1815–1820. The saturation estimate
around 1494 only affects routing. This case does not first reach the later
missing-phase reconciliation path around 3866. Compare NaCl, admitted as the
native Halite mineral, to distinguish finite equilibrium from unconditional
dissolution.

The pinned vendored `vendor/iphreeqc/database/llnl.dat` contains a Niter record
around 14355–14361, with dissolution log K, enthalpy and an analytical
temperature law. Data existing in an inactive database does not establish
compatibility with active activity models. The active Pitzer database contains
potassium interactions but no nitrate master, nitrate interactions or Niter;
its neutral nitrogen gas component is a different identity. Consequently this
concentrated nitrate inventory falls back to WATEQ4F with the existing
concentration-validity caveat.

## Ordered, independently reviewable tasks

1. **Define the missing-phase contract.** Inventory every solid assigned
   `Dissolves` without an admitted mineral or reviewed finite-solubility model,
   and every positive solid phase freed during route fallback. Classify intended
   support and document the dilute-speciation versus finite-saturation boundary.
   Add controls that distinguish real dissolution from unsupported phase removal,
   preserving elemental inventory and the existing failure evidence. Deliver a
   source-bound inventory and explicit runtime coverage policy. A warning alone
   does not repair the KNO3 equilibrium, and changing this result to unsupported
   does not satisfy its frozen physical forecast.
2. **Review complete Niter phase admission.** Trace how existing database
   extensions reach both the derived index and the actual solver bytes. Review
   the full LLNL reaction, standard-state compatibility, temperature law, units,
   validity and provenance. The current foreign-phase helper handles hydroxides;
   it cannot safely synthesize Niter or discard its temperature dependence.
   Deliver a narrow phase extension only after data review, with cold/hot,
   undersaturated/saturated, reverse-path and conservation tests. Preserve
   concentrated-activity caveats; phase admission alone earns no quantitative
   concentrated-solution claim.
3. **Close the activity-model gap.** Establish independently reviewed nitrate
   activity parameters and their concentration/temperature domain, including
   required mixed-ion interactions and compatibility with the Niter phase law.
   Adding a nitrate master alone is insufficient. Verify route selection and
   refusal boundaries, potassium/nitrate/water conservation, concentration
   dependence, temperature dependence and physical readback. Require the frozen
   cold/hot control plus independent reference controls before claiming closure.
4. **Consider a separate bounded pure-water model.** A reviewed KNO3 solubility
   curve could support a narrower pure-water domain through existing finite-solid
   machinery. Specify common-ion, mixed-solvent and unsupported-mixture refusals
   explicitly. Preserve ion-speciation limits; do not present this as a general
   mixed-electrolyte equilibrium repair.

Do not change amounts, optical values or forecasts to obtain a pass. Freeze each
new candidate/source/binary/lock association separately. Keep phase admission,
quantitative solubility and empirical calibration claims distinct.

# Preserve paid condensed phase fractions through CEA

A separate smaller native control reproduces the saved zinc discrepancy against
accepted source `5194f961`, binary `d21fa912`. Its relevant phase/CEA/stack/aqueous
source matches integrated main `e63a0824`; the separately tested parser differs.
This is an exact-source native reproduction, not a combined-main deployment claim.

Read [original design, script and raw results](../audits/target468-20261016/paid-metal-phase-first-control/)
and [the first assessment](../audits/target468-20261016/paid-metal-phase-first-control/assessment.json).
Adding0.1mol Zn at400°C then100J announces0.00546802095mol melting, but ownership
contains all0.1mol as liquid and no solid. Represented Zn is conserved; the phase
and energy result is incorrect. No experiment or forecast was changed to hide it.

The CEA charge currently omits owned phase from its enthalpy mapping, the HP
search can stop near a discontinuous condensed-phase transition, and readback
unconditionally overwrites contents even when no chemical event is reported.
The prepared repair addresses the destructive readback boundary: for an
adiabatic condensed-only charge on a supported core solid/liquid coexistence
plateau, preserve the already paid fractions when CEA returns the same chemical
amounts and unchanged atmosphere.
Use roundoff-level comparisons and exact species sets, not visible-event cutoffs.
A missing latent-heat route, a state away from the plateau, new gas, consumed
atmospheric oxygen, owned gas/aqueous matter or changed chemical
amounts prevent the handoff. Real oxidation remains on its chemical route.

The regression controls cover Zn and Pb below/within/above the latent budget,
unchanged repeated settlement, reverse cooling, liquid pouring, a naphthalene
control, explicit trace-gas/amount exclusions and actual magnesium oxidation.
A clock wait is a different physical boundary: the clock includes ambient heat
exchange. Its original invariant assumption is preserved and explicitly qualified;
repeated unchanged settlement, rather than a time advance, tests invariance.

## Acceptance still required

Formatting and source review are complete. The repaired code has not yet been
compiled or executed: native/Rust resource gates are closed locally, and hosted
validation is pending. Keep the repair draft until actual native controls on both
platforms and the full original own fleet pass on its exact reviewed head.
Integrate only after the pending accepted production-delivery checkpoint, through
normal protected exact-head auto-merge, then bind a separate repaired-source replay.
No catalog addition, deployed fix or empirical calibration is claimed.

## Separate remaining thermochemical scope

This handoff does not implement general multiphase HP equilibrium. Phase-aware
incoming CEA enthalpy, discontinuous-root energy residuals and genuinely reacting
partially molten charges remain a distinct follow-up. Freeze independent element,
phase and energy controls before changing those algorithms; preserve actual
reactant phases and refuse unsupported energy closure rather than returning a
pure phase at a nearby temperature. Do not fit heat capacities or latent heats
to the original zinc point. Continue the [scoped phase lane](saved64-review-lanes-20261018.md#c-preserve-latent-phase-fractions-through-downstream-metal-writers).

## First hosted validation failure

The first macOS native job at source `f309ec3c` stopped in Clippy before
the physics tests: the new test module preceded production items
(`items_after_test_module`). The correction moves that entire unchanged module
to the end of `crates/kerotakis-cea/src/thermal.rs`; no assertion, runtime
behavior or lint policy changes. Formatting and diff checks pass. Actual native
physics and complete fleet acceptance still belong to the corrected source.
The [original failed job](https://github.com/CrispStrobe/kerotakis/actions/runs/39119175824/job/117651062431)
remains the first validation outcome, not a scientific failure or pass.

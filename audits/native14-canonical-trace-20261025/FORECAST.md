# Native ammonia and carbonate canonical-pass trace forecast

## Source and preserved first failures

This bounded diagnostic starts from the exact source revision `6e44feab03a646b412c123b21b275e3b95942931` (tree `e4aa4c8dad734b370c77100ea47a1c201d3c4a50`). The captured full-test failures on that source are:

- `carbonate::strong_acid_on_baking_soda_also_fizzes`: the existing assertion requires more than `0.015 mol` evolved CO2; the failure reported `-0 mol`.
- `coupled_ammonia::acid_trapping_couples_finite_ammonia_gas_and_protonation_in_both_addition_orders`: execution emitted `SolverFailed` with the existing detail that the physical aqueous/gas answer did not close within eight passes.

These failures remain the observed baseline. The trace is intended to expose each existing native pass and the exact before/after represented state at that boundary; it does not change either test's setup or expected result.

## Instrumentation scope

Add diagnostic-only output inside the existing canonical aqueous pass driver. For each first native answer and each reentry answer, record the call index, stage, requested/returned database route, clock CO2 amount, heat quote, events, whether the existing physical-closure predicate was true, and the before/after contents, complete solution fields, free proton/hydroxide, solute charge, temperature, pending clock transfer and headspace. Output is one JSON object per line prefixed `CANONICAL_DIAGNOSTIC`.

The instrumentation must not alter any input, state, route, event, heat, tolerance, iteration cap, or decision. It must not add assertions or select scientific outcomes. Only the two named existing native tests are run in the later private worker; both exit outcomes are preserved even when they fail. No other target, full sweep, CLI, provider, or original is selected by this trace.

## Interpretation limits

The trace is observational evidence for these two concrete failures only. It is not a chemistry repair, model calibration, pass, or qualification credit. A later source change requires a separate frozen forecast and source-bound validation. Existing original counts and every other acceptance gate remain unchanged.

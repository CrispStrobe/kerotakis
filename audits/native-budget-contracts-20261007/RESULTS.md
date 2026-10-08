# Native budget baseline

[Hosted run 37704022496](https://github.com/CrispStrobe/kerotakis/actions/runs/37704022496)
compiled and executed 25 contracts at `3746d277`: **11 passed, 14 failed**.
The seven original raw component guards passed. Of 18 new allocation and
reconciliation controls, four passed and fourteen failed.
[The source-bound baseline receipt](baseline.json) lists every outcome and
binds the raw artifact/log hash.

Demonstrated failures include Ca/Sr and closed-carbon overdraw, malformed
owned gas and raw aqueous values, pure-phase co-owner omission, arbitrary
rescaling and synthesized missing aqueous inventory. Missing owned gas already
refuses; it must keep doing so.

Eight additional contracts are frozen before production edits:
[receipt](supplement-freeze.json). They cover an independently chosen numerical
cap boundary, aggregate overflow, malformed initial owners and open carbon.
The cap separates native residual (1e-7 relative), selected-output rounding
(5e-12 relative for twelve fractional digits in scientific notation) and positive-sum
arithmetic gamma(n). It is an adapter acceptance ceiling, not empirical
validation of PHREEQC accuracy. Comparisons have no absolute inventory floor.
The raw cap scales to the aqueous/remainder comparison; the phase cap scales
to available/final owned totals. Zero raw aqueous with a positive remainder
refuses even when the remainder is small.

The initial and expanded pre-repair baselines are recorded below; repair validation remains pending. Live native
precipitation/dissolution/repeated equilibrium and native/stack rollback must
also pass before claiming this bounded repair accepted.

## Expanded baseline and reviewed refinement

[Run 37728961663](https://github.com/CrispStrobe/kerotakis/actions/runs/37728961663) executed 33 contracts before production changes: **13 passed, 20 failed**. [The supplemental baseline](supplement-baseline.json) records every outcome.

Nine additional [refinement controls](refinement-freeze.json) cover binary cancellation, mandatory raw carbon columns and realistic external phases. Their separate baseline runs against the first repair, before refinement. Compensated sums preserve representable low components; owner-scale arithmetic roundoff remains separate from the aqueous-relative native cap. An accepted arithmetic-only discrepancy leaves the raw trace unchanged.

Vendored `print.cpp:punch_totals` reads aqueous master totals; `model.cpp` builds those from species, and `prep.cpp` constructs aqueous species-list entries. `basicsubs.cpp:system_total_elt` explicitly includes additional owners for SYS, so a scalar SYS total is not an aqueous witness. The old claim that selected `-totals` legitimately includes mixed-crystal inventory is unsupported. This repair accepts the reviewed exclusive route, refuses unwitnessed inclusive totals, and isolates mixed interface co-owners. A future diagnostic witness can sum only SYS entries labelled `aq`.

Strict carbon validation exposed a source-informed fixture omission: the original seven raw guard contracts did not supply the `C(-4)` column that direct and MIX builders request on the WATEQ4F route. The original file is preserved unchanged. A separate corrected harness adds only that zero column, with all seven assertion bodies unchanged; [its adaptation receipt](raw-harness-adaptation.json) binds both forms. No original forecasts were regenerated and no test identity is recognized by production.

Production refinements and the complete-column harness await hosted native and live precipitation/dissolution/repeat validation. No accepted repair claim is made yet.

## Latest source and pending gates

The current production repair is [9140ee58](https://github.com/CrispStrobe/kerotakis/commit/9140ee58). The [50 boundary contracts](README.md) include external-phase validation and complete native/stack refusal preservation. [Native/live validation 37730149327](https://github.com/CrispStrobe/kerotakis/actions/runs/37730149327) is queued. Refinement baseline [37729480945](https://github.com/CrispStrobe/kerotakis/actions/runs/37729480945) completed as described below; external-phase baseline [37730066599](https://github.com/CrispStrobe/kerotakis/actions/runs/37730066599) completed as described below. No results are inferred from queue state.

The earlier unstarted repair dispatches 37729120125 and 37729728076 were cancelled and superseded by the complete native/live scope. They are not acceptance evidence. [The source inventory](repair-source-9140.json) binds the production module and frozen/adapted contracts; original receipt files remain unchanged. This repair branch is not yet integrated into main or released.

## First-repair refinement baseline

[Run 37729480945](https://github.com/CrispStrobe/kerotakis/actions/runs/37729480945) compiled and executed 42 cache-only contracts at `5767d34ffcea3bfc8ad764499072300226b96821`: **29 passed, 13 failed**. [The immutable receipt](refinement-baseline.json) binds the artifact log and every outcome. The original seven raw component controls passed; the original allocation/reconciliation group passed 14 of 18; the supplement passed seven of eight; the nine new refinement controls passed one and failed eight.

The newly frozen failures demonstrate cancellation refusal, two external gas-phase identity refusals, a noncarbon boundary incorrectly opening carbon, and four missing/malformed raw carbon bypasses. Complete zero-carbon readback passed. These are actual executed assertions, not compilation or workflow failures.

Five older failures have a distinct fixture boundary ambiguity: the allocation and supplemental helpers start from an open vessel, clear `problem.phases` and `problem.gases`, but retain atmospheric CO2 in `problem.external_gases`. The affected assertions are closed-carbon gas overdraw, ten-percent raw carbon deficit/excess, missing aqueous carbon with a positive remainder, and malformed initial closed gas. Those results cannot independently establish a failure under a genuinely closed carbon boundary. Original bytes and outcomes remain preserved; a corrected closed-boundary harness must disclose its adaptation and obtain fresh hosted evidence.

This baseline precedes the production refinements and is not acceptance evidence. It contains no live precipitation/dissolution/repeated-equilibrium execution.

## External-phase and refusal baseline

[Run 37730066599](https://github.com/CrispStrobe/kerotakis/actions/runs/37730066599) compiled and executed 50 cache-only contracts at `142f2e68b50106a964e1779f5c1d58e73de3c4c5`: **41 passed, nine failed**. [The immutable receipt](external-refusal-baseline.json) binds the artifact log and every outcome. All nine refinement contracts and all seven complete-column raw component controls passed.

Four external-phase controls failed: negative initial/final amounts were accepted, and both nonfinite controls failed at their first loop value, NaN. The later infinity values were not reached in this failing baseline. Zero boundary amounts accepted and missing final amounts refused. The remaining five failures are the already documented closed-boundary fixture ambiguities; this run still used the original allocation and supplemental helpers.

Both cache-injected refusal controls passed: native Ca overdraw refused with its expected diagnostic and preserved the complete serialized vessel; a stack containing that native route recorded failure, emitted the failure event and preserved the vessel. This is evidence for this rejection path with fabricated complete cached output, not live chemistry or a comprehensive transaction matrix. Corrected closed-boundary harnesses and the external-phase production repair require their own hosted validation. The pending native/live run is not replaced or treated as successful by this baseline.

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

The current production repair is [9140ee58](https://github.com/CrispStrobe/kerotakis/commit/9140ee58). The [50 boundary contracts](README.md) include external-phase validation and complete native/stack refusal preservation. [Native/live validation 37730149327](https://github.com/CrispStrobe/kerotakis/actions/runs/37730149327) is queued. Refinement baseline [37729480945](https://github.com/CrispStrobe/kerotakis/actions/runs/37729480945) and external-phase baseline [37730066599](https://github.com/CrispStrobe/kerotakis/actions/runs/37730066599) are queued separately. No results are inferred from queue state.

The earlier unstarted repair dispatches 37729120125 and 37729728076 were cancelled and superseded by the complete native/live scope. They are not acceptance evidence. [The source inventory](repair-source-9140.json) binds the production module and frozen/adapted contracts; original receipt files remain unchanged. This repair branch is not yet integrated into main or released.

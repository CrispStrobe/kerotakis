# Native phase budget and reconciliation freeze, 2026-10-07

These 18 source-informed tests exercise the production
`PhreeqcEquilibrator::apply_balance_corrections` seam directly without running
a PHREEQC solve. This is a frozen baseline proposal, not an implemented repair
or a captured failure report. [The receipt](freeze.json) binds the contracts.
The only source edit is a `cfg(test)` module inclusion.

Independent budget: initial aqueous Ca 0.002 and C 0.002 mol plus typed
CaCO3 0.003 and SrCO3 0.004 mol. Available Ca/Sr/C is 0.005/0.004/0.009.
Final typed 0.002/0.003 leaves aqueous 0.003/0.001/0.004. Pure Calcite
co-owners must be added/debited exactly once; closed CO2 owns one carbon each.
Scale controls repeat at 1, 1e-12 and 1e-150 without an absolute tolerance floor.

Expect exact exclusive remainders, exhaustion and valid primary/gas ownership
to accept. Expect elemental overdraw, negative/nonfinite/missing owned gas,
malformed raw aqueous amounts, ten percent raw discrepancies, absent positive
aqueous matter and apparently inclusive totals without a witness to refuse.
The supported exact controls use a 32-epsilon arithmetic assertion bound;
this is not a selected-output or native solver convergence allowance.

Run the existing hosted `pure-still-contracts` dispatch with
`focused_scope=crystal-readback` on this branch. Its existing substring filter
includes this new module and the seven unchanged original raw guards.
Preserve failures as baseline evidence before production edits. A compile or
harness failure is not a demonstrated engine failure.

Still required before the repair: aggregate overflow and boundary-budget
fixtures; a justified separate selected-output/arithmetic/solver allowance;
external-carbon controls; witnessed inclusive representation if supported;
live precipitation/dissolution/repeat tests; full-state refusal/rollback
through native and stack callers. Do not claim full H/O/charge, speciation,
heat accuracy or general solid-solution chemistry.

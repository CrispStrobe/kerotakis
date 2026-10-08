# Native phase budget and reconciliation freeze, 2026-10-07

These 18 initial source-informed tests exercise the production
`PhreeqcEquilibrator::apply_balance_corrections` seam directly without running
a PHREEQC solve. At freeze commit `3746d277`, the only source edit was a
`cfg(test)` module inclusion. [The immutable receipt](freeze.json) binds those
contracts. [RESULTS.md](RESULTS.md) records the executed failing baselines and
subsequent repair work; freezing expectations does not establish acceptance.

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

The current suite has 50 boundary test functions: 18 initial allocation
controls, eight numerical controls, nine cancellation/carbon/boundary controls,
eight external-phase/refusal controls and seven original raw guards through
the explicitly corrected complete-column harness. The original seven-test
source is preserved unchanged; see [the adaptation](raw-harness-adaptation.json).
These are source-informed boundary contracts, not fifty blind CLI experiments.

Hosted `pure-still-contracts` with `focused_scope=crystal-readback` runs the
boundary contracts without a native engine. `focused_scope=native-budget`
runs the native library tests plus existing live crystal precipitation,
dissolution and repeated-equilibrium controls. Baseline, harness failure and
accepted repair outcomes must be reported separately.

The production guard at `9140ee58` checks nonduplicated Ca/Sr/C owners and
finite nonnegative gas/phase/aqueous values, uses compensated owner sums,
bounds raw reconciliation and preserves arithmetic-only trace discrepancies.
Only a CO2 external boundary opens carbon. Interface co-ownership and
unwitnessed inclusive reporting refuse. Hosted validation and complete-state
refusal controls remain pending; do not claim H/O/charge, speciation, heat
accuracy or general solid-solution chemistry.

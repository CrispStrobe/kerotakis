# Next bounded native reconciliation change

This is a source-review plan, not an executed experiment or a completed repair. Freeze independent controls before changing native Ca/Sr/C reconciliation.

First certify the phase budget before computing any nonnegative aqueous remainder: initial aqueous, primary phases, typed components and closed gas provide the available element inventory. Final primary phases, typed components and closed gas consume that inventory. Finite nonnegative amounts and finite aggregates are mandatory. A phase allocation beyond the documented numerical budget must refuse rather than become a zero aqueous remainder. Missing gas values must not vanish through `filter_map`.

Then compare raw aqueous totals with the independently computed remainder before rescaling. The existing comment about databases reporting mixed phases inside or outside selected totals is not established by the reviewed native source: vendored [print.cpp](../../vendor/iphreeqc/src/phreeqcpp/print.cpp) around line 2798 punches master totals divided by aqueous water mass; [model.cpp](../../vendor/iphreeqc/src/phreeqcpp/model.cpp) around lines 4939–4984 accumulates master totals from species amounts. A discrepancy equal to mixed-phase inventory is not by itself a certificate for inclusive reporting. Surface/exchange co-owners need separate witnesses or a narrow refusal policy.

If a real inclusive route is established, record an independent aqueous witness from the same solve and normalize only when that witness, the total and explicitly identified owners agree. Do not infer or synthesize missing matter from the pre-solve budget.

Candidate independent fixture: initial Ca 0.005 mol, Sr 0.004 mol and C 0.009 mol; final mixed components CaCO3 0.002 mol and SrCO3 0.003 mol. Exclusive aqueous remainders are Ca 0.003 mol, Sr 0.001 mol and C 0.004 mol. Forecasts to freeze:

- Exact supported remainders accept without material correction.
- Final CaCO3 0.006 mol or SrCO3 0.005 mol refuses the corresponding overdraw.
- Closed CO2 gas taking final carbon above 0.009 mol refuses; missing, negative or nonfinite owned gas refuses explicitly.
- Raw aqueous results 10% above/below the remainder, or zero with a positive remainder, refuse unexplained residuals.
- An apparently inclusive selected total without a separate same-solve witness refuses unsupported normalization.
- A confirmed inclusive representation with a matching aqueous witness normalizes while preserving all identified ownership.
- Scaling the entire inventory down preserves the same fractional refusal; no absolute tolerance floor conceals a trace discrepancy.
- Valid native precipitation, dissolution and settled repeated equilibrium remain supported.
- External CO2 qualifies carbon as open while closed Ca/Sr budgets still apply.

Separate output rounding, arithmetic error and solver residual in a scale-dependent budget. Freeze controls clearly inside/outside that budget before choosing implementation thresholds. This change would establish bounded Ca/Sr/C reconciliation only, not complete H/O/formal-charge closure, accurate speciation, calibrated reaction heat or general solid-solution chemistry.

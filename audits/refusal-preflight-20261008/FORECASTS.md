# Refusal and preflight contracts

This bounded lane is stacked above the safety-veto repair at `f6ad9946fdf71ab3273dfb5d46350e569c7f5cb6`. Expectations are source-informed and frozen before production changes or execution. Hosted baseline and repaired results remain pending; static inspection is not acceptance.

## Contract and scope

A command that explicitly refuses before physical work must retain its diagnostic and append its log entry, preserve the complete Bench otherwise, and call no solver. An invalid command returning BenchError must preserve the complete Bench including its log and keep its existing exception type. Compare serialized state and use a counting solver that changes temperature/free proton if called.

The frozen suite is `crates/kerotakis-core/tests/refusal_preflight_atomicity.rs`, with 28 independent test functions after macro expansion:

| Domain | Cases | Forecast |
| --- | --- | --- |
| Drain, single liquid phase | Existing and absent receiver | Diagnostic only; no receiver creation or settlement. |
| Drain, unreviewed iodine partition temperature | Water/hexane/I2 at 300 K, existing and absent receiver | Temperature-domain refusal preserves graph and inventories; no settlement. |
| Evaporate, no water | Ethanol present, fraction 0.5 | Diagnostic only, no settlement. |
| Unsupported nuclide | Xe-135, positive amount | Unsupported teaching-set diagnostic, preserved state. |
| Unknown direct reaction | Direct Operator with an unknown non-selectivity name | Unknown-reaction diagnostic, preserved state. |
| Successful/partial success controls | Layered Drain; reviewed iodine Drain; partial evaporation with ethanol notice; complete evaporation with stranded salt notice; I-131 spike; saponification | Physical changes and settlement retained. Successful fresh Drain creates its receiver before its success event. |
| Invalid Decant | Low/high/NaN fraction; missing source; absent self-transfer | Existing exception; no receiver, inventory, stock or log changes. |
| Invalid Filter | Missing source; absent self-transfer | Existing exception; no receiver or other mutation. |
| Already-safe Drain | Missing source; absent self-transfer | Existing exception and full-state preservation retained. |
| Already-safe Extract | Zero/NaN solvent; zero stages; unknown solvent; missing source; self-transfer | Existing exception and full-state preservation retained. |

The inherited 20 `safety_veto_atomicity` controls additionally freeze accepted Decant/Filter receiver creation and creation/warning/success event order, seven veto paths, and the intentional accidental-spill exception. Run both suites for repaired acceptance.

## Bounded implementation and exclusions

Only explicit refusal branches gain Unchanged disposition. Drain must not create a destination until its read-only refusal checks finish. Decant/Filter must validate existing errors before creating a receiver, while successful transfer creation/event order stays intact. No generic NotYetModeled rule, full-Bench snapshot rollback, new exception type, or public API change is intended.

Evaporation notices after water removal and the curated reaction boundary describe partial success; they must continue settlement. This lane does not change zero-fraction semantics, unsupported-nuclide numeric validation, general selectivity/curated-reaction refusals, stock behavior, thermal kernels, native chemistry or distillation. Source locations before this lane: `crates/kerotakis-core/src/bench.rs`, branches Drain, Evaporate, SpikeNuclide, React, Decant and Filter. Reviewed iodine temperature bounds are in `crates/kerotakis-core/src/apparatus.rs::PARTITION_COEFFICIENTS`.

## Reproduction and evidence

Baseline source is the parent safety-veto head with this frozen test file and no new production repair. Execute `cargo test -p kerotakis-core --test refusal_preflight_atomicity --test safety_veto_atomicity -j1 -- --test-threads=1` on a hosted runner. Preserve compiler failures separately from observed contract failures, raw logs, exact revision/toolchain, manifests and generated lock hash. Do not alter frozen assertions after seeing outcomes; disclose any harness correction with a new receipt. Full required CI and an integrated-source audit remain additional acceptance gates.

# Per-dose titration safety

This source-informed round closes the composite titration screen gap identified by the safety/transfer audit. The original eight-contract freeze at `9001e50` had a missing explicit `ops::Endpoint` import. Initial run [37185433087](https://github.com/CrispStrobe/kerotakis/actions/runs/37185433087) stopped at compilation; no behavioral tests ran. `initial-freeze.json` retains that original record. The import was corrected and two additional colour/potential controls were frozen at `ca17322` before behavioral execution. No expectations changed, and those ten test hashes remain unchanged after the behavioral baseline.

Baseline [37185646509](https://github.com/CrispStrobe/kerotakis/actions/runs/37185646509) confirmed nine failures out of ten new contracts. The titration loop ignored its supplied safety screen entirely, including endpoint-refinement trials. Rejected first doses reached the solver and changed the flask; later vetoes were ignored; fractional trials were not screened; warnings were absent. The already-reached pH endpoint control passed. Existing library, redox and transaction suites passed.

## Repair

Each dose prepares its own raw physical proposal from the same pre-increment flask snapshot, including titrant, carrier water and the existing adiabatic temperature balance. It calls `assess_pour(before, proposal)` before solver settlement. The default trait delegation to `assess` remains supported. An allowed or warned proposal can be solved; a veto returns its diagnostic without running that proposal's solver or committing its state.

An ordinary full-dose veto stops the loop and preserves earlier accepted doses. A veto during fractional endpoint refinement also stops the loop, discarding the current provisional full dose and every trial belonging to that increment. The initial full trial may already have been solved to bracket the endpoint; this is recorded by the contract's one allowed solver call. No claim is made that all solver calls preceding a later veto can be undone. The physical flask changes only after an accepted increment is chosen.

Warnings stay with individual proposals and only the selected proposal's events are committed. A warning from an overshooting full dose does not describe a smaller accepted endpoint dose. Colour and potential endpoint vetoes no longer claim the configured step limit was exhausted. A titration already at its pH endpoint performs no pour, screen call or solver call. The journal records the accepted history and final refusal in one entry; a later refusal can coexist with a `Titrated` summary of prior accepted increments.

## Validation

Repair source `0126790b` passed focused [37185907627](https://github.com/CrispStrobe/kerotakis/actions/runs/37185907627): **679 tests**, including all ten new frozen contracts, fifteen existing redox titration tests, thirty-eight solver transaction tests and the core library suite. The full regression manifest now includes both new targets and the existing redox target, with their source hashes and freeze files bound to the receipt. Full [37186113101](https://github.com/CrispStrobe/kerotakis/actions/runs/37186113101) accepted source `0126790b7329aa374fa592d69aa116e368bc106c`: **1,512 Rust tests, all 17 stages, WASM and frozen CLI controls passed**. Hosted artifact proof and third-fifty replay [37186846990](https://github.com/CrispStrobe/kerotakis/actions/runs/37186846990) passed **347 integrity checks** and **980 replay checks**, retaining the nine existing replay qualifications. The earlier safety audit's explicitly excluded pure-water extraction forecast remains qualified; there are no behavioral contract exclusions in this titration round.

Accepted binary SHA-256: `e3429690edcb9bf4163d1cdf0fc0f592f9958f6c8bfe7a47c183b79ab762e487`. Receipts are in `accepted/`; raw focused and replay artifacts are archived with the corresponding archived CI evidence. The complete validated binary artifact remains on GitHub. Workflow defaults name this successful full run and its exact source.

## Resources and scope
Validation and experiment execution used resource-aware scheduling. Machine-specific resource snapshots, storage locations and preservation inventories are retained privately; public scientific evidence remains indexed by the run links and receipts in this report.

This repair screens the raw dose before equilibration. It does not add post-equilibration safety policy, alter stock-draw or numeric precision semantics, or establish an all-or-nothing transaction for an entire titration. The selected contract is per-increment atomic refusal with prior accepted progress retained. Titration dose precision/stock accounting and the other early-stop diagnostic reasons were subsequently addressed by the [quantity and accounting follow-up](../titration-accounting-contracts-20261004/RESULTS.md). Spill-recovery probe/quantity consistency and still-cut donor debit accuracy near an ULP remain separate gaps for further contracts.

The final evidence commit was pushed from the storage checkout and then synchronized into the main workspace once sufficient disk headroom returned.

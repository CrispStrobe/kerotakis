# Stock ownership integration

This slice depends on PR #756 and ports accepted `stock.rs` ownership plus
the minimal Bench serialization/refusal adapters. It carries conservative
precision controls, opt-in compensated controls and finite-precision boundary
controls. Original test bytes are retained in adjacent `.rs.txt` snapshots;
compiled copies have only rustfmt changes. [Hashes and mapping](stock-port.json).

It changes default stock draws from main's permissive absolute slack to the
already-audited conservative relative debit certificate. Invalid draws and
inaccurate or swallowed debits refuse; opt-in compensated draws retain a
bounded low component. Refusals are atomic inside the ledger. This does not
prove every existing Bench operation rolls back after later solver failure;
transaction integration remains AUD-00/TXN-01 work.

[The ownership decision](../../docs/chemistry-amount-ownership-decision-20261007.md)
states the public API and save boundary. Hosted integration validation is
pending; original branch evidence does not establish this port's acceptance.

## Main-based refusal integration follow-up

Source review at `cdcf183f11394b8365a39ddf375b6a455dfa3ea9` found that
`Add` and `AddMaterial` returned stock refusal events with the default
reequilibration disposition. `step_with` consequently cleared solution state
and called downstream solvers even though no stock was withdrawn. The bounded
follow-up marks both refusal paths unchanged, matching extraction.

Four new contracts in
`crates/kerotakis-core/tests/stock_refusal_short_circuit.rs` use a solver that
counts calls and mutates temperature/inventory if reached. They require zero
solver calls and identical vessel/stock snapshots for precision/exhaustion
refusals in both modes, and invalid species requests; the positive control
requires successful draws to retain normal solver execution. Attempted
operations still enter the log. These are new integration controls; the 25
frozen original contracts and their receipt remain unchanged. No hosted failing
baseline was executed for this follow-up; the defect was established by source
control-flow review. Local execution remains deferred to hosted gates.

[Follow-up source and contract hashes](stock-refusal-followup.json) bind this
reviewed integration change separately from the immutable original port receipt.

## Frozen pre-fix baseline dispatch

The unchanged follow-up test bytes were separately placed on pre-fix stock
source `cdcf183f` in baseline harness commit `8b6c9bda`. Production source is
unchanged on that branch. Hosted
[run 37729403884](https://github.com/CrispStrobe/kerotakis/actions/runs/37729403884)
was dispatched once and remains queued without an executed result at this
2026-10-08 checkpoint. [Dispatch provenance](stock-baseline-dispatch.json)
binds source/test/workflow hashes separately from original receipts. A behavior
failure is expected, but has not been observed; no acceptance or failing-baseline
claim follows from dispatch alone. Record completed output in a new receipt
when available. The standalone baseline branch is not intended to merge.

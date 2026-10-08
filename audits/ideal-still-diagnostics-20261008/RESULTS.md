# Prepared ideal-liquid still diagnostic integration

This follow-up starts from binary diagnostic source
`73d231b3210b8c512c24268cad961d3b4a5be19e`. Forecast commit `a5377136`
precedes production edits. [Frozen expectations](freeze.json) bind the new
kernel and operator contracts; the original batch source, including all prior
unit contracts, remains in `original-batch.rs.txt`. Original unit test bodies
and inherited integration contracts remain unchanged.

## Scope and compatibility

`batch::ideal_still_checked` returns existing `vle::StillError` categories.
The public `ideal_still` Option API delegates with `.ok()`. The same arithmetic,
checks and branch order remain, with refused proposals now classified as
invalid input, request/composition/condensate/residue/energy representation,
phase evaluation, integration limit or incomplete cut. No new tolerances or
parameter fits were introduced. Root bisection remains 64 iterations, amount
mesh 1024, component step bound one quarter, relative completion `1e-12` and
production integration limit 100000. The zero-limit policy test injects zero
only through a private evaluator seam.

The already-checked pure-component helper becomes crate-visible so the batch
kernel can retain its original categories directly. The original private
Option/counter helpers remain available under `cfg(test)`; existing counter and
boundary tests retain their source bytes. The StillError description now covers
both binary and ideal-liquid cuts, with existing variants/codes unchanged.

The core additional-solvent planner uses the checked API and ten literal
`ideal-still-*` Phrase keys, including a future non-exhaustive fallback.
German and French cover every key. Precision and integration failures no longer
assert that a valid request lies outside the constant-latent domain. The
existing operator refusal path already skips downstream solvers; new controls
require full serialized bench preservation except the attempted-operation log.
Event, Operator and save schemas remain unchanged.

## Expectations and validation state

Three kernel contracts cover finite/nonnegative/model/stage/pressure validation,
independently specified precision versus phase refusals, and analytic accepted
pure/equal-volatility cuts alongside Option compatibility. One private contract
requires an exhausted integration budget to refuse rather than return a partial
success. Two operator contracts cover five independently declared additional-
solvent refusal fixtures, zero calls to a mutating solver and unchanged state,
plus an ordinary methanol cut with transfer and normal solver processing.

The new checked API did not exist at forecast time, so executing those new
kernel contracts before implementation would be an API compilation failure,
not behavioral evidence. The separately dispatched operator-only
[baseline 37732719083](https://github.com/CrispStrobe/kerotakis/actions/runs/37732719083)
excludes those future kernel API tests and can establish behavior independently;
its outcome remains pending at this preparation checkpoint.

[Prepared source bindings](prepared-repair.json) identify the changed files.
Rustfmt, whitespace, TOML parsing, ten-key translation completeness, frozen
forecast hashes and original batch unit-test bytes pass static checks. No local
Cargo or application run occurred. Repaired hosted acceptance and delivery are
pending; source inspection does not replace execution.

## Remaining scope

Successful-cut scalar withdrawal, residual cutoffs, ownership and transaction
certificates remain separate work. The planner retains its existing portion
selection policy; negative condensed portions can still be skipped before the
kernel receives its finite/nonnegative inventory. Malformed owner validation
needs an explicit adapter-level contract rather than a claim that kernel guards
certify the original vessel. This diagnostic slice does not alter that selection
policy or introduce broader inventory reconstruction.

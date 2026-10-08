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
certificates remain separate work. Binary-only route selection and general Vessel/Henry-partition validation are
outside this additional-solvent guard. No broader inventory reconstruction is
introduced.

## Supplementary individual-owner guard

Forecast commit `450dc107` independently freezes malformed individual condensed
owners before adapter changes, in a separate test file and
[portion-validation receipt](portion-validation-freeze.json). Original forecast
files and receipts remain unchanged. Negative, NaN and both infinities are
covered in liquid and aqueous entries: malformed additional solvent beside
water, malformed water beside positive methanol, and duplicate positive/negative
methanol whose aggregate would be positive. Invalid owners must not disappear
through filtering or cancel during coalescing. Full-state comparisons also
preserve exact amount bits, because JSON alone maps nonfinite numbers to null.

The adapter now checks malformed condensed amounts before coalescing only when
an additional-solvent route is confirmed. A malformed nonzero additional-solvent
entry can itself confirm that route; zero inactive entries cannot. Such inputs
retain the typed `ideal-still-invalid-input` refusal and skip downstream solvers.
Water/ethanol-only inputs retain the existing binary selection and keys; zero
inactive coordinates and ordinary positive methanol cuts remain supported.
These expectations are prepared controls, not executed acceptance.

Separate unchanged-production baseline branch
`audit/ideal-still-portions-baseline-20261008`, harness `1989584b`, is prepared
locally from forecast source before the guard. Its production source equals
`a1685e9d`; its targeted hosted-only workflow would run the supplementary core
contracts, preserving logs, source/test hashes, exit status and generated lock.
No push or dispatch has occurred for that baseline at this checkpoint.
[Supplementary source bindings](portion-validation-prepared-repair.json) identify
the guard and exact tests separately from the original receipts.

## Refreshed prerequisite

Refreshed #759 at `ada512df` includes thermo prerequisite `1f662be8` and
its separately disclosed spirit-still golden adaptation. The inherited delta
contains the eight observed cut amounts, their preserved original snapshot,
source-bound adaptation receipt and independent cut/component/enrichment checks;
no kernel or original forecast changes were merged. See
[the prerequisite disclosure](../integration-20261007/THERMO-GOLDEN.md).
This dependency refresh does not supply executed acceptance of the ideal
diagnostic or individual-owner changes. Original and supplementary forecast
hashes remain unchanged.

## Public fixture access adaptation

Binary baseline `37731589674` exposed that an external integration fixture
cannot call private `Bench::vessel_mut`. Both ideal operator files had inherited
that fixture pattern. [The adaptation](fixture-access-adaptation.json) preserves
their exact original forecast bytes and replaces only donor setup access with
the existing public vessel vector, followed by rustfmt. No production API or
assertion changes are introduced. Original freezes continue to bind the preserved
original snapshots; adapted execution bytes have their own hashes.
Queued ideal baselines `37732719083` and `37734101871` were cancelled before
execution. Neither supplies behavior evidence. Corrected baseline delivery and
acceptance remain pending.

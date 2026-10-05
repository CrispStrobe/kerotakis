# Typed mixed-crystal conservation and raw readback, 2026-10-04

The strict molecular certificate now admits the existing closed aragonite-strontianite owner using explicit neutral CaCO3/SrCO3 formula units. Each inventory entry carries composition from the owner that admitted its amount. Registry-backed material and typed crystals share an identity only when their formulas agree. Unregistered primary SrCO3 and unrelated opaque material remain explicit refusals; there is no generic formula-string fallback.

General transaction validation now rejects blank or duplicate phase labels, missing or repeated model components, negative/nonfinite component amounts and finite component values whose aggregate moles or mass overflow. Refusal preserves the original vessel. Typed zero seeds remain valid.

PHREEQC selected crystal component values are checked directly before balance correction. Negative/nonfinite values refuse and name their selected column; valid positive and zero values survive unchanged. The previous maximum-with-zero projection could conceal negative, NaN and negative-infinity values.

The 24 strict/shape controls were frozen in commit `857ff091`; the seven raw-readback and two aggregate-overflow controls were frozen in `f09b0841`. Original test hashes and forecasts are unchanged. These are source-informed boundary controls, not new blind CLI experiments.

## Baseline evidence

Hosted core baseline [37239938929](https://github.com/CrispStrobe/kerotakis/actions/runs/37239938929) at `1667e4cd6e035718a5fda5ff3823822ae9a0fc07` ran 106 tests: 93 passed and 13 failed. Seven failures exposed blanket refusal of valid typed crystal ownership; six exposed acceptance of malformed shape or ambiguous labels. The prior negative/nonfinite amount controls and existing solver/conservation controls passed.

Raw baseline [37240279145](https://github.com/CrispStrobe/kerotakis/actions/runs/37240279145) stopped at a missing test import and provides no behavioral evidence. The import was corrected in the test-only parent hook without modifying frozen tests. Actual behavioral baseline [37240626139](https://github.com/CrispStrobe/kerotakis/actions/runs/37240626139) at `c3a21d592a590790b21e629e6356d90fbb32fe55` passed three controls and failed four: negative, NaN and negative infinity were concealed, and positive infinity lacked a column-specific refusal. Positive, zero and missing-value controls passed.

Expanded core baseline [37240863428](https://github.com/CrispStrobe/kerotakis/actions/runs/37240863428) at the same `c3a21d592a590790b21e629e6356d90fbb32fe55` ran 108 tests: 93 passed and 15 failed. It reproduced the 13 original failures and confirmed both aggregate-overflow failures.

The full selection retains four exact source-preserving legacy exclusions already qualified in the [preceding report](../post-equilibrium-contracts-20261004/RESULTS.md): pure-water extraction without extractable solute, the zero-transfer screen-call observer, and two extreme-ratio single-phase Drain forecasts. Existing supported replacement controls and all original forecasts remain preserved. No new exclusions were added for this repair.

## Accepted full-source verification

Hosted run [37241135174](https://github.com/CrispStrobe/kerotakis/actions/runs/37241135174) passed at repair source `8cdab1f6ecd0b1132c4008014a5a0c5476429d17`: 1,865 Rust test executions, all 17 native/WASM/CLI stages and 198 source bindings. This includes every new frozen control, both raw-readback feature modes, the existing typed crystal state tests, and live precipitation, acid dissolution and settled repeated-equilibrium controls. The executable SHA-256 is `67797d56ced1531f44dce7252c272b34d19183a4d4571a076b9dcf67d3a634cb`.

An independent artifact check passed all 432 bindings, including source content at the tested commit, stage logs, resolved dependency lock, executable and seven CLI control-family captures. The compact receipts are stored in [accepted](accepted); the complete hosted artifact is retained with the corresponding CI run. No local build or application run was needed.

The third-fifty replay [37242349555](https://github.com/CrispStrobe/kerotakis/actions/runs/37242349555) passed 1,065 checks against that exact executable, retaining all nine original qualifications. Its full-artifact integrity check independently passed 432 bindings. The increase over the earlier replay count comes from eight additional source bindings, not eight additional blind experiments.

The fourth-fifty replay [37242514682](https://github.com/CrispStrobe/kerotakis/actions/runs/37242514682) passed all 1,148 checks and 124 text/JSON captures: 50 original cases plus 12 separately frozen followups. Original classifications remain 21 bounded agreements, seven qualified agreements, 13 expected invalid-input refusals and nine author protocol errors. Followups retain two author errors, three bounded agreements and seven qualified agreements. Original scripts, expectations and qualifications remain unchanged.

This phase adds conservation coverage and guards; it makes no new performance claim. The earlier representative timing result remains bound to its earlier source and is not a measurement of this revision.

## Scientific limits

This certificate establishes represented elemental/formal-charge bookkeeping, not thermodynamic equilibrium, derived aqueous speciation, kinetics, reaction heat or nuclear closure. Strict conservation can certify material redistribution across multiple distinct typed crystal owners; the native adapter still supports only one mixed phase per vessel in its existing dilute inorganic wateq4f domain.

Native analytical Ca/Sr/C balance correction still computes nonnegative remainders and rescales aqueous totals without a bounded raw residual policy. Corrected conservation is not independent evidence against phase overdraw or raw solver error. External gas boundaries skip carbon correction. See [LIMITS.md](../solid-solution-raw-guards-20261004/LIMITS.md).

Unsupported surface/exchanger owners, unresolved/object mass, soap aggregates and nuclear ownership remain outside the strict molecular certificate. Authoritative compensated vessel/still migration remains open; see [MIGRATION.md](../compensated-amount-contracts-20261004/MIGRATION.md).

All builds and app executions in this phase run on hosted CI. Large artifacts are retained separately from this source checkout; local checks are limited to source, hashes and metadata. A second agent performed a read-only source review without finding a concrete regression. Existing unmerged/dirty secondary worktrees remain preserved.

The next native correction change is specified in [NEXT.md](../solid-solution-raw-guards-20261004/NEXT.md), including future forecasts to freeze independently, raw phase budgets and same-solve reporting witnesses.

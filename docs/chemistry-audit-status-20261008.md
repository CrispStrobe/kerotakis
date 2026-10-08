# Chemistry audit checkpoint — 2026-10-08

This checkpoint updates [the accepted audit report](chemistry-audit-status-20261005.md). The full accepted engine remains [8cdab1f6](https://github.com/CrispStrobe/kerotakis/commit/8cdab1f6ecd0b1132c4008014a5a0c5476429d17). New integration ports and native repairs below are pending acceptance. Queued checks are not successful validation.

## Delivered and pending

| Change | Public review | State at this checkpoint |
| --- | --- | --- |
| Compensated Amount primitive and 15 preserved contracts | [PR #756](https://github.com/CrispStrobe/kerotakis/pull/756) | All 59 checks passed; merged as `b31455cc` |
| Compensated stock owner, localized refusals and solver short-circuit | [PR #757](https://github.com/CrispStrobe/kerotakis/pull/757) | Head `a490f37c`; required checks pending |
| Accepted thermo kernels and 58 integration controls | [PR #758](https://github.com/CrispStrobe/kerotakis/pull/758) | Head `45c44a4a`; required checks pending |
| Binary still refusal categories and unchanged-state handling, including empty donors | [PR #759](https://github.com/CrispStrobe/kerotakis/pull/759) | Head `73d231b3`; stacked on #758; baseline and required checks pending |
| Bounded native mixed-crystal allocation and raw readback | [PR #760](https://github.com/CrispStrobe/kerotakis/pull/760) | Head `016df1dc`; main-based baseline and required checks pending |

Stock and Amount do not migrate authoritative vessel inventory. Thermo kernels do not close scalar donor/receiver ownership. The native port includes only the checked Ca/Sr/C allocation/readback changes and finite typed-owner validity checks; unrelated trace-retention, heat and native scaling work remains outside it.

## Executed native evidence and fixture correction

The original expanded pre-repair baseline compiled 33 tests: 13 passed and 20 failed. A first-repair refinement baseline compiled 42: 29 passed and 13 failed. The external-phase baseline compiled 50: 41 passed and nine failed; four new failures involved negative or NaN external-phase amounts. Infinity loop cases were not reached after the failing NaN assertion.

[Native/live run 37730149327](https://github.com/CrispStrobe/kerotakis/actions/runs/37730149327) at `9140ee58` compiled successfully: 108 library tests passed, five failed, and all three live crystal precipitation/dissolution/repeat controls passed. The overall run failed. [Source-bound results and immutable receipts](https://github.com/CrispStrobe/kerotakis/blob/014ac1be/audits/native-budget-contracts-20261007/RESULTS.md) preserve outcomes and hashes.

Five older failures expose a fixture ambiguity: default open headspace introduced an atmospheric CO2 boundary that fixtures claiming closed carbon had not cleared. Original files and observed outcomes are preserved. Separate fixture copies clear only the inherited external boundary, retaining every assertion; explicit open-carbon controls add their own boundary. The original seven raw guards also have a separately hashed complete-column adaptation adding mandatory zero `C(-4)`, with assertion bodies unchanged. These adaptations must remain disclosed. The 50 native boundary functions are source-informed controls, not 50 new blind CLI experiments.

## Immediate executable lanes

1. **Accept or repair the corrected native donor.** Inspect [corrected pre-repair baseline 37731003330](https://github.com/CrispStrobe/kerotakis/actions/runs/37731003330) and [corrected native/live repair 37731158308](https://github.com/CrispStrobe/kerotakis/actions/runs/37731158308). Classify compile, harness and behavior failures separately. Preserve each raw log and source/test hash. Fix genuine allocation/readback failures without fitting tolerances to output. Require all supported boundary and live controls to pass before accepting this bounded repair.
2. **Validate the independent main native port.** Inspect [main-based baseline 37731431537](https://github.com/CrispStrobe/kerotakis/actions/runs/37731431537), then #760's exact-head required gates. Its contract-only commit precedes production edits. Review the source map in `audits/integration-native-budget-20261008/production-port.json` on the PR branch. Donor acceptance cannot substitute for main-based acceptance. Do not merge any baseline-only workflow branch.
3. **Finish stock and thermo integration.** Inspect #757 and #758, repair concrete failures, and merge only matching validated heads. Stock's corrected refusal baseline is [37731328644](https://github.com/CrispStrobe/kerotakis/actions/runs/37731328644). Its earlier baseline failed before executing tests because it hashed an absent generated lockfile; it supplies no behavioral evidence. Corrected baseline workflows bind tracked manifests first and preserve the generated lock after Cargo, including failed tests.
4. **Finish binary still diagnostics.** Inspect [baseline 37731589674](https://github.com/CrispStrobe/kerotakis/actions/runs/37731589674) and #759's checks. Require distinct refusal categories, complete state preservation, logged attempts, zero downstream mutating-solver calls on refusal and the normal successful-cut control. Merge #758 first, retarget #759 to main, refresh its gates, then merge when validated. Preserve original forecasts and supplementary empty-donor freeze. The additional-solvent ideal-still API still collapses numerical failures to a generic domain refusal; give that API a separate checked diagnostic contract rather than widening this binary claim.
5. **Validate the integrated source.** After coherent ports merge, run the full exact-source audit and preserved third/fourth replays against the resulting executable. Record new source and binary bindings, retain the four legacy exclusions and original qualifications, and update this checkpoint. Only then establish a release-profile baseline for PERF-01.

These lanes advance AUD-00/01/02 and UX-01/TXN-01; they do not complete the [22-lane roadmap](chemistry-audit-next-lanes.md). NUM-01 still needs the complete vessel writer/reader/API/save inventory; NUM-02/03/04 need authoritative mechanical, chemical and still ownership. Native allocation does not certify H/O/charge, speciation, heat accuracy or general solid-solution chemistry. Private machine policy controls resource use and agent concurrency.

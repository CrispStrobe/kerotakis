# Prepared historical transaction and Amount test port

Branch [`audit/historical-transaction-amount-controls-20261008`](https://github.com/CrispStrobe/kerotakis/tree/audit/historical-transaction-amount-controls-20261008), head `b0cc75a13d634915e91b009b8f834541bd16ecd0`, prepares 53 historical functions from accepted source `8cdab1f6`: 38 complete solver transaction/boundary/schema functions and 15 standalone Amount arithmetic/persistence functions. It depends on #771's repair at `26cd8fc7`. There are no production changes relative to that prerequisite. That initial preparation is preserved in the manifest. [PR #773](https://github.com/CrispStrobe/kerotakis/pull/773) is now open at `fb50bebc7e0c27a111ea2af5bab50ac436f43990`, including actual merged main `18dc3987`. That head had no production changes. Its compile failure exposed the missing transient heat-input API; the later repair below includes production changes.

[The port manifest](port.json) binds original sources and the formatting adaptation. `solver_transactions.rs` is copied byte-for-byte. The original Amount file and original freeze are preserved in the prepared branch; the target is normalized by rustfmt for current formatting gates. Its complete bytes equal rustfmt output from the archived original. Expectations are unchanged. The historical freeze hash binds the archived original, not the formatted target; any later full-audit integrity review must preserve and verify that distinction.

The prerequisite #771 is accepted and actual main ancestry is integrated. The initial plan remains archived; consume current checks using the separately bound current-head plan below. Compile all 53 functions and inherited workspace tests. Preserve any first failure before editing; distinguish missing APIs from behavioral failures and fix real regressions without weakening historical expectations. Require all five final-head gates. Bind the formatted target's actual hash in native acceptance plans and preserve the original source/freeze hashes separately.

The transaction target extends complete snapshot, atomic route, boundary, derived-state, geometry and signed electrical-state coverage. The Amount target covers bounded two-component arithmetic, underflow/quantization and persistence. It does not migrate authoritative vessel ownership or prove arbitrary precision. These are restored historical contracts, not new independent chemistry forecasts or full historical audit acceptance.

## Preserved failures and current repair

[The initial API failure](first-failure.json) identifies nine compiler diagnostics for absent `HeatInput`/`Vessel::heat_input` on `fb50bebc`. The per-pass heat-context restoration at `85c79f51` enabled execution. [The runtime baseline](runtime-first-failure.json) records 31 transaction passes/seven failures, all 15 Amount controls passing, and all 604 core library tests passing, with actual checkout/tree identity verified. Keep these two failure classes separate.

The first atomic repair at `9d0b32ed` passed all 204 planned functions, including all 38 transactions. [The preserved broader failure](repair-9d-compatibility-failure.json) records five workspace failures: German translation and unchanged still/stock refusal behavior. Native gate failure remains decisive despite focused passes.

Current [PR #773](https://github.com/CrispStrobe/kerotakis/pull/773) head is `b83be1e96e430232e76a85eb0e5eb28293c30903`, including actual main `0af91885`. It retains per-pass heat context, atomic numeric rollback and ordered deposit metadata tracking. An explicit apply disposition now separates unchanged refusal from committing candidates: restore state and stock, retain refusal events and log the attempt, without replacing that refusal by validation of an unchanged malformed starting state. Committing paths retain numeric validation before ordinary solver execution and final publication. The default `step` API rejects nonfinite typed Add arguments as errors; configurable `step_with` preserves structured stock-refusal narration and skips solver execution. The new numeric-domain error has German text.

[The exact-head 234-function plan](../integration-20261008/native-controls/pr773-b83be1e9-plan.json) adds every function from all five affected compatibility targets to the original 204, across twenty-two unchanged fixtures. [The current Mac native review](../integration-20261008/native-controls/pr773-b83-mac.json) now passes all234 functions unfiltered and all workspace/lint/Codex/Curiosity steps on the verified source tree. Other required gates remain pending; formatting, TOML/placeholder and diff checks pass. The initial test-only manifest remains immutable and describes its earlier snapshot.

`Vessel::new` initializes the transient context to `None`; external complete struct literals require the field, while serialization omits it. `BenchError::InvalidState` affects exhaustive external matches. Rollback covers bench-owned physical state, waste, stock, broken vessels and journal; solver-private state is outside scope. Cloning overhead remains unmeasured. Titration has a separate trial path; TXN-01 defines its pending admission controls. No native reaction thermochemistry or authoritative compensated owner migration is claimed.

## Corrected paired transaction diagnostic

The first [paired dispatch](paired-dispatch.json) failed before execution because of the workflow's manifest path. [Its failure receipt](paired-harness-first-failure.json), raw log and partial artifact remain preserved; no baseline/repair outcome is inferred.

[The separately frozen current manifest](paired-run-b83be1e9.json) binds baseline `85c79f51`, repair `b83be1e9`, the unchanged 38-function fixture and 604 inherited library tests. [Corrected dispatch 37897838141](paired-dispatch-b83be1e9.json) verifies harness `2e3ab99a407c6401ea9ba64fd1899b5d01352051`; [offline review](paired-review-b83be1e9.json) and [run/source/submodule bindings](paired-bindings-b83be1e9.json) now pass. The baseline reproduces exactly31 passes/seven preserved failures; repair passes all38; both sources pass all604 library tests under one generated lock. New mode `transaction-state-repaired` fixes the manifest/sparse-checkout path while preserving every earlier job and manifest. Preparation checks verify all referenced manifest files exist. This is a historical diagnostic, not new independent chemistry forecasts.

Archive `transaction-state-repaired-RUN_ID` and run the audit harness branch's `tools/delta-contract-review.py --artifact ARTIFACT --lane transactions --manifest audits/historical-transaction-amount-controls-20261008/paired-run-b83be1e9.json --harness DISPATCHED_SHA --report NEW_REPORT`. Require source/tree, clean checkout, unchanged fixture, complete unique function names, zero ignored/filtered results, matching lock bytes and raw log hashes. Verify the baseline failed names reproduce the preserved seven; repair must pass all 38 and both sources all 604. Compile/harness failures cannot substitute for runtime rejections. Final native234/workspace acceptance additionally verifies diagnostic compatibility. Any source-head change needs new bindings and fresh gates.

## Separately prepared current-source CLI replay

[The current source manifest](cli-source-b83be1e9.json) pins repaired source/tree `b83be1e9`. Harness [`2e3ab99a`](https://github.com/CrispStrobe/kerotakis/commit/2e3ab99a407c6401ea9ba64fd1899b5d01352051) prepares mode `refusal-transaction-cli-replay`; [CLI run37900400346](cli-dispatch-b83be1e9.json) is now dispatched after those prerequisites passed, and is building the exact executable. Runtime replay outcomes remain pending. Earlier 85/9d source manifests/jobs remain unchanged and must not be dispatched as accepted candidates.

Dispatch identity is verified against harness `2e3ab99a`; do not dispatch a duplicate while this run is active. Require actual `headSha` to match; archive `refusal-transaction-cli-replay-RUN_ID`, independently reparse every process and explicitly verify binary, lock, source/tree, two gitlinks, toolchain/build records and six dispatched harness hashes. Expected classifications remain fifth 46 passes/three expected refusals/one qualification, sixth 49 passes/one qualification, plus eight strict-stage checks. Require native234, full workspace and all five final-head gates before merge; record merged-tree equality separately.

Preserve these source snapshots for later repairs and create separately named source manifests/jobs and plans. Never overwrite executable receipts or qualifications. The branch-only audit harness workflow must not replace the production fleet workflow.

## Offline executable binding review

After downloading the CLI artifact, independently re-evaluate every recorded process with the audit harness's `tools/combined-cli-replay.py --review`, selecting the current source manifest and a new report. Then run [the reusable binding reviewer](../../tools/cli-binding-review.py):

```sh
python3 tools/cli-binding-review.py \
  --artifact ARTIFACT --run-metadata RUN_METADATA \
  --harness 2e3ab99a407c6401ea9ba64fd1899b5d01352051 \
  --source-manifest audits/historical-transaction-amount-controls-20261008/cli-source-b83be1e9.json \
  --results-review NEW_PROCESS_REVIEW --report NEW_BINDING_REVIEW
```

`RUN_METADATA` must include `databaseId,url,headSha,event,status,conclusion,jobs` from the completed dispatch. The tool stream-hashes the actual binary and lock, verifies exact source/tree and clean build records, all source gitlinks, six dispatched harness hashes and five uploaded harness inputs, successful run identity, and equality with the independently recomputed process report. The workflow is checked against its dispatched Git blob; its uploaded hash record is retained. Reports are exclusively created outside preserved artifacts. Binding review requires the separate successful process re-evaluation and does not replace final PR gates.

[Reviewer verification](cli-binding-reviewer-controls.json) reproduces the original accepted source26 replay bindings and rejects four altered archives. It runs no application and establishes no new chemistry or performance result.

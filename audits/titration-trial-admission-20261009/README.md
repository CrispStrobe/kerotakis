# Frozen titration trial admission contracts

[The freeze](freeze.json) binds ten source-informed functions and twenty declared subcases, authored against accepted candidate `b83be1e9` before any titration repair. [The prepared fixture](titration_trial_admission_contracts.rs) is formatted and parsed by rustfmt; it has not been compiled or executed. No runtime failure count or supported chemistry outcome is claimed.

The next tranche addresses `Bench::titrate_loop` bypassing the ordinary apply path. Inspection shows its safety argument is unused, its upfront numeric check examines only the dose product being nonpositive, and trial candidates reach solver hooks before shared state validation. These are source observations; the baseline execution must establish actual outcomes before implementation.

The fixture freezes nonfinite concentration/step/pH target, individual nonpositive factors including two negatives, overflowing dose product, finite dose with overflowing carrier inventory, malformed initial inventory, a valid single mechanical dose, a zero-increment no-op and prospective safety veto. Invalid candidates must refuse before applicability, solve or safety hooks, preserving the complete bench, stock and journal. A veto keeps only the veto and one logged attempt. Positive dosing checks the independently declared concentration-times-volume amount; the inert observer predicts no pH or chemistry.

After #773's complete acceptance/integration, copy the frozen file byte-identically to `crates/kerotakis-core/tests/titration_trial_admission_contracts.rs`. Record actual baseline source/tree, fixture SHA, generated lock, toolchain, raw logs and unfiltered named results in a hosted run. Rustfmt success is not compilation success; prerequisite API/compiler problems must be classified separately. Preserve first failing logs before production edits. A failed function may stop its parameter loop, so do not claim all twenty subcases ran on a failing baseline.

Repair only demonstrated causes. Validate each raw input separately before products, shared complete candidate state before hooks, and safety disposition before solver invocation. Keep accepted #773 diagnostics and every original test unchanged. Use independently supported positive dose/no-op controls; do not impose an invented pH range or arbitrary dose floor. Require all ten frozen functions and inherited234/workspace/gates on the repaired exact source.

These controls cover admission and the first trial. They do not settle rollback of previously accepted increments, solver failure or bisection/final-safety behavior. Freeze that next failure-injection tranche before extending the implementation; decide the operation-versus-increment rollback boundary explicitly. [TXN-01](../../docs/chemistry-audit-next-lanes.md) retains the broader route matrix.

## Prepared hosted baseline

[The prepared harness manifest](prepared-harness.json) binds harness `d08ba3185355d19dd084d5a2a7eb79401ce5d5d6`, mode `titration-admission-baseline`. All earlier jobs remain unchanged. YAML, Bash and both embedded Python programs parse; frozen fixture/source-tree bindings are checked. [Actual dispatch37913920525](dispatch.json) is now pending after #773 merged-tree equality was verified. The prepared manifest remains an immutable pre-dispatch snapshot; no baseline outcomes are claimed yet.

After #773 completes acceptance/integration, verify main's tree against the reviewed candidate and the selected harness ref against its expected commit. Then dispatch from the audit harness branch:

```sh
gh workflow run chemistry-audit.yml --repo CrispStrobe/kerotakis \
  --ref audit/fifth-independent-50-20261008 \
  -f campaign=titration-admission-baseline
```

The job checks out exact source `b83be1e9` and records its clean tracked tree before injecting the byte-identical frozen fixture into an absent untracked test target. This baseline measures the tracked model plus a separately bound added test; it does not claim that the fixture belonged to the original source tree. The job verifies no tracked code changed, records status after injection, generates one lock and runs all ten fixture functions plus all604 inherited library functions with that lock. It archives `titration-admission-baseline-RUN_ID`, including raw logs, exit codes, fixture, lock, source/tree, submodules, toolchain and harness hashes even on failure.

Verify actual dispatch `headSha`, every frozen named outcome, unique/unfiltered summary counts, fixture and lock hashes, the exact two source gitlinks, empty tracked diff and only the expected untracked test target. Inspect function failures as model outcomes; job success means the baseline was collected completely, not that its functions passed. A compile/harness failure does not establish a behavioral count. Preserve first failure evidence before production edits. Port the frozen fixture into the repair branch without changing its bytes; bind each later repaired source and run the inherited234/workspace/gates separately.

The audit harness groups concurrency by ref and campaign with cancellation disabled. The production fleet groups by PR/ref and cancels superseded runs. Waiting for #773 here is an acceptance dependency, not a claim that this new campaign would cancel its jobs.

## Offline baseline review

[The reviewer](../../tools/titration-baseline-review.py) checks the complete unique/unfiltered ten-function results, all604 inherited names and summaries, exit codes and archived result agreement. It also verifies the frozen fixture, clean tracked source before injection, only the declared untracked target afterward, source/tree, two gitlinks, generated lock, toolchain records, three harness hashes and actual successful dispatch. It accepts collection of failed model functions. Compilation failure or missing outcomes cannot pass this review.

```sh
python3 tools/titration-baseline-review.py \
  --artifact ARTIFACT_DIRECTORY --run-metadata RUN_METADATA_JSON \
  --harness d08ba3185355d19dd084d5a2a7eb79401ce5d5d6 \
  --report NEW_REPORT_JSON
```

[Eight synthetic controls](reviewer-controls.json) verify collection acceptance with a model failure and rejection of dirty source, altered fixture, filtered results, duplicate inherited names, exit mismatch, uninitialized submodules and altered lock. These are reviewer checks, not execution of the titration fixture.

## Verified baseline and active repair

[The reviewed baseline37913920525](baseline-review.json) records one pass (valid zero-increment no-op) and nine failed functions, with all604 inherited library functions passing. Source/tree, frozen bytes, clean tracked code plus the separately injected fixture, one lock, submodules, harness and actual run identity are verified. Failed parameter loops may stop early; the twenty declared subcases are not twenty observed baseline executions. Raw logs and hashes are preserved before repair.

[PR #774](https://github.com/CrispStrobe/kerotakis/pull/774), [bound repair0346701f](repair-0346701f.json), validates individual finite inputs and positive concentration/volume before products, finite dose/carrier amounts and initial state. It validates and safety-screens prospective full and refinement trials before solver hooks, and validates solved candidates before acceptance. Only accepted trial warning events enter the stream. Invalid state or veto uses the existing whole-operation rollback; solver-error narration retains its existing behavior. This does not freeze or prove later increment/solver-error/final-safety semantics.

The ten-function target is byte-identical to the freeze. [Native244 plan](../integration-20261008/native-controls/pr774-0346701f-plan.json) preserves all prior234 fixture hashes and adds ten functions. Workspace/native/five gates/fleet are queued; no repaired runtime success is claimed. [The source-specific CLI manifest](cli-source-0346701f.json) pins this new tree and unchanged100-case/164-process forecasts. Harness `88dfa166` adds mode `titration-admission-cli-replay` with every older job unchanged; it is prepared but undispatched. Dispatch after native acceptance, then independently review process outcomes and executable/lock/source/submodule/harness bindings. Any repair-head change needs a fresh plan, manifest and run.

The reviewer also supports `--freeze` and `--job-name` for separately frozen tranches; its defaults retain this admission run. A fresh offline recheck of this existing archive still reports one pass/nine failures and604 inherited passes. [The generalized reviewer controls](../titration-trial-publication-20261009/reviewer-controls.json) include a synthetic seven-function archive. This changes review tooling only; original baseline receipts, manifests and control hashes remain historical snapshots.

Current acceptance checkpoint — 13:44 UTC: [Mac244](../integration-20261008/native-controls/pr774-034-mac.json) and [Linux244](../integration-20261008/native-controls/pr774-034-linux.json) independently verify every planned function, workspace/lint/Codex/Curiosity and actual checkout-tree equality. [All five gates and complete CI/fleet](../integration-20261008/pr774-gates-awaiting-cli.json) pass on034. [CLI37938775232](cli-dispatch-0346701f.json) is dispatched on immutable harness88dfa after this native acceptance; its outcomes remain pending. The prepared repair manifest is a historical pre-dispatch snapshot, not the current dispatch state. No merge before independent process and executable binding acceptance.

Merged checkpoint — 14:00 UTC: [#774 main-tree equality](main-tree-equality.json) records merge659cd45d after all exact-head acceptance. [CLI process review](cli-results-review-0346701f.json) and [bindings](cli-bindings-review-0346701f.json) accept164 processes/eight strict checks with original qualifications. The separate [publication baseline](../titration-trial-publication-20261009/dispatch.json) is now dispatched. Earlier preparation/pending paragraphs above describe their dated checkpoints.

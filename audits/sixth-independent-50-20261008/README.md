# Another fifty: evidence accumulation

Fifty new expectations were frozen at commit `f88cbdcd75d5d113418ec418d14e1cfba57bad9a` before consulting further experiment outputs. The [machine forecast](forecast.json) and [readable expectations](FORECASTS.md) define 72 CLI processes. Forecast SHA256: `45e554283e8dd27e9483d45fe9cd33dd66a2d4fa208a8e47b7d3b2fb99ea3a0d`.

[Hosted run 37762502850](https://github.com/CrispStrobe/kerotakis/actions/runs/37762502850) was dispatched at harness `d7e04b44e2ee9b2be13f6ef5b7d693722fe8d9dc`. Execution is queued; no result or 50/50 claim is made. This isolated campaign reuses the original fifth-run executable, SHA256 `fcf424a65384c5ef5949960f228fda4e1c3e893ec93c65d58fe5b12064352967`, built from production source `7bc43e089753483710507133ccc8377591bb3490`. It predates subsequent main repairs. Its [build receipt](../fifth-independent-50-20261008/build-receipt.json) records the source, dependencies and toolchain. No new compilation is needed for this initial evidence pass.

The cases were independently designed from general reasoning. Earlier findings were already known, so this is not a blinded experiment. Syntax discovery used CLI help/grammar, inspection field declarations and registry identity keys. Existing experiment files and prior forecasts were not used to design the cases. Expectations cover ownership, transfer composition, thermal equivalence, weak-acid/base buffers, precipitation, magnetic separation and gas boundaries.

Each script must exit successfully, produce valid JSON, finish with inspection, and meet every frozen check. A model or safety notice remains a qualification. An author error remains an author error until a separately frozen adaptation is executed. Scientific bounds must not be fitted to observed output to obtain a perfect score.

## Evidence review lane

Download artifact `sixth-fifty-37762502850` after the run completes. Preserve its original bytes. Verify the recorded harness commit and file hashes against the dispatched commit. Verify the source, executable and generated dependency lock against the preserved fifth build receipt.

Run `tools/sixth-independent-50/review.py` with `--forecast audits/sixth-independent-50-20261008/forecast.json`, `--out-dir` pointing to the downloaded `sixth-experiments` directory, and `--report` pointing to a new file outside the artifact. The reviewer never executes the application. It checks all script/output hashes, reparses raw observations, recomputes every expectation, and checks individual and aggregate results. Its exit status is nonzero unless all 50 are ordinary passes; qualifications remain visible in the report.

Publish reviewed counts and source-bound receipts in this directory. Classify each unmet expectation as an author error, incorrect scientific premise, declared model limitation, or reproduced engine defect. Preserve the original forecast and output for all categories. Freeze any corrected-input replay separately before execution.

## Repair lane

For each reproduced engine defect, freeze a failing control and a positive neighbor before changing production. Implement a bounded repair, execute the controls on baseline and repair under the same dependency resolution, and preserve both outcomes. Run the relevant required integration checks on the final implementation head. Re-run the original forecast against the repaired source using a newly bound executable; never attribute old-binary passes to current main.

## Previous fifty completion lane

The original fifth-run counts remain [42 ordinary passes, three expected refusals, one model qualification and four author errors](../fifth-independent-50-20261008/RESULTS.md). The [separately frozen syntax replay](../fifth-independent-50-20261008/syntax-replay.json) corrects identifiers in four cases while preserving every original chemical expectation. Review [run 37760938060](https://github.com/CrispStrobe/kerotakis/actions/runs/37760938060) independently when available. A 50/50 numerical/declared-contract result must still disclose model qualifications and expected refusals; it does not establish unrestricted chemical coverage.

The original excessive-stage distillation check accepted any visible refusal. Add a separately frozen check that binds the refusal to the distillation operator and invalid-input category, with a valid neighboring stage count. Execute it against a newly bound current-main CLI. Existing native coverage of the category does not substitute for that CLI evidence.

## Integration and performance lanes

The latest integration state and final-head checks are tracked in [the integration status](../integration-20261008/STATUS.md) on the systematic audit branch. Pending repairs remain PRs [763](https://github.com/CrispStrobe/kerotakis/pull/763), [764](https://github.com/CrispStrobe/kerotakis/pull/764), and [765](https://github.com/CrispStrobe/kerotakis/pull/765). Acceptance requires their current-head checks; earlier diagnostic passes do not establish combined-source acceptance.

The original fifth-run profiler remains a separate lane. Review its archived samples with `tools/fifth-independent-50/review_profile.py` before accepting timings. Choose an optimization only after a successful workload demonstrates a measured hot path. Whole-process timings include startup and serialization; seven samples do not establish stable tail latency. The campaign-only workflow must not replace main's regular chemistry audit workflow in a merge.

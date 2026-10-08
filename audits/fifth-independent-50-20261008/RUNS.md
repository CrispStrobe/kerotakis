# Fifth batch execution checkpoint

The [original forecast](forecast.json) contains **50 independently authored experiments and 84 CLI scripts**. It was committed as `e343fb73` before any new script ran. Its SHA-256 remains `4041d15bc70b108437bf6df909cbbf892f79d737b6ef6faae7f99bbde21a15f5`. Authoring scope and exposure limits are disclosed in [FORECASTS.md](FORECASTS.md); this is not a claim of experimental blinding.

[Hosted run 37751230970](https://github.com/CrispStrobe/kerotakis/actions/runs/37751230970) is queued at harness `2289ac9f`. Production source is pinned independently to main commit `7bc43e089753483710507133ccc8377591bb3490`. The build preserves its generated lock, toolchain, submodule revisions and executable hash. Behavior and profiling jobs consume that same executable and proceed independently after the build.

The first dispatch, [37750775718](https://github.com/CrispStrobe/kerotakis/actions/runs/37750775718), was cancelled while queued, with zero executed job steps. Static review identified a missing workflow directory in the evidence jobs' sparse checkout. The correction also requires a final inspection for supported scripts and preserves release debug information for profiling. No script, expectation or numerical bound changed. Neither dispatch supplies an executed result at this checkpoint.

Checker controls passed locally without invoking the app: missing observations and zero denominators fail; a timeout cannot count as an expected CLI refusal; a model notice cannot count as an ordinary scientific pass; supported scripts require their final inspection. Python syntax, workflow YAML, sparse path and original forecast hash checks passed.

## What the next agent should do

1. Inspect the hosted build, then download all three artifacts: `fifth-main-build-37751230970`, `fifth-fifty-37751230970` and `fifth-profile-37751230970`. Retain failed logs and original forecasts. A harness/build failure is not chemistry evidence.
2. Read `results.json` alongside every failed case's raw stdout/stderr, input and check observations. Distinguish genuine engine failures, explicit model boundaries, author/syntax errors and missing observability. Do not fit bounds to results or silently substitute scripts. Any correction gets an independently recorded adaptation and replay.
3. Read `performance.json` and Callgrind summaries. Native measurements cover whole CLI processes, including startup and JSON; instrumented profiles identify instruction costs. Seven samples do not establish stable tail latency. There is no speedup claim yet.
4. Choose one demonstrated correctness gap or measured expensive path, freeze its repair/optimization contract, make a bounded change, and replay unchanged controls. Compare performance on the same runner where practical and verify output agreement.

The workflow on this campaign branch replaces the registered dispatch file only for this source-bound run. **Do not merge it over main's fleet workflow.** Promote reusable harness work separately after review. Existing integration PRs remain independent of this campaign.

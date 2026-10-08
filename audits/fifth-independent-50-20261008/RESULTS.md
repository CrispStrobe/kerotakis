# Fifth independently forecast fifty — original results

[Run 37751230970](https://github.com/CrispStrobe/kerotakis/actions/runs/37751230970) executed all 84 original CLI processes for 50 cases. Production source is `7bc43e089753483710507133ccc8377591bb3490`, harness `2289ac9f`, executable SHA-256 `fcf424a65384c5ef5949960f228fda4e1c3e893ec93c65d58fe5b12064352967`. The [forecast](forecast.json) remains unchanged at SHA-256 `4041d15bc70b108437bf6df909cbbf892f79d737b6ef6faae7f99bbde21a15f5`.

| Reviewed outcome | Cases |
| --- | ---: |
| Ordinary agreement with frozen checks | 42 |
| Visible refusals satisfying the original checks | 3 |
| Agreement with an explicit model qualification | 1 |
| Author identifier/syntax errors; chemistry not evaluated | 4 |

[The source-bound receipt](experiments-receipt.json) preserves original checker counts and the separate review classification, with hashes for every original input, stdout, stderr and execution envelope. All 84 archived runs match their frozen scripts; every recorded check was independently recomputed from raw output. The original four errors were labeled `missing_final_inspection_protocol_failure` because the checker tested inspection before nonzero exit; raw stderr identifies their actual cause. No original result was overwritten.

## What failed and what remains uncertain

**F25–F28 aborted because of author syntax.** `add v1 acetic acid ...` tokenizes the species as `acetic`, which the CLI rejects and helpfully suggests `CH3COOH`. The first invalid line prevents the weak-acid/buffer forecast from being evaluated. The later `sodium acetate` token would have the same issue. Separate [syntax adaptation](syntax-adaptation.json) uses canonical `CH3COOH` and `NaOAc`, preserves every dose/check/bound and reuses the original executable. [Replay 37760938060](https://github.com/CrispStrobe/kerotakis/actions/runs/37760938060) is queued at this checkpoint. These four original author errors are not evidence of incorrect chemistry.

**F34 passes its pH relation with a gas-rate qualification.** The emitted notice states that finite CO2 uptake is instantaneous equilibrium; mass-transfer rates and wait-dependent degassing are not modeled. Retain that qualification. This does not establish gas kinetics accuracy.

**F48–F50 satisfy the original refusal checks, with distinct limits.** F48 preserves the dry-salt donor state and emits a visible refusal. F49 exits with an explicit invalid-fraction error; it does not establish full rollback because no final inspection is reached. F50 correctly parses `stages: 129`, preserves water ownership and refuses, but reports `not-modeled.no-bubble-point` instead of a stage-count category. The original forecast did not check the category, so its pass remains preserved. This exposes a weakness in the oracle and an old-source diagnostic gap. Merged binary PR #759 separately covers stages 129 as `invalid-input` in `still_refusal_diagnostics.rs`; a CLI replay on that newer source remains separate from this frozen-source run.

No numerical chemistry failure is demonstrated by the completed valid scripts under their declared checks. Four chemistry forecasts remain unanswered until the disclosed replay completes; visible refusals and model qualifications are not unrestricted scientific acceptance. Bounds were not fitted to outputs.

## Next steps

1. Consume the syntax replay, verify its executable/parent/replay hashes and unchanged checks, and classify actual outcomes. Preserve original failures alongside adapted runs.
2. Consume the queued profiling job. Recheck every archived native sample with `tools/fifth-independent-50/review_profile.py`; the dispatched profiler's job status alone cannot validate sample behavior. No performance result or speedup is claimed yet.
3. Strengthen future refusal controls with independently declared categories and positive neighboring requests. Replay the excessive-stage case on a source containing #759. Do not retrofit the original forecast or change bounds to obtain a pass.
4. After remaining implementation gates and merges, validate one exact integrated source with the full audit and preserved replays. This campaign predates later main safety/binary repairs and cannot certify them.

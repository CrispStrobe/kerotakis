# Withdrawal trace baseline and bounded repair acceptance

[Hosted run 37756438042](https://github.com/CrispStrobe/kerotakis/actions/runs/37756438042) compiled and executed the same ten source-informed test functions on both sources:

| Source | Passed | Failed | Receipt |
| --- | ---: | ---: | --- |
| Test-only baseline `28ca045541c797745b088a9686ce3f37baf828ce` | 1 | 9 | [baseline.json](baseline.json) |
| Bounded repair `b6306db986d4bc2a683f7b01b03c314a033a686f` | 10 | 0 | [repair.json](repair.json) |

The baseline reproduces positive owned matter disappearing during zero/absent requests, ordinary debits, excluded-phase withdrawals, partial tiny debits and exhaustion of another owner. Its tiny-dose exhaustion compatibility control passes. Both sources use adapted fixture SHA-256 `e5085dfdf8efab27f5c5af0576e47cb2b31e5f4cd9c1f9f6801ce2afb218ebb4` and identical generated dependency locks. Source/tree, 215 production/manifest bindings per source, original/adapted fixtures, toolchain, execution envelopes and full-log hashes were verified against the downloaded artifacts. Compilation succeeded on both sources; these are assertion outcomes, not compilation failures.

The repair changes the two `Vessel` withdrawal helpers to retain positive portions until actual exhaustion. All ten functions pass, including each loop's full trace-size/mode coverage. Baseline failing loops stop at their first assertion; their later rows were not executed. The overall diagnostic workflow may be red because its baseline is deliberately pre-repair. Inspect the independent repair result.

The [original source-informed forecast](https://github.com/CrispStrobe/kerotakis/blob/6c65032f/audits/withdrawal-trace-20261008/FORECASTS.md) precedes production changes. Static review adapted four full-vessel comparisons through existing serde APIs because `Vessel` has no `PartialEq`; [original bytes and adaptation hashes](https://github.com/CrispStrobe/kerotakis/blob/28ca0455/audits/withdrawal-trace-20261008/fixture-comparison-adaptation.json) remain preserved. No inputs, expected states or bounds changed.

[PR #765](https://github.com/CrispStrobe/kerotakis/pull/765) still requires fresh gates at head `3d2c6b3a744cf3993ddcc036b135371fa8d6e1d2`, which incorporates later main changes. This is bounded mechanical-helper acceptance. Other positive-amount cutoffs, bulk-plus-tiny subtraction, compensated vessel ownership, provenance lots and equilibrium reconstruction remain separate work. These ten source-informed functions are separate from the fifth independently forecast fifty, whose execution jobs remain queued at this checkpoint.

The receipts are published on the audit branch so recording this result preserves the implementation head under validation. Branch-only diagnostic workflows must never merge over main's fleet workflow.

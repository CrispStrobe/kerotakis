# Pure-liquid still optimization

This round follows the validated distillation-contract repairs. It targets a measured-cost opportunity visible in both numerical kernels: an exactly pure cut repeatedly solves an unchanged phase state through the 1024-step mesh and every ideal stage.

## Change and contracts

Both kernels now validate their existing inventory, model and pressure requirements, obtain one initial bubble answer, then calculate exactly pure overhead and latent heat analytically. Fraction cuts use the requested fraction of inventory; energy cuts lift the affordable amount, capped by stock. Start and end boiling temperatures agree and pure cuts do not report an azeotrope. The public output structs and APIs are unchanged. Every positive second component, including a trace, keeps the original mixture integration path.

A shared numerical helper distinguishes genuine full requests from partial arithmetic that rounds to the whole stock. Positive underflow, unrepresentable positive partial residue, zero or nonfinite positive-cut heat refuse. Full-inventory heat overflow does not invalidate an affordable finite partial cut. Energy rounding never publishes a heat amount above the supplied budget. Pure cuts with representable subnormal overhead now work even when the former mesh would underflow; the previous refusal test was explicitly updated to check this capability extension while retaining refusal of an underflowed requested amount.

Private evaluator seams allow deterministic local counters without shared mutable globals: successful pure cuts use one phase answer at stages 1, 4, and 128; positive mixtures retain repeated phase evaluation. Analytic material/heat tests cover water, ethanol and generic pure components, scales, component positions, zero/full/partial/energy inputs, overflow, underflow, inactive invalid properties and pressure-domain refusals.

## Frozen hosted comparison

`predictions.json`, `freeze.json` and scripts were committed in `fbefd55a` before execution. These are seven source-informed correctness controls and four profile cases, not another blind fifty. Correctness controls check pure water/ethanol/methanol at one and 128 stages, full and excess-energy cuts, zero requests, microscopic energy/fraction equivalence and a positive ethanol trace in water.

Profiles compare hash-verified validated debug CLI executables on the same hosted runner. Each profile warms both binaries, then uses two ABBA sequences, yielding four timed samples per variant. The same scripts and independent geometric pure-cut expectations check every invocation. There is no fixed timing pass threshold and no release/browser performance claim. Raw samples and process streams will be retained on the NAS.

## Validation

Focused hosted run [37111684236](https://github.com/CrispStrobe/kerotakis/actions/runs/37111684236) passed 125 thermodynamic test executions, including both deterministic phase-call controls. Its log hash and diagnostic-only receipt were checked; they are saved in `focused/`. Diagnostic receipts alone cannot supply a trusted CLI executable. Full hosted validation [37111890212](https://github.com/CrispStrobe/kerotakis/actions/runs/37111890212) passed all eleven stages, including 1,343 Rust test executions, WASM compilation, the original fifty CLI assessment and 56 checks replaying the previous four distillation controls. Exact source: `fbefd55a9b5fe2eec3139b5c2ba3d57ecb468007`; executable SHA-256: `fcbbee6c8b982c0e5b326919e7b96c0901d9ec8ecba8e9c912e5a9ac9e5fab70`. All source, log and dependency-lock hashes were checked locally without executing the app. Logs and receipt are in `validation/`, and the prior four controls' raw evidence is in `previous-controls/`.

Matched hosted comparison [37112786318](https://github.com/CrispStrobe/kerotakis/actions/runs/37112786318) passed 861 checks over 47 CLI invocations: seven frozen correctness scripts plus four profiles, each with two warmups and eight alternating timed samples. Both executables used the same Rust compiler and resolved dependency-lock hash. Baseline source is `8744d131928dd7416c1d207f22408ba228540570`, SHA-256 `11cd342ab32cccce909ba97f5389609ad2c7f4d516a7c71853def2d19ba41265` from successful validation run 37107431953.

| Profile | Validated baseline median | Optimized median | Median ratio |
|---|---:|---:|---:|
| P01: five successive 1% water cuts, one stage |5.675s|0.715s|7.93×|
| P02: five successive 1% ethanol cuts, one stage |5.776s|0.665s|8.68×|
| P03: five successive 1% methanol cuts, one stage |0.765s|0.665s|1.15×|
| P04: one 1% water cut, four stages |4.673s|0.665s|7.02×|

These are four timed samples per variant on one hosted runner and include process startup, JSON output and waiting overhead. The small methanol difference is not a general throughput guarantee. The deterministic kernel controls separately establish one phase answer for exactly pure cuts. Mixture integration and its existing extreme-composition representation limits are unchanged.

`performance/` preserves every raw correctness-control stream, executable receipts, complete timing samples and stream hashes. All 47 raw process streams and the executables remain in the corresponding archived CI evidence and the separately retained artifact. Third-fifty replay [37112963346](https://github.com/CrispStrobe/kerotakis/actions/runs/37112963346) passed 913 checks with nine separate qualifications. Next-fifty plus thirteen-controls replay [37113212375](https://github.com/CrispStrobe/kerotakis/actions/runs/37113212375) passed 410 checks with its two limitations retained. Both used the exact optimized executable above. Receipts and verification are in `third-suite-replay/` and `prior-suite-replay/`; complete raw outputs remain respectively in the separately retained artifact and the separately retained artifact. Passing repaired contracts still includes explicit refusal and unsupported observations; not every original physical forecast is claimed fulfilled.

## Resources and limits
Validation and experiment execution used resource-aware scheduling. Machine-specific resource snapshots, storage locations and preservation inventories are retained privately; public scientific evidence remains indexed by the run links and receipts in this report.
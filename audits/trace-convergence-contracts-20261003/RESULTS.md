# Trace phase convergence and activity consistency

This source-informed round follows the positive-coordinate guards. It asks whether a retained positive trace is actually converged, and whether reported phase data use activity coefficients at the returned liquid composition. The first eight direct-kernel controls were committed in `f9c5dd55` before baseline execution; they are synthetic numerical experiments, not another blind fifty or empirical property measurements. A ninth classification-cycle control was added with the repair.

## Reproduced defects

The old dew and TP flash loops accepted maximum absolute composition changes below 1e-9 and 1e-10. A trace can change by half its own amount while meeting either cutoff. Hosted baseline [37117806650](https://github.com/CrispStrobe/kerotakis/actions/runs/37117806650) failed seven of the eight new controls while every existing thermodynamic test passed. One existing-correct two-phase activity control passed.

Three independent contracting activity laws demonstrate wrong fixed points in dew, all-vapor and two-phase flash calculations at trace scales 1e-12 and 1e-100. Equal-pressure dew has the analytic trace ratio 1/4; the all-vapor flash has ratio 4^(-2/3); a three-component two-phase control has vapor fraction 1/2 and trace liquid/feed ratio 1/4. Two discontinuous laws demonstrate oscillations that were accepted after an absolute-small update. Two more controls demonstrate premature activity acceptance when a tiny composition update crosses an activity-law boundary. These are arithmetic controls, not claims of real substances with those activity laws.

## Repair

Every coordinate must now meet the existing absolute tolerance and a relative tolerance at its own scale, with no bulk or absolute floor. Exact inactive zeros can settle; the preceding positive-coordinate guards still refuse any lost volatile. Iteration caps remain 80 for dew and 60 for flash. An unresolved trace or phase cycle refuses instead of publishing an intermediate split.

Before publishing a converged dew or vapor/two-phase flash candidate, the activity callback is evaluated at that candidate. Its finite positive coefficients must agree relatively with those used to compute the candidate, within 1e-8. This adds one activity evaluation when a candidate appears settled. An all-liquid classification reached after a prior split rechecks the actual feed composition before returning feed liquid and K-values; a stale liquid guess cannot supply those coefficients.

## Validation

Focused hosted run [37118055071](https://github.com/CrispStrobe/kerotakis/actions/runs/37118055071) passed all 146 thermodynamic test executions, including all nine new controls and the earlier trace, pure, phase inversion and distillation contracts. Its diagnostic receipt and log hash were verified; `baseline/` and `focused/` preserve both runs. A diagnostic receipt alone is not an accepted executable validation.

Full hosted validation [37118167891](https://github.com/CrispStrobe/kerotakis/actions/runs/37118167891) passed all twelve stages: 1,365 Rust test executions, WASM compilation, the original fifty CLI assessment, 56 prior distillation checks and 55 trace-phase CLI checks. Validated source is `fd9eeedbbc1ad9499c5ccd25b5837aafb4254eaf`; executable SHA-256 is `241ffe589539b436c075e9ec020f368306820644156c63abfdee69823a0cb14e`. All source, log, executable and resolved dependency-lock hashes were independently checked locally without running the app. Raw stdout/stderr hashes for both four-case CLI suites were also verified.

`validation/` retains all twelve logs, the exact dependency lock and validation receipt. `distillation/` and `trace-phase/` retain complete control streams and verification records. The executable and original fifty replay artifact remain in the corresponding archived CI evidence. The source-informed numerical tests are direct kernel controls; they are not new CLI capabilities. No direct use of these dew/flash APIs was found in core, CLI or WASM source; those packages were still validated for regressions.

## Resources and limits
Validation and experiment execution used resource-aware scheduling. Machine-specific resource snapshots, storage locations and preservation inventories are retained privately; public scientific evidence remains indexed by the run links and receipts in this report.

The stricter convergence contract can explicitly refuse mixtures that did not settle within the existing caps; it does not guarantee global convergence for arbitrary activity laws. Existing binary scalar relative precision limits, f64 underflow boundaries, missing empirical properties and unsupported registry identities remain documented in earlier reports. WASM compilation does not establish browser runtime behavior.

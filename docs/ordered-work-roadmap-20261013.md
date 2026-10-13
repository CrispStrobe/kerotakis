# Ordered work roadmap — 2026-10-13

The user authorized these six lanes in this exact order. Lane 1 is active;
lanes 2–6 are queued. Planning or a queued job does not complete a lane.
Preserve original forecasts, first failures, accepted source bindings and
resource rejections. Large local work requires the private resource gate.

1. **Release performance — measured; fleet closure pending.**
   [Draft PR785](https://github.com/CrispStrobe/kerotakis/pull/785) defers the optional
   explanation engine until first valid `explain`, retaining primary solver
   initialization and cached optional unavailability. The candidate is
   `d06ffe3a`, based on accepted main `7337bb58`.
   [Controlled paired results](../audits/release-performance-20261013/PAIRED-RESULTS-V3.md)
   pass independent review of 88 process outputs and 64 timed samples, with exact
   equality to the original admitted chemistry outputs. Observed median wall
   reductions: startup 21.0%, one salt 26.6%, batch 10.2%, titration 4.5%. These are
   host-specific; the titration effect is modest relative to variation. All seven
   rebuilt-candidate controls and all five actual required CI logs pass independent
   review. The original safety admission failure and interrupted timing attempt
   remain preserved. No general significance or application-memory gain is claimed.
   [Completed fleet steps](../audits/release-performance-20261013/candidate-fleet-completed-steps-20261013.json)
   now verifies all 33 jobs and source-defined steps pass. The source-bound
   [v2 raw review](../audits/release-performance-20261013/candidate-fleet-v2-saved-output-review.json)
   accepts 288 cases/368 checks, and [partial v1 review](../audits/release-performance-20261013/candidate-fleet-v1-v3-partial-review.json)
   accepts 24 cases/32 checks. Total accepted: 312 cases/400 checks; remaining:
   390 cases/591 checks plus v1/v3 aggregate recomputation.
   [V2 aggregate closure](../audits/release-performance-20261013/candidate-fleet-v2-aggregate-independent-review.json)
   now matches accepted own reports and independently verifies all 33 source job
   and step outcomes. Distillation's
   downloaded bytes are retained; its law evaluation stopped at the load gate.
   Next accept the own saved family/frozen outputs and actual aggregates at the
   exact candidate source, retain the completed 33-job checkpoint, and finish the
   narrow PR's guarded acceptance. Detailed executable review obligations are in
   [the results handoff](../audits/release-performance-20261013/PAIRED-RESULTS-V3.md).
   Downloads, recomputation and execution require open operating resource gates;
   local review stops at closed load/memory gates. A
   [hosted remaining-case review](../audits/release-performance-20261013/hosted-remaining-fleet-review-dispatch.json)
   failed on a reviewer assumption that a tracked lock must occur in an untracked
   snapshot. [The preserved failure and correction](../audits/release-performance-20261013/hosted-remaining-fleet-review-first-failure.json)
   bind the original failure. A further [lock-contract correction](../audits/release-performance-20261013/hosted-remaining-fleet-review-lock-contract-correction.json)
   proves root `Cargo.lock` is ignored and absent from Git. The queued revision
   `d3323f96` failed at that incorrect tracked-lock assertion. The next revision
   reached a [report-transport mismatch](../audits/release-performance-20261013/hosted-remaining-fleet-report-transport-correction.json):
   frozen analyzers use integer evidence keys that their original JSON writers
   stringify. All failed outcomes remain preserved. Run38450225092 is queued at
   `c5c56f9c`, using the original JSON transport with collision rejection and
   persisted recomputed reports; six focused checks pass without changing values,
   verdicts or thresholds. Summary lock digest and accepted own-release bytes
   remain bound. Do not duplicate the active job. Review its raw receipt before
   accepting more
   cases, then execute the prepared aggregate reader and reconcile own v1/v3
   outputs, including all thirteen v1 and five v3 artifact downloads. PR785 remains draft; later lanes remain queued.
2. **Fifty fresh CLI experiments.** Use a fresh agent that learns only syntax,
   without reading existing corpus cases or implementation details. Freeze all
   fifty original scripts and expectations before execution. Preserve first
   outcomes, classify gaps, and repair demonstrated app failures with regression
   controls. These are distinct from performance inputs and annotation matrices.
3. **Annotation contract.** Use the [actual-main owner plan](../audits/yaml-parser-migration-20261012/annotation-owner-contract-plan-7337bb58.md).
   Freeze and run its proposed fifty controls; preserve the original 18/2
   outcomes. Resolve documented boundaries versus intentional support expansion,
   retaining strict modeled fields, unknown-key refusals and duplicate behavior.
4. **Historical regression coverage.** Refresh the presence inventory, port one
   prerequisite-complete behavior group at a time through narrow source-bound
   PRs, preserve original controls and exclusions, then replay the historical
   audit against actual integrated source. Missing paths are not proven defects.
5. **Broader safety.** Scope intermediate-product observation and multiple
   locations with supported solver routes. Freeze positive/benign/priority and
   discarded-trial controls before implementation; qualify unsupported domains.
6. **Accounting and scientific accuracy.** First choose authoritative amount
   ownership/API/save compatibility, inventory and migrate all writers, prove
   solver reconstruction, then close still donor/receiver/energy precision.
   Separately acquire reviewed primary property/calibration data for one defined
   domain, record licenses and uncertainty, and compare actual predictions with
   those independent references. Do not substitute internal consistency for
   empirical accuracy. Detailed prerequisites are in [the lane catalog](chemistry-audit-next-lanes.md).

Complete each preceding deliverable before executing the next lane. Use isolated
implementation branches and private build targets; publish concrete evidence
and scoped limitations at every checkpoint. Never merge the broad audit branch.

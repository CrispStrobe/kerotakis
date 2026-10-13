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
   [Fleet inventory](../audits/release-performance-20261013/candidate-fleet-closure-inventory-20261013.json)
   preserves the earlier snapshot. [Current fleet progress](../audits/release-performance-20261013/candidate-fleet-progress-20261013.json)
   verifies all 33 materialized source jobs: 32 successful, only the 288-case
   aggregate queued. Source-bound 240-case and 120-case aggregate log outcomes
   are recorded; independent own saved family/aggregate closure remains pending.
   Next accept the own saved family/frozen outputs and actual aggregates at the
   exact candidate source, obtain a completed 33-job checkpoint, and finish the
   narrow PR's guarded acceptance. Detailed executable review obligations are in
   [the results handoff](../audits/release-performance-20261013/PAIRED-RESULTS-V3.md).
   Downloads, recomputation and execution require open operating resource gates;
   the latest fleet review stopped before downloads because load/memory gates
   closed. PR785 remains draft; later lanes remain queued.
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

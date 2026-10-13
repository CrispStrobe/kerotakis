# PR785 controlled paired results

The lazy optional explanation engine reduces observed median wall time in all
four admitted fresh-process workloads. Both binaries use the same independently
reviewed controlled build environment. All 88 process outputs match the original
admitted chemistry references; all seven rebuilt-candidate compatibility controls
pass independent raw review.

| Workload | Baseline median / MAD (ms) | Candidate median / MAD (ms) | Median reduction |
| --- | ---: | ---: | ---: |
| Startup | 277.81 / 9.41 | 219.57 / 14.38 | 21.0% |
| One salt addition | 325.89 / 19.28 | 239.35 / 18.42 | 26.6% |
| 64 salt additions | 706.98 / 24.02 | 634.74 / 19.78 | 10.2% |
| Titration | 1417.51 / 37.07 | 1354.00 / 35.29 | 4.5% |

These are host-specific observations from eight timed samples per variant per
workload, in the frozen ABBA/BAAB order. Wall time includes startup, the timing
wrapper and output I/O. The titration difference is modest relative to variation;
no general statistical significance, browser or warm-kernel improvement is
claimed. Wrapper-inclusive waited RSS can inherit parent high-water memory;
raw GNU child RSS remains separate. No pure-application memory improvement is
concluded. Do not compare these absolute times with the earlier hosted baseline.

[Machine-readable results](paired-v3-completed.json),
[independent raw review](paired-v3-independent-review.json),
[accepted build/compiler review](controlled-build-independent-review.json),
[seven compatibility controls](controlled-candidate-controls-v3-independent-review.json)
and [frozen protocol](protocol-v3/README.md) bind the exact source, ELF and lock.
Baseline source is `2bf9c771`, candidate source `d06ffe3a`; rebuilt ELF digests
start `b1867d84` and `1e94804f`, respectively. Native configuration and option
logic were reviewed; exhaustive native per-file compiler argv was not captured.

The first setup attempt omitted a required binding field and ran no workload.
The [subsequent partial attempt](paired-v3-first-resource-rejection.json) stopped
at the real memory gate after 51 timed samples. Both are preserved. The complete
attempt uses a separate evidence destination and pools no partial-run samples.
The original safety admission failure remains untimed and unadmitted.

[All five actual required CI logs](candidate-ordinary-ci-independent-review-v4.json)
now pass independent review; the correction from an optional wasm job to the
required WebAssembly bench job is explicit. The separate own chemistry fleet
still requires saved-output closure. PR785 remains draft. Next review its
remaining source-bound fleet outcomes and update the narrow PR with these
results; merge only after all its outstanding acceptance conditions pass.

The next agent should close the following obligations in order:

1. Refresh metadata for own fleet run38387729183 at `d06ffe3a`, confirming tested
   synthetic checkout `4df5deab` has parents `7337bb58` and `d06ffe3a` and the
   candidate tree. Reconcile all 33 source-expanded jobs from the pinned
   `.github/workflows/chemistry-audit.yml`; the v2 aggregate was absent from the
   earlier 32-job snapshot. Preserve changes in status and first failures.
2. Under fresh resource gates, collect only this run's family/frozen artifacts.
   Bind exact artifact API source association, name, digest/size, safe ZIP members,
   raw streams, manifests, harness/analyzer hashes and ordinary CLI `b290b436`
   with captured lock `5135e2cc`. This fleet uses that binary, rather than the
   different rebuilt candidate used for paired timing. Recompute actual saved
   outcomes without running the CLI. Qualify any reused source-owned evaluator;
   it does not provide an independent scientific oracle. Retain first rejections
   and incomplete reviews. The known total is 702 cases/991 engineering-law checks;
   do not claim that total before its actual own outputs are accepted.
3. Review actual 240/120/288-case aggregate outputs against accepted own family
   reports, including the three previously queued v2 families: thermochemical
   paths, analytical observables and materials transport. Require the final
   completed 33-job/source-bound checkpoint. Do not borrow earlier-source results,
   dispatch duplicates or cancel queued jobs.
4. Attach accepted results to the narrow PR, verify its source/base and current
   required checks, then finish guarded acceptance. Keep the broad audit and
   isolated inspection branches unmerged. Only after lane 1 is complete start
   the fresh syntax-only fifty experiments described in the ordered roadmap.

The bounded fleet inventory is [source-bound here](candidate-fleet-closure-inventory-20261013.json).
The latest two family-review attempts stopped at their initial load/memory gates;
no new family archive was downloaded. Previously accepted performance evidence
and original forecasts remain intact.

Latest progress: [all 33 source-defined jobs now materialized](candidate-fleet-progress-20261013.json),
with 32 successful jobs and the 288-case aggregate queued. The 240-case/338-check
and 120-case/150-check aggregate logs are source-bound and successful. Their
own saved family outputs still require recomputation before full acceptance.
The two formerly queued inspection jobs also completed successfully:
[original seven-control saved-output calculation](hosted-control-raw-review-completed.json)
and [controlled compiler-witness inspection](hosted-controlled-inspection-completed.json).
These are separate from the accepted rebuilt-candidate timing evidence.

The prepared v2 family reader now persists each resource snapshot before its
assertions, including a rejecting snapshot, and rejects JSON NaN/infinity and
numeric overflow recursively. The first method and initial rejection records
remain intact. Strict JSON rejection checks and an actual gate-journal probe
pass; no fleet archive or case has been accepted by this preparation. Large
saved-output work still waits for the operating load/memory gates to reopen.

Current closure checkpoint: [all 33 own fleet jobs and source-defined steps pass](candidate-fleet-completed-steps-20261013.json).
[V2 saved-output review](candidate-fleet-v2-saved-output-review.json) accepts all
12 families/288 cases/368 checks. [Partial v1 review](candidate-fleet-v1-v3-partial-review.json)
accepts acid/base buffers, another 24 cases/32 checks. Total 312 cases/400 checks
are accepted; 390 cases/591 checks and aggregate recomputation remain.
Distillation's own archive and raw-input checks completed, but its law evaluation
stopped at the resource gate and is unaccepted. Reuse protected saved bytes
through a reviewed resume revision; do not rerun the CLI or overwrite accepted
reports. The prepared resume changes orchestration only, requires independent
review, and preserves the original forecasts and source-owned evaluators.

[V2 aggregate closure](candidate-fleet-v2-aggregate-independent-review.json) now
passes independent review: 12 families, 288 cases, 80 relations and 368 checks
match the accepted own reports and actual hosted aggregate output. All twelve
actual download IDs, sizes and digests match the accepted family artifacts.
The same reviewer independently verifies all 33 source-defined job and step
outcomes. No family analyzer or CLI was rerun for this aggregate review.
V1/v3 aggregate closure and the remaining 390 cases/591 checks are pending.

[A hosted saved-output reviewer is dispatched](hosted-remaining-fleet-review-dispatch.json)
as [run38440523017](https://github.com/CrispStrobe/kerotakis/actions/runs/38440523017)
at isolated audit revision `93b4d3c1`. It binds the exact accepted acid/base
receipt and source without downloading or reanalyzing that family, then checks
the remaining 390 saved cases/591 checks using the unchanged original forecasts
and source-owned evaluators. Its own resource checks precede source checkout,
API reads, extraction and evaluation; raw partial results and failures are retained.
It executes no CLI or build and does not certify science or merging.

The [reader source](https://github.com/CrispStrobe/kerotakis/blob/93b4d3c1fdb24530dd4f76740d5e908ecb6a7637/audits/release-performance-20261013/review-remaining-fleets.py)
is root-reviewed and freeze-bound; independent launcher source audit is not
claimed. Wait for the exact-source job result, inspect actual raw/summary receipts,
and accept the newly reviewed families before closing the two remaining
aggregates. Do not duplicate the dispatch or merge its inspection branch.
Local cached distillation bytes and all earlier results remain protected; the
hosted worker fetches its own artifact copy to perform this review.

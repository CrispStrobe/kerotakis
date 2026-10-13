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

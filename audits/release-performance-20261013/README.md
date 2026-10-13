# Ordered roadmap lane 1: release performance

The user authorized six lanes in order: performance, fifty fresh CLI experiments,
annotation contracts, historical regression coverage, broader safety, then deeper
accounting and scientific calibration. Later lanes remain queued until the
preceding lane's deliverable is complete; do not substitute planning for completion.

Current phase: baseline collection is frozen and dispatched as
[run38368899782](https://github.com/CrispStrobe/kerotakis/actions/runs/38368899782).
The [first hosted run38358454350](https://github.com/CrispStrobe/kerotakis/actions/runs/38358454350)
failed artifact preparation because its completed source run returned an empty
PR association; no pilot or timing executed. [Setup revision 3](workflow-revision-v3.json)
checks the exact merged PR head/base/merge in addition to all original checkout,
job, ZIP, binary and lock guards. [Association-repair dispatch](dispatch-v3.json) preserves the
original failed run and unchanged measurement definitions.
The [association-repair run38363500803](https://github.com/CrispStrobe/kerotakis/actions/runs/38363500803)
passed that check but failed raw build-log retrieval when the hosted GitHub CLI
rejected terminal escape sequences. Its benchmark step was skipped.
[Setup revision 4](workflow-revision-v4.json) detects the documented
[raw-response option](https://cli.github.com/manual/gh_api), uses it for file-bound
logs and ZIP bytes, and records the CLI version. [Current dispatch](dispatch-v4.json)
pins the new run. All earlier failures and frozen definitions remain preserved.
Four admitted workloads are now reviewed; the safety pilot remains unadmitted.
[Saved-output review](baseline-output-review-v4.json) and
[independent review](baseline-independent-review-v4.json) preserve all five outcomes.
The complete-five flag remains false. Eight timed samples per admitted workload
match their pilot outputs exactly.

| Workload | Median wall (ms) | MAD (ms) |
| --- | ---: | ---: |
| Startup | 86.28 | 1.02 |
| One salt addition | 95.18 | 0.84 |
| 64 salt additions | 260.75 | 1.10 |
| Titration | 531.69 | 2.81 |

The safety forecast required both gas and reducer warnings; the actual screen
returns its first priority finding. Both hazards were present, but only the gas
warning was emitted. Preserve this failed admission forecast and its untimed
status; it is not evidence that the aggregate exposure API omitted the reducer.

[Startup attribution](attribution-startup.json) identifies eager construction of
the optional explanation engine as about30% of startup instruction work. A narrow
candidate is lazy initialization on first `explain`, retaining an independent
engine and one-time unavailable state. [Draft PR785](https://github.com/CrispStrobe/kerotakis/pull/785) prepares this one
optimization on actual main. [Candidate plan](candidate-plan.json) pins its source,
own CI runs, seven prospective correctness controls and paired acceptance policy.
Two new native controls cover invalid-target noninitialization and cached
unavailability with preserved primary PHREEQC computation. Formatting and source
review pass; own tests and paired numbers are pending. It remains unmerged.
Optional-engine failure diagnostics intentionally move to first valid explanation.
No candidate speedup is accepted.
The instrumented titration exceeded its60-second diagnostic budget; preserve that
incomplete profile separately from its successful ordinary release benchmark.
[Manifest](manifest.json), [inputs](scripts/), [profiler](profile.py) and
[freeze](freeze.json) specify five representative performance workloads.
The own reviewed release artifact was built from `2bf9c771`; accepted main
`7337bb58` has the same tracked tree. No new baseline build is inferred.

Hosted execution uses the existing chemistry audit manual selector
`release-performance`, calling a reusable performance workflow. All regular
fleet jobs are excluded for this selector. It records source, binary, captured
artifact lock, workflow revision and raw commands before and after collection.
Local large starts remain subject to the private operating resource gate.

Each script gets one admission pilot before timing, two fresh-process warmups
and eight timed repetitions. Refused routes and exhausted titration endpoints
are not successful workload measurements. Full canonical JSON must match the
pilot, preserving array order and numeric values; raw stream hashes remain.
Blocking wait and a process-group watchdog avoid polling lag. GNU wall/CPU/RSS
and high-resolution parent wall/waited-child resource measurements have distinct
scopes. Fresh-process warmups do not measure a warm in-process solver alone.

Next: independently review actual baseline outputs and variance, attribute cost
with call/allocation profiling or a source-bound API benchmark, then select one
measured hot path. Compare exact-source baseline/candidate release artifacts in
paired alternating runs with unchanged output contracts. Report variance and
regressions; no browser, warm-kernel or end-to-end speedup is inferred from an
isolated microbenchmark. Use a narrow branch and private build target for any
candidate, followed by relevant correctness checks and its own required gates.

The 50 fresh experiments in lane 2 must be authored before execution by an agent
that learns only syntax, without reading existing cases or implementation details.
They are separate from these source-informed performance workloads and from the
annotation owner's proposed fifty controls.


[Dispatch binding](dispatch.json) pins audit revision and source. The original
workflow dispatch was rejected before any run because runner paths were used
in an unavailable context. [Workflow revision](workflow-revision-v2.json)
fixes runner setup while preserving every frozen input and measurement rule.
[Ordered roadmap](../../docs/ordered-work-roadmap-20261013.md) keeps subsequent
lanes queued until the performance deliverable is complete.

[Saved-output reviewer](review.py) independently recomputes pilot admission,
canonical output comparisons and all reported statistics from downloaded raw
evidence without executing the CLI. Run it with `--evidence` pointing to the
extracted evidence directory and `--out` to a new review JSON file. It checks
recorded source guards and run identity; actual runner API/job/upload association
and ZIP/binary byte verification remain separate acceptance requirements. It
reports incomplete or refused workloads without converting them to successes.
Use `--dispatch audits/release-performance-20261013/dispatch-v4.json` for the
corrected run. The reviewer is a later evidence consumer; it does not change the frozen inputs,
profiler, workflow or baseline run. The four admitted baseline reviews above are accepted with their recorded
source/archive/clock qualifications; safety remains unadmitted.

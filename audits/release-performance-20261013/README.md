# Ordered roadmap lane 1: release performance

The user authorized six lanes in order: performance, fifty fresh CLI experiments,
annotation contracts, historical regression coverage, broader safety, then deeper
accounting and scientific calibration. Later lanes remain queued until the
preceding lane's deliverable is complete; do not substitute planning for completion.

Current phase: baseline collection is frozen and dispatched as
[run38358454350](https://github.com/CrispStrobe/kerotakis/actions/runs/38358454350).
No measurements are accepted yet.
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
The reviewer is a later evidence consumer; it does not change the frozen inputs,
profiler, workflow or baseline run. No actual baseline review is accepted yet.

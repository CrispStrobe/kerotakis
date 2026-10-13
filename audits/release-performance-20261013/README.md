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
review pass. [Seven baseline acceptance controls](candidate-controls-baseline.json)
pass and retain reference streams, including first/repeated explanation in both
languages and modes, empty/invalid vessels and a masked quest transcript.
Candidate outcomes, own tests and paired numbers are pending. It remains unmerged.
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

[Paired protocol](PAIRED.md) and [paired freeze](paired-freeze.json) prepare the
resource-gated candidate-control and alternating measurement consumers. Their
source/binary/lock, actual receipt bytes and frozen consumer/reference checks
passed static review and small guard tests. They have not executed the candidate
or collected paired measurements; own CI and build-environment witnesses remain
prerequisites. Missing compiler evidence requires a controlled rebuild and new
paired policy, rather than an inferred source-caused improvement.

[Candidate artifact preflight](candidate-artifact-preflight.json) confirms the
own PR785 build completed and its artifact is available with the expected
synthetic checkout parents and tree. Full CI is pending. Artifact bytes, actual
candidate lock and build-environment witnesses still require review before
candidate controls or paired execution; availability is not artifact acceptance.

Current candidate is `d06ffe3a`: [source revision](candidate-revision-v2.json) and
[version2 plan](candidate-plan-v2.json) preserve the first three own CI failures
and their test-only locale justification repair. Production `main.rs` and all
control scripts/expectations are unchanged. New own CI and artifact review are
pending. The earlier artifact preflight and paired freeze bind the first head;
they do not certify this revision. Rebind a new consumer protocol/freeze and
references to the corrected head before candidate execution; preserve version1
and reuse the seven saved baseline streams without rerunning their admissions.

[Version2 comparison namespace](protocol-v2/README.md) now freezes the corrected
head, versioned entry points and explicitly reused original baseline references.
Use its consumers for `d06ffe3a`; version1 stays historical. New CI/artifact
acceptance and actual candidate execution remain pending.

[Corrected-head macOS native review](candidate-macos-ci-review-v2.json) verifies
the actual job log and synthetic checkout parents/tree for `d06ffe3a`. Format,
Clippy, workspace tests, codex lint and curiosity coverage completed successfully;
both new lazy-initialization regressions and the repaired locale guard passed.
This accepts the macOS job only. Linux native, preflight and paired wasm are
queued, browser CI is running, and the current-head audited CLI build is queued
in the captured snapshot. Full CI, candidate controls and speedup remain pending.

[Version2 wrapper probe](protocol-v2-wrapper-review.json) checks both actual
entry points select the version2 definitions and wrapper identity, bind the
corrected candidate and verify every frozen shared/versioned file. The dispatch
body was replaced by an observer: no CLI or timing ran. This root check does not
replace independent protocol or artifact review. Frozen definitions are unchanged.

[Current-head release artifact review](candidate-artifact-independent-review-v2.json)
independently accepts the actual `d06ffe3a` build's ZIP, binary, lock, pinned
modules and source/build/upload association. The binary has not executed here.
[Independent version2 review](protocol-v2-independent-review.json) also accepts
its frozen definition namespace, retaining the stated external prerequisites.

The actual ordinary build log lacks Cargo/native compiler versions and explicit
compiler flag witnesses, so equal build environments cannot be established from
these artifacts. [Controlled build collector](controlled-build.py),
[workflow](../../.github/workflows/release-performance-paired-build.yml),
[build freeze](controlled-build-freeze.json) and
[independent static review](controlled-build-independent-static-review.json)
prepare both pinned sources under one observed toolchain with the same actual
dependency lock, fresh separate targets, explicit sanitized flags, verbose build
logs and source/module guards. This collects builds only. Actual compiler
invocations, artifacts and equal environments still need review; it does not
run the candidate or accept performance. Freeze a new comparison version binding
the rebuilt bytes before candidate execution, preserving version1/version2 and
all original control inputs, streams and failed forecasts.

The first standalone controlled-build dispatch was rejected with HTTP 404 because
the new workflow is absent from the default branch; no build or CLI began.
[Version2 routing freeze](controlled-build-freeze-v2.json) preserves the original
collector and workflow bytes, adds a reusable workflow, and selects
`release-performance-controlled-builds` through the registered chemistry audit
workflow. The ordinary fleet builder is excluded for this selection. The
collector's original workflow hash identifies its preserved specification; a
separate routing witness records the actual caller/reusable workflow hashes.
This routing correction does not change source, dependency lock or build policy.

[Independent routing review](controlled-build-routing-independent-review.json)
found and resolved eight generic fleet predicates that initially admitted the
new selection, including three `always()` aggregate jobs. Every job predicate
now evaluates to excluded except the dedicated controlled build. This was
corrected before dispatch; no unintended fleet jobs executed.

[Controlled-build dispatch](controlled-build-dispatch.json) binds the actual
[queued hosted run](https://github.com/CrispStrobe/kerotakis/actions/runs/38396270148)
to audit revision `d49d9659` and the version2 build freeze. A second rejected
dispatch supplied a commit SHA where GitHub requires a branch/tag ref; it created
no run. The accepted dispatch used the audit branch after verifying its remote
head, and the resulting run API confirms the exact revision. No build result,
candidate execution or paired measurement is accepted yet.

[Current-head wasm/browser CI review](candidate-wasm-browser-ci-review-v2.json)
checks the actual logs and tested checkout. The wasm bridge's ten checks pass,
including desktop/browser pH agreement within `1e-6` and ionic-strength agreement
within `1e-9`. Browser headless, semantic DOM, PWA, locale and UX steps pass;
svelte-check reports zero errors and two warnings, which remain recorded. These
are compatibility checks; they do not establish a CLI speedup or full CI closure.

[Saved paired-output reviewer](review-paired.py) and
[preparation receipt](paired-reviewer-preparation.json) prepare independent
recomputation of admission, unchanged canonical outputs, eight timed samples per
variant in the frozen order, raw-clock consistency and median/MAD statistics.
It uses the independent saved-baseline reviewer rather than the collection
program. Small metadata fixtures reject altered raw streams, clocks and command
arguments; no CLI or actual paired-data review ran. Independent source review
and a complete evidence fixture are still pending.

For actual saved evidence, supply `--evidence`, the actual `--protocol` namespace,
`--bindings`, accepted `--artifact-review`, `--build-review`, `--controls-review`,
and `--runtime-map`; write `--out` to an unused file in an existing evidence
directory. The operating-reviewed runtime map supplies `baseline` and `candidate`
objects with recorded `binary` and `source` locations, `scripts` mapping workload
IDs to recorded arguments, and `comparison_root` for exact GNU-time output
arguments. Raw evidence remains private. The reviewer hashes the actual map and
prerequisite receipt bytes. Recorded source guards are checked for equality and
identity; module/source bytes, physical filesystem placement and compiler
invocations require external review. A successful review does not accept a
speedup or authorize merging.

[Current-head Linux native review](candidate-linux-native-ci-review-v2.json)
verifies format, Clippy, workspace tests, codex lint and curiosity coverage in the
actual job log. Both lazy-initialization regressions, the required primary-engine
failure guard and the repaired locale guard pass. Both native platforms, browser
and paired wasm now have source-bound log reviews. Preflight is running in the
latest snapshot; the controlled build is still queued. Full CI and performance
acceptance remain pending.

[Running-build checkpoint](controlled-build-running-checkpoint.json) confirms
the controlled collector started at the exact reviewed audit revision. It is
compiling and preflight is running at the candidate revision. The earlier queued
snapshots are preserved; no artifact, compiler-environment, control or performance
result is accepted from an in-progress job.

[Completed ordinary CI review](candidate-ordinary-ci-completed-v2.json) records
25 successful jobs and the intentional PR publishing skip, with all five required
job logs reviewed at `d06ffe3a`. Preflight reports `preflight clean`. Independent
full gate review and the separate chemistry fleet saved-output closure remain
pending; this does not accept candidate performance or merging.

[Completed controlled-build preflight](controlled-build-completed-preflight.json)
verifies the actual source-bound job and uploaded artifact ID/digest. Both builds
completed under the reviewed collector; the artifact's binary bytes, lock,
effective compiler records, guards and actual routing witness have not yet been
downloaded or accepted. Review those under an open artifact-review resource gate
before binding a further comparison version to the rebuilt bytes.

[Hosted control dispatch](hosted-controls-dispatch.json) binds the first candidate
control collection to [run38407663054](https://github.com/CrispStrobe/kerotakis/actions/runs/38407663054)
and isolated audit revision `f8ca430e`. It uses the already independently reviewed
ordinary release artifact `b290b436` at candidate `d06ffe3a`, preserving all seven
version2 inputs, expectations and saved baseline references. The original audit
branch's workflow bytes and freezes are unchanged; the isolated branch routes
only the correctness collector. No whole source worktree was created locally.

The [hosted launcher](https://github.com/CrispStrobe/kerotakis/blob/f8ca430eeed82f4c9e78fe8db972f4786c130576/audits/release-performance-20261013/hosted-controls.py)
checks the accepted receipt, actual artifact/member bytes, source identity,
filesystem placement and existing resource gates, then invokes the actual frozen
version2 consumer. Its job preserves raw streams and failures. The run is queued;
no control output is accepted yet. Native compiler matching is not required for
these correctness controls. Paired timing still requires review of the controlled
build artifacts and a further comparison version; these controls cannot certify
a rebuilt binary with a different digest.

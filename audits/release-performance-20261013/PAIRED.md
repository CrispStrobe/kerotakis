# Candidate comparison protocol

PR785 remains a draft until its own checks and paired results are accepted.
These consumers perform no builds, downloads, merges or later-lane experiments.
Keep raw evidence and operational locations private.

1. Review the actual baseline and candidate release artifacts, source checkout
   parents/tree, binary bytes, captured locks, build/upload job logs and API
   associations. The baseline identity is the previously reviewed `2bf9c771`
   artifact; candidate source is `55359032`. Produce an accepted artifact receipt.
2. Run [controls.py](controls.py) with the candidate binary/source/lock,
   repository object database, artifact receipt, bindings and exclusive evidence
   destination. Its seven outputs must match [the baseline references](candidate-controls-reference.json).
   Review all raw streams and source guards before accepting `controls.json`.
3. Review compiler versions, target, release profile, Rust flags, native compiler
   and native build flags for both artifacts. Require actual witnesses and equal
   build environments. Do not fill missing facts with assumptions. If the reused
   baseline lacks these witnesses, perform a controlled hosted rebuild of both
   sources under one captured environment. That requires a **new version** of
   this paired policy to bind the rebuilt baseline bytes; the original reused
   baseline SHA remains preserved in this version.
4. Run [paired.py](paired.py) only after accepted artifact, build-environment and
   candidate-control receipts exist. The resource gate runs before every child.
   Each admitted workload has two pilots, two fresh-process warmups per variant,
   then four alternating ABBA/BAAB blocks: eight timed samples per variant.
   Every output must equal its pilot and the original admitted hosted output.
5. Independently review the actual raw streams, commands, clocks, sample order,
   source/binary/lock guards and statistics. Report medians, MAD, outliers and
   regressions. `comparison_complete` establishes collection and output equality;
   it does not assert a speedup or authorize integration. Required own CI and
   native/MCP/locale/quest controls still apply before a guarded narrow merge.

[Paired freeze](paired-freeze.json) pins both consumers, their references,
the original profiler/inputs and the candidate plan before candidate execution.
Do not rewrite it after outcomes. Preserve failed admissions, partial samples,
resource rejections and source-guard failures. A changed policy needs a new
version with retained original evidence.

The supplied bindings contain `baseline` and `candidate` objects with `head`,
`tree`, `binary_sha256` and `lock_sha256`. They also contain the accepted review
flags and SHA-256 fields consumed by the scripts. Both consumers hash the actual
artifact receipt passed through `--artifact-review`; `paired.py` additionally
hashes `--build-review` and `--candidate-controls-review`. These must match the
bindings' `artifact_review_receipt_sha256`,
`build_environment_review_receipt_sha256` and
`candidate_controls_review_receipt_sha256` fields.

Receipt schemas are deliberately explicit:

- **Artifact review:** `all_checks_pass: true`, plus `baseline` and `candidate`
  objects carrying the same four identity fields. The review itself must prove
  the real artifact/job/source association; these consumers do not reconstruct
  GitHub provenance from a hash or flag.
- **Build review:** `same_build_environment_verified: true`, both binary SHA
  fields, and equal `baseline_environment`/`candidate_environment` objects.
  Required witnesses are `rustc`, `cargo`, `target`,
  `release_profile_sha256`, `native_toolchain`, `rustflags` and
  `native_build_flags`. Empty flags must be explicit observed values.
- **Control review:** accepted actual `controls.json` bytes, including all seven
  ordered passing results, exact candidate identity and equal source guards,
  control-consumer/paired-freeze/plan/reference SHA fields. A review must accept
  those actual underlying bytes; do not replace them with an unbound summary.

Invalid-vessel stderr replaces only the exact script-argument prefix with
`{script}` so relocating the repository does not create a false mismatch. All
remaining stderr bytes, JSON values/array order and ordinary text are retained.
Original raw baseline hashes remain recorded. Masked text equality preserves
the baseline transcript; the existing own quest tests independently check the
concealed identity contract. Parent source blobs are checked by both consumers;
recursive native-module provenance remains part of the artifact/source review.

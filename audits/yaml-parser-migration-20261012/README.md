# YAML parser migration prerequisite

PR782’s own preflight job failed on `RUSTSEC-2024-0449`: the archived
`unsafe-libyaml0.2.11` dependency enters through `serde_yaml_ng0.10.0`.
The [observed blocker](observed-preflight-blocker.json) preserves the exact run,
head, job and raw-log hash. This is a dependency failure separate from the final
safety repair. The [official advisory](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/unsafe-libyaml/RUSTSEC-2024-0449.md)
provides no patched version. Keep `deny.toml` unchanged.

The [original fifty controls](freeze.json) are frozen against accepted main
`a125c88d5e4e97cfc55eb2f51e0490e9cb0ed21b` before execution or production changes.
Thirty positive controls compare complete parsed mechanisms across equivalent
YAML inputs; twenty controls require explicit typed or syntax refusal. They
cover selectors, nested annotations, numeric/complex mapping keys, nonfinite
values, unit-bearing activation energy, aliases, duplicate fields and multiple
documents. Synthetic rates test admission semantics, not measured chemistry.
The [local original baseline](local-original-baseline-review.json) passes all
50 controls, unfiltered, with exact test identities and exit zero. All 2,253
tracked main files and both tracked submodule inventories were rechecked after
execution. This uses an isolated leaf package, its separately captured lock and
default development profile; it does not certify hosted workspace resolution.
An [independent review](local-original-baseline-independent-review.json) verifies
the exact outcomes, tracked source, captured leaf lock and PR783's two-file
preparation. It accepts the original baseline only.
The first attempt stopped on a missing cached dependency before executing any
tests. Its log is preserved separately from the completed attempt.

The [original candidate proposal](candidate-dependency.patch) replaces one workspace
manifest entry with the registry-pinned `noyalib-serde-yaml=0.0.57`, retaining the
existing `serde_yaml_ng` dependency name. Its published facade exports the current
API names and uses native safe Rust. Exact published source was inspected, rather
than relying on its current repository branch. The facade still documents mapping
and tag differences; compatibility is a hypothesis until the frozen controls,
curated packs and affected suites pass. `MechanismError::Yaml` exposes the parser’s
error type, so its underlying public payload type changes despite retained source
spelling. No parser error-message byte equality is claimed.

The proposal and unchanged frozen fixture are now committed separately as
[draft PR783](https://github.com/CrispStrobe/kerotakis/pull/783), head
`24f2ff12878582d480da78a9f721ae59bb2ce7b4`, based directly on accepted main.
[Source preparation](repair-source-preparation.json) binds its two changed files;
[its own CI and fleet](candidate-validation-dispatch.json) are queued. No candidate
compilation, compatibility result or advisory clearance is accepted yet. The
earlier unapplied preparation describes the historical proposal stage.

Production YAML is limited to `kinetics/mechanism.rs`: typed admission, selector
values, ignored or refused annotations and diagnostic rendering. The generic
pack tests also use YAML values. There is no project YAML persistence writer to
migrate. A generic JSON value replacement would narrow YAML value support and
must not be substituted silently.

Executable lanes:

1. Run the dedicated `yaml-parser-baseline` dispatch in
   [the audit workflow](../../.github/workflows/chemistry-audit.yml) on its committed
   audit revision. The collector checks every accepted-source blob, captures the
   actual fresh lock, injects the frozen fixture, preserves all first outcomes and
   runs complete affected libraries. Verify artifact hashes and exact identities.
   Successful capture is not parser-migration acceptance.
2. Review the completed original outcomes and PR783's candidate results.
   Preserve the original fixture and freeze. If the
   facade changes numeric/complex-key or tag behavior, resolve that explicitly;
   do not silently weaken or discard those controls to obtain a passing build.
3. Capture the candidate’s actual resolved lock and dependency graph. Require
   `cargo deny check` with the unchanged policy and proof that `unsafe-libyaml`
   is absent. Repeat the identical fifty controls, compare complete summaries,
   reaction order and provenance of all three shipped mechanism packs, and run
   affected core/CEA tests plus CLI mechanism commands and WASM checks.
4. Require the dependency PR’s own CI and fleet before guarded integration.
   Rebase or stack PR782 deliberately afterwards, then validate its resulting
   source/head/tree and all own gates. Existing PR782-head results cannot certify
   a source that includes the parser migration automatically.

Local large work remains subject to the resource gates. Raw evidence and host
operating details are kept privately. No advisory exception or parser-migration
integration acceptance has been made here. Main remains unchanged; PR783 is a
validation candidate.

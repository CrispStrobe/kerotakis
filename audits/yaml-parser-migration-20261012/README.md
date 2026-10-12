# YAML parser migration prerequisite

PR782’s own preflight job failed on `RUSTSEC-2024-0449`: the archived
`unsafe-libyaml0.2.11` dependency enters through `serde_yaml_ng0.10.0`.
The [observed blocker](observed-preflight-blocker.json) preserves the exact run,
head, job and raw-log hash. This is a dependency failure separate from the final
safety repair. The [official advisory](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/unsafe-libyaml/RUSTSEC-2024-0449.md)
provides no patched version. Keep `deny.toml` unchanged.

The current [PR783](https://github.com/CrispStrobe/kerotakis/pull/783) candidate is
`1acdda4696e565be5dee13f0598eb2aa8081304e`. It uses pinned `yaml_serde=0.10.7`
under the existing dependency name, with the unchanged fifty-test fixture.
[Its source preparation](maintained-fork-source-preparation.json) and
[published-source review](maintained-fork-published-source-review.json) bind the
maintained YAML Organization fork and its `libyaml-rs` backend. The backend still
contains unsafe code. This is a maintenance replacement, not a memory-safety
claim. [All fifty local controls pass](local-maintained-fork-fifty-1acdda46.json),
unfiltered, at this head. All 2,254 candidate tracked files and both tracked
submodule inventories were rechecked; its separately captured leaf lock excludes
archived `unsafe-libyaml`. The [independent fifty-test review](maintained-fork-independent-review-1acdda46.json)
accepts those local controls. The separately captured
[full workspace resolution and unchanged policy](local-workspace-policy-1acdda46.json)
now pass advisories, bans, licenses and sources;
[its independent review](workspace-policy-independent-review-1acdda46.json)
confirms that bounded result. All
[652 affected library tests](local-affected-libraries-1acdda46.json)
(20 CEA, 604 core, 28 safety) and all
[15 unchanged mechanism-pack tests](local-mechanism-packs-1acdda46.json) pass,
unfiltered, with the actual separately captured workspace lock. The original
632 core/safety test identities and outcomes match. These are local checks;
existing ignored native caches were reused.
[This head's own CI and fleet](maintained-fork-validation-dispatch.json)
remain queued/pending. The [local core/data/scene WASM compile check](local-wasm-core-check-1acdda46.json)
passes, with [independent compile/static-control review](wasm-static-cli-independent-review-1acdda46.json).
The separately [frozen fifteen CLI controls](cli-controls/README.md) now
[all pass on the rebuilt candidate CLI](local-cli-first-controls-1acdda46.json).
An [independent CLI review](cli-independent-review-1acdda46.json) accepts those
actual outputs, build provenance and tracked module/source inventories.
They cover shipped-pack inspection and rates, inert bounded simulation, ignored
complex metadata and explicit refusals. The first CLI build failed on generated
upstream documentation absent from the archive; a separate candidate-owned
module archive supplied those derived ignored files without changing tracked
source or the protected original. The retry built successfully.
Remaining bridge/bench-WASM jobs, complete own CI/fleet and integration remain
pending.
[Own-run source associations](own-checkpoint-independent-review-1acdda46.json)
are independently verified; pending native/preflight jobs are not accepted.

The separately frozen [twenty-case supplement](supplement-20/README.md) has
[18 pass / 2 fail on both original and replacement](supplement-20/paired-first-outcomes.json),
with identical per-case outcomes. Custom-tagged scalar and sequence metadata
are refused by both parsers. Those two new positive forecasts remain unmet;
they reveal a pre-existing annotation-admission limit requiring adjudication,
not an observed migration regression. Preserve both first runs and both freezes.
Do not report this supplement as twenty passing expectations.
An [independent affected-regression review](affected-regressions-independent-review-1acdda46.json)
verifies all 652 library and 15 pack outcomes, source/module inventories, locks
and the paired supplement first failures.
[The candidate's own macOS native review](native-macos-independent-review-1acdda46.json)
now accepts 454 focused controls across 38 fixtures, including all fifty YAML
contracts, and the full workspace's 3,345 passing tests with four ignored.
All thirteen library binaries pass (935 tests). The ordinary hosted lock is
uncaptured; no local or fleet lock is borrowed for this result.
[Own Linux native results](native-linux-review-1acdda46.json) now independently
derive the same 454 focused passes across 38 fixtures and all 3,345 workspace
passes with four ignored, at the verified candidate merge tree. All Linux job
steps pass; the ordinary native lock remains uncaptured.
[Independent Linux review](native-linux-independent-review-1acdda46.json)
also verifies exact actual outcomes and all thirteen library inventories
(935 passes); it does not borrow macOS results or the captured fleet lock.
[The own-source browser and IPhreeqc WASM review](own-browser-runtime-review-1acdda46.json)
accepts the completed demo and PHREEQC WASM jobs at the verified candidate merge
tree. Browser controls include five DOM lesson snapshots, PWA, German rendering
and UX checks; Svelte reports zero errors and two warnings. Bridge/bench-WASM,
Linux/preflight and full fleet acceptance remain separate pending gates.
[Own core WASM compilation and browser-engine bridge](own-wasm-core-bridge-review-1acdda46.json)
now pass at that same source-bound merge tree. The bridge verifies absent-solver
refusal, live solving, desktop/browser pH and ionic-strength agreement, species
transport and five live R1 scenarios. Hosted lock and built binaries are
uncaptured; bench-WASM and complete CI/fleet remain pending.
[Own full preflight](own-preflight-review-1acdda46.json) now also passes at the
verified candidate merge tree, including all four cargo-deny categories under
unchanged policy. Ordinary hosted lock and advisory database revision are
uncaptured. Reporting-only upstream counts do not close scientific-data gaps.
[The own release CLI artifact](own-cli-artifact-review-1acdda46.json) and
[its independent review](own-cli-artifact-independent-review-1acdda46.json) bind
the completed fleet build, exact four-file ZIP, actual release binary checksum
and independently captured fleet lock. Its lock hash happens to equal the local
workspace lock; it was hashed from the ZIP separately. Static ELF compatibility
is checked, but this artifact has not been executed and the fleet is pending.

[The annotation contract review](annotation-contract-review.md) records the
observed custom-tag boundary and six annotation owners. It also identifies
nested rate-key admission as a separate inquiry. The separately
[frozen twelve nested-rate controls](nested-rate-controls/README.md) completed
**4 pass / 8 fail** on the unchanged candidate binary. The
[independent first-outcome review](nested-rate-first-independent-review-1acdda46.json)
verifies all captures, source, binary and actual lock: every unsupported key
was accepted with exit zero, no diagnostic and the complete summary of its
valid counterpart. `B: 1` is demonstrably discarded in favor of default `b=0`.
Troe `t2` is accepted; its omission is inferred from source because inspect
omits Troe parameters. Three valid shapes and the missing-required-`Ea`
refusal pass. This confirms a nested admission defect against the documented
supported subset, not a measured chemistry or parser-migration regression.
The separate narrow repair is now [draft PR784](https://github.com/CrispStrobe/kerotakis/pull/784),
head `51e99750`, stacked on PR783. Its
[source preparation](nested-rate-repair-preparation-51e99750.json) binds exactly
two changed files and thirteen multiwitness regression functions. Typed owner/key
refusals cover all supplied nested owners before reaction dispatch. PLOG key
order and supplied Troe `T2` preservation have explicit controls. Formatting
and [independent static review](nested-rate-repair-static-review-51e99750.json) pass; no repaired-source behavior, full CI/fleet or integration
is accepted. Its own hosted CI/fleet are queued.
[The reviewed rebinding helper](cli-rebinding-preparation.md) can prepare new
CLI15 and nested12 bindings from a separately reviewed own artifact while
keeping every original expectation and collector byte fixed. Its eight
synthetic preparation tests pass; no repaired CLI run is claimed. A separately
prepared bulk-storage leaf test remains resource-gated before compilation. Rebase deliberately after the
dependency change integrates, then validate and bind the resulting source anew. Preserve the first freeze and outcomes.

The [manifest patch](maintained-fork-dependency.patch) is separate from the
preserved earlier facade proposal and its observed regression below.

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
The [hosted original baseline](hosted-original-baseline-review.json) is now
verified separately: original50/50 plus core604/safety28, unfiltered, with all
manifested artifact hashes, original source inventory and a fresh hosted
workspace lock. It accepts the original only; its result cannot certify PR783.

The [original candidate proposal](candidate-dependency.patch) replaces one workspace
manifest entry with the registry-pinned `noyalib-serde-yaml=0.0.57`, retaining the
existing `serde_yaml_ng` dependency name. Its published facade exports the current
API names and uses native safe Rust. Exact published source was inspected, rather
than relying on its current repository branch. The facade still documents mapping
and tag differences; compatibility is a hypothesis until the frozen controls,
curated packs and affected suites pass. `MechanismError::Yaml` exposes the parser’s
error type, so its underlying public payload type changes despite retained source
spelling. No parser error-message byte equality is claimed.

The first proposal and unchanged frozen fixture were committed separately as
the initial [PR783](https://github.com/CrispStrobe/kerotakis/pull/783) head
`24f2ff12878582d480da78a9f721ae59bb2ce7b4`, based directly on accepted main.
[Source preparation](repair-source-preparation.json) binds its two changed files;
[its own CI and fleet](candidate-validation-dispatch.json) are queued. The
[first local candidate run](local-candidate-first-outcomes-24f2ff12.json) completed
49 pass / 1 fail: `note_complex_keys` rejects a sequence key inside an ignored
annotation, while accepted main passes it. All candidate tracked files and the
unchanged fixture were rechecked afterwards. The candidate is not accepted.
No advisory clearance or integration is accepted. The
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
2. Preserve the original50/50 and first candidate49/50 outcomes. The published
   facade has only error/stringify policies for complex keys; global stringification
   also changes typed mechanism keys and must not be used to hide the regression.
   The separate maintained-fork snapshot now passes all unchanged controls.
   Review PR783's resulting head and its own candidate results.
   Preserve the original fixture and freeze. If the
   facade changes numeric/complex-key or tag behavior, resolve that explicitly;
   do not silently weaken or discard those controls to obtain a passing build.
3. Capture the candidate’s actual resolved lock and dependency graph. Require
   `cargo deny check` with the unchanged policy and proof that `unsafe-libyaml`
   is absent. Repeat the identical fifty controls, compare complete summaries,
   reaction order and provenance of all three shipped mechanism packs, and run
   affected core/CEA tests plus CLI mechanism commands and WASM checks. In
   particular, run all fifteen unchanged `gas_mechanism_packs` tests, not a
   filtered subset: these check provenance, units, reaction order, compileability
   and bounded combustion behavior. Derive and check their exact identities from
   `crates/kerotakis-core/tests/gas_mechanism_packs.rs` at the candidate head.
   Freeze a separate supplement for custom tags, ordinary merge keys, duplicate
   annotation fields and quoted versus unquoted nonfinite numeric values on
   accepted main before running the replacement. These cases are outside the
   current fifty; preserve first outcomes and scope any refusal-policy change
   explicitly rather than inferring compatibility from public API names.
4. Require the dependency PR’s own CI and fleet before guarded integration.
   Rebase or stack PR782 deliberately afterwards, then validate its resulting
   source/head/tree and all own gates. Existing PR782-head results cannot certify
   a source that includes the parser migration automatically.

Local large work remains subject to the resource gates. Raw evidence and host
operating details are kept privately. No advisory exception or parser-migration
integration acceptance has been made here. Main remains unchanged; PR783 is a
validation candidate.

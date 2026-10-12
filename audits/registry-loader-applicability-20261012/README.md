# Mandatory runtime fallback applicability — frozen controls

The original baseline fails **38 of 50 source-informed engineering controls**:
26 Rust controls yield 6 passes/20 failures; 24 generator cases yield
6 passes/18 failures. The first repair at `62c70f63` passes **50/50**, independently
verified from raw results. [Draft PR781](https://github.com/CrispStrobe/kerotakis/pull/781)
now has a test assertion spelling correction at `7cce5b8f`; validation of that
revised head and integration remain pending. These are engineering controls,
not blind scientific forecasts or empirical chemical accuracy measurements.

## Verified evidence

[Original hosted Rust review](hosted-original-baseline-review.json) binds exact
main `093db6c9a9e64f1d704ded354685cee4ded6a9ec`, tree
`22ab9d3ad14b824274c6c4d7c00e780808fc857f`, all 2,251 regular source blobs,
actual hosted lock, 26 exact unfiltered identities and Cargo exit 101.
Eighteen selector-first negatives and two standalone direct/decoded-pack
runtime witnesses fail. Six compatibility/metadata controls pass.

[Original generator review](generated-first-baseline-review.json) verifies
6 passes/18 failures. All 18 negative inputs exit successfully and emit species
output. The execution uses the preserved original generator artifact, SHA-256
`e9ae23f79b1b31231cba7e70c44681c1e302987acc5453d9c245ca030724a32c`, whose
tracked source tree equals main093. This original generator execution does not
claim a fresh compilation or dependency reproduction.

[Paired candidate review](paired-candidate-review.json) independently verifies
[run 38217196803](https://github.com/CrispStrobe/kerotakis/actions/runs/38217196803):
26 Rust passes, zero ignored/filtered, and all 24 generator cases pass on head
`62c70f634213bbad68a2ef3d99971625f9f0a2db`, tree
`748d01a09cd8f589068a6d8f798e95c98498f5aa`. All 2,253 regular source hashes,
raw artifact hashes, exits and frozen inputs are checked. Saved generator outputs
are independently evaluated again: all 18 refusals identify species/property/
constraint and emit no species output; six compatibility cases each preserve
186 scalar triples/phases and 47 zero-Cp rows. The actual newly compiled
build-script artifact differs from the original. Its compiler report has
`fresh=false` and its raw log contains the compiler invocation. Other dependency
artifacts may be cached. The actual candidate-hosted lock is captured; its
bytes equal the original hosted lock. Neither is the separate local offline lock.
Recursive Gitlink HEAD/status checks do not independently compare all submodule
or ignored environment contents.

## Current repair and next actions

The shared selector checks only the three mandatory records **after** all
uniqueness and phase joins. Non-null temperature, pressure, pH, ionic-strength
or medium constraints explicitly refuse when context-free construction has no
usable context. A non-null quantity-condition phase must match its selected row.
Conditional typed documents and complete hashed-pack metadata remain valid.
Missing/null fields, notes, consistent phase and unused foreign-phase records
remain supported. No measured constants or new reference context are invented.

[Initial source preparation](repair-source-preparation.json) binds the original
three-file repair. Its own CI run 38216841844 failed macOS Clippy before native
tests: two runtime `.err().expect()` assertions trigger `clippy::err_expect`.
[The assertion-port receipt](lint-port-preparation.json) records the correction
at `7cce5b8f861bbe3c5fabdffb5eb8c0ca4acae06f`, tree
`cbe987d1fba5eaeaecbd7a23232fb2105eee8684`: three runtime extraction sites across
two production fixture ports now use `expect_err()`. The non-Debug selector
extraction remains unchanged. Original frozen fixtures, inputs, expectations and
first outcomes remain immutable; this is an explicit production-port adaptation.
Do not borrow earlier-head CI or 50/50 acceptance for the revised head.

Next, collect and independently review the revised head's paired 26+24 controls.
The dedicated `registry-applicability-candidate` workflow selection pins that
head and checks exact assertion normalization against the original fixtures.
Capture the actual lock and newly compiled generator, not an old binary.
Use [the revised native plan](native-controls-plan-7cce5b8f.json) for all
404 focused functions across 37 fixtures, including the prior 378 functions.
Also require complete affected core/data/safety libraries, separate registry
admission, the head's own complete CI, both native inventories and all 33 fleet
jobs with every declared step. Prepare a fresh integration plan and read-only
preflight; only after all gates pass advance the draft and perform an ordinary
head-guarded merge. Verify actual parents/tree and refresh the integration ledger
and current-main presence inventory. Never merge the broad audit branch or use
another PR's old acceptance plan.

```sh
gh workflow run chemistry-audit.yml \
  --ref audit/systematic-chemistry-20261002 \
  -f audit=registry-applicability-candidate
```

Use hosted runners while local load, available memory or disk headroom prevent
builds. Do not start large local work or consolidate unreviewed worktrees under
those conditions.

## Frozen policy and scope

[Original freeze](freeze.json) contains 24 Rust controls: 18 negatives and six
compatibility/metadata controls. Negative helpers observe the selector first;
original failures do not establish their later runtime assertions.
[Supplemental freeze](runtime-route-freeze.json) adds two independent direct and
decoded-pack runtime witnesses without selector prechecks.
[Generator freeze](generated-freeze.json) separately specifies 24 immutable cases.
These give 50 distinct engineering controls, not 50 independent chemistry claims.

[Metadata inventory](source-metadata-inventory.json) records 558 mandatory rows
across 186 species; all quantity-condition phases agree and no other known
constraint is non-null. [Preparatory source binding](source-binding.json) and
[static review](preparatory-review.json) describe preparation, not execution.
The earlier [unapplied proposal](candidate-preparation.json),
[patch](candidate-applicability.patch) and [static review](candidate-preparation-review.json)
retain their historical identities. Their preparation-only statuses are superseded
by the actual source branch and later source-specific execution receipts above.

Contextful interval edges and alternative selection, malformed/unknown condition
containers, optional properties, polynomials, multiple mass anchors and general
typed phase consistency remain separate policies. Closure of this selected,
context-free mandatory fallback slice does not close those broader lanes.

## Reproducing the preserved original baseline

`hosted_original_baseline.py` and the `registry-applicability-baseline` workflow
selection check out exact main093 independently of the audit collector, verify
source bytes/freezes, inject two unchanged fixtures, capture the actual fresh
hosted lock and run both Rust binaries with `--no-fail-fast`. Successful evidence
collection means complete capture, even when contracts fail. Missing compilation,
extra/missing identities or ignored/filtered controls invalidate capture.
Never overwrite original evidence or describe dependency failure as a contract
failure. The original baseline is already captured and reviewed; rerunning it
is unnecessary unless a specific reproduction question requires it.

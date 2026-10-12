# Mandatory runtime fallback applicability — frozen controls

**Integrated as [PR781](https://github.com/CrispStrobe/kerotakis/pull/781), main
`a125c88d5e4e97cfc55eb2f51e0490e9cb0ed21b`.** [Actual merge proof](main-tree-equality.json)
verifies parents `093db6c9`/`7cce5b8f` and tree
`cbe987d1fba5eaeaecbd7a23232fb2105eee8684`, exactly equal to tested source.
The revised repair passes **50/50** engineering controls, both native inventories,
complete own CI and all 33 chemistry fleet jobs. The original baseline remains
12 passes/38 failures: Rust6/20 and generator6/18. Original freezes and first
failures remain unchanged. These are source-informed engineering expectations,
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

## Accepted repair and integration

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

[Revised paired review](paired-candidate-review-7cce5b8f.json) independently
verifies all26 Rust controls and24 generator cases at actual revised head
`7cce5b8f`, including the exact three assertion adaptations, all2,253 source hashes,
actual hosted lock and newly compiled generator. Prior-head results remain
historical, rather than certifying this head.

Own source-bound [Linux](native-linux-review-7cce5b8f.json) and
[macOS](native-macos-review-7cce5b8f.json) each pass **404 focused functions across
37 fixtures**, including the prior378. Separate [registry admission](data-admission-macos-review-7cce5b8f.json)
passes10/10; [affected libraries](library-macos-review-7cce5b8f.json) pass
604core/17data/28safety, all unfiltered. [Complete own CI](full-ci-review-7cce5b8f.json)
verifies26 source-declared jobs,25 successes and the expected publication skip,
with all five required gates. [Complete own fleet](full-fleet-review-7cce5b8f.json)
verifies all33 jobs and every declared named/ordinal step. Ordinary native CI's
actual dependency lock remains uncaptured; the paired run's captured lock is
not substituted for it.

[Source-specific preparation](integration-preparation-7cce5b8f.json) binds11
receipts and all three changed source files. [Final preflight](preflight-ready-7cce5b8f.json)
passes with draftfalse and exact current synthetic/native parents and tree.
The ordinary head-guarded merge additionally checks snapshot freshness, all three
own run identities/attempts/statuses, current PR rollup and immediate head/base/main,
then verifies actual postmerge parents/tree/main. An independent postmerge review
confirms those proofs. Head and base checks are not an atomic base lock; actual
parents/tree are therefore mandatory. The broad audit branch was not merged.
[Current-main historical presence](../integration-20261008/full-audit-port-inventory-pr781-merged.json)
remains50 missing named tests/129 missing paths; presence is not runtime acceptance.

This selected context-free mandatory fallback slice is complete. Next applicability
work must independently specify contextful interval edges and alternative selection,
optional records/polynomials, malformed/unknown conditions or multiple anchors
before collecting a new unchanged-source baseline. Keep those policies separate
from this accepted refusal rule. Completed baseline, paired and CI/fleet runs
need no redispatch. Large future builds and profiling remain resource-gated.

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

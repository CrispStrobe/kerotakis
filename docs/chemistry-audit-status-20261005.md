# Chemistry audit: current state, 2026-10-05


## Current checkpoint — 2026-10-12: both repair-native jobs accepted

PR784's [own Linux results](../audits/yaml-parser-migration-20261012/native-linux-review-51e99750.json)
and [independent review](../audits/yaml-parser-migration-20261012/native-linux-independent-review-51e99750.json)
now verify **3,358 workspace passes/four ignored**, focused 467/39, all thirteen
library binaries (935 passes) and all fifteen gas-pack controls, at exact repair
head `51e99750`. Every Linux outcome is derived from its own actual log; no
macOS result or captured fleet lock is substituted. Both platforms pass formatting
and all-target workspace Clippy with warnings denied. Ordinary native locks remain
uncaptured.

Full preflight and the bench/bridge runtime checks are accepted separately.
Core WASM compilation, demo browser, standalone IPhreeqc, complete CI/fleet and
integration remain pending. Preserve the annotation supplement's two historical
failures and all original freezes. Follow [the next-step lanes](chemistry-work-agenda-20261012.md).

## Earlier checkpoint — 2026-10-12: repair preflight and two WASM runtime gates accepted

PR784's [new completed-gate review](../audits/yaml-parser-migration-20261012/own-completed-gates-review-51e99750.json)
accepts full preflight, WASM bench and browser-engine bridge at exact repair
head `51e99750`. [Independent preflight review](../audits/yaml-parser-migration-20261012/own-preflight-independent-review-51e99750.json)
verifies non-light completion, unchanged policy across all four dependency
categories, 3,358 workspace passes/four ignored, and four additional identity
controls. [Independent WASM runtime review](../audits/yaml-parser-migration-20261012/own-wasm-runtime-independent-review-51e99750.json)
verifies bench 32 controls plus 8,105 protocol checks across 113 lessons and
bridge 10 controls. Ordinary job locks, WASM binaries and execution-time advisory
database revision remain uncaptured; the captured fleet lock stays separate.

Linux native and the remaining core/browser/standalone IPhreeqc jobs are still
pending. Both full fleets remain incomplete; neither PR is integrated.
Follow [the exact next-step lanes](chemistry-work-agenda-20261012.md).

## Earlier checkpoint — 2026-10-12: dependency CI accepted; integration guard awaits fleet

PR783's [own complete CI gate](../audits/yaml-parser-migration-20261012/own-ci-gate-aggregate-1acdda46.json)
is now accepted at exact head `1acdda46`: all five required gates have reviewed
source-bound outcomes, and all 26 jobs have the expected successful/skip states.
Legacy oracle still ran zero tests through its explicit absent-feature skip.
The [read-only integration preparation](../audits/yaml-parser-migration-20261012/integration-preparation-1acdda46.json)
binds the accepted CI and both native 454/38 inventories without changing their
original receipts. [Independent preparation review](../audits/yaml-parser-migration-20261012/integration-preparation-independent-review-1acdda46.json)
checks that adaptation. [Actual readiness](../audits/yaml-parser-migration-20261012/integration-awaiting-fleet-1acdda46.json)
is false because the full 33 fleet is incomplete. No integration is accepted.
Next complete fleet review and rerun fresh guards; PR784 Linux and preflight
remain running or queued. Earlier checkpoints retain their historical scope.

## Earlier checkpoint — 2026-10-12: repaired native libraries and packs accepted

PR784 head `51e99750` now has [own macOS native evidence](../audits/yaml-parser-migration-20261012/native-macos-review-51e99750.json)
and [independent review](../audits/yaml-parser-migration-20261012/native-macos-independent-review-51e99750.json):
**3,358 workspace passes, four ignored, zero failures**; 467 focused passes across
39 fixtures; all thirteen library binaries (935 passes); and all fifteen
unchanged gas-pack controls. The job also passes formatting and all-target
workspace Clippy with warnings denied. Actual checkout, source tree and merge
parents are verified against its own run. Ordinary native lock remains
uncaptured; the separately captured fleet lock is not borrowed.

Together with typed63 and own release CLI27/27, this closes repaired-source
library and pack checks in the accepted macOS scope. Linux native, full preflight,
complete CI/fleet and integration remain pending. No merge is claimed.
Follow [the next-step lanes](chemistry-work-agenda-20261012.md).

PR783's [remaining own CI control review](../audits/yaml-parser-migration-20261012/own-final-ci-controls-review-1acdda46.json)
accepts presentation1,630 passes across123 files. Its Legacy BASIC job performs
an explicit absent-feature skip and runs zero oracle tests. The exact workflow
inventory is26 completed jobs (25 successes, one publish skip). Full required
gate aggregation and complete fleet acceptance remain separate.

## Earlier checkpoint — 2026-10-12: repaired CLI27/27 accepted

PR784's [own release artifact](../audits/yaml-parser-migration-20261012/own-cli-artifact-review-51e99750.json)
and [independent artifact review](../audits/yaml-parser-migration-20261012/own-cli-artifact-independent-review-51e99750.json)
bind exact repair head `51e99750`, its merge tree/parents, archive, binary and
actual fleet lock. [New runtime captures](../audits/yaml-parser-migration-20261012/repaired-cli-controls-review-51e99750.json)
pass **15/15 general CLI controls and 12/12 nested-rate controls**, with original
expectations and collector bytes unchanged. All eight formerly silent unknown
keys now produce the expected refusals. [Independent runtime review](../audits/yaml-parser-migration-20261012/repaired-cli-independent-review-51e99750.json)
verifies all 27 outcomes and source/binary/lock lineage. This supplements the
accepted typed63 regressions; the annotation supplement's two historical failures
remain preserved.

Full repaired-source affected libraries, pack 15, complete own CI/fleet and
integration remain pending. Hosted native tests are running or queued; fleets
are incomplete. Follow [the next-step lanes](chemistry-work-agenda-20261012.md).

## Earlier checkpoint — 2026-10-12: repair leaf passes; dependency CI reports success

PR784 head `51e99750` now passes **50 original parser controls and 13 repair
regression functions**, unfiltered, with exact source and actual leaf lock
checked before and after execution. [The root receipt](../audits/yaml-parser-migration-20261012/local-nested-rate-repair-regressions-51e99750.json)
and [independent review](../audits/yaml-parser-migration-20261012/nested-rate-repair-leaf-independent-review-51e99750.json)
accept this bounded leaf scope. The unchanged supplement is still **18 pass /
2 historical custom-tag failures**, independently verified against both earlier
sources; no forecast or first outcome was rewritten. Full affected libraries,
packs and newly bound repaired CLI controls remain pending.

PR783's [own WASM bench](../audits/yaml-parser-migration-20261012/own-wasm-bench-review-1acdda46.json)
passes 32 runtime controls and 8,105 protocol checks across 113 lessons.
[The hosted status checkpoint](../audits/yaml-parser-migration-20261012/hosted-progress-20261012-1755.json)
reports complete CI success (25 successful jobs, one skipped); it does not
replace detailed reviews of previously unreviewed jobs. Its fleet remains
incomplete. PR784's own CI and release CLI build are still running or queued.
Neither narrow PR nor the broad audit branch is integrated by these results.
Follow [the executable next-step lanes](chemistry-work-agenda-20261012.md).
Earlier dated checkpoints retain their historical scopes.

## Earlier checkpoint — 2026-10-12: both dependency-native jobs verified

[Own Linux native evidence](../audits/yaml-parser-migration-20261012/native-linux-review-1acdda46.json)
now verifies 454 focused controls across 38 fixtures and all 3,345 workspace
passes with four ignored, matching the separately accepted macOS inventory.
Both jobs bind to PR783's actual candidate merge tree; ordinary native locks
remain uncaptured. The [own fleet CLI artifact](../audits/yaml-parser-migration-20261012/own-cli-artifact-review-1acdda46.json)
is independently reviewed with its actual captured fleet lock, binary checksum,
ZIP digest and clean pinned submodule record. It has not been executed locally;
bench-WASM, complete CI/fleet and integration remain pending.

[Source-rebinding preparation](../audits/yaml-parser-migration-20261012/cli-rebinding-preparation.md)
now preserves the frozen CLI15/nested12 expectations and collectors while
recording a separately reviewed new artifact/source association. All eight
synthetic helper tests pass. PR784's repaired behavior is still unverified;
its own hosted jobs are queued and local leaf compilation is resource-gated.
Earlier dated checkpoints below retain their historical scopes.

## Current checkpoint — 2026-10-12: nested rate-key loss confirmed

The [twelve frozen nested-rate CLI controls](../audits/yaml-parser-migration-20261012/nested-rate-controls/README.md)
completed **4 pass / 8 fail**, independently verified against their unchanged
freeze, candidate source, binary and actual lock. All eight unsupported nested
keys were accepted with unchanged successful summaries. `B: 1` is visibly lost
to default `b=0`; Troe `t2` omission is source-inferred because inspect does not
expose Troe parameters. Three valid shapes and missing-required-`Ea` pass.
[The independent receipt](../audits/yaml-parser-migration-20261012/nested-rate-first-independent-review-1acdda46.json)
preserves every first outcome. The separate typed admission repair is now
[draft PR784](https://github.com/CrispStrobe/kerotakis/pull/784), head `51e99750`,
stacked on PR783. Thirteen regression functions and two changed files are bound
in [preparation evidence](../audits/yaml-parser-migration-20261012/nested-rate-repair-preparation-51e99750.json).
Formatting/static checks pass; own CI/fleet are queued. Repaired-source behavior
and integration are pending.

[Own core WASM and browser-engine bridge jobs](../audits/yaml-parser-migration-20261012/own-wasm-core-bridge-review-1acdda46.json)
now pass at PR783's verified merge tree. The bridge includes absent-solver
refusal, live solving, desktop/browser pH and ionic-strength agreement, species
transport and five live R1 scenarios. [Own full preflight](../audits/yaml-parser-migration-20261012/own-preflight-review-1acdda46.json)
now passes, including all four unchanged dependency-policy categories.
Linux/bench-WASM/full CI/fleet remain pending. Ordinary hosted locks and built binaries are uncaptured.
Earlier dated checkpoints below retain their historical scopes.

## Current checkpoint — 2026-10-12: own macOS and browser jobs verified

[PR783's own macOS native evidence](../audits/yaml-parser-migration-20261012/native-macos-independent-review-1acdda46.json)
verifies 454 focused controls, including YAML50, and 3,345 workspace passes with
four ignored. All thirteen library binaries pass (935 tests). The ordinary
native lock is uncaptured. [Completed own browser and PHREEQC WASM jobs](../audits/yaml-parser-migration-20261012/own-browser-runtime-review-1acdda46.json)
are bound to the verified candidate merge tree; they include DOM snapshots,
PWA, German rendering, UX and the declared PHREEQC WASM controls. Complete
Linux/preflight/bridge/bench-WASM/fleet gates and integration remain pending.

[Annotation adjudication](../audits/yaml-parser-migration-20261012/annotation-contract-review.md)
documents the two observed custom-tagged note refusals without rewriting their
positive forecasts. The six actual annotation owners do not establish a nested
rate-key refusal guarantee; that separate source-indicated inquiry is being
[frozen as twelve prospective controls](../audits/yaml-parser-migration-20261012/nested-rate-controls/README.md)
before execution. Their first run remains resource-gated and pending; no
nested-rate production repair is claimed yet.
Earlier dated checkpoints below retain their historical scopes.

## Current checkpoint — 2026-10-12: CLI15 and WASM compile verified

The [candidate parser lane](../audits/yaml-parser-migration-20261012/README.md)
now also passes all fifteen separately frozen CLI controls and the locked/offline
core/data/scene WASM compile check. The CLI binary is bound to PR783 head
`1acdda46`, its actual workspace lock and a completed build. Inspection, finite
rates, inert bounded simulation, complex annotations and four refusals pass.
The first native build failed on generated documentation missing from the
archived module; its failure is preserved and a candidate-owned archive with
verbatim derived documentation completed the retry. No tracked production
change was needed. WASM runtime/browser, complete own CI/fleet and actual
integration remain pending. The separate tagged-metadata forecasts remain18/2
on both parsers. Earlier checkpoints below retain their historical scopes.

## Current checkpoint — 2026-10-12: dependency repair clears local policy

[PR783](https://github.com/CrispStrobe/kerotakis/pull/783) head `1acdda46`
passes the unchanged original fifty parser contracts, all 652 CEA/core/safety
library tests, all 15 mechanism-pack tests, and all four unchanged
dependency-policy checks. The actual candidate workspace lock excludes archived
`unsafe-libyaml`. Independent reviews accept the local fifty and policy scopes.
The replacement remains an unsafe maintained backend; no memory-safety proof
is claimed. [Evidence and remaining lanes](../audits/yaml-parser-migration-20261012/README.md)
bind the exact source, locks and unfiltered outcomes.

The new prospective twenty-case supplement yields identical **18 pass / 2 fail**
on accepted main and the replacement. Two custom-tagged ignored annotations
remain refused on both. Preserve those failed forecasts and adjudicate the
annotation support contract in the separate documented lane. CLI/WASM, own
hosted CI/fleet and integration remain pending. Main is still `a125c88d`; PR781
is integrated and PR782 must be deliberately rebound after the dependency repair.
Earlier dated sections below are historical checkpoints.

## Maintained YAML candidate 50/50 after preserved facade regression — 2026-10-12

The first PR783 candidate completed49/50: it rejected a complex mapping key
inside an ignored annotation that accepted main supports. That failure and source
are preserved. [Revised PR783 head1acdda46](https://github.com/CrispStrobe/kerotakis/pull/783)
uses pinned yaml_serde0.10.7 under the existing dependency name and now passes
[all fifty unchanged controls](../audits/yaml-parser-migration-20261012/local-maintained-fork-fifty-1acdda46.json).
Tracked candidate and submodule inventories were rechecked after execution.
The maintained libyaml-rs backend retains unsafe code; no memory-safety claim
is made. The separately captured leaf lock excludes archived unsafe-libyaml,
but full workspace graph and unchanged-policy cargo-deny remain unaccepted.

Its own revised-head CI38271407738 and fleet38271407728 are queued/pending.
All fifteen shipped mechanism-pack tests, affected core/CEA libraries, CLI/WASM,
additional tag/merge controls and guarded integration remain required. Local
further starts are blocked by load4.52, despite available memory and disk headroom.
Public lane descriptions preserve both first outcomes and executable next steps.
Main remains a125; PR782 still needs the prerequisite and new-source validation.
Earlier entries below describe historical checkpoints.

## Original YAML controls 50/50; dependency candidate PR783 queued — 2026-10-12

The [original parser baseline](../audits/yaml-parser-migration-20261012/local-original-baseline-review.json)
passes all fifty frozen controls with exact unfiltered identities. All tracked
main and submodule source bytes were rechecked. The isolated leaf package has
its own captured lock and development profile; hosted baseline remains queued.
The first cache-miss attempt executed no tests and is preserved separately.

[Draft PR783](https://github.com/CrispStrobe/kerotakis/pull/783), head `24f2ff12`,
contains only the pinned safe-parser dependency alias and unchanged fifty-test
fixture. Its own CI and fleet are queued; candidate behavior, actual graph,
advisory clearance and integration are unaccepted. Preserve complex-key/tag
controls and run shipped packs, affected core/CEA, CLI and WASM before acceptance.
PR782's safety/native/regression checks pass, but its preflight failed on the
parser dependency. After the prerequisite is accepted, its revised source needs
fresh own gates. Main remains `a125c88d`. Earlier entries below are history.

## PR782 native and regressions pass; YAML dependency blocks preflight — 2026-10-12

Both own nativeCI jobs verify454focusedtests/38fixtures plus649libraries at
PR782head24dda. The previous404focusedcontrols andtargetedsafetyClippy also pass,
in addition to frozen50/50 andlocal632libraries. The separate hosted original
baseline independently confirms24pass/26fail and632libraries, with its own actual
capturedlock. [All source-specific receipts](../audits/production-final-rule-matrix-20261012/README.md)
preserve original failures and cache/lock qualifications.

WholeCI failedpreflight on `RUSTSEC-2024-0449`, archivedunsafe-libyaml through
serde_yaml_ng. [The separate migration lane](../audits/yaml-parser-migration-20261012/README.md)
freezes fifty original YAML admission controls and an unapplied one-manifest-line
candidate. [Hosted original parser baseline38266836155](https://github.com/CrispStrobe/kerotakis/actions/runs/38266836155)
is dispatched; no migration or parser outcomes are claimed yet. Keepdeny.toml
unchanged. PR782 remainsdraft; fleet/CLI andfreshsourceintegration remainpending.
Main stilla125; earlier updates below are historical.

## PR782 closes the frozen fifty locally; remaining gates pending — 2026-10-12

[Candidate head24dda97b](https://github.com/CrispStrobe/kerotakis/pull/782)
passes [all fifty frozen controls](../audits/production-final-rule-matrix-20261012/local-candidate-fifty.json).
Original24pass/26fail and complete baseline632librarypass remain preserved.
The first candidate library invocation was rejected at load6.14; a fresh
gate later opened and all604core+28safety library tests passed. Previous404
focused controls, lint, own nativeCI/fullfleet and guarded integration remain
pending. Further build starts are blocked by load10.85/MemAvailable2617MiB. The new expected native inventory is454functions
across38fixtures, derived from exact source with all prior fixturebytes unchanged.
Synthetic rule routing and actual production CLI chemistry are separate claims.
Main remains `a125c88d`; PR782 is a draft, not an integrated repair.

## Final-rule first failures verified; narrow PR782 drafted — 2026-10-12

The [original fifty-control baseline](../audits/production-final-rule-matrix-20261012/local-original-baseline.json)
completed24pass/26fail; complete core604/safety28 libraries pass.
[Independent review](../audits/production-final-rule-matrix-20261012/local-baseline-independent-review.json)
verifies all source/module blobs, frozen inputs, actual offline lock and exact
outcomes. Twenty-two direct warnings and four accepted-journal warnings are
missing. [Draft PR782](https://github.com/CrispStrobe/kerotakis/pull/782) adds only
the four reviewed whitelist entries and ports the unchanged fifty controls.
Candidate execution is running separately; native/CLI/CI/fleet and integration
acceptance remain pending. The separate hosted baseline remains queued. Original
failures and resource/cache qualifications are preserved.

## Remaining final-safety fifty frozen; hosted baseline dispatched — 2026-10-12

[Fifty original source-informed controls](../audits/production-final-rule-matrix-20261012/README.md)
are frozen against accepted main `a125c88d` for the four remaining reviewed
final-state rules. They cover introduction/identity, cutoffs, priority, exact
warning metadata, accepted journals and discarded refinement. Synthetic
inventories and pH test routing; these are not physical chemistry predictions.
[Hosted run38252519107](https://github.com/CrispStrobe/kerotakis/actions/runs/38252519107)
was dispatched at audit revision `2deeb4d2` after the local resource gate rejected
host load. The collector also runs complete core/safety library suites. No
outcomes or production repair are claimed at dispatch. Read and preserve raw
first failures before any narrow repair; native/CLI/CI/fleet acceptance remains
separate. Earlier PR781 integration is complete.

## Mandatory fallback applicability integrated — 2026-10-12

[PR781](https://github.com/CrispStrobe/kerotakis/pull/781) is merged as main `a125c88d`, with [actual parents/tree independently verified](../audits/registry-loader-applicability-20261012/main-tree-equality.json) equal to tested head `7cce5b8f`. Its own revised paired50/50, both native404-function/37-fixture inventories, registry admission10, complete604core/17data/28safety libraries, completeCI26 and full 33 fleet/every declared step are accepted. Fresh preflight and immediate run/rollup/head/base/main guards preceded the ordinary merge. Original Rust6pass/20fail and generator6pass/18fail, frozen fixtures and exact assertion-port adaptation remain preserved. Ordinary native CI's lock stays uncaptured; paired execution captures its own actual lock and newly compiled generator.

The selected context-free mandatory fallback slice is closed on main. Contextful intervals/alternatives, optional properties/polynomials, unknown conditions, multiple mass anchors and empirical datasets remain separate. [Current-main historical presence](../audits/integration-20261008/full-audit-port-inventory-pr781-merged.json) still has50 missing named tests/129 paths, not179 demonstrated defects. Next: remaining reviewed final-safety rule matrix, coherent historical regression ports, then measured release optimization when resources permit. Earlier pending/preparation updates below are history.

## Applicability candidate 50/50; revised-head validation pending — 2026-10-12

[Independent paired review](../audits/registry-loader-applicability-20261012/paired-candidate-review.json) verifies all 26 Rust and24 generator controls passing at PR781 head `62c70f63`, with actual source, lock, freshly compiled generator and raw outcomes. Original26 Rust controls remain6 pass/20 fail; original24 generator cases remain6 pass/18 fail. This closes the candidate's frozen engineering expectations, not empirical chemical accuracy or integration.

Broader macOS CI failed before native tests on `clippy::err_expect`. [The correction](../audits/registry-loader-applicability-20261012/lint-port-preparation.json) changes only three runtime assertion extraction sites in production ports; frozen audit fixtures remain unchanged. PR781 now has head `7cce5b8f`. Next: validate that head's paired50, ownCI/native404 across37fixtures and full 33 fleet, then fresh guarded integration. Prior-head results cannot certify the revised head. [Exact resume instructions](../audits/registry-loader-applicability-20261012/README.md) and [today's agenda](chemistry-work-agenda-20261012.md) are current; earlier pending/preparation statements below are history.

## Applicability repair prepared as PR781 — 2026-10-12

[DraftPR781](https://github.com/CrispStrobe/kerotakis/pull/781) applies the narrow shared-selector applicability rule and ports two unchanged frozen fixtures. The original hosted26-control baseline is independently verified at6 pass/20 fail, with actual source/lock/raw-log bindings and standalone direct/decoded-pack failures; original24 generator cases remain6 pass/18 fail. OwnCI38216841844/native404/37fixtures/fullfleet38216841880 and paired validation38217196803 are pending. Actual candidate50/50, repair acceptance and main integration are not yet claimed. Earlier queued-baseline/unapplied-proposal updates below are history.


## Hosted applicability collection and today’s agenda — 2026-10-12

[Run38213787330](https://github.com/CrispStrobe/kerotakis/actions/runs/38213787330) is dispatched to collect the unchanged26 Rust baseline controls on exact main093, independently of the audit workflow source. It was queued at dispatch; no test outcomes or repair acceptance are claimed. The dedicated job preserves actual source and dependency evidence, and successful capture is distinct from passing contracts. [Today’s agenda](chemistry-work-agenda-20261012.md) prioritizes scoped applicability closure, remaining final-safety coverage, historical regression binding restoration and measured release optimization.


## Applicability generator baseline — 2026-10-12

[The first generator baseline](../audits/registry-loader-applicability-20261012/README.md) is independently reviewed:6 pass/18 fail across24 frozen cases. All selected context-constrained or condition-phase-conflicting negative inputs were silently admitted and emitted species output. The six compatibility cases preserve186 scalar triples/phases and47 zero-Cp rows. Execution used the previously observed E644 artifact with tracked tree equal to main093; no rebuild is claimed. The separate26 Rust controls remain unexecuted. A narrow candidate patch is prepared but unapplied/uncompiled; original Rust baseline, paired repair validation and own integration gates remain pending. Earlier preparation-only statements below are history.


## Applicability preparation — 2026-10-12

[26 mandatory fallback controls](../audits/registry-loader-applicability-20261012/README.md) are frozen on exact accepted main093 before execution. The inventory covers558 mandatory records/186 species, with matching phase metadata and no other non-null applicability constraints. The proposed narrow policy preserves valid conditional typed documents and pack metadata, while refusing selected context-dependent runtime fallbacks when construction supplies no usable context. Two separate route witnesses avoid the original negatives' selector-first limitation. Baseline tests, generator controls, any repair and its acceptance remain pending; no new demonstrated failure or closure is claimed.


## Current registry integration update — 2026-10-12

Main `093db6c9` now contains the independently accepted mandatory scalar-bound, unit-normalization and phase-selection repairs. [PR779 acceptance](../audits/registry-loader-units-20261011/main-tree-equality.json) and [PR780 acceptance](../audits/registry-loader-phase-selection-20261011/main-tree-equality.json) bind each own complete CI/native/full 33 fleet and verified actual merge parents/tree; [PR778 acceptance](../audits/registry-loader-properties-20261011/main-tree-equality.json) remains unchanged. No new-base rerun or independently captured hosted lock is claimed.

[The current checkpoint](../audits/integration-20261008/STATUS.md) and [doable lanes](chemistry-audit-next-lanes.md) now start with mandatory fallback applicability contracts, historical binding restoration and remaining safety coverage. Optional curves, multiple mass anchors and empirical data are separate. Current historical presence gaps remain50 named tests/129 paths; full historical audit acceptance remains source-specific. Earlier queued/draft updates below are history.

## Current scalar integration update — 2026-10-11

[PR778 is merged](../audits/registry-loader-properties-20261011/main-tree-equality.json) as main `e391f64b`, after independent complete CI/native333/full 33-fleet acceptance and actual parent/tree verification. The shared scalar-bound admission repair now reaches main across runtime loading, typed validation, hashed packs and generation, preserving supported zero-Cp and signed-energy conventions.

[PR779’s current-main preparation](../audits/registry-loader-units-20261011/README.md) verifies tracked-tree compatibility without claiming another test execution after retargeting. Its own CI/native358 are accepted; fleet26/33 and integration remain pending. PR780 own CI/native378 are accepted; its own fleet and parent integration remain pending. [The current checkpoint](../audits/integration-20261008/STATUS.md) and [next lanes](chemistry-audit-next-lanes.md) define remaining work. Historical full-audit bindings still need50 tests/129 paths on actual merged main; broad phase/schema/empirical-data acceptance remains separate. Older updates below are history.

## Current integration update — 2026-10-11

[PR777 merged-tree acceptance](../audits/production-acid-projection-20261010/main-tree-equality.json) verifies main `bf6f4157`, actual merge parents and exact equality to the reviewed Dbe tree. Both native322 focused inventories, completeCI, all 33 chemistry fleet jobs and declared steps, source-boundCLI164 and paired warning evidence are accepted. Original forecast and acid-route qualifications remain unchanged.

DraftPR778 is based on main and has complete ownCI plus both native333 focused inventories accepted; its own fleet remains pending. [Current-main source-tree compatibility](../audits/registry-loader-properties-20261011/current-main-tree-compatibility-f515.json) is verified, without claiming a new-base test rerun; final fresh premerge guards and actual merge verification remain required. Unit repair PR779 has complete own CI and both native358 focused inventories accepted; its fleet and parent integration remain pending. Phase repair PR780 also has complete own CI and both native378 focused inventories accepted; its fleet and parent integration remain pending. [Current checkpoint](../audits/integration-20261008/STATUS.md) and [next lanes](chemistry-audit-next-lanes.md) define the remaining work. Historical full-audit coverage still needs50 missing tests/129 paths; newer narrow repairs do not replace its source-specific acceptance. Older dated updates below are history.

## Current integration update — 2026-10-10

[Current checkpoint](../audits/integration-20261008/STATUS.md): #776 is merged as6ea6938f with exact tested-tree equality, native267 on both platforms, complete CI/five gates/fleet, source-bound CLI164 and an accepted actual CLI warning pair. Three reviewed final rules now distinguish new from unchanged findings; the chloramine positive emits its reviewed warning in JSON/text while setup and the benign counterpart stay quiet. Original replay qualifications remain unchanged.

[Next lanes](chemistry-audit-next-lanes.md) prioritize the remaining reviewed-rule tranche, historical audit binding restoration (50 missing named tests/129 bound paths), authoritative numeric ownership/reconstruction and measured release optimization. Historical full-audit acceptance remains only8cdab1f6. Older checkpoints below are retained history.

Start here, then choose a task from [the next-step lanes](chemistry-audit-next-lanes.md). This checkpoint covers the independent chemistry audits and their general repairs. Existing product tasks retain their IDs in [PLAN.md](../PLAN.md), [OPTIMIZATION.md](../OPTIMIZATION.md), [ROADMAP-GUI.md](../ROADMAP-GUI.md) and [ROADMAP-Webapp.md](../ROADMAP-Webapp.md).

## Which revision this describes

The accepted engine source is [`8cdab1f6ecd0b1132c4008014a5a0c5476429d17`](https://github.com/CrispStrobe/kerotakis/commit/8cdab1f6ecd0b1132c4008014a5a0c5476429d17). Reports and acceptance receipts were published through [`9331f9e9`](https://github.com/CrispStrobe/kerotakis/commit/9331f9e974ff460a91a1617883ae22e1d319e4d1) on [`audit/systematic-chemistry-20261002`](https://github.com/CrispStrobe/kerotakis/tree/audit/systematic-chemistry-20261002).

At implementation checkpoint `9331f9e9`, that branch was 168 commits ahead of `main` at [`f7d45f6f`](https://github.com/CrispStrobe/kerotakis/commit/f7d45f6fefacc0a53312356f8b801f7e371b8042), with no commits behind it. Subsequent documentation commits do not change that accepted engine source. **Branch-verified does not mean merged into main, released, or deployed.** A documentation-only cherry-pick also does not port the implementation. Recheck ancestry before starting integration lane AUD-00.

## Accepted evidence

| Evidence | Exact result | Public record |
| --- | --- | --- |
| Full source audit | 1,865 Rust test executions; 17 successful native/WASM/CLI stages; 198 source bindings | [Run 37241135174](https://github.com/CrispStrobe/kerotakis/actions/runs/37241135174) |
| Artifact integrity | 432 checks binding source, logs, dependency lock, executable and control captures | [Integrity receipt](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/accepted/full-integrity.json) |
| Third fifty replay | 1,065 checks; nine scientific/protocol qualifications retained | [Run 37242349555](https://github.com/CrispStrobe/kerotakis/actions/runs/37242349555) |
| Fourth fifty replay | 1,148 checks; 124 text/JSON captures for 50 originals and 12 separate followups | [Run 37242514682](https://github.com/CrispStrobe/kerotakis/actions/runs/37242514682) |
| Latest repair baseline | 15 core failures and four native readback failures demonstrated before repair; 33 new frozen boundary tests pass afterward | [Crystal report](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/RESULTS.md) |

The accepted executable SHA-256 is `67797d56ced1531f44dce7252c272b34d19183a4d4571a076b9dcf67d3a634cb`. [Full validation](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/accepted/full-validation.json) and [the binding index](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/strict-solid-solution-owner-contracts-20261004/accepted/binding-check.json) preserve the exact identities. CI artifacts may expire; checked-in receipts remain public, but receipts alone cannot recreate absent raw output. Use a new revision-bound hosted run when raw evidence is unavailable.

These are selected audit gates, not a claim that every workspace test, platform, browser or release gate passed on this revision. Rust test executions include repeated feature-mode tests. New boundary tests are source-informed controls, not additional blind experiments.

## What the branch repairs

- Transfer preparation certifies donor and receiver proposals; relevant mechanical operations screen contextual prospective state and commit accepted proposals. Spill creation/recovery has explicit ownership and rollback contracts.
- Extraction uses sequential fresh-solvent contacts at actual contact temperatures, screens each stage and commits atomically. Bounded coefficient/saturation coverage remains explicit.
- Accepted final equilibrium states have an opt-in safety hook. Titration uses that hook inside virtual dosing, discards refused trials and preserves earlier accepted increments and stock accounting.
- `SolverStack::with_required_conservation` provides opt-in represented element/formal-charge closure. Compensated inventory differences precede element totals, so unchanged background cannot conceal a trace change. Unsupported owners and unsafe bounded arithmetic refuse.
- The strict certificate now admits the existing typed CaCO3/SrCO3 crystal owner. General transaction checks reject malformed/ambiguous crystals and overflowing aggregate quantities. Raw native crystal components refuse negative/nonfinite values before correction and identify their selected column.
- Zero-work operations skip solver/safety processing; finite zero/scientific-notation wait and charge inputs have explicit syntax contracts. Nuclide ledgers have canonical persisted isotope identities and state validation.
- `Amount` provides bounded two-component arithmetic. Opt-in stock owns it authoritatively and persists its mode; vessel/still quantities are not thereby migrated.
- Sparse iodine optics exposes one reviewed, condition-specific aqueous datum with explicit scope. Copper displacement narration describes the supported pathway and actual remaining copper.
- Distillation, native trace serialization, phase ownership, donor/receiver accounting and refused-operation rollback have expanded preserved controls. Explicit refusal remains distinct from completing a physical forecast.

The [preceding systematic report](https://github.com/CrispStrobe/kerotakis/blob/9331f9e974ff460a91a1617883ae22e1d319e4d1/audits/post-equilibrium-contracts-20261004/RESULTS.md) and [audit index](../tools/chemistry-audit/README.md) explain the earlier stages.

## What remains open

| Gap | Next lanes | Boundary to preserve |
| --- | --- | --- |
| Unbounded native Ca/Sr/C correction, phase overdraw and unsupported reporting normalization | AUD-01, AUD-02 | Corrected conservation is not independent evidence of accurate raw output |
| Authoritative compensated vessel/still ownership, public API, saves and solver reconstruction | NUM-01–NUM-04 | Stock adoption and strict ledger arithmetic do not migrate physical inventories |
| Surface/exchanger and incomplete material owners in the strict certificate | OWN-01, OWN-02 | No generic formula fallback or blanket admission of partially represented matter |
| Native trace/oxidation-state thresholds | VAL-01 | Numerical noise and real trace matter need different evidence |
| Observable support, missing identity/data, density, spectra and reaction heat | OBS-01, DATA-01–DATA-03, ENG-01 | Computed zero, unavailable data and uncalibrated estimates must stay distinguishable |
| Imposed headspace work, kinetics and transient electrochemical migration | ENG-02, KIN-01, ELEC-01 | Equilibrium and steady transport do not establish these capabilities |
| Complete transaction/presentation coverage and independent holdouts | TXN-01, UX-01, EXP-01 | Passing isolated examples does not close every operator path |
| Release performance attribution | PERF-01 | No current speedup or release-equivalence claim |

Strict molecular closure does not certify aqueous speciation, reaction energy, kinetics or nuclear closure. Typed multi-crystal bookkeeping does not extend the native adapter beyond one mixed phase in its existing dilute inorganic wateq4f domain. Generic unregistered bulk SrCO3 remains refused.

Four exact legacy exclusions remain disclosed in the full validator: pure-water extraction without extractable solute; the zero-transfer screen-call observer; and two extreme-ratio Drain forecasts in the modeled single-phase domain. Original tests and supported replacement controls remain preserved. The third replay retains nine qualifications. Fourth-original classifications remain 21 bounded agreements, seven qualified agreements, 13 expected invalid-input refusals and nine author protocol errors; followups retain two author errors, three bounded agreements and seven qualified agreements.

The earlier corrected representative profile covered ten workloads, 20 warmups and 80 timed debug-process runs at an earlier source. Median wall changes ranged from about 0.6% faster to 2.5% slower. Four repetitions, debug startup and those source identities do not establish a release speedup. See [run 37219758069](https://github.com/CrispStrobe/kerotakis/actions/runs/37219758069).

## Pickup order

1. Read this checkpoint, the chosen lane and its linked contracts. Establish the actual branch and implementation revision.
2. Start AUD-00 for integration. AUD-01, NUM-01 and OBS-01 designs can proceed independently against the accepted source; implementation ports must respect their dependencies.
3. Freeze new expectations and positive/refusal controls before production changes. Preserve failed evidence and immutable original experiments.
4. Finish one bounded lane with source-bound validation and a reviewable PR. Record source, executable, checks, retained qualifications and remaining scope in its report.
5. Update this checkpoint when accepted behavior changes. Environment, artifact-storage locations and machine resource decisions belong in private operating notes, not public task descriptions.

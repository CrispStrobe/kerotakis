# Mandatory runtime fallback applicability — frozen controls

Status: **Generated baseline: 6 pass / 18 fail across 24 frozen cases; the
separate 26 Rust controls remain unexecuted.** These are source-informed
engineering controls. Source is accepted main
`093db6c9a9e64f1d704ded354685cee4ded6a9ec`, tree
`22ab9d3ad14b824274c6c4d7c00e780808fc857f`.

[The metadata inventory](source-metadata-inventory.json) finds 558 mandatory
records across 186 species, all with matching quantity-condition phase and no
non-null temperature, pressure, pH, ionic-strength or medium constraints.
[The preparatory source binding](source-binding.json) compares all 2,251 tracked
regular files to the isolated source. Gitlink revisions are recorded; their
contents were not freshly compared. The separate offline lock is local evidence,
not a captured hosted lock or authoritative dependency reproduction.

[Independent preparatory review](preparatory-review.json) verifies the frozen
identities and exact-source metadata; it does not claim execution.

## Frozen policy and limits

Context-free runtime construction supplies no temperature, pressure, pH,
ionic-strength or medium context. A selected mandatory fallback with any such
non-null constraint should explicitly refuse with species, property and
constraint in the diagnostic. A non-null quantity-condition phase should agree
with the selected row phase. Missing/null fields, consistent phase and notes
remain supported. Ignore valid unused foreign-phase records.

Conditional typed documents and correctly hashed pack DTOs must remain valid and
retain their complete metadata. The refusal applies when converting them into
context-free runtime species. Conditions-aware resolution remains separate.
No scientific constants or measurements are supplied by these synthetic inputs.

[Original freeze](freeze.json): 24 controls, including 18 refusal controls and
six compatibility/metadata controls. Its negative helper checks the selector
first; if that fails, later direct and pack-runtime assertions are not observed.
Do not count those later assertions as independent baseline observations.
[Supplemental freeze](runtime-route-freeze.json): two independent direct and
decoded-pack runtime witnesses, without a selector precheck. Total: **26 distinct
test functions**, not 26 already observed outcomes.

The Rust fixtures do not execute the build generator. The separately frozen
generator cases and their observed outcomes are described below. Contextful interval edges,
alternative selection, malformed/unknown conditions, optional properties,
polynomials, multiple molar-mass anchors and general typed phase consistency are
outside this slice. All 186 canonical scalar triples and 47 zero-Cp rows are
compatibility expectations, pending execution.

## Observed generator gap and prepared repair

[Separately frozen generator controls](generated-freeze.json) contain 24 cases.
[Independent first-baseline review](generated-first-baseline-review.json) verifies
**6 passes / 18 failures** from preserved inputs, exits, logs and generated files.
All 18 expected refusals exited successfully and emitted species output: three
selected mandatory properties with each of five known context constraints, plus
three conflicting quantity-condition phases. The six compatibility cases each
preserve all 186 scalar triples/phases and 47 zero-Cp rows.

This executed the previously observed E644 generator artifact, SHA-256
`e9ae23f79b1b31231cba7e70c44681c1e302987acc5453d9c245ca030724a32c`.
Its tracked source tree equals accepted main093. This is a new execution of a
preserved artifact, without a rebuild or a new dependency-reproduction claim.
It does not establish the outcomes of the 26 pending Rust controls.

[The candidate patch](candidate-applicability.patch) adds applicability checks
after uniqueness and phase selection, on only the three selected records. It
preserves typed/pack admission and unused foreign-phase records. The patch is
**unapplied and uncompiled**; a check-only application succeeds on the baseline
source. [Preparation metadata](candidate-preparation.json) records its identity.
[Independent static review](candidate-preparation-review.json) confirms the
policy and unchanged baseline, without execution acceptance.
Collect and review the original Rust baseline before applying it. Then run the
paired candidate, prior controls, affected libraries and the same generator
cases against an independently source-bound newly compiled artifact. No repair
acceptance or integration is claimed.

The generator collector requires explicit artifact and registry hashes, refuses
an existing output directory, preserves every case even on failure, and exits
nonzero if any contract fails. Run it with a verified artifact from the declared
source; the collector cannot infer source provenance from a filename:

```sh
python3 audits/registry-loader-applicability-20261012/fixtures/generated_applicability_contracts.py \
  --binary target/debug/build/kerotakis-core-c887cd5668b1fa5c/build-script-build \
  --binary-sha256 e9ae23f79b1b31231cba7e70c44681c1e302987acc5453d9c245ca030724a32c \
  --registry data/registry/registry-source-v1.json \
  --registry-sha256 48487e1940c5e0122207d352c78e4377f8f7b47d421be2f78d62b61a096c3774 \
  --out audit-results/applicability-generator-baseline
```

The recorded artifact location is historical and may be absent. Verify a
preserved copy or build/source-bind a new artifact; never substitute an unrelated
binary. Original 26 Rust functions plus 24 generator cases form 50 engineering
controls, **not 50 executed outcomes or blind chemistry experiments**.

## Resume the baseline

Use an isolated checkout of the exact source above. Obtain these frozen files
from the audit branch and verify their SHA-256 against their respective freezes
before copying them into the recorded `injected_fixture_path` destinations.
Refuse to overwrite an existing fixture or raw result. Record the actual source,
lock, command, exit status and complete output for each invocation. Recheck host
resources before starting each build and use one Cargo worker.

```sh
CARGO_BUILD_JOBS=1 cargo test --offline --locked -p kerotakis-core \
  --test registry_loader_applicability_contracts
CARGO_BUILD_JOBS=1 cargo test --offline --locked -p kerotakis-core \
  --test registry_applicability_runtime_route_contracts
```

An offline lock/cache must already be available; dependency failures are not
contract failures. Preserve exact test names, failures and zero ignored/filtered
counts. Independently review the first outcomes before editing production.
If a repair is justified, change the shared selector narrowly on a main-based
source branch, port these fixtures byte-for-byte, and verify prior scalar/unit/
phase controls plus complete affected libraries. Require the repair's own CI,
both native inventories, full chemistry fleet and actual merge-tree acceptance.
The broad audit branch must not be merged as the repair.

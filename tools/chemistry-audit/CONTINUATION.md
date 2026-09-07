# Chemistry audit continuation

## Current state

- Worktree: `/mnt/volume1/kero-experiment-audit`.
- Branch: `audit/chemistry-experiments-20260906`.
- PR: #504; do not merge.
- Current base: `139a18d6b70f0d19c7a65e4e7699e5c30c3b58e0`. The rebased
  head must be read from Git because this document is part of its final commit.
- Pre-rebase head `b5894243` is retained by backup branch
  `backup/audit-504-before-main-sync-20260907-141633`.
- Catalog: 113 unique entries, 28 models, 189 defined concepts and 167 used
  concepts. Preserve all entries and translations.
- `crates/kerotakis-core/tests/golden/lessons.actual.json` is an intentionally
  retained untracked diagnostic artifact. Do not delete or stage it casually.
- CI run 34133207021 tested the rebased PR and failed preflight, native Linux,
  native macOS and the browser demo. The element/i18n/registry/Scene causes and
  narrow repairs are recorded in `HISTORY.md`; rebuilt CI is still required.
- Expansion and catalog work are paused until the repaired PR run passes.
- The sixth fleet has 24 inputs and 66 frozen checks. It has not run on a CLI
  built from the repaired head. Its contract is `SIXTH-BATCH.md`.
- Private source-derived fleets may finish outside this repository. Their data
  must not enter PR #504.

## Required order

1. Commit only the reviewed CI repair. Exclude `scene_golden.rs`, every
   `*.actual.json` file and all temporary diagnostics.
2. Push with an explicit lease against the observed remote audit head.
3. Inspect PR #504 checks without changing the branch; poll no more often than
   every 300 seconds.
4. If CI is green, record that completion in `HISTORY.md`; remove the resolved
   CI blocker from this file and notify the integrator.
5. If CI fails, download failed logs and generated artifacts once. Map each
   failure to a specific source or expectation before editing.
6. Fix only audit-owned causes. Preserve all 113 entries, every diagnostic test
   and every evidence artifact. Do not replace conflicted or generated files
   wholesale; compare records and fields semantically.
7. Run narrow non-compiling checks locally where safe. Let CI own full Rust,
   browser and cross-platform builds while the host remains memory constrained.
8. After repaired CI passes, use the exact rebuilt CLI for the sixth fleet in a
   fresh output directory. Run the frozen analyzer without changing its bounds.
9. Classify every sixth result as pass, fail, unsupported or uncertain. Diagnose
   failures before proposing a production change.
10. Only then review distinct successful lessons for catalog inclusion. Coordinate
    with the integrator before editing shared catalog/discovery files.

## Task scopes

### CI observer

Read this file and `INTEGRATOR-HANDOFF.md`. Make no edits. Poll GitHub no more
often than every 300 seconds. Report run ID, tested head/merge SHA and every job
state. A queued or missing run is not success.

### CI failure diagnostician

Receive named failed jobs and explicit file ownership. Read failed logs and
artifacts, compare the tested merge tree with branch head, and return root cause,
affected fields, proposed narrow repair and verification. Do not commit, push,
merge, bless snapshots or edit outside assigned files.

### Snapshot reconciler

Act only on explicitly assigned files. Unless edit authority is also delegated,
review without changing, committing or pushing. Prove whether differences are
semantic or serialization-only by comparing both platform artifacts, stable
record identities, field values and ordering. Retain all 113 catalog entries,
179 registry identities and 37 heat-capacity polynomial records. Never accept an
artifact merely because CI generated it.

### Sixth-fleet runner

Begin only after CI passes and the runner is given a rebuilt CLI whose source
revision and SHA-256 are recorded. Follow `SIXTH-BATCH.md`. Use a new output
directory, one isolated process per case and immutable scripts. Preserve raw
stdout, stderr, exit state, binary/source hashes and failed checks.

### Sixth-fleet reviewer

Read the frozen contract and raw outputs. Recompute laws from reported state;
do not copy simulator answers into expectations. Check required events, finite
nonnegative inventory, conservation, model domain and matched controls. Separate
wrong expectations from production defects and unimplemented physics.

### Catalog reviewer

Start only after the corresponding experiments pass scientific review. Prefer
one lesson with meaningful variants over duplicate cards. Require original EN/DE
prose, per-step guidance, prediction diagnosis, model boundary, provenance and
snapshot/lint coverage. Do not publish private source mappings.

Current candidates are diprotic remaining-equivalent capacity, zinc precipitation
with acid reversal, and conductivity with blank/neutral controls. Repeated
equilibrium and minor parameter variations belong primarily in regression tests.
This is a review queue, not authority to edit the catalog.

## Evidence and scientific contract

- Audit IDs label evidence only; runtime code must not recognize them.
- Checks use independent equations, stoichiometry, conservation or paired
  controls. Simulator output is never an oracle for its own expected range.
- Tolerances are absolute plus relative where depletion makes percentage-only
  checks unsafe. Do not widen them after observing output.
- Open, sealed, pressure-controlled and reservoir boundaries have different
  ledgers. Count transferred or escaped matter at the correct interface.
- Analytical inventory, free species, activity, phase and instrument response
  are different observables. Passing one does not certify the others.
- Equilibrium does not establish rate, catalyst, nucleation, apparatus or
  detection behavior. Unsupported endpoints must say what extension is needed.
- A proposed extension needs a reusable acceptance experiment and permissively
  licensed data when parameters are required.

## Git and coordination protocol

Before editing, record `git status --short`, branch, head and remote head. Avoid
unrelated dirty files. Before pushing, run scoped tests, `git diff --check`, and
semantic count/preservation checks. Push with:

```sh
git push --force-with-lease=refs/heads/audit/chemistry-experiments-20260906:<observed-remote-head> \
  origin HEAD:refs/heads/audit/chemistry-experiments-20260906
```

Use force-with-lease only because prior coordination requires an explicit lease;
never use an unguarded force push. Do not rebase again without direct authority.

When a task finishes, move its outcome and decisive evidence into `HISTORY.md`,
then remove its instructions and status from active documents.

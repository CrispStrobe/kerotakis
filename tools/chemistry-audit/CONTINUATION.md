# Chemistry audit continuation

> **Superseded record, 2026-09-08.** PR #504 was closed unmerged: its product
> half landed as #529 and this toolkit landed separately. The worktree, branch,
> CI run and task queue described below are historical. Nothing here is a live
> instruction. See `README.md` for current state and for the evidence archive.

## Current state

- Worktree: `/mnt/volume1/kero-experiment-audit`.
- Branch: `audit/chemistry-experiments-20260906`.
- PR: #504; do not merge.
- Actual audit merge base: `139a18d6b70f0d19c7a65e4e7699e5c30c3b58e0`.
  Current `origin/main` observed before the documentation amendment:
  `2d364e0dfccb7d635ceb12a0fa9c6001b2b14dea`. No further rebase is authorized.
- CI repair commit: `ff7edc6cd9805c8665a5865cc6a71e85b39ee833`.
  Read the current branch head from Git because this document follows it.
- Pre-rebase head `b5894243` is retained by backup branch
  `backup/audit-504-before-main-sync-20260907-141633`.
- Catalog: 113 unique entries, 28 models, 189 defined concepts and 167 used
  concepts. Preserve all entries and translations.
- `crates/kerotakis-core/tests/golden/lessons.actual.json` is an intentionally
  retained untracked diagnostic artifact. Do not delete or stage it casually.
- CI run 34139824033 tests head `f9ada6f3`. At the last permitted poll it was
  still running its full preflight, with two known failed jobs: native macOS
  curiosity coverage and the real-browser semantic DOM golden. Exact diagnosis
  and grouped prompt IDs are recorded in `HISTORY.md`. Do not merge or bless
  either baseline.
- Expansion and catalog work are paused until the repaired PR run passes.
- The sixth fleet has 24 inputs and 66 frozen checks. It has not run on a CLI
  built from the repaired head. Its contract is `SIXTH-BATCH.md`.
- Private source-derived fleets may finish outside this repository. Their data
  must not enter PR #504.

## Required order

1. Let run 34139824033 settle; poll GitHub no more often than every 300 seconds.
2. Repair the three pure-water freeze/melt failures by keeping independent water
   phase transitions out of a failing aqueous solve. Mixtures must still fail
   explicitly when aqueous state is unresolved.
3. Correct coverage route attribution generically. A successful pure-water setup
   route must not reclassify a later qualitative or typed answer. Classify the 69
   changed rows after that repair; do not regenerate the baseline wholesale.
4. Treat `aq-053` and `aq-085` as unsupported expectations unless a reviewed
   hypochlorite or repeated-extraction model is added. Make precipitation an
   answering event for `mat-086` only if its event trace proves that precipitation
   answered the question despite an unrelated boundary event.
5. Fix production behavior behind the browser differences while preserving
   current main's audited GUI003 goldens. Do not stage generated DOM output.
6. Preserve all 113 entries, every evidence artifact and the untracked lessons
   diagnostic. Exclude temporary `scene_golden.rs` logging and every
   `*.actual.json` file from commits.
7. Run focused regressions, `git diff --check`, count checks and curiosity
   coverage. Push only an internally consistent repair with an explicit lease.
8. Leave PR #504 draft and unmerged. Expansion remains out of scope until its CI
   is green and the integrator authorizes the next tranche.

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

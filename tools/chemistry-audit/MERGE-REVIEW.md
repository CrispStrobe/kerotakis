# Safe integration review, 2026-09-07

## Latest audit state — 2026-09-07 09:38 UTC

This supersedes earlier pending-run checkpoints below. Ownership restrictions
and the minimum five-minute GitHub polling interval remain in force. No
reserved integrator branches/files or shared planning files were edited.

- Clean third-fleet validation now passes all 36 runs and 174 checks, including
  physical-state preservation on refused distillation; native inventory 69/69.
  Evidence: `native-cli-validation-20260907`.
- The next batch was authored and executed: 36 new cases 123–158, all 108
  scientific checks pass, native inventory 49/49 across 197 rows, no alias
  leaks. Evidence and pre-execution hashes: `fourth-batch-1` and its reports.
  No tolerances changed after observing output. CI now includes this fleet;
  actionlint and whitespace checks pass.
- Across four fleets, 158 distinct cases and 446 scientific checks pass at
  their recorded revisions. Original failed and timeout runs remain preserved.
  This is not a guarantee outside documented model domains or a full CI pass.
- CLI repository provenance tests now pass (2/2). Its remaining failure was
  a stale count: three reviewed distributed data slices had been added.
- Five native assertion failures were reviewed and repaired without changing
  production chemistry or numerical tolerances: extended dataset provenance,
  metastable-phase boundary wording, and distinct missing-versus-excluded
  ferric phases. Those three test files still require CI execution. Their
  local build was stopped after sccache shutdown/CPU starvation; only validated
  audit-owned compiler/client/Cargo PIDs were terminated, no artifacts deleted.
- Linux/macOS snapshot artifacts are byte-identical. All differences are
  explained by newer main heat-capacity schema and German metadata; see
  `CI-SNAPSHOT-REVIEW.md`. They must be integrated with their matching source,
  not copied blindly into this older local base.

Remaining: integrator-owned README/catalog counts and source/snapshot sync
(`INTEGRATOR-HANDOFF.md`), then fresh passing combined CI and coordinated safe
merge. Latest inspected completed run is still 34096877766 (failed at older
head 4be55cb6). No additional status polling since approximately 09:10:50 UTC;
subsequent accesses read completed logs/artifacts only. No merge is claimed.
No local build or experiment process remains running at this checkpoint.

Catalog recommendations: a conserved ternary still cut, equal/split-charge
electrolysis, and three-water heat grouping. The last two may best extend
existing lessons. Recommendations are audit-owned notes, not edits to the
parallel integrator's app catalog. Endpoint refinement and reverse equilibrium
were already preserved earlier in this audit.

## Current handoff — 2026-09-07 09:12 UTC

This section supersedes the historical checkpoints below. Work stays in
`/mnt/volume1/kero-experiment-audit`, branch
`audit/chemistry-experiments-20260906`; latest pushed implementation is
`4be55cb6d029b3868e79a2f854fc249b55a22dcc`. PR #504 is not merged.

Coordination boundary: the parallel integrator owns PR #497 GUI-003, KIDS
procedure kits, standing adsorption/partition/osmosis readouts, live
Catalog/StoryMap discovery, main sync, and shared planning-MD reconciliation.
Do not edit/rebase/push those branches or files. Keep audit progress/history
in this directory. No other worktree or process was changed for this handoff.
GitHub status polls must be at least five minutes apart. Last status poll:
2026-09-07 approximately 09:10:50 UTC; next no earlier than 09:16 UTC.
Reading that completed run's failure logs is not a new status poll.

Validated locally: original/second native fleets 86 runs and 164 scientific
checks; native inventory 150/150; selected catalog 10 runs/30 checks; full
catalog 110 entries; web 1,125 tests; phase fallback three tests; reviewed core
lesson snapshot; atomic refusal two tests; native startup two tests; step
prose ten tests. Native startup now fails closed if required PHREEQC cannot
initialize; native experiment outputs already carried PHREEQC provenance.
Core-only snapshots deliberately do not instantiate that native stack.

Third-fleet discovery used one unchanged native binary: 36 runs, 173/174
checks, native inventory 69/69. Its one physical defect (post-refusal solver
mutation) is fixed and has passing focused tests. The rebuilt-binary replay
in `native-cli-refusal-fixed-20260907` completed with 32 successful runs and
four 90-second startup timeouts (91–94): 155 checks passed, ten unmet because
those outputs were missing. Host load was approximately 89 during the issue;
that is evidence of contention, not proof every timeout has that sole cause.
This is not a clean full-fleet validation. Preserve both runs without waiving
checks; retry in a fresh directory under usable execution conditions.

CI run 34096877766 is complete **failure** (22 jobs completed). Failed jobs:
Full preflight, browser demo, presentation adapters, native macOS and Linux.
Failure logs identify repository provenance, codex/registry snapshots, README
counts, German catalog count (110 actual versus 108 expected), withheld-phase
diagnostics, aqueous provenance/routing, and ferric-chloride acidity assertions.
These require review against the exact CI merge source; do not blindly bless
snapshots or change scientific tolerances. Catalog/discovery and shared README
changes may overlap the integrator's ownership: coordinate before touching
those files. Full CI and any necessary current-base integration remain pending;
no merge approval, force push, admin bypass, or claim of all-errors-fixed.

Next audit work: diagnose remaining owned chemistry/provenance failures; review
computed snapshot artifacts; obtain a clean rebuilt CLI fleet; then obtain
fresh passing combined CI and coordinate safe integration. Do not independently
reconcile main or shared planning files. No local builds/replay processes from
this agent remain running at this handoff.

The user requested checking CI, safely merging PR #504, then continuing.
This review integrates main `3860e36f4fc0787946bc017518607d197fe5e7ba`
into audit head `f33faa96ff1f5dcf4aa5d5ca5680e6dff2a934ad` before any
merge of the PR into main. No failing or incomplete gate is a merge approval.

## Conflicts and semantic integration

- The three textual conflicts were `codex/rates.toml`, `codex/i18n/de.toml`,
  and the codex export snapshot. Field-level three-way comparison found no
  overlapping value changes: preserve our finite-acid corrections and two
  lessons alongside main's new entries, translations, and progress metadata.
- Main requires authored `progress` on every entry. Endpoint refinement is
  `intermediate`; reverse mass action is `advanced`. All 110 entries have
  valid metadata. The snapshot is merged by ID/field and ordered by source
  TOML, not regenerated from guessed chemistry outputs.
- Registry/source review retains all eight aqueous-basis additions and
  main's liquid-nitrogen provenance/latent-heat update. The two missing KSCN
  and SCN- registry snapshot records are transcribed from the curated source
  using the actual serializer's field mapping; all 176 identity keys agree.
- Read-only review found no overlapping calculation lost between our solved
  phase readback and main's CEA atmospheric-energy changes, nor between our
  finite-acid/solvent code and main's scene/event updates. This is a bounded
  integration review, not independent validation of every upstream model.

## CI findings repaired before retesting

- Native macOS Clippy: replace an indexed loop with an iterator and move the
  redox-ledger test module after production items. No lint suppression.
- Web catalog labels/template and exact count updated for the merged 110
  entries. About notice and license HTML regenerated from complete locked
  dependency sources; no package-install scripts or application build ran.
- All 20 focused locale/catalog/About tests pass locally after integration.
- The live CLI provenance gate, unlike the shell gate, lacked the two reviewed
  federal public-domain identifiers. Added those exact **data-only** references
  with tests retaining GPL, NC, code-lane and unreviewed-reference refusals.

## Still required

The earlier WebAssembly gate exposed an independent phase-physics regression:
an unavailable aqueous engine prevented pure water from freezing. The fallback
now retains `SolverFailed`, clears stale speciation, and computes water phase
changes only for a verified pure-water inventory. Unknown and ionic mixtures
remain withheld. New tests cover freezing, melting, boiling, conserved mass and
latent energy at three scales, and mixture refusal; execution remains a CI gate.
Workflow concurrency supersedes only revisions of the same PR/ref, never other
worktrees' branches or their runs.

A local full web sweep found one further stale corpus-count assertion; updated
it to 110 codex plus 60 guided entries. Its focused suite passes all 26 tests.
The sweep itself was not clean (1,121 passed, one stale-count failure and one
worker timeout), so full web validation remains required. Pre-execution review
also strengthened third-fleet atomic-refusal physical-state checks and added
electrolysis atom ledgers including external gas exchange without changing
the predeclared numeric tolerances.

The CLI-only native build completed successfully in 8m32s after the user
requested this smaller execution route. The clean single-worker web rerun
passes all 1,125 tests in 94 files. The first native replay passes 50 runs and
43 scientific checks without solver failures; later fleets are still running.
All three pure-water phase-fallback tests pass locally.

Actual core frozen-output execution produced exactly two added lessons and
zero changed or removed existing lessons. Added only those computed records:
the endpoint lesson correctly discloses the core-only harness's absent aqueous
engine; reverse equilibrium computes the same final composition from both
starting directions. The original actual-output file is retained with the
native replay evidence. The updated snapshot test passes locally.
Fresh macOS CI additionally exposed a test-only `explicit_auto_deref` warning
in the native MIX boundary test; removed the unnecessary dereference.

CI now uploads generated `*.actual.json` files even on failure for that review.
The native matrix no longer cancels Linux when macOS fails, allowing both
platforms' independent results and the Linux fleet evidence to be collected.

The combined source requires fresh CI, including workspace tests, catalog
lint, provenance, and the experiment checks. Cases 87–122 remain designed but
not executed at this checkpoint. Recheck main/head identities and the final
CI conclusions immediately before any PR merge; never force an unresolved PR.

## Subsequent native CLI discovery

The CLI replay has now executed cases 87–122: 173/174 checks pass, with the one
atomic-distillation refusal failure retained in `native-cli-20260907`. The
runner now carries an explicit unchanged-operation disposition and skips
post-operation solvers for the refused transfer, preserving its diagnostic.
Both focused refusal/success-control tests pass; rebuilt CLI replay is next.

CI also exposed absent per-step guidance for the two new lessons. English and
German guidance is added, and the ten selected audit guides now reflect finite
acid and documented model limits instead of obsolete yields/precision claims.
All ten step-prose tests pass and all 94 paced scripts/530 sentences validate.

## Native engine startup contract

The CLI experiment outputs contain native PHREEQC provenance; the engine was
not absent from those runs. `frozen_behavior` intentionally uses the engine-free
core bench, now explicitly documented at its runner. A separate real startup
gap existed: CLI initialization failure printed a warning and substituted a
reduced stack. Native bench startup now fails with an actionable error and
nonzero status instead. Optional explanation-path comparison warns separately;
its failure does not misrepresent the initialized primary engine as missing.
Constructor-failure and successful native-provenance tests are added; their
execution is pending at this edit. This does not invent missing chemical data
or expand any solver's documented model domain.

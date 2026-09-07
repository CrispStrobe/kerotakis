# Safe integration review, 2026-09-07

## Implementation checkpoint — 2026-09-07 11:33 UTC

The generic curated-reaction diagnostic fix is implemented: model refusal,
nonfinite result, absent/depleted forward-and-reverse capacity, and supported
near-zero equilibrium extent are distinct. A valid equilibrium no-op emits
the existing OrgReacted event with extent zero and an unchanged-inventory
boundary. No experimental ID dispatch or numerical chemistry change was added.
Six focused helper/integration regressions cover these distinctions, reverse
reaction, and repeated requests over multiple scales. They are NOT yet executed:
the local compile was stopped under severe host starvation, exact owned Cargo
and rustc PIDs verified gone. Formatting/diff checks pass; compiled verification
and the new CLI are delegated to CI, not represented by the old binary's results.

Catalog: three original entries, EN/DE prose and German title labels are added.
Native replay of 13 selected entries passes 37 checks; 199 focused web tests pass.
Actual native export was reviewed: exactly three added entries, no other semantic
change. The existing cached engine-free snapshot executable computed the three
new lesson records; all existing records were unchanged. Only those additions
were accepted. This is not verification of the new diagnostic implementation.
Fresh source-built snapshot checks remain required in CI.

Fifth native inventory cross-check also passes 213/213 comparisons over 226 rows.
Fifth regression fleet is now wired into CI. Sixth fleet is frozen (24 cases,
66 checks), missing-output negative control passes, and CI will run it on the
rebuilt CLI. No sixth scientific pass is claimed before execution. Inputs and
failure evidence are retained by the existing artifact workflow.

Preservation decision: keep all fifth controls in the regression corpus. Strong
next app candidates are diprotic remaining-equivalent capacity and hydroxide-
limited zinc precipitation/acid reversal. Unequal silver feeds, equal charge
and repeated equilibrium overlap existing lessons and should not become duplicate
catalog cards. See SIXTH-BATCH for rationale. Source research stays private.

Ownership remains audit branch / PR #504. No merge is claimed. Last GitHub status
read approximately 11:31 UTC: OPEN, mergeability UNKNOWN, head a375f29d; do not
poll again before 11:36 UTC. Integrator owns reserved branches and main sync.

## Fifth fleet complete — 2026-09-07 11:08 UTC

All 36 cases completed with exit zero and zero solver_failed events. The frozen
analyser reports 119/119 checks passing: 36 execution, 42 conservation and 41
model-law/control checks. Root read the actual report and confirmed no failed
checks. Evidence is `fifth-batch-1/summary.json` and `law-checks.json`; no bounds
were changed. Total per-case elapsed time 477.525 s, median 12.67 s, range
6.376–25.051 s. This supersedes the in-flight checkpoint below.

One diagnostic correctness gap remains despite passing numerical checks:
case 192 is already at ester equilibrium, yet the repeated request says it
needs both reactants. Both remain present (acid 0.010276049844845335 mol,
ethanol 0.002276049844845333 mol); bench.rs uses the same message for near-zero
extent. The state/idempotence checks pass; the explanation is misleading.
Organic-rich mixtures also explicitly withhold aqueous ionic speciation;
their ideal ester result does not validate missing nonaqueous activity models.

Timing diagnosis: recorder starts a fresh CLI per case. Each run eagerly builds
both primary and optional explanation aqueous engines, each loading three
databases and obtaining neutralisation data. Host has four CPUs and load ~28.
A separate no-operation startup probe measured 3.98 s wall versus 0.73 s CPU;
Three simple operations measured 13.70 s wall / 1.90 s CPU; thirty operations
in one timing-only shared session measured 10.08 s wall / 2.18 s CPU. These
single samples demonstrate noisy wall-time contention, not a reliable speedup
ratio. The existing MCP reset also reconstructs the engine session, so it does
not yet provide isolated engine reuse. No runtime change
or safe merge has been made by this diagnostic work. Catalog verification,
diagnostic repair, source-backed extensions and combined CI remain pending.

## Execution underway — 2026-09-07 11:02 UTC

The frozen fifth fleet is now running through the existing native CLI (recorder
PID 4032401); 15 of 36 cases have completed with exit zero at this checkpoint.
Scientific pass counts await the unchanged 119-check analyser. Preflight input,
analyser, protocol and binary hashes are in `fifth-batch-1/preflight.json`;
raw outputs are retained. The assigned execution agent owns completion monitoring
and analysis, with no rebuild or tolerance changes authorized.

Separately, three private original analogue runs completed with all 12 declared
checks passing and no solver failures. Root independently rechecked their raw
scientific/control outputs. Exact inputs, source relationships and results stay
in private research storage. These narrow successes do not validate the original
source apparatus/endpoints or fix the previously identified model limitations.
Catalog integration and combined CI/merge remain pending.

## Intermediate review — 2026-09-07 10:54 UTC

Root inspected private intermediate files and checked their schema/gap
references, then resumed idle reviewers with explicit remaining coverage and
machine-readable variant/check tasks. This establishes research progress only:
no new source-inspired experiment has been executed or scientifically validated.
Exact findings remain private as instructed. No reserved KIDS/integration files
were changed. Catalog replay/export and the frozen fifth-fleet execution remain
pending; no new chemistry fix, passing combined CI or merge is claimed.

## Latest checkpoint — 2026-09-07 10:38 UTC

Ownership remains audit branch `audit/chemistry-experiments-20260906`, PR #504;
latest pushed implementation is `a375f29d`. No merge or new combined CI pass
is claimed. All reserved integration branches/files remain untouched.

Three independently authored catalog additions are in progress locally, with
German translations and step guidance. Agent-reported schema/prose checks pass;
root runtime replay, generated exports/snapshots and final integration checks
remain required. Fifth fleet inputs are frozen: 36 new cases, 119 predeclared
checks, with a missing-output negative control. It has not yet been executed.

Source research is intentionally outside the public repository, now backed by
a separately verified private repository. Exact source inventories, source-to-
model gaps, URLs, and archival assets must stay private; this public checkpoint
records only progress. Research assignments cover the supplied manuals and a
new library triage; educational index review remains with root. Assignment is
not completed coverage. No source-derived runtime data or copied manual text
has been added to the app. Reuse and scientific validation remain separate gates.

Next: finish/replay the three catalog entries, execute the frozen fifth fleet,
continue private source review, reconcile combined CI with the integrator, and
merge only after validation. GitHub status polling remains >=300 seconds.

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

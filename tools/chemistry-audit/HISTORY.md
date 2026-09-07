# Chemistry audit history and continuation

## 2026-09-07 09:38 UTC — clean replay and fourth fleet

The third fleet now passes all 174 checks across 36 runs, and 69 native
inventory comparisons, on the repaired CLI. The earlier timeout run remains
preserved; no assertion was waived. A fourth independently designed fleet
(123–158) then completed with 108/108 scientific checks and 49/49 native
inventory comparisons. Four fleets now cover 158 distinct cases with 446
passing scientific checks at their recorded revisions.

Reviewed stale CI assertions were updated without altering numerical chemistry
criteria. CLI provenance integration tests pass locally; three repaired native
test files await CI after a resource-starved local build was stopped safely.
The fourth fleet is wired into CI and its generator/analyzer/model limits were
hashed before interpreting output. See `FOURTH-BATCH.md` and `fourth-batch-1`.

Current cross-branch failures include integrator-owned counts and source/schema
snapshot synchronization, documented in `INTEGRATOR-HANDOFF.md` and
`CI-SNAPSHOT-REVIEW.md`. Reserved files remain untouched. PR #504 is unmerged;
fresh combined CI is still required. Catalog candidates are recommendations,
not app changes: ternary cuts, equal/split charge, and three-water heat grouping.

## 2026-09-07 — native CLI and integration checkpoint

Audit branch only: `audit/chemistry-experiments-20260906`; PR #504 remains
unmerged. See `MERGE-REVIEW.md` for the current ownership boundary and CI
poll timestamp; do not edit the parallel integrator's branches or planning.

The native CLI was built independently of browser/mobile targets. Earlier
fleets pass 86 runs/164 scientific checks, and the selected catalog passes
10 runs/30 checks. Native PHREEQC provenance is present. Engine-free core
snapshots test a different, deliberately reduced harness.

The next designed 36 experiments ran with 173/174 checks passing. They exposed
a genuine atomic-refusal bug: declined distillation still invoked solvers and
changed the source state. A typed unchanged-operation disposition repairs this;
two focused refusal/success-control tests pass. Original failed evidence stays
in `native-cli-20260907`.

The rebuilt CLI replay suffered four startup timeouts amid severe shared-host
contention. Its complete evidence is in `native-cli-refusal-fixed-20260907`;
32 runs completed, 155 checks passed, ten lacked required outputs. Do not call
this a clean replay or remove the missing-output failures.

Native CLI initialization no longer silently substitutes a reduced stack on
PHREEQC failure. Constructor-failure and successful native-provenance tests
both pass. Optional explanation-engine failures are diagnosed separately.
New lesson guidance is present in English/German, and obsolete precision/yield
claims in the selected repaired guides were reconciled to model boundaries.

Continuation plan: resolve current CI chemistry/provenance failures, coordinate
overlapping catalog/readme/snapshot integration, rerun the fixed fleet cleanly,
and require fresh combined CI before merge. CI run 34096877766 failed; no full
pass is claimed. New imports retain the stated permissive-data restrictions;
the repository's existing AGPL license has not been changed.

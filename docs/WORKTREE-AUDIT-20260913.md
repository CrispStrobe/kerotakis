# Worktree cleanup audit — 2026-09-13

This is a deletion gate, not a deletion instruction. The 19 worktrees listed
in the 2026-09-07 triage were checked again against `main` at `b9d6d45e`.

All 19 are clean, have no process whose current directory is inside the
worktree, and have no non-build file modified in the preceding 24 hours.
Nevertheless, **none is approved for automatic deletion**: every branch still
has at least one patch that `git cherry main <branch>` considers unique. Main
absorbed most of this work through re-authored pull requests, so neither commit
ancestry nor patch identity alone proves that the remaining bytes are
dispensable.

| Worktree group | Count | Commits ahead of main | Patch-unique commits |
|---|---:|---:|---:|
| corrosion, corrosion scene, crystals, gel scene, guided14, household, invisible ink, kids catalog, kids connected, mechanisms A, persistent readouts, stage2 A, stage2c, translations | 14 | 14 | 14 |
| learning progress | 1 | 2 | 2 |
| mechanisms B | 1 | 7 | 3 |
| mechanisms C | 1 | 2 | 1 |
| stage5 accessibility | 1 | 2 | 2 |
| visual journey tests | 1 | 5 | 5 |

Before any deletion, compare each unique patch by intent and behavior with the
named merged PR, preserve any evidence or test absent from main, and repeat the
dirty/process/recent-write checks immediately before removal. Until that
semantic review is complete, retaining these directories is the safe result.

The repository currently registers 75 worktrees in total. This audit is
deliberately limited to the 19 previously nominated candidates; it makes no
claim that any of the other 56 are obsolete.

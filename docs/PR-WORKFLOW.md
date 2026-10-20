# PR delivery: auto-merge by default

The user authorized continuing implementation, fixes, commits, pushes and merges,
and explicitly requested auto-merging future PRs and completion of all PRs in
this workstream. No additional confirmation is required for that scope.

1. Start a narrow branch from actual integrated main. Keep unrelated audit and
   infrastructure history out of implementation/documentation PRs.
2. Use draft status while source review or acceptance outside branch protection
   remains incomplete. Record concrete blockers. Product changes require their
   focused controls and applicable full own fleet, not only the five protected CI
   contexts. Do not cancel active source-bound runs to make optional changes.
3. Once review-ready and non-protected acceptance is complete, mark ready and
   enable normal auto-merge with the reviewed head:

   ```console
   gh pr ready NUMBER
   gh pr merge NUMBER --auto --merge --match-head-commit REVIEWED_HEAD
   ```

   Protected checks must still gate integration. Never use admin bypass, force
   push, waived checks or weakened tests to obtain a merge. If checks are already
   green, auto-merge may merge immediately.
4. Documentation-only PRs can enable auto-merge after scope/link/consistency
   review; required CI still runs. A new chemistry fleet is not required when
   there are no runtime, workflow, harness, data or scientific-contract changes.
5. Follow failures through diagnosis and repair. Preserve first logs/results;
   review the new head and its own checks before re-enabling auto-merge if GitHub
   removes it. Do not leave accepted work as an unmerged draft.
6. Before integration, refresh head/base/main, required contexts and applicable
   rules. Verify the actual merged parents/tree against the reviewed candidate
   afterward. Record the merge and update current documentation.

PR787 was the only open PR at the 2026-10-15 checkpoint. It merged through this
auto-merge command after 25 CI and 33 fleet jobs passed. Public auto-merge is
already enabled in repository settings; required branch protection remains intact.
Sign commits as required by [CONTRIBUTING](../CONTRIBUTING.md).

## Batch related delivery work

Prepare a cohesive implementation and its maintained status/handoff updates together before opening a PR. Fold related ready work into that PR while its acceptance head is still being prepared. Avoid a standalone documentation PR for every intermediate checkpoint. Freeze source-specific evidence separately and reference its immutable public audit commit; never merge broad audit history.

Preserve an already reviewed head with active or completed acceptance. Optional bundling should not restart its validation; include later summaries with the next substantive change. Independent chemistry repairs still require their own frozen source/forecast/first-failure evidence even when delivery is batched.

CI optimization starts with an explicit eight-file maintained-summary fast path. Unknown paths, root README/CONTRIBUTING (consumed by tests), scientific data, runtime, workflow and contract changes retain FULL. Classification errors must fail the stable required gate. Preserve exact required platform context names and unchanged full-runtime commands. Full own-fleet and actual delivery remain mandatory where applicable; a docs-only skip is not source/product acceptance.

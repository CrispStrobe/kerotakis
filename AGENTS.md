# Kerotakis agent instructions

Read [current state](docs/STATUS.md), the [ordered agenda](docs/improvement-agenda-20261014.md)
and [PR delivery policy](docs/PR-WORKFLOW.md) before choosing work.
Use [fresh-agent task scopes](docs/next-agent-handoff.md) for current prerequisites
and acceptance. Read ignored `CLAUDE.local.md` when available; never
commit its contents. Explicit user
instructions take priority. Private host details belong only in the private
operating handoff; keep public documentation to repository/public paths.

- Complete authorized work, including validation, commits, pushes and integration.
  Default to auto-merge for every review-ready PR using an exact reviewed head.
  Keep a PR draft only while named review/non-protected acceptance work remains.
  Never leave fully accepted work waiting for another user confirmation.
- Preserve branch protection and all applicable acceptance, including product
  own-fleet checks outside protected contexts. Never use admin bypass, force
  pushes or weakened tests. Documentation-only PRs follow the narrower policy.
- Use narrow branches from actual main. Never merge broad audit or isolated
  infrastructure branches. Sign commits under CONTRIBUTING.
- Preserve original forecasts, first failures, raw evidence and exact source,
  binary and lock bindings. Freeze follow-ups separately. Unsupported chemistry
  is a coverage limit, not a scientific pass. No unearned calibration claims.
- Execute the approved lanes in order. Refresh current source and prerequisites;
  older dated documents preserve history and can contain obsolete pending notes.
- Check load, available memory and disk before large work or additional agents.
  Respect private resource thresholds and protected worktree/evidence inventories.
  Use hosted bounded validation when local gates are closed. Avoid duplicate jobs
  or executions and do not cancel active runs for optional changes.
- Update maintained Markdown summaries after meaningful progress. Keep frozen
  experiment documents, historical evidence and third-party material immutable.

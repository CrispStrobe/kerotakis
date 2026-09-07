# Chemistry audit

This directory contains reproducible chemistry probes, analyzers and preserved
evidence. Production chemistry must derive outcomes from general models; it must
never dispatch on an audit case or embed an expected result.

## Read order

1. `CONTINUATION.md` — current state, rules and agent-ready tasks.
2. `INTEGRATOR-HANDOFF.md` — PR #504 ownership and merge boundary.
3. `SIXTH-BATCH.md` — frozen contract for the only pending public fleet.
4. `HISTORY.md` — completed work and superseded decisions.

No other Markdown file is required. Raw evidence directories are immutable;
their JSON, NDJSON, scripts, stderr and hash manifests are the authority.

## Non-negotiable rules

- Work only in `/mnt/volume1/kero-experiment-audit` on
  `audit/chemistry-experiments-20260906` unless the user assigns another scope.
- Do not touch the integrator's reserved GUI, KIDS, readout, discovery or main-
  synchronization branches and files. Do not merge PR #504.
- Preserve failed runs and generated diagnostic artifacts. Never overwrite an
  evidence directory; use a new directory for a replay.
- Freeze checks and tolerances before execution. Missing output, solver failure,
  unsupported behavior and implausible output remain visible; they are not passes.
- Check conservation, model laws and matched controls. Process exit zero alone
  is not scientific validation.
- Keep source research, URLs, OCR and source-to-gap inventories private. Public
  app prose and scripts must be independently authored.
- Add no GPL, LGPL, non-commercial or otherwise incompatible dependency or data.
  A public document is not permission to reuse its text, figures or constants.
- Use `apply_patch` for edits. Preserve unrelated and untracked user files.
- GitHub status reads must be at least 300 seconds apart. Record the UTC time.

Completed results and rationale belong only in `HISTORY.md`. Active documents
contain only facts needed to make the next correct decision.

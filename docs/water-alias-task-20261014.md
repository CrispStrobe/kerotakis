# Canonical water input alias — completed implementation lane

PR787 merged on 2026-10-15 as `4d3203a3`, from candidate `d5c6cecf` on
`8a6aa68e`. All 25 CI and 33 own-fleet jobs passed; the app suite passed 1,638
of 1,638 tests across 123 files. Both native platforms passed the nine alias
regressions, workspace/CODEX/coverage with zero unmet requirements, runtime
failures or drift. The actual merge parents/tree match the validated candidate.

[Integration evidence](https://github.com/CrispStrobe/kerotakis/blob/audit/systematic-chemistry-20261002/audits/fresh-syntax50-20261013/water-alias-integration.json)
and [final acceptance](https://github.com/CrispStrobe/kerotakis/blob/audit/systematic-chemistry-20261002/audits/fresh-syntax50-20261013/water-alias-final-acceptance.json)
retain actual source, jobs and log hashes. Current runtime checksum/lock members
were read with CRC/hash verification; the full current ZIP was not downloaded
locally. Its own fleet executed against its built CLI, whose archived checksum
matches the earlier fully byte-verified executable.

## Implemented contract

- Exact `H2O` input resolves to canonical `water`; this is one identity, not a
  second registry species or a broader formula-normalization feature.
- Text, direct/JSON operators, vessel APIs, finite stock, logs and snapshot
  identities agree. Operators normalize before safety assessment and dispatch.
- Both stock restore formats normalize a single alias without changing its
  authoritative amount, unit or compensated components. A restore containing
  both alias and canonical bottle keys refuses instead of overwriting one.
- Inventory/provenance lots preserve quantities through restore and transfer.
- Story access, mission kits and usage use engine-parsed identities. There is no
  second product alias table in the frontend.
- Story usage counts a labelled accepted dispense, not invented mass conversions.
  Addition/material-addition or positive-volume titration charges once; veto,
  exhaustion and zero-dose endpoints remain uncharged.
- The pure dispense predicate belongs in `storyStock.ts`. The visualization
  coverage guard and scene-drawn classifications remain unchanged.

The implementation spans seven core files and four app files. The original nine
native and eight new app regressions remain the regression contract. Preserve
the first app guard failures, the invalid-command CLI failures, and separately
frozen corrected CLI passes; do not rewrite their original outcomes.

## What follows

This lane is complete. Do not repeat its merge or rerun unchanged controls.
Proceed to [the forty naming follow-ups](improvement-agenda-20261014.md) and
[the 43-case closure plan](closure43-plan.md). The original blind fifty and
old-source normalized forty remain immutable. Future product changes get narrow
PRs with [auto-merge](PR-WORKFLOW.md), fresh applicable acceptance and exact-head
guards. Closed combustion and empirical calibration were not added by this fix.

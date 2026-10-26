# Deployment snapshot lifecycle: scoped follow-up

The [first measured quota failure](../audits/deployment-snapshot-quota-first-failure-20261026/README.md) exposes a resource-lifecycle gap: old deployment removal can leave native execution snapshots behind. Immediate approved cleanup preserves all retained deployments and their rollback snapshots. Do not solve this by disabling isolation, shortening the lifetime of a snapshot used by a retained release, or silently upgrading billing.

## Safe inventory and explicit cleanup

Add a bounded, dry-run-first maintenance tool for the existing deployment project. Read complete paginated snapshot, retained-deployment and sandbox inventories; fail closed on an incomplete page, ambiguous ownership, unknown snapshot binding or active deployment. Resolve each retained deployment to its exact native snapshot using its authenticated build receipt or immutable deployment metadata. Names, age and lack of a production alias alone are insufficient evidence of non-use. Preserve every retained preview/rollback deployment, sandbox current/source snapshot and required parent dependency. Keep provider credentials in the parent environment and out of logs, arguments, snapshots and evidence.

The plan must enumerate exact candidate identities, project ownership, size, protection reason and raw inventory hashes. Require an explicit apply action against that reviewed plan. Recheck inventories before mutation, record each deletion intent before issuing it, and verify each result. An ambiguous network result must stop without automatic replay. Inventory changes invalidate the plan. An unrelated project must never become a candidate through a missing project filter. Use provider-confirmed deletion/readback, not local file removal, as completion evidence.

## Deployment evidence and acceptance

Preserve native build/source/actual producer-lock evidence even when snapshot preparation or later deployment fails. Keep a partial build artifact visibly incomplete; it cannot substitute for an accepted release with sandbox and isolation receipts. Bind retry results to the actual workflow attempt, preserving original first-failure logs and metadata. A known failure before any chemistry request permits one explicit reviewed infrastructure retry; an ambiguous issued experiment does not.

Acceptance must exercise complete/incomplete pagination, foreign project IDs, a retained deployment without an alias, preview/rollback references, sandbox references, unknown binding, concurrent deployment, changed inventory, ambiguous deletion response and once-only retry guards. Include a real dry-run against provider metadata. Do not use an asserted mock result as evidence of actual cleanup or deployment. No chemistry, policy, physical budget, isolation or experiment assertion may change.

This operational follow-up is separate from the ordered delivery → unchanged originals → owned diagnostic-localization work. Batch its maintained documentation with the next substantive delivery; never merge broad audit history or restart a valid acceptance head for optional edits.

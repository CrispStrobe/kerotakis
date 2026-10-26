# Deployment quota first failure and approved cleanup

Accepted source `c51758fa09a8ce0089bc93bb64338a587834a405` remains unchanged. [Production workflow attempt1](https://github.com/CrispStrobe/kerotakis/actions/runs/40284030952/attempts/1) builds both the web payload and native engine, then fails at snapshot creation with provider HTTP402: snapshot storage quota exhausted. Isolation verification, artifact publication and deployment do not complete. No API or browser controls were issued. This is an infrastructure failure, not a chemistry failure or product pass.

A complete inventory of the one accessible Vercel team covers39 projects. Only this project has snapshots. Its three retained deployments are each bound to their snapshot through retained actual deployment logs; there are no remaining sandboxes. After explicit user approval, exactly17 unreferenced snapshots are deleted, removing14,125,880,596 bytes (13.16GiB). Fresh provider readback preserves all three current/rollback snapshots. No deployment, live snapshot or unrelated project resource is deleted.

The [machine-readable receipt](FIRST-FAILURE-AND-CLEANUP.json) preserves the actual first attempt and cleanup evidence hashes. A separately reviewed attempt2 must use the same accepted source and workflow run, verify no active deployment or preexisting artifact, and preserve attempt1. Do not redispatch a new source, replay a possibly issued chemistry request, relax sandbox isolation, or count successful builds as release acceptance.

Actual artifact/native/producer-lock/frontend binding and15API/86browser checks remain required. Unchanged-original qualification and owned diagnostic EN/DE/FR implementation/delivery follow in their existing order.338/468 originals remains unchanged. See the [snapshot lifecycle scope](../../docs/deployment-snapshot-lifecycle.md) for preventing recurrence.

The [reviewed attempt2](RETRY-ISSUED.json) is now issued on the same workflow/source. It has not earned deployed acceptance; the initial quota failure remains separately frozen.

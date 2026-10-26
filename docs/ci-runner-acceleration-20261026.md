# CI runner acceleration: measured next-agent lanes

Keep the active reviewed diagnostic head stable. Implement these changes only in a separate narrow branch from actual integrated main after its delivery. Read STATUS and PR-WORKFLOW first. [Historical accepted-run timings](../audits/ci-runner-timing-20261026/README.md) show duplicate native work and many very short fleet jobs; the current queue also has unassigned runners. Workflow changes cannot guarantee immediate runner availability.

## Lane1: measure and compose equivalent native acceptance

Inputs are `.github/workflows/ci.yml`, `tools/preflight.sh`, `tools/ci_changes.py` and its tests, plus the actual historical timing envelope. Full preflight takes24.3minutes; native macOS23.0minutes and Ubuntu16.8minutes. Identify the script's internal step boundaries before claiming which share is redundant. Both native jobs already run full workspace tests, Codex and curiosity gates; preflight also runs workspace tests. Its third Clippy copy is already explicitly removed in CI, so do not propose that as new work.

Retain local preflight's complete default behavior. If adding a CI-only mode that omits a genuinely duplicated command, make the stable `Full preflight gate` explicitly depend on the matching successful native matrix result, under the same exact tested source/tree and command configuration. All independent preflight checks must still execute. A missing, skipped, cancelled or failed prerequisite must fail the stable gate. Preserve Ubuntu/macOS coverage, the full no-fail-fast workspace command, cache-only tests, Codex,500 curiosity requirements and every other existing acceptance. Do not infer equivalence across different features or toolchains.

Acceptance: deterministic workflow/gate controls exercise docs/full/unknown classifications, successful composition, missing outputs, both-platform failure, skipped/cancelled dependencies and command/config drift. Execute actual full source checks on a representative runtime candidate and verify the required context names/app bindings remain unchanged. Compare actual step/queue timings afterward; report observations rather than an invented speedup.

## Lane2: shard fleet setup without losing logical checks

Start with `.github/workflows/chemistry-audit.yml` and its frozen harnesses/analyzers under `tools/chemistry-audit`. Inventory the actual33 job identities,29 family groups and142 authored commands from current source before editing. Existing code already builds the CLI once. The remaining32 historical jobs total352 seconds of job wall, while one nine-second family waits153 seconds. A small number of shards may reduce repeated checkout/download/setup and runner scheduling.

Freeze a manifest assigning every existing logical group and authored command exactly once. Try a small explicit shard count; keep the groups' existing commands, arguments, fixtures, analyzer laws, source/binary/actual-lock bindings, timeouts and isolated evidence directories. Each group's raw stdout/stderr and exit status must survive. A failure must mark the group and aggregate red while still allowing the independent groups that currently run in parallel to produce their evidence. Preserve always-run artifact publication and distinguish an unexecuted group from a pass. No continue-on-error, missing-output success, skipped law or reconstructed raw experiment is acceptable.

Acceptance: a source comparison proves complete exactly-once group/command coverage; negative controls inject an early/middle/late command failure, missing artifact, corrupt binding, timeout and analyzer rejection. All groups must retain their own outcomes and evidence even inside a shared job. Update any delivery consumer that assumes33 physical jobs to validate the unchanged logical manifest and142 commands instead. Run the real unchanged fleets once on the candidate and compare their logical outcomes to the accepted baseline. Preserve all first failures.

## Lane3: evaluate shared build artifacts only with exact consumer contracts

Inspect the bench-Wasm, bridge, IPhreeqc-Wasm and real-demo jobs before attempting reuse. A cache-only bench and a live aqueous-engine demo can require different features/assets; similar names are not build equivalence. Reuse requires identical source/tree, producer lock, toolchain, features, compiler flags, generated registry/cache inputs, JS exports and artifact manifest for every consumer. Keep the protected real-browser semantic/DOM checks and downstream integration tests. Unknown identity or a missing artifact must rebuild or fail visibly. No cache hit may substitute for executing an acceptance check.

Acceptance: actual source-bound producer/consumer manifests, wrong-feature/wrong-lock/missing-file negative controls and the unchanged browser/bridge suites. Measure cold and warm paths separately. Avoid adding this lane to a valid candidate with active acceptance just to save a future run.

## Queue and delivery boundary

The existing eight-file maintained-summary fast path is intentionally narrow; retain full CI on unknown, runtime, workflow, scientific-contract, root README and CONTRIBUTING changes. Do not weaken classification or protection to make the current queue disappear. Do not cancel valid pending/active runs for these optional changes, create duplicate executions, register an unsupervised worker with production credentials, or alter billing/concurrency settings implicitly. Keep any host/account details in the ignored private handoff. Finish every lane through normal reviewed-head auto-merge with actual applicable source and own-fleet acceptance.

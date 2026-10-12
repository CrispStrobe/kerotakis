# Chemistry work agenda — 2026-10-12

The priority is integrating the demonstrated mandatory fallback applicability repair.
Accepted main remains `093db6c9`. Original Rust controls yield6 passes/20 failures;
original generator cases yield6 passes/18 failures. The first repair at `62c70f63`
is independently verified **50/50**, including a newly compiled generator.
PR781 now has a production test assertion correction at `7cce5b8f` after macOS
Clippy failed before native tests. Its own validation and integration remain open.
[Evidence and exact resume instructions](../audits/registry-loader-applicability-20261012/README.md)
are the source of truth.

## 1. Validate the revised repair head

Collect paired26 Rust +24 unchanged generator expectations on `7cce5b8f`, retaining
actual source, lock, compiler artifact, raw outputs and exits. The production
ports change only three runtime `.err().expect()` calls to `expect_err()`;
original frozen fixtures and first failures stay immutable. Verify the exact
normalization, not byte identity with the original ports. Do not borrow the
previous head's50/50 result for the revised head.

Use hosted runners while shared-host load, memory or disk headroom prohibit
local builds. Keep large local work resource-gated and use one Cargo worker.
The original baseline is already captured; no redundant baseline rerun is needed.

## 2. Complete own integration gates and merge the narrow repair

Verify the revised head's own complete CI, both native inventories with404 focused
functions across37 fixtures, full affected libraries and registry admission,
and all33 fleet jobs with every declared step. Derive actual inventories and
fixture hashes from this head; preserve prior378 focused controls. Prepare a
fresh read-only integration preflight, advance the draft only when all other
gates are ready, then use an ordinary head-guarded merge and verify actual
parents/tree. Refresh portable acceptance receipts, ledger and current-main
presence inventory. Never merge the broad audit branch as implementation.

Done means this selected context-free mandatory fallback slice is integrated.
Fifty source-informed engineering controls do not establish empirical chemical
accuracy. Optional curves, unknown/malformed conditions, contextful intervals and
multiple mass anchors remain separate policies.

## 3. Extend the next reviewed final-safety rule tranche

Follow [SAF-01](chemistry-audit-next-lanes.md#saf-01--integrate-one-production-final-safety-exposure-slice).
Choose one already reviewed rule outside the accepted final-rule slices. Freeze
positive, unchanged, benign, distinct-pair, priority and discarded-refinement
controls before changing behavior. Establish actual surviving species, phase
and backend capability; add JSON/text witnesses for supported routes. Keep
intermediate products as a separately defined observation contract. Do not
invent hazards or infer support from an alias-only test.

## 4. Restore historical regression bindings in coherent groups

Follow [AUD-00](chemistry-audit-next-lanes.md#aud-00--integrate-the-accepted-audit-branch-without-losing-evidence)
and the [current-main presence inventory](../audits/integration-20261008/full-audit-port-inventory-pr780-merged.json).
There are 50 missing named tests and 129 missing bound paths, which are coverage
and integration gaps rather than 179 demonstrated defects. Map one prerequisite
complete group to current main, preserve its original controls and four legacy
exclusions, and submit a narrow independently validated port. Refresh the
inventory after integration; only then schedule source-bound full-audit replay.

## 5. Measure one performance cost after correctness is stable

Follow [PERF-01](chemistry-audit-next-lanes.md#perf-01--measure-release-behavior-then-optimize-one-verified-hot-path).
Measure release exposure-scan/private-trial clone cost on an accepted source,
with representative workloads and correctness checks. Optimize only a measured
hot path and compare paired runs. Defer large local profiling or cache/worktree
consolidation until resource gates permit it; retain evidence and protected
worktree contents. Reviewed scientific datasets and authoritative inventory
ownership remain substantive followups, rather than guessed constants or broad
unverified numerical changes.

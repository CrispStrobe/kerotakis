# Chemistry work agenda — 2026-10-12

The primary completion target is the demonstrated mandatory fallback
applicability gap. Accepted main is `093db6c9`; the preserved generator baseline
has **6 passing and 18 failing cases**. The proposed repair is reviewed but
unapplied and uncompiled. [Evidence and exact resume instructions](../audits/registry-loader-applicability-20261012/README.md)
remain the source of truth.

## 1. Capture the original Rust baseline without altering its source

Run the dedicated `registry-applicability-baseline` selection of
`.github/workflows/chemistry-audit.yml`. It checks out exact main093 separately
from the audit collector and injects the two byte-identical frozen fixtures.
Keep the actual dependency lock, full tracked-source hashes, submodule identities,
commands, both raw test blocks and exit status. Independently review the 24+2
exact test identities; dependency or compilation failure is not a contract
failure. Successful evidence collection is not repair acceptance.

Use hosted runners when the shared host lacks load, memory or disk headroom.
Keep local build starts resource-gated and use one worker. The broad audit branch
supplies fixtures and tooling; do not merge it as an implementation change.

## 2. Apply the narrow repair and close its complete frozen slice

After reviewing original Rust outcomes, start a main-based source branch and
apply `candidate-applicability.patch`. Port the two unchanged fixtures. Preserve
uniqueness and phase selection before checking only the three selected mandatory
records. Keep conditional typed documents, complete pack metadata, supported
null/default fields, notes and unused foreign-phase records valid.

Collect paired results for all 26 Rust functions and all 24 generator cases;
the candidate generator must be newly compiled and source-bound. Preserve the
original failures. Verify the prior 378 focused functions, making 404 functions
across 37 fixtures with the new ports, plus complete affected libraries and the
separate registry admission suite. Derive these inventories from the actual
source again before acceptance. Require the repair's own CI, both native
inventories, full chemistry fleet and actual guarded merge parents/tree.

Done means the scoped gap is integrated with its evidence. Fifty engineering
controls do not establish empirical chemical accuracy or broader applicability
support. Optional curves, unknown/malformed conditions, contextful interval
selection and multiple mass anchors remain separate policies.

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

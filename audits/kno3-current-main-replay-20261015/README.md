# Source-pinned KNO3 observation replay

This isolated audit branch prepares a dispatch-only Linux release build of
product source `d791d3062886e4a06dbc7c9d1635383da3e05359`, followed by **only**
experiment 15 from the frozen forty spelling-normalized controls. Do not merge
this infrastructure branch. Preparation is not execution or scientific acceptance.

`design/README.md`, `design/expectations.json` and `design/scripts/15.lab` are
byte-identical copies of the original frozen design. The full expectations file
retains its original forty-case scope; this collector selects only ID 15.
`design-binding.json` records those hashes and the original freeze hash. No
forecast or original result is replaced. The original near-20 °C all-dissolved
KNO3 failure remains evidence independently of any new outcome.

The audit checkout is bound to the dispatched revision, while the separate
product checkout is pinned to the SHA above. The hosted collector records clean
source/tree/submodule identities before and after each child process, actual
tool versions and hashes, explicit build environment, resource gates, build
arguments and verbose logs. Heavy stages require 4,736 MiB available memory
and 8,192 MiB free disk; light stages require 3,400 MiB available memory. The
checksum-pinned SUNDIALS fetch precedes the locked build. No build cache is restored. Because this product
revision has no tracked Cargo.lock, it resolves dependencies once, preserves the
actual lock, and builds with `--locked`; this does not claim dependency identity
with the earlier frozen binary. Native logs/environment are witnesses, not a
claim that every native translation unit was independently audited.

Dispatch the existing recognized `chemistry-audit.yml` workflow on this isolated
branch with `lane=kno3-current-main-replay`. Its branch-local caller invokes the
new reusable workflow at the same commit. Every existing fleet job excludes
this selection, including the three `always()` aggregates. The lane has a
separate concurrency group. Default-main workflows are unchanged.
`routing-witness.json` binds the actual dispatched/audit/workflow revisions and
SHA-256 of both caller and callee. `audit-inputs/` preserves both workflows,
producer, design binding and copied frozen inputs for inspection. GitHub's
actual run/artifact association still needs independent verification afterward.

The resulting binary and its hash are preserved with the lock. The observation
uses `kero run scripts/15.lab --json` in the copied design directory. Raw stdout,
stderr, status, timeout and elapsed time are retained. Process failure and closed
resource gates stop collection and preserve an error. The upload step runs on
failure as well. Manual review must bind the actual run/artifact and inspect
temperatures, aqueous/solid inventories and unchanged forecast before assigning
a physical verdict. Exit zero and a completion marker confer no scientific pass.

No push, dispatch, build or observation was performed during preparation.

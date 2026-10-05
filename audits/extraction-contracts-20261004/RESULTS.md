# Extraction safety proposals and numerical accounting

Thirty source-informed contracts were frozen at `f0a59665` before baseline execution. [Baseline 37207118261](https://github.com/CrispStrobe/kerotakis/actions/runs/37207118261) confirmed **20 failures and ten supported controls**: seven numerical failures and thirteen bench failures. Two supplementary review controls were frozen at `ce7fd860` before correction/execution; those two have no separate pre-repair baseline. All thirty-two frozen hashes remain unchanged.

## Findings and repair

Absent receivers bypassed safety assessment. Existing source and receiver probes used ordinary assessment instead of before/after pour context and showed pre-mix temperatures. Receiver additions could disappear or incur substantial relative error, including across combined liquid/aqueous inventory. A tiny positive extraction could be reported as zero, allowing fresh solvent to be charged without accounting for the intended solute transfer.

Extraction now prepares the source contact state, retained source, complete existing or virtual receiver, and a cloned stock ledger before committing them. Both screens use the pour hook with original snapshots and prepared proposals. Incoming solvent disturbs localized surfaces; prepared pressure and raw state are certified before screening/engine calls. Partition and solubility data are selected at the prepared contact temperature. The existing conservative total-solvent contact policy for staged extraction is retained: it is not a model of individual contact-stage geometry or thermal history.

Actual eligible donor debit must agree with the predicted extracted amount within relative `1e-8`, and output inventory must close against the original loading. Receiver solvent and solute deposits use the established checked-increment policy, including combined condensed phases. Numeric refusal restores the full checkpoint and journal and bypasses screen/solver calls. An absent receiver is created only after every preparation and both screens succeed and solvent stock can be reserved. Accepted fresh-solvent provenance is retained. Veto and stock refusal retain atomic physical behavior; stock refusal drops speculative warnings and diagnostics claiming that solutes were moved.

The numerical model avoids subtracting a nearly unchanged aqueous inventory to calculate a tiny organic yield. Logarithmic ratio/scale calculations and `ln_1p`/`exp_m1` retain positive yields, amplify repeated tiny-stage transfers, and rescale minority quantities when an intermediate normalized fraction is unrepresentable. Reviewed solubility extraction uses normalized capacities and the stable unsaturated calculation. Unrepresentable saturated remainders refuse rather than inventing an aqueous quantity. Library calculations can retain a mathematically positive minority even when the finite bench donor cannot represent its debit; the bench then refuses that physical transfer explicitly.

Ordinary staged extraction, small supported loadings, invalid inputs, stock shortage/precision refusal, broken receivers, existing receiver inventory and solvent provenance remain controls. The heat-uncertainty review control ensures that an adiabatic receiver's screened markers agree with those propagated into the committed receiver.

## Validation

Runtime source `3b46a652c2e4a90078abc8df5412193a22aa4ab0` passed [focused validation 37207540604](https://github.com/CrispStrobe/kerotakis/actions/runs/37207540604): **742 tests**, including all thirty-two contracts and existing extraction, transfer, stock, transaction and locale tests. [Full validation 37207780340](https://github.com/CrispStrobe/kerotakis/actions/runs/37207780340) passed **1,611 Rust tests and all 17 stages**, including WASM and frozen CLI suites. [Exact-binary third-fifty replay 37208896676](https://github.com/CrispStrobe/kerotakis/actions/runs/37208896676) passed **1009 checks** with **376 full-artifact integrity checks**. The 9 existing scientific qualifications remain explicit; this does not assert unqualified scientific success for every original experiment expectation.

Accepted binary SHA-256: `3784c4401c81384f730e127be1402babd4596153ab8aa5cf29b3a42913432cd8`. All 142 source bindings match the tested commit. [Accepted receipts](accepted/full-validation.json) preserve exact source/dependency, stage-log and executable identity. Small raw focused/proof/replay artifacts are archived on storage; the full executable stays hosted. Workflow defaults identify this accepted runtime source and full run rather than the later evidence commit.

## Worktree maintenance

A lightweight review found no clean, already-merged secondary worktree eligible for safe removal: 61 were clean but unmerged, five dirty, and one could not inspect its status. The two merged candidates had submodule/local changes. Authored and unmerged work was preserved.

The already-offloaded `kero-satphase` had a relative submodule Git pointer that resolved to the wrong directory after relocation. Only that pointer was changed to its existing absolute Git metadata location, with the original archived on storage. Submodule Git lookup and whole-worktree status now complete. The older offload layout still reports top-level type/deletion changes and a circular `.gitignore` warning; it was left intact rather than reconstructing or resetting potentially authored source. The [maintenance receipt](worktree-maintenance.json) records the exact change and remaining layout issue. No worktree or branch was removed.

## Resources and remaining scope
Validation and experiment execution used resource-aware scheduling. Machine-specific resource snapshots, storage locations and preservation inventories are retained privately; public scientific evidence remains indexed by the run links and receipts in this report.

This is a floating-point contract with quantified precision, not a persisted compensated quantity representation. Per-stage extraction thermal/geometry policy, accidental-spill and discard precision matrices, still-cut donor accuracy near an ULP, post-equilibration safety and independent custom-solver chemical conservation remain further work.

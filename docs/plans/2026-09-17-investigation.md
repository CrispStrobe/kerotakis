# Investigation foundations implementation plan

> **Status 2026-09-18: tasks 1-4 are implemented and verified; see the
> handoff and status file [`2026-09-18-handoff.md`](2026-09-18-handoff.md)
> for what is verified, what is not, the build environment, and the scoped
> remainder (S1-S8). Read that first.**

> **For Hermes:** Use subagent-driven-development to implement and review these tasks.

**Goal:** Deliver bounded measurements and inference on real sample state, persistent protein heat history, and checked column/coupling behavior without claiming new empirical accuracy.

**Architecture:** Extend existing core contracts and CellChain rather than replace solvers. Physical truth stays separate from serializable learner observations. Protein history belongs to unresolved material portions so ordinary transfers carry it.

**Tech Stack:** Existing Rust workspace, serde and seeded rand dependencies; no runtime Python or new numerical dependency required.

## Verification environment

The existing Rust installation is inaccessible. An isolated minimal stable toolchain is installed under /tmp/kerotakis-cargo and /tmp/kerotakis-rustup; use CARGO_TARGET_DIR=/tmp/kerotakis-target. Do not change the user's shell files or inaccessible installation. Run baseline protein tests before the first production change. Cargo.lock is locally generated and explicitly ignored by .gitignore, not tracked upstream; reconcile it against the fetched manifests, then use --locked for repeat runs. A fresh Cargo home needs online dependency downloads before offline tests can run. Background-shell exports did not survive into the next tool call here, so pass the isolated environment explicitly on each build invocation. No commits or pushes requested.

## 1. First regression: irreversible protein heat history

- Test: crates/kerotakis-core/tests/protein_denaturation.rs.
- Modify: crates/kerotakis-core/src/protein.rs, vessel.rs and bench.rs plus affected literal constructors.
- Add a bench-level heat/cool regression first; run and verify the old implementation fails by losing denaturation on cooling.
- Store a defaulted bounded denatured fraction on each unresolved portion; retain the existing threshold rule as a qualitative irreversible envelope, not an invented kinetic law.
- Update on physical operations, never on read-only observation. Preserve old direct observation behavior for current temperature.
- Verify old snapshots, new snapshot round trips, transfer/split/recombination, and mixing cooked with raw material without applying one portion's history to another.

## 2. Measurement records

- New core measurement module, tests, and lib.rs export.
- Test two-point calibration and replay first, then implement checked finite instrument configuration, quantization, saturation/censoring, seeded error, lag and persisted response state.
- Keep range status, chemical-model validity and uncertainty sources separate. Legacy ideal instruments remain compatible.
- Error models are declared instructional settings, not empirical manufacturer specifications.

## 3. Unknown-water activity and inference

- Use existing Vessel/InstrumentContract paths for physical observations, not a disconnected synthetic chemistry calculator.
- Provide a runnable native example with a serializable private experiment and an independently serializable learner notebook containing no composition/identity.
- Add bounded two-parameter estimation over explicitly declared hypotheses; retain alternatives and indicate non-identifiability rather than overstate precision.
- Tests must exercise calibration, noisy replay, notebook serialization, ambiguous data and a distinguishing observation. Synthetic tests validate inference software only.
- Document runtime limits and invocation. Do not claim a full web UI if only the native example ships.

## 4. Coupling and column checks

- Inspect delta adapter coverage and standard-stack usages before changing semantics.
- Introduce bounded enforceable dependency/ownership checking for the aqueous/adsorption interaction, not a universal scheduler.
- Extend existing transport tests with breakthrough conservation and time/spatial refinement on a declared reference problem. Do not label analytical convergence as independent experimental validation.

## 5. Independent evidence

- Read an accessible original primary measurement source, record page/table/units and rights basis, keep fitted and holdout observations separate.
- Add reviewed benchmark rows/tests only for quantities actually computed by the runtime. If source access or domain support blocks a benchmark, document it honestly rather than fabricate values.
- Correct stale validation documentation concerning already-wired registry inputs.

## 6. Acceptance and review

Run new targeted tests, core regression suite, applicable CLI/example checks, formatting, and WASM check if target install succeeds. Perform separate spec and code-quality reviews and address significant findings. Preserve the release bump and avoid unrelated edits. Report actual commands/results and any incomplete requested components.

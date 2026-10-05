# Phase ownership and bounded affine equilibrium

This pass continues the pushed `8c41d267` checkpoint. The original fifty
scripts and predictions are unchanged. Independent expectations were encoded
in new regressions before changing their respective production routes;
[predictions](phase-ownership/predictions.json) preserve their physical and
public-API contracts. [Baseline results](phase-ownership/baseline.json)
distinguish new failures from positive controls that already passed.

Source checkpoints: `d0474082` contains the engine fixes and hosted audit;
`b1314442` corrects the harness's initial assumption that the ignored root
`Cargo.lock` was tracked. Both are pushed to
`audit/systematic-chemistry-20261002`.

## Changes and baseline evidence

- Drain now withdraws each selected species from its selected phase. The
  prior route chose liquid water but consumed same-species ice first, taking
  0.4 mol ice to zero. Solids and gases remain in their original compartments;
  the lower solvent and dissolved salt reach the receiver once.
- The ethanol/water still withdraws liquid water and liquid/aqueous ethanol,
  matching the stock supplied to the VLE model. Previously its liquid-water
  cut reduced 0.4 mol ice to 0.20217170079525873 mol. Gas and solid stock now
  remain, and the cut matches a liquid-only positive control.
- Staged extraction withdraws only the selected aqueous and reviewed solid
  reservoir, retaining other phases of the same species. Previously it
  consumed gas iodine, reducing 0.0003 mol to 0.0001 mol. New controls compare
  the modeled split and full species balance with the eligible-only pool.
- Public cell-chain transport refuses nonfinite or negative stationary bulk
  amounts rather than silently removing them during mobile cleanup. The
  baseline accepted NaN solid iron. Zero and finite positive stationary
  stocks remain valid controls.
- A finite aqueous inventory can have finite mass and heat capacity while
  its temperature-change energy overflows. Transport now prepares a complete
  candidate update, verifies finite mixing-energy endpoints/readback and
  deposited stock/charge, and commits once. A later-cell failure leaves the
  complete chain unchanged and does not start reactive solves. The baseline
  accepted overflowing mixing energy; a moderate hot/cold control already
  passed its independent energy ledger. Candidate memory remains bounded by
  chain state, not step count.
- The certified affine Gibbs fallback extends from four species to at most
  six: exactly two condensed phases and two to four positive gases, with
  component rank equal to species count minus one. It retains full gas
  activity, every original atom budget, complete NASA interval coverage and
  the independent global dual stability certificate at unchanged tolerances.
  Tiny nitrogen with finite argon previously refused with
  `NotConverged(144)`; ordinary finite-diluent TP/HP already passed. Six-species
  controls cover temperature, pressure, inventory scaling and reversed pool
  order, and public-entry recovery. Seven-species and multiple-reaction pools
  remain outside this bounded certificate.

The phase-route tests use the real bench operator/validation/publication path
with an empty chemistry stack. This isolates inventory ownership across
explicit compartments, including adversarial saved/API states; it is not a
claim that ice/liquid or iodine gas/liquid coexist stably at room temperature.
The initial three-test baseline log names `phase_routes`; these new tests
are retained separately as `selected_phase_transfer`, and the preexisting
`phase_routes` suite is unchanged.

## Worktree review

The read-only ancestry scan found 67 other registered worktrees, of which
two heads are already contained in the current audit branch. Both have
modified submodule state; one also has untracked `.gate` files. Neither was
removed. [Review receipt](phase-ownership/worktree-review.json) records their
working state and the process cwd/executable check. Unmerged heads were
preserved without attempting to equate cherry-picked or rewritten history.

When unrelated activity reduced local free disk below the build startup gate,
a metadata-only review identified nine generated core test executables older
than two hours and outside every final validation target. No compiler or
active executable used them. Removing those regenerable files recovered
about 451 MiB without reading/copying their contents; source, worktrees,
compiler libraries and current validation binaries were preserved. The
[cleanup receipt](phase-ownership/cache-cleanup.json) records each file and
the measured disk headroom.

## Resource-aware validation
Validation and experiment execution used resource-aware scheduling. Machine-specific resource snapshots, storage locations and preservation inventories are retained privately; public scientific evidence remains indexed by the run links and receipts in this report.

The existing chemistry-audit workflow now offers a manual `independent-50`
selection on an isolated hosted Ubuntu runner. It runs the selected native
and WASM checks serially, then the frozen original CLI scripts plus four
explicit-sucrose controls. Other audit fleets retain their original defaults
and are skipped for this selection. Logs, source hashes, the tested binary
and generated dependency lockfile are preserved together. The harness uses
`--no-fail-fast` and records all selected build/check outcomes; a failed suite
does not hide independent suite results.

The first hosted attempt stopped before compilation because a new `--locked`
flag was incompatible with the repository's ignored root lockfile. Its
[failure evidence](phase-ownership/hosted-lock-failure/) remains separate;
no chemistry tests ran there. The corrected
[hosted run](https://github.com/CrispStrobe/kerotakis/actions/runs/37048306794)
uses the existing CI dependency-resolution policy and records the actual
generated lockfile. [Execution receipt](phase-ownership/hosted-execution.json)
pins both attempts and their source checkpoints.

## Final results and reproducibility

The corrected hosted run passed all selected checks at
`b131444248625737a1edd5c02559fa50f1e83203` using Rust 1.99.0.
[Validation receipt](phase-ownership/validation.json) records every command,
exit code, log hash, source hash and the generated dependency lock hash.
[Summary](phase-ownership/validation-summary.json) binds the replay and suite
results to that checkpoint.

| Selected Rust suites | Passed |
| --- | ---: |
| Core units and operator/inventory regressions | 778 |
| CEA units and integration tests | 123 |
| PHREEQC exchange/surface transport | 2 |
| CLI units and contracts | 62 |
| Total | **965** |

The WASM target compile check passed; no browser runtime or frontend test
count is claimed. The earlier local CEA run is an additional positive check,
not added again to the 965 hosted-test total.

All 100 original text/JSON CLI starts are preserved in
[replay evidence](phase-ownership/replay/): 96 successes and four expected
ambiguous-sugar refusals. Four separate explicit-sucrose controls succeeded.
The physical assessment met **49 of 50** frozen expectations: one known zinc
rate gap, zero unexpected mismatches and zero `SolverFailed` events. The
supplemental rate, gas-explanation, combustion and layer-scope checks passed.
The original predictions SHA-256 remains
`0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.
These timings include builds and process execution and are not performance
benchmarks.

Large artifacts are retained separately.
The hosted tested binary (96,138,496 bytes), exact generated lockfile and
GitHub-prepared source archive (16,608,786 bytes) were hash-verified. All seven
changed source members match the current checkout, the committed Git blobs
and the source archive. The binary hash also matches the hosted CLI replay.
[Artifact receipt](phase-ownership/artifact-hashes.json) records their paths,
sizes and digests; [tested lockfile](phase-ownership/tested-Cargo.lock) is
versioned with the evidence. The queued local source-archive guard was stopped
while still waiting, and the prepared archive was retrieved instead. Transfers
and verification used low priority and bounded buffers; no additional local
build or large in-memory archive was started.

## Remaining scope

This is still a one-reaction certificate. Larger represented pools, multiple
reaction degrees, absent-gas tangent boundaries and unsupported source
intervals require further work. Finite multiphase/swept combustion and broad
aqueous/interface reconciliation remain model extensions. The original zinc
kinetics expectation still requires a justified rate model.

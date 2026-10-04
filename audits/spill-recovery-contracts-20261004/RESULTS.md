# Spill recovery: screening and quantity contracts

Fifteen source-informed contracts were frozen at `c9f9e319` before baseline execution. [Baseline 37201593997](https://github.com/CrispStrobe/kerotakis/actions/runs/37201593997) confirmed **12 failures and three supported controls**. Two supplementary inspection/review controls were frozen separately before repair execution: pour-hook snapshots and incoming gas pressure. Those two have no separate pre-repair execution. All seventeen test hashes remain unchanged.

## Findings and repair

The recovery safety probe omitted unresolved parcels, showed the receiver's old temperature before adiabatic mixing and omitted committed unpriced-heat markers. Recovery also retained localized surface geometry while pouring new inventory into the receiver. Fractional multiplication could create material without an accurately represented donor debit or erase subnormal material; unchecked receiver additions could swallow or badly round the arriving inventory, including across combined condensed phases. Zero recovery still ran settlement and could change temperature despite moving nothing.

The repair prepares checked retained and moved quantities for every spill phase and unresolved parcel. Recovery of broken-container solids and gases therefore remains available. It uses the shared prepared receiver to incorporate the actual material, temperature and disturbed surface; the screened receiver is the object committed after acceptance. Adiabatic unpriced-heat markers are prepared before screening. Raw state validation precedes engine calls.

Review also found that gas from a broken-container spill could enter a sealed receiver without its proposed pressure being screened. The receiver now refreshes pressure (and owned volume for pressure-controlled boundaries) before certification and screening. The safety pour hook receives the actual receiver before the operation and the complete prepared receiver afterward. A veto retains the existing journaled atomic-refusal behavior and skips settlement. Numeric refusal restores the entire command checkpoint and journal and calls neither screen nor solver. Zero recovery skips both screening and settlement. Ordinary partial and exact full-quantum recoveries remain supported.

The relative precision policy is the existing `1e-8` policy for donor splits and receiver deposits. This repair does not introduce arbitrary-precision or compensated quantities. Existing accepted-transfer solver-failure semantics remain in force.

## Validation

Runtime source `b8b448ec2a95b2f18ea8b9fabd47d075d89926e3` passed [focused validation 37201866023](https://github.com/CrispStrobe/kerotakis/actions/runs/37201866023): **726 tests**, including all seventeen recovery contracts and existing spill, transfer, transaction and locale controls. [Full regression 37202053377](https://github.com/CrispStrobe/kerotakis/actions/runs/37202053377) passed **1,579 Rust tests and all 17 stages**, including WASM and the frozen CLI suites. [Exact-binary third-fifty replay 37202945305](https://github.com/CrispStrobe/kerotakis/actions/runs/37202945305) passed **1003 checks** and **370 full-artifact integrity checks**. The 9 existing scientific qualifications remain explicit; this is not an unqualified pass of every original scientific expectation.

Accepted executable SHA-256: `433fe5a9690e3e6cdce45d5b74ebb2552dd3df78b704a66b254f4bb51394290c`. All 136 source bindings match the tested commit. [Accepted receipts](accepted/full-validation.json) preserve source/dependency, stage-log and executable hashes. Small raw artifacts remain on storage; the full executable artifact stays hosted. Workflow defaults identify the accepted runtime source and full run, rather than this later evidence commit.

## Resources and further work

Initial load was about 1.8, available memory 5.6 GiB, and root/volume disks had about 4.9/5.1 GiB available. Two read-only agents reviewed spill recovery and extraction; a subsequent read-only spill patch review caught the pressure gap. All builds and app execution stayed serial on hosted runners. Small source and evidence operations ran locally; raw artifacts and the pre-repair source backup are on `/mnt/storage/kerotakis-maintenance-20261004/`.

The separate [extraction proposal](extraction-follow-up.md) was subsequently frozen and addressed by the [extraction audit](../extraction-contracts-20261004/RESULTS.md): absent receivers, pour hooks, prepared thermal state, donor/receiver precision and cancellation of tiny positive partition yields. Per-stage thermal/geometry policy remains separate scope. Accidental spill creation and deliberate discard still use separate fractional arithmetic and need their own donor/receiver precision matrices; recovery contracts do not validate creation of the spill. Still-cut donor accuracy near an ULP, post-equilibration safety and persisted compensated quantities remain separate work.

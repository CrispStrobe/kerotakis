# Transport effluent ownership repair

Two frozen pure-water controls on c446/670 complete but place the receiver at
the inlet temperature rather than the outgoing cell: errors of−40K and+40K.
[First raw failures and source binding](https://github.com/CrispStrobe/kerotakis/blob/d3b7cf5088a9001cce0e75ebdf6651943142e8f7/audits/new-experiments-201-20261016/transport-effluent-followup/baseline-results/README.md)
remain immutable. Represented material and cell temperature checks pass there.

The bench now mixes each actual effluent parcel into one local receiver
proposal at its own temperature and phase, carrying analytical charge and
measured free acid/base references. Only event narration aggregates species
amounts. Memory no longer needs to grow with step count. All chain/receiver
proposals commit after validation. Column, representative inlet and receiver
roles must be distinct; aliased roles refuse before mutation. Valid zero-flow
checks geometry once and preserves matter, temperature and caches.

`transport.rs` scales and transfers measured proton/hydroxide alongside mobile
matter and invalidates moved-state characterization. New parcel fields default
to zero when reading historical JSON. Hydraulic volume remains the existing
liquid-phase solvent proxy; no tolerance, mineral or hydrate model changes.

Acceptance added: ten core groups for legacy JSON, opposite exact temperature
controls, multiple-parcel nonempty-receiver enthalpy, aqueous phase retention,
signed charge and acid/base complements, zero flow, rejected geometry, alias
rollback and tracer movement. Two native groups run exact copies of the frozen
scripts through the standard PHREEQC stack, checking temperature and represented
material/charge including the unchanged representative inlet.

Source diff review and rustfmt pass. No Cargo/native/Wasm execution has run for
this head locally. Compile, both native platforms, the two source-bound native
controls, Wasm compatibility and the full applicable own fleet remain required.
Keep the PR draft until all nonprotected acceptance is complete, then use normal
exact-head auto-merge. Preserve any first CI failure separately before repair.
This is an ownership repair, not empirical heat calibration or expanded reactive
hydraulic coverage; original TR003/TR007 domain refusals remain separate.

# Supply finite native gas inventory in moles

The closed brine GS-010 failure is a native input-boundary mismatch. The adapter
supplies ideal pressure `nRT/V`, but PHREEQC initializes gas moles with the
database's equation of state. `pitzer.dat` gives CO2 critical parameters;
`wateq4f.dat` does not. The native initializer therefore follows different
branches even though Kerotakis supplied the same finite amount.

A [distinct smaller CLI trace](../audits/target468-20261016/finite-gas-eos-first-control/)
uses authenticated source `5b054e6c` and binary `6c617d2b`, preserving input and
selected output. Its PHREEQC adapter matches the integrated source. A separate
real solver comparison uses the authenticated production IPhreeqc Wasm bytes
and exact vendored Pitzer database. Supplying0.001mol through ideal pressure
returns0.0010003390707956424mol carbon. Native raw component moles return
0.000999999999999987mol, against the same frozen1e-10mol bound. Both actual
solver calls return0; only the latter conserves the supplied carbon.

The source correction uses existing `GAS_PHASE_RAW` component moles, explicitly
selects that finite phase, and retains the native database/EOS. Temperature in
that raw format is Kelvin. Pressure is only a required starting estimate;
it no longer initializes matter. Every supported gas gets its owned molar
amount, including initially absent phases that may form from aqueous matter.
No material is rescaled after solving, and no critical constants, activity
parameters, equilibrium constants or scientific forecasts change.

Existing fractional-temperature/solvent serialization tests retain their material
bounds and open-input control, updated for the raw format's explicit molar and
Kelvin fields. New actual native controls cover dilute/Pitzer-routed solutions,
two gas loadings, two temperatures, four repeated equilibrations, and initially
empty gas phases. Each checks C/H/O/Na/Cl and charge, plus actual aqueous/gas
readback. A direct real-solver check of the proposed format and empty-phase
formation also succeeds. The first comparison reader asked for an unexported
whole-string function; the correction uses the existing exported line reader.
That harness failure is separate from the preserved legacy carbon failure.

## Remaining acceptance

Formatting/source inspection pass. The repaired Rust code has not yet been
compiled or executed. Run the new focused controls and unchanged headspace,
finite inventory, native and full original fleet gates on the exact signed
head; inspect both native platforms. Preserve any first failures. Integrate
normally only after applicable acceptance, then freeze a separate production
GS-010 replay. The original deployed failure remains unchanged and no new
catalog selection or empirical salinity calibration is claimed.


## First actual native acceptance and combined-source follow-up

Source `8eece0b1` passed Clippy and all524 PHREEQC native tests across97
test targets, with zero failures or ignored tests. Both new finite-molar
controls pass. Saved review authenticates the actual worker/source/tree, raw
streams and producer lock `66886813`; this is not a full-fleet acceptance.

The normally integrated phase/energy repair `0a5a92f1` is now merged into this
narrow branch for combined-source validation. Re-run acceptance on that new
head before integration; do not relabel the earlier native run as execution
of the combined source. Original evidence and forecasts remain unchanged.

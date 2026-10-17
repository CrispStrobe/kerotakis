# Organic-solubility provenance and finite availability

Saved first120 case PR-016 exposes a shared limitation: legacy NaCl/ethanol
solubility controls its reported 55.60973940% AgCl yield, while residual reactants
are labelled liquid. The table contains65 rows with withdrawn provenance and one chlorophyll
teaching estimate without a measured numerical limit.
The original output and forecasts remain unchanged; this case is held from
working selection. A successful process exit does not validate that yield.

## Bounded repair

The65 legacy rows now carry structured `Withdrawn` provenance; the additional
chlorophyll row carries `EditorialEstimate` with a distinct diagnostic. Both numerical
consumers report `ModelBoundary`: the curated route records it before consuming
reagents, and the nonaqueous route includes liquid-only residues. German and
French diagnostics disclose that dissolved amounts, phase partition and dependent
yields remain provisional. Numerical table values are unchanged.

A missing organic-solubility row now makes only already-liquid inventory available
to a solvent-gated reaction. It cannot silently make all solid inventory available.
This removes the unsafe fallback before any future table shrink. Existing gated
silver reactions have table rows for every reactant; this fallback correction
introduces no new solubility value or new reaction support.

Acceptance: rustfmt, whitespace and engine locale lint; native regression groups
for all66 pairs with split phases, liquid-only residues and inert controls,
complete reagent consumption, and missing-row finite availability. Both native
platforms, applicable Wasm/browser/cache checks and the full own fleet must pass
on the reviewed head before integration. No scientific pass is assigned by this
repair. Hosted acceptance is pending until its source-bound checks finish.

## Next implementation lanes

1. **Phase ownership.** Trace Add, curated availability/withdrawal, organic
   dissolution and measurements. Freeze PR-016 and solid/liquid starting-stock
   controls; prove retained plus reacted material by phase and element. Distinguish
   explicit prepared solutions from unmeasured solid feed. Never manufacture a
   dissolved fraction from the absence of data.
2. **Compatible primary data.** For each required solute/solvent pair, record the
   original measurement, temperature, units, composition convention, licence and
   checksum under the repository provenance policy. Do not replace withdrawn
   values with uncited estimates or republished handbook numbers. Begin with
   NaCl/ethanol and AgNO3/ethanol because they control PR-016.
3. **Retire legacy numerical support.** After the ownership contract and data are
   reviewed, use supported rows only. Unsupported pairs must preserve finite
   inventory and disclose the unresolved dissolution/yield rather than claim a
   zero-solubility result. Test complete and incomplete consumption, repeated
   waits, dilution, mixed solvents and downstream measurements.
4. **Bound physical follow-ups.** Freeze new exact-source protocols and numerical
   forecasts before execution. Preserve the first PR-016 result and compare phase
   ownership, AgCl conversion and balances. Select a working case only after its
   stated physical checks pass; conservation alone is not empirical calibration.

## Preserved source-review correction

The first pushed head `173115fbbb02c48578dd315e3b243e84cf4a9a43` missed the
additional chlorophyll initializer and incorrectly asserted65 total rows.
Independent source review identified its missing required field before any local
compile or chemical execution. The correction explicitly distinguishes65
withdrawn entries from one editorial estimate and tests all66. This required
compile correction supersedes that head for acceptance; no first result is erased.

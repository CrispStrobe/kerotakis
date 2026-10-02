# Conservation and condensed-boundary followup

This continues the source and replay evidence through `77334c56`.
The original 50 designs and predictions are unchanged.

## Changes

- Bulk and electrode withdrawals preserve every unrelated positive trace instead
  of applying a presentation cutoff to physical inventory. Public withdrawals
  sum the quantities actually removed: requesting 1e100 mol from 0.009 mol
  must report 0.009 mol, not zero after subtractive cancellation. Nonfinite
  requests return an invalid amount without mutating inventory.
- Adsorption, finite headspace partitioning and heat of mixing now opt into
  production element-ledger enforcement. Their scoped changes redistribute
  the same represented species or change temperature. Surface sulfate/water
  and sodium/calcium exchange examples independently check atoms and charge;
  they do not assert a universal PHREEQC/interface conservation tolerance.
- Mass audits also detect creation from empty or trace initial inventory.
  Previously the mass comparison only ran when the initial mass exceeded its
  floor. Stale comments claiming the corrected CO2 readback still had an
  ignored mass-conservation defect are updated; physical assertions remain.
- Complete snapshots reject invalid electrode deposit thickness, coverage and
  resistivity, overflowing aggregate film resistance or reactive area,
  invalid capacitance/potential pairing, nonfinite electrochemical diagnostics,
  overflowing current-area products and partial-current sums. Signed currents
  and potentials remain valid. Material progress fractions and persisted
  visual geometry receive their own numerical-domain checks, with atomic
  refusal and positive-control regression tests.
- The general TP engine can now certify selected all-condensed equilibria
  without attempting a singular log-gas Newton variable. It considers a
  feasible condensed seed or pure stoichiometric inventory, then verifies
  occupied-phase equality, every inactive condensed-phase affinity, full
  element balance and the joint ideal-gas tangent inequality. A missing
  certificate preserves the original Newton path/refusal. This is a bounded
  proof of stability, not guessed acceptance of a stalled iteration.
- The certificate uses the existing NASA thermodynamic records and Newton
  chemical-potential tolerance. Its regressions independently check graphite
  saturation pressure on both sides, the joint C/C2/C3 gas tangent, carbonate
  competitors, salt solid/liquid selection, tiny positive stock and numeric
  overflow. Numeric output retains every positive mole, keeping composition,
  gas totals and enthalpy on the same inventory basis. The certificate requires
  an actual NASA fit interval at the requested temperature for every pool
  record; an out-of-range gas cannot be ignored to claim condensed stability.
- Preserving positive traces exposed an ethanol combustion reference bug:
  its gas fit starts at 300 K, while its NASA header independently supplies
  formation enthalpy at 298.15 K. The exact header datum is now accepted at
  that reference point. Neighboring out-of-range temperatures still refuse;
  no heat-capacity polynomial is extrapolated.

The condensed optimality conditions follow [NASA RP-1311 Part I, equations
2.9–2.11 and sections 3.4/3.6](https://ntrs.nasa.gov/api/citations/19950013764/downloads/19950013764.pdf).

## Evidence

Source checkpoint: `caf9ebe9`. Final core validation passed 673 tests; final
CEA validation passed 107 tests; CLI contracts passed 62 tests, for 842 selected
Rust tests in this pass. Browser cross-target compilation also passed
(`cargo check -p kerotakis-wasm --target wasm32-unknown-unknown -j1`); this is
not a claim of browser runtime coverage. No frontend test run is counted here.

The actual rebuilt CLI ran all 50 original scripts in text and JSON mode:
96 commands succeeded and four formula-ambiguous sugar commands refused as
expected. Four separate explicit-sucrose followup commands succeeded. The
physical assessment passed 49 of 50 expectations, with only the known zinc
rate-model gap and zero unexpected mismatches. All 50 assessed cases contain
no `SolverFailed` events; supplemental combustion, layer-scope and rate-boundary
checks pass. Frozen predictions still hash to
`0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.

[Validation receipt](followup/validation-summary.json) pins the complete source
checkpoint, binary hash, source hashes, test log hashes, resource policy,
cleanup receipts and superseded failures. Raw stdout/stderr and the
[assessment](followup/assessment.txt) remain in `followup/`.

The first interface test requested a deliberately withheld ethanol/water
mixing-heat prediction; its corrected acetone/water fixture independently checks
C/H/O balance. Trace preservation initially exposed seven alcohol-combustion
regressions; all seven pass after the exact reference-anchor fix. A new
conservation helper was also corrected to include reactant-only NASA records
and explicitly reject unknown names, rather than silently skipping them.
The original 50 physical checks and frozen predictions remain unchanged.
A supplemental check had incorrectly prohibited every ethanol exhaust event,
including a retained 1e-18 mol Newton trace emitted after ignition. It now
requires zero ethanol venting before ignition and bounds summed post-ignition
ethanol exhaust to 1e-8 of the original feed. Flame energy and temperature
are unchanged; the superseded assertion failure is preserved alongside the
successful assessment. Numeric inventory is not filtered to satisfy the audit.

Builds remain single-job and tests serial. Resource guards gate startup by
available RAM, swap, load and disk space and terminate only their own process
when RAM/swap or disk headroom becomes unsafe. Large receipts and verified
archives stay under `/mnt/storage/kerotakis-maintenance-20261002/`; active
compilation stays on `/mnt/volume1`. Completed reproducible test executables
and a verified superseded library archive were removed to recover
728,445,777 bytes of disk space; source, branches and active build outputs
were preserved. Tested source and CLI artifacts were archived on storage with
hash receipts. The final browser check completed with about 5.28 GiB available
RAM, 9.70 GiB free swap, load1 3.12 and 6.88 GiB free workspace disk.

## Remaining limits

Positive-gas multicomponent Newton rank degeneracies still need component
reduction and stronger phase birth/extinction handling. The condensed branch
cannot find every feasible mixed condensed inventory or certify every stable
endpoint; it refuses/falls back when its proof search is incomplete. Full
finite multiphase combustion, swept thermal balances and broad aqueous/interface
ledger reconciliation remain larger model extensions. Zinc/acid dissolution
still lacks an approved rate model; no kinetic constants are guessed here.

Subsequent component and ordered-transfer closure is recorded in
[COMPONENTS-AFTER.md](COMPONENTS-AFTER.md), with separate validation receipts.

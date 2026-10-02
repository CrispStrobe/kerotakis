# Further closure of the independent audit

This continues the work committed in `590b37489717cd226ca08e915202973e2a0e01da`.
The 50 original scripts and predictions remain frozen. Raw replay results and
final validation receipts for this pass are in `extended/`; previous stages
remain in their original directories.

Subsequent conservation and condensed-boundary work is documented in
[FOLLOWUP-AFTER.md](FOLLOWUP-AFTER.md), with separate replay receipts.

## Changes

- Production solver commits validate gas-event identity, amount and formula.
  Solvers with complete element ledgers opt into balance enforcement before
  state or events become visible. Inlets extend the incoming ledger; outlets
  extend the outgoing ledger. Both ordinary and native MIX proposals roll back
  on refusal. This is enabled for CEA and selected core physical passes, not
  asserted for every incomplete aqueous/interface model.
- Numerical checks cover aggregate mass, charge, element inventory, liquid
  volume, heat capacity and sensible energy, as well as individual quantities. NaN cannot
  silently satisfy the conservation comparison. Operator proposals restore
  vessels, spills, cabinet stock and journal length if application fails or
  overflows; accumulated waste receives the same checks as vessels.
- The core retains its explicitly disclosed absolute-zero cooling floor as a
  mathematical boundary, while native thermochemistry, ignition and heat-feed
  records require positive temperatures. Warming from that floor omits an
  unsupported native feed record. Empty-vessel ignition retains its established
  no-fuel result instead of being rewritten as a missing chemistry engine.
- Finite, nameable ideal-gas combustion now retains products and solves the
  sealed U/V or pressure-controlled H/P balance. Tests independently check
  energy, atoms, pressure and expansion work. The supported pool is restricted
  C/H/O/N chemistry; unsupported condensed/interface/sweep/fuel-rich domains,
  steam below 800 K and pressures above 100 bar refuse explicitly. Conservation
  is not a claim of complete species coverage or high-temperature accuracy.
- Open CEA accounts for net atmospheric O2/N2 exchange, including original
  inventory returning to the room and numerical traces. Trace amounts remain
  in the numeric ledger and pass through existing presentation filters.
  Unmatched energy floors and adiabatic TP guesses no longer commit.
- Strict energy checks exposed a general condensed phase transition gap.
  Equal-Gibbs solid/liquid coexistence now closes the latent-heat interval with
  partial phase amounts. It checks source ranges, pressure, gas state, competing
  polymorphs, element balance and corrected enthalpy. Source coefficient rounding
  bounds the equality check; unrelated discontinuities are refused. Chemical
  coexistence additionally verifies common active chemical potentials and rules
  out lower-Gibbs inactive condensed phases before mixing endpoint states.
- Source-limited chalk heating exposed repeated artificial air heating. The
  bounded CaCO3/CaO/CO2 crucible slice now uses 1 bar total released-gas pressure
  without entrained room-air thermal mass. It represents CaCO3/CaO condensed
  phases and CO2/CO/O2 ideal gases only, with a 2000 K domain ceiling. A NASA
  mass-action gas solve and decomposition-affinity endpoints avoid the broad
  TP engine's all-condensed/incipient-gas Newton degeneracy in this explicit
  slice; that broader numerical limitation is not claimed fixed. Oxygen-requiring
  flame routes retain cold-air pricing. Independent NASA calculations put decomposition near
  1159.09 K and the 0.1 mol latent interval at 9.738–26.317 kJ; the old 7 kJ
  partial-conversion fixture is corrected to 17 kJ. The original 40 kJ
  full-conversion and apparatus-ceiling expectations remain strict.
- Each eligible HEAT pass supplies native thermochemistry with its pre-pass
  inventory, temperature and the joules actually delivered under the source
  ceiling. This lets the native engine price fusion/calcination from the
  original state; the provisional core thermometer is not its energy target.
  The context is step-local, omitted from persistence and cleared after each
  pass. Changed stock, prior boundary transfers or incomplete interfaces do
  not qualify for this recovery. External room air is priced independently
  at 298.15 K rather than preheated to a hot vessel's temperature. Successful native heat budgets commit their verified
  temperature even without species changes, including corrections below 1 K;
  presentation thresholds no longer determine physical energy accounting.
- Feed thermochemistry uses the actual in-range phase or a bounded in-range
  condensed sibling before considering vapour. Missing records refuse instead
  of borrowing vapour formation energy. A stable-phase recovery explicitly
  discloses that preceding dry HEAT latent energy is not reconstructed.
- `olive_oil` is a distinct conserved unresolved mixture. Its reference geometry
  uses the primary pycnometer value 0.9161 g/mL at 20 C for the reported sample;
  uncertainty and population variation are not invented. The single scalar has
  a scoped attribution grant and pinned transcription/source PDF hash. Its
  composition, heat capacity, spectrum, partition models and kinetics remain
  absent. Native/web German and French names and About attribution are updated.
- The lowest-detail gas narration no longer asserts liquid absorption or visible
  bubbles for every boundary flow. Web coverage includes the existing missing
  heat disclosure and confidence path. The exporter regression now recognizes
  the previously reviewed KNO3 primary record while preserving all other legacy
  contracts and exact registry regeneration. Actual CLI finite-flame checks
  distinguish retained products below the glass rating, complete accounted
  venting after a burst, and expansion under pressure control; sealing traps
  represented room air rather than creating an evacuated vessel.

## Evidence and practical limits

Final counts and hashes are in `extended/validation-summary.json`. Build, test,
resource and cleanup logs remain under `/mnt/storage/kerotakis-maintenance-20261002/`.
The tested source checkpoint is `17ba241b707efd78cd10f7daffd897e7684e891b`.
There are 913 distinct selected passing Rust tests: 728 core, 99 CEA, 77 CLI
and 9 data/export. This combines final reruns with unchanged earlier suites,
not a claim that every workspace test ran again. All 62 final CLI contracts
and all 99 CEA package tests passed against the final aggregate numeric guards.
The web's full 1,645-test run exposed five failures; their three affected files
were corrected and all 36 tests in those files passed. Type checking found zero
errors; a subsequent two-file clarification of intentional initial-state reads
removed both existing warnings. Twelve existing frontend tests passed after
that change, and the final type check reported zero errors and zero warnings.

`verify_extended.py` assesses the same frozen physical expectations. The two
ambiguous formula-only sugar inputs retain explicit sucrose followups; the
three original olive-oil scripts now run without a substitute. Oil mass uses
its reviewed reference density rather than the old vegetable-oil surrogate's
arithmetic. Original predictions are not changed to hide model boundaries.
The 100 original CLI subprocesses completed with 96 successes and four explicit
ambiguous-sugar formula refusals; four sucrose followup subprocesses succeeded.
The final physical assessment meets 49 of 50 expectations, with zero unexpected
mismatches and no `SolverFailed` events in any assessed case. An initial assessor
error is retained in the logs: `look` lowers to a `measure`/`eyes` JSON operator,
and the 50 mL water dose has 49.85 g mass, so the original oil-layer scene must
conserve 95.655 g using the reviewed olive-oil density. Correcting that harness
did not require another source change or modify the frozen designs.
Zinc acid dissolution still lacks an approved rate model and remains an honest
unmet physical prediction, not an inertness claim.

The new element guard is deliberately opt-in: broader aqueous/interface ledgers
need independent reconciliation before enforcement. Dry HEAT calorimetry outside the validated unchanged-stock NASA slice,
full finite multiphase combustion, radicals/ionisation/real gases,
swept thermal inlet/outlet balances and new measured kinetics are not supplied
by these patches. These remain substantive model extensions rather than guessed
numbers. The coexistence tests establish balance and stability in their source
slice, not universal phase diagrams.

Builds use one Cargo job, tests run serially, and CLI replays use one worker.
Three agents were used at most. Large references, verified archives and receipts
stay on `/mnt/storage`; active compilation stays on `/mnt/volume1`. Cleanup only
removes reproducible ignored build caches after completed checks or verified
archival, preserving source, branches and unmerged worktrees. Extra validation
and snapshots add runtime work; no speedup claim is made from a busy shared host.

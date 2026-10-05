# Inventory routes and positive-gas phase endpoints

This pass continues `062514c5`. The original fifty scripts and frozen
predictions remain unchanged.

Filtration previously transferred liquid and aqueous portions, then deleted
every source portion except solids. The actual CLI sealed 0.032294582578672695
mol nitrogen with water; filtration erased that nitrogen from both vessels
without a vent event. Cleanup now removes exactly the transferred phases.
The operator regression checks ordinary and trace gas, the retained solid,
and the liquid/dissolved receiver inventory. The independent CLI input and
raw before/after output are in [inventory-routes](inventory-routes/).

The same audit found positive inventory cutoffs in transport, bound-dye
release, fermentation material cleanup and four private withdrawal helpers.
Transport now retains positive mobile remainders and unmoved solid traces;
tests cover Courant fractions zero, one-half and one, checking cell plus
effluent rather than relying on a tolerance larger than the entire trace.
Bound release preserves separate matching portions and unrelated bound
traces. Kinetic, phase-route, fermentation and curated withdrawals now use
the shared phase-specific withdrawal API; curated liquid-before-solid order
is preserved. A large kinetic withdrawal reports actual removed stock.
Real lactose consumption checks remaining milk, water and unrelated oil.
No kinetic rate constants or isotherm parameters changed.

A represented carbonate/lime/CO2 pool with excess carbon necessarily has
positive gas. At fixed temperature and pressure its pure-gas chemical
potential is fixed; admitting both condensed phases can make Newton singular.
The new fallback runs only after that singularity, for exactly one gas and
at most one occupied condensed endpoint. It independently verifies every
original atom budget, occupied chemical-potential equality, inactive-phase
stability inequality, actual NASA interval coverage and finite readback.
These dual conditions certify the global Gibbs minimum in the represented
pool. A failed proof still refuses; no stalled Newton state is accepted.

Analytical tests compare the carbonate decomposition reaction affinity
against the returned endpoint across temperature, pressure, pool order and
inventory scale. They include equal-G coexistence, unsupported or inconsistent
domains, nonfinite energy and positive inventory below the Newton trace floor.
At equal G, an endpoint is a valid minimizer; this does not uniquely select
an interior phase fraction or establish a general mixed-gas solver.

## Validation

Source checkpoint: `7cd7d10e`. Selected core validation passed 725 tests,
CEA passed 115 (44 unit tests and 71 integration tests), and CLI contracts
passed 62: 902 Rust tests in this pass. Browser cross-target compilation
passed; no browser runtime or frontend test count is claimed.

The public positive-gas endpoint regression refuses with `NotConverged(2)`
when only the new singularity callback is disabled. Its raw failing log and
restoration receipt are retained; the restored final source passed the full
CEA suite. The pre-fix filtration unit regression also erased a positive
gas trace, complementing the actual CLI evidence above.

The rebuilt CLI retains all trapped nitrogen on the same filtration script,
with no solver failure. The original fifty scripts ran in both text and JSON:
96 commands succeeded and four formula-ambiguous sugar commands refused as
expected. Four separate explicit-sucrose followups succeeded. Assessment met
49 of 50 frozen predictions; zinc kinetics is the single known gap, with zero
unexpected mismatches and no `SolverFailed` events. Supplemental combustion,
rate-boundary and layer-scope checks pass. Prediction SHA-256 remains
`0fd6e4831f5da264c892a5718b8619ae267388df0512fe54dab04fff30287064`.

[Validation receipt](inventory-routes/validation-summary.json),
[CLI comparison](inventory-routes/filter-assessment.json),
[assessment](inventory-routes/assessment.txt) and all raw replay output are
retained. The tested primary Git source archive and CLI binary were copied to
the separately retained artifact; ten changed source members and the binary copy were verified
against their recorded hashes. The [artifact receipt](inventory-routes/tested-artifact-hashes.json)
pins those files to the tested checkpoint.

Builds used one Cargo job and serial tests, with large work on storage.
Startup required load1 <= 3.5 on four CPUs, available memory >= 3200 MiB,
free swap >= 600 MiB and workspace disk >= 2048 MiB. Pressure checks paused
only the own process group when load1 exceeded 6 or memory/swap/disk became
unsafe, and resumed under the startup thresholds. Actual resource pauses
and waits are recorded; elapsed times include waiting and are not benchmarks.
Two agents performed bounded source work without concurrent builds.
The final archive completed at load1 2.99, with approximately 4.6 GiB available
RAM, 9.5 GiB free swap and 4.9 GiB free workspace disk. No unrelated processes,
worktrees or branches were removed in this pass.

## Remaining scope

General mixed-gas transient rank loss, arbitrary condensed assemblages,
finite multiphase/swept combustion and broad aqueous/interface bookkeeping
remain outside these bounded fixes. Zinc/acid kinetics still lacks an
approved rate model. Scientific stability claims apply to the represented
thermochemical pool, not every possible chemical species.

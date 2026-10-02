# Systematic extension of the independent audit

Further work is recorded in [EXTENDED-AFTER.md](EXTENDED-AFTER.md); this report
retains the results of the preceding checkpoint.

This implements the follow-up to [SYSTEMATIC.md](SYSTEMATIC.md). The original
50 scripts and prediction hash remain unchanged. Baseline and targeted-fix
results remain in `results/` and `after/`; this extension's raw CLI output is in
`systematic/`. Resource/build logs and cleanup receipts live under
`/mnt/storage/kerotakis-maintenance-20261002/`.

## Changes and newly discovered causes

**Whole-state commits and rollback.** Snapshot deltas preserve inventory,
interfaces, boundaries, solution/speciation, caches and transient model flags.
They reject stale bases and edited terms. Shared numerical guards include
headspace/bath constraints, dissolved and interface state, pe/redox, gas-driving
pressure and signed charge/energy/transfer fields. Native deltas validate the resulting
state before assignment, including cumulative overdraw and finite arithmetic.
SolverStack restores state after failed/invalid solves and declined/failed MIX.
Conserved orchestration includes explicit gas inlet/outlet events and checks even
an event-only proposal. Production solvers now have rollback and state-validity
contracts; an element veto has not been enabled indiscriminately for every
production route because open-reservoir ledgers are not uniformly complete.

**Canonical aqueous state.** Recharacterization rejects changes to finite
interfaces, secondary solvent routes or per-redox inventories. Accepted changes
refresh pH, pe, proton/hydroxide activities and gas-driving pressure coherently.
Native molality/activity selected output refines matching aqueous species within
the existing solve; the human report rounds to four significant figures. Unselected
complexes keep that report's rounding limits. No extra solve is required.

A final MIX review found that native merged readback omitted surfaces, exchange
beds and mixed crystals. Empty interfaces could therefore disappear without
violating an elemental ledger. PHREEQC MIX now declines whenever the target or
either source owns these interfaces, so the ordinary operator fallback retains
the named interfaces and runs full direct chemistry. Nine interface/owner API
cases check whole-state nonmutation; a fractional zinc-sulfate MIX into a named
HFO bed checks adsorption, capacities, inlet zinc and whole-bench zinc/mass.

The extended surface regression found a separate physical bug: HFO initialization
with `SURFACE -equilibrate 1` loaded zinc onto the interface without debiting the
solution. Removing that preload gives the actual batch one finite adsorption
budget, rather than rescaling an inconsistent result after the fact.

The existing surface transport oracle then exposed a numerical edge in pure-water
HFO initialization. PHREEQC's high-precision output selects a 1e-12 convergence
tolerance; its charge residual criterion multiplies that by ionic strength and
water mass. In the minimized 0.099999464 kg cell the criterion was around 1e-20 mol,
below the 1.09e-19 mol roundoff residual. Empty-electrolyte finite-surface
initialization now uses 1e-10 (still stricter than the native 1e-8 default), while
keeping high-precision output. Neighbor-volume regressions verify H/O conservation,
finite capacities and no imported zinc/sulfur. Solute-bearing problems and the
original transport/oracle acceptance thresholds are unchanged.

**Observable support.** One derived model-support contract supplies status,
phase scope, reasons, assumptions, provenance and validity for temperature,
enthalpy, mass, pressure, pH, conductivity, absorbance, reaction rate, headspace
volume, density and activity. Measurement
records/events and scenes carry this contract; CLI prose and web quantities expose
it. Instrument range remains distinct from model validity. Mixed-solvent and
heat-capacity limits propagate to dependent observables. A thermostat establishes
temperature while missing reaction heat remains missing. Existing kinetic records
expose their mechanism-specific domain and uncertainty; absent kinetics produces
an unsupported rate, not computed inertness or an invented zero.

`kero coverage observables FILE --json` executes a script through the ordinary
solver stack and reports the final per-vessel observable matrix. Parse/operation
errors and SolverFailed events fail the command.

**Ignition and gas boundaries.** Trial ignition cannot trigger bulk phase or
material transitions at the artificial spark temperature. Failed trials restore
complete state and discard trial physical events. Successful chemistry uses the
actual feed temperature in the feed/air enthalpy balance; the spark is an ignition
condition. Previously some magnesium/ethanol temperatures included artificial
bulk preheating. Tests and the magnesium lesson now reflect the physical feed
balance and explicitly distinguish the model's diluted-air temperature from a
calibrated local flame peak.

Only an open vessel admits room air. Sealed, finite pressure-controlled and swept
vessels preserve their inventory and report an unsupported energy/boundary route.
Their closed UV/HP or carrier-gas energy models have not been invented. Open ethanol
and methanol combustion now reach their existing supported routes; isopropanol
checks unsupported-trial rollback. Unresolved spectators retain their material
inventory and carry a missing-model disclosure.

## Independent matrix

The new tests exercise complete snapshot transactions, stale/edited proposals,
invalid successes, failures and declined MIX; finite interfaces and strict carbon
readback; 18 carbon dose/volume/boundary combinations through add/wait/heat/wait;
six drain quantities with unresolved oil; four alcohol fuel/dose combinations;
nine KNO3 concentration/amount cooling cases; and eight finite/swept ignition
boundary scenarios. Presentation contracts cover three locales, three support
statuses and three prose registers, persistence, scene agreement, instrument
routes and web confidence for both endpoints of a temperature difference.
They extend the original experiments rather than replacing them with corpus goldens.

The original case 43 prediction permits either ethanol combustion products/heat
or an explicit boundary. Its old checker only tested the boundary arm. The
separate `verify_systematic.py` accepts either original arm and adds a combustion
energy/product assertion. It does not edit the original checker or predictions.

## Final validation

830 distinct selected Rust tests passed: 652 core, 46 PHREEQC, 27 CEA,
104 CLI and one codex export snapshot. Repeated invocations are counted once,
using the final result for each target. All 113 lessons replayed without solver
failure, and their 33 observation/cache checks passed. The three-language transcript
and CLI JSON contracts passed. Web: 49 tests, zero type errors and two existing
warnings. Engine locale coverage is 726/726 keys in both German and French;
422 fill sites have no unfilled placeholders. Shell and public CLI provenance
gates both pass. The reviewed KNO3 approval now pins its existing transcription
and rights review and accepts only that exact data record; six mutation cases
reject broadened identities, origins, terms, outputs, code and checksums.

The final CLI ran 100 original JSON/text processes and 14 follow-up processes.
The five original unavailable inputs still refuse; all documented adaptations
and diagnostic follow-ups execute successfully. The physical assessment remains
49/50, with zinc kinetics the one unmet prediction. Supplemental combustion,
gas prose, rate-boundary and aqueous-layer assertions all pass. No original
prediction or script was edited. The binary hash, source hashes, per-target
counts and raw logs are in `systematic/validation-summary.json`,
`systematic/source-manifest.json` and `systematic/validation/`.

The initial broad CLI validation caught an incomplete KNO3 release record and
then its outdated source-count assertion. Those failures and the subsequent
passing rechecks are preserved. Numerical and presentation code remained unchanged
during the narrowly scoped provenance correction; the final CLI was replayed afterward.

## Remaining scientific scope

Zinc/acid rate predictions still require reviewed kinetics or a mixed-potential
model with surface-condition inputs and independent calibration. Missing complex
copper spectra and olive-oil identity/parameters remain missing. Five registry
adaptations in the original follow-ups are still explicit, including vegetable
oil as a surrogate rather than olive-specific validation. KNO3 cooling uses a
reviewed infinite-dilution datum, not finite-concentration calibration.

Closed combustion, sweep energy accounting, experimental flame-temperature
calibration, exhaustive production reservoir ledgers and reference-curve kinetic
validation remain further work. The bounded sequence matrix substantially improves
regression protection; it does not establish accuracy for every ingredient,
operator, phase, concentration or timescale. No runtime-speed improvement is
claimed without profiling.

## Resource handling

Three agents coordinated source/review work; only the root ran builds. Builds,
native tests and CLI runs used one worker. Active libraries and CLI remain on
`/mnt/volume1`; archives, logs and hash receipts are on `/mnt/storage`. Completed
session-generated test executables were reclaimed when disk space became tight.
Only clean, merged, inactive worktrees were consolidated; dirty or unmerged work
and branches were retained.

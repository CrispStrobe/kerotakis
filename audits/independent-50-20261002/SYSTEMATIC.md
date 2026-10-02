# Broader gaps exposed by the independent audit

The audit discovered several cross-cutting problems. The fixes address observed
failures; 49 passing checks are not evidence that these classes of failure are
closed throughout the engine. Priorities below describe proposed follow-up work,
not already implemented capabilities. Each workstream should keep independent
physical expectations separate from existing implementation snapshots.

## 1. Consistent state at solver and operator boundaries — highest priority

**Observed:** a canonical characterisation reported atmospheric carbon speciation
while the committed vessel retained a finite carbon dose. Scene and drainage also
used different representations of the same oil/water layers.

**Implemented:** an element-conservation guard on canonical recharacterisation,
carbon regression through addition/wait/heat, and shared unresolved-material
layers for the affected drainage path.

**Still needed:** an explicit solve-result contract that commits inventory,
interfaces, speciation, gas exchange, constraints and energy together. Merely
preserving elements cannot prove that the solution used the intended pressure,
reservoir, temperature or redox constraints. A declined canonical pass is a guard,
not a complete redesign of solver ownership.

**Systematic checks:** operation sequences across open/closed vessels, finite gas
doses/reservoirs, precipitation, heat, mix, drain and transfer. Check element and
resolved/unresolved mass totals with recorded inputs, outputs and reservoirs;
charge balance where the representation supports it; solution/species agreement;
and the selected boundary conditions. Include interfaces, solids and gas outlets
in the accounting. Use metamorphic properties only where physically applicable:
addition order after equilibrium, splitting/recombining compatible portions,
scaling extensive quantities, and save/reload equivalence. Order dependence in
kinetic or irreversible operations can be legitimate.

**Closure criterion:** every committing solver/operator boundary exposes the
same accounting contract, and bounded sequence tests exercise transitions between
solver routes rather than checking only isolated end states.

## 2. Model coverage and validity must belong to each observable

**Observed:** missing dissolution heat appeared as exactly zero heat, although
pH/speciation could still be calculated. pH/conductivity in an oil/water vessel
needed an aqueous-layer label.

**Implemented:** persistent missing-heat disclosure, propagation through relevant
transfers/persistence, temperature confidence in the web summary, and aqueous
measurement scope.

**Still needed:** a shared representation for each observable's status, reason,
phase scope, assumptions, data provenance and validity range. Distinguish a
computed zero, an unknown quantity, an estimate outside calibration, a bounded
aggregate model, and an unsupported route. Define how limitations propagate,
combine, and clear after a new measurement or imposed boundary condition. A
thermostat can establish temperature without making an unknown reaction enthalpy
known.

**Systematic checks:** a registry-to-observable coverage matrix, intentional
missing-data injection, and consistent JSON/CLI/scene/web/persistence behavior.
Audit other implicit-zero fallbacks, not just heat. Concentration, temperature,
ionic strength and phase limits should be exercised at and beyond their boundaries.

**Closure criterion:** unsupported or uncalibrated dependencies cannot silently
produce a fully confident observable; other supported observables remain useful.

## 3. Kinetics needs validation rather than binary thermodynamic gates

**Observed:** zinc in acid was classified as computed inertness using an
approximate fixed hydrogen-overpotential gate. The audit does not validate the
peroxide catalyst's rate constant either.

**Implemented:** explicit missing-rate reporting for the affected metal/acid
cases. The original zinc dissolution prediction remains unmet.

**Still needed:** decide which rates the app actually promises, then introduce
reviewed rate/mixed-potential models with their experimental domain. Surface area,
metal/surface condition, passivation, concentration, temperature and observation
time matter; available models may support only a subset. Missing inputs must
produce a stated boundary, not a fabricated absence of reaction.

**Systematic checks:** independent reference curves and limiting regimes, varying
amount and time separately, acid strength, surface area and passivation, including
both reacting and genuinely suppressed controls. Audit every claim of 'inert',
'slow', 'complete' and lesson-timescale completion for its model basis.

**Closure criterion:** rate claims have documented inputs, evidence and validity
limits; equilibrium possibility is not presented as an observed rate.

## 4. Trial actions and committed events need one transaction contract

**Observed:** unsuccessful ignition could retain trial vaporisation and emit
physical events inconsistent with the restored state.

**Implemented:** rollback of the unsuccessful ignition trial, removal of its
physical-transition events, and re-equilibration at the real temperature.

**Still needed:** review all speculative solves, failed actions, safety vetoes and
fallback paths. State rollback alone is insufficient if events, caches, scene
metadata or uncertainty flags leak from the trial.

**Systematic checks:** inject failures after each trial stage; compare complete
state before/after and verify event payloads against the committed delta. Explicit
ordinary chemistry that genuinely commits should remain distinguishable from
preview effects and diagnostic refusal events.

**Closure criterion:** trial state/events commit together or are discarded
together, across operators rather than only ignition.

## 5. Presentation and material operators need semantic consistency tests

**Observed:** gas electrolysis arithmetic was correct while prose claimed no
hydrogen co-evolution and referred to weighing a plated electrode. A visibly
layered vessel was called single-phase by its drain operator.

**Implemented:** process-specific gas explanations, matching material layers for
the affected drain, and scope notes in measurements.

**Still needed:** common capability/phase descriptions used by operators and
presentations. Review drain, decant, filter, extract and distil across represented
species, unresolved materials and mixed cases. Do not assume every visible layer
has a calibrated partition model.

**Systematic checks:** compare quantities, product identities, phase counts,
operation disposition and uncertainty claims across JSON, all prose registers,
locales, inspection and scenes. Localization/placeholder lints catch structure,
not chemical contradictions. Test unsupported mixtures and emulsions as well as
successful separations.

**Closure criterion:** presentations never assert stronger support than the
model; an operation's refusal describes the same physical state the user sees.

## 6. Registry completeness and independent calibration

**Observed:** two formula inputs and three olive-oil inputs were unavailable;
KNO3 heat, ethanol combustion and a complete Cu spectrum were gaps. Missing
registry identity, missing parameters and missing model routes are different
problems.

**Implemented:** a reviewed, traceable KNO3 datum and heat pathway. Five input
adaptations are explicit; ethanol combustion and the Cu spectrum remain bounded.

**Still needed:** prioritize a public capability matrix by ingredient, operation
and observable, with supported keys/aliases, data sources and limits. Extend
models only with reviewed evidence; one infinite-dilution heat datum does not
calibrate finite-concentration calorimetry. Independent reference suites should
cover concentration, temperature, phase boundaries and observation time, rather
than expanding only lesson-based golden snapshots.

**Closure criterion:** users can discover the supported domain before execution;
parameter promotion has dimensional, source, uncertainty and validity checks;
independent holdout experiments assess numerical accuracy within that domain.

## Execution order and resource limits

Start with the state/transaction contracts and observable coverage matrix: these
prevent confident-looking wrong answers across many features. Then extend the
cross-operator/material and cross-presentation matrix, followed by independently
validated kinetics and prioritized missing routes/data. Keep the original 50
predictions frozen as a holdout; new suites need separately declared expectations.

Use a small deterministic CI matrix and a larger scheduled matrix of bounded
operation sequences. Record seeds, failing scripts, solver route and accounting
residuals. Cap run time and workers; minimize a failure before expanding the
matrix. This host should retain single-job builds and single-worker runs while
memory is tight. Store bulk results/reference data on /mnt/storage and the active
CLI/libraries on /mnt/volume1. Profile a release build before selecting performance
optimizations: this audit supports correctness work, not a runtime-speed claim.

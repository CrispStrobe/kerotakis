//! Model orchestrator — transactional solver execution (ARCH-011).
//!
//! The orchestrator replaces the direct-mutation solver loop with a
//! transactional pipeline: plan → adapt → audit → commit.
//!
//! Each old `Equilibrator` is wrapped in a `DeltaAdapter` that runs the
//! solver on a cloned vessel, diffs the result into a `StateDelta`, and
//! commits atomically with conservation auditing.

use crate::delta::{StateDelta, ThermalDelta};
use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::Phase;
use crate::vessel::Vessel;

/// Errors from the orchestrator pipeline.
#[derive(Debug)]
pub enum OrchestratorError {
    Solver(SolveError),
    Conservation(Vec<crate::delta::DeltaError>),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorError::Solver(e) => write!(f, "solver error: {}", e),
            OrchestratorError::Conservation(errors) => {
                write!(f, "conservation errors: ")?;
                for e in errors {
                    write!(f, "{e}; ")?;
                }
                Ok(())
            }
        }
    }
}

impl From<SolveError> for OrchestratorError {
    fn from(e: SolveError) -> Self {
        OrchestratorError::Solver(e)
    }
}

/// Diff two vessel states to produce a StateDelta.
fn diff_vessels(before: &Vessel, after: &Vessel, source: &'static str) -> StateDelta {
    let mut delta = StateDelta::new(source);

    // Collect (species, phase) → moles for both states
    let before_amounts = species_amounts(before);
    let after_amounts = species_amounts(after);

    // Species that changed or appeared
    for ((species, phase), after_moles) in &after_amounts {
        let before_moles = before_amounts
            .get(&(species.clone(), *phase))
            .copied()
            .unwrap_or(0.0);
        let change = after_moles - before_moles;
        if change.abs() > 1e-15 {
            delta = delta.with_moles(
                crate::species::SpeciesId::new(species),
                *phase,
                change,
            );
        }
    }

    // Species that disappeared
    for ((species, phase), before_moles) in &before_amounts {
        if !after_amounts.contains_key(&(species.clone(), *phase)) && *before_moles > 1e-15 {
            delta = delta.with_moles(
                crate::species::SpeciesId::new(species),
                *phase,
                -*before_moles,
            );
        }
    }

    // Thermal change
    if (before.temperature.0 - after.temperature.0).abs() > 1e-15 {
        delta = delta.with_thermal(ThermalDelta::SetTemperature(after.temperature));
    }

    delta
}

fn species_amounts(vessel: &Vessel) -> std::collections::HashMap<(String, Phase), f64> {
    let mut amounts = std::collections::HashMap::new();
    for portion in &vessel.contents {
        *amounts
            .entry((portion.species.0.clone(), portion.phase))
            .or_insert(0.0) += portion.moles.0;
    }
    amounts
}

/// The model orchestrator: runs solvers through a transactional pipeline.
pub struct Orchestrator {
    solvers: Vec<Box<dyn Equilibrator>>,
    /// Conservation tolerance for chemistry solvers.
    pub conservation_tolerance: f64,
}

impl Orchestrator {
    pub fn new(solvers: Vec<Box<dyn Equilibrator>>) -> Self {
        Self {
            solvers,
            conservation_tolerance: 1e-7,
        }
    }

    /// Execute all applicable solvers on a vessel through the transactional
    /// pipeline: plan → adapt → audit → commit.
    ///
    /// Returns the concatenated events from all solvers.
    pub fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, OrchestratorError> {
        let mut all_events = Vec::new();

        for solver in &mut self.solvers {
            // Phase 1: Plan — check capability
            let cap = solver.capability(vessel);
            if !cap.applicability.is_applicable() {
                continue;
            }

            // Phase 2: Adapt
            // ARCH-012: prefer native delta path for migrated solvers
            let (delta, events) = if let Some(result) = solver.equilibrate_delta(vessel) {
                // Migrated solver — produces delta directly, no clone needed
                match result {
                    Ok(pair) => pair,
                    Err(e) => {
                        all_events.push(Event::SolverFailed {
                            vessel: vessel.id,
                            solver: solver.name().to_string(),
                            detail: e.to_string(),
                        });
                        continue;
                    }
                }
            } else {
                // Legacy solver — clone-mutate-diff adaptation path
                let mut copy = vessel.clone();
                copy.solution = None;
                let events = match solver.equilibrate(&mut copy) {
                    Ok(events) => events,
                    Err(e) => {
                        all_events.push(Event::SolverFailed {
                            vessel: vessel.id,
                            solver: solver.name().to_string(),
                            detail: e.to_string(),
                        });
                        continue;
                    }
                };
                (diff_vessels(vessel, &copy, solver.name()), events)
            };

            if delta.is_empty() {
                // Solver ran but made no changes — still report events
                all_events.extend(events);
                continue;
            }

            // Phase 3+4: Audit + Commit
            if cap.is_chemistry {
                match delta.commit_conserved(vessel, self.conservation_tolerance) {
                    Ok(()) => all_events.extend(events),
                    Err(errors) => {
                        all_events.push(Event::SolverFailed {
                            vessel: vessel.id,
                            solver: solver.name().to_string(),
                            detail: format!("conservation violation: {:?}", errors),
                        });
                    }
                }
            } else {
                match delta.commit(vessel) {
                    Ok(()) => all_events.extend(events),
                    Err(errors) => {
                        all_events.push(Event::SolverFailed {
                            vessel: vessel.id,
                            solver: solver.name().to_string(),
                            detail: format!("commit failed: {:?}", errors),
                        });
                    }
                }
            }
        }

        Ok(all_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve::{MixingEquilibrator, HonestyEquilibrator};
    use crate::species::SpeciesId;
    use crate::units::{Kelvin, Moles};
    use crate::vessel::VesselId;

    fn water_vessel() -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin(298.15);
        v.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
        v
    }

    #[test]
    fn orchestrator_runs_mixing_on_water() {
        let mut v = water_vessel();
        let mut orch = Orchestrator::new(vec![
            Box::new(MixingEquilibrator),
            Box::new(HonestyEquilibrator),
        ]);

        let events = orch.equilibrate(&mut v).unwrap();
        // MixingEquilibrator produces no events for a single pure substance;
        // HonestyEquilibrator reports the situation.
        // The key test is that the orchestrator runs without error and
        // the vessel state is reasonable.
        assert!(v.temperature.0 > 0.0);
        assert!(!v.contents.is_empty());
        // No solver failures
        assert!(
            !events.iter().any(|e| matches!(e, Event::SolverFailed { .. })),
            "unexpected solver failure: {:?}",
            events
        );
    }

    #[test]
    fn orchestrator_preserves_mass_through_mixing() {
        let mut v = water_vessel();
        v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
        let mass_before = v.mass();

        let mut orch = Orchestrator::new(vec![Box::new(MixingEquilibrator)]);
        orch.equilibrate(&mut v).unwrap();

        let mass_after = v.mass();
        assert!(
            (mass_before.0 - mass_after.0).abs() < 1e-10,
            "mass changed: {} -> {}",
            mass_before.0,
            mass_after.0
        );
    }

    #[test]
    fn diff_vessels_detects_species_change() {
        let before = water_vessel();
        let mut after = before.clone();
        after.deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Aqueous);

        let delta = diff_vessels(&before, &after, "test");
        assert!(!delta.is_empty());
        assert!(
            delta.net_moles(&SpeciesId::new("NaCl"), Phase::Aqueous) > 0.0
        );
    }

    #[test]
    fn diff_vessels_detects_temperature_change() {
        let before = water_vessel();
        let mut after = before.clone();
        after.temperature = Kelvin(373.15);

        let delta = diff_vessels(&before, &after, "test");
        assert!(!delta.is_empty());
        assert!(delta.thermal.is_some());
    }

    #[test]
    fn diff_of_identical_vessels_is_empty() {
        let v = water_vessel();
        let delta = diff_vessels(&v, &v, "test");
        assert!(delta.is_empty());
    }

    // ── ARCH-012: migration tests ─────────────────────────────────

    #[test]
    fn mixing_produces_native_delta() {
        let mut v = water_vessel();
        v.thermal_mode = crate::vessel::ThermalMode::Thermostatted(Kelvin(310.0));
        v.temperature = Kelvin(298.15);

        let mut mixing = MixingEquilibrator;
        let result = mixing.equilibrate_delta(&v);
        assert!(result.is_some(), "mixing should return a delta");

        let (delta, events) = result.unwrap().unwrap();
        assert!(!delta.is_empty(), "delta should have a thermal change");
        assert!(delta.thermal.is_some());
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], Event::TemperatureChanged { .. }));
    }

    #[test]
    fn honesty_produces_native_delta() {
        let mut v = water_vessel();
        v.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);

        let mut honesty = HonestyEquilibrator;
        let result = honesty.equilibrate_delta(&v);
        assert!(result.is_some(), "honesty should return a delta");

        let (delta, events) = result.unwrap().unwrap();
        assert!(delta.is_empty(), "honesty should produce no mutations");
        // Should report the unmodeled solid
        assert!(
            events.iter().any(|e| matches!(e, Event::NotYetModeled { .. })),
            "honesty should report unmodeled solids"
        );
    }

    #[test]
    fn orchestrator_uses_native_delta_path() {
        // Thermostatted vessel: mixing should change temperature via native delta
        let mut v = water_vessel();
        v.thermal_mode = crate::vessel::ThermalMode::Thermostatted(Kelvin(310.0));
        v.temperature = Kelvin(298.15);

        let mut orch = Orchestrator::new(vec![
            Box::new(MixingEquilibrator),
            Box::new(HonestyEquilibrator),
        ]);

        let events = orch.equilibrate(&mut v).unwrap();
        // Temperature should have changed to the thermostat value
        assert!(
            (v.temperature.0 - 310.0).abs() < 1e-10,
            "temperature should be 310K, got {}",
            v.temperature.0
        );
        // Should have a TemperatureChanged event
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::TemperatureChanged { .. })),
            "expected TemperatureChanged event: {:?}",
            events
        );
        // No solver failures
        assert!(
            !events.iter().any(|e| matches!(e, Event::SolverFailed { .. })),
            "unexpected solver failure: {:?}",
            events
        );
    }
}

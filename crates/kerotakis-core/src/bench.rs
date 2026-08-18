//! The bench: a set of vessels, the operator log, and the step loop
//! (operator → L0 → apply → re-equilibrate → events).

use serde::{Deserialize, Serialize};

use crate::ops::{Event, Instrument, LogEntry, Operator};
use crate::solve::{
    adiabatic_mix_temperature, Equilibrator, MixingEquilibrator, PermissiveScreen, SafetyScreen,
};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Moles};
use crate::vessel::{ThermalMode, Vessel, VesselId};

#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error("no vessel {0}")]
    NoSuchVessel(VesselId),
    #[error("unknown species '{0}' — not in the registry")]
    UnknownSpecies(SpeciesId),
    #[error("amount must be positive")]
    NonPositiveAmount,
    #[error("fraction must be within 0..=1")]
    BadFraction,
    #[error("source and target vessel are the same")]
    SelfTransfer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bench {
    pub vessels: Vec<Vessel>,
    pub log: Vec<LogEntry>,
}

impl Default for Bench {
    fn default() -> Self {
        Self::new()
    }
}

impl Bench {
    /// A bench starts with one empty vessel — there is always something to
    /// pour into.
    pub fn new() -> Self {
        Bench {
            vessels: vec![Vessel::new(VesselId(0), "beaker")],
            log: Vec::new(),
        }
    }

    pub fn vessel(&self, id: VesselId) -> Result<&Vessel, BenchError> {
        self.vessels
            .iter()
            .find(|v| v.id == id)
            .ok_or(BenchError::NoSuchVessel(id))
    }

    fn vessel_mut(&mut self, id: VesselId) -> Result<&mut Vessel, BenchError> {
        self.vessels
            .iter_mut()
            .find(|v| v.id == id)
            .ok_or(BenchError::NoSuchVessel(id))
    }

    /// Run one operator through the full loop with the default v0 solver and
    /// screen. The returned events are also appended to the log.
    pub fn step(&mut self, op: Operator) -> Result<Vec<Event>, BenchError> {
        self.step_with(op, &MixingEquilibrator, &PermissiveScreen)
    }

    /// Run one operator with explicit solver and safety screen.
    pub fn step_with(
        &mut self,
        op: Operator,
        solver: &dyn Equilibrator,
        screen: &dyn SafetyScreen,
    ) -> Result<Vec<Event>, BenchError> {
        let mut events = self.apply(&op, screen)?;

        // Re-equilibrate every vessel the operator touched (v0: mutating ops
        // touch at most two).
        for id in op_touches(&op) {
            let vessel = self.vessel_mut(id)?;
            match solver.equilibrate(vessel) {
                Ok(mut more) => events.append(&mut more),
                Err(e) => events.push(Event::SolverFailed {
                    vessel: id,
                    solver: solver.name().to_string(),
                    detail: e.to_string(),
                }),
            }
        }

        self.log.push(LogEntry {
            step: self.log.len(),
            operator: op,
            events: events.clone(),
        });
        Ok(events)
    }

    fn apply(
        &mut self,
        op: &Operator,
        screen: &dyn SafetyScreen,
    ) -> Result<Vec<Event>, BenchError> {
        let mut events = Vec::new();
        match op {
            Operator::NewVessel => {
                let id = VesselId(self.vessels.iter().map(|v| v.id.0 + 1).max().unwrap_or(0));
                self.vessels.push(Vessel::new(id, "beaker"));
                events.push(Event::VesselCreated { vessel: id });
            }
            Operator::Add {
                vessel,
                species: sid,
                moles,
                at,
            } => {
                if moles.0 <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let data =
                    species::lookup(sid).ok_or_else(|| BenchError::UnknownSpecies(sid.clone()))?;

                // L0 runs against the prospective state, before mutation.
                let mut probe = self.vessel(*vessel)?.clone();
                probe.deposit(sid.clone(), *moles, data.standard_phase);
                if let Some(reason) = screen.veto(&probe) {
                    events.push(Event::SafetyVeto { reason });
                    return Ok(events);
                }

                let t_in = at.unwrap_or(Kelvin::STANDARD);
                let cp_in = moles.0 * data.heat_capacity;
                let v = self.vessel_mut(*vessel)?;
                if matches!(v.thermal_mode, ThermalMode::Adiabatic) {
                    let t_new =
                        adiabatic_mix_temperature(v.temperature, v.heat_capacity(), t_in, cp_in);
                    if (t_new.0 - v.temperature.0).abs() > 1e-9 {
                        events.push(Event::TemperatureChanged {
                            vessel: v.id,
                            from: v.temperature,
                            to: t_new,
                        });
                    }
                    v.temperature = t_new;
                }
                v.deposit(sid.clone(), *moles, data.standard_phase);
                events.push(Event::Added {
                    vessel: *vessel,
                    species: sid.clone(),
                    moles: *moles,
                });
            }
            Operator::Heat { vessel, energy } | Operator::Cool { vessel, energy } => {
                if energy.0 < 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let signed = if matches!(op, Operator::Cool { .. }) {
                    -energy.0
                } else {
                    energy.0
                };
                let v = self.vessel_mut(*vessel)?;
                let cp = v.heat_capacity();
                if cp > 0.0 {
                    let from = v.temperature;
                    let to = Kelvin((from.0 + signed / cp).max(0.0));
                    v.temperature = to;
                    events.push(Event::TemperatureChanged {
                        vessel: *vessel,
                        from,
                        to,
                    });
                } else {
                    events.push(Event::NotYetModeled {
                        vessel: *vessel,
                        what: "heating an empty vessel (container heat capacity not modelled)"
                            .to_string(),
                    });
                }
            }
            Operator::Stir { vessel } => {
                let v = self.vessel(*vessel)?;
                events.push(Event::NotYetModeled {
                    vessel: v.id,
                    what:
                        "stirring changes nothing the current solvers model (kinetics arrive in P5)"
                            .to_string(),
                });
            }
            Operator::Decant { from, to, fraction } => {
                if !(0.0..=1.0).contains(fraction) {
                    return Err(BenchError::BadFraction);
                }
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                // Take the liquid fraction out of `from`…
                let (portions, t_from) = {
                    let src = self.vessel_mut(*from)?;
                    let mut moved = Vec::new();
                    for p in src.contents.iter_mut() {
                        if matches!(p.phase, Phase::Liquid | Phase::Aqueous) {
                            let n = Moles(p.moles.0 * fraction);
                            p.moles = Moles(p.moles.0 - n.0);
                            if n.0 > 0.0 {
                                moved.push((p.species.clone(), n, p.phase));
                            }
                        }
                    }
                    src.contents.retain(|p| p.moles.0 > 1e-15);
                    (moved, src.temperature)
                };
                // …and mix it into `to` with the energy balance.
                let cp_in: f64 = portions
                    .iter()
                    .filter_map(|(s, n, _)| species::lookup(s).map(|d| n.0 * d.heat_capacity))
                    .sum();
                let dst = self.vessel_mut(*to)?;
                if matches!(dst.thermal_mode, ThermalMode::Adiabatic) {
                    let t_new = adiabatic_mix_temperature(
                        dst.temperature,
                        dst.heat_capacity(),
                        t_from,
                        cp_in,
                    );
                    if !portions.is_empty() && (t_new.0 - dst.temperature.0).abs() > 1e-9 {
                        events.push(Event::TemperatureChanged {
                            vessel: *to,
                            from: dst.temperature,
                            to: t_new,
                        });
                    }
                    dst.temperature = t_new;
                }
                for (s, n, phase) in portions {
                    dst.deposit(s, n, phase);
                }
                events.push(Event::Transferred {
                    from: *from,
                    to: *to,
                    fraction: *fraction,
                });
            }
            Operator::Measure { vessel, instrument } => {
                let v = self.vessel(*vessel)?;
                let (value, unit) = match instrument {
                    Instrument::Thermometer => (v.temperature.to_celsius(), "°C".to_string()),
                    Instrument::Balance => (v.mass().0, "g".to_string()),
                };
                events.push(Event::Measured {
                    vessel: *vessel,
                    instrument: *instrument,
                    value,
                    unit,
                });
            }
        }
        Ok(events)
    }

    /// Total moles of one species across the whole bench.
    pub fn total_moles(&self, species: &SpeciesId) -> Moles {
        Moles(self.vessels.iter().map(|v| v.moles_of(species).0).sum())
    }

    /// Total sensible enthalpy across the bench, J (see `Vessel::enthalpy`).
    pub fn total_enthalpy(&self) -> Joules {
        Joules(self.vessels.iter().map(|v| v.enthalpy().0).sum())
    }
}

/// Which vessels an operator touches (for re-equilibration).
fn op_touches(op: &Operator) -> Vec<VesselId> {
    match op {
        Operator::NewVessel => vec![],
        Operator::Add { vessel, .. }
        | Operator::Heat { vessel, .. }
        | Operator::Cool { vessel, .. }
        | Operator::Stir { vessel } => vec![*vessel],
        Operator::Decant { from, to, .. } => vec![*from, *to],
        Operator::Measure { .. } => vec![],
    }
}

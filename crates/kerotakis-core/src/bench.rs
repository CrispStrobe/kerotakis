//! The bench: a set of vessels, the operator log, and the step loop
//! (operator → L0 → apply → re-equilibrate → events).

use serde::{Deserialize, Serialize};

use crate::instrument::InstrumentContract;
use crate::ops::{ElutedPeak, Event, Instrument, LogEntry, Operator};
use crate::solve::{
    adiabatic_mix_temperature, Equilibrator, HonestyEquilibrator, MixingEquilibrator,
    PermissiveScreen, SafetyScreen, SafetyVerdict, SolverStack,
};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Liters, Moles, Pascal};
use crate::vessel::{Headspace, ThermalMode, Vessel, VesselId};

/// The temperature a match or spark brings its immediate surroundings to.
pub const IGNITION_K: f64 = 1200.0;

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
    #[error(transparent)]
    Kinetics(#[from] crate::kinetics::IntegrationError),
    #[error(transparent)]
    Transport(#[from] crate::transport::TransportError),
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

    /// Run one operator through the full loop with the default solver stack
    /// (physics + honesty, no chemistry engines) and a permissive screen.
    /// The returned events are also appended to the log.
    pub fn step(&mut self, op: Operator) -> Result<Vec<Event>, BenchError> {
        let mut default_stack = SolverStack::new(vec![
            Box::new(MixingEquilibrator),
            Box::new(crate::nonaqueous::NonAqueousEquilibrator),
            Box::new(crate::hmix::MixingEnthalpyEquilibrator),
            Box::new(HonestyEquilibrator),
        ]);
        self.step_with(op, &mut default_stack, &PermissiveScreen)
    }

    /// Run one operator with explicit solver and safety screen.
    pub fn step_with(
        &mut self,
        op: Operator,
        solver: &mut dyn Equilibrator,
        screen: &dyn SafetyScreen,
    ) -> Result<Vec<Event>, BenchError> {
        if let Operator::Titrate { .. } = &op {
            return self.titrate_loop(op, solver, screen);
        }
        let temperature_before = match &op {
            Operator::Ignite { vessel } => self.vessel(*vessel)?.temperature,
            _ => Kelvin::STANDARD,
        };
        // Snapshot source vessels before apply for MIX routing.
        let mix_sources = match &op {
            Operator::Mix {
                a,
                b,
                into,
                fraction_a,
                fraction_b,
            } => {
                let snap_a = self.vessel(*a)?.clone();
                let snap_b = self.vessel(*b)?.clone();
                Some((*into, snap_a, *fraction_a, snap_b, *fraction_b))
            }
            _ => None,
        };
        let mut events = self.apply(&op, screen)?;
        // Waiting advances the whole bench, so every vessel is re-settled.
        let touched: Vec<VesselId> = match &op {
            Operator::Wait { .. } => self.vessels.iter().map(|v| v.id).collect(),
            _ => op_touches(&op),
        };

        // Re-equilibrate every vessel the operator touched (v0: mutating ops
        // touch at most two). A touched vessel's previous solution
        // characterisation is stale by definition; the solver stack either
        // recomputes it or the honesty pass reports the gap.
        for id in touched.iter().copied() {
            let vessel = self.vessel_mut(id)?;
            vessel.solution = None;
            // For MIX, try native solver mixing on the target vessel.
            if let Some((mix_into, ref snap_a, frac_a, ref snap_b, frac_b)) = mix_sources {
                if id == mix_into {
                    if let Some(result) = solver.mix(vessel, snap_a, frac_a, snap_b, frac_b) {
                        match result {
                            Ok(mut more) => {
                                events.append(&mut more);
                                vessel.refresh_pressure();
                                continue;
                            }
                            Err(_) => {
                                // MIX failed; fall through to normal equilibrate.
                            }
                        }
                    }
                }
            }
            if solver.applies(vessel) {
                match solver.equilibrate(vessel) {
                    Ok(mut more) => events.append(&mut more),
                    Err(e) => events.push(Event::SolverFailed {
                        vessel: id,
                        solver: solver.name().to_string(),
                        detail: e.to_string(),
                    }),
                }
            }
            vessel.refresh_pressure();
        }

        // A temperature announced mid-step may be overtaken by a later
        // solver: a phase change pins the vessel at its transition point,
        // so "cooled to -71 C" becomes false when the water froze at 0 C
        // and stayed there. Correct the last reading per vessel to the
        // temperature the vessel actually ended at, and drop it if nothing
        // moved after all.
        for id in touched.iter().copied() {
            let Ok(actual) = self.vessel(id).map(|v| v.temperature) else {
                continue;
            };
            let last = events.iter().rposition(
                |e| matches!(e, Event::TemperatureChanged { vessel, .. } if *vessel == id),
            );
            if let Some(i) = last {
                if let Event::TemperatureChanged { from, to, .. } = &mut events[i] {
                    *to = actual;
                    let stale = (from.0 - to.0).abs() < 0.01;
                    if stale {
                        events.remove(i);
                    }
                }
            }
        }

        // A spark held to something that will not burn leaves nothing
        // behind: put the vessel back as it was, and say so.
        if let Operator::Ignite { vessel } = &op {
            let caught = events.iter().any(|e| match e {
                Event::Consumed { moles, .. }
                | Event::GasEvolved { moles, .. }
                | Event::GasContained { moles, .. }
                | Event::Precipitated { moles, .. } => moles.0 >= crate::OBSERVABLE_MOLES,
                Event::ReactionOccurred { .. } => true,
                _ => false,
            });
            // Asked *before* the revert, while the vessel is still at flame
            // temperature: that is the state whose flammability was — or
            // was not — evaluated.
            let examined = self
                .vessel(*vessel)
                .map(|v| solver.chemistry_applies(v))
                .unwrap_or(false);
            if !caught {
                if let Ok(v) = self.vessel_mut(*vessel) {
                    v.temperature = temperature_before;
                }
                events.retain(|e| {
                    !matches!(
                        e,
                        Event::Ignited { .. }
                            | Event::TemperatureChanged { .. }
                            | Event::ThermalEquilibrium { .. }
                    )
                });
                // It would not burn — but a metal salt still colours the
                // flame, which is the flame test and worth seeing.
                let painted = self.vessel(*vessel).ok().and_then(|v| {
                    v.contents.iter().find_map(|p| {
                        species::lookup(&p.species)
                            .and_then(|d| d.flame_colour)
                            .map(|c| (p.species.clone(), c.to_string()))
                    })
                });
                match painted {
                    Some((species, colour)) => events.push(Event::FlameTest {
                        vessel: *vessel,
                        species,
                        colour,
                    }),
                    // "Nothing ignited" is a claim about the substance, and
                    // we may only make it if an engine actually looked.
                    // Ethanol has no condensed form in the NASA data, so
                    // the thermal solver never engages — and reporting that
                    // silence as "it does not burn" would be a false
                    // observation dressed as a result.
                    None if !examined => events.push(Event::NotYetModeled {
                        vessel: *vessel,
                        what: "whether these contents burn: no wired solver models combustion for them, so the lab cannot say either way".to_string(),
                    }),
                    None => events.push(Event::DidNotIgnite { vessel: *vessel }),
                }
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
            Operator::NewVessel { kind } => {
                let label = kind.as_deref().unwrap_or("beaker");
                let id = VesselId(self.vessels.iter().map(|v| v.id.0 + 1).max().unwrap_or(0));
                self.vessels.push(Vessel::new(id, label));
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
                match screen.assess(&probe) {
                    SafetyVerdict::Allow => {}
                    SafetyVerdict::Warn {
                        severity,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
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
                    let wanted = from.0 + signed / cp;
                    // A vessel can only give up the heat it has. Clamping
                    // at absolute zero and saying nothing let a request for
                    // more be silently granted: two grams of magnesia at
                    // 2769 °C, asked for 10 kJ it did not hold, came back
                    // at exactly −273.15 °C as though that were an answer.
                    //
                    // No coolant is modelled — `cool` removes energy without
                    // a reservoir to remove it into — so the only bound
                    // available is the vessel's own heat content, and the
                    // bench has to say when a request runs past it. That
                    // this bound is absolute zero is itself the tell: long
                    // before it, constant heat capacities have stopped
                    // describing anything, since every Cp here is a room-
                    // temperature figure treated as temperature-independent.
                    let to = Kelvin(wanted.max(0.0));
                    v.temperature = to;
                    events.push(Event::TemperatureChanged {
                        vessel: *vessel,
                        from,
                        to,
                    });
                    if wanted < 0.0 {
                        let could_pay = cp * from.0 / 1000.0;
                        events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: format!(
                                "this vessel had only {could_pay:.2} kJ to give up before absolute \
                                 zero, and {:.2} kJ were asked of it. No coolant \
                                 is modelled here — nothing sets how cold the \
                                 surroundings are — so the rest simply could not \
                                 be removed. The heat capacities are room-\
                                 temperature values held constant, which stops \
                                 being true long before this",
                                energy.0 / 1000.0,
                            ),
                        });
                    }
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
                        "stirring changes nothing this lab models: rates depend on concentration, temperature and catalysts here, and mixing and surface area are not modelled at all"
                            .to_string(),
                });
            }
            Operator::Seal {
                vessel,
                headspace_volume,
            } => {
                if headspace_volume.0 <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let v = self.vessel_mut(*vessel)?;
                let previous = v.headspace;
                let source_pressure = v.pressure;
                v.headspace = Headspace::Sealed {
                    volume: *headspace_volume,
                };
                let trapped = trap_boundary_gas(v, previous, *headspace_volume, source_pressure);
                v.refresh_pressure();
                events.push(Event::VesselSealed {
                    vessel: *vessel,
                    headspace_volume: *headspace_volume,
                    trapped_air: trapped,
                });
            }
            Operator::Regulate {
                vessel,
                pressure,
                initial_volume,
            } => {
                if pressure.0 <= 0.0 || initial_volume.0 <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let v = self.vessel_mut(*vessel)?;
                let previous = v.headspace;
                v.headspace = Headspace::PressureControlled {
                    pressure: *pressure,
                    volume: *initial_volume,
                };
                let trapped = trap_boundary_gas(v, previous, *initial_volume, *pressure);
                v.refresh_pressure();
                events.push(Event::VesselPressureControlled {
                    vessel: *vessel,
                    pressure: *pressure,
                    initial_volume: *initial_volume,
                    trapped_gas: trapped,
                });
            }
            Operator::Sweep { vessel, pressure } => {
                if pressure.0 <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let v = self.vessel_mut(*vessel)?;
                let gases = vent_headspace(v);
                v.headspace = Headspace::Swept {
                    pressure: *pressure,
                };
                v.refresh_pressure();
                events.push(Event::VesselSwept {
                    vessel: *vessel,
                    pressure: *pressure,
                });
                for (species, moles) in gases {
                    events.push(Event::GasEvolved {
                        vessel: *vessel,
                        species,
                        moles,
                    });
                }
            }
            Operator::Open { vessel } => {
                let v = self.vessel_mut(*vessel)?;
                let gases = vent_headspace(v);
                v.headspace = Headspace::Open;
                v.refresh_pressure();
                events.push(Event::VesselOpened { vessel: *vessel });
                for (species, moles) in gases {
                    events.push(Event::GasEvolved {
                        vessel: *vessel,
                        species,
                        moles,
                    });
                }
            }
            Operator::Decant { from, to, fraction } => {
                if !(0.0..=1.0).contains(fraction) {
                    return Err(BenchError::BadFraction);
                }
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                // Work out what would move, without mutating yet.
                let (would_move, t_from) = {
                    let src = self.vessel(*from)?;
                    let moved: Vec<_> = src
                        .contents
                        .iter()
                        .filter(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous))
                        .filter_map(|p| {
                            let n = Moles(p.moles.0 * fraction);
                            (n.0 > 0.0).then(|| (p.species.clone(), n, p.phase))
                        })
                        .collect();
                    (moved, src.temperature)
                };

                // L0 on the prospective target state, before mutation —
                // pouring one vessel into another can create the hazard.
                let mut probe = self.vessel(*to)?.clone();
                for (s, n, phase) in &would_move {
                    probe.deposit(s.clone(), *n, *phase);
                }
                match screen.assess(&probe) {
                    SafetyVerdict::Allow => {}
                    SafetyVerdict::Warn {
                        severity,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // Apply: take the liquid fraction out of `from`…
                let portions = {
                    let src = self.vessel_mut(*from)?;
                    for p in src.contents.iter_mut() {
                        if matches!(p.phase, Phase::Liquid | Phase::Aqueous) {
                            p.moles = Moles(p.moles.0 * (1.0 - fraction));
                        }
                    }
                    src.contents.retain(|p| p.moles.0 > 1e-15);
                    would_move
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
            Operator::Mix {
                a,
                b,
                into,
                fraction_a,
                fraction_b,
            } => {
                if !(0.0..=1.0).contains(fraction_a) || !(0.0..=1.0).contains(fraction_b) {
                    return Err(BenchError::BadFraction);
                }
                if a == into || b == into {
                    return Err(BenchError::SelfTransfer);
                }
                if a == b {
                    return Err(BenchError::SelfTransfer);
                }
                // Gather what would move from each source.
                let (move_a, t_a) = {
                    let src = self.vessel(*a)?;
                    let moved: Vec<_> = src
                        .contents
                        .iter()
                        .filter(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous))
                        .filter_map(|p| {
                            let n = Moles(p.moles.0 * fraction_a);
                            (n.0 > 0.0).then(|| (p.species.clone(), n, p.phase))
                        })
                        .collect();
                    (moved, src.temperature)
                };
                let (move_b, t_b) = {
                    let src = self.vessel(*b)?;
                    let moved: Vec<_> = src
                        .contents
                        .iter()
                        .filter(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous))
                        .filter_map(|p| {
                            let n = Moles(p.moles.0 * fraction_b);
                            (n.0 > 0.0).then(|| (p.species.clone(), n, p.phase))
                        })
                        .collect();
                    (moved, src.temperature)
                };

                // L0 on the prospective target state.
                let mut probe = self.vessel(*into)?.clone();
                for (s, n, phase) in move_a.iter().chain(move_b.iter()) {
                    probe.deposit(s.clone(), *n, *phase);
                }
                match screen.assess(&probe) {
                    SafetyVerdict::Allow => {}
                    SafetyVerdict::Warn {
                        severity,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // Withdraw fractions from sources.
                {
                    let src_a = self.vessel_mut(*a)?;
                    for p in src_a.contents.iter_mut() {
                        if matches!(p.phase, Phase::Liquid | Phase::Aqueous) {
                            p.moles = Moles(p.moles.0 * (1.0 - fraction_a));
                        }
                    }
                    src_a.contents.retain(|p| p.moles.0 > 1e-15);
                    src_a.solution = None;
                }
                {
                    let src_b = self.vessel_mut(*b)?;
                    for p in src_b.contents.iter_mut() {
                        if matches!(p.phase, Phase::Liquid | Phase::Aqueous) {
                            p.moles = Moles(p.moles.0 * (1.0 - fraction_b));
                        }
                    }
                    src_b.contents.retain(|p| p.moles.0 > 1e-15);
                    src_b.solution = None;
                }

                // Deposit into target with adiabatic energy balance.
                let cp_a: f64 = move_a
                    .iter()
                    .filter_map(|(s, n, _)| species::lookup(s).map(|d| n.0 * d.heat_capacity))
                    .sum();
                let cp_b: f64 = move_b
                    .iter()
                    .filter_map(|(s, n, _)| species::lookup(s).map(|d| n.0 * d.heat_capacity))
                    .sum();
                let dst = self.vessel_mut(*into)?;
                if matches!(dst.thermal_mode, ThermalMode::Adiabatic) {
                    // Three-body adiabatic mix: vessel + stream_a + stream_b.
                    let cp_dst = dst.heat_capacity();
                    let total_cp = cp_dst + cp_a + cp_b;
                    if total_cp > 0.0 {
                        let t_new = Kelvin(
                            (cp_dst * dst.temperature.0 + cp_a * t_a.0 + cp_b * t_b.0) / total_cp,
                        );
                        if (t_new.0 - dst.temperature.0).abs() > 1e-9 {
                            events.push(Event::TemperatureChanged {
                                vessel: *into,
                                from: dst.temperature,
                                to: t_new,
                            });
                        }
                        dst.temperature = t_new;
                    }
                }
                for (s, n, phase) in move_a.into_iter().chain(move_b) {
                    dst.deposit(s, n, phase);
                }
                events.push(Event::Mixed {
                    a: *a,
                    b: *b,
                    into: *into,
                    fraction_a: *fraction_a,
                    fraction_b: *fraction_b,
                });
            }
            Operator::Filter { from, to } => {
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                // Everything liquid + dissolved would move; probe the target.
                let (would_move, t_from) = {
                    let src = self.vessel(*from)?;
                    let moved: Vec<_> = src
                        .contents
                        .iter()
                        .filter(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous))
                        .map(|p| (p.species.clone(), p.moles, p.phase))
                        .collect();
                    (moved, src.temperature)
                };
                let mut probe = self.vessel(*to)?.clone();
                for (s, n, phase) in &would_move {
                    probe.deposit(s.clone(), *n, *phase);
                }
                match screen.assess(&probe) {
                    SafetyVerdict::Allow => {}
                    SafetyVerdict::Warn {
                        severity,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                let src = self.vessel_mut(*from)?;
                src.contents.retain(|p| p.phase == Phase::Solid);
                let cp_in: f64 = would_move
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
                    dst.temperature = t_new;
                }
                for (s, n, phase) in would_move {
                    dst.deposit(s, n, phase);
                }
                events.push(Event::Filtered {
                    from: *from,
                    to: *to,
                });
            }
            Operator::Ignite { vessel } => {
                let v = self.vessel_mut(*vessel)?;
                if v.is_empty() {
                    events.push(Event::DidNotIgnite { vessel: *vessel });
                } else {
                    // A match brings a small volume to flame temperature.
                    // Whether anything catches is for the solvers to say;
                    // if nothing does, `step_with` puts the spark back out.
                    let from = v.temperature;
                    if from.0 < IGNITION_K {
                        v.temperature = Kelvin(IGNITION_K);
                    }
                    let flame = v
                        .contents
                        .iter()
                        .filter_map(|p| species::lookup(&p.species))
                        .find_map(|d| d.flame_colour)
                        .map(str::to_string);
                    events.push(Event::Ignited {
                        vessel: *vessel,
                        flame,
                    });
                }
            }
            Operator::Evaporate { vessel, fraction } => {
                if !(0.0..=1.0).contains(fraction) {
                    return Err(BenchError::BadFraction);
                }
                let v = self.vessel_mut(*vessel)?;
                let water = SpeciesId::new("water");
                let present = v.moles_of(&water);
                if present.0 <= 0.0 {
                    events.push(Event::NotYetModeled {
                        vessel: *vessel,
                        what: "nothing to evaporate — no water in the vessel".to_string(),
                    });
                } else {
                    let removed = v.withdraw(&water, Moles(present.0 * fraction));
                    events.push(Event::Evaporated {
                        vessel: *vessel,
                        moles: removed,
                    });
                    // Taking the last of the solvent leaves dissolved
                    // matter with nothing to be dissolved in.
                    //
                    // The mass is right and the chemistry is not: 0.1 mol of
                    // chloride ion, labelled Aqueous, in a beaker holding no
                    // water. Nothing catches it either, because with no
                    // solvent left no aqueous solver applies — so unlike
                    // evaporating to 99%, where all three databases refuse
                    // the 100 mol/kgw brine out loud, the impossible state
                    // arrives in silence.
                    //
                    // The bench cannot fix it by crystallising, because
                    // which solids form is not decidable from the ions
                    // alone: sodium, potassium, chloride and nitrate in one
                    // beaker can dry to more than one set of salts, and
                    // guessing one would be inventing a result. So it says
                    // what it is holding and what it cannot decide.
                    let dry = v.moles_of(&water).0 <= 0.0;
                    let stranded: Vec<&str> = v
                        .contents
                        .iter()
                        .filter(|p| p.phase == Phase::Aqueous)
                        .filter_map(|p| species::lookup(&p.species).map(|d| d.name))
                        .collect();
                    if dry && !stranded.is_empty() {
                        events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: format!(
                                "the last of the water is gone and {} are still shown as \
                                 dissolved, which is not a state a beaker can be in. What \
                                 they crystallise into is not decidable from the ions alone, \
                                 so the bench will not guess at the solids",
                                stranded.join(", ")
                            ),
                        });
                    }
                    // No energy is charged for the vaporisation, and that
                    // is decided rather than forgotten: `evaporate` means
                    // the dish is on a hotplate, and the ~40.7 kJ/mol comes
                    // from outside the ledger as it does in the lab.
                    // Charging for it without modelling the burner would
                    // have a beaker freeze itself dry, which is further
                    // from the truth than saying nothing. The consequence,
                    // written up in PLAN's known gaps: the thermometer
                    // after this operator is not a claim.
                    //
                    // Other volatile liquids would co-evaporate by relative
                    // volatility — that is L3's job; say so.
                    let other_liquids: Vec<&str> = v
                        .contents
                        .iter()
                        .filter(|p| p.phase == Phase::Liquid && p.species != water)
                        .filter_map(|p| species::lookup(&p.species).map(|d| d.name))
                        .collect();
                    if !other_liquids.is_empty() {
                        events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: format!(
                                "co-evaporation of {} needs vapour-liquid equilibrium — that is `distil`'s job; only the water was removed",
                                other_liquids.join(", ")
                            ),
                        });
                    }
                }
            }
            Operator::Distil {
                from,
                to,
                fraction,
                energy,
                stages,
            } => {
                let take = match (fraction, energy) {
                    (Some(f), None) => {
                        if !(0.0..=1.0).contains(f) {
                            return Err(BenchError::BadFraction);
                        }
                        kerotakis_thermo::vle::StillTake::Fraction(*f)
                    }
                    (None, Some(e)) => {
                        if e.0 < 0.0 {
                            return Err(BenchError::BadFraction);
                        }
                        kerotakis_thermo::vle::StillTake::EnergyKj(e.0 / 1000.0)
                    }
                    // Exactly one way of asking; both or neither is a
                    // malformed request, not a chemistry question.
                    _ => return Err(BenchError::BadFraction),
                };
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                self.vessel(*to)?; // the receiver must exist before the boil
                let water = SpeciesId::new("water");
                let ethanol = SpeciesId::new("ethanol");
                let src = self.vessel_mut(*from)?;
                // Ethanol counts in either label: with water present the
                // aqueous pass files it as dissolved (it has no derived
                // role, so it dissolves without speciation), and alone it
                // is a liquid. Both are the same volatile matter.
                let w: f64 = src
                    .contents
                    .iter()
                    .filter(|p| p.species == water && p.phase == Phase::Liquid)
                    .map(|p| p.moles.0)
                    .sum();
                let e: f64 = src
                    .contents
                    .iter()
                    .filter(|p| {
                        p.species == ethanol
                            && (p.phase == Phase::Liquid || p.phase == Phase::Aqueous)
                    })
                    .map(|p| p.moles.0)
                    .sum();
                let volatile = w + e;
                if volatile <= 0.0 {
                    events.push(Event::NotYetModeled {
                        vessel: *from,
                        what: "distillation of a vessel with no liquid water or ethanol — \
                               other volatile liquids need their Antoine constants curated \
                               first"
                            .to_string(),
                    });
                } else {
                    let pressure_kpa = src.pressure.0 / 1000.0;
                    match kerotakis_thermo::vle::ethanol_water_still(
                        w,
                        e,
                        take,
                        *stages,
                        pressure_kpa,
                    ) {
                        None => events.push(Event::NotYetModeled {
                            vessel: *from,
                            what: format!(
                                "a bubble point for this mixture at {pressure_kpa:.1} kPa — \
                                 outside the fitted Antoine ranges"
                            ),
                        }),
                        Some(cut) => {
                            // The Rayleigh cut: vapour composition follows
                            // the pot as it drifts, through `stages` ideal
                            // stages at total reflux — the honest upper
                            // bound a real column cannot beat. The energy
                            // number is the latent heat the burner paid
                            // and the condenser dumped; it never touches
                            // the vessel ledger, and the event says so.
                            let removed_e = src.withdraw(&ethanol, Moles(cut.ethanol_over));
                            let removed_w = src.withdraw(&water, Moles(cut.water_over));
                            let at = Kelvin(cut.t_start_c + 273.15);
                            let ended = Kelvin(cut.t_end_c + 273.15);
                            let energy_kj = cut.energy_kj;
                            let azeotropic = cut.azeotrope_limited;
                            // The condensate carries the source's sensible
                            // enthalpy into the receiver (adiabatic mixing,
                            // the decant rule): the boil's latent and
                            // sensible surplus came from the burner and is
                            // externally powered, so the ledger must not
                            // invent it. `at` reports where it boiled, not
                            // what the receiver's thermometer reads.
                            let t_from = src.temperature;
                            let cp_in: f64 = [(&water, removed_w), (&ethanol, removed_e)]
                                .iter()
                                .filter_map(|(s, n)| {
                                    species::lookup(s).map(|d| n.0 * d.heat_capacity)
                                })
                                .sum();
                            let dst = self.vessel_mut(*to)?;
                            if matches!(dst.thermal_mode, ThermalMode::Adiabatic) {
                                let t_new = adiabatic_mix_temperature(
                                    dst.temperature,
                                    dst.heat_capacity(),
                                    t_from,
                                    cp_in,
                                );
                                if (t_new.0 - dst.temperature.0).abs() > 1e-9 {
                                    events.push(Event::TemperatureChanged {
                                        vessel: *to,
                                        from: dst.temperature,
                                        to: t_new,
                                    });
                                }
                                dst.temperature = t_new;
                            }
                            if removed_w.0 > 0.0 {
                                dst.deposit(water.clone(), removed_w, Phase::Liquid);
                            }
                            if removed_e.0 > 0.0 {
                                dst.deposit(ethanol.clone(), removed_e, Phase::Liquid);
                            }
                            // Like `evaporate`, the still is externally
                            // powered: the latent heat is billed on the
                            // event, not the ledger, and the thermometer
                            // after this operator is not a claim (PLAN,
                            // known gaps).
                            events.push(Event::Distilled {
                                from: *from,
                                to: *to,
                                water: removed_w,
                                ethanol: removed_e,
                                at,
                                ended,
                                stages: *stages,
                                energy_kj,
                                azeotropic,
                            });
                        }
                    }
                }
            }
            Operator::Drain { from, to } => {
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                self.vessel(*to)?;
                let src = self.vessel_mut(*from)?;
                let Some((_upper, lower)) = crate::solve::layered_pair(src) else {
                    events.push(Event::NotYetModeled {
                        vessel: *from,
                        what: "draining a single-phase liquid — the funnel only separates \
                               what the thermodynamics has already split into layers"
                            .to_string(),
                    });
                    return Ok(events);
                };
                let lower_id = SpeciesId::new(lower);
                let upper_id = SpeciesId::new(_upper);
                // The lower layer takes its solvent and everything dissolved
                // in it — except that a neutral solute with a curated UNIFAC
                // decomposition obeys its computed partition coefficient and
                // leaves some of itself dissolved in the upper layer. Solids
                // stay behind: a stopcock passes liquid, and a solid sitting
                // in the funnel is a filtration question, not a separation
                // one.
                let lower_solvent_moles: f64 = src
                    .contents
                    .iter()
                    .filter(|p| p.species == lower_id && p.phase == Phase::Liquid)
                    .map(|p| p.moles.0)
                    .sum();
                let upper_solvent_moles: f64 = src
                    .contents
                    .iter()
                    .filter(|p| p.species == upper_id && p.phase == Phase::Liquid)
                    .map(|p| p.moles.0)
                    .sum();
                let t_k = src.temperature.0;
                let mut partitioned: Vec<(SpeciesId, f64)> = Vec::new();
                let mut moved: Vec<(SpeciesId, Moles, Phase)> = Vec::new();
                for p in src.contents.iter() {
                    let is_lower_solvent = p.species == lower_id && p.phase == Phase::Liquid;
                    let dissolved = p.phase == Phase::Aqueous
                        || (p.phase == Phase::Liquid
                            && p.species != lower_id
                            && p.species != upper_id);
                    if is_lower_solvent {
                        moved.push((p.species.clone(), p.moles, p.phase));
                    } else if dissolved {
                        match partition_groups(&p.species) {
                            Some(solute) => {
                                let f = kerotakis_thermo::lle::partition_fraction_lower(
                                    &solute,
                                    &water_groups(),
                                    &hexane_groups(),
                                    lower_solvent_moles,
                                    upper_solvent_moles,
                                    t_k,
                                );
                                moved.push((p.species.clone(), Moles(p.moles.0 * f), p.phase));
                                partitioned.push((p.species.clone(), f));
                            }
                            None => moved.push((p.species.clone(), p.moles, p.phase)),
                        }
                    }
                }
                let solvent_moles = moved
                    .iter()
                    .filter(|(s, ..)| *s == lower_id)
                    .map(|(_, m, _)| m.0)
                    .sum::<f64>();
                for (spec, m, _) in &moved {
                    src.withdraw(spec, *m);
                }
                let t_from = src.temperature;
                let cp_in: f64 = moved
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
                    if !moved.is_empty() && (t_new.0 - dst.temperature.0).abs() > 1e-9 {
                        events.push(Event::TemperatureChanged {
                            vessel: *to,
                            from: dst.temperature,
                            to: t_new,
                        });
                    }
                    dst.temperature = t_new;
                }
                for (spec, m, phase) in moved {
                    dst.deposit(spec, m, phase);
                }
                for (species, f) in partitioned {
                    events.push(Event::Partitioned {
                        vessel: *from,
                        species,
                        fraction_lower: f,
                    });
                }
                events.push(Event::Drained {
                    from: *from,
                    to: *to,
                    solvent: lower_id,
                    moles: Moles(solvent_moles),
                });
            }
            Operator::Wait { seconds } => {
                // Kinetics runs here, before the solver stack: rates change
                // the composition, and the fast equilibria — speciation,
                // acid-base — then re-settle around whatever is left. That
                // ordering is operator splitting, and it is the right way
                // round because equilibrium is the faster process.
                let seconds = seconds.max(0.0);
                for vessel in self.vessels.iter_mut() {
                    vessel.elapsed_seconds += seconds;
                    // EXP-49: decay is the slowest clock on the bench;
                    // it runs beside kinetics on the same shared time.
                    for step in crate::nuclide::advance(&mut vessel.nuclides, seconds) {
                        events.push(Event::Decayed {
                            vessel: vessel.id,
                            parent: step.parent.to_string(),
                            daughter: step.daughter.to_string(),
                            mode: format!("{:?}", step.mode),
                            moles: Moles(step.moles),
                            half_life_s: step.half_life_s,
                            equation: step.equation,
                        });
                    }
                    for (reaction, moles) in crate::kinetics::advance(vessel, seconds)? {
                        if moles.0 < crate::OBSERVABLE_MOLES {
                            continue;
                        }
                        let (ea, catalyst) = reaction.effective_activation_energy(vessel);
                        events.push(Event::Reacted {
                            vessel: vessel.id,
                            reaction: reaction.id.to_string(),
                            equation: reaction.equation.to_string(),
                            moles,
                            seconds,
                            catalyst: catalyst.map(|c| {
                                species::lookup_key(c.species)
                                    .map(|d| d.name.to_string())
                                    .unwrap_or_else(|| c.species.to_string())
                            }),
                            activation_energy: ea,
                        });
                    }
                }
            }
            Operator::Measure { vessel, instrument } => {
                let v = self.vessel(*vessel)?;
                match instrument {
                    Instrument::Thermometer => events.push(Event::Measured {
                        vessel: *vessel,
                        instrument: *instrument,
                        value: v.temperature.to_celsius(),
                        unit: "°C".to_string(),
                    }),
                    Instrument::Balance => events.push(Event::Measured {
                        vessel: *vessel,
                        instrument: *instrument,
                        value: v.mass().0,
                        unit: "g".to_string(),
                    }),
                    Instrument::Eyes => events.push(Event::Observed {
                        vessel: *vessel,
                        appearance: crate::appearance::observe(v),
                    }),
                    Instrument::PhMeter => match &v.solution {
                        Some(info) => events.push(Event::Measured {
                            vessel: *vessel,
                            instrument: *instrument,
                            value: info.ph,
                            unit: "pH".to_string(),
                        }),
                        None => events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: "the pH meter reads nothing — no aqueous solution has been characterised in this vessel"
                                .to_string(),
                        }),
                    },
                    Instrument::PressureGauge => events.push(Event::Measured {
                        vessel: *vessel,
                        instrument: *instrument,
                        value: v.pressure.0 / 1000.0,
                        unit: "kPa".to_string(),
                    }),
                    Instrument::VolumeMeter => {
                        let vol_ml = match v.headspace {
                            crate::vessel::Headspace::Sealed { volume } |
                            crate::vessel::Headspace::PressureControlled { volume, .. } => volume.0 * 1000.0,
                            _ => 0.0,
                        };
                        events.push(Event::Measured {
                            vessel: *vessel,
                            instrument: *instrument,
                            value: vol_ml,
                            unit: "mL".to_string(),
                        })
                    }
                    Instrument::ConductivityMeter => match &v.solution {
                        Some(info) => events.push(Event::Measured {
                            vessel: *vessel,
                            instrument: *instrument,
                            value: info.ionic_strength * 100_000.0,
                            unit: "µS/cm".to_string(),
                        }),
                        None => events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: "the conductivity meter reads nothing — no aqueous solution has been characterised".to_string(),
                        }),
                    },
                    Instrument::Spectrophotometer => {
                        let spec = crate::instrument::Spectrophotometer::default();
                        if let Some(reading) = spec.measure(v) {
                            events.push(Event::Measured {
                                vessel: *vessel,
                                instrument: *instrument,
                                value: reading.value,
                                unit: reading.observable,
                            });
                        } else {
                            events.push(Event::NotYetModeled {
                                vessel: *vessel,
                                what: "no aqueous solution for spectrophotometer".to_string(),
                            });
                        }
                    }
                    Instrument::Calorimeter => {
                        let cal = crate::instrument::Calorimeter;
                        if let Some(reading) = cal.measure(v) {
                            events.push(Event::Measured {
                                vessel: *vessel,
                                instrument: *instrument,
                                value: reading.value,
                                unit: reading.unit,
                            });
                        }
                    }
                    Instrument::GeigerCounter => {
                        events.push(Event::Measured {
                            vessel: *vessel,
                            instrument: *instrument,
                            value: crate::nuclide::total_activity_bq(&v.nuclides),
                            unit: "Bq".to_string(),
                        });
                    }
                    Instrument::Chromatograph => {
                        // The mobile phase is water: the sample is whatever
                        // sits dissolved in it. K per solute is the same
                        // γ∞(water)/γ∞(alkane) ratio the separating funnel
                        // partitions on, so column and funnel cannot
                        // disagree about hydrophobicity. Nothing is
                        // consumed: an analytical injection is an aliquot
                        // too small for the ledger to see.
                        let water = SpeciesId::new("water");
                        let mobile_moles: f64 = v
                            .contents
                            .iter()
                            .filter(|p| p.species == water && p.phase == Phase::Liquid)
                            .map(|p| p.moles.0)
                            .sum();
                        if mobile_moles <= 0.0 {
                            events.push(Event::NotYetModeled {
                                vessel: *vessel,
                                what: "chromatography needs an aqueous sample — \
                                       the column's mobile phase is water"
                                    .to_string(),
                            });
                        } else {
                            let column =
                                crate::instrument::ChromatographyColumn::school();
                            let t_k = v.temperature.0;
                            let mut injectable: std::collections::BTreeMap<
                                SpeciesId,
                                f64,
                            > = std::collections::BTreeMap::new();
                            let mut outside: std::collections::BTreeSet<SpeciesId> =
                                std::collections::BTreeSet::new();
                            for p in v.contents.iter() {
                                let dissolved = p.phase == Phase::Aqueous
                                    || (p.phase == Phase::Liquid && p.species != water);
                                if !dissolved || p.moles.0 <= 0.0 {
                                    continue;
                                }
                                if partition_groups(&p.species).is_some() {
                                    *injectable.entry(p.species.clone()).or_insert(0.0) +=
                                        p.moles.0;
                                } else {
                                    outside.insert(p.species.clone());
                                }
                            }
                            if injectable.is_empty() {
                                events.push(Event::NotYetModeled {
                                    vessel: *vessel,
                                    what: "nothing dissolved here has a curated UNIFAC \
                                           decomposition, so the column's method is \
                                           silent — ions want ion exchange, which is \
                                           not modeled"
                                        .to_string(),
                                });
                            } else {
                                let mut peaks: Vec<ElutedPeak> = injectable
                                    .into_iter()
                                    .map(|(species, moles)| {
                                        let solute = partition_groups(&species)
                                            .expect("filtered on Some above");
                                        let k =
                                            kerotakis_thermo::lle::infinite_dilution_gamma(
                                                &solute,
                                                &water_groups(),
                                                t_k,
                                            ) / kerotakis_thermo::lle::infinite_dilution_gamma(
                                                &solute,
                                                &hexane_groups(),
                                                t_k,
                                            );
                                        let tr = column.retention_time(k);
                                        ElutedPeak {
                                            species,
                                            retention_time_s: tr,
                                            width_s: column.peak_width(tr),
                                            relative_area: moles,
                                            partition_k: k,
                                        }
                                    })
                                    .collect();
                                peaks.sort_by(|a, b| {
                                    a.retention_time_s.total_cmp(&b.retention_time_s)
                                });
                                let largest = peaks
                                    .iter()
                                    .map(|p| p.relative_area)
                                    .fold(0.0_f64, f64::max);
                                for p in &mut peaks {
                                    p.relative_area /= largest;
                                }
                                events.push(Event::Chromatographed {
                                    vessel: *vessel,
                                    plates: column.plates,
                                    void_time_s: column.void_time_s,
                                    peaks,
                                    outside_method: outside.into_iter().collect(),
                                });
                            }
                        }
                    }
                }
            }
            Operator::Electrolyse {
                vessel,
                amps,
                seconds,
            } => {
                if *amps <= 0.0 || *seconds <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let v = self.vessel(*vessel)?;
                match crate::displacement::electrolyse(v, *amps, *seconds) {
                    Some(run) => {
                        if run.moles > crate::OBSERVABLE_MOLES {
                            let (species, moles) = (run.species.clone(), Moles(run.moles));
                            let taken = (run.ion.clone(), Moles(run.moles * run.ion_per_metal));
                            let v = self.vessel_mut(*vessel)?;
                            v.deposit(species.clone(), moles, Phase::Solid);
                            v.withdraw(&taken.0, taken.1);
                            events.push(Event::Electrolysed {
                                vessel: *vessel,
                                species,
                                coulombs: run.coulombs,
                                electrons: Moles(run.electrons),
                                moles,
                                grams: run.grams,
                                per_ion: run.per_ion,
                            });
                            // The other electrode has to be somewhere.
                            //
                            // Taking copper out of solution leaves its
                            // charge behind, and the solve balances that
                            // with acid: the beaker goes from pH 4.27 to
                            // 1.84 on 0.01 mol of electrons. That is the
                            // right chemistry for an *inert* anode —
                            // 2 H₂O → O₂ + 4 H⁺ + 4 e⁻, carbon rods, the
                            // school cell — but the acid was appearing
                            // without the oxygen that pays for it.
                            //
                            // Booked here so the ledger closes. A copper
                            // anode instead dissolves to replace what
                            // plates out, holds Cu²⁺ constant and makes no
                            // acid at all; that is electrorefining and it
                            // is a different cell, which the register says.
                            let oxygen = Moles(run.electrons / 4.0);
                            if oxygen.0 > crate::OBSERVABLE_MOLES {
                                let v = self.vessel_mut(*vessel)?;
                                v.withdraw(&SpeciesId::new("water"), Moles(oxygen.0 * 2.0));
                                let oxygen_id = SpeciesId::new("O2");
                                if v.retain_gas(oxygen_id.clone(), oxygen) {
                                    events.push(Event::GasContained {
                                        vessel: *vessel,
                                        species: oxygen_id,
                                        moles: oxygen,
                                    });
                                } else {
                                    events.push(Event::GasEvolved {
                                        vessel: *vessel,
                                        species: oxygen_id,
                                        moles: oxygen,
                                    });
                                }
                            }
                        }
                        // The charge asked for more than the beaker had.
                        // A real cell answers that by electrolysing the
                        // water instead; this one says it cannot.
                        if run.demanded > run.moles * (1.0 + 1e-9) + crate::OBSERVABLE_MOLES {
                            events.push(Event::NotYetModeled {
                                vessel: *vessel,
                                what: format!(
                                    "the charge would deposit {:.4} mol and the solution \
                                     holds only {:.4} mol. Past that a real cell starts \
                                     electrolysing the water itself, which this bench does \
                                     not model, so the rest of the charge went nowhere",
                                    run.demanded, run.moles
                                ),
                            });
                        }
                    }
                    None => {
                        let why = crate::displacement::why_no_electrode(self.vessel(*vessel)?);
                        events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: format!("nothing here can be electrolysed: {why}"),
                        });
                    }
                }
            }
            Operator::Cell { a, b } => {
                if a == b {
                    return Err(BenchError::SelfTransfer);
                }
                let (va, vb) = (self.vessel(*a)?, self.vessel(*b)?);
                match crate::displacement::cell(va, vb) {
                    Ok(cell) => {
                        let (anode, cathode) = if cell.anode_is_first {
                            (*a, *b)
                        } else {
                            (*b, *a)
                        };
                        events.push(Event::CellVoltage {
                            anode,
                            cathode,
                            volts: cell.volts,
                            standard_volts: cell.standard_volts,
                            notation: cell.notation(),
                            equation: cell.equation(),
                        });
                    }
                    Err(why) => events.push(Event::NoCell { a: *a, b: *b, why }),
                }
            }
            Operator::Grind {
                vessel,
                species,
                diameter_um,
            } => {
                let _v = self.vessel(*vessel)?;
                events.push(Event::NotYetModeled {
                    vessel: *vessel,
                    what: format!(
                        "particle size set to {diameter_um} µm for {} — heterogeneous rate \
                         scaling requires surface-area model integration",
                        species.0
                    ),
                });
            }
            Operator::Irradiate {
                vessel,
                wavelength_nm,
                irradiance_w_m2,
            } => {
                let _v = self.vessel(*vessel)?;
                events.push(Event::NotYetModeled {
                    vessel: *vessel,
                    what: format!(
                        "UV source at {wavelength_nm} nm, {irradiance_w_m2} W/m² — \
                         photolysis rate integration requires coupled kinetics",
                    ),
                });
            }
            Operator::SpikeNuclide {
                vessel,
                nuclide,
                moles,
            } => {
                if moles.0 <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let Some(data) = crate::nuclide::lookup_notation(nuclide) else {
                    let known: Vec<&str> = crate::nuclide::TEACHING_NUCLIDES
                        .iter()
                        .map(|n| n.nuclide)
                        .collect();
                    events.push(Event::NotYetModeled {
                        vessel: *vessel,
                        what: format!(
                            "no curated nuclide '{nuclide}' — the teaching set: {}",
                            known.join(", ")
                        ),
                    });
                    return Ok(events);
                };
                let v = self.vessel_mut(*vessel)?;
                let parsed = crate::nuclide::Nuclide::parse(nuclide)
                    .expect("lookup_notation vetted the notation");
                v.nuclides.deposit(parsed, moles.0);
                let activity = data
                    .decay
                    .as_ref()
                    .map(|d| {
                        let lambda = (2.0_f64).ln() / d.half_life_s;
                        moles.0 * 6.022e23 * lambda
                    })
                    .unwrap_or(0.0);
                events.push(Event::HazardWarning {
                    severity: crate::solve::Severity::Caution,
                    hazard: "radioactive source: ionising radiation".to_string(),
                    real_world: "on a real bench this needs shielding, \
                                 dosimetry and a licence; safe only because \
                                 this lab is virtual"
                        .to_string(),
                });
                events.push(Event::NuclideSpiked {
                    vessel: *vessel,
                    nuclide: nuclide.clone(),
                    moles: *moles,
                    activity_bq: activity,
                });
            }
            Operator::React { vessel, reaction } => {
                match crate::curated::ORG_REACTIONS
                    .iter()
                    .find(|r| r.name == reaction)
                {
                    None => {
                        // The parser vets names, but an operator can arrive
                        // by JSON; refuse out loud rather than panic.
                        events.push(Event::NotYetModeled {
                            vessel: *vessel,
                            what: format!(
                                "no curated reaction named '{reaction}' — curated: {}",
                                crate::curated::ORG_REACTIONS
                                    .iter()
                                    .map(|r| r.name)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        });
                    }
                    Some(r) => {
                        let v = self.vessel_mut(*vessel)?;
                        let extent = r
                            .reactants
                            .iter()
                            .map(|(key, coeff)| v.moles_of(&SpeciesId::new(key)).0 / coeff)
                            .fold(f64::INFINITY, f64::min);
                        if !(extent.is_finite() && extent > 1e-12) {
                            let needs: Vec<&str> = r.reactants.iter().map(|(k, _)| *k).collect();
                            events.push(Event::NotYetModeled {
                                vessel: *vessel,
                                what: format!(
                                    "nothing for {} to work on — it needs {} together                                      in the vessel",
                                    r.name,
                                    needs.join(" and ")
                                ),
                            });
                        } else {
                            for (key, coeff) in r.reactants {
                                v.withdraw(&SpeciesId::new(key), Moles(extent * coeff));
                            }
                            for (key, coeff, phase) in r.products {
                                v.deposit(SpeciesId::new(key), Moles(extent * coeff), *phase);
                            }
                            events.push(Event::OrgReacted {
                                vessel: *vessel,
                                name: r.name.to_string(),
                                equation: r.equation.to_string(),
                                extent: Moles(extent),
                                boundary: r.boundary.to_string(),
                            });
                        }
                    }
                }
            }
            Operator::Dilute { vessel, volume } => {
                let water = SpeciesId::new("water");
                let data = species::lookup(&water)
                    .ok_or_else(|| BenchError::UnknownSpecies(water.clone()))?;
                let moles = data.moles_from_liters(*volume);
                if moles.0 <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let v = self.vessel_mut(*vessel)?;
                if matches!(v.thermal_mode, ThermalMode::Adiabatic) {
                    let t_new = adiabatic_mix_temperature(
                        v.temperature,
                        v.heat_capacity(),
                        Kelvin::STANDARD,
                        moles.0 * data.heat_capacity,
                    );
                    if (t_new.0 - v.temperature.0).abs() > 1e-9 {
                        events.push(Event::TemperatureChanged {
                            vessel: *vessel,
                            from: v.temperature,
                            to: t_new,
                        });
                    }
                    v.temperature = t_new;
                }
                v.deposit(water, moles, data.standard_phase);
                events.push(Event::Diluted {
                    vessel: *vessel,
                    volume: *volume,
                    moles,
                });
            }
            Operator::Transport {
                chain,
                inlet,
                receiver,
                steps,
                courant,
            } => {
                if chain.is_empty() {
                    return Err(BenchError::NonPositiveAmount);
                }
                if *steps == 0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                for cid in chain.iter() {
                    self.vessel(*cid)?;
                }
                self.vessel(*inlet)?;
                self.vessel(*receiver)?;
                let inlet_vessel = self.vessel(*inlet)?.clone();
                let chain_vessels: Vec<crate::vessel::Vessel> = chain
                    .iter()
                    .map(|id| self.vessel(*id).cloned())
                    .collect::<Result<_, _>>()?;
                let mut cell_chain = crate::transport::CellChain::new(chain_vessels)?;

                let mut total_effluent: Vec<(SpeciesId, Moles)> = Vec::new();
                for _ in 0..*steps {
                    let step = cell_chain.advance(&inlet_vessel, *courant)?;
                    for portion in &step.effluent.contents {
                        if let Some(entry) = total_effluent
                            .iter_mut()
                            .find(|(s, _)| *s == portion.species)
                        {
                            entry.1 = Moles(entry.1 .0 + portion.moles.0);
                        } else {
                            total_effluent.push((portion.species.clone(), portion.moles));
                        }
                    }
                }

                for (i, cid) in chain.iter().enumerate() {
                    let updated = &cell_chain.cells()[i];
                    let v = self.vessel_mut(*cid)?;
                    v.contents = updated.contents.clone();
                    v.temperature = updated.temperature;
                    v.solute_charge = updated.solute_charge;
                    v.solution = None;
                }

                let t_eff = inlet_vessel.temperature;
                let cp_eff: f64 = total_effluent
                    .iter()
                    .filter_map(|(s, n)| species::lookup(s).map(|d| n.0 * d.heat_capacity))
                    .sum();
                let dst = self.vessel_mut(*receiver)?;
                if matches!(dst.thermal_mode, ThermalMode::Adiabatic) && cp_eff > 0.0 {
                    let t_new = adiabatic_mix_temperature(
                        dst.temperature,
                        dst.heat_capacity(),
                        t_eff,
                        cp_eff,
                    );
                    if (t_new.0 - dst.temperature.0).abs() > 1e-9 {
                        events.push(Event::TemperatureChanged {
                            vessel: *receiver,
                            from: dst.temperature,
                            to: t_new,
                        });
                    }
                    dst.temperature = t_new;
                }
                for (spec, moles) in &total_effluent {
                    let phase = species::lookup(spec)
                        .map(|d| d.standard_phase)
                        .unwrap_or(Phase::Aqueous);
                    dst.deposit(spec.clone(), *moles, phase);
                }

                events.push(Event::Transported {
                    chain: chain.clone(),
                    receiver: *receiver,
                    steps: *steps,
                    courant: *courant,
                    effluent_moles: total_effluent,
                });
            }
            Operator::Titrate { .. } => {
                unreachable!("titrate is handled by titrate_loop in step_with")
            }
        }
        Ok(events)
    }

    fn titrate_loop(
        &mut self,
        op: Operator,
        solver: &mut dyn Equilibrator,
        _screen: &dyn SafetyScreen,
    ) -> Result<Vec<Event>, BenchError> {
        let (vessel, titrant, concentration, step, target_ph, max_steps) = match &op {
            Operator::Titrate {
                vessel,
                titrant,
                concentration,
                step,
                target_ph,
                max_steps,
            } => (
                *vessel,
                titrant.clone(),
                *concentration,
                *step,
                *target_ph,
                *max_steps,
            ),
            _ => unreachable!(),
        };

        let data =
            species::lookup(&titrant).ok_or_else(|| BenchError::UnknownSpecies(titrant.clone()))?;
        // The burette holds a standard solution: each step delivers
        // concentration × volume moles of titrant, carried by the water
        // of the step volume. (Delivering the *pure* substance by volume
        // — the previous reading — doses ~50× per mL for NaOH and leaps
        // the whole curve in one step; no practical is run that way.)
        let moles_per_step = Moles(concentration * step.0);
        let water = SpeciesId::new("water");
        let water_data =
            species::lookup(&water).ok_or_else(|| BenchError::UnknownSpecies(water.clone()))?;
        let water_per_step = water_data.moles_from_liters(step);
        if moles_per_step.0 <= 0.0 {
            return Err(BenchError::NonPositiveAmount);
        }

        let mut events = Vec::new();
        let mut curve: Vec<(f64, f64)> = Vec::new();

        // Read initial pH if available.
        {
            let v = self.vessel(vessel)?;
            if let Some(info) = &v.solution {
                curve.push((0.0, info.ph));
            }
        }

        let mut total_volume = Liters(0.0);
        let mut reached = false;

        for _ in 0..max_steps {
            // Sub-step: add one increment of titrant at standard temperature.
            let v = self.vessel_mut(vessel)?;
            if matches!(v.thermal_mode, ThermalMode::Adiabatic) {
                let t_new = adiabatic_mix_temperature(
                    v.temperature,
                    v.heat_capacity(),
                    Kelvin::STANDARD,
                    moles_per_step.0 * data.heat_capacity
                        + water_per_step.0 * water_data.heat_capacity,
                );
                v.temperature = t_new;
            }
            v.deposit(titrant.clone(), moles_per_step, data.standard_phase);
            v.deposit(water.clone(), water_per_step, Phase::Liquid);
            total_volume = Liters(total_volume.0 + step.0);

            // Re-equilibrate so the solver computes the new pH.
            let v = self.vessel_mut(vessel)?;
            v.solution = None;
            if solver.applies(v) {
                match solver.equilibrate(v) {
                    Ok(mut more) => events.append(&mut more),
                    Err(e) => events.push(Event::SolverFailed {
                        vessel,
                        solver: solver.name().to_string(),
                        detail: e.to_string(),
                    }),
                }
            }
            self.vessel_mut(vessel)?.refresh_pressure();

            // Read pH after this step.
            let v = self.vessel(vessel)?;
            match &v.solution {
                Some(info) => {
                    let ml = total_volume.0 * 1000.0;
                    let ph = info.ph;
                    let prev_ph = curve.last().map(|&(_, p)| p);
                    curve.push((ml, ph));
                    if let Some(prev) = prev_ph {
                        let crossed = (prev <= target_ph && ph >= target_ph)
                            || (prev >= target_ph && ph <= target_ph);
                        if crossed {
                            reached = true;
                            break;
                        }
                    }
                }
                None => {
                    events.push(Event::NotYetModeled {
                        vessel,
                        what: "titration needs an aqueous solver to compute pH \
                               after each addition — none is wired"
                            .to_string(),
                    });
                    break;
                }
            }
        }

        let final_ph = curve.last().map(|&(_, p)| p).unwrap_or(f64::NAN);
        let step_count = if curve.is_empty() {
            0
        } else {
            (curve.len() as u32).saturating_sub(1)
        };

        if step_count > 0 || reached {
            events.push(Event::Titrated {
                vessel,
                titrant,
                concentration,
                steps: step_count,
                total_volume,
                final_ph,
                curve,
            });
        }

        self.log.push(LogEntry {
            step: self.log.len(),
            operator: op,
            events: events.clone(),
        });
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
        Operator::NewVessel { .. } => vec![],
        Operator::Add { vessel, .. }
        | Operator::Heat { vessel, .. }
        | Operator::Cool { vessel, .. }
        | Operator::Stir { vessel }
        | Operator::Seal { vessel, .. }
        | Operator::Regulate { vessel, .. }
        | Operator::Sweep { vessel, .. }
        | Operator::Open { vessel } => vec![*vessel],
        Operator::Evaporate { vessel, .. } | Operator::Ignite { vessel } => vec![*vessel],
        // Electrolysis moves matter, so the vessel is re-settled after it.
        Operator::Electrolyse { vessel, .. } => vec![*vessel],
        Operator::Decant { from, to, .. }
        | Operator::Filter { from, to }
        | Operator::Distil { from, to, .. }
        | Operator::Drain { from, to } => vec![*from, *to],
        Operator::Mix { a, b, into, .. } => vec![*a, *b, *into],
        Operator::Grind { vessel, .. }
        | Operator::Irradiate { vessel, .. }
        | Operator::Dilute { vessel, .. }
        | Operator::React { vessel, .. }
        | Operator::SpikeNuclide { vessel, .. }
        | Operator::Titrate { vessel, .. } => vec![*vessel],
        Operator::Transport {
            chain, receiver, ..
        } => {
            let mut touched = chain.clone();
            touched.push(*receiver);
            touched
        }
        Operator::Measure { .. } | Operator::Cell { .. } => vec![],
        Operator::Wait { .. } => vec![],
    }
}

fn vent_headspace(vessel: &mut Vessel) -> Vec<(SpeciesId, Moles)> {
    let gases = vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Gas)
        .map(|portion| (portion.species.clone(), portion.moles))
        .collect();
    vessel
        .contents
        .retain(|portion| portion.phase != Phase::Gas);
    gases
}

/// Fill a newly material-closed boundary from the environment it replaced.
/// An open room contributes dry air; an inert sweep contributes nitrogen.
/// Reconfiguring an already closed boundary preserves its existing inventory.
fn trap_boundary_gas(
    vessel: &mut Vessel,
    previous: Headspace,
    volume: Liters,
    pressure: Pascal,
) -> Moles {
    if matches!(
        previous,
        Headspace::Sealed { .. } | Headspace::PressureControlled { .. }
    ) {
        return Moles(0.0);
    }

    const R_LITRE_PASCAL: f64 = 8_314.462_618;
    const AIR_N2: f64 = 0.7901;
    const AIR_O2: f64 = 0.2095;
    const AIR_CO2: f64 = 0.0004;
    let moles = pressure.0 * volume.0 / (R_LITRE_PASCAL * vessel.temperature.0);
    if matches!(previous, Headspace::Swept { .. }) {
        vessel.deposit(SpeciesId::new("N2"), Moles(moles), Phase::Gas);
    } else {
        vessel.deposit(SpeciesId::new("N2"), Moles(moles * AIR_N2), Phase::Gas);
        vessel.deposit(SpeciesId::new("O2"), Moles(moles * AIR_O2), Phase::Gas);
        vessel.deposit(SpeciesId::new("CO2"), Moles(moles * AIR_CO2), Phase::Gas);
    }
    Moles(moles)
}

/// UNIFAC group decompositions for the partitioning solutes and the two
/// curated layer solvents. A solute earns partitioning by entering this
/// table; everything else travels entirely with the water it is
/// dissolved in, which is exactly right for ions.
fn partition_groups(species: &SpeciesId) -> Option<kerotakis_thermo::unifac::GroupDecomposition> {
    let mut g = kerotakis_thermo::unifac::GroupDecomposition::new();
    match species.0.as_str() {
        "ethanol" => {
            g.insert(1, 1); // CH3
            g.insert(2, 1); // CH2
            g.insert(14, 1); // OH
        }
        "methanol" => {
            g.insert(1, 1); // CH3
            g.insert(14, 1); // OH
        }
        "propanone" => {
            g.insert(1, 1); // CH3
            g.insert(18, 1); // CH3CO — the ketone carries its own methyl
        }
        _ => return None,
    }
    Some(g)
}

fn water_groups() -> kerotakis_thermo::unifac::GroupDecomposition {
    let mut g = kerotakis_thermo::unifac::GroupDecomposition::new();
    g.insert(16, 1);
    g
}

fn hexane_groups() -> kerotakis_thermo::unifac::GroupDecomposition {
    let mut g = kerotakis_thermo::unifac::GroupDecomposition::new();
    g.insert(1, 2);
    g.insert(2, 4);
    g
}

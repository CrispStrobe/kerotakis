//! The bench: a set of vessels, the operator log, and the step loop
//! (operator → L0 → apply → re-equilibrate → events).

use serde::{Deserialize, Serialize};

use crate::ops::{Event, Instrument, LogEntry, Operator};
use crate::solve::{
    adiabatic_mix_temperature, Equilibrator, HonestyEquilibrator, MixingEquilibrator,
    PermissiveScreen, SafetyScreen, SafetyVerdict, SolverStack,
};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Joules, Kelvin, Moles};
use crate::vessel::{ThermalMode, Vessel, VesselId};

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
        let temperature_before = match &op {
            Operator::Ignite { vessel } => self.vessel(*vessel)?.temperature,
            _ => Kelvin::STANDARD,
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
            if !solver.applies(vessel) {
                continue;
            }
            match solver.equilibrate(vessel) {
                Ok(mut more) => events.append(&mut more),
                Err(e) => events.push(Event::SolverFailed {
                    vessel: id,
                    solver: solver.name().to_string(),
                    detail: e.to_string(),
                }),
            }
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
                                "co-evaporation of {} needs vapour-liquid equilibrium (L3, not wired yet) — only the water was removed",
                                other_liquids.join(", ")
                            ),
                        });
                    }
                }
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
                    for (reaction, moles) in crate::kinetics::advance(vessel, seconds) {
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
                                events.push(Event::GasEvolved {
                                    vessel: *vessel,
                                    species: SpeciesId::new("O2"),
                                    moles: oxygen,
                                });
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
        Operator::Evaporate { vessel, .. } | Operator::Ignite { vessel } => vec![*vessel],
        // Electrolysis moves matter, so the vessel is re-settled after it.
        Operator::Electrolyse { vessel, .. } => vec![*vessel],
        Operator::Decant { from, to, .. } | Operator::Filter { from, to } => vec![*from, *to],
        Operator::Measure { .. } | Operator::Cell { .. } => vec![],
        // Handled by the caller, which has the vessel list: waiting touches
        // every vessel on the bench, because the clock is shared.
        Operator::Wait { .. } => vec![],
    }
}

//! The bench: a set of vessels, the operator log, and the step loop
//! (operator → L0 → apply → re-equilibrate → events).

use serde::{Deserialize, Serialize};

use crate::authority::SpillDestination;
use crate::instrument::InstrumentContract;
use crate::material::{self, MaterialBasis, MaterialRecipe, MaterialRole};
use crate::ops::{
    CentrifugeSeparation, ElutedPeak, Endpoint, Event, Instrument, LogEntry,
    MaterialComponentAdded, Operator,
};
use crate::solve::{
    adiabatic_mix_temperature, Equilibrator, HonestyEquilibrator, MixingEquilibrator,
    PermissiveScreen, SafetyScreen, SafetyVerdict, SolverStack,
};
use crate::species::{self, Phase, SpeciesId};
use crate::spill::SpillCompartment;
use crate::units::{Grams, Joules, Kelvin, Liters, Moles, Pascal};
use crate::vessel::{
    Headspace, MaterialLot, MaterialObject, MaterialObjectState, ObjectComponent, ThermalMode,
    UnresolvedMaterialPortion, Vessel, VesselId,
};

/// The temperature a match or spark brings its immediate surroundings to.
pub const IGNITION_K: f64 = 1200.0;

/// The one word a person would use for the liquid in this vessel.
///
/// EXP-39's self-indicating endpoint reads this and nothing else. There is
/// no visibility constant anywhere in the path: the word comes out of the
/// registry's ε(λ) for whatever is dissolved, Beer–Lambert over the
/// vessel's own path length, and the CIE observer — the same pipeline that
/// paints the bench. Permanganate is its own indicator here for the reason
/// it is one in a flask, that its molar absorptivity is enormous, and a
/// species without a curated spectrum simply cannot end a titration this
/// way, which is the honest answer rather than a silent one.
fn liquid_colour_word_of(vessel: &Vessel) -> &'static str {
    let seen = crate::appearance::observe(vessel);
    seen.liquid
        .as_ref()
        .map(|colour| crate::appearance::liquid_colour_word(colour, seen.cloudiness))
        .unwrap_or("colourless")
}

/// KID-21: a refusal that has an obvious remedy states it.
///
/// Every message below was already correct and none of them said what to do
/// instead, which is a different kind of unhelpful from being wrong. The
/// children's corpus lost an experiment to each: `filter v1 v2` refused
/// because `v2` did not exist yet and never mentioned `new`, and a learner
/// who has just watched chalk dissolve is told there is no solid to grind
/// without being told that is *because* it dissolved.
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error("no vessel {0} — make it first with `new`, which creates the next free vessel")]
    NoSuchVessel(VesselId),
    #[error("unknown species '{0}' — not in the registry")]
    UnknownSpecies(SpeciesId),
    #[error("unknown material '{0}' — not in the recipe registry")]
    UnknownMaterial(String),
    #[error("material recipe identity does not match the pinned operator")]
    MaterialRecipeMismatch,
    #[error("amount must be positive")]
    NonPositiveAmount,
    #[error("nothing on the shelf is called '{0}' — it is neither a species nor a material")]
    UnstockableKey(String),
    #[error("the '{key}' bottle holds {remaining} {unit}, and {requested} {unit} was asked for")]
    StockExhausted {
        key: String,
        requested: f64,
        remaining: f64,
        unit: crate::stock::StockUnit,
    },
    #[error("fraction must be within 0..=1")]
    BadFraction,
    #[error("source and target vessel are the same")]
    SelfTransfer,
    #[error("vessel {0} is not empty — transfer or dispose of its contents first")]
    VesselNotEmpty(VesselId),
    #[error("the last vessel must stay on the bench")]
    LastVessel,
    #[error("vessel {0} is broken and cannot be used")]
    BrokenVessel(VesselId),
    #[error("no spill exists at the requested destination")]
    NoSuchSpill,
    #[error(
        "vessel {vessel} contains no solid {species} to grind — grinding changes a \
         solid's particle size, so it has to happen before the solid dissolves, not after"
    )]
    SolidNotPresent {
        vessel: VesselId,
        species: SpeciesId,
    },
    #[error("centrifuge cannot run this vessel: {0}")]
    CentrifugeUnavailable(String),
    #[error(
        "centrifuge rotor is {imbalance_g:.2} g out of balance (sample {sample_g:.2} g, counterbalance {counterbalance_g:.2} g); match within 0.10 g"
    )]
    CentrifugeImbalance {
        sample_g: f64,
        counterbalance_g: f64,
        imbalance_g: f64,
    },
    #[error(transparent)]
    Kinetics(#[from] crate::kinetics::IntegrationError),
    #[error(transparent)]
    Transport(#[from] crate::transport::TransportError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bench {
    pub vessels: Vec<Vessel>,
    pub log: Vec<LogEntry>,
    #[serde(default)]
    pub spills: Vec<SpillCompartment>,
    #[serde(default)]
    pub broken_vessels: Vec<VesselId>,
    /// BRD-002: how much of each shelf entry is left. Empty means every
    /// bottle is bottomless, which is what a sandbox wants and what every
    /// snapshot written before this field carried — hence `default`, so an
    /// older token still restores.
    #[serde(default, skip_serializing_if = "crate::stock::StockLedger::is_empty")]
    pub stock: crate::stock::StockLedger,
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
            spills: Vec::new(),
            broken_vessels: Vec::new(),
            stock: crate::stock::StockLedger::default(),
        }
    }

    pub fn vessel(&self, id: VesselId) -> Result<&Vessel, BenchError> {
        self.vessels
            .iter()
            .find(|v| v.id == id)
            .ok_or(BenchError::NoSuchVessel(id))
    }

    pub fn spill(&self, destination: &SpillDestination) -> Option<&SpillCompartment> {
        self.spills
            .iter()
            .find(|spill| &spill.destination == destination)
    }

    pub fn is_broken(&self, vessel: VesselId) -> bool {
        self.broken_vessels.contains(&vessel)
    }

    fn vessel_mut(&mut self, id: VesselId) -> Result<&mut Vessel, BenchError> {
        self.vessels
            .iter_mut()
            .find(|v| v.id == id)
            .ok_or(BenchError::NoSuchVessel(id))
    }

    /// KID-15: a receiver a pour is aimed at, created if it is not there.
    ///
    /// `filter v1 v2` refused with "no vessel v2" until a learner worked out
    /// that `new` comes first. On a real bench you do not declare the jar
    /// before you pour into it — you put one under the funnel, which is what
    /// pouring *means*. So the three transfer verbs now bring their receiver
    /// into being at the moment they need it, and say so with the same
    /// `VesselCreated` event `new` emits, so nothing downstream can tell the
    /// difference between a jar the learner asked for and one the pour did.
    ///
    /// Only the *destination* is created. A pour out of a vessel that does
    /// not exist is still a refusal, because there is nothing to pour.
    fn ensure_destination(&mut self, id: VesselId, events: &mut Vec<Event>) {
        if self.vessels.iter().any(|v| v.id == id) {
            return;
        }
        if self.broken_vessels.contains(&id) {
            return;
        }
        self.vessels.push(Vessel::new(id, "beaker"));
        events.push(Event::VesselCreated { vessel: id });
    }

    fn move_to_spill(
        &mut self,
        from: VesselId,
        destination: &SpillDestination,
        fraction: f64,
        all_phases: bool,
        screen: &dyn SafetyScreen,
        events: &mut Vec<Event>,
    ) -> Result<(), BenchError> {
        let source = self.vessel(from)?.clone();
        if !source.material_objects.is_empty() {
            events.push(Event::ObjectSpillBoundary {
                vessel: from,
                object_count: source.material_objects.len(),
            });
        }
        let eligible = |phase: Phase| all_phases || matches!(phase, Phase::Liquid | Phase::Aqueous);
        let moved = source
            .contents
            .iter()
            .filter(|portion| eligible(portion.phase))
            .map(|portion| {
                let mut moved = portion.clone();
                moved.moles.0 *= fraction;
                moved
            })
            .filter(|portion| portion.moles.0 > 1e-15)
            .collect::<Vec<_>>();
        let unresolved = source
            .unresolved_materials
            .iter()
            .filter(|portion| all_phases || material::unresolved_portion_is_liquid(portion))
            .map(|portion| {
                let mut moved = portion.clone();
                moved.amount *= fraction;
                moved
            })
            .filter(|portion| portion.amount > 1e-15)
            .collect::<Vec<_>>();

        let mut spill = self
            .spill(destination)
            .cloned()
            .unwrap_or_else(|| SpillCompartment::new(destination.clone(), source.temperature));
        let old_cp: f64 = spill
            .contents
            .iter()
            .filter_map(|portion| {
                species::lookup(&portion.species).map(|data| data.heat_capacity * portion.moles.0)
            })
            .sum();
        let incoming_cp: f64 = moved
            .iter()
            .filter_map(|portion| {
                species::lookup(&portion.species).map(|data| data.heat_capacity * portion.moles.0)
            })
            .sum();
        if old_cp + incoming_cp > 0.0 {
            spill.temperature = Kelvin(
                (old_cp * spill.temperature.0 + incoming_cp * source.temperature.0)
                    / (old_cp + incoming_cp),
            );
        }
        for portion in &moved {
            if let Some(existing) = spill.contents.iter_mut().find(|candidate| {
                candidate.species == portion.species && candidate.phase == portion.phase
            }) {
                existing.moles.0 += portion.moles.0;
            } else {
                spill.contents.push(portion.clone());
            }
        }
        spill.unresolved_materials.extend(unresolved.clone());
        if !spill.sources.contains(&from) {
            spill.sources.push(from);
        }
        let mut contributors = spill
            .contents
            .iter()
            .map(|portion| portion.species.clone())
            .collect::<Vec<_>>();
        contributors.sort_by(|a, b| a.0.cmp(&b.0));
        contributors.dedup();
        match screen.assess(&spill.as_vessel_probe()) {
            SafetyVerdict::Allow => {}
            SafetyVerdict::Warn {
                severity,
                rule,
                hazard,
                real_world,
            } => events.push(Event::SpillHazard {
                destination: destination.clone(),
                severity,
                rule,
                hazard,
                real_world,
                contributors: contributors.clone(),
            }),
            SafetyVerdict::Veto { reason } => events.push(Event::SpillHazard {
                destination: destination.clone(),
                severity: crate::solve::Severity::Danger,
                rule: String::new(),
                hazard: reason,
                real_world: "Do not touch the spill; follow the declared cleanup procedure.".into(),
                contributors,
            }),
        }

        let source = self.vessel_mut(from)?;
        for portion in &mut source.contents {
            if eligible(portion.phase) {
                portion.moles.0 *= 1.0 - fraction;
            }
        }
        source.contents.retain(|portion| portion.moles.0 > 1e-15);
        for portion in &mut source.unresolved_materials {
            if all_phases || material::unresolved_portion_is_liquid(portion) {
                portion.amount *= 1.0 - fraction;
            }
        }
        source
            .unresolved_materials
            .retain(|portion| portion.amount > 1e-15);
        if let Some(existing) = self
            .spills
            .iter_mut()
            .find(|item| item.destination == *destination)
        {
            *existing = spill;
        } else {
            self.spills.push(spill);
        }
        Ok(())
    }

    fn recover_spill(
        &mut self,
        destination: &SpillDestination,
        to: VesselId,
        fraction: f64,
        screen: &dyn SafetyScreen,
        events: &mut Vec<Event>,
    ) -> Result<bool, BenchError> {
        let index = self
            .spills
            .iter()
            .position(|spill| spill.destination == *destination)
            .ok_or(BenchError::NoSuchSpill)?;
        let spill = self.spills[index].clone();
        let mut probe = self.vessel(to)?.clone();
        for portion in &spill.contents {
            probe.deposit(
                portion.species.clone(),
                Moles(portion.moles.0 * fraction),
                portion.phase,
            );
        }
        match screen.assess(&probe) {
            SafetyVerdict::Allow => {}
            SafetyVerdict::Warn {
                severity,
                rule,
                hazard,
                real_world,
            } => events.push(Event::HazardWarning {
                severity,
                rule,
                hazard,
                real_world,
            }),
            SafetyVerdict::Veto { reason } => {
                events.push(Event::SafetyVeto { reason });
                return Ok(false);
            }
        }
        let incoming_cp: f64 = spill
            .contents
            .iter()
            .filter_map(|portion| {
                species::lookup(&portion.species)
                    .map(|data| data.heat_capacity * portion.moles.0 * fraction)
            })
            .sum();
        let receiver = self.vessel_mut(to)?;
        if matches!(receiver.thermal_mode, ThermalMode::Adiabatic) {
            receiver.temperature = adiabatic_mix_temperature(
                receiver.temperature,
                receiver.heat_capacity(),
                spill.temperature,
                incoming_cp,
            );
        }
        for portion in &spill.contents {
            receiver.deposit(
                portion.species.clone(),
                Moles(portion.moles.0 * fraction),
                portion.phase,
            );
        }
        receiver
            .unresolved_materials
            .extend(spill.unresolved_materials.iter().map(|portion| {
                let mut moved = portion.clone();
                moved.amount *= fraction;
                moved
            }));
        for portion in &mut self.spills[index].contents {
            portion.moles.0 *= 1.0 - fraction;
        }
        self.spills[index]
            .contents
            .retain(|portion| portion.moles.0 > 1e-15);
        for portion in &mut self.spills[index].unresolved_materials {
            portion.amount *= 1.0 - fraction;
        }
        self.spills[index]
            .unresolved_materials
            .retain(|portion| portion.amount > 1e-15);
        if self.spills[index].contents.is_empty()
            && self.spills[index].unresolved_materials.is_empty()
        {
            self.spills.remove(index);
        }
        Ok(true)
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
        let curdling_before = op_touches(&op)
            .into_iter()
            .filter_map(|id| {
                self.vessel(id).ok().map(|vessel| {
                    (
                        id,
                        crate::curdling::observe(vessel)
                            .map(|curds| curds.formed_fraction)
                            .unwrap_or(0.0),
                    )
                })
            })
            .collect::<Vec<_>>();
        let gel_before = op_touches(&op)
            .into_iter()
            .filter_map(|id| {
                self.vessel(id).ok().map(|vessel| {
                    (
                        id,
                        crate::gel::observe(vessel)
                            .map(|gel| gel.gelled_fraction)
                            .unwrap_or(0.0),
                    )
                })
            })
            .collect::<Vec<_>>();
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
            vessel.mark_liquid_contact();
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
            // CAP-25: sealed glass has a limit, and exceeding it is an
            // event, not a scripted animation. The seal fails, the
            // gases vent, and the ledger stays exact through the bang.
            if vessel.is_sealed() && vessel.pressure.0 > crate::senses::GLASS_BURST_PA {
                let at = vessel.pressure.0;
                let gases = vent_headspace(vessel);
                vessel.headspace = Headspace::Open;
                vessel.refresh_pressure();
                events.push(Event::Burst {
                    vessel: id,
                    at_pa: at,
                    rating_pa: crate::senses::GLASS_BURST_PA,
                });
                events.push(Event::HazardWarning {
                    severity: crate::solve::Severity::Danger,
                    rule: "sealed-vessel-burst".to_string(),
                    hazard: "sealed vessel over-pressurised and burst".to_string(),
                    real_world: "flying glass and a pressure wave — sealed \
                                 systems on a heat source are how real labs \
                                 get hurt; safe only because this lab is \
                                 virtual"
                        .to_string(),
                });
                for (species, moles) in gases {
                    events.push(Event::GasEvolved {
                        vessel: id,
                        species,
                        moles,
                    });
                }
            }
        }

        for id in touched.iter().copied() {
            let before = curdling_before
                .iter()
                .find(|(candidate, _)| *candidate == id)
                .map(|(_, fraction)| *fraction)
                .unwrap_or(0.0);
            let Some(after) = self.vessel(id).ok().and_then(crate::curdling::observe) else {
                continue;
            };
            if after.formed_fraction > before + 1e-9 {
                events.push(Event::CurdlingChanged {
                    vessel: id,
                    material: after.material,
                    from_formed_fraction: before,
                    to_formed_fraction: after.formed_fraction,
                    separation_progress: after.separation_progress,
                    curd_solids_mass_g: after.curd_solids_mass_g,
                    acid_species: SpeciesId::new(&after.acid_species),
                    acid_moles: Moles(after.acid_moles),
                });
            }
        }

        // KID-11: foam is a property of gas meeting a surfactant, and it
        // used to be a property of one reaction id. The trap lives here
        // rather than in `advance_vessel_time` because the gas that makes a
        // volcano erupt leaves during the solver pass of the ADD step —
        // there is no `wait` in a volcano, and by the next one the carbon
        // dioxide is long gone from the ledger.
        let foam_seconds = match &op {
            Operator::Wait { seconds } => seconds.max(0.0),
            Operator::Stir { seconds, .. } => seconds.max(0.0),
            _ => 0.0,
        };
        let mut foam_events: Vec<(usize, Event)> = Vec::new();
        for id in touched.iter().copied() {
            let (gas, after) = gas_made_this_step(&events, id);
            // Nothing new to trap and no time to drain is not a foam
            // event. Without this, a vessel that had once foamed would
            // report its unchanged foam again on every later `look`,
            // which is noise in the log and churn in the goldens — the
            // old call site got this for free by running only on `stir`
            // and `wait`.
            if gas <= 0.0 && foam_seconds <= 0.0 {
                continue;
            }
            let Ok(vessel) = self.vessel_mut(id) else {
                continue;
            };
            let Some(foam) = crate::foam::advance(vessel, foam_seconds, gas) else {
                continue;
            };
            if foam.volume_liters >= 1e-6 || vessel.foam.peak_volume_liters > 0.0 {
                let at = after.map(|index| index + 1).unwrap_or(events.len());
                foam_events.push((
                    at,
                    Event::FoamChanged {
                        vessel: id,
                        trapped_gas_liters: foam.trapped_gas_liters,
                        volume_liters: foam.volume_liters,
                        height_cm: foam.height_cm,
                        overflow_liters: foam.overflow_liters,
                        half_life_seconds: foam.half_life_seconds,
                    },
                ));
            }
        }
        // Insert from the back so earlier positions stay valid.
        foam_events.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
        for (at, event) in foam_events {
            events.insert(at.min(events.len()), event);
        }

        // KID-13: gas leaving the liquid is what the dancing raisin rides.
        // The trigger is the gas, not the clock — this bench degasses a
        // glass in one step, so the moment to say it is the moment the
        // bubbles appear, which is when the fizzy water meets the raisin.
        for id in touched.iter().copied() {
            let evolving = events.iter().any(|event| {
                matches!(event, Event::GasEvolved { vessel, moles, .. }
                    if *vessel == id && moles.0 >= crate::OBSERVABLE_MOLES)
            });
            if !evolving {
                continue;
            }
            let Some(ride) = self.vessel(id).ok().and_then(crate::buoyancy::observe) else {
                continue;
            };
            events.push(Event::BubbleRide {
                vessel: id,
                object: ride.material,
                object_density_g_per_ml: ride.object_density_g_per_ml,
                liquid_density_g_per_ml: ride.liquid_density_g_per_ml,
                lift_gas_fraction: ride.lift_gas_fraction,
            });
        }

        for id in touched.iter().copied() {
            let before = gel_before
                .iter()
                .find(|(candidate, _)| *candidate == id)
                .map(|(_, fraction)| *fraction)
                .unwrap_or(0.0);
            let Some(after) = self.vessel(id).ok().and_then(crate::gel::observe) else {
                continue;
            };
            if after.gelled_fraction > before + 1e-9 {
                events.push(Event::GelFormed {
                    vessel: id,
                    polymer: SpeciesId::new(after.polymer),
                    crosslinker: SpeciesId::new(after.crosslinker),
                    from_gelled_fraction: before,
                    to_gelled_fraction: after.gelled_fraction,
                    polymer_grams: after.polymer_grams,
                    crosslinker_moles: Moles(after.crosslinker_moles),
                });
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
            // A coupled solver can leave a trailing provisional event whose
            // `from` already equals the settled temperature.  Correcting
            // that event turns it into a no-op; after removing it, the
            // preceding temperature event becomes the final announcement
            // and must be reconciled too.  Continue until the last surviving
            // announcement describes a real move to the exposed state.
            while let Some(i) = events.iter().rposition(
                |e| matches!(e, Event::TemperatureChanged { vessel, .. } if *vessel == id),
            ) {
                if let Event::TemperatureChanged { from, to, .. } = &mut events[i] {
                    *to = actual;
                    let stale = (from.0 - to.0).abs() < 0.01;
                    if stale {
                        events.remove(i);
                        continue;
                    }
                }
                break;
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
            // A chemistry solver may quantify the heat released using its
            // own thermodynamic model. Carry that number on the ignition
            // event itself so every host can scale the flame without
            // reverse-engineering temperature or composition changes.
            let reaction_energy_j = events.iter().find_map(|event| match event {
                Event::ThermalEquilibrium {
                    reaction_energy_j, ..
                } => *reaction_energy_j,
                _ => None,
            });
            if caught {
                if let Some(Event::Ignited { energy_j, .. }) = events
                    .iter_mut()
                    .find(|event| matches!(event, Event::Ignited { .. }))
                {
                    *energy_j = reaction_energy_j;
                }
            }
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
                    None if !examined => events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoSolver,
                        vessel: *vessel,
                        what: "whether these contents burn: no wired solver models combustion for them, so the lab cannot say either way".to_string(),
                    }),
                    // KID-12: "nothing happens, not everything burns" is
                    // the wrong sentence when the bench has just said
                    // WHY nothing happened. A smothered candle is not an
                    // unburnable one, and telling a learner otherwise
                    // teaches the opposite of the demonstration.
                    None if events
                        .iter()
                        .any(|event| matches!(event, Event::FlameStarved { .. })) => {}
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
        if !matches!(op, Operator::Impact { .. } | Operator::RemoveVessel { .. }) {
            if let Some(vessel) = op_touches(op).into_iter().find(|id| self.is_broken(*id)) {
                return Err(BenchError::BrokenVessel(vessel));
            }
        }
        match op {
            Operator::NewVessel { kind } => {
                let label = kind.as_deref().unwrap_or("beaker");
                let id = VesselId(
                    self.vessels
                        .iter()
                        .map(|v| v.id.0)
                        .chain(self.broken_vessels.iter().map(|id| id.0))
                        .max()
                        .map_or(0, |id| id + 1),
                );
                self.vessels.push(Vessel::new(id, label));
                events.push(Event::VesselCreated { vessel: id });
            }
            Operator::RemoveVessel { vessel } => {
                if self.vessels.len() <= 1 {
                    return Err(BenchError::LastVessel);
                }
                if !self.vessel(*vessel)?.is_empty() {
                    return Err(BenchError::VesselNotEmpty(*vessel));
                }
                self.vessels.retain(|candidate| candidate.id != *vessel);
                events.push(Event::VesselRemoved { vessel: *vessel });
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
                        rule,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        rule,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // BRD-002: the bottle is drawn only once the safety screen
                // has let the operation through — a vetoed dispense must
                // not cost the shelf anything, because it never happened.
                if let Err(refusal) = self.stock.draw(&sid.0, moles.0) {
                    events.push(stock_refusal_event(&sid.0, refusal));
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
                v.deposit_lot(
                    sid.clone(),
                    *moles,
                    data.standard_phase,
                    Some("reagent bottle".to_string()),
                    None,
                );
                let total_after = v.moles_of(sid);
                events.push(Event::Added {
                    vessel: *vessel,
                    species: sid.clone(),
                    moles: *moles,
                    total_after: Some(total_after),
                });
            }
            Operator::StockShelf { key, amount } => {
                if !amount.is_finite() || *amount < 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let unit = crate::stock::stock_unit(key)
                    .ok_or_else(|| BenchError::UnstockableKey(key.clone()))?;
                self.stock.stock(key, *amount, unit);
                events.push(Event::ShelfStocked {
                    key: key.clone(),
                    amount: *amount,
                    unit,
                });
            }
            Operator::AddMaterial {
                vessel,
                material: material_name,
                recipe_id,
                recipe_version,
                total_amount,
                basis,
                sample_seed,
                at,
            } => {
                if !total_amount.is_finite() || *total_amount <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let recipe = material::lookup(material_name, None)
                    .ok_or_else(|| BenchError::UnknownMaterial(material_name.clone()))?;
                if recipe.id != *recipe_id
                    || recipe.version != *recipe_version
                    || recipe.basis != *basis
                {
                    return Err(BenchError::MaterialRecipeMismatch);
                }
                let expansion = recipe
                    .expand(*total_amount, *sample_seed)
                    .ok_or(BenchError::NonPositiveAmount)?;
                let components = expansion
                    .components
                    .iter()
                    .map(|component| {
                        let sid = SpeciesId::new(&component.species_id);
                        let data = species::lookup(&sid)
                            .ok_or_else(|| BenchError::UnknownSpecies(sid.clone()))?;
                        let moles = material_amount_to_moles(&recipe, component.amount, data);
                        Ok((sid, data.standard_phase, component.amount, moles))
                    })
                    .collect::<Result<Vec<_>, BenchError>>()?;

                // Assess the fully expanded prospective mixture once. This
                // avoids allowing a hazardous combination merely because its
                // ingredients happened to be deposited one at a time.
                //
                // KID-3: the screen is told what the vessel held *before*
                // the pour, so it can tell a mixture the learner made from
                // the contents of one reviewed bottle. Lugol's iodine ships
                // iodine and iodide together; screening them against each
                // other raised a "can detonate" banner on a starch test.
                let before = self.vessel(*vessel)?.clone();
                let mut probe = before.clone();
                for (sid, phase, _, moles) in &components {
                    probe.deposit(sid.clone(), *moles, *phase);
                }
                match screen.assess_pour(&before, &probe) {
                    SafetyVerdict::Allow => {}
                    SafetyVerdict::Warn {
                        severity,
                        rule,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        rule,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // The recipe's own basis amount is what the bottle is
                // counted in, so a 5% vinegar bottle empties in millilitres
                // poured — not in moles of acetic acid, which is a number
                // nobody reads off a label.
                if let Err(refusal) = self.stock.draw(&recipe.canonical_key, *total_amount) {
                    events.push(stock_refusal_event(&recipe.canonical_key, refusal));
                    return Ok(events);
                }

                let t_in = at.unwrap_or(Kelvin::STANDARD);
                let cp_in = components
                    .iter()
                    .filter_map(|(sid, _, _, moles)| {
                        species::lookup(sid).map(|data| moles.0 * data.heat_capacity)
                    })
                    .sum();
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
                let coherent = recipe
                    .roles
                    .iter()
                    .any(|role| matches!(role, MaterialRole::CoherentObject));
                if coherent {
                    v.material_objects.push(MaterialObject {
                        material: recipe.canonical_key.clone(),
                        recipe_id: recipe_id.clone(),
                        recipe_version: *recipe_version,
                        mass_g: *total_amount,
                        components: components
                            .iter()
                            .map(|(species, _, _, moles)| ObjectComponent {
                                species: species.clone(),
                                moles: *moles,
                            })
                            .collect(),
                        state: MaterialObjectState::default(),
                    });
                } else {
                    for (sid, phase, _, moles) in &components {
                        v.deposit_lot(
                            sid.clone(),
                            *moles,
                            *phase,
                            Some(format!("material recipe {recipe_id}")),
                            None,
                        );
                    }
                }
                if !coherent && expansion.unresolved_amount > 0.0 {
                    v.unresolved_materials.push(UnresolvedMaterialPortion {
                        material: material_name.clone(),
                        recipe_id: recipe_id.clone(),
                        recipe_version: *recipe_version,
                        basis: *basis,
                        amount: expansion.unresolved_amount,
                        enzyme_hydrolysis: None,
                    });
                }
                events.push(Event::MaterialAdded {
                    vessel: *vessel,
                    material: material_name.clone(),
                    recipe_id: recipe_id.clone(),
                    recipe_version: *recipe_version,
                    total_amount: *total_amount,
                    basis: *basis,
                    sample_seed: *sample_seed,
                    components: components
                        .iter()
                        .map(|(species, _, basis_amount, moles)| MaterialComponentAdded {
                            species: species.clone(),
                            basis_amount: *basis_amount,
                            moles: *moles,
                        })
                        .collect(),
                    unresolved_amount: expansion.unresolved_amount,
                });
                if let Some(spread) = crate::surface_spread::after_material_added(v, &recipe) {
                    let material = v
                        .surface_particles
                        .as_ref()
                        .map(|particles| particles.material.clone())
                        .unwrap_or_else(|| "floating particles".to_string());
                    events.push(Event::SurfaceSpread {
                        vessel: *vessel,
                        material,
                        from_cleared_fraction: spread.from_cleared_fraction,
                        to_cleared_fraction: spread.to_cleared_fraction,
                        coverage_fraction: spread.coverage_fraction,
                    });
                }
                let colour_components = components
                    .iter()
                    .map(|(species, _, _, moles)| (species.clone(), *moles))
                    .collect::<Vec<_>>();
                if let Some(spread) =
                    crate::surface_colour::after_material_added(v, &recipe, &colour_components)
                {
                    events.push(Event::SurfaceColourSpread {
                        vessel: *vessel,
                        from_spread_fraction: spread.from_spread_fraction,
                        to_spread_fraction: spread.to_spread_fraction,
                        spot_count: spread.spot_count,
                    });
                }
                precipitate_declared_soap(v, &recipe, &mut events);
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
                    events.push(Event::EnergyTransferred {
                        vessel: *vessel,
                        heating: signed >= 0.0,
                        requested_j: energy.0,
                        delivered_j: (to.0 - from.0).abs() * cp,
                        time_coupled: false,
                    });
                    if wanted < 0.0 {
                        let could_pay = cp * from.0 / 1000.0;
                        events.push(Event::NotYetModeled {
                            cause: crate::ops::NotModelledCause::ModelBoundary,
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
                        cause: crate::ops::NotModelledCause::NoSolver,
                        vessel: *vessel,
                        what: "heating an empty vessel (container heat capacity not modelled)"
                            .to_string(),
                    });
                }
            }
            Operator::Stir {
                vessel,
                rpm,
                seconds,
            } => {
                if !rpm.is_finite() || !seconds.is_finite() || *rpm <= 0.0 || *seconds <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let v = self.vessel(*vessel)?;
                // A 25 mm bar is the default bench-scale stir bar. The
                // delivered linear speed is physical state, not an animation
                // preset: clients and future transport models consume it.
                let bar_length_m = 0.025;
                let tip_speed_m_s = std::f64::consts::PI * bar_length_m * rpm / 60.0;
                let resuspended_fraction =
                    (1.0 - (-tip_speed_m_s * seconds / 0.3).exp()).clamp(0.0, 1.0);
                let solid_portions = v
                    .contents
                    .iter()
                    .filter(|portion| {
                        portion.phase == Phase::Solid
                            && !crate::displacement::is_elemental_metal(&portion.species.0)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let rate_coupled = solid_portions
                    .iter()
                    .any(|portion| crate::kinetics::is_surface_catalyst(&portion.species));
                let elapsed_seconds = v.elapsed_seconds;
                let has_liquid = v.liquid_volume().0 > 0.0;
                let vessel_id = v.id;
                let v = self.vessel_mut(*vessel)?;
                let mixed_surface_colours = crate::surface_colour::homogenize(v);
                if has_liquid {
                    for portion in solid_portions {
                        let mut found = false;
                        for lot in &mut v.lots {
                            if lot.species == portion.species && lot.phase == Phase::Solid {
                                lot.suspended_fraction = Some(
                                    lot.suspended_fraction
                                        .unwrap_or(0.0)
                                        .max(resuspended_fraction),
                                );
                                found = true;
                            }
                        }
                        if !found {
                            v.lots.push(MaterialLot {
                                species: portion.species,
                                moles: portion.moles,
                                phase: Phase::Solid,
                                added_at: elapsed_seconds,
                                hydrated_at: None,
                                source: Some("legacy vessel state".to_string()),
                                particle_size_um: None,
                                suspended_fraction: Some(resuspended_fraction),
                            });
                        }
                    }
                    v.resolved.invalidate();
                }
                events.push(Event::Stirred {
                    vessel: vessel_id,
                    rpm: *rpm,
                    seconds: *seconds,
                    bar_length_m,
                    tip_speed_m_s,
                    resuspended_fraction,
                    rate_coupled,
                });
                // KID-13: a suspension dense enough to argue back. Reported
                // here because it is a property of the mixture under *this*
                // stir — the same mixture at rest is a liquid, which is the
                // whole of what oobleck has to teach.
                if let Some(thick) = self
                    .vessel(*vessel)
                    .ok()
                    .and_then(|v| crate::rheology::observe(v, tip_speed_m_s))
                {
                    events.push(Event::Thickened {
                        vessel: vessel_id,
                        solid: SpeciesId::new(thick.solid),
                        strength: thick.strength,
                        solid_mass_fraction: thick.solid_mass_fraction,
                        tip_speed_m_s: thick.tip_speed_m_s,
                        sheared_hard: thick.sheared_hard,
                    });
                }
                if mixed_surface_colours > 0 {
                    events.push(Event::SurfaceColourMixed {
                        vessel: vessel_id,
                        spot_count: mixed_surface_colours,
                    });
                }
                // Stirring is a timed bench operation, not a decorative
                // gesture. Let the selected vessel's slow chemistry run for
                // the delivered duration after the solid has been lifted
                // into suspension. Gravity settling is deliberately disabled
                // while the bar is turning.
                let vessel = self.vessel_mut(*vessel)?;
                advance_vessel_time(
                    vessel,
                    *seconds,
                    false,
                    crate::kinetics::KineticContext {
                        mixing_tip_speed_m_s: tip_speed_m_s,
                    },
                    &mut events,
                )?;
                let before = crate::emulsion::observe(vessel)
                    .map(|observation| observation.dispersed_fraction)
                    .unwrap_or(0.0);
                if let Some(emulsion) = crate::emulsion::after_stir(vessel, resuspended_fraction) {
                    if emulsion.dispersed_fraction > before + 1e-9 {
                        events.push(Event::EmulsionChanged {
                            vessel: vessel.id,
                            material: emulsion.material,
                            from_dispersed_fraction: before,
                            to_dispersed_fraction: emulsion.dispersed_fraction,
                            dispersed_volume_l: emulsion.dispersed_volume_l,
                            half_life_seconds: emulsion.half_life_seconds,
                        });
                    }
                }
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
            Operator::Spill {
                from,
                destination,
                fraction,
                replay_seed,
            } => {
                if !fraction.is_finite() || !(0.0..=1.0).contains(fraction) {
                    return Err(BenchError::BadFraction);
                }
                if *fraction == 0.0 {
                    return Ok(events);
                }
                if self.is_broken(*from) {
                    return Err(BenchError::BrokenVessel(*from));
                }
                self.move_to_spill(*from, destination, *fraction, false, screen, &mut events)?;
                events.push(Event::SpillCreated {
                    destination: destination.clone(),
                    source: *from,
                    fraction: *fraction,
                    replay_seed: *replay_seed,
                });
            }
            Operator::Impact {
                vessel,
                impulse_ns,
                destination_if_broken,
                replay_seed,
            } => {
                if !impulse_ns.is_finite() || *impulse_ns < 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                if self.is_broken(*vessel) {
                    return Err(BenchError::BrokenVessel(*vessel));
                }
                let kind = self.vessel(*vessel)?.label.as_str();
                let threshold = impact_threshold_ns(kind);
                if *impulse_ns < threshold {
                    events.push(Event::CollisionWithstood {
                        vessel: *vessel,
                        impulse_ns: *impulse_ns,
                        replay_seed: *replay_seed,
                    });
                } else {
                    self.move_to_spill(
                        *vessel,
                        destination_if_broken,
                        1.0,
                        true,
                        screen,
                        &mut events,
                    )?;
                    self.broken_vessels.push(*vessel);
                    events.push(Event::ContainerBroken {
                        vessel: *vessel,
                        destination: destination_if_broken.clone(),
                        impulse_ns: *impulse_ns,
                        replay_seed: *replay_seed,
                    });
                    events.push(Event::SpillCreated {
                        destination: destination_if_broken.clone(),
                        source: *vessel,
                        fraction: 1.0,
                        replay_seed: *replay_seed,
                    });
                }
            }
            Operator::RecoverSpill {
                destination,
                to,
                fraction,
            } => {
                if !fraction.is_finite() || !(0.0..=1.0).contains(fraction) {
                    return Err(BenchError::BadFraction);
                }
                if *fraction == 0.0 {
                    return Ok(events);
                }
                if self.is_broken(*to) {
                    return Err(BenchError::BrokenVessel(*to));
                }
                if self.recover_spill(destination, *to, *fraction, screen, &mut events)? {
                    events.push(Event::SpillRecovered {
                        destination: destination.clone(),
                        to: *to,
                        fraction: *fraction,
                    });
                }
            }
            Operator::Decant { from, to, fraction } => {
                self.ensure_destination(*to, &mut events);
                if !(0.0..=1.0).contains(fraction) {
                    return Err(BenchError::BadFraction);
                }
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                // Work out what would move, without mutating yet.
                let (would_move, unresolved_move, t_from) = {
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
                    let unresolved: Vec<_> = src
                        .unresolved_materials
                        .iter()
                        .filter(|portion| material::unresolved_portion_is_liquid(portion))
                        .map(|portion| {
                            let mut moved = portion.clone();
                            moved.amount *= fraction;
                            moved
                        })
                        .filter(|portion| portion.amount > 1e-15)
                        .collect();
                    (moved, unresolved, src.temperature)
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
                        rule,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        rule,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // Pouring disturbs both free surfaces. Release any localized
                // dye into the ordinary conserved liquid inventory before
                // proportional withdrawal; a zero-fraction rehearsal changes
                // no physical state.
                if *fraction > 0.0 {
                    for id in [*from, *to] {
                        let count = crate::surface_colour::homogenize(self.vessel_mut(id)?);
                        if count > 0 {
                            events.push(Event::SurfaceColourMixed {
                                vessel: id,
                                spot_count: count,
                            });
                        }
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
                    for portion in &mut src.unresolved_materials {
                        if material::unresolved_portion_is_liquid(portion) {
                            portion.amount *= 1.0 - fraction;
                        }
                    }
                    src.unresolved_materials
                        .retain(|portion| portion.amount > 1e-15);
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
                dst.unresolved_materials.extend(unresolved_move);
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
                let (move_a, unresolved_a, t_a) = {
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
                    let unresolved: Vec<_> = src
                        .unresolved_materials
                        .iter()
                        .filter(|portion| material::unresolved_portion_is_liquid(portion))
                        .map(|portion| {
                            let mut moved = portion.clone();
                            moved.amount *= fraction_a;
                            moved
                        })
                        .filter(|portion| portion.amount > 1e-15)
                        .collect();
                    (moved, unresolved, src.temperature)
                };
                let (move_b, unresolved_b, t_b) = {
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
                    let unresolved: Vec<_> = src
                        .unresolved_materials
                        .iter()
                        .filter(|portion| material::unresolved_portion_is_liquid(portion))
                        .map(|portion| {
                            let mut moved = portion.clone();
                            moved.amount *= fraction_b;
                            moved
                        })
                        .filter(|portion| portion.amount > 1e-15)
                        .collect();
                    (moved, unresolved, src.temperature)
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
                        rule,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        rule,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // Combining streams is itself mechanical mixing. Only a
                // source that contributes liquid is disturbed; the receiving
                // surface is disturbed when either stream contributes.
                for (id, active) in [
                    (*a, *fraction_a > 0.0),
                    (*b, *fraction_b > 0.0),
                    (*into, *fraction_a > 0.0 || *fraction_b > 0.0),
                ] {
                    if active {
                        let count = crate::surface_colour::homogenize(self.vessel_mut(id)?);
                        if count > 0 {
                            events.push(Event::SurfaceColourMixed {
                                vessel: id,
                                spot_count: count,
                            });
                        }
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
                    for portion in &mut src_a.unresolved_materials {
                        if material::unresolved_portion_is_liquid(portion) {
                            portion.amount *= 1.0 - fraction_a;
                        }
                    }
                    src_a
                        .unresolved_materials
                        .retain(|portion| portion.amount > 1e-15);
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
                    for portion in &mut src_b.unresolved_materials {
                        if material::unresolved_portion_is_liquid(portion) {
                            portion.amount *= 1.0 - fraction_b;
                        }
                    }
                    src_b
                        .unresolved_materials
                        .retain(|portion| portion.amount > 1e-15);
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
                dst.unresolved_materials
                    .extend(unresolved_a.into_iter().chain(unresolved_b));
                events.push(Event::Mixed {
                    a: *a,
                    b: *b,
                    into: *into,
                    fraction_a: *fraction_a,
                    fraction_b: *fraction_b,
                    temperature_a: t_a,
                    temperature_b: t_b,
                    temperature_into: dst.temperature,
                });
            }
            Operator::Filter { from, to } => {
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                self.ensure_destination(*to, &mut events);
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
                        rule,
                        hazard,
                        real_world,
                    } => events.push(Event::HazardWarning {
                        severity,
                        rule,
                        hazard,
                        real_world,
                    }),
                    SafetyVerdict::Veto { reason } => {
                        events.push(Event::SafetyVeto { reason });
                        return Ok(events);
                    }
                }

                // A full filtration pour also destroys localized surface-drop
                // geometry on both sides; dye conservation remains in the
                // resolved portions transferred below.
                for id in [*from, *to] {
                    let count = crate::surface_colour::homogenize(self.vessel_mut(id)?);
                    if count > 0 {
                        events.push(Event::SurfaceColourMixed {
                            vessel: id,
                            spot_count: count,
                        });
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
            Operator::Magnet { from, to } => {
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                let (magnetic_solids, remained_ids) = {
                    let src = self.vessel(*from)?;
                    let mag: Vec<_> = src
                        .contents
                        .iter()
                        .filter(|p| {
                            p.phase == Phase::Solid
                                && species::lookup(&p.species).is_some_and(|d| d.magnetic)
                        })
                        .map(|p| (p.species.clone(), p.moles, p.phase))
                        .collect();
                    let rem: Vec<_> = src
                        .contents
                        .iter()
                        .filter(|p| {
                            p.phase == Phase::Solid
                                && !species::lookup(&p.species).is_some_and(|d| d.magnetic)
                        })
                        .map(|p| p.species.clone())
                        .collect();
                    (mag, rem)
                };
                if magnetic_solids.is_empty() {
                    events.push(Event::MagnetSeparated {
                        from: *from,
                        to: *to,
                        attracted: vec![],
                        remained: remained_ids,
                    });
                } else {
                    let src = self.vessel_mut(*from)?;
                    src.contents.retain(|p| {
                        !(p.phase == Phase::Solid
                            && species::lookup(&p.species).is_some_and(|d| d.magnetic))
                    });
                    let dst = self.vessel_mut(*to)?;
                    let attracted_ids: Vec<_> =
                        magnetic_solids.iter().map(|(s, _, _)| s.clone()).collect();
                    for (s, n, phase) in magnetic_solids {
                        dst.deposit(s, n, phase);
                    }
                    events.push(Event::MagnetSeparated {
                        from: *from,
                        to: *to,
                        attracted: attracted_ids,
                        remained: remained_ids,
                    });
                }
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
                        energy_j: None,
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
                        cause: crate::ops::NotModelledCause::NothingToActOn,
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
                            cause: crate::ops::NotModelledCause::NoSolver,
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
                        events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoSolver,
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
                        cause: crate::ops::NotModelledCause::NotParameterised,
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
                            cause: crate::ops::NotModelledCause::ModelBoundary,
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
                self.ensure_destination(*to, &mut events);
                if from == to {
                    return Err(BenchError::SelfTransfer);
                }
                self.vessel(*to)?;
                let src = self.vessel_mut(*from)?;
                let Some((_upper, lower)) = crate::solve::layered_pair(src) else {
                    events.push(Event::NotYetModeled {
                        cause: crate::ops::NotModelledCause::NothingToActOn,
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
                    advance_vessel_time(
                        vessel,
                        seconds,
                        true,
                        crate::kinetics::KineticContext::default(),
                        &mut events,
                    )?;
                    advance_prepared_objects(vessel, seconds, &mut events);
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
                        None => events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoSolution,
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
                    // This meter reads the SEALED HEADSPACE, not the liquid.
                    // On an open vessel it used to report `0.00 mL`, which is
                    // a confident wrong number: a beaker holding 500 mL of
                    // water measured 0.00 mL while `inspect` on the same
                    // vessel printed "500.0 mL liquid". Nothing about that
                    // reads as a gap — it reads as an answer.
                    //
                    // The conductivity meter immediately below is the pattern:
                    // when it has nothing it can report, it says so. An
                    // instrument that cannot make its measurement must refuse,
                    // not return a zero that a learner will read as a volume.
                    Instrument::VolumeMeter => match v.headspace {
                        crate::vessel::Headspace::Sealed { volume }
                        | crate::vessel::Headspace::PressureControlled { volume, .. } => {
                            events.push(Event::Measured {
                                vessel: *vessel,
                                instrument: *instrument,
                                value: volume.0 * 1000.0,
                                unit: "mL".to_string(),
                            })
                        }
                        _ => events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::BoundaryMismatch,
                            vessel: *vessel,
                            what: "the volume meter reads a sealed vessel's headspace, and \
                                   this one is open — seal it to give the gas a measured \
                                   volume. The liquid's own volume is on the vessel line."
                                .to_string(),
                        }),
                    },
                    Instrument::ConductivityMeter => match &v.solution {
                        Some(info) => events.push(Event::Measured {
                            vessel: *vessel,
                            instrument: *instrument,
                            value: crate::conductivity::specific_conductance(info)
                                .microsiemens_per_cm,
                            unit: "µS/cm".to_string(),
                        }),
                        // The dry-solid path: one isolated solid with a
                        // curated resistivity reads as a material property
                        // in S/m (copper wire against iron wire). A dry
                        // solid the registry has no resistivity for is a
                        // missing datum, which is a different refusal from
                        // "no solution here" and is named as one.
                        None => match crate::conductivity::dry_solid_conductance(v) {
                            Some(solid) => events.push(Event::Measured {
                                vessel: *vessel,
                                instrument: *instrument,
                                value: solid.conductivity_s_per_m,
                                unit: "S/m".to_string(),
                            }),
                            None => {
                                let lone_dry_solid = v.liquid_volume().0 <= 0.0
                                    && v.unresolved_materials.is_empty()
                                    && v
                                        .contents
                                        .iter()
                                        .filter(|p| p.phase == Phase::Solid && p.moles.0 > crate::OBSERVABLE_MOLES)
                                        .count()
                                        == 1;
                                if lone_dry_solid {
                                    events.push(Event::NotYetModeled {
                                        cause: crate::ops::NotModelledCause::NoReviewedDatum,
                                        vessel: *vessel,
                                        what: "the conductivity meter is on a dry solid the registry carries no electrical resistivity for — the reading needs a reviewed value, not a guess".to_string(),
                                    });
                                } else {
                                    events.push(Event::NotYetModeled {
                                        cause: crate::ops::NotModelledCause::NoSolution,
                                        vessel: *vessel,
                                        what: "the conductivity meter reads nothing — no aqueous solution has been characterised".to_string(),
                                    });
                                }
                            }
                        },
                    },
                    // KID-19a: density is a measurement, and it is the one
                    // that answers a question a balance cannot. Five grams
                    // of copper, five grams of zinc and five grams of
                    // aluminium all weigh five grams; what tells them apart
                    // is that the aluminium is a much bigger piece.
                    Instrument::Densitometer => {
                        // A hydrometer floats in the liquid, so a liquid
                        // answers even when solids are sitting in it.
                        if let Some(density) = crate::buoyancy::liquid_density_g_per_ml(v) {
                            events.push(Event::Measured {
                                vessel: *vessel,
                                instrument: *instrument,
                                value: density,
                                unit: "g/mL".to_string(),
                            });
                            // KID-19b: and say what the number leaves out.
                            if crate::buoyancy::ionic_volume_unaccounted(v) {
                                events.push(Event::NotYetModeled {
                                    vessel: *vessel,
                                    // The ions have a density field; it is a
                                    // structural default rather than a measured
                                    // value, which is a datum nobody reviewed.
                                    cause: crate::ops::NotModelledCause::NoReviewedDatum,
                                    what: "the dissolved ions' share of this density: every ion in the registry carries water's density as a structural default rather than a measured one, so a salt solution reads as the water it was made from. The solvent's figure is right and the solutes' is missing".to_string(),
                                });
                            }
                        } else {
                            // Dry: then the instrument is a balance and a
                            // measuring cylinder, and the reading is the
                            // substance's own density. That is a property
                            // of ONE substance — a heap of two powders has
                            // a mass and a volume but no density anyone
                            // should be told — so more than one refuses,
                            // and says which ones it found.
                            let solids: Vec<&crate::vessel::Portion> = v
                                .contents
                                .iter()
                                .filter(|portion| {
                                    portion.phase == Phase::Solid
                                        && portion.moles.0 > crate::OBSERVABLE_MOLES
                                })
                                .collect();
                            let objects: Vec<&crate::vessel::UnresolvedMaterialPortion> =
                                v.unresolved_materials.iter().collect();
                            match (solids.as_slice(), objects.as_slice()) {
                                ([portion], []) => {
                                    match species::lookup(&portion.species)
                                        .map(|data| data.density)
                                        .filter(|density| *density > 0.0)
                                    {
                                        Some(density) => events.push(Event::Measured {
                                            vessel: *vessel,
                                            instrument: *instrument,
                                            value: density,
                                            unit: "g/mL".to_string(),
                                        }),
                                        None => events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoReviewedDatum,
                                            vessel: *vessel,
                                            what: format!(
                                                "the density of {} — the registry carries no reviewed density for it, so the piece has a mass here and no size",
                                                portion.species.0
                                            ),
                                        }),
                                    }
                                }
                                ([], [portion]) => {
                                    match crate::material::lookup(&portion.material, None)
                                        .and_then(|recipe| {
                                            recipe.bulk_density.map(|density| density.value)
                                        }) {
                                        Some(density) => events.push(Event::Measured {
                                            vessel: *vessel,
                                            instrument: *instrument,
                                            value: density,
                                            unit: "g/mL".to_string(),
                                        }),
                                        None => events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoReviewedDatum,
                                            vessel: *vessel,
                                            what: format!(
                                                "the density of {} — the recipe carries no reviewed bulk density",
                                                portion.material
                                            ),
                                        }),
                                    }
                                }
                                ([], []) => events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NothingToActOn,
                                    vessel: *vessel,
                                    what: "a density — the vessel is empty, and there is nothing to float the hydrometer in and nothing to weigh".to_string(),
                                }),
                                _ => {
                                    let mut named: Vec<String> = solids
                                        .iter()
                                        .map(|portion| {
                                            species::lookup(&portion.species)
                                                .map(|data| data.name.to_string())
                                                .unwrap_or_else(|| portion.species.0.clone())
                                        })
                                        .collect();
                                    named.extend(
                                        objects.iter().map(|portion| portion.material.clone()),
                                    );
                                    events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoSolver,
                                        vessel: *vessel,
                                        what: format!(
                                            "a single density for this vessel: it holds {}, and a density belongs to one substance rather than to a mixture of them. Weigh them separately",
                                            named.join(" and ")
                                        ),
                                    });
                                }
                            }
                        }
                    }
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
                            events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoSolution,
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
                    // EXP-33: not a scalar `Measured`, because the useful
                    // answer is often a refusal with a reason, and because
                    // a curated constant travels with its citation.
                    Instrument::MeltingPointApparatus => {
                        events.push(Event::TransitionPointRead {
                            vessel: *vessel,
                            reading: crate::instrument::read_transition(
                                v,
                                crate::instrument::TransitionRead::Melting,
                            ),
                        });
                    }
                    Instrument::BoilingPointApparatus => {
                        events.push(Event::TransitionPointRead {
                            vessel: *vessel,
                            reading: crate::instrument::read_transition(
                                v,
                                crate::instrument::TransitionRead::Boiling,
                            ),
                        });
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
                            events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NoSolution,
                                vessel: *vessel,
                                what: "chromatography needs an aqueous sample — \
                                       the column's mobile phase is water"
                                    .to_string(),
                            });
                        } else {
                            let column =
                                crate::instrument::ChromatographyColumn::school();
                            let plate = crate::instrument::PaperPlate::school();
                            let t_k = v.temperature.0;
                            let mut injectable: std::collections::BTreeMap<
                                SpeciesId,
                                f64,
                            > = std::collections::BTreeMap::new();
                            let mut outside: std::collections::BTreeSet<SpeciesId> =
                                std::collections::BTreeSet::new();
                            // Kept apart from `outside`, because "this method
                            // cannot separate ions" and "this model has no groups
                            // for that dye" are different sentences and only one
                            // of them is a result.
                            let mut unparameterised: std::collections::BTreeSet<SpeciesId> =
                                std::collections::BTreeSet::new();
                            for p in v.contents.iter() {
                                let dissolved = p.phase == Phase::Aqueous
                                    || (p.phase == Phase::Liquid && p.species != water);
                                if !dissolved || p.moles.0 <= 0.0 {
                                    continue;
                                }
                                // KID-9: a group decomposition where one is
                                // honest, a reviewed coefficient where it
                                // would not be. A food dye is a large
                                // glycoside; splitting it into UNIFAC groups
                                // would be a fiction dressed as a
                                // calculation, and leaving it out meant the
                                // ink experiment every child does had
                                // nothing to separate.
                                if partition_groups(&p.species).is_some()
                                    || crate::instrument::curated_partition_k(&p.species.0)
                                        .is_some()
                                {
                                    *injectable.entry(p.species.clone()).or_insert(0.0) +=
                                        p.moles.0;
                                } else if is_ionic(&p.species) {
                                    // Genuinely outside the METHOD: a partition
                                    // column separates by how a neutral solute
                                    // divides between two phases, and an ion does
                                    // not do that — it wants ion exchange, which
                                    // this column is not.
                                    outside.insert(p.species.clone());
                                } else {
                                    // Outside this MODEL, not outside the method:
                                    // a neutral solute with no curated UNIFAC
                                    // decomposition is one the column would
                                    // separate on a real bench. Saying "not
                                    // separated" would be a confident negative
                                    // about a gap — the two food dyes in bio-104
                                    // are the case, and paper chromatography
                                    // separating them is the classic demonstration.
                                    unparameterised.insert(p.species.clone());
                                }
                            }
                            if injectable.is_empty() && !unparameterised.is_empty() {
                                // A solute this model cannot decompose is a GAP,
                                // and must stay one. The column would separate it
                                // on a real bench; reporting "not separated" would
                                // dress a missing parameter set as a result.
                                let names: Vec<&str> = unparameterised
                                    .iter()
                                    .map(|s| s.0.as_str())
                                    .collect();
                                events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NotParameterised,
                                    vessel: *vessel,
                                    what: format!(
                                        "the column has no curated group decomposition for \
                                         {} — a real column would separate these, so this \
                                         is a gap in the model rather than a result",
                                        names.join(", ")
                                    ),
                                });
                            } else if injectable.is_empty() && outside.is_empty() {
                                // Nothing dissolved at all: there is no sample,
                                // which is a different thing from a sample the
                                // method cannot see.
                                events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NothingToActOn,
                                    vessel: *vessel,
                                    what: "chromatography needs something dissolved to \
                                           inject — this vessel holds only its mobile \
                                           phase"
                                        .to_string(),
                                });
                            } else if injectable.is_empty() {
                                // A sample the method cannot see is an ANSWER, not a
                                // gap, and the answer was already computed: `outside`
                                // holds exactly the species this column cannot
                                // separate. It used to be discarded and replaced with
                                // "the column's method is silent", which reports the
                                // engine's silence rather than the column's result —
                                // and the question a learner asked ("will dissolved
                                // salt appear in this neutral-solute method?") has a
                                // real answer: no, and here is what passed through
                                // unseparated.
                                //
                                // An empty chromatogram is a chromatogram. The run
                                // happened, the detector saw nothing, and naming what
                                // went past it is the whole of the method's scope.
                                events.push(Event::Chromatographed {
                                    vessel: *vessel,
                                    plates: column.plates,
                                    void_time_s: column.void_time_s,
                                    peaks: Vec::new(),
                                    outside_method: outside.into_iter().collect(),
                                });
                            } else {
                                let mut peaks: Vec<ElutedPeak> = injectable
                                    .into_iter()
                                    .map(|(species, moles)| {
                                        let k = match partition_groups(&species) {
                                            Some(solute) => {
                                                kerotakis_thermo::lle::infinite_dilution_gamma(
                                                    &solute,
                                                    &water_groups(),
                                                    t_k,
                                                ) / kerotakis_thermo::lle::infinite_dilution_gamma(
                                                    &solute,
                                                    &hexane_groups(),
                                                    t_k,
                                                )
                                            }
                                            None => crate::instrument::curated_partition_k(
                                                &species.0,
                                            )
                                            .expect("filtered on Some above")
                                            .0,
                                        };
                                        let tr = column.retention_time(k);
                                        ElutedPeak {
                                            species,
                                            retention_time_s: tr,
                                            width_s: column.peak_width(tr),
                                            relative_area: moles,
                                            partition_k: k,
                                            rf: plate.rf(k),
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
                                amps: *amps,
                                seconds: *seconds,
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
                                cause: crate::ops::NotModelledCause::NothingToActOn,
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
                    // No metal half-cell is not the same as nothing to
                    // electrolyse. Brine needs no metal at all — carbon
                    // rods, hydrogen at one and chlorine at the other — and
                    // the refusal below was accurate about the model and
                    // wrong about the chemistry.
                    None => match crate::displacement::electrolyse_solvent(
                        self.vessel(*vessel)?,
                        *amps,
                        *seconds,
                    ) {
                        Some(run) => {
                            let v = self.vessel_mut(*vessel)?;
                            if run.water_spent > crate::OBSERVABLE_MOLES {
                                v.withdraw(&SpeciesId::new("water"), Moles(run.water_spent));
                            }
                            // Cathode.
                            if run.cathode_moles > crate::OBSERVABLE_MOLES {
                                if run.cathode_plates {
                                    v.deposit(
                                        run.cathode.clone(),
                                        Moles(run.cathode_moles),
                                        Phase::Solid,
                                    );
                                    if let Some((ion, taken)) = &run.cathode_ion {
                                        v.withdraw(ion, Moles(*taken));
                                    }
                                    events.push(Event::Electrolysed {
                                        vessel: *vessel,
                                        species: run.cathode.clone(),
                                        amps: *amps,
                                        seconds: *seconds,
                                        coulombs: run.coulombs,
                                        electrons: Moles(run.electrons),
                                        moles: Moles(run.cathode_moles),
                                        grams: crate::species::lookup(&run.cathode)
                                            .map(|d| run.cathode_moles * d.molar_mass)
                                            .unwrap_or(0.0),
                                        per_ion: run.electrons
                                            / run.cathode_moles.max(f64::MIN_POSITIVE),
                                    });
                                } else {
                                    let m = Moles(run.cathode_moles);
                                    if v.retain_gas(run.cathode.clone(), m) {
                                        events.push(Event::GasContained {
                                            vessel: *vessel,
                                            species: run.cathode.clone(),
                                            moles: m,
                                        });
                                    } else {
                                        events.push(Event::GasEvolved {
                                            vessel: *vessel,
                                            species: run.cathode.clone(),
                                            moles: m,
                                        });
                                    }
                                }
                            }
                            // The caustic soda the chloralkali cell is FOR.
                            if run.hydroxide_made > crate::OBSERVABLE_MOLES {
                                v.deposit(
                                    SpeciesId::new("OH-"),
                                    Moles(run.hydroxide_made),
                                    Phase::Aqueous,
                                );
                            }
                            if run.protons_made > crate::OBSERVABLE_MOLES {
                                v.deposit(
                                    SpeciesId::new("H+"),
                                    Moles(run.protons_made),
                                    Phase::Aqueous,
                                );
                            }
                            // Anode.
                            if run.chloride_spent > crate::OBSERVABLE_MOLES {
                                v.withdraw(&SpeciesId::new("Cl-"), Moles(run.chloride_spent));
                            }
                            if run.anode_moles > crate::OBSERVABLE_MOLES {
                                let m = Moles(run.anode_moles);
                                if v.retain_gas(run.anode.clone(), m) {
                                    events.push(Event::GasContained {
                                        vessel: *vessel,
                                        species: run.anode.clone(),
                                        moles: m,
                                    });
                                } else {
                                    events.push(Event::GasEvolved {
                                        vessel: *vessel,
                                        species: run.anode.clone(),
                                        moles: m,
                                    });
                                }
                            }
                        }
                        None => {
                            let why = crate::displacement::why_no_electrode(self.vessel(*vessel)?);
                            events.push(Event::NotYetModeled {
                                cause: crate::ops::NotModelledCause::NothingToActOn,
                                vessel: *vessel,
                                what: format!("nothing here can be electrolysed: {why}"),
                            });
                        }
                    },
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
                    Err(why) => {
                        if let Some(cell) = crate::displacement::acid_zinc_copper_cell(va, vb) {
                            let (anode, cathode) = if cell.anode_is_first {
                                (*a, *b)
                            } else {
                                (*b, *a)
                            };
                            events.push(Event::AcidMetalCellVoltage {
                                anode,
                                cathode,
                                volts: cell.volts,
                                ph: cell.ph,
                            });
                        } else {
                            events.push(Event::NoCell { a: *a, b: *b, why });
                        }
                    }
                }
            }
            Operator::Grind {
                vessel,
                species,
                diameter_um,
            } => {
                if *diameter_um <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let data = species::lookup(species)
                    .ok_or_else(|| BenchError::UnknownSpecies(species.clone()))?;
                let v = self.vessel_mut(*vessel)?;
                let solid_moles = Moles(
                    v.contents
                        .iter()
                        .filter(|portion| {
                            portion.species == *species && portion.phase == Phase::Solid
                        })
                        .map(|portion| portion.moles.0)
                        .sum(),
                );
                if solid_moles.0 <= 0.0 {
                    return Err(BenchError::SolidNotPresent {
                        vessel: *vessel,
                        species: species.clone(),
                    });
                }

                let mut found_lot = false;
                for lot in &mut v.lots {
                    if lot.species == *species && lot.phase == Phase::Solid {
                        lot.particle_size_um = Some(*diameter_um);
                        found_lot = true;
                    }
                }
                // Saves created before lot tracking still gain real particle state.
                if !found_lot {
                    let has_liquid = v.liquid_volume().0 > 0.0;
                    v.lots.push(MaterialLot {
                        species: species.clone(),
                        moles: solid_moles,
                        phase: Phase::Solid,
                        added_at: v.elapsed_seconds,
                        hydrated_at: None,
                        source: Some("legacy vessel state".to_string()),
                        particle_size_um: Some(*diameter_um),
                        suspended_fraction: Some(if has_liquid { 1.0 } else { 0.0 }),
                    });
                }
                v.resolved.invalidate();

                let volume_m3 = solid_moles.0 * data.molar_mass / data.density * 1e-6;
                let surface_area_m2 = 6.0 * volume_m3 / (*diameter_um * 1e-6);
                events.push(Event::Ground {
                    vessel: *vessel,
                    species: species.clone(),
                    diameter_um: *diameter_um,
                    solid_moles,
                    surface_area_m2,
                    rate_coupled: crate::kinetics::is_surface_catalyst(species),
                });
            }
            Operator::Centrifuge {
                vessel,
                rpm,
                seconds,
                rotor_radius_m,
                counterbalance_g,
            } => {
                let v = self.vessel(*vessel)?;
                if *rpm < 0.0 || *seconds < 0.0 || *rotor_radius_m <= 0.0 {
                    return Err(BenchError::NonPositiveAmount);
                }
                let sample_mass_g = v.mass().0;
                let counterbalance_g = counterbalance_g.unwrap_or(sample_mass_g);
                let imbalance_g = (sample_mass_g - counterbalance_g).abs();
                if imbalance_g > 0.10 {
                    return Err(BenchError::CentrifugeImbalance {
                        sample_g: sample_mass_g,
                        counterbalance_g,
                        imbalance_g,
                    });
                }
                let liquid_volume_l = v.liquid_volume().0;
                if liquid_volume_l <= 0.0 {
                    return Err(BenchError::CentrifugeUnavailable(
                        "a liquid medium is required".to_string(),
                    ));
                }
                let liquid_mass_g: f64 = v
                    .contents
                    .iter()
                    .filter(|portion| portion.phase == Phase::Liquid)
                    .filter_map(|portion| {
                        species::lookup(&portion.species)
                            .map(|data| portion.moles.0 * data.molar_mass)
                    })
                    .sum();
                let fluid_density_kg_m3 = liquid_mass_g / liquid_volume_l;
                let viscosity_cp = crate::properties::water_viscosity_cp(v.temperature.0)
                    .map_err(BenchError::CentrifugeUnavailable)?
                    .value;
                let dynamic_viscosity_pa_s = viscosity_cp / 1000.0;
                let mut separations = Vec::new();
                let mut rcf = 0.0;
                for portion in v
                    .contents
                    .iter()
                    .filter(|portion| portion.phase == Phase::Solid)
                {
                    let data = species::lookup(&portion.species)
                        .ok_or_else(|| BenchError::UnknownSpecies(portion.species.clone()))?;
                    let diameter = v
                        .lots
                        .iter()
                        .rev()
                        .find(|lot| lot.species == portion.species && lot.phase == Phase::Solid)
                        .and_then(|lot| lot.particle_size_um);
                    let particle_size_assumed = diameter.is_none();
                    let particle_diameter_um = diameter.unwrap_or(100.0);
                    let result = crate::centrifuge::run(crate::centrifuge::CentrifugeInput {
                        rpm: *rpm,
                        seconds: *seconds,
                        rotor_radius_m: *rotor_radius_m,
                        tube_path_m: 0.04,
                        particle_diameter_m: particle_diameter_um * 1e-6,
                        particle_density_kg_m3: data.density * 1000.0,
                        fluid_density_kg_m3,
                        dynamic_viscosity_pa_s,
                    })
                    .map_err(|error| BenchError::CentrifugeUnavailable(error.to_string()))?;
                    rcf = result.rcf;
                    separations.push(CentrifugeSeparation {
                        species: portion.species.clone(),
                        particle_diameter_um,
                        particle_size_assumed,
                        particle_density_kg_m3: data.density * 1000.0,
                        terminal_speed_m_s: result.terminal_speed_m_s,
                        distance_m: result.distance_m,
                        separated_fraction: result.separated_fraction,
                        direction: result.direction,
                    });
                }
                if separations.is_empty() {
                    return Err(BenchError::CentrifugeUnavailable(
                        "no solid particles are present".to_string(),
                    ));
                }
                let v = self.vessel_mut(*vessel)?;
                for separation in &separations {
                    let remaining = 1.0 - separation.separated_fraction;
                    let mut found = false;
                    for lot in &mut v.lots {
                        if lot.species == separation.species && lot.phase == Phase::Solid {
                            lot.suspended_fraction =
                                Some(lot.suspended_fraction.unwrap_or(1.0) * remaining);
                            found = true;
                        }
                    }
                    if !found {
                        let moles = v.moles_of(&separation.species);
                        v.lots.push(MaterialLot {
                            species: separation.species.clone(),
                            moles,
                            phase: Phase::Solid,
                            added_at: v.elapsed_seconds,
                            hydrated_at: None,
                            source: Some("solver-created solid".to_string()),
                            particle_size_um: Some(separation.particle_diameter_um),
                            suspended_fraction: Some(remaining),
                        });
                    }
                }
                v.resolved.invalidate();
                events.push(Event::Centrifuged {
                    vessel: *vessel,
                    rpm: *rpm,
                    seconds: *seconds,
                    rotor_radius_m: *rotor_radius_m,
                    rcf,
                    sample_mass_g,
                    counterbalance_g,
                    imbalance_g,
                    fluid_density_kg_m3,
                    dynamic_viscosity_pa_s,
                    separations,
                    state_coupled: true,
                });
            }
            Operator::Irradiate {
                vessel,
                wavelength_nm,
                irradiance_w_m2,
            } => {
                let _v = self.vessel(*vessel)?;
                events.push(Event::Irradiated {
                    vessel: *vessel,
                    wavelength_nm: *wavelength_nm,
                    irradiance_w_m2: *irradiance_w_m2,
                    photolysis_coupled: false,
                });
            }
            Operator::Particles { vessel } => {
                let v = self.vessel(*vessel)?;
                events.push(Event::ParticlesCounted {
                    vessel: *vessel,
                    census: crate::particles::census(v, 30),
                });
            }
            Operator::Smell { vessel } => {
                let v = self.vessel(*vessel)?;
                let noticed = crate::senses::waft(v);
                for o in &noticed {
                    if o.hazardous {
                        events.push(Event::HazardWarning {
                            severity: crate::solve::Severity::Caution,
                            rule: "hazardous-vapour".to_string(),
                            hazard: format!("{} vapour is hazardous to inhale", o.species),
                            real_world: "on a real bench this one is never \
                                         smelled directly — fume hood, waft \
                                         only, and some not even then"
                                .to_string(),
                        });
                    }
                }
                events.push(Event::Smelled {
                    vessel: *vessel,
                    notes: noticed
                        .iter()
                        .map(|o| (SpeciesId::new(o.species), o.description.to_string()))
                        .collect(),
                });
            }
            Operator::TestGas { vessel, test } => {
                let v = self.vessel_mut(*vessel)?;
                events.extend(crate::gas_tests::dispatch(v, *vessel, *test));
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
                        cause: crate::ops::NotModelledCause::NotParameterised,
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
                    rule: "ionising-radiation".to_string(),
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
                if crate::selectivity::is_selectivity_verb(reaction) {
                    let v = self.vessel_mut(*vessel)?;
                    events.extend(crate::selectivity::dispatch(v));
                } else {
                    match crate::curated::ORG_REACTIONS
                        .iter()
                        .find(|r| r.name == reaction)
                    {
                        None => {
                            // The parser vets names, but an operator can arrive
                            // by JSON; refuse out loud rather than panic.
                            events.push(Event::NotYetModeled {
                                cause: crate::ops::NotModelledCause::NotParameterised,
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
                                let needs: Vec<&str> =
                                    r.reactants.iter().map(|(k, _)| *k).collect();
                                events.push(Event::NotYetModeled { cause: crate::ops::NotModelledCause::NothingToActOn,
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
        let (vessel, titrant, concentration, step, target_ph, max_steps, endpoint) = match &op {
            Operator::Titrate {
                vessel,
                titrant,
                concentration,
                step,
                target_ph,
                max_steps,
                endpoint,
            } => (
                *vessel,
                titrant.clone(),
                *concentration,
                *step,
                *target_ph,
                *max_steps,
                *endpoint,
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
        // EXP-39: the redox half of the same curve, sparse by design —
        // a step where the engine withholds pe contributes no point,
        // because at equivalence pe is undefined rather than large.
        let mut pe_curve: Vec<(f64, f64)> = Vec::new();

        // Read initial pH if available.
        {
            let v = self.vessel(vessel)?;
            if let Some(info) = &v.solution {
                curve.push((0.0, info.ph));
                if let Some(pe) = info.pe {
                    pe_curve.push((0.0, pe));
                }
            }
        }

        // EXP-39: the flask as the eye finds it before the first drop.
        // A self-indicating endpoint is a *change* from this, not an
        // absolute colour — a titration into an already-yellow solution
        // still has an endpoint, and hard-coding "not colourless" would
        // have declared it reached before the burette was opened.
        let baseline_colour = liquid_colour_word_of(self.vessel(vessel)?);

        let mut total_volume = Liters(0.0);
        let mut reached = false;
        let mut pe_ever_pinned = false;

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
            // The colour is read from the same equilibrated vessel, so a
            // colour seen here is a colour that stays: the endpoint of a
            // self-indicating titration is a statement about equilibrium,
            // not about how long you stand and watch.
            let colour = liquid_colour_word_of(v);
            match &v.solution {
                Some(info) => {
                    let ml = total_volume.0 * 1000.0;
                    let ph = info.ph;
                    let prev_ph = curve.last().map(|&(_, p)| p);
                    curve.push((ml, ph));
                    if let Some(pe) = info.pe {
                        pe_curve.push((ml, pe));
                        pe_ever_pinned = true;
                    }
                    let arrived = match endpoint {
                        Endpoint::Ph => prev_ph.is_some_and(|prev| {
                            (prev <= target_ph && ph >= target_ph)
                                || (prev >= target_ph && ph <= target_ph)
                        }),
                        Endpoint::Pe { compare, value } => {
                            info.pe.is_some_and(|pe| compare.holds(pe, value))
                        }
                        Endpoint::ColourPersists => colour != baseline_colour,
                    };
                    if arrived {
                        reached = true;
                        break;
                    }
                }
                None => {
                    // Without a characterised solution there is no pH and
                    // no speciation, so no endpoint of any kind can be
                    // read. The colour endpoint is the one exception worth
                    // naming: it is read off the vessel, not the solve —
                    // but an unsolved vessel holds the titrant as the
                    // solid it was added as, and a solid has no ε(λ).
                    events.push(Event::NotYetModeled {
                        cause: crate::ops::NotModelledCause::NoSolver,
                        vessel,
                        what: "titration needs an aqueous solver to compute pH \
                               after each addition — none is wired"
                            .to_string(),
                    });
                    break;
                }
            }
        }

        // CAP-12 promised a titration that "refuses politely when no
        // endpoint is reachable and says why". The pH endpoint keeps its
        // original silence — changing it would rewrite curves that are
        // already pinned — but the EXP-39 endpoints say so, and say which
        // of the two ways they failed: the target was never met, or, for
        // the potentiometric one, no potential was ever definable at all.
        // Those are different findings and a reader is owed the difference:
        // a flask holding only the reduced half of a couple has no
        // potential, and reporting that as "pe never got high enough"
        // would invent a measurement to explain a missing one.
        if !reached && !curve.is_empty() {
            let what = match endpoint {
                Endpoint::Ph => None,
                Endpoint::Pe { compare, value } if !pe_ever_pinned => Some(format!(
                    "the burette ran to its {max_steps}-step limit without reaching \
                     pe {} {value}: no potential was pinned at any point in this \
                     titration. With only one oxidation state of a couple in the \
                     flask the electron balance has no root, so pe is undefined \
                     rather than low — add the other half of a redox couple, or \
                     titrate to a colour instead",
                    compare.symbol()
                )),
                Endpoint::Pe { compare, value } => Some(format!(
                    "the burette ran to its {max_steps}-step limit without reaching \
                     pe {} {value}; the last potential this flask pinned was {:.2}",
                    compare.symbol(),
                    pe_curve.last().map(|&(_, pe)| pe).unwrap_or(f64::NAN),
                )),
                Endpoint::ColourPersists => Some(format!(
                    "the burette ran to its {max_steps}-step limit and the liquid is \
                     still {baseline_colour}: either the endpoint is further away \
                     than {max_steps} increments, or nothing here carries a curated \
                     absorption spectrum for the eye to read. Raise `max`, or \
                     titrate to a pH or a pe instead"
                )),
            };
            if let Some(what) = what {
                events.push(Event::NotYetModeled {
                    vessel,
                    what,
                    cause: crate::ops::NotModelledCause::ModelBoundary,
                });
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
                pe_curve,
                endpoint_reached: Some(reached),
                endpoint,
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

/// Advance the slow clocks for one vessel. `WAIT` calls this for the whole
/// bench; timed apparatus calls it only for the vessel being operated.
/// The clocks themselves, and their order, live in `clock.rs`.
fn advance_prepared_objects(vessel: &mut Vessel, seconds: f64, events: &mut Vec<Event>) {
    if seconds <= 0.0 {
        return;
    }
    let volume_l = vessel.liquid_volume().0.max(1e-9);
    let external_osmolarity = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 != "water" && matches!(p.phase, Phase::Aqueous | Phase::Liquid))
        .map(|p| p.moles.0)
        .sum::<f64>()
        / volume_l;
    let external_water = vessel
        .contents
        .iter()
        .find(|p| p.species.0 == "water")
        .map_or(0.0, |p| p.moles.0);
    let ascorbate = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "ascorbic_acid")
        .map(|p| p.moles.0)
        .sum::<f64>();
    let oxygen_fraction = if matches!(vessel.headspace, Headspace::Open) {
        0.21
    } else {
        0.0
    };
    let mut water_delta = 0.0;
    for object in &mut vessel.material_objects {
        let Some(recipe) = material::lookup(&object.material, None) else {
            continue;
        };
        object.state.elapsed_seconds += seconds;
        if let Some(internal) = recipe.roles.iter().find_map(|role| match role {
            MaterialRole::OsmoticMembrane {
                internal_osmolarity_mol_per_litre,
            } => Some(*internal_osmolarity_mol_per_litre),
            _ => None,
        }) {
            if let Some(prediction) =
                crate::kitchen_biology::egg_osmosis(internal, external_osmolarity, seconds)
            {
                if let Some(water) = object
                    .components
                    .iter_mut()
                    .find(|c| c.species.0 == "water")
                {
                    let requested = water.moles.0 * prediction.water_fraction;
                    let signed = match prediction.direction {
                        crate::kitchen_biology::OsmosisDirection::IntoObject => {
                            requested.min((external_water - water_delta).max(0.0))
                        }
                        crate::kitchen_biology::OsmosisDirection::OutOfObject => {
                            -requested.min(water.moles.0)
                        }
                        crate::kitchen_biology::OsmosisDirection::Balanced => 0.0,
                    };
                    water.moles.0 += signed;
                    water_delta += signed;
                    let grams = signed
                        * species::lookup(&SpeciesId::new("water"))
                            .map_or(18.01528, |d| d.molar_mass);
                    object.mass_g += grams;
                    object.state.exchanged_water_moles += signed;
                    if signed.abs() > 1e-15 {
                        events.push(Event::OsmosisChanged {
                            vessel: vessel.id,
                            material: object.material.clone(),
                            water_moles: signed,
                            mass_change_g: grams,
                        });
                    }
                }
            }
        }
        if recipe
            .roles
            .iter()
            .any(|role| matches!(role, MaterialRole::BrowningSurface))
        {
            if let Some(fraction) = crate::kitchen_biology::apple_browning(
                object.state.elapsed_seconds,
                oxygen_fraction,
                ascorbate / object.mass_g.max(1e-9),
            ) {
                if fraction > object.state.browned_fraction + 1e-9 {
                    object.state.browned_fraction = fraction;
                    events.push(Event::BrowningChanged {
                        vessel: vessel.id,
                        material: object.material.clone(),
                        browned_fraction: fraction,
                    });
                }
            }
        }
    }
    if water_delta > 0.0 {
        vessel.withdraw(&SpeciesId::new("water"), Moles(water_delta));
    } else if water_delta < 0.0 {
        vessel.deposit(SpeciesId::new("water"), Moles(-water_delta), Phase::Liquid);
    }
}

fn precipitate_declared_soap(
    vessel: &mut Vessel,
    recipe: &MaterialRecipe,
    events: &mut Vec<Event>,
) {
    let Some(eq_per_g) = recipe.roles.iter().find_map(|role| match role {
        MaterialRole::FattySoapEquivalent { moles_per_gram } => Some(*moles_per_gram),
        _ => None,
    }) else {
        return;
    };
    let Some(index) = vessel
        .unresolved_materials
        .iter()
        .rposition(|p| p.recipe_id == recipe.id)
    else {
        return;
    };
    let soap_g = vessel.unresolved_materials[index].amount;
    let ca = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "Ca+2")
        .map(|p| p.moles.0)
        .sum::<f64>();
    let mg = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "Mg+2")
        .map(|p| p.moles.0)
        .sum::<f64>();
    let Some(scum) = crate::kitchen_biology::soap_scum(ca + mg, soap_g * eq_per_g) else {
        return;
    };
    if scum.divalent_ion_bound_moles <= 0.0 {
        return;
    }
    let ca_take = scum.divalent_ion_bound_moles.min(ca);
    let mg_take = scum.divalent_ion_bound_moles - ca_take;
    let soap_mass_g = scum.soap_bound_moles / eq_per_g;
    let ion_mass_g = ca_take
        * species::lookup(&SpeciesId::new("Ca+2")).map_or(40.078, |data| data.molar_mass)
        + mg_take * species::lookup(&SpeciesId::new("Mg+2")).map_or(24.305, |data| data.molar_mass);
    let sodium_mass_g = scum.soap_bound_moles
        * species::lookup(&SpeciesId::new("Na+")).map_or(22.989_769, |data| data.molar_mass);
    // The aggregate receives exactly the consumed sodium-soap mass plus the
    // bound Ca/Mg mass, less the sodium returned to solution. This keeps the
    // ledger exact for calcium, magnesium, and mixtures of the two.
    let aggregate_mass_g = soap_mass_g + ion_mass_g - sodium_mass_g;
    vessel.withdraw(&SpeciesId::new("Ca+2"), Moles(ca_take));
    vessel.withdraw(&SpeciesId::new("Mg+2"), Moles(mg_take));
    vessel.deposit(
        SpeciesId::new("Na+"),
        Moles(scum.soap_bound_moles),
        Phase::Aqueous,
    );
    vessel.unresolved_materials[index].amount =
        (soap_g - scum.soap_bound_moles / eq_per_g).max(0.0);
    let state = vessel.soap_scum.get_or_insert_with(Default::default);
    state.aggregate_mass_g += aggregate_mass_g;
    state.divalent_ion_moles += scum.divalent_ion_bound_moles;
    state.soap_equivalent_moles += scum.soap_bound_moles;
    events.push(Event::SoapScumFormed {
        vessel: vessel.id,
        aggregate_mass_g,
        divalent_ion_moles: scum.divalent_ion_bound_moles,
    });
}

fn advance_vessel_time(
    vessel: &mut Vessel,
    seconds: f64,
    settle_under_gravity: bool,
    kinetic_context: crate::kinetics::KineticContext,
    events: &mut Vec<Event>,
) -> Result<(), BenchError> {
    crate::clock::advance(
        vessel,
        seconds,
        crate::clock::ClockContext {
            settle_under_gravity,
            kinetic: kinetic_context,
        },
        events,
    )?;
    advance_prepared_objects(vessel, seconds.max(0.0), events);
    Ok(())
}

/// KID-11: how much gas was made in this vessel this step, from whichever
/// engine saw it.
///
/// Two engines report gas and they report it in different words. A curated
/// kinetic reaction announces `GasProduced`; the aqueous solver announces
/// `GasEvolved` when gas leaves an open vessel and `GasContained` when a
/// closed one keeps it. A baking-soda volcano is entirely the second kind
/// — 0.049 mol of carbon dioxide, in two parcels, with no `GasProduced`
/// anywhere — and peroxide is entirely the first.
///
/// The two totals are combined with `max`, not `+`. They are two VIEWS of
/// one step, and a parcel that both engines described would otherwise be
/// counted twice, which would inflate an observable rather than a number
/// anyone can check. No shipped path reports the same parcel both ways
/// today; `max` means that if one ever does, the foam is under-claimed
/// rather than doubled, and under-claiming is the safe direction for a
/// bounded teaching observable.
///
/// Returns the gas and the index of the last event that reported it, so
/// the foam can be told beside its own cause. A lesson like
/// `elephant-toothpaste-catalyst-dose` is a two-vessel fair comparison,
/// and a `wait` touches both: appending every vessel's foam after every
/// vessel's chemistry groups the transcript by event kind instead of by
/// vessel, which is the wrong axis for the one thing that lesson exists to
/// let a learner do. The gas line, then what the gas did.
fn gas_made_this_step(events: &[Event], vessel: VesselId) -> (f64, Option<usize>) {
    let mut produced = 0.0;
    let mut reported = 0.0;
    let mut last = None;
    for (index, event) in events.iter().enumerate() {
        match event {
            Event::GasProduced {
                vessel: id, moles, ..
            } if *id == vessel => {
                produced += moles.0;
                last = Some(index);
            }
            Event::GasEvolved {
                vessel: id, moles, ..
            }
            | Event::GasContained {
                vessel: id, moles, ..
            } if *id == vessel => {
                reported += moles.0;
                last = Some(index);
            }
            _ => {}
        }
    }
    (produced.max(reported), last)
}

/// BRD-002: carry a typed shelf refusal into the event stream so it reaches
/// the register in all three voices.
///
/// It is an event and not a returned `Err` on purpose, and the choice is
/// the same one `SafetyVeto` already made: a refusal the learner caused is
/// something the lab should *say*, in the journal, next to everything else
/// that happened — not an error string thrown past the narrative. The typed
/// form still exists ([`crate::stock::StockRefusal`], and
/// [`BenchError::StockExhausted`] for callers who reach the ledger
/// directly); this is how it speaks.
fn stock_refusal_event(key: &str, refusal: crate::stock::StockRefusal) -> Event {
    let crate::stock::StockRefusal::Exhausted {
        requested,
        remaining,
        unit,
    } = refusal;
    Event::StockExhausted {
        key: key.to_string(),
        requested,
        remaining,
        unit,
    }
}

/// Which vessels an operator touches (for re-equilibration).
fn op_touches(op: &Operator) -> Vec<VesselId> {
    match op {
        Operator::NewVessel { .. }
        | Operator::RemoveVessel { .. }
        // Stocking the shelf changes the cabinet, not a vessel.
        | Operator::StockShelf { .. } => vec![],
        Operator::Add { vessel, .. }
        | Operator::AddMaterial { vessel, .. }
        | Operator::Heat { vessel, .. }
        | Operator::Cool { vessel, .. }
        | Operator::Stir { vessel, .. }
        | Operator::Seal { vessel, .. }
        | Operator::Regulate { vessel, .. }
        | Operator::Sweep { vessel, .. }
        | Operator::Open { vessel } => vec![*vessel],
        Operator::Evaporate { vessel, .. } | Operator::Ignite { vessel } => vec![*vessel],
        // Electrolysis moves matter, so the vessel is re-settled after it.
        Operator::Electrolyse { vessel, .. } => vec![*vessel],
        Operator::Spill { from, .. } => vec![*from],
        Operator::Impact { vessel, .. } => vec![*vessel],
        Operator::RecoverSpill { to, .. } => vec![*to],
        Operator::Decant { from, to, .. }
        | Operator::Filter { from, to }
        | Operator::Magnet { from, to }
        | Operator::Distil { from, to, .. }
        | Operator::Drain { from, to } => vec![*from, *to],
        Operator::Mix { a, b, into, .. } => vec![*a, *b, *into],
        Operator::Grind { vessel, .. }
        | Operator::Centrifuge { vessel, .. }
        | Operator::Irradiate { vessel, .. }
        | Operator::Dilute { vessel, .. }
        | Operator::React { vessel, .. }
        | Operator::SpikeNuclide { vessel, .. }
        | Operator::Smell { vessel }
        | Operator::Particles { vessel }
        | Operator::TestGas { vessel, .. }
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

/// Conservative deterministic break thresholds in N·s. Unknown glassware
/// uses the beaker threshold; replay therefore never depends on scene timing.
fn impact_threshold_ns(kind: &str) -> f64 {
    match kind {
        "tube" => 0.8,
        "flask" => 1.2,
        "cylinder" => 1.0,
        "crucible" => 2.5,
        _ => 1.5,
    }
}

fn material_amount_to_moles(
    recipe: &MaterialRecipe,
    amount: f64,
    data: &species::SpeciesData,
) -> Moles {
    match recipe.basis {
        MaterialBasis::MassFraction => data.moles_from_grams(Grams(amount)),
        MaterialBasis::MoleFraction => Moles(amount),
        MaterialBasis::VolumeFraction => data.moles_from_liters(Liters(amount / 1000.0)),
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
/// Whether a dissolved species carries a charge.
///
/// The line between "outside the method" and "outside the model" for a
/// partition column: a column separates by how a NEUTRAL solute divides
/// between two phases, so an ion is genuinely outside the method however
/// good the parameters get — it wants ion exchange. A neutral solute with
/// no curated decomposition is merely unparameterised, and a real column
/// would separate it.
fn is_ionic(species: &SpeciesId) -> bool {
    crate::stoich::parse_formula(&species.0)
        .map(|f| f.charge != 0.0)
        .unwrap_or(false)
}

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

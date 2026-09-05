//! The slow clock: every engine that moves a vessel through time, in one
//! fixed order, each a named member — the `wait`-side twin of the solver
//! stack in `solve.rs`.
//!
//! `bench.rs` held this as one long function that called each engine by
//! hand: emulsion, gravity, decay, the curated kinetics, fermentation, the
//! enzymes, every one spliced in where its experiment was added. The
//! ORDER is physics and stays fixed here — operator splitting, the slow
//! processes first and the fast equilibria re-settled afterwards by the
//! solver stack, because equilibrium is the faster process — and a test
//! pins it. What changes is the cost of the next slow engine: one `Clock`
//! and one line in `standard_clocks`, not another paragraph in the bench.
//!
//! A refactor and nothing else: the events, their order and their numbers
//! are the ones the bench emitted before. The golden lessons are the proof.

use crate::kinetics::{IntegrationError, KineticContext};
use crate::ops::{CentrifugeSeparation, Event};
use crate::species::{self, Phase, SpeciesId};
use crate::units::{Kelvin, Moles};
use crate::vessel::{ThermalMode, Vessel};

/// What the caller knows about this passage of time.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClockContext {
    /// Whether suspended solids may settle: `wait` says yes; a turning
    /// stir bar says no.
    pub settle_under_gravity: bool,
    pub kinetic: KineticContext,
}

/// One slow engine. `advance` moves the vessel by `seconds` and appends
/// the events that describe what moved; it may not reorder or re-settle
/// anything another clock owns.
pub trait Clock {
    fn name(&self) -> &'static str;
    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError>;
}

/// The standard clocks, in the standard order.
pub fn standard_clocks() -> Vec<Box<dyn Clock>> {
    vec![
        Box::new(EmulsionClock),
        Box::new(GravityClock),
        Box::new(ElapsedClock),
        Box::new(DecayClock),
        Box::new(CuratedKineticsClock),
        Box::new(FermentationClock),
        Box::new(EnzymeClock),
    ]
}

/// Run every standard clock over one vessel. `Bench` calls this for
/// `wait` on every vessel and for timed apparatus on the one operated.
pub fn advance(
    vessel: &mut Vessel,
    seconds: f64,
    ctx: ClockContext,
    events: &mut Vec<Event>,
) -> Result<(), IntegrationError> {
    let seconds = seconds.max(0.0);
    for clock in standard_clocks() {
        clock.advance(vessel, seconds, &ctx, events)?;
    }
    Ok(())
}

/// An emulsion left standing separates on its half-life.
pub struct EmulsionClock;

impl Clock for EmulsionClock {
    fn name(&self) -> &'static str {
        "emulsion"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        let emulsion_before = crate::emulsion::observe(vessel);
        crate::emulsion::advance(vessel, seconds);
        if let Some(before) = emulsion_before {
            let after = crate::emulsion::observe(vessel);
            let after_fraction = after
                .as_ref()
                .map(|observation| observation.dispersed_fraction)
                .unwrap_or(0.0);
            if before.dispersed_fraction > after_fraction + 1e-9 {
                events.push(Event::EmulsionChanged {
                    vessel: vessel.id,
                    material: before.material,
                    from_dispersed_fraction: before.dispersed_fraction,
                    to_dispersed_fraction: after_fraction,
                    dispersed_volume_l: after
                        .map(|observation| observation.dispersed_volume_l)
                        .unwrap_or(0.0),
                    half_life_seconds: before.half_life_seconds,
                });
            }
        }
        Ok(())
    }
}

/// Suspended solids settle under gravity — unless a stir bar says not.
pub struct GravityClock;

impl Clock for GravityClock {
    fn name(&self) -> &'static str {
        "gravity"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        if ctx.settle_under_gravity {
            let settled = settle_vessel_under_gravity(vessel, seconds);
            if !settled.is_empty() {
                events.push(Event::GravitySettled {
                    vessel: vessel.id,
                    seconds,
                    separations: settled,
                });
            }
        }
        Ok(())
    }
}

/// The vessel's own clock hand. It sits here, between settling and decay,
/// because that is where the bench moved it; the clocks after it read
/// `elapsed_seconds` as already advanced.
pub struct ElapsedClock;

impl Clock for ElapsedClock {
    fn name(&self) -> &'static str {
        "elapsed"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        _events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        vessel.elapsed_seconds += seconds;
        Ok(())
    }
}

/// EXP-49: radioactive decay, the slowest clock on the bench.
pub struct DecayClock;

impl Clock for DecayClock {
    fn name(&self) -> &'static str {
        "decay"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        // EXP-49: decay is the slowest clock on the bench; it runs beside
        // kinetics on the same shared time.
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
        Ok(())
    }
}

/// The curated rate laws (`kinetics.rs`), integrated over the interval,
/// with the peroxide decomposition's heat applied to an adiabatic vessel.
pub struct CuratedKineticsClock;

impl Clock for CuratedKineticsClock {
    fn name(&self) -> &'static str {
        "curated-kinetics"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        for (reaction, moles) in
            crate::kinetics::advance_with_context(vessel, seconds, ctx.kinetic)?
        {
            if reaction.id == "peroxide-decomposition" {
                // 2 H2O2(l) -> 2 H2O(l) + O2(g), approximately -98.2 kJ
                // per stoichiometric extent at 25 °C.
                let energy_j = 98_200.0 * moles.0;
                let from = vessel.temperature;
                if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
                    let heat_capacity = vessel.heat_capacity();
                    if heat_capacity > 0.0 {
                        vessel.temperature = Kelvin(from.0 + energy_j / heat_capacity);
                    }
                }
                if moles.0 >= crate::OBSERVABLE_MOLES {
                    events.push(Event::GasProduced {
                        vessel: vessel.id,
                        reaction: reaction.id.to_string(),
                        species: SpeciesId::new("O2"),
                        moles,
                        rate_moles_per_second: if seconds > 0.0 {
                            moles.0 / seconds
                        } else {
                            0.0
                        },
                    });
                    events.push(Event::ReactionHeatReleased {
                        vessel: vessel.id,
                        reaction: reaction.id.to_string(),
                        energy_j,
                    });
                    if (vessel.temperature.0 - from.0).abs() > 1e-9 {
                        events.push(Event::TemperatureChanged {
                            vessel: vessel.id,
                            from,
                            to: vessel.temperature,
                        });
                    }
                }
            }
            if moles.0 < crate::OBSERVABLE_MOLES {
                continue;
            }
            let (ea, catalyst) =
                reaction.effective_activation_energy_with_context(vessel, ctx.kinetic);
            events.push(Event::Reacted {
                vessel: vessel.id,
                reaction: reaction.id.to_string(),
                equation: reaction.equation.to_string(),
                moles,
                seconds,
                catalyst: catalyst.map(|c| {
                    species::lookup_key(c.species)
                        .map(|data| data.name.to_string())
                        .unwrap_or_else(|| c.species.to_string())
                }),
                activation_energy: ea,
            });
        }
        Ok(())
    }
}

/// Yeast on sucrose.
pub struct FermentationClock;

impl Clock for FermentationClock {
    fn name(&self) -> &'static str {
        "fermentation"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        if let Some(step) = crate::fermentation::advance(vessel, seconds) {
            if step.sucrose_moles >= crate::OBSERVABLE_MOLES {
                events.push(Event::GasProduced {
                    vessel: vessel.id,
                    reaction: "yeast-sucrose-fermentation".to_string(),
                    species: SpeciesId::new("CO2"),
                    moles: Moles(step.carbon_dioxide_moles),
                    rate_moles_per_second: step.carbon_dioxide_moles / seconds.max(f64::EPSILON),
                });
                events.push(Event::Fermented {
                    vessel: vessel.id,
                    sucrose_moles: Moles(step.sucrose_moles),
                    ethanol_moles: Moles(step.ethanol_moles),
                    carbon_dioxide_moles: Moles(step.carbon_dioxide_moles),
                    active_yeast_grams: step.active_yeast_grams,
                    seconds,
                });
            }
        }
        Ok(())
    }
}

/// The bounded enzyme families over their substrates.
pub struct EnzymeClock;

impl Clock for EnzymeClock {
    fn name(&self) -> &'static str {
        "enzymes"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        for step in crate::enzyme_activity::advance(vessel, seconds) {
            events.push(Event::EnzymeHydrolysed {
                vessel: vessel.id,
                family: step.family,
                material: step.material,
                substrate: step.substrate.to_string(),
                hydrolysed_mass_g: step.hydrolysed_mass_g,
                converted_fraction: step.converted_fraction,
                seconds,
            });
        }
        Ok(())
    }
}

pub(crate) fn settle_vessel_under_gravity(
    vessel: &mut Vessel,
    seconds: f64,
) -> Vec<CentrifugeSeparation> {
    if seconds <= 0.0 || vessel.liquid_volume().0 <= 0.0 {
        return Vec::new();
    }
    let liquid_volume_l = vessel.liquid_volume().0;
    let liquid_mass_g: f64 = vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Liquid)
        .filter_map(|portion| {
            species::lookup(&portion.species).map(|data| portion.moles.0 * data.molar_mass)
        })
        .sum();
    let fluid_density_kg_m3 = liquid_mass_g / liquid_volume_l;
    let Ok(viscosity) = crate::properties::water_viscosity_cp(vessel.temperature.0) else {
        return Vec::new();
    };
    let dynamic_viscosity_pa_s = viscosity.value / 1000.0;
    let mut separations = Vec::new();

    for lot in &mut vessel.lots {
        if lot.phase != Phase::Solid || lot.suspended_fraction.unwrap_or(0.0) <= 0.0 {
            continue;
        }
        let Some(data) = species::lookup(&lot.species) else {
            continue;
        };
        let particle_density_kg_m3 = data.density * 1000.0;
        if particle_density_kg_m3 <= fluid_density_kg_m3 {
            // Creaming needs a top-layer state; do not paint it as sediment.
            continue;
        }
        let particle_size_assumed = lot.particle_size_um.is_none();
        let particle_diameter_um = lot.particle_size_um.unwrap_or(100.0);
        let Ok(result) = crate::centrifuge::sediment(crate::centrifuge::SedimentationInput {
            seconds,
            path_m: 0.04,
            acceleration_m_s2: 9.806_65,
            particle_diameter_m: particle_diameter_um * 1e-6,
            particle_density_kg_m3,
            fluid_density_kg_m3,
            dynamic_viscosity_pa_s,
        }) else {
            continue;
        };
        if result.separated_fraction <= 1e-12 {
            continue;
        }
        lot.suspended_fraction =
            Some(lot.suspended_fraction.unwrap_or(1.0) * (1.0 - result.separated_fraction));
        separations.push(CentrifugeSeparation {
            species: lot.species.clone(),
            particle_diameter_um,
            particle_size_assumed,
            particle_density_kg_m3,
            terminal_speed_m_s: result.terminal_speed_m_s,
            distance_m: result.distance_m,
            separated_fraction: result.separated_fraction,
            direction: result.direction,
        });
    }
    if !separations.is_empty() {
        vessel.resolved.invalidate();
    }
    separations
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The order is physics, not history: pinned so that a new clock is
    /// placed on purpose and a reordering is a reviewed change.
    #[test]
    fn the_standard_clocks_keep_their_order() {
        let names: Vec<&str> = standard_clocks().iter().map(|c| c.name()).collect();
        assert_eq!(
            names,
            [
                "emulsion",
                "gravity",
                "elapsed",
                "decay",
                "curated-kinetics",
                "fermentation",
                "enzymes"
            ]
        );
    }

    #[test]
    fn a_negative_interval_is_no_time_at_all() {
        let mut v = Vessel::new(crate::vessel::VesselId(0), "v1");
        let mut events = Vec::new();
        advance(&mut v, -5.0, ClockContext::default(), &mut events).expect("advances");
        assert!(events.is_empty());
        assert_eq!(v.elapsed_seconds, 0.0);
    }
}

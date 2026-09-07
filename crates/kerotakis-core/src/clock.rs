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
use crate::solve::Equilibrator;
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
        Box::new(GasMechanismClock),
        Box::new(FermentationClock),
        Box::new(EnzymeClock),
        Box::new(GasExchangeClock),
        Box::new(AmbientClock),
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
        let mut energy_j = 0.0;
        for step in crate::nuclide::advance(&mut vessel.nuclides, seconds) {
            energy_j += step.energy_j;
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
        // th-122: a tracer is spiked into the nuclide ledger, but a block
        // of uranium is a chemical portion, and the two are deliberately
        // separate stores. A bulk radionuclide decays all the same, so its
        // heat is counted from the contents. The block is NOT transmuted:
        // a day turns four parts in ten trillion of it into thorium, and
        // ledgering that would be false precision dressed as conservation.
        for portion in &vessel.contents {
            if portion.phase != Phase::Solid {
                continue;
            }
            if let Some(heat) =
                crate::nuclide::bulk_decay_heat(&portion.species.0, portion.moles.0, seconds)
            {
                energy_j += heat.energy_j;
            }
        }
        // Decay heat is heat like any other: it raises an adiabatic
        // vessel's temperature through its own heat capacity, and a
        // thermostatted one not at all. The event is pushed either way,
        // because "the block released this much and the bath took it" is
        // an answer and a silent nothing is not.
        if energy_j > 0.0 {
            let from = vessel.temperature;
            if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
                let heat_capacity = vessel.heat_capacity();
                if heat_capacity > 0.0 {
                    vessel.temperature = Kelvin(from.0 + energy_j / heat_capacity);
                }
            }
            events.push(Event::ReactionHeatReleased {
                vessel: vessel.id,
                reaction: "radioactive-decay".to_string(),
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

/// BRD-041: the shipped gas-phase mechanism packs, run on any vessel that
/// holds two or more of a pack's species as gas, with the heat of what
/// burned applied to an adiabatic vessel.
///
/// The integrator is isothermal inside the interval and the heat lands at
/// its end, as the standard formation-enthalpy difference over what
/// reacted: no dissociation, no temperature-dependent heat capacity. That
/// is the same approximation the peroxide step above makes, stated here
/// because a burn is a bigger number. Where CEA sits in the solver stack
/// it re-equilibrates the hot products afterwards and owns the flame
/// temperature; the packs own the time.
///
/// Where the rate laws are zero — a cold mixture, a mixture with no
/// radical to carry a chain, a global step outside its fitted window —
/// the integrator returns in a handful of evaluations and nothing is
/// said, which is also what happens on a real bench.
pub struct GasMechanismClock;

/// Tolerances for a stiff chain: extents in nanomoles and a first step
/// short enough to see the induction period.
const GAS_MECHANISM_OPTIONS: crate::kinetics::IntegrationOptions =
    crate::kinetics::IntegrationOptions {
        relative_tolerance: 1e-8,
        absolute_tolerance_moles: 1e-16,
        initial_step_seconds: 1e-9,
    };

impl Clock for GasMechanismClock {
    fn name(&self) -> &'static str {
        "gas-mechanisms"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        if seconds <= 0.0 {
            return Ok(());
        }
        // One pack per vessel per interval — see `packs::pack_for`.
        if let Some(pack) = crate::kinetics::packs::pack_for(vessel) {
            let report = crate::kinetics::advance_network_with_options(
                vessel,
                seconds,
                &pack.network,
                GAS_MECHANISM_OPTIONS,
            )?;
            let mut released_j = 0.0;
            for (reaction, moles) in &report.extents {
                if let Some(enthalpy) = pack.reaction_enthalpy_j_per_mol(reaction) {
                    released_j -= enthalpy * moles.0;
                }
                if moles.0.abs() < crate::OBSERVABLE_MOLES {
                    continue;
                }
                events.push(Event::Reacted {
                    vessel: vessel.id,
                    reaction: reaction.id.to_string(),
                    equation: reaction.equation.to_string(),
                    moles: *moles,
                    seconds,
                    catalyst: None,
                    activation_energy: reaction.forward.arrhenius.activation_energy,
                });
            }
            if released_j.abs() < 1e-9 {
                return Ok(());
            }
            let from = vessel.temperature;
            if matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
                let heat_capacity = vessel.heat_capacity();
                if heat_capacity > 0.0 {
                    vessel.temperature = Kelvin(from.0 + released_j / heat_capacity);
                    vessel.refresh_pressure();
                }
            }
            events.push(Event::ReactionHeatReleased {
                vessel: vessel.id,
                reaction: pack.id.to_string(),
                energy_j: released_j,
            });
            if (vessel.temperature.0 - from.0).abs() > 1e-9 {
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from,
                    to: vessel.temperature,
                });
            }
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

/// The room the bench stands in.
///
/// One number, and the same one `Kelvin::STANDARD` uses, because a
/// teaching bench has exactly one room and every reagent on the shelf is
/// tabulated at it. A settable room temperature is a real feature and this
/// is not it: it would need a `Regulate`-style operator, a saved field and
/// a story about what a "room" is when two benches are open at once.
pub const ROOM_TEMPERATURE: Kelvin = Kelvin(298.15);

/// Below this a temperature move is not announced.
///
/// The same 0.01 K the reconciliation pass in `bench.rs` uses to drop a
/// stale `TemperatureChanged`; stated here so the clock does not emit an
/// event that pass would silently remove.
pub const TEMPERATURE_EVENT_FLOOR_K: f64 = 0.01;

/// Natural-convection heat-transfer coefficient for bench glassware in
/// still room air, W/(m²·K).
///
/// **Where the number comes from.** Bahrami, *Natural Convection*, ENSC
/// 388 (F09) lecture notes, Simon Fraser University
/// (<https://www.sfu.ca/~mbahrami/ENSC%20388/Notes/Natural%20Convection.pdf>,
/// opened 2026-09-06) gives the isothermal vertical plate as
/// `Nu = 0.59 Ra^(1/4)` for `1e4 ≤ Ra ≤ 1e9`, with
/// `Ra = g β (Ts − T∞) L³ / ν² · Pr` and `β = 1/T_film` for an ideal gas.
/// Take a beaker wall as a vertical plate of its own height, `L = 0.095 m`,
/// at `Ts = 70 °C` in a `25 °C` room: the film is 320.7 K, so
/// `β = 3.12e-3 1/K`, and with air at that film temperature
/// (`k = 0.0278 W/(m·K)`, `ν = 17.9e-6 m²/s`, `Pr = 0.703`)
///
/// ```text
/// Ra = 9.81 · 3.12e-3 · 45 · 0.095³ / (17.9e-6)² · 0.703 = 2.6e6
/// Nu = 0.59 · (2.6e6)^(1/4)                             = 23.7
/// h  = Nu · k / L = 23.7 · 0.0278 / 0.095               = 6.9 W/(m²·K)
/// ```
///
/// which is why the constant is 7 and why the familiar "5 to 25 W/(m²·K)
/// for free convection in air" band is the right one to land in. The
/// value is held CONSTANT rather than recomputed per step, and that is an
/// approximation with a direction: `h ∝ ΔT^(1/4)`, so a beaker only two
/// kelvin from the room really loses heat about a third more slowly than
/// this says, and a red-hot one rather faster.
///
/// **Radiation is not modelled at all**, and that is the larger boundary.
/// At 2500 °C a glowing crucible sheds far more by radiation than by
/// convection (σT⁴ against h·ΔT crosses over near 300 °C), so this bench
/// cools a very hot vessel more slowly than a real one. It is stated
/// rather than fudged: adding an emissivity would be a second curated
/// constant and a second claim.
pub const NATURAL_CONVECTION_H_W_PER_M2K: f64 = 7.0;

/// Specific heat capacity of the glass the vessel itself is made of,
/// J/(g·K).
///
/// Borosilicate 3.3 — the laboratory glass a beaker is — is tabulated at
/// about 0.83 kJ/(kg·K) near room temperature by its makers, and the
/// porcelain a crucible is made of is close enough to the same number that
/// this bench does not carry a second one. PENDING REVIEW: no positively
/// identified datasheet page was opened for this row.
pub const WALL_SPECIFIC_HEAT_J_PER_G_K: f64 = 0.83;

/// The geometry the ambient exchange reads off a vessel.
///
/// `VESSEL_KINDS` in `vessel.rs` already turns a label into an optical
/// path length; this is the same idea for heat. Each row is one nominal
/// piece of school glassware, as an equivalent upright cylinder: an
/// outside diameter, a height, and what the empty piece weighs. Nothing
/// here is fitted — they are catalogue dimensions for the commonest size
/// of each kind, and the area and wall heat capacity are arithmetic on
/// them, so a reviewer can check both with a ruler and a balance.
#[derive(Debug, Clone, Copy)]
pub struct GlasswareThermal {
    /// The `Vessel::label` this describes.
    pub kind: &'static str,
    /// Equivalent-cylinder outside diameter, m.
    pub diameter_m: f64,
    /// Height of the wall that faces the room, m.
    pub height_m: f64,
    /// Mass of the empty piece, g.
    pub wall_mass_g: f64,
    pub provenance: &'static str,
}

/// One row per drawable glassware kind, keyed the way `VESSEL_KINDS` is.
pub const GLASSWARE_THERMAL: &[GlasswareThermal] = &[
    GlasswareThermal {
        kind: "beaker",
        diameter_m: 0.070,
        height_m: 0.095,
        wall_mass_g: 100.0,
        provenance: "A 250 mL Griffin low-form borosilicate beaker: 70 mm outside diameter, 95 mm high, about 100 g empty. Side wall plus base is pi*0.070*0.095 + pi*0.070^2/4 = 0.0247 m^2, which is the 0.02 m^2 order of magnitude a bench sheet quotes for one",
    },
    GlasswareThermal {
        kind: "flask",
        diameter_m: 0.065,
        height_m: 0.145,
        wall_mass_g: 120.0,
        provenance: "A 250 mL Erlenmeyer flask, 85 mm across the base and 145 mm high. It is a cone, so the equivalent cylinder takes the MEAN diameter, 65 mm, rather than the base: using the base would over-state the wall by a third",
    },
    GlasswareThermal {
        kind: "tube",
        diameter_m: 0.016,
        height_m: 0.125,
        wall_mass_g: 15.0,
        provenance: "A 16 x 150 mm test tube, about 15 g empty, with 125 mm of it out of the rack and facing the room",
    },
    GlasswareThermal {
        kind: "cylinder",
        diameter_m: 0.026,
        height_m: 0.230,
        wall_mass_g: 90.0,
        provenance: "A 100 mL graduated cylinder, 26 mm across and 230 mm tall on its foot, about 90 g empty",
    },
    GlasswareThermal {
        kind: "crucible",
        diameter_m: 0.040,
        height_m: 0.035,
        wall_mass_g: 25.0,
        provenance: "A 30 mL porcelain crucible, 40 mm across the rim and 35 mm deep, about 25 g. Porcelain, not borosilicate, but its specific heat is within a tenth of the same number and this bench does not carry a second one",
    },
];

/// The geometry for a vessel label; an unknown label reads as the classic
/// beaker, exactly as `vessel::path_cm_for` does for the light path.
pub fn glassware_thermal(label: &str) -> &'static GlasswareThermal {
    GLASSWARE_THERMAL
        .iter()
        .find(|row| row.kind == label)
        .unwrap_or(&GLASSWARE_THERMAL[0])
}

/// Wall area facing the room, m²: an upright cylinder's side plus its base.
///
/// The open mouth is not counted. It is a real path — a hot beaker loses
/// heat upward through the air above it, and faster than through the glass
/// — but it is evaporation and plume convection rather than a wall, and
/// this bench does not model either.
pub fn exchange_area_m2(label: &str) -> f64 {
    let g = glassware_thermal(label);
    std::f64::consts::PI * g.diameter_m * g.height_m
        + std::f64::consts::PI * g.diameter_m * g.diameter_m / 4.0
}

/// Liquid-side mass transfer velocity for CO₂ across a STILL water
/// surface, m/s.
///
/// Cole & Caraco (1998), *Limnology and Oceanography* 43, 647–656,
/// measured gas transfer on Mirror Lake, New Hampshire with an SF₆
/// tracer and fitted
///
/// ```text
/// k600 = 2.07 + 0.215 * U10^1.7      cm/h,   for U10 < 9 m/s
/// ```
///
/// The intercept is the still-air value — 2.07 cm/h = 5.75e-6 m/s — and
/// it is the one a bench wants, because there is no wind over a beaker.
/// The Schmidt number 600 the fit is normalised to IS CO₂ in fresh water
/// at 20 °C, so at bench temperature this already is the CO₂ velocity and
/// takes no Schmidt correction. That is the only reason a limnology
/// number can be carried across without a second conversion nobody
/// checked.
///
/// **It is a lake at low wind and not a beaker, and that is the
/// boundary.** A lake surface at nominally zero wind still has convective
/// overturn and some fetch; a 70 mm beaker has neither, and its walls
/// suppress what little circulation is left. So this is an UPPER bound,
/// and an open vessel here drifts toward the room somewhat faster than a
/// real still one would. Nothing was fitted to make a test pass.
///
/// **Chemical enhancement is not modelled**, and it is the larger
/// omission. In a strongly alkaline solution `CO2 + OH- -> HCO3-` is fast
/// enough to consume CO₂ at the interface and steepen the gradient that
/// drives it; a real bottle of sodium hydroxide skins over with carbonate
/// far faster than this says, by a factor reaching an order of magnitude
/// near pH 12. This clock carries the physical film transfer alone, so it
/// UNDER-states the carbonation of a strong base. Stated rather than
/// fudged, exactly as radiation is for the ambient exchange above.
pub const STILL_SURFACE_K_L_M_PER_S: f64 = 5.75e-6;

/// Partial pressure of CO₂ in room air, atm.
///
/// 3.9083e-4 atm, i.e. 391 ppm. The aqueous adapter has carried this as a
/// log₁₀ for as long as it has had a CO₂ reservoir (`ATMOSPHERIC` in
/// `derived.rs`, −3.408); this is the same number in the units a rate
/// wants, and a test in that crate asserts the two still agree, so a
/// later edit cannot move one without the other.
///
/// It sits a little below the current global mean, which passed 420 ppm
/// some years ago. Left where it was deliberately: raising it moves the
/// equilibrium pH of every open vessel on the bench, which is a decision
/// about the whole corpus and not a detail of this clock.
pub const ATMOSPHERIC_CO2_ATM: f64 = 3.9083e-4;

/// The free liquid surface an open vessel exchanges gas across, m².
///
/// The mouth of the equivalent cylinder, `pi*d^2/4` — the one face
/// [`exchange_area_m2`] deliberately does NOT count, because for heat it
/// is a plume rather than a wall. For gas it is the whole story:
/// dissolved CO₂ leaves, and room CO₂ arrives, through the top and
/// nowhere else.
///
/// It is the vessel's MOUTH and not the liquid's own surface, so a beaker
/// holding 5 mL is credited the same area as one holding 200. That is
/// exact for the cylinder each row is modelled as, and for the conical
/// flask it inherits the same mean-diameter approximation already taken
/// for heat.
pub fn free_surface_area_m2(label: &str) -> f64 {
    let g = glassware_thermal(label);
    std::f64::consts::PI * g.diameter_m * g.diameter_m / 4.0
}

/// What the empty vessel itself can hold, J/K.
pub fn wall_heat_capacity_j_per_k(label: &str) -> f64 {
    glassware_thermal(label).wall_mass_g * WALL_SPECIFIC_HEAT_J_PER_G_K
}

/// `h·A` for a vessel, W/K — the whole ambient model in one number.
///
/// For the default beaker: 7.0 W/(m²·K) × 0.0247 m² = 0.173 W/K, so a
/// beaker 45 K above the room sheds about 7.8 W and a block of dry ice
/// 103.5 K below it takes about 17.9 W.
pub fn ambient_conductance_w_per_k(label: &str) -> f64 {
    NATURAL_CONVECTION_H_W_PER_M2K * exchange_area_m2(label)
}

/// The heat capacity the ambient exchange charges the vessel with, J/K.
///
/// The contents', whenever there are any: that is the same
/// `Vessel::heat_capacity` the `heat` and `cool` operators divide by and
/// the same one `phase_route`'s ledger spends, and the ambient exchange
/// has to use it or the energy it delivers would not be the energy a
/// phase change is allowed to spend. It is per-PHASE — ice is spent at
/// 37.7 J/(mol·K) and steam at 33.6, not at liquid water's 75.3 — so the
/// room melts a beaker of ice on exactly the arithmetic a burner melts it
/// on, and there is no second table anywhere that could disagree.
///
/// The glass wall stands in ONLY for a vessel with nothing in it. The
/// wall is a real thermal mass — a 250 mL beaker holds as much heat as
/// 20 g of water — but the bench's heat ledger does not carry it, and
/// adding it to `Vessel::heat_capacity` would change the answer to every
/// adiabatic mix, every reaction heat and every `heat` on the bench. So
/// it is used where there is nothing else to hold the heat and nowhere
/// else, and the seam is stated rather than hidden: an empty beaker cools
/// on the wall's clock, a beaker with a millilitre in it on the
/// millilitre's, and the second is faster than a real one.
fn ambient_thermal_mass(vessel: &Vessel) -> f64 {
    let contents = vessel.heat_capacity();
    if contents > 0.0 {
        contents
    } else {
        wall_heat_capacity_j_per_k(&vessel.label)
    }
}

/// Does the drift toward the room cross a transition this bench pays
/// latent heat for?
///
/// If not, the vessel is a lumped capacitance and one closed-form
/// exponential is the exact answer for the whole interval. If so, the
/// interval has to be walked, because a substance sitting ON its
/// transition temperature takes every joule as latent heat and does not
/// move the thermometer at all — which is the whole reason dry ice in a
/// beaker is not simply a beaker that warms up.
fn crosses_a_paid_transition(vessel: &Vessel, from: f64, room: f64) -> bool {
    let (lo, hi) = if from <= room {
        (from, room)
    } else {
        (room, from)
    };
    let within = |k: f64| k >= lo && k <= hi;
    vessel.contents.iter().any(|portion| {
        if portion.moles.0 <= 1e-12 {
            return false;
        }
        let key = portion.species.0.as_str();
        // Water's freezing and boiling belong to the solvent model, not
        // to `phase_route`; both are driven below.
        if key == "water" {
            return within(273.15) || within(373.15);
        }
        let Some(data) = species::lookup(&portion.species) else {
            return false;
        };
        let Some(t) = data.transitions else {
            return false;
        };
        let paid = crate::phase_route::sublimation_enthalpy(key).is_some()
            || crate::phase_route::fusion_enthalpy(key).is_some()
            || crate::phase_route::vaporisation_enthalpy(key).is_some();
        let thresholds = [
            paid.then_some(t.melting_k).flatten(),
            paid.then_some(t.boiling_k).flatten(),
            t.sublimation_k,
            t.dehydration_k,
        ];
        thresholds.into_iter().flatten().any(within)
    })
}

/// Let the phase engines see the temperature the room just moved.
///
/// The clock does not reimplement melting, boiling, subliming or
/// dehydrating: it hands the vessel to the two engines that already own
/// them and that the solver stack will run again afterwards anyway. That
/// is the point of doing it here rather than there — the stack runs ONCE
/// per operator, and a block of dry ice needs the heat delivered and spent
/// over and over across half an hour, not once at the end of it.
fn settle_phases(vessel: &mut Vessel, events: &mut Vec<Event>) {
    let id = vessel.id;
    // The solvent model is asked only where there IS solvent: it is the
    // engine that owns ice and steam, and running it over a beaker of dry
    // ice would be several hundred no-ops per wait.
    let water = SpeciesId::new("water");
    if vessel
        .contents
        .iter()
        .any(|portion| portion.species == water && portion.moles.0 > 1e-12)
    {
        let mut states = crate::solve::StateEquilibrator;
        match states.equilibrate(vessel) {
            Ok(mut more) => events.append(&mut more),
            Err(error) => events.push(Event::SolverFailed {
                vessel: id,
                solver: states.name().to_string(),
                detail: error.to_string(),
            }),
        }
    }
    let mut routes = crate::phase_route::PhaseRouteEquilibrator;
    match routes.equilibrate(vessel) {
        Ok(mut more) => events.append(&mut more),
        Err(error) => events.push(Event::SolverFailed {
            vessel: id,
            solver: routes.name().to_string(),
            detail: error.to_string(),
        }),
    }
}

/// How far above (or below) the room a sub-step may drive the thermometer
/// so that the phase ledger can be paid, K.
///
/// It is a carrier, not a claim: the vessel is put back where the physics
/// says within the same sub-step. The number has to clear the largest
/// `ΔH / Cp` any latent heat on this bench asks for, because that is the
/// excursion which lets the LAST of a substance change phase in one
/// sub-step instead of an ever-smaller fraction of it for ever:
///
/// The `Cp` is the one `Vessel::heat_capacity` will actually spend, which
/// since the heat-source ceiling landed is the CONDENSED PHASE's rather
/// than the registry's — ice is 37.7 J/(mol·K), not water's 75.3:
///
/// ```text
/// water, vaporisation    40 650 / 75.3  = 540 K   (the largest)
/// dry ice, sublimation   25 200 / 47.0  = 536 K
/// water, fusion (ice)     6 010 / 37.7  = 159 K
/// water, fusion (liquid)  6 010 / 75.3  =  80 K
/// nitrogen, vaporisation  5 570 / 57.2  =  97 K
/// ethanol, fusion         4 930 / 112.3 =  44 K
/// ```
///
/// 1000 K clears all of them with room to spare. The excursion only
/// approaches it when the vessel's contents have almost no heat capacity
/// left — which is to say when there is almost nothing in the vessel, an
/// unresolved plastic object included, since `Vessel::heat_capacity`
/// counts those too and a vessel holding one never gets near this.
const LEDGER_EXCURSION_K: f64 = 1000.0;

/// The most sub-intervals one `wait` is walked in.
const MAX_AMBIENT_SUBSTEPS: usize = 512;

/// How finely to walk an interval in which a phase change is being paid
/// for: a quarter of the vessel's own relaxation time `C/(h·A)`, never
/// finer than a second, never more than [`MAX_AMBIENT_SUBSTEPS`] pieces.
fn ambient_substeps(seconds: f64, thermal_mass: f64, conductance: f64) -> usize {
    if thermal_mass <= 0.0 || conductance <= 0.0 {
        return 1;
    }
    let step = (0.25 * thermal_mass / conductance).max(1.0);
    let count = (seconds / step).ceil();
    if !count.is_finite() || count < 1.0 {
        return 1;
    }
    (count as usize).min(MAX_AMBIENT_SUBSTEPS)
}

/// Fold the sub-steps' events into what one `wait` should say.
///
/// A hundred sub-steps of subliming dry ice is one thing that happened,
/// not a hundred. Amounts add, a state change is announced once per route
/// it took, and the provisional temperatures are dropped entirely: the
/// clock announces the temperature the vessel ENDS the wait at, once, and
/// `bench.rs` reconciles even that against whatever the solver stack does
/// afterwards.
fn coalesce_substep_events(scratch: Vec<Event>, events: &mut Vec<Event>) {
    let mut out: Vec<Event> = Vec::new();
    for event in scratch {
        match &event {
            Event::TemperatureChanged { .. } => continue,
            Event::GasEvolved {
                vessel,
                species,
                moles,
            } => {
                let (id, key, amount) = (*vessel, species.clone(), *moles);
                if let Some(Event::GasEvolved { moles: total, .. }) =
                    out.iter_mut().find(|candidate| {
                        matches!(candidate, Event::GasEvolved { vessel: v, species: s, .. }
                            if *v == id && *s == key)
                    })
                {
                    total.0 += amount.0;
                    continue;
                }
            }
            Event::GasContained {
                vessel,
                species,
                moles,
            } => {
                let (id, key, amount) = (*vessel, species.clone(), *moles);
                if let Some(Event::GasContained { moles: total, .. }) =
                    out.iter_mut().find(|candidate| {
                        matches!(candidate, Event::GasContained { vessel: v, species: s, .. }
                            if *v == id && *s == key)
                    })
                {
                    total.0 += amount.0;
                    continue;
                }
            }
            other => {
                if out.contains(other) {
                    continue;
                }
            }
        }
        out.push(event);
    }
    events.extend(out);
}

/// The room, at last.
///
/// Every clock above this one is something happening INSIDE the vessel.
/// This one is the vessel sitting in a room, and it runs last because it
/// is the only one whose driving force is a temperature the others may
/// have just changed: a peroxide decomposition warms the beaker in the
/// same wait that the room then starts taking that warmth back.
///
/// Newton's law of cooling, `Q̇ = h·A·(T_room − T)`, with `h·A` from
/// [`ambient_conductance_w_per_k`] and the vessel's own heat capacity as
/// the thermal mass. It is a lumped-capacitance model: one temperature for
/// the whole vessel, no gradient through the glass and none through the
/// contents, which is the standard approximation for a small well-mixed
/// object and is stated on [`NATURAL_CONVECTION_H_W_PER_M2K`] along with
/// the radiation it leaves out.
///
/// **A sealed vessel exchanges heat too.** The seal keeps matter in; it is
/// not a vacuum flask, and the glass conducts either way. What it changes
/// is where the gas goes: cooling a sealed flask drops its pressure and
/// may condense its vapour, and the contents stay in the flask.
///
/// **A thermostatted vessel does not drift at all**, because the bath IS
/// its surroundings, and modelling a water bath as a beaker in a room
/// would be modelling it twice.
///
/// **This is not a second heat-delivery path.** `Bench::deliver_remaining_heat`
/// is what a BURNER does — a dose bounded by the apparatus's own ceiling,
/// offered in passes with the solver stack in scope. This is what a ROOM
/// does over an interval, and it exists here because `wait` has no solver
/// in scope and the room's driving force is a temperature the clocks above
/// have just changed. What the two share is everything that could
/// disagree: `Vessel::heat_capacity` for the thermal mass, and
/// `phase_route`/`solve::StateEquilibrator` for what a phase change costs.
/// Neither owns a joule the other cannot see.
/// Below this, a transfer is not worth a step's arithmetic: a nanomole of
/// CO₂ in a beaker moves the pH by far less than the bench narrates.
const CO2_TRANSFER_FLOOR_MOL: f64 = 1e-9;

/// EXP-57: an open vessel trades CO₂ with the room, and it takes hours.
///
/// Before this, room air could only ever take carbon AWAY. The aqueous
/// adapter offered an open vessel a CO2(g) phase holding zero moles, and a
/// phase with nothing in it can be precipitated into but never dissolved
/// from — so a supersaturated beaker degassed, and an undersaturated one
/// sat there. Worse, the offer was gated on carbon ALREADY being dissolved,
/// so a beaker of distilled water or of sodium hydroxide was not offered
/// the room at all. A bottle of caustic soda going off in the air is one of
/// the first things a bench teaches, and it could not happen here.
///
/// This clock is the inward direction, and it is a rate because that is
/// what it physically is — the same way the room was wrong about heat
/// before the ambient clock: the destination was right and the journey was
/// missing. Outward transport is unchanged and still instantaneous; see
/// the note in `advance` for why that half needs a decision rather than a
/// patch.
///
/// The flux across a still surface is
///
/// ```text
/// dn/dt = k_L * A * ([CO2]_eq - [CO2]_aq)
///       = k_L * A * K_H * (p_air - p_solution)
/// ```
///
/// with `k_L` from [`STILL_SURFACE_K_L_M_PER_S`], `A` the vessel's mouth
/// from [`free_surface_area_m2`], `K_H` the Sander (2015) coefficient
/// `properties.rs` already carries — temperature-corrected, so a warm
/// beaker holds less — and `p_solution` whatever the aqueous solver last
/// measured, read off the vessel because `solution` is wiped every step.
///
/// For the default 250 mL beaker holding 200 mL, `A` is 3.85e-3 m², so
/// `k_L*A/V` is 1.1e-4 s⁻¹ and the time constant is about 2.5 hours. That
/// is the number that matters: a glass left over a lunch break has moved
/// perceptibly, and one left overnight has arrived.
///
/// This clock computes the amount and parks it in
/// [`Vessel::pending_co2_transfer_mol`]; the aqueous tail spends it. It
/// deliberately emits no gas event of its own — the solver that applies
/// the carbon announces it, and KID-11 is explicit that one parcel
/// described by two engines is how an observable comes to be counted
/// twice.
pub struct GasExchangeClock;

impl Clock for GasExchangeClock {
    fn name(&self) -> &'static str {
        "gas-exchange"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        _events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        if seconds <= 0.0 {
            return Ok(());
        }
        // Only a vessel open to the room exchanges with the room. A sealed
        // or swept headspace is a boundary the aqueous adapter already
        // owns, and a second engine moving the same CO₂ would double it.
        if !matches!(vessel.headspace, crate::vessel::Headspace::Open) {
            return Ok(());
        }
        // Nothing has characterised this solution, so there is no measured
        // pressure to drive anything. A vessel nobody has solved is not
        // yet carbonating: the first solve measures it, and the next wait
        // moves it.
        let Some(p_solution) = vessel.co2_partial_pressure_atm else {
            return Ok(());
        };
        let volume_l = vessel.liquid_volume().0;
        if volume_l <= 0.0 {
            return Ok(());
        }
        let Some(coeff) = crate::properties::henry_lookup("CO2") else {
            return Ok(());
        };
        let k_h = crate::properties::henry_at_t(coeff, vessel.temperature.0).value;
        let driving = ATMOSPHERIC_CO2_ATM - p_solution;
        // INWARD only, for now, and the asymmetry is deliberate rather
        // than an oversight.
        //
        // Carbon leaving is already handled, and well: the aqueous adapter
        // offers an open vessel a CO2(g) phase holding zero moles, so a
        // supersaturated solution degasses to the atmospheric value within
        // the step. That is the `dt -> infinity` answer for the outward
        // direction too, and it is wrong in the same way — but replacing
        // it needs a rule this clock cannot yet state, because the two
        // ways carbon leaves a beaker are not one process:
        //
        //   * FIZZ. Vinegar on baking soda. Bubbles nucleate and rise, and
        //     it is over in seconds. Thermodynamically a bubble needs the
        //     dissolved gas above the ambient TOTAL pressure to hold
        //     itself open — but a well-mixed model never sees the local
        //     supersaturation at the grain where the acid lands, so a
        //     1 atm threshold silently stops hydrochloric acid on
        //     bicarbonate from fizzing at all. Measured while writing
        //     this: 0.02 mol in a litre reaches 0.59 atm, and the beaker
        //     went quiet.
        //   * SEEPING. A bicarbonate solution standing on the bench,
        //     drifting basic over days. That IS this film transfer, and
        //     it is what the outward rate would be for.
        //
        // Deciding between them changes what the bench teaches about the
        // most-performed reaction there is, so it is left to a reviewed
        // change of its own rather than settled here as a side effect.
        // Taking the inward direction alone is strictly additive: nothing
        // that degasses today behaves differently, and the direction that
        // could not happen AT ALL now can.
        if driving <= 0.0 {
            return Ok(());
        }

        // `k_L*A` is m³/s and the concentration is mol/L, so the litre has
        // to be spelled out: a cubic metre is a thousand of them. It is
        // the one unit on this bench that is not SI, and the one that
        // quietly loses a factor of a thousand.
        const LITRES_PER_M3: f64 = 1000.0;
        let area = free_surface_area_m2(&vessel.label);
        let mut transfer =
            STILL_SURFACE_K_L_M_PER_S * area * k_h * driving * LITRES_PER_M3 * seconds;

        // The most that can cross before the difference is gone. Past it
        // the vessel would overshoot the room and come back on the next
        // wait, which is an oscillation rather than a slower approach.
        // This is the counterpart of `ambient_substeps`: the same
        // stability problem, settled with one clamp rather than by
        // subdividing, because unlike heat this driving force has no paid
        // transition to cross on the way.
        //
        // **The capacity is not Henry's law alone, and that is the whole
        // difficulty.** Henry says what a litre of water holds as
        // DISSOLVED GAS, and for plain water that is the answer. It is not
        // the answer for anything that CONSUMES the gas: 0.02 mol of
        // sodium hydroxide in 200 mL takes up seven and a half THOUSAND
        // times the Henry capacity, because every CO₂ that arrives becomes
        // bicarbonate and stops exerting a pressure. Clamping there would
        // have admitted one 7500th of the right amount per wait, and the
        // beaker of hydroxide this clock exists to un-break would have sat
        // in the room looking inert — a capacity computed from something
        // that merely correlates with it, holding only while nothing
        // reacts.
        //
        // So the chemical sink is counted too, in the direction it acts:
        //
        //   INWARD  — the free hydroxide the aqueous solver measured, a
        //             CO₂ apiece. It is carried on the vessel for exactly
        //             this class of reason.
        //   OUTWARD — the dissolved carbon actually present, because a
        //             solution cannot give back more than it holds.
        //
        // Both are bounds rather than predictions: the solver re-measures
        // the pressure next step and the rate picks up from there.
        let henry_capacity = k_h * driving.abs() * volume_l;
        let capacity = henry_capacity + vessel.free_hydroxide.max(0.0);
        if transfer.abs() > capacity {
            transfer = capacity.copysign(transfer);
        }
        if transfer.abs() <= CO2_TRANSFER_FLOOR_MOL {
            return Ok(());
        }
        vessel.pending_co2_transfer_mol += transfer;
        Ok(())
    }
}

pub struct AmbientClock;

impl Clock for AmbientClock {
    fn name(&self) -> &'static str {
        "ambient"
    }

    fn advance(
        &self,
        vessel: &mut Vessel,
        seconds: f64,
        _ctx: &ClockContext,
        events: &mut Vec<Event>,
    ) -> Result<(), IntegrationError> {
        if seconds <= 0.0 {
            return Ok(());
        }
        if !matches!(vessel.thermal_mode, ThermalMode::Adiabatic) {
            return Ok(());
        }
        let conductance = ambient_conductance_w_per_k(&vessel.label);
        if conductance <= 0.0 {
            return Ok(());
        }
        let room = ROOM_TEMPERATURE.0;
        let from = vessel.temperature;
        let crossing = crosses_a_paid_transition(vessel, from.0, room);
        // Already the room's own temperature, to within the resolution
        // this bench narrates: nothing to exchange and nothing to say. The
        // floor is deliberately the SAME 0.01 K, so the room does not
        // silently undo a rise the bench has decided is too small to
        // mention — a gram of uranium warming itself by 6 mK in a day
        // keeps those 6 mK, and its steady state against the room is five
        // orders of magnitude below them anyway.
        if !crossing && (from.0 - room).abs() <= TEMPERATURE_EVENT_FLOOR_K {
            return Ok(());
        }

        let mut scratch = Vec::new();
        if crossing {
            let steps = ambient_substeps(seconds, ambient_thermal_mass(vessel), conductance);
            let dt = seconds / steps as f64;
            for _ in 0..steps {
                let capacity = ambient_thermal_mass(vessel);
                if capacity <= 0.0 {
                    break;
                }
                let now = vessel.temperature.0;
                let heading_up = room >= now;
                // The heat the room delivers over this sub-interval does
                // not depend on the vessel's heat capacity at all —
                // `h·A·(T_room − T)·dt` is joules. What the capacity sets
                // is the TEMPERATURE that carries those joules, and the
                // temperature is how `phase_route`'s ledger is paid:
                // its budget is `heat_capacity × (T − threshold)`. So the
                // excursion is deliberately not clamped at the room's own
                // temperature. Clamping it there was a bug with a tell —
                // the last tenth of a gram of dry ice went geometrically
                // rather than linearly and NEVER finished, so the beaker
                // stayed pinned at −78.5 °C for ever holding a
                // hundred-billionth of a mole.
                let excursion = now + conductance * (room - now) * dt / capacity;
                let ceiling = room.max(now) + LEDGER_EXCURSION_K;
                let floor = (room.min(now) - LEDGER_EXCURSION_K).max(0.0);
                vessel.temperature = Kelvin(excursion.clamp(floor, ceiling));
                vessel.refresh_pressure();
                settle_phases(vessel, &mut scratch);
                // Whatever no phase change absorbed was never delivered:
                // the room cannot carry a vessel past its own temperature.
                // Redo the sub-step as the plain lumped exchange it then
                // was, over whatever mass the phase change left behind —
                // which is how the beaker the last of the dry ice just
                // left starts warming on its glass rather than jumping to
                // room temperature.
                let settled = vessel.temperature.0;
                if (heading_up && settled > room) || (!heading_up && settled < room) {
                    let left = ambient_thermal_mass(vessel).max(1e-12);
                    let to = room + (now - room) * (-conductance * dt / left).exp();
                    vessel.temperature = Kelvin(to.max(0.0));
                    vessel.refresh_pressure();
                }
            }
        } else {
            // No transition in the way: the lumped capacitance has a
            // closed-form answer and a sub-stepped one would only be a
            // worse version of it.
            let capacity = ambient_thermal_mass(vessel);
            if capacity > 0.0 {
                let to = room + (from.0 - room) * (-conductance * seconds / capacity).exp();
                vessel.temperature = Kelvin(to.max(0.0));
                vessel.refresh_pressure();
            }
        }

        coalesce_substep_events(scratch, events);
        let to = vessel.temperature;
        if (to.0 - from.0).abs() > TEMPERATURE_EVENT_FLOOR_K {
            events.push(Event::TemperatureChanged {
                vessel: vessel.id,
                from,
                to,
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
                "gas-mechanisms",
                "fermentation",
                "enzymes",
                // EXP-57. Ahead of the room, because the gas it moves is
                // a composition change and `ambient` spends the heat
                // capacity that change alters.
                "gas-exchange",
                "ambient"
            ]
        );
    }

    /// The conductance is arithmetic on two catalogue dimensions, so it
    /// is checkable with a ruler: pi*0.070*0.095 side plus pi*0.070^2/4
    /// base is 0.02474 m^2, and 7.0 W/(m^2 K) of that is 0.1732 W/K.
    #[test]
    fn a_beakers_conductance_is_arithmetic_on_its_geometry() {
        let area = exchange_area_m2("beaker");
        assert!(
            (area - 0.024_740).abs() < 1e-5,
            "beaker wall area {area} m^2"
        );
        let ua = ambient_conductance_w_per_k("beaker");
        assert!((ua - 0.173_18).abs() < 1e-4, "beaker hA {ua} W/K");
        // An unknown label is a beaker, exactly as the light path is.
        assert_eq!(
            ambient_conductance_w_per_k("v1"),
            ambient_conductance_w_per_k("beaker")
        );
        // A test tube is a much smaller thing and loses much less.
        assert!(ambient_conductance_w_per_k("tube") < ua / 3.0);
    }

    /// A vessel already at room temperature has nothing to exchange, and
    /// half a minute of it is still nothing. The floor is the same 0.01 K
    /// the reconciliation pass in `bench.rs` uses.
    #[test]
    fn a_room_temperature_beaker_says_nothing_in_thirty_seconds() {
        let mut v = Vessel::new(crate::vessel::VesselId(0), "beaker");
        let mut events = Vec::new();
        advance(&mut v, 30.0, ClockContext::default(), &mut events).expect("advances");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::TemperatureChanged { .. })),
            "{events:?}"
        );
        assert!((v.temperature.0 - ROOM_TEMPERATURE.0).abs() < 1e-12);
    }

    /// The glass is the only thermal mass an empty beaker has: 100 g of
    /// borosilicate at 0.83 J/(g K) is 83 J/K, and 83/0.1732 is a 479 s
    /// time constant, so an hour is seven and a half of them and almost
    /// nothing of a 2471 K excess survives it.
    #[test]
    fn an_empty_beaker_left_at_flame_temperature_cools_on_the_glass_alone() {
        let mut v = Vessel::new(crate::vessel::VesselId(0), "beaker");
        v.temperature = Kelvin(2769.45);
        let mut events = Vec::new();
        advance(&mut v, 3600.0, ClockContext::default(), &mut events).expect("advances");
        let tau = wall_heat_capacity_j_per_k("beaker") / ambient_conductance_w_per_k("beaker");
        let expected = ROOM_TEMPERATURE.0 + (2769.45 - ROOM_TEMPERATURE.0) * (-3600.0 / tau).exp();
        assert!(
            (v.temperature.0 - expected).abs() < 0.5,
            "cooled to {} K, Newton says {expected} K (tau {tau} s)",
            v.temperature.0
        );
        assert!(v.temperature.0 < 373.15, "still {} K", v.temperature.0);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::TemperatureChanged { .. }))
                .count(),
            1,
            "one announcement per vessel per wait: {events:?}"
        );
    }

    /// A bath is the surroundings; a beaker in a bath is not also a beaker
    /// in a room.
    #[test]
    fn a_thermostatted_vessel_does_not_drift() {
        let mut v = Vessel::new(crate::vessel::VesselId(0), "beaker");
        v.thermal_mode = ThermalMode::Thermostatted(Kelvin(350.0));
        v.temperature = Kelvin(350.0);
        let mut events = Vec::new();
        advance(&mut v, 3600.0, ClockContext::default(), &mut events).expect("advances");
        assert_eq!(v.temperature.0, 350.0);
    }

    #[test]
    fn a_negative_interval_is_no_time_at_all() {
        let mut v = Vessel::new(crate::vessel::VesselId(0), "v1");
        let mut events = Vec::new();
        advance(&mut v, -5.0, ClockContext::default(), &mut events).expect("advances");
        assert!(events.is_empty());
        assert_eq!(v.elapsed_seconds, 0.0);
    }

    /// 200 mL of water in the default beaker, as a bench pours it.
    fn open_beaker_of_water() -> Vessel {
        let mut v = Vessel::new(crate::vessel::VesselId(0), "beaker");
        v.temperature = Kelvin(298.15);
        // 200 g of water is 11.1 mol. The registry key is `water`, not
        // `H2O`: `liquid_volume` resolves the portion through
        // `species::lookup`, and a name it cannot resolve reads as no
        // liquid at all rather than as an error.
        v.deposit(
            SpeciesId::new("water"),
            Moles(200.0 / 18.015),
            Phase::Liquid,
        );
        v
    }

    /// The mouth is not the wall, and gas uses the face heat does not.
    #[test]
    fn the_gas_surface_is_the_mouth_and_not_the_wall() {
        let mouth = free_surface_area_m2("beaker");
        assert!((mouth - std::f64::consts::PI * 0.070 * 0.070 / 4.0).abs() < 1e-9);
        // The wall the room takes heat through is the side plus the base,
        // which is a much larger face than the opening on top.
        assert!(mouth < exchange_area_m2("beaker") / 4.0);
    }

    /// EXP-57, the defect this clock exists for: a beaker of plain water
    /// holds no carbon at all, and the aqueous reservoir was therefore
    /// never offered to it. Air does not ask whether carbon is already
    /// present, so neither does this.
    #[test]
    fn an_open_beaker_of_plain_water_takes_up_room_carbon() {
        let mut v = open_beaker_of_water();
        // No carbon: the solver measured a partial pressure of zero, which
        // is the largest driving force there is.
        v.co2_partial_pressure_atm = Some(0.0);
        let mut events = Vec::new();
        advance(&mut v, 3600.0, ClockContext::default(), &mut events).expect("advances");
        assert!(
            v.pending_co2_transfer_mol > 0.0,
            "an open beaker must take up CO2 from the room, got {}",
            v.pending_co2_transfer_mol
        );
    }

    /// The rate is the published one, arithmetic and all. Cole & Caraco's
    /// still-air intercept over a beaker's mouth gives a time constant of
    /// about two and a half hours for 200 mL — the number that decides
    /// whether a glass left over a lunch break has visibly moved.
    #[test]
    fn the_uptake_is_the_published_transfer_velocity() {
        let mut v = open_beaker_of_water();
        v.co2_partial_pressure_atm = Some(0.0);
        let mut events = Vec::new();
        advance(&mut v, 3600.0, ClockContext::default(), &mut events).expect("advances");

        let area = free_surface_area_m2("beaker");
        let k_h = crate::properties::henry_at_t(
            crate::properties::henry_lookup("CO2").expect("CO2 is tabulated"),
            298.15,
        )
        .value;
        let expected =
            STILL_SURFACE_K_L_M_PER_S * area * k_h * ATMOSPHERIC_CO2_ATM * 1000.0 * 3600.0;
        assert!(
            (v.pending_co2_transfer_mol - expected).abs() < expected * 1e-9,
            "expected {expected}, got {}",
            v.pending_co2_transfer_mol
        );

        // And the time constant that follows from it, stated so a reader
        // can check the claim in the doc comment above.
        let to_equilibrium = k_h * ATMOSPHERIC_CO2_ATM * 0.2;
        let tau = to_equilibrium / (expected / 3600.0);
        assert!(
            (7200.0..11000.0).contains(&tau),
            "the beaker's time constant should be hours, got {tau} s"
        );
    }

    /// Soda water is far above the room's partial pressure, and this clock
    /// deliberately does NOT touch it. Outward transport still belongs to
    /// the aqueous adapter, which degasses an open vessel to the
    /// atmospheric value within the step; a second engine moving the same
    /// carbon would move it twice. The boundary is asserted so that
    /// whoever makes degassing a rate has to come here and say so.
    #[test]
    fn a_supersaturated_vessel_is_left_to_the_aqueous_adapter() {
        let mut v = open_beaker_of_water();
        v.co2_partial_pressure_atm = Some(1.0);
        let mut events = Vec::new();
        advance(&mut v, 86_400.0, ClockContext::default(), &mut events).expect("advances");
        assert_eq!(
            v.pending_co2_transfer_mol, 0.0,
            "outgassing is not this clock's to move"
        );
    }

    /// A long enough wait arrives, and does not sail past and come back.
    #[test]
    fn the_transfer_never_overshoots_equilibrium() {
        let mut v = open_beaker_of_water();
        v.co2_partial_pressure_atm = Some(0.0);
        // The clock works from the vessel's OWN volume, through the
        // registry's density for water; 200 g is near enough 0.2 L but not
        // exactly, and a ceiling checked against a rounder number than the
        // one the code used is a test that fails for arithmetic reasons.
        let volume_l = v.liquid_volume().0;
        let mut events = Vec::new();
        advance(
            &mut v,
            86_400.0 * 30.0,
            ClockContext::default(),
            &mut events,
        )
        .expect("advances");
        let k_h = crate::properties::henry_at_t(
            crate::properties::henry_lookup("CO2").expect("CO2 is tabulated"),
            298.15,
        )
        .value;
        let to_equilibrium = k_h * ATMOSPHERIC_CO2_ATM * volume_l;
        assert!(
            v.pending_co2_transfer_mol <= to_equilibrium * (1.0 + 1e-9),
            "a month of waiting must not overshoot: {} vs {to_equilibrium}",
            v.pending_co2_transfer_mol
        );
    }

    /// A sealed vessel has a headspace the aqueous adapter already owns.
    /// Two engines moving the same CO2 would move it twice.
    #[test]
    fn a_sealed_vessel_exchanges_nothing_with_the_room() {
        let mut v = open_beaker_of_water();
        v.co2_partial_pressure_atm = Some(0.0);
        v.headspace = crate::vessel::Headspace::Sealed {
            volume: crate::units::Liters(0.05),
        };
        let mut events = Vec::new();
        advance(&mut v, 86_400.0, ClockContext::default(), &mut events).expect("advances");
        assert_eq!(v.pending_co2_transfer_mol, 0.0);
    }

    /// The defect that would have left the hydroxide half of EXP-57
    /// cosmetic. A solution that CONSUMES carbon dioxide takes up far more
    /// than Henry's law alone permits, and a ceiling set at the Henry
    /// figure let a beaker of alkali absorb one 7500th of what it should —
    /// which over any wait a bench would actually type is indistinguishable
    /// from inert.
    #[test]
    fn an_alkaline_vessel_may_take_up_more_than_henry_allows() {
        let mut v = open_beaker_of_water();
        v.co2_partial_pressure_atm = Some(0.0);
        // What the aqueous solver would have measured for a bench-strength
        // hydroxide: 0.1 mol/L in 200 mL.
        v.free_hydroxide = 0.02;
        let mut events = Vec::new();
        advance(
            &mut v,
            86_400.0 * 365.0,
            ClockContext::default(),
            &mut events,
        )
        .expect("advances");

        let k_h = crate::properties::henry_at_t(
            crate::properties::henry_lookup("CO2").expect("CO2 is tabulated"),
            298.15,
        )
        .value;
        let henry_only = k_h * ATMOSPHERIC_CO2_ATM * v.liquid_volume().0;
        assert!(
            v.pending_co2_transfer_mol > henry_only * 100.0,
            "the hydroxide sink must lift the ceiling: took {} against a \
             Henry capacity of {henry_only}",
            v.pending_co2_transfer_mol
        );
        // Still bounded by the alkali that is actually there.
        assert!(
            v.pending_co2_transfer_mol <= 0.02 + henry_only + 1e-12,
            "and not more than the hydroxide can consume, got {}",
            v.pending_co2_transfer_mol
        );
    }

    /// A vessel no solver has looked at has no measured pressure, and a
    /// missing measurement is not a zero one — reading it as zero would
    /// carbonate every unsolved beaker on the bench at full tilt.
    #[test]
    fn an_unsolved_vessel_exchanges_nothing() {
        let mut v = open_beaker_of_water();
        assert_eq!(v.co2_partial_pressure_atm, None);
        let mut events = Vec::new();
        advance(&mut v, 86_400.0, ClockContext::default(), &mut events).expect("advances");
        assert_eq!(v.pending_co2_transfer_mol, 0.0);
    }
}

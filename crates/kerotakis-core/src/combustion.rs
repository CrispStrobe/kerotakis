//! KID-12: burning the solids a kitchen actually has, and letting a
//! flame run out of air.
//!
//! Combustion on this bench was, until now, whatever NASA CEA could
//! reach: hand it a hot charge of species the thermochemical data names,
//! and Gibbs minimisation finds the flame. That covers hydrogen, methane,
//! ethanol, magnesium — and it covers none of the three things a child
//! actually sets fire to. `thermo.inp` has no long-chain solid paraffin
//! (its C-numbers stop at naphthalene and n-decyl), no cellulose and no
//! sucrose, and `charge()` declines the WHOLE vessel the moment one
//! species is outside the dataset. So a candle, a sheet of paper and a
//! spoonful of sugar all reached `NotYetModeled` — the honest answer,
//! and a very thin one for the three commonest fires in a house.
//!
//! This module is the curated complement, and it is deliberately small:
//! a table of complete-combustion stoichiometries with a measured heat of
//! combustion each, an autoignition temperature each, and one piece of
//! real physics on top — **a flame needs a minimum oxygen fraction, not
//! merely some oxygen.** That last part is the whole reason the module
//! earns its place, because it is what the classic demonstrations are
//! about:
//!
//! * A candle under a jar goes out **with most of the oxygen still
//!   there** (about 16% of the jar, not 0%). Every child is told the
//!   flame "used up the oxygen"; it did not, and this model says so.
//! * Carbon dioxide poured over a flame puts it out **by dilution**,
//!   without touching the fuel and without reacting with anything. That
//!   is a fire extinguisher, and here it is one line of arithmetic
//!   rather than a special case.
//! * A nitrogen-swept vessel will not light at all.
//!
//! ## What this claims and what it does not
//!
//! It claims: complete combustion to carbon dioxide and water, the heat
//! that releases, the oxygen it consumes, and whether the surrounding gas
//! can still support a flame. It does **not** claim flame temperature
//! (CEA owns that where it has the data), soot, smoke, char, carbon
//! monoxide from incomplete burning, flame spread, a burn *rate*, or
//! anything about a wick. One `ignite` burns what the air allows, all at
//! once; the clock is not modelled. Where NASA data exists, CEA runs
//! first and this solver never sees the vessel — the table below is only
//! for fuels the dataset does not carry.
//!
//! ## Diesel arrived the same way (th-048)
//!
//! The table stopped being three kitchen solids when the corpus asked why
//! diesel and petrol need different ignition conditions. Petrol's
//! surrogate is hexane, which `thermo.inp` HAS, so petrol burns through
//! CEA and never reaches this file. Diesel's surrogate is a C12/C16
//! n-alkane pair, and `thermo.inp`'s saturated chains stop at n-octane
//! and the n-decyl radical — so diesel lands here, exactly as the candle
//! and the paper did, and for exactly the same reason.
//!
//! That routing split is not an accident of data coverage; it is the
//! answer. `burnable` fires the moment a vessel is above a fuel's
//! autoignition temperature with a boundary to burn against, and the
//! diesel surrogates' 476 K and 478 K are below the petrol surrogate's
//! 498 K. Warm a sealed flask of each to 490 K without a spark and the
//! diesel goes; the petrol is answered by `Event::BelowAutoignition` and
//! sits there. That is compression ignition, computed rather than
//! narrated. What this file does NOT carry is the flash point, which is
//! the other half of the real difference and the half that runs the other
//! way — see the `fuel/diesel` recipe's own lot assumptions.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::Phase;
use crate::units::{Kelvin, Moles};
use crate::vessel::{Headspace, Provenance, Vessel};
use crate::SpeciesId;

/// The oxygen mole fraction below which an ordinary diffusion flame in
/// air can no longer sustain itself.
///
/// The number every classroom repeats — "the candle burned all the
/// oxygen" — is wrong, and wrong in a way that is easy to show: a candle
/// in a sealed jar goes out while roughly four fifths of the oxygen is
/// still in there. The limiting oxygen concentration for a hydrocarbon
/// diffusion flame in nitrogen sits near 16 vol%, well above zero,
/// because the flame needs a reaction rate that beats its own heat loss.
///
/// Editorial judgement (Kerotakis): 0.16 is a single teaching value
/// standing in for a quantity that genuinely depends on fuel, geometry,
/// wick size, orientation and diluent. Carbon dioxide is a better
/// smotherer than nitrogen at equal dilution because it carries more
/// heat away per mole, and this constant does not distinguish them.
pub const LIMITING_OXYGEN_FRACTION: f64 = 0.16;

/// One curated fuel, burned completely.
#[derive(Debug, Clone, Copy)]
pub struct Fuel {
    /// Registry species key.
    pub species: &'static str,
    /// Moles of O₂ consumed per mole of fuel.
    pub oxygen: f64,
    /// Moles of CO₂ produced per mole of fuel.
    pub carbon_dioxide: f64,
    /// Moles of H₂O produced per mole of fuel.
    pub water: f64,
    /// Heat released per mole of fuel burned, J/mol, as a positive
    /// number. (−ΔH°c, so the sign convention is "this much comes out".)
    pub heat_j_per_mol: f64,
    /// The temperature at which this fuel catches, K.
    pub autoignition_k: f64,
    /// Balance-checked by `every_curated_fuel_balances`.
    pub equation: &'static str,
    pub provenance: &'static str,
}

/// The curated fuels, each one a substance NASA CEA cannot name.
///
/// The first three are deliberate: every entry has to justify itself
/// against the alternative of saying nothing, and each of them is
/// something a kitchen holds and a child burns — the candle, the paper,
/// the sugar. The last two are the diesel surrogate, and they are here
/// for the same structural reason rather than a kitchen one: `thermo.inp`
/// has no n-alkane above C10, so a C12/C16 diesel cut cannot reach the
/// equilibrium solver at all.
///
/// The first three are solids and the last two are liquids, which is why
/// the burn withdraws its fuel from whatever phase holds it rather than
/// from `Phase::Solid`.
pub const FUELS: &[Fuel] = &[
    Fuel {
        species: "paraffin",
        // C25H52 + 38 O2 -> 25 CO2 + 26 H2O
        oxygen: 38.0,
        carbon_dioxide: 25.0,
        water: 26.0,
        // 46.0 MJ/kg x 0.352691 kg/mol. Paraffin wax is quoted at
        // 43-47 MJ/kg depending on blend; the mid value is used and the
        // spread is smaller than the difference between one candle and
        // another.
        heat_j_per_mol: 16_224_000.0,
        // Wax does not burn as a solid: the wick draws liquid wax up,
        // the heat vaporises it, and the VAPOUR burns. 473 K is the
        // autoignition temperature of paraffin vapour in air. The bench
        // has no wick and no melt pool, so this threshold stands in for
        // the whole of that mechanism.
        autoignition_k: 473.0,
        equation: "C25H52 + 38 O2 -> 25 CO2 + 26 H2O",
        provenance: "Complete combustion of the registry's representative long-chain alkane. Heat of combustion from the standard 46 MJ/kg figure for paraffin wax scaled by the C25H52 molar mass; autoignition near 473 K is the vapour value quoted for paraffinic hydrocarbons. Editorial judgement (Kerotakis): a real candle burns its vapour at a wick, in a melt pool, with a luminous soot-emitting flame, and none of the three is modelled here — this is the heat and the products, not the flame",
    },
    Fuel {
        species: "cellulose",
        // C6H10O5 + 6 O2 -> 6 CO2 + 5 H2O, per anhydroglucose unit.
        oxygen: 6.0,
        carbon_dioxide: 6.0,
        water: 5.0,
        // 17.3 MJ/kg x 0.16214 kg/mol, the standard value for cellulose
        // and the reason paper carries roughly a third of the energy of
        // the same mass of wax.
        heat_j_per_mol: 2_810_000.0,
        // 506 K is the auto-ignition temperature of paper, and it is
        // 451 degrees Fahrenheit. The novel's title is a real number.
        autoignition_k: 506.0,
        equation: "C6H10O5 + 6 O2 -> 6 CO2 + 5 H2O",
        provenance: "Complete combustion of cellulose per anhydroglucose unit, the repeat unit the registry installs. Heat of combustion 17.3 MJ/kg is the standard cellulose value; autoignition near 506 K is the long-quoted figure for paper. Editorial judgement (Kerotakis): real burning paper chars first and leaves ash, and pyrolysis to volatiles is the actual mechanism — the model goes straight to carbon dioxide and water, so it releases the right heat and leaves nothing behind, which a real sheet never does",
    },
    Fuel {
        species: "sucrose",
        // C12H22O11 + 12 O2 -> 12 CO2 + 11 H2O
        oxygen: 12.0,
        carbon_dioxide: 12.0,
        water: 11.0,
        // 16.5 MJ/kg x 0.342296 kg/mol, and the same number a
        // nutrition label reports as 3.94 kcal/g.
        heat_j_per_mol: 5_640_000.0,
        // Sugar caramelises far below this and burns above it; the
        // brown stage in between is chemistry this model does not have.
        autoignition_k: 683.0,
        equation: "C12H22O11 + 12 O2 -> 12 CO2 + 11 H2O",
        provenance: "Complete combustion of sucrose, whose heat of combustion is the same number nutrition labels carry as 3.94 kcal per gram. Editorial judgement (Kerotakis): between melting and burning, sugar caramelises and then chars through a family of reactions this table does not contain, so the model offers only the two ends — unchanged below 683 K, fully burned above it with enough air. The black snake is not in here",
    },
    Fuel {
        species: "dodecane",
        // C12H26 + 18.5 O2 -> 12 CO2 + 13 H2O
        oxygen: 18.5,
        carbon_dioxide: 12.0,
        water: 13.0,
        // -ΔH°c from the commonly tabulated standard formation
        // enthalpies: 12 CO2(g) at -393.51 and 13 H2O(l) at -285.83
        // against C12H26(l) at -350.9 kJ/mol gives 8087 kJ/mol, which is
        // 47.5 MJ/kg — the figure a fuel table quotes for diesel.
        heat_j_per_mol: 8_087_000.0,
        autoignition_k: 476.0,
        equation: "C12H26 + 18.5 O2 -> 12 CO2 + 13 H2O",
        provenance: "Complete combustion of n-dodecane, the light half of the registry's diesel surrogate. Heat of combustion is the difference of the commonly tabulated standard formation enthalpies (12 CO2(g) −393.51, 13 H2O(l) −285.83, C12H26(l) −350.9 kJ/mol → 8087 kJ/mol, 47.5 MJ/kg), so the arithmetic is checkable rather than quoted. Autoignition near 476 K is the commonly tabulated value for n-dodecane, and commercial diesel is tabulated at about 483 K — pending-review lane: the primary sources (Zabetakis, U.S. Bureau of Mines Bulletin 627, 1965; CRC) were not re-read for this row and no page is cited. Editorial judgement (Kerotakis): the products are carbon dioxide and liquid-basis water with no soot, and a real diesel flame is a sooting spray flame whose whole character is the soot",
    },
    Fuel {
        species: "hexadecane",
        // C16H34 + 24.5 O2 -> 16 CO2 + 17 H2O
        oxygen: 24.5,
        carbon_dioxide: 16.0,
        water: 17.0,
        // 16 CO2(g) at -393.51 and 17 H2O(l) at -285.83 against
        // C16H34(l) at -456.1 kJ/mol: 10699 kJ/mol, or 47.2 MJ/kg.
        heat_j_per_mol: 10_699_000.0,
        autoignition_k: 478.0,
        equation: "C16H34 + 24.5 O2 -> 16 CO2 + 17 H2O",
        provenance: "Complete combustion of n-hexadecane — cetane, the fuel the cetane scale is defined against. Heat of combustion is the difference of the commonly tabulated standard formation enthalpies (16 CO2(g) −393.51, 17 H2O(l) −285.83, C16H34(l) −456.1 kJ/mol → 10699 kJ/mol, 47.2 MJ/kg). Autoignition near 478 K is the commonly tabulated value — pending-review lane: no primary page is cited. Editorial judgement (Kerotakis): this row is what makes th-048's answer computed rather than asserted, because 478 K sits below hexane's 498 K in GAS_AUTOIGNITION and the two tables are compared by a test rather than by prose",
    },
];

/// The fuel table entry for a species key, if it has one.
pub fn fuel_of(species: &str) -> Option<&'static Fuel> {
    FUELS.iter().find(|fuel| fuel.species == species)
}

/// BRD-041 routing: the autoignition temperature of a fuel the registry
/// names, in air near one atmosphere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasAutoignition {
    pub species: &'static str,
    pub autoignition_k: f64,
    pub provenance: &'static str,
}

const ZABETAKIS: &str =
    "Autoignition temperature in air, as commonly tabulated from M. G. Zabetakis, \
    Flammability Characteristics of Combustible Gases and Vapors, U.S. Bureau of Mines Bulletin \
    627 (1965). Pending-review lane: the bulletin was not re-read for this row, so the value is \
    the standard tabulated one and the page is not cited";

/// The fuels the thermal solver must not burn without a spark below these
/// temperatures. Equilibrium would burn methane and air the moment they
/// were warm; a real mixture sits there until a spark or its autoignition
/// temperature, and this table is where the bench says so.
pub const GAS_AUTOIGNITION: &[GasAutoignition] = &[
    GasAutoignition {
        species: "H2",
        autoignition_k: 773.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "methane",
        autoignition_k: 810.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "propane",
        autoignition_k: 743.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "butane",
        autoignition_k: 678.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "CO",
        autoignition_k: 882.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "ethanol",
        autoignition_k: 636.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "methanol",
        autoignition_k: 658.15,
        provenance: ZABETAKIS,
    },
    GasAutoignition {
        species: "propanone",
        autoignition_k: 738.15,
        provenance: ZABETAKIS,
    },
    // Hexane is also the registry's petrol surrogate (`fuel/petrol`), so
    // this row is the number th-048 compares the diesel surrogate against.
    // No `diesel` row is added beside it, and the absence is deliberate:
    // `unsparked_fuels` is read only by `kerotakis_cea`'s equilibrator,
    // which declines any vessel holding a species NASA CEA cannot name —
    // and it cannot name a C12 or C16 alkane. A diesel row here would
    // never be reached. The live diesel autoignition temperatures are in
    // `FUELS` above, where `burnable` reads them, and
    // `the_diesel_surrogate_self_ignites_below_the_petrol_surrogate` pins
    // the comparison across the two tables.
    GasAutoignition {
        species: "hexane",
        autoignition_k: 498.15,
        provenance: ZABETAKIS,
    },
];

pub fn gas_autoignition(species: &str) -> Option<&'static GasAutoignition> {
    GAS_AUTOIGNITION.iter().find(|row| row.species == species)
}

/// Fuels standing in this vessel with oxygen to hand — an O₂ portion, or
/// an open headspace on the room's air — while the vessel is below their
/// autoignition temperature. Each is returned with that temperature. A
/// spark (`ignite`) takes the vessel to 1200 K, above every row here, so
/// a sparked mixture is never on this list.
pub fn unsparked_fuels(vessel: &Vessel) -> Vec<(SpeciesId, f64)> {
    let oxygen = vessel.moles_of(&SpeciesId::new("O2")).0 > crate::OBSERVABLE_MOLES
        || matches!(vessel.headspace, crate::vessel::Headspace::Open);
    if !oxygen {
        return Vec::new();
    }
    GAS_AUTOIGNITION
        .iter()
        .filter(|row| vessel.temperature.0 < row.autoignition_k)
        .filter(|row| vessel.moles_of(&SpeciesId::new(row.species)).0 > crate::OBSERVABLE_MOLES)
        .map(|row| (SpeciesId::new(row.species), row.autoignition_k))
        .collect()
}

/// What the vessel's boundary offers a flame.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Air {
    /// Open to the room: the flame draws as much air as it needs. This
    /// is the same assumption CEA's `charge()` makes for an open vessel,
    /// where the atmosphere supplies oxygen up to the stoichiometric
    /// demand — being consistent with it matters more than either
    /// choice.
    Room,
    /// A closed boundary owns its gas, so the flame has exactly this
    /// much oxygen inside this much gas — and it goes out when the
    /// fraction falls too far, not when the oxygen runs out.
    Owned { oxygen: f64, total: f64 },
    /// A nitrogen purge. There is no oxygen and none is coming.
    Inert,
}

fn air(vessel: &Vessel) -> Air {
    match vessel.headspace {
        Headspace::Open => Air::Room,
        Headspace::Swept { .. } => Air::Inert,
        Headspace::Sealed { .. } | Headspace::PressureControlled { .. } => Air::Owned {
            oxygen: vessel
                .contents
                .iter()
                .filter(|p| p.phase == Phase::Gas && p.species.0 == "O2")
                .map(|p| p.moles.0)
                .sum(),
            total: vessel.gas_moles().0,
        },
    }
}

/// How much fuel a closed vessel's gas can burn before the oxygen
/// fraction reaches the limit.
///
/// Burning ξ moles of fuel takes `a·ξ` oxygen out of the gas and puts
/// `(c + w)·ξ` back in, so both the oxygen and the total move. Solving
///
/// ```text
///   (O₂ − aξ) / (T + (c + w − a)ξ) = f
/// ```
///
/// for ξ gives the extent at which the flame reaches the limiting
/// fraction. A vessel that is ALREADY below the limit returns zero or
/// less: that is the carbon-dioxide extinguisher, and it needs no
/// special case.
fn oxygen_limited_extent(fuel: &Fuel, oxygen: f64, total: f64) -> f64 {
    let f = LIMITING_OXYGEN_FRACTION;
    let swell = fuel.carbon_dioxide + fuel.water - fuel.oxygen;
    let denominator = fuel.oxygen + f * swell;
    if denominator <= 0.0 {
        // No stoichiometry in the table has this shape, but a future one
        // could: rather than divide by something non-positive, fall back
        // to the oxygen inventory alone.
        return (oxygen / fuel.oxygen).max(0.0);
    }
    ((oxygen - f * total) / denominator).max(0.0)
}

/// Burning as a solver: it runs where a curated fuel is above its
/// autoignition temperature and there is a boundary to burn against.
#[derive(Debug, Default, Clone, Copy)]
pub struct CombustionEquilibrator;

/// Whether this vessel holds liquid water. Combustion declines those to
/// the aqueous engine, exactly as `cea-thermal` does: a vessel with a
/// solution in it is not a fire, and two solvers must not both own it.
fn has_liquid_water(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
}

fn burnable(vessel: &Vessel) -> Vec<(&'static Fuel, f64)> {
    if has_liquid_water(vessel) {
        return Vec::new();
    }
    FUELS
        .iter()
        .filter(|fuel| vessel.temperature.0 >= fuel.autoignition_k)
        .filter_map(|fuel| {
            let moles = vessel.moles_of(&SpeciesId::new(fuel.species)).0;
            (moles > crate::OBSERVABLE_MOLES).then_some((fuel, moles))
        })
        .collect()
}

impl Equilibrator for CombustionEquilibrator {
    fn name(&self) -> &'static str {
        "curated-combustion"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        !burnable(vessel).is_empty()
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let fuels = burnable(vessel);
        if fuels.is_empty() {
            return Ok(Vec::new());
        }
        let air_at_start = air(vessel);
        let mut air = air_at_start;
        let mut events = Vec::new();
        let mut released_j = 0.0;
        let mut starved: Option<(SpeciesId, f64, f64)> = None;

        for (fuel, present) in fuels {
            let extent = match air {
                Air::Room => present,
                Air::Inert => 0.0,
                Air::Owned { oxygen, total } => {
                    oxygen_limited_extent(fuel, oxygen, total).min(present)
                }
            };
            if extent <= crate::OBSERVABLE_MOLES {
                // Nothing of this fuel burned, and the reason is the air.
                // Report the fraction the flame actually met.
                let fraction = match air {
                    Air::Room => 1.0,
                    Air::Inert => 0.0,
                    Air::Owned { oxygen, total } => {
                        if total > 0.0 {
                            oxygen / total
                        } else {
                            0.0
                        }
                    }
                };
                starved.get_or_insert((SpeciesId::new(fuel.species), 0.0, fraction));
                continue;
            }

            let id = SpeciesId::new(fuel.species);
            let oxygen_used = extent * fuel.oxygen;
            let carbon_dioxide = extent * fuel.carbon_dioxide;
            let steam = extent * fuel.water;

            // Whatever phase holds it: the three kitchen fuels are solids,
            // the two diesel surrogates are liquids, and a fuel that
            // burned has left the vessel either way. Taking it from
            // `Phase::Solid` alone would have burned the diesel without
            // consuming it, and the ledger would have carried it forever.
            vessel.withdraw(&id, Moles(extent));
            let remaining = vessel.moles_of(&id);
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: fuel.equation.to_string(),
            });
            events.push(Event::Consumed {
                vessel: vessel.id,
                species: id.clone(),
                moles: Moles(extent),
                remaining: Some(remaining),
            });

            match air {
                Air::Room | Air::Inert => {
                    // An open vessel exchanges with the room: the oxygen
                    // came from outside the ledger and the products leave
                    // it again. Nothing about the room is tracked, which
                    // is why a candle in the open never runs out of air.
                    events.push(Event::GasEvolved {
                        vessel: vessel.id,
                        species: SpeciesId::new("CO2"),
                        moles: Moles(carbon_dioxide),
                    });
                    events.push(Event::GasEvolved {
                        vessel: vessel.id,
                        species: SpeciesId::new("water"),
                        moles: Moles(steam),
                    });
                }
                Air::Owned { oxygen, total } => {
                    let taken = vessel.withdraw_phase(
                        &SpeciesId::new("O2"),
                        Moles(oxygen_used),
                        Phase::Gas,
                    );
                    events.push(Event::Consumed {
                        vessel: vessel.id,
                        species: SpeciesId::new("O2"),
                        moles: taken,
                        remaining: Some(vessel.moles_of(&SpeciesId::new("O2"))),
                    });
                    vessel.deposit(SpeciesId::new("CO2"), Moles(carbon_dioxide), Phase::Gas);
                    vessel.deposit(SpeciesId::new("water"), Moles(steam), Phase::Gas);
                    events.push(Event::GasContained {
                        vessel: vessel.id,
                        species: SpeciesId::new("CO2"),
                        moles: Moles(carbon_dioxide),
                    });
                    events.push(Event::GasContained {
                        vessel: vessel.id,
                        species: SpeciesId::new("water"),
                        moles: Moles(steam),
                    });
                    let oxygen_left = (oxygen - taken.0).max(0.0);
                    let total_left = total - taken.0 + carbon_dioxide + steam;
                    air = Air::Owned {
                        oxygen: oxygen_left,
                        total: total_left,
                    };
                    if extent < present {
                        // The flame stopped while fuel was still there:
                        // the jar, not the candle, ended this.
                        let fraction = if total_left > 0.0 {
                            oxygen_left / total_left
                        } else {
                            0.0
                        };
                        starved.get_or_insert((id.clone(), extent, fraction));
                    }
                }
            }

            released_j += extent * fuel.heat_j_per_mol;
        }

        if let Some((fuel, burned, oxygen_fraction)) = starved {
            events.push(Event::FlameStarved {
                vessel: vessel.id,
                fuel,
                burned: Moles(burned),
                oxygen_fraction,
            });
        }

        if released_j <= 0.0 {
            return Ok(events);
        }

        // The heat, and what the vessel does with it.
        //
        // In an OPEN vessel the products leave, and they leave hot: the
        // energy goes up with the exhaust, which is why a candle does not
        // heat its own candlestick to the flame temperature. Booking that
        // heat into whatever happens to be left in the beaker produced a
        // 6000 °C beaker in the curiosity corpus — the arithmetic was
        // right and the physics was absent. So an open burn reports its
        // energy and warms nothing; only a closed boundary, which keeps
        // its own hot gas, can be warmed by it.
        let heat_stays = matches!(air_at_start, Air::Owned { .. });
        if heat_stays && matches!(vessel.thermal_mode, crate::vessel::ThermalMode::Adiabatic) {
            let cp = vessel.heat_capacity();
            if cp > 0.0 {
                let from = vessel.temperature;
                let to = Kelvin(from.0 + released_j / cp);
                vessel.temperature = to;
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from,
                    to,
                });
            }
        }
        events.push(Event::ThermalEquilibrium {
            vessel: vessel.id,
            temperature: vessel.temperature,
            reaction_energy_j: Some(released_j),
            holds_nothing: vessel.contents.is_empty(),
            provenance: Provenance {
                engine: "curated combustion (Kerotakis)".to_string(),
                dataset: "kerotakis:combustion:curated-fuels-v1".to_string(),
                model: "complete combustion to CO2 and H2O at a tabulated heat of combustion, with a limiting oxygen fraction of 0.16 for a closed boundary".to_string(),
                dataset_sources: FUELS
                    .iter()
                    .map(|fuel| format!("{}: {}", fuel.species, fuel.equation))
                    .collect(),
                routing: "NASA CEA carries no thermochemistry for this fuel, so the curated table answered instead of the vessel reaching the model boundary".to_string(),
            },
        });
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every equation in the table conserves C, H and O. A heat of
    /// combustion attached to an unbalanced equation would be a number
    /// with nothing behind it.
    #[test]
    fn every_curated_fuel_balances() {
        for fuel in FUELS {
            let s = crate::species::lookup(&SpeciesId::new(fuel.species))
                .unwrap_or_else(|| panic!("{} must be an installed species", fuel.species));
            let formula = s.formula;
            // Read the counts straight off the formula string: the table
            // must agree with the species it names, not with itself.
            let count = |element: &str| -> f64 {
                let mut total = 0.0;
                let bytes: Vec<char> = formula.chars().collect();
                let mut i = 0;
                while i < bytes.len() {
                    let start = i;
                    i += 1;
                    while i < bytes.len() && bytes[i].is_ascii_lowercase() {
                        i += 1;
                    }
                    let symbol: String = bytes[start..i].iter().collect();
                    let digits_start = i;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    let n: f64 = if i > digits_start {
                        bytes[digits_start..i]
                            .iter()
                            .collect::<String>()
                            .parse()
                            .expect("digits")
                    } else {
                        1.0
                    };
                    if symbol == element {
                        total += n;
                    }
                }
                total
            };
            assert_eq!(
                count("C"),
                fuel.carbon_dioxide,
                "carbon in {}",
                fuel.species
            );
            assert_eq!(count("H"), fuel.water * 2.0, "hydrogen in {}", fuel.species);
            assert_eq!(
                count("O") + fuel.oxygen * 2.0,
                fuel.carbon_dioxide * 2.0 + fuel.water,
                "oxygen in {}",
                fuel.species
            );
        }
    }

    /// th-048's answer, pinned across the two tables that hold it.
    ///
    /// "Diesel self-ignites at a lower temperature than petrol" is a
    /// comparison between a `FUELS` row and a `GAS_AUTOIGNITION` row,
    /// because the two fuels route to different solvers — and a claim
    /// that lives in two tables is exactly the kind that drifts apart
    /// silently. Both diesel surrogates must stay below the petrol
    /// surrogate, or the corpus row is answering with the wrong sign.
    #[test]
    fn the_diesel_surrogate_self_ignites_below_the_petrol_surrogate() {
        let petrol = gas_autoignition("hexane")
            .expect("hexane is the registry's petrol surrogate and carries an autoignition row");
        for key in ["dodecane", "hexadecane"] {
            let diesel = fuel_of(key).unwrap_or_else(|| panic!("{key} is a curated fuel"));
            assert!(
                diesel.autoignition_k < petrol.autoignition_k,
                "{key} autoignites at {} K, which is not below the petrol surrogate's {} K",
                diesel.autoignition_k,
                petrol.autoignition_k
            );
        }
    }

    /// A curated fuel must be a species the shelf actually has, in the
    /// phase the burn withdraws from. The three kitchen fuels are solids
    /// and the two diesel surrogates are liquids; nothing here may be a
    /// gas, because a gas fuel with a NASA record would never reach this
    /// solver and one without a record has no thermochemistry at all.
    #[test]
    fn every_curated_fuel_is_a_condensed_registry_species() {
        for fuel in FUELS {
            let s = crate::species::lookup(&SpeciesId::new(fuel.species))
                .unwrap_or_else(|| panic!("{} must be an installed species", fuel.species));
            assert!(
                matches!(s.standard_phase, Phase::Solid | Phase::Liquid),
                "{} is {:?}, not a condensed fuel",
                fuel.species,
                s.standard_phase
            );
        }
    }

    /// The extinguisher, as arithmetic: a jar whose oxygen fraction is
    /// already under the limit burns nothing at all, however much oxygen
    /// it holds in absolute terms.
    #[test]
    fn a_diluted_jar_burns_nothing_even_with_oxygen_in_it() {
        let fuel = fuel_of("paraffin").expect("paraffin is a curated fuel");
        // 1 mol of oxygen is a great deal of oxygen — inside 10 mol of
        // carbon dioxide it is 9%, and no candle burns in that.
        assert_eq!(oxygen_limited_extent(fuel, 1.0, 11.0), 0.0);
        // The same oxygen in ordinary air does burn.
        assert!(oxygen_limited_extent(fuel, 1.0, 5.0) > 0.0);
    }

    /// A sealed jar leaves most of its oxygen behind — the point of the
    /// demonstration, and the opposite of what the phrase "it used up
    /// the oxygen" says.
    #[test]
    fn a_sealed_jar_stops_with_oxygen_to_spare() {
        let fuel = fuel_of("paraffin").expect("paraffin is a curated fuel");
        // A 1 L jar of room air holds about 0.041 mol of gas, of which
        // 0.0086 mol is oxygen.
        let oxygen = 0.0086;
        let total = 0.041;
        let extent = oxygen_limited_extent(fuel, oxygen, total);
        let left = oxygen - extent * fuel.oxygen;
        assert!(left > 0.0, "the flame must not consume the last oxygen");
        assert!(
            left / oxygen > 0.7,
            "most of the oxygen is still there: {left} of {oxygen}"
        );
    }
}

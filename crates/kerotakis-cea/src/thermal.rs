//! The L2g bench solver: heating, decomposing and burning things.
//!
//! Where the aqueous engine owns solutions, this owns the dry, hot regime —
//! solids and gases exchanging with the vessel's atmosphere. It runs when
//! there is no liquid water to speak of and at least one species maps into
//! the NASA data.
//!
//! Two modelling choices, both deliberate and both visible to the user:
//!
//! * **The atmosphere is a reservoir, not inventory.** A vessel stands open
//!   in air: oxygen is available without being weighed in, and product
//!   gases leave. This mirrors the aqueous solver's escaping-gas phases,
//!   and it is what makes the problem well-posed — with no atmosphere,
//!   calcite below its decomposition point has no gas phase at all. Being a
//!   reservoir cuts one way only: the room may be warmed by what happens in
//!   the vessel, and may never pay for it. See
//!   [`crate::gibbs::OpenAtmosphere`].
//! * **Species map by composition, not by a hand-written table.** A solid
//!   `CaCO3` in the registry finds CEA's `CaCO3(cr)` because their formulas
//!   agree; nothing lists the pair.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kerotakis_core::phrase::{Phrase, Slot};
use kerotakis_core::species::{self, Phase};
use kerotakis_core::{
    Equilibrator, Event, Kelvin, Moles, Portion, Provenance, SolveError, SpeciesId, ThermalMode,
    Vessel,
};

use crate::gibbs::{equilibrate_tp, OpenAtmosphere};
use crate::nasa9::{db, Species};

/// Air, as mole fractions of the reservoir the vessel stands in.
const AIR: &[(&str, f64)] = &[("N2", 0.78), ("O2", 0.21)];

/// The balanced burn a liquid fuel announces once it has caught.
///
/// The composition and the energy both come out of the Gibbs solve; this
/// table only supplies the familiar written equation to put beside them,
/// because a reader recognises `2 CH₃OH + 3 O₂ → 2 CO₂ + 4 H₂O` and does
/// not recognise a mole table. A fuel earns a row here once its liquid
/// record actually burns in the solver — the row is a label, never the
/// reason anything happened.
const LIQUID_FUEL_COMBUSTION: &[(&str, &str)] = &[
    ("methanol", "2 CH₃OH(l) + 3 O₂(g) → 2 CO₂(g) + 4 H₂O(g)"),
    ("ethanol", "C₂H₅OH(l) + 3 O₂(g) → 2 CO₂(g) + 3 H₂O(g)"),
];

/// Below this temperature the thermal solver stands down and lets solids
/// be: equilibrium would oxidise every metal on the bench, and only
/// kinetics (L5) explains why the world is not like that.
pub const KINETIC_THRESHOLD_K: f64 = 500.0;

/// How much atmosphere participates, relative to the condensed matter in
/// the vessel. A beaker's headspace is small but never zero; this keeps the
/// oxygen supply generous enough not to be the limiting reagent by accident
/// while staying finite.
///
/// **This is a chemical control volume and nothing else.** It sizes two
/// things: how much oxygen a burn may draw on, and how much nitrogen is
/// there to dilute the gas phase — the second is why a carbonate gives way
/// below its one-bar decomposition temperature, since what the equilibrium
/// answers to is CO₂'s partial pressure and not its amount.
///
/// It is **not** a thermal mass the vessel owns. It used to act as one in
/// the adiabatic solve, which is how eight times the vessel's own moles of
/// air came to pay part of a calcination; [`crate::gibbs::OpenAtmosphere`]
/// is the rule that stops it. Enlarging or shrinking this number therefore
/// changes what the chemistry can reach and what the flame is diluted by,
/// and no longer changes who pays for an endothermic step.
const AIR_RATIO: f64 = 8.0;

/// The temperature the exhaust of an open burn is vented at, K.
///
/// The Gibbs solve answers the question the flame asks, and it answers it
/// correctly: at 2769 K a stoichiometric ethanol equilibrium really is
/// partly dissociated, and about a twelfth of the hydrogen really is loose
/// H₂ rather than water. That is the truth INSIDE the flame.
///
/// It was not the truth about what left the beaker. The products were
/// vented FROZEN at flame temperature — the transcript beside
/// `C₂H₅OH(l) + 3 O₂(g) → 2 CO₂(g) + 3 H₂O(g)` read "0.4677 mol water ↑,
/// 0.3425 mol carbon dioxide ↑, **0.0461 mol hydrogen ↑**", so the equation
/// the lab printed and the moles the lab counted disagreed with each other
/// by 9 % of the fuel's hydrogen. A plume does not stay at 2500 K on its
/// way to the ceiling: it cools within centimetres, the radicals recombine
/// long before anything could be collected, and what a bench catches over a
/// spirit burner is carbon dioxide and water.
///
/// So the gas half of the equilibrium is re-settled at this temperature
/// before it is vented. 1000 K is a flue temperature of the right order —
/// combustion exhaust leaves an open flame somewhere between a few hundred
/// K above ambient and about 1200 K — and, much more usefully, **the answer
/// does not depend on picking it exactly**: anywhere under about 1400 K a
/// C/H/O/N exhaust with the flame's own excess air leaves less than a
/// ten-thousandth of its hydrogen as H₂, so anything in the plausible flue
/// band gives the complete-combustion composition the printed equation
/// claims. `the_exhaust_temperature_hardly_matters` in
/// `tests/vented_products.rs` is that statement as a test rather than as a
/// promise.
///
/// **What this does NOT do** is book the recombination's heat. The energy
/// and the vessel's own temperature both stay the adiabatic flame's,
/// because that is what the flame was; the extra enthalpy released as the
/// exhaust cools leaves with the exhaust, which is exactly where it goes in
/// a room. A closed vessel would need that heat back, and a closed vessel
/// is not what this path models — see the module note on the atmosphere as
/// a reservoir.
const EXHAUST_K: f64 = 1000.0;

/// The composition the product gas has by the time it has left.
///
/// Takes the element budget of the GAS half of the flame equilibrium and
/// re-minimises it at [`EXHAUST_K`] over the gas half of the same pool. It
/// is the same solver, the same data and the same conserved elements — the
/// only thing that changes is the temperature the answer is asked at, so
/// nothing is created, destroyed or renamed on the way out.
///
/// `None` where there is no gas to vent; the caller falls back to the flame
/// composition where the re-solve does not converge, which keeps a
/// convergence failure from losing matter.
fn exhaust_composition(
    flame: &[(String, f64)],
    pool: &[&'static Species],
) -> Option<Vec<(String, f64)>> {
    let mut budget: BTreeMap<String, f64> = BTreeMap::new();
    let mut any_gas = false;
    for (name, moles) in flame {
        let Some(s) = db().get(name) else { continue };
        if !s.is_gas() || *moles <= 0.0 {
            continue;
        }
        any_gas = true;
        for (element, count) in &s.composition {
            *budget.entry(element.clone()).or_insert(0.0) += count * moles;
        }
    }
    if !any_gas {
        return None;
    }
    let gases: Vec<&'static Species> = pool.iter().copied().filter(|s| s.is_gas()).collect();
    equilibrate_tp(&budget, &gases, EXHAUST_K, 1.0)
        .ok()
        .map(|eq| eq.composition)
}

/// The registry row that names a CEA species, or a refusal.
///
/// The pool is built from nameable species, so a miss cannot normally
/// happen — but if it ever does, refuse rather than quietly lose matter.
fn registry_row(name: &str) -> Result<&'static species::SpeciesData, SolveError> {
    species::REGISTRY
        .iter()
        .find(|r| {
            cea_name(r.key)
                .is_some_and(|mapped| cea_identity_stem(mapped) == cea_identity_stem(name))
        })
        .ok_or_else(|| SolveError::NotConverged {
            solver: "cea-thermal".to_string(),
            detail: format!("the equilibrium contains {name}, which has no name in the registry"),
        })
}

/// Registry key → CEA species name, derived by matching chemical formulas.
fn mapping() -> &'static BTreeMap<&'static str, &'static str> {
    static MAP: OnceLock<BTreeMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut map = BTreeMap::new();
        for reg in species::REGISTRY {
            let Some(want) = formula_composition(reg.formula) else {
                continue;
            };
            let want_gas = reg.standard_phase == Phase::Gas;
            // Prefer the phase the registry says the substance is in, and
            // among equals the most stable form (lowest G at 298 K).
            let products = db()
                .species
                .values()
                .filter(|s| s.is_gas() == want_gas && s.composition == want)
                .min_by(|a, b| {
                    let ga = a.g(298.15).unwrap_or(f64::MAX);
                    let gb = b.g(298.15).unwrap_or(f64::MAX);
                    ga.total_cmp(&gb)
                });
            // CEA deliberately separates feed-only thermochemistry after
            // `END PRODUCTS`. The liquid alcohols — CH3OH(L), C2H5OH(L) —
            // live there: such a record may enter an energy balance, but
            // must never be invented as an equilibrium product. Prefer the
            // ordinary product set and consult that separate feed set only
            // when the requested room phase is absent.
            let reactants = db()
                .reactants
                .values()
                .filter(|s| s.is_gas() == want_gas && s.composition == want)
                .min_by(|a, b| {
                    let ga = a.g(298.15).unwrap_or(f64::MAX);
                    let gb = b.g(298.15).unwrap_or(f64::MAX);
                    ga.total_cmp(&gb)
                });
            let best = products.or(reactants);
            if let Some(s) = best {
                map.insert(reg.key, s.name.as_str());
            }
        }
        map
    })
}

/// The CEA species a registry key resolves to, if any.
pub fn cea_name(registry_key: &str) -> Option<&'static str> {
    mapping().get(registry_key).copied()
}

fn cea_species(registry_key: &str) -> Option<&'static Species> {
    let name = cea_name(registry_key)?;
    db().get(name).or_else(|| db().get_reactant(name))
}

fn formation_anchor(species: &Species, temperature: f64) -> bool {
    // Some room-standard records (CaCO3, MgO, ethanol gas) start Cp at 300 K,
    // but their NASA header independently supplies Hf at 298.15 K. Use that
    // exact reference datum, never clamp/extrapolate the polynomial.
    temperature == crate::T_REF
        && !species.name.ends_with("(L)")
        && species
            .t_range()
            .is_some_and(|(lo, _)| lo <= 300.0 && (!species.is_gas() || temperature < lo))
}

fn record_enthalpy(species: &Species, temperature: f64) -> Option<f64> {
    if formation_anchor(species, temperature) {
        Some(species.h_formation)
    } else {
        species.h(temperature)
    }
}

fn enthalpy_within_record(species: &Species, temperature: f64, phase: Phase) -> Option<f64> {
    let valid = |candidate: &Species| {
        candidate
            .t_range()
            .is_some_and(|(lo, hi)| temperature >= lo && temperature <= hi)
            || formation_anchor(candidate, temperature)
    };
    let stem = cea_identity_stem(&species.name);
    let same_family = |candidate: &Species| {
        candidate.composition == species.composition && cea_identity_stem(&candidate.name) == stem
    };
    let original_matches = if phase == Phase::Gas {
        species.is_gas()
    } else {
        !species.is_gas() && species.name.ends_with("(L)") == (phase == Phase::Liquid)
    };
    if original_matches && valid(species) {
        return record_enthalpy(species, temperature);
    }
    if phase == Phase::Gas {
        // Steam and alcohol vapour may be booked under a liquid-standard
        // registry identity: the actual portion phase owns feed enthalpy.
        return db()
            .species
            .values()
            .filter(|s| s.is_gas() && same_family(s) && valid(s))
            .min_by(|a, b| {
                a.g(temperature)
                    .unwrap()
                    .total_cmp(&b.g(temperature).unwrap())
            })
            .and_then(|s| record_enthalpy(s, temperature));
    }
    let condensed: Vec<_> = db()
        .species
        .values()
        .filter(|s| !s.is_gas() && same_family(s))
        .chain(std::iter::once(species).filter(|s| !s.is_gas()))
        .collect();
    let matches_phase = |s: &&Species| s.name.ends_with("(L)") == (phase == Phase::Liquid);
    let select = |same_phase: bool| {
        condensed
            .iter()
            .copied()
            .filter(|s| valid(s) && (!same_phase || matches_phase(s)))
            .min_by(|a, b| {
                a.g(temperature)
                    .unwrap()
                    .total_cmp(&b.g(temperature).unwrap())
            })
    };
    // Honour an in-range actual phase first. If the booked phase has left
    // its NASA range, use an in-range condensed sibling at lowest G. This
    // is a bounded stable-feed assumption, not operator fusion calorimetry;
    // melting/latent heat in a heat operation still belongs to its owner.
    if let Some(s) = select(true).or_else(|| select(false)) {
        return record_enthalpy(s, temperature);
    }
    let ceiling = condensed
        .iter()
        .filter_map(|s| s.t_range().map(|(_, hi)| hi))
        .fold(f64::NEG_INFINITY, f64::max);
    if temperature > ceiling {
        return db()
            .species
            .values()
            .filter(|s| s.is_gas() && same_family(s) && valid(s))
            .min_by(|a, b| {
                a.g(temperature)
                    .unwrap()
                    .total_cmp(&b.g(temperature).unwrap())
            })
            .and_then(|s| record_enthalpy(s, temperature));
    }
    // A gap/below-range state has no verified feed enthalpy. Never clamp a
    // phase polynomial or borrow its vapour's formation/latent energy.
    None
}

fn cea_identity_stem(name: &str) -> &str {
    name.split(['(', ',']).next().unwrap_or(name)
}

/// Element counts of a registry formula, ignoring charge and hydrate
/// notation (hydrates are not in the NASA condensed set).
fn formula_composition(formula: &str) -> Option<BTreeMap<String, f64>> {
    if formula.contains('·') || formula.contains(':') || formula.contains(['+', '-']) {
        return None;
    }
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_uppercase() {
            return None;
        }
        let mut sym = chars[i].to_string();
        i += 1;
        while i < chars.len() && chars[i].is_ascii_lowercase() {
            sym.push(chars[i]);
            i += 1;
        }
        let mut digits = String::new();
        while i < chars.len() && chars[i].is_ascii_digit() {
            digits.push(chars[i]);
            i += 1;
        }
        let n: f64 = if digits.is_empty() {
            1.0
        } else {
            digits.parse().ok()?
        };
        *counts.entry(sym).or_insert(0.0) += n;
    }
    (!counts.is_empty()).then_some(counts)
}

/// The species a mixture of these elements may resolve into: exactly those
/// the registry can name, plus the atmosphere.
///
/// This is the honesty boundary made structural. The NASA data holds every
/// exotic carbide and nitride those elements could form; letting the
/// minimiser reach for one we cannot name would mean either dropping it
/// (losing mass) or showing the user a formula with no story attached.
/// Widening what the lab can discover is therefore a deliberate act:
/// name the substance in the registry, and the solver can find it.
fn pool_for(elements: &[String]) -> Vec<&'static Species> {
    let mut names: Vec<&'static str> = species::REGISTRY
        .iter()
        .filter_map(|r| cea_name(r.key))
        .collect();
    for (air, _) in AIR {
        names.push(air);
    }
    names.sort_unstable();
    names.dedup();
    let mut pool: Vec<&'static Species> = names
        .iter()
        .filter_map(|n| db().get(n))
        .filter(|s| {
            !s.composition.is_empty()
                && s.composition
                    .keys()
                    .all(|el| elements.iter().any(|e| e == el))
        })
        .collect();
    // Each registry key maps to one CEA record — the standard phase at
    // room temperature. The substance's other phases must be reachable
    // too, or the solver is structurally unable to boil or melt it: with
    // `water → H2O(L)` alone a hydrogen flame has no steam to make, and
    // the minimiser returns H2 and O2 sitting unreacted at 927 °C
    // labelled equilibrium (curiosity th-034); with `NaNO3 → NaNO3(a)`
    // alone, sodium above 500 K has no in-range condensed carrier and
    // leaves the vessel as an absurd nitrate vapour. Admit every
    // condensed record of identical composition (the (a)/(b)/(L) phase
    // families). The gas of that composition is admitted only when it is
    // unique — gas isomers share a composition without being phases of
    // anything — AND the condensed family's data ends below combustion
    // temperatures, so the vapour is the substance's only continuation.
    // Water qualifies (liquid record ends at 600 K); salt, magnesium and
    // iron do not (liquid records to 6000 K), which keeps burning
    // magnesium from venting itself as metal vapour and a salted flame
    // from inventing sodium chloride gas as an ignition.
    let siblings: Vec<&'static Species> = pool
        .iter()
        .flat_map(|mapped| {
            // The temperature where this substance's condensed data ends,
            // across its whole phase family. Water: 600 K. Salt, magnesium,
            // iron: their liquid records run to 6000 K.
            let condensed_top = db()
                .species
                .values()
                .filter(|s| !s.is_gas() && s.composition == mapped.composition)
                .filter_map(|s| s.t_range().map(|(_, hi)| hi))
                .fold(f64::NEG_INFINITY, f64::max);
            db().species.values().filter(move |s| {
                s.composition == mapped.composition
                    && s.name != mapped.name
                    && (!s.is_gas()
                        || (condensed_top < 1000.0
                            && db()
                                .species
                                .values()
                                .filter(|o| o.is_gas() && o.composition == s.composition)
                                .count()
                                == 1))
            })
        })
        .collect();
    pool.extend(siblings);
    pool.sort_by(|a, b| a.name.cmp(&b.name));
    pool.dedup_by(|a, b| a.name == b.name);
    pool
}

pub struct ThermalEquilibrator;

/// What the vessel offers the solver, and what it holds back.
struct Charge {
    /// Element budget including the atmospheric reservoir.
    budget: BTreeMap<String, f64>,
    /// The atmosphere the vessel stands in, as species rather than
    /// elements: how much N2 and O2 was admitted. Both the reactants'
    /// enthalpy and the open-vessel rule that governs it need the amounts
    /// by name — see [`OpenAtmosphere`].
    air: BTreeMap<String, f64>,
    /// Registry species that mapped, with their amounts.
    mapped: Vec<(SpeciesId, f64)>,
    /// At least one input came from CEA's feed-only section rather than its
    /// admissible equilibrium products (liquid methanol or ethanol).
    used_feed_thermo: bool,
}

pub(crate) fn heat_input(
    vessel: &Vessel,
) -> Result<Option<&kerotakis_core::vessel::HeatInput>, SolveError> {
    let Some(input) = vessel.heat_input.as_ref() else {
        return Ok(None);
    };
    if !input.delivered_j.is_finite()
        || input.delivered_j < 0.0
        || !input.temperature.0.is_finite()
        || input.temperature.0 <= 0.0
    {
        return Err(SolveError::NotConverged {
            solver: "cea-thermal".into(),
            detail: "invalid HEAT source budget".into(),
        });
    }
    if input
        .contents
        .iter()
        .chain(vessel.contents.iter())
        .any(|p| !p.moles.0.is_finite() || p.moles.0 < 0.0)
    {
        return Err(SolveError::NotConverged {
            solver: "cea-thermal".into(),
            detail: "invalid HEAT source stock".into(),
        });
    }
    if !matches!(vessel.thermal_mode, ThermalMode::Adiabatic)
        || !vessel.unpriced_heat.is_empty()
        || !vessel.unresolved_materials.is_empty()
        || !vessel.material_objects.is_empty()
        || !vessel.surfaces.is_empty()
        || !vessel.exchanges.is_empty()
        || !vessel.solid_solutions.is_empty()
        || !vessel.electrodes.is_empty()
        || !vessel.adsorbed.is_empty()
        || vessel
            .step_start
            .as_ref()
            .is_some_and(|s| !s.gas_out.is_empty())
    {
        return Ok(None);
    }
    let stocks = |contents: &[Portion]| {
        let mut totals = BTreeMap::<SpeciesId, f64>::new();
        for p in contents {
            *totals.entry(p.species.clone()).or_default() += p.moles.0;
        }
        totals
    };
    let before = stocks(&input.contents);
    let after = stocks(&vessel.contents);
    if before
        .values()
        .chain(after.values())
        .any(|n| !n.is_finite())
    {
        return Err(SolveError::NotConverged {
            solver: "cea-thermal".into(),
            detail: "overflowing HEAT source stock".into(),
        });
    }
    if before.len() != after.len()
        || before.iter().any(|(id, n)| {
            after
                .get(id)
                .is_none_or(|m| (m - n).abs() > n.abs() * 1e-9 + 1e-15)
        })
    {
        return Ok(None);
    }
    Ok(Some(input))
}

fn charge(vessel: &Vessel) -> Option<Charge> {
    // Unresolved materials remain spectators in the vessel. The mapped
    // dry feed may still be answered with explicit partial-coverage notes.
    // Liquid water means this is a solution: the aqueous engine owns it.
    let has_liquid_water = vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid);
    if has_liquid_water {
        return None;
    }

    let mut budget: BTreeMap<String, f64> = BTreeMap::new();
    let mut mapped = Vec::new();
    let mut condensed_moles = 0.0;
    let mut used_feed_thermo = false;
    for p in &vessel.contents {
        let Some(cea) = cea_name(&p.species.0) else {
            return None; // something here is outside the NASA data: decline
        };
        used_feed_thermo |= db().get(cea).is_none() && db().get_reactant(cea).is_some();
        let s = cea_species(&p.species.0)?;
        for (el, count) in &s.composition {
            *budget.entry(el.clone()).or_insert(0.0) += count * p.moles.0;
        }
        mapped.push((p.species.clone(), p.moles.0));
        if !s.is_gas() {
            condensed_moles += p.moles.0;
        }
    }
    if mapped.is_empty() {
        return None;
    }
    // A hot sealed atmosphere is a gas-law problem, not a combustion
    // equilibrium. Sending unchanged N2/O2/CO2 through the open-atmosphere
    // CEA route discarded every gas as exhaust, even though the headspace was
    // sealed. Stand down for this inert gas-only inventory; the bench's
    // pressure settlement retains it and applies P=nRT/V.
    if vessel.owns_headspace_gas()
        && condensed_moles == 0.0
        && mapped
            .iter()
            .all(|(id, _)| matches!(id.0.as_str(), "N2" | "O2" | "CO2"))
    {
        return None;
    }

    // The atmosphere the vessel stands in.
    // Keep the historical atmosphere floor for hot solids, but give an
    // organic fuel enough open-room oxygen to reach a fuel-lean flame. The
    // elemental oxygen demand for C/H/O matter is C + H/4 - O/2 mol O2;
    // a 20% margin avoids making the arbitrary teaching control volume the
    // limiting reagent.
    let stoich_o2 = budget.get("C").copied().unwrap_or(0.0)
        + budget.get("H").copied().unwrap_or(0.0) / 4.0
        - budget.get("O").copied().unwrap_or(0.0) / 2.0;
    let air_moles = ((condensed_moles.max(0.01)) * AIR_RATIO).max(stoich_o2.max(0.0) * 1.20 / 0.21);
    let mut air: BTreeMap<String, f64> = BTreeMap::new();
    // A carbonate crucible releases its own gas. It is not a flame that
    // entrains a finite air charge: redrawing that arbitrary thermal mass
    // every numerical burner chunk makes calcination depend on chunk size.
    // This narrow CaCO3/CaO/CO2 model uses total product-gas pressure 1 bar,
    // hence (unlike the flame route) no dilution by ambient nitrogen.
    for (name, fraction) in AIR
        .iter()
        .filter(|_| vessel.uses_atmospheric_reservoir() && !carbonate_crucible(vessel))
    {
        let Some(s) = db().get(name) else { continue };
        *air.entry(s.name.clone()).or_insert(0.0) += fraction * air_moles;
        for (el, count) in &s.composition {
            *budget.entry(el.clone()).or_insert(0.0) += count * fraction * air_moles;
        }
    }
    Some(Charge {
        budget,
        air,
        mapped,
        used_feed_thermo,
    })
}

fn carbonate_crucible(vessel: &Vessel) -> bool {
    !vessel.contents.is_empty()
        && vessel
            .contents
            .iter()
            .any(|p| p.moles.0 > 0.0 && matches!(p.species.0.as_str(), "CaCO3" | "CaO"))
        && vessel.unresolved_materials.is_empty()
        && vessel.material_objects.is_empty()
        && vessel.surfaces.is_empty()
        && vessel.exchanges.is_empty()
        && vessel.solid_solutions.is_empty()
        && vessel.electrodes.is_empty()
        && vessel.adsorbed.is_empty()
        && vessel
            .contents
            .iter()
            .all(|p| matches!(p.species.0.as_str(), "CaCO3" | "CaO" | "CO2"))
}

impl Equilibrator for ThermalEquilibrator {
    fn name(&self) -> &'static str {
        "cea-thermal"
    }

    fn element_conservation_tolerance(&self) -> Option<f64> {
        Some(1e-7)
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        // Equilibrium is not the whole story. At room temperature a
        // magnesium ribbon is thermodynamically desperate to become oxide
        // and simply does not, because an oxide skin protects it — that is
        // kinetics, and it belongs to L5. Below this threshold the lab
        // leaves solids alone, which is also what a user sees on a bench.
        vessel.temperature.0 >= KINETIC_THRESHOLD_K && charge(vessel).is_some()
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let Some(charge) = charge(vessel) else {
            return Ok(Vec::new());
        };
        // BRD-041: equilibrium would burn methane and air the moment they
        // were warm. A real mixture below its autoignition temperature sits
        // there until a spark, and `ignite` IS the spark — it takes the
        // vessel to 1200 K, above every tabulated autoignition point. So a
        // warm, unsparked fuel is answered by name rather than burned, and
        // the slow clock's mechanism packs own whatever happens with time.
        let unsparked = kerotakis_core::combustion::unsparked_fuels(vessel);
        if !unsparked.is_empty() {
            return Ok(unsparked
                .into_iter()
                .map(|(fuel, autoignition_k)| Event::BelowAutoignition {
                    vessel: vessel.id,
                    fuel,
                    autoignition: kerotakis_core::Kelvin(autoignition_k),
                    temperature: vessel.temperature,
                })
                .collect());
        }
        if let Some(result) = crate::closed::equilibrate(vessel) {
            return result;
        }
        // This path owns an open-room HP/TP balance and a vented exhaust.
        // Finite and swept boundaries require different energy, pressure and
        // outlet contracts; preserving their inventory is better than posing
        // them as an open beaker. Autoignition diagnostics above remain valid.
        if let Some(refusal) = crate::closed::boundary_refusal(vessel) {
            return Ok(vec![refusal]);
        }
        let elements: Vec<String> = charge.budget.keys().cloned().collect();
        let carbonate_route = carbonate_crucible(vessel);
        let mut pool = if carbonate_route {
            crate::carbonate::pool()
        } else {
            pool_for(&elements)
        };
        if charge.used_feed_thermo {
            let feed_stems = charge
                .mapped
                .iter()
                .filter_map(|(id, _)| {
                    let name = cea_name(&id.0)?;
                    db().get_reactant(name)
                        .is_some()
                        .then(|| cea_identity_stem(name))
                })
                .collect::<Vec<_>>();
            pool.extend(
                ["CO2", "H2O", "H2", "O2", "N2"]
                    .into_iter()
                    .filter_map(|name| db().get(name)),
            );
            pool.extend(db().species.values().filter(|candidate| {
                candidate.is_gas()
                    && feed_stems
                        .iter()
                        .any(|stem| *stem == cea_identity_stem(&candidate.name))
            }));
            pool.sort_by(|a, b| a.name.cmp(&b.name));
            pool.dedup_by(|a, b| a.name == b.name);
            // A flame calculation needs the feed's vapour plus the small,
            // stable C/H/O/N gas set. The general dry-solid pool also
            // contains every named hydrocarbon with those elements; offering
            // unrelated isomers makes the minimisation ill-conditioned and
            // would let burning ethanol turn into hexane merely because both
            // names exist in the shelf registry.
            pool.retain(|species| {
                species.is_gas()
                    && (matches!(species.name.as_str(), "CO2" | "H2O" | "H2" | "O2" | "N2")
                        || feed_stems
                            .iter()
                            .any(|stem| *stem == cea_identity_stem(&species.name)))
            });
        }
        let t = vessel.temperature.0.clamp(200.0, 6000.0);
        let heat_budget = heat_input(vessel)?;
        let delivered_heat = heat_budget.map_or(0.0, |input| input.delivered_j);
        let using_heat_budget = heat_budget.is_some();
        // The spark initiates a small reaction zone; it does not preheat
        // the whole liquid or the incoming atmosphere to 1200 K.
        let feed_t = heat_budget.map_or_else(
            || {
                vessel
                    .ignition_feed_temperature
                    .map_or(t, |temperature| temperature.0.clamp(200.0, 6000.0))
            },
            |input| input.temperature.0,
        );

        // Enthalpy the vessel and its share of the atmosphere carry into
        // the problem.
        let mut recovered_condensed_feed = false;
        let feed_contents = heat_budget.map_or(vessel.contents.as_slice(), |input| {
            input.contents.as_slice()
        });
        let h_before = feed_contents.iter().try_fold(0.0, |h, portion| {
            let source = cea_species(&portion.species.0);
            if let Some(source) = source {
                let actual_phase_supported = db()
                    .species
                    .values()
                    .chain(std::iter::once(source))
                    .any(|candidate| {
                        candidate.composition == source.composition
                            && cea_identity_stem(&candidate.name) == cea_identity_stem(&source.name)
                            && if portion.phase == Phase::Gas {
                                candidate.is_gas()
                            } else {
                                !candidate.is_gas()
                                    && candidate.name.ends_with("(L)")
                                        == (portion.phase == Phase::Liquid)
                            }
                            && (candidate
                                .t_range()
                                .is_some_and(|(lo, hi)| feed_t >= lo && feed_t <= hi)
                                || formation_anchor(candidate, feed_t))
                    });
                if using_heat_budget && !actual_phase_supported {
                    return Err(SolveError::NotConverged {
                        solver: "cea-thermal".into(),
                        detail: "HEAT prestate has no verified actual-phase thermochemistry".into(),
                    });
                }
                if portion.phase != Phase::Gas && !actual_phase_supported {
                    recovered_condensed_feed = true;
                }
            }
            let value = source
                .and_then(|s| enthalpy_within_record(s, feed_t, portion.phase))
                .ok_or_else(|| SolveError::NotConverged {
                    solver: "cea-thermal".into(),
                    detail: format!(
                        "no in-range feed thermochemistry for {} at {feed_t} K in {:?}",
                        portion.species, portion.phase
                    ),
                })?;
            Ok::<_, SolveError>(h + value * portion.moles.0)
        })? + delivered_heat
            + air_enthalpy(&charge);

        // An adiabatic vessel conserves enthalpy, so the products *and*
        // the temperature come out of one solve. Dividing a reaction's ΔH
        // by the vessel's own heat capacity would be wrong by orders of
        // magnitude here: a gram of burning magnesium heats the air around
        // it, not just the speck of oxide it leaves behind.
        if std::env::var("KERO_CEA_DEBUG").is_ok() {
            eprintln!(
                "CHARGE t={t:.1} budget={:?} mapped={:?} h_before={h_before:.4e} feed={}",
                charge.budget, charge.mapped, charge.used_feed_thermo
            );
        }
        // The vessel stands open. Its atmosphere is in the budget because
        // the chemistry needs it there — the oxygen a burn consumes, and
        // the nitrogen whose partial pressure sets where a carbonate gives
        // way — but it is the room's heat, not the vessel's. Carrying it
        // here lets the adiabatic search refuse to spend it.
        let atmosphere = OpenAtmosphere {
            admitted: charge.air.clone(),
            inlet_k: Kelvin::STANDARD.0,
        };
        let adiabatic = matches!(vessel.thermal_mode, ThermalMode::Adiabatic);
        let eq = if adiabatic {
            let result = if carbonate_route {
                crate::gibbs::equilibrate_hp_using(
                    &charge.budget,
                    &pool,
                    h_before,
                    None,
                    300.0,
                    // Deterministic NASA/mass-action arithmetic can close
                    // below core's 1e-9 J source-headroom stopping scale.
                    // This is numerical precision, not dataset accuracy.
                    Some(1e-10),
                    |t| crate::carbonate::tp(&charge.budget, &pool, t),
                )
            } else {
                crate::gibbs::equilibrate_hp_open(
                    &charge.budget,
                    &pool,
                    h_before,
                    1.0,
                    Some(&atmosphere),
                )
            };
            result.map_err(|e| SolveError::NotConverged {
                solver: "cea-thermal".to_string(),
                detail: e.to_string(),
            })?
        } else {
            (if carbonate_route {
                crate::carbonate::tp(&charge.budget, &pool, t)
            } else {
                equilibrate_tp(&charge.budget, &pool, t, 1.0)
            })
            .map_err(|e| SolveError::NotConverged {
                solver: "cea-thermal".to_string(),
                detail: e.to_string(),
            })?
        };
        if carbonate_route && eq.temperature > 2000.0 {
            return Err(SolveError::NotConverged { solver: "cea-thermal".into(), detail: "restricted CaCO3/CaO and CO2/CO/O2 crucible model is limited to 2000 K; calcium vapour, graphite and atomic gases are not represented".into() });
        }
        let t_final = eq.temperature;

        // Put the products back at the physical feed temperature and compare
        // their enthalpy with the reactants'. The difference is the chemical
        // energy the adiabatic solve converted into sensible heat. This uses
        // the same NASA-9 records and exact equilibrium composition as the
        // flame-temperature solve; it is not inferred from a UI animation.
        let products_at_initial_t = eq.composition.iter().try_fold(0.0, |h, (name, moles)| {
            let product = db().get(name).ok_or_else(|| SolveError::NotConverged {
                solver: "cea-thermal".into(),
                detail: "missing product reference thermochemistry".into(),
            })?;
            let phase = if product.is_gas() {
                Phase::Gas
            } else if product.name.ends_with("(L)") {
                Phase::Liquid
            } else {
                Phase::Solid
            };
            let value = enthalpy_within_record(product, feed_t, phase).ok_or_else(|| {
                SolveError::NotConverged {
                    solver: "cea-thermal".into(),
                    detail: format!("no in-range product reference for {name} at {feed_t} K"),
                }
            })?;
            Ok::<_, SolveError>(h + value * moles)
        })?;
        let reaction_energy_j = (h_before - delivered_heat - products_at_initial_t).max(0.0);
        let mut dataset_sources = eq.sources.clone();
        dataset_sources.extend(charge.mapped.iter().filter_map(|(id, _)| {
            let name = cea_name(&id.0)?;
            let feed = db().get_reactant(name)?;
            Some(format!("{}: {}", feed.name, feed.reference))
        }));
        dataset_sources.sort();
        dataset_sources.dedup();

        // Map the result back: condensed species become vessel contents,
        // product gases leave, atmospheric gases return to the reservoir.
        //
        // The two halves are read at two different temperatures on purpose.
        // What STAYS in the vessel is at the flame's own equilibrium,
        // because that is the state the vessel is in. What LEAVES is read
        // at [`EXHAUST_K`], because a plume is not a flame and the frozen
        // dissociation products of one are not what comes off the other.
        let mut events = Vec::new();
        let mut contents: Vec<Portion> = Vec::new();
        for (name, moles) in &eq.composition {
            let Some(s) = db().get(name) else { continue };
            if s.is_gas() || *moles <= 0.0 {
                continue;
            }
            let reg = registry_row(name)?;
            contents.push(Portion {
                species: SpeciesId::new(reg.key),
                moles: Moles(*moles),
                // CEA distinguishes condensed phases in the record identity;
                // the room-temperature registry phase is not the solved phase.
                phase: if s.name.ends_with("(L)") {
                    Phase::Liquid
                } else {
                    Phase::Solid
                },
            });
        }
        let vented = exhaust_composition(&eq.composition, &pool).unwrap_or_else(|| {
            // The re-solve did not converge. Say what the flame said rather
            // than say nothing; the printed equation and these moles may
            // then disagree, which is the old behaviour and is at least not
            // a loss of matter.
            eq.composition.clone()
        });
        for (name, moles) in &vented {
            let Some(s) = db().get(name) else { continue };
            if !s.is_gas() {
                continue;
            }
            // Returned air is balanced against the admitted reservoir below.
            if AIR.iter().any(|(n, _)| n == name) {
                continue;
            }
            if *moles <= 0.0 {
                continue;
            }
            let reg = registry_row(name)?;
            events.push(Event::GasEvolved {
                vessel: vessel.id,
                species: SpeciesId::new(reg.key),
                moles: Moles(*moles),
            });
        }
        // Validate the minimizer and exhaust re-equilibration against the
        // full numerical charge before closing roundoff in the room's net
        // N2/O2 transfers. Presentation thresholds must not debit atoms.
        let mut product_atoms = BTreeMap::<String, f64>::new();
        for (name, amount) in eq
            .composition
            .iter()
            .filter(|(name, _)| db().get(name).is_some_and(|s| !s.is_gas()))
            .chain(
                vented
                    .iter()
                    .filter(|(name, _)| db().get(name).is_some_and(|s| s.is_gas())),
            )
        {
            let s = db().get(name).unwrap();
            for (element, count) in &s.composition {
                *product_atoms.entry(element.clone()).or_default() += count * amount;
            }
        }
        for (element, before) in &charge.budget {
            let after = product_atoms.get(element).copied().unwrap_or(0.0);
            if (after - before).abs() > before.abs() * 1e-7 + 1e-15 {
                return Err(SolveError::NotConverged { solver: "cea-thermal".into(), detail: format!("minimizer/exhaust elemental residual for {element}: {before:e} -> {after:e}") });
            }
        }
        let before_ledger = kerotakis_core::ledger::ConservedLedger::from_vessel(vessel);
        let mut retained = vessel.clone();
        retained.contents = contents.clone();
        let mut accounted_atoms =
            kerotakis_core::ledger::ConservedLedger::from_vessel(&retained).elements;
        for event in &events {
            if let Event::GasEvolved { species, moles, .. } = event {
                let row = species::lookup(species).unwrap();
                let formula = kerotakis_core::stoich::parse_formula(row.formula).map_err(|e| {
                    SolveError::NotConverged {
                        solver: "cea-thermal".into(),
                        detail: e.to_string(),
                    }
                })?;
                for (element, count) in formula.counts {
                    *accounted_atoms.entry(element).or_default() += count * moles.0;
                }
            }
        }
        for (name, _) in AIR {
            let returned = vented
                .iter()
                .filter(|(s, _)| s == name)
                .map(|(_, n)| *n)
                .sum::<f64>();
            let raw_net_inlet = charge.air.get(*name).copied().unwrap_or(0.0) - returned;
            let element = if *name == "O2" { "O" } else { "N" };
            // Equivalent to admitted-minus-returned to native element
            // tolerance, but based on actual bookable product atoms. This
            // prevents native roundoff inventing an unbalanced trace inlet
            // for an element absent from the owned feed (especially N).
            let net_inlet = if charge.air.is_empty() {
                // With no atmospheric inlet, a roundoff correction must
                // never manufacture an oxygen source. Book the actual
                // generated gas; the full native atom check above applies.
                raw_net_inlet
            } else {
                (accounted_atoms.get(element).copied().unwrap_or(0.0)
                    - before_ledger.elements.get(element).copied().unwrap_or(0.0))
                    / 2.0
            };
            if (net_inlet - raw_net_inlet).abs() * 2.0
                > charge.budget.get(element).copied().unwrap_or(0.0) * 1e-7 + 1e-15
            {
                return Err(SolveError::NotConverged {
                    solver: "cea-thermal".into(),
                    detail: format!("air roundoff closure exceeded native tolerance for {element}"),
                });
            }
            if net_inlet == 0.0 {
                continue;
            }
            let reg = registry_row(name)?;
            events.push(if net_inlet > 0.0 {
                Event::GasAbsorbed {
                    vessel: vessel.id,
                    species: SpeciesId::new(reg.key),
                    moles: Moles(net_inlet),
                }
            } else {
                // Any original finite O2/N2 which left is included here.
                Event::GasEvolved {
                    vessel: vessel.id,
                    species: SpeciesId::new(reg.key),
                    moles: Moles(-net_inlet),
                }
            });
        }

        // Report what changed among the solids.
        //
        // A solid that appears where there is no liquid is not a
        // precipitate and did not make anything go cloudy — magnesium
        // burnt in an empty beaker leaves an ash. The flag travels with
        // the event so the renderer can say "left behind" rather than
        // "came out of solution"; the event itself is unchanged, so a
        // quest claiming `precipitated:MgO` still matches.
        let dry = vessel.liquid_volume().0 <= 0.0
            && !contents.iter().any(|p| {
                matches!(p.phase, Phase::Liquid | Phase::Aqueous)
                    && p.moles.0 > kerotakis_core::OBSERVABLE_MOLES
            });
        for portion in &contents {
            let before = vessel.moles_of(&portion.species).0;
            let delta = portion.moles.0 - before;
            if delta >= kerotakis_core::OBSERVABLE_MOLES {
                events.push(Event::Precipitated {
                    vessel: vessel.id,
                    species: portion.species.clone(),
                    moles: Moles(delta),
                    dry,
                });
            }
        }
        // Per species and over every condensed phase it is in, not per
        // portion and solids only.
        //
        // `after` always summed the whole species; `before` read ONE portion
        // and only if that portion happened to be solid. A ribbon of
        // magnesium that a spark has taken past its 923 K melting point is a
        // liquid portion by the time the minimisation sees it, so it burned,
        // precipitated its oxide, and was never reported consumed — the
        // `Precipitated` loop above has no such guard, which is exactly the
        // asymmetry `kero codex lint` caught: six entries kept
        // `precipitated:MgO` and lost `consumed:Mg`.
        //
        // `combustion.rs` fixed the same defect on the core path and left
        // the reason: "Taking it from `Phase::Solid` alone would have burned
        // the diesel without consuming it, and the ledger would have carried
        // it forever." A fuel that burned has left the vessel whatever phase
        // was holding it.
        //
        // Gases are excluded because they have their own events above
        // (`GasEvolved`), not because they cannot be consumed.
        for (index, p) in vessel.contents.iter().enumerate() {
            if p.phase == Phase::Gas {
                continue;
            }
            // One event per species: skip if an earlier condensed portion of
            // the same species has already been counted.
            if vessel.contents[..index]
                .iter()
                .any(|c| c.species == p.species && c.phase != Phase::Gas)
            {
                continue;
            }
            let before: f64 = vessel
                .contents
                .iter()
                .filter(|c| c.species == p.species && c.phase != Phase::Gas)
                .map(|c| c.moles.0)
                .sum();
            let after: f64 = contents
                .iter()
                .filter(|c| c.species == p.species)
                .map(|c| c.moles.0)
                .sum();
            if before - after >= kerotakis_core::OBSERVABLE_MOLES {
                events.push(Event::Consumed {
                    vessel: vessel.id,
                    species: p.species.clone(),
                    moles: Moles(before - after),
                    remaining: Some(Moles(after)),
                });
            }
        }

        let burning_fuel = (!events.is_empty())
            .then(|| {
                LIQUID_FUEL_COMBUSTION.iter().find(|(key, _)| {
                    charge
                        .mapped
                        .iter()
                        .any(|(species, amount)| species.0 == *key && *amount > 1e-12)
                })
            })
            .flatten();
        if let Some((_, equation)) = burning_fuel {
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: equation.to_string(),
            });
        }
        if !events.is_empty()
            && charge
                .mapped
                .iter()
                .any(|(species, amount)| species.0 == "Fe" && *amount > 1e-12)
            && contents
                .iter()
                .any(|portion| portion.species.0 == "Fe2O3" && portion.moles.0 > 1e-12)
        {
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: "4 Fe(s) + 3 O₂(g) → 2 Fe₂O₃(s)".to_string(),
            });
        }

        let changed = !events.is_empty() || using_heat_budget || vessel.contents != contents;
        // Asked before the assignment, because it is a claim about what the
        // burn LEFT: a vessel that had something in it and now has nothing.
        let holds_nothing = contents.is_empty() && !vessel.contents.is_empty();
        vessel.contents = contents;

        // The temperature the adiabatic solve found.
        // An accepted HP result is a numerical energy state, even when
        // chemistry is invisible or the correction is less than 1 K.
        // Presentation thresholds must never retain the provisional HEAT
        // temperature instead of the verified delivered-energy solution.
        if changed && adiabatic {
            let from = vessel.temperature;
            vessel.temperature = Kelvin(t_final);
            if from != vessel.temperature {
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from,
                    to: Kelvin(t_final),
                });
            }
        }

        if changed {
            events.push(Event::ThermalEquilibrium {
                vessel: vessel.id,
                temperature: Kelvin(t_final),
                reaction_energy_j: (reaction_energy_j > 1.0).then_some(reaction_energy_j),
                holds_nothing,
                provenance: Provenance::new(
                    "Gibbs minimisation (Kerotakis)",
                    // A name, and nothing but a name.
                    "NASA CEA thermo.inp",
                    // `NASA-9` is the polynomial set's name and stays put;
                    // the rest is a sentence. Ignition-feed and spectator
                    // clauses preserve their own translation recipes.
                    {
                        let model = Phrase::new(
                            "provenance.model.nasa9-polynomials",
                            "{name} polynomials, ideal gas + pure condensed phases",
                            vec![(
                                "name".to_string(),
                                Slot::text("NASA-9"),
                            )],
                        );
                        let model = if carbonate_route {
                            Phrase::new("provenance.model.cea-carbonate-crucible", "{model}; restricted CaCO3/CaO condensed phases and CO2/CO/O2 ideal gases at 1 bar, at most 2000 K; no entrained room-air thermal mass, calcium vapour, graphite or atomic gases", vec![("model".into(), Slot::phrase(model))])
                        } else if vessel.ignition_trial {
                            Phrase::new(
                                "provenance.model.ignition-feed-boundary",
                                "{model}; localized spark zone {zone} K, bulk feed {feed} K, room air {air} K; open-air allocation {ratio} times the represented feed amount",
                                vec![
                                    ("model".into(), Slot::phrase(model)),
                                    ("zone".into(), Slot::number(format!("{t:.2}"))),
                                    ("feed".into(), Slot::number(format!("{feed_t:.2}"))),
                                    ("air".into(), Slot::number(format!("{:.2}", Kelvin::STANDARD.0))),
                                    ("ratio".into(), Slot::number(format!("{:.2}", charge.air.values().sum::<f64>() / charge.mapped.iter().map(|(_, amount)| amount).sum::<f64>().max(f64::MIN_POSITIVE)))),
                                ],
                            )
                        } else {
                            Phrase::new("provenance.model.room-air-reservoir", "{model}; owned bulk feed {feed} K, external room-air inlet {air} K", vec![
                                ("model".into(), Slot::phrase(model)),
                                ("feed".into(), Slot::number(format!("{feed_t:.2}"))),
                                ("air".into(), Slot::number(format!("{:.2}", Kelvin::STANDARD.0))),
                            ])
                        };
                        let model = if using_heat_budget {
                            Phrase::new("provenance.model.cea-heat-budget", "{model}; HEAT uses pre-pass NASA enthalpy at {feed} K plus {heat} J actually delivered; phase latent heat is included in the HP product balance", vec![
                                ("model".into(), Slot::phrase(model)), ("feed".into(), Slot::number(format!("{feed_t:.2}"))), ("heat".into(), Slot::number(format!("{delivered_heat:.2}"))),
                            ])
                        } else { model };
                        let model = if recovered_condensed_feed {
                            Phrase::new(
                                "provenance.model.cea-stable-condensed-feed",
                                "{model}; an out-of-range booked condensed feed uses an available NASA phase at bulk temperature; preceding dry HEAT phase-change latent energy is not reconstructed",
                                vec![("model".into(), Slot::phrase(model))],
                            )
                        } else { model };
                        if vessel.unresolved_materials.iter().any(|portion| portion.amount > 0.0) {
                            Phrase::new(
                                "provenance.model.unresolved-spectators",
                                "{model}; only represented species participate in combustion; unresolved materials are retained without a reaction or heat-capacity model",
                                vec![("model".into(), Slot::phrase(model))],
                            )
                        } else {
                            model
                        }
                    },
                    dataset_sources,
                        Phrase::bare(
                            "routing.dry-solids-and-gases",
                            "chosen because this vessel is dry solids and gases, which the aqueous engine does not model",
                        ),
                ),
            });
        }
        Ok(events)
    }
}

fn air_enthalpy(charge: &Charge) -> f64 {
    // Unowned room air enters at its own standard room temperature, even
    // when the owned feed has been heated. Whether it may pay heat once the solve
    // lands is `OpenAtmosphere`'s question, not this one's: what stays air
    // has its sensible change taken back out of the balance wherever it
    // would be paying for the chemistry rather than being warmed by it.
    charge
        .air
        .iter()
        .filter_map(|(name, moles)| Some(db().get(name)?.h(Kelvin::STANDARD.0)? * moles))
        .sum()
}

#[cfg(test)]
mod feed_thermochemistry_contracts {
    use super::*;

    #[test]
    fn gas_reference_anchor_uses_header_hf_without_extrapolating_the_cp_fit() {
        let ethanol = db().get("C2H5OH").unwrap();
        assert_eq!(ethanol.t_range().unwrap().0, 300.0);
        assert_eq!(
            enthalpy_within_record(ethanol, crate::T_REF, Phase::Gas),
            Some(ethanol.h_formation)
        );
        assert!(
            (ethanol.h_formation + 234_950.0).abs() < 1.0,
            "NASA C2H5OH header's authoritative Hf datum"
        );
        // The reference point supplies no invented derivative/integral at
        // neighboring temperatures below the actual Cp domain.
        for t in [crate::T_REF - 0.01, crate::T_REF + 0.01, 299.0] {
            assert!(enthalpy_within_record(ethanol, t, Phase::Gas).is_none());
        }
        assert_eq!(
            enthalpy_within_record(ethanol, 300.0, Phase::Gas),
            ethanol.h(300.0)
        );
    }

    #[test]
    fn magnesium_above_its_solid_range_uses_liquid_not_vapour_enthalpy() {
        let solid = db().get("Mg(cr)").unwrap();
        let liquid = db().get("Mg(L)").unwrap();
        let gas = db().get("Mg").unwrap();
        let t = 991.6;
        let expected = liquid.h(t).unwrap();
        assert_eq!(
            enthalpy_within_record(solid, t, Phase::Solid),
            Some(expected)
        );
        assert_eq!(
            enthalpy_within_record(solid, t, Phase::Liquid),
            Some(expected)
        );
        assert!((gas.h(t).unwrap() - expected).abs() > 100_000.0);
        assert_eq!(enthalpy_within_record(solid, t, Phase::Gas), gas.h(t));
        assert_eq!(
            enthalpy_within_record(solid, 923.0, Phase::Solid),
            solid.h(923.0)
        );
        assert_eq!(
            enthalpy_within_record(solid, 923.0, Phase::Liquid),
            liquid.h(923.0)
        );
    }

    #[test]
    fn feed_phase_outside_every_condensed_record_never_clamps_polynomials() {
        let solid = db().get("Mg(cr)").unwrap();
        assert!(enthalpy_within_record(solid, 50.0, Phase::Solid).is_none());
        assert_eq!(
            enthalpy_within_record(solid, 7000.0, Phase::Solid),
            db().get("Mg").unwrap().h(7000.0)
        );
        assert!(enthalpy_within_record(solid, 30_000.0, Phase::Solid).is_none());
    }

    #[test]
    fn heated_feed_cannot_preheat_unowned_room_air_for_free() {
        let mut v = Vessel::new(kerotakis_core::VesselId(0), "hot owned magnesium");
        v.temperature = Kelvin(991.6);
        v.deposit(SpeciesId::new("Mg"), Moles(0.05), Phase::Liquid);
        let charge = charge(&v).unwrap();
        let expected_room = charge
            .air
            .iter()
            .map(|(name, n)| n * db().get(name).unwrap().h(Kelvin::STANDARD.0).unwrap())
            .sum::<f64>();
        let fabricated_hot_air = charge
            .air
            .iter()
            .map(|(name, n)| n * db().get(name).unwrap().h(v.temperature.0).unwrap())
            .sum::<f64>();
        assert_eq!(air_enthalpy(&charge), expected_room);
        assert!(fabricated_hot_air - expected_room > 8000.0);
    }
}

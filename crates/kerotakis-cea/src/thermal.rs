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
//!   calcite below its decomposition point has no gas phase at all.
//! * **Species map by composition, not by a hand-written table.** A solid
//!   `CaCO3` in the registry finds CEA's `CaCO3(cr)` because their formulas
//!   agree; nothing lists the pair.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kerotakis_core::species::{self, Phase};
use kerotakis_core::{
    Equilibrator, Event, Kelvin, Moles, Portion, Provenance, SolveError, SpeciesId, ThermalMode,
    Vessel,
};

use crate::gibbs::equilibrate_tp;
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
const AIR_RATIO: f64 = 8.0;

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

fn enthalpy_within_record(species: &Species, temperature: f64) -> Option<f64> {
    if db().get_reactant(&species.name).is_none() && species.is_gas() {
        return species.h(temperature);
    }
    if db().get_reactant(&species.name).is_none()
        && !species.is_gas()
        && species
            .t_range()
            .is_some_and(|(low, high)| temperature >= low && temperature <= high)
    {
        return species.h(temperature);
    }
    if species
        .t_range()
        .is_some_and(|(_, high)| temperature > high)
    {
        // `ignite` represents a small vapour zone brought to flame
        // temperature. Once a feed-only liquid record ends at its boiling
        // range, continue with the matching product-side gas polynomial;
        // evaluating a liquid polynomial hundreds of kelvin beyond its
        // validity would be worse than either phase model.
        if let Some(gas) = db().species.values().find(|candidate| {
            candidate.is_gas()
                && cea_identity_stem(&candidate.name) == cea_identity_stem(&species.name)
        }) {
            return gas.h(temperature);
        }
    }
    let t = species
        .t_range()
        .map(|(low, high)| temperature.clamp(low, high))
        .unwrap_or(temperature);
    species.h(t)
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
    /// Elements contributed by the atmosphere alone, so the products can be
    /// told apart from the air they burned in.
    from_air: BTreeMap<String, f64>,
    /// Registry species that mapped, with their amounts.
    mapped: Vec<(SpeciesId, f64)>,
    /// At least one input came from CEA's feed-only section rather than its
    /// admissible equilibrium products (liquid methanol or ethanol).
    used_feed_thermo: bool,
}

fn charge(vessel: &Vessel) -> Option<Charge> {
    // Liquid water means this is a solution: the aqueous engine owns it.
    let has_liquid_water = vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid);
    if has_liquid_water && vessel.temperature.0 < 373.15 {
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
    let mut from_air: BTreeMap<String, f64> = BTreeMap::new();
    for (name, fraction) in AIR {
        let Some(s) = db().get(name) else { continue };
        for (el, count) in &s.composition {
            let n = count * fraction * air_moles;
            *budget.entry(el.clone()).or_insert(0.0) += n;
            *from_air.entry(el.clone()).or_insert(0.0) += n;
        }
    }
    Some(Charge {
        budget,
        from_air,
        mapped,
        used_feed_thermo,
    })
}

impl Equilibrator for ThermalEquilibrator {
    fn name(&self) -> &'static str {
        "cea-thermal"
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
        let elements: Vec<String> = charge.budget.keys().cloned().collect();
        let mut pool = pool_for(&elements);
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

        // Enthalpy the vessel and its share of the atmosphere carry into
        // the problem.
        let h_before: f64 = charge
            .mapped
            .iter()
            .filter_map(|(sid, moles)| {
                let s = cea_species(&sid.0)?;
                Some(enthalpy_within_record(s, t)? * moles)
            })
            .sum::<f64>()
            + air_enthalpy(&charge, t);

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
        let adiabatic = matches!(vessel.thermal_mode, ThermalMode::Adiabatic);
        let (eq, feed_tp_fallback) = if adiabatic {
            match crate::gibbs::equilibrate_hp(&charge.budget, &pool, h_before, 1.0) {
                Ok(eq) => (eq, false),
                // HP is the preferred flame calculation. The first liquid-
                // fuel slice retains a deterministic TP fallback at the
                // explicit ignition-zone temperature because CEA's
                // feed-only condensed record can leave the HP iteration
                // without a feasible initial temperature bracket. The event
                // provenance says which route was used.
                Err(_) if charge.used_feed_thermo => (
                    equilibrate_tp(&charge.budget, &pool, t, 1.0).map_err(|e| {
                        SolveError::NotConverged {
                            solver: "cea-thermal".to_string(),
                            detail: e.to_string(),
                        }
                    })?,
                    true,
                ),
                Err(e) => {
                    return Err(SolveError::NotConverged {
                        solver: "cea-thermal".to_string(),
                        detail: e.to_string(),
                    })
                }
            }
        } else {
            (
                equilibrate_tp(&charge.budget, &pool, t, 1.0).map_err(|e| {
                    SolveError::NotConverged {
                        solver: "cea-thermal".to_string(),
                        detail: e.to_string(),
                    }
                })?,
                false,
            )
        };
        let t_final = eq.temperature;

        // Put the products back at the ignition temperature and compare
        // their enthalpy with the reactants'. The difference is the chemical
        // energy the adiabatic solve converted into sensible heat. This uses
        // the same NASA-9 records and exact equilibrium composition as the
        // flame-temperature solve; it is not inferred from a UI animation.
        let products_at_initial_t: f64 = eq
            .composition
            .iter()
            .filter_map(|(name, moles)| db().get(name)?.h(t).map(|h| h * moles))
            .sum();
        let reaction_energy_j = (h_before - products_at_initial_t).max(0.0);
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
        let mut events = Vec::new();
        let mut contents: Vec<Portion> = Vec::new();
        for (name, moles) in &eq.composition {
            let Some(s) = db().get(name) else { continue };
            let Some(reg) = species::REGISTRY.iter().find(|r| {
                cea_name(r.key)
                    .is_some_and(|mapped| cea_identity_stem(mapped) == cea_identity_stem(name))
            }) else {
                // The pool is built from nameable species, so this cannot
                // normally happen — but if it ever does, refuse rather than
                // quietly lose matter.
                return Err(SolveError::NotConverged {
                    solver: "cea-thermal".to_string(),
                    detail: format!(
                        "the equilibrium contains {name}, which has no name in the registry"
                    ),
                });
            };
            if s.is_gas() {
                // Air that stayed air is the reservoir's business.
                let atmospheric = AIR.iter().any(|(n, _)| n == name);
                let produced = if atmospheric {
                    let consumed_from_air: f64 = charge
                        .from_air
                        .get(s.composition.keys().next().unwrap_or(&String::new()))
                        .copied()
                        .unwrap_or(0.0);
                    let _ = consumed_from_air;
                    0.0
                } else {
                    *moles
                };
                if produced >= kerotakis_core::OBSERVABLE_MOLES {
                    events.push(Event::GasEvolved {
                        vessel: vessel.id,
                        species: SpeciesId::new(reg.key),
                        moles: Moles(produced),
                    });
                }
            } else if *moles > 1e-12 {
                contents.push(Portion {
                    species: SpeciesId::new(reg.key),
                    moles: Moles(*moles),
                    phase: reg.standard_phase,
                });
            }
        }

        // Report what changed among the solids.
        for portion in &contents {
            let before = vessel.moles_of(&portion.species).0;
            let delta = portion.moles.0 - before;
            if delta >= kerotakis_core::OBSERVABLE_MOLES {
                events.push(Event::Precipitated {
                    vessel: vessel.id,
                    species: portion.species.clone(),
                    moles: Moles(delta),
                });
            }
        }
        for p in &vessel.contents {
            let after: f64 = contents
                .iter()
                .filter(|c| c.species == p.species)
                .map(|c| c.moles.0)
                .sum();
            if p.phase == Phase::Solid && p.moles.0 - after >= kerotakis_core::OBSERVABLE_MOLES {
                events.push(Event::Consumed {
                    vessel: vessel.id,
                    species: p.species.clone(),
                    moles: Moles(p.moles.0 - after),
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

        let changed = !events.is_empty();
        vessel.contents = contents;

        // The temperature the adiabatic solve found.
        if changed && adiabatic && (t_final - vessel.temperature.0).abs() > 1.0 {
            let from = vessel.temperature;
            vessel.temperature = Kelvin(t_final);
            events.push(Event::TemperatureChanged {
                vessel: vessel.id,
                from,
                to: Kelvin(t_final),
            });
        }

        if changed {
            events.push(Event::ThermalEquilibrium {
                vessel: vessel.id,
                temperature: Kelvin(t_final),
                reaction_energy_j: (reaction_energy_j > 1.0).then_some(reaction_energy_j),
                provenance: Provenance {
                    engine: "Gibbs minimisation (Kerotakis)".to_string(),
                    dataset: "NASA CEA thermo.inp".to_string(),
                    model: if feed_tp_fallback {
                        "NASA-9 polynomials, ideal gas + pure condensed phases; TP liquid-feed fallback at the explicit ignition-zone temperature".to_string()
                    } else {
                        "NASA-9 polynomials, ideal gas + pure condensed phases".to_string()
                    },
                    dataset_sources,
                    routing: if feed_tp_fallback {
                        "liquid fuel used CEA's separate feed thermochemistry; HP did not bracket, so composition was solved at the explicit ignition-zone temperature and the reaction energy remains reported separately".to_string()
                    } else {
                        "chosen because this vessel is dry solids and gases, which the aqueous engine does not model".to_string()
                    },
                },
            });
        }
        Ok(events)
    }
}

fn air_enthalpy(charge: &Charge, t: f64) -> f64 {
    // The air that entered the problem, valued at the same temperature.
    AIR.iter()
        .filter_map(|(name, fraction)| {
            let s = db().get(name)?;
            let el = s.composition.keys().next()?;
            let atoms = s.composition.values().next()?;
            let moles = charge.from_air.get(el).copied().unwrap_or(0.0) / atoms;
            let _ = fraction;
            Some(s.h(t)? * moles)
        })
        .sum()
}

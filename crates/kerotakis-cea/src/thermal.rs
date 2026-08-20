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
            let best = db()
                .species
                .values()
                .filter(|s| s.is_gas() == want_gas && s.composition == want)
                .min_by(|a, b| {
                    let ga = a.g(298.15).unwrap_or(f64::MAX);
                    let gb = b.g(298.15).unwrap_or(f64::MAX);
                    ga.total_cmp(&gb)
                });
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
    names
        .iter()
        .filter_map(|n| db().get(n))
        .filter(|s| {
            !s.composition.is_empty()
                && s.composition
                    .keys()
                    .all(|el| elements.iter().any(|e| e == el))
        })
        .collect()
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
    for p in &vessel.contents {
        let Some(cea) = cea_name(&p.species.0) else {
            return None; // something here is outside the NASA data: decline
        };
        let s = db().get(cea)?;
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
    let air_moles = (condensed_moles.max(0.01)) * AIR_RATIO;
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
        let pool = pool_for(&elements);
        let t = vessel.temperature.0.clamp(200.0, 6000.0);

        // Enthalpy the vessel and its share of the atmosphere carry into
        // the problem.
        let h_before: f64 = charge
            .mapped
            .iter()
            .filter_map(|(sid, moles)| {
                let s = db().get(cea_name(&sid.0)?)?;
                Some(s.h(t)? * moles)
            })
            .sum::<f64>()
            + air_enthalpy(&charge, t);

        // An adiabatic vessel conserves enthalpy, so the products *and*
        // the temperature come out of one solve. Dividing a reaction's ΔH
        // by the vessel's own heat capacity would be wrong by orders of
        // magnitude here: a gram of burning magnesium heats the air around
        // it, not just the speck of oxide it leaves behind.
        let adiabatic = matches!(vessel.thermal_mode, ThermalMode::Adiabatic);
        let eq = if adiabatic {
            crate::gibbs::equilibrate_hp(&charge.budget, &pool, h_before, 1.0)
        } else {
            equilibrate_tp(&charge.budget, &pool, t, 1.0)
        }
        .map_err(|e| SolveError::NotConverged {
            solver: "cea-thermal".to_string(),
            detail: e.to_string(),
        })?;
        let t_final = eq.temperature;

        // Map the result back: condensed species become vessel contents,
        // product gases leave, atmospheric gases return to the reservoir.
        let mut events = Vec::new();
        let mut contents: Vec<Portion> = Vec::new();
        for (name, moles) in &eq.composition {
            let Some(s) = db().get(name) else { continue };
            let Some(reg) = species::REGISTRY
                .iter()
                .find(|r| cea_name(r.key) == Some(name.as_str()))
            else {
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
                    phase: Phase::Solid,
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
                provenance: Provenance {
                    engine: "Gibbs minimisation (Kerotakis)".to_string(),
                    dataset: "NASA CEA thermo.inp".to_string(),
                    model: "NASA-9 polynomials, ideal gas + pure condensed phases".to_string(),
                    dataset_sources: eq.sources.clone(),
                    routing: "chosen because this vessel is dry solids and gases, which the aqueous engine does not model".to_string(),
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

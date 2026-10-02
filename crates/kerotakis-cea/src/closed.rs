//! Finite gas equilibrium and explicit refusals for remaining boundary domains.
//!
//! A rigid adiabatic vessel requires U/V equilibrium, not the open route's
//! H/P balance. A pressure-controlled finite vessel needs its own closed H/P
//! balance with retained products and volume work. A sweep needs an explicit
//! carrier-gas inlet/outlet model. The supported gas-only HP/UV slice verifies
//! energy, pressure and inventory; condensed, multiphase and swept domains
//! remain explicit refusals and never borrow oxygen or vent finite inventory.
use kerotakis_core::phrase::Phrase;
use kerotakis_core::{ops::NotModelledCause, vessel::Headspace, Event, Vessel};
use std::collections::BTreeMap;

use crate::{db, equilibrate_tp, Equilibrium, Species, R};
use kerotakis_core::{
    species, Kelvin, Moles, Phase, Portion, Provenance, SolveError, SpeciesId, ThermalMode,
};

fn failure(detail: impl Into<String>) -> SolveError {
    SolveError::NotConverged {
        solver: "cea-closed".into(),
        detail: detail.into(),
    }
}

/// Pressure-consistent gas equilibrium at assigned T and V (litres).
/// NASA RP-1311 Part II examples 3/4 distinguish HP and UV combustion:
/// https://ntrs.nasa.gov/api/citations/19960044559/downloads/19960044559.pdf.
/// For ideal gases a TP equilibrium whose P=nRT/V is also the fixed-volume
/// equilibrium. Condensed phases are deliberately outside this first slice.
pub fn equilibrate_tv(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    t: f64,
    volume_l: f64,
) -> Result<Equilibrium, SolveError> {
    if !t.is_finite()
        || !(200.0..=6000.0).contains(&t)
        || !volume_l.is_finite()
        || volume_l <= 0.0
        || candidates.is_empty()
        || candidates.iter().any(|s| !s.is_gas())
        || budget.is_empty()
        || budget.values().any(|n| !n.is_finite() || *n <= 0.0)
    {
        return Err(failure("invalid or unsupported gas TV inputs"));
    }
    let active: Vec<_> = candidates
        .iter()
        .copied()
        .filter(|s| s.composition.keys().all(|e| budget.contains_key(e)))
        .collect();
    let atoms = budget.values().sum::<f64>();
    let largest = active
        .iter()
        .map(|s| s.composition.values().sum::<f64>())
        .fold(0.0, f64::max);
    let smallest = active
        .iter()
        .map(|s| s.composition.values().sum::<f64>())
        .fold(f64::INFINITY, f64::min);
    if largest <= 0.0 || !atoms.is_finite() {
        return Err(failure("no finite gas pressure bracket"));
    }
    // Atom conservation bounds total molecular moles regardless of reaction.
    // J/(bar litre) = 1/100, hence this pressure is in bar.
    let factor = R * t / (100.0 * volume_l);
    let mut lo = atoms / largest * factor * (1.0 - 1e-7);
    let mut hi = atoms / smallest * factor * (1.0 + 1e-7);
    if !lo.is_finite() || !hi.is_finite() || lo <= 0.0 {
        return Err(failure("nonfinite gas pressure bracket"));
    }
    for _ in 0..48 {
        let pressure = (lo * hi).sqrt();
        let eq =
            equilibrate_tp(budget, &active, t, pressure).map_err(|e| failure(e.to_string()))?;
        let actual = eq.gas_moles * factor;
        if !actual.is_finite() || actual <= 0.0 {
            return Err(failure("invalid product gas pressure"));
        }
        if (actual - pressure).abs() <= actual * 1e-8 {
            return Ok(eq);
        }
        if actual > pressure {
            lo = pressure;
        } else {
            hi = pressure;
        }
    }
    Err(failure("TV pressure residual did not close"))
}

/// Assigned-energy equilibrium with a verified root rather than a clamped
/// temperature floor. Failed samples do not imply an energy residual sign.
fn energy_root(
    target: f64,
    internal: bool,
    mut evaluate: impl FnMut(f64) -> Result<Equilibrium, SolveError>,
) -> Result<Equilibrium, SolveError> {
    if !target.is_finite() {
        return Err(failure("nonfinite energy target"));
    }
    let energy = |eq: &Equilibrium| {
        eq.enthalpy
            - if internal {
                eq.gas_moles * R * eq.temperature
            } else {
                0.0
            }
    };
    let tolerance = 1e-5 + target.abs() * 1e-8;
    let mut lower: Option<(f64, f64)> = None;
    let mut bracket = None;
    for t in [
        200.0, 300.0, 450.0, 650.0, 900.0, 1200.0, 1600.0, 2100.0, 2700.0, 3400.0, 4200.0, 5000.0,
        6000.0,
    ] {
        let Ok(eq) = evaluate(t) else { continue };
        let residual = energy(&eq) - target;
        if !residual.is_finite() {
            return Err(failure("nonfinite product energy"));
        }
        if residual.abs() <= tolerance {
            return Ok(eq);
        }
        if residual < 0.0 {
            lower = Some((t, residual));
        } else if let Some((low, _)) = lower {
            bracket = Some((low, t));
            break;
        }
    }
    let Some((mut lo, mut hi)) = bracket else {
        return Err(failure("no converged energy bracket within 200–6000 K"));
    };
    for _ in 0..48 {
        let t = 0.5 * (lo + hi);
        let eq = evaluate(t)?;
        let residual = energy(&eq) - target;
        if !residual.is_finite() {
            return Err(failure("nonfinite product energy"));
        }
        if residual.abs() <= tolerance {
            return Ok(eq);
        }
        if residual < 0.0 {
            lo = t;
        } else {
            hi = t;
        }
    }
    Err(failure("closed energy residual did not close"))
}

pub fn equilibrate_uv(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    internal_energy: f64,
    volume_l: f64,
) -> Result<Equilibrium, SolveError> {
    energy_root(internal_energy, true, |t| {
        equilibrate_tv(budget, candidates, t, volume_l)
    })
}

pub fn equilibrate_hp_verified(
    budget: &BTreeMap<String, f64>,
    candidates: &[&Species],
    enthalpy: f64,
    pressure_bar: f64,
) -> Result<Equilibrium, SolveError> {
    if budget.is_empty()
        || budget.values().any(|n| !n.is_finite() || *n <= 0.0)
        || candidates.is_empty()
        || !pressure_bar.is_finite()
        || pressure_bar <= 0.0
        || candidates.iter().any(|s| !s.is_gas())
    {
        return Err(failure("invalid or unsupported gas HP inputs"));
    }
    energy_root(enthalpy, false, |t| {
        equilibrate_tp(budget, candidates, t, pressure_bar).map_err(|e| failure(e.to_string()))
    })
}

fn registry_gas(s: &Species) -> Option<&'static species::SpeciesData> {
    species::REGISTRY.iter().find(|row| {
        !row.formula.contains(['+', '-'])
            && kerotakis_core::stoich::parse_formula(row.formula)
                .is_ok_and(|formula| formula.counts == s.composition)
    })
}

/// Supported finite gas-only slice. Returning None leaves other boundary
/// domains to the explicit refusal; errors never mutate stock or metadata.
pub(crate) fn equilibrate(vessel: &mut Vessel) -> Option<Result<Vec<Event>, SolveError>> {
    if !matches!(
        vessel.headspace,
        Headspace::Sealed { .. } | Headspace::PressureControlled { .. }
    ) || !vessel.unresolved_materials.is_empty()
        || !vessel.material_objects.is_empty()
        || !vessel.surfaces.is_empty()
        || !vessel.exchanges.is_empty()
        || !vessel.solid_solutions.is_empty()
        || !vessel.electrodes.is_empty()
        || !vessel.adsorbed.is_empty()
        || vessel.contents.is_empty()
        || vessel.contents.iter().any(|p| p.phase != Phase::Gas)
    {
        return None;
    }
    // Restricted nameable combustion pool, with stable nitrogen oxides;
    // ionisation and unnamed radicals are not modelled. Avoid unsupported
    // graphite/condensation equilibria by requiring fuel-lean carbon feed
    // and checking that any produced steam remains above 800 K.
    let pool: Vec<_> = [
        "H2", "O2", "N2", "CO", "CO2", "H2O", "CH4", "NO", "NO2", "N2O", "NH3",
    ]
    .into_iter()
    .filter_map(|name| db().get(name))
    .filter(|s| registry_gas(s).is_some())
    .collect();
    let mut budget = BTreeMap::new();
    let heat_budget = match crate::thermal::heat_input(vessel) {
        Ok(input) => input,
        Err(e) => return Some(Err(e)),
    };
    if heat_budget.is_some_and(|input| input.contents.iter().any(|p| p.phase != Phase::Gas)) {
        return None;
    }
    let delivered_heat = heat_budget.map_or(0.0, |input| input.delivered_j);
    let using_heat_budget = heat_budget.is_some();
    let feed_t = heat_budget.map_or(
        vessel
            .ignition_feed_temperature
            .unwrap_or(vessel.temperature)
            .0,
        |input| input.temperature.0,
    );
    if !feed_t.is_finite() || !(200.0..=6000.0).contains(&feed_t) {
        return Some(Err(failure("feed temperature outside gas data domain")));
    }
    let mut enthalpy = 0.0;
    let mut feed_moles = 0.0;
    for portion in &vessel.contents {
        let row = species::lookup(&portion.species)?;
        let formula = kerotakis_core::stoich::parse_formula(row.formula).ok()?;
        let gas = pool.iter().find(|s| s.composition == formula.counts)?;
        if !portion.moles.0.is_finite() || portion.moles.0 <= 0.0 {
            return Some(Err(failure("invalid finite gas stock")));
        }
        for (element, count) in &gas.composition {
            *budget.entry(element.clone()).or_insert(0.0) += count * portion.moles.0;
        }
        if !gas
            .t_range()
            .is_some_and(|(lo, hi)| feed_t >= lo && feed_t <= hi)
        {
            return Some(Err(failure(
                "feed temperature outside the species thermochemistry range",
            )));
        }
        enthalpy += gas.h(feed_t)? * portion.moles.0;
        feed_moles += portion.moles.0;
    }
    let carbon = budget.get("C").copied().unwrap_or(0.0);
    if carbon > 0.0
        && budget.get("O").copied().unwrap_or(0.0) + 1e-12
            < 2.0 * carbon + budget.get("H").copied().unwrap_or(0.0) / 2.0
    {
        return None;
    }
    Some((|| {
        let adiabatic = matches!(vessel.thermal_mode, ThermalMode::Adiabatic);
        let (eq, route) = match vessel.headspace {
            Headspace::Sealed { volume } if adiabatic => (
                equilibrate_uv(
                    &budget,
                    &pool,
                    enthalpy - feed_moles * R * feed_t + delivered_heat,
                    volume.0,
                )?,
                "U/V",
            ),
            Headspace::PressureControlled { pressure, .. } if adiabatic => (
                equilibrate_hp_verified(
                    &budget,
                    &pool,
                    enthalpy + delivered_heat,
                    pressure.0 / 100000.0,
                )?,
                "H/P",
            ),
            Headspace::Sealed { volume } => (
                equilibrate_tv(&budget, &pool, vessel.temperature.0, volume.0)?,
                "T/V",
            ),
            Headspace::PressureControlled { pressure, .. } => (
                equilibrate_tp(&budget, &pool, vessel.temperature.0, pressure.0 / 100000.0)
                    .map_err(|e| failure(e.to_string()))?,
                "T/P",
            ),
            _ => unreachable!(),
        };
        if eq.pressure_bar > 100.0 || (eq.temperature < 800.0 && eq.moles_of("H2O") > 1e-10) {
            return Ok(vec![boundary_refusal(vessel).unwrap()]);
        }
        let before = kerotakis_core::ledger::ConservedLedger::from_vessel(vessel);
        let products_h_at_feed = eq
            .composition
            .iter()
            .map(|(name, amount)| {
                db().get(name)
                    .and_then(|s| s.h(feed_t))
                    .map(|h| h * amount)
                    .ok_or_else(|| failure("unavailable product reference energy"))
            })
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .sum::<f64>();
        let chemical_energy = enthalpy
            - products_h_at_feed
            - if matches!(vessel.headspace, Headspace::Sealed { .. }) {
                (feed_moles - eq.gas_moles) * R * feed_t
            } else {
                0.0
            };
        let mut candidate = vessel.clone();
        candidate.contents = eq
            .composition
            .iter()
            .map(|(name, amount)| {
                let gas = db()
                    .get(name)
                    .ok_or_else(|| failure("missing product thermochemistry"))?;
                let row =
                    registry_gas(gas).ok_or_else(|| failure("unnameable finite gas product"))?;
                Ok(Portion {
                    species: SpeciesId::new(row.key),
                    moles: Moles(*amount),
                    phase: Phase::Gas,
                })
            })
            .collect::<Result<Vec<_>, SolveError>>()?;
        candidate.temperature = Kelvin(eq.temperature);
        candidate.refresh_pressure();
        if adiabatic {
            let actual_h = eq
                .composition
                .iter()
                .map(|(name, amount)| {
                    db().get(name)
                        .and_then(|s| s.h(eq.temperature))
                        .map(|h| h * amount)
                        .ok_or_else(|| failure("unavailable product energy"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .sum::<f64>();
            let internal = matches!(vessel.headspace, Headspace::Sealed { .. });
            let target = enthalpy + delivered_heat
                - if internal {
                    feed_moles * R * feed_t
                } else {
                    0.0
                };
            let actual = actual_h
                - if internal {
                    candidate.gas_moles().0 * R * eq.temperature
                } else {
                    0.0
                };
            if !actual.is_finite() || (actual - target).abs() > 2e-5 + target.abs() * 2e-8 {
                return Err(failure("finite gas readback failed energy conservation"));
            }
        }
        if !before
            .check_against(
                &kerotakis_core::ledger::ConservedLedger::from_vessel(&candidate),
                1e-7,
                1e-12,
            )
            .is_empty()
        {
            return Err(failure("finite gas readback failed elemental conservation"));
        }
        if (candidate.pressure.0 / 100000.0 - eq.pressure_bar).abs() > eq.pressure_bar * 2e-8 {
            return Err(failure("finite gas pressure readback mismatch"));
        }
        let mut events = Vec::new();
        for product in &candidate.contents {
            let formed = product.moles.0 - vessel.moles_of(&product.species).0;
            if formed > 1e-12 {
                events.push(Event::GasContained {
                    vessel: vessel.id,
                    species: product.species.clone(),
                    moles: Moles(formed),
                });
            }
        }
        for feed in &vessel.contents {
            let consumed = feed.moles.0 - candidate.moles_of(&feed.species).0;
            if consumed > 1e-12 {
                events.push(Event::Consumed {
                    vessel: vessel.id,
                    species: feed.species.clone(),
                    moles: Moles(consumed),
                    remaining: Some(candidate.moles_of(&feed.species)),
                });
            }
        }
        if (candidate.temperature.0 - vessel.temperature.0).abs() > 1e-6 {
            events.push(Event::TemperatureChanged {
                vessel: vessel.id,
                from: vessel.temperature,
                to: candidate.temperature,
            });
        }
        let model = Phrase::bare("provenance.model.cea-finite-gas", "NASA-9 restricted nameable ideal-gas equilibrium; retained inventory, no ambient air; no condensed phases, radicals, ionisation or real-gas correction; steam above 800 K and pressure at most 100 bar");
        let model = if using_heat_budget {
            Phrase::new(
                "provenance.model.cea-finite-heat-budget",
                "{model}; HEAT adds {heat} J to NASA pre-pass {energy} at {feed} K",
                vec![
                    ("model".into(), kerotakis_core::phrase::Slot::phrase(model)),
                    (
                        "heat".into(),
                        kerotakis_core::phrase::Slot::number(format!("{delivered_heat:.2}")),
                    ),
                    (
                        "energy".into(),
                        kerotakis_core::phrase::Slot::text(
                            if matches!(vessel.headspace, Headspace::Sealed { .. }) {
                                "U"
                            } else {
                                "H"
                            },
                        ),
                    ),
                    (
                        "feed".into(),
                        kerotakis_core::phrase::Slot::number(format!("{feed_t:.2}")),
                    ),
                ],
            )
        } else {
            model
        };
        events.push(Event::ThermalEquilibrium {
            vessel: vessel.id,
            temperature: candidate.temperature,
            reaction_energy_j: (chemical_energy > 1.0).then_some(chemical_energy),
            holds_nothing: false,
            provenance: Provenance::new(
                "Gibbs minimisation (Kerotakis)",
                "NASA CEA thermo.inp",
                model,
                eq.sources.clone(),
                Phrase::new(
                    "routing.cea-finite-gas",
                    "{route}: closed ideal-gas equilibrium with retained products",
                    vec![("route".into(), kerotakis_core::phrase::Slot::text(route))],
                ),
            ),
        });
        *vessel = candidate;
        Ok(events)
    })())
}

pub(crate) fn boundary_refusal(vessel: &Vessel) -> Option<Event> {
    let reason = match vessel.headspace {
        Headspace::Open => return None,
        Headspace::Sealed { .. } => Phrase::bare(
            "not-modeled.cea-sealed-energy-boundary",
            "this sealed charge lies outside the validated finite ideal-gas energy and pressure domain; condensed phases, interfaces, fuel-rich carbon, steam below 800 K and pressures above 100 bar remain unsupported, so its inventory is retained",
        ),
        Headspace::PressureControlled { .. } => Phrase::bare(
            "not-modeled.cea-finite-pressure-boundary",
            "this finite pressure-controlled charge lies outside the validated ideal-gas enthalpy and expansion-work domain; condensed phases, interfaces, fuel-rich carbon, steam below 800 K and pressures above 100 bar remain unsupported, so its inventory is retained",
        ),
        Headspace::Swept { .. } => Phrase::bare(
            "not-modeled.cea-swept-boundary",
            "the swept vessel does not admit room oxygen; the open-flame CEA route has no carrier-gas inlet and outlet balance for this boundary, so no thermal chemistry has been committed",
        ),
    };
    Some(Event::not_modeled(
        vessel.id,
        NotModelledCause::BoundaryMismatch,
        reason,
    ))
}

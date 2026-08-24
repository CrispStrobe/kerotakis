//! EXP-44: heat of mixing on the bench, as a state function.
//!
//! hᴱ is a state function of (composition, T), so the bench never
//! integrates a path: each settle, the vessel's total excess enthalpy
//! Hᴱ = n·hᴱ(x) is recomputed and the DIFFERENCE from the stored value
//! is released or absorbed as heat. Pouring the ethanol in one go or
//! five reaches the same temperature to machine precision — the
//! path-independence test is the design, stated as an invariant.
//!
//! The allowlist is the honesty core: UNIFAC parameters are VLE-fitted
//! and their Gibbs–Helmholtz hᴱ is only sometimes right. A pair earns
//! heat application by having its derived curve verified against the
//! literature sign and shape (acetone–water: verified, applied).
//! Ethanol–water is deliberately WITHHELD: the real hand-warmth exists,
//! but this parameter set inverts the dilute-end sign, and a wrong
//! sign taught with confidence is worse than a stated gap. The thermo
//! crate's `the_allowlist_verification_holds` test pins the deviation
//! so a parameter upgrade reopens the question loudly.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::Phase;
use crate::vessel::Vessel;
use kerotakis_thermo::unifac::GroupDecomposition;

/// Pairs whose UNIFAC-derived hᴱ curve was verified against literature
/// sign and shape. Each entry: (organic species key, verification note).
pub const VERIFIED_PAIRS_WITH_WATER: &[(&str, &str)] = &[(
    "propanone",
    "acetone–water: exothermic mid-range, mildly positive at high x — \
     matches the literature S-shape (e.g. IDST calorimetric series)",
)];

fn groups_of(key: &str) -> Option<GroupDecomposition> {
    let mut g = GroupDecomposition::new();
    match key {
        "water" => {
            g.insert(16, 1);
        }
        "propanone" => {
            g.insert(1, 1);
            g.insert(18, 1);
        }
        _ => return None,
    }
    Some(g)
}

/// The vessel's current total excess enthalpy under the allowlist
/// model: exactly water + one verified organic, single liquid phase.
/// Everything else is 0 — not because mixtures elsewhere have no heat,
/// but because this model refuses to claim numbers it cannot back.
fn total_excess_j(vessel: &Vessel) -> f64 {
    if crate::solve::layered_pair(vessel).is_some() {
        return 0.0;
    }
    let mut water = 0.0;
    let mut organic: Option<(&str, f64)> = None;
    let mut other_liquid = false;
    for p in &vessel.contents {
        if !matches!(p.phase, Phase::Liquid | Phase::Aqueous) || p.moles.0 <= 0.0 {
            continue;
        }
        match p.species.0.as_str() {
            "water" => water += p.moles.0,
            key if VERIFIED_PAIRS_WITH_WATER.iter().any(|(k, _)| *k == key) => {
                organic = Some(match organic {
                    Some((k, m)) if k == key => (k, m + p.moles.0),
                    Some(_) => return 0.0, // two organics: outside the model
                    None => (key, p.moles.0),
                });
            }
            _ => other_liquid = true,
        }
    }
    let Some((key, n_org)) = organic else {
        return 0.0;
    };
    if other_liquid || water <= 0.0 || n_org <= 0.0 {
        return 0.0;
    }
    let n = water + n_org;
    let x = n_org / n;
    let comps = [
        (groups_of(key).expect("allowlisted"), x),
        (groups_of("water").expect("water"), 1.0 - x),
    ];
    // Evaluated at the 25 °C reference, NOT the vessel's own T: mixing
    // enthalpies are quoted at standard temperature, hᴱ's own
    // T-dependence sits below this parameter set's honesty floor — and
    // the fixed reference is what makes the state-function bookkeeping
    // EXACTLY path-independent (evaluating at current T would let the
    // pour path leak back in through hᴱ(T)).
    const T_REF: f64 = 298.15;
    n * kerotakis_thermo::excess::excess_enthalpy_j_per_mol(&comps, T_REF)
}

/// Below this, the difference is numerical noise, not chemistry.
const OBSERVABLE_J: f64 = 0.05;

/// Applies the heat of mixing incrementally as composition changes.
/// Runs in every stack between the mixing pass and the honesty pass.
pub struct MixingEnthalpyEquilibrator;

impl Equilibrator for MixingEnthalpyEquilibrator {
    fn name(&self) -> &'static str {
        "heat-of-mixing"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        (total_excess_j(vessel) - vessel.excess_enthalpy_j).abs() > OBSERVABLE_J
            || vessel.excess_enthalpy_j.abs() > OBSERVABLE_J
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        let now = total_excess_j(vessel);
        let delta = now - vessel.excess_enthalpy_j;
        vessel.excess_enthalpy_j = now;
        if delta.abs() <= OBSERVABLE_J {
            return Ok(events);
        }
        // Forming a more negative Hᴱ releases heat: q = −ΔHᴱ.
        let q = -delta;
        if matches!(vessel.thermal_mode, crate::vessel::ThermalMode::Adiabatic) {
            let cp = vessel.heat_capacity();
            if cp > 0.0 {
                let from = vessel.temperature;
                let to = crate::units::Kelvin(from.0 + q / cp);
                vessel.temperature = to;
                events.push(Event::TemperatureChanged {
                    vessel: vessel.id,
                    from,
                    to,
                });
            }
        }
        events.push(Event::HeatOfMixing {
            vessel: vessel.id,
            joules: q,
        });
        Ok(events)
    }
}

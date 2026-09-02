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
//! literature sign and shape (acetone–water: verified, applied). It is an
//! allowlist of *binaries*, not of organics-with-water — see
//! `VERIFIED_PAIRS` for why that distinction was worth making and what it
//! was costing.
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

/// Binary pairs whose UNIFAC-derived hᴱ curve was verified against
/// literature sign and shape. Each entry: (species key, species key,
/// verification note). Order does not matter — see [`verified_pair`].
///
/// **This used to be `VERIFIED_PAIRS_WITH_WATER`, and every entry was an
/// organic paired with an implicit water.** That shape was an accident of
/// the first verified pair rather than a claim about chemistry: heat of
/// mixing is a property of a binary, and nothing in `hᴱ` or in the UNIFAC
/// machinery behind it privileges water. The anchor had a real cost — a
/// non-aqueous binary read `0.0` J *structurally*, so acetone and
/// chloroform (the textbook exothermic pair, and one of EXP-44's stated
/// gaps) would have returned "no heat" even with a complete and verified
/// parameter table behind it. That is a wrong answer wearing the costume
/// of an honest refusal, which is the failure mode this whole allowlist
/// exists to prevent.
///
/// The list is still one pair long. Widening it is not this change:
/// admitting a pair means verifying its derived curve against the
/// literature, and THERMO-004 restricts the sources. What changed is that
/// a non-aqueous pair is now *reachable* — the only thing standing between
/// acetone–chloroform and a computed answer is its groups, its parameters
/// and its verification, which is where the question belongs.
pub const VERIFIED_PAIRS: &[(&str, &str, &str)] = &[(
    "propanone",
    "water",
    "acetone–water: exothermic mid-range, mildly positive at high x — \
     matches the literature S-shape (e.g. IDST calorimetric series)",
)];

/// The verification note for an unordered pair, or `None` if the pair has
/// not earned heat application.
///
/// Unordered on purpose: which species a learner poured first is not a
/// thermodynamic fact, and a table that had to list both orders would
/// eventually list one.
pub fn verified_pair(a: &str, b: &str) -> Option<&'static str> {
    VERIFIED_PAIRS
        .iter()
        .find_map(|(x, y, note)| ((*x == a && *y == b) || (*x == b && *y == a)).then_some(*note))
}

/// The UNIFAC main-group decomposition for a species this model knows.
///
/// Kept beside the allowlist because the two must move together: a pair
/// cannot be verified without its groups, and groups without a verified
/// pair compute nothing.
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

/// The vessel's current total excess enthalpy under the allowlist model:
/// exactly one verified binary, single liquid phase. Everything else is 0
/// — not because mixtures elsewhere have no heat, but because this model
/// refuses to claim numbers it cannot back.
///
/// "One verified binary" is the whole rule now. It used to be "water plus
/// one verified organic", which refused a non-aqueous pair for the wrong
/// reason: not because the pair was unverified, but because neither
/// component was water. See [`VERIFIED_PAIRS`].
///
/// Note what is still deliberately refused, so the widening is not read as
/// more than it is: a third liquid, because hᴱ over a ternary is not the
/// sum of its binaries and this model has no ternary claim; a separated
/// pair, because two layers are not a mixture; and any pair not on the
/// allowlist, because UNIFAC parameters are VLE-fitted and their
/// Gibbs–Helmholtz hᴱ is only sometimes right.
fn total_excess_j(vessel: &Vessel) -> f64 {
    if crate::solve::layered_pair(vessel).is_some() {
        return 0.0;
    }
    // The liquid components, pooled by species: one entry per distinct
    // liquid, in the order they were first met.
    let mut liquids: Vec<(&str, f64)> = Vec::new();
    for p in &vessel.contents {
        if !matches!(p.phase, Phase::Liquid | Phase::Aqueous) || p.moles.0 <= 0.0 {
            continue;
        }
        let key = p.species.0.as_str();
        match liquids.iter_mut().find(|(k, _)| *k == key) {
            Some(entry) => entry.1 += p.moles.0,
            None => liquids.push((key, p.moles.0)),
        }
    }
    // A binary, and only a binary: one liquid has nothing to mix with,
    // three are a ternary this model makes no claim about.
    let [(key_a, n_a), (key_b, n_b)] = liquids[..] else {
        return 0.0;
    };
    if verified_pair(key_a, key_b).is_none() || n_a <= 0.0 || n_b <= 0.0 {
        return 0.0;
    }
    let (Some(groups_a), Some(groups_b)) = (groups_of(key_a), groups_of(key_b)) else {
        // Unreachable while the allowlist and `groups_of` move together,
        // and a silent 0.0 rather than a panic if they ever do not: a
        // missing decomposition is a gap in this model, not a claim that
        // the pair mixes without heat.
        return 0.0;
    };
    let n = n_a + n_b;
    let x = n_a / n;
    let comps = [(groups_a, x), (groups_b, 1.0 - x)];
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

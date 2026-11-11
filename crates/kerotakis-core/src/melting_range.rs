//! Melting RANGES for the materials that have no melting point, and the
//! one honesty rule every solid on this bench obeys when heated past where
//! it melts (reviewed-transition-points lane, 2026-11-11).
//!
//! # Why a range and not a point
//!
//! Paraffin wax is a blend of long-chain alkanes and a polyolefin is a
//! semicrystalline polymer with a distribution of lamella thicknesses.
//! Neither has a sharp melting point: each melts over a range, and the
//! range is a property of the grade. A single curated temperature would be
//! a sharper claim than the material supports, so these rows carry the
//! sourced lower and upper bound and nothing else. The melting-point
//! apparatus reports the range (`PurityVerdict::MeltingRange`), never a
//! point.
//!
//! # Melting through a range pays its latent heat (po-009 / th-004, 2026-11-11)
//!
//! A range row that carries a sourced enthalpy of fusion AND a sourced
//! lower bound melts by [`settle_range_melts`]: the liquid fraction rises
//! linearly from 0 at the lower bound to 1 at the upper bound, and the
//! fusion enthalpy is spread evenly over that span. The vessel's
//! temperature and the solid/liquid split are solved together from one
//! energy balance, so heat put into a wax inside its range goes mostly to
//! melting it and the thermometer rises far less than sensible heat alone
//! would say. Cooling back through the range releases the same heat.
//! Linear distribution is a stated model choice: the real shape of a
//! grade's melting endotherm is not sourced here; only the total and the
//! span are.
//!
//! # The rule ("never silently keep it solid")
//!
//! A solid stands above the upper bound of its sourced melting point or
//! range. If [`crate::phase_route::fusion_enthalpy`] or [`fusion_j_per_mol`]
//! carries a latent heat for it, it has already been melted and paid for,
//! and this module says nothing. Otherwise the bench has no latent heat to
//! pay with, so it cannot model the melt — and it says so with a typed
//! refusal (EN/DE/FR) rather than showing a white solid at 190 °C as
//! though that were a prediction: `not-modeled.melting-not-modelled` for
//! a sharp melting point, `not-modeled.melting-range-latent-heat-unsourced`
//! for a range, which also says the thermometer omits the latent heat.
//!
//! The polyolefins carry no enthalpy of fusion: the sourced value is that
//! of a 100 % crystalline polymer, a real grade's crystallinity is not
//! known to this bench, and their lower bound is not sourced either.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase, SpeciesId};
use crate::vessel::Vessel;

/// A sourced melting range at 101.325 kPa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeltingRange {
    pub species: &'static str,
    /// Lower bound, K, where one is sourced. `None` for the polyolefins:
    /// the source gives the equilibrium melting temperature of a perfect
    /// crystal, which bounds a real grade's melting from ABOVE, and nothing
    /// for where a commercial grade starts to melt.
    pub lower_k: Option<f64>,
    /// Upper bound, K: no crystal of this material survives above it.
    pub upper_k: f64,
    /// Primary citation with DOI, printed by the apparatus.
    pub source: &'static str,
    /// What the range does not claim.
    pub boundary: &'static str,
    /// Specific enthalpy of fusion of the material as a whole, J/kg, where
    /// one is sourced for it. `None` refuses the melt with a typed reason.
    pub fusion_j_per_kg: Option<f64>,
}

/// Paraffin wax and the two polyolefins of po-009.
///
/// * Paraffin wax: A. Sharma, V. V. Tyagi, C. R. Chen and D. Buddhi,
///   'Review on thermal energy storage with phase change materials and
///   applications', Renew. Sustain. Energy Rev. 13 (2009) 318-345,
///   doi:10.1016/j.rser.2007.10.005, Table 2a: technical-grade paraffins
///   melting between 42-44 and 66-68 degrees Celsius (315.15-341.15 K
///   across the grades), latent heat 189 kJ/kg for five grades (6106,
///   5838, 6035, 6403, 6499). Sharma et al. compile these rows from the
///   primary measurements of A. Abhat, 'Low temperature latent heat
///   thermal energy storage: heat storage materials', Solar Energy 30
///   (1983) 313-332, doi:10.1016/0038-092X(83)90186-X. The order of
///   magnitude is corroborated by the 150-250 kJ/kg quoted for paraffin
///   waxes in the property table of
///   https://en.wikipedia.org/wiki/Phase-change_material, which cites M.
///   Kenisarin and K. Mahkamov, Renew. Sustain. Energy Rev. 11 (2007)
///   1913-1965, doi:10.1016/j.rser.2006.05.005. Spread over the span of the grades
///   (315.15-341.15 K), because the grade in the beaker is not known.
/// * Polyethylene: U. Gaur and B. Wunderlich, J. Phys. Chem. Ref. Data 10
///   (1981) 119, doi:10.1063/1.555636: equilibrium melting temperature
///   414.6 +/- 0.5 K.
/// * Isotactic polypropylene: U. Gaur and B. Wunderlich, J. Phys. Chem.
///   Ref. Data 10 (1981) 1051, doi:10.1063/1.555650: best-estimate
///   equilibrium melting temperature 460.7 K.
pub const MELTING_RANGES: &[MeltingRange] = &[
    MeltingRange {
        species: "paraffin",
        lower_k: Some(315.15),
        upper_k: 341.15,
        source: "A. Sharma, V. V. Tyagi, C. R. Chen and D. Buddhi, 'Review on thermal energy storage with phase change materials and applications', Renew. Sustain. Energy Rev. 13 (2009) 318-345, doi:10.1016/j.rser.2007.10.005, Table 2a: technical-grade paraffin waxes melt between 42-44 and 66-68 degrees Celsius depending on grade; latent heat of fusion 189 kJ/kg for five grades (compiled from A. Abhat, Solar Energy 30 (1983) 313-332, doi:10.1016/0038-092X(83)90186-X)",
        boundary: "commercial paraffin wax is a blend of long-chain alkanes and melts over a range set by its grade; this is the span of the technical grades in the source, not the range of any one candle. The sourced 189 kJ/kg is paid as the wax is heated through that span, spread evenly over it: the total is sourced, the shape of the melting curve is not",
        fusion_j_per_kg: Some(189_000.0),
    },
    MeltingRange {
        species: "PE",
        lower_k: None,
        upper_k: 414.6,
        source: "U. Gaur and B. Wunderlich, 'Heat capacity and other thermodynamic properties of linear macromolecules. II. Polyethylene', J. Phys. Chem. Ref. Data 10 (1981) 119-152, doi:10.1063/1.555636: equilibrium melting temperature 414.6 +/- 0.5 K",
        boundary: "the equilibrium melting temperature of a perfect polyethylene crystal: a real semicrystalline grade melts over a range BELOW it, set by its crystallinity and lamella thickness, and that lower end is not sourced here. Its enthalpy of fusion depends on that unknown crystallinity, so the latent heat is not paid and the melt is refused",
        fusion_j_per_kg: None,
    },
    MeltingRange {
        species: "PP",
        lower_k: None,
        upper_k: 460.7,
        source: "U. Gaur and B. Wunderlich, 'Heat capacity and other thermodynamic properties of linear macromolecules. IV. Polypropylene', J. Phys. Chem. Ref. Data 10 (1981) 1051-1064, doi:10.1063/1.555650: best-estimate equilibrium melting temperature of isotactic polypropylene 460.7 K",
        boundary: "the equilibrium melting temperature of a perfect isotactic polypropylene crystal: a real grade melts over a range BELOW it, set by tacticity and crystallinity, and that lower end is not sourced here. Its enthalpy of fusion depends on that unknown crystallinity, so the latent heat is not paid and the melt is refused",
        fusion_j_per_kg: None,
    },
];

/// The sourced melting range of a material, where it has one.
pub fn range(species: &str) -> Option<&'static MeltingRange> {
    MELTING_RANGES.iter().find(|row| row.species == species)
}

/// The temperature above which a solid of this species must no longer be
/// shown as solid: its sourced melting point, or the top of its sourced
/// melting range.
pub fn melted_above_k(species: &str) -> Option<f64> {
    species::lookup(&SpeciesId::new(species))
        .and_then(|data| data.transitions)
        .and_then(|t| t.melting_k)
        .or_else(|| range(species).map(|row| row.upper_k))
}

/// A pair of materials whose melts are sourced as mutually immiscible.
///
/// Melting-point depression is a property of a SOLUTION: a mixture melts
/// below its lowest-melting component only where the components dissolve
/// in each other's melt (a solution or eutectic system). Components that
/// do not mix when molten each melt over their own range, as they would
/// alone, so the depression law must not be claimed for them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImmiscibleMelt {
    pub a: &'static str,
    pub b: &'static str,
    pub source: &'static str,
}

/// * Polyethylene / polypropylene: S. Teh, A. Rudin and J. C. Keung, 'A
///   review of polyethylene-polypropylene blends and their
///   compatibilization', Adv. Polym. Technol. 13 (1994) 1-23,
///   doi:10.1002/adv.1994.060130101 (abstract: "the incompatible nature of
///   polyethylenes and polypropylenes"; blends are phase-separated and
///   show the separate melting endotherms of both components).
pub const IMMISCIBLE_MELTS: &[ImmiscibleMelt] = &[ImmiscibleMelt {
    a: "PE",
    b: "PP",
    source: "S. Teh, A. Rudin and J. C. Keung, 'A review of polyethylene-polypropylene blends and their compatibilization', Adv. Polym. Technol. 13 (1994) 1-23, doi:10.1002/adv.1994.060130101",
}];

/// Whether two materials are sourced as immiscible in the melt.
pub fn immiscible(a: &str, b: &str) -> bool {
    IMMISCIBLE_MELTS
        .iter()
        .any(|row| (row.a == a && row.b == b) || (row.a == b && row.b == a))
}

/// Whether every pair of these components is sourced as immiscible in the
/// melt — the only case in which the melting-point apparatus drops the
/// depression law for a mixture. Fewer than two components is not a blend.
pub fn all_immiscible(components: &[&str]) -> bool {
    components.len() > 1
        && components
            .iter()
            .enumerate()
            .all(|(i, a)| components[i + 1..].iter().all(|b| immiscible(a, b)))
}

/// The enthalpy of fusion of a range material, J/mol, where it can be
/// paid: the row has a sourced specific enthalpy of fusion AND a sourced
/// lower bound to spread it from, and the registry knows the molar mass.
pub fn fusion_j_per_mol(species: &str) -> Option<f64> {
    let row = range(species)?;
    let per_kg = row.fusion_j_per_kg?;
    row.lower_k?;
    let molar_mass = species::lookup(&SpeciesId::new(species))?.molar_mass;
    (molar_mass > 0.0).then(|| per_kg * molar_mass / 1000.0)
}

/// The liquid fraction of a range material at `t_k`: 0 at or below the
/// lower bound, 1 at or above the upper, linear between.
pub fn melted_fraction(row: &MeltingRange, t_k: f64) -> f64 {
    let Some(lower) = row.lower_k else {
        return if t_k > row.upper_k { 1.0 } else { 0.0 };
    };
    ((t_k - lower) / (row.upper_k - lower)).clamp(0.0, 1.0)
}

const TRACE: f64 = 1e-12;

fn moles_in(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn withdraw(vessel: &mut Vessel, key: &str, phase: Phase, moles: f64) {
    let mut remaining = moles;
    for p in vessel.contents.iter_mut() {
        if p.species.0 == key && p.phase == phase && remaining > 0.0 {
            let take = p.moles.0.min(remaining);
            p.moles = crate::units::Moles(p.moles.0 - take);
            remaining -= take;
        }
    }
    vessel.contents.retain(|p| p.moles.0 > 1e-15);
}

/// Melt (or freeze) every range material with a payable enthalpy of
/// fusion to the state the vessel's energy allows, paying the latent heat
/// out of (or back into) the vessel's sensible heat.
///
/// The vessel arrives at `T0` with each material at liquid fraction `f0`
/// (whatever the last step left). The new temperature `T` solves
///
/// ```text
/// H_sensible(T0 -> T) + sum_i L_i n_i (f_i(T) - f0_i) = 0
/// ```
///
/// which is monotone increasing in `T` (every heat capacity is positive
/// and every `f_i` is non-decreasing), so it is bisected on the bracket
/// `[min(T0, lowest lower bound), max(T0, highest upper bound)]`, on which
/// it changes sign. The split is then set to `f_i(T)`.
///
/// The sensible term is integrated over the phases the vessel holds on
/// arrival; the solid/liquid heat-capacity difference over the moved
/// fraction is a second-order term this does not iterate on.
pub fn settle_range_melts(vessel: &mut Vessel) -> Vec<Event> {
    struct Melter {
        key: String,
        row: &'static MeltingRange,
        latent: f64,
        total: f64,
        liquid0: f64,
    }
    let mut melters: Vec<Melter> = Vec::new();
    for portion in &vessel.contents {
        if !matches!(portion.phase, Phase::Solid | Phase::Liquid) || portion.moles.0 <= TRACE {
            continue;
        }
        let key = portion.species.0.as_str();
        if melters.iter().any(|m| m.key == key) {
            continue;
        }
        let (Some(row), Some(latent)) = (range(key), fusion_j_per_mol(key)) else {
            continue;
        };
        let liquid0 = moles_in(vessel, key, Phase::Liquid);
        let total = moles_in(vessel, key, Phase::Solid) + liquid0;
        melters.push(Melter {
            key: key.to_string(),
            row,
            latent,
            total,
            liquid0,
        });
    }
    if melters.is_empty() || vessel.heat_capacity() <= 0.0 {
        return Vec::new();
    }
    let t0 = vessel.temperature.0;
    let balance = |t: f64| -> f64 {
        vessel.energy_between(t0, t)
            + melters
                .iter()
                .map(|m| m.latent * (m.total * melted_fraction(m.row, t) - m.liquid0))
                .sum::<f64>()
    };
    let at_t0 = balance(t0);
    // Already consistent: nothing to melt or freeze at this temperature.
    let scale: f64 = melters.iter().map(|m| m.latent * m.total).sum();
    if at_t0.abs() <= 1e-9 * scale.max(1.0) {
        return Vec::new();
    }
    let mut lo = melters
        .iter()
        .filter_map(|m| m.row.lower_k)
        .fold(t0, f64::min);
    let mut hi = melters.iter().map(|m| m.row.upper_k).fold(t0, f64::max);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if balance(mid) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
        if hi - lo <= 1e-10 {
            break;
        }
    }
    let t1 = (0.5 * (lo + hi)).max(0.0);

    let mut events = Vec::new();
    for m in &melters {
        let target = m.total * melted_fraction(m.row, t1);
        let delta = target - m.liquid0;
        let id = SpeciesId::new(&m.key);
        if delta > TRACE {
            let n = delta.min(moles_in(vessel, &m.key, Phase::Solid));
            withdraw(vessel, &m.key, Phase::Solid, n);
            vessel.deposit(id.clone(), crate::units::Moles(n), Phase::Liquid);
            events.push(Event::state_changed(
                vessel.id,
                id,
                Phase::Solid,
                Phase::Liquid,
                crate::units::Kelvin(t1),
                0.0,
                crate::units::Moles(n),
            ));
        } else if delta < -TRACE {
            let n = (-delta).min(moles_in(vessel, &m.key, Phase::Liquid));
            withdraw(vessel, &m.key, Phase::Liquid, n);
            vessel.deposit(id.clone(), crate::units::Moles(n), Phase::Solid);
            events.push(Event::state_changed(
                vessel.id,
                id,
                Phase::Liquid,
                Phase::Solid,
                crate::units::Kelvin(t1),
                0.0,
                crate::units::Moles(n),
            ));
        }
    }
    let from = vessel.temperature;
    if (t1 - from.0).abs() > 1e-9 {
        let to = crate::units::Kelvin(t1);
        vessel.temperature = to;
        vessel.refresh_pressure();
        events.push(Event::TemperatureChanged {
            vessel: vessel.id,
            from,
            to,
        });
    }
    events
}

/// One typed refusal per solid species standing above where it melts
/// without a latent heat to melt it with.
pub fn unmodelled_melts(vessel: &Vessel) -> Vec<Event> {
    let now = vessel.temperature.0;
    let mut seen: Vec<&str> = Vec::new();
    let mut events = Vec::new();
    for portion in &vessel.contents {
        if portion.phase != Phase::Solid || portion.moles.0 <= 1e-12 {
            continue;
        }
        let key = portion.species.0.as_str();
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        // A latent heat on the bench means the phase route melts it; water's
        // ice belongs to the solvent model (`solve::StateEquilibrator`).
        if key == "water"
            || crate::phase_route::fusion_enthalpy(key).is_some()
            || fusion_j_per_mol(key).is_some()
        {
            continue;
        }
        let Some(melts) = melted_above_k(key) else {
            continue;
        };
        if now <= melts {
            continue;
        }
        // A sharp melting point and a melting range say different things:
        // past a range, the thermometer has also skipped a latent heat
        // whose size this bench does not know.
        let has_point = species::lookup(&SpeciesId::new(key))
            .and_then(|data| data.transitions)
            .and_then(|t| t.melting_k)
            .is_some();
        let slots = vec![
            (
                "name".to_string(),
                Slot::term("species", species::english_name(key)),
            ),
            (
                "melting".to_string(),
                Slot::number(format!("{:.0}", melts - 273.15)),
            ),
        ];
        let phrase = if has_point {
            Phrase::new(
                "not-modeled.melting-not-modelled",
                "{name} heated above its melting point ({melting} °C): melting is not modelled here, so the solid shown is not a prediction that it stays solid",
                slots,
            )
        } else {
            Phrase::new(
                "not-modeled.melting-range-latent-heat-unsourced",
                "{name} heated past its melting range (up to {melting} °C): its enthalpy of fusion is not sourced, so the latent heat is not paid — the temperature shown overstates the warming, and the solid shown is not a prediction that it stays solid",
                slots,
            )
        };
        events.push(Event::not_modeled(
            vessel.id,
            NotModelledCause::NoReviewedDatum,
            phrase,
        ));
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_range_is_ordered_and_cited() {
        for row in MELTING_RANGES {
            assert!(
                row.lower_k.is_none_or(|low| low < row.upper_k),
                "{}",
                row.species
            );
            assert!(row.source.contains("doi"), "{} has no DOI", row.species);
            if let Some(per_kg) = row.fusion_j_per_kg {
                assert!(per_kg > 0.0, "{}", row.species);
                assert!(
                    row.source.contains("kJ/kg"),
                    "{}: latent heat uncited",
                    row.species
                );
            }
            assert!(
                species::lookup(&SpeciesId::new(row.species)).is_some(),
                "{} is not in the registry",
                row.species
            );
        }
    }

    #[test]
    fn every_immiscible_pair_is_cited_and_known() {
        for row in IMMISCIBLE_MELTS {
            assert!(row.source.contains("doi"), "{}/{}", row.a, row.b);
            assert!(range(row.a).is_some() || species::lookup(&SpeciesId::new(row.a)).is_some());
            assert!(range(row.b).is_some() || species::lookup(&SpeciesId::new(row.b)).is_some());
            assert!(immiscible(row.b, row.a), "symmetric");
        }
    }
}

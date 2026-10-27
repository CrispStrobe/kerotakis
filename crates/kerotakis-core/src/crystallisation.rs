//! Crystallisation heat on cooling: the solid a cooled solution owes, and
//! the heat it gives back, in the SAME energy ledger as the cooling.
//!
//! TH-007 is the case that put this here: 30 g KNO3 in 50 g water, cooled
//! by 12 kJ. Potassium nitrate's solubility falls about sixteen-fold
//! between 90 °C and 0 °C, so a cooled feed must deposit crystals, and
//! every mole that crystallises hands back its enthalpy of dissolution
//! (~35 kJ/mol). The bench instead moved the thermometer alone: no
//! crystals, and the 12 kJ bought a temperature drop the real beaker cannot
//! reach because crystallisation pays part of the bill.
//!
//! # The mechanism, once for every salt
//!
//! The PRIMARY solubility source is the routed database's own phase,
//! solved by PHREEQC in the aqueous step (mixed electrolytes and common
//! ions included); `kerotakis_phreeqc::crystallisation_route` enumerates
//! which registry solids that owns on which route. This module is the
//! coupled energy solve for a salt booked on its ions, and it runs at
//! runtime only against a curve that is explicitly `runtime_admitted`.
//! Measured curves are otherwise VALIDATION references, and a saturated
//! solution of such a salt is refused ([`Refusal::NoRoutedPhase`]).
//!
//! A salt is owned by this solve when, and only when, three facts hold:
//!
//! 1. an admitted, measured pure-water solubility CURVE ([`CURVES`]) with an
//!    explicit temperature domain;
//! 2. the dissolved salt is booked on its ions (the aqueous engine's
//!    ledger), so the amount in solution is the scarcer ion;
//! 3. a PRICED enthalpy of dissolution, supplied by the one pricer the
//!    routed aqueous engine registers ([`register_dissolution_pricer`]),
//!    falling back to the registry's curated `dissolution_enthalpy_kj`.
//!
//! The final state of a cooling dose is then the root `T_f` of
//!
//! ```text
//! Q_removed = H_sensible(T_f -> T_0, contents at T_f) + n(T_f) * dH_diss
//! n(T)      = max(0, dissolved - capacity(T))
//! ```
//!
//! which is monotone in `T_f` (capacity rises with temperature for every
//! curve shipped, and [`audit_curves`] fails otherwise), so bisection finds
//! the one root there is. An unsaturated solution has `n = 0` everywhere on
//! its path and cools exactly as before.
//!
//! # Domain, and what is refused
//!
//! * Below the lowest measured point of a curve the next physics is the
//!   salt–ice eutectic, which this module does not own: a dose that would
//!   carry the solution there is refused ([`Refusal::BelowCurveDomain`])
//!   rather than extrapolated.
//! * The curves are BINARY salt–water measurements. A solution with any
//!   other dissolved ion, or an unpaired excess of one of the salt's own
//!   ions (the common-ion case), is outside them ([`Refusal::NotBinary`]).
//! * A curve salt with no priced dissolution enthalpy on the route is
//!   refused ([`Refusal::Unpriced`]) — crystallising it without its heat
//!   would be exactly the defect this module exists to close.
//!
//! Solvent freezing is NOT duplicated here: brine freezing with its latent
//! heat and the 252 K admissibility cap is owned by
//! [`crate::solve::StateEquilibrator`] (PRs #852/#857).

use crate::{species, Moles, Phase, SpeciesId, Vessel};
use std::sync::OnceLock;

/// One measured point: °C, grams of anhydrous salt per 100 g of water.
pub type CurvePoint = (f64, f64);

/// A reviewed binary salt–water solubility curve.
#[derive(Debug, Clone, Copy)]
pub struct SolubilityCurve {
    /// Registry key of the anhydrous solid that crystallises.
    pub salt: &'static str,
    /// Ledger keys of the cation and anion it dissolves into, and their
    /// stoichiometric counts per formula unit.
    pub cation: (&'static str, f64),
    pub anion: (&'static str, f64),
    /// Measured points, strictly increasing in temperature.
    pub points: &'static [CurvePoint],
    pub source: &'static str,
    /// Whether the bench may crystallise against this curve at runtime.
    /// The primary solubility source is the routed database's own phase
    /// (`kerotakis_phreeqc::crystallisation_route`); a measured curve is a
    /// VALIDATION reference unless it is explicitly admitted. None is.
    pub runtime_admitted: bool,
}

/// Every reviewed curve. A row is a claim that the curve was READ from the
/// cited measurement; nothing here is fitted, smoothed or extrapolated.
pub const CURVES: &[SolubilityCurve] = &[SolubilityCurve {
    salt: "KNO3",
    cation: ("K+", 1.0),
    anion: ("NO3-", 1.0),
    // Column II ("Solubility", parts of salt per 100 parts of water) of
    // the potassium nitrate table. The boiling-point row (311.64) is not
    // carried: the paper gives no temperature for it.
    points: &[
        (0.40, 13.43),
        (14.90, 25.78),
        (30.80, 47.52),
        (44.75, 74.50),
        (60.05, 111.18),
        (76.00, 156.61),
        (91.65, 210.20),
    ],
    source: "Earl of Berkeley, 'On some physical constants of saturated solutions', \
             Phil. Trans. R. Soc. Lond. A 203 (1904) 189-215, doi:10.1098/rsta.1904.0020, \
             potassium nitrate table, column II (parts of salt per 100 parts water); read \
             2026-10-27 from the open scan https://archive.org/details/philosophicaltr203royauoft \
             (public domain, 1904). Each point is the mean of an approach from super- and \
             undersaturation. The scan's OCR prints the sixth temperature as '7G-00'; it is \
             read as 76.00 C, consistent with the neighbouring points and with the Comey \
             comparison column (159.0) printed beside it.",
    // No routed database (wateq4f, minteq.v4, pitzer) defines Niter, and
    // llnl.dat's Niter under the default route's Debye-Hueckel activities
    // misses this curve by 51-74% (tests/crystallisation_validation.rs in
    // kerotakis-phreeqc). So KNO3 crystallisation is refused at runtime.
    runtime_admitted: false,
}];

/// The reviewed curve for a registry salt, if one is shipped.
pub fn curve_for(salt: &str) -> Option<&'static SolubilityCurve> {
    CURVES.iter().find(|c| c.salt == salt)
}

impl SolubilityCurve {
    /// Lowest and highest measured temperature, K.
    pub fn domain_k(&self) -> (f64, f64) {
        let first = self.points.first().map(|p| p.0).unwrap_or(f64::NAN);
        let last = self.points.last().map(|p| p.0).unwrap_or(f64::NAN);
        (first + 273.15, last + 273.15)
    }

    /// g per 100 g water at `t_k`, linear between measured points, `None`
    /// outside the measured range.
    pub fn grams_per_100g_water(&self, t_k: f64) -> Option<f64> {
        let t_c = t_k - 273.15;
        let (lo, hi) = self.domain_k();
        if t_k.is_nan() || t_k < lo - 1e-9 || t_k > hi + 1e-9 {
            return None;
        }
        let t_c = t_c.clamp(lo - 273.15, hi - 273.15);
        self.points.windows(2).find_map(|w| {
            let ((t0, s0), (t1, s1)) = (w[0], w[1]);
            (t_c >= t0 && t_c <= t1).then(|| s0 + (s1 - s0) * (t_c - t0) / (t1 - t0))
        })
    }

    /// Moles of salt `water_g` grams of water hold at `t_k`.
    ///
    /// Above the measured range a solution holding no more than the
    /// highest measured point is certainly unsaturated (every shipped
    /// curve rises; [`audit_curves`]), so it answers that bound; inside the
    /// range it answers the curve; below it, `None`.
    pub fn capacity_mol(&self, water_g: f64, t_k: f64) -> Option<f64> {
        let molar_mass = species::lookup_key(self.salt)?.molar_mass;
        let (_, hi) = self.domain_k();
        let grams = if t_k > hi {
            self.points.last()?.1
        } else {
            self.grams_per_100g_water(t_k)?
        };
        Some(grams * water_g / 100.0 / molar_mass)
    }
}

/// Structural checks every curve must pass: at least two points, strictly
/// increasing temperature, strictly increasing solubility (the bisection's
/// monotonicity rests on it), finite positive values, a registry solid and
/// a charge-balanced ion pair. Returns the failures, empty when sound.
pub fn audit_curves() -> Vec<String> {
    let mut bad = Vec::new();
    for c in CURVES {
        if c.points.len() < 2 {
            bad.push(format!("{}: fewer than two points", c.salt));
        }
        for w in c.points.windows(2) {
            if w[1].0 <= w[0].0 {
                bad.push(format!("{}: temperatures not increasing", c.salt));
            }
            if w[1].1 <= w[0].1 {
                bad.push(format!("{}: solubility not increasing", c.salt));
            }
        }
        if c.points
            .iter()
            .any(|p| !(p.0.is_finite() && p.1.is_finite() && p.1 > 0.0))
        {
            bad.push(format!("{}: non-finite or non-positive point", c.salt));
        }
        if species::lookup_key(c.salt).is_none_or(|d| d.standard_phase != Phase::Solid) {
            bad.push(format!("{}: not a registry solid", c.salt));
        }
        let charge = |key: &str| {
            species::lookup_key(key)
                .and_then(|d| crate::stoich::parse_formula(d.formula).ok())
                .map(|f| f.charge)
        };
        match (charge(c.cation.0), charge(c.anion.0)) {
            (Some(qc), Some(qa)) if (qc * c.cation.1 + qa * c.anion.1).abs() < 1e-12 => {}
            _ => bad.push(format!("{}: ion pair not charge balanced", c.salt)),
        }
    }
    bad
}

/// The enthalpy of dissolution of a solid, kJ/mol (positive endothermic).
pub type DissolutionPricer = fn(&str) -> Option<f64>;

static PRICER: OnceLock<DissolutionPricer> = OnceLock::new();

/// Register the routed aqueous engine's dissolution pricer. The first
/// registration wins; later ones are ignored, so a process has one price
/// for a solid, not two.
pub fn register_dissolution_pricer(pricer: DissolutionPricer) {
    let _ = PRICER.set(pricer);
}

/// The enthalpy of dissolution this process prices a solid at, kJ/mol.
pub fn dissolution_enthalpy_kj(salt: &str) -> Option<f64> {
    PRICER
        .get()
        .and_then(|price| price(salt))
        .or_else(|| species::lookup_key(salt).and_then(|d| d.dissolution_enthalpy_kj))
        .filter(|h| h.is_finite())
}

/// Why a curve salt's crystallisation was not computed.
#[derive(Debug, Clone, PartialEq)]
pub enum Refusal {
    /// No priced enthalpy of dissolution on this route.
    Unpriced { salt: &'static str },
    /// The dose would carry the solution below the lowest measured point,
    /// toward the salt–ice eutectic this module does not own.
    BelowCurveDomain { salt: &'static str, floor_k: f64 },
    /// Other dissolved ions, or an unpaired excess of the salt's own: the
    /// binary pure-water curve does not describe this solution.
    NotBinary { salt: &'static str },
    /// Saturation is reached, but no routed database phase owns this
    /// salt's solubility and its measured curve is a validation reference
    /// only.
    NoRoutedPhase { salt: &'static str },
}

/// The owned answer to a cooling dose.
#[derive(Debug, Clone, PartialEq)]
pub struct Crystallised {
    pub salt: &'static str,
    pub final_k: f64,
    pub moles: f64,
    /// Heat released by crystallisation, J (positive).
    pub heat_j: f64,
}

/// Ions treated as solvent traces rather than solutes for the binary test.
const SOLVENT_TRACE_IONS: &[&str] = &["H+", "OH-"];
/// Moles below which another solute does not break the binary domain.
const BINARY_TRACE_MOLES: f64 = 1e-6;

fn aqueous(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Aqueous && p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn liquid_water_g(vessel: &Vessel) -> f64 {
    let mol: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Liquid && p.species.0 == "water")
        .map(|p| p.moles.0)
        .sum();
    mol * species::lookup_key("water").map_or(18.015, |d| d.molar_mass)
}

/// The curve salt this vessel's solution is a binary solution of, with the
/// amount dissolved, or `None` when no curve salt is dissolved here.
/// `Err` when one is dissolved but the solution is not binary.
pub fn binary_solution(
    vessel: &Vessel,
) -> Option<Result<(&'static SolubilityCurve, f64), Refusal>> {
    let curve = CURVES.iter().find(|c| {
        aqueous(vessel, c.cation.0) > BINARY_TRACE_MOLES
            && aqueous(vessel, c.anion.0) > BINARY_TRACE_MOLES
    })?;
    let units_c = aqueous(vessel, curve.cation.0) / curve.cation.1;
    let units_a = aqueous(vessel, curve.anion.0) / curve.anion.1;
    let paired = units_c.min(units_a);
    let unpaired = (units_c - units_a).abs();
    let others = vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Aqueous)
        .filter(|p| {
            p.species.0 != curve.cation.0
                && p.species.0 != curve.anion.0
                && !SOLVENT_TRACE_IONS.contains(&p.species.0.as_str())
        })
        .map(|p| p.moles.0)
        .sum::<f64>();
    if others > BINARY_TRACE_MOLES || unpaired > BINARY_TRACE_MOLES.max(paired * 1e-9) {
        return Some(Err(Refusal::NotBinary { salt: curve.salt }));
    }
    Some(Ok((curve, paired)))
}

/// Solve a cooling dose of `removed_j` from the vessel's current state.
///
/// `None` when no owned crystallisation is involved: no curve salt in
/// solution, or a binary solution that stays unsaturated all the way to
/// where pure sensible cooling would put it (the caller then cools exactly
/// as it always did). `Some(Ok)` is the owned final state; `Some(Err)` a
/// typed refusal.
///
/// `sensible_after(t_f, n)` must return the heat, J (positive), that the
/// vessel's contents give up between `t_f` and their start temperature
/// once `n` moles of the salt have left solution as solid.
pub fn cool_with_crystallisation(
    vessel: &Vessel,
    removed_j: f64,
    sensible_after: &dyn Fn(f64, f64) -> f64,
) -> Option<Result<Crystallised, Refusal>> {
    let (curve, dissolved) = match binary_solution(vessel)? {
        Ok(found) => found,
        Err(refusal) => {
            // Only a refusal if crystallisation could actually be reached:
            // a non-binary solution far below every capacity is still just
            // a solution cooling, and saying nothing new there keeps the
            // unsaturated path byte-identical.
            return saturation_reachable(vessel, removed_j, None).then_some(Err(refusal));
        }
    };
    if !saturation_reachable(vessel, removed_j, Some((curve, dissolved))) {
        return None;
    }
    if !curve.runtime_admitted {
        return Some(Err(Refusal::NoRoutedPhase { salt: curve.salt }));
    }
    Some(solve(vessel, curve, dissolved, removed_j, sensible_after))
}

/// The coupled energy/crystallisation solve against an explicit curve,
/// admitted or not: the mechanism the runtime uses, exposed so validation
/// fixtures can exercise it against a measured reference.
pub fn cool_against_curve(
    vessel: &Vessel,
    curve: &'static SolubilityCurve,
    removed_j: f64,
    sensible_after: &dyn Fn(f64, f64) -> f64,
) -> Result<Crystallised, Refusal> {
    let (found, dissolved) =
        binary_solution(vessel).ok_or(Refusal::NotBinary { salt: curve.salt })??;
    if found.salt != curve.salt {
        return Err(Refusal::NotBinary { salt: curve.salt });
    }
    solve(vessel, curve, dissolved, removed_j, sensible_after)
}

/// Does the pure-sensible end point (no crystallisation) lie at or below
/// the saturation temperature of a curve salt?
fn saturation_reachable(
    vessel: &Vessel,
    removed_j: f64,
    found: Option<(&'static SolubilityCurve, f64)>,
) -> bool {
    let water_g = liquid_water_g(vessel);
    let pure_end = vessel.temperature_after(-removed_j);
    let (curve, dissolved) = match found {
        Some(found) => found,
        None => match CURVES
            .iter()
            .find(|c| aqueous(vessel, c.cation.0) > BINARY_TRACE_MOLES)
        {
            Some(c) => {
                let n = (aqueous(vessel, c.cation.0) / c.cation.1)
                    .min(aqueous(vessel, c.anion.0) / c.anion.1);
                (c, n)
            }
            None => return false,
        },
    };
    match curve.capacity_mol(water_g, pure_end.max(0.0)) {
        Some(capacity) => dissolved > capacity,
        // Below the measured range: whether the solution would be
        // saturated there is exactly what the curve cannot say.
        None => true,
    }
}

fn solve(
    vessel: &Vessel,
    curve: &'static SolubilityCurve,
    dissolved: f64,
    removed_j: f64,
    sensible_after: &dyn Fn(f64, f64) -> f64,
) -> Result<Crystallised, Refusal> {
    let dh_kj =
        dissolution_enthalpy_kj(curve.salt).ok_or(Refusal::Unpriced { salt: curve.salt })?;
    let water_g = liquid_water_g(vessel);
    let t0 = vessel.temperature.0;
    let (floor_k, _) = curve.domain_k();
    let crystallised =
        |t: f64| -> Option<f64> { Some((dissolved - curve.capacity_mol(water_g, t)?).max(0.0)) };
    // Heat given up on the way to `t`, crystallisation included, less the
    // dose: increasing as `t` falls.
    let balance = |t: f64| -> Option<f64> {
        let n = crystallised(t)?;
        Some(sensible_after(t, n) + n * dh_kj * 1000.0 - removed_j)
    };
    let below = balance(floor_k).ok_or(Refusal::BelowCurveDomain {
        salt: curve.salt,
        floor_k,
    })?;
    if below < 0.0 {
        return Err(Refusal::BelowCurveDomain {
            salt: curve.salt,
            floor_k,
        });
    }
    let (mut lo, mut hi) = (floor_k, t0);
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        // `balance` is positive below the root and negative above it.
        match balance(mid) {
            Some(b) if b > 0.0 => lo = mid,
            _ => hi = mid,
        }
        if hi - lo < 1e-10 {
            break;
        }
    }
    let final_k = 0.5 * (lo + hi);
    let moles = crystallised(final_k).unwrap_or(0.0);
    Ok(Crystallised {
        salt: curve.salt,
        final_k,
        moles,
        heat_j: moles * dh_kj * 1000.0,
    })
}

/// Move `moles` of a curve salt out of solution onto the solid, keeping
/// every element and the charge balance.
pub fn apply(vessel: &mut Vessel, curve: &SolubilityCurve, moles: f64) {
    for (ion, count) in [curve.cation, curve.anion] {
        vessel.withdraw_phase(&SpeciesId::new(ion), Moles(moles * count), Phase::Aqueous);
    }
    vessel.deposit(SpeciesId::new(curve.salt), Moles(moles), Phase::Solid);
}

/// Is a SOLID portion of this curve salt held by this ledger rather than
/// handed to an aqueous engine to dissolve? Yes exactly when the solution
/// is already saturated with it at the vessel's temperature: re-dissolving
/// such crystals in the next aqueous step would undo the crystallisation
/// (and, on a priced route, charge its heat back).
pub fn holds_solid(vessel: &Vessel, salt: &str) -> bool {
    let Some(curve) = curve_for(salt).filter(|c| c.runtime_admitted) else {
        return false;
    };
    let Some(Ok((found, dissolved))) = binary_solution(vessel) else {
        return false;
    };
    if found.salt != curve.salt {
        return false;
    }
    curve
        .capacity_mol(liquid_water_g(vessel), vessel.temperature.0)
        .is_some_and(|capacity| dissolved >= capacity * (1.0 - 1e-9))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_curve_passes_its_structural_audit() {
        assert!(audit_curves().is_empty(), "{:?}", audit_curves());
    }

    #[test]
    fn kno3_curve_reads_the_measured_points_and_refuses_outside_them() {
        let c = curve_for("KNO3").unwrap();
        assert!((c.grams_per_100g_water(273.15 + 30.80).unwrap() - 47.52).abs() < 1e-9);
        assert!((c.grams_per_100g_water(273.15 + 91.65).unwrap() - 210.20).abs() < 1e-9);
        assert!(c.grams_per_100g_water(273.15).is_none());
        assert!(c.grams_per_100g_water(273.15 + 95.0).is_none());
        assert!(curve_for("NaCl").is_none(), "halite is the routed engine's");
    }
}

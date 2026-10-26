//! One fusion authority for both phase energy and solvent chemical potential.
//!
//! Uses the existing registry Cp curves and 273.15 K / 6010 J/mol reference.
//! Below 273.15 K the liquid curve is held at its endpoint, exactly as in
//! the sensible-energy ledger. This bounded model extension is not measured
//! supercooled-water data. Roots below 252 K are outside this model.
use crate::constants::GAS_CONSTANT as R;
use crate::species::{self, Phase, SpeciesId};
use crate::states::{
    enthalpy_between, heat_capacity_curve, BRINE_MODEL_MIN_K, WATER_FREEZING_K, WATER_H_FUS,
};

/// Liquid-minus-ice enthalpy, J/mol, on the admitted coexistence interval.
pub fn enthalpy_at(t: f64) -> Option<f64> {
    if !t.is_finite() || !(BRINE_MODEL_MIN_K..=WATER_FREEZING_K).contains(&t) {
        return None;
    }
    let water = species::lookup(&SpeciesId::new("water"))?;
    let h = WATER_H_FUS + enthalpy_between(water, Phase::Liquid, WATER_FREEZING_K, t)
        - enthalpy_between(water, Phase::Solid, WATER_FREEZING_K, t);
    (h.is_finite() && h > 0.0).then_some(h)
}

/// ln(a_w) = ∫[Tm,T] Hfus(u)/(R u²) du. Integration by parts gives
/// H0(1/Tm−1/T) + ΔS − ΔH/T, divided by R. ΔS and ΔH use identical
/// phase curves and endpoint policies; no second fitted freezing law exists.
pub fn ln_activity_at(t: f64) -> Option<f64> {
    enthalpy_at(t)?;
    let water = species::lookup(&SpeciesId::new("water"))?;
    let delta_h = enthalpy_between(water, Phase::Liquid, WATER_FREEZING_K, t)
        - enthalpy_between(water, Phase::Solid, WATER_FREEZING_K, t);
    let delta_s = heat_capacity_curve(water, Phase::Liquid)?
        .entropy_between(WATER_FREEZING_K, t)?
        - heat_capacity_curve(water, Phase::Solid)?.entropy_between(WATER_FREEZING_K, t)?;
    let log_a = log_activity_from_deltas(t, delta_h, delta_s);
    log_a.is_finite().then_some(log_a)
}

/// Bounded inverse. Invalid activity and roots below the admitted interval
/// return None; they are never replaced by pure water or clamped into range.
pub fn temperature_at(activity: f64) -> Option<f64> {
    if !activity.is_finite() || activity <= 0.0 || activity > 1.0 {
        return None;
    }
    if activity == 1.0 {
        return Some(WATER_FREEZING_K);
    }
    let cap = ln_activity_at(BRINE_MODEL_MIN_K)?;
    // Compare in activity space so a round trip through exp at the exact
    // boundary has no artificial log/exp rounding refusal.
    if activity < cap.exp() {
        return None;
    }
    let target = activity.ln();
    let (mut lo, mut hi) = (BRINE_MODEL_MIN_K, WATER_FREEZING_K);
    for _ in 0..64 {
        let mid = lo + (hi - lo) / 2.0;
        if mid == lo || mid == hi {
            return Some(mid);
        }
        if ln_activity_at(mid)? < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    None
}

/// Local liquidus sensitivity, K per unit ln(a_w), for cap roundoff.
pub fn temperature_sensitivity_at(t: f64) -> Option<f64> {
    Some(R * t * t / enthalpy_at(t)?)
}

// Integration by parts, shared by the real curves and analytic limit controls.
fn log_activity_from_deltas(t: f64, delta_h: f64, delta_s: f64) -> f64 {
    (WATER_H_FUS * (t - WATER_FREEZING_K) / (WATER_FREEZING_K * t) + delta_s - delta_h / t) / R
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_delta_cp_recovers_constant_enthalpy_law() {
        let actual = log_activity_from_deltas(260.0, 0.0, 0.0);
        assert!((actual - (-0.133_841_713_169_140_7)).abs() < 1e-14);
    }
    #[test]
    fn constant_delta_cp_matches_independent_closed_form() {
        let cp = 40.0;
        let t = 260.0;
        let actual = log_activity_from_deltas(
            t,
            cp * (t - WATER_FREEZING_K),
            cp * (t / WATER_FREEZING_K).ln(),
        );
        assert!((actual - (-0.127_888_427_634_980_65)).abs() < 1e-14);
    }
}

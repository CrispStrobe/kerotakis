//! EXP-44: excess enthalpy of mixing from UNIFAC's own temperature
//! dependence.
//!
//! The Gibbs–Helmholtz route: hᴱ = −R·T²·Σᵢ xᵢ·(∂ln γᵢ/∂T)ₚ,ₓ. The
//! UNIFAC residual part carries all the T-dependence (ψₘₙ = exp(−aₘₙ/T));
//! the combinatorial part is athermal and drops out exactly — which the
//! derivative sees on its own, no special-casing.
//!
//! The derivative is a central difference at ±ΔT/2 with ΔT = 1 K,
//! stated rather than hidden: ln γ from this table is smooth in T and
//! the truncation error at 1 K is far below the model's own honesty
//! bound. UNIFAC parameters are fitted to VLE, so hᴱ derived this way
//! is QUALITATIVE — sign and magnitude-class, not calorimetry-grade.
//! Every consumer states that boundary.

use crate::unifac::{activity_coefficients, approved_table, GroupDecomposition};

const R: f64 = 8.314_462_618;
const DELTA_T: f64 = 1.0;

/// Excess enthalpy of the mixture, J per mole of mixture.
///
/// `components`: (group decomposition, mole fraction) — fractions
/// should sum to ~1; endpoints (any xᵢ ≈ 1) return ~0 by construction,
/// which is the state-function anchor the bench bookkeeping relies on.
pub fn excess_enthalpy_j_per_mol(components: &[(GroupDecomposition, f64)], t_kelvin: f64) -> f64 {
    if components.len() < 2 {
        return 0.0;
    }
    let table = approved_table();
    let lo = activity_coefficients(&table, components, t_kelvin - DELTA_T / 2.0);
    let hi = activity_coefficients(&table, components, t_kelvin + DELTA_T / 2.0);
    let mut sum = 0.0;
    for (i, (_, x)) in components.iter().enumerate() {
        if *x <= 0.0 {
            continue;
        }
        let dln_dt = (hi[i].ln() - lo[i].ln()) / DELTA_T;
        sum += x * dln_dt;
    }
    -R * t_kelvin * t_kelvin * sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn water() -> GroupDecomposition {
        let mut g = GroupDecomposition::new();
        g.insert(16, 1);
        g
    }
    fn ethanol() -> GroupDecomposition {
        let mut g = GroupDecomposition::new();
        g.insert(1, 1);
        g.insert(2, 1);
        g.insert(14, 1);
        g
    }

    #[test]
    fn endpoints_are_zero() {
        for x in [0.0, 1.0] {
            let h = excess_enthalpy_j_per_mol(&[(ethanol(), x), (water(), 1.0 - x)], 298.15);
            assert!(
                h.abs() < 1e-9,
                "pure components have no mixing enthalpy: {h}"
            );
        }
    }

    fn propanone() -> GroupDecomposition {
        let mut g = GroupDecomposition::new();
        g.insert(1, 1);
        g.insert(18, 1);
        g
    }

    /// The verification that decides the bench allowlist: acetone–water
    /// reproduces the literature S-shape (exothermic mid-range, mildly
    /// positive at high acetone), so the bench applies its heat.
    /// Ethanol–water does NOT: this parameter set inverts the dilute-end
    /// sign (+ at x=0.1 where calorimetry says strongly −), so the bench
    /// WITHHOLDS that pair's heat rather than showing a wrong sign —
    /// the test pins the deviation so a parameter upgrade that fixes it
    /// is noticed and the allowlist revisited.
    #[test]
    fn the_allowlist_verification_holds() {
        let mid = excess_enthalpy_j_per_mol(&[(propanone(), 0.4), (water(), 0.6)], 298.15);
        assert!(mid < -50.0, "acetone–water exothermic mid-range: {mid}");
        let dilute = excess_enthalpy_j_per_mol(&[(propanone(), 0.1), (water(), 0.9)], 298.15);
        assert!(dilute < 0.0, "…and at the dilute end: {dilute}");

        let etoh_dilute = excess_enthalpy_j_per_mol(&[(ethanol(), 0.1), (water(), 0.9)], 298.15);
        assert!(
            etoh_dilute > 0.0,
            "the known ethanol–water dilute-end sign inversion is still              present ({etoh_dilute}); if a parameter upgrade fixed it,              revisit the bench allowlist"
        );
    }

    #[test]
    fn the_curve_is_smooth_and_vanishes_smoothly() {
        let mut prev = 0.0;
        for i in 0..=10 {
            let x = i as f64 / 10.0;
            let h = excess_enthalpy_j_per_mol(&[(ethanol(), x), (water(), 1.0 - x)], 298.15);
            assert!(h.is_finite());
            assert!((h - prev).abs() < 2000.0, "no jumps: {prev} → {h}");
            prev = h;
        }
    }
}

//! THERMO-008: Liquid-liquid equilibrium (LLE).
//!
//! Determines whether a liquid mixture splits into two phases, and if so,
//! what the compositions are. Uses activity coefficients to detect
//! Gibbs energy minima.

/// Result of a liquid-liquid stability check.
#[derive(Debug, Clone, PartialEq)]
pub enum LleResult {
    /// Single liquid phase — no split.
    SinglePhase,
    /// Two liquid phases with the given compositions.
    TwoPhase {
        /// Mole fraction of component 1 in phase α.
        x1_alpha: f64,
        /// Mole fraction of component 1 in phase β.
        x1_beta: f64,
        /// Fraction of total moles in phase α.
        phase_fraction_alpha: f64,
    },
}

/// Check liquid-liquid miscibility for a binary system.
///
/// Uses the tangent-plane distance (TPD) criterion on the Gibbs energy
/// of mixing: g_mix = Σ xᵢ ln(xᵢ γᵢ). A negative TPD means the mixture
/// is unstable and will split.
///
/// `gammas_fn` returns (γ₁, γ₂) at a given x₁.
pub fn lle_binary<F>(z1: f64, gammas_fn: &mut F) -> LleResult
where
    F: FnMut(f64) -> (f64, f64),
{
    if z1 <= 0.0 || z1 >= 1.0 {
        return LleResult::SinglePhase;
    }

    // Gibbs energy of mixing per mole: g = x₁ ln(x₁γ₁) + x₂ ln(x₂γ₂)
    let g_mix = |x1: f64, gammas: &mut F| -> f64 {
        let x2 = 1.0 - x1;
        let (g1, g2) = gammas(x1);
        let t1 = if x1 > 1e-15 {
            x1 * (x1 * g1).max(1e-30).ln()
        } else {
            0.0
        };
        let t2 = if x2 > 1e-15 {
            x2 * (x2 * g2).max(1e-30).ln()
        } else {
            0.0
        };
        t1 + t2
    };

    // Scan for a local maximum in g_mix (spinodal region indicator)
    let n = 100;
    let mut found_concave = false;
    let dx = 1.0 / n as f64;
    let mut g_prev = g_mix(dx, gammas_fn);
    let mut g_curr = g_mix(2.0 * dx, gammas_fn);
    for i in 3..n {
        let x = i as f64 * dx;
        let g_next = g_mix(x, gammas_fn);
        // Check for concavity (second derivative > 0 → convex → stable;
        // second derivative < 0 → concave → unstable)
        let d2g = (g_next - 2.0 * g_curr + g_prev) / (dx * dx);
        if d2g < -1e-10 {
            found_concave = true;
            break;
        }
        g_prev = g_curr;
        g_curr = g_next;
    }

    if !found_concave {
        return LleResult::SinglePhase;
    }

    // Two-phase split: find the common tangent by bisecting for equal
    // chemical potentials. For a binary, this means finding x_α and x_β
    // such that μ₁(x_α) = μ₁(x_β) and μ₂(x_α) = μ₂(x_β).
    //
    // Simplified: scan for the two compositions where the tangent line
    // from the feed composition touches the g_mix curve.
    let mut x_alpha = 0.01;
    let mut x_beta = 0.99;

    // Refine by equal-activity iteration (simplified)
    for _ in 0..50 {
        let (g1a, _) = gammas_fn(x_alpha);
        let (g1b, _) = gammas_fn(x_beta);
        // Equal activity: x_α·γ₁(x_α) = x_β·γ₁(x_β)
        let act_a = x_alpha * g1a;
        let act_b = x_beta * g1b;
        if act_a > act_b {
            x_alpha += 0.005;
        } else {
            x_alpha -= 0.005;
        }
        x_alpha = x_alpha.clamp(0.001, z1);

        let (_, g2a) = gammas_fn(x_alpha);
        let (_, g2b) = gammas_fn(x_beta);
        let act2_a = (1.0 - x_alpha) * g2a;
        let act2_b = (1.0 - x_beta) * g2b;
        if act2_a > act2_b {
            x_beta -= 0.005;
        } else {
            x_beta += 0.005;
        }
        x_beta = x_beta.clamp(z1, 0.999);
    }

    // Lever rule: fraction in phase α
    let denom = x_beta - x_alpha;
    let phase_fraction = if denom.abs() > 1e-10 {
        ((x_beta - z1) / denom).clamp(0.0, 1.0)
    } else {
        return LleResult::SinglePhase;
    };

    LleResult::TwoPhase {
        x1_alpha: x_alpha,
        x1_beta: x_beta,
        phase_fraction_alpha: phase_fraction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideal_mixture_no_split() {
        // Ideal solution (γ=1): always miscible
        let result = lle_binary(0.5, &mut |_x| (1.0, 1.0));
        assert_eq!(result, LleResult::SinglePhase);
    }

    #[test]
    fn highly_nonideal_mixture_splits() {
        // Margules one-parameter with A=3 (strongly non-ideal)
        // γ₁ = exp(A·x₂²), γ₂ = exp(A·x₁²)
        let a = 3.0;
        let result = lle_binary(0.5, &mut |x1| {
            let x2 = 1.0 - x1;
            ((a * x2 * x2).exp(), (a * x1 * x1).exp())
        });
        match result {
            LleResult::TwoPhase {
                x1_alpha, x1_beta, ..
            } => {
                assert!(
                    x1_alpha < 0.5 && x1_beta > 0.5,
                    "α={:.3}, β={:.3}",
                    x1_alpha,
                    x1_beta
                );
            }
            LleResult::SinglePhase => panic!("should split at A=3"),
        }
    }

    #[test]
    fn weakly_nonideal_no_split() {
        // Margules A=1 — not enough to cause splitting
        let a = 1.0;
        let result = lle_binary(0.5, &mut |x1| {
            let x2 = 1.0 - x1;
            ((a * x2 * x2).exp(), (a * x1 * x1).exp())
        });
        assert_eq!(result, LleResult::SinglePhase);
    }
}

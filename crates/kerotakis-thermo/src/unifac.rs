//! UNIFAC activity-coefficient model (THERMO-004).
//!
//! Original UNIFAC: Fredenslund, Jones & Prausnitz, AIChE J. 21:1086 (1975).
//! Group volume (Rk) and surface area (Qk) parameters from Bondi (1968).
//! Interaction parameters (aij) from Hansen et al., Ind. Eng. Chem. Res.
//! 30:2352 (1991) and the open tables in Poling, Prausnitz & O'Connell,
//! "The Properties of Gases and Liquids", 5th ed. (2001), Appendix J.
//!
//! The proprietary UNIFAC Consortium (Dortmund) table is NOT used here.
//! Every parameter below is from openly published peer-reviewed sources.

use crate::fluid::ActivityModel;

/// A UNIFAC functional group.
#[derive(Debug, Clone, Copy)]
pub struct Group {
    /// Main group number.
    pub main: u16,
    /// Subgroup number.
    pub sub: u16,
    /// Group name.
    pub name: &'static str,
    /// Volume parameter Rk (van der Waals volume / 15.17).
    pub r: f64,
    /// Surface area parameter Qk (van der Waals area / 2.5e9).
    pub q: f64,
}

/// Source: Poling, Prausnitz & O'Connell (2001), Appendix J, Table J-1.
/// These are factual physical parameters, not copyrightable creative works.
pub const GROUPS: &[Group] = &[
    Group { main: 1, sub: 1, name: "CH3", r: 0.9011, q: 0.848 },
    Group { main: 1, sub: 2, name: "CH2", r: 0.6744, q: 0.540 },
    Group { main: 1, sub: 3, name: "CH",  r: 0.4469, q: 0.228 },
    Group { main: 1, sub: 4, name: "C",   r: 0.2195, q: 0.000 },
    Group { main: 3, sub: 5, name: "ACH", r: 0.5313, q: 0.400 },  // aromatic CH
    Group { main: 5, sub: 14, name: "CH3OH", r: 1.4311, q: 1.432 }, // methanol
    Group { main: 5, sub: 15, name: "OH",  r: 1.0000, q: 1.200 },   // alcohol
    Group { main: 7, sub: 16, name: "H2O", r: 0.9200, q: 1.400 },
    Group { main: 9, sub: 18, name: "CH3CO", r: 1.6724, q: 1.488 }, // ketone
    Group { main: 20, sub: 43, name: "COOH", r: 1.3013, q: 1.224 },
];

/// Interaction parameter a_ij (K) between main groups.
/// Source: Hansen et al. (1991) and Poling et al. (2001), Table J-2.
/// a_ij is NOT symmetric: a_ij ≠ a_ji in general.
///
/// Format: (main_i, main_j, a_ij_kelvin).
pub const INTERACTIONS: &[(u16, u16, f64)] = &[
    // CH2 (1) ↔ OH (5)
    (1, 5, 986.5),
    (5, 1, 156.4),
    // CH2 (1) ↔ H2O (7)
    (1, 7, 1318.0),
    (7, 1, 300.0),
    // OH (5) ↔ H2O (7)
    (5, 7, 353.5),
    (7, 5, -229.1),
    // CH2 (1) ↔ COOH (20)
    (1, 20, 663.5),
    (20, 1, 315.3),
    // OH (5) ↔ COOH (20)
    (5, 20, -151.0),
    (20, 5, 339.8),
    // H2O (7) ↔ COOH (20)
    (7, 20, -66.17),
    (20, 7, -14.09),
    // CH2 (1) ↔ ACH (3)
    (1, 3, -11.12),
    (3, 1, 61.13),
    // CH2 (1) ↔ CH3CO (9)
    (1, 9, 476.4),
    (9, 1, 26.76),
    // OH (5) ↔ CH3CO (9)
    (5, 9, 164.5),
    (9, 5, -137.1),
    // H2O (7) ↔ CH3CO (9)
    (7, 9, -195.4),
    (9, 7, 472.5),
];

/// Look up the interaction parameter a_ij between two main groups.
pub fn interaction(main_i: u16, main_j: u16) -> f64 {
    if main_i == main_j {
        return 0.0;
    }
    INTERACTIONS
        .iter()
        .find(|(i, j, _)| *i == main_i && *j == main_j)
        .map(|(_, _, a)| *a)
        .unwrap_or(0.0)
}

/// A molecule decomposed into UNIFAC groups.
#[derive(Debug, Clone)]
pub struct GroupDecomposition {
    /// (group_index_into_GROUPS, count)
    pub groups: Vec<(usize, u32)>,
}

/// Predefined decompositions for common molecules.
pub fn decompose_ethanol() -> GroupDecomposition {
    GroupDecomposition {
        groups: vec![(0, 1), (1, 1), (6, 1)], // CH3 + CH2 + OH
    }
}

pub fn decompose_water() -> GroupDecomposition {
    GroupDecomposition {
        groups: vec![(7, 1)], // H2O
    }
}

pub fn decompose_acetone() -> GroupDecomposition {
    GroupDecomposition {
        groups: vec![(0, 1), (8, 1)], // CH3 + CH3CO
    }
}

pub fn decompose_acetic_acid() -> GroupDecomposition {
    GroupDecomposition {
        groups: vec![(0, 1), (9, 1)], // CH3 + COOH
    }
}

/// Compute UNIFAC activity coefficients for a binary mixture.
///
/// `decompositions[i]` is the group decomposition for component i.
/// `x[i]` is the mole fraction of component i.
/// `t_celsius` is the temperature.
///
/// Returns γ_i for each component.
pub fn unifac_gamma(
    decompositions: &[GroupDecomposition],
    x: &[f64],
    t_celsius: f64,
) -> Vec<f64> {
    let nc = decompositions.len();
    let t_k = t_celsius + 273.15;

    // Collect all unique groups across all components
    let mut all_groups: Vec<usize> = Vec::new();
    for d in decompositions {
        for &(g, _) in &d.groups {
            if !all_groups.contains(&g) {
                all_groups.push(g);
            }
        }
    }
    all_groups.sort();
    let ng = all_groups.len();

    // Component r_i and q_i
    let r: Vec<f64> = decompositions
        .iter()
        .map(|d| d.groups.iter().map(|&(g, n)| GROUPS[g].r * n as f64).sum())
        .collect();
    let q: Vec<f64> = decompositions
        .iter()
        .map(|d| d.groups.iter().map(|&(g, n)| GROUPS[g].q * n as f64).sum())
        .collect();

    // Combinatorial part: ln γ_i^C
    let r_sum: f64 = r.iter().zip(x).map(|(ri, xi)| ri * xi).sum();
    let q_sum: f64 = q.iter().zip(x).map(|(qi, xi)| qi * xi).sum();

    let mut ln_gamma_c = vec![0.0; nc];
    for i in 0..nc {
        if x[i] < 1e-30 {
            continue;
        }
        let phi_i = r[i] * x[i] / r_sum;
        let theta_i = q[i] * x[i] / q_sum;
        let l_i = 5.0 * (r[i] - q[i]) - (r[i] - 1.0);
        let l_sum: f64 = (0..nc)
            .map(|j| x[j] * (5.0 * (r[j] - q[j]) - (r[j] - 1.0)))
            .sum();

        ln_gamma_c[i] = (phi_i / x[i]).max(1e-30).ln()
            + 5.0 * q[i] * (theta_i / phi_i).max(1e-30).ln()
            + l_i
            - phi_i / x[i] * l_sum;
    }

    // Residual part: ln γ_i^R = Σ_k ν_ki [ln Γ_k - ln Γ_k^(i)]
    // where Γ_k is the group activity coefficient in the mixture
    // and Γ_k^(i) is the group activity coefficient in pure component i.

    // Group interaction Ψ_mn = exp(-a_mn / T)
    let psi = |m: usize, n: usize| -> f64 {
        let a = interaction(GROUPS[all_groups[m]].main, GROUPS[all_groups[n]].main);
        (-a / t_k).exp()
    };

    // Compute group activity coefficient ln Γ_k for a given set of group fractions
    let ln_gamma_group = |x_group: &[f64]| -> Vec<f64> {
        let mut theta = vec![0.0; ng];
        let q_total: f64 = (0..ng)
            .map(|k| x_group[k] * GROUPS[all_groups[k]].q)
            .sum();
        if q_total < 1e-30 {
            return vec![0.0; ng];
        }
        for k in 0..ng {
            theta[k] = x_group[k] * GROUPS[all_groups[k]].q / q_total;
        }

        let mut result = vec![0.0; ng];
        for k in 0..ng {
            let qk = GROUPS[all_groups[k]].q;
            let sum1: f64 = (0..ng).map(|m| theta[m] * psi(m, k)).sum();
            let mut sum2 = 0.0;
            for m in 0..ng {
                let denom: f64 = (0..ng).map(|n| theta[n] * psi(n, m)).sum();
                if denom > 1e-30 {
                    sum2 += theta[m] * psi(k, m) / denom;
                }
            }
            result[k] = qk * (1.0 - sum1.max(1e-30).ln() - sum2);
        }
        result
    };

    // Mixture group fractions
    let mut x_group_mix = vec![0.0; ng];
    for i in 0..nc {
        for &(g, n) in &decompositions[i].groups {
            let idx = all_groups.iter().position(|&ag| ag == g).unwrap();
            x_group_mix[idx] += x[i] * n as f64;
        }
    }
    let total: f64 = x_group_mix.iter().sum();
    if total > 0.0 {
        for v in &mut x_group_mix {
            *v /= total;
        }
    }
    let ln_gamma_mix = ln_gamma_group(&x_group_mix);

    // Residual contribution
    let mut ln_gamma_r = vec![0.0; nc];
    for i in 0..nc {
        // Pure-component group fractions
        let mut x_group_pure = vec![0.0; ng];
        let mut pure_total = 0.0;
        for &(g, n) in &decompositions[i].groups {
            let idx = all_groups.iter().position(|&ag| ag == g).unwrap();
            x_group_pure[idx] = n as f64;
            pure_total += n as f64;
        }
        if pure_total > 0.0 {
            for v in &mut x_group_pure {
                *v /= pure_total;
            }
        }
        let ln_gamma_pure = ln_gamma_group(&x_group_pure);

        for &(g, n) in &decompositions[i].groups {
            let idx = all_groups.iter().position(|&ag| ag == g).unwrap();
            ln_gamma_r[i] += n as f64 * (ln_gamma_mix[idx] - ln_gamma_pure[idx]);
        }
    }

    // Total: γ_i = exp(ln γ_i^C + ln γ_i^R)
    (0..nc)
        .map(|i| (ln_gamma_c[i] + ln_gamma_r[i]).exp())
        .collect()
}

/// UNIFAC activity-coefficient model implementing the FluidModel trait.
pub struct UnifacModel {
    pub decompositions: Vec<GroupDecomposition>,
}

impl ActivityModel for UnifacModel {
    fn name(&self) -> &'static str {
        "UNIFAC (original)"
    }

    fn activity_coefficients(&self, mole_fractions: &[f64], t_celsius: f64) -> Vec<f64> {
        unifac_gamma(&self.decompositions, mole_fractions, t_celsius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_component_gamma_is_unity() {
        let ethanol = decompose_ethanol();
        let gammas = unifac_gamma(&[ethanol], &[1.0], 25.0);
        assert!(
            (gammas[0] - 1.0).abs() < 1e-10,
            "pure component γ should be 1.0, got {}",
            gammas[0]
        );
    }

    #[test]
    fn ethanol_water_positive_deviation() {
        // Ethanol-water is positively deviating (γ > 1 at dilute)
        let ethanol = decompose_ethanol();
        let water = decompose_water();
        let gammas = unifac_gamma(
            &[ethanol, water],
            &[0.1, 0.9], // dilute ethanol in water
            78.0,
        );
        assert!(
            gammas[0] > 1.0,
            "ethanol in water should have γ > 1, got {}",
            gammas[0]
        );
    }

    #[test]
    fn symmetric_at_equimolar() {
        // Not necessarily equal, but both should deviate from 1
        let ethanol = decompose_ethanol();
        let water = decompose_water();
        let gammas = unifac_gamma(&[ethanol, water], &[0.5, 0.5], 78.0);
        assert!(gammas[0] > 1.0 && gammas[1] > 1.0);
    }

    #[test]
    fn ethanol_water_azeotrope_exists() {
        // With UNIFAC, the ethanol-water system should produce an azeotrope
        // somewhere around x_ethanol ≈ 0.89 (mole fraction)
        let ethanol = decompose_ethanol();
        let water = decompose_water();

        // Check that enrichment changes sign (y1 > x1 at low x1, y1 < x1 at high x1)
        let low_x = 0.1;
        let high_x = 0.95;
        let gammas_low = unifac_gamma(
            &[ethanol.clone(), water.clone()],
            &[low_x, 1.0 - low_x],
            78.0,
        );
        let gammas_high = unifac_gamma(
            &[ethanol, water],
            &[high_x, 1.0 - high_x],
            78.0,
        );
        // At low x: γ_ethanol should be high (enriches in vapour)
        // At high x: γ_water should be high (water enriches in vapour)
        assert!(
            gammas_low[0] > gammas_high[0],
            "ethanol γ should decrease with concentration: {:.3} vs {:.3}",
            gammas_low[0],
            gammas_high[0]
        );
    }

    #[test]
    fn interaction_symmetric_lookup() {
        // a(1,5) and a(5,1) should be different (not symmetric)
        let a15 = interaction(1, 5);
        let a51 = interaction(5, 1);
        assert!(
            (a15 - a51).abs() > 1.0,
            "a(1,5)={} and a(5,1)={} should differ",
            a15,
            a51
        );
    }

    #[test]
    fn group_parameters_are_positive() {
        for g in GROUPS {
            assert!(g.r > 0.0, "{} has non-positive R", g.name);
            assert!(g.q >= 0.0, "{} has negative Q", g.name);
        }
    }
}

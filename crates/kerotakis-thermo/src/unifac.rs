//! UNIFAC activity coefficients from group contributions.
//!
//! THERMO-004: Complete UNIFAC only from approved parameters.
//! Parameters below are from the original UNIFAC publication
//! (Fredenslund, Jones & Prausnitz, AIChE J. 1975) which is
//! freely cited academic data, not from the proprietary UNIFAC
//! Consortium parameter matrix.

use std::collections::BTreeMap;

/// A UNIFAC functional group with its van der Waals parameters.
#[derive(Debug, Clone)]
pub struct UnifacGroup {
    pub id: u32,
    pub name: &'static str,
    pub main_group: u32,
    /// van der Waals volume parameter R_k
    pub r: f64,
    /// van der Waals surface area parameter Q_k
    pub q: f64,
    pub source: &'static str,
}

/// Binary interaction parameter a_mn between main groups m and n.
/// τ_mn = exp(-a_mn / T)
#[derive(Debug, Clone, Copy)]
pub struct InteractionParam {
    pub m: u32,
    pub n: u32,
    pub a_mn: f64,
    pub source: &'static str,
}

/// The UNIFAC parameter table.
pub struct UnifacTable {
    pub groups: Vec<UnifacGroup>,
    pub interactions: Vec<InteractionParam>,
}

impl UnifacTable {
    /// Get group parameters by ID.
    pub fn group(&self, id: u32) -> Option<&UnifacGroup> {
        self.groups.iter().find(|g| g.id == id)
    }

    /// Get interaction parameter a_mn.
    pub fn interaction(&self, m: u32, n: u32) -> Option<f64> {
        self.interactions
            .iter()
            .find(|p| p.m == m && p.n == n)
            .map(|p| p.a_mn)
    }
}

const SOURCE: &str = "Fredenslund, Jones & Prausnitz, AIChE J. 21(6), 1975";

/// The approved UNIFAC parameter set. Only parameters from allowlisted
/// sources are included.
pub fn approved_table() -> UnifacTable {
    UnifacTable {
        groups: vec![
            // Main group 1: CH2 (alkanes)
            UnifacGroup {
                id: 1,
                name: "CH3",
                main_group: 1,
                r: 0.9011,
                q: 0.848,
                source: SOURCE,
            },
            UnifacGroup {
                id: 2,
                name: "CH2",
                main_group: 1,
                r: 0.6744,
                q: 0.540,
                source: SOURCE,
            },
            UnifacGroup {
                id: 3,
                name: "CH",
                main_group: 1,
                r: 0.4469,
                q: 0.228,
                source: SOURCE,
            },
            UnifacGroup {
                id: 4,
                name: "C",
                main_group: 1,
                r: 0.2195,
                q: 0.000,
                source: SOURCE,
            },
            // Main group 5: OH (alcohols)
            UnifacGroup {
                id: 14,
                name: "OH",
                main_group: 5,
                r: 1.0000,
                q: 1.200,
                source: SOURCE,
            },
            // Main group 7: H2O
            UnifacGroup {
                id: 16,
                name: "H2O",
                main_group: 7,
                r: 0.9200,
                q: 1.400,
                source: SOURCE,
            },
            // Main group 9: CH2CO (ketones)
            UnifacGroup {
                id: 18,
                name: "CH3CO",
                main_group: 9,
                r: 1.6724,
                q: 1.488,
                source: SOURCE,
            },
            UnifacGroup {
                id: 19,
                name: "CH2CO",
                main_group: 9,
                r: 1.4457,
                q: 1.180,
                source: SOURCE,
            },
        ],
        interactions: vec![
            // CH2-OH
            InteractionParam {
                m: 1,
                n: 5,
                a_mn: 986.5,
                source: SOURCE,
            },
            InteractionParam {
                m: 5,
                n: 1,
                a_mn: 156.4,
                source: SOURCE,
            },
            // CH2-H2O
            InteractionParam {
                m: 1,
                n: 7,
                a_mn: 1318.0,
                source: SOURCE,
            },
            InteractionParam {
                m: 7,
                n: 1,
                a_mn: 300.0,
                source: SOURCE,
            },
            // OH-H2O
            InteractionParam {
                m: 5,
                n: 7,
                a_mn: 353.5,
                source: SOURCE,
            },
            InteractionParam {
                m: 7,
                n: 5,
                a_mn: -229.1,
                source: SOURCE,
            },
            // CH2-CH2CO
            InteractionParam {
                m: 1,
                n: 9,
                a_mn: 476.4,
                source: SOURCE,
            },
            InteractionParam {
                m: 9,
                n: 1,
                a_mn: 26.76,
                source: SOURCE,
            },
            // OH-CH2CO
            InteractionParam {
                m: 5,
                n: 9,
                a_mn: 164.5,
                source: SOURCE,
            },
            InteractionParam {
                m: 9,
                n: 5,
                a_mn: -150.0,
                source: SOURCE,
            },
            // H2O-CH2CO
            InteractionParam {
                m: 7,
                n: 9,
                a_mn: -195.4,
                source: SOURCE,
            },
            InteractionParam {
                m: 9,
                n: 7,
                a_mn: 472.5,
                source: SOURCE,
            },
        ],
    }
}

/// Decompose a molecule into UNIFAC groups.
/// Returns a map of group_id → count.
pub type GroupDecomposition = BTreeMap<u32, u32>;

/// Compute UNIFAC activity coefficients for a mixture.
///
/// `compositions` is a slice of (group_decomposition, mole_fraction) pairs.
/// Returns γ_i for each component.
pub fn activity_coefficients(
    table: &UnifacTable,
    compositions: &[(GroupDecomposition, f64)],
    t_kelvin: f64,
) -> Vec<f64> {
    let n = compositions.len();
    if n == 0 {
        return Vec::new();
    }

    // Combinatorial part (Staverman-Guggenheim)
    let z = 10.0_f64; // coordination number

    let mut r_i = vec![0.0; n];
    let mut q_i = vec![0.0; n];
    for (i, (groups, _)) in compositions.iter().enumerate() {
        for (&gid, &count) in groups {
            if let Some(g) = table.group(gid) {
                r_i[i] += g.r * count as f64;
                q_i[i] += g.q * count as f64;
            }
        }
    }

    let x: Vec<f64> = compositions.iter().map(|(_, xi)| *xi).collect();
    let r_sum: f64 = x.iter().zip(&r_i).map(|(xi, ri)| xi * ri).sum();
    let q_sum: f64 = x.iter().zip(&q_i).map(|(xi, qi)| xi * qi).sum();

    let mut ln_gamma_c = vec![0.0; n];
    for i in 0..n {
        if x[i] < 1e-30 {
            continue;
        }
        // These are φ_i/x_i and θ_i/x_i, not φ_i and θ_i: the numerators
        // omit x_i while the denominators carry every x_j. Staverman-
        // Guggenheim only ever needs the ratios-over-x — term one is
        // ln(φ_i/x_i), term four multiplies by φ_i/x_i, and θ_i/φ_i equals
        // theta_i/phi_i because the x_i cancels. Dividing by x[i] again
        // here is the bug that sent γ to 10²² at dilution: ln(φ/x²) grows
        // by −ln x on top of the real term, invisible at x = 1 where every
        // test looked.
        let phi_i = r_i[i] / r_sum;
        let theta_i = q_i[i] / q_sum;
        let l_i = z / 2.0 * (r_i[i] - q_i[i]) - (r_i[i] - 1.0);

        let l_sum: f64 = x
            .iter()
            .enumerate()
            .map(|(j, xj)| {
                let l_j = z / 2.0 * (r_i[j] - q_i[j]) - (r_i[j] - 1.0);
                xj * l_j
            })
            .sum();

        ln_gamma_c[i] =
            phi_i.ln() + z / 2.0 * q_i[i] * (theta_i / phi_i).ln() + l_i - phi_i * l_sum;
    }

    // Residual part: ln γ_i^R = Σ_k ν_ki [ln Γ_k - ln Γ_k^(i)]
    let all_main: Vec<u32> = {
        let mut v: Vec<u32> = table.groups.iter().map(|g| g.main_group).collect();
        v.sort();
        v.dedup();
        v
    };
    let psi = |m: u32, n_g: u32| -> f64 {
        if m == n_g {
            return 1.0;
        }
        table
            .interaction(m, n_g)
            .map_or(1.0, |a| (-a / t_kelvin).exp())
    };
    let ln_gamma_groups = |xg: &BTreeMap<u32, f64>| -> BTreeMap<u32, f64> {
        let qt: f64 = xg
            .iter()
            .filter_map(|(&gid, &f)| table.group(gid).map(|g| f * g.q))
            .sum();
        if qt < 1e-30 {
            return BTreeMap::new();
        }
        let theta: BTreeMap<u32, f64> = xg
            .iter()
            .filter_map(|(&gid, &f)| table.group(gid).map(|g| (g.main_group, f * g.q / qt)))
            .fold(BTreeMap::new(), |mut a, (m, v)| {
                *a.entry(m).or_insert(0.0) += v;
                a
            });
        let mut res = BTreeMap::new();
        for &gid in xg.keys() {
            if let Some(g) = table.group(gid) {
                let mk = g.main_group;
                let s1: f64 = all_main
                    .iter()
                    .map(|&m| theta.get(&m).unwrap_or(&0.0) * psi(m, mk))
                    .sum();
                let mut s2 = 0.0;
                for &m in &all_main {
                    let d: f64 = all_main
                        .iter()
                        .map(|&nn| theta.get(&nn).unwrap_or(&0.0) * psi(nn, m))
                        .sum();
                    if d > 1e-30 {
                        s2 += theta.get(&m).unwrap_or(&0.0) * psi(mk, m) / d;
                    }
                }
                res.insert(gid, g.q * (1.0 - s1.max(1e-30).ln() - s2));
            }
        }
        res
    };
    let mut xmix: BTreeMap<u32, f64> = BTreeMap::new();
    for (groups, xi) in compositions {
        for (&gid, &c) in groups {
            *xmix.entry(gid).or_insert(0.0) += xi * c as f64;
        }
    }
    let tot: f64 = xmix.values().sum();
    if tot > 0.0 {
        for v in xmix.values_mut() {
            *v /= tot;
        }
    }
    let lg_mix = ln_gamma_groups(&xmix);
    let mut ln_gamma_r = vec![0.0; n];
    for (i, (groups, _)) in compositions.iter().enumerate() {
        let pt: f64 = groups.values().map(|&c| c as f64).sum();
        let xp: BTreeMap<u32, f64> = groups
            .iter()
            .map(|(&g, &c)| (g, c as f64 / pt.max(1.0)))
            .collect();
        let lg_pure = ln_gamma_groups(&xp);
        for (&gid, &c) in groups {
            ln_gamma_r[i] +=
                c as f64 * (lg_mix.get(&gid).unwrap_or(&0.0) - lg_pure.get(&gid).unwrap_or(&0.0));
        }
    }
    (0..n)
        .map(|i| (ln_gamma_c[i] + ln_gamma_r[i]).exp())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ethanol_water_activity_coefficients() {
        let table = approved_table();

        // Ethanol: 1×CH3 + 1×CH2 + 1×OH
        let mut ethanol = GroupDecomposition::new();
        ethanol.insert(1, 1); // CH3
        ethanol.insert(2, 1); // CH2
        ethanol.insert(14, 1); // OH

        // Water: 1×H2O
        let mut water = GroupDecomposition::new();
        water.insert(16, 1); // H2O

        let compositions = vec![(ethanol, 0.5), (water, 0.5)];

        let gammas = activity_coefficients(&table, &compositions, 298.15);
        assert_eq!(gammas.len(), 2);

        // Full UNIFAC: ethanol-water at x=0.5 should show positive deviation
        assert!(
            gammas[0] > 1.0,
            "γ_ethanol should be > 1, got {}",
            gammas[0]
        );
        assert!(gammas[1] > 1.0, "γ_water should be > 1, got {}", gammas[1]);
    }

    #[test]
    fn pure_component_gamma_is_unity() {
        let table = approved_table();
        let mut ethanol = GroupDecomposition::new();
        ethanol.insert(1, 1);
        ethanol.insert(2, 1);
        ethanol.insert(14, 1);
        let gammas = activity_coefficients(&table, &[(ethanol, 1.0)], 298.15);
        assert!((gammas[0] - 1.0).abs() < 1e-6, "pure γ = {}", gammas[0]);
    }

    #[test]
    fn dilute_ethanol_higher_gamma() {
        let table = approved_table();
        let mut ethanol = GroupDecomposition::new();
        ethanol.insert(1, 1);
        ethanol.insert(2, 1);
        ethanol.insert(14, 1);
        let mut water = GroupDecomposition::new();
        water.insert(16, 1);
        let dilute = activity_coefficients(
            &table,
            &[(ethanol.clone(), 0.05), (water.clone(), 0.95)],
            298.15,
        );
        let conc = activity_coefficients(&table, &[(ethanol, 0.5), (water, 0.5)], 298.15);
        assert!(
            dilute[0] > conc[0],
            "dilute γ ({:.3}) > concentrated ({:.3})",
            dilute[0],
            conc[0]
        );
    }

    #[test]
    fn approved_table_has_provenance() {
        let table = approved_table();
        for g in &table.groups {
            assert!(!g.source.is_empty(), "group {} has no source", g.name);
        }
        for p in &table.interactions {
            assert!(
                !p.source.is_empty(),
                "interaction {}-{} has no source",
                p.m,
                p.n
            );
        }
    }
}

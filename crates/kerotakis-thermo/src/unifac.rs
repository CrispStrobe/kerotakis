//! Original UNIFAC: activity coefficients of a liquid mixture from the
//! functional groups its molecules are made of (PLAN.md, P3p — "The
//! UNIFAC question, precisely").
//!
//! The algorithm is the small half — Fredenslund, Jones & Prausnitz,
//! AIChE J. 21 (1975) 1086, a UNIQUAC combinatorial term plus a residual
//! term summed over groups. The real work is the data: every group volume,
//! surface and interaction parameter here is transcribed from the
//! *published* original-UNIFAC tables (Hansen, Rasmussen, Fredenslund,
//! Schiller & Gmehling, Ind. Eng. Chem. Res. 30 (1991) 2352–2355, as
//! republished openly by DDBST), carries that source on the row, and was
//! checked on 2026-08-20 against an independent implementation (the
//! `thermo` Python oracle, `tools/fixtures/vle-ethanol-water.json`): all
//! ten γ points agree to 5e-7 relative, which is the fixture's own print
//! precision. Nothing is taken from the UNIFAC Consortium's maintained
//! tables (proprietary) or from the frozen `unifac` crate's embedded data
//! (warning clause) — budgeted, as PLAN says, as data curation.
//!
//! Scope of this tranche: the groups that describe water, ethanol and
//! acetic acid — CH₃, CH₂, OH, H₂O, COOH. The ethanol–water azeotrope is
//! the acceptance test. A molecule this table cannot build is a
//! `GroupError`, not a guess.

use std::collections::BTreeMap;

/// One UNIFAC subgroup: its main group (interaction parameters are per
/// main group), van der Waals volume `r` and surface `q`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Subgroup {
    pub name: &'static str,
    pub main: u8,
    pub r: f64,
    pub q: f64,
    pub source: &'static str,
}

const DDBST_RQ: &str = "R, Q: original UNIFAC subgroup table, Hansen et al. 1991 (Ind. Eng. Chem. Res. 30, 2352), as published by DDBST (ddbst.com/published-parameters-unifac.html), read 2026-08-20";

pub const SUBGROUPS: &[Subgroup] = &[
    Subgroup {
        name: "CH3",
        main: 1,
        r: 0.9011,
        q: 0.848,
        source: DDBST_RQ,
    },
    Subgroup {
        name: "CH2",
        main: 1,
        r: 0.6744,
        q: 0.540,
        source: DDBST_RQ,
    },
    Subgroup {
        name: "OH",
        main: 5,
        r: 1.0000,
        q: 1.200,
        source: DDBST_RQ,
    },
    Subgroup {
        name: "H2O",
        main: 7,
        r: 0.9200,
        q: 1.400,
        source: DDBST_RQ,
    },
    Subgroup {
        name: "COOH",
        main: 20,
        r: 1.3013,
        q: 1.224,
        source: DDBST_RQ,
    },
];

/// Interaction parameter a(m→n) in kelvin between two *main* groups. Not
/// symmetric: a(1,5) = 986.5 and a(5,1) = 156.4 are two different numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interaction {
    pub from: u8,
    pub to: u8,
    pub a_kelvin: f64,
    pub source: &'static str,
}

const DDBST_A: &str = "a_mn: original UNIFAC interaction table, Hansen et al. 1991 (Ind. Eng. Chem. Res. 30, 2352), as published by DDBST, read 2026-08-20";

pub const INTERACTIONS: &[Interaction] = &[
    Interaction {
        from: 1,
        to: 5,
        a_kelvin: 986.5,
        source: DDBST_A,
    },
    Interaction {
        from: 5,
        to: 1,
        a_kelvin: 156.4,
        source: DDBST_A,
    },
    Interaction {
        from: 1,
        to: 7,
        a_kelvin: 1318.0,
        source: DDBST_A,
    },
    Interaction {
        from: 7,
        to: 1,
        a_kelvin: 300.0,
        source: DDBST_A,
    },
    Interaction {
        from: 1,
        to: 20,
        a_kelvin: 663.5,
        source: DDBST_A,
    },
    Interaction {
        from: 20,
        to: 1,
        a_kelvin: 315.3,
        source: DDBST_A,
    },
    Interaction {
        from: 5,
        to: 7,
        a_kelvin: 353.5,
        source: DDBST_A,
    },
    Interaction {
        from: 7,
        to: 5,
        a_kelvin: -229.1,
        source: DDBST_A,
    },
    Interaction {
        from: 5,
        to: 20,
        a_kelvin: 199.0,
        source: DDBST_A,
    },
    Interaction {
        from: 20,
        to: 5,
        a_kelvin: -151.0,
        source: DDBST_A,
    },
    Interaction {
        from: 7,
        to: 20,
        a_kelvin: -14.09,
        source: DDBST_A,
    },
    Interaction {
        from: 20,
        to: 7,
        a_kelvin: -66.17,
        source: DDBST_A,
    },
];

/// The lattice coordination number of the combinatorial term; 10 by the
/// original paper's convention.
const Z: f64 = 10.0;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GroupError {
    #[error("no UNIFAC subgroup named '{0}' in this lab's table")]
    UnknownSubgroup(String),
    #[error("no interaction parameter between main groups {0} and {1}: the table does not cover this pair, so the mixture cannot be computed")]
    MissingInteraction(u8, u8),
    #[error("a component with no groups")]
    EmptyComponent,
    #[error("mole fractions must be positive and sum to one (got {0})")]
    BadComposition(f64),
}

fn subgroup(name: &str) -> Result<&'static Subgroup, GroupError> {
    SUBGROUPS
        .iter()
        .find(|s| s.name == name)
        .ok_or_else(|| GroupError::UnknownSubgroup(name.to_string()))
}

fn interaction(from: u8, to: u8) -> Result<f64, GroupError> {
    if from == to {
        return Ok(0.0);
    }
    INTERACTIONS
        .iter()
        .find(|i| i.from == from && i.to == to)
        .map(|i| i.a_kelvin)
        .ok_or(GroupError::MissingInteraction(from, to))
}

/// A molecule as UNIFAC sees it: how many of each subgroup.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: &'static str,
    groups: Vec<(&'static Subgroup, f64)>,
}

impl Component {
    pub fn new(name: &'static str, groups: &[(&str, f64)]) -> Result<Self, GroupError> {
        if groups.is_empty() {
            return Err(GroupError::EmptyComponent);
        }
        let groups = groups
            .iter()
            .map(|(g, n)| subgroup(g).map(|s| (s, *n)))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Component { name, groups })
    }

    /// H₂O is its own group.
    pub fn water() -> Self {
        Self::new("water", &[("H2O", 1.0)]).expect("in table")
    }

    /// CH₃–CH₂–OH.
    pub fn ethanol() -> Self {
        Self::new("ethanol", &[("CH3", 1.0), ("CH2", 1.0), ("OH", 1.0)]).expect("in table")
    }

    /// CH₃–COOH.
    pub fn acetic_acid() -> Self {
        Self::new("acetic acid", &[("CH3", 1.0), ("COOH", 1.0)]).expect("in table")
    }

    fn r(&self) -> f64 {
        self.groups.iter().map(|(s, n)| n * s.r).sum()
    }

    fn q(&self) -> f64 {
        self.groups.iter().map(|(s, n)| n * s.q).sum()
    }
}

/// A liquid mixture at a composition.
#[derive(Debug, Clone, PartialEq)]
pub struct Mixture {
    components: Vec<(Component, f64)>,
}

impl Mixture {
    pub fn new(components: &[(Component, f64)]) -> Result<Self, GroupError> {
        let sum: f64 = components.iter().map(|(_, x)| x).sum();
        if components.iter().any(|(_, x)| *x <= 0.0) || (sum - 1.0).abs() > 1e-9 {
            return Err(GroupError::BadComposition(sum));
        }
        // Check the interaction table covers every pair up front, so a
        // missing parameter is a refusal at construction, not a NaN later.
        let mains: Vec<u8> = components
            .iter()
            .flat_map(|(c, _)| c.groups.iter().map(|(s, _)| s.main))
            .collect();
        for &m in &mains {
            for &n in &mains {
                interaction(m, n)?;
            }
        }
        Ok(Mixture {
            components: components.to_vec(),
        })
    }

    /// Activity coefficients γᵢ at `t_kelvin`, in component order.
    pub fn activity_coefficients(&self, t_kelvin: f64) -> Vec<f64> {
        let n = self.components.len();
        let x: Vec<f64> = self.components.iter().map(|(_, x)| *x).collect();
        let r: Vec<f64> = self.components.iter().map(|(c, _)| c.r()).collect();
        let q: Vec<f64> = self.components.iter().map(|(c, _)| c.q()).collect();

        // --- Combinatorial (UNIQUAC): size and shape.
        let sx_r: f64 = (0..n).map(|i| x[i] * r[i]).sum();
        let sx_q: f64 = (0..n).map(|i| x[i] * q[i]).sum();
        let l: Vec<f64> = (0..n)
            .map(|i| Z / 2.0 * (r[i] - q[i]) - (r[i] - 1.0))
            .collect();
        let sx_l: f64 = (0..n).map(|i| x[i] * l[i]).sum();
        let ln_c: Vec<f64> = (0..n)
            .map(|i| {
                let phi = x[i] * r[i] / sx_r;
                let theta = x[i] * q[i] / sx_q;
                (phi / x[i]).ln() + Z / 2.0 * q[i] * (theta / phi).ln() + l[i] - phi / x[i] * sx_l
            })
            .collect();

        // --- Residual: group contributions in the mixture minus in the
        // pure component, so a pure liquid has γ = 1 by construction.
        let mut groups: Vec<&'static Subgroup> = Vec::new();
        for (c, _) in &self.components {
            for (s, _) in &c.groups {
                if !groups.iter().any(|g| g.name == s.name) {
                    groups.push(s);
                }
            }
        }
        let count = |c: &Component, g: &Subgroup| -> f64 {
            c.groups
                .iter()
                .filter(|(s, _)| s.name == g.name)
                .map(|(_, n)| *n)
                .sum()
        };
        let ln_gamma_k = |x_groups: &BTreeMap<&str, f64>| -> BTreeMap<&'static str, f64> {
            let s: f64 = groups.iter().map(|g| x_groups[g.name] * g.q).sum();
            let theta: Vec<f64> = groups.iter().map(|g| x_groups[g.name] * g.q / s).collect();
            let psi = |m: usize, k: usize| -> f64 {
                (-interaction(groups[m].main, groups[k].main).expect("checked at construction")
                    / t_kelvin)
                    .exp()
            };
            let mut out = BTreeMap::new();
            for (k, g) in groups.iter().enumerate() {
                let t1: f64 = (0..groups.len())
                    .map(|m| theta[m] * psi(m, k))
                    .sum::<f64>()
                    .ln();
                let t2: f64 = (0..groups.len())
                    .map(|m| {
                        theta[m] * psi(k, m)
                            / (0..groups.len())
                                .map(|nn| theta[nn] * psi(nn, m))
                                .sum::<f64>()
                    })
                    .sum();
                out.insert(g.name, g.q * (1.0 - t1 - t2));
            }
            out
        };
        let total: f64 = (0..n)
            .map(|i| {
                x[i] * self.components[i]
                    .0
                    .groups
                    .iter()
                    .map(|(_, c)| c)
                    .sum::<f64>()
            })
            .sum();
        let x_mix: BTreeMap<&str, f64> = groups
            .iter()
            .map(|g| {
                (
                    g.name,
                    (0..n)
                        .map(|i| x[i] * count(&self.components[i].0, g))
                        .sum::<f64>()
                        / total,
                )
            })
            .collect();
        let ln_gamma_mix = ln_gamma_k(&x_mix);
        (0..n)
            .map(|i| {
                let c = &self.components[i].0;
                let own: f64 = c.groups.iter().map(|(_, n)| n).sum();
                let x_pure: BTreeMap<&str, f64> =
                    groups.iter().map(|g| (g.name, count(c, g) / own)).collect();
                let ln_gamma_pure = ln_gamma_k(&x_pure);
                let ln_r: f64 = c
                    .groups
                    .iter()
                    .map(|(s, nk)| nk * (ln_gamma_mix[s.name] - ln_gamma_pure[s.name]))
                    .sum();
                (ln_c[i] + ln_r).exp()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pure_liquid_has_unit_activity() {
        let m = Mixture::new(&[(Component::ethanol(), 1.0)]).unwrap();
        let g = m.activity_coefficients(298.15);
        assert!((g[0] - 1.0).abs() < 1e-12, "{g:?}");
    }

    #[test]
    fn every_interaction_has_its_reverse_and_a_source() {
        for i in INTERACTIONS {
            assert!(
                INTERACTIONS
                    .iter()
                    .any(|j| j.from == i.to && j.to == i.from),
                "a({},{}) has no a({},{})",
                i.from,
                i.to,
                i.to,
                i.from
            );
            assert!(i.source.contains("Hansen"));
        }
    }

    #[test]
    fn a_molecule_the_table_cannot_build_is_refused() {
        assert!(matches!(
            Component::new("benzene", &[("ACH", 6.0)]),
            Err(GroupError::UnknownSubgroup(_))
        ));
    }
}

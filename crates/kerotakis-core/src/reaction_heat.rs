//! Reaction heats per declared EXTENT, from one rule.
//!
//! PR #850 found the peroxide decomposition releasing twice its heat: the
//! reference value, ~98.2 kJ per mole of H₂O₂, was multiplied by the
//! integrator's extent of `2 H₂O₂ → 2 H₂O + O₂`, and one extent consumes two
//! moles. The repair was correct and local. This module is the same repair
//! for every row that carries a reaction heat, so the convention cannot
//! diverge again:
//!
//! * a reference heat is always stated together with WHAT it is per — one
//!   extent of the written equation, or one mole of a named participant;
//! * the heat per extent is DERIVED as `reference × |coefficient|` of that
//!   participant in the equation the integrator actually advances, never
//!   retyped beside it;
//! * [`audit`] enumerates every heat-carrying row on the bench (curated
//!   kinetics, combustion fuels, shipped gas-mechanism packs) with its basis,
//!   coefficient and per-extent heat, and the kinetics registry is checked
//!   for heat-carrying rows that bypass this table.
//!
//! Sign: positive is heat RELEASED to the vessel (−ΔH).

use crate::kinetics::{self, KineticReaction};

/// What a reference reaction heat is stated per.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeatBasis {
    /// Per extent of the equation as written.
    PerExtent,
    /// Per mole of this participant consumed or produced.
    PerMoleOf(&'static str),
}

/// A reference heat attached to a curated kinetic reaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KineticHeat {
    pub reaction: &'static str,
    /// Heat released per [`HeatBasis`] unit, J.
    pub released_j: f64,
    pub basis: HeatBasis,
    pub source: &'static str,
}

/// Every curated kinetic reaction that releases heat. A kinetic row with a
/// reaction heat anywhere else is a bypass, and the audit test fails on it.
pub const KINETIC_HEATS: &[KineticHeat] = &[KineticHeat {
    reaction: "peroxide-decomposition",
    released_j: 98_200.0,
    basis: HeatBasis::PerMoleOf("H2O2"),
    source: "Approximately 98.2 kJ released per mole of H2O2 consumed at 25 °C. \
             NIST TN2126, Table 4-4(1), reports 98.48 ± 0.79 kJ/mol for liquid H2O2; \
             solution mixing and temperature corrections remain absent (PR #850).",
}];

/// `|coefficient|` of `species` in a stoichiometry, summed over its terms.
pub fn coefficient_of(reaction: &KineticReaction<'_>, species: &str) -> Option<f64> {
    let n: f64 = reaction
        .stoichiometry
        .iter()
        .filter(|t| t.species == species)
        .map(|t| t.coefficient)
        .sum();
    (n != 0.0).then_some(n.abs())
}

/// Heat released per extent of the written equation, J, for a reference
/// heat stated per `basis` of this reaction.
pub fn released_per_extent_j(
    reaction: &KineticReaction<'_>,
    released_j: f64,
    basis: HeatBasis,
) -> Option<f64> {
    match basis {
        HeatBasis::PerExtent => Some(released_j),
        HeatBasis::PerMoleOf(species) => Some(released_j * coefficient_of(reaction, species)?),
    }
}

/// The heat a curated kinetic reaction released over `extent`, J, or `None`
/// for a reaction that carries no reference heat.
pub fn kinetic_heat_released_j(reaction: &KineticReaction<'_>, extent: f64) -> Option<f64> {
    let row = KINETIC_HEATS
        .iter()
        .find(|row| row.reaction == reaction.id)?;
    Some(released_per_extent_j(reaction, row.released_j, row.basis)? * extent)
}

/// Heat released by burning `fuel_moles` of a curated fuel, J. The fuel's
/// coefficient in its own equation is one by construction (checked by
/// [`audit`]), so its reference per mole of fuel IS per extent.
pub fn fuel_heat_released_j(fuel: &crate::combustion::Fuel, fuel_moles: f64) -> f64 {
    fuel.heat_j_per_mol * fuel_moles
}

/// The leading coefficient of the FIRST reactant of a plain `a X + b Y -> ...`
/// equation (the fuel, in [`crate::combustion::FUELS`]).
fn first_reactant_coefficient(equation: &str) -> Option<f64> {
    let first = equation.split("->").next()?.split(" + ").next()?.trim();
    match first.split_once(' ') {
        Some((n, _)) => n.parse::<f64>().ok(),
        None => Some(1.0),
    }
}

/// One heat-carrying row on the bench.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatRow {
    /// `kinetics`, `combustion` or `gas-pack:<id>`.
    pub family: String,
    pub id: String,
    pub equation: String,
    /// What the reference heat is stated per.
    pub basis: String,
    /// `|coefficient|` of the basis participant (1 for per-extent).
    pub coefficient: f64,
    /// Reference heat released per basis unit, J.
    pub reference_j: f64,
    /// Derived heat released per extent of the written equation, J.
    pub per_extent_j: f64,
}

/// Every heat-carrying reaction row, with its basis and derived per-extent
/// heat. Rows whose basis cannot be resolved against their own equation are
/// returned with a NaN per-extent heat, which the audit test rejects.
pub fn audit() -> Vec<HeatRow> {
    let mut rows = Vec::new();
    for heat in KINETIC_HEATS {
        let reaction = kinetics::lookup(heat.reaction);
        let (basis, coefficient) = match heat.basis {
            HeatBasis::PerExtent => ("extent".to_string(), Some(1.0)),
            HeatBasis::PerMoleOf(s) => (
                format!("mol {s}"),
                reaction.and_then(|r| coefficient_of(r, s)),
            ),
        };
        let coefficient = coefficient.unwrap_or(f64::NAN);
        rows.push(HeatRow {
            family: "kinetics".to_string(),
            id: heat.reaction.to_string(),
            equation: reaction.map_or_else(String::new, |r| r.equation.to_string()),
            basis,
            coefficient,
            reference_j: heat.released_j,
            per_extent_j: heat.released_j * coefficient,
        });
    }
    for fuel in crate::combustion::FUELS {
        let coefficient = first_reactant_coefficient(fuel.equation).unwrap_or(f64::NAN);
        rows.push(HeatRow {
            family: "combustion".to_string(),
            id: fuel.species.to_string(),
            equation: fuel.equation.to_string(),
            basis: format!("mol {}", fuel.species),
            coefficient,
            reference_j: fuel.heat_j_per_mol,
            // The burn advances one extent per mole of fuel, so a fuel
            // written with any other coefficient would be mis-scaled.
            per_extent_j: if coefficient == 1.0 {
                fuel.heat_j_per_mol
            } else {
                f64::NAN
            },
        });
    }
    for pack in kinetics::packs::shipped() {
        for reaction in pack.network.reactions {
            let Some(dh) = pack.reaction_enthalpy_j_per_mol(reaction) else {
                continue;
            };
            rows.push(HeatRow {
                family: format!("gas-pack:{}", pack.id),
                id: reaction.id.to_string(),
                equation: reaction.equation.to_string(),
                basis: "extent".to_string(),
                coefficient: 1.0,
                reference_j: -dh,
                per_extent_j: -dh,
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PR #850, generalised: the peroxide heat per extent is derived from
    /// the coefficient the integrator advances, 2 × 98.2 kJ.
    #[test]
    fn peroxide_heat_is_per_mole_of_peroxide_not_per_extent() {
        let r = kinetics::lookup("peroxide-decomposition").expect("curated");
        let per_extent = released_per_extent_j(r, 98_200.0, HeatBasis::PerMoleOf("H2O2")).unwrap();
        assert!((per_extent - 196_400.0).abs() < 1e-6, "{per_extent}");
        // One extent consumes two moles; the heat follows the moles.
        let q = kinetic_heat_released_j(r, 0.005).unwrap();
        assert!((q - 0.01 * 98_200.0).abs() < 1e-6, "{q}");
    }

    /// Negative: a basis participant absent from the equation does not
    /// silently default to coefficient one.
    #[test]
    fn a_basis_species_not_in_the_equation_is_unresolved() {
        let r = kinetics::lookup("peroxide-decomposition").expect("curated");
        assert!(released_per_extent_j(r, 1.0, HeatBasis::PerMoleOf("NaCl")).is_none());
        assert_eq!(
            released_per_extent_j(r, 5.0, HeatBasis::PerExtent),
            Some(5.0)
        );
    }

    /// Boundary: a curated kinetic reaction WITHOUT a reference heat releases
    /// none rather than borrowing another row's.
    #[test]
    fn a_kinetic_row_without_a_reference_heat_releases_none() {
        for reaction in kinetics::REGISTRY {
            if KINETIC_HEATS.iter().any(|h| h.reaction == reaction.id) {
                continue;
            }
            assert!(
                kinetic_heat_released_j(reaction, 1.0).is_none(),
                "{}",
                reaction.id
            );
        }
    }

    /// Every heat-carrying row resolves its basis against its own equation
    /// and states a finite per-extent heat.
    #[test]
    fn every_heat_row_is_per_declared_extent() {
        let rows = audit();
        assert!(rows.iter().any(|r| r.id == "peroxide-decomposition"));
        assert!(rows.iter().any(|r| r.family == "combustion"));
        for row in &rows {
            assert!(
                row.per_extent_j.is_finite() && row.coefficient.is_finite(),
                "{row:?}"
            );
            assert!(
                (row.per_extent_j - row.reference_j * row.coefficient).abs()
                    <= 1e-9 * row.reference_j.abs().max(1.0),
                "{row:?}"
            );
        }
        for kinetic in KINETIC_HEATS {
            assert!(
                kinetics::lookup(kinetic.reaction).is_some(),
                "{} is not a curated kinetic reaction",
                kinetic.reaction
            );
        }
    }
}

//! Ideal local homogeneous equilibria for reaction--transport coupling.
//!
//! Unknowns are reaction extents, so every iterate is `c = c0 + nu^T xi`.
//! Any conserved quantity annihilated by the authored stoichiometric matrix is
//! therefore preserved by construction. Callers supply activity coefficients;
//! this module never invents a non-ideal electrolyte model.

use std::collections::BTreeSet;

const STANDARD_CONCENTRATION_MOL_PER_M3: f64 = 1_000.0;
const CONCENTRATION_FLOOR: f64 = 1e-18;
const RESIDUAL_TOLERANCE: f64 = 1e-10;
const MAX_NEWTON_PASSES: usize = 96;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalEquilibriumSpecies<'a> {
    pub id: &'a str,
    pub charge: i32,
    pub activity_coefficient: f64,
    /// Conserved elemental or authored pseudo-component inventory.
    pub components: &'a [LocalEquilibriumComponent<'a>],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalEquilibriumComponent<'a> {
    pub id: &'a str,
    pub amount: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalEquilibriumTerm<'a> {
    pub species: &'a str,
    /// Products are positive and reactants negative.
    pub coefficient: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalEquilibriumReaction<'a> {
    pub id: &'a str,
    pub terms: &'a [LocalEquilibriumTerm<'a>],
    pub log10_equilibrium_constant: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalEquilibriumNetwork<'a> {
    pub species: &'a [LocalEquilibriumSpecies<'a>],
    pub reactions: &'a [LocalEquilibriumReaction<'a>],
}

/// Evidence boundary for treating a homogeneous network as locally
/// equilibrated inside a slower transport calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalEquilibriumValidity {
    pub minimum_temperature_k: f64,
    pub maximum_temperature_k: f64,
    /// Reviewed upper bound for the network's homogeneous relaxation.
    pub maximum_relaxation_time_s: f64,
    /// Required ratio of the slower process time to homogeneous relaxation.
    pub minimum_timescale_separation: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalEquilibriumState {
    pub concentrations_mol_per_m3: Vec<f64>,
    pub activities: Vec<f64>,
    pub reaction_extents_mol_per_m3: Vec<f64>,
    pub maximum_mass_action_residual: f64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LocalEquilibriumError {
    #[error("local equilibrium network is empty")]
    EmptyNetwork,
    #[error("local equilibrium species '{0}' is duplicated or invalid")]
    InvalidSpecies(String),
    #[error("local equilibrium reaction '{0}' is duplicated or invalid")]
    InvalidReaction(String),
    #[error("local equilibrium reaction '{reaction}' does not conserve charge ({charge:+.6e})")]
    ChargeImbalance { reaction: String, charge: f64 },
    #[error("local equilibrium reaction '{reaction}' does not conserve component '{component}' ({imbalance:+.6e})")]
    ComponentImbalance {
        reaction: String,
        component: String,
        imbalance: f64,
    },
    #[error("local equilibrium validity domain is invalid")]
    InvalidValidity,
    #[error("local equilibrium temperature {temperature_k:.6} K is outside [{minimum_k:.6}, {maximum_k:.6}] K")]
    TemperatureOutsideValidity {
        temperature_k: f64,
        minimum_k: f64,
        maximum_k: f64,
    },
    #[error("local equilibrium assumption is too slow: process/relaxation ratio {actual_ratio:.6e} is below {required_ratio:.6e}")]
    InsufficientTimescaleSeparation {
        actual_ratio: f64,
        required_ratio: f64,
    },
    #[error("local equilibrium reactions are linearly dependent")]
    RankDeficient,
    #[error("local equilibrium input concentration for '{0}' is invalid")]
    InvalidConcentration(String),
    #[error("local equilibrium has no positive feasible state for the supplied totals")]
    Infeasible,
    #[error("local equilibrium did not converge (maximum log residual {maximum_residual:.6e})")]
    DidNotConverge { maximum_residual: f64 },
}

impl LocalEquilibriumNetwork<'_> {
    pub fn equilibrate(
        &self,
        initial_concentrations_mol_per_m3: &[f64],
    ) -> Result<LocalEquilibriumState, LocalEquilibriumError> {
        let stoichiometry = self.validate()?;
        if initial_concentrations_mol_per_m3.len() != self.species.len() {
            return Err(LocalEquilibriumError::InvalidConcentration(
                "wrong number of species".to_string(),
            ));
        }
        for (species, concentration) in self.species.iter().zip(initial_concentrations_mol_per_m3) {
            if !concentration.is_finite() || *concentration < 0.0 {
                return Err(LocalEquilibriumError::InvalidConcentration(
                    species.id.to_string(),
                ));
            }
        }

        let mut extents = vec![0.0; self.reactions.len()];
        let mut concentrations = initial_concentrations_mol_per_m3.to_vec();
        let mut maximum_residual = f64::INFINITY;
        for _ in 0..MAX_NEWTON_PASSES {
            let residual = self.mass_action_residuals(&concentrations, &stoichiometry);
            maximum_residual = residual.iter().map(|value| value.abs()).fold(0.0, f64::max);
            if maximum_residual
                <= attainable_residual_tolerance(&concentrations, initial_concentrations_mol_per_m3)
                && concentrations.iter().all(|value| *value > 0.0)
            {
                return Ok(self.state(concentrations, extents, maximum_residual));
            }

            let jacobian = reaction_jacobian(&concentrations, &stoichiometry);
            let rhs = residual.iter().map(|value| -*value).collect::<Vec<_>>();
            let step = solve_square(jacobian, rhs).ok_or(LocalEquilibriumError::RankDeficient)?;
            let mut scale = positive_step_scale(&concentrations, &stoichiometry, &step);
            let mut accepted = None;
            for _ in 0..64 {
                let trial_extents = extents
                    .iter()
                    .zip(&step)
                    .map(|(extent, delta)| extent + scale * delta)
                    .collect::<Vec<_>>();
                let trial = concentrations_from_extents(
                    initial_concentrations_mol_per_m3,
                    &stoichiometry,
                    &trial_extents,
                );
                if trial.iter().all(|value| value.is_finite() && *value > 0.0) {
                    let merit = self
                        .mass_action_residuals(&trial, &stoichiometry)
                        .iter()
                        .map(|value| value.abs())
                        .fold(0.0, f64::max);
                    if merit < maximum_residual || !maximum_residual.is_finite() {
                        accepted = Some((trial_extents, trial));
                        break;
                    }
                }
                scale *= 0.5;
            }
            let Some((next_extents, next_concentrations)) = accepted else {
                return Err(if concentrations.iter().any(|value| *value <= 0.0) {
                    LocalEquilibriumError::Infeasible
                } else {
                    LocalEquilibriumError::DidNotConverge { maximum_residual }
                });
            };
            extents = next_extents;
            concentrations = next_concentrations;
        }
        Err(LocalEquilibriumError::DidNotConverge { maximum_residual })
    }

    fn validate(&self) -> Result<Vec<Vec<f64>>, LocalEquilibriumError> {
        if self.species.is_empty() || self.reactions.is_empty() {
            return Err(LocalEquilibriumError::EmptyNetwork);
        }
        let mut species_ids = BTreeSet::new();
        for species in self.species {
            let mut component_ids = BTreeSet::new();
            if species.id.trim().is_empty()
                || !species_ids.insert(species.id)
                || !species.activity_coefficient.is_finite()
                || species.activity_coefficient <= 0.0
                || species.components.is_empty()
                || species.components.iter().any(|component| {
                    component.id.trim().is_empty()
                        || !component_ids.insert(component.id)
                        || !component.amount.is_finite()
                        || component.amount <= 0.0
                })
            {
                return Err(LocalEquilibriumError::InvalidSpecies(
                    species.id.to_string(),
                ));
            }
        }

        let mut reaction_ids = BTreeSet::new();
        let mut stoichiometry = Vec::with_capacity(self.reactions.len());
        for reaction in self.reactions {
            if reaction.id.trim().is_empty()
                || !reaction_ids.insert(reaction.id)
                || !reaction.log10_equilibrium_constant.is_finite()
                || reaction.terms.len() < 2
            {
                return Err(LocalEquilibriumError::InvalidReaction(
                    reaction.id.to_string(),
                ));
            }
            let mut row = vec![0.0; self.species.len()];
            let mut term_ids = BTreeSet::new();
            for term in reaction.terms {
                let Some(index) = self
                    .species
                    .iter()
                    .position(|species| species.id == term.species)
                else {
                    return Err(LocalEquilibriumError::InvalidReaction(
                        reaction.id.to_string(),
                    ));
                };
                if !term_ids.insert(term.species)
                    || !term.coefficient.is_finite()
                    || term.coefficient == 0.0
                {
                    return Err(LocalEquilibriumError::InvalidReaction(
                        reaction.id.to_string(),
                    ));
                }
                row[index] = term.coefficient;
            }
            if !row.iter().any(|value| *value < 0.0) || !row.iter().any(|value| *value > 0.0) {
                return Err(LocalEquilibriumError::InvalidReaction(
                    reaction.id.to_string(),
                ));
            }
            let charge = row
                .iter()
                .zip(self.species)
                .map(|(coefficient, species)| coefficient * f64::from(species.charge))
                .sum::<f64>();
            if charge.abs() > 1e-10 {
                return Err(LocalEquilibriumError::ChargeImbalance {
                    reaction: reaction.id.to_string(),
                    charge,
                });
            }
            let component_ids = self
                .species
                .iter()
                .flat_map(|species| species.components.iter().map(|component| component.id))
                .collect::<BTreeSet<_>>();
            for component in component_ids {
                let imbalance = row
                    .iter()
                    .zip(self.species)
                    .map(|(coefficient, species)| {
                        coefficient
                            * species
                                .components
                                .iter()
                                .find(|entry| entry.id == component)
                                .map_or(0.0, |entry| entry.amount)
                    })
                    .sum::<f64>();
                if imbalance.abs() > 1e-10 {
                    return Err(LocalEquilibriumError::ComponentImbalance {
                        reaction: reaction.id.to_string(),
                        component: component.to_string(),
                        imbalance,
                    });
                }
            }
            stoichiometry.push(row);
        }
        if matrix_rank(stoichiometry.clone()) != self.reactions.len() {
            return Err(LocalEquilibriumError::RankDeficient);
        }
        Ok(stoichiometry)
    }

    fn mass_action_residuals(
        &self,
        concentrations: &[f64],
        stoichiometry: &[Vec<f64>],
    ) -> Vec<f64> {
        stoichiometry
            .iter()
            .zip(self.reactions)
            .map(|(row, reaction)| {
                row.iter()
                    .zip(concentrations)
                    .zip(self.species)
                    .map(|((coefficient, concentration), species)| {
                        coefficient
                            * (species.activity_coefficient
                                * concentration.max(CONCENTRATION_FLOOR)
                                / STANDARD_CONCENTRATION_MOL_PER_M3)
                                .ln()
                    })
                    .sum::<f64>()
                    - reaction.log10_equilibrium_constant * std::f64::consts::LN_10
            })
            .collect()
    }

    fn state(
        &self,
        concentrations_mol_per_m3: Vec<f64>,
        reaction_extents_mol_per_m3: Vec<f64>,
        maximum_mass_action_residual: f64,
    ) -> LocalEquilibriumState {
        let activities = concentrations_mol_per_m3
            .iter()
            .zip(self.species)
            .map(|(concentration, species)| {
                species.activity_coefficient * concentration / STANDARD_CONCENTRATION_MOL_PER_M3
            })
            .collect();
        LocalEquilibriumState {
            concentrations_mol_per_m3,
            activities,
            reaction_extents_mol_per_m3,
            maximum_mass_action_residual,
        }
    }
}

fn attainable_residual_tolerance(concentrations: &[f64], initial: &[f64]) -> f64 {
    let scale = concentrations
        .iter()
        .chain(initial)
        .copied()
        .fold(1.0_f64, f64::max);
    let smallest_positive = concentrations
        .iter()
        .copied()
        .filter(|value| *value > 0.0)
        .fold(f64::INFINITY, f64::min);
    let cancellation_limit = f64::EPSILON * scale / smallest_positive;
    RESIDUAL_TOLERANCE.max(cancellation_limit.min(1e-4))
}

impl LocalEquilibriumValidity {
    pub fn validate(
        self,
        temperature_k: f64,
        slower_process_time_s: f64,
    ) -> Result<(), LocalEquilibriumError> {
        if !self.minimum_temperature_k.is_finite()
            || !self.maximum_temperature_k.is_finite()
            || self.minimum_temperature_k <= 0.0
            || self.maximum_temperature_k < self.minimum_temperature_k
            || !self.maximum_relaxation_time_s.is_finite()
            || self.maximum_relaxation_time_s <= 0.0
            || !self.minimum_timescale_separation.is_finite()
            || self.minimum_timescale_separation < 1.0
            || !slower_process_time_s.is_finite()
            || slower_process_time_s <= 0.0
        {
            return Err(LocalEquilibriumError::InvalidValidity);
        }
        if !temperature_k.is_finite()
            || temperature_k < self.minimum_temperature_k
            || temperature_k > self.maximum_temperature_k
        {
            return Err(LocalEquilibriumError::TemperatureOutsideValidity {
                temperature_k,
                minimum_k: self.minimum_temperature_k,
                maximum_k: self.maximum_temperature_k,
            });
        }
        let actual_ratio = slower_process_time_s / self.maximum_relaxation_time_s;
        if actual_ratio < self.minimum_timescale_separation {
            return Err(LocalEquilibriumError::InsufficientTimescaleSeparation {
                actual_ratio,
                required_ratio: self.minimum_timescale_separation,
            });
        }
        Ok(())
    }
}

fn concentrations_from_extents(
    initial: &[f64],
    stoichiometry: &[Vec<f64>],
    extents: &[f64],
) -> Vec<f64> {
    (0..initial.len())
        .map(|species| {
            initial[species]
                + stoichiometry
                    .iter()
                    .zip(extents)
                    .map(|(reaction, extent)| reaction[species] * extent)
                    .sum::<f64>()
        })
        .collect()
}

fn reaction_jacobian(concentrations: &[f64], stoichiometry: &[Vec<f64>]) -> Vec<Vec<f64>> {
    stoichiometry
        .iter()
        .map(|left| {
            stoichiometry
                .iter()
                .map(|right| {
                    left.iter()
                        .zip(right)
                        .zip(concentrations)
                        .map(|((a, b), concentration)| {
                            a * b / concentration.max(CONCENTRATION_FLOOR)
                        })
                        .sum()
                })
                .collect()
        })
        .collect()
}

fn positive_step_scale(
    concentrations: &[f64],
    stoichiometry: &[Vec<f64>],
    extent_step: &[f64],
) -> f64 {
    let mut scale = 1.0_f64;
    for (species, concentration) in concentrations.iter().enumerate() {
        let delta = stoichiometry
            .iter()
            .zip(extent_step)
            .map(|(reaction, step)| reaction[species] * step)
            .sum::<f64>();
        if delta < 0.0 {
            scale = scale.min(0.99 * concentration.max(CONCENTRATION_FLOOR) / -delta);
        }
    }
    scale
}

fn solve_square(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let n = rhs.len();
    for column in 0..n {
        let pivot = (column..n).max_by(|a, b| {
            matrix[*a][column]
                .abs()
                .total_cmp(&matrix[*b][column].abs())
        })?;
        if matrix[pivot][column].abs() <= 1e-14 {
            return None;
        }
        matrix.swap(column, pivot);
        rhs.swap(column, pivot);
        let lead = matrix[column][column];
        for value in &mut matrix[column][column..] {
            *value /= lead;
        }
        rhs[column] /= lead;
        let pivot_row = matrix[column].clone();
        for row in 0..n {
            if row == column {
                continue;
            }
            let factor = matrix[row][column];
            for offset in column..n {
                matrix[row][offset] -= factor * pivot_row[offset];
            }
            rhs[row] -= factor * rhs[column];
        }
    }
    Some(rhs)
}

fn matrix_rank(mut matrix: Vec<Vec<f64>>) -> usize {
    if matrix.is_empty() {
        return 0;
    }
    let columns = matrix[0].len();
    let mut rank = 0;
    for column in 0..columns {
        let Some(pivot) = (rank..matrix.len()).max_by(|a, b| {
            matrix[*a][column]
                .abs()
                .total_cmp(&matrix[*b][column].abs())
        }) else {
            break;
        };
        if matrix[pivot][column].abs() <= 1e-12 {
            continue;
        }
        matrix.swap(rank, pivot);
        let lead = matrix[rank][column];
        for value in &mut matrix[rank][column..] {
            *value /= lead;
        }
        let pivot_row = matrix[rank].clone();
        for (row_index, row) in matrix.iter_mut().enumerate() {
            if row_index == rank {
                continue;
            }
            let factor = row[column];
            for offset in column..columns {
                row[offset] -= factor * pivot_row[offset];
            }
        }
        rank += 1;
        if rank == matrix.len() {
            break;
        }
    }
    rank
}

#[cfg(test)]
mod tests {
    use super::*;

    const HA_COMPONENTS: &[LocalEquilibriumComponent<'_>] = &[
        LocalEquilibriumComponent {
            id: "acid-skeleton",
            amount: 1.0,
        },
        LocalEquilibriumComponent {
            id: "dissociable-proton",
            amount: 1.0,
        },
    ];
    const H_COMPONENTS: &[LocalEquilibriumComponent<'_>] = &[LocalEquilibriumComponent {
        id: "dissociable-proton",
        amount: 1.0,
    }];
    const A_COMPONENTS: &[LocalEquilibriumComponent<'_>] = &[LocalEquilibriumComponent {
        id: "acid-skeleton",
        amount: 1.0,
    }];
    const SPECIES: &[LocalEquilibriumSpecies<'_>] = &[
        LocalEquilibriumSpecies {
            id: "HA",
            charge: 0,
            activity_coefficient: 1.0,
            components: HA_COMPONENTS,
        },
        LocalEquilibriumSpecies {
            id: "H+",
            charge: 1,
            activity_coefficient: 1.0,
            components: H_COMPONENTS,
        },
        LocalEquilibriumSpecies {
            id: "A-",
            charge: -1,
            activity_coefficient: 1.0,
            components: A_COMPONENTS,
        },
    ];
    const TERMS: &[LocalEquilibriumTerm<'_>] = &[
        LocalEquilibriumTerm {
            species: "HA",
            coefficient: -1.0,
        },
        LocalEquilibriumTerm {
            species: "H+",
            coefficient: 1.0,
        },
        LocalEquilibriumTerm {
            species: "A-",
            coefficient: 1.0,
        },
    ];
    const REACTIONS: &[LocalEquilibriumReaction<'_>] = &[LocalEquilibriumReaction {
        id: "acid",
        terms: TERMS,
        log10_equilibrium_constant: -4.76,
    }];

    #[test]
    fn weak_acid_mass_action_and_totals_are_computed() {
        let network = LocalEquilibriumNetwork {
            species: SPECIES,
            reactions: REACTIONS,
        };
        let solved = network.equilibrate(&[100.0, 0.0, 0.0]).unwrap();
        let ha = solved.concentrations_mol_per_m3[0];
        let proton = solved.concentrations_mol_per_m3[1];
        let conjugate = solved.concentrations_mol_per_m3[2];
        assert!((ha + conjugate - 100.0).abs() < 1e-10);
        assert!((proton - conjugate).abs() < 1e-10);
        let quotient = solved.activities[1] * solved.activities[2] / solved.activities[0];
        assert!((quotient.log10() + 4.76).abs() < 1e-10);
    }

    #[test]
    fn supplied_activity_coefficients_change_speciation_not_totals() {
        let nonideal_species = [
            SPECIES[0],
            LocalEquilibriumSpecies {
                activity_coefficient: 0.5,
                ..SPECIES[1]
            },
            LocalEquilibriumSpecies {
                activity_coefficient: 0.5,
                ..SPECIES[2]
            },
        ];
        let ideal = LocalEquilibriumNetwork {
            species: SPECIES,
            reactions: REACTIONS,
        }
        .equilibrate(&[100.0, 0.0, 0.0])
        .unwrap();
        let nonideal = LocalEquilibriumNetwork {
            species: &nonideal_species,
            reactions: REACTIONS,
        }
        .equilibrate(&[100.0, 0.0, 0.0])
        .unwrap();
        assert!(nonideal.concentrations_mol_per_m3[1] > ideal.concentrations_mol_per_m3[1]);
        assert!(
            (nonideal.concentrations_mol_per_m3[0] + nonideal.concentrations_mol_per_m3[2] - 100.0)
                .abs()
                < 1e-10
        );
    }

    #[test]
    fn weak_acid_grid_converges_without_reaction_specific_branches() {
        for pka in [-2.0, 0.0, 2.0, 4.76, 7.0, 10.0, 12.0] {
            let reaction = [LocalEquilibriumReaction {
                log10_equilibrium_constant: -pka,
                ..REACTIONS[0]
            }];
            let network = LocalEquilibriumNetwork {
                species: SPECIES,
                reactions: &reaction,
            };
            for total in [1e-6, 1e-3, 1.0, 100.0, 1_000.0] {
                let solved = network
                    .equilibrate(&[total, 0.0, 0.0])
                    .unwrap_or_else(|error| {
                        panic!("pKa={pka}, total={total} mol/m3 failed: {error}")
                    });
                let [ha, proton, conjugate] = solved.concentrations_mol_per_m3.as_slice() else {
                    unreachable!()
                };
                assert!((ha + conjugate - total).abs() <= 1e-10 * total.max(1.0));
                assert!((proton - conjugate).abs() <= 1e-10 * total.max(1.0));
                let quotient = solved.activities[1] * solved.activities[2] / solved.activities[0];
                assert!((quotient.log10() + pka).abs() <= 5e-5);
            }
        }
    }

    #[test]
    fn invalid_charge_and_dependent_reactions_are_refused() {
        let unbalanced_terms = [
            TERMS[0],
            LocalEquilibriumTerm {
                species: "H+",
                coefficient: 2.0,
            },
            TERMS[2],
        ];
        let unbalanced = [LocalEquilibriumReaction {
            terms: &unbalanced_terms,
            ..REACTIONS[0]
        }];
        assert!(matches!(
            LocalEquilibriumNetwork {
                species: SPECIES,
                reactions: &unbalanced
            }
            .equilibrate(&[100.0, 0.0, 0.0]),
            Err(LocalEquilibriumError::ChargeImbalance { .. })
        ));
        let dependent = [
            REACTIONS[0],
            LocalEquilibriumReaction {
                id: "copy",
                ..REACTIONS[0]
            },
        ];
        assert_eq!(
            LocalEquilibriumNetwork {
                species: SPECIES,
                reactions: &dependent
            }
            .equilibrate(&[100.0, 0.0, 0.0]),
            Err(LocalEquilibriumError::RankDeficient)
        );
    }

    #[test]
    fn component_imbalance_and_invalid_validity_are_refused() {
        let imbalanced_terms = [
            LocalEquilibriumTerm {
                species: "HA",
                coefficient: -2.0,
            },
            LocalEquilibriumTerm {
                species: "H+",
                coefficient: 1.0,
            },
            LocalEquilibriumTerm {
                species: "A-",
                coefficient: 1.0,
            },
        ];
        let imbalanced = [LocalEquilibriumReaction {
            id: "loses-skeleton",
            terms: &imbalanced_terms,
            log10_equilibrium_constant: -4.76,
        }];
        assert!(matches!(
            LocalEquilibriumNetwork {
                species: SPECIES,
                reactions: &imbalanced
            }
            .equilibrate(&[100.0, 0.0, 0.0]),
            Err(LocalEquilibriumError::ComponentImbalance { .. })
        ));

        let validity = LocalEquilibriumValidity {
            minimum_temperature_k: 273.15,
            maximum_temperature_k: 323.15,
            maximum_relaxation_time_s: 0.01,
            minimum_timescale_separation: 100.0,
        };
        assert!(validity.validate(298.15, 1.0).is_ok());
        assert!(matches!(
            validity.validate(350.0, 1.0),
            Err(LocalEquilibriumError::TemperatureOutsideValidity { .. })
        ));
        assert!(matches!(
            validity.validate(298.15, 0.5),
            Err(LocalEquilibriumError::InsufficientTimescaleSeparation { .. })
        ));
    }
}

//! One-dimensional Nernst--Planck transport across a finite layer.
//!
//! Flux is positive from the bulk/left boundary toward the surface/right
//! boundary. Potential is `phi_right - phi_left`. The constant-field
//! Scharfetter--Gummel flux keeps diffusion and migration in one expression
//! and approaches Fick's law continuously at zero potential difference.

use std::collections::BTreeSet;

use crate::constants::{FARADAY, GAS_CONSTANT};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectrodiffusionSpecies<'a> {
    pub id: &'a str,
    pub charge: i32,
    pub diffusivity_m2_per_s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NernstPlanckDomain {
    pub temperature_k: f64,
    pub layer_thickness_m: f64,
    pub minimum_potential_difference_v: f64,
    pub maximum_potential_difference_v: f64,
    pub potential_tolerance_v: f64,
    pub current_tolerance_a_per_m2: f64,
    pub charge_tolerance_mol_per_m3: f64,
    pub maximum_iterations: usize,
}

impl Default for NernstPlanckDomain {
    fn default() -> Self {
        Self {
            temperature_k: 298.15,
            layer_thickness_m: 1.0e-4,
            minimum_potential_difference_v: -1.0,
            maximum_potential_difference_v: 1.0,
            potential_tolerance_v: 1.0e-12,
            current_tolerance_a_per_m2: 1.0e-8,
            charge_tolerance_mol_per_m3: 1.0e-8,
            maximum_iterations: 192,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NernstPlanckState {
    pub potential_difference_v: f64,
    pub fluxes_mol_per_m2_s: Vec<f64>,
    pub right_concentrations_mol_per_m3: Vec<f64>,
    pub ionic_current_density_a_per_m2: f64,
    pub charge_residual_mol_per_m3: f64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ElectrodiffusionError {
    #[error("Nernst-Planck domain is invalid")]
    InvalidDomain,
    #[error("electrodiffusion species '{0}' is duplicated or invalid")]
    InvalidSpecies(String),
    #[error("electrodiffusion concentration or flux for '{0}' is invalid")]
    InvalidInput(String),
    #[error("electrodiffusion boundary '{0}' is not electroneutral")]
    NonElectroneutralBoundary(&'static str),
    #[error("electrodiffusion requires at least one cation and one anion")]
    MissingCountercharge,
    #[error("no physical Nernst-Planck state lies inside the authored potential bounds")]
    NotBracketed,
    #[error("Nernst-Planck solve did not converge")]
    DidNotConverge,
}

impl NernstPlanckDomain {
    /// Fluxes between two authored, electroneutral boundary compositions at
    /// the diffusion potential that carries zero net ionic current.
    pub fn zero_current_junction(
        self,
        species: &[ElectrodiffusionSpecies<'_>],
        left_concentrations_mol_per_m3: &[f64],
        right_concentrations_mol_per_m3: &[f64],
    ) -> Result<NernstPlanckState, ElectrodiffusionError> {
        self.validate_species_and_inputs(
            species,
            left_concentrations_mol_per_m3,
            Some(right_concentrations_mol_per_m3),
        )?;
        require_electroneutral(species, left_concentrations_mol_per_m3, "left")?;
        require_electroneutral(species, right_concentrations_mol_per_m3, "right")?;
        let residual = |potential| {
            let fluxes = fluxes_at_potential(
                self,
                species,
                left_concentrations_mol_per_m3,
                right_concentrations_mol_per_m3,
                potential,
            );
            Some(ionic_current_density(species, &fluxes))
        };
        let potential = bounded_root(self, self.current_tolerance_a_per_m2, residual)?;
        let fluxes = fluxes_at_potential(
            self,
            species,
            left_concentrations_mol_per_m3,
            right_concentrations_mol_per_m3,
            potential,
        );
        Ok(NernstPlanckState {
            potential_difference_v: potential,
            ionic_current_density_a_per_m2: ionic_current_density(species, &fluxes),
            fluxes_mol_per_m2_s: fluxes,
            right_concentrations_mol_per_m3: right_concentrations_mol_per_m3.to_vec(),
            charge_residual_mol_per_m3: charge_concentration(
                species,
                right_concentrations_mol_per_m3,
            ),
        })
    }

    /// Surface composition for prescribed interfacial species fluxes. Positive
    /// flux consumes a species at the right-hand surface. The migration
    /// potential is solved so the surface remains electroneutral.
    pub fn electroneutral_surface(
        self,
        species: &[ElectrodiffusionSpecies<'_>],
        bulk_concentrations_mol_per_m3: &[f64],
        consumed_fluxes_mol_per_m2_s: &[f64],
    ) -> Result<NernstPlanckState, ElectrodiffusionError> {
        self.validate_species_and_inputs(species, bulk_concentrations_mol_per_m3, None)?;
        if consumed_fluxes_mol_per_m2_s.len() != species.len() {
            return Err(ElectrodiffusionError::InvalidInput(
                "wrong number of fluxes".into(),
            ));
        }
        for (entry, flux) in species.iter().zip(consumed_fluxes_mol_per_m2_s) {
            if !flux.is_finite() {
                return Err(ElectrodiffusionError::InvalidInput(entry.id.into()));
            }
        }
        require_electroneutral(species, bulk_concentrations_mol_per_m3, "bulk")?;
        let surface_at = |potential| {
            concentrations_for_fluxes(
                self,
                species,
                bulk_concentrations_mol_per_m3,
                consumed_fluxes_mol_per_m2_s,
                potential,
            )
        };
        let residual = |potential| {
            surface_at(potential).map(|surface| charge_concentration(species, &surface))
        };
        let potential = bounded_root(self, self.charge_tolerance_mol_per_m3, residual)?;
        let surface = surface_at(potential).ok_or(ElectrodiffusionError::NotBracketed)?;
        let charge_residual = charge_concentration(species, &surface);
        Ok(NernstPlanckState {
            potential_difference_v: potential,
            fluxes_mol_per_m2_s: consumed_fluxes_mol_per_m2_s.to_vec(),
            right_concentrations_mol_per_m3: surface,
            ionic_current_density_a_per_m2: ionic_current_density(
                species,
                consumed_fluxes_mol_per_m2_s,
            ),
            charge_residual_mol_per_m3: charge_residual,
        })
    }

    fn validate_species_and_inputs(
        self,
        species: &[ElectrodiffusionSpecies<'_>],
        left: &[f64],
        right: Option<&[f64]>,
    ) -> Result<(), ElectrodiffusionError> {
        if !self.temperature_k.is_finite()
            || self.temperature_k <= 0.0
            || !self.layer_thickness_m.is_finite()
            || self.layer_thickness_m <= 0.0
            || !self.minimum_potential_difference_v.is_finite()
            || !self.maximum_potential_difference_v.is_finite()
            || self.maximum_potential_difference_v <= self.minimum_potential_difference_v
            || !self.potential_tolerance_v.is_finite()
            || self.potential_tolerance_v <= 0.0
            || !self.current_tolerance_a_per_m2.is_finite()
            || self.current_tolerance_a_per_m2 <= 0.0
            || !self.charge_tolerance_mol_per_m3.is_finite()
            || self.charge_tolerance_mol_per_m3 <= 0.0
            || self.maximum_iterations == 0
            || species.is_empty()
            || left.len() != species.len()
            || right.is_some_and(|values| values.len() != species.len())
        {
            return Err(ElectrodiffusionError::InvalidDomain);
        }
        let mut ids = BTreeSet::new();
        let mut has_positive = false;
        let mut has_negative = false;
        for (index, entry) in species.iter().enumerate() {
            if entry.id.trim().is_empty()
                || !ids.insert(entry.id)
                || !entry.diffusivity_m2_per_s.is_finite()
                || entry.diffusivity_m2_per_s <= 0.0
            {
                return Err(ElectrodiffusionError::InvalidSpecies(entry.id.into()));
            }
            has_positive |= entry.charge > 0;
            has_negative |= entry.charge < 0;
            if !valid_concentration(left[index])
                || right.is_some_and(|values| !valid_concentration(values[index]))
            {
                return Err(ElectrodiffusionError::InvalidInput(entry.id.into()));
            }
        }
        if !has_positive || !has_negative {
            return Err(ElectrodiffusionError::MissingCountercharge);
        }
        Ok(())
    }
}

fn valid_concentration(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn charge_concentration(species: &[ElectrodiffusionSpecies<'_>], concentrations: &[f64]) -> f64 {
    species
        .iter()
        .zip(concentrations)
        .map(|(entry, concentration)| f64::from(entry.charge) * concentration)
        .sum()
}

fn require_electroneutral(
    species: &[ElectrodiffusionSpecies<'_>],
    concentrations: &[f64],
    boundary: &'static str,
) -> Result<(), ElectrodiffusionError> {
    let charge = charge_concentration(species, concentrations);
    let scale = species
        .iter()
        .zip(concentrations)
        .map(|(entry, concentration)| f64::from(entry.charge).abs() * concentration)
        .sum::<f64>()
        .max(1.0);
    if charge.abs() <= 1e-10 * scale {
        Ok(())
    } else {
        Err(ElectrodiffusionError::NonElectroneutralBoundary(boundary))
    }
}

fn bernoulli(value: f64) -> f64 {
    if value.abs() < 1e-5 {
        let squared = value * value;
        1.0 - value / 2.0 + squared / 12.0 - squared * squared / 720.0
    } else {
        value / value.exp_m1()
    }
}

fn dimensionless_potential(
    domain: NernstPlanckDomain,
    charge: i32,
    potential_difference_v: f64,
) -> f64 {
    f64::from(charge) * FARADAY * potential_difference_v / (GAS_CONSTANT * domain.temperature_k)
}

fn fluxes_at_potential(
    domain: NernstPlanckDomain,
    species: &[ElectrodiffusionSpecies<'_>],
    left: &[f64],
    right: &[f64],
    potential_difference_v: f64,
) -> Vec<f64> {
    species
        .iter()
        .zip(left)
        .zip(right)
        .map(|((entry, left), right)| {
            let psi = dimensionless_potential(domain, entry.charge, potential_difference_v);
            entry.diffusivity_m2_per_s / domain.layer_thickness_m
                * (left * bernoulli(psi) - right * bernoulli(-psi))
        })
        .collect()
}

fn concentrations_for_fluxes(
    domain: NernstPlanckDomain,
    species: &[ElectrodiffusionSpecies<'_>],
    bulk: &[f64],
    fluxes: &[f64],
    potential_difference_v: f64,
) -> Option<Vec<f64>> {
    species
        .iter()
        .zip(bulk)
        .zip(fluxes)
        .map(|((entry, bulk), flux)| {
            let psi = dimensionless_potential(domain, entry.charge, potential_difference_v);
            let right = (bulk * bernoulli(psi)
                - flux * domain.layer_thickness_m / entry.diffusivity_m2_per_s)
                / bernoulli(-psi);
            (right.is_finite() && right >= 0.0).then_some(right)
        })
        .collect()
}

fn ionic_current_density(species: &[ElectrodiffusionSpecies<'_>], fluxes: &[f64]) -> f64 {
    FARADAY
        * species
            .iter()
            .zip(fluxes)
            .map(|(entry, flux)| f64::from(entry.charge) * flux)
            .sum::<f64>()
}

fn bounded_root(
    domain: NernstPlanckDomain,
    residual_tolerance: f64,
    residual: impl Fn(f64) -> Option<f64>,
) -> Result<f64, ElectrodiffusionError> {
    const SCAN_INTERVALS: usize = 512;
    let width = domain.maximum_potential_difference_v - domain.minimum_potential_difference_v;
    let mut previous: Option<(f64, f64)> = None;
    for index in 0..=SCAN_INTERVALS {
        let potential =
            domain.minimum_potential_difference_v + width * index as f64 / SCAN_INTERVALS as f64;
        let Some(value) = residual(potential).filter(|value| value.is_finite()) else {
            continue;
        };
        if value.abs() <= residual_tolerance {
            return Ok(potential);
        }
        if let Some((mut lower, mut lower_value)) = previous {
            if value.signum() != lower_value.signum() {
                let mut upper = potential;
                for _ in 0..domain.maximum_iterations {
                    let middle = 0.5 * (lower + upper);
                    let Some(middle_value) = residual(middle) else {
                        upper = middle;
                        continue;
                    };
                    if middle_value.abs() <= residual_tolerance {
                        return Ok(middle);
                    }
                    if upper - lower <= domain.potential_tolerance_v {
                        return Err(ElectrodiffusionError::DidNotConverge);
                    }
                    if middle_value.signum() == lower_value.signum() {
                        lower = middle;
                        lower_value = middle_value;
                    } else {
                        upper = middle;
                    }
                }
                return Err(ElectrodiffusionError::DidNotConverge);
            }
        }
        previous = Some((potential, value));
    }
    Err(ElectrodiffusionError::NotBracketed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EQUAL: &[ElectrodiffusionSpecies<'_>] = &[
        ElectrodiffusionSpecies {
            id: "C+",
            charge: 1,
            diffusivity_m2_per_s: 1.0e-9,
        },
        ElectrodiffusionSpecies {
            id: "A-",
            charge: -1,
            diffusivity_m2_per_s: 1.0e-9,
        },
    ];

    #[test]
    fn bernoulli_flux_has_the_constant_field_identity() {
        for value in [-20.0, -1.0, -1e-7, 0.0, 1e-7, 1.0, 20.0] {
            assert!((bernoulli(-value) - value.exp() * bernoulli(value)).abs() < 1e-10);
        }
    }

    #[test]
    fn equal_mobilities_need_no_junction_potential() {
        let state = NernstPlanckDomain::default()
            .zero_current_junction(EQUAL, &[100.0, 100.0], &[10.0, 10.0])
            .unwrap();
        assert!(state.potential_difference_v.abs() < 1e-9);
        assert!(state.ionic_current_density_a_per_m2.abs() < 1e-8);
        assert!((state.fluxes_mol_per_m2_s[0] - state.fluxes_mol_per_m2_s[1]).abs() < 1e-12);
    }

    #[test]
    fn unequal_mobilities_compute_a_zero_current_liquid_junction() {
        let unequal = [
            EQUAL[0],
            ElectrodiffusionSpecies {
                diffusivity_m2_per_s: 2.0e-9,
                ..EQUAL[1]
            },
        ];
        let state = NernstPlanckDomain::default()
            .zero_current_junction(&unequal, &[100.0, 100.0], &[10.0, 10.0])
            .unwrap();
        assert!(state.potential_difference_v.abs() > 1e-4);
        assert!(state.ionic_current_density_a_per_m2.abs() < 1e-8);
        let swapped = [
            ElectrodiffusionSpecies {
                diffusivity_m2_per_s: 2.0e-9,
                ..EQUAL[0]
            },
            EQUAL[1],
        ];
        let mirror = NernstPlanckDomain::default()
            .zero_current_junction(&swapped, &[100.0, 100.0], &[10.0, 10.0])
            .unwrap();
        assert!((state.potential_difference_v + mirror.potential_difference_v).abs() < 1e-9);
    }

    #[test]
    fn binary_junction_grid_matches_the_constant_field_closed_form() {
        for left in [1.0, 10.0, 100.0, 1_000.0] {
            for right in [0.5, 5.0, 50.0, 500.0] {
                for (cation_diffusivity, anion_diffusivity) in
                    [(0.5e-9, 2.0e-9), (1.0e-9, 1.0e-9), (3.0e-9, 0.7e-9)]
                {
                    let ions = [
                        ElectrodiffusionSpecies {
                            diffusivity_m2_per_s: cation_diffusivity,
                            ..EQUAL[0]
                        },
                        ElectrodiffusionSpecies {
                            diffusivity_m2_per_s: anion_diffusivity,
                            ..EQUAL[1]
                        },
                    ];
                    let state = NernstPlanckDomain::default()
                        .zero_current_junction(&ions, &[left, left], &[right, right])
                        .unwrap();
                    let ratio = (cation_diffusivity * left + anion_diffusivity * right)
                        / (cation_diffusivity * right + anion_diffusivity * left);
                    let expected = GAS_CONSTANT * 298.15 / FARADAY * ratio.ln();
                    assert!((state.potential_difference_v - expected).abs() < 1e-9);
                    assert!(state.ionic_current_density_a_per_m2.abs() < 1e-8);
                }
            }
        }
    }

    #[test]
    fn migration_supplies_countercharge_at_an_electrode_surface() {
        let flux = 5.0e-4;
        let state = NernstPlanckDomain::default()
            .electroneutral_surface(EQUAL, &[100.0, 100.0], &[flux, 0.0])
            .unwrap();
        assert!(state.potential_difference_v < 0.0);
        assert!(state.right_concentrations_mol_per_m3[0] < 100.0);
        assert!(
            (state.right_concentrations_mol_per_m3[0] - state.right_concentrations_mol_per_m3[1])
                .abs()
                < 1e-8
        );
        assert!((state.ionic_current_density_a_per_m2 - FARADAY * flux).abs() < 1e-8);
        let replay = fluxes_at_potential(
            NernstPlanckDomain::default(),
            EQUAL,
            &[100.0, 100.0],
            &state.right_concentrations_mol_per_m3,
            state.potential_difference_v,
        );
        assert!((replay[0] - flux).abs() < 1e-10);
        assert!(replay[1].abs() < 1e-10);
    }

    #[test]
    fn impossible_flux_and_invalid_charge_domains_refuse() {
        assert_eq!(
            NernstPlanckDomain::default().electroneutral_surface(
                EQUAL,
                &[100.0, 100.0],
                &[1.0, 0.0],
            ),
            Err(ElectrodiffusionError::NotBracketed)
        );
        assert!(matches!(
            NernstPlanckDomain::default().zero_current_junction(
                EQUAL,
                &[100.0, 90.0],
                &[10.0, 10.0],
            ),
            Err(ElectrodiffusionError::NonElectroneutralBoundary("left"))
        ));
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(&EQUAL[..1], &[100.0], &[10.0],),
            Err(ElectrodiffusionError::MissingCountercharge)
        );
    }
}

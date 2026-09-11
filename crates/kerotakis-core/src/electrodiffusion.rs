//! One-dimensional Nernst--Planck transport across a finite layer.
//!
//! Flux is positive from the bulk/left boundary toward the surface/right
//! boundary. Potential is `phi_right - phi_left`. The constant-field
//! Scharfetter--Gummel flux keeps diffusion and migration in one expression
//! and approaches Fick's law continuously at zero potential difference.
//!
//! The bounded solver converges once its residual falls inside the authored
//! residual tolerance. Reaching that costs a bracket of roughly
//! `residual_tolerance / |d residual / d phi|` volts, and the slope of the
//! ionic current grows with `(D / L) * c`, so the fastest ion, the thinnest
//! layer and the strongest solution set the resolution a case needs. The
//! bracket is therefore floored by the spacing of `f64` near the root and not
//! by a constant, which keeps the working range attached to the physics.

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
    /// Optional bracket floor for the bisection, in volts. Convergence needs
    /// a bracket of about `current_tolerance_a_per_m2 / |dI/dphi|`, and
    /// `dI/dphi` grows with `(D / L) * c`, so a constant floor silently caps
    /// the working range at whichever concentration, diffusivity and layer
    /// thickness the constant was chosen for. `None` lets bisection run down
    /// to the spacing of `f64` near the root, so the range follows the case.
    /// `Some(volts)` pins a coarser floor for a caller who would rather be
    /// refused early than iterate.
    pub potential_tolerance_v: Option<f64>,
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
            potential_tolerance_v: None,
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
            || self
                .potential_tolerance_v
                .is_some_and(|floor| !floor.is_finite() || floor <= 0.0)
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
                    if middle <= lower || middle >= upper {
                        // The bracket is one `f64` wide: no further halving
                        // can move the residual, so the caller has asked for a
                        // residual finer than the root can be resolved to.
                        return Err(ElectrodiffusionError::DidNotConverge);
                    }
                    let Some(middle_value) = residual(middle) else {
                        upper = middle;
                        continue;
                    };
                    if middle_value.abs() <= residual_tolerance {
                        return Ok(middle);
                    }
                    if domain
                        .potential_tolerance_v
                        .is_some_and(|floor| upper - lower <= floor)
                    {
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

    /// Slope of the ionic current in the potential. The bisection stops when
    /// the current is inside its tolerance, so the potential that tolerance
    /// buys is `current_tolerance_a_per_m2 / |dI/dphi|` -- which is what makes
    /// the solver's working range a property of the case rather than a
    /// constant.
    fn current_slope(
        domain: NernstPlanckDomain,
        ions: &[ElectrodiffusionSpecies<'_>],
        left: &[f64],
        right: &[f64],
        potential: f64,
    ) -> f64 {
        let step = 1.0e-8;
        let above = ionic_current_density(
            ions,
            &fluxes_at_potential(domain, ions, left, right, potential + step),
        );
        let below = ionic_current_density(
            ions,
            &fluxes_at_potential(domain, ions, left, right, potential - step),
        );
        (above - below) / (2.0 * step)
    }

    #[test]
    fn binary_junction_grid_matches_the_constant_field_closed_form() {
        // The grid deliberately runs past what a bench reaches, because a grid
        // that stops where the solver stops is not evidence of a working
        // range: two molar, the 9.31e-9 m^2/s of the hydrogen ion, and a ten
        // micrometre diffusion layer are all inside it.
        let mut worst_ratio: f64 = 0.0;
        for layer_thickness_m in [1.0e-4, 1.0e-5] {
            for left in [1.0, 10.0, 100.0, 1_000.0, 2_000.0] {
                for right in [0.5, 5.0, 50.0, 500.0] {
                    for (cation_diffusivity, anion_diffusivity) in [
                        (0.5e-9, 2.0e-9),
                        (1.0e-9, 1.0e-9),
                        (3.0e-9, 0.7e-9),
                        (5.0e-9, 0.7e-9),
                        (9.31e-9, 2.03e-9),
                    ] {
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
                        let domain = NernstPlanckDomain {
                            layer_thickness_m,
                            ..Default::default()
                        };
                        let state = domain
                            .zero_current_junction(&ions, &[left, left], &[right, right])
                            .unwrap_or_else(|error| {
                                panic!(
                                    "L={layer_thickness_m} {left}->{right} \
                                     D={cation_diffusivity}/{anion_diffusivity}: {error}"
                                )
                            });
                        let ratio = (cation_diffusivity * left + anion_diffusivity * right)
                            / (cation_diffusivity * right + anion_diffusivity * left);
                        let expected = GAS_CONSTANT * 298.15 / FARADAY * ratio.ln();
                        let slope = current_slope(
                            domain,
                            &ions,
                            &[left, left],
                            &[right, right],
                            state.potential_difference_v,
                        );
                        let allowed =
                            2.0 * domain.current_tolerance_a_per_m2 / slope.abs() + 1.0e-14;
                        let error = (state.potential_difference_v - expected).abs();
                        worst_ratio = worst_ratio.max(error / allowed);
                        assert!(
                            error <= allowed,
                            "L={layer_thickness_m} {left}->{right} \
                             D={cation_diffusivity}/{anion_diffusivity}: \
                             {} vs {expected}, error {error} > {allowed}",
                            state.potential_difference_v
                        );
                        assert!(
                            state.ionic_current_density_a_per_m2.abs()
                                <= domain.current_tolerance_a_per_m2
                        );
                    }
                }
            }
        }
        // The bound is the real one, not a loose one: something on the grid
        // must come close to spending the whole budget.
        assert!(worst_ratio > 0.1, "accuracy bound is slack: {worst_ratio}");
    }

    #[test]
    fn the_fastest_aqueous_ions_and_bench_strengths_are_inside_the_working_range() {
        // Each of these sat one step past the edge of the original grid and
        // refused with `DidNotConverge` under the old constant 1e-12 bracket
        // floor, because the floor sat above the
        // `current_tolerance / |dI/dphi|` those cases need.
        let cliff = [
            ("two molar", 2_000.0, 500.0, 3.0e-9, 0.7e-9, 1.0e-4),
            ("fast cation", 1_000.0, 500.0, 5.0e-9, 0.7e-9, 1.0e-4),
            (
                "ten micrometre layer",
                1_000.0,
                500.0,
                3.0e-9,
                0.7e-9,
                1.0e-5,
            ),
            (
                "hydrogen ion at one molar",
                1_000.0,
                500.0,
                9.31e-9,
                2.03e-9,
                1.0e-4,
            ),
        ];
        for (name, left, right, cation_diffusivity, anion_diffusivity, layer_thickness_m) in cliff {
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
            let domain = NernstPlanckDomain {
                layer_thickness_m,
                ..Default::default()
            };
            let state = domain
                .zero_current_junction(&ions, &[left, left], &[right, right])
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            let ratio = (cation_diffusivity * left + anion_diffusivity * right)
                / (cation_diffusivity * right + anion_diffusivity * left);
            let expected = GAS_CONSTANT * 298.15 / FARADAY * ratio.ln();
            assert!(
                (state.potential_difference_v - expected).abs() < 1e-11,
                "{name}: {} vs {expected}",
                state.potential_difference_v
            );
            // Pin the defect: the old constant floor refuses every one of them.
            assert_eq!(
                NernstPlanckDomain {
                    potential_tolerance_v: Some(1.0e-12),
                    ..domain
                }
                .zero_current_junction(&ions, &[left, left], &[right, right]),
                Err(ElectrodiffusionError::DidNotConverge),
                "{name} no longer needs a finer bracket than 1e-12 V"
            );
        }
    }

    #[test]
    fn multi_ion_junction_matches_the_goldman_hodgkin_katz_voltage() {
        // With every ion monovalent the zero-current constant-field condition
        // has an exact closed form for any number of ions and any asymmetric
        // pair of electroneutral endpoints, so this oracle tests far more than
        // the binary symmetric case.
        let ions = [
            ElectrodiffusionSpecies {
                id: "H+",
                charge: 1,
                diffusivity_m2_per_s: 9.31e-9,
            },
            ElectrodiffusionSpecies {
                id: "Na+",
                charge: 1,
                diffusivity_m2_per_s: 1.33e-9,
            },
            ElectrodiffusionSpecies {
                id: "K+",
                charge: 1,
                diffusivity_m2_per_s: 1.96e-9,
            },
            ElectrodiffusionSpecies {
                id: "Cl-",
                charge: -1,
                diffusivity_m2_per_s: 2.03e-9,
            },
            ElectrodiffusionSpecies {
                id: "NO3-",
                charge: -1,
                diffusivity_m2_per_s: 1.90e-9,
            },
        ];
        let endpoints = [
            (
                [100.0, 400.0, 50.0, 300.0, 250.0],
                [10.0, 20.0, 500.0, 400.0, 130.0],
            ),
            (
                [1.0, 999.0, 0.0, 500.0, 500.0],
                [500.0, 0.0, 500.0, 10.0, 990.0],
            ),
            (
                [1_000.0, 500.0, 500.0, 1_800.0, 200.0],
                [20.0, 60.0, 20.0, 25.0, 75.0],
            ),
        ];
        for (left, right) in endpoints {
            let state = NernstPlanckDomain::default()
                .zero_current_junction(&ions, &left, &right)
                .unwrap();
            let mut numerator = 0.0;
            let mut denominator = 0.0;
            for (index, ion) in ions.iter().enumerate() {
                if ion.charge > 0 {
                    numerator += ion.diffusivity_m2_per_s * left[index];
                    denominator += ion.diffusivity_m2_per_s * right[index];
                } else {
                    numerator += ion.diffusivity_m2_per_s * right[index];
                    denominator += ion.diffusivity_m2_per_s * left[index];
                }
            }
            let expected = GAS_CONSTANT * 298.15 / FARADAY * (numerator / denominator).ln();
            assert!(
                (state.potential_difference_v - expected).abs() < 1e-11,
                "{left:?} -> {right:?}: {} vs {expected}",
                state.potential_difference_v
            );
            assert!(state.ionic_current_density_a_per_m2.abs() < 1e-8);
        }
    }

    #[test]
    fn divalent_junction_matches_the_two_to_one_closed_form() {
        // A 2:1 electrolyte has no Goldman form, but the zero-current
        // condition is a quadratic in `exp(F phi / R T)`, which is an exact
        // oracle for an asymmetric, multivalent case.
        for (cation_diffusivity, anion_diffusivity, left_cation, right_cation) in [
            (0.79e-9, 2.03e-9, 100.0, 10.0),
            (1.30e-9, 2.03e-9, 500.0, 5.0),
            (0.79e-9, 9.31e-9, 50.0, 700.0),
        ] {
            let ions = [
                ElectrodiffusionSpecies {
                    id: "M2+",
                    charge: 2,
                    diffusivity_m2_per_s: cation_diffusivity,
                },
                ElectrodiffusionSpecies {
                    id: "X-",
                    charge: -1,
                    diffusivity_m2_per_s: anion_diffusivity,
                },
            ];
            let (left_anion, right_anion) = (2.0 * left_cation, 2.0 * right_cation);
            let state = NernstPlanckDomain::default()
                .zero_current_junction(
                    &ions,
                    &[left_cation, left_anion],
                    &[right_cation, right_anion],
                )
                .unwrap();
            let quadratic =
                -(4.0 * cation_diffusivity * right_cation + anion_diffusivity * left_anion);
            let linear = -anion_diffusivity * (left_anion - right_anion);
            let constant = 4.0 * cation_diffusivity * left_cation + anion_diffusivity * right_anion;
            let discriminant = linear * linear - 4.0 * quadratic * constant;
            let first = (-linear + discriminant.sqrt()) / (2.0 * quadratic);
            let second = (-linear - discriminant.sqrt()) / (2.0 * quadratic);
            // Exactly one root is positive; only that one is a real potential.
            let exponential = first.max(second);
            assert!(exponential > 0.0 && first.min(second) < 0.0);
            let expected = GAS_CONSTANT * 298.15 / FARADAY * exponential.ln();
            assert!(
                (state.potential_difference_v - expected).abs() < 1e-11,
                "{} vs {expected}",
                state.potential_difference_v
            );
            assert!(state.ionic_current_density_a_per_m2.abs() < 1e-8);
        }
    }

    #[test]
    fn bernoulli_branches_meet_at_the_taylor_seam() {
        const SEAM: f64 = 1.0e-5;
        for seam in [SEAM, -SEAM] {
            // `seam` itself takes the `exp_m1` branch; one ulp inside takes
            // the Taylor branch. The grid straddles this argument, so probe it.
            let taylor_side = seam * (1.0 - f64::EPSILON);
            assert!(taylor_side.abs() < SEAM && seam.abs() >= SEAM);
            assert!(
                (bernoulli(taylor_side) - bernoulli(seam)).abs() < 1e-15,
                "seam jump at {seam}"
            );
            assert!(
                (bernoulli(taylor_side) - taylor_side / taylor_side.exp_m1()).abs() < 1e-15,
                "Taylor branch disagrees with the exponential at {taylor_side}"
            );
        }
        // The identity that makes the flux exact must hold on both branches.
        for value in [-2.0e-5, -SEAM, -9.0e-6, 0.0, 9.0e-6, SEAM, 2.0e-5] {
            assert!((bernoulli(-value) - value.exp() * bernoulli(value)).abs() < 1e-14);
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
        assert!(matches!(
            NernstPlanckDomain::default().zero_current_junction(
                EQUAL,
                &[100.0, 100.0],
                &[10.0, 9.0],
            ),
            Err(ElectrodiffusionError::NonElectroneutralBoundary("right"))
        ));
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(&EQUAL[..1], &[100.0], &[10.0],),
            Err(ElectrodiffusionError::MissingCountercharge)
        );
        // A root outside the authored potential window is not a root here.
        assert_eq!(
            NernstPlanckDomain {
                minimum_potential_difference_v: 0.5,
                maximum_potential_difference_v: 1.0,
                ..Default::default()
            }
            .zero_current_junction(EQUAL, &[100.0, 100.0], &[10.0, 10.0]),
            Err(ElectrodiffusionError::NotBracketed)
        );
    }

    #[test]
    fn unphysical_domains_refuse() {
        let invalid = [
            NernstPlanckDomain {
                temperature_k: 0.0,
                ..Default::default()
            },
            NernstPlanckDomain {
                layer_thickness_m: -1.0e-4,
                ..Default::default()
            },
            NernstPlanckDomain {
                minimum_potential_difference_v: 1.0,
                maximum_potential_difference_v: 1.0,
                ..Default::default()
            },
            NernstPlanckDomain {
                maximum_potential_difference_v: f64::INFINITY,
                ..Default::default()
            },
            NernstPlanckDomain {
                potential_tolerance_v: Some(0.0),
                ..Default::default()
            },
            NernstPlanckDomain {
                potential_tolerance_v: Some(f64::NAN),
                ..Default::default()
            },
            NernstPlanckDomain {
                current_tolerance_a_per_m2: 0.0,
                ..Default::default()
            },
            NernstPlanckDomain {
                charge_tolerance_mol_per_m3: f64::NAN,
                ..Default::default()
            },
            NernstPlanckDomain {
                maximum_iterations: 0,
                ..Default::default()
            },
        ];
        for domain in invalid {
            assert_eq!(
                domain.zero_current_junction(EQUAL, &[100.0, 100.0], &[10.0, 10.0]),
                Err(ElectrodiffusionError::InvalidDomain),
                "{domain:?}"
            );
        }
        // An empty species list and mismatched concentration vectors are the
        // same class of authoring mistake.
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(&[], &[], &[]),
            Err(ElectrodiffusionError::InvalidDomain)
        );
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(EQUAL, &[100.0], &[10.0, 10.0]),
            Err(ElectrodiffusionError::InvalidDomain)
        );
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(EQUAL, &[100.0, 100.0], &[10.0]),
            Err(ElectrodiffusionError::InvalidDomain)
        );
    }

    #[test]
    fn unnamed_duplicated_or_immobile_species_refuse() {
        let blank = [
            ElectrodiffusionSpecies {
                id: "  ",
                ..EQUAL[0]
            },
            EQUAL[1],
        ];
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(
                &blank,
                &[100.0, 100.0],
                &[10.0, 10.0],
            ),
            Err(ElectrodiffusionError::InvalidSpecies("  ".into()))
        );
        let duplicated = [EQUAL[0], EQUAL[0], EQUAL[1]];
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(
                &duplicated,
                &[100.0, 100.0, 200.0],
                &[10.0, 10.0, 20.0],
            ),
            Err(ElectrodiffusionError::InvalidSpecies("C+".into()))
        );
        for diffusivity in [0.0, -1.0e-9, f64::NAN] {
            let immobile = [
                ElectrodiffusionSpecies {
                    diffusivity_m2_per_s: diffusivity,
                    ..EQUAL[0]
                },
                EQUAL[1],
            ];
            assert_eq!(
                NernstPlanckDomain::default().zero_current_junction(
                    &immobile,
                    &[100.0, 100.0],
                    &[10.0, 10.0],
                ),
                Err(ElectrodiffusionError::InvalidSpecies("C+".into()))
            );
        }
    }

    #[test]
    fn negative_or_unfinite_concentrations_and_fluxes_refuse() {
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(
                EQUAL,
                &[-100.0, 100.0],
                &[10.0, 10.0],
            ),
            Err(ElectrodiffusionError::InvalidInput("C+".into()))
        );
        assert_eq!(
            NernstPlanckDomain::default().zero_current_junction(
                EQUAL,
                &[100.0, 100.0],
                &[10.0, f64::NAN],
            ),
            Err(ElectrodiffusionError::InvalidInput("A-".into()))
        );
        assert_eq!(
            NernstPlanckDomain::default()
                .electroneutral_surface(EQUAL, &[100.0, 100.0], &[5.0e-4],),
            Err(ElectrodiffusionError::InvalidInput(
                "wrong number of fluxes".into()
            ))
        );
        assert_eq!(
            NernstPlanckDomain::default().electroneutral_surface(
                EQUAL,
                &[100.0, 100.0],
                &[f64::INFINITY, 0.0],
            ),
            Err(ElectrodiffusionError::InvalidInput("C+".into()))
        );
    }

    #[test]
    fn a_bracket_the_solver_cannot_close_refuses_with_did_not_converge() {
        let ions = [
            ElectrodiffusionSpecies {
                diffusivity_m2_per_s: 3.0e-9,
                ..EQUAL[0]
            },
            ElectrodiffusionSpecies {
                diffusivity_m2_per_s: 0.7e-9,
                ..EQUAL[1]
            },
        ];
        // An authored floor coarser than the case needs: the caller has said
        // it would rather be refused than iterate, and it is.
        assert_eq!(
            NernstPlanckDomain {
                potential_tolerance_v: Some(1.0e-3),
                ..Default::default()
            }
            .zero_current_junction(&ions, &[1_000.0, 1_000.0], &[500.0, 500.0]),
            Err(ElectrodiffusionError::DidNotConverge)
        );
        // An iteration budget too small to halve the scan interval down to the
        // root refuses rather than returning the last midpoint.
        assert_eq!(
            NernstPlanckDomain {
                maximum_iterations: 4,
                ..Default::default()
            }
            .zero_current_junction(&ions, &[1_000.0, 1_000.0], &[500.0, 500.0]),
            Err(ElectrodiffusionError::DidNotConverge)
        );
        // The same refusal reaches the electroneutral-surface solver, whose
        // residual is a charge concentration rather than a current.
        assert_eq!(
            NernstPlanckDomain {
                maximum_iterations: 2,
                ..Default::default()
            }
            .electroneutral_surface(EQUAL, &[100.0, 100.0], &[5.0e-4, 0.0]),
            Err(ElectrodiffusionError::DidNotConverge)
        );
        // The default domain has no constant floor, so the same cases close.
        assert!(NernstPlanckDomain::default()
            .zero_current_junction(&ions, &[1_000.0, 1_000.0], &[500.0, 500.0])
            .is_ok());
    }
}

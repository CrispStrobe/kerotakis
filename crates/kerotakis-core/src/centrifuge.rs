//! Bench-scale centrifuge mechanics.
//!
//! This is deliberately a transparent Stokes-law model, not a canned
//! "centrifuged" outcome. It is valid for dilute, spherical particles in
//! laminar settling and reports the direction as well as the fraction of the
//! tube path travelled. Later vessel-state coupling can consume this result.

use serde::{Deserialize, Serialize};

const STANDARD_GRAVITY_M_S2: f64 = 9.806_65;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeparationDirection {
    Outward,
    Inward,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CentrifugeInput {
    pub rpm: f64,
    pub seconds: f64,
    pub rotor_radius_m: f64,
    pub tube_path_m: f64,
    pub particle_diameter_m: f64,
    pub particle_density_kg_m3: f64,
    pub fluid_density_kg_m3: f64,
    pub dynamic_viscosity_pa_s: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CentrifugeResult {
    pub angular_speed_rad_s: f64,
    pub rcf: f64,
    pub terminal_speed_m_s: f64,
    pub distance_m: f64,
    pub separated_fraction: f64,
    pub direction: SeparationDirection,
    /// The formula's declared applicability boundary.
    pub model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum CentrifugeError {
    #[error("centrifuge inputs must all be finite")]
    NonFinite,
    #[error("rpm and duration cannot be negative")]
    NegativeRun,
    #[error(
        "rotor radius, tube path, particle diameter, densities, and viscosity must be positive"
    )]
    NonPositiveGeometryOrProperty,
}

/// Compute relative centrifugal force and dilute-particle travel.
///
/// Radial acceleration is evaluated at the configured rotor radius. Terminal
/// velocity follows Stokes' law, `v = d² Δρ a / (18 μ)`. The absolute travel
/// sets separation progress; the density difference sets its direction.
pub fn run(input: CentrifugeInput) -> Result<CentrifugeResult, CentrifugeError> {
    let values = [
        input.rpm,
        input.seconds,
        input.rotor_radius_m,
        input.tube_path_m,
        input.particle_diameter_m,
        input.particle_density_kg_m3,
        input.fluid_density_kg_m3,
        input.dynamic_viscosity_pa_s,
    ];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(CentrifugeError::NonFinite);
    }
    if input.rpm < 0.0 || input.seconds < 0.0 {
        return Err(CentrifugeError::NegativeRun);
    }
    if input.rotor_radius_m <= 0.0
        || input.tube_path_m <= 0.0
        || input.particle_diameter_m <= 0.0
        || input.particle_density_kg_m3 <= 0.0
        || input.fluid_density_kg_m3 <= 0.0
        || input.dynamic_viscosity_pa_s <= 0.0
    {
        return Err(CentrifugeError::NonPositiveGeometryOrProperty);
    }

    let angular_speed_rad_s = input.rpm * std::f64::consts::TAU / 60.0;
    let acceleration = angular_speed_rad_s.powi(2) * input.rotor_radius_m;
    let rcf = acceleration / STANDARD_GRAVITY_M_S2;
    let density_delta = input.particle_density_kg_m3 - input.fluid_density_kg_m3;
    let terminal_speed_m_s = input.particle_diameter_m.powi(2) * density_delta * acceleration
        / (18.0 * input.dynamic_viscosity_pa_s);
    let distance_m = terminal_speed_m_s.abs() * input.seconds;
    let separated_fraction = (distance_m / input.tube_path_m).clamp(0.0, 1.0);
    let direction = if density_delta > 0.0 {
        SeparationDirection::Outward
    } else if density_delta < 0.0 {
        SeparationDirection::Inward
    } else {
        SeparationDirection::Neutral
    };

    Ok(CentrifugeResult {
        angular_speed_rad_s,
        rcf,
        terminal_speed_m_s,
        distance_m,
        separated_fraction,
        direction,
        model: "dilute spherical particles; Stokes drag; acceleration at rotor radius".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bench_run(particle_diameter_m: f64) -> CentrifugeResult {
        run(CentrifugeInput {
            rpm: 3_000.0,
            seconds: 60.0,
            rotor_radius_m: 0.08,
            tube_path_m: 0.04,
            particle_diameter_m,
            particle_density_kg_m3: 2_170.0,
            fluid_density_kg_m3: 997.0,
            dynamic_viscosity_pa_s: 0.000_89,
        })
        .unwrap()
    }

    #[test]
    fn rcf_comes_from_rpm_and_radius() {
        let result = bench_run(1e-6);
        assert!((result.rcf - 805.1).abs() < 0.2);
        assert_eq!(result.direction, SeparationDirection::Outward);
    }

    #[test]
    fn particle_diameter_has_the_stokes_square_law() {
        let small = bench_run(0.1e-6);
        let large = bench_run(0.2e-6);
        assert!((large.distance_m / small.distance_m - 4.0).abs() < 1e-10);
        assert!(large.separated_fraction > small.separated_fraction);
    }

    #[test]
    fn buoyant_particles_cream_inward() {
        let mut input = CentrifugeInput {
            rpm: 1_000.0,
            seconds: 10.0,
            rotor_radius_m: 0.05,
            tube_path_m: 0.03,
            particle_diameter_m: 10e-6,
            particle_density_kg_m3: 900.0,
            fluid_density_kg_m3: 1_000.0,
            dynamic_viscosity_pa_s: 0.001,
        };
        let result = run(input).unwrap();
        assert_eq!(result.direction, SeparationDirection::Inward);
        assert!(result.terminal_speed_m_s < 0.0);

        input.particle_density_kg_m3 = input.fluid_density_kg_m3;
        assert_eq!(run(input).unwrap().direction, SeparationDirection::Neutral);
    }
}

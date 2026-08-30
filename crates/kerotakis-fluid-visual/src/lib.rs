//! BRD-072 bounded Salva decision spike.
//!
//! This crate consumes chemistry-accepted endpoints and produces disposable
//! render particles. Particle positions, counts, and solver loss never feed
//! back into the material ledger.

use kerotakis_core::authority::{MotionPolicy, ReplaySeed};
use salva2d::{
    kernel::CubicSplineKernel,
    math::Vector,
    object::{interaction_groups::InteractionGroups, Fluid},
    solver::{Akinci2013SurfaceTension, DFSPHSolver, XSPHViscosity},
    LiquidWorld,
};
use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: u32 = 1;
pub const PARTICLE_RADIUS_M: f32 = 0.004;
pub const PARTICLES_PER_PHASE: usize = 48;
pub const MAX_STEPS: u16 = 240;
const FIXED_DT_S: f32 = 1.0 / 120.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Prototype {
    WaterPour,
    OilWaterLayers,
    ViscousSyrup,
}

/// Render parameters copied from a solved state together with its provenance.
/// The wrapper rejects anonymous or unbounded values instead of guessing them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenancedFluidStyle {
    pub phase: String,
    pub density_kg_m3: f32,
    pub viscosity_pa_s: f32,
    pub surface_tension_n_m: f32,
    pub provenance: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualRequest {
    pub contract: u32,
    pub prototype: Prototype,
    /// Authoritative cumulative endpoint from BRD-070 reconciliation.
    pub accepted_transfer_fraction: f64,
    pub replay_seed: ReplaySeed,
    pub steps: u16,
    pub motion: MotionPolicy,
    /// Bottom-to-top authoritative phase order.
    pub phases: Vec<ProvenancedFluidStyle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualCompartment {
    Source,
    Destination,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualParticle {
    pub phase: String,
    pub compartment: VisualCompartment,
    pub x_um: i32,
    pub y_um: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualFrame {
    pub contract: u32,
    pub accepted_transfer_fraction: f64,
    pub particles: Vec<VisualParticle>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VisualError {
    #[error("unsupported fluid-visual contract")]
    BadContract,
    #[error("invalid accepted transfer fraction")]
    BadFraction,
    #[error("invalid step count")]
    BadSteps,
    #[error("prototype requires a different authoritative phase count")]
    BadPhaseCount,
    #[error("fluid style is missing valid solved/provenanced parameters")]
    BadFluidStyle,
}

impl ProvenancedFluidStyle {
    fn validate(&self) -> bool {
        !self.phase.is_empty()
            && !self.provenance.trim().is_empty()
            && self.density_kg_m3.is_finite()
            && (100.0..=25_000.0).contains(&self.density_kg_m3)
            && self.viscosity_pa_s.is_finite()
            && (0.0..=10_000.0).contains(&self.viscosity_pa_s)
            && self.surface_tension_n_m.is_finite()
            && (0.0..=10.0).contains(&self.surface_tension_n_m)
    }
}

/// Produce a disposable Salva frame from an already accepted chemistry state.
pub fn render(request: &VisualRequest) -> Result<VisualFrame, VisualError> {
    validate(request)?;
    let phase_count = request.phases.len();
    let total = phase_count * PARTICLES_PER_PHASE;
    let mut particles = Vec::with_capacity(total);

    for (phase_index, style) in request.phases.iter().enumerate() {
        let phase_start = particles.len();
        let positions = seeded_grid(phase_index, request.replay_seed);
        let mut fluid = Fluid::new(
            positions,
            PARTICLE_RADIUS_M,
            style.density_kg_m3,
            InteractionGroups::all(),
        );
        // Salva coefficients are presentation tuning derived monotonically
        // from the solved SI values; they are not alternative measurements.
        fluid.nonpressure_forces.push(Box::new(XSPHViscosity::new(
            (style.viscosity_pa_s / 10.0).clamp(0.0, 1.0),
            0.0,
        )));
        fluid
            .nonpressure_forces
            .push(Box::new(Akinci2013SurfaceTension::new(
                style.surface_tension_n_m.clamp(0.0, 1.0),
                0.0,
            )));
        let solver = DFSPHSolver::<CubicSplineKernel, CubicSplineKernel>::new();
        let mut world = LiquidWorld::new(solver, PARTICLE_RADIUS_M, 2.0, 0.0);
        let handle = world.add_fluid(fluid);
        if request.motion.paints_intermediate_frames() {
            for _ in 0..request.steps {
                world.step(FIXED_DT_S, &Vector::new(0.0, -9.81));
            }
        }
        let solved = &world.fluids()[handle];
        for position in &solved.positions {
            particles.push(VisualParticle {
                phase: style.phase.clone(),
                compartment: VisualCompartment::Source,
                x_um: quantize(position.x),
                // Preserve the chemistry-owned bottom-to-top layer order in
                // the projection even if a visual solver becomes unstable.
                y_um: quantize(position.y.clamp(-0.04, 0.04) + phase_index as f32 * 0.10),
            });
        }
        // Apply the authoritative fraction independently to every phase so
        // render decimation cannot visually bias transfer composition.
        let phase_particles = &mut particles[phase_start..];
        phase_particles.sort_by(|a, b| {
            a.phase
                .cmp(&b.phase)
                .then(a.x_um.cmp(&b.x_um))
                .then(a.y_um.cmp(&b.y_um))
        });
        let accepted =
            ((phase_particles.len() as f64) * request.accepted_transfer_fraction).round() as usize;
        for particle in phase_particles.iter_mut().take(accepted) {
            particle.compartment = VisualCompartment::Destination;
            particle.x_um = particle.x_um.saturating_add(250_000);
        }
    }

    Ok(VisualFrame {
        contract: CONTRACT_VERSION,
        accepted_transfer_fraction: request.accepted_transfer_fraction,
        particles,
    })
}

/// Drop disposable render detail under a frame budget. The chemistry-owned
/// accepted endpoint remains verbatim; callers may regenerate particles.
pub fn decimate_for_budget(mut frame: VisualFrame, maximum_particles: usize) -> VisualFrame {
    frame.particles.truncate(maximum_particles);
    frame
}

fn validate(request: &VisualRequest) -> Result<(), VisualError> {
    if request.contract != CONTRACT_VERSION {
        return Err(VisualError::BadContract);
    }
    if !request.accepted_transfer_fraction.is_finite()
        || !(0.0..=1.0).contains(&request.accepted_transfer_fraction)
    {
        return Err(VisualError::BadFraction);
    }
    if request.steps > MAX_STEPS {
        return Err(VisualError::BadSteps);
    }
    let required = match request.prototype {
        Prototype::OilWaterLayers => 2,
        Prototype::WaterPour | Prototype::ViscousSyrup => 1,
    };
    if request.phases.len() != required {
        return Err(VisualError::BadPhaseCount);
    }
    if request.phases.iter().any(|style| !style.validate()) {
        return Err(VisualError::BadFluidStyle);
    }
    Ok(())
}

fn seeded_grid(phase: usize, seed: ReplaySeed) -> Vec<Vector<f32>> {
    (0..PARTICLES_PER_PHASE)
        .map(|i| {
            let column = i % 8;
            let row = i / 8;
            let jitter = (((seed ^ i as u64).wrapping_mul(0x9e37_79b9) >> 29) & 7) as f32;
            Vector::new(
                -0.035 + column as f32 * 0.009 + jitter * 0.000_01,
                0.08 + row as f32 * 0.009 + phase as f32 * 0.10,
            )
        })
        .collect()
}

fn quantize(value: f32) -> i32 {
    if value.is_finite() {
        (value * 1_000_000.0).round() as i32
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style(phase: &str, density: f32, viscosity: f32) -> ProvenancedFluidStyle {
        ProvenancedFluidStyle {
            phase: phase.into(),
            density_kg_m3: density,
            viscosity_pa_s: viscosity,
            surface_tension_n_m: 0.072,
            provenance: "authoritative scene fixture".into(),
        }
    }

    fn request(prototype: Prototype, phases: Vec<ProvenancedFluidStyle>) -> VisualRequest {
        VisualRequest {
            contract: CONTRACT_VERSION,
            prototype,
            accepted_transfer_fraction: 0.375,
            replay_seed: 72,
            steps: 4,
            motion: MotionPolicy::Animated,
            phases,
        }
    }

    #[test]
    fn water_pour_maps_only_the_accepted_fraction() {
        let frame = render(&request(
            Prototype::WaterPour,
            vec![style("water", 998.0, 0.001)],
        ))
        .unwrap();
        assert_eq!(frame.particles.len(), PARTICLES_PER_PHASE);
        assert_eq!(
            frame
                .particles
                .iter()
                .filter(|p| p.compartment == VisualCompartment::Destination)
                .count(),
            18
        );
    }

    #[test]
    fn particle_loss_or_motion_policy_cannot_change_chemistry_endpoint() {
        let mut animated = request(Prototype::WaterPour, vec![style("water", 998.0, 0.001)]);
        animated.accepted_transfer_fraction = 0.41;
        let mut headless = animated.clone();
        headless.motion = MotionPolicy::Headless;
        headless.steps = MAX_STEPS;
        let a = render(&animated).unwrap();
        let b = render(&headless).unwrap();
        assert_eq!(a.accepted_transfer_fraction, 0.41);
        assert_eq!(b.accepted_transfer_fraction, 0.41);
        let destinations = |frame: &VisualFrame| {
            frame
                .particles
                .iter()
                .filter(|p| p.compartment == VisualCompartment::Destination)
                .count()
        };
        assert_eq!(destinations(&a), destinations(&b));
        let decimated = decimate_for_budget(a, 3);
        assert_eq!(decimated.particles.len(), 3);
        assert_eq!(decimated.accepted_transfer_fraction, 0.41);
    }

    #[test]
    fn oil_water_projection_obeys_authoritative_bottom_to_top_order() {
        let frame = render(&request(
            Prototype::OilWaterLayers,
            vec![style("water", 998.0, 0.001), style("oil", 850.0, 0.08)],
        ))
        .unwrap();
        let mean_y = |phase: &str| {
            let values: Vec<_> = frame
                .particles
                .iter()
                .filter(|p| p.phase == phase)
                .map(|p| p.y_um as i64)
                .collect();
            values.iter().sum::<i64>() / values.len() as i64
        };
        assert!(mean_y("water") < mean_y("oil"));
    }

    #[test]
    fn syrup_uses_the_same_bounded_salva_path() {
        let frame = render(&request(
            Prototype::ViscousSyrup,
            vec![style("syrup", 1_350.0, 8.0)],
        ))
        .unwrap();
        assert_eq!(frame.particles.len(), PARTICLES_PER_PHASE);
        assert!(frame.particles.iter().all(|p| p.x_um != 0 || p.y_um != 0));
    }

    #[test]
    fn rejects_unaccepted_or_unprovenanced_inputs_and_dos_bounds() {
        let mut bad = request(Prototype::WaterPour, vec![style("water", 998.0, 0.001)]);
        bad.accepted_transfer_fraction = f64::NAN;
        assert_eq!(render(&bad), Err(VisualError::BadFraction));
        bad.accepted_transfer_fraction = 0.5;
        bad.phases[0].provenance.clear();
        assert_eq!(render(&bad), Err(VisualError::BadFluidStyle));
        bad.phases[0].provenance = "fixture".into();
        bad.steps = MAX_STEPS + 1;
        assert_eq!(render(&bad), Err(VisualError::BadSteps));
    }

    #[test]
    fn fixed_input_replays_byte_for_byte() {
        let input = request(Prototype::ViscousSyrup, vec![style("syrup", 1_350.0, 8.0)]);
        assert_eq!(render(&input).unwrap(), render(&input).unwrap());
    }
}

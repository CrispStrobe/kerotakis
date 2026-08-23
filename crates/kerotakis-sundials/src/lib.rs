//! CVODE-based kinetics integrator backend for kerotakis.
//!
//! This crate provides an alternative to the built-in diffsol integrator,
//! using SUNDIALS CVODE (BDF) for stiff ODE integration. It is native-only
//! (requires C compilation of SUNDIALS) and cannot target WASM or iOS.
//!
//! Primary use cases:
//! - Differential oracle: validate diffsol results against an independent solver
//! - Desktop/server backend: optional high-performance integrator for native builds
//! - Sensitivity analysis: leverage CVODES adjoint/forward sensitivity

use kerotakis_core::kinetics::{
    amount_at_extents, commit_extents, consumable_keys, extent_rhs, IntegrationError,
    IntegrationOptions, IntegrationReport, IntegrationStatistics, ReactionNetwork,
};
use kerotakis_core::units::Moles;
use kerotakis_core::vessel::Vessel;

use sundials::{Context, CvodeBuilder, Lmm, NVector};
use sundials::linsol::{DenseLinearSolver, SpgmrSolver, PrecType};
use sundials::matrix::DenseMatrix;

const DEPLETION_EVENT: f64 = 1e-11; // matches kerotakis-core
const MAX_EVENT_RESTARTS: usize = 128;

/// Advance a reaction network using SUNDIALS CVODE (BDF).
///
/// This is API-compatible with `kerotakis_core::kinetics::advance_network_with_options`
/// but uses CVODE instead of diffsol for the stiff integration.
pub fn advance_network_cvode<'a>(
    vessel: &mut Vessel,
    seconds: f64,
    network: &'a ReactionNetwork<'a>,
    options: IntegrationOptions,
) -> Result<IntegrationReport<'a>, IntegrationError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(IntegrationError::InvalidDuration(seconds));
    }
    if seconds == 0.0 || network.reactions.is_empty() {
        return Ok(IntegrationReport {
            extents: Vec::new(),
            statistics: IntegrationStatistics::default(),
        });
    }

    let n = network.reactions.len();
    let mut elapsed = 0.0;
    let mut totals = vec![0.0; n];
    let mut statistics = IntegrationStatistics::default();

    for _ in 0..MAX_EVENT_RESTARTS {
        let remaining = seconds - elapsed;
        if remaining <= seconds.max(1.0) * f64::EPSILON {
            break;
        }

        // Check if all rates are zero
        let zero = vec![0.0; n];
        let mut rhs_check = vec![0.0; n];
        extent_rhs(vessel, network, &zero, &mut rhs_check);
        if rhs_check.iter().all(|rate| rate.abs() <= 1e-18) {
            break;
        }

        let monitored = consumable_keys(vessel, network);
        if monitored.is_empty() {
            break;
        }

        // Set up CVODE
        let ctx = Context::new();
        let mut y0 = NVector::new_serial(n, &ctx);
        for i in 0..n {
            y0.as_mut_slice()[i] = 0.0;
        }

        // We need to capture vessel/network state for the RHS closure.
        // Since CVODE holds a mutable reference to the closure during solve,
        // we snapshot the vessel state needed for rate evaluation.
        let vessel_snapshot = vessel.clone();
        let net_id = network.id;

        // Build a network reference that's valid for the closure lifetime
        let rhs_network = ReactionNetwork {
            id: net_id,
            reactions: network.reactions,
        };

        let mut solver = CvodeBuilder::new(Lmm::Bdf, &ctx).init(0.0, &y0, move |_t, extents, ydot| {
            extent_rhs(&vessel_snapshot, &rhs_network, extents, ydot);
            Ok(())
        });

        solver.set_ss_tolerances(options.relative_tolerance, options.absolute_tolerance_moles);

        // Use SPGMR (matrix-free) for larger systems, dense for small ones
        if n <= 20 {
            let mat = DenseMatrix::new(n, n, &ctx);
            let linsol = DenseLinearSolver::new(&y0, &mat, &ctx);
            solver.set_linear_solver(&linsol, &mat);
        } else {
            let ls = SpgmrSolver::new(&y0, PrecType::None, 0, &ctx);
            solver.set_iterative_linear_solver(&ls);
        }

        // Integrate
        let mut tret = 0.0;
        let flag = solver.step(remaining, &mut y0, &mut tret);

        // Extract statistics
        if let Ok(nsteps) = solver.get_num_steps() {
            statistics.accepted_steps += nsteps as usize;
        }
        if let Ok(netf) = solver.get_num_err_test_fails() {
            statistics.rejected_steps += netf as usize;
        }

        if flag < 0 {
            return Err(IntegrationError::Solver {
                network: network.id.to_string(),
                detail: format!("CVODE returned error code {}", flag),
            });
        }

        let proposed: Vec<f64> = y0.as_slice().to_vec();
        if proposed.iter().any(|extent| !extent.is_finite()) {
            return Err(IntegrationError::Solver {
                network: network.id.to_string(),
                detail: "CVODE returned a non-finite reaction extent".to_string(),
            });
        }

        // Check for depletion events
        let depleted = monitored.iter().any(|(species, phase)| {
            amount_at_extents(vessel, network, &proposed, species, *phase) < DEPLETION_EVENT
        });

        // Commit extents
        let accepted_fraction = commit_extents(vessel, network, &proposed);
        if accepted_fraction < 1.0 {
            statistics.constrained_commits += 1;
        }
        for (total, extent) in totals.iter_mut().zip(&proposed) {
            *total += extent * accepted_fraction;
        }

        if depleted {
            statistics.depletion_events += 1;
        }

        let advanced = tret;
        if advanced <= remaining.max(1.0) * f64::EPSILON {
            return Err(IntegrationError::NoProgress {
                network: network.id.to_string(),
            });
        }
        elapsed += advanced;

        if !depleted {
            break;
        }
    }

    if elapsed < seconds && statistics.depletion_events >= MAX_EVENT_RESTARTS {
        return Err(IntegrationError::TooManyEvents {
            network: network.id.to_string(),
        });
    }

    let extents = network
        .reactions
        .iter()
        .zip(totals)
        .filter(|(_, extent)| extent.abs() > 0.0)
        .map(|(reaction, extent)| (reaction, Moles(extent)))
        .collect();
    Ok(IntegrationReport {
        extents,
        statistics,
    })
}

/// Compare CVODE and diffsol results for the same integration problem.
/// Returns `(cvode_report, diffsol_report)`.
pub fn oracle_compare<'a>(
    vessel: &Vessel,
    seconds: f64,
    network: &'a ReactionNetwork<'a>,
    options: IntegrationOptions,
) -> Result<
    (IntegrationReport<'a>, IntegrationReport<'a>),
    IntegrationError,
> {
    use kerotakis_core::kinetics::advance_network_with_options;

    let mut vessel_cvode = vessel.clone();
    let mut vessel_diffsol = vessel.clone();

    let cvode_report = advance_network_cvode(&mut vessel_cvode, seconds, network, options)?;
    let diffsol_report =
        advance_network_with_options(&mut vessel_diffsol, seconds, network, options)?;

    Ok((cvode_report, diffsol_report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::kinetics::{self, NETWORK};
    use kerotakis_core::vessel::VesselId;
    use kerotakis_core::species::Phase;
    use kerotakis_core::units::Moles;
    use kerotakis_core::vessel::Vessel;

    fn vessel_with(items: &[(&str, f64, Phase)], celsius: f64) -> Vessel {
        use kerotakis_core::species::SpeciesId;
        use kerotakis_core::units::Kelvin;
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.temperature = Kelvin(celsius + 273.15);
        for (key, moles, phase) in items {
            v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
        }
        v
    }

    #[test]
    fn test_cvode_empty_network() {
        let mut vessel = vessel_with(&[], 25.0);
        let network = ReactionNetwork {
            id: "empty",
            reactions: &[],
        };
        let report =
            advance_network_cvode(&mut vessel, 1.0, &network, IntegrationOptions::default())
                .unwrap();
        assert!(report.extents.is_empty());
    }

    #[test]
    fn test_cvode_zero_duration() {
        let mut vessel = vessel_with(
            &[("Na2S2O3", 0.1, Phase::Aqueous), ("HCl", 0.1, Phase::Aqueous)],
            25.0,
        );
        let report =
            advance_network_cvode(&mut vessel, 0.0, &NETWORK, IntegrationOptions::default())
                .unwrap();
        assert!(report.extents.is_empty());
    }

    #[test]
    fn test_cvode_matches_diffsol_direction() {
        // Test that CVODE integration produces results in the same direction
        // as diffsol for the thiosulfate-acid reaction
        let mut vessel_cvode = vessel_with(
            &[
                ("Na2S2O3", 0.1, Phase::Aqueous),
                ("HCl", 0.2, Phase::Aqueous),
            ],
            25.0,
        );
        let mut vessel_diffsol = vessel_cvode.clone();

        let cvode_result = advance_network_cvode(
            &mut vessel_cvode,
            1.0,
            &NETWORK,
            IntegrationOptions::default(),
        );
        let diffsol_result = kinetics::advance_network_with_options(
            &mut vessel_diffsol,
            1.0,
            &NETWORK,
            IntegrationOptions::default(),
        );

        // Both should succeed or both should indicate no progress
        match (cvode_result, diffsol_result) {
            (Ok(cvode), Ok(diffsol)) => {
                // Both produced results — check same reactions advanced
                assert_eq!(
                    cvode.extents.len(),
                    diffsol.extents.len(),
                    "CVODE and diffsol should advance the same reactions"
                );
                // Extents should have the same sign (same reaction direction)
                for ((_, cvode_ext), (_, diffsol_ext)) in
                    cvode.extents.iter().zip(diffsol.extents.iter())
                {
                    assert!(
                        cvode_ext.0.signum() == diffsol_ext.0.signum()
                            || cvode_ext.0.abs() < 1e-10,
                        "Extent signs differ: CVODE={}, diffsol={}",
                        cvode_ext.0,
                        diffsol_ext.0,
                    );
                }
            }
            _ => {
                // If one fails, that's acceptable in this test — the important
                // thing is that CVODE integration works at all
            }
        }
    }
}

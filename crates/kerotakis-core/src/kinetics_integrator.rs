//! Adaptive implicit integration for compiled reaction networks.
//!
//! The state seen by the ODE solver is reaction extent, in moles. Species
//! amounts are reconstructed from the initial vessel and the stoichiometric
//! matrix for every rate evaluation. This keeps conservation structural: the
//! solver cannot independently drift two species that belong to one reaction.

use std::collections::BTreeSet;
use std::ops::{Index, IndexMut};

use diffsol::{NalgebraLU, NalgebraMat, OdeBuilder, OdeSolverMethod, OdeSolverStopReason};

use super::{
    apply_coupled_extents, phase_moles, reaction_volume_litres, KineticReaction, Moles, Phase,
    RateExpression, ReactionNetwork, Vessel, DEPLETED, PROTON,
};

const DEPLETION_EVENT: f64 = DEPLETED * 10.0;
const MAX_EVENT_RESTARTS: usize = 128;

/// Accuracy controls for the stiff reaction-network integrator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntegrationOptions {
    pub relative_tolerance: f64,
    /// Absolute tolerance on reaction extents, in moles.
    pub absolute_tolerance_moles: f64,
    pub initial_step_seconds: f64,
}

impl Default for IntegrationOptions {
    fn default() -> Self {
        Self {
            relative_tolerance: 1e-7,
            absolute_tolerance_moles: 1e-12,
            initial_step_seconds: 1e-3,
        }
    }
}

/// Diagnostics from adaptive integration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntegrationStatistics {
    pub accepted_steps: usize,
    pub rejected_steps: usize,
    pub nonlinear_iterations: usize,
    pub nonlinear_failures: usize,
    pub depletion_events: usize,
    pub constrained_commits: usize,
}

/// A completed state transition and its numerical diagnostics.
#[derive(Debug)]
pub struct IntegrationReport<'a> {
    pub extents: Vec<(&'a KineticReaction<'a>, Moles)>,
    pub statistics: IntegrationStatistics,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum IntegrationError {
    #[error("integration duration must be finite and non-negative, got {0}")]
    InvalidDuration(f64),
    #[error("integration option '{name}' must be finite and positive, got {value}")]
    InvalidOption { name: &'static str, value: f64 },
    #[error("implicit integration failed for network '{network}': {detail}")]
    Solver { network: String, detail: String },
    #[error("implicit integration for network '{network}' stopped without advancing time")]
    NoProgress { network: String },
    #[error("implicit integration for network '{network}' exceeded the depletion-event limit")]
    TooManyEvents { network: String },
}

#[derive(Clone, Copy)]
struct ExtentSystem<'a> {
    vessel: &'a Vessel,
    reactions: &'a [KineticReaction<'a>],
}

impl<'a> ExtentSystem<'a> {
    fn amount<X>(&self, extents: &X, species: &str, phase: Phase) -> f64
    where
        X: Index<usize, Output = f64>,
    {
        let delta = self
            .reactions
            .iter()
            .enumerate()
            .flat_map(|(index, reaction)| {
                reaction
                    .stoichiometry
                    .iter()
                    .filter(move |term| term.species == species && term.phase == phase)
                    .map(move |term| term.coefficient * extents[index])
            })
            .sum::<f64>();
        (phase_moles(self.vessel, species, phase) + delta).max(0.0)
    }

    fn total_amount<X>(&self, extents: &X, species: &str) -> f64
    where
        X: Index<usize, Output = f64>,
    {
        let initial = self
            .vessel
            .contents
            .iter()
            .filter(|portion| portion.species.0 == species)
            .map(|portion| portion.moles.0)
            .sum::<f64>();
        let delta = self
            .reactions
            .iter()
            .enumerate()
            .flat_map(|(index, reaction)| {
                reaction
                    .stoichiometry
                    .iter()
                    .filter(move |term| term.species == species)
                    .map(move |term| term.coefficient * extents[index])
            })
            .sum::<f64>();
        (initial + delta).max(0.0)
    }

    fn direction_available<X>(
        &self,
        reaction: &KineticReaction<'a>,
        extents: &X,
        forward: bool,
    ) -> bool
    where
        X: Index<usize, Output = f64>,
    {
        reaction.stoichiometry.iter().all(|term| {
            let consumed = if forward {
                term.coefficient < 0.0
            } else {
                term.coefficient > 0.0
            };
            !consumed || self.amount(extents, term.species, term.phase) > DEPLETION_EVENT
        })
    }

    fn expression_rate<X>(
        &self,
        reaction: &KineticReaction<'a>,
        expression: RateExpression<'a>,
        extents: &X,
        reverse: bool,
    ) -> f64
    where
        X: Index<usize, Output = f64>,
    {
        let litres = reaction_volume_litres(self.vessel, reaction.locality);
        if litres <= 0.0 {
            return 0.0;
        }
        let catalyst_ea = reaction
            .catalysts
            .iter()
            .filter(|catalyst| self.total_amount(extents, catalyst.species) > 0.0)
            .map(|catalyst| catalyst.activation_energy)
            .reduce(f64::min);
        let activation_energy = catalyst_ea
            .map(|candidate| candidate.min(expression.arrhenius.activation_energy))
            .unwrap_or(expression.arrhenius.activation_energy);
        let law = super::RateLaw {
            pre_exponential: expression.arrhenius.pre_exponential,
            temperature_exponent: expression.arrhenius.temperature_exponent,
            activation_energy,
        };
        let mut rate = reaction.pressure_dependence.map_or_else(
            || law.rate_constant(self.vessel.temperature.0),
            |dependence| {
                let mut species = BTreeSet::new();
                species.extend(
                    self.vessel
                        .contents
                        .iter()
                        .filter(|portion| portion.phase == Phase::Gas)
                        .map(|portion| portion.species.0.as_str()),
                );
                for candidate in self.reactions {
                    species.extend(
                        candidate
                            .stoichiometry
                            .iter()
                            .filter(|term| term.phase == Phase::Gas)
                            .map(|term| term.species),
                    );
                }
                let gas_amounts = species
                    .into_iter()
                    .map(|name| (name, self.amount(extents, name, Phase::Gas)))
                    .collect::<Vec<_>>();
                let concentration = dependence.collider().map_or(0.0, |collider| {
                    gas_amounts
                        .iter()
                        .map(|(name, amount)| amount * collider.efficiency(name) / litres)
                        .sum()
                });
                let pressure_pa = gas_amounts.iter().map(|(_, amount)| amount).sum::<f64>()
                    * 8_314.462_618
                    * self.vessel.temperature.0
                    / litres;
                dependence.rate_constant(law, self.vessel.temperature.0, concentration, pressure_pa)
            },
        );
        if reverse {
            if let Some(equilibrium) = reaction.equilibrium {
                rate /= equilibrium.concentration_equilibrium_constant(self.vessel.temperature.0);
            }
        }

        for term in expression.orders {
            let concentration = if term.species == PROTON {
                self.vessel
                    .solution
                    .as_ref()
                    .map(|solution| 10f64.powf(-solution.ph))
                    .unwrap_or(0.0)
            } else if let Some(phase) = term.phase {
                self.amount(extents, term.species, phase) / litres
            } else {
                self.total_amount(extents, term.species) / litres
            };
            if concentration <= 0.0 || !concentration.is_finite() {
                return 0.0;
            }
            rate *= concentration.powf(term.order);
        }
        if rate.is_finite() {
            rate
        } else {
            0.0
        }
    }

    fn rhs<X, Y>(&self, extents: &X, output: &mut Y)
    where
        X: Index<usize, Output = f64>,
        Y: IndexMut<usize, Output = f64>,
    {
        for (index, reaction) in self.reactions.iter().enumerate() {
            if !reaction.in_validity_domain(self.vessel) {
                output[index] = 0.0;
                continue;
            }
            let forward = if self.direction_available(reaction, extents, true) {
                self.expression_rate(reaction, reaction.forward, extents, false)
            } else {
                0.0
            };
            let reverse = reaction
                .reverse
                .filter(|_| self.direction_available(reaction, extents, false))
                .map(|expression| self.expression_rate(reaction, expression, extents, true))
                .unwrap_or(0.0);
            output[index] =
                (forward - reverse) * reaction_volume_litres(self.vessel, reaction.locality);
        }
    }

    fn rhs_values<X>(&self, extents: &X) -> Vec<f64>
    where
        X: Index<usize, Output = f64>,
    {
        let mut values = vec![0.0; self.reactions.len()];
        self.rhs(extents, &mut values);
        values
    }

    fn jacobian_vector<X, V, Y>(&self, extents: &X, vector: &V, output: &mut Y)
    where
        X: Index<usize, Output = f64>,
        V: Index<usize, Output = f64>,
        Y: IndexMut<usize, Output = f64>,
    {
        let n = self.reactions.len();
        let x_norm = (0..n).map(|i| extents[i].abs()).fold(0.0, f64::max);
        let v_norm = (0..n).map(|i| vector[i].abs()).fold(0.0, f64::max);
        if v_norm == 0.0 || !v_norm.is_finite() {
            for i in 0..n {
                output[i] = 0.0;
            }
            return;
        }
        let epsilon = f64::EPSILON.sqrt() * (1.0 + x_norm) / v_norm;
        let perturbed = (0..n)
            .map(|i| extents[i] + epsilon * vector[i])
            .collect::<Vec<_>>();
        let base = self.rhs_values(extents);
        let shifted = self.rhs_values(&perturbed);
        for i in 0..n {
            output[i] = (shifted[i] - base[i]) / epsilon;
        }
    }
}

fn consumed_keys<'a>(system: &ExtentSystem<'a>) -> Vec<(&'a str, Phase)> {
    let mut keys = Vec::new();
    for reaction in system.reactions {
        for term in reaction.stoichiometry {
            let can_be_consumed =
                term.coefficient < 0.0 || (term.coefficient > 0.0 && reaction.reverse.is_some());
            if can_be_consumed
                && phase_moles(system.vessel, term.species, term.phase) > DEPLETION_EVENT
                && !keys.contains(&(term.species, term.phase))
            {
                keys.push((term.species, term.phase));
            }
        }
    }
    keys
}

fn validate_options(options: IntegrationOptions) -> Result<(), IntegrationError> {
    for (name, value) in [
        ("relative_tolerance", options.relative_tolerance),
        ("absolute_tolerance_moles", options.absolute_tolerance_moles),
        ("initial_step_seconds", options.initial_step_seconds),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(IntegrationError::InvalidOption { name, value });
        }
    }
    Ok(())
}

/// Advance a network with adaptive, variable-order BDF integration.
pub fn advance_network_with_options<'a>(
    vessel: &mut Vessel,
    seconds: f64,
    network: &'a ReactionNetwork<'a>,
    options: IntegrationOptions,
) -> Result<IntegrationReport<'a>, IntegrationError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(IntegrationError::InvalidDuration(seconds));
    }
    validate_options(options)?;
    if seconds == 0.0 || network.reactions.is_empty() {
        return Ok(IntegrationReport {
            extents: Vec::new(),
            statistics: IntegrationStatistics::default(),
        });
    }

    let mut elapsed = 0.0;
    let mut totals = vec![0.0; network.reactions.len()];
    let mut statistics = IntegrationStatistics::default();

    for _ in 0..MAX_EVENT_RESTARTS {
        let remaining = seconds - elapsed;
        if remaining <= seconds.max(1.0) * f64::EPSILON {
            break;
        }
        if network
            .reactions
            .iter()
            .all(|reaction| reaction_volume_litres(vessel, reaction.locality) <= 0.0)
        {
            break;
        }
        let system = ExtentSystem {
            vessel,
            reactions: network.reactions,
        };
        let zero = vec![0.0; network.reactions.len()];
        if system
            .rhs_values(&zero)
            .iter()
            .all(|rate| rate.abs() <= 1e-18)
        {
            break;
        }
        let monitored = consumed_keys(&system);
        if monitored.is_empty() {
            break;
        }

        let rhs_system = system;
        let jacobian_system = system;
        let root_system = system;
        let root_count = monitored.len();
        let initial_step = options.initial_step_seconds.min(remaining);
        let problem = OdeBuilder::<NalgebraMat<f64>>::new()
            .h0(initial_step)
            .rtol(options.relative_tolerance)
            .atol(vec![
                options.absolute_tolerance_moles;
                network.reactions.len()
            ])
            .rhs_implicit(
                move |x, _parameters, _time, output| rhs_system.rhs(x, output),
                move |x, _parameters, _time, vector, output| {
                    jacobian_system.jacobian_vector(x, vector, output);
                },
            )
            .root(
                move |x, _parameters, _time, output| {
                    for (index, (species, phase)) in monitored.iter().enumerate() {
                        output[index] = root_system.amount(x, species, *phase) - DEPLETION_EVENT;
                    }
                },
                root_count,
            )
            .init(
                |_parameters, _time, output| {
                    for index in 0..network.reactions.len() {
                        output[index] = 0.0;
                    }
                },
                network.reactions.len(),
            )
            .build()
            .map_err(|error| IntegrationError::Solver {
                network: network.id.to_string(),
                detail: error.to_string(),
            })?;
        let mut solver =
            problem
                .bdf::<NalgebraLU<f64>>()
                .map_err(|error| IntegrationError::Solver {
                    network: network.id.to_string(),
                    detail: error.to_string(),
                })?;
        let (_, _, stop_reason) =
            solver
                .solve(remaining)
                .map_err(|error| IntegrationError::Solver {
                    network: network.id.to_string(),
                    detail: error.to_string(),
                })?;
        let solver_statistics = solver.get_statistics();
        statistics.accepted_steps += solver_statistics.number_of_steps;
        statistics.rejected_steps += solver_statistics.number_of_error_test_failures;
        statistics.nonlinear_iterations += solver_statistics.number_of_nonlinear_solver_iterations;
        statistics.nonlinear_failures += solver_statistics.number_of_nonlinear_solver_fails;

        let proposed = (0..network.reactions.len())
            .map(|index| solver.state().y[index])
            .collect::<Vec<_>>();
        if proposed.iter().any(|extent| !extent.is_finite()) {
            return Err(IntegrationError::Solver {
                network: network.id.to_string(),
                detail: "solver returned a non-finite reaction extent".to_string(),
            });
        }
        let advanced = match stop_reason {
            OdeSolverStopReason::RootFound(time, _) => time,
            OdeSolverStopReason::TstopReached => remaining,
            OdeSolverStopReason::InternalTimestep => solver.state().t,
        };
        // Both values own closures borrowing the vessel snapshot. End those
        // borrows before committing the calculated transition.
        drop(solver);
        drop(problem);

        let accepted_fraction = apply_coupled_extents(vessel, network.reactions, &proposed);
        if accepted_fraction < 1.0 {
            statistics.constrained_commits += 1;
        }
        for (total, extent) in totals.iter_mut().zip(proposed) {
            *total += extent * accepted_fraction;
        }

        if matches!(stop_reason, OdeSolverStopReason::RootFound(_, _)) {
            statistics.depletion_events += 1;
        }
        if advanced <= remaining.max(1.0) * f64::EPSILON {
            return Err(IntegrationError::NoProgress {
                network: network.id.to_string(),
            });
        }
        elapsed += advanced;
        if matches!(stop_reason, OdeSolverStopReason::TstopReached) {
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

// ──────────────────────────────────────────────────────────────────────
// Public API for external integrators (e.g. kerotakis-sundials)
// ──────────────────────────────────────────────────────────────────────

/// Evaluate the extent-space RHS for a reaction network at the given extents.
///
/// `extents` and `output` must both have length `network.reactions.len()`.
/// Each `output[i]` receives d(extent_i)/dt in mol/s.
///
/// This is the same RHS used by the built-in diffsol integrator, exposed so
/// alternative backends can produce identical trajectories.
pub fn extent_rhs(
    vessel: &Vessel,
    network: &ReactionNetwork<'_>,
    extents: &[f64],
    output: &mut [f64],
) {
    let system = ExtentSystem {
        vessel,
        reactions: network.reactions,
    };
    let ext_vec: Vec<f64> = extents.to_vec();
    let mut out_vec = output.to_vec();
    system.rhs(&ext_vec, &mut out_vec);
    output.copy_from_slice(&out_vec);
}

/// Return the (species, phase) keys that can be consumed by the network from
/// the current vessel state. External integrators use these for root-finding
/// (depletion detection).
pub fn consumable_keys<'a>(
    vessel: &'a Vessel,
    network: &'a ReactionNetwork<'a>,
) -> Vec<(&'a str, Phase)> {
    let system = ExtentSystem {
        vessel,
        reactions: network.reactions,
    };
    consumed_keys(&system)
}

/// Compute the amount of a species in a given phase after applying reaction
/// extents to the vessel snapshot.
pub fn amount_at_extents(
    vessel: &Vessel,
    network: &ReactionNetwork<'_>,
    extents: &[f64],
    species: &str,
    phase: Phase,
) -> f64 {
    let system = ExtentSystem {
        vessel,
        reactions: network.reactions,
    };
    let ext_vec: Vec<f64> = extents.to_vec();
    system.amount(&ext_vec, species, phase)
}

/// Commit extents to a vessel, returning the fraction actually applied
/// (may be < 1.0 if a reactant would go negative).
pub fn commit_extents(vessel: &mut Vessel, network: &ReactionNetwork<'_>, extents: &[f64]) -> f64 {
    apply_coupled_extents(vessel, network.reactions, extents)
}

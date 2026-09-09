//! Whole-curve polarization prediction and parameter fitting.
//!
//! Measurements carry their potential reference and electrode area.  The
//! model therefore compares signed geometric current densities on the SHE
//! scale instead of silently mixing instrument conventions.  A curve is the
//! sum of independently parameterised partial reactions plus, during a
//! sweep, the double-layer charging current `C_dl dE/dt`.

use crate::electrochemistry::{ElectrochemistryError, ElectrodeKineticModel, PartialReaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PotentialReference {
    StandardHydrogen,
    /// A declared experimental reference.  Its offset must come from the
    /// measurement metadata for the actual filling solution and temperature.
    DeclaredOffset {
        label: String,
        volts_vs_she: f64,
    },
}

impl PotentialReference {
    pub fn to_she(&self, measured_v: f64) -> Result<f64, PolarizationError> {
        let offset = match self {
            Self::StandardHydrogen => 0.0,
            Self::DeclaredOffset {
                label,
                volts_vs_she,
            } => {
                if label.trim().is_empty() || !volts_vs_she.is_finite() {
                    return Err(PolarizationError::InvalidObservation(
                        "reference label and SHE offset must be explicit and finite",
                    ));
                }
                *volts_vs_she
            }
        };
        if !measured_v.is_finite() {
            return Err(PolarizationError::InvalidObservation(
                "measured potential must be finite",
            ));
        }
        Ok(measured_v + offset)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarizationObservation {
    /// Potential reported by the instrument, against `reference`.
    pub measured_potential_v: f64,
    pub measured_current_a: f64,
    pub geometric_area_m2: f64,
    pub temperature_k: f64,
    pub reference: PotentialReference,
    /// Signed scan rate. Positive scans anodically; zero is a hold.
    #[serde(default)]
    pub sweep_rate_v_per_s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_s: Option<f64>,
}

impl PolarizationObservation {
    pub fn potential_she_v(&self) -> Result<f64, PolarizationError> {
        self.validate()?;
        self.reference.to_she(self.measured_potential_v)
    }

    pub fn current_density_a_per_m2(&self) -> Result<f64, PolarizationError> {
        self.validate()?;
        Ok(self.measured_current_a / self.geometric_area_m2)
    }

    pub fn validate(&self) -> Result<(), PolarizationError> {
        if !self.measured_current_a.is_finite()
            || !self.geometric_area_m2.is_finite()
            || self.geometric_area_m2 <= 0.0
            || !self.temperature_k.is_finite()
            || self.temperature_k <= 0.0
            || !self.sweep_rate_v_per_s.is_finite()
            || self
                .time_s
                .is_some_and(|time| !time.is_finite() || time < 0.0)
        {
            return Err(PolarizationError::InvalidObservation(
                "current, positive area and temperature, scan rate, and optional non-negative time must be finite",
            ));
        }
        self.reference.to_she(self.measured_potential_v)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarizationReaction {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_record_id: Option<String>,
    /// Absolute equilibrium potential on the SHE scale.
    pub equilibrium_potential_she_v: f64,
    pub kinetics: ElectrodeKineticModel,
    pub reactive_area_ratio: f64,
    pub film_resistance_ohm_m2: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiting_current_anodic_a_per_m2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limiting_current_cathodic_a_per_m2: Option<f64>,
}

impl PolarizationReaction {
    fn borrowed(&self) -> PartialReaction<'_> {
        PartialReaction {
            id: &self.id,
            parameter_record_id: self.parameter_record_id.as_deref(),
            equilibrium_potential_v: self.equilibrium_potential_she_v,
            kinetics: self.kinetics,
            reactive_area_ratio: self.reactive_area_ratio,
            film_resistance_ohm_m2: self.film_resistance_ohm_m2,
            limiting_current_anodic_a_per_m2: self.limiting_current_anodic_a_per_m2,
            limiting_current_cathodic_a_per_m2: self.limiting_current_cathodic_a_per_m2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarizationModel {
    pub reactions: Vec<PolarizationReaction>,
    #[serde(default)]
    pub double_layer_capacitance_f_per_m2: f64,
}

impl PolarizationModel {
    pub fn predict_current_density(
        &self,
        observation: &PolarizationObservation,
    ) -> Result<PolarizationPrediction, PolarizationError> {
        observation.validate()?;
        if self.reactions.is_empty()
            || !self.double_layer_capacitance_f_per_m2.is_finite()
            || self.double_layer_capacitance_f_per_m2 < 0.0
        {
            return Err(PolarizationError::InvalidModel(
                "a curve needs reactions and a finite non-negative double-layer capacitance",
            ));
        }
        if self
            .reactions
            .iter()
            .any(|reaction| reaction.id.trim().is_empty())
            || self.reactions.iter().enumerate().any(|(index, reaction)| {
                self.reactions[..index]
                    .iter()
                    .any(|prior| prior.id == reaction.id)
            })
        {
            return Err(PolarizationError::InvalidModel(
                "partial reactions must have unique non-empty identifiers",
            ));
        }
        let potential = observation.potential_she_v()?;
        let mut partial_currents = Vec::with_capacity(self.reactions.len());
        for reaction in &self.reactions {
            let current = reaction
                .borrowed()
                .current_density(potential, observation.temperature_k)
                .map_err(PolarizationError::Electrochemistry)?;
            partial_currents.push((reaction.id.clone(), current));
        }
        let faradaic = partial_currents.iter().map(|(_, current)| current).sum();
        let capacitive = self.double_layer_capacitance_f_per_m2 * observation.sweep_rate_v_per_s;
        Ok(PolarizationPrediction {
            potential_she_v: potential,
            partial_current_densities_a_per_m2: partial_currents,
            faradaic_current_density_a_per_m2: faradaic,
            capacitive_current_density_a_per_m2: capacitive,
            total_current_density_a_per_m2: faradaic + capacitive,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarizationPrediction {
    pub potential_she_v: f64,
    pub partial_current_densities_a_per_m2: Vec<(String, f64)>,
    pub faradaic_current_density_a_per_m2: f64,
    pub capacitive_current_density_a_per_m2: f64,
    pub total_current_density_a_per_m2: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FitSelector {
    EquilibriumPotential { reaction: usize },
    ExchangeCurrentDensity { reaction: usize },
    TafelSlope { reaction: usize },
    ButlerVolmerAnodicTransferCoefficient { reaction: usize },
    ButlerVolmerCathodicTransferCoefficient { reaction: usize },
    PassivationOnsetOverpotential { reaction: usize },
    PassiveCurrentDensity { reaction: usize },
    TranspassiveOnsetOverpotential { reaction: usize },
    TranspassiveTafelSlope { reaction: usize },
    ReactiveAreaRatio { reaction: usize },
    FilmResistance { reaction: usize },
    LimitingCurrentAnodic { reaction: usize },
    LimitingCurrentCathodic { reaction: usize },
    DoubleLayerCapacitance,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FitScale {
    #[default]
    Linear,
    Log,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitParameter {
    pub selector: FitSelector,
    pub lower: f64,
    pub upper: f64,
    pub initial: f64,
    #[serde(default)]
    pub scale: FitScale,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FitOptions {
    pub max_iterations: usize,
    /// Every nth observation is withheld. Zero disables holdout validation.
    pub holdout_every: usize,
    /// Scale inside the signed asinh residual. This preserves the zero
    /// crossing while preventing a high-current tail from owning the fit.
    pub current_scale_a_per_m2: f64,
}

impl Default for FitOptions {
    fn default() -> Self {
        Self {
            max_iterations: 400,
            holdout_every: 5,
            current_scale_a_per_m2: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FitDiagnostics {
    pub converged: bool,
    pub iterations: usize,
    pub evaluations: usize,
    pub training_points: usize,
    pub holdout_points: usize,
    pub signed_rmse_a_per_m2: f64,
    pub asinh_rmse: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holdout_signed_rmse_a_per_m2: Option<f64>,
    pub jacobian_rank: usize,
    pub parameter_count: usize,
    pub at_bounds: Vec<FitSelector>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolarizationFit {
    pub model: PolarizationModel,
    pub parameters: Vec<FittedParameter>,
    pub diagnostics: FitDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FittedParameter {
    pub selector: FitSelector,
    pub value: f64,
    pub lower: f64,
    pub upper: f64,
    pub at_bound: bool,
    pub scale: FitScale,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolarizationError {
    InvalidObservation(&'static str),
    InvalidModel(&'static str),
    InvalidFit(&'static str),
    Electrochemistry(ElectrochemistryError),
}

/// Fit explicitly selected parameters to an entire signed polarization curve.
/// Bounds are part of the model contract, never inferred from the data.
pub fn fit_polarization_curve(
    model: &PolarizationModel,
    observations: &[PolarizationObservation],
    parameters: &[FitParameter],
    options: FitOptions,
) -> Result<PolarizationFit, PolarizationError> {
    if parameters.is_empty()
        || options.max_iterations == 0
        || !options.current_scale_a_per_m2.is_finite()
        || options.current_scale_a_per_m2 <= 0.0
    {
        return Err(PolarizationError::InvalidFit(
            "fit parameters, iterations, and a positive finite current scale are required",
        ));
    }
    for observation in observations {
        observation.validate()?;
    }
    for parameter in parameters {
        if !parameter.lower.is_finite()
            || !parameter.upper.is_finite()
            || !parameter.initial.is_finite()
            || parameter.lower >= parameter.upper
            || !(parameter.lower..=parameter.upper).contains(&parameter.initial)
            || (parameter.scale == FitScale::Log && parameter.lower <= 0.0)
        {
            return Err(PolarizationError::InvalidFit(
                "each parameter needs finite ordered bounds containing its initial value; logarithmic bounds must be positive",
            ));
        }
    }
    if parameters.iter().enumerate().any(|(index, parameter)| {
        parameters[..index]
            .iter()
            .any(|prior| prior.selector == parameter.selector)
    }) {
        return Err(PolarizationError::InvalidFit(
            "each fitted parameter selector must be unique",
        ));
    }
    let training: Vec<usize> = (0..observations.len())
        .filter(|index| options.holdout_every == 0 || (index + 1) % options.holdout_every != 0)
        .collect();
    let holdout: Vec<usize> = (0..observations.len())
        .filter(|index| options.holdout_every != 0 && (index + 1) % options.holdout_every == 0)
        .collect();
    if training.len() <= parameters.len() {
        return Err(PolarizationError::InvalidFit(
            "the training curve must contain more observations than fitted parameters",
        ));
    }

    let n = parameters.len();
    let initial: Vec<f64> = parameters.iter().map(normalize_parameter).collect();
    let mut simplex = Vec::with_capacity(n + 1);
    simplex.push(initial.clone());
    for axis in 0..n {
        let mut vertex = initial.clone();
        vertex[axis] = (vertex[axis] + 0.08).min(1.0);
        if (vertex[axis] - initial[axis]).abs() < 1e-12 {
            vertex[axis] = (vertex[axis] - 0.16).max(0.0);
        }
        simplex.push(vertex);
    }
    let mut evaluations = 0;
    let objective = |point: &[f64]| -> Result<f64, PolarizationError> {
        let candidate = apply_point(model, parameters, point)?;
        training.iter().try_fold(0.0, |sum, &index| {
            let observed = observations[index].current_density_a_per_m2()?;
            let predicted = candidate
                .predict_current_density(&observations[index])?
                .total_current_density_a_per_m2;
            let residual = (predicted / options.current_scale_a_per_m2).asinh()
                - (observed / options.current_scale_a_per_m2).asinh();
            Ok(sum + residual * residual)
        })
    };
    let mut values = simplex
        .iter()
        .map(|point| {
            evaluations += 1;
            objective(point)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut converged = false;
    let mut iterations = 0;
    for iteration in 0..options.max_iterations {
        iterations = iteration + 1;
        let mut order: Vec<usize> = (0..=n).collect();
        order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
        simplex = order.iter().map(|&index| simplex[index].clone()).collect();
        values = order.iter().map(|&index| values[index]).collect();
        let diameter = simplex[1..]
            .iter()
            .flat_map(|point| point.iter().zip(&simplex[0]))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        if diameter < 1e-8 && (values[n] - values[0]).abs() < 1e-10 {
            converged = true;
            break;
        }
        let centroid: Vec<f64> = (0..n)
            .map(|axis| simplex[..n].iter().map(|point| point[axis]).sum::<f64>() / n as f64)
            .collect();
        let trial = |factor: f64| -> Vec<f64> {
            centroid
                .iter()
                .zip(&simplex[n])
                .map(|(middle, worst)| (middle + factor * (middle - worst)).clamp(0.0, 1.0))
                .collect()
        };
        let reflected = trial(1.0);
        evaluations += 1;
        let reflected_value = objective(&reflected)?;
        if reflected_value < values[0] {
            let expanded = trial(2.0);
            evaluations += 1;
            let expanded_value = objective(&expanded)?;
            if expanded_value < reflected_value {
                simplex[n] = expanded;
                values[n] = expanded_value;
            } else {
                simplex[n] = reflected;
                values[n] = reflected_value;
            }
        } else if reflected_value < values[n - 1] {
            simplex[n] = reflected;
            values[n] = reflected_value;
        } else {
            let contracted = trial(0.5);
            evaluations += 1;
            let contracted_value = objective(&contracted)?;
            if contracted_value < values[n] {
                simplex[n] = contracted;
                values[n] = contracted_value;
            } else {
                let best_vertex = simplex[0].clone();
                for vertex in simplex.iter_mut().take(n + 1).skip(1) {
                    for (coordinate, best_coordinate) in vertex.iter_mut().zip(&best_vertex) {
                        *coordinate = 0.5 * (*coordinate + best_coordinate);
                    }
                    evaluations += 1;
                }
                for vertex in 1..=n {
                    values[vertex] = objective(&simplex[vertex])?;
                }
            }
        }
    }
    let mut order: Vec<usize> = (0..=n).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let best = &simplex[order[0]];
    let fitted_model = apply_point(model, parameters, best)?;
    let fitted_parameters: Vec<FittedParameter> = parameters
        .iter()
        .zip(best)
        .map(|(parameter, &coordinate)| {
            let value = denormalize_parameter(coordinate, parameter);
            FittedParameter {
                selector: parameter.selector.clone(),
                value,
                lower: parameter.lower,
                upper: parameter.upper,
                at_bound: coordinate <= 1e-6 || 1.0 - coordinate <= 1e-6,
                scale: parameter.scale,
            }
        })
        .collect();
    let rank = jacobian_rank(
        &fitted_model,
        observations,
        parameters,
        best,
        &training,
        options.current_scale_a_per_m2,
    )?;
    if rank < n {
        return Err(PolarizationError::InvalidFit(
            "the selected parameters are not independently identifiable from this curve",
        ));
    }
    let (signed_rmse, asinh_rmse) = metrics(
        &fitted_model,
        observations,
        &training,
        options.current_scale_a_per_m2,
    )?;
    let holdout_rmse = (!holdout.is_empty())
        .then(|| {
            metrics(
                &fitted_model,
                observations,
                &holdout,
                options.current_scale_a_per_m2,
            )
        })
        .transpose()?
        .map(|metrics| metrics.0);
    let at_bounds = fitted_parameters
        .iter()
        .filter(|parameter| parameter.at_bound)
        .map(|parameter| parameter.selector.clone())
        .collect();
    Ok(PolarizationFit {
        model: fitted_model,
        parameters: fitted_parameters,
        diagnostics: FitDiagnostics {
            converged,
            iterations,
            evaluations,
            training_points: training.len(),
            holdout_points: holdout.len(),
            signed_rmse_a_per_m2: signed_rmse,
            asinh_rmse,
            holdout_signed_rmse_a_per_m2: holdout_rmse,
            jacobian_rank: rank,
            parameter_count: n,
            at_bounds,
        },
    })
}

fn apply_point(
    model: &PolarizationModel,
    parameters: &[FitParameter],
    point: &[f64],
) -> Result<PolarizationModel, PolarizationError> {
    let mut candidate = model.clone();
    for (parameter, &coordinate) in parameters.iter().zip(point) {
        let value = denormalize_parameter(coordinate, parameter);
        match parameter.selector {
            FitSelector::EquilibriumPotential { reaction } => {
                reaction_mut(&mut candidate, reaction)?.equilibrium_potential_she_v = value;
            }
            FitSelector::ReactiveAreaRatio { reaction } => {
                reaction_mut(&mut candidate, reaction)?.reactive_area_ratio = value;
            }
            FitSelector::FilmResistance { reaction } => {
                reaction_mut(&mut candidate, reaction)?.film_resistance_ohm_m2 = value;
            }
            FitSelector::LimitingCurrentAnodic { reaction } => {
                reaction_mut(&mut candidate, reaction)?.limiting_current_anodic_a_per_m2 =
                    Some(value);
            }
            FitSelector::LimitingCurrentCathodic { reaction } => {
                reaction_mut(&mut candidate, reaction)?.limiting_current_cathodic_a_per_m2 =
                    Some(value);
            }
            FitSelector::DoubleLayerCapacitance => {
                candidate.double_layer_capacitance_f_per_m2 = value;
            }
            FitSelector::ExchangeCurrentDensity { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ButlerVolmer { parameters } => parameters.j0 = value,
                    ElectrodeKineticModel::DirectionalTafel {
                        exchange_current_density_a_per_m2,
                        ..
                    }
                    | ElectrodeKineticModel::ActivePassive {
                        exchange_current_density_a_per_m2,
                        ..
                    } => *exchange_current_density_a_per_m2 = value,
                }
            }
            FitSelector::TafelSlope { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::DirectionalTafel {
                        tafel_slope_v_per_decade,
                        ..
                    } => *tafel_slope_v_per_decade = value,
                    ElectrodeKineticModel::ActivePassive {
                        active_tafel_slope_v_per_decade,
                        ..
                    } => *active_tafel_slope_v_per_decade = value,
                    ElectrodeKineticModel::ButlerVolmer { .. } => {
                        return Err(PolarizationError::InvalidFit(
                            "Tafel-slope fitting requires a Tafel or active/passive reaction",
                        ));
                    }
                }
            }
            FitSelector::ButlerVolmerAnodicTransferCoefficient { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ButlerVolmer { parameters } => {
                        parameters.alpha_a = value;
                    }
                    _ => {
                        return Err(PolarizationError::InvalidFit(
                            "an anodic transfer coefficient requires Butler-Volmer kinetics",
                        ));
                    }
                }
            }
            FitSelector::ButlerVolmerCathodicTransferCoefficient { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ButlerVolmer { parameters } => {
                        parameters.alpha_c = value;
                    }
                    _ => {
                        return Err(PolarizationError::InvalidFit(
                            "a cathodic transfer coefficient requires Butler-Volmer kinetics",
                        ));
                    }
                }
            }
            FitSelector::PassivationOnsetOverpotential { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ActivePassive {
                        passivation_onset_overpotential_v,
                        ..
                    } => *passivation_onset_overpotential_v = value,
                    _ => {
                        return Err(PolarizationError::InvalidFit(
                            "passivation onset requires active/passive kinetics",
                        ));
                    }
                }
            }
            FitSelector::PassiveCurrentDensity { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ActivePassive {
                        passive_current_density_a_per_m2,
                        ..
                    } => *passive_current_density_a_per_m2 = value,
                    _ => {
                        return Err(PolarizationError::InvalidFit(
                            "passive current requires active/passive kinetics",
                        ));
                    }
                }
            }
            FitSelector::TranspassiveOnsetOverpotential { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ActivePassive {
                        transpassive: Some(branch),
                        ..
                    } => branch.onset_overpotential_v = value,
                    _ => {
                        return Err(PolarizationError::InvalidFit(
                            "transpassive onset requires a configured transpassive branch",
                        ));
                    }
                }
            }
            FitSelector::TranspassiveTafelSlope { reaction } => {
                let kinetics = &mut reaction_mut(&mut candidate, reaction)?.kinetics;
                match kinetics {
                    ElectrodeKineticModel::ActivePassive {
                        transpassive: Some(branch),
                        ..
                    } => branch.tafel_slope_v_per_decade = value,
                    _ => {
                        return Err(PolarizationError::InvalidFit(
                            "transpassive slope requires a configured transpassive branch",
                        ));
                    }
                }
            }
        }
    }
    // Exercise validation at one benign point before the optimizer starts.
    for reaction in &candidate.reactions {
        reaction
            .borrowed()
            .current_density(reaction.equilibrium_potential_she_v, 298.15)
            .map_err(PolarizationError::Electrochemistry)?;
    }
    Ok(candidate)
}

fn reaction_mut(
    model: &mut PolarizationModel,
    index: usize,
) -> Result<&mut PolarizationReaction, PolarizationError> {
    model
        .reactions
        .get_mut(index)
        .ok_or(PolarizationError::InvalidFit(
            "fit selector names a reaction outside the model",
        ))
}

fn metrics(
    model: &PolarizationModel,
    observations: &[PolarizationObservation],
    indices: &[usize],
    scale: f64,
) -> Result<(f64, f64), PolarizationError> {
    let mut signed_sse = 0.0;
    let mut transformed_sse = 0.0;
    for &index in indices {
        let observed = observations[index].current_density_a_per_m2()?;
        let predicted = model
            .predict_current_density(&observations[index])?
            .total_current_density_a_per_m2;
        signed_sse += (predicted - observed).powi(2);
        transformed_sse += ((predicted / scale).asinh() - (observed / scale).asinh()).powi(2);
    }
    Ok((
        (signed_sse / indices.len() as f64).sqrt(),
        (transformed_sse / indices.len() as f64).sqrt(),
    ))
}

fn jacobian_rank(
    fitted: &PolarizationModel,
    observations: &[PolarizationObservation],
    parameters: &[FitParameter],
    point: &[f64],
    indices: &[usize],
    scale: f64,
) -> Result<usize, PolarizationError> {
    let mut columns = Vec::with_capacity(parameters.len());
    for axis in 0..parameters.len() {
        let step = 1e-5;
        let mut lower = point.to_vec();
        let mut upper = point.to_vec();
        lower[axis] = (lower[axis] - step).max(0.0);
        upper[axis] = (upper[axis] + step).min(1.0);
        let denominator = upper[axis] - lower[axis];
        let low_model = apply_point(fitted, parameters, &lower)?;
        let high_model = apply_point(fitted, parameters, &upper)?;
        let mut column = Vec::with_capacity(indices.len());
        for &index in indices {
            let low = low_model
                .predict_current_density(&observations[index])?
                .total_current_density_a_per_m2;
            let high = high_model
                .predict_current_density(&observations[index])?
                .total_current_density_a_per_m2;
            column.push(((high / scale).asinh() - (low / scale).asinh()) / denominator);
        }
        columns.push(column);
    }
    // Modified Gram-Schmidt on Jacobian columns.  Coordinates are normalized
    // to their bounds, making the relative threshold meaningful across units.
    let largest = columns
        .iter()
        .map(|column| dot(column, column).sqrt())
        .fold(0.0, f64::max);
    let tolerance = largest * 1e-7;
    let mut basis: Vec<Vec<f64>> = Vec::new();
    for mut column in columns {
        for vector in &basis {
            let projection = dot(&column, vector);
            for (entry, unit) in column.iter_mut().zip(vector) {
                *entry -= projection * unit;
            }
        }
        let norm = dot(&column, &column).sqrt();
        if norm > tolerance {
            for entry in &mut column {
                *entry /= norm;
            }
            basis.push(column);
        }
    }
    Ok(basis.len())
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(left, right)| left * right).sum()
}

fn normalize_parameter(parameter: &FitParameter) -> f64 {
    match parameter.scale {
        FitScale::Linear => {
            (parameter.initial - parameter.lower) / (parameter.upper - parameter.lower)
        }
        FitScale::Log => {
            (parameter.initial.ln() - parameter.lower.ln())
                / (parameter.upper.ln() - parameter.lower.ln())
        }
    }
}

fn denormalize_parameter(coordinate: f64, parameter: &FitParameter) -> f64 {
    let coordinate = coordinate.clamp(0.0, 1.0);
    match parameter.scale {
        FitScale::Linear => parameter.lower + coordinate * (parameter.upper - parameter.lower),
        FitScale::Log => (parameter.lower.ln()
            + coordinate * (parameter.upper.ln() - parameter.lower.ln()))
        .exp(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::electrochemistry::TafelDirection;

    fn reaction(direction: TafelDirection, equilibrium: f64, j0: f64) -> PolarizationReaction {
        PolarizationReaction {
            id: format!("{direction:?}"),
            parameter_record_id: None,
            equilibrium_potential_she_v: equilibrium,
            kinetics: ElectrodeKineticModel::DirectionalTafel {
                direction,
                exchange_current_density_a_per_m2: j0,
                tafel_slope_v_per_decade: 0.12,
                electrons_per_extent: 1.0,
            },
            reactive_area_ratio: 1.0,
            film_resistance_ohm_m2: 0.0,
            limiting_current_anodic_a_per_m2: None,
            limiting_current_cathodic_a_per_m2: None,
        }
    }

    fn observation(potential: f64, current_density: f64) -> PolarizationObservation {
        PolarizationObservation {
            measured_potential_v: potential - 0.210,
            measured_current_a: current_density * 2.0e-4,
            geometric_area_m2: 2.0e-4,
            temperature_k: 298.15,
            reference: PotentialReference::DeclaredOffset {
                label: "declared laboratory reference".into(),
                volts_vs_she: 0.210,
            },
            sweep_rate_v_per_s: 0.0,
            time_s: None,
        }
    }

    #[test]
    fn reference_area_and_capacitance_are_explicit() {
        let model = PolarizationModel {
            reactions: vec![reaction(TafelDirection::Anodic, -0.4, 0.2)],
            double_layer_capacitance_f_per_m2: 0.5,
        };
        let mut point = observation(-0.4, 0.0);
        point.sweep_rate_v_per_s = 0.002;
        let prediction = model.predict_current_density(&point).unwrap();
        assert!((prediction.potential_she_v + 0.4).abs() < 1e-12);
        assert!((prediction.faradaic_current_density_a_per_m2 - 0.2).abs() < 1e-12);
        assert!((prediction.capacitive_current_density_a_per_m2 - 0.001).abs() < 1e-12);
    }

    #[test]
    fn fits_a_mixed_whole_curve_and_reports_holdout() {
        let truth = PolarizationModel {
            reactions: vec![
                reaction(TafelDirection::Anodic, -0.42, 0.08),
                reaction(TafelDirection::Cathodic, -0.05, 0.003),
            ],
            double_layer_capacitance_f_per_m2: 0.0,
        };
        let observations: Vec<_> = (-30..=30)
            .map(|step| {
                let potential = -0.5 + step as f64 * 0.01;
                let shell = observation(potential, 0.0);
                let current = truth
                    .predict_current_density(&shell)
                    .unwrap()
                    .total_current_density_a_per_m2;
                observation(potential, current)
            })
            .collect();
        let mut initial = truth.clone();
        initial.reactions[0].kinetics = ElectrodeKineticModel::DirectionalTafel {
            direction: TafelDirection::Anodic,
            exchange_current_density_a_per_m2: 0.03,
            tafel_slope_v_per_decade: 0.12,
            electrons_per_extent: 1.0,
        };
        let fit = fit_polarization_curve(
            &initial,
            &observations,
            &[FitParameter {
                selector: FitSelector::ExchangeCurrentDensity { reaction: 0 },
                lower: 0.005,
                upper: 0.2,
                initial: 0.03,
                scale: FitScale::Log,
            }],
            FitOptions {
                current_scale_a_per_m2: 0.01,
                ..FitOptions::default()
            },
        )
        .unwrap();
        assert!((fit.parameters[0].value - 0.08).abs() < 1e-5);
        assert!(fit.diagnostics.signed_rmse_a_per_m2 < 1e-5);
        assert!(fit.diagnostics.holdout_signed_rmse_a_per_m2.unwrap() < 1e-5);
        assert_eq!(fit.diagnostics.jacobian_rank, 1);
    }

    #[test]
    fn refuses_unidentifiable_duplicate_scales() {
        let model = PolarizationModel {
            reactions: vec![reaction(TafelDirection::Anodic, -0.4, 0.1)],
            double_layer_capacitance_f_per_m2: 0.0,
        };
        let observations: Vec<_> = (0..12)
            .map(|step| observation(-0.5 + step as f64 * 0.02, 0.1))
            .collect();
        let error = fit_polarization_curve(
            &model,
            &observations,
            &[
                FitParameter {
                    selector: FitSelector::ExchangeCurrentDensity { reaction: 0 },
                    lower: 0.01,
                    upper: 1.0,
                    initial: 0.1,
                    scale: FitScale::Log,
                },
                FitParameter {
                    selector: FitSelector::ReactiveAreaRatio { reaction: 0 },
                    lower: 0.1,
                    upper: 2.0,
                    initial: 1.0,
                    scale: FitScale::Linear,
                },
            ],
            FitOptions::default(),
        )
        .unwrap_err();
        assert_eq!(
            error,
            PolarizationError::InvalidFit(
                "the selected parameters are not independently identifiable from this curve"
            )
        );
    }

    #[test]
    fn refuses_missing_reference_metadata() {
        let mut point = observation(0.0, 0.0);
        point.reference = PotentialReference::DeclaredOffset {
            label: String::new(),
            volts_vs_she: 0.0,
        };
        assert!(point.validate().is_err());
    }

    #[test]
    fn fit_selectors_reach_transport_and_passivation_parameters() {
        let model = PolarizationModel {
            reactions: vec![PolarizationReaction {
                id: "passive".into(),
                parameter_record_id: None,
                equilibrium_potential_she_v: -0.4,
                kinetics: ElectrodeKineticModel::ActivePassive {
                    exchange_current_density_a_per_m2: 0.1,
                    active_tafel_slope_v_per_decade: 0.1,
                    electrons_per_extent: 2.0,
                    passivation_onset_overpotential_v: 0.2,
                    passive_current_density_a_per_m2: 0.01,
                    transpassive: Some(crate::electrochemistry::TranspassiveBranch {
                        onset_overpotential_v: 0.8,
                        tafel_slope_v_per_decade: 0.2,
                    }),
                },
                reactive_area_ratio: 1.0,
                film_resistance_ohm_m2: 0.0,
                limiting_current_anodic_a_per_m2: None,
                limiting_current_cathodic_a_per_m2: None,
            }],
            double_layer_capacitance_f_per_m2: 0.0,
        };
        let parameter = |selector, lower, upper| FitParameter {
            selector,
            lower,
            upper,
            initial: lower,
            scale: FitScale::Linear,
        };
        let parameters = vec![
            parameter(
                FitSelector::PassivationOnsetOverpotential { reaction: 0 },
                0.1,
                0.3,
            ),
            parameter(
                FitSelector::PassiveCurrentDensity { reaction: 0 },
                0.005,
                0.02,
            ),
            parameter(
                FitSelector::TranspassiveOnsetOverpotential { reaction: 0 },
                0.6,
                1.0,
            ),
            parameter(
                FitSelector::TranspassiveTafelSlope { reaction: 0 },
                0.1,
                0.3,
            ),
            parameter(FitSelector::LimitingCurrentAnodic { reaction: 0 }, 1.0, 3.0),
        ];
        let changed = apply_point(&model, &parameters, &[0.5; 5]).unwrap();
        let ElectrodeKineticModel::ActivePassive {
            passivation_onset_overpotential_v,
            passive_current_density_a_per_m2,
            transpassive: Some(transpassive),
            ..
        } = changed.reactions[0].kinetics
        else {
            panic!("active/passive model changed kind")
        };
        assert!((passivation_onset_overpotential_v - 0.2).abs() < 1e-12);
        assert!((passive_current_density_a_per_m2 - 0.0125).abs() < 1e-12);
        assert!((transpassive.onset_overpotential_v - 0.8).abs() < 1e-12);
        assert!((transpassive.tafel_slope_v_per_decade - 0.2).abs() < 1e-12);
        assert_eq!(
            changed.reactions[0].limiting_current_anodic_a_per_m2,
            Some(2.0)
        );
    }
}

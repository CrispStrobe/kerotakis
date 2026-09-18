//! INF-001: bounded two-parameter estimation over declared hypotheses.
//!
//! The unknown-water activity: the learner holds a hidden sample, gathers
//! calibrated measurements, and fits competing composition hypotheses. The
//! fitter here is deliberately modest: a bounded grid search over two
//! parameters per hypothesis, minimising squared error against the learner's
//! observations. What it refuses to do is pretend a single best fit settles
//! the question — it reports ambiguity so a learner can choose a
//! distinguishing measurement.
//!
//! Synthetic tests below validate the inference software only. A pass here
//! is not evidence that the chemistry matches the world.

use serde::{Deserialize, Serialize};

/// Why an inference input was refused.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InferenceError {
    #[error("{0}")]
    Config(String),
    #[error("grid bounds must be finite, got {0}")]
    NonFinite(f64),
    #[error("observation {index} references unknown observable '{observable}'")]
    UnknownObservable { index: usize, observable: String },
}

/// One declared hypothesis: a forward model from two parameters to the
/// predicted observables, plus the bounds the learner is allowed to search.
pub trait Hypothesis {
    fn name(&self) -> &str;
    /// (low, high) bounds for parameters a and b. Both must be finite.
    fn bounds(&self) -> ((f64, f64), (f64, f64));
    /// Forward model: predicted observables for one parameter pair.
    fn predict(&self, a: f64, b: f64) -> Vec<(&'static str, f64)>;
}

/// An observation the learner has made: which observable, what value, and
/// the declared measurement uncertainty (one sigma, same unit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub observable: String,
    pub value: f64,
    pub sigma: f64,
}

impl Observation {
    pub fn new(observable: &str, value: f64, sigma: f64) -> Result<Self, InferenceError> {
        if !value.is_finite() || !sigma.is_finite() || sigma <= 0.0 {
            return Err(InferenceError::Config(
                "observation value must be finite and sigma positive".into(),
            ));
        }
        Ok(Self {
            observable: observable.to_string(),
            value,
            sigma,
        })
    }
}

/// The learner's record of one hypothesis fit. Best-fit parameters, the
/// chi-squared at the fit, the runner-up's chi-squared, and whether the
/// data actually distinguish the two.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypothesisFit {
    pub name: String,
    pub best_a: f64,
    pub best_b: f64,
    pub chi_squared: f64,
    /// Degrees of freedom proxy: observations minus fitted parameters. Never
    /// negative here — an underdetermined fit reports zero and its
    /// `identifiable` flag says so rather than inventing precision.
    pub degrees_of_freedom: usize,
    /// True only when this grid has no nearby alternatives and at least two
    /// distinct observables. Not a proof of structural identifiability.
    pub identifiable: bool,
    /// Runner-up parameter pairs within the ambiguity threshold, if any.
    pub ambiguous_alternatives: Vec<(f64, f64)>,
}

/// Grid-search fit of a two-parameter hypothesis against observations.
/// The grid is the optimizer: bounds are honoured exactly, and the reported
/// best is the best *evaluated* point, which the bounds guarantee exists.
pub fn fit_two_parameters(
    hypothesis: &dyn Hypothesis,
    observations: &[Observation],
    grid_steps: usize,
) -> Result<HypothesisFit, InferenceError> {
    if grid_steps < 2 {
        return Err(InferenceError::Config(
            "the grid needs at least two steps so bounds are actually evaluated".into(),
        ));
    }
    if observations.is_empty() {
        return Err(InferenceError::Config(
            "fitting needs at least one observation".into(),
        ));
    }
    let ((a_lo, a_hi), (b_lo, b_hi)) = hypothesis.bounds();
    if !a_lo.is_finite() || !a_hi.is_finite() || !b_lo.is_finite() || !b_hi.is_finite() {
        return Err(InferenceError::NonFinite(f64::NAN));
    }
    if a_hi <= a_lo || b_hi <= b_lo {
        return Err(InferenceError::Config(
            "bounds must satisfy low < high for both parameters".into(),
        ));
    }
    for observation in observations {
        Observation::new(
            &observation.observable,
            observation.value,
            observation.sigma,
        )?;
    }

    let mut best: Option<(f64, f64, f64)> = None; // (sse, a, b)
    let mut candidates: Vec<(f64, f64, f64)> = Vec::new();
    for step_a in 0..grid_steps {
        let a = a_lo + (a_hi - a_lo) * step_a as f64 / (grid_steps - 1) as f64;
        for step_b in 0..grid_steps {
            let b = b_lo + (b_hi - b_lo) * step_b as f64 / (grid_steps - 1) as f64;
            let mut chi = 0.0;
            let predictions = hypothesis.predict(a, b);
            for (index, observation) in observations.iter().enumerate() {
                let mut matches = predictions
                    .iter()
                    .filter(|(name, _)| *name == observation.observable);
                let prediction =
                    matches
                        .next()
                        .ok_or_else(|| InferenceError::UnknownObservable {
                            index,
                            observable: observation.observable.clone(),
                        })?;
                if matches.next().is_some() || !prediction.1.is_finite() {
                    return Err(InferenceError::Config(
                        "predictions must be unique and finite".into(),
                    ));
                }
                let residual = (observation.value - prediction.1) / observation.sigma;
                chi += residual * residual;
            }
            if !chi.is_finite() {
                return Err(InferenceError::Config("non-finite fit score".into()));
            }
            candidates.push((chi, a, b));
            if best.is_none() || chi < best.unwrap().0 {
                best = Some((chi, a, b));
            }
        }
    }
    let (chi_squared, best_a, best_b) = best.unwrap();

    // Instructional ambiguity heuristic, NOT a confidence contour or a
    // structural-identifiability test. Grid resolution affects this result.
    let threshold = chi_squared + observations.len() as f64;
    let mut ambiguous_alternatives: Vec<(f64, f64)> = candidates
        .into_iter()
        .filter(|(chi, a, b)| {
            *chi <= threshold && !((*a - best_a).abs() < 1e-12 && (*b - best_b).abs() < 1e-12)
        })
        .map(|(_, a, b)| (a, b))
        .collect();
    ambiguous_alternatives.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    ambiguous_alternatives.truncate(8);

    Ok(HypothesisFit {
        name: hypothesis.name().to_string(),
        best_a,
        best_b,
        chi_squared,
        degrees_of_freedom: observations.len().saturating_sub(2),
        identifiable: ambiguous_alternatives.is_empty()
            && observations
                .iter()
                .map(|o| &o.observable)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                >= 2,
        ambiguous_alternatives,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Linear {
        slope_scale: f64,
    }

    impl Hypothesis for Linear {
        fn name(&self) -> &str {
            "linear"
        }
        fn bounds(&self) -> ((f64, f64), (f64, f64)) {
            ((0.0, 10.0 * self.slope_scale), (0.0, 1.0))
        }
        fn predict(&self, a: f64, b: f64) -> Vec<(&'static str, f64)> {
            vec![("x", a), ("y", b)]
        }
    }

    #[test]
    fn recovers_parameters_from_clean_data() {
        // True parameters: a = 4.0, b = 0.5. Predictions grid-aligned.
        let observations = vec![
            Observation::new("x", 4.0, 0.01).unwrap(),
            Observation::new("y", 0.5, 0.01).unwrap(),
        ];
        let fit = fit_two_parameters(&Linear { slope_scale: 1.0 }, &observations, 41).unwrap();
        assert!((fit.best_a - 4.0).abs() < 0.3, "{}", fit.best_a);
        assert!((fit.best_b - 0.5).abs() < 0.05, "{}", fit.best_b);
        assert!(fit.identifiable);
    }

    #[test]
    fn ambiguous_data_report_alternatives_not_certainty() {
        // Two observations that two different parameter pairs explain
        // equally well: the fit must say so.
        struct Degenerate;
        impl Hypothesis for Degenerate {
            fn name(&self) -> &str {
                "degenerate"
            }
            fn bounds(&self) -> ((f64, f64), (f64, f64)) {
                ((0.0, 10.0), (0.0, 10.0))
            }
            // The prediction depends only on a + b, so a = 2, b = 2 and
            // a = 4, b = 0 both explain "sum = 4".
            fn predict(&self, a: f64, b: f64) -> Vec<(&'static str, f64)> {
                vec![("sum", a + b)]
            }
        }
        let observations = vec![Observation::new("sum", 4.0, 0.01).unwrap()];
        let fit = fit_two_parameters(&Degenerate, &observations, 11).unwrap();
        assert!(!fit.identifiable);
        assert!(!fit.ambiguous_alternatives.is_empty());
    }

    #[test]
    fn refuses_unknown_observables_and_bad_config() {
        let observations = vec![Observation::new("nope", 1.0, 1.0).unwrap()];
        assert!(matches!(
            fit_two_parameters(&Linear { slope_scale: 1.0 }, &observations, 5),
            Err(InferenceError::UnknownObservable { index: 0, .. })
        ));
        assert!(fit_two_parameters(&Linear { slope_scale: 1.0 }, &[], 5).is_err());
        assert!(fit_two_parameters(&Linear { slope_scale: 1.0 }, &observations, 1).is_err());
        assert!(Observation::new("x", f64::NAN, 1.0).is_err());
        assert!(Observation::new("x", 1.0, 0.0).is_err());
    }
}

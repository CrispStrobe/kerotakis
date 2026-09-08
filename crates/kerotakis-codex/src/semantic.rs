//! Authored semantic claims over states captured during a real replay.
//!
//! The evaluator deliberately knows nothing about the engine. Hosts capture a
//! small, stable trace from their engine and pass it here, keeping the schema
//! usable by the CLI, browser, and future promotion tooling.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Equal,
    Increasing,
    Decreasing,
    Conserved,
    Unchanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub kind: Kind,
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
    pub samples: Vec<Sample>,
}

fn default_tolerance() -> f64 {
    1e-9
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// `initial`, `final`, or `after:N` (N is the one-based operation count).
    pub step: String,
    /// `mass_g`, `temperature_c`, `elapsed_s`, `ph`, or `moles:SPECIES`.
    pub metric: String,
    /// Vessel label. Omission sums additive metrics over the whole bench.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vessel: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trace {
    /// Includes the initial state at index zero and one state after each op.
    pub states: Vec<State>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    pub vessels: BTreeMap<String, VesselValues>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VesselValues {
    pub mass_g: f64,
    pub temperature_c: f64,
    pub pressure_kpa: f64,
    pub elapsed_s: f64,
    pub ph: Option<f64>,
    pub moles: BTreeMap<String, f64>,
}

impl Assertion {
    pub fn evaluate(&self, trace: &Trace) -> Result<(), String> {
        if !self.tolerance.is_finite() || self.tolerance < 0.0 {
            return Err("tolerance must be a finite non-negative number".into());
        }
        if self.samples.len() < 2 {
            return Err("a semantic assertion needs at least two samples".into());
        }
        let values = self
            .samples
            .iter()
            .map(|s| sample(s, trace))
            .collect::<Result<Vec<_>, _>>()?;
        let tol = self.tolerance;
        let ok = match self.kind {
            Kind::Equal | Kind::Conserved | Kind::Unchanged => {
                values[1..].iter().all(|v| (v - values[0]).abs() <= tol)
            }
            Kind::Increasing => values.windows(2).all(|w| w[1] > w[0] + tol),
            Kind::Decreasing => values.windows(2).all(|w| w[0] > w[1] + tol),
        };
        if ok {
            Ok(())
        } else {
            Err(format!(
                "{:?} failed: observed {values:?} (tolerance {tol})",
                self.kind
            ))
        }
    }
}

fn sample(spec: &Sample, trace: &Trace) -> Result<f64, String> {
    let index = match spec.step.as_str() {
        "initial" => 0,
        "final" => trace
            .states
            .len()
            .checked_sub(1)
            .ok_or("replay trace is empty")?,
        s if s.starts_with("after:") => s[6..]
            .parse::<usize>()
            .map_err(|_| format!("invalid step '{s}'"))?,
        s => {
            return Err(format!(
                "invalid step '{s}' (use initial, final, or after:N)"
            ))
        }
    };
    let state = trace
        .states
        .get(index)
        .ok_or_else(|| format!("step '{}' is outside this replay", spec.step))?;
    if let Some(label) = &spec.vessel {
        let vessel = state
            .vessels
            .get(label)
            .ok_or_else(|| format!("unknown vessel '{label}' at step '{}'", spec.step))?;
        vessel_metric(vessel, &spec.metric)
    } else {
        if spec.metric != "mass_g" && !spec.metric.starts_with("moles:") {
            return Err(format!("metric '{}' needs a vessel", spec.metric));
        }
        state
            .vessels
            .values()
            .map(|v| vessel_metric(v, &spec.metric))
            .sum()
    }
}

fn vessel_metric(v: &VesselValues, metric: &str) -> Result<f64, String> {
    match metric {
        "mass_g" => Ok(v.mass_g),
        "temperature_c" => Ok(v.temperature_c),
        "pressure_kpa" => Ok(v.pressure_kpa),
        "elapsed_s" => Ok(v.elapsed_s),
        "ph" => v.ph.ok_or_else(|| "pH was not characterised".into()),
        m if m.starts_with("moles:") => Ok(*v.moles.get(&m[6..]).unwrap_or(&0.0)),
        _ => Err(format!("unknown metric '{metric}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn trace(values: &[f64]) -> Trace {
        Trace {
            states: values
                .iter()
                .map(|x| State {
                    vessels: BTreeMap::from([(
                        "v1".into(),
                        VesselValues {
                            mass_g: *x,
                            ..Default::default()
                        },
                    )]),
                })
                .collect(),
        }
    }
    fn claim(kind: Kind) -> Assertion {
        Assertion {
            kind,
            tolerance: 1e-6,
            samples: vec![
                Sample {
                    step: "initial".into(),
                    metric: "mass_g".into(),
                    vessel: Some("v1".into()),
                },
                Sample {
                    step: "final".into(),
                    metric: "mass_g".into(),
                    vessel: Some("v1".into()),
                },
            ],
        }
    }
    #[test]
    fn evaluates_equal_ordered_and_failed_claims() {
        assert!(claim(Kind::Equal)
            .evaluate(&trace(&[2.0, 2.0000001]))
            .is_ok());
        assert!(claim(Kind::Increasing)
            .evaluate(&trace(&[2.0, 3.0]))
            .is_ok());
        assert!(claim(Kind::Decreasing)
            .evaluate(&trace(&[2.0, 3.0]))
            .is_err());
    }
    #[test]
    fn validates_selectors_instead_of_silently_skipping_them() {
        let mut c = claim(Kind::Equal);
        c.samples[1].step = "after:9".into();
        assert!(c.evaluate(&trace(&[1.0])).unwrap_err().contains("outside"));
        c.samples[1].step = "final".into();
        c.samples[1].metric = "temperature_c".into();
        c.samples[1].vessel = None;
        assert!(c
            .evaluate(&trace(&[1.0]))
            .unwrap_err()
            .contains("needs a vessel"));
    }
    #[test]
    fn authored_toml_is_backward_compatible_and_defaults_tolerance() {
        #[derive(Deserialize)]
        struct Wrapper {
            assertions: Vec<Assertion>,
        }
        let parsed: Wrapper = toml::from_str(
            r#"
            [[assertions]]
            kind = "unchanged"
            [[assertions.samples]]
            step = "after:1"
            metric = "moles:NaCl"
            vessel = "v1"
            [[assertions.samples]]
            step = "final"
            metric = "moles:NaCl"
            vessel = "v1"
        "#,
        )
        .unwrap();
        assert_eq!(parsed.assertions[0].tolerance, 1e-9);
        assert_eq!(parsed.assertions[0].kind, Kind::Unchanged);
    }
}

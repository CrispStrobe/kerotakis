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
    Ratio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    pub kind: Kind,
    #[serde(default = "default_tolerance")]
    pub tolerance: f64,
    /// Scale-aware slack for equality-like relations. Zero preserves the
    /// absolute-only behaviour of every assertion authored before this field.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub relative_tolerance: f64,
    /// Expected `second / first`, required only for [`Kind::Ratio`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f64>,
    pub samples: Vec<Sample>,
}

fn default_tolerance() -> f64 {
    1e-9
}

fn is_zero(value: &f64) -> bool {
    *value == 0.0
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
    /// Numeric fields emitted by the operation that produced this state.
    /// The initial state has none. Hosts decide which engine fields to expose;
    /// resolution here is generic and never guesses at an absent field.
    pub events: Vec<ScalarEvent>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScalarEvent {
    pub kind: String,
    pub fields: BTreeMap<String, f64>,
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
        if !self.relative_tolerance.is_finite() || self.relative_tolerance < 0.0 {
            return Err("relative_tolerance must be a finite non-negative number".into());
        }
        if self.samples.len() < 2 {
            return Err("a semantic assertion needs at least two samples".into());
        }
        let metric = &self.samples[0].metric;
        if self.samples.iter().any(|sample| sample.metric != *metric) {
            return Err("all samples in a semantic assertion must use the same metric".into());
        }
        if self.kind == Kind::Conserved
            && (!is_additive(metric) || self.samples.iter().any(|sample| sample.vessel.is_some()))
        {
            return Err(
                "a conserved assertion must use whole-bench mass_g or moles:SPECIES samples".into(),
            );
        }
        if self.kind == Kind::Unchanged {
            let vessel = &self.samples[0].vessel;
            if self.samples.iter().any(|sample| sample.vessel != *vessel) {
                return Err("an unchanged assertion must keep the same vessel".into());
            }
        }
        match (self.kind, self.ratio) {
            (Kind::Ratio, Some(ratio)) if ratio.is_finite() => {
                if self.samples.len() != 2 {
                    return Err("a ratio assertion needs exactly two samples".into());
                }
            }
            (Kind::Ratio, Some(_)) => return Err("ratio must be finite".into()),
            (Kind::Ratio, None) => return Err("a ratio assertion requires ratio".into()),
            (_, Some(_)) => return Err("ratio is only valid for a ratio assertion".into()),
            (_, None) => {}
        }
        let values = self
            .samples
            .iter()
            .map(|s| sample(s, trace))
            .collect::<Result<Vec<_>, _>>()?;
        let tol = self.tolerance;
        let close =
            |a: f64, b: f64| (a - b).abs() <= tol + self.relative_tolerance * a.abs().max(b.abs());
        let ok = match self.kind {
            Kind::Equal | Kind::Conserved | Kind::Unchanged => {
                values[1..].iter().all(|v| close(*v, values[0]))
            }
            Kind::Increasing => values.windows(2).all(|w| w[1] > w[0] + tol),
            Kind::Decreasing => values.windows(2).all(|w| w[0] > w[1] + tol),
            Kind::Ratio => close(values[1], self.ratio.expect("validated") * values[0]),
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
    let index = step_index(&spec.step, trace)?;
    let state = &trace.states[index];
    if let Some(selector) = spec.metric.strip_prefix("event:") {
        if spec.vessel.is_some() {
            return Err("an event scalar sample must not name a vessel".into());
        }
        if !spec.step.starts_with("after:") {
            return Err("an event scalar sample must use an exact after:N step".into());
        }
        let (kind, field) = selector
            .split_once('.')
            .filter(|(kind, field)| !kind.is_empty() && !field.is_empty() && !field.contains('.'))
            .ok_or_else(|| format!("invalid event scalar metric '{}'", spec.metric))?;
        let matches = state
            .events
            .iter()
            .filter(|event| event.kind == kind)
            .collect::<Vec<_>>();
        return match matches.as_slice() {
            [] => Err(format!("event '{kind}' is absent at step '{}'", spec.step)),
            [event] => event.fields.get(field).copied().ok_or_else(|| {
                format!(
                    "field '{field}' is absent from event '{kind}' at step '{}'",
                    spec.step
                )
            }),
            _ => Err(format!(
                "event '{kind}' is ambiguous at step '{}'",
                spec.step
            )),
        };
    }
    if spec.metric.starts_with("event") {
        return Err(format!("invalid event scalar metric '{}'", spec.metric));
    }
    vessel_sample(spec, state)
}

fn step_index(step: &str, trace: &Trace) -> Result<usize, String> {
    let index = match step {
        "initial" => 0,
        "final" => trace
            .states
            .len()
            .checked_sub(1)
            .ok_or("replay trace is empty")?,
        s if s.starts_with("after:") => {
            let index = s[6..]
                .parse::<usize>()
                .map_err(|_| format!("invalid step '{s}'"))?;
            if index == 0 {
                return Err("invalid step 'after:0' (operation counts are one-based)".into());
            }
            index
        }
        s => {
            return Err(format!(
                "invalid step '{s}' (use initial, final, or after:N)"
            ))
        }
    };
    trace
        .states
        .get(index)
        .map(|_| index)
        .ok_or_else(|| format!("step '{step}' is outside this replay"))
}

fn vessel_sample(spec: &Sample, state: &State) -> Result<f64, String> {
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
        if let Some(species) = spec.metric.strip_prefix("moles:") {
            if !state
                .vessels
                .values()
                .any(|v| v.moles.contains_key(species))
            {
                return Err(format!("species '{species}' is absent from the bench"));
            }
            return Ok(state
                .vessels
                .values()
                .map(|v| v.moles.get(species).copied().unwrap_or(0.0))
                .sum());
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
        m if m.starts_with("moles:") => v
            .moles
            .get(&m[6..])
            .copied()
            .ok_or_else(|| format!("species '{}' is absent from the vessel", &m[6..])),
        _ => Err(format!("unknown metric '{metric}'")),
    }
}

fn is_additive(metric: &str) -> bool {
    metric == "mass_g" || metric.starts_with("moles:")
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
                    events: Vec::new(),
                })
                .collect(),
        }
    }
    fn claim(kind: Kind) -> Assertion {
        Assertion {
            kind,
            tolerance: 1e-6,
            relative_tolerance: 0.0,
            ratio: None,
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
        c.samples[0].metric = "temperature_c".into();
        c.samples[1].metric = "temperature_c".into();
        c.samples[1].vessel = None;
        assert!(c
            .evaluate(&trace(&[1.0]))
            .unwrap_err()
            .contains("needs a vessel"));
        c.samples[0].metric = "mass_g".into();
        c.samples[0].step = "after:0".into();
        c.samples[1].metric = "mass_g".into();
        assert!(c
            .evaluate(&trace(&[1.0]))
            .unwrap_err()
            .contains("one-based"));
    }
    #[test]
    fn refuses_vacuous_or_structurally_invalid_relationships() {
        let mut c = claim(Kind::Equal);
        for sample in &mut c.samples {
            sample.metric = "moles:TYPO".into();
        }
        assert!(c
            .evaluate(&trace(&[1.0, 1.0]))
            .unwrap_err()
            .contains("absent"));
        for sample in &mut c.samples {
            sample.vessel = None;
        }
        assert!(c
            .evaluate(&trace(&[1.0, 1.0]))
            .unwrap_err()
            .contains("absent"));

        let mut mixed = claim(Kind::Equal);
        mixed.samples[1].metric = "temperature_c".into();
        assert!(mixed
            .evaluate(&trace(&[1.0, 1.0]))
            .unwrap_err()
            .contains("same metric"));

        let conserved = claim(Kind::Conserved);
        assert!(conserved
            .evaluate(&trace(&[1.0, 1.0]))
            .unwrap_err()
            .contains("whole-bench"));

        let mut unchanged = claim(Kind::Unchanged);
        unchanged.samples[1].vessel = Some("v2".into());
        assert!(unchanged
            .evaluate(&trace(&[1.0, 1.0]))
            .unwrap_err()
            .contains("same vessel"));
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
        assert_eq!(parsed.assertions[0].relative_tolerance, 0.0);
        assert_eq!(parsed.assertions[0].ratio, None);
        assert_eq!(parsed.assertions[0].kind, Kind::Unchanged);
        let exported = toml::to_string(&parsed.assertions[0]).unwrap();
        assert!(!exported.contains("relative_tolerance"));
        assert!(!exported.contains("ratio ="));
    }

    #[test]
    fn ratio_uses_second_over_first_with_absolute_and_relative_slack() {
        let mut ratio = claim(Kind::Ratio);
        ratio.ratio = Some(2.0);
        ratio.tolerance = 1e-12;
        assert!(ratio.evaluate(&trace(&[3.0, 6.0])).is_ok());
        assert!(ratio.evaluate(&trace(&[3.0, 6.1])).is_err());

        ratio.relative_tolerance = 0.02;
        assert!(ratio.evaluate(&trace(&[3.0, 6.1])).is_ok());
        // Relative tolerance cannot erase an absolute near-zero error.
        ratio.tolerance = 1e-5;
        assert!(ratio.evaluate(&trace(&[0.0, 9e-6])).is_ok());
        assert!(ratio.evaluate(&trace(&[0.0, 2e-5])).is_err());

        let encoded = toml::to_string(&ratio).unwrap();
        assert!(encoded.contains("kind = \"ratio\""));
        assert!(encoded.contains("ratio = 2.0"));
        assert!(encoded.contains("relative_tolerance = 0.02"));
        let decoded: Assertion = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.kind, Kind::Ratio);
        assert_eq!(decoded.ratio, Some(2.0));
        assert_eq!(decoded.relative_tolerance, 0.02);
    }

    #[test]
    fn ratio_rejects_hostile_or_misplaced_parameters() {
        let mut ratio = claim(Kind::Ratio);
        assert!(ratio
            .evaluate(&trace(&[1.0, 2.0]))
            .unwrap_err()
            .contains("requires ratio"));
        ratio.ratio = Some(f64::NAN);
        assert!(ratio
            .evaluate(&trace(&[1.0, 2.0]))
            .unwrap_err()
            .contains("finite"));
        ratio.ratio = Some(2.0);
        ratio.relative_tolerance = -1.0;
        assert!(ratio
            .evaluate(&trace(&[1.0, 2.0]))
            .unwrap_err()
            .contains("non-negative"));
        ratio.relative_tolerance = 0.0;
        ratio.samples.push(ratio.samples[0].clone());
        assert!(ratio
            .evaluate(&trace(&[1.0, 2.0]))
            .unwrap_err()
            .contains("exactly two"));

        let mut equal = claim(Kind::Equal);
        equal.ratio = Some(1.0);
        assert!(equal
            .evaluate(&trace(&[1.0, 1.0]))
            .unwrap_err()
            .contains("only valid"));
    }

    fn event_claim(step: &str, metric: &str) -> Assertion {
        Assertion {
            kind: Kind::Decreasing,
            tolerance: 0.0001,
            relative_tolerance: 0.0,
            ratio: None,
            samples: vec![
                Sample {
                    step: "after:1".into(),
                    metric: metric.into(),
                    vessel: None,
                },
                Sample {
                    step: step.into(),
                    metric: metric.into(),
                    vessel: None,
                },
            ],
        }
    }

    fn voltage_trace(second_events: Vec<ScalarEvent>) -> Trace {
        let voltage = |value| ScalarEvent {
            kind: "cell_voltage".into(),
            fields: BTreeMap::from([("volts".into(), value)]),
        };
        Trace {
            states: vec![
                State::default(),
                State {
                    events: vec![voltage(1.12)],
                    ..Default::default()
                },
                State {
                    events: second_events,
                    ..Default::default()
                },
            ],
        }
    }

    #[test]
    fn event_scalars_are_selected_at_exact_operation_boundaries() {
        let trace = voltage_trace(vec![ScalarEvent {
            kind: "cell_voltage".into(),
            fields: BTreeMap::from([("volts".into(), 1.10)]),
        }]);
        assert!(event_claim("after:2", "event:cell_voltage.volts")
            .evaluate(&trace)
            .is_ok());
        assert!(event_claim("final", "event:cell_voltage.volts")
            .evaluate(&trace)
            .unwrap_err()
            .contains("exact after:N"));
        let mut with_vessel = event_claim("after:2", "event:cell_voltage.volts");
        with_vessel.samples[0].vessel = Some("v1".into());
        assert!(with_vessel
            .evaluate(&trace)
            .unwrap_err()
            .contains("must not name"));
    }

    #[test]
    fn event_scalars_refuse_typos_absence_and_ambiguity() {
        let one = ScalarEvent {
            kind: "cell_voltage".into(),
            fields: BTreeMap::from([("volts".into(), 1.10)]),
        };
        let trace = voltage_trace(vec![one.clone()]);
        assert!(event_claim("after:2", "event:cell_voltage.typo")
            .evaluate(&trace)
            .unwrap_err()
            .contains("field 'typo' is absent"));
        assert!(event_claim("after:2", "event:no_such.volts")
            .evaluate(&trace)
            .unwrap_err()
            .contains("is absent"));
        assert!(event_claim("after:2", "event:cell_voltage")
            .evaluate(&trace)
            .unwrap_err()
            .contains("invalid event scalar"));
        let ambiguous = voltage_trace(vec![one.clone(), one]);
        assert!(event_claim("after:2", "event:cell_voltage.volts")
            .evaluate(&ambiguous)
            .unwrap_err()
            .contains("ambiguous"));
    }
}

//! DATA-005: Property-resolution ladder.
//!
//! Given a species, property, and conditions, return the best available value
//! together with its quality rung, uncertainty, validity bounds, and full
//! provenance. Return `Unavailable` rather than a naked default.

use crate::{
    Applicability, Interval, Method, Phase, PhaseProperty, PhaseThermodynamicRecord,
    RegistryDocument, Uncertainty,
};

/// Quality rung in the property-resolution ladder, from most to least
/// trustworthy. Callers can decide their own policy ("only show measured
/// values to students" vs "use anything available for a first estimate").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rung {
    /// Direct laboratory measurement.
    Measured = 0,
    /// Computed from first principles or calibrated model.
    Calculated = 1,
    /// Derived from another measured/calculated value (e.g. Hess' law).
    Derived = 2,
    /// Imported from an external database without independent verification.
    Imported = 3,
    /// Editorial estimate or textbook value without stated provenance.
    Editorial = 4,
}

impl Rung {
    fn from_method(method: &Method) -> Self {
        match method {
            Method::Measured(_) => Self::Measured,
            Method::Calculated(_) => Self::Calculated,
            Method::Derived(_) => Self::Derived,
            Method::Imported(_) => Self::Imported,
            Method::Editorial(_) => Self::Editorial,
        }
    }
}

/// A resolved property value with full provenance.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedValue {
    pub value: f64,
    pub unit_symbol: String,
    pub rung: Rung,
    pub uncertainty: Uncertainty,
    pub source_id: String,
    pub method_detail: String,
    pub conditions: Applicability,
}

/// The result of a property resolution: either a value with provenance, or
/// an explicit `Unavailable` with the reason.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Resolution {
    Resolved(Box<ResolvedValue>),
    Unavailable { reason: String },
}

impl Resolution {
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Resolved(v) => Some(v.value),
            Self::Unavailable { .. } => None,
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }
}

/// Condition point for resolution. `None` fields match any condition.
#[derive(Debug, Clone, Default)]
pub struct Conditions {
    pub temperature_k: Option<f64>,
    pub pressure_pa: Option<f64>,
    pub phase: Option<Phase>,
}

/// Check whether a record's applicability covers the requested conditions.
fn conditions_match(applicability: &Applicability, request: &Conditions) -> bool {
    if let (Some(req_phase), Some(app_phase)) = (&request.phase, &applicability.phase) {
        if req_phase != app_phase {
            return false;
        }
    }
    if let (Some(req_t), Some(interval)) = (request.temperature_k, &applicability.temperature) {
        if !interval_contains(interval, req_t) {
            return false;
        }
    }
    if let (Some(req_p), Some(interval)) = (request.pressure_pa, &applicability.pressure) {
        if !interval_contains(interval, req_p) {
            return false;
        }
    }
    true
}

fn interval_contains(interval: &Interval, value: f64) -> bool {
    value >= interval.lower && value <= interval.upper
}

/// Resolve a phase-thermodynamic property for a species.
///
/// Returns the best-quality record that matches the species, property, phase,
/// and conditions. "Best" means the lowest rung number (measured beats
/// calculated beats derived, etc.). Among records on the same rung, the one
/// whose conditions most specifically match the request is preferred.
pub fn resolve_phase_property(
    doc: &RegistryDocument,
    species_id: &str,
    property: &PhaseProperty,
    conditions: &Conditions,
) -> Resolution {
    let candidates: Vec<&PhaseThermodynamicRecord> = doc
        .phase_thermodynamics
        .iter()
        .filter(|r| r.species_id == species_id && &r.property == property)
        .filter(|r| {
            if let Some(phase) = &conditions.phase {
                r.phase == *phase
            } else {
                true
            }
        })
        .filter(|r| conditions_match(&r.quantity.conditions, conditions))
        .collect();

    if candidates.is_empty() {
        return Resolution::Unavailable {
            reason: format!(
                "no {property:?} record for species {species_id:?} under requested conditions"
            ),
        };
    }

    // Pick the best rung. Among same-rung, prefer the one with tightest
    // conditions (smallest temperature interval, if any).
    let best = candidates
        .into_iter()
        .min_by_key(|r| {
            let rung = Rung::from_method(&r.quantity.method) as u32;
            let specificity = r
                .quantity
                .conditions
                .temperature
                .as_ref()
                .map(|i| ((i.upper - i.lower) * 1e6) as u64)
                .unwrap_or(u64::MAX);
            (rung, specificity)
        })
        .unwrap();

    Resolution::Resolved(Box::new(ResolvedValue {
        value: best.quantity.value,
        unit_symbol: best.quantity.unit.symbol.clone(),
        rung: Rung::from_method(&best.quantity.method),
        uncertainty: best.quantity.uncertainty.clone(),
        source_id: best.quantity.source_id.clone(),
        method_detail: best.quantity.method.detail().to_string(),
        conditions: best.quantity.conditions.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn sample_doc() -> RegistryDocument {
        let mut doc = RegistryDocument::empty();
        doc.sources.push(SourceRecord {
            id: "test-src".into(),
            citation: "test".into(),
            licence: "MIT".into(),
            lane: SourceLane::Runtime,
            origin: None,
            revision: None,
            retrieved: None,
        });
        doc.phase_thermodynamics.push(PhaseThermodynamicRecord {
            id: "mm-nacl".into(),
            species_id: "sodium-chloride".into(),
            phase: Phase::Solid,
            property: PhaseProperty::MolarMass,
            quantity: NumericRecord {
                value: 58.44,
                unit: Unit {
                    symbol: "g/mol".into(),
                    dimension: Dimension::MolarMass,
                },
                conditions: Applicability::default(),
                uncertainty: Uncertainty::Exact,
                source_id: "test-src".into(),
                method: Method::Measured("IUPAC atomic weights".into()),
            },
        });
        doc.phase_thermodynamics.push(PhaseThermodynamicRecord {
            id: "cp-nacl-editorial".into(),
            species_id: "sodium-chloride".into(),
            phase: Phase::Solid,
            property: PhaseProperty::MolarHeatCapacity,
            quantity: NumericRecord {
                value: 50.5,
                unit: Unit {
                    symbol: "J/(mol·K)".into(),
                    dimension: Dimension::MolarHeatCapacity,
                },
                conditions: Applicability::default(),
                uncertainty: Uncertainty::NotReported,
                source_id: "test-src".into(),
                method: Method::Editorial("textbook value".into()),
            },
        });
        doc
    }

    #[test]
    fn resolves_measured_molar_mass() {
        let doc = sample_doc();
        let res = resolve_phase_property(
            &doc,
            "sodium-chloride",
            &PhaseProperty::MolarMass,
            &Conditions::default(),
        );
        match res {
            Resolution::Resolved(v) => {
                assert!((v.value - 58.44).abs() < 1e-10);
                assert_eq!(v.rung, Rung::Measured);
            }
            _ => panic!("expected resolved"),
        }
    }

    #[test]
    fn returns_unavailable_for_missing_property() {
        let doc = sample_doc();
        let res = resolve_phase_property(
            &doc,
            "sodium-chloride",
            &PhaseProperty::BoilingTemperature,
            &Conditions::default(),
        );
        assert!(!res.is_available());
    }

    #[test]
    fn returns_unavailable_for_unknown_species() {
        let doc = sample_doc();
        let res = resolve_phase_property(
            &doc,
            "unobtainium",
            &PhaseProperty::MolarMass,
            &Conditions::default(),
        );
        assert!(!res.is_available());
    }

    #[test]
    fn editorial_rung_is_correct() {
        let doc = sample_doc();
        let res = resolve_phase_property(
            &doc,
            "sodium-chloride",
            &PhaseProperty::MolarHeatCapacity,
            &Conditions::default(),
        );
        match res {
            Resolution::Resolved(v) => assert_eq!(v.rung, Rung::Editorial),
            _ => panic!("expected resolved"),
        }
    }
}

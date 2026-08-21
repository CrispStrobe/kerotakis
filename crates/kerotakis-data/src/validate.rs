use std::collections::HashSet;

use crate::{
    Applicability, CompositionRecord, Dimension, Interval, ModelSubject, NumericRecord,
    OpticalRecord, RegistryDocument, Uncertainty, REGISTRY_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} registry validation issue(s)", self.issues.len())?;
        for issue in &self.issues {
            write!(f, "\n{}: {}", issue.path, issue.detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

impl RegistryDocument {
    /// Validate cross-record references and the provenance envelope around
    /// every number. Returns all discovered issues in deterministic order.
    pub fn validate(&self) -> Result<(), ValidationError> {
        let mut validator = Validator::new(self);
        validator.run();
        if validator.issues.is_empty() {
            Ok(())
        } else {
            Err(ValidationError {
                issues: validator.issues,
            })
        }
    }
}

struct Validator<'a> {
    document: &'a RegistryDocument,
    sources: HashSet<String>,
    species: HashSet<String>,
    issues: Vec<ValidationIssue>,
}

impl<'a> Validator<'a> {
    fn new(document: &'a RegistryDocument) -> Self {
        Self {
            document,
            sources: HashSet::new(),
            species: HashSet::new(),
            issues: Vec::new(),
        }
    }

    fn run(&mut self) {
        if self.document.schema != REGISTRY_SCHEMA_VERSION {
            self.issue(
                "schema",
                format!(
                    "unsupported version {}; expected {REGISTRY_SCHEMA_VERSION}",
                    self.document.schema
                ),
            );
        }

        self.validate_sources();
        self.validate_identities();
        self.validate_compositions();
        self.validate_phase_thermodynamics();
        self.validate_transport();
        self.validate_optical();
        self.validate_safety();
        self.validate_microstates();
        self.validate_model_parameters();
    }

    fn validate_sources(&mut self) {
        let mut ids = HashSet::new();
        for (index, source) in self.document.sources.clone().iter().enumerate() {
            let path = format!("sources[{index}]");
            self.nonempty(&format!("{path}.id"), &source.id);
            self.nonempty(&format!("{path}.citation"), &source.citation);
            self.nonempty(&format!("{path}.licence"), &source.licence);
            if !source.id.trim().is_empty() && !ids.insert(source.id.clone()) {
                self.issue(format!("{path}.id"), "duplicate source id");
            }
        }
        self.sources = ids;
    }

    fn validate_identities(&mut self) {
        let mut ids = HashSet::new();
        let mut keys = HashSet::new();
        for (index, identity) in self.document.identities.clone().iter().enumerate() {
            let path = format!("identities[{index}]");
            self.nonempty(&format!("{path}.id"), &identity.id);
            self.nonempty(&format!("{path}.canonical_key"), &identity.canonical_key);
            self.nonempty(&format!("{path}.name"), &identity.name);
            if !identity.id.trim().is_empty() && !ids.insert(identity.id.clone()) {
                self.issue(format!("{path}.id"), "duplicate identity id");
            }
            if !identity.canonical_key.trim().is_empty()
                && !keys.insert(identity.canonical_key.clone())
            {
                self.issue(format!("{path}.canonical_key"), "duplicate canonical key");
            }
            for (kind, value) in &identity.identifiers {
                self.nonempty(&format!("{path}.identifiers key"), kind);
                self.nonempty(&format!("{path}.identifiers.{kind}"), value);
            }
            self.evidence(
                &format!("{path}.evidence"),
                &identity.evidence.source_id,
                &identity.evidence.method,
            );
        }
        self.species = ids;
    }

    fn validate_compositions(&mut self) {
        let mut ids = HashSet::new();
        let mut species = HashSet::new();
        for (index, record) in self.document.compositions.clone().iter().enumerate() {
            let path = format!("compositions[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.species_ref(&format!("{path}.species_id"), &record.species_id);
            if !species.insert(record.species_id.clone()) {
                self.issue(
                    format!("{path}.species_id"),
                    "a species may have only one composition record",
                );
            }
            self.nonempty(&format!("{path}.formula"), &record.formula);
            self.evidence(
                &format!("{path}.evidence"),
                &record.evidence.source_id,
                &record.evidence.method,
            );
            if record.elements.is_empty() {
                self.issue(format!("{path}.elements"), "composition has no elements");
            }
            self.numeric(
                &format!("{path}.net_charge"),
                &record.net_charge,
                Some(Dimension::Dimensionless),
            );
            self.validate_elements(&path, record);
        }
    }

    fn validate_elements(&mut self, path: &str, record: &CompositionRecord) {
        let mut elements = HashSet::new();
        for (index, element) in record.elements.iter().enumerate() {
            let element_path = format!("{path}.elements[{index}]");
            self.nonempty(&format!("{element_path}.element"), &element.element);
            if !elements.insert(element.element.clone()) {
                self.issue(format!("{element_path}.element"), "duplicate element");
            }
            self.numeric(
                &format!("{element_path}.count"),
                &element.count,
                Some(Dimension::Dimensionless),
            );
            if element.count.value < 0.0 {
                self.issue(
                    format!("{element_path}.count.value"),
                    "must be non-negative",
                );
            }
        }
    }

    fn validate_phase_thermodynamics(&mut self) {
        let mut ids = HashSet::new();
        for (index, record) in self
            .document
            .phase_thermodynamics
            .clone()
            .iter()
            .enumerate()
        {
            let path = format!("phase_thermodynamics[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.species_ref(&format!("{path}.species_id"), &record.species_id);
            self.numeric(
                &format!("{path}.quantity"),
                &record.quantity,
                record.property.expected_dimension(),
            );
        }
    }

    fn validate_transport(&mut self) {
        let mut ids = HashSet::new();
        for (index, record) in self.document.transport.clone().iter().enumerate() {
            let path = format!("transport[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.species_ref(&format!("{path}.species_id"), &record.species_id);
            self.numeric(
                &format!("{path}.quantity"),
                &record.quantity,
                record.property.expected_dimension(),
            );
        }
    }

    fn validate_optical(&mut self) {
        let mut ids = HashSet::new();
        for (index, record) in self.document.optical.clone().iter().enumerate() {
            let path = format!("optical[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.species_ref(&format!("{path}.species_id"), &record.species_id);
            self.optical_record(&path, record);
            self.evidence(
                &format!("{path}.evidence"),
                &record.evidence.source_id,
                &record.evidence.method,
            );
        }
    }

    fn optical_record(&mut self, path: &str, record: &OpticalRecord) {
        if record.appearance.is_none()
            && record.reflective_srgb.is_none()
            && record.spectrum.is_empty()
        {
            self.issue(path, "optical record carries no observation or spectrum");
        }
        if let Some(rgb) = &record.reflective_srgb {
            let valid = rgb.len() == 7
                && rgb.starts_with('#')
                && rgb[1..]
                    .chars()
                    .all(|character| character.is_ascii_hexdigit());
            if !valid {
                self.issue(format!("{path}.reflective_srgb"), "expected #RRGGBB");
            }
        }
        for (index, sample) in record.spectrum.iter().enumerate() {
            let sample_path = format!("{path}.spectrum[{index}]");
            self.numeric(
                &format!("{sample_path}.wavelength"),
                &sample.wavelength,
                Some(Dimension::Wavelength),
            );
            self.numeric(
                &format!("{sample_path}.molar_absorptivity"),
                &sample.molar_absorptivity,
                Some(Dimension::MolarAbsorptivity),
            );
        }
    }

    fn validate_safety(&mut self) {
        let mut ids = HashSet::new();
        for (index, record) in self.document.safety.clone().iter().enumerate() {
            let path = format!("safety[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.species_ref(&format!("{path}.species_id"), &record.species_id);
            if record.classifications.is_empty()
                && record.statements.is_empty()
                && record.limits.is_empty()
            {
                self.issue(&path, "safety record carries no classification or limit");
            }
            self.evidence(
                &format!("{path}.evidence"),
                &record.evidence.source_id,
                &record.evidence.method,
            );
            for (limit_index, limit) in record.limits.iter().enumerate() {
                self.numeric(
                    &format!("{path}.limits[{limit_index}].quantity"),
                    &limit.quantity,
                    limit.kind.expected_dimension(),
                );
            }
        }
    }

    fn validate_microstates(&mut self) {
        let mut ids = HashSet::new();
        for (index, record) in self.document.microstates.clone().iter().enumerate() {
            let path = format!("microstates[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.species_ref(&format!("{path}.species_id"), &record.species_id);
            self.nonempty(&format!("{path}.label"), &record.label);
            self.evidence(
                &format!("{path}.evidence"),
                &record.evidence.source_id,
                &record.evidence.method,
            );
            self.numeric(
                &format!("{path}.formal_charge"),
                &record.formal_charge,
                Some(Dimension::Dimensionless),
            );
            if let Some(energy) = &record.relative_energy {
                self.numeric(
                    &format!("{path}.relative_energy"),
                    energy,
                    Some(Dimension::MolarEnergy),
                );
            }
            if let Some(fraction) = &record.equilibrium_fraction {
                self.numeric(
                    &format!("{path}.equilibrium_fraction"),
                    fraction,
                    Some(Dimension::Dimensionless),
                );
                if !(0.0..=1.0).contains(&fraction.value) {
                    self.issue(
                        format!("{path}.equilibrium_fraction.value"),
                        "must be between zero and one",
                    );
                }
            }
        }
    }

    fn validate_model_parameters(&mut self) {
        let mut ids = HashSet::new();
        for (index, record) in self.document.model_parameters.clone().iter().enumerate() {
            let path = format!("model_parameters[{index}]");
            self.record_id(&path, &record.id, &mut ids);
            self.nonempty(&format!("{path}.model"), &record.model);
            self.nonempty(&format!("{path}.parameter"), &record.parameter);
            if let ModelSubject::Species(species) = &record.subject {
                self.species_ref(&format!("{path}.subject"), species);
            }
            self.numeric(&format!("{path}.quantity"), &record.quantity, None);
        }
    }

    fn record_id(&mut self, path: &str, id: &str, ids: &mut HashSet<String>) {
        self.nonempty(&format!("{path}.id"), id);
        if !id.trim().is_empty() && !ids.insert(id.to_string()) {
            self.issue(format!("{path}.id"), "duplicate record id in this family");
        }
    }

    fn species_ref(&mut self, path: &str, species: &str) {
        self.nonempty(path, species);
        if !species.trim().is_empty() && !self.species.contains(species) {
            self.issue(path, format!("unknown species id '{species}'"));
        }
    }

    fn numeric(&mut self, path: &str, record: &NumericRecord, expected: Option<Dimension>) {
        if !record.value.is_finite() {
            self.issue(format!("{path}.value"), "must be finite");
        }
        self.nonempty(&format!("{path}.unit.symbol"), &record.unit.symbol);
        if let Some(expected) = expected {
            if record.unit.dimension != expected {
                self.issue(
                    format!("{path}.unit.dimension"),
                    format!("is {:?}; expected {:?}", record.unit.dimension, expected),
                );
            }
        }
        self.applicability(&format!("{path}.conditions"), &record.conditions);
        self.uncertainty(path, record);
        self.evidence(path, &record.source_id, &record.method);
    }

    fn applicability(&mut self, path: &str, applicability: &Applicability) {
        if let Some(interval) = &applicability.temperature {
            self.interval(
                &format!("{path}.temperature"),
                interval,
                Dimension::Temperature,
            );
        }
        if let Some(interval) = &applicability.pressure {
            self.interval(&format!("{path}.pressure"), interval, Dimension::Pressure);
        }
        if let Some(interval) = &applicability.ph {
            self.interval(&format!("{path}.ph"), interval, Dimension::Dimensionless);
        }
        if let Some(interval) = &applicability.ionic_strength {
            self.interval(
                &format!("{path}.ionic_strength"),
                interval,
                Dimension::Concentration,
            );
        }
    }

    fn interval(&mut self, path: &str, interval: &Interval, expected: Dimension) {
        if !interval.lower.is_finite() || !interval.upper.is_finite() {
            self.issue(path, "bounds must be finite");
        }
        if interval.lower > interval.upper {
            self.issue(path, "lower bound exceeds upper bound");
        }
        self.nonempty(&format!("{path}.unit.symbol"), &interval.unit.symbol);
        if interval.unit.dimension != expected {
            self.issue(
                format!("{path}.unit.dimension"),
                format!("is {:?}; expected {:?}", interval.unit.dimension, expected),
            );
        }
    }

    fn uncertainty(&mut self, path: &str, record: &NumericRecord) {
        match &record.uncertainty {
            Uncertainty::Exact | Uncertainty::NotReported => {}
            Uncertainty::Absolute { plus_minus } => {
                if !plus_minus.is_finite() || *plus_minus < 0.0 {
                    self.issue(
                        format!("{path}.uncertainty.plus_minus"),
                        "must be finite and non-negative",
                    );
                }
            }
            Uncertainty::Relative { fraction } => {
                if !fraction.is_finite() || *fraction < 0.0 {
                    self.issue(
                        format!("{path}.uncertainty.fraction"),
                        "must be finite and non-negative",
                    );
                }
            }
            Uncertainty::Interval { lower, upper } => {
                if !lower.is_finite() || !upper.is_finite() || lower > upper {
                    self.issue(
                        format!("{path}.uncertainty"),
                        "interval bounds must be finite and ordered",
                    );
                } else if record.value < *lower || record.value > *upper {
                    self.issue(
                        format!("{path}.uncertainty"),
                        "interval does not contain the value",
                    );
                }
            }
        }
    }

    fn nonempty(&mut self, path: &str, value: &str) {
        if value.trim().is_empty() {
            self.issue(path, "must not be empty");
        }
    }

    fn evidence(&mut self, path: &str, source_id: &str, method: &crate::Method) {
        self.nonempty(&format!("{path}.source_id"), source_id);
        if !source_id.trim().is_empty() && !self.sources.contains(source_id) {
            self.issue(
                format!("{path}.source_id"),
                format!("unknown source id '{source_id}'"),
            );
        }
        if method.detail().trim().is_empty() {
            self.issue(format!("{path}.method"), "method detail is empty");
        }
    }

    fn issue(&mut self, path: impl Into<String>, detail: impl Into<String>) {
        self.issues.push(ValidationIssue {
            path: path.into(),
            detail: detail.into(),
        });
    }
}

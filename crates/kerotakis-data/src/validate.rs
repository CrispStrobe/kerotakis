use std::collections::HashSet;

use crate::{
    Applicability, CompositionRecord, Dimension, FractionRange, Interval, MaterialExpansionPolicy,
    MaterialPhysicalForm, ModelSubject, NumericRecord, OpticalRecord, RegistryDocument,
    Uncertainty, REGISTRY_SCHEMA_VERSION,
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
        self.validate_material_recipes();
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
            && record.flame_colour.is_none()
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

    fn validate_material_recipes(&mut self) {
        let mut ids = HashSet::new();
        let mut keys = HashSet::new();
        let mut material_names = HashSet::new();
        let species_names = self
            .document
            .identities
            .iter()
            .flat_map(|identity| {
                std::iter::once(identity.id.as_str())
                    .chain(std::iter::once(identity.name.as_str()))
                    .chain(identity.synonyms.iter().map(String::as_str))
            })
            .map(normalize_name)
            .collect::<HashSet<_>>();

        for (index, recipe) in self.document.material_recipes.clone().iter().enumerate() {
            let path = format!("material_recipes[{index}]");
            self.record_id(&path, &recipe.id, &mut ids);
            self.nonempty(&format!("{path}.canonical_key"), &recipe.canonical_key);
            self.nonempty(&format!("{path}.name"), &recipe.name);
            if recipe.version == 0 {
                self.issue(format!("{path}.version"), "must be at least one");
            }
            if !recipe.canonical_key.trim().is_empty()
                && !keys.insert(normalize_name(&recipe.canonical_key))
            {
                self.issue(format!("{path}.canonical_key"), "duplicate material key");
            }
            for (name_path, name) in [
                (format!("{path}.canonical_key"), &recipe.canonical_key),
                (format!("{path}.name"), &recipe.name),
            ] {
                if !name.trim().is_empty() && !material_names.insert(normalize_name(name)) {
                    self.issue(name_path.clone(), "duplicate material name or alias");
                }
                if species_names.contains(&normalize_name(name)) {
                    self.issue(
                        name_path,
                        format!("material name '{name}' overrides a canonical species"),
                    );
                }
            }

            for (language, aliases) in &recipe.aliases {
                self.nonempty(&format!("{path}.aliases language"), language);
                if aliases.is_empty() {
                    self.issue(
                        format!("{path}.aliases.{language}"),
                        "alias list must not be empty",
                    );
                }
                for alias in aliases {
                    self.nonempty(&format!("{path}.aliases.{language}"), alias);
                    if !alias.trim().is_empty() && !material_names.insert(normalize_name(alias)) {
                        self.issue(
                            format!("{path}.aliases.{language}"),
                            format!("duplicate material name or alias '{alias}'"),
                        );
                    }
                    if species_names.contains(&normalize_name(alias)) {
                        self.issue(
                            format!("{path}.aliases.{language}"),
                            format!("material alias '{alias}' overrides a canonical species"),
                        );
                    }
                }
            }

            if recipe.components.is_empty() {
                self.issue(
                    format!("{path}.components"),
                    "recipe has no resolved components",
                );
            }
            if let Some(density) = &recipe.bulk_density {
                self.numeric(
                    &format!("{path}.bulk_density"),
                    density,
                    Some(Dimension::MassDensity),
                );
                if density.value <= 0.0 {
                    self.issue(format!("{path}.bulk_density.value"), "must be positive");
                }
            }
            let mut component_species = HashSet::new();
            let mut lower_sum = 0.0;
            let mut upper_sum = 0.0;
            for (component_index, component) in recipe.components.iter().enumerate() {
                let component_path = format!("{path}.components[{component_index}]");
                self.species_ref(
                    &format!("{component_path}.species_id"),
                    &component.species_id,
                );
                if !component_species.insert(component.species_id.clone()) {
                    self.issue(
                        format!("{component_path}.species_id"),
                        "duplicate component species",
                    );
                }
                self.fraction_range(&format!("{component_path}.fraction"), component.fraction);
                lower_sum += component.fraction.lower;
                upper_sum += component.fraction.upper;
                self.evidence(
                    &format!("{component_path}.evidence"),
                    &component.evidence.source_id,
                    &component.evidence.method,
                );
                if matches!(recipe.expansion_policy, MaterialExpansionPolicy::Fixed)
                    && (component.fraction.upper - component.fraction.lower).abs() > 1e-12
                {
                    self.issue(
                        format!("{component_path}.fraction"),
                        "fixed expansion requires an exact component fraction",
                    );
                }
            }
            if upper_sum > 1.0 + 1e-12 {
                self.issue(
                    format!("{path}.components"),
                    format!("upper component fractions sum to {upper_sum}; expected at most one"),
                );
            }
            if lower_sum > 1.0 + 1e-12 {
                self.issue(
                    format!("{path}.components"),
                    format!("lower component fractions sum to {lower_sum}; expected at most one"),
                );
            }
            let remainder = FractionRange {
                lower: (1.0 - upper_sum).max(0.0),
                upper: (1.0 - lower_sum).max(0.0),
            };
            match recipe.unresolved_fraction {
                Some(unresolved) => {
                    self.fraction_range(&format!("{path}.unresolved_fraction"), unresolved);
                    if unresolved.lower > remainder.lower + 1e-12
                        || unresolved.upper < remainder.upper - 1e-12
                    {
                        self.issue(
                            format!("{path}.unresolved_fraction"),
                            format!(
                                "must contain the conserved remainder {}..{}",
                                remainder.lower, remainder.upper
                            ),
                        );
                    }
                }
                None if remainder.upper > 1e-12 => self.issue(
                    format!("{path}.unresolved_fraction"),
                    format!("missing conserved remainder up to {}", remainder.upper),
                ),
                None => {}
            }

            if let MaterialPhysicalForm::CompositeObject {
                geometry: Some(geometry),
            } = &recipe.physical_form
            {
                if geometry
                    .surface_area_m2
                    .is_some_and(|value| !value.is_finite() || value <= 0.0)
                {
                    self.issue(
                        format!("{path}.physical_form.geometry.surface_area_m2"),
                        "must be finite and positive",
                    );
                }
                if geometry
                    .characteristic_length_m
                    .is_some_and(|value| !value.is_finite() || value <= 0.0)
                {
                    self.issue(
                        format!("{path}.physical_form.geometry.characteristic_length_m"),
                        "must be finite and positive",
                    );
                }
            }

            for (substitution_index, substitution) in recipe.substitutions.iter().enumerate() {
                let substitution_path = format!("{path}.substitutions[{substitution_index}]");
                if !component_species.contains(&substitution.component_species_id) {
                    self.issue(
                        format!("{substitution_path}.component_species_id"),
                        "substitution target is not a recipe component",
                    );
                }
                self.species_ref(
                    &format!("{substitution_path}.substitute_species_id"),
                    &substitution.substitute_species_id,
                );
                if !substitution.ratio.is_finite() || substitution.ratio <= 0.0 {
                    self.issue(
                        format!("{substitution_path}.ratio"),
                        "must be finite and positive",
                    );
                }
                self.evidence(
                    &format!("{substitution_path}.evidence"),
                    &substitution.evidence.source_id,
                    &substitution.evidence.method,
                );
            }

            if let MaterialExpansionPolicy::Seeded { salt } = &recipe.expansion_policy {
                self.nonempty(&format!("{path}.expansion_policy.salt"), salt);
            }
            if let MaterialPhysicalForm::Other { description } = &recipe.physical_form {
                self.nonempty(&format!("{path}.physical_form.description"), description);
            }
            for (assumption_index, assumption) in recipe.lot_assumptions.iter().enumerate() {
                self.nonempty(
                    &format!("{path}.lot_assumptions[{assumption_index}]"),
                    assumption,
                );
            }
            self.evidence(
                &format!("{path}.evidence"),
                &recipe.evidence.source_id,
                &recipe.evidence.method,
            );
        }
    }

    fn fraction_range(&mut self, path: &str, range: FractionRange) {
        if !range.lower.is_finite() || !range.upper.is_finite() {
            self.issue(path, "bounds must be finite");
        } else if range.lower < 0.0 || range.upper > 1.0 || range.lower > range.upper {
            self.issue(path, "expected ordered bounds between zero and one");
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

fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

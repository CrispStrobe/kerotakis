//! Prospective YAML admission compatibility; synthetic rates are not measurements.
use kerotakis_core::kinetics::mechanism::{parse_yaml, MechanismError};
const BASE: &str = "description: synthetic mechanism\nphases:\n- name: gas\n  thermo: ideal-gas\n  species: all\n  reactions: all\nspecies:\n- name: A\n  composition: {H: 2}\n- name: B\n  composition: {H: 2}\nreactions:\n- equation: A => B\n  rate-constant: {A: 1.0, b: 0.0, Ea: 0.0}\n";
fn equal(text: String) {
    assert_eq!(
        parse_yaml(&text).expect("supported form"),
        parse_yaml(BASE).expect("independent baseline form")
    );
}
fn replace(old: &str, new: &str) -> String {
    assert_eq!(BASE.matches(old).count(), 1);
    BASE.replace(old, new)
}
fn rejected(text: String, kind: &str, needle: &str) {
    let error = parse_yaml(&text).expect_err("unsupported form must refuse");
    let actual = match &error {
        MechanismError::Yaml(_) => "yaml",
        MechanismError::UnsupportedFieldValue { .. } => "value",
        MechanismError::UnsupportedSection { .. } => "section",
        MechanismError::UnsupportedField { .. } => "field",
        MechanismError::InvalidComposition { .. } => "composition",
        MechanismError::NonFiniteActivationEnergy { .. } => "activation",
        MechanismError::InvalidRate { .. } => "rate",
        MechanismError::UnsupportedUnit { .. } => "unit",
        MechanismError::UnsupportedPhase { .. } => "phase",
        other => panic!("unexpected refusal: {other:?}"),
    };
    assert_eq!(actual, kind, "{error}");
    if !needle.is_empty() {
        assert!(error.to_string().contains(needle), "{error}");
    }
}

#[test]
fn base_admission() {
    equal(BASE.to_string());
}
#[test]
fn document_markers() {
    equal(format!("---\n{BASE}...\n"));
}
#[test]
fn crlf() {
    equal(BASE.replace("\n", "\r\n"));
}
#[test]
fn comments() {
    equal(format!(
        "# authored compatibility probe\n{BASE}# trailing note\n"
    ));
}
#[test]
fn quoted_description() {
    equal(replace(
        "description: synthetic mechanism",
        "description: \"synthetic mechanism\"",
    ));
}
#[test]
fn single_quoted_description() {
    equal(replace(
        "description: synthetic mechanism",
        "description: 'synthetic mechanism'",
    ));
}
#[test]
fn species_explicit() {
    equal(replace("species: all", "species: [A, B]"));
}
#[test]
fn species_absent() {
    equal(replace("  species: all\n", ""));
}
#[test]
fn species_null() {
    equal(replace("species: all", "species: null"));
}
#[test]
fn reactions_absent() {
    equal(replace("  reactions: all\n", ""));
}
#[test]
fn reactions_null() {
    equal(replace("reactions: all", "reactions: null"));
}
#[test]
fn scientific_rate() {
    equal(replace("A: 1.0", "A: 1e0"));
}
#[test]
fn integer_rate() {
    equal(replace("A: 1.0", "A: 1"));
}
#[test]
fn integer_exponent() {
    equal(replace("b: 0.0", "b: 0"));
}
#[test]
fn integer_activation() {
    equal(replace("Ea: 0.0", "Ea: 0"));
}
#[test]
fn zero_activation_unit() {
    equal(replace("Ea: 0.0", "Ea: \"0 J/mol\""));
}
#[test]
fn flow_rate_block() {
    equal(replace(
        "  rate-constant: {A: 1.0, b: 0.0, Ea: 0.0}",
        "  rate-constant:\n    A: 1.0\n    b: 0.0\n    Ea: 0.0",
    ));
}
#[test]
fn zero_alias() {
    equal(replace("b: 0.0, Ea: 0.0", "b: &zero 0.0, Ea: *zero"));
}
#[test]
fn note_scalar() {
    equal(format!("{BASE}note: plain\n"));
}
#[test]
fn note_yes() {
    equal(format!("{BASE}note: yes\n"));
}
#[test]
fn note_no() {
    equal(format!("{BASE}note: no\n"));
}
#[test]
fn note_on() {
    equal(format!("{BASE}note: on\n"));
}
#[test]
fn note_off() {
    equal(format!("{BASE}note: off\n"));
}
#[test]
fn note_nested() {
    equal(format!(
        "{BASE}note: {{a: [1, null, true, {{nested: value}}]}}\n"
    ));
}
#[test]
fn note_nan() {
    equal(format!("{BASE}note: .nan\n"));
}
#[test]
fn note_infinity() {
    equal(format!("{BASE}note: .inf\n"));
}
#[test]
fn note_numeric_keys() {
    equal(format!("{BASE}note: {{1: value}}\n"));
}
#[test]
fn note_complex_keys() {
    equal(format!("{BASE}note: {{[a, b]: value}}\n"));
}
#[test]
fn species_transport() {
    equal(replace("  composition: {H: 2}\n- name: B", "  composition: {H: 2}\n  transport: {model: gas, diameter: 2.0, metadata: [one, two]}\n- name: B"));
}
#[test]
fn species_equation_of_state() {
    equal(replace("  composition: {H: 2}\n- name: B", "  composition: {H: 2}\n  equation-of-state: {model: Redlich-Kwong, a: [1.0, 2.0], b: 3.0}\n- name: B"));
}
#[test]
fn species_mapping() {
    rejected(
        replace("species: all", "species: {external.yaml: all}"),
        "value",
        "species",
    );
}
#[test]
fn species_mixed() {
    rejected(
        replace("species: all", "species: [A, 1]"),
        "value",
        "species",
    );
}
#[test]
fn species_boolean() {
    rejected(replace("species: all", "species: true"), "value", "species");
}
#[test]
fn reactions_none() {
    rejected(
        replace("reactions: all", "reactions: none"),
        "value",
        "reactions",
    );
}
#[test]
fn reactions_sequence() {
    rejected(
        replace("reactions: all", "reactions: [all]"),
        "value",
        "reactions",
    );
}
#[test]
fn reactions_yes() {
    rejected(
        replace("reactions: all", "reactions: yes"),
        "value",
        "reactions",
    );
}
#[test]
fn reactions_no() {
    rejected(
        replace("reactions: all", "reactions: no"),
        "value",
        "reactions",
    );
}
#[test]
fn reactions_on() {
    rejected(
        replace("reactions: all", "reactions: on"),
        "value",
        "reactions",
    );
}
#[test]
fn reactions_off() {
    rejected(
        replace("reactions: all", "reactions: off"),
        "value",
        "reactions",
    );
}
#[test]
fn unknown_section() {
    rejected(
        format!("{BASE}foreign-section: {{items: [1, 2]}}\n"),
        "section",
        "foreign-section",
    );
}
#[test]
fn unknown_species_field() {
    rejected(
        replace(
            "  composition: {H: 2}\n- name: B",
            "  composition: {H: 2}\n  foreign-species-field: {a: .nan}\n- name: B",
        ),
        "field",
        "foreign-species-field",
    );
}
#[test]
fn composition_nan() {
    rejected(
        replace(
            "composition: {H: 2}\n- name: B",
            "composition: {H: .nan}\n- name: B",
        ),
        "composition",
        "A",
    );
}
#[test]
fn composition_negative() {
    rejected(
        replace(
            "composition: {H: 2}\n- name: B",
            "composition: {H: -2}\n- name: B",
        ),
        "composition",
        "A",
    );
}
#[test]
fn activation_nan() {
    rejected(replace("Ea: 0.0", "Ea: .nan"), "activation", "Ea");
}
#[test]
fn rate_infinity() {
    rejected(replace("A: 1.0", "A: .inf"), "rate", "A");
}
#[test]
fn rate_zero() {
    rejected(replace("A: 1.0", "A: 0.0"), "rate", "A");
}
#[test]
fn unsupported_phase() {
    rejected(
        replace("thermo: ideal-gas", "thermo: surface"),
        "phase",
        "gas",
    );
}
#[test]
fn multiple_documents() {
    rejected(format!("{BASE}---\ndescription: other\n"), "yaml", "");
}
#[test]
fn malformed_flow() {
    rejected(replace("species: all", "species: [A, B"), "yaml", "");
}
#[test]
fn duplicate_description() {
    rejected(
        replace(
            "description: synthetic mechanism",
            "description: synthetic mechanism\ndescription: other",
        ),
        "yaml",
        "",
    );
}

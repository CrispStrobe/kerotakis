//! Frozen prospective YAML edge controls. Synthetic constants are not measurements.
use kerotakis_core::kinetics::mechanism::{parse_yaml, MechanismError};
const BASE: &str = "description: synthetic mechanism\nphases:\n- name: gas\n  thermo: ideal-gas\n  species: all\n  reactions: all\nspecies:\n- name: A\n  composition: {H: 2}\n- name: B\n  composition: {H: 2}\nreactions:\n- equation: A => B\n  rate-constant: {A: 1.0, b: 0.0, Ea: 0.0}\n";
fn replace(old: &str, new: &str) -> String {
    assert_eq!(BASE.matches(old).count(), 1, "unambiguous input mutation");
    BASE.replace(old, new)
}
fn equal(text: String) {
    let reference = parse_yaml(BASE).expect("ordinary independently authored witness");
    assert_eq!(
        parse_yaml(&text).expect("supported annotation form"),
        reference
    );
}
enum Expected {
    Unit(&'static str, &'static str),
    Rate,
    Activation,
    Value(&'static str, &'static str),
    Field(&'static str, &'static str),
    Section(&'static str),
    Yaml,
}
fn rejected(text: String, expected: Expected) {
    parse_yaml(BASE).expect("ordinary witness must remain admitted");
    let error = parse_yaml(&text).expect_err("frozen expectation requires refusal");
    match (expected, error) {
        (Expected::Unit(want_kind, want_unit), MechanismError::UnsupportedUnit { kind, unit }) => {
            assert_eq!(kind, want_kind);
            assert_eq!(unit, want_unit);
        }
        (
            Expected::Rate,
            MechanismError::InvalidRate {
                reaction,
                field,
                value,
            },
        ) => {
            assert_eq!(reaction, 1);
            assert_eq!(field, "A");
            assert!(value.is_infinite() && value.is_sign_negative());
        }
        (Expected::Activation, MechanismError::NonFiniteActivationEnergy { reaction, value }) => {
            assert_eq!(reaction, 1);
            assert!(value.is_infinite() && value.is_sign_negative());
        }
        (
            Expected::Value(want_owner, want_field),
            MechanismError::UnsupportedFieldValue {
                owner,
                field,
                value,
            },
        ) => {
            assert_eq!(owner, want_owner);
            assert_eq!(field, want_field);
            assert!(value.contains("external.yaml"));
        }
        (
            Expected::Field(want_owner, want_field),
            MechanismError::UnsupportedField { owner, field },
        ) => {
            assert_eq!(owner, want_owner);
            assert_eq!(field, want_field);
        }
        (Expected::Section(want_section), MechanismError::UnsupportedSection { section }) => {
            assert_eq!(section, want_section);
        }
        (Expected::Yaml, MechanismError::Yaml(_)) => {}
        (_, other) => panic!("wrong frozen refusal category: {other:?}"),
    }
}
#[test]
fn quoted_rate_decimal() {
    rejected(
        replace("A: 1.0", "A: \"1.0\""),
        Expected::Unit("pre-exponential", "1.0"),
    );
}

#[test]
fn quoted_rate_scientific() {
    rejected(
        replace("A: 1.0", "A: \"1e0\""),
        Expected::Unit("pre-exponential", "1e0"),
    );
}

#[test]
fn quoted_activation_decimal() {
    rejected(
        replace("Ea: 0.0", "Ea: \"0.0\""),
        Expected::Unit("activation-energy", "0.0"),
    );
}

#[test]
fn quoted_activation_scientific() {
    rejected(
        replace("Ea: 0.0", "Ea: \"0e0\""),
        Expected::Unit("activation-energy", "0e0"),
    );
}

#[test]
fn quoted_rate_infinity() {
    rejected(
        replace("A: 1.0", "A: \".inf\""),
        Expected::Unit("pre-exponential", ".inf"),
    );
}

#[test]
fn quoted_activation_nan() {
    rejected(
        replace("Ea: 0.0", "Ea: \".nan\""),
        Expected::Unit("activation-energy", ".nan"),
    );
}

#[test]
fn unquoted_rate_negative_infinity() {
    rejected(replace("A: 1.0", "A: -.inf"), Expected::Rate);
}

#[test]
fn unquoted_activation_negative_infinity() {
    rejected(replace("Ea: 0.0", "Ea: -.inf"), Expected::Activation);
}

#[test]
fn ignored_custom_scalar_tag() {
    equal(format!("{BASE}note: !AuthorNote provenance\n"));
}

#[test]
fn ignored_custom_sequence_tag() {
    equal(format!("{BASE}note: !AuthorNotes [one, two]\n"));
}

#[test]
fn tagged_species_external_mapping() {
    rejected(
        replace("species: all", "species: !ExternalSet {external.yaml: all}"),
        Expected::Value("phase 'gas'", "species"),
    );
}

#[test]
fn custom_tag_rate_scalar() {
    rejected(replace("A: 1.0", "A: !CustomRate 1.0"), Expected::Yaml);
}

#[test]
fn ordinary_merge_document() {
    rejected(
        format!("{BASE}<<: {{description: ignored}}\n"),
        Expected::Section("<<"),
    );
}

#[test]
fn ordinary_merge_phase() {
    rejected(
        replace(
            "  thermo: ideal-gas",
            "  thermo: ideal-gas\n  <<: {transport: ignored}",
        ),
        Expected::Field("phase 'gas'", "<<"),
    );
}

#[test]
fn ordinary_merge_species() {
    rejected(
        replace(
            "  composition: {H: 2}\n- name: B",
            "  composition: {H: 2}\n  <<: {note: ignored}\n- name: B",
        ),
        Expected::Field("species 'A'", "<<"),
    );
}

#[test]
fn ignored_nested_merge_alias() {
    equal(format!("{BASE}note:\n  defaults: &annotation {{author: witness}}\n  selected: {{<<: *annotation, detail: retained}}\n"));
}

#[test]
fn duplicate_document_annotation() {
    equal(format!("{BASE}note: first\nnote: second\n"));
}

#[test]
fn duplicate_known_rate_field() {
    rejected(replace("A: 1.0", "A: 1.0, A: 2.0"), Expected::Yaml);
}

#[test]
fn duplicate_composition_last_value() {
    equal(replace(
        "composition: {H: 2}\n- name: B",
        "composition: {H: 9, H: 2}\n- name: B",
    ));
}

#[test]
fn duplicate_nested_annotation_key() {
    rejected(
        format!("{BASE}note: {{author: first, author: second}}\n"),
        Expected::Yaml,
    );
}

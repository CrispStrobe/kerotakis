//! Nested modifiers must refuse by owner rather than silently changing rates.
//! These synthetic admission controls make no measured-kinetics claim.
use kerotakis_core::kinetics::mechanism::{parse_yaml, MechanismError};

const HEADER: &str = "description: nested rate admission\nunits: {length: cm, quantity: mol, activation-energy: J/mol, pressure: Pa}\nphases:\n- name: gas\n  thermo: ideal-gas\n  species: all\n  reactions: all\nspecies:\n- name: A\n  composition: {H: 2}\n- name: B\n  composition: {H: 2}\nreactions:\n";
const ELEMENTARY: &str = "- equation: A => B\n  rate-constant: {A: 1.0, b: 0.0, Ea: 0.0}\n";
const THIRD_BODY: &str =
    "- equation: A + M => B + M\n  type: three-body\n  rate-constant: {A: 1.0, b: 0.0, Ea: 0.0}\n";
const FALLOFF: &str = "- equation: A (+M) => B (+M)\n  type: falloff\n  high-P-rate-constant: {A: 1.0, b: 0.0, Ea: 0.0}\n  low-P-rate-constant: {A: 2.0, b: 0.0, Ea: 0.0}\n  Troe: {A: 0.5, T3: 1000.0, T1: 10000.0, T2: 5000.0}\n";
const PLOG: &str = "- equation: A => B\n  type: pressure-dependent-Arrhenius\n  rate-constants:\n  - {P: 100000.0, A: 1.0, b: 0.0, Ea: 0.0}\n  - {P: 200000.0, A: 2.0, b: 0.0, Ea: 0.0}\n";

fn input(reaction: &str) -> String {
    format!("{HEADER}{reaction}")
}

fn unsupported(reaction: &str, owner: &str, field: &str) {
    match parse_yaml(&input(reaction)).expect_err("unsupported nested field must refuse") {
        MechanismError::UnsupportedField {
            owner: actual_owner,
            field: actual_field,
        } => {
            assert_eq!(actual_owner, format!("reaction 1 {owner}"));
            assert_eq!(actual_field, field);
        }
        error => panic!("expected contextual UnsupportedField, got {error:?}"),
    }
}

#[test]
fn eight_observed_silent_acceptances_refuse_contextually() {
    for (reaction, owner, field) in [
        (ELEMENTARY.replace("b: 0.0", "B: 1.0"), "rate-constant", "B"),
        (
            ELEMENTARY.replace("Ea: 0.0}", "Ea: 0.0, units: {time: min}}"),
            "rate-constant",
            "units",
        ),
        (
            ELEMENTARY.replace("Ea: 0.0}", "Ea: 0.0, orders: {A: 2.0}}"),
            "rate-constant",
            "orders",
        ),
        (
            FALLOFF.replace(
                "A: 1.0, b: 0.0, Ea: 0.0}",
                "A: 1.0, b: 0.0, Ea: 0.0, negative-A: true}",
            ),
            "high-P-rate-constant",
            "negative-A",
        ),
        (
            FALLOFF.replace(
                "A: 2.0, b: 0.0, Ea: 0.0}",
                "A: 2.0, b: 0.0, Ea: 0.0, units: {quantity: kmol}}",
            ),
            "low-P-rate-constant",
            "units",
        ),
        (FALLOFF.replace("T2: 5000.0", "t2: 5000.0"), "Troe", "t2"),
        (
            PLOG.replace(
                "P: 100000.0, A: 1.0, b: 0.0, Ea: 0.0}",
                "P: 100000.0, A: 1.0, b: 0.0, Ea: 0.0, units: {pressure: atm}}",
            ),
            "rate-constants entry 1",
            "units",
        ),
        (
            PLOG.replace(
                "P: 200000.0, A: 2.0, b: 0.0, Ea: 0.0}",
                "P: 200000.0, A: 2.0, b: 0.0, Ea: 0.0, orders: {A: 2.0}}",
            ),
            "rate-constants entry 2",
            "orders",
        ),
    ] {
        unsupported(&reaction, owner, field);
    }
}

#[test]
fn third_body_rate_unknown_field_refuses() {
    unsupported(
        &THIRD_BODY.replace("Ea: 0.0}", "Ea: 0.0, B: 1.0}"),
        "rate-constant",
        "B",
    );
}

#[test]
fn unused_nested_owners_cannot_hide_unknown_fields() {
    for (reaction, owner) in [
        (format!("{ELEMENTARY}  high-P-rate-constant: {{A: 1.0, Ea: 0.0, foreign: true}}\n"), "high-P-rate-constant"),
        (format!("{ELEMENTARY}  low-P-rate-constant: {{A: 1.0, Ea: 0.0, foreign: true}}\n"), "low-P-rate-constant"),
        (format!("{FALLOFF}  rate-constant: {{A: 1.0, Ea: 0.0, foreign: true}}\n"), "rate-constant"),
        (format!("{THIRD_BODY}  Troe: {{A: 0.5, T3: 1000.0, T1: 10000.0, foreign: true}}\n"), "Troe"),
        (format!("{ELEMENTARY}  rate-constants:\n  - {{P: 100000.0, A: 1.0, Ea: 0.0, foreign: true}}\n"), "rate-constants entry 1"),
    ] {
        unsupported(&reaction, owner, "foreign");
    }
}

#[test]
fn pressure_key_is_consumed_by_plog_owner() {
    parse_yaml(&input(PLOG)).expect("valid P is not a flattened rate modifier");
    unsupported(
        &ELEMENTARY.replace("Ea: 0.0}", "Ea: 0.0, P: 100000.0}"),
        "rate-constant",
        "P",
    );
}

#[test]
fn known_troe_t2_remains_supported_and_optional() {
    parse_yaml(&input(FALLOFF)).expect("known T2");
    parse_yaml(&input(&FALLOFF.replace(", T2: 5000.0", ""))).expect("optional T2");
}

#[test]
fn exponent_default_preserves_normalized_mechanism() {
    for reaction in [ELEMENTARY, THIRD_BODY, FALLOFF, PLOG] {
        assert_eq!(
            parse_yaml(&input(reaction)).unwrap(),
            parse_yaml(&input(&reaction.replace("b: 0.0, ", ""))).unwrap()
        );
    }
}

#[test]
fn missing_or_misspelled_activation_energy_remains_yaml_error() {
    for reaction in [ELEMENTARY, THIRD_BODY, FALLOFF, PLOG] {
        for changed in [
            reaction.replace("Ea: 0.0", "ea: 0.0"),
            reaction.replace(", Ea: 0.0", ""),
        ] {
            assert!(matches!(
                parse_yaml(&input(&changed)),
                Err(MechanismError::Yaml(_))
            ));
        }
    }
}

#[test]
fn duplicate_known_fields_remain_yaml_errors() {
    for reaction in [ELEMENTARY, THIRD_BODY, FALLOFF, PLOG] {
        for (field, replacement) in [
            ("A: 1.0", "A: 1.0, A: 2.0"),
            ("b: 0.0", "b: 0.0, b: 1.0"),
            ("Ea: 0.0", "Ea: 0.0, Ea: 1.0"),
        ] {
            assert!(
                matches!(
                    parse_yaml(&input(&reaction.replace(field, replacement))),
                    Err(MechanismError::Yaml(_))
                ),
                "{reaction}: {field}"
            );
        }
    }
    assert!(matches!(
        parse_yaml(&input(
            &PLOG.replace("P: 100000.0", "P: 100000.0, P: 150000.0")
        )),
        Err(MechanismError::Yaml(_))
    ));
    assert!(matches!(
        parse_yaml(&input(
            &FALLOFF.replace("T2: 5000.0", "T2: 5000.0, T2: 6000.0")
        )),
        Err(MechanismError::Yaml(_))
    ));
}

#[test]
fn known_field_type_errors_remain_yaml_errors() {
    for reaction in [ELEMENTARY, THIRD_BODY, FALLOFF, PLOG] {
        assert!(matches!(
            parse_yaml(&input(&reaction.replace("b: 0.0", "b: [0.0]"))),
            Err(MechanismError::Yaml(_))
        ));
    }
    assert!(matches!(
        parse_yaml(&input(&FALLOFF.replace("T2: 5000.0", "T2: [5000.0]"))),
        Err(MechanismError::Yaml(_))
    ));
}

#[test]
fn numeric_admission_is_not_weakened_by_unknown_capture() {
    assert!(matches!(
        parse_yaml(&input(&ELEMENTARY.replace("A: 1.0", "A: 0.0"))),
        Err(MechanismError::InvalidRate { .. })
    ));
    assert!(matches!(
        parse_yaml(&input(&ELEMENTARY.replace("Ea: 0.0", "Ea: .nan"))),
        Err(MechanismError::NonFiniteActivationEnergy { .. })
    ));
}

#[test]
fn established_annotation_owners_remain_accepted() {
    let annotated = format!(
        "{}  note: {{details: [1, true, null]}}\n  id: authored\n  duplicate: true\n",
        input(ELEMENTARY)
    );
    assert_eq!(
        parse_yaml(&annotated).unwrap(),
        parse_yaml(&input(ELEMENTARY)).unwrap()
    );
    let document_note = format!("{}note: {{details: [1, true, null]}}\n", input(ELEMENTARY));
    assert_eq!(
        parse_yaml(&document_note).unwrap(),
        parse_yaml(&input(ELEMENTARY)).unwrap()
    );
}

#[test]
fn plog_pressure_ownership_is_independent_of_key_order() {
    let reordered = PLOG
        .replace(
            "P: 100000.0, A: 1.0, b: 0.0, Ea: 0.0",
            "A: 1.0, b: 0.0, Ea: 0.0, P: 100000.0",
        )
        .replace(
            "P: 200000.0, A: 2.0, b: 0.0, Ea: 0.0",
            "A: 2.0, b: 0.0, Ea: 0.0, P: 200000.0",
        );
    assert_eq!(
        parse_yaml(&input(PLOG)).unwrap(),
        parse_yaml(&input(&reordered)).unwrap()
    );
}

#[test]
fn supplied_troe_t2_survives_lowering() {
    assert_ne!(
        parse_yaml(&input(FALLOFF)).unwrap(),
        parse_yaml(&input(&FALLOFF.replace(", T2: 5000.0", ""))).unwrap()
    );
}

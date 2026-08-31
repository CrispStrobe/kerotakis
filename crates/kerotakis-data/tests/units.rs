//! BRD-003: the checked-in spelling table is the contract every importer
//! reads. Adding an upstream spelling means adding a row here, so a reviewer
//! sees exactly which strings became which canonical unit.

use std::path::Path;

use kerotakis_data::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Case {
    #[serde(default)]
    #[allow(dead_code)]
    note: Option<String>,
    input: String,
    value: f64,
    /// The dimension the target field carries, when it declares one.
    #[serde(default)]
    dimension: Option<Dimension>,
    #[serde(default)]
    expect: Option<Expected>,
    /// The typed refusal this spelling must produce.
    #[serde(default)]
    reject: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Expected {
    symbol: String,
    dimension: Dimension,
    value: f64,
}

fn cases() -> Vec<Case> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/units/spellings.json");
    let bytes = std::fs::read(&path).expect("read the spelling fixture");
    serde_json::from_slice(&bytes).expect("parse the spelling fixture")
}

fn refusal(error: &UnitNormalizationError) -> &'static str {
    match error {
        UnitNormalizationError::EmptyUnit => "empty_unit",
        UnitNormalizationError::UnknownUnit { .. } => "unknown_unit",
        UnitNormalizationError::DimensionMismatch { .. } => "dimension_mismatch",
        UnitNormalizationError::UnsupportedDimension { .. } => "unsupported_dimension",
        UnitNormalizationError::NonFiniteValue { .. } => "non_finite_value",
        UnitNormalizationError::BelowAbsoluteZero { .. } => "below_absolute_zero",
    }
}

fn normalize(case: &Case) -> Result<NormalizedQuantity, UnitNormalizationError> {
    match &case.dimension {
        Some(dimension) => normalize_quantity_for(case.value, &case.input, dimension),
        None => normalize_quantity(case.value, &case.input),
    }
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1e-9 * expected.abs().max(1.0)
}

#[test]
fn the_fixture_covers_a_meaningful_spread_of_upstream_spellings() {
    let cases = cases();
    assert!(
        cases.len() >= 30,
        "the spelling fixture is the coverage floor: {} cases",
        cases.len()
    );
    assert!(cases.iter().any(|case| case.reject.is_some()));
    assert!(cases.iter().any(|case| case.expect.is_some()));
}

#[test]
fn every_fixture_spelling_normalizes_or_refuses_exactly_as_recorded() {
    for case in cases() {
        let outcome = normalize(&case);
        match (&case.expect, &case.reject) {
            (Some(expected), None) => {
                let actual = outcome
                    .unwrap_or_else(|error| panic!("{:?} should normalize: {error}", case.input));
                assert_eq!(actual.unit.symbol, expected.symbol, "{:?}", case.input);
                assert_eq!(
                    actual.unit.dimension, expected.dimension,
                    "{:?}",
                    case.input
                );
                assert!(
                    close(actual.value, expected.value),
                    "{:?}: expected {}, got {}",
                    case.input,
                    expected.value,
                    actual.value
                );
            }
            (None, Some(reason)) => {
                let error = outcome
                    .expect_err(&format!("{:?} should have been refused", case.input))
                    .clone();
                assert_eq!(refusal(&error), reason, "{:?}", case.input);
                if reason != "empty_unit" {
                    assert_eq!(
                        error.original(),
                        case.input,
                        "the refusal must preserve the original spelling"
                    );
                }
            }
            _ => panic!("{:?} must declare exactly one of expect/reject", case.input),
        }
    }
}

#[test]
fn a_normalized_value_normalizes_to_itself() {
    for case in cases() {
        let Ok(once) = normalize(&case) else { continue };
        let twice = normalize_quantity_for(once.value, &once.unit.symbol, &once.unit.dimension)
            .expect("a canonical spelling is always normalizable");
        assert_eq!(once, twice, "{:?} is not idempotent", case.input);
    }
}

#[test]
fn conversions_invert_back_onto_the_source_spelling() {
    for case in cases() {
        let conversion = match &case.dimension {
            Some(dimension) => normalize_unit_for(&case.input, dimension),
            None => normalize_unit(&case.input),
        };
        let Ok(conversion) = conversion else { continue };
        let back = conversion.invert(conversion.apply(case.value));
        assert!(
            close(back, case.value),
            "{:?} did not round-trip: {back}",
            case.input
        );
    }
}

#[test]
fn every_normalizable_dimension_has_exactly_one_canonical_spelling() {
    for dimension in normalizable_dimensions() {
        let symbol = canonical_symbol(&dimension).expect("a normalizable dimension has a symbol");
        let conversion =
            normalize_unit_for(symbol, &dimension).expect("its own symbol normalizes back");
        assert!(conversion.is_identity(), "{symbol} is not canonical");
    }
    assert!(canonical_symbol(&Dimension::RateConstant).is_none());
    assert!(canonical_symbol(&Dimension::Other("kerotakis-invented".into())).is_none());
}

#[test]
fn molecular_length_normalizes_to_angstroms_without_becoming_wavelength() {
    for (value, spelling, expected_angstrom) in
        [(3.7, "Å", 3.7), (0.37, "nm", 3.7), (3.7e-10, "m", 3.7)]
    {
        let normalized =
            normalize_quantity_for(value, spelling, &Dimension::MolecularLength).unwrap();
        assert_eq!(normalized.unit.symbol, "Ang");
        assert_eq!(normalized.unit.dimension, Dimension::MolecularLength);
        assert!(close(normalized.value, expected_angstrom));
    }

    // Preserve the established default and the explicitly typed optical path.
    assert_eq!(
        normalize_quantity(500.0, "nm").unwrap().unit.dimension,
        Dimension::Wavelength
    );
    assert_eq!(
        normalize_quantity_for(500.0, "nm", &Dimension::Wavelength)
            .unwrap()
            .unit
            .dimension,
        Dimension::Wavelength
    );
}

#[test]
fn physical_length_and_optical_wavelength_are_not_interchangeable() {
    assert!(matches!(
        normalize_quantity_for(1.0, "m", &Dimension::Wavelength),
        Err(UnitNormalizationError::DimensionMismatch {
            expected: Dimension::Wavelength,
            found: Dimension::MolecularLength,
            ..
        })
    ));
    assert!(matches!(
        normalize_quantity_for(500.0, "pm", &Dimension::MolecularLength),
        Err(UnitNormalizationError::DimensionMismatch {
            expected: Dimension::MolecularLength,
            found: Dimension::Wavelength,
            ..
        })
    ));
}

#[test]
fn a_non_finite_source_value_is_refused_rather_than_stored() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            normalize_quantity(value, "g/mL"),
            Err(UnitNormalizationError::NonFiniteValue { .. })
        ));
    }
    // An in-range value whose conversion overflows is caught on the far side.
    assert!(matches!(
        normalize_quantity(f64::MAX, "atm"),
        Err(UnitNormalizationError::NonFiniteValue { .. })
    ));
}

#[test]
fn a_quarantined_quantity_is_normalized_only_through_its_declared_dimension() {
    let field = CandidateField::new(serde_json::json!(1.03), "record.density", "CC0-1.0")
        .with_unit("g·cm⁻³");
    let rule =
        RuntimeFieldPolicy::new("mass_density", ["CC0-1.0"]).with_dimension(Dimension::MassDensity);
    let (value, unit) = normalize_candidate_quantity(&field, &rule)
        .expect("a reviewed density normalizes")
        .expect("a unit-bearing field is a quantity");
    assert_eq!(unit.symbol, "g/mL");
    assert!(close(value.as_f64().unwrap(), 1.03));

    // The same bytes against a field that declares a different dimension.
    let wrong = RuntimeFieldPolicy::new("boiling_point", ["CC0-1.0"])
        .with_dimension(Dimension::Temperature);
    assert!(matches!(
        normalize_candidate_quantity(&field, &wrong),
        Err(FieldRejectionReason::UnitNotNormalized { .. })
    ));

    // A quantity field with no unit, and a unit field with no number.
    let unitless = CandidateField::new(serde_json::json!(1.03), "record.density", "CC0-1.0");
    assert!(matches!(
        normalize_candidate_quantity(&unitless, &rule),
        Err(FieldRejectionReason::MissingUnit { .. })
    ));
    let textual = CandidateField::new(serde_json::json!("1.03 g/cm3"), "record.density", "CC0-1.0")
        .with_unit("g/cm3");
    assert!(matches!(
        normalize_candidate_quantity(&textual, &rule),
        Err(FieldRejectionReason::NonNumericQuantity { .. })
    ));
}

#[test]
fn review_records_both_the_canonical_and_the_source_unit() {
    use std::collections::BTreeMap;

    let candidate = QuarantinedCandidate {
        adapter_id: "synthetic-v1".into(),
        source_record_id: "snapshot/ethanol".into(),
        external_record_id: "ethanol".into(),
        identity_key: None,
        fields: BTreeMap::from([
            (
                "density".to_string(),
                CandidateField::new(serde_json::json!(0.789), "record.density", "CC0-1.0")
                    .with_unit("g/cm3"),
            ),
            (
                "boiling_point".to_string(),
                CandidateField::new(serde_json::json!(78.37), "record.bp", "CC0-1.0")
                    .with_unit("°C"),
            ),
            (
                "vapour_pressure".to_string(),
                CandidateField::new(serde_json::json!(5.95), "record.vp", "CC0-1.0")
                    .with_unit("kPa at 20 C"),
            ),
        ]),
    };
    let policy = PromotionPolicy {
        fields: BTreeMap::from([
            (
                "density".to_string(),
                RuntimeFieldPolicy::new("mass_density", ["CC0-1.0"])
                    .with_dimension(Dimension::MassDensity),
            ),
            (
                "boiling_point".to_string(),
                RuntimeFieldPolicy::new("boiling_point", ["CC0-1.0"])
                    .with_dimension(Dimension::Temperature),
            ),
            (
                "vapour_pressure".to_string(),
                RuntimeFieldPolicy::new("vapour_pressure", ["CC0-1.0"])
                    .with_dimension(Dimension::Pressure),
            ),
        ]),
    };

    let review = review_candidate(&candidate, &policy);
    let density = &review.accepted["mass_density"];
    assert_eq!(density.unit.as_ref().unwrap().symbol, "g/mL");
    assert_eq!(density.source_unit.as_deref(), Some("g/cm3"));
    let boiling = &review.accepted["boiling_point"];
    assert!(close(boiling.value.as_f64().unwrap(), 351.52));
    assert_eq!(boiling.unit.as_ref().unwrap().symbol, "K");

    // A unit smuggling conditions along with it is refused verbatim, not
    // parsed heuristically.
    assert!(!review.accepted.contains_key("vapour_pressure"));
    let rejection = review
        .rejected
        .iter()
        .find(|rejection| rejection.field == "vapour_pressure")
        .expect("the compound spelling is rejected");
    let FieldRejectionReason::UnitNotNormalized { unit, .. } = &rejection.reason else {
        panic!("expected a unit refusal, got {:?}", rejection.reason);
    };
    assert_eq!(unit, "kPa at 20 C");
}

//! Frozen before implementing the sparse optical-point API. No vessel spectrum claim.
use kerotakis_core::sparse_optics::{
    measure_free_iodine, reviewed_iodine_record, CertifiedFreeIodine,
    ExperimentalMedium, OpticalSolvent, SparseOpticsError,
};

fn certified(c: f64) -> CertifiedFreeIodine {
    CertifiedFreeIodine::from_independent_measurement(c, "independent free-I2 assay receipt").unwrap()
}
fn medium() -> ExperimentalMedium {
    ExperimentalMedium {
        solvent: OpticalSolvent::Water,
        perchloric_acid_molar: 0.10,
        potassium_iodate_molar: 1.5e-6,
    }
}
fn read(c: f64, path: f64) -> Result<f64, SparseOpticsError> {
    measure_free_iodine(&certified(c), medium(), 298.15, 460.0, path).map(|r| r.absorbance)
}

#[test]
fn reviewed_point_has_the_independent_aqueous_prediction() {
    assert!((read(1e-4, 1.0).unwrap() - 0.0746).abs() < 1e-12);
}
#[test]
fn concentration_and_path_scale_independently() {
    let a = read(1e-4, 1.0).unwrap();
    assert!((read(2e-4, 1.0).unwrap() / a - 2.0).abs() < 1e-12);
    assert!((read(1e-4, 2.0).unwrap() / a - 2.0).abs() < 1e-12);
}
#[test]
fn exact_zero_concentration_has_zero_absorbance() {
    assert_eq!(read(0.0, 1.0).unwrap(), 0.0);
}
#[test]
fn invalid_concentration_is_rejected_at_certification_boundary() {
    for c in [-1e-4, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(CertifiedFreeIodine::from_independent_measurement(c, "assay"), Err(SparseOpticsError::InvalidConcentration)));
    }
}
#[test]
fn certification_provenance_cannot_be_empty() {
    for evidence in ["", " ", "\n\t"] {
        assert!(matches!(CertifiedFreeIodine::from_independent_measurement(1e-4, evidence), Err(SparseOpticsError::MissingCertification)));
    }
}
#[test]
fn invalid_path_is_rejected_even_for_a_blank() {
    for path in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for c in [0.0, 1e-4] {
            assert!(matches!(read(c, path), Err(SparseOpticsError::InvalidPath)));
        }
    }
}
#[test]
fn unreviewed_wavelength_is_not_rounded_to_a_nearby_grid_bin() {
    for wavelength in [465.0, 445.0, 524.0, 525.0, 353.0, 0.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(measure_free_iodine(&certified(1e-4), medium(), 298.15, wavelength, 1.0), Err(SparseOpticsError::UnsupportedWavelength)));
    }
}
#[test]
fn unreviewed_temperature_is_not_interpolated() {
    for temperature in [273.15, 298.15001, 310.0, 0.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(measure_free_iodine(&certified(1e-4), medium(), temperature, 460.0, 1.0), Err(SparseOpticsError::UnsupportedTemperature)));
    }
}
#[test]
fn hexane_cannot_borrow_the_aqueous_coefficient() {
    let mut organic = medium();
    organic.solvent = OpticalSolvent::Hexane;
    assert!(matches!(measure_free_iodine(&certified(1e-4), organic, 298.15, 460.0, 1.0), Err(SparseOpticsError::UnsupportedMedium)));
}
#[test]
fn acid_and_iodate_conditions_are_required_independently() {
    for acid in [0.0, 0.01, -0.1, f64::NAN, f64::INFINITY] {
        let mut m = medium(); m.perchloric_acid_molar = acid;
        assert!(matches!(measure_free_iodine(&certified(1e-4), m, 298.15, 460.0, 1.0), Err(SparseOpticsError::UnsupportedMedium)));
    }
    for iodate in [0.0, 1.0e-6, -1.5e-6, f64::NAN, f64::INFINITY] {
        let mut m = medium(); m.potassium_iodate_molar = iodate;
        assert!(matches!(measure_free_iodine(&certified(1e-4), m, 298.15, 460.0, 1.0), Err(SparseOpticsError::UnsupportedMedium)));
    }
}
#[test]
fn unsupported_conditions_are_not_supported_just_because_sample_is_blank() {
    let mut m = medium(); m.perchloric_acid_molar = 0.0;
    assert!(matches!(measure_free_iodine(&certified(0.0), m, 298.15, 460.0, 1.0), Err(SparseOpticsError::UnsupportedMedium)));
}
#[test]
fn final_overflow_is_explicit() {
    assert!(matches!(read(f64::MAX, f64::MAX), Err(SparseOpticsError::UnrepresentableAbsorbance)));
}
#[test]
fn intermediate_overflow_does_not_refuse_a_representable_product() {
    let a = read(1e308, 1e-308).unwrap();
    assert!((a / 746.0 - 1.0).abs() < 1e-12);
}
#[test]
fn positive_underflow_is_not_reported_as_a_clear_sample() {
    assert!(matches!(read(f64::from_bits(1), f64::from_bits(1)), Err(SparseOpticsError::UnrepresentableAbsorbance)));
}
#[test]
fn output_preserves_certification_source_and_exact_point_scope() {
    let r = measure_free_iodine(&certified(1e-4), medium(), 298.15, 460.0, 1.0).unwrap();
    assert_eq!(r.wavelength_nm, 460.0);
    assert_eq!(r.temperature_k, 298.15);
    assert_eq!(r.epsilon_l_mol_cm, 746.0);
    assert_eq!(r.certification_evidence, "independent free-I2 assay receipt");
    assert_eq!(r.source_doi, "10.1021/ja01148a504");
}
#[test]
fn machine_readable_record_keeps_species_medium_and_provenance() {
    let record = serde_json::to_value(reviewed_iodine_record()).unwrap();
    assert_eq!(record["species"], "I2");
    assert_eq!(record["wavelength_nm"], 460.0);
    assert_eq!(record["epsilon_l_mol_cm"], 746.0);
    assert_eq!(record["temperature_k"], 298.15);
    assert_eq!(record["medium"]["solvent"], "water");
    assert_eq!(record["medium"]["perchloric_acid_molar"], 0.10);
    assert_eq!(record["medium"]["potassium_iodate_molar"], 1.5e-6);
    assert_eq!(record["source_doi"], "10.1021/ja01148a504");
    assert!(record["limitations"].as_array().is_some_and(|a| !a.is_empty()));
}
#[test]
fn sparse_record_does_not_claim_to_fill_the_existing_visible_spectrum() {
    let record = reviewed_iodine_record();
    assert_eq!(record.wavelength_nm, 460.0);
    assert!(!kerotakis_core::spectrum::BAND_NM.contains(&460.0));
    assert!(kerotakis_core::species::lookup(&kerotakis_core::SpeciesId::new("I2")).unwrap().spectrum.is_none());
}

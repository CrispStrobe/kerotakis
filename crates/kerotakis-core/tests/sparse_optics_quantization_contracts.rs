//! Numerical scope controls; extreme concentrations do not establish empirical calibration.
use kerotakis_core::sparse_optics::*;
fn read(c: f64, path: f64) -> Result<SparseAbsorbance, SparseOpticsError> {
    let c =
        CertifiedFreeIodine::from_independent_measurement(c, "arithmetic-only control").unwrap();
    measure_free_iodine(
        &c,
        ExperimentalMedium {
            solvent: OpticalSolvent::Water,
            perchloric_acid_molar: 0.1,
            potassium_iodate_molar: 1.5e-6,
        },
        298.15,
        460.0,
        path,
    )
}
#[test]
fn materially_quantized_nonzero_absorbance_is_not_an_accurate_result() {
    assert!(matches!(
        read(f64::from_bits(1), 0.001),
        Err(SparseOpticsError::UnrepresentableAbsorbance)
    ));
}
#[test]
fn accurate_subnormal_absorbance_remains_supported() {
    for bits in [1, 2, 746] {
        assert_eq!(
            read(f64::from_bits(bits), 1.0 / 746.0).unwrap().absorbance,
            f64::from_bits(bits)
        );
    }
}

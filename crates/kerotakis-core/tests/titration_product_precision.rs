//! Independent review controls for both operand orders and captured initial curve data.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
use kerotakis_core::*;
#[test]
fn nominal_product_checks_do_not_mask_error_in_either_small_operand() {
    for (c, volume) in [(f64::from_bits(3), 0.75), (1.5, f64::from_bits(1))] {
        let mut b = bench();
        let before = physical(&b);
        let mut m = Model::default();
        assert!(b
            .step_with(op(c, volume, 30.0, 1), &mut m, &PermissiveScreen)
            .is_err());
        assert_eq!(physical(&b), before);
        assert_eq!(m.calls, 0);
        assert!(b.log.is_empty());
    }
}
#[test]
fn exactly_represented_subnormal_nominal_concentration_remains_supported() {
    let mut b = bench();
    let mut m = Model {
        quantum: true,
        ..Model::default()
    };
    let q = f64::from_bits(1);
    let e = b
        .step_with(op(q, 1.0, 30.0, 1), &mut m, &PermissiveScreen)
        .unwrap();
    assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("NaOH")).0, q);
    assert_eq!(m.calls, 1);
    journal(&b, &e);
}
#[test]
fn solver_repair_cannot_hide_nonfinite_initial_curve_points() {
    for potential in [false, true] {
        let mut b = bench();
        let info = b.vessels[0].solution.as_mut().unwrap();
        if potential {
            info.pe = Some(f64::NAN)
        } else {
            info.ph = f64::NAN
        };
        let before = physical(&b);
        let mut m = Model::default();
        assert!(b
            .step_with(op(10.0, 0.001, 30.0, 1), &mut m, &PermissiveScreen)
            .is_err());
        assert_eq!(physical(&b), before);
        assert_eq!(m.calls, 0);
        assert!(b.log.is_empty());
    }
}

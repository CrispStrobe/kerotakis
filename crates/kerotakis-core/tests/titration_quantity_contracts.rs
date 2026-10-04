//! Receiver, fraction and invalid-input contracts frozen before baseline execution.
mod common {
    include!("common/titration_accounting.rs");
}
use common::*;
use kerotakis_core::ops::Endpoint;
use kerotakis_core::*;
#[test]
fn two_negative_inputs_do_not_make_a_positive_dose() {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model::default();
    assert!(b
        .step_with(op(-10.0, -0.001, 7.0, 1), &mut m, &PermissiveScreen)
        .is_err());
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 0);
    assert!(b.log.is_empty());
}
#[test]
fn nonfinite_concentration_or_step_refuses_before_any_trial() {
    for n in [f64::NAN, f64::INFINITY] {
        for o in [op(n, 0.001, 7.0, 1), op(10.0, n, 7.0, 1)] {
            let mut b = bench();
            let before = physical(&b);
            let mut m = Model::default();
            assert!(b.step_with(o, &mut m, &PermissiveScreen).is_err());
            assert_eq!(physical(&b), before);
            assert_eq!(m.calls, 0);
            assert!(b.log.is_empty());
        }
    }
}
#[test]
fn nonfinite_active_endpoint_refuses() {
    for n in [f64::NAN, f64::INFINITY] {
        for endpoint in [
            Endpoint::Ph,
            Endpoint::Pe {
                compare: ops::Compare::Above,
                value: n,
            },
        ] {
            let mut b = bench();
            let mut o = op(10.0, 0.001, n, 1);
            if let Operator::Titrate { endpoint: e, .. } = &mut o {
                *e = endpoint;
            }
            let before = physical(&b);
            let mut m = Model::default();
            assert!(b.step_with(o, &mut m, &PermissiveScreen).is_err());
            assert_eq!(physical(&b), before);
            assert_eq!(m.calls, 0);
        }
    }
}
#[test]
fn overflowing_carrier_conversion_refuses_before_trial() {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model::default();
    assert!(b
        .step_with(op(1e-308, 1e308, 30.0, 1), &mut m, &PermissiveScreen)
        .is_err());
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 0);
}
#[test]
fn swallowed_titrant_increment_refuses_atomically() {
    let mut b = bench();
    b.vessels[0].deposit(SpeciesId::new("NaOH"), Moles(1e20), Phase::Solid);
    quantity_refusal(b, op(10.0, 0.001, 30.0, 1), Model::default(), 0);
}
#[test]
fn inaccurate_nonzero_titrant_increment_refuses_atomically() {
    let mut b = bench();
    b.vessels[0].deposit(SpeciesId::new("NaOH"), Moles(1e12), Phase::Solid);
    quantity_refusal(b, op(1.0, 0.001, 30.0, 1), Model::default(), 0);
}
#[test]
fn swallowed_carrier_increment_refuses_atomically() {
    let mut b = bench();
    b.vessels[0].contents[0].moles = Moles(1e20);
    quantity_refusal(b, op(10.0, 0.001, 30.0, 1), Model::default(), 0);
}
#[test]
fn condensed_carrier_inventory_cannot_hide_behind_an_empty_phase() {
    let mut b = bench();
    b.vessels[0].contents[0].phase = Phase::Aqueous;
    b.vessels[0].contents[0].moles = Moles(1e20);
    quantity_refusal(b, op(10.0, 0.001, 30.0, 1), Model::default(), 0);
}
#[test]
fn receiver_overflow_refuses_atomically() {
    let mut b = bench();
    b.vessels[0].deposit(SpeciesId::new("NaOH"), Moles(1e308), Phase::Solid);
    quantity_refusal(b, op(1e308, 1.0, 30.0, 1), Model::default(), 0);
}
#[test]
fn underflowing_refinement_cannot_commit_provisional_full_dose() {
    let tiny = f64::from_bits(1);
    quantity_refusal(
        bench(),
        op(tiny / 0.001, 0.001, 7.0, 1),
        Model {
            quantum: true,
            ..Model::default()
        },
        1,
    );
}
#[test]
fn inaccurately_rounded_subnormal_refinement_refuses() {
    let tiny = f64::from_bits(3);
    quantity_refusal(
        bench(),
        op(tiny / 0.001, 0.001, 7.0, 1),
        Model {
            quantum: true,
            ..Model::default()
        },
        1,
    );
}
#[test]
fn invalid_successful_solver_proposal_never_reaches_flask() {
    let mut b = bench();
    let before = physical(&b);
    let mut m = Model {
        invalid: true,
        ..Model::default()
    };
    let e = b
        .step_with(op(10.0, 0.001, 30.0, 1), &mut m, &PermissiveScreen)
        .unwrap();
    assert_eq!(physical(&b), before);
    assert_eq!(m.calls, 1);
    assert!(e.iter().any(|e| matches!(e, Event::SolverFailed { .. })));
    journal(&b, &e);
}
#[test]
fn ordinary_and_exact_subnormal_full_doses_remain_supported() {
    for c in [10.0, f64::from_bits(1) / 0.001] {
        let mut b = bench();
        let mut m = Model {
            quantum: true,
            ..Model::default()
        };
        b.step_with(op(c, 0.001, 30.0, 1), &mut m, &PermissiveScreen)
            .unwrap();
        assert_eq!(m.calls, 1);
        assert_eq!(b.vessels[0].moles_of(&SpeciesId::new("NaOH")).0, c * 0.001);
    }
}
#[test]
fn solver_failure_does_not_claim_non_ph_step_limit_exhaustion() {
    for endpoint in [
        Endpoint::ColourPersists,
        Endpoint::Pe {
            compare: ops::Compare::Above,
            value: 8.0,
        },
    ] {
        let mut b = bench();
        let mut o = op(10.0, 0.001, 30.0, 9);
        if let Operator::Titrate { endpoint: e, .. } = &mut o {
            *e = endpoint;
        }
        let mut m = Model {
            fail: true,
            ..Model::default()
        };
        let e = b.step_with(o, &mut m, &PermissiveScreen).unwrap();
        assert_eq!(e.len(), 1, "{e:?}");
        assert!(matches!(e[0], Event::SolverFailed { .. }));
        journal(&b, &e);
    }
}

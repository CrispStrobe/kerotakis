//! References frozen before the source repair, using Decimal/Simpson quadrature.
use kerotakis_core::{
    heat_capacity::{CpForm, CpInterval, CpPolynomial},
    states, water_fusion, Phase,
};
use serde_json::Value;

fn references() -> Value {
    serde_json::from_str(include_str!(
        "../../../audits/water-fusion-authority-20261023/references.json"
    ))
    .unwrap()
}
fn number(v: &Value) -> f64 {
    v.as_str().unwrap().parse().unwrap()
}

#[test]
fn independent_shipped_curve_quadrature_agrees_at_all_frozen_temperatures() {
    for row in references()["controls"].as_array().unwrap() {
        let t = number(&row["temperature_k"]);
        assert!(
            (water_fusion::enthalpy_at(t).unwrap() - number(&row["fusion_enthalpy_j_mol"])).abs()
                < 2e-6
        );
        assert!(
            (water_fusion::ln_activity_at(t).unwrap() - number(&row["ln_activity"])).abs() < 2e-11
        );
        assert!(
            (water_fusion::temperature_sensitivity_at(t).unwrap()
                - number(&row["dtemperature_dlnactivity"]))
            .abs()
                < 5e-8
        );
        let a = states::activity_at_freezing_point(t);
        assert!((water_fusion::temperature_at(a).unwrap() - t).abs() < 1e-9);
    }
}
#[test]
fn particle_caps_are_rederived_from_the_same_authority() {
    let refs = references();
    assert!(
        (states::brine_model_max_particle_molality()
            - number(&refs["ideal_cap_particle_molality"]))
        .abs()
            < 2e-8
    );
    let activity = states::SolventActivity::from_speciation((-0.018015_f64 * 2.0).exp(), 2.0, 1.0);
    assert!(
        (activity.particle_molality_freezing_at(252.0)
            - number(&refs["osmotic_phi_one_cap_particle_molality"]))
        .abs()
            < 2e-8
    );
}
#[test]
fn inverse_is_monotone_and_retains_the_dilute_limiting_slope() {
    let mut prior = states::activity_at_freezing_point(252.0);
    for i in 1..=423 {
        let t = 252.0 + 21.15 * i as f64 / 423.0;
        let a = states::activity_at_freezing_point(t);
        assert!(a > prior);
        assert!((states::freezing_point_from_activity(a) - t).abs() < 1e-9);
        prior = a;
    }
    let a = (-1e-7_f64).exp();
    let slope = (273.15 - water_fusion::temperature_at(a).unwrap()) / 1e-7;
    let expected = 8.31446261815324 * 273.15_f64.powi(2) / 6010.0;
    assert!((slope - expected).abs() < 2e-5);
}
#[test]
fn no_invalid_or_below_cap_root_is_admitted_as_a_temperature() {
    for a in [f64::NAN, f64::INFINITY, 0.0, -1.0, 1.000001, 0.1] {
        assert!(water_fusion::temperature_at(a).is_none());
        assert!(states::freezing_point_from_activity(a).is_nan());
    }
    for t in [f64::NAN, f64::INFINITY, 0.0, 251.999999, 273.150001] {
        assert!(water_fusion::ln_activity_at(t).is_none());
        assert!(water_fusion::enthalpy_at(t).is_none());
    }
    let cap = states::activity_at_freezing_point(252.0);
    assert!(water_fusion::temperature_at(cap).is_some());
    assert!(water_fusion::temperature_at(cap - 1e-10).is_none());
    assert!(water_fusion::temperature_at(cap + 1e-10).unwrap() > 252.0);
}

// Independent Simpson integral of Cp/T checks every analytic basis term,
// both forms, interval seams, held endpoints, antisymmetry and invalid input.
fn curve(form: CpForm) -> CpPolynomial {
    let coefficients = match form {
        CpForm::Nasa9 => [200.0, 10.0, 3.0, 0.01, 1e-5, 1e-8, 1e-11],
        CpForm::Shomate => [30.0, 1.0, 2.0, 3.0, 0.4, 0.0, 0.0],
    };
    let intervals = Box::leak(Box::new([
        CpInterval {
            t_min: 250.0,
            t_max: 275.0,
            form,
            coefficients,
            reference: "Synthetic analytic control",
        },
        CpInterval {
            t_min: 275.0,
            t_max: 300.0,
            form,
            coefficients,
            reference: "Synthetic analytic control",
        },
    ]));
    CpPolynomial {
        phase: Phase::Liquid,
        intervals,
        source: "Synthetic",
        method: "Analytic control",
        boundary: "Hold endpoints",
    }
}
#[test]
fn entropy_primitives_match_independent_quadrature_for_both_forms() {
    for form in [CpForm::Nasa9, CpForm::Shomate] {
        let p = curve(form);
        for (lo, hi) in [
            (230.0, 250.0),
            (250.0, 275.0),
            (275.0, 300.0),
            (300.0, 320.0),
            (275.0, 275.000001),
        ] {
            let n = 1024;
            let step = (hi - lo) / n as f64;
            let total = (0..=n)
                .map(|i| {
                    let t = lo + step * i as f64;
                    let w = if i == 0 || i == n {
                        1.0
                    } else if i % 2 == 0 {
                        2.0
                    } else {
                        4.0
                    };
                    w * p.cp(t).unwrap() / t
                })
                .sum::<f64>()
                * step
                / 3.0;
            let s = p.entropy_between(lo, hi).unwrap();
            assert!((s - total).abs() < 1e-11, "{form:?}: {s} vs {total}");
            assert_eq!(p.entropy_between(hi, lo), Some(-s));
        }
        assert!(
            (p.entropy_between(230.0, 320.0).unwrap()
                - (p.entropy_between(230.0, 275.0).unwrap()
                    + p.entropy_between(275.0, 320.0).unwrap()))
            .abs()
                < 1e-12
        );
        for t in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(p.entropy_between(t, 273.15).is_none());
        }
    }
}

#[test]
fn below_domain_state_emits_a_typed_refusal_without_inventory_mutation() {
    use kerotakis_core::{Equilibrator, Kelvin, Moles, SpeciesId, Vessel, VesselId};
    let mut v = Vessel::new(VesselId(0), "Synthetic out-of-domain activity");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.temperature = Kelvin(251.0);
    v.solution = Some(kerotakis_core::vessel::SolutionInfo {
        scope: Default::default(),
        solvent_kg: Some(0.018_015),
        pe: None,
        redox: vec![],
        ph: 7.0,
        ionic_strength: 20.0,
        species: vec![kerotakis_core::vessel::SpeciesDetail {
            name: "Na+".into(),
            molality: 40.0,
            activity: 40.0,
        }],
        provenance: None,
        solvent_activity: None,
    });
    let before = serde_json::to_value(&v).unwrap();
    let events = kerotakis_core::solve::StateEquilibrator
        .equilibrate(&mut v)
        .unwrap();
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
    let events = serde_json::to_string(&events).unwrap();
    assert!(
        events.contains("not-modeled.partial-freezing-cap"),
        "{events}"
    );
}

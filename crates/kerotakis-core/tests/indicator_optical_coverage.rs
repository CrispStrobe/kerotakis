//! Proposed causal controls, not executed or native solver observations.
use kerotakis_core::instrument::Spectrophotometer;
use kerotakis_core::vessel::AdsorbedAmount;
use kerotakis_core::*;
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "optics");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v
}
fn indicator(phase: Phase, n: f64) -> Vessel {
    let mut v = water();
    v.deposit(SpeciesId::new("methyl_orange"), Moles(n), phase);
    v
}
fn solved(v: &mut Vessel, ph: f64) {
    v.solution = Some(SolutionInfo {
        scope: Default::default(),
        solvent_kg: Some(0.1),
        particle_molality: None,
        pe: None,
        redox: Vec::new(),
        ph,
        ionic_strength: 0.0,
        species: Vec::new(),
        provenance: None,
        solvent_activity: None,
    });
}
fn ph_gaps(v: &Vessel) -> Vec<String> {
    serde_json::to_value(appearance::observe(v))
        .unwrap()
        .get("optical_ph_gaps")
        .map(|value| serde_json::from_value(value.clone()).unwrap())
        .unwrap_or_default()
}
fn unchanged(v: &Vessel) {
    let before = serde_json::to_value(v).unwrap();
    let _ = Spectrophotometer::default().measure_spectrum(v);
    let _ = Spectrophotometer::default().measure(v);
    let _ = appearance::observe(v);
    assert_eq!(serde_json::to_value(v).unwrap(), before);
}
#[test]
fn missing_ph_is_refusal_not_zero() {
    let v = indicator(Phase::Aqueous, 1e-6);
    let c = appearance::observe(&v);
    assert_eq!(ph_gaps(&v), vec!["methyl_orange"]);
    assert!(c.spectral_gaps.is_empty());
    assert!(Spectrophotometer::default().measure(&v).is_none());
    assert!(appearance::observe(&v)
        .notes
        .iter()
        .any(|p| p.key == "look.indicator-optics-missing-ph"));
    unchanged(&v);
}
#[test]
fn nonfinite_ph_refuses_without_poisoning_partial_bands() {
    for ph in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut v = indicator(Phase::Aqueous, 1e-6);
        solved(&mut v, ph);
        assert_eq!(ph_gaps(&v), vec!["methyl_orange"]);
        assert!(Spectrophotometer::default()
            .measure_spectrum(&v)
            .iter()
            .all(|a| a.is_finite()));
        assert!(Spectrophotometer::default().measure(&v).is_none());
        unchanged(&v);
    }
}
#[test]
fn finite_solved_ph_has_positive_spectrum() {
    for ph in [-1.0, 3.47, 8.0] {
        let mut v = indicator(Phase::Aqueous, 1e-6);
        solved(&mut v, ph);
        assert!(ph_gaps(&v).is_empty());
        let r = Spectrophotometer::default().measure(&v).unwrap();
        assert!(r.value.is_finite() && r.value > 0.0);
        unchanged(&v);
    }
}
#[test]
fn non_ph_absorber_does_not_need_solution() {
    let mut v = water();
    v.deposit(SpeciesId::new("MnO4-"), Moles(1e-6), Phase::Aqueous);
    assert!(v.solution.is_none());
    assert!(ph_gaps(&v).is_empty());
    assert!(Spectrophotometer::default().measure(&v).unwrap().value > 0.0);
    unchanged(&v);
}
#[test]
fn mixed_partial_spectrum_is_preserved_but_not_reported_complete() {
    let mut v = water();
    v.deposit(SpeciesId::new("MnO4-"), Moles(1e-6), Phase::Aqueous);
    let supported = Spectrophotometer::default().measure_spectrum(&v);
    assert!(supported.iter().any(|a| *a > 0.0));
    v.deposit(SpeciesId::new("methyl_orange"), Moles(1e-6), Phase::Aqueous);
    assert_eq!(Spectrophotometer::default().measure_spectrum(&v), supported);
    assert!(Spectrophotometer::default().measure(&v).is_none());
    unchanged(&v);
}
#[test]
fn valid_blank_stays_zero() {
    let v = water();
    assert!(ph_gaps(&v).is_empty());
    assert_eq!(Spectrophotometer::default().measure(&v).unwrap().value, 0.0);
    unchanged(&v);
}
#[test]
fn solid_indicator_has_no_aqueous_gap() {
    let v = indicator(Phase::Solid, 1e-6);
    assert!(ph_gaps(&v).is_empty());
    assert_eq!(
        Spectrophotometer::default().measure_spectrum(&v),
        [0.0; spectrum::BANDS]
    );
    unchanged(&v);
}
#[test]
fn adsorbed_indicator_has_no_aqueous_gap() {
    let mut v = water();
    v.deposit(
        SpeciesId::new("activated_charcoal"),
        Moles(0.01),
        Phase::Solid,
    );
    v.adsorbed.push(AdsorbedAmount {
        sorbent: SpeciesId::new("activated_charcoal"),
        sorbate: SpeciesId::new("methyl_orange"),
        moles: Moles(1e-6),
    });
    assert!(ph_gaps(&v).is_empty());
    assert_eq!(
        Spectrophotometer::default().measure_spectrum(&v),
        [0.0; spectrum::BANDS]
    );
    unchanged(&v);
}
#[test]
fn zero_indicator_has_no_gap() {
    let v = indicator(Phase::Aqueous, 0.0);
    assert!(ph_gaps(&v).is_empty());
    unchanged(&v);
}
#[test]
fn liquid_indicator_requires_ph() {
    let v = indicator(Phase::Liquid, 1e-6);
    assert_eq!(ph_gaps(&v), vec!["methyl_orange"]);
    assert!(Spectrophotometer::default().measure(&v).is_none());
    unchanged(&v);
}
#[test]
fn native_spectrum_and_ph_gaps_both_survive() {
    let mut v = indicator(Phase::Aqueous, 1e-6);
    solved(&mut v, f64::NAN);
    v.solution.as_mut().unwrap().species.push(SpeciesDetail {
        name: "Cu(NH3)4+2".into(),
        molality: 0.01,
        activity: 0.01,
    });
    let c = appearance::observe(&v);
    assert_eq!(c.spectral_gaps, vec!["Cu(NH3)4+2"]);
    assert_eq!(ph_gaps(&v), vec!["methyl_orange"]);
    let seen = appearance::observe(&v);
    assert!(seen.notes.iter().any(|p| p.key == "look.spectral-gap"));
    assert!(seen
        .notes
        .iter()
        .any(|p| p.key == "look.indicator-optics-missing-ph"));
    unchanged(&v);
}
#[test]
fn bench_and_appearance_refusals_are_localized_and_nonconsuming() {
    let v = indicator(Phase::Aqueous, 1e-6);
    let seen = appearance::observe(&v);
    let note = seen
        .notes
        .iter()
        .find(|p| p.key == "look.indicator-optics-missing-ph")
        .unwrap();
    let mut b = Bench::new();
    b.vessels[0] = v;
    let before = serde_json::to_value(&b.vessels[0].contents).unwrap();
    let events = b
        .step_with(
            Operator::Measure {
                vessel: VesselId(0),
                instrument: Instrument::Spectrophotometer,
            },
            &mut SolverStack::new(vec![]),
            &PermissiveScreen,
        )
        .unwrap();
    assert!(!events.iter().any(|e| matches!(
        e,
        Event::Measured {
            instrument: Instrument::Spectrophotometer,
            ..
        }
    )));
    let reason = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(p), ..
            } if p.key == "not-modeled.indicator-optics-missing-ph" => Some(p),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        serde_json::to_value(&b.vessels[0].contents).unwrap(),
        before
    );
    assert_eq!(b.vessels[0].elapsed_seconds, 0.0);
    for locale in [Locale::parse("de"), Locale::parse("fr")] {
        assert_ne!(reason.render(locale), reason.render(Locale::EN));
        assert_ne!(note.render(locale), note.render(Locale::EN));
        assert!(!reason.render(locale).contains("{species}"));
        assert!(!note.render(locale).contains("{species}"));
    }
}

#[test]
fn scene_serializes_distinct_ph_gap_without_inventing_missing_spectral_data() {
    let v = indicator(Phase::Aqueous, 1e-6);
    let seen = serde_json::to_value(appearance::observe(&v)).unwrap();
    let scene = serde_json::to_value(scene_vessel(&v)).unwrap();
    assert_eq!(
        seen["optical_ph_gaps"],
        serde_json::json!(["methyl_orange"])
    );
    assert!(seen.get("spectral_gaps").is_none());
    assert_eq!(
        scene["liquid"]["optical_ph_gaps"],
        serde_json::json!(["methyl_orange"])
    );
    assert!(scene["liquid"].get("spectral_gaps").is_none());
    unchanged(&v);
}
#[test]
fn scene_omits_empty_ph_gap_and_older_payloads_remain_readable() {
    let v = water();
    let mut seen = serde_json::to_value(appearance::observe(&v)).unwrap();
    let mut scene = serde_json::to_value(scene_vessel(&v)).unwrap();
    assert!(seen.get("optical_ph_gaps").is_none());
    assert!(scene["liquid"].get("optical_ph_gaps").is_none());
    seen.as_object_mut().unwrap().remove("optical_ph_gaps");
    let restored: Appearance = serde_json::from_value(seen).unwrap();
    assert!(serde_json::to_value(restored)
        .unwrap()
        .get("optical_ph_gaps")
        .is_none());
    scene["liquid"]
        .as_object_mut()
        .unwrap()
        .remove("optical_ph_gaps");
    let restored: kerotakis_core::scene::SceneVessel = serde_json::from_value(scene).unwrap();
    assert!(serde_json::to_value(restored).unwrap()["liquid"]
        .get("optical_ph_gaps")
        .is_none());
}

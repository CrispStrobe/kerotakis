//! A byproduct trace of water does not validate an aqueous model over bulk peroxide.
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn mixture(water: f64, peroxide: f64, acid: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "peroxide medium");
    v.deposit(SpeciesId::new("water"), Moles(water), Phase::Liquid);
    v.deposit(SpeciesId::new("H2O2"), Moles(peroxide), Phase::Liquid);
    if acid > 0.0 {
        v.deposit(SpeciesId::new("HClO"), Moles(acid), Phase::Liquid);
    }
    v
}

#[test]
fn direct_solver_declines_peroxide_rich_medium_without_calling_engine() {
    for (water, peroxide, acid) in [
        (0.00001, 0.326375, 0.019062),
        (0.00001, 0.326375, 0.0),
        (0.49, 0.51, 0.001),
    ] {
        let mut eq = PhreeqcEquilibrator::new().expect("solver");
        let mut v = mixture(water, peroxide, acid);
        let before = v.contents.clone();
        v.free_proton = 0.1;
        v.free_hydroxide = 0.2;
        let calls = eq.engine_calls();
        // Call the actual public entry point before checking the routing gate.
        let events = eq
            .equilibrate(&mut v)
            .expect("explicit domain refusal, not convergence failure");
        assert!(!eq.applies(&v));
        assert_eq!(eq.engine_calls(), calls);
        assert!(v.solution.is_none());
        assert_eq!((v.free_proton, v.free_hydroxide), (0.0, 0.0));
        assert_eq!(v.contents, before);
        assert!(events.iter().any(|e| matches!(e, Event::NotYetModeled { reason: Some(p), .. } if p.key == "not-modeled.peroxide-rich-medium")), "{events:?}");
        assert!(eq.equilibrate(&mut v).is_ok());
        assert_eq!(eq.engine_calls(), calls);
        assert_eq!(v.contents, before);
    }
}

#[cfg(feature = "engine")]
#[test]
fn water_rich_peroxide_still_solves_and_stale_complete_reading_is_withdrawn() {
    let mut eq = PhreeqcEquilibrator::new().expect("native engine");
    let mut v = mixture(5.55, 0.001, 0.0001);
    assert!(eq.applies(&v));
    eq.equilibrate(&mut v).expect("water-rich positive control");
    assert!(
        eq.engine_calls() > 0,
        "positive control must actually use native chemistry"
    );
    assert!(v.solution.as_ref().is_some_and(|s| s.ph.is_finite()));
    // Keep a real native reading while replacing the medium. Honesty must
    // check this domain before its Complete-solution early return.
    v.contents.clear();
    v.deposit(SpeciesId::new("H2O2"), Moles(0.326375), Phase::Liquid);
    v.deposit(SpeciesId::new("water"), Moles(0.00001), Phase::Liquid);
    let before = v.contents.clone();
    HonestyEquilibrator.equilibrate(&mut v).expect("honesty");
    assert!(v.solution.is_none());
    assert_eq!((v.free_proton, v.free_hydroxide), (0.0, 0.0));
    assert_eq!(v.contents, before);
}

#[test]
fn external_browser_hook_is_not_asked_to_solve_peroxide_matrix() {
    let mut eq = PhreeqcEquilibrator::new().expect("solver");
    eq.set_hook(Box::new(|_, _| {
        panic!("an aqueous browser hook must not receive a peroxide-rich problem")
    }));
    let mut v = mixture(0.00001, 0.326375, 0.019062);
    let before = v.contents.clone();
    let events = eq.equilibrate(&mut v).expect("domain refusal");
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    assert!(v.solution.is_none());
    assert_eq!(v.contents, before);
}

//! Transactional bound-inventory regression tests.
use kerotakis_core::{delta::StateDelta, orchestrator::diff_vessels, *};

fn bound_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "bound dye");
    v.adsorbed.push(kerotakis_core::vessel::AdsorbedAmount {
        sorbent: SpeciesId::new("activated_charcoal"),
        sorbate: SpeciesId::new("methyl_orange"),
        moles: Moles(1.0),
    });
    v
}

fn change(delta: StateDelta, moles: f64) -> StateDelta {
    delta.with_adsorbed(
        SpeciesId::new("activated_charcoal"),
        SpeciesId::new("methyl_orange"),
        moles,
    )
}

#[test]
fn cumulative_overdraw_is_rejected_without_mutation() {
    let mut v = bound_vessel();
    let before = serde_json::to_value(&v).unwrap();
    let delta = change(change(StateDelta::new("test"), -0.6), -0.6);
    assert!(delta.commit(&mut v).is_err());
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
}

#[test]
fn nonfinite_bound_changes_are_rejected() {
    for amount in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(!change(StateDelta::new("test"), amount)
            .validate(&bound_vessel())
            .is_empty());
    }
}

#[test]
fn desorption_scales_bulk_and_bound_changes_together() {
    let mut v = bound_vessel();
    let delta = change(StateDelta::new("test"), -2.0).with_moles(
        SpeciesId::new("methyl_orange"),
        Phase::Aqueous,
        2.0,
    );
    let limited = delta.inventory_limited(&v).unwrap();
    assert!((limited.accepted_fraction - 0.5).abs() < 1e-12);
    limited.delta.commit_conserved(&mut v, 1e-10).unwrap();
    assert!(v.adsorbed.is_empty());
    assert!((v.moles_of(&SpeciesId::new("methyl_orange")).0 - 1.0).abs() < 1e-12);
}

#[test]
fn diff_carries_complete_desorption() {
    let mut before = bound_vessel();
    let mut after = before.clone();
    after.adsorbed.clear();
    after.deposit(SpeciesId::new("methyl_orange"), Moles(1.0), Phase::Aqueous);
    let delta = diff_vessels(&before, &after, "test");
    assert!(!delta.is_empty());
    delta.commit_conserved(&mut before, 1e-10).unwrap();
    assert!(before.adsorbed.is_empty());
    assert!((before.moles_of(&SpeciesId::new("methyl_orange")).0 - 1.0).abs() < 1e-12);
}

#[test]
fn unbalanced_binding_rolls_back() {
    let mut v = bound_vessel();
    let before = serde_json::to_value(&v).unwrap();
    assert!(change(StateDelta::new("test"), 1.0)
        .commit_conserved(&mut v, 1e-10)
        .is_err());
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
}

#[test]
fn later_deposit_cannot_hide_earlier_overdraw() {
    let v = bound_vessel();
    let delta = change(change(StateDelta::new("test"), -2.0), 2.0);
    assert!(!delta.validate(&v).is_empty());
}

//! Source-informed contracts for complete cloned solver proposals.
use kerotakis_core::{orchestrator::diff_vessels, vessel::Headspace, *};

fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    v
}
fn key(v: &Vessel) -> String {
    format!("{v:?}")
}
fn derived(v: &mut Vessel) {
    v.free_proton = 0.001;
    v.pending_co2_transfer_mol = 0.0001;
    v.excess_enthalpy_j = 17.0;
    v.aqueous_routing_said = Some("accepted solver route".into());
    v.honesty_said.push("accepted diagnosis".into());
    v.solution =
        Some(serde_json::from_value(serde_json::json!({"ph":3.0,"ionic_strength":0.01})).unwrap());
    v.resolved.valid = true;
    v.resolved.solution = v.solution.clone();
}
fn refuses(mut candidate: Vessel) {
    // Include a valid derived change, so malformed no-op candidates are not
    // mistaken for a test that did not exercise proposal application.
    candidate.honesty_said.push("trial must not leak".into());
    let mut actual = water();
    let before = key(&actual);
    assert!(diff_vessels(&actual, &candidate, "invalid")
        .commit(&mut actual)
        .is_err());
    assert_eq!(key(&actual), before);
}

#[test]
fn derived_only_proposal_commits_complete_state() {
    let mut actual = water();
    let mut after = actual.clone();
    derived(&mut after);
    let proposal = diff_vessels(&actual, &after, "derived");
    assert!(!proposal.is_empty());
    proposal.commit_conserved(&mut actual, 1e-10).unwrap();
    assert_eq!(key(&actual), key(&after));
}
#[test]
fn complete_snapshot_preserves_solver_boundary_state() {
    let mut actual = water();
    let mut after = actual.clone();
    after.headspace = Headspace::Sealed {
        volume: Liters(0.1),
    };
    after.pressure = Pascal(150_000.0);
    diff_vessels(&actual, &after, "boundary")
        .commit(&mut actual)
        .unwrap();
    assert_eq!(key(&actual), key(&after));
}
#[test]
fn tiny_addition_and_removal_survive_snapshot_application() {
    for amount in [1e-18, f64::from_bits(1)] {
        let mut actual = water();
        let mut after = actual.clone();
        after.deposit(SpeciesId::new("NaCl"), Moles(amount), Phase::Solid);
        diff_vessels(&actual, &after, "tiny-add")
            .commit(&mut actual)
            .unwrap();
        assert_eq!(key(&actual), key(&after));
        let mut gone = actual.clone();
        gone.contents
            .retain(|p| p.species != SpeciesId::new("NaCl"));
        diff_vessels(&actual, &gone, "tiny-remove")
            .commit(&mut actual)
            .unwrap();
        assert_eq!(key(&actual), key(&gone));
    }
}
#[test]
fn stale_material_change_refuses_without_overwriting() {
    let mut actual = water();
    let mut after = actual.clone();
    derived(&mut after);
    let proposal = diff_vessels(&actual, &after, "stale");
    actual.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);
    let before = key(&actual);
    assert!(proposal.commit(&mut actual).is_err());
    assert_eq!(key(&actual), before);
}
#[test]
fn stale_transient_narration_refuses_without_overwriting() {
    let mut actual = water();
    let mut after = actual.clone();
    derived(&mut after);
    let proposal = diff_vessels(&actual, &after, "stale");
    actual
        .honesty_said
        .push("intervening accepted notice".into());
    let before = key(&actual);
    assert!(proposal.commit(&mut actual).is_err());
    assert_eq!(key(&actual), before);
}
#[test]
fn edited_public_terms_refuse_instead_of_disagreeing_with_snapshot() {
    let mut actual = water();
    let mut after = actual.clone();
    derived(&mut after);
    let proposal = diff_vessels(&actual, &after, "edited").with_moles(
        SpeciesId::new("NaCl"),
        Phase::Solid,
        0.1,
    );
    let before = key(&actual);
    assert!(proposal.commit(&mut actual).is_err());
    assert_eq!(key(&actual), before);
}
#[test]
fn complete_snapshot_has_no_uniform_inventory_scaling() {
    let before = water();
    let mut after = before.clone();
    derived(&mut after);
    assert!(diff_vessels(&before, &after, "scale")
        .inventory_limited(&before)
        .is_err());
}
#[test]
fn unbalanced_snapshot_restores_all_fields() {
    let mut actual = water();
    let before = key(&actual);
    let mut after = actual.clone();
    derived(&mut after);
    after.deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Solid);
    assert!(diff_vessels(&actual, &after, "unbalanced")
        .commit_conserved(&mut actual, 1e-10)
        .is_err());
    assert_eq!(key(&actual), before);
}
#[test]
fn unchanged_snapshot_is_empty_and_preserves_state() {
    let mut actual = water();
    let before = key(&actual);
    let delta = diff_vessels(&actual, &actual, "unchanged");
    assert!(delta.is_empty());
    delta.commit(&mut actual).unwrap();
    assert_eq!(key(&actual), before);
}
#[test]
fn invalid_candidate_bulk_never_commits() {
    for amount in [f64::NAN, f64::INFINITY, -1.0] {
        let mut after = water();
        after.contents[0].moles = Moles(amount);
        refuses(after);
    }
}
#[test]
fn invalid_candidate_pressure_and_boundary_never_commit() {
    for n in [f64::NAN, f64::INFINITY, -1.0] {
        let mut after = water();
        after.pressure = Pascal(n);
        refuses(after);
        let mut after = water();
        after.headspace = Headspace::Sealed { volume: Liters(n) };
        refuses(after);
    }
}
#[test]
fn invalid_candidate_derived_solution_never_commits() {
    for n in [f64::NAN, f64::INFINITY] {
        for resolved in [false, true] {
            let mut after = water();
            derived(&mut after);
            if resolved {
                after.resolved.solution.as_mut().unwrap().ph = n;
            } else {
                after.solution.as_mut().unwrap().ph = n;
            }
            refuses(after);
        }
    }
}
#[test]
fn invalid_candidate_transient_amounts_never_commit() {
    for n in [f64::NAN, f64::INFINITY, -1.0] {
        let mut after = water();
        after.free_proton = n;
        refuses(after);
        let mut after = water();
        after.elapsed_seconds = n;
        refuses(after);
    }
}
#[test]
fn invalid_candidate_provenance_lots_never_commit() {
    for n in [f64::NAN, f64::INFINITY, -1.0] {
        let mut after = water();
        let mut lot: kerotakis_core::vessel::MaterialLot = serde_json::from_value(
            serde_json::json!({"species":"water","moles":1.0,"phase":"liquid","added_at":0.0}),
        )
        .unwrap();
        lot.moles = Moles(n);
        after.lots.push(lot);
        refuses(after);
    }
}
#[test]
fn candidate_identity_cannot_replace_another_vessel() {
    let mut after = water();
    after.id = VesselId(999);
    refuses(after);
}
#[test]
fn invalid_candidate_temperature_never_commits() {
    for n in [f64::NAN, f64::INFINITY, -1.0] {
        let mut after = water();
        after.temperature = Kelvin(n);
        refuses(after);
    }
}

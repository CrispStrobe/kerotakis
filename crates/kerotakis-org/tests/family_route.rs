//! BRD-020, the router over the real oracle: the shipped family pack,
//! matched through chematic, gated, and applied to a vessel ledger.
//!
//! `kerotakis-core` tests the router's logic over a fake oracle; these
//! tests are where the SMIRKS, the curated structure table and the
//! registry meet, and where a record that reads well but cannot name its
//! products would be found out.

use kerotakis_core::family::{FamilyRouter, StructureOracle};
use kerotakis_core::solve::{Applicability, Equilibrator};
use kerotakis_core::vessel::VesselId;
use kerotakis_core::{species, Event, Kelvin, Moles, Phase, SpeciesId, Vessel};
use kerotakis_org::family_oracle::{family_equilibrator, family_records, ChematicOracle};

fn vessel(portions: &[(&str, f64)], kelvin: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "v1");
    for (key, moles) in portions {
        let phase = species::lookup_key(key)
            .map(|d| d.standard_phase)
            .unwrap_or(Phase::Liquid);
        v.deposit(SpeciesId::new(key), Moles(*moles), phase);
    }
    v.temperature = Kelvin(kelvin);
    v
}

fn moles(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

fn mass_g(v: &Vessel) -> f64 {
    v.contents
        .iter()
        .map(|p| {
            p.moles.0
                * species::lookup(&p.species)
                    .expect("registry species")
                    .molar_mass
        })
        .sum()
}

fn fired(events: &[Event], family: &str) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::OrgReacted { name, .. } if name == family))
}

#[test]
fn the_shipped_pack_lints_and_names_its_families() {
    let records = family_records().expect("the pack lints clean");
    let ids: Vec<&str> = records.iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"fischer-esterification"), "{ids:?}");
    assert!(ids.contains(&"alkaline-ester-hydrolysis"), "{ids:?}");
    // Every exemplar substrate has a curated structure — otherwise the
    // record could never fire and would be documentation wearing a
    // record's clothes.
    for record in &records {
        for key in &record.substrates {
            assert!(
                ChematicOracle.groups_of(key).is_some(),
                "{}: substrate {key} has no curated structure",
                record.id
            );
        }
    }
}

#[test]
fn hot_acid_and_alcohol_over_sulfuric_acid_esterify_to_k() {
    let mut v = vessel(
        &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
        340.0,
    );
    let before = mass_g(&v);
    let mut router = family_equilibrator();
    assert!(router.applies(&v));
    let events = router.equilibrate(&mut v).expect("equilibrates");
    assert!(fired(&events, "fischer-esterification"), "{events:?}");
    // K = 4, equimolar: two thirds converted.
    // log K is carried to five figures in the pack, so K = 3.99995, not 4.
    let ester = moles(&v, "ethyl_acetate");
    assert!((ester - 0.2 / 3.0).abs() < 1e-5, "ester {ester}");
    assert!((moles(&v, "water") - 0.2 / 3.0).abs() < 1e-5);
    assert!((mass_g(&v) - before).abs() < 1e-9, "mass drifted");
    assert!(
        (moles(&v, "H2SO4") - 0.001).abs() < 1e-12,
        "catalyst consumed"
    );
}

#[test]
fn cold_or_uncatalysed_the_family_declines_in_words() {
    let router = family_equilibrator();
    let cold = vessel(
        &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
        298.15,
    );
    assert!(!router.applies(&cold));
    match router.capability(&cold).applicability {
        Applicability::NotApplicable { reason } => {
            assert!(reason.contains("temperature_k"), "{reason}")
        }
        other => panic!("expected a decline, got {other:?}"),
    }
    let bare = vessel(&[("CH3COOH", 0.1), ("ethanol", 0.1)], 340.0);
    let e = router.evaluate(&bare);
    assert_eq!(e.declined.len(), 1, "{:?}", e.declined);
    assert_eq!(e.declined[0].gate, "catalyst");
}

#[test]
fn water_pushes_the_esterification_back() {
    let mut dry = vessel(
        &[("CH3COOH", 0.1), ("ethanol", 0.1), ("H2SO4", 0.001)],
        340.0,
    );
    let mut wet = vessel(
        &[
            ("CH3COOH", 0.1),
            ("ethanol", 0.1),
            ("H2SO4", 0.001),
            ("water", 1.0),
        ],
        340.0,
    );
    family_equilibrator().equilibrate(&mut dry).unwrap();
    family_equilibrator().equilibrate(&mut wet).unwrap();
    assert!(
        moles(&wet, "ethyl_acetate") < 0.5 * moles(&dry, "ethyl_acetate"),
        "{} vs {}",
        moles(&wet, "ethyl_acetate"),
        moles(&dry, "ethyl_acetate")
    );
}

#[test]
fn warm_ester_and_hydroxide_in_water_saponify_and_keep_the_sodium() {
    let mut v = vessel(
        &[("water", 1.0), ("ethyl_acetate", 0.1), ("NaOH", 0.1)],
        340.0,
    );
    let before = mass_g(&v);
    let events = family_equilibrator().equilibrate(&mut v).unwrap();
    assert!(fired(&events, "alkaline-ester-hydrolysis"), "{events:?}");
    assert!(moles(&v, "ethyl_acetate") < 1e-12, "the ester is gone");
    assert!(moles(&v, "NaOH") < 1e-12, "the hydroxide is spent");
    assert!((moles(&v, "ethanol") - 0.1).abs() < 1e-9);
    assert!(
        (moles(&v, "CH3COO-") - 0.1).abs() < 1e-9,
        "the acetate anion"
    );
    assert!(
        (moles(&v, "Na+") - 0.1).abs() < 1e-9,
        "the spectator sodium"
    );
    // To the registry's own precision: the ions' molar masses are tabulated
    // apart from the salt's, so agreement is to the milligram here, not the
    // femtogram the esterification test can ask for.
    assert!(
        (mass_g(&v) - before).abs() < 1e-3,
        "mass drifted by {} g",
        mass_g(&v) - before
    );
    // Charge: Na+ and CH3COO- balance, exactly as NaOH did.
    assert!(
        v.solute_charge.abs() < 1e-9,
        "solute charge {}",
        v.solute_charge
    );
}

#[test]
fn hydroxide_the_aqueous_tail_measured_saponifies_too() {
    // After a solve the tail keeps a strong base as Na+ plus alkalinity,
    // with no hydroxide portion at all. The record names "OH-" and the
    // router backs it with the charge; the match runs through chematic's
    // [OH-] slot exactly as a poured NaOH portion does.
    let mut v = vessel(
        &[("water", 1.0), ("ethyl_acetate", 0.1), ("Na+", 0.1)],
        340.0,
    );
    v.solute_charge = 0.1;
    v.free_hydroxide = 0.1;
    let events = family_equilibrator().equilibrate(&mut v).unwrap();
    assert!(fired(&events, "alkaline-ester-hydrolysis"), "{events:?}");
    assert!(moles(&v, "ethyl_acetate") < 1e-12, "the ester is gone");
    assert!((moles(&v, "ethanol") - 0.1).abs() < 1e-9);
    assert!((moles(&v, "CH3COO-") - 0.1).abs() < 1e-9);
    assert!(
        (moles(&v, "Na+") - 0.1).abs() < 1e-9,
        "the sodium was already there"
    );
    assert!(
        moles(&v, "OH-") < 1e-12,
        "no hydroxide portion was conjured"
    );
    assert!(
        v.solute_charge.abs() < 1e-9,
        "the alkalinity is spent: {}",
        v.solute_charge
    );
}

#[test]
fn a_balanced_salt_solution_is_not_alkaline() {
    let mut v = vessel(
        &[
            ("water", 1.0),
            ("ethyl_acetate", 0.1),
            ("Na+", 0.1),
            ("Cl-", 0.1),
        ],
        340.0,
    );
    v.solute_charge = 0.0;
    let e = family_equilibrator().evaluate(&v);
    assert!(e.ready.is_empty(), "{:?}", e.ready);
    assert!(e.declined.is_empty(), "{:?}", e.declined);
}

#[test]
fn cold_saponification_declines_and_dry_ester_is_not_asked() {
    let router = family_equilibrator();
    let cold = vessel(
        &[("water", 1.0), ("ethyl_acetate", 0.1), ("NaOH", 0.1)],
        298.15,
    );
    let e = router.evaluate(&cold);
    assert!(e.ready.is_empty());
    assert!(e
        .declined
        .iter()
        .any(|d| d.family == "alkaline-ester-hydrolysis" && d.gate == "temperature_k"));
    // Ester and hydroxide without water: the medium gate declines.
    let dry = vessel(&[("ethyl_acetate", 0.1), ("NaOH", 0.1)], 340.0);
    let e = router.evaluate(&dry);
    assert!(
        e.declined.iter().any(|d| d.gate == "medium"),
        "{:?}",
        e.declined
    );
}

#[test]
fn methyl_acetate_is_a_registry_task_not_a_silent_drop() {
    let mut v = vessel(
        &[("CH3COOH", 0.1), ("methanol", 0.1), ("H2SO4", 0.001)],
        340.0,
    );
    let events = family_equilibrator().equilibrate(&mut v).unwrap();
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("cannot name")
        )),
        "{events:?}"
    );
    assert!(
        (moles(&v, "CH3COOH") - 0.1).abs() < 1e-12,
        "nothing was consumed"
    );
}

#[test]
fn pouring_order_does_not_change_the_answer() {
    let mut ab = vessel(
        &[("CH3COOH", 0.1), ("ethanol", 0.15), ("H2SO4", 0.001)],
        340.0,
    );
    let mut ba = vessel(
        &[("H2SO4", 0.001), ("ethanol", 0.15), ("CH3COOH", 0.1)],
        340.0,
    );
    family_equilibrator().equilibrate(&mut ab).unwrap();
    family_equilibrator().equilibrate(&mut ba).unwrap();
    for key in ["CH3COOH", "ethanol", "ethyl_acetate", "water"] {
        assert!((moles(&ab, key) - moles(&ba, key)).abs() < 1e-12, "{key}");
    }
}

#[test]
fn the_router_type_is_the_one_the_stack_boxes() {
    // A compile-time statement: the shipped router is a boxed equilibrator.
    let boxed: Box<dyn Equilibrator> = Box::new(family_equilibrator());
    assert_eq!(boxed.name(), "reaction-families");
    let _: FamilyRouter<ChematicOracle> = family_equilibrator();
}

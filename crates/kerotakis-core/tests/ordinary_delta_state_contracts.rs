//! Existing-API complete-state controls for ordinary, non-conserving commits.
use kerotakis_core::{
    compartment::ElectrodeState,
    delta::{StateDelta, ThermalDelta},
    vessel::Headspace,
    *,
};
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v
}
fn electrode() -> ElectrodeState {
    ElectrodeState {
        label: "anode".into(),
        material: "Pt".into(),
        surface_preparation: None,
        substrate_moles: None,
        area_m2: 0.001,
        roughness: 1.0,
        double_layer_capacitance_f_per_m2: None,
        interfacial_potential_v: None,
        interfacial_species: vec![],
        diagnostics: None,
        deposits: vec![],
    }
}
fn refuse(mut v: Vessel, delta: StateDelta) {
    v.honesty_said.push("existing diagnosis".into());
    let before = format!("{v:?}");
    assert!(delta.commit(&mut v).is_err());
    assert_eq!(
        format!("{v:?}"),
        before,
        "complete original state must survive"
    );
}
fn warm() -> StateDelta {
    StateDelta::new("ordinary-state-audit")
        .with_thermal(ThermalDelta::SetTemperature(Kelvin(310.0)))
}
#[test]
fn ordinary_commit_rejects_malformed_canonical_electrodes() {
    for duplicate in [false, true] {
        let mut v = water();
        let mut e = electrode();
        if duplicate {
            v.electrodes.push(e.clone());
        } else {
            e.area_m2 = 0.0;
        }
        v.electrodes.push(e);
        refuse(v, warm());
    }
}
#[test]
fn ordinary_commit_rejects_malformed_typed_crystal_ownership() {
    let mut v = water();
    let mut c = SolidSolution::aragonite_strontianite("crystal", Moles(0.1), Moles(0.1));
    c.components.pop();
    v.solid_solutions.push(c);
    refuse(v, warm());
}
#[test]
fn ordinary_commit_rejects_invalid_unrelated_derived_state() {
    for charge in [true, false] {
        let mut v = water();
        if charge {
            v.solute_charge = f64::NAN;
        } else {
            v.pending_co2_transfer_mol = f64::INFINITY;
        }
        refuse(v, warm());
    }
}
#[test]
fn ordinary_commit_rejects_invalid_headspace_domain() {
    let mut v = water();
    v.headspace = Headspace::Sealed {
        volume: Liters(0.0),
    };
    refuse(v, warm());
}
#[test]
fn ordinary_commit_rejects_pressure_overflow_created_during_apply() {
    let mut v = water();
    v.deposit(SpeciesId::new("H2"), Moles(1.0), Phase::Gas);
    v.headspace = Headspace::Sealed {
        volume: Liters(f64::from_bits(1)),
    };
    v.pressure = Pascal(101_325.0);
    refuse(v, warm());
}
#[test]
fn empty_ordinary_commit_cannot_certify_invalid_existing_state() {
    let mut v = water();
    v.nuclides
        .inventory
        .insert(nuclide::Nuclide::new("C", 14), -1.0);
    refuse(v, StateDelta::new("empty-state-audit"));
}
#[test]
fn valid_ordinary_matter_addition_and_withdrawal_remain_supported() {
    let mut v = water();
    StateDelta::new("external-addition")
        .with_moles(SpeciesId::new("H2"), Phase::Gas, 0.25)
        .commit(&mut v)
        .unwrap();
    assert_eq!(v.moles_of(&SpeciesId::new("H2")), Moles(0.25));
    StateDelta::new("external-withdrawal")
        .with_moles(SpeciesId::new("H2"), Phase::Gas, -0.25)
        .commit(&mut v)
        .unwrap();
    assert_eq!(v.moles_of(&SpeciesId::new("H2")), Moles(0.0));
}
#[test]
fn valid_ordinary_commit_preserves_trace_owners_and_narration() {
    let mut v = water();
    let key = nuclide::Nuclide::new("C", 14);
    v.nuclides.inventory.insert(key.clone(), f64::from_bits(1));
    v.electrodes.push(electrode());
    v.honesty_said.push("supported state".into());
    let mut expected = v.clone();
    expected.temperature = Kelvin(310.0);
    expected.refresh_pressure();
    warm().commit(&mut v).unwrap();
    assert_eq!(format!("{v:?}"), format!("{expected:?}"));
    assert_eq!(v.nuclides.inventory[&key].to_bits(), 1);
}

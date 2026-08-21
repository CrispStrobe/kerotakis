use kerotakis_core::*;

fn carbonate_crystal(calcium: f64, strontium: f64) -> SolidSolution {
    SolidSolution::aragonite_strontianite("carbonate crystal", Moles(calcium), Moles(strontium))
}

#[test]
fn mixed_crystal_owns_each_end_member_and_its_mass() {
    let crystal = carbonate_crystal(0.75, 0.25);

    assert!(crystal.has_valid_state());
    assert_eq!(crystal.total_moles(), Moles(1.0));
    assert_eq!(
        crystal.moles_of(SolidSolutionComponent::CalciumCarbonate),
        Moles(0.75)
    );
    assert_eq!(
        crystal.moles_of(SolidSolutionComponent::StrontiumCarbonate),
        Moles(0.25)
    );
    let expected = 0.75 * 100.087 + 0.25 * 147.628;
    assert!((crystal.mass().0 - expected).abs() < 1e-12);

    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.solid_solutions.push(crystal);
    assert!((vessel.mass().0 - expected).abs() < 1e-12);
}

#[test]
fn mixed_crystal_rejects_missing_duplicate_and_negative_components() {
    let mut crystal = carbonate_crystal(0.5, 0.5);
    crystal.components.pop();
    assert!(!crystal.has_valid_state());

    let mut crystal = carbonate_crystal(0.5, 0.5);
    crystal.components[1].component = SolidSolutionComponent::CalciumCarbonate;
    assert!(!crystal.has_valid_state());

    let mut crystal = carbonate_crystal(0.5, 0.5);
    crystal.components[1].moles = Moles(-1e-6);
    assert!(!crystal.has_valid_state());
}

#[test]
fn old_vessel_json_defaults_to_no_solid_solutions() {
    let json = r#"{
        "elapsed_seconds": 0.0,
        "id": 0,
        "label": "legacy beaker",
        "contents": [],
        "temperature": 298.15,
        "pressure": 101325.0,
        "thermal_mode": "adiabatic",
        "headspace": "open",
        "surfaces": [],
        "exchanges": [],
        "solute_charge": 0.0,
        "solution": null
    }"#;

    let vessel: Vessel = serde_json::from_str(json).expect("legacy vessel state");
    assert!(vessel.solid_solutions.is_empty());
}

#[test]
fn vessel_renderer_names_the_mixed_phase_and_end_members() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.solid_solutions.push(carbonate_crystal(0.003, 0.001));

    let rendered = render_vessel(&vessel, Register::LV2).join("\n");
    assert!(rendered.contains("carbonate crystal mixed crystal"));
    assert!(rendered.contains("CaCO3"));
    assert!(rendered.contains("SrCO3"));
}

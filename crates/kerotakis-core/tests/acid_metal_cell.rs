use kerotakis_core::{displacement, Moles, Phase, SpeciesId, Vessel, VesselId};

fn acidic_metal(id: usize, metal: &str, ph: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(id), "lemon half-cell");
    vessel.deposit(SpeciesId::new("water"), Moles(2.75), Phase::Liquid);
    vessel.deposit(SpeciesId::new(metal), Moles(0.01), Phase::Solid);
    vessel.solution = Some(kerotakis_core::vessel::SolutionInfo {
        ph,
        pe: None,
        redox: Vec::new(),
        ionic_strength: 0.02,
        species: Vec::new(),
        provenance: None,
    });
    vessel
}

#[test]
fn zinc_and_copper_in_acid_offer_a_bounded_open_circuit_estimate() {
    let zinc = acidic_metal(0, "Zn", 1.86);
    let copper = acidic_metal(1, "Cu", 1.86);
    let cell = displacement::acid_zinc_copper_cell(&zinc, &copper).expect("acid cell");
    assert!(cell.anode_is_first);
    assert!((0.63..0.68).contains(&cell.volts), "{} V", cell.volts);
    assert_eq!(cell.ph, 1.86);
}

#[test]
fn the_bounded_path_does_not_turn_arbitrary_metal_or_neutral_water_into_a_lemon_cell() {
    let zinc = acidic_metal(0, "Zn", 7.0);
    let copper = acidic_metal(1, "Cu", 7.0);
    assert!(displacement::acid_zinc_copper_cell(&zinc, &copper).is_none());
    let iron = acidic_metal(0, "Fe", 2.0);
    assert!(displacement::acid_zinc_copper_cell(&iron, &copper).is_none());
}

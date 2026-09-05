use kerotakis_core::kitchen_biology::{apple_browning, egg_osmosis, soap_scum, OsmosisDirection};

#[test]
fn egg_water_follows_the_osmotic_gradient_and_time_is_bounded() {
    let water = egg_osmosis(0.30, 0.0, 86_400.0).unwrap();
    let syrup = egg_osmosis(0.30, 1.0, 86_400.0).unwrap();
    assert_eq!(water.direction, OsmosisDirection::IntoObject);
    assert_eq!(syrup.direction, OsmosisDirection::OutOfObject);
    assert!(water.water_fraction > 0.0 && water.water_fraction <= 0.40);
    assert!(egg_osmosis(0.30, 0.0, 172_800.0).unwrap().water_fraction > water.water_fraction);
}

#[test]
fn apple_needs_oxygen_and_ascorbate_inhibits_browning() {
    let air = apple_browning(900.0, 0.21, 0.0).unwrap();
    let protected = apple_browning(900.0, 0.21, 2.0e-4).unwrap();
    assert!(air > 0.5);
    assert!(protected < air / 5.0);
    assert_eq!(apple_browning(900.0, 0.0, 0.0), Some(0.0));
}

#[test]
fn soap_scum_is_two_to_one_and_limiting_reagent_bounded() {
    let ion_limited = soap_scum(0.001, 0.010).unwrap();
    assert!((ion_limited.soap_bound_moles - 0.002).abs() < 1e-12);
    assert!((ion_limited.divalent_ion_bound_moles - 0.001).abs() < 1e-12);
    let soap_limited = soap_scum(0.010, 0.002).unwrap();
    assert!((soap_limited.divalent_ion_bound_moles - 0.001).abs() < 1e-12);
    assert!(soap_limited.aggregate_mass_g > 0.0);
}

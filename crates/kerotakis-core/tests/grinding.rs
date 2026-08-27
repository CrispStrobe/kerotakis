use kerotakis_core::script::parse_op;
use kerotakis_core::*;

#[test]
fn grinding_persists_particle_size_and_computes_surface_area() {
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 NaCl 1g").unwrap().unwrap())
        .unwrap();
    let events = bench
        .step(parse_op("grind v1 NaCl 50um").unwrap().unwrap())
        .unwrap();

    let lot = bench.vessels[0]
        .lots
        .iter()
        .find(|lot| lot.species.0 == "NaCl")
        .unwrap();
    assert_eq!(lot.particle_size_um, Some(50.0));

    let Event::Ground {
        diameter_um,
        solid_moles,
        surface_area_m2,
        rate_coupled,
        ..
    } = events
        .iter()
        .find(|event| matches!(event, Event::Ground { .. }))
        .unwrap()
    else {
        unreachable!()
    };
    assert_eq!(*diameter_um, 50.0);
    assert!(solid_moles.0 > 0.017 && solid_moles.0 < 0.018);
    // 1 g / 2.17 g mL⁻¹, spherical particles: A = 6V/d ≈ 0.0553 m².
    assert!((*surface_area_m2 - 0.0553).abs() < 0.0002);
    assert!(!rate_coupled);
}

#[test]
fn grinding_requires_the_requested_solid() {
    let mut bench = Bench::new();
    bench
        .step(parse_op("add v1 water 10mL").unwrap().unwrap())
        .unwrap();
    let error = bench
        .step(parse_op("grind v1 NaCl 50um").unwrap().unwrap())
        .unwrap_err();
    assert!(matches!(error, BenchError::SolidNotPresent { .. }));
}

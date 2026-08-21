//! Typed surface-interface bookkeeping that does not require a chemistry engine.

use kerotakis_core::*;

fn hfo() -> SurfaceSites {
    SurfaceSites {
        label: "iron oxide grains".to_string(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.09),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(5e-6),
        weak_capacity: Moles(2e-4),
        occupancy: Vec::new(),
        water_release: Moles(0.0),
    }
}

#[test]
fn old_vessels_deserialise_without_interfaces() {
    let json = r#"{
        "elapsed_seconds": 0.0,
        "id": 0,
        "label": "beaker",
        "contents": [],
        "temperature": 298.15,
        "pressure": 101325.0,
        "thermal_mode": "adiabatic",
        "headspace": { "boundary": "open" },
        "solute_charge": 0.0,
        "solution": null
    }"#;
    let vessel: Vessel = serde_json::from_str(json).expect("old vessel JSON remains readable");
    assert!(vessel.surfaces.is_empty());
    assert!(vessel.exchanges.is_empty());
}

#[test]
fn surface_capacity_and_occupancy_are_separate_ledgers() {
    let mut surface = hfo();
    surface.occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Strong,
        sorbate: SurfaceSorbate::Zinc,
        moles: Moles(2e-6),
    });
    surface.occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Weak,
        sorbate: SurfaceSorbate::Sulfate,
        moles: Moles(1e-5),
    });

    assert_eq!(surface.capacity(SurfaceSiteKind::Strong), Moles(5e-6));
    assert_eq!(surface.occupied(SurfaceSiteKind::Strong), Moles(2e-6));
    assert_eq!(surface.occupied(SurfaceSiteKind::Weak), Moles(1e-5));
    assert_eq!(surface.bound(SurfaceSorbate::Zinc), Moles(2e-6));
    assert_eq!(surface.bound(SurfaceSorbate::Sulfate), Moles(1e-5));
    assert!(surface.has_valid_capacity());

    surface.occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Strong,
        sorbate: SurfaceSorbate::Zinc,
        moles: Moles(4e-6),
    });
    assert!(!surface.has_valid_capacity());
}

#[test]
fn vessel_mass_includes_the_interface_and_its_bound_sorbate() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    let mut surface = hfo();
    surface.occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Strong,
        sorbate: SurfaceSorbate::Zinc,
        moles: Moles(1e-6),
    });
    surface.occupancy.push(SurfaceOccupancy {
        site: SurfaceSiteKind::Weak,
        sorbate: SurfaceSorbate::Sulfate,
        moles: Moles(2e-6),
    });
    surface.water_release = Moles(1e-6);
    vessel.surfaces.push(surface);

    let zinc_mass = species::lookup_key("Zn+2").unwrap().molar_mass * 1e-6;
    let sulfate_mass = species::lookup_key("SO4-2").unwrap().molar_mass * 2e-6;
    let released_water_mass = species::lookup_key("water").unwrap().molar_mass * 1e-6;
    assert!(
        (vessel.mass().0 - (0.09 + zinc_mass + sulfate_mass - released_water_mass)).abs() < 1e-12
    );
    assert!(
        !vessel.is_empty(),
        "a physical oxide interface is vessel contents"
    );
}

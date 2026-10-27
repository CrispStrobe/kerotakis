use kerotakis_core::nonaqueous::{water_fraction_among_solvents, AQUEOUS_WATER_FRACTION_FLOOR};
use kerotakis_core::ops::NotModelledCause;
use kerotakis_core::solve::{Equilibrator, HonestyEquilibrator};
use kerotakis_core::{Event, Phase, SpeciesId, Vessel, VesselId};

#[test]
fn acetic_acid_counts_as_solvent_when_routing_low_water_organic_mixtures() {
    let mut vessel = Vessel::new(VesselId(0), "mixed organic test");
    vessel.deposit(
        SpeciesId::new("water"),
        kerotakis_core::Moles(0.009),
        Phase::Liquid,
    );
    vessel.deposit(
        SpeciesId::new("ethanol"),
        kerotakis_core::Moles(0.006),
        Phase::Liquid,
    );
    vessel.deposit(
        SpeciesId::new("ethyl_acetate"),
        kerotakis_core::Moles(0.001),
        Phase::Liquid,
    );
    vessel.deposit(
        SpeciesId::new("CH3COOH"),
        kerotakis_core::Moles(0.018),
        Phase::Liquid,
    );

    let fraction = water_fraction_among_solvents(&vessel).expect("mixed solvent");
    assert!((fraction - 9.0 / 34.0).abs() < 1e-12, "{fraction}");
    assert!(fraction < AQUEOUS_WATER_FRACTION_FLOOR);

    let events = HonestyEquilibrator
        .equilibrate(&mut vessel)
        .expect("honesty pass");
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled {
                cause: NotModelledCause::ModelBoundary,
                what,
                ..
            } if what.contains("water is 26%")
        )),
        "{events:?}"
    );
}

#[test]
fn decomposition_water_does_not_make_bulk_peroxide_an_aqueous_medium() {
    let mut vessel = Vessel::new(VesselId(0), "peroxide matrix");
    vessel.deposit(
        SpeciesId::new("H2O2"),
        kerotakis_core::Moles(0.326375),
        Phase::Liquid,
    );
    vessel.deposit(
        SpeciesId::new("water"),
        kerotakis_core::Moles(0.00001),
        Phase::Liquid,
    );
    let before = vessel.contents.clone();
    let fraction = water_fraction_among_solvents(&vessel).expect("peroxide is a liquid matrix");
    assert!((fraction - 0.00001 / 0.326385).abs() < 1e-12);
    let events = HonestyEquilibrator
        .equilibrate(&mut vessel)
        .expect("honesty");
    assert!(events.iter().any(|e| matches!(e, Event::NotYetModeled { reason: Some(p), .. } if p.key == "not-modeled.peroxide-rich-medium")), "{events:?}");
    assert_eq!(vessel.contents, before);
    assert!(vessel.solution.is_none());
    assert!(
        !kerotakis_core::nonaqueous::KNOWN_SOLVENTS.contains(&"H2O2"),
        "an aqueous-domain guard must not invent organic-solubility data"
    );
}

// Lane 9 (#839 generalised): every registry liquid has a declared medium
// class, and admission follows the class rather than a peroxide special case.
mod registry_liquid_media {
    use kerotakis_core::nonaqueous::{
        liquid_medium, non_water_matrix_boundary, water_fraction_among_solvents, LiquidMedium,
        AQUEOUS_WATER_FRACTION_FLOOR,
    };
    use kerotakis_core::{Event, Moles, Phase, SpeciesId, Vessel, VesselId};

    fn vessel(items: &[(&str, f64)]) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "medium admission");
        for (k, n) in items {
            v.deposit(SpeciesId::new(k), Moles(*n), Phase::Liquid);
        }
        v
    }

    #[test]
    fn every_registry_liquid_has_a_declared_medium_class() {
        let unclassified: Vec<&str> = kerotakis_core::species::all_species()
            .into_iter()
            .filter(|s| s.standard_phase == Phase::Liquid)
            .map(|s| s.key)
            .filter(|k| liquid_medium(k).is_none())
            .collect();
        assert!(
            unclassified.is_empty(),
            "classify these registry liquids in REGISTRY_LIQUID_MEDIA: {unclassified:?}"
        );
    }

    #[test]
    fn concentrated_sulfuric_acid_refuses_with_a_typed_reason() {
        let mut v = vessel(&[("H2SO4", 1.0), ("water", 0.1)]);
        let before = v.contents.clone();
        let event = non_water_matrix_boundary(&mut v).expect("acid-majority liquid");
        assert!(
            matches!(&event, Event::NotYetModeled { reason: Some(p), .. } if p.key == "not-modeled.concentrated-acid-medium"),
            "{event:?}"
        );
        assert_eq!(v.contents, before);
        assert!(v.solution.is_none());
    }

    #[test]
    fn dilute_acid_and_the_exact_floor_remain_admitted() {
        let mut dilute = vessel(&[("H2SO4", 0.05), ("water", 5.0)]);
        assert!(water_fraction_among_solvents(&dilute).unwrap() >= AQUEOUS_WATER_FRACTION_FLOOR);
        assert!(non_water_matrix_boundary(&mut dilute).is_none());
        let mut floor = vessel(&[("H3PO4", 1.0), ("water", 1.0)]);
        assert_eq!(water_fraction_among_solvents(&floor), Some(0.5));
        assert!(non_water_matrix_boundary(&mut floor).is_none());
    }

    #[test]
    fn reagent_bottle_solutes_are_not_a_competing_matrix() {
        for key in ["HCl", "NH3", "NaOCl", "HI"] {
            assert_eq!(liquid_medium(key), Some(LiquidMedium::AqueousReagentSolute));
            let v = vessel(&[(key, 1.0), ("water", 0.1)]);
            assert_eq!(water_fraction_among_solvents(&v), None, "{key}");
        }
    }

    #[test]
    fn other_organic_liquids_count_like_the_known_solvents() {
        for key in ["methanol", "isopropanol", "dodecane"] {
            let v = vessel(&[(key, 2.0), ("water", 1.0)]);
            let x = water_fraction_among_solvents(&v).expect(key);
            assert!((x - 1.0 / 3.0).abs() < 1e-12, "{key}: {x}");
        }
        // Organic majority keeps its own honesty route, not this boundary.
        let mut v = vessel(&[("methanol", 2.0), ("water", 1.0)]);
        assert!(non_water_matrix_boundary(&mut v).is_none());
    }
}

use kerotakis_core::nonaqueous::{water_fraction_among_solvents, AQUEOUS_WATER_FRACTION_FLOOR};
use kerotakis_core::solve::{Equilibrator, HonestyEquilibrator};
use kerotakis_core::{Event, NotModelledCause, Phase, SpeciesId, Vessel, VesselId};

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

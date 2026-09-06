//! What the beaker is doing an hour after the flame went out.
//!
//! `add v1 ethanol 10mL; ignite v1` used to leave an EMPTY beaker sitting
//! at 2496 °C for the rest of the session, because the bench had no room
//! for it to stand in. It has one now (`clock::AmbientClock`), and the
//! only thermal mass an empty beaker has is its own glass:
//!
//! ```text
//! C   = 100 g * 0.83 J/(g K)                  = 83 J/K
//! hA  = 7.0 W/(m^2 K) * 0.02474 m^2           = 0.17318 W/K
//! tau = 83 / 0.17318                          = 479 s
//! 1 h = 3600 / 479                            = 7.5 time constants
//! T   = 298.15 + 2471.3 * e^-7.51             = 299.5 K = 26 °C
//! ```
//!
//! Radiation is not modelled, and at 2496 °C it would dominate — so this
//! is the SLOW answer and a real beaker would be cooler still.

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

#[test]
fn the_beaker_the_flame_left_behind_is_cool_an_hour_later() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("ethanol"),
                moles: Moles(0.010),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add ethanol");
    bench
        .step_with(
            Operator::Ignite { vessel: v },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("ignite ethanol");

    let flame = bench.vessel(v).unwrap().temperature.0;
    assert!(
        flame > 2_000.0,
        "the flame temperature should still be the adiabatic one, not {flame} K"
    );
    assert!(
        bench.vessel(v).unwrap().contents.is_empty(),
        "everything burned and the products left: {:?}",
        bench.vessel(v).unwrap().contents
    );

    let events = bench
        .step_with(
            Operator::Wait { seconds: 3600.0 },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("wait an hour");

    let after = bench.vessel(v).unwrap().temperature.0;
    let tau = kerotakis_core::clock::wall_heat_capacity_j_per_k("beaker")
        / kerotakis_core::clock::ambient_conductance_w_per_k("beaker");
    let newton = 298.15 + (flame - 298.15) * (-3600.0 / tau).exp();
    assert!(
        (after - newton).abs() < 1.0,
        "an hour later the beaker is at {after} K; Newton's law on the glass alone \
         (tau = {tau} s) says {newton} K"
    );
    assert!(
        after < 373.15,
        "an empty beaker cannot still be at {after} K an hour after the flame"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TemperatureChanged { .. })),
        "and the bench has to SAY so: {events:?}"
    );
}

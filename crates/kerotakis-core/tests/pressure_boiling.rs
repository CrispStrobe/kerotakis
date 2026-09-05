//! BRD-032, first slice: water boils where the pressure says it does.
//!
//! Until this landed, the bench boiled water at 100 °C in a vacuum flask and
//! at 100 °C in a pressure cooker. `states::transitions` took a molality and
//! nothing else, so `regulate v1 50000Pa` changed the number on the pressure
//! gauge and nothing else in the vessel — which is the shape of defect
//! BREADTH warns about, a quantity claimed to depend on X that does not move
//! when X does. The corpus asks about it directly in th-019 and th-020.
//!
//! What is routed here is narrow and deliberate. The boiling temperature
//! comes from inverting the BRD-031 pack's cleared saturation-pressure
//! correlation for the solvent, found by InChIKey; the correlation supplies
//! the *shift* and the registry's reviewed normal boiling point supplies the
//! anchor, so one atmosphere is unchanged to the last bit. Outside the
//! pressure window the cleared fit spans, the bench keeps the curated answer
//! and says so, rather than extrapolating a local fit into a region it has
//! no data for.

use kerotakis_core::states::BoilingRoute;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn step(bench: &mut Bench, op: Operator) -> Vec<Event> {
    bench
        .step_with(op, &mut stack(), &PermissiveScreen)
        .expect("operator")
}

fn water_bench(moles: f64) -> Bench {
    let mut bench = Bench::new();
    step(
        &mut bench,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(moles),
            at: None,
        },
    );
    bench
}

fn boiled_at_k(events: &[Event]) -> Option<f64> {
    events.iter().find_map(|event| match event {
        Event::StateChanged {
            from: Phase::Liquid,
            to: Phase::Gas,
            at,
            ..
        } => Some(at.0),
        _ => None,
    })
}

fn routing(events: &[Event]) -> Option<(BoilingRoute, f64, f64)> {
    events.iter().find_map(|event| match event {
        Event::BoilingPointRouted {
            route,
            boiling,
            shifted_by,
            ..
        } => Some((*route, boiling.0, *shifted_by)),
        _ => None,
    })
}

#[test]
fn an_open_beaker_is_untouched_by_this_change() {
    // The anchor exists so that this is exactly true rather than nearly
    // true. Stull's fit reproduces water's normal boiling point to 0.003 K,
    // and a bench that moved every open vessel by that much to gain nothing
    // would be a worse bench.
    let mut bench = water_bench(5.55);
    let events = step(
        &mut bench,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(60_000.0),
        },
    );
    let boiled = boiled_at_k(&events).expect("60 kJ boils 100 mL of water");
    assert!(
        (boiled - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-12,
        "an open beaker boiled at {boiled} K"
    );
    assert!(
        routing(&events).is_none(),
        "an open beaker must not emit a routing event at all"
    );
}

#[test]
fn lowering_the_pressure_makes_water_boil_sooner() {
    // th-019, and the whole point of the slice. 50 kPa is roughly the
    // pressure at 5 500 m, where a kettle really does stop making tea.
    let mut bench = water_bench(5.55);
    step(
        &mut bench,
        Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(50_000.0),
            initial_volume: Liters(1.0),
        },
    );
    let events = step(
        &mut bench,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(30_000.0),
        },
    );
    let boiled = boiled_at_k(&events).expect("30 kJ passes the reduced boiling point");
    let celsius = boiled - 273.15;
    // The steam tables put water's boiling point at 50 kPa at 81.3 °C.
    assert!(
        (celsius - 81.3).abs() < 0.5,
        "50 kPa boiled at {celsius:.3} °C, not near 81.3 °C"
    );
    let (route, reported, shift) = routing(&events).expect("a routing event");
    assert_eq!(route, BoilingRoute::ClearedCorrelation);
    assert!((reported - boiled).abs() < 1e-12);
    assert!(shift < -18.0 && shift > -19.0, "shift was {shift} K");
}

#[test]
fn a_pressure_above_the_cleared_window_is_named_and_not_extrapolated() {
    // th-020's shape. Water's shipped Antoine fit stops at 100 °C, which is
    // to say at one atmosphere, so a sealed vessel whose air has warmed past
    // that is already outside it. The honest answer is the curated boiling
    // point plus a statement that the pressure was not modelled — and that
    // statement is exactly what makes this different from the silence the
    // bench used to keep.
    let mut bench = water_bench(5.55);
    step(
        &mut bench,
        Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(200_000.0),
            initial_volume: Liters(1.0),
        },
    );
    let events = step(
        &mut bench,
        Operator::Heat {
            vessel: VesselId(0),
            energy: Joules(60_000.0),
        },
    );
    let boiled = boiled_at_k(&events).expect("60 kJ still passes 100 °C");
    assert!(
        (boiled - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-12,
        "an unrouted vessel must keep the curated boiling point, got {boiled} K"
    );
    let (route, _, shift) = routing(&events).expect("an unrouted vessel still says so");
    assert_eq!(route, BoilingRoute::PressureOutsideClearedWindow);
    assert!(!route.routed());
    assert!(shift.abs() < 1e-12, "an unrouted vessel shifts nothing");
}

#[test]
fn the_pressure_shift_does_not_depend_on_how_much_water_there_is() {
    // Scale invariance. The boiling point is an intensive property, so
    // doubling the vessel's contents and the energy must land on the same
    // temperature; if the shift ever picked up an amount it would fail here.
    fn boiling_at_60_kpa(moles: f64, joules: f64) -> f64 {
        let mut bench = water_bench(moles);
        step(
            &mut bench,
            Operator::Regulate {
                vessel: VesselId(0),
                pressure: Pascal(60_000.0),
                initial_volume: Liters(1.0),
            },
        );
        let events = step(
            &mut bench,
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
            },
        );
        let (route, boiling, _) = routing(&events).expect("a routing event");
        assert_eq!(route, BoilingRoute::ClearedCorrelation);
        boiling
    }

    let small = boiling_at_60_kpa(5.55, 30_000.0);
    let large = boiling_at_60_kpa(11.1, 60_000.0);
    assert!(
        (small - large).abs() < 1e-12,
        "twice the water boiled at {large} K rather than {small} K"
    );
    // 60 kPa boils water at 86.0 °C by the steam tables.
    assert!(
        (small - 273.15 - 86.0).abs() < 0.5,
        "60 kPa boiled at {} °C",
        small - 273.15
    );
}

#[test]
fn pressure_and_dissolved_particles_are_reported_as_two_different_shifts() {
    // A vessel under vacuum has a lower boiling point without anything being
    // dissolved in it. Folding the two into one number would make the
    // colligative sentence — "higher than pure water, because of what is
    // dissolved in it" — say the opposite of what happened.
    let (transitions, route) = kerotakis_core::states::transitions_at(0.5, 50.0);
    assert_eq!(route, BoilingRoute::ClearedCorrelation);
    assert!(
        transitions.boiling_elevation() > 0.2,
        "salt still raises the boiling point: {}",
        transitions.boiling_elevation()
    );
    assert!(
        transitions.boiling_pressure_shift() < -18.0,
        "the vacuum still lowers it: {}",
        transitions.boiling_pressure_shift()
    );
    assert!(
        transitions.boiling_k < kerotakis_core::states::WATER_BOILING_K,
        "and the vacuum wins: {}",
        transitions.boiling_k
    );
}

#[test]
fn a_vessel_with_no_usable_pressure_keeps_the_curated_answer() {
    let (transitions, route) = kerotakis_core::states::transitions_at(0.0, 0.0);
    assert_eq!(route, BoilingRoute::NoUsablePressure);
    assert!(!route.routed());
    assert!((transitions.boiling_k - kerotakis_core::states::WATER_BOILING_K).abs() < 1e-12);
    let (_, nan_route) = kerotakis_core::states::transitions_at(0.0, f64::NAN);
    assert_eq!(nan_route, BoilingRoute::NoUsablePressure);
}

#[test]
fn boiling_rises_monotonically_across_the_whole_routed_window() {
    // BRD-032's stated acceptance, checked on the surface the bench uses
    // rather than only on the pack API underneath it.
    let mut previous = f64::NEG_INFINITY;
    for tick in 0..=40 {
        let pressure = 1.0 + f64::from(tick) * 2.5;
        let (transitions, route) = kerotakis_core::states::transitions_at(0.0, pressure);
        if !route.routed() {
            continue;
        }
        assert!(
            transitions.boiling_k > previous,
            "{pressure} kPa gave {} K, not above {previous} K",
            transitions.boiling_k
        );
        previous = transitions.boiling_k;
    }
    assert!(
        previous.is_finite(),
        "no pressure in the sweep routed at all"
    );
}

#[test]
fn the_solvent_reaches_its_parameters_through_the_registry_inchikey() {
    // The seam, from the runtime side: the pack row the boiling route uses
    // is the one whose InChIKey the registry's water carries.
    let row = kerotakis_core::states::solvent_row().expect("water has a pack row");
    let data = species::lookup(&SpeciesId::new("water")).expect("registry water");
    assert_eq!(row.identity.inchikey, data.inchikey);
    assert!(row.saturation_model().is_some());
}

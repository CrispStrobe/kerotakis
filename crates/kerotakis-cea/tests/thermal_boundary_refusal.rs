//! Finite and swept inventories must never enter the open-room CEA ledger.
use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::{
    ops::NotModelledCause, vessel::Headspace, Equilibrator, Event, Kelvin, Liters, Moles, Pascal,
    Phase, Portion, SpeciesId, Vessel, VesselId,
};

fn charge(fuel: &str, boundary: Headspace) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.temperature = Kelvin(1200.0);
    vessel.headspace = boundary;
    vessel.contents = vec![
        Portion {
            species: SpeciesId::new(fuel),
            moles: Moles(0.001),
            phase: Phase::Solid,
        },
        Portion {
            species: SpeciesId::new("O2"),
            moles: Moles(0.01),
            phase: Phase::Gas,
        },
        Portion {
            species: SpeciesId::new("N2"),
            moles: Moles(0.02),
            phase: Phase::Gas,
        },
    ];
    vessel
}

#[test]
fn unsupported_boundaries_keep_the_entire_charge_and_report_the_gap() {
    for fuel in ["C", "Mg"] {
        for boundary in [
            Headspace::Sealed {
                volume: Liters(1.0),
            },
            Headspace::PressureControlled {
                pressure: Pascal(101325.0),
                volume: Liters(1.0),
            },
            Headspace::Swept {
                pressure: Pascal(10000.0),
            },
        ] {
            let mut vessel = charge(fuel, boundary);
            let before = serde_json::to_value(&vessel).unwrap();
            let mut solver = ThermalEquilibrator;
            assert!(
                solver.applies(&vessel),
                "the route must report its boundary gap"
            );
            let events = solver.equilibrate(&mut vessel).unwrap();
            assert_eq!(
                serde_json::to_value(&vessel).unwrap(),
                before,
                "fuel, oxygen, nitrogen, energy and boundary must be unchanged"
            );
            assert!(
                matches!(
                    events.as_slice(),
                    [Event::NotYetModeled {
                        cause: NotModelledCause::BoundaryMismatch,
                        ..
                    }]
                ),
                "no combustion, exhaust or thermal-equilibrium success: {events:?}"
            );
        }
    }
}

#[test]
fn open_magnesium_still_uses_the_supported_thermal_route() {
    let mut vessel = charge("Mg", Headspace::Open);
    let mut solver = ThermalEquilibrator;
    assert!(solver.applies(&vessel));
    let events = solver.equilibrate(&mut vessel).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::ThermalEquilibrium { .. })));
    assert!(vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "MgO" && p.moles.0 > 0.0));
}

#[test]
fn inert_sealed_gas_still_stands_down_for_pressure_settlement() {
    let mut vessel = charge(
        "C",
        Headspace::Sealed {
            volume: Liters(1.0),
        },
    );
    vessel.contents.retain(|p| p.phase == Phase::Gas);
    let solver = ThermalEquilibrator;
    assert!(!solver.applies(&vessel));
}

/// Lane 6 (refuse, not clamp): above the route's 6000 K domain the charge is
/// refused with a typed reason before any mutation, instead of being booked
/// at a clamped 6000 K.
#[test]
fn vessel_outside_the_route_temperature_domain_is_refused_not_clamped() {
    let mut vessel = charge("C", Headspace::Open);
    vessel.temperature = Kelvin(7000.0);
    let before = serde_json::to_value(&vessel).unwrap();
    let mut solver = ThermalEquilibrator;
    assert!(solver.applies(&vessel));
    let events = solver.equilibrate(&mut vessel).unwrap();
    assert_eq!(serde_json::to_value(&vessel).unwrap(), before);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled {
                cause: NotModelledCause::ModelBoundary,
                reason: Some(p),
                ..
            } if p.key == "not-modeled.cea-temperature-range"
        )),
        "{events:?}"
    );
}

//! An open-room CEA solve cannot silently change a different boundary.
use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::{ops::NotModelledCause, *};

fn vessel(boundary: Headspace, oxygen: bool) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "flask");
    v.headspace = boundary;
    v.temperature = Kelvin(1200.0);
    v.deposit(SpeciesId::new("methane"), Moles(0.01), Phase::Gas);
    if oxygen {
        v.deposit(SpeciesId::new("O2"), Moles(0.03), Phase::Gas);
    }
    v.deposit(SpeciesId::new("N2"), Moles(0.02), Phase::Gas);
    // The finite gas-only slice is supported; a condensed metal remains an
    // explicitly unsupported multiphase energy boundary.
    v.deposit(SpeciesId::new("Mg"), Moles(0.001), Phase::Solid);
    v.refresh_pressure();
    v
}
fn refuses_without_changes(v: &mut Vessel) {
    let state = format!("{v:?}");
    let ledger = ledger::ConservedLedger::from_vessel(v);
    let events = ThermalEquilibrator.equilibrate(v).unwrap();
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::NotYetModeled {
                cause: NotModelledCause::BoundaryMismatch,
                ..
            }
        )),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|event| matches!(
            event,
            Event::GasEvolved { .. }
                | Event::GasAbsorbed { .. }
                | Event::GasContained { .. }
                | Event::Consumed { .. }
                | Event::ThermalEquilibrium { .. }
        )),
        "{events:?}"
    );
    assert_eq!(format!("{v:?}"), state);
    assert!(ledger
        .check_against(&ledger::ConservedLedger::from_vessel(v), 1e-12, 1e-15)
        .is_empty());
}
#[test]
fn finite_headspace_does_not_import_oxygen_or_vent_matter() {
    for volume in [0.01, 0.1, 1.0] {
        for oxygen in [false, true] {
            let mut v = vessel(
                Headspace::Sealed {
                    volume: Liters(volume),
                },
                oxygen,
            );
            refuses_without_changes(&mut v);
            let expected = v.gas_moles().0 * 8314.462618 * v.temperature.0 / volume;
            assert!((v.pressure.0 - expected).abs() < expected * 1e-12);
        }
    }
}
#[test]
fn finite_pressure_controller_preserves_inventory_and_volume_when_route_declines() {
    let mut v = vessel(
        Headspace::PressureControlled {
            pressure: Pascal(200_000.0),
            volume: Liters(1.0),
        },
        true,
    );
    refuses_without_changes(&mut v);
    assert_eq!(v.pressure, Pascal(200_000.0));
}
#[test]
fn nitrogen_sweep_never_borrows_room_oxygen_to_burn_fuel() {
    let mut v = vessel(
        Headspace::Swept {
            pressure: Pascal::ATMOSPHERIC,
        },
        false,
    );
    refuses_without_changes(&mut v);
    assert_eq!(v.moles_of(&SpeciesId::new("methane")), Moles(0.01));
    assert_eq!(v.moles_of(&SpeciesId::new("O2")), Moles(0.0));
}

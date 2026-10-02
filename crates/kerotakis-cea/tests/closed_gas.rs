//! Independent constraints for the finite ideal-gas HP/UV slice.
use kerotakis_cea::{closed, db, ThermalEquilibrator, R};
use kerotakis_core::*;
use std::collections::BTreeMap;

fn gas_energy(v: &Vessel, internal: bool) -> f64 {
    v.contents
        .iter()
        .map(|p| {
            let row = species::lookup(&p.species).unwrap();
            let formula = stoich::parse_formula(row.formula).unwrap();
            let gas = db()
                .species
                .values()
                .find(|s| s.is_gas() && s.composition == formula.counts)
                .unwrap();
            p.moles.0
                * (gas.h(v.temperature.0).unwrap()
                    - if internal { R * v.temperature.0 } else { 0.0 })
        })
        .sum()
}

#[test]
fn pure_nitrogen_matches_analytic_temperature_pressure_and_energy() {
    let n = 0.03;
    let pool = [db().get("N2").unwrap()];
    let budget = BTreeMap::from([("N".to_string(), 2.0 * n)]);
    for t in [500.0, 900.0, 1700.0] {
        for volume in [0.2, 1.0, 3.0] {
            let h = n * pool[0].h(t).unwrap();
            let u = h - n * R * t;
            let eq = closed::equilibrate_uv(&budget, &pool, u, volume).unwrap();
            assert!((eq.temperature - t).abs() < 1e-4);
            assert!((eq.gas_moles - n).abs() < 1e-10);
            let pressure = n * R * t / (100.0 * volume);
            assert!((eq.pressure_bar - pressure).abs() < pressure * 2e-8);
            assert!((eq.enthalpy - eq.gas_moles * R * eq.temperature - u).abs() < 1e-4);
            let hp = closed::equilibrate_hp_verified(&budget, &pool, h, pressure).unwrap();
            assert!((hp.temperature - t).abs() < 1e-4);
            assert!((hp.enthalpy - h).abs() < 1e-4);
        }
    }
}

fn reactive(boundary: Headspace) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "finite flame");
    v.headspace = boundary;
    v.temperature = Kelvin(1200.0);
    v.deposit(SpeciesId::new("H2"), Moles(0.02), Phase::Gas);
    v.deposit(SpeciesId::new("O2"), Moles(0.015), Phase::Gas);
    v.deposit(SpeciesId::new("N2"), Moles(0.02), Phase::Gas);
    v.refresh_pressure();
    v
}

#[test]
fn finite_heat_pass_adds_actual_q_to_prepass_u_or_h() {
    for boundary in [
        Headspace::Sealed {
            volume: Liters(1.0),
        },
        Headspace::PressureControlled {
            pressure: Pascal(200_000.0),
            volume: Liters(1.0),
        },
    ] {
        let mut v = reactive(boundary);
        let mut initial = v.clone();
        initial.temperature = Kelvin(900.0);
        initial.refresh_pressure();
        let internal = matches!(initial.headspace, Headspace::Sealed { .. });
        let before = gas_energy(&initial, internal);
        v.heat_input = Some(vessel::HeatInput {
            contents: initial.contents.clone(),
            temperature: initial.temperature,
            delivered_j: 500.0,
        });
        let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
        assert!(
            (gas_energy(&v, internal) - before - 500.0).abs() < 1e-4 + before.abs() * 2e-8,
            "{events:?}"
        );
        assert!(events.iter().any(|e| matches!(e, Event::ThermalEquilibrium { provenance, .. } if provenance.model.contains("HEAT adds 500.00 J"))), "{events:?}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::GasAbsorbed { .. } | Event::GasEvolved { .. })),
            "{events:?}"
        );
    }
}

#[test]
fn rigid_combustion_closes_internal_energy_pressure_and_every_element() {
    for volume in [0.5, 1.0, 2.0] {
        let mut v = reactive(Headspace::Sealed {
            volume: Liters(volume),
        });
        let before = ledger::ConservedLedger::from_vessel(&v);
        let u = gas_energy(&v, true);
        let mass = v.mass().0;
        let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::GasContained { species, .. } if species.0 == "water")),
            "{events:?}"
        );
        assert!(
            !events.iter().any(|e| matches!(
                e,
                Event::GasAbsorbed { .. } | Event::GasEvolved { .. } | Event::NotYetModeled { .. }
            )),
            "{events:?}"
        );
        assert!(before
            .check_against(&ledger::ConservedLedger::from_vessel(&v), 1e-7, 1e-12)
            .is_empty());
        assert!((gas_energy(&v, true) - u).abs() < 1e-4 + u.abs() * 2e-8);
        assert!((v.mass().0 - mass).abs() < mass * 1e-7);
        let mut reference_products = v.clone();
        reference_products.temperature = Kelvin(1200.0);
        let chemical_release = u - gas_energy(&reference_products, true);
        let reported_release = events
            .iter()
            .find_map(|e| match e {
                Event::ThermalEquilibrium {
                    reaction_energy_j, ..
                } => *reaction_energy_j,
                _ => None,
            })
            .expect("closed combustion discloses its NASA formation-energy release");
        assert!((reported_release - chemical_release).abs() < 1e-4 + chemical_release.abs() * 1e-8);
        assert_eq!(v.headspace_volume(), Some(Liters(volume)));
        let pressure = v.gas_moles().0 * R * 1000.0 * v.temperature.0 / volume;
        assert!((v.pressure.0 - pressure).abs() < pressure * 1e-8);
        assert!(v.temperature.0 > 1500.0);
    }
}

#[test]
fn piston_combustion_closes_enthalpy_and_has_explicit_expansion_work() {
    let pressure = Pascal(200_000.0);
    let mut v = reactive(Headspace::PressureControlled {
        pressure,
        volume: Liters(1.0),
    });
    let volume_before = v.headspace_volume().unwrap().0;
    let before = ledger::ConservedLedger::from_vessel(&v);
    let h = gas_energy(&v, false);
    let u = gas_energy(&v, true);
    let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasContained { .. })),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::GasAbsorbed { .. } | Event::GasEvolved { .. } | Event::NotYetModeled { .. }
        )),
        "{events:?}"
    );
    assert!(before
        .check_against(&ledger::ConservedLedger::from_vessel(&v), 1e-7, 1e-12)
        .is_empty());
    assert!((gas_energy(&v, false) - h).abs() < 1e-4 + h.abs() * 2e-8);
    assert_eq!(v.pressure, pressure);
    let volume_after = v.headspace_volume().unwrap().0;
    assert!(volume_after > volume_before);
    let work = pressure.0 * (volume_after - volume_before) * 1e-3;
    assert!((u - gas_energy(&v, true) - work).abs() < 1e-4 + h.abs() * 2e-8);
}

#[test]
fn unsupported_condensed_feed_and_failed_energy_roots_never_invent_heat() {
    let n2 = [db().get("N2").unwrap()];
    let budget = BTreeMap::from([("N".to_string(), 0.06)]);
    assert!(closed::equilibrate_hp_verified(&budget, &n2, -1e10, 1.0).is_err());
    assert!(closed::equilibrate_uv(&budget, &n2, 1e10, 1.0).is_err());
    assert!(closed::equilibrate_tv(&budget, &n2, 900.0, 0.0).is_err());
    // The shared/open HP route must also reject unmatched cold floors and
    // unavailable high-energy roots instead of labelling them adiabatic.
    assert!(kerotakis_cea::equilibrate_hp(&budget, &n2, -1e10, 1.0).is_err());
    assert!(kerotakis_cea::equilibrate_hp(&budget, &n2, 1e10, 1.0).is_err());
    let mut v = reactive(Headspace::Sealed {
        volume: Liters(1.0),
    });
    v.deposit(SpeciesId::new("Mg"), Moles(0.001), Phase::Solid);
    let before = format!("{v:?}");
    let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. })));
    assert_eq!(format!("{v:?}"), before);
}

#[test]
fn failed_closed_solver_is_atomic() {
    let mut v = reactive(Headspace::Sealed {
        volume: Liters(0.0),
    });
    let before = format!("{v:?}");
    assert!(ThermalEquilibrator.equilibrate(&mut v).is_err());
    assert_eq!(format!("{v:?}"), before);
}

#![cfg(feature = "engine")]
use kerotakis_core::ops::Endpoint;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

#[test]
fn endpoint_is_solved_not_the_first_full_increment_past_it() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for (analyte, titrant) in [("HCl", "NaOH"), ("NaOH", "HCl")] {
        for volume in [0.01, 0.1, 1.0] {
            for concentration in [0.01, 0.1] {
                for increment_fraction in [0.3, 0.7, 1.3] {
                    let mut bench = Bench::new();
                    let v = &mut bench.vessels[0];
                    v.thermal_mode = ThermalMode::Thermostatted(Kelvin::STANDARD);
                    v.deposit(
                        SpeciesId::new("water"),
                        species::lookup_key("water")
                            .unwrap()
                            .moles_from_liters(Liters(volume)),
                        Phase::Liquid,
                    );
                    let amount = volume * 0.001;
                    v.deposit(
                        SpeciesId::new(analyte),
                        Moles(amount),
                        species::lookup_key(analyte).unwrap().standard_phase,
                    );
                    solver.equilibrate(v).unwrap();
                    let before_mass = v.mass().0;
                    let events = bench
                        .step_with(
                            Operator::Titrate {
                                vessel: VesselId(0),
                                titrant: SpeciesId::new(titrant),
                                concentration,
                                step: Liters(amount / concentration * increment_fraction),
                                target_ph: 7.0,
                                max_steps: 20,
                                endpoint: Endpoint::Ph,
                            },
                            &mut solver,
                            &PermissiveScreen,
                        )
                        .unwrap();
                    assert!(
                        !events
                            .iter()
                            .any(|e| matches!(e, Event::SolverFailed { .. })),
                        "{events:?}"
                    );
                    let v = &bench.vessels[0];
                    let ph = v.solution.as_ref().unwrap().ph;
                    assert!((ph - 7.0).abs() <= 1e-4, "{analyte} {volume} L titrated with {concentration} M, increment {increment_fraction}: pH={ph}");
                    let delivered = events
                        .iter()
                        .find_map(|e| match e {
                            Event::Titrated { total_volume, .. } => Some(total_volume.0),
                            _ => None,
                        })
                        .expect("titration event");
                    assert!(
                        (delivered * concentration / amount - 1.0).abs() < 1e-4,
                        "neutral endpoint must agree with equivalent balance: {delivered} L"
                    );
                    let expected_mass = before_mass
                        + delivered
                            * concentration
                            * species::lookup_key(titrant).unwrap().molar_mass
                        + species::lookup_key("water")
                            .unwrap()
                            .moles_from_liters(Liters(delivered))
                            .0
                            * 18.015;
                    assert!(
                        (v.mass().0 - expected_mass).abs() < 1e-5 + expected_mass * 1e-6,
                        "trial doses must not leak into inventory"
                    );
                }
            }
        }
    }
}

//! Bounded independent operation matrices, exercising the production solver order.
#![cfg(feature = "engine")]
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        DisplacementEquilibrator::wrapping(Box::new(PhreeqcEquilibrator::new().unwrap())),
    )))])
}
fn step(b: &mut Bench, s: &mut SolverStack, line: &str) -> Vec<Event> {
    let events = b
        .step_with(
            script::parse_op(line).unwrap().unwrap(),
            s,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{line}: {events:?}"
    );
    events
}
fn liquid_carbon(v: &Vessel) -> (f64, f64, f64) {
    let inventory = v
        .contents
        .iter()
        .filter(|p| {
            matches!(p.phase, Phase::Liquid | Phase::Aqueous)
                && ["CO2(aq)", "HCO3-", "CO3-2"].contains(&p.species.0.as_str())
        })
        .map(|p| p.moles.0)
        .sum::<f64>();
    let info = v.solution.as_ref().expect("represented aqueous carbon");
    let solved = info
        .species
        .iter()
        .filter(|p| ["CO2", "H2CO3", "HCO3-", "CO3-2"].contains(&p.name.as_str()))
        .map(|p| p.molality)
        .sum::<f64>()
        * info.solvent_kg.unwrap();
    // Native distribution rows print four significant figures. Bound the
    // roundoff of each reported species rather than treating the human
    // report as the high-precision selected-output inventory.
    let report_roundoff: f64 = info
        .species
        .iter()
        .filter(|p| ["CO2", "H2CO3", "HCO3-", "CO3-2"].contains(&p.name.as_str()))
        .filter(|p| p.molality > 0.0)
        .map(|p| 0.5 * 10f64.powf(p.molality.log10().floor() - 3.0))
        .sum::<f64>()
        * info.solvent_kg.unwrap();
    (inventory, solved, report_roundoff)
}
#[test]
fn carbon_state_agrees_across_dose_volume_boundary_and_operation_matrix() {
    for volume in [50, 100, 200] {
        for dose in [0.00001, 0.0001, 0.001] {
            for sealed in [false, true] {
                let mut b = Bench::new();
                let mut s = stack();
                step(&mut b, &mut s, &format!("add v1 water {volume}mL"));
                if sealed {
                    step(&mut b, &mut s, "seal v1 100mL");
                }
                step(&mut b, &mut s, &format!("add v1 CO2 {dose}mol"));
                for op in [None, Some("wait 1s"), Some("heat v1 1J"), Some("wait 2s")] {
                    if let Some(op) = op {
                        step(&mut b, &mut s, op);
                    }
                    let (inventory, solved, report_roundoff) =
                        liquid_carbon(b.vessel(VesselId(0)).unwrap());
                    assert!((inventory - solved).abs() < 1e-9 + report_roundoff,
                        "volume={volume} dose={dose} sealed={sealed} op={op:?}: {inventory} vs {solved}");
                }
            }
        }
    }
}
#[test]
fn oil_drain_conserves_mass_and_ions_across_layer_amounts() {
    for water in [20, 100, 200] {
        for oil in [10, 50] {
            let mut b = Bench::new();
            let mut s = stack();
            step(&mut b, &mut s, &format!("add v1 water {water}mL"));
            step(&mut b, &mut s, "add v1 NaCl 0.001mol");
            step(&mut b, &mut s, &format!("add v1 vegetable_oil {oil}mL"));
            step(&mut b, &mut s, "new beaker");
            let before = b.vessel(VesselId(0)).unwrap().mass().0;
            let events = step(&mut b, &mut s, "drain v1 v2");
            let a = b.vessel(VesselId(0)).unwrap();
            let dst = b.vessel(VesselId(1)).unwrap();
            assert!(
                (a.mass().0 + dst.mass().0 - before).abs() < 1e-7,
                "{events:?}"
            );
            assert!(a.moles_of(&SpeciesId::new("water")).0 < 1e-8);
            assert!((dst.moles_of(&SpeciesId::new("Na+")).0 - 0.001).abs() < 1e-8);
            assert!((dst.moles_of(&SpeciesId::new("Cl-")).0 - 0.001).abs() < 1e-8);
            assert!(dst.unresolved_materials.is_empty());
            assert!(!a.unresolved_materials.is_empty());
        }
    }
}
#[test]
fn actual_production_stack_burns_alcohol_before_phase_route_can_vent_feed() {
    for (fuel, co2_factor, water_factor) in [("ethanol", 2., 3.), ("methanol", 1., 2.)] {
        for dose in [0.001, 0.01] {
            let mut b = Bench::new();
            let mut s = stack();
            step(&mut b, &mut s, &format!("add v1 {fuel} {dose}mol"));
            let events = step(&mut b, &mut s, "ignite v1");
            let v = b.vessel(VesselId(0)).unwrap();
            assert!(
                v.moles_of(&SpeciesId::new(fuel)).0 < dose * 1e-4,
                "{fuel}: {events:?}"
            );
            assert!(!v.ignition_trial);
            assert!(events
                .iter()
                .any(|e| matches!(e, Event::Ignited { energy_j: Some(q), .. } if *q > 100.)));
            for (key, factor) in [("CO2", co2_factor), ("water", water_factor)] {
                let out: f64 = events
                    .iter()
                    .filter_map(|e| match e {
                        Event::GasEvolved { species, moles, .. } if species.0 == key => {
                            Some(moles.0)
                        }
                        _ => None,
                    })
                    .sum();
                assert!(
                    (out - dose * factor).abs() < dose * factor * 0.01,
                    "{fuel} {key}: {out} {events:?}"
                );
            }
            assert!(!events
                .iter()
                .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == fuel)));
        }
    }
}
#[test]
fn failed_trials_and_ordinary_heat_keep_distinct_physical_meanings() {
    let mut b = Bench::new();
    let mut s = stack();
    step(&mut b, &mut s, "add v1 isopropanol 1g");
    let before = serde_json::to_value(b.vessel(VesselId(0)).unwrap()).unwrap();
    step(&mut b, &mut s, "ignite v1");
    assert_eq!(
        serde_json::to_value(b.vessel(VesselId(0)).unwrap()).unwrap(),
        before
    );
    assert!(!b.vessel(VesselId(0)).unwrap().ignition_trial);
    let mut heat = Bench::new();
    let mut s = stack();
    step(&mut heat, &mut s, "add v1 ethanol 1g");
    let events = step(&mut heat, &mut s, "heat v1 2000J");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "ethanol")),
        "{events:?}"
    );
    assert!(!heat.vessel(VesselId(0)).unwrap().ignition_trial);
}
#[test]
fn cooling_scales_with_reviewed_dissolution_amount_without_false_precision() {
    for volume in [50, 100, 200] {
        for amount in [0.001, 0.005, 0.01] {
            let mut b = Bench::new();
            let mut s = stack();
            step(&mut b, &mut s, &format!("add v1 water {volume}mL"));
            step(&mut b, &mut s, &format!("add v1 KNO3 {amount}mol"));
            let v = b.vessel(VesselId(0)).unwrap();
            let dilute_estimate = amount * 34894.56 / (4.18 * volume as f64);
            let cooling = 298.15 - v.temperature.0;
            assert!(
                (cooling / dilute_estimate - 1.).abs() < 0.08,
                "{volume}, {amount}: {cooling} vs {dilute_estimate}"
            );
            assert!(v.unpriced_heat.is_empty());
        }
    }
}

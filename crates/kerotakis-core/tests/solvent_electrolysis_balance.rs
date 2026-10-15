//! Independent half-reaction balances for named hydrogen versus metal cathodes.
use kerotakis_core::{displacement, ledger::ConservedLedger, *};

const ELECTRONS: f64 = 0.002;
const AMPS: f64 = 0.5;

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

fn deposit(bench: &mut Bench, species: &str, amount: f64, phase: Phase) {
    bench.vessels[0].deposit(SpeciesId::new(species), Moles(amount), phase);
}

fn amount(vessel: &Vessel, species: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == species)
        .map(|portion| portion.moles.0)
        .sum()
}

fn water() -> Bench {
    let mut bench = Bench::new();
    deposit(&mut bench, "water", 1.0, Phase::Liquid);
    bench
}

fn run_and_close(bench: &mut Bench) -> Vec<Event> {
    let before = ConservedLedger::from_vessel(bench.vessel(VesselId(0)).unwrap());
    let events = bench
        .step(Operator::Electrolyse {
            vessel: VesselId(0),
            amps: AMPS,
            seconds: ELECTRONS * displacement::FARADAY / AMPS,
        })
        .unwrap();
    // The open cell exports neutral gas. Account for that measured output
    // when comparing the operator's complete matter/charge transaction.
    let mut system = bench.vessel(VesselId(0)).unwrap().clone();
    for event in &events {
        if let Event::GasEvolved { species, moles, .. } = event {
            system.deposit(species.clone(), *moles, Phase::Gas);
        }
    }
    let after = ConservedLedger::from_vessel(&system);
    close(after.charge, before.charge);
    let elements: std::collections::BTreeSet<_> = before
        .elements
        .keys()
        .chain(after.elements.keys())
        .collect();
    for element in elements {
        close(
            after.elements.get(element).copied().unwrap_or(0.0),
            before.elements.get(element).copied().unwrap_or(0.0),
        );
    }
    events
}

fn evolved(events: &[Event], species: &str) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved {
                species: id, moles, ..
            } if id.0 == species => Some(moles.0),
            _ => None,
        })
        .sum()
}

#[test]
fn chloralkali_hydrogen_consumes_water_and_leaves_charge_balancing_hydroxide() {
    let mut bench = water();
    deposit(&mut bench, "Na+", 0.01, Phase::Aqueous);
    deposit(&mut bench, "Cl-", 0.01, Phase::Aqueous);
    let predicted = displacement::electrolyse_solvent(
        bench.vessel(VesselId(0)).unwrap(),
        AMPS,
        ELECTRONS * displacement::FARADAY / AMPS,
    )
    .unwrap();
    assert!(!predicted.cathode_plates);
    close(predicted.hydroxide_made, ELECTRONS);
    close(predicted.water_spent, ELECTRONS);
    close(predicted.protons_made, 0.0);
    // Main H2 is not counted again as a metal-depletion coproduct.
    close(predicted.cathode_hydrogen_moles, 0.0);
    close(predicted.electrons_to_hydrogen, 0.0);
    let events = run_and_close(&mut bench);
    let final_vessel = bench.vessel(VesselId(0)).unwrap();
    close(amount(final_vessel, "Na+"), 0.01);
    close(amount(final_vessel, "Cl-"), 0.01 - ELECTRONS);
    close(amount(final_vessel, "OH-"), ELECTRONS);
    close(amount(final_vessel, "water"), 1.0 - ELECTRONS);
    close(evolved(&events, "H2"), ELECTRONS / 2.0);
    close(evolved(&events, "Cl2"), ELECTRONS / 2.0);
}

#[test]
fn sulfate_water_splitting_leaves_no_net_acid_and_retains_supporting_salt() {
    let mut bench = water();
    deposit(&mut bench, "Na+", 0.02, Phase::Aqueous);
    deposit(&mut bench, "SO4-2", 0.01, Phase::Aqueous);
    let events = run_and_close(&mut bench);
    let vessel = bench.vessel(VesselId(0)).unwrap();
    close(amount(vessel, "H+"), 0.0);
    close(amount(vessel, "OH-"), 0.0);
    close(amount(vessel, "water"), 1.0 - ELECTRONS / 2.0);
    close(amount(vessel, "Na+"), 0.02);
    close(amount(vessel, "SO4-2"), 0.01);
    close(evolved(&events, "H2"), ELECTRONS / 2.0);
    close(evolved(&events, "O2"), ELECTRONS / 4.0);
}

#[test]
fn metal_plating_and_its_hydrogen_remainder_keep_distinct_electron_accounts() {
    // No copper, exhausted part-way, and enough copper for the whole dose.
    for copper in [0.0, 0.00025, 0.002] {
        let mut bench = water();
        deposit(&mut bench, "Na+", 0.02, Phase::Aqueous);
        deposit(&mut bench, "SO4-2", 0.01 + copper, Phase::Aqueous);
        if copper > 0.0 {
            deposit(&mut bench, "Cu+2", copper, Phase::Aqueous);
        }
        let metal_electrons = (2.0 * copper).min(ELECTRONS);
        let hydrogen_electrons = ELECTRONS - metal_electrons;
        let predicted = displacement::electrolyse_solvent(
            bench.vessel(VesselId(0)).unwrap(),
            AMPS,
            ELECTRONS * displacement::FARADAY / AMPS,
        )
        .unwrap();
        close(predicted.water_spent, ELECTRONS / 2.0);
        close(predicted.protons_made, metal_electrons);
        close(predicted.hydroxide_made, 0.0);
        if copper > 0.0 {
            assert!(predicted.cathode_plates);
            close(predicted.electrons_to_hydrogen, hydrogen_electrons);
            close(predicted.cathode_hydrogen_moles, hydrogen_electrons / 2.0);
        }
        let events = run_and_close(&mut bench);
        let vessel = bench.vessel(VesselId(0)).unwrap();
        close(amount(vessel, "Cu"), metal_electrons / 2.0);
        close(amount(vessel, "Cu+2"), copper - metal_electrons / 2.0);
        close(amount(vessel, "H+"), metal_electrons);
        close(amount(vessel, "water"), 1.0 - ELECTRONS / 2.0);
        close(evolved(&events, "H2"), hydrogen_electrons / 2.0);
        close(evolved(&events, "O2"), ELECTRONS / 4.0);
    }
}

fn assert_water_dose_rejected(bench: &mut Bench) {
    let before = serde_json::to_value(&*bench).unwrap();
    let error = bench
        .step(Operator::Electrolyse {
            vessel: VesselId(0),
            amps: AMPS,
            seconds: ELECTRONS * displacement::FARADAY / AMPS,
        })
        .unwrap_err();
    match error {
        kerotakis_core::bench::BenchError::InvalidState(detail) => {
            assert!(detail.contains("electrolysis water budget"));
            assert!(detail.contains("No electrode products or partial current dose"));
        }
        other => panic!("expected typed water-budget refusal, got {other:?}"),
    }
    assert_eq!(
        serde_json::to_value(&*bench).unwrap(),
        before,
        "a refused dose must retain the entire bench, including logs and measurements"
    );
}

fn electrolyte_with_water(water_moles: f64, chloride: bool) -> Bench {
    let mut bench = Bench::new();
    deposit(&mut bench, "water", water_moles, Phase::Liquid);
    deposit(&mut bench, "Na+", 0.02, Phase::Aqueous);
    if chloride {
        deposit(&mut bench, "Cl-", 0.02, Phase::Aqueous);
    } else {
        deposit(&mut bench, "SO4-2", 0.01, Phase::Aqueous);
    }
    bench
}

#[test]
fn solvent_electrolysis_refuses_overdraw_and_dry_endpoint_without_partial_mutation() {
    for chloride in [false, true] {
        let required = if chloride { ELECTRONS } else { ELECTRONS / 2.0 };
        for available in [required / 10.0, required] {
            let mut bench = electrolyte_with_water(available, chloride);
            assert_water_dose_rejected(&mut bench);
        }
        // A dose leaving actual aqueous solvent is admitted; the refused
        // endpoint above is a dry-phase model boundary, not a current clamp.
        let mut bench = electrolyte_with_water(required * 1.01, chloride);
        run_and_close(&mut bench);
        close(
            amount(bench.vessel(VesselId(0)).unwrap(), "water"),
            required * 0.01,
        );
    }
}

#[test]
fn metal_half_cell_refuses_water_overdraw_before_plating_or_changing_its_measurements() {
    for available in [ELECTRONS / 20.0, ELECTRONS / 2.0] {
        let mut bench = Bench::new();
        deposit(&mut bench, "water", available, Phase::Liquid);
        deposit(&mut bench, "Cu", 0.001, Phase::Solid);
        deposit(&mut bench, "Cu+2", 0.001, Phase::Aqueous);
        deposit(&mut bench, "SO4-2", 0.001, Phase::Aqueous);
        // Existing electrode admission requires a measured aqueous activity.
        // This fixture supplies that admission state, not a new activity model.
        bench.vessels[0].solution = Some(kerotakis_core::vessel::SolutionInfo {
            scope: Default::default(),
            solvent_kg: Some(available * 18.015 / 1000.0),
            ph: 4.0,
            pe: None,
            redox: vec![],
            ionic_strength: 0.01,
            provenance: None,
            solvent_activity: None,
            species: vec![kerotakis_core::vessel::SpeciesDetail {
                name: "Cu+2".into(),
                molality: 0.01,
                activity: 0.01,
            }],
        });
        assert!(
            displacement::electrode(bench.vessel(VesselId(0)).unwrap()).is_some(),
            "positive admission control for the metal-half-cell branch"
        );
        assert_water_dose_rejected(&mut bench);
    }
}

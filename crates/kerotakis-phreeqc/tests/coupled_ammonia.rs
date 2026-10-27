//! Finite reduced-nitrogen partitioning and protonation share one owner.
#![cfg(feature = "engine")]
use kerotakis_core::{ledger::ConservedLedger, *};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn amount(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn closed(expected: &Vessel, actual: &Vessel) {
    let a = ConservedLedger::from_vessel(expected);
    let b = ConservedLedger::from_vessel(actual);
    for (element, before) in &a.elements {
        let after = b.elements.get(element).copied().unwrap_or(0.0);
        assert!(
            (before - after).abs() < 1e-9 + before.abs() * 1e-9,
            "{element}: {before} -> {after}; {actual:?}"
        );
    }
    assert!(
        (a.charge - b.charge).abs() < 1e-10,
        "charge: {} -> {}",
        a.charge,
        b.charge
    );
}

fn step(b: &mut Bench, s: &mut SolverStack, op: Operator) {
    let events = b.step_with(op, s, &PermissiveScreen).unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{events:?}"
    );
}
fn add(b: &mut Bench, s: &mut SolverStack, key: &str, mol: f64) {
    step(
        b,
        s,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(mol),
            at: None,
        },
    );
}

fn arm(volume: f64, dose: f64, acid_order: Option<bool>, regulated: bool) -> Vessel {
    let mut b = Bench::new();
    b.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    let mut s =
        kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())]);
    add(&mut b, &mut s, "water", 3.8739938939772416);
    let op = if regulated {
        Operator::Regulate {
            vessel: VesselId(0),
            pressure: Pascal(101325.0),
            initial_volume: Liters(volume),
        }
    } else {
        Operator::Seal {
            vessel: VesselId(0),
            headspace_volume: Liters(volume),
        }
    };
    step(&mut b, &mut s, op);
    let mut expected = b.vessels[0].clone();
    expected.deposit(SpeciesId::new("NH3"), Moles(dose), Phase::Liquid);
    if acid_order.is_some() {
        expected.deposit(SpeciesId::new("HCl"), Moles(2.0 * dose), Phase::Liquid);
    }
    if acid_order == Some(true) {
        add(&mut b, &mut s, "HCl", 2.0 * dose);
    }
    add(&mut b, &mut s, "NH3", dose);
    if acid_order == Some(false) {
        add(&mut b, &mut s, "HCl", 2.0 * dose);
    }
    for _ in 0..3 {
        let v = &b.vessels[0];
        closed(&expected, v);
        for key in ["N2", "O2"] {
            assert!(
                (amount(v, key, Phase::Gas) - amount(&expected, key, Phase::Gas)).abs() < 1e-12,
                "carrier must remain inert"
            );
        }
        assert!(
            amount(v, "NO3-", Phase::Aqueous) < 1e-12,
            "no oxidation of reduced nitrogen"
        );
        if regulated {
            assert!((v.pressure.0 - 101325.0).abs() < 1e-5);
        }
        step(&mut b, &mut s, Operator::Wait { seconds: 1.0 });
    }
    b.vessels[0].clone()
}

#[test]
fn acid_trapping_couples_finite_ammonia_gas_and_protonation_in_both_addition_orders() {
    for volume in [0.035, 0.35, 3.5] {
        for dose in [0.00008, 0.0008] {
            for regulated in [false, true] {
                let neutral = arm(volume, dose, None, regulated);
                let first = arm(volume, dose, Some(true), regulated);
                let after = arm(volume, dose, Some(false), regulated);
                eprintln!(
                    "AMMONIA_ARMS {}",
                    serde_json::json!({"volume":volume,"dose":dose,"regulated":regulated,"neutral":neutral,"acid_first":first,"acid_after":after})
                );
                let ng = amount(&neutral, "NH3", Phase::Gas);
                assert!(ng > 1e-9, "positive control must actually form ammonia gas");
                for acid in [&first, &after] {
                    let ag = amount(acid, "NH3", Phase::Gas);
                    // Strong acid doubles supplied ammonia: almost all reduced
                    // nitrogen must be NH4+, not just a tiny gas sign change.
                    assert!(ag < ng * 1e-4, "acid gas {ag}, neutral {ng}");
                    assert!(amount(acid, "NH4+", Phase::Aqueous) > 0.999 * dose);
                    assert!(amount(acid, "NH3", Phase::Aqueous) < dose * 1e-5);
                    assert!((amount(acid, "Cl-", Phase::Aqueous) - 2.0 * dose).abs() < 1e-10);
                }
                assert!(
                    (first.solution.as_ref().unwrap().ph - after.solution.as_ref().unwrap().ph)
                        .abs()
                        < 1e-5
                );
                assert!(
                    (amount(&first, "NH3", Phase::Gas) - amount(&after, "NH3", Phase::Gas)).abs()
                        < 1e-10
                );
            }
        }
    }
}

#[test]
fn supplied_gas_and_initially_absent_gas_close_the_same_reduced_nitrogen_pool() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for phase in [Phase::Gas, Phase::Aqueous] {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.headspace = Headspace::Sealed {
            volume: Liters(0.35),
        };
        v.deposit(SpeciesId::new("water"), Moles(3.87), Phase::Liquid);
        v.deposit(SpeciesId::new("NH3"), Moles(0.0008), phase);
        let expected = v.clone();
        solver.equilibrate(&mut v).unwrap();
        eprintln!(
            "SUPPLIED_AMMONIA {}",
            serde_json::json!({"input_phase":phase,"actual":v})
        );
        closed(&expected, &v);
        assert!(amount(&v, "NH3", Phase::Gas) > 1e-9);
        assert!(amount(&v, "NH3", Phase::Aqueous) > 1e-4);
        assert!(amount(&v, "NH4+", Phase::Aqueous) > 1e-6);
    }
}

#[test]
fn extended_route_ammonia_gas_phase_conserves_the_vessel() {
    // Acetic acid routes this sealed vessel to minteq.v4, which now defines
    // NH3(g); the coupled gas/protonation equilibrium must solve and keep
    // every supplied atom rather than refuse.
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    for supplied_gas in [false, true] {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.headspace = Headspace::Sealed {
            volume: Liters(0.35),
        };
        v.deposit(SpeciesId::new("water"), Moles(3.87), Phase::Liquid);
        v.deposit(
            SpeciesId::new("NH3"),
            Moles(0.0008),
            if supplied_gas {
                Phase::Gas
            } else {
                Phase::Aqueous
            },
        );
        v.deposit(SpeciesId::new("CH3COOH"), Moles(0.0001), Phase::Aqueous);
        let expected = v.clone();
        let result = solver.equilibrate(&mut v);
        eprintln!(
            "EXTENDED_AMMONIA {}",
            serde_json::json!({"supplied_gas":supplied_gas,"actual":v,"events_or_error":format!("{result:?}")})
        );
        result.unwrap();
        closed(&expected, &v);
        assert!(amount(&v, "NH3", Phase::Gas) > 0.0);
        assert!(amount(&v, "NH4+", Phase::Aqueous) > 1e-5);
    }
}

#[test]
fn dry_ammonia_and_core_henry_fallback_keep_their_own_inventory() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.headspace = Headspace::Sealed {
        volume: Liters(0.35),
    };
    v.deposit(SpeciesId::new("NH3"), Moles(0.0008), Phase::Liquid);
    // A manually built empty headspace starts at zero pressure, not the
    // constructor's atmospheric default. Match that physical fixture
    // before checking an unchanged dry solve.
    v.refresh_pressure();
    let before = serde_json::to_value(&v).unwrap();
    solver.equilibrate(&mut v).unwrap();
    assert_eq!(before, serde_json::to_value(&v).unwrap());
    v.deposit(SpeciesId::new("water"), Moles(3.87), Phase::Liquid);
    let expected = v.clone();
    kerotakis_core::volatility::HeadspacePartitionEquilibrator
        .equilibrate(&mut v)
        .unwrap();
    closed(&expected, &v);
    assert!(amount(&v, "NH3", Phase::Gas) > 1e-9);
}

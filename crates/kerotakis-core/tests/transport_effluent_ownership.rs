//! MODEL007 narrow seam: effluent ownership through the bench.
//!
//! Bench previously collapsed every CellChain effluent to (SpeciesId, Moles),
//! priced enthalpy as Liquid at inlet temperature, re-deposited at
//! standard_phase, and dropped solute_charge / free acid-base. This file
//! exercises only Bench::step_with(Operator::Transport) via public fixtures
//! (Bench::vessels is pub; vessel_mut is private, as PR809 showed).

use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{Bench, Equilibrator, MobileParcel, SolveError};

struct Noop;

impl Equilibrator for Noop {
    fn name(&self) -> &'static str {
        "noop-test"
    }
    fn equilibrate(&mut self, _vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Ok(Vec::new())
    }
}

// Backward compat: old JSON without free fields must still load.
#[test]
fn mobile_parcel_deserializes_old_json_without_free_fields() {
    let old = r#"{"contents":[],"temperature":298.15,"solute_charge":0.0}"#;
    let parcel: MobileParcel = serde_json::from_str(old).expect("old parcel loads");
    assert_eq!(parcel.free_hydroxide, 0.0);
    assert_eq!(parcel.free_proton, 0.0);
    assert_eq!(parcel.solute_charge, 0.0);
}

// Use realistic liquid-range temps: 293.15K (20C) and 333.15K (60C), not 273K freezing.
const COLD_K: f64 = 293.15;
const HOT_K: f64 = 333.15;

#[test]
fn hotcell_cold_inlet_empty_receiver_takes_cell_temperature() {
    // Baseline mirrors transport-thermal:hot-cell-cold-inlet (60C cell, 20C inlet, empty adiabatic receiver)
    // One full upwind cell of pure water leaves at pre-transport cell temp; empty receiver must retain it.
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].temperature = Kelvin(HOT_K);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(COLD_K);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[2];
        // empty adiabatic receiver at arbitrary 298K but no mass
        v.temperature = Kelvin(298.0);
    }
    let pre_cell_temp = bench.vessels[0].temperature.0;
    let pre_inlet_temp = bench.vessels[1].temperature.0;
    let mut noop = Noop;
    bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0)],
                inlet: VesselId(1),
                receiver: VesselId(2),
                steps: 1,
                courant: 1.0,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    // Empty receiver has no thermal mass, so its final temp must equal effluent's cell temp exactly (within 0.01K baseline tolerance)
    let recv_temp = bench.vessels[2].temperature.0;
    assert!(
        (recv_temp - pre_cell_temp).abs() < 0.01,
        "empty receiver must take cell effluent temp {} not inlet {}, got {}",
        pre_cell_temp,
        pre_inlet_temp,
        recv_temp
    );
    // Cell must now be at inlet temp
    assert!(
        (bench.vessels[0].temperature.0 - pre_inlet_temp).abs() < 0.01,
        "cell should be at inlet temp after Cf=1"
    );
    // Representative inlet unchanged
    assert!((bench.vessels[1].temperature.0 - pre_inlet_temp).abs() < 1e-9);
    assert!(bench.vessels[0].solution.is_none());
    assert!(bench.vessels[2].solution.is_none());
}

#[test]
fn cold_cell_hot_inlet_empty_receiver_takes_cell_temperature() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].temperature = Kelvin(COLD_K);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(HOT_K);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[2];
        v.temperature = Kelvin(298.0);
    }
    let pre_cell_temp = bench.vessels[0].temperature.0;
    let pre_inlet_temp = bench.vessels[1].temperature.0;
    let mut noop = Noop;
    bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0)],
                inlet: VesselId(1),
                receiver: VesselId(2),
                steps: 1,
                courant: 1.0,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    let recv_temp = bench.vessels[2].temperature.0;
    assert!(
        (recv_temp - pre_cell_temp).abs() < 0.01,
        "empty receiver must take cell temp {} not inlet {}, got {}",
        pre_cell_temp,
        pre_inlet_temp,
        recv_temp
    );
    assert!((bench.vessels[0].temperature.0 - pre_inlet_temp).abs() < 0.01);
}

#[test]
fn multiple_parcels_nonempty_receiver_conserves_enthalpy() {
    // 2 cells at COLD, inlet HOT, receiver nonempty at 298K with water mass.
    // Each step's effluent has different source T; old code priced both as HOT inlet, inventing heat.
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].temperature = Kelvin(COLD_K);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(COLD_K);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[2];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(HOT_K);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[3];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    let before_chain_energy: f64 = [0, 1].iter().map(|i| bench.vessels[*i].enthalpy().0).sum();
    let before_recv_energy = bench.vessels[3].enthalpy().0;
    let before_total = before_chain_energy + before_recv_energy;
    let inlet_energy_per_step = {
        let inlet = &bench.vessels[2];
        let parcel = MobileParcel::from_vessel(inlet);
        parcel.sensible_energy().0
    };
    let mut noop = Noop;
    bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(1)],
                inlet: VesselId(2),
                receiver: VesselId(3),
                steps: 2,
                courant: 1.0,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    assert!((bench.vessels[0].temperature.0 - HOT_K).abs() < 1e-9);
    assert!((bench.vessels[1].temperature.0 - HOT_K).abs() < 1e-9);
    let after_chain_energy: f64 = [0, 1].iter().map(|i| bench.vessels[*i].enthalpy().0).sum();
    let after_recv_energy = bench.vessels[3].enthalpy().0;
    let after_total = after_chain_energy + after_recv_energy;
    let expected = before_total + inlet_energy_per_step * 2.0;
    assert!(
        (after_total - expected).abs() < 1e-4,
        "enthalpy must close with per-parcel T, diff {}",
        after_total - expected
    );
    assert!(bench.vessels[3].moles_of(&SpeciesId::new("water")).0 > 5.5509);
}

#[test]
fn aqueous_portion_whose_standard_phase_is_solid_stays_aqueous() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new("CaCO3"), Moles(0.001), Phase::Aqueous);
    bench.vessels[0].temperature = Kelvin(298.15);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.15);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let mut noop = Noop;
    bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0)],
                inlet: VesselId(1),
                receiver: VesselId(3),
                steps: 1,
                courant: 1.0,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    let recv = &bench.vessels[3];
    let has_aq = recv
        .contents
        .iter()
        .any(|p| p.species.0 == "CaCO3" && p.phase == Phase::Aqueous);
    let has_solid = recv
        .contents
        .iter()
        .any(|p| p.species.0 == "CaCO3" && p.phase == Phase::Solid);
    assert!(
        has_aq,
        "CaCO3 must stay aqueous, old code made it solid via standard_phase"
    );
    assert!(!has_solid, "must not invent solid CaCO3");
}

#[test]
fn baseline_h_oh_charge_fraction_complement_positive_and_negative() {
    // Positive charge case
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Aqueous);
    bench.vessels[0].solute_charge = 0.01;
    bench.vessels[0].free_hydroxide = 0.004;
    bench.vessels[0].free_proton = 0.002;
    bench.vessels[0].temperature = Kelvin(298.0);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let initial_charge = bench.vessels[0].solute_charge;
    let initial_oh = bench.vessels[0].free_hydroxide;
    let initial_h = bench.vessels[0].free_proton;
    let mut noop = Noop;
    bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0)],
                inlet: VesselId(1),
                receiver: VesselId(2),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    assert!((bench.vessels[0].solute_charge - initial_charge * 0.5).abs() < 1e-12);
    assert!((bench.vessels[2].solute_charge - initial_charge * 0.5).abs() < 1e-12);
    assert!((bench.vessels[0].free_hydroxide - initial_oh * 0.5).abs() < 1e-12);
    assert!((bench.vessels[2].free_hydroxide - initial_oh * 0.5).abs() < 1e-12);
    assert!((bench.vessels[0].free_proton - initial_h * 0.5).abs() < 1e-12);
    assert!((bench.vessels[2].free_proton - initial_h * 0.5).abs() < 1e-12);

    // Negative charge case
    let mut bench2 = Bench::new();
    bench2.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench2.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Aqueous);
    bench2.vessels[0].solute_charge = -0.015;
    bench2.vessels[0].free_hydroxide = 0.003;
    bench2.vessels[0].free_proton = 0.007;
    bench2.vessels[0].temperature = Kelvin(298.0);
    bench2.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench2.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    bench2.step(Operator::NewVessel { kind: None }).unwrap();
    let ic2 = bench2.vessels[0].solute_charge;
    let oh2 = bench2.vessels[0].free_hydroxide;
    let h2 = bench2.vessels[0].free_proton;
    bench2
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0)],
                inlet: VesselId(1),
                receiver: VesselId(2),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    assert!((bench2.vessels[0].solute_charge - ic2 * 0.5).abs() < 1e-12);
    assert!((bench2.vessels[2].solute_charge - ic2 * 0.5).abs() < 1e-12);
    assert!((bench2.vessels[0].free_hydroxide - oh2 * 0.5).abs() < 1e-12);
    assert!((bench2.vessels[2].free_hydroxide - oh2 * 0.5).abs() < 1e-12);
    assert!((bench2.vessels[0].free_proton - h2 * 0.5).abs() < 1e-12);
    assert!((bench2.vessels[2].free_proton - h2 * 0.5).abs() < 1e-12);
}

#[test]
fn zero_courant_without_invented_movement_or_heat() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].temperature = Kelvin(333.15);
    bench.vessels[0].solution = Some(kerotakis_core::vessel::SolutionInfo {
        scope: Default::default(),
        solvent_kg: Some(0.1),
        pe: None,
        redox: vec![],
        ph: 7.0,
        ionic_strength: 0.01,
        species: vec![],
        provenance: None,
        solvent_activity: None,
    });
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(COLD_K);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[2];
        v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        v.temperature = Kelvin(298.0);
        v.solution = Some(kerotakis_core::vessel::SolutionInfo {
            scope: Default::default(),
            solvent_kg: Some(0.02),
            pe: None,
            redox: vec![],
            ph: 8.0,
            ionic_strength: 0.01,
            species: vec![],
            provenance: None,
            solvent_activity: None,
        });
    }
    let before_cell = bench.vessels[0].clone();
    let before_recv = bench.vessels[2].clone();
    let mut noop = Noop;
    let events = bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0)],
                inlet: VesselId(1),
                receiver: VesselId(2),
                steps: 3,
                courant: 0.0,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    assert_eq!(bench.vessels[0].temperature, before_cell.temperature);
    assert_eq!(bench.vessels[0].contents, before_cell.contents);
    assert_eq!(bench.vessels[0].solute_charge, before_cell.solute_charge);
    assert!(
        bench.vessels[0].solution.is_some(),
        "zero-Courant must not invalidate chain cache"
    );
    assert_eq!(bench.vessels[2].temperature, before_recv.temperature);
    assert_eq!(bench.vessels[2].contents, before_recv.contents);
    assert!(
        bench.vessels[2].solution.is_some(),
        "zero-Courant must not invalidate receiver cache"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::TemperatureChanged { .. })),
        "zero-Courant must not emit heat"
    );
    let transported = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }))
        .expect("still emits Transported");
    if let Event::Transported { effluent_moles, .. } = transported {
        assert!(effluent_moles.is_empty(), "zero effluent at Cf=0");
    }
}

#[test]
fn rejected_uniform_geometry_atomic_rollback() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].temperature = Kelvin(298.0);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(4.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[2];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[3];
        v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    let before_cells = vec![bench.vessels[0].clone(), bench.vessels[1].clone()];
    let before_recv = bench.vessels[3].clone();
    let before_log_len = bench.log.len();
    let mut noop = Noop;
    let result = bench.step_with(
        Operator::Transport {
            chain: vec![VesselId(0), VesselId(1)],
            inlet: VesselId(2),
            receiver: VesselId(3),
            steps: 1,
            courant: 0.5,
        },
        &mut noop,
        &kerotakis_core::PermissiveScreen,
    );
    assert!(result.is_err(), "nonuniform chain must be rejected");
    assert_eq!(
        bench.vessels[0].moles_of(&SpeciesId::new("water")).0,
        before_cells[0].moles_of(&SpeciesId::new("water")).0
    );
    assert_eq!(
        bench.vessels[1].moles_of(&SpeciesId::new("water")).0,
        before_cells[1].moles_of(&SpeciesId::new("water")).0
    );
    assert_eq!(bench.vessels[3].contents, before_recv.contents);
    assert_eq!(bench.log.len(), before_log_len);
}

#[test]
fn alias_guards_reject_duplicate_and_overlapping_roles() {
    // Helper to build minimal valid bench: 2 chain cells + inlet + receiver, all equal water, distinct temps safe.
    fn valid_bench() -> Bench {
        let mut b = Bench::new();
        b.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        b.vessels[0].temperature = Kelvin(298.0);
        b.step(Operator::NewVessel { kind: None }).unwrap();
        {
            let v = &mut b.vessels[1];
            v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
            v.temperature = Kelvin(298.0);
        }
        b.step(Operator::NewVessel { kind: None }).unwrap();
        {
            let v = &mut b.vessels[2];
            v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
            v.temperature = Kelvin(298.0);
        }
        b.step(Operator::NewVessel { kind: None }).unwrap();
        {
            let v = &mut b.vessels[3];
            v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
            v.temperature = Kelvin(298.0);
        }
        b
    }
    let mut noop = Noop;
    // Duplicate chain ids
    {
        let mut b = valid_bench();
        let before = b.vessels.clone();
        let before_log = b.log.len();
        let r = b.step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(0)],
                inlet: VesselId(2),
                receiver: VesselId(3),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        );
        assert!(r.is_err(), "duplicate chain must be rejected");
        assert_eq!(
            serde_json::to_value(&b.vessels).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
        assert_eq!(b.log.len(), before_log);
    }
    // Inlet alias chain
    {
        let mut b = valid_bench();
        let before = b.vessels.clone();
        let r = b.step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(1)],
                inlet: VesselId(0),
                receiver: VesselId(3),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        );
        assert!(r.is_err(), "inlet in chain must be rejected");
        assert_eq!(
            serde_json::to_value(&b.vessels).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
    }
    // Receiver alias chain
    {
        let mut b = valid_bench();
        let before = b.vessels.clone();
        let r = b.step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(1)],
                inlet: VesselId(2),
                receiver: VesselId(1),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        );
        assert!(r.is_err(), "receiver in chain must be rejected");
        assert_eq!(
            serde_json::to_value(&b.vessels).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
    }
    // Receiver == inlet
    {
        let mut b = valid_bench();
        let before = b.vessels.clone();
        let r = b.step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(1)],
                inlet: VesselId(2),
                receiver: VesselId(2),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        );
        assert!(r.is_err(), "receiver == inlet must be rejected");
        assert_eq!(
            serde_json::to_value(&b.vessels).unwrap(),
            serde_json::to_value(&before).unwrap()
        );
    }
    // Unique proper topology still works
    {
        let mut b = valid_bench();
        let r = b.step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(1)],
                inlet: VesselId(2),
                receiver: VesselId(3),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        );
        assert!(r.is_ok(), "valid distinct topology must succeed");
    }
}

// Existing tracer profile still holds (regression guard)
#[test]
fn existing_tracer_transport_still_unchanged() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
    bench.vessels[0].deposit(SpeciesId::new("NaCl"), Moles(0.01), Phase::Aqueous);
    bench.vessels[0].temperature = Kelvin(298.0);
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[1];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    {
        let v = &mut bench.vessels[2];
        v.deposit(SpeciesId::new("water"), Moles(5.5509), Phase::Liquid);
        v.temperature = Kelvin(298.0);
    }
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let mut noop = Noop;
    bench
        .step_with(
            Operator::Transport {
                chain: vec![VesselId(0), VesselId(1)],
                inlet: VesselId(2),
                receiver: VesselId(3),
                steps: 1,
                courant: 0.5,
            },
            &mut noop,
            &kerotakis_core::PermissiveScreen,
        )
        .unwrap();
    // Binomial: cell0 retains 0.5*0.01, cell1 gets 0.5*0.01
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaCl")).0 - 0.005).abs() < 1e-12);
    assert!((bench.vessels[1].moles_of(&SpeciesId::new("NaCl")).0 - 0.005).abs() < 1e-12);
}

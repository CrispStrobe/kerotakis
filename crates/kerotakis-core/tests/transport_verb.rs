//! Integration test: the `transport` verb wired through the bench.

use kerotakis_core::ops::{Event, Operator};
use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::Moles;
use kerotakis_core::vessel::VesselId;
use kerotakis_core::Bench;

fn setup_column() -> Bench {
    let mut bench = Bench::new();
    // v1 = VesselId(0) already exists; add water
    bench
        .step(Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(5.5509),
            at: None,
        })
        .unwrap();
    // v2
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(1),
            species: SpeciesId::new("water"),
            moles: Moles(5.5509),
            at: None,
        })
        .unwrap();
    // v3
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(2),
            species: SpeciesId::new("water"),
            moles: Moles(5.5509),
            at: None,
        })
        .unwrap();
    // v4 = inlet (same water volume + NaCl as aqueous tracer)
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
        .step(Operator::Add {
            vessel: VesselId(3),
            species: SpeciesId::new("water"),
            moles: Moles(5.5509),
            at: None,
        })
        .unwrap();
    // Deposit NaCl as Aqueous directly — the bench's default solver can't
    // dissolve it, but transport moves aqueous portions.
    bench.vessels[3].deposit(SpeciesId::new("NaCl"), Moles(0.1), Phase::Aqueous);
    // v5 = receiver
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    bench
}

#[test]
fn salt_pulse_disperses_through_column() {
    let mut bench = setup_column();
    let nacl = SpeciesId::new("NaCl");

    let events = bench
        .step(Operator::Transport {
            chain: vec![VesselId(0), VesselId(1), VesselId(2)],
            inlet: VesselId(3),
            receiver: VesselId(4),
            steps: 3,
            courant: 0.5,
        })
        .unwrap();

    let transported = events
        .iter()
        .find(|e| matches!(e, Event::Transported { .. }));
    assert!(transported.is_some(), "must emit Transported event");

    // Inlet is unchanged (it's a template, not consumed).
    let inlet_nacl = bench.vessel(VesselId(3)).unwrap().moles_of(&nacl).0;
    assert!(
        (inlet_nacl - 0.1).abs() < 1e-12,
        "inlet vessel must not be consumed"
    );

    // Conservation: total NaCl across chain + receiver = what was injected.
    // Cf=0.5, inlet has 0.1 mol NaCl, each step injects 0.05 mol → 3 steps = 0.15 mol.
    let chain_nacl: f64 = [VesselId(0), VesselId(1), VesselId(2)]
        .iter()
        .map(|id| bench.vessel(*id).unwrap().moles_of(&nacl).0)
        .sum();
    let receiver_nacl = bench.vessel(VesselId(4)).unwrap().moles_of(&nacl).0;
    let injected = 0.05 * 3.0;
    assert!(
        (chain_nacl + receiver_nacl - injected).abs() < 1e-10,
        "injected NaCl must equal chain + receiver: {chain_nacl} + {receiver_nacl} vs {injected}"
    );

    // After 3 steps at Cf=0.5 into 3 cells, cell 1 must have some NaCl.
    let cell1 = bench.vessel(VesselId(0)).unwrap().moles_of(&nacl).0;
    assert!(cell1 > 0.0, "cell 1 must have some NaCl after 3 steps");
}

#[test]
fn binomial_profile_after_two_steps() {
    let mut bench = setup_column();
    let nacl = SpeciesId::new("NaCl");

    bench
        .step(Operator::Transport {
            chain: vec![VesselId(0), VesselId(1), VesselId(2)],
            inlet: VesselId(3),
            receiver: VesselId(4),
            steps: 2,
            courant: 0.5,
        })
        .unwrap();

    // After 2 steps at Cf=0.5, the expected profile is the binomial
    // (0.05 mol injected per step):
    //   cell 0: 0.5 * 0.05 + 0.5 * 0.05 * 0.5 = 0.0375  wait...
    // Actually: step 1 injects 0.05 into cell 0.
    //   cell 0 = 0.05, cell 1 = 0, cell 2 = 0
    // Step 2: cell 0 gives 0.5*0.05=0.025 to cell 1, gets 0.05 from inlet.
    //   cell 0 = 0.5*0.05 + 0.05 = 0.075, wait no...
    // Let me be precise: after step 1:
    //   cell 0 has: (1-0.5)*0 + 0.5*inlet = 0 + 0.05 = 0.05
    //   cell 1 has: (1-0.5)*0 + 0.5*0 = 0  (the old cell 0 had 0 NaCl)
    //   cell 2 has: 0
    // After step 2:
    //   cell 0 has: 0.5*0.05 + 0.5*0.1 = 0.025 + 0.05 = 0.075
    //   cell 1 has: 0.5*0 + 0.5*0.05 = 0.025
    //   cell 2 has: 0
    // Wait, the upwind scheme: each cell retains (1-Cf) of its own and gets Cf from upstream.
    // After step 1 (inlet has 0.1 mol NaCl total, Cf=0.5 → injects 0.05):
    //   cell0 = (1-0.5)*0 + 0.5*0.1 = 0.05
    //   cell1 = (1-0.5)*0 + 0.5*cell0_before = 0.5*0 = 0
    //   cell2 = 0
    // After step 2:
    //   cell0 = 0.5*0.05 + 0.5*0.1 = 0.025 + 0.05 = 0.075
    //   cell1 = 0.5*0 + 0.5*0.05 = 0.025
    //   cell2 = 0
    let c0 = bench.vessel(VesselId(0)).unwrap().moles_of(&nacl).0;
    let c1 = bench.vessel(VesselId(1)).unwrap().moles_of(&nacl).0;
    let c2 = bench.vessel(VesselId(2)).unwrap().moles_of(&nacl).0;
    assert!(
        (c0 - 0.075).abs() < 1e-10,
        "cell 0 after 2 steps: expected 0.075, got {c0}"
    );
    assert!(
        (c1 - 0.025).abs() < 1e-10,
        "cell 1 after 2 steps: expected 0.025, got {c1}"
    );
    assert!(
        c2.abs() < 1e-15,
        "cell 2 after 2 steps: expected 0, got {c2}"
    );
}

#[test]
fn transport_refuses_empty_chain() {
    let mut bench = setup_column();
    let result = bench.step(Operator::Transport {
        chain: vec![],
        inlet: VesselId(3),
        receiver: VesselId(4),
        steps: 3,
        courant: 0.5,
    });
    assert!(result.is_err());
}

#[test]
fn transport_refuses_zero_steps() {
    let mut bench = setup_column();
    let result = bench.step(Operator::Transport {
        chain: vec![VesselId(0), VesselId(1), VesselId(2)],
        inlet: VesselId(3),
        receiver: VesselId(4),
        steps: 0,
        courant: 0.5,
    });
    assert!(result.is_err());
}

#[test]
fn water_volume_conserved_in_chain_cells() {
    let mut bench = setup_column();
    let water = SpeciesId::new("water");

    let cell_water_before: Vec<f64> = (0..3)
        .map(|i| bench.vessel(VesselId(i)).unwrap().moles_of(&water).0)
        .collect();

    bench
        .step(Operator::Transport {
            chain: vec![VesselId(0), VesselId(1), VesselId(2)],
            inlet: VesselId(3),
            receiver: VesselId(4),
            steps: 3,
            courant: 0.5,
        })
        .unwrap();

    for (i, &before) in cell_water_before.iter().enumerate() {
        let after = bench.vessel(VesselId(i)).unwrap().moles_of(&water).0;
        assert!(
            (after - before).abs() < 1e-6,
            "cell {i} water changed: {after} vs {before}",
        );
    }
}

#[test]
fn receiver_collects_effluent() {
    let mut bench = setup_column();
    let water = SpeciesId::new("water");

    bench
        .step(Operator::Transport {
            chain: vec![VesselId(0), VesselId(1), VesselId(2)],
            inlet: VesselId(3),
            receiver: VesselId(4),
            steps: 3,
            courant: 0.5,
        })
        .unwrap();

    let receiver_water = bench.vessel(VesselId(4)).unwrap().moles_of(&water).0;
    assert!(receiver_water > 0.0, "receiver must collect water effluent");
}

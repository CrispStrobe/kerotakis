//! EXP-49's acceptance: half-life recovered from computed activity
//! decay, nuclear equations balanced by mass number and charge on
//! every curated row, and the invariant transmutation actually keeps —
//! nucleons, not elements.

use kerotakis_core::nuclide::{self, DecayMode};
use kerotakis_core::*;

fn spike(bench: &mut Bench, notation: &str, moles: f64) -> Vec<Event> {
    bench
        .step(
            script::parse_op(&format!("add v1 {notation} {moles}mol"))
                .expect("grammar")
                .expect("an operator"),
        )
        .expect("spike")
}

fn geiger(bench: &mut Bench) -> f64 {
    let events = bench
        .step(script::parse_op("measure v1 geiger").unwrap().unwrap())
        .unwrap();
    events
        .iter()
        .find_map(|e| match e {
            Event::Measured { value, unit, .. } if unit == "Bq" => Some(*value),
            _ => None,
        })
        .expect("a Geiger reading in Bq")
}

#[test]
fn half_life_is_recovered_from_the_activity_series() {
    let mut bench = Bench::new();
    spike(&mut bench, "I-131", 1e-9);
    let a0 = geiger(&mut bench);
    assert!(a0 > 0.0, "a fresh tracer is active");
    // One half-life of I-131 (NUBASE2020): 693,377 s.
    bench.step(Operator::Wait { seconds: 693_377.0 }).unwrap();
    let a1 = geiger(&mut bench);
    let ratio = a1 / a0;
    assert!(
        (ratio - 0.5).abs() < 1e-3,
        "after one half-life the counter reads half: {ratio}"
    );
    // Two more half-lives → an eighth of the start.
    bench
        .step(Operator::Wait {
            seconds: 2.0 * 693_377.0,
        })
        .unwrap();
    let a3 = geiger(&mut bench);
    assert!(
        (a3 / a0 - 0.125).abs() < 1e-3,
        "three half-lives is an eighth: {}",
        a3 / a0
    );
}

#[test]
fn every_curated_equation_balances_a_and_z() {
    for data in nuclide::TEACHING_NUCLIDES {
        let Some(decay) = data.decay.as_ref() else {
            continue;
        };
        let daughter = nuclide::lookup_notation(decay.daughter).unwrap_or_else(|| {
            panic!(
                "{}: daughter {} must be curated",
                data.nuclide, decay.daughter
            )
        });
        let a_parent = nuclide::Nuclide::parse(data.nuclide).unwrap().mass_number as i64;
        let a_daughter = nuclide::Nuclide::parse(decay.daughter).unwrap().mass_number as i64;
        let (a_emitted, z_emitted): (i64, i64) = match decay.mode {
            DecayMode::Alpha => (4, 2),
            DecayMode::BetaMinus => (0, -1),
            DecayMode::BetaPlus | DecayMode::ElectronCapture => (0, 1),
            DecayMode::Gamma => (0, 0),
            DecayMode::SpontaneousFission => continue,
        };
        assert_eq!(
            a_parent,
            a_daughter + a_emitted,
            "{}: mass number balances",
            data.nuclide
        );
        assert_eq!(
            data.z as i64,
            daughter.z as i64 + z_emitted,
            "{}: charge balances",
            data.nuclide
        );
        assert!(
            nuclide::nuclear_equation(data).is_some(),
            "{}: the equation is written",
            data.nuclide
        );
    }
}

#[test]
fn nucleons_conserve_through_the_alpha_case() {
    // Rn-222 → Po-218 + He-4: the element changes, the α leaves the
    // atom — and the ledger keeps every nucleon because the He-4 stays.
    let mut bench = Bench::new();
    spike(&mut bench, "Rn-222", 1e-9);
    let before = nuclide::nucleon_moles(&bench.vessel(VesselId(0)).unwrap().nuclides);
    let events = bench.step(Operator::Wait { seconds: 330_350.0 }).unwrap();
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::Decayed { parent, daughter, .. }
                if parent == "Rn-222" && daughter == "Po-218"
        )),
        "the α decay is an event: {events:?}"
    );
    let ledger = &bench.vessel(VesselId(0)).unwrap().nuclides;
    let after = nuclide::nucleon_moles(ledger);
    assert!(
        (before - after).abs() < before * 1e-12,
        "Σ A·n is exact across transmutation: {before} → {after}"
    );
    let he4 = ledger
        .inventory
        .get(&nuclide::Nuclide::new("He", 4))
        .copied()
        .unwrap_or(0.0);
    assert!(he4 > 0.0, "the α parcels are in the ledger as He-4");
}

#[test]
fn the_chain_propagates_across_waits() {
    // Sr-90 → Y-90 → Zr-90: two real β steps. After many Y-90
    // half-lives with negligible Sr decay, Zr-90 appears.
    let mut bench = Bench::new();
    spike(&mut bench, "Sr-90", 1e-9);
    for _ in 0..10 {
        bench.step(Operator::Wait { seconds: 230_580.0 }).unwrap();
    }
    let ledger = &bench.vessel(VesselId(0)).unwrap().nuclides;
    let zr = ledger
        .inventory
        .get(&nuclide::Nuclide::new("Zr", 90))
        .copied()
        .unwrap_or(0.0);
    assert!(zr > 0.0, "the granddaughter exists: the chain is real");
    let before = 90.0 * 1e-9;
    let now = nuclide::nucleon_moles(ledger);
    assert!(
        (now - before).abs() < before * 1e-9,
        "nucleons conserve down the chain: {before} → {now}"
    );
}

#[test]
fn no_time_no_decay_and_unknowns_refuse() {
    let mut bench = Bench::new();
    let events = spike(&mut bench, "Co-60", 1e-9);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::HazardWarning { .. })),
        "the source is warned about: {events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::Decayed { .. })),
        "spiking is not waiting"
    );
    let events = bench
        .step(Operator::SpikeNuclide {
            vessel: VesselId(0),
            nuclide: "U-235".to_string(),
            moles: Moles(1e-9),
        })
        .unwrap();
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("teaching set")
        )),
        "an uncurated nuclide refuses with the shelf listed: {events:?}"
    );
}

#[test]
fn the_isomer_is_not_its_ground_state() {
    // Tc-99m → Tc-99 + γ: same element, same mass number, different
    // nuclide. The metastable flag keeps them distinct ledger keys —
    // without it the γ transition would be a no-op and the counter
    // would never fall.
    let mut bench = Bench::new();
    spike(&mut bench, "Tc-99m", 1e-9);
    let a0 = geiger(&mut bench);
    bench.step(Operator::Wait { seconds: 21_624.0 }).unwrap();
    let a1 = geiger(&mut bench);
    assert!(
        (a1 / a0 - 0.5).abs() < 1e-3,
        "one Tc-99m half-life halves the count: {}",
        a1 / a0
    );
    let ledger = &bench.vessel(VesselId(0)).unwrap().nuclides;
    let ground = nuclide::Nuclide::parse("Tc-99").unwrap();
    let iso = nuclide::Nuclide::parse("Tc-99m").unwrap();
    assert!(ledger.inventory.get(&ground).copied().unwrap_or(0.0) > 0.0);
    assert!(ledger.inventory.get(&iso).copied().unwrap_or(0.0) > 0.0);
    assert_ne!(ground, iso, "distinct keys, distinct nuclides");
}

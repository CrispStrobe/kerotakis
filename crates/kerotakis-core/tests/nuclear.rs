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

// ── th-122: the block that warms itself ──────────────────────────────
//
// "Can a block of uranium warm itself through radioactive decay?" is a
// question with a trap in it. The answer is yes, and it is yes by six
// thousandths of a degree in a day — small enough that a test asserting
// only the SIGN of ΔT would pass with the energy per decay wrong by any
// factor at all. So these tests pin the energy, and the temperature is
// checked as its consequence rather than as the claim.

/// 1 g of uranium is 1/238.029 mol. Its U-238 decays at λ = ln2/4.468 Gy,
/// which over 86 400 s is 1.075e9 decays — a specific activity of 12 440
/// Bq/g, which is the textbook figure for U-238 and is where this
/// arithmetic can be checked from outside. Each α leaves 4.270 MeV in the
/// metal, so the day deposits 7.35e-4 J into a heat capacity of 0.1162
/// J/K, and the block ends 6.3 mK warmer.
#[test]
fn a_block_of_uranium_warms_itself_and_the_number_is_why() {
    let moles = 1.0 / 238.029;
    let heat = nuclide::bulk_decay_heat("uranium", moles, 86_400.0).expect("uranium is bulk");

    let bq_per_gram = heat.decays / 86_400.0;
    assert!(
        (bq_per_gram - 12_440.0).abs() < 60.0,
        "U-238's specific activity is about 12 440 Bq/g, got {bq_per_gram}"
    );
    assert!(
        (heat.energy_j - 7.352e-4).abs() / 7.352e-4 < 0.01,
        "a day deposits about 7.35e-4 J per gram, got {}",
        heat.energy_j
    );
    // The energy is the decay count times the curated energy per decay,
    // and nothing else. If this identity ever stops holding, something has
    // started adding energy the ledger did not count.
    let per_decay_mev = heat.energy_j / heat.decays / nuclide::JOULES_PER_MEV;
    assert!(
        (per_decay_mev - 4.270).abs() < 1e-6,
        "the heat is 4.270 MeV per decay exactly, got {per_decay_mev}"
    );
}

/// The same thing through the bench, because a model nothing runs is not a
/// capability. `wait 24h` must book the heat, name it, and move the
/// thermometer.
#[test]
fn the_bench_books_the_decay_heat_and_the_thermometer_moves() {
    let mut bench = Bench::new();
    bench
        .step(script::parse_op("add v1 uranium 1g").unwrap().unwrap())
        .expect("uranium is on the shelf");
    let before = bench.vessel(VesselId(0)).unwrap().temperature.0;
    let events = bench
        .step(script::parse_op("wait 24h").unwrap().unwrap())
        .expect("a day passes");

    let energy = events
        .iter()
        .find_map(|e| match e {
            Event::ReactionHeatReleased {
                reaction, energy_j, ..
            } if reaction == "radioactive-decay" => Some(*energy_j),
            _ => None,
        })
        .unwrap_or_else(|| panic!("decay heat is booked and named: {events:?}"));
    assert!(
        (energy - 7.352e-4).abs() / 7.352e-4 < 0.01,
        "{energy} J in a day"
    );

    let after = bench.vessel(VesselId(0)).unwrap().temperature.0;
    let rise = after - before;
    assert!(
        (rise - 6.33e-3).abs() < 2e-4,
        "an adiabatic gram of uranium warms about 6.3 mK in a day, got {rise}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::TemperatureChanged { .. })),
        "a temperature that moved is a temperature that is reported"
    );
}

/// Every radioactive row either states the energy one of its decays leaves
/// behind, with a source, or states why it does not. A silent zero would be
/// indistinguishable from a decay that releases nothing.
#[test]
fn every_radioactive_row_accounts_for_its_energy() {
    for data in nuclide::TEACHING_NUCLIDES {
        let Some(decay) = data.decay.as_ref() else {
            continue;
        };
        match &decay.deposits {
            nuclide::Deposited::Mev { mev, source } => {
                assert!(*mev > 0.0, "{}: a stated energy is positive", data.nuclide);
                assert!(
                    !source.trim().is_empty(),
                    "{}: a stated energy carries its source",
                    data.nuclide
                );
                // The mass defect bounds it: what the sample keeps cannot
                // exceed what the transition released. For β⁻ the mean
                // electron energy is far below Q because the neutrino
                // leaves, and this check is what stops an endpoint energy
                // being pasted in by mistake.
                if let Some(daughter) = nuclide::lookup_notation(decay.daughter) {
                    let mut products = daughter.mass_u;
                    if decay.mode == DecayMode::Alpha {
                        products += nuclide::lookup_notation("He-4").expect("alpha").mass_u;
                    }
                    let q_mev = (data.mass_u - products) * 931.494;
                    if q_mev > 0.0 {
                        assert!(
                            *mev <= q_mev + 1e-6,
                            "{}: deposits {mev} MeV but Q is only {q_mev}",
                            data.nuclide
                        );
                    }
                    if decay.mode == DecayMode::BetaMinus {
                        assert!(
                            *mev < 0.6 * q_mev,
                            "{}: a β⁻ mean is well under its endpoint {q_mev}, got {mev}",
                            data.nuclide
                        );
                    }
                }
            }
            nuclide::Deposited::NotCurated(why) => assert!(
                why.len() > 40,
                "{}: an absent energy is explained, not merely absent",
                data.nuclide
            ),
        }
    }
}

/// The refusal, and it is a real one: Tc-99m's transition is almost all
/// γ, the photon leaves a bench-scale sample, and no reviewed split of the
/// internal-conversion remainder is recorded. So it books no heat, rather
/// than a plausible-looking fraction of 140 keV.
#[test]
fn a_row_with_no_reviewed_energy_books_none() {
    let tc = nuclide::lookup_notation("Tc-99m").expect("curated");
    let decay = tc.decay.as_ref().expect("radioactive");
    assert!(matches!(decay.deposits, nuclide::Deposited::NotCurated(_)));
    assert_eq!(decay.deposits.mev(), 0.0);

    let mut bench = Bench::new();
    spike(&mut bench, "Tc-99m", 1e-9);
    let events = bench.step(Operator::Wait { seconds: 21_624.0 }).unwrap();
    assert!(
        events.iter().any(|e| matches!(e, Event::Decayed { .. })),
        "the isomer still decays"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionHeatReleased { .. })),
        "and it books no heat it cannot justify: {events:?}"
    );
}

/// The boundary the answer depends on, written as a test so it cannot be
/// quietly widened. A block of uranium old enough for its whole series to
/// have reached secular equilibrium deposits about 51.7 MeV per U-238
/// atom — twelve times what this bench books — because every daughter down
/// to lead is decaying alongside it. The ledger stops at Th-234, so the
/// bench must under-report, and the Th-234 row says so in its own source.
#[test]
fn the_series_beyond_thorium_is_declared_absent_not_included() {
    let u = nuclide::lookup_notation("U-238").expect("curated");
    let decay = u.decay.as_ref().expect("radioactive");
    assert_eq!(decay.daughter, "Th-234");
    assert!(
        decay.deposits.mev() < 5.0,
        "the first α is booked, not the 51.7 MeV series total"
    );
    let th = nuclide::lookup_notation("Th-234").expect("the daughter is curated");
    assert!(th.decay.is_none(), "the chain stops here on purpose");
    assert!(
        th.source.contains("secular equilibrium"),
        "and the row that stops it says what stopping costs"
    );
    // The isotopic assumption is written down beside the key it applies to.
    let bulk = nuclide::bulk_radionuclide("uranium").expect("bridged");
    assert_eq!(bulk.nuclide, "U-238");
    assert!(bulk.isotopics.contains("U-234"));
}

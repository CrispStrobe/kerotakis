//! Conservation invariants under random operator sequences (PLAN.md,
//! "Testing is part of the architecture"): matter is never created or
//! destroyed by any operator, and energy changes only by exactly the heat
//! that operators put in or take out.

use kerotakis_core::units::Liters;
use kerotakis_core::*;
use proptest::prelude::*;

/// The sensible heat matter carries in with it when it arrives at
/// `celsius`, J, counted as deliberate input to the budget.
///
/// This used to be `n * Cp * (T_in - T_ref)` with `Cp` the room-temperature
/// constant. That is the same number only while the heat capacity is flat,
/// and water's is not: it has a minimum near 35 °C and reaches 76.17
/// J/(mol·K) at 0 °C, so a mole of ice-cold water arriving carries 0.23 %
/// more away from 25 °C than the rectangle said. The budget has to measure
/// the same quantity the bench holds, or the test compares two different
/// claims and calls the difference a leak.
fn arriving_enthalpy(key: &str, moles: f64, celsius: f64) -> f64 {
    let data = kerotakis_core::species::lookup(&SpeciesId::new(key)).expect("curated species");
    moles
        * kerotakis_core::states::enthalpy_between(
            data,
            data.standard_phase,
            Kelvin::STANDARD.0,
            celsius + 273.15,
        )
}

#[derive(Debug, Clone)]
enum RandOp {
    AddWater {
        celsius: f64,
        moles: f64,
    },
    AddEthanol {
        celsius: f64,
        moles: f64,
    },
    AddSalt {
        moles: f64,
    },
    Heat {
        joules: f64,
    },
    Cool {
        joules: f64,
    },
    NewVessel,
    Decant {
        from: usize,
        to: usize,
        fraction: f64,
    },
    Distil {
        from: usize,
        to: usize,
        fraction: f64,
    },
    Measure,
    Dilute {
        volume_ml: f64,
    },
    /// EXP-39: the burette. In this bench no aqueous solver is wired, so
    /// the loop deposits exactly one increment and then says honestly
    /// that it cannot read a pH — which is what `lessons/titration.lab`'s
    /// golden has pinned since CAP-12. That makes the delivered amount
    /// exactly `concentration x step` of titrant plus the carrier water
    /// of one step volume, for every endpoint, and *that* is the
    /// invariant worth holding: a burette delivers what it says it
    /// delivers, and changing what stops it must not change what it
    /// pours.
    Titrate {
        concentration: f64,
        step_ml: f64,
        endpoint: u8,
    },
}

fn rand_op() -> impl Strategy<Value = RandOp> {
    prop_oneof![
        (0.0f64..100.0, 0.001f64..50.0)
            .prop_map(|(celsius, moles)| RandOp::AddWater { celsius, moles }),
        (0.0f64..70.0, 0.001f64..20.0)
            .prop_map(|(celsius, moles)| RandOp::AddEthanol { celsius, moles }),
        (0.001f64..5.0).prop_map(|moles| RandOp::AddSalt { moles }),
        (0.1f64..50_000.0).prop_map(|joules| RandOp::Heat { joules }),
        (0.1f64..20_000.0).prop_map(|joules| RandOp::Cool { joules }),
        Just(RandOp::NewVessel),
        (0usize..4, 0usize..4, 0.0f64..1.0).prop_map(|(from, to, fraction)| RandOp::Decant {
            from,
            to,
            fraction
        }),
        (0usize..4, 0usize..4, 0.0f64..1.0).prop_map(|(from, to, fraction)| RandOp::Distil {
            from,
            to,
            fraction
        }),
        Just(RandOp::Measure),
        (1.0f64..500.0).prop_map(|volume_ml| RandOp::Dilute { volume_ml }),
        (0.01f64..2.0, 0.05f64..5.0, 0u8..3).prop_map(|(concentration, step_ml, endpoint)| {
            RandOp::Titrate {
                concentration,
                step_ml,
                endpoint,
            }
        }),
    ]
}

/// Applies one random op; returns the net heat deliberately put into the
/// bench by this op (J), or None if the op was rejected (rejections must not
/// mutate — checked separately below).
fn apply(bench: &mut Bench, op: &RandOp) -> Option<f64> {
    let vessel_ids: Vec<VesselId> = bench.vessels.iter().map(|v| v.id).collect();
    let pick = |i: usize| vessel_ids[i % vessel_ids.len()];
    let result = match op {
        RandOp::AddWater { celsius, moles } => bench
            .step(Operator::Add {
                vessel: pick(0),
                species: SpeciesId::new("water"),
                moles: Moles(*moles),
                at: Some(Kelvin::from_celsius(*celsius)),
            })
            .map(|_| arriving_enthalpy("water", *moles, *celsius)),
        RandOp::AddEthanol { celsius, moles } => bench
            .step(Operator::Add {
                vessel: pick(1),
                species: SpeciesId::new("ethanol"),
                moles: Moles(*moles),
                at: Some(Kelvin::from_celsius(*celsius)),
            })
            .map(|_| arriving_enthalpy("ethanol", *moles, *celsius)),
        RandOp::AddSalt { moles } => bench
            .step(Operator::Add {
                vessel: pick(2),
                species: SpeciesId::new("NaCl"),
                moles: Moles(*moles),
                at: None,
            })
            .map(|_| 0.0),
        RandOp::Heat { joules } => {
            let v = pick(0);
            // A burner has a temperature of its own, so what is asked for
            // and what crosses are two different numbers: a tenth of a mole
            // of water cannot absorb 50 kJ, because long before that it is
            // as hot as the flame. Charge the ledger with what the bench
            // says it delivered — and *that* claim is the one under test,
            // since the enthalpy on the other side of the balance is
            // computed from the vessel, not from this event.
            bench
                .step(Operator::Heat {
                    vessel: v,
                    energy: Joules(*joules),
                    source: None,
                })
                .map(|events| {
                    events
                        .iter()
                        .find_map(|event| match event {
                            Event::EnergyTransferred {
                                vessel,
                                heating: true,
                                delivered_j,
                                ..
                            } if *vessel == v => Some(*delivered_j),
                            _ => None,
                        })
                        .unwrap_or(0.0)
                })
        }
        RandOp::Cool { joules } => {
            let v = pick(1);
            // Cooling clamps at 0 K; compute the heat actually removed.
            let before = bench.vessel(v).unwrap().enthalpy().0;
            bench
                .step(Operator::Cool {
                    vessel: v,
                    energy: Joules(*joules),
                })
                .map(|_| bench.vessel(v).unwrap().enthalpy().0 - before)
        }
        RandOp::NewVessel => bench.step(Operator::NewVessel { kind: None }).map(|_| 0.0),
        RandOp::Decant { from, to, fraction } => {
            let (f, t) = (pick(*from), pick(*to));
            if f == t {
                return None;
            }
            bench
                .step(Operator::Decant {
                    from: f,
                    to: t,
                    fraction: *fraction,
                })
                .map(|_| 0.0)
        }
        RandOp::Distil { from, to, fraction } => {
            let (f, t) = (pick(*from), pick(*to));
            if f == t {
                return None;
            }
            // Externally powered, like `evaporate`: matter moves, the
            // ledger's heat does not.
            bench
                .step(Operator::Distil {
                    from: f,
                    to: t,
                    fraction: Some(*fraction),
                    energy: None,
                    stages: 1 + (*from as u32 % 3),
                })
                .map(|_| 0.0)
        }
        RandOp::Measure => bench
            .step(Operator::Measure {
                vessel: pick(3),
                instrument: Instrument::Thermometer,
            })
            .map(|_| 0.0),
        RandOp::Dilute { volume_ml } => bench
            .step(Operator::Dilute {
                vessel: pick(0),
                volume: Liters(*volume_ml / 1000.0),
            })
            .map(|_| 0.0),
        RandOp::Titrate {
            concentration,
            step_ml,
            endpoint,
        } => bench
            .step(Operator::Titrate {
                vessel: pick(0),
                titrant: SpeciesId::new("NaOH"),
                concentration: *concentration,
                step: Liters(*step_ml / 1000.0),
                target_ph: 7.0,
                max_steps: 8,
                endpoint: match endpoint {
                    0 => kerotakis_core::ops::Endpoint::Ph,
                    1 => kerotakis_core::ops::Endpoint::Pe {
                        compare: kerotakis_core::ops::Compare::Above,
                        value: 8.0,
                    },
                    _ => kerotakis_core::ops::Endpoint::ColourPersists,
                },
            })
            // Titrant and its carrier water enter at the standard
            // temperature and the adiabatic mix conserves enthalpy, so
            // no heat is deliberately added.
            .map(|_| 0.0),
    };
    result.ok()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Mass balance: for every species, total moles across the bench equal
    /// exactly what was added, no matter the operator sequence.
    #[test]
    fn mass_is_conserved(ops in proptest::collection::vec(rand_op(), 1..40)) {
        let mut bench = Bench::new();
        let mut added: std::collections::BTreeMap<&str, f64> = Default::default();
        for op in &ops {
            let before: Vec<f64> = ["water", "ethanol", "NaCl", "NaOH"]
                .iter()
                .map(|k| bench.total_moles(&SpeciesId::new(k)).0)
                .collect();
            let accepted = apply(&mut bench, op).is_some();
            if accepted {
                if let RandOp::AddWater { moles, .. } = op {
                    *added.entry("water").or_default() += moles;
                }
                if let RandOp::AddEthanol { moles, .. } = op {
                    *added.entry("ethanol").or_default() += moles;
                }
                if let RandOp::AddSalt { moles } = op {
                    *added.entry("NaCl").or_default() += moles;
                }
                if let RandOp::Dilute { volume_ml } = op {
                    let data = kerotakis_core::species::lookup(&SpeciesId::new("water")).unwrap();
                    *added.entry("water").or_default() +=
                        data.moles_from_liters(Liters(*volume_ml / 1000.0)).0;
                }
                if let RandOp::Titrate { concentration, step_ml, .. } = op {
                    let step = Liters(*step_ml / 1000.0);
                    let data = kerotakis_core::species::lookup(&SpeciesId::new("water")).unwrap();
                    *added.entry("NaOH").or_default() += concentration * step.0;
                    *added.entry("water").or_default() += data.moles_from_liters(step).0;
                }
            } else {
                // A rejected op must not have mutated anything.
                for (i, k) in ["water", "ethanol", "NaCl", "NaOH"].iter().enumerate() {
                    prop_assert!((bench.total_moles(&SpeciesId::new(k)).0 - before[i]).abs() < 1e-12);
                }
            }
        }
        for (k, expected) in &added {
            let got = bench.total_moles(&SpeciesId::new(k)).0;
            prop_assert!(
                (got - expected).abs() < 1e-9 * expected.max(1.0),
                "species {k}: expected {expected} mol, bench holds {got} mol"
            );
        }
    }

    /// Energy balance: bench enthalpy changes only by the heat deliberately
    /// put in (Heat/Cool, or matter entering warmer/colder than reference).
    #[test]
    fn energy_is_conserved(ops in proptest::collection::vec(rand_op(), 1..40)) {
        let mut bench = Bench::new();
        let mut budget = 0.0f64;
        // The GROSS energy the script handled, alongside the net it ends up
        // holding. These are different numbers - by two or three orders of
        // magnitude on any script that warms a beaker and then pours
        // something cold into it - and only one of them is the size of the
        // arithmetic that produced the residue. See the bound.
        let mut gross = 0.0f64;
        for op in &ops {
            if let Some(q) = apply(&mut bench, op) {
                budget += q;
                gross += q.abs();
            }
        }
        let h = bench.total_enthalpy().0;

        // Two terms, because the residue has two sources and they scale
        // with different things.
        //
        // FIRST, three parts in a million of the energy the script MOVED.
        // This was a part in a hundred thousand of the energy the script
        // ends up HOLDING, and that is the wrong quantity, because `budget`
        // is a signed sum that near-cancellation can drive to nothing while
        // the arithmetic behind it handled tens of kilojoules. The run that
        // made this test flaky on main added 9.49 mol of ethanol, spent
        // 32.2 kJ of burner on it and then poured in 31.7 mol of water at
        // 3.8 C: it handled 102 kJ and netted 414 J, and scaling by the
        // 414 J asked the ledger for a precision the 102 kJ never had.
        // Over 4096 random scripts (up to 1.18 MJ of traffic, up to 39
        // operators) the worst residue against the net was 1.7e-6 - on a
        // script that moved 204 kJ and netted 42.7 J, six times inside the
        // old bound and heading the wrong way - while the worst against the
        // gross was 3.5e-8. `gross` is also an upper bound on the largest
        // enthalpy the bench ever held, which is the quantity actually
        // being differenced, so it is the right ruler in both readings.
        // Three parts in a million is the same strength as the old bound on
        // a typical script, where gross is about three times net; what it
        // removes is the cliff, not the rigour.
        //
        // SECOND, a floor that scales with the MATTER differenced rather
        // than the energy moved, because the two come apart: water poured
        // in at 25.0 C moves no energy at all and is still differenced.
        // Liquid water's NASA-9 fit sums an antiderivative of terms ~1.2e9
        // J/mol cancelling to -9.2e8, so a double gives up ~2.6e-7 J per
        // mole per difference (#509; ice's fit gives up 4e-11 and
        // nitrogen's 4e-12 - it is that one curve's conditioning, not the
        // ledger). The worst case in the sweep is exactly this: 33.3 mol of
        // near-room-temperature water, 306 J of traffic, 1.08e-5 J left
        // over, which is 3.2e-7 J per mole and would sit only 28x inside a
        // purely relative bound. Five microjoules per mole is twenty times
        // the per-difference figure, leaving room for the several
        // differences an operator takes. PLAN.md's `(T - T_mid)`
        // reformulation would buy most of this term back.
        //
        // With both terms the tightest margin over those 4096 scripts is
        // 100x, and the next tightest 228x.
        //
        // The 0.0157 J that failed job 101869813421 was NOT this floor and
        // is not tolerated here. Dissolving a solid in an organic solvent
        // moved the portion from its own Cp(T) curve onto the flat registry
        // constant and rewrote the vessel's enthalpy for free; that is a
        // leak, and it is fixed in `nonaqueous.rs`. Its seed is pinned in
        // `conservation.proptest-regressions` so it is re-run every time.
        let water = bench.total_moles(&SpeciesId::new("water")).0;
        let tolerance = 3e-6 * gross.max(1.0) + 5e-6 * water;
        prop_assert!(
            (h - budget).abs() < tolerance,
            "bench enthalpy {h} J diverged from heat budget {budget} J by more \
             than {tolerance} J, across {gross} J of gross energy handled"
        );
    }
}

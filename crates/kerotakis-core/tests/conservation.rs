//! Conservation invariants under random operator sequences (PLAN.md,
//! "Testing is part of the architecture"): matter is never created or
//! destroyed by any operator, and energy changes only by exactly the heat
//! that operators put in or take out.

use kerotakis_core::units::Liters;
use kerotakis_core::*;
use proptest::prelude::*;

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
            .map(|_| {
                // Matter entering at T_in brings enthalpy n·Cp·(T_in − T_ref)
                // with it; count it as deliberate input.
                *moles * 75.3 * (celsius + 273.15 - 298.15)
            }),
        RandOp::AddEthanol { celsius, moles } => bench
            .step(Operator::Add {
                vessel: pick(1),
                species: SpeciesId::new("ethanol"),
                moles: Moles(*moles),
                at: Some(Kelvin::from_celsius(*celsius)),
            })
            .map(|_| *moles * 112.3 * (celsius + 273.15 - 298.15)),
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
        for op in &ops {
            if let Some(q) = apply(&mut bench, op) {
                budget += q;
            }
        }
        let h = bench.total_enthalpy().0;
        prop_assert!(
            (h - budget).abs() < 1e-6 * budget.abs().max(1.0),
            "bench enthalpy {h} J diverged from heat budget {budget} J"
        );
    }
}

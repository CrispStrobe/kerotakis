//! Measured aqueous coordinates travel with the aliquot but are not matter.
use kerotakis_core::vessel::StepStart;
use kerotakis_core::*;

#[derive(Default)]
struct Capture {
    starts: Vec<(VesselId, StepStart, bool)>,
}
impl Equilibrator for Capture {
    fn name(&self) -> &'static str {
        "capture-transfer-start"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.starts.push((
            vessel.id,
            vessel.step_start.clone().unwrap(),
            vessel.resolved.valid,
        ));
        Ok(vec![])
    }
}

fn fixture_vessel_mut(bench: &mut Bench, id: VesselId) -> &mut Vessel {
    bench
        .vessels
        .iter_mut()
        .find(|vessel| vessel.id == id)
        .expect("fixture vessel exists")
}

fn fixture() -> Bench {
    let mut bench = Bench::new();
    bench.vessels.push(Vessel::new(VesselId(1), "receiver"));
    let source = fixture_vessel_mut(&mut bench, VesselId(0));
    source.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    source.deposit(SpeciesId::new("Mg+2"), Moles(1e-5), Phase::Aqueous);
    source.deposit(
        SpeciesId::new("base_equivalents"),
        Moles(2e-5),
        Phase::Aqueous,
    );
    source.deposit(SpeciesId::new("Mg(OH)2"), Moles(0.002), Phase::Solid);
    // A measured distribution differs from the analytical base coordinate.
    source.free_hydroxide = 1.8e-5;
    source.free_proton = 4e-12;
    source.solute_charge = 2e-5;
    source.resolved.valid = true;
    bench
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-14, "{a} != {b}");
}
fn amount(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

#[test]
fn full_filter_moves_measured_baseline_once_and_not_extra_ions() {
    let mut bench = fixture();
    let mut capture = Capture::default();
    bench
        .step_with(
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(1),
            },
            &mut capture,
            &PermissiveScreen,
        )
        .unwrap();
    let source = bench.vessel(VesselId(0)).unwrap();
    let receiver = bench.vessel(VesselId(1)).unwrap();
    near(source.free_hydroxide, 0.0);
    near(source.free_proton, 0.0);
    near(source.solute_charge, 0.0);
    near(receiver.free_hydroxide, 1.8e-5);
    near(receiver.free_proton, 4e-12);
    near(receiver.solute_charge, 2e-5);
    near(amount(source, "Mg(OH)2"), 0.002);
    near(amount(receiver, "Mg(OH)2"), 0.0);
    near(amount(receiver, "base_equivalents"), 2e-5);
    near(amount(receiver, "water"), 5.0);
    assert_eq!(
        receiver.contents.len(),
        3,
        "no extra measured H+/OH- portions"
    );
    let (_, start, valid) = capture
        .starts
        .iter()
        .find(|(id, _, _)| *id == VesselId(1))
        .unwrap();
    near(start.free_hydroxide, 1.8e-5);
    near(start.free_proton, 4e-12);
    near(start.solute_charge, 2e-5);
    assert!(
        !valid,
        "a source solution/cache was not copied as a current answer"
    );
}

#[test]
fn proportional_decants_to_nonempty_receivers_keep_complement_and_add_once() {
    for fraction in [0.0, 0.25, 1.0] {
        let mut bench = fixture();
        let receiver = fixture_vessel_mut(&mut bench, VesselId(1));
        receiver.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
        receiver.deposit(SpeciesId::new("Cl-"), Moles(1e-4), Phase::Aqueous);
        receiver.deposit(SpeciesId::new("H+"), Moles(1e-4), Phase::Aqueous);
        receiver.free_proton = 1e-4;
        receiver.free_hydroxide = 1e-12;
        receiver.solute_charge = -1e-4;
        let mut capture = Capture::default();
        bench
            .step_with(
                Operator::Decant {
                    from: VesselId(0),
                    to: VesselId(1),
                    fraction,
                },
                &mut capture,
                &PermissiveScreen,
            )
            .unwrap();
        let source = bench.vessel(VesselId(0)).unwrap();
        let receiver = bench.vessel(VesselId(1)).unwrap();
        near(source.free_hydroxide, 1.8e-5 * (1.0 - fraction));
        near(receiver.free_hydroxide, 1e-12 + 1.8e-5 * fraction);
        near(source.free_proton, 4e-12 * (1.0 - fraction));
        near(receiver.free_proton, 1e-4 + 4e-12 * fraction);
        near(source.solute_charge, 2e-5 * (1.0 - fraction));
        near(receiver.solute_charge, -1e-4 + 2e-5 * fraction);
        near(amount(source, "water") + amount(receiver, "water"), 6.0);
        near(amount(source, "Mg+2") + amount(receiver, "Mg+2"), 1e-5);
        near(amount(source, "Mg(OH)2"), 0.002);
        assert_eq!(
            receiver
                .contents
                .iter()
                .filter(|p| p.species.0 == "H+")
                .count(),
            1
        );
    }
}

#[test]
fn acid_and_neutral_filtrates_do_not_become_analytical_free_base() {
    for (oh, h, charge) in [(1e-12, 1e-4, -1e-4), (1e-8, 1e-8, 0.0)] {
        let mut bench = fixture();
        let source = fixture_vessel_mut(&mut bench, VesselId(0));
        source
            .contents
            .retain(|p| p.species.0 == "water" || p.phase == Phase::Solid);
        if charge < 0.0 {
            source.deposit(SpeciesId::new("Cl-"), Moles(-charge), Phase::Aqueous);
            source.deposit(SpeciesId::new("H+"), Moles(-charge), Phase::Aqueous);
        }
        source.free_hydroxide = oh;
        source.free_proton = h;
        source.solute_charge = charge;
        let mut capture = Capture::default();
        bench
            .step_with(
                Operator::Filter {
                    from: VesselId(0),
                    to: VesselId(1),
                },
                &mut capture,
                &PermissiveScreen,
            )
            .unwrap();
        let receiver = bench.vessel(VesselId(1)).unwrap();
        near(receiver.free_hydroxide, oh);
        near(receiver.free_proton, h);
        near(receiver.solute_charge, charge);
        near(amount(receiver, "base_equivalents"), 0.0);
    }
}

#[test]
fn filter_preserves_nontransferred_owned_gas_and_stationary_surface() {
    let mut bench = fixture();
    let source = fixture_vessel_mut(&mut bench, VesselId(0));
    source.headspace = Headspace::Sealed {
        volume: Liters(0.1),
    };
    source.deposit(SpeciesId::new("N2"), Moles(0.001), Phase::Gas);
    source.surfaces.push(SurfaceSites {
        label: "retained interface".into(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(0.1),
        specific_area_m2_per_g: 10.0,
        strong_capacity: Moles(1e-6),
        weak_capacity: Moles(1e-5),
        occupancy: vec![],
        water_release: Moles(0.0),
    });
    let mut capture = Capture::default();
    let events = bench
        .step_with(
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(1),
            },
            &mut capture,
            &PermissiveScreen,
        )
        .unwrap();
    let source = bench.vessel(VesselId(0)).unwrap();
    let receiver = bench.vessel(VesselId(1)).unwrap();
    near(amount(source, "N2"), 0.001);
    near(amount(receiver, "N2"), 0.0);
    assert_eq!(source.surfaces.len(), 1);
    assert!(receiver.surfaces.is_empty());
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::GasEvolved { .. } | Event::GasAbsorbed { .. })));
}

struct Veto;
impl SafetyScreen for Veto {
    fn assess(&self, vessel: &Vessel) -> SafetyVerdict {
        if vessel.id != VesselId(0) && amount(vessel, "water") > 0.0 {
            SafetyVerdict::Veto {
                reason: "test prospective receiver refusal".into(),
            }
        } else {
            SafetyVerdict::Allow
        }
    }
}
#[test]
fn refused_pours_keep_matter_and_measured_fields_atomic() {
    for op in [
        Operator::Filter {
            from: VesselId(0),
            to: VesselId(1),
        },
        Operator::Filter {
            from: VesselId(0),
            to: VesselId(2),
        },
        Operator::Decant {
            from: VesselId(0),
            to: VesselId(1),
            fraction: 0.25,
        },
    ] {
        let mut bench = fixture();
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let mut capture = Capture::default();
        let events = bench.step_with(op, &mut capture, &Veto).unwrap();
        assert!(events.iter().any(|e| matches!(e, Event::SafetyVeto { .. })));
        assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
        assert!(capture.starts.is_empty());
    }
}

#[test]
fn newly_created_filtrate_receiver_gets_the_same_baseline() {
    let mut bench = fixture();
    let mut capture = Capture::default();
    bench
        .step_with(
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(2),
            },
            &mut capture,
            &PermissiveScreen,
        )
        .unwrap();
    let receiver = bench.vessel(VesselId(2)).unwrap();
    near(receiver.free_hydroxide, 1.8e-5);
    near(receiver.free_proton, 4e-12);
    near(receiver.solute_charge, 2e-5);
    near(amount(receiver, "water"), 5.0);
}

struct FailReceiver;
impl Equilibrator for FailReceiver {
    fn name(&self) -> &'static str {
        "intentional-receiver-failure"
    }
    fn applies(&self, v: &Vessel) -> bool {
        v.id == VesselId(1)
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.free_hydroxide = 4.0;
        v.deposit(SpeciesId::new("Na+"), Moles(1.0), Phase::Aqueous);
        Err(SolveError::NotConverged {
            reason: None,
            solver: self.name().into(),
            detail: "deliberate trial failure".into(),
        })
    }
}
#[test]
fn solver_failure_rolls_back_trial_but_preserves_committed_transfer_baseline() {
    let mut bench = fixture();
    let mut solver = SolverStack::new(vec![Box::new(FailReceiver)]);
    let events = bench
        .step_with(
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(1),
            },
            &mut solver,
            &PermissiveScreen,
        )
        .unwrap();
    let receiver = bench.vessel(VesselId(1)).unwrap();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    near(receiver.free_hydroxide, 1.8e-5);
    near(amount(receiver, "Na+"), 0.0);
    near(amount(receiver, "water"), 5.0);
    assert!(receiver.solution.is_none());
    assert!(!receiver.resolved.valid);
}

#[test]
fn metadata_overflow_refuses_and_restores_both_vessels() {
    let mut bench = fixture();
    fixture_vessel_mut(&mut bench, VesselId(0)).free_hydroxide = f64::MAX;
    fixture_vessel_mut(&mut bench, VesselId(1)).free_hydroxide = f64::MAX;
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let mut capture = Capture::default();
    assert!(bench
        .step_with(
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(1)
            },
            &mut capture,
            &PermissiveScreen
        )
        .is_err());
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(capture.starts.is_empty());
}

#[test]
fn negative_measured_source_cannot_be_hidden_by_cancellation_in_receiver() {
    let mut bench = fixture();
    fixture_vessel_mut(&mut bench, VesselId(0)).free_hydroxide = -1.0;
    fixture_vessel_mut(&mut bench, VesselId(1)).free_hydroxide = 2.0;
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let mut capture = Capture::default();
    assert!(bench
        .step_with(
            Operator::Filter {
                from: VesselId(0),
                to: VesselId(1)
            },
            &mut capture,
            &PermissiveScreen
        )
        .is_err());
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(capture.starts.is_empty());
}

#[test]
fn no_water_carrier_does_not_migrate_old_aqueous_measurements() {
    for keep_dry_ions in [false, true] {
        let mut bench = fixture();
        let source = fixture_vessel_mut(&mut bench, VesselId(0));
        source
            .contents
            .retain(|p| p.phase == Phase::Solid || (keep_dry_ions && p.phase == Phase::Aqueous));
        // Deliberately retain last-measurement fields: they are not a fresh
        // aqueous answer, nor permission to invent a liquid carrier.
        let mut capture = Capture::default();
        bench
            .step_with(
                Operator::Filter {
                    from: VesselId(0),
                    to: VesselId(1),
                },
                &mut capture,
                &PermissiveScreen,
            )
            .unwrap();
        let source = bench.vessel(VesselId(0)).unwrap();
        let receiver = bench.vessel(VesselId(1)).unwrap();
        near(source.free_hydroxide, 1.8e-5);
        near(receiver.free_hydroxide, 0.0);
        near(amount(receiver, "water"), 0.0);
        assert!(receiver.solution.is_none());
    }
}

/// TH-015: `mix` withdraws a common fraction of every liquid/aqueous portion
/// of each source, so the measured hydroxide/proton coordinates must travel
/// with it. Left behind, mixing caustic soda into hydrochloric acid met no
/// hydroxide in the receiver's StepStart and charged no neutralisation heat.
#[test]
fn mix_carries_each_sources_measured_baseline_into_the_receiver() {
    for (fraction_a, fraction_b) in [(1.0, 1.0), (0.5, 0.25), (0.0, 1.0)] {
        let mut bench = fixture();
        bench.vessels.push(Vessel::new(VesselId(2), "acid"));
        bench.vessels.push(Vessel::new(VesselId(3), "mixed"));
        let acid = fixture_vessel_mut(&mut bench, VesselId(2));
        acid.deposit(SpeciesId::new("water"), Moles(2.775), Phase::Liquid);
        acid.deposit(SpeciesId::new("Cl-"), Moles(0.005), Phase::Aqueous);
        acid.free_proton = 0.005;
        acid.free_hydroxide = 1e-14;
        acid.solute_charge = -0.005;
        let mut capture = Capture::default();
        bench
            .step_with(
                Operator::Mix {
                    a: VesselId(0),
                    b: VesselId(2),
                    into: VesselId(3),
                    fraction_a,
                    fraction_b,
                },
                &mut capture,
                &PermissiveScreen,
            )
            .unwrap();
        let mixed = bench.vessel(VesselId(3)).unwrap();
        near(
            mixed.free_hydroxide,
            1.8e-5 * fraction_a + 1e-14 * fraction_b,
        );
        near(mixed.free_proton, 4e-12 * fraction_a + 0.005 * fraction_b);
        near(mixed.solute_charge, 2e-5 * fraction_a - 0.005 * fraction_b);
        let base = bench.vessel(VesselId(0)).unwrap();
        near(base.free_hydroxide, 1.8e-5 * (1.0 - fraction_a));
        let acid = bench.vessel(VesselId(2)).unwrap();
        near(acid.free_proton, 0.005 * (1.0 - fraction_b));
        let (_, start, _) = capture
            .starts
            .iter()
            .find(|(id, _, _)| *id == VesselId(3))
            .unwrap();
        near(start.free_hydroxide, mixed.free_hydroxide);
    }
}

//! Source-informed contracts frozen before the refusal/preflight repair or execution.
use kerotakis_core::ops::NotModelledCause;
use kerotakis_core::*;

#[derive(Default)]
struct CountingMutator(Vec<VesselId>);
impl Equilibrator for CountingMutator {
    fn name(&self) -> &'static str {
        "frozen-refusal-mutator"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.0.push(vessel.id);
        vessel.temperature.0 += 1.0;
        vessel.free_proton += 0.001;
        Ok(vec![])
    }
}

fn physical(bench: &Bench) -> serde_json::Value {
    let mut state = serde_json::to_value(bench).unwrap();
    state.as_object_mut().unwrap().remove("log");
    state
}

fn prepared(fresh: bool) -> Bench {
    let mut bench = Bench::new();
    if !fresh {
        bench
            .vessels
            .push(Vessel::new(VesselId(2), "existing receiver"));
    }
    for vessel in &mut bench.vessels {
        vessel.deposit(SpeciesId::new("water"), Moles(4.0), Phase::Liquid);
        vessel.free_proton = 1e-7;
        vessel.solution = Some(SolutionInfo {
            solvent_activity: None,
            scope: Default::default(),
            solvent_kg: Some(0.072),
            pe: None,
            redox: vec![],
            ph: 7.0,
            ionic_strength: 0.001,
            species: vec![],
            provenance: None,
        });
    }
    bench
}
fn deposit(bench: &mut Bench, species: &str, amount: f64, phase: Phase) {
    bench.vessels[0].deposit(SpeciesId::new(species), Moles(amount), phase);
}
fn diagnostic(events: &[Event], key: &str) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { reason: Some(p), .. } if p.key == key))
}
fn refused(mut bench: Bench, op: Operator, key: &str) {
    let before = physical(&bench);
    let mut solver = CountingMutator::default();
    let events = bench
        .step_with(op, &mut solver, &PermissiveScreen)
        .expect("diagnostic refusal");
    assert!(
        diagnostic(&events, key),
        "expected refusal {key}: {events:?}"
    );
    assert_eq!(events.len(), 1, "refusal retains only its diagnostic");
    assert!(
        solver.0.is_empty(),
        "refusal must not settle: {:?}",
        solver.0
    );
    assert_eq!(
        physical(&bench),
        before,
        "complete state except refusal log"
    );
    assert_eq!(bench.log.len(), 1);
    assert_eq!(
        serde_json::to_value(&bench.log[0].events).unwrap(),
        serde_json::to_value(&events).unwrap()
    );
}
fn drain(fresh: bool, domain: bool) {
    let mut bench = prepared(fresh);
    if domain {
        deposit(&mut bench, "hexane", 4.0, Phase::Liquid);
        deposit(&mut bench, "I2", 0.001, Phase::Aqueous);
        bench.vessels[0].temperature = Kelvin(300.0);
        assert!(
            solve::layered_pair(&bench.vessels[0]).is_some(),
            "domain test must reach layered branch"
        );
    }
    refused(
        bench,
        Operator::Drain {
            from: VesselId(0),
            to: VesselId(2),
        },
        if domain {
            "not-modeled.distribution-coefficient-temperature"
        } else {
            "not-modeled.drain-a-single-phase-liquid"
        },
    );
}
#[test]
fn single_phase_drain_existing_receiver_is_unchanged() {
    drain(false, false);
}
#[test]
fn single_phase_drain_fresh_receiver_is_unchanged() {
    drain(true, false);
}
#[test]
fn unreviewed_drain_existing_receiver_is_unchanged() {
    drain(false, true);
}
#[test]
fn unreviewed_drain_fresh_receiver_is_unchanged() {
    drain(true, true);
}
#[test]
fn water_free_evaporation_is_unchanged() {
    let mut bench = prepared(false);
    bench.vessels[0].withdraw(&SpeciesId::new("water"), Moles(4.0));
    deposit(&mut bench, "ethanol", 1.0, Phase::Liquid);
    refused(
        bench,
        Operator::Evaporate {
            vessel: VesselId(0),
            fraction: 0.5,
        },
        "not-modeled.nothing-to-evaporate",
    );
}
#[test]
fn unsupported_nuclide_is_unchanged() {
    assert!(nuclide::lookup_notation("Xe-135").is_none());
    refused(
        prepared(false),
        Operator::SpikeNuclide {
            vessel: VesselId(0),
            nuclide: "Xe-135".into(),
            moles: Moles(0.001),
        },
        "not-modeled.no-curated-nuclide",
    );
}
#[test]
fn unknown_direct_reaction_is_unchanged() {
    let name = "__unsupported_refusal_contract__";
    assert!(!selectivity::is_selectivity_verb(name));
    assert!(!curated::ORG_REACTIONS.iter().any(|r| r.name == name));
    refused(
        prepared(false),
        Operator::React {
            vessel: VesselId(0),
            reaction: name.into(),
        },
        "not-modeled.no-curated-reaction",
    );
}

fn accepted(bench: &mut Bench, op: Operator) -> Vec<Event> {
    let mut solver = CountingMutator::default();
    let events = bench
        .step_with(op, &mut solver, &PermissiveScreen)
        .expect("supported operation accepts");
    assert!(
        !solver.0.is_empty(),
        "accepted/partially modeled operations must settle"
    );
    events
}
#[test]
fn supported_layered_drain_creates_receiver_and_settles() {
    let mut bench = prepared(true);
    deposit(&mut bench, "hexane", 4.0, Phase::Liquid);
    assert!(solve::layered_pair(&bench.vessels[0]).is_some());
    let events = accepted(
        &mut bench,
        Operator::Drain {
            from: VesselId(0),
            to: VesselId(2),
        },
    );
    let created = events
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::VesselCreated {
                    vessel: VesselId(2)
                }
            )
        })
        .unwrap();
    let success = events
        .iter()
        .position(|e| matches!(e, Event::Drained { .. }))
        .unwrap();
    assert!(created < success);
    assert_eq!(
        bench
            .vessel(VesselId(0))
            .unwrap()
            .moles_of(&SpeciesId::new("water"))
            .0,
        0.0
    );
    assert_eq!(
        bench
            .vessel(VesselId(2))
            .unwrap()
            .moles_of(&SpeciesId::new("water"))
            .0,
        4.0
    );
    assert_eq!(
        bench
            .vessel(VesselId(0))
            .unwrap()
            .moles_of(&SpeciesId::new("hexane"))
            .0,
        4.0
    );
}
#[test]
fn reviewed_iodine_drain_creates_receiver_and_settles() {
    let mut bench = prepared(true);
    deposit(&mut bench, "hexane", 4.0, Phase::Liquid);
    deposit(&mut bench, "I2", 0.001, Phase::Aqueous);
    assert_eq!(bench.vessels[0].temperature.0, 298.15);
    assert!(solve::layered_pair(&bench.vessels[0]).is_some());
    let events = accepted(
        &mut bench,
        Operator::Drain {
            from: VesselId(0),
            to: VesselId(2),
        },
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Partitioned { species, .. } if species.0 == "I2")));
    assert!(events.iter().any(|e| matches!(e, Event::Drained { .. })));
    assert!(!diagnostic(
        &events,
        "not-modeled.distribution-coefficient-temperature"
    ));
    let source = bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new("I2"))
        .0;
    let receiver = bench
        .vessel(VesselId(2))
        .unwrap()
        .moles_of(&SpeciesId::new("I2"))
        .0;
    assert!(source > 0.0 && receiver > 0.0);
    assert!((source + receiver - 0.001).abs() < 1e-15);
}
#[test]
fn partial_evaporation_with_coevaporation_notice_still_settles() {
    let mut bench = prepared(false);
    deposit(&mut bench, "ethanol", 1.0, Phase::Liquid);
    let events = accepted(
        &mut bench,
        Operator::Evaporate {
            vessel: VesselId(0),
            fraction: 0.5,
        },
    );
    assert!(diagnostic(&events, "not-modeled.co-evaporation"));
    assert!(events.iter().any(|e| matches!(e, Event::Evaporated { .. })));
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("water")).0, 2.0);
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("ethanol")).0, 1.0);
}
#[test]
fn complete_evaporation_with_stranded_solute_notice_still_settles() {
    let mut bench = prepared(false);
    deposit(&mut bench, "NaCl", 0.01, Phase::Aqueous);
    let events = accepted(
        &mut bench,
        Operator::Evaporate {
            vessel: VesselId(0),
            fraction: 1.0,
        },
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::NotYetModeled {
            cause: NotModelledCause::NoSolver,
            ..
        }
    )));
    assert!(events.iter().any(|e| matches!(e, Event::Evaporated { .. })));
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("water")).0, 0.0);
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("NaCl")).0, 0.01);
}
#[test]
fn supported_nuclide_spike_still_settles() {
    let mut bench = prepared(false);
    let events = accepted(
        &mut bench,
        Operator::SpikeNuclide {
            vessel: VesselId(0),
            nuclide: "I-131".into(),
            moles: Moles(0.001),
        },
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::NuclideSpiked { .. })));
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::HazardWarning { .. })));
    assert_eq!(
        bench.vessels[0].nuclides.inventory[&nuclide::Nuclide::parse("I-131").unwrap()],
        0.001
    );
}
#[test]
fn supported_reaction_with_model_boundary_still_settles() {
    let mut bench = prepared(false);
    deposit(&mut bench, "ethyl_acetate", 1.0, Phase::Liquid);
    deposit(&mut bench, "NaOH", 1.0, Phase::Aqueous);
    let events = accepted(
        &mut bench,
        Operator::React {
            vessel: VesselId(0),
            reaction: "saponification".into(),
        },
    );
    assert!(events.iter().any(|e| matches!(e, Event::OrgReacted { name, extent, .. } if name == "saponification" && extent.0 > 0.0)));
    assert!(bench.vessels[0].moles_of(&SpeciesId::new("NaOAc")).0 > 0.0);
}

#[derive(Clone, Copy, Debug)]
enum ErrorCase {
    DecantLow,
    DecantHigh,
    DecantNan,
    DecantMissing,
    DecantSelf,
    FilterMissing,
    FilterSelf,
    DrainMissing,
    DrainSelf,
    ExtractZero,
    ExtractNan,
    ExtractStages,
    ExtractUnknown,
    ExtractMissing,
    ExtractSelf,
}
fn error_case(case: ErrorCase) {
    let mut bench = prepared(true);
    let from = VesselId(0);
    let to = VesselId(2);
    let missing = VesselId(9);
    let op = match case {
        ErrorCase::DecantLow | ErrorCase::DecantHigh | ErrorCase::DecantNan => Operator::Decant {
            from,
            to,
            fraction: match case {
                ErrorCase::DecantLow => -0.1,
                ErrorCase::DecantHigh => 1.1,
                _ => f64::NAN,
            },
        },
        ErrorCase::DecantMissing => Operator::Decant {
            from: missing,
            to,
            fraction: 0.5,
        },
        ErrorCase::DecantSelf => Operator::Decant {
            from: missing,
            to: missing,
            fraction: 0.5,
        },
        ErrorCase::FilterMissing => Operator::Filter { from: missing, to },
        ErrorCase::FilterSelf => Operator::Filter {
            from: missing,
            to: missing,
        },
        ErrorCase::DrainMissing => Operator::Drain { from: missing, to },
        ErrorCase::DrainSelf => Operator::Drain {
            from: missing,
            to: missing,
        },
        _ => Operator::Extract {
            from: if matches!(case, ErrorCase::ExtractMissing) {
                missing
            } else {
                from
            },
            to: if matches!(case, ErrorCase::ExtractSelf) {
                from
            } else {
                to
            },
            solvent: SpeciesId::new(if matches!(case, ErrorCase::ExtractUnknown) {
                "__unknown_solvent__"
            } else {
                "hexane"
            }),
            total_solvent: Moles(match case {
                ErrorCase::ExtractZero => 0.0,
                ErrorCase::ExtractNan => f64::NAN,
                _ => 1.0,
            }),
            stages: if matches!(case, ErrorCase::ExtractStages) {
                0
            } else {
                1
            },
        },
    };
    let before = serde_json::to_value(&bench).unwrap();
    let mut solver = CountingMutator::default();
    let result = bench.step_with(op, &mut solver, &PermissiveScreen);
    let expected = match (&case, &result) {
        (
            ErrorCase::DecantLow | ErrorCase::DecantHigh | ErrorCase::DecantNan,
            Err(BenchError::BadFraction),
        ) => true,
        (
            ErrorCase::DecantMissing
            | ErrorCase::FilterMissing
            | ErrorCase::DrainMissing
            | ErrorCase::ExtractMissing,
            Err(BenchError::NoSuchVessel(VesselId(9))),
        ) => true,
        (
            ErrorCase::DecantSelf
            | ErrorCase::FilterSelf
            | ErrorCase::DrainSelf
            | ErrorCase::ExtractSelf,
            Err(BenchError::SelfTransfer),
        ) => true,
        (
            ErrorCase::ExtractZero | ErrorCase::ExtractNan | ErrorCase::ExtractStages,
            Err(BenchError::NonPositiveAmount),
        ) => true,
        (ErrorCase::ExtractUnknown, Err(BenchError::UnknownSpecies(id)))
            if id.0 == "__unknown_solvent__" =>
        {
            true
        }
        _ => false,
    };
    assert!(
        expected,
        "{case:?}: existing exception must remain: {result:?}"
    );
    assert!(solver.0.is_empty());
    assert_eq!(
        serde_json::to_value(&bench).unwrap(),
        before,
        "{case:?}: errors change neither graph, stock nor log"
    );
}
macro_rules! error_contract {
    ($name:ident, $case:ident) => {
        #[test]
        fn $name() {
            error_case(ErrorCase::$case);
        }
    };
}
error_contract!(bad_low_decant_preserves_state, DecantLow);
error_contract!(bad_high_decant_preserves_state, DecantHigh);
error_contract!(bad_nan_decant_preserves_state, DecantNan);
error_contract!(missing_source_decant_preserves_state, DecantMissing);
error_contract!(missing_self_decant_preserves_state, DecantSelf);
error_contract!(missing_source_filter_preserves_state, FilterMissing);
error_contract!(self_filter_preserves_state, FilterSelf);
error_contract!(missing_source_drain_remains_safe, DrainMissing);
error_contract!(self_drain_remains_safe, DrainSelf);
error_contract!(zero_extract_remains_safe, ExtractZero);
error_contract!(nan_extract_remains_safe, ExtractNan);
error_contract!(zero_stage_extract_remains_safe, ExtractStages);
error_contract!(unknown_solvent_extract_remains_safe, ExtractUnknown);
error_contract!(missing_source_extract_remains_safe, ExtractMissing);
error_contract!(self_extract_remains_safe, ExtractSelf);

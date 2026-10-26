//! Transaction/ownership controls use an artificial conservative transform;
//! they do not install or validate any physical reaction endpoint.
use kerotakis_core::aqueous_continuation::{
    whole_step_heat_available, CurrentAqueousAnswer, TrustedAqueousReadback,
};
use kerotakis_core::vessel::{AqueousHeatOwnership, StepStart};
use kerotakis_core::*;

#[derive(Clone, Copy)]
enum Answer {
    Complete,
    Missing,
    Error,
    Invalid,
    Warms,
    LosesOwnership,
    NonConservative,
    WrongQuestion,
    OldReceipt,
    ChangesThermalMode,
    ChangesHeatInput,
    ChangesBoundary,
    InventsContainedGas,
}

struct Aqueous {
    answer: Answer,
    calls: usize,
    heat_charges: usize,
}

fn solution(vessel: &mut Vessel) {
    vessel.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: SolutionScope::Complete,
        solvent_kg: Some(vessel.moles_of(&SpeciesId::new("water")).0 * 0.018015),
        particle_molality: None,
        ph: 7.0,
        ionic_strength: 1e-7,
        pe: None,
        redox: Vec::new(),
        species: Vec::new(),
        provenance: None,
    });
}

impl Aqueous {
    fn new(answer: Answer) -> Self {
        Self {
            answer,
            calls: 0,
            heat_charges: 0,
        }
    }
}

impl Equilibrator for Aqueous {
    fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
        // Explicit synthetic adapter contract; not native/browser proof.
        Some(self)
    }
    fn name(&self) -> &'static str {
        "continuation-test-aqueous"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        match self.answer {
            Answer::Error => {
                // Deliberately spoil the private candidate before failing.
                vessel.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
                vessel.temperature = Kelvin(330.0);
                return Err(SolveError::NotConverged {
                    reason: None,
                    solver: self.name().into(),
                    detail: "controlled readback failure".into(),
                });
            }
            Answer::Missing => return Ok(Vec::new()),
            _ => {}
        }
        solution(vessel);
        if matches!(self.answer, Answer::LosesOwnership) {
            vessel.aqueous_heat = AqueousHeatOwnership::WholeStep;
        }
        if matches!(self.answer, Answer::NonConservative) {
            vessel.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
        }
        if matches!(self.answer, Answer::ChangesThermalMode) {
            vessel.thermal_mode = ThermalMode::Thermostatted(vessel.temperature);
        }
        if matches!(self.answer, Answer::ChangesHeatInput) {
            vessel.heat_input = Some(kerotakis_core::vessel::HeatInput {
                contents: vessel.contents.clone(),
                temperature: vessel.temperature,
                delivered_j: 1.0,
            });
        }
        if matches!(self.answer, Answer::ChangesBoundary) {
            vessel.headspace = if vessel.owns_headspace_gas() {
                Headspace::Open
            } else {
                Headspace::Sealed {
                    volume: Liters(1.0),
                }
            };
        }
        if matches!(self.answer, Answer::Invalid) {
            vessel.solution.as_mut().unwrap().ph = f64::NAN;
        }
        if whole_step_heat_available(vessel) {
            self.heat_charges += 1;
            vessel.temperature = Kelvin(vessel.temperature.0 + 0.25);
        }
        if matches!(self.answer, Answer::Warms) {
            vessel.temperature = Kelvin(vessel.temperature.0 + 1.0);
        }
        Ok(if matches!(self.answer, Answer::InventsContainedGas) {
            vec![Event::GasContained {
                vessel: vessel.id,
                species: SpeciesId::new("O2"),
                moles: Moles(0.001),
            }]
        } else {
            Vec::new()
        })
    }
}

#[test]
fn erased_wrapper_forwards_only_an_explicit_trusted_adapter() {
    use kerotakis_core::aqueous_continuation::{characterize_via, continue_via};
    struct Untrusted(Aqueous);
    impl Equilibrator for Untrusted {
        fn name(&self) -> &'static str {
            "generic-complete-looking-answer"
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            self.0.equilibrate(v)
        }
    }
    let mut vessel = fixture(false);
    let receipt = CurrentAqueousAnswer::attest_computed(&vessel, &vessel).unwrap();
    let before = format!("{vessel:?}");
    let mut untrusted =
        DisplacementEquilibrator::wrapping(Box::new(Untrusted(Aqueous::new(Answer::Complete))));
    assert!(characterize_via(&mut untrusted, &mut vessel).is_err());
    assert!(
        continue_via(&mut vessel, &receipt, &mut untrusted, |_| panic!(
            "untrusted bridge invoked transform"
        ))
        .is_err()
    );
    assert_eq!(format!("{vessel:?}"), before);
    let mut trusted = DisplacementEquilibrator::wrapping(Box::new(Aqueous::new(Answer::Complete)));
    continue_via(&mut vessel, &receipt, &mut trusted, transform).unwrap();
    assert!(!whole_step_heat_available(&vessel));
}

#[test]
fn stack_and_phase_wrappers_forward_the_unique_trusted_adapter() {
    use kerotakis_core::aqueous_continuation::{characterize_via, continue_via};
    for sealed in [false, true] {
        let mut vessel = fixture(sealed);
        let snapshot = vessel.step_start.clone();
        let mut stack = SolverStack::new(vec![
            Box::new(MixingEquilibrator),
            Box::new(PhaseEquilibrator::wrapping(Box::new(
                DisplacementEquilibrator::wrapping(Box::new(Aqueous::new(Answer::Complete))),
            ))),
            Box::new(HonestyEquilibrator),
        ]);
        let (_, receipt) = characterize_via(&mut stack, &mut vessel).unwrap();
        let temperature = vessel.temperature;
        let events = continue_via(&mut vessel, &receipt, &mut stack, transform).unwrap();
        assert_eq!(vessel.step_start, snapshot);
        assert_eq!(vessel.temperature, temperature);
        assert!(!whole_step_heat_available(&vessel));
        assert!(vessel.moles_of(&SpeciesId::new("H2O2")).0.abs() < 1e-12);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    Event::GasContained { .. } | Event::GasEvolved { .. }
                ))
                .count(),
            1
        );
    }
}

#[test]
fn plain_and_ambiguous_stacks_do_not_authorize_cached_answers() {
    use kerotakis_core::aqueous_continuation::{characterize_via, continue_via};
    // This generic engine can return a complete-looking cached solution but
    // explicitly does not opt into the current-input readback contract.
    struct Cached;
    impl Equilibrator for Cached {
        fn name(&self) -> &'static str {
            "cached-test-answer"
        }
        fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            solution(vessel);
            Ok(vec![])
        }
    }
    let stacks = [
        SolverStack::new(vec![]),
        SolverStack::new(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
            Cached,
        )))]),
        SolverStack::new(vec![
            Box::new(Aqueous::new(Answer::Complete)),
            Box::new(PhaseEquilibrator::wrapping(Box::new(Aqueous::new(
                Answer::Complete,
            )))),
        ]),
    ];
    for mut stack in stacks {
        let mut vessel = fixture(false);
        let receipt = CurrentAqueousAnswer::attest_computed(&vessel, &vessel).unwrap();
        let before = format!("{vessel:?}");
        assert!(characterize_via(&mut stack, &mut vessel).is_err());
        assert!(continue_via(&mut vessel, &receipt, &mut stack, |_| {
            panic!("missing or ambiguous adapter must not invoke a transformation")
        })
        .is_err());
        assert_eq!(format!("{vessel:?}"), before);
    }
}

#[test]
fn nested_stack_preserves_unique_authority_and_failed_readback_rollback() {
    use kerotakis_core::aqueous_continuation::{characterize_via, continue_via};
    let mut stack = SolverStack::new(vec![Box::new(SolverStack::new(vec![Box::new(
        Aqueous::new(Answer::Complete),
    )]))]);
    let mut vessel = fixture(false);
    let (_, receipt) = characterize_via(&mut stack, &mut vessel).unwrap();
    // The outer stack must expose the actual trusted adapter, so the existing
    // receipt freshness check still refuses a later mutation before transform.
    vessel.deposit(SpeciesId::new("NaCl"), Moles(0.001), Phase::Solid);
    let before = format!("{vessel:?}");
    assert!(continue_via(&mut vessel, &receipt, &mut stack, |_| {
        panic!("stale receipt must not invoke transform")
    })
    .is_err());
    assert_eq!(format!("{vessel:?}"), before);

    let mut stack = SolverStack::new(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        Aqueous::new(Answer::Error),
    )))]);
    let mut vessel = fixture(false);
    let receipt = CurrentAqueousAnswer::attest_computed(&vessel, &vessel).unwrap();
    let before = format!("{vessel:?}");
    assert!(continue_via(&mut vessel, &receipt, &mut stack, transform).is_err());
    assert_eq!(format!("{vessel:?}"), before);
}

#[test]
fn stack_forwarding_preserves_an_optional_partial_answer_without_a_receipt() {
    use kerotakis_core::aqueous_continuation::{characterize_optional_via, characterize_via};
    struct Partial;
    impl Equilibrator for Partial {
        fn name(&self) -> &'static str {
            "partial-test-answer"
        }
        fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
            Some(self)
        }
        fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            solution(vessel);
            vessel.solution.as_mut().unwrap().scope = SolutionScope::SolventOnly;
            Ok(vec![])
        }
    }
    impl TrustedAqueousReadback for Partial {
        fn characterize_optional_current(
            &mut self,
            vessel: &mut Vessel,
        ) -> Result<(Vec<Event>, Option<CurrentAqueousAnswer>), SolveError> {
            self.equilibrate(vessel).map(|events| (events, None))
        }
        fn characterize_current(
            &mut self,
            _: &mut Vessel,
        ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
            Err(SolveError::NotConverged {
                reason: None,
                solver: "partial-test-answer".into(),
                detail: "complete answer unavailable".into(),
            })
        }
    }
    let mut stack = SolverStack::new(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        Partial,
    )))]);
    let mut vessel = fixture(false);
    let (_, receipt) = characterize_optional_via(&mut stack, &mut vessel).unwrap();
    assert!(receipt.is_none());
    assert_eq!(
        vessel.solution.as_ref().unwrap().scope,
        SolutionScope::SolventOnly
    );
    let before = format!("{vessel:?}");
    assert!(characterize_via(&mut stack, &mut vessel).is_err());
    assert_eq!(format!("{vessel:?}"), before);
}

// Explicit trusted test-double contract: fixture answers are synthetic. This
// is not a native PHREEQC receipt or evidence of chemical correctness.
impl TrustedAqueousReadback for Aqueous {
    fn characterize_current(
        &mut self,
        vessel: &mut Vessel,
    ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
        let mut input = vessel.clone();
        let events = self.equilibrate(vessel)?;
        if matches!(self.answer, Answer::WrongQuestion) {
            input.deposit(SpeciesId::new("water"), Moles(0.01), Phase::Liquid);
        }
        let receipt = if matches!(self.answer, Answer::OldReceipt) {
            let old = fixture(false);
            CurrentAqueousAnswer::attest_computed(&old, &old)?
        } else {
            CurrentAqueousAnswer::attest_computed(&input, vessel)?
        };
        Ok((events, receipt))
    }
}

fn continue_fixture(
    vessel: &mut Vessel,
    aqueous: &mut Aqueous,
    transform: impl FnOnce(&mut Vessel) -> Result<Vec<Event>, SolveError>,
) -> Result<Vec<Event>, SolveError> {
    // The manually populated fixture is the test double's initial answer.
    let initial = CurrentAqueousAnswer::attest_computed(vessel, vessel)?;
    kerotakis_core::aqueous_continuation::continue_without_reaction_heat(
        vessel, &initial, aqueous, transform,
    )
}

fn fixture(sealed: bool) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "continuation control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    if sealed {
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        // This no-boiling fixture needs an occupied headspace. Otherwise
        // independent phase equilibrium legitimately cools the near-vacuum.
        assert!(vessel.retain_gas(SpeciesId::new("N2"), Moles(0.04)));
    }
    solution(&mut vessel);
    let mut start = StepStart::capture(&vessel);
    start.note_gas_out(&SpeciesId::new("O2"), 1e-7);
    vessel.step_start = Some(start);
    vessel
}

fn transform(vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
    // Atom/charge-conserving fixture. No production reaction data is added.
    vessel.withdraw_phase(&SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    vessel.deposit(SpeciesId::new("water"), Moles(0.002), Phase::Liquid);
    let oxygen = SpeciesId::new("O2");
    let moles = Moles(0.001);
    let gas = if vessel.retain_gas(oxygen.clone(), moles) {
        Event::GasContained {
            vessel: vessel.id,
            species: oxygen,
            moles,
        }
    } else {
        Event::GasEvolved {
            vessel: vessel.id,
            species: oxygen,
            moles,
        }
    };
    Ok(vec![gas])
}

#[test]
fn accepted_continuation_preserves_original_snapshot_and_survives_phase_tail_reentry() {
    for sealed in [false, true] {
        let mut vessel = fixture(sealed);
        let snapshot = vessel.step_start.clone();
        let temperature = vessel.temperature;
        let mut aqueous = Aqueous::new(Answer::Complete);
        let events = continue_fixture(&mut vessel, &mut aqueous, transform).unwrap();
        assert_eq!(vessel.step_start, snapshot);
        assert!(!whole_step_heat_available(&vessel));
        assert_eq!(vessel.moles_of(&SpeciesId::new("H2O2")).0, 0.0);
        assert!((vessel.moles_of(&SpeciesId::new("water")).0 - 5.552).abs() < 1e-12);
        assert_eq!(
            vessel.moles_of(&SpeciesId::new("O2")).0,
            if sealed { 0.001 } else { 0.0 }
        );
        assert!(
            events.iter().any(|event| matches!(event,
            Event::GasContained { .. } if sealed))
                || events.iter().any(|event| matches!(event,
            Event::GasEvolved { .. } if !sealed))
        );
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::NotYetModeled { .. })));
        // Re-enter the actual shared phase-coupling entry point within this
        // same operator. No pass can reprice from the untouched old snapshot.
        for _ in 0..2 {
            equilibrate_phase_coupled(&mut aqueous, &mut vessel).unwrap();
            assert!(!whole_step_heat_available(&vessel));
        }
        assert_eq!(aqueous.calls, 3);
        assert_eq!(aqueous.heat_charges, 0);
        assert_eq!(vessel.temperature, temperature);
        // Internal heat/dose processing may recapture or clear StepStart.
        // Ownership outlives that narrower snapshot lifecycle.
        vessel.step_start = None;
        vessel.step_start = Some(StepStart::capture(&vessel));
        assert!(!whole_step_heat_available(&vessel));
    }
}

#[test]
fn failed_missing_invalid_or_thermally_unowned_readback_rolls_back_everything() {
    for answer in [
        Answer::Error,
        Answer::Missing,
        Answer::Invalid,
        Answer::Warms,
        Answer::LosesOwnership,
        Answer::NonConservative,
        Answer::WrongQuestion,
        Answer::OldReceipt,
        Answer::ChangesThermalMode,
        Answer::ChangesHeatInput,
        Answer::ChangesBoundary,
        Answer::InventsContainedGas,
    ] {
        for sealed in [false, true] {
            let mut vessel = fixture(sealed);
            let before = format!("{vessel:?}");
            let mut aqueous = Aqueous::new(answer);
            assert!(continue_fixture(&mut vessel, &mut aqueous, transform).is_err());
            assert_eq!(format!("{vessel:?}"), before);
            assert!(whole_step_heat_available(&vessel));
        }
    }
}

#[test]
fn missing_initial_characterization_or_operator_snapshot_never_calls_transform() {
    for missing_solution in [false, true] {
        let mut vessel = fixture(false);
        if missing_solution {
            vessel.solution = None;
        } else {
            vessel.step_start = None;
        }
        let before = serde_json::to_value(&vessel).unwrap();
        let mut aqueous = Aqueous::new(Answer::Complete);
        let result = continue_fixture(&mut vessel, &mut aqueous, |_| {
            panic!("inadmissible continuation called transform")
        });
        assert!(result.is_err());
        assert_eq!(aqueous.calls, 0);
        assert_eq!(serde_json::to_value(&vessel).unwrap(), before);
    }
}

#[test]
fn nonconservative_transform_and_snapshot_replacement_are_rejected_before_readback() {
    for replace_snapshot in [false, true] {
        let mut vessel = fixture(false);
        let before = serde_json::to_value(&vessel).unwrap();
        let mut aqueous = Aqueous::new(Answer::Complete);
        assert!(continue_fixture(&mut vessel, &mut aqueous, |candidate| {
            if replace_snapshot {
                candidate.step_start = None;
            } else {
                candidate.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
            }
            Ok(Vec::new())
        })
        .is_err());
        assert_eq!(aqueous.calls, 0);
        assert_eq!(serde_json::to_value(&vessel).unwrap(), before);
    }
}

#[test]
fn bench_operator_lifecycle_restores_heat_ownership_on_success_and_error() {
    struct MarkThenFail(bool);
    impl Equilibrator for MarkThenFail {
        fn name(&self) -> &'static str {
            "continuation-lifecycle-control"
        }
        fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            assert!(
                whole_step_heat_available(vessel),
                "new operator inherited old ownership"
            );
            vessel.aqueous_heat = AqueousHeatOwnership::UnmodeledContinuation;
            if self.0 {
                Err(SolveError::NotConverged {
                    reason: None,
                    solver: self.name().into(),
                    detail: "controlled failure".into(),
                })
            } else {
                Ok(Vec::new())
            }
        }
    }
    for fail in [false, true] {
        let mut bench = Bench::new();
        for _ in 0..2 {
            bench
                .step_with(
                    Operator::Wait { seconds: 0.0 },
                    &mut MarkThenFail(fail),
                    &PermissiveScreen,
                )
                .unwrap();
            assert!(bench.vessels.iter().all(whole_step_heat_available));
        }
        let malformed = Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(f64::NAN),
            at: None,
        };
        let _ = bench.step_with(malformed, &mut MarkThenFail(fail), &PermissiveScreen);
        assert!(bench.vessels.iter().all(whole_step_heat_available));
        for vessel in &mut bench.vessels {
            vessel.aqueous_heat = AqueousHeatOwnership::UnmodeledContinuation;
        }
        assert!(bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(999),
                    species: SpeciesId::new("water"),
                    moles: Moles(1.0),
                    at: None
                },
                &mut MarkThenFail(fail),
                &PermissiveScreen,
            )
            .is_err());
        assert!(bench.vessels.iter().all(whole_step_heat_available));
    }
}

#[test]
fn delta_snapshot_carries_policy_and_rejects_a_stale_ownership_base() {
    let original = fixture(false);
    let mut accepted = original.clone();
    let events = continue_fixture(
        &mut accepted,
        &mut Aqueous::new(Answer::Complete),
        transform,
    )
    .unwrap();
    let delta = kerotakis_core::orchestrator::diff_vessels(
        &original,
        &accepted,
        "continuation-delta-control",
    );
    let mut destination = original.clone();
    delta
        .commit_conserved_with_events(&mut destination, 1e-7, &events)
        .unwrap();
    assert!(!whole_step_heat_available(&destination));
    assert_eq!(destination.step_start, original.step_start);
    let mut stale = original;
    stale.aqueous_heat = AqueousHeatOwnership::UnmodeledContinuation;
    assert!(delta
        .commit_conserved_with_events(&mut stale, 1e-7, &events)
        .is_err());
}

#[test]
fn deposit_retaining_a_solution_does_not_retain_its_answer_receipt() {
    let mut vessel = fixture(false);
    let initial = CurrentAqueousAnswer::attest_computed(&vessel, &vessel).unwrap();
    vessel.deposit(SpeciesId::new("water"), Moles(0.01), Phase::Liquid);
    assert!(
        vessel.solution.is_some(),
        "the original freshness trap is exercised"
    );
    let before = format!("{vessel:?}");
    let mut aqueous = Aqueous::new(Answer::Complete);
    assert!(
        kerotakis_core::aqueous_continuation::continue_without_reaction_heat(
            &mut vessel,
            &initial,
            &mut aqueous,
            |_| panic!("stale initial answer admitted"),
        )
        .is_err()
    );
    assert_eq!(aqueous.calls, 0);
    assert_eq!(format!("{vessel:?}"), before);
}

#[test]
fn material_transform_cannot_change_thermal_inputs_or_boundary_configuration() {
    for mutation in 0..4 {
        let mut vessel = fixture(false);
        let before = format!("{vessel:?}");
        let mut aqueous = Aqueous::new(Answer::Complete);
        assert!(continue_fixture(&mut vessel, &mut aqueous, |candidate| {
            match mutation {
                0 => candidate.thermal_mode = ThermalMode::Thermostatted(candidate.temperature),
                1 => {
                    candidate.heat_input = Some(kerotakis_core::vessel::HeatInput {
                        contents: candidate.contents.clone(),
                        temperature: candidate.temperature,
                        delivered_j: 1.0,
                    })
                }
                2 => {
                    candidate.headspace = Headspace::Sealed {
                        volume: Liters(1.0),
                    }
                }
                _ => candidate.pending_co2_transfer_mol = 0.001,
            }
            Ok(Vec::new())
        })
        .is_err());
        assert_eq!(aqueous.calls, 0);
        assert_eq!(format!("{vessel:?}"), before);
    }
}

#[test]
fn aggregate_contained_gas_must_match_new_owned_gas_not_old_inventory() {
    for variant in 0..6 {
        let mut vessel = fixture(true);
        if variant == 1 {
            assert!(vessel.retain_gas(SpeciesId::new("O2"), Moles(0.001)));
        }
        let before = format!("{vessel:?}");
        let mut aqueous = Aqueous::new(Answer::Complete);
        assert!(continue_fixture(&mut vessel, &mut aqueous, |candidate| {
            if variant == 0 {
                let mut events = transform(candidate)?;
                events.push(events[0].clone());
                Ok(events)
            } else if variant == 4 {
                let _ = transform(candidate)?;
                Ok(Vec::new())
            } else if variant == 5 {
                let mut events = transform(candidate)?;
                if let Event::GasContained { moles, .. } = &mut events[0] {
                    moles.0 *= 0.5;
                }
                Ok(events)
            } else if variant == 2 {
                let _ = transform(candidate)?;
                Ok(vec![Event::GasContained {
                    vessel: candidate.id,
                    species: SpeciesId::new("N2"),
                    moles: Moles(0.001),
                }])
            } else {
                Ok(vec![Event::GasContained {
                    vessel: candidate.id,
                    species: SpeciesId::new("O2"),
                    moles: Moles(if variant == 3 { 1e-20 } else { 0.001 }),
                }])
            }
        })
        .is_err());
        assert_eq!(aqueous.calls, 0);
        assert_eq!(format!("{vessel:?}"), before);
    }
}

#[test]
fn dissolution_of_owned_gas_requires_no_negative_contained_event() {
    let mut vessel = fixture(true);
    vessel.deposit(SpeciesId::new("O2"), Moles(0.001), Phase::Gas);
    let mut aqueous = Aqueous::new(Answer::Complete);
    continue_fixture(&mut vessel, &mut aqueous, |candidate| {
        candidate.withdraw(&SpeciesId::new("O2"), Moles(0.001));
        candidate.deposit(SpeciesId::new("O2"), Moles(0.001), Phase::Aqueous);
        Ok(Vec::new())
    })
    .unwrap();
}

#[test]
fn fixed_sealed_boundary_cannot_export_a_conservative_gas_parcel() {
    let mut vessel = fixture(true);
    let before = format!("{vessel:?}");
    let mut aqueous = Aqueous::new(Answer::Complete);
    assert!(continue_fixture(&mut vessel, &mut aqueous, |candidate| {
        let _ = transform(candidate)?;
        candidate.withdraw_phase(&SpeciesId::new("O2"), Moles(0.001), Phase::Gas);
        Ok(vec![Event::GasEvolved {
            vessel: candidate.id,
            species: SpeciesId::new("O2"),
            moles: Moles(0.001),
        }])
    })
    .is_err());
    assert_eq!(aqueous.calls, 0);
    assert_eq!(format!("{vessel:?}"), before);
}

struct ContinuationRoute;

impl Equilibrator for ContinuationRoute {
    fn name(&self) -> &'static str {
        "synthetic-conservative-continuation"
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        continue_fixture(vessel, &mut Aqueous::new(Answer::Complete), transform)
    }
}

#[test]
fn ordinary_stack_books_only_released_gas_once_after_transaction_commit() {
    for sealed in [false, true] {
        let mut vessel = fixture(sealed);
        let original = vessel.step_start.as_ref().unwrap().clone();
        let mut stack = SolverStack::new(vec![Box::new(ContinuationRoute)]);
        stack.equilibrate(&mut vessel).unwrap();
        let start = vessel.step_start.as_ref().unwrap();
        assert_eq!(start.contents, original.contents);
        assert_eq!(start.solute_charge, original.solute_charge);
        let outward = start
            .gas_out
            .iter()
            .find(|(species, _)| species.0 == "O2")
            .unwrap()
            .1
             .0;
        let expected = 1e-7 + if sealed { 0.0 } else { 0.001 };
        assert!((outward - expected).abs() < 1e-15);
    }
}

#[test]
fn unknown_aqueous_heat_does_not_disable_conservative_vacuum_boiling() {
    use kerotakis_core::ledger::ConservedLedger;
    let mut vessel = Vessel::new(VesselId(0), "independent phase cooling");
    vessel.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    assert!(vessel.retain_gas(SpeciesId::new("O2"), Moles(0.001)));
    vessel.aqueous_heat = AqueousHeatOwnership::UnmodeledContinuation;
    let before = ConservedLedger::from_vessel(&vessel);
    let temperature = vessel.temperature;
    let events = StateEquilibrator.equilibrate(&mut vessel).unwrap();
    assert!(vessel.temperature.0 < temperature.0);
    assert!(!whole_step_heat_available(&vessel));
    assert!(events.iter().any(|event| matches!(event,
        Event::StateChanged { species, from: Phase::Liquid, to: Phase::Gas, .. }
            if species.0 == "water"
    )));
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::GasEvolved { .. })));
    let after = ConservedLedger::from_vessel(&vessel);
    assert_eq!(before.charge, after.charge);
    let elements: std::collections::BTreeSet<_> = before
        .elements
        .keys()
        .chain(after.elements.keys())
        .collect();
    for element in elements {
        assert!(
            (before.elements.get(element).copied().unwrap_or(0.0)
                - after.elements.get(element).copied().unwrap_or(0.0))
            .abs()
                < 1e-10
        );
    }
}

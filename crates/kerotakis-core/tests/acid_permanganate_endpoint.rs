//! Frozen engineering expectations; synthetic aqueous answers are not chemistry validation.
use kerotakis_core::aqueous_continuation::{CurrentAqueousAnswer, TrustedAqueousReadback};
use kerotakis_core::curated::{fires, CuratedReaction, PostAqueousEndpoint, REACTIONS};
use kerotakis_core::ledger::ConservedLedger;
use kerotakis_core::vessel::{AqueousHeatOwnership, StepStart};
use kerotakis_core::*;

fn endpoint() -> &'static CuratedReaction {
    REACTIONS
        .iter()
        .find(|r| r.post_aqueous == Some(PostAqueousEndpoint::AcidicPermanganatePeroxide))
        .unwrap()
}
fn solution(v: &mut Vessel, ph: f64) {
    v.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: SolutionScope::Complete,
        solvent_kg: Some(v.moles_of(&SpeciesId::new("water")).0 * 0.018015),
        particle_molality: None,
        ph,
        ionic_strength: 0.1,
        pe: None,
        redox: vec![],
        species: vec![],
        provenance: None,
    });
}
fn fixture(mn: f64, peroxide: f64, acid: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "acid endpoint contract");
    for (key, n, phase) in [
        ("water", 5.55, Phase::Liquid),
        ("K+", mn, Phase::Aqueous),
        ("MnO4-", mn, Phase::Aqueous),
        ("SO4-2", acid / 2.0, Phase::Aqueous),
        ("H+", acid, Phase::Aqueous),
        ("H2O2", peroxide, Phase::Liquid),
    ] {
        v.deposit(SpeciesId::new(key), Moles(n), phase);
    }
    v.solute_charge = displacement::solute_charge(&v);
    solution(&mut v, 1.0);
    v.step_start = Some(StepStart::capture(&v));
    v
}
fn amount(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-11, "{a} != {b}");
}
#[derive(Default)]
struct Aqueous {
    calls: usize,
    fail_second: bool,
    change_initial: bool,
}
impl Equilibrator for Aqueous {
    fn name(&self) -> &'static str {
        "synthetic-acid-endpoint-answer"
    }
    fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
        Some(self)
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.characterize_current(v).map(|(events, _)| events)
    }
}
impl TrustedAqueousReadback for Aqueous {
    fn characterize_current(
        &mut self,
        v: &mut Vessel,
    ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
        let input = v.clone();
        self.calls += 1;
        if self.fail_second && self.calls == 2 {
            v.deposit(SpeciesId::new("Na+"), Moles(1.0), Phase::Aqueous);
            return Err(SolveError::NotConverged {
                reason: None,
                solver: self.name().into(),
                detail: "controlled current-question miss".into(),
            });
        }
        if self.change_initial && self.calls == 1 {
            v.temperature = Kelvin(v.temperature.0 + 1.0);
        }
        let ph = if amount(v, "H+") <= 1e-12 { 7.0 } else { 1.0 };
        solution(v, ph);
        let receipt = CurrentAqueousAnswer::attest_computed(&input, v)?;
        Ok((vec![], receipt))
    }
}
fn gas(events: &[Event], contained: bool) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasContained { species, moles, .. } if contained && species.0 == "O2" => {
                Some(moles.0)
            }
            Event::GasEvolved { species, moles, .. } if !contained && species.0 == "O2" => {
                Some(moles.0)
            }
            _ => None,
        })
        .sum()
}
#[test]
fn independent_limiting_reagents_conserve_atoms_charge_and_excess() {
    for (mn, peroxide, acid, x) in [
        (0.002, 0.005, 0.006, 0.001),
        (0.001, 0.005, 0.006, 0.0005),
        (0.002, 0.001, 0.006, 0.0002),
    ] {
        for sealed in [false, true] {
            let mut v = fixture(mn, peroxide, acid);
            if sealed {
                v.headspace = Headspace::Sealed {
                    volume: Liters(1.0),
                };
                v.deposit(SpeciesId::new("N2"), Moles(0.04), Phase::Gas);
            }
            let before = v.clone();
            let initial = ConservedLedger::from_vessel(&v);
            let mut solver = Aqueous::default();
            let events = displacement::over(&mut solver, &mut v).unwrap();
            assert_eq!(
                solver.calls, 2,
                "one first answer and one committed readback"
            );
            close(amount(&v, "MnO4-"), mn - 2.0 * x);
            close(amount(&v, "H2O2"), peroxide - 5.0 * x);
            close(amount(&v, "H+"), acid - 6.0 * x);
            close(amount(&v, "Mn+2"), 2.0 * x);
            close(amount(&v, "water"), 5.55 + 8.0 * x);
            close(amount(&v, "K+"), mn);
            close(amount(&v, "SO4-2"), acid / 2.0);
            close(gas(&events, sealed), 5.0 * x);
            close(gas(&events, !sealed), 0.0);
            close(amount(&v, "O2"), if sealed { 5.0 * x } else { 0.0 });
            let mut after = ConservedLedger::from_vessel(&v);
            if !sealed {
                *after.elements.entry("O".into()).or_default() += 10.0 * x;
            }
            for (element, n) in initial.elements {
                close(n, *after.elements.get(&element).unwrap_or(&0.0));
            }
            close(initial.charge, after.charge);
            let emitted_mass = if sealed {
                0.0
            } else {
                5.0 * x * species::lookup(&SpeciesId::new("O2")).unwrap().molar_mass
            };
            // Registry molar masses are rounded independently; atom/charge
            // closure above is strict, this mass check allows their rounding.
            assert!((initial.mass - after.mass - emitted_mass).abs() <= 1e-6 * initial.mass);
            close(v.temperature.0, before.temperature.0);
            assert_eq!(v.step_start, before.step_start);
            assert_eq!(v.aqueous_heat, AqueousHeatOwnership::UnmodeledContinuation);
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, Event::ReactionOccurred { .. }))
                    .count(),
                1
            );
            assert!(v.solution.is_some());
        }
    }
}
#[test]
fn dry_wrong_phase_proxy_only_and_competing_media_do_not_fire() {
    for mutation in [
        "dry",
        "solid-mn",
        "gas-peroxide",
        "solid-acid",
        "no-proton",
        "chloride",
        "no-sulfate",
        "weak-acid",
        "neutral",
        "basic",
        "uncharacterized",
        "nan-ph",
        "nan-pool",
        "unbalanced-counterion",
    ] {
        let mut v = fixture(0.002, 0.005, 0.006);
        match mutation {
            "dry" => {
                v.contents.retain(|p| p.species.0 != "water");
            }
            "solid-mn" => {
                v.contents
                    .iter_mut()
                    .find(|p| p.species.0 == "MnO4-")
                    .unwrap()
                    .phase = Phase::Solid
            }
            "gas-peroxide" => {
                v.contents
                    .iter_mut()
                    .find(|p| p.species.0 == "H2O2")
                    .unwrap()
                    .phase = Phase::Gas
            }
            "solid-acid" => {
                v.contents
                    .iter_mut()
                    .find(|p| p.species.0 == "H+")
                    .unwrap()
                    .phase = Phase::Solid
            }
            "no-proton" => {
                v.contents.retain(|p| p.species.0 != "H+");
                v.free_proton = 0.006;
            }
            "chloride" => v.deposit(SpeciesId::new("Cl-"), Moles(0.001), Phase::Aqueous),
            "no-sulfate" => v.contents.retain(|p| p.species.0 != "SO4-2"),
            "weak-acid" => v.deposit(SpeciesId::new("CH3COOH"), Moles(0.001), Phase::Liquid),
            "neutral" => v.solution.as_mut().unwrap().ph = 7.0,
            "basic" => v.solution.as_mut().unwrap().ph = 12.0,
            "uncharacterized" => v.solution = None,
            "nan-ph" => v.solution.as_mut().unwrap().ph = f64::NAN,
            "nan-pool" => {
                v.contents
                    .iter_mut()
                    .find(|p| p.species.0 == "H2O2")
                    .unwrap()
                    .moles = Moles(f64::NAN)
            }
            "unbalanced-counterion" => v.contents.retain(|p| p.species.0 != "K+"),
            _ => unreachable!(),
        }
        assert!(!fires(&v, endpoint()), "{mutation}");
    }
}
#[test]
fn trace_exhaustion_and_declared_ph_ceiling_are_exact_domain_controls() {
    for n in [0.0, 1e-12, 5e-12] {
        assert!(!fires(&fixture(0.002, n, 0.006), endpoint()));
    }
    assert!(fires(&fixture(0.002, 1e-11, 0.006), endpoint()));
    let mut v = fixture(0.002, 0.005, 0.006);
    v.solution.as_mut().unwrap().ph = 3.0;
    assert!(fires(&v, endpoint()));
    v.solution.as_mut().unwrap().ph = 3.0001;
    assert!(!fires(&v, endpoint()));
}
#[test]
fn legacy_pre_aqueous_stage_never_owns_the_new_row() {
    let mut v = fixture(0.002, 0.005, 0.006);
    let before = format!("{v:?}");
    let events = CuratedEquilibrator.equilibrate(&mut v).unwrap();
    assert_eq!(format!("{v:?}"), before);
    assert!(events.is_empty());
}
#[test]
fn failed_current_readback_rolls_back_open_and_retained_products() {
    for sealed in [false, true] {
        let mut v = fixture(0.002, 0.005, 0.006);
        if sealed {
            v.headspace = Headspace::Sealed {
                volume: Liters(1.0),
            };
        }
        let before = format!("{v:?}");
        let mut solver = Aqueous {
            calls: 0,
            fail_second: true,
            change_initial: true,
        };
        assert!(displacement::over(&mut solver, &mut v).is_err());
        assert_eq!(format!("{v:?}"), before);
    }
}
#[test]
fn initial_oxygen_is_retained_and_exhausted_endpoint_cannot_repeat() {
    let mut v = fixture(0.002, 0.005, 0.006);
    v.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    v.deposit(SpeciesId::new("O2"), Moles(0.002), Phase::Gas);
    let events = displacement::over(&mut Aqueous::default(), &mut v).unwrap();
    close(gas(&events, true), 0.005);
    close(amount(&v, "O2"), 0.007);
    assert!(!fires(&v, endpoint()));
    let events = displacement::over(&mut Aqueous::default(), &mut v).unwrap();
    assert_eq!(gas(&events, true), 0.0);
}

#[test]
fn incomplete_optional_answer_keeps_the_original_scope_and_runs_once() {
    struct Partial(usize);
    impl Equilibrator for Partial {
        fn name(&self) -> &'static str {
            "synthetic-solvent-only"
        }
        fn trusted_aqueous_readback(&mut self) -> Option<&mut dyn TrustedAqueousReadback> {
            Some(self)
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            self.0 += 1;
            solution(v, 7.0);
            v.solution.as_mut().unwrap().scope = SolutionScope::SolventOnly;
            Ok(vec![])
        }
    }
    impl TrustedAqueousReadback for Partial {
        fn characterize_current(
            &mut self,
            _: &mut Vessel,
        ) -> Result<(Vec<Event>, CurrentAqueousAnswer), SolveError> {
            panic!("partial answer must not attempt strict attestation")
        }
        fn characterize_optional_current(
            &mut self,
            v: &mut Vessel,
        ) -> Result<(Vec<Event>, Option<CurrentAqueousAnswer>), SolveError> {
            self.equilibrate(v).map(|events| (events, None))
        }
    }
    let initial = fixture(0.002, 0.005, 0.006);
    let mut expected = initial.clone();
    Partial(0).equilibrate(&mut expected).unwrap();
    let mut actual = initial;
    let mut solver = Partial(0);
    let events = displacement::over(&mut solver, &mut actual).unwrap();
    assert_eq!(solver.0, 1);
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::ReactionOccurred { .. })));
}

#[test]
fn acid_limited_residual_mixture_leaving_acidic_domain_rolls_back_atomically() {
    for sealed in [false, true] {
        let mut v = fixture(0.002, 0.005, 0.003);
        if sealed {
            v.headspace = Headspace::Sealed {
                volume: Liters(1.0),
            };
        }
        let before = format!("{v:?}");
        let mut solver = Aqueous::default();
        let events = displacement::over(&mut solver, &mut v).unwrap();
        assert_eq!(
            solver.calls, 2,
            "a genuine final pH is checked before commit"
        );
        assert_eq!(format!("{v:?}"), before);
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::ReactionOccurred { .. } | Event::GasContained { .. } | Event::GasEvolved { .. }
        )));
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })));
    }
}

//! Pure-water latent heat remains computable when the aqueous engine is absent.
use kerotakis_core::*;

struct MissingChemistry;
impl Equilibrator for MissingChemistry {
    fn name(&self) -> &'static str {
        "missing-test-chemistry"
    }
    fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        Err(SolveError::NotConverged {
            solver: self.name().into(),
            detail: "no cached result or engine".into(),
        })
    }
}

fn stale_solution() -> SolutionInfo {
    SolutionInfo {
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: vec![],
        ph: 3.0,
        ionic_strength: 0.1,
        species: vec![SpeciesDetail {
            name: "obsolete solute".into(),
            molality: 0.1,
            activity: 0.1,
        }],
        provenance: None,
    }
}

fn water(moles: f64, phase: Phase, temperature: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "phase fallback");
    v.deposit(SpeciesId::new("water"), Moles(moles), phase);
    v.temperature = Kelvin(temperature);
    v.solution = Some(stale_solution());
    v.resolved.solution = Some(stale_solution());
    v.resolved.valid = true;
    v.free_proton = 0.001;
    v.free_hydroxide = 0.001;
    v
}

fn assert_phase_computed_without_asking_chemistry(v: &mut Vessel) -> Vec<Event> {
    let events = equilibrate_phase_coupled(&mut MissingChemistry, v).unwrap();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::SolverFailed { .. }))
            .count(),
        0,
        "independent solvent phase physics must run before aqueous chemistry"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::StateChanged { .. })),
        "{events:?}"
    );
    assert!(v.solution.is_none() && v.resolved.solution.is_none() && !v.resolved.valid);
    assert_eq!(v.free_proton, 0.0);
    assert_eq!(v.free_hydroxide, 0.0);
    events
}

#[test]
fn pure_water_freezes_melts_and_boils_with_conserved_latent_energy_across_scales() {
    for amount in [0.05, 5.0, 50.0] {
        for (initial, final_phase, boundary, initial_t, latent) in [
            (
                Phase::Liquid,
                Phase::Solid,
                states::WATER_FREEZING_K,
                263.15,
                states::WATER_H_FUS,
            ),
            (
                Phase::Solid,
                Phase::Liquid,
                states::WATER_FREEZING_K,
                283.15,
                states::WATER_H_FUS,
            ),
            (
                Phase::Liquid,
                Phase::Gas,
                states::WATER_BOILING_K,
                383.15,
                states::WATER_H_VAP,
            ),
        ] {
            let mut v = water(amount, initial, initial_t);
            // The sensible heat the vessel gives up walking to the phase
            // boundary, taken the way the engine takes it since #509: the
            // INTEGRAL of Cp(T) between the two temperatures, not
            // `Cp · ΔT` read at one of them. Water's Cp is not flat over
            // 20 K, so the rectangle and the area differ in the fourth
            // decimal — which is exactly the size of the assertion below.
            let available = v.energy_between(boundary, initial_t).abs();
            let events = assert_phase_computed_without_asking_chemistry(&mut v);
            assert!((v.temperature.0 - boundary).abs() < 1e-9);
            let transferred: f64 = events
                .iter()
                .filter_map(|event| match event {
                    Event::StateChanged {
                        to,
                        moles: Some(moles),
                        ..
                    } if *to == final_phase => Some(moles.0),
                    _ => None,
                })
                .sum();
            assert!(transferred > 0.0 && transferred < amount);
            assert!((transferred * latent - available).abs() < 1e-8 * available);
            let vented: f64 = events
                .iter()
                .filter_map(|event| match event {
                    Event::GasEvolved { species, moles, .. } if species.0 == "water" => {
                        Some(moles.0)
                    }
                    _ => None,
                })
                .sum();
            let remaining: f64 = v.contents.iter().map(|p| p.moles.0).sum();
            assert!((remaining + vented - amount).abs() < 1e-10 * amount);
        }
    }
}

#[test]
fn unsolved_ionic_and_unknown_mixtures_do_not_get_a_pure_water_phase_answer() {
    for solute in ["NaCl", "unregistered-test-solute"] {
        let mut v = water(5.0, Phase::Liquid, 263.15);
        v.deposit(SpeciesId::new(solute), Moles(0.001), Phase::Aqueous);
        let before = serde_json::to_value(&v).unwrap();
        let events = equilibrate_phase_coupled(&mut MissingChemistry, &mut v).unwrap();
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], Event::SolverFailed { .. }));
    }
}

#[test]
fn no_transition_keeps_aqueous_failure_explicit_and_clears_stale_measurements() {
    let mut v = water(5.0, Phase::Liquid, 298.15);
    let events = equilibrate_phase_coupled(&mut MissingChemistry, &mut v).unwrap();
    assert_eq!(v.temperature.0, 298.15);
    assert_eq!(v.contents[0].moles.0, 5.0);
    assert!(v.solution.is_none() && v.resolved.solution.is_none());
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::SolverFailed { .. }));
}

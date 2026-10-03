//! Transactional recovery seeds liquid from paid ice superheat; it does not
//! assign a guessed liquidus to the initial concentrated, unsolved brine.
use kerotakis_core::*;

fn amount(v: &Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn frozen(stock: f64, t: f64, gas_ratio: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "residual brine");
    v.headspace = Headspace::Sealed {
        volume: Liters(stock * 0.02),
    };
    v.deposit(SpeciesId::new("water"), Moles(stock), Phase::Solid);
    v.deposit(SpeciesId::new("water"), Moles(stock * 1e-12), Phase::Liquid);
    v.deposit(SpeciesId::new("Na+"), Moles(stock * 1e-10), Phase::Aqueous);
    v.deposit(SpeciesId::new("Cl-"), Moles(stock * 1e-10), Phase::Aqueous);
    v.deposit(SpeciesId::new("N2"), Moles(stock * gas_ratio), Phase::Gas);
    v.temperature = Kelvin(t);
    v.refresh_pressure();
    v
}

struct NeedsLiquid {
    minimum: f64,
    permanent_failure: bool,
    corrupt: bool,
    calls: usize,
}
impl Equilibrator for NeedsLiquid {
    fn name(&self) -> &'static str {
        "liquid-seed-test"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        if self.permanent_failure || amount(v, "water", Phase::Liquid) < self.minimum {
            if self.corrupt {
                v.deposit(SpeciesId::new("Cu"), Moles(1.0), Phase::Solid);
                v.temperature = Kelvin(800.0);
            }
            return Err(SolveError::NotConverged {
                solver: self.name().into(),
                detail: "residual liquid unresolved".into(),
            });
        }
        let water_kg = amount(v, "water", Phase::Liquid) * constants::WATER_MOLAR_MASS_KG_PER_MOL;
        v.solution = Some(SolutionInfo {
            scope: Default::default(),
            solvent_kg: Some(water_kg),
            pe: None,
            redox: vec![],
            ph: 7.0,
            ionic_strength: 0.0,
            species: ["Na+", "Cl-"]
                .into_iter()
                .map(|name| SpeciesDetail {
                    name: name.into(),
                    molality: amount(v, name, Phase::Aqueous) / water_kg,
                    activity: amount(v, name, Phase::Aqueous) / water_kg,
                })
                .collect(),
            provenance: None,
            solvent_activity: None,
        });
        Ok(vec![])
    }
}

// Assemble the anchored phase energy from each public species record, rather
// than calling the recovery helper or Vessel::energy_between. The liquid/ice
// offset is the reviewed fusion value at the common reference temperature.
fn energy(v: &Vessel) -> f64 {
    let tm = states::WATER_FREEZING_K;
    v.contents
        .iter()
        .map(|p| {
            let data = species::lookup(&p.species).unwrap();
            let mut sensible = states::enthalpy_between(data, p.phase, tm, v.temperature.0);
            if p.phase == Phase::Gas && v.is_sealed() {
                sensible -= constants::GAS_CONSTANT * (v.temperature.0 - tm);
            }
            let latent = if p.species.0 == "water" && p.phase == Phase::Liquid {
                states::WATER_H_FUS
            } else {
                0.0
            };
            p.moles.0 * (sensible + latent)
        })
        .sum()
}

fn assert_energy(before: &Vessel, after: &Vessel) {
    let a = energy(before);
    let b = energy(after);
    assert!(
        (a - b).abs() <= 1e-10 * a.abs().max(1.0),
        "before={a}, after={b}"
    );
}

#[test]
fn paid_partial_and_complete_thaw_recovers_chemistry_across_stock_and_gas_scales() {
    for stock in [0.005, 0.5, 50.0] {
        for gas_ratio in [0.0, 0.01, 0.1] {
            for (t, full) in [(278.15, false), (500.0, true)] {
                let mut v = frozen(stock, t, gas_ratio);
                let before = v.clone();
                let mut chemistry = NeedsLiquid {
                    minimum: stock * 1e-6,
                    permanent_failure: false,
                    corrupt: true,
                    calls: 0,
                };
                let events = equilibrate_phase_coupled(&mut chemistry, &mut v).unwrap();
                assert!(chemistry.calls >= 2);
                assert!(
                    !events
                        .iter()
                        .any(|e| matches!(e, Event::SolverFailed { .. })),
                    "stock={stock:.17e}, gas_ratio={gas_ratio:.17e}, initial_T={t:.17e}, full={full}, chemistry_calls={}; {events:?}", chemistry.calls
                );
                assert!(events.iter().any(|e| matches!(
                    e,
                    Event::StateChanged {
                        from: Phase::Solid,
                        to: Phase::Liquid,
                        ..
                    }
                )));
                assert!(v.solution.is_some());
                assert_eq!(
                    amount(&v, "Cu", Phase::Solid),
                    0.0,
                    "failed chemistry must not leak stock"
                );
                assert_eq!(
                    amount(&v, "N2", Phase::Gas),
                    amount(&before, "N2", Phase::Gas)
                );
                for key in ["Na+", "Cl-"] {
                    assert_eq!(
                        amount(&v, key, Phase::Aqueous),
                        amount(&before, key, Phase::Aqueous)
                    );
                }
                let water_before = amount(&before, "water", Phase::Solid)
                    + amount(&before, "water", Phase::Liquid);
                let water_after =
                    amount(&v, "water", Phase::Solid) + amount(&v, "water", Phase::Liquid);
                assert!((water_after - water_before).abs() <= stock * 1e-12);
                assert_eq!(amount(&v, "water", Phase::Solid) == 0.0, full);
                if !full {
                    assert!(v.temperature.0 <= states::WATER_FREEZING_K);
                }
                assert_energy(&before, &v);
            }
        }
    }
}

#[test]
fn persistent_failure_discards_seed_and_partial_chemistry_mutations() {
    let mut v = frozen(0.5, 300.0, 0.01);
    let before = format!("{v:?}");
    let mut chemistry = NeedsLiquid {
        minimum: 0.0,
        permanent_failure: true,
        corrupt: true,
        calls: 0,
    };
    let events = equilibrate_phase_coupled(&mut chemistry, &mut v).unwrap();
    assert_eq!(format!("{v:?}"), before);
    assert_eq!(chemistry.calls, 2);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::SolverFailed { .. }));
}

#[test]
fn no_superheat_or_unresolved_matter_cannot_fund_a_liquid_seed() {
    for t in [263.15, states::WATER_FREEZING_K] {
        let mut v = frozen(0.5, t, 0.01);
        let before = format!("{v:?}");
        let mut chemistry = NeedsLiquid {
            minimum: 0.1,
            permanent_failure: false,
            corrupt: false,
            calls: 0,
        };
        let events = equilibrate_phase_coupled(&mut chemistry, &mut v).unwrap();
        assert_eq!(format!("{v:?}"), before);
        assert_eq!(chemistry.calls, 1);
        assert!(matches!(events[0], Event::SolverFailed { .. }));
    }
    let mut v = frozen(0.5, 300.0, 0.01);
    v.deposit(
        SpeciesId::new("unknown-test-material"),
        Moles(1e-18),
        Phase::Aqueous,
    );
    let before = format!("{v:?}");
    let mut chemistry = NeedsLiquid {
        minimum: 0.1,
        permanent_failure: false,
        corrupt: false,
        calls: 0,
    };
    equilibrate_phase_coupled(&mut chemistry, &mut v).unwrap();
    assert_eq!(format!("{v:?}"), before);
    assert_eq!(chemistry.calls, 1);
}

#[test]
fn ice_melting_is_not_preempted_by_residual_liquids_boiling_domain() {
    let mut v = frozen(0.5, 500.0, 0.01);
    // Deliberately exceed the activity domain. Melting is physically selected
    // first; boiling of the pre-melt liquid must not veto that transition.
    v.solution = Some(SolutionInfo {
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: vec![],
        ph: 7.0,
        ionic_strength: 0.0,
        species: vec![SpeciesDetail {
            name: "Na+".into(),
            molality: 100.0,
            activity: 100.0,
        }],
        provenance: None,
        solvent_activity: None,
    });
    let events = StateEquilibrator.equilibrate(&mut v).unwrap();
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged {
                from: Phase::Solid,
                to: Phase::Liquid,
                ..
            }
        )),
        "{events:?}"
    );
}

#[test]
fn successful_retry_with_unfunded_heat_is_not_certified() {
    struct AddsHeat(NeedsLiquid);
    impl Equilibrator for AddsHeat {
        fn name(&self) -> &'static str {
            "unfunded-test-heat"
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            let events = self.0.equilibrate(v)?;
            v.temperature.0 += 10.0;
            Ok(events)
        }
    }
    let mut v = frozen(0.5, 500.0, 0.01);
    let before = format!("{v:?}");
    let mut chemistry = AddsHeat(NeedsLiquid {
        minimum: 0.1,
        permanent_failure: false,
        corrupt: false,
        calls: 0,
    });
    let events = equilibrate_phase_coupled(&mut chemistry, &mut v).unwrap();
    assert_eq!(format!("{v:?}"), before);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::SolverFailed { .. }));
}

#[test]
fn successful_retry_cannot_reassign_the_sealed_compartment() {
    struct ChangesVolume(NeedsLiquid);
    impl Equilibrator for ChangesVolume {
        fn name(&self) -> &'static str {
            "compartment-corruption-test"
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            let events = self.0.equilibrate(v)?;
            if let Headspace::Sealed { volume } = &mut v.headspace {
                volume.0 *= 2.0;
            }
            v.refresh_pressure();
            Ok(events)
        }
    }
    // Doubling a rigid volume changes neither atoms nor our ideal-gas Cv
    // ledger. The refusal therefore tests compartment ownership independently
    // of the element/energy checks.
    let mut v = frozen(0.5, 500.0, 0.01);
    let before = format!("{v:?}");
    let mut chemistry = ChangesVolume(NeedsLiquid {
        minimum: 0.1,
        permanent_failure: false,
        corrupt: false,
        calls: 0,
    });
    let events = equilibrate_phase_coupled(&mut chemistry, &mut v).unwrap();
    assert!(chemistry.0.calls >= 2);
    assert_eq!(format!("{v:?}"), before);
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0], Event::SolverFailed { .. }));
}

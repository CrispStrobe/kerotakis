//! NASA energy starts before HEAT, not from its temporary Cp-only temperature.
use kerotakis_cea::{cea_name, db, equilibrate_tp, ThermalEquilibrator};
use kerotakis_core::{vessel::HeatInput, *};
use std::collections::BTreeMap;

fn heated(key: &str, amount: f64, delivered: f64, guess: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "heated charge");
    v.deposit(SpeciesId::new(key), Moles(amount), Phase::Solid);
    v.heat_input = Some(HeatInput {
        contents: v.contents.clone(),
        temperature: Kelvin::STANDARD,
        delivered_j: delivered,
    });
    v.temperature = Kelvin(guess);
    v
}

#[test]
fn heated_magnesium_uses_delivered_energy_not_the_cp_only_temperature_guess() {
    let mut results = vec![];
    for guess in [900.0, 991.6, 1500.0] {
        let mut v = heated("Mg", 0.05, 1000.0, guess);
        let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
        let provenance = events
            .iter()
            .find_map(|event| match event {
                Event::ThermalEquilibrium { provenance, .. } => Some(provenance),
                _ => None,
            })
            .expect("budgeted HP result");
        assert!(provenance.model.contains("1000.00 J actually delivered"));
        assert!(!provenance
            .model
            .contains("latent energy is not reconstructed"));
        assert!(v.moles_of(&SpeciesId::new("MgO")).0 > 0.049);
        results.push(v);
    }
    for v in &results[1..] {
        assert!((v.temperature.0 - results[0].temperature.0).abs() < 1e-7);
        assert_eq!(v.contents, results[0].contents);
    }
}

#[test]
fn calcination_can_use_the_room_formation_anchor_plus_supplied_heat() {
    // Above the independent 26.317 kJ completion bound, while products
    // remain inside the restricted 2000 K model (unlike the old 60 kJ dose).
    let mut v = heated("CaCO3", 0.1, 30_000.0, 1800.0);
    let before = v.clone();
    let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
    assert!(v.moles_of(&SpeciesId::new("CaO")).0 > 0.09, "{events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::ThermalEquilibrium { provenance, .. } if provenance.model.contains("30000.00 J actually delivered"))));

    // The reported plume is re-equilibrated at 1000 K; its enthalpy cannot
    // equal the HP flame enthalpy. Returned air is also absent from the net
    // reservoir ledger. This carbonate route has no entrained air; recover
    // the complete vent gas atoms, and independently reconstruct the gas
    // at the retained products' flame temperature. This TP calculation does
    // not use ThermalEquilibrator's HEAT target or feed-pricing helpers.
    let source_h = 0.1 * db().get(cea_name("CaCO3").unwrap()).unwrap().h_formation + 30_000.0;
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::GasAbsorbed { .. })),
        "{events:?}"
    );
    let mut gas_atoms = BTreeMap::<String, f64>::new();
    let mut add_gas = |key: &str, moles: f64| {
        let formula =
            stoich::parse_formula(species::lookup(&SpeciesId::new(key)).unwrap().formula).unwrap();
        for (element, count) in formula.counts {
            *gas_atoms.entry(element).or_default() += count * moles;
        }
    };
    for event in &events {
        match event {
            Event::GasEvolved { species, moles, .. } => add_gas(&species.0, moles.0),
            Event::GasAbsorbed { species, moles, .. } => add_gas(&species.0, -moles.0),
            _ => (),
        }
    }
    let mut names: Vec<_> = species::REGISTRY
        .iter()
        .filter_map(|s| cea_name(s.key))
        .collect();
    names.sort_unstable();
    names.dedup();
    let gases: Vec<_> = names
        .into_iter()
        .filter_map(|name| db().get(name))
        .filter(|s| s.is_gas() && s.composition.keys().all(|e| gas_atoms.contains_key(e)))
        .collect();
    let flame = equilibrate_tp(&gas_atoms, &gases, v.temperature.0, 1.0).unwrap();
    let retained_h: f64 = v
        .contents
        .iter()
        .map(|p| {
            let composition = stoich::parse_formula(species::lookup(&p.species).unwrap().formula)
                .unwrap()
                .counts;
            let phase = db()
                .species
                .values()
                .filter(|s| {
                    !s.is_gas()
                        && s.composition == composition
                        && s.name.ends_with("(L)") == (p.phase == Phase::Liquid)
                        && s.t_range()
                            .is_some_and(|(lo, hi)| (lo..=hi).contains(&v.temperature.0))
                })
                .min_by(|a, b| {
                    a.g(v.temperature.0)
                        .unwrap()
                        .total_cmp(&b.g(v.temperature.0).unwrap())
                })
                .unwrap();
            p.moles.0 * phase.h(v.temperature.0).unwrap()
        })
        .sum();
    let residual = retained_h + flame.enthalpy - source_h;
    assert!(
        residual.abs() < 1e-3 + source_h.abs() * 2e-7,
        "independent NASA HP energy residual {residual} J; events: {events:?}"
    );
    let mut incoming = before;
    let mut outgoing = v;
    for event in events {
        match event {
            Event::GasAbsorbed { species, moles, .. } => {
                incoming.deposit(species, moles, Phase::Gas)
            }
            Event::GasEvolved { species, moles, .. } => {
                outgoing.deposit(species, moles, Phase::Gas)
            }
            _ => (),
        }
    }
    let violations = ledger::ConservedLedger::from_vessel(&incoming).check_against(
        &ledger::ConservedLedger::from_vessel(&outgoing),
        1e-7,
        1e-15,
    );
    assert!(
        violations.iter().all(|v| v.quantity == "mass"),
        "{violations:?}"
    );
}

#[test]
fn carbonate_heat_outside_represented_species_domain_is_refused_atomically() {
    let mut v = heated("CaCO3", 0.1, 60_000.0, 1800.0);
    let before = format!("{v:?}");
    let error = ThermalEquilibrator.equilibrate(&mut v).unwrap_err();
    assert!(error.to_string().contains("2000 K"), "{error}");
    assert_eq!(format!("{v:?}"), before);
}

#[test]
fn carbonate_crucible_preserves_initial_oxide_excess_carbon_and_mixed_stock() {
    for (carbonate, oxide, gas) in [(0.0, 0.1, 0.1), (0.05, 0.05, 0.05), (0.05, 0.0, 0.15)] {
        let mut v = Vessel::new(VesselId(0), "oxide/carbonate mix");
        v.thermal_mode = ThermalMode::Thermostatted(Kelvin(1400.0));
        v.temperature = Kelvin(1400.0);
        if carbonate > 0.0 {
            v.deposit(SpeciesId::new("CaCO3"), Moles(carbonate), Phase::Solid);
        }
        if oxide > 0.0 {
            v.deposit(SpeciesId::new("CaO"), Moles(oxide), Phase::Solid);
        }
        if gas > 0.0 {
            v.deposit(SpeciesId::new("CO2"), Moles(gas), Phase::Gas);
        }
        let before = ledger::ConservedLedger::from_vessel(&v);
        let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
        assert!(
            v.moles_of(&SpeciesId::new("CaO")).0 > carbonate + oxide - 1e-9,
            "{events:?}"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::GasAbsorbed { .. } | Event::SolverFailed { .. })),
            "{events:?}"
        );
        for event in events {
            if let Event::GasEvolved { species, moles, .. } = event {
                v.deposit(species, moles, Phase::Gas);
            }
        }
        let violations =
            before.check_against(&ledger::ConservedLedger::from_vessel(&v), 1e-7, 1e-15);
        assert!(
            violations.iter().all(|v| v.quantity == "mass"),
            "{violations:?}"
        );
    }
}

#[test]
fn small_carbonate_heat_budget_uses_the_valid_300_to_400_k_interval() {
    let mut v = heated("CaCO3", 0.1, 100.0, 1200.0);
    let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
    assert!((300.0..400.0).contains(&v.temperature.0), "{events:?}");
    let carbonate = db().get(cea_name("CaCO3").unwrap()).unwrap();
    let residual = 0.1 * (carbonate.h(v.temperature.0).unwrap() - carbonate.h_formation) - 100.0;
    assert!(
        residual.abs() < 0.01,
        "independent NASA heat residual {residual} J"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { .. } | Event::GasAbsorbed { .. })),
        "{events:?}"
    );
}

#[test]
fn sub_kelvin_heat_correction_commits_verified_energy_without_chemical_change() {
    let mut reference = heated("CaCO3", 0.1, 100.0, 1200.0);
    ThermalEquilibrator.equilibrate(&mut reference).unwrap();
    let mut v = heated("CaCO3", 0.1, 100.0, reference.temperature.0 + 0.5);
    let initial = v.temperature;
    let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
    assert!(
        (v.temperature.0 - reference.temperature.0).abs() < 1e-7,
        "{events:?}"
    );
    assert!(events.iter().any(|e|matches!(e,Event::TemperatureChanged{from,to,..} if *from==initial && *to==v.temperature)),"{events:?}");
    assert!(events.iter().any(|e|matches!(e,Event::ThermalEquilibrium{provenance,..} if provenance.model.contains("100.00 J actually delivered"))),"{events:?}");
    let carbonate = db().get(cea_name("CaCO3").unwrap()).unwrap();
    assert!(
        (0.1 * (carbonate.h(v.temperature.0).unwrap() - carbonate.h_formation) - 100.0).abs()
            < 0.01
    );
}

#[test]
fn hot_carbon_dioxide_without_positive_calcium_stock_uses_the_generic_route() {
    for zero_calcium in [false, true] {
        let mut v = Vessel::new(VesselId(0), "CO2 only");
        v.thermal_mode = ThermalMode::Thermostatted(Kelvin(1400.0));
        v.temperature = Kelvin(1400.0);
        v.deposit(SpeciesId::new("CO2"), Moles(0.1), Phase::Gas);
        if zero_calcium {
            v.contents.push(Portion {
                species: SpeciesId::new("CaO"),
                moles: Moles(0.0),
                phase: Phase::Solid,
            });
        }
        let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
        assert!(events.iter().any(|e|matches!(e,Event::ThermalEquilibrium{provenance,..} if !provenance.model.contains("restricted CaCO3/CaO"))),"{events:?}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::SolverFailed { .. })),
            "{events:?}"
        );
    }
}

#[test]
fn invalid_heat_budgets_are_atomic_in_the_direct_solver_api() {
    for bad in [f64::NAN, f64::INFINITY, -1.0] {
        let mut v = heated("Mg", 0.05, bad, 1200.0);
        let before = format!("{v:?}");
        assert!(ThermalEquilibrator.equilibrate(&mut v).is_err());
        assert_eq!(format!("{v:?}"), before);
    }
    for bad in [f64::NAN, -1.0, f64::MAX] {
        let mut v = heated("Mg", 0.05, 1000.0, 1200.0);
        let input = v.heat_input.as_mut().unwrap();
        input.contents[0].moles = Moles(bad);
        if bad == f64::MAX {
            input.contents.push(input.contents[0].clone());
        }
        let before = format!("{v:?}");
        assert!(ThermalEquilibrator.equilibrate(&mut v).is_err());
        assert_eq!(format!("{v:?}"), before);
    }
}

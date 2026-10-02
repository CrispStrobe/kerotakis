//! Reservoir transfers account for actual atoms, including numerical traces.
use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::*;

fn assert_boundary_balance(before: &Vessel, after: &Vessel, events: &[Event]) {
    let mut incoming = before.clone();
    let mut outgoing = after.clone();
    for event in events {
        match event {
            Event::GasAbsorbed { species, moles, .. } => {
                incoming.deposit(species.clone(), *moles, Phase::Gas)
            }
            Event::GasEvolved { species, moles, .. } => {
                outgoing.deposit(species.clone(), *moles, Phase::Gas)
            }
            _ => (),
        }
    }
    let errors = ledger::ConservedLedger::from_vessel(&incoming)
        .check_against(
            &ledger::ConservedLedger::from_vessel(&outgoing),
            1e-7,
            1e-15,
        )
        .into_iter()
        .filter(|violation| violation.quantity != "mass")
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "{errors:?}; {events:?}");
    // Registry molar masses independently round elemental weights: CaCO3
    // need not be exactly CaO + CO2 in catalog grams even when every atom
    // is conserved. Derive the maximum catalog discrepancy for these actual
    // stocks from one additive atomic-weight basis (NASA atomic records),
    // rather than relaxing the elemental contract or using a flat ppm bound.
    let catalog_rounding = |v: &Vessel| {
        v.contents
            .iter()
            .map(|p| {
                let row = species::lookup(&p.species).unwrap();
                let formula = stoich::parse_formula(row.formula).unwrap();
                let additive_mass = formula
                    .counts
                    .iter()
                    .map(|(element, count)| {
                        kerotakis_cea::db().get(element).unwrap().molar_mass * count
                    })
                    .sum::<f64>();
                p.moles.0 * (row.molar_mass - additive_mass).abs()
            })
            .sum::<f64>()
    };
    let mass_bound = catalog_rounding(&incoming)
        + catalog_rounding(&outgoing)
        + incoming.mass().0 * 1e-7
        + 1e-12;
    assert!((incoming.mass().0 - outgoing.mass().0).abs() <= mass_bound);
}

#[test]
fn open_fuels_and_calcination_close_stock_plus_net_reservoir_flows() {
    for (key, amount, phase) in [
        ("ethanol", 0.01, Phase::Liquid),
        ("Mg", 0.01, Phase::Solid),
        ("CaCO3", 0.01, Phase::Solid),
        ("H2", 0.02, Phase::Gas),
    ] {
        let mut v = Vessel::new(VesselId(0), "open hot charge");
        v.temperature = Kelvin(1200.0);
        v.deposit(SpeciesId::new(key), Moles(amount), phase);
        let before = v.clone();
        let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
        assert_boundary_balance(&before, &v, &events);
    }
}

#[test]
fn original_air_inventory_returned_to_the_room_is_an_outlet() {
    let mut v = Vessel::new(VesselId(0), "open finite charge");
    v.temperature = Kelvin(1200.0);
    v.deposit(SpeciesId::new("H2"), Moles(0.02), Phase::Gas);
    v.deposit(SpeciesId::new("N2"), Moles(0.002), Phase::Gas);
    v.deposit(SpeciesId::new("O2"), Moles(0.001), Phase::Gas);
    let before = v.clone();
    let events = ThermalEquilibrator.equilibrate(&mut v).unwrap();
    assert_boundary_balance(&before, &v, &events);
    assert!(events.iter().any(|e| matches!(e, Event::GasEvolved { species, moles, .. } if species.0 == "N2" && moles.0 > 0.0019)), "{events:?}");
    assert!(events.iter().any(|e| matches!(e, Event::GasAbsorbed { species, moles, .. } if species.0 == "O2" && moles.0 > 0.0)), "{events:?}");
}

#[test]
fn production_stack_prices_boundary_flows_on_one_step_only() {
    use std::cell::RefCell;
    use std::rc::Rc;
    struct Observer(Rc<RefCell<Vec<Vec<(SpeciesId, Moles)>>>>);
    impl Equilibrator for Observer {
        fn name(&self) -> &'static str {
            "read-only-step-start-observer"
        }
        fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            self.0
                .borrow_mut()
                .push(v.step_start.as_ref().unwrap().gas_out.clone());
            Ok(vec![])
        }
    }
    let seen = Rc::new(RefCell::new(vec![]));
    let mut stack = SolverStack::new(vec![
        Box::new(ThermalEquilibrator),
        Box::new(Observer(seen.clone())),
    ]);
    let mut bench = Bench::new();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("ethanol"),
                moles: Moles(0.01),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    seen.borrow_mut().clear();
    let events = bench
        .step_with(
            Operator::Ignite {
                vessel: VesselId(0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{events:?}"
    );
    assert!(seen
        .borrow()
        .iter()
        .any(|flows| flows.iter().any(|(id, n)| id.0 == "O2" && n.0 < 0.0)));
    assert!(bench.vessels[0].step_start.is_none());
    seen.borrow_mut().clear();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(5.5509),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(seen.borrow().iter().all(Vec::is_empty));
    assert!(bench.vessels[0].step_start.is_none());
}

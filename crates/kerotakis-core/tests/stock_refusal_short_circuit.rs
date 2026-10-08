//! Refused stock operations must not run or mutate through downstream solvers.
use kerotakis_core::script::parse_op;
use kerotakis_core::solve::{Equilibrator, PermissiveScreen, SolveError};
use kerotakis_core::stock::{StockLedger, StockUnit};
use kerotakis_core::{Bench, Event, Kelvin, Moles, Operator, SpeciesId, Vessel, VesselId};

#[derive(Default)]
struct MutatingSolver {
    calls: usize,
}
impl Equilibrator for MutatingSolver {
    fn name(&self) -> &'static str {
        "stock-refusal-observer"
    }
    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        vessel.temperature = Kelvin(400.0);
        vessel.deposit(
            SpeciesId::new("water"),
            Moles(1.0),
            kerotakis_core::Phase::Liquid,
        );
        Ok(Vec::new())
    }
}

fn assert_refusal_preserves_state(op: Operator, key: &str, unit: StockUnit) {
    for compensated in [false, true] {
        for precision in [false, true] {
            let mut bench = Bench::new();
            bench.stock = if compensated {
                StockLedger::compensated()
            } else {
                StockLedger::default()
            };
            let balance = if precision {
                if compensated {
                    1e300
                } else {
                    1e12
                }
            } else {
                0.0005
            };
            bench.stock.stock(key, balance, unit);
            if compensated && precision {
                bench.stock.draw(key, 1e12).unwrap();
            }
            let vessel_before = serde_json::to_value(bench.vessel(VesselId(0)).unwrap()).unwrap();
            let stock_before = bench.stock.clone();
            let mut solver = MutatingSolver::default();
            let events = bench
                .step_with(op.clone(), &mut solver, &PermissiveScreen)
                .unwrap();
            assert_eq!(solver.calls, 0, "refusal must not call the solver");
            assert_eq!(bench.stock, stock_before);
            assert_eq!(
                serde_json::to_value(bench.vessel(VesselId(0)).unwrap()).unwrap(),
                vessel_before
            );
            assert_eq!(
                bench.log.len(),
                1,
                "the attempted operation remains in the log"
            );
            assert!(events.iter().any(|event| matches!(
                event,
                Event::StockExhausted { .. } | Event::NotYetModeled { .. }
            )));
            assert!(!events
                .iter()
                .any(|event| matches!(event, Event::Added { .. } | Event::MaterialAdded { .. })));
        }
    }
}

#[test]
fn refused_species_draw_skips_solver_in_both_stock_modes() {
    assert_refusal_preserves_state(
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new("water"),
            moles: Moles(0.001),
            at: None,
        },
        "water",
        StockUnit::Mole,
    );
}

#[test]
fn refused_material_draw_skips_solver_in_both_stock_modes() {
    let op = parse_op("add v1 white_vinegar_5_percent 0.001g")
        .unwrap()
        .unwrap();
    assert_refusal_preserves_state(op, "white_vinegar_5_percent", StockUnit::Gram);
}

#[test]
fn invalid_species_draw_skips_solver_and_preserves_state() {
    for requested in [f64::NAN, f64::INFINITY] {
        let mut bench = Bench::new();
        let before = serde_json::to_value(bench.vessel(VesselId(0)).unwrap()).unwrap();
        let mut solver = MutatingSolver::default();
        let events = bench
            .step_with(
                Operator::Add {
                    vessel: VesselId(0),
                    species: SpeciesId::new("water"),
                    moles: Moles(requested),
                    at: None,
                },
                &mut solver,
                &PermissiveScreen,
            )
            .unwrap();
        assert_eq!(solver.calls, 0);
        assert_eq!(
            serde_json::to_value(bench.vessel(VesselId(0)).unwrap()).unwrap(),
            before
        );
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::NotYetModeled { .. })));
    }
}

#[test]
fn successful_draw_still_runs_solver() {
    let mut bench = Bench::new();
    bench.stock.stock("water", 1.0, StockUnit::Mole);
    let mut solver = MutatingSolver::default();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(0.001),
                at: None,
            },
            &mut solver,
            &PermissiveScreen,
        )
        .unwrap();
    assert_eq!(solver.calls, 1);
    assert_eq!(
        bench.vessel(VesselId(0)).unwrap().temperature,
        Kelvin(400.0)
    );
}

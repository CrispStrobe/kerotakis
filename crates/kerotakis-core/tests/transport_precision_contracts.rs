//! Source-informed scalar transport boundaries, frozen before the repair.
use kerotakis_core::*;

fn cell(id: usize, tracer: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(id), "precision cell");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.deposit(
        SpeciesId::new("passive-tracer"),
        Moles(tracer),
        Phase::Aqueous,
    );
    v
}

fn state(chain: &CellChain) -> serde_json::Value {
    serde_json::to_value(chain.cells()).unwrap()
}

#[test]
fn positive_parcel_underflow_refuses_without_erasing_a_component() {
    for minimum_in_inlet in [false, true] {
        let tiny = f64::from_bits(1);
        let mut chain =
            CellChain::new(vec![cell(0, if minimum_in_inlet { 0.0 } else { tiny })]).unwrap();
        let inlet = cell(9, if minimum_in_inlet { tiny } else { 0.0 });
        let before = state(&chain);
        assert!(
            chain.advance(&inlet, 0.5).is_err(),
            "positive half-subnormal parcel must refuse"
        );
        assert_eq!(state(&chain), before);
    }
}

#[test]
fn receiver_lost_or_inaccurate_positive_increment_refuses_atomically() {
    let ulp_half = f64::from_bits(0.5_f64.to_bits() + 1) - 0.5;
    for incoming_full_cell in [1e-20, 1.5 * ulp_half] {
        let mut chain = CellChain::new(vec![cell(0, 1.0)]).unwrap();
        let inlet = cell(9, incoming_full_cell);
        let before = state(&chain);
        assert!(
            chain.advance(&inlet, 0.5).is_err(),
            "inaccurate receiver increment must refuse"
        );
        assert_eq!(state(&chain), before);
    }
    let mut chain = CellChain::new(vec![cell(0, 1e-20), cell(1, 1.0)]).unwrap();
    let before = state(&chain);
    assert!(chain.advance(&cell(9, 0.0), 0.5).is_err());
    assert_eq!(
        state(&chain),
        before,
        "later-cell refusal must restore earlier cells"
    );
}

#[test]
fn positive_zero_debit_transport_refuses_before_reactive_solver_calls() {
    struct Counter(usize);
    impl Equilibrator for Counter {
        fn name(&self) -> &'static str {
            "must-not-run"
        }
        fn equilibrate(&mut self, _: &mut Vessel) -> Result<Vec<Event>, SolveError> {
            self.0 += 1;
            Ok(vec![])
        }
    }
    let mut chain = CellChain::new(vec![cell(0, 1.0)]).unwrap();
    let before = state(&chain);
    let mut solver = Counter(0);
    assert!(chain
        .advance_reactive(&cell(9, 0.0), 1e-20, &mut solver)
        .is_err());
    assert_eq!(state(&chain), before);
    assert_eq!(solver.0, 0);
}

#[test]
fn changed_transport_invalidates_resolved_solution_and_zero_step_preserves_it() {
    for fraction in [0.0, 0.5, 1.0] {
        let mut v = cell(0, 1.0);
        v.resolved.valid = true;
        let mut chain = CellChain::new(vec![v]).unwrap();
        let before = state(&chain);
        chain.advance(&cell(9, 0.0), fraction).unwrap();
        if fraction == 0.0 {
            assert_eq!(state(&chain), before);
        } else {
            assert!(!chain.cells()[0].resolved.valid);
        }
    }
}

#[test]
fn independently_scaled_trace_matrix_and_exact_endpoints_remain_supported() {
    for initial in [1e-100, 1e-14, 1.0] {
        for fraction in [0.0, 0.125, 0.5, 1.0] {
            let mut chain = CellChain::new(vec![cell(0, initial), cell(1, 0.0)]).unwrap();
            let step = chain.advance(&cell(9, 0.0), fraction).unwrap();
            let tracer = SpeciesId::new("passive-tracer");
            let left = chain.cells()[0].moles_of(&tracer).0;
            let arrived = chain.cells()[1].moles_of(&tracer).0;
            assert!((left / initial - (1.0 - fraction)).abs() < 1e-12);
            assert!((arrived / initial - fraction).abs() < 1e-12);
            assert!(((left + arrived) / initial - 1.0).abs() < 1e-12);
            assert_eq!(step.effluent.moles_of(&tracer).0, 0.0);
        }
    }
}

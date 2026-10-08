//! Frozen raw external-phase validation before boundary event construction.
use super::*;

fn external_raw(initial: f64, final_amount: Option<f64>) -> Result<(), SolveError> {
    let mut vessel = Vessel::new(VesselId(0), "external phase raw fixture");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
        .solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "owned crystal",
            Moles(0.003),
            Moles(0.004),
        ));
    let mut problem = partition(&vessel).expect("valid initial partition");
    problem.totals = vec![("Ca".into(), 0.0), ("Sr".into(), 0.0), ("C".into(), 0.0)];
    problem.phases = vec![("CO2(g)".into(), initial, 0.0)];
    problem.gases.clear();
    problem.external_gases = vec![ExternalGas {
        phase: "CO2(g)".into(),
        species: "CO2".into(),
        initial_moles: initial,
        kind: ExternalGasKind::Reservoir,
    }];
    let crystals = problem.solid_solutions.clone();
    let mut ions = vec![("Ca".into(), 0.0), ("Sr".into(), 0.0), ("C".into(), 0.0)];
    let value = |column: &str| match column {
        "pH" => Some(7.0),
        "mu" => Some(0.01),
        "CO2(g)" => final_amount,
        _ => None,
    };
    PhreeqcEquilibrator::apply_balance_corrections(
        &vessel,
        &problem,
        &mut ions,
        &mut [],
        &[],
        &crystals,
        &value,
    )?;
    Ok(())
}

#[test]
fn zero_external_phase_initial_and_final_remain_valid() {
    external_raw(0.0, Some(0.0)).expect("zero boundary amounts remain supported");
}

#[test]
fn missing_external_phase_final_refuses() {
    assert!(external_raw(0.0, None).is_err());
}

#[test]
fn negative_external_phase_initial_refuses() {
    assert!(external_raw(-0.001, Some(0.0)).is_err());
}

#[test]
fn nonfinite_external_phase_initial_refuses() {
    for n in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
        assert!(
            external_raw(n, Some(0.0)).is_err(),
            "invalid external initial {n} must refuse"
        );
    }
}

#[test]
fn negative_external_phase_final_refuses() {
    assert!(external_raw(0.0, Some(-0.001)).is_err());
}

#[test]
fn nonfinite_external_phase_final_refuses() {
    for n in [f64::NAN, f64::NEG_INFINITY, f64::INFINITY] {
        assert!(
            external_raw(0.0, Some(n)).is_err(),
            "invalid external final {n} must refuse"
        );
    }
}

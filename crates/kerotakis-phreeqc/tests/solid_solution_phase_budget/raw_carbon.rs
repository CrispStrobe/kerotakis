//! Frozen end-to-end raw carbon guards, before any clamping or zero-total skip.
use super::*;

fn raw_carbon(column: &str, supplied: Option<f64>) -> Result<(), SolveError> {
    let solver = PhreeqcEquilibrator::new().expect("construct native adapter");
    let mut vessel = Vessel::new(VesselId(0), "exhausted carbon raw fixture");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
        .solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "exhausted crystal",
            Moles(0.003),
            Moles(0.004),
        ));
    let mut problem = partition(&vessel).expect("valid crystal input");
    problem.totals = vec![("Ca".into(), 0.0), ("Sr".into(), 0.0), ("C".into(), 0.0)];
    problem.elements = vec!["Ca".into(), "Sr".into(), "C".into()];
    problem.phases.clear();
    problem.gases.clear();
    problem.external_gases.clear();
    let mut values = BTreeMap::<String, f64>::from([
        ("mass_H2O".into(), 1.0),
        ("Ca".into(), 0.0),
        ("Sr".into(), 0.0),
        ("C".into(), 0.0),
        ("s_Aragonite".into(), 0.003),
        ("s_Strontianite".into(), 0.004),
        ("pH".into(), 7.0),
        ("mu".into(), 0.01),
    ]);
    let states = valence_totals(&problem, "wateq4f");
    assert!(
        states.iter().any(|state| state == "C(-4)"),
        "fixture exercises the actual carbon redox branch"
    );
    for state in states {
        values.insert(state, 0.0);
    }
    match supplied {
        Some(n) => {
            values.insert(column.into(), n);
        }
        None => {
            values.remove(column);
        }
    }
    let rows = vec![
        values.keys().cloned().collect::<Vec<_>>(),
        values.values().map(ToString::to_string).collect::<Vec<_>>(),
    ];
    let value = |name: &str| -> Option<f64> {
        let index = rows[0].iter().position(|header| header == name)?;
        rows[1].get(index)?.parse().ok()
    };
    let (_, mut surfaces, exchanges, crystals, mut ions, _, _) =
        solver.readback_raw_values(&problem, "wateq4f", &rows, &value)?;
    PhreeqcEquilibrator::apply_balance_corrections(
        &vessel,
        &problem,
        &mut ions,
        &mut surfaces,
        &exchanges,
        &crystals,
        &value,
    )?;
    Ok(())
}

#[test]
fn complete_zero_carbon_readback_accepts_exact_exhaustion() {
    raw_carbon("C", Some(0.0)).expect("zero raw carbon with complete columns is valid");
}

#[test]
fn missing_raw_carbon_base_refuses_even_at_exact_exhaustion() {
    assert!(raw_carbon("C", None).is_err());
}

#[test]
fn malformed_raw_carbon_base_refuses_before_zero_skip() {
    for n in [-1.0, f64::NEG_INFINITY, f64::NAN, f64::INFINITY] {
        assert!(
            raw_carbon("C", Some(n)).is_err(),
            "invalid raw C={n} must refuse"
        );
    }
}

#[test]
fn missing_requested_carbon_state_refuses_at_exact_exhaustion() {
    assert!(raw_carbon("C(-4)", None).is_err());
}

#[test]
fn malformed_raw_carbon_state_refuses_before_clamping() {
    for n in [-1.0, f64::NEG_INFINITY, f64::NAN, f64::INFINITY] {
        assert!(
            raw_carbon("C(-4)", Some(n)).is_err(),
            "invalid raw C(-4)={n} must refuse"
        );
    }
}

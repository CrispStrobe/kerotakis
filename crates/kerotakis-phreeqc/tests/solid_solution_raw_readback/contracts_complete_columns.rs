//! Source-informed contracts: typed phase values must be validated before correction.
//! Included as a private unit module to exercise readback without executing PHREEQC.
#![cfg(test)]
use super::*;

fn readback(
    calcium: Option<&str>,
    strontium: Option<&str>,
) -> Result<Vec<SolidSolution>, SolveError> {
    let solver = PhreeqcEquilibrator::new().expect("construct readback adapter");
    let mut vessel = Vessel::new(VesselId(0), "raw typed component fixture");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
        .solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "owned mixed crystal",
            Moles(0.003),
            Moles(0.004),
        ));
    let problem = partition(&vessel).expect("valid carbonate ownership input");
    let mut headers = vec!["mass_H2O".to_string(), "Ca".into(), "Sr".into(), "C".into()];
    let mut values = vec![
        "1".to_string(),
        "0.001".into(),
        "0.002".into(),
        "0.003".into(),
    ];
    headers.push("C(-4)".into());
    values.push("0".into());
    for (name, value) in [("s_Aragonite", calcium), ("s_Strontianite", strontium)] {
        if let Some(value) = value {
            headers.push(name.into());
            values.push(value.into());
        }
    }
    let rows = vec![headers, values];
    // Match production's selected-output parsing, including parsed NaN/infinity.
    let value = |column: &str| -> Option<f64> {
        let index = rows[0].iter().position(|name| name == column)?;
        rows[1].get(index)?.parse().ok()
    };
    solver
        .readback_raw_values(&problem, "wateq4f", &rows, &value)
        .map(|result| result.3)
}

#[test]
fn positive_components_survive_raw_readback_without_projection() {
    let phases = readback(Some("0.002"), Some("0.003")).expect("valid native component output");
    assert_eq!(phases.len(), 1);
    assert_eq!(
        phases[0]
            .moles_of(SolidSolutionComponent::CalciumCarbonate)
            .0,
        0.002
    );
    assert_eq!(
        phases[0]
            .moles_of(SolidSolutionComponent::StrontiumCarbonate)
            .0,
        0.003
    );
    assert!(phases[0].has_valid_state());
}

#[test]
fn zero_components_remain_valid_empty_seed_ownership() {
    let phases = readback(Some("0"), Some("0")).expect("zero seeds remain supported");
    assert_eq!(phases[0].total_moles().0, 0.0);
    assert!(phases[0].has_valid_state());
}

fn rejects_either_component(invalid: Option<&str>) {
    for (calcium, strontium, column) in [
        (invalid, Some("0.003"), "s_Aragonite"),
        (Some("0.002"), invalid, "s_Strontianite"),
    ] {
        let error = readback(calcium, strontium)
            .expect_err("malformed native ownership must refuse before correction");
        let detail = error.to_string();
        assert!(
            detail.contains(column),
            "refusal must identify {column}: {detail}"
        );
    }
}

#[test]
fn negative_component_refuses_before_correction() {
    rejects_either_component(Some("-0.001"));
}
#[test]
fn nan_component_refuses_before_correction() {
    rejects_either_component(Some("NaN"));
}
#[test]
fn negative_infinity_component_refuses_before_correction() {
    rejects_either_component(Some("-inf"));
}
#[test]
fn positive_infinity_component_refuses_before_correction() {
    rejects_either_component(Some("inf"));
}
#[test]
fn missing_component_refuses_before_correction() {
    rejects_either_component(None);
}

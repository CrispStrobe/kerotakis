//! Frozen boundary controls include the gas equilibrium phase emitted by partition.
use super::*;

fn boundary(
    species: &str,
    phase: &str,
    include_phase: bool,
    raw_c: f64,
) -> Result<Vec<(String, f64)>, SolveError> {
    let mut vessel = Vessel::new(VesselId(0), "external boundary crystal");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
        .solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "owned crystal",
            Moles(0.003),
            Moles(0.004),
        ));
    let mut problem = partition(&vessel).expect("valid input");
    problem.totals = vec![("Ca".into(), 0.0), ("Sr".into(), 0.0), ("C".into(), 0.0)];
    problem.phases.clear();
    problem.gases.clear();
    problem.external_gases = vec![ExternalGas {
        phase: phase.into(),
        species: species.into(),
        initial_moles: 0.0,
        kind: if species == "CO2" {
            ExternalGasKind::Reservoir
        } else {
            ExternalGasKind::Dose
        },
    }];
    if include_phase {
        problem.phases.push((phase.into(), 0.0, 0.0));
    }
    let crystals = problem.solid_solutions.clone();
    let mut ions = vec![("Ca".into(), 0.0), ("Sr".into(), 0.0), ("C".into(), raw_c)];
    let value = |column: &str| match column {
        "pH" => Some(7.0),
        "mu" => Some(0.01),
        name if name == phase => Some(0.0),
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
    Ok(ions)
}

#[test]
fn actual_external_carbon_phase_does_not_require_a_solid_registry_identity() {
    let ions = boundary("CO2", "CO2(g)", true, 1.0)
        .expect("reviewed open carbon boundary with closed cations");
    assert_eq!(ions[2].1, 1.0);
}

#[test]
fn noncarbon_external_boundary_does_not_open_the_carbon_budget() {
    // Omit the phase in this seam control so an unrelated phase lookup error
    // cannot masquerade as the required carbon budget refusal.
    assert!(boundary("HBr", "HBr(g)", false, 1.0).is_err());
}

#[test]
fn noncarbon_external_phase_preserves_supported_closed_carbon() {
    boundary("HBr", "HBr(g)", true, 0.0)
        .expect("reviewed noncarbon boundary with exactly exhausted carbon");
}

//! Independently declared allocation/reconciliation controls. No PHREEQC solve.
//! Included privately to exercise the production correction seam.
use super::*;

fn controlled_fixture(
    scale: f64,
    final_ca: f64,
    final_sr: f64,
    raw: [f64; 3],
    gas: Option<Option<f64>>,
    primary: Option<(f64, f64)>,
    adjust: impl FnOnce(&mut Vessel, &mut Problem, &mut Vec<(String, f64)>),
) -> Result<Vec<(String, f64)>, SolveError> {
    let mut vessel = Vessel::new(VesselId(0), "native budget contract");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel
        .solid_solutions
        .push(SolidSolution::aragonite_strontianite(
            "owned carbonate",
            Moles(0.003 * scale),
            Moles(0.004 * scale),
        ));
    let mut problem = partition(&vessel).expect("valid input partition");
    // Independent total budget: Ca .005, Sr .004, C .009.
    // Typed inventory is separate from aqueous totals, never counted twice.
    problem.totals = vec![
        ("Ca".into(), 0.002 * scale),
        ("Sr".into(), 0.0),
        ("C".into(), 0.002 * scale),
    ];
    problem.phases.clear();
    problem.gases.clear();
    problem.external_gases.clear();
    if let Some((initial, _)) = primary {
        problem
            .phases
            .push(("Calcite".into(), initial * scale, 0.0));
    }
    if gas.is_some() {
        problem.gases.push(("CO2(g)".into(), "CO2".into(), 0.0));
    }
    let mut ions: Vec<(String, f64)> = ["Ca", "Sr", "C"]
        .into_iter()
        .zip(raw)
        .map(|(key, n)| (key.into(), n * scale))
        .collect();
    adjust(&mut vessel, &mut problem, &mut ions);
    let final_crystals = vec![SolidSolution::aragonite_strontianite(
        "owned carbonate",
        Moles(final_ca * scale),
        Moles(final_sr * scale),
    )];
    let value = |column: &str| match column {
        "pH" => Some(7.0),
        "mu" => Some(0.01),
        "Calcite" => primary.map(|(_, final_n)| final_n * scale),
        "g_CO2(g)" => gas.flatten().map(|n| n * scale),
        _ => None,
    };
    PhreeqcEquilibrator::apply_balance_corrections(
        &vessel,
        &problem,
        &mut ions,
        &mut [],
        &[],
        &final_crystals,
        &value,
    )?;
    Ok(ions)
}

fn exact() -> [f64; 3] {
    [0.003, 0.001, 0.004]
}

#[test]
fn inside_declared_raw_budget_accepts() {
    for scale in [1.0, 1e-12, 1e-150] {
        for sign in [-1.0, 1.0] {
            let mut raw = exact();
            raw[0] *= 1.0 + sign * 0.25e-7;
            assert!(controlled_fixture(scale, 0.002, 0.003, raw, None, None, |_, _, _| {}).is_ok());
        }
    }
}
#[test]
fn outside_declared_raw_budget_refuses() {
    for scale in [1.0, 1e-12, 1e-150] {
        for sign in [-1.0, 1.0] {
            let mut raw = exact();
            raw[0] *= 1.0 + sign * 2e-7;
            assert!(
                controlled_fixture(scale, 0.002, 0.003, raw, None, None, |_, _, _| {}).is_err()
            );
        }
    }
}
#[test]
fn aggregate_available_overflow_refuses() {
    assert!(
        controlled_fixture(1.0, 0.002, 0.003, exact(), None, None, |_, p, _| {
            p.totals = vec![("Ca".into(), 1e308), ("Ca".into(), 1e308)];
        })
        .is_err()
    );
}
#[test]
fn malformed_available_aqueous_refuses() {
    for amount in [-0.001, f64::NAN, f64::INFINITY] {
        assert!(
            controlled_fixture(1.0, 0.002, 0.003, exact(), None, None, |_, p, _| {
                p.totals[0].1 = amount;
            })
            .is_err()
        );
    }
}
#[test]
fn malformed_primary_phase_refuses() {
    for amount in [-0.001, f64::NAN, f64::INFINITY] {
        assert!(controlled_fixture(
            1.0,
            0.002,
            0.003,
            exact(),
            None,
            Some((0.0, amount)),
            |_, _, _| {}
        )
        .is_err());
        assert!(controlled_fixture(
            1.0,
            0.002,
            0.003,
            exact(),
            None,
            Some((amount, 0.0)),
            |_, _, _| {}
        )
        .is_err());
    }
}
#[test]
fn malformed_initial_closed_gas_refuses() {
    for amount in [-0.001, f64::NAN, f64::INFINITY] {
        assert!(controlled_fixture(
            1.0,
            0.002,
            0.003,
            exact(),
            Some(Some(0.0)),
            None,
            |v, _, _| {
                v.contents.push(Portion {
                    species: SpeciesId::new("CO2"),
                    moles: Moles(amount),
                    phase: Phase::Gas,
                });
            }
        )
        .is_err());
    }
}
#[test]
fn external_carbon_is_open_with_closed_cations() {
    let raw = [0.003, 0.001, 1.0];
    let result = controlled_fixture(1.0, 0.002, 0.003, raw, None, None, |_, p, _| {
        p.external_gases.push(ExternalGas {
            phase: "CO2(g)".into(),
            species: "CO2".into(),
            initial_moles: 0.0,
            kind: ExternalGasKind::Reservoir,
        });
    })
    .unwrap();
    assert_eq!(result[2].1, 1.0);
}
#[test]
fn external_carbon_cannot_hide_calcium_overdraw() {
    assert!(
        controlled_fixture(1.0, 0.006, 0.003, exact(), None, None, |_, p, _| {
            p.external_gases.push(ExternalGas {
                phase: "CO2(g)".into(),
                species: "CO2".into(),
                initial_moles: 0.0,
                kind: ExternalGasKind::Reservoir,
            });
        })
        .is_err()
    );
}

//! Independently declared allocation/reconciliation controls. No PHREEQC solve.
//! Included privately to exercise the production correction seam.
use super::*;

fn fixture(
    scale: f64,
    final_ca: f64,
    final_sr: f64,
    raw: [f64; 3],
    gas: Option<Option<f64>>,
    primary: Option<(f64, f64)>,
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
fn exact_exclusive_remainders_accept_without_material_correction() {
    for scale in [1.0, 1e-12, 1e-150] {
        let ions = fixture(scale, 0.002, 0.003, exact(), None, None).unwrap();
        for ((_, actual), expected) in ions.iter().zip(exact()) {
            assert!((actual / (expected * scale) - 1.0).abs() <= 32.0 * f64::EPSILON);
        }
    }
}

#[test]
fn exact_phase_exhaustion_accepts_zero_aqueous() {
    let ions = fixture(1.0, 0.005, 0.004, [0.0; 3], None, None).unwrap();
    assert!(ions.iter().all(|(_, n)| *n == 0.0));
}

#[test]
fn calcium_phase_overdraw_refuses_at_every_scale() {
    for scale in [1.0, 1e-12, 1e-150] {
        assert!(fixture(scale, 0.006, 0.003, exact(), None, None).is_err());
    }
}

#[test]
fn strontium_phase_overdraw_refuses_at_every_scale() {
    for scale in [1.0, 1e-12, 1e-150] {
        assert!(fixture(scale, 0.002, 0.005, exact(), None, None).is_err());
    }
}

#[test]
fn closed_carbon_gas_overdraw_refuses() {
    assert!(fixture(1.0, 0.002, 0.003, exact(), Some(Some(0.005)), None).is_err());
}

#[test]
fn missing_owned_gas_refuses() {
    assert!(fixture(1.0, 0.002, 0.003, exact(), Some(None), None).is_err());
}

#[test]
fn negative_owned_gas_refuses() {
    assert!(fixture(1.0, 0.002, 0.003, exact(), Some(Some(-0.001)), None).is_err());
}

#[test]
fn nonfinite_owned_gas_refuses() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(fixture(1.0, 0.002, 0.003, exact(), Some(Some(value)), None).is_err());
    }
}

#[test]
fn exact_closed_gas_budget_accepts() {
    let ions = fixture(
        1.0,
        0.002,
        0.003,
        [0.003, 0.001, 0.003],
        Some(Some(0.001)),
        None,
    )
    .unwrap();
    assert!((ions[2].1 / 0.003 - 1.0).abs() <= 32.0 * f64::EPSILON);
}

#[test]
fn primary_solid_dissolution_is_counted_once() {
    let raw = [0.004, 0.001, 0.005];
    let ions = fixture(1.0, 0.002, 0.003, raw, None, Some((0.001, 0.0))).unwrap();
    for ((_, actual), expected) in ions.iter().zip(raw) {
        assert!((actual / expected - 1.0).abs() <= 32.0 * f64::EPSILON);
    }
}

#[test]
fn primary_solid_precipitation_is_debited_once() {
    let raw = [0.002, 0.001, 0.003];
    let ions = fixture(1.0, 0.002, 0.003, raw, None, Some((0.0, 0.001))).unwrap();
    for ((_, actual), expected) in ions.iter().zip(raw) {
        assert!((actual / expected - 1.0).abs() <= 32.0 * f64::EPSILON);
    }
}

#[test]
fn primary_and_typed_overdraw_refuses() {
    assert!(fixture(1.0, 0.002, 0.003, exact(), None, Some((0.0, 0.004))).is_err());
}

#[test]
fn ten_percent_raw_excess_refuses_at_every_scale() {
    for scale in [1.0, 1e-12, 1e-150] {
        for index in 0..3 {
            let mut raw = exact();
            raw[index] *= 1.1;
            assert!(fixture(scale, 0.002, 0.003, raw, None, None).is_err());
        }
    }
}

#[test]
fn ten_percent_raw_deficit_refuses_at_every_scale() {
    for scale in [1.0, 1e-12, 1e-150] {
        for index in 0..3 {
            let mut raw = exact();
            raw[index] *= 0.9;
            assert!(fixture(scale, 0.002, 0.003, raw, None, None).is_err());
        }
    }
}

#[test]
fn zero_aqueous_with_positive_remainder_refuses() {
    for index in 0..3 {
        let mut raw = exact();
        raw[index] = 0.0;
        assert!(fixture(1.0, 0.002, 0.003, raw, None, None).is_err());
    }
}

#[test]
fn apparently_inclusive_raw_total_without_witness_refuses() {
    assert!(fixture(1.0, 0.002, 0.003, [0.005, 0.004, 0.009], None, None).is_err());
}

#[test]
fn nonfinite_raw_aqueous_refuses() {
    for index in 0..3 {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut raw = exact();
            raw[index] = value;
            assert!(fixture(1.0, 0.002, 0.003, raw, None, None).is_err());
        }
    }
}

#[test]
fn negative_raw_aqueous_refuses() {
    for index in 0..3 {
        let mut raw = exact();
        raw[index] = -0.001;
        assert!(fixture(1.0, 0.002, 0.003, raw, None, None).is_err());
    }
}

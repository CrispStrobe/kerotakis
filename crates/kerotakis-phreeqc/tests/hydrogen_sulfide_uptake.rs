//! Hydrogen sulfide bubbled into an open beaker (C1 sulfur lane,
//! 2026-11-24): the dose dissolves up to its Henry threshold as `H2S(aq)`
//! (NIST WebBook SRD 69, 0.10 mol/(kg bar)), the excess escapes, and the
//! dissolved share is a weak acid that precipitates the acid-insoluble
//! metal sulfides (CuS, PbS) and nothing where no sulfide phase exists.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn run(script: &[&str]) -> (Bench, Vec<Vec<Event>>) {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    let mut steps = Vec::new();
    for op in script {
        steps.push(
            bench
                .step_with(
                    kerotakis_core::script::parse_op(op)
                        .expect("parse")
                        .expect("a known verb"),
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step"),
        );
    }
    (bench, steps)
}

fn moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn evolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == key => Some(moles.0),
            _ => None,
        })
        .sum()
}

fn failed(steps: &[Vec<Event>]) -> bool {
    steps
        .iter()
        .flatten()
        .any(|e| matches!(e, Event::SolverFailed { .. }))
}

#[test]
fn hydrogen_sulfide_water_is_weakly_acidic() {
    // 1 mmol in 10 mL is 0.1 mol/kg, below the ~0.1 mol/kg Henry threshold
    // only just; pKa1 about 7 gives pH about 4.
    let (bench, steps) = run(&["add v1 water 10mL", "add v1 hydrogen_sulfide 0.0009mol"]);
    assert!(!failed(&steps), "{steps:#?}");
    assert!(moles(&bench, "hydrogen_sulfide", Phase::Gas) < 1e-12);
    let kept = moles(&bench, "H2S(aq)", Phase::Aqueous) + moles(&bench, "HS-", Phase::Aqueous);
    assert!((kept - 0.0009).abs() < 1e-8, "{kept}");
    let ph = bench
        .vessel(VesselId(0))
        .unwrap()
        .solution
        .clone()
        .unwrap()
        .ph;
    assert!((3.7..4.4).contains(&ph), "pH {ph}");
}

#[test]
fn a_dose_beyond_the_threshold_escapes_and_a_dry_vessel_keeps_none() {
    let (bench, steps) = run(&["add v1 water 5mL", "add v1 hydrogen_sulfide 0.002mol"]);
    let out = evolved(&steps[1], "hydrogen_sulfide");
    // 0.1013 mol/(L atm) x 5 mL = 0.51 mmol held.
    assert!((1.3e-3..1.6e-3).contains(&out), "{out}");
    let kept = moles(&bench, "H2S(aq)", Phase::Aqueous) + moles(&bench, "HS-", Phase::Aqueous);
    assert!((kept + out - 0.002).abs() < 1e-8);
    let (bench, steps) = run(&["add v1 NaCl 1g", "add v1 hydrogen_sulfide 0.001mol"]);
    assert!((evolved(&steps[1], "hydrogen_sulfide") - 0.001).abs() < 1e-12);
    assert!(moles(&bench, "H2S(aq)", Phase::Aqueous) < 1e-15);
}

#[test]
fn copper_and_lead_sulfides_precipitate_from_acid() {
    for (salt, grams, solid) in [("CuSO4", "0.1g", "CuS"), ("PbSO4", "0g", "PbS")] {
        let script: Vec<String> = if salt == "PbSO4" {
            vec![
                "add v1 water 3mL".into(),
                "add v1 Pb+2 0.0005mol".into(),
                "add v1 Cl- 0.001mol".into(),
                "add v1 HCl 0.0005mol".into(),
                "add v1 hydrogen_sulfide 0.002mol".into(),
            ]
        } else {
            vec![
                "add v1 water 5mL".into(),
                format!("add v1 {salt} {grams}"),
                "add v1 HCl 0.001mol".into(),
                "add v1 hydrogen_sulfide 0.002mol".into(),
            ]
        };
        let refs: Vec<&str> = script.iter().map(String::as_str).collect();
        let (bench, steps) = run(&refs);
        assert!(!failed(&steps), "{salt}: {steps:#?}");
        let made = moles(&bench, solid, Phase::Solid);
        assert!(made > 1e-4, "{solid}: {made}\n{steps:#?}");
    }
}

#[test]
fn a_metal_with_no_sulfide_phase_gives_no_precipitate() {
    // Magnesium has no sulfide mineral in the routed databases (MgS
    // hydrolyses): hydrogen sulfide only acidifies the solution.
    let (bench, steps) = run(&[
        "add v1 water 5mL",
        "add v1 MgSO4 0.1g",
        "add v1 hydrogen_sulfide 0.0005mol",
    ]);
    assert!(!failed(&steps), "{steps:#?}");
    let solids: Vec<_> = bench
        .vessel(VesselId(0))
        .unwrap()
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Solid && p.moles.0 > 1e-12)
        .map(|p| p.species.0.clone())
        .collect();
    assert!(solids.is_empty(), "{solids:?}");
    // The dose sits at the Henry threshold of 5 mL; the heat of dissolving
    // the salt lowers it a little, so some escapes. Sulfur is conserved.
    let kept = moles(&bench, "H2S(aq)", Phase::Aqueous) + moles(&bench, "HS-", Phase::Aqueous);
    let out = evolved(&steps[2], "hydrogen_sulfide");
    assert!(kept > 0.0003, "{kept}");
    assert!((kept + out - 0.0005).abs() < 1e-8, "{kept} + {out}");
}

#[test]
fn lead_acetate_meets_hydrogen_sulfide_without_a_solver_failure() {
    // C1 E228's tube: a sulfide fully precipitated as PbS once left a
    // 1.7e-16 mol S(-2) rounding target the readback check refused.
    let (bench, steps) = run(&[
        "add v1 water 0.5mL",
        "add v1 Pb+2 0.0001mol",
        "add v1 CH3COO- 0.0002mol",
        "add v1 hydrogen_sulfide 0.0005mol",
    ]);
    assert!(!failed(&steps), "{steps:#?}");
    assert!(moles(&bench, "PbS", Phase::Solid) > 5e-5);
}

#[test]
fn zinc_sulfide_stays_dissolved_in_molar_hydrochloric_acid() {
    // The classical H2S separation: in ~1 M HCl the FRESH zinc sulfide
    // (wateq4f ZnS(a), log K -9.052) is undersaturated, so zinc stays in
    // solution while copper (below) is precipitated. Sphalerite (log K
    // -11.618) would have come down here, and did before 2026-11-25.
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 HCl 0.01mol",
        "add v1 ZnSO4 0.001mol",
        "add v1 hydrogen_sulfide 0.002mol",
    ]);
    assert!(!failed(&steps), "{steps:#?}");
    let zns = moles(&bench, "ZnS", Phase::Solid);
    assert!(zns < 1e-7, "ZnS from 1 M HCl: {zns}\n{steps:#?}");
}

#[test]
fn copper_sulfide_still_precipitates_from_molar_hydrochloric_acid() {
    // Negative control for the zinc case: covellite has no amorphous
    // database entry, keeps its crystal and comes down from 1 M HCl.
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 HCl 0.01mol",
        "add v1 CuSO4 0.001mol",
        "add v1 hydrogen_sulfide 0.002mol",
    ]);
    assert!(!failed(&steps), "{steps:#?}");
    let cus = moles(&bench, "CuS", Phase::Solid);
    assert!(cus > 5e-4, "CuS: {cus}\n{steps:#?}");
}

#[test]
fn zinc_sulfide_precipitates_from_a_neutral_sulfide_solution() {
    // Without the acid the fresh solid is still deeply supersaturated.
    let (bench, steps) = run(&[
        "add v1 water 10mL",
        "add v1 ZnSO4 0.001mol",
        "add v1 Na2S 0.001mol",
    ]);
    assert!(!failed(&steps), "{steps:#?}");
    let zns = moles(&bench, "ZnS", Phase::Solid);
    assert!(zns > 5e-4, "ZnS: {zns}\n{steps:#?}");
}

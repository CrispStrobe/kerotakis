//! Nitrous acid on the routed databases (2026-11-27).
//!
//! wateq4f and minteq.v4 define NO2- (N(3)) but no `NO2- + H+ = HNO2`
//! row, so a nitrite solution had no conjugate weak acid: sodium nitrite
//! read neutral and acidified nitrite had no buffer. The row is borrowed
//! from llnl.dat (log_k 3.2206, stated delta_H) in
//! data/thermo/shipped-complex-borrows.dat. These controls are physical
//! bounds, not fitted readbacks.
#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn run(script: &[&str]) -> Bench {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    for op in script {
        bench
            .step_with(
                kerotakis_core::script::parse_op(op)
                    .expect("parse")
                    .expect("a known verb"),
                &mut stack,
                &PermissiveScreen,
            )
            .unwrap_or_else(|e| panic!("{op}: {e}"));
    }
    bench
}

fn vessel(bench: &Bench) -> &Vessel {
    bench.vessel(VesselId(0)).unwrap()
}

fn molality(bench: &Bench, name: &str) -> f64 {
    vessel(bench)
        .solution
        .as_ref()
        .map(|s| {
            s.species
                .iter()
                .filter(|x| x.name == name)
                .map(|x| x.molality)
                .sum()
        })
        .unwrap_or(0.0)
}

#[test]
fn the_borrowed_row_is_llnls_stated_constant() {
    let text = std::str::from_utf8(kerotakis_phreeqc::complexation::BORROWED).unwrap();
    let at = text.find("NO2- + H+ = HNO2").expect("nitrous acid row");
    let row = &text[at..];
    assert!(row.contains("log_k 3.2206"));
    assert!(row.contains("-delta_H -14.782 kJ/mol"));
    // The analytic fit (which disagrees with the stated log_k) is not taken.
    assert!(!row.lines().take(4).any(|l| l.contains("-analytic")));
}

#[test]
fn sodium_nitrite_hydrolyses_mildly_alkaline() {
    // 0.01 mol NaNO2 in 100 mL: Kb = Kw/Ka = 1e-14/10^-3.22 = 1.7e-11,
    // [OH-] = sqrt(0.1 * 1.7e-11) = 1.3e-6, pH about 8.1, HNO2 about
    // 1.3e-6 mol/kg. If the open beaker also takes up room-air CO2
    // (p 4e-4 atm) the charge balance h + HNO2 = OH- + HCO3- gives pH about
    // 6.7 and HNO2 about 3e-5. Either way HNO2 is present; without the row
    // it is exactly absent. Bounds span both cases plus activity effects.
    let bench = run(&["add v1 water 100mL", "add v1 NaNO2 0.01mol"]);
    let ph = vessel(&bench).solution.as_ref().expect("solved").ph;
    assert!(ph > 6.4 && ph < 8.6, "pH {ph}");
    let hno2 = molality(&bench, "HNO2");
    assert!(hno2 > 3e-7 && hno2 < 1e-4, "HNO2 {hno2}");
    // Matter still books back under the nitrite ion.
    assert!((vessel(&bench).moles_of(&SpeciesId::new("NO2-")).0 - 0.01).abs() < 1e-6);
}

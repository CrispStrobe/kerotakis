//! PLAN P3s: the two colligative numbers a school textbook prints, taken
//! off this bench rather than out of a table.
//!
//! Both constants the answers rest on are derived in `states.rs` from
//! water's enthalpies of fusion and vaporisation rather than tabulated, and
//! the particle count comes from PHREEQC's own speciation rather than from
//! a looked-up van 't Hoff factor. So neither of these tests can pass by
//! having the right answer written down somewhere; the whole chain has to
//! be right at once.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )))])
}

fn step(bench: &mut Bench, solvers: &mut SolverStack, operator: Operator) -> Vec<Event> {
    bench
        .step_with(operator, solvers, &PermissiveScreen)
        .expect("bench step")
}

fn add(bench: &mut Bench, solvers: &mut SolverStack, key: &str, moles: f64) {
    step(
        bench,
        solvers,
        Operator::Add {
            vessel: VesselId(0),
            species: SpeciesId::new(key),
            moles: Moles(moles),
            at: None,
        },
    );
}

/// One mole of sodium chloride in one kilogram of water: 58.4 g, the exact
/// spoonful every textbook uses, and it freezes near −3.4 °C.
///
/// Not −3.72. Two moles of particles per mole of salt would give the ideal
/// 2 × 1.86, and real brine at this concentration does not manage it: some
/// of the sodium and chloride is paired, so the particle count PHREEQC
/// reports is below two per formula unit and the depression is below the
/// ideal. That gap is the reason the van 't Hoff factor is a *measured*
/// quantity in a textbook and a *counted* one here.
#[test]
fn a_textbook_spoonful_of_salt_freezes_the_water_near_minus_three_point_four() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    // 1 kg of water is 55.51 mol; 1 mol of NaCl is 58.44 g.
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "NaCl", 1.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    let freezing_c = transitions.freezing_k - 273.15;
    assert!(
        (freezing_c + 3.4).abs() < 0.4,
        "1 molal NaCl freezes near -3.4 C; this bench says {freezing_c:.3} C \
         from {:.4} mol/kg of counted particles (an ideal i = 2 would give \
         -3.72 C, and ion pairing is why it is less)",
        transitions.solute_molality
    );
    assert!(
        transitions.solute_molality > 1.5 && transitions.solute_molality < 2.0,
        "a mole of NaCl must count as more than one particle and fewer than \
         two: {:.4} mol/kg",
        transitions.solute_molality
    );
}

/// One mole of sucrose in one kilogram of water: 342 g, and it boils at
/// 100.5 °C.
///
/// Sucrose is the case that has to be got right by counting rather than by
/// speciating: it is a non-electrolyte, no shipped database carries it, and
/// reading that silence as "nothing is dissolved" is what used to make a
/// sugar solution boil at exactly 100.0 °C.
#[test]
fn a_mole_of_sugar_raises_the_boiling_point_by_half_a_degree() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "sucrose", 1.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    let boiling_c = transitions.boiling_k - 273.15;
    assert!(
        (boiling_c - 100.51).abs() < 0.1,
        "342 g of sucrose in a kilogram of water boils at 100.5 C; this bench \
         says {boiling_c:.3} C from {:.4} mol/kg of particles",
        transitions.solute_molality
    );
    assert!(
        (transitions.boiling_elevation() - 0.512).abs() < 0.02,
        "the elevation is K_b itself at one molal: {:.4} K",
        transitions.boiling_elevation()
    );
    // And the other end of the same particle count: a non-electrolyte
    // depresses the freezing point by K_f, four times as far.
    assert!(
        (transitions.freezing_depression() - 1.86).abs() < 0.05,
        "the same one molal depresses freezing by K_f = 1.86 K: {:.4} K",
        transitions.freezing_depression()
    );
}

/// Road salt, put the way a learner asks it: does the ice on the path
/// actually melt, and does the bench agree that sugar would be a worse buy?
///
/// Equal MASSES, because that is what is in the bag. 58.4 g of salt is one
/// mole and dissociates; 58.4 g of sucrose is 0.17 mol and does not. The
/// factor of about twelve between the two depressions is the entire
/// argument for why the gritter carries salt.
#[test]
fn salt_beats_sugar_on_the_path_by_about_a_factor_of_twelve() {
    let depression = |key: &str, moles: f64| {
        let mut bench = Bench::new();
        let mut solvers = stack();
        add(&mut bench, &mut solvers, "water", 55.51);
        add(&mut bench, &mut solvers, key, moles);
        let vessel = bench.vessel(VesselId(0)).unwrap();
        solve::vessel_transitions(vessel).0.freezing_depression()
    };
    // 58.44 g of each: 1.0 mol of NaCl, 0.1707 mol of sucrose.
    let salt = depression("NaCl", 1.0);
    let sugar = depression("sucrose", 0.1707);
    assert!(
        salt / sugar > 9.0 && salt / sugar < 15.0,
        "equal masses are not equal particle counts: salt {salt:.3} K against \
         sugar {sugar:.3} K, a ratio of {:.1}",
        salt / sugar
    );
}

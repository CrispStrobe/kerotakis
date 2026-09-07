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
/// spoonful every textbook uses.
///
/// The bench says −3.72 °C and a thermometer in real brine says about
/// −3.4, and the eight per cent between them is a boundary of this model
/// rather than an error in it — so it is asserted from BOTH ends here,
/// which is the only way a test can hold a stated approximation to
/// account.
///
/// The particle count is not where the gap is. Counting is what this bench
/// does honestly: the speciation is asked how many particles there are
/// rather than a van 't Hoff factor being looked up, and for this salt the
/// answer really is two — no database this bench LOADS defines an aqueous
/// NaCl ion pair (minteq.v4 carries `Halite`, which is the solid), so
/// nothing is paired and nothing pretends to be.
///
/// **That scope word is load-bearing, and it used to be missing.** The
/// sentence read "no database shipped with the bench", and one shipped
/// here does: `vendor/iphreeqc/database/llnl.dat` line 5733 writes
/// `Na+ + Cl- = NaCl`, `log_k -0.777`, with a calculated ΔH beside it.
/// Vendored and not routed is a different claim from absent, and this is
/// the file where a stated approximation is held to account, so it is the
/// last place that should blur them.
///
/// **The pair is deliberately NOT borrowed, and the arithmetic is why.**
/// log K −0.777 is K = 0.167, so with ideal ionic activities about one
/// formula unit in seven would be paired; at one molal, where γ± ≈ 0.66
/// and the ion-activity product is nearer 0.44, about one in fourteen.
/// That moves the particle count from 2.00 to roughly 1.93 and this test's
/// answer from −3.72 °C to about −3.59 — most of the way to −3.4, by
/// precisely the mechanism this comment and PLAN's P3s item both say is
/// not the problem. It would be a better number reached through worse
/// physics, and it would then hide the real correction behind a partial
/// cancellation, which is the failure mode that is hardest to find later.
///
/// The decisive evidence is which datasets carry the pair. Not
/// phreeqc.dat, not wateq4f.dat, not minteq.v4.dat — and above all not
/// pitzer.dat, the dataset built FOR concentrated brine and the one that
/// would need an ion pair most if the deviation at one molal were
/// speciation. pitzer has none; it handles Na⁺/Cl⁻ with virial
/// coefficients instead (`PITZER`, `-B0`, `Cl-  Na+  7.534e-2`, with `-B1`
/// and `-C0` rows beside it), which is to say it corrects the SOLVENT's
/// activity and leaves both ions free. llnl's `NaCl` is a fitting device
/// that extends a Debye–Hückel-family database's range, not a claim that a
/// fourteenth of dissolved salt is undissociated, and its log K is fitted
/// to llnl's own activity model rather than to the one wateq4f runs. So
/// the fix stays the one PLAN already named — the osmotic coefficient,
/// which PHREEQC computes and this bench does not yet read.
///
/// The gap is that `states::transitions` applies the DILUTE-solution law,
/// ΔT = K_f · m, at one molal, where the solvent's activity is no longer
/// its mole fraction. A textbook's i ≈ 1.85 for this solution is that
/// activity correction wearing the particle count's clothes — it is not a
/// claim that fifteen per cent of the salt is undissociated. PLAN's P3s
/// text points at the fix ("PHREEQC gives us the osmotic coefficient
/// already"); until it is wired, the law is used where it is about nine
/// per cent optimistic and this test is where that is written down.
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

    // What the model IS: two counted particles through the dilute law.
    assert!(
        (transitions.solute_molality - 2.0).abs() < 0.05,
        "a mole of NaCl dissociates into two counted particles and none of \
         the shipped databases pairs them back up: {:.4} mol/kg",
        transitions.solute_molality
    );
    assert!(
        (freezing_c + 3.72).abs() < 0.06,
        "two molal of particles through K_f = 1.86 is -3.72 C: got {freezing_c:.3} C"
    );

    // What the world is, and how far the model is from it. Widening this
    // band would hide the approximation; narrowing it would fail on a
    // change that made the answer BETTER.
    assert!(
        (freezing_c + 3.4).abs() < 0.4,
        "real 1 molal brine freezes near -3.4 C and the dilute law is \
         entitled to be a few tenths optimistic, not more: {freezing_c:.3} C"
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

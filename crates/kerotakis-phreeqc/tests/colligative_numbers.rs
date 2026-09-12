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
/// The bench says −3.44 °C and a thermometer in real brine says about
/// −3.4, so it is asserted from BOTH ends here — the model's own number and
/// the world's — which is the only way a test can hold a stated
/// approximation to account. It said −3.72 until 2026-09-11, and what
/// closed the eight per cent is worth keeping written down, because two
/// plausible explanations for that gap were wrong.
///
/// **It was never the particle count.** Counting is what this bench does
/// honestly: the speciation is asked how many particles there are rather
/// than a van 't Hoff factor being looked up, and for this salt the answer
/// really is two — no database this bench LOADS defines an aqueous NaCl ion
/// pair (minteq.v4 carries `Halite`, which is the solid), so nothing is
/// paired and nothing pretends to be.
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
/// precisely the mechanism this comment says is not the problem. It would
/// be a better number reached through worse physics, and it would then
/// hide the real correction behind a partial cancellation, which is the
/// failure mode that is hardest to find later.
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
/// fourteenth of dissolved salt is undissociated.
///
/// **So the fix was the solvent's activity, and it is now wired.** The gap
/// was that `states::transitions` applied the DILUTE-solution law, ΔT =
/// K_f·m, at one molal, where the solvent's activity is no longer its mole
/// fraction. A textbook's i ≈ 1.85 for this solution is that activity
/// correction wearing the particle count's clothes. The bench now computes
/// `1/T_f = 1/T_f° − (R/ΔH_fus)·ln a_w` and takes a_w from the same
/// pitzer.dat speciation that answered everything else about this beaker:
/// a_w = 0.9668, an osmotic coefficient of 0.937 against a measured 0.936,
/// and −3.44 °C.
///
/// **And it was never a NEW number, either.** The water activity has been
/// on the wire the whole time, as the `H2O` row of the ordinary species
/// distribution — the same rows `particles` draws its census from. Nothing
/// was added to the readback, the cache or the wasm path to reach it.
///
/// What is left between −3.44 and a measurement is the osmotic
/// coefficient's own temperature dependence: PHREEQC solved this solution
/// at the vessel's temperature, near 24 °C after the dissolution
/// endotherm, and φ for this salt falls by about a per cent between there
/// and the freezing point, which is worth a few hundredths of a kelvin in
/// the direction of the measurement. Published depressions for 1.000 molal
/// NaCl sit between 3.37 and 3.44 K depending on the source, which is why
/// the world-facing band below is a tenth and a half rather than a
/// hundredth.
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

    // The count, unchanged and still counted rather than assumed.
    assert!(
        (transitions.solute_molality - 2.0).abs() < 0.05,
        "a mole of NaCl dissociates into two counted particles and none of \
         the shipped databases pairs them back up: {:.4} mol/kg",
        transitions.solute_molality
    );
    // The activity, and where it came from. An ideal route here would mean
    // the ion-interaction speciation was not consulted, and the test would
    // pass its temperature band by 0.16 K while having lost the point.
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IonInteraction,
        "one molal brine routes to pitzer.dat, and its solvent activity is \
         the whole of this fix"
    );
    let phi = transitions.osmotic_coefficient();
    assert!(
        (phi - 0.937).abs() < 0.01,
        "the osmotic coefficient of 1 molal NaCl is 0.936 by measurement: {phi:.4}"
    );
    assert!(
        (transitions.water_activity() - 0.9668).abs() < 0.002,
        "a_w = {:.5}",
        transitions.water_activity()
    );

    // What the model says.
    assert!(
        (freezing_c + 3.44).abs() < 0.06,
        "the solvent-activity relation puts one molal brine at -3.44 C: got {freezing_c:.3} C"
    );
    // What the world says. Narrowing this would fail on a change that made
    // the answer better; widening it would let the dilute law back in — its
    // -3.72 is outside this band, which is the point of keeping the band.
    assert!(
        (freezing_c + 3.4).abs() < 0.15,
        "real 1 molal brine freezes near -3.4 C: {freezing_c:.3} C"
    );
    // And the dilute law, named so the regression is visible rather than
    // remembered: it is 8 % steeper than the relation that replaced it.
    let dilute_law = -states::cryoscopic_constant() * transitions.solute_molality;
    assert!(
        dilute_law < freezing_c - 0.2,
        "the dilute law would have said {dilute_law:.3} C"
    );
    // The van 't Hoff factor a table prints for this solution, computed
    // rather than looked up. Textbooks print 1.85.
    let i = transitions.effective_vant_hoff_factor(1.0).unwrap();
    assert!((i - 1.85).abs() < 0.04, "i = {i:.3}");
}

/// A saturated chloride brine boils where a cook finds it, and that is the
/// other half of the same correction.
///
/// Boiling-point elevation does NOT need its own treatment — it is the same
/// derivation with the vapour on the other side of the equality, and it
/// takes the same a_w — but it does need its own test, because the
/// correction changes SIGN, and a relation pinned only in the dilute half
/// would be pinned only where the two laws agree in direction.
///
/// Two crossings, and they are not the same crossing. A chloride's osmotic
/// coefficient passes one near 2.3 molal (0.955 at 1.5, 0.983 at 2.0, 1.013
/// at 2.5 by measurement), which is where a_w stops falling faster than the
/// ideal solvent's and starts falling slower. The relation itself overtakes
/// ΔT = K·m a little later, because the logarithm's own curvature holds it
/// back: at 2 molal NaCl the depression is 7.12 K against the law's 7.44,
/// and it is not until about 2.9 molal that it passes. So below roughly
/// three molal the dilute law OVER-states both shifts and above it
/// UNDER-states them, which is why one test at each end is the least this
/// can be pinned with.
///
/// Six molal NaCl is the second case: pitzer.dat reports φ = 1.27, the bench
/// says 108.0 °C, a measurement of a saturated brine says about 108.7, and
/// the dilute law said 106.1. A correction worth nearly two degrees, in the
/// opposite direction from the one the freezing test pins.
#[test]
fn a_saturated_brine_boils_hotter_than_the_dilute_law_allows() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "NaCl", 6.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IonInteraction
    );
    assert!(
        transitions.osmotic_coefficient() > 1.0,
        "a concentrated chloride's osmotic coefficient passes one, and that \
         is why the dilute law under-states this end: {:.3}",
        transitions.osmotic_coefficient()
    );
    let boiling_c = transitions.boiling_k - 273.15;
    assert!(
        (boiling_c - 108.0).abs() < 0.4,
        "a saturated brine boils near 108 C: {boiling_c:.2} C"
    );
    assert!(
        transitions.boiling_elevation()
            > states::ebullioscopic_constant() * transitions.solute_molality,
        "above three molal the relation is STEEPER than the dilute law, not \
         shallower: {:.3} K against the law's {:.3} K",
        transitions.boiling_elevation(),
        states::ebullioscopic_constant() * transitions.solute_molality
    );
    // And it is still inside the dataset's stated range. This brine's
    // ionic strength is about 6.2 mol/kgw — halite saturates near 6.11 in
    // this dataset — against the ion-interaction ceiling of 20, which
    // `pitzer.dat`'s own evaporite sequence justifies: it carries
    // `MgCl2_4H2O` and `Carnallite`, and bischofite saturates near I = 17.
    assert!(
        transitions.within_model_range(),
        "a saturated chloride brine is the case pitzer.dat exists for"
    );

    // …and the stage shows the boil while withholding the freeze, because
    // the two boundaries are different boundaries. The activity model is
    // inside its range here, so the boil is an ordinary answer; the
    // liquidus this relation computes is −25.7 °C, which is past the 252 K
    // eutectic boundary `StateEquilibrator` refuses to cross. Drawing that
    // plateau would promise a temperature the solver has already declined
    // to reach. The dilute law printed −22.3 °C for the same beaker and was
    // wrong the same way, only by less.
    let picture = scene::scene_vessel(vessel);
    assert!(
        picture.melting_point_k.is_none(),
        "a saturated brine's liquidus is below the declared boundary and \
         must not be drawn: {:?}",
        picture.melting_point_k
    );
    assert!(
        picture
            .boiling_point_k
            .is_some_and(|k| (k - transitions.boiling_k).abs() < 1e-9),
        "the boil is unaffected and is the same number the state layer \
         computed: {:?}",
        picture.boiling_point_k
    );
    assert!(transitions.freezing_k < states::BRINE_MODEL_MIN_K);
}

/// One mole of sucrose in one kilogram of water: 342 g, and it boils at
/// 100.5 °C.
///
/// Sucrose is the case that has to be got right by counting rather than by
/// speciating: it is a non-electrolyte, no shipped database carries it, and
/// reading that silence as "nothing is dissolved" is what used to make a
/// sugar solution boil at exactly 100.0 °C.
///
/// It is also the case that shows what the 2026-09-11 activity route does
/// NOT do. No database carries sucrose, so no speciation computed a solvent
/// activity for this beaker and Raoult's law stands in: a_w is the
/// solvent's mole fraction, the route says `IdealSolution` rather than
/// pretending otherwise, and the answers move by the curvature of the
/// logarithm alone — the depression from 1.859 K to 1.831, the elevation
/// from 0.513 K to 0.509. Measurements of one molal sucrose put the
/// depression at about 1.86, because this sugar's own osmotic coefficient
/// is 1.02 rather than 1 and pushes back almost exactly as far as the
/// linearisation did. The bench does not know that number and does not
/// pretend to: a two per cent shortfall, declared ideal, is the honest
/// form of not knowing, and it is inside this test's band from both sides.
#[test]
fn a_mole_of_sugar_raises_the_boiling_point_by_half_a_degree() {
    let mut bench = Bench::new();
    let mut solvers = stack();
    add(&mut bench, &mut solvers, "water", 55.51);
    add(&mut bench, &mut solvers, "sucrose", 1.0);

    let vessel = bench.vessel(VesselId(0)).unwrap();
    let (transitions, _) = solve::vessel_transitions(vessel);
    assert_eq!(
        transitions.activity_route(),
        states::ActivityRoute::IdealSolution,
        "no database carries sucrose, so nothing computed a solvent activity \
         for it and the bench must say so rather than assume one"
    );
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

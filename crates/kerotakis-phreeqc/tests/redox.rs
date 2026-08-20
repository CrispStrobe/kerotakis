//! Coupled redox: an oxidant and a reductant that actually react, and the
//! refusal when the electron ledger cannot be made to balance.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn add(bench: &mut Bench, eq: &mut PhreeqcEquilibrator, v: VesselId, key: &str, moles: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            eq,
            &PermissiveScreen,
        )
        .expect("step");
}

/// `KMnO4` into `FeSO4`: permanganate to Mn(II), iron(II) to iron(III), in
/// the 1:5 ratio the half-equations demand — found by asking the
/// thermodynamics for the pe that balances the electrons, not by applying a
/// reaction rule.
fn titrate(kmno4: f64) -> Vec<RedoxState> {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.55);
    add(&mut bench, &mut eq, v, "HCl", 0.01);
    add(&mut bench, &mut eq, v, "FeSO4", 0.005);
    add(&mut bench, &mut eq, v, "KMnO4", kmno4);
    bench
        .vessel(v)
        .expect("vessel")
        .solution
        .as_ref()
        .map(|s| s.redox.clone())
        .unwrap_or_default()
}

fn fraction(states: &[RedoxState], element: &str, oxidation: i32) -> f64 {
    let total: f64 = states
        .iter()
        .filter(|s| s.element == element)
        .map(|s| s.molality)
        .sum();
    let want: f64 = states
        .iter()
        .filter(|s| s.element == element && s.oxidation == oxidation)
        .map(|s| s.molality)
        .sum();
    if total > 0.0 {
        want / total
    } else {
        0.0
    }
}

/// One permanganate oxidises five iron(II), so a fifth of an equivalent
/// oxidises a fifth of the iron. The ratio is the answer, not an input.
#[test]
fn permanganate_oxidises_iron_in_the_ratio_the_half_equations_give() {
    for (kmno4, oxidised) in [(0.0002, 0.20), (0.0005, 0.50)] {
        let states = titrate(kmno4);
        assert!(
            (fraction(&states, "Fe", 3) - oxidised).abs() < 0.02,
            "{kmno4} mol MnO4- should oxidise {:.0}% of the iron, got {:.1}%: {states:?}",
            oxidised * 100.0,
            fraction(&states, "Fe", 3) * 100.0,
        );
        assert!(
            fraction(&states, "Mn", 2) > 0.98,
            "all the manganese should end at Mn(II) below equivalence, got {states:?}"
        );
    }
}

/// Past equivalence the books cannot be balanced, and the bench must say so
/// rather than inventing the electrons.
///
/// 0.0015 mol of permanganate needs 0.0075 mol of electrons; 0.005 mol of
/// iron(II) can supply 0.005. The missing 0.0025 mol would have to come from
/// oxidising water or chloride, which this bench does not carry — and
/// PHREEQC will do it silently if asked, which is what used to happen: the
/// bisection ran to the edge of its bracket, every last manganese came back
/// as Mn(II), and 12% of the electron inventory appeared from nowhere. A
/// colourless answer to the one titration whose entire point is that the
/// excess stays purple.
///
/// The refusal is the honest outcome, not a gap: the solver declines, the
/// stack carries on, and the vessel is reported with its elements in the
/// states they were added in — with the routing saying exactly that.
#[test]
fn excess_oxidant_is_refused_rather_than_balanced_from_nowhere() {
    let states = titrate(0.0015);
    assert!(
        fraction(&states, "Mn", 7) > 0.98,
        "with the coupling refused, manganese should be shown as added — Mn(VII) — \
         not reduced by electrons that do not exist: {states:?}"
    );
    assert!(
        fraction(&states, "Fe", 2) > 0.98,
        "and the iron should be shown as added too: {states:?}"
    );
}

/// Swapping the acid changes nothing, and that is the point.
///
/// The tempting story about the refusal above is the textbook one: that
/// permanganate titrations are run in sulfuric acid because hydrochloric
/// acid's chloride gets oxidised. It is a good story and it is not what is
/// happening here. The electrons are owed by the solvent, so the beaker is
/// refused either way, with the same residual to four figures.
///
/// The test exists to stop that story being written back into a codex
/// entry: an entry framed as "the bench refuses this, and the refusal is
/// why your lab reaches for sulfuric" would be teaching something the
/// engine does not show.
#[test]
fn the_refusal_does_not_depend_on_which_acid() {
    let mut hydrochloric = None;
    let mut sulfuric = None;
    for (acid, slot) in [("HCl", &mut hydrochloric), ("H2SO4", &mut sulfuric)] {
        let mut eq = PhreeqcEquilibrator::new().expect("engine");
        let mut bench = Bench::new();
        let v = VesselId(0);
        add(&mut bench, &mut eq, v, "water", 5.55);
        add(&mut bench, &mut eq, v, acid, 0.005);
        add(&mut bench, &mut eq, v, "FeSO4", 0.005);
        add(&mut bench, &mut eq, v, "KMnO4", 0.0015);
        let states = bench
            .vessel(v)
            .expect("vessel")
            .solution
            .as_ref()
            .map(|s| s.redox.clone())
            .unwrap_or_default();
        *slot = Some(fraction(&states, "Mn", 7));
    }
    assert!(
        hydrochloric.expect("hcl") > 0.98 && sulfuric.expect("h2so4") > 0.98,
        "excess permanganate is refused in both acids — the chloride is not what \
         decides it: HCl {hydrochloric:?}, H2SO4 {sulfuric:?}"
    );
}

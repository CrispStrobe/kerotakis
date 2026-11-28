//! Coupled redox: an oxidant and a reductant that actually react, and the
//! refusal when the electron ledger cannot be made to balance.

#![cfg(feature = "engine")]

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

/// Electrons booked in the dissolved oxidation states, mol. 5.55 mol of
/// water is 0.1 kg; the reaction's own water shifts that by under 0.1%.
fn electron_sum(states: &[RedoxState], kgw: f64) -> f64 {
    states
        .iter()
        .filter(|s| s.element == "Fe" || s.element == "Mn")
        .map(|s| s.oxidation as f64 * s.molality * kgw)
        .sum()
}

/// Past equivalence the excess permanganate stays permanganate.
///
/// 0.0015 mol of permanganate needs 0.0075 mol of electrons; 0.005 mol of
/// iron(II) can supply 0.005. Thermodynamics says the rest comes from
/// oxidising water, and PHREEQC will do that if asked: before 2026-11-28
/// this beaker was first refused (its pe lies above the stability field of
/// water) and had been, earlier still, returned with every manganese as
/// Mn(II) and 12% of the electrons from nowhere. Water is not oxidised by
/// permanganate on a bench, which is why the excess stays purple, and the
/// engine now holds it so (`WATER_INERT_OXIDANTS`): all the iron is
/// iron(III), 1 mmol of manganese is Mn(II), the remaining 0.5 mmol is
/// still Mn(VII), and the electron books close.
#[test]
fn excess_permanganate_stays_permanganate_and_the_books_balance() {
    let states = titrate(0.0015);
    assert!(
        fraction(&states, "Fe", 3) > 0.999,
        "every iron(II) is oxidised past equivalence: {states:?}"
    );
    assert!(
        (fraction(&states, "Mn", 7) - 1.0 / 3.0).abs() < 0.005,
        "a third of the manganese is the excess and stays Mn(VII): {states:?}"
    );
    assert!(
        (fraction(&states, "Mn", 2) - 2.0 / 3.0).abs() < 0.005,
        "two thirds took the iron's electrons: {states:?}"
    );
    // 0.005 Fe(II) + 0.0015 Mn(VII) went in: 0.0205 mol of oxidation.
    let kgw = 0.1;
    let booked = electron_sum(&states, kgw);
    assert!(
        (booked - 0.0205).abs() < 0.0205 * 3e-3,
        "the electron ledger must close: booked {booked:.6e} of 2.05e-2"
    );
}

/// An acid-starved flask is refused, and refused for the right reason.
///
/// With only 0.005 mol HCl the 8 protons each permanganate consumes run
/// out, the pH climbs, and the iron(III) that forms precipitates as its
/// hydroxide. The coupled balance counts dissolved oxidation states only,
/// so a root found there would leave too much permanganate standing
/// against electrons locked in the solid. The bench declines that answer
/// and shows the elements as added. In sulfuric acid at the same amount
/// the protons suffice, no solid forms, and the excess is balanced — so
/// the acid matters through its protons, not through its chloride.
#[test]
fn an_acid_starved_flask_is_refused_and_a_sulfuric_one_is_balanced() {
    let mut fractions = Vec::new();
    for acid in ["HCl", "H2SO4"] {
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
        fractions.push((fraction(&states, "Mn", 7), fraction(&states, "Fe", 2)));
    }
    let (hcl_mn7, hcl_fe2) = fractions[0];
    let (h2so4_mn7, h2so4_fe2) = fractions[1];
    assert!(
        hcl_mn7 > 0.98 && hcl_fe2 > 0.98,
        "refused: shown as added, Mn(VII) {hcl_mn7:.3}, Fe(II) {hcl_fe2:.3}"
    );
    assert!(
        (h2so4_mn7 - 1.0 / 3.0).abs() < 0.005 && h2so4_fe2 < 1e-3,
        "balanced: Mn(VII) {h2so4_mn7:.3}, Fe(II) {h2so4_fe2:.3}"
    );
}

/// Solve a titration and hand back both what happened and what the beaker
/// ended up being, since the honesty of a redox answer lives in both.
fn titrate_in(acid: &str, kmno4: f64) -> (Vec<Event>, Option<SolutionInfo>) {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.55);
    add(&mut bench, &mut eq, v, acid, 0.005);
    add(&mut bench, &mut eq, v, "FeSO4", 0.005);
    let events = bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new("KMnO4"),
                moles: Moles(kmno4),
                at: None,
            },
            &mut eq,
            &PermissiveScreen,
        )
        .expect("step");
    (events, bench.vessel(v).expect("vessel").solution.clone())
}

/// The potential jumps at the equivalence point, which is what makes the
/// endpoint detectable.
///
/// Before 2026-11-28 the search stopped at pe 17, below where the last
/// trace of permanganate can exist, so at equivalence the electron sum
/// only approached its target and no potential was reported. With the
/// water-inert oxidant held above the water ceiling the trace iron(II) and
/// trace permanganate both have a place, the sum crosses its target, and
/// the equivalence point has its own, textbook-defined potential: between
/// the iron couple's (before) and the permanganate couple's (after), and
/// far from both.
#[test]
fn the_potential_jumps_at_the_equivalence_point() {
    let pe = |kmno4: f64| {
        let (_, solution) = titrate_in("H2SO4", kmno4);
        let solution = solution.expect("solution");
        assert!(
            !solution.redox.is_empty(),
            "the oxidation-state split is an answer and must survive"
        );
        solution
            .pe
            .unwrap_or_else(|| panic!("{kmno4} mol MnO4-: the balance pins pe"))
    };
    let (half, before, at, after) = (pe(0.0005), pe(0.0009), pe(0.0010), pe(0.0011));
    assert!(
        half < before && before < at && at < after,
        "{half} {before} {at} {after}"
    );
    assert!(
        at - before > 3.0,
        "the jump into equivalence is steep: {before} -> {at}"
    );
    // Past equivalence the permanganate couple sets pe, which lies above
    // the old ceiling and below its own standard pe (1.507 V, pe 25.5).
    assert!(after > 17.0 && after < 25.5, "{after}");
}

/// A beaker whose coupling stood down says so in the stream.
///
/// It shows its elements in the states they went in as — permanganate still
/// purple, iron still iron(II) — which looks like an ordinary answer. The
/// reason it is not one used to live only in `explain`'s routing, and the
/// reader most likely to be misled is exactly the one who does not know to
/// ask for it. The acid-starved flask is the case that still stands down.
#[test]
fn a_stood_down_coupling_is_announced_where_it_happens() {
    let (events, _) = titrate_in("HCl", 0.0015);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("have not reacted with each other")
        )),
        "the stand-down must appear in the event stream, not only in explain: {events:?}"
    );
}

/// **A pe nothing constrains is withheld, and one a couple fixes is not.**
///
/// `th-100` — water, `NaHCO3`, `HCl`, sealed — used to publish a pe, and
/// which pe depended on the order the two reagents went in:
///
/// | | pH | ionic strength | gauge | **pe** |
/// |---|---|---|---|---|
/// | bicarbonate first | 5.555778 | 0.497017 | agrees to 1e-6 | **12.780243** |
/// | acid first | 5.555338 | 0.497017 | agrees to 1e-6 | **-0.055944** |
///
/// Every other surface agreed and pe moved 12.84, about 760 mV. The beaker
/// holds no couple — `solution.redox` is empty both ways — so the number
/// was whatever the solver's path left behind, published under the
/// vessel's own name with nothing to say it meant nothing. The generated
/// `Order` rule in `corpus_perturbation.rs` found it.
///
/// The guard that was supposed to prevent this asked whether any element
/// COULD carry more than one oxidation state in the dataset. Carbon can,
/// so a fizzing bicarbonate solution satisfied it. It now additionally
/// requires that some element actually IS split — which is the difference
/// between "this dataset knows carbon can be reduced" and "this beaker has
/// an oxidising power".
///
/// The second half of this test is the one that matters: the fix must not
/// withhold a pe that a real couple fixes.
#[test]
fn a_pe_no_couple_constrains_is_not_published() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    let v = VesselId(0);
    add(&mut bench, &mut eq, v, "water", 5.551);
    add(&mut bench, &mut eq, v, "NaHCO3", 0.05);
    add(&mut bench, &mut eq, v, "HCl", 0.05);
    let solution = bench.vessel(v).expect("vessel").solution.clone();
    let fizzy = solution.expect("the carbonate solution is characterised");
    assert!(
        fizzy.redox.is_empty(),
        "no element in a bicarbonate/HCl beaker presents a split: {:?}",
        fizzy.redox
    );
    assert_eq!(
        fizzy.pe, None,
        "pe was published for a beaker with no couple to fix it — it is a \
         path residue, not an oxidising power"
    );

    // The control. Iron(III) in water reports its split, so its pe is a
    // fact about the beaker and must survive.
    let mut bench = Bench::new();
    add(&mut bench, &mut eq, v, "water", 5.551);
    add(&mut bench, &mut eq, v, "FeCl3", 0.01);
    let ferric = bench
        .vessel(v)
        .expect("vessel")
        .solution
        .clone()
        .expect("the iron solution is characterised");
    assert!(
        !ferric.redox.is_empty(),
        "iron(III) has to report its oxidation state"
    );
    assert!(
        ferric.pe.is_some_and(|pe| pe > 10.0),
        "a ferric solution is strongly oxidising and its pe must still be \
         published: {:?}",
        ferric.pe
    );
}

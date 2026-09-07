//! Boron, and the refusal that was never a refusal.
//!
//! Three sentences shipped saying this bench could not do borate. The
//! worst of them, on the borosilicate-glass recipe, said outright that "no
//! shipped database defines a borate that would let it dissolve or react".
//! Every dataset this lab ROUTES defines it:
//!
//! ```text
//! wateq4f.dat   line   16   B         H3BO3     0         10.81   10.81
//! wateq4f.dat   line  335   H3BO3 = H2BO3- + H+          log_k -9.24
//! minteq.v4.dat line   17   B         H3BO3     0         B       10.81
//! minteq.v4.dat line 4912   H3BO3 = H2BO3- + H+          log_k -9.236
//! pitzer.dat               B         B(OH)3    0         B       10.81
//! ```
//!
//! with minteq's polyborates, its Na/Mg/Ca/Sr/Ba/Ag borate pairs and its
//! Hfo sorption rows beside them. Nothing had to be borrowed from an
//! unrouted file, as hypochlorite and lactate were. The gap was three
//! lines in `derived.rs`: no group in `oxyanion_groups` contained boron
//! and `B` was not in `RESIDUE_OK`, so every boron formula came back
//! unmappable and the registry wrote that down as a fact about the world.
//!
//! These are the claims the fix buys, and the one it deliberately does
//! not.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::derived::{self, DerivedRole};
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn run(script: &[&str]) -> (Bench, VesselId) {
    let mut bench = Bench::new();
    let mut stack = stack();
    for op in script {
        bench
            .step_with(
                kerotakis_core::script::parse_op(op)
                    .expect("parse")
                    .expect("a known verb"),
                &mut stack,
                &PermissiveScreen,
            )
            .expect("step");
    }
    (bench, VesselId(0))
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("a borate solution must be characterised")
        .ph
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

/// Boric acid computes its own pH, which is the whole of the claim that
/// used to be denied.
///
/// pKa 9.24 makes this a very weak acid: 0.02 mol in 200 mL is 0.1 mol/L,
/// and √(K_a·C) = √(10^−9.24 × 0.1) = 7.6 × 10⁻⁶, so pH 5.12 on paper. A
/// bench that could not speciate borate would either refuse the vessel a
/// solution or hand back water's own 7.
#[test]
fn boric_acid_computes_its_own_ph() {
    let (bench, v) = run(&["add v1 water 200mL", "add v1 H3BO3 0.02mol"]);
    let p = ph(&bench, v);
    assert!(
        (4.6..5.8).contains(&p),
        "0.1 mol/L boric acid is about pH 5.1 at pKa 9.24, got {p}"
    );
}

/// It is a WEAK acid, and the square-root law is how a test says so.
///
/// A hundredfold dilution moves a strong acid two pH units and a weak one
/// exactly one, because √C carries the concentration. This is the
/// assertion that fails if the couple is ever dropped and the pH starts
/// coming from charge balance alone.
#[test]
fn a_hundredfold_dilution_moves_boric_acid_one_unit_not_two() {
    let (strong, v) = run(&["add v1 water 200mL", "add v1 H3BO3 0.02mol"]);
    let (weak, w) = run(&["add v1 water 200mL", "add v1 H3BO3 0.0002mol"]);
    let shift = ph(&weak, w) - ph(&strong, v);
    assert!(
        (0.6..1.4).contains(&shift),
        "a weak acid diluted a hundredfold moves about one unit, got {shift}"
    );
}

/// The borax buffer, assembled from the parts the bench can hold.
///
/// This is the arithmetic borax itself performs. Na₂B₄O₇ is four borons
/// against two sodiums, so dissolving it half-neutralises the boric acid —
/// the charge balance asks for two moles of anion out of four moles of
/// boron — and half-neutralised is the definition of a buffer sitting AT
/// its pKa. Four millimoles of boric acid and two of hydroxide is the same
/// solution by a different road, and it must read 9.24.
///
/// The salt itself is not used here, and `borax_keeps_its_curated_bound`
/// below is why.
#[test]
fn four_borons_against_two_sodiums_is_the_borax_buffer() {
    let (bench, v) = run(&[
        "add v1 water 200mL",
        "add v1 H3BO3 0.004mol",
        "add v1 NaOH 0.002mol",
    ]);
    let p = ph(&bench, v);
    assert!(
        (8.8..9.7).contains(&p),
        "half-neutralised boric acid sits at pKa 9.24, got {p}"
    );
}

/// And it is a BUFFER, which a pH alone does not show.
///
/// Adding another half-milliequivalent of base to the half-neutralised
/// solution above moves it a fraction of a unit. The same base added to
/// plain water at the same amount takes it past pH 11.
#[test]
fn the_borate_couple_actually_resists() {
    let (buffered, v) = run(&[
        "add v1 water 200mL",
        "add v1 H3BO3 0.004mol",
        "add v1 NaOH 0.002mol",
        "add v1 NaOH 0.0005mol",
    ]);
    let (plain, w) = run(&["add v1 water 200mL", "add v1 NaOH 0.0005mol"]);
    let buffered_ph = ph(&buffered, v);
    let plain_ph = ph(&plain, w);
    assert!(
        buffered_ph < 10.0,
        "the borate couple holds the pH near its pKa, got {buffered_ph}"
    );
    assert!(
        plain_ph > 10.5,
        "the same base in plain water is a strong-base pH, got {plain_ph}"
    );
}

/// The element total divides between BOTH names, not one of them.
///
/// pKa 9.24 sits inside the range a bench works in, so which member the
/// beaker holds is a real question rather than a formality at the
/// extremes. At the buffer point the ledger must carry both.
#[test]
fn the_ledger_carries_both_members_of_the_couple() {
    let (bench, v) = run(&[
        "add v1 water 200mL",
        "add v1 H3BO3 0.004mol",
        "add v1 NaOH 0.002mol",
    ]);
    let acid = moles(&bench, v, "H3BO3");
    let base = moles(&bench, v, "H2BO3-");
    assert!(
        acid > 1e-4 && base > 1e-4,
        "half-neutralised boron is carried as both H3BO3 ({acid}) and H2BO3- ({base})"
    );
    let total = acid + base;
    assert!(
        (total - 0.004).abs() < 5e-4,
        "the boron is conserved: {total} against 0.004 mol in"
    );
    let ratio = base / acid;
    assert!(
        (0.4..2.5).contains(&ratio),
        "at the pKa the two are within a factor of two, got {ratio}"
    );
}

/// What the derivation actually books, and what it refuses.
///
/// The extraction is exact rather than greedy: it takes ALL of a formula's
/// boron together with 1.5 oxygens per boron — the B₂O₃ oxide unit — or
/// none of it, and only when nothing but oxygen, hydrogen and simple
/// cations is left beside it. That is what keeps it from reaching into a
/// sulfate's or a carbonate's oxygen, which is the mistake #530's first
/// `ClO` attempt made on atacamite's hydroxides.
#[test]
fn the_extraction_books_boron_and_nothing_else_moves() {
    match derived::role("H3BO3") {
        Some(DerivedRole::Dissolves(els)) => {
            assert_eq!(els.as_slice(), &[("B".to_string(), 1.0)]);
        }
        other => panic!("boric acid must dissolve as one boron, got {other:?}"),
    }
    match derived::role("H2BO3-") {
        Some(DerivedRole::Dissolves(els)) => {
            assert_eq!(els.as_slice(), &[("B".to_string(), 1.0)]);
        }
        other => panic!("dihydrogenborate must dissolve as one boron, got {other:?}"),
    }
    // Nothing that is not a borate acquired a boron contribution.
    for s in kerotakis_core::species::registry() {
        let Some(DerivedRole::Dissolves(els)) = derived::role(s.key) else {
            continue;
        };
        let has_boron = els.iter().any(|(el, _)| el == "B");
        let is_borate = matches!(s.key, "H3BO3" | "H2BO3-");
        assert_eq!(has_boron, is_borate, "{} books boron: {els:?}", s.key);
    }
    // And the element really is in every routed dataset, which is the
    // fact the three shipped sentences denied.
    for tag in derived::DB_TAGS {
        assert!(
            derived::index_for(tag).has_element("B"),
            "{tag} defines boron"
        );
    }
    assert_eq!(
        derived::booking_ion("B"),
        Some("H3BO3"),
        "dissolved boron books as the databases' own master species"
    );
}

/// Borax is NOT given an aqueous role, and this is the reason written as
/// a test rather than as a comment.
///
/// `DerivedRole::Dissolves` is phase-blind: `partition` enters a portion's
/// element totals whatever phase it is in. Handing this salt one would
/// have entered the crystals sitting on the bottom of the snowflake beaker
/// into solution along with the part that actually dissolved — 25 g into
/// 100 mL is 0.497 mol of boron in 0.1 kg of solvent — and taken the
/// crystals with it, silently, on the same step that `saturation_moves`
/// had just done the honest arithmetic.
///
/// A routed dataset that spelled a sodium borate SOLID would settle it the
/// way `Halite` settles table salt, and none does: minteq.v4 and wateq4f
/// have `Pb(BO2)2`, `Zn(BO2)2`, `Cd(BO2)2` and (minteq only) `Co(BO2)2`
/// between them and nothing else. So the curated bound is the only bound
/// there is, and the salt keeps its own key.
#[test]
fn borax_keeps_its_curated_bound_and_its_own_key() {
    assert!(
        derived::role("Na2B4O7").is_none(),
        "borax must not acquire an aqueous role while its solubility bound is curated"
    );
    let (bench, v) = run(&["add v1 water 100mL", "add v1 Na2B4O7 25g"]);
    let left = moles(&bench, v, "Na2B4O7");
    assert!(
        left > 0.1,
        "most of 25 g of borax is still borax after a solve, got {left} mol"
    );
}

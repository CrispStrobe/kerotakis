//! BRD-020 phase 2 — the ledger a reaction family must balance.
//!
//! The IR shipped contract-first: templates that could be applied, with
//! nothing checking that applying one conserves matter. That check has to
//! exist before products enter the vessel ledger, because the moment they
//! do, an invented atom stops being local to the template.

use kerotakis_org::templates::{
    apply_template, apply_template_any_order, conservation, e2_alkyl_halide, esterification,
    ledger_of, saponification, sn2_alkyl_halide, ReactionTemplate,
};

fn parse(smiles: &str) -> chematic::core::Molecule {
    chematic::smiles::parse(smiles).expect("test SMILES parses")
}

fn ledger_for(smiles: &[&str]) -> kerotakis_org::templates::Ledger {
    let molecules: Vec<_> = smiles.iter().map(|s| parse(s)).collect();
    let refs: Vec<_> = molecules.iter().collect();
    ledger_of(&refs)
}

#[test]
fn every_curated_template_conserves_atoms_and_charge() {
    // The four shipped families, each applied to a substrate it is written
    // for. If any of them dropped a by-product this would fail, which is
    // the point: `apply_template` refuses rather than returning the loss.
    let cases: Vec<(ReactionTemplate, Vec<&str>)> = vec![
        (esterification(), vec!["CC(=O)O", "CCO"]),
        (saponification(), vec!["CCOC(C)=O", "[OH-]"]),
        (sn2_alkyl_halide(), vec!["CCBr", "[OH-]"]),
        // Propyl, not ethyl: the E2 pattern needs two ADJACENT CH2 groups,
        // and in ethyl bromide the carbon next to the CH2Br is a CH3. The
        // substrate has to be right before the template can be judged.
        (e2_alkyl_halide(), vec!["CCCBr", "[OH-]"]),
    ];
    for (template, reactants) in cases {
        let products = apply_template(&template, &reactants)
            .unwrap_or_else(|e| panic!("{} should apply: {e}", template.name));
        assert!(
            !products.is_empty(),
            "{} produced nothing at all",
            template.name
        );
    }
}

#[test]
fn the_ledger_counts_implicit_hydrogens_rather_than_only_heavy_atoms() {
    // A ledger that counted only heavy atoms would balance while every
    // hydrogen quietly vanished — the exact error it exists to catch.
    let ethanol = ledger_for(&["CCO"]);
    assert_eq!(ethanol.atoms.get("C"), Some(&2));
    assert_eq!(ethanol.atoms.get("O"), Some(&1));
    assert_eq!(ethanol.atoms.get("H"), Some(&6));
    assert_eq!(ethanol.charge, 0);
}

#[test]
fn the_ledger_tracks_charge() {
    let hydroxide = ledger_for(&["[OH-]"]);
    assert_eq!(hydroxide.charge, -1);
    let together = ledger_for(&["[OH-]", "[Na+]"]);
    assert_eq!(together.charge, 0);
}

#[test]
fn conservation_names_exactly_what_did_not_balance() {
    let before = ledger_for(&["CC(=O)O", "CCO"]);
    // The ester WITHOUT its water by-product: two hydrogens and an oxygen
    // short, which is what a template that drops a by-product looks like.
    let after = ledger_for(&["CC(=O)OCC"]);
    let imbalance = conservation(&before, &after).expect_err("this cannot balance");
    assert!(
        imbalance.atoms.contains_key("O"),
        "oxygen went missing and the report should say so: {imbalance}"
    );
    assert!(imbalance.atoms.contains_key("H"));
    // The message is actionable: "O: 3 in, 2 out", not "invalid".
    let said = imbalance.to_string();
    assert!(said.contains("in,"), "unhelpful message: {said}");

    // And a balanced pair reports nothing.
    let full = ledger_for(&["CC(=O)OCC", "O"]);
    assert!(conservation(&before, &full).is_ok());
}

#[test]
fn a_template_that_invents_matter_is_refused_by_name() {
    // Deliberately broken: the right-hand side keeps the carbonyl carbon but
    // conjures a second one. A rule like this must never reach a vessel.
    let inventing = ReactionTemplate {
        name: "invents-a-carbon".into(),
        family: "test".into(),
        smirks: "[C:1](=[O:2])[OH:3]>>[C:1](=[O:2])[OH:3].[CH4]".into(),
        source: "deliberately wrong, for the conservation test".into(),
        validated: false,
    };
    let error = apply_template(&inventing, &["CC(=O)O"])
        .expect_err("a template that invents carbon must be refused");
    assert!(
        error.contains("does not conserve"),
        "refusal should say why: {error}"
    );
    assert!(
        error.contains("invents-a-carbon"),
        "refusal should name the rule: {error}"
    );
    assert!(
        error.contains("C:"),
        "refusal should name the element: {error}"
    );
}

#[test]
fn a_family_fires_whichever_way_round_the_bench_poured_them() {
    // SMIRKS matching is positional; a bench is not. The same two molecules
    // handed over in the other order must give the same answer, and must do
    // so deterministically.
    let acid_first = apply_template_any_order(&esterification(), &["CC(=O)O", "CCO"])
        .expect("applies acid-first");
    let alcohol_first = apply_template_any_order(&esterification(), &["CCO", "CC(=O)O"])
        .expect("applies alcohol-first");
    assert_eq!(acid_first, alcohol_first);
    assert!(!acid_first.is_empty());

    // Same inputs, same answer, every time — not "whichever permutation the
    // iterator happened to reach first".
    for _ in 0..5 {
        assert_eq!(
            apply_template_any_order(&esterification(), &["CCO", "CC(=O)O"]).unwrap(),
            alcohol_first
        );
    }
}

#[test]
fn an_out_of_domain_substrate_declines_rather_than_overgeneralising() {
    // Methane has no carboxylic acid and no alcohol. An esterification rule
    // asked about it must say no, in any order, rather than inventing a
    // plausible-looking product.
    let refused = apply_template_any_order(&esterification(), &["C", "C"]);
    assert!(
        refused.is_err() || refused.as_ref().is_ok_and(|p| p.is_empty()),
        "esterification claimed to act on methane: {refused:?}"
    );

    // And an alkane is not a haloalkane.
    let no_halide = apply_template_any_order(&sn2_alkyl_halide(), &["CC", "[OH-]"]);
    assert!(
        no_halide.is_err() || no_halide.as_ref().is_ok_and(|p| p.is_empty()),
        "SN2 claimed to act on ethane: {no_halide:?}"
    );
}

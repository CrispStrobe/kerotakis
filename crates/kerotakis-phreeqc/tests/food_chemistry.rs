//! BRD-012.S03 — the food-chemistry identity tranche.
//!
//! Glucose, fructose, malic acid, citric acid and cellulose: the pure
//! species that the household-material recipes name and could not resolve.
//! A registry identity is not a capability, so what is pinned here is what
//! the engine can honestly *say* about each one — and, for one of them,
//! what it honestly refuses to say.
//!
//! The dividing line this file exists to hold is between the two acids.
//! Citric acid computes its pH, because minteq.v4 defines a Citrate master
//! species with all three protonation constants. Malic acid does not,
//! because malate is in none of the databases vendored with iphreeqc —
//! and the failure mode that matters is not the missing feature, it is a
//! beaker of acid quietly reporting a neutral pH. So the refusal is an
//! event on the stream, not a comment in a source file.
//!
//! The windows below are set from the chemistry — a solubility capacity
//! from its own arithmetic, an acid's pH from its first dissociation
//! constant — and are wide enough to be about that rather than about one
//! database revision. Every number was read off the engine before it was
//! pinned. At the time of writing: citric acid pH 1.717 on minteq.v4
//! against acetic acid's 2.530 at the same 0.5 mol/kgw; 0.5060 of 1.0 mol
//! of glucose dissolved with 0.4940 left solid and the two summing back
//! to 1.000000; and salt water at pH 6.985 unmoved to three decimals by
//! dissolving a sugar in it.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(kerotakis_core::hmix::MixingEnthalpyEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn run(adds: &[(&str, f64)]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut solvers = stack();
    let mut events = Vec::new();
    for (key, moles) in adds {
        events.extend(
            bench
                .step_with(
                    Operator::Add {
                        vessel: VesselId(0),
                        species: SpeciesId::new(key),
                        moles: Moles(*moles),
                        at: None,
                    },
                    &mut solvers,
                    &ReactiveGroupScreen,
                )
                .unwrap_or_else(|e| panic!("ADD {key}: {e}")),
        );
    }
    (bench, events)
}

fn solution(bench: &Bench) -> SolutionInfo {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .solution
        .clone()
        .expect("the aqueous engine characterised the solution")
}

/// Moles of `key` held in `phase`.
fn phase_moles(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn not_yet_modeled(events: &[Event]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled { what, .. } => Some(what.as_str()),
            _ => None,
        })
        .collect()
}

/// 5.55 mol of water is 0.1 kg — the school beaker these tests work in.
const WATER: (&str, f64) = ("water", 5.55);

// ── the neutral-solute rung ──────────────────────────────────────────

#[test]
fn glucose_dissolves_up_to_its_limit_and_the_excess_stays_solid() {
    // 90.9 g per 100 mL of water is 0.0909 kg of glucose in this beaker,
    // which at 180.156 g/mol is 0.5046 mol of capacity. Ask for twice
    // that and half of it has to still be sitting on the bottom.
    let asked = 1.0;
    let (bench, _) = run(&[WATER, ("glucose", asked)]);

    let dissolved = phase_moles(&bench, "glucose", Phase::Aqueous);
    let solid = phase_moles(&bench, "glucose", Phase::Solid);

    assert!(
        (dissolved - 0.5046).abs() < 0.02,
        "glucose should dissolve to its 90.9 g/100 mL capacity (~0.505 mol here), got {dissolved}"
    );
    assert!(
        solid > 0.4,
        "the excess glucose must stay solid, got {solid} mol undissolved"
    );
    // Nothing may go missing between the two phases: this is the whole
    // point of a capacity rung rather than a quiet subtraction.
    assert!(
        (dissolved + solid - asked).abs() < 1e-9,
        "glucose was not conserved: {dissolved} aqueous + {solid} solid != {asked}"
    );
}

#[test]
fn a_little_glucose_dissolves_completely() {
    // A test that only ever fires at the limit is not a test of the limit.
    let (bench, _) = run(&[WATER, ("glucose", 0.05)]);
    assert!(
        phase_moles(&bench, "glucose", Phase::Solid) < 1e-9,
        "0.05 mol is far under capacity and should all dissolve"
    );
    assert!((phase_moles(&bench, "glucose", Phase::Aqueous) - 0.05).abs() < 1e-9);
    // Sugar and water alone give the aqueous engine nothing to speciate,
    // so there is deliberately no pH to assert here: a solution of a
    // neutral solute is characterised by the rung, not by PHREEQC. The
    // pH claim belongs in the test below, which puts an electrolyte in
    // the glass so that a pH exists to be wrong about.
}

#[test]
fn glucose_is_not_three_acetates() {
    // The guard in `derived::contribution_from_counts`, exercised through
    // the whole engine. C6H12O6 is exactly three times C2H3O2 plus three
    // protons, so before that guard the sugar decomposed into acetate by
    // arithmetic — and then acidified whatever it was dissolved in.
    //
    // A pinch of salt is what makes the failure visible: it gives the
    // solver a solution to characterise, so the sugar's (wrongly) derived
    // acetate would show up both as a species and as a moved pH.
    let (salty, _) = run(&[WATER, ("NaCl", 0.01)]);
    let (sweet, _) = run(&[WATER, ("NaCl", 0.01), ("glucose", 0.05)]);
    let (before, after) = (solution(&salty), solution(&sweet));

    let acetate: Vec<&str> = after
        .species
        .iter()
        .map(|s| s.name.as_str())
        .filter(|n| n.contains("Acetate") || n.contains("CH3COO"))
        .collect();
    assert!(
        acetate.is_empty(),
        "dissolving a sugar must not put acetate in the glass: {acetate:?}"
    );
    assert!(
        (after.ph - before.ph).abs() < 0.2,
        "adding a sugar must not move the pH: {} -> {}",
        before.ph,
        after.ph
    );
    assert!(
        after.ph > 6.0,
        "salt water with sugar in it is not acidic, got pH {}",
        after.ph
    );
}

// ── the two isomers stay two ─────────────────────────────────────────

#[test]
fn glucose_and_fructose_are_distinct_end_to_end() {
    use kerotakis_core::species;

    let glucose = species::lookup_key("glucose").expect("glucose in the registry");
    let fructose = species::lookup_key("fructose").expect("fructose in the registry");

    // Same formula, same molar mass — they are isomers, and a pipeline
    // that keyed on either would silently merge them.
    assert_eq!(glucose.formula, fructose.formula);
    assert_eq!(glucose.formula, "C6H12O6");
    assert!((glucose.molar_mass - fructose.molar_mass).abs() < 1e-9);

    // And yet they are two substances. The InChIKey is where that has to
    // survive, because it is the identity the rest of the world joins on.
    assert_ne!(
        glucose.inchikey, fructose.inchikey,
        "the two sugars must not collapse into one identity"
    );
    assert!(!glucose.inchikey.is_empty() && !fructose.inchikey.is_empty());

    // Both identities carry the D configuration but deliberately leave the
    // anomeric centre unspecified. Pin the official-library results so a
    // future identity-path change cannot silently broaden them to D/L-neutral
    // skeletons or narrow them to a particular alpha/beta anomer.
    assert_eq!(glucose.inchikey, "WQZGKKKJIJFFOK-GASJEMHNSA-N");
    assert_eq!(fructose.inchikey, "LKDRXBCSQODPBY-VRPWFDPXSA-N");

    // Independently, glucose and fructose are structural isomers. Their
    // connectivity prefixes must differ regardless of stereochemical layer.
    assert_ne!(
        glucose.inchikey.split('-').next(),
        fructose.inchikey.split('-').next(),
        "the distinctness must come from the skeleton, not from a stereo layer"
    );

    // They differ in the bench-visible way too: fructose is far more
    // soluble than glucose, which is why fructose syrups do not crystallise.
    let g = glucose
        .aqueous_solubility_g_per_100_ml
        .expect("glucose limit");
    let f = fructose
        .aqueous_solubility_g_per_100_ml
        .expect("fructose limit");
    assert!(
        f > g * 2.0,
        "fructose ({f} g/100 mL) should be much more soluble than glucose ({g})"
    );

    // End to end: both in one beaker, both still there, neither renamed.
    let (bench, _) = run(&[WATER, ("glucose", 0.05), ("fructose", 0.05)]);
    assert!((phase_moles(&bench, "glucose", Phase::Aqueous) - 0.05).abs() < 1e-9);
    assert!((phase_moles(&bench, "fructose", Phase::Aqueous) - 0.05).abs() < 1e-9);
}

// ── the acids: one computed, one refused ─────────────────────────────

#[test]
fn citric_acid_computes_a_genuinely_acidic_ph() {
    // 0.05 mol in 0.1 kgw is 0.5 mol/kgw of a triprotic acid whose first
    // pKa is 3.13. minteq.v4 carries Citrate with all three protonation
    // constants, so this number comes out of the database rather than out
    // of a curated table. The window is set wide around the first
    // proton's own arithmetic — sqrt(Ka1 * C) puts it near pH 1.7 — so
    // that it is about the chemistry and not about one database revision.
    // Computed: pH 1.717, and the database really does resolve the whole
    // protonation ladder rather than a single lumped acid —
    // H3(Citrate) 4.78e-1, H2(Citrate)- 2.21e-2, H(Citrate)-2 3.38e-5,
    // Citrate-3 1.29e-9.
    let (bench, _) = run(&[WATER, ("citric_acid", 0.05)]);
    let info = solution(&bench);
    assert!(
        info.ph > 1.0 && info.ph < 3.0,
        "citric acid should give a clearly acidic solution, got pH {}",
        info.ph
    );

    // It must have been routed to the database that can actually speak
    // about it — the routing is what makes the number honest.
    let dataset = info
        .provenance
        .as_ref()
        .map(|p| p.dataset.clone())
        .unwrap_or_default();
    assert!(
        dataset.contains("minteq"),
        "citrate chemistry lives only in minteq.v4, routed to {dataset:?}"
    );

    // And the citrate is really in the speciation, not just implied by a
    // proton balance.
    let citrate: Vec<&str> = info
        .species
        .iter()
        .map(|s| s.name.as_str())
        .filter(|n| n.contains("itrate"))
        .collect();
    assert!(
        !citrate.is_empty(),
        "the speciation should name citrate species: {:?}",
        info.species.iter().map(|s| &s.name).collect::<Vec<_>>()
    );
}

#[test]
fn citric_acid_is_more_acidic_than_the_same_amount_of_acetic_acid() {
    // A sanity check on the direction, not just the magnitude: citric
    // acid's first proton (pKa 3.13) is a good deal stronger than acetic
    // acid's (pKa 4.76), and both routes go through the same database.
    let (citric, _) = run(&[WATER, ("citric_acid", 0.05)]);
    let (acetic, _) = run(&[WATER, ("CH3COOH", 0.05)]);
    let (cp, ap) = (solution(&citric).ph, solution(&acetic).ph);
    assert!(
        cp < ap,
        "citric acid (pH {cp}) should be more acidic than acetic acid (pH {ap})"
    );
}

#[test]
fn malic_acid_dissolves_but_says_its_acidity_is_not_modelled() {
    // The failure mode this test exists for: a carboxylic acid that
    // dissolves and leaves the solution reading pH 7.00 with no comment.
    // Malate is in none of the vendored PHREEQC databases, so this lab
    // cannot compute the acid's protons — and must say so rather than
    // publish the neutral number as if it meant something.
    let (bench, events) = run(&[WATER, ("malic_acid", 0.05)]);

    // It does dissolve: that part is real chemistry and is not withheld.
    assert!(
        phase_moles(&bench, "malic_acid", Phase::Aqueous) > 0.049,
        "0.05 mol of malic acid is well under its 55.8 g/100 mL limit"
    );

    // And the boundary is spoken, on the event stream, naming the gap.
    let spoken = not_yet_modeled(&events);
    assert!(
        spoken
            .iter()
            .any(|w| w.contains("acidity is not modelled") && w.contains("malate")),
        "the unmodelled acidity must be said out loud, got: {spoken:?}"
    );
}

#[test]
fn the_two_acids_are_not_treated_alike() {
    // The whole point of the tranche: same functional group, same shelf,
    // and the engine is honest that it can do one and not the other.
    let (_, citric_events) = run(&[WATER, ("citric_acid", 0.05)]);
    let (_, malic_events) = run(&[WATER, ("malic_acid", 0.05)]);

    let citric_refusals: Vec<&str> = not_yet_modeled(&citric_events)
        .into_iter()
        .filter(|w| w.contains("acidity is not modelled"))
        .collect();
    assert!(
        citric_refusals.is_empty(),
        "citric acid's acidity IS modelled and must not carry the refusal: {citric_refusals:?}"
    );
    assert!(
        not_yet_modeled(&malic_events)
            .iter()
            .any(|w| w.contains("acidity is not modelled")),
        "malic acid's acidity is not modelled and must carry the refusal"
    );
}

// ── the polymer ──────────────────────────────────────────────────────

#[test]
fn cellulose_enters_as_starch_does_and_does_not_dissolve() {
    use kerotakis_core::species;

    let cellulose = species::lookup_key("cellulose").expect("cellulose in the registry");
    let starch = species::lookup_key("starch").expect("starch in the registry");

    // The precedent, followed exactly: a polymer enters as its repeat
    // unit, not as a molecule. Both are the anhydroglucose unit.
    assert_eq!(cellulose.formula, "C6H10O5");
    assert_eq!(cellulose.formula, starch.formula);
    assert!(
        (cellulose.molar_mass - starch.molar_mass).abs() < 0.01,
        "both are the anhydroglucose unit: {} vs {}",
        cellulose.molar_mass,
        starch.molar_mass
    );

    // And no InChIKey is asserted, for the same reason starch asserts
    // none: (C6H10O5)n is not a molecule, and a monomer's key would be a
    // claim about the polymer that is not true of it.
    assert!(
        cellulose.inchikey.is_empty(),
        "cellulose must not claim a molecular identity, got {:?}",
        cellulose.inchikey
    );
    assert!(starch.inchikey.is_empty());

    // Cellulose does not dissolve in water, and nothing here pretends it
    // does — no capacity, and the solid stays solid.
    assert!(cellulose.aqueous_solubility_g_per_100_ml.is_none());
    let (bench, _) = run(&[WATER, ("cellulose", 0.05)]);
    assert!(
        (phase_moles(&bench, "cellulose", Phase::Solid) - 0.05).abs() < 1e-9,
        "cellulose must stay solid in water"
    );
}

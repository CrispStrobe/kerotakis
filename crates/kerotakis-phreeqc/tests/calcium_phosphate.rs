#![cfg(feature = "engine")]
//! Milk holds a calcium phosphate, and says which one it is not holding.
//!
//! Since the serum minerals landed, every vessel of milk carried an
//! apology: it was supersaturated against Hydroxylapatite (SI +8.3),
//! Ca3(PO4)2(beta) (SI +1.7) and Ca4H(PO4)3:3H2O (SI +1.3), all three of
//! them in minteq.v4.dat and none of them on this shelf, "so nothing can
//! precipitate out of it here". That was a gap in the registry rather
//! than a fact about milk, and it is closed: the three phases are now
//! registry solids, so the aqueous tail can pose them and the honesty
//! pass has nothing left to say about the registry.
//!
//! What it does still say — and what this file pins as deliberate rather
//! than as a leftover — is the OTHER note, the one about rates. Fresh
//! milk serum really is about eight log units supersaturated against
//! hydroxylapatite and really does not precipitate it: the casein micelle
//! holds its calcium phosphate as stabilised nanoclusters, and casein is
//! not modelled here. Two of the three phases therefore carry a kinetic
//! floor in the registry, exactly as tenorite does, and the tail reports
//! that it is withholding them. Letting apatite equilibrate instead takes
//! this milk to pH 5.73 and strips 58 % of the calcium out of the serum,
//! which is a beaker with no milk protein in it.
//!
//! The one left reachable is octacalcium phosphate, which is what a
//! near-neutral calcium phosphate solution actually gives at bench
//! temperature and is the family milk's own colloidal calcium phosphate
//! belongs to. About 3.4 mg of it comes out of 100 mL.

use kerotakis_core::render::{render_events, render_vessel, Register};
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// The registry-gap apology, by the two phrases that are its own and not
/// the kinetic note's. Matching on "supersaturated against" alone would
/// catch the curated withholding note too, and that note is supposed to
/// be there.
const REGISTRY_GAP: [&str; 2] = [
    "not in this lab's registry",
    "nothing can precipitate out of it here",
];

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

/// 100 mL of milk (103 g), poured and equilibrated, with the lines a
/// person would see: `inspect v1` and everything the step said.
fn milk() -> (Vessel, Vec<String>) {
    let recipe = kerotakis_core::material::lookup("whole_milk", None).expect("the milk recipe");
    let mut bench = Bench::new();
    let mut solvers = stack();
    let events = bench
        .step_with(
            Operator::AddMaterial {
                vessel: VesselId(0),
                material: recipe.canonical_key.clone(),
                recipe_id: recipe.id.clone(),
                recipe_version: recipe.version,
                total_amount: 103.0,
                basis: recipe.basis,
                sample_seed: 0,
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("add milk");
    let vessel = bench.vessel(VesselId(0)).expect("the beaker").clone();
    let mut lines = render_events(&events, Register::LV2);
    lines.extend(render_vessel(&vessel, Register::LV2));
    (vessel, lines)
}

fn solid(vessel: &Vessel, key: &str) -> f64 {
    vessel.moles_of(&SpeciesId::new(key)).0
}

/// The three phases the apology used to name are registry solids, and each
/// one pairs with the minteq.v4 phase it was named after. This is the
/// mapping the whole file rests on and nothing in `derived` was edited to
/// get it — the composition matcher found them.
#[test]
fn the_three_named_phases_reached_the_candidate_list() {
    use kerotakis_phreeqc::derived;
    for (phase, species) in [
        ("Hydroxylapatite", "hydroxylapatite"),
        ("Ca3(PO4)2(beta)", "Ca3(PO4)2"),
        ("Ca4H(PO4)3:3H2O", "octacalcium_phosphate"),
    ] {
        let found = derived::phase_by_name(phase)
            .unwrap_or_else(|| panic!("{phase} must resolve to a registry solid"));
        assert_eq!(
            found.species, species,
            "{phase} must book as {species}, not {}",
            found.species
        );
    }
    // wateq4f spells apatite differently and writes its dissolution with
    // four protons rather than one. Both spellings must stay resolvable,
    // because a readback from either database has to be nameable.
    assert_eq!(
        kerotakis_phreeqc::derived::phase_by_name("Hydroxyapatite").map(|p| p.species),
        Some("hydroxylapatite"),
        "wateq4f's spelling must resolve too"
    );
}

/// The apology is gone, and what replaced it is a solid in the glass.
#[test]
fn milk_precipitates_a_calcium_phosphate_instead_of_apologising() {
    let (vessel, lines) = milk();
    let text = lines.join("\n");

    for phrase in REGISTRY_GAP {
        assert!(
            !text.contains(phrase),
            "milk still reports a registry gap ({phrase:?}). Full output:\n{text}"
        );
    }

    let ocp = solid(&vessel, "octacalcium_phosphate");
    let mass_mg = ocp * 500.27 * 1000.0;
    assert!(
        ocp > 1e-7,
        "100 mL of milk must lay down a calcium phosphate; it holds {ocp:.4e} mol \
         ({mass_mg:.3} mg) of octacalcium phosphate. Full output:\n{text}"
    );
    // A few milligrams, not a curd. The bound is loose on purpose: what is
    // being pinned is that a real but small colloid comes out, which is
    // what a recipe carrying no casein can honestly produce.
    assert!(
        (0.5..=20.0).contains(&mass_mg),
        "the colloid should be a few milligrams per 100 mL, got {mass_mg:.3} mg"
    );
    assert!(
        text.to_lowercase().contains("octacalcium phosphate"),
        "and `inspect` has to show it. Full output:\n{text}"
    );
}

/// The two withheld phases stay out of the glass, and the tail says so.
///
/// This is the note that is NOT an apology: hydroxylapatite is the stable
/// solid, milk is hugely supersaturated against it, and it does not form.
/// Asserting the note positively is the point — a silent withholding would
/// be the dishonest version of the same behaviour.
#[test]
fn apatite_is_withheld_out_loud_rather_than_precipitated() {
    let (vessel, lines) = milk();
    let text = lines.join("\n");

    assert_eq!(
        solid(&vessel, "hydroxylapatite"),
        0.0,
        "apatite must not form in milk at bench temperature"
    );
    assert_eq!(
        solid(&vessel, "Ca3(PO4)2"),
        0.0,
        "beta-tricalcium phosphate is a furnace product, not a precipitate"
    );
    assert!(
        text.contains("Hydroxylapatite") && text.contains("deliberately holding back"),
        "milk must say that it is holding apatite back, and why. Full output:\n{text}"
    );
    assert!(
        text.contains("claim about rates"),
        "and it must say that the claim is about rates. Full output:\n{text}"
    );
}

/// The pH did not move, which is the whole reason the recipe's calcium
/// share moved with it.
///
/// `milk_buffer.rs` pins fresh milk to the 6.4–7.0 window and this file
/// must not quietly widen it, so the same window is asserted again from
/// the other side of the change. Precipitation releases protons, so an
/// unadjusted recipe would have landed at 6.42 with 0.02 of margin; the
/// calcium the solver now lays down as octacalcium phosphate is the
/// calcium the recipe books extra, and the two cancel to three decimals.
#[test]
fn fresh_milk_still_reads_the_ph_of_fresh_milk() {
    let (vessel, _) = milk();
    let info = vessel
        .solution
        .clone()
        .expect("milk characterises a solution");
    assert!(
        (6.4..=7.0).contains(&info.ph),
        "fresh milk is pH 6.6 to 6.8 and equilibrating its colloid must not move it out of \
         the window milk_buffer.rs pins; the tail computed pH {} at ionic strength {} mol/kgw \
         with {:.4e} mol of octacalcium phosphate in the glass",
        info.ph,
        info.ionic_strength,
        solid(&vessel, "octacalcium_phosphate")
    );
    // Holt's diffusate is 0.073 mol/kgw. Taking a little calcium and
    // phosphate out of the serum must not move that either.
    assert!(
        (0.02..=0.2).contains(&info.ionic_strength),
        "milk serum's ionic strength should stay near 0.07 mol/kgw, got {}",
        info.ionic_strength
    );
}

/// The colloid is a buffer as well as a solid: acid redissolves it.
///
/// This is the behaviour the registry addition actually buys, beyond
/// silencing a note. Before it, the same dose of acid had only the serum's
/// phosphate and citrate to spend itself on; now some of it goes into
/// pulling the calcium phosphate back into solution, which is what an acid
/// dose does to real milk and is why a yoghurt is less acid than a bare
/// buffer calculation says.
#[test]
fn acid_dissolves_the_colloid_before_it_takes_the_ph_down() {
    let recipe = kerotakis_core::material::lookup("whole_milk", None).expect("the milk recipe");
    let mut bench = Bench::new();
    let mut solvers = stack();
    bench
        .step_with(
            Operator::AddMaterial {
                vessel: VesselId(0),
                material: recipe.canonical_key.clone(),
                recipe_id: recipe.id.clone(),
                recipe_version: recipe.version,
                total_amount: 103.0,
                basis: recipe.basis,
                sample_seed: 0,
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("add milk");
    let before = solid(
        bench.vessel(VesselId(0)).expect("the beaker"),
        "octacalcium_phosphate",
    );

    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("HCl"),
                moles: Moles(0.000_5),
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("acidify the milk");
    let vessel = bench.vessel(VesselId(0)).expect("the beaker");
    let after = solid(vessel, "octacalcium_phosphate");
    let ph = vessel
        .solution
        .clone()
        .expect("acidified milk is still a solution")
        .ph;

    assert!(
        after < before,
        "0.5 mmol of acid must dissolve part of the colloid: {before:.4e} mol before, \
         {after:.4e} mol after, pH {ph}"
    );
    // The same dose took the colloid-free recipe to pH 5.67. Having a
    // solid to redissolve is worth several tenths, and that is the claim.
    assert!(
        ph > 5.8,
        "0.5 mmol of HCl into 100 mL of milk should stay well above pH 5.8 now that the \
         colloid can redissolve; got {ph} with {after:.4e} mol of colloid left"
    );
}

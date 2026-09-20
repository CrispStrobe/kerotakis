#![cfg(feature = "engine")]
//! Milk has a pH, and the pH is the mineral buffer's.
//!
//! Before the serum minerals were resolved, `whole_milk` was
//! `[water 0.87]` with everything else conserved as unresolved solids. A
//! vessel of it held one species, that species was the solvent, and the
//! aqueous tail characterised no solution at all — so `measure ph` on a
//! beaker of milk read nothing, and the two yoghurt rows in the curiosity
//! corpus were `missing` for that reason rather than for want of a
//! fermentation. The recipe now books milk's diffusible phase: potassium,
//! sodium, the soluble share of the calcium, chloride, inorganic phosphate
//! and citrate. This file is what holds that to the number.
//!
//! What is pinned here is a WINDOW, not a digit. Fresh cow's milk is
//! pH 6.6 to 6.8 and the recipe is built to land inside it; the window is
//! about the chemistry rather than about one database revision, and the
//! failure message prints what the tail actually computed so that a
//! reviewer who moves a mineral can see where it went.
//!
//! HALF THE REST ARRIVED ON 2026-09-18 and half did not. The colloidal
//! calcium phosphate is now booked into the recipe as a solid — 189 mg per
//! 100 mL, entered from the colloidal inorganic phosphorus FDC's total
//! leaves — and it dissolves as acid arrives, which is what milk's colloid
//! really does between pH 6.6 and about 4.9. Casein's own buffering is
//! still modelled by nothing, and Kim et al. 2018 quoting Salaün et al.
//! 2005 rank it at about 35% of milk's buffer capacity.
//!
//! So an acidified milk is no longer a plain lower bound, and it is not a
//! prediction either. It reads 4.60 at the acid a cited yoghurt carries,
//! where that yoghurt measured 4.6 and where this recipe read 3.94 before
//! — and that agreement is a coincidence of where the modelled colloid
//! runs out rather than a validated buffer.
//! `crates/kerotakis-phreeqc/tests/lactate_speciation.rs` says so at
//! length and asserts the residual by showing the beaker fall away past
//! the colloid. The acidification test below still asserts an ordering
//! against water rather than a value, for the same reason it always did.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

/// `grams` of the whole-milk recipe in a beaker, equilibrated.
fn milk(grams: f64) -> Vessel {
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
                total_amount: grams,
                basis: recipe.basis,
                sample_seed: 0,
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("add milk");
    bench.vessel(VesselId(0)).expect("the beaker").clone()
}

fn solution(vessel: &Vessel) -> SolutionInfo {
    vessel
        .solution
        .clone()
        .expect("milk's mineral buffer must characterise a solution")
}

/// 100 mL of fresh milk, and the number a pH meter dipped into it reads.
#[test]
fn fresh_milk_reads_the_ph_of_fresh_milk() {
    let vessel = milk(103.0);
    let info = solution(&vessel);
    assert!(
        (6.4..=7.0).contains(&info.ph),
        "fresh milk is pH 6.6 to 6.8 and this recipe is built to land in it; \
         the tail computed pH {} at ionic strength {} mol/kgw",
        info.ph,
        info.ionic_strength
    );
    // Milk's serum is a real ionic strength, not a trace. Holt's diffusate
    // is quoted at 0.073 mol/kgw; anything an order of magnitude away means
    // a mineral did not reach the solver.
    assert!(
        (0.02..=0.2).contains(&info.ionic_strength),
        "milk serum's ionic strength should be near 0.07 mol/kgw, got {}",
        info.ionic_strength
    );
}

/// The buffer has to reach the database that can speciate it. Citrate and
/// free phosphoric acid live only in minteq.v4 among the three files this
/// lab loads, and the router picks by the elements the problem produced —
/// so this also proves the citrate and phosphate got there at all.
#[test]
fn the_milk_buffer_routes_to_the_dataset_that_carries_citrate_and_phosphate() {
    let vessel = milk(103.0);
    let info = solution(&vessel);
    let dataset = info
        .provenance
        .as_ref()
        .map(|p| p.dataset.clone())
        .unwrap_or_default();
    assert!(
        dataset.contains("minteq"),
        "citrate and free phosphate live only in minteq.v4; routed to {dataset:?}"
    );
}

/// A buffer is a thing that resists, so the test of one is a comparison
/// and not a value. The same dose of strong acid into milk and into the
/// water milk is mostly made of: the milk must end up markedly less acid.
///
/// The dose is deliberately small — 0.5 mmol into 100 mL, well inside what
/// the phosphate and citrate can absorb — because the claim being made is
/// that the buffer exists, not that it is milk's whole buffer. It is not:
/// casein is missing, and the recipe says so.
#[test]
fn the_mineral_buffer_actually_buffers() {
    let acid = 0.000_5;

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
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("HCl"),
                moles: Moles(acid),
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("acidify the milk");
    let acidified_milk = solution(bench.vessel(VesselId(0)).expect("the beaker")).ph;

    // The same acid into the same mass of plain water.
    let mut plain = Bench::new();
    let mut plain_solvers = stack();
    for (key, moles) in [("water", 89.61 / 18.015), ("HCl", acid)] {
        plain
            .step_with(
                Operator::Add {
                    vessel: VesselId(0),
                    species: SpeciesId::new(key),
                    moles: Moles(moles),
                    at: None,
                },
                &mut plain_solvers,
                &PermissiveScreen,
            )
            .expect("acidify the water");
    }
    let acidified_water = plain
        .vessel(VesselId(0))
        .expect("the beaker")
        .solution
        .clone()
        .expect("acid in water is a solution")
        .ph;

    assert!(
        acidified_milk > acidified_water + 1.0,
        "0.5 mmol of HCl takes water to pH {acidified_water} and must leave buffered \
         milk far above it; milk went to pH {acidified_milk}"
    );
    assert!(
        acidified_milk > 5.5,
        "0.5 mmol of HCl is well inside what milk's phosphate and citrate absorb; \
         milk went to pH {acidified_milk}"
    );
}

/// The colloid is in the glass, and putting it there moved nothing about
/// fresh milk.
///
/// That is not luck and it is the reason this could be done at all. The
/// serum this recipe books is already at octacalcium phosphate's
/// saturation — it is why the solver was already laying about 37 mg of the
/// stuff down — so adding more of the same SOLID cannot move a dissolved
/// amount. A charge-neutral mineral at its own saturation disturbs
/// nothing until an acid comes for it, which is exactly the behaviour
/// milk's colloidal calcium phosphate has.
///
/// Measured when this landed: pH 6.5636 and ionic strength 0.07412 with
/// the colloid and without it, and the serum the solver hands back
/// unchanged at Ca 8.1 and inorganic phosphate 10.1 mmol per kg of water.
#[test]
fn the_colloidal_calcium_phosphate_is_in_the_glass_and_fresh_milk_did_not_move() {
    let vessel = milk(103.0);
    let colloid = vessel.moles_of(&SpeciesId::new("octacalcium_phosphate")).0;
    assert!(
        colloid > 3.0e-4,
        "the recipe books 0.3667 mmol of colloidal calcium phosphate per 100 g \
         and the solver adds its own; 100 mL holds {colloid:.6} mol"
    );
    let info = solution(&vessel);
    assert!(
        (6.4..=7.0).contains(&info.ph),
        "booking the colloid must not move fresh milk's pH, and it did not \
         when this landed: 6.5636 either way. Got {}",
        info.ph
    );
}

/// The residual is NAMED ON THE WIRE, which is the owner's condition on
/// shipping half a buffer (2026-09-18).
///
/// A half-corrected buffer that reads as a fully-corrected one is worse
/// than the uncorrected one, because nobody re-checks a number that looks
/// right — and this one looks right: with the acid a cited yoghurt carries
/// the beaker reads 4.60 against that yoghurt's measured 4.6. So the
/// vessel has to say, every time it is acidified, that casein's own
/// buffering is not modelled and roughly what that is worth.
///
/// Asserted in BOTH languages, because "named on the wire" means named to
/// the reader and not to the English-speaking reader. Nothing in the
/// sentence is welded: it is a `Phrase` with a key, and German comes from
/// `crates/kerotakis-core/i18n/de.toml` with no code of its own.
#[test]
fn the_casein_residual_is_named_on_the_wire_in_both_languages() {
    use kerotakis_core::i18n::Locale;
    use kerotakis_core::render::{render_events_in, Register};

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
    let events = bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("lactic_acid"),
                moles: Moles(0.0035),
                at: None,
            },
            &mut solvers,
            &PermissiveScreen,
        )
        .expect("acidify the milk");

    let english = render_events_in(&events, Register::LV2, Locale::EN).join("\n");
    for claim in [
        "casein's own buffering is not here at all",
        "about a third of milk's buffer capacity",
        "colloidal calcium phosphate is here",
    ] {
        assert!(
            english.contains(claim),
            "an acidified milk must say {claim:?} on the wire. Full output:\n{english}"
        );
    }

    let german = render_events_in(&events, Register::LV2, Locale::parse("de")).join("\n");
    for claim in [
        "die Pufferwirkung des Caseins fehlt dagegen vollständig",
        "etwa ein Drittel der Pufferkapazität von Milch",
        "Ihr kolloidales Calciumphosphat ist vorhanden",
    ] {
        assert!(
            german.contains(claim),
            "and it must say it in German too: {claim:?} is missing. Full output:\n{german}"
        );
    }
    // The pH inside that sentence is a NUMBER slot, so German gets its own
    // decimal separator rather than an English one in the middle of a
    // German paragraph.
    assert!(
        german.contains("pH 4,"),
        "the pH in the German sentence must use a German decimal comma. \
         Full output:\n{german}"
    );
}

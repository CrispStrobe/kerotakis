//! The particle drawing's captions, and the tail that outlived I18N-10.
//!
//! `Census::render` took a `Register` and **no `Locale`** until 2026-09-18,
//! so a German session drew its particles under English captions. The
//! `", and {n} more"` tail was the last piece of reader-facing English the
//! I18N-10 sweep left behind, and it is instructive that it survived: it
//! could not be translated on its own. It is glued onto a caption built in
//! the same function, so rendering *und 3 weitere* inside "also in there,
//! too few to draw" would have read worse than leaving it English. Six
//! captions had to move together, with the species list as a SLOT rather
//! than a fragment — the same reason `appearance.rs` could not be fixed by
//! translating `"there is"` and `" and "` separately (#626).
//!
//! **This test lives in the phreeqc crate, and that is the finding.** The
//! first draft sat in `kerotakis-core` and could not reach the tail at all:
//! a `Bench` with no equilibrator never speciates, so eight trace salts
//! stay solid, never enter the census, and nothing is elided. The elision
//! only happens on the speciated path, where the ions appear and all but
//! four fall below one glyph. A core-only test would have asserted the
//! captions of a picture the product never draws.

#![cfg(feature = "engine")]

use kerotakis_core::particles::census;
use kerotakis_core::render::Register;
use kerotakis_core::{Bench, Locale, Moles, Operator, PermissiveScreen, SpeciesId, VesselId};
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// Water and eight trace salts: enough distinct ions that the census draws
/// four and counts the rest, which is the only way to reach the tail.
fn crowded(bench: &mut Bench, eq: &mut PhreeqcEquilibrator) {
    let v = VesselId(0);
    // Not `let _ =`. A swallowed add is a fixture that quietly stops
    // testing what it says it tests: the first draft named MgCl2, KBr and
    // NaF, none of which the registry carries, and discarding the errors
    // hid that until the elision guard below fired.
    let mut add = |bench: &mut Bench, key: &str, moles: f64| {
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
            .unwrap_or_else(|e| panic!("the fixture cannot add {key}: {e:?}"));
    };
    add(bench, "water", 5.55);
    for key in [
        "NaCl", "KCl", "CaCl2", "NaNO3", "Na2SO4", "KI", "NaBr", "NH4Cl",
    ] {
        add(bench, key, 1e-7);
    }
}

#[test]
fn the_census_captions_are_german_in_a_german_session() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    crowded(&mut bench, &mut eq);
    let vessel = bench.vessel(VesselId(0)).expect("vessel");
    let english = census(vessel, 30).render(Register::LV2, Locale::EN);
    let german = census(vessel, 30).render(Register::LV2, Locale::parse("de"));

    assert!(
        !english.trim().is_empty(),
        "the census drew nothing, so there is no caption to translate"
    );
    assert!(
        english.contains("one") && english.contains('\u{2248}'),
        "English scale line missing:\n{english}"
    );
    assert!(
        german.contains("ein") && german.contains('\u{2248}'),
        "the scale line did not reach German:\n{german}"
    );

    for fragment in [
        "also in there",
        "present below one glyph",
        "drawn from the inventory",
        "not to scale",
        ", and ",
    ] {
        assert!(
            !german.contains(fragment),
            "English fragment {fragment:?} survived into the German census:\n{german}"
        );
    }
}

/// **The tail is the point.** It renders through the catalogue, so its
/// separator and its count phrase are the language's business.
#[test]
fn the_elided_remainder_counts_in_the_readers_language() {
    let mut eq = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    crowded(&mut bench, &mut eq);
    let vessel = bench.vessel(VesselId(0)).expect("vessel");
    let english = census(vessel, 30).render(Register::LV2, Locale::EN);
    let german = census(vessel, 30).render(Register::LV2, Locale::parse("de"));

    // Only meaningful while the census actually elides something. If the
    // fixture stops crowding the beaker this says so rather than passing
    // over a tail that is not there — which is exactly what it caught
    // twice while this test was being written.
    assert!(
        english.contains(", and ") && english.contains("more"),
        "the fixture no longer elides any species, so the tail is untested:\n{english}"
    );
    assert!(
        german.contains("weitere"),
        "the elided-remainder tail is still English in a German session:\n{german}"
    );
    assert!(
        !german.contains(", and "),
        "the English tail survived:\n{german}"
    );
}

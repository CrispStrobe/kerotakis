//! BRD-023 / bio-112: hydrogen peroxide bleaching a hair pigment.
//!
//! The claim this file defends is narrow and is exactly the question:
//! the colour goes, it takes MINUTES rather than an instant, and no atom
//! is lost while it happens. What it deliberately does not defend is a
//! rate — the entry's own uncertainty note says the constant is an
//! editorial classroom timescale — so nothing here asserts a number of
//! seconds against a measurement. It asserts the shape.

use kerotakis_core::species::{Phase, SpeciesId};
use kerotakis_core::units::{Kelvin, Moles};
use kerotakis_core::vessel::{Vessel, VesselId};
use kerotakis_core::{appearance, kinetics};

/// The corpus script's own vessel: `add v1 hair_pigment 0.001mol`,
/// `add v1 H2O2 0.01mol`. There is no water, and that is on purpose —
/// the peroxide is the only liquid, so it is also the reaction volume.
fn bleach_vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("hair_pigment"), Moles(0.001), Phase::Solid);
    v.deposit(SpeciesId::new("H2O2"), Moles(0.01), Phase::Liquid);
    v.temperature = Kelvin::STANDARD;
    v
}

fn moles(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}

#[test]
fn ten_minutes_takes_the_colour_out_and_one_minute_does_not() {
    let mut early = bleach_vessel();
    kinetics::advance(&mut early, 60.0).expect("one minute");
    assert!(
        moles(&early, "hair_pigment") > 0.5 * 0.001,
        "after a minute most of the pigment is still there: {}",
        moles(&early, "hair_pigment")
    );
    assert!(
        appearance::observe(&early).words.contains("black"),
        "and it still looks black: {}",
        appearance::observe(&early).words
    );

    let mut v = bleach_vessel();
    let before = appearance::observe(&v);
    assert!(
        before.words.contains("black"),
        "the pigment starts black: {}",
        before.words
    );

    kinetics::advance(&mut v, 600.0).expect("ten minutes");
    let after = appearance::observe(&v);
    assert!(
        !after.words.contains("black"),
        "after ten minutes nothing black is named: {}",
        after.words
    );
    assert!(
        after.words.contains("bleached hair pigment"),
        "and the colourless product is what is named instead: {}",
        after.words
    );
    // The threshold that makes the sentence change is a RATIO — a settled
    // solid is named while it is at least a tenth of the largest heap —
    // so this is the quantity the rendered words actually turn on, and
    // asserting it is what keeps the two claims from drifting apart.
    assert!(
        moles(&v, "hair_pigment") < 0.1 * moles(&v, "hair_pigment_ox"),
        "the residue is under a tenth of the product: {} against {}",
        moles(&v, "hair_pigment"),
        moles(&v, "hair_pigment_ox")
    );
}

/// The oxidant is what makes it happen, and its absence is what stops it.
/// Without this the row would pass on a pigment that fades by itself.
#[test]
fn without_peroxide_nothing_bleaches() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("hair_pigment"), Moles(0.001), Phase::Solid);
    v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
    v.temperature = Kelvin::STANDARD;
    kinetics::advance(&mut v, 3_600.0).expect("an hour");
    assert!(
        (moles(&v, "hair_pigment") - 0.001).abs() < 1e-12,
        "an hour in water changes nothing: {}",
        moles(&v, "hair_pigment")
    );
    assert!(moles(&v, "hair_pigment_ox") < 1e-12);
}

/// Ten times less oxidant is slower. The rate law is not measured, so
/// this asserts the ORDER — that the peroxide concentration is in it at
/// all — rather than any particular half-life.
///
/// Both beakers hold the same 100 mL of water on purpose. Diluting the
/// peroxide by using less of it does NOT work here: neat peroxide is its
/// own solvent, so halving the dose halves the reaction volume with it
/// and the concentration never moves. That is a real property of the
/// corpus script's vessel and worth knowing before reading a rate off it.
#[test]
fn less_peroxide_bleaches_more_slowly() {
    fn in_water(h2o2: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.deposit(SpeciesId::new("hair_pigment"), Moles(0.001), Phase::Solid);
        v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
        v.deposit(SpeciesId::new("H2O2"), Moles(h2o2), Phase::Liquid);
        v.temperature = Kelvin::STANDARD;
        v
    }
    let mut strong = in_water(0.01);
    let mut weak = in_water(0.001);
    kinetics::advance(&mut strong, 300.0).expect("five minutes");
    kinetics::advance(&mut weak, 300.0).expect("five minutes");
    assert!(
        moles(&strong, "hair_pigment_ox") > 0.0,
        "the stronger bottle bleached something"
    );
    assert!(
        moles(&weak, "hair_pigment") > moles(&strong, "hair_pigment"),
        "the weaker bottle has more pigment left: {} against {}",
        moles(&weak, "hair_pigment"),
        moles(&strong, "hair_pigment")
    );
}

/// Mass and every element, across the run. The bleach is written as one
/// atom-balanced step and the integrator has to keep it one.
#[test]
fn the_bleach_conserves_every_element() {
    use std::collections::BTreeMap;
    fn elements(v: &Vessel) -> BTreeMap<String, f64> {
        let mut totals = BTreeMap::<String, f64>::new();
        for p in &v.contents {
            let data = kerotakis_core::species::lookup(&p.species).expect("registry species");
            let parsed =
                kerotakis_core::stoich::parse_formula(data.formula).expect("formula parses");
            for (element, count) in &parsed.counts {
                *totals.entry(element.clone()).or_default() += p.moles.0 * count;
            }
        }
        totals
    }
    let mut v = bleach_vessel();
    let before = elements(&v);
    kinetics::advance(&mut v, 600.0).expect("ten minutes");
    let after = elements(&v);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    for (element, count) in &before {
        assert!(
            (count - after[element]).abs() < 1e-9 * count.max(1e-9),
            "{element}: {count} mol in, {} mol out",
            after[element]
        );
    }
}

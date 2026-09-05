//! BRD-032 / bio-103: activated charcoal taking a dye out of water.
//!
//! The point of this file is the trap the model exists to avoid. Adding
//! the charcoal as a species and stopping there would let the script run
//! and then answer WRONGLY — `filter` keeps solids and pours the
//! solution, so the carbon comes out and the dye goes through — and a
//! confident wrong answer about what removes a dye is worse than the
//! bench saying it does not know. So the first test here is the one that
//! would fail if the isotherm were deleted and the species left.

use kerotakis_core::adsorption::{self, ISOTHERMS};
use kerotakis_core::ops::Event;
use kerotakis_core::species::{self, Phase, SpeciesId};
use kerotakis_core::units::Moles;
use kerotakis_core::vessel::{Vessel, VesselId};

const DYE: &str = "methyl_orange";
const CARBON: &str = "activated_charcoal";

/// The corpus script's vessel: 100 mL of water, 0.001 mol of dye, 1 g of
/// carbon. `add v1 water 100mL` is 5.5343 mol.
fn beaker(carbon_grams: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
    v.deposit(SpeciesId::new(DYE), Moles(0.001), Phase::Aqueous);
    if carbon_grams > 0.0 {
        let carbon = species::lookup(&SpeciesId::new(CARBON)).expect("registry carbon");
        v.deposit(
            SpeciesId::new(CARBON),
            Moles(carbon_grams / carbon.molar_mass),
            Phase::Solid,
        );
    }
    v
}

fn dissolved(v: &Vessel) -> f64 {
    v.moles_of(&SpeciesId::new(DYE)).0
}

fn held(v: &Vessel) -> f64 {
    v.adsorbed_moles(&SpeciesId::new(DYE)).0
}

/// Everything a failing assertion in this file needs in order to say WHY.
/// A bare "the split is reported, not silent" tells the next reader that
/// something is wrong and nothing about what, and the numbers behind the
/// isotherm are not reachable from the outside once the call has returned.
fn state(v: &Vessel) -> String {
    let carbon = species::lookup(&SpeciesId::new(CARBON));
    let dye = species::lookup(&SpeciesId::new(DYE));
    format!(
        "[carbon {:?} g (M={:?}, mol={:?}), litres {}, dye M={:?}, dissolved {}, held {}, portions {:?}, adsorbed {:?}]",
        carbon.map(|d| v.moles_of(&SpeciesId::new(CARBON)).0 * d.molar_mass),
        carbon.map(|d| d.molar_mass),
        v.moles_of(&SpeciesId::new(CARBON)).0,
        v.liquid_volume().0,
        dye.map(|d| d.molar_mass),
        dissolved(v),
        held(v),
        v.contents
            .iter()
            .map(|p| (p.species.0.clone(), p.moles.0, p.phase))
            .collect::<Vec<_>>(),
        v.adsorbed
            .iter()
            .map(|a| (a.sorbent.0.clone(), a.sorbate.0.clone(), a.moles.0))
            .collect::<Vec<_>>(),
    )
}

#[test]
fn the_carbon_takes_most_of_the_dye_and_not_all_of_it() {
    let mut v = beaker(1.0);
    let events = adsorption::equilibrate(&mut v);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::Adsorbed { .. })),
        "the split is reported, not silent {}",
        state(&v)
    );
    assert!(
        held(&v) > 0.0,
        "something was actually adsorbed {}",
        state(&v)
    );
    // A gram of carbon at 200 mg/g holds 200 mg; the script pours 327 mg
    // of dye on it. The answer to "can charcoal remove a food dye from
    // water" is therefore "most of it, and here is what is left" — and
    // asserting BOTH halves is what stops the row from being read as a
    // clean removal.
    assert!(
        held(&v) > 0.5 * 0.001,
        "more than half is held {}",
        state(&v)
    );
    assert!(
        dissolved(&v) > 0.05 * 0.001,
        "and the rest is still in the water {}",
        state(&v)
    );
    assert!(
        (held(&v) + dissolved(&v) - 0.001).abs() < 1e-12,
        "no dye was created or destroyed {}",
        state(&v)
    );
}

/// The claim that matters, and the one a recipe alone would get wrong.
#[test]
fn the_filtrate_carries_only_what_was_still_dissolved() {
    let mut v = beaker(1.0);
    adsorption::equilibrate(&mut v);
    let bound = held(&v);
    let free = dissolved(&v);
    assert!(bound > 0.0 && free > 0.0, "{}", state(&v));

    // `filter` rewrites `contents` and touches nothing else. Reproduce
    // exactly that here rather than driving a Bench, so the test states
    // the property the verb relies on: what is bound to the retained
    // solid is retained WITH it.
    let mut filtrate = Vessel::new(VesselId(1), "beaker");
    for portion in v
        .contents
        .iter()
        .filter(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous))
    {
        filtrate.deposit(portion.species.clone(), portion.moles, portion.phase);
    }
    v.contents.retain(|p| p.phase == Phase::Solid);

    assert!(
        (dissolved(&filtrate) - free).abs() < 1e-12,
        "the filtrate carries the dissolved dye and no more"
    );
    assert!(
        filtrate.adsorbed.is_empty(),
        "and none of the bound dye followed the water"
    );
    assert!(
        (held(&v) - bound).abs() < 1e-12,
        "which is still on the carbon in the first beaker"
    );
    assert!(dissolved(&v) < 1e-12, "whose liquid has been poured off");
}

/// More carbon holds more dye. Without this the row would pass on a
/// model that removed a fixed amount regardless of the dose.
#[test]
fn more_carbon_holds_more_dye() {
    let mut little = beaker(0.1);
    let mut lots = beaker(5.0);
    adsorption::equilibrate(&mut little);
    adsorption::equilibrate(&mut lots);
    assert!(
        held(&lots) > held(&little),
        "5 g holds more than 0.1 g: {} against {}",
        state(&lots),
        state(&little)
    );
    assert!(
        dissolved(&lots) < dissolved(&little),
        "and leaves less in the water: {} against {}",
        state(&lots),
        state(&little)
    );
    // Five grams at 200 mg/g could hold 1000 mg and the beaker has 327,
    // so the excess carbon takes essentially all of it. That is the
    // isotherm being a CAPACITY and not a fixed fraction.
    assert!(
        dissolved(&lots) < 0.02 * 0.001,
        "excess carbon strips the solution {}",
        state(&lots)
    );
}

#[test]
fn no_carbon_means_no_adsorption() {
    let mut v = beaker(0.0);
    let events = adsorption::equilibrate(&mut v);
    assert!(
        events.is_empty(),
        "nothing to report: {events:?} {}",
        state(&v)
    );
    assert!(held(&v) < 1e-15);
    assert!((dissolved(&v) - 0.001).abs() < 1e-12);
}

/// Langmuir is an equilibrium, so it runs both ways: dilute the beaker
/// and the dye comes back off. Asserting it is what keeps the model an
/// isotherm rather than a one-way sink dressed as one.
#[test]
fn diluting_the_beaker_releases_some_of_the_dye() {
    let mut v = beaker(1.0);
    adsorption::equilibrate(&mut v);
    let bound_before = held(&v);
    v.deposit(SpeciesId::new("water"), Moles(10.0 * 5.5343), Phase::Liquid);
    adsorption::equilibrate(&mut v);
    assert!(
        held(&v) < bound_before,
        "eleven times the water holds less on the carbon: was {bound_before}, now {}",
        state(&v)
    );
    assert!(
        (held(&v) + dissolved(&v) - 0.001).abs() < 1e-12,
        "and the dye is still all accounted for"
    );
}

/// The bound dye is matter and the vessel has to weigh it, or every
/// adsorption reads as mass quietly leaving the beaker.
#[test]
fn binding_the_dye_does_not_change_the_vessel_mass() {
    let mut v = beaker(1.0);
    let before = v.mass().0;
    adsorption::equilibrate(&mut v);
    let after = v.mass().0;
    assert!(
        (after - before).abs() < 1e-9,
        "mass conserved across adsorption: {before} g -> {after} g"
    );
}

/// Every curated row names real registry species and carries the two
/// provenance fields the table's honesty rests on.
#[test]
fn every_isotherm_is_curated_and_says_what_it_does_not_claim() {
    assert!(!ISOTHERMS.is_empty());
    for isotherm in ISOTHERMS {
        assert!(
            species::lookup(&SpeciesId::new(isotherm.sorbent)).is_some(),
            "{} is a registry species",
            isotherm.sorbent
        );
        assert!(
            species::lookup(&SpeciesId::new(isotherm.sorbate)).is_some(),
            "{} is a registry species",
            isotherm.sorbate
        );
        assert!(isotherm.capacity_mg_per_g > 0.0);
        assert!(isotherm.affinity_l_per_mg > 0.0);
        assert!(!isotherm.boundary.is_empty(), "{}", isotherm.sorbate);
        assert!(
            isotherm.source.contains("PENDING REVIEW"),
            "{}: neither parameter is measured and the source must say so",
            isotherm.sorbate
        );
    }
}

/// The capacity is a real ceiling: the loading the model reports can
/// never exceed the curated monolayer.
#[test]
fn the_loading_never_exceeds_the_curated_capacity() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(5.5343), Phase::Liquid);
    v.deposit(SpeciesId::new(DYE), Moles(0.05), Phase::Aqueous);
    let carbon = species::lookup(&SpeciesId::new(CARBON)).expect("registry carbon");
    v.deposit(
        SpeciesId::new(CARBON),
        Moles(1.0 / carbon.molar_mass),
        Phase::Solid,
    );
    let events = adsorption::equilibrate(&mut v);
    let loading = events
        .iter()
        .find_map(|event| match event {
            Event::Adsorbed {
                loading_mg_per_g, ..
            } => Some(*loading_mg_per_g),
            _ => None,
        })
        .unwrap_or_else(|| panic!("an Adsorbed event {}", state(&v)));
    let capacity = ISOTHERMS
        .iter()
        .find(|isotherm| isotherm.sorbate == DYE)
        .expect("the curated pair")
        .capacity_mg_per_g;
    assert!(
        loading <= capacity + 1e-9,
        "{loading} mg/g is over the {capacity} mg/g monolayer"
    );
    assert!(
        loading > 0.9 * capacity,
        "and a large excess of dye should approach it: {loading}"
    );
}

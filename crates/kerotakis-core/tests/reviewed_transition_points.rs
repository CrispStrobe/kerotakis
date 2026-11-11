//! Reviewed transition points (2026-11-11).
//!
//! th-003 / an-007: every transition temperature the apparatus prints now
//! cites a primary measurement or public-domain table, never the withdrawn
//! editorial tranche. th-004 / po-009: paraffin wax and the polyolefins
//! melt over a sourced RANGE, and a solid heated past where it melts with
//! no latent heat to melt it is told so, never silently kept solid.

use kerotakis_core::instrument::{PurityVerdict, TransitionRead};
use kerotakis_core::melting_range;
use kerotakis_core::ops::NotModelledCause;
use kerotakis_core::species::Phase;
use kerotakis_core::*;

fn vessel_with(contents: &[(&str, f64, Phase)], t_k: f64) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v.temperature = Kelvin(t_k);
    v
}

fn settle(v: &mut vessel::Vessel) -> Vec<Event> {
    PhaseRouteEquilibrator.equilibrate(v).expect("phase routes")
}

fn unmodelled(events: &[Event]) -> Vec<&Event> {
    events
        .iter()
        .filter(|e| {
            matches!(e, Event::NotYetModeled {
                cause: NotModelledCause::NoReviewedDatum,
                reason: Some(reason), ..
            } if reason.key == "not-modeled.melting-not-modelled"
                || reason.key == "not-modeled.melting-range-latent-heat-unsourced")
        })
        .collect()
}

#[test]
fn no_transition_value_cites_the_withdrawn_tranche() {
    for data in species::registry() {
        let Some(t) = data.transitions else { continue };
        assert!(
            !t.source.contains("NO PRIMARY REFERENCE IS CLAIMED"),
            "{} still cites the withdrawn tranche",
            data.key
        );
        assert!(
            !t.source.contains("phase-transitions-v1"),
            "{} still names the withdrawn tranche",
            data.key
        );
    }
}

#[test]
fn the_th003_alcohols_have_sourced_points() {
    for key in ["tert_butanol", "isopropanol", "ethanol", "propanone"] {
        let t = species::lookup(&SpeciesId::new(key))
            .and_then(|d| d.transitions)
            .unwrap_or_else(|| panic!("{key} has no transitions"));
        assert!(t.melting_k.is_some(), "{key} lost its melting point");
        assert!(t.source.contains("doi"), "{key}: {}", t.source);
    }
}

#[test]
fn tert_butanol_still_melts_on_its_plateau() {
    // th-003's behaviour: a solid bottle warmed past its melting point
    // pays the latent heat and sits on the plateau; it is not warned about.
    let melting = species::lookup(&SpeciesId::new("tert_butanol"))
        .and_then(|d| d.transitions)
        .and_then(|t| t.melting_k)
        .expect("tert-butanol melting point");
    let mut v = vessel_with(&[("tert_butanol", 0.0675, Phase::Solid)], melting + 5.0);
    let events = settle(&mut v);
    assert!(unmodelled(&events).is_empty(), "{events:?}");
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::StateChanged {
                to: Phase::Liquid,
                ..
            }
        )),
        "{events:?}"
    );
}

#[test]
fn paraffin_reads_a_sourced_range_not_a_point() {
    let row = melting_range::range("paraffin").expect("paraffin range");
    let v = vessel_with(&[("paraffin", 0.014, Phase::Solid)], 293.15);
    let r = read_transition(&v, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::MeltingRange);
    assert!(r.value_c.is_none());
    let low = r.range_low_c.expect("a lower end");
    let high = r.range_high_c.expect("an upper end");
    assert!((low - (row.lower_k.expect("sourced lower end") - 273.15)).abs() < 1e-9);
    assert!((high - (row.upper_k - 273.15)).abs() < 1e-9);
    assert!(r.source.unwrap().contains("doi"));
}

#[test]
fn polyethylene_and_polypropylene_read_their_ranges() {
    for key in ["PE", "PP"] {
        let v = vessel_with(&[(key, 0.02, Phase::Solid)], 298.15);
        let r = read_transition(&v, TransitionRead::Melting);
        assert_eq!(r.verdict, PurityVerdict::MeltingRange, "{key}");
        assert!(r.value_c.is_none(), "{key}: a polymer has no point");
        assert!(r.range_high_c.is_some(), "{key}");
    }
}

#[test]
fn a_polyolefin_blend_is_a_mixture_bounded_by_the_lower_range() {
    let pe = melting_range::range("PE").expect("PE range");
    let pp = melting_range::range("PP").expect("PP range");
    let v = vessel_with(
        &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
        298.15,
    );
    let r = read_transition(&v, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Mixture);
    let bound = r.lowest_component_c.expect("a lower bound");
    let lowest = pe
        .lower_k
        .unwrap_or(pe.upper_k)
        .min(pp.lower_k.unwrap_or(pp.upper_k));
    assert!((bound - (lowest - 273.15)).abs() < 1e-9);
}

#[test]
fn heated_past_its_range_a_wax_pays_its_sourced_latent_heat_instead_of_a_warning() {
    // po-009 / th-004 (2026-11-11): paraffin now carries a sourced
    // enthalpy of fusion, so it is melted and paid for, not warned about.
    let row = melting_range::range("paraffin").expect("paraffin range");
    let mut v = vessel_with(&[("paraffin", 0.014, Phase::Solid)], row.upper_k + 5.0);
    let events = settle(&mut v);
    assert!(unmodelled(&events).is_empty(), "{events:?}");
    assert!(v.temperature.0 < row.upper_k + 5.0, "the melt cooled it");
}

#[test]
fn a_blend_at_194_c_pays_both_polymers_instead_of_warning() {
    // po-009's transcript: 0.5 g PE + 0.5 g PP heated to 193.9 °C, above
    // both sourced upper ends (141.45 and 187.55 °C). Both rows now carry
    // a sourced grade latent heat, so the melt is paid: no warning, the
    // vessel cools by the heat the melting took, and liquid appears.
    let mut v = vessel_with(
        &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
        467.04,
    );
    let events = settle(&mut v);
    assert!(unmodelled(&events).is_empty(), "{events:?}");
    assert!(v.temperature.0 < 467.04, "the melt cooled it");
    assert!(
        v.contents
            .iter()
            .any(|p| p.species.0 == "PE" && p.phase == Phase::Liquid),
        "{:?}",
        v.contents
    );
}

#[test]
fn inside_or_below_the_range_nothing_is_said() {
    let row = melting_range::range("paraffin").expect("paraffin range");
    for t in [293.15, row.upper_k - 0.5] {
        let mut v = vessel_with(&[("paraffin", 0.014, Phase::Solid)], t);
        let events = settle(&mut v);
        assert!(unmodelled(&events).is_empty(), "{t}: {events:?}");
    }
}

#[test]
fn a_liquid_is_never_warned_about() {
    let row = melting_range::range("paraffin").expect("paraffin range");
    let mut v = vessel_with(&[("paraffin", 0.014, Phase::Liquid)], row.upper_k + 20.0);
    let events = settle(&mut v);
    assert!(unmodelled(&events).is_empty(), "{events:?}");
}

#[test]
fn the_warning_is_rendered_in_german_and_french() {
    // A sharp-melting solid with no sourced enthalpy of fusion (AgNO3,
    // 483 K): every range row now pays, so the sharp-point refusal is the
    // one left to render.
    let mut v = vessel_with(&[("AgNO3", 0.01, Phase::Solid)], 500.0);
    let events = settle(&mut v);
    let Some(Event::NotYetModeled {
        reason: Some(reason),
        ..
    }) = unmodelled(&events).first().copied()
    else {
        panic!("no warning: {events:?}");
    };
    let en = reason.render(Locale::EN);
    let de = reason.render(Locale::parse("de"));
    let fr = reason.render(Locale::parse("fr"));
    assert_eq!(reason.key, "not-modeled.melting-not-modelled");
    assert!(en.contains("melting is not modelled"), "{en}");
    assert!(
        en.contains("210"),
        "the sourced melting point is quoted: {en}"
    );
    assert!(de.contains("Schmelzpunkt"), "{de}");
    assert!(fr.contains("point de fusion"), "{fr}");
}

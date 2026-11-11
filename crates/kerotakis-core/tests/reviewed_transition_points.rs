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
            } if reason.key == "not-modeled.melting-not-modelled")
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
fn heated_past_its_range_a_wax_is_told_melting_is_not_modelled() {
    let row = melting_range::range("paraffin").expect("paraffin range");
    let mut v = vessel_with(&[("paraffin", 0.014, Phase::Solid)], row.upper_k + 5.0);
    let events = settle(&mut v);
    assert_eq!(unmodelled(&events).len(), 1, "{events:?}");
}

#[test]
fn a_blend_at_194_c_warns_for_both_polymers() {
    // po-009's transcript: 0.5 g PE + 0.5 g PP heated to 193.9 °C, above
    // both sourced upper ends (141.45 and 187.55 °C).
    let mut v = vessel_with(
        &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
        467.04,
    );
    let events = settle(&mut v);
    assert_eq!(unmodelled(&events).len(), 2, "{events:?}");
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
    let row = melting_range::range("PE").expect("PE range");
    let mut v = vessel_with(&[("PE", 0.02, Phase::Solid)], row.upper_k + 30.0);
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
    assert!(en.contains("melting is not modelled"), "{en}");
    assert!(en.contains("141"), "the sourced upper end is quoted: {en}");
    assert!(de.contains("Schmelzpunkt"), "{de}");
    assert!(fr.contains("point de fusion"), "{fr}");
}

// ── Restored 2026-11-11 (transition-data-restore lane) ─────────────────
//
// Values dropped with the withdrawn tranche and restored from a primary
// table read off its page image: NBS Circular 500 (1952), Series II
// (public domain), or Smith and Calvert, J. Am. Chem. Soc. 36 (1914) 1363,
// doi:10.1021/ja02184a003, for NH4Cl. Decompositions with no thermal-
// analysis source stay absent.

fn transitions(key: &str) -> species::PhaseTransitions {
    species::lookup(&SpeciesId::new(key))
        .and_then(|d| d.transitions)
        .unwrap_or_else(|| panic!("{key} has no transitions"))
}

#[test]
fn restored_circular_500_points_carry_their_table_values() {
    // (species, melting K, boiling K) exactly as printed in the K column.
    let rows: [(&str, Option<f64>, Option<f64>); 17] = [
        ("NaCl", Some(1073.8), Some(1738.0)),
        ("AgNO3", Some(483.0), None),
        ("AgCl", Some(728.0), Some(1830.0)),
        ("NaOH", Some(593.0), None),
        ("Na2CO3", Some(1127.0), None),
        ("CaCl2", Some(1055.0), None),
        ("MgSO4", Some(1400.0), None),
        ("CaO", Some(2873.0), None),
        ("MgO", Some(3173.0), None),
        ("KOH", Some(673.0), Some(1600.0)),
        ("NaNO3", Some(583.0), None),
        ("KNO3", Some(610.0), None),
        ("KI", Some(958.0), Some(1597.0)),
        ("NaBr", Some(1023.0), Some(1665.0)),
        ("FeCl3", Some(577.0), None),
        ("Na2SO4", Some(1163.0), None),
        ("BaCl2", Some(1235.0), None),
        ("BaSO4", Some(1623.0), None),
    ];
    for (key, melting, boiling) in rows {
        let t = transitions(key);
        assert_eq!(t.melting_k, melting, "{key} melting");
        assert_eq!(t.boiling_k, boiling, "{key} boiling");
        assert!(
            t.source.contains("Circular 500"),
            "{key} must name the table it was read from: {}",
            t.source
        );
    }
}

#[test]
fn ammonium_chloride_sublimes_at_the_measured_one_atmosphere_point() {
    let t = transitions("NH4Cl");
    assert_eq!(t.sublimation_k, Some(610.95));
    assert!(t.melting_k.is_none() && t.boiling_k.is_none());
    assert!(t.source.contains("doi:10.1021/ja02184a003"), "{}", t.source);
}

#[test]
fn dry_ice_is_reviewed_against_span_and_wagner() {
    let t = transitions("dry_ice");
    assert_eq!(t.sublimation_k, Some(194.686));
    assert!(t.source.contains("doi:10.1063/1.555991"));
    assert!(!t.source.contains("PENDING"), "{}", t.source);
}

#[test]
fn unsourced_decompositions_and_boils_stay_absent() {
    // Negative: no value is invented where no primary was read.
    for key in ["Pb(NO3)2"] {
        let dec = species::lookup(&SpeciesId::new(key))
            .and_then(|d| d.transitions)
            .and_then(|t| t.decomposition_k);
        assert!(dec.is_none(), "{key} decomposition must stay no_data");
    }
    for key in ["H2O2", "NaOH", "CaCl2", "BaCl2"] {
        assert!(
            transitions(key).boiling_k.is_none(),
            "{key} boiling must stay no_data"
        );
    }
}

// ── Element boiling points from Zhang, Evans and Yang 2011 (2026-11-11) ──
//
// Y. Zhang, J. R. G. Evans and S. Yang, J. Chem. Eng. Data 56 (2011)
// 328-337, doi:10.1021/je1011086, as quoted by Wikipedia's "Boiling points
// of the elements (data page)". The NBS Circular 500 boiling points these
// replace were up to ~190 K low (aluminium 2600 K against 2743 K).

#[test]
fn element_boiling_points_follow_zhang_evans_and_yang_2011() {
    let rows: [(&str, Option<f64>, f64); 9] = [
        ("Al", Some(933.473), 2743.0),
        ("Mg", Some(922.0), 1383.0),
        ("Fe", Some(1811.0), 3273.0),
        ("Cu", Some(1357.77), 2868.0),
        ("Zn", Some(692.677), 1180.0),
        ("Ag", Some(1234.93), 2483.0),
        ("Pb", Some(600.65), 2017.0),
        ("S", Some(392.0), 718.2),
        ("I2", Some(386.8), 457.2),
    ];
    for (key, melting, boiling) in rows {
        let t = transitions(key);
        assert_eq!(t.melting_k, melting, "{key} melting is unchanged");
        assert_eq!(t.boiling_k, Some(boiling), "{key} boiling");
        assert!(
            t.source.contains("doi:10.1021/je1011086"),
            "{key} must cite Zhang, Evans and Yang 2011: {}",
            t.source
        );
    }
}

#[test]
fn circular_500_boiling_points_are_gone_for_the_elements() {
    // Negative: the 1949 values must not survive anywhere for these.
    for (key, old) in [
        ("Al", 2600.0),
        ("Mg", 1393.0),
        ("Fe", 3073.0),
        ("Cu", 2855.0),
        ("Ag", 2466.0),
        ("Pb", 2023.0),
        ("S", 717.76),
    ] {
        assert_ne!(transitions(key).boiling_k, Some(old), "{key}");
    }
}

#[test]
fn webbook_averages_are_replaced_by_one_cited_measurement() {
    // (species, boiling K, DOI of the single measurement the WebBook lists)
    for (key, boiling, doi) in [
        ("isopropanol", 355.4, "10.1021/ja01658a031"),
        ("propanone", 329.23, "10.1021/ja01658a031"),
        ("tert_butanol", 355.47, "10.1016/0021-9614(69)90047-0"),
        ("ethyl_acetate", 350.26, "10.6028/jres.018.028"),
        ("hexadecane", 559.0, "10.1039/jr9500000075"),
        ("naphthalene", 491.15, "10.1039/tf9676300447"),
        ("dodecane", 489.41, "10.6028/jres.024.022"),
    ] {
        let t = transitions(key);
        assert_eq!(t.boiling_k, Some(boiling), "{key} boiling");
        assert!(
            t.source.contains(doi),
            "{key} must cite {doi}: {}",
            t.source
        );
        assert!(
            !t.source.contains("own average"),
            "{key} still rests on a WebBook average: {}",
            t.source
        );
    }
    let dodecane = transitions("dodecane");
    assert_eq!(dodecane.melting_k, Some(263.58));
    assert!(dodecane.source.contains("doi:10.1021/ja01631a005"));
    assert!(transitions("liquid_nitrogen")
        .source
        .contains("doi:10.1063/1.555754"));
    assert!(transitions("sucrose")
        .source
        .contains("doi:10.1007/BF00912664"));
}

#[test]
fn sourced_decompositions_and_dehydrations_carry_their_thermal_analysis_values() {
    // (species, decomposition K, dehydration K, DOI of the primary)
    for (key, dec, dehyd, doi) in [
        ("CuSO4", Some(873.15), None, "10.1016/0040-6031(75)80038-4"),
        ("KMnO4", Some(523.15), None, "10.1039/j19710001821"),
        ("FeSO4", Some(873.15), None, "10.3390/ma17122800"),
        ("ZnSO4", Some(863.15), None, "10.1021/i200015a010"),
        (
            "chalcanthite",
            None,
            Some(473.15),
            "10.3390/molecules31081342",
        ),
        ("epsomite", None, Some(538.15), "10.1115/1.4000275"),
    ] {
        let t = transitions(key);
        assert_eq!(t.decomposition_k, dec, "{key} decomposition");
        assert_eq!(t.dehydration_k, dehyd, "{key} dehydration");
        assert!(
            t.source.contains(doi),
            "{key} must cite {doi}: {}",
            t.source
        );
        assert!(
            !t.source.contains("NOT A MEASURED TRANSITION"),
            "{key} is still the model threshold: {}",
            t.source
        );
    }
    // A decomposing salt reads as decomposing, never as melting.
    let (k, outcome) = transitions("KMnO4").melting_reading().expect("a reading");
    assert_eq!(k, 523.15);
    assert_eq!(outcome, species::TransitionOutcome::Decomposes);
}

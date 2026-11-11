//! po-009 / th-004 (2026-11-11): heating through a sourced melting RANGE
//! pays the sourced enthalpy of fusion, spread over the range; a range
//! with no sourced enthalpy of fusion is refused with a typed reason; and
//! a blend of components immiscible in the melt is not claimed to melt
//! below its lowest component.

use kerotakis_core::instrument::{PurityVerdict, TransitionRead, IMMISCIBLE_BLEND_BOUNDARY};
use kerotakis_core::melting_range::{self, melted_fraction};
use kerotakis_core::ops::NotModelledCause;
use kerotakis_core::script::parse_op;
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

fn in_phase(v: &vessel::Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn paraffin_moles(grams: f64) -> f64 {
    grams
        / species::lookup(&SpeciesId::new("paraffin"))
            .expect("paraffin")
            .molar_mass
}

fn refusals<'a>(events: &'a [Event], key: &str) -> Vec<&'a Event> {
    events
        .iter()
        .filter(|e| {
            matches!(e, Event::NotYetModeled {
                cause: NotModelledCause::NoReviewedDatum,
                reason: Some(reason), ..
            } if reason.key == key)
        })
        .collect()
}

#[test]
fn only_a_range_with_a_sourced_lower_bound_and_enthalpy_is_payable() {
    let row = melting_range::range("paraffin").expect("paraffin range");
    assert_eq!(row.fusion_j_per_kg, Some(189_000.0));
    let molar_mass = species::lookup(&SpeciesId::new("paraffin"))
        .unwrap()
        .molar_mass;
    let per_mol = melting_range::fusion_j_per_mol("paraffin").expect("payable");
    assert!((per_mol - 189.0 * molar_mass).abs() < 1e-6, "{per_mol}");
    // The polyolefins: no sourced grade enthalpy, no sourced lower end.
    assert!(melting_range::fusion_j_per_mol("PE").is_none());
    assert!(melting_range::fusion_j_per_mol("PP").is_none());
    // Not a range material at all.
    assert!(melting_range::fusion_j_per_mol("naphthalene").is_none());
}

#[test]
fn the_melted_fraction_is_linear_across_the_range() {
    let row = melting_range::range("paraffin").unwrap();
    let low = row.lower_k.unwrap();
    assert_eq!(melted_fraction(row, low - 1.0), 0.0);
    assert_eq!(melted_fraction(row, row.upper_k + 1.0), 1.0);
    let mid = 0.5 * (low + row.upper_k);
    assert!((melted_fraction(row, mid) - 0.5).abs() < 1e-12);
}

#[test]
fn heating_paraffin_into_its_range_pays_the_latent_heat() {
    // th-004: 5 g of paraffin at 20 °C given 500 J. Sensible heat alone
    // carried it to ~73 °C, past the whole range, with no melt.
    let n = paraffin_moles(5.0);
    let start = 293.15;
    let mut v = vessel_with(&[("paraffin", n, Phase::Solid)], start);
    let sensible_only = v.temperature_after(500.0);
    v.temperature = Kelvin(sensible_only);
    let before = v.clone();
    let events = settle(&mut v);

    let row = melting_range::range("paraffin").unwrap();
    let t = v.temperature.0;
    assert!(t < sensible_only - 1.0, "latent heat lowered it: {t}");
    assert!(
        t > row.lower_k.unwrap() && t < row.upper_k,
        "inside the range: {t}"
    );
    // The split matches the range at the new temperature.
    let liquid = in_phase(&v, "paraffin", Phase::Liquid);
    let solid = in_phase(&v, "paraffin", Phase::Solid);
    assert!((liquid + solid - n).abs() < 1e-12, "mass conserved");
    assert!((liquid - n * melted_fraction(row, t)).abs() < 1e-9 * n);
    // Energy: sensible from the start plus latent paid equals 500 J.
    let latent = melting_range::fusion_j_per_mol("paraffin").unwrap() * liquid;
    let sensible = before.energy_between(start, t);
    assert!(
        (sensible + latent - 500.0).abs() < 1e-3,
        "{sensible} + {latent} != 500"
    );
    assert!(events.iter().any(|e| matches!(
        e,
        Event::StateChanged {
            from: Phase::Solid,
            to: Phase::Liquid,
            ..
        }
    )));
    assert!(refusals(&events, "not-modeled.melting-range-latent-heat-unsourced").is_empty());
}

#[test]
fn cooling_molten_paraffin_through_its_range_releases_it() {
    let n = paraffin_moles(5.0);
    let row = melting_range::range("paraffin").unwrap();
    let mut v = vessel_with(&[("paraffin", n, Phase::Liquid)], 300.0);
    let events = settle(&mut v);
    let t = v.temperature.0;
    assert!(t > 300.0, "freezing released heat: {t}");
    assert!(t > row.lower_k.unwrap() && t < row.upper_k, "{t}");
    let liquid = in_phase(&v, "paraffin", Phase::Liquid);
    assert!((liquid - n * melted_fraction(row, t)).abs() < 1e-9 * n);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::StateChanged {
            from: Phase::Liquid,
            to: Phase::Solid,
            ..
        }
    )));
}

#[test]
fn below_its_range_or_already_consistent_nothing_moves() {
    let n = paraffin_moles(5.0);
    let mut v = vessel_with(&[("paraffin", n, Phase::Solid)], 293.15);
    assert!(settle(&mut v).is_empty());
    assert_eq!(v.temperature.0, 293.15);
    let row = melting_range::range("paraffin").unwrap();
    let mut hot = vessel_with(&[("paraffin", n, Phase::Liquid)], row.upper_k + 10.0);
    assert!(settle(&mut hot).is_empty());
    assert_eq!(hot.temperature.0, row.upper_k + 10.0);
}

#[test]
fn a_settled_split_is_stable_on_a_second_pass() {
    let n = paraffin_moles(5.0);
    let mut v = vessel_with(&[("paraffin", n, Phase::Solid)], 330.0);
    settle(&mut v);
    let t = v.temperature.0;
    let liquid = in_phase(&v, "paraffin", Phase::Liquid);
    settle(&mut v);
    assert!(
        (v.temperature.0 - t).abs() < 1e-3,
        "{} vs {t}",
        v.temperature.0
    );
    assert!((in_phase(&v, "paraffin", Phase::Liquid) - liquid).abs() < 1e-6 * n);
}

#[test]
fn a_polyolefin_past_its_range_is_refused_typed_and_left_unmoved() {
    let row = melting_range::range("PE").unwrap();
    let t = row.upper_k + 20.0;
    let mut v = vessel_with(&[("PE", 0.02, Phase::Solid)], t);
    let events = settle(&mut v);
    let found = refusals(&events, "not-modeled.melting-range-latent-heat-unsourced");
    assert_eq!(found.len(), 1, "{events:?}");
    // No invented melt and no invented latent heat.
    assert_eq!(v.temperature.0, t);
    assert!(in_phase(&v, "PE", Phase::Liquid) == 0.0);
    let Event::NotYetModeled {
        reason: Some(reason),
        ..
    } = found[0]
    else {
        unreachable!()
    };
    let en = reason.render(Locale::EN);
    let de = reason.render(Locale::parse("de"));
    let fr = reason.render(Locale::parse("fr"));
    assert!(en.contains("temperature shown overstates"), "{en}");
    assert!(de.contains("Schmelzenthalpie"), "{de}");
    assert!(fr.contains("enthalpie de fusion"), "{fr}");
}

#[test]
fn a_pe_pp_blend_is_immiscible_and_not_claimed_to_melt_lower() {
    assert!(melting_range::all_immiscible(&["PE", "PP"]));
    assert!(melting_range::all_immiscible(&["PP", "PE"]));
    assert!(!melting_range::all_immiscible(&["PE"]));
    assert!(!melting_range::all_immiscible(&["PE", "PP", "paraffin"]));

    let v = vessel_with(
        &[("PE", 0.0178, Phase::Solid), ("PP", 0.0119, Phase::Solid)],
        298.15,
    );
    let r = read_transition(&v, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Mixture);
    assert!(r.immiscible_blend);
    assert_eq!(r.boundary.as_deref(), Some(IMMISCIBLE_BLEND_BOUNDARY));
    let events = vec![Event::TransitionPointRead {
        vessel: VesselId(0),
        reading: r,
    }];
    for register in [Register::LV1, Register::LV2, Register::LV3] {
        let text = render_events(&events, register).join("\n");
        assert!(!text.contains("beginning below"), "{text}");
        assert!(!text.contains("always does"), "{text}");
    }
    let lv2 = render_events(&events, Register::LV2).join("\n");
    assert!(lv2.contains("not depressed"), "{lv2}");
    let de = render_events_in(&events, Register::LV2, Locale::parse("de")).join("\n");
    assert!(de.contains("nicht mischen"), "{de}");
    let fr = render_events_in(&events, Register::LV2, Locale::parse("fr")).join("\n");
    assert!(fr.contains("non miscibles"), "{fr}");
}

#[test]
fn a_mixture_not_sourced_as_immiscible_keeps_the_solution_law() {
    let v = vessel_with(
        &[
            ("naphthalene", 0.05, Phase::Solid),
            ("NaCl", 0.02, Phase::Solid),
        ],
        298.15,
    );
    let r = read_transition(&v, TransitionRead::Melting);
    assert_eq!(r.verdict, PurityVerdict::Mixture);
    assert!(!r.immiscible_blend);
    let boundary = r.boundary.expect("boundary");
    assert!(
        boundary.contains("dissolve in each other's melt"),
        "{boundary}"
    );
    assert!(boundary.contains("SIZE of the depression is not claimed"));
}

#[test]
fn th004_script_heats_paraffin_less_than_hexadecane_now() {
    // End to end through the bench: the wax's thermometer reading after
    // 500 J sits inside its sourced range instead of past it.
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ]);
    for command in ["add v1 paraffin 5g @ 20C", "heat v1 0.5kJ"] {
        let op = parse_op(command)
            .unwrap_or_else(|e| panic!("parse {command}: {e}"))
            .expect("operator");
        bench
            .step_with(op, &mut stack, &PermissiveScreen)
            .unwrap_or_else(|e| panic!("{command}: {e}"));
    }
    let v = bench.vessel(VesselId(0)).unwrap();
    let row = melting_range::range("paraffin").unwrap();
    let t = v.temperature.0;
    assert!(
        t > row.lower_k.unwrap() && t < row.upper_k,
        "paraffin after 500 J: {t} K"
    );
    assert!(in_phase(v, "paraffin", Phase::Liquid) > 0.0);
}

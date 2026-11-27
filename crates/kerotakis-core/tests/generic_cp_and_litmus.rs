//! Generic-Cp lane (2026-11-25): positive and negative controls for
//!
//! * Kopp's-rule heat capacity for ANY registry solid without a reviewed
//!   figure (`heat_capacity::estimated_solid_heat_capacity`), with its typed
//!   note, and reviewed figures always winning;
//! * litmus as a table-driven indicator, bleached by the generic chlorine
//!   route.

use kerotakis_core::heat_capacity::{estimated_solid_heat_capacity, kopp_solid_heat_capacity};
use kerotakis_core::solve::Equilibrator;
use kerotakis_core::vessel::Headspace;
use kerotakis_core::*;

const KOPP_NOTE: &str = "not-modeled.kopp-estimated-heat-capacity";

fn data(key: &str) -> &'static species::SpeciesData {
    species::lookup(&SpeciesId::new(key)).expect("registry species")
}

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn heat(bench: &mut Bench, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
                source: None,
            },
            &mut stack(),
            &PermissiveScreen,
        )
        .expect("heat")
}

fn note_keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(r), ..
            } => Some(r.key.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_solid_without_a_reviewed_heat_capacity_is_charged_by_kopp() {
    for key in ["Co(NO3)2", "cobalt_nitrate_hexahydrate", "Ca(CH3COO)2"] {
        let d = data(key);
        assert_eq!(
            d.heat_capacity, 0.0,
            "{key}: no reviewed Cp in the registry"
        );
        let est = estimated_solid_heat_capacity(d, Phase::Solid).expect(key);
        assert_eq!(Some(est), kopp_solid_heat_capacity(d.formula), "{key}");
        assert_eq!(
            states::heat_capacity_at(d, Phase::Solid, 298.15),
            est,
            "{key}"
        );
        let e = states::enthalpy_between(d, Phase::Solid, 298.15, 398.15);
        assert!((e - 100.0 * est).abs() < 1e-9, "{key}: {e}");
    }
    // Co(NO3)2: Co and two N take Kopp's 25.9, six O take 16.7.
    let cn = estimated_solid_heat_capacity(data("Co(NO3)2"), Phase::Solid).unwrap();
    assert!((cn - (3.0 * 25.9 + 6.0 * 16.7)).abs() < 1e-9, "{cn}");
}

#[test]
fn a_reviewed_heat_capacity_always_wins() {
    // Negative controls: a registry figure or a Cp curve is never replaced,
    // and nothing but a solid is estimated.
    for key in ["CoCl2", "Fe3O4", "NaCl", "CaCO3"] {
        assert!(
            estimated_solid_heat_capacity(data(key), Phase::Solid).is_none(),
            "{key} has a reviewed Cp"
        );
    }
    assert!(estimated_solid_heat_capacity(data("Co+2"), Phase::Aqueous).is_none());
    assert!(estimated_solid_heat_capacity(data("Co(NO3)2"), Phase::Aqueous).is_none());
    assert!(estimated_solid_heat_capacity(data("water"), Phase::Solid).is_none());
}

#[test]
fn heating_an_estimated_solid_says_so_and_lands_on_its_estimate() {
    let n = 0.01;
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("Ca(CH3COO)2"), Moles(n), Phase::Solid);
    let cp = n * estimated_solid_heat_capacity(data("Ca(CH3COO)2"), Phase::Solid).unwrap();
    let events = heat(&mut bench, 200.0);
    assert!(
        note_keys(&events).iter().any(|k| k == KOPP_NOTE),
        "{events:#?}"
    );
    let t = bench.vessel(VesselId(0)).unwrap().temperature.0;
    assert!(
        (t - (298.15 + 200.0 / cp)).abs() < 0.5,
        "{t} K for Cp {cp} J/K"
    );
}

#[test]
fn heating_a_measured_solid_says_nothing_about_an_estimate() {
    let mut bench = Bench::new();
    bench.vessels[0].deposit(SpeciesId::new("CoCl2"), Moles(0.01), Phase::Solid);
    let events = heat(&mut bench, 200.0);
    assert!(!note_keys(&events).iter().any(|k| k == KOPP_NOTE));
}

#[test]
fn every_zero_heat_capacity_registry_solid_is_kopp_estimable() {
    // The mechanism is generic: whichever registry solid carries no reviewed
    // figure is charged by Kopp. Today that is exactly the three added with
    // it, plus anhydrous copper(II) nitrate, KHSO4 and the five
    // hexacyanoferrate salts; every other zero-Cp registry row is a dissolved species, which the
    // aqueous convention (and, when dry, `vessel::residue_molar`) covers.
    let mut estimated: Vec<&str> = species::REGISTRY
        .iter()
        .filter(|d| estimated_solid_heat_capacity(d, d.standard_phase).is_some())
        .map(|d| d.key)
        .collect();
    estimated.sort_unstable();
    assert_eq!(
        estimated,
        [
            "Ca(CH3COO)2",
            "Co(NO3)2",
            // Redox species (2026-11-25): NBS lists no Cp for Cu(NO3)2 or
            // KHSO4. Complex-salts lane: none located for the five
            // hexacyanoferrate salts.
            "Cu(NO3)2",
            "Cu2Fe(CN)6",
            "K3Fe(CN)6",
            "K4Fe(CN)6",
            "KHSO4",
            "Prussian_blue",
            "Zn2Fe(CN)6",
            "cobalt_nitrate_hexahydrate"
        ]
    );
}

// ── litmus ───────────────────────────────────────────────────────────

#[test]
fn litmus_is_red_in_acid_and_blue_in_base() {
    let red = indicator::spectrum_at_ph("litmus", 3.0).expect("litmus is pH dependent");
    let blue = indicator::spectrum_at_ph("litmus", 10.0).unwrap();
    let litmus = indicator::lookup("litmus").unwrap();
    assert!(litmus.base_fraction(3.0) < 0.01);
    assert!(litmus.base_fraction(10.0) > 0.99);
    // The red form absorbs at shorter wavelengths than the blue one.
    let peak = |s: &[f64]| {
        s.iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap()
    };
    assert!(peak(&red) < peak(&blue));
    // Negative control: the bleached form is not an indicator.
    assert!(!indicator::is_ph_dependent("litmus_ox"));
}

fn chlorine_water_with(dye: &str) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(0.0277), Phase::Liquid);
    v.headspace = Headspace::Sealed {
        volume: Liters(0.02),
    };
    v.deposit(SpeciesId::new(dye), Moles(1.1e-5), Phase::Aqueous);
    v
}

#[test]
fn chlorine_water_bleaches_litmus() {
    let mut v = chlorine_water_with("litmus");
    v.deposit(SpeciesId::new("Cl2"), Moles(0.0008), Phase::Gas);
    v.solute_charge = displacement::solute_charge(&v);
    let events = CuratedEquilibrator.equilibrate(&mut v).unwrap();
    assert!(v.moles_of(&SpeciesId::new("litmus")).0 < 1e-15);
    assert!(v.moles_of(&SpeciesId::new("litmus_ox")).0 > 1.0e-5);
    assert!(!note_keys(&events)
        .iter()
        .any(|k| k == "not-modeled.chlorine-bleach-unregistered"));
}

#[test]
fn litmus_without_an_oxidant_is_untouched() {
    let mut v = chlorine_water_with("litmus");
    v.solute_charge = displacement::solute_charge(&v);
    CuratedEquilibrator.equilibrate(&mut v).unwrap();
    assert_eq!(v.moles_of(&SpeciesId::new("litmus")).0, 1.1e-5);
    assert_eq!(v.moles_of(&SpeciesId::new("litmus_ox")).0, 0.0);
}

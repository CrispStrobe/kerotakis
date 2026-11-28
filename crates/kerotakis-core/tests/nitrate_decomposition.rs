//! Dry metal nitrates decompose by cation class (2026-11-28 sweep, S2/S3).
//!
//! Group 1 except lithium: nitrate → nitrite + O2. Lithium, group 2,
//! transition and post-transition metals: nitrate → oxide + NO2 + O2. Noble
//! metals: nitrate → metal + NO2 + O2. Onsets and reaction enthalpies are
//! cited in `kerotakis_core::nitrate_decomposition`; these tests hold the
//! products, the gases, and the conservation of mass and of every element.

use std::collections::BTreeMap;

use kerotakis_core::nitrate_decomposition::decomposition;
use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

const V: VesselId = VesselId(0);

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: V,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("add")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: V,
                energy: Joules(joules),
                source: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn elements(key: &str) -> BTreeMap<String, f64> {
    let data = species::lookup_key(key).unwrap_or_else(|| panic!("{key} is a registry species"));
    stoich::parse_formula(data.formula)
        .unwrap_or_else(|e| panic!("{key}: {e:?}"))
        .counts
}

fn molar_mass(key: &str) -> f64 {
    species::lookup_key(key).unwrap().molar_mass
}

fn evolved(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == key => Some(moles.0),
            _ => None,
        })
        .sum()
}

/// Every element, and the mass, in the vessel plus everything that left as
/// gas must equal what was put in.
fn assert_conserved(bench: &Bench, events: &[Event], nitrate: &str, n: f64) {
    let vessel = bench.vessel(V).unwrap();
    let mut out: BTreeMap<String, f64> = BTreeMap::new();
    let mut mass = 0.0;
    let mut book = |key: &str, moles: f64| {
        for (el, count) in elements(key) {
            *out.entry(el).or_insert(0.0) += count * moles;
        }
        mass += moles * molar_mass(key);
    };
    for p in &vessel.contents {
        book(&p.species.0, p.moles.0);
    }
    for e in events {
        if let Event::GasEvolved { species, moles, .. } = e {
            book(&species.0, moles.0);
        }
    }
    for (el, count) in elements(nitrate) {
        let got = out.get(&el).copied().unwrap_or(0.0);
        assert!(
            (got - count * n).abs() < 1e-12,
            "{nitrate}: {el} {got} vs {} — {out:?}",
            count * n
        );
    }
    assert_eq!(out.len(), elements(nitrate).len(), "{out:?}");
    let want = n * molar_mass(nitrate);
    // Registry molar masses are rounded per species; allow their rounding.
    assert!(
        (mass - want).abs() / want < 1e-4,
        "{nitrate}: {mass} g vs {want} g"
    );
}

/// Heat enough to reach the onset and pay the whole reaction, plus margin:
/// each pass leaves the contents just below the onset (the residue's heat
/// capacity is smaller than the salt's), and that sensible heat is spent
/// without buying any reaction.
fn full_dose(bench: &Bench, nitrate: &str, n: f64) -> f64 {
    let d = decomposition(nitrate).unwrap();
    let v = bench.vessel(V).unwrap();
    v.energy_between(v.temperature.0, d.onset_k) + 1.5 * n * d.heat_j_per_mol + 50.0
}

fn decomposes(nitrate: &str, n: f64) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, nitrate, n);
    let dose = full_dose(&bench, nitrate, n);
    let events = heat(&mut bench, &mut s, dose);
    (bench, events)
}

fn moles(bench: &Bench, key: &str) -> f64 {
    bench.vessel(V).unwrap().moles_of(&SpeciesId::new(key)).0
}

#[test]
fn sodium_nitrate_gives_nitrite_and_oxygen_and_no_sodium() {
    // S2 acceptance: about 5.9 mmol O2 from 1 g NaNO3, and no sodium metal.
    let n = 1.0 / molar_mass("NaNO3");
    let (bench, events) = decomposes("NaNO3", n);
    assert!(moles(&bench, "NaNO3") < 1e-12, "{events:#?}");
    assert!((moles(&bench, "NaNO2") - n).abs() < 1e-12);
    let o2 = evolved(&events, "O2");
    assert!((o2 - n / 2.0).abs() < 1e-12, "{o2}");
    assert!((o2 - 0.0059).abs() < 0.0001, "{o2}");
    assert_eq!(evolved(&events, "NO2"), 0.0);
    assert_eq!(moles(&bench, "Na"), 0.0);
    assert!(events.iter().any(|e| matches!(e,
        Event::ReactionOccurred { equation, .. } if equation == "2 NaNO₃ →Δ 2 NaNO₂ + O₂↑")));
    assert_conserved(&bench, &events, "NaNO3", n);
}

#[test]
fn potassium_nitrate_gives_nitrite_and_oxygen() {
    let n = 0.01;
    let (bench, events) = decomposes("KNO3", n);
    assert!(moles(&bench, "KNO3") < 1e-12, "{events:#?}");
    assert!((moles(&bench, "KNO2") - n).abs() < 1e-12);
    assert!((evolved(&events, "O2") - n / 2.0).abs() < 1e-12);
    assert_eq!(evolved(&events, "NO2"), 0.0);
    assert_eq!(moles(&bench, "K"), 0.0);
    assert_conserved(&bench, &events, "KNO3", n);
}

#[test]
fn copper_nitrate_gives_black_oxide_brown_gas_and_oxygen() {
    // S3 acceptance: 2 Cu(NO3)2 → 2 CuO + 4 NO2 + O2.
    let n = 1.0 / molar_mass("Cu(NO3)2");
    let (bench, events) = decomposes("Cu(NO3)2", n);
    assert!(moles(&bench, "Cu(NO3)2") < 1e-12, "{events:#?}");
    assert!((moles(&bench, "CuO") - n).abs() < 1e-12);
    assert!((evolved(&events, "NO2") - 2.0 * n).abs() < 1e-12);
    assert!((evolved(&events, "O2") - n / 2.0).abs() < 1e-12);
    assert_conserved(&bench, &events, "Cu(NO3)2", n);
    let look = appearance::observe(bench.vessel(V).unwrap()).words;
    assert!(look.contains("black"), "{look}");
}

#[test]
fn anhydrous_copper_nitrate_is_drawn_blue_green() {
    // S3: the salt was drawn "white" because its curated appearance had no
    // reflectance behind it.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "Cu(NO3)2", 0.005);
    let look = appearance::observe(bench.vessel(V).unwrap()).words;
    assert!(look.contains("blue-green"), "{look}");
    assert!(!look.contains("white"), "{look}");
}

#[test]
fn silver_nitrate_gives_the_metal() {
    let n = 0.005;
    let (bench, events) = decomposes("AgNO3", n);
    assert!(moles(&bench, "AgNO3") < 1e-12, "{events:#?}");
    assert!((moles(&bench, "Ag") - n).abs() < 1e-12);
    assert!((evolved(&events, "NO2") - n).abs() < 1e-12);
    assert!((evolved(&events, "O2") - n / 2.0).abs() < 1e-12);
    assert_conserved(&bench, &events, "AgNO3", n);
}

#[test]
fn lead_nitrate_is_refused_while_lead_oxide_is_not_a_species() {
    // Pb(NO3)2 has a cited onset (539 K) and a class (oxide), but PbO is not
    // a registry species: the salt is left whole, and the bench says why.
    let n = 0.003;
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "Pb(NO3)2", n);
    let v = bench.vessel(V).unwrap();
    let dose = v.energy_between(v.temperature.0, 700.0);
    let events = heat(&mut bench, &mut s, dose);
    assert!(bench.vessel(V).unwrap().temperature.0 > 539.0);
    assert!((moles(&bench, "Pb(NO3)2") - n).abs() < 1e-12);
    assert_eq!(evolved(&events, "NO2"), 0.0);
    assert!(
        events.iter().any(|e| matches!(e,
        Event::NotYetModeled { reason: Some(r), .. }
            if r.key == "not-modeled.nitrate-decomposition-residue-missing")),
        "{events:#?}"
    );
}

#[test]
fn nothing_decomposes_below_the_onset() {
    for (nitrate, n) in [
        ("NaNO3", 0.01),
        ("KNO3", 0.01),
        ("Cu(NO3)2", 0.005),
        ("AgNO3", 0.005),
    ] {
        let d = decomposition(nitrate).unwrap();
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, nitrate, n);
        let v = bench.vessel(V).unwrap();
        let target = d.onset_k - 20.0;
        let dose = v.energy_between(v.temperature.0, target);
        let events = heat(&mut bench, &mut s, dose);
        let t = bench.vessel(V).unwrap().temperature.0;
        assert!(t < d.onset_k, "{nitrate}: {t} K");
        assert!(
            (moles(&bench, nitrate) - n).abs() < 1e-12,
            "{nitrate}: {events:#?}"
        );
        assert_eq!(evolved(&events, "O2"), 0.0, "{nitrate}");
        assert_eq!(evolved(&events, "NO2"), 0.0, "{nitrate}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::ReactionOccurred { .. })),
            "{nitrate}: {events:#?}"
        );
    }
}

#[test]
fn the_reaction_is_paid_from_the_heat_above_the_onset() {
    // Half the reaction heat above the onset decomposes half the salt, and
    // leaves the contents no hotter than the onset: the heat paid for the
    // reaction is gone. (Slightly below it, because the nitrite left behind
    // has a smaller heat capacity than the nitrate it replaced.)
    let n = 0.01;
    let d = decomposition("NaNO3").unwrap();
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "NaNO3", n);
    let v = bench.vessel(V).unwrap();
    let dose = v.energy_between(v.temperature.0, d.onset_k) + 0.5 * n * d.heat_j_per_mol;
    let events = heat(&mut bench, &mut s, dose);
    let left = moles(&bench, "NaNO3");
    assert!((left - 0.5 * n).abs() < 1e-4 * n, "{left}: {events:#?}");
    let t = bench.vessel(V).unwrap().temperature.0;
    assert!(t <= d.onset_k + 1e-6 && t > d.onset_k - 50.0, "{t}");
    assert_eq!(moles(&bench, "Na"), 0.0);
}

#[test]
fn a_dissolved_nitrate_is_not_this_route() {
    // In water the salt is in solution and the aqueous engine owns it.
    let mut v = Vessel::new(V, "beaker");
    v.deposit(SpeciesId::new("water"), Moles(1.0), species::Phase::Liquid);
    v.deposit(SpeciesId::new("NaNO3"), Moles(0.01), species::Phase::Solid);
    v.temperature = Kelvin(900.0);
    assert!(!nitrate_decomposition::fires(&v));
}

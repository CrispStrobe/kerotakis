//! C1 audit class "nothing crystallises on evaporation; no melting or
//! sublimation" (2026-11-24).
//!
//! Two defects shared one shape — the absence of a model reported as an
//! observation:
//!
//! * a solution taken to dryness kept its ions filed aqueous in a beaker
//!   with no water, even where the residue is uniquely decidable (one
//!   cation, or one anion, or a neutral molecular solute);
//! * sulfur, iodine, sodium hydroxide and borax had melting points and no
//!   latent heat, so they stood as solids far above them; naphthalene and
//!   iodine had boiling points and no latent heat, so they stood as liquids
//!   at any temperature, silently.
//!
//! Positive and negative cases for both.

use kerotakis_core::dryness::{crystallise_to_dryness, salt_of};
use kerotakis_core::phase_route::PhaseRouteEquilibrator;
use kerotakis_core::species::Phase;
use kerotakis_core::*;

fn vessel_with(contents: &[(&str, f64, Phase)]) -> vessel::Vessel {
    let mut v = vessel::Vessel::new(VesselId(0), "test");
    for (key, moles, phase) in contents {
        v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
    }
    v
}

fn amount(v: &vessel::Vessel, key: &str, phase: Phase) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

/// Element totals (and net charge) over every portion, any phase.
fn elements(v: &vessel::Vessel) -> std::collections::BTreeMap<String, f64> {
    let mut out = std::collections::BTreeMap::new();
    for p in &v.contents {
        let data = species::lookup(&p.species).expect("registry species");
        let f = stoich::parse_formula(data.formula).expect("formula");
        for (el, n) in f.counts {
            *out.entry(el).or_insert(0.0) += n * p.moles.0;
        }
        *out.entry("charge".to_string()).or_insert(0.0) += f.charge * p.moles.0;
    }
    out
}

fn assert_conserved(before: &vessel::Vessel, after: &vessel::Vessel) {
    let (a, b) = (elements(before), elements(after));
    for (el, n) in &a {
        let m = b.get(el).copied().unwrap_or(0.0);
        assert!((n - m).abs() < 1e-12, "{el}: {n} before, {m} after");
    }
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

// ---------------------------------------------------------------- dryness

#[test]
fn salt_lookup_is_by_formula_and_charge_balance() {
    assert_eq!(salt_of("Na+", "Cl-"), Some(("NaCl", 1.0, 1.0)));
    assert_eq!(salt_of("Na+", "SO4-2"), Some(("Na2SO4", 2.0, 1.0)));
    assert_eq!(salt_of("K+", "NO3-"), Some(("KNO3", 1.0, 1.0)));
    // No solid hydrochloric acid: H+ is not a salt-forming cation here.
    assert_eq!(salt_of("H+", "Cl-"), None);
    // Wrong way round is not a salt.
    assert_eq!(salt_of("Cl-", "Na+"), None);
}

#[test]
fn brine_dried_leaves_sodium_chloride_not_aqueous_ions() {
    // c1-6-E090 / E189: 0.0183 mol of each ion and no water.
    let mut v = vessel_with(&[
        ("Na+", 0.0183, Phase::Aqueous),
        ("Cl-", 0.0183, Phase::Aqueous),
    ]);
    let before = v.clone();
    let residue = crystallise_to_dryness(&mut v);
    assert!((amount(&v, "NaCl", Phase::Solid) - 0.0183).abs() < 1e-12);
    assert_eq!(amount(&v, "Na+", Phase::Aqueous), 0.0);
    assert_eq!(amount(&v, "Cl-", Phase::Aqueous), 0.0);
    assert!(residue.stranded.is_empty(), "{:?}", residue.stranded);
    assert!(residue.events.iter().any(|e| matches!(
        e,
        Event::Precipitated { species, .. } if species.0 == "NaCl"
    )));
    assert!(note_keys(&residue.events)
        .iter()
        .any(|k| k == "not-modeled.dry-residue-anhydrous"));
    assert_conserved(&before, &v);
}

#[test]
fn a_common_cation_decides_every_salt() {
    let mut v = vessel_with(&[
        ("Na+", 0.03, Phase::Aqueous),
        ("Cl-", 0.01, Phase::Aqueous),
        ("SO4-2", 0.01, Phase::Aqueous),
    ]);
    let before = v.clone();
    let residue = crystallise_to_dryness(&mut v);
    assert!((amount(&v, "NaCl", Phase::Solid) - 0.01).abs() < 1e-12);
    assert!((amount(&v, "Na2SO4", Phase::Solid) - 0.01).abs() < 1e-12);
    assert!(residue.stranded.is_empty());
    assert_conserved(&before, &v);
}

#[test]
fn a_molecular_solute_dries_to_its_own_solid() {
    // c1-6-E004: sugar water evaporated.
    let mut v = vessel_with(&[("sucrose", 0.0146, Phase::Aqueous)]);
    let residue = crystallise_to_dryness(&mut v);
    assert!((amount(&v, "sucrose", Phase::Solid) - 0.0146).abs() < 1e-12);
    assert_eq!(amount(&v, "sucrose", Phase::Aqueous), 0.0);
    assert!(residue.stranded.is_empty());
}

#[test]
fn a_reciprocal_salt_system_is_not_guessed() {
    // Na+/K+/Cl-/NO3- can dry to NaCl + KNO3 or KCl + NaNO3.
    let mut v = vessel_with(&[
        ("Na+", 0.01, Phase::Aqueous),
        ("K+", 0.01, Phase::Aqueous),
        ("Cl-", 0.01, Phase::Aqueous),
        ("NO3-", 0.01, Phase::Aqueous),
    ]);
    let residue = crystallise_to_dryness(&mut v);
    assert!(residue.events.is_empty(), "{:#?}", residue.events);
    assert_eq!(residue.stranded.len(), 4);
    assert!(!v.contents.iter().any(|p| p.phase == Phase::Solid));
}

#[test]
fn an_ion_with_no_solid_partner_stays_stranded() {
    // Excess hydrochloric acid beside brine: NaCl crystallises, H+ and its
    // share of the chloride do not become an invented solid.
    let mut v = vessel_with(&[
        ("Na+", 0.01, Phase::Aqueous),
        ("H+", 0.005, Phase::Aqueous),
        ("Cl-", 0.015, Phase::Aqueous),
    ]);
    let before = v.clone();
    let residue = crystallise_to_dryness(&mut v);
    assert!((amount(&v, "NaCl", Phase::Solid) - 0.01).abs() < 1e-12);
    assert!((amount(&v, "H+", Phase::Aqueous) - 0.005).abs() < 1e-12);
    assert!((amount(&v, "Cl-", Phase::Aqueous) - 0.005).abs() < 1e-12);
    assert_eq!(residue.stranded.len(), 2, "{:?}", residue.stranded);
    assert_conserved(&before, &v);

    // Pure acid: nothing at all is booked, and no residue note is said.
    let mut acid = vessel_with(&[("H+", 0.01, Phase::Aqueous), ("Cl-", 0.01, Phase::Aqueous)]);
    let residue = crystallise_to_dryness(&mut acid);
    assert!(residue.events.is_empty());
    assert_eq!(residue.stranded.len(), 2);
}

#[test]
fn evaporating_to_dryness_on_the_bench_books_the_residue() {
    let mut bench = Bench::new();
    let mut s = SolverStack::new(vec![
        Box::new(MixingEquilibrator) as Box<dyn Equilibrator>,
        Box::new(HonestyEquilibrator),
    ]);
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
                at: None,
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("add water");
    bench.vessels[0].deposit(SpeciesId::new("Na+"), Moles(0.01), Phase::Aqueous);
    bench.vessels[0].deposit(SpeciesId::new("Cl-"), Moles(0.01), Phase::Aqueous);
    let events = bench
        .step_with(
            Operator::Evaporate {
                vessel: VesselId(0),
                fraction: 1.0,
            },
            &mut s,
            &PermissiveScreen,
        )
        .expect("evaporate");
    let v = bench.vessel(VesselId(0)).unwrap();
    assert!(
        (amount(v, "NaCl", Phase::Solid) - 0.01).abs() < 1e-12,
        "{:#?}",
        v.contents
    );
    assert!(
        !events.iter().any(|e| matches!(
            e,
            Event::NotYetModeled { what, .. } if what.contains("the last of the water is gone")
        )),
        "nothing is left stranded: {events:#?}"
    );
}

// ------------------------------------------------------- melting / boiling

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(PhaseRouteEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, s: &mut SolverStack, key: &str, n: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(n),
                at: None,
            },
            s,
            &PermissiveScreen,
        )
        .expect("add");
}

fn heat(bench: &mut Bench, s: &mut SolverStack, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(joules),
                source: None,
            },
            s,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn total(bench: &Bench, key: &str, phase: Phase) -> f64 {
    amount(bench.vessel(VesselId(0)).unwrap(), key, phase)
}

#[test]
fn sulfur_melts_on_its_plateau_and_pays_the_latent_heat() {
    // c1-6-E134. 0.1 mol S: 25 -> 118.85 C is about 0.1 x 22.6 x 93.85 =
    // 212 J; melting it all costs 0.1 x 1728 = 173 J. 300 J lands on the
    // plateau with both phases present.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "S", 0.1);
    let events = heat(&mut bench, &mut s, 300.0);
    let t = bench.vessel(VesselId(0)).unwrap().temperature.0;
    assert!((t - 392.0).abs() < 0.5, "on the plateau: {t} K");
    assert!(total(&bench, "S", Phase::Liquid) > 1e-6);
    assert!(total(&bench, "S", Phase::Solid) > 1e-6);
    assert!(!note_keys(&events)
        .iter()
        .any(|k| k == "not-modeled.melting-not-modelled"));
    // Enough heat melts all of it.
    heat(&mut bench, &mut s, 400.0);
    assert!(total(&bench, "S", Phase::Solid) < 1e-9);
}

#[test]
fn sodium_hydroxide_and_borax_melt() {
    for (key, n, joules) in [("NaOH", 0.05, 6_000.0), ("Na2B4O7", 0.001, 400.0)] {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, key, n);
        let events = heat(&mut bench, &mut s, joules);
        assert!(
            total(&bench, key, Phase::Liquid) > 1e-6,
            "{key}: {events:#?}"
        );
        assert!(
            !note_keys(&events)
                .iter()
                .any(|k| k == "not-modeled.melting-not-modelled"),
            "{key}"
        );
    }
}

#[test]
fn naphthalene_and_iodine_boil_instead_of_standing_as_liquids() {
    // c1-6-E290 heated 1.56 mmol naphthalene to 1500 C and kept it liquid.
    for key in ["naphthalene", "I2"] {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, key, 0.002);
        let events = heat(&mut bench, &mut s, 2_000.0);
        assert!(
            total(&bench, key, Phase::Liquid) < 1e-9,
            "{key} still liquid: {events:#?}"
        );
        assert!(
            !note_keys(&events)
                .iter()
                .any(|k| k == "not-modeled.boiling-not-modelled"),
            "{key}"
        );
    }
}

#[test]
fn concentrated_sulfuric_acid_boils_at_its_janaf_point() {
    // c1-6-E147 heated 0.05 mol H2SO4 (with copper) to 947 C and kept the
    // acid liquid. NSRDS-NBS 37 gives Tb = 608.6 K, Delta-Hv = 13.39
    // kcal/mol: 0.05 mol from 25 C needs ~0.05 x 155 x 310 = 2.4 kJ of
    // sensible heat plus 0.05 x 56.02 = 2.8 kJ to boil it all.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "H2SO4", 0.05);
    let events = heat(&mut bench, &mut s, 3_500.0);
    let t = bench.vessel(VesselId(0)).unwrap().temperature.0;
    // Part boiled: the vessel sits on the 608.6 K plateau, not above it.
    assert!(
        (t - 608.6).abs() < 0.5,
        "on the boiling plateau: {t} K {events:#?}"
    );
    let liquid = total(&bench, "H2SO4", Phase::Liquid);
    assert!(liquid > 1e-4 && liquid < 0.05, "partly boiled: {liquid}");
    assert!(!note_keys(&events)
        .iter()
        .any(|k| k == "not-modeled.boiling-not-modelled"));
    // Enough heat boils all of it; none is left as a liquid above 608.6 K.
    heat(&mut bench, &mut s, 6_000.0);
    assert!(total(&bench, "H2SO4", Phase::Liquid) < 1e-9);
}

#[test]
fn sulfuric_acid_below_its_boiling_point_stays_liquid() {
    // Negative control: 0.05 mol warmed by 1 kJ (~150 C) does not boil.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "H2SO4", 0.05);
    heat(&mut bench, &mut s, 1_000.0);
    let t = bench.vessel(VesselId(0)).unwrap().temperature.0;
    assert!(t < 600.0 && t > 350.0, "{t} K");
    assert!((total(&bench, "H2SO4", Phase::Liquid) - 0.05).abs() < 1e-9);
}

#[test]
fn a_liquid_with_no_sourced_boil_is_refused_not_kept_silently() {
    // Dodecane has a reviewed 489 K boiling point and no enthalpy of
    // vaporisation on this bench: past 489 K it must be said.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "dodecane", 0.01);
    let events = heat(&mut bench, &mut s, 20_000.0);
    assert!(bench.vessel(VesselId(0)).unwrap().temperature.0 > 489.41);
    assert!(
        note_keys(&events)
            .iter()
            .any(|k| k == "not-modeled.boiling-not-modelled"),
        "{events:#?}"
    );
}

#[test]
fn a_solid_with_no_sourced_fusion_still_says_so() {
    // Potassium nitrate (610 K) keeps its refusal: no row was invented.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "KNO3", 0.01);
    let events = heat(&mut bench, &mut s, 10_000.0);
    assert!(
        note_keys(&events)
            .iter()
            .any(|k| k == "not-modeled.melting-not-modelled"),
        "{events:#?}"
    );
}

#[test]
fn boiling_brine_dry_books_sodium_chloride() {
    // The boiling branch of the state solver takes the water away; it books
    // the same residue `evaporate` does.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 0.5);
    add(&mut bench, &mut s, "Na+", 0.01);
    add(&mut bench, &mut s, "Cl-", 0.01);
    let events = heat(&mut bench, &mut s, 30_000.0);
    assert!(total(&bench, "water", Phase::Liquid) < 1e-9);
    let salt = total(&bench, "NaCl", Phase::Solid) + total(&bench, "NaCl", Phase::Liquid);
    assert!((salt - 0.01).abs() < 1e-9, "{events:#?}");
    assert!(total(&bench, "Na+", Phase::Aqueous) < 1e-12);
    assert!(total(&bench, "Cl-", Phase::Aqueous) < 1e-12);
}

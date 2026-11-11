//! EXP-14: amylase-catalysed starch hydrolysis.
//!
//! Since 2026-11-11 (batch3-diverse:en-001) digestion advances over bench
//! time at the cited unit-definition rate (`starch_amylase`): 1 U liberates
//! 1.0 mg maltose in 3 min, ≥5 U/mg (Sigma-Aldrich A3176). Nothing happens
//! on `add`; a wait is needed. 1e-6 mol amylase (55 mg, 275 U) liberates
//! 0.05 mol maltose (17.1 g) in about 11 200 s, so the tests wait 4 h.

use kerotakis_core::*;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

/// Long enough for 1e-6 mol amylase to finish 0.1 mol repeat units.
const DIGEST_SECONDS: f64 = 4.0 * 3600.0;

fn wait(bench: &mut Bench, stack: &mut SolverStack, seconds: f64) -> Vec<Event> {
    bench
        .step_with(Operator::Wait { seconds }, stack, &PermissiveScreen)
        .expect("wait")
}

/// The unit rate is for gelatinised (cooked) starch. The bench books that
/// state when starch sits in water at >= 85 °C while time passes
/// (`cooking_at_85_c_or_above_gelatinises_starch_on_the_bench`); the
/// positive tests below start from that booked state directly.
fn cook(bench: &mut Bench) {
    let v = &mut bench.vessels[0];
    let starch = v.moles_of(&SpeciesId::new("starch")).0;
    v.starch_gelatinised_moles = Some(starch);
}

fn total_moles(bench: &Bench, key: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .unwrap()
        .moles_of(&SpeciesId::new(key))
        .0
}

fn total_element(bench: &Bench, element: &str) -> f64 {
    let vessel = bench.vessel(VesselId(0)).unwrap();
    let mut total = 0.0;
    for p in &vessel.contents {
        if let Some(data) = species::lookup(&p.species) {
            if let Ok(f) = stoich::parse_formula(data.formula) {
                total += p.moles.0 * f.counts.get(element).copied().unwrap_or(0.0);
            }
        }
    }
    total
}

// ── Stoichiometric balance ───────────────────────────────────────────

#[test]
fn equation_balances() {
    let eq = stoich::parse_equation("2 C6H10O5 + H2O -> C12H22O11").expect("parse");
    assert!(eq.is_balanced(), "imbalance: {:?}", eq.element_imbalance());
}

// ── Enzyme gate ─────────────────────────────────────────────────────

#[test]
fn fires_with_amylase() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    cook(&mut bench);
    let on_add = add(&mut bench, &mut s, "amylase", 1e-6);
    assert!(
        !on_add
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("amylase"))),
        "hydrolysis must not complete on add: {on_add:?}"
    );
    let events = wait(&mut bench, &mut s, 600.0);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("amylase"))),
        "hydrolysis must proceed during a wait: {events:?}"
    );
}

#[test]
fn does_not_fire_without_amylase() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    let events = add(&mut bench, &mut s, "starch", 0.1);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::ReactionOccurred { equation, .. }
                if equation.contains("amylase") || equation.contains("maltose"))),
        "hydrolysis must NOT fire without amylase: {events:?}"
    );
}

#[test]
fn amylase_not_consumed() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    cook(&mut bench);
    add(&mut bench, &mut s, "amylase", 1e-6);
    wait(&mut bench, &mut s, 600.0);
    let amylase = total_moles(&bench, "amylase");
    assert!(
        (amylase - 1e-6).abs() < 1e-12,
        "amylase must not be consumed: {amylase}"
    );
}

// ── Products ────────────────────────────────────────────────────────

#[test]
fn produces_maltose() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    cook(&mut bench);
    add(&mut bench, &mut s, "amylase", 1e-6);
    wait(&mut bench, &mut s, DIGEST_SECONDS);
    let maltose = total_moles(&bench, "maltose");
    assert!(
        (maltose - 0.05).abs() < 1e-6,
        "expected 0.05 mol maltose (2:1 starch:maltose): {maltose}"
    );
}

#[test]
fn starch_consumed() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    cook(&mut bench);
    add(&mut bench, &mut s, "amylase", 1e-6);
    wait(&mut bench, &mut s, DIGEST_SECONDS);
    let starch = total_moles(&bench, "starch");
    assert!(starch < 1e-9, "starch should be fully consumed: {starch}");
}

// ── Conservation ────────────────────────────────────────────────────

#[test]
fn conserves_elements() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    let c_before = total_element(&bench, "C");
    let h_before = total_element(&bench, "H");
    let o_before = total_element(&bench, "O");
    // 1e-9 mol: catalytic trace, well above TRACE (1e-12) but negligible element contribution
    add(&mut bench, &mut s, "amylase", 1e-9);
    wait(&mut bench, &mut s, 600.0);
    let c_after = total_element(&bench, "C");
    let h_after = total_element(&bench, "H");
    let o_after = total_element(&bench, "O");
    assert!(
        (c_after - c_before).abs() < 1e-6,
        "C: {c_before} → {c_after}"
    );
    assert!(
        (h_after - h_before).abs() < 1e-6,
        "H: {h_before} → {h_after}"
    );
    assert!(
        (o_after - o_before).abs() < 1e-6,
        "O: {o_before} → {o_after}"
    );
}

// ── Starch-positive vs starch-negative (Lugol) ──────────────────────

#[test]
fn starch_positive_without_enzyme() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    let starch = total_moles(&bench, "starch");
    assert!(starch > 0.09, "starch must remain without enzyme: {starch}");
}

#[test]
fn starch_negative_after_enzyme() {
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    cook(&mut bench);
    add(&mut bench, &mut s, "amylase", 1e-6);
    wait(&mut bench, &mut s, DIGEST_SECONDS);
    let starch = total_moles(&bench, "starch");
    assert!(
        starch < 1e-9,
        "starch must be consumed after enzyme: {starch}"
    );
}

// ── Rendering ───────────────────────────────────────────────────────

#[test]
fn three_registers_render() {
    let reaction = Event::ReactionOccurred {
        vessel: VesselId(0),
        equation: "2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁".into(),
    };
    for level in [1, 2, 3] {
        let text = render::render_event(&reaction, render::Register(level));
        assert!(!text.is_empty(), "level {level} must produce output");
    }
}

// ── Time course (batch3-diverse:en-001) ─────────────────────────────

#[test]
fn digestion_is_partial_before_the_cited_rate_can_finish() {
    // 1e-6 mol (55 mg, 275 U) for 600 s: 275 × 600/180 = 916.7 mg maltose,
    // 2.678e-3 mol, i.e. 5.356e-3 mol repeat units of the 0.1 mol.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 55.0);
    add(&mut bench, &mut s, "starch", 0.1);
    cook(&mut bench);
    add(&mut bench, &mut s, "amylase", 1e-6);
    assert!((total_moles(&bench, "starch") - 0.1).abs() < 1e-12);
    wait(&mut bench, &mut s, 600.0);
    let maltose = total_moles(&bench, "maltose");
    let enzyme_mg = 1e-6 * species::lookup_key("amylase").unwrap().molar_mass * 1000.0;
    let maltose_mg = enzyme_mg * 5.0 * 600.0 / 180.0;
    let expected = maltose_mg / 1000.0 / species::lookup_key("maltose").unwrap().molar_mass;
    assert!(
        (maltose - expected).abs() < 1e-9,
        "maltose {maltose} vs cited-rate {expected}"
    );
    assert!((total_moles(&bench, "starch") - (0.1 - 2.0 * maltose)).abs() < 1e-9);
}

#[test]
fn outside_the_cited_ph_window_digestion_is_refused_typed() {
    let mut v = Vessel::new(VesselId(0), "acid digestion");
    v.deposit(SpeciesId::new("water"), Moles(55.0), Phase::Liquid);
    v.deposit(SpeciesId::new("starch"), Moles(0.1), Phase::Solid);
    v.deposit(SpeciesId::new("amylase"), Moles(1e-6), Phase::Aqueous);
    v.starch_gelatinised_moles = Some(0.1);
    v.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: SolutionScope::Complete,
        solvent_kg: Some(55.0 * 0.018015),
        particle_molality: None,
        ph: 1.0,
        ionic_strength: 0.1,
        pe: None,
        redox: vec![],
        species: vec![],
        provenance: None,
    });
    let events = starch_amylase::advance(&mut v, 600.0);
    let phrase = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(p), ..
            } if p.key == "not-modeled.amylase-outside-unit-conditions" => Some(p),
            _ => None,
        })
        .unwrap_or_else(|| panic!("typed refusal: {events:?}"));
    let en = phrase.render(i18n::Locale::EN);
    for locale in ["de", "fr"] {
        assert_ne!(phrase.render(i18n::Locale::parse(locale)), en);
    }
    assert!((v.moles_of(&SpeciesId::new("starch")).0 - 0.1).abs() < 1e-12);
    assert_eq!(v.moles_of(&SpeciesId::new("maltose")).0, 0.0);
    // Inside the window the same vessel digests.
    if let Some(solution) = v.solution.as_mut() {
        solution.ph = 6.9;
    }
    starch_amylase::advance(&mut v, 600.0);
    assert!(v.moles_of(&SpeciesId::new("maltose")).0 > 0.0);
}

// ── Coverage table (docs/starch-amylase-coverage-20261111.json) ─────

#[test]
fn the_coverage_table_matches_the_cited_constants() {
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/starch-amylase-coverage-20261111.json"
    ))
    .unwrap();
    let a = &table["activity"];
    assert_eq!(
        a["specific_activity_u_per_mg"].as_f64().unwrap(),
        starch_amylase::SPECIFIC_ACTIVITY_U_PER_MG
    );
    assert_eq!(
        a["unit_seconds"].as_f64().unwrap(),
        starch_amylase::UNIT_SECONDS
    );
    assert_eq!(a["instant_curated_row_present"], false);
    assert!(!curated::REACTIONS
        .iter()
        .any(|r| r.catalyst == Some("amylase")));
    assert_eq!(table["starch_iodine_optics"]["status"], "refused");
}

// ── Heat inactivation (Miłek 2021, doi:10.3390/molecules26144117) ───

fn heated_digestion_vessel(celsius: f64) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "heated digestion");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v.deposit(SpeciesId::new("starch"), Moles(0.1), Phase::Solid);
    v.deposit(SpeciesId::new("amylase"), Moles(1e-6), Phase::Aqueous);
    v.temperature = Kelvin::from_celsius(celsius);
    v.starch_gelatinised_moles = Some(0.1);
    v
}

#[test]
fn boiling_inactivates_amylase_and_digests_nothing() {
    // curiosity bio-030: starch and amylase in water at 100 °C.
    let mut v = heated_digestion_vessel(100.0);
    let events = starch_amylase::advance(&mut v, 300.0);
    let phrase = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled {
                reason: Some(p), ..
            } if p.key == "not-modeled.amylase-heat-inactivated" => Some(p),
            _ => None,
        })
        .unwrap_or_else(|| panic!("typed heat-inactivation event: {events:?}"));
    let en = phrase.render(i18n::Locale::EN);
    for locale in ["de", "fr"] {
        assert_ne!(phrase.render(i18n::Locale::parse(locale)), en);
    }
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::ReactionOccurred { .. })));
    assert!((v.moles_of(&SpeciesId::new("starch")).0 - 0.1).abs() < 1e-12);
    assert_eq!(v.moles_of(&SpeciesId::new("maltose")).0, 0.0);
    assert_eq!(starch_amylase::active_enzyme_moles(&v), 0.0);
    assert!(!starch_amylase::consumes(&v, &SpeciesId::new("starch")));
}

#[test]
fn heat_inactivation_is_irreversible_on_cooling_but_fresh_enzyme_works() {
    let mut v = heated_digestion_vessel(90.0);
    starch_amylase::advance(&mut v, 60.0);
    v.temperature = Kelvin::from_celsius(20.0);
    let events = starch_amylase::advance(&mut v, 600.0);
    assert!(events.iter().any(|e| matches!(
        e,
        Event::NotYetModeled { reason: Some(p), .. }
            if p.key == "not-modeled.amylase-heat-inactivated"
    )));
    assert_eq!(v.moles_of(&SpeciesId::new("maltose")).0, 0.0);
    // A fresh dose added after cooling is active.
    v.deposit(SpeciesId::new("amylase"), Moles(1e-6), Phase::Aqueous);
    assert!((starch_amylase::active_enzyme_moles(&v) - 1e-6).abs() < 1e-15);
    starch_amylase::advance(&mut v, 600.0);
    assert!(v.moles_of(&SpeciesId::new("maltose")).0 > 0.0);
}

#[test]
fn body_temperature_and_room_temperature_digestion_are_unchanged() {
    // At or below the fitted optimum (311.06 K) no inactivation is claimed.
    for celsius in [20.0, 37.0] {
        let mut v = heated_digestion_vessel(celsius);
        starch_amylase::advance(&mut v, 600.0);
        assert_eq!(v.amylase_inactive_moles, None, "{celsius} °C");
        // 1e-6 mol = 55 mg (registry mass), 275 U, 600 s: 916.7 mg maltose.
        let mg = v.moles_of(&SpeciesId::new("maltose")).0
            * species::lookup_key("maltose").unwrap().molar_mass
            * 1000.0;
        let m = species::lookup_key("amylase").unwrap().molar_mass;
        let expected = 1e-6 * m * 1000.0 * 5.0 * 600.0 / 180.0;
        assert!((mg - expected).abs() < 1e-6 * expected, "{celsius}: {mg}");
    }
}

#[test]
fn falling_limb_inactivation_follows_the_cited_first_order_constant() {
    // 50 °C: k_d = (1.46/3) exp[(164900/8.315)(1/311.06 - 1/323.15)] per min.
    let t: f64 = 323.15;
    let k_min = 1.46 / 3.0 * (164_900.0 / 8.315 * (1.0 / 311.06 - 1.0 / t)).exp();
    let k = starch_amylase::inactivation_rate_per_s(t).unwrap();
    assert!((k * 60.0 - k_min).abs() < 1e-9 * k_min);
    let mut v = heated_digestion_vessel(50.0);
    starch_amylase::advance(&mut v, 60.0);
    let lost = v.amylase_inactive_moles.unwrap();
    let expected = 1e-6 * (1.0 - (-k * 60.0).exp());
    assert!((lost - expected).abs() < 1e-12 * 1e-6 + 1e-9 * expected);
    // Partial digestion: less than the same minute at 37 °C, more than zero.
    let warm = v.moles_of(&SpeciesId::new("maltose")).0;
    let mut body = heated_digestion_vessel(37.0);
    starch_amylase::advance(&mut body, 60.0);
    let full = body.moles_of(&SpeciesId::new("maltose")).0;
    assert!(warm > 0.0 && warm < full, "{warm} vs {full}");
    assert!((warm - full * (1.0 - (-k * 60.0).exp()) / (k * 60.0)).abs() < 1e-9 * full);
    // Monotone, and complete above the fitted data ceiling.
    assert_eq!(starch_amylase::inactivation_rate_per_s(311.06), Some(0.0));
    assert!(
        starch_amylase::inactivation_rate_per_s(333.15).unwrap()
            > starch_amylase::inactivation_rate_per_s(323.15).unwrap()
    );
    assert_eq!(
        starch_amylase::inactivation_rate_per_s(343.16),
        Some(f64::INFINITY)
    );
}

#[test]
fn heat_inactivation_constants_match_the_coverage_table() {
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/starch-amylase-coverage-20261111.json"
    ))
    .unwrap();
    let h = &table["heat_inactivation"];
    assert_eq!(
        h["t_opt_k"].as_f64().unwrap(),
        starch_amylase::INACTIVATION_T_OPT_K
    );
    assert_eq!(
        h["e_d_j_per_mol"].as_f64().unwrap(),
        starch_amylase::INACTIVATION_E_D_J_PER_MOL
    );
    assert_eq!(
        h["beta"].as_f64().unwrap(),
        starch_amylase::INACTIVATION_BETA
    );
    assert_eq!(
        h["complete_loss_above_k"].as_f64().unwrap(),
        starch_amylase::COMPLETE_LOSS_ABOVE_K
    );
    assert_eq!(h["refusal"], "not-modeled.amylase-heat-inactivated");
}

// ── Native granules vs gelatinised starch (EN-001 rerun) ────────────
// Slaughter, Ellis, Butterworth, BBA 1525 (2001) 29-36,
// doi:10.1016/S0304-4165(00)00162-8: granules are digested 13- to 239-fold
// less efficiently depending on plant source, so no granule rate is claimed.
// Gelatinisation onset 55-85 °C (Belitz et al., Food Chemistry 2004,
// pp. 318-323, as quoted by Wikipedia "Starch gelatinization").

fn native_refusal(events: &[Event]) -> Option<&Phrase> {
    events.iter().find_map(|e| match e {
        Event::NotYetModeled {
            reason: Some(p), ..
        } if p.key == "not-modeled.amylase-native-starch-granules" => Some(p),
        _ => None,
    })
}

#[test]
fn native_starch_granules_at_37_c_are_refused_typed_not_digested() {
    // The EN-001 protocol: starch stirred into 37 °C water, amylase, 20 min.
    let mut bench = Bench::new();
    let mut s = stack();
    add(&mut bench, &mut s, "water", 2.767);
    bench.vessels[0].temperature = Kelvin::from_celsius(37.0);
    add(&mut bench, &mut s, "starch", 0.20 / 162.14);
    add(&mut bench, &mut s, "amylase", 1.8e-7);
    let events = wait(&mut bench, &mut s, 1200.0);
    let phrase = native_refusal(&events)
        .unwrap_or_else(|| panic!("typed native-granule refusal: {events:?}"));
    let en = phrase.render(i18n::Locale::EN);
    for locale in ["de", "fr"] {
        assert_ne!(phrase.render(i18n::Locale::parse(locale)), en);
    }
    assert!(!events.iter().any(|e| matches!(e,
        Event::ReactionOccurred { equation, .. } if equation.contains("amylase"))));
    assert!((total_moles(&bench, "starch") - 0.20 / 162.14).abs() < 1e-15);
    assert_eq!(total_moles(&bench, "maltose"), 0.0);
}

#[test]
fn cooking_at_85_c_or_above_gelatinises_starch_on_the_bench() {
    let mut v = Vessel::new(VesselId(0), "cook");
    v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    v.deposit(SpeciesId::new("starch"), Moles(0.01), Phase::Solid);
    // Below the highest cited onset nothing is booked.
    v.temperature = Kelvin::from_celsius(80.0);
    starch_amylase::advance(&mut v, 600.0);
    assert_eq!(starch_amylase::gelatinised_moles(&v), 0.0);
    // At 90 °C it is, irreversibly.
    v.temperature = Kelvin::from_celsius(90.0);
    starch_amylase::advance(&mut v, 60.0);
    assert!((starch_amylase::gelatinised_moles(&v) - 0.01).abs() < 1e-15);
    v.temperature = Kelvin::from_celsius(37.0);
    v.deposit(SpeciesId::new("amylase"), Moles(1e-7), Phase::Aqueous);
    let events = starch_amylase::advance(&mut v, 60.0);
    assert!(native_refusal(&events).is_none(), "{events:?}");
    assert!(v.moles_of(&SpeciesId::new("maltose")).0 > 0.0);
}

#[test]
fn only_the_gelatinised_pool_is_digested_when_raw_starch_is_added_after_cooking() {
    let mut v = Vessel::new(VesselId(0), "mixed pools");
    v.deposit(SpeciesId::new("water"), Moles(55.0), Phase::Liquid);
    v.deposit(SpeciesId::new("starch"), Moles(0.01), Phase::Solid);
    v.temperature = Kelvin::from_celsius(95.0);
    starch_amylase::advance(&mut v, 60.0);
    v.temperature = Kelvin::from_celsius(37.0);
    // Raw starch stirred in after cooling stays native.
    v.deposit(SpeciesId::new("starch"), Moles(0.02), Phase::Solid);
    v.deposit(SpeciesId::new("amylase"), Moles(1e-6), Phase::Aqueous);
    let events = starch_amylase::advance(&mut v, 4.0 * 3600.0);
    assert!(native_refusal(&events).is_some(), "{events:?}");
    let maltose = v.moles_of(&SpeciesId::new("maltose")).0;
    assert!(
        (maltose - 0.005).abs() < 1e-12,
        "only cooked pool: {maltose}"
    );
    assert!((v.moles_of(&SpeciesId::new("starch")).0 - 0.02).abs() < 1e-12);
    assert_eq!(starch_amylase::gelatinised_moles(&v), 0.0);
    assert!(!starch_amylase::consumes(&v, &SpeciesId::new("starch")));
}

#[test]
fn gelatinisation_constants_match_the_coverage_table() {
    let table: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/starch-amylase-coverage-20261111.json"
    ))
    .unwrap();
    let g = &table["native_granules"];
    assert_eq!(
        g["gelatinised_at_k"].as_f64().unwrap(),
        starch_amylase::GELATINISED_AT_K
    );
    assert_eq!(g["refusal"], "not-modeled.amylase-native-starch-granules");
}

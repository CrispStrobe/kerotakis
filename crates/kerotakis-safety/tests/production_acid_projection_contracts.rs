//! Synthetic reporting-projection controls; not chemistry forecasts.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::solve::{Equilibrator, SafetyScreen, SafetyVerdict, SolveError};
use kerotakis_core::{
    Bench, Event, Liters, Moles, Operator, Phase, Severity, SpeciesId, Vessel, VesselId,
};
use kerotakis_safety::ReactiveGroupScreen;
fn probe(partner: &str) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "acid-projection-probe");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.deposit(
        SpeciesId::new(partner),
        Moles(0.0001),
        if partner == "NaOCl" {
            Phase::Liquid
        } else {
            Phase::Solid
        },
    );
    v
}
fn named(partner: &str, acid: &str) -> Vessel {
    let mut v = probe(partner);
    v.deposit(SpeciesId::new(acid), Moles(0.0001), Phase::Liquid);
    v
}
fn projected(partner: &str, acidity: f64, ph: f64) -> Vessel {
    let mut v = probe(partner);
    v.solute_charge = -acidity;
    v.solution =
        Some(serde_json::from_value(serde_json::json!({"ph":ph,"ionic_strength":0.01})).unwrap());
    v
}
fn quiet(before: &Vessel, after: &Vessel) {
    assert!(matches!(
        ReactiveGroupScreen.assess_equilibrated(before, after),
        SafetyVerdict::Allow
    ));
}
fn warn(before: &Vessel, after: &Vessel, expected: &str) {
    let (level, text, context) = match expected {
        "bleach-acid-chlorine" => (Severity::Danger, "mixing bleach with acid releases chlorine, a toxic gas", "Chlorine gas was used as a chemical weapon; even small amounts damage lungs."),
        "acid-metal-hydrogen" => (Severity::Caution, "strong acid dissolves this metal, releasing hydrogen gas which is flammable", "Any metal above hydrogen in the activity series gives off hydrogen gas in acid; magnesium ribbon in hydrochloric acid is the school version, where the gas pops with a lit splint. The gas is genuinely flammable whichever metal and acid made it."),
        "acid-carbonate-co2" => (Severity::Caution, "strong acid and carbonate fizz vigorously, releasing carbon dioxide — the mixture can spatter", "Adding acid to chalk or baking soda foams over if the vessel is too small. CO₂ displaces air in enclosed spaces."),
        _ => panic!("unfrozen rule"),
    };
    match ReactiveGroupScreen.assess_equilibrated(before, after) {
        SafetyVerdict::Warn {
            severity,
            rule,
            hazard,
            real_world,
        } => {
            assert_eq!(severity, level);
            assert_eq!(rule, expected);
            assert_eq!(hazard, text);
            assert_eq!(real_world, context);
        }
        other => panic!("expected reviewed {expected} warning, got {other:?}"),
    }
}

#[test]
fn bleach_acid_new_dissolved_strong_acidity_warns() {
    warn(
        &probe("NaOCl"),
        &projected("NaOCl", 0.0001, 1.0),
        "bleach-acid-chlorine",
    );
}

#[test]
fn bleach_acid_unchanged_dissolved_acidity_stays_quiet() {
    let before = projected("NaOCl", 0.0001, 1.0);
    quiet(&before, &before.clone());
}

#[test]
fn bleach_acid_hydrochloric_reporting_projection_stays_quiet() {
    quiet(&named("NaOCl", "HCl"), &projected("NaOCl", 0.0001, 1.0));
}

#[test]
fn bleach_acid_sulfuric_reporting_projection_stays_quiet() {
    quiet(&named("NaOCl", "H2SO4"), &projected("NaOCl", 0.0001, 1.0));
}

#[test]
fn bleach_acid_at_or_below_acidity_gate_stays_quiet() {
    for amount in [5e-10, 1e-9] {
        quiet(&probe("NaOCl"), &projected("NaOCl", amount, 1.0));
    }
}

#[test]
fn acid_metal_new_dissolved_strong_acidity_warns() {
    warn(
        &probe("Mg"),
        &projected("Mg", 0.0001, 1.0),
        "acid-metal-hydrogen",
    );
}

#[test]
fn acid_metal_unchanged_dissolved_acidity_stays_quiet() {
    let before = projected("Mg", 0.0001, 1.0);
    quiet(&before, &before.clone());
}

#[test]
fn acid_metal_hydrochloric_reporting_projection_stays_quiet() {
    quiet(&named("Mg", "HCl"), &projected("Mg", 0.0001, 1.0));
}

#[test]
fn acid_metal_sulfuric_reporting_projection_stays_quiet() {
    quiet(&named("Mg", "H2SO4"), &projected("Mg", 0.0001, 1.0));
}

#[test]
fn acid_metal_at_or_below_acidity_gate_stays_quiet() {
    for amount in [5e-10, 1e-9] {
        quiet(&probe("Mg"), &projected("Mg", amount, 1.0));
    }
}

#[test]
fn acid_carbonate_new_dissolved_strong_acidity_warns() {
    warn(
        &probe("CaCO3"),
        &projected("CaCO3", 0.0001, 1.0),
        "acid-carbonate-co2",
    );
}

#[test]
fn acid_carbonate_unchanged_dissolved_acidity_stays_quiet() {
    let before = projected("CaCO3", 0.0001, 1.0);
    quiet(&before, &before.clone());
}

#[test]
fn acid_carbonate_hydrochloric_reporting_projection_stays_quiet() {
    quiet(&named("CaCO3", "HCl"), &projected("CaCO3", 0.0001, 1.0));
}

#[test]
fn acid_carbonate_sulfuric_reporting_projection_stays_quiet() {
    quiet(&named("CaCO3", "H2SO4"), &projected("CaCO3", 0.0001, 1.0));
}

#[test]
fn acid_carbonate_at_or_below_acidity_gate_stays_quiet() {
    for amount in [5e-10, 1e-9] {
        quiet(&probe("CaCO3"), &projected("CaCO3", amount, 1.0));
    }
}

#[test]
fn acid_metal_above_strong_ph_gate_stays_quiet() {
    quiet(&probe("Mg"), &projected("Mg", 0.0001, 2.0001));
}

#[test]
fn acid_metal_at_strong_ph_gate_warns() {
    warn(
        &probe("Mg"),
        &projected("Mg", 0.0001, 2.0),
        "acid-metal-hydrogen",
    );
}

#[test]
fn acid_carbonate_above_strong_ph_gate_stays_quiet() {
    quiet(&probe("CaCO3"), &projected("CaCO3", 0.0001, 2.0001));
}

#[test]
fn acid_carbonate_at_strong_ph_gate_warns() {
    warn(
        &probe("CaCO3"),
        &projected("CaCO3", 0.0001, 2.0),
        "acid-carbonate-co2",
    );
}

struct Projection {
    calls: usize,
}
impl Equilibrator for Projection {
    fn name(&self) -> &'static str {
        "synthetic-acid-reporting-projection"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        v.contents.retain(|p| p.species.0 != "HCl");
        v.solute_charge = -0.0001;
        v.solution = Some(
            serde_json::from_value(serde_json::json!({"ph":1.0,"ionic_strength":0.01})).unwrap(),
        );
        Ok(Vec::new())
    }
}
fn prospective_only(partner: &str, expected_rule: &str) {
    let mut bench = Bench::new();
    bench.vessels[0] = named(partner, "HCl");
    let mut model = Projection { calls: 0 };
    let events = bench
        .step_with(
            Operator::Titrate {
                vessel: VesselId(0),
                titrant: SpeciesId::new("NaOH"),
                concentration: 1.0,
                step: Liters(0.001),
                target_ph: 12.0,
                max_steps: 1,
                endpoint: Endpoint::Ph,
            },
            &mut model,
            &ReactiveGroupScreen,
        )
        .unwrap();
    assert_eq!(model.calls, 1);
    let warnings: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::HazardWarning { rule, .. } => Some(rule.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(warnings, [expected_rule]);
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
    assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new("HCl")).0, 0.0);
    assert_eq!(bench.vessels[0].solute_charge, -0.0001);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaOH")).0 - 0.001).abs() < 1e-12);
}

#[test]
fn bleach_acid_projection_keeps_only_prospective_warning() {
    prospective_only("NaOCl", "bleach-acid-chlorine");
}

#[test]
fn acid_metal_projection_keeps_only_prospective_warning() {
    prospective_only("Mg", "acid-metal-hydrogen");
}

#[test]
fn acid_carbonate_projection_keeps_only_prospective_warning() {
    prospective_only("CaCO3", "acid-carbonate-co2");
}

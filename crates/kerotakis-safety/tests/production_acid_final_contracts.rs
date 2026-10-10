//! Frozen source-informed engineering controls, not chemical predictions.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::solve::{Equilibrator, SafetyScreen, SafetyVerdict, SolveError};
use kerotakis_core::{
    Bench, Event, Liters, Moles, Operator, Phase, Severity, SpeciesId, Vessel, VesselId,
};
use kerotakis_safety::ReactiveGroupScreen;

fn probe(keys: &[&str]) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "acid-final-probe");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    for key in keys {
        add(&mut v, key, 0.0001);
    }
    v
}
fn add(v: &mut Vessel, key: &str, amount: f64) {
    let phase = if matches!(key, "Mg" | "Zn" | "CaCO3" | "NaHCO3" | "CaO") {
        Phase::Solid
    } else {
        Phase::Liquid
    };
    v.deposit(SpeciesId::new(key), Moles(amount), phase);
}
fn quiet(before: &Vessel, after: &Vessel) {
    assert!(matches!(
        ReactiveGroupScreen.assess_equilibrated(before, after),
        SafetyVerdict::Allow
    ));
}
fn fields(v: SafetyVerdict) -> (Severity, String, String, String) {
    match v {
        SafetyVerdict::Warn {
            severity,
            rule,
            hazard,
            real_world,
        } => (severity, rule, hazard, real_world),
        other => panic!("expected reviewed warning, got {other:?}"),
    }
}
fn warning(before: &Vessel, after: &Vessel, rule: &str) {
    let actual = fields(ReactiveGroupScreen.assess_equilibrated(before, after));
    let (severity, hazard, real_world) = match rule {
        "bleach-acid-chlorine" => (Severity::Danger, "mixing bleach with acid releases chlorine, a toxic gas", "Chlorine gas was used as a chemical weapon; even small amounts damage lungs."),
        "acid-metal-hydrogen" => (Severity::Caution, "strong acid dissolves this metal, releasing hydrogen gas which is flammable", "Any metal above hydrogen in the activity series gives off hydrogen gas in acid; magnesium ribbon in hydrochloric acid is the school version, where the gas pops with a lit splint. The gas is genuinely flammable whichever metal and acid made it."),
        "acid-carbonate-co2" => (Severity::Caution, "strong acid and carbonate fizz vigorously, releasing carbon dioxide — the mixture can spatter", "Adding acid to chalk or baking soda foams over if the vessel is too small. CO₂ displaces air in enclosed spaces."),
        _ => panic!("unfrozen rule"),
    };
    assert_eq!(
        actual,
        (
            severity,
            rule.to_string(),
            hazard.to_string(),
            real_world.to_string()
        )
    );
}

#[test]
fn bleach_acid_introduced() {
    let before = probe(&["NaOCl"]);
    let after = probe(&["NaOCl", "HCl"]);
    warning(&before, &after, "bleach-acid-chlorine");
}

#[test]
fn bleach_acid_unchanged() {
    let before = probe(&["NaOCl", "HCl"]);
    quiet(&before, &before.clone());
}

#[test]
fn bleach_acid_benign() {
    let before = probe(&["NaOCl"]);
    let after = probe(&["NaOCl", "NaCl"]);
    quiet(&before, &after);
}

#[test]
fn bleach_acid_distinct_pair() {
    let before = probe(&["NaOCl", "HCl"]);
    let mut after = before.clone();
    add(&mut after, "H2SO4", 0.0001);
    warning(&before, &after, "bleach-acid-chlorine");
}

#[test]
fn bleach_acid_behind_old_priority() {
    let before = probe(&["CaO"]);
    let mut after = before.clone();
    add(&mut after, "NaOCl", 0.0001);
    add(&mut after, "HCl", 0.0001);
    warning(&before, &after, "bleach-acid-chlorine");
}

#[test]
fn bleach_acid_at_or_below_cutoff() {
    let before = probe(&["NaOCl"]);
    for amount in [5e-13, 1e-12] {
        let mut after = before.clone();
        add(&mut after, "HCl", amount);
        quiet(&before, &after);
    }
}

#[test]
fn bleach_acid_above_cutoff() {
    let before = probe(&["NaOCl"]);
    let mut after = before.clone();
    add(&mut after, "HCl", 2e-12);
    warning(&before, &after, "bleach-acid-chlorine");
}

#[test]
fn acid_metal_introduced() {
    let before = probe(&["HCl"]);
    let after = probe(&["HCl", "Mg"]);
    warning(&before, &after, "acid-metal-hydrogen");
}

#[test]
fn acid_metal_unchanged() {
    let before = probe(&["HCl", "Mg"]);
    quiet(&before, &before.clone());
}

#[test]
fn acid_metal_benign() {
    let before = probe(&["HCl"]);
    let after = probe(&["HCl", "NaCl"]);
    quiet(&before, &after);
}

#[test]
fn acid_metal_distinct_pair() {
    let before = probe(&["HCl", "Mg"]);
    let mut after = before.clone();
    add(&mut after, "Zn", 0.0001);
    warning(&before, &after, "acid-metal-hydrogen");
}

#[test]
fn acid_metal_behind_old_priority() {
    let before = probe(&["CaO"]);
    let mut after = before.clone();
    add(&mut after, "HCl", 0.0001);
    add(&mut after, "Mg", 0.0001);
    warning(&before, &after, "acid-metal-hydrogen");
}

#[test]
fn acid_metal_at_or_below_cutoff() {
    let before = probe(&["HCl"]);
    for amount in [5e-13, 1e-12] {
        let mut after = before.clone();
        add(&mut after, "Mg", amount);
        quiet(&before, &after);
    }
}

#[test]
fn acid_metal_above_cutoff() {
    let before = probe(&["HCl"]);
    let mut after = before.clone();
    add(&mut after, "Mg", 2e-12);
    warning(&before, &after, "acid-metal-hydrogen");
}

#[test]
fn acid_carbonate_introduced() {
    let before = probe(&["HCl"]);
    let after = probe(&["HCl", "CaCO3"]);
    warning(&before, &after, "acid-carbonate-co2");
}

#[test]
fn acid_carbonate_unchanged() {
    let before = probe(&["HCl", "CaCO3"]);
    quiet(&before, &before.clone());
}

#[test]
fn acid_carbonate_benign() {
    let before = probe(&["HCl"]);
    let after = probe(&["HCl", "NaCl"]);
    quiet(&before, &after);
}

#[test]
fn acid_carbonate_distinct_pair() {
    let before = probe(&["HCl", "CaCO3"]);
    let mut after = before.clone();
    add(&mut after, "NaHCO3", 0.0001);
    warning(&before, &after, "acid-carbonate-co2");
}

#[test]
fn acid_carbonate_behind_old_priority() {
    let before = probe(&["CaO"]);
    let mut after = before.clone();
    add(&mut after, "HCl", 0.0001);
    add(&mut after, "CaCO3", 0.0001);
    warning(&before, &after, "acid-carbonate-co2");
}

#[test]
fn acid_carbonate_at_or_below_cutoff() {
    let before = probe(&["HCl"]);
    for amount in [5e-13, 1e-12] {
        let mut after = before.clone();
        add(&mut after, "CaCO3", amount);
        quiet(&before, &after);
    }
}

#[test]
fn acid_carbonate_above_cutoff() {
    let before = probe(&["HCl"]);
    let mut after = before.clone();
    add(&mut after, "CaCO3", 2e-12);
    warning(&before, &after, "acid-carbonate-co2");
}

#[test]
fn bleach_reporting_ion_keeps_existing_strong_acid_pair_quiet() {
    quiet(&probe(&["NaOCl", "HCl"]), &probe(&["ClO-", "HCl"]));
}
#[test]
fn bleach_reporting_hypochlorous_keeps_existing_strong_acid_pair_quiet() {
    quiet(&probe(&["NaOCl", "HCl"]), &probe(&["HClO", "HCl"]));
}
#[test]
fn reordered_existing_acid_metal_pair_stays_quiet() {
    let before = probe(&["HCl", "Mg"]);
    let mut after = before.clone();
    after.contents.reverse();
    quiet(&before, &after);
}

#[derive(Clone, Copy)]
enum Injection {
    Always,
    FullDoseOnly,
    None,
}
struct Model {
    pair: [&'static str; 2],
    injection: Injection,
    calls: usize,
}
impl Equilibrator for Model {
    fn name(&self) -> &'static str {
        "acid-final-injection"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        let dose = v.moles_of(&SpeciesId::new("NaOH")).0;
        if matches!(self.injection, Injection::Always)
            || matches!(self.injection, Injection::FullDoseOnly) && dose > 0.0015
        {
            for key in self.pair {
                add(v, key, 0.0001);
            }
        }
        v.solution = Some(
            serde_json::from_value(serde_json::json!({"ph":4.0+1000.0*dose,"ionic_strength":0.01}))
                .unwrap(),
        );
        Ok(Vec::new())
    }
}
fn operation(step: f64, target: f64) -> Operator {
    Operator::Titrate {
        vessel: VesselId(0),
        titrant: SpeciesId::new("NaOH"),
        concentration: 1.0,
        step: Liters(step),
        target_ph: target,
        max_steps: 1,
        endpoint: Endpoint::Ph,
    }
}
fn rules(events: &[Event]) -> Vec<&str> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::HazardWarning { rule, .. } => Some(rule.as_str()),
            _ => None,
        })
        .collect()
}
fn run(pair: [&'static str; 2], rule: &str, injection: Injection, preexisting: bool) {
    let mut bench = Bench::new();
    bench.vessels[0] = probe(if preexisting { &pair } else { &[] });
    bench.vessels[0].solution =
        Some(serde_json::from_value(serde_json::json!({"ph":4.0,"ionic_strength":0.01})).unwrap());
    let mut model = Model {
        pair,
        injection,
        calls: 0,
    };
    let refining = matches!(injection, Injection::FullDoseOnly);
    let events = bench
        .step_with(
            operation(
                if refining { 0.002 } else { 0.001 },
                if refining { 5.0 } else { 12.0 },
            ),
            &mut model,
            &ReactiveGroupScreen,
        )
        .unwrap();
    assert_eq!(model.calls, if refining { 2 } else { 1 });
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaOH")).0 - 0.001).abs() < 1e-12);
    if refining {
        assert!(rules(&events).is_empty());
        for key in pair {
            assert_eq!(bench.vessels[0].moles_of(&SpeciesId::new(key)).0, 0.0);
        }
    } else {
        assert_eq!(rules(&events), [rule]);
        for key in pair {
            assert!((bench.vessels[0].moles_of(&SpeciesId::new(key)).0 - 0.0001).abs() < 1e-12);
        }
    }
}

#[test]
fn bleach_acid_accepted_warning_journal() {
    run(
        ["NaOCl", "HCl"],
        "bleach-acid-chlorine",
        Injection::Always,
        false,
    );
}

#[test]
fn bleach_acid_discarded_full_dose_quiet() {
    run(
        ["NaOCl", "HCl"],
        "bleach-acid-chlorine",
        Injection::FullDoseOnly,
        false,
    );
}

#[test]
fn bleach_acid_preexisting_prospective_only() {
    run(
        ["NaOCl", "HCl"],
        "bleach-acid-chlorine",
        Injection::None,
        true,
    );
}

#[test]
fn acid_metal_accepted_warning_journal() {
    run(
        ["HCl", "Mg"],
        "acid-metal-hydrogen",
        Injection::Always,
        false,
    );
}

#[test]
fn acid_metal_discarded_full_dose_quiet() {
    run(
        ["HCl", "Mg"],
        "acid-metal-hydrogen",
        Injection::FullDoseOnly,
        false,
    );
}

#[test]
fn acid_metal_preexisting_prospective_only() {
    run(["HCl", "Mg"], "acid-metal-hydrogen", Injection::None, true);
}

#[test]
fn acid_carbonate_accepted_warning_journal() {
    run(
        ["HCl", "CaCO3"],
        "acid-carbonate-co2",
        Injection::Always,
        false,
    );
}

#[test]
fn acid_carbonate_discarded_full_dose_quiet() {
    run(
        ["HCl", "CaCO3"],
        "acid-carbonate-co2",
        Injection::FullDoseOnly,
        false,
    );
}

#[test]
fn acid_carbonate_preexisting_prospective_only() {
    run(
        ["HCl", "CaCO3"],
        "acid-carbonate-co2",
        Injection::None,
        true,
    );
}

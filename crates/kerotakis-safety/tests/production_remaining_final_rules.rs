//! Original prospective source-informed final-rule controls.
//! Injected inventories and pH are routing probes, not chemical predictions.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::solve::{Equilibrator, SafetyScreen, SafetyVerdict, SolveError};
use kerotakis_core::{
    Bench, Event, Headspace, Liters, Moles, Operator, Phase, Severity, SpeciesId, Vessel, VesselId,
};
use kerotakis_safety::ReactiveGroupScreen;

const RULES: [&str; 4] = [
    "bleach-weak-acid-chlorine",
    "oxidizer-flammable-gas",
    "oxidizer-reducer",
    "water-reactive-base",
];
const PAIRS: [[&str; 2]; 4] = [
    ["NaOCl", "CH3COOH"],
    ["KMnO4", "H2"],
    ["KMnO4", "Na2SO3"],
    ["CaO", "NaOH"],
];
fn metadata(case: usize) -> (Severity, &'static str, &'static str) {
    match case {
        0 => (Severity::Danger, "mixing bleach with vinegar or another weak acid releases chlorine, a toxic gas", "Bleach and vinegar is one of the commonest household poisonings there is — both are ordinary kitchen bottles, which is exactly why people mix them."),
        1 => (Severity::Danger, "a strong oxidizer in the presence of a flammable gas creates an explosion risk", "Hydrogen and chlorine mixtures can detonate when exposed to light or a spark."),
        2 => (Severity::Danger, "mixing a strong oxidizer with a reducing agent can cause a violent, potentially explosive reaction", "Permanganate and sulfite solutions react vigorously; at scale, such mixtures can detonate."),
        3 => (Severity::Caution, "this water-reactive substance already reacted with water; combining with a strong base adds further heat", "Quicklime (CaO) generates enough heat on contact with water to cause burns; adding caustic soda on top is reckless."),
        _ => panic!("unfrozen case"),
    }
}
fn vessel() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "remaining-final-rule-probe");
    v.headspace = Headspace::Sealed {
        volume: Liters(1.0),
    };
    v.refresh_pressure();
    v
}
fn add(v: &mut Vessel, key: &str, amount: f64) {
    let phase = match key {
        "water" | "CH3COOH" | "H2O2" => Phase::Liquid,
        "H2" | "methane" => Phase::Gas,
        "ClO-" | "MnO4-" => Phase::Aqueous,
        _ => Phase::Solid,
    };
    if phase == Phase::Gas {
        assert!(v.retain_gas(SpeciesId::new(key), Moles(amount)));
    } else {
        v.deposit(SpeciesId::new(key), Moles(amount), phase);
    }
}
fn pair(case: usize) -> Vessel {
    let mut v = vessel();
    for key in PAIRS[case] {
        add(&mut v, key, 1e-4);
    }
    v
}
fn allow(before: &Vessel, after: &Vessel) {
    assert!(matches!(
        ReactiveGroupScreen.assess_equilibrated(before, after),
        SafetyVerdict::Allow
    ));
}
fn warning(case: usize, verdict: SafetyVerdict) {
    let (level, words, real) = metadata(case);
    match verdict {
        SafetyVerdict::Warn {
            severity,
            rule,
            hazard,
            real_world,
        } => {
            assert_eq!(rule, RULES[case]);
            assert_eq!(severity, level);
            assert_eq!(hazard, words);
            assert_eq!(real_world, real);
        }
        other => panic!("expected {} warning, got {other:?}", RULES[case]),
    }
}
#[derive(Clone, Copy)]
enum Mode {
    Introduced,
    ReversedIntroduction,
    Unchanged,
    Reordered,
    Benign,
    BelowCutoff,
    AboveCutoff,
    OlderPriority,
}
fn direct(case: usize, mode: Mode) {
    match mode {
        Mode::Introduced => warning(
            case,
            ReactiveGroupScreen.assess_equilibrated(&vessel(), &pair(case)),
        ),
        Mode::ReversedIntroduction => {
            let mut after = pair(case);
            after.contents.reverse();
            warning(
                case,
                ReactiveGroupScreen.assess_equilibrated(&vessel(), &after),
            );
        }
        Mode::Unchanged => {
            let before = pair(case);
            allow(&before, &before.clone());
        }
        Mode::Reordered => {
            let before = pair(case);
            let mut after = before.clone();
            after.contents.reverse();
            allow(&before, &after);
        }
        Mode::Benign => {
            let before = vessel();
            let mut after = before.clone();
            add(&mut after, PAIRS[case][1], 1e-4);
            allow(&before, &after);
        }
        Mode::BelowCutoff => {
            for amount in [5e-13, 1e-12] {
                let before = vessel();
                let mut after = before.clone();
                add(&mut after, PAIRS[case][0], amount);
                add(&mut after, PAIRS[case][1], 1e-4);
                allow(&before, &after);
            }
        }
        Mode::AboveCutoff => {
            let before = vessel();
            let mut after = before.clone();
            add(&mut after, PAIRS[case][0], 2e-12);
            add(&mut after, PAIRS[case][1], 1e-4);
            warning(
                case,
                ReactiveGroupScreen.assess_equilibrated(&before, &after),
            );
        }
        Mode::OlderPriority => {
            let mut before = vessel();
            add(&mut before, "water", 1.0);
            add(&mut before, "CaO", 1e-4);
            let mut after = before.clone();
            for key in PAIRS[case] {
                add(&mut after, key, 1e-4);
            }
            warning(
                case,
                ReactiveGroupScreen.assess_equilibrated(&before, &after),
            );
        }
    }
}
fn distinct(case: usize, additional: &str) {
    let before = pair(case);
    let mut after = before.clone();
    add(&mut after, additional, 1e-4);
    warning(
        case,
        ReactiveGroupScreen.assess_equilibrated(&before, &after),
    );
}
fn alias(case: usize, old: &str, new: &str) {
    let before = pair(case);
    let mut after = before.clone();
    let index = after
        .contents
        .iter()
        .position(|p| p.species.0 == old)
        .unwrap();
    let amount = after.contents.remove(index).moles.0;
    add(&mut after, new, amount);
    allow(&before, &after);
}
#[derive(Clone, Copy)]
enum Injection {
    Always,
    FullDoseOnly,
}
struct Model {
    case: usize,
    mode: Injection,
    calls: usize,
}
impl Equilibrator for Model {
    fn name(&self) -> &'static str {
        "remaining-final-rule-routing-injection"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        self.calls += 1;
        let dose = v.moles_of(&SpeciesId::new("NaCl")).0;
        if matches!(self.mode, Injection::Always) || dose > 0.0015 {
            for key in PAIRS[self.case] {
                if self.case != 3 || key != "CaO" {
                    add(v, key, 1e-4);
                }
            }
        }
        v.solution = Some(
            serde_json::from_value(serde_json::json!({"ph":4.0+1000.0*dose,"ionic_strength":0.01}))
                .unwrap(),
        );
        v.refresh_pressure();
        Ok(Vec::new())
    }
}
fn routed(case: usize, discarded: bool) {
    let mut bench = Bench::new();
    bench.vessels[0] = vessel();
    add(&mut bench.vessels[0], "water", 1.0);
    if case == 3 {
        add(&mut bench.vessels[0], "CaO", 1e-4);
    }
    bench.vessels[0].solution =
        Some(serde_json::from_value(serde_json::json!({"ph":4.0,"ionic_strength":0.01})).unwrap());
    let mut model = Model {
        case,
        mode: if discarded {
            Injection::FullDoseOnly
        } else {
            Injection::Always
        },
        calls: 0,
    };
    let events = bench
        .step_with(
            Operator::Titrate {
                vessel: VesselId(0),
                titrant: SpeciesId::new("NaCl"),
                concentration: 1.0,
                step: Liters(if discarded { 0.002 } else { 0.001 }),
                target_ph: if discarded { 5.0 } else { 12.0 },
                max_steps: 1,
                endpoint: Endpoint::Ph,
            },
            &mut model,
            &ReactiveGroupScreen,
        )
        .unwrap();
    assert_eq!(model.calls, if discarded { 2 } else { 1 });
    let actual: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::HazardWarning { rule, .. } => Some(rule.as_str()),
            _ => None,
        })
        .collect();
    let mut expected = Vec::new();
    if case == 3 {
        expected.push("water-reactive-slaking");
    }
    if !discarded {
        expected.push(RULES[case]);
    }
    assert_eq!(actual, expected);
    assert_eq!(bench.log.len(), 1);
    assert_eq!(bench.log[0].events, events);
    assert!((bench.vessels[0].moles_of(&SpeciesId::new("NaCl")).0 - 0.001).abs() < 1e-12);
    for key in PAIRS[case] {
        let wanted = if discarded && !(case == 3 && key == "CaO") {
            0.0
        } else {
            1e-4
        };
        let actual = bench.vessels[0].moles_of(&SpeciesId::new(key)).0;
        if wanted == 0.0 {
            assert_eq!(actual, wanted, "discarded pair member survived: {key}");
        } else {
            assert!(
                (actual - wanted).abs() < 1e-12,
                "accepted pair inventory changed: {key}"
            );
        }
    }
    if discarded {
        assert_eq!(
            bench.vessels[0].moles_of(&SpeciesId::new(PAIRS[case][1])).0,
            0.0
        );
    } else {
        assert!(
            (bench.vessels[0].moles_of(&SpeciesId::new(PAIRS[case][1])).0 - 1e-4).abs() < 1e-12
        );
        let (level, words, real) = metadata(case);
        let warnings: Vec<_> = events
            .iter()
            .filter(|e| matches!(e,Event::HazardWarning {rule,..} if rule==RULES[case]))
            .collect();
        assert_eq!(warnings.len(), 1);
        match warnings[0] {
            Event::HazardWarning {
                severity,
                hazard,
                real_world,
                ..
            } => {
                assert_eq!(*severity, level);
                assert_eq!(hazard, words);
                assert_eq!(real_world, real);
            }
            _ => unreachable!(),
        }
    }
    if case == 1 {
        assert!(bench.vessels[0].pressure.0.is_finite());
        assert_eq!(
            bench.vessels[0].gas_moles().0,
            if discarded { 0.0 } else { 1e-4 }
        );
    }
}

#[test]
fn weak_acid_introduced() {
    direct(0, Mode::Introduced);
}

#[test]
fn weak_acid_reverse_introduction() {
    direct(0, Mode::ReversedIntroduction);
}

#[test]
fn weak_acid_unchanged() {
    direct(0, Mode::Unchanged);
}

#[test]
fn weak_acid_reordered() {
    direct(0, Mode::Reordered);
}

#[test]
fn weak_acid_benign() {
    direct(0, Mode::Benign);
}

#[test]
fn weak_acid_below_portion_cutoff() {
    direct(0, Mode::BelowCutoff);
}

#[test]
fn weak_acid_above_portion_cutoff() {
    direct(0, Mode::AboveCutoff);
}

#[test]
fn weak_acid_older_priority() {
    direct(0, Mode::OlderPriority);
}

#[test]
fn flammable_gas_introduced() {
    direct(1, Mode::Introduced);
}

#[test]
fn flammable_gas_reverse_introduction() {
    direct(1, Mode::ReversedIntroduction);
}

#[test]
fn flammable_gas_unchanged() {
    direct(1, Mode::Unchanged);
}

#[test]
fn flammable_gas_reordered() {
    direct(1, Mode::Reordered);
}

#[test]
fn flammable_gas_benign() {
    direct(1, Mode::Benign);
}

#[test]
fn flammable_gas_below_portion_cutoff() {
    direct(1, Mode::BelowCutoff);
}

#[test]
fn flammable_gas_above_portion_cutoff() {
    direct(1, Mode::AboveCutoff);
}

#[test]
fn flammable_gas_older_priority() {
    direct(1, Mode::OlderPriority);
}

#[test]
fn reducer_introduced() {
    direct(2, Mode::Introduced);
}

#[test]
fn reducer_reverse_introduction() {
    direct(2, Mode::ReversedIntroduction);
}

#[test]
fn reducer_unchanged() {
    direct(2, Mode::Unchanged);
}

#[test]
fn reducer_reordered() {
    direct(2, Mode::Reordered);
}

#[test]
fn reducer_benign() {
    direct(2, Mode::Benign);
}

#[test]
fn reducer_below_portion_cutoff() {
    direct(2, Mode::BelowCutoff);
}

#[test]
fn reducer_above_portion_cutoff() {
    direct(2, Mode::AboveCutoff);
}

#[test]
fn reducer_older_priority() {
    direct(2, Mode::OlderPriority);
}

#[test]
fn reactive_base_introduced() {
    direct(3, Mode::Introduced);
}

#[test]
fn reactive_base_reverse_introduction() {
    direct(3, Mode::ReversedIntroduction);
}

#[test]
fn reactive_base_unchanged() {
    direct(3, Mode::Unchanged);
}

#[test]
fn reactive_base_reordered() {
    direct(3, Mode::Reordered);
}

#[test]
fn reactive_base_benign() {
    direct(3, Mode::Benign);
}

#[test]
fn reactive_base_below_portion_cutoff() {
    direct(3, Mode::BelowCutoff);
}

#[test]
fn reactive_base_above_portion_cutoff() {
    direct(3, Mode::AboveCutoff);
}

#[test]
fn reactive_base_older_priority() {
    direct(3, Mode::OlderPriority);
}

#[test]
fn gas_distinct_fuel() {
    distinct(1, "methane");
}

#[test]
fn gas_distinct_oxidizer() {
    distinct(1, "H2O2");
}

#[test]
fn reducer_distinct_reducer() {
    distinct(2, "KI");
}

#[test]
fn reducer_distinct_oxidizer() {
    distinct(2, "KIO3");
}

#[test]
fn base_distinct_base() {
    distinct(3, "KOH");
}

#[test]
fn weak_hypochlorite_alias() {
    alias(0, "NaOCl", "ClO-");
}

#[test]
fn gas_permanganate_alias() {
    alias(1, "KMnO4", "MnO4-");
}

#[test]
fn reducer_permanganate_alias() {
    alias(2, "KMnO4", "MnO4-");
}

#[test]
fn weak_acidity_at_or_below_cutoff() {
    for amount in [5e-10, 1e-9] {
        let before = vessel();
        let mut after = before.clone();
        add(&mut after, "NaOCl", 1e-4);
        add(&mut after, "CH3COOH", amount);
        allow(&before, &after);
    }
}

#[test]
fn weak_acidity_above_cutoff() {
    let before = vessel();
    let mut after = before.clone();
    add(&mut after, "NaOCl", 1e-4);
    add(&mut after, "CH3COOH", 2e-9);
    warning(0, ReactiveGroupScreen.assess_equilibrated(&before, &after));
}

#[test]
fn weak_acid_accepted_journal() {
    routed(0, false);
}

#[test]
fn weak_acid_discarded_refinement() {
    routed(0, true);
}

#[test]
fn flammable_gas_accepted_journal() {
    routed(1, false);
}

#[test]
fn flammable_gas_discarded_refinement() {
    routed(1, true);
}

#[test]
fn reducer_accepted_journal() {
    routed(2, false);
}

#[test]
fn reducer_discarded_refinement() {
    routed(2, true);
}

#[test]
fn reactive_base_accepted_journal() {
    routed(3, false);
}

#[test]
fn reactive_base_discarded_refinement() {
    routed(3, true);
}

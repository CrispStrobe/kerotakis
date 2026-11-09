//! The Pitzer route refuses outside each electrolyte's stated molality
//! range instead of extrapolating (docs/activity-model-range-coverage-20261109.md).
//!
//! Production defect: `add v1 HCl 10mL 37%` then `measure v1 ph` routed to
//! pitzer.dat and reported pH -5.37 at I = 16.1 mol/kgw, where the HCl
//! parameters are fitted to about 6 mol/kg and the measured mean activity
//! coefficient (Hamer & Wu 1972) implies about -2.8.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const KEY: &str = "not-modeled.activity-model-outside-range";

fn stack() -> SolverStack {
    kerotakis_stack::standard_stack(vec![Box::new(PhreeqcEquilibrator::new().unwrap())])
}

fn run(lines: &[&str]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut solver = stack();
    let mut last = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let op = script::parse_op(line).unwrap().expect(line);
        last = bench.step_with(op, &mut solver, &PermissiveScreen).unwrap();
        if i == 0 {
            // Hold the temperature from here on: these are activity
            // questions, and the dissolution heat of a mole of NaOH would
            // otherwise boil the beaker and answer a different one.
            bench.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        }
    }
    (bench, last)
}

fn refusal(events: &[Event]) -> Option<&Phrase> {
    events.iter().find_map(|e| match e {
        Event::NotYetModeled {
            reason: Some(reason),
            ..
        } if reason.key == KEY => Some(reason),
        _ => None,
    })
}

fn measured_ph(events: &[Event]) -> Option<f64> {
    events.iter().find_map(|e| match e {
        Event::Measured { value, unit, .. } if unit == "pH" => Some(*value),
        _ => None,
    })
}

#[test]
fn concentrated_hydrochloric_acid_refuses_a_ph_and_keeps_its_inventory() {
    let (bench, events) = run(&["add v1 HCl 10mL 37%", "measure v1 ph"]);
    assert_eq!(measured_ph(&events), None, "no extrapolated pH: {events:?}");
    let reason = refusal(&events).expect("the pH meter states the range refusal");
    let en = reason.render(Locale::EN);
    assert!(
        en.contains("HCl") && en.contains("fitted up to about 6"),
        "{en}"
    );
    for lang in ["de", "fr"] {
        let text = reason.render(Locale::parse(lang));
        assert_ne!(text, en, "{lang} translated");
        assert!(text.contains("HCl"), "{lang}: {text}");
    }
    let v = &bench.vessels[0];
    assert!(v.solution.is_none(), "activity-derived outputs withdrawn");
    assert!(
        v.contents
            .iter()
            .any(|p| p.species.0.contains("Cl") && p.moles.0 > 0.1),
        "chloride inventory kept: {:?}",
        v.contents
    );
}

#[test]
fn one_molal_hcl_still_reports_its_ph() {
    let (bench, events) = run(&["add v1 water 100mL", "add v1 HCl 0.1mol", "measure v1 ph"]);
    let ph = measured_ph(&events).expect("1 m HCl is inside the 6 m range");
    assert!((-0.2..=0.2).contains(&ph), "pH {ph}");
    assert!(refusal(&events).is_none());
    assert!(bench.vessels[0].activity_range_refusal.is_none());
}

#[test]
fn saturated_brine_inside_the_nacl_range_still_answers() {
    let (bench, events) = run(&["add v1 water 100mL", "add v1 NaCl 40g", "measure v1 ph"]);
    assert!(measured_ph(&events).is_some(), "{events:?}");
    assert!(refusal(&events).is_none());
    let info = bench.vessels[0].solution.as_ref().unwrap();
    assert!(info
        .provenance
        .as_ref()
        .unwrap()
        .dataset
        .contains("pitzer.dat"));
}

#[test]
fn hot_saturated_brine_answers_inside_its_saturation_scope() {
    // Halite saturates near 6.6 mol/kg at 90 C, above the 25 C figure; the
    // NaCl row is valid to saturation 0-300 C (Pitzer, Peiper & Busey 1984).
    let mut bench = Bench::new();
    let mut solver = stack();
    for line in ["add v1 water 100mL", "add v1 NaCl 45g", "measure v1 ph"] {
        if line.starts_with("add v1 NaCl") {
            bench.vessels[0].thermal_mode = ThermalMode::Thermostatted(Kelvin(363.15));
        }
        let op = script::parse_op(line).unwrap().expect(line);
        let events = bench.step_with(op, &mut solver, &PermissiveScreen).unwrap();
        assert!(refusal(&events).is_none(), "{line}: {events:?}");
        if line.starts_with("measure") {
            assert!(
                measured_ph(&events).is_some_and(f64::is_finite),
                "{events:?}"
            );
        }
    }
    assert!(bench.vessels[0].activity_range_refusal.is_none());
}

#[test]
fn ten_molal_sodium_hydroxide_is_outside_its_six_molal_range() {
    let (_, events) = run(&["add v1 water 100mL", "add v1 NaOH 1mol", "measure v1 ph"]);
    assert_eq!(measured_ph(&events), None, "{events:?}");
    let en = refusal(&events)
        .expect("NaOH range refusal")
        .render(Locale::EN);
    assert!(en.contains("NaOH") && en.contains("about 6 mol/kg"), "{en}");
}

#[test]
fn activity_range_coverage_is_current() {
    let doc: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/activity-model-range-coverage-20261109.json"
    ))
    .unwrap();
    let rows: Vec<&serde_json::Value> = doc["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["class"] != "absent")
        .collect();
    let table = kerotakis_phreeqc::activity_range::table();
    assert_eq!(
        rows.len(),
        table.electrolytes.len(),
        "regenerate the coverage table"
    );
    for (json, row) in rows.iter().zip(&table.electrolytes) {
        assert_eq!(json["electrolyte"], row.electrolyte.as_str());
        assert_eq!(
            json["max_molality"].as_f64(),
            row.max_molality,
            "{}",
            row.electrolyte
        );
        assert_eq!(json["source"], row.source.as_str());
        assert_eq!(
            json["saturation_scope"]["phase"].as_str(),
            row.saturation_scope.as_ref().map(|s| s.phase.as_str()),
            "{}",
            row.electrolyte
        );
    }
}

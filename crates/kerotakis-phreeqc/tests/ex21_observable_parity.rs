#![cfg(all(
    feature = "engine",
    any(feature = "legacy-basic-oracle", feature = "my-basic")
))]

use kerotakis_phreeqc::Phreeqc;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DifferenceClass {
    AdapterBugFixed,
}

const EX21_DIFFERENCE_CLASS: DifferenceClass = DifferenceClass::AdapterBugFixed;

struct NumericFixture {
    name: String,
    expected: f64,
    absolute_tolerance: f64,
    relative_tolerance: f64,
    explanation: &'static str,
}

fn fields(line: &str) -> Vec<&str> {
    line.split(|character: char| character.is_whitespace() || character == ';')
        .filter(|field| !field.is_empty())
        .collect()
}

fn generated_observables(generated: &str) -> BTreeMap<String, f64> {
    let mut values = BTreeMap::new();
    for line in generated.lines() {
        let fields = fields(line);
        if fields.is_empty() {
            continue;
        }
        if fields[0].eq_ignore_ascii_case("SOLUTION") && fields.len() >= 4 {
            let solution: usize = fields[1].parse().expect("generated SOLUTION number");
            let water = fields
                .iter()
                .position(|field| field.eq_ignore_ascii_case("-water"))
                .and_then(|index| fields.get(index + 1))
                .and_then(|field| field.parse::<f64>().ok());
            if let Some(water) = water {
                values.insert(format!("solution_{solution}_water_kg"), water);
            }
        } else if fields[0].eq_ignore_ascii_case("MIX") && fields.len() >= 4 {
            let mix: usize = fields[1].parse().expect("generated MIX number");
            let factor: f64 = fields
                .last()
                .unwrap()
                .parse()
                .expect("generated MIX factor");
            values.insert(format!("mix_{mix}_factor"), factor);
        } else if fields[0].eq_ignore_ascii_case("-time") && fields.len() == 2 {
            values.insert(
                "transport_time_s".into(),
                fields[1].parse().expect("generated transport time"),
            );
        } else if fields[0].eq_ignore_ascii_case("-shifts") && fields.len() == 2 {
            values.insert(
                "transport_shifts".into(),
                fields[1].parse().expect("generated transport shifts"),
            );
        }
    }
    values
}

fn fixtures() -> Vec<NumericFixture> {
    let waters = [
        0.0013963051330271184,
        0.00007269549170333492,
        0.00008942283188987075,
        0.00010615017207640658,
        0.00012287751226294237,
        0.00013960485244947818,
        0.00015633219263601407,
        0.00017305953282254955,
        0.00018978687300908535,
        0.0002065142131956213,
        0.0002232415533821569,
        0.00023996889356869258,
        0.005026556287900859,
        0.2,
    ];
    let mixes = [
        0.0006693208027942003,
        0.00019357408729591639,
        0.00015438972867857676,
        0.00018624953891685327,
        0.00021810934915512974,
        0.00024996915939340625,
        0.0002818289696316827,
        0.00031368877986995915,
        0.00034554859010823565,
        0.00037740840034651205,
        0.00040926821058478855,
        0.000441128020823065,
        0.0007650914555608387,
        0.004253267653739294,
    ];
    let mut result = Vec::new();
    for (index, expected) in waters.into_iter().enumerate() {
        result.push(NumericFixture {
            name: format!("solution_{}_water_kg", index + 4),
            expected,
            absolute_tolerance: 1e-12,
            relative_tolerance: 5e-5,
            explanation:
                "the legacy oracle renders water masses with five significant digits; half a last-place unit is formatting-only",
        });
    }
    for (index, expected) in mixes.into_iter().enumerate() {
        result.push(NumericFixture {
            name: format!("mix_{}_factor", index + 3),
            expected,
            absolute_tolerance: 1e-12,
            relative_tolerance: 5e-5,
            explanation: "the legacy oracle renders mixing factors with five significant digits; half a last-place unit is formatting-only",
        });
    }
    result.push(NumericFixture {
        name: "transport_time_s".into(),
        expected: 1542.857142857143,
        absolute_tolerance: 5e-2,
        relative_tolerance: 0.0,
        explanation: "legacy ex21 renders time with five significant digits (1542.9), so half of the last displayed decimal place is formatting-only",
    });
    result.push(NumericFixture {
        name: "transport_shifts".into(),
        expected: 1120.0,
        absolute_tolerance: 0.0,
        relative_tolerance: 0.0,
        explanation: "the generated shift count is integral and must agree exactly",
    });
    result
}

#[test]
fn ex21_generator_matches_legacy_observables_with_declared_tolerances() {
    assert_eq!(EX21_DIFFERENCE_CLASS, DifferenceClass::AdapterBugFixed);
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/iphreeqc");
    let database = fs::read(vendor.join("database/phreeqc.dat")).unwrap();
    let input = fs::read_to_string(vendor.join("phreeqc3-examples/ex21")).unwrap();
    let generator = input
        .split_once("INCLUDE$ radial")
        .expect("ex21 generated-input include marker")
        .0;
    let mut engine = Phreeqc::with_database(&database).unwrap();
    engine.run(generator).unwrap();

    let generated = engine.selected_output_string();
    let actual = generated_observables(&generated);
    assert_eq!(
        actual.len(),
        30,
        "ex21 generated structure is exact (14 solutions, 14 MIX entries, time, shifts); keys={:?}",
        actual.keys().collect::<Vec<_>>()
    );
    for fixture in fixtures() {
        let actual = actual
            .get(&fixture.name)
            .unwrap_or_else(|| panic!("missing ex21 observable {}", fixture.name));
        let tolerance =
            fixture.absolute_tolerance + fixture.relative_tolerance * fixture.expected.abs();
        assert!(
            (actual - fixture.expected).abs() <= tolerance,
            "{}: actual={}, expected={}, abs_tol={}, rel_tol={}; {}",
            fixture.name,
            actual,
            fixture.expected,
            fixture.absolute_tolerance,
            fixture.relative_tolerance,
            fixture.explanation
        );
    }
}

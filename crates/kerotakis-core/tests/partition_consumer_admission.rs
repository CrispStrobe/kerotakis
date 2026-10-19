//! App consumers must preserve the checked model boundary and use the declared
//! molecular decomposition. Oracle rows are an independent implementation of
//! the same published parameters, not experimental calibration.
use kerotakis_core::*;

fn sample() -> Bench {
    let mut bench = Bench::new();
    let vessel = &mut bench.vessels[0];
    vessel.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("ethanol"), Moles(0.01), Phase::Aqueous);
    bench
}

fn op(text: &str) -> Operator {
    script::parse_op(text).unwrap().unwrap()
}

#[test]
fn invalid_column_coordinates_refuse_without_panicking_or_mutating_inventory() {
    // Zero is the disclosed COOL floor; tiny positive T exercises numerical range.
    for temperature in [0.0, 1e-300] {
        let mut bench = sample();
        bench.vessels[0].temperature = Kelvin(temperature);
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let bits = bench.vessels[0].temperature.0.to_bits();
        let events = bench.step(op("chromatograph v1")).unwrap();
        assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
        assert_eq!(bench.vessels[0].temperature.0.to_bits(), bits);
        assert!(!events
            .iter()
            .any(|event| matches!(event, Event::Chromatographed { .. })));
        let phrase = events
            .iter()
            .find_map(|event| match event {
                Event::NotYetModeled {
                    reason: Some(reason),
                    ..
                } if reason.key == "not-modeled.column-partition-refused" => Some(reason),
                _ => None,
            })
            .expect("the refused coefficient must be explicit");
        for locale in [Locale::DE, Locale::FR] {
            assert_ne!(phrase.render(locale), phrase.render(Locale::EN));
            assert!(!phrase
                .render(locale)
                .contains("cannot calculate a finite partition coefficient"));
        }
    }
}

#[test]
fn globally_invalid_temperatures_keep_the_bench_error_and_rollback_contract() {
    for temperature in [-1.0, f64::NAN, f64::INFINITY] {
        let mut bench = sample();
        bench.vessels[0].temperature = Kelvin(temperature);
        let before = serde_json::to_value(&bench.vessels).unwrap();
        let bits = bench.vessels[0].temperature.0.to_bits();
        assert!(matches!(
            bench.step(op("chromatograph v1")),
            Err(BenchError::InvalidState(_))
        ));
        assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
        assert_eq!(bench.vessels[0].temperature.0.to_bits(), bits);
    }
}

#[test]
fn invalid_extraction_coordinate_refuses_without_panicking_or_moving_matter() {
    let mut bench = sample();
    bench.step(op("new")).unwrap();
    bench.vessels[0].temperature = Kelvin(0.0);
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let events = bench
        .step(op("extract v1 v2 hexane 0.5mol stages 2"))
        .unwrap();
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(!events
        .iter()
        .any(|event| matches!(event, Event::Extracted { .. })));
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { .. })));
}

fn reference_gamma(solvent: &str, temperature: f64) -> f64 {
    include_str!("../../kerotakis-thermo/tests/fixtures/thermo_oracle.tsv")
        .lines()
        .find_map(|line| {
            let row: Vec<_> = line.split('\t').collect();
            if row.len() == 5
                && row[0] == "gammainf"
                && row[1] == "methanol"
                && row[2] == solvent
                && row[3].parse::<f64>().ok() == Some(temperature)
            {
                row[4].parse().ok()
            } else {
                None
            }
        })
        .expect("unchanged independent implementation row")
}

#[test]
fn methanol_column_uses_the_dedicated_subgroup_reference_at_both_temperatures() {
    for temperature in [298.15, 310.0] {
        let mut bench = Bench::new();
        bench.vessels[0].deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
        bench.vessels[0].deposit(SpeciesId::new("methanol"), Moles(0.01), Phase::Liquid);
        bench.vessels[0].temperature = Kelvin(temperature);
        let events = bench.step(op("chromatograph v1")).unwrap();
        let actual = events
            .iter()
            .find_map(|event| match event {
                Event::Chromatographed { peaks, .. } => peaks
                    .iter()
                    .find(|peak| peak.species.0 == "methanol")
                    .map(|peak| peak.partition_k),
                _ => None,
            })
            .expect("methanol has a modeled peak");
        let expected =
            reference_gamma("water", temperature) / reference_gamma("hexane", temperature);
        assert!(
            ((actual - expected) / expected).abs() < 1e-4,
            "T={temperature}: column={actual}, independent implementation={expected}"
        );
    }
}

#[test]
fn a_partial_sample_keeps_its_supported_peak_and_missing_data_note() {
    let mut bench = sample();
    bench.vessels[0].deposit(SpeciesId::new("ethyl_acetate"), Moles(0.01), Phase::Liquid);
    let before = serde_json::to_value(&bench.vessels).unwrap();
    let events = bench.step(op("chromatograph v1")).unwrap();
    assert_eq!(serde_json::to_value(&bench.vessels).unwrap(), before);
    assert!(events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { .. })));
    let peaks = events
        .iter()
        .find_map(|event| match event {
            Event::Chromatographed { peaks, .. } => Some(peaks),
            _ => None,
        })
        .unwrap();
    assert_eq!(peaks.len(), 1);
    assert_eq!(peaks[0].species.0, "ethanol");
}

#[cfg(test)]
mod numerical_water_activity_tests {
    use super::numerical_species_distribution;

    const REPORT: &str = "Distribution of species\nH2O 5.551e+01 9.500e-01 1.744 -0.022 0.0\nNa+ 1.000e+00 0.7000 0 -0.15 -0.15\n-----\n";

    fn rows(value: &str) -> Vec<Vec<String>> {
        vec![vec!["la_H2O".into()], vec![value.into()]]
    }

    #[test]
    fn numerical_activity_keeps_distinct_states_with_the_same_rounded_report() {
        for activity in [0.950001234567_f64, 0.950004567891_f64] {
            let precise =
                numerical_species_distribution(REPORT, &rows(&activity.log10().to_string()))
                    .expect("numerical water activity");
            let water = precise.iter().find(|s| s.name == "H2O").unwrap();
            assert!((water.activity - activity).abs() < 2e-15);
            assert_eq!(water.molality, 55.51);
            assert_eq!(
                precise.iter().find(|s| s.name == "Na+").unwrap().activity,
                0.7
            );
        }
    }

    #[test]
    fn numerical_activity_uses_the_final_native_solution_row() {
        let mut selected = rows("-0.1");
        selected.push(vec!["-0.2".into()]);
        let result = numerical_species_distribution(REPORT, &selected).unwrap();
        assert!(
            (result.iter().find(|s| s.name == "H2O").unwrap().activity - 10.0_f64.powf(-0.2)).abs()
                < 2e-15
        );
    }

    #[test]
    fn cold_and_icy_inputs_preserve_both_source_solvent_budgets_and_temperature() {
        use kerotakis_core::{Kelvin, Moles, Phase, SpeciesId, Vessel, VesselId};
        for (temperature, ice) in [(260.12345678912345, false), (285.12345678912345, true)] {
            let mut cold = Vessel::new(VesselId(0), "phase-sensitive input");
            cold.temperature = Kelvin(temperature);
            cold.deposit(
                SpeciesId::new("water"),
                Moles(1.2345678912345678),
                Phase::Liquid,
            );
            cold.deposit(SpeciesId::new("NaCl"), Moles(0.017), Phase::Solid);
            if ice {
                cold.deposit(SpeciesId::new("water"), Moles(0.1), Phase::Solid);
            }
            let mut warm = Vessel::new(VesselId(1), "warm source");
            warm.temperature = Kelvin(298.1500375477956);
            warm.deposit(
                SpeciesId::new("water"),
                Moles(2.345678912345678),
                Phase::Liquid,
            );
            warm.deposit(SpeciesId::new("NaCl"), Moles(0.011), Phase::Solid);
            let pa = super::partition(&cold).unwrap();
            let pb = super::partition(&warm).unwrap();
            let direct = super::build_input(&cold, &pa, "wateq4f");
            let fields = |input: &str, field: &str| -> Vec<f64> {
                input
                    .lines()
                    .filter_map(|line| line.trim().strip_prefix(field))
                    .map(|value| value.trim().parse().unwrap())
                    .collect()
            };
            assert_eq!(
                fields(&direct, "temp "),
                vec![cold.temperature.to_celsius()]
            );
            assert_eq!(fields(&direct, "water "), vec![pa.kgw]);
            let mut combined = cold.clone();
            for portion in &warm.contents {
                combined.deposit(portion.species.clone(), portion.moles, portion.phase);
            }
            let merged = super::partition(&combined).unwrap();
            let mixed =
                super::build_mix_input(&cold, &pa, &warm, &pb, 1.0, 1.0, "wateq4f", &merged, false);
            assert_eq!(
                fields(&mixed, "temp "),
                vec![cold.temperature.to_celsius(), warm.temperature.to_celsius()]
            );
            assert_eq!(fields(&mixed, "water "), vec![pa.kgw, pb.kgw]);
            assert!(super::phase_input_precision(&cold));
            assert!(!super::phase_input_precision(&warm));
            // Receiver-only sensitivity must preserve both warm inputs too.
            let mut warm_combined = warm.clone();
            for portion in &warm.contents {
                warm_combined.deposit(portion.species.clone(), portion.moles, portion.phase);
            }
            let warm_merged = super::partition(&warm_combined).unwrap();
            let receiver_cold = super::build_mix_input(
                &warm,
                &pb,
                &warm,
                &pb,
                1.0,
                1.0,
                "wateq4f",
                &warm_merged,
                true,
            );
            assert_eq!(
                fields(&receiver_cold, "temp "),
                vec![warm.temperature.to_celsius(); 2]
            );
            assert_eq!(fields(&receiver_cold, "water "), vec![pb.kgw; 2]);
            let ordinary = super::build_mix_input(
                &warm,
                &pb,
                &warm,
                &pb,
                1.0,
                1.0,
                "wateq4f",
                &warm_merged,
                false,
            );
            assert_eq!(
                fields(&ordinary, "temp "),
                vec![
                    format!("{:.4}", warm.temperature.to_celsius())
                        .parse::<f64>()
                        .unwrap();
                    2
                ]
            );
            assert_eq!(
                fields(&ordinary, "water "),
                vec![format!("{:.9}", pb.kgw).parse::<f64>().unwrap(); 2]
            );
        }
    }

    fn cache_entry(value: &str) -> super::CacheEntry {
        super::CacheEntry {
            key: "test-current-input\n    -activities H2O\n".into(),
            rows: rows(value),
            species: super::parse_species_distribution(REPORT),
            saturation: vec![],
            redox_adjusted: false,
            pe_undetermined: false,
        }
    }

    #[test]
    fn imported_current_cache_normalizes_water_from_numerical_rows() {
        let mut solver = super::PhreeqcEquilibrator::new().unwrap();
        let activity = 0.950001234567_f64;
        let data = super::CacheData {
            entries: vec![cache_entry(&activity.log10().to_string())],
            ..Default::default()
        };
        assert_eq!(solver.import_cache(data), 1);
        let exported = solver.export_cache();
        let water = exported.entries[0]
            .species
            .iter()
            .find(|s| s.name == "H2O")
            .unwrap();
        assert!((water.activity - activity).abs() < 2e-15);
    }

    #[test]
    fn imported_current_cache_rejects_missing_or_invalid_numerical_activity() {
        let mut solver = super::PhreeqcEquilibrator::new().unwrap();
        for value in ["NaN", "-999", "999"] {
            assert_eq!(
                solver.import_cache(super::CacheData {
                    entries: vec![cache_entry(value)],
                    ..Default::default()
                }),
                0
            );
        }
        let mut entry = cache_entry("0");
        entry.rows = vec![];
        assert_eq!(
            solver.import_cache(super::CacheData {
                entries: vec![entry],
                ..Default::default()
            }),
            0
        );
    }

    #[test]
    fn missing_or_invalid_numerical_activity_refuses_instead_of_using_the_report() {
        for value in ["NaN", "inf", "-inf", "invalid", "999", "-999"] {
            assert!(
                numerical_species_distribution(REPORT, &rows(value)).is_err(),
                "{value}"
            );
        }
        for selected in [
            vec![],
            vec![vec!["la_H2O".into()]],
            vec![vec!["pH".into()], vec!["7".into()]],
            vec![vec!["la_H2O".into()], vec![]],
            vec![
                vec!["la_H2O".into(), "la_H2O".into()],
                vec!["0".into(), "0".into()],
            ],
        ] {
            assert!(numerical_species_distribution(REPORT, &selected).is_err());
        }
        assert!(numerical_species_distribution("", &rows("0")).is_err());
    }
}


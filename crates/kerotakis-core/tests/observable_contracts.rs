use kerotakis_core::instrument::{
    Calorimeter, ConductivityMeter, InstrumentContract, MeltingPointApparatus, PhMeter,
    Thermometer, TransitionRead,
};
use kerotakis_core::measurement::{Calibration, MeasurementDevice, MeasurementRecord};
use kerotakis_core::*;

fn solution(ionic_strength: f64, mut species: serde_json::Value, scope: &str) -> SolutionInfo {
    for entry in species.as_array_mut().unwrap() {
        entry["activity"] = entry["molality"].clone();
    }
    serde_json::from_value(serde_json::json!({
        "ph": 7.0, "ionic_strength": ionic_strength, "species": species, "scope": scope
    }))
    .unwrap()
}
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "sample");
    v.deposit(SpeciesId::new("water"), Moles(5.0), Phase::Liquid);
    v
}

#[test]
fn incomplete_heat_is_local_to_observables_and_thermostat_does_not_price_it() {
    let mut v = water();
    v.unpriced_heat.push(SpeciesId::new("chalcanthite"));
    for (name, expected) in [
        ("temperature", ObservableStatus::Incomplete),
        ("enthalpy", ObservableStatus::Incomplete),
        ("mass", ObservableStatus::Computed),
    ] {
        assert_eq!(observable_support(&v, name).status, expected, "{name}");
    }
    v.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    assert_eq!(
        observable_support(&v, "temperature").status,
        ObservableStatus::Computed
    );
    assert_eq!(
        observable_support(&v, "enthalpy").status,
        ObservableStatus::Incomplete
    );
    assert_eq!(v.unpriced_heat.len(), 1);
}

#[test]
fn conductivity_domain_and_coverage_are_not_silently_upgraded_by_calibration() {
    for (strength, distribution, scope, expected, reason) in [
        (
            0.001,
            serde_json::json!([{"name":"Na+", "molality":0.001},{"name":"Cl-", "molality":0.001}]),
            "complete",
            ObservableStatus::Computed,
            None,
        ),
        (
            0.5,
            serde_json::json!([{"name":"Na+", "molality":0.5},{"name":"Cl-", "molality":0.5}]),
            "complete",
            ObservableStatus::Estimated,
            Some("outside-dilute-conductivity-domain"),
        ),
        (
            0.001,
            serde_json::json!([]),
            "complete",
            ObservableStatus::Estimated,
            Some("mean-mobility-without-speciation"),
        ),
        (
            0.001,
            serde_json::json!([{"name":"Cs+", "molality":0.001}]),
            "complete",
            ObservableStatus::Incomplete,
            Some("missing-ion-mobility"),
        ),
        (
            0.001,
            serde_json::json!([]),
            "solvent_only",
            ObservableStatus::Incomplete,
            Some("uncovered-solutes"),
        ),
    ] {
        let mut v = water();
        v.solution = Some(solution(strength, distribution, scope));
        let mut device = MeasurementDevice::new(ConductivityMeter, 7)
            .unwrap()
            .with_calibration(
                Calibration::two_point(0.0, 1000.0, 0.0, 1000.0, "reference").unwrap(),
            )
            .unwrap();
        let record = device.record(&v, 0.0).unwrap();
        let support = record.model_support.unwrap();
        assert_eq!(support.status, expected);
        if let Some(reason) = reason {
            assert!(support.reasons.iter().any(|s| s == reason), "{support:?}");
        }
        assert_eq!(record.calibration_id.as_deref(), Some("reference"));
    }
}

#[test]
fn nonreference_temperature_is_not_a_calibrated_conductivity_reading() {
    let mut v = water();
    v.solution = Some(solution(
        0.001,
        serde_json::json!([{"name":"Na+","molality":0.001}]),
        "complete",
    ));
    for (kelvin, expected) in [(298.15, true), (323.15, false), (273.15, false)] {
        v.temperature = Kelvin(kelvin);
        assert_eq!(ConductivityMeter.measure(&v).unwrap().in_range, expected);
        assert_eq!(
            observable_support(&v, "conductivity").status,
            if expected {
                ObservableStatus::Computed
            } else {
                ObservableStatus::Estimated
            }
        );
    }
}

#[test]
fn scene_measurement_manifest_and_persistence_share_the_same_support() {
    let mut v = water();
    v.unpriced_heat.push(SpeciesId::new("chalcanthite"));
    let scene = scene_vessel(&v);
    let support = observable_support(&v, "temperature");
    assert_eq!(
        scene
            .observables
            .iter()
            .find(|s| s.observable == "temperature"),
        Some(&support)
    );
    let mut device = MeasurementDevice::new(Thermometer, 4).unwrap();
    let record = device.record(&v, 0.0).unwrap();
    assert_eq!(record.model_support, Some(support.clone()));
    let restored: MeasurementRecord =
        serde_json::from_str(&serde_json::to_string(&record).unwrap()).unwrap();
    assert_eq!(restored, record);
    let restored: Vessel = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
    assert_eq!(observable_manifest(&restored), observable_manifest(&v));
    assert_eq!(
        Calorimeter
            .support(&v, &Calorimeter.measure(&v).unwrap())
            .status,
        ObservableStatus::Incomplete
    );
}

#[test]
fn unknown_routes_are_discoverable_and_zero_mass_is_computed_zero() {
    let v = Vessel::new(VesselId(0), "empty");
    assert_eq!(
        observable_support(&v, "pH").status,
        ObservableStatus::Unsupported
    );
    assert_eq!(
        observable_support(&v, "new-unregistered-observable").status,
        ObservableStatus::Unsupported
    );
    assert_eq!(
        observable_support(&v, "mass").status,
        ObservableStatus::Computed
    );
    assert_eq!(instrument::Balance.measure(&v).unwrap().value, 0.0);
    assert!(PhMeter.measure(&v).is_none());
}

#[test]
fn spectral_gaps_do_not_become_complete_zero_absorbance() {
    let mut v = water();
    v.deposit(SpeciesId::new("Cu+2"), Moles(0.001), Phase::Aqueous);
    v.solution = Some(solution(
        0.002,
        serde_json::json!([{"name":"Cu(NH3)4+2", "molality":0.001}]),
        "complete",
    ));
    let support = observable_support(&v, "absorbance");
    assert_eq!(support.status, ObservableStatus::Incomplete, "{support:?}");
    assert!(instrument::Spectrophotometer::default()
        .measure(&v)
        .is_none());
    assert!(matches!(
        observable_support(&v, "pH").status,
        ObservableStatus::Computed | ObservableStatus::Estimated
    ));
}

#[test]
fn aqueous_scope_is_preserved_even_when_the_vessel_has_multiple_layers() {
    let mut v = water();
    v.deposit(SpeciesId::new("hexane"), Moles(0.1), Phase::Liquid);
    v.solution = Some(solution(0.001, serde_json::json!([]), "complete"));
    assert_eq!(
        observable_support(&v, "pH").scope,
        ObservableScope::AqueousPhase
    );
    assert_eq!(
        observable_support(&v, "mass").scope,
        ObservableScope::WholeVessel
    );
    v.solution.as_mut().unwrap().scope = vessel::SolutionScope::SolventOnly;
    assert_eq!(
        observable_support(&v, "pH").scope,
        ObservableScope::SolventOnly
    );
    assert_eq!(
        observable_support(&v, "pH").status,
        ObservableStatus::Incomplete
    );
}

#[test]
fn legacy_measurement_records_load_without_invented_model_support() {
    let record: MeasurementRecord = serde_json::from_value(serde_json::json!({
        "observable":"temperature", "value":25.0, "unit":"C", "precision":0.1,
        "in_range":true, "elapsed_seconds":0.0, "sample_index":0
    }))
    .unwrap();
    assert_eq!(record.model_support, None);
}

#[test]
fn boiling_fallback_at_uncovered_pressure_is_not_in_range() {
    let mut v = water();
    v.pressure = Pascal(5e6);
    let instrument = MeltingPointApparatus(TransitionRead::Boiling);
    let reading = instrument.measure(&v).unwrap();
    assert!(!reading.in_range);
    assert_eq!(
        instrument.support(&v, &reading).status,
        ObservableStatus::Estimated
    );
    assert!(instrument
        .support(&v, &reading)
        .assumptions
        .iter()
        .any(|s| s.contains("normal boiling point")));
}

#[test]
fn bench_json_measurements_use_same_contract_as_ideal_devices_and_scene() {
    let mut bench = Bench::new();
    bench.vessels[0] = water();
    bench.vessels[0]
        .unpriced_heat
        .push(SpeciesId::new("chalcanthite"));
    for (instrument, observable) in [
        (Instrument::Thermometer, "temperature"),
        (Instrument::Balance, "mass"),
        (Instrument::Calorimeter, "enthalpy"),
        (Instrument::PressureGauge, "pressure"),
    ] {
        let events = bench
            .step(Operator::Measure {
                vessel: VesselId(0),
                instrument,
            })
            .unwrap();
        let expected = observable_support(&bench.vessels[0], observable);
        assert!(events.iter().any(|event| matches!(event, Event::Measured { model_support: Some(support), .. } if support == &expected)), "{events:?}");
        let restored: Vec<Event> =
            serde_json::from_str(&serde_json::to_string(&events).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(&events).unwrap()
        );
    }
    let legacy: Event = serde_json::from_value(serde_json::json!({
        "event":"measured", "vessel":0, "instrument":"thermometer", "value":25.0, "unit":"C"
    }))
    .unwrap();
    assert!(matches!(
        legacy,
        Event::Measured {
            model_support: None,
            ..
        }
    ));
}

#[test]
fn heat_capacity_missing_data_and_curve_endpoints_have_distinct_support() {
    let mut v = water();
    let data = species::lookup(&SpeciesId::new("water")).unwrap();
    let curve = states::heat_capacity_curve(data, Phase::Liquid).unwrap();
    let (lower, upper) = curve.range().unwrap();
    for (temperature, expected) in [
        (lower, ObservableStatus::Computed),
        ((lower + upper) / 2.0, ObservableStatus::Computed),
        (upper, ObservableStatus::Computed),
        (upper + 1.0, ObservableStatus::Estimated),
    ] {
        v.temperature = Kelvin(temperature);
        let support = observable_support(&v, "temperature");
        assert_eq!(support.status, expected, "{support:?}");
        assert!(support.provenance.contains(&curve.source.to_string()));
    }
    v.temperature = Kelvin(298.15);
    // Intentional missing-data injection: a known ion has deliberately
    // omitted partial Cp in solution, but zero Cp is no model of a dry solid.
    for (phase, expected, reason) in [
        (
            Phase::Aqueous,
            ObservableStatus::Estimated,
            "aqueous-partial-heat-capacity-omitted",
        ),
        (
            Phase::Solid,
            ObservableStatus::Incomplete,
            "missing-heat-capacity",
        ),
    ] {
        let mut sample = v.clone();
        sample.deposit(SpeciesId::new("Na+"), Moles(0.001), phase);
        let support = observable_support(&sample, "temperature");
        assert_eq!(support.status, expected);
        assert!(support.reasons.iter().any(|r| r == reason));
        sample.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
        assert_eq!(
            observable_support(&sample, "temperature").status,
            ObservableStatus::Computed
        );
        assert_eq!(observable_support(&sample, "enthalpy").status, expected);
    }
}

#[test]
fn mixed_solvent_boundary_does_not_treat_a_separate_oil_layer_as_cosolvent() {
    for (ethanol, hexane, expected, reason) in [
        (0.0, 10.0, ObservableStatus::Computed, None),
        (
            1.0,
            0.0,
            ObservableStatus::Estimated,
            Some("mixed-solvent-activity-not-calibrated"),
        ),
        (
            10.0,
            0.0,
            ObservableStatus::Unsupported,
            Some("mostly-organic-aqueous-route-declined"),
        ),
        (
            1.0,
            10.0,
            ObservableStatus::Estimated,
            Some("mixed-solvent-activity-not-calibrated"),
        ),
    ] {
        let mut sample = water();
        sample.deposit(SpeciesId::new("ethanol"), Moles(ethanol), Phase::Liquid);
        sample.deposit(SpeciesId::new("hexane"), Moles(hexane), Phase::Liquid);
        sample.solution = Some(solution(
            0.001,
            serde_json::json!([{"name":"Na+", "molality":0.001}]),
            "complete",
        ));
        for observable in ["pH", "conductivity"] {
            let support = observable_support(&sample, observable);
            assert_eq!(
                support.status, expected,
                "{ethanol}/{hexane}/{observable}: {support:?}"
            );
            assert_eq!(support.scope, ObservableScope::AqueousPhase);
            if let Some(reason) = reason {
                assert!(support.reasons.iter().any(|r| r == reason));
            }
        }
    }
}

#[test]
fn material_heat_gap_is_disclosed_without_disabling_aqueous_chemistry() {
    let mut bench = Bench::new();
    for line in ["add v1 water 50mL", "add v1 vegetable_oil 50mL"] {
        bench
            .step(script::parse_op(line).unwrap().unwrap())
            .unwrap();
    }
    let sample = &mut bench.vessels[0];
    sample.solution = Some(solution(0.001, serde_json::json!([]), "complete"));
    assert_eq!(
        observable_support(sample, "temperature").status,
        ObservableStatus::Incomplete
    );
    assert!(observable_support(sample, "temperature")
        .reasons
        .iter()
        .any(|r| r == "missing-material-heat-capacity"));
    assert_eq!(
        observable_support(sample, "pH").status,
        ObservableStatus::Estimated
    );
    sample.temperature = Kelvin(solve::AQUEOUS_MODEL_CEILING_K + 1.0);
    assert_eq!(
        observable_support(sample, "pH").status,
        ObservableStatus::Unsupported
    );
}

#[test]
fn available_kinetic_routes_are_estimates_and_no_zinc_route_is_not_inertness() {
    let mut peroxide = water();
    peroxide.deposit(SpeciesId::new("H2O2"), Moles(0.1), Phase::Liquid);
    let mut zinc_acid = water();
    zinc_acid.deposit(SpeciesId::new("Zn"), Moles(0.001), Phase::Solid);
    zinc_acid.deposit(SpeciesId::new("H+"), Moles(0.01), Phase::Aqueous);
    zinc_acid.solution = Some(solution(
        0.1,
        serde_json::json!([{"name":"H+", "molality":0.1}]),
        "complete",
    ));
    for (sample, expected) in [
        (&peroxide, ObservableStatus::Estimated),
        (&zinc_acid, ObservableStatus::Unsupported),
    ] {
        let support = observable_support(sample, "reaction_rate");
        assert_eq!(support.status, expected, "{support:?}");
        assert_eq!(support.scope, ObservableScope::WholeVessel);
        assert!(support
            .assumptions
            .iter()
            .any(|text| text.contains("not an aggregate rate")));
        if expected == ObservableStatus::Estimated {
            assert!(!support.provenance.is_empty());
            assert!(support
                .assumptions
                .iter()
                .any(|text| text.contains("uncertainty")));
            assert!(support
                .reasons
                .iter()
                .any(|text| text.starts_with("selected-rate-model:")));
        } else {
            assert!(support
                .reasons
                .iter()
                .any(|text| text == "no-selected-rate-model"));
            assert!(support
                .assumptions
                .iter()
                .any(|text| text.contains("not zero rate or inertness")));
        }
        assert!(observable_manifest(sample)
            .iter()
            .any(|row| row == &support));
    }
}

#[test]
fn temperature_limitations_propagate_to_dependent_observables_not_imposed_pressure() {
    let mut sample = water();
    sample.unpriced_heat.push(SpeciesId::new("chalcanthite"));
    sample.solution = Some(solution(
        0.001,
        serde_json::json!([{"name":"Na+","molality":0.001}]),
        "complete",
    ));
    assert_eq!(
        observable_support(&sample, "pressure").status,
        ObservableStatus::Computed
    );
    assert_eq!(
        observable_support(&sample, "pH").status,
        ObservableStatus::Estimated
    );
    sample.headspace = vessel::Headspace::Sealed {
        volume: Liters(0.1),
    };
    assert_eq!(
        observable_support(&sample, "pressure").status,
        ObservableStatus::Incomplete
    );
    sample.thermal_mode = ThermalMode::Thermostatted(Kelvin(298.15));
    assert_eq!(
        observable_support(&sample, "pressure").status,
        ObservableStatus::Computed
    );
    assert_eq!(
        observable_support(&sample, "pH").status,
        ObservableStatus::Computed
    );
    assert_eq!(
        observable_support(&sample, "enthalpy").status,
        ObservableStatus::Incomplete
    );
}

#[test]
fn measurement_support_is_visible_in_each_locale_and_appropriate_register() {
    for (locale, incomplete, unsupported, estimated) in [
        (
            Locale::EN,
            "estimate is incomplete",
            "does not support this observable",
            "model estimate",
        ),
        (
            Locale::parse("de"),
            "Schätzung ist unvollständig",
            "unterstützt diese Messgröße",
            "Modellschätzung",
        ),
        (
            Locale::parse("fr"),
            "estimation est incomplète",
            "ne prend pas en charge cette grandeur",
            "estimation du modèle",
        ),
    ] {
        for (status, phrase) in [
            (ObservableStatus::Incomplete, incomplete),
            (ObservableStatus::Unsupported, unsupported),
            (ObservableStatus::Estimated, estimated),
        ] {
            let mut support = observable_support(&water(), "mass");
            support.status = status;
            let event = Event::Measured {
                vessel: VesselId(0),
                instrument: Instrument::Balance,
                value: 1.0,
                unit: "g".into(),
                note: None,
                note_reason: None,
                model_support: Some(support),
            };
            for level in 1..=3 {
                let rendered =
                    render_events_in(std::slice::from_ref(&event), Register(level), locale)
                        .join("\n");
                assert_eq!(
                    rendered.contains(phrase),
                    status != ObservableStatus::Estimated || level >= 2,
                    "{status:?}/{locale:?}/{level}: {rendered}"
                );
            }
        }
    }
}

#[test]
fn typed_support_does_not_repeat_existing_missing_temperature_disclosure() {
    let mut bench = Bench::new();
    bench.vessels[0] = water();
    bench.vessels[0]
        .unpriced_heat
        .push(SpeciesId::new("chalcanthite"));
    let events = bench
        .step(Operator::Measure {
            vessel: VesselId(0),
            instrument: Instrument::Thermometer,
        })
        .unwrap();
    for level in 1..=3 {
        let rendered = render_events_in(&events, Register(level), Locale::EN).join("\n");
        assert!(rendered.contains("incomplete"));
        assert!(!rendered.contains("contributing model data"), "{rendered}");
    }
}

#[test]
fn manifest_discovers_all_scalar_operator_observables_and_refuses_empty_density() {
    let empty = Vessel::new(VesselId(0), "empty");
    let matrix = observable_manifest(&empty);
    for name in ["headspace_volume", "density", "activity"] {
        assert!(matrix.iter().any(|row| row.observable == name));
    }
    assert_eq!(
        observable_support(&empty, "density").status,
        ObservableStatus::Unsupported
    );
    let mut metal = empty.clone();
    metal.deposit(SpeciesId::new("Cu"), Moles(0.01), Phase::Solid);
    assert_eq!(
        observable_support(&metal, "density").scope,
        ObservableScope::DrySolid
    );
    assert_eq!(
        observable_support(&metal, "density").status,
        ObservableStatus::Estimated
    );
    metal.deposit(SpeciesId::new("Zn"), Moles(0.01), Phase::Solid);
    assert_eq!(
        observable_support(&metal, "density").status,
        ObservableStatus::Unsupported
    );
    let mut liquid = water();
    liquid.deposit(SpeciesId::new("Cu"), Moles(0.01), Phase::Solid);
    assert_eq!(
        observable_support(&liquid, "density").scope,
        ObservableScope::LiquidPhase
    );
    liquid.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
    assert_eq!(
        observable_support(&liquid, "density").status,
        ObservableStatus::Incomplete
    );
}

#[test]
fn solvent_only_spectrum_never_claims_complete_sample_coverage() {
    let mut sample = water();
    sample.deposit(SpeciesId::new("Na+"), Moles(0.001), Phase::Aqueous);
    for (scope, expected) in [
        ("complete", ObservableStatus::Estimated),
        ("solvent_only", ObservableStatus::Incomplete),
    ] {
        sample.solution = Some(solution(0.001, serde_json::json!([]), scope));
        let support = observable_support(&sample, "absorbance");
        assert_eq!(support.status, expected);
        if scope == "solvent_only" {
            assert_eq!(support.scope, ObservableScope::SolventOnly);
            assert!(support
                .reasons
                .iter()
                .any(|reason| reason == "uncovered-solutes"));
        }
    }
}

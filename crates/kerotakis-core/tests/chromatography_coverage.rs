//! Modeled peaks cannot hide other positive components of the injected sample.
use kerotakis_core::chromatography::sample;
use kerotakis_core::*;

fn vessel() -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "chromatography coverage");
    vessel.deposit(SpeciesId::new("water"), Moles(5.5), Phase::Liquid);
    vessel
}

#[test]
fn mapped_ethanol_does_not_hide_neutral_ester_acid_or_outside_method_ions() {
    let mut vessel = vessel();
    for (key, moles, phase) in [
        ("ethanol", 0.001, Phase::Liquid),
        ("ethyl_acetate", 0.000633, Phase::Liquid),
        ("ethyl_acetate", 0.0000707, Phase::Aqueous),
        ("CH3COOH", 0.001, Phase::Aqueous),
        ("Na+", 0.001, Phase::Aqueous),
        ("Cl-", 0.001, Phase::Aqueous),
    ] {
        vessel.deposit(SpeciesId::new(key), Moles(moles), phase);
    }
    let before = serde_json::to_value(&vessel).unwrap();
    let result = sample(&vessel);
    assert_eq!(result.peaks.len(), 1);
    assert_eq!(result.peaks[0].species.0, "ethanol");
    assert_eq!(result.peaks[0].relative_area, 1.0);
    assert_eq!(
        result.unparameterised,
        vec![SpeciesId::new("CH3COOH"), SpeciesId::new("ethyl_acetate")]
    );
    assert_eq!(
        result.outside_method,
        vec![SpeciesId::new("Cl-"), SpeciesId::new("Na+")]
    );
    assert_eq!(serde_json::to_value(&vessel).unwrap(), before);
}

#[test]
fn missing_neutrals_are_reported_with_or_without_a_mapped_peak() {
    for mapped in [false, true] {
        let mut vessel = vessel();
        vessel.deposit(
            SpeciesId::new("ethyl_acetate"),
            Moles(5e-16),
            Phase::Aqueous,
        );
        if mapped {
            vessel.deposit(SpeciesId::new("ethanol"), Moles(0.001), Phase::Aqueous);
        }
        let result = sample(&vessel);
        assert_eq!(
            result.unparameterised,
            vec![SpeciesId::new("ethyl_acetate")]
        );
        assert_eq!(result.peaks.len(), usize::from(mapped));
        assert!(result.outside_method.is_empty());
    }
}

#[test]
fn supported_mapped_sample_keeps_ideal_detector_area_and_retention_relationships() {
    let mut vessel = vessel();
    vessel.deposit(SpeciesId::new("ethanol"), Moles(0.001), Phase::Aqueous);
    vessel.deposit(SpeciesId::new("methanol"), Moles(0.0005), Phase::Aqueous);
    let result = sample(&vessel);
    assert!(result.unparameterised.is_empty() && result.outside_method.is_empty());
    assert_eq!(result.peaks.len(), 2);
    let ethanol = result
        .peaks
        .iter()
        .find(|p| p.species.0 == "ethanol")
        .unwrap();
    let methanol = result
        .peaks
        .iter()
        .find(|p| p.species.0 == "methanol")
        .unwrap();
    assert_eq!(ethanol.relative_area, 1.0);
    assert_eq!(methanol.relative_area, 0.5);
    for peak in result.peaks {
        assert!(peak.retention_time_s >= 60.0 && peak.width_s > 0.0);
        assert!(peak.rf > 0.0 && peak.rf <= 1.0);
    }
}

#[test]
fn only_positive_dissolved_inventory_is_part_of_the_injected_sample() {
    let mut vessel = vessel();
    vessel.deposit(SpeciesId::new("ethyl_acetate"), Moles(0.001), Phase::Gas);
    vessel.deposit(SpeciesId::new("CH3COOH"), Moles(0.001), Phase::Solid);
    vessel.deposit(
        SpeciesId::new("unrepresented-neutral"),
        Moles(0.0),
        Phase::Aqueous,
    );
    let result = sample(&vessel);
    assert!(
        result.peaks.is_empty()
            && result.unparameterised.is_empty()
            && result.outside_method.is_empty()
    );
}

#[test]
fn bench_event_and_all_registers_disclose_missing_neutral_retention() {
    let mut bench = Bench::new();
    bench.vessels[0] = vessel();
    bench.vessels[0].deposit(SpeciesId::new("ethanol"), Moles(0.001), Phase::Aqueous);
    bench.vessels[0].deposit(SpeciesId::new("ethyl_acetate"), Moles(0.001), Phase::Liquid);
    let events = bench
        .step(Operator::Measure {
            vessel: VesselId(0),
            instrument: Instrument::Chromatograph,
        })
        .unwrap();
    let result = events
        .iter()
        .find(|event| matches!(event, Event::Chromatographed { .. }))
        .unwrap();
    assert!(
        matches!(result, Event::Chromatographed { peaks, unparameterised, outside_method, .. } if peaks.len()==1 && unparameterised==&vec![SpeciesId::new("ethyl_acetate")] && outside_method.is_empty())
    );
    for level in [
        render::Register::LV1,
        render::Register::LV2,
        render::Register::LV3,
    ] {
        let text = render::render_event(result, level);
        assert!(
            text.contains("Incomplete sample coverage") && text.contains("ethyl_acetate"),
            "{text}"
        );
        assert!(text.contains("elution is unknown"), "{text}");
    }
    // Saved event logs from before this additive coverage field still load.
    let mut saved = serde_json::to_value(result).unwrap();
    saved.as_object_mut().unwrap().remove("unparameterised");
    let restored: Event = serde_json::from_value(saved).unwrap();
    assert!(
        matches!(restored, Event::Chromatographed { unparameterised, .. } if unparameterised.is_empty())
    );
}

#[test]
fn only_unparameterised_neutrals_refuse_without_announcing_a_modeled_chromatogram() {
    let mut bench = Bench::new();
    bench.vessels[0] = vessel();
    bench.vessels[0].deposit(SpeciesId::new("ethyl_acetate"), Moles(0.001), Phase::Liquid);
    let events = bench.step(Operator::Measure {
        vessel: VesselId(0), instrument: Instrument::Chromatograph,
    }).unwrap();
    assert!(!events.iter().any(|e| matches!(e, Event::Chromatographed { .. })));
    assert!(events.iter().any(|e| matches!(e, Event::NotYetModeled { cause: ops::NotModelledCause::NotParameterised, what, .. } if what.contains("ethyl_acetate") && what.contains("unresolved"))));
}

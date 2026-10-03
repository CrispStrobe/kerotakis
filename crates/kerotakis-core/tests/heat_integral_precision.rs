//! Integral precision against independent high-precision antiderivatives.
use kerotakis_core::*;

#[test]
fn nasa_integrals_match_independent_eighty_digit_references() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../audits/thermal-contracts-20261003/integral-references.json"
    ))
    .unwrap();
    for row in fixture["references"].as_array().unwrap() {
        let phase = match row["phase"].as_str().unwrap() {
            "liquid" => Phase::Liquid,
            "solid" => Phase::Solid,
            "gas" => Phase::Gas,
            _ => unreachable!(),
        };
        let data = species::lookup(&SpeciesId::new(row["species"].as_str().unwrap())).unwrap();
        let t0 = row["t0"].as_f64().unwrap();
        let t1 = row["t1"].as_f64().unwrap();
        let expected = row["joules_per_mol"].as_f64().unwrap();
        let actual = states::enthalpy_between(data, phase, t0, t1);
        assert!(
            (actual - expected).abs() <= 8.0 * f64::EPSILON * expected.abs(),
            "{row}: actual={actual:.17e}, reference={expected:.17e}"
        );
        assert_eq!(states::enthalpy_between(data, phase, t1, t0), -actual);
    }
}

#[test]
fn liquid_temperature_inversion_closes_at_thaw_certificate_precision() {
    for amount in [0.005, 0.5, 50.0] {
        let mut vessel = Vessel::new(VesselId(0), "water calorimeter");
        vessel.deposit(SpeciesId::new("water"), Moles(amount), Phase::Liquid);
        for end in [298.15, 308.0889802715948, 350.0, 550.0] {
            let dose = vessel.energy_between(states::WATER_FREEZING_K, end);
            let actual_end = vessel.temperature_after_from(states::WATER_FREEZING_K, dose);
            let actual = vessel.energy_between(states::WATER_FREEZING_K, actual_end);
            assert!(
                (actual - dose).abs() <= 128.0 * f64::EPSILON * actual.abs().max(dose.abs()).max(1.0),
                "amount={amount}, end={end}, actual_end={actual_end:.17e}, actual={actual:.17e}, dose={dose:.17e}"
            );
        }
    }
}

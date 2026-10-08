//! Frozen owner-scale cancellation control; all fixture arithmetic is binary exact.
use super::*;

#[test]
fn owner_sum_cancellation_accepts_raw_without_rescaling_trace_remainder() {
    for scale in [1.0_f64, 2.0_f64.powi(-400), 2.0_f64.powi(400)] {
        // Exact available = 1 + 2^-54, final owned = 1 - 2^-40.
        // The exact aqueous remainder is 2^-40 + 2^-54. Each supplied
        // operand is representable; only a naive available sum loses 2^-54.
        let initial_aqueous = 2.0_f64.powi(-54) * scale;
        let final_owned = (1.0 - 2.0_f64.powi(-40)) * scale;
        let raw_remainder = (2.0_f64.powi(-40) + 2.0_f64.powi(-54)) * scale;
        let mut vessel = Vessel::new(VesselId(0), "binary cancellation fixture");
        vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
        vessel
            .solid_solutions
            .push(SolidSolution::aragonite_strontianite(
                "binary crystal",
                Moles(scale),
                Moles(0.0),
            ));
        let mut problem = partition(&vessel).expect("valid binary input");
        problem.totals = vec![
            ("Ca".into(), initial_aqueous),
            ("Sr".into(), 0.0),
            ("C".into(), initial_aqueous),
        ];
        problem.phases.clear();
        problem.gases.clear();
        problem.external_gases.clear();
        let crystals = vec![SolidSolution::aragonite_strontianite(
            "binary crystal",
            Moles(final_owned),
            Moles(0.0),
        )];
        let mut ions = vec![
            ("Ca".into(), raw_remainder),
            ("Sr".into(), 0.0),
            ("C".into(), raw_remainder),
        ];
        let value = |column: &str| match column {
            "pH" => Some(7.0),
            "mu" => Some(0.01),
            _ => None,
        };
        PhreeqcEquilibrator::apply_balance_corrections(
            &vessel,
            &problem,
            &mut ions,
            &mut [],
            &[],
            &crystals,
            &value,
        )
        .expect("exact physical raw remainder survives bounded owner-sum cancellation");
        assert_eq!(
            ions[0].1.to_bits(),
            raw_remainder.to_bits(),
            "Ca trace must not be projected to an inaccurate subtraction"
        );
        assert_eq!(
            ions[2].1.to_bits(),
            raw_remainder.to_bits(),
            "C trace must not be projected to an inaccurate subtraction"
        );
    }
}

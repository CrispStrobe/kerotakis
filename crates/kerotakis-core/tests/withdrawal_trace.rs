//! Source-informed mechanical ownership forecasts; no equilibrium solver.
use kerotakis_core::*;

fn prepared(trace: f64) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "withdrawal trace control");
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    vessel.deposit(SpeciesId::new("NaCl"), Moles(trace), Phase::Solid);
    vessel
}

fn traces() -> [f64; 5] {
    [1e-12, 1e-15, 5e-16, 1e-18, f64::from_bits(1)]
}

#[test]
fn zero_withdrawal_preserves_all_owned_portions() {
    for trace in traces() {
        let mut vessel = prepared(trace);
        let before = vessel.clone();
        assert_eq!(
            vessel.withdraw(&SpeciesId::new("water"), Moles(0.0)),
            Moles(0.0)
        );
        assert_eq!(
            serde_json::to_value(&vessel).unwrap(),
            serde_json::to_value(&before).unwrap(),
            "trace {trace:e}"
        );
    }
}

#[test]
fn absent_species_withdrawal_preserves_all_owned_portions() {
    for trace in traces() {
        let mut vessel = prepared(trace);
        let before = vessel.clone();
        assert_eq!(
            vessel.withdraw(&SpeciesId::new("ethanol"), Moles(1.0)),
            Moles(0.0)
        );
        assert_eq!(
            serde_json::to_value(&vessel).unwrap(),
            serde_json::to_value(&before).unwrap(),
            "trace {trace:e}"
        );
    }
}

#[test]
fn zero_phase_withdrawal_preserves_all_owned_portions() {
    for trace in traces() {
        let mut vessel = prepared(trace);
        let before = vessel.clone();
        assert_eq!(
            vessel.withdraw_phase(&SpeciesId::new("water"), Moles(0.0), Phase::Liquid),
            Moles(0.0)
        );
        assert_eq!(
            serde_json::to_value(&vessel).unwrap(),
            serde_json::to_value(&before).unwrap(),
            "trace {trace:e}"
        );
    }
}

#[test]
fn absent_phase_withdrawal_preserves_all_owned_portions() {
    for trace in traces() {
        let mut vessel = prepared(trace);
        let before = vessel.clone();
        assert_eq!(
            vessel.withdraw_phase(&SpeciesId::new("water"), Moles(1.0), Phase::Solid),
            Moles(0.0)
        );
        assert_eq!(
            serde_json::to_value(&vessel).unwrap(),
            serde_json::to_value(&before).unwrap(),
            "trace {trace:e}"
        );
    }
}

#[test]
fn bulk_withdrawal_preserves_unrelated_trace() {
    for trace in traces() {
        let mut vessel = prepared(trace);
        let salt = vessel.contents[1].clone();
        assert_eq!(
            vessel.withdraw(&SpeciesId::new("water"), Moles(0.25)),
            Moles(0.25)
        );
        assert_eq!(vessel.contents[0].moles, Moles(0.75));
        assert_eq!(vessel.contents.get(1), Some(&salt), "trace {trace:e}");
    }
}

#[test]
fn phase_withdrawal_preserves_same_species_in_excluded_phase() {
    for trace in traces() {
        let mut vessel = prepared(trace);
        vessel.deposit(SpeciesId::new("water"), Moles(trace), Phase::Solid);
        let excluded = vessel.contents[2].clone();
        assert_eq!(
            vessel.withdraw_phase(&SpeciesId::new("water"), Moles(0.25), Phase::Liquid),
            Moles(0.25)
        );
        assert_eq!(
            vessel
                .contents
                .iter()
                .find(|p| p.species.0 == "water" && p.phase == Phase::Solid),
            Some(&excluded)
        );
    }
}

#[test]
fn partial_withdrawal_keeps_positive_trace_remainder() {
    for phase_only in [false, true] {
        let mut vessel = Vessel::new(VesselId(0), "positive remainder");
        let species = SpeciesId::new("water");
        vessel.deposit(species.clone(), Moles(2e-16), Phase::Liquid);
        let removed = if phase_only {
            vessel.withdraw_phase(&species, Moles(1e-16), Phase::Liquid)
        } else {
            vessel.withdraw(&species, Moles(1e-16))
        };
        assert_eq!(removed, Moles(1e-16));
        assert_eq!(vessel.contents.len(), 1);
        assert_eq!(vessel.contents[0].moles, Moles(1e-16));
    }
}

#[test]
fn exact_exhaustion_removes_only_exhausted_owner() {
    for phase_only in [false, true] {
        let mut vessel = prepared(1e-18);
        let salt = vessel.contents[1].clone();
        let species = SpeciesId::new("water");
        let removed = if phase_only {
            vessel.withdraw_phase(&species, Moles(1.0), Phase::Liquid)
        } else {
            vessel.withdraw(&species, Moles(1.0))
        };
        assert_eq!(removed, Moles(1.0));
        assert_eq!(vessel.contents, vec![salt]);
    }
}

#[test]
fn tiny_owned_dose_can_be_withdrawn_completely() {
    for phase_only in [false, true] {
        let mut vessel = Vessel::new(VesselId(0), "tiny dose");
        let species = SpeciesId::new("water");
        vessel.deposit(species.clone(), Moles(1e-18), Phase::Liquid);
        let removed = if phase_only {
            vessel.withdraw_phase(&species, Moles(1e-18), Phase::Liquid)
        } else {
            vessel.withdraw(&species, Moles(1e-18))
        };
        assert_eq!(removed, Moles(1e-18));
        assert!(vessel.contents.is_empty());
    }
}

#[test]
fn repeated_bulk_withdrawals_keep_unrelated_trace_unchanged() {
    for phase_only in [false, true] {
        let mut vessel = prepared(1e-18);
        let salt = vessel.contents[1].clone();
        for _ in 0..4 {
            let species = SpeciesId::new("water");
            let removed = if phase_only {
                vessel.withdraw_phase(&species, Moles(0.125), Phase::Liquid)
            } else {
                vessel.withdraw(&species, Moles(0.125))
            };
            assert_eq!(removed, Moles(0.125));
            assert_eq!(vessel.contents.get(1), Some(&salt));
        }
        assert_eq!(vessel.contents[0].moles, Moles(0.5));
    }
}

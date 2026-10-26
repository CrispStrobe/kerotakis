//! Shared mechanical acceptance, not a production reaction or chemistry forecast.
//! Fresh local vessels keep the artificial callback outside the learner bench.
use kerotakis_core::aqueous_continuation::{
    characterize_via, continue_via, whole_step_heat_available,
};
use kerotakis_core::vessel::StepStart;
use kerotakis_core::*;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ContinuationAcceptanceReport {
    pub schema: u32,
    pub mechanical_fixture: bool,
    pub cases: Vec<ContinuationCase>,
}

impl ContinuationAcceptanceReport {
    pub fn passed(&self) -> bool {
        self.cases.iter().all(|case| case.passed)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ContinuationCase {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

/// `expect_missing` asks for a different, deliberately un-prewarmed callback
/// extent. Use only with cache-only or deliberately failing-hook acceptance;
/// a working solver should answer that question rather than refuse it.
pub fn run_continuation_acceptance(
    solver: &mut dyn Equilibrator,
    expect_missing: bool,
) -> ContinuationAcceptanceReport {
    let cases = [false, true]
        .into_iter()
        .map(|sealed| {
            let result = scenario(solver, sealed, expect_missing);
            let (passed, detail) = match result {
                Ok(Some(refusal)) => (true, refusal),
                Ok(None) => (true, "passed mechanical control".into()),
                Err(detail) => (false, detail),
            };
            ContinuationCase {
                id: format!(
                    "{}-{}",
                    if sealed { "sealed" } else { "open" },
                    if expect_missing {
                        "changed-input-refusal"
                    } else {
                        "continuation"
                    }
                ),
                passed,
                detail,
            }
        })
        .collect();
    ContinuationAcceptanceReport {
        schema: 1,
        mechanical_fixture: true,
        cases,
    }
}

fn scenario(
    solver: &mut dyn Equilibrator,
    sealed: bool,
    expect_missing: bool,
) -> Result<Option<String>, String> {
    let mut vessel = Vessel::new(VesselId(0), "isolated mechanical continuation control");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("HCl"), Moles(0.005), Phase::Liquid);
    vessel.deposit(SpeciesId::new("H2O2"), Moles(0.002), Phase::Liquid);
    if sealed {
        vessel.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        // This control isolates aqueous heat ownership from physical boiling.
        // An empty one-litre headspace plus only the callback's O2 is a
        // near-vacuum; StateEquilibrator legitimately cools it by vaporisation.
        vessel.retain_gas(SpeciesId::new("N2"), Moles(0.04));
    }
    vessel.step_start = Some(StepStart::capture(&vessel));
    let (_, receipt) =
        characterize_via(solver, &mut vessel).map_err(|e| format!("initial answer: {e}"))?;
    let mut stale = vessel.clone();
    stale.deposit(SpeciesId::new("water"), Moles(0.01), Phase::Liquid);
    let stale_before = format!("{stale:?}");
    let mut stale_callback_called = false;
    let stale_result = continue_via(&mut stale, &receipt, solver, |_| {
        stale_callback_called = true;
        Ok(Vec::new())
    });
    if stale_result.is_ok() || stale_callback_called || format!("{stale:?}") != stale_before {
        return Err("stale initial receipt did not refuse atomically".into());
    }
    let before = format!("{vessel:?}");
    let temperature = vessel.temperature;
    let snapshot = vessel.step_start.clone();
    let initial_peroxide = vessel.moles_of(&SpeciesId::new("H2O2")).0;
    if initial_peroxide < 0.0019 {
        return Err("fixture peroxide unavailable".into());
    }
    let extent = initial_peroxide * if expect_missing { 0.5 } else { 1.0 };
    let result = continue_via(&mut vessel, &receipt, solver, |candidate| {
        // Artificial atom/charge-balanced callback. This installs no chemical
        // reaction row, rate law, empirical endpoint or reviewed reaction heat.
        candidate.withdraw(&SpeciesId::new("H2O2"), Moles(extent));
        candidate.deposit(SpeciesId::new("water"), Moles(extent), Phase::Liquid);
        let species = SpeciesId::new("O2");
        let moles = Moles(extent / 2.0);
        Ok(vec![if candidate.retain_gas(species.clone(), moles) {
            Event::GasContained {
                vessel: candidate.id,
                species,
                moles,
            }
        } else {
            Event::GasEvolved {
                vessel: candidate.id,
                species,
                moles,
            }
        }])
    });
    if expect_missing {
        let error = result
            .err()
            .ok_or("un-prewarmed callback unexpectedly answered")?;
        let SolveError::NotConverged { solver, detail, .. } = &error;
        let expected_failure = (solver == "phreeqc-aqueous (cache-only build)"
            && detail.contains("this state is not in the shipped results"))
            || (solver == "phreeqc-aqueous (external engine)"
                && detail.contains("controlled continuation current-question failure"));
        if !expected_failure {
            return Err(format!(
                "unexpected refusal instead of cache/hook failure: {error}"
            ));
        }
        if format!("{vessel:?}") != before {
            return Err("failed readback changed full vessel state".into());
        }
        return Ok(Some(format!("expected cache/hook refusal: {error}")));
    }
    let events = result.map_err(|e| format!("continuation answer: {e}"))?;
    let gas_report = events
        .iter()
        .filter_map(|event| match event {
            Event::GasContained { species, moles, .. } if sealed && species.0 == "O2" => {
                Some(moles.0)
            }
            Event::GasEvolved { species, moles, .. } if !sealed && species.0 == "O2" => {
                Some(moles.0)
            }
            _ => None,
        })
        .sum::<f64>();
    if (gas_report - extent / 2.0).abs() > extent * 1e-7
        || vessel.moles_of(&SpeciesId::new("H2O2")).0 > initial_peroxide * 1e-7
    {
        return Err("mechanical callback product or gas accounting mismatch".into());
    }
    if whole_step_heat_available(&vessel)
        || vessel.step_start != snapshot
        || vessel.temperature != temperature
    {
        return Err("continuation lost ownership, original snapshot or temperature".into());
    }
    if !events
        .iter()
        .any(|e| matches!(e, Event::NotYetModeled { .. }))
    {
        return Err("unknown heat boundary not reported".into());
    }
    for _ in 0..2 {
        equilibrate_phase_coupled(solver, &mut vessel)
            .map_err(|e| format!("shared wrapper reentry: {e}"))?;
        if whole_step_heat_available(&vessel) || vessel.temperature != temperature {
            return Err("shared wrapper reentry reclaimed unknown heat".into());
        }
    }
    Ok(None)
}

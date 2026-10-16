//! Actual host acceptance for the bounded endpoint, separate from mechanical callbacks.
//! This validates engineering balances, not empirical rate/heat accuracy.
use crate::continuation_acceptance::ContinuationCase;
use kerotakis_core::aqueous_continuation::characterize_via;
use kerotakis_core::curated::{fires, PostAqueousEndpoint, REACTIONS};
use kerotakis_core::vessel::StepStart;
use kerotakis_core::*;

#[derive(serde::Serialize)]
pub struct AcidEndpointReport {
    pub schema: u32,
    pub endpoint: &'static str,
    pub cases: Vec<ContinuationCase>,
}
impl AcidEndpointReport {
    pub fn passed(&self) -> bool {
        self.cases.iter().all(|c| c.passed)
    }
}

fn fixture(sealed: bool, missing: bool) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "bounded acidic endpoint acceptance");
    for (key, n, phase) in [
        ("water", 5.55, Phase::Liquid),
        ("H2SO4", 0.006, Phase::Liquid),
        ("KMnO4", 0.002, Phase::Solid),
        ("H2O2", if missing { 0.0045 } else { 0.005 }, Phase::Liquid),
    ] {
        v.deposit(SpeciesId::new(key), Moles(n), phase);
    }
    if sealed {
        v.headspace = Headspace::Sealed {
            volume: Liters(1.0),
        };
        v.retain_gas(SpeciesId::new("N2"), Moles(0.04));
    }
    v.step_start = Some(StepStart::capture(&v));
    v
}
fn amount(v: &Vessel, key: &str) -> f64 {
    v.moles_of(&SpeciesId::new(key)).0
}
fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-10 + 1e-6 * a.abs().max(b.abs())
}

fn scenario(
    solver: &mut dyn Equilibrator,
    sealed: bool,
    missing: bool,
    wrapper: bool,
) -> Result<String, String> {
    let mut v = fixture(sealed, missing);
    characterize_via(solver, &mut v).map_err(|e| format!("initial real answer: {e}"))?;
    let row = REACTIONS
        .iter()
        .find(|r| r.post_aqueous == Some(PostAqueousEndpoint::AcidicPermanganatePeroxide))
        .unwrap();
    if !fires(&v, row) {
        return Err(format!(
            "real initial answer outside frozen endpoint domain: {v:?}"
        ));
    }
    let mn = amount(&v, "MnO4-");
    let per = amount(&v, "H2O2");
    let acid = amount(&v, "H+");
    let x = (mn / 2.0)
        .min(per / 5.0)
        .min(acid / 6.0)
        .min((-v.solute_charge).max(0.0) / 6.0);
    let before = v.clone();
    let result = if wrapper {
        solver.equilibrate(&mut v)
    } else {
        displacement::over(solver, &mut v)
    };
    if missing {
        let error = result
            .err()
            .ok_or("deliberately unprewarmed endpoint unexpectedly answered")?;
        let text = error.to_string();
        if !(text.contains("this state is not in the shipped results")
            || text.contains("controlled acidic endpoint current-question failure"))
        {
            return Err(format!(
                "unexpected error rather than missing current question: {error}"
            ));
        }
        if format!("{v:?}") != format!("{before:?}") {
            return Err("failed endpoint readback changed full vessel".into());
        }
        return Ok(format!("expected atomic readback refusal: {error}"));
    }
    let events = result.map_err(|e| format!("endpoint readback: {e}"))?;
    let reactions = events
        .iter()
        .filter(|e| matches!(e,Event::ReactionOccurred{equation,..} if equation==row.equation))
        .count();
    let oxygen: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasContained { species, moles, .. } if sealed && species.0 == "O2" => {
                Some(moles.0)
            }
            Event::GasEvolved { species, moles, .. } if !sealed && species.0 == "O2" => {
                Some(moles.0)
            }
            _ => None,
        })
        .sum();
    if reactions != 1
        || !close(amount(&v, "MnO4-"), mn - 2.0 * x)
        || !close(amount(&v, "H2O2"), per - 5.0 * x)
        || !close(amount(&v, "Mn+2") - amount(&before, "Mn+2"), 2.0 * x)
        || !close(oxygen, 5.0 * x)
        || !close(amount(&v, "K+"), amount(&before, "K+"))
        || !close(amount(&v, "SO4-2"), amount(&before, "SO4-2"))
        || !close(v.temperature.0, before.temperature.0)
        || v.solution.as_ref().is_none_or(|s| !s.ph.is_finite())
        || kerotakis_core::aqueous_continuation::whole_step_heat_available(&v)
    {
        return Err(format!("endpoint balance/reading contract failed: extent={x}, O2={oxygen}, vessel={v:?}, events={events:?}"));
    }
    Ok(format!(
        "bounded endpoint: extent={x:.12e} mol, O2={oxygen:.12e} mol; no rate/heat calibration"
    ))
}
fn report(solver: &mut dyn Equilibrator, missing: bool, wrapper: bool) -> AcidEndpointReport {
    let cases = [false, true]
        .into_iter()
        .map(|sealed| {
            let result = scenario(solver, sealed, missing, wrapper);
            ContinuationCase {
                id: format!(
                    "{}-{}",
                    if sealed { "sealed" } else { "open" },
                    if missing {
                        "missing-readback"
                    } else {
                        "endpoint"
                    }
                ),
                passed: result.is_ok(),
                detail: result.unwrap_or_else(|e| e),
            }
        })
        .collect();
    AcidEndpointReport {
        schema: 1,
        endpoint: "sulfate-acidic-permanganate-peroxide",
        cases,
    }
}
/// The actual native/browser wrapper's Equilibrator method is exercised.
pub fn run_wrapper(solver: &mut dyn Equilibrator, missing: bool) -> AcidEndpointReport {
    report(solver, missing, true)
}
/// Prewarm the same core host bridge around a borrowed concrete native engine.
pub fn run_aqueous(solver: &mut dyn Equilibrator) -> AcidEndpointReport {
    report(solver, false, false)
}
/// Warm only the initial questions of a separate missing-readback control.
/// Never compute that variant's transformed answer: cache-only/failing-hook
/// hosts must decline it atomically, rather than borrow another endpoint.
pub fn warm_missing_initials(solver: &mut dyn Equilibrator) -> Result<(), String> {
    for sealed in [false, true] {
        let mut v = fixture(sealed, true);
        characterize_via(solver, &mut v).map_err(|e| format!("missing fixture initial: {e}"))?;
        characterize_via(solver, &mut v)
            .map_err(|e| format!("missing fixture current initial: {e}"))?;
    }
    Ok(())
}

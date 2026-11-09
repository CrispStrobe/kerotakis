#![cfg(feature = "engine")]
//! Double-indicator (Warder) alkalinity titrations in an open beaker.
//!
//! One titration increment is delivered with no elapsed time. Room-air
//! exchange is a rate and cannot relax a supersaturated solution within it;
//! before the fix every 0.1 mL vented the CO2 that carbonate alkalinity was
//! still holding at pH 9-10, each vented mole spending a proton, and both a
//! carbonate/bicarbonate and a hydroxide/carbonate sample read the pH 8.3
//! endpoint at the same 0.855 mmol.
use kerotakis_core::ops::Endpoint;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

const MMOL: f64 = 1e-3;
const HCL_M: f64 = 0.1;

fn sample(solver: &mut PhreeqcEquilibrator, parts: &[(&str, f64)]) -> Bench {
    let mut bench = Bench::new();
    let v = &mut bench.vessels[0];
    assert!(matches!(v.headspace, Headspace::Open));
    v.deposit(
        SpeciesId::new("water"),
        species::lookup_key("water")
            .unwrap()
            .moles_from_liters(Liters(0.1)),
        Phase::Liquid,
    );
    for (name, mmol) in parts {
        v.deposit(
            SpeciesId::new(name),
            Moles(mmol * MMOL),
            species::lookup_key(name).unwrap().standard_phase,
        );
    }
    solver.equilibrate(v).unwrap();
    bench
}

/// Returns (acid delivered in mol, CO2 vented in mol, final pH).
fn titrate_to(bench: &mut Bench, solver: &mut PhreeqcEquilibrator, ph: f64) -> (f64, f64, f64) {
    let events = bench
        .step_with(
            Operator::Titrate {
                vessel: VesselId(0),
                titrant: SpeciesId::new("HCl"),
                concentration: HCL_M,
                step: Liters(0.0001),
                target_ph: ph,
                max_steps: 200,
                endpoint: Endpoint::Ph,
            },
            solver,
            &PermissiveScreen,
        )
        .unwrap();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })),
        "{events:?}"
    );
    let (volume, reached) = events
        .iter()
        .find_map(|e| match e {
            Event::Titrated {
                total_volume,
                endpoint_reached,
                ..
            } => Some((total_volume.0, *endpoint_reached)),
            _ => None,
        })
        .expect("titration event");
    assert_eq!(reached, Some(true), "endpoint pH {ph} not reached");
    let vented: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    let final_ph = bench.vessels[0].solution.as_ref().unwrap().ph;
    (volume * HCL_M, vented, final_ph)
}

/// (label, NaOH, Na2CO3, NaHCO3) in mmol.
const CASES: &[(&str, f64, f64, f64)] = &[
    ("hydroxide + carbonate", 0.4, 0.3, 0.0),
    ("carbonate + bicarbonate", 0.0, 0.3, 0.4),
    ("pure carbonate", 0.0, 0.5, 0.0),
    ("pure hydroxide", 0.5, 0.0, 0.0),
    ("hydroxide-rich", 0.8, 0.1, 0.0),
];

#[test]
fn warder_endpoints_follow_composition() {
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut first_endpoints = Vec::new();
    for &(label, oh, co3, hco3) in CASES {
        let parts: Vec<(&str, f64)> = [("NaOH", oh), ("Na2CO3", co3), ("NaHCO3", hco3)]
            .into_iter()
            .filter(|(_, n)| *n > 0.0)
            .collect();
        let mut bench = sample(&mut solver, &parts);
        let (p, vented_first, ph1) = titrate_to(&mut bench, &mut solver, 8.3);
        let (extra, _, ph2) = titrate_to(&mut bench, &mut solver, 4.5);
        let t = p + extra;
        // Phenolphthalein: OH- + CO3^2- -> HCO3-. Methyl orange: all of it.
        let p_expected = (oh + co3) * MMOL;
        let t_expected = (oh + 2.0 * co3 + hco3) * MMOL;
        assert!((ph1 - 8.3).abs() < 1e-3 && (ph2 - 4.5).abs() < 1e-3);
        assert!(
            (p - p_expected).abs() <= 0.05 * p_expected + 0.01 * MMOL,
            "{label}: first endpoint {p:e} mol, expected {p_expected:e}"
        );
        assert!(
            (t - t_expected).abs() <= 0.03 * t_expected + 0.01 * MMOL,
            "{label}: total {t:e} mol, expected {t_expected:e}"
        );
        // Closed in time: nothing leaves above pH 8.3.
        assert_eq!(vented_first, 0.0, "{label}: CO2 vented above pH 8.3");
        // Ideal reconstruction recovers the composition.
        let (oh_r, co3_r, hco3_r) = if 2.0 * p >= t {
            (2.0 * p - t, t - p, 0.0)
        } else {
            (0.0, p, t - 2.0 * p)
        };
        for (what, got, want) in [
            ("OH", oh_r, oh),
            ("CO3", co3_r, co3),
            ("HCO3", hco3_r, hco3),
        ] {
            assert!(
                (got / MMOL - want).abs() <= 0.06 * (oh + 2.0 * co3 + hco3) + 0.01,
                "{label}: reconstructed {what} {} mmol, expected {want}",
                got / MMOL
            );
        }
        first_endpoints.push(p);
    }
    // Different compositions must not share one endpoint.
    assert!((first_endpoints[0] - first_endpoints[1]).abs() > 0.3 * MMOL);
}

#[test]
fn zero_time_dose_does_not_change_ordinary_open_beaker_venting() {
    // Scope control: once the titration is over, an ordinary solve of the
    // CO2-supersaturated beaker still relaxes it toward room air.
    let mut solver = PhreeqcEquilibrator::new().unwrap();
    let mut bench = sample(&mut solver, &[("Na2CO3", 0.5)]);
    let (_, _, _) = titrate_to(&mut bench, &mut solver, 4.5);
    let v = &mut bench.vessels[0];
    assert!(
        !v.zero_time_dose,
        "flag must not persist past the titration"
    );
    v.solution = None;
    let events = solver.equilibrate(v).unwrap();
    let vented: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(
        vented > 0.0,
        "ordinary open-beaker solve stopped venting: {events:?}"
    );
}

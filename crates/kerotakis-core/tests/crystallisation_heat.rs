//! Crystallisation heat on cooling (TH-007 family), through
//! `kerotakis_core::crystallisation`: one energy ledger for the sensible
//! cooling and the crystallisation it drives.
//!
//! The dissolution enthalpy registered here is the one the routed engine
//! supplies once PR #867 lands: llnl.dat `Niter`, `KNO3 = K+ + NO3-`,
//! `-delta_H 35.4794 kJ/mol` (LLNL thermo.com.V8.R6.230, public domain),
//! which #867's `reference:llnl.dat:Niter` route extracts. Registered as a
//! pricer, exactly as the phreeqc stack will, not copied into the module.

use kerotakis_core::crystallisation::{self, Refusal};
use kerotakis_core::*;

fn niter_pricer(salt: &str) -> Option<f64> {
    (salt == "KNO3").then_some(35.4794)
}

const WATER_MOL: f64 = 2.775_464_890_369_136_6; // 50 g
const KNO3_MOL: f64 = 0.296_726_513_107_399_17; // 30 g

fn solution(t_k: f64, salt_mol: f64, extra: &[(&str, f64)]) -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(WATER_MOL), Phase::Liquid);
    v.deposit(SpeciesId::new("K+"), Moles(salt_mol), Phase::Aqueous);
    v.deposit(SpeciesId::new("NO3-"), Moles(salt_mol), Phase::Aqueous);
    for (key, n) in extra {
        v.deposit(SpeciesId::new(key), Moles(*n), Phase::Aqueous);
    }
    v.temperature = Kelvin(t_k);
    v
}

fn cool(v: &Vessel, joules: f64) -> Option<Result<crystallisation::Crystallised, Refusal>> {
    crystallisation::register_dissolution_pricer(niter_pricer);
    let from = v.temperature.0;
    let sensible = |t_f: f64, n: f64| {
        let mut end = v.clone();
        if let Some(Ok((curve, _))) = crystallisation::binary_solution(v) {
            crystallisation::apply(&mut end, curve, n);
        }
        end.energy_between(t_f, from)
    };
    crystallisation::cool_with_crystallisation(v, joules, &sensible)
}

/// TH-007 after #867's dissolution cooling: 321.76 K, then `cool 12kJ`.
#[test]
fn kno3_cooling_crystallises_releases_heat_and_stops_where_the_ledger_closes() {
    let v = solution(321.76, KNO3_MOL, &[]);
    let owned = cool(&v, 12_000.0)
        .expect("saturation is reached")
        .expect("owned");
    // Independent hand balance (water Cp 75.3 J/mol/K, solid KNO3 96.4,
    // ions 0, Berkeley curve, 35.48 kJ/mol): 291.35 K, 0.147 mol.
    assert!((289.0..294.0).contains(&owned.final_k), "{owned:?}");
    assert!((0.13..0.16).contains(&owned.moles), "{owned:?}");
    assert!(owned.moles < KNO3_MOL);
    // Pure sensible cooling of the same dose goes far colder: the crystals
    // paid part of the bill.
    assert!(owned.final_k > v.temperature_after(-12_000.0) + 10.0);
    // One ledger: sensible + crystallisation heat == the dose.
    let mut end = v.clone();
    crystallisation::apply(
        &mut end,
        crystallisation::curve_for("KNO3").unwrap(),
        owned.moles,
    );
    let sensible = end.energy_between(owned.final_k, 321.76);
    assert!(
        (sensible + owned.heat_j - 12_000.0).abs() < 1e-3,
        "{sensible}"
    );
    assert!((owned.heat_j - owned.moles * 35_479.4).abs() < 1e-6);
    // ...and the solution it leaves is exactly saturated.
    let water_g = WATER_MOL * species::lookup_key("water").unwrap().molar_mass;
    let cap = crystallisation::curve_for("KNO3")
        .unwrap()
        .capacity_mol(water_g, owned.final_k)
        .unwrap();
    assert!((KNO3_MOL - owned.moles - cap).abs() < 1e-8);
    // Element and charge closure of the move.
    let k: f64 = end
        .contents
        .iter()
        .filter(|p| p.species.0 == "K+" || p.species.0 == "KNO3")
        .map(|p| p.moles.0)
        .sum();
    assert!((k - KNO3_MOL).abs() < 1e-12);
    assert!(crystallisation::holds_solid(
        &{
            let mut e = end.clone();
            e.temperature = Kelvin(owned.final_k);
            e
        },
        "KNO3"
    ));
}

#[test]
fn an_unsaturated_solution_cools_without_a_phase_change() {
    // 0.05 mol in 50 g water saturates near 5 C; 2 kJ from 50 C does not
    // get there.
    let v = solution(323.15, 0.05, &[]);
    assert!(cool(&v, 2_000.0).is_none());
}

#[test]
fn a_dose_past_the_measured_curve_toward_the_eutectic_is_refused() {
    let v = solution(321.76, KNO3_MOL, &[]);
    match cool(&v, 40_000.0) {
        Some(Err(Refusal::BelowCurveDomain { salt, floor_k })) => {
            assert_eq!(salt, "KNO3");
            assert!((floor_k - 273.55).abs() < 1e-9);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_mixed_solution_is_outside_the_binary_curve() {
    let v = solution(321.76, KNO3_MOL, &[("Na+", 0.1), ("Cl-", 0.1)]);
    assert!(matches!(
        cool(&v, 12_000.0),
        Some(Err(Refusal::NotBinary { salt: "KNO3" }))
    ));
}

/// The brine-freezing route of #852/#857 is untouched: no curve owns NaCl,
/// so a cold brine never enters this module and keeps its ice plateau
/// (`tests/brine_phase_energy.rs`).
#[test]
fn nacl_brine_is_not_claimed_by_the_crystallisation_ledger() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(WATER_MOL), Phase::Liquid);
    v.deposit(SpeciesId::new("Na+"), Moles(0.2), Phase::Aqueous);
    v.deposit(SpeciesId::new("Cl-"), Moles(0.2), Phase::Aqueous);
    v.temperature = Kelvin(275.0);
    assert!(crystallisation::binary_solution(&v).is_none());
    assert!(cool(&v, 30_000.0).is_none());
}

//! PHREEQC-derived solubilities on the routed databases, validated against
//! independent measured references. The routed database's phase is the
//! primary solubility source (`kerotakis_phreeqc::crystallisation_route`);
//! the measured points are only the check. Each case prints its actual
//! deviation so the hosted log records it.
//!
//! References:
//! * NaCl 35.84 and KCl 34.37 parts per 100 parts water at 15.20 and
//!   19.55 C — Earl of Berkeley, Phil. Trans. R. Soc. A 203 (1904) 189-215,
//!   doi:10.1098/rsta.1904.0020, the same reading the registry already
//!   cites (`literature/berkeley-saturated-solutions-1904`).
//! * KNO3, seven points 0.40-91.65 C, same paper, carried in
//!   `kerotakis_core::crystallisation::CURVES`.

#![cfg(feature = "engine")]

use kerotakis_core::crystallisation;
use kerotakis_phreeqc::{databases, Phreeqc};

/// Tolerances, stated once. Pitzer is the concentrated-brine route and
/// must reproduce a saturated chloride closely; Debye-Hueckel routes are
/// outside their ionic-strength domain at saturation and are held to a
/// looser bound (the router already caveats them above 1 molal).
const PITZER_TOLERANCE: f64 = 0.05;
const DEBYE_HUECKEL_TOLERANCE: f64 = 0.15;

fn saturation_molality(db: &[u8], extra_phases: &str, phase: &str, element: &str, t_c: f64) -> f64 {
    let mut pqc = Phreeqc::with_database(db).expect("load database");
    pqc.run(&format!(
        "{extra_phases}
SOLUTION 1
    units mol/kgw
    temp  {t_c}
    pH    7
EQUILIBRIUM_PHASES 1
    {phase} 0 20
SELECTED_OUTPUT
    -reset false
    -totals {element}
END
"
    ))
    .expect("equilibrate against the phase");
    pqc.last_value(element).expect("total column")
}

fn deviation(derived: f64, measured: f64) -> f64 {
    derived / measured - 1.0
}

const NACL_M: f64 = 35.84 * 10.0 / 58.443; // 15.20 C
const KCL_M: f64 = 34.37 * 10.0 / 74.551; // 19.55 C

#[test]
fn halite_and_sylvite_on_pitzer_reproduce_the_measured_saturation() {
    let nacl = saturation_molality(databases::PITZER, "", "Halite", "Na", 15.20);
    let kcl = saturation_molality(databases::PITZER, "", "Sylvite", "K", 19.55);
    let (dn, dk) = (deviation(nacl, NACL_M), deviation(kcl, KCL_M));
    eprintln!(
        "pitzer Halite {nacl:.4} m vs {NACL_M:.4} ({:+.2}%)",
        dn * 100.0
    );
    eprintln!(
        "pitzer Sylvite {kcl:.4} m vs {KCL_M:.4} ({:+.2}%)",
        dk * 100.0
    );
    assert!(dn.abs() < PITZER_TOLERANCE && dk.abs() < PITZER_TOLERANCE);
}

#[test]
fn halite_on_the_debye_hueckel_routes_is_within_its_stated_bound() {
    for (tag, db) in [
        ("wateq4f", databases::WATEQ4F.to_vec()),
        ("minteq.v4", databases::minteq_v4().to_vec()),
    ] {
        let nacl = saturation_molality(&db, "", "Halite", "Na", 15.20);
        let d = deviation(nacl, NACL_M);
        eprintln!(
            "{tag} Halite {nacl:.4} m vs {NACL_M:.4} ({:+.2}%)",
            d * 100.0
        );
        assert!(d.abs() < DEBYE_HUECKEL_TOLERANCE, "{tag}: {d}");
    }
}

/// The llnl.dat `Niter` entry, read from the vendored file rather than
/// retyped.
fn llnl_niter_block() -> String {
    const LLNL: &str = include_str!("../../../vendor/iphreeqc/database/llnl.dat");
    let block: Vec<&str> = LLNL
        .lines()
        .skip_while(|l| l.trim_end() != "Niter")
        .take_while(|l| !l.trim().is_empty())
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect();
    assert!(!block.is_empty(), "llnl defines Niter");
    format!("PHASES\n{}\n", block.join("\n"))
}

/// Why KNO3 is NOT admitted: no routed database defines Niter, and posing
/// llnl's Niter on the default route (wateq4f activities) misses every
/// measured point by far more than any stated tolerance (an independent
/// pre-run estimate gave -51% at 0.4 C to -74% at 91.65 C). This test
/// pins that failure so admitting the phase later requires a real
/// concentrated-nitrate activity model, not a tolerance change.
#[test]
fn llnl_niter_on_the_default_route_fails_validation_against_berkeley() {
    let curve = crystallisation::curve_for("KNO3").expect("reference curve");
    assert!(!curve.runtime_admitted);
    let phases = llnl_niter_block();
    let molar_mass = 101.1032;
    for &(t_c, grams) in curve.points {
        let measured = grams * 10.0 / molar_mass;
        let derived = saturation_molality(databases::WATEQ4F, &phases, "Niter", "K", t_c);
        let d = deviation(derived, measured);
        eprintln!(
            "wateq4f+llnl Niter {t_c:5.2} C: {derived:.3} m vs {measured:.3} ({:+.1}%)",
            d * 100.0
        );
        assert!(d < -DEBYE_HUECKEL_TOLERANCE * 2.0, "{t_c} C: {d}");
    }
}

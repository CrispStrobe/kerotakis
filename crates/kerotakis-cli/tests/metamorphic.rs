//! Metamorphic invariants: properties that relate *two* runs of the bench,
//! which conservation checks on a single run cannot see (PLAN.md, "Testing
//! is part of the architecture"). Each test drives the real binary through
//! the `--json` contract, so what is checked is what every client sees.

use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Distinguishes concurrent `run` calls: test threads run in parallel, and
/// a content-derived name once collided (two scenarios share a script), so
/// one thread removed the directory the other was writing into.
static CASE: AtomicUsize = AtomicUsize::new(0);

/// Run a script through `kero run --json` and parse the step stream.
fn run(script: &str) -> Vec<serde_json::Value> {
    let dir = std::env::temp_dir().join(format!(
        "kero-meta-{}-{}",
        std::process::id(),
        CASE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("case.lab");
    std::fs::write(&lab, script).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        out.status.success(),
        "script failed:\n{script}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let steps = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).expect("every line is JSON"))
        .collect();
    std::fs::remove_dir_all(&dir).ok();
    steps
}

/// The first vessel's state after the last step.
fn final_vessel(steps: &[serde_json::Value]) -> serde_json::Value {
    steps.last().expect("at least one step")["bench"]["vessels"][0].clone()
}

fn ph(vessel: &serde_json::Value) -> f64 {
    vessel["solution"]["ph"]
        .as_f64()
        .unwrap_or_else(|| panic!("no characterised solution: {vessel}"))
}

fn ionic_strength(vessel: &serde_json::Value) -> f64 {
    vessel["solution"]["ionic_strength"].as_f64().unwrap()
}

/// Total moles of one species in the vessel, across phases.
fn moles_of(vessel: &serde_json::Value, species: &str) -> f64 {
    vessel["contents"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["species"] == species)
        .map(|p| p["moles"].as_f64().unwrap())
        .sum()
}

/// Equilibrium has no memory: the order reagents were added in must not
/// change the state they equilibrate to.
///
/// Tolerances, derived rather than guessed — reaching them took five
/// rounds of measurement across two sessions (2026-08-19/20; the wrong
/// turns are chronicled in PLAN.md, "Testing is part of the
/// architecture"):
///
/// *Element totals* (conserved) agree to ~6e-11 mol → asserted at 1e-9.
/// The *dissolved/solid split* (solved equilibrium) moves ~5e-9 → 1e-6.
/// *pH* is the sharp one: charge-balanced pH amplifies composition
/// imbalance by ~1.5e6 pH per mol/kgw and rides temperature at
/// dpH/dT ≈ -0.0163/K — which is how a 0.82 K path-dependent temperature
/// bug (dissolution enthalpy unrecorded for a phase the routed database
/// cannot name; fixed generically in 39c592e, regression-tested
/// engine-side) masqueraded as a 1.4e-2 composition mystery through
/// three wrong mechanisms. What remains after the fix is the solver's
/// own 0.05 K temperature convergence tolerance, worth 0.05 × 0.0163 ≈
/// 8e-4 in pH — so the 1e-3 assertion is that bound with headroom.
/// Measured here post-fix: 7.9e-5.
///
/// Excluded along the way, so nobody re-litigates them: print precision
/// (~1e-7 against a 1.4e-2 symptom), solver KNOBS tolerance
/// (bit-identical), quantisation cancellation (a non-dyadic x1.7 scaling
/// holds to 2.5e-11), uniform solute rescaling (K+ carried none of the
/// excess), and carrying moles forward as a pH fix (re-solving both
/// final states at one temperature leaves 2.8e-10).
#[test]
fn order_independence_of_equilibrium() {
    let salt_first = run("add v1 water 100mL\nadd v1 NaCl 0.1mol\nadd v1 AgNO3 0.01mol");
    let silver_first = run("add v1 water 100mL\nadd v1 AgNO3 0.01mol\nadd v1 NaCl 0.1mol");
    let (a, b) = (final_vessel(&salt_first), final_vessel(&silver_first));

    assert!(
        (ph(&a) - ph(&b)).abs() < 1e-3,
        "pH depends on addition order: {} vs {}",
        ph(&a),
        ph(&b)
    );
    assert!(
        (ionic_strength(&a) - ionic_strength(&b)).abs() < 1e-3,
        "ionic strength depends on addition order: {} vs {}",
        ionic_strength(&a),
        ionic_strength(&b)
    );
    let (agcl_a, agcl_b) = (moles_of(&a, "AgCl"), moles_of(&b, "AgCl"));
    assert!(
        agcl_a > 0.009,
        "the marquee precipitate is missing: {agcl_a} mol"
    );
    assert!(
        (agcl_a - agcl_b).abs() < 1e-6,
        "precipitate depends on addition order: {agcl_a} vs {agcl_b} mol"
    );
    // Element totals are the conserved, well-conditioned invariant (see
    // the comment above) — the dissolved/solid split is covered at its own
    // tier by the AgCl assertion. The existence check keeps a renamed
    // species key from passing as a vacuous 0 == 0.
    let total = |v: &serde_json::Value, parts: &[&str]| -> f64 {
        parts.iter().map(|s| moles_of(v, s)).sum()
    };
    for (element, parts) in [
        ("sodium", &["Na+"][..]),
        ("chlorine", &["Cl-", "AgCl"][..]),
        ("silver", &["Ag+", "AgCl"][..]),
    ] {
        let (ta, tb) = (total(&a, parts), total(&b, parts));
        assert!(ta > 1e-6, "{element} missing from the rebuilt vessel");
        assert!(
            (ta - tb).abs() < 1e-9,
            "total {element} depends on addition order: {ta} vs {tb} mol"
        );
    }
}

/// Doubling everything doubles the extensive quantities and changes no
/// intensive one — pH and ionic strength describe the solution, not the
/// beaker. Catches unit and normalisation bugs no single run reveals.
#[test]
fn scale_invariance_of_intensive_properties() {
    let x1 = run("add v1 water 100mL\nadd v1 NaCl 0.1mol\nadd v1 AgNO3 0.01mol");
    let x2 = run("add v1 water 200mL\nadd v1 NaCl 0.2mol\nadd v1 AgNO3 0.02mol");
    let (a, b) = (final_vessel(&x1), final_vessel(&x2));

    assert!(
        (ph(&a) - ph(&b)).abs() < 1e-6,
        "pH is intensive; doubling the beaker moved it: {} vs {}",
        ph(&a),
        ph(&b)
    );
    assert!(
        (ionic_strength(&a) - ionic_strength(&b)).abs() < 1e-6,
        "ionic strength is intensive: {} vs {}",
        ionic_strength(&a),
        ionic_strength(&b)
    );
    let (agcl_1, agcl_2) = (moles_of(&a, "AgCl"), moles_of(&b, "AgCl"));
    assert!(
        (agcl_2 - 2.0 * agcl_1).abs() < 1e-6,
        "the precipitate is extensive; expected {} to double to {agcl_2}",
        agcl_1
    );
}

/// Adding water to an acid must move the pH toward 7 and never past it —
/// dilution monotonicity, over the computed activities rather than the
/// naive -log(c) (which is why the steps are not exactly 0.30).
#[test]
fn dilution_moves_ph_toward_seven() {
    let mut previous = None;
    for ml in [100, 200, 400, 800] {
        let steps = run(&format!("add v1 water {ml}mL\nadd v1 HCl 0.001mol"));
        let now = ph(&final_vessel(&steps));
        assert!(now < 7.0, "an acid cannot dilute past neutral: pH {now}");
        if let Some(before) = previous {
            assert!(
                now > before,
                "dilution must raise an acid's pH: {before} -> {now} at {ml} mL"
            );
        }
        previous = Some(now);
    }
}

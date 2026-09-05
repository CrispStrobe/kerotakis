#![cfg(feature = "engine")]
//! `free_proton` and `free_hydroxide` are measurements, and the point of
//! them is that they are not the titratable totals a gate might otherwise
//! reach for.
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn brew(steps: &[(&str, f64)]) -> Vessel {
    let mut bench = Bench::new();
    let mut stack = SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]));
    let v = VesselId(0);
    for (key, moles) in steps {
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new(key),
                    moles: Moles(*moles),
                    at: None,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("add");
    }
    bench.vessel(v).unwrap().clone()
}

/// A strong acid is almost entirely dissociated, so the measurement and
/// the titratable total agree.
#[test]
fn a_strong_acid_measures_what_it_could_titrate() {
    let v = brew(&[("water", 5.55), ("HCl", 0.1)]);
    assert!(
        (v.free_proton - 0.1).abs() < 0.02,
        "0.1 mol of HCl should measure ~0.1 mol of free protons, got {}",
        v.free_proton
    );
    assert!(v.free_hydroxide < 1e-9, "an acid holds no free base");
}

/// A weak one does not, and this is the whole reason the field exists
/// rather than a gate reaching for `unspent_acidity`.
#[test]
fn a_weak_acid_measures_far_less_than_it_could_titrate() {
    let v = brew(&[("water", 5.55), ("CH3COOH", 0.1)]);
    let titratable = kerotakis_core::displacement::unspent_acidity(&v);
    assert!(
        v.free_proton < titratable / 50.0,
        "acetic acid should be barely dissociated: {} free against {} titratable",
        v.free_proton,
        titratable
    );
    assert!(v.free_proton > 0.0, "but it is an acid, and some is loose");
}

/// And a weak BASE holds measurable hydroxide without any hydroxide having
/// been added — which is why the heat balance may not read a positive
/// solute charge as free base.
#[test]
fn a_bicarbonate_holds_measured_hydroxide_it_was_never_given() {
    let v = brew(&[("water", 5.55), ("NaHCO3", 0.02)]);
    assert!(
        v.free_hydroxide > 0.0 && v.free_hydroxide < 1e-4,
        "hydrolysis, not an alkali: {}",
        v.free_hydroxide
    );
    assert!(
        v.solute_charge > 100.0 * v.free_hydroxide,
        "the charge excess is carbonate alkalinity and dwarfs the hydroxide: \
         charge {} against measured {}",
        v.solute_charge,
        v.free_hydroxide
    );
}

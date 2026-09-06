//! TEMPORARY diagnostic: what the open-vessel air reservoir is worth.
//!
//! Prints the charge, the air admitted, the temperatures and the energy
//! terms for the scenarios the fix has to keep honest. Deleted once the
//! numbers are assertions.

use kerotakis_cea::{db, ThermalEquilibrator};
use kerotakis_core::*;
use std::fmt::Write as _;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, s: &mut SolverStack, v: VesselId, key: &str, moles: f64) {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            s,
            &PermissiveScreen,
        )
        .expect("add");
}

fn report(out: &mut String, title: &str, bench: &Bench, v: VesselId, events: &[Event]) {
    let _ = writeln!(out, "\n=== {title} ===");
    for e in events {
        match e {
            Event::EnergyTransferred {
                requested_j,
                delivered_j,
                sensible_j,
                passes,
                capped,
                heating,
                ..
            } => {
                let _ = writeln!(
                    out,
                    "  ENERGY heating={heating} requested={requested_j:.1} delivered={delivered_j:.1} \
                     sensible={sensible_j:.1} passes={passes} capped={capped}"
                );
            }
            Event::TemperatureChanged { from, to, .. } => {
                let _ = writeln!(out, "  T {:.2} -> {:.2} K", from.0, to.0);
            }
            Event::ThermalEquilibrium {
                temperature,
                reaction_energy_j,
                ..
            } => {
                let _ = writeln!(
                    out,
                    "  EQ at {:.2} K, reaction_energy={:?}",
                    temperature.0, reaction_energy_j
                );
            }
            Event::GasEvolved { species, moles, .. } => {
                let _ = writeln!(out, "  GAS {} {:.6} mol", species.0, moles.0);
            }
            Event::Precipitated { species, moles, .. } => {
                let _ = writeln!(out, "  SOLID +{:.6} mol {}", moles.0, species.0);
            }
            Event::Consumed {
                species,
                moles,
                remaining,
                ..
            } => {
                let _ = writeln!(
                    out,
                    "  CONSUMED {:.6} mol {} (left {:?})",
                    moles.0, species.0, remaining
                );
            }
            Event::Ignited { energy_j, .. } => {
                let _ = writeln!(out, "  IGNITED energy={energy_j:?}");
            }
            other => {
                let _ = writeln!(
                    out,
                    "  {}",
                    format!("{other:?}").chars().take(120).collect::<String>()
                );
            }
        }
    }
    let ves = bench.vessel(v).expect("vessel");
    let _ = writeln!(
        out,
        "  -- final T {:.2} K, Cp {:.3} J/K, enthalpy(sensible) {:.1} J",
        ves.temperature.0,
        ves.heat_capacity(),
        ves.enthalpy().0
    );
    for p in &ves.contents {
        let _ = writeln!(
            out,
            "  -- holds {:.6} mol {} ({:?})",
            p.moles.0, p.species.0, p.phase
        );
    }
}

/// The air the charge admits today, per `thermal::charge`.
fn air_moles(condensed: f64, c: f64, h: f64, o: f64) -> f64 {
    let stoich_o2 = c + h / 4.0 - o / 2.0;
    (condensed.max(0.01) * 8.0).max(stoich_o2.max(0.0) * 1.20 / 0.21)
}

fn h_of(name: &str, t: f64) -> f64 {
    db().get(name)
        .or_else(|| db().get_reactant(name))
        .and_then(|s| s.h(t))
        .unwrap_or(f64::NAN)
}

#[test]
fn measure_the_open_air_reservoir() {
    let mut out = String::new();
    let v = VesselId(0);

    // A. chalk, 40 kJ
    {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, v, "CaCO3", 0.1);
        let events = bench
            .step_with(
                Operator::Heat {
                    vessel: v,
                    energy: Joules(40_000.0),
                    source: None,
                },
                &mut s,
                &PermissiveScreen,
            )
            .expect("heat");
        report(
            &mut out,
            "A. 0.1 mol CaCO3 + 40 kJ (bunsen)",
            &bench,
            v,
            &events,
        );
    }

    // B. chalk, 5 kJ
    {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, v, "CaCO3", 0.1);
        let events = bench
            .step_with(
                Operator::Heat {
                    vessel: v,
                    energy: Joules(5_000.0),
                    source: None,
                },
                &mut s,
                &PermissiveScreen,
            )
            .expect("heat");
        report(
            &mut out,
            "B. 0.1 mol CaCO3 + 5 kJ (bunsen)",
            &bench,
            v,
            &events,
        );
    }

    // C. ethanol 0.010 mol, ignite
    for (name, key, moles) in [
        ("C. ethanol 0.010 mol ignite", "ethanol", 0.010),
        ("D. methanol 0.010 mol ignite", "methanol", 0.010),
        ("G. ethanol 10 mL ignite", "ethanol", 10.0 * 0.789 / 46.069),
        ("E. iron 1 g ignite", "Fe", 1.0 / 55.845),
    ] {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, v, key, moles);
        let events = bench
            .step_with(Operator::Ignite { vessel: v }, &mut s, &PermissiveScreen)
            .expect("ignite");
        report(&mut out, name, &bench, v, &events);
    }

    // F. magnesium 0.05 mol + 1 kJ
    {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, v, "Mg", 0.05);
        let events = bench
            .step_with(
                Operator::Heat {
                    vessel: v,
                    energy: Joules(1_000.0),
                    source: None,
                },
                &mut s,
                &PermissiveScreen,
            )
            .expect("heat");
        report(&mut out, "F. 0.05 mol Mg + 1 kJ", &bench, v, &events);
    }

    // The air each charge admits, and what its sensible heat is worth.
    let _ = writeln!(&mut out, "\n=== admitted air ===");
    for (what, condensed, c, h, o, t) in [
        ("chalk 0.1 mol at 1773 K", 0.1, 0.1, 0.0, 0.3, 1773.15),
        ("chalk 0.1 mol at 908 K", 0.1, 0.1, 0.0, 0.3, 908.0),
        (
            "ethanol 0.010 mol at 1200 K",
            0.010,
            0.020,
            0.060,
            0.010,
            1200.0,
        ),
        ("Mg 0.05 mol at 500 K", 0.05, 0.0, 0.0, 0.0, 500.0),
    ] {
        let a = air_moles(condensed, c, h, o);
        let n2 = a * 0.78;
        let o2 = a * 0.21;
        let at_t = n2 * h_of("N2", t) + o2 * h_of("O2", t);
        let at_298 = n2 * h_of("N2", 298.15) + o2 * h_of("O2", 298.15);
        let _ = writeln!(
            &mut out,
            "  {what}: air {a:.4} mol (N2 {n2:.4}, O2 {o2:.4}); H(t)={at_t:.1} J, H(298)={at_298:.1} J, \
             sensible = {:.1} J",
            at_t - at_298
        );
    }

    let _ = writeln!(&mut out, "\n=== NASA enthalpies, J/mol ===");
    for name in ["CaCO3(cr)", "CaO(cr)", "CO2", "N2", "O2"] {
        let _ = write!(&mut out, "  {name:12}");
        for t in [298.15, 900.0, 1000.0, 1100.0, 1200.0, 1400.0, 1773.15] {
            let _ = write!(&mut out, " {t:.0}K={:.0}", h_of(name, t));
        }
        let _ = writeln!(&mut out);
    }
    let _ = writeln!(&mut out, "\n=== derived costs for 0.1 mol chalk ===");
    let dh298 = h_of("CaO(cr)", 298.15) + h_of("CO2", 298.15) - h_of("CaCO3(cr)", 298.15);
    let _ = writeln!(&mut out, "  NASA dH_calcination(298) = {:.1} J/mol", dh298);
    for t in [1000.0, 1100.0, 1200.0, 1773.15] {
        let dh = h_of("CaO(cr)", t) + h_of("CO2", t) - h_of("CaCO3(cr)", t);
        let _ = writeln!(&mut out, "  NASA dH_calcination({t:.0}) = {dh:.1} J/mol");
    }
    let _ = writeln!(
        &mut out,
        "  CaO 0.1 mol 298->1773 sensible (NASA) = {:.1} J",
        0.1 * (h_of("CaO(cr)", 1773.15) - h_of("CaO(cr)", 298.15))
    );
    let _ = writeln!(
        &mut out,
        "  CO2 0.1 mol 298->1100 sensible (NASA) = {:.1} J; to 1773 = {:.1} J",
        0.1 * (h_of("CO2", 1100.0) - h_of("CO2", 298.15)),
        0.1 * (h_of("CO2", 1773.15) - h_of("CO2", 298.15))
    );
    let _ = writeln!(
        &mut out,
        "  CaCO3 0.1 mol 298->1773 sensible (NASA) = {:.1} J (registry Cp says {:.1} J)",
        0.1 * (h_of("CaCO3(cr)", 1773.15) - h_of("CaCO3(cr)", 298.15)),
        0.1 * 82.3 * (1773.15 - 298.15)
    );

    panic!("{out}");
}

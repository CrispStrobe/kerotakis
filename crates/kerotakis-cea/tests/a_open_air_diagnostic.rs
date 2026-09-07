//! TEMPORARY diagnostic: the open-vessel air charge, pass by pass.
//!
//! Named to sort before every other test target, because `cargo test`
//! stops at the first failing binary and this one panics on purpose.

use kerotakis_cea::{db, ThermalEquilibrator};
use kerotakis_core::apparatus::BUNSEN_CEILING_K;
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

fn brief(out: &mut String, title: &str, bench: &Bench, v: VesselId, events: &[Event]) {
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
                    "  ENERGY heating={heating} requested={requested_j:.1} \
                     delivered={delivered_j:.1} sensible={sensible_j:.1} passes={passes} \
                     capped={capped}"
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
                    "  EQ at {:.2} K, reaction_energy={reaction_energy_j:?}",
                    temperature.0
                );
            }
            Event::GasEvolved { species, moles, .. } => {
                let _ = writeln!(out, "  GAS {} {:.6} mol", species.0, moles.0);
            }
            Event::Ignited { energy_j, .. } => {
                let _ = writeln!(out, "  IGNITED energy={energy_j:?}");
            }
            Event::Precipitated { species, moles, .. } => {
                let _ = writeln!(out, "  SOLID +{:.6} mol {}", moles.0, species.0);
            }
            Event::Consumed { species, moles, .. } => {
                let _ = writeln!(out, "  CONSUMED {:.6} mol {}", moles.0, species.0);
            }
            _ => {}
        }
    }
    let ves = bench.vessel(v).expect("vessel");
    let _ = writeln!(
        out,
        "  -- final T {:.2} K, Cp {:.3} J/K, sensible {:.1} J",
        ves.temperature.0,
        ves.heat_capacity(),
        ves.enthalpy().0
    );
    for p in &ves.contents {
        let _ = writeln!(out, "  -- holds {:.6} mol {}", p.moles.0, p.species.0);
    }
}

/// `Bench::deliver_remaining_heat`, replayed here so every pass is visible.
fn trace_passes(out: &mut String, title: &str, moles: f64, dose_j: f64) {
    let _ = writeln!(out, "\n=== PASS TRACE {title} ===");
    let mut vessel = Vessel::new(VesselId(0), "crucible");
    vessel.contents.push(Portion {
        species: SpeciesId::new("CaCO3"),
        moles: Moles(moles),
        phase: kerotakis_core::species::Phase::Solid,
    });
    let mut solver = ThermalEquilibrator;
    let mut delivered = 0.0f64;
    for pass in 0..40u32 {
        let cp = vessel.heat_capacity();
        if cp <= 0.0 {
            let _ = writeln!(out, "  pass {pass}: no heat capacity left");
            break;
        }
        let remaining = dose_j - delivered;
        if remaining <= 1e-6 {
            let _ = writeln!(out, "  pass {pass}: dose spent");
            break;
        }
        let before = vessel.temperature.0;
        let room = ((BUNSEN_CEILING_K - before) * cp).min(remaining);
        if room <= 1e-9 {
            let _ = writeln!(out, "  pass {pass}: at the flame, nothing left to give");
            break;
        }
        vessel.temperature = Kelvin(before + room / cp);
        delivered += room;
        let events = match solver.equilibrate(&mut vessel) {
            Ok(e) => e,
            Err(e) => {
                let _ = writeln!(out, "  pass {pass}: SOLVER FAILED {e}");
                break;
            }
        };
        let co2: f64 = events
            .iter()
            .filter_map(|e| match e {
                Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
                _ => None,
            })
            .sum();
        let chalk = vessel.moles_of(&SpeciesId::new("CaCO3")).0;
        let lime = vessel.moles_of(&SpeciesId::new("CaO")).0;
        let _ = writeln!(
            out,
            "  pass {pass}: Cp {cp:.3} J/K, {before:.1} K -> ceiling, took {room:.1} J \
             (total {delivered:.1}); settled at {:.1} K; CO2 {co2:.6}; chalk {chalk:.6}, \
             lime {lime:.6}",
            vessel.temperature.0
        );
    }
    let _ = writeln!(
        out,
        "  TOTAL delivered {delivered:.1} J of {dose_j:.1}; final {:.1} K, Cp {:.3}, \
         sensible {:.1} J",
        vessel.temperature.0,
        vessel.heat_capacity(),
        vessel.enthalpy().0
    );
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

    // The state pass 1 leaves behind, probed at the exact temperatures
    // the HP bisection visits.
    {
        let mut vessel = Vessel::new(VesselId(0), "crucible");
        for (key, moles) in [("CaCO3", 0.052207), ("CaO", 0.047793)] {
            vessel.contents.push(Portion {
                species: SpeciesId::new(key),
                moles: Moles(moles),
                phase: kerotakis_core::species::Phase::Solid,
            });
        }
        vessel.temperature = Kelvin(BUNSEN_CEILING_K);
        let mut ts = vec![250.0, 400.0, 640.0, 1024.0, 1638.0, 2621.0];
        // The bisection's own walk from (250, 6000).
        let (mut lo, mut hi) = (250.0f64, 6000.0f64);
        for _ in 0..12 {
            let mid = 0.5 * (lo + hi);
            ts.push(mid);
            if mid > 1000.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let _ = writeln!(&mut out, "\n=== PROBE: the state pass 1 leaves ===");
        for line in kerotakis_cea::thermal::probe(&vessel, &ts) {
            let _ = writeln!(&mut out, "{line}");
        }
    }

    trace_passes(&mut out, "0.1 mol CaCO3, 40 kJ", 0.1, 40_000.0);
    trace_passes(&mut out, "0.1 mol CaCO3, 5 kJ", 0.1, 5_000.0);

    for (title, kj) in [("A. chalk + 40 kJ", 40.0), ("B. chalk + 5 kJ", 5.0)] {
        let mut bench = Bench::new();
        let mut s = stack();
        add(&mut bench, &mut s, v, "CaCO3", 0.1);
        let events = bench
            .step_with(
                Operator::Heat {
                    vessel: v,
                    energy: Joules(kj * 1000.0),
                    source: None,
                },
                &mut s,
                &PermissiveScreen,
            )
            .expect("heat");
        brief(&mut out, title, &bench, v, &events);
        let delivered = events
            .iter()
            .find_map(|e| match e {
                Event::EnergyTransferred {
                    heating: true,
                    delivered_j,
                    ..
                } => Some(*delivered_j),
                _ => None,
            })
            .unwrap_or(f64::NAN);
        let warming = bench.vessel(v).expect("vessel").enthalpy().0;
        let lime = bench
            .vessel(v)
            .expect("vessel")
            .moles_of(&SpeciesId::new("CaO"))
            .0;
        let chemistry = lime * 178_800.0;
        let _ = writeln!(
            &mut out,
            "  -- CLOSURE delivered {delivered:.1} vs warming {warming:.1} + chemistry \
             {chemistry:.1} = {:.1} -> {:.1} %",
            warming + chemistry,
            100.0 * delivered / (warming + chemistry)
        );
    }

    for (title, key, moles) in [
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
        brief(&mut out, title, &bench, v, &events);
    }
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
        brief(&mut out, "F. 0.05 mol Mg + 1 kJ", &bench, v, &events);
    }

    let _ = writeln!(&mut out, "\n=== NASA enthalpies, J/mol ===");
    for name in ["CaCO3(cr)", "CaO(cr)", "CO2", "N2", "O2"] {
        let _ = write!(&mut out, "  {name:12}");
        for t in [298.15, 900.0, 1000.0, 1100.0, 1200.0, 1400.0, 1773.15] {
            let _ = write!(&mut out, " {t:.0}K={:.0}", h_of(name, t));
        }
        let _ = writeln!(&mut out);
    }

    panic!("{out}");
}

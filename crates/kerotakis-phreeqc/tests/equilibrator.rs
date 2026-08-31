//! The aqueous equilibrator through the bench loop: dissolution,
//! precipitation and solubility limits computed from thermodynamic data, and
//! bounded fuzzing (P0: random inputs → no crash, honest failure state).

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        // CAP-23: the honesty pass stands aside for pairs this rung
        // covers, so every stack that carries Honesty must carry this
        // before it — mirroring all three production stacks.
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
    vessel: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

/// Total moles of one element across every portion of the bench, computed
/// from the inventory (ions, salts, minerals).
fn element_total(bench: &Bench, element: &str) -> f64 {
    let contribution: &[(&str, &[(&str, f64)])] = &[
        ("NaCl", &[("Na", 1.0), ("Cl", 1.0)]),
        ("AgNO3", &[("Ag", 1.0), ("N", 1.0)]),
        ("AgCl", &[("Ag", 1.0), ("Cl", 1.0)]),
        ("Na+", &[("Na", 1.0)]),
        ("Cl-", &[("Cl", 1.0)]),
        ("Ag+", &[("Ag", 1.0)]),
        ("NO3-", &[("N", 1.0)]),
    ];
    bench
        .vessels
        .iter()
        .flat_map(|v| v.contents.iter())
        .map(|p| {
            contribution
                .iter()
                .find(|(k, _)| *k == p.species.0)
                .map(|(_, els)| {
                    els.iter()
                        .filter(|(el, _)| *el == element)
                        .map(|(_, c)| c * p.moles.0)
                        .sum::<f64>()
                })
                .unwrap_or(0.0)
        })
        .sum()
}

#[test]
fn salt_dissolves_in_water() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.55); // ~100 mL
    let events = add(&mut bench, &mut stack, v, "NaCl", 0.05);

    assert!(
        events.iter().any(|e| matches!(e, Event::Dissolved { .. })),
        "salt below saturation must dissolve, got {events:?}"
    );
    let vessel = bench.vessel(v).unwrap();
    assert!(
        !vessel
            .contents
            .iter()
            .any(|p| p.species.0 == "NaCl" && p.phase == Phase::Solid),
        "no solid salt should remain"
    );
    assert!((vessel.moles_of(&SpeciesId::new("Na+")).0 - 0.05).abs() < 1e-6);
    let ph = vessel.solution.clone().expect("characterised").ph;
    assert!((ph - 7.0).abs() < 0.5, "NaCl solution is neutral, pH {ph}");
}

#[test]
fn halite_stops_dissolving_at_saturation() {
    // 8 mol NaCl into 1 kg water: solubility is ~6.1 mol/kgw, the rest must
    // stay solid — a computed solubility limit, not a scripted one.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51); // 1.000 kg
    add(&mut bench, &mut stack, v, "NaCl", 8.0);

    let vessel = bench.vessel(v).unwrap();
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "NaCl" && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    let dissolved = vessel.moles_of(&SpeciesId::new("Na+")).0;
    assert!(
        solid > 0.5 && dissolved > 5.0 && dissolved < 7.5,
        "expected ~6.1 mol/kgw dissolved with the rest solid; got {dissolved} dissolved, {solid} solid"
    );
    assert!((solid + dissolved - 8.0).abs() < 1e-6, "sodium conserved");
}

#[test]
fn silver_nitrate_plus_salt_precipitates_silver_chloride() {
    // The marquee sequence of the whole product.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaCl", 0.01);
    let events = add(&mut bench, &mut stack, v, "AgNO3", 0.01);

    let precipitated = events.iter().find_map(|e| match e {
        Event::Precipitated { species, moles, .. } if species.0 == "AgCl" => Some(moles.0),
        _ => None,
    });
    let agcl = precipitated.expect("AgCl must precipitate");
    assert!(
        agcl > 0.0098 && agcl <= 0.01,
        "nearly all silver precipitates as AgCl, got {agcl} mol"
    );

    // Conservation: every atom of Ag and Cl still on the bench.
    assert!((element_total(&bench, "Ag") - 0.01).abs() < 1e-6);
    assert!((element_total(&bench, "Cl") - 0.01).abs() < 1e-6);
    assert!((element_total(&bench, "Na") - 0.01).abs() < 1e-6);

    // And the child register says the famous line.
    let text = render_event(
        &Event::Precipitated {
            vessel: v,
            species: SpeciesId::new("AgCl"),
            moles: Moles(agcl),
        },
        Register::LV1,
    );
    assert!(text.contains("cloudy") && text.contains("white"), "{text}");

    // GUI-092, the other half of the same beaker: the net ionic equation,
    // derived from the speciation *this* solve produced.
    //
    // The derivation is unit-tested against `SolutionInfo` fixtures in the
    // core; this asserts that those fixtures are the shape the solver
    // really returns. It is worth its own assertions here rather than a
    // separate solve because the first two attempts failed for reasons no
    // fixture would have caught — a trace floor of our own hid the
    // depleted silver, and the neutral AgCl(aq) complex outranked the free
    // ion — so the whole species distribution is printed on failure. The
    // next person to see this red should not have to guess.
    let speciation = || -> String {
        match bench.vessels[0].solution.as_ref() {
            None => "no solver characterised this vessel".to_string(),
            Some(s) if s.species.is_empty() => "solution present, species list EMPTY".to_string(),
            Some(s) => s
                .species
                .iter()
                .map(|d| format!("{}={:.3e}", d.name, d.molality))
                .collect::<Vec<_>>()
                .join(" "),
        }
    };
    let derived = kerotakis_core::net_ionic_for(&events, &bench.vessels);
    let net = derived
        .first()
        .unwrap_or_else(|| panic!("no net ionic equation from: {}", speciation()));
    assert_eq!(
        net.equation,
        "Ag⁺(aq) + Cl⁻(aq) → AgCl(s)",
        "speciation was: {}",
        speciation()
    );

    // The spectators are the whole point: named as such, and absent from
    // the equation.
    let named: Vec<&str> = net
        .reactants
        .iter()
        .chain(net.products.iter())
        .map(|t| t.species.as_str())
        .collect();
    assert!(
        !named.contains(&"Na+") && !named.contains(&"NO3-"),
        "the spectators are in the equation: {}",
        net.equation
    );
    let spectators: Vec<&str> = net.spectators.iter().map(|t| t.species.as_str()).collect();
    assert!(
        spectators.contains(&"Na+") && spectators.contains(&"NO3-"),
        "sodium and nitrate must be named as spectators, got {spectators:?} from: {}",
        speciation()
    );
}

#[test]
fn ethanol_bench_answers_with_the_curated_verdict() {
    // CAP-23 superseded the old expectation here: salt in ethanol used
    // to demand an unmodelled flag, but "NaCl is practically insoluble
    // in ethanol" is a handbook answer, not a gap. The aqueous solver
    // still declines (no solution is claimed) and the verdict comes
    // from the non-aqueous rung with numbers.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "ethanol", 1.0);
    let events = add(&mut bench, &mut stack, v, "NaCl", 0.01);

    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::DissolvedInSolvent { species, .. } if species.0 == "NaCl"
        )),
        "salt in ethanol gets the handbook verdict, got {events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::NotYetModeled { .. })),
        "an answered pair must not also apologise, got {events:?}"
    );
    assert!(
        bench.vessel(v).unwrap().solution.is_none(),
        "no aqueous solution is claimed for an organic phase"
    );
}

/// Bounded fuzz (P0): random additions of mapped species in random amounts —
/// the loop must never panic, and every accepted step either equilibrates or
/// fails honestly while conserving elements.
#[test]
fn fuzz_random_aqueous_benches() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut stack = stack();
    let keys = ["water", "NaCl", "AgNO3", "Ag+", "Cl-", "Na+", "NO3-"];
    for seed in 0u64..200 {
        let mut bench = Bench::new();
        let mut expected = std::collections::BTreeMap::new();
        for step in 0..6 {
            let mut h = DefaultHasher::new();
            (seed, step).hash(&mut h);
            let r = h.finish();
            let key = keys[(r % keys.len() as u64) as usize];
            let amount = ((r >> 8) % 10_000) as f64 / 1000.0 + 1e-4; // 0.0001..10 mol
            let events = bench
                .step_with(
                    Operator::Add {
                        vessel: VesselId(0),
                        species: SpeciesId::new(key),
                        moles: Moles(amount),
                        at: None,
                    },
                    &mut stack,
                    &PermissiveScreen,
                )
                .expect("step never hard-errors for valid species");
            // Solver failures are allowed; panics and silent element loss
            // are not.
            let _ = &events;
            *expected.entry(key).or_insert(0.0) += amount;
        }
        for (el, per_key) in [
            ("Na", vec![("NaCl", 1.0), ("Na+", 1.0)]),
            ("Ag", vec![("AgNO3", 1.0), ("Ag+", 1.0)]),
        ] {
            let want: f64 = per_key
                .iter()
                .map(|(k, c)| c * expected.get(*k).copied().unwrap_or(0.0))
                .sum();
            let got = element_total(&bench, el);
            assert!(
                (got - want).abs() < 1e-6 * want.max(1.0),
                "seed {seed}: element {el} drifted: want {want}, got {got}"
            );
        }
    }
}

/// Enthalpy is a state function: the same two salts, dissolved in either
/// order, must leave the beaker at the same temperature.
///
/// This failed for a long time, and the failure was invisible because the
/// chemistry was right. Mineral phases are discovered per database by
/// formula match, and Sylvite exists only in `pitzer.dat`. A phase the
/// routed database does not define is dissolved into element totals
/// instead — correct, and deliberate — but no `Dissolved` event was
/// recorded for it, and the dissolution enthalpy rides on the event.
///
/// Since the router picks a database by ionic strength, potassium chloride
/// therefore cooled the beaker only when something else had already made
/// the solution concentrated enough to route to pitzer. KCl-then-NaCl
/// ended 0.82 K warmer than NaCl-then-KCl, and that gap was also the whole
/// of a long-standing order-dependent pH: 0.82 K against water's
/// dpH/dT ≈ -0.0163/K is 1.34e-2, and the observed drift was 1.36e-2.
/// Holding the temperature equal collapsed it to 2.8e-10.
///
/// The residual tolerance below is not slack for that bug — it is three
/// orders larger than what remains (~4 mK), which comes from applying
/// ΔT = q/cp incrementally while cp grows with each addition.
#[test]
fn dissolution_heat_does_not_depend_on_order() {
    let mut s = stack();
    let mut final_t = Vec::new();
    let (salt, potash) = (("NaCl", 0.05), ("KCl", 0.02));
    for order in [[salt, potash], [potash, salt]] {
        let mut bench = Bench::new();
        let v = VesselId(0);
        add(&mut bench, &mut s, v, "water", 5.534_276_991_396_059);
        for (species, moles) in order {
            add(&mut bench, &mut s, v, species, moles);
        }
        final_t.push(bench.vessel(v).expect("vessel").temperature.to_celsius());
    }
    let drift = (final_t[0] - final_t[1]).abs();
    assert!(
        drift < 0.05,
        "dissolving the same salts in a different order changed the final \
         temperature by {drift:.4} K ({:.4} vs {:.4}) — enthalpy is not \
         behaving as a state function",
        final_t[0],
        final_t[1],
    );
}

/// Hess's law, with a reagent arriving into a beaker that is no longer at
/// room temperature.
///
/// The sibling test above swaps two salts into cold water, where every
/// addition happens at 25 °C and the mixing term is zero. This one is the
/// case that broke: caustic soda warms the beaker by ten degrees, and the
/// acid then enters at 25 °C into a hot solution. Adding the acid first
/// gave 35.68 °C and adding it second gave 35.49 °C — same reagents, same
/// final solution, 0.19 K apart.
///
/// The cause was heat capacity appearing and vanishing. Dissolved matter
/// carries none in this model, so an acid credited with a liquid's Cp while
/// it mixed was stripped of it the moment the solver called it chloride,
/// and the sensible heat that Cp was holding went with it. Balancing
/// enthalpy across the solve — a relabelling of matter cannot change an
/// adiabatic vessel's enthalpy — gives it back, and both orders land on
/// T₀ + q/Cp(water).
///
/// NOTE the number this test does *not* check. Neutralisation enthalpy is
/// not modelled, so both paths are short of a real bench by about 13 K
/// (−57.3 kJ/mol over 0.1 mol). Hess's law holding is a statement about the
/// two paths agreeing, and they now agree exactly; it is not yet a claim
/// that either is the right temperature.
#[test]
fn hess_holds_when_a_reagent_arrives_into_a_warm_beaker() {
    let mut s = stack();
    let mut final_t = Vec::new();
    let (acid, base) = (("HCl", 0.1), ("NaOH", 0.100_007_5));
    for order in [[acid, base], [base, acid]] {
        let mut bench = Bench::new();
        let v = VesselId(0);
        add(&mut bench, &mut s, v, "water", 5.534_276_991_396_059);
        for (species, moles) in order {
            add(&mut bench, &mut s, v, species, moles);
        }
        final_t.push(bench.vessel(v).expect("vessel").temperature.to_celsius());
    }
    let drift = (final_t[0] - final_t[1]).abs();
    assert!(
        drift < 1e-6,
        "acid-then-base and base-then-acid must reach the same temperature: \
         {:.6} vs {:.6}, {drift:.2e} K apart",
        final_t[0],
        final_t[1],
    );
}

/// The heat of neutralisation is counted, and counted only once.
///
/// `H⁺ + OH⁻ → H₂O` never appears as a reaction PHREEQC reports — it is
/// handed element totals and cannot tell an acid just added from one that
/// was always there — so its heat was simply absent. The classic school
/// experiment came out 13.7 K cold, silently.
///
/// The enthalpy is read from the routed database rather than curated:
/// `DELTA_H_SPECIES("OH-")` is the reverse of the reaction defining OH-,
/// and the three datasets answer -55.91, -55.81 and -56.36 kJ/mol against a
/// literature -55.8. The extent comes from the solutes' net charge, which
/// is the vessel's unspent acidity: what cancels is the overlap of the acid
/// already there with the base arriving.
///
/// The third case is the one that catches a naive extent. Acid onto acid
/// must release nothing — a formula that merely watched |charge| change
/// would happily invent heat for it.
#[test]
fn neutralisation_heat_is_counted_once() {
    let warm = |steps: &[(&str, f64)]| -> f64 {
        let mut s = stack();
        let mut bench = Bench::new();
        let v = VesselId(0);
        add(&mut bench, &mut s, v, "water", 5.534_276_991_396_059);
        for (species, moles) in steps {
            add(&mut bench, &mut s, v, species, *moles);
        }
        bench.vessel(v).expect("vessel").temperature.to_celsius()
    };

    let acid_then_base = warm(&[("HCl", 0.1), ("NaOH", 0.100_007_5)]);
    let base_then_acid = warm(&[("NaOH", 0.100_007_5), ("HCl", 0.1)]);
    assert!(
        (acid_then_base - 49.2).abs() < 0.5,
        "0.1 mol neutralised should warm the beaker to about 49 °C, not {acid_then_base:.2}"
    );
    assert!(
        (acid_then_base - base_then_acid).abs() < 1e-6,
        "and both orders must agree: {acid_then_base:.6} vs {base_then_acid:.6}"
    );

    // More acid into acid neutralises nothing.
    let acid_alone = warm(&[("HCl", 0.05)]);
    let more_acid = warm(&[("HCl", 0.05), ("HCl", 0.05)]);
    assert!(
        (more_acid - acid_alone).abs() < 0.05,
        "adding acid to acid must release no neutralisation heat: {acid_alone:.3} then \
         {more_acid:.3}"
    );
}

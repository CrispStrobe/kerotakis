//! Metal reactivity in solution: displacement by every pair of the series,
//! amphoteric metals in alkali, and notes that agree with what happened.
//!
//! A sweep of the bench reported three gaps. Displacement between series
//! metals (H1) is pinned here pair by pair, in both orders of addition, with
//! the negative cells beside the positive ones. Aluminium and zinc in
//! sodium hydroxide (H2) go through [`kerotakis_core::amphoteric`]. And a
//! metal that has just reacted must not, in the same step, be told it
//! "stays as the metal" or "does not react" for a reason that its own
//! reaction contradicts (H5).

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    let aqueous = PhreeqcEquilibrator::new().expect("engine");
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(DisplacementEquilibrator::wrapping(Box::new(aqueous))),
        Box::new(StateEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(bench: &mut Bench, stack: &mut SolverStack, key: &str, moles: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &PermissiveScreen,
        )
        .expect("step")
}

/// 100 mL of water, then each addition in turn; the last step's events.
fn run(additions: &[(&str, f64)]) -> (Bench, Vec<Event>) {
    let mut stack = stack();
    let mut bench = Bench::new();
    add(&mut bench, &mut stack, "water", 5.55);
    let mut last = Vec::new();
    for (key, moles) in additions {
        last = add(&mut bench, &mut stack, key, *moles);
    }
    (bench, last)
}

fn moles_of(bench: &Bench, key: &str, phase: Phase) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == phase)
        .map(|p| p.moles.0)
        .sum()
}

fn keys(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Inert {
                reason: Some(p), ..
            } => Some(p.key.clone()),
            Event::NotYetModeled {
                reason: Some(p), ..
            } => Some(p.key.clone()),
            _ => None,
        })
        .collect()
}

fn hydrogen(events: &[Event]) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. }
            | Event::GasContained { species, moles, .. }
                if species.0 == "H2" =>
            {
                Some(moles.0)
            }
            _ => None,
        })
        .sum()
}

fn plated(events: &[Event], metal: &str, onto: &str) -> bool {
    events.iter().any(|e| {
        matches!(e, Event::Plated { species, onto: o, .. } if species.0 == metal && o.0 == onto)
    })
}

/// The positive cells of the grid, each in both orders of addition: the
/// metal into the salt solution, and the salt onto the metal standing in
/// water.
#[test]
fn a_metal_displaces_the_ion_of_every_nobler_metal_in_either_order() {
    for (metal, salt, noble, n_metal) in [
        ("Zn", "CuSO4", "Cu", 0.01),
        ("Fe", "CuSO4", "Cu", 0.01),
        ("Mg", "CuSO4", "Cu", 0.01),
        ("Cu", "AgNO3", "Ag", 0.01),
        ("Zn", "AgNO3", "Ag", 0.01),
    ] {
        for order in [
            [(metal, n_metal), (salt, 0.005)],
            [(salt, 0.005), (metal, n_metal)],
        ] {
            let (bench, events) = run(&order);
            assert!(
                plated(&events, noble, metal),
                "{metal} + {salt} ({order:?}) plates {noble}: {events:?}"
            );
            assert!(
                moles_of(&bench, noble, Phase::Solid) > 0.004,
                "{metal} + {salt}: {noble} metal in the beaker"
            );
            // The step that did the plating says nothing that denies it.
            let said = keys(&events);
            assert!(
                !said.iter().any(|k| k == "not-modeled.stays-as-the-metal"
                    || k == "inert.hydrogen-overpotential"
                    || k == "inert.uphill-in-series"),
                "{metal} + {salt} ({order:?}): contradicting note {said:?}"
            );
        }
    }
}

/// The negative cells: the nobler metal does not take the ion of the more
/// reactive one, and a metal above hydrogen makes no hydrogen in acid.
#[test]
fn the_negative_cells_stay_negative() {
    let (bench, events) = run(&[("Cu", 0.01), ("ZnSO4", 0.01)]);
    assert!(
        !events.iter().any(|e| matches!(e, Event::Plated { .. })),
        "{events:?}"
    );
    assert!((moles_of(&bench, "Cu", Phase::Solid) - 0.01).abs() < 1e-12);
    assert!(keys(&events).iter().any(|k| k == "inert.uphill-in-series"));

    let (bench, events) = run(&[("Ag", 0.01), ("CuSO4", 0.01)]);
    assert!(
        !events.iter().any(|e| matches!(e, Event::Plated { .. })),
        "{events:?}"
    );
    assert!((moles_of(&bench, "Ag", Phase::Solid) - 0.01).abs() < 1e-12);

    let (bench, events) = run(&[("Ag", 0.01), ("HCl", 0.01)]);
    assert_eq!(hydrogen(&events), 0.0, "{events:?}");
    assert!((moles_of(&bench, "Ag", Phase::Solid) - 0.01).abs() < 1e-12);
    assert!(keys(&events).iter().any(|k| k == "inert.above-hydrogen"));
}

/// Magnesium in hydrochloric acid, in either order: 5 mmol of hydrogen for
/// 5 mmol of magnesium, and none of the "nothing happens here" sentences in
/// the step that made it.
#[test]
fn magnesium_in_acid_is_never_also_told_it_stays_a_metal() {
    for order in [
        [("Mg", 0.005), ("HCl", 0.01)],
        [("HCl", 0.01), ("Mg", 0.005)],
    ] {
        let (_, events) = run(&order);
        assert!(
            (hydrogen(&events) - 0.005).abs() < 1e-4,
            "{order:?}: {events:?}"
        );
        let said = keys(&events);
        assert!(
            !said.iter().any(|k| k == "not-modeled.stays-as-the-metal"),
            "{order:?}: {said:?}"
        );
        for e in &events {
            if let Event::Inert { species, .. } = e {
                assert_ne!(species.0, "Mg", "magnesium reacted: {e:?}");
            }
        }
    }
}

/// Aluminium foil in sodium hydroxide: it dissolves as aluminate and gives
/// three hydrogens for every two aluminiums, and the step warms the water.
#[test]
fn aluminium_dissolves_in_sodium_hydroxide_with_hydrogen() {
    let (bench, events) = run(&[("NaOH", 0.02), ("Al", 0.01)]);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::ReactionOccurred { equation, .. } if equation == "2 Al + 2 OH- + 6 H2O → 2 Al(OH)4- + 3 H2"
        )),
        "{events:?}"
    );
    assert!(
        moles_of(&bench, "Al", Phase::Solid) < 1e-9,
        "all the foil is gone"
    );
    assert!((hydrogen(&events) - 0.015).abs() < 1e-9, "{events:?}");
    assert!((moles_of(&bench, "Al+3", Phase::Aqueous) - 0.01).abs() < 1e-9);
    let warmed = events
        .iter()
        .any(|e| matches!(e, Event::TemperatureChanged { from, to, .. } if to.0 > from.0 + 1.0));
    assert!(warmed, "the reaction is strongly exothermic: {events:?}");
    let said = keys(&events);
    assert!(
        !said.iter().any(|k| k == "not-modeled.metal-outside-series"),
        "{said:?}"
    );

    // The other order: foil first, caustic soda onto it.
    let (bench, events) = run(&[("Al", 0.01), ("NaOH", 0.02)]);
    assert!((hydrogen(&events) - 0.015).abs() < 1e-9, "{events:?}");
    assert!(moles_of(&bench, "Al", Phase::Solid) < 1e-9);
}

/// An excess of aluminium stops when the free hydroxide is bound: one per
/// aluminium for Al(OH)₄⁻.
#[test]
fn aluminium_in_excess_stops_when_the_base_is_spent() {
    let (bench, events) = run(&[("NaOH", 0.004), ("Al", 0.01)]);
    let left = moles_of(&bench, "Al", Phase::Solid);
    assert!(
        (left - 0.006).abs() < 1e-6,
        "aluminium left {left}: {events:?}"
    );
    assert!((hydrogen(&events) - 0.006).abs() < 1e-6, "{events:?}");
}

/// Without alkali the hydroxo route does not run: aluminium in water stays,
/// and in acid it is still the series' unlisted metal.
#[test]
fn aluminium_in_water_or_acid_is_not_the_alkali_route() {
    let (bench, events) = run(&[("Al", 0.01)]);
    assert_eq!(hydrogen(&events), 0.0);
    assert!((moles_of(&bench, "Al", Phase::Solid) - 0.01).abs() < 1e-12);

    let (bench, events) = run(&[("HCl", 0.01), ("Al", 0.01)]);
    assert_eq!(hydrogen(&events), 0.0, "{events:?}");
    assert!((moles_of(&bench, "Al", Phase::Solid) - 0.01).abs() < 1e-12);
    assert!(keys(&events)
        .iter()
        .any(|k| k == "not-modeled.metal-outside-series"));
}

/// Zinc in cold dilute caustic soda: thermodynamically downhill, kinetically
/// blocked by hydrogen's overpotential on zinc — said as such, not as
/// "stays as the metal, nothing dissolved here".
#[test]
fn zinc_in_cold_dilute_alkali_is_told_it_is_kinetically_blocked() {
    let (bench, events) = run(&[("NaOH", 0.02), ("Zn", 0.01)]);
    assert_eq!(hydrogen(&events), 0.0, "{events:?}");
    assert!((moles_of(&bench, "Zn", Phase::Solid) - 0.01).abs() < 1e-12);
    let said = keys(&events);
    assert!(
        said.iter().any(|k| k == "inert.amphoteric-overpotential"),
        "{said:?}"
    );
    assert!(
        !said.iter().any(|k| k == "not-modeled.stays-as-the-metal"),
        "{said:?}"
    );
}

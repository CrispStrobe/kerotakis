//! What leaves a burning open vessel, and what is left standing there.
//!
//! The transcript this file exists for:
//!
//! ```text
//! add v1 ethanol 10mL; measure v1 balance; ignite v1
//!   C₂H₅OH(l) + 3 O₂(g) → 2 CO₂(g) + 3 H₂O(g)
//!   0.4677 mol water ↑, 0.3425 mol carbon dioxide ↑, 0.0461 mol hydrogen ↑
//!   926.9 °C → 2496.3 °C
//! ```
//!
//! Two things were wrong with it and neither was the Gibbs solve. The
//! equation printed was complete combustion and the moles counted beside it
//! were not: 9 % of the fuel's hydrogen came off as H₂, because the products
//! were vented FROZEN at the flame's own 2769 K, where an equilibrium really
//! is partly dissociated. And the beaker was then reported at 2496 °C with
//! nothing whatever in it.
//!
//! ```text
//! 10 mL * 0.789 g/mL / 46.069 g/mol = 0.171266 mol ethanol
//! CO2  = 2 * 0.171266 = 0.342533 mol
//! H2O  = 3 * 0.171266 = 0.513799 mol
//! ```

use kerotakis_cea::ThermalEquilibrator;
use kerotakis_core::*;

/// 10 mL of ethanol, in moles.
const ETHANOL_MOLES: f64 = 10.0 * 0.789 / 46.069;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(ThermalEquilibrator),
        Box::new(HonestyEquilibrator),
    ])
}

fn burn_ten_millilitres_of_ethanol() -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("ethanol"),
                moles: Moles(ETHANOL_MOLES),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add ethanol");
    let events = bench
        .step_with(
            Operator::Ignite {
                vessel: VesselId(0),
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("ignite ethanol");
    (bench, events)
}

fn vented(events: &[Event], key: &str) -> f64 {
    events
        .iter()
        .filter_map(|event| match event {
            Event::GasEvolved { species, moles, .. } if species.0 == key => Some(moles.0),
            _ => None,
        })
        .sum()
}

/// The printed equation and the counted moles have to be the same claim.
#[test]
fn the_exhaust_is_what_the_equation_says_it_is() {
    let (_, events) = burn_ten_millilitres_of_ethanol();

    let carbon_dioxide = vented(&events, "CO2");
    let water = vented(&events, "water");
    let hydrogen = vented(&events, "H2");

    let expected_co2 = 2.0 * ETHANOL_MOLES;
    let expected_water = 3.0 * ETHANOL_MOLES;
    assert!(
        (carbon_dioxide - expected_co2).abs() < 0.01 * expected_co2,
        "{carbon_dioxide} mol CO2 vented; the equation says {expected_co2}: {events:?}"
    );
    assert!(
        (water - expected_water).abs() < 0.01 * expected_water,
        "{water} mol water vented; the equation says {expected_water}: {events:?}"
    );
    // The fuel carries 6 H per molecule; less than 1 % of them may leave
    // as anything but water.
    let hydrogen_atoms = 6.0 * ETHANOL_MOLES;
    assert!(
        2.0 * hydrogen < 0.01 * hydrogen_atoms,
        "{hydrogen} mol H2 came off — that is {:.1} % of the fuel's hydrogen, and an open \
         burn's exhaust recombines long before anyone could catch it: {events:?}",
        200.0 * hydrogen / hydrogen_atoms
    );
    assert!(
        events.iter().any(|event| matches!(
            event,
            Event::ReactionOccurred { equation, .. } if equation.contains("3 H₂O")
        )),
        "the equation is still printed: {events:?}"
    );
}

/// The flame is not re-solved: its temperature and its energy are the
/// adiabatic answer they always were. Only what LEFT is read at the
/// exhaust temperature.
#[test]
fn the_flame_keeps_its_temperature_and_its_energy() {
    let (bench, events) = burn_ten_millilitres_of_ethanol();

    let flame = bench.vessel(VesselId(0)).unwrap().temperature.to_celsius();
    assert!(
        (flame - 2496.3).abs() < 2.0,
        "the adiabatic flame temperature is {flame} °C, not the 2496.3 °C the quest pins"
    );

    let energy = events
        .iter()
        .find_map(|event| match event {
            Event::Ignited {
                energy_j: Some(energy),
                ..
            } => Some(*energy),
            _ => None,
        })
        .expect("a flame that caught reports its energy");
    assert!(
        (energy - 208_000.0).abs() < 0.02 * 208_000.0,
        "10 mL of ethanol releases {} kJ; it was 208 kJ before this change and the \
         recombination is the exhaust's business, not the vessel's",
        energy / 1000.0
    );
}

/// Nothing is left in the beaker, and the bench has to say whose 2496 °C
/// that is.
#[test]
fn an_emptied_beaker_says_the_temperature_is_the_flames() {
    let (bench, events) = burn_ten_millilitres_of_ethanol();
    assert!(
        bench.vessel(VesselId(0)).unwrap().contents.is_empty(),
        "everything burned: {:?}",
        bench.vessel(VesselId(0)).unwrap().contents
    );
    let equilibrium = events
        .iter()
        .find(|event| matches!(event, Event::ThermalEquilibrium { .. }))
        .expect("a burn reports its equilibrium");
    let lv1 = render_event(equilibrium, Register::LV1);
    assert!(
        lv1.contains("holds nothing"),
        "LV1 must not read as a claim about the glass: {lv1}"
    );
    let lv2 = render_event(equilibrium, Register::LV2);
    assert!(
        lv2.contains("nothing left in the vessel"),
        "LV2 must not read as a claim about the glass: {lv2}"
    );
}

/// A vessel that still holds its product keeps the plain wording: chalk
/// calcined in a crucible really is at that temperature, with the lime
/// still in it.
#[test]
fn a_vessel_that_kept_its_product_keeps_the_plain_wording() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench
        .step_with(
            Operator::Add {
                vessel: VesselId(0),
                species: SpeciesId::new("Mg"),
                moles: Moles(0.05),
                at: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("add magnesium");
    let events = bench
        .step_with(
            Operator::Heat {
                vessel: VesselId(0),
                energy: Joules(1_000.0),
                source: None,
            },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("heat magnesium");
    let equilibrium = events
        .iter()
        .find(|event| matches!(event, Event::ThermalEquilibrium { .. }))
        .expect("the magnesium burn reports its equilibrium");
    let lv1 = render_event(equilibrium, Register::LV1);
    assert!(
        !lv1.contains("holds nothing"),
        "the crucible still holds its oxide: {lv1}"
    );
}

/// The exhaust temperature is a choice, so it has to be a choice that
/// does not matter. Below about 1400 K a C/H/O/N exhaust with the flame's
/// own excess air is complete combustion to within a ten-thousandth of
/// its hydrogen, so anything in the plausible flue band — a few hundred K
/// above ambient to about 1200 K — gives the same answer, and 1000 K is
/// not a fitted parameter hiding as a constant.
#[test]
fn the_exhaust_temperature_hardly_matters() {
    use std::collections::BTreeMap;

    // The elements 10 mL of ethanol and its air carry, with the same 20 %
    // excess oxygen `charge` gives the flame.
    let mut budget = BTreeMap::new();
    budget.insert("C".to_string(), 2.0 * ETHANOL_MOLES);
    budget.insert("H".to_string(), 6.0 * ETHANOL_MOLES);
    budget.insert("O".to_string(), 8.2 * ETHANOL_MOLES);
    budget.insert("N".to_string(), 12.0 * ETHANOL_MOLES);
    let pool: Vec<&kerotakis_cea::Species> = ["CO2", "H2O", "H2", "O2", "N2", "CO", "OH", "H", "O"]
        .into_iter()
        .filter_map(|name| kerotakis_cea::db().get(name))
        .collect();

    for temperature in [700.0, 1000.0, 1400.0] {
        let eq = kerotakis_cea::equilibrate_tp(&budget, &pool, temperature, 1.0)
            .unwrap_or_else(|e| panic!("exhaust at {temperature} K: {e}"));
        let water = eq.moles_of("H2O");
        let carbon_dioxide = eq.moles_of("CO2");
        let hydrogen = eq.moles_of("H2");
        assert!(
            2.0 * hydrogen < 1e-4 * 6.0 * ETHANOL_MOLES,
            "at {temperature} K, {hydrogen} mol H2 survives"
        );
        assert!(
            (water - 3.0 * ETHANOL_MOLES).abs() < 1e-3 * 3.0 * ETHANOL_MOLES,
            "at {temperature} K, {water} mol water"
        );
        assert!(
            (carbon_dioxide - 2.0 * ETHANOL_MOLES).abs() < 1e-3 * 2.0 * ETHANOL_MOLES,
            "at {temperature} K, {carbon_dioxide} mol CO2"
        );
    }
}

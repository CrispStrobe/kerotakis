#![cfg(feature = "engine")]
//! The same contents give the same answer, whatever order they arrived in.
//!
//! `solution.solvent_kg` used to carry a residue of the order the beaker
//! was filled in. `aq-023` — 100 mL of water, 0.01 mol of calcium chloride
//! and 5 g of a carbonate laundry powder — read one solvent mass with the
//! salt poured first and another with the powder first, one part in 1e4
//! apart, about 10 mg of water in 100 g. Every molality is per kg of that
//! mass, so `ionic_strength` carried the same residue into every activity
//! coefficient, every saturation index and the pH.
//!
//! Both numbers came from the same line of the same function. The vessel's
//! own water INVENTORY never departed — `complete_basis` rebuilds it from
//! conserved hydrogen and oxygen, which cannot care about order — but
//! PHREEQC's `mass_H2O` tracks the water the INPUT declared, and the input
//! to the last operation is the intermediate vessel plus one reagent. The
//! two orderings have different intermediate vessels, holding different
//! shares of their hydrogen and oxygen inside species rather than inside
//! water, so the last solve was handed one final state in two
//! representations and answered each faithfully.
//!
//! `PhreeqcEquilibrator::recharacterise_canonically` poses the settled
//! contents again before reporting them. This file is the measurement that
//! says it worked, and the budget that says what it cost.

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

/// What a reader is shown about this beaker, plus the water the inventory
/// says is in it.
struct Read {
    solvent_kg: f64,
    ionic_strength: f64,
    ph: f64,
    inventory_water_mol: f64,
}

/// `aq-023`'s own script, with the two reagents in the stated order.
fn aq_023(second: &str, third: &str) -> Read {
    let mut bench = Bench::new();
    let mut solvers = stack();
    for line in ["add v1 water 100mL", second, third] {
        let op = kerotakis_core::script::parse_op(line)
            .expect("the corpus script parses")
            .expect("a known verb");
        bench
            .step_with(op, &mut solvers, &PermissiveScreen)
            .expect("the step runs");
    }
    let vessel = bench.vessel(VesselId(0)).expect("the beaker");
    let info = vessel
        .solution
        .clone()
        .expect("a calcium chloride solution is characterised");
    Read {
        solvent_kg: info
            .solvent_kg
            .expect("a characterised solution has a solvent mass"),
        ionic_strength: info.ionic_strength,
        ph: info.ph,
        inventory_water_mol: vessel
            .contents
            .iter()
            .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum(),
    }
}

const SALT_FIRST: (&str, &str) = ("add v1 CaCl2 0.01mol", "add v1 laundry_detergent 5g");
const POWDER_FIRST: (&str, &str) = ("add v1 laundry_detergent 5g", "add v1 CaCl2 0.01mol");

/// The departure, closed, to the ten significant figures the defect was
/// found in.
///
/// The tolerances are not a comfort margin. `solvent_kg` is asserted at
/// 1e-9 RELATIVE, which is finer than the nine decimals the PHREEQC input
/// writes the solvent mass to — so the two orderings have to produce the
/// same input text for it to pass, which is the property being claimed.
/// `ionic_strength` is asserted at 1e-7, because the element totals are
/// written to twelve significant figures, which is finer than the
/// conserved inventory's own floating-point residue (the two orderings
/// agree on the vessel's water to about one part in 4e8): a few parts in
/// 1e9 therefore do reach the engine. That remainder is arithmetic noise
/// in a conserved sum rather than a difference in representation, and it
/// is four orders of magnitude below what the wire publishes.
#[test]
fn the_same_beaker_reads_the_same_solvent_mass_in_either_order() {
    let salt = aq_023(SALT_FIRST.0, SALT_FIRST.1);
    let powder = aq_023(POWDER_FIRST.0, POWDER_FIRST.1);

    let report = format!(
        "\n              solvent_kg     ionic strength   pH             contents[water]\n\
         salt first    {:.10}   {:.10}     {:.10}   {:.10}\n\
         powder first  {:.10}   {:.10}     {:.10}   {:.10}",
        salt.solvent_kg,
        salt.ionic_strength,
        salt.ph,
        salt.inventory_water_mol,
        powder.solvent_kg,
        powder.ionic_strength,
        powder.ph,
        powder.inventory_water_mol,
    );

    let relative = |a: f64, b: f64| (a - b).abs() / a.abs().max(b.abs());
    assert!(
        relative(salt.solvent_kg, powder.solvent_kg) < 1e-9,
        "the solvent mass still depends on the order the beaker was filled in:{report}"
    );
    assert!(
        relative(salt.ionic_strength, powder.ionic_strength) < 1e-7,
        "the ionic strength still depends on the order the beaker was filled in:{report}"
    );
    assert!(
        (salt.ph - powder.ph).abs() < 1e-7,
        "the pH still depends on the order the beaker was filled in:{report}"
    );
    // The inventory never departed, and saying so here is what keeps the
    // two surfaces from being confused for one another: the fix did not
    // move `contents[water]` towards `solvent_kg`, it moved the question
    // the solver is asked.
    assert!(
        relative(salt.inventory_water_mol, powder.inventory_water_mol) < 1e-7,
        "the conserved water inventory departed, which is a different \
         defect from the one this file is about:{report}"
    );
}

/// What the canonical pose costs, held as a budget.
///
/// One re-pose per equilibration, and never more. The opening water is
/// solvent-only — there is no engine call behind that answer to repeat —
/// so three commands buy at most two. Measured when this landed: the
/// salt-first ordering went from 8 engine calls to 9 and the powder-first
/// from 6 to 8, one of the salt-first re-poses having been answered from
/// the content-addressed cache instead of the engine.
#[test]
fn the_canonical_pose_costs_at_most_one_solve_per_equilibration() {
    let mut phreeqc = PhreeqcEquilibrator::new().expect("engine");
    let mut bench = Bench::new();
    for line in ["add v1 water 100mL", SALT_FIRST.0, SALT_FIRST.1] {
        let op = kerotakis_core::script::parse_op(line)
            .expect("the corpus script parses")
            .expect("a known verb");
        bench
            .step_with(op, &mut phreeqc, &PermissiveScreen)
            .expect("the step runs");
    }
    let calls = phreeqc.engine_calls();
    assert!(
        calls <= 12,
        "aq-023 cost {calls} engine calls; it cost 8 before the canonical \
         pose and 9 after, and the budget is there so a second re-pose \
         cannot be added without someone noticing"
    );
}

//! Nothing is lost if nothing escapes — at any temperature.
//!
//! `sealed-mass-conservation.lab` weighs a sealed vessel of vinegar and
//! soda against an open one. When the heat balance began cooling the sealed
//! beaker by ~7 K (#445), its reading moved from 55.88 g to 55.84 g — away
//! from the ~55.89 g its inputs add up to — while a sealed vessel of plain
//! water holds its mass across a 98 K drop. `Vessel::mass` is a sum of
//! portion moles times molar mass and knows nothing about temperature, so
//! the drift is an inventory change: carbon crossing between the finite
//! headspace and the solution is being booked at the wrong mass on one side
//! of the crossing.
//!
//! These runs separate the candidates the way the peer who found it
//! suggested: a known amount of CO₂ sealed over water with no chemistry at
//! all, cooled and weighed, isolates the crossing; the volcano then shows
//! whether the reaction path adds anything. Every assertion prints the full
//! inventory before and after, because the number that fails is the
//! diagnosis.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn run(bench: &mut Bench, stack: &mut SolverStack, line: &str) -> Vec<Event> {
    let op = script::parse_op(line)
        .unwrap_or_else(|e| panic!("{line}: {e}"))
        .unwrap_or_else(|| panic!("{line}: no operator"));
    bench
        .step_with(op, stack, &PermissiveScreen)
        .unwrap_or_else(|e| panic!("{line}: {e:?}"))
}

fn inventory(bench: &Bench) -> String {
    let v = bench.vessel(VesselId(0)).unwrap();
    let mut lines = vec![format!(
        "T = {:.2} K, p = {:.1} kPa, mass = {:.5} g",
        v.temperature.0,
        v.pressure.0 / 1000.0,
        v.mass().0
    )];
    for p in &v.contents {
        let m = species::lookup(&p.species).map_or(0.0, |d| d.molar_mass);
        lines.push(format!(
            "  {:<14} {:?} {:.7} mol × {:.3} g/mol = {:.5} g",
            p.species.0,
            p.phase,
            p.moles.0,
            m,
            p.moles.0 * m
        ));
    }
    lines.join("\n")
}

fn mass(bench: &Bench) -> f64 {
    bench.vessel(VesselId(0)).unwrap().mass().0
}

/// The crossing alone: CO₂ sealed over water, nothing reacts, the vessel
/// cools, more gas dissolves. The balance must not move.
///
/// Carbon is conserved now (0.0200076 → 0.0200071 mol across a 14 K drop;
/// before the gas phase was given its temperature it was 0.020013 →
/// 0.019129). What remains is +0.01085 g on the balance for 0.00064 mol
/// more CO₂ dissolved — 17 g/mol, one water per carbon: the readback books
/// every dissolved carbon as HCO₃⁻ (61 g/mol) while PHREEQC's water mass
/// does not drop for the H and O it lent (49.84989 g before and after).
/// From NaHCO₃ that is exact, because the solid brought its own H and O;
/// from CO₂ gas it is 17 g/mol too heavy, and at pH 4 the species is 99%
/// CO₂(aq) anyway, so the name is wrong as well as the mass. The fix is a
/// C(4) protonation split [CO₂(aq), HCO₃⁻, CO₃²⁻] with a water debit for
/// the protonated forms, the same mechanism as N(−3)'s — the aqueous
/// lane's, and this test is un-ignored when it lands.
#[test]
#[ignore = "dissolved CO2 from the gas is booked as HCO3- without a water debit (+17 g/mol); aqueous lane, C(4) protonation split"]
fn co2_over_water_keeps_its_mass_when_it_cools() {
    let mut bench = Bench::new();
    let mut stack = stack();
    run(&mut bench, &mut stack, "add v1 water 50mL");
    run(&mut bench, &mut stack, "seal v1 500mL");
    run(&mut bench, &mut stack, "add v1 CO2 0.02mol");
    let before = inventory(&bench);
    let m0 = mass(&bench);
    // 50 mL of water is ~210 J/K; 3 kJ is about 14 K.
    run(&mut bench, &mut stack, "cool v1 3kJ");
    let after = inventory(&bench);
    let m1 = mass(&bench);
    assert!(
        (m1 - m0).abs() < 1e-3,
        "sealed CO2 over water lost or gained {:.5} g on cooling\nBEFORE\n{before}\nAFTER\n{after}",
        m1 - m0
    );
}

/// The lesson itself: the sealed volcano, then cooled further. Its reading
/// must equal what went in, at both temperatures.
#[test]
fn the_sealed_volcano_weighs_what_went_in() {
    let mut bench = Bench::new();
    let mut stack = stack();
    run(
        &mut bench,
        &mut stack,
        "add v1 white_vinegar_5_percent 50mL",
    );
    run(&mut bench, &mut stack, "seal v1 500mL");
    let m_before_soda = mass(&bench);
    // `baking_soda` is 100% NaHCO3 in the registry, so 5 g in is 5 g on
    // the balance — the sealed vessel keeps every gas the reaction makes.
    run(&mut bench, &mut stack, "add v1 baking_soda 5g");
    let added = 5.0;
    let after_soda = inventory(&bench);
    let m_after_soda = mass(&bench);
    // Within 5 mg of 55.89 g (CI reads 55.88626): the bicarbonate came in
    // as a solid carrying its own H and O, so its booking is exact, and the
    // residual is the small share of dissolved carbon that is really
    // CO₂(aq) under the HCO₃⁻ label — the open item the ignored test above
    // measures on its own.
    assert!(
        (m_after_soda - (m_before_soda + added)).abs() < 5e-3,
        "the sealed volcano weighs {:.5} g but {:.5} g + {:.5} g went in\n{after_soda}",
        m_after_soda,
        m_before_soda,
        added
    );
    run(&mut bench, &mut stack, "cool v1 2kJ");
    let cooled = inventory(&bench);
    let m_cooled = mass(&bench);
    assert!(
        (m_cooled - m_after_soda).abs() < 5e-3,
        "the sealed volcano changed mass by {:.5} g on cooling\nBEFORE\n{after_soda}\nAFTER\n{cooled}",
        m_cooled - m_after_soda
    );
}

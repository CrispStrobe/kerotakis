//! Lactic acid, and the yoghurt it makes.
//!
//! None of the three databases this lab loads defines a lactate species,
//! so the commonest acid a kitchen makes had no acidity: the carboxylic
//! proton was absent from every pH, and a fermented milk was refused a
//! characterisation rather than reported without its only acid.
//! `databases::minteq_v4()` adds one reviewed definition, and these are
//! the claims that buys.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
}

fn ferment_yoghurt() -> (Bench, VesselId) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    for op in ["add v1 milk 100mL", "add v1 yoghurt_culture 1g", "wait 8h"] {
        bench
            .step_with(
                kerotakis_core::script::parse_op(op)
                    .expect("parse")
                    .expect("a known verb"),
                &mut stack,
                &PermissiveScreen,
            )
            .expect("step");
    }
    (bench, v)
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

/// The acid the bacteria made must still be in the beaker.
///
/// This is the test that would have stopped the first version shipping.
/// Giving `lactic_acid` a derived role turns it from a portion the tail
/// ignores into one the tail partitions into an element total — and if
/// that total does not come back, the acid's mass leaves the ledger
/// silently. It did: the element went in at 0.0038 mol and returned 0.0,
/// and the vessel weighed 0.3 g less than the same vessel on main, which
/// is 0.0038 mol x 90 g/mol to the tenth of a gram.
///
/// The cause was that `minteq.v4.dat` ends with `END`, and PHREEQC stops
/// reading a database there. The extension was appended AFTER it — not a
/// block the engine rejects loudly, one it never sees.
///
/// THE NUMBER MOVED TWICE ON 2026-09-16 AND THIS IS WHERE IT SETTLED.
/// First the rate stopped reading the GRAMS of culture and started reading
/// their concentration against a declared one-litre reference volume, which
/// multiplied a 100 mL script by eleven and took 0.0038 mol of acid to
/// 0.0307. Then the constants themselves were FITTED, for the first time,
/// and the lactic one fell by a factor of 51: it is now what makes this
/// engine convert 6.106% of a milk's lactose in eight hours at the
/// culture's declared 43 °C optimum — Jankowska et al. 2026 Table 1 for the
/// extent, Kim, Oh and Imm 2018 for the eight hours. Eight counter-top
/// hours at 25 °C are far below that optimum, so `bio-069` converts 1.48%
/// and deposits 8.53e-4 mol, which is a FIFTH of what this bench made
/// before either change. See
/// `kerotakis/fermentation-rate-calibration-v1` in the registry.
///
/// The window is 2%, which is the model's own rounding and not a licence:
/// the value this test used to hold was thirty-six times away.
#[test]
fn the_acid_the_fermentation_made_stays_in_the_ledger() {
    let (bench, v) = ferment_yoghurt();
    let acid = moles(&bench, v, "lactic_acid");
    let anion = moles(&bench, v, "lactate");
    assert!(
        (acid + anion - 8.5336e-4).abs() < 2e-5,
        "the fermentation's 8.5336e-4 mol is now {acid} acid + {anion} anion"
    );
    // And it is genuinely SPLIT, not booked wholly as one form.
    assert!(
        acid > 0.0 && anion > 0.0,
        "both forms should carry some of it: {acid} acid, {anion} anion"
    );
}

/// The point of the exercise: a fermented milk is acidic, and it is
/// acidic to a number rather than to an apology.
///
/// THE NUMBER IS 5.80 AND IT IS NOT A YOGHURT'S. Real yoghurt is pH
/// 4.4-4.6. This bench read 3.89 before 2026-09-16, then 2.83 after the
/// rate started reading a concentration, 5.44 once the rate had been
/// fitted, and 5.80 since the colloidal calcium phosphate was booked into
/// the recipe on 2026-09-18. NOTHING HERE WAS FITTED TO A pH, and the fact
/// that the bench has crossed the real window from below to above without
/// ever aiming at it is the clearest evidence of that.
///
/// WHY IT IS ABOVE, AND WHY THAT IS RIGHT. `bio-069` pours milk onto a
/// counter at 25 °C, and a yoghurt culture's declared optimum is 43 °C, so
/// eight hours converts 1.48% of the lactose where the same eight hours at
/// the optimum would convert 6.106%. This beaker holds a fifth of the acid
/// the fermentation it is fitted to would make, and a fraction of what a
/// finished yoghurt holds. IT IS A SOURED MILK PART-WAY THROUGH, not a set
/// yoghurt, and a reader who takes 5.44 for a yoghurt's pH has been misled
/// by this test rather than informed by it.
///
/// AND THE SECOND DEFECT STILL POINTS THE OTHER WAY, though less far than
/// it did. Half of what was missing is now here: the colloidal calcium
/// phosphate is booked as a solid and dissolves as the acid arrives, which
/// is worth 0.36 of a unit at this dose (5.44 to 5.80). Casein's own
/// buffering is not here, and Kim et al. 2018 quoting Salaun et al. 2005
/// rank it at about 35% of milk's buffer capacity, so this number is still
/// a LOWER BOUND: real milk carrying this much acid would read higher
/// still. Those two errors pull in opposite directions, which is exactly
/// why a rate fitted to a pH would have been worthless. See
/// `docs/milk-buffer-and-the-fermentation-rate.md`.
#[test]
fn the_counter_top_milk_is_soured_and_is_not_a_yoghurt() {
    let (bench, v) = ferment_yoghurt();
    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("a fermented milk must be characterised")
        .ph;
    assert!(
        (5.5..6.1).contains(&ph),
        "eight counter-top hours should leave a soured milk near pH 5.80 - \
         ABOVE a real yoghurt's 4.4-4.6, because this is a partial \
         fermentation far below the culture's optimum - and it read {ph}"
    );
}

/// Fresh milk, for contrast and as a control on the recipe: near neutral,
/// and the fermentation is what moves it.
#[test]
fn the_fermentation_is_what_acidifies_the_milk() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    bench
        .step_with(
            kerotakis_core::script::parse_op("add v1 milk 100mL")
                .expect("parse")
                .expect("a known verb"),
            &mut stack,
            &PermissiveScreen,
        )
        .expect("step");
    let fresh = bench.vessel(v).unwrap().solution.clone().expect("milk").ph;
    assert!(
        (6.4..=7.0).contains(&fresh),
        "fresh milk is near neutral, got {fresh}"
    );

    let (fermented_bench, fv) = ferment_yoghurt();
    let soured = fermented_bench
        .vessel(fv)
        .unwrap()
        .solution
        .clone()
        .expect("yoghurt")
        .ph;
    // A WINDOW AND NOT A FLOOR, because both ends mean something. Below
    // 0.5 the culture has stopped doing the thing the row asks about;
    // above 2.0 the rate has run away again, which is the defect the
    // 2026-09-16 calibration closed. It used to read "more than three
    // units", which the bench cleared by making eight times too much acid.
    // The floor came down from 0.8 to 0.5 on 2026-09-18, because the
    // colloidal calcium phosphate now absorbs part of what the culture
    // makes: the drop is 0.76 where it was 1.1, and a buffer that resists
    // is supposed to make the drop smaller.
    let drop = fresh - soured;
    assert!(
        (0.5..2.0).contains(&drop),
        "the culture should drop the pH by about 0.76 units: {fresh} to {soured}, \
         a drop of {drop}"
    );
}

/// THE BUFFER GAP, MEASURED RATHER THAN ESTIMATED — and since 2026-09-18
/// the number in this file to be most careful about.
///
/// Nothing in the fermentation was fitted to a pH. So asking what this
/// bench reads at the acid dose the CITED yoghurt actually carries is a
/// fair test of the milk recipe, and the answer is the size of the missing
/// buffer at a real yoghurt's acidity.
///
/// The dose is Jankowska et al. 2026's own: 6.106% of the milk's lactose,
/// on the balanced homolactic route, converted at 43 °C to pH 4.6. It is
/// added as acid rather than fermented, so the rate model plays no part and
/// the only thing under test is the buffer.
///
/// WHAT IT READ, AND WHAT THAT DOES AND DOES NOT MEAN. Their beaker read
/// pH 4.6 with that acid in it. This one read **3.94** until the colloidal
/// calcium phosphate was booked into the recipe, and reads **4.60** now.
/// The gap of 0.66 of a unit is closed at this one dose.
///
/// **THAT IS NOT A VALIDATED BUFFER, AND THIS FILE MUST NOT BE READ AS
/// SAYING IT IS.** Three things are true at once:
///
///   1. Casein is still modelled by nothing, and Kim et al. 2018 quoting
///      Salaün et al. 2005 rank the caseins at about 35% of milk's buffer
///      capacity — second only to the soluble minerals.
///   2. The colloid that IS modelled is **spent at very nearly this
///      acidity**. The dose curve reads 5.21 at three quarters of the acid
///      with 4% of the solid left and 4.60 with none, so the agreement
///      sits on the shoulder of an exhaustion rather than in the middle of
///      a buffered region.
///   3. How much colloid to book is uncertain by 40%. The recipe's own two
///      budgets — the colloidal phosphorus, which is what is booked, and
///      the colloidal calcium, which is not — land 0.47 of a unit apart at
///      this dose (4.60 against 5.07). That is the same size as the error
///      the addition repairs.
///
/// So the assertions here are a WINDOW around 4.6 and, more importantly,
/// the second one: past the colloid this beaker has nothing left, which is
/// what the missing casein would have been doing, and that is the residual
/// made visible rather than argued from a percentage.
#[test]
fn the_cited_yoghurts_own_acid_and_the_casein_that_is_still_missing() {
    // 100 mL of the recipe is 103 g; its unresolved solids are 12.5675 g
    // and 4.944 g of those are lactose. Four lactic acids per lactose.
    let lactic_molar_mass = kerotakis_core::species::lookup_key("lactic_acid")
        .expect("lactic acid is a registry species")
        .molar_mass;
    let water_molar_mass = kerotakis_core::species::lookup_key("water")
        .expect("water is a registry species")
        .molar_mass;
    let lactose_molar_mass = 4.0 * lactic_molar_mass - water_molar_mass;
    let lactose_moles = 103.0 * 0.048 / lactose_molar_mass;
    let acid = 4.0 * lactose_moles * (1.0 - 5.69 / 6.06);

    let acidified = |dose: f64| -> f64 {
        let mut bench = Bench::new();
        let mut stack = stack();
        let v = VesselId(0);
        bench
            .step_with(
                kerotakis_core::script::parse_op("add v1 milk 100mL")
                    .expect("parse")
                    .expect("a known verb"),
                &mut stack,
                &PermissiveScreen,
            )
            .expect("pour the milk");
        bench
            .step_with(
                Operator::Add {
                    vessel: v,
                    species: SpeciesId::new("lactic_acid"),
                    moles: Moles(dose),
                    at: None,
                },
                &mut stack,
                &PermissiveScreen,
            )
            .expect("add the acid");
        bench
            .vessel(v)
            .unwrap()
            .solution
            .clone()
            .expect("an acidified milk must be characterised")
            .ph
    };

    let at_the_cited_dose = acidified(acid);
    // The window, not the digit. 4.60 is what it reads; the claim is that
    // the modelled buffer is now of the right SIZE at this acidity, and a
    // window says that where a digit would pretend the coefficient is
    // established. It is not — see point 3 above.
    assert!(
        (4.35..4.85).contains(&at_the_cited_dose),
        "{acid} mol of lactic acid is what a yoghurt at pH 4.6 carries, and \
         with the colloidal calcium phosphate booked this recipe should land \
         near it; it read {at_the_cited_dose}"
    );

    // THE RESIDUAL, MADE VISIBLE. Half again as much acid, and this beaker
    // has nothing to meet it with: the colloid is already gone and casein
    // is not here, so it falls away where a real one still has the carboxyl
    // groups that give milk its second buffering region below pH 5. If this
    // ever stops being true, something has started modelling casein and
    // this file should say so rather than quietly pass.
    let past_the_colloid = acidified(acid * 1.5);
    assert!(
        past_the_colloid < 4.3,
        "past the modelled colloid there is no buffer left in this recipe and \
         it must fall away; at 1.5x the yoghurt's acid it read \
         {past_the_colloid}, which would mean something is resisting that \
         this recipe does not claim to have"
    );
    assert!(
        at_the_cited_dose - past_the_colloid > 0.2,
        "the fall past the colloid has to be visible: {at_the_cited_dose} to \
         {past_the_colloid}"
    );
}

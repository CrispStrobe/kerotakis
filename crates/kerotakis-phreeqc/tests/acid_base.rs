//! Acid–base chemistry through charge balance, titration to equivalence,
//! reaction heat in the energy balance, and the L0 veto in the full loop.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

fn add(
    bench: &mut Bench,
    stack: &mut SolverStack,
    v: VesselId,
    key: &str,
    moles: f64,
) -> Vec<Event> {
    bench
        .step_with(
            Operator::Add {
                vessel: v,
                species: SpeciesId::new(key),
                moles: Moles(moles),
                at: None,
            },
            stack,
            &ReactiveGroupScreen,
        )
        .expect("step")
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("characterised")
        .ph
}

#[test]
fn hydrochloric_acid_is_acidic() {
    // 0.001 mol HCl in 1 kg water → pH ≈ 3.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "HCl", 0.001);
    let ph = ph(&bench, v);
    assert!(
        (ph - 3.0).abs() < 0.1,
        "0.001 m HCl should be pH ~3, got {ph}"
    );
}

#[test]
fn sodium_hydroxide_is_basic() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaOH", 0.001);
    let ph = ph(&bench, v);
    assert!(
        (ph - 11.0).abs() < 0.2,
        "0.001 m NaOH should be pH ~11, got {ph}"
    );
}

#[test]
fn titration_walks_the_curve_to_equivalence() {
    // Strong acid titrated with strong base: acidic → equivalence (≈7) →
    // basic, all from charge balance against the same database.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "HCl", 0.01);
    let start = ph(&bench, v);
    assert!(start < 2.3, "0.01 m strong acid, got pH {start}");

    add(&mut bench, &mut stack, v, "NaOH", 0.005);
    let halfway = ph(&bench, v);
    assert!(
        halfway > start && halfway < 3.0,
        "half-neutralised strong acid stays acidic, got pH {halfway}"
    );

    add(&mut bench, &mut stack, v, "NaOH", 0.005);
    let equivalence = ph(&bench, v);
    assert!(
        (equivalence - 7.0).abs() < 0.3,
        "equivalence point of strong-strong titration is ~7, got {equivalence}"
    );

    add(&mut bench, &mut stack, v, "NaOH", 0.001);
    let excess = ph(&bench, v);
    assert!(excess > 10.5, "excess base swings basic, got pH {excess}");
}

#[test]
fn dissolving_sodium_hydroxide_warms_the_water() {
    // 0.1 mol NaOH (ΔH_dis = −44.5 kJ/mol) into 100 mL water:
    // Q = 4.45 kJ into Cp ≈ 418 J/K → ΔT ≈ +10.6 K.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.55);
    let events = add(&mut bench, &mut stack, v, "NaOH", 0.1);

    assert!(
        events.iter().any(
            |e| matches!(e, Event::TemperatureChanged { to, from, .. } if to.0 > from.0 + 5.0)
        ),
        "dissolving NaOH must warm the vessel, got {events:?}"
    );
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        (t - 35.6).abs() < 2.0,
        "expected ~35.6 °C after the exotherm, got {t:.1} °C"
    );
}

#[test]
fn endothermic_salt_cools_slightly() {
    // NaCl ΔH_dis = +3.88 kJ/mol: 1 mol into 1 L water → ΔT ≈ −0.9 K.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 55.51);
    add(&mut bench, &mut stack, v, "NaCl", 1.0);
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        t < 25.0 && t > 23.5,
        "dissolving NaCl cools slightly, got {t:.2} °C"
    );
}

#[test]
fn bleach_and_ammonia_warns_then_shows_the_chloramine() {
    // Pedagogy over prohibition: the warning always comes first, and then
    // the virtual lab shows precisely what would happen.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.55);
    add(&mut bench, &mut stack, v, "NaOCl", 0.1);
    let events = add(&mut bench, &mut stack, v, "NH3", 0.1);

    let warn_pos = events
        .iter()
        .position(
            |e| matches!(e, Event::HazardWarning { hazard, .. } if hazard.contains("chloramine")),
        )
        .expect("hazard warning must be present");
    let gas_pos = events
        .iter()
        .position(
            |e| matches!(e, Event::GasEvolved { species, moles, .. } if species.0 == "NH2Cl" && (moles.0 - 0.1).abs() < 1e-9),
        )
        .expect("chloramine gas must actually evolve");
    assert!(warn_pos < gas_pos, "the warning precedes the chemistry");
    assert!(
        events.iter().any(
            |e| matches!(e, Event::ReactionOccurred { equation, .. } if equation.contains("NH2Cl"))
        ),
        "the equation is shown"
    );

    // The reactants are consumed; the NaOH byproduct makes it basic.
    let vessel = bench.vessel(v).unwrap();
    assert!((vessel.moles_of(&SpeciesId::new("NaOCl")).0).abs() < 1e-9);
    assert!((vessel.moles_of(&SpeciesId::new("NH3")).0).abs() < 1e-9);
    let ph = vessel.solution.clone().expect("characterised").ph;
    assert!(
        ph > 12.0,
        "0.1 mol NaOH byproduct in 100 mL is strongly basic, got pH {ph}"
    );
}

#[test]
fn household_cleaner_recipes_reach_the_same_warning_and_reaction() {
    let mut bench = Bench::new();
    let mut stack = stack();
    for command in [
        "add v1 Chlorreiniger_5% 10mL",
        "add v1 Ammoniakreiniger_5% 10mL",
    ] {
        let op = kerotakis_core::script::parse_op(command)
            .expect("valid localized cleaner command")
            .expect("operator");
        let events = bench
            .step_with(op, &mut stack, &ReactiveGroupScreen)
            .expect("add household cleaner");
        if command.contains("Ammoniak") {
            let warn = events
                .iter()
                .position(|event| {
                    matches!(event,
                        Event::HazardWarning { hazard, .. } if hazard.contains("chloramine")
                    )
                })
                .expect("bleach plus ammonia warning");
            let gas = events
                .iter()
                .position(|event| {
                    matches!(event,
                        Event::GasEvolved { species, moles, .. }
                            if species.0 == "NH2Cl" && moles.0 > 0.006
                    )
                })
                .expect("the resolved recipe components make chloramine");
            assert!(warn < gas, "warning precedes computed chemistry");
        }
    }
}

#[test]
fn decanting_bleach_into_ammonia_warns_first() {
    let mut bench = Bench::new();
    let mut stack = stack();
    bench.step(Operator::NewVessel { kind: None }).unwrap();
    let (a, b) = (VesselId(0), VesselId(1));
    add(&mut bench, &mut stack, a, "NaOCl", 0.1);
    add(&mut bench, &mut stack, b, "NH3", 0.1);
    let events = bench
        .step_with(
            Operator::Decant {
                from: a,
                to: b,
                fraction: 0.5,
            },
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("step");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::HazardWarning { .. })),
        "pouring bleach into ammonia must warn, got {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::GasEvolved { species, .. } if species.0 == "NH2Cl")),
        "and the gas forms in the target vessel, got {events:?}"
    );
}

#[test]
fn the_titrate_verb_finds_the_equivalence_point() {
    // The burette holds a standard solution: 0.01 mol HCl titrated with
    // 1 mol/L NaOH at 1 mL per step must cross pH 7 at 10 mL — moles of
    // base equal moles of acid at equivalence (the codex's own claim) —
    // within the one-step resolution the step size allows.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 27.75);
    add(&mut bench, &mut stack, v, "HCl", 0.01);
    let events = bench
        .step_with(
            kerotakis_core::script::parse_op("titrate v1 NaOH 1M 1mL until ph 7 max 50")
                .expect("grammar")
                .expect("an operator"),
            &mut stack,
            &ReactiveGroupScreen,
        )
        .expect("titration runs");
    let (concentration, total_volume, final_ph, curve) = events
        .iter()
        .find_map(|e| match e {
            Event::Titrated {
                concentration,
                total_volume,
                final_ph,
                curve,
                ..
            } => Some((*concentration, *total_volume, *final_ph, curve.clone())),
            _ => None,
        })
        .expect("a Titrated event");
    assert!((concentration - 1.0).abs() < 1e-12);
    let delivered_moles = concentration * total_volume.0;
    assert!(
        (delivered_moles - 0.01).abs() <= 0.001 + 1e-12,
        "equivalence at 0.01 mol NaOH within one 0.001-mol step, got {delivered_moles}"
    );
    assert!(
        final_ph >= 7.0,
        "the crossing step ends basic, got {final_ph}"
    );
    // The curve is a real curve: it starts acidic, crawls, then leaps.
    let start = curve.first().expect("initial pH").1;
    assert!(
        start < 2.5,
        "0.01 mol HCl in 0.5 L starts acidic, got {start}"
    );
    assert!(
        curve.len() >= 9,
        "several readings before the leap, got {} points",
        curve.len()
    );
}

/// A bottle of household ammonia computes its own pH.
///
/// It could not, until the derivation learned that ammonia is the same
/// valence-carrying unit as ammonium one proton lighter. Before that the
/// nitrogen fell through to the residue rules — where bare N is not
/// allowed — and the beaker refused: "no aqueous solution has been
/// characterised in this vessel". Both shipped databases that carry
/// nitrogen have known how to do this all along (`NH4+ = NH3 + H+`, with
/// N(-3) mastered by NH4+); nothing was ever asking them.
#[test]
fn household_ammonia_computes_its_own_ph() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "NH3", 0.01);

    let ph = bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("an ammonia solution is a solution")
        .ph;
    // 0.1 mol/L of a base with Kb 1.8e-5: [OH-] = sqrt(Kb·c) = 1.34e-3,
    // pOH 2.87, pH 11.13. The textbook answer, not a curated one.
    assert!(
        (ph - 11.13).abs() < 0.1,
        "0.1 M ammonia is pH 11.1, got {ph:.2}"
    );
}

/// And it is still ammonia afterwards.
///
/// This is the half that had to come with it. The readback books an
/// element total as one ion, and reduced nitrogen's booking ion is NH4+ —
/// so giving ammonia a role without the protonation split would have made
/// the beaker's own contents disagree with its pH: a solution reading 11.1
/// whose ledger said ammonium. `senses::waft` walks that ledger, so the
/// bench would have stopped smelling of ammonia at the exact moment you
/// measured it.
#[test]
fn an_ammonia_solution_is_still_made_of_ammonia() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "NH3", 0.01);

    let vessel = bench.vessel(v).unwrap();
    let ammonia = vessel.moles_of(&SpeciesId::new("NH3")).0;
    let ammonium = vessel.moles_of(&SpeciesId::new("NH4+")).0;

    // At pH 11.1, five orders above pKa 9.25 by two, the free base is
    // almost all of it — but not all, and the rest is not thrown away.
    assert!(
        ammonia > 0.9 * 0.01,
        "above pKa the free base dominates, got {ammonia:.6} mol NH3"
    );
    assert!(
        ammonium > 0.0,
        "the conjugate acid is present, not rounded out of existence"
    );
    // The element total stays authoritative: the split decides how to
    // name the nitrogen, never how much of it there is.
    assert!(
        (ammonia + ammonium - 0.01).abs() < 1e-6,
        "nitrogen is conserved across the split: {ammonia:.9} + {ammonium:.9}"
    );
    assert!(
        !kerotakis_core::senses::waft(vessel).is_empty(),
        "and the beaker still smells of what is in it"
    );
}

/// The other side of the same table row: ammonium chloride is unmoved.
///
/// Group extraction is greedy and ordered, and NH4 must be tried before
/// NH3 or this salt decomposes as ammonia plus a stray proton — booking a
/// school reagent as a solution of a weak base and hydrochloric acid. The
/// pH is the assertion that catches it: 5.2 is a weak acid, 11.1 would be
/// the base, and a mis-ordered table gives neither.
#[test]
fn ammonium_chloride_is_still_an_ammonium_salt() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "NH4Cl", 0.01);

    let vessel = bench.vessel(v).unwrap();
    let ph = vessel.solution.clone().expect("characterised").ph;
    assert!(
        (4.8..5.6).contains(&ph),
        "0.1 M ammonium chloride is a weak acid near pH 5.2, got {ph:.2}"
    );
    let ammonium = vessel.moles_of(&SpeciesId::new("NH4+")).0;
    let ammonia = vessel.moles_of(&SpeciesId::new("NH3")).0;
    assert!(
        ammonium > 0.99 * (ammonium + ammonia),
        "four pH units below pKa the salt is ammonium, got {ammonium:.6} \
         mol NH4+ against {ammonia:.9} mol NH3"
    );
}

/// Vinegar and baking soda fizz — in water, which is the only way anybody
/// has ever done it.
///
/// This reaction has been curated, reviewed and present in `curated.rs`
/// since the beginning, and until the acetate protonation split it could
/// not fire in a beaker. The readback booked the whole Acetate total as
/// `CH3COO-`, so pouring vinegar into water handed back acetate ion, and by
/// the time the bicarbonate arrived the acid named in the reactant list was
/// no longer in the vessel. It would only ever have fired in a dry one.
///
/// The corpus said so and nobody read it: `aq-059` sat at reason code
/// `computed-route`, and the classifier checks the curated route FIRST — so
/// that code meant, in writing, that the curated route produced no events.
#[test]
fn vinegar_and_baking_soda_fizz_in_water() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "CH3COOH", 0.05);

    // The acid is in the ledger as an acid. That is the precondition, and
    // it is the whole of what was missing.
    let free_acid = bench
        .vessel(v)
        .unwrap()
        .moles_of(&SpeciesId::new("CH3COOH"))
        .0;
    assert!(
        free_acid > 0.04,
        "a pH 2.5 solution is mostly undissociated acid, got {free_acid:.5} mol"
    );

    let events = add(&mut bench, &mut stack, v, "NaHCO3", 0.05);
    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::ReactionOccurred { equation, .. } if equation.contains("NaHCO")
        )),
        "the curated reaction fires, got {events:?}"
    );
    // The carbon is conserved across the two routes that share it: the
    // curated equation takes the free acid, the aqueous route takes the
    // small remainder, and neither invents any.
    let co2: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::GasEvolved { species, moles, .. } if species.0 == "CO2" => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(
        (co2 - 0.05).abs() < 1e-3,
        "0.05 mol of bicarbonate gives 0.05 mol of gas, got {co2:.5}"
    );
}

/// And vinegar dissolves an eggshell, for the same reason and by the same
/// arithmetic.
#[test]
fn vinegar_dissolves_the_calcium_carbonate_in_an_eggshell() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 5.5343);
    add(&mut bench, &mut stack, v, "CH3COOH", 0.1);
    let events = add(&mut bench, &mut stack, v, "CaCO3", 0.05);

    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::ReactionOccurred { equation, .. } if equation.contains("CaCO")
        )),
        "the curated reaction fires, got {events:?}"
    );
    let shell = bench
        .vessel(v)
        .unwrap()
        .moles_of(&SpeciesId::new("CaCO3"))
        .0;
    // Not to zero, and it should not be: what is left is the saturation
    // residue, 0.08% of the shell sitting in equilibrium with a solution
    // that has taken all the calcium it will hold. A real shell in a real
    // glass of vinegar stops in the same place for the same reason.
    assert!(
        shell < 0.001,
        "two moles of acid per mole of carbonate dissolves essentially all \
         of it, {shell:.6} mol left of 0.05"
    );
}

/// Acid first, the volcano now works and it gets cold.
///
/// The reaction was never simply dead: `curated` runs before the aqueous
/// tail, so on the step where a reagent is ADDED the ledger still holds it
/// as written and the match succeeds. What killed it was a reagent that had
/// already been through a solve — and the readback renames what it books.
/// So this was order dependence rather than absence, which is worse: "add
/// them in the other order" is a workaround somebody finds by accident and
/// never understands.
///
/// The two routes also disagreed about the SIGN of the temperature change.
/// Vinegar and baking soda is one of the few kitchen reactions a child can
/// feel, and what it does is get COLD. The aqueous route had it warming,
/// because it books the heat of H⁺ + OH⁻ → H₂O for whatever consumed the
/// acid and nothing for the endothermic half — the bicarbonate breaking up
/// and the gas leaving. That is a quantity claimed to be the reaction's
/// enthalpy which is invariant over what the reaction was.
///
/// (The curated route is not claiming a better enthalpy. It claims none —
/// see the entry in `curated.rs` — so the cooling here is the dissolution
/// terms alone. A route that declines to claim beats one that claims the
/// wrong sign; neither has the number.)
#[test]
fn the_volcano_cools_when_the_acid_goes_in_first() {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", 2.7);
    add(&mut bench, &mut stack, v, "CH3COOH", 0.042);
    let events = add(&mut bench, &mut stack, v, "NaHCO3", 0.05);

    assert!(
        events.iter().any(|e| matches!(
            e,
            Event::ReactionOccurred { equation, .. } if equation.contains("NaHCO")
        )),
        "the acid survived its solve, so the curated route is reachable: {events:?}"
    );
    let t = bench.vessel(v).unwrap().temperature.to_celsius();
    assert!(
        t < 25.0,
        "the volcano is endothermic and the beaker gets colder, got {t:.1} °C"
    );
}

/// Whichever order you build it in, the beaker gets colder.
///
/// It did not. Put the soda in first with water present and it dissolves,
/// and the acid then meets a bicarbonate rather than the solid — so the
/// curated route was unreachable and the aqueous tail answered. The aqueous
/// tail computes how much acid was cancelled from the solutes' net charge,
/// which cannot tell `HCO₃⁻ + H⁺ → H₂O + CO₂↑` from `H⁺ + OH⁻ → H₂O`
/// because in both a negative solute and an acid disappear together. It
/// charged the first at the second's enthalpy and warmed the beaker to
/// 25.8 °C.
///
/// So the volcano was endothermic one way round and exothermic the other,
/// and the difference was which reagent you happened to dissolve first.
/// Now both cool. They do not agree on how much — 21.0 °C against 24.4 °C —
/// and that difference is honest: the acid taken by the carbonate has a
/// heat this lab does not hold, and the run says so in a `NotYetModeled`
/// rather than borrowing the nearest number in the file.
#[test]
fn the_volcano_cools_whichever_order_you_build_it_in() {
    let brew = |soda_first: bool| {
        let mut bench = Bench::new();
        let mut stack = stack();
        let v = VesselId(0);
        add(&mut bench, &mut stack, v, "water", 2.7);
        let events = if soda_first {
            add(&mut bench, &mut stack, v, "NaHCO3", 0.05);
            add(&mut bench, &mut stack, v, "CH3COOH", 0.042)
        } else {
            add(&mut bench, &mut stack, v, "CH3COOH", 0.042);
            add(&mut bench, &mut stack, v, "NaHCO3", 0.05)
        };
        (bench.vessel(v).unwrap().temperature.to_celsius(), events)
    };
    for soda_first in [true, false] {
        let (t, events) = brew(soda_first);
        assert!(
            t < 25.0,
            "the volcano is endothermic and the beaker gets colder \
             (soda_first={soda_first}), got {t:.1} °C"
        );
        // And it reaches a reviewed equation either way round. Renaming is
        // symmetric, so this needs a row written on the SALT and one on the
        // ION; whichever reagent went through a solve first, one of the two
        // still names what is in the vessel.
        assert!(
            events.iter().any(|e| matches!(
                e,
                Event::ReactionOccurred { equation, .. }
                    if equation.contains("NaHCO") || equation.contains("HCO₃⁻")
            )),
            "a curated equation is reachable in either order \
             (soda_first={soda_first}): {events:?}"
        );
    }
}

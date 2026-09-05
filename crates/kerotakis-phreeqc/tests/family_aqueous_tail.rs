//! BRD-020's router, proven through the real aqueous tail.
//!
//! `kerotakis-org`'s `family_route` tests stop at the ledger: a family
//! fires, the products are named, mass and charge balance. What none of
//! them ask is what PHREEQC then makes of those products, or what the
//! router makes of a vessel PHREEQC has already been through — and both
//! are places a family can be wrong in a way the ledger cannot see.
//!
//! Those tests build their vessels by hand, with `deposit`, and never
//! run a solver under the router. That is the right unit test and it is
//! also why the order dependence pinned at the bottom of this file
//! survived: in a hand-built vessel `NaOH` is a portion forever, and in
//! a real one it is a portion for exactly one step.
//!
//! Most of what is asserted here is *route independence* — the same
//! chemistry reached two ways leaving the beaker in the same state.
//! That is a stronger claim than any single pinned pH, and it does not
//! quietly become vacuous when the database or the dilution changes.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// The standard stack — the one the CLI, the shell and wasm all share —
/// with the real engine as its aqueous tail. The router's position in it
/// is `kerotakis-stack`'s to own; this file only needs it to be in there.
fn stack() -> SolverStack {
    SolverStack::new(kerotakis_stack::standard_solvers(vec![Box::new(
        PhreeqcEquilibrator::new().expect("engine"),
    )]))
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
            &PermissiveScreen,
        )
        .expect("add")
}

fn heat(bench: &mut Bench, stack: &mut SolverStack, v: VesselId, joules: f64) -> Vec<Event> {
    bench
        .step_with(
            Operator::Heat {
                vessel: v,
                energy: Joules(joules),
            },
            stack,
            &PermissiveScreen,
        )
        .expect("heat")
}

fn ph(bench: &Bench, v: VesselId) -> f64 {
    bench
        .vessel(v)
        .unwrap()
        .solution
        .clone()
        .expect("the tail characterised the solution")
        .ph
}

fn moles(bench: &Bench, v: VesselId, key: &str) -> f64 {
    bench.vessel(v).unwrap().moles_of(&SpeciesId::new(key)).0
}

fn temperature(bench: &Bench, v: VesselId) -> f64 {
    bench.vessel(v).unwrap().temperature.0
}

fn saponified(events: &[Event]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::OrgReacted { name, .. } if name == "alkaline-ester-hydrolysis"))
}

/// One litre of water, so the acetate lands near 0.1 mol/L — the dilution
/// the weak-acid tests already work at, and one the activity model in
/// minteq.v4 is entitled to an opinion about.
const WATER: f64 = 55.5;
const SCALE: f64 = 0.1;
/// Enough to carry a litre of water from room temperature past the
/// family's 323.15 K gate with room to spare, without approaching the
/// boil. Asserted rather than assumed, below.
const HEAT_J: f64 = 180_000.0;

/// Ester in warm water, then the base — the pouring order in which the
/// router still has a hydroxide to see. See
/// `only_the_pouring_order_decides_whether_the_ester_hydrolyses` for why
/// that qualification is here and not a detail.
fn saponify() -> (Bench, VesselId, Vec<Event>) {
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", WATER);
    add(&mut bench, &mut stack, v, "ethyl_acetate", SCALE);
    heat(&mut bench, &mut stack, v, HEAT_J);
    let events = add(&mut bench, &mut stack, v, "NaOH", SCALE);
    (bench, v, events)
}

#[test]
fn warm_saponification_survives_the_aqueous_tail() {
    let (bench, v, events) = saponify();

    let t = temperature(&bench, v);
    assert!(
        t > 323.15 && t < 373.15,
        "the heat should clear the family's gate without boiling, got {t} K"
    );
    assert!(saponified(&events), "the family did not fire: {events:?}");

    // The ester is gone and the base is spent — to completion, as the
    // record's refusal domain claims.
    assert!(
        moles(&bench, v, "ethyl_acetate") < 1e-9,
        "ester left over: {}",
        moles(&bench, v, "ethyl_acetate")
    );
    assert!(
        moles(&bench, v, "NaOH") < 1e-9,
        "hydroxide left over: {}",
        moles(&bench, v, "NaOH")
    );

    // The products reach the tail as ions and come back speciated: the
    // acetate takes a proton off water often enough to show, which is
    // the whole reason sodium acetate is a basic salt.
    assert!(
        (moles(&bench, v, "ethanol") - SCALE).abs() < 1e-6,
        "the alcohol: {}",
        moles(&bench, v, "ethanol")
    );
    assert!(
        (moles(&bench, v, "Na+") - SCALE).abs() < 1e-6,
        "the spectator sodium: {}",
        moles(&bench, v, "Na+")
    );
    let acetate = moles(&bench, v, "CH3COO-") + moles(&bench, v, "CH3COOH");
    assert!(
        (acetate - SCALE).abs() < 1e-6,
        "acetate + its conjugate acid should account for the ester: {acetate}"
    );
    assert!(
        moles(&bench, v, "CH3COOH") > 0.0,
        "a basic acetate solution holds a little of the free acid"
    );

    let p = ph(&bench, v);
    assert!(
        p > 8.0 && p < 9.0,
        "0.1 M sodium acetate should be mildly basic, got pH {p}"
    );
}

/// The defect this file was first written to look for, and did not find.
///
/// `solute_charge` is the vessel's memory of how much free acid or base
/// it holds; the aqueous tail recovers the extent of `H⁺ + OH⁻ → H₂O` by
/// differencing it. A router that moved ions without refreshing it would
/// leave that difference looking exactly like a neutralisation — and an
/// ester hydrolysis would come out warm, charged at the
/// strong-acid-strong-base enthalpy, for a reaction whose heat this
/// bench does not hold and does not claim.
#[test]
fn saponification_is_not_billed_as_a_neutralisation() {
    let (_bench, _v, events) = saponify();
    let neutralised: f64 = events
        .iter()
        .filter_map(|e| match e {
            Event::Neutralised { moles, .. } => Some(moles.0),
            _ => None,
        })
        .sum();
    assert!(
        neutralised < 1e-9,
        "the tail booked {neutralised} mol of neutralisation for an ester hydrolysis"
    );
}

/// Two ions or one salt: the tail must not care.
///
/// The router deposits `Na+` and `CH3COO-` separately; a stockroom
/// bottle of the same salt arrives as one `NaOAc` portion. They are the
/// same beaker once the water has had its say, and if they are not, one
/// of the two routes is lying about what it made.
#[test]
fn the_router_arrives_where_the_salt_would_have() {
    let (by_reaction, rv, _) = saponify();

    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", WATER);
    add(&mut bench, &mut stack, v, "ethanol", SCALE);
    heat(&mut bench, &mut stack, v, HEAT_J);
    add(&mut bench, &mut stack, v, "NaOAc", SCALE);

    let reacted = ph(&by_reaction, rv);
    let bottled = ph(&bench, v);
    assert!(
        (reacted - bottled).abs() < 0.05,
        "saponification landed at pH {reacted}, a bottle of the same salt at {bottled}"
    );
}

/// Le Chatelier, counted in moles, with the water the tail actually has.
///
/// The router's esterification takes K = 4 in *moles*, so water already
/// in the beaker pushes it back. `family_route` makes this claim at the
/// ledger; it is repeated here because in the full stack the aqueous
/// tail runs afterwards and could, in principle, take the ester apart
/// again — and because the leftover acid then has to speciate sensibly
/// rather than sit in the vessel uncharacterised.
#[test]
fn water_pushes_esterification_back_in_the_full_stack() {
    let brew = |water: f64| {
        let mut bench = Bench::new();
        let mut stack = stack();
        let v = VesselId(0);
        if water > 0.0 {
            add(&mut bench, &mut stack, v, "water", water);
        }
        add(&mut bench, &mut stack, v, "CH3COOH", SCALE);
        add(&mut bench, &mut stack, v, "ethanol", SCALE);
        add(&mut bench, &mut stack, v, "H2SO4", 0.001);
        heat(&mut bench, &mut stack, v, HEAT_J);
        (bench, v)
    };

    let (dry_bench, dv) = brew(0.0);
    let (wet_bench, wv) = brew(WATER);
    let dry = moles(&dry_bench, dv, "ethyl_acetate");
    let wet = moles(&wet_bench, wv, "ethyl_acetate");
    assert!(dry > 0.0, "the dry case should esterify at all");
    assert!(
        wet < 0.5 * dry,
        "water should push the equilibrium back: wet {wet} vs dry {dry}"
    );

    let p = ph(&wet_bench, wv);
    assert!(
        p < 4.0,
        "leftover acetic acid over a trace of H2SO4 should be acidic, got pH {p}"
    );
}

/// **A known defect, pinned rather than asserted as correct.**
///
/// Saponification fires only if the hydroxide is added to an
/// already-warm vessel. Add the base first and heat afterwards — the
/// order a learner is likelier to use, and the order the curiosity
/// scripts write — and nothing happens at any temperature.
///
/// The cause is not in the gates. `FamilyRouter::candidates` reads
/// species out of `vessel.contents`, and the aqueous tail does not
/// leave a hydroxide there: it dissolves `NaOH` and writes back `Na+`
/// alone, with the alkalinity carried as `solute_charge = +0.1`. That
/// is a deliberate convention and not itself a bug — the neutralisation
/// extent above is *derived* from it, and writing `OH⁻` back as a
/// portion would double-count water's own hydrogen and oxygen. But it
/// means a family whose substrate is a strong base can only match
/// during the single step that base is added, before the tail below the
/// router has spoken. `NaOH` is the only such substrate in the shipped
/// pack, so this is one record's problem today and the pack's problem
/// as soon as a second one lands.
///
/// The fix is a design decision in the family IR — how a record names a
/// reactant the ledger holds as free alkalinity rather than as a
/// portion — and belongs to `family.rs`'s owner. It is deliberately not
/// patched here: making `candidates` conjure an `NaOH` that is not in
/// the ledger would consume a portion that does not exist and deposit a
/// second mole of sodium beside the one already dissolved.
///
/// This test asserts what the bench *does*, so it turns red the day the
/// behaviour changes — in either direction.
#[test]
fn only_the_pouring_order_decides_whether_the_ester_hydrolyses() {
    // Base last, into a warm vessel: fires.
    let (_, _, late_base) = saponify();
    assert!(saponified(&late_base), "{late_base:?}");

    // Base first, then heat: the same three reagents, the same
    // temperature, and nothing happens.
    let mut bench = Bench::new();
    let mut stack = stack();
    let v = VesselId(0);
    add(&mut bench, &mut stack, v, "water", WATER);
    add(&mut bench, &mut stack, v, "ethyl_acetate", SCALE);
    let cold_base = add(&mut bench, &mut stack, v, "NaOH", SCALE);
    let heated = heat(&mut bench, &mut stack, v, HEAT_J);
    let waited = bench
        .step_with(
            Operator::Wait { seconds: 600.0 },
            &mut stack,
            &PermissiveScreen,
        )
        .expect("wait");

    assert!(!saponified(&cold_base), "fired cold: {cold_base:?}");
    assert!(
        temperature(&bench, v) > 323.15,
        "the second beaker must clear the same gate, got {} K",
        temperature(&bench, v)
    );
    assert!(!saponified(&heated), "DEFECT FIXED: fired on heating");
    assert!(!saponified(&waited), "DEFECT FIXED: fired on waiting");
    assert!(
        (moles(&bench, v, "ethyl_acetate") - SCALE).abs() < 1e-9,
        "the ester is untouched"
    );

    // And the router cannot explain itself either: it reports neither a
    // firing nor a decline, because it never saw a hydroxide to match
    // the ester against. A learner asking what this bench can do with a
    // warm alkaline ester is told nothing at all, which is the part of
    // this that matters beyond the one reaction.
    let vessel = bench.vessel(v).unwrap();
    let router = kerotakis_org::family_oracle::family_equilibrator();
    let evaluated = router.evaluate(vessel);
    assert!(
        evaluated.ready.is_empty() && evaluated.declined.is_empty(),
        "DEFECT FIXED: the router now has an opinion: {evaluated:?}"
    );
    assert!(
        moles(&bench, v, "NaOH") < 1e-12 && moles(&bench, v, "Na+") > 0.0,
        "the tail is expected to have eaten the hydroxide and left the sodium"
    );
}

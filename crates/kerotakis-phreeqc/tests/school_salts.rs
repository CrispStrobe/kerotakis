//! BRD-012.S02 — the P0 school-essential salts, and the gated barium pair.
//!
//! A registry identity is not a capability. These species were added as
//! data, so the thing worth pinning is not that `kero species` lists them
//! but that the shipped USGS databases can actually *say something* about
//! each one: an ammonium salt that is weakly acidic because ammonium is a
//! weak acid, an iron(III) salt that is strongly acidic because ferric
//! iron hydrolyses, a sulfate that simply dissolves, and the sulfate test
//! itself — barium chloride into a sulfate solution giving barite, which
//! is EXP-30's open row.
//!
//! Every number below was read off the engine before it was pinned, and
//! the windows are wide enough to be about the chemistry rather than
//! about one database revision. The computed values at the time of
//! writing: NH4Cl pH 4.93, FeCl3 pH 1.92, Na2SO4 pH 7.34 at I = 1.50,
//! BaCl2 pH 6.95, Ba(OH)2 pH 13.11, and 0.009996 of 0.01 mol barium
//! coming down as barite.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use kerotakis_safety::ReactiveGroupScreen;

fn stack() -> SolverStack {
    SolverStack::new(vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(kerotakis_core::hmix::MixingEnthalpyEquilibrator),
        Box::new(PhreeqcEquilibrator::new().expect("engine")),
        Box::new(HonestyEquilibrator),
    ])
}

/// One beaker, the additions in order, the events and the final bench.
fn run(adds: &[(&str, f64)]) -> (Bench, Vec<Event>) {
    let mut bench = Bench::new();
    let mut solvers = stack();
    let mut events = Vec::new();
    for (key, moles) in adds {
        events.extend(
            bench
                .step_with(
                    Operator::Add {
                        vessel: VesselId(0),
                        species: SpeciesId::new(key),
                        moles: Moles(*moles),
                        at: None,
                    },
                    &mut solvers,
                    &ReactiveGroupScreen,
                )
                .unwrap_or_else(|e| panic!("ADD {key}: {e}")),
        );
    }
    (bench, events)
}

fn solution(bench: &Bench) -> SolutionInfo {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .solution
        .clone()
        .expect("the aqueous engine characterised the solution")
}

fn names(info: &SolutionInfo) -> Vec<&str> {
    info.species.iter().map(|s| s.name.as_str()).collect()
}

fn precipitated(events: &[Event], species: &str) -> f64 {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Precipitated {
                species: s, moles, ..
            } if s.0 == species => Some(moles.0),
            _ => None,
        })
        .sum()
}

/// Moles of `element` held anywhere in the vessel, solid or dissolved.
fn element_moles(bench: &Bench, element: &str) -> f64 {
    bench
        .vessel(VesselId(0))
        .expect("vessel")
        .contents
        .iter()
        .filter_map(|p| {
            let data = kerotakis_core::species::lookup(&p.species)?;
            let formula = kerotakis_core::stoich::parse_formula(data.formula).ok()?;
            Some(p.moles.0 * formula.counts.get(element).copied().unwrap_or(0.0))
        })
        .sum()
}

/// 5.55 mol of water is 0.1 kg — the school beaker these tests work in.
const WATER: (&str, f64) = ("water", 5.55);

#[test]
fn ammonium_chloride_dissolves_and_the_solution_is_weakly_acidic() {
    let (bench, _) = run(&[WATER, ("NH4Cl", 0.05)]);
    let info = solution(&bench);
    // NH4+ is a weak acid (pKa 9.25). At 0.5 mol/kgw it sits a little
    // below neutral — computed pH 4.93 — which is a world away from the
    // strong acid an untagged chloride balance would have invented.
    assert!(
        info.ph > 4.0 && info.ph < 7.0,
        "ammonium chloride should be weakly acidic, got pH {}",
        info.ph
    );
    // The nitrogen went in as ammonium and is still ammonium: the air
    // above an open beaker must not oxidise a school salt to nitrate.
    let ammonium = info
        .species
        .iter()
        .find(|s| s.name == "NH4+")
        .unwrap_or_else(|| panic!("NH4+ in the speciation: {:?}", names(&info)));
    assert!(
        ammonium.molality > 0.4,
        "nearly all the nitrogen should be free ammonium, got {}",
        ammonium.molality
    );
    assert!(
        !names(&info).contains(&"NO3-"),
        "nothing in this beaker oxidised the ammonium: {:?}",
        names(&info)
    );
}

#[test]
fn iron_iii_chloride_makes_a_genuinely_acidic_solution() {
    let (bench, events) = run(&[WATER, ("FeCl3", 0.01)]);
    let info = solution(&bench);
    // Ferric hydrolysis, computed rather than curated: Fe³⁺ + H₂O ⇌
    // FeOH²⁺ + H⁺, giving pH 1.92 at 0.1 mol/kgw. This is why the
    // reagent bottle is kept acidified, and it is the reason the salt
    // is worth having on a school shelf at all.
    assert!(
        info.ph < 3.0,
        "ferric chloride hydrolyses; pH should be strongly acidic, got {}",
        info.ph
    );
    // Iron(III), not this lab's default iron(II): the salt's own
    // stoichiometry fixed the oxidation state, and the chloro-complexes
    // that dominate are ferric ones.
    assert!(
        names(&info).contains(&"Fe+3") && names(&info).contains(&"FeCl+2"),
        "the iron must be ferric: {:?}",
        names(&info)
    );
    assert!(
        !names(&info).contains(&"Fe+2"),
        "there is no reductant in this beaker: {:?}",
        names(&info)
    );
    // And the honest half: at pH 2 the amorphous hydroxide is not
    // supersaturated, but hematite and goethite are — phases the
    // registry does not carry. The bench says so instead of quietly
    // reporting a solution that could not stand.
    let note = events
        .iter()
        .find_map(|e| match e {
            Event::NotYetModeled { what, .. } => Some(what.clone()),
            _ => None,
        })
        .expect("the honesty pass speaks about the unmodelled phases");
    assert!(
        note.contains("supersaturated") && note.contains("Hematite"),
        "the note must name what it cannot precipitate: {note}"
    );
    assert!(
        precipitated(&events, "Fe(OH)3") < 1e-9,
        "the amorphous hydroxide is undersaturated at pH 2: {events:?}"
    );
}

#[test]
fn sodium_sulfate_dissolves_and_stays_neutral() {
    let (bench, _) = run(&[WATER, ("Na2SO4", 0.05)]);
    let info = solution(&bench);
    assert!(
        info.ph > 6.5 && info.ph < 8.0,
        "sodium sulfate is the neutral salt of a strong acid and a strong \
         base, got pH {}",
        info.ph
    );
    let sulfate = info
        .species
        .iter()
        .find(|s| s.name == "SO4-2")
        .unwrap_or_else(|| panic!("free sulfate in the speciation: {:?}", names(&info)));
    assert!(
        sulfate.molality > 0.4,
        "0.5 mol/kgw is well under thenardite saturation — the salt is in \
         solution, not on the bottom: {}",
        sulfate.molality
    );
    // I ≈ 3·m is the arithmetic that says a 2:1 electrolyte dissolved.
    assert!(
        info.ionic_strength > 1.0,
        "I ≈ 3·m for a 2:1 salt, got {}",
        info.ionic_strength
    );
}

/// EXP-30's sulfate row: the classic anion test, computed.
#[test]
fn barium_chloride_precipitates_barite_out_of_a_sulfate_solution() {
    let (bench, events) = run(&[WATER, ("Na2SO4", 0.01), ("BaCl2", 0.01)]);
    let barite = precipitated(&events, "BaSO4");
    assert!(
        barite > 0.009,
        "0.01 mol of barium against 0.01 mol of sulfate comes down almost \
         quantitatively (Barite log K -9.97), got {barite} mol"
    );
    // Conservation across the solve: nothing invented, nothing lost.
    let ba = element_moles(&bench, "Ba");
    let s = element_moles(&bench, "S");
    assert!(
        (ba - 0.01).abs() < 1e-6,
        "barium must be conserved, got {ba} mol"
    );
    assert!(
        (s - 0.01).abs() < 1e-6,
        "sulfur must be conserved, got {s} mol"
    );
}

/// The other half of the same verdict: no sulfate, no precipitate. A
/// test that only ever fires is not a test.
#[test]
fn barium_chloride_alone_precipitates_nothing() {
    let (bench, events) = run(&[WATER, ("BaCl2", 0.01)]);
    assert!(
        precipitated(&events, "BaSO4") < 1e-9,
        "barium chloride in plain water is a clear solution: {events:?}"
    );
    let info = solution(&bench);
    assert!(
        info.species.iter().any(|s| s.name == "Ba+2"),
        "the barium is dissolved: {:?}",
        names(&info)
    );
}

#[test]
fn barium_hydroxide_is_a_strong_alkali_in_solution() {
    let (bench, _) = run(&[WATER, ("Ba(OH)2", 0.005)]);
    let info = solution(&bench);
    assert!(
        info.ph > 12.5,
        "baryta water is strongly alkaline — two hydroxides per formula \
         unit, computed pH 13.1 — got {}",
        info.ph
    );
}

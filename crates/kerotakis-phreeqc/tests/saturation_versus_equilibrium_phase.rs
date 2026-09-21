//! Two mechanisms bound the same dissolution, and they have to compose.
//!
//! `kerotakis_core::solve::saturation_moves` reads the registry's curated
//! `aqueous_solubility_g_per_100_ml` and moves a solid into the aqueous
//! compartment as an **undissociated portion of itself** — a bench-level
//! statement that this much went into solution. `DerivedRole::Mineral`
//! says the routed database spells this solid, so the solve poses it in
//! `EQUILIBRIUM_PHASES` and its saturation index does the bounding.
//!
//! Where both apply they used to disagree about what problem was posed.
//! `partition` folded only the `Phase::Solid` part into the phase; what
//! the bench had already moved took the other branch and entered as
//! ELEMENT TOTALS. For most minerals that is a difference of a few
//! micromoles. For a hydroxide it is the whole point of the solid:
//! `contribution_from_counts` drops hydroxide pairs into the charge-balance
//! domain, so portlandite's elements are `[(Ca, 1)]` and the moved portion
//! entered calcium with **no base attached**.
//!
//! Sourced by PR #699, which found four measured solubilities it could not
//! ship because of this. The number that showed it was `Ca(OH)2` at
//! 0.15633 g/100 mL (Bates, Bower & Smith 1956): above the ~2e-4 mol/L line
//! where chalk, sulfur and quartz sit, the whole 0.01 mol dose of
//! `lessons/limewater.lab` moves, and the lesson's only observation — the
//! liquid going milky when carbon dioxide arrives — went with it.
//!
//! The invariant these tests pin is deliberately not a number: **a mineral
//! contributes the same posed problem whichever condensed phase the bench
//! has it booked in.** A datum that changes bookkeeping must not change
//! chemistry.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// One kilogram of water and 0.01 mol of slaked lime, with the lime
/// standing in `lime` — `Phase::Solid` as it is poured, `Phase::Aqueous`
/// as `saturation_moves` leaves it once the solid carries a measured
/// solubility above the trace line.
fn limewater(lime: Phase) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("water"), Moles(55.51), Phase::Liquid);
    vessel.deposit(SpeciesId::new("Ca(OH)2"), Moles(0.01), lime);
    vessel
}

fn dissolved_calcium(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.phase == Phase::Aqueous)
        .map(|portion| {
            let calcium = species::lookup(&portion.species)
                .and_then(|data| stoich::parse_formula(data.formula).ok())
                .and_then(|formula| formula.counts.get("Ca").copied())
                .unwrap_or(0.0);
            portion.moles.0 * calcium
        })
        .sum()
}

fn solved(lime: Phase) -> Vessel {
    let mut solver = PhreeqcEquilibrator::new().expect("engine");
    let mut vessel = limewater(lime);
    solver.equilibrate(&mut vessel).expect("limewater solves");
    vessel
}

/// The mechanism, stated as an invariant rather than as a number.
#[test]
fn a_mineral_poses_the_same_problem_whichever_condensed_phase_holds_it() {
    let poured = solved(Phase::Solid);
    let moved = solved(Phase::Aqueous);

    let ph = |vessel: &Vessel| {
        vessel
            .solution
            .as_ref()
            .unwrap_or_else(|| panic!("limewater has a solution"))
            .ph
    };

    assert!(
        (ph(&poured) - ph(&moved)).abs() < 1e-6,
        "a saturation move is bookkeeping, not chemistry: lime as a solid \
         gave pH {:.4} and the same lime booked as dissolved gave pH {:.4}",
        ph(&poured),
        ph(&moved),
    );
    assert!(
        (dissolved_calcium(&poured) - dissolved_calcium(&moved)).abs() < 1e-9,
        "calcium in solution moved with the bookkeeping: {} vs {}",
        dissolved_calcium(&poured),
        dissolved_calcium(&moved),
    );
}

/// And the half of it that actually bit: the base.
///
/// Slaked lime's whole contribution to limewater is alkalinity. Entered as
/// element totals it arrived as calcium alone, because the formula's two
/// hydroxides had already been dropped into the charge-balance domain that
/// only a posed phase puts back. A limewater that is not alkaline has
/// nothing for carbon dioxide to precipitate.
#[test]
fn slaked_lime_booked_as_dissolved_is_still_alkaline() {
    let moved = solved(Phase::Aqueous);
    let ph = moved.solution.as_ref().expect("a solution").ph;
    assert!(
        ph > 11.0,
        "0.01 mol of slaked lime in a litre of water is limewater whichever \
         compartment the bench has it in; this read pH {ph:.4}"
    );
}

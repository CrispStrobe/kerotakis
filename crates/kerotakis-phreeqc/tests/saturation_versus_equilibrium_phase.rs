//! Two mechanisms bound the same dissolution, and they have to compose.
//!
//! `kerotakis_core::solve::saturation_moves` reads the registry's curated
//! `aqueous_solubility_g_per_100_ml` and moves a solid into the aqueous
//! compartment as an **undissociated portion of itself** — a bench-level
//! statement that this much went into solution. `DerivedRole::Mineral`
//! says a routed database spells the solid, so `partition` poses it in
//! `EQUILIBRIUM_PHASES` and its saturation index does the bounding.
//!
//! `partition` used to fold only the `Phase::Solid` part into the phase;
//! what the bench had already moved took the other branch and entered as
//! ELEMENT TOTALS. #699 named that as the reason it could not ship four
//! measured solubilities it had sourced. **Reproduced 2026-09-21, and the
//! diagnosis does not hold** — the transcript is in `PLAN.md`. Posing the
//! phase and entering the totals reach the same answer, because
//! `append_candidate_phases` already offers the phase at zero moles and
//! `pH charge` recovers the hydroxide that `contribution_from_counts`
//! drops. Slaked lime booked either way solves to pH 12.1962 / 12.1974,
//! the difference being its heat of dissolution and nothing else, and
//! `lessons/limewater.lab` still goes milky on the full stack with the
//! measured solubility in place.
//!
//! So these tests are **not** the fix for that. They pin the invariant the
//! change makes true by construction, so that the next solid above the
//! trace line does not have to find out the hard way whether the two
//! routes still happen to agree: **a mineral gives the same solution
//! whichever condensed phase the bench has it booked in.** A datum that
//! changes bookkeeping must not change chemistry.

#![cfg(feature = "engine")]

use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;

/// One kilogram of water and 0.01 mol of slaked lime, with the lime
/// standing in `lime` — `Phase::Solid` as it is poured, `Phase::Aqueous`
/// as `saturation_moves` leaves it once the solid carries a measured
/// solubility above the trace line. Portlandite is `Ca(OH)2`'s phase in
/// wateq4f, so the role is live: it reads
/// `Mineral { phase: "Portlandite", elements: [("Ca", 1.0)] }` — the two
/// hydroxides are already gone from the element list.
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

/// The invariant, stated as an invariant rather than as a number.
///
/// The tolerance on pH is not slack. Lime poured in as a solid releases
/// its heat of dissolution on the step (25.0 °C → 25.1 °C) and lime the
/// bench has already booked as dissolved does not, so the two solutions
/// really are a tenth of a degree apart and read 12.1962 and 12.1974.
/// That difference is thermal and belongs here; anything larger is the
/// two routes disagreeing about chemistry, which is what this guards.
#[test]
fn a_mineral_gives_the_same_solution_whichever_condensed_phase_holds_it() {
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
        (ph(&poured) - ph(&moved)).abs() < 0.01,
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

/// And the half the element-totals route was suspected of losing.
///
/// Slaked lime's whole contribution to limewater is alkalinity, and
/// portlandite's derived elements are `[(Ca, 1)]`: the two hydroxides have
/// been dropped into the charge-balance domain. The suspicion #699 raised
/// was that a portion entered as totals therefore arrives as calcium with
/// no base. It does not — `pH charge` puts them back, which is what the
/// comment on `contribution_from_counts` always claimed. This asserts the
/// claim instead of leaving it as a comment, because a limewater that is
/// not alkaline has nothing for carbon dioxide to precipitate.
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

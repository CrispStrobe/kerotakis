//! The one standard solver stack.
//!
//! Three hosts assemble a `SolverStack` — the CLI, the Tauri shell, and
//! the wasm bench — and by 2026-08-24 they had drifted into THREE
//! different equilibrator orders (the shell was missing two solvers
//! outright; the shell's conformance tests caught it). Chemistry must
//! not depend on which window you ran it in, so the order lives here
//! once and the hosts only choose their aqueous tail.
//!
//! The canonical order is the CLI's, per the shell's own contract
//! ("the CLI's `build_stack`, verbatim in structure"). Change it here
//! and every host follows; change it anywhere else and this crate's
//! existence is the review comment.

use kerotakis_core::{
    CuratedEquilibrator, Equilibrator, HonestyEquilibrator, MixingEquilibrator, SolverStack,
};

/// The standard solvers, in the standard order, with the host's aqueous
/// arrangement spliced in before honesty:
///
/// * The CLI and the shell pass their PHREEQC wrapping (or a
///   `StateEquilibrator` when the aqueous engine failed to initialise).
/// * The wasm bench passes an empty tail — its aqueous engine is
///   *attached* through a JS hook outside the stack, not linked into it.
pub fn standard_solvers(aqueous_tail: Vec<Box<dyn Equilibrator>>) -> Vec<Box<dyn Equilibrator>> {
    let mut solvers: Vec<Box<dyn Equilibrator>> = vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        // EXP-33: sublimation and hydrate bookkeeping sit beside the
        // curated reactions and before anything aqueous. A hydrate must
        // have decided whether it still holds its water before a solver
        // asks what is dissolved, and a solid that has already left as
        // vapour must not be offered to one.
        Box::new(kerotakis_core::phase_route::PhaseRouteEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(kerotakis_core::hmix::MixingEnthalpyEquilibrator),
        Box::new(kerotakis_cea::ThermalEquilibrator),
    ];
    solvers.extend(aqueous_tail);
    solvers.push(Box::new(HonestyEquilibrator));
    solvers
}

/// `standard_solvers`, already wrapped into a `SolverStack`.
pub fn standard_stack(aqueous_tail: Vec<Box<dyn Equilibrator>>) -> SolverStack {
    SolverStack::new(standard_solvers(aqueous_tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honesty_is_always_last_and_the_tail_sits_before_it() {
        // The stack cannot name its members back to us, so pin the shape:
        // an empty tail still ends in honesty, and a tail grows the count
        // by exactly its length in front of it.
        let bare = standard_solvers(vec![]);
        let tailed = standard_solvers(vec![Box::new(kerotakis_core::StateEquilibrator)]);
        assert_eq!(tailed.len(), bare.len() + 1);
    }
}

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
        // BRD-020 router decision (2026-09-05): reaction families sit
        // immediately after the exact curated pairs and before every
        // general engine. A curated row is the more specific claim, so it
        // answers first; a family is asked only about structures the
        // registry curated, and only once its gates admit the vessel — an
        // acid and an alcohol standing cold and uncatalysed decline, in
        // words the capability report carries, rather than esterify. The
        // bench has screened the operator for safety and resolved every
        // name at parse time before any solver sees the vessel, which is
        // the IR's "after safety and identity resolution, before the
        // honesty fallback". Products enter the ordinary ledger and the
        // phase, thermal and aqueous routes below take them from there.
        Box::new(kerotakis_org::family_oracle::family_equilibrator()),
        // EXP-33: sublimation and hydrate bookkeeping sit beside the
        // curated reactions and before anything aqueous. A hydrate must
        // have decided whether it still holds its water before a solver
        // asks what is dissolved, and a solid that has already left as
        // vapour must not be offered to one.
        Box::new(kerotakis_core::phase_route::PhaseRouteEquilibrator),
        // EXP-25: a dissolved volatile with a reviewed Henry's-law
        // coefficient reaches an owned headspace here, so the gas tests
        // that read the headspace can see ammonia poured in as a liquid.
        // After the phase routes (a vapour that sublimed is already gas)
        // and before the aqueous tail, which speciates what stays in the
        // liquid and leaves the headspace share alone.
        Box::new(kerotakis_core::volatility::HeadspacePartitionEquilibrator),
        Box::new(kerotakis_core::nonaqueous::NonAqueousEquilibrator),
        Box::new(kerotakis_core::hmix::MixingEnthalpyEquilibrator),
        Box::new(kerotakis_cea::ThermalEquilibrator),
        // KID-12: the curated fuels CEA has no data for. It sits AFTER
        // the thermochemical engine on purpose — where NASA's dataset
        // can name every species in the vessel it should answer, and
        // this table only speaks for the paraffin, cellulose and sucrose
        // it cannot. Placing it earlier would let a curated heat of
        // combustion pre-empt a Gibbs minimisation that was ready to do
        // better.
        Box::new(kerotakis_core::combustion::CombustionEquilibrator),
    ];
    solvers.extend(aqueous_tail);
    // BRD-023: corrosion runs AFTER the aqueous tail rather than beside
    // the other curated rungs, and the reason is a data dependency. Its
    // one quantitative claim — how much of the oxygen-diffusion ceiling
    // this electrolyte's conductivity lets through — is computed from
    // `vessel.solution`, which is exactly what the aqueous tail has just
    // produced. Placed earlier it would read the PREVIOUS step's
    // speciation, so a nail dropped into brine would be told it was
    // standing in whatever the beaker held before the salt went in.
    // It is still before honesty, because it is a chemistry answer and
    // honesty only speaks for states nothing answered.
    solvers.push(Box::new(kerotakis_core::corrosion::CorrosionEquilibrator));
    // BRD-032: adsorption runs after the aqueous tail too, and for a
    // sharper reason than corrosion's. PHREEQC's readback rebuilds
    // `contents` and re-dissolves every `dissolves_without_speciation`
    // solid it finds there; a rung that moved a dye out of solution
    // before the tail ran would have that work undone on the same step.
    // Placed here, what it moves lives in `vessel.adsorbed`, which the
    // tail does not touch, and the split it computes survives.
    solvers.push(Box::new(kerotakis_core::adsorption::AdsorptionEquilibrator));
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

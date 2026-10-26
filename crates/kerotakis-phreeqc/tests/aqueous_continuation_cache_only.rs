//! Concrete adapter controls for the no-engine/cache-and-hook configuration.
//! No synthetic receipt or replacement readback trait is used here.
#![cfg(not(feature = "engine"))]

use kerotakis_core::aqueous_continuation::TrustedAqueousReadback;
use kerotakis_core::*;
use kerotakis_phreeqc::PhreeqcEquilibrator;
use std::{cell::Cell, rc::Rc};

#[test]
fn concrete_cache_miss_and_failed_hook_cannot_reuse_an_old_solution() {
    let mut vessel = Vessel::new(VesselId(0), "uncached aqueous question");
    vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    vessel.deposit(SpeciesId::new("HCl"), Moles(0.005), Phase::Liquid);
    // Intentionally stale complete-looking reading. The concrete adapter must
    // clear this on its private candidate, not attest it as a computed answer.
    vessel.solution = Some(SolutionInfo {
        solvent_activity: None,
        scope: SolutionScope::Complete,
        solvent_kg: Some(0.1),
        particle_molality: None,
        ph: 7.0,
        ionic_strength: 1e-7,
        pe: None,
        redox: Vec::new(),
        species: Vec::new(),
        provenance: None,
    });
    let before = format!("{vessel:?}");
    let mut aqueous = PhreeqcEquilibrator::new().unwrap();
    assert!(aqueous.characterize_current(&mut vessel).is_err());
    assert_eq!(format!("{vessel:?}"), before);

    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    aqueous.set_hook(Box::new(move |_, _| {
        observed.set(observed.get() + 1);
        Err("controlled current-question hook failure".into())
    }));
    // A materially different question must also fail rather than borrow the
    // stale solution retained by deposit or create a trusted answer receipt.
    vessel.deposit(SpeciesId::new("water"), Moles(0.01), Phase::Liquid);
    let changed = format!("{vessel:?}");
    assert!(aqueous.characterize_current(&mut vessel).is_err());
    assert!(
        calls.get() > 0,
        "the hook must receive the uncached question"
    );
    assert_eq!(format!("{vessel:?}"), changed);
}

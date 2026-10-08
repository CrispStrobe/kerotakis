//! Independent finite component versus aggregate validity boundaries.
use kerotakis_core::{Moles, SolidSolution};

#[test]
fn finite_components_with_overflowing_total_are_invalid() {
    let phase =
        SolidSolution::aragonite_strontianite("overflow total", Moles(f64::MAX), Moles(f64::MAX));
    assert!(phase
        .components
        .iter()
        .all(|entry| entry.moles.0.is_finite()));
    assert!(!phase.total_moles().0.is_finite());
    assert!(
        !phase.has_valid_state(),
        "aggregate overflow must invalidate typed ownership"
    );
}

#[test]
fn finite_total_with_overflowing_mass_is_invalid() {
    let phase = SolidSolution::aragonite_strontianite("overflow mass", Moles(1e307), Moles(0.0));
    assert!(phase.total_moles().0.is_finite());
    assert!(!phase.mass().0.is_finite());
    assert!(
        !phase.has_valid_state(),
        "mass overflow must invalidate typed ownership"
    );
}

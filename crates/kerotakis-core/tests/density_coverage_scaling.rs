//! Physical density coverage follows composition, independently of sample scale.
#[test]
fn missing_ionic_volume_coverage_does_not_depend_on_sample_scale() {
    use kerotakis_core::{Moles, Phase, SpeciesId, Vessel, VesselId};
    let mut baseline = None;
    for scale in [1e-8, 1e-6, 1e-4, 1.0] {
        let mut vessel = Vessel::new(VesselId(0), "scaled dilute brine");
        vessel.deposit(SpeciesId::new("water"), Moles(55.5 * scale), Phase::Liquid);
        vessel.deposit(SpeciesId::new("Na+"), Moles(0.001 * scale), Phase::Aqueous);
        vessel.deposit(SpeciesId::new("Cl-"), Moles(0.001 * scale), Phase::Aqueous);
        let support = kerotakis_core::coverage::observable_support(&vessel, "density");
        assert!(support
            .reasons
            .iter()
            .any(|r| r == "missing-dissolved-ion-volume"));
        if let Some((status, reasons)) = &baseline {
            assert_eq!(&support.status, status);
            assert_eq!(&support.reasons, reasons);
        } else {
            baseline = Some((support.status, support.reasons));
        }
    }
    let mut pure = Vessel::new(VesselId(0), "water control");
    pure.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
    assert!(!kerotakis_core::buoyancy::ionic_volume_unaccounted(&pure));
}

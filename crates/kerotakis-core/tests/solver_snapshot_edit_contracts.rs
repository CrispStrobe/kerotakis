//! Public emptiness and commit behavior for edited complete proposals.
use kerotakis_core::{
    delta::ThermalDelta, orchestrator::diff_vessels, Kelvin, Moles, Phase, SpeciesId, Vessel,
    VesselId,
};

#[test]
fn editing_an_unchanged_snapshot_cannot_hide_nonempty_terms() {
    let mut vessel = Vessel::new(VesselId(0), "beaker");
    vessel.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    let before = format!("{vessel:?}");
    for proposal in [
        diff_vessels(&vessel, &vessel, "edit").with_moles(
            SpeciesId::new("NaCl"),
            Phase::Solid,
            0.1,
        ),
        diff_vessels(&vessel, &vessel, "edit")
            .with_thermal(ThermalDelta::SetTemperature(Kelvin(300.0))),
        diff_vessels(&vessel, &vessel, "edit").with_adsorbed(
            SpeciesId::new("activated_charcoal"),
            SpeciesId::new("methyl_orange"),
            0.001,
        ),
    ] {
        assert!(!proposal.is_empty());
        assert!(proposal.commit(&mut vessel).is_err());
        assert_eq!(format!("{vessel:?}"), before);
    }
}

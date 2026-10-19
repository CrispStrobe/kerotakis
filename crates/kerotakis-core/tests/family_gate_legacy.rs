//! Existing English gate messages are a compatibility field, not a locale ID.
use kerotakis_core::family::*;
use kerotakis_core::{Kelvin, Moles, Phase, SpeciesId, Vessel, VesselId};
use std::collections::BTreeMap;
struct UnusedOracle;
impl StructureOracle for UnusedOracle {
    fn groups_of(&self, _: &str) -> Option<Vec<String>> {
        None
    }
    fn apply(
        &self,
        _: &FamilyRecord,
        _: &[&str],
    ) -> Result<Option<Vec<String>>, kerotakis_core::Phrase> {
        panic!("ledger-only gate control")
    }
}
#[test]
fn dynamic_english_temperature_and_catalyst_messages_remain_exact() {
    for (gates, expected) in [
        (GateSet { temperature_k: Some(KelvinWindow { min: Some(350.0), max: Some(400.0) }), ..GateSet::default() }, "the vessel stands at 298.1 K and the family needs between 350.0 and 400.0 K"),
        (GateSet { catalyst: Some(CatalystGate::AnyOf { keys: vec!["H2SO4".into(), "H+".into(), "unlisted-catalyst".into()] }), ..GateSet::default() }, "no H2SO4 or H+ or unlisted-catalyst in the vessel — the family needs its catalyst present"),
    ] {
        let record = FamilyRecord { id: "legacy-gate-control".into(), version: 1, smirks: String::new(), substrates: Vec::new(), ledger_reactants: BTreeMap::from([("ethanol".into(), 1.0)]), ledger_products: BTreeMap::from([("ethanol".into(), 1.0)]), gates, priority: 0, outcome: OutcomeModel::ToCompletion, confidence: FamilyConfidence::CuratedInstance, provenance: "Gate presentation control only".into(), refusal_domain: "No chemical-yield claim".into() };
        let mut vessel = Vessel::new(VesselId(0), "beaker");
        vessel.temperature = Kelvin(298.125);
        vessel.deposit(SpeciesId::new("ethanol"), Moles(0.01), Phase::Liquid);
        let e = FamilyRouter::new(UnusedOracle, vec![record]).evaluate(&vessel);
        assert_eq!(e.declined.len(), 1);
        assert_eq!(e.declined[0].reason, expected);
    }
}

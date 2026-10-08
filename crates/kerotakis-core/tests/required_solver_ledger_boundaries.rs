//! Source-informed strict opt-in ledger completeness/trace/charge boundaries.
//! This is not a blind independent chemistry experiment batch.
use kerotakis_core::material::MaterialBasis;
use kerotakis_core::vessel::{
    AdsorbedAmount, MaterialLot, MaterialObject, MaterialObjectState, ObjectComponent,
    UnresolvedMaterialPortion,
};
use kerotakis_core::*;

#[derive(Clone, Copy)]
enum Action {
    Warm,
    AddOpaque,
    DropUnresolved,
    DropSalt,
    AddSalt(f64),
    OxidizeIron,
    PhaseChange,
    CreateWaterAndDropLots,
    DropHydrogen,
    AddHydrogen(f64),
    OxidizeTraceIron(f64),
    BalancedRedox(f64),
    DropSolidWater,
    AddSolidWater(f64),
    ConsolidateWaterPhases,
}
struct Route(Action);
impl Equilibrator for Route {
    fn name(&self) -> &'static str {
        "strict-boundary-audit"
    }
    fn element_conservation_tolerance(&self) -> Option<f64> {
        None
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        match self.0 {
            Action::Warm => v.temperature = Kelvin(300.0),
            Action::AddOpaque => v.deposit(
                SpeciesId::new("opaque-audit-molecule"),
                Moles(0.1),
                Phase::Aqueous,
            ),
            Action::DropUnresolved => v.unresolved_materials.clear(),
            Action::DropSalt => v.contents.retain(|p| p.species.0 != "NaCl"),
            Action::AddSalt(n) => v.deposit(SpeciesId::new("NaCl"), Moles(n), Phase::Aqueous),
            Action::OxidizeIron => {
                v.contents.retain(|p| p.species.0 != "Fe+2");
                v.deposit(SpeciesId::new("Fe+3"), Moles(0.1), Phase::Aqueous);
            }
            Action::PhaseChange => {
                for p in &mut v.contents {
                    if p.species.0 == "water" {
                        p.phase = Phase::Solid;
                    }
                }
            }
            Action::DropHydrogen => v.contents.retain(|p| p.species.0 != "H2"),
            Action::AddHydrogen(n) => v.deposit(SpeciesId::new("H2"), Moles(n), Phase::Gas),
            Action::OxidizeTraceIron(n) => {
                v.contents.retain(|p| p.species.0 != "Fe+2");
                v.deposit(SpeciesId::new("Fe+3"), Moles(n), Phase::Aqueous);
            }
            Action::BalancedRedox(n) => {
                v.contents
                    .retain(|p| p.species.0 != "Fe+2" && p.species.0 != "Cl2");
                v.deposit(SpeciesId::new("Fe+3"), Moles(n), Phase::Aqueous);
                v.deposit(SpeciesId::new("Cl-"), Moles(n), Phase::Aqueous);
            }
            Action::DropSolidWater => v
                .contents
                .retain(|p| !(p.species.0 == "water" && p.phase == Phase::Solid)),
            Action::AddSolidWater(n) => v.deposit(SpeciesId::new("water"), Moles(n), Phase::Solid),
            Action::ConsolidateWaterPhases => {
                let total = v.moles_of(&SpeciesId::new("water"));
                v.contents.retain(|p| p.species.0 != "water");
                v.deposit(SpeciesId::new("water"), total, Phase::Solid);
            }
            Action::CreateWaterAndDropLots => {
                v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
                v.lots.clear();
            }
        }
        Ok(vec![])
    }
    fn mix(
        &mut self,
        v: &mut Vessel,
        _: &Vessel,
        _: f64,
        _: &Vessel,
        _: f64,
    ) -> Option<Result<Vec<Event>, SolveError>> {
        Some(self.equilibrate(v))
    }
}
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v
}
fn refused(mut v: Vessel, action: Action) {
    let before = serde_json::to_value(&v).unwrap();
    let mut s =
        SolverStack::with_required_conservation(vec![Box::new(Route(action))], 1e-8).unwrap();
    let events = s.equilibrate(&mut v).unwrap();
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
    assert!(events
        .iter()
        .any(|e| matches!(e,Event::SolverFailed{solver,..} if solver=="strict-boundary-audit")));
    assert!(matches!(
        s.last_routes[0].outcome,
        SolverRouteOutcome::Failed
    ));
}
fn accepted(mut v: Vessel, action: Action) -> Vessel {
    let mut s =
        SolverStack::with_required_conservation(vec![Box::new(Route(action))], 1e-8).unwrap();
    let events = s.equilibrate(&mut v).unwrap();
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::SolverFailed { .. })));
    assert!(matches!(
        s.last_routes[0].outcome,
        SolverRouteOutcome::Succeeded { .. }
    ));
    v
}
fn unresolved() -> UnresolvedMaterialPortion {
    UnresolvedMaterialPortion {
        material: "opaque-audit-material".into(),
        recipe_id: "opaque-audit-recipe".into(),
        recipe_version: 1,
        basis: MaterialBasis::MassFraction,
        amount: 0.1,
        enzyme_hydrolysis: None,
        protein_denatured_fraction: 0.0,
    }
}
fn object(mass_g: f64, components: Vec<ObjectComponent>) -> MaterialObject {
    MaterialObject {
        material: "audit-object".into(),
        recipe_id: "audit-object-recipe".into(),
        recipe_version: 1,
        mass_g,
        components,
        state: MaterialObjectState::default(),
    }
}
fn electrode(key: &str, amount: Option<f64>) -> ElectrodeState {
    serde_json::from_value(serde_json::json!({"label":"audit electrode","material":key,"substrate_moles":amount,"area_m2":0.001,"roughness":1.0})).unwrap()
}
#[test]
fn positive_unknown_bulk_inventory_cannot_receive_a_complete_certificate() {
    let mut v = water();
    v.deposit(
        SpeciesId::new("opaque-audit-molecule"),
        Moles(0.1),
        Phase::Aqueous,
    );
    refused(v, Action::Warm);
}
#[test]
fn custom_route_cannot_create_an_omitted_unknown_species() {
    refused(water(), Action::AddOpaque);
}
#[test]
fn zero_unknown_bulk_portion_does_not_make_a_known_ledger_incomplete() {
    let mut v = water();
    v.deposit(
        SpeciesId::new("opaque-audit-molecule"),
        Moles(0.0),
        Phase::Aqueous,
    );
    assert_eq!(accepted(v, Action::Warm).temperature, Kelvin(300.0));
}
#[test]
fn positive_unknown_bound_sorbate_cannot_receive_a_complete_certificate() {
    let mut v = water();
    v.adsorbed.push(AdsorbedAmount {
        sorbent: SpeciesId::new("carbon"),
        sorbate: SpeciesId::new("opaque-audit-molecule"),
        moles: Moles(0.1),
    });
    refused(v, Action::Warm);
}
#[test]
fn unknown_prepared_object_component_is_not_silently_skipped() {
    let mut v = water();
    v.material_objects.push(object(
        0.1,
        vec![ObjectComponent {
            species: SpeciesId::new("opaque-audit-molecule"),
            moles: Moles(0.1),
        }],
    ));
    refused(v, Action::Warm);
}
#[test]
fn unknown_finite_electrode_substrate_is_owned_unaccounted_matter() {
    let mut v = water();
    v.electrodes
        .push(electrode("opaque-audit-electrode", Some(0.1)));
    refused(v, Action::Warm);
}
#[test]
fn external_electrode_with_no_owned_inventory_does_not_block_element_closure() {
    let mut v = water();
    v.electrodes.push(electrode("opaque-audit-electrode", None));
    assert_eq!(accepted(v, Action::Warm).temperature, Kelvin(300.0));
}
#[test]
fn positive_unresolved_named_material_requires_an_explicit_incomplete_ledger_refusal() {
    let mut v = water();
    v.unresolved_materials.push(unresolved());
    refused(v, Action::Warm);
}
#[test]
fn deleting_unresolved_mass_cannot_hide_behind_unchanged_known_elements() {
    let mut v = water();
    v.unresolved_materials.push(unresolved());
    refused(v, Action::DropUnresolved);
}
#[test]
fn prepared_object_without_resolved_components_has_no_complete_element_ledger() {
    let mut v = water();
    v.material_objects.push(object(0.1, vec![]));
    refused(v, Action::Warm);
}
#[test]
fn known_object_components_do_not_certify_an_unexplained_mass_remainder() {
    let mut v = water();
    v.material_objects.push(object(
        1.0,
        vec![ObjectComponent {
            species: SpeciesId::new("water"),
            moles: Moles(0.01),
        }],
    ));
    refused(v, Action::Warm);
}
#[test]
fn fully_resolved_mass_matched_object_can_receive_a_certificate() {
    let mut v = water();
    let amount = 0.01;
    let mass = species::lookup(&SpeciesId::new("water"))
        .unwrap()
        .molar_mass
        * amount;
    v.material_objects.push(object(
        mass,
        vec![ObjectComponent {
            species: SpeciesId::new("water"),
            moles: Moles(amount),
        }],
    ));
    assert_eq!(accepted(v, Action::Warm).temperature, Kelvin(300.0));
}
#[test]
fn unique_trace_element_loss_is_not_dismissed_by_an_absolute_floor() {
    for amount in [1e-16, 1e-200, f64::from_bits(1)] {
        let mut v = water();
        v.deposit(SpeciesId::new("NaCl"), Moles(amount), Phase::Aqueous);
        refused(v, Action::DropSalt);
    }
}
#[test]
fn creating_a_new_trace_element_from_zero_is_not_dismissed_by_an_absolute_floor() {
    for amount in [1e-16, 1e-200, f64::from_bits(1)] {
        refused(water(), Action::AddSalt(amount));
    }
}
#[test]
fn oxidation_state_change_without_electron_boundary_cannot_change_total_charge() {
    let mut v = water();
    assert!(species::lookup(&SpeciesId::new("Fe+2")).is_some());
    assert!(species::lookup(&SpeciesId::new("Fe+3")).is_some());
    v.deposit(SpeciesId::new("Fe+2"), Moles(0.1), Phase::Aqueous);
    refused(v, Action::OxidizeIron);
}
#[test]
fn custom_mix_checks_charge_closure_as_well_as_elements() {
    let mut v = water();
    v.deposit(SpeciesId::new("Fe+2"), Moles(0.1), Phase::Aqueous);
    let before = serde_json::to_value(&v).unwrap();
    let a = water();
    let b = water();
    let mut s =
        SolverStack::with_required_conservation(vec![Box::new(Route(Action::OxidizeIron))], 1e-8)
            .unwrap();
    assert!(s.mix(&mut v, &a, 0.5, &b, 0.5).unwrap().is_err());
    assert_eq!(serde_json::to_value(&v).unwrap(), before);
}
#[test]
fn legitimate_phase_change_does_not_violate_element_and_charge_closure() {
    let v = accepted(water(), Action::PhaseChange);
    assert!(v.contents.iter().all(|p| p.phase == Phase::Solid));
}
#[test]
fn addition_lots_are_provenance_not_a_second_owned_inventory() {
    let mut v = water();
    v.lots.push(MaterialLot {
        species: SpeciesId::new("water"),
        moles: Moles(2.0),
        phase: Phase::Liquid,
        added_at: 0.0,
        hydrated_at: None,
        source: Some("audit bottle".into()),
        particle_size_um: None,
        suspended_fraction: None,
    });
    // Removing a historical lot is not a physical debit that can finance new water.
    refused(v, Action::CreateWaterAndDropLots);
}

#[test]
fn trace_reaction_loss_is_not_hidden_by_unchanged_water_element_background() {
    let mut v = water();
    v.deposit(SpeciesId::new("H2"), Moles(1e-200), Phase::Gas);
    refused(v, Action::DropHydrogen);
}
#[test]
fn trace_reaction_creation_is_not_hidden_by_unchanged_water_element_background() {
    refused(water(), Action::AddHydrogen(1e-200));
}
#[test]
fn trace_charge_change_is_not_hidden_by_inert_charged_background() {
    let mut v = water();
    v.deposit(SpeciesId::new("Cu+2"), Moles(1.0), Phase::Aqueous);
    v.deposit(SpeciesId::new("Fe+2"), Moles(1e-200), Phase::Aqueous);
    refused(v, Action::OxidizeTraceIron(1e-200));
}
#[test]
fn truly_balanced_trace_redox_can_close_atoms_and_charge() {
    let mut v = water();
    let amount = 1e-200;
    assert!(species::lookup(&SpeciesId::new("Cl-")).is_some());
    v.deposit(SpeciesId::new("Fe+2"), Moles(amount), Phase::Aqueous);
    v.deposit(SpeciesId::new("Cl2"), Moles(amount * 0.5), Phase::Gas);
    let v = accepted(v, Action::BalancedRedox(amount));
    assert_eq!(v.moles_of(&SpeciesId::new("Fe+3")), Moles(amount));
    assert_eq!(v.moles_of(&SpeciesId::new("Cl-")), Moles(amount));
}
#[test]
fn trace_loss_across_phase_ownership_cannot_disappear_in_species_aggregation() {
    let mut v = water();
    v.deposit(SpeciesId::new("water"), Moles(1e-200), Phase::Solid);
    refused(v, Action::DropSolidWater);
}
#[test]
fn trace_creation_across_phase_ownership_cannot_disappear_in_species_aggregation() {
    refused(water(), Action::AddSolidWater(1e-200));
}
#[test]
fn ordinary_mixed_phase_ownership_can_be_combined_without_double_counting() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Solid);
    let v = accepted(v, Action::ConsolidateWaterPhases);
    assert_eq!(v.moles_of(&SpeciesId::new("water")), Moles(2.0));
    assert_eq!(v.contents.len(), 1);
}

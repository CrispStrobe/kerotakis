//! Existing-API contracts for canonical electrode schema at solver acceptance.
use kerotakis_core::{
    compartment::{ElectrodeDeposit, ElectrodeInterfacialSpecies, ElectrodeState},
    orchestrator::diff_vessels,
    *,
};
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v
}
fn electrode() -> ElectrodeState {
    ElectrodeState {
        label: "anode".into(),
        material: "Pt".into(),
        surface_preparation: None,
        substrate_moles: None,
        area_m2: 0.001,
        roughness: 1.0,
        double_layer_capacitance_f_per_m2: None,
        interfacial_potential_v: None,
        interfacial_species: vec![],
        diagnostics: None,
        deposits: vec![],
    }
}
fn interface() -> ElectrodeInterfacialSpecies {
    ElectrodeInterfacialSpecies {
        reaction_id: "reaction".into(),
        species: "Cu+2".into(),
        surface_concentration_mol_per_m3: 1.0,
    }
}
fn malformed() -> Vec<ElectrodeState> {
    let mut rows = vec![];
    let mut e = electrode();
    e.area_m2 = 0.0;
    rows.push(e);
    let mut e = electrode();
    e.roughness = 0.0;
    rows.push(e);
    let mut e = electrode();
    e.label = " ".into();
    rows.push(e);
    let mut e = electrode();
    e.material = " ".into();
    rows.push(e);
    let mut e = electrode();
    e.surface_preparation = Some(" ".into());
    rows.push(e);
    let mut e = electrode();
    let mut i = interface();
    i.reaction_id = " ".into();
    e.interfacial_species.push(i);
    rows.push(e);
    let mut e = electrode();
    let mut i = interface();
    i.species = " ".into();
    e.interfacial_species.push(i);
    rows.push(e);
    let mut e = electrode();
    e.interfacial_species = vec![interface(), interface()];
    rows.push(e);
    let mut e = electrode();
    e.deposits.push(ElectrodeDeposit {
        species: " ".into(),
        moles: 0.1,
        thickness_m: None,
        coverage_fraction: None,
        effect: None,
        electrical_resistivity_ohm_m: None,
    });
    rows.push(e);
    for e in &rows {
        assert!(e.validate().is_err());
    }
    rows
}
struct Route(Vec<ElectrodeState>);
impl Equilibrator for Route {
    fn name(&self) -> &'static str {
        "electrode-schema-trial"
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        v.electrodes = self.0.clone();
        v.honesty_said.push("trial".into());
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
fn snapshot_refuses(electrodes: Vec<ElectrodeState>) {
    let mut v = water();
    let before = format!("{v:?}");
    let mut after = v.clone();
    after.electrodes = electrodes;
    after.honesty_said.push("trial".into());
    assert!(diff_vessels(&v, &after, "schema").commit(&mut v).is_err());
    assert_eq!(format!("{v:?}"), before);
}
fn direct_refuses(electrodes: Vec<ElectrodeState>, native_mix: bool) {
    let mut v = water();
    let before = format!("{v:?}");
    let source = v.clone();
    let mut stack = SolverStack::new(vec![Box::new(Route(electrodes))]);
    if native_mix {
        assert!(stack
            .mix(&mut v, &source, 0.5, &source, 0.5)
            .unwrap()
            .is_err());
    } else {
        assert!(matches!(
            &stack.equilibrate(&mut v).unwrap()[..],
            [Event::SolverFailed { .. }]
        ));
    }
    assert_eq!(format!("{v:?}"), before);
}
#[test]
fn malformed_canonical_electrodes_refuse_complete_snapshot_commits() {
    for e in malformed() {
        snapshot_refuses(vec![e]);
    }
}
#[test]
fn malformed_canonical_electrodes_refuse_successful_direct_routes() {
    for e in malformed() {
        direct_refuses(vec![e], false);
    }
}
#[test]
fn malformed_canonical_electrodes_refuse_successful_mix_routes() {
    for e in malformed() {
        direct_refuses(vec![e], true);
    }
}
#[test]
fn duplicate_electrode_labels_refuse_all_commit_paths() {
    for native_mix in [false, true] {
        direct_refuses(vec![electrode(), electrode()], native_mix);
    }
    snapshot_refuses(vec![electrode(), electrode()]);
}
#[test]
fn valid_transient_electrode_snapshot_preserves_complete_state() {
    let mut v = water();
    let mut after = v.clone();
    let mut e = electrode();
    e.interfacial_species.push(interface());
    e.double_layer_capacitance_f_per_m2 = Some(0.2);
    e.interfacial_potential_v = Some(-0.1);
    assert!(e.validate().is_ok());
    after.electrodes.push(e);
    diff_vessels(&v, &after, "valid").commit(&mut v).unwrap();
    assert_eq!(format!("{v:?}"), format!("{after:?}"));
}
#[test]
fn valid_direct_and_mix_routes_keep_electrodes_and_narration() {
    for native_mix in [false, true] {
        let mut v = water();
        let source = v.clone();
        let mut e = electrode();
        e.interfacial_species.push(interface());
        let mut expected = v.clone();
        expected.electrodes = vec![e.clone()];
        expected.honesty_said.push("trial".into());
        let mut stack = SolverStack::new(vec![Box::new(Route(vec![e]))]);
        if native_mix {
            assert!(stack
                .mix(&mut v, &source, 0.5, &source, 0.5)
                .unwrap()
                .is_ok());
        } else {
            assert!(stack.equilibrate(&mut v).unwrap().is_empty());
        }
        assert_eq!(format!("{v:?}"), format!("{expected:?}"));
    }
}
#[test]
fn same_species_in_distinct_interface_reactions_remains_supported() {
    let mut e = electrode();
    let mut second = interface();
    second.reaction_id = "other".into();
    e.interfacial_species = vec![interface(), second];
    assert!(e.validate().is_ok());
    let mut v = water();
    let mut after = v.clone();
    after.electrodes.push(e);
    diff_vessels(&v, &after, "distinct-keys")
        .commit(&mut v)
        .unwrap();
    assert_eq!(format!("{v:?}"), format!("{after:?}"));
}
#[test]
fn distinct_electrode_labels_remain_supported() {
    let mut second = electrode();
    second.label = "cathode".into();
    let mut v = water();
    let mut after = v.clone();
    after.electrodes = vec![electrode(), second];
    diff_vessels(&v, &after, "distinct-electrodes")
        .commit(&mut v)
        .unwrap();
    assert_eq!(format!("{v:?}"), format!("{after:?}"));
}

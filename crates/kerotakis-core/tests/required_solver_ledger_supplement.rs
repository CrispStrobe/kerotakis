//! Supplementary strict owner/boundary coverage, frozen before implementation.
use kerotakis_core::nuclide::Nuclide;
use kerotakis_core::vessel::{
    ExchangeIon, ExchangeOccupancy, ExchangeSites, SoapScumState, SurfaceModel, SurfaceSites,
};
use kerotakis_core::*;
#[derive(Clone, Copy)]
enum Action {
    Warm,
    ExportHydrogen(bool),
    ImportHydrogen,
    FalseExport,
    ExportOxidizedIron,
}
struct Route(Action);
impl Equilibrator for Route {
    fn name(&self) -> &'static str {
        "strict-owner-audit"
    }
    fn element_conservation_tolerance(&self) -> Option<f64> {
        None
    }
    fn equilibrate(&mut self, v: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = vec![];
        match self.0 {
            Action::Warm => v.temperature = Kelvin(300.0),
            Action::ExportHydrogen(correct) => {
                v.contents.retain(|p| p.species.0 != "H2");
                events.push(Event::GasEvolved {
                    vessel: v.id,
                    species: SpeciesId::new("H2"),
                    moles: Moles(if correct { 1e-200 } else { 2e-200 }),
                });
            }
            Action::ImportHydrogen => {
                v.deposit(SpeciesId::new("H2"), Moles(1e-200), Phase::Aqueous);
                events.push(Event::GasAbsorbed {
                    vessel: v.id,
                    species: SpeciesId::new("H2"),
                    moles: Moles(1e-200),
                });
            }
            Action::FalseExport => events.push(Event::GasEvolved {
                vessel: v.id,
                species: SpeciesId::new("H2"),
                moles: Moles(1e-200),
            }),
            Action::ExportOxidizedIron => {
                v.contents.retain(|p| p.species.0 != "Fe+2");
                events.push(Event::GasEvolved {
                    vessel: v.id,
                    species: SpeciesId::new("Fe+3"),
                    moles: Moles(0.1),
                });
            }
        }
        Ok(events)
    }
}
fn water() -> Vessel {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(2.0), Phase::Liquid);
    v
}
fn run(mut v: Vessel, action: Action, accept: bool) {
    let before = serde_json::to_value(&v).unwrap();
    let mut s =
        SolverStack::with_required_conservation(vec![Box::new(Route(action))], 1e-8).unwrap();
    let events = s.equilibrate(&mut v).unwrap();
    if accept {
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
    } else {
        assert_eq!(serde_json::to_value(&v).unwrap(), before);
        assert!(events
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. })));
    }
}
#[test]
fn unrepresented_surface_oxide_mass_cannot_receive_a_complete_certificate() {
    let mut v = water();
    v.surfaces.push(SurfaceSites {
        label: "oxide".into(),
        model: SurfaceModel::HydrousFerricOxide,
        mass: Grams(1.0),
        specific_area_m2_per_g: 600.0,
        strong_capacity: Moles(0.001),
        weak_capacity: Moles(0.01),
        occupancy: vec![],
        water_release: Moles(0.0),
    });
    run(v, Action::Warm, false);
}
#[test]
fn unrepresented_exchanger_support_mass_cannot_receive_a_complete_certificate() {
    let mut v = water();
    v.exchanges.push(ExchangeSites {
        label: "resin".into(),
        dry_mass: Grams(1.0),
        capacity: Moles(0.1),
        occupancy: vec![ExchangeOccupancy {
            ion: ExchangeIon::Sodium,
            moles: Moles(0.1),
        }],
    });
    run(v, Action::Warm, false);
}
#[test]
fn soap_scum_aggregate_mass_has_no_complete_molecular_inventory() {
    let mut v = water();
    v.soap_scum = Some(SoapScumState {
        aggregate_mass_g: 0.1,
        divalent_ion_moles: 0.001,
        soap_equivalent_moles: 0.002,
    });
    run(v, Action::Warm, false);
}
#[test]
fn separate_positive_nuclide_inventory_requires_a_separate_certificate() {
    let mut v = water();
    v.nuclides.deposit(Nuclide::new("C", 14), 1e-12);
    run(v, Action::Warm, false);
}
#[test]
fn matched_trace_gas_export_closes_without_an_absolute_floor() {
    let mut v = water();
    v.deposit(SpeciesId::new("H2"), Moles(1e-200), Phase::Gas);
    run(v, Action::ExportHydrogen(true), true);
}
#[test]
fn mismatched_trace_gas_export_cannot_hide_in_large_water_background() {
    let mut v = water();
    v.deposit(SpeciesId::new("H2"), Moles(1e-200), Phase::Gas);
    run(v, Action::ExportHydrogen(false), false);
}
#[test]
fn matched_trace_gas_import_closes_without_an_absolute_floor() {
    run(water(), Action::ImportHydrogen, true);
}
#[test]
fn unbacked_trace_gas_narration_is_not_a_conserving_boundary() {
    run(water(), Action::FalseExport, false);
}
#[test]
fn gas_boundary_must_balance_formal_charge_as_well_as_atoms() {
    let mut v = water();
    v.deposit(SpeciesId::new("Fe+2"), Moles(0.1), Phase::Aqueous);
    run(v, Action::ExportOxidizedIron, false);
}
#[test]
fn a_lost_third_amount_correction_cannot_receive_an_exact_inventory_certificate() {
    let mut v = Vessel::new(VesselId(0), "beaker");
    v.deposit(SpeciesId::new("water"), Moles(1.0), Phase::Liquid);
    v.deposit(SpeciesId::new("water"), Moles(1e-20), Phase::Solid);
    v.deposit(SpeciesId::new("water"), Moles(1e-200), Phase::Aqueous);
    run(v, Action::Warm, false);
}

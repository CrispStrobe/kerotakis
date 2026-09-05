//! # kerotakis-core
//!
//! Bench and vessel state machine for the Kerotakis virtual laboratory:
//! operators, energy balance, solver router, L0 safety hook, and register
//! rendering. This crate is the invariant of the project — it compiles to
//! `wasm32-unknown-unknown` and all five native targets from one source, and
//! every client (CLI, wasm, mobile) consumes the same public API.
//!
//! See PLAN.md at the repository root for the architecture this implements.

/// The smallest amount worth telling a user about: bookkeeping stays
/// exact, but observation has a detection limit, exactly as instruments
/// do. A nanomole of chlorine is not a gas cloud, and reporting it as one
/// is a lie of scale.
pub const OBSERVABLE_MOLES: f64 = 1e-6;

pub mod adsorption;
pub mod apparatus;
pub mod appearance;
pub mod authority;
pub mod bench;
pub mod buoyancy;
pub mod butler_volmer;
pub mod cabinet;
pub mod cache_key;
pub mod catalog;
pub mod centrifuge;
pub mod chart;
pub mod chemiluminescence;
pub mod clock;
pub mod combustion;
pub mod compartment;
pub mod conductivity;
pub mod constants;
pub mod corrosion;
pub mod coverage;
pub mod curated;
pub mod curdling;
pub mod delta;
pub mod displacement;
pub mod electrochemistry;
pub mod element_coverage;
pub mod emulsion;
pub mod enzyme;
pub mod enzyme_activity;
pub mod exact_stoich;
pub mod family;
pub mod fermentation;
pub mod foam;
pub mod gas_tests;
pub mod gel;
pub mod hmix;
pub mod i18n;
pub mod indicator;
pub mod instrument;
pub mod intern;
pub mod ionic;
pub mod kinetics;
pub mod kitchen_biology;
pub mod ledger;
pub mod material;
pub mod molecule;
pub mod nonaqueous;
pub mod nuclide;
pub mod ops;
pub mod orchestrator;
pub mod packs;
pub mod packs_manifest;
pub mod parallel;
pub mod particles;
pub mod phase_route;
pub mod photochem;
pub mod pigment;
pub mod plastics;
pub mod polymer;
pub mod properties;
pub mod protein;
pub mod relations;
pub mod render;
pub mod rheology;
pub mod scene;
pub mod script;
pub mod selectivity;
pub mod senses;
pub mod solve;
pub mod species;
pub mod species_loader;
pub mod spectrum;
pub mod spill;
mod starch_iodine;
pub mod states;
pub mod statistics;
pub mod stock;
pub mod stoich;
pub mod surface_colour;
pub mod surface_spread;
pub mod swelling;
pub mod transport;
pub mod units;
pub mod uv;
pub mod vessel;

pub use appearance::{observe, Appearance};
pub use bench::{Bench, BenchError};
pub use compartment::{
    Compartment, ElectrodeDeposit, ElectrodeState, Environment, Interface, InterfaceKind,
    VolumeMode,
};
pub use coverage::{coverage_manifest, CoverageReport, SolverCoverage};
pub use curated::CuratedEquilibrator;
pub use delta::{DeltaError, MoleDelta, StateDelta, ThermalDelta};
pub use displacement::DisplacementEquilibrator;
pub use element_coverage::{
    element_coverage_json, element_coverage_report, element_coverage_report_with_lessons,
    element_coverage_report_with_routes, ElementCapability, ElementCoverageEntry,
    ElementCoverageError, ElementCoverageReport, ElementShelfItem, InstalledLessonRoute,
    InstalledRunnableRoute, RunnableElementRoute, ShelfItemKind, ELEMENT_SYMBOLS,
};
pub use i18n::Locale;
pub use instrument::{
    read_transition, Balance, ConductivityMeter, InstrumentContract, InstrumentMode,
    MeltingPointApparatus, PhMeter, PressureGauge, PurityVerdict, Reading, Thermometer,
    TransitionRead, TransitionReading,
};
pub use ionic::{net_ionic, net_ionic_for, IonTerm, IonicBasis, NetIonic};
pub use ledger::{audit_conservation, ConservedLedger};
pub use ops::{Event, Instrument, LogEntry, Operator, PolymerState};
pub use orchestrator::Orchestrator;
pub use phase_route::PhaseRouteEquilibrator;
pub use pigment::{opaque_mixture_colour, PigmentAmount, PigmentMixError, PigmentOptics};
pub use render::{
    localize_event, localize_events, render_event, render_event_in, render_events,
    render_events_in, render_ionic, render_ionic_for, render_ionic_in, render_vessel,
    render_vessel_in, Register,
};
pub use scene::{scene, scene_of, scene_vessel, Scene, SceneStockBottle, SceneVessel};
pub use solve::{
    equilibrate_phase_coupled, Applicability, CapabilityReport, Equilibrator, HonestyEquilibrator,
    MixingEquilibrator, PermissiveScreen, PhaseEquilibrator, SafetyScreen, SafetyVerdict, Severity,
    SolveError, SolverRoute, SolverRouteKind, SolverRouteOutcome, SolverStack, StateEquilibrator,
    ValidityBounds, PHASE_COUPLED_TEMPERATURE_TOLERANCE_K,
};
pub use species::{Colour, Phase, SpeciesId};
pub use spectrum::{Rgb, Spectrum};
pub use spill::SpillCompartment;
pub use stock::{stock_unit, StockAmount, StockLedger, StockRefusal, StockUnit};
pub use transport::{
    CellChain, CellReaction, MobileParcel, ReactiveTransportError, ReactiveTransportStep,
    TransportError, TransportStep,
};
pub use units::{Grams, Joules, Kelvin, Liters, Moles, Pascal};
pub use vessel::{
    ExchangeIon, ExchangeOccupancy, ExchangeSites, Headspace, Portion, Provenance, RedoxState,
    SolidSolution, SolidSolutionAmount, SolidSolutionComponent, SolidSolutionModel, SolutionInfo,
    SpeciesDetail, SurfaceModel, SurfaceOccupancy, SurfaceSiteKind, SurfaceSites, SurfaceSorbate,
    ThermalMode, Vessel, VesselId,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermal_mixing_of_hot_and_cold_water() {
        // Equal amounts of water at 20 °C and 80 °C meet in the middle.
        let mut bench = Bench::new();
        let v = VesselId(0);
        bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
                at: Some(Kelvin::from_celsius(20.0)),
            })
            .unwrap();
        bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(1.0),
                at: Some(Kelvin::from_celsius(80.0)),
            })
            .unwrap();
        let t = bench.vessel(v).unwrap().temperature.to_celsius();
        assert!((t - 50.0).abs() < 1e-9, "expected 50 °C, got {t}");
    }

    #[test]
    fn heating_raises_temperature_by_q_over_cp() {
        let mut bench = Bench::new();
        let v = VesselId(0);
        bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(10.0), // Cp = 753 J/K
                at: None,
            })
            .unwrap();
        bench
            .step(Operator::Heat {
                vessel: v,
                energy: Joules(7530.0),
            })
            .unwrap();
        let t = bench.vessel(v).unwrap().temperature.0;
        assert!((t - (298.15 + 10.0)).abs() < 1e-9);
    }

    #[test]
    fn decant_conserves_mass_and_enthalpy() {
        let mut bench = Bench::new();
        bench.step(Operator::NewVessel { kind: None }).unwrap();
        let (a, b) = (VesselId(0), VesselId(1));
        bench
            .step(Operator::Add {
                vessel: a,
                species: SpeciesId::new("water"),
                moles: Moles(2.0),
                at: Some(Kelvin::from_celsius(60.0)),
            })
            .unwrap();
        let before_h = bench.total_enthalpy().0;
        let before_n = bench.total_moles(&SpeciesId::new("water")).0;
        bench
            .step(Operator::Decant {
                from: a,
                to: b,
                fraction: 0.25,
            })
            .unwrap();
        assert!((bench.total_moles(&SpeciesId::new("water")).0 - before_n).abs() < 1e-12);
        assert!((bench.total_enthalpy().0 - before_h).abs() < 1e-6);
    }

    #[test]
    fn solid_in_liquid_is_honestly_not_modeled() {
        let mut bench = Bench::new();
        let v = VesselId(0);
        bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new("water"),
                moles: Moles(5.0),
                at: None,
            })
            .unwrap();
        let events = bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new("NaCl"),
                moles: Moles(0.1),
                at: None,
            })
            .unwrap();
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::NotYetModeled { .. })),
            "salt in water must be flagged honest-unmodelled until L2 lands"
        );
    }

    #[test]
    fn measure_never_mutates() {
        let mut bench = Bench::new();
        let v = VesselId(0);
        bench
            .step(Operator::Add {
                vessel: v,
                species: SpeciesId::new("ethanol"),
                moles: Moles(1.0),
                at: None,
            })
            .unwrap();
        let snapshot = serde_json::to_string(&bench.vessels).unwrap();
        bench
            .step(Operator::Measure {
                vessel: v,
                instrument: Instrument::Balance,
            })
            .unwrap();
        bench
            .step(Operator::Measure {
                vessel: v,
                instrument: Instrument::Thermometer,
            })
            .unwrap();
        assert_eq!(snapshot, serde_json::to_string(&bench.vessels).unwrap());
    }
}

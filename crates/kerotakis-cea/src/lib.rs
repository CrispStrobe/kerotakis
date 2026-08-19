//! # kerotakis-cea
//!
//! Thermochemistry from NASA CEA's `thermo.inp` (Apache-2.0): NASA-9
//! polynomials, formation enthalpies, and — building on them — gas and
//! condensed-phase equilibrium, the L2g layer that makes `heat` and
//! `ignite` computed chemistry rather than curated lookups (PLAN.md).
//!
//! Every species record carries its own literature citation, which we keep
//! as provenance.

pub mod gibbs;
pub mod nasa9;
pub mod thermal;

pub use gibbs::{equilibrate_hp, equilibrate_tp, CeaError, Equilibrium};
pub use nasa9::{db, Species, ThermoDb, R, T_REF};
pub use thermal::{cea_name, ThermalEquilibrator};

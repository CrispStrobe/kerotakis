//! Typed source records for Kerotakis chemistry data.
//!
//! This crate is deliberately upstream of the runtime registry. It describes
//! reviewed source material without deciding how a compact app pack stores or
//! resolves it. DATA-002 exports today's handwritten registry into this shape;
//! DATA-003 compiles it into a deterministic binary pack; DATA-004 loads that
//! pack behind the runtime registry API.

mod adapter;
pub mod chebi;
pub mod fluid_parameters;
mod material_history;
pub mod model_pack;
mod pack;
mod property_bounds;
mod provenance;
mod pubchem;
mod resolve;
mod runtime_scalar_records;
mod schema;
mod units;
pub mod usda;
mod validate;

pub use adapter::*;
pub use material_history::*;
pub use model_pack::{ModelPackManifest, PackContents, PackLane, PackRejectReason};
pub use pack::{
    build_pack, load_pack, serialize_pack_payload, PackError, PACK_MAGIC, PACK_VERSION,
};
pub use property_bounds::{
    normalize_runtime_scalar, validate_runtime_scalar, RuntimeScalarProperty,
};
pub use provenance::*;
pub use pubchem::*;
pub use resolve::{resolve_phase_property, Conditions, Resolution, ResolvedValue, Rung};
pub use runtime_scalar_records::{select_runtime_scalar_records, RuntimeScalarRecords};
pub use schema::*;
pub use units::*;
pub use validate::{ValidationError, ValidationIssue};

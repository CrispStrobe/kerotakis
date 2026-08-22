//! Typed source records for Kerotakis chemistry data.
//!
//! This crate is deliberately upstream of the runtime registry. It describes
//! reviewed source material without deciding how a compact app pack stores or
//! resolves it. DATA-002 exports today's handwritten registry into this shape;
//! DATA-003 compiles it into a deterministic binary pack; DATA-004 loads that
//! pack behind the runtime registry API.

mod pack;
mod resolve;
mod schema;
mod validate;

pub use pack::{load_pack, PackError, PACK_MAGIC, PACK_VERSION};
pub use resolve::{resolve_phase_property, Conditions, Resolution, ResolvedValue, Rung};
pub use schema::*;
pub use validate::{ValidationError, ValidationIssue};

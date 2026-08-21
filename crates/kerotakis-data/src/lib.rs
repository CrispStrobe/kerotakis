//! Typed source records for Kerotakis chemistry data.
//!
//! This crate is deliberately upstream of the runtime registry. It describes
//! reviewed source material without deciding how a compact app pack stores or
//! resolves it. DATA-002 exports today's handwritten registry into this shape;
//! DATA-003 compiles it. Keeping those steps separate lets this contract become
//! strict before any runtime behavior depends on it.

mod schema;
mod validate;

pub use schema::*;
pub use validate::{ValidationError, ValidationIssue};

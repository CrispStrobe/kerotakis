//! BRD-030 spike library: the adapter prototype only.
//!
//! Split from the binary so the wasm verdict can be measured on the part
//! that would actually ship if BRD-032 ever went ahead — the adapter over
//! feos — without dragging in the corpus driver's filesystem and JSON use,
//! which a browser build would never have.

pub mod adapter;

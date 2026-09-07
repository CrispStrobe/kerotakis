//! Low-resource check of the production numerical kernel while the workspace
//! compile is contended. This does not replace the integrated workspace gate.
//! Only the independent public request type is mirrored, not the algorithm.
mod vle {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum StillTake {
        Fraction(f64),
        EnergyKj(f64),
    }
}

#[path = "../../crates/kerotakis-thermo/src/batch.rs"]
mod batch;

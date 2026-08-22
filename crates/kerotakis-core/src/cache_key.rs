//! PERF-002: Node-level cache keys.
//!
//! A cache key identifies a unique combination of model version, dataset
//! hash, input state, and precision settings so that previously computed
//! results can be reused without re-solving.

use serde::{Deserialize, Serialize};

/// A cache key for a computed result. Two identical keys mean the
/// computation would produce the same output.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    /// Solver name and version.
    pub solver: String,
    /// Dataset identifier (e.g. "phreeqc.dat" hash).
    pub dataset: String,
    /// Hash of the input state (species, amounts, T, P).
    pub input_hash: String,
    /// Precision/tolerance settings that affect the result.
    pub precision: String,
}

impl CacheKey {
    /// Build a cache key from components.
    pub fn new(
        solver: impl Into<String>,
        dataset: impl Into<String>,
        input_hash: impl Into<String>,
        precision: impl Into<String>,
    ) -> Self {
        Self {
            solver: solver.into(),
            dataset: dataset.into(),
            input_hash: input_hash.into(),
            precision: precision.into(),
        }
    }

    /// A simple string representation for use as a filesystem cache path.
    pub fn to_path_component(&self) -> String {
        format!(
            "{}_{}_{}_{}",
            self.solver, self.dataset, self.input_hash, self.precision
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_keys_are_deterministic() {
        let k1 = CacheKey::new("PHREEQC-3.6", "abc123", "def456", "default");
        let k2 = CacheKey::new("PHREEQC-3.6", "abc123", "def456", "default");
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_inputs_produce_different_keys() {
        let k1 = CacheKey::new("PHREEQC-3.6", "abc123", "input1", "default");
        let k2 = CacheKey::new("PHREEQC-3.6", "abc123", "input2", "default");
        assert_ne!(k1, k2);
    }
}

//! OPT-4: SpeciesId string interning via lasso.
//!
//! The current SpeciesId(String) allocates a new string for every
//! species reference. This module provides a global intern table
//! that makes SpeciesId lookups O(1) and Copy-friendly.
//!
//! Not yet wired into SpeciesId (that requires the DATA-010 lifetime
//! refactor). The infrastructure is ready for when it lands.

use lasso::{Spur, ThreadedRodeo};
use std::sync::OnceLock;

static INTERN: OnceLock<ThreadedRodeo> = OnceLock::new();

/// Get or create the global species string interner.
pub fn interner() -> &'static ThreadedRodeo {
    INTERN.get_or_init(ThreadedRodeo::default)
}

/// Intern a species key and return a compact handle.
pub fn intern(key: &str) -> Spur {
    interner().get_or_intern(key)
}

/// Resolve an interned handle back to its string.
pub fn resolve(key: Spur) -> &'static str {
    interner().resolve(&key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_round_trips() {
        let k1 = intern("water");
        let k2 = intern("water");
        assert_eq!(k1, k2, "same string → same key");
        assert_eq!(resolve(k1), "water");
    }

    #[test]
    fn different_strings_get_different_keys() {
        let k1 = intern("NaCl");
        let k2 = intern("AgNO3");
        assert_ne!(k1, k2);
    }

    #[test]
    fn interned_key_is_copy() {
        let k = intern("CaCO3");
        let k2 = k; // Copy, not move
        assert_eq!(resolve(k), resolve(k2));
    }
}

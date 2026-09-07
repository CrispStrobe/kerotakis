//! Reviewed gas uptake constants with explicit standard-state conversion.
//! Native equilibrium computes arbitrary finite inventories, not canned yields.

pub const SOURCE_ID: &str = "sander-2023-hbr-reference";
pub const REFERENCE_TEMPERATURE_K: f64 = 298.15;
pub const EXTENSION: &[u8] = include_bytes!("../../../data/thermo/sander-2023-hbr.dat");

pub fn temperature_covered(temperature_k: f64) -> bool {
    temperature_k.is_finite() && (temperature_k - REFERENCE_TEMPERATURE_K).abs() < 1e-8
}

/// Append before END so the native engine and parsed data index agree.
pub fn append_to(database: &[u8]) -> Vec<u8> {
    let at = crate::databases::find_last_end(database).unwrap_or(database.len());
    let mut bytes = Vec::with_capacity(database.len() + EXTENSION.len() + 1);
    bytes.extend_from_slice(&database[..at]);
    bytes.push(b'\n');
    bytes.extend_from_slice(EXTENSION);
    bytes.extend_from_slice(&database[at..]);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gas_slice_converts_dissociative_not_molecular_henry_constant() {
        let density: f64 = 997.0430117423601;
        let density_slope: f64 = -0.25659174059455836;
        let gas_constant = 8.31446261815324;
        let log_k = (8.2e9 * 101325.0 / density.powi(2)).log10();
        let dh = (-gas_constant * 10000.0
            - 2.0 * gas_constant * REFERENCE_TEMPERATURE_K.powi(2) * density_slope / density)
            / 1000.0;
        for database in [
            crate::databases::wateq4f(),
            crate::databases::minteq_v4(),
            crate::databases::pitzer(),
        ] {
            let index = crate::dbindex::DbIndex::parse(database);
            let phase = &index.phases["HBr(g)"];
            assert!((phase.log_k.unwrap() - log_k).abs() < 1e-12);
            assert!((phase.delta_h_kj.unwrap() - dh).abs() < 1e-10);
            assert!(index.has_element("Br"));
        }
        assert!(temperature_covered(298.15));
        assert!(!temperature_covered(300.0));
        assert!(!temperature_covered(f64::NAN));
    }
}

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
    }

    /// SO2(g) is deliberately NOT part of this slice (2026-11-19 review,
    /// docs/gas-phase-coverage-20261027.md "SO2(g)"). A Henry constant must
    /// dissolve into the route's own neutral S(IV) species, and no routed
    /// database has one: wateq4f, minteq.v4 and pitzer define sulfur only as
    /// S(-2)/S(6), and minteq.v4's reviewed `Sulfite` pseudo-element carries
    /// `Sulfite-2`/`H(Sulfite)-` but neither dissolved SO2 nor H2SO3. Posing
    /// SO2 through `EQUILIBRIUM_GASES` would also book its sulfur as bare
    /// `S`, which the redox-coupled routes would convert to sulfate. This
    /// pins the typed boundary until a coherent gas/neutral/bisulfite slice
    /// is reviewed; flipping any assertion here means that review happened.
    #[test]
    fn sulfur_dioxide_gas_is_refused_until_a_neutral_s_iv_slice_exists() {
        for (route, database) in [
            ("wateq4f", crate::databases::wateq4f()),
            ("minteq.v4", crate::databases::minteq_v4()),
            ("pitzer", crate::databases::pitzer()),
        ] {
            let index = crate::dbindex::DbIndex::parse(database);
            assert!(!index.has_phase("SO2(g)"), "{route} gained SO2(g)");
            assert!(!index.has_element("S(4)"), "{route} gained S(4)");
            let text = String::from_utf8_lossy(database);
            for token in text.split_whitespace() {
                assert!(
                    !matches!(token, "SO2" | "H2SO3" | "HSO3-" | "SO3-2" | "H2(Sulfite)"),
                    "{route} now defines the S(IV) species {token}"
                );
            }
        }
        let minteq = crate::dbindex::DbIndex::parse(crate::databases::minteq_v4());
        assert!(minteq.has_element("Sulfite"));
        assert!(!crate::derived::EQUILIBRIUM_GASES
            .iter()
            .any(|(phase, species)| *phase == "SO2(g)" || *species == "SO2"));
        assert!(!crate::derived::ATMOSPHERIC
            .iter()
            .any(|(phase, species, _)| *phase == "SO2(g)" || *species == "SO2"));
        assert!(!crate::gas_phases::headspace_coupled("SO2(g)"));
        let rows = crate::gas_phases::coverage();
        let so2: Vec<_> = rows.iter().filter(|row| row.gas == "SO2(g)").collect();
        assert_eq!(so2.len(), 3);
        assert!(so2
            .iter()
            .all(|row| row.status == crate::gas_phases::GasStatus::Refused));
    }

    #[test]
    fn every_routed_database_defines_the_same_ammonia_gas_phase() {
        let wateq = crate::dbindex::DbIndex::parse(crate::databases::wateq4f());
        let minteq = crate::dbindex::DbIndex::parse(crate::databases::minteq_v4());
        let reference = &wateq.phases["NH3(g)"];
        let routed = &minteq.phases["NH3(g)"];
        assert_eq!(routed.log_k, reference.log_k);
        assert!((routed.delta_h_kj.unwrap() - reference.delta_h_kj.unwrap()).abs() < 1e-12);
        assert!(minteq.has_phase("NH3(g)"));
        assert!(!temperature_covered(300.0));
        assert!(!temperature_covered(f64::NAN));
    }
}

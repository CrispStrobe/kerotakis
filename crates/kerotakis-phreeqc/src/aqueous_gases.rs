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

/// Reviewed neutral S(IV) + SO2(g) slice for minteq.v4's `Sulfite`
/// pseudo-element (Goldberg and Parker 1985, Table 9). DATA ONLY: it is
/// deliberately NOT appended to any route yet. Wiring it needs, in one
/// change: `derived::PROTONATION_SPLITS` mapping `H2(Sulfite)` to the
/// registry's aqueous `SO2`, an `EQUILIBRIUM_GASES`-style capability that
/// books the `Sulfite` element (not bare `S`) and removes the one water
/// per mole the gas formula carries, and a single owner decision between
/// the native solve and `kerotakis-core::sulfur_iv`. Appending the species
/// alone would let readback drop the neutral fraction of every acidified
/// sulfite vessel. See docs/gas-phase-coverage-20261027.md.
pub mod sulfur_dioxide {
    pub const SOURCE_ID: &str = "goldberg-parker-1985-so2";
    pub const EXTENSION: &[u8] =
        include_bytes!("../../../data/thermo/goldberg-parker-1985-so2.dat");
    /// Table 9 Gibbs energies, kJ/mol, 298.15 K, 1 bar.
    pub const DG_FIRST_DISSOCIATION_KJ: f64 = 10.60;
    pub const DG_SECOND_DISSOCIATION_KJ: f64 = 40.94;
    pub const DG_HENRY_KJ: f64 = -0.51;
    pub const DG_GAS_TO_BISULFITE_KJ: f64 = 10.09;
    pub const DH_FIRST_DISSOCIATION_KJ: f64 = -17.80;
    pub const DH_HENRY_KJ: f64 = -26.97;
    /// The registry route's existing second constant (llnl borrow).
    pub const ROUTED_LOG_K_BISULFITE: f64 = 7.2054;

    pub fn log_k(delta_g_kj: f64) -> f64 {
        let gas_constant = 8.31446261815324;
        -delta_g_kj * 1000.0
            / (gas_constant * super::REFERENCE_TEMPERATURE_K * std::f64::consts::LN_10)
    }

    /// Henry constant, mol kg-1 atm-1 (source is per bar).
    pub fn log_henry_atm() -> f64 {
        log_k(DG_HENRY_KJ) + 1.01325f64.log10()
    }

    /// The slice inserted before `END` of an already-extended minteq.v4.
    pub fn append_to(database: &[u8]) -> Vec<u8> {
        let at = crate::databases::find_last_end(database).unwrap_or(database.len());
        let mut bytes = Vec::with_capacity(database.len() + EXTENSION.len() + 1);
        bytes.extend_from_slice(&database[..at]);
        bytes.push(b'\n');
        bytes.extend_from_slice(EXTENSION);
        bytes.extend_from_slice(&database[at..]);
        bytes
    }
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

    #[test]
    fn sulfur_dioxide_slice_matches_goldberg_parker_table_9() {
        use super::sulfur_dioxide::*;
        let first = log_k(DG_FIRST_DISSOCIATION_KJ);
        assert!((first + 1.8570396554686752).abs() < 1e-12);
        assert!((log_henry_atm() - 0.09506474678061971).abs() < 1e-12);
        // Process (E) is (A) + (D) in the paper's own table.
        let gas_to_bisulfite = log_k(DG_GAS_TO_BISULFITE_KJ);
        assert!((first + log_k(DG_HENRY_KJ) - gas_to_bisulfite).abs() < 1e-12);
        // The routed second constant is consistent within Table 9's
        // +-0.20 kJ/mol (0.035 log units), so it is kept, not replaced.
        let second = -log_k(DG_SECOND_DISSOCIATION_KJ);
        assert!((second - 7.172377688196939).abs() < 1e-9);
        assert!((ROUTED_LOG_K_BISULFITE - second).abs() < log_k(-0.20));

        let extended = append_to(crate::databases::minteq_v4());
        let index = crate::dbindex::DbIndex::parse(&extended);
        let phase = &index.phases["SO2(g)"];
        assert!((phase.log_k.unwrap() - log_henry_atm()).abs() < 1e-12);
        assert!((phase.delta_h_kj.unwrap() - DH_HENRY_KJ).abs() < 1e-12);
        assert!((index.species_delta_h_kj["H2(Sulfite)"] + DH_FIRST_DISSOCIATION_KJ).abs() < 1e-12);
        assert!(index.has_element("Sulfite"));
    }

    /// The reviewed SO2 slice ([`sulfur_dioxide`]) exists as data but no
    /// route carries it yet: wateq4f and pitzer have no S(IV) at all (typed
    /// refusal, permanently until a sulfur-redox-isolated S(IV) exists
    /// there), and minteq.v4 waits for the readback/booking/ownership
    /// wiring. Posing SO2 through `EQUILIBRIUM_GASES` today would book bare
    /// `S`, which the redox-coupled routes turn into sulfate. Wiring the
    /// slice must flip the minteq.v4 assertions here deliberately.
    #[test]
    fn sulfur_dioxide_slice_is_not_routed_until_wired() {
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

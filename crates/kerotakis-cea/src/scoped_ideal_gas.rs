//! Provenance-preserving view of the six fluids in the BRD-031 scope.
//!
//! This module deliberately does not copy coefficients. It selects gas-phase
//! [`Species`](crate::nasa9::Species) records from the already embedded CEA
//! database, avoiding a second table that could drift. These are ideal-gas
//! NASA-9 thermochemistry records, **not** PC-SAFT/residual-fluid parameters.

use crate::nasa9::{db, Species};

/// Original report documenting the NASA-9 form and Appendix D records.
pub const REPORT_TITLE: &str =
    "NASA Glenn Coefficients for Calculating Thermodynamic Properties of Individual Species";
pub const REPORT_REVISION: &str = "NASA/TP-2002-211556, September 2002";
pub const REPORT_NTRS_ID: &str = "20020085330";
pub const REPORT_URL: &str = "https://ntrs.nasa.gov/citations/20020085330";
pub const REPORT_PDF_SHA256: &str =
    "0ab4154b0fdeac29581f8d047a4cb6836c138c0b41da8f28990ef7d3e2756765";

/// Exact vendored input used at runtime. NASA CEA distributes this revised
/// database under Apache-2.0; see `vendor/nasa-cea/LICENSE.txt` and
/// `vendor/nasa-cea/NOTICE.txt`. Its header records NASA GRC updates dated
/// 2021-08-16 and 2021-09-09, including corrected low-temperature fit ranges.
pub const RUNTIME_DATABASE_PATH: &str = "vendor/nasa-cea/thermo.inp";
pub const RUNTIME_DATABASE_SHA256: &str =
    "fa7746572952d74e249e818a82a35c113829742fb421a308e167185528884363";
pub const RUNTIME_DATABASE_LICENSE: &str = "Apache-2.0 (NASA CEA LICENSE.txt and NOTICE.txt)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportLocator {
    pub formula: &'static str,
    /// Printed Appendix D page, rather than PDF viewer page index.
    pub report_page: u16,
    pub record_reference: &'static str,
}

pub const REPORT_LOCATORS: &[ReportLocator] = &[
    ReportLocator {
        formula: "CO2",
        report_page: 85,
        record_reference: "Gurvich,1991 pt1 p27 pt2 p24.",
    },
    ReportLocator {
        formula: "C2H5OH",
        report_page: 92,
        record_reference: "Hf:TRC(6/87) w5030. Chao,1986.",
    },
    ReportLocator {
        formula: "H2O",
        report_page: 131,
        record_reference: "Hf:Cox,1989. Woolley,1987. TRC(10/88) tuv25.",
    },
    ReportLocator {
        formula: "NH3",
        report_page: 154,
        record_reference: "Gurvich,1989 pt1 p354 pt2 p219. Haar,1968.",
    },
    ReportLocator {
        formula: "N2",
        report_page: 156,
        record_reference: "Ref-Elm. Gurvich,1978 pt1 p280 pt2 p207.",
    },
    ReportLocator {
        formula: "O2",
        report_page: 166,
        record_reference: "Ref-Elm. Gurvich,1989 pt1 p94 pt2 p9.",
    },
];

#[derive(Debug, Clone, Copy)]
pub struct ScopedIdealGas {
    pub locator: &'static ReportLocator,
    pub thermo: &'static Species,
}

/// Select one of the six scoped ideal-gas records from the canonical parser.
pub fn scoped_ideal_gas(formula: &str) -> Option<ScopedIdealGas> {
    let locator = REPORT_LOCATORS
        .iter()
        .find(|locator| locator.formula == formula)?;
    let thermo = db().species.get(formula)?;
    thermo
        .is_gas()
        .then_some(ScopedIdealGas { locator, thermo })
}

/// Iterate over all six records in stable scope order.
pub fn scoped_ideal_gases() -> impl Iterator<Item = ScopedIdealGas> {
    REPORT_LOCATORS
        .iter()
        .filter_map(|locator| scoped_ideal_gas(locator.formula))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_locators_resolve_to_canonical_gas_records() {
        let records: Vec<_> = scoped_ideal_gases().collect();
        assert_eq!(records.len(), REPORT_LOCATORS.len());
        assert_eq!(
            records
                .iter()
                .map(|record| (record.locator.formula, record.locator.report_page))
                .collect::<Vec<_>>(),
            vec![
                ("CO2", 85),
                ("C2H5OH", 92),
                ("H2O", 131),
                ("NH3", 154),
                ("N2", 156),
                ("O2", 166),
            ]
        );
        assert!(records.iter().all(|record| record.thermo.is_gas()));
    }

    #[test]
    fn runtime_records_retain_report_level_references() {
        for record in scoped_ideal_gases() {
            assert_eq!(record.thermo.reference, record.locator.record_reference);
            assert!(!record.thermo.intervals.is_empty());
            assert!(record.thermo.intervals.iter().all(|interval| interval
                .coeffs
                .iter()
                .all(|coefficient| coefficient.is_finite())));
        }
    }

    #[test]
    fn revised_runtime_range_is_not_misrepresented_as_the_2002_pdf() {
        // Appendix D p. 92 prints 200 K for ethanol. The runtime file's
        // 2021-09-09 change log says ranges arbitrarily set to 200 K were reset
        // to 300 K where the underlying fit only starts there.
        assert_eq!(
            scoped_ideal_gas("C2H5OH").unwrap().thermo.t_range(),
            Some((300.0, 6000.0))
        );
        assert_ne!(REPORT_PDF_SHA256, RUNTIME_DATABASE_SHA256);
    }

    #[test]
    fn representative_values_are_read_from_the_canonical_table() {
        let co2 = scoped_ideal_gas("CO2").unwrap().thermo;
        assert_eq!(co2.h_formation, -393_510.0);
        assert_eq!(co2.intervals[0].coeffs[0], 4.943650540e4);

        let oxygen = scoped_ideal_gas("O2").unwrap().thermo;
        assert_eq!(oxygen.intervals.len(), 3);
        assert_eq!(oxygen.intervals[2].coeffs[8], -5.530621610e2);
    }

    #[test]
    fn out_of_scope_species_are_refused() {
        assert!(scoped_ideal_gas("CH4").is_none());
        assert!(scoped_ideal_gas("H2O(L)").is_none());
    }
}

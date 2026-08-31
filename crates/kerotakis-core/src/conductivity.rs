//! INST-004 / CAP-22: specific conductance from the solved speciation.
//!
//! The meter used to report `ionic_strength × 100 000` with a comment
//! admitting it was rough. This module replaces that with Kohlrausch's law
//! of independent migration: at infinite dilution every ion conducts on
//! its own, so the specific conductance is the sum of each ion's limiting
//! molar conductivity times its concentration,
//!
//! ```text
//! κ = Σᵢ λ°ᵢ · cᵢ
//! ```
//!
//! with λ°ᵢ in S·cm²·mol⁻¹ and cᵢ in mol·L⁻¹ (κ then lands in µS·cm⁻¹
//! after the factor 1000). The λ° table is measured data, not theory.
//!
//! Honesty boundary, stated rather than hidden:
//!
//! * **Dilute solutions only.** Independent migration is the c → 0 limit;
//!   ion–ion drag (relaxation and electrophoresis) lowers real
//!   conductivities below the sum as concentration grows. At the 0.01 M
//!   KCl calibration standard the sum overestimates by ~6%; past
//!   I ≈ 0.1 mol/kgw the estimate is no longer trustworthy and the
//!   result says so (`within_dilute_limit = false`, and the meter
//!   reports the reading as out of its calibrated range).
//! * **Coverage is accounted, not assumed.** Charged species with no
//!   tabulated λ° are left out of the sum and reported by name, with the
//!   fraction of the total charge the sum did cover. Neutral aqueous
//!   complexes (AgCl°, CO₂(aq)) carry no current and are rightly absent.
//! * **No speciation, no Kohlrausch.** When a solver reported only pH and
//!   ionic strength, the old scaling survives as an explicitly labelled
//!   mean-mobility estimate (I × 10⁵ µS/cm ≈ a 1:1 electrolyte of
//!   ordinary mobility) instead of masquerading as the real model.

use crate::vessel::{SolutionInfo, SpeciesDetail};

/// Where every λ° in [`LIMITING_CONDUCTIVITY`] comes from.
pub const LAMBDA_SOURCE: &str = "λ°: CRC Handbook of Chemistry and Physics, \
    'Ionic Conductivity and Diffusion at Infinite Dilution' (Vanýsek), \
    25 °C, water";

/// Limiting molar ionic conductivities λ° at 25 °C in S·cm²·mol⁻¹, keyed
/// by PHREEQC species name (charge written as trailing `+2` / `-2`).
/// Values for multivalent ions are for the ion itself, not the
/// equivalent — Ca⁺² is ~119, i.e. 2 × λ°(½Ca⁺²).
pub const LIMITING_CONDUCTIVITY: &[(&str, f64)] = &[
    ("H+", 349.65),
    ("OH-", 198.0),
    ("Li+", 38.66),
    ("Na+", 50.08),
    ("K+", 73.48),
    ("NH4+", 73.5),
    ("Ag+", 61.9),
    ("Ca+2", 118.94),
    ("Mg+2", 106.0),
    ("Ba+2", 127.2),
    ("Sr+2", 118.9),
    ("Cu+2", 107.2),
    ("Zn+2", 105.6),
    ("Fe+2", 108.0),
    ("Fe+3", 204.0),
    ("Al+3", 183.0),
    ("Mn+2", 107.0),
    ("Pb+2", 142.0),
    ("Cl-", 76.31),
    ("Br-", 78.1),
    ("I-", 76.8),
    ("F-", 55.4),
    ("NO3-", 71.42),
    ("ClO4-", 67.3),
    ("HCO3-", 44.5),
    ("CO3-2", 138.6),
    ("SO4-2", 160.0),
    ("MnO4-", 61.3),
];

/// Ionic strength above which independent migration is no longer an
/// honest model (mol/kgw). Chosen where the neglected ion–ion drag grows
/// from percent-level to tens of percent.
pub const DILUTE_LIMIT_MOLAL: f64 = 0.1;

/// What backed the number — the two paths must stay distinguishable all
/// the way to the reader.
#[derive(Debug, Clone, PartialEq)]
pub enum Basis {
    /// Kohlrausch sum over the solved speciation.
    Kohlrausch {
        /// Fraction of the total ionic charge concentration the λ° table
        /// covered (1.0 = every charged species contributed).
        covered_charge_fraction: f64,
        /// Charged species that were left out for lack of a tabulated λ°.
        omitted: Vec<String>,
    },
    /// `I × 10⁵` — the labelled fallback when no speciation was reported.
    MeanMobility,
}

/// A specific-conductance estimate that knows what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub microsiemens_per_cm: f64,
    pub basis: Basis,
    /// False above [`DILUTE_LIMIT_MOLAL`]: the number is then an
    /// extrapolation past the model's validity, not a measurement.
    pub within_dilute_limit: bool,
}

impl Estimate {
    /// Whether the meter may present this as an in-calibration reading:
    /// dilute, and (on the Kohlrausch path) with at least 90% of the
    /// ionic charge actually covered by the table.
    pub fn trustworthy(&self) -> bool {
        self.within_dilute_limit
            && match &self.basis {
                Basis::Kohlrausch {
                    covered_charge_fraction,
                    ..
                } => *covered_charge_fraction >= 0.9,
                Basis::MeanMobility => false,
            }
    }
}

/// The ionic charge encoded in a PHREEQC species name: `Na+` → +1,
/// `Ca+2` → +2, `CO3-2` → −2, `AgCl` → 0. PHREEQC writes the charge as a
/// trailing run of `+`/`-` signs, optionally followed by the count.
pub fn ion_charge(name: &str) -> i32 {
    let core = name.trim();
    let Some(pos) = core.find(['+', '-']) else {
        return 0;
    };
    // Everything from the first sign on is the charge suffix, e.g. "+2",
    // "-", "++" (older dialects double the sign instead of counting).
    let suffix = &core[pos..];
    let sign = if suffix.starts_with('+') { 1 } else { -1 };
    let signs = suffix
        .chars()
        .take_while(|c| *c == '+' || *c == '-')
        .count();
    let digits: String = suffix
        .chars()
        .skip(signs)
        .take_while(char::is_ascii_digit)
        .collect();
    match digits.parse::<i32>() {
        Ok(n) => sign * n,
        Err(_) => sign * signs as i32,
    }
}

/// Specific conductance of a solved solution, honestly labelled.
pub fn specific_conductance(info: &SolutionInfo) -> Estimate {
    let within_dilute_limit = info.ionic_strength <= DILUTE_LIMIT_MOLAL;
    if info.species.is_empty() {
        return Estimate {
            microsiemens_per_cm: info.ionic_strength * 100_000.0,
            basis: Basis::MeanMobility,
            within_dilute_limit,
        };
    }
    let lambda = |name: &str| -> Option<f64> {
        LIMITING_CONDUCTIVITY
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, l)| *l)
    };
    let mut kappa_us_cm = 0.0;
    let mut charge_covered = 0.0;
    let mut charge_total = 0.0;
    let mut omitted = Vec::new();
    for SpeciesDetail { name, molality, .. } in &info.species {
        let z = ion_charge(name);
        if z == 0 {
            continue; // neutral complexes carry no current
        }
        // Dilute aqueous: mol/kgw ≈ mol/L, the same approximation the
        // dilute-limit boundary already commits this model to.
        let c_mol_l = *molality;
        charge_total += f64::from(z.abs()) * c_mol_l;
        match lambda(name) {
            Some(l) => {
                kappa_us_cm += l * c_mol_l * 1000.0;
                charge_covered += f64::from(z.abs()) * c_mol_l;
            }
            None => omitted.push(name.clone()),
        }
    }
    let covered_charge_fraction = if charge_total > 0.0 {
        charge_covered / charge_total
    } else {
        1.0 // nothing charged in solution: κ = 0 covers everything there is
    };
    Estimate {
        microsiemens_per_cm: kappa_us_cm,
        basis: Basis::Kohlrausch {
            covered_charge_fraction,
            omitted,
        },
        within_dilute_limit,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ion(name: &str, molality: f64) -> SpeciesDetail {
        SpeciesDetail {
            name: name.to_string(),
            molality,
            activity: molality,
        }
    }

    fn solved(ionic_strength: f64, species: Vec<SpeciesDetail>) -> SolutionInfo {
        SolutionInfo {
            pe: None,
            redox: Vec::new(),
            ph: 7.0,
            ionic_strength,
            species,
            provenance: None,
        }
    }

    #[test]
    fn phreeqc_charge_suffixes_parse() {
        assert_eq!(ion_charge("Na+"), 1);
        assert_eq!(ion_charge("Cl-"), -1);
        assert_eq!(ion_charge("Ca+2"), 2);
        assert_eq!(ion_charge("CO3-2"), -2);
        assert_eq!(ion_charge("Fe+3"), 3);
        assert_eq!(ion_charge("AgCl"), 0);
        assert_eq!(ion_charge("CO2"), 0);
        // Older dialects double the sign instead of counting it.
        assert_eq!(ion_charge("Ca++"), 2);
        assert_eq!(ion_charge("S--"), -2);
    }

    /// The 0.01 mol/kg KCl calibration standard reads 1413 µS/cm. The
    /// infinite-dilution sum must land close — and must land HIGH,
    /// because the neglected ion–ion drag only ever lowers the real
    /// value. A result below the standard would mean the model or the
    /// data is wrong, not that the approximation improved.
    #[test]
    fn kcl_calibration_standard_within_model_error() {
        let info = solved(0.01, vec![ion("K+", 0.01), ion("Cl-", 0.01)]);
        let est = specific_conductance(&info);
        let kappa = est.microsiemens_per_cm;
        assert!(kappa > 1413.0, "must overestimate the standard: {kappa}");
        assert!(
            (kappa - 1413.0) / 1413.0 < 0.07,
            "within 7% of the 1413 µS/cm standard: {kappa}"
        );
        assert!(est.trustworthy());
        match est.basis {
            Basis::Kohlrausch {
                covered_charge_fraction,
                ref omitted,
            } => {
                assert_eq!(covered_charge_fraction, 1.0);
                assert!(omitted.is_empty());
            }
            Basis::MeanMobility => panic!("speciation was present"),
        }
    }

    /// Grotthuss hopping makes protons the fastest ion in water: at equal
    /// concentration HCl must out-conduct KCl, and KCl out-conduct NaCl
    /// (K⁺ outruns the more strongly hydrated Na⁺). The old
    /// ionic-strength scaling could not see any of this.
    #[test]
    fn mobility_ordering_is_visible() {
        let at = |cation: &str| {
            specific_conductance(&solved(0.01, vec![ion(cation, 0.01), ion("Cl-", 0.01)]))
                .microsiemens_per_cm
        };
        let (hcl, kcl, nacl) = (at("H+"), at("K+"), at("Na+"));
        assert!(
            hcl > 2.0 * kcl,
            "protons conduct several-fold: {hcl} vs {kcl}"
        );
        assert!(kcl > nacl, "K+ outruns Na+: {kcl} vs {nacl}");
    }

    #[test]
    fn neutral_complexes_carry_no_current() {
        let bare = solved(0.001, vec![ion("Ag+", 0.001), ion("NO3-", 0.001)]);
        let with_complex = solved(
            0.001,
            vec![
                ion("Ag+", 0.001),
                ion("NO3-", 0.001),
                ion("AgCl", 0.005), // neutral ion pair, however abundant
            ],
        );
        let a = specific_conductance(&bare);
        let b = specific_conductance(&with_complex);
        assert_eq!(a.microsiemens_per_cm, b.microsiemens_per_cm);
        assert!(b.trustworthy(), "a neutral complex is not 'uncovered'");
    }

    #[test]
    fn untabulated_ions_are_confessed_not_ignored() {
        let info = solved(
            0.02,
            vec![
                ion("Na+", 0.01),
                ion("Cl-", 0.01),
                ion("W12O41-10", 0.002), // exotic: nothing in the table
            ],
        );
        let est = specific_conductance(&info);
        match est.basis {
            Basis::Kohlrausch {
                covered_charge_fraction,
                ref omitted,
            } => {
                assert!(covered_charge_fraction < 0.9);
                assert_eq!(omitted, &vec!["W12O41-10".to_string()]);
            }
            Basis::MeanMobility => panic!("speciation was present"),
        }
        assert!(!est.trustworthy(), "a fifth of the charge is unaccounted");
    }

    #[test]
    fn concentrated_solutions_leave_the_validity_window() {
        let info = solved(1.0, vec![ion("Na+", 1.0), ion("Cl-", 1.0)]);
        let est = specific_conductance(&info);
        assert!(!est.within_dilute_limit);
        assert!(!est.trustworthy());
    }

    #[test]
    fn no_speciation_falls_back_to_labelled_mean_mobility() {
        let info = solved(0.01, Vec::new());
        let est = specific_conductance(&info);
        assert_eq!(est.basis, Basis::MeanMobility);
        assert_eq!(est.microsiemens_per_cm, 1000.0);
        assert!(!est.trustworthy(), "a guess never presents as calibrated");
    }
}

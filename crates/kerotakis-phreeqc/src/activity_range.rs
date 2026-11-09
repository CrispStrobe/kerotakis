//! The Pitzer route's stated range, per binary electrolyte.
//!
//! `pitzer.dat` is routed for concentrated brines because the
//! ion-interaction model is the one built for them, but its parameters
//! are FITTED, electrolyte by electrolyte, to a stated molality: HCl to
//! about 6 mol/kg (Pitzer & Mayorga 1973). Past that the virial series is
//! an extrapolation. A 37 % hydrochloric acid (16 mol/kg) came out at
//! pH -5.4 where the measured mean activity coefficient implies about
//! -2.8. A model evaluated outside its stated domain refuses instead of
//! answering (docs/model-domain-refusal-coverage-20261027.md), so this
//! module states which electrolyte is outside its range and the solver
//! withdraws the activity-derived outputs (pH, activities, saturation
//! indices). The inventory is not touched.
//!
//! # Composition rule
//!
//! From the solved DISSOLVED element totals (mol/kgw, PHREEQC `-totals`):
//!
//! 1. Every strong ion in the table's `ions` map gets its molality; S is
//!    counted as sulfate.
//! 2. The strong charge imbalance `sum(z_anion m) - sum(z_cation m)` is
//!    carried by H+ (positive) or OH- (negative), after subtracting the
//!    most the weak acid systems present could absorb (`weak_acid_capacity`
//!    equivalents per mol of C, B, Si). A buffered solution therefore never
//!    reads as an excess strong acid or base.
//! 3. A binary electrolyte's stoichiometric molality is the most of it the
//!    ions could make: `min(m_cation / nu_cation, m_anion / nu_anion)`.
//!    In a mixture this is an upper bound per pair, so the rule is
//!    conservative in the direction of refusing.
//! 4. Any row with a stated `max_molality` exceeded refuses; the row with
//!    the largest ratio is named. Rows without a stated range are not
//!    checked and not claimed.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use kerotakis_core::phrase::{Phrase, Slot};
use serde::Deserialize;

/// The checked-in, cited table (see its `sources`).
pub const TABLE_JSON: &str = include_str!("../data/pitzer-electrolyte-ranges.json");

#[derive(Debug, Clone, Deserialize)]
pub struct Electrolyte {
    pub electrolyte: String,
    pub cation: String,
    pub anion: String,
    pub nu_cation: f64,
    pub nu_anion: f64,
    pub max_molality: Option<f64>,
    pub source: String,
    /// Cited validity to saturation over a temperature range (see the
    /// table's `sources.saturation_scope`).
    #[serde(default)]
    pub saturation_scope: Option<SaturationScope>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaturationScope {
    /// The PHREEQC phase whose saturation index bounds the scope.
    pub phase: String,
    pub t_min_c: f64,
    pub t_max_c: f64,
    pub source: String,
}

/// Saturation index tolerance for "at or below saturation".
pub const SATURATION_SI_TOLERANCE: f64 = 0.01;

/// What the same solve says about the conditions a saturation scope needs.
#[derive(Debug, Clone, Copy)]
pub struct Conditions<'a> {
    pub temperature_c: f64,
    /// (phase, saturation index) from the same solve.
    pub saturation: &'a [(String, f64)],
}

impl Conditions<'_> {
    /// 25 C with no saturation information: only the 25 C molality applies.
    pub const STANDARD: Conditions<'static> = Conditions {
        temperature_c: 25.0,
        saturation: &[],
    };
}

impl Electrolyte {
    /// Is `m` inside this row's stated range under `conditions`?
    pub fn admits(&self, m: f64, conditions: Conditions<'_>) -> bool {
        let Some(max) = self.max_molality else {
            return true;
        };
        if m <= max {
            return true;
        }
        let Some(scope) = &self.saturation_scope else {
            return false;
        };
        let t = conditions.temperature_c;
        if !(scope.t_min_c..=scope.t_max_c).contains(&t) {
            return false;
        }
        conditions
            .saturation
            .iter()
            .find(|(phase, _)| *phase == scope.phase)
            .is_some_and(|(_, si)| si.is_finite() && *si <= SATURATION_SI_TOLERANCE)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Table {
    pub sources: BTreeMap<String, String>,
    pub ions: BTreeMap<String, f64>,
    pub weak_acid_capacity: BTreeMap<String, f64>,
    pub electrolytes: Vec<Electrolyte>,
}

pub fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        serde_json::from_str(TABLE_JSON).expect("pitzer-electrolyte-ranges.json parses")
    })
}

/// One electrolyte past its stated range.
#[derive(Debug, Clone, PartialEq)]
pub struct Breach {
    pub electrolyte: String,
    pub molality: f64,
    pub max_molality: f64,
    pub source: String,
}

impl Breach {
    /// The typed refusal the solver records and the pH meter reports.
    pub fn phrase(&self) -> Phrase {
        Phrase::new(
            "not-modeled.activity-model-outside-range",
            "the activity model's stated range: the Pitzer parameters for {electrolyte} are fitted up to about {range} mol/kg water, and this solution holds {molality} mol/kg of it, so pH, activities and saturation indices are not reported (the inventory is unchanged)",
            vec![
                (
                    "electrolyte".to_string(),
                    Slot::text(self.electrolyte.clone()),
                ),
                (
                    "range".to_string(),
                    Slot::number(format!("{}", self.max_molality)),
                ),
                (
                    "molality".to_string(),
                    Slot::number(format!("{:.1}", self.molality)),
                ),
            ],
        )
    }
}

/// Stoichiometric molality of every table electrolyte, from dissolved
/// element totals keyed by base element (mol/kgw). Rule in the module docs.
pub fn electrolyte_molalities(totals: &BTreeMap<String, f64>) -> Vec<(&'static Electrolyte, f64)> {
    let table = table();
    let mut ions: BTreeMap<&str, f64> = BTreeMap::new();
    let mut imbalance = 0.0;
    for (ion, z) in &table.ions {
        if ion == "H" || ion == "OH" {
            continue;
        }
        let m = totals.get(ion.as_str()).copied().unwrap_or(0.0).max(0.0);
        ions.insert(ion.as_str(), m);
        imbalance -= z * m;
    }
    let weak: f64 = table
        .weak_acid_capacity
        .iter()
        .map(|(el, eq)| eq * totals.get(el.as_str()).copied().unwrap_or(0.0).max(0.0))
        .sum();
    let strong = (imbalance.abs() - weak).max(0.0);
    ions.insert("H", if imbalance > 0.0 { strong } else { 0.0 });
    ions.insert("OH", if imbalance < 0.0 { strong } else { 0.0 });
    table
        .electrolytes
        .iter()
        .map(|row| {
            let c = ions.get(row.cation.as_str()).copied().unwrap_or(0.0) / row.nu_cation;
            let a = ions.get(row.anion.as_str()).copied().unwrap_or(0.0) / row.nu_anion;
            (row, c.min(a))
        })
        .collect()
}

/// The electrolyte furthest outside its stated range, if any is.
pub fn breach_of(totals: &BTreeMap<String, f64>, conditions: Conditions<'_>) -> Option<Breach> {
    electrolyte_molalities(totals)
        .into_iter()
        .filter_map(|(row, m)| {
            let max = row.max_molality?;
            (!row.admits(m, conditions)).then_some((row, m, m / max))
        })
        .max_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(row, m, _)| Breach {
            electrolyte: row.electrolyte.clone(),
            molality: m,
            max_molality: row.max_molality.expect("filtered"),
            source: row.source.clone(),
        })
}

/// Read the solved dissolved totals for `elements` through the solver's
/// selected-output reader and check them. A valence-tagged column is used
/// only when its base element has no untagged column of its own.
pub fn breach(
    elements: &[String],
    value: &dyn Fn(&str) -> Option<f64>,
    conditions: Conditions<'_>,
) -> Option<Breach> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    let untagged: Vec<&str> = elements
        .iter()
        .filter(|e| !e.contains('('))
        .map(String::as_str)
        .collect();
    for element in elements {
        let base = element.split('(').next().unwrap_or(element);
        if element.contains('(') && untagged.contains(&base) {
            continue;
        }
        if let Some(m) = value(element).filter(|m| m.is_finite()) {
            *totals.entry(base.to_string()).or_insert(0.0) += m;
        }
    }
    breach_of(&totals, conditions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn totals(pairs: &[(&str, f64)]) -> BTreeMap<String, f64> {
        pairs.iter().map(|(e, m)| (e.to_string(), *m)).collect()
    }

    #[test]
    fn every_row_cites_a_listed_source_and_known_ions() {
        let t = table();
        for row in &t.electrolytes {
            assert!(t.sources.contains_key(&row.source), "{}", row.electrolyte);
            if let Some(scope) = &row.saturation_scope {
                assert!(t.sources.contains_key(&scope.source), "{}", row.electrolyte);
                assert!(scope.t_min_c < scope.t_max_c, "{}", row.electrolyte);
            }
            assert!(t.ions.contains_key(&row.cation), "{}", row.electrolyte);
            assert!(t.ions.contains_key(&row.anion), "{}", row.electrolyte);
            assert_eq!(
                row.max_molality.is_none(),
                row.source == "none",
                "{}",
                row.electrolyte
            );
            let zc = t.ions[&row.cation];
            let za = t.ions[&row.anion];
            assert!(
                (zc * row.nu_cation + za * row.nu_anion).abs() < 1e-12,
                "{} is not neutral",
                row.electrolyte
            );
        }
    }

    #[test]
    fn concentrated_hydrochloric_acid_is_outside_the_hcl_range() {
        let b = breach_of(&totals(&[("Cl", 16.1)]), Conditions::STANDARD)
            .expect("16 m HCl is outside 6 m");
        assert_eq!(b.electrolyte, "HCl");
        assert!((b.molality - 16.1).abs() < 1e-9);
        assert_eq!(b.max_molality, 6.0);
    }

    #[test]
    fn one_molal_hcl_and_saturated_brine_are_inside() {
        assert_eq!(
            breach_of(&totals(&[("Cl", 1.0)]), Conditions::STANDARD),
            None
        );
        assert_eq!(
            breach_of(&totals(&[("Na", 6.15), ("Cl", 6.15)]), Conditions::STANDARD),
            None
        );
    }

    #[test]
    fn ten_molal_sodium_hydroxide_is_outside_its_range() {
        let b = breach_of(&totals(&[("Na", 10.0)]), Conditions::STANDARD).expect("10 m NaOH > 6 m");
        assert_eq!(b.electrolyte, "NaOH");
        assert_eq!(b.max_molality, 6.0);
        assert_eq!(
            breach_of(&totals(&[("Na", 5.0)]), Conditions::STANDARD),
            None
        );
    }

    /// LiOH, CaBr2 and Ba(OH)2 carry exactly the Pitzer-Mayorga 1973
    /// parameters in the shipped pitzer.dat, so they inherit its max m
    /// (4, 2, 0.1 mol/kg) and are checked like every other ranged row.
    #[test]
    fn rows_ranged_at_the_2026_11_09_check_refuse_past_their_fit() {
        let b = breach_of(&totals(&[("Li", 5.0)]), Conditions::STANDARD).expect("5 m LiOH > 4 m");
        assert_eq!((b.electrolyte.as_str(), b.max_molality), ("LiOH", 4.0));
        assert_eq!(
            breach_of(&totals(&[("Li", 3.5)]), Conditions::STANDARD),
            None
        );
        let b = breach_of(&totals(&[("Ca", 2.5), ("Br", 5.0)]), Conditions::STANDARD)
            .expect("2.5 m CaBr2 > 2 m");
        assert_eq!((b.electrolyte.as_str(), b.max_molality), ("CaBr2", 2.0));
        assert_eq!(
            breach_of(&totals(&[("Ca", 1.5), ("Br", 3.0)]), Conditions::STANDARD),
            None
        );
        let b = breach_of(&totals(&[("Ba", 0.2)]), Conditions::STANDARD)
            .expect("0.2 m Ba(OH)2 > 0.1 m");
        assert_eq!((b.electrolyte.as_str(), b.max_molality), ("Ba(OH)2", 0.1));
        assert_eq!(
            breach_of(&totals(&[("Ba", 0.05)]), Conditions::STANDARD),
            None
        );
    }

    #[test]
    fn hot_saturated_brine_is_inside_its_saturation_scope() {
        let hot = |t: f64, si: f64| (t, vec![("Halite".to_string(), si)]);
        let brine = totals(&[("Na", 7.0), ("Cl", 7.0)]);
        let (t, sat) = hot(100.0, 0.0);
        let at = Conditions {
            temperature_c: t,
            saturation: &sat,
        };
        assert_eq!(breach_of(&brine, at), None, "saturated at 100 C");
        let (t, sat) = hot(100.0, 0.2);
        let over = Conditions {
            temperature_c: t,
            saturation: &sat,
        };
        assert_eq!(breach_of(&brine, over).unwrap().electrolyte, "NaCl");
        let (t, sat) = hot(320.0, 0.0);
        let beyond = Conditions {
            temperature_c: t,
            saturation: &sat,
        };
        assert_eq!(breach_of(&brine, beyond).unwrap().electrolyte, "NaCl");
        // No saturation index from the solve: not claimed.
        assert!(breach_of(
            &brine,
            Conditions {
                temperature_c: 100.0,
                saturation: &[]
            }
        )
        .is_some());
        // KCl's scope stops at 250 C.
        let kcl = totals(&[("K", 7.0), ("Cl", 7.0)]);
        let sat = vec![("Sylvite".to_string(), 0.0)];
        assert!(breach_of(
            &kcl,
            Conditions {
                temperature_c: 100.0,
                saturation: &sat
            }
        )
        .is_none());
        assert!(breach_of(
            &kcl,
            Conditions {
                temperature_c: 260.0,
                saturation: &sat
            }
        )
        .is_some());
        // HCl has no saturation scope: hot does not help.
        let acid = totals(&[("Cl", 16.0)]);
        assert!(breach_of(
            &acid,
            Conditions {
                temperature_c: 100.0,
                saturation: &sat
            }
        )
        .is_some());
    }

    #[test]
    fn weak_acid_buffering_is_not_read_as_strong_base() {
        // 3 m Na2CO3: Na 6 m unbalanced by strong anions, but carbonate can
        // absorb 6 equivalents; no NaOH is claimed.
        assert_eq!(
            breach_of(&totals(&[("Na", 6.0), ("C", 3.0)]), Conditions::STANDARD),
            None
        );
    }

    #[test]
    fn the_refusal_names_the_electrolyte_and_both_molalities() {
        let b = breach_of(&totals(&[("Cl", 16.1)]), Conditions::STANDARD).unwrap();
        let en = b.phrase().render(kerotakis_core::Locale::EN);
        assert!(
            en.contains("HCl") && en.contains("16.1") && en.contains("6"),
            "{en}"
        );
        for lang in ["de", "fr"] {
            let other = b.phrase().render(kerotakis_core::Locale::parse(lang));
            assert_ne!(other, en, "{lang} must be translated");
            assert!(
                other.contains("HCl") && other.contains("16,1"),
                "{lang}: {other}"
            );
        }
    }
}

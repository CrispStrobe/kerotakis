//! The flame test, read from the cation (AN-001/AN-002, 2026-11-10).
//!
//! A salt held in a flame does not burn; its metal cation is excited and
//! emits at its own wavelengths. The colour therefore follows the CATION,
//! not the whole salt name: NaCl and NaBr give the same sodium yellow.
//! Before this module the colour was a per-species curated field, so NaBr
//! (no curated colour) was sent through combustion and came back with a
//! solver failure and "no fuel", and a NaCl+KCl mixture reported only the
//! first salt it met.
//!
//! [`EMISSIONS`] is the one table. Atomic lines are NIST Atomic Spectra
//! Database values (Kramida, Ralchenko, Reader and NIST ASD Team, NIST ASD
//! v5.11, https://physics.nist.gov/asd, air wavelengths). For Ca, Sr, Ba
//! and Cu the visible colour of a flame is dominated by molecular band
//! emission (CaOH, SrOH, BaOH, CuCl/CuOH), identified in Pearse and Gaydon,
//! *The Identification of Molecular Spectra*, 4th ed., Chapman & Hall,
//! 1976; the band note names the emitter instead of inventing band heads.
//! A cation present in the sample without a row is refused with a typed,
//! translated not-modelled reason rather than painted with a guess.
//! `docs/flame-emission-coverage-20261110.md` lists the table and
//! `tools/flame_emission_coverage.py --check` holds the doc to it.

use crate::ops::{Event, NotModelledCause};
use crate::phrase::{Phrase, Slot};
use crate::species::{self, SpeciesId};
use crate::vessel::{Portion, VesselId};

/// One cation's flame emission.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CationEmission {
    /// Element symbol of the cation.
    pub element: &'static str,
    /// Colour word; a key of the `[flame]` locale section.
    pub colour: &'static str,
    /// Strongest atomic emission lines, nm (NIST ASD, air).
    pub lines_nm: &'static [f64],
    /// Molecular emitter that carries the visible colour, where it is not
    /// the atomic line itself (Pearse and Gaydon 1976).
    pub band_emitter: Option<&'static str>,
    /// Whether sodium's yellow D lines visually mask this colour.
    pub masked_by_sodium: bool,
}

/// Every cation whose flame emission the lab carries.
pub const EMISSIONS: &[CationEmission] = &[
    CationEmission {
        element: "Li",
        colour: "crimson red",
        lines_nm: &[670.776, 670.791, 610.354],
        band_emitter: None,
        masked_by_sodium: false,
    },
    CationEmission {
        element: "Na",
        colour: "bright yellow",
        lines_nm: &[588.995, 589.592],
        band_emitter: None,
        masked_by_sodium: false,
    },
    CationEmission {
        element: "K",
        colour: "lilac",
        lines_nm: &[766.490, 769.896, 404.414, 404.721],
        band_emitter: None,
        masked_by_sodium: true,
    },
    CationEmission {
        element: "Ca",
        colour: "orange-red",
        lines_nm: &[422.673],
        band_emitter: Some("CaOH"),
        masked_by_sodium: false,
    },
    CationEmission {
        element: "Sr",
        colour: "crimson red",
        lines_nm: &[460.733],
        band_emitter: Some("SrOH"),
        masked_by_sodium: false,
    },
    CationEmission {
        element: "Ba",
        colour: "apple green",
        lines_nm: &[553.548],
        band_emitter: Some("BaOH"),
        masked_by_sodium: false,
    },
    CationEmission {
        element: "Cu",
        colour: "blue-green",
        lines_nm: &[324.754, 327.396],
        band_emitter: Some("CuCl/CuOH"),
        masked_by_sodium: false,
    },
];

/// Elements that are never the emitting cation of a salt.
const NONMETALS: &[&str] = &[
    "H", "He", "B", "C", "N", "O", "F", "Ne", "Si", "P", "S", "Cl", "Ar", "As", "Se", "Br", "Kr",
    "Te", "I", "Xe", "At", "Rn",
];

/// The table row for an element, if the lab carries one.
pub fn emission(element: &str) -> Option<&'static CationEmission> {
    EMISSIONS.iter().find(|row| row.element == element)
}

/// The metal elements of a species formula, and whether the species can
/// burn: carbon that is not already carbonate-oxidised, or a bare metal
/// (magnesium ribbon is a fuel, not a flame-test salt).
fn metals_of(formula: &str) -> Option<(Vec<String>, bool)> {
    let parsed = crate::stoich::parse_formula(formula).ok()?;
    let c = parsed.counts.get("C").copied().unwrap_or(0.0);
    let o = parsed.counts.get("O").copied().unwrap_or(0.0);
    let metals = parsed
        .counts
        .keys()
        .filter(|el| !NONMETALS.contains(&el.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let elemental_metal = parsed.counts.len() == 1 && parsed.charge == 0.0 && metals.len() == 1;
    let fuel = (c > 0.0 && o < 3.0 * c) || elemental_metal;
    Some((metals, fuel))
}

/// Whether the vessel is a flame-test sample: salts (or their ions) and
/// water only, with at least one metal cation and nothing that can burn.
/// Such a sample never enters the combustion solvers.
pub fn is_flame_test_sample(contents: &[Portion]) -> bool {
    let mut cation = false;
    for portion in contents.iter().filter(|p| p.moles.0 > 0.0) {
        let Some(data) = species::lookup(&portion.species) else {
            return false;
        };
        let Some((metals, fuel)) = metals_of(data.formula) else {
            return false;
        };
        if fuel {
            return false;
        }
        if metals.is_empty() {
            let parsed = crate::stoich::parse_formula(data.formula).ok();
            let water = data.key == "water";
            let anion = parsed.is_some_and(|f| f.charge < 0.0);
            if !(water || anion) {
                return false;
            }
        } else {
            cation = true;
        }
    }
    cation
}

/// One flame-test event per cation present, plus a typed refusal for each
/// cation without emission data. Curated per-species colours win for a
/// single-cation salt, so existing salts keep their observed wording.
pub fn flame_events(vessel: VesselId, contents: &[Portion]) -> (Vec<Event>, Vec<Event>) {
    let mut seen: Vec<(String, SpeciesId, Option<&'static str>)> = Vec::new();
    for portion in contents.iter().filter(|p| p.moles.0 > 0.0) {
        let Some(data) = species::lookup(&portion.species) else {
            continue;
        };
        let Some((metals, _)) = metals_of(data.formula) else {
            continue;
        };
        let curated = (metals.len() == 1).then_some(data.flame_colour).flatten();
        for metal in metals {
            if !seen.iter().any(|(el, _, _)| *el == metal) {
                seen.push((metal, portion.species.clone(), curated));
            }
        }
    }
    let sodium = seen
        .iter()
        .find(|(el, _, _)| el == "Na")
        .map(|(_, sid, _)| sid.clone());
    let mut lit = Vec::new();
    let mut refused = Vec::new();
    for (element, sid, curated) in &seen {
        let row = emission(element);
        match curated.or(row.map(|row| row.colour)) {
            Some(colour) => lit.push(Event::FlameTest {
                vessel,
                species: sid.clone(),
                colour: colour.to_string(),
                masked_by: if row.is_some_and(|row| row.masked_by_sodium) {
                    sodium.clone()
                } else {
                    None
                },
            }),
            None => refused.push(Event::not_modeled(
                vessel,
                NotModelledCause::NotParameterised,
                Phrase::new(
                    "not-modeled.flame-emission-no-data",
                    "the flame colour of {element} in {name}: the lab carries no sourced emission data for this cation, so it does not say what colour it gives",
                    vec![
                        ("element".to_string(), Slot::text(element.clone())),
                        (
                            "name".to_string(),
                            Slot::term("species", species::english_name(sid.0.as_str())),
                        ),
                    ],
                ),
            )),
        }
    }
    (lit, refused)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::Phase;
    use crate::units::Moles;

    fn solid(key: &str, n: f64) -> Portion {
        Portion {
            species: SpeciesId::new(key),
            moles: Moles(n),
            phase: Phase::Solid,
        }
    }

    #[test]
    fn every_row_has_lines_and_a_translated_colour_word() {
        for row in EMISSIONS {
            assert!(!row.lines_nm.is_empty(), "{}", row.element);
            assert!(crate::stoich::is_element(row.element), "{}", row.element);
        }
    }

    #[test]
    fn sodium_bromide_is_a_flame_test_and_glows_sodium_yellow() {
        let contents = [solid("NaBr", 0.001)];
        assert!(is_flame_test_sample(&contents));
        let (lit, refused) = flame_events(VesselId(0), &contents);
        assert!(refused.is_empty(), "{refused:?}");
        assert!(
            matches!(lit.as_slice(), [Event::FlameTest { colour, masked_by: None, .. }] if colour == "bright yellow"),
            "{lit:?}"
        );
    }

    #[test]
    fn sodium_and_potassium_both_emit_and_sodium_masks_potassium() {
        let contents = [solid("NaCl", 0.0017), solid("KCl", 0.00013)];
        assert!(is_flame_test_sample(&contents));
        let (lit, refused) = flame_events(VesselId(0), &contents);
        assert!(refused.is_empty(), "{refused:?}");
        assert_eq!(lit.len(), 2, "{lit:?}");
        assert!(lit.iter().any(|e| matches!(e,
            Event::FlameTest { species, masked_by: None, .. } if species.0 == "NaCl")));
        assert!(lit.iter().any(|e| matches!(e,
            Event::FlameTest { species, masked_by: Some(by), .. }
                if species.0 == "KCl" && by.0 == "NaCl")));
    }

    #[test]
    fn potassium_alone_is_not_masked() {
        let (lit, _) = flame_events(VesselId(0), &[solid("KCl", 0.001)]);
        assert!(
            matches!(
                lit.as_slice(),
                [Event::FlameTest {
                    masked_by: None,
                    ..
                }]
            ),
            "{lit:?}"
        );
    }

    #[test]
    fn a_cation_without_emission_data_is_refused_not_painted() {
        // Zinc salts have no row: the lab refuses rather than guesses.
        let contents = [solid("ZnSO4", 0.001)];
        let (lit, refused) = flame_events(VesselId(0), &contents);
        assert!(lit.is_empty(), "{lit:?}");
        assert!(
            matches!(refused.as_slice(), [Event::NotYetModeled {
                cause: NotModelledCause::NotParameterised,
                reason: Some(reason), ..
            }] if reason.key == "not-modeled.flame-emission-no-data"),
            "{refused:?}"
        );
    }

    #[test]
    fn a_fuel_is_not_a_flame_test_sample() {
        assert!(!is_flame_test_sample(&[solid("Mg", 0.001)]));
        assert!(!is_flame_test_sample(&[
            solid("NaCl", 0.001),
            solid("ethanol", 0.01)
        ]));
        assert!(!is_flame_test_sample(&[]));
    }
}

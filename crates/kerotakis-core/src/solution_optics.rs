//! Shared optical accounting for observations and instruments.
//!
//! An analytical component is not a chromophore concentration. When a native
//! distribution exists, use its molality and its own solvent mass. Never
//! assign the free ion's spectrum to its complexes.
use crate::{species, stoich, vessel::Vessel, Phase, SpeciesId};

fn same_species(native: &str, key: &str) -> bool {
    if native == key {
        return true;
    }
    let Some(data) = species::lookup(&SpeciesId::new(key)) else {
        return false;
    };
    // Formula-key ions permit charge-spelling aliases (Cu+2 / Cu2+).
    // Named organics must not be identified merely by empirical formula.
    if data.key != data.formula {
        return false;
    }
    match (
        stoich::parse_formula(native),
        stoich::parse_formula(data.formula),
    ) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

pub(crate) fn visible_moles(vessel: &Vessel, key: &str, analytical: f64) -> f64 {
    let measured = vessel.solution.as_ref().and_then(|solution| {
        let kg = solution
            .solvent_kg
            .filter(|kg| kg.is_finite() && *kg > 0.0)?;
        let data = species::lookup(&SpeciesId::new(key))?;
        let formula = stoich::parse_formula(data.formula).ok()?;
        if formula.charge == 0.0 || solution.species.is_empty() {
            return None;
        }
        let represented = solution.species.iter().any(|s| {
            stoich::parse_formula(&s.name)
                .ok()
                .is_some_and(|f| formula.counts.keys().all(|el| f.counts.contains_key(el)))
        });
        represented.then(|| {
            solution
                .species
                .iter()
                .filter(|s| same_species(&s.name, key))
                .map(|s| s.molality.max(0.0) * kg)
                .sum::<f64>()
        })
    });
    measured.unwrap_or_else(|| {
        (analytical - crate::surface_colour::sequestered_moles(vessel, &SpeciesId::new(key)))
            .max(0.0)
    })
}

/// Coverage of the supported partial spectrum. Missing prerequisites must not
/// be presented as a complete zero-absorbance measurement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpticalCoverage {
    pub missing_spectra: Vec<String>,
    pub missing_ph: Vec<String>,
    /// Dissolved species the registry describes as coloured but holds no
    /// reviewed molar absorptivity for. Reading them as zero would report
    /// iodine as transparent; the instrument refuses instead.
    pub missing_absorptivity: Vec<String>,
}
impl OpticalCoverage {
    pub fn is_complete(&self) -> bool {
        self.missing_spectra.is_empty()
            && self.missing_ph.is_empty()
            && self.missing_absorptivity.is_empty()
    }
}

/// How the visible-absorbance model treats one registry species. The
/// classification is generic: it reads only the registry's curated
/// appearance, its spectrum and the indicator tables, so a new coloured
/// species without ε(λ) is refused the moment it is registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromophoreClass {
    /// Reviewed ε(λ) in the registry.
    Parameterised,
    /// pH-dependent spectrum from the indicator/pigment tables.
    ParameterisedPh,
    /// Curated as coloured, no reviewed ε(λ): the spectrophotometer refuses.
    MissingAbsorptivity,
    /// Curated as colourless/white: no visible absorption is claimed.
    OutsideMethod,
    /// No curated appearance at all; counted as non-absorbing and listed
    /// so the assumption is visible.
    Unclassified,
}

impl ChromophoreClass {
    pub fn key(self) -> &'static str {
        match self {
            ChromophoreClass::Parameterised => "parameterised",
            ChromophoreClass::ParameterisedPh => "parameterised_ph",
            ChromophoreClass::MissingAbsorptivity => "missing_absorptivity",
            ChromophoreClass::OutsideMethod => "outside_method",
            ChromophoreClass::Unclassified => "unclassified",
        }
    }
}

/// Appearance words that claim no visible colour.
const TRANSPARENT_WORDS: &[&str] = &["colourless", "colorless", "white", "clear", "translucent"];

/// Appearance words that name a visible colour.
const COLOUR_WORDS: &[&str] = &[
    "black", "grey", "gray", "silver", "red", "orange", "yellow", "green", "blue", "violet",
    "purple", "pink", "brown",
];

/// Whether a curated appearance claims a visible colour: it names a colour
/// and no transparency ("white to yellowish powder" is not a chromophore
/// claim; "sharp-smelling, toxic" is not a colour at all).
pub fn appearance_is_coloured(appearance: &str) -> bool {
    let lower = appearance.to_lowercase();
    COLOUR_WORDS.iter().any(|w| lower.contains(w))
        && !TRANSPARENT_WORDS.iter().any(|w| lower.contains(w))
}

pub fn chromophore_class(key: &str) -> ChromophoreClass {
    if crate::indicator::is_ph_dependent(key) {
        return ChromophoreClass::ParameterisedPh;
    }
    let Some(data) = species::lookup_key(key) else {
        return ChromophoreClass::Unclassified;
    };
    if data.spectrum.is_some() {
        return ChromophoreClass::Parameterised;
    }
    match data.appearance {
        Some(a) if appearance_is_coloured(a) => ChromophoreClass::MissingAbsorptivity,
        Some(_) => ChromophoreClass::OutsideMethod,
        None => ChromophoreClass::Unclassified,
    }
}

/// Dissolved/liquid inventory curated as coloured with no reviewed ε(λ).
fn missing_absorptivity(vessel: &Vessel) -> Vec<String> {
    vessel
        .contents
        .iter()
        .filter(|p| matches!(p.phase, Phase::Aqueous | Phase::Liquid))
        .filter(|p| p.moles.0 > 0.0)
        .filter(|p| chromophore_class(&p.species.0) == ChromophoreClass::MissingAbsorptivity)
        .filter(|p| visible_moles(vessel, &p.species.0, p.moles.0) > 0.0)
        .map(|p| p.species.0.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The shared instrument coverage disclosure for the spectrophotometer:
/// which present species the method measures, which it would need a
/// parameter for, and which it does not see at all.
pub fn disclosure(vessel: &Vessel) -> crate::instrument_coverage::InstrumentCoverage {
    let coverage = coverage(vessel);
    let mut missing: Vec<String> = coverage
        .missing_spectra
        .iter()
        .chain(&coverage.missing_ph)
        .chain(&coverage.missing_absorptivity)
        .cloned()
        .collect();
    missing.sort();
    missing.dedup();
    let mut supported = Vec::new();
    let mut outside = Vec::new();
    for p in vessel
        .contents
        .iter()
        .filter(|p| matches!(p.phase, Phase::Aqueous | Phase::Liquid) && p.moles.0 > 0.0)
    {
        let key = p.species.0.clone();
        if missing.contains(&key) {
            continue;
        }
        match chromophore_class(&key) {
            ChromophoreClass::Parameterised | ChromophoreClass::ParameterisedPh => {
                supported.push(key)
            }
            _ => outside.push(key),
        }
    }
    crate::instrument_coverage::InstrumentCoverage::new(
        "spectrophotometer",
        supported,
        missing,
        outside,
    )
}

/// Check the actual visible aqueous/liquid inventory, not provenance or a
/// retained solid/adsorbed pool. No missing pH is inferred to be neutral.
pub fn coverage(vessel: &Vessel) -> OpticalCoverage {
    let valid_ph = vessel.solution.as_ref().is_some_and(|s| s.ph.is_finite());
    let missing_ph = if valid_ph {
        Vec::new()
    } else {
        vessel
            .contents
            .iter()
            .filter(|p| matches!(p.phase, Phase::Aqueous | Phase::Liquid))
            .filter(|p| p.moles.0 > 0.0 && crate::indicator::is_ph_dependent(&p.species.0))
            .filter(|p| visible_moles(vessel, &p.species.0, p.moles.0) > 0.0)
            .map(|p| p.species.0.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect()
    };
    OpticalCoverage {
        missing_spectra: missing_native_spectra(vessel),
        missing_ph,
        missing_absorptivity: missing_absorptivity(vessel),
    }
}

/// Legacy missing-spectrum list for scene consumers. Missing pH has a distinct
/// typed cause in coverage(), not an absent spectrum.
pub fn spectral_gaps(vessel: &Vessel) -> Vec<String> {
    coverage(vessel).missing_spectra
}

/// Missing spectra for native species containing a known chromophore element.
/// This is a coverage warning, not a claim that each such species is coloured.
fn missing_native_spectra(vessel: &Vessel) -> Vec<String> {
    let Some(solution) = &vessel.solution else {
        return Vec::new();
    };
    let elements: std::collections::BTreeSet<_> = species::REGISTRY
        .iter()
        .filter(|s| s.spectrum.is_some())
        .filter_map(|s| stoich::parse_formula(s.formula).ok())
        .filter(|f| f.charge != 0.0)
        .flat_map(|f| f.counts.into_keys())
        .filter(|el| !matches!(el.as_str(), "H" | "O" | "C" | "N" | "S"))
        .collect();
    solution
        .species
        .iter()
        .filter(|s| s.molality > 1e-9)
        .filter(|s| {
            stoich::parse_formula(&s.name)
                .ok()
                .is_some_and(|f| f.counts.keys().any(|el| elements.contains(el)))
        })
        .filter(|s| {
            !species::REGISTRY
                .iter()
                .any(|data| data.spectrum.is_some() && same_species(&s.name, data.key))
        })
        .map(|s| s.name.clone())
        .collect()
}

pub fn absorbance(vessel: &Vessel, path_cm: f64) -> [f64; crate::spectrum::BANDS] {
    let mut result = [0.0; crate::spectrum::BANDS];
    let litres = vessel.liquid_volume().0.max(1e-9);
    for portion in &vessel.contents {
        if !matches!(portion.phase, Phase::Aqueous | Phase::Liquid) {
            continue;
        }
        let key = &portion.species.0;
        let spectrum = if crate::indicator::is_ph_dependent(key) {
            vessel
                .solution
                .as_ref()
                .filter(|s| s.ph.is_finite())
                .and_then(|s| crate::indicator::spectrum_at_ph(key, s.ph))
        } else {
            species::lookup(&portion.species)
                .and_then(|s| s.spectrum)
                .copied()
        };
        let Some(spectrum) = spectrum else {
            continue;
        };
        let concentration = visible_moles(vessel, key, portion.moles.0) / litres;
        for (out, epsilon) in result.iter_mut().zip(spectrum) {
            *out += epsilon * concentration * path_cm;
        }
    }
    // Native complexes need not be analytical inventory components. If a
    // spectrum is registered for one, include it even without a bottle portion.
    if let Some(solution) = &vessel.solution {
        if let Some(kg) = solution.solvent_kg.filter(|kg| kg.is_finite() && *kg > 0.0) {
            for native in &solution.species {
                if let Some(data) = species::REGISTRY
                    .iter()
                    .find(|data| data.spectrum.is_some() && same_species(&native.name, data.key))
                {
                    if vessel.contents.iter().any(|p| {
                        p.species.0 == data.key && matches!(p.phase, Phase::Aqueous | Phase::Liquid)
                    }) {
                        continue;
                    }
                    for (out, epsilon) in result.iter_mut().zip(data.spectrum.unwrap()) {
                        *out += epsilon * native.molality.max(0.0) * kg / litres * path_cm;
                    }
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        vessel::{SolutionInfo, SpeciesDetail},
        Moles, VesselId,
    };

    #[test]
    fn complexes_do_not_inherit_the_free_ions_absorption() {
        let mut v = Vessel::new(VesselId(0), "beaker");
        v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        v.deposit(SpeciesId::new("Cu+2"), Moles(0.01), Phase::Aqueous);
        let total_spectrum = absorbance(&v, 1.0);
        assert!(total_spectrum.iter().any(|x| *x > 0.0));
        v.solution = Some(SolutionInfo {
            solvent_activity: None,
            scope: Default::default(),
            solvent_kg: Some(0.1),
            particle_molality: None,
            ph: 9.0,
            pe: None,
            redox: vec![],
            ionic_strength: 0.1,
            provenance: None,
            species: vec![
                SpeciesDetail {
                    name: "Cu+2".into(),
                    molality: 0.001,
                    activity: 0.0005,
                },
                SpeciesDetail {
                    name: "Cu(NH3)4+2".into(),
                    molality: 0.099,
                    activity: 0.05,
                },
            ],
        });
        let free_spectrum = absorbance(&v, 1.0);
        for (free, total) in free_spectrum.iter().zip(total_spectrum) {
            assert!((free - total * 0.01).abs() < 1e-12);
        }
        assert_eq!(spectral_gaps(&v), vec!["Cu(NH3)4+2"]);
        assert!(crate::appearance::observe(&v)
            .words
            .contains("Colour is incomplete"));
        v.solution.as_mut().unwrap().species.remove(0);
        assert_eq!(absorbance(&v, 1.0), [0.0; crate::spectrum::BANDS]);
    }

    #[test]
    fn beer_lambert_uses_molarity_and_path_length_not_molality() {
        use crate::instrument::Spectrophotometer;
        for amount in [1e-6, 1e-4, 1e-2] {
            let mut v = Vessel::new(VesselId(0), "beaker");
            v.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
            v.deposit(SpeciesId::new("MnO4-"), Moles(amount), Phase::Aqueous);
            let one = absorbance(&v, 1.0);
            for path in [0.1, 0.5, 2.0, 10.0] {
                let measured = Spectrophotometer { path_cm: path }.measure_spectrum(&v);
                for (a, b) in measured.iter().zip(one) {
                    assert!((a - b * path).abs() < 1e-10);
                }
            }
        }
    }
}

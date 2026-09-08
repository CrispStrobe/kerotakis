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

/// Missing spectra for native species containing a known chromophore element.
/// This is a coverage warning, not a claim that each such species is coloured.
pub fn spectral_gaps(vessel: &Vessel) -> Vec<String> {
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
            scope: Default::default(),
            solvent_kg: Some(0.1),
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

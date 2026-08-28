//! The blue-black iodine/iodide/starch observation.
//!
//! Lugol solution is not merely iodine in water: iodide makes iodine soluble,
//! and polyiodide hosted by sufficiently long amylose helices produces a broad
//! charge-transfer band around 600--650 nm. The detailed polyiodide structure
//! and binding stoichiometry remain debated, so this module owns only the
//! bounded observable needed by the school starch test.
//!
//! Optical basis: Pesek et al., *Molecules* 27 (2022) 8974,
//! doi:10.3390/molecules27248974 (CC BY), with the broader evidence review in
//! Pesek & Silaghi-Dumitrescu, *Molecules* 29 (2024) 641,
//! doi:10.3390/molecules29030641 (CC BY).

use crate::species::Phase;
use crate::{Moles, SpeciesId, Vessel};

const IODINE: &str = "I2";
const IODIDE_SOURCE: &str = "KI";
const STARCH: &str = "starch";

/// Editorial upper bound: one optically active iodine equivalent per twenty
/// anhydroglucose units. This is a saturation parameter, not a molecular
/// formula for a complex whose polyiodide structure remains unsettled.
const STARCH_SITE_FRACTION: f64 = 0.05;

/// Literature-backed optical surrogate for the broad amylose-polyiodide band.
/// Reviews and primary spectra place the maximum between roughly 600 and
/// 650 nm depending on starch source, chain length, temperature and iodide.
const COMPLEX_PEAK_NM: f64 = 620.0;
const COMPLEX_PEAK_EPSILON: f64 = 2_500.0;
const COMPLEX_WIDTH_NM: f64 = 75.0;
const FREE_POLYIODIDE_PEAK_NM: f64 = 460.0;
const FREE_POLYIODIDE_PEAK_EPSILON: f64 = 50.0;
const FREE_POLYIODIDE_WIDTH_NM: f64 = 100.0;

fn phase_moles(vessel: &Vessel, key: &str, phase: Phase) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == key && portion.phase == phase)
        .map(|portion| portion.moles.0)
        .sum()
}

fn total_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|portion| portion.species.0 == key)
        .map(|portion| portion.moles.0)
        .sum()
}

/// Iodide-assisted iodine transfer to the aqueous bookkeeping phase.
///
/// One aqueous iodide equivalent can support at most one iodine equivalent in
/// this reduced representation. KI is retained: this is phase bookkeeping for
/// I2/I-/polyiodide solution, not a claim that KI is consumed.
pub(crate) fn iodine_to_dissolve(vessel: &Vessel) -> Moles {
    if vessel.liquid_volume().0 <= 0.0 {
        return Moles(0.0);
    }
    let solid_iodine = phase_moles(vessel, IODINE, Phase::Solid);
    let iodide_inventory = total_moles(vessel, IODIDE_SOURCE);
    let already_aqueous = phase_moles(vessel, IODINE, Phase::Aqueous);
    Moles(solid_iodine.min((iodide_inventory - already_aqueous).max(0.0)))
}

/// Effective iodine equivalents contributing the blue-black complex.
pub(crate) fn complex_moles(vessel: &Vessel) -> f64 {
    if phase_moles(vessel, "water", Phase::Liquid) <= 0.0 {
        return 0.0;
    }
    let iodine = phase_moles(vessel, IODINE, Phase::Aqueous);
    let iodide = phase_moles(vessel, IODIDE_SOURCE, Phase::Aqueous);
    let starch_sites = total_moles(vessel, STARCH) * STARCH_SITE_FRACTION;
    iodine.min(iodide).min(starch_sites).max(0.0)
}

pub(crate) fn has_aqueous_lugol_colour(vessel: &Vessel) -> bool {
    phase_moles(vessel, IODINE, Phase::Aqueous) > 0.0
        && phase_moles(vessel, IODIDE_SOURCE, Phase::Aqueous) > 0.0
}

pub(crate) fn add_absorbance(
    vessel: &Vessel,
    litres: f64,
    path_cm: f64,
    absorbance: &mut crate::spectrum::Spectrum,
) -> f64 {
    let complex = complex_moles(vessel);
    let aqueous_iodine = phase_moles(vessel, IODINE, Phase::Aqueous);
    let free_iodine = (aqueous_iodine - complex).max(0.0);
    for (moles, peak) in [
        (
            free_iodine,
            (
                FREE_POLYIODIDE_PEAK_NM,
                FREE_POLYIODIDE_PEAK_EPSILON,
                FREE_POLYIODIDE_WIDTH_NM,
            ),
        ),
        (
            complex,
            (COMPLEX_PEAK_NM, COMPLEX_PEAK_EPSILON, COMPLEX_WIDTH_NM),
        ),
    ] {
        if moles <= 0.0 {
            continue;
        }
        let spectrum = crate::spectrum::bands(&[peak]);
        let concentration = moles / litres.max(1e-12);
        for (total, epsilon) in absorbance.iter_mut().zip(spectrum) {
            *total += epsilon * concentration * path_cm;
        }
    }
    complex
}

pub(crate) fn covers_solid(vessel: &Vessel, species: &SpeciesId) -> bool {
    species.0 == STARCH && complex_moles(vessel) > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lugol_starch_vessel() -> Vessel {
        let mut vessel = Vessel::new(crate::VesselId(0), "beaker");
        vessel.deposit(SpeciesId::new("water"), Moles(5.55), Phase::Liquid);
        vessel.deposit(SpeciesId::new(IODIDE_SOURCE), Moles(0.001), Phase::Aqueous);
        vessel.deposit(SpeciesId::new(IODINE), Moles(0.0005), Phase::Aqueous);
        vessel.deposit(SpeciesId::new(STARCH), Moles(0.02), Phase::Solid);
        vessel
    }

    #[test]
    fn all_three_components_and_water_are_required() {
        let vessel = lugol_starch_vessel();
        assert!(complex_moles(&vessel) > 0.0);
        for missing in [IODINE, IODIDE_SOURCE, STARCH, "water"] {
            let mut control = vessel.clone();
            control
                .contents
                .retain(|portion| portion.species.0 != missing);
            assert_eq!(complex_moles(&control), 0.0, "missing {missing}");
        }
    }

    #[test]
    fn complex_is_bounded_by_each_inventory() {
        let vessel = lugol_starch_vessel();
        let complex = complex_moles(&vessel);
        assert!(complex <= phase_moles(&vessel, IODINE, Phase::Aqueous));
        assert!(complex <= phase_moles(&vessel, IODIDE_SOURCE, Phase::Aqueous));
        assert!(complex <= total_moles(&vessel, STARCH) * STARCH_SITE_FRACTION);
    }
}

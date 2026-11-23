//! The biuret test: protein + alkali + a little copper(II) turns violet.
//!
//! In strong alkali the deprotonated peptide nitrogens of a chain bind
//! copper(II) as a square-planar Cu–N₄ chelate. The chelate holds the copper
//! in solution — a protein-free blank throws down pale-blue Cu(OH)₂ — and
//! absorbs near 540–550 nm, which is why the solution looks violet. The
//! colour reaction is the basis of the biuret protein assay (Gornall,
//! Bardawill & David, J. Biol. Chem. 177 (1949) 751–766, read at 540 nm).
//!
//! WHAT THIS MODULE OWNS: an eye-only observation, the same kind of bounded
//! surrogate as [`crate::starch_iodine`]. Proteins are recipe-level
//! conserved material ([`crate::protein`]), not species, so the copper stays
//! booked in the ledger as the hydroxide or ion the aqueous solver gave it;
//! the chelate changes how the tube LOOKS, not what it contains. The band
//! position follows the assay; its absorptivity and the four-residues-per-Cu
//! site count below are editorial, NOT a calibrated protein assay, and a
//! typed note says so whenever the colour is shown.

use crate::species::Phase;
use crate::Vessel;

/// Mean amino-acid residue mass, g/mol (the conventional ~110 Da).
const MEAN_RESIDUE_G_PER_MOL: f64 = 110.0;
/// Peptide nitrogens per chelated copper (square-planar Cu–N₄).
const RESIDUES_PER_COPPER: f64 = 4.0;

const PEAK_NM: f64 = 545.0;
/// Editorial eye-only band strength per chelated copper, L mol⁻¹ cm⁻¹.
const PEAK_EPSILON: f64 = 60.0;
const WIDTH_NM: f64 = 70.0;

const TRACE: f64 = 1e-12;

fn moles(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && phases.contains(&p.phase))
        .map(|p| p.moles.0)
        .sum()
}

/// Copper(II) the chelate can draw on: the dissolved ion, the bottle salt
/// before a solve, and freshly precipitated hydroxide (which a peptide in
/// alkali redissolves). Black CuO is not reclaimed.
fn copper_ii(vessel: &Vessel) -> f64 {
    moles(vessel, "Cu+2", &[Phase::Aqueous])
        + moles(vessel, "CuSO4", &[Phase::Aqueous, Phase::Solid])
        + moles(vessel, "Cu(OH)2", &[Phase::Solid])
}

pub(crate) fn peptide_sites(vessel: &Vessel) -> f64 {
    let grams: f64 = crate::protein::observe(vessel)
        .iter()
        .map(|p| p.protein_mass_g)
        .sum();
    grams / MEAN_RESIDUE_G_PER_MOL / RESIDUES_PER_COPPER
}

/// Moles of copper held in the violet peptide chelate.
pub fn complex_moles(vessel: &Vessel) -> f64 {
    if moles(vessel, "water", &[Phase::Liquid]) <= TRACE || !crate::reducing_sugar::alkaline(vessel)
    {
        return 0.0;
    }
    copper_ii(vessel).min(peptide_sites(vessel)).max(0.0)
}

/// Whether the chelate has taken up all the copper(II) hydroxide, so no
/// blue precipitate is to be seen.
pub fn covers_solid(vessel: &Vessel, species: &crate::SpeciesId) -> bool {
    species.0 == "Cu(OH)2" && {
        let complex = complex_moles(vessel);
        complex > TRACE && complex >= copper_ii(vessel) - TRACE
    }
}

/// Add the chelate's violet band to `absorbance`.
pub(crate) fn add_absorbance(
    vessel: &Vessel,
    litres: f64,
    path_cm: f64,
    absorbance: &mut crate::spectrum::Spectrum,
) -> f64 {
    let complex = complex_moles(vessel);
    if complex > TRACE {
        let band = crate::spectrum::bands(&[(PEAK_NM, PEAK_EPSILON, WIDTH_NM)]);
        let concentration = complex / litres.max(1e-12);
        for (total, epsilon) in absorbance.iter_mut().zip(band) {
            *total += epsilon * concentration * path_cm;
        }
    }
    complex
}

pub(crate) fn note() -> crate::phrase::Phrase {
    crate::phrase::Phrase::bare(
        "look.biuret-surrogate",
        "The violet is the biuret copper–peptide colour, drawn as an eye-only surrogate band: it is not a calibrated protein assay, and the copper stays booked in the inventory as the solver left it.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Moles, SpeciesId, VesselId};

    fn blank() -> Vessel {
        let mut v = Vessel::new(VesselId(0), "test tube");
        v.deposit(SpeciesId::new("water"), Moles(0.5), Phase::Liquid);
        v.deposit(SpeciesId::new("Na+"), Moles(0.005), Phase::Aqueous);
        v.deposit(
            SpeciesId::new("base_equivalents"),
            Moles(0.005),
            Phase::Aqueous,
        );
        v.deposit(SpeciesId::new("Cu(OH)2"), Moles(5e-5), Phase::Solid);
        v
    }

    #[test]
    fn no_protein_no_violet() {
        let v = blank();
        assert_eq!(complex_moles(&v), 0.0);
        assert!(!covers_solid(&v, &SpeciesId::new("Cu(OH)2")));
    }

    #[test]
    fn site_count_is_bounded_by_copper() {
        let v = blank();
        assert!(complex_moles(&v) <= copper_ii(&v));
    }
}

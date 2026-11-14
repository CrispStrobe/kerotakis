//! Solid and liquid density of the SAME substance, where both are sourced
//! as a pair (th-004, 2026-11-11, round 4).
//!
//! The registry carries one density per species, for the phase it is
//! stocked in. A substance that has melted in the beaker was therefore
//! weighed against its own solid at the solid's density, and whether the
//! remaining solid floated or sank in its own melt came down to a
//! floating-point tie: th-004's half-melted paraffin "floated on top" of
//! the wax it was melting in. Real paraffin, like most substances, is
//! denser as a solid than as a melt and sinks in it.
//!
//! A row here supplies both densities from ONE source, so the comparison
//! is between two measured numbers rather than a number and itself.
//! Where no row exists the registry density is used as before for both
//! phases; nothing is invented for a melt whose density is not sourced.

use crate::species::{self, Phase, SpeciesId};

/// A sourced solid/liquid density pair of one substance, g/mL.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaseDensityPair {
    pub species: &'static str,
    /// Density of the solid, g/mL, at `solid_at_c`.
    pub solid_g_per_ml: f64,
    pub solid_at_c: f64,
    /// Density of the melt, g/mL, at `liquid_at_c`.
    pub liquid_g_per_ml: f64,
    pub liquid_at_c: f64,
    /// Primary citation with DOI.
    pub source: &'static str,
}

/// * Paraffin wax: A. Gasia, L. Miró, A. de Gracia, C. Barreneche and
///   L. F. Cabeza, 'Experimental evaluation of a paraffin as phase change
///   material for thermal energy storage in laboratory equipment and in a
///   shell-and-tube heat exchanger', Appl. Sci. 6 (2016) 112,
///   doi:10.3390/app6040112, Table 1 (technical paraffin RT58, melting
///   53-59 degrees Celsius, manufacturer data): density solid 0.88 kg/L at
///   15 degrees Celsius, liquid 0.77 kg/L at 80 degrees Celsius, volume
///   expansion 12.5 %. The solid value agrees with the ~900 kg/m3 quoted
///   for paraffin wax by https://en.wikipedia.org/wiki/Paraffin_wax.
pub const PHASE_DENSITY_PAIRS: &[PhaseDensityPair] = &[PhaseDensityPair {
    species: "paraffin",
    solid_g_per_ml: 0.88,
    solid_at_c: 15.0,
    liquid_g_per_ml: 0.77,
    liquid_at_c: 80.0,
    source: "A. Gasia, L. Miró, A. de Gracia, C. Barreneche and L. F. Cabeza, Appl. Sci. 6 (2016) 112, doi:10.3390/app6040112, Table 1 (paraffin RT58): density solid 0.88 kg/L at 15 °C, liquid 0.77 kg/L at 80 °C",
}];

/// The sourced pair for a species, where it has one.
pub fn pair(species: &str) -> Option<&'static PhaseDensityPair> {
    PHASE_DENSITY_PAIRS
        .iter()
        .find(|row| row.species == species)
}

/// Density of `species` in `phase`, g/mL: the sourced pair's value for a
/// solid or a liquid where a pair exists, otherwise the registry's single
/// density. `None` when neither is known (or the registry carries 0).
pub fn density_in_phase(species: &SpeciesId, phase: Phase) -> Option<f64> {
    if let Some(row) = pair(species.0.as_str()) {
        match phase {
            Phase::Solid => return Some(row.solid_g_per_ml),
            Phase::Liquid => return Some(row.liquid_g_per_ml),
            _ => {}
        }
    }
    species::lookup(species)
        .map(|data| data.density)
        .filter(|d| *d > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pair_is_cited_positive_and_registered() {
        for row in PHASE_DENSITY_PAIRS {
            assert!(row.source.contains("doi"), "{}", row.species);
            assert!(row.solid_g_per_ml > 0.0 && row.liquid_g_per_ml > 0.0);
            assert!(
                species::lookup(&SpeciesId::new(row.species)).is_some(),
                "{} is not in the registry",
                row.species
            );
        }
    }
}

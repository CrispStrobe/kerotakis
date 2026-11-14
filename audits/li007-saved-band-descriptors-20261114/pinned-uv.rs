//! BRD-014.S05: what a sun-protection material does to ultraviolet light.
//!
//! The question "does sunscreen absorb UV?" invites a Beer–Lambert answer,
//! and the assessment that preceded this module rejected that on two
//! grounds: the visible spectral table (`spectrum::BANDS`) is a colour
//! model with forty-odd consumers and no UV in it, and the mineral filters
//! scatter as much as they absorb, so an absorbance would misdescribe them.
//! What a sun-protection label actually states is an ATTENUATION per band:
//! the SPF is an erythemal dose ratio the UV-B band dominates, and a
//! broad-spectrum label adds a UV-A protection factor. So a named material
//! carries [`MaterialRole::UvAttenuation`] — two factors at the standard
//! film — and `irradiate` at a UV wavelength reads the transmitted fraction
//! off it, with the mechanism in words. No spectrum inside a band, no
//! photostability, no skin: the boundary says so.

use crate::material::{self, MaterialRole};
use crate::vessel::Vessel;

/// The two bands a sun-protection label is defined over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UvBand {
    /// 280–320 nm: the erythemal band, and the one the SPF measures.
    UvB,
    /// 320–400 nm: the broad-spectrum band, and the UV-A protection factor.
    UvA,
}

impl UvBand {
    pub fn of(wavelength_nm: f64) -> Option<Self> {
        if (280.0..320.0).contains(&wavelength_nm) {
            Some(Self::UvB)
        } else if (320.0..400.0).contains(&wavelength_nm) {
            Some(Self::UvA)
        } else {
            None
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UvB => "UV-B",
            Self::UvA => "UV-A",
        }
    }
}

/// What one named material did to the light.
#[derive(Debug, Clone, PartialEq)]
pub struct UvReading {
    pub material: String,
    pub wavelength_nm: f64,
    pub band: UvBand,
    /// Fraction of the incident light that gets through, 0–1.
    pub transmitted_fraction: f64,
    pub mechanism: String,
    pub boundary: String,
}

/// Every named material in the vessel that attenuates light at this
/// wavelength, with the fraction it lets through. Empty outside 280–400 nm
/// and for materials without the role: the bench then says nothing, which
/// is what it said before this module existed.
pub fn attenuate(vessel: &Vessel, wavelength_nm: f64) -> Vec<UvReading> {
    let Some(band) = UvBand::of(wavelength_nm) else {
        return Vec::new();
    };
    material::named_objects(vessel)
        .into_iter()
        .filter_map(|recipe| {
            recipe.roles.iter().find_map(|role| match role {
                MaterialRole::UvAttenuation {
                    spf,
                    uva_protection_factor,
                    mechanism,
                    boundary,
                    ..
                } => {
                    let factor = match band {
                        UvBand::UvB => *spf,
                        UvBand::UvA => *uva_protection_factor,
                    };
                    Some(UvReading {
                        material: recipe.name.clone(),
                        wavelength_nm,
                        band,
                        transmitted_fraction: (1.0 / factor).clamp(0.0, 1.0),
                        mechanism: mechanism.clone(),
                        boundary: boundary.clone(),
                    })
                }
                _ => None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_are_the_labels_bands() {
        assert_eq!(UvBand::of(300.0), Some(UvBand::UvB));
        assert_eq!(UvBand::of(350.0), Some(UvBand::UvA));
        assert_eq!(UvBand::of(400.0), None);
        assert_eq!(UvBand::of(250.0), None);
    }
}

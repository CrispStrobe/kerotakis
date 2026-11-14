//! Colour of a vessel's own headspace from cited gas-phase cross sections.
//!
//! A sealed jar of chlorine is yellow-green, of bromine vapour red-brown,
//! of iodine vapour violet. The liquid colour path ([`crate::spectrum`])
//! is Beer–Lambert over dissolved species; this module is the same law
//! over the gas a vessel OWNS (sealed or pressure-controlled):
//!
//! ```text
//! A(λ) = Σ ε_gas(λ) · (n_gas / V_headspace) · l
//! ```
//!
//! with ε in L mol⁻¹ cm⁻¹ from the absorption cross section σ in
//! cm² molecule⁻¹ (ε = σ · N_A / (1000 · ln 10) = σ / 3.8235e-21) and
//! `l` = [`crate::spectrum::BEAKER_PATH_CM`], the same viewing depth the
//! liquid uses. Each ε is the mean of the cited cross section over the band
//! centre ± 10 nm (structured spectra such as I₂ are therefore band
//! averages, not line peaks).
//!
//! Cross sections are the MPI-Mainz UV/VIS Spectral Atlas data files
//! (Keller-Rudek et al., Earth Syst. Sci. Data 5 (2013) 365,
//! doi:10.5194/essd-5-365-2013; archive doi:10.5281/zenodo.6951, CC BY 4.0),
//! each naming its primary measurement or evaluation below.
//!
//! A gas present in the headspace with neither a cited row nor a cited or
//! registry-stated colourless appearance blocks the colour: the scene then
//! carries no colour and names the gas, rather than drawing it clear.
//! An open or swept vessel has no headspace of its own and no colour here.
//! Coverage: `docs/gas-colour-coverage-20261112.md`.

use serde::{Deserialize, Serialize};

use crate::species::Phase;
use crate::spectrum::{transmitted_colour, Rgb, Spectrum, BANDS, BEAKER_PATH_CM};
use crate::vessel::Vessel;

/// One cited gas-phase spectrum on the 16 renderer bands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GasSpectrum {
    /// Registry key of the gas portion.
    pub species: &'static str,
    /// ε per band, L mol⁻¹ cm⁻¹ (band mean of σ / 3.8235e-21).
    pub epsilon: Spectrum,
    pub source: &'static str,
}

const CL2_JPL: &str =
    "Sander et al., JPL Publication 10-6 (2011) recommendation (Maric, Burrows & \
Moortgat, J. Photochem. Photobiol. A 70 (1993) 205), 298 K, 260-550 nm, via the MPI-Mainz atlas \
file Cl2_JPL-2010(2011)_298K_260-550nm(rec); 565-705 nm lie beyond the data, where the continuum \
has fallen below 0.003 L mol-1 cm-1 (545 nm band), so they are 0";
const BR2_JPL: &str = "Sander et al., JPL Publication 10-6 (2011) recommendation (Maric, Burrows & \
Moortgat, J. Photochem. Photobiol. A 83 (1994) 179), 298 K, 200-650 nm; 665-705 nm from Passchier, \
Christian & Gregory, J. Phys. Chem. 71 (1967) 937, doi:10.1021/j100863a025 (agrees within ~4 % where \
both exist); both via the MPI-Mainz atlas";
const I2_SAIZ_LOPEZ: &str = "Saiz-Lopez, Saunders, Joseph, Ashworth & Plane, Atmos. Chem. Phys. 4 \
(2004) 1443, 295 K, 182-750 nm, via the MPI-Mainz atlas (band means over the banded 500-630 nm \
region)";
const NO2_VANDAELE: &str = "Vandaele, Hermans, Fally, Carleer, Colin, Merienne & Jenouvrier, \
high-resolution Fourier transform NO2 visible cross section (2002), 294 K, 384-725 nm, atlas file \
NO2_Vandaele(2002)_294K_384-725nm(vis-dilut1); the data underlying the JPL-2010 recommendation";
const O3_JPL: &str =
    "Sander et al., JPL Publication 10-6 (2011) recommendation (Reims group: Brion, \
Chakir, Charbonnier, Daumont, Parisse & Malicet), 293-298 K, via the MPI-Mainz atlas";

/// The cited table: ε at 405, 425, … 705 nm.
pub const TABLE: &[GasSpectrum] = &[
    GasSpectrum {
        species: "Cl2",
        epsilon: [
            4.08, 2.21, 1.22, 0.549, 0.195, 0.0556, 0.013, 0.00255, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0,
        ],
        source: CL2_JPL,
    },
    GasSpectrum {
        species: "Br2",
        epsilon: [
            157.0, 162.0, 135.0, 115.0, 100.0, 75.7, 47.8, 27.0, 14.8, 7.95, 3.91, 1.66, 0.67,
            0.288, 0.288, 0.0994,
        ],
        source: BR2_JPL,
    },
    GasSpectrum {
        species: "I2",
        epsilon: [
            9.29, 25.4, 70.9, 193.0, 406.0, 688.0, 828.0, 745.0, 420.0, 204.0, 116.0, 79.0, 58.8,
            47.2, 35.8, 21.5,
        ],
        source: I2_SAIZ_LOPEZ,
    },
    GasSpectrum {
        species: "NO2",
        epsilon: [
            158.0, 143.0, 125.0, 101.0, 77.4, 56.7, 39.9, 27.8, 19.0, 12.5, 8.0, 5.94, 4.92, 3.44,
            3.55, 4.44,
        ],
        source: NO2_VANDAELE,
    },
    GasSpectrum {
        species: "O3",
        epsilon: [
            0.00482, 0.0153, 0.0415, 0.0964, 0.197, 0.364, 0.587, 0.835, 1.12, 1.21, 1.27, 0.999,
            0.72, 0.498, 0.327, 0.213,
        ],
        source: O3_JPL,
    },
];

/// Gases cited as without visible absorption (380-750 nm) at bench
/// amounts: (registry key, evidence).
pub const COLOURLESS: &[(&str, &str)] = &[
    (
        "SO2",
        "Hermans, Vandaele & Fally, JQSRT 110 (2009) 756: 1.1e-24 cm2 at 405 nm, noise beyond 410 nm",
    ),
    (
        "hydrogen_sulfide",
        "Grosch, Fateev & Clausen, JQSRT 154 (2015) 28: below detection from 340 nm",
    ),
    (
        "NH3",
        "Cheng et al., Astrophys. J. 647 (2006) 1535: absorption ends near 230 nm",
    ),
    (
        "CO2",
        "Shemansky, J. Chem. Phys. 56 (1972) 1582: 1.4e-25 cm2 at 300 nm, none in the visible",
    ),
    (
        "O2",
        "He et al., Atmos. Chem. Phys. 21 (2021) 14927: 1.5e-26 cm2 at 405 nm, falling to 1.3e-27",
    ),
];

/// Below this many moles a gas portion does not count.
const TRACE: f64 = 1e-12;

/// The headspace colour a renderer may draw.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeadspaceColour {
    /// Transmitted sRGB through `path_cm` of headspace; `None` when a
    /// present gas has no cited spectrum (see `uncited`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub srgb: Option<[u8; 3]>,
    pub path_cm: f64,
    /// The cited coloured gases that contribute.
    pub gases: Vec<String>,
    /// Gases present with neither a cited spectrum nor a stated colourless
    /// appearance; any entry withholds `srgb`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncited: Vec<String>,
}

fn row(key: &str) -> Option<&'static GasSpectrum> {
    TABLE.iter().find(|r| r.species == key)
}

/// Whether a gas is known to be colourless: cited above, or the registry's
/// own appearance says so.
pub fn is_colourless(key: &str) -> bool {
    COLOURLESS.iter().any(|(k, _)| *k == key)
        || crate::species::lookup_key(key)
            .and_then(|d| d.appearance)
            .is_some_and(|a| {
                let a = a.to_ascii_lowercase();
                a.contains("colourless") || a.contains("colorless")
            })
}

/// Colour of the vessel's own headspace, or `None` when it has none to
/// show (open vessel, or only colourless gas).
pub fn headspace_colour(vessel: &Vessel) -> Option<HeadspaceColour> {
    let litres = vessel.headspace_volume()?.0;
    if !(litres.is_finite() && litres > 0.0) {
        return None;
    }
    let mut absorbance = [0.0; BANDS];
    let mut gases = Vec::new();
    let mut uncited = Vec::new();
    for portion in vessel
        .contents
        .iter()
        .filter(|p| p.phase == Phase::Gas && p.moles.0 > TRACE)
    {
        let key = portion.species.0.as_str();
        if let Some(r) = row(key) {
            let molar = portion.moles.0 / litres;
            for (a, e) in absorbance.iter_mut().zip(r.epsilon) {
                *a += e * molar * BEAKER_PATH_CM;
            }
            if !gases.iter().any(|g| g == key) {
                gases.push(key.to_string());
            }
        } else if !is_colourless(key) && !uncited.iter().any(|g| g == key) {
            uncited.push(key.to_string());
        }
    }
    if gases.is_empty() && uncited.is_empty() {
        return None;
    }
    Some(HeadspaceColour {
        srgb: uncited.is_empty().then(|| {
            let Rgb { r, g, b } = transmitted_colour(&absorbance);
            [r, g, b]
        }),
        path_cm: BEAKER_PATH_CM,
        gases,
        uncited,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::species::SpeciesId;
    use crate::units::{Liters, Moles};
    use crate::vessel::{Headspace, VesselId};

    fn jar(gas: &str, moles: f64) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "jar");
        v.headspace = Headspace::Sealed {
            volume: Liters(0.5),
        };
        v.deposit(SpeciesId::new(gas), Moles(moles), Phase::Gas);
        v
    }

    #[test]
    fn epsilon_is_sigma_over_the_cited_conversion() {
        // Cl2 at 405 nm: sigma 1.56e-20 cm2 -> 4.08 L/mol/cm.
        assert!((1.56e-20 / 3.8235e-21 - TABLE[0].epsilon[0]).abs() < 0.01);
    }

    #[test]
    fn a_jar_of_chlorine_is_yellow_green_and_a_jar_of_air_shows_nothing() {
        let c = headspace_colour(&jar("Cl2", 0.02)).expect("coloured");
        let [r, g, b] = c.srgb.expect("cited");
        assert!(b < g && b < r, "{:?}", c.srgb);
        assert!(headspace_colour(&jar("O2", 0.02)).is_none());
    }

    #[test]
    fn iodine_vapour_absorbs_green_so_it_looks_violet() {
        let [r, g, b] = headspace_colour(&jar("I2", 1e-4)).unwrap().srgb.unwrap();
        assert!(g < r && g < b, "{r} {g} {b}");
    }

    #[test]
    fn an_open_vessel_has_no_headspace_colour() {
        let mut v = jar("Cl2", 0.02);
        v.headspace = Headspace::Open;
        assert!(headspace_colour(&v).is_none());
    }

    #[test]
    fn every_row_and_colourless_entry_names_a_source() {
        for r in TABLE {
            assert!(
                r.source.contains("19") || r.source.contains("20"),
                "{}",
                r.species
            );
            assert!(r.epsilon.iter().all(|e| e.is_finite() && *e >= 0.0));
        }
        for (k, s) in COLOURLESS {
            assert!(!s.is_empty(), "{k}");
        }
    }
}

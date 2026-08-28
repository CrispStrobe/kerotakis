//! Colour, done properly: absorption spectra rather than a tint per
//! species.
//!
//! One RGB value per solute cannot answer the questions a chemistry lab
//! asks. Mixtures do not blend in RGB — their *absorbances add*. Dilute
//! permanganate is pink and concentrated permanganate is purple, which is
//! not two colours but one spectrum at two concentrations. A test tube and
//! a beaker of the same solution look different because the light travels
//! further. All three fall out of Beer–Lambert and none of them fall out of
//! colour mixing.
//!
//! So a coloured species carries a coarse molar absorptivity spectrum —
//! sixteen bands across the visible — and the pipeline is the real one:
//!
//! ```text
//! A(λ) = Σᵢ εᵢ(λ) · cᵢ · l        (absorbances add)
//! T(λ) = 10^−A(λ)                  (Beer–Lambert)
//! XYZ  = ∫ T(λ) S(λ) x̄ȳz̄(λ) dλ    (CIE 1931 observer)
//! sRGB = gamma(M · XYZ)
//! ```
//!
//! The colour-matching functions are computed from the published
//! multi-lobe piecewise-Gaussian fit (Wyman, Sklar & Hoffman, *JCGT* 2013)
//! rather than copied from a table — a formula we implement, not data we
//! redistribute.
//!
//! What stays curated is ε(λ) itself, and it must: the interesting ions
//! absorb through d–d and charge-transfer bands that no data we can ship
//! delivers honestly, and TD-DFT is least reliable for exactly those
//! transitions. Reflective colour (a white powder, a black lump) is different
//! physics again — scattering rather than transmission — and opaque pigment
//! mixtures use the separate Kubelka–Munk path in [`crate::pigment`].

use serde::{Deserialize, Serialize};

/// Number of spectral bands a curated spectrum carries.
pub const BANDS: usize = 16;

/// Band centres, nm. 405–705 in 20 nm steps: the visible range plus enough
/// of the red edge to catch copper's band, which peaks in the near
/// infrared and colours solutions only with its tail.
pub const BAND_NM: [f64; BANDS] = [
    405.0, 425.0, 445.0, 465.0, 485.0, 505.0, 525.0, 545.0, 565.0, 585.0, 605.0, 625.0, 645.0,
    665.0, 685.0, 705.0,
];

/// Molar absorptivity per band, L·mol⁻¹·cm⁻¹.
pub type Spectrum = [f64; BANDS];

/// The depth of liquid light travels through in a typical vessel, cm.
/// A beaker viewed from above is a few centimetres of solution; a test
/// tube viewed from the side is about one. Path length is why the same
/// solution looks different in different glassware, so it is a parameter
/// rather than a constant.
pub const BEAKER_PATH_CM: f64 = 4.0;

/// An sRGB colour.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Piecewise-Gaussian lobe: σ differs either side of the peak.
fn lobe(x: f64, mu: f64, sigma_low: f64, sigma_high: f64) -> f64 {
    let sigma = if x < mu { sigma_low } else { sigma_high };
    let t = (x - mu) / sigma;
    (-0.5 * t * t).exp()
}

/// CIE 1931 2° colour-matching functions, from the published multi-lobe
/// fit (Wyman, Sklar & Hoffman 2013). Max error under 1% of peak, which is
/// far below the uncertainty in any curated ε(λ).
fn cie_xyz_bar(nm: f64) -> (f64, f64, f64) {
    let x = 1.056 * lobe(nm, 599.8, 37.9, 31.0) + 0.362 * lobe(nm, 442.0, 16.0, 26.7)
        - 0.065 * lobe(nm, 501.1, 20.4, 26.2);
    let y = 0.821 * lobe(nm, 568.8, 46.9, 40.5) + 0.286 * lobe(nm, 530.9, 16.3, 31.1);
    let z = 1.217 * lobe(nm, 437.0, 11.8, 36.0) + 0.681 * lobe(nm, 459.0, 26.0, 13.8);
    (x, y, z)
}

/// The colour of light that has passed through a solution.
///
/// `absorbance` is the total A(λ) per band — the sum over every solute of
/// ε(λ)·c·l, which is where mixtures compose correctly.
pub fn transmitted_colour(absorbance: &Spectrum) -> Rgb {
    let mut transmittance = [0.0; BANDS];
    for i in 0..BANDS {
        transmittance[i] = (-absorbance[i] * std::f64::consts::LN_10).exp();
    }
    spectral_colour(&transmittance)
}

/// Convert a sampled reflectance spectrum into display sRGB.
///
/// This shares the observer and white adaptation used for transmitted light,
/// but its samples come from a scattering model rather than Beer–Lambert.
pub fn reflected_colour(reflectance: &Spectrum) -> Rgb {
    spectral_colour(reflectance)
}

fn spectral_colour(fraction: &Spectrum) -> Rgb {
    // Illuminant: equal-energy, the neutral choice for a teaching lab.
    // sRGB's primaries are referenced to D65, though, so the sample is
    // adapted to the D65 white point afterwards — otherwise clear water
    // comes out faintly warm, which is a white-balance artefact rather
    // than chemistry.
    let mut xyz = [0.0f64; 3];
    let mut white = [0.0f64; 3];
    for (i, &nm) in BAND_NM.iter().enumerate() {
        let (xb, yb, zb) = cie_xyz_bar(nm);
        let sample = fraction[i].clamp(0.0, 1.0);
        xyz[0] += sample * xb;
        xyz[1] += sample * yb;
        xyz[2] += sample * zb;
        white[0] += xb;
        white[1] += yb;
        white[2] += zb;
    }
    // von Kries adaptation onto D65, so full transmittance is exactly white.
    const D65: [f64; 3] = [0.95047, 1.0, 1.08883];
    for c in 0..3 {
        if white[c] > 0.0 {
            xyz[c] = xyz[c] / white[c] * D65[c];
        }
    }
    xyz_to_srgb(xyz)
}

/// CIE XYZ (D65-referenced sRGB primaries) to gamma-encoded sRGB.
fn xyz_to_srgb(xyz: [f64; 3]) -> Rgb {
    let (x, y, z) = (xyz[0], xyz[1], xyz[2]);
    let linear = [
        3.2406 * x - 1.5372 * y - 0.4986 * z,
        -0.9689 * x + 1.8758 * y + 0.0415 * z,
        0.0557 * x - 0.2040 * y + 1.0570 * z,
    ];
    let encode = |c: f64| {
        let c = c.clamp(0.0, 1.0);
        let v = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (v * 255.0).round().clamp(0.0, 255.0) as u8
    };
    Rgb {
        r: encode(linear[0]),
        g: encode(linear[1]),
        b: encode(linear[2]),
    }
}

/// Build a spectrum from Gaussian absorption bands: `(centre nm, peak ε,
/// width nm)`. Curating a spectrum by naming its bands is how the
/// literature reports them, so this is the shape the data arrives in.
pub fn bands(peaks: &[(f64, f64, f64)]) -> Spectrum {
    let mut out = [0.0f64; BANDS];
    for (i, &nm) in BAND_NM.iter().enumerate() {
        for &(centre, epsilon, width) in peaks {
            let t = (nm - centre) / width;
            out[i] += epsilon * (-0.5 * t * t).exp();
        }
    }
    out
}

/// A rising or falling edge — the shape of a band whose peak lies outside
/// the visible, like copper's, where only the tail colours the solution.
pub fn edge(at_400: f64, at_700: f64) -> Spectrum {
    let mut out = [0.0f64; BANDS];
    for (i, &nm) in BAND_NM.iter().enumerate() {
        let t = (nm - 405.0) / (705.0 - 405.0);
        // Quadratic rise: absorption in a band's tail climbs faster than
        // linearly toward the peak.
        out[i] = at_400 + (at_700 - at_400) * t * t;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_absorbing_is_white() {
        let c = transmitted_colour(&[0.0; BANDS]);
        assert!(c.r > 250 && c.g > 250 && c.b > 250, "{c:?}");
    }

    #[test]
    fn absorbing_everything_is_black() {
        let c = transmitted_colour(&[5.0; BANDS]);
        assert!(c.r < 5 && c.g < 5 && c.b < 5, "{c:?}");
    }

    #[test]
    fn absorbing_green_gives_magenta() {
        // The complementary-colour rule, which is the whole reason a
        // spectrum beats an RGB value: what you see is what is *left*.
        let spectrum = bands(&[(525.0, 1.0, 40.0)]);
        let mut a = [0.0; BANDS];
        for i in 0..BANDS {
            a[i] = spectrum[i] * 2.0;
        }
        let c = transmitted_colour(&a);
        assert!(
            c.r > c.g && c.b > c.g,
            "absorbing green leaves magenta: {c:?}"
        );
    }

    #[test]
    fn concentration_changes_hue_not_just_depth() {
        // Permanganate: pink when dilute, deep purple when concentrated.
        // One spectrum, two concentrations — the thing an RGB tint cannot
        // reproduce.
        let eps = bands(&[(525.0, 2400.0, 45.0)]);
        let colour_at = |c: f64| {
            let mut a = [0.0; BANDS];
            for i in 0..BANDS {
                a[i] = eps[i] * c * BEAKER_PATH_CM;
            }
            transmitted_colour(&a)
        };
        let dilute = colour_at(2e-5);
        let strong = colour_at(4e-4);
        assert!(
            dilute.r > 180 && dilute.g > 100,
            "dilute reads as a pale pink: {dilute:?}"
        );
        assert!(
            strong.g < dilute.g && strong.r < dilute.r,
            "concentrated is darker and more saturated: {dilute:?} → {strong:?}"
        );
        assert!(
            strong.b > strong.g,
            "and stays on the violet side: {strong:?}"
        );
    }

    #[test]
    fn path_length_matters() {
        let eps = bands(&[(525.0, 2400.0, 45.0)]);
        let at = |l: f64| {
            let mut a = [0.0; BANDS];
            for i in 0..BANDS {
                a[i] = eps[i] * 1e-4 * l;
            }
            transmitted_colour(&a)
        };
        let thin = at(1.0);
        let thick = at(8.0);
        assert!(
            thick.g < thin.g,
            "the same solution is deeper through more liquid: {thin:?} → {thick:?}"
        );
    }

    #[test]
    fn absorbances_add_so_mixtures_compose() {
        // Blue and yellow give green — because the absorbances add, not
        // because we blended two RGB values.
        let yellowish = bands(&[(450.0, 1.0, 40.0)]); // absorbs blue
        let blueish = bands(&[(620.0, 1.0, 40.0)]); // absorbs red
        let mut both = [0.0; BANDS];
        for i in 0..BANDS {
            both[i] = (yellowish[i] + blueish[i]) * 1.5;
        }
        let c = transmitted_colour(&both);
        assert!(
            c.g >= c.r && c.g >= c.b,
            "absorbing red and blue leaves green: {c:?}"
        );
    }
}

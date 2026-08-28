//! Opaque pigment mixing with a bounded Kubelka–Munk model.
//!
//! Paint is not coloured water. In an optically thick, diffusely lit paint
//! layer, absorption (`K`) and scattering (`S`) coefficients mix by amount;
//! the ratio `K/S` then determines reflectance. This first model deliberately
//! assumes a layer thick enough that the substrate no longer affects the
//! result. Thin watercolor washes, gloss, drying and proprietary binders
//! require separate reviewed models.

use crate::spectrum::{reflected_colour, Rgb, Spectrum, BANDS};

/// Curated spectral coefficients for one pigment/binder surrogate.
#[derive(Debug, Clone, PartialEq)]
pub struct PigmentOptics {
    pub key: String,
    pub absorption: Spectrum,
    pub scattering: Spectrum,
}

/// One pigment contribution. `optics: None` keeps missing proprietary data
/// explicit instead of silently falling back to an RGB swatch.
#[derive(Debug, Clone, Copy)]
pub struct PigmentAmount<'a> {
    pub key: &'a str,
    pub amount: f64,
    pub optics: Option<&'a PigmentOptics>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PigmentMixError {
    EmptyMix,
    InvalidAmount { key: String },
    MissingOptics { key: String },
    InvalidCoefficients { key: String },
    NoScattering { wavelength_nm: f64 },
}

/// Mix an optically thick paint layer and return its computed colour.
///
/// Amounts may be any consistent mass or volume unit. Only their ratios
/// matter. The result is order-independent and never blends display RGB.
pub fn opaque_mixture_colour(parts: &[PigmentAmount<'_>]) -> Result<Rgb, PigmentMixError> {
    if parts.is_empty() {
        return Err(PigmentMixError::EmptyMix);
    }

    let mut total = 0.0;
    let mut absorption = [0.0; BANDS];
    let mut scattering = [0.0; BANDS];
    for part in parts {
        if !part.amount.is_finite() || part.amount < 0.0 {
            return Err(PigmentMixError::InvalidAmount {
                key: part.key.to_string(),
            });
        }
        if part.amount == 0.0 {
            continue;
        }
        let optics = part.optics.ok_or_else(|| PigmentMixError::MissingOptics {
            key: part.key.to_string(),
        })?;
        for i in 0..BANDS {
            let k = optics.absorption[i];
            let s = optics.scattering[i];
            if !k.is_finite() || k < 0.0 || !s.is_finite() || s < 0.0 {
                return Err(PigmentMixError::InvalidCoefficients {
                    key: optics.key.clone(),
                });
            }
            absorption[i] += part.amount * k;
            scattering[i] += part.amount * s;
        }
        total += part.amount;
    }
    if total <= 0.0 {
        return Err(PigmentMixError::EmptyMix);
    }

    let mut reflectance = [0.0; BANDS];
    for i in 0..BANDS {
        if scattering[i] <= 0.0 {
            return Err(PigmentMixError::NoScattering {
                wavelength_nm: crate::spectrum::BAND_NM[i],
            });
        }
        let ratio = absorption[i] / scattering[i];
        let a = 1.0 + ratio;
        reflectance[i] = a - (a * a - 1.0).sqrt();
    }
    Ok(reflected_colour(&reflectance))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spectrum::bands;

    fn pigment(key: &str, absorption: Spectrum, scattering: f64) -> PigmentOptics {
        PigmentOptics {
            key: key.to_string(),
            absorption,
            scattering: [scattering; BANDS],
        }
    }

    #[test]
    fn white_and_black_bound_the_model() {
        let white = pigment("white", [0.0; BANDS], 1.0);
        let black = pigment("black", [10.0; BANDS], 0.1);
        let mixed = |p: &PigmentOptics| {
            opaque_mixture_colour(&[PigmentAmount {
                key: &p.key,
                amount: 1.0,
                optics: Some(p),
            }])
            .unwrap()
        };
        let w = mixed(&white);
        let b = mixed(&black);
        assert!(w.r > 250 && w.g > 250 && w.b > 250, "{w:?}");
        assert!(b.r < 25 && b.g < 25 && b.b < 25, "{b:?}");
    }

    #[test]
    fn subtractive_blue_and_yellow_leave_green() {
        let yellow = pigment("yellow", bands(&[(440.0, 4.0, 45.0)]), 1.0);
        let blue = pigment("blue", bands(&[(650.0, 4.0, 65.0)]), 1.0);
        let colour = opaque_mixture_colour(&[
            PigmentAmount {
                key: "yellow",
                amount: 1.0,
                optics: Some(&yellow),
            },
            PigmentAmount {
                key: "blue",
                amount: 1.0,
                optics: Some(&blue),
            },
        ])
        .unwrap();
        assert!(colour.g > colour.r && colour.g > colour.b, "{colour:?}");
    }

    #[test]
    fn mixing_is_order_independent() {
        let red = pigment("red", bands(&[(500.0, 3.0, 70.0)]), 1.0);
        let white = pigment("white", [0.0; BANDS], 2.0);
        let a = opaque_mixture_colour(&[
            PigmentAmount {
                key: "red",
                amount: 1.0,
                optics: Some(&red),
            },
            PigmentAmount {
                key: "white",
                amount: 3.0,
                optics: Some(&white),
            },
        ]);
        let b = opaque_mixture_colour(&[
            PigmentAmount {
                key: "white",
                amount: 3.0,
                optics: Some(&white),
            },
            PigmentAmount {
                key: "red",
                amount: 1.0,
                optics: Some(&red),
            },
        ]);
        assert_eq!(a, b);
    }

    #[test]
    fn missing_optics_never_falls_back_to_rgb() {
        assert_eq!(
            opaque_mixture_colour(&[PigmentAmount {
                key: "mystery_acrylic",
                amount: 1.0,
                optics: None,
            }]),
            Err(PigmentMixError::MissingOptics {
                key: "mystery_acrylic".to_string(),
            })
        );
    }
}

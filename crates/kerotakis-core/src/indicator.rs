//! Acid–base indicators: colour as a computed consequence of pH.
//!
//! An indicator is not a dye that "turns pink at the endpoint". It is a
//! weak acid whose two forms happen to absorb differently, and everything
//! a titration does with it follows from that one fact. `HIn ⇌ H⁺ + In⁻`
//! has a pKa like any other weak acid, so the ratio of the forms is
//! Henderson–Hasselbalch, and the colour is their two spectra mixed in
//! that ratio. The famous "range" — phenolphthalein 8.3 to 10.0 — is not a
//! property anyone measured separately; it is where that ratio passes
//! through the region the eye can tell apart, roughly pKa ± 1.
//!
//! Computing it rather than tabulating it earns three things a lookup
//! cannot. The transition is *gradual*, because the ratio is. The endpoint
//! sits where the chemistry puts it rather than where the dye does — which
//! is why phenolphthalein is right for a weak acid titrated with a strong
//! base, whose equivalence point is at pH 8.8 and not at 7, and why methyl
//! orange would report that same titration as finished long before it is.
//! And the colour comes out of the same Beer–Lambert pipeline as every
//! other solute, so an indicator in a coloured solution composes with it
//! instead of overwriting it.
//!
//! **What is curated, and how firmly.** The pKa values and the absorption
//! maxima are literature quantities and are quoted as such. The peak molar
//! absorptivities are good to about a significant figure. The *band shape*
//! is ours: a Gaussian of stated width at the stated maximum, which is an
//! idealisation of a real vibronic envelope, not a measurement of one. It
//! reproduces the hue and the concentration at which the colour becomes
//! visible, and it does not claim to reproduce a spectrophotometer trace.

use crate::spectrum::{bands, Spectrum};

/// A weak acid whose conjugate forms differ in colour.
pub struct Indicator {
    /// Registry key of the species this describes.
    pub key: &'static str,
    /// −log₁₀ Ka for `HIn ⇌ H⁺ + In⁻`.
    pub pka: f64,
    /// ε(λ) of the acid form, which dominates below the pKa.
    pub acid: fn() -> Spectrum,
    /// ε(λ) of the base form, which dominates above it.
    pub base: fn() -> Spectrum,
    /// What a chemist calls each form, for the words the bench prints.
    pub acid_colour: &'static str,
    pub base_colour: &'static str,
    pub provenance: &'static str,
}

impl Indicator {
    /// The fraction in the base form at this pH — Henderson–Hasselbalch,
    /// which is all an indicator's "range" ever was.
    pub fn base_fraction(&self, ph: f64) -> f64 {
        crate::relations::henderson_hasselbalch_fraction(self.pka, ph)
    }

    /// The two spectra mixed in the ratio the pH sets.
    pub fn spectrum_at(&self, ph: f64) -> Spectrum {
        let f = self.base_fraction(ph);
        let (acid, base) = ((self.acid)(), (self.base)());
        let mut out = [0.0; crate::spectrum::BANDS];
        for (i, band) in out.iter_mut().enumerate() {
            *band = (1.0 - f) * acid[i] + f * base[i];
        }
        out
    }

    /// Where the eye can first tell, and where it has finished telling:
    /// the classical "transition range", derived rather than tabulated.
    ///
    /// Ten-to-one either way is the usual convention for the point at which
    /// one form visually dominates, which is what puts the range at about
    /// pKa ± 1.
    pub fn transition_range(&self) -> (f64, f64) {
        (self.pka - 1.0, self.pka + 1.0)
    }
}

pub const INDICATORS: &[Indicator] = &[
    Indicator {
        key: "phenolphthalein",
        // Transition 8.3–10.0 in every textbook, which is this pKa ± 1.
        pka: 9.4,
        // Colourless: the lactone form has no visible absorption at all,
        // which is why the endpoint is a colour appearing out of nothing
        // rather than one colour becoming another.
        acid: || bands(&[]),
        // The quinoid dianion, λmax 553 nm, ε ≈ 2.0e4 — an intense band,
        // which is why a drop of it colours a whole flask.
        base: || bands(&[(553.0, 20_000.0, 45.0)]),
        acid_colour: "colourless",
        base_colour: "magenta",
        provenance: "pKa 9.4 and λmax 553 nm are literature values; ε ≈ 2.0e4 L/mol/cm good to about a significant figure; Gaussian band shape is ours",
    },
    Indicator {
        key: "methyl_orange",
        // Transition 3.1–4.4, centred a little below this pKa.
        pka: 3.47,
        // Protonated azonium form, red, λmax ≈ 505 nm.
        acid: || bands(&[(505.0, 24_000.0, 50.0)]),
        // Yellow azo form, λmax ≈ 464 nm.
        base: || bands(&[(464.0, 26_000.0, 55.0)]),
        acid_colour: "red",
        base_colour: "yellow",
        provenance: "pKa 3.47 and λmax 505/464 nm are literature values; ε values good to about a significant figure; Gaussian band shapes are ours",
    },
    Indicator {
        key: "bromothymol_blue",
        // Transition 6.0–7.6: the one indicator that actually straddles
        // neutral, which is why it is the schools' pond-water dye.
        pka: 7.1,
        acid: || bands(&[(430.0, 14_000.0, 50.0)]),
        base: || bands(&[(615.0, 35_000.0, 55.0)]),
        acid_colour: "yellow",
        base_colour: "blue",
        provenance: "pKa 7.1 and λmax 430/615 nm are literature values; ε values good to about a significant figure; Gaussian band shapes are ours",
    },
];

pub fn lookup(key: &str) -> Option<&'static Indicator> {
    INDICATORS.iter().find(|i| i.key == key)
}

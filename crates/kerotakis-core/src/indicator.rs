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

/// KID-8: a pigment whose colour passes through more than two forms.
///
/// An `Indicator` above is one weak acid with two coloured forms, and that
/// is the whole story for phenolphthalein. Red cabbage is not that story.
/// Its anthocyanins lose protons in steps, and each step has its own
/// colour, which is why the classroom rainbow runs red → purple → blue →
/// green → yellow across the range rather than switching once. Two forms
/// cannot produce five colours, and tabulating five would throw away the
/// reason the bench computes colour at all.
///
/// So the ladder is the same Henderson–Hasselbalch idea generalised the way
/// a polyprotic acid already is elsewhere in this engine: `n` successive
/// pKa values give `n + 1` forms, the fraction in each is the standard
/// stepwise distribution, and the spectrum is every form's ε(λ) mixed in
/// those fractions. The green nobody put in the table falls out of it — a
/// blue form and a yellow form present together absorb at both ends of the
/// visible range and leave a window in the middle.
/// One rung: a form's ε(λ) and the word a chemist uses for it.
pub struct PigmentForm {
    pub spectrum: fn() -> Spectrum,
    pub colour: &'static str,
}

pub struct PigmentLadder {
    /// Registry key of the species this describes.
    pub key: &'static str,
    /// Successive pKa values, low to high. `n` of them mean `n + 1` forms.
    pub pkas: &'static [f64],
    /// One entry per form, most protonated first.
    pub forms: &'static [PigmentForm],
    pub provenance: &'static str,
}

impl PigmentLadder {
    /// The fraction in each form at this pH.
    ///
    /// For stepwise dissociations the un-normalised weight of the form that
    /// has lost `k` protons is `10^(k·pH − Σ pKa₁..pKa_k)`. Those exponents
    /// reach ±40 over the pH range the bench allows, so the maximum is
    /// subtracted before exponentiating — otherwise a pH of 14 overflows a
    /// double and every fraction comes back NaN.
    pub fn fractions(&self, ph: f64) -> Vec<f64> {
        let mut exponents = Vec::with_capacity(self.forms.len());
        let mut cumulative = 0.0;
        for k in 0..self.forms.len() {
            if k > 0 {
                cumulative += self.pkas[k - 1];
            }
            exponents.push(k as f64 * ph - cumulative);
        }
        let peak = exponents.iter().copied().fold(f64::MIN, f64::max);
        let weights: Vec<f64> = exponents
            .iter()
            .map(|e| 10f64.powf((e - peak).max(-300.0)))
            .collect();
        let total: f64 = weights.iter().sum();
        if total <= 0.0 || !total.is_finite() {
            // No answer is better than a fabricated one: hand back the most
            // protonated form rather than a vector of NaN.
            let mut out = vec![0.0; self.forms.len()];
            out[0] = 1.0;
            return out;
        }
        weights.into_iter().map(|w| w / total).collect()
    }

    /// Every form's spectrum, mixed in the fractions the pH sets.
    pub fn spectrum_at(&self, ph: f64) -> Spectrum {
        let fractions = self.fractions(ph);
        let mut out = [0.0; crate::spectrum::BANDS];
        for (fraction, form) in fractions.iter().zip(self.forms) {
            let eps = (form.spectrum)();
            for (band, e) in out.iter_mut().zip(eps.iter()) {
                *band += fraction * e;
            }
        }
        out
    }

    /// The word for whichever form dominates here — for prose, never for
    /// the colour itself, which stays the computed spectrum.
    pub fn dominant_form(&self, ph: f64) -> &'static str {
        let fractions = self.fractions(ph);
        fractions
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(index, _)| self.forms[index].colour)
            .unwrap_or("")
    }
}

pub const PIGMENT_LADDERS: &[PigmentLadder] = &[PigmentLadder {
    key: "anthocyanin",
    // Cyanidin glycosides lose protons in three steps over the range a
    // kitchen can reach. The values are the middle of the ranges the
    // anthocyanin literature reports for cyanidin-3-glucoside; red cabbage
    // is a mixture of acylated cyanidin glycosides whose exact pKa values
    // differ from any single one of them, and that is stated rather than
    // hidden behind a decimal place.
    pkas: &[4.0, 7.0, 11.0],
    forms: &[
        // Flavylium cation: the red of vinegar-side cabbage water.
        PigmentForm { spectrum: || bands(&[(520.0, 26_000.0, 55.0)]), colour: "red" },
        // Quinoidal base: purple, and the reason untreated cabbage water is
        // the colour it is.
        PigmentForm { spectrum: || bands(&[(555.0, 18_000.0, 65.0)]), colour: "purple" },
        // Anionic quinoidal base: blue.
        PigmentForm { spectrum: || bands(&[(600.0, 16_000.0, 70.0)]), colour: "blue" },
        // Chalcone and its relatives: yellow. Where this form overlaps the
        // blue one the mixture absorbs at both ends and looks green, which
        // is the colour no table in this file contains.
        PigmentForm { spectrum: || bands(&[(395.0, 9_000.0, 40.0)]), colour: "yellow" },
    ],
    provenance: "Red-cabbage anthocyanin as a four-form ladder. Stepwise pKa 4.0 / 7.0 / 11.0 sit inside the ranges the anthocyanin literature reports for cyanidin-3-glucoside (flavylium/quinoidal near 4, the anionic quinoidal near 7, chalcone formation above 10). Peak ε ≈ 2.6e4 L/mol/cm for the flavylium form is the standard cyanidin-3-glucoside figure and is good to about a significant figure; the ε values of the other three forms are explicitly editorial (Kerotakis), chosen so each colour becomes visible at the concentration a jar of cabbage water actually is. Gaussian band shapes are ours, as they are for every indicator here. Red cabbage is a mixture of acylated cyanidin glycosides rather than one compound, so no InChIKey is asserted and no claim is made about a particular cultivar, extraction or the copigmentation that shifts these bands in real juice"
}];

pub fn lookup(key: &str) -> Option<&'static Indicator> {
    INDICATORS.iter().find(|i| i.key == key)
}

pub fn lookup_ladder(key: &str) -> Option<&'static PigmentLadder> {
    PIGMENT_LADDERS.iter().find(|p| p.key == key)
}

/// Does this species' colour depend on the pH of the solution it is in?
pub fn is_ph_dependent(key: &str) -> bool {
    lookup(key).is_some() || lookup_ladder(key).is_some()
}

/// The ε(λ) of a pH-dependent colourant at this pH, whichever table it
/// lives in. `None` means this species' colour does not depend on pH.
pub fn spectrum_at_ph(key: &str, ph: f64) -> Option<Spectrum> {
    if let Some(indicator) = lookup(key) {
        return Some(indicator.spectrum_at(ph));
    }
    lookup_ladder(key).map(|ladder| ladder.spectrum_at(ph))
}

#[cfg(test)]
mod ladder_tests {
    use super::*;

    /// KID-8: the fractions are a probability distribution at every pH the
    /// bench allows, including the ends where the exponents overflow a
    /// double if they are not normalised first.
    #[test]
    fn the_ladder_fractions_sum_to_one_across_the_whole_range() {
        for ladder in PIGMENT_LADDERS {
            assert_eq!(
                ladder.forms.len(),
                ladder.pkas.len() + 1,
                "{}: n pKa values describe n+1 forms",
                ladder.key
            );
            for step in -20..=340 {
                let ph = f64::from(step) / 20.0;
                let fractions = ladder.fractions(ph);
                let total: f64 = fractions.iter().sum();
                assert!(
                    (total - 1.0).abs() < 1e-9,
                    "{} at pH {ph}: fractions sum to {total}",
                    ladder.key
                );
                assert!(
                    fractions.iter().all(|f| f.is_finite() && *f >= 0.0),
                    "{} at pH {ph}: {fractions:?}",
                    ladder.key
                );
            }
        }
    }

    /// Each form must actually be the majority somewhere, or it is a
    /// spectrum the bench carries and can never show.
    #[test]
    fn every_form_on_a_ladder_dominates_somewhere() {
        for ladder in PIGMENT_LADDERS {
            for (index, form) in ladder.forms.iter().enumerate() {
                let name = form.colour;
                let reached = (-20..=340).any(|step| {
                    let fractions = ladder.fractions(f64::from(step) / 20.0);
                    fractions
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .is_some_and(|(top, _)| top == index)
                });
                assert!(reached, "{}: the {name} form is never dominant", ladder.key);
            }
        }
    }

    /// A two-form ladder must reduce to Henderson–Hasselbalch, because that
    /// is what the generalisation claims to generalise.
    #[test]
    fn a_two_form_ladder_agrees_with_henderson_hasselbalch() {
        let ladder = PigmentLadder {
            key: "test",
            pkas: &[7.1],
            forms: &[
                PigmentForm {
                    spectrum: || bands(&[(430.0, 14_000.0, 50.0)]),
                    colour: "yellow",
                },
                PigmentForm {
                    spectrum: || bands(&[(615.0, 35_000.0, 55.0)]),
                    colour: "blue",
                },
            ],
            provenance: "test",
        };
        for step in 0..=140 {
            let ph = f64::from(step) / 10.0;
            let expected = crate::relations::henderson_hasselbalch_fraction(7.1, ph);
            assert!(
                (ladder.fractions(ph)[1] - expected).abs() < 1e-12,
                "pH {ph}: {} vs {expected}",
                ladder.fractions(ph)[1]
            );
        }
    }
}

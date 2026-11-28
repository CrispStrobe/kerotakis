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
    /// Successive −log₁₀ Ka values, low to high. The last one is the
    /// colour-bearing step `HIn ⇌ H⁺ + In⁻`; any before it lead from the
    /// fully protonated dye to the form that loses the last proton, and
    /// every form except the fully deprotonated one wears the `acid`
    /// spectrum. Phenolphthalein needs two: its pink dianion is two protons
    /// away from the colourless lactone, which is why it stays colourless
    /// at neutral pH even at the concentrations a bench overdoses it to.
    pub pkas: &'static [f64],
    /// The visual transition range as printed in the reference tables
    /// (CRC / reviewed literature) — the pH window over which the eye sees
    /// the colour change. Sourced, not derived; see `provenance`.
    pub transition: (f64, f64),
    /// ε(λ) of the acid form(s), which dominate below the colour pKa.
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
    ///
    /// With more than one dissociation step this is the stepwise
    /// distribution's last form, normalised against overflow the same way
    /// [`PigmentLadder::fractions`] is.
    pub fn base_fraction(&self, ph: f64) -> f64 {
        if let [pka] = self.pkas {
            return crate::relations::henderson_hasselbalch_fraction(*pka, ph);
        }
        stepwise_fractions(self.pkas, ph)
            .last()
            .copied()
            .unwrap_or(0.0)
    }

    /// The pKa of the colour-bearing step.
    pub fn colour_pka(&self) -> f64 {
        self.pkas.last().copied().unwrap_or(f64::NAN)
    }

    /// The pH at which a titration "until the colour persists" is read as
    /// finished with this indicator.
    ///
    /// A two-colour indicator (bromothymol blue, methyl orange) is read at
    /// its mixed colour, which is its pKIn — the textbook statement that
    /// an indicator's end point is at pH ≈ pKIn. A one-colour indicator
    /// (phenolphthalein) has no mixed colour: the eye reads the presence or
    /// absence of the coloured form, and that threshold is the edge of the
    /// sourced transition range on the coloured side's far end — the first
    /// persistent pink going up, the last of the pink going down, both at
    /// the lower edge.
    pub fn endpoint_ph(&self) -> f64 {
        let acid_colourless = (self.acid)().iter().all(|e| *e == 0.0);
        let base_colourless = (self.base)().iter().all(|e| *e == 0.0);
        match (acid_colourless, base_colourless) {
            (true, false) => self.transition.0,
            (false, true) => self.transition.1,
            _ => self.colour_pka(),
        }
    }

    /// The sourced word for the form a two-colour indicator shows at this
    /// pH: `acid_colour` below its sourced visual transition range,
    /// `base_colour` above it, and `None` inside the range (the mixed
    /// colour) and for a one-colour indicator such as phenolphthalein,
    /// whose word is the computed colour's.
    ///
    /// The Beer–Lambert colour stays the computed spectrum; this only names
    /// it. A Gaussian band of either form, at the concentrations a bench
    /// overdoses a dye to, absorbs far enough into its neighbours that the
    /// hue arithmetic drifts (bromothymol blue's yellow read "orange", its
    /// blue "purple", methyl orange's yellow "red" — 2026-11-28 sweep, S5);
    /// the colour a chemist reports for each form and the range it is seen
    /// outside of are table entries with a source, and the table says it.
    pub fn form_word(&self, ph: f64) -> Option<&'static str> {
        let acid_colourless = (self.acid)().iter().all(|e| *e == 0.0);
        let base_colourless = (self.base)().iter().all(|e| *e == 0.0);
        if acid_colourless || base_colourless || !ph.is_finite() {
            return None;
        }
        let (lo, hi) = self.transition;
        if ph < lo {
            Some(self.acid_colour)
        } else if ph > hi {
            Some(self.base_colour)
        } else {
            None
        }
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
        let pka = self.colour_pka();
        (pka - 1.0, pka + 1.0)
    }
}

pub const INDICATORS: &[Indicator] = &[
    Indicator {
        key: "phenolphthalein",
        // Lactone H2In ⇌ HIn⁻ (colourless) pKa 9.05, HIn⁻ ⇌ In²⁻ (pink)
        // pKa 9.50: Z. Tamura et al., Anal. Sci. 12 (1996) 927–930,
        // doi:10.2116/analsci.12.927. Two protons between colourless and
        // pink is why the pink fraction falls a hundredfold per pH unit
        // below the range, not tenfold.
        pkas: &[9.05, 9.50],
        // Visual range 8.3–10.0 (colourless → pink): Rathod, Murthy &
        // Bandyopadhyay, J. Chem. Educ. 96 (2019) 486–494; CRC Handbook,
        // "Acid-Base Indicators".
        transition: (8.3, 10.0),
        // Colourless: the lactone form has no visible absorption at all,
        // which is why the endpoint is a colour appearing out of nothing
        // rather than one colour becoming another.
        acid: || bands(&[]),
        // The quinoid dianion, λmax 553 nm, ε ≈ 2.0e4 — an intense band,
        // which is why a drop of it colours a whole flask.
        base: || bands(&[(553.0, 20_000.0, 45.0)]),
        acid_colour: "colourless",
        base_colour: "magenta",
        provenance: "stepwise pKa 9.05 / 9.50 from Tamura et al., Anal. Sci. 12 (1996) 927, doi:10.2116/analsci.12.927; visual range 8.3–10.0 from Rathod et al., J. Chem. Educ. 96 (2019) 486 and the CRC Handbook indicator table; λmax 553 nm are literature values; ε ≈ 2.0e4 L/mol/cm good to about a significant figure; Gaussian band shape is ours",
    },
    Indicator {
        key: "methyl_orange",
        // Transition 3.1–4.4 (red → yellow), CRC Handbook "Acid-Base
        // Indicators"; pKIn 3.47 (Sandberg et al., J. Phys. Chem. 76
        // (1972) 4023, doi:10.1021/j100670a024).
        pkas: &[3.47],
        transition: (3.1, 4.4),
        // Protonated azonium form, red, λmax ≈ 505 nm.
        acid: || bands(&[(505.0, 24_000.0, 50.0)]),
        // Yellow azo form, λmax ≈ 464 nm.
        base: || bands(&[(464.0, 26_000.0, 55.0)]),
        acid_colour: "red",
        base_colour: "yellow",
        provenance: "pKa 3.47 (water, 25 °C): R. G. Sandberg, G. H. Henderson, R. D. White and E. M. Eyring, J. Phys. Chem. 76 (1972) 4023, doi:10.1021/j100670a024, as quoted by Wikipedia 'Methyl orange' (read 2026-11-28); transition 3.1–4.4 red → yellow from the CRC Handbook indicator table; λmax 505/464 nm are literature values; ε values good to about a significant figure; Gaussian band shapes are ours",
    },
    Indicator {
        key: "bromothymol_blue",
        // Transition 6.0–7.6: the one indicator that actually straddles
        // neutral, which is why it is the schools' pond-water dye.
        // Transition 6.0–7.6 (yellow → blue), CRC Handbook "Acid-Base
        // Indicators"; pKIn 7.1.
        pkas: &[7.1],
        transition: (6.0, 7.6),
        acid: || bands(&[(430.0, 14_000.0, 50.0)]),
        base: || bands(&[(615.0, 35_000.0, 55.0)]),
        acid_colour: "yellow",
        base_colour: "blue",
        provenance: "pKa 7.1 and λmax 430/615 nm are literature values; ε values good to about a significant figure; transition ranges are the CRC Handbook indicator table's; Gaussian band shapes are ours",
    },
    Indicator {
        key: "litmus",
        // Litmus (azolitmin, its principal chromophore): visual transition
        // pH 4.5–8.3, red → blue, as handbook indicator tables quote it
        // (CRC Handbook / Lange's "Acid-Base Indicators"); the US
        // Pharmacopeia's reagent monograph gives "approximately pH 4.5 to 8,
        // red to blue" (USP 35–NF 30, Indicators and Indicator Test Papers).
        // Litmus is a mixture of lichen dyes, so no single pKa is published:
        // the colour pKIn here is the centre of that sourced range, 6.4,
        // which is derived, not measured.
        pkas: &[6.4],
        transition: (4.5, 8.3),
        // EDITORIAL band positions: no reviewed azolitmin spectrum was
        // found. The acid form is red (absorbs blue-green), the base form
        // blue (absorbs orange); the bands are placed where those
        // complementary colours require and are not a measurement.
        acid: || bands(&[(500.0, 15_000.0, 55.0)]),
        base: || bands(&[(590.0, 15_000.0, 60.0)]),
        acid_colour: "red",
        base_colour: "blue",
        provenance: "visual transition pH 4.5–8.3 (red → blue) as quoted in handbook indicator tables (CRC Handbook / Lange's Handbook 'Acid-Base Indicators'), consistent with USP 35–NF 30 'Indicators and Indicator Test Papers' (approximately pH 4.5 to 8, red to blue); pKIn 6.4 is the centre of that range, derived because litmus is a mixture with no single published pKa; band positions and ε are EDITORIAL (Kerotakis), chosen only to give the sourced red and blue, and are not a measured spectrum; Gaussian band shapes are ours",
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
        stepwise_fractions(self.pkas, ph)
    }
}

/// The fraction in each of the `pkas.len() + 1` forms of a stepwise acid.
fn stepwise_fractions(pkas: &[f64], ph: f64) -> Vec<f64> {
    let forms = pkas.len() + 1;
    let mut exponents = Vec::with_capacity(forms);
    let mut cumulative = 0.0;
    for k in 0..forms {
        if k > 0 {
            cumulative += pkas[k - 1];
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
        let mut out = vec![0.0; forms];
        out[0] = 1.0;
        return out;
    }
    weights.into_iter().map(|w| w / total).collect()
}

impl PigmentLadder {
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

#[cfg(test)]
mod indicator_tests {
    use super::*;

    #[test]
    fn phenolphthalein_pink_falls_a_hundredfold_per_ph_unit_below_its_range() {
        let p = lookup("phenolphthalein").expect("phenolphthalein");
        assert!(p.base_fraction(7.0) < 1e-4, "{}", p.base_fraction(7.0));
        assert!(p.base_fraction(11.0) > 0.9);
        let ratio = p.base_fraction(7.0) / p.base_fraction(6.0);
        assert!((ratio - 100.0).abs() < 1.0, "{ratio}");
    }

    /// S5 (2026-11-28 sweep): every two-colour indicator names its sourced
    /// acid colour at pH 3 below its range and its base colour at pH 11
    /// above it, and names neither inside its mixed band.
    #[test]
    fn two_colour_indicators_name_their_sourced_forms() {
        let mo = lookup("methyl_orange").unwrap();
        assert_eq!(mo.form_word(3.0), Some("red"));
        assert_eq!(mo.form_word(7.0), Some("yellow"));
        assert_eq!(mo.form_word(11.0), Some("yellow"));
        assert_eq!(mo.form_word(3.47), None);
        let btb = lookup("bromothymol_blue").unwrap();
        assert_eq!(btb.form_word(3.0), Some("yellow"));
        assert_eq!(btb.form_word(7.0), None);
        assert_eq!(btb.form_word(5.0), Some("yellow"));
        assert_eq!(btb.form_word(11.0), Some("blue"));
        assert_eq!(lookup("phenolphthalein").unwrap().form_word(11.0), None);
        for i in INDICATORS {
            let (lo, hi) = i.transition;
            if let Some(word) = i.form_word(lo - 0.5) {
                assert_eq!(word, i.acid_colour, "{}", i.key);
            }
            if let Some(word) = i.form_word(hi + 0.5) {
                assert_eq!(word, i.base_colour, "{}", i.key);
            }
        }
    }

    #[test]
    fn endpoints_follow_the_sourced_ranges() {
        let p = lookup("phenolphthalein").expect("phenolphthalein");
        assert_eq!(p.endpoint_ph(), 8.3);
        assert_eq!(lookup("bromothymol_blue").expect("btb").endpoint_ph(), 7.1);
        assert_eq!(lookup("methyl_orange").expect("mo").endpoint_ph(), 3.47);
        let litmus = lookup("litmus").expect("litmus");
        assert_eq!(litmus.transition, (4.5, 8.3));
        assert_eq!(litmus.endpoint_ph(), 6.4);
        for i in INDICATORS {
            let (lo, hi) = i.transition;
            assert!(lo < hi, "{}", i.key);
            assert!(
                (lo - 0.5..=hi + 0.5).contains(&i.colour_pka()),
                "{}: pKIn {} outside {lo}–{hi}",
                i.key,
                i.colour_pka()
            );
        }
    }
}

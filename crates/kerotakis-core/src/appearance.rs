//! What a vessel looks like.
//!
//! The first thing a nine-year-old does is *look*, not measure — so this
//! turns state into what the eye would report: a colour, how cloudy it is,
//! what has settled, and whether anything is bubbling.
//!
//! Colour is computed, not tinted. Each dissolved species contributes its
//! molar absorptivity spectrum ε(λ); the absorbances add, Beer–Lambert
//! turns the total into a transmittance, and `crate::spectrum` integrates
//! what is left against the CIE 1931 observer. So mixtures compose the way
//! a beaker composes them, concentration changes hue rather than only
//! depth, and the depth of liquid the light crosses is a parameter.
//!
//! Two things stay curated, for reasons rather than convenience. ε(λ)
//! itself must: the ions that are actually coloured owe it to d–d and
//! charge-transfer transitions, and no dataset we can ship — nor any
//! affordable calculation, TD-DFT least of all here — delivers those
//! honestly. And the colour of a *solid* stays a plain sRGB value, because
//! a white powder is white by scattering, not by transmission; running it
//! through Beer–Lambert would be the wrong physics wearing the right
//! equations. Turbidity comes from suspended solid volume.

use serde::{Deserialize, Serialize};

use crate::species::{self, Colour, Phase};
use crate::vessel::Vessel;

/// What the eye reports.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Appearance {
    /// The liquid's colour, sRGB.
    pub liquid: Option<Colour>,
    /// 0 = clear, 1 = opaque. Suspended solid, before it settles.
    pub cloudiness: f64,
    /// Solid sitting at the bottom, with its colour.
    pub deposit: Option<(String, Colour)>,
    /// Whether anything is visibly not-liquid-not-settled: bubbles.
    pub bubbling: bool,
    /// A plain-words summary, which is what a young reader actually wants.
    pub words: String,
}

/// How much suspended solid counts as fully opaque, in moles per litre of
/// liquid. A precipitate is visible long before it is thick.
const OPAQUE_AT: f64 = 0.05;

pub fn observe(vessel: &Vessel) -> Appearance {
    let litres = vessel.liquid_volume().0.max(1e-9);

    // --- Colour: Beer–Lambert over the solutes' absorption spectra.
    //
    // Absorbances add, so mixtures compose correctly; concentration and
    // path length enter the same way they do in a real beaker. A species
    // with no curated spectrum contributes nothing rather than being
    // guessed at.
    let mut absorbance = [0.0f64; crate::spectrum::BANDS];
    for p in &vessel.contents {
        if !matches!(p.phase, Phase::Aqueous | Phase::Liquid) {
            continue;
        }
        let Some(spectrum) = species::lookup(&p.species).and_then(|d| d.spectrum) else {
            continue;
        };
        let concentration = p.moles.0 / litres;
        let eps = spectrum();
        for (band, e) in absorbance.iter_mut().zip(eps.iter()) {
            *band += e * concentration * crate::spectrum::BEAKER_PATH_CM;
        }
    }
    let has_liquid = vessel
        .contents
        .iter()
        .any(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous));
    let liquid = has_liquid.then(|| {
        let rgb = crate::spectrum::transmitted_colour(&absorbance);
        Colour {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            strength: 0.0,
        }
    });

    // --- Cloudiness and deposit from suspended solid.
    let mut solid_moles = 0.0;
    let mut biggest: Option<(&str, f64, Colour)> = None;
    for p in &vessel.contents {
        if p.phase != Phase::Solid {
            continue;
        }
        solid_moles += p.moles.0;
        let data = species::lookup(&p.species);
        let colour = data.and_then(|d| d.colour).unwrap_or(Colour {
            r: 220,
            g: 220,
            b: 220,
            strength: 0.0,
        });
        let name = data.map(|d| d.name).unwrap_or(p.species.0.as_str());
        if biggest.as_ref().is_none_or(|(_, m, _)| p.moles.0 > *m) {
            biggest = Some((name, p.moles.0, colour));
        }
    }
    let cloudiness = if has_liquid {
        (solid_moles / litres / OPAQUE_AT).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let deposit = biggest.map(|(name, _, colour)| (name.to_string(), colour));

    // Gas in a vessel that also holds liquid is gas coming *out* of the
    // liquid, which is the single most visible thing in a school kinetics
    // practical. This field existed and was hardcoded `false`, so a flask
    // holding 0.05 mol of oxygen was described as "colourless and clear".
    let bubbling = has_liquid
        && vessel
            .contents
            .iter()
            .any(|p| p.phase == Phase::Gas && p.moles.0 >= crate::OBSERVABLE_MOLES);

    let words = describe(&liquid, cloudiness, &deposit, has_liquid, bubbling, vessel);
    Appearance {
        liquid,
        cloudiness,
        deposit,
        bubbling,
        words,
    }
}

/// Plain words for a colour — a child says "blue", not "#286ED2".
///
/// `solid` picks the right word for an uncoloured thing: a powder is
/// *white*, a liquid is *colourless*, and calling either by the other's
/// name is the sort of small wrongness a child notices immediately.
fn colour_word(c: &Colour, solid: bool) -> &'static str {
    let (r, g, b) = (c.r as f64, c.g as f64, c.b as f64);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    if chroma < 12.0 {
        return match (max, solid) {
            (m, true) if m > 200.0 => "white",
            (m, false) if m > 200.0 => "colourless",
            (m, _) if m > 120.0 => "grey",
            _ => "black",
        };
    }
    // Hue angle, the standard way round the colour wheel.
    let hue = if max == r {
        60.0 * (((g - b) / chroma) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / chroma + 2.0)
    } else {
        60.0 * ((r - g) / chroma + 4.0)
    };
    let hue = if hue < 0.0 { hue + 360.0 } else { hue };
    let dark = max < 120.0;
    // Saturation, which is what separates pink from purple: they are the
    // same hue. Dilute permanganate and concentrated permanganate are one
    // spectrum at two concentrations, and a chemist calls the first pink
    // and the second purple — so the word has to follow the washing-out,
    // not the hue angle, or the whole point of computing the colour from
    // ε(λ) is thrown away at the last step.
    let saturation = chroma / max.max(1.0);
    match hue {
        h if !(15.0..330.0).contains(&h) => {
            if saturation < 0.7 {
                "pink"
            } else {
                "red"
            }
        }
        h if h < 45.0 => "orange",
        h if h < 70.0 => "yellow",
        h if h < 160.0 => "green",
        h if h < 200.0 => "blue-green",
        h if h < 250.0 => "blue",
        _ if saturation < 0.7 => "pink",
        _ if dark => "deep purple",
        _ => "purple",
    }
}

fn describe(
    liquid: &Option<Colour>,
    cloudiness: f64,
    deposit: &Option<(String, Colour)>,
    has_liquid: bool,
    bubbling: bool,
    vessel: &Vessel,
) -> String {
    if vessel.is_empty() {
        return "The beaker is empty.".to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    if has_liquid {
        let word = liquid
            .as_ref()
            .map(|c| colour_word(c, false))
            .unwrap_or("colourless");
        let clarity = if cloudiness > 0.6 {
            "and so cloudy you cannot see through it"
        } else if cloudiness > 0.15 {
            "and cloudy"
        } else if cloudiness > 0.01 {
            "and very slightly hazy"
        } else {
            "and clear"
        };
        parts.push(if word == "colourless" {
            format!("The liquid is colourless {clarity}")
        } else {
            format!("The liquid is {word} {clarity}")
        });
    }
    if bubbling {
        parts.push("bubbles of gas are rising through it".to_string());
    }
    if let Some((name, colour)) = deposit {
        let word = colour_word(colour, true);
        parts.push(if has_liquid {
            format!("there is {word} {name} at the bottom")
        } else {
            format!("there is {word} {name} in the beaker")
        });
    }
    let mut text = parts.join(", ");
    text.push('.');
    // Sentence case.
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn vessel_with(items: &[(&str, f64, Phase)]) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        for (key, moles, phase) in items {
            v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
        }
        v
    }

    #[test]
    fn water_is_colourless_and_clear() {
        let a = observe(&vessel_with(&[("water", 5.55, Phase::Liquid)]));
        assert!(a.words.contains("colourless"), "{}", a.words);
        assert!(a.words.contains("clear"), "{}", a.words);
        assert!(a.cloudiness < 0.01);
    }

    #[test]
    fn copper_sulfate_solution_is_blue() {
        let a = observe(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("Cu+2", 0.05, Phase::Aqueous),
        ]));
        let c = a.liquid.expect("a colour");
        assert!(c.b > c.r, "blue channel dominates: {c:?}");
        assert!(a.words.contains("blue"), "{}", a.words);
    }

    #[test]
    fn permanganate_is_visible_at_a_tiny_concentration() {
        // The point of the strength parameter: you can see permanganate at
        // a concentration where copper would be invisible.
        let faint = observe(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("MnO4-", 0.00001, Phase::Aqueous),
        ]));
        assert_ne!(colour_word(&faint.liquid.unwrap(), false), "colourless");

        let copper_same = observe(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("Cu+2", 0.00001, Phase::Aqueous),
        ]));
        assert_eq!(
            colour_word(&copper_same.liquid.unwrap(), false),
            "colourless"
        );
    }

    #[test]
    fn a_precipitate_makes_it_cloudy() {
        let a = observe(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("AgCl", 0.01, Phase::Solid),
        ]));
        assert!(a.cloudiness > 0.1, "cloudiness {}", a.cloudiness);
        assert!(a.words.contains("cloudy"), "{}", a.words);
        assert!(a.words.contains("silver chloride"), "{}", a.words);
    }

    #[test]
    fn permanganate_goes_from_pink_to_purple_as_it_concentrates() {
        // The payoff of computing colour from a spectrum rather than
        // tinting: one substance, one ε(λ), and the *word* changes with
        // concentration exactly as it does in a beaker.
        let word_at = |c: f64| {
            let a = observe(&vessel_with(&[
                ("water", 55.5, Phase::Liquid),
                ("MnO4-", c, Phase::Aqueous),
            ]));
            colour_word(&a.liquid.expect("a colour"), false)
        };
        assert_eq!(word_at(1e-5), "pink", "dilute permanganate is pink");
        assert_eq!(
            word_at(1e-3),
            "purple",
            "concentrated permanganate is purple"
        );
    }

    #[test]
    fn an_empty_beaker_says_so() {
        let a = observe(&Vessel::new(VesselId(0), "beaker"));
        assert!(a.words.contains("empty"));
    }
}

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
        // An indicator has two spectra and the pH picks the mixture, so it
        // is asked for one only once the solution has been characterised.
        // Without a pH there is no answer to give, and guessing at one
        // would make the bench assert a colour it has not computed.
        let eps = if crate::indicator::is_ph_dependent(&p.species.0) {
            // KID-8: a two-form indicator and a four-form pigment ladder ask
            // the same question of the solution, so they are asked it the
            // same way.
            match vessel.solution.as_ref() {
                Some(sol) => match crate::indicator::spectrum_at_ph(&p.species.0, sol.ph) {
                    Some(spectrum) => spectrum,
                    None => continue,
                },
                None => continue,
            }
        } else {
            match species::lookup(&p.species).and_then(|d| d.spectrum) {
                Some(spectrum) => *spectrum,
                None => continue,
            }
        };
        let visible_moles =
            (p.moles.0 - crate::surface_colour::sequestered_moles(vessel, &p.species)).max(0.0);
        let concentration = visible_moles / litres;
        for (band, e) in absorbance.iter_mut().zip(eps.iter()) {
            *band += e * concentration * crate::vessel::path_cm_for(&vessel.label);
        }
    }
    let starch_iodine_complex_moles = crate::starch_iodine::add_absorbance(
        vessel,
        litres,
        crate::vessel::path_cm_for(&vessel.label),
        &mut absorbance,
    );
    let has_liquid = vessel
        .contents
        .iter()
        .any(|p| matches!(p.phase, Phase::Liquid | Phase::Aqueous));
    let pigment_layers = vessel
        .unresolved_materials
        .iter()
        .filter_map(|portion| {
            let recipe = crate::material::lookup(&portion.material, None)?;
            let optics = crate::material::pigment_optics(&recipe)?;
            Some((optics, portion.amount))
        })
        .collect::<Vec<_>>();
    let pigment_amounts = pigment_layers
        .iter()
        .map(|(optics, amount)| crate::pigment::PigmentAmount {
            key: &optics.key,
            amount: *amount,
            optics: Some(optics),
        })
        .collect::<Vec<_>>();
    let pigment_colour = (!pigment_amounts.is_empty())
        .then(|| crate::pigment::opaque_mixture_colour(&pigment_amounts).ok())
        .flatten();
    let mut liquid = pigment_colour
        .or_else(|| has_liquid.then(|| crate::spectrum::transmitted_colour(&absorbance)))
        .map(|rgb| Colour {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
            strength: 0.0,
        });
    let colloid = crate::material::colloid_observation(vessel);
    if let (Some(colloid), Some(colour)) = (colloid, &mut liquid) {
        let opacity = colloid.cloudiness;
        // The computed dye spectrum still absorbs light in an opaque,
        // scattering medium. Modulate the colloid's scattered-white base by
        // that spectral result before applying opacity; painting the base
        // over it at opacity=1 would make stirred food colouring disappear.
        // Localized surface dye is excluded from `colour` above, so untouched
        // milk remains warm white.
        let tint = |transmitted: u8, base: u8| transmitted as f64 * base as f64 / 255.0;
        colour.r = ((colour.r as f64 * (1.0 - opacity) + tint(colour.r, colloid.srgb[0]) * opacity)
            .round()) as u8;
        colour.g = ((colour.g as f64 * (1.0 - opacity) + tint(colour.g, colloid.srgb[1]) * opacity)
            .round()) as u8;
        colour.b = ((colour.b as f64 * (1.0 - opacity) + tint(colour.b, colloid.srgb[2]) * opacity)
            .round()) as u8;
    }

    // --- Cloudiness and deposit from suspended solid.
    let mut solid_moles = 0.0;
    let mut biggest: Option<(&str, f64, Colour)> = None;
    // KID-5: every settled solid, not only the largest.
    //
    // A nail rusting in salt water ends with grey iron and reddish-brown
    // iron(III) oxide side by side at the bottom, and the description named
    // the iron and stopped — so the one thing the experiment exists to show
    // was in the ledger, drawn nowhere, and spoken of by nobody. That is the
    // same defect the particle view already refuses to commit: a picture
    // that silently omits a species teaches that the species is not there.
    let mut settled: Vec<(&str, f64, Colour)> = Vec::new();
    // KID-19b: a solid lighter than the liquid is at the TOP, and saying it
    // is at the bottom is not a simplification, it is the wrong answer to
    // the question K32 asks. The registry knew every plastic's density all
    // along; nothing put it beside the liquid's.
    let mut floating: Vec<(&str, f64, Colour)> = Vec::new();
    let liquid_density = crate::buoyancy::liquid_density_g_per_ml(vessel);
    for p in &vessel.contents {
        if p.phase != Phase::Solid {
            continue;
        }
        let rises = liquid_density
            .and_then(|density| crate::buoyancy::floats_in(&p.species, density))
            .unwrap_or(false);
        // A plated metal is not a suspension.
        //
        // Every solid used to count towards turbidity, so copper displaced
        // onto a magnesium ribbon made the beaker "so cloudy you cannot see
        // through it" — but displaced metal deposits on the surface it grew
        // on, or settles as a coherent sponge. It is the most *visible*
        // thing in the beaker and the least cloudy. Silver chloride
        // genuinely does hang in the water and still counts.
        //
        // It still names itself as the deposit below: what changes is that
        // you can see through the liquid to look at it.
        let tracked_suspension = vessel.suspended_fraction_of(&p.species);
        // Neither a plated metal nor something sitting on the surface is a
        // suspension: five grams of floating polystyrene made the water
        // "so cloudy you cannot see through it", which is the same defect
        // the plated-metal branch below was written to fix.
        if !crate::displacement::is_elemental_metal(&p.species.0) && !rises {
            solid_moles += p.moles.0 * tracked_suspension.unwrap_or(1.0);
        }
        let data = species::lookup(&p.species);
        let colour = if p.species.0 == "starch" && starch_iodine_complex_moles > 0.0 {
            Colour {
                r: 15,
                g: 20,
                b: 48,
                strength: 0.0,
            }
        } else {
            data.and_then(|d| d.colour).unwrap_or(Colour {
                r: 220,
                g: 220,
                b: 220,
                strength: 0.0,
            })
        };
        let name = data.map(|d| d.name).unwrap_or(p.species.0.as_str());
        let settled_moles = p.moles.0 * tracked_suspension.map(|f| 1.0 - f).unwrap_or(1.0);
        // A floating solid is not settled, so `settled_moles` is the wrong
        // measure of it — a tracked suspension makes that term zero and the
        // plastic would be named nowhere at all, which is the silent miss
        // K32 recorded. What floats is all of it.
        // Below the observable floor nothing is "at the bottom": a decant
        // that carried 11 µmol of antlerite across was told there was green
        // solid in the receiving beaker.
        if rises && p.moles.0 >= crate::OBSERVABLE_MOLES {
            floating.push((name, p.moles.0, colour));
        } else if settled_moles >= crate::OBSERVABLE_MOLES {
            {
                settled.push((name, settled_moles, colour));
                if biggest.as_ref().is_none_or(|(_, m, _)| settled_moles > *m) {
                    biggest = Some((name, settled_moles, colour));
                }
            }
        }
    }
    let particle_cloudiness = if pigment_colour.is_some() {
        1.0
    } else if has_liquid {
        (solid_moles / litres / OPAQUE_AT).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let emulsion_cloudiness = crate::emulsion::observe(vessel)
        .map(|emulsion| 0.78 * emulsion.dispersed_fraction)
        .unwrap_or(0.0);
    let colloid_cloudiness = colloid.map(|colloid| colloid.cloudiness).unwrap_or(0.0);
    let protein_cloudiness = crate::protein::observe(vessel)
        .iter()
        .filter(|protein| protein.coagulated)
        .map(|protein| protein.denatured_fraction)
        .fold(0.0_f64, f64::max);
    let cloudiness = particle_cloudiness
        .max(emulsion_cloudiness)
        .max(colloid_cloudiness)
        .max(protein_cloudiness);
    let deposit = biggest.map(|(name, _, colour)| (name.to_string(), colour));
    // Ordered by how much of it there is, and cut where a solid stops being
    // worth mentioning: a tenth of the largest heap is still a heap, a
    // thousandth is a contaminant nobody would point at. `deposit` keeps its
    // single-value shape so the scene contract is unchanged; the extra names
    // reach the reader through the words.
    settled.sort_by(|a, b| b.1.total_cmp(&a.1));
    let visible_floor = settled.first().map(|(_, m, _)| m * 0.1).unwrap_or(0.0);
    let deposits: Vec<(String, Colour)> = settled
        .iter()
        .filter(|(_, moles, _)| *moles >= visible_floor)
        .take(3)
        .map(|(name, _, colour)| ((*name).to_string(), *colour))
        .collect();

    floating.sort_by(|a, b| b.1.total_cmp(&a.1));
    let floats: Vec<(String, Colour)> = floating
        .iter()
        .take(3)
        .map(|(name, _, colour)| ((*name).to_string(), *colour))
        .collect();

    // Gas in a vessel that also holds liquid is gas coming *out* of the
    // liquid, which is the single most visible thing in a school kinetics
    // practical. This field existed and was hardcoded `false`, so a flask
    // holding 0.05 mol of oxygen was described as "colourless and clear".
    let bubbling = has_liquid
        && vessel
            .contents
            .iter()
            .any(|p| p.phase == Phase::Gas && p.moles.0 >= crate::OBSERVABLE_MOLES);

    let words = describe(
        LiquidState {
            colour: &liquid,
            cloudiness,
            present: has_liquid,
            density: liquid_density,
        },
        &deposits,
        &floats,
        bubbling,
        vessel,
    );
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
pub(crate) fn colour_word(c: &Colour, solid: bool) -> &'static str {
    let (r, g, b) = (c.r as f64, c.g as f64, c.b as f64);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let chroma = max - min;
    // KID-5: a solid with a hue nobody can see is not that hue.
    //
    // Zinc is rgb(186, 196, 200) — a chroma of 14, which clears the
    // absolute cut-off below by two, and a *saturation* of 0.07, which is
    // grey by any measure. The hue arithmetic duly reported "blue-green
    // zinc". Chroma alone cannot separate a pale metal from a pale colour
    // because it does not know how bright the sample is; saturation does,
    // and it is the same quantity the pink/purple rule below already
    // relies on. Applied to solids only: a dilute coloured *solution* is
    // genuinely pale-coloured, and Beer–Lambert has already said so.
    let achromatic_solid = solid && chroma / max.max(1.0) < 0.12;
    if chroma < 12.0 || achromatic_solid {
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
            // KID-5: the pink/red split is a *transmission* rule. It exists
            // because dilute and concentrated permanganate are one spectrum
            // at two path-lengths, and a chemist calls the first pink and
            // the second purple — washing-out, not hue. A solid's colour is
            // scattering, and there the same rule is the wrong physics:
            // iron(III) oxide is rgb(145, 66, 54), a saturation of 0.63,
            // and it came out "pink". Rust is not pink, and a child looking
            // at a rusted nail is the reader who would notice first.
            if solid {
                if saturation < 0.75 {
                    "reddish brown"
                } else {
                    "red"
                }
            } else if saturation < 0.7 {
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

/// A bright scattering liquid is visibly white rather than "colourless".
/// Transmission-only liquids retain the ordinary colour vocabulary.
pub(crate) fn liquid_colour_word(c: &Colour, cloudiness: f64) -> &'static str {
    let max = c.r.max(c.g).max(c.b);
    let min = c.r.min(c.g).min(c.b);
    if cloudiness > 0.6 && max > 220 && max - min < 15 {
        "white"
    } else {
        colour_word(c, false)
    }
}

struct LiquidState<'a> {
    colour: &'a Option<Colour>,
    cloudiness: f64,
    present: bool,
    density: Option<f64>,
}

fn describe(
    liquid: LiquidState<'_>,
    deposits: &[(String, Colour)],
    floats: &[(String, Colour)],
    bubbling: bool,
    vessel: &Vessel,
) -> String {
    let LiquidState {
        colour,
        cloudiness,
        present: has_liquid,
        density: liquid_density,
    } = liquid;
    if vessel.is_empty() {
        return "The beaker is empty.".to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    if has_liquid {
        let word = if crate::starch_iodine::complex_moles(vessel) > 0.0 {
            "blue-black"
        } else if crate::starch_iodine::has_aqueous_lugol_colour(vessel) {
            "brown"
        } else {
            colour
                .as_ref()
                .map(|c| liquid_colour_word(c, cloudiness))
                .unwrap_or("colourless")
        };
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
    for protein in crate::protein::observe(vessel)
        .into_iter()
        .filter(|protein| protein.coagulated)
    {
        parts.push(format!(
            "the protein in {} has denatured and coagulated into an opaque white solid",
            protein.material
        ));
    }
    if !deposits.is_empty() {
        let named: Vec<String> = deposits
            .iter()
            .map(|(name, colour)| format!("{} {name}", colour_word(colour, true)))
            .collect();
        let list = match named.split_last() {
            Some((last, [])) => last.clone(),
            Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
            None => String::new(),
        };
        parts.push(if has_liquid {
            format!("there is {list} at the bottom")
        } else {
            format!("there is {list} in the beaker")
        });
    }
    // A powder that sits ON the water is at the top of it too, and it is
    // named by its recipe rather than by a species: the grains are
    // conserved unresolved matter, which is exactly why nothing was
    // saying they were there.
    for float in crate::material::surface_floaters(vessel) {
        parts.push(if float.coverage >= 0.999 {
            format!("a skin of {} covers the surface", float.material)
        } else {
            format!("grains of {} float on the surface", float.material)
        });
    }
    // KID-19b: and what is lighter than the liquid is at the top of it.
    if !floats.is_empty() && has_liquid {
        let named: Vec<String> = floats
            .iter()
            .map(|(name, colour)| format!("{} {name}", colour_word(colour, true)))
            .collect();
        let list = match named.split_last() {
            Some((last, [])) => last.clone(),
            Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
            None => String::new(),
        };
        parts.push(format!("{list} floats on top"));
    }
    // A named object is governed by its whole-object bulk density. Comparing
    // only its resolved ingredients gets porous pumice, foam and fruit wrong:
    // the trapped air that lowers their bulk density is not a species.
    for solid in crate::material::conserved_unresolved_solids(vessel) {
        let position = liquid_density
            .zip(solid.bulk_density_g_per_ml)
            .map(|(liquid, object)| object < liquid);
        parts.push(match (has_liquid, position) {
            (true, Some(true)) => format!(
                "a piece of {} {} floats on top",
                solid.colour_word, solid.material
            ),
            (true, Some(false)) => format!(
                "a piece of {} {} is at the bottom",
                solid.colour_word, solid.material
            ),
            _ => format!(
                "a piece of {} {} is in the beaker",
                solid.colour_word, solid.material
            ),
        });
    }
    for object in crate::material::bulk_solid_objects(vessel) {
        // Role-backed conserved solids were already described above with
        // their reviewed colour; do not name the same raisin twice.
        if crate::material::conserved_unresolved_solids(vessel)
            .iter()
            .any(|solid| solid.recipe_id == object.recipe_id)
        {
            continue;
        }
        parts.push(if has_liquid {
            if liquid_density.is_some_and(|liquid| object.bulk_density_g_per_ml < liquid) {
                format!("a piece of {} floats on top", object.material)
            } else {
                format!("a piece of {} is at the bottom", object.material)
            }
        } else {
            format!("a piece of {} is in the beaker", object.material)
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

    /// A plated metal is the most visible thing in the beaker and the
    /// least cloudy.
    ///
    /// Displaced copper grows on the surface it came out on, or settles as
    /// a coherent sponge; it does not hang in the water. Counting every
    /// solid as turbidity made a magnesium ribbon in copper sulfate report
    /// a liquid "so cloudy you cannot see through it", which is the one
    /// thing that beaker is not.
    #[test]
    fn a_plated_metal_does_not_cloud_the_liquid() {
        let a = observe(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("Cu", 0.01, Phase::Solid),
        ]));
        assert!(a.cloudiness < 0.01, "cloudiness {}", a.cloudiness);
        assert!(!a.words.contains("cloudy"), "{}", a.words);
        // Still seen, still named — it is the deposit, not a suspension.
        assert!(a.words.contains("copper"), "{}", a.words);
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

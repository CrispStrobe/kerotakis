//! GUI-003: the scene — a versioned render model of the bench.
//!
//! A bench canvas needs to *paint* a vessel: how much liquid, what colour,
//! what sits at the bottom with which texture, whether gas is rising,
//! whether the top is open or sealed, and which numbers deserve a badge.
//! All of that is already computed — by `appearance::observe` (colour from
//! spectra, cloudiness from suspended solid), by the vessel state itself,
//! and by the aqueous solvers — but a client would have to re-derive it
//! from raw `Vessel` JSON, and two clients would re-derive it differently.
//!
//! The scene is that derivation done once, engine-side, so the web canvas
//! and a native canvas paint the same picture and a golden test can pin the
//! frame. It is a *render model*, not more state: everything here is a
//! projection of the vessel, recomputed on demand, never stored.
//!
//! Contract rules (PROTOCOL.md): the serialized shape is protocol API.
//! Evolution is additive — new fields arrive with `#[serde(default)]`, and
//! consumers ignore fields they do not know. Effects and apparatus are
//! deliberately absent from v1: an effect must never fire without a
//! computed event behind it, so they enter when their state does.

use serde::{Deserialize, Serialize};

use crate::appearance::{self, colour_word};
use crate::ops::Confidence;
use crate::species::{self, Colour, Phase};
use crate::vessel::{Headspace, Vessel, VesselId};
use crate::Bench;

/// Bumped only for a breaking change, which the evolution rules exist to
/// prevent. Expect this to stay 1.
pub const SCENE_VERSION: u32 = 1;

/// Everything a bench canvas needs, nothing it must derive.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    /// Format version, always [`SCENE_VERSION`].
    pub scene: u32,
    pub vessels: Vec<SceneVessel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneVessel {
    pub id: VesselId,
    /// "beaker" — the drawable kind, from the vessel's label.
    pub label: String,
    /// `None` when the vessel holds no liquid phase.
    pub liquid: Option<SceneLiquid>,
    /// The liquid as VISIBLE layers, bottom first (GUI-058). One entry
    /// for an ordinary mixed solution; two when computed liquid–liquid
    /// equilibrium splits the phases (the organic floats by density).
    /// The volumes sum to `liquid.volume_l`; a renderer that stacks
    /// these draws exactly the engine's phase picture.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<SceneLayer>,
    /// Solids present, aggregated per species, largest first.
    pub solids: Vec<SceneSolid>,
    /// Gas visibly rising through the liquid.
    pub bubbling: bool,
    /// Persistent foam target derived from gas production and a declared
    /// stabilizer role. Absent for the no-soap control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub foam: Option<SceneFoam>,
    /// Unresolved floating grains at the liquid surface, including the
    /// computed central clearing made by a recipe-declared surfactant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_particles: Option<SceneSurfaceParticles>,
    /// Temporary oil-in-water dispersion produced by a computed stir action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emulsion: Option<SceneEmulsion>,
    /// The gas boundary, serialized with its existing `boundary` tag:
    /// open, sealed, pressure_controlled, or swept.
    #[serde(flatten)]
    pub headspace: Headspace,
    pub temperature_k: f64,
    pub pressure_pa: f64,
    /// Bench time this vessel has experienced, seconds.
    pub elapsed_s: f64,
    /// Current material mass in grams. Container/tube tare is excluded and
    /// cancels when equal centrifuge tubes are used opposite each other.
    #[serde(default)]
    pub mass_g: f64,
    /// The plain-words observation from `appearance::observe` — the lv1
    /// sentence, and the accessibility text for the drawn vessel.
    pub words: String,
    /// Numbers worth pinning to the vessel, each with the confidence class
    /// its visual encoding follows (GUI-023).
    pub badges: Vec<Badge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneFoam {
    pub trapped_gas_liters: f64,
    pub volume_liters: f64,
    pub height_cm: f64,
    pub overflow_liters: f64,
    /// Tint carried into the bubble films by the computed liquid mixture.
    /// Foam remains mostly gas, so renderers should lighten this colour
    /// rather than paint it as an opaque block.
    #[serde(default = "default_foam_srgb")]
    pub srgb: [u8; 3],
    #[serde(default = "default_foam_colour_word")]
    pub colour_word: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSurfaceParticles {
    pub material: String,
    pub coverage_fraction: f64,
    pub cleared_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEmulsion {
    pub material: String,
    pub dispersed_volume_l: f64,
    pub dispersed_fraction: f64,
    pub half_life_seconds: f64,
}

fn default_foam_srgb() -> [u8; 3] {
    [245, 245, 245]
}

fn default_foam_colour_word() -> String {
    "colourless".to_string()
}

/// One visible liquid layer (GUI-058).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneLayer {
    /// Species key for the dominant phase, or "solution" for the mixed
    /// aqueous layer.
    pub species: String,
    pub name: String,
    pub volume_l: f64,
    pub srgb: [u8; 3],
    pub colour_word: String,
}

/// The liquid, ready to paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneLiquid {
    pub volume_l: f64,
    /// Transmitted colour through `path_length_cm` of this liquid,
    /// computed by Beer–Lambert over the CIE 1931 observer.
    pub srgb: [u8; 3],
    /// The colour in plain words ("blue", "pink") — colour never carries
    /// meaning alone, so the word travels with the value.
    pub colour_word: String,
    /// 0 = clear, 1 = opaque, from suspended solid.
    pub cloudiness: f64,
    /// The path length the colour was computed for. A canvas drawing a
    /// wider or narrower vessel may rescale absorbance against this basis.
    pub path_length_cm: f64,
}

/// One solid species in the vessel, ready to paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSolid {
    /// Registry key ("AgCl").
    pub species: String,
    /// Display name ("silver chloride").
    pub name: String,
    pub moles: f64,
    pub srgb: [u8; 3],
    pub colour_word: String,
    /// An elemental metal deposits as a coating or coherent sponge and does
    /// not cloud the liquid; anything else is a precipitate that suspends
    /// and settles. Decides texture, and matches the turbidity physics in
    /// `appearance::observe`.
    pub metallic: bool,
    /// Fraction currently in the settled deposit. Legacy state remains fully
    /// visible at the bottom until an operation establishes suspension state.
    #[serde(default = "fully_settled")]
    pub settled_fraction: f64,
}

fn fully_settled() -> f64 {
    1.0
}

/// A number pinned to the drawn vessel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    /// Stable key: "ph", "ionic_strength", "pe".
    pub key: String,
    pub value: f64,
    /// How strongly the engine stands behind it (GUI-023's fixed visual
    /// encoding follows this).
    pub confidence: Confidence,
}

/// The whole bench, as a canvas paints it.
pub fn scene(bench: &Bench) -> Scene {
    scene_of(&bench.vessels)
}

/// The render model over any vessel slice — for callers that hold vessels
/// without a `Bench` (the CLI/MCP `--json` contract builder).
pub fn scene_of(vessels: &[Vessel]) -> Scene {
    Scene {
        scene: SCENE_VERSION,
        vessels: vessels.iter().map(scene_vessel).collect(),
    }
}

/// One vessel's render model.
pub fn scene_vessel(v: &Vessel) -> SceneVessel {
    let seen = appearance::observe(v);
    let material_layers = crate::material::immiscible_liquid_layers(v);
    let emulsion_observation = crate::emulsion::observe(v);
    let material_volume_l: f64 = material_layers.iter().map(|layer| layer.volume_l).sum();
    let homogeneous_material_volume_l = crate::material::homogeneous_unresolved_liquid_volume_l(v);
    let resolved_volume_l = v.liquid_volume().0;

    let mut liquid = seen
        .liquid
        .as_ref()
        .map(|c| SceneLiquid {
            volume_l: resolved_volume_l + homogeneous_material_volume_l + material_volume_l,
            srgb: [c.r, c.g, c.b],
            colour_word: appearance::liquid_colour_word(c, seen.cloudiness).to_string(),
            cloudiness: seen.cloudiness,
            path_length_cm: crate::vessel::path_cm_for(&v.label),
        })
        .or_else(|| {
            material_layers.first().map(|layer| SceneLiquid {
                volume_l: material_volume_l,
                srgb: layer.srgb,
                colour_word: layer.colour_word.clone(),
                cloudiness: 0.0,
                path_length_cm: crate::vessel::path_cm_for(&v.label),
            })
        });
    if let (Some(liquid), Some(emulsion)) = (&mut liquid, &emulsion_observation) {
        liquid.cloudiness = liquid.cloudiness.max(0.78 * emulsion.dispersed_fraction);
    }

    // Layers (GUI-058): the engine's computed phase split, made drawable.
    let mut layers = match (&liquid, crate::solve::layered_pair(v)) {
        (Some(l), Some((upper_key, lower_key))) => {
            let upper_data = species::lookup(&crate::SpeciesId::new(upper_key));
            let upper_vol: f64 = v
                .contents
                .iter()
                .filter(|p| p.phase == Phase::Liquid && p.species.0 == upper_key)
                .filter_map(|p| upper_data.map(|d| d.liters_from_moles(p.moles).0))
                .sum();
            let upper_colour = upper_data.and_then(|d| d.colour).unwrap_or(Colour {
                r: 235,
                g: 238,
                b: 240,
                strength: 0.0,
            });
            vec![
                // Bottom: the aqueous layer wears the solution's own
                // observed colour; its volume is what the organic left.
                SceneLayer {
                    species: lower_key.to_string(),
                    name: species::lookup(&crate::SpeciesId::new(lower_key))
                        .map(|d| d.name.to_string())
                        .unwrap_or_else(|| lower_key.to_string()),
                    volume_l: resolved_volume_l - upper_vol,
                    srgb: l.srgb,
                    colour_word: l.colour_word.clone(),
                },
                SceneLayer {
                    species: upper_key.to_string(),
                    name: upper_data
                        .map(|d| d.name.to_string())
                        .unwrap_or_else(|| upper_key.to_string()),
                    volume_l: upper_vol,
                    srgb: [upper_colour.r, upper_colour.g, upper_colour.b],
                    colour_word: colour_word(&upper_colour, false).to_string(),
                },
            ]
        }
        (Some(l), None) if resolved_volume_l > 0.0 => vec![SceneLayer {
            species: "solution".to_string(),
            name: "solution".to_string(),
            volume_l: resolved_volume_l + homogeneous_material_volume_l,
            srgb: l.srgb,
            colour_word: l.colour_word.clone(),
        }],
        _ => Vec::new(),
    };
    layers.extend(material_layers.iter().map(|layer| SceneLayer {
        species: layer.key.clone(),
        name: layer.material.clone(),
        volume_l: layer.volume_l,
        srgb: layer.srgb,
        colour_word: layer.colour_word.clone(),
    }));
    if let Some(emulsion) = &emulsion_observation {
        if let Some(oil_layer) = layers.iter_mut().find(|layer| {
            layer.species
                == material_layers
                    .iter()
                    .find(|material| material.recipe_id == emulsion.oil_recipe_id)
                    .map(|material| material.key.as_str())
                    .unwrap_or("")
        }) {
            oil_layer.volume_l = (oil_layer.volume_l - emulsion.dispersed_volume_l).max(0.0);
        }
        if let Some(aqueous_layer) = layers.first_mut() {
            if !material_layers
                .iter()
                .any(|material| material.key == aqueous_layer.species)
            {
                aqueous_layer.volume_l += emulsion.dispersed_volume_l;
            }
        }
        layers.retain(|layer| layer.volume_l > 1e-9);
    }

    // Aggregate solids per species, keeping first-seen order, then sort by
    // amount so the biggest deposit paints first.
    let mut solids: Vec<SceneSolid> = Vec::new();
    for p in v.contents.iter().filter(|p| p.phase == Phase::Solid) {
        if let Some(existing) = solids.iter_mut().find(|s| s.species == p.species.0) {
            existing.moles += p.moles.0;
            continue;
        }
        let data = species::lookup(&p.species);
        let colour = data.and_then(|d| d.colour).unwrap_or(Colour {
            r: 220,
            g: 220,
            b: 220,
            strength: 0.0,
        });
        solids.push(SceneSolid {
            species: p.species.0.to_string(),
            name: data
                .map(|d| d.name.to_string())
                .unwrap_or_else(|| p.species.0.to_string()),
            moles: p.moles.0,
            srgb: [colour.r, colour.g, colour.b],
            colour_word: colour_word(&colour, true).to_string(),
            metallic: crate::displacement::is_elemental_metal(&p.species.0),
            settled_fraction: v
                .suspended_fraction_of(&p.species)
                .map(|fraction| 1.0 - fraction)
                .unwrap_or(1.0),
        });
    }
    solids.sort_by(|a, b| b.moles.total_cmp(&a.moles));

    let mut badges = Vec::new();
    if let Some(sol) = &v.solution {
        badges.push(Badge {
            key: "ph".into(),
            value: sol.ph,
            confidence: Confidence::Computed,
        });
        badges.push(Badge {
            key: "ionic_strength".into(),
            value: sol.ionic_strength,
            confidence: Confidence::Computed,
        });
        if let Some(pe) = sol.pe {
            badges.push(Badge {
                key: "pe".into(),
                value: pe,
                confidence: Confidence::Computed,
            });
        }
    }
    let foam = (v.foam.volume_liters > 1e-9).then(|| {
        let (capacity_l, area_cm2) = match v.label.as_str() {
            "tube" => (0.030, 3.0),
            "cylinder" => (0.100, 8.0),
            "flask" => (0.250, 20.0),
            "crucible" => (0.050, 18.0),
            _ => (0.250, 28.0),
        };
        SceneFoam {
            trapped_gas_liters: v.foam.trapped_gas_liters,
            volume_liters: v.foam.volume_liters,
            height_cm: v.foam.volume_liters * 1000.0 / area_cm2,
            overflow_liters: (v.liquid_volume().0 + v.foam.volume_liters - capacity_l).max(0.0),
            srgb: liquid
                .as_ref()
                .map(|liquid| liquid.srgb)
                .unwrap_or_else(default_foam_srgb),
            colour_word: liquid
                .as_ref()
                .map(|liquid| liquid.colour_word.clone())
                .unwrap_or_else(default_foam_colour_word),
        }
    });
    if let Some(foam) = &foam {
        badges.push(Badge {
            key: "foam_height_cm".into(),
            value: foam.height_cm,
            confidence: Confidence::Modeled,
        });
    }

    let mut words = seen.words;
    if let Some(emulsion) = &emulsion_observation {
        words.push_str(&format!(
            " Stirring has dispersed {:.0}% of the {} as cloudy droplets; the rest remains above the water.",
            emulsion.dispersed_fraction * 100.0,
            emulsion.material,
        ));
    } else if let Some(layer) = material_layers.first() {
        if resolved_volume_l > 0.0 {
            words.push_str(&format!(
                " {} forms a separate {} layer above the water.",
                layer.material, layer.colour_word
            ));
        } else {
            words.push_str(&format!(" The vessel contains {}.", layer.material));
        }
    }
    if let Some(foam) = &foam {
        if foam.overflow_liters > 0.0 {
            words.push_str(" Foam is spilling over the rim.");
        } else {
            words.push_str(" Foam is standing above the liquid.");
        }
    }

    SceneVessel {
        id: v.id,
        label: v.label.clone(),
        liquid,
        layers,
        solids,
        bubbling: seen.bubbling,
        foam,
        surface_particles: v
            .surface_particles
            .as_ref()
            .map(|particles| SceneSurfaceParticles {
                material: particles.material.clone(),
                coverage_fraction: particles.coverage_fraction,
                cleared_fraction: particles.cleared_fraction,
            }),
        emulsion: emulsion_observation.map(|emulsion| SceneEmulsion {
            material: emulsion.material,
            dispersed_volume_l: emulsion.dispersed_volume_l,
            dispersed_fraction: emulsion.dispersed_fraction,
            half_life_seconds: emulsion.half_life_seconds,
        }),
        headspace: v.headspace,
        temperature_k: v.temperature.0,
        pressure_pa: v.pressure.0,
        elapsed_s: v.elapsed_seconds,
        mass_g: v.mass().0,
        words,
        badges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Moles, SpeciesId};

    fn vessel_with(items: &[(&str, f64, Phase)]) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "beaker");
        for (key, moles, phase) in items {
            v.deposit(SpeciesId::new(key), Moles(*moles), *phase);
        }
        v
    }

    #[test]
    fn an_ordinary_solution_is_one_layer_matching_the_liquid() {
        let v = vessel_with(&[("water", 5.55, Phase::Liquid)]);
        let s = scene_vessel(&v);
        let l = s.liquid.as_ref().expect("liquid");
        assert_eq!(s.layers.len(), 1);
        assert_eq!(s.layers[0].species, "solution");
        assert!((s.layers[0].volume_l - l.volume_l).abs() < 1e-12);
        assert_eq!(s.layers[0].srgb, l.srgb);
    }

    #[test]
    fn hexane_on_water_renders_two_layers_water_at_the_bottom() {
        // 5.55 mol water (~100 mL) + 0.5 mol hexane (~65 mL): the LLE
        // splits them; the scene must say so, in the right order, with
        // volumes that add up to the whole liquid.
        let v = vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("hexane", 0.5, Phase::Liquid),
        ]);
        let s = scene_vessel(&v);
        assert_eq!(
            s.layers.len(),
            2,
            "LLE must split the render: {:?}",
            s.layers
        );
        assert_eq!(s.layers[0].species, "water", "bottom layer");
        assert_eq!(s.layers[1].species, "hexane", "top layer floats");
        let total: f64 = s.layers.iter().map(|l| l.volume_l).sum();
        let liquid = s.liquid.as_ref().expect("liquid").volume_l;
        assert!(
            (total - liquid).abs() < 1e-9,
            "layer volumes {total} must sum to the liquid {liquid}"
        );
        assert!(
            s.layers[1].volume_l > 0.05 && s.layers[1].volume_l < 0.08,
            "0.5 mol hexane is ~65 mL, got {}",
            s.layers[1].volume_l
        );
    }

    #[test]
    fn a_new_bench_is_one_empty_beaker() {
        let s = scene(&Bench::new());
        assert_eq!(s.scene, SCENE_VERSION);
        assert_eq!(s.vessels.len(), 1);
        let v = &s.vessels[0];
        assert!(v.liquid.is_none());
        assert!(v.solids.is_empty());
        assert!(!v.bubbling);
        assert!(v.foam.is_none());
        assert!(v.words.contains("empty"), "{}", v.words);
    }

    #[test]
    fn persistent_foam_is_part_of_the_scene_and_accessible_words() {
        let mut v = vessel_with(&[("water", 5.55, Phase::Liquid)]);
        v.foam.trapped_gas_liters = 0.20;
        v.foam.volume_liters = 0.22;
        v.foam.peak_volume_liters = 0.22;
        let scene = scene_vessel(&v);
        let foam = scene.foam.expect("foam render target");
        assert!((foam.volume_liters - 0.22).abs() < 1e-12);
        assert!(foam.height_cm > 0.0);
        assert!(foam.overflow_liters > 0.0);
        assert!(scene.words.contains("spilling over"));
    }

    #[test]
    fn foam_carries_the_computed_liquid_colour() {
        let mut v = vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("betanin", 0.000_001, Phase::Aqueous),
        ]);
        v.foam.trapped_gas_liters = 0.10;
        v.foam.volume_liters = 0.12;
        v.foam.peak_volume_liters = 0.12;

        let scene = scene_vessel(&v);
        let liquid = scene.liquid.expect("coloured liquid");
        let foam = scene.foam.expect("foam render target");
        assert_eq!(foam.srgb, liquid.srgb);
        assert_eq!(foam.colour_word, liquid.colour_word);
        assert_ne!(foam.colour_word, "colourless");
    }

    #[test]
    fn copper_solution_paints_blue_with_the_word_attached() {
        let s = scene_vessel(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("Cu+2", 0.05, Phase::Aqueous),
        ]));
        let liquid = s.liquid.expect("a liquid");
        assert!(liquid.srgb[2] > liquid.srgb[0], "blue dominates");
        assert_eq!(liquid.colour_word, "blue");
        assert!(liquid.volume_l > 0.09, "≈100 mL of water");
        assert_eq!(liquid.path_length_cm, crate::spectrum::BEAKER_PATH_CM);
    }

    #[test]
    fn a_precipitate_is_textured_a_plated_metal_is_metallic() {
        let s = scene_vessel(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("AgCl", 0.01, Phase::Solid),
            ("Cu", 0.002, Phase::Solid),
        ]));
        let agcl = s.solids.iter().find(|x| x.species == "AgCl").unwrap();
        let cu = s.solids.iter().find(|x| x.species == "Cu").unwrap();
        assert!(!agcl.metallic);
        assert!(cu.metallic);
        // Largest deposit paints first.
        assert_eq!(s.solids[0].species, "AgCl");
        // The precipitate clouds the liquid; the sponge does not add to it.
        assert!(s.liquid.unwrap().cloudiness > 0.1);
    }

    #[test]
    fn the_gas_boundary_keeps_its_tag() {
        let mut v = vessel_with(&[("water", 5.55, Phase::Liquid)]);
        v.headspace = Headspace::Sealed {
            volume: crate::Liters(0.2),
        };
        let json = serde_json::to_value(scene_vessel(&v)).unwrap();
        assert_eq!(json["boundary"], "sealed");
        assert!(json["volume"].is_number());
    }

    /// Glassware geometry reaches the colour pipeline: the same solution
    /// reports a shorter light path in a test tube than in a beaker.
    #[test]
    fn the_tube_has_a_shorter_light_path() {
        let mut v = vessel_with(&[("water", 5.55, Phase::Liquid)]);
        v.label = "tube".to_string();
        let s = scene_vessel(&v);
        assert_eq!(s.liquid.unwrap().path_length_cm, 1.2);
        assert_eq!(s.label, "tube");
    }

    /// The serialized field names are protocol API (PROTOCOL.md). This is
    /// the tripwire: renaming a field fails here before it breaks a client.
    #[test]
    fn the_scene_shape_is_pinned() {
        let s = scene_vessel(&vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("AgCl", 0.01, Phase::Solid),
        ]));
        let json = serde_json::to_value(&s).unwrap();
        for key in [
            "id",
            "label",
            "liquid",
            "solids",
            "bubbling",
            "boundary",
            "temperature_k",
            "pressure_pa",
            "elapsed_s",
            "words",
            "badges",
        ] {
            assert!(json.get(key).is_some(), "missing scene key {key}");
        }
        let liquid = &json["liquid"];
        for key in [
            "volume_l",
            "srgb",
            "colour_word",
            "cloudiness",
            "path_length_cm",
        ] {
            assert!(liquid.get(key).is_some(), "missing liquid key {key}");
        }
        let solid = &json["solids"][0];
        for key in [
            "species",
            "name",
            "moles",
            "srgb",
            "colour_word",
            "metallic",
        ] {
            assert!(solid.get(key).is_some(), "missing solid key {key}");
        }
        // And it round-trips.
        let back: SceneVessel = serde_json::from_value(json).unwrap();
        assert_eq!(back, s);
    }
}

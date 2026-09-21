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
use crate::i18n::Locale;
use crate::ops::Confidence;
use crate::phrase::{compose, Phrase, Slot};
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
    /// BRD-002: the finite bottles on the shelf, in stable key order.
    /// Empty — and omitted from the wire — in a sandbox where nothing runs
    /// out, so a host written before this field sees exactly what it saw
    /// before. Absence of a key here means an unlimited supply, never zero.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stock: Vec<SceneStockBottle>,
}

/// What is left in one shelf bottle, in the unit the `add` grammar takes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneStockBottle {
    /// The shelf key — the same one `species` reports and `add` accepts.
    pub key: String,
    pub remaining: f64,
    /// "mol", "g" or "mL".
    pub unit: String,
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
    /// Coherent named material objects, positioned by whole-object bulk
    /// density rather than by the density of their resolved ingredients.
    #[serde(default)]
    pub bulk_objects: Vec<SceneBulkObject>,
    /// Protective surface films asserted by the provenance of a coherent
    /// material object still present in this vessel. This is persistent state,
    /// never a reconstruction from transient events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coatings: Vec<SceneCoating>,
    /// Current fraction of a tracked metal inventory locked in the oxide
    /// named by its corrosion route. This is a projection of current contents,
    /// not event history, a rate, coating thickness, or surface coverage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub corrosion: Vec<SceneCorrosion>,
    /// Current sorbent/sorbate split from the vessel's stored adsorption
    /// ledger. This is equilibrium bookkeeping, never replayed event history.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adsorption: Vec<SceneAdsorption>,
    /// Equilibrium distribution of supported neutral solutes while both
    /// liquid layers remain together. Recomputed from current vessel matter,
    /// never reconstructed from a past `partitioned` event.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub partition: Vec<ScenePartition>,
    /// Prepared coherent objects with object-owned inventories.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_objects: Vec<SceneMaterialObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soap_scum: Option<SceneSoapScum>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lemon_paper_mark: Option<SceneLemonPaperMark>,
    /// Borate-crosslinked polymer fraction derived from the vessel's current
    /// inventory. This is a visible-state projection, not rheology.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gel: Option<SceneGel>,
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
    /// Resolved food-colour drops whose geometry is still localized at an
    /// opaque liquid surface rather than mixed through the bulk.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surface_colours: Vec<SceneSurfaceColour>,
    /// Temporary oil-in-water dispersion produced by a computed stir action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emulsion: Option<SceneEmulsion>,
    /// Recipe-level aggregate curds separated from a colloidal liquid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curds: Option<SceneCurds>,
    /// Water retained by a declared superabsorbent network. This is a
    /// projection of conserved vessel matter, not an animation trigger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swelling: Option<SceneSwelling>,
    /// Persistent relative light output of a declared chemiluminescent
    /// system at this vessel's temperature and elapsed time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chemiluminescence: Option<SceneChemiluminescence>,
    /// Current conversion of supported unresolved food substrates. This is
    /// stored vessel state projected into the scene, not reconstructed event
    /// history or a claim about visible texture.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enzyme_hydrolysis: Vec<SceneEnzymeHydrolysis>,
    /// Explicit, persistent electrode state. Values absent from this
    /// projection must remain unavailable rather than inferred from a net
    /// current or an animation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub electrodes: Vec<SceneElectrode>,
    /// The gas boundary, serialized with its existing `boundary` tag:
    /// open, sealed, pressure_controlled, or swept.
    #[serde(flatten)]
    pub headspace: Headspace,
    /// Litres of gas space above the liquid, where the vessel OWNS its gas
    /// — sealed and pressure-controlled. `None` for an open or swept
    /// vessel, whose headspace is the room.
    ///
    /// A renderer could reconstruct it from `V = nRT/P` at the moment a
    /// `vessel_pressure_controlled` event named the trapped moles, and
    /// that reconstruction is correct for an ideal gas and stale the
    /// instant anything else changes the headspace. This is the engine's
    /// own figure, standing for as long as it is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headspace_volume_l: Option<f64>,
    /// Moles of gas the vessel holds in that headspace. `None` on the same
    /// vessels `headspace_volume_l` is `None` on, and for the same reason:
    /// an open vessel's gas belongs to the room, not to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headspace_moles: Option<f64>,
    /// The temperature this vessel's liquid would boil at, K — its own
    /// pressure and its own dissolved particles already in it.
    ///
    /// The bench computes this every time something boils and reports it in
    /// `state_changed.at`, but a transition event only exists at the moment
    /// of transition. Between events a renderer had to fall back on pure
    /// water at one atmosphere, which is wrong under a partial vacuum,
    /// wrong in a pressure cooker, wrong for a salted solvent and wrong for
    /// every solvent that is not water — so a vessel SITTING at 350 K under
    /// vacuum could not be drawn boiling. `None` where the vessel holds no
    /// liquid, or where the registry has no transition for the liquid it
    /// holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boiling_point_k: Option<f64>,
    /// The temperature the same liquid would freeze at, K — the other end
    /// of the same plateau, depressed by the same dissolved particles.
    ///
    /// Pressure is deliberately not applied: the pressure dependence of a
    /// melting point is about −0.0074 K per atmosphere for water and this
    /// bench has no model for it, so `states` leaves freezing unmoved and
    /// so does this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melting_point_k: Option<f64>,
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
    ///
    /// English, always, and GENERATED from `clauses` and `notes` below
    /// rather than written beside them, so there is one sentence and not
    /// two to drift. [`SceneVessel::say`] recomposes it in the reader's
    /// language; `crate::scene::localize` does that for a whole scene on
    /// the way out.
    pub words: String,
    /// I18N-11: `words` as a recipe rather than a string — the clauses
    /// `appearance::observe` composed, carried through unchanged.
    ///
    /// `appearance.rs` has spoken German since I18N-7, and the scene then
    /// threw that away: it took the finished English `words` and appended
    /// its own `format!`s to it. The web bench paints from the scene, so a
    /// German reader met the whole tail in English — osmosis, the gel, the
    /// glow, the enzyme, every one of them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clauses: Vec<Phrase>,
    /// The sentences the SCENE adds after the observation: standing
    /// bookkeeping that is a whole statement rather than a clause of the
    /// description — how much water osmosis has moved, what fraction of
    /// the polymer has gelled, where a solute has partitioned.
    ///
    /// Separate from `clauses` because they join differently: a clause is
    /// comma-joined inside one sentence and ends at the full stop, and
    /// each of these is its own sentence after it. That is the same split
    /// `Appearance` makes between `clauses` and `notes`, for the same
    /// reason.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Phrase>,
    /// Numbers worth pinning to the vessel, each with the confidence class
    /// its visual encoding follows (GUI-023).
    pub badges: Vec<Badge>,
}

impl SceneVessel {
    /// Everything this vessel says, in `locale`.
    ///
    /// A scene deserialised from before the clause list existed — a saved
    /// frame, a golden fixture, a host on an older engine — has none, and
    /// gets its English `words` back rather than an empty caption. That is
    /// the same fallback `Appearance::say` makes, and for the same reason:
    /// a missing translation must degrade to English, never to nothing.
    pub fn say(&self, locale: Locale) -> String {
        if self.clauses.is_empty() && self.notes.is_empty() {
            return self.words.clone();
        }
        say(&self.clauses, &self.notes, locale)
    }
}

/// The observation and the standing notes as one caption, in `locale`.
///
/// Shared by `scene_vessel`, which uses it to GENERATE the English, and by
/// [`SceneVessel::say`], which uses it to say the same thing in the
/// reader's language. One function, so the two cannot diverge.
fn say(clauses: &[Phrase], notes: &[Phrase], locale: Locale) -> String {
    let mut text = compose(clauses, locale);
    for note in notes {
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(&note.render(locale));
    }
    text
}

/// The whole scene in `locale` — every sentence it carries, recomposed.
///
/// The counterpart of `render::localize_events`, and needed for the same
/// reason: the web bench paints from the SCENE rather than from the
/// rendered notebook lines, so a caption that is only translated on its
/// way through `render.rs` never reaches the canvas at all. The recipes
/// are left exactly as they are — they are the source, translating them
/// twice is the bug this shape exists to prevent — and only the English
/// `words` fields are replaced.
#[must_use]
pub fn localize(scene: &Scene, locale: Locale) -> Scene {
    if locale.is_english() {
        return scene.clone();
    }
    Scene {
        vessels: scene
            .vessels
            .iter()
            .map(|v| SceneVessel {
                words: v.say(locale),
                coatings: v
                    .coatings
                    .iter()
                    .map(|c| SceneCoating {
                        words: c
                            .phrase
                            .as_ref()
                            .map_or_else(|| c.words.clone(), |p| p.render(locale)),
                        ..c.clone()
                    })
                    .collect(),
                corrosion: v
                    .corrosion
                    .iter()
                    .map(|c| SceneCorrosion {
                        words: c
                            .phrase
                            .as_ref()
                            .map_or_else(|| c.words.clone(), |p| p.render(locale)),
                        ..c.clone()
                    })
                    .collect(),
                ..v.clone()
            })
            .collect(),
        ..scene.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneElectrode {
    pub label: String,
    pub material: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_preparation: Option<String>,
    pub geometric_area_m2: f64,
    pub roughness: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interfacial_potential_v: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<crate::compartment::ElectrodeDiagnostics>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub surface_species: Vec<SceneElectrodeSurfaceSpecies>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deposits: Vec<SceneElectrodeDeposit>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneElectrodeSurfaceSpecies {
    pub reaction_id: String,
    pub species: String,
    pub concentration_mol_per_m3: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneElectrodeDeposit {
    pub species: String,
    pub moles: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage_fraction: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneMaterialObject {
    pub material: String,
    pub recipe_id: String,
    pub mass_g: f64,
    pub exchanged_water_moles: f64,
    pub browned_fraction: f64,
    /// Standing, cumulative water transfer for recipes whose reviewed role is
    /// an osmotic membrane. This deliberately does not claim object geometry,
    /// membrane mechanics, or a final equilibrium mass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub osmosis: Option<SceneOsmosis>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneOsmosis {
    /// Signed cumulative modeled transfer: positive is into the object.
    pub water_moles: f64,
    /// The same transfer expressed using water's registry molar mass.
    pub mass_change_g: f64,
    /// `into_object`, `out_of_object`, or `balanced`.
    pub direction: String,
    /// Short machine-visible model provenance and boundary for clients.
    pub basis: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSoapScum {
    pub aggregate_mass_g: f64,
    pub divalent_ion_moles: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneLemonPaperMark {
    pub dry: bool,
    pub browned_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneGel {
    pub polymer: String,
    pub crosslinker: String,
    pub gelled_fraction: f64,
    pub polymer_grams: f64,
    pub crosslinker_moles: f64,
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
pub struct SceneSurfaceColour {
    pub material: String,
    pub srgb: [u8; 3],
    pub spread_fraction: f64,
    pub relative_amount: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEmulsion {
    pub material: String,
    pub dispersed_volume_l: f64,
    pub dispersed_fraction: f64,
    pub half_life_seconds: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneCurds {
    pub material: String,
    pub formed_fraction: f64,
    pub separation_progress: f64,
    pub solids_mass_g: f64,
    pub srgb: [u8; 3],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSwelling {
    pub dry_polymer_g: f64,
    pub retained_water_g: f64,
    pub swelling_ratio_g_per_g: f64,
    pub capacity_g_per_g: f64,
    pub saturated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneChemiluminescence {
    pub relative_intensity: f64,
    pub half_life_s: f64,
    pub elapsed_s: f64,
    pub temperature_k: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEnzymeHydrolysis {
    pub family: crate::enzyme::EnzymeFamily,
    pub material: String,
    pub substrate: String,
    pub converted_fraction: f64,
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
    /// Species whose optical contribution could not be computed, so the
    /// `srgb` above is INCOMPLETE and the engine knows which part of it is
    /// missing. Already carried to the PROSE as the `look.spectral-gap`
    /// note; this is the same admission, on its way to the picture.
    ///
    /// Names, not a flag. The names cost a few short strings per frame and
    /// buy the only answer a reader who notices the mark can ask for —
    /// *what* is missing — and the prose beside the drawing already names
    /// them, so a bare boolean would have made the two disagree about how
    /// much the model is willing to say.
    ///
    /// Never a colour: the whole content of this signal is that the model
    /// could not compute one. Empty — and omitted from the wire — when the
    /// colour is whole, so a host written before this field sees exactly
    /// what it saw before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spectral_gaps: Vec<String>,
}

/// One solid species in the vessel, ready to paint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSolid {
    /// Registry key ("AgCl").
    pub species: String,
    /// Display name ("silver chloride").
    pub name: String,
    pub moles: f64,
    /// Pure-solid volume derived from registry molar mass and density. This is
    /// additive across lots of the same species and lets renderers scale a
    /// deposit from physical amount instead of inventing a moles-to-pixels
    /// conversion. Zero means the registry has no usable density.
    #[serde(default)]
    pub volume_l: f64,
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
    /// This resolved ingredient is already painted as part of a coherent
    /// named bulk object and must not also appear as a loose deposit.
    #[serde(default)]
    pub represented_by_bulk_object: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneBulkObject {
    pub material: String,
    pub recipe_id: String,
    pub amount_g: f64,
    pub bulk_density_g_per_ml: f64,
    /// "floating", "sunk", or "dry" when there is no liquid to compare.
    pub position: String,
    pub srgb: [u8; 3],
}

/// A source-backed protective film on a coherent material object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneCoating {
    /// "paint" or "passive_film". Renderers must not infer thickness or
    /// coverage from this label.
    pub kind: String,
    /// Material recipe whose persistent lot provenance supports the film.
    pub recipe_id: String,
    /// Registry key of the protected metal.
    pub host_species: String,
    /// Short accessible description of what the projection claims.
    ///
    /// English, and generated from `phrase` below. The web draws it as the
    /// SVG `<title>` of the film, separately from the vessel's own words,
    /// so it needs its own recipe rather than only a place in `notes`.
    pub words: String,
    /// `words` as a recipe (I18N-11).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phrase: Option<Phrase>,
}

/// Standing corrosion bookkeeping for one metal whose route the engine knows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneCorrosion {
    pub metal: String,
    pub corroding: bool,
    pub metal_in_oxide_moles: f64,
    pub metal_in_oxide_fraction: f64,
    /// Accessible boundary statement. Oxide added directly is intentionally
    /// indistinguishable from oxide made in this vessel.
    ///
    /// English, generated from `phrase` below.
    pub words: String,
    /// `words` as a recipe (I18N-11). Carries a percentage, so no
    /// whole-sentence lookup could ever have reached it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phrase: Option<Phrase>,
}

/// Standing adsorption bookkeeping for one supported sorbent/sorbate pair.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneAdsorption {
    pub sorbent: String,
    pub sorbate: String,
    pub held_mg: f64,
    pub still_dissolved_mg: f64,
    pub held_fraction: f64,
    /// Absent only for an inconsistent legacy snapshot with held dye but no
    /// remaining sorbent mass on which to express a loading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading_mg_per_g: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loading_fraction: Option<f64>,
    /// The model's limitations, carried with the quantities they qualify.
    pub boundary: String,
    /// Honest provenance text; current parameters remain pending review.
    pub provenance: String,
}

/// Standing two-liquid equilibrium for one supported neutral solute.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScenePartition {
    pub species: String,
    pub lower_solvent: String,
    pub upper_solvent: String,
    pub total_moles: f64,
    pub lower_moles: f64,
    pub upper_moles: f64,
    pub fraction_lower: f64,
    pub boundary: String,
    pub provenance: String,
}

fn standing_partition(vessel: &Vessel) -> Vec<ScenePartition> {
    let Some((upper, lower)) = crate::solve::layered_pair(vessel) else {
        return Vec::new();
    };
    let solvent_moles = |key: &str| {
        vessel
            .contents
            .iter()
            .filter(|portion| portion.species.0 == key && portion.phase == Phase::Liquid)
            .map(|portion| portion.moles.0)
            .sum::<f64>()
    };
    let lower_solvent_moles = solvent_moles(lower);
    let upper_solvent_moles = solvent_moles(upper);
    let mut solutes = std::collections::BTreeMap::<String, f64>::new();
    for portion in &vessel.contents {
        if portion.species.0 == lower || portion.species.0 == upper {
            continue;
        }
        if !matches!(portion.phase, Phase::Aqueous | Phase::Liquid) {
            continue;
        }
        *solutes.entry(portion.species.0.clone()).or_default() += portion.moles.0;
    }
    let lower_id = crate::SpeciesId::new(lower);
    let upper_id = crate::SpeciesId::new(upper);
    let Some(lower_volume_l) = crate::species::lookup(&lower_id)
        .and_then(|data| data.molar_volume_l_per_mol())
        .map(|volume| volume * lower_solvent_moles)
    else {
        return Vec::new();
    };
    let Some(upper_volume_l) = crate::species::lookup(&upper_id)
        .and_then(|data| data.molar_volume_l_per_mol())
        .map(|volume| volume * upper_solvent_moles)
    else {
        return Vec::new();
    };
    solutes
        .into_iter()
        .filter_map(|(species, total_moles)| {
            let prediction = crate::bench::partition_k(
                &crate::SpeciesId::new(&species),
                &upper_id,
                &lower_id,
                vessel.temperature.0,
            )?;
            let fraction_lower = crate::apparatus::extract(
                total_moles,
                lower_volume_l,
                upper_volume_l,
                prediction.k_organic_over_aqueous,
            )
            .aqueous_moles
                / total_moles;
            Some(ScenePartition {
                species,
                lower_solvent: lower.to_string(),
                upper_solvent: upper.to_string(),
                total_moles,
                lower_moles: total_moles * fraction_lower,
                upper_moles: total_moles * (1.0 - fraction_lower),
                fraction_lower,
                boundary: format!("instant equal-activity equilibrium at the current temperature and layer volumes; neutral dilute solutes only, with no mass-transfer rate, emulsion, interface geometry, ion partition, or concentration-dependent solute interaction; {}", prediction.model),
                provenance: prediction.provenance,
            })
        })
        .collect()
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
    Scene {
        stock: bench
            .stock
            .entries()
            .map(|(key, amount)| SceneStockBottle {
                key: key.to_string(),
                remaining: amount.amount,
                unit: amount.unit.label().to_string(),
            })
            .collect(),
        ..scene_of(&bench.vessels)
    }
}

/// The render model over any vessel slice — for callers that hold vessels
/// without a `Bench` (the CLI/MCP `--json` contract builder).
pub fn scene_of(vessels: &[Vessel]) -> Scene {
    Scene {
        scene: SCENE_VERSION,
        vessels: vessels.iter().map(scene_vessel).collect(),
        stock: Vec::new(),
    }
}

/// The plateau this vessel would be held at, and the one its liquid would
/// freeze at — `(boiling_k, melting_k)`, either or both `None`.
///
/// Water first, and deliberately: it is the solvent the whole phase model
/// in `states` is written around, and where there is liquid water its
/// transitions are the vessel's, colligative elevation and pressure shift
/// already in them. That is the same call `StateEquilibrator` makes, so the
/// standing scene value and the `state_changed.at` a boil reports are one
/// number rather than two that agree by inspection.
///
/// Any other liquid takes the registry's own reviewed normal transition
/// temperatures, with the boiling point shifted for the vessel's pressure
/// by the same cleared correlation the boiling-point apparatus uses
/// (BRD-031/BRD-032), anchored on the curated value so an open flask reads
/// exactly what the book says. Where several non-aqueous liquids share a
/// vessel the most abundant one answers: this is one vessel's plateau, not
/// a phase diagram of a mixture, and a mixture's real boiling range is
/// distillation's question rather than the stage's.
fn liquid_transitions(v: &Vessel) -> (Option<f64>, Option<f64>) {
    const SOLVENT: &str = "water";
    let liquid = |key: &str| -> f64 {
        v.contents
            .iter()
            .filter(|portion| {
                portion.species.0 == key && portion.phase == crate::species::Phase::Liquid
            })
            .map(|portion| portion.moles.0)
            .sum()
    };
    if liquid(SOLVENT) > 0.0 {
        let (transitions, _) = crate::solve::vessel_transitions(v);
        // Past the stated range of the model that supplied the solvent's
        // activity there is no plateau to draw. `StateEquilibrator` refuses
        // the transition in that case and names the boundary; a stage that
        // went on printing the extrapolated number beside the refusal would
        // be the same claim made quietly, which is the one thing the
        // boundary exists to prevent.
        if !transitions.within_model_range() {
            return (None, None);
        }
        // The eutectic boundary is one-sided, and so is this. Below 252 K
        // the liquidus needs a phase diagram this bench does not have —
        // `StateEquilibrator` will not freeze past it and says so — but the
        // BOIL of the same brine is an ordinary answer from a model that is
        // still inside its range, so only the melting point is withheld.
        //
        // A saturated chloride is the case: the relation puts its liquidus
        // at −25.7 °C, four degrees past the boundary the solver refuses to
        // cross, and a stage that drew that plateau would be promising a
        // temperature the bench has already declined to reach. It printed
        // −22.3 °C for the same beaker under the dilute law and was wrong
        // in the same way, only by less (2026-09-11).
        let freezing = (transitions.freezing_k >= crate::states::BRINE_MODEL_MIN_K)
            .then_some(transitions.freezing_k);
        return (Some(transitions.boiling_k), freezing);
    }
    let Some(principal) = v
        .contents
        .iter()
        .filter(|portion| portion.phase == crate::species::Phase::Liquid && portion.moles.0 > 0.0)
        .max_by(|a, b| a.moles.0.total_cmp(&b.moles.0))
    else {
        return (None, None);
    };
    let Some(data) = crate::species::lookup(&principal.species) else {
        return (None, None);
    };
    let Some(transitions) = data.transitions else {
        return (None, None);
    };
    let boiling = transitions.boiling_k.map(|normal| {
        let (shift, route) =
            crate::states::boiling_shift_for_k(data.inchikey, v.pressure.0 / 1000.0);
        if route.routed() {
            normal + shift
        } else {
            normal
        }
    });
    (boiling, transitions.melting_k)
}

/// One vessel's render model.
pub fn scene_vessel(v: &Vessel) -> SceneVessel {
    let seen = appearance::observe(v);
    let (boiling, melting) = liquid_transitions(v);
    let material_layers = crate::material::immiscible_liquid_layers(v);
    let emulsion_observation = crate::emulsion::observe(v);
    let curdling_observation = crate::curdling::observe(v);
    let swelling_observation = crate::swelling::observe(v);
    let gel_observation = crate::gel::observe(v);
    let chemiluminescence_observation = crate::chemiluminescence::observe(v);
    let enzyme_hydrolysis = crate::enzyme_activity::observe(v);
    let partition = standing_partition(v);
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
            spectral_gaps: seen.spectral_gaps.clone(),
        })
        .or_else(|| {
            material_layers.first().map(|layer| SceneLiquid {
                volume_l: material_volume_l,
                srgb: layer.srgb,
                colour_word: layer.colour_word.clone(),
                cloudiness: 0.0,
                path_length_cm: crate::vessel::path_cm_for(&v.label),
                spectral_gaps: seen.spectral_gaps.clone(),
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

    let bulk_observations = crate::material::bulk_solid_objects(v);
    let coatings: Vec<SceneCoating> = crate::corrosion::BARRIERS
        .iter()
        .filter(|barrier| {
            crate::corrosion::barrier_for(v, barrier.metal) == Some(*barrier)
                && v.contents.iter().any(|portion| {
                    portion.species.0 == barrier.metal
                        && portion.phase == Phase::Solid
                        && portion.moles.0 > crate::OBSERVABLE_MOLES
                })
        })
        .map(|barrier| {
            let recipe_id = barrier
                .lot_source
                .strip_prefix("material recipe ")
                .unwrap_or(barrier.lot_source)
                .to_string();
            let (kind, phrase) = if recipe_id == "metal/painted-iron" {
                (
                    "paint",
                    Phrase::bare(
                        "scene.coating-paint",
                        "The painted iron has a complete protective paint film; scratches are not modeled.",
                    ),
                )
            } else {
                (
                    "passive_film",
                    Phrase::bare(
                        "scene.coating-passive-film",
                        "The stainless steel has a transparent protective passive film; its thickness is not drawn to scale.",
                    ),
                )
            };
            SceneCoating {
                kind: kind.to_string(),
                recipe_id,
                host_species: barrier.metal.to_string(),
                words: phrase.render(Locale::EN),
                phrase: Some(phrase),
            }
        })
        .collect();
    let corrosion_verdicts = crate::corrosion::verdicts(v);
    let corrosion: Vec<SceneCorrosion> = crate::corrosion::GATED_REACTIONS
        .iter()
        .filter_map(|(_, metal)| {
            let (locked, fraction) = crate::corrosion::corroded_extent(v, metal)?;
            let corroding = corrosion_verdicts
                .iter()
                .find(|verdict| verdict.metal == *metal)
                .is_some_and(|verdict| verdict.corroding);
            let phrase = Phrase::new(
                "scene.corrosion",
                "{percent}% of the tracked {metal} is currently locked in its modeled oxide. Oxide added directly is indistinguishable from oxide formed here; this is not a corrosion rate, history, thickness, or surface coverage.",
                vec![
                    (
                        "percent".to_string(),
                        Slot::number(format!("{:.0}", fraction * 100.0)),
                    ),
                    (
                        "metal".to_string(),
                        Slot::term(
                            "species",
                            species::lookup(&crate::SpeciesId::new(metal))
                                .map(|data| data.name)
                                .unwrap_or(*metal),
                        ),
                    ),
                ],
            );
            Some(SceneCorrosion {
                metal: (*metal).to_string(),
                corroding,
                metal_in_oxide_moles: locked.0,
                metal_in_oxide_fraction: fraction,
                words: phrase.render(Locale::EN),
                phrase: Some(phrase),
            })
        })
        .collect();
    let adsorption: Vec<SceneAdsorption> = crate::adsorption::standing(v)
        .into_iter()
        .map(|row| SceneAdsorption {
            sorbent: row.sorbent,
            sorbate: row.sorbate,
            held_mg: row.held_mg,
            still_dissolved_mg: row.still_dissolved_mg,
            held_fraction: row.held_fraction,
            loading_mg_per_g: row.loading_mg_per_g,
            loading_fraction: row.loading_fraction,
            boundary: row.boundary,
            provenance: row.provenance,
        })
        .collect();
    let bulk_component_keys: std::collections::BTreeSet<String> = bulk_observations
        .iter()
        .flat_map(|object| {
            crate::material::lookup_versioned(&object.recipe_id, 1)
                .into_iter()
                .flat_map(|recipe| recipe.components.into_iter().map(|part| part.species_id))
        })
        .collect();

    // Aggregate solids per species, keeping first-seen order, then sort by
    // amount so the biggest deposit paints first.
    let mut solids: Vec<SceneSolid> = Vec::new();
    for p in v.contents.iter().filter(|p| p.phase == Phase::Solid) {
        let data = species::lookup(&p.species);
        let volume_l = data
            .filter(|species| species.density.is_finite() && species.density > 0.0)
            .map(|species| species.liters_from_moles(crate::Moles(p.moles.0)).0)
            .unwrap_or(0.0);
        if let Some(existing) = solids.iter_mut().find(|s| s.species == p.species.0) {
            existing.moles += p.moles.0;
            existing.volume_l += volume_l;
            continue;
        }
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
            volume_l,
            srgb: [colour.r, colour.g, colour.b],
            colour_word: colour_word(&colour, true).to_string(),
            metallic: crate::displacement::is_elemental_metal(&p.species.0),
            settled_fraction: v
                .suspended_fraction_of(&p.species)
                .map(|fraction| 1.0 - fraction)
                .unwrap_or(1.0),
            represented_by_bulk_object: bulk_component_keys.contains(&p.species.0),
        });
    }
    solids.sort_by(|a, b| b.moles.total_cmp(&a.moles));

    let liquid_density = crate::buoyancy::liquid_density_g_per_ml(v);
    let conserved = crate::material::conserved_unresolved_solids(v);
    let bulk_objects = bulk_observations
        .into_iter()
        .map(|object| {
            let srgb = conserved
                .iter()
                .find(|solid| solid.recipe_id == object.recipe_id)
                .map(|solid| solid.srgb)
                .or_else(|| {
                    crate::material::lookup_versioned(&object.recipe_id, 1).and_then(|recipe| {
                        recipe.components.iter().find_map(|component| {
                            species::lookup(&crate::SpeciesId(component.species_id.clone()))
                                .and_then(|data| data.colour)
                                .map(|colour| [colour.r, colour.g, colour.b])
                        })
                    })
                })
                .unwrap_or([176, 160, 128]);
            let position = match liquid_density {
                Some(liquid) if object.bulk_density_g_per_ml < liquid => "floating",
                Some(_) => "sunk",
                None => "dry",
            };
            SceneBulkObject {
                material: object.material,
                recipe_id: object.recipe_id,
                amount_g: object.amount,
                bulk_density_g_per_ml: object.bulk_density_g_per_ml,
                position: position.to_string(),
                srgb,
            }
        })
        .collect();

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
    let material_objects: Vec<_> = v
        .material_objects
        .iter()
        .map(|object| {
            let is_osmotic = crate::material::lookup(&object.material, None).is_some_and(|recipe| {
                    recipe.roles.iter().any(|role| {
                        matches!(role, crate::material::MaterialRole::OsmoticMembrane { .. })
                    })
                });
            let water_moles = object.state.exchanged_water_moles;
            let osmosis = is_osmotic.then(|| SceneOsmosis {
                water_moles,
                // The second of the two fallbacks; see `bench.rs` for why
                // neither survives. This is the readout side of the same
                // exchange that computes the mass there, so the two must
                // agree by construction rather than by both happening to
                // find the registry.
                mass_change_g: water_moles * crate::constants::WATER_MOLAR_MASS_G_PER_MOL,
                direction: if water_moles > 1e-15 {
                    "into_object"
                } else if water_moles < -1e-15 {
                    "out_of_object"
                } else {
                    "balanced"
                }
                .to_string(),
                basis: "stored cumulative output of the bounded osmolarity-gradient teaching model (20% water exchange per 1 osmol/L contrast over a 24 h exponential timescale, capped at 40% per step); not object size, membrane mechanics, ion selectivity, or final equilibrium"
                    .to_string(),
            });
            SceneMaterialObject {
                material: object.material.clone(),
                recipe_id: object.recipe_id.clone(),
                mass_g: object.mass_g,
                exchanged_water_moles: water_moles,
                browned_fraction: object.state.browned_fraction,
                osmosis,
            }
        })
        .collect();

    // I18N-11: every sentence below used to be a bare `format!` pushed onto
    // the finished English `words`, and the web bench paints from the scene
    // — so `appearance.rs` speaking German since I18N-7 bought a German
    // reader nothing here. They are clauses with named holes now, exactly
    // as #626 established, and the English is GENERATED from them rather
    // than written beside them: one sentence, not two to drift.
    let clauses = seen.clauses;
    let mut notes = seen.notes;
    for object in &material_objects {
        if let Some(osmosis) = &object.osmosis {
            if osmosis.water_moles.abs() <= 1e-15 {
                continue;
            }
            let slots = vec![
                (
                    "water".to_string(),
                    Slot::number(format!("{:.4}", osmosis.water_moles.abs())),
                ),
                (
                    "mass".to_string(),
                    Slot::number(format!("{:.3}", osmosis.mass_change_g.abs())),
                ),
                (
                    "material".to_string(),
                    Slot::term("material", object.material.clone()),
                ),
            ];
            // Three templates rather than one with a translated preposition:
            // German takes a different case for water going IN and water
            // coming OUT, and "into" is not a word that can be swapped in
            // place. The direction is a property of the sentence.
            notes.push(match osmosis.direction.as_str() {
                "into_object" => Phrase::new(
                    "scene.osmosis-in",
                    "Osmosis has moved {water} mol ({mass} g) of water into the {material} cumulatively in the teaching model; object size, membrane mechanics, and final equilibrium are not modeled.",
                    slots,
                ),
                "out_of_object" => Phrase::new(
                    "scene.osmosis-out",
                    "Osmosis has moved {water} mol ({mass} g) of water out of the {material} cumulatively in the teaching model; object size, membrane mechanics, and final equilibrium are not modeled.",
                    slots,
                ),
                _ => Phrase::new(
                    "scene.osmosis-either",
                    "Osmosis has moved {water} mol ({mass} g) of water into or out of the {material} cumulatively in the teaching model; object size, membrane mechanics, and final equilibrium are not modeled.",
                    slots,
                ),
            });
        }
    }
    if let Some(gel) = &gel_observation {
        notes.push(Phrase::new(
            "scene.gel",
            "A translucent cohesive gel contains {percent}% of the {polymer} polymer.",
            vec![
                (
                    "percent".to_string(),
                    Slot::number(format!("{:.0}", gel.gelled_fraction * 100.0)),
                ),
                ("polymer".to_string(), Slot::term("polymer", gel.polymer)),
            ],
        ));
    }
    notes.extend(coatings.iter().filter_map(|c| c.phrase.clone()));
    notes.extend(corrosion.iter().filter_map(|c| c.phrase.clone()));
    if let Some(swelling) = &swelling_observation {
        notes.push(Phrase::new(
            "scene.swelling",
            "The superabsorbent network retains {water} g of water ({ratio} times its dry mass).",
            vec![
                (
                    "water".to_string(),
                    Slot::number(format!("{:.1}", swelling.retained_water_g)),
                ),
                (
                    "ratio".to_string(),
                    Slot::number(format!("{:.1}", swelling.swelling_ratio_g_per_g)),
                ),
            ],
        ));
    }
    if let Some(glow) = &chemiluminescence_observation {
        notes.push(Phrase::new(
            "scene.chemiluminescence",
            "The luminol system is glowing blue at relative intensity {intensity}; its estimated half-life here is {half_life} seconds.",
            vec![
                (
                    "intensity".to_string(),
                    Slot::number(format!("{:.2}", glow.relative_intensity)),
                ),
                (
                    "half_life".to_string(),
                    Slot::number(format!("{:.1}", glow.half_life_s)),
                ),
            ],
        ));
    }
    for progress in &enzyme_hydrolysis {
        notes.push(Phrase::new(
            "scene.enzyme-conversion",
            "The bounded enzyme model reports {percent}% conversion of {substrate} in {material}.",
            vec![
                (
                    "percent".to_string(),
                    Slot::number(format!("{:.0}", progress.converted_fraction * 100.0)),
                ),
                (
                    "substrate".to_string(),
                    Slot::term("substrate", progress.substrate),
                ),
                (
                    "material".to_string(),
                    Slot::term("material", progress.material.clone()),
                ),
            ],
        ));
    }
    if !v.surface_colours.is_empty() {
        let spread = v
            .surface_colours
            .iter()
            .map(|spot| spot.spread_fraction)
            .fold(0.0, f64::max);
        notes.push(if spread > 0.01 {
            Phrase::bare(
                "scene.food-colour-spread",
                "Food-colour streaks have spread across the milk surface.",
            )
        } else {
            Phrase::bare(
                "scene.food-colour-resting",
                "Food-colour drops are resting on the milk surface.",
            )
        });
    }
    for row in &adsorption {
        notes.push(Phrase::new(
            "scene.adsorption",
            "{percent}% of the tracked {sorbate} is held on {sorbent}; {dissolved} mg remains dissolved. This is an equilibrium split, not a removal rate.",
            vec![
                (
                    "percent".to_string(),
                    Slot::number(format!("{:.0}", row.held_fraction * 100.0)),
                ),
                (
                    "sorbate".to_string(),
                    Slot::term(
                        "species",
                        species::lookup(&crate::SpeciesId::new(&row.sorbate))
                            .map(|data| data.name)
                            .unwrap_or(&row.sorbate),
                    ),
                ),
                (
                    "sorbent".to_string(),
                    Slot::term(
                        "species",
                        species::lookup(&crate::SpeciesId::new(&row.sorbent))
                            .map(|data| data.name)
                            .unwrap_or(&row.sorbent),
                    ),
                ),
                (
                    "dissolved".to_string(),
                    Slot::number(format!("{:.2}", row.still_dissolved_mg)),
                ),
            ],
        ));
    }
    for split in &partition {
        notes.push(Phrase::new(
            "scene.partition",
            "At equilibrium, {lower_percent}% of the modeled {species} is in the lower {lower} layer and {upper_percent}% is in the upper {upper} layer.",
            vec![
                (
                    "lower_percent".to_string(),
                    Slot::number(format!("{:.0}", split.fraction_lower * 100.0)),
                ),
                (
                    "species".to_string(),
                    Slot::term(
                        "species",
                        species::lookup(&crate::SpeciesId::new(&split.species))
                            .map(|data| data.name)
                            .unwrap_or(&split.species),
                    ),
                ),
                (
                    "lower".to_string(),
                    Slot::term(
                        "species",
                        species::lookup(&crate::SpeciesId::new(&split.lower_solvent))
                            .map(|data| data.name)
                            .unwrap_or(&split.lower_solvent),
                    ),
                ),
                (
                    "upper_percent".to_string(),
                    Slot::number(format!("{:.0}", (1.0 - split.fraction_lower) * 100.0)),
                ),
                (
                    "upper".to_string(),
                    Slot::term(
                        "species",
                        species::lookup(&crate::SpeciesId::new(&split.upper_solvent))
                            .map(|data| data.name)
                            .unwrap_or(&split.upper_solvent),
                    ),
                ),
            ],
        ));
    }
    if let Some(emulsion) = &emulsion_observation {
        notes.push(Phrase::new(
            "scene.emulsion",
            "Stirring has dispersed {percent}% of the {material} as cloudy droplets; the rest remains above the water.",
            vec![
                (
                    "percent".to_string(),
                    Slot::number(format!("{:.0}", emulsion.dispersed_fraction * 100.0)),
                ),
                (
                    "material".to_string(),
                    Slot::term("material", emulsion.material.clone()),
                ),
            ],
        ));
    } else if let Some(layer) = material_layers.first() {
        if resolved_volume_l > 0.0 {
            notes.push(Phrase::new(
                "scene.material-layer",
                "{material} forms a separate {colour} layer above the water.",
                vec![
                    (
                        "material".to_string(),
                        Slot::term("material", layer.material.clone()),
                    ),
                    (
                        "colour".to_string(),
                        Slot::term("appearance", layer.colour_word.clone()),
                    ),
                ],
            ));
        } else {
            notes.push(Phrase::new(
                "scene.vessel-contains",
                "The vessel contains {material}.",
                vec![(
                    "material".to_string(),
                    Slot::term("material", layer.material.clone()),
                )],
            ));
        }
    }
    if let Some(curds) = &curdling_observation {
        notes.push(Phrase::new(
            "scene.curdling",
            "Soft curds containing {mass} g of modeled aggregate solids have separated from the {material} into cloudy whey.",
            vec![
                (
                    "mass".to_string(),
                    Slot::number(format!("{:.2}", curds.curd_solids_mass_g)),
                ),
                (
                    "material".to_string(),
                    Slot::term("material", curds.material.clone()),
                ),
            ],
        ));
    }
    if let Some(foam) = &foam {
        notes.push(if foam.overflow_liters > 0.0 {
            Phrase::bare("scene.foam-overflow", "Foam is spilling over the rim.")
        } else {
            Phrase::bare("scene.foam-standing", "Foam is standing above the liquid.")
        });
    }
    let words = say(&clauses, &notes, Locale::EN);

    SceneVessel {
        id: v.id,
        label: v.label.clone(),
        liquid,
        layers,
        solids,
        bulk_objects,
        coatings,
        corrosion,
        adsorption,
        partition,
        material_objects,
        soap_scum: v.soap_scum.as_ref().map(|scum| SceneSoapScum {
            aggregate_mass_g: scum.aggregate_mass_g,
            divalent_ion_moles: scum.divalent_ion_moles,
        }),
        lemon_paper_mark: v.lemon_paper_mark.as_ref().map(|mark| SceneLemonPaperMark {
            dry: mark.dry,
            browned_fraction: mark.browned_fraction,
        }),
        gel: gel_observation.map(|gel| SceneGel {
            polymer: gel.polymer.to_string(),
            crosslinker: gel.crosslinker.to_string(),
            gelled_fraction: gel.gelled_fraction,
            polymer_grams: gel.polymer_grams,
            crosslinker_moles: gel.crosslinker_moles,
        }),
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
        surface_colours: {
            let max_moles = v
                .surface_colours
                .iter()
                .map(|spot| spot.moles.0)
                .fold(0.0, f64::max)
                .max(1e-30);
            v.surface_colours
                .iter()
                .map(|spot| SceneSurfaceColour {
                    material: spot.material.clone(),
                    srgb: spot.srgb,
                    spread_fraction: spot.spread_fraction,
                    relative_amount: (spot.moles.0 / max_moles).clamp(0.0, 1.0),
                })
                .collect()
        },
        emulsion: emulsion_observation.map(|emulsion| SceneEmulsion {
            material: emulsion.material,
            dispersed_volume_l: emulsion.dispersed_volume_l,
            dispersed_fraction: emulsion.dispersed_fraction,
            half_life_seconds: emulsion.half_life_seconds,
        }),
        curds: curdling_observation.map(|curds| SceneCurds {
            material: curds.material,
            formed_fraction: curds.formed_fraction,
            separation_progress: curds.separation_progress,
            solids_mass_g: curds.curd_solids_mass_g,
            srgb: curds.curd_srgb,
        }),
        swelling: swelling_observation.map(|seen| SceneSwelling {
            dry_polymer_g: seen.dry_polymer_g,
            retained_water_g: seen.retained_water_g,
            swelling_ratio_g_per_g: seen.swelling_ratio_g_per_g,
            capacity_g_per_g: seen.capacity_g_per_g,
            saturated: seen.saturated,
        }),
        chemiluminescence: chemiluminescence_observation.map(|seen| SceneChemiluminescence {
            relative_intensity: seen.relative_intensity,
            half_life_s: seen.half_life_s,
            elapsed_s: seen.elapsed_s,
            temperature_k: seen.temperature_k,
        }),
        enzyme_hydrolysis: enzyme_hydrolysis
            .into_iter()
            .map(|seen| SceneEnzymeHydrolysis {
                family: seen.family,
                material: seen.material,
                substrate: seen.substrate.to_string(),
                converted_fraction: seen.converted_fraction,
            })
            .collect(),
        electrodes: v
            .electrodes
            .iter()
            .map(|electrode| SceneElectrode {
                label: electrode.label.clone(),
                material: electrode.material.clone(),
                surface_preparation: electrode.surface_preparation.clone(),
                geometric_area_m2: electrode.area_m2,
                roughness: electrode.roughness,
                interfacial_potential_v: electrode.interfacial_potential_v,
                diagnostics: electrode.diagnostics.clone(),
                surface_species: electrode
                    .interfacial_species
                    .iter()
                    .map(|state| SceneElectrodeSurfaceSpecies {
                        reaction_id: state.reaction_id.clone(),
                        species: state.species.clone(),
                        concentration_mol_per_m3: state.surface_concentration_mol_per_m3,
                    })
                    .collect(),
                deposits: electrode
                    .deposits
                    .iter()
                    .map(|deposit| SceneElectrodeDeposit {
                        species: deposit.species.clone(),
                        moles: deposit.moles,
                        thickness_m: deposit.thickness_m,
                        coverage_fraction: deposit.coverage_fraction,
                    })
                    .collect(),
            })
            .collect(),
        headspace: v.headspace,
        headspace_volume_l: v.headspace_volume().map(|litres| litres.0),
        headspace_moles: v.headspace_volume().map(|_| v.gas_moles().0),
        boiling_point_k: boiling,
        melting_point_k: melting,
        temperature_k: v.temperature.0,
        pressure_pa: v.pressure.0,
        elapsed_s: v.elapsed_seconds,
        mass_g: v.mass().0,
        words,
        clauses,
        notes,
        badges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::MaterialBasis;
    use crate::vessel::{MaterialLot, UnresolvedMaterialPortion};
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

    /// GUI-119. The engine has always known which species it could not
    /// give a colour to, and has always said so in the prose. The scene
    /// dropped it, so the picture painted a confident colour over a gap.
    ///
    /// Both directions, because only one of them is the defect: a whole
    /// colour must carry no gap, and the same vessel with an uncomputable
    /// complex in it must carry the complex by name.
    #[test]
    fn an_incomplete_colour_reaches_the_scene_naming_what_is_missing() {
        use crate::vessel::{SolutionInfo, SpeciesDetail};
        let mut v = vessel_with(&[
            ("water", 5.55, Phase::Liquid),
            ("Cu+2", 0.01, Phase::Aqueous),
        ]);
        let whole = scene_vessel(&v).liquid.expect("a liquid");
        assert!(
            whole.spectral_gaps.is_empty(),
            "a colour the model computed whole admits no gap: {:?}",
            whole.spectral_gaps
        );
        assert!(
            !serde_json::to_value(&whole).unwrap()["spectral_gaps"].is_array(),
            "an empty list stays off the wire, so an older host sees what it saw"
        );

        // The ammine complex carries almost all the copper and has no
        // registered spectrum, so the colour above is most of the copper
        // missing. `solution_optics::spectral_gaps` has reported this for
        // as long as it has existed.
        v.solution = Some(SolutionInfo {
            solvent_activity: None,
            scope: Default::default(),
            solvent_kg: Some(0.1),
            ph: 9.0,
            pe: None,
            redox: vec![],
            ionic_strength: 0.1,
            provenance: None,
            species: vec![
                SpeciesDetail {
                    name: "Cu+2".into(),
                    molality: 0.001,
                    activity: 0.0005,
                },
                SpeciesDetail {
                    name: "Cu(NH3)4+2".into(),
                    molality: 0.099,
                    activity: 0.05,
                },
            ],
        });
        let gapped = scene_vessel(&v).liquid.expect("a liquid");
        assert_eq!(
            gapped.spectral_gaps,
            vec!["Cu(NH3)4+2".to_string()],
            "the scene names what the prose names"
        );
        // And it is still only an admission. The engine did not invent the
        // missing absorbance: it painted the tenth of the copper it could
        // still see, so the colour got PALER rather than gaining a tint
        // nobody computed.
        assert!(
            gapped.srgb.iter().copied().map(u32::from).sum::<u32>()
                >= whole.srgb.iter().copied().map(u32::from).sum::<u32>(),
            "the uncomputable share was left out, not guessed at"
        );
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
            "bulk_objects",
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
            "volume_l",
            "srgb",
            "colour_word",
            "metallic",
            "settled_fraction",
            "represented_by_bulk_object",
        ] {
            assert!(solid.get(key).is_some(), "missing solid key {key}");
        }
        let agcl = species::lookup(&crate::SpeciesId("AgCl".into())).unwrap();
        assert!(
            (solid["volume_l"].as_f64().unwrap() - agcl.liters_from_moles(crate::Moles(0.01)).0)
                .abs()
                < 1e-12
        );
        // And it round-trips.
        let back: SceneVessel = serde_json::from_value(json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn material_provenance_projects_protective_coatings() {
        let mut painted = vessel_with(&[]);
        painted.deposit_lot(
            crate::SpeciesId::new("Fe"),
            crate::Moles(0.1),
            Phase::Solid,
            Some("material recipe metal/painted-iron".into()),
            None,
        );
        let scene = scene_vessel(&painted);
        assert_eq!(scene.coatings.len(), 1);
        assert_eq!(scene.coatings[0].kind, "paint");
        assert_eq!(scene.coatings[0].recipe_id, "metal/painted-iron");
        assert!(scene.words.contains("complete protective paint film"));

        let mut stainless = vessel_with(&[]);
        stainless.deposit_lot(
            crate::SpeciesId::new("Fe"),
            crate::Moles(0.1),
            Phase::Solid,
            Some("material recipe metal/stainless-steel".into()),
            None,
        );
        assert_eq!(scene_vessel(&stainless).coatings[0].kind, "passive_film");
    }

    #[test]
    fn coating_projection_fails_closed_for_bare_or_mixed_iron() {
        let bare = vessel_with(&[("Fe", 0.1, Phase::Solid)]);
        assert!(scene_vessel(&bare).coatings.is_empty());

        let mut mixed = vessel_with(&[]);
        mixed.deposit_lot(
            crate::SpeciesId::new("Fe"),
            crate::Moles(0.1),
            Phase::Solid,
            None,
            None,
        );
        mixed.deposit_lot(
            crate::SpeciesId::new("Fe"),
            crate::Moles(0.1),
            Phase::Solid,
            Some("material recipe metal/painted-iron".into()),
            None,
        );
        assert!(scene_vessel(&mixed).coatings.is_empty());
    }

    #[test]
    fn older_scene_json_without_coatings_still_deserializes() {
        let mut json = serde_json::to_value(scene_vessel(&vessel_with(&[]))).unwrap();
        json.as_object_mut().unwrap().remove("coatings");
        let old: SceneVessel = serde_json::from_value(json).unwrap();
        assert!(old.coatings.is_empty());
    }

    #[test]
    fn older_scene_json_without_adsorption_still_deserializes() {
        let mut json = serde_json::to_value(scene_vessel(&vessel_with(&[]))).unwrap();
        json.as_object_mut().unwrap().remove("adsorption");
        let old: SceneVessel = serde_json::from_value(json).unwrap();
        assert!(old.adsorption.is_empty());
    }

    #[test]
    fn older_scene_json_without_partition_still_deserializes() {
        let mut json = serde_json::to_value(scene_vessel(&vessel_with(&[]))).unwrap();
        json.as_object_mut().unwrap().remove("partition");
        let old: SceneVessel = serde_json::from_value(json).unwrap();
        assert!(old.partition.is_empty());
    }

    #[test]
    fn swelling_is_persistent_scene_state_with_accessible_words() {
        let mut v = vessel_with(&[(
            "water",
            50.0 / crate::constants::WATER_MOLAR_MASS_G_PER_MOL,
            Phase::Liquid,
        )]);
        v.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "instant snow".into(),
            recipe_id: crate::swelling::RECIPE_ID.into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 0.5,
            protein_denatured_fraction: 0.0,
            enzyme_hydrolysis: None,
        });
        let scene = scene_vessel(&v);
        let swelling = scene.swelling.expect("swelling render target");
        assert!((swelling.retained_water_g - 50.0).abs() < 0.01);
        assert!((swelling.swelling_ratio_g_per_g - 100.0).abs() < 0.1);
        assert!(scene.words.contains("superabsorbent network retains"));
    }

    #[test]
    fn chemiluminescence_scene_tracks_temperature_and_elapsed_time() {
        let mut v = vessel_with(&[("H2O2", 0.002, Phase::Liquid)]);
        v.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "luminol glow solution".into(),
            recipe_id: crate::chemiluminescence::RECIPE_ID.into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 20.0,
            protein_denatured_fraction: 0.0,
            enzyme_hydrolysis: None,
        });
        v.lots.push(MaterialLot {
            species: SpeciesId::new("H2O2"),
            moles: Moles(0.002),
            phase: Phase::Liquid,
            added_at: 0.0,
            hydrated_at: None,
            source: None,
            particle_size_um: None,
            suspended_fraction: None,
        });
        v.temperature = crate::Kelvin(293.15);
        v.elapsed_seconds = 60.0;
        let scene = scene_vessel(&v);
        let glow = scene
            .chemiluminescence
            .expect("chemiluminescence render target");
        assert!((glow.relative_intensity - 0.5).abs() < 1e-9);
        assert_eq!(glow.elapsed_s, 60.0);
        assert!(scene.words.contains("glowing blue"));
    }

    #[test]
    fn enzyme_conversion_is_persistent_readout_not_a_fake_texture() {
        let mut v = vessel_with(&[("water", 5.55, Phase::Liquid)]);
        v.unresolved_materials.push(UnresolvedMaterialPortion {
            material: "whole milk".into(),
            recipe_id: "household/whole-milk-surrogate".into(),
            recipe_version: 1,
            basis: MaterialBasis::MassFraction,
            amount: 13.0,
            protein_denatured_fraction: 0.0,
            enzyme_hydrolysis: Some(crate::vessel::EnzymeHydrolysisState {
                family: crate::enzyme::EnzymeFamily::Lactase,
                converted_fraction: 0.625,
                carried_enzyme_denatured: false,
            }),
        });
        let scene = scene_vessel(&v);
        assert_eq!(scene.enzyme_hydrolysis.len(), 1);
        let reading = &scene.enzyme_hydrolysis[0];
        assert_eq!(reading.substrate, "lactose in milk");
        assert!((reading.converted_fraction - 0.625).abs() < 1e-12);
        assert!(scene.words.contains("% conversion of lactose in milk"));
    }

    #[test]
    fn additive_observation_fields_default_when_old_scene_is_read() {
        let mut json = serde_json::to_value(scene_vessel(&vessel_with(&[]))).unwrap();
        json.as_object_mut().unwrap().remove("swelling");
        json.as_object_mut().unwrap().remove("chemiluminescence");
        json.as_object_mut().unwrap().remove("gel");
        json.as_object_mut().unwrap().remove("enzyme_hydrolysis");
        json.as_object_mut().unwrap().remove("electrodes");
        let old: SceneVessel = serde_json::from_value(json).unwrap();
        assert!(old.swelling.is_none());
        assert!(old.chemiluminescence.is_none());
        assert!(old.gel.is_none());
        assert!(old.enzyme_hydrolysis.is_empty());
        assert!(old.electrodes.is_empty());
    }

    #[test]
    fn electrode_authority_projects_only_persistent_engine_state() {
        let mut vessel = vessel_with(&[]);
        vessel.electrodes.push(crate::compartment::ElectrodeState {
            label: "working".into(),
            material: "Pt".into(),
            surface_preparation: Some("polished".into()),
            area_m2: 2.5e-4,
            roughness: 1.2,
            double_layer_capacitance_f_per_m2: Some(0.2),
            interfacial_potential_v: Some(0.315),
            interfacial_species: vec![crate::compartment::ElectrodeInterfacialSpecies {
                reaction_id: "hydrogen".into(),
                species: "H+".into(),
                surface_concentration_mol_per_m3: 12.5,
            }],
            ..Default::default()
        });
        let scene = scene_vessel(&vessel);
        let electrode = scene.electrodes.first().expect("electrode projection");
        assert_eq!(electrode.label, "working");
        assert_eq!(electrode.interfacial_potential_v, Some(0.315));
        assert_eq!(electrode.surface_species[0].species, "H+");
        assert_eq!(electrode.surface_species[0].concentration_mol_per_m3, 12.5);
    }

    #[test]
    fn gel_scene_is_derived_from_current_inventory_with_accessible_words() {
        let v = vessel_with(&[
            ("PVA", 0.25, Phase::Solid),
            ("Na2B4O7", 0.001, Phase::Liquid),
        ]);
        let observed = crate::gel::observe(&v).expect("gel observation");
        let scene = scene_vessel(&v);
        let gel = scene.gel.expect("persistent gel render target");
        assert_eq!(gel.polymer, observed.polymer);
        assert_eq!(gel.crosslinker, observed.crosslinker);
        assert!((gel.gelled_fraction - observed.gelled_fraction).abs() < 1e-12);
        assert!((gel.polymer_grams - observed.polymer_grams).abs() < 1e-12);
        assert!((gel.crosslinker_moles - observed.crosslinker_moles).abs() < 1e-12);
        assert!(scene.words.contains("translucent cohesive gel"));
        assert!(scene.words.contains("PVA polymer"));
    }

    #[test]
    fn polymer_without_crosslinker_has_no_gel_scene() {
        let scene = scene_vessel(&vessel_with(&[("PVA", 0.25, Phase::Solid)]));
        assert!(scene.gel.is_none());
    }
}

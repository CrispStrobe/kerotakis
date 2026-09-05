//! BRD-023 / BRD-014: rusting, galvanic couples and barrier coatings —
//! which metal corrodes, what spares the others, and how fast.
//!
//! Until this module the bench had one sentence for a nail in a glass of
//! water, and `displacement::bystanders` wrote it: *"iron stays as the
//! metal: nothing dissolved here sits below it in the activity series.
//! Its slow reaction with water itself … is a rate this lab does not
//! model."* That was honest and it was the wrong question. Iron in a
//! glass of water is not waiting for something to displace; it is
//! corroding, and the cathode is the dissolved oxygen, which the activity
//! series never looks at. Ten prompts in the curiosity corpus asked about
//! rust and got that apology back.
//!
//! # The model
//!
//! One corrosion cell per vessel, stated as four claims.
//!
//! **1. Rusting needs three things at once.** A metal, liquid water, and
//! oxygen. The anodic half `Fe → Fe²⁺ + 2 e⁻` and the cathodic half
//! `O₂ + 2 H₂O + 4 e⁻ → 4 OH⁻` are two halves of one circuit, and the
//! water is both the ionic conductor between them and a reagent in the
//! second. Take any one away and the circuit does not turn over. So an
//! oxygen-free vessel gets a typed *no corrosion, and here is which of
//! the three is missing* rather than a stand-aside, and that negative is
//! as much a result as the positive.
//!
//! **2. The lowest-E° metal in contact is the anode.** Two metals
//! touching in an electrolyte are a battery, and the battery decides
//! which one dissolves: the less noble one gives up its electrons for
//! both. That single rule is galvanising (zinc below iron, so the zinc
//! goes first and keeps going after the coat is scratched), the
//! sacrificial anode on a hull, and the reason a copper fitting on a
//! steel pipe eats the steel. The potentials are `displacement::SERIES`
//! — the same CRC table the displacement route already computes with,
//! because a bench must not hold two activity series.
//!
//! **3. Salt speeds it up, and there is a ceiling.** The cathodic
//! reaction is limited by how fast oxygen can reach the surface, which
//! Levich gives as `i_L = n F D c / δ` — about 42 µA/cm² for quiescent
//! air-saturated water at 25 °C. That is the ceiling. Below it the cell
//! is throttled by the ohmic resistance of the electrolyte between the
//! anodic and the cathodic patch, and that is what dissolved salt
//! changes: [`ohmic_throttle`] interpolates monotonically from ~0 in
//! distilled water to ~1 in brine. So a nail in distilled water barely
//! rusts, a nail in tap water rusts, and a nail in sea water rusts about
//! twice as fast as in tap water — which is the school result, including
//! the part that surprises people, that brine is not ten times faster.
//!
//! **4. A barrier stops it, and a barrier is a claim about an object.**
//! Stainless steel carries a chromium(III) oxide film; painted iron
//! carries a paint film. Both are properties of the *object* rather than
//! of the iron in it, so [`BARRIERS`] is keyed on the material recipe the
//! lot came from, and applies only when every lot of that metal in the
//! vessel arrived under a barrier.
//!
//! # What this route does not claim
//!
//! * **It does not move the inventory.** Mass loss is current × area ×
//!   time and this bench has no surface area for a nail — every recipe's
//!   `surface_area_m2` is `null`. A rate per unit area is the quantity
//!   that survives that ignorance, so the route reports µA/cm² and mm/yr
//!   and consumes nothing. Inventing an area would make the assumed area,
//!   not the chemistry, the whole answer.
//! * **The rate is the diffusion-limited ceiling of a bare surface.** Real
//!   mild steel in sea water settles near 0.1–0.15 mm/yr, well under the
//!   ~0.5 mm/yr computed here, because the rust film itself becomes the
//!   oxygen barrier within days. That film is not modelled, so the number
//!   is an upper bound on a freshly exposed surface and is labelled as one.
//! * **No pitting, no crevice corrosion, no stress corrosion, no area
//!   ratio.** A small anode against a large cathode corrodes far faster
//!   than a large one, and this route has no areas to form the ratio with.
//! * **No atmospheric weathering.** The green of an old copper contact is
//!   a patina of basic copper carbonate and sulfate, built over years from
//!   CO₂ and SO₂ in the air. There is no gaseous CO₂/SO₂ weathering route
//!   on this bench and the module says so rather than inventing one.
//! * **Acid corrosion belongs to `displacement`.** Where free acidity is
//!   present the cathode is `2 H⁺ + 2 e⁻ → H₂`, not oxygen, and the
//!   displacement route already computes it with its own overpotential
//!   gate. This route stands aside there so the two never both speak.
//! * **Temperature is not in the rate.** `D`, `c` and `δ` are all quoted
//!   at 25 °C and used at the vessel temperature uncorrected.

use crate::displacement::{Couple, SERIES};
use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError, SolverRouteKind};
use crate::species::{self, Phase, SpeciesId};
use crate::vessel::{Headspace, Vessel};

/// Solubility of oxygen in water in equilibrium with air, mol/L, at
/// 25 °C and 1 atm.
///
/// 8.26 mg/L over 31.999 g/mol. Source: the IUPAC Solubility Data Series
/// evaluation of oxygen in water (Battino, ed., vol. 7), which is the
/// tabulation the familiar "about 8 mg/L" of every water-quality manual
/// comes from.
pub const OXYGEN_SOLUBILITY_MOL_PER_L: f64 = 2.58e-4;

/// Diffusion coefficient of dissolved oxygen in water, cm²/s, at 25 °C.
///
/// Source: CRC Handbook of Chemistry and Physics, diffusion coefficients
/// of gases in water; the reviewed range is 2.0–2.4 × 10⁻⁵ cm²/s and the
/// value used is the low-middle of it.
pub const OXYGEN_DIFFUSIVITY_CM2_PER_S: f64 = 2.1e-5;

/// Nernst diffusion-layer thickness for a quiescent, naturally
/// convecting solution, cm.
///
/// Editorial judgement (Kerotakis): 0.05 cm is the standard teaching
/// figure for natural convection (Fontana, *Corrosion Engineering*), and
/// it is the single softest number in this module. Stirring thins the
/// layer and raises the limiting current in direct proportion; this bench
/// has a `stir` verb and the corrosion route does not read it, so a
/// stirred beaker reports the quiescent rate.
pub const DIFFUSION_LAYER_CM: f64 = 0.05;

/// The conductivity at which the corrosion cell reaches half its
/// diffusion-limited current, µS/cm.
///
/// Editorial judgement (Kerotakis), and the model form is the claim
/// rather than the constant. A corrosion cell is an anodic patch and a
/// cathodic patch in series through the electrolyte, so its current is
/// limited by whichever resistance is larger: oxygen transport at the
/// cathode, or the ohmic path between the two. Writing that as
/// `i = i_L · κ/(κ + κ½)` gives the one behaviour the school experiment
/// is about — more dissolved salt, faster rusting, monotonically, up to a
/// ceiling it cannot pass. `κ½` is set at ordinary tap water (≈500 µS/cm)
/// so distilled water (≈1 µS/cm) is throttled to under a percent of the
/// ceiling, tap water sits at half, and sea water (≈50 000 µS/cm) is
/// within a percent of it.
///
/// What it is not: a fitted parameter. Real ohmic control depends on the
/// separation of the anodic and cathodic sites, which is geometry this
/// bench does not have. The direction and the saturation are the claim;
/// the half-point is a stated choice.
pub const OHMIC_HALF_CONDUCTIVITY_US_PER_CM: f64 = 500.0;

/// Seconds in a Julian year, for penetration rates quoted per year.
const SECONDS_PER_YEAR: f64 = 31_557_600.0;

/// The mass, valence and density a penetration rate needs.
///
/// Kept here rather than read off the registry because a corrosion rate
/// divides by the density: the registry's `density` field is a general
/// structural value with a documented habit of falling back to water's,
/// and a silent factor of eight in a millimetres-per-year figure is the
/// kind of wrong that looks right.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetalDatum {
    /// Registry key of the metal.
    pub species: &'static str,
    /// g/mol.
    pub molar_mass: f64,
    /// Electrons per atom in the anodic dissolution assumed here.
    pub electrons: f64,
    /// Density of the solid metal at 25 °C, g/cm³.
    pub density_g_per_cm3: f64,
    pub source: &'static str,
}

const CRC_METALS: &str = "Molar masses from the IUPAC 2021 standard atomic weights; densities of the solid metals at 25 °C from the CRC Handbook of Chemistry and Physics, 'Physical Constants of Inorganic Compounds'";

/// Every metal in [`SERIES`] that can be an electrode here.
pub const METALS: &[MetalDatum] = &[
    MetalDatum {
        species: "Ag",
        molar_mass: 107.8682,
        electrons: 1.0,
        density_g_per_cm3: 10.49,
        source: CRC_METALS,
    },
    MetalDatum {
        species: "Cu",
        molar_mass: 63.546,
        electrons: 2.0,
        density_g_per_cm3: 8.96,
        source: CRC_METALS,
    },
    MetalDatum {
        species: "Pb",
        molar_mass: 207.2,
        electrons: 2.0,
        density_g_per_cm3: 11.34,
        source: CRC_METALS,
    },
    MetalDatum {
        species: "Fe",
        // Fe(II) is the anodic product; the Fe(III) of rust proper is
        // made afterwards, by oxygen, out in the solution.
        molar_mass: 55.845,
        electrons: 2.0,
        density_g_per_cm3: 7.874,
        source: CRC_METALS,
    },
    MetalDatum {
        species: "Zn",
        molar_mass: 65.38,
        electrons: 2.0,
        density_g_per_cm3: 7.134,
        source: CRC_METALS,
    },
    MetalDatum {
        species: "Mg",
        molar_mass: 24.305,
        electrons: 2.0,
        density_g_per_cm3: 1.738,
        source: CRC_METALS,
    },
];

pub fn metal_datum(species: &str) -> Option<&'static MetalDatum> {
    METALS.iter().find(|m| m.species == species)
}

/// A barrier that a named object carries and its bare metal does not.
///
/// Keyed on the lot source the material route stamps on every component
/// it deposits (`material recipe <recipe id>`), because the barrier is a
/// fact about the object and the bench resolves the object into ordinary
/// species. Both recipes say in their own `lot_assumptions` that they
/// have no geometry to place a coating with; this table is where the
/// bench keeps the sentence they could not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Barrier {
    /// `MaterialLot::source` written by the material route.
    pub lot_source: &'static str,
    /// The metal it protects.
    pub metal: &'static str,
    /// The sentence the verdict carries.
    pub why: &'static str,
    pub source: &'static str,
}

pub const BARRIERS: &[Barrier] = &[
    Barrier {
        lot_source: "material recipe metal/stainless-steel",
        metal: "Fe",
        why: "this iron came in as stainless steel, and stainless steel does not rust for a reason ordinary steel cannot borrow: its chromium reacts with oxygen faster than the iron does and builds a chromium(III) oxide film a couple of nanometres thick, transparent, tightly bound, and self-repairing — scratch it and it reforms out of the same air that would have rusted the iron. Iron oxide is none of those things: it is loose, it flakes, and it exposes fresh metal underneath, which is why rust does not stop and a passive film does",
        source: "Passivity of Fe-Cr alloys and the Cr2O3 film: Uhlig & Revie, Corrosion and Corrosion Control, 4th ed., ch. on passivity; the ~11% chromium threshold for stainless behaviour is the classical Tammann result. Editorial judgement (Kerotakis): no Cr species is installed in this registry, so the film is asserted from the object's identity rather than computed from its chromium — the recipe metal/stainless-steel says exactly this in its own lot assumptions, and this row is the bench acting on it",
    },
    Barrier {
        lot_source: "material recipe metal/painted-iron",
        metal: "Fe",
        why: "this iron came in under a complete paint film, and a sound coating simply keeps the water and the oxygen off the steel: no electrolyte on the surface, no cathodic reaction, no cell. That is the whole mechanism, and it is also the whole limitation — paint is a barrier and not a cure. Break the film anywhere and the steel at the break corrodes at the bare-metal rate, with no help from the paint around it, because paint gives no cathodic protection. Zinc does, which is the difference between a chipped painted railing and a scratched galvanised one",
        source: "Barrier protection by organic coatings and the absence of cathodic protection at a defect: Jones, Principles and Prevention of Corrosion, 2nd ed., ch. on coatings. Editorial judgement (Kerotakis): the recipe metal/painted-iron holds the paint as a conserved unresolved 5% with no geometry, so 'the film is complete' is an assumption of the object rather than a state the bench can inspect — there is no scratch verb, and a broken film is described here rather than simulated",
    },
];

/// Oxygen reduction's diffusion-limited current density, A/cm².
///
/// Levich over the constants above: `i_L = n F D c / δ` with n = 4 for
/// `O₂ + 2 H₂O + 4 e⁻ → 4 OH⁻`. About 4.2 × 10⁻⁵ A/cm², i.e. 42 µA/cm²,
/// which is where the textbook range of 20–100 µA/cm² for quiescent
/// aerated water comes from.
pub fn oxygen_limiting_current_a_per_cm2() -> f64 {
    crate::electrochemistry::limiting_current_density(
        4.0,
        OXYGEN_DIFFUSIVITY_CM2_PER_S,
        OXYGEN_SOLUBILITY_MOL_PER_L / 1000.0,
        DIFFUSION_LAYER_CM,
    )
}

/// The fraction of the diffusion-limited current an electrolyte of this
/// conductivity lets through: `κ / (κ + κ½)`.
///
/// Monotone increasing, zero at zero conductivity, and asymptotic to one.
pub fn ohmic_throttle(microsiemens_per_cm: f64) -> f64 {
    let k = microsiemens_per_cm.max(0.0);
    k / (k + OHMIC_HALF_CONDUCTIVITY_US_PER_CM)
}

/// Uniform penetration rate, mm/yr, from a corrosion current density by
/// Faraday's law: `mm/yr = i M / (n F ρ)`, converted from cm/s.
///
/// Independent of area, which is exactly why this is the quantity the
/// route reports: it survives the bench having no geometry.
pub fn penetration_mm_per_year(current_a_per_cm2: f64, metal: &MetalDatum) -> f64 {
    let cm_per_second = current_a_per_cm2 * metal.molar_mass
        / (metal.electrons * crate::displacement::FARADAY * metal.density_g_per_cm3);
    cm_per_second * SECONDS_PER_YEAR * 10.0
}

/// What the route has to say about one metal in one vessel.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// Registry key of the metal.
    pub metal: &'static str,
    /// Whether it is corroding here.
    pub corroding: bool,
    pub why: String,
    /// Corrosion current density, µA/cm², where the model computes one.
    pub current_density_ua_per_cm2: Option<f64>,
    /// Uniform penetration rate, mm/yr, where the model computes one.
    pub penetration_mm_per_year: Option<f64>,
}

fn solid_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum()
}

fn aqueous_moles(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

fn has_liquid_water(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > 0.0)
}

/// Whether oxygen can reach this surface.
///
/// An open beaker exchanges with the room, so its water is air-saturated
/// whether or not anybody added oxygen to it — the same reading of an
/// open boundary that `combustion::air` makes for a candle. A sealed
/// vessel owns its gas, and a swept one has had it purged away.
fn oxygen_present(vessel: &Vessel) -> bool {
    match vessel.headspace {
        Headspace::Open => true,
        Headspace::Swept { .. } => false,
        Headspace::Sealed { .. } | Headspace::PressureControlled { .. } => {
            vessel.moles_of(&SpeciesId::new("O2")).0 > crate::OBSERVABLE_MOLES
        }
    }
}

/// Whether the displacement route owns this vessel instead.
///
/// Two cases, and in both the cathode is not oxygen: free acidity makes
/// it `2 H⁺ + 2 e⁻ → H₂`, and a dissolved noble-metal ion makes it that
/// metal plating out. Both are computed by `displacement` with its own
/// thermodynamics, and two solvers must not both narrate one beaker.
fn displacement_owns(vessel: &Vessel) -> bool {
    if crate::displacement::unspent_acidity(vessel) > crate::OBSERVABLE_MOLES {
        return true;
    }
    SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .any(|c| aqueous_moles(vessel, c.oxidised) > crate::OBSERVABLE_MOLES)
}

fn barrier_for(vessel: &Vessel, metal: &str) -> Option<&'static Barrier> {
    let mut found: Option<&'static Barrier> = None;
    for lot in vessel.lots.iter() {
        if lot.species.0 != metal || lot.phase != Phase::Solid {
            continue;
        }
        let source = lot.source.as_deref().unwrap_or("");
        // Every lot of this metal has to have arrived under a barrier.
        // A stainless spoon dropped in beside a bare nail protects the
        // spoon and not the nail, and the bench cannot tell their iron
        // apart once it is in `contents` — so the `?` here is the claim
        // being withdrawn by ONE bare lot rather than extended to metal
        // it was never about.
        let barrier = BARRIERS
            .iter()
            .find(|b| b.metal == metal && b.lot_source == source)?;
        found = Some(barrier);
    }
    // `None` when no lot of this metal exists at all, which is the right
    // answer: a barrier is a claim about an object that was added, and
    // metal that arrived by some other road has none.
    found
}

fn display_name(key: &str) -> &str {
    species::lookup_key(key).map(|d| d.name).unwrap_or(key)
}

fn conductivity_us_per_cm(vessel: &Vessel) -> Option<f64> {
    vessel
        .solution
        .as_ref()
        .map(|info| crate::conductivity::specific_conductance(info).microsiemens_per_cm)
}

/// Every corrosion verdict this vessel earns, one per metal present.
///
/// Empty when the route has nothing to say: no metal, no liquid water, or
/// a beaker the displacement route owns.
pub fn verdicts(vessel: &Vessel) -> Vec<Verdict> {
    if !has_liquid_water(vessel) || displacement_owns(vessel) {
        return Vec::new();
    }
    let present: Vec<&'static Couple> = SERIES
        .iter()
        .filter(|c| c.reduced_phase == Phase::Solid)
        .filter(|c| solid_moles(vessel, c.reduced) > crate::OBSERVABLE_MOLES)
        .collect();
    if present.is_empty() {
        return Vec::new();
    }

    // The anode is the lowest-E° metal in contact that is neither noble
    // nor behind a barrier. Metals above hydrogen are never the anode of
    // an oxygen cell on this bench (see the copper sentence below), and a
    // barriered object is not in the circuit at all.
    let anode: Option<&'static Couple> = present
        .iter()
        .copied()
        .filter(|c| c.e0_volts < 0.0)
        .filter(|c| barrier_for(vessel, c.reduced).is_none())
        .min_by(|a, b| a.e0_volts.total_cmp(&b.e0_volts));

    // Which metals the anode is holding up, named once so the anode's
    // own sentence can say so. Barriered and noble metals are excluded:
    // neither owes its survival to the anode.
    let protected: Vec<&'static str> = present
        .iter()
        .copied()
        .filter(|c| c.e0_volts < 0.0)
        .filter(|c| barrier_for(vessel, c.reduced).is_none())
        .filter(|c| anode.is_some_and(|a| a.reduced != c.reduced))
        .map(|c| c.reduced)
        .collect();

    let oxygen = oxygen_present(vessel);
    let conductivity = conductivity_us_per_cm(vessel);
    let ceiling = oxygen_limiting_current_a_per_cm2();

    let mut out = Vec::new();
    for couple in present.iter().copied() {
        let metal = couple.reduced;
        let name = display_name(metal);

        if let Some(barrier) = barrier_for(vessel, metal) {
            out.push(Verdict {
                metal,
                corroding: false,
                why: barrier.why.to_string(),
                current_density_ua_per_cm2: None,
                penetration_mm_per_year: None,
            });
            continue;
        }

        if couple.e0_volts > 0.0 {
            let patina = if metal == "Cu" {
                ". The green on an old copper contact is not rust and is not this reaction: it is a patina of basic copper carbonate and sulfate, grown over years from carbon dioxide and sulfur dioxide in the air on a first film of Cu2O. That is atmospheric weathering, it needs a gas phase this bench does not carry, and no route here claims it"
            } else {
                ""
            };
            out.push(Verdict {
                metal,
                corroding: false,
                why: format!(
                    "{name} sits above hydrogen in the activity series (E° {:+.3} V), so in aerated neutral water it is the cathode of any corrosion cell rather than the anode: oxygen takes electrons at its surface and it stays as the metal{patina}",
                    couple.e0_volts
                ),
                current_density_ua_per_cm2: None,
                penetration_mm_per_year: None,
            });
            continue;
        }

        let Some(anode) = anode else {
            continue;
        };

        if anode.reduced != metal {
            let other = display_name(anode.reduced);
            out.push(Verdict {
                metal,
                corroding: false,
                why: format!(
                    "{name} does not corrode while {other} is in contact with it here. The two metals in one electrolyte are a cell, and the cell decides: {other} sits below {name} in the series (E° {:+.3} V against {:+.3} V), so {other} is the anode and gives up the electrons for both, and {name} is the cathode and is spared. This is what galvanising is and why a scratch does not undo it — the zinc protects the iron it is merely NEXT to, not only the iron it covers, and it goes on doing so until the zinc is gone",
                    anode.e0_volts, couple.e0_volts
                ),
                current_density_ua_per_cm2: None,
                penetration_mm_per_year: None,
            });
            continue;
        }

        if !oxygen {
            out.push(Verdict {
                metal,
                corroding: false,
                why: format!(
                    "{name} and liquid water are both here and there is no oxygen, so nothing rusts. Rusting needs three things at once — the metal, liquid water, and oxygen — because it is a circuit: the metal gives up electrons at the anode, oxygen takes them at the cathode, and the water carries the ions between the two. Remove the oxygen and there is no cathode; boiled water under oil keeps a nail bright for weeks"
                ),
                current_density_ua_per_cm2: None,
                penetration_mm_per_year: None,
            });
            continue;
        }

        let Some(datum) = metal_datum(metal) else {
            continue;
        };
        let protecting = if protected.is_empty() {
            String::new()
        } else {
            let names: Vec<&str> = protected.iter().copied().map(display_name).collect();
            format!(
                ", and it is protecting the {} here for as long as it lasts",
                names.join(" and ")
            )
        };

        match conductivity {
            Some(kappa) => {
                let throttle = ohmic_throttle(kappa);
                let current = ceiling * throttle;
                let mm = penetration_mm_per_year(current, datum);
                out.push(Verdict {
                    metal,
                    corroding: true,
                    why: format!(
                        "{name}, liquid water and oxygen are all three here, which is everything corrosion needs: {name} gives up electrons at the anode, oxygen takes them at the cathode (O2 + 2 H2O + 4 e- -> 4 OH-), and the water between them carries the ions{protecting}. How fast is a race between two resistances. Oxygen can only reach the surface so quickly, which caps the cell at {:.0} µA/cm²; below that cap the current is throttled by the electrolyte's own resistance, and this one conducts at {:.0} µS/cm, which lets {:.0}% of the cap through — {:.1} µA/cm², or about {:.2} mm a year of uniform loss on a bare surface. That is why dissolved salt speeds rusting up and distilled water almost stops it, and why brine is roughly twice tap water rather than ten times: the cap is the same for both",
                        ceiling * 1e6,
                        kappa,
                        throttle * 100.0,
                        current * 1e6,
                        mm
                    ),
                    current_density_ua_per_cm2: Some(current * 1e6),
                    penetration_mm_per_year: Some(mm),
                });
            }
            None => out.push(Verdict {
                metal,
                corroding: true,
                why: format!(
                    "{name}, liquid water and oxygen are all three here, which is everything corrosion needs: {name} gives up electrons at the anode, oxygen takes them at the cathode (O2 + 2 H2O + 4 e- -> 4 OH-), and the water between them carries the ions{protecting}. The rate is not given because no aqueous solution has been characterised in this vessel yet, and the electrolyte's conductivity is half of what sets it"
                ),
                current_density_ua_per_cm2: None,
                penetration_mm_per_year: None,
            }),
        }
    }
    out
}

/// Whether the corrosion route has a verdict for this metal in this
/// vessel.
///
/// `displacement::bystanders` asks before writing its "reaction with
/// water itself is a rate this lab does not model" apology: once this
/// route speaks for the metal, that sentence is no longer true, and two
/// contradictory sentences about one nail are worse than either.
pub fn speaks_for(vessel: &Vessel, metal: &str) -> bool {
    verdicts(vessel).iter().any(|v| v.metal == metal)
}

/// Corrosion as a solver: it reports and deliberately moves nothing.
#[derive(Debug, Default, Clone, Copy)]
pub struct CorrosionEquilibrator;

impl Equilibrator for CorrosionEquilibrator {
    fn name(&self) -> &'static str {
        "corrosion"
    }

    fn route_kind(&self) -> SolverRouteKind {
        SolverRouteKind::Computed
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        !verdicts(vessel).is_empty()
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let id = vessel.id;
        Ok(verdicts(vessel)
            .into_iter()
            .map(|v| Event::Corroded {
                vessel: id,
                species: SpeciesId::new(v.metal),
                corroding: v.corroding,
                why: v.why,
                current_density_ua_per_cm2: v.current_density_ua_per_cm2,
                penetration_mm_per_year: v.penetration_mm_per_year,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_metal_datum_names_a_series_couple() {
        for datum in METALS {
            assert!(
                SERIES.iter().any(|c| c.reduced == datum.species),
                "{} is not in the activity series",
                datum.species
            );
        }
        for couple in SERIES.iter().filter(|c| c.reduced_phase == Phase::Solid) {
            assert!(
                metal_datum(couple.reduced).is_some(),
                "{} has no mass/density datum",
                couple.reduced
            );
        }
    }

    #[test]
    fn every_barrier_names_a_series_metal() {
        for barrier in BARRIERS {
            assert!(
                SERIES.iter().any(|c| c.reduced == barrier.metal),
                "{} is not in the activity series",
                barrier.metal
            );
        }
    }

    #[test]
    fn the_oxygen_ceiling_is_the_textbook_band() {
        let micro = oxygen_limiting_current_a_per_cm2() * 1e6;
        assert!(
            (20.0..=100.0).contains(&micro),
            "limiting current {micro} µA/cm² is outside the quoted 20-100 band"
        );
    }

    #[test]
    fn salt_raises_the_throttle_monotonically_towards_one() {
        let distilled = ohmic_throttle(1.0);
        let tap = ohmic_throttle(500.0);
        let brine = ohmic_throttle(50_000.0);
        assert!(distilled < tap, "tap water must beat distilled");
        assert!(tap < brine, "brine must beat tap water");
        assert!(
            distilled < 0.01,
            "distilled water must be strongly throttled"
        );
        assert!(
            (tap - 0.5).abs() < 1e-12,
            "the half point is the half point"
        );
        assert!(
            brine > 0.98 && brine < 1.0,
            "brine must approach the cap without passing it"
        );
        assert_eq!(ohmic_throttle(0.0), 0.0);
        assert_eq!(ohmic_throttle(-5.0), 0.0);
    }

    #[test]
    fn iron_penetrates_faster_than_the_same_current_on_a_denser_metal() {
        let current = oxygen_limiting_current_a_per_cm2();
        let iron = penetration_mm_per_year(current, metal_datum("Fe").unwrap());
        let lead = penetration_mm_per_year(current, metal_datum("Pb").unwrap());
        assert!(iron > 0.0 && lead > 0.0);
        assert!(
            iron > lead,
            "iron {iron} mm/yr should outrun lead {lead} mm/yr at equal current"
        );
        assert!(
            (0.1..=2.0).contains(&iron),
            "iron at the oxygen limit should be a fraction of a millimetre a year, not {iron}"
        );
    }
}

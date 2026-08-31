//! CAP-23, rung 1: computed answers for the single-solvent organic
//! bench. Every wired solution model assumed water; a beaker of
//! ethanol full of salts drew a wall of "not yet modelled". But
//! "sodium chloride is practically insoluble in ethanol" is not a
//! model gap — it is chemistry with a handbook value, and "zinc does
//! not react with dry ethanol at bench conditions" is knowledge, not
//! an apology. This equilibrator says those things with numbers.
//!
//! Model boundary, stated once and carried in every event: dissolved
//! amounts in an organic solvent are undissociated solute held at the
//! curated solubility limit. No speciation, no activity model, no
//! electrical conductivity claim — those need an electrolyte theory
//! for the solvent, which this deliberately is not. Mixtures of two
//! organic solvents, and water present in any amount, are outside
//! this rung: the honesty pass keeps those.

use crate::ops::Event;
use crate::solve::{Equilibrator, SolveError};
use crate::species::{self, Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Solvents this rung understands: one of these as the only liquid,
/// with no water anywhere, makes the vessel a single-solvent organic
/// bench.
pub const KNOWN_SOLVENTS: &[&str] = &["ethanol", "hexane", "ethyl_acetate", "propanone"];

/// Curated solubility of a solid in an organic solvent near room
/// temperature, g of solute per 100 mL of solvent. `0.0` renders as
/// "practically insoluble" — the CRC's own "i". Values are handbook
/// magnitudes, good to the first digit; the point is the verdict and
/// its order of magnitude, and the provenance says exactly that.
pub struct OrganicSolubility {
    pub solute: &'static str,
    pub solvent: &'static str,
    pub g_per_100ml: f64,
    pub source: &'static str,
}

pub const ORGANIC_SOLUBILITY: &[OrganicSolubility] = &[
    // ── ethanol ─────────────────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "ethanol",
        g_per_100ml: 0.065,
        source: "CRC Handbook, 97th ed.: NaCl in ethanol 0.065 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "ethanol",
        g_per_100ml: 0.03,
        source: "CRC Handbook, 97th ed.: KCl sparingly soluble in ethanol, ~0.03 g/100 mL",
    },
    OrganicSolubility {
        solute: "AgCl",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: AgCl insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "ethanol",
        g_per_100ml: 2.1,
        source: "CRC Handbook, 97th ed.: AgNO3 in ethanol 2.1 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "Pb(NO3)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Pb(NO3)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "ethanol",
        g_per_100ml: 13.9,
        source: "CRC Handbook, 97th ed.: NaOH in ethanol 13.9 g/100 mL",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaCO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CaCl2",
        solvent: "ethanol",
        g_per_100ml: 25.8,
        source: "CRC Handbook, 97th ed.: CaCl2 in ethanol 25.8 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "NaOAc",
        solvent: "ethanol",
        g_per_100ml: 5.3,
        source: "CRC Handbook, 97th ed.: NaOAc in ethanol ~5.3 g/100 mL",
    },
    OrganicSolubility {
        solute: "MgSO4",
        solvent: "ethanol",
        g_per_100ml: 1.2,
        source: "CRC Handbook, 97th ed.: MgSO4 slightly soluble in ethanol, ~1.2 g/100 mL",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "ethanol",
        g_per_100ml: 0.066,
        source: "CRC Handbook, 97th ed.: sulfur in ethanol ~0.066 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "NaHCO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaHCO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Na2CO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Na2CO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Na2SO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Na2SO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Na2S2O3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Na2S2O3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "FeSO4",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: FeSO4 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CuSO4 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "ZnSO4",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: ZnSO4 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "MnO2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: MnO2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Ca(OH)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Ca(OH)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Cu(OH)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Cu(OH)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CuO",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CuO insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "MgO",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: MgO insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CaO",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaO practically insoluble in ethanol at 25 °C",
    },
    OrganicSolubility {
        solute: "gypsum",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaSO4·2H2O insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: graphite insoluble in organic solvents ('i')",
    },
    // ── hexane ──────────────────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaCl insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: KCl insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "AgCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: AgCl insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: AgNO3 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "Pb(NO3)2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Pb(NO3)2 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaOH insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaCO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CaCl2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaCl2 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "NaOAc",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaOAc insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "MgSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: MgSO4 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "hexane",
        g_per_100ml: 0.05,
        source: "CRC Handbook, 97th ed.: sulfur slightly soluble in hexane, ~0.05 g/100 mL",
    },
    OrganicSolubility {
        solute: "NaHCO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaHCO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "Na2CO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Na2CO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "FeSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: FeSO4 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CuSO4 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "ZnSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: ZnSO4 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "MnO2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: MnO2 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CaO",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaO insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "MgO",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: MgO insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "Ca(OH)2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Ca(OH)2 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "Cu(OH)2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: Cu(OH)2 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CuO",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CuO insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "hexane",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: graphite insoluble in organic solvents ('i')",
    },
    // ── propanone (acetone) ─────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaCl insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: KCl insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "propanone",
        g_per_100ml: 0.44,
        source: "CRC Handbook, 97th ed.: AgNO3 in acetone ~0.44 g/100 mL (20 °C)",
    },
    OrganicSolubility {
        solute: "CaCl2",
        solvent: "propanone",
        g_per_100ml: 33.3,
        source: "CRC Handbook, 97th ed.: CaCl2 very soluble in acetone, ~33.3 g/100 mL",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaCO3 insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaOH insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "MgSO4",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: MgSO4 insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CuSO4 insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: sulfur insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "propanone",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: graphite insoluble in organic solvents ('i')",
    },
    // ── ethyl_acetate ───────────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaCl insoluble in ethyl acetate ('i')",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CaCO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "ethyl_acetate",
        g_per_100ml: 1.8,
        source: "CRC Handbook, 97th ed.: sulfur in ethyl acetate ~1.8 g/100 mL",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: CuSO4 insoluble in ethyl acetate ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: NaOH insoluble in ethyl acetate ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        source: "CRC Handbook, 97th ed.: graphite insoluble in organic solvents ('i')",
    },
];

/// Metal–solvent pairs where the computed answer is "no reaction at
/// bench conditions", with the reason a learner can check. KMnO4 is
/// deliberately NOT here and NOT in the solubility table: permanganate
/// in ethanol reacts (the classic oxidation the safety screen warns
/// about) — that entry belongs to the curated-reaction rung, and
/// tabulating it as inert or merely soluble would be a lie.
pub const INERT_IN_SOLVENT: &[(&str, &str, &str)] = &[
    // ── ethanol ─────────────────────────────────────────────────────
    (
        "Zn",
        "ethanol",
        "the activity series needs a proton source or an aqueous couple; \
         dry ethanol at bench temperature offers neither at an observable rate",
    ),
    (
        "Fe",
        "ethanol",
        "no aqueous couple and no acid: iron sits in dry ethanol unchanged",
    ),
    (
        "Cu",
        "ethanol",
        "copper is below hydrogen in the series and dry ethanol offers \
         nothing to oxidise it",
    ),
    (
        "Mg",
        "ethanol",
        "magnesium ethoxide forms only at reflux with a catalyst; at bench \
         temperature the ribbon sits unchanged",
    ),
    (
        "Pb",
        "ethanol",
        "no aqueous couple and no acid: lead sits in dry ethanol unchanged",
    ),
    (
        "Ag",
        "ethanol",
        "silver is noble against everything this solvent can offer",
    ),
    (
        "MnO2",
        "ethanol",
        "manganese dioxide is an insoluble oxide; it sits as a brown-black \
         precipitate in ethanol unchanged",
    ),
    // ── hexane ──────────────────────────────────────────────────────
    (
        "Zn",
        "hexane",
        "hexane is non-polar and aprotic; no redox or acid-base pathway \
         to attack zinc at bench conditions",
    ),
    (
        "Fe",
        "hexane",
        "iron is inert in dry hydrocarbons — no water, no acid, no oxidant",
    ),
    (
        "Cu",
        "hexane",
        "copper is inert in dry hydrocarbons — no oxidant present",
    ),
    (
        "Mg",
        "hexane",
        "magnesium is inert in dry hydrocarbons — no protic or Lewis-acid pathway",
    ),
    (
        "Pb",
        "hexane",
        "lead is inert in dry hydrocarbons at bench temperature",
    ),
    (
        "Ag",
        "hexane",
        "silver is noble and hexane offers nothing to oxidise it",
    ),
    // ── propanone (acetone) ─────────────────────────────────────────
    (
        "Zn",
        "propanone",
        "dry acetone is aprotic; no acid-base or redox pathway attacks \
         zinc at bench temperature",
    ),
    (
        "Fe",
        "propanone",
        "iron sits in dry acetone unchanged — no aqueous couple, no acid",
    ),
    (
        "Cu",
        "propanone",
        "copper is below hydrogen and dry acetone offers nothing to oxidise it",
    ),
    (
        "Mg",
        "propanone",
        "magnesium is inert in dry acetone at bench temperature",
    ),
    (
        "Pb",
        "propanone",
        "lead is inert in dry acetone — no aqueous couple, no acid",
    ),
    (
        "Ag",
        "propanone",
        "silver is noble against everything acetone can offer",
    ),
    // ── ethyl_acetate ───────────────────────────────────────────────
    (
        "Zn",
        "ethyl_acetate",
        "dry ethyl acetate is aprotic; no pathway attacks zinc at bench \
         temperature",
    ),
    (
        "Fe",
        "ethyl_acetate",
        "iron sits in dry ethyl acetate unchanged — no aqueous couple, no acid",
    ),
    (
        "Cu",
        "ethyl_acetate",
        "copper is inert in dry ethyl acetate — no oxidant present",
    ),
    (
        "Mg",
        "ethyl_acetate",
        "magnesium is inert in dry ethyl acetate at bench temperature",
    ),
    (
        "Pb",
        "ethyl_acetate",
        "lead is inert in dry ethyl acetate — no aqueous couple, no acid",
    ),
    (
        "Ag",
        "ethyl_acetate",
        "silver is noble against everything ethyl acetate can offer",
    ),
];

/// The single organic solvent of a water-free vessel, if there is
/// exactly one and it is on the known list.
pub fn single_organic_solvent(vessel: &Vessel) -> Option<&'static str> {
    let mut found: Option<&'static str> = None;
    for p in &vessel.contents {
        match p.phase {
            Phase::Aqueous => return None,
            Phase::Liquid => {
                if p.species.0 == "water" {
                    return None;
                }
                // A dissolved liquid solute (e.g. previously dissolved
                // NaOH) must not be mistaken for a second solvent; only
                // known solvents count.
                if let Some(s) = KNOWN_SOLVENTS.iter().find(|s| **s == p.species.0.as_str()) {
                    match found {
                        None => found = Some(s),
                        Some(prev) if prev == *s => {}
                        Some(_) => return None,
                    }
                }
            }
            _ => {}
        }
    }
    found
}

/// The mole fraction of water among the liquids that could act as a
/// solvent, when at least one known organic solvent is present beside
/// water. `None` when the question does not arise (no organic solvent,
/// or no water).
///
/// CAP-23 rung 3, routing half: an aqueous activity model describes a
/// solution IN WATER. Below [`AQUEOUS_WATER_FRACTION_FLOOR`] the liquid
/// is chiefly the organic, water is the minority guest, and asking the
/// aqueous engine anyway produced divergence dressed as a crash
/// (curiosity th-057: permanganate in ethanol, whose curated oxidation
/// leaves a trace of by-product water). The Born-corrected mixed-solvent
/// log K remains declined until someone brings data worth trusting.
pub fn water_fraction_among_solvents(vessel: &Vessel) -> Option<f64> {
    let mut water = 0.0f64;
    let mut organic = 0.0f64;
    for p in &vessel.contents {
        if !matches!(p.phase, Phase::Liquid | Phase::Aqueous) {
            continue;
        }
        if p.species.0 == "water" {
            water += p.moles.0;
        } else if KNOWN_SOLVENTS.contains(&p.species.0.as_str()) {
            organic += p.moles.0;
        }
    }
    (organic > 0.0 && water > 0.0).then(|| water / (water + organic))
}

/// Below this water mole fraction the aqueous engine is not asked: the
/// mixture's dielectric environment is the organic's, not water's, and
/// every shipped activity model assumes the latter. An editorial teaching
/// boundary, stated wherever it acts.
pub const AQUEOUS_WATER_FRACTION_FLOOR: f64 = 0.5;

/// True when this rung has a computed verdict for the pair — the
/// honesty pass consults this so a spoken answer is not followed by an
/// apology for the same solid.
pub fn verdict_exists(species: &SpeciesId, solvent: &str) -> bool {
    ORGANIC_SOLUBILITY
        .iter()
        .any(|r| r.solute == species.0 && r.solvent == solvent)
        || INERT_IN_SOLVENT
            .iter()
            .any(|(m, s, _)| *m == species.0 && *s == solvent)
}

/// Applies curated organic-solvent verdicts: dissolution to the
/// handbook limit, and computed inertness. Sits between the curated
/// reactions and the aqueous engine in every stack.
pub struct NonAqueousEquilibrator;

impl Equilibrator for NonAqueousEquilibrator {
    fn name(&self) -> &'static str {
        "nonaqueous-curated"
    }

    fn applies(&self, vessel: &Vessel) -> bool {
        let Some(solvent) = single_organic_solvent(vessel) else {
            return false;
        };
        vessel
            .contents
            .iter()
            .any(|p| p.phase == Phase::Solid && verdict_exists(&p.species, solvent))
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        let Some(solvent) = single_organic_solvent(vessel) else {
            return Ok(events);
        };
        let solvent_id = SpeciesId::new(solvent);
        let solvent_data =
            species::lookup(&solvent_id).expect("known solvents are registry species");
        let solvent_moles: f64 = vessel
            .contents
            .iter()
            .filter(|p| p.species == solvent_id && p.phase == Phase::Liquid)
            .map(|p| p.moles.0)
            .sum();
        let solvent_ml = solvent_moles * solvent_data.molar_mass / solvent_data.density;

        // Collect verdicts first; mutate after (no aliasing games).
        let mut dissolutions: Vec<(SpeciesId, Moles, Moles)> = Vec::new();
        let mut inerts: Vec<(SpeciesId, &'static str)> = Vec::new();
        for p in &vessel.contents {
            if p.phase != Phase::Solid || p.moles.0 <= 0.0 {
                continue;
            }
            if let Some(row) = ORGANIC_SOLUBILITY
                .iter()
                .find(|r| r.solute == p.species.0 && r.solvent == solvent)
            {
                let data = species::lookup(&p.species);
                let limit_mol = match data {
                    Some(d) => row.g_per_100ml * (solvent_ml / 100.0) / d.molar_mass,
                    None => 0.0,
                };
                // Already-dissolved portion counts against the limit.
                let already: f64 = vessel
                    .contents
                    .iter()
                    .filter(|q| q.species == p.species && q.phase == Phase::Liquid)
                    .map(|q| q.moles.0)
                    .sum();
                let room = (limit_mol - already).max(0.0);
                let dissolve = p.moles.0.min(room);
                // A species already settled at its limit was verdicted on
                // a previous pass; repeating "0.0000 mol dissolved" every
                // step is noise. (A fully insoluble solid has no dissolved
                // marker to remember, so its verdict does repeat — the
                // price of statelessness, and the honest default.)
                if dissolve <= 0.0 && already > 0.0 {
                    continue;
                }
                dissolutions.push((
                    p.species.clone(),
                    Moles(dissolve),
                    Moles(p.moles.0 - dissolve),
                ));
            } else if let Some((_, _, why)) = INERT_IN_SOLVENT
                .iter()
                .find(|(m, s, _)| *m == p.species.0 && *s == solvent)
            {
                inerts.push((p.species.clone(), *why));
            }
        }

        for (species_id, dissolve, remaining) in dissolutions {
            if dissolve.0 > 0.0 {
                vessel.withdraw(&species_id, dissolve);
                vessel.deposit(species_id.clone(), dissolve, Phase::Liquid);
            }
            events.push(Event::DissolvedInSolvent {
                vessel: vessel.id,
                species: species_id,
                solvent: solvent_id.clone(),
                dissolved: dissolve,
                undissolved: remaining,
            });
        }
        for (species_id, why) in inerts {
            events.push(Event::InertInSolvent {
                vessel: vessel.id,
                species: species_id,
                solvent: solvent_id.clone(),
                why: why.to_string(),
            });
        }
        Ok(events)
    }
}

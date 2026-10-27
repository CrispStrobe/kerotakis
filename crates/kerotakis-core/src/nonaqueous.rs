//! CAP-23, rung 1: computed answers for the single-solvent organic
//! bench. Every wired solution model assumed water; a beaker of
//! ethanol full of salts drew a wall of "not yet modelled". But
//! "sodium chloride is practically insoluble in ethanol" is not a
//! model gap — it is chemistry with a handbook value, and "zinc does
//! not react with dry ethanol at bench conditions" is knowledge, not
//! an apology. The legacy numerical table now has withdrawn provenance:
//! every affected pair reports a model boundary, including liquid residues.
//! Retained numerical behavior is provisional, not a validated prediction.
//!
//! Model boundary, stated once and carried in every event: dissolved
//! amounts in an organic solvent are undissociated solute held at the
//! curated solubility limit. No speciation, no activity model, no
//! electrical conductivity claim — those need an electrolyte theory
//! for the solvent, which this deliberately is not. Mixtures of two
//! organic solvents, and water present in any amount, are outside
//! this rung: the honesty pass keeps those.

use crate::i18n::Locale;
use crate::ops::Event;
use crate::phrase::{Phrase, Slot};
use crate::solve::{Equilibrator, SolveError};
use crate::species::{self, Phase, SpeciesId};
use crate::units::Moles;
use crate::vessel::Vessel;

/// Solvents this rung understands: one of these as the only liquid,
/// with no water anywhere, makes the vessel a single-solvent organic
/// bench.
pub const KNOWN_SOLVENTS: &[&str] = &["ethanol", "hexane", "ethyl_acetate", "propanone", "CH3COOH"];

/// Curated solubility of a solid in an organic solvent near room
/// temperature, g of solute per 100 mL of solvent. `0.0` renders as
/// "practically insoluble". Values are general-reference magnitudes, good
/// to the first digit; the point is the verdict and its order of
/// magnitude, and the provenance says exactly that.
///
/// ## Sixty-five legacy source lines were withdrawn on 2026-09-13
///
/// All sixty-five of these rows cited "CRC Handbook, 97th ed." as the
/// source of the number. PLAN.md's provenance audit puts that handbook on
/// the commercial row, and the project's rule for an avoid-row source is
/// that the claim stops and the prose stays as commentary. So the citation
/// is gone from every row and the magnitude it carried is kept, prefixed
/// by a statement that nothing supports it.
///
/// This is the same withdrawal the phase route's latent heats took, and it
/// is here for the same two reasons.
///
/// The first is that these were live claims, not historical notes: the
/// `source` field exists so that a bench which prints a solubility verdict
/// can print the book behind it, and the book it named is one the project
/// may not ship from.
///
/// The numbers remain as explicitly provisional legacy behavior, with a
/// machine-readable withdrawal status and a model boundary at each consumer.
/// Missing rows no longer imply unlimited solid dissolution in `curated`:
/// only already-liquid inventory is available without a solubility cap.
/// Replacing or disabling the remaining legacy values requires a separately
/// reviewed phase-ownership contract and compatible primary measurements.
///
/// No open replacement was found. The IUPAC-NIST Solubility Data Series
/// is published in J. Phys. Chem. Ref. Data and is copyrighted; PubChem's
/// experimental solubilities for these salts cite the same handbook, so
/// routing through PubChem would launder the problem rather than fix it.
/// What a row here needs is its own primary measurement with a DOI, in the
/// shape `literature/hartley-campbell-iodine-water` uses in the registry —
/// which is, appropriately, a 1908 solubility measurement.
/// Machine-readable support status; never infer provenance from source prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolubilityProvenance {
    Withdrawn,
    /// A qualitative teaching estimate, not a measured numerical solubility.
    EditorialEstimate,
}

pub struct OrganicSolubility {
    pub solute: &'static str,
    pub solvent: &'static str,
    pub g_per_100ml: f64,
    pub source: &'static str,
    pub provenance: SolubilityProvenance,
}

pub const ORGANIC_SOLUBILITY: &[OrganicSolubility] = &[
    // ── ethanol ─────────────────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "ethanol",
        g_per_100ml: 0.065,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaCl in ethanol 0.065 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "ethanol",
        g_per_100ml: 0.03,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: KCl sparingly soluble in ethanol, ~0.03 g/100 mL",
    },
    OrganicSolubility {
        solute: "AgCl",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: AgCl insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "ethanol",
        g_per_100ml: 2.1,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: AgNO3 in ethanol 2.1 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "Pb(NO3)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Pb(NO3)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "ethanol",
        g_per_100ml: 13.9,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaOH in ethanol 13.9 g/100 mL",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CaCl2",
        solvent: "ethanol",
        g_per_100ml: 25.8,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCl2 in ethanol 25.8 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "NaOAc",
        solvent: "ethanol",
        g_per_100ml: 5.3,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaOAc in ethanol ~5.3 g/100 mL",
    },
    OrganicSolubility {
        solute: "MgSO4",
        solvent: "ethanol",
        g_per_100ml: 1.2,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MgSO4 slightly soluble in ethanol, ~1.2 g/100 mL",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "ethanol",
        g_per_100ml: 0.066,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: sulfur in ethanol ~0.066 g/100 mL (25 °C)",
    },
    OrganicSolubility {
        solute: "NaHCO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaHCO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Na2CO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Na2CO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Na2SO3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Na2SO3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Na2S2O3",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Na2S2O3 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "FeSO4",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: FeSO4 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CuSO4 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "ZnSO4",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: ZnSO4 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "MnO2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MnO2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Ca(OH)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Ca(OH)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "Cu(OH)2",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Cu(OH)2 insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CuO",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CuO insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "MgO",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MgO insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "CaO",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaO practically insoluble in ethanol at 25 °C",
    },
    OrganicSolubility {
        solute: "gypsum",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaSO4·2H2O insoluble in ethanol ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "ethanol",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: graphite insoluble in organic solvents ('i')",
    },
    // bio-091: the leaf-pigment extraction. This is the one row in this
    // table with no handbook number behind it, and the source line says so
    // rather than borrowing the CRC's authority for a figure the CRC does
    // not print. Ethanol is the standard bench solvent for chlorophyll for
    // exactly this reason, and 2 g/100 mL stands for "freely soluble" at the
    // right order of magnitude — no more than that.
    OrganicSolubility {
        solute: "chlorophyll",
        solvent: "ethanol",
        g_per_100ml: 2.0,
        provenance: SolubilityProvenance::EditorialEstimate,
        source: "Editorial judgement (Kerotakis): no general reference tabulates a chlorophyll solubility, because an extract is a mixture of chlorophyll a, chlorophyll b and allomers that degrade in solution. Ethanol is nevertheless the standard solvent for extracting leaf pigment, and 2 g/100 mL is an order-of-magnitude teaching figure standing for 'freely soluble'. The verdict is the claim; the digit is not",
    },
    // ── hexane ──────────────────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaCl insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: KCl insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "AgCl",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: AgCl insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: AgNO3 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "Pb(NO3)2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Pb(NO3)2 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaOH insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CaCl2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCl2 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "NaOAc",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaOAc insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "MgSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MgSO4 insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "hexane",
        g_per_100ml: 0.05,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: sulfur slightly soluble in hexane, ~0.05 g/100 mL",
    },
    OrganicSolubility {
        solute: "NaHCO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaHCO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "Na2CO3",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Na2CO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "FeSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: FeSO4 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CuSO4 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "ZnSO4",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: ZnSO4 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "MnO2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MnO2 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CaO",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaO insoluble in hydrocarbons ('i')",
    },
    OrganicSolubility {
        solute: "MgO",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MgO insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "Ca(OH)2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Ca(OH)2 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "Cu(OH)2",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: Cu(OH)2 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "CuO",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CuO insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "hexane",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: graphite insoluble in organic solvents ('i')",
    },
    // ── propanone (acetone) ─────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaCl insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "KCl",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: KCl insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "AgNO3",
        solvent: "propanone",
        g_per_100ml: 0.44,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: AgNO3 in acetone ~0.44 g/100 mL (20 °C)",
    },
    OrganicSolubility {
        solute: "CaCl2",
        solvent: "propanone",
        g_per_100ml: 33.3,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCl2 very soluble in acetone, ~33.3 g/100 mL",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCO3 insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaOH insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "MgSO4",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: MgSO4 insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CuSO4 insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: sulfur insoluble in acetone ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "propanone",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: graphite insoluble in organic solvents ('i')",
    },
    // ── ethyl_acetate ───────────────────────────────────────────────
    OrganicSolubility {
        solute: "NaCl",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaCl insoluble in ethyl acetate ('i')",
    },
    OrganicSolubility {
        solute: "CaCO3",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CaCO3 insoluble in organic solvents ('i')",
    },
    OrganicSolubility {
        solute: "S",
        solvent: "ethyl_acetate",
        g_per_100ml: 1.8,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: sulfur in ethyl acetate ~1.8 g/100 mL",
    },
    OrganicSolubility {
        solute: "CuSO4",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: CuSO4 insoluble in ethyl acetate ('i')",
    },
    OrganicSolubility {
        solute: "NaOH",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: NaOH insoluble in ethyl acetate ('i')",
    },
    OrganicSolubility {
        solute: "C",
        solvent: "ethyl_acetate",
        g_per_100ml: 0.0,
        provenance: SolubilityProvenance::Withdrawn,
        source: "NO SOURCE IS CLAIMED; withdrawn 2026-09-13, see OrganicSolubility. The magnitude: graphite insoluble in organic solvents ('i')",
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

/// How a registry liquid relates to the aqueous model's medium admission.
///
/// Declared, never inferred from spelling: every registry species whose
/// standard phase is liquid must appear in [`REGISTRY_LIQUID_MEDIA`]
/// (`tests/nonaqueous_routing.rs` enumerates the registry and fails on a new
/// unclassified liquid). Several registry "liquids" are the dissolved solute
/// of an aqueous reagent bottle (hydrochloric acid, ammonia solution,
/// bleach): their own water arrives as `water`, so they are solutes, not a
/// competing matrix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiquidMedium {
    /// The aqueous solvent itself.
    Water,
    /// An organic liquid that competes with water as the solvent. Counted
    /// whatever its miscibility: no validated two-liquid activity/partition
    /// model exists, so a majority of it refuses rather than solves only
    /// the water layer.
    Organic,
    /// Liquid hydrogen peroxide (#839).
    Peroxide,
    /// A concentrated mineral acid liquid (sulfuric, phosphoric).
    MineralAcid,
    /// The solute of an aqueous reagent bottle; not a matrix.
    AqueousReagentSolute,
    /// Not a solvent matrix at bench conditions (a cryogen boils off).
    NotAMatrix,
}

/// Every registry liquid, classified. Order is irrelevant.
pub const REGISTRY_LIQUID_MEDIA: &[(&str, LiquidMedium)] = &[
    ("water", LiquidMedium::Water),
    ("ethanol", LiquidMedium::Organic),
    ("hexane", LiquidMedium::Organic),
    ("ethyl_acetate", LiquidMedium::Organic),
    ("propanone", LiquidMedium::Organic),
    ("CH3COOH", LiquidMedium::Organic),
    ("methanol", LiquidMedium::Organic),
    ("isopropanol", LiquidMedium::Organic),
    ("tert_butanol", LiquidMedium::Organic),
    ("lactic_acid", LiquidMedium::Organic),
    ("bromoethane", LiquidMedium::Organic),
    ("tert_butyl_bromide", LiquidMedium::Organic),
    ("dodecane", LiquidMedium::Organic),
    ("hexadecane", LiquidMedium::Organic),
    ("H2O2", LiquidMedium::Peroxide),
    ("H2SO4", LiquidMedium::MineralAcid),
    ("H3PO4", LiquidMedium::MineralAcid),
    ("HCl", LiquidMedium::AqueousReagentSolute),
    ("HI", LiquidMedium::AqueousReagentSolute),
    ("NH3", LiquidMedium::AqueousReagentSolute),
    ("NaOCl", LiquidMedium::AqueousReagentSolute),
    ("liquid_nitrogen", LiquidMedium::NotAMatrix),
];

pub fn liquid_medium(key: &str) -> Option<LiquidMedium> {
    REGISTRY_LIQUID_MEDIA
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, m)| *m)
}

/// Owned liquid/aqueous moles of each matrix class.
fn matrix_moles(vessel: &Vessel) -> (f64, f64, f64, f64) {
    let (mut water, mut organic, mut peroxide, mut acid) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for p in &vessel.contents {
        if !matches!(p.phase, Phase::Liquid | Phase::Aqueous) {
            continue;
        }
        let key = p.species.0.as_str();
        // KNOWN_SOLVENTS stays authoritative for the organic rung even if a
        // pack ever loads one without a registry liquid row.
        let medium = liquid_medium(key).or_else(|| {
            KNOWN_SOLVENTS
                .contains(&key)
                .then_some(LiquidMedium::Organic)
        });
        match medium {
            Some(LiquidMedium::Water) => water += p.moles.0,
            Some(LiquidMedium::Organic) => organic += p.moles.0,
            Some(LiquidMedium::Peroxide) => peroxide += p.moles.0,
            Some(LiquidMedium::MineralAcid) => acid += p.moles.0,
            _ => {}
        }
    }
    (water, organic, peroxide, acid)
}

/// The mole fraction of water among the liquids that could act as a
/// solvent, when an organic liquid, liquid peroxide or concentrated mineral
/// acid matrix is present (see [`LiquidMedium`]). Peroxide and mineral acids
/// are counted for aqueous-model admission only, without adding them to the
/// organic solubility tables. `None` when no competing matrix is present;
/// a dry peroxide or acid has fraction zero.
///
/// CAP-23 rung 3, routing half: an aqueous activity model describes a
/// solution IN WATER. Below [`AQUEOUS_WATER_FRACTION_FLOOR`] the liquid
/// is chiefly the organic, water is the minority guest, and asking the
/// aqueous engine anyway produced divergence dressed as a crash
/// (curiosity th-057: permanganate in ethanol, whose curated oxidation
/// leaves a trace of by-product water). The Born-corrected mixed-solvent
/// log K remains declined until someone brings data worth trusting.
pub fn water_fraction_among_solvents(vessel: &Vessel) -> Option<f64> {
    let (water, organic, peroxide, acid) = matrix_moles(vessel);
    ((organic > 0.0 && water > 0.0) || peroxide > 0.0 || acid > 0.0)
        .then(|| water / (water + organic + peroxide + acid))
}

/// Below this water mole fraction the aqueous engine is not asked: the
/// mixture is not admitted as a water-majority solvent. The shipped
/// aqueous activity models do not validate that competing liquid matrix. An editorial teaching
/// boundary, stated wherever it acts.
pub const AQUEOUS_WATER_FRACTION_FLOOR: f64 = 0.5;

/// Kept for callers of the #839 name; see [`non_water_matrix_boundary`].
pub fn peroxide_medium_boundary(vessel: &mut Vessel) -> Option<Event> {
    non_water_matrix_boundary(vessel)
}

/// Withdraw aqueous diagnostics over a peroxide- or mineral-acid-rich
/// matrix. This is an admission boundary using the existing editorial
/// water-majority floor, not a fitted activity model. Matter and
/// independently represented kinetics remain untouched. Both the direct
/// aqueous entry point and the honesty tail use it, including previously
/// complete readings. Organic majorities keep their own
/// `not-modeled.mostly-organic-solvent` honesty route.
pub fn non_water_matrix_boundary(vessel: &mut Vessel) -> Option<Event> {
    let (_, _, peroxide, acid) = matrix_moles(vessel);
    if (peroxide <= 0.0 && acid <= 0.0)
        || water_fraction_among_solvents(vessel).is_none_or(|x| x >= AQUEOUS_WATER_FRACTION_FLOOR)
    {
        return None;
    }
    let reason = if peroxide > 0.0 {
        Phrase::bare(
            "not-modeled.peroxide-rich-medium",
            "this peroxide-containing liquid is below the aqueous model's editorial water-majority admission boundary. A trace of water produced by a reaction does not establish an aqueous solvent. No validated activity model for this peroxide-rich medium is available: aqueous pH, ionic strength and speciation are withdrawn, the inventory is retained, and independently represented reaction progress is not a complete prediction for the mixture",
        )
    } else {
        let water = water_fraction_among_solvents(vessel).unwrap_or(0.0);
        Phrase::new(
            "not-modeled.concentrated-acid-medium",
            "this concentrated mineral acid is below the aqueous model's editorial water-majority admission boundary (water is {percent}% of the liquid). No validated activity model for a concentrated-acid medium is available: aqueous pH, ionic strength and speciation are withdrawn and the inventory is retained",
            vec![(
                "percent".to_string(),
                Slot::number(format!("{:.0}", water * 100.0)),
            )],
        )
    };
    vessel.solution = None;
    vessel.free_proton = 0.0;
    vessel.free_hydroxide = 0.0;
    vessel.resolved.invalidate();
    Some(Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::ModelBoundary,
        reason,
    ))
}

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

/// Report unvalidated numerical support before any consumer can use or consume
/// the solute. Liquid residues matter too: a dissolved label is not provenance.
/// This does not replace the legacy values with unlimited dissolution or claim
/// that their phase partition, reaction extent or yield has been validated.
pub(crate) fn solubility_boundaries(vessel: &Vessel, solvent: &str) -> Vec<Event> {
    ORGANIC_SOLUBILITY
        .iter()
        .filter(|row| {
            row.solvent == solvent
                && vessel.contents.iter().any(|portion| {
                    portion.species.0 == row.solute
                        && matches!(portion.phase, Phase::Solid | Phase::Liquid)
                        && portion.moles.0 > 0.0
                })
        })
        .map(|row| {
            let (key, english) = match row.provenance {
                SolubilityProvenance::Withdrawn => (
                    "not-modeled.withdrawn-organic-solubility",
                    "{solute} in {solvent}: the legacy solubility value has withdrawn provenance and no supported replacement. Reported dissolved amounts, phase partition and dependent reaction yields are provisional, not validated predictions; organic electrolyte speciation and activities are not modeled",
                ),
                SolubilityProvenance::EditorialEstimate => (
                    "not-modeled.editorial-organic-solubility",
                    "{solute} in {solvent}: the solubility value is an editorial teaching estimate, not a measured numerical limit. Reported dissolved amounts, phase partition and dependent reaction yields are provisional, not validated predictions",
                ),
            };
            Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::ModelBoundary,
                Phrase::new(
                    key,
                    english,
                    vec![
                        ("solute".into(), Slot::text(row.solute)),
                        ("solvent".into(), Slot::text(solvent)),
                    ],
                ),
            )
        })
        .collect()
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
        vessel.contents.iter().any(|p| {
            p.moles.0 > 0.0
                && matches!(p.phase, Phase::Solid | Phase::Liquid)
                && verdict_exists(&p.species, solvent)
        })
    }

    fn equilibrate(&mut self, vessel: &mut Vessel) -> Result<Vec<Event>, SolveError> {
        let mut events = Vec::new();
        let Some(solvent) = single_organic_solvent(vessel) else {
            return Ok(events);
        };
        events.extend(solubility_boundaries(vessel, solvent));
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

        // Enthalpy, not temperature, is what survives the relabelling —
        // the same rule `kerotakis-phreeqc`'s aqueous tail already keeps
        // across speciation, kept here across dissolution.
        //
        // Calling a solid "dissolved" moves it from its own Cp(T) curve to
        // the flat registry constant, because a species that owns only a
        // solid curve has none for the liquid phase and
        // `states::heat_capacity_curve` falls back to the number. Curve and
        // constant agree at the reference temperature and nowhere else, so
        // relabelling at a fixed temperature silently rewrote the vessel's
        // enthalpy: sodium chloride into warm ethanol lost
        // 4.3e-5 J per (K above 298.15 K) squared, no matter how much salt
        // was poured, because only the handbook limit ever dissolves. That
        // is 0.0157 J on a 46 K beaker and it is what made
        // `conservation::energy_is_conserved` flaky.
        //
        // Dissolution in this rung carries no heat of solution — the curated
        // rows are solubilities, not enthalpies — so the vessel's enthalpy is
        // exactly what it was and the temperature is whatever the new
        // mixture's heat capacity says it is. The shift is real but tiny:
        // 1.5e-5 K in the case above, because the two Cp models differ in
        // the third decimal, not the first.
        let t_ref = crate::units::Kelvin::STANDARD.0;
        let held = vessel.enthalpy().0;
        let relabelled = dissolutions.iter().any(|(_, dissolve, _)| dissolve.0 > 0.0);

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
        // A vessel held at a bath temperature is held; only an adiabatic one
        // moves with its own contents.
        if relabelled
            && matches!(vessel.thermal_mode, crate::vessel::ThermalMode::Adiabatic)
            && vessel.heat_capacity() > 0.0
        {
            vessel.temperature = crate::units::Kelvin(vessel.temperature_after_from(t_ref, held));
        }

        for (species_id, why) in inerts {
            // I18N-8: keyed by the ROW of INERT_IN_SOLVENT rather than by
            // the English sentence in it. The verdict is curated prose
            // with no holes, so the key has to name a place or a reword
            // orphans every translation of it — and the place a curated
            // verdict lives is the pair it was written about.
            let reason = Phrase::new(
                &format!("inert-in-solvent.{}-{}", species_id.0, solvent),
                why,
                Vec::new(),
            );
            events.push(Event::InertInSolvent {
                vessel: vessel.id,
                species: species_id,
                solvent: solvent_id.clone(),
                why: reason.render(Locale::EN),
                reason: Some(reason),
            });
        }
        Ok(events)
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    use crate::VesselId;

    fn ethanol() -> Vessel {
        let mut vessel = Vessel::new(VesselId(0), "provenance");
        vessel.deposit(SpeciesId::new("ethanol"), Moles(1.0), Phase::Liquid);
        vessel
    }

    #[test]
    fn every_unvalidated_pair_reports_one_boundary_for_split_phases() {
        assert_eq!(ORGANIC_SOLUBILITY.len(), 66);
        assert_eq!(
            ORGANIC_SOLUBILITY
                .iter()
                .filter(|row| row.provenance == SolubilityProvenance::Withdrawn)
                .count(),
            65
        );
        assert_eq!(
            ORGANIC_SOLUBILITY
                .iter()
                .filter(|row| row.provenance == SolubilityProvenance::EditorialEstimate)
                .count(),
            1
        );
        for row in ORGANIC_SOLUBILITY {
            let mut vessel = Vessel::new(VesselId(0), "all legacy pairs");
            vessel.deposit(SpeciesId::new(row.solvent), Moles(1.0), Phase::Liquid);
            vessel.deposit(SpeciesId::new(row.solute), Moles(0.001), Phase::Solid);
            vessel.deposit(SpeciesId::new(row.solute), Moles(0.001), Phase::Liquid);
            let events = solubility_boundaries(&vessel, row.solvent);
            let expected_key = match row.provenance {
                SolubilityProvenance::Withdrawn => "not-modeled.withdrawn-organic-solubility",
                SolubilityProvenance::EditorialEstimate => {
                    "not-modeled.editorial-organic-solubility"
                }
            };
            assert_eq!(events.len(), 1, "{} in {}", row.solute, row.solvent);
            assert!(matches!(&events[0], Event::NotYetModeled {
                cause: crate::ops::NotModelledCause::ModelBoundary, reason: Some(reason), ..
            } if reason.key == expected_key
                && reason.render(Locale::parse("de")) != reason.render(Locale::EN)
                && reason.render(Locale::parse("fr")) != reason.render(Locale::EN)
                && reason.text_slot("solute") == Some(row.solute)
                && reason.text_slot("solvent") == Some(row.solvent)));
        }
    }

    #[test]
    fn liquid_only_residue_still_reports_and_inert_metal_does_not() {
        let mut vessel = ethanol();
        vessel.deposit(SpeciesId::new("NaCl"), Moles(0.001), Phase::Liquid);
        let mut solver = NonAqueousEquilibrator;
        assert!(solver.applies(&vessel));
        let before = vessel.contents.clone();
        assert_eq!(solver.equilibrate(&mut vessel).unwrap().len(), 1);
        assert_eq!(vessel.contents, before);
        let mut metal = ethanol();
        metal.deposit(SpeciesId::new("Zn"), Moles(0.001), Phase::Solid);
        assert!(solubility_boundaries(&metal, "ethanol").is_empty());
        assert!(matches!(
            &solver.equilibrate(&mut metal).unwrap()[0],
            Event::InertInSolvent { .. }
        ));
    }

    #[test]
    fn curated_boundary_survives_complete_reagent_consumption() {
        let mut vessel = ethanol();
        vessel.deposit(SpeciesId::new("NaCl"), Moles(0.001), Phase::Liquid);
        vessel.deposit(SpeciesId::new("AgNO3"), Moles(0.001), Phase::Liquid);
        let events = crate::curated::CuratedEquilibrator
            .equilibrate(&mut vessel)
            .unwrap();
        assert_eq!(vessel.moles_of(&SpeciesId::new("NaCl")).0, 0.0);
        assert_eq!(vessel.moles_of(&SpeciesId::new("AgNO3")).0, 0.0);
        assert_eq!(vessel.moles_of(&SpeciesId::new("AgCl")).0, 0.001);
        let reaction_index = events
            .iter()
            .position(|event| matches!(event, Event::ReactionOccurred { .. }))
            .unwrap();
        assert_eq!(
            events[..reaction_index]
                .iter()
                .filter(|event| matches!(event, Event::NotYetModeled { .. }))
                .count(),
            2
        );
    }
}

//! Alpha-amylase digestion of starch over bench time (2026-11-11,
//! batch3-diverse:en-001).
//!
//! The defect: the curated row `2 (C6H10O5) + H2O ->[amylase] C12H22O11`
//! completed the whole digestion the instant the enzyme was added, so a
//! "sample after 20 min" protocol and a "sample after 10 s" protocol read
//! the same. The row is retired; digestion now advances only while bench
//! time passes (`clock::EnzymeClock`), at a rate taken from a stated
//! enzyme UNIT DEFINITION and a stated specific activity:
//!
//! * Unit definition (Sigma-Aldrich A3176, alpha-amylase from porcine
//!   pancreas, Type VI-B; Bernfeld-type reducing-sugar assay): "One unit
//!   will liberate 1.0 mg of maltose from starch in 3 min at pH 6.9 at
//!   20 °C." Read on the product page (sigmaaldrich.com/product/sigma/a3176,
//!   mirrored at scientificlabs.com A3176-5MU, opened 2026-11-11).
//! * Specific activity: "≥5 units/mg solid" (same page, Specifications).
//!   The page's title says ≥10; the lower specification is used, so the
//!   modelled digestion is a LOWER BOUND on what a real lot does.
//! * Same page: "The pH range for activity is 5.5 to 8.0, with the pH
//!   optimum at 7" and "Chloride ions are necessary for activity and
//!   stability."
//!
//! Model (one generic mechanism, no fitted constants):
//! `maltose mg/s = (enzyme mg × 5 U/mg) × (1.0 mg / 180 s)`, zero order in
//! starch (the unit is an initial rate in substrate excess), capped by the
//! starch present. Each maltose consumes two anhydroglucose repeat units
//! and one water, the same aggregate stoichiometry the retired row used.
//!
//! Declared boundaries:
//! * The rate is the unit-definition rate (20 °C, pH 6.9). No Arrhenius
//!   speed-up of the catalytic rate is claimed, and none is invented.
//!
//! Heat inactivation (2026-11-11, curiosity bio-030 "Does boiling destroy
//! amylase activity?" — the bench used to digest starch at 100 °C):
//!
//! * Source: Miłek J., "Determination of Activation Energies and the
//!   Optimum Temperatures of Hydrolysis of Starch by α-Amylase from Porcine
//!   Pancreas", Molecules 26 (2021) 4117, doi:10.3390/molecules26144117
//!   (open access, PMC8306296). Read in full via Europe PMC: Eqs. (2)-(7)
//!   treat deactivation as first order, `da/dt = -k_d a`,
//!   `k_d(T) = B exp(-E_d/RT)`, `β = B t_a exp(-E_d/(R T_opt))`, R = 8.315
//!   J/(mol K). Table 1 row 1a (Sigma-Aldrich porcine pancreatic enzyme,
//!   pH 6.9 sodium phosphate, t_a = 3 min — the same supplier, pH and assay
//!   time as the A3176 unit definition above; data of Akhond et al., Appl.
//!   Biochem. Biotechnol. 180 (2016) 954-968, doi:10.1007/s12010-016-2145-1):
//!   T_opt = 311.06 K, E_d = 164.9 kJ/mol, β = 1.46. Hence
//!   `k_d(T) = (β/t_a) exp[(E_d/R)(1/T_opt - 1/T)]` per minute.
//! * Applied only on the falling limb, T > T_opt, which is where Eq. (6)
//!   attributes the measured activity loss to deactivation. Below T_opt
//!   the same fit is dominated by the catalytic Arrhenius term and its
//!   k_d is not constrained by the data, so no inactivation is claimed
//!   there (37 °C digestion is unchanged).
//! * The fitted data (Fig. 1a) end at 343 K. Above 343.15 K no extrapolation
//!   is claimed: the enzyme is booked as completely inactivated at once
//!   (the fit's own k_d there already exceeds 180 per minute, a half-life
//!   under a quarter of a second). Corroboration only, different organism:
//!   wheat α-amylase loses activity completely around 85 °C (Singh,
//!   Shandilya, Kundu, Kayastha, PLoS ONE 10 (2015) e0129203,
//!   doi:10.1371/journal.pone.0129203, Fig. 3A), and wheat amylase is the
//!   more heat-stable of the two.
//! * Irreversible: the inactivated moles are kept on the vessel
//!   (`Vessel::amylase_inactive_moles`); cooling does not restore them and
//!   freshly added enzyme is active. Not carried by a pour (declared).
//! * Starch present with no active enzyme left is reported with
//!   `not-modeled.amylase-heat-inactivated` and nothing is digested.
//! * A characterised solution with pH outside 5.5-8.0 is outside the
//!   source; digestion is then refused with
//!   `not-modeled.amylase-outside-unit-conditions` instead of run.
//! * An enzyme dose that cannot be converted to milligrams (no registry
//!   molar mass) is refused with `not-modeled.amylase-dose-not-convertible`.
//!
//! Native (ungelatinised) starch granules (2026-11-11, EN-001 rerun: 0.20 g
//! starch stirred into 37 °C water was digested completely to maltose at
//! the soluble-starch rate):
//!
//! * The A3176 unit is measured on a starch SOLUTION (Bernfeld assay), i.e.
//!   gelatinised starch. Granular starch is a different substrate: Slaughter,
//!   Ellis, Butterworth, "An investigation of the action of porcine
//!   pancreatic α-amylase on native and gelatinised starches", Biochim.
//!   Biophys. Acta 1525 (2001) 29-36, doi:10.1016/S0304-4165(00)00162-8,
//!   report that gelatinisation raised k_cat/K_m by 13-fold (waxy rice) to
//!   239-fold (potato). The factor depends on the botanical source, which
//!   the bench's single `starch` species does not carry, so no generic
//!   granule rate is claimed. Since round 4 the plant source is read from
//!   the material recipe instead (below); native granules of unstated
//!   source are refused with `not-modeled.amylase-starch-source-unspecified`
//!   and stay as they are.
//! * Gelatinised state: starch held in liquid water at or above 85 °C during
//!   a clock step is booked as gelatinised (`Vessel::starch_gelatinised_moles`,
//!   irreversible; retrogradation not modelled). 85 °C is the HIGHEST onset
//!   in the cited range "some types of unmodified native starches start
//!   swelling at 55 °C, other types at 85 °C" (Belitz, Grosch, Schieberle,
//!   Food Chemistry, 3rd ed., Springer 2004, pp. 318-323, ISBN
//!   3-540-40818-5; value as quoted by
//!   https://en.wikipedia.org/wiki/Starch_gelatinization). Between 55 and
//!   85 °C the state depends on the unknown botanical source, so it is not
//!   booked. Starch added later is native again.
//! * Product spectrum: α-amylase gives maltose, maltotriose and α-limit
//!   dextrins, not pure maltose; the unit definition counts reducing sugar
//!   as maltose equivalents. The product is booked as maltose equivalents
//!   at the cited rate; the limit-dextrin end point is NOT modelled (no
//!   sourced limit extent) and is disclosed, not hidden.
//! * The chloride requirement is disclosed, not gated: the unit's assay
//!   buffer supplies it, and the bench does not know a chloride threshold.
//!
//! Per-material, per-state digestible fractions (2026-11-11, lane A,
//! round 4). The single `starch` species carries no plant source, but a
//! material recipe does: the lot it deposits is stamped
//! `material recipe <id>` (`material::LOT_SOURCE_PREFIX`). One generic
//! mechanism now reads that attribution and the starch state:
//!
//! * State: native (never gelatinised), gelatinised (held in liquid water
//!   at >= 85 °C, above) and retrograded (gelatinised starch then held in
//!   liquid water at the protocol's 4 °C for a cumulative 36 h, the
//!   retrogradation treatment of the cited tables; reheating to >= 85 °C
//!   books it gelatinised again). Retrogradation at other temperatures or
//!   times is not claimed.
//! * Table [`DIGESTIBILITY`]: Englyst-style RDS (digested by 20 min), SDS
//!   (20-120 min) and RS (> 120 min) per botanical source and state, all in
//!   ONE protocol ("protocol W"): 4 U Sigma A3176 porcine pancreatic
//!   α-amylase + 4 U A. niger amyloglucosidase (Megazyme E-AMGDF) per 10 mg
//!   starch (400 U/g), pH 6.0 phosphate, 37 °C. Each row names its paper
//!   and table. Where a paper reports a range over cultivars the row uses
//!   the LOWER digested end (smallest RDS, largest RS) and keeps the range.
//! * Time course: piecewise-linear between the measured cumulative points
//!   (0, 0), (20 min, RDS), (120 min, RDS + SDS), applied to what is left
//!   of each pool as a conditional fraction; nothing beyond 120 min is
//!   claimed (`not-modeled.amylase-starch-beyond-source-time`). The clock
//!   restarts when the starch changes state.
//! * Window: only at 37 °C (± 0.5 K, the reporting precision) and an A3176
//!   dose of 0.5x-2x the protocol's 400 U/g (using the ≥5 U/mg lower
//!   specification, above). No sourced law scales granule digestion with
//!   dose or temperature (interfacial, binding-limited), so outside the
//!   window it is refused (`not-modeled.amylase-starch-outside-source-protocol`).
//! * Upper-bound caveat: the tabulated values include amyloglucosidase,
//!   which acts synergistically on granules; amylase alone is expected to
//!   digest no faster, so the booked amount is an upper-bound-like figure
//!   (disclosed in `not-modeled.amylase-starch-product-split-unsourced`).
//! * Products: α-amylase gives maltose, maltotriose and α-limit dextrins
//!   ("The linear structures are comprised of maltose, maltotriose, and a
//!   minor amount of maltotetraose", PLoS One 8 (2013) e59745,
//!   doi:10.1371/journal.pone.0059745). No readable source gives the split,
//!   so the product is booked as maltose equivalents and the unsourced split
//!   is reported, not hidden.
//! * Plain `starch` (no material recipe: a generic laboratory starch of
//!   unstated plant source, insoluble in the registry) keeps the A3176
//!   soluble-starch unit-rate route once gelatinised (the unit is defined
//!   on dissolved starch). Native or retrograded starch of unstated source
//!   is refused (`not-modeled.amylase-starch-source-unspecified`): native
//!   RDS spans 1.7 % (green banana) to 19.6 % (wheat) in protocol W.
//! * A material whose starch has no row (whole foods such as flour, bread,
//!   rice grain or potato tuber: the starch sits in a food matrix the
//!   isolated-starch tables do not describe; no readable primary value was
//!   found) is refused (`not-modeled.amylase-starch-fraction-no-data`).
//! * The iodine colour reads the starch that remains on the ledger, so it
//!   follows the digestion without a separate rule.
//! * Declared: starch added after the clock started is put on the same
//!   clock; the attribution and the clock are not carried by a pour; the
//!   table and unit-rate routes do not share the enzyme dose between them.

use crate::ops::Event;
use crate::phrase::{Phrase, Slot};
use crate::species::{self, Phase};
use crate::vessel::Vessel;
use crate::{Moles, SpeciesId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ENZYME: &str = "amylase";
pub const SUBSTRATE: &str = "starch";
pub const PRODUCT: &str = "maltose";

/// Sigma-Aldrich A3176 specification, ≥5 units per mg solid (lower bound).
pub const SPECIFIC_ACTIVITY_U_PER_MG: f64 = 5.0;
/// One unit liberates 1.0 mg maltose in 3 min (180 s).
pub const UNIT_MALTOSE_MG: f64 = 1.0;
pub const UNIT_SECONDS: f64 = 180.0;
/// Source pH window for activity (same page).
pub const PH_RANGE: (f64, f64) = (5.5, 8.0);

pub const EQUATION: &str = "2 (C₆H₁₀O₅) + H₂O →[amylase] C₁₂H₂₂O₁₁";
/// Miłek 2021 (doi:10.3390/molecules26144117) Table 1 row 1a: optimum
/// temperature of the Sigma porcine pancreatic enzyme, K.
pub const INACTIVATION_T_OPT_K: f64 = 311.06;
/// Same row: activation energy of deactivation, J/mol (164.9 kJ/mol).
pub const INACTIVATION_E_D_J_PER_MOL: f64 = 164_900.0;
/// Same row: dimensionless β = B t_a exp(-E_d/(R T_opt)).
pub const INACTIVATION_BETA: f64 = 1.46;
/// Same row: assay time t_a, minutes.
pub const INACTIVATION_ASSAY_MINUTES: f64 = 3.0;
/// Gas constant as the paper uses it, J/(mol K).
pub const INACTIVATION_GAS_CONSTANT: f64 = 8.315;
/// Highest temperature in the fitted data (Fig. 1a), K. Above it the enzyme
/// is booked as completely inactivated rather than extrapolated.
pub const COMPLETE_LOSS_ABOVE_K: f64 = 343.15;
pub const INACTIVATION_SOURCE: &str = "Miłek, Molecules 26 (2021) 4117, doi:10.3390/molecules26144117, Table 1 row 1a (Sigma porcine pancreatic α-amylase, data of Akhond et al. 2016, doi:10.1007/s12010-016-2145-1): first-order deactivation, T_opt 311.06 K, E_d 164.9 kJ/mol, β 1.46, t_a 3 min";

/// Highest native-starch swelling onset, K (85 °C; Belitz, Grosch,
/// Schieberle, Food Chemistry 3rd ed. 2004 pp. 318-323, as quoted by
/// Wikipedia "Starch gelatinization").
pub const GELATINISED_AT_K: f64 = 358.15;
pub const GELATINISATION_SOURCE: &str = "Belitz, Grosch, Schieberle, Food Chemistry, 3rd ed., Springer 2004, pp. 318-323 (ISBN 3-540-40818-5): unmodified native starches start swelling at 55 °C (some types) to 85 °C (others); value as quoted by https://en.wikipedia.org/wiki/Starch_gelatinization";
pub const NATIVE_GRANULE_SOURCE: &str = "Slaughter, Ellis, Butterworth, Biochim. Biophys. Acta 1525 (2001) 29-36, doi:10.1016/S0304-4165(00)00162-8: gelatinisation raises porcine pancreatic α-amylase k_cat/K_m 13-fold (waxy rice) to 239-fold (potato); source-dependent, so no generic granule rate";

/// Starch state for the digestibility table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarchState {
    Native,
    Gelatinised,
    Retrograded,
}

impl StarchState {
    pub const ALL: [StarchState; 3] = [
        StarchState::Native,
        StarchState::Gelatinised,
        StarchState::Retrograded,
    ];

    pub fn key(self) -> &'static str {
        match self {
            StarchState::Native => "native",
            StarchState::Gelatinised => "gelatinised",
            StarchState::Retrograded => "retrograded",
        }
    }

    fn phrase(self) -> Phrase {
        match self {
            StarchState::Native => Phrase::bare(
                "not-modeled.amylase-starch-state-native",
                "uncooked (native)",
            ),
            StarchState::Gelatinised => Phrase::bare(
                "not-modeled.amylase-starch-state-gelatinised",
                "cooked (gelatinised)",
            ),
            StarchState::Retrograded => Phrase::bare(
                "not-modeled.amylase-starch-state-retrograded",
                "cooked and cold-stored (retrograded)",
            ),
        }
    }
}

/// One measured digestibility row (protocol W, see the module docs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DigestibilityRow {
    pub botanical: &'static str,
    pub state: StarchState,
    /// % of total starch digested by 20 min (RDS; lower end of a range).
    pub rds_pct: f64,
    /// % digested by 120 min, 100 − RS (largest RS of a range).
    pub digested_120_pct: f64,
    /// RDS / SDS / RS exactly as reported.
    pub reported: &'static str,
    pub source: &'static str,
}

const MOLECULES_2018_2132: &str =
    "Molecules 23 (2018) 2132, doi:10.3390/molecules23092132, Table 6 (protocol W, sect. 3.12)";
const SCI_REP_2017_12312: &str = "Sci. Rep. 7 (2017) 12312, doi:10.1038/s41598-017-10802-3, Table 5 (wheat cv. Yangmai 16, well-watered; protocol W)";
const MOLECULES_2017_1526: &str = "Molecules 22 (2017) 1526, doi:10.3390/molecules22091526, Table 6 (six normal rice cultivars; protocol W)";
const MOLECULES_2018_2312: &str = "Molecules 23 (2018) 2312, doi:10.3390/molecules23092312, Table 4 (green banana flesh starch; protocol W, 1 mL slurry + 1 mL enzyme)";
const FOODS_2024_1103: &str = "Foods 13 (2024) 1103, doi:10.3390/foods13071103, Table 5 (three sweet potato varieties; A3176 + E-AMGDF, 37 °C)";

/// RDS / SDS / RS per botanical source and state, protocol W (400 U A3176
/// per g starch plus amyloglucosidase, 37 °C). Values read from the open
/// full texts named in each row (Europe PMC, 2026-11-11).
pub const DIGESTIBILITY: &[DigestibilityRow] = &[
    DigestibilityRow {
        botanical: "maize",
        state: StarchState::Native,
        rds_pct: 6.45,
        digested_120_pct: 29.44,
        reported: "6.45 / 22.98 / 70.56",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "maize",
        state: StarchState::Gelatinised,
        rds_pct: 81.59,
        digested_120_pct: 89.56,
        reported: "81.59 / 7.97 / 10.44",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "maize",
        state: StarchState::Retrograded,
        rds_pct: 78.72,
        digested_120_pct: 86.71,
        reported: "78.72 / 7.99 / 13.29",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "potato",
        state: StarchState::Native,
        rds_pct: 2.27,
        digested_120_pct: 8.60,
        reported: "2.27 / 6.33 / 91.40",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "potato",
        state: StarchState::Gelatinised,
        rds_pct: 84.14,
        digested_120_pct: 92.63,
        reported: "84.14 / 8.49 / 7.37",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "potato",
        state: StarchState::Retrograded,
        rds_pct: 80.15,
        digested_120_pct: 88.43,
        reported: "80.15 / 8.29 / 11.57",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "pea",
        state: StarchState::Native,
        rds_pct: 6.20,
        digested_120_pct: 26.43,
        reported: "6.20 / 20.23 / 73.57",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "pea",
        state: StarchState::Gelatinised,
        rds_pct: 83.66,
        digested_120_pct: 98.21,
        reported: "83.66 / 14.55 / 1.79",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "pea",
        state: StarchState::Retrograded,
        rds_pct: 78.49,
        digested_120_pct: 96.28,
        reported: "78.49 / 17.80 / 3.72",
        source: MOLECULES_2018_2132,
    },
    DigestibilityRow {
        botanical: "wheat",
        state: StarchState::Native,
        rds_pct: 19.6,
        digested_120_pct: 32.3,
        reported: "19.6 / 12.7 / 67.7",
        source: SCI_REP_2017_12312,
    },
    DigestibilityRow {
        botanical: "wheat",
        state: StarchState::Gelatinised,
        rds_pct: 60.4,
        digested_120_pct: 61.2,
        reported: "60.4 / 0.8 / 38.8",
        source: SCI_REP_2017_12312,
    },
    DigestibilityRow {
        botanical: "wheat",
        state: StarchState::Retrograded,
        rds_pct: 49.2,
        digested_120_pct: 53.5,
        reported: "49.2 / 4.3 / 46.5",
        source: SCI_REP_2017_12312,
    },
    DigestibilityRow {
        botanical: "rice",
        state: StarchState::Gelatinised,
        rds_pct: 73.6,
        digested_120_pct: 85.0,
        reported: "73.6-83.7 / 8.9-9.7 / 7.3-15.0",
        source: MOLECULES_2017_1526,
    },
    DigestibilityRow {
        botanical: "rice",
        state: StarchState::Retrograded,
        rds_pct: 68.7,
        digested_120_pct: 77.2,
        reported: "68.7-76.7 / 7.5-10.1 / 15.2-22.8",
        source: MOLECULES_2017_1526,
    },
    DigestibilityRow {
        botanical: "green banana",
        state: StarchState::Native,
        rds_pct: 1.7,
        digested_120_pct: 5.9,
        reported: "1.7 / 4.3 / 94.1",
        source: MOLECULES_2018_2312,
    },
    DigestibilityRow {
        botanical: "green banana",
        state: StarchState::Gelatinised,
        rds_pct: 73.0,
        digested_120_pct: 78.1,
        reported: "73.0 / 5.1 / 21.9",
        source: MOLECULES_2018_2312,
    },
    DigestibilityRow {
        botanical: "sweet potato",
        state: StarchState::Native,
        rds_pct: 2.7,
        digested_120_pct: 13.7,
        reported: "2.7-6.8 / 10.7-19.6 / 73.6-86.3",
        source: FOODS_2024_1103,
    },
    DigestibilityRow {
        botanical: "sweet potato",
        state: StarchState::Gelatinised,
        rds_pct: 74.2,
        digested_120_pct: 75.7,
        reported: "74.2-84.6 / 0.9-3.7 / 14.2-24.3",
        source: FOODS_2024_1103,
    },
];

/// Every registry material whose recipe resolves `starch`, with the
/// botanical row it reads (`None`: no measured row; refused typed).
/// `tools/starch_amylase_coverage.py --check` fails when a starch-bearing
/// recipe is missing here.
pub const MATERIAL_BOTANICAL: &[(&str, Option<&str>)] = &[
    ("household/cornstarch", Some("maize")),
    ("household/heat-activated-baking-powder-surrogate", None),
    ("household/cocoa-powder", None),
    ("household/wheat-flour-surrogate", None),
    ("household/flour-and-water-dough", None),
    ("food/potato", None),
    ("food/bread", None),
    ("food/pasta", None),
    ("food/rice", None),
    ("food/dried-beans", None),
    ("plant/dry-seed", None),
    ("plant/germinating-seed", None),
    ("food/cake-batter", None),
    ("food/popcorn-kernel", None),
];

/// Protocol W dose of A3176, U per g starch (4 U per 10 mg).
pub const PROTOCOL_DOSE_U_PER_G: f64 = 400.0;
/// Accepted dose window as multiples of the protocol dose.
pub const PROTOCOL_DOSE_WINDOW: (f64, f64) = (0.5, 2.0);
/// Protocol W temperature, K (37 °C).
pub const PROTOCOL_T_K: f64 = 310.15;
/// Reporting precision of the protocol temperature, K.
pub const PROTOCOL_T_TOLERANCE_K: f64 = 0.5;
/// End of the RDS window, minutes.
pub const RDS_MINUTES: f64 = 20.0;
/// End of the SDS window, minutes.
pub const SDS_END_MINUTES: f64 = 120.0;
/// Retrogradation treatment of the cited tables: 4 °C for 36 h.
pub const RETROGRADE_HOLD_K: f64 = 277.15;
pub const RETROGRADE_HOLD_SECONDS: f64 = 36.0 * 3600.0;

/// Per-vessel starch-digestion bookkeeping for the table route.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StarchDigestionState {
    /// Repeat units booked as retrograded (a part of the gelatinised pool).
    #[serde(default)]
    pub retrograded_moles: f64,
    /// Cumulative seconds the gelatinised starch has sat at 4 °C in water.
    #[serde(default)]
    pub cold_hold_seconds: f64,
    /// Minutes of in-window enzyme action since the starch last changed state.
    #[serde(default)]
    pub clock_minutes: f64,
    /// Starch grams when the clock started (the dose basis, U per g).
    #[serde(default)]
    pub clock_start_g: f64,
    /// Repeat units digested so far per lot source (recipe id, "" = none).
    #[serde(default)]
    pub digested_by_source: BTreeMap<String, f64>,
}

pub fn row(botanical: &str, state: StarchState) -> Option<&'static DigestibilityRow> {
    DIGESTIBILITY
        .iter()
        .find(|r| r.botanical == botanical && r.state == state)
}

pub fn botanical_of(recipe_id: &str) -> Option<&'static str> {
    MATERIAL_BOTANICAL
        .iter()
        .find(|(id, _)| *id == recipe_id)
        .and_then(|(_, b)| *b)
}

/// Cumulative fraction digested at `minutes` (piecewise linear between the
/// measured points; flat after 120 min).
pub fn digested_fraction(row: &DigestibilityRow, minutes: f64) -> f64 {
    let t = minutes.max(0.0);
    let pct = if t <= RDS_MINUTES {
        row.rds_pct * t / RDS_MINUTES
    } else if t <= SDS_END_MINUTES {
        row.rds_pct
            + (row.digested_120_pct - row.rds_pct) * (t - RDS_MINUTES)
                / (SDS_END_MINUTES - RDS_MINUTES)
    } else {
        row.digested_120_pct
    };
    (pct / 100.0).clamp(0.0, 1.0)
}

/// Fraction of what is LEFT at `from` minutes that is digested by `to`.
pub fn conditional_fraction(row: &DigestibilityRow, from: f64, to: f64) -> f64 {
    let left = 1.0 - digested_fraction(row, from);
    if left <= 0.0 {
        return 0.0;
    }
    ((digested_fraction(row, to) - digested_fraction(row, from)) / left).clamp(0.0, 1.0)
}

/// Retrograded repeat units, mol (capped by the gelatinised pool).
pub fn retrograded_moles(vessel: &Vessel) -> f64 {
    vessel
        .starch_digestion
        .as_ref()
        .map_or(0.0, |s| s.retrograded_moles)
        .min(gelatinised_moles(vessel))
        .max(0.0)
}

/// Current starch repeat units per lot source (recipe id; "" for starch not
/// from a material recipe), scaled to the ledger total.
pub fn source_shares(vessel: &Vessel) -> Vec<(String, f64)> {
    let now = total(vessel, SUBSTRATE);
    if now <= 0.0 {
        return Vec::new();
    }
    let mut left: BTreeMap<String, f64> = BTreeMap::new();
    for lot in vessel.lots.iter().filter(|l| l.species.0 == SUBSTRATE) {
        let key = lot
            .source
            .as_deref()
            .and_then(|s| s.strip_prefix(crate::material::LOT_SOURCE_PREFIX))
            .unwrap_or("")
            .to_string();
        *left.entry(key).or_insert(0.0) += lot.moles.0;
    }
    if let Some(state) = vessel.starch_digestion.as_ref() {
        for (key, gone) in &state.digested_by_source {
            if let Some(m) = left.get_mut(key) {
                *m = (*m - gone).max(0.0);
            }
        }
    }
    let sum: f64 = left.values().sum();
    if sum <= 0.0 {
        return vec![(String::new(), now)];
    }
    left.into_iter()
        .filter(|(_, m)| *m > 0.0)
        .map(|(k, m)| (k, m / sum * now))
        .collect()
}

fn reset_clock(vessel: &mut Vessel) {
    if let Some(state) = vessel.starch_digestion.as_mut() {
        state.clock_minutes = 0.0;
        state.clock_start_g = 0.0;
    }
}

/// Book gelatinised starch held in liquid water at the protocol's 4 °C for
/// a cumulative 36 h as retrograded.
pub fn record_retrogradation(vessel: &mut Vessel, seconds: f64) {
    let gel = gelatinised_moles(vessel);
    if seconds <= 0.0
        || gel <= 0.0
        || !has_liquid_water(vessel)
        || (vessel.temperature.0 - RETROGRADE_HOLD_K).abs() > PROTOCOL_T_TOLERANCE_K
    {
        return;
    }
    let state = vessel.starch_digestion.get_or_insert_with(Default::default);
    if state.retrograded_moles >= gel {
        return;
    }
    state.cold_hold_seconds += seconds;
    if state.cold_hold_seconds >= RETROGRADE_HOLD_SECONDS {
        state.retrograded_moles = gel;
        reset_clock(vessel);
    }
}

pub const SOURCE: &str = "Sigma-Aldrich A3176 alpha-amylase (porcine pancreas, Type VI-B): 1 U liberates 1.0 mg maltose from starch in 3 min at pH 6.9, 20 °C; ≥5 U/mg solid";

fn total(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key)
        .map(|p| p.moles.0)
        .sum()
}

fn has_liquid_water(vessel: &Vessel) -> bool {
    vessel
        .contents
        .iter()
        .any(|p| p.species.0 == "water" && p.phase == Phase::Liquid && p.moles.0 > 0.0)
}

/// First-order heat-inactivation rate constant, per second, at `t_k`.
///
/// `Some(0.0)` at or below the fitted optimum (no inactivation claimed),
/// `Some(f64::INFINITY)` above `COMPLETE_LOSS_ABOVE_K` (complete loss), the
/// Miłek 2021 Eq. (5)/(7) value in between. `None` for a non-finite input.
pub fn inactivation_rate_per_s(t_k: f64) -> Option<f64> {
    if !t_k.is_finite() || t_k <= 0.0 {
        return None;
    }
    if t_k <= INACTIVATION_T_OPT_K {
        return Some(0.0);
    }
    if t_k > COMPLETE_LOSS_ABOVE_K {
        return Some(f64::INFINITY);
    }
    let per_minute = INACTIVATION_BETA / INACTIVATION_ASSAY_MINUTES
        * (INACTIVATION_E_D_J_PER_MOL / INACTIVATION_GAS_CONSTANT
            * (1.0 / INACTIVATION_T_OPT_K - 1.0 / t_k))
            .exp();
    Some(per_minute / 60.0)
}

/// Active (not heat-inactivated) amylase, mol.
pub fn active_enzyme_moles(vessel: &Vessel) -> f64 {
    (total(vessel, ENZYME) - vessel.amylase_inactive_moles.unwrap_or(0.0)).max(0.0)
}

/// Gelatinised starch repeat units available to the unit-rate route, mol
/// (capped by the ledger total).
pub fn gelatinised_moles(vessel: &Vessel) -> f64 {
    vessel
        .starch_gelatinised_moles
        .unwrap_or(0.0)
        .min(total(vessel, SUBSTRATE))
        .max(0.0)
}

/// Book every starch repeat unit as gelatinised when the vessel holds
/// liquid water at or above `GELATINISED_AT_K` while time passes.
pub fn record_gelatinisation(vessel: &mut Vessel, seconds: f64) {
    if seconds <= 0.0
        || !has_liquid_water(vessel)
        || !vessel.temperature.0.is_finite()
        || vessel.temperature.0 < GELATINISED_AT_K
    {
        return;
    }
    let starch = total(vessel, SUBSTRATE);
    if starch > 0.0 {
        let changed = vessel.starch_gelatinised_moles.unwrap_or(0.0) < starch
            || retrograded_moles(vessel) > 0.0;
        vessel.starch_gelatinised_moles = Some(starch);
        if let Some(state) = vessel.starch_digestion.as_mut() {
            // Re-gelatinised in hot water: no longer retrograded.
            state.retrograded_moles = 0.0;
            state.cold_hold_seconds = 0.0;
        }
        if changed {
            reset_clock(vessel);
        }
    }
}

fn grams_of(moles: f64) -> f64 {
    species::lookup_key(SUBSTRATE).map_or(0.0, |d| moles * d.molar_mass)
}

fn source_unspecified_event(vessel: &Vessel, moles: f64) -> Event {
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::NoReviewedDatum,
        Phrase::new(
            "not-modeled.amylase-starch-source-unspecified",
            "starch digestion by amylase: {grams} g of the starch is uncooked (native) or retrograded starch of unstated plant source; how much of such starch amylase digests depends on the source (at the same dose and 37 °C, 1.7 % in 20 min for green banana starch, 6.45 % for maize, 19.6 % for wheat), so no digestion of it is booked and it is left as it is; naming the starch, for example cornstarch, selects a measured fraction",
            vec![("grams".to_string(), Slot::number(format!("{:.3}", grams_of(moles))))],
        ),
    )
}

fn material_term(recipe_id: &str) -> Slot {
    let name = crate::material::all()
        .into_iter()
        .find(|r| r.id == recipe_id)
        .map_or_else(|| recipe_id.to_string(), |r| r.name);
    Slot::term("material", name)
}

fn no_data_event(vessel: &Vessel, recipe_id: &str, state: StarchState, moles: f64) -> Event {
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::NoReviewedDatum,
        Phrase::new(
            "not-modeled.amylase-starch-fraction-no-data",
            "starch digestion by amylase: no measured digestible fraction was found for the {state} starch of {material}, so no digestion of its {grams} g of starch is booked and it is left as it is",
            vec![
                ("state".to_string(), Slot::phrase(state.phrase())),
                ("material".to_string(), material_term(recipe_id)),
                ("grams".to_string(), Slot::number(format!("{:.3}", grams_of(moles)))),
            ],
        ),
    )
}

fn outside_protocol_event(vessel: &Vessel, u_per_g: f64) -> Event {
    let celsius = vessel.temperature.0 - 273.15;
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::ModelBoundary,
        Phrase::new(
            "not-modeled.amylase-starch-outside-source-protocol",
            "starch digestion by amylase at {celsius} °C and {dose} U per g of starch: the measured digestible fractions hold only at 37 °C and 200 to 800 U per g (half to twice the source dose), and no sourced law carries them to other doses or temperatures, so no digestion is booked and the starch is left as it is",
            vec![
                ("celsius".to_string(), Slot::number(format!("{celsius:.1}"))),
                ("dose".to_string(), Slot::number(format!("{u_per_g:.0}"))),
            ],
        ),
    )
}

fn beyond_time_event(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::ModelBoundary,
        Phrase::bare(
            "not-modeled.amylase-starch-beyond-source-time",
            "starch digestion by amylase: the measured digestible fractions end at 120 min; the starch still left after that (the resistant fraction) is not digested further and is left as it is",
        ),
    )
}

fn product_note_event(vessel: &Vessel) -> Event {
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::NoReviewedDatum,
        Phrase::bare(
            "not-modeled.amylase-starch-product-split-unsourced",
            "starch digestion by amylase: the digested starch is booked as maltose equivalents; amylase actually gives maltose, maltotriose and limit dextrins, and no source for their split was found; the measured fractions were taken with amyloglucosidase added, which speeds granule digestion, so amylase alone digests at most this much",
        ),
    )
}

/// Whether amylase can digest starch here (enzyme, substrate and liquid
/// water present). Used by the insolubility remark so it does not tell a
/// learner "starch does not react" while the clock is digesting it.
pub fn consumes(vessel: &Vessel, species: &SpeciesId) -> bool {
    species.0 == SUBSTRATE
        && active_enzyme_moles(vessel) > 0.0
        && has_liquid_water(vessel)
        && (gelatinised_moles(vessel) > 0.0
            || source_shares(vessel)
                .iter()
                .any(|(id, _)| botanical_of(id).is_some()))
}

/// Active enzyme milligrams, or `None` when the dose has no registry molar
/// mass to convert it.
pub fn enzyme_mg(vessel: &Vessel) -> Option<f64> {
    let data = species::lookup_key(ENZYME)?;
    (data.molar_mass.is_finite() && data.molar_mass > 0.0)
        .then(|| active_enzyme_moles(vessel) * data.molar_mass * 1000.0)
}

/// Book first-order inactivation over `seconds` at the vessel temperature.
/// Returns the enzyme-weighted effective digestion time: the seconds of
/// full activity the surviving enzyme delivers, `(1 - e^{-k t}) / k`.
fn inactivate(vessel: &mut Vessel, seconds: f64) -> f64 {
    let active = active_enzyme_moles(vessel);
    let k = inactivation_rate_per_s(vessel.temperature.0).unwrap_or(0.0);
    if active <= 0.0 || k <= 0.0 {
        return seconds;
    }
    let (lost, effective) = if k.is_infinite() {
        (active, 0.0)
    } else {
        let surviving = (-k * seconds).exp();
        (active * (1.0 - surviving), (1.0 - surviving) / k)
    };
    vessel.amylase_inactive_moles = Some(vessel.amylase_inactive_moles.unwrap_or(0.0) + lost);
    effective
}

fn heat_inactivated_event(vessel: &Vessel) -> Event {
    let celsius = vessel.temperature.0 - 273.15;
    Event::not_modeled(
        vessel.id,
        crate::ops::NotModelledCause::ModelBoundary,
        Phrase::new(
            "not-modeled.amylase-heat-inactivated",
            "starch digestion by amylase at {celsius} °C: the enzyme has been irreversibly heat-inactivated (first-order loss of activity above its 38 °C optimum, complete above 70 °C), so no digestion is booked and the starch is left as it is",
            vec![("celsius".to_string(), Slot::number(format!("{celsius:.0}")))],
        ),
    )
}

/// Maltose (mol) the stated activity liberates in `seconds`, before the
/// starch cap.
pub fn maltose_capacity_moles(enzyme_mg: f64, seconds: f64) -> Option<f64> {
    let maltose = species::lookup_key(PRODUCT)?;
    if maltose.molar_mass <= 0.0 {
        return None;
    }
    let mg = enzyme_mg * SPECIFIC_ACTIVITY_U_PER_MG * UNIT_MALTOSE_MG * seconds / UNIT_SECONDS;
    Some(mg / 1000.0 / maltose.molar_mass)
}

/// Advance the digestion over `seconds`. Returns the events to report.
pub fn advance(vessel: &mut Vessel, seconds: f64) -> Vec<Event> {
    let mut events = Vec::new();
    // Gelatinisation is booked with or without enzyme (cook, cool, digest).
    record_gelatinisation(vessel, seconds);
    record_retrogradation(vessel, seconds);
    if seconds <= 0.0 || total(vessel, ENZYME) <= 0.0 || !has_liquid_water(vessel) {
        return events;
    }
    // Inactivation is booked whether or not starch is present: a boiled
    // enzyme stays dead when the starch arrives later.
    // The dose is read BEFORE this interval's inactivation: the effective
    // time below already weights it by the surviving fraction.
    let had_active = active_enzyme_moles(vessel) > 0.0;
    let dose_mg = enzyme_mg(vessel);
    let effective_seconds = inactivate(vessel, seconds);
    if total(vessel, SUBSTRATE) <= 0.0 {
        return events;
    }
    if !had_active || effective_seconds <= 0.0 {
        events.push(heat_inactivated_event(vessel));
        return events;
    }
    let Some(mg) = dose_mg else {
        events.push(Event::not_modeled(
            vessel.id,
            crate::ops::NotModelledCause::NoReviewedDatum,
            Phrase::bare(
                "not-modeled.amylase-dose-not-convertible",
                "starch digestion by amylase: the enzyme amount cannot be converted to milligrams, so its activity in units is unknown and no digestion is booked",
            ),
        ));
        return events;
    };
    if let Some(ph) = vessel
        .solution
        .as_ref()
        .map(|s| s.ph)
        .filter(|ph| ph.is_finite())
    {
        if ph < PH_RANGE.0 || ph > PH_RANGE.1 {
            events.push(Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::ModelBoundary,
                Phrase::new(
                    "not-modeled.amylase-outside-unit-conditions",
                    "starch digestion by amylase at pH {ph}: the cited activity holds only between pH 5.5 and 8.0, so no digestion rate is claimed and the starch is left as it is",
                    vec![("ph".to_string(), Slot::number(format!("{ph:.2}")))],
                ),
            ));
            return events;
        }
    }
    digest(vessel, mg, effective_seconds, &mut events);
    events
}

/// One digestion interval of `seconds` effective enzyme time: the table
/// route for attributed starch, the unit-rate route for gelatinised plain
/// starch, typed refusals for everything else.
fn digest(vessel: &mut Vessel, mg: f64, seconds: f64, events: &mut Vec<Event>) {
    let starch_total = total(vessel, SUBSTRATE);
    if starch_total <= 0.0 {
        return;
    }
    let gel_all = gelatinised_moles(vessel);
    let retro_all = retrograded_moles(vessel);
    let pools = |m: f64| -> [(StarchState, f64); 3] {
        [
            (
                StarchState::Native,
                (m * (1.0 - gel_all / starch_total)).max(0.0),
            ),
            (
                StarchState::Gelatinised,
                (m * (gel_all - retro_all) / starch_total).max(0.0),
            ),
            (
                StarchState::Retrograded,
                (m * retro_all / starch_total).max(0.0),
            ),
        ]
    };
    let state = vessel.starch_digestion.clone().unwrap_or_default();
    let start_g = if state.clock_minutes > 0.0 && state.clock_start_g > 0.0 {
        state.clock_start_g
    } else {
        grams_of(starch_total)
    };
    let u_per_g = if start_g > 0.0 {
        mg * SPECIFIC_ACTIVITY_U_PER_MG / start_g
    } else {
        f64::INFINITY
    };
    let dose_window = PROTOCOL_DOSE_WINDOW.0 * PROTOCOL_DOSE_U_PER_G
        ..=PROTOCOL_DOSE_WINDOW.1 * PROTOCOL_DOSE_U_PER_G;
    let in_window = (vessel.temperature.0 - PROTOCOL_T_K).abs() <= PROTOCOL_T_TOLERANCE_K
        && dose_window.contains(&u_per_g);
    let from = state.clock_minutes;
    let to = from + seconds / 60.0;

    // (source, state, repeat units digested)
    let mut digested: Vec<(String, StarchState, f64)> = Vec::new();
    let mut unspecified_refused = 0.0;
    let mut unspecified_gel = 0.0;
    let mut table_used = false;
    let mut outside = false;
    let mut beyond = false;
    for (source, m) in source_shares(vessel) {
        let parts = pools(m);
        if source.is_empty() {
            unspecified_refused += parts[0].1 + parts[2].1;
            unspecified_gel += parts[1].1;
            continue;
        }
        for (st, pool) in parts {
            if pool <= 0.0 {
                continue;
            }
            let Some(r) = botanical_of(&source).and_then(|b| row(b, st)) else {
                events.push(no_data_event(vessel, &source, st, pool));
                continue;
            };
            if !in_window {
                outside = true;
                continue;
            }
            table_used = true;
            if to > SDS_END_MINUTES {
                beyond = true;
            }
            let d = pool * conditional_fraction(r, from, to);
            if d > 0.0 {
                digested.push((source.clone(), st, d));
            }
        }
    }
    if outside {
        events.push(outside_protocol_event(vessel, u_per_g));
    }
    if unspecified_refused > 0.0 {
        events.push(source_unspecified_event(vessel, unspecified_refused));
    }
    // Plain gelatinised starch: the A3176 unit-rate route (dissolved starch).
    if unspecified_gel > 0.0 {
        if let Some(capacity) = maltose_capacity_moles(mg, seconds) {
            let d = (2.0 * capacity).min(unspecified_gel);
            if d > 0.0 {
                digested.push((String::new(), StarchState::Gelatinised, d));
            }
        }
    }
    if table_used {
        let st = vessel.starch_digestion.get_or_insert_with(Default::default);
        if st.clock_minutes <= 0.0 {
            st.clock_start_g = start_g;
        }
        st.clock_minutes = to;
    }
    let wanted: f64 = digested.iter().map(|(_, _, d)| d).sum();
    if wanted <= 0.0 {
        if beyond {
            events.push(beyond_time_event(vessel));
        }
        return;
    }
    let water = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum::<f64>();
    // Each maltose equivalent takes one water; scale down if water is short.
    let scale = (2.0 * water / wanted).min(1.0);
    if scale <= 0.0 {
        return;
    }
    // Starch is the whole repeat-unit pool, whatever phase it sits in.
    let mut take = wanted * scale;
    for phase in [Phase::Aqueous, Phase::Solid, Phase::Liquid] {
        if take <= 0.0 {
            break;
        }
        take -= vessel
            .withdraw_phase(&SpeciesId::new(SUBSTRATE), Moles(take), phase)
            .0;
    }
    let done = wanted * scale - take.max(0.0);
    let ratio = if wanted > 0.0 { done / wanted } else { 0.0 };
    let mut gel_gone = 0.0;
    let mut retro_gone = 0.0;
    {
        let st = vessel.starch_digestion.get_or_insert_with(Default::default);
        for (source, s, d) in &digested {
            let d = d * ratio;
            *st.digested_by_source.entry(source.clone()).or_insert(0.0) += d;
            match s {
                StarchState::Native => {}
                StarchState::Gelatinised => gel_gone += d,
                StarchState::Retrograded => {
                    gel_gone += d;
                    retro_gone += d;
                }
            }
        }
        st.retrograded_moles = (st.retrograded_moles - retro_gone).max(0.0);
    }
    if gel_gone > 0.0 {
        vessel.starch_gelatinised_moles =
            Some((vessel.starch_gelatinised_moles.unwrap_or(0.0) - gel_gone).max(0.0));
    }
    let extent = done / 2.0;
    vessel.withdraw_phase(&SpeciesId::new("water"), Moles(extent), Phase::Liquid);
    vessel.deposit(SpeciesId::new(PRODUCT), Moles(extent), Phase::Aqueous);
    events.push(Event::ReactionOccurred {
        vessel: vessel.id,
        equation: EQUATION.to_string(),
    });
    if table_used {
        events.push(product_note_event(vessel));
    }
    if beyond {
        events.push(beyond_time_event(vessel));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digestion_vessel(enzyme_g: f64) -> Vessel {
        let mut v = Vessel::new(crate::VesselId(0), "beaker");
        v.deposit(SpeciesId::new("water"), Moles(2.767), Phase::Liquid);
        // 0.20 g starch as anhydroglucose repeat units (162.14 g/mol).
        v.deposit(
            SpeciesId::new(SUBSTRATE),
            Moles(0.20 / 162.14),
            Phase::Solid,
        );
        let m = species::lookup_key(ENZYME).unwrap().molar_mass;
        v.deposit(SpeciesId::new(ENZYME), Moles(enzyme_g / m), Phase::Aqueous);
        // Cooked starch: the unit rate is for gelatinised starch.
        v.starch_gelatinised_moles = Some(0.20 / 162.14);
        v
    }

    #[test]
    fn the_unit_definition_sets_the_rate() {
        // 10 mg enzyme × 5 U/mg = 50 U → 50 mg maltose per 3 min.
        let moles = maltose_capacity_moles(10.0, 180.0).unwrap();
        let mg = moles * species::lookup_key(PRODUCT).unwrap().molar_mass * 1000.0;
        assert!((mg - 50.0).abs() < 1e-9, "{mg}");
    }

    #[test]
    fn digestion_is_partial_after_a_short_wait_and_complete_later() {
        let mut v = digestion_vessel(0.010);
        let starch0 = total(&v, SUBSTRATE);
        advance(&mut v, 60.0);
        let after_minute = total(&v, SUBSTRATE);
        assert!(after_minute > 0.0 && after_minute < starch0);
        advance(&mut v, 1140.0);
        assert!(total(&v, SUBSTRATE) < 1e-15);
        assert!((total(&v, PRODUCT) - starch0 / 2.0).abs() < 1e-12);
    }

    #[test]
    fn digestion_conserves_carbon_hydrogen_oxygen() {
        let mut v = digestion_vessel(0.002);
        let count = |v: &Vessel, el: &str| -> f64 {
            v.contents
                .iter()
                .map(|p| {
                    let f =
                        crate::stoich::parse_formula(species::lookup(&p.species).unwrap().formula)
                            .unwrap();
                    p.moles.0 * f.counts.get(el).copied().unwrap_or(0.0)
                })
                .sum()
        };
        let before: Vec<f64> = ["C", "H", "O"].iter().map(|e| count(&v, e)).collect();
        advance(&mut v, 300.0);
        for (i, el) in ["C", "H", "O"].iter().enumerate() {
            assert!((count(&v, el) - before[i]).abs() < 1e-12, "{el}");
        }
    }

    #[test]
    fn no_time_no_digestion() {
        let mut v = digestion_vessel(0.010);
        let starch0 = total(&v, SUBSTRATE);
        assert!(advance(&mut v, 0.0).is_empty());
        assert_eq!(total(&v, SUBSTRATE), starch0);
    }

    #[test]
    fn iodine_complex_follows_the_starch_left_after_table_digestion() {
        let make = || {
            let mut v = Vessel::new(crate::VesselId(0), "iodine");
            v.deposit(SpeciesId::new("water"), Moles(2.767), Phase::Liquid);
            v.deposit_lot(
                SpeciesId::new(SUBSTRATE),
                Moles(0.20 / 162.14),
                Phase::Solid,
                Some("material recipe household/cornstarch".to_string()),
                None,
            );
            v.temperature = crate::units::Kelvin::from_celsius(95.0);
            advance(&mut v, 60.0);
            v.temperature = crate::units::Kelvin::from_celsius(37.0);
            v
        };
        let control = make();
        let mut treated = make();
        let m = species::lookup_key(ENZYME).unwrap().molar_mass;
        treated.deposit(SpeciesId::new(ENZYME), Moles(0.010 / m), Phase::Aqueous);
        advance(&mut treated, 1200.0);
        let with_iodine = |mut v: Vessel| {
            v.deposit(SpeciesId::new("I2"), Moles(1.0), Phase::Aqueous);
            v.deposit(SpeciesId::new("I-"), Moles(1.0), Phase::Aqueous);
            crate::starch_iodine::complex_moles(&v)
        };
        let ratio = with_iodine(treated) / with_iodine(control);
        // Gelatinised maize RDS 81.59 % (Molecules 23 (2018) 2132, Table 6).
        assert!((ratio - (1.0 - 0.8159)).abs() < 1e-9, "{ratio}");
    }
}

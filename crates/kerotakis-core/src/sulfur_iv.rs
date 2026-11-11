//! S(IV) reductant coupling: sulfur dioxide, hydrogen sulfite and sulfite
//! as a reductant pool that iodine and hydrogen peroxide oxidise to
//! sulfate.
//!
//! KI-011 found the gap: thiosulfate's acid decomposition
//! (`S₂O₃²⁻ + 2 H⁺ → S + SO₂ + H₂O`) left its SO₂ beside iodine and
//! hydrogen peroxide for 100 s. Both oxidise S(IV) in milliseconds, so an
//! unreacted S(IV) pool next to either is not a state a beaker can hold,
//! and the decomposition is not a pure loss of sink capacity: each S(IV)
//! still reduces one I₂.
//!
//! One generic writer, for every S(IV) form in the vessel (`SO2` dissolved
//! or held in the vessel's own gas phase while it has liquid water, `HSO3-`,
//! `SO3-2`, and every Na/K salt of the ions the registry carries), against
//! the two oxidants:
//!
//! * iodine (I₂ and the I₃⁻ it forms with iodide):
//!   `S(IV) + I₂ (+ H₂O) → SO₄²⁻ + 2 I⁻ + n H⁺`. Second-order rate
//!   constants at 25.0 °C, μ = 0.50 M, p[H⁺] 4.1-6.6: I₂ + HSO₃⁻ 1.8·10⁹,
//!   I₃⁻ + HSO₃⁻ 1.4·10⁷, I₂ + SO₃²⁻ 3.2·10⁹, I₃⁻ + SO₃²⁻ 2.8·10⁸
//!   M⁻¹ s⁻¹. Source: B. S. Yiin and D. W. Margerum, "Non-metal redox
//!   kinetics: reactions of iodine and triiodide with sulfite and hydrogen
//!   sulfite and the hydrolysis of iodosulfate", Inorg. Chem. 29 (1990)
//!   1559-1564, doi:10.1021/ic00333a023; values as printed in the abstract
//!   of the same work, B. S. Yiin, PhD dissertation, Purdue University
//!   1990 (AAI9018935, docs.lib.purdue.edu/dissertations/AAI9018935).
//! * hydrogen peroxide: `S(IV) + H₂O₂ → SO₄²⁻ + n H⁺ (+ H₂O)`, with
//!   `rate = k [H⁺][H₂O₂][HSO₃⁻] / (1 + K[H⁺])`, k = 7.50·10⁷ M⁻² s⁻¹,
//!   K = 13 M⁻¹ at 25 °C: M. R. Hoffmann and J. G. Calvert, "Chemical
//!   transformation modules for Eulerian acid deposition models, Vol. II:
//!   The aqueous-phase chemistry", NCAR / US EPA (1985), rate-law summary
//!   table ("k = 7.50E+07 (1/M2s) K = 1.30E+01"; archive.org OCR). The
//!   temperature dependence is the measured ΔH‡ = 37 kJ/mol of J. V.
//!   McArdle and M. R. Hoffmann, J. Phys. Chem. 87 (1983) 5425-5429,
//!   doi:10.1021/j150644a024 (abstract, CaltechAUTHORS): k₁ = 2.6·10⁶
//!   M⁻¹ s⁻¹ and k₂/k₋₁ = 16 M⁻¹ at 15 °C over pH 0.0-4.5, whose product
//!   moved to 25 °C (7.2·10⁷) cross-checks the 25 °C k within 4 %. The
//!   measured acidity range pH 0-4.5 bounds the law here.
//!
//! Every other registry oxidant has a row in [`OXIDANT_TABLE`], priced by
//! one generic writer when it is the only S(IV) oxidant in the vessel:
//!
//! * rated (second-order constants per S(IV) form, 25 °C): ozone
//!   (Hoffmann & Calvert 1985: SO₂·H₂O 2.4·10⁴, HSO₃⁻ 3.7·10⁵, SO₃²⁻
//!   1.5·10⁹ M⁻¹ s⁻¹), HOBr + SO₃²⁻ 5.0·10⁹ and OBr⁻ + SO₃²⁻ 1.0·10⁸
//!   (Troy & Margerum, Inorg. Chem. 30 (1991) 3538,
//!   doi:10.1021/ic00018a028, as quoted by Liu et al., Geophys. Res. Lett.
//!   47 (2020), doi:10.1029/2019GL086465; a database page gives 2.3·10⁹
//!   for HOBr, conflict noted). Run as instantaneous only while every
//!   present S(IV) form has a constant and lives at most [`MAX_INSTANT_S`].
//! * stoichiometric (no rate constant claimed, typed note on every run):
//!   Br₂, MnO₄⁻ (acid, → Mn²⁺), Cr₂O₇²⁻ / HCrO₄⁻ (acid, → Cr³⁺). Proton
//!   demand is limited by represented acidity; the rest is refused typed
//!   (neutral/basic permanganate products are not pinned by a source).
//! * owned elsewhere: HClO / ClO⁻ (the admitted `halogen_redox` rows;
//!   HOCl + SO₃²⁻ 7.6·10⁸ M⁻¹ s⁻¹ at 25 °C, μ 0.5, Fogelman, Walker &
//!   Margerum, Inorg. Chem. 28 (1989) 986, doi:10.1021/ic00305a002).
//! * refused typed: nitrite (rate = k[NO₂⁻][HSO₃⁻][H⁺], k = 4.0-4.5·10³
//!   M⁻² s⁻¹ at 22 °C, but the product is hydroxylamine disulfonate, not
//!   in the registry: D. Oblath, PhD thesis, LBL-13026 (1981), Table II),
//!   and dissolved O₂ (uncatalysed or Fe/Mn-catalysed autoxidation: no
//!   reviewed rate that holds without catalyst, pH and ionic strength).
//!
//! Both oxidations are treated as instantaneous on the bench clock (S(IV)
//! lifetime milliseconds at the concentrations the bench holds); the
//! peroxide path is admitted only when its pseudo-first-order lifetime is
//! at most [`MAX_INSTANT_S`] and inside the measured acidity range.
//!
//! When both oxidants are present they COMPETE for the S(IV). Both rate
//! laws are first order in the same hydrogen sulfite fraction α₁, so α₁
//! and the volume cancel, and with peroxide in excess (constant) the
//! competition integrates in closed form:
//!
//! `s₀ = (i₀ - i) + β ln(i₀ / i)`, `β = k_H·h₀ / k_I` (moles),
//!
//! where `i` is the iodine left when S(IV) is exhausted, `k_I` the
//! triiodide-weighted iodine constant (Palmer et al. 1984 K = 698 L/mol,
//! [`crate::iodine_chemistry::split`]) and `k_H` the McArdle-Hoffmann
//! constant at the vessel's acidity and temperature. Limitations stated in
//! the coverage table `docs/sulfur-iv-coupling-coverage-20261111.md`: the
//! I₂/I₃⁻ split is the one at the start of the coupling, a direct SO₂(aq)
//! + I₂ path is not separately priced, Yiin-Margerum constants are 25 °C
//! values, and thiosulfate's own iodine titration (the curated row) runs
//! first.
//!
//! Every gap is a typed refusal, never a silent pool:
//! `not-modeled.sulfite-peroxide-outside-measured-acid` and
//! `not-modeled.sulfite-peroxide-slow`.

use crate::ops::Event;
use crate::species::Phase;
use crate::units::Moles;
use crate::vessel::Vessel;
use crate::SpeciesId;

const TRACE: f64 = 1e-12;

/// Yiin & Margerum 1990 (Yiin dissertation abstract), 25.0 °C, μ = 0.50
/// M (M⁻¹ s⁻¹): I₂ + HSO₃⁻.
pub const K_I2_HSO3: f64 = 1.8e9;
/// Yiin & Margerum 1990: I₃⁻ + HSO₃⁻.
pub const K_I3_HSO3: f64 = 1.4e7;
/// Yiin & Margerum 1990: I₂ + SO₃²⁻.
pub const K_I2_SO3: f64 = 3.2e9;
/// Yiin & Margerum 1990: I₃⁻ + SO₃²⁻.
pub const K_I3_SO3: f64 = 2.8e8;

/// Hoffmann & Calvert 1985 (Vol. II summary table): k of
/// `k[H⁺][H₂O₂][HSO₃⁻]/(1 + K[H⁺])`, M⁻² s⁻¹ at 25 °C.
pub const HC_K_M2_S1: f64 = 7.5e7;
/// Hoffmann & Calvert 1985: K of the same law, M⁻¹ at 25 °C (held
/// constant over the bench range).
pub const HC_KEQ_M1: f64 = 13.0;
/// Temperature of the Hoffmann & Calvert constants, K.
pub const HC_T_K: f64 = 298.15;

/// Hoffmann & Calvert 1985 ozone constants on (SO₂·H₂O, HSO₃⁻, SO₃²⁻),
/// M⁻¹ s⁻¹ at 25 °C ("k0 = 2.40E+04 k1 = 3.70E+05 k2 = 1.50E+09").
pub const O3_K: [f64; 3] = [2.4e4, 3.7e5, 1.5e9];
/// Troy & Margerum 1991 (as quoted by Liu et al. 2020): HOBr + SO₃²⁻,
/// M⁻¹ s⁻¹ at 25 °C.
pub const HOBR_K_SO3: f64 = 5.0e9;
/// Troy & Margerum 1991 (as quoted, JPCA 113 (2009) 9988): OBr⁻ + SO₃²⁻.
pub const OBR_K_SO3: f64 = 1.0e8;
/// Fogelman, Walker & Margerum 1989: HOCl + SO₃²⁻, M⁻¹ s⁻¹ at 25 °C.
pub const HOCL_K_SO3: f64 = 7.6e8;
/// Oblath 1981 (LBL-13026, Table II): k of `k[NO₂⁻][HSO₃⁻][H⁺]` at
/// 22 °C, μ 0.20, M⁻² s⁻¹: lowest and highest of the four runs.
pub const NITRITE_K_M2_S1: (f64, f64) = (4.0e3, 4.5e3);

/// McArdle & Hoffmann 1983: k₁ at 15 °C, M⁻¹ s⁻¹.
pub const MH_K1_M1_S1: f64 = 2.6e6;
/// McArdle & Hoffmann 1983: k₂/k₋₁, M⁻¹ (15 °C; held constant).
pub const MH_K2_OVER_KM1_M1: f64 = 16.0;
/// McArdle & Hoffmann 1983: ΔH‡ of k₁, J/mol.
pub const MH_DH_ACT_J_MOL: f64 = 37_000.0;
/// Temperature of the McArdle & Hoffmann constants, K.
pub const MH_T_K: f64 = 288.15;
/// Measured acidity range of the McArdle & Hoffmann law: pH 0.0-4.5,
/// as [H⁺] in mol/L (low, high).
pub const MH_PROTON_RANGE_M: (f64, f64) = (3.162_277_660_168_379_5e-5, 1.0);

/// Longest pseudo-first-order S(IV) lifetime (s) the peroxide path is
/// treated as instantaneous for. A modelling bound, not a constant of
/// nature: above it the coupling is refused rather than integrated.
pub const MAX_INSTANT_S: f64 = 1.0;

/// Aqueous bench range the 25 °C constants are used over (same as the
/// halogen writers).
pub const DOMAIN_K: (f64, f64) = crate::halogen_redox::DOMAIN_K;

const R_GAS: f64 = 8.314_462_618;

/// One S(IV) ion: its key, O and H counts and charge.
#[derive(Debug, Clone, Copy)]
pub struct SivIon {
    pub key: &'static str,
    pub oxygen: f64,
    pub hydrogen: f64,
    pub charge: f64,
}

/// The S(IV) members the writer owns.
pub const IONS: &[SivIon] = &[
    SivIon {
        key: "SO2",
        oxygen: 2.0,
        hydrogen: 0.0,
        charge: 0.0,
    },
    SivIon {
        key: "HSO3-",
        oxygen: 3.0,
        hydrogen: 1.0,
        charge: -1.0,
    },
    SivIon {
        key: "SO3-2",
        oxygen: 3.0,
        hydrogen: 0.0,
        charge: -2.0,
    },
];

/// Oxidants whose S(IV) shares are integrated in closed form (KI-011).
pub const OXIDANTS: &[&str] = &["I2", "H2O2"];

/// How an oxidant's S(IV) oxidation is priced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pricing {
    /// Competing rate laws in closed form ([`OXIDANTS`]).
    Competing,
    /// Second-order constants per S(IV) form ([`SivOxidant::k`]).
    Rated,
    /// Instantaneous stoichiometric titration; no rate constant claimed.
    Stoichiometric,
    /// An admitted `halogen_redox` row owns it.
    HalogenWriter,
    /// The product is not a registry species.
    ProductNotInRegistry,
    /// No reviewed rate holds for it.
    NoReviewedRate,
}

impl Pricing {
    pub fn key(self) -> &'static str {
        match self {
            Pricing::Competing => "competing",
            Pricing::Rated => "rated",
            Pricing::Stoichiometric => "stoichiometric",
            Pricing::HalogenWriter => "halogen_writer",
            Pricing::ProductNotInRegistry => "product_not_in_registry",
            Pricing::NoReviewedRate => "no_reviewed_rate",
        }
    }
}

/// One oxidant of S(IV): its reduction half
/// `key + electrons e⁻ + protons H⁺ → products + water H₂O`, its rate
/// constants on (SO₂, HSO₃⁻, SO₃²⁻) where measured, and its pricing.
#[derive(Debug, Clone, Copy)]
pub struct SivOxidant {
    pub key: &'static str,
    pub products: &'static [(&'static str, f64)],
    pub electrons: f64,
    pub protons: f64,
    pub water: f64,
    pub k: [Option<f64>; 3],
    pub pricing: Pricing,
}

const fn ox(
    key: &'static str,
    products: &'static [(&'static str, f64)],
    electrons: f64,
    protons: f64,
    water: f64,
    k: [Option<f64>; 3],
    pricing: Pricing,
) -> SivOxidant {
    SivOxidant {
        key,
        products,
        electrons,
        protons,
        water,
        k,
        pricing,
    }
}

/// Every S(IV) oxidant the bench names (sources in the module docs and
/// `docs/sulfur-iv-coupling-coverage-20261111.md`).
pub const OXIDANT_TABLE: &[SivOxidant] = &[
    ox(
        "I2",
        &[("I-", 2.0)],
        2.0,
        0.0,
        0.0,
        [None, Some(K_I2_HSO3), Some(K_I2_SO3)],
        Pricing::Competing,
    ),
    ox(
        "H2O2",
        &[],
        2.0,
        2.0,
        2.0,
        [None, None, None],
        Pricing::Competing,
    ),
    ox(
        "O3",
        &[("O2", 1.0)],
        2.0,
        2.0,
        1.0,
        [Some(O3_K[0]), Some(O3_K[1]), Some(O3_K[2])],
        Pricing::Rated,
    ),
    ox(
        "HBrO",
        &[("Br-", 1.0)],
        2.0,
        1.0,
        1.0,
        [None, None, Some(HOBR_K_SO3)],
        Pricing::Rated,
    ),
    ox(
        "BrO-",
        &[("Br-", 1.0)],
        2.0,
        2.0,
        1.0,
        [None, None, Some(OBR_K_SO3)],
        Pricing::Rated,
    ),
    ox(
        "HClO",
        &[("Cl-", 1.0)],
        2.0,
        1.0,
        1.0,
        [None, None, Some(HOCL_K_SO3)],
        Pricing::HalogenWriter,
    ),
    ox(
        "ClO-",
        &[("Cl-", 1.0)],
        2.0,
        2.0,
        1.0,
        [None, None, None],
        Pricing::HalogenWriter,
    ),
    ox(
        "Br2",
        &[("Br-", 2.0)],
        2.0,
        0.0,
        0.0,
        [None, None, None],
        Pricing::Stoichiometric,
    ),
    ox(
        "MnO4-",
        &[("Mn+2", 1.0)],
        5.0,
        8.0,
        4.0,
        [None, None, None],
        Pricing::Stoichiometric,
    ),
    ox(
        "Cr2O7-2",
        &[("Cr+3", 2.0)],
        6.0,
        14.0,
        7.0,
        [None, None, None],
        Pricing::Stoichiometric,
    ),
    ox(
        "HCrO4-",
        &[("Cr+3", 1.0)],
        3.0,
        7.0,
        4.0,
        [None, None, None],
        Pricing::Stoichiometric,
    ),
    ox(
        "NO2-",
        &[],
        0.0,
        0.0,
        0.0,
        [None, None, None],
        Pricing::ProductNotInRegistry,
    ),
    ox(
        "O2",
        &[],
        0.0,
        0.0,
        0.0,
        [None, None, None],
        Pricing::NoReviewedRate,
    ),
];

/// The product Oblath 1981 reports for nitrite + hydrogen sulfite, named
/// in the refusal (it is not a registry species).
pub const NITRITE_PRODUCT: &str = "hydroxylamine disulfonate";

/// The table row for `key`.
pub fn oxidant(key: &str) -> Option<&'static SivOxidant> {
    OXIDANT_TABLE.iter().find(|o| o.key == key)
}

/// Index of an S(IV) ion in [`SivOxidant::k`].
pub fn ion_index(ion: &SivIon) -> usize {
    match ion.key {
        "SO2" => 0,
        "HSO3-" => 1,
        _ => 2,
    }
}

/// Per mole of S(IV) of `ion` oxidised by `ox`:
/// `(oxidant moles, water consumed, protons consumed)`; negative water or
/// protons are released.
pub fn generic_path(ion: &SivIon, ox: &SivOxidant) -> (f64, f64, f64) {
    let per = 2.0 / ox.electrons;
    (
        per,
        (4.0 - ion.oxygen) - per * ox.water,
        per * ox.protons - (4.0 + ion.charge),
    )
}

/// Per mole of S(IV) of `ion` oxidised by I₂:
/// `(water consumed, protons released)`.
pub fn iodine_path(ion: &SivIon) -> (f64, f64) {
    (4.0 - ion.oxygen, 4.0 + ion.charge)
}

/// Per mole of S(IV) of `ion` oxidised by H₂O₂:
/// `(water released, protons released)`.
pub fn peroxide_path(ion: &SivIon) -> (f64, f64) {
    (ion.oxygen - 2.0, 2.0 + ion.charge)
}

/// Registry keys delivering each ion: `(form, ion)`.
pub fn forms() -> Vec<(String, &'static SivIon)> {
    let mut out = Vec::new();
    for ion in IONS {
        for form in crate::halogen_redox::registry_forms(ion.key) {
            out.push((form, ion));
        }
    }
    out
}

fn liquid_water(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

fn in_medium(vessel: &Vessel) -> bool {
    (DOMAIN_K.0..=DOMAIN_K.1).contains(&vessel.temperature.0) && liquid_water(vessel) > 0.0
}

/// Phases a form is drawn from. `SO2` booked as gas by a rate law (the
/// thiosulfate-acid law) is still held by the vessel over its liquid
/// water and counts; salts are drawn dissolved first, then solid.
fn phases_of(form: &str) -> &'static [Phase] {
    if form == "SO2" {
        &[Phase::Aqueous, Phase::Gas]
    } else {
        &[Phase::Aqueous, Phase::Solid]
    }
}

fn amount(vessel: &Vessel, key: &str, phases: &[Phase]) -> f64 {
    let id = SpeciesId::new(key);
    let id = id.canonical();
    vessel
        .contents
        .iter()
        .filter(|p| &p.species == id.as_ref() && phases.contains(&p.phase))
        .map(|p| p.moles.0.max(0.0))
        .sum()
}

/// S(IV) moles in this vessel that the writer can reach, per form.
pub fn pool(vessel: &Vessel) -> Vec<(String, &'static SivIon, f64)> {
    if !in_medium(vessel) {
        return Vec::new();
    }
    forms()
        .into_iter()
        .map(|(f, ion)| {
            let n = amount(vessel, &f, phases_of(&f));
            (f, ion, n)
        })
        .filter(|(_, _, n)| *n > TRACE)
        .collect()
}

/// Remaining S(IV) reducing capacity in moles of I₂ (two electrons per
/// S(IV)) — the sink capacity S(IV) adds beside thiosulfate.
pub fn reducing_capacity_i2_equivalents(vessel: &Vessel) -> f64 {
    pool(vessel).iter().map(|(_, _, n)| n).sum()
}

const IODINE_PHASES: &[Phase] = &[Phase::Aqueous, Phase::Solid];
const PEROXIDE_PHASES: &[Phase] = &[Phase::Liquid, Phase::Aqueous];

fn litres(vessel: &Vessel) -> f64 {
    vessel.liquid_volume().0.max(1e-9)
}

/// Free proton concentration (mol/L): the ledger's H⁺ row, or the
/// anion charge excess where released protons are booked as charge (the
/// halogen writers' convention).
fn proton_molar(vessel: &Vessel) -> f64 {
    let row = amount(vessel, "H+", &[Phase::Aqueous]);
    let charge = (-crate::displacement::solute_charge(vessel)).max(0.0);
    row.max(charge) / litres(vessel)
}

/// Whether the halogen table's admitted iodine/sulfite writers own this
/// ion when no peroxide competes (they hold I₂ + HSO₃⁻/SO₃²⁻ and their
/// salts; SO₂ has no half there).
fn halogen_owned(ion: &SivIon) -> bool {
    ion.key != "SO2"
}

/// Eyring factor with the McArdle-Hoffmann ΔH‡ from `t_ref` to `t_k`.
fn eyring(t_k: f64, t_ref: f64) -> f64 {
    (t_k / t_ref) * (-(MH_DH_ACT_J_MOL / R_GAS) * (1.0 / t_k - 1.0 / t_ref)).exp()
}

/// McArdle-Hoffmann k₁ at `t_k` (Eyring with ΔH‡₁ from 15 °C). Used as
/// the independent cross-check of the 25 °C Hoffmann-Calvert constant.
pub fn mh_k1(t_k: f64) -> f64 {
    MH_K1_M1_S1 * eyring(t_k, MH_T_K)
}

/// Effective second-order peroxide constant on HSO₃⁻ (M⁻¹ s⁻¹),
/// `k[H⁺]/(1 + K[H⁺])` with the Hoffmann-Calvert 25 °C constants moved by
/// the McArdle-Hoffmann ΔH‡, or `None` outside the measured acidity range.
pub fn peroxide_k(t_k: f64, proton_m: f64) -> Option<f64> {
    if !(MH_PROTON_RANGE_M.0..=MH_PROTON_RANGE_M.1).contains(&proton_m) {
        return None;
    }
    Some(HC_K_M2_S1 * proton_m / (1.0 + HC_KEQ_M1 * proton_m) * eyring(t_k, HC_T_K))
}

/// Triiodide-weighted iodine constant on HSO₃⁻ (M⁻¹ s⁻¹) for this
/// vessel's iodine/iodide split.
pub fn iodine_k(vessel: &Vessel) -> f64 {
    match crate::iodine_chemistry::speciate(vessel) {
        Some(s) if s.iodine + s.triiodide > 0.0 => {
            (s.iodine * K_I2_HSO3 + s.triiodide * K_I3_HSO3) / (s.iodine + s.triiodide)
        }
        _ => K_I2_HSO3,
    }
}

/// Iodine left (mol) when `s0` mol of S(IV) has been shared between `i0`
/// mol of iodine and peroxide with competition parameter `beta` (mol),
/// from `s0 = (i0 - i) + beta ln(i0 / i)`.
pub fn iodine_left(s0: f64, i0: f64, beta: f64) -> f64 {
    if i0 <= 0.0 || s0 <= 0.0 {
        return i0.max(0.0);
    }
    if beta <= 0.0 {
        return (i0 - s0).max(0.0);
    }
    let f = |i: f64| (i0 - i) + beta * (i0 / i).ln() - s0;
    // f is decreasing in i, +inf at 0 and -s0 at i0.
    let (mut lo, mut hi) = ((i0 - s0).max(0.0).max(i0 * 1e-300), i0);
    if f(lo) <= 0.0 {
        return lo;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if f(mid) > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Why the peroxide path is not run in this vessel, if it is not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PeroxideGap {
    OutsideMeasuredAcid { proton_m: f64 },
    Slow { lifetime_s: f64 },
}

/// The planned split: S(IV) moles to iodine and to peroxide, and any
/// peroxide refusal.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    pub s_iv: f64,
    /// S(IV) moles the iodine share is spread over: all of it when
    /// peroxide competes, otherwise only the forms the halogen writers do
    /// not own.
    pub iodine_basis: f64,
    /// Whether the peroxide path is admitted (so this writer owns every
    /// S(IV) form).
    pub peroxide_competes: bool,
    pub to_iodine: f64,
    pub to_peroxide: f64,
    pub peroxide_gap: Option<PeroxideGap>,
}

/// Plan the coupling for this vessel, or `None` when there is no S(IV)
/// or no oxidant present.
pub fn plan(vessel: &Vessel) -> Option<Plan> {
    let pool = pool(vessel);
    let s0: f64 = pool.iter().map(|(_, _, n)| n).sum();
    if s0 <= TRACE {
        return None;
    }
    let i0 = amount(vessel, "I2", IODINE_PHASES);
    let h0 = amount(vessel, "H2O2", PEROXIDE_PHASES);
    if i0 <= TRACE && h0 <= TRACE {
        return None;
    }
    let mut gap = None;
    let mut k_h = 0.0;
    if h0 > TRACE {
        let proton = proton_molar(vessel);
        match peroxide_k(vessel.temperature.0, proton) {
            None => gap = Some(PeroxideGap::OutsideMeasuredAcid { proton_m: proton }),
            Some(k) => {
                let lifetime = 1.0 / (k * h0 / litres(vessel));
                if lifetime <= MAX_INSTANT_S {
                    k_h = k;
                } else {
                    gap = Some(PeroxideGap::Slow {
                        lifetime_s: lifetime,
                    });
                }
            }
        }
    }
    let competes = k_h > 0.0;
    let basis: f64 = if competes {
        s0
    } else {
        pool.iter()
            .filter(|(_, ion, _)| !halogen_owned(ion))
            .map(|(_, _, n)| n)
            .sum()
    };
    let (to_iodine, to_peroxide) = if !competes {
        (basis.min(i0), 0.0)
    } else if i0 <= TRACE {
        (0.0, s0.min(h0))
    } else {
        let beta = k_h * h0 / iodine_k(vessel);
        let di = i0 - iodine_left(s0, i0, beta);
        let dh = (s0 - di).max(0.0);
        if dh <= h0 {
            (di, dh)
        } else {
            // Peroxide not in excess after all: it is spent and the rest
            // goes to iodine as far as iodine reaches.
            ((s0 - h0).min(i0), h0)
        }
    };
    Some(Plan {
        s_iv: s0,
        iodine_basis: basis,
        peroxide_competes: competes,
        to_iodine,
        to_peroxide,
        peroxide_gap: gap,
    })
}

/// Whether the writer would change this vessel now.
pub fn fires(vessel: &Vessel) -> bool {
    plan(vessel).is_some_and(|p| p.to_iodine + p.to_peroxide > TRACE)
        || other_plan(vessel).is_some_and(|p| p.run.is_some_and(|r| r.s_iv > TRACE))
}

// ── Every other S(IV) oxidant: one generic writer ───────────────────────

const OTHER_PHASES: &[Phase] = &[Phase::Aqueous, Phase::Solid];

fn oxidant_phases(ox: &SivOxidant) -> &'static [Phase] {
    match ox.key {
        "H2O2" => PEROXIDE_PHASES,
        "I2" => IODINE_PHASES,
        "O2" | "O3" => &[Phase::Aqueous],
        _ => OTHER_PHASES,
    }
}

/// Registry forms delivering an oxidant (the ion and its Na/K salts).
fn oxidant_forms(ox: &SivOxidant) -> Vec<String> {
    crate::halogen_redox::registry_forms(ox.key)
}

/// Moles of `ox` the vessel holds, over all its forms.
pub fn oxidant_amount(vessel: &Vessel, ox: &SivOxidant) -> f64 {
    oxidant_forms(ox)
        .iter()
        .map(|f| amount(vessel, f, oxidant_phases(ox)))
        .sum()
}

/// Why an S(IV) oxidant other than iodine and peroxide is not run.
#[derive(Debug, Clone, PartialEq)]
pub enum OtherGap {
    /// A rated oxidant lives longer than [`MAX_INSTANT_S`] with S(IV).
    Slow {
        oxidant: &'static str,
        lifetime_s: f64,
    },
    /// No measured constant for a present S(IV) form.
    NoRateForForm {
        oxidant: &'static str,
        form: &'static str,
    },
    /// The reaction's product is not a registry species.
    ProductNotInRegistry {
        oxidant: &'static str,
        product: &'static str,
    },
    /// No reviewed rate (dissolved O₂: autoxidation).
    NoReviewedRate { oxidant: &'static str },
    /// Represented acidity runs out before the S(IV) or the oxidant.
    AcidLimited { oxidant: &'static str, left: f64 },
    /// Several S(IV) oxidants are present and no shared rate prices
    /// their competition.
    Competition { oxidant: &'static str },
}

/// The admitted run of one other oxidant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OtherRun {
    pub oxidant: &'static str,
    /// S(IV) moles oxidised (spread over the forms by their amounts).
    pub s_iv: f64,
}

/// Plan for every S(IV) oxidant outside the iodine/peroxide competition.
#[derive(Debug, Clone, PartialEq)]
pub struct OtherPlan {
    pub run: Option<OtherRun>,
    pub gaps: Vec<OtherGap>,
}

/// Plan the other oxidants in this vessel, or `None` when there is no
/// S(IV) or no such oxidant present.
pub fn other_plan(vessel: &Vessel) -> Option<OtherPlan> {
    let pool = pool(vessel);
    let s0: f64 = pool.iter().map(|(_, _, n)| n).sum();
    if s0 <= TRACE {
        return None;
    }
    let present: Vec<(&'static SivOxidant, f64)> = OXIDANT_TABLE
        .iter()
        .map(|o| (o, oxidant_amount(vessel, o)))
        .filter(|(_, n)| *n > TRACE)
        .collect();
    let mut gaps = Vec::new();
    let runnable: Vec<(&'static SivOxidant, f64)> = present
        .iter()
        .copied()
        .filter(|(o, _)| matches!(o.pricing, Pricing::Rated | Pricing::Stoichiometric))
        .collect();
    for (o, _) in &present {
        match o.pricing {
            Pricing::ProductNotInRegistry => gaps.push(OtherGap::ProductNotInRegistry {
                oxidant: o.key,
                product: NITRITE_PRODUCT,
            }),
            Pricing::NoReviewedRate => gaps.push(OtherGap::NoReviewedRate { oxidant: o.key }),
            _ => {}
        }
    }
    if runnable.is_empty() {
        return (!gaps.is_empty()).then_some(OtherPlan { run: None, gaps });
    }
    let competitors = present
        .iter()
        .filter(|(o, _)| {
            matches!(
                o.pricing,
                Pricing::Competing
                    | Pricing::HalogenWriter
                    | Pricing::Rated
                    | Pricing::Stoichiometric
            )
        })
        .count();
    if competitors > 1 {
        for (o, _) in &runnable {
            gaps.push(OtherGap::Competition { oxidant: o.key });
        }
        return Some(OtherPlan { run: None, gaps });
    }
    let (o, n_ox) = runnable[0];
    if o.pricing == Pricing::Rated {
        let litres = litres(vessel);
        let mut longest: f64 = 0.0;
        for (_, ion, _) in &pool {
            let Some(k) = o.k[ion_index(ion)] else {
                gaps.push(OtherGap::NoRateForForm {
                    oxidant: o.key,
                    form: ion.key,
                });
                return Some(OtherPlan { run: None, gaps });
            };
            longest = longest.max(1.0 / (k * n_ox / litres));
        }
        if longest > MAX_INSTANT_S {
            gaps.push(OtherGap::Slow {
                oxidant: o.key,
                lifetime_s: longest,
            });
            return Some(OtherPlan { run: None, gaps });
        }
    }
    // Capacity: S(IV) itself, the oxidant (2/electrons per S(IV)), and the
    // represented acidity the net equation draws per S(IV).
    let reach = s0.min(n_ox * o.electrons / 2.0);
    let demand_per_s: f64 = pool
        .iter()
        .map(|(_, ion, n)| n * generic_path(ion, o).2.max(0.0))
        .sum::<f64>()
        / s0;
    let mut s = reach;
    if demand_per_s > 0.0 {
        s = s.min(crate::displacement::unspent_acidity(vessel).max(0.0) / demand_per_s);
    }
    if reach - s > TRACE {
        gaps.push(OtherGap::AcidLimited {
            oxidant: o.key,
            left: reach - s,
        });
    }
    let run = (s > TRACE).then_some(OtherRun {
        oxidant: o.key,
        s_iv: s,
    });
    Some(OtherPlan { run, gaps })
}

fn withdraw_oxidant(vessel: &mut Vessel, ox: &SivOxidant, moles: f64) {
    let mut left = moles;
    for form in oxidant_forms(ox) {
        for phase in oxidant_phases(ox) {
            if left <= 0.0 {
                return;
            }
            let taken = vessel
                .withdraw_phase(&SpeciesId::new(&form), Moles(left), *phase)
                .0;
            if taken > 0.0 {
                for (cation, k) in crate::halogen_redox::spectators_of(&form, ox.key) {
                    vessel.deposit(SpeciesId::new(cation), Moles(taken * k), Phase::Aqueous);
                }
            }
            left -= taken;
        }
    }
}

/// Display equation for one ion and a table oxidant, in whole numbers.
pub fn generic_equation(ion: &SivIon, ox: &SivOxidant) -> String {
    let scale = if (ox.electrons as i64) % 2 == 0 {
        ox.electrons / 2.0
    } else {
        ox.electrons
    };
    let (per, water_in, protons_in) = generic_path(ion, ox);
    let mut lhs = vec![term(per * scale, ox.key), term(scale, ion.key)];
    let mut rhs = Vec::new();
    if water_in > 1e-9 {
        lhs.push(term(water_in * scale, "water"));
    }
    if protons_in > 1e-9 {
        lhs.push(term(protons_in * scale, "H+"));
    }
    for (product, k) in ox.products {
        rhs.push(term(per * k * scale, product));
    }
    rhs.push(term(scale, "SO4-2"));
    if protons_in < -1e-9 {
        rhs.push(term(-protons_in * scale, "H+"));
    }
    if water_in < -1e-9 {
        rhs.push(term(-water_in * scale, "water"));
    }
    format!("{} → {}", lhs.join(" + "), rhs.join(" + "))
}

/// Run the other-oxidant plan. Returns whether anything changed.
pub fn apply_other(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    use crate::phrase::{Phrase, Slot};
    let Some(OtherPlan { run: Some(run), .. }) = other_plan(vessel) else {
        return false;
    };
    let Some(ox) = oxidant(run.oxidant) else {
        return false;
    };
    let pool = pool(vessel);
    let s0: f64 = pool.iter().map(|(_, _, n)| n).sum();
    let mut said: Vec<String> = Vec::new();
    for (form, ion, n) in pool {
        let x = run.s_iv * n / s0;
        if x <= TRACE {
            continue;
        }
        let (per, water_in, protons_in) = generic_path(ion, ox);
        withdraw(vessel, &form, x, phases_of(&form));
        withdraw_oxidant(vessel, ox, x * per);
        if protons_in > 0.0 {
            crate::displacement::consume_acidity(vessel, x * protons_in);
        }
        if water_in > 0.0 {
            vessel.withdraw_phase(&SpeciesId::new("water"), Moles(x * water_in), Phase::Liquid);
        } else if water_in < 0.0 {
            vessel.deposit(SpeciesId::new("water"), Moles(-x * water_in), Phase::Liquid);
        }
        vessel.deposit(SpeciesId::new("SO4-2"), Moles(x), Phase::Aqueous);
        for (product, k) in ox.products {
            vessel.deposit(SpeciesId::new(product), Moles(x * per * k), Phase::Aqueous);
        }
        for (cation, k) in crate::halogen_redox::spectators_of(&form, ion.key) {
            vessel.deposit(SpeciesId::new(cation), Moles(x * k), Phase::Aqueous);
        }
        let eq = generic_equation(ion, ox);
        if !said.contains(&eq) {
            events.push(Event::ReactionOccurred {
                vessel: vessel.id,
                equation: eq.clone(),
            });
            said.push(eq);
        }
    }
    if ox.pricing == Pricing::Stoichiometric {
        events.push(Event::not_modeled(
            vessel.id,
            crate::ops::NotModelledCause::RateNotModelled,
            Phrase::new(
                "not-modeled.sulfite-oxidant-no-rate-claimed",
                "{oxidant} oxidises sulfur(IV) to sulfate here as an instantaneous stoichiometric titration; no rate constant is claimed for it, so the speed of the reaction is not modelled",
                vec![("oxidant".to_string(), Slot::text(pretty(ox.key)))],
            ),
        ));
    }
    true
}

fn pretty(key: &str) -> String {
    crate::halogen_redox::pretty(key)
}

fn term(coef: f64, key: &str) -> String {
    if (coef - 1.0).abs() < 1e-9 {
        pretty(key)
    } else {
        format!("{} {}", coef.round() as i64, pretty(key))
    }
}

/// Display equation for one ion and oxidant.
pub fn equation(ion: &SivIon, oxidant: &str) -> String {
    let mut lhs = vec![pretty(oxidant), pretty(ion.key)];
    let mut rhs = Vec::new();
    if oxidant == "I2" {
        let (w, h) = iodine_path(ion);
        if w > 0.0 {
            lhs.push(term(w, "water"));
        }
        rhs.push(term(2.0, "I-"));
        rhs.push(pretty("SO4-2"));
        if h > 0.0 {
            rhs.push(term(h, "H+"));
        }
    } else {
        rhs.push(pretty("SO4-2"));
        let (w, h) = peroxide_path(ion);
        if h > 0.0 {
            rhs.push(term(h, "H+"));
        }
        if w > 0.0 {
            rhs.push(term(w, "water"));
        }
    }
    format!("{} → {}", lhs.join(" + "), rhs.join(" + "))
}

fn withdraw(vessel: &mut Vessel, key: &str, moles: f64, phases: &[Phase]) {
    let mut left = moles;
    for phase in phases {
        if left <= 0.0 {
            break;
        }
        left -= vessel
            .withdraw_phase(&SpeciesId::new(key), Moles(left), *phase)
            .0;
    }
}

/// Run the coupling to completion. Returns whether anything changed. The
/// caller refreshes `solute_charge`.
pub fn apply(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let other = apply_other(vessel, events);
    apply_competing(vessel, events) || other
}

fn apply_competing(vessel: &mut Vessel, events: &mut Vec<Event>) -> bool {
    let Some(plan) = plan(vessel) else {
        return false;
    };
    if plan.to_iodine + plan.to_peroxide <= TRACE {
        return false;
    }
    let pool = pool(vessel);
    let s0 = plan.s_iv;
    let mut said: Vec<String> = Vec::new();
    for (form, ion, n) in pool {
        let share = n / s0;
        let iodine_share = if plan.peroxide_competes {
            share
        } else if halogen_owned(ion) || plan.iodine_basis <= 0.0 {
            0.0
        } else {
            n / plan.iodine_basis
        };
        let spectators = crate::halogen_redox::spectators_of(&form, ion.key);
        for (oxidant, x) in [
            ("I2", plan.to_iodine * iodine_share),
            ("H2O2", plan.to_peroxide * share),
        ] {
            if x <= TRACE {
                continue;
            }
            withdraw(vessel, &form, x, phases_of(&form));
            let (water_in, water_out, _protons) = if oxidant == "I2" {
                withdraw(vessel, "I2", x, IODINE_PHASES);
                let (w, h) = iodine_path(ion);
                (w, 0.0, h)
            } else {
                withdraw(vessel, "H2O2", x, PEROXIDE_PHASES);
                let (w, h) = peroxide_path(ion);
                (0.0, w, h)
            };
            if water_in > 0.0 {
                vessel.withdraw_phase(&SpeciesId::new("water"), Moles(x * water_in), Phase::Liquid);
            }
            if water_out > 0.0 {
                vessel.deposit(SpeciesId::new("water"), Moles(x * water_out), Phase::Liquid);
            }
            vessel.deposit(SpeciesId::new("SO4-2"), Moles(x), Phase::Aqueous);
            if oxidant == "I2" {
                vessel.deposit(SpeciesId::new("I-"), Moles(2.0 * x), Phase::Aqueous);
            }
            // Released protons stay in the charge balance (the ledger's
            // strong-acid convention, as in `halogen_redox`); `protons` is
            // the equation's count and is checked by the closure test.
            for (cation, k) in &spectators {
                vessel.deposit(SpeciesId::new(*cation), Moles(x * k), Phase::Aqueous);
            }
            let eq = equation(ion, oxidant);
            if !said.contains(&eq) {
                events.push(Event::ReactionOccurred {
                    vessel: vessel.id,
                    equation: eq.clone(),
                });
                said.push(eq);
            }
        }
    }
    true
}

/// Whether this module covers the halogen table's `oxidant + reductant`
/// group (members as the table spells them).
pub fn covers(oxidant: &str, reductant: &str) -> bool {
    OXIDANT_TABLE.iter().any(|o| o.key == oxidant) && IONS.iter().any(|i| i.key == reductant)
}

/// Typed notes for S(IV) left beside an oxidant that the writers refuse.
pub fn unpriced_notes(vessel: &Vessel) -> Vec<Event> {
    let mut events = peroxide_notes(vessel);
    events.extend(other_notes(vessel));
    events
}

fn peroxide_notes(vessel: &Vessel) -> Vec<Event> {
    use crate::phrase::{Phrase, Slot};
    let Some(plan) = plan(vessel) else {
        return Vec::new();
    };
    let left = plan.s_iv - plan.to_iodine - plan.to_peroxide;
    let mut events = Vec::new();
    match plan.peroxide_gap {
        Some(PeroxideGap::OutsideMeasuredAcid { proton_m }) => {
            events.push(Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::NoReviewedDatum,
                Phrase::new(
                    "not-modeled.sulfite-peroxide-outside-measured-acid",
                    "sulfur(IV) and hydrogen peroxide are together here, but the reviewed rate law for this oxidation was measured between pH 0 and 4.5 and here (H+) is {proton} mol/L, so the peroxide oxidation of {amount} mol of sulfur(IV) is not priced and it is left unreacted. That is a gap in the model, not evidence that nothing happens",
                    vec![
                        ("proton".to_string(), Slot::text(format!("{proton_m:.3e}"))),
                        ("amount".to_string(), Slot::text(format!("{left:.3e}"))),
                    ],
                ),
            ));
        }
        Some(PeroxideGap::Slow { lifetime_s }) => {
            events.push(Event::not_modeled(
                vessel.id,
                crate::ops::NotModelledCause::ModelBoundary,
                Phrase::new(
                    "not-modeled.sulfite-peroxide-slow",
                    "sulfur(IV) and hydrogen peroxide are together here, but at these concentrations the reviewed rate law gives sulfur(IV) a lifetime of about {lifetime} s, too slow to treat as instantaneous, and no time-resolved writer integrates it, so {amount} mol of sulfur(IV) is left unreacted",
                    vec![
                        ("lifetime".to_string(), Slot::text(format!("{lifetime_s:.3e}"))),
                        ("amount".to_string(), Slot::text(format!("{left:.3e}"))),
                    ],
                ),
            ));
        }
        None => {}
    }
    if left <= TRACE {
        events.clear();
    }
    events
}

fn other_note(vessel: &Vessel, gap: &OtherGap, s_iv: f64) -> Event {
    use crate::ops::NotModelledCause as Cause;
    use crate::phrase::{Phrase, Slot};
    let name = |k: &str| ("oxidant".to_string(), Slot::text(pretty(k)));
    let amount = ("amount".to_string(), Slot::text(format!("{s_iv:.3e}")));
    let (cause, key, en, slots) = match gap {
        OtherGap::Slow {
            oxidant,
            lifetime_s,
        } => (
            Cause::ModelBoundary,
            "not-modeled.sulfite-oxidant-slow",
            "sulfur(IV) and {oxidant} are together here, but at these concentrations the reviewed rate constants give sulfur(IV) a lifetime of about {lifetime} s, too slow to treat as instantaneous, and no time-resolved writer integrates it, so {amount} mol of sulfur(IV) is left unreacted",
            vec![
                name(*oxidant),
                ("lifetime".to_string(), Slot::text(format!("{lifetime_s:.3e}"))),
                amount,
            ],
        ),
        OtherGap::NoRateForForm { oxidant, form } => (
            Cause::NoReviewedDatum,
            "not-modeled.sulfite-oxidant-no-rate-for-form",
            "sulfur(IV) and {oxidant} are together here, but no reviewed rate constant was found for {oxidant} with {form}, so {amount} mol of sulfur(IV) is left unreacted. That is a gap in the model, not evidence that nothing happens",
            vec![
                name(*oxidant),
                ("form".to_string(), Slot::text(pretty(form))),
                amount,
            ],
        ),
        OtherGap::ProductNotInRegistry { oxidant, .. } => (
            Cause::PhaseNotInRegistry,
            "not-modeled.sulfite-oxidant-product-not-in-registry",
            "sulfur(IV) and {oxidant} are together here; the reviewed kinetics (Oblath 1981) give hydroxylamine disulfonate, not sulfate, and that species is not in this lab's registry, so {amount} mol of sulfur(IV) is left unreacted. That is a gap in the model, not evidence that nothing happens",
            vec![name(*oxidant), amount],
        ),
        OtherGap::NoReviewedRate { oxidant } => (
            Cause::NoReviewedDatum,
            "not-modeled.sulfite-autoxidation",
            "sulfur(IV) and dissolved {oxidant} are together here, but its autoxidation, uncatalysed or catalysed by iron or manganese, has no reviewed rate that holds without the catalyst, pH and ionic strength being pinned, so {amount} mol of sulfur(IV) is left unoxidised by it. That is a gap in the model, not evidence that nothing happens",
            vec![name(*oxidant), amount],
        ),
        OtherGap::AcidLimited { oxidant, left } => (
            Cause::ModelBoundary,
            "not-modeled.sulfite-oxidant-acid-limited",
            "{oxidant} oxidises sulfur(IV) here only as far as the represented acid reaches; in a neutral or basic solution its product is not pinned by a reviewed source, so {amount} mol of sulfur(IV) is left unreacted beside it",
            vec![
                name(*oxidant),
                ("amount".to_string(), Slot::text(format!("{left:.3e}"))),
            ],
        ),
        OtherGap::Competition { oxidant } => (
            Cause::RateNotModelled,
            "not-modeled.sulfite-oxidant-competition-unpriced",
            "{oxidant} is one of several sulfur(IV) oxidants here and no shared reviewed rate prices how they share the sulfur(IV), so {oxidant} is left unreacted with {amount} mol of sulfur(IV). That is a gap in the model, not evidence that nothing happens",
            vec![name(*oxidant), amount],
        ),
    };
    Event::not_modeled(vessel.id, cause, Phrase::new(key, en, slots))
}

/// Typed notes for the other oxidants' gaps in this vessel.
pub fn other_notes(vessel: &Vessel) -> Vec<Event> {
    let Some(plan) = other_plan(vessel) else {
        return Vec::new();
    };
    let s0 = reducing_capacity_i2_equivalents(vessel);
    plan.gaps
        .iter()
        .map(|g| other_note(vessel, g, s0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formula(key: &str) -> crate::stoich::Formula {
        let text = if key == "water" { "H2O" } else { key };
        crate::stoich::parse_formula(text).expect(key)
    }

    fn closes(left: &[(&str, f64)], right: &[(&str, f64)]) -> bool {
        let mut el: std::collections::BTreeMap<String, f64> = Default::default();
        let mut charge = 0.0;
        for (side, sign) in [(left, -1.0), (right, 1.0)] {
            for (k, n) in side {
                let f = formula(k);
                for (e, c) in f.counts {
                    *el.entry(e).or_default() += sign * n * c;
                }
                charge += sign * n * f.charge;
            }
        }
        charge.abs() < 1e-9 && el.values().all(|v| v.abs() < 1e-9)
    }

    #[test]
    fn every_s_iv_path_closes_atoms_and_charge() {
        for ion in IONS {
            let (w, h) = iodine_path(ion);
            assert!(closes(
                &[(ion.key, 1.0), ("I2", 1.0), ("water", w)],
                &[("SO4-2", 1.0), ("I-", 2.0), ("H+", h)]
            ));
            let (w, h) = peroxide_path(ion);
            assert!(closes(
                &[(ion.key, 1.0), ("H2O2", 1.0)],
                &[("SO4-2", 1.0), ("H+", h), ("water", w)]
            ));
            let f = formula(ion.key);
            assert_eq!(f.counts.get("O").copied().unwrap_or(0.0), ion.oxygen);
            assert_eq!(f.counts.get("H").copied().unwrap_or(0.0), ion.hydrogen);
            assert_eq!(f.charge, ion.charge);
        }
    }

    #[test]
    fn competition_root_closes_the_integrated_balance() {
        let (s0, i0, beta) = (2.5e-5, 1.5e-5, 1.7e-5);
        let i = iodine_left(s0, i0, beta);
        assert!(i > 0.0 && i < i0);
        let back = (i0 - i) + beta * (i0 / i).ln();
        assert!((back - s0).abs() < 1e-12 * s0.max(1.0) + 1e-18);
        assert!((iodine_left(1e-5, 3e-5, 0.0) - 2e-5).abs() < 1e-18);
    }

    #[test]
    fn peroxide_constant_is_hoffmann_calvert_at_25_c() {
        let k = peroxide_k(HC_T_K, 0.1).unwrap();
        assert!((k - 7.5e7 * 0.1 / 2.3).abs() < 1e-9 * k);
        assert!(peroxide_k(298.15, 1e-6).is_none());
    }

    #[test]
    fn mcardle_hoffmann_cross_checks_the_25_c_peroxide_constant() {
        // k1·(k2/k-1) at 15 °C moved to 25 °C with ΔH‡ 37 kJ/mol: within
        // 10 % of Hoffmann & Calvert's 7.5e7 M^-2 s^-1.
        let moved = mh_k1(HC_T_K) * MH_K2_OVER_KM1_M1;
        assert!((moved / HC_K_M2_S1 - 1.0).abs() < 0.10, "{moved}");
    }

    #[test]
    fn every_table_oxidant_path_closes_atoms_and_charge() {
        for o in OXIDANT_TABLE.iter().filter(|o| o.electrons > 0.0) {
            for ion in IONS {
                let (per, water_in, protons_in) = generic_path(ion, o);
                let mut left = vec![(ion.key, 1.0), (o.key, per), ("water", water_in)];
                left.push(("H+", protons_in));
                let mut right: Vec<(&str, f64)> = vec![("SO4-2", 1.0)];
                for (p, k) in o.products {
                    right.push((*p, per * k));
                }
                assert!(closes(&left, &right), "{} + {}", o.key, ion.key);
            }
            // The competing pair agrees with its own paths.
            if o.key == "H2O2" {
                for ion in IONS {
                    let (_, w, h) = generic_path(ion, o);
                    let (w2, h2) = peroxide_path(ion);
                    assert!((w + w2).abs() < 1e-12 && (h + h2).abs() < 1e-12);
                }
            }
        }
    }

    #[test]
    fn permanganate_equation_is_in_whole_numbers() {
        let mn = oxidant("MnO4-").unwrap();
        let hso3 = IONS.iter().find(|i| i.key == "HSO3-").unwrap();
        assert_eq!(
            generic_equation(hso3, mn),
            "2 MnO₄⁻ + 5 HSO₃⁻ + H⁺ → 2 Mn²⁺ + 5 SO₄²⁻ + 3 H₂O"
        );
    }
}

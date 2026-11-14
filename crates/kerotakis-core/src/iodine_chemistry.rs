//! Iodine family (2026-11-10): triiodide association, iodine optics and the
//! ferric/iodide equilibrium, as one generic module over the analytical
//! inventory.
//!
//! The ledger keeps dissolved iodine as one analytical pool, `I2(aq)`, and
//! iodide as `I-` (or an undissociated Na/K iodide salt). Every curated row,
//! rate law and halogen writer already reads and writes those keys, so the
//! triiodide ion is NOT a separate inventory entry: it is a computed split of
//! the two pools, used wherever the split changes an observable:
//!
//! * the dissolved-iodine capacity (iodine is far more soluble in iodide
//!   solution, which is what Lugol's solution is);
//! * the visible absorbance (free I2 and I3- absorb very differently);
//! * the ferric/iodide equilibrium (which consumes free iodide).
//!
//! Sources (constants at 25 °C, ideal activities = molarities):
//!
//! * `I2 + I- ⇌ I3-`, K = 698 (molal, used here as L/mol): Palmer, Ramette
//!   & Mesmer, *J. Solution Chem.* 13 (1984) 673, doi:10.1007/BF00650374. (Earlier
//!   spectrophotometric values span 698–770; Awtrey & Connick,
//!   *J. Am. Chem. Soc.* 73 (1951) 1842, doi:10.1021/ja01148a097.)
//! * ε(I2, 460 nm) = 722 and ε(I3-, 460 nm) = 1022 L mol⁻¹ cm⁻¹ at 25 °C
//!   (temperature coefficients −0.19 %/°C and +0.48 %/°C, measured 20–35 °C):
//!   Table 3 of "Colorimetric Determination of Dissolved Oxygen: Assessment
//!   of Methodological Influences on Iodine Spectra, Isosbestic Point,
//!   Precision and Accuracy", ACS Omega 9 (2024) 47350,
//!   doi:10.1021/acsomega.4c09084, open access PMC11603272
//!   (read 2026-11-11: "460 729 722 715 708 –0.19%" for I2 and
//!   "460 997 1022 1045 1070 0.48%" for I3-). This replaces the earlier
//!   unconfirmed I2 value of 746 and I3- 975 (Awtrey & Connick,
//!   *J. Am. Chem. Soc.* 73 (1951) 1842, doi:10.1021/ja01148a097).
//!   ε(I3-, 353 nm) = 26 400 L mol⁻¹ cm⁻¹: Awtrey & Connick (1951); later
//!   sources give 24 800–26 400. The temperature coefficients are applied
//!   by the cited single-wavelength reading
//!   ([`crate::uvvis_absorptivity`]); the 16-band colour spectrum is the
//!   25 °C one.
//! * E°(Fe³⁺/Fe²⁺) from the halogen redox half table (llnl.dat, 0.7696 V);
//!   E°(I2(aq)/I⁻) = 0.621 V: Bard, Parsons & Jordan, *Standard Potentials
//!   in Aqueous Solution* (IUPAC/Dekker, 1985).
//!
//! The starch–iodine complex keeps its editorial eye colour in
//! [`crate::starch_iodine`]; no reviewed molar absorptivity exists for it
//! here, so the spectrophotometer refuses a vessel holding the complex with
//! a typed reason instead of a number. Re-checked 2026-11-11
//! (batch3-diverse:en-001): the cited optical source itself states
//! "reliable extinction coefficients for the amylose-iodine-iodide complex
//! are not known" (Pesek et al., Molecules 27 (2022) 8974, section 2.1,
//! doi:10.3390/molecules27248974), and the blue-value assay (Corn Refiners
//! Association method AMYLO.01, 620 nm, 1 cm) calibrates against starches
//! of known amylose content instead of using an ε. See docs/starch-amylase-coverage-20261111.md.

use crate::species::Phase;
use crate::vessel::Vessel;
use crate::{Moles, SpeciesId};

/// Triiodide association constant at 25 °C, L/mol (Palmer et al. 1984).
pub const TRIIODIDE_K: f64 = 698.0;

/// Free I2(aq) visible band: maximum 460 nm, ε 722 L mol⁻¹ cm⁻¹ at 25 °C
/// (ACS Omega 2024, PMC11603272, Table 3); the width is the band's ≈100 nm
/// FWHM as a Gaussian σ.
pub const I2_BAND: (f64, f64, f64) = (460.0, 722.0, 42.0);

/// I3⁻: the 353 nm band (ε 26 400) whose visible tail colours the solution.
/// σ is FITTED so that ε(460 nm) = 1022 at 25 °C (ACS Omega 2024,
/// PMC11603272, Table 3), the second cited point:
/// σ = 107 / sqrt(2 ln(26400/1022)) = 41.96 nm.
pub const I3_BAND: (f64, f64, f64) = (353.0, 26_400.0, 41.96);

/// E°(I2(aq)/I⁻), V (Bard, Parsons & Jordan 1985).
pub const E0_I2AQ_IODIDE_V: f64 = 0.621;

/// Nernst slope at 25 °C, V per decade (RT ln10 / F).
const NERNST_V: f64 = 0.059_16;

/// Registry keys whose visible absorbance this module supplies.
pub const OPTICAL_KEYS: &[&str] = &["I2"];

const IODINE: &str = "I2";
const TRACE: f64 = 1e-12;

/// The triiodide split of dissolved iodine and iodide, in moles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TriiodideSplit {
    /// Free I2(aq).
    pub iodine: f64,
    /// I3⁻ (each holds one I2 and one I⁻ of the analytical pools).
    pub triiodide: f64,
    /// Free I⁻.
    pub iodide: f64,
}

/// Split `iodine_total` mol of dissolved I2 and `iodide_total` mol of
/// iodide in `litres` of solution by `I2 + I- ⇌ I3-`. Mass balance is exact
/// by construction: iodine + triiodide = iodine_total and iodide +
/// triiodide = iodide_total.
pub fn split(iodine_total: f64, iodide_total: f64, litres: f64) -> TriiodideSplit {
    let t = iodine_total.max(0.0);
    let b = iodide_total.max(0.0);
    if t <= 0.0 || b <= 0.0 || litres.is_nan() || litres <= 0.0 {
        return TriiodideSplit {
            iodine: t,
            triiodide: 0.0,
            iodide: b,
        };
    }
    // K z = (T - z)(B - z) / V in moles: K' = K / V.
    let k = TRIIODIDE_K / litres;
    let s = k * (t + b) + 1.0;
    let disc = (s * s - 4.0 * k * k * t * b).max(0.0);
    // Stable root of k z² - s z + k T B = 0.
    let z = (2.0 * k * t * b / (s + disc.sqrt())).clamp(0.0, t.min(b));
    TriiodideSplit {
        iodine: t - z,
        triiodide: z,
        iodide: b - z,
    }
}

/// Iodide in solution (moles of I⁻): the ion plus every dissolved Na/K
/// iodide salt the halogen table derives from the registry.
pub fn dissolved_iodide(vessel: &Vessel) -> f64 {
    crate::halogen_redox::registry_forms("I-")
        .iter()
        .map(|key| {
            vessel
                .contents
                .iter()
                .filter(|p| p.species.0 == *key && p.phase == Phase::Aqueous)
                .map(|p| p.moles.0)
                .sum::<f64>()
        })
        .sum()
}

fn aqueous(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && p.phase == Phase::Aqueous)
        .map(|p| p.moles.0)
        .sum()
}

fn liquid_water(vessel: &Vessel) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == "water" && p.phase == Phase::Liquid)
        .map(|p| p.moles.0)
        .sum()
}

/// The triiodide split of this vessel's dissolved iodine, or `None` when it
/// holds no liquid water.
pub fn speciate(vessel: &Vessel) -> Option<TriiodideSplit> {
    if liquid_water(vessel) <= 0.0 {
        return None;
    }
    Some(split(
        aqueous(vessel, IODINE),
        dissolved_iodide(vessel),
        vessel.liquid_volume().0.max(1e-9),
    ))
}

/// Dissolved-iodine capacity (mol) in iodide solution, from the pure-water
/// capacity `pure` (mol) in `water_litres`: at saturation free I2 is at its
/// pure-water solubility S0, and I3⁻ adds K·S0·[I⁻]free with
/// [I⁻]free = I_total / (1 + K·S0). Without iodide this is `pure` exactly.
pub fn enhanced_capacity(vessel: &Vessel, pure: f64, water_litres: f64) -> f64 {
    if water_litres.is_nan() || water_litres <= 0.0 || pure.is_nan() || pure <= 0.0 {
        return pure;
    }
    let iodide = dissolved_iodide(vessel) / water_litres;
    if iodide <= 0.0 {
        return pure;
    }
    let s0 = pure / water_litres;
    pure * (1.0 + TRIIODIDE_K * iodide / (1.0 + TRIIODIDE_K * s0))
}

/// Pure-water I2 capacity (mol) and the water volume (L), read exactly as
/// `solve::saturation_moves` reads every registry solubility.
pub fn pure_water_capacity(vessel: &Vessel) -> Option<(f64, f64)> {
    let water = liquid_water(vessel);
    if water <= 0.0 {
        return None;
    }
    let water_ml = crate::species::lookup_key("water")?
        .liters_from_moles(Moles(water))
        .0
        * 1000.0;
    let data = crate::species::lookup(&SpeciesId::new(IODINE))?;
    let limit = data.aqueous_solubility_at(vessel.temperature.0)?;
    Some((
        limit * water_ml / 100.0 / data.molar_mass,
        water_ml / 1000.0,
    ))
}

/// Solid I2 that dissolves now: up to the triiodide-enhanced capacity.
pub(crate) fn iodine_to_dissolve(vessel: &Vessel) -> Option<Moles> {
    let (pure, litres) = pure_water_capacity(vessel)?;
    let capacity = enhanced_capacity(vessel, pure, litres);
    let solid: f64 = vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == IODINE && p.phase == Phase::Solid)
        .map(|p| p.moles.0)
        .sum();
    Some(Moles(
        solid.min((capacity - aqueous(vessel, IODINE)).max(0.0)),
    ))
}

/// The split the optics read: dissolved iodine and iodide less the share
/// bound in the starch complex (`complex` mol, one I2 and one I⁻ each).
fn optical_parts(vessel: &Vessel, litres: f64, complex: f64) -> TriiodideSplit {
    split(
        (aqueous(vessel, IODINE) - complex).max(0.0),
        (dissolved_iodide(vessel) - complex).max(0.0),
        litres,
    )
}

/// The free I2 / I3⁻ / free I⁻ amounts (mol) behind this vessel's iodine
/// absorbance — the SAME split [`add_absorbance`] integrates, so a reading
/// and its reported speciation cannot disagree. `None` when the vessel
/// holds no liquid water or no dissolved iodine outside the starch complex.
pub fn optical_split(vessel: &Vessel) -> Option<TriiodideSplit> {
    if liquid_water(vessel) <= 0.0 {
        return None;
    }
    let litres = vessel.liquid_volume().0.max(1e-9);
    let parts = optical_parts(vessel, litres, crate::starch_iodine::complex_moles(vessel));
    (parts.iodine + parts.triiodide > 0.0).then_some(parts)
}

/// Add free-I2 and I3⁻ absorbance. Iodine bound in the starch complex
/// (`complex` mol, one I2 and one I⁻ each) is excluded: the complex has its
/// own band.
pub fn add_absorbance(
    vessel: &Vessel,
    litres: f64,
    path_cm: f64,
    complex: f64,
    absorbance: &mut crate::spectrum::Spectrum,
) {
    if liquid_water(vessel) <= 0.0 {
        return;
    }
    let litres = litres.max(1e-12);
    let parts = optical_parts(vessel, litres, complex);
    for (moles, band) in [(parts.iodine, I2_BAND), (parts.triiodide, I3_BAND)] {
        if moles <= 0.0 {
            continue;
        }
        let spectrum = crate::spectrum::bands(&[band]);
        for (total, epsilon) in absorbance.iter_mut().zip(spectrum) {
            *total += epsilon * moles / litres * path_cm;
        }
    }
}

// ---------------------------------------------------------------------------
// Oxidant/iodide equilibria: (2/n) Ox + 2 I⁻ ⇌ (2/n) Red + I2(aq)
// ---------------------------------------------------------------------------
//
// One generic solver for every couple of the halogen redox half table that
// the pair classifier calls EQUILIBRIUM-LIMITED against iodide (0 < ΔE° <
// `COMPLETE_MARGIN_V`), so it cannot honestly run to completion. A couple
// is solved here only when its half is a plain ion couple (no protons, no
// water, one reduced ion per oxidised ion), so K is pH-independent and the
// quotient is built from the two ions alone. On the current table that is
// Fe³⁺/Fe²⁺ only; `solves_pair` names the set and the coverage table
// `docs/iodide-redox-coverage-20261111.md` lists every oxidant × iodide pair
// with how it is handled.
//
// Inventory: the oxidant, reduced ion and iodide are read from EVERY
// dissolved form that delivers them — the bare ion and each registry salt
// of it (KI/NaI for iodide; FeCl3, FeSO4 … for iron). The aqueous route
// leaves a salt added this step undissociated until its own readback, and
// reading the bare ion only is exactly why ferric chloride and potassium
// iodide used to stand side by side unreacted (EC-008, RX-004). A salt
// share that reacts releases its counter-ions as free ions.

const IODIDE: &str = "I-";

/// Anions a cation salt may carry and still count as that cation's
/// dissolved form (a strong-acid anion that is a spectator in the couple).
const SALT_ANIONS: &[&str] = &["Cl-", "NO3-", "SO4-2", "ClO4-"];

/// One registry key delivering an ion: `per` ions per formula unit and the
/// counter-ions (key, count per ion) a reacting share releases.
#[derive(Debug, Clone)]
struct Form {
    key: String,
    per: f64,
    spectators: Vec<(String, f64)>,
}

fn formula_counts(key: &str) -> Option<crate::stoich::Formula> {
    let data = crate::species::lookup_key(key)?;
    crate::stoich::parse_formula(data.formula).ok()
}

/// `salt` = `a` × cation + `b` × one anion of `SALT_ANIONS`, electrically
/// neutral, nothing else. Returns (a, anion, b).
fn cation_salt(
    salt: &crate::stoich::Formula,
    cation: &crate::stoich::Formula,
) -> Option<(f64, &'static str, f64)> {
    if salt.charge != 0.0 || cation.charge <= 0.0 || cation.counts.len() != 1 {
        return None;
    }
    let (el, n) = cation.counts.iter().next()?;
    let a = salt.counts.get(el)? / n;
    if a <= 0.0 || (a - a.round()).abs() > 1e-9 {
        return None;
    }
    for &anion in SALT_ANIONS {
        let Ok(af) = crate::stoich::parse_formula(anion) else {
            continue;
        };
        let b = a * cation.charge / -af.charge;
        let mut rest = salt.counts.clone();
        rest.remove(el);
        let fits = af.counts.iter().all(|(e, c)| {
            rest.remove(e)
                .is_some_and(|have| (have - b * c).abs() < 1e-9)
        });
        if fits && rest.is_empty() {
            return Some((a, anion, b));
        }
    }
    None
}

fn forms_uncached(ion: &str) -> Vec<Form> {
    let Ok(ion_f) = crate::stoich::parse_formula(ion) else {
        return Vec::new();
    };
    if ion_f.charge < 0.0 {
        // Anions: the halogen table's Na/K salt derivation, shared.
        return crate::halogen_redox::registry_forms(ion)
            .into_iter()
            .map(|key| Form {
                spectators: crate::halogen_redox::spectators_of(&key, ion)
                    .into_iter()
                    .map(|(c, n)| (c.to_string(), n))
                    .collect(),
                key,
                per: 1.0,
            })
            .collect();
    }
    let mut out = Vec::new();
    if crate::species::REGISTRY.iter().any(|s| s.key == ion) {
        out.push(Form {
            key: ion.to_string(),
            per: 1.0,
            spectators: Vec::new(),
        });
    }
    let mut salts: Vec<Form> = crate::species::REGISTRY
        .iter()
        .filter(|s| s.key != ion)
        .filter_map(|s| {
            let f = formula_counts(s.key)?;
            let (a, anion, b) = cation_salt(&f, &ion_f)?;
            Some(Form {
                key: s.key.to_string(),
                per: a,
                spectators: vec![(anion.to_string(), b / a)],
            })
        })
        .collect();
    salts.sort_by(|x, y| x.key.cmp(&y.key));
    out.extend(salts);
    out
}

/// Every registry form of `ion`, cached per ion (the set of ions is the
/// fixed couple list plus iodide).
fn forms(ion: &str) -> &'static [Form] {
    use std::collections::BTreeMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<BTreeMap<String, &'static [Form]>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut map = cache.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(found) = map.get(ion) {
        return found;
    }
    let leaked: &'static [Form] = Box::leak(forms_uncached(ion).into_boxed_slice());
    map.insert(ion.to_string(), leaked);
    leaked
}

/// Registry keys counted as dissolved `ion` by the equilibrium solver.
pub fn ion_form_keys(ion: &str) -> Vec<String> {
    forms(ion).iter().map(|f| f.key.clone()).collect()
}

/// Dissolved moles of `ion` over all its forms.
fn dissolved_ion(vessel: &Vessel, ion: &str) -> f64 {
    forms(ion)
        .iter()
        .map(|f| aqueous(vessel, &f.key) * f.per)
        .sum()
}

/// Remove `amount` mol of `ion` from its dissolved forms (bare ion first),
/// releasing each reacting salt share's counter-ions.
fn take_ion(vessel: &mut Vessel, ion: &str, amount: f64) {
    let mut left = amount;
    for form in forms(ion) {
        if left <= 0.0 {
            break;
        }
        let have = aqueous(vessel, &form.key) * form.per;
        let take = have.min(left);
        if take <= 0.0 {
            continue;
        }
        vessel.withdraw_phase(
            &SpeciesId::new(&form.key),
            Moles(take / form.per),
            Phase::Aqueous,
        );
        for (counter, n) in &form.spectators {
            vessel.deposit(SpeciesId::new(counter), Moles(take * n), Phase::Aqueous);
        }
        left -= take;
    }
}

/// A plain ion couple from the halogen half table that is equilibrium-
/// limited against iodide and is solved here.
fn plain(h: &crate::halogen_redox::Half) -> bool {
    h.protons == 0.0
        && h.water == 0.0
        && (h.reduced_per_oxidised - 1.0).abs() < 1e-12
        && h.electrons > 0.0
}

/// The couples this module solves against iodide, from the pair table.
pub fn equilibrium_couples() -> Vec<&'static crate::halogen_redox::Half> {
    use crate::halogen_redox::{half_by_id, pairs, PairStatus};
    pairs()
        .iter()
        .filter(|p| p.reductant == "I2/I-" && p.status == PairStatus::EquilibriumLimited)
        .filter_map(|p| half_by_id(p.oxidant))
        .filter(|h| plain(h))
        .collect()
}

/// Whether the (oxidant half, reductant half) pair is solved here.
pub fn solves_pair(oxidant: &str, reductant: &str) -> bool {
    reductant == "I2/I-" && equilibrium_couples().iter().any(|h| h.id == oxidant)
}

/// log10 K for (2/n) Ox + 2 I⁻ ⇌ (2/n) Red + I2(aq): two electrons
/// transferred, so log K = 2 (E°ox − E°(I2(aq)/I⁻)) / (RT ln10 / F).
pub fn couple_log_k(h: &crate::halogen_redox::Half) -> f64 {
    2.0 * (h.e0_v - E0_I2AQ_IODIDE_V) / NERNST_V
}

/// log10 K for 2 Fe³⁺ + 2 I⁻ ⇌ 2 Fe²⁺ + I2(aq), from the two E° values.
pub fn ferric_iodide_log_k() -> f64 {
    let fe = crate::halogen_redox::half_by_id("Fe+3/Fe+2")
        .map(|h| h.e0_v)
        .unwrap_or(0.7696);
    2.0 * (fe - E0_I2AQ_IODIDE_V) / NERNST_V
}

/// Signed extent (mol of I2 formed) that brings this vessel to the couple's
/// equilibrium with iodide: positive oxidises iodide, negative reduces
/// iodine. Zero when the medium is not aqueous, outside the 25 °C
/// constants' domain, or nothing can move.
pub fn couple_extent(vessel: &Vessel, h: &crate::halogen_redox::Half) -> f64 {
    let (lo_k, hi_k) = crate::halogen_redox::DOMAIN_K;
    if liquid_water(vessel) <= 0.0 || !(lo_k..=hi_k).contains(&vessel.temperature.0) {
        return 0.0;
    }
    // Ox and Red per mole of I2 formed.
    let nu = 2.0 / h.electrons;
    let a = dissolved_ion(vessel, h.oxidised);
    let b = dissolved_ion(vessel, IODIDE);
    let c = dissolved_ion(vessel, h.reduced);
    let d = aqueous(vessel, IODINE);
    // Forward needs Ox and I⁻; reverse needs Red and I2.
    let hi = (a / nu).min(b / 2.0);
    let lo = -(c / nu).min(d);
    if hi <= TRACE && -lo <= TRACE {
        return 0.0;
    }
    let litres = vessel.liquid_volume().0.max(1e-9);
    let ln_k = couple_log_k(h) * std::f64::consts::LN_10;
    // g(x) = ln Q(x) - ln K, increasing in x; Q uses FREE I2 and I⁻ from
    // the triiodide split (iodide bound as I3⁻ is not available).
    let g = |x: f64| {
        let parts = split(d + x, b - 2.0 * x, litres);
        let ox = (a - nu * x) / litres;
        let red = (c + nu * x) / litres;
        nu * (red.ln() - ox.ln()) + (parts.iodine / litres).ln()
            - 2.0 * (parts.iodide / litres).ln()
            - ln_k
    };
    let (mut low, mut high) = (lo, hi);
    for _ in 0..200 {
        let mid = 0.5 * (low + high);
        if mid == low || mid == high {
            break;
        }
        let value = g(mid);
        if value.is_nan() {
            break;
        }
        if value > 0.0 {
            high = mid;
        } else {
            low = mid;
        }
    }
    let x = 0.5 * (low + high);
    let scale = a.max(b).max(c).max(d);
    if x.abs() <= TRACE.max(1e-9 * scale) {
        0.0
    } else {
        x
    }
}

/// Fe³⁺/I⁻ extent (see [`couple_extent`]).
pub fn ferric_iodide_extent(vessel: &Vessel) -> f64 {
    crate::halogen_redox::half_by_id("Fe+3/Fe+2")
        .map(|h| couple_extent(vessel, h))
        .unwrap_or(0.0)
}

/// Whether any solved oxidant/iodide equilibrium would move this vessel.
pub fn ferric_iodide_fires(vessel: &Vessel) -> bool {
    equilibrium_couples()
        .iter()
        .any(|h| couple_extent(vessel, h) != 0.0)
}

fn coefficient(n: f64, key: &str) -> String {
    let pretty = crate::halogen_redox::pretty(key);
    if (n - 1.0).abs() < 1e-9 {
        pretty
    } else {
        format!("{} {pretty}", n.round() as i64)
    }
}

/// Move the vessel to every solved oxidant/iodide equilibrium. Element and
/// charge closure is exact: (2/n) Ox^q + 2 I⁻ ↔ (2/n) Red^(q−n) + I2 carries
/// charge 2q/n − 2 on both sides, and a reacting salt share releases its
/// counter-ions unchanged. The caller refreshes `solute_charge`. No
/// reaction heat is charged.
pub fn ferric_iodide_apply(vessel: &mut Vessel, events: &mut Vec<crate::ops::Event>) -> bool {
    let mut moved = false;
    for h in equilibrium_couples() {
        let x = couple_extent(vessel, h);
        if x == 0.0 {
            continue;
        }
        let nu = 2.0 / h.electrons;
        let left = format!(
            "{} + 2 {}",
            coefficient(nu, h.oxidised),
            crate::halogen_redox::pretty(IODIDE)
        );
        let right = format!(
            "{} + {}",
            coefficient(nu, h.reduced),
            crate::halogen_redox::pretty(IODINE)
        );
        let amount = x.abs();
        let equation = if x > 0.0 {
            take_ion(vessel, h.oxidised, nu * amount);
            take_ion(vessel, IODIDE, 2.0 * amount);
            vessel.deposit(
                SpeciesId::new(h.reduced),
                Moles(nu * amount),
                Phase::Aqueous,
            );
            vessel.deposit(SpeciesId::new(IODINE), Moles(amount), Phase::Aqueous);
            format!("{left} → {right}")
        } else {
            take_ion(vessel, h.reduced, nu * amount);
            vessel.withdraw_phase(&SpeciesId::new(IODINE), Moles(amount), Phase::Aqueous);
            vessel.deposit(
                SpeciesId::new(h.oxidised),
                Moles(nu * amount),
                Phase::Aqueous,
            );
            vessel.deposit(SpeciesId::new(IODIDE), Moles(2.0 * amount), Phase::Aqueous);
            format!("{right} → {left}")
        };
        events.push(crate::ops::Event::ReactionOccurred {
            vessel: vessel.id,
            equation,
        });
        moved = true;
    }
    if moved {
        vessel.resolved.invalidate();
    }
    moved
}

// ---------------------------------------------------------------------------
// Iodine inventory (AN-004, 2026-11-11): every iodine pool of a vessel
// ---------------------------------------------------------------------------
//
// `inspect` used to report only `optical_split`, i.e. the free I2 / I3⁻ /
// free I⁻ left OUTSIDE the starch complex, and dropped a vessel whose
// dissolved iodine was entirely starch-bound. The bound share was then an
// unnamed hole in the iodine ledger. `iodine_inventory` is the generic
// replacement: every portion of the vessel that carries iodine atoms (by
// its registry formula) lands in exactly one named pool, and the pools are
// closed against the ledger's iodine atoms. Pools:
//
// * aqueous I2 + dissolved iodide (every registry form of I⁻) split into
//   starch-bound complex, free I2, I3⁻ (K of Palmer et al. 1984, above)
//   and free I⁻;
// * every other iodine-bearing portion (solid I2, AgI, iodate, HI gas, …)
//   reported as its own pool with its iodine-atom count;
// * a portion whose formula cannot be parsed but names iodine is listed as
//   unresolved, never silently counted as zero.
//
// The starch-bound amount is `starch_iodine::complex_moles`, bounded by an
// EDITORIAL site fraction (one I3⁻ equivalent per twenty anhydroglucose
// units). The binding stoichiometry of the amylose–polyiodide complex is
// not settled (Pesek et al., Molecules 27 (2022) 8974,
// doi:10.3390/molecules27248974; Pesek & Silaghi-Dumitrescu, Molecules 29
// (2024) 641, doi:10.3390/molecules29030641), so whenever that pool is
// non-zero the inventory carries the typed limitation
// `not-modeled.starch-iodine-binding-stoichiometry` instead of presenting
// the bound amount as a measured quantity.

/// Editorial starch site fraction behind the starch-bound pool (iodine
/// equivalents per anhydroglucose unit). Not a sourced constant: see the
/// typed limitation above.
pub const STARCH_SITE_FRACTION: f64 = crate::starch_iodine::STARCH_SITE_FRACTION;

/// The typed limitation key carried by a non-zero starch-bound pool.
pub const STARCH_BINDING_LIMITATION: &str = "not-modeled.starch-iodine-binding-stoichiometry";

/// One iodine-bearing portion outside the I2/I⁻/I3⁻/starch split.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct IodinePool {
    pub species: String,
    pub phase: Phase,
    pub moles: f64,
    /// Moles of iodine ATOMS this portion holds.
    pub iodine_atoms_mol: f64,
}

/// Every iodine pool of one vessel, closed against its ledger.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct IodineInventory {
    pub vessel: crate::VesselId,
    /// Free I2(aq) outside starch and triiodide.
    pub free_i2_mol: f64,
    /// I3⁻ (one I2 + one I⁻ each).
    pub triiodide_mol: f64,
    /// Free I⁻ (all registry iodide forms, less I3⁻ and starch share).
    pub free_iodide_mol: f64,
    /// Starch–triiodide complex, as I3⁻ equivalents (one I2 + one I⁻ each).
    pub starch_bound_triiodide_mol: f64,
    /// I2 dissolved in a water-free liquid solvent (an extraction receiver
    /// such as hexane): molecular I2, no triiodide or starch split.
    pub nonaqueous_i2_mol: f64,
    /// The liquid hosting `nonaqueous_i2_mol`, when the vessel has no water.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonaqueous_solvent: Option<String>,
    /// Oxidised iodine as I2 equivalents: free I2 + I3⁻ + starch-bound +
    /// I2 held in a non-aqueous solvent.
    pub oxidised_iodine_mol: f64,
    /// Iodine-bearing portions outside the split above.
    pub other_pools: Vec<IodinePool>,
    /// Species that may hold iodine but whose formula did not parse.
    pub unresolved_species: Vec<String>,
    /// Iodine atoms (mol) in the vessel's ledger.
    pub ledger_iodine_atoms_mol: f64,
    /// Iodine atoms (mol) summed over the pools above.
    pub pooled_iodine_atoms_mol: f64,
    /// ledger − pooled.
    pub closure_residual_mol: f64,
    pub triiodide_k_l_per_mol: f64,
    /// Typed limitations on this inventory (e.g. the editorial starch pool).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<crate::phrase::Phrase>,
}

/// Iodine atoms per formula unit of `key`: `Some(n)` when the formula
/// parses, `None` when it does not.
fn iodine_atoms(key: &str) -> Option<f64> {
    let data = crate::species::lookup_key(key)?;
    let f = crate::stoich::parse_formula(data.formula).ok()?;
    Some(f.counts.get("I").copied().unwrap_or(0.0))
}

/// Whether an unparsed key or formula text may name the element iodine
/// (`I` not followed by a lowercase letter, so `In`/`Ir` do not count).
fn may_name_iodine(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .any(|(i, c)| *c == 'I' && !chars.get(i + 1).is_some_and(|n| n.is_ascii_lowercase()))
}

/// Every iodine pool of `vessel`. A vessel holding no iodine returns an
/// inventory of zeros (callers list it so a control vessel is never
/// silently dropped).
pub fn iodine_inventory(vessel: &Vessel) -> IodineInventory {
    let iodide_keys = crate::halogen_redox::registry_forms(IODIDE);
    let has_water = liquid_water(vessel) > 0.0;
    // SP-011: a water-free liquid (hexane after an extraction) holds I2 as
    // dissolved molecules. That is oxidised iodine and must be counted as
    // such, not filed as an "other" pool beside a false aqueous tag.
    let solvent = crate::vessel::nonaqueous_solvent(&vessel.contents).cloned();
    let mut nonaqueous_i2 = 0.0;
    let mut ledger = 0.0;
    let mut other: Vec<IodinePool> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();
    for p in &vessel.contents {
        let key = p.species.0.as_str();
        let Some(n) = iodine_atoms(key) else {
            let text = crate::species::lookup_key(key)
                .map(|d| d.formula.to_string())
                .unwrap_or_else(|| key.to_string());
            if may_name_iodine(&text) && !unresolved.iter().any(|u| u == key) {
                unresolved.push(key.to_string());
            }
            continue;
        };
        if n <= 0.0 || p.moles.0 <= 0.0 {
            continue;
        }
        ledger += p.moles.0 * n;
        let in_split = has_water
            && p.phase == Phase::Aqueous
            && (key == IODINE || iodide_keys.iter().any(|k| k == key));
        if in_split {
            continue;
        }
        if solvent.is_some() && p.phase == Phase::Aqueous && key == IODINE {
            nonaqueous_i2 += p.moles.0;
            continue;
        }
        match other
            .iter_mut()
            .find(|o| o.species == key && o.phase == p.phase)
        {
            Some(o) => {
                o.moles += p.moles.0;
                o.iodine_atoms_mol += p.moles.0 * n;
            }
            None => other.push(IodinePool {
                species: key.to_string(),
                phase: p.phase,
                moles: p.moles.0,
                iodine_atoms_mol: p.moles.0 * n,
            }),
        }
    }
    let (complex, parts) = if has_water {
        let complex = crate::starch_iodine::complex_moles(vessel);
        let litres = vessel.liquid_volume().0.max(1e-9);
        (complex, optical_parts(vessel, litres, complex))
    } else {
        (
            0.0,
            TriiodideSplit {
                iodine: 0.0,
                triiodide: 0.0,
                iodide: 0.0,
            },
        )
    };
    // The split's own atom count, computed from the split (not copied from
    // the ledger), so the closure below is a real check.
    let split_pooled = 2.0 * parts.iodine + 3.0 * parts.triiodide + parts.iodide + 3.0 * complex;
    let other_atoms: f64 = other.iter().map(|o| o.iodine_atoms_mol).sum();
    let pooled = split_pooled + other_atoms + 2.0 * nonaqueous_i2;
    let mut limitations = Vec::new();
    if complex > TRACE {
        limitations.push(crate::phrase::Phrase::new(
            "not-modeled.starch-iodine-binding-stoichiometry",
            "the amount of iodine held by starch: the binding stoichiometry of the amylose–polyiodide complex is not settled, so the starch-bound pool is an editorial upper-bounded estimate (at most one triiodide per twenty glucose units), not a measured quantity",
            Vec::new(),
        ));
    }
    IodineInventory {
        vessel: vessel.id,
        free_i2_mol: parts.iodine,
        triiodide_mol: parts.triiodide,
        free_iodide_mol: parts.iodide,
        starch_bound_triiodide_mol: complex,
        nonaqueous_i2_mol: nonaqueous_i2,
        nonaqueous_solvent: if nonaqueous_i2 > 0.0 {
            solvent.map(|s| s.0)
        } else {
            None
        },
        oxidised_iodine_mol: parts.iodine + parts.triiodide + complex + nonaqueous_i2,
        other_pools: other,
        unresolved_species: unresolved,
        ledger_iodine_atoms_mol: ledger,
        pooled_iodine_atoms_mol: pooled,
        closure_residual_mol: ledger - pooled,
        triiodide_k_l_per_mol: TRIIODIDE_K,
        limitations,
    }
}

/// Whether the vessel holds any iodine (in its ledger or unresolved).
pub fn holds_iodine(inv: &IodineInventory) -> bool {
    inv.ledger_iodine_atoms_mol > 0.0 || !inv.unresolved_species.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_closes_both_pools_and_meets_k() {
        for (t, b, v) in [(1e-4, 1e-3, 0.1), (3e-4, 3e-3, 0.03), (1e-3, 1e-5, 1.0)] {
            let s = split(t, b, v);
            assert!((s.iodine + s.triiodide - t).abs() <= 1e-18);
            assert!((s.iodide + s.triiodide - b).abs() <= 1e-18);
            let k = (s.triiodide / v) / ((s.iodine / v) * (s.iodide / v));
            assert!((k / TRIIODIDE_K - 1.0).abs() < 1e-8, "{k}");
        }
    }

    #[test]
    fn no_iodide_no_triiodide() {
        let s = split(1e-4, 0.0, 0.1);
        assert_eq!(s.triiodide, 0.0);
        assert_eq!(s.iodine, 1e-4);
    }

    #[test]
    fn i3_band_fit_reproduces_the_second_cited_point() {
        let (centre, peak, sigma) = I3_BAND;
        let t = (460.0 - centre) / sigma;
        let at_460 = peak * (-0.5 * t * t).exp();
        assert!((at_460 - 1022.0).abs() < 5.0, "{at_460}");
    }

    #[test]
    fn ferric_iodide_constant_is_from_the_two_potentials() {
        let log_k = ferric_iodide_log_k();
        assert!((log_k - 2.0 * (0.7696 - 0.621) / 0.05916).abs() < 1e-9);
    }
}

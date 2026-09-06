//! INST-004 / CAP-22: specific conductance from the solved speciation.
//!
//! The meter used to report `ionic_strength × 100 000` with a comment
//! admitting it was rough. This module replaces that with Kohlrausch's law
//! of independent migration: at infinite dilution every ion conducts on
//! its own, so the specific conductance is the sum of each ion's limiting
//! molar conductivity times its concentration,
//!
//! ```text
//! κ = Σᵢ λ°ᵢ · cᵢ
//! ```
//!
//! with λ°ᵢ in S·cm²·mol⁻¹ and cᵢ in mol·L⁻¹ (κ then lands in µS·cm⁻¹
//! after the factor 1000). The λ° table is measured data, not theory.
//!
//! Honesty boundary, stated rather than hidden:
//!
//! Independent migration is the c → 0 limit, and the sum alone is only
//! that limit. Ion–ion drag — the relaxation and electrophoretic effects
//! Debye, Hückel and Onsager named — lowers a real conductivity further
//! and further below the sum as concentration grows, and the bare sum has
//! no term for it. Ten grams of table salt in 100 mL is 1.71 mol/kgw, and
//! the sum read 21.7 S/m against a measured 13 to 14: not a percent-level
//! approximation but a number sixty percent too big, printed as a
//! measurement. So the sum is scaled by [`concentration_factor`]:
//!
//! ```text
//! κ = (Σᵢ λ°ᵢ · cᵢ) · 1 / (1 + a√I + b·I)
//! ```
//!
//! `a` and `b` are fitted, not derived, and [`FIT_SOURCE`] says to what.
//!
//! Honesty boundary, stated rather than hidden:
//!
//! * **The correction is an alkali-halide fit.** It is one function of
//!   ionic strength applied to every ion, so it carries no charge type and
//!   no ion size: it reproduces sodium and potassium halides across the
//!   bench's whole range and is a weaker claim for anything else. Where a
//!   2:2 salt is mostly ion-paired the speciation has already removed the
//!   pairs — a neutral complex carries no current — so what is left for
//!   this factor is the atmosphere effect on the free ions, which is what
//!   it is fitted to. Above [`FITTED_LIMIT_MOLAL`] it is an extrapolation
//!   and the result says so (`within_fitted_range = false`).
//! * **Dilute is still where it is trustworthy.** Past
//!   I ≈ 0.1 mol/kgw the estimate is out of its calibrated range and the
//!   result says so (`within_dilute_limit = false`), correction or no
//!   correction: a fitted factor makes the number closer, not measured.
//! * **Coverage is accounted, not assumed.** Charged species with no
//!   tabulated λ° are left out of the sum and reported by name, with the
//!   fraction of the total charge the sum did cover. Neutral aqueous
//!   complexes (AgCl°, CO₂(aq)) carry no current and are rightly absent.
//! * **No speciation, no Kohlrausch.** When a solver reported only pH and
//!   ionic strength, the old scaling survives as an explicitly labelled
//!   mean-mobility estimate (I × 10⁵ µS/cm ≈ a 1:1 electrolyte of
//!   ordinary mobility) instead of masquerading as the real model.
//!
//! # The other kind of conductor
//!
//! A metal conducts for a different reason than a solution does. There are
//! no ions moving through a copper wire; there are electrons that belong to
//! no particular atom, and what limits them is scattering off the lattice.
//! Kohlrausch says nothing about that, and no amount of λ° data would make
//! it. So [`dry_solid_conductance`] is a separate path over a separate
//! datum — the registry's curated `electrical_resistivity` — and it fires
//! only where the solution model does not: a dry vessel, one sample, no
//! aqueous phase at all. A beaker with a copper wire standing in salt water
//! is a solution measurement and stays one, because that is what the probe
//! would read.
//!
//! And a third kind, which is neither. A porcelain insulator conducts by
//! almost nothing at all — twenty orders of magnitude below the copper —
//! and the reason is that it has no free carriers rather than that its
//! carriers are slow. The number for it cannot ride a species record,
//! because porcelain is not a species: the recipe resolves 68% of it into
//! silica, and the silica's record would be quartz sand's. So a named
//! object carries its own reviewed resistivity as a material role, and
//! [`dry_solid_conductance`] reads THAT for a vessel holding one object.
//! Insulators and semiconductors also differ from metals in a way the
//! datum has to admit: their resistivity is a span, not a constant, so
//! every such row states the span its class covers beside the point value
//! the meter reads.

use crate::vessel::{SolutionInfo, SpeciesDetail, Vessel};

/// Where every λ° in [`LIMITING_CONDUCTIVITY`] comes from.
pub const LAMBDA_SOURCE: &str = "λ°: CRC Handbook of Chemistry and Physics, \
    'Ionic Conductivity and Diffusion at Infinite Dilution' (Vanýsek), \
    25 °C, water";

/// Limiting molar ionic conductivities λ° at 25 °C in S·cm²·mol⁻¹, keyed
/// by PHREEQC species name (charge written as trailing `+2` / `-2`).
/// Values for multivalent ions are for the ion itself, not the
/// equivalent — Ca⁺² is ~119, i.e. 2 × λ°(½Ca⁺²).
pub const LIMITING_CONDUCTIVITY: &[(&str, f64)] = &[
    ("H+", 349.65),
    ("OH-", 198.0),
    ("Li+", 38.66),
    ("Na+", 50.08),
    ("K+", 73.48),
    ("NH4+", 73.5),
    ("Ag+", 61.9),
    ("Ca+2", 118.94),
    ("Mg+2", 106.0),
    ("Ba+2", 127.2),
    ("Sr+2", 118.9),
    ("Cu+2", 107.2),
    ("Zn+2", 105.6),
    ("Fe+2", 108.0),
    ("Fe+3", 204.0),
    ("Al+3", 183.0),
    ("Mn+2", 107.0),
    ("Pb+2", 142.0),
    ("Cl-", 76.31),
    ("Br-", 78.1),
    ("I-", 76.8),
    ("F-", 55.4),
    ("NO3-", 71.42),
    ("ClO4-", 67.3),
    ("HCO3-", 44.5),
    ("CO3-2", 138.6),
    ("SO4-2", 160.0),
    ("MnO4-", 61.3),
];

/// Ionic strength above which independent migration is no longer an
/// honest model (mol/kgw). Chosen where the neglected ion–ion drag grows
/// from percent-level to tens of percent.
pub const DILUTE_LIMIT_MOLAL: f64 = 0.1;

/// Where the concentration correction's two coefficients come from.
pub const FIT_SOURCE: &str = "Concentration correction: a two-parameter \
    empirical attenuation of the Kohlrausch sum, κ = Σ λ°ᵢcᵢ / (1 + a√I + bI), \
    fitted by hand to the measured specific conductance of aqueous sodium \
    chloride and potassium chloride at 25 °C as tabulated in the CRC Handbook \
    of Chemistry and Physics, 'Concentrative Properties of Aqueous Solutions: \
    Conversion Tables' and 'Electrical Conductivity of Aqueous Solutions' — \
    1413 µS/cm for the 0.01 mol/kg KCl calibration standard (the IUPAC/OIML \
    reference value), and roughly 8.5 S/m at 1 mol/L and 15.5 S/m at 2 mol/L \
    for NaCl. The FORM is Kohlrausch's √c law with a linear term added \
    because the √c law alone is valid only to about 0.1 mol/L; it is an \
    empirical fit in the spirit of the Casteel–Amis equation, not that \
    equation, and NO edition of any handbook was opened for a per-salt \
    parameter set. Reproduces the four fitted points to better than 2% and \
    is an extrapolation above 2 mol/kgw";

/// Coefficient of the √I term. Kohlrausch's law of the square root is the
/// leading behaviour of ion–ion drag and this is its size; it is fitted
/// rather than the Onsager coefficient, because the Onsager coefficient is
/// the c → 0 slope and this function has to hold at 2 mol/kgw as well.
const FIT_SQRT: f64 = 0.5324;

/// Coefficient of the linear term. Negative: past the first half-molal the
/// square-root law over-corrects, which is exactly why Kohlrausch's law
/// alone is quoted only to about 0.1 mol/L. The denominator is monotone in
/// I up to I ≈ 19 mol/kgw, far above saturated brine, so the function never
/// turns around inside anything a bench can hold.
const FIT_LINEAR: f64 = -0.0608;

/// Ionic strength above which the concentration correction is an
/// extrapolation past what it was fitted to (mol/kgw). Ten grams of table
/// salt in 100 mL of water — about the most concentrated thing a school
/// bench makes on purpose — is 1.71, and sits inside it.
pub const FITTED_LIMIT_MOLAL: f64 = 2.0;

/// How much of the infinite-dilution sum survives the ion atmosphere at
/// this ionic strength: 1.0 at infinite dilution, 0.95 at the 0.01 mol/kg
/// calibration standard, 0.63 at 1.7 mol/kgw.
///
/// See [`FIT_SOURCE`]. Clamped at 1.0 from above so a negative ionic
/// strength — which cannot happen, but the type permits — can never make
/// a solution conduct better than its own ions could.
pub fn concentration_factor(ionic_strength: f64) -> f64 {
    let i = ionic_strength.max(0.0);
    (1.0 / (1.0 + FIT_SQRT * i.sqrt() + FIT_LINEAR * i)).clamp(0.0, 1.0)
}

/// What backed the number — the two paths must stay distinguishable all
/// the way to the reader.
#[derive(Debug, Clone, PartialEq)]
pub enum Basis {
    /// Kohlrausch sum over the solved speciation.
    Kohlrausch {
        /// Fraction of the total ionic charge concentration the λ° table
        /// covered (1.0 = every charged species contributed).
        covered_charge_fraction: f64,
        /// Charged species that were left out for lack of a tabulated λ°.
        omitted: Vec<String>,
    },
    /// `I × 10⁵` — the labelled fallback when no speciation was reported.
    MeanMobility,
}

/// A specific-conductance estimate that knows what it is.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub microsiemens_per_cm: f64,
    pub basis: Basis,
    /// False above [`DILUTE_LIMIT_MOLAL`]: the number is then an
    /// extrapolation past the model's validity, not a measurement.
    pub within_dilute_limit: bool,
    /// What [`concentration_factor`] took off the infinite-dilution sum.
    /// 1.0 on the mean-mobility path, which is a different approximation
    /// with its own error and would only be double-counted by this one.
    pub concentration_factor: f64,
    /// False above [`FITTED_LIMIT_MOLAL`]: the correction itself is then
    /// outside the data it was fitted to.
    pub within_fitted_range: bool,
}

impl Estimate {
    /// Whether the meter may present this as an in-calibration reading:
    /// dilute, and (on the Kohlrausch path) with at least 90% of the
    /// ionic charge actually covered by the table.
    pub fn trustworthy(&self) -> bool {
        self.within_dilute_limit
            && match &self.basis {
                Basis::Kohlrausch {
                    covered_charge_fraction,
                    ..
                } => *covered_charge_fraction >= 0.9,
                Basis::MeanMobility => false,
            }
    }
}

/// The ionic charge encoded in a PHREEQC species name: `Na+` → +1,
/// `Ca+2` → +2, `CO3-2` → −2, `AgCl` → 0. PHREEQC writes the charge as a
/// trailing run of `+`/`-` signs, optionally followed by the count.
pub fn ion_charge(name: &str) -> i32 {
    let core = name.trim();
    let Some(pos) = core.find(['+', '-']) else {
        return 0;
    };
    // Everything from the first sign on is the charge suffix, e.g. "+2",
    // "-", "++" (older dialects double the sign instead of counting).
    let suffix = &core[pos..];
    let sign = if suffix.starts_with('+') { 1 } else { -1 };
    let signs = suffix
        .chars()
        .take_while(|c| *c == '+' || *c == '-')
        .count();
    let digits: String = suffix
        .chars()
        .skip(signs)
        .take_while(char::is_ascii_digit)
        .collect();
    match digits.parse::<i32>() {
        Ok(n) => sign * n,
        Err(_) => sign * signs as i32,
    }
}

/// Specific conductance of a solved solution, honestly labelled.
pub fn specific_conductance(info: &SolutionInfo) -> Estimate {
    let within_dilute_limit = info.ionic_strength <= DILUTE_LIMIT_MOLAL;
    let within_fitted_range = info.ionic_strength <= FITTED_LIMIT_MOLAL;
    if info.species.is_empty() {
        return Estimate {
            microsiemens_per_cm: info.ionic_strength * 100_000.0,
            basis: Basis::MeanMobility,
            within_dilute_limit,
            concentration_factor: 1.0,
            within_fitted_range,
        };
    }
    let attenuation = concentration_factor(info.ionic_strength);
    let lambda = |name: &str| -> Option<f64> {
        LIMITING_CONDUCTIVITY
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, l)| *l)
    };
    let mut kappa_us_cm = 0.0;
    let mut charge_covered = 0.0;
    let mut charge_total = 0.0;
    let mut omitted = Vec::new();
    for SpeciesDetail { name, molality, .. } in &info.species {
        let z = ion_charge(name);
        if z == 0 {
            continue; // neutral complexes carry no current
        }
        // Dilute aqueous: mol/kgw ≈ mol/L, the same approximation the
        // dilute-limit boundary already commits this model to.
        let c_mol_l = *molality;
        charge_total += f64::from(z.abs()) * c_mol_l;
        match lambda(name) {
            Some(l) => {
                kappa_us_cm += l * c_mol_l * 1000.0;
                charge_covered += f64::from(z.abs()) * c_mol_l;
            }
            None => omitted.push(name.clone()),
        }
    }
    let covered_charge_fraction = if charge_total > 0.0 {
        charge_covered / charge_total
    } else {
        1.0 // nothing charged in solution: κ = 0 covers everything there is
    };
    Estimate {
        microsiemens_per_cm: kappa_us_cm * attenuation,
        basis: Basis::Kohlrausch {
            covered_charge_fraction,
            omitted,
        },
        within_dilute_limit,
        concentration_factor: attenuation,
        within_fitted_range,
    }
}

/// What the meter reads from a dry solid: the curated resistivity, its
/// reciprocal, and the citation that has to travel with both.
///
/// Not a [`Reading`](crate::instrument::Reading), because the useful answer
/// carries a boundary and a source, and a bare scalar has nowhere to put
/// them — the same argument the melting-point apparatus makes for
/// `TransitionReading`.
#[derive(Debug, Clone, PartialEq)]
pub struct SolidConductance {
    /// The species key, or the material recipe's canonical key, that the
    /// reading is about.
    pub species: String,
    /// Ω·m at 293.15 K, as curated.
    pub resistivity_ohm_m: f64,
    /// The span the reviewed class of material covers, when the single
    /// value above is one point inside a range the object does not pin
    /// down. `None` for a pure solid, whose resistivity IS the claim.
    ///
    /// It exists because the insulator and semiconductor rows are not
    /// like the metal rows. Copper's resistivity is a constant of copper;
    /// porcelain's moves by orders of magnitude with the alkali content
    /// of its glassy phase and with temperature, and doped silicon's is
    /// set by a dopant concentration no recipe here states. Quoting the
    /// point without the span would be a confidence the data has not got.
    pub span_ohm_m: Option<(f64, f64)>,
    /// S/m — the reciprocal, which is what a conductance meter reads.
    pub conductivity_s_per_m: f64,
    /// The tranche citation behind the number.
    pub source: String,
    /// What the row does not claim: purity, temper, alloy, anisotropy,
    /// temperature dependence, surface leakage, dopant level.
    pub boundary: Option<String>,
}

impl SolidConductance {
    /// The span as conductances, high first, because a conductance meter
    /// reads S/m and the reciprocal reverses the order.
    pub fn span_s_per_m(&self) -> Option<(f64, f64)> {
        self.span_ohm_m
            .map(|(lower, upper)| (1.0 / upper, 1.0 / lower))
    }
}

/// The dry-solid path, or `None` if this vessel is not one.
///
/// Four conditions, and each of them is a refusal the meter should make
/// rather than a case to approximate:
///
/// * **No characterised solution.** If a solver has spoken about this
///   vessel, the probe is in a solution and the Kohlrausch path owns it.
/// * **No liquid at all.** A wet solid conducts through the film of liquid
///   on it, which is neither model.
/// * **One sample.** Two metals touching are a circuit with a geometry,
///   and this reading has no geometry — it is a material property, not a
///   resistance in ohms. So: exactly one solid species and nothing
///   unresolved, or exactly one named object and nothing that did not
///   arrive as part of it.
/// * **A curated resistivity.** Most solids in this registry have none, and
///   the meter says so instead of inventing one.
///
/// # Why there are two paths
///
/// A pure solid's resistivity rides its species record, because that is
/// where a handbook puts a constant of a substance. An insulator on this
/// shelf is not a substance: `porcelain` is a fired object, 68% resolved
/// silica and a conserved remainder, and its resistivity belongs to
/// neither half — it belongs to the object. Reading the silica's record
/// would be reading quartz sand, which is a different material with the
/// same species key, and reading nothing was the refusal this function
/// used to make. So a named object's resistivity is a reviewed property
/// of the recipe, and this is where the meter picks it up.
pub fn dry_solid_conductance(vessel: &Vessel) -> Option<SolidConductance> {
    if vessel.solution.is_some() {
        return None;
    }
    if vessel.liquid_volume().0 > 0.0 {
        return None;
    }
    // Any non-solid left over (a gas headspace aside) means the sample is
    // not the isolated lump this reading describes. Checked before either
    // path, because it disqualifies both.
    if vessel.contents.iter().any(|other| {
        other.phase != crate::species::Phase::Solid
            && other.phase != crate::species::Phase::Gas
            && other.moles.0 > crate::OBSERVABLE_MOLES
    }) {
        return None;
    }
    named_object_conductance(vessel).or_else(|| pure_solid_conductance(vessel))
}

/// The lot source the material route stamps on everything a named recipe
/// deposits. `corrosion::Barrier` keys on the whole string; here the tail
/// is the recipe id and the reading needs it.
const MATERIAL_LOT_PREFIX: &str = "material recipe ";

/// Record `candidate` as the one object seen so far, or report that it
/// disagrees with the object already seen.
fn agrees(seen: &mut Option<String>, candidate: &str) -> bool {
    match seen {
        Some(already) => already == candidate,
        None => {
            *seen = Some(candidate.to_string());
            true
        }
    }
}

/// One solid species, one curated species resistivity.
///
/// Refuses outright while any named material is in the vessel: an object
/// that resolves nothing at all (silicon) leaves the beaker holding only
/// whatever else was dropped in beside it, and reporting THAT solid's
/// resistivity would answer about the wrong thing.
fn pure_solid_conductance(vessel: &Vessel) -> Option<SolidConductance> {
    if vessel
        .unresolved_materials
        .iter()
        .any(|portion| portion.amount > 0.0)
    {
        return None;
    }
    let mut solids = vessel.contents.iter().filter(|portion| {
        portion.phase == crate::species::Phase::Solid && portion.moles.0 > crate::OBSERVABLE_MOLES
    });
    let portion = solids.next()?;
    if solids.next().is_some() {
        return None;
    }
    let resistivity = crate::species::lookup(&portion.species)?.electrical_resistivity?;
    Some(SolidConductance {
        species: portion.species.0.clone(),
        resistivity_ohm_m: resistivity.ohm_m,
        span_ohm_m: None,
        conductivity_s_per_m: resistivity.conductivity_s_per_m(),
        source: resistivity.source.to_string(),
        boundary: resistivity.boundary.map(String::from),
    })
}

/// One named object, and its own reviewed resistivity.
///
/// The strictness is the pure path's, restated for a material: one recipe,
/// and everything solid in the vessel arrived as part of it. A porcelain
/// dish resolves 68% of itself into `SiO2`, so the vessel holds silica
/// beside the unresolved remainder and "no other solids" would refuse the
/// object its own inventory. The rule that means what the pure path's
/// means is therefore about PROVENANCE: the silica may stay because the
/// porcelain brought it, and a copper wire dropped in beside the dish
/// makes this a circuit with a geometry again, so the meter stands down.
///
/// Both halves of an object are consulted, because recipes differ in
/// which half they have. `porcelain` and `glass` keep an unresolved
/// remainder; `quartz` and `silica_glass` resolve into silica entirely and
/// have no unresolved portion at all; `silicon` resolves into nothing and
/// has no species portion at all. Reading only one of the two would leave
/// a third of the shelf unreadable for a reason that is about bookkeeping
/// rather than about electricity.
fn named_object_conductance(vessel: &Vessel) -> Option<SolidConductance> {
    use crate::material::MaterialRole;
    let mut id: Option<String> = None;
    for portion in vessel
        .unresolved_materials
        .iter()
        .filter(|portion| portion.amount > 0.0)
    {
        if !agrees(&mut id, &portion.recipe_id) {
            return None;
        }
    }
    for portion in vessel.contents.iter().filter(|portion| {
        portion.phase == crate::species::Phase::Solid && portion.moles.0 > crate::OBSERVABLE_MOLES
    }) {
        let from = vessel.lots.iter().find_map(|lot| {
            if lot.species.0 != portion.species.0 || lot.phase != crate::species::Phase::Solid {
                return None;
            }
            lot.source.as_deref()?.strip_prefix(MATERIAL_LOT_PREFIX)
        })?;
        if !agrees(&mut id, from) {
            return None;
        }
    }
    let id = id?;
    let recipe = crate::material::all()
        .into_iter()
        .find(|recipe| recipe.id == id)?;
    let (ohm_m, span_lower, span_upper, boundary, source) =
        recipe.roles.iter().find_map(|role| match role {
            MaterialRole::BulkElectricalResistivity {
                ohm_m,
                span_lower_ohm_m,
                span_upper_ohm_m,
                boundary,
                source,
            } => Some((
                *ohm_m,
                *span_lower_ohm_m,
                *span_upper_ohm_m,
                boundary.clone(),
                source.clone(),
            )),
            _ => None,
        })?;
    if !ohm_m.is_finite() || ohm_m <= 0.0 {
        return None;
    }
    Some(SolidConductance {
        species: recipe.canonical_key.clone(),
        resistivity_ohm_m: ohm_m,
        span_ohm_m: Some((span_lower, span_upper)),
        conductivity_s_per_m: 1.0 / ohm_m,
        source,
        boundary: Some(boundary),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ion(name: &str, molality: f64) -> SpeciesDetail {
        SpeciesDetail {
            name: name.to_string(),
            molality,
            activity: molality,
        }
    }

    fn solved(ionic_strength: f64, species: Vec<SpeciesDetail>) -> SolutionInfo {
        SolutionInfo {
            pe: None,
            redox: Vec::new(),
            ph: 7.0,
            ionic_strength,
            species,
            provenance: None,
        }
    }

    #[test]
    fn phreeqc_charge_suffixes_parse() {
        assert_eq!(ion_charge("Na+"), 1);
        assert_eq!(ion_charge("Cl-"), -1);
        assert_eq!(ion_charge("Ca+2"), 2);
        assert_eq!(ion_charge("CO3-2"), -2);
        assert_eq!(ion_charge("Fe+3"), 3);
        assert_eq!(ion_charge("AgCl"), 0);
        assert_eq!(ion_charge("CO2"), 0);
        // Older dialects double the sign instead of counting it.
        assert_eq!(ion_charge("Ca++"), 2);
        assert_eq!(ion_charge("S--"), -2);
    }

    /// The 0.01 mol/kg KCl calibration standard reads 1413 µS/cm. The
    /// model must land close — and must land HIGH, because the
    /// concentration correction is fitted to hold across two decades and
    /// is deliberately gentler than the truth at the dilute end. A result
    /// below the standard would mean the correction had started
    /// over-correcting where the drag is still percent-level, which is
    /// the failure mode a fitted factor has and the bare sum did not.
    #[test]
    fn kcl_calibration_standard_within_model_error() {
        let info = solved(0.01, vec![ion("K+", 0.01), ion("Cl-", 0.01)]);
        let est = specific_conductance(&info);
        let kappa = est.microsiemens_per_cm;
        assert!(kappa > 1413.0, "must overestimate the standard: {kappa}");
        assert!(
            (kappa - 1413.0) / 1413.0 < 0.07,
            "within 7% of the 1413 µS/cm standard: {kappa}"
        );
        assert!(est.trustworthy());
        match est.basis {
            Basis::Kohlrausch {
                covered_charge_fraction,
                ref omitted,
            } => {
                assert_eq!(covered_charge_fraction, 1.0);
                assert!(omitted.is_empty());
            }
            Basis::MeanMobility => panic!("speciation was present"),
        }
    }

    /// Grotthuss hopping makes protons the fastest ion in water: at equal
    /// concentration HCl must out-conduct KCl, and KCl out-conduct NaCl
    /// (K⁺ outruns the more strongly hydrated Na⁺). The old
    /// ionic-strength scaling could not see any of this.
    #[test]
    fn mobility_ordering_is_visible() {
        let at = |cation: &str| {
            specific_conductance(&solved(0.01, vec![ion(cation, 0.01), ion("Cl-", 0.01)]))
                .microsiemens_per_cm
        };
        let (hcl, kcl, nacl) = (at("H+"), at("K+"), at("Na+"));
        assert!(
            hcl > 2.0 * kcl,
            "protons conduct several-fold: {hcl} vs {kcl}"
        );
        assert!(kcl > nacl, "K+ outruns Na+: {kcl} vs {nacl}");
    }

    #[test]
    fn neutral_complexes_carry_no_current() {
        let bare = solved(0.001, vec![ion("Ag+", 0.001), ion("NO3-", 0.001)]);
        let with_complex = solved(
            0.001,
            vec![
                ion("Ag+", 0.001),
                ion("NO3-", 0.001),
                ion("AgCl", 0.005), // neutral ion pair, however abundant
            ],
        );
        let a = specific_conductance(&bare);
        let b = specific_conductance(&with_complex);
        assert_eq!(a.microsiemens_per_cm, b.microsiemens_per_cm);
        assert!(b.trustworthy(), "a neutral complex is not 'uncovered'");
    }

    #[test]
    fn untabulated_ions_are_confessed_not_ignored() {
        let info = solved(
            0.02,
            vec![
                ion("Na+", 0.01),
                ion("Cl-", 0.01),
                ion("W12O41-10", 0.002), // exotic: nothing in the table
            ],
        );
        let est = specific_conductance(&info);
        match est.basis {
            Basis::Kohlrausch {
                covered_charge_fraction,
                ref omitted,
            } => {
                assert!(covered_charge_fraction < 0.9);
                assert_eq!(omitted, &vec!["W12O41-10".to_string()]);
            }
            Basis::MeanMobility => panic!("speciation was present"),
        }
        assert!(!est.trustworthy(), "a fifth of the charge is unaccounted");
    }

    /// Ten grams of table salt in 100 mL of water: 1.71 mol/kgw, and a
    /// measured 13 to 14 S/m. The bare Kohlrausch sum reads 21.7 — the
    /// defect this correction exists for — and the corrected model has to
    /// land in the measured window, not merely nearer it.
    #[test]
    fn kitchen_brine_matches_the_measured_conductivity() {
        let info = solved(1.711, vec![ion("Na+", 1.711), ion("Cl-", 1.711)]);
        let est = specific_conductance(&info);
        let s_per_m = est.microsiemens_per_cm / 10_000.0;
        assert!(
            (13.0..14.0).contains(&s_per_m),
            "CRC gives 1 mol/L NaCl 8.5 S/m and 2 mol/L about 15.5, so 1.71 \
             mol/kgw is 13 to 14: got {s_per_m:.2} S/m"
        );
        let uncorrected = est.microsiemens_per_cm / est.concentration_factor / 10_000.0;
        assert!(
            uncorrected > 21.0,
            "and the uncorrected sum is what it used to print: {uncorrected:.1} S/m"
        );
        assert!(est.within_fitted_range, "1.71 is inside the fit");
        assert!(!est.within_dilute_limit, "and far outside the dilute limit");
    }

    /// The correction is a curve, not a switch: it must be gentle where
    /// the drag is gentle, and it must never make a solution conduct
    /// better than its own ions could at infinite dilution.
    #[test]
    fn the_concentration_correction_is_monotone_over_the_bench_range() {
        assert_eq!(concentration_factor(0.0), 1.0);
        let mut previous = 1.0;
        let mut i = 0.0;
        while i <= 6.0 {
            let f = concentration_factor(i);
            assert!(f <= 1.0, "never above the infinite-dilution sum at I={i}");
            assert!(f <= previous + 1e-12, "monotone falling at I={i}: {f}");
            previous = f;
            i += 0.05;
        }
        // Two decades of CRC data, reproduced to better than 2%.
        for (ionic_strength, measured) in [(0.01, 0.9433), (1.0, 0.6725), (2.0, 0.6131)] {
            let modelled = concentration_factor(ionic_strength);
            assert!(
                (modelled / measured - 1.0).abs() < 0.02,
                "I={ionic_strength}: modelled {modelled:.4} against {measured:.4}"
            );
        }
    }

    /// Saturated brine is past what the two coefficients were fitted to,
    /// and the estimate says so rather than quietly extrapolating.
    #[test]
    fn past_the_fitted_range_the_estimate_admits_it() {
        let info = solved(5.0, vec![ion("Na+", 5.0), ion("Cl-", 5.0)]);
        let est = specific_conductance(&info);
        assert!(!est.within_fitted_range);
        assert!(!est.trustworthy());
    }

    #[test]
    fn concentrated_solutions_leave_the_validity_window() {
        let info = solved(1.0, vec![ion("Na+", 1.0), ion("Cl-", 1.0)]);
        let est = specific_conductance(&info);
        assert!(!est.within_dilute_limit);
        assert!(!est.trustworthy());
    }

    #[test]
    fn no_speciation_falls_back_to_labelled_mean_mobility() {
        let info = solved(0.01, Vec::new());
        let est = specific_conductance(&info);
        assert_eq!(est.basis, Basis::MeanMobility);
        assert_eq!(est.microsiemens_per_cm, 1000.0);
        assert!(!est.trustworthy(), "a guess never presents as calibrated");
    }
}

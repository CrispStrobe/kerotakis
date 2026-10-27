//! Certified strong-acid / strong-base reserve carriers.
//!
//! One typed table of the registry substances whose acid/base capacity is
//! stoichiometric (fully dissociated over the whole aqueous pH range the
//! curated and named rows use), shared by every consumer that used to name a
//! literal reagent (`NaOH` in named saponification, `HCl` in the bleach rows).
//!
//! The reserve is an ANALYTICAL, charge-backed quantity: spectator-cation
//! equivalents minus strong-acid-anion equivalents (base side), or literal
//! strong-acid protons minus strong base (acid side). It is never the
//! instantaneous free hydroxide/proton of a measurement: an acetate buffer at
//! pH 4.66 holds 5.17e-4 mol of basis but only ~4.6e-10 mol of free OH-, and
//! neither number is a strong-base reserve. Weak acid/base buffers (carbonate,
//! ammonia, phosphate, borate, sulfite, hypochlorite, organic acids, sulfide)
//! are therefore NOT carriers: their capacity is pH-dependent and their
//! presence blocks certification instead of being counted.
//!
//! Deliberately not carriers, with reasons recorded in the coverage table
//! (`docs/reserve-coverage-20261027.md`):
//! * `H2SO4` on the ACID side: the second proton (HSO4-, pKa ~1.99) is not a
//!   strong acid and the registry has no hydrogen-sulfate ion to hold the
//!   half-spent state. It still counts as two strong-acid ANION equivalents on
//!   the base side, where every proton is neutralised in the alkaline excess.
//! * `Ca(OH)2`, `Mg(OH)2` and their cations: solubility-limited, their
//!   hydroxide precipitates within the alkaline range, so their capacity is
//!   not stoichiometric.
//! * `HCl` on the acid side of the generic reserve: literal hydrochloric acid
//!   has its own curated rows; counting it again would let two rows race over
//!   one bottle. It remains a base-side anion carrier.
use crate::{species::Phase, Moles, SpeciesId, Vessel};

/// Which phases of a carrier's portion are admitted into a reserve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admit {
    /// Liquid or aqueous (a poured acid before or after a solve).
    Fluid,
    /// Any condensed phase (a salt or base pellet, liquid or dissolved).
    NotGas,
    /// A dissolved ion only.
    AqueousOnly,
}

impl Admit {
    pub fn admits(self, phase: Phase) -> bool {
        match self {
            Admit::Fluid => matches!(phase, Phase::Liquid | Phase::Aqueous),
            Admit::NotGas => !matches!(phase, Phase::Gas),
            Admit::AqueousOnly => matches!(phase, Phase::Aqueous),
        }
    }
}

/// One certified strong carrier. Equivalents are per mole of the key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Carrier {
    pub key: &'static str,
    /// Strong-base spectator cation equivalents (Na+, K+, Ba2+).
    pub cation_eq: f64,
    /// Strong-acid anion equivalents (Cl-, Br-, I-, NO3-, SO4-2).
    pub anion_eq: f64,
    /// Owned hydroxide coordinates this carrier releases on neutralisation.
    pub hydroxide_eq: f64,
    /// Literal strong-acid protons available to the ACID-side reserve.
    pub proton_eq: f64,
    /// Conjugate anion deposited when the acid-side reserve spends a proton.
    pub conjugate: Option<&'static str>,
    /// Cation released per mole when a base-side hydroxide carrier is spent.
    pub cation: Option<(&'static str, f64)>,
    pub admit: Admit,
    /// Whether owning this carrier makes the inventory an ionic (aqueous) one.
    pub ionic: bool,
}

const fn c(
    key: &'static str,
    cation_eq: f64,
    anion_eq: f64,
    hydroxide_eq: f64,
    admit: Admit,
    ionic: bool,
) -> Carrier {
    Carrier {
        key,
        cation_eq,
        anion_eq,
        hydroxide_eq,
        proton_eq: 0.0,
        conjugate: None,
        cation: None,
        admit,
        ionic,
    }
}

const fn base(key: &'static str, hydroxide_eq: f64, cation: &'static str, ionic: bool) -> Carrier {
    Carrier {
        key,
        cation_eq: hydroxide_eq,
        anion_eq: 0.0,
        hydroxide_eq,
        proton_eq: 0.0,
        conjugate: None,
        cation: Some((cation, 1.0)),
        admit: Admit::NotGas,
        ionic,
    }
}

const fn acid(key: &'static str, conjugate: &'static str, generic_acid_side: bool) -> Carrier {
    Carrier {
        key,
        cation_eq: 0.0,
        anion_eq: 1.0,
        hydroxide_eq: 0.0,
        proton_eq: if generic_acid_side { 1.0 } else { 0.0 },
        conjugate: Some(conjugate),
        cation: None,
        admit: Admit::Fluid,
        ionic: false,
    }
}

/// Every certified strong carrier. Order is the spending order.
pub const CARRIERS: &[Carrier] = &[
    // Strong bases. `NaOH` keeps the existing molecular template (non-ionic).
    base("NaOH", 1.0, "Na+", false),
    base("KOH", 1.0, "K+", true),
    // Ba(OH)2: one mole releases one Ba2+ and two hydroxide equivalents.
    base("Ba(OH)2", 2.0, "Ba+2", true),
    c("OH-", 0.0, 0.0, 1.0, Admit::AqueousOnly, true),
    c(
        crate::species::BASE_EQUIVALENTS,
        0.0,
        0.0,
        1.0,
        Admit::AqueousOnly,
        true,
    ),
    // Strong acids. HCl is a base-side anion only (see module docs).
    acid("HCl", "Cl-", false),
    acid("HBr", "Br-", true),
    acid("HI", "I-", true),
    acid("HNO3", "NO3-", true),
    // H2SO4: two anion equivalents on the base side, no acid-side protons.
    c("H2SO4", 0.0, 2.0, 0.0, Admit::Fluid, false),
    // Spectator ions.
    c("Na+", 1.0, 0.0, 0.0, Admit::AqueousOnly, true),
    c("K+", 1.0, 0.0, 0.0, Admit::AqueousOnly, true),
    c("Ba+2", 2.0, 0.0, 0.0, Admit::AqueousOnly, true),
    c("Cl-", 0.0, 1.0, 0.0, Admit::AqueousOnly, true),
    c("Br-", 0.0, 1.0, 0.0, Admit::AqueousOnly, true),
    c("I-", 0.0, 1.0, 0.0, Admit::AqueousOnly, true),
    c("NO3-", 0.0, 1.0, 0.0, Admit::AqueousOnly, true),
    c("SO4-2", 0.0, 2.0, 0.0, Admit::AqueousOnly, true),
    // Neutral salts of a strong acid and a strong base.
    c("NaCl", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("KCl", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("NaBr", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("KBr", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("KI", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("NaNO3", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("KNO3", 1.0, 1.0, 0.0, Admit::NotGas, false),
    c("Na2SO4", 2.0, 2.0, 0.0, Admit::NotGas, false),
    c("BaCl2", 2.0, 2.0, 0.0, Admit::NotGas, false),
    // Sodium hydrogen sulfate: fully neutralised in an alkaline excess.
    c("NaHSO4", 1.0, 2.0, 0.0, Admit::NotGas, false),
];

pub fn carrier(key: &str) -> Option<&'static Carrier> {
    CARRIERS.iter().find(|c| c.key == key)
}

/// Weak acid/base reservoirs whose capacity is pH-dependent. Presence blocks
/// certification; nothing here is ever counted as strong reserve.
pub const WEAK_BUFFERS: &[(&str, &str)] = &[
    ("CO2", "carbonate"),
    ("CO2(aq)", "carbonate"),
    ("CO3-2", "carbonate"),
    ("HCO3-", "carbonate"),
    ("Na2CO3", "carbonate"),
    ("NaHCO3", "carbonate"),
    ("NH3", "ammonia"),
    ("NH4+", "ammonia"),
    ("NH4Cl", "ammonia"),
    ("NH4NO3", "ammonia"),
    ("H3PO4", "phosphate"),
    ("H2PO4-", "phosphate"),
    ("HPO4-2", "phosphate"),
    ("PO4-3", "phosphate"),
    ("H3BO3", "borate"),
    ("H2BO3-", "borate"),
    ("Na2B4O7", "borate"),
    ("SO2", "sulfite"),
    ("SO3-2", "sulfite"),
    ("HSO3-", "sulfite"),
    ("Na2SO3", "sulfite"),
    ("NaHSO3", "sulfite"),
    ("ClO-", "hypochlorite"),
    ("HClO", "hypochlorite"),
    ("NaOCl", "hypochlorite"),
    ("hydrogen_sulfide", "sulfide"),
    ("H2C2O4", "organic acid"),
    ("citric_acid", "organic acid"),
    ("C6H5O7-3", "organic acid"),
    ("lactic_acid", "organic acid"),
    ("lactate", "organic acid"),
    ("malic_acid", "organic acid"),
    ("ascorbic_acid", "organic acid"),
];

pub fn weak_buffer_class(key: &str) -> Option<&'static str> {
    WEAK_BUFFERS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, class)| *class)
}

fn owned(v: &Vessel, key: &str, admit: Admit) -> f64 {
    v.contents
        .iter()
        .filter(|p| p.species.0 == key && admit.admits(p.phase) && p.moles.0 > 0.0)
        .map(|p| p.moles.0)
        .sum()
}

/// Literal strong-acid protons certified as an ACID-side reserve, beyond what
/// [`crate::displacement::unspent_acidity`] already sees.
///
/// Literal molecular acids are charge-neutral portions, so neither the solute
/// charge nor the ledger weak acids count them. The certified amount is the
/// generic literal-acid protons minus every strong base that would neutralise
/// them first (literal hydroxide molecules plus the positive solute-charge
/// excess, which already carries dissolved hydroxide/basis). Any weak buffer
/// other than the hypochlorite couple the bleach rows consume blocks the
/// certificate entirely (returns zero): its capacity would be pH-dependent,
/// and the aqueous solve, not this reserve, owns that answer.
pub fn certified_literal_acid_protons(v: &Vessel) -> f64 {
    if !v.unresolved_materials.is_empty() || !v.adsorbed.is_empty() || !v.exchanges.is_empty() {
        return 0.0;
    }
    let mut protons = 0.0;
    let mut strong_base = 0.0;
    for p in &v.contents {
        let n = p.moles.0;
        if !n.is_finite() || n < 0.0 {
            return 0.0;
        }
        if n == 0.0 {
            continue;
        }
        let key = p.species.0.as_str();
        if let Some(class) = weak_buffer_class(key) {
            if class != "hypochlorite" && !matches!(p.phase, Phase::Gas) {
                return 0.0;
            }
        }
        if let Some(c) = carrier(key) {
            if !c.admit.admits(p.phase) {
                continue;
            }
            protons += c.proton_eq * n;
            // Literal molecular bases only: dissolved hydroxide and the basis
            // are excluded from solute charge and show up as its positive part.
            if c.cation.is_some() {
                strong_base += c.hydroxide_eq * n;
            }
        }
    }
    let excess = protons - strong_base - v.solute_charge.max(0.0);
    if excess.is_finite() && excess > 0.0 {
        excess
    } else {
        0.0
    }
}

/// Spend `moles` of certified literal strong-acid protons: each proton
/// withdraws its acid from the liquid/aqueous phase and deposits the conjugate
/// anion as dissolved. The caller owns the matching reaction products and
/// refreshes solute charge after the full reaction (same contract as
/// [`crate::displacement::consume_acidity`]). Returns the protons spent.
pub(crate) fn consume_literal_acid(v: &mut Vessel, moles: f64) -> f64 {
    let mut remaining = moles.max(0.0);
    let mut spent = 0.0;
    for c in CARRIERS.iter().filter(|c| c.proton_eq > 0.0) {
        let Some(conjugate) = c.conjugate else {
            continue;
        };
        for phase in [Phase::Aqueous, Phase::Liquid] {
            if remaining <= 0.0 {
                break;
            }
            let have = v
                .contents
                .iter()
                .filter(|p| p.species.0 == c.key && p.phase == phase)
                .map(|p| p.moles.0)
                .sum::<f64>()
                * c.proton_eq;
            let take = remaining.min(have);
            if take <= 0.0 {
                continue;
            }
            let molecules = v
                .withdraw_phase(&SpeciesId::new(c.key), Moles(take / c.proton_eq), phase)
                .0;
            let protons = molecules * c.proton_eq;
            v.deposit(SpeciesId::new(conjugate), Moles(molecules), Phase::Aqueous);
            remaining -= protons;
            spent += protons;
        }
    }
    spent
}

/// The amount of every carrier owned in an admitted phase, for diagnostics.
pub fn owned_carriers(v: &Vessel) -> Vec<(&'static str, f64)> {
    CARRIERS
        .iter()
        .map(|c| (c.key, owned(v, c.key, c.admit)))
        .filter(|(_, n)| *n > 0.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VesselId;

    fn vessel(items: &[(&str, f64, Phase)]) -> Vessel {
        let mut v = Vessel::new(VesselId(0), "reserve");
        for (k, n, p) in items {
            v.deposit(SpeciesId::new(k), Moles(*n), *p);
        }
        v.solute_charge = crate::displacement::solute_charge(&v);
        v
    }

    #[test]
    fn every_carrier_is_a_registry_species_with_matching_charge_balance() {
        for c in CARRIERS {
            // KBr and HNO3 are admitted when a pack defines them; the shipped
            // registry does not, and a missing definition already refuses.
            if matches!(c.key, crate::species::BASE_EQUIVALENTS | "KBr" | "HNO3") {
                continue;
            }
            let data = crate::species::lookup_key(c.key)
                .unwrap_or_else(|| panic!("{} is not a registry species", c.key));
            let f = crate::stoich::parse_formula(data.formula).unwrap();
            // cation - anion - hydroxide + carried protons = formal charge.
            let protons = match c.key {
                "H2SO4" => 2.0,
                "NaHSO4" => 1.0,
                _ if c.conjugate.is_some() => 1.0,
                _ => 0.0,
            };
            let net = c.cation_eq - c.anion_eq - c.hydroxide_eq + protons;
            assert!(
                (net - f.charge).abs() < 1e-12,
                "{}: equivalents {net} vs charge {}",
                c.key,
                f.charge
            );
        }
    }

    #[test]
    fn buffers_are_never_carriers() {
        for (k, _) in WEAK_BUFFERS {
            assert!(carrier(k).is_none(), "{k} is a buffer and a carrier");
        }
    }

    #[test]
    fn literal_hydrobromic_acid_is_certified_and_hcl_is_left_to_its_rows() {
        let v = vessel(&[
            ("water", 5.0, Phase::Liquid),
            ("HBr", 0.01, Phase::Liquid),
            ("HCl", 0.02, Phase::Liquid),
        ]);
        assert!((certified_literal_acid_protons(&v) - 0.01).abs() < 1e-15);
    }

    #[test]
    fn strong_base_is_subtracted_before_acid_is_certified() {
        let v = vessel(&[
            ("water", 5.0, Phase::Liquid),
            ("HBr", 0.01, Phase::Liquid),
            ("NaOH", 0.004, Phase::Solid),
        ]);
        assert!((certified_literal_acid_protons(&v) - 0.006).abs() < 1e-15);
        let neutral = vessel(&[
            ("water", 5.0, Phase::Liquid),
            ("HBr", 0.01, Phase::Liquid),
            ("KOH", 0.01, Phase::Solid),
        ]);
        assert_eq!(certified_literal_acid_protons(&neutral), 0.0);
    }

    #[test]
    fn carbonate_ammonia_and_acetate_buffers_block_the_certificate() {
        for buffer in ["NaHCO3", "NH3", "H3PO4", "Na2B4O7"] {
            let v = vessel(&[
                ("water", 5.0, Phase::Liquid),
                ("HBr", 0.01, Phase::Liquid),
                (buffer, 1e-6, Phase::Aqueous),
            ]);
            assert_eq!(certified_literal_acid_protons(&v), 0.0, "{buffer}");
        }
        // Sulfuric acid's second proton is not certified on the acid side.
        let v = vessel(&[
            ("water", 5.0, Phase::Liquid),
            ("H2SO4", 0.01, Phase::Liquid),
        ]);
        assert_eq!(certified_literal_acid_protons(&v), 0.0);
    }

    #[test]
    fn consuming_literal_acid_conserves_charge_and_elements() {
        let mut v = vessel(&[("water", 5.0, Phase::Liquid), ("HBr", 0.01, Phase::Liquid)]);
        let spent = consume_literal_acid(&mut v, 0.004);
        assert!((spent - 0.004).abs() < 1e-15);
        assert!((v.moles_of(&SpeciesId::new("HBr")).0 - 0.006).abs() < 1e-15);
        assert!((v.moles_of(&SpeciesId::new("Br-")).0 - 0.004).abs() < 1e-15);
        // Boundary: asking for more than is owned spends only what is owned.
        let spent = consume_literal_acid(&mut v, 1.0);
        assert!((spent - 0.006).abs() < 1e-15);
        assert_eq!(v.moles_of(&SpeciesId::new("HBr")).0, 0.0);
    }
}

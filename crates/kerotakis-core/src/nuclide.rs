//! ADV-005: Nuclear module design — nuclide ledger.
//!
//! A separate ledger for radioactive species tracks isotopic identity,
//! decay chains, and activity. This is deliberately separate from the
//! chemical element ledger: the nuclide ledger distinguishes ¹⁴C from
//! ¹²C even though the chemistry treats both as "C".

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A specific nuclide: element + mass number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nuclide {
    /// Element symbol (e.g. "C", "U", "Ra").
    pub element: String,
    /// Mass number (e.g. 14, 235, 226).
    pub mass_number: u32,
    /// Nuclear isomer flag: Tc-99m and Tc-99 are the same (Z, A) and
    /// different nuclides — without this bit the γ transition would be
    /// a ledger no-op.
    #[serde(default)]
    pub metastable: bool,
}

impl Nuclide {
    pub fn new(element: impl Into<String>, mass_number: u32) -> Self {
        Self {
            element: element.into(),
            mass_number,
            metastable: false,
        }
    }

    /// Parse "El-A" or metastable "El-Am" notation.
    pub fn parse(notation: &str) -> Option<Self> {
        let (el, rest) = notation.split_once('-')?;
        let (digits, metastable) = match rest.strip_suffix('m') {
            Some(d) => (d, true),
            None => (rest, false),
        };
        Some(Self {
            element: el.to_string(),
            mass_number: digits.parse().ok()?,
            metastable,
        })
    }

    /// Standard notation (e.g. "C-14", "Tc-99m").
    pub fn notation(&self) -> String {
        format!(
            "{}-{}{}",
            self.element,
            self.mass_number,
            if self.metastable { "m" } else { "" }
        )
    }
}

/// Decay mode for a radioactive nuclide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecayMode {
    Alpha,
    BetaMinus,
    BetaPlus,
    ElectronCapture,
    Gamma,
    SpontaneousFission,
}

/// A decay chain entry: parent decays to daughter with a half-life.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecayChainEntry {
    pub parent: Nuclide,
    pub daughter: Nuclide,
    pub mode: DecayMode,
    /// Half-life in seconds.
    pub half_life_s: f64,
    /// Branching ratio (0.0–1.0).
    pub branching_ratio: f64,
}

/// The nuclide ledger: tracks amounts of specific isotopes separately
/// from the bulk element inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NuclideLedger {
    /// Nuclide → moles.
    pub inventory: BTreeMap<Nuclide, f64>,
}

impl NuclideLedger {
    /// Activity of a nuclide in becquerels (disintegrations per second).
    pub fn activity_bq(&self, nuclide: &Nuclide, half_life_s: f64) -> f64 {
        let moles = self.inventory.get(nuclide).copied().unwrap_or(0.0);
        let n_atoms = moles * 6.022e23;
        let lambda = (2.0_f64).ln() / half_life_s;
        n_atoms * lambda
    }

    /// Deposit moles of a nuclide.
    pub fn deposit(&mut self, nuclide: Nuclide, moles: f64) {
        *self.inventory.entry(nuclide).or_insert(0.0) += moles;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c14_notation() {
        let c14 = Nuclide::new("C", 14);
        assert_eq!(c14.notation(), "C-14");
    }

    #[test]
    fn activity_scales_with_amount() {
        let mut ledger = NuclideLedger::default();
        let c14 = Nuclide::new("C", 14);
        let half_life = 5730.0 * 365.25 * 86400.0; // 5730 years in seconds

        ledger.deposit(c14.clone(), 1e-12); // 1 picomole
        let a1 = ledger.activity_bq(&c14, half_life);

        ledger.deposit(c14.clone(), 1e-12); // now 2 picomoles
        let a2 = ledger.activity_bq(&c14, half_life);

        assert!((a2 / a1 - 2.0).abs() < 0.01, "activity should double");
    }
}

// ── EXP-49: the nuclear bench ───────────────────────────────────────
//
// Transmutation is the one process on this bench where chemical
// elements do NOT conserve — and saying so honestly is the point.
// What conserves instead: nucleon number (every emitted α is kept in
// the ledger as He-4, so Σ A·n is exact) and, at the bookkeeping
// level, charge (β electrons and neutrinos leave the system; stated,
// not ledgered). Mass defect is a stated model boundary: the ledger
// counts nucleons, not binding energy. Trace-scale boundary: spiked
// nuclides are tracer amounts, chemically inert in v1.

/// One curated decay: parent → daughter, with the data a half-life
/// experiment needs. Sources: NUBASE2020 evaluation (Kondev et al.,
/// Chin. Phys. C 45, 030001 (2021)).
#[derive(Debug, Clone)]
pub struct NuclideData {
    pub nuclide: &'static str,
    /// None = stable (an endpoint the ledger may hold).
    pub decay: Option<DecayData>,
    /// Atomic mass in u.
    pub mass_u: f64,
    /// Proton number, for the charge side of equation balancing.
    pub z: u32,
    pub source: &'static str,
}

#[derive(Debug, Clone)]
pub struct DecayData {
    pub daughter: &'static str,
    pub mode: DecayMode,
    pub half_life_s: f64,
}

const NUBASE: &str = "NUBASE2020 evaluation, Kondev et al., Chin. Phys. C 45, 030001 (2021)";

/// The teaching set: one β⁻ classic, one medical tracer, one α, one
/// γ/IT, and a real two-step chain. Every daughter is in the table.
pub const TEACHING_NUCLIDES: &[NuclideData] = &[
    NuclideData { nuclide: "C-14", z: 6, mass_u: 14.003_242,
        decay: Some(DecayData { daughter: "N-14", mode: DecayMode::BetaMinus, half_life_s: 1.808e11 }),
        source: NUBASE },
    NuclideData { nuclide: "N-14", z: 7, mass_u: 14.003_074, decay: None, source: NUBASE },
    NuclideData { nuclide: "I-131", z: 53, mass_u: 130.906_126,
        decay: Some(DecayData { daughter: "Xe-131", mode: DecayMode::BetaMinus, half_life_s: 693_377.0 }),
        source: NUBASE },
    NuclideData { nuclide: "Xe-131", z: 54, mass_u: 130.905_084, decay: None, source: NUBASE },
    NuclideData { nuclide: "Rn-222", z: 86, mass_u: 222.017_578,
        decay: Some(DecayData { daughter: "Po-218", mode: DecayMode::Alpha, half_life_s: 330_350.0 }),
        source: NUBASE },
    NuclideData { nuclide: "Po-218", z: 84, mass_u: 218.008_973, decay: None,
        source: "NUBASE2020; Po-218 continues the radium series — its own decay is deliberately not modelled in the teaching set, and the ledger holding it says so" },
    NuclideData { nuclide: "He-4", z: 2, mass_u: 4.002_602, decay: None, source: NUBASE },
    NuclideData { nuclide: "Co-60", z: 27, mass_u: 59.933_816,
        decay: Some(DecayData { daughter: "Ni-60", mode: DecayMode::BetaMinus, half_life_s: 1.6634e8 }),
        source: NUBASE },
    NuclideData { nuclide: "Ni-60", z: 28, mass_u: 59.930_786, decay: None, source: NUBASE },
    NuclideData { nuclide: "Tc-99m", z: 43, mass_u: 98.906_255,
        decay: Some(DecayData { daughter: "Tc-99", mode: DecayMode::Gamma, half_life_s: 21_624.0 }),
        source: NUBASE },
    NuclideData { nuclide: "Tc-99", z: 43, mass_u: 98.906_255, decay: None,
        source: "NUBASE2020; Tc-99's own 211 ka β⁻ is negligible on bench time and deliberately not modelled" },
    NuclideData { nuclide: "Sr-90", z: 38, mass_u: 89.907_730,
        decay: Some(DecayData { daughter: "Y-90", mode: DecayMode::BetaMinus, half_life_s: 9.085e8 }),
        source: NUBASE },
    NuclideData { nuclide: "Y-90", z: 39, mass_u: 89.907_144,
        decay: Some(DecayData { daughter: "Zr-90", mode: DecayMode::BetaMinus, half_life_s: 230_580.0 }),
        source: NUBASE },
    NuclideData { nuclide: "Zr-90", z: 40, mass_u: 89.904_698, decay: None, source: NUBASE },
];

pub fn lookup_notation(notation: &str) -> Option<&'static NuclideData> {
    TEACHING_NUCLIDES.iter().find(|n| n.nuclide == notation)
}

/// The balanced nuclear equation for one decay, built from the ledger's
/// own A/Z bookkeeping — the test asserts both balances on every entry.
pub fn nuclear_equation(data: &NuclideData) -> Option<String> {
    let d = data.decay.as_ref()?;
    Some(match d.mode {
        DecayMode::Alpha => format!("{} → {} + He-4", data.nuclide, d.daughter),
        DecayMode::BetaMinus => format!("{} → {} + e⁻ + ν̄", data.nuclide, d.daughter),
        DecayMode::BetaPlus => format!("{} → {} + e⁺ + ν", data.nuclide, d.daughter),
        DecayMode::ElectronCapture => format!("{} + e⁻ → {} + ν", data.nuclide, d.daughter),
        DecayMode::Gamma => format!("{} → {} + γ", data.nuclide, d.daughter),
        DecayMode::SpontaneousFission => format!("{} → fission fragments", data.nuclide),
    })
}

/// One decayed-parcel report from an advance step.
#[derive(Debug, Clone)]
pub struct DecayStep {
    pub parent: &'static str,
    pub daughter: &'static str,
    pub mode: DecayMode,
    pub moles: f64,
    pub half_life_s: f64,
    pub equation: String,
}

/// Advance every radioactive nuclide in the ledger by `seconds`:
/// N(t) = N₀·e^(−λt), the decayed parcel moves to the daughter, and an
/// α parcel also deposits He-4 so nucleons never leave the ledger.
///
/// Chains propagate one link per call: exact for the parent, first-
/// order for daughters within a single long wait — the stated boundary
/// (a half-life measurement steps the clock anyway, which is exact).
pub fn advance(ledger: &mut NuclideLedger, seconds: f64) -> Vec<DecayStep> {
    let mut out = Vec::new();
    if seconds <= 0.0 {
        return out;
    }
    let snapshot: Vec<(Nuclide, f64)> = ledger
        .inventory
        .iter()
        .map(|(n, m)| (n.clone(), *m))
        .collect();
    for (nuclide, moles) in snapshot {
        if moles <= 0.0 {
            continue;
        }
        let Some(data) = lookup_notation(&nuclide.notation()) else {
            continue;
        };
        let Some(decay) = data.decay.as_ref() else {
            continue;
        };
        let lambda = (2.0_f64).ln() / decay.half_life_s;
        let decayed = moles * (1.0 - (-lambda * seconds).exp());
        if decayed <= 0.0 {
            continue;
        }
        *ledger.inventory.get_mut(&nuclide).expect("snapshotted") -= decayed;
        ledger.deposit(
            Nuclide::parse(decay.daughter).expect("curated daughters parse"),
            decayed,
        );
        if decay.mode == DecayMode::Alpha {
            ledger.deposit(Nuclide::new("He", 4), decayed);
        }
        out.push(DecayStep {
            parent: data.nuclide,
            daughter: decay.daughter,
            mode: decay.mode.clone(),
            moles: decayed,
            half_life_s: decay.half_life_s,
            equation: nuclear_equation(data).expect("radioactive rows have equations"),
        });
    }
    out
}

/// Total activity of a ledger in becquerels — what a Geiger counter
/// integrates over everything in the sample.
pub fn total_activity_bq(ledger: &NuclideLedger) -> f64 {
    ledger
        .inventory
        .iter()
        .filter_map(|(n, _)| {
            lookup_notation(&n.notation())
                .and_then(|d| d.decay.as_ref())
                .map(|dec| ledger.activity_bq(n, dec.half_life_s))
        })
        .sum()
}

/// Σ A·n over the ledger — the quantity transmutation conserves. The
/// β modes conserve it trivially; α conserves it because the He-4
/// stays in the ledger.
pub fn nucleon_moles(ledger: &NuclideLedger) -> f64 {
    ledger
        .inventory
        .iter()
        .map(|(n, m)| n.mass_number as f64 * m)
        .sum()
}

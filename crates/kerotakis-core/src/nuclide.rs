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
// not ledgered). Mass defect is no longer entirely unclaimed: each
// radioactive row now carries the energy ONE of its decays deposits in
// the sample, and `DecayClock` books that as heat. What is still not
// claimed is the rest of the defect — the neutrino carries roughly two
// thirds of a β⁻ transition's Q out of the world, and a decay CHAIN
// deposits its later steps' energy only once the ledger has actually
// reached those steps. Trace-scale boundary: spiked nuclides are tracer
// amounts, chemically inert in v1.

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
    /// What one decay of this parent leaves behind as heat.
    pub deposits: Deposited,
}

/// The energy ONE decay deposits in the sample, or the reason there is no
/// reviewed value for it.
///
/// This is deliberately not the transition's Q, and the gap between the two
/// is why the number is curated rather than computed from `mass_u`:
///
/// * For **α** it is essentially Q. The α and the recoiling daughter both
///   stop within microns of where they started, so the sample keeps it all.
/// * For **β⁻** it is the MEAN ELECTRON energy, which is roughly a third of
///   the endpoint, because the antineutrino leaves the world carrying the
///   rest. Booking Q here would overstate the heat about threefold.
/// * **Penetrating γ is excluded.** A 1.3 MeV photon crosses a gram of
///   anything and keeps going; it is a dose to the room, not heat in the
///   beaker. Where a row's decay is mostly γ, that row says so.
///
/// It is also not the energy of a decay SERIES. A block of natural uranium
/// in secular equilibrium deposits about 51.7 MeV per U-238 atom — twelve
/// times the 4.270 MeV of the first α — because every daughter down to
/// lead is decaying alongside it. This bench books the energy of the decays
/// it actually counts, and the ledger counts one link per step.
#[derive(Debug, Clone)]
pub enum Deposited {
    /// Mean energy retained by the sample per decay, MeV, and its source.
    Mev { mev: f64, source: &'static str },
    /// No reviewed value. The prose says why; no heat is booked, and the
    /// clock's total says how much of the sample it could not account for.
    NotCurated(&'static str),
}

impl Deposited {
    /// MeV per decay, or zero where nothing is claimed.
    pub fn mev(&self) -> f64 {
        match self {
            Self::Mev { mev, .. } => *mev,
            Self::NotCurated(_) => 0.0,
        }
    }
}

const NUBASE: &str = "NUBASE2020 evaluation, Kondev et al., Chin. Phys. C 45, 030001 (2021)";
/// Provenance for every `Deposited::Mev` below.
///
/// Alpha rows quote the transition Q. Beta rows quote the MEAN electron
/// energy, not the endpoint, and exclude any accompanying gamma: both are
/// stated per row rather than left to be inferred, because a reader who
/// mistakes an endpoint for a mean will be wrong by a factor of three and
/// a reader who counts the gamma will be wrong by more.
///
/// AS WITH THE OTHER CURATED TRANCHES IN THIS REPOSITORY, THIS IS NOT A
/// TRANSCRIPTION FROM A POSITIVELY IDENTIFIED COPY of any single evaluation
/// and no edition-level provenance is claimed. The NNDC/ENSDF decay-data
/// evaluations are the intended primary reference, the values below are the
/// commonly tabulated ones and agree with it to the precision quoted, and
/// every row is flagged for reviewer confirmation before any stronger claim
/// is made.
const DECAY_ENERGY: &str = "Decay energies as commonly tabulated from the NNDC/ENSDF evaluated decay data; alpha rows are the transition Q, beta rows are the MEAN electron energy (not the endpoint) with penetrating gamma excluded. Recorded as commonly tabulated and flagged for reviewer confirmation against a positively identified copy; no edition-level provenance is claimed";

/// The teaching set: one β⁻ classic, one medical tracer, one α, one
/// γ/IT, and a real two-step chain. Every daughter is in the table.
pub const TEACHING_NUCLIDES: &[NuclideData] = &[
    NuclideData { nuclide: "C-14", z: 6, mass_u: 14.003_242,
        decay: Some(DecayData { daughter: "N-14", mode: DecayMode::BetaMinus, half_life_s: 1.808e11 , deposits: Deposited::Mev { mev: 0.0495, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "N-14", z: 7, mass_u: 14.003_074, decay: None, source: NUBASE },
    NuclideData { nuclide: "I-131", z: 53, mass_u: 130.906_126,
        decay: Some(DecayData { daughter: "Xe-131", mode: DecayMode::BetaMinus, half_life_s: 693_377.0 , deposits: Deposited::Mev { mev: 0.1819, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "Xe-131", z: 54, mass_u: 130.905_084, decay: None, source: NUBASE },
    NuclideData { nuclide: "Rn-222", z: 86, mass_u: 222.017_578,
        decay: Some(DecayData { daughter: "Po-218", mode: DecayMode::Alpha, half_life_s: 330_350.0 , deposits: Deposited::Mev { mev: 5.590, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "Po-218", z: 84, mass_u: 218.008_973, decay: None,
        source: "NUBASE2020; Po-218 continues the radium series — its own decay is deliberately not modelled in the teaching set, and the ledger holding it says so" },
    NuclideData { nuclide: "He-4", z: 2, mass_u: 4.002_602, decay: None, source: NUBASE },
    NuclideData { nuclide: "Co-60", z: 27, mass_u: 59.933_816,
        decay: Some(DecayData { daughter: "Ni-60", mode: DecayMode::BetaMinus, half_life_s: 1.6634e8 , deposits: Deposited::Mev { mev: 0.0958, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "Ni-60", z: 28, mass_u: 59.930_786, decay: None, source: NUBASE },
    NuclideData { nuclide: "Tc-99m", z: 43, mass_u: 98.906_255,
        decay: Some(DecayData { daughter: "Tc-99", mode: DecayMode::Gamma, half_life_s: 21_624.0 , deposits: Deposited::NotCurated("an isomeric transition is almost all gamma: the 140.5 keV photon leaves a bench-scale sample rather than heating it, so what stays behind is the small internal-conversion and Auger part. No reviewed split of the two is recorded here, so no heat is booked at all - a tracer that would warm nothing measurable is the wrong place to start inventing one") }),
        source: NUBASE },
    NuclideData { nuclide: "Tc-99", z: 43, mass_u: 98.906_255, decay: None,
        source: "NUBASE2020; Tc-99's own 211 ka β⁻ is negligible on bench time and deliberately not modelled" },
    NuclideData { nuclide: "Sr-90", z: 38, mass_u: 89.907_730,
        decay: Some(DecayData { daughter: "Y-90", mode: DecayMode::BetaMinus, half_life_s: 9.085e8 , deposits: Deposited::Mev { mev: 0.1958, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "Y-90", z: 39, mass_u: 89.907_144,
        decay: Some(DecayData { daughter: "Zr-90", mode: DecayMode::BetaMinus, half_life_s: 230_580.0 , deposits: Deposited::Mev { mev: 0.9337, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "Zr-90", z: 40, mass_u: 89.904_698, decay: None, source: NUBASE },
    // th-122: the block that warms itself. U-238 is added and U-235 is
    // deliberately NOT — `tests/nuclear.rs` uses U-235 as its example of a
    // nuclide the teaching set refuses, and that refusal is worth more than
    // a second uranium row.
    NuclideData { nuclide: "U-238", z: 92, mass_u: 238.050_787,
        decay: Some(DecayData { daughter: "Th-234", mode: DecayMode::Alpha, half_life_s: 1.410e17,
            deposits: Deposited::Mev { mev: 4.270, source: DECAY_ENERGY } }),
        source: NUBASE },
    NuclideData { nuclide: "Th-234", z: 90, mass_u: 234.043_601, decay: None,
        source: "NUBASE2020; Th-234 opens the rest of the uranium series and its own 24-day beta is deliberately not modelled in the teaching set. That truncation is why a block of uranium here warms about a twelfth as fast as a block that has stood long enough for the whole series to reach secular equilibrium, and the ledger holding it says so" },
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

/// Joules per MeV: one megaelectronvolt is the elementary charge times a
/// million volts. Derived from the CODATA constants this crate already
/// carries rather than entered as a second, driftable number.
pub const JOULES_PER_MEV: f64 = crate::constants::ELEMENTARY_CHARGE * 1.0e6;

/// One decayed-parcel report from an advance step.
#[derive(Debug, Clone)]
pub struct DecayStep {
    pub parent: &'static str,
    pub daughter: &'static str,
    pub mode: DecayMode,
    pub moles: f64,
    pub half_life_s: f64,
    pub equation: String,
    /// Energy this parcel of decays deposited in the sample, joules.
    /// Zero where the row states no reviewed energy — which is a refusal
    /// to guess, not a claim that the decay released nothing.
    pub energy_j: f64,
    /// The row that produced `energy_j`, so a reader can see whether it is
    /// a curated number or an admitted gap.
    pub deposits: Deposited,
}

/// Joules deposited by `moles` of decays that each leave `mev` behind.
fn deposited_joules(moles: f64, deposits: &Deposited) -> f64 {
    moles * crate::constants::AVOGADRO * deposits.mev() * JOULES_PER_MEV
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
            energy_j: deposited_joules(decayed, &decay.deposits),
            deposits: decay.deposits.clone(),
        });
    }
    out
}

/// Total activity of a ledger in becquerels — what a Geiger counter
/// integrates over everything in the sample.
pub fn total_activity_bq(ledger: &NuclideLedger) -> f64 {
    ledger
        .inventory
        .keys()
        .filter_map(|n| {
            lookup_notation(&n.notation())
                .and_then(|d| d.decay.as_ref())
                .map(|dec| ledger.activity_bq(n, dec.half_life_s))
        })
        .sum()
}

/// A species whose BULK is one radionuclide, rather than a tracer spiked
/// into a beaker.
///
/// The nuclide ledger and the chemical contents are deliberately separate
/// (see `vessel::Vessel::nuclides`): ¹⁴C and ¹²C are one element to the
/// chemistry and two nuclides here. That separation is right for a tracer
/// and wrong for a block of metal, where the nuclide IS the substance. This
/// table is the bridge, and it is a table rather than a flag so that the
/// isotopic assumption is written down beside the key it applies to.
#[derive(Debug, Clone)]
pub struct BulkRadionuclide {
    /// Registry species key.
    pub species_key: &'static str,
    /// The nuclide the bulk is modelled as, in `TEACHING_NUCLIDES`.
    pub nuclide: &'static str,
    /// What the single-nuclide model gets wrong about the real material.
    pub isotopics: &'static str,
}

pub const BULK_RADIONUCLIDES: &[BulkRadionuclide] = &[BulkRadionuclide {
    species_key: "uranium",
    nuclide: "U-238",
    isotopics: "modelled as pure U-238. Natural uranium is 99.27% U-238 by mass, so the mass is nearly right, but its specific activity is about twice this: U-234 at roughly 54 ppm sits in secular equilibrium with U-238 and, having a half-life eighteen thousand times shorter, contributes almost as many decays again. The 0.72% U-235 adds a little more. Neither is claimed here",
}];

pub fn bulk_radionuclide(species_key: &str) -> Option<&'static BulkRadionuclide> {
    BULK_RADIONUCLIDES
        .iter()
        .find(|entry| entry.species_key == species_key)
}

/// Heat deposited by `moles` of a bulk radionuclide over `seconds`.
///
/// The block is NOT transmuted. Over any time a bench simulates, the
/// decayed fraction of a long-lived nuclide is around one part in a
/// trillion — a day of U-238 turns 4e-13 of the block into thorium — and
/// ledgering that would be false precision dressed as conservation. What is
/// real at that scale is the energy, because a tiny fraction of a very
/// large Avogadro number is still a great many decays.
pub fn bulk_decay_heat(species_key: &str, moles: f64, seconds: f64) -> Option<BulkDecayHeat> {
    if moles <= 0.0 || seconds <= 0.0 {
        return None;
    }
    let bulk = bulk_radionuclide(species_key)?;
    let data = lookup_notation(bulk.nuclide)?;
    let decay = data.decay.as_ref()?;
    let lambda = (2.0_f64).ln() / decay.half_life_s;
    let decayed = moles * (1.0 - (-lambda * seconds).exp());
    Some(BulkDecayHeat {
        species_key: bulk.species_key,
        nuclide: data.nuclide,
        decays: decayed * crate::constants::AVOGADRO,
        energy_j: deposited_joules(decayed, &decay.deposits),
        deposits: decay.deposits.clone(),
        isotopics: bulk.isotopics,
    })
}

/// What a bulk radioactive solid did to itself over one clock step.
#[derive(Debug, Clone)]
pub struct BulkDecayHeat {
    pub species_key: &'static str,
    pub nuclide: &'static str,
    /// Number of decays in the step — the activity, integrated.
    pub decays: f64,
    pub energy_j: f64,
    pub deposits: Deposited,
    pub isotopics: &'static str,
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

//! Cited single-wavelength molar absorptivities (2026-11-11, lane F).
//!
//! The 16-band visible spectrum in [`crate::spectrum`] is what the eye and
//! the peak reading use; most of its curves are band shapes, not a table of
//! measured ε. A real UV-vis method is read at ONE analytical wavelength
//! with a measured ε. This module is that method, generically: every row of
//! [`TABLE`] is one (species, wavelength, ε) value with its source, and
//!
//! ```text
//!   A(λ) = l · Σ_species ε(species, λ, T) · c(species)
//! ```
//!
//! is summed over every species with a cited row at λ (±[`MATCH_NM`]).
//! A wavelength is read only when nothing else present could absorb there
//! unaccounted for: a present chromophore with no cited row at λ, a
//! coloured registry species with no reviewed ε, a species of
//! [`NO_DATA`], the starch–iodine complex, an unmodelled Br2/Br⁻
//! association, or a temperature outside a row's measured range all BLOCK
//! the reading with a typed reason (`not-modeled.uvvis-absorptivity-no-data`,
//! EN/DE/FR) instead of a silently low number.
//!
//! Concentrations: free I2 and I3⁻ come from the triiodide split
//! ([`crate::iodine_chemistry::optical_split`], starch-bound iodine
//! excluded). Every other species is read from the native speciation when
//! the solver represents its elements (molality × solvent kg / litres), and
//! otherwise from the analytical aqueous carriers listed on its row (KMnO4
//! carries one MnO4⁻, K2Cr2O7 one Cr2O7²⁻ ...).
//!
//! Confidence per row: `read` = the number was read in the cited document
//! (primary, or an open-access paper quoting/measuring it); `quoted` = read
//! in a secondary that names the primary; `snippet` = only a quotation of
//! the paper was read (lower confidence, flagged in the coverage table).
//! Coverage: docs/uvvis-absorptivity-coverage-20261111.{md,json}, held by
//! `tools/uvvis_absorptivity_coverage.py --check` and
//! `uvvis_absorptivity_coverage_is_current`.

use crate::species::Phase;
use crate::vessel::Vessel;

/// How firmly a row's number is established (see module docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    Read,
    Quoted,
    Snippet,
}

impl Confidence {
    pub fn key(self) -> &'static str {
        match self {
            Confidence::Read => "read",
            Confidence::Quoted => "quoted",
            Confidence::Snippet => "snippet",
        }
    }
}

/// One cited molar absorptivity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CitedAbsorptivity {
    /// Formula key of the absorbing species (formula-compared against the
    /// native speciation, so `HClO` matches `HOCl`).
    pub species: &'static str,
    /// Analytical wavelength, nm.
    pub nm: f64,
    /// ε at 25 °C (or the source's stated temperature), L mol⁻¹ cm⁻¹.
    pub epsilon: f64,
    /// Relative temperature coefficient of ε per °C about 25 °C, and the
    /// measured range (°C) it holds over. `None`: no temperature
    /// dependence claimed; the row is a room-temperature value.
    pub temperature: Option<(f64, f64, f64)>,
    /// Analytical inventory keys that carry this species when the native
    /// speciation does not represent it: (registry key, species per unit).
    pub carriers: &'static [(&'static str, f64)],
    pub confidence: Confidence,
    pub source: &'static str,
}

/// Wavelength match tolerance, nm: a cited row answers a request within
/// ±1 nm of its own wavelength and nowhere else.
pub const MATCH_NM: f64 = 1.0;

const ACS_OMEGA_2024: &str = "Colorimetric Determination of Dissolved Oxygen: Assessment of \
Methodological Influences on Iodine Spectra, Isosbestic Point, Precision and Accuracy, ACS Omega \
9 (2024) 47350, doi:10.1021/acsomega.4c09084, open access PMC11603272, Table 3 (read 2026-11-11)";
const MCBEATH_2020: &str = "McBeath, Wilkinson & Graham, Chemosphere 251 (2020) 126626, \
doi:10.1016/j.chemosphere.2020.126626 (\"3340 M-1 cm-1 at 525 nm\"), as quoted by Heliyon 2023, \
PMC9958420; 545 nm value from the same source. Older teaching value 2400-2500 at 525 nm conflicts";
const BURKE_1976: &str = "Burke & Mavrodineanu, J. Res. NBS 80A (1976) 631, \
doi:10.6028/jres.080a.062, PMC5293529, Table II: 10.661 kg g-1 cm-1 at 350 nm for K2Cr2O7 in \
0.001 M HClO4, 23.5 C (x 294.18 g/mol = 3136 per Cr2O7, 1568 per Cr; >= 95.8 % of Cr as HCrO4-)";
const CARRINGTON_1956: &str = "Carrington & Symons, J. Chem. Soc. (1956) 3373, \
doi:10.1039/JR9560003373, value as quoted by Wikipedia 'Manganate' (lambda_max 606 nm, \
epsilon 1710)";
const ESWRT_2016: &str = "Environ. Sci.: Water Res. Technol. 2 (2016) 565 (NSF PAR 10283504): \
HOCl 98-101 M-1 cm-1 at 235 nm, OCl- 359-365 M-1 cm-1 at 292 nm (midpoints used)";
const BROMINE_THIOUREA: &str = "'Oxidation of Thiourea by Aqueous Bromine' (snippet via lookchem \
PDF): Br2 171 and Br3- 1006 mol-1 dm3 cm-1 at 390 nm";
const FRANK_OSWALT_1947: &str = "Frank & Oswalt, J. Am. Chem. Soc. 69 (1947) 1321, \
doi:10.1021/ja01198a026, as quoted by a university handout (4700 L mol-1 cm-1, acidic HClO4); an \
I = 0.5 M study gives 2841, so the value is condition dependent";
const CU_AMMINE_2002: &str = "Anal. Chim. Acta (2002), S0003267002001356 (snippet): \
epsilon(640) = 77 M-1 cm-1 for Cu(NH3)4 2+";
const NI_AQUA: &str = "Metz group (UMass) paper (snippet): Ni(H2O)6 2+ 395 nm, epsilon about 5.2 \
M-1 cm-1";

const HAUPT_1952: &str = "Haupt, J. Res. NBS 48 (1952) 414, RP2331 (vol. 48 no. 6, \
nvlpubs V48.N06.A03), Table 1 (molar absorbancy index aM = As/bc, 0.0400 g/L K2CrO4 in 0.05 N KOH, 25 C, c = 2.059e-4 M, \
1 cm): 4795 at 370 nm, 4815 at 375 nm, 1922 at 400 nm, 160 at 450 nm; public-domain NBS paper read \
from nvlpubs.nist.gov";

const DAVIS_GIBSON_1931: &str = "Davis & Gibson, Filters for the reproduction of sunlight and \
daylight and the determination of color temperature, NBS Miscellaneous Publication 114 (1931), \
Table 6: absorbancy of 14.481 g/L CoSO4.(NH4)2SO4.6H2O (395.22 g/mol, 0.03664 M) with 10 cc/L \
H2SO4, 10.00 mm, 25 C: 0.1635 at 500, 0.1742 at 510, 0.1689 at 520 nm, so epsilon = A/c = 4.46, \
4.75, 4.61 L mol-1 cm-1 for Co(H2O)6 2+; Beer's law stated to hold; public-domain NBS publication \
read from nvlpubs.nist.gov";

const FERGUSSON_1936: &str = "Fergusson, Slotin & Style, Trans. Faraday Soc. 32 (1936) 956, \
doi:10.1039/TF9363200956, aqueous chlorine extinction coefficients of their Fig. 2 as digitised \
by the MPI-Mainz UV/VIS Spectral Atlas (Keller-Rudek et al., ESSD 5 (2013) 365, \
doi:10.5194/essd-5-365-2013; cross section = epsilon x 3.8235e-21): 2.08e-19 at 313, 2.54e-19 at \
334 (maximum), 1.05e-19 cm2 at 365 nm, i.e. epsilon 54, 66 and 27 L mol-1 cm-1 (read off a figure)";

/// The cited table. Append rows; never edit a value without its source.
pub const TABLE: &[CitedAbsorptivity] = &[
    CitedAbsorptivity {
        species: "I2",
        nm: 460.0,
        epsilon: 722.0,
        temperature: Some((-0.0019, 20.0, 35.0)),
        carriers: &[],
        confidence: Confidence::Read,
        source: ACS_OMEGA_2024,
    },
    CitedAbsorptivity {
        species: "I3-",
        nm: 460.0,
        epsilon: 1022.0,
        temperature: Some((0.0048, 20.0, 35.0)),
        carriers: &[],
        confidence: Confidence::Read,
        source: ACS_OMEGA_2024,
    },
    CitedAbsorptivity {
        species: "MnO4-",
        nm: 525.0,
        epsilon: 3340.0,
        temperature: None,
        carriers: &[("MnO4-", 1.0), ("KMnO4", 1.0)],
        confidence: Confidence::Quoted,
        source: MCBEATH_2020,
    },
    CitedAbsorptivity {
        species: "MnO4-",
        nm: 545.0,
        epsilon: 3220.0,
        temperature: None,
        carriers: &[("MnO4-", 1.0), ("KMnO4", 1.0)],
        confidence: Confidence::Quoted,
        source: MCBEATH_2020,
    },
    CitedAbsorptivity {
        species: "MnO4-2",
        nm: 606.0,
        epsilon: 1710.0,
        temperature: None,
        carriers: &[("MnO4-2", 1.0), ("K2MnO4", 1.0)],
        confidence: Confidence::Quoted,
        source: CARRINGTON_1956,
    },
    CitedAbsorptivity {
        species: "Cr2O7-2",
        nm: 350.0,
        epsilon: 3136.0,
        temperature: None,
        carriers: &[("Cr2O7-2", 1.0), ("K2Cr2O7", 1.0), ("Na2Cr2O7", 1.0)],
        confidence: Confidence::Read,
        source: BURKE_1976,
    },
    CitedAbsorptivity {
        species: "HCrO4-",
        nm: 350.0,
        epsilon: 1568.0,
        temperature: None,
        carriers: &[("HCrO4-", 1.0)],
        confidence: Confidence::Read,
        source: BURKE_1976,
    },
    CitedAbsorptivity {
        species: "HOCl",
        nm: 235.0,
        epsilon: 100.0,
        temperature: None,
        carriers: &[("HOCl", 1.0)],
        confidence: Confidence::Read,
        source: ESWRT_2016,
    },
    CitedAbsorptivity {
        species: "OCl-",
        nm: 292.0,
        epsilon: 362.0,
        temperature: None,
        carriers: &[("OCl-", 1.0)],
        confidence: Confidence::Read,
        source: ESWRT_2016,
    },
    CitedAbsorptivity {
        species: "Br2",
        nm: 390.0,
        epsilon: 171.0,
        temperature: None,
        carriers: &[("Br2", 1.0)],
        confidence: Confidence::Snippet,
        source: BROMINE_THIOUREA,
    },
    CitedAbsorptivity {
        species: "Br3-",
        nm: 390.0,
        epsilon: 1006.0,
        temperature: None,
        carriers: &[("Br3-", 1.0)],
        confidence: Confidence::Snippet,
        source: BROMINE_THIOUREA,
    },
    CitedAbsorptivity {
        species: "FeSCN+2",
        nm: 447.0,
        epsilon: 4700.0,
        temperature: None,
        carriers: &[("FeSCN+2", 1.0)],
        confidence: Confidence::Quoted,
        source: FRANK_OSWALT_1947,
    },
    CitedAbsorptivity {
        species: "Cu(NH3)4+2",
        nm: 640.0,
        epsilon: 77.0,
        temperature: None,
        carriers: &[("Cu(NH3)4+2", 1.0)],
        confidence: Confidence::Snippet,
        source: CU_AMMINE_2002,
    },
    CitedAbsorptivity {
        species: "Ni+2",
        nm: 395.0,
        epsilon: 5.2,
        temperature: None,
        carriers: &[("Ni+2", 1.0)],
        confidence: Confidence::Snippet,
        source: NI_AQUA,
    },
    CitedAbsorptivity {
        species: "CrO4-2",
        nm: 370.0,
        epsilon: 4795.0,
        temperature: None,
        carriers: &[("CrO4-2", 1.0)],
        confidence: Confidence::Read,
        source: HAUPT_1952,
    },
    CitedAbsorptivity {
        species: "CrO4-2",
        nm: 375.0,
        epsilon: 4815.0,
        temperature: None,
        carriers: &[("CrO4-2", 1.0)],
        confidence: Confidence::Read,
        source: HAUPT_1952,
    },
    CitedAbsorptivity {
        species: "CrO4-2",
        nm: 400.0,
        epsilon: 1922.0,
        temperature: None,
        carriers: &[("CrO4-2", 1.0)],
        confidence: Confidence::Read,
        source: HAUPT_1952,
    },
    CitedAbsorptivity {
        species: "CrO4-2",
        nm: 450.0,
        epsilon: 160.0,
        temperature: None,
        carriers: &[("CrO4-2", 1.0)],
        confidence: Confidence::Read,
        source: HAUPT_1952,
    },
    CitedAbsorptivity {
        species: "Co+2",
        nm: 500.0,
        epsilon: 4.46,
        temperature: None,
        carriers: &[("Co+2", 1.0)],
        confidence: Confidence::Read,
        source: DAVIS_GIBSON_1931,
    },
    CitedAbsorptivity {
        species: "Co+2",
        nm: 510.0,
        epsilon: 4.75,
        temperature: None,
        carriers: &[("Co+2", 1.0)],
        confidence: Confidence::Read,
        source: DAVIS_GIBSON_1931,
    },
    CitedAbsorptivity {
        species: "Co+2",
        nm: 520.0,
        epsilon: 4.61,
        temperature: None,
        carriers: &[("Co+2", 1.0)],
        confidence: Confidence::Read,
        source: DAVIS_GIBSON_1931,
    },
    CitedAbsorptivity {
        species: "Cl2",
        nm: 313.0,
        epsilon: 54.0,
        temperature: None,
        carriers: &[("Cl2", 1.0)],
        confidence: Confidence::Quoted,
        source: FERGUSSON_1936,
    },
    CitedAbsorptivity {
        species: "Cl2",
        nm: 334.0,
        epsilon: 66.0,
        temperature: None,
        carriers: &[("Cl2", 1.0)],
        confidence: Confidence::Quoted,
        source: FERGUSSON_1936,
    },
    CitedAbsorptivity {
        species: "Cl2",
        nm: 365.0,
        epsilon: 27.0,
        temperature: None,
        carriers: &[("Cl2", 1.0)],
        confidence: Confidence::Quoted,
        source: FERGUSSON_1936,
    },
];

/// Absorbers known to be present-and-coloured with NO sourced ε: their
/// presence blocks every cited reading (reason in the coverage doc).
pub const NO_DATA: &[(&str, &str)] = &[(
    "FeCl+2",
    "335 nm band (Liu et al., Chem. Geol. 231 (2006) 326); epsilon only in a figure",
)];

/// Registry carriers whose dissolved split between cited species is a pH
/// equilibrium: they count only through the native speciation, and block
/// the reading when the solver has not represented them.
const SPLIT_CARRIERS: &[(&str, &[&str])] = &[
    ("NaOCl", &["HOCl", "OCl-"]),
    ("KOCl", &["HOCl", "OCl-"]),
    ("Ca(OCl)2", &["HOCl", "OCl-"]),
    // Chromate salts split into CrO4-2, HCrO4- and Cr2O7-2 by pH, so they
    // count only through the native speciation (Haupt's rows are alkaline).
    ("K2CrO4", &["CrO4-2", "HCrO4-", "Cr2O7-2"]),
    ("Na2CrO4", &["CrO4-2", "HCrO4-", "Cr2O7-2"]),
];

const BROMIDE_FORMS: &[&str] = &["Br-", "NaBr", "KBr"];
const TRACE: f64 = 1e-12;

/// One cited-wavelength reading (or its refusal).
#[derive(Debug, Clone, PartialEq)]
pub struct CitedReading {
    pub nm: f64,
    /// Beer–Lambert sum, absorbance units.
    pub absorbance: f64,
    /// (species, ε used at the vessel temperature, mol/L).
    pub contributions: Vec<(String, f64, f64)>,
    /// What prevents a complete reading at this wavelength; empty = read.
    pub blocked: Vec<String>,
}

fn formula_eq(a: &str, b: &str) -> bool {
    a == b
        || matches!(
            (crate::stoich::parse_formula(a), crate::stoich::parse_formula(b)),
            (Ok(x), Ok(y)) if x == y
        )
}

fn aqueous(vessel: &Vessel, key: &str) -> f64 {
    vessel
        .contents
        .iter()
        .filter(|p| p.species.0 == key && matches!(p.phase, Phase::Aqueous | Phase::Liquid))
        .map(|p| p.moles.0.max(0.0))
        .sum()
}

/// Native-speciation moles of `key` when the solver represents every
/// element of it other than H and O; `None` when it does not.
fn native_moles(vessel: &Vessel, key: &str) -> Option<f64> {
    let solution = vessel.solution.as_ref()?;
    let kg = solution
        .solvent_kg
        .filter(|kg| kg.is_finite() && *kg > 0.0)?;
    if solution.species.is_empty() {
        return None;
    }
    let formula = crate::stoich::parse_formula(key).ok()?;
    let heavy: Vec<&String> = formula
        .counts
        .keys()
        .filter(|el| !matches!(el.as_str(), "H" | "O"))
        .collect();
    let represented = solution.species.iter().any(|s| {
        crate::stoich::parse_formula(&s.name)
            .ok()
            .is_some_and(|f| !heavy.is_empty() && heavy.iter().all(|el| f.counts.contains_key(*el)))
    });
    represented.then(|| {
        solution
            .species
            .iter()
            .filter(|s| formula_eq(&s.name, key))
            .map(|s| s.molality.max(0.0) * kg)
            .sum()
    })
}

/// Moles of a table species in the vessel (see module docs).
fn species_moles(vessel: &Vessel, row: &CitedAbsorptivity) -> f64 {
    match row.species {
        "I2" | "I3-" => crate::iodine_chemistry::optical_split(vessel)
            .map(|s| {
                if row.species == "I2" {
                    s.iodine
                } else {
                    s.triiodide
                }
            })
            .unwrap_or(0.0),
        key => native_moles(vessel, key).unwrap_or_else(|| {
            row.carriers
                .iter()
                .map(|(carrier, per)| aqueous(vessel, carrier) * per)
                .sum()
        }),
    }
}

fn distinct_species() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for row in TABLE {
        if !out.contains(&row.species) {
            out.push(row.species);
        }
    }
    out
}

fn first_row(species: &str) -> &'static CitedAbsorptivity {
    TABLE.iter().find(|r| r.species == species).unwrap()
}

/// Cited species present in the vessel (moles above trace).
pub fn present_species(vessel: &Vessel) -> Vec<&'static str> {
    distinct_species()
        .into_iter()
        .filter(|s| species_moles(vessel, first_row(s)) > TRACE)
        .collect()
}

fn is_cited(name: &str) -> bool {
    TABLE.iter().any(|r| formula_eq(name, r.species))
}

/// Everything present that could absorb at `nm` without a cited ε there.
fn blockers(vessel: &Vessel, nm: f64, present: &[&'static str]) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    let t_c = vessel.temperature.0 - 273.15;
    for species in present {
        match TABLE
            .iter()
            .find(|r| r.species == *species && (r.nm - nm).abs() <= MATCH_NM)
        {
            None => {
                out.insert(species.to_string());
            }
            Some(row) => {
                if let Some((_, lo, hi)) = row.temperature {
                    if !(lo..=hi).contains(&t_c) {
                        out.insert(format!("{} ({lo:.0}-{hi:.0} °C)", row.species));
                    }
                }
            }
        }
    }
    let carriers_of_present = |key: &str| {
        TABLE
            .iter()
            .any(|r| present.contains(&r.species) && r.carriers.iter().any(|(c, _)| *c == key))
    };
    let coverage = crate::solution_optics::coverage(vessel);
    for key in coverage
        .missing_absorptivity
        .iter()
        .chain(&coverage.missing_ph)
    {
        if carriers_of_present(key.as_str()) {
            continue;
        }
        if let Some((_, split)) = SPLIT_CARRIERS.iter().find(|(c, _)| c == key) {
            if split.iter().all(|s| native_moles(vessel, s).is_some()) {
                continue;
            }
        }
        out.insert(key.clone());
    }
    for name in &coverage.missing_spectra {
        if !is_cited(name) {
            out.insert(name.clone());
        }
    }
    if coverage.starch_iodine_complex {
        out.insert("starch-iodine complex".to_string());
    }
    for (key, _) in NO_DATA {
        let native = native_moles(vessel, key).unwrap_or(0.0);
        if native > TRACE || aqueous(vessel, key) > TRACE {
            out.insert(key.to_string());
        }
    }
    // Registry 16-band spectra are band shapes, not cited ε at λ.
    for p in vessel
        .contents
        .iter()
        .filter(|p| matches!(p.phase, Phase::Aqueous | Phase::Liquid) && p.moles.0 > TRACE)
    {
        let key = &p.species.0;
        if is_cited(key) || carriers_of_present(key.as_str()) {
            continue;
        }
        let spectral = crate::species::lookup_key(key).is_some_and(|d| d.spectrum.is_some());
        if spectral && crate::solution_optics::visible_moles(vessel, key, p.moles.0) > TRACE {
            out.insert(key.clone());
        }
    }
    // Br2 + Br⁻ ⇌ Br3⁻ is not solved here: without the solver's Br3⁻ the
    // split (and so the 390 nm sum) is unknown.
    if present.contains(&"Br2")
        && native_moles(vessel, "Br3-").is_none()
        && BROMIDE_FORMS.iter().any(|k| aqueous(vessel, k) > TRACE)
    {
        out.insert("Br3-".to_string());
    }
    out.into_iter().collect()
}

/// ε of a row at the vessel temperature (the coefficient applied only
/// inside its measured range; outside it the reading is blocked).
pub fn epsilon_at(row: &CitedAbsorptivity, t_c: f64) -> f64 {
    match row.temperature {
        Some((coef, _, _)) => row.epsilon * (1.0 + coef * (t_c - 25.0)),
        None => row.epsilon,
    }
}

/// Read the vessel at `nm` through `path_cm` of solution.
pub fn absorbance_at(vessel: &Vessel, nm: f64, path_cm: f64) -> CitedReading {
    let present = present_species(vessel);
    let litres = vessel.liquid_volume().0.max(1e-9);
    let t_c = vessel.temperature.0 - 273.15;
    let mut absorbance = 0.0;
    let mut contributions = Vec::new();
    for row in TABLE.iter().filter(|r| (r.nm - nm).abs() <= MATCH_NM) {
        let moles = species_moles(vessel, row);
        if moles <= TRACE {
            continue;
        }
        let epsilon = epsilon_at(row, t_c);
        let molar = moles / litres;
        absorbance += epsilon * molar * path_cm;
        contributions.push((row.species.to_string(), epsilon, molar));
    }
    CitedReading {
        nm,
        absorbance,
        contributions,
        blocked: blockers(vessel, nm, &present),
    }
}

/// Every cited analytical wavelength of the species present, each read
/// (or blocked). Empty when no cited species is present.
pub fn readings(vessel: &Vessel, path_cm: f64) -> Vec<CitedReading> {
    let present = present_species(vessel);
    let mut wavelengths: Vec<f64> = TABLE
        .iter()
        .filter(|r| present.contains(&r.species))
        .map(|r| r.nm)
        .collect();
    wavelengths.sort_by(|a, b| a.total_cmp(b));
    wavelengths.dedup();
    wavelengths
        .into_iter()
        .map(|nm| absorbance_at(vessel, nm, path_cm))
        .collect()
}

//! The λ° table, checked against measurements made outside this repository.
//!
//! # Why this file exists
//!
//! A const-table mutation pass on 2026-09-16 moved every literal in
//! `conductivity.rs` by +25 % and asked whether anything in the tree
//! complained. Twenty-three of twenty-eight mutants survived, and the
//! survivors were nearly the whole limiting-molar-conductivity table: λ°(Ca²⁺)
//! could read 148.7 instead of 118.94 and the conductivity meter would report
//! it without a murmur. The λ° table's own doc comment says it is "measured
//! data, not theory" — and a measured value no test can distinguish from a
//! wrong one is, for the suite's purposes, indistinguishable from a
//! placeholder.
//!
//! The obvious repair is twenty-three assertions, one per constant, each
//! saying that a number is the number it is. That is not a repair. It raises
//! a mutation score and adds no evidence, because a test written to kill a
//! mutant is a test written by someone who already knows the answer.
//!
//! `CONTRIBUTING.md` §4 says what the alternative is, and says it as the
//! deliberate opposite of the engine's rule:
//!
//! > An engine that carries the answer it is supposed to compute has stopped
//! > computing. A test that carries a CITED EXTERNAL MEASUREMENT is doing the
//! > one thing the engine cannot do for itself, which is to check the answer
//! > against the world rather than against another path through the same
//! > database.
//!
//! So every number in this file comes from outside this repository, is named
//! where it comes from, and was read on 2026-09-17 by the route recorded in
//! `provenance/upstreams.toml`. Three things are checked, in descending order
//! of how much they are worth:
//!
//! 1. **The calibration ladder** — six potassium chloride solutions whose
//!    conductivity is fixed by international metrological recommendation.
//!    This is the strongest evidence here, because it is not a compilation of
//!    anyone's table: it is what a conductivity meter is calibrated against,
//!    and it exercises the λ° table, the concentration correction and the
//!    molality/molarity approximation in one number.
//! 2. **The table against two independent compilations** — one American
//!    (public-domain U.S. Geological Survey, reporting Harned and Owen), one
//!    German (a Sartorius handbook, read at one remove). Neither is the
//!    compilation the shipped table was transcribed from, so agreement is
//!    corroboration rather than a round trip.
//! 3. **Relations that are physics** — the anomalous Grotthuss ions, and the
//!    hydration ordering of the alkali metals. These say nothing about a
//!    single value and everything about a table that has drifted.
//!
//! # What this file does NOT establish, stated because the gap is the finding
//!
//! **Six** of the twenty-eight tabulated ions — Zn²⁺, Fe²⁺, Al³⁺, Mn²⁺, Pb²⁺
//! and MnO₄⁻ — are in [`UNCORROBORATED`]. They are not wrong; they are
//! unverified here, and they rest on the single compilation the shipped table
//! names. The list is asserted to be exactly right, so a seventh cannot be
//! added in silence, and so that finding a source for one of them is a
//! one-line deletion.
//!
//! ## The second search, 2026-09-18: one ion out, two disagreements in
//!
//! It was seven. **λ°(Fe³⁺) has left the list**, because two compilations
//! reached by two unrelated routes, neither of them the CRC Handbook, both
//! print ⅓Fe³⁺ = 68 — see the row in [`CORROBORATED`] and the caveat on it.
//! The rest of the search is recorded here rather than in a commit message
//! because **its negative half is the more useful half**.
//!
//! Nine sources were read and four could not be reached. What most of the nine
//! have in common is the reason these ions are hard:
//!
//! * **Glasstone's *Introduction to Electrochemistry* (1942) table XIII**, p.
//!   56, "Ion conductances at infinite dilution at 25° " — read in full, and
//!   it is twenty-two ions ending at ½Mg²⁺ = 53.06. It cites **MacInnes'
//!   *Principles of Electrochemistry* (1939) p. 342**, whose own table the
//!   scan's OCR could not resolve; Glasstone's reproduction of it is what was
//!   read. **Lange's *Handbook of Chemistry* 7th ed. (1949) p. 1417**,
//!   "Equivalent conductance of the separate ions", attributed by Lange's to
//!   Johnston, *J. Am. Chem. Soc.* 31 (1909) 1010 — fifteen rows, of which
//!   nine ion labels survive the scan (Ag⁺, ½Ba²⁺, acetate, ⅓citrate,
//!   ½C₂O₄²⁻, ½Ca²⁺, Cl⁻, ¼Fe(CN)₆⁴⁻, H⁺) and none is one of the six; the
//!   remaining six labels were not legible, so this one is recorded as *not
//!   carrying them as far as could be read* rather than as settled.
//!
//!   Those, with **USGS WSP 2311** and the **Sartorius** table from #625, are
//!   one family: all of them get λ° by splitting a measured Λ° with a
//!   **transference number**, and all of them stop around magnesium.
//!   Transference numbers of that precision were never measured for salts
//!   that hydrolyse, which is what Zn²⁺, Fe²⁺, Fe³⁺, Al³⁺, Mn²⁺ and Pb²⁺ all
//!   do. **They are not missing from the precise tables by accident; they are
//!   excluded by the method those tables use** — which is also why looking
//!   harder in that family will not find them.
//! * **International Critical Tables VI (1929) p. 230, table 3** carries ion
//!   conductances **at 18 °C**, and of the six it has only ½Pb²⁺ = 61. Its
//!   own temperature coefficient would carry that to about 71 at 25 °C, which
//!   is a fifteen per cent extrapolation and settles nothing at the half a
//!   per cent this file works to.
//! * **USGS WSP 2254** (Hem, 1985) has a chapter on conductance and no λ°
//!   table at all.
//! * The **French Wikipedia** list of ionic molar conductivities prints all
//!   six to the digit — because its bibliography is Vanýsek in the CRC
//!   Handbook 87th ed. p. 5-78, which is the shipped table's own source. Two
//!   printings of one table are one source, so this is not corroboration; it
//!   is a confirmation of where the shipped numbers come from, down to a page.
//! * **hydrochemistry.eu** has the author of `phreeqc.dat`'s own account of
//!   how PHREEQC computes specific conductance: from diffusion coefficients,
//!   checked against the Handbook. That is the round trip `LAMBDA_SOURCE`
//!   already refuses, now confirmed from the other end.
//!
//! **Two compilations reached outside that family**, and section 2b is what
//! measuring them showed. [`KRESHKOV`] carries five of the seven and not zinc;
//! [`HUEBSCHMANN`] carries **all seven, zinc included**, and is the only thing
//! found anywhere that does. Both are far too coarse to corroborate — and the
//! two of them **disagree with each other about manganese and lead by seven
//! and eight per cent**, which is why those two ions are not merely unsettled
//! but actively contested. See [`CONTRADICTED`].
//!
//! Not reached, and recorded so the next attempt starts further on: Robinson
//! and Stokes' *Electrolyte Solutions*, Harned and Owen, and Conway's
//! *Electrochemical Data* — all three are on archive.org as lending-restricted
//! items whose search-inside endpoint answers **HTTP 403**; and Johnston's
//! 1909 paper itself, acs.org being unreachable from here as
//! `acs-education` already records.

use kerotakis_core::conductivity::{
    concentration_factor, specific_conductance, Basis, Estimate, DILUTE_LIMIT_MOLAL,
    FITTED_LIMIT_MOLAL, LIMITING_CONDUCTIVITY,
};
use kerotakis_core::vessel::{SolutionInfo, SpeciesDetail};

// ---------------------------------------------------------------------------
// Scaffolding
// ---------------------------------------------------------------------------

fn ion(name: &str, molality: f64) -> SpeciesDetail {
    SpeciesDetail {
        name: name.to_string(),
        molality,
        activity: molality,
    }
}

fn solved(ionic_strength: f64, species: Vec<SpeciesDetail>) -> SolutionInfo {
    SolutionInfo {
        solvent_activity: None,
        scope: Default::default(),
        solvent_kg: None,
        pe: None,
        redox: Vec::new(),
        ph: 7.0,
        ionic_strength,
        species,
        provenance: None,
    }
}

fn shipped(species: &str) -> f64 {
    LIMITING_CONDUCTIVITY
        .iter()
        .find(|(name, _)| *name == species)
        .unwrap_or_else(|| panic!("{species} is not in the shipped λ° table"))
        .1
}

// ---------------------------------------------------------------------------
// 1. The calibration ladder
// ---------------------------------------------------------------------------

/// One potassium chloride standard: the mass of KCl the recipe calls for, and
/// the conductivity that mass is defined to produce at 25 °C.
struct KclStandard {
    /// Grams of KCl per 1000 g of solution, weighed in vacuo.
    grams_per_kg_solution: f64,
    /// Conductivity at 25 °C, µS/cm (the source prints S/m; ×10⁴ here).
    measured_us_cm: f64,
    /// Where the pair comes from.
    source: &'static str,
    /// What the model is allowed to be out by at this point, as a fraction,
    /// and why that number and not a tighter one.
    tolerance: f64,
    /// Whether the model must read HIGH here. The concentration correction is
    /// fitted to hold across two decades and is deliberately gentler than the
    /// truth at the dilute end, so a reading BELOW the standard would mean it
    /// had started over-correcting where the drag is still percent-level.
    must_overestimate: bool,
}

/// Relative molecular mass of KCl. Not a measurement of conductivity and not
/// sourced as one: 39.0983 + 35.45 from the IUPAC standard atomic weights,
/// which the older 35.453 for chlorine moves by 0.004 % — four orders of
/// magnitude below the tolerances below, so the choice cannot matter here.
const KCL_MOLAR_MASS: f64 = 74.5483;

/// The six standards, primary and secondary, of OIML R 56.
///
/// The three primary rows are Jones and Bradshaw's 1933 measurements as
/// corrected to the absolute ohm by the IUPAC recommendation; the three
/// secondary rows are Shedlovsky's of 1932. Both papers are named in the
/// recommendation's reference list and neither was opened here — what was
/// read is the recommendation, which prints the values as the standard.
const KCL_STANDARDS: &[KclStandard] = &[
    // --- secondary standards: Shedlovsky (1932), OIML R 56 table 2 --------
    KclStandard {
        grams_per_kg_solution: 0.074_66,
        measured_us_cm: 146.9,
        source: "OIML R 56 (1981) table 2, secondary standard; T. Shedlovsky, \
                 J. Am. Chem. Soc. 54 (1932) 1411",
        // A millimolal solution is where independent migration is supposed to
        // be at its best, so this is the tightest rung on the ladder and the
        // one that would notice a small drift in λ°(K⁺) or λ°(Cl⁻).
        tolerance: 0.015,
        must_overestimate: true,
    },
    KclStandard {
        grams_per_kg_solution: 0.149_32,
        measured_us_cm: 291.6,
        source: "OIML R 56 (1981) table 2, secondary standard; T. Shedlovsky, \
                 J. Am. Chem. Soc. 54 (1932) 1411",
        tolerance: 0.015,
        must_overestimate: true,
    },
    KclStandard {
        grams_per_kg_solution: 0.373_29,
        measured_us_cm: 718.2,
        source: "OIML R 56 (1981) table 2, secondary standard; T. Shedlovsky, \
                 J. Am. Chem. Soc. 54 (1932) 1411",
        tolerance: 0.015,
        must_overestimate: true,
    },
    // --- primary standards: Jones and Bradshaw (1933), OIML R 56 table 1 --
    KclStandard {
        grams_per_kg_solution: 0.745_263,
        measured_us_cm: 1408.3,
        source: "OIML R 56 (1981) table 1, primary standard (0.01 D); \
                 G. Jones and B. C. Bradshaw, J. Am. Chem. Soc. 55 (1933) 1780, \
                 corrected to the absolute ohm per the IUPAC recommendation",
        // This is the rung the meter is actually calibrated on. 1.5 % is the
        // model's own admitted error here and the module doc quotes it.
        tolerance: 0.02,
        must_overestimate: true,
    },
    KclStandard {
        grams_per_kg_solution: 7.419_13,
        measured_us_cm: 12852.0,
        source: "OIML R 56 (1981) table 1, primary standard (0.1 D); \
                 G. Jones and B. C. Bradshaw, J. Am. Chem. Soc. 55 (1933) 1780",
        // At the top of the calibrated range the fitted attenuation is doing
        // 14 % of the work, so 2 % is the honest allowance.
        tolerance: 0.02,
        must_overestimate: true,
    },
    KclStandard {
        grams_per_kg_solution: 71.135_2,
        measured_us_cm: 111_310.0,
        source: "OIML R 56 (1981) table 1, primary standard (1 D); \
                 G. Jones and B. C. Bradshaw, J. Am. Chem. Soc. 55 (1933) 1780",
        // TEN TIMES LOOSER, AND THE DIRECTION FLIPS. At one molal two of the
        // model's stated approximations have stopped holding at once: the
        // attenuation is extrapolating past the alkali-halide fit, and
        // mol/kgw is no longer mol/L in a solution 7 % salt by mass. The
        // module says both in its own docs; this row is what saying so
        // costs, measured rather than asserted.
        tolerance: 0.10,
        must_overestimate: false,
    },
];

/// Molality in mol/kgw of a standard defined by mass fraction.
fn molality_of(standard: &KclStandard) -> f64 {
    let moles = standard.grams_per_kg_solution / KCL_MOLAR_MASS;
    let water_kg = 1.0 - standard.grams_per_kg_solution / 1000.0;
    moles / water_kg
}

fn kcl_estimate(molality: f64) -> Estimate {
    // A 1:1 electrolyte's ionic strength is its molality.
    specific_conductance(&solved(
        molality,
        vec![ion("K+", molality), ion("Cl-", molality)],
    ))
}

/// The meter must agree with the standards it would be calibrated against,
/// across the two decades they span.
///
/// This is the one test here that is not about a table at all. It is the
/// whole path — λ°(K⁺) + λ°(Cl⁻), the Kohlrausch sum, the fitted attenuation
/// and the mol/L ≈ mol/kgw approximation — compared against a number fixed by
/// an intergovernmental recommendation rather than by anyone's compilation.
#[test]
fn the_meter_tracks_the_oiml_potassium_chloride_standards() {
    for standard in KCL_STANDARDS {
        let molality = molality_of(standard);
        let estimate = kcl_estimate(molality);
        let modelled = estimate.microsiemens_per_cm;
        let error = (modelled - standard.measured_us_cm) / standard.measured_us_cm;
        assert!(
            error.abs() < standard.tolerance,
            "{} g/kg KCl is {molality:.6} mol/kgw: model {modelled:.1} µS/cm \
             against the standard's {:.1} µS/cm, {:+.2} % — outside the {:.1} % \
             this rung allows.\n  source: {}",
            standard.grams_per_kg_solution,
            standard.measured_us_cm,
            100.0 * error,
            100.0 * standard.tolerance,
            standard.source,
        );
        if standard.must_overestimate {
            assert!(
                error > 0.0,
                "{} g/kg KCl: the model read {modelled:.1} µS/cm, BELOW the \
                 standard's {:.1}. Inside the calibrated range the fitted \
                 attenuation is deliberately gentler than the truth, so a low \
                 reading means it has begun over-correcting where ion–ion drag \
                 is still percent-level.\n  source: {}",
                standard.grams_per_kg_solution,
                standard.measured_us_cm,
                standard.source,
            );
        }
    }
}

/// The correction earns its keep: without it the sum misses the standards by
/// far more than with it, at every rung.
///
/// Stated as a comparison rather than as a value, because "the corrected
/// number is closer to the measurement than the uncorrected one" is a claim
/// about the world that no blessing can satisfy.
#[test]
fn the_concentration_correction_is_an_improvement_at_every_standard() {
    for standard in KCL_STANDARDS {
        let molality = molality_of(standard);
        let corrected = kcl_estimate(molality).microsiemens_per_cm;
        let bare = corrected / concentration_factor(molality);
        let with = (corrected - standard.measured_us_cm).abs();
        let without = (bare - standard.measured_us_cm).abs();
        assert!(
            with < without,
            "at {molality:.6} mol/kgw the bare Kohlrausch sum ({bare:.1}) is \
             closer to the measured {:.1} µS/cm than the corrected estimate \
             ({corrected:.1}) — the correction is making the answer worse.\n  \
             source: {}",
            standard.measured_us_cm,
            standard.source,
        );
    }
}

// ---------------------------------------------------------------------------
// 2. The table against independent compilations
// ---------------------------------------------------------------------------

/// One ion's λ°, as some compilation other than the shipped table's prints it.
struct Corroboration {
    /// PHREEQC species key, as [`LIMITING_CONDUCTIVITY`] writes it.
    species: &'static str,
    /// |z|. The published value is per EQUIVALENT where `per_equivalent` is
    /// set, and the table ships per MOLE: Ca²⁺ is ~119, not ~59.5.
    charge: f64,
    /// The number as the source prints it, S·cm²·mol⁻¹ (or per equivalent).
    published: f64,
    /// Whether `published` is per equivalent and must be multiplied by
    /// `charge` to reach the molar value the table ships.
    per_equivalent: bool,
    /// Whether the source states its values are on the INTERNATIONAL ohm.
    /// The USGS table says so in its own header and prints the conversion:
    /// multiply by 0.999505 for SI. It is a 0.05 % effect and it is applied
    /// rather than absorbed into the tolerance, because a correction a source
    /// hands you and you decline to apply is a silent 0.05 % of disagreement.
    international_ohm: bool,
    source: Cited,
}

/// A citation, wrapped in a struct with one field for a reason worth writing
/// down: `kero provenance upstreams` finds a value-bound citation by looking
/// for a FIELD named `source` (or `provenance`, `citation`, ...) followed by a
/// string literal. A `const NAME: &str = "..."` is invisible to it, because
/// what follows the colon there is a type and not a quote.
///
/// That is not a hypothetical. It is why `conductivity.rs`'s own
/// `LAMBDA_SOURCE` — the string that says the whole λ° table came from the
/// CRC Handbook, which `provenance/upstreams.toml` refuses as a systematic
/// source — does not appear in that lint's output at all. The finding is
/// recorded in docs/MUTATION-SENSITIVITY.md §8c rather than repaired here,
/// because making it visible changes a reviewed offence count and that is the
/// owner's call. This file at least declines to add a second invisible one.
struct Cited {
    source: &'static str,
}

/// The conversion the USGS table's own header prints.
const INTERNATIONAL_TO_ABSOLUTE_OHM: f64 = 0.999_505;

/// U.S. Geological Survey, Water-Supply Paper 2311 (Miller, Bradford and
/// Peters, 1988), table 5 on page 10: "Limiting equivalent conductances (λ°)
/// of selected ions". A work of the United States Government, in the public
/// domain, and it names its own upstream: Harned and Owen (1964) page 231,
/// with the fluoride row from Franks (1973) page 178.
const USGS: Cited = Cited {
    source: "USGS Water-Supply Paper 2311 (Miller, Bradford and Peters, 1988) \
             table 5 p. 10, per equivalent on the international ohm; the paper \
             attributes the values to Harned and Owen (1964) p. 231, fluoride \
             to Franks (1973) p. 178",
};

/// Sartorius, "Handbuch der Elektroanalytik, Teil 3: Die elektrische
/// Leitfähigkeit", as reproduced in the German Wikipedia article "Molare
/// Leitfähigkeit" and read there on 2026-09-17. READ AT ONE REMOVE, and
/// recorded that way: the handbook itself was not opened. It earns its place
/// because it is the only reachable source for five ions the USGS table does
/// not carry, and because where the two overlap they agree to 0.2 %.
const SARTORIUS: Cited = Cited {
    source: "Sartorius, Handbuch der Elektroanalytik Teil 3: Die elektrische \
             Leitfähigkeit, as reproduced in \
             de.wikipedia.org/wiki/Molare_Leitfähigkeit and read there \
             2026-09-17 — READ AT ONE REMOVE: the handbook itself was not \
             opened, and the value is the handbook's, not Wikipedia's",
};

const CORROBORATED: &[Corroboration] = &[
    // --- USGS WSP 2311 table 5, per equivalent, international ohm ---------
    c("H+", 1.0, 349.8, true, true, USGS),
    c("OH-", 1.0, 197.8, true, true, USGS),
    c("Li+", 1.0, 38.66, true, true, USGS),
    c("Na+", 1.0, 50.11, true, true, USGS),
    c("K+", 1.0, 73.52, true, true, USGS),
    c("NH4+", 1.0, 73.4, true, true, USGS),
    c("Ca+2", 2.0, 59.50, true, true, USGS),
    c("Mg+2", 2.0, 53.06, true, true, USGS),
    c("Sr+2", 2.0, 59.46, true, true, USGS),
    c("Cl-", 1.0, 76.35, true, true, USGS),
    c("Br-", 1.0, 78.20, true, true, USGS),
    c("I-", 1.0, 76.9, true, true, USGS),
    c("F-", 1.0, 55.32, true, true, USGS),
    c("NO3-", 1.0, 71.44, true, true, USGS),
    c("HCO3-", 1.0, 44.5, true, true, USGS),
    c("SO4-2", 2.0, 80.0, true, true, USGS),
    // --- Sartorius handbook, molar (multivalent rows printed as ½M) -------
    c("Ag+", 1.0, 61.9, false, false, SARTORIUS),
    c("Ba+2", 2.0, 63.6, true, false, SARTORIUS),
    c("Cu+2", 2.0, 53.6, true, false, SARTORIUS),
    c("ClO4-", 1.0, 67.4, false, false, SARTORIUS),
    c("CO3-2", 2.0, 69.3, true, false, SARTORIUS),
    // --- the one ion the 2026-09-18 search moved out of UNCORROBORATED ----
    //
    // ⅓Fe³⁺ = 68, printed identically by TWO compilations reached by two
    // different routes and neither of them the CRC Handbook: Hübschmann and
    // Links (1991) p. 62 via de.wikipedia, and Kreshkov (1970) p. 74 via a
    // Russian reference collection. Both are coarse sources — §2b measures
    // exactly how coarse — and the source named here is the one on the route
    // this repository has already cleared.
    //
    // THE CAVEAT, BECAUSE IT IS REAL: both print two significant figures, so
    // "68" is anything in [67.5, 68.5] and this row cannot distinguish 204.0
    // from 203. What it does establish is the thing UNCORROBORATED exists to
    // deny — that the value rests on one compilation. It no longer does, and
    // two that have nothing to do with each other write down the same number.
    c("Fe+3", 3.0, 68.0, true, false, HUEBSCHMANN),
];

/// `const fn` only so the table above reads as data rather than as twenty-one
/// struct literals.
const fn c(
    species: &'static str,
    charge: f64,
    published: f64,
    per_equivalent: bool,
    international_ohm: bool,
    source: Cited,
) -> Corroboration {
    Corroboration {
        species,
        charge,
        published,
        per_equivalent,
        international_ohm,
        source,
    }
}

/// The ions no second source could be reached for, on 2026-09-17.
///
/// This is not a list of doubtful values. It is a list of values that rest on
/// one compilation, which is a different and smaller claim than the rest of
/// the table can make. The sources that would settle them — Vanýsek's table
/// in the CRC Handbook, Robinson and Stokes' appendix, the transference-number
/// papers that split each salt's Λ° into two ions — are either refused as a
/// systematic source by `provenance/upstreams.toml` or were not reachable.
///
/// SEARCHED AGAIN ON 2026-09-18: SEVEN BECAME SIX. λ°(Fe³⁺) left, on two
/// compilations from unrelated routes printing the same number. Every one of
/// the six that remain now has at least one coarse source and a measured
/// bound (section 2b) — zinc included, which had nothing at all before — and
/// none of them has corroboration, because those bounds are an order of
/// magnitude wider than the half a per cent this file's corroboration test
/// demands. The distinction is the point of keeping this list: "not
/// contradicted by a loose table" and "confirmed by an independent one" are
/// different claims, and only the second one gets an ion out of here.
const UNCORROBORATED: &[&str] = &["Zn+2", "Fe+2", "Al+3", "Mn+2", "Pb+2", "MnO4-"];

/// Every λ° this engine ships that an independent compilation also prints
/// must agree with it.
///
/// Half a percent is the allowance, and it is not a round number picked to
/// pass: the largest disagreement any row below actually shows is 0.19 %
/// (ammonium and fluoride), the international-to-absolute ohm correction is
/// alone worth 0.05 %, and compilations of this vintage differ in the fourth
/// significant figure as a matter of course. Half a percent leaves that
/// headroom and nothing like enough for a value that has moved.
#[test]
fn the_lambda_table_agrees_with_independent_compilations() {
    const TOLERANCE: f64 = 0.005;
    let mut worst = (0.0f64, "");
    for row in CORROBORATED {
        let mut expected = row.published;
        if row.per_equivalent {
            expected *= row.charge;
        }
        if row.international_ohm {
            expected *= INTERNATIONAL_TO_ABSOLUTE_OHM;
        }
        let shipped = shipped(row.species);
        let error = (shipped - expected) / expected;
        assert!(
            error.abs() < TOLERANCE,
            "λ°({}) ships as {shipped} S·cm²·mol⁻¹ but the independent value \
             is {expected:.3} ({:+.2} %), outside the {:.1} % two compilations \
             of this kind may differ by.\n  source: {}",
            row.species,
            100.0 * error,
            100.0 * TOLERANCE,
            row.source.source,
        );
        if error.abs() > worst.0 {
            worst = (error.abs(), row.species);
        }
    }
    // Recorded rather than asserted tightly: if the worst disagreement ever
    // approaches the tolerance, the interesting question is which value moved.
    assert!(
        worst.0 < TOLERANCE,
        "worst disagreement was {} at {:.3} %",
        worst.1,
        100.0 * worst.0
    );
}

/// Every ion in the shipped table is either corroborated above or named as
/// uncorroborated — and nothing is named twice or named and absent.
///
/// This is the part that covers the CLASS rather than the twenty-one values.
/// A new ion added to `LIMITING_CONDUCTIVITY` fails this test until somebody
/// either finds a second source for it or writes it down as resting on one.
#[test]
fn every_shipped_lambda_is_either_corroborated_or_declared_unverified() {
    for (species, _) in LIMITING_CONDUCTIVITY {
        let corroborated = CORROBORATED.iter().any(|row| row.species == *species);
        let declared = UNCORROBORATED.contains(species);
        assert!(
            corroborated != declared,
            "λ°({species}) is {}. Every shipped λ° must be checked against a \
             source that is not the one it was transcribed from, or else be \
             listed in UNCORROBORATED so that resting on a single compilation \
             is a recorded fact rather than an unnoticed one.",
            if corroborated {
                "in both lists"
            } else {
                "in neither list"
            }
        );
    }
    for row in CORROBORATED {
        assert!(
            LIMITING_CONDUCTIVITY.iter().any(|(n, _)| *n == row.species),
            "{} is corroborated here but no longer shipped — drop the row",
            row.species
        );
    }
    for species in UNCORROBORATED {
        assert!(
            LIMITING_CONDUCTIVITY.iter().any(|(n, _)| n == species),
            "{species} is declared unverified here but no longer shipped — \
             drop the row"
        );
    }
}

// ---------------------------------------------------------------------------
// 2b. The two coarse sources, and what measuring them says they can settle
// ---------------------------------------------------------------------------

/// One ion's λ° as one of the coarse compilations prints it.
///
/// Deliberately NOT [`Corroboration`], and the two must not become one struct:
/// a row here has been measured and found too coarse to corroborate, and the
/// distinction is the entire content of this section. What these rows are, is
/// the only reachable evidence of any kind about ions the precise tables do
/// not carry.
struct Coarse {
    species: &'static str,
    /// |z|, as in [`Corroboration`].
    charge: f64,
    /// The number as the source prints it, per EQUIVALENT throughout — both
    /// tables write their multivalent rows as ½Pb²⁺, ⅓Fe³⁺ and so on.
    published: f64,
    source: Cited,
}

/// A. P. Kreshkov, *Osnovy analiticheskoi khimii* (Fundamentals of Analytical
/// Chemistry), Moscow: Khimiya, 1970, p. 74.
///
/// READ AT ONE REMOVE, and through a route thinner than the Sartorius one:
/// the book was not opened, and what was read is the entry reproducing its
/// table in the "Khimicheskii spravochnik" collection at
/// chemical_reference.academic.ru, on 2026-09-18. `de-wikipedia` at least
/// carries a licence anyone can read; that host publishes no terms that could
/// be found, and `provenance/upstreams.toml` says so rather than implying one.
///
/// ONE REASON TO BELIEVE THE TRANSCRIPTION ANYWAY, worth writing down because
/// it is checkable and because three of its values disagree with the rest of
/// this file by enough to look like typing errors made on the way. The source
/// prints two columns, cations and anions, each sorted DESCENDING by Λ°, and
/// the reproduction preserves that order exactly — including for all three
/// outliers: ½Cu²⁺ 56.6 between ½Sr²⁺ 59.5 and ½Cd²⁺ 54, I⁻ 78.8 above Br⁻
/// 78.1, ClO₄⁻ 64.5 between HS⁻ 65 and F⁻ 55.4. A slipped digit would break
/// the sort. So those three are the book's numbers, not the route's damage.
const KRESHKOV: Cited = Cited {
    source: "A. P. Kreshkov, Osnovy analiticheskoi khimii (Fundamentals of \
             Analytical Chemistry), Moscow: Khimiya, 1970, p. 74, limiting \
             equivalent conductivities of ions in water at 25 °C — as \
             reproduced in the Khimicheskii spravochnik entry at \
             chemical_reference.academic.ru and read there 2026-09-18. READ \
             AT ONE REMOVE: no copy of the book was opened, and the value is \
             the book's, not the website's",
};

/// U. Hübschmann and E. Links, *Tabellen zur Chemie*, Hamburg: Verlag
/// Handwerk und Technik, 1991, p. 62, with some values from G. Milazzo,
/// *Elektrochemie*, Vienna: Springer, 1952.
///
/// READ AT ONE REMOVE ON THE SAME ROUTE #625 ALREADY CLEARED — the German
/// Wikipedia, whose terms are CC BY-SA 4.0 and are recorded in the
/// `de-wikipedia` row. The article is "Ionenbeweglichkeit", read 2026-09-18,
/// and it states its own basis in one sentence: *"Grundlage der Tabelle sind
/// Werte der Grenzleitfähigkeiten für 25 °C aus dem Buch Tabellen zur Chemie
/// (Hübschmann, 1991) und einige Werte aus Elektrochemie (Milazzo, 1952)"*.
/// Which of the rows below are Milazzo's and which Hübschmann's it does not
/// say, so both works are named and neither was opened.
///
/// WHY THIS TABLE MATTERS MORE THAN ITS PRECISION SUGGESTS: it is the only
/// thing reached anywhere that carries **λ°(Zn²⁺)**, and the only one that
/// carries all seven of the ions [`UNCORROBORATED`] used to name.
///
/// A CAUTION THE ARTICLE EARNS, and the reason nothing here is read off its
/// other columns. It also prints an ion-mobility column *v*, and says that
/// column was computed from the conductivities rather than the reverse. For
/// most rows the two are consistent under λ = *v*·F. For **three of the ones
/// this section needs** they are not: ½Mn²⁺ prints 50 against a *v* implying
/// 53.5, ½Fe²⁺ prints 53.5 against a *v* implying 68.0 (which is ⅓Fe³⁺'s own
/// mobility, so that cell is simply wrong), and ½Pb²⁺ prints 65 against a *v*
/// implying 70.0. **Only the conductivity column is used here**, because the
/// article says that is the column that came from the books.
const HUEBSCHMANN: Cited = Cited {
    source: "U. Hübschmann and E. Links, Tabellen zur Chemie, Hamburg: Verlag \
             Handwerk und Technik, 1991, p. 62, with some values from \
             G. Milazzo, Elektrochemie, Vienna: Springer, 1952 — as reproduced \
             in the table of de.wikipedia.org/wiki/Ionenbeweglichkeit, which \
             names those two works as its basis, and read there 2026-09-18. \
             READ AT ONE REMOVE: neither book was opened, and the values are \
             the books', not the encyclopaedia's",
};

/// Every row of either table which this engine also ships.
///
/// Twenty-one rows of each overlap ions [`CORROBORATED`] already settles
/// against USGS and Sartorius, and that overlap is not decoration — **it is
/// the instrument**. A compilation nobody here has calibrated cannot settle
/// anything; a compilation measured against twenty-one values two other
/// sources already agree on can settle exactly as much as that measurement
/// allows, and no more.
const COARSE: &[Coarse] = &[
    // --- Kreshkov 1970 p. 74: the calibration rows ------------------------
    k("H+", 1.0, 349.8),
    k("OH-", 1.0, 198.3),
    k("NH4+", 1.0, 73.6),
    k("K+", 1.0, 73.5),
    k("Ba+2", 2.0, 63.6),
    k("Ag+", 1.0, 61.9),
    k("Ca+2", 2.0, 59.5),
    k("Sr+2", 2.0, 59.5),
    k("Cu+2", 2.0, 56.6),
    k("Mg+2", 2.0, 53.1),
    k("Na+", 1.0, 50.1),
    k("Li+", 1.0, 38.7),
    k("SO4-2", 2.0, 80.0),
    k("I-", 1.0, 78.8),
    k("Br-", 1.0, 78.1),
    k("Cl-", 1.0, 76.4),
    k("NO3-", 1.0, 71.5),
    k("CO3-2", 2.0, 69.3),
    k("ClO4-", 1.0, 64.5),
    k("F-", 1.0, 55.4),
    k("HCO3-", 1.0, 44.5),
    // --- Kreshkov 1970 p. 74: the ions nothing precise carries. NO ZINC:
    //     this table is sorted descending and zinc's place, between ½Mg²⁺
    //     53.1 and Na⁺ 50.1, is simply empty. Fe³⁺ has left UNCORROBORATED
    //     on the strength of this row agreeing with the next source's, and
    //     stays here as one of the two halves of that agreement.
    k("Pb+2", 2.0, 70.0),
    k("Fe+3", 3.0, 68.0),
    k("Al+3", 3.0, 63.0),
    k("Fe+2", 2.0, 53.5),
    k("Mn+2", 2.0, 53.5),
    // --- Hübschmann and Links 1991 p. 62: the calibration rows ------------
    h("H+", 1.0, 349.8),
    h("OH-", 1.0, 197.6),
    h("NH4+", 1.0, 73.4),
    h("K+", 1.0, 73.52),
    h("Ba+2", 2.0, 63.64),
    h("Ag+", 1.0, 61.92),
    h("Ca+2", 2.0, 59.5),
    h("Sr+2", 2.0, 59.46),
    h("Cu+2", 2.0, 54.0),
    h("Mg+2", 2.0, 53.06),
    h("Na+", 1.0, 50.11),
    h("Li+", 1.0, 38.69),
    h("SO4-2", 2.0, 80.0),
    h("I-", 1.0, 76.8),
    h("Br-", 1.0, 78.3),
    h("Cl-", 1.0, 76.34),
    h("NO3-", 1.0, 71.44),
    h("CO3-2", 2.0, 74.0),
    h("ClO4-", 1.0, 68.0),
    h("F-", 1.0, 55.0),
    h("HCO3-", 1.0, 44.5),
    // --- Hübschmann and Links 1991 p. 62: all seven, zinc included --------
    h("Zn+2", 2.0, 53.0),
    h("Fe+2", 2.0, 53.5),
    h("Fe+3", 3.0, 68.0),
    h("Al+3", 3.0, 63.0),
    h("Mn+2", 2.0, 50.0),
    h("Pb+2", 2.0, 65.0),
    h("MnO4-", 1.0, 61.0),
];

const fn k(species: &'static str, charge: f64, published: f64) -> Coarse {
    Coarse {
        species,
        charge,
        published,
        source: KRESHKOV,
    }
}

const fn h(species: &'static str, charge: f64, published: f64) -> Coarse {
    Coarse {
        species,
        charge,
        published,
        source: HUEBSCHMANN,
    }
}

/// The two coarse sources, by the string that identifies each.
const COARSE_SOURCES: &[&str] = &[KRESHKOV.source, HUEBSCHMANN.source];

/// The molar value a coarse row prints.
fn coarse_molar(row: &Coarse) -> f64 {
    row.published * row.charge
}

/// The molar value [`CORROBORATED`] independently settles for an ion, with
/// the per-equivalent and international-ohm corrections its own rows carry.
///
/// A ROW WHOSE CORROBORATION IS ITSELF ONE OF THESE COARSE SOURCES IS NOT AN
/// INDEPENDENT VALUE and returns `None`. That is not bookkeeping: λ°(Fe³⁺) is
/// corroborated by [`HUEBSCHMANN`], so calibrating Hübschmann against it would
/// be the source measuring itself and would read a flawless 0 %. The one ion
/// this search moved is therefore the one ion it may not be judged by.
fn independent(species: &str) -> Option<f64> {
    CORROBORATED
        .iter()
        .find(|c| c.species == species)
        .filter(|c| !COARSE_SOURCES.contains(&c.source.source))
        .map(|c| {
            let mut v = c.published;
            if c.per_equivalent {
                v *= c.charge;
            }
            if c.international_ohm {
                v *= INTERNATIONAL_TO_ABSOLUTE_OHM;
            }
            v
        })
}

/// How far one coarse source is from an INDEPENDENT value, over every ion
/// where both it and [`CORROBORATED`] have one.
///
/// External against external: the shipped table is never consulted, so this
/// measures the source and not the engine.
fn calibration(source: &str) -> Vec<(f64, &'static str)> {
    COARSE
        .iter()
        .filter(|row| row.source.source == source)
        .filter_map(|row| {
            independent(row.species)
                .map(|known| (((coarse_molar(row) - known) / known).abs(), row.species))
        })
        .collect()
}

/// The worst that source got an independently settled ion wrong. The bound
/// every claim below is allowed to make, and it is MEASURED rather than
/// chosen — picking a number here would make the next test a pin.
fn measured_bound(source: &str) -> f64 {
    calibration(source)
        .into_iter()
        .fold(0.0f64, |a, (d, _)| a.max(d))
}

/// Both coarse sources are real λ° tables, and both are too coarse to
/// corroborate.
///
/// Both halves are measured, because both are load bearing and they pull in
/// opposite directions.
///
/// MOST OF EACH AGREES CLOSELY. A table that did not reproduce the values two
/// independent compilations already agree on would be a list of numbers, not
/// a compilation, and nothing could be built on it.
///
/// AND SOME OF EACH DOES NOT, BY SEVERAL PER CENT — on ions USGS and a
/// Sartorius handbook have already settled. That is not noise to average
/// away: it is the measured probability that any single row is from a looser
/// lineage, and it applies to the rows nobody can check as much as to the
/// ones where it was caught.
///
/// SO THIS ASSERTS THAT EACH SOURCE'S WORST CALIBRATION ERROR EXCEEDS THE
/// CORROBORATION TOLERANCE. Read it the right way round: it is not a test
/// that wants the sources to be bad. It is the reason the remaining ions do
/// not leave [`UNCORROBORATED`], written where it will expire loudly. If a
/// better printing ever puts all twenty-one rows inside half a per cent, this
/// fails and the next person decides the question again with better evidence.
#[test]
fn both_coarse_sources_are_real_tables_and_too_coarse_to_corroborate() {
    const TOLERANCE: f64 = 0.005;
    for source in COARSE_SOURCES {
        let rows = calibration(source);
        assert_eq!(
            rows.len(),
            21,
            "the calibration is the whole instrument here: {} of this source's \
             rows overlap CORROBORATED, not 21. If an ion moved between the \
             lists, this number moves with it and the bound is no longer the \
             one that was measured.\n  source: {source}",
            rows.len(),
        );
        let inside = rows.iter().filter(|(d, _)| *d < TOLERANCE).count();
        assert!(
            inside >= 17,
            "only {inside} of 21 calibration rows agree with an independent \
             compilation to {:.1} %. Below that this is not a λ° table of the \
             same lineage at all, and its rows for the unchecked ions mean \
             nothing.\n  source: {source}",
            100.0 * TOLERANCE,
        );
        let worst = rows
            .iter()
            .copied()
            .fold((0.0f64, ""), |a, b| if b.0 > a.0 { b } else { a });
        assert!(
            worst.0 > TOLERANCE,
            "this source now agrees with every independently corroborated ion \
             to better than {:.1} % (worst: {} at {:.2} %). It was admitted \
             here as a source too loose to corroborate, and that is why ions \
             it reaches are still in UNCORROBORATED. If this fails the premise \
             has changed — re-decide the question, do not relax the \
             test.\n  source: {source}",
            100.0 * TOLERANCE,
            worst.1,
            100.0 * worst.0,
        );
    }
}

/// The (source, ion) pairs where a coarse source puts the shipped value
/// OUTSIDE its own measured error — a contradiction rather than a caution.
///
/// Two, and both are Hübschmann and Links against a metal the other coarse
/// source reads differently:
///
/// * **½Mn²⁺**: 50 against Kreshkov's 53.5 and the shipped 53.5 — the shipped
///   value is 7.0 % high on this source, and exactly equal to the other one.
/// * **½Pb²⁺**: 65 against Kreshkov's 70 and the shipped 71.0 — 9.2 % high on
///   this source and 1.4 % high on the other.
///
/// **THE TWO SECOND SOURCES DISAGREE WITH EACH OTHER ABOUT MANGANESE AND LEAD
/// BY SEVEN AND EIGHT PER CENT.** Where two independent compilations cannot
/// agree with each other, neither can settle the ion, and the shipped value
/// is not adjudicated by either. That is the finding, and it is recorded
/// rather than acted on.
///
/// The list is asserted to be exactly right, so a third contradiction cannot
/// appear in silence — the same guarantee [`UNCORROBORATED`] gives.
const CONTRADICTED: &[(&str, &str)] = &[("Mn+2", "Hübschmann"), ("Pb+2", "Hübschmann")];

/// Which ions each coarse source contradicts, checked against the list above.
///
/// This is the weaker claim that IS available for the ions no precise
/// compilation reaches, and it is worth having because nothing else in this
/// repository says anything at all about them. It does not say a shipped
/// value is right. It says that a compilation which reaches the ion, and
/// whose error on ions we *can* check is bounded, does or does not put it
/// outside that bound.
#[test]
fn a_coarse_source_that_contradicts_a_shipped_value_is_named_and_counted() {
    let mut found: Vec<(&str, &str)> = Vec::new();
    for row in COARSE {
        let bound = measured_bound(row.source.source);
        let published = coarse_molar(row);
        let error = (shipped(row.species) - published) / published;
        if error.abs() > bound {
            let tag = if row.source.source == KRESHKOV.source {
                "Kreshkov"
            } else {
                "Hübschmann"
            };
            found.push((row.species, tag));
            assert!(
                CONTRADICTED.contains(&(row.species, tag)),
                "λ°({}) ships as {} S·cm²·mol⁻¹ against {tag}'s {published} \
                 ({:+.2} %), OUTSIDE the {:.2} % that source's own worst error \
                 on an independently corroborated ion. That is a NEW \
                 contradiction, and it wants reporting to a human — with both \
                 numbers and both sources — rather than a change to the \
                 shipped value. docs/MUTATION-SENSITIVITY.md §8d is where the \
                 two known ones are written down.\n  source: {}",
                row.species,
                shipped(row.species),
                100.0 * error,
                100.0 * bound,
                row.source.source,
            );
        }
    }
    for pair in CONTRADICTED {
        assert!(
            found.contains(pair),
            "{} is recorded as contradicted by {} and is not — the \
             disagreement has gone away, which is a thing to look at rather \
             than to delete.",
            pair.0,
            pair.1,
        );
    }
}

/// What this instrument can detect, stated as a test instead of as a claim.
///
/// The const-table mutation pass moves a literal by ±25 %. Six mutants in
/// `conductivity.rs` survived the 2026-09-17 re-run, and they were exactly the
/// ions no second source reached. Every one of them now has a source — too
/// coarse to corroborate it, and nowhere near too coarse to notice a quarter.
///
/// So this asserts the sensitivity directly: for every ion still in
/// [`UNCORROBORATED`], a value a quarter high AND a quarter low both fall
/// outside the measured bound of at least one source that carries it, which
/// is what [`a_coarse_source_that_contradicts_a_shipped_value_is_named_and_counted`]
/// would then report. **It is the survivor count without running the harness
/// and without a falsified constant ever touching the disk** — the hazard
/// §8b's two SIGKILLs left in a worktree.
///
/// THEN THE HARNESS WAS RUN AND AGREED: same six ids, on a runner,
/// `tools/mutation/results/2026-09-18-conductivity-rerun.json` — **6 caught,
/// 0 survived**, five of them by this test and all six by the one above.
/// §8b's twenty-three conductivity survivors are closed.
///
/// THAT IS NOT THE SAME AS THE VALUES BEING RIGHT, and the distinction is
/// the reason this file is long. A hundred per cent here means the tree would
/// notice a quarter. Five of the six are still uncorroborated, two of them are
/// contradicted outright by one of the sources that reach them, and two more
/// disagree with both.
#[test]
fn a_quarter_wrong_would_fall_outside_the_measured_bound() {
    for species in UNCORROBORATED {
        let mut seen = false;
        for row in COARSE.iter().filter(|r| r.species == *species) {
            seen = true;
            let bound = measured_bound(row.source.source);
            let published = coarse_molar(row);
            for factor in [1.25, 0.75] {
                let mutated = shipped(species) * factor;
                let error = ((mutated - published) / published).abs();
                assert!(
                    error > bound,
                    "a λ°({species}) moved to {mutated} — {factor}× the \
                     shipped value — would still sit inside this source's own \
                     {:.2} % error, so a quarter-wrong value here is invisible \
                     to §2b and this ion's mutant survives for a second \
                     reason.\n  source: {}",
                    100.0 * bound,
                    row.source.source,
                );
            }
        }
        assert!(
            seen,
            "λ°({species}) is uncorroborated AND no coarse source carries it, \
             so nothing in this repository can distinguish it from a value a \
             quarter wrong. That was true of all seven before 2026-09-18 and \
             is true of none of them now; if it becomes true again, \
             docs/MUTATION-SENSITIVITY.md §8d's survivor count is wrong."
        );
    }
}
// ---------------------------------------------------------------------------
// 3. Relations that are physics
// ---------------------------------------------------------------------------

/// The proton and the hydroxide do not migrate; they hop.
///
/// Grotthuss transfer moves the charge along a hydrogen-bond chain without
/// moving any one particle the length of the path, and it is the reason these
/// two ions stand outside every other row in the table. Per unit charge — the
/// only fair comparison, since a divalent ion carries twice the current for
/// the same drift — no ordinary aqueous ion reaches half of the hydroxide's
/// mobility, let alone the proton's.
///
/// A relation, not a value: it holds for the table as shipped, and it would
/// still be the right claim about a table with a different ion in it.
#[test]
fn the_grotthuss_ions_stand_apart_from_every_ordinary_ion() {
    let per_charge = |species: &str| {
        let z = kerotakis_core::conductivity::ion_charge(species).unsigned_abs() as f64;
        shipped(species) / z
    };
    let fastest_ordinary = LIMITING_CONDUCTIVITY
        .iter()
        .filter(|(name, _)| *name != "H+" && *name != "OH-")
        .map(|(name, _)| (per_charge(name), *name))
        .fold((0.0f64, ""), |a, b| if b.0 > a.0 { b } else { a });
    assert!(
        shipped("H+") > shipped("OH-"),
        "the proton hops faster than the hole it leaves: λ°(H+) must exceed \
         λ°(OH-)"
    );
    for hopper in ["H+", "OH-"] {
        assert!(
            per_charge(hopper) > 2.0 * fastest_ordinary.0,
            "λ°({hopper})/|z| = {:.1} is not twice the fastest ordinary ion \
             ({} at {:.1}). Either an ordinary ion has been given a mobility \
             it cannot have, or a Grotthuss ion has lost one.",
            per_charge(hopper),
            fastest_ordinary.1,
            fastest_ordinary.0,
        );
    }
}

/// Down group 1 the bare ion gets bigger and the moving ion gets smaller.
///
/// Li⁺ has the smallest crystal radius and therefore the strongest field, so
/// it drags the largest hydration shell and is the slowest of the three; K⁺
/// the largest crystal radius, the weakest field, the smallest shell and the
/// highest mobility. It is the textbook demonstration that what moves through
/// water is not the ion.
#[test]
fn hydration_reverses_the_alkali_ordering() {
    assert!(
        shipped("Li+") < shipped("Na+") && shipped("Na+") < shipped("K+"),
        "λ°(Li+) < λ°(Na+) < λ°(K+) is the hydrated-radius ordering, and the \
         table reads {} / {} / {}",
        shipped("Li+"),
        shipped("Na+"),
        shipped("K+"),
    );
}

// ---------------------------------------------------------------------------
// 4. The two boundaries, tied to the measurements that justify them
// ---------------------------------------------------------------------------

/// The dilute limit IS the last primary standard, to within a percent.
///
/// [`DILUTE_LIMIT_MOLAL`] is what `within_dilute_limit` means, and what the
/// flag is FOR is the promise that the reading sits inside the range where
/// the model has been shown to track a measurement. The ladder above says
/// where that range ends: the 0.1 D primary standard is 0.1003 mol/kgw and
/// the model is 0.5 % out there; the next standard up is ten times further
/// on and the model is 6 % out. The constant and the standard agree to a
/// quarter of a percent, which is not a coincidence and should not be able
/// to stop being true quietly.
///
/// So the boundary is bracketed by the standard rather than compared with
/// itself: one percent below the 0.1 D solution the meter is inside its
/// calibrated range, one percent above it the meter is not. WHAT THIS IS NOT:
/// a measurement of 0.1 mol/kgw. No experiment returns that number. It is the
/// rule that the flag may not run past the last solution anybody calibrated
/// against, and it is the weakest test in this file for exactly that reason —
/// it is here because the alternative, asserting that a constant equals
/// itself, is worth less.
#[test]
fn the_dilute_limit_is_the_last_primary_standard() {
    let last_standard = molality_of(&KCL_STANDARDS[4]);
    assert!(
        (last_standard - 0.1003).abs() < 0.001,
        "KCL_STANDARDS[4] should be the 0.1 D solution, found \
         {last_standard:.4} mol/kgw"
    );
    let inside = last_standard * 0.99;
    assert!(
        kcl_estimate(inside).within_dilute_limit,
        "at {inside:.4} mol/kgw — one percent below the 0.1 D primary standard, \
         where the model is 0.5 % out — the meter refuses to call itself \
         calibrated. DILUTE_LIMIT_MOLAL is {DILUTE_LIMIT_MOLAL}."
    );
    let outside = last_standard * 1.01;
    assert!(
        !kcl_estimate(outside).within_dilute_limit,
        "at {outside:.4} mol/kgw the model still calls itself dilute, but the \
         calibration ladder stops at the 0.1 D standard, {last_standard:.4} \
         mol/kgw, and the next standard above it is ten times further on where \
         the model is 6 % out. DILUTE_LIMIT_MOLAL is {DILUTE_LIMIT_MOLAL}."
    );
}

/// Past the most concentrated measurement the correction was fitted to, the
/// estimate must say it is extrapolating.
///
/// `conductivity::FIT_SOURCE` names the fit's targets, and the furthest one out
/// is sodium chloride at 2 mol/L. `within_fitted_range` exists to say when the
/// correction has left its data; a boundary above the last fitted point would
/// be the flag telling the reader that an extrapolation is not one.
#[test]
fn past_the_last_fitted_measurement_the_estimate_admits_extrapolating() {
    // 1.71 mol/kgw — ten grams of table salt in 100 mL, and inside the fit.
    let brine = specific_conductance(&solved(1.71, vec![ion("Na+", 1.71), ion("Cl-", 1.71)]));
    assert!(
        brine.within_fitted_range,
        "1.71 mol/kgw sits between the fit's 1 mol/L and 2 mol/L NaCl targets \
         and must not be called an extrapolation"
    );
    // The fit's furthest target is 2 mol/L NaCl, whose ionic strength is 2.0
    // mol/kgw. Anything above it is past the data.
    let past = 2.01;
    let beyond = specific_conductance(&solved(past, vec![ion("Na+", past), ion("Cl-", past)]));
    assert!(
        !beyond.within_fitted_range,
        "at {past} mol/kgw the estimate still claims to be inside the fitted \
         range, but the most concentrated solution the two coefficients were \
         fitted to is 2 mol/L NaCl. FITTED_LIMIT_MOLAL is {FITTED_LIMIT_MOLAL}."
    );
    assert!(
        !beyond.trustworthy(),
        "an extrapolation is never in calibration"
    );
}

/// The coverage fraction is a fraction.
///
/// Not a measurement, and it is here because it is the cheapest true thing
/// that can be said about the number the Kohlrausch path reports beside its
/// answer: no sum over a subset of the charge can cover more of the charge
/// than there is. The const-table pass's sibling run found that nothing in
/// the tree asserted it.
#[test]
fn the_covered_charge_fraction_is_never_more_than_all_of_it() {
    let cases = [
        solved(0.01, vec![ion("K+", 0.01), ion("Cl-", 0.01)]),
        solved(0.01, vec![ion("K+", 0.01), ion("W12O41-10", 0.001)]),
        solved(0.0, vec![]),
        solved(0.001, vec![ion("AgCl", 0.001)]),
    ];
    for case in cases {
        let estimate = specific_conductance(&case);
        if let Basis::Kohlrausch {
            covered_charge_fraction,
            ..
        } = estimate.basis
        {
            assert!(
                (0.0..=1.0).contains(&covered_charge_fraction),
                "covered_charge_fraction = {covered_charge_fraction} is not a \
                 fraction"
            );
        }
        assert!(
            estimate.concentration_factor <= 1.0,
            "the correction can only take conductance away, never add it"
        );
    }
}

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
//! Seven of the twenty-eight tabulated ions — Zn²⁺, Fe²⁺, Fe³⁺, Al³⁺, Mn²⁺,
//! Pb²⁺ and MnO₄⁻ — are in [`UNCORROBORATED`] because **no second source for
//! them could be reached**. They are not wrong; they are unverified here, and
//! they rest on the single compilation the shipped table names. The list is
//! asserted to be exactly right, so an eighth cannot be added in silence, and
//! so that finding a source for one of them is a one-line deletion.

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
    source: &'static str,
}

/// The conversion the USGS table's own header prints.
const INTERNATIONAL_TO_ABSOLUTE_OHM: f64 = 0.999_505;

/// U.S. Geological Survey, Water-Supply Paper 2311 (Miller, Bradford and
/// Peters, 1988), table 5 on page 10: "Limiting equivalent conductances (λ°)
/// of selected ions". A work of the United States Government, in the public
/// domain, and it names its own upstream: Harned and Owen (1964) page 231,
/// with the fluoride row from Franks (1973) page 178.
const USGS: &str = "USGS Water-Supply Paper 2311 (Miller, Bradford and Peters, \
    1988) table 5 p. 10, per equivalent on the international ohm; the paper \
    attributes the values to Harned and Owen (1964) p. 231, fluoride to Franks \
    (1973) p. 178";

/// Sartorius, "Handbuch der Elektroanalytik, Teil 3: Die elektrische
/// Leitfähigkeit", as reproduced in the German Wikipedia article "Molare
/// Leitfähigkeit" and read there on 2026-09-17. READ AT ONE REMOVE, and
/// recorded that way: the handbook itself was not opened. It earns its place
/// because it is the only reachable source for five ions the USGS table does
/// not carry, and because where the two overlap they agree to 0.2 %.
const SARTORIUS: &str = "Sartorius, Handbuch der Elektroanalytik Teil 3: Die \
    elektrische Leitfähigkeit, as reproduced in de.wikipedia.org/wiki/\
    Molare_Leitfähigkeit and read there 2026-09-17 — read at one remove";

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
];

/// `const fn` only so the table above reads as data rather than as twenty-one
/// struct literals.
const fn c(
    species: &'static str,
    charge: f64,
    published: f64,
    per_equivalent: bool,
    international_ohm: bool,
    source: &'static str,
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
const UNCORROBORATED: &[&str] = &["Zn+2", "Fe+2", "Fe+3", "Al+3", "Mn+2", "Pb+2", "MnO4-"];

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
            row.source,
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

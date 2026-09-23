//! Tier C: the Henry's-law table, checked against a compilation it did not
//! come from — and against the databases the solver beside it runs on.
//!
//! `properties::HENRY_COEFFICIENTS` carries six gases from Sander (2015),
//! an atmospheric-chemistry compilation. The aqueous solver runs on the USGS
//! PHREEQC databases, a different lineage, and four of those vendored files
//! tabulate the same gas dissolutions. `tools/gen-henry-oracle.py` reads them
//! out into `fixtures/henry_databases.tsv`.
//!
//! **What this is independent of.** The reference is a different compilation
//! by different people from a different field, so this is not the round trip
//! the colligative family had to disclaim — it is not our arithmetic checked
//! against our own dataset. **What it is NOT independent of:** the primary
//! measurements. Two compilations of the same gas's solubility may well rest
//! on overlapping experiments, and where they agree to a tenth of a per cent
//! (CO2 does) that is the most likely explanation. Agreement here means the
//! number we ship is the number the field carries; it does not mean the field
//! is right.
//!
//! **And it is more than an oracle.** `kerotakis-phreeqc::aqueous` asks
//! `henry_lookup("CO2")` for a solubility while solving a speciation on a
//! database that states its own. So a disagreement on this fixture is not a
//! remote discrepancy: it is the bench holding two answers to "how much of
//! this gas dissolves", and the six gases below are consumed by the fizz in
//! `displacement.rs`, the carbonate clock in `clock.rs` and the outgassing in
//! `volatility.rs`.
//!
//! Two of the twelve rows DISAGREE, materially, and they are pinned rather
//! than banded away. See `TEMPERATURE_DISAGREEMENTS`.

use kerotakis_core::properties::{henry_lookup, HENRY_COEFFICIENTS};

const R: f64 = 8.314_462_618;

// Every gap in this file is `|ours - reference| / reference`. Stated once,
// because the first draft of the pinned disagreements below was written
// against `/ ours` and the two conventions differ by a quarter on the very
// rows the pin exists for — the H2/phreeqc.dat row read 20.6% under one and
// 26.0% under the other, and the pin caught it.

struct Row {
    gas: String,
    database: String,
    log_k: f64,
    delta_h_j: Option<f64>,
}

fn rows() -> Vec<Row> {
    include_str!("fixtures/henry_databases.tsv")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            assert_eq!(c.len(), 5, "fixture row has the wrong shape: {l}");
            Row {
                gas: c[0].to_string(),
                database: c[1].to_string(),
                log_k: c[2].parse().expect("log_k"),
                delta_h_j: (c[3] != "na").then(|| c[3].parse().expect("delta_h")),
            }
        })
        .collect()
}

/// Relative agreement claimed for each gas's solubility constant, with the
/// argument for the width. Every band is set by the spread BETWEEN the USGS
/// databases themselves, because nothing tighter than that spread is a
/// meaningful claim about a compiled value; and every band is narrow enough
/// to exclude a transposed digit in the shipped coefficient.
const SOLUBILITY_BANDS: &[(&str, f64, &str)] = &[
    (
        "CO2",
        0.015,
        "1.5%. phreeqc.dat and wateq4f.dat both say -1.468 and pitzer.dat says \
         -1.465, so the USGS files spread 0.7% among themselves and no band \
         below that means anything; the rest is the molal-to-molar conversion \
         this comparison deliberately ignores (~0.2% in pure water at 25 C). \
         EXCLUDES: 3.5e-2 in place of our 3.4e-2, which is 2.9% away — the \
         shape of a transcription slip in the only gas the carbonate clock, \
         the fizz and the aqueous solver all read.",
    ),
    (
        "O2",
        0.04,
        "4%. phreeqc.dat and llnl.dat agree exactly (-2.8983), so there is no \
         inter-database spread to spend and the whole band is the gap between \
         the two compilations, 2.8%, plus headroom. EXCLUDES: 1.4e-3 for our \
         1.3e-3, 7.7% away.",
    ),
    (
        "N2",
        0.12,
        "12%, and it is wide because the references disagree: phreeqc.dat's \
         -3.1864 and wateq4f.dat's -3.26 are 18% apart, with Sander's 6.1e-4 \
         sitting between them. A band tighter than the reference spread would \
         be a claim the references do not support. EXCLUDES: a decimal slip or \
         a factor of two, and it is recorded here that nitrogen's solubility \
         is the least well corroborated number in this table.",
    ),
    (
        "H2",
        0.115,
        "11.5%. phreeqc.dat (-3.1) and wateq4f.dat (-3.15) are 12% apart and \
         Sander's 7.8e-4 sits between them, 1.8% from one and 9.2% from the \
         other. Same argument as nitrogen: the band is the reference spread. \
         EXCLUDES: a factor of two, and any drift that would take our value \
         outside the range the two USGS files bracket.",
    ),
    (
        "NH3",
        0.115,
        "11.5%. phreeqc.dat and llnl.dat say 1.7966 and wateq4f.dat says 1.77, \
         6.3% apart; our 5.7e1 is 9.8% from the first pair and 3.3% from the \
         second. The band admits both references. EXCLUDES: 6.0e1 or 5.0e1 for \
         our 5.7e1, both further out than any reference sits.",
    ),
];

/// Gases in the shipped table that this oracle CANNOT reach, and why. The
/// reverse check below fails if a gas is neither corroborated nor listed
/// here, so a seventh gas added to `HENRY_COEFFICIENTS` cannot quietly
/// arrive with no check and no admission that it has none.
const UNREACHABLE: &[(&str, &str)] = &[(
    "Cl2",
    "No vendored database writes chlorine's dissolution as `Cl2 = Cl2`. \
     llnl.dat writes `Cl2 + H2O = 0.5 O2 + 2 Cl- + 2 H+`, whose log_k folds \
     in hydrolysis and a redox half-reaction; comparing that to a Henry \
     constant would be wrong by orders of magnitude and would read as a \
     finding. So chlorine's 9.2e-2 mol/(L atm) and its 2500 K rest on Sander \
     alone and are corroborated by nothing in this repository. \
     \
     CHECKED AGAINST THE PUBLISHED SPREAD, 2026-09-22, and BOTH NUMBERS SIT \
     OUTSIDE IT. Sander's own current compilation (version 5.0.0, ACP 23 \
     (2023) 10901, CC-BY, per-species table at henrys-law.org for CAS \
     7782-50-5) lists: 6.0e-4 with 3000 K (Yakovkin 1900, measured), 6.2e-4 \
     with 3500 K (Whitney and Vivian 1941, measured), 6.1e-4 with 3200 K \
     (Aieta and Roberts 1986, measured), 6.1e-4 with 2800 K (Wagman 1982, \
     theory), 7.4e-4 with 2600 K (Lin and Pehkonen 1998, review) and 8.7e-4 \
     (Hayer 2022, estimate), all mol/(m^3 Pa). Our 9.2e-2 mol/(L atm) is \
     9.08e-4 in those units - ABOVE every entry, including the estimate - \
     and our 2500 K is BELOW every temperature dependence listed. \
     \
     THE LIKELY CAUSE IS A QUANTITY MISMATCH, not a transcription slip, and \
     it has this repository's own name on it: a number correct for the thing \
     it belongs to, applied to a different thing wearing the same name. \
     Chlorine hydrolyses, so there are two constants - the INTRINSIC one for \
     molecular Cl2, and an EFFECTIVE one that also counts the HOCl. Sander \
     marks the entries that report the second with a note reading 'the total \
     solubility of chlorine (i.e. the sum of Cl2 and HOCl)', and that note is \
     on his OWN earlier reviews of 2011 and 2006. Our 9.2e-2 mol/(L atm) is \
     0.092 mol/L at one atmosphere, which is the textbook TOTAL solubility of \
     chlorine, about 6.5 g/L. The intrinsic value is near 6.1e-4, half as \
     much again lower. Sander is explicit that the effective constant should \
     be avoided for halogens, because it is not a constant: as the chlorine \
     grows more dilute the hydrolysis runs further right and the ratio moves. \
     \
     WHAT IT COSTS TODAY: nothing computed, and that is why this is recorded \
     rather than fixed here. `volatility::coefficient_for` returns None for \
     any species the registry carries as a gas, which chlorine is, and every \
     other caller of `henry_lookup` names CO2 explicitly - so no solver reads \
     this row. It is still PRINTED: `kero properties` lists every coefficient \
     in this table, and the `henry` query answers `gas=Cl2`. So the bench \
     will tell somebody this number while no test can catch it being wrong. \
     \
     WHY IT IS NOT CHANGED IN THIS COMMIT: picking the replacement is a \
     judgement, not an arithmetic. Three independent determinations 86 years \
     apart agree on 6.0-6.2e-4, which is the obvious candidate; but Sander's \
     own first-listed row is Burkholder (2019), which carries the total \
     solubility note, so the compilation's recommendation and its measured \
     rows do not point the same way. That is an owner's call about which \
     value this bench should publish, and it should be made deliberately.",
)];

/// Where the two lineages agree on the temperature coefficient.
const TEMPERATURE_BANDS: &[(&str, f64, &str)] = &[
    (
        "CO2",
        0.02,
        "2%. -4.776 kcal in phreeqc.dat and wateq4f.dat is 2403 K against our \
         2400; pitzer.dat's -20.19 kJ is 2428 K, 1.2% out, which is the whole \
         spread the band has to hold. EXCLUDES: 2000 or 3000 K, either of \
         which would move the fizz visibly over a bench's temperature range.",
    ),
    (
        "O2",
        0.04,
        "4%. Only llnl.dat states an enthalpy for oxygen (-12.1336 kJ, 1459 K \
         against our 1500, 2.7% out); phreeqc.dat gives an analytic expression \
         instead, which the generator refuses to differentiate. One reference, \
         so the band is the gap plus headroom rather than a spread. EXCLUDES: \
         a 10% drift, which over 0 to 40 C is worth about 1% in solubility.",
    ),
    (
        "NH3",
        0.03,
        "3%. wateq4f.dat's -8.17 kcal gives 4111 K and llnl.dat's -35.2251 kJ \
         gives 4237 K, 3.1% apart, with our 4200 between them and within 2.1% \
         of the further one. EXCLUDES: anything outside the range the two \
         references bracket.",
    ),
];

/// Where they do NOT agree. These are pinned to the figure measured on the
/// day they were found, not banded wide enough to pass: a band that admits a
/// 90% disagreement is not a check, it is a way of not looking. Both sides
/// are static tables, so the gap is exactly reproducible and a tight pin
/// costs nothing — it fails if either compilation is re-read, in EITHER
/// direction, which is what should happen.
///
/// The finding itself: hydrogen's and nitrogen's temperature coefficients
/// are the two numbers in this table with no corroboration at all, and the
/// USGS files disagree with each other about hydrogen by a factor of 2.2.
/// What would settle each, so the pin is a queue and not a shrug. HYDROGEN:
/// check PHREEQC's default unit for a bare `-delta_h` (the whole size of
/// the gap turns on it), then read Sander 2015's own H2 entry against a
/// primary enthalpy-of-solution measurement. NITROGEN: phreeqc.dat states
/// no enthalpy but DOES carry an `-analytic` expression, which this oracle
/// refuses to differentiate because the file states no validity range for
/// it; establishing that range would give a third, independent value for
/// about an hour's work, and is the cheapest next step in this whole file.
/// Neither is load-bearing today — the gas the bench actually watches
/// dissolve and escape is carbon dioxide — but a lesson that warmed a bottle
/// of soda water and asked about dissolved nitrogen would be resting on an
/// uncorroborated number, and this is where that is written down.
const TEMPERATURE_DISAGREEMENTS: &[(&str, &str, f64, &str)] = &[
    (
        "H2",
        "phreeqc.dat",
        0.2598,
        "phreeqc.dat's -3.3 gives 397 K against Sander's 500 K — but that \
         row is the ONE in the fixture whose delta_h states no unit, so the \
         size of this disagreement rests on PHREEQC's documented kJ default. \
         Read as kcal it would be 1661 K. Verify the default before acting \
         on this row; see tools/gen-henry-oracle.py",
    ),
    (
        "H2",
        "wateq4f.dat",
        0.4351,
        "wateq4f.dat's -1.759 kcal gives 885 K against Sander's 500 K — and \
         2.2 times phreeqc.dat's own answer for the same gas",
    ),
    (
        "N2",
        "wateq4f.dat",
        0.9023,
        "wateq4f.dat's -1.358 kcal gives 683 K against Sander's 1300 K; \
         phreeqc.dat states no enthalpy for nitrogen at all",
    ),
];

fn band(table: &[(&str, f64, &str)], gas: &str) -> Option<f64> {
    table.iter().find(|(g, _, _)| *g == gas).map(|(_, b, _)| *b)
}

/// Henry's constant at 298.15 K, mol/(L·atm), as the shipped table gives it.
fn ours(gas: &str) -> (f64, f64) {
    let c = henry_lookup(gas).unwrap_or_else(|| panic!("{gas} is not in the shipped table"));
    (c.hcp_298, c.c_kelvin)
}

#[test]
fn the_shipped_databases_corroborate_the_solubility_constants() {
    let mut checked = 0;
    for row in rows() {
        let Some(tolerance) = band(SOLUBILITY_BANDS, &row.gas) else {
            panic!(
                "{} has a fixture row from {} but no argued band; add one with \
                 its reason rather than removing the row",
                row.gas, row.database
            );
        };
        let reference = 10f64.powf(row.log_k);
        let (hcp, _) = ours(&row.gas);
        let gap = (hcp - reference).abs() / reference;
        assert!(
            gap < tolerance,
            "{}: we ship {hcp:.6e} mol/(L·atm), {} states {reference:.6e} \
             ({:.2}% apart, band {:.1}%)",
            row.gas,
            row.database,
            100.0 * gap,
            100.0 * tolerance
        );
        checked += 1;
    }
    assert!(checked >= 10, "only {checked} solubility rows reached");
}

#[test]
fn the_temperature_coefficients_agree_where_they_agree() {
    let mut checked = 0;
    for row in rows() {
        let Some(delta_h) = row.delta_h_j else {
            continue;
        };
        if TEMPERATURE_DISAGREEMENTS
            .iter()
            .any(|(g, db, _, _)| *g == row.gas && *db == row.database)
        {
            continue;
        }
        let Some(tolerance) = band(TEMPERATURE_BANDS, &row.gas) else {
            panic!(
                "{} from {} states an enthalpy but is neither banded nor listed \
                 as a disagreement — decide which and say why",
                row.gas, row.database
            );
        };
        let reference = -delta_h / R;
        let (_, c_kelvin) = ours(&row.gas);
        let gap = (c_kelvin - reference).abs() / reference;
        assert!(
            gap < tolerance,
            "{}: we ship −ΔsolH/R = {c_kelvin} K, {} implies {reference:.1} K \
             ({:.2}% apart, band {:.1}%)",
            row.gas,
            row.database,
            100.0 * gap,
            100.0 * tolerance
        );
        checked += 1;
    }
    assert!(checked >= 5, "only {checked} temperature rows reached");
}

#[test]
fn the_two_known_temperature_disagreements_are_exactly_where_they_were() {
    for (gas, database, recorded, note) in TEMPERATURE_DISAGREEMENTS {
        let row = rows()
            .into_iter()
            .find(|r| r.gas == *gas && r.database == *database)
            .unwrap_or_else(|| panic!("the fixture no longer carries {gas} from {database}"));
        let reference = -row
            .delta_h_j
            .expect("a disagreement row states an enthalpy")
            / R;
        let (_, c_kelvin) = ours(gas);
        let gap = (c_kelvin - reference).abs() / reference;
        assert!(
            (gap - recorded).abs() < 0.01,
            "{gas}/{database} was {:.2}% apart when this was recorded and is now \
             {:.2}%. Both sides are static tables, so something was re-read. If \
             it got better, say which side changed and tighten this; if it got \
             worse, that is the finding. Context: {note}",
            100.0 * recorded,
            100.0 * gap
        );
    }
}

/// The reverse check, in the shape `catalog.rs` uses for verbs: every gas the
/// bench ships is either corroborated by a database row or named unreachable
/// with a reason. This is what stops the coverage claim from drifting — a
/// seventh gas cannot arrive with no check and no admission of it.
#[test]
fn every_shipped_gas_is_either_corroborated_or_named_unreachable() {
    let fixture = rows();
    for coefficient in HENRY_COEFFICIENTS {
        let gas = coefficient.formula;
        let corroborated = fixture.iter().any(|r| r.gas == gas);
        let excused = UNREACHABLE.iter().any(|(g, _)| *g == gas);
        assert!(
            corroborated != excused,
            "{gas} is {}: every shipped gas must be corroborated by a vendored \
             database or listed in UNREACHABLE with the reason it cannot be",
            if corroborated {
                "both corroborated and listed unreachable"
            } else {
                "neither corroborated nor listed unreachable"
            }
        );
    }
    for (gas, reason) in UNREACHABLE {
        assert!(
            henry_lookup(gas).is_some(),
            "{gas} is listed unreachable but is no longer shipped; drop the row"
        );
        assert!(
            reason.len() > 120,
            "{gas}'s unreachability needs an argument, not a shrug"
        );
    }
    // The published number for this oracle, recomputed from the rows so it
    // cannot drift: gases whose SOLUBILITY is corroborated, and gases whose
    // TEMPERATURE COEFFICIENT is. Those are different counts and reporting
    // one for the other is how a coverage claim gets inflated.
    let solubility: std::collections::BTreeSet<&str> =
        fixture.iter().map(|r| r.gas.as_str()).collect();
    let temperature: std::collections::BTreeSet<&str> = fixture
        .iter()
        .filter(|r| r.delta_h_j.is_some())
        .filter(|r| {
            !TEMPERATURE_DISAGREEMENTS
                .iter()
                .any(|(g, db, _, _)| *g == r.gas && *db == r.database)
        })
        .map(|r| r.gas.as_str())
        .collect();
    assert_eq!(
        solubility.len(),
        5,
        "five of the six shipped gases have a corroborated solubility: {solubility:?}"
    );
    assert_eq!(
        temperature.len(),
        3,
        "three of the six have a corroborated temperature coefficient: {temperature:?}"
    );
    eprintln!(
        "henry oracle: solubility corroborated for {} of {} gases, temperature \
         coefficient for {}; {} unreachable, {} pinned disagreements",
        solubility.len(),
        HENRY_COEFFICIENTS.len(),
        temperature.len(),
        UNREACHABLE.len(),
        TEMPERATURE_DISAGREEMENTS.len()
    );
}

/// The fixture must not drift from the file it was read out of.
///
/// A generated fixture is a snapshot, and a snapshot of a vendored submodule
/// goes stale silently the day the submodule is bumped — the test would keep
/// passing against numbers the shipped database no longer states, which is
/// exactly the "decorative provenance" failure this work was sent to look
/// for. So when the submodule is present (it is in CI, because
/// `kerotakis-phreeqc` builds against it) every row is re-derived from the
/// database file directly.
///
/// Re-deriving rather than hashing the file, deliberately: a whole-file hash
/// fails on any unrelated edit anywhere in a 6000-line database, which is
/// noise that gets a check switched off. This fails only when the number
/// moves. It also re-checks the generator's parse in a second language.
///
/// Skipped, with a printed note, where the submodule is not checked out. A
/// skip that says so is honest; a skip that is silent is the same defect one
/// level up.
#[test]
fn the_fixture_still_matches_the_vendored_databases() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/iphreeqc/database");
    if !root.is_dir() {
        eprintln!(
            "vendor/iphreeqc is not checked out; the fixture was NOT re-derived. \
             Run `git submodule update --init vendor/iphreeqc` to make this test real."
        );
        return;
    }
    let mut rederived = 0;
    for row in rows() {
        let path = root.join(&row.database);
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        // The databases are Latin-1 (phreeqc.dat carries a degree sign in a
        // comment), so decode byte-wise rather than assuming UTF-8.
        let text: String = bytes.iter().map(|b| *b as char).collect();
        let phase = format!("{}(g)", row.gas);
        let mut in_block = false;
        let mut log_k = None;
        for line in text.lines() {
            let trimmed = line.trim();
            if !line.starts_with([' ', '\t']) && !trimmed.is_empty() {
                if in_block {
                    break;
                }
                in_block = trimmed == phase;
                continue;
            }
            if !in_block {
                continue;
            }
            for part in line.split('#').next().unwrap_or("").split(';') {
                let part = part.trim();
                let lower = part.to_ascii_lowercase();
                let states_log_k = lower.starts_with("log_k") || lower.starts_with("-log_k");
                if states_log_k && log_k.is_none() {
                    log_k = part
                        .split_whitespace()
                        .nth(1)
                        .and_then(|v| v.parse::<f64>().ok());
                }
            }
        }
        let found = log_k.unwrap_or_else(|| {
            panic!(
                "{} no longer states a log_k for {} — the fixture is stale, \
                 regenerate it with tools/gen-henry-oracle.py and re-read every \
                 band in this file",
                row.database, phase
            )
        });
        assert!(
            (found - row.log_k).abs() < 1e-12,
            "{} states log_k {found} for {phase}; the fixture says {}. Regenerate \
             the fixture and re-argue the bands rather than editing one side",
            row.database,
            row.log_k
        );
        rederived += 1;
    }
    assert_eq!(rederived, 12, "every fixture row must be re-derivable");
}

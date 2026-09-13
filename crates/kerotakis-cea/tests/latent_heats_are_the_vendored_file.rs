//! The metals' and salts' enthalpies of fusion are NASA's file, not a
//! handbook retyping.
//!
//! `phase_route::FUSION_ENTHALPIES` is a curated table in `kerotakis-core`,
//! and it used to cite the CRC Handbook of Chemistry and Physics — a
//! commercial reference the provenance audit in PLAN.md puts on the avoid
//! row, alongside the NIST WebBook and JANAF-online. For the nine
//! inorganic rows the replacement was already on disk: NASA CEA's
//! `thermo.inp` is vendored under Apache-2.0, it carries a crystal record
//! and a liquid record for each of these substances, and the enthalpy of
//! fusion is the difference between them at the melting point.
//!
//! That makes the citation checkable rather than merely asserted, which is
//! the point of this file. `kerotakis-core` cannot depend on
//! `kerotakis-cea` — the edge runs the other way — so the numbers are
//! transcribed, and a transcription with nothing watching it is exactly
//! what rotted the rows this replaces. This crate is the only one that can
//! see both sides, so the join lives here, next to
//! `heat_capacity_curves_are_the_vendored_file`.
//!
//! **What this test does and does not prove.** It proves the transcription
//! is faithful — that the twenty-six characters in the table are the ones
//! the vendored file yields. It does not validate the number, and it
//! cannot: re-deriving a value from the database it was taken from is
//! circular, and an oracle that shares its subject proves nothing. The
//! claim the row makes is a provenance claim, and this is the machinery
//! that keeps that claim true, not a second opinion on silver.
//!
//! It is also the reason the nine rows carry a source at all rather than
//! stopping. CEA is a cleared, vendored, Apache-2.0 *data source*, which is
//! the second-choice lane: an individually cited measurement with a DOI, in
//! the shape `literature/hartley-campbell-iodine-water` takes, is the first
//! choice, and none was found for these nine that did not route back
//! through an avoid-row compilation.
//!
//! The organic rows are deliberately absent from `PAIRS`: CEA carries no
//! condensed methanol, acetone, propan-2-ol, hexane, ethyl acetate,
//! acetic acid or naphthalene, so those rows cannot be checked this way
//! and are re-sourced separately. `every_cea_sourced_row_is_listed_here`
//! below is what stops a future row from citing CEA without being checked.

use kerotakis_cea::nasa9;
use kerotakis_core::phase_route::FUSION_ENTHALPIES;

/// Registry species key, CEA crystal record, CEA liquid record.
///
/// The melting point is not written down here on purpose: it is CEA's own
/// interval boundary, read out of the liquid record, so a change to the
/// vendored file moves the temperature and the enthalpy together.
const PAIRS: &[(&str, &str, &str)] = &[
    ("Pb", "Pb(cr)", "Pb(L)"),
    ("Zn", "Zn(cr)", "Zn(L)"),
    ("Mg", "Mg(cr)", "Mg(L)"),
    ("Al", "AL(cr)", "AL(L)"),
    ("Ag", "Ag(cr)", "Ag(L)"),
    ("Cu", "Cu(cr)", "Cu(L)"),
    // Iron melts out of delta, not the alpha the bench starts from: CEA
    // carries alpha, gamma, delta and liquid, and only the last solid
    // touches the melting point.
    ("Fe", "Fe(d)", "Fe(L)"),
    ("NaCl", "NaCL(cr)", "NaCL(L)"),
    ("KCl", "KCL(cr)", "KCL(L)"),
];

/// A row is CEA-sourced when it says so. The provenance string is the
/// claim; this constant is how the test finds the rows making it.
///
/// It is the DERIVATION sentence, not the file path, and the difference
/// matters. Fourteen rows that CEA cannot source mention
/// `vendor/nasa-cea/thermo.inp` in their prose — they record that CEA was
/// checked and why it was rejected, which is exactly the kind of negative
/// result worth keeping next to the value. Matching on the path alone
/// would make writing down a rejection indistinguishable from making a
/// claim, and would have punished the more honest row.
const CEA_CLAIM: &str = "Derived from NASA CEA's `thermo.inp`";

/// The file the claim resolves to, asserted separately so a row cannot
/// make the claim without naming the file it is claiming.
const CEA_PATH: &str = "vendor/nasa-cea/thermo.inp";

#[test]
fn fusion_enthalpies_come_from_the_vendored_cea_tables() {
    let db = nasa9::db();
    for (key, crystal, liquid) in PAIRS {
        let solid = db
            .get(crystal)
            .unwrap_or_else(|| panic!("{crystal} is not in the vendored thermo.inp"));
        let melt = db
            .get(liquid)
            .unwrap_or_else(|| panic!("{liquid} is not in the vendored thermo.inp"));

        // CEA starts the liquid record at the melting point, so the file
        // tells us the temperature; we do not tell the file.
        let (melting_k, _) = melt
            .t_range()
            .unwrap_or_else(|| panic!("{liquid} has no temperature range"));

        let derived = melt
            .h(melting_k)
            .and_then(|hot| Some(hot - solid.h(melting_k)?))
            .unwrap_or_else(|| panic!("{key}: thermo.inp gives no enthalpy at {melting_k} K"));

        let row = FUSION_ENTHALPIES
            .iter()
            .find(|row| row.species == *key)
            .unwrap_or_else(|| panic!("{key} has no enthalpy of fusion to check"));

        // The table ships two decimal places of a kJ/mol, so a 5 J/mol
        // window is the rounding and nothing else. A transposed digit or a
        // handbook value creeping back in is orders of magnitude wider:
        // silver's retired 11.28 would miss by 280.
        let shipped = row.kj_per_mol * 1000.0;
        assert!(
            (derived - shipped).abs() < 5.0,
            "{key}: thermo.inp gives {derived:.1} J/mol of fusion at {melting_k} K, \
             the table ships {shipped:.1}. Either the vendored file moved or the row \
             was edited away from it; the provenance string claims the file",
        );

        assert!(
            row.provenance.contains(CEA_CLAIM) && row.provenance.contains(CEA_PATH),
            "{key} is checked against the vendored CEA file but its provenance does \
             not cite it",
        );
        assert!(
            row.provenance.contains(crystal) && row.provenance.contains(liquid),
            "{key}: the provenance string should name the two records the number \
             comes from, {crystal} and {liquid}",
        );
    }
}

/// The inverse guard. A row may not claim the vendored file unless this
/// test is actually checking it — otherwise "derived from thermo.inp"
/// becomes a sentence anyone can type, which is the failure mode the
/// whole exercise is about.
///
/// Note what it does NOT forbid: mentioning the file. A row that says CEA
/// was checked and rejected is more useful than one that says nothing, and
/// it is the claim that has to be earned, not the noun.
#[test]
fn every_cea_sourced_row_is_listed_here() {
    for row in FUSION_ENTHALPIES {
        if !row.provenance.contains(CEA_CLAIM) {
            continue;
        }
        assert!(
            PAIRS.iter().any(|(key, _, _)| *key == row.species),
            "{} cites the vendored CEA file but is not in PAIRS, so nothing \
             re-derives it",
            row.species
        );
    }
}

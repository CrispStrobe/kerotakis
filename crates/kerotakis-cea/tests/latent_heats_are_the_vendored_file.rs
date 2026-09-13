//! CEA is the bench's independent CROSS-CHECK on the metals and salts, not
//! their source. It was their source, for one day, and why it stopped being
//! one is the reason this file is worth reading.
//!
//! `phase_route::FUSION_ENTHALPIES` cited the CRC Handbook — a commercial
//! reference on PLAN.md's avoid row — for nine inorganic rows. The
//! replacement looked free: NASA CEA's `thermo.inp` is vendored under
//! Apache-2.0, it carries a crystal record and a liquid record for each of
//! these substances, and `H(liquid, Tm) - H(crystal, Tm)` is an enthalpy of
//! fusion. Six of the nine values moved by a fraction of a per cent when
//! that difference was taken, and the derived numbers landed on round
//! figures — 4812, 7300, 8400, 10700, 28200 J/mol — which read as evidence
//! that the derivation was recovering the tables the fitters started from.
//!
//! Silver moved by 2.5 per cent, 11.28 to 11.00, and that was recorded as a
//! disagreement between evaluations. It was not. US Bureau of Mines
//! Bulletin 672 — public domain, and independent of both CEA and the
//! handbook — prints 2.700 kcal/mol at 1235.08 K, which is 11.297, and
//! corroborates it from its own enthalpy-increment column. CEA's `Ag(cr)`
//! record agrees with Bulletin 672 to 0.2 per cent; its `Ag(L)` record sits
//! about 350 J/mol low. CEA's silver records cite Cox 1989, the CODATA Key
//! Values, which does not publish enthalpies of fusion at all — so that
//! citation could never have been the provenance of an 11.00.
//!
//! **A difference of two independently fitted polynomials is not a
//! tabulated transition enthalpy.** Each fit can be excellent over its own
//! interval and their difference at the shared boundary still carries both
//! residuals, because nothing in the fitting constrains them to meet at the
//! evaluated ΔH. `thermo.inp` exists to compute thermodynamic functions,
//! not to tabulate phase changes. The nine rows now cite documents that
//! print the phase change itself.
//!
//! So this file keeps doing the arithmetic and stops calling it provenance.
//! A three per cent band catches a transposed digit or a record swapped for
//! its neighbour — the failure modes a check like this is actually for —
//! while leaving room for the residuals now known to live in the
//! difference. Silver's gap is pinned AS a gap: if a future `thermo.inp`
//! ever closed it, that is news about the file and someone should look,
//! rather than a quiet vindication of a number this bench no longer ships.
//!
//! The organic rows are absent from `PAIRS` because CEA carries no
//! condensed methanol, acetone, propan-2-ol, hexane, ethyl acetate, acetic
//! acid or naphthalene. Nothing on disk reaches them.

use kerotakis_cea::nasa9;
use kerotakis_core::phase_route::FUSION_ENTHALPIES;

/// Registry species key, CEA crystal record, CEA liquid record.
///
/// The melting point is not written down here on purpose: it is CEA's own
/// interval boundary, read out of the liquid record, so a change to the
/// vendored file moves the temperature and the comparison together.
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

/// Silver's known gap between the tabulated value and CEA's difference,
/// J/mol. Pinned rather than tolerated.
const SILVER_GAP_J: std::ops::Range<f64> = 200.0..500.0;

#[test]
fn cea_still_agrees_with_the_metals_and_salts_to_within_its_fit_residuals() {
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
        let shipped = row.kj_per_mol * 1000.0;

        if *key == "Ag" {
            let gap = shipped - derived;
            assert!(
                SILVER_GAP_J.contains(&gap),
                "silver's CEA difference is {derived:.0} J/mol against a shipped \
                 {shipped:.0}, a gap of {gap:.0}. It has been about 300 since the \
                 Ag(L) record's offset was found. If the vendored file has changed, \
                 the nine rows' sourcing deserves another look",
            );
            continue;
        }

        // Three per cent catches a transposed digit or a record swapped for
        // its neighbour. It deliberately does NOT catch a fit residual,
        // because those are known to be in here and are not errors.
        let error = (derived - shipped).abs() / shipped;
        assert!(
            error < 0.03,
            "{key}: thermo.inp differences to {derived:.0} J/mol at {melting_k} K \
             against a shipped {shipped:.0} — {:.1} per cent apart, which is too far \
             to be a fit residual. Either a record was swapped or a digit moved",
            error * 100.0,
        );
    }
}

/// No row may claim `thermo.inp` as its source any more.
///
/// The nine that did for one day now cite Bureau of Mines Bulletin 672 and
/// NSRDS-NBS 37, which print the phase change rather than requiring it to
/// be differenced out of two fits. This guard is what stops the convenient
/// answer coming back: "derived from thermo.inp" is a sentence anyone can
/// type, and for this quantity it would be the wrong one.
///
/// It matches the derivation sentence, not the file path — rows that
/// mention `vendor/nasa-cea/thermo.inp` in order to record that CEA was
/// checked and rejected are exactly the rows this effort wants more of.
#[test]
fn no_fusion_row_claims_to_be_derived_from_the_cea_polynomials() {
    const DERIVED_FROM_CEA: &str = "Derived from NASA CEA's `thermo.inp`";
    for row in FUSION_ENTHALPIES {
        assert!(
            !row.provenance.contains(DERIVED_FROM_CEA),
            "{} says it was derived from the CEA polynomials. A difference of two \
             fitted polynomials at their shared boundary carries both fits' \
             residuals; silver was 2.7 per cent wrong that way. Cite a document \
             that prints the transition enthalpy",
            row.species
        );
    }
}

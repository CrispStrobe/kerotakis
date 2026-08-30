//! A3 / CAP-13: repro cases for the chematic `write_mol` bugs that kept
//! eleven species out of the identity tranche.
//!
//! The tranche gate (`native_identity.rs`) only ever says *whether* a key
//! matches. These tests say *why* it did not, in three named classes, and
//! they are written so that each one fails the moment its class stops being
//! true — a fixed class is a species to promote into `CURATED_STRUCTURES`,
//! not a test to quietly delete.
//!
//! Writing them down that way immediately cost two of the eleven their
//! deferral. Four were one missing V2000 field (class 1, fixed here). A
//! fifth, phenolphthalein, was never a library bug at all — it had been
//! measured against a SMILES for the wrong tautomer. Six remain.
//!
//! Pipeline under test: SMILES → chematic `Molecule` → V2000 molfile
//! (`chematic::mol::write_mol`) → official IUPAC InChI library. Only the
//! middle step is suspect; the SMILES parse succeeds and InChI is the
//! authority.

#![cfg(feature = "native-inchi")]

use chematic::mol::{write_mol, MolMetadata};

fn molfile(smiles: &str) -> String {
    let mol = chematic::smiles::parse(smiles).unwrap_or_else(|e| panic!("{smiles}: {e}"));
    write_mol(&mol, &MolMetadata::default())
}

fn inchi_of(molfile: &str) -> (String, String) {
    let out = inchi::from_molfile(molfile, ()).expect("official InChI accepts the molfile");
    let s = out.inchi().to_string();
    let k = inchi::inchikey(&s).expect("InChIKey");
    (s, k)
}

fn key_of(smiles: &str) -> String {
    inchi_of(&molfile(smiles)).1
}

/// Like [`key_of`], but reports a failure anywhere in the pipeline instead
/// of panicking. The still-deferred classes use this: a species that cannot
/// be keyed at all is every bit as deferred as one keyed wrongly, and which
/// of the two it is today is not something those tests should be pinned to.
fn try_key_of(smiles: &str) -> Result<String, String> {
    let mol = chematic::smiles::parse(smiles).map_err(|e| format!("SMILES parse: {e}"))?;
    let written = write_mol(&mol, &MolMetadata::default());
    let out = inchi::from_molfile(&written, ()).map_err(|e| format!("official InChI: {e}"))?;
    let s = out.inchi().to_string();
    inchi::inchikey(&s).map_err(|e| format!("InChIKey: {e}"))
}

/// Blank the V2000 atom block's valence field (`vvv`, columns 49–51,
/// 1-based) on every atom line — i.e. reproduce byte-for-byte what
/// **unpatched** chematic-mol 0.18.0 wrote, so the diagnosis can be
/// demonstrated and not merely asserted.
fn as_unpatched_chematic_wrote_it(molfile: &str) -> String {
    let mut out = String::new();
    let mut natoms = 0usize;
    let mut atom_i = 0usize;
    for (i, line) in molfile.lines().enumerate() {
        if i == 3 {
            natoms = line[0..3].trim().parse().expect("counts line");
        }
        if (4..4 + natoms).contains(&i) && line.len() >= 51 {
            let mut bytes = line.as_bytes().to_vec();
            bytes[48..51].copy_from_slice(b"  0");
            out.push_str(std::str::from_utf8(&bytes).expect("ASCII molfile"));
            out.push('\n');
            atom_i += 1;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    assert_eq!(atom_i, natoms, "every atom line was rewritten");
    out
}

// ---------------------------------------------------------------------------
// Class 1 — implicit H in the V2000 writer. FIXED, by the one local patch
// inside the vendored chematic tree (vendor/chematic-0.18/…/mol2000.rs).
// ---------------------------------------------------------------------------

/// The bug, stated as the difference one V2000 field makes.
///
/// A bracket SMILES atom carries an explicit hydrogen count, and for a bare
/// element that count is *zero*. V2000 can only say so through the atom
/// block's valence field (`vvv`): `0` there means "unspecified", and every
/// standard-valence reader — the IUPAC InChI library included — then fills
/// the atom back up with hydrogens. chematic-mol 0.18.0 never wrote the
/// field, so four bare elements reached InChI as entirely different
/// substances.
#[test]
fn class1_the_valence_field_is_the_whole_bug() {
    let cases = [
        ("[Mg]", "InChI=1S/Mg.2H", "InChI=1S/Mg"),
        ("[Pb]", "InChI=1S/Pb.2H", "InChI=1S/Pb"),
        // Not a typo: elemental carbon was reaching InChI as methane, and
        // elemental sulfur as hydrogen sulfide.
        ("[C]", "InChI=1S/CH4/h1H4", "InChI=1S/C"),
        ("[S]", "InChI=1S/H2S/h1H2", "InChI=1S/S"),
    ];
    for (smiles, was, now) in cases {
        let written = molfile(smiles);
        let (broken, _) = inchi_of(&as_unpatched_chematic_wrote_it(&written));
        assert_eq!(
            broken, was,
            "{smiles}: the pre-patch molfile should still reproduce the bug"
        );
        let (fixed, _) = inchi_of(&written);
        assert_eq!(fixed, now, "{smiles}: the patched writer must encode it");
    }
}

/// The four species the fix unlocks, checked end to end against the keys
/// the registry curates. `native_identity.rs` gates these too, now that
/// they are in `CURATED_STRUCTURES`; here they are named as the bug's
/// blast radius so the connection survives.
#[test]
fn class1_unlocks_four_registry_species() {
    let unlocked = [
        ("Mg", "[Mg]", "FYYHWMGAXLPEAU-UHFFFAOYSA-N"),
        ("Pb", "[Pb]", "WABPQHHGFIMREM-UHFFFAOYSA-N"),
        ("C", "[C]", "OKTJSMMVPCPJKN-UHFFFAOYSA-N"),
        ("S", "[S]", "NINIDFKCEFEMDL-UHFFFAOYSA-N"),
    ];
    for (id, smiles, expected) in unlocked {
        assert_eq!(key_of(smiles), expected, "{id} ({smiles})");
    }
}

/// The fix is narrow on purpose: an atom with no explicit hydrogen count
/// still leaves the valence field unspecified, so molecules made only of
/// organic-subset atoms are written exactly as before.
#[test]
fn class1_leaves_organic_subset_molecules_byte_identical() {
    for smiles in [
        "CCO",
        "CC(=O)O",
        "O=C=O",
        "CCCCCC",
        "OCC(O)C1OC(=O)C(O)=C1O",
    ] {
        let written = molfile(smiles);
        assert_eq!(
            written,
            as_unpatched_chematic_wrote_it(&written),
            "{smiles} must be untouched by the valence-field patch"
        );
    }
}

// ---------------------------------------------------------------------------
// Class 2 — kekulisation. TWO OF THREE STILL DEFERRED; the third was never
// a library bug at all.
// ---------------------------------------------------------------------------

/// Phenolphthalein, the deferral that dissolved on contact with a repro.
///
/// It was filed as a `write_mol` kekulisation bug ("outputs the open (acid)
/// form"). It is not one: given the closed lactone, chematic kekulises it,
/// writes it, and the official library reproduces the registry's key
/// exactly. The writer had been faithfully encoding the molecule it was
/// handed — the open acid form — and the wrong tautomer went in.
///
/// Kept as a standing test rather than folded into the tranche gate alone,
/// because the interesting claim is not "this key matches" but "this class
/// was misdiagnosed, and a chematic bug was blamed for a curation
/// mistake".
#[test]
fn class2_phenolphthalein_was_never_a_kekulisation_bug() {
    assert_eq!(
        key_of("Oc1ccc(cc1)C1(OC(=O)c2ccccc21)c1ccc(O)cc1"),
        "KJFMBFZCATUALV-UHFFFAOYSA-N",
        "the closed lactone keys to the registry's phenolphthalein"
    );
}

/// Two of the three indicator dyes. Each parses and writes a molfile the
/// official library accepts, and each one keys to something other than the
/// dye.
///
/// **Phenolphthalein is no longer here**, and that is the finding: it was
/// deferred as a kekulisation failure and is not one. chematic kekulises
/// the closed lactone perfectly — the original deferral was measured
/// against a SMILES for the *open* (acid) form, so `write_mol` faithfully
/// encoded the wrong tautomer and the mismatch was read as a library bug.
/// Curating the lactone was the entire fix; it is in `CURATED_STRUCTURES`
/// and gated by `native_identity.rs` now. Which leaves the open question
/// for the remaining two honest: it may equally be curation rather than
/// kekulisation, and nobody has checked.
///
/// Asserted as an *inequality* deliberately. When these two start matching
/// — from a library fix or from a better curated SMILES — this test fails,
/// and the failure is the instruction: promote the species into
/// `CURATED_STRUCTURES` and delete the case.
#[test]
fn class2_kekulisation_still_wrong() {
    let deferred = [
        (
            "methyl_orange",
            "CN(C)c1ccc(cc1)N=Nc1ccc(cc1)S(=O)(=O)[O-].[Na+]",
            "BSKHPKMHTQYZBB-UHFFFAOYSA-N",
        ),
        (
            "bromothymol_blue",
            "CC(C)c1cc(Br)c(O)c(C)c1C1(OS(=O)(=O)c2ccccc21)c1cc(C(C)C)c(O)c(Br)c1C",
            "FBSFWRHWHYMIOG-UHFFFAOYSA-N",
        ),
    ];
    for (id, smiles, registry_key) in deferred {
        if let Ok(got) = try_key_of(smiles) {
            assert_ne!(
                got, registry_key,
                "{id} now matches — the kekulisation class is fixed. Promote \
                 it into CURATED_STRUCTURES (and grow the tranche pin), then \
                 drop this case."
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Class 3 — connectivity and charge. STILL DEFERRED.
// ---------------------------------------------------------------------------

/// Four individual bugs, not one class with one fix — grouped only because
/// each is about connectivity or charge surviving the molfile:
///
/// - `Cu(OH)2`: the Cu–O bonds are written but InChI reads back a
///   disconnected `Cu.2H2O` with a `/q+2/p-2` charge balance, not connected
///   copper(II) hydroxide. Every ionic spelling (`[Cu+2].[OH-].[OH-]`)
///   lands on the same wrong key.
/// - `MnO4-`: Mn–O connectivity is lost outright — InChI sees `Mn.4O`.
/// - `Pb+2`: the charge survives, the connectivity hash does not.
/// - `Pb(NO3)2`: the connectivity hash matches; the charge layer does not
///   (the net −2 proton balance is lost across the fragments).
///
/// Same inequality shape and same instruction as class 2.
#[test]
fn class3_connectivity_and_charge_still_wrong() {
    let deferred = [
        ("Cu(OH)2", "O[Cu]O", "PTTPXKJBFFKCEK-UHFFFAOYSA-N"),
        (
            "Cu(OH)2 (ionic)",
            "[Cu+2].[OH-].[OH-]",
            "PTTPXKJBFFKCEK-UHFFFAOYSA-N",
        ),
        ("MnO4-", "[O-][Mn](=O)(=O)=O", "VLTRZXGMWDSKGL-UHFFFAOYSA-M"),
        ("Pb+2", "[Pb+2]", "XMOCLSLCDHWDHP-UHFFFAOYSA-N"),
        (
            "Pb(NO3)2",
            "[Pb+2].[O-][N+](=O)[O-].[O-][N+](=O)[O-]",
            "RLJMLMKIBZAXJO-UHFFFAOYSA-L",
        ),
    ];
    for (id, smiles, registry_key) in deferred {
        if let Ok(got) = try_key_of(smiles) {
            assert_ne!(
                got, registry_key,
                "{id} now matches — promote it into CURATED_STRUCTURES (and \
                 grow the tranche pin), then drop this case."
            );
        }
    }
}

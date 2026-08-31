//! CAP-13 spike: the identity layers the molfile bridge could not carry.
//!
//! BRD-010's two-route check found 23 of a 100-record PubChem fixture
//! disagreeing with PubChem's own Standard InChIKey, and every one of them
//! carried the `UHFFFAOYSA` signature — the second block that says "no
//! stereo, no isotope". PubChem's own InChI re-keyed 100/100 clean on the
//! same records, so the loss was ours: a coordinate-less V2000 molfile has
//! nowhere to put a stereocentre and chematic's writer emitted no isotope
//! at all, which collapsed every enantiomer pair onto one key and every
//! isotopologue onto its natural-abundance parent.
//!
//! This is that class as a standing corpus. The SMILES and the expected
//! keys are PubChem's own, taken from the pinned snapshot BRD-010 checked
//! in (`kerotakis-data/tests/fixtures/quarantine/pubchem-v1`), so the pairs
//! that must stay *distinct* — L- and D-alanine, the three tartaric acids,
//! the two limonenes, the glucose anomers — are distinguished by the
//! authority, not by us.

#![cfg(feature = "native-inchi")]

use kerotakis_org::inchi_validate::{
    molfile_route_inchikey_from_smiles, native_inchikey_from_smiles,
};

/// `(label, SMILES, PubChem Standard InChIKey)`.
const STEREO_AND_ISOTOPE: &[(&str, &str, &str)] = &[
    // --- tetrahedral: enantiomers that must not collide ---
    (
        "L-alanine",
        "C[C@@H](C(=O)O)N",
        "QNAYBMKLOCPYGJ-REOHCLBHSA-N",
    ),
    (
        "D-alanine",
        "C[C@H](C(=O)O)N",
        "QNAYBMKLOCPYGJ-UWTATZPHSA-N",
    ),
    (
        "L-lactic acid",
        "C[C@@H](C(=O)O)O",
        "JVTAAEKCZFNVCJ-REOHCLBHSA-N",
    ),
    (
        "D-lactic acid",
        "C[C@H](C(=O)O)O",
        "JVTAAEKCZFNVCJ-UWTATZPHSA-N",
    ),
    (
        "L-tartaric acid",
        "[C@@H]([C@H](C(=O)O)O)(C(=O)O)O",
        "FEWJPZIEWOKRBE-JCYAYHJZSA-N",
    ),
    (
        "D-tartaric acid",
        "[C@H]([C@@H](C(=O)O)O)(C(=O)O)O",
        "FEWJPZIEWOKRBE-LWMBPPNESA-N",
    ),
    (
        "meso-tartaric acid",
        "[C@@H]([C@@H](C(=O)O)O)(C(=O)O)O",
        "FEWJPZIEWOKRBE-XIXRPRMCSA-N",
    ),
    (
        "(-)-limonene",
        "CC1=CC[C@H](CC1)C(=C)C",
        "XMGQYMWWDOXHJM-SNVBAGLBSA-N",
    ),
    (
        "(+)-limonene",
        "CC1=CC[C@@H](CC1)C(=C)C",
        "XMGQYMWWDOXHJM-JTQLQIEISA-N",
    ),
    // --- ring stereocentres: the sugar anomers ---
    (
        "alpha-D-glucose",
        "C([C@@H]1[C@H]([C@@H]([C@H]([C@H](O1)O)O)O)O)O",
        "WQZGKKKJIJFFOK-DVKNGEFBSA-N",
    ),
    (
        "beta-D-glucose",
        "C([C@@H]1[C@H]([C@@H]([C@H]([C@@H](O1)O)O)O)O)O",
        "WQZGKKKJIJFFOK-VFUOTHLCSA-N",
    ),
    (
        "L-glucose",
        "C([C@H]1[C@@H]([C@H]([C@@H](C(O1)O)O)O)O)O",
        "WQZGKKKJIJFFOK-ZZWDRFIYSA-N",
    ),
    // --- double-bond geometry ---
    ("trans-2-butene", "C/C=C/C", "IAQRGUVFOMOMEM-ONEGZZNKSA-N"),
    // --- isotopes ---
    ("deuterium", "[2H][2H]", "UFHFLCQGNIYNRP-VVKOMZTBSA-N"),
    ("tritium", "[3H][3H]", "UFHFLCQGNIYNRP-JMRXTUGHSA-N"),
    ("heavy water", "[2H]O[2H]", "XLYOFNOQVPJJNP-ZSJDYOACSA-N"),
    (
        "methanol-d4",
        "[2H]C([2H])([2H])O[2H]",
        "OKKJLVBELUTLKV-MZCSYVLQSA-N",
    ),
    (
        "chloroform-d",
        "[2H]C(Cl)(Cl)Cl",
        "HEDRZPFGACZZDS-MICDWDOJSA-N",
    ),
    (
        "benzene-d6",
        "[2H]C1=C(C(=C(C(=C1[2H])[2H])[2H])[2H])[2H]",
        "UHOVQNZJYSORNB-MZWXYZOWSA-N",
    ),
];

#[test]
fn stereo_and_isotope_layers_survive_the_bridge() {
    let mut failures = Vec::new();
    for (label, smiles, expected) in STEREO_AND_ISOTOPE {
        match native_inchikey_from_smiles(smiles) {
            Ok(key) if key == *expected => {}
            Ok(key) => failures.push(format!(
                "{label}: PubChem says {expected}, we compute {key}"
            )),
            Err(e) => failures.push(format!("{label}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "the stereo/isotope layer is not surviving the bridge:\n{}",
        failures.join("\n")
    );
}

/// The evidence for why the bridge stopped going through a molfile: on this
/// same corpus the old route agrees with almost nothing, and what it does
/// produce is the stereo-free, isotope-free key of a *different* substance.
#[test]
fn the_molfile_route_is_the_one_that_loses_them() {
    let lost: Vec<&str> = STEREO_AND_ISOTOPE
        .iter()
        .filter(|(_, smiles, expected)| {
            !matches!(molfile_route_inchikey_from_smiles(smiles), Ok(key) if key == **expected)
        })
        .map(|(label, _, _)| *label)
        .collect();
    assert_eq!(
        lost.len(),
        STEREO_AND_ISOTOPE.len(),
        "the molfile route was expected to lose every one of these; it kept: {:?}",
        STEREO_AND_ISOTOPE
            .iter()
            .map(|(l, _, _)| *l)
            .filter(|l| !lost.contains(l))
            .collect::<Vec<_>>()
    );
}

/// Enantiomers are the point: a bridge that drops stereo makes them equal,
/// and a registry that keys on identity would then merge them.
#[test]
fn enantiomers_do_not_collide() {
    for (a, b) in [
        ("C[C@@H](C(=O)O)N", "C[C@H](C(=O)O)N"),
        ("CC1=CC[C@H](CC1)C(=C)C", "CC1=CC[C@@H](CC1)C(=C)C"),
    ] {
        let ka = native_inchikey_from_smiles(a).expect("key");
        let kb = native_inchikey_from_smiles(b).expect("key");
        assert_ne!(ka, kb, "{a} and {b} came out with the same identity");
    }
}

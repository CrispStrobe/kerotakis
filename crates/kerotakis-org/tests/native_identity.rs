//! CAP-13's contract: the registry's curated InChIKeys are recomputed
//! by the official IUPAC InChI library from a curated structure and
//! must match — a mismatch is a curation bug and fails the build.
//!
//! First tranche: every species with a hand-pinned SMILES below. The
//! count is pinned so the tranche can only grow deliberately; species
//! without a structural identity (minerals by formula unit, enzymes,
//! indicator dyes awaiting kekulisation care) join as their SMILES are
//! curated.

#![cfg(feature = "native-inchi")]

use kerotakis_core::species::{self, SpeciesId};
use kerotakis_org::inchi_validate::native_inchikey_from_smiles;

use kerotakis_org::inchi_validate::CURATED_STRUCTURES;

#[test]
fn registry_inchikeys_are_recomputed_and_match() {
    let mut failures = Vec::new();
    for (id, smiles) in CURATED_STRUCTURES {
        let expected = species::lookup(&SpeciesId::new(id))
            .unwrap_or_else(|| panic!("{id} must be in the registry"))
            .inchikey;
        match native_inchikey_from_smiles(smiles) {
            Ok(key) if key == expected => {}
            Ok(key) => failures.push(format!(
                "{id}: registry says {expected}, official InChI computes {key} from {smiles}"
            )),
            Err(e) => failures.push(format!("{id}: could not compute from {smiles}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "curation bugs — the registry key is not the identity of the \
         curated structure:\n{}",
        failures.join("\n")
    );
}

#[test]
fn the_tranche_only_grows_deliberately() {
    assert_eq!(
        CURATED_STRUCTURES.len(),
        // EXP-50 adds bromide/HBr/NaBr plus five selectivity substrates and
        // products; EXP-30 adds the four qualitative-analysis hydroxides;
        // BRD-012.S02 adds the three P0 school salts (NH4Cl, FeCl3,
        // Na2SO4) with the ammonium ion the databases book them against,
        // and the gated barium tranche (BaCl2, Ba(OH)2, Ba+2, BaSO4). The
        // CAP-13 spike adds the four bare bracket atoms the molfile bridge
        // could not spell (Mg, Pb, C, S) and phenolphthalein, whose
        // deferral was a wrong curated SMILES rather than a kekulisation
        // problem — see provenance/cap-13-chematic-molfile-spike.md.
        102,
        "structures were added or removed — update this pin and say why \
         in the same commit"
    );
}

/// The aluminium correction, shown rather than asserted. The key the
/// registry carried until 2026-08-30 is the one the *hydride* has — which
/// is why nothing caught it: the identity gate reached the InChI library
/// through a V2000 molfile that could not say a bracket atom has no
/// hydrogens, so it recomputed AlH3 from `[Al]` and the two wrong answers
/// agreed with each other.
#[test]
fn the_aluminium_key_the_registry_used_to_carry_is_alumane() {
    assert_eq!(
        native_inchikey_from_smiles("[AlH3]").expect("alumane"),
        "AZDRQVAHHNSJOQ-UHFFFAOYSA-N",
        "the retired key is alumane's"
    );
    assert_eq!(
        native_inchikey_from_smiles("[Al]").expect("aluminium"),
        "XAGFODPZIPBFFR-UHFFFAOYSA-N",
        "the metal's own key (PubChem CID 5359268)"
    );
}

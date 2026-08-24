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
        76,
        "structures were added or removed — update this pin and say why \
         in the same commit"
    );
}

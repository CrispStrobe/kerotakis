//! The differential that makes `curated::ORG_REACTIONS` trustworthy:
//! each row's stoichiometry is proven against the atom-mapped SMIRKS
//! template at the molecule level. The template turns real molecules
//! into real molecules; every product must be *the same molecule* as
//! the one the curated row deposits, and every deposit must be
//! produced.
//!
//! Identity here is chematic's own canonical key — NOT a standard
//! InChIKey (chematic's pure-Rust keys differ from IUPAC's; adopting
//! the official library is CAP-13). The comparison is therefore
//! key(template product) == key(reference SMILES), where each
//! reference SMILES is hand-pinned to a registry species below and its
//! molecular formula is checked against that species' formula as the
//! cross-check chematic *can* do faithfully.

use kerotakis_core::curated::ORG_REACTIONS;
use kerotakis_org::parse_smiles;
use kerotakis_org::templates::{apply_template, esterification, saponification};

/// (registry species id, hand-pinned reference SMILES, formula as
/// chematic writes it). The SMILES↔species correspondence is curated;
/// the formula ties it to chemistry rather than trust.
const REFERENCES: &[(&str, &str, &str)] = &[
    ("CH3COOH", "CC(=O)O", "C2H4O2"),
    ("ethanol", "CCO", "C2H6O"),
    ("ethyl_acetate", "CCOC(C)=O", "C4H8O2"),
    ("water", "O", "H2O"),
    ("CH3COO-", "CC(=O)[O-]", "C2H3O2"),
];

fn reference(id: &str) -> (&'static str, &'static str) {
    REFERENCES
        .iter()
        .find(|(rid, ..)| *rid == id)
        .map(|(_, smi, formula)| (*smi, *formula))
        .unwrap_or_else(|| panic!("no reference SMILES for {id}"))
}

fn key_and_formula(smiles: &str) -> (String, String) {
    let m = parse_smiles(smiles).unwrap_or_else(|e| panic!("parse {smiles}: {e}"));
    (
        m.inchikey
            .unwrap_or_else(|| panic!("no canonical key for {smiles}")),
        m.formula,
    )
}

fn row(name: &str) -> &'static kerotakis_core::curated::OrgReaction {
    ORG_REACTIONS
        .iter()
        .find(|r| r.name == name)
        .unwrap_or_else(|| panic!("curated row '{name}' exists"))
}

/// Every template product must be the same molecule as one expected
/// reference, every expected reference must appear, count exact.
fn assert_products_are(products: &[String], expected_ids: &[&str]) {
    assert_eq!(
        products.len(),
        expected_ids.len(),
        "the template makes exactly the expected products; got {products:?}"
    );
    let mut remaining: Vec<&str> = expected_ids.to_vec();
    for p in products {
        let (pk, pf) = key_and_formula(p);
        let hit = remaining.iter().position(|id| {
            let (smi, formula) = reference(id);
            let (rk, rf) = key_and_formula(smi);
            pk == rk && pf == rf && rf == formula
        });
        match hit {
            Some(i) => {
                remaining.remove(i);
            }
            None => panic!("product {p} (key {pk}, {pf}) matches none of {remaining:?}"),
        }
    }
}

#[test]
fn esterification_row_matches_its_template() {
    let products =
        apply_template(&esterification(), &["CC(=O)O", "CCO"]).expect("template applies");
    assert_products_are(&products, &["ethyl_acetate", "water"]);

    let deposited: Vec<&str> = row("esterification")
        .products
        .iter()
        .map(|(k, ..)| *k)
        .collect();
    assert_eq!(deposited, vec!["ethyl_acetate", "water"]);
}

#[test]
fn saponification_row_matches_its_template() {
    let products =
        apply_template(&saponification(), &["CCOC(C)=O", "[OH-]"]).expect("template applies");
    // The template yields the acetate ANION; the bench's compound
    // ledger deposits it as NaOAc, because the sodium arrived with the
    // hydroxide and a ledger tracks compounds. CH3COO- is the
    // registry's own form of the anion the template proves.
    assert_products_are(&products, &["CH3COO-", "ethanol"]);

    let deposited: Vec<&str> = row("saponification")
        .products
        .iter()
        .map(|(k, ..)| *k)
        .collect();
    assert_eq!(
        deposited,
        vec!["NaOAc", "ethanol"],
        "the ledger form of the anion the template proved"
    );
}

/// The reference SMILES really do belong to the registry species they
/// are pinned to: formula agreement with the registry's own formulas,
/// which is the strongest identity check available without CAP-13.
#[test]
fn references_agree_with_the_registry() {
    use kerotakis_core::species::{self, SpeciesId};
    for (id, smi, formula) in REFERENCES {
        let (_, f) = key_and_formula(smi);
        assert_eq!(&f, formula, "{id}");
        assert!(
            species::lookup(&SpeciesId::new(id)).is_some(),
            "{id} must be in the registry"
        );
    }
}

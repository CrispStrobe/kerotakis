//! BRD-020: the chematic-backed implementation of the reaction-family
//! IR's structural seam.
//!
//! `kerotakis_core::family` owns the records, gates, conflict order and
//! explanations; it deliberately knows nothing about SMILES. This module
//! supplies the [`StructureOracle`] those records are matched through —
//! today over `chematic`, and replaced wholesale by BRD-022's selected
//! engine without a record changing.
//!
//! Two honesty rules are load-bearing here:
//!
//! * A species with no curated structure does not silently pass a group
//!   gate — `groups_of` returns `None` and the family DECLINES with the
//!   reason, because "we cannot see its groups" is not "it has none".
//! * A product the registry cannot name is an error, never a quiet drop:
//!   the pool of the nameable is the boundary, exactly as in the thermal
//!   solver's species pool.

use kerotakis_core::family::{FamilyRecord, StructureOracle};

use crate::inchi_validate::CURATED_STRUCTURES;
use crate::{groups, templates};

/// The registry-anchored structural oracle: structures come only from
/// `CURATED_STRUCTURES` (the same table the official-InChI identity gate
/// pins), so a family can never fire on a structure nobody curated.
pub struct ChematicOracle;

fn smiles_of(species_key: &str) -> Option<&'static str> {
    CURATED_STRUCTURES
        .iter()
        .find(|(key, _)| *key == species_key)
        .map(|(_, smiles)| *smiles)
}

/// chematic's own canonical key for a SMILES string: its InChI, keyed.
/// Not the official InChIKey (that is CAP-13's authority) — here the
/// same algorithm is compared with itself, which is all identity
/// matching needs.
fn chematic_key(smiles: &str) -> Option<String> {
    let m = chematic::smiles::parse(smiles).ok()?;
    let inchi_str = chematic::inchi::inchi(&m);
    (!inchi_str.is_empty()).then(|| chematic::inchi::inchi_key(&inchi_str))
}

/// The registry key whose curated structure canonicalises identically to
/// `smiles`, if any — one algorithm, compared with itself.
fn key_of_product(smiles: &str) -> Option<&'static str> {
    let want = chematic_key(smiles)?;
    CURATED_STRUCTURES
        .iter()
        .find_map(|(key, s)| (chematic_key(s).as_deref() == Some(&*want)).then_some(*key))
}

impl StructureOracle for ChematicOracle {
    fn groups_of(&self, species_key: &str) -> Option<Vec<String>> {
        let smiles = smiles_of(species_key)?;
        Some(
            groups::perceive_groups(smiles)
                .into_iter()
                .map(|g| g.name.to_string())
                .collect(),
        )
    }

    fn apply(
        &self,
        record: &FamilyRecord,
        substrate_keys: &[&str],
    ) -> Result<Option<Vec<String>>, String> {
        // Every substrate needs a curated structure; a family asked about
        // a structureless species has not matched — it was never asked.
        let mut substrate_smiles = Vec::with_capacity(substrate_keys.len());
        for key in substrate_keys {
            match smiles_of(key) {
                Some(s) => substrate_smiles.push(s),
                None => return Ok(None),
            }
        }
        let template = templates::ReactionTemplate {
            name: record.id.clone(),
            family: record.id.clone(),
            smirks: record.smirks.clone(),
            source: record.provenance.clone(),
            validated: true,
        };
        let products = match templates::apply_template(&template, &substrate_smiles) {
            Ok(p) if p.is_empty() => return Ok(None),
            Ok(p) => p,
            // A malformed pattern is a record bug, surfaced; a pattern
            // that simply does not match reports "not asked" above.
            Err(e) if e.contains("failed") => return Ok(None),
            Err(e) => return Err(e),
        };
        let mut product_keys = Vec::with_capacity(products.len());
        for p in &products {
            match key_of_product(p) {
                Some(key) => product_keys.push(key.to_string()),
                None => {
                    return Err(format!(
                        "family {} produced a structure the registry cannot name ({p}); \
                         the pool of the nameable is the boundary, and widening it is a \
                         registry task, not a silent drop",
                        record.id
                    ))
                }
            }
        }
        Ok(Some(product_keys))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kerotakis_core::family::{FamilyConfidence, GateSet, MediumGate, OutcomeModel};

    /// The esterification family, expressed as a BRD-020 record over the
    /// SAME SMIRKS the landed `react` verb uses (ORG-006). The parity
    /// claim this file exists for: the IR route and the legacy template
    /// route name identical products for identical substrates.
    fn esterification_record() -> FamilyRecord {
        let legacy = templates::esterification();
        FamilyRecord {
            id: "esterification".into(),
            version: 1,
            smirks: legacy.smirks,
            substrates: vec!["CH3COOH".into(), "ethanol".into()],
            ledger_reactants: Default::default(),
            ledger_products: Default::default(),
            gates: GateSet {
                medium: Some(MediumGate::OrganicSolvent {
                    solvent: "ethanol".into(),
                }),
                required_groups: vec!["carboxylic acid".into(), "hydroxyl".into()],
                ..GateSet::default()
            },
            priority: 0,
            outcome: OutcomeModel::Equilibrium {
                log_k: 0.6,
                source: legacy.source,
            },
            confidence: FamilyConfidence::CuratedFamily,
            provenance: "Fischer esterification (March), via ORG-006".into(),
            refusal_domain: "claims esters of simple alcohols and carboxylic acids only; \
                             no claim under aqueous excess, and no rate claim"
                .into(),
        }
    }

    #[test]
    fn the_ir_route_matches_the_legacy_template_route() {
        let record = esterification_record();
        let oracle = ChematicOracle;
        let via_ir = oracle
            .apply(&record, &["CH3COOH", "ethanol"])
            .expect("oracle answers")
            .expect("the family matches its own exemplar substrates");
        // The legacy route, exactly as tests/template_oracle.rs drives it.
        let legacy = templates::apply_template(&templates::esterification(), &["CC(=O)O", "CCO"])
            .expect("legacy template applies");
        let legacy_keys: Vec<String> = legacy
            .iter()
            .filter_map(|p| key_of_product(p).map(str::to_string))
            .collect();
        assert_eq!(
            via_ir, legacy_keys,
            "one SMIRKS, two routes, one answer — or the IR is a fork, not a home"
        );
        assert!(
            via_ir.iter().any(|k| k == "ethyl_acetate"),
            "the ester is named: {via_ir:?}"
        );
    }

    #[test]
    fn a_structureless_species_is_not_asked() {
        let record = esterification_record();
        let oracle = ChematicOracle;
        // Hematite has no entry in CURATED_STRUCTURES; the family reports
        // no match rather than pretending to have perceived anything.
        assert_eq!(oracle.apply(&record, &["Fe2O3", "ethanol"]).unwrap(), None);
        assert_eq!(oracle.groups_of("Fe2O3"), None);
    }

    #[test]
    fn group_perception_feeds_the_gate_vocabulary() {
        let oracle = ChematicOracle;
        let acid_groups = oracle.groups_of("CH3COOH").expect("curated structure");
        assert!(
            acid_groups.iter().any(|g| g.contains("carboxylic")),
            "acetic acid carries its acid group: {acid_groups:?}"
        );
    }
}

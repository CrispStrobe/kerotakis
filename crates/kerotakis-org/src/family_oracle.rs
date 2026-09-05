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

use kerotakis_core::family::{FamilyRecord, FamilyRouter, StructureOracle};

use crate::inchi_validate::CURATED_STRUCTURES;
use crate::{groups, templates};

/// The shipped family pack (BRD-020 → BRD-023 v1), linted at load.
pub const FAMILY_PACK_V1: &str = include_str!("../../../data/families/families-v1.toml");

/// The pack's records, or the lint's complaint. A shipped pack that does
/// not lint is a build defect; `family_equilibrator` treats it as one.
pub fn family_records() -> Result<Vec<FamilyRecord>, String> {
    kerotakis_core::family::load_records(FAMILY_PACK_V1)
}

/// The reaction-family solver the standard stack carries: the shipped
/// records over the chematic-backed oracle.
pub fn family_equilibrator() -> FamilyRouter<ChematicOracle> {
    FamilyRouter::new(
        ChematicOracle,
        family_records().expect("the shipped family pack lints clean"),
    )
}

/// The registry-anchored structural oracle: structures come only from
/// `CURATED_STRUCTURES` (the same table the official-InChI identity gate
/// pins), so a family can never fire on a structure nobody curated.
pub struct ChematicOracle;

/// Structures for the router's charge-backed keys (`family::FREE_HYDROXIDE`),
/// which the aqueous tail carries as `solute_charge` rather than as a
/// portion. Kept apart from `CURATED_STRUCTURES` because that table is the
/// official-InChI identity gate's, and a bare ion is not a registry
/// identity the gate can pin.
const VIRTUAL_STRUCTURES: &[(&str, &str)] = &[(kerotakis_core::family::FREE_HYDROXIDE, "[OH-]")];

fn smiles_of(species_key: &str) -> Option<&'static str> {
    CURATED_STRUCTURES
        .iter()
        .chain(VIRTUAL_STRUCTURES.iter())
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
    // A curated SMILES written exactly as the toolkit wrote the product
    // is the same structure by construction — and it is the only route
    // for a bare ion like `[Na+]`, whose InChI the toolkit may not form.
    if let Some((key, _)) = CURATED_STRUCTURES
        .iter()
        .chain(VIRTUAL_STRUCTURES.iter())
        .find(|(_, s)| *s == smiles)
    {
        return Some(key);
    }
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
        // Whole molecules first.
        let whole = run(&template, &substrate_smiles);
        if let Ok(Some(products)) = &whole {
            return Ok(Some(name_all(&record.id, products, &[])?));
        }
        // A salt arrives as ONE registry species and two fragments — NaOH
        // is `[Na+].[OH-]` — while the pattern names only the fragment
        // that reacts. Matched whole, the toolkit drops the spectator and
        // the conservation ledger refuses the product set (Na: 1 in, 0
        // out), which is the ledger doing its job, not the family being
        // wrong. So offer each fragment of one such substrate in its slot
        // and carry the rest through unchanged, as spectators the ledger
        // still has to name: the sodium does not vanish because the
        // hydroxide was the interesting half. The whole-molecule error is
        // kept and surfaced only if no fragment matches either.
        for (slot, smiles) in substrate_smiles.iter().enumerate() {
            if !smiles.contains('.') {
                continue;
            }
            let fragments: Vec<&str> = smiles.split('.').collect();
            for (chosen, fragment) in fragments.iter().enumerate() {
                let mut trial = substrate_smiles.clone();
                trial[slot] = *fragment;
                if let Some(products) = run(&template, &trial)? {
                    let spectators: Vec<&str> = fragments
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != chosen)
                        .map(|(_, f)| *f)
                        .collect();
                    return Ok(Some(name_all(&record.id, &products, &spectators)?));
                }
            }
        }
        whole.map(|_| None)
    }
}

/// One template application: `Ok(None)` when the pattern does not match,
/// `Err` when the record itself is at fault (a malformed pattern, or a
/// product set that does not conserve matter — surfaced, never swallowed).
fn run(
    template: &templates::ReactionTemplate,
    smiles: &[&str],
) -> Result<Option<Vec<String>>, String> {
    match templates::apply_template(template, smiles) {
        Ok(p) if p.is_empty() => Ok(None),
        Ok(p) => Ok(Some(p)),
        Err(e) if e.contains("failed") => Ok(None),
        Err(e) => Err(e),
    }
}

/// Registry keys for every product and every spectator fragment, or the
/// refusal that names the structure nobody curated.
fn name_all(family: &str, products: &[String], spectators: &[&str]) -> Result<Vec<String>, String> {
    let mut keys = Vec::new();
    for smiles in products
        .iter()
        .map(String::as_str)
        .chain(spectators.iter().copied())
    {
        keys.extend(keys_of_product(family, smiles)?);
    }
    Ok(keys)
}

/// A product is named whole where the registry knows it whole, and
/// fragment by fragment where the toolkit wrote a salt as `[Na+].CC(=O)[O-]`
/// and the registry knows the ions. Anything else is the boundary.
fn keys_of_product(family: &str, smiles: &str) -> Result<Vec<String>, String> {
    if let Some(key) = key_of_product(smiles) {
        return Ok(vec![key.to_string()]);
    }
    if smiles.contains('.') {
        let mut keys = Vec::new();
        for fragment in smiles.split('.') {
            match key_of_product(fragment) {
                Some(key) => keys.push(key.to_string()),
                None => return Err(unnameable(family, smiles)),
            }
        }
        return Ok(keys);
    }
    Err(unnameable(family, smiles))
}

fn unnameable(family: &str, smiles: &str) -> String {
    format!(
        "family {family} produced a structure the registry cannot name ({smiles}); the pool of \
         the nameable is the boundary, and widening it is a registry task, not a silent drop"
    )
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

//! ORG-005: Atom-mapped transformation templates.
//!
//! A reaction template is a SMIRKS pattern that maps reactant atoms to
//! product atoms. The engine applies the template to actual molecules
//! to predict products.

use serde::{Deserialize, Serialize};

/// A curated reaction template with atom mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionTemplate {
    /// Human-readable name (e.g. "esterification").
    pub name: String,
    /// The reaction family this belongs to.
    pub family: String,
    /// SMIRKS string defining the atom-mapped transformation.
    pub smirks: String,
    /// Provenance: where this template came from.
    pub source: String,
    /// Whether this template has been validated against an oracle.
    pub validated: bool,
}

// ── BRD-020 phase 2: the conservation ledger ──────────────────────
//
// A transformation rule that invents or destroys atoms is not a
// transformation, and the moment its products enter the vessel ledger the
// error stops being local. So every application is weighed: same atoms,
// same charge, in and out. The rule must name why it declined, and
// "carbon: 4 in, 3 out" is a reason a chemist can act on.

/// What a set of molecules is made of.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    /// Element symbol → count, INCLUDING implicit hydrogens. A ledger that
    /// counted only heavy atoms would balance while hydrogens vanished.
    pub atoms: std::collections::BTreeMap<String, u32>,
    /// Summed formal charge.
    pub charge: i32,
}

/// What a template application failed to conserve. Only the differences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imbalance {
    /// element → (in, out), for elements whose counts differ.
    pub atoms: std::collections::BTreeMap<String, (u32, u32)>,
    /// (in, out) when charge is not conserved.
    pub charge: Option<(i32, i32)>,
}

impl std::fmt::Display for Imbalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts: Vec<String> = self
            .atoms
            .iter()
            .map(|(element, (before, after))| format!("{element}: {before} in, {after} out"))
            .collect();
        if let Some((before, after)) = self.charge {
            parts.push(format!("charge: {before} in, {after} out"));
        }
        write!(f, "{}", parts.join("; "))
    }
}

/// Weigh one molecule.
pub fn molecule_ledger(mol: &chematic::core::Molecule) -> Ledger {
    let mut ledger = Ledger::default();
    for (idx, atom) in mol.atoms() {
        *ledger
            .atoms
            .entry(atom.element.symbol().to_string())
            .or_insert(0) += 1;
        let implicit = mol.implicit_hydrogen_count(idx);
        if implicit > 0 {
            *ledger.atoms.entry("H".to_string()).or_insert(0) += implicit as u32;
        }
        ledger.charge += atom.charge as i32;
    }
    ledger
}

/// Weigh a set of molecules.
pub fn ledger_of(molecules: &[&chematic::core::Molecule]) -> Ledger {
    let mut total = Ledger::default();
    for mol in molecules {
        let one = molecule_ledger(mol);
        for (element, count) in one.atoms {
            *total.atoms.entry(element).or_insert(0) += count;
        }
        total.charge += one.charge;
    }
    total
}

/// Do these two ledgers balance?
pub fn conservation(before: &Ledger, after: &Ledger) -> Result<(), Imbalance> {
    let mut atoms = std::collections::BTreeMap::new();
    let elements: std::collections::BTreeSet<&String> =
        before.atoms.keys().chain(after.atoms.keys()).collect();
    for element in elements {
        let (a, b) = (
            before.atoms.get(element).copied().unwrap_or(0),
            after.atoms.get(element).copied().unwrap_or(0),
        );
        if a != b {
            atoms.insert(element.clone(), (a, b));
        }
    }
    let charge = (before.charge != after.charge).then_some((before.charge, after.charge));
    if atoms.is_empty() && charge.is_none() {
        Ok(())
    } else {
        Err(Imbalance { atoms, charge })
    }
}

/// Apply a SMIRKS template to reactant SMILES and return product SMILES.
pub fn apply_template(
    template: &ReactionTemplate,
    reactant_smiles: &[&str],
) -> Result<Vec<String>, String> {
    let reactants: Vec<_> = reactant_smiles
        .iter()
        .map(|s| chematic::smiles::parse(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("invalid reactant SMILES: {e}"))?;

    let reactant_refs: Vec<_> = reactants.iter().collect();
    let products = chematic::rxn::run_reactants(&template.smirks, &reactant_refs)
        .map_err(|e| format!("template application failed: {e}"))?;

    // BRD-020 phase 2. Weigh every product set against the reactants before
    // any of it reaches a caller. A set that does not balance is refused by
    // name — a rule that drops the by-product is not conserving, and one
    // that invents an atom is not a transformation.
    let before = ledger_of(&reactant_refs);
    for (index, set) in products.iter().enumerate() {
        let refs: Vec<&chematic::core::Molecule> = set.iter().collect();
        if let Err(imbalance) = conservation(&before, &ledger_of(&refs)) {
            return Err(format!(
                "template '{}' does not conserve matter (product set {}): {imbalance}",
                template.name,
                index + 1
            ));
        }
    }

    Ok(products
        .iter()
        .flat_map(|p| p.iter().map(chematic::smiles::write))
        .collect())
}

/// Apply a template without caring which order the reactants arrived in.
///
/// SMIRKS matching is POSITIONAL: `acid.alcohol>>ester.water` matches an
/// acid in slot one and an alcohol in slot two, and the same two molecules
/// handed over the other way round simply do not match. A bench does not
/// know which vessel the learner poured first, so a family matcher that
/// depends on that is a rule that fires or declines by accident.
///
/// Order is resolved DETERMINISTICALLY: permutations are tried in a fixed
/// order and the first conserving match wins, so the same inputs always
/// give the same products. Only up to three reactants are permuted — beyond
/// that the count stops being a small constant, and a template needing four
/// mapped reactants should say which is which.
pub fn apply_template_any_order(
    template: &ReactionTemplate,
    reactant_smiles: &[&str],
) -> Result<Vec<String>, String> {
    if reactant_smiles.len() > 3 {
        return apply_template(template, reactant_smiles);
    }
    let mut last_error = None;
    for order in permutations(reactant_smiles) {
        match apply_template(template, &order) {
            Ok(products) if !products.is_empty() => return Ok(products),
            Ok(_) => {}
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        format!(
            "template '{}' matches these reactants in no order",
            template.name
        )
    }))
}

/// Every ordering of up to three items, in a fixed sequence.
fn permutations<'a>(items: &[&'a str]) -> Vec<Vec<&'a str>> {
    match items {
        [] => vec![vec![]],
        [a] => vec![vec![a]],
        [a, b] => vec![vec![a, b], vec![b, a]],
        [a, b, c] => vec![
            vec![a, b, c],
            vec![a, c, b],
            vec![b, a, c],
            vec![b, c, a],
            vec![c, a, b],
            vec![c, b, a],
        ],
        _ => vec![items.to_vec()],
    }
}

// ── ORG-008: Conditions and incompatibility filters ────────────────

/// Conditions under which a reaction template applies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateConditions {
    /// Minimum temperature required (°C).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_temp_c: Option<f64>,
    /// Maximum temperature (°C) — above this, decomposition dominates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_temp_c: Option<f64>,
    /// Whether a catalyst is required.
    #[serde(default)]
    pub requires_catalyst: bool,
    /// Whether an acid/base medium is required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_medium: Option<String>,
    /// Functional groups that must NOT be present (incompatibility).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incompatible_groups: Vec<String>,
}

impl TemplateConditions {
    /// Check whether conditions are met for the given state.
    pub fn check(
        &self,
        temp_c: f64,
        has_catalyst: bool,
        present_groups: &[&str],
    ) -> Result<(), String> {
        if let Some(min) = self.min_temp_c {
            if temp_c < min {
                return Err(format!("temperature {temp_c}°C below minimum {min}°C"));
            }
        }
        if let Some(max) = self.max_temp_c {
            if temp_c > max {
                return Err(format!("temperature {temp_c}°C above maximum {max}°C"));
            }
        }
        if self.requires_catalyst && !has_catalyst {
            return Err("catalyst required but not present".into());
        }
        for incompat in &self.incompatible_groups {
            if present_groups.iter().any(|g| g == incompat) {
                return Err(format!("incompatible functional group: {incompat}"));
            }
        }
        Ok(())
    }
}

// ── ORG-010: Family registry ──────────────────────────────────────

/// A curated reaction family with its templates, conditions, and audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionFamily {
    pub name: String,
    pub templates: Vec<ReactionTemplate>,
    pub conditions: TemplateConditions,
    /// Source audit: where the family definition comes from.
    pub source_audit: String,
    /// Counterexamples: known cases where this family does NOT apply.
    pub counterexamples: Vec<String>,
    /// Selectivity/yield boundary: what this family does NOT claim.
    pub boundary: String,
}

/// The curated esterification template (ORG-006).
pub fn esterification() -> ReactionTemplate {
    ReactionTemplate {
        name: "esterification".into(),
        family: "condensation".into(),
        smirks: "[C:1](=[O:2])[OH:3].[OH:4][C:5]>>[C:1](=[O:2])[O:4][C:5].[OH2:3]".into(),
        source: "Fischer esterification, curated from March's Advanced Organic Chemistry".into(),
        validated: true,
    }
}

/// The curated saponification template (ORG-006).
pub fn saponification() -> ReactionTemplate {
    ReactionTemplate {
        name: "saponification".into(),
        family: "hydrolysis".into(),
        smirks: "[C:1](=[O:2])[O:3][C:4].[OH-:5]>>[C:1](=[O:2])[O-:5].[OH:3][C:4]".into(),
        source: "Alkaline ester hydrolysis, curated from March's Advanced Organic Chemistry".into(),
        validated: true,
    }
}

// ── EXP-50: Mechanistic selectivity templates ────────────────────

/// SN2: back-side attack on primary haloalkane by nucleophile.
pub fn sn2_alkyl_halide() -> ReactionTemplate {
    ReactionTemplate {
        name: "sn2-alkyl-halide".into(),
        family: "nucleophilic-substitution".into(),
        smirks: "[C:1][Br:2].[OH-:3]>>[C:1][OH:3].[Br-:2]".into(),
        source: "SN2 nucleophilic substitution, March's Advanced Organic Chemistry ch. 10".into(),
        validated: true,
    }
}

/// E2: anti-periplanar elimination from haloalkane by strong base.
pub fn e2_alkyl_halide() -> ReactionTemplate {
    ReactionTemplate {
        name: "e2-alkyl-halide".into(),
        family: "elimination".into(),
        smirks: "[CH2:1][CH2:2][Br:3].[OH-:4]>>[CH:1]=[CH2:2].[Br-:3].[OH2:4]".into(),
        source: "E2 elimination, March's Advanced Organic Chemistry ch. 10".into(),
        validated: true,
    }
}

// ── ORG-011: Oracle enrichment pipeline ───────────────────────────

/// An oracle-enriched property for a molecule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResult {
    /// The molecule (SMILES or InChIKey).
    pub molecule: String,
    /// The computed property name.
    pub property: String,
    /// The computed value.
    pub value: f64,
    /// Unit of the value.
    pub unit: String,
    /// Which oracle computed it.
    pub oracle: String,
    /// Whether the result has been individually reviewed.
    pub reviewed: bool,
}

/// Oracle enrichment pipeline: takes raw oracle output, validates it,
/// and produces reviewed records for the runtime registry.
pub fn validate_oracle_result(result: &OracleResult) -> Result<(), String> {
    if !result.reviewed {
        return Err(format!(
            "oracle result for {} ({}) has not been individually reviewed",
            result.molecule, result.property
        ));
    }
    if result.value.is_nan() || result.value.is_infinite() {
        return Err(format!(
            "oracle result for {} has non-finite value: {}",
            result.molecule, result.value
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_reject_low_temperature() {
        let cond = TemplateConditions {
            min_temp_c: Some(100.0),
            ..Default::default()
        };
        assert!(cond.check(25.0, false, &[]).is_err());
        assert!(cond.check(150.0, false, &[]).is_ok());
    }

    #[test]
    fn conditions_reject_incompatible_group() {
        let cond = TemplateConditions {
            incompatible_groups: vec!["aldehyde".into()],
            ..Default::default()
        };
        assert!(cond.check(25.0, false, &["alcohol"]).is_ok());
        assert!(cond.check(25.0, false, &["aldehyde"]).is_err());
    }

    #[test]
    fn esterification_family_applies() {
        // ORG-006/010: verify the curated esterification family template
        let t = esterification();
        assert_eq!(t.family, "condensation");
        assert!(t.validated);
        assert!(!t.smirks.is_empty());
    }

    #[test]
    fn template_serializes() {
        let t = ReactionTemplate {
            name: "esterification".into(),
            family: "condensation".into(),
            smirks: "[C:1](=[O:2])[OH:3].[OH:4][C:5]>>[C:1](=[O:2])[O:4][C:5].[OH2:3]".into(),
            source: "curated".into(),
            validated: false,
        };
        let json = serde_json::to_string(&t).unwrap();
        let loaded: ReactionTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "esterification");
    }
}

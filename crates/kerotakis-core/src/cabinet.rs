//! BRD-002: finding what is on the shelf.
//!
//! Stocking a bottle assumes you know what to ask for. The catalogue is
//! several hundred registry species plus every named material recipe, and
//! until now a host could list the species — all of them, unfiltered — or
//! nothing. A learner looking for "the vinegar" had no way to discover
//! that `vinegar` is a name the bench takes, what unit it is dispensed in,
//! or that `Essig` reaches the same bottle.
//!
//! Two things make this worth a module rather than a `filter` in one host:
//!
//! * **A material and a species are asked for the same way** — `add v1
//!   NaCl 1g`, `add v1 vinegar 250mL` — so a search that returns only one
//!   kind teaches a distinction the grammar does not make. The two are
//!   returned side by side, each saying which it is and what unit it
//!   dispenses in.
//! * **Aliases are the point, not a detail.** A recipe carries its names
//!   per language, and the German ones are how a German bench is usable at
//!   all. A match on an alias reports *which* alias matched, so the answer
//!   explains itself rather than looking like a coincidence.
//!
//! What this deliberately does not do is rank. Results come back in one
//! stated order — exact key first, then name, then alias, then substring —
//! because a relevance score nobody can explain is worse than a rule a
//! reader can predict.

use crate::material;
use crate::species;
use crate::stock::{stock_unit, StockUnit};

/// Whether an entry is a registry species or a named material recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CabinetKind {
    /// A canonical registry identity, dispensed in moles.
    Species,
    /// A reviewed named mixture or object, dispensed in its recipe basis.
    Material,
}

/// Why this entry matched, strongest first. The order of the variants is
/// the order results are returned in.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum CabinetMatch {
    /// The query is the key you would type.
    Key,
    /// The query is the entry's display name, or its formula.
    Name,
    /// The query is one of its aliases, in some language.
    Alias,
    /// The query appears somewhere in one of the above.
    Substring,
}

/// One shelf entry a query found.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CabinetEntry {
    /// What to type: the key `add` takes.
    pub key: String,
    pub name: String,
    pub kind: CabinetKind,
    /// The unit a dispense of this entry is counted in.
    pub unit: StockUnit,
    /// Formula for a species; the recipe's physical form for a material.
    pub detail: String,
    pub matched: CabinetMatch,
    /// The alias that matched, when `matched` is `Alias` — with its
    /// language tag, because "which German word was that" is the question
    /// a learner actually has.
    pub via: Option<String>,
}

fn norm(text: &str) -> String {
    text.trim().to_lowercase()
}

/// Search the catalogue — species and materials together.
///
/// An empty query returns nothing rather than everything: "show me all of
/// it" is what an unfiltered listing is for, and a search that silently
/// becomes one is a surprise waiting to happen in a host that forwards a
/// blank input.
pub fn search(query: &str, limit: usize) -> Vec<CabinetEntry> {
    let q = norm(query);
    if q.is_empty() || limit == 0 {
        return Vec::new();
    }
    let mut found: Vec<CabinetEntry> = Vec::new();

    let rank = |haystacks: &[&str]| -> Option<(CabinetMatch, Option<String>)> {
        for (index, text) in haystacks.iter().enumerate() {
            if norm(text) == q {
                return Some((
                    if index == 0 {
                        CabinetMatch::Key
                    } else {
                        CabinetMatch::Name
                    },
                    None,
                ));
            }
        }
        haystacks
            .iter()
            .any(|text| norm(text).contains(&q))
            .then_some((CabinetMatch::Substring, None))
    };

    for s in species::registry() {
        if let Some((matched, via)) = rank(&[s.key, s.name, s.formula]) {
            found.push(CabinetEntry {
                key: s.key.to_string(),
                name: s.name.to_string(),
                kind: CabinetKind::Species,
                unit: StockUnit::Mole,
                detail: s.formula.to_string(),
                matched,
                via,
            });
        }
    }

    for recipe in material::all() {
        let mut hit = rank(&[&recipe.canonical_key, &recipe.name]);
        // An alias match outranks a substring match on the canonical name,
        // because a learner who typed `Essig` meant the recipe, not a
        // coincidence inside some other word.
        if !matches!(hit, Some((CabinetMatch::Key | CabinetMatch::Name, _))) {
            for (language, names) in &recipe.aliases {
                if let Some(alias) = names.iter().find(|alias| norm(alias) == q) {
                    hit = Some((CabinetMatch::Alias, Some(format!("{language}: {alias}"))));
                    break;
                }
            }
        }
        if hit.is_none() {
            for (language, names) in &recipe.aliases {
                if let Some(alias) = names.iter().find(|alias| norm(alias).contains(&q)) {
                    hit = Some((
                        CabinetMatch::Substring,
                        Some(format!("{language}: {alias}")),
                    ));
                    break;
                }
            }
        }
        let Some((matched, via)) = hit else { continue };
        found.push(CabinetEntry {
            key: recipe.canonical_key.clone(),
            name: recipe.name.clone(),
            kind: CabinetKind::Material,
            // The dispensing unit is the one the ledger already counts in,
            // so what a search reports and what `stock` refuses in agree.
            unit: stock_unit(&recipe.canonical_key).unwrap_or(StockUnit::Gram),
            detail: physical_form_label(&recipe.physical_form),
            matched,
            via,
        });
    }

    // A substring hit is a FALLBACK, not a supplement. `find Essig` finds
    // vinegar by its German alias exactly — and also finds hand soap,
    // because "Essig" sits inside "Fluessigseife". Ordering put the right
    // answer first, but a learner who typed a word the catalogue knows
    // exactly should not have to read past coincidences to be sure. So
    // once anything matches exactly, the coincidences are dropped;
    // `find chlor`, which matches nothing exactly, still finds the eleven
    // chlorides, which is the case the verb exists for.
    if found.iter().any(|e| e.matched != CabinetMatch::Substring) {
        found.retain(|e| e.matched != CabinetMatch::Substring);
    }
    found.sort_by(|a, b| a.matched.cmp(&b.matched).then_with(|| a.key.cmp(&b.key)));
    found.truncate(limit);
    found
}

fn physical_form_label(form: &material::MaterialPhysicalForm) -> String {
    use material::MaterialPhysicalForm as F;
    match form {
        F::HomogeneousLiquid => "homogeneous liquid".to_string(),
        F::Suspension => "suspension".to_string(),
        F::Powder => "powder".to_string(),
        F::Granules => "granules".to_string(),
        F::BulkSolid => "bulk solid".to_string(),
        F::GasMixture => "gas mixture".to_string(),
        F::CompositeObject { .. } => "composite object".to_string(),
        // The recipe wrote its own description because none of the named
        // forms fit; repeating it is more use than calling it "other".
        F::Other { description } => description.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_exact_key_comes_first() {
        let hits = search("NaCl", 10);
        assert_eq!(hits.first().map(|e| e.key.as_str()), Some("NaCl"));
        assert_eq!(hits[0].matched, CabinetMatch::Key);
        assert_eq!(hits[0].kind, CabinetKind::Species);
    }

    #[test]
    fn a_substring_finds_what_an_exact_query_would_miss() {
        // The point of the verb: a learner who half-remembers a name.
        let hits = search("chlor", 50);
        assert!(
            hits.iter().any(|e| e.key == "NaCl"),
            "a partial name finds the salt: {hits:?}"
        );
        assert!(hits.iter().all(|e| e.matched == CabinetMatch::Substring));
    }

    /// A word the catalogue knows exactly should not arrive buried in
    /// coincidences. `Essig` is vinegar's German alias — and it is also
    /// inside `Fluessigseife`, which is hand soap.
    #[test]
    fn an_exact_hit_drops_the_coincidences() {
        let hits = search("Essig", 50);
        assert!(!hits.is_empty(), "the German alias finds something");
        assert!(
            hits.iter().all(|e| e.matched != CabinetMatch::Substring),
            "an exact match suppresses substring noise: {hits:?}"
        );
        assert!(
            hits.iter().any(|e| e.via.as_deref() == Some("de: Essig")),
            "and says which alias matched: {hits:?}"
        );
    }

    #[test]
    fn an_empty_query_finds_nothing_rather_than_everything() {
        assert!(search("", 50).is_empty());
        assert!(search("   ", 50).is_empty());
        assert!(search("NaCl", 0).is_empty());
    }

    /// Species and materials answer the same question, because `add` takes
    /// them the same way. A search that returned only one kind would teach
    /// a distinction the grammar does not make.
    #[test]
    fn materials_are_searched_beside_species() {
        let kinds: Vec<CabinetKind> = material::all()
            .iter()
            .take(1)
            .flat_map(|recipe| search(&recipe.canonical_key, 5))
            .map(|e| e.kind)
            .collect();
        assert!(
            kinds.contains(&CabinetKind::Material),
            "a recipe's own key finds the recipe: {kinds:?}"
        );
    }

    /// The unit a search reports is the unit the ledger counts in, or a
    /// learner is told to stock a bottle in one unit and refused in
    /// another.
    #[test]
    fn the_reported_unit_is_the_one_the_ledger_uses() {
        for entry in search("NaCl", 5) {
            if entry.kind == CabinetKind::Species {
                assert_eq!(entry.unit, StockUnit::Mole);
            }
        }
        for recipe in material::all() {
            let Some(expected) = stock_unit(&recipe.canonical_key) else {
                continue;
            };
            let hit = search(&recipe.canonical_key, 20)
                .into_iter()
                .find(|e| e.key == recipe.canonical_key && e.kind == CabinetKind::Material);
            if let Some(hit) = hit {
                assert_eq!(hit.unit, expected, "{} reports its shelf unit", hit.key);
            }
        }
    }
}

//! Metal–ligand complexation coverage and route selection, derived from the
//! routed databases themselves.
//!
//! Element coverage is not ligand coverage. wateq4f defines Ag and N(-3) but
//! no silver ammine; minteq.v4 defines nickel, cobalt, iron and mercury
//! ammines that wateq4f lacks. A vessel holding a metal and a ligand whose
//! complexes exist only on one route must be solved on that route, or its
//! speciation silently omits the complexes.
//!
//! Nothing here is a hand list of complexes. Every `SOLUTION_SPECIES`
//! reaction of every routed database (as the engine runs it, extensions
//! included) is reduced to the master elements on its left-hand side; a
//! species whose elements are exactly one metal and one ligand (protons and
//! hydroxide aside) is a binary complex of that (metal, ligand) family. The only reviewed inputs are classifications:
//!
//! * which ligand keys decide a route ([`ROUTE_DECIDING_LIGANDS`]) — the
//!   strong, specific complexing ligands. Background ion pairs (chloride,
//!   sulfate, carbonate, nitrate, hydroxide, …) are enumerated and reported
//!   but do not move the route: both databases carry most of them, and
//!   rerouting every common salt would trade the activity model of ordinary
//!   water for a weak ion pair;
//! * which metals decide a route: d- and p-block metals. s-block (group 1/2)
//!   cations form only weak outer-sphere pairs with these ligands; their
//!   pairs are reported, not route-deciding;
//! * complex families known to exist that no routed database defines
//!   ([`KNOWN_FAMILIES_WITHOUT_ROUTED_DATA`]), each with a textbook source.
//!   A vessel holding one is refused with a typed reason instead of being
//!   speciated without its dominant complexes. No constant is invented.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::{databases, derived};

/// Databases the aqueous route selector uses, in reporting order.
pub const ROUTED_DATABASES: [&str; 3] = ["wateq4f", "minteq.v4", "pitzer"];

/// Routes a metal–ligand family may select, in preference order. The first
/// route defining every present family wins, so wateq4f stays the default
/// wherever it is complete. pitzer carries none of these families.
pub const LIGAND_ROUTES: [&str; 2] = ["wateq4f", "minteq.v4"];

/// A ligand analytical key (as the databases name the master element) and
/// the family name used in reports.
#[derive(Debug, Clone, Copy)]
pub struct LigandFamily {
    pub key: &'static str,
    pub family: &'static str,
}

/// Strong, specific complexing ligands whose complexes decide the route.
/// `Oxalate` is listed although no routed database defines it, so the
/// coverage table states that gap instead of omitting it.
pub const ROUTE_DECIDING_LIGANDS: &[LigandFamily] = &[
    LigandFamily {
        key: "N(-3)",
        family: "ammine",
    },
    LigandFamily {
        key: "Thiosulfate",
        family: "thiosulfate",
    },
    LigandFamily {
        key: "Cyanide",
        family: "cyanide",
    },
    LigandFamily {
        key: "Thiocyanate",
        family: "thiocyanate",
    },
    LigandFamily {
        key: "Oxalate",
        family: "oxalate",
    },
    LigandFamily {
        key: "Citrate",
        family: "citrate",
    },
    LigandFamily {
        key: "Edta",
        family: "EDTA",
    },
    LigandFamily {
        key: "Acetate",
        family: "acetate",
    },
];

/// A complex family known to form that no routed database defines.
#[derive(Debug, Clone, Copy)]
pub struct KnownFamily {
    pub metal: &'static str,
    pub ligand: &'static str,
    pub source: &'static str,
}

/// Qualitative existence only, never a constant. A test fails as soon as a
/// routed database gains one of these, so the entry then becomes a route.
pub const KNOWN_FAMILIES_WITHOUT_ROUTED_DATA: &[KnownFamily] = &[
    KnownFamily {
        metal: "Zn",
        ligand: "N(-3)",
        source: "Greenwood and Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 29 (Zn(NH3)4 2+)",
    },
    KnownFamily {
        metal: "Cd",
        ligand: "N(-3)",
        source: "Greenwood and Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 29 (Cd(NH3)4 2+)",
    },
    KnownFamily {
        metal: "Zn",
        ligand: "Cyanide",
        source: "Greenwood and Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 29 (Zn(CN)4 2-)",
    },
    KnownFamily {
        metal: "Ag",
        ligand: "Thiosulfate",
        source: "Greenwood and Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 28 (Ag(S2O3)2 3-, photographic fixing)",
    },
    KnownFamily {
        metal: "Fe",
        ligand: "Oxalate",
        source: "Greenwood and Earnshaw, Chemistry of the Elements, 2nd ed. (1997), ch. 25 (Fe(C2O4)3 3-)",
    },
];

const ELEMENT_SYMBOLS: &[&str] = &[
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am",
];

const NONMETALS: &[&str] = &[
    "H", "He", "B", "C", "N", "O", "F", "Ne", "Si", "P", "S", "Cl", "Ar", "As", "Se", "Br", "Kr",
    "Te", "I", "Xe", "At", "Rn",
];

const S_BLOCK: &[&str] = &[
    "Li", "Na", "K", "Rb", "Cs", "Fr", "Be", "Mg", "Ca", "Sr", "Ba", "Ra",
];

/// Pseudo ligand key for hydrolysis species (metal + water, no ligand).
pub const HYDROXIDE: &str = "OH";

/// The element symbol of a canonical key: `Fe(3)` → `Fe`.
pub fn base(element: &str) -> &str {
    element.split('(').next().unwrap_or(element)
}

pub fn is_metal(symbol: &str) -> bool {
    ELEMENT_SYMBOLS.contains(&symbol) && !NONMETALS.contains(&symbol)
}

/// d- and p-block metals: the cations whose specific complexes matter.
pub fn is_route_deciding_metal(symbol: &str) -> bool {
    is_metal(symbol) && !S_BLOCK.contains(&symbol)
}

pub fn ligand_family(key: &str) -> Option<&'static str> {
    ROUTE_DECIDING_LIGANDS
        .iter()
        .find(|ligand| ligand.key == key)
        .map(|ligand| ligand.family)
}

/// Per database: species name → (master elements on its left-hand side,
/// whether the reaction consumes water).
type SpeciesElements = BTreeMap<String, (BTreeSet<String>, bool)>;

fn database_bytes(tag: &str) -> &'static [u8] {
    match tag {
        "minteq.v4" => databases::minteq_v4(),
        "pitzer" => databases::pitzer(),
        _ => databases::wateq4f(),
    }
}

/// Reduce every `SOLUTION_SPECIES` reaction to master elements. A
/// secondary species on a left-hand side contributes the elements of its
/// own (most recent) definition.
fn species_elements(db: &[u8], masters: &BTreeMap<String, String>) -> SpeciesElements {
    let text = String::from_utf8_lossy(db);
    let mut out: SpeciesElements = BTreeMap::new();
    let mut in_species = false;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("");
        if line.trim().is_empty() {
            continue;
        }
        let first = line.split_whitespace().next().unwrap_or("");
        if !line.starts_with([' ', '\t'])
            && first.len() > 3
            && first.chars().all(|c| c.is_ascii_uppercase() || c == '_')
        {
            in_species = first == "SOLUTION_SPECIES";
            continue;
        }
        if !in_species || !line.contains('=') {
            continue;
        }
        let (lhs, rhs) = line.split_once('=').unwrap_or((line, ""));
        let Some(name) = rhs.split_whitespace().next() else {
            continue;
        };
        let mut elements = BTreeSet::new();
        let mut water = false;
        for token in lhs.split_whitespace() {
            if token == "+" || token.chars().all(|c| c.is_ascii_digit() || c == '.') {
                continue;
            }
            let stripped = token.trim_start_matches(|c: char| c.is_ascii_digit() || c == '.');
            let token = if stripped
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '(')
            {
                stripped
            } else {
                token
            };
            if token == "H2O" {
                water = true;
            }
            if let Some(element) = masters.get(token) {
                elements.insert(element.clone());
            } else if let Some((known, _)) = out.get(token) {
                elements.extend(known.iter().cloned());
            }
        }
        out.insert(name.to_string(), (elements, water));
    }
    out
}

/// (metal symbol, ligand key) → database → complex species names.
pub type PairCoverage = BTreeMap<(String, String), BTreeMap<&'static str, Vec<String>>>;

fn build_coverage() -> PairCoverage {
    let mut pairs: PairCoverage = BTreeMap::new();
    for tag in ROUTED_DATABASES {
        let index = derived::index_for(tag);
        let species = species_elements(database_bytes(tag), &index.species_element);
        for (name, (elements, water)) in &species {
            let metals: BTreeSet<&str> = elements
                .iter()
                .map(|e| base(e))
                .filter(|b| is_metal(b))
                .collect();
            if metals.is_empty() {
                continue;
            }
            let mut ligands: BTreeSet<String> = elements
                .iter()
                .filter(|e| !is_metal(base(e)) && !matches!(base(e), "H" | "O"))
                .cloned()
                .collect();
            if ligands.is_empty() && *water {
                ligands.insert(HYDROXIDE.to_string());
            }
            // Binary complexes only: one metal, one ligand (protons and
            // hydroxide aside). A ternary species such as NH4Fe(CN)6-3 is
            // a cyanide complex with an ammonium counter-ion, not evidence
            // that the database defines iron ammines.
            if metals.len() != 1 || ligands.len() != 1 {
                continue;
            }
            for metal in &metals {
                for ligand in &ligands {
                    pairs
                        .entry((metal.to_string(), ligand.clone()))
                        .or_default()
                        .entry(tag)
                        .or_default()
                        .push(name.clone());
                }
            }
        }
    }
    pairs
}

pub fn coverage() -> &'static PairCoverage {
    static COVERAGE: OnceLock<PairCoverage> = OnceLock::new();
    COVERAGE.get_or_init(build_coverage)
}

/// Databases (from `routes`) that define at least one complex of the pair.
pub fn routes_defining(metal: &str, ligand: &str) -> Vec<&'static str> {
    let Some(by_db) = coverage().get(&(metal.to_string(), ligand.to_string())) else {
        return Vec::new();
    };
    ROUTED_DATABASES
        .into_iter()
        .filter(|tag| by_db.contains_key(tag))
        .collect()
}

/// Complex species of the ammine family on any routed database.
pub fn is_ammine_complex(name: &str) -> bool {
    coverage()
        .iter()
        .filter(|((_, ligand), _)| ligand == "N(-3)")
        .any(|(_, by_db)| by_db.values().any(|names| names.iter().any(|n| n == name)))
}

/// Metals the species registry can supply (any registered formula).
pub fn registry_metals() -> BTreeSet<String> {
    let mut metals = BTreeSet::new();
    for data in kerotakis_core::species::registry() {
        for symbol in formula_symbols(data.formula) {
            if is_metal(&symbol) {
                metals.insert(symbol);
            }
        }
    }
    metals
}

fn formula_symbols(formula: &str) -> Vec<String> {
    let chars: Vec<char> = formula.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let mut symbol = chars[i].to_string();
            if i + 1 < chars.len() && chars[i + 1].is_ascii_lowercase() {
                symbol.push(chars[i + 1]);
                i += 1;
            }
            out.push(symbol);
        }
        i += 1;
    }
    out
}

/// The outcome of metal–ligand route selection for one problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LigandRoute {
    /// No route-deciding (metal, ligand) family is present.
    Unconstrained,
    /// Every present family is defined on `db`, which also holds every
    /// element of the problem.
    Route {
        db: &'static str,
        pairs: Vec<(String, String)>,
    },
    /// A present family is known to form, but no routed database defines it.
    Undefined { pairs: Vec<(String, String)> },
    /// Present families are each defined somewhere, but on no single route
    /// that also holds every element.
    Conflict { pairs: Vec<(String, String)> },
}

/// Route-deciding (metal, ligand) families present among canonical problem
/// elements. A pair that no database defines and no reviewed source names
/// is not a complex family and is not returned.
pub fn present_pairs(elements: &[String]) -> Vec<(String, String)> {
    let metals: BTreeSet<&str> = elements
        .iter()
        .map(|e| base(e))
        .filter(|b| is_route_deciding_metal(b))
        .collect();
    let ligands: BTreeSet<&str> = elements
        .iter()
        .map(String::as_str)
        .filter(|e| ligand_family(e).is_some())
        .collect();
    let mut pairs = Vec::new();
    for metal in &metals {
        for ligand in &ligands {
            let defined = !routes_defining(metal, ligand).is_empty();
            let known = KNOWN_FAMILIES_WITHOUT_ROUTED_DATA
                .iter()
                .any(|k| k.metal == *metal && k.ligand == *ligand);
            if defined || known {
                pairs.push((metal.to_string(), ligand.to_string()));
            }
        }
    }
    pairs
}

pub fn select(elements: &[String]) -> LigandRoute {
    let pairs = present_pairs(elements);
    if pairs.is_empty() {
        return LigandRoute::Unconstrained;
    }
    let undefined: Vec<(String, String)> = pairs
        .iter()
        .filter(|(m, l)| {
            !routes_defining(m, l)
                .iter()
                .any(|tag| LIGAND_ROUTES.contains(tag))
        })
        .cloned()
        .collect();
    if !undefined.is_empty() {
        return LigandRoute::Undefined { pairs: undefined };
    }
    for db in LIGAND_ROUTES {
        let index = derived::index_for(db);
        if pairs
            .iter()
            .all(|(m, l)| routes_defining(m, l).contains(&db))
            && elements.iter().all(|e| index.has_element(e))
        {
            return LigandRoute::Route { db, pairs };
        }
    }
    LigandRoute::Conflict { pairs }
}

impl LigandRoute {
    /// True when a solve on `db` would omit or contradict a present family.
    pub fn blocks(&self, db: &str) -> bool {
        match self {
            LigandRoute::Unconstrained => false,
            LigandRoute::Route { db: chosen, .. } => *chosen != db,
            _ => true,
        }
    }
}

/// One row of the generated coverage table.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CoverageRow {
    pub metal: String,
    pub ligand: String,
    pub family: String,
    pub registry_supplies_metal: bool,
    pub route_deciding: bool,
    pub complexes: BTreeMap<String, Vec<String>>,
    pub decision: String,
    pub source: Option<String>,
}

fn family_label(ligand: &str) -> String {
    if ligand == HYDROXIDE {
        return "hydroxide".to_string();
    }
    ligand_family(ligand)
        .map(str::to_string)
        .unwrap_or_else(|| format!("background ({ligand})"))
}

/// Every enumerated (metal, ligand) family plus the reviewed known gaps.
pub fn coverage_rows() -> Vec<CoverageRow> {
    let registry = registry_metals();
    let mut rows: Vec<CoverageRow> = Vec::new();
    for ((metal, ligand), by_db) in coverage() {
        let deciding = is_route_deciding_metal(metal) && ligand_family(ligand).is_some();
        let defining: Vec<&str> = LIGAND_ROUTES
            .into_iter()
            .filter(|tag| by_db.contains_key(tag))
            .collect();
        let decision = if !deciding {
            if is_route_deciding_metal(metal) {
                "reported; background ligand, not route-deciding".to_string()
            } else {
                "reported; s-block or background pair, not route-deciding".to_string()
            }
        } else if defining.is_empty() {
            "not on a ligand route (pitzer only); refused if present".to_string()
        } else {
            format!("routes to {}", defining[0])
        };
        rows.push(CoverageRow {
            metal: metal.clone(),
            ligand: ligand.clone(),
            family: family_label(ligand),
            registry_supplies_metal: registry.contains(metal),
            route_deciding: deciding,
            complexes: by_db
                .iter()
                .map(|(tag, names)| (tag.to_string(), names.clone()))
                .collect(),
            decision,
            source: None,
        });
    }
    for known in KNOWN_FAMILIES_WITHOUT_ROUTED_DATA {
        if coverage().contains_key(&(known.metal.to_string(), known.ligand.to_string())) {
            continue;
        }
        rows.push(CoverageRow {
            metal: known.metal.to_string(),
            ligand: known.ligand.to_string(),
            family: family_label(known.ligand),
            registry_supplies_metal: registry.contains(known.metal),
            route_deciding: true,
            complexes: BTreeMap::new(),
            decision: "missing on every routed database; refused with a typed reason".to_string(),
            source: Some(known.source.to_string()),
        });
    }
    rows.sort_by(|a, b| (&a.metal, &a.ligand).cmp(&(&b.metal, &b.ligand)));
    rows
}

pub fn coverage_json() -> String {
    let mut text = serde_json::to_string_pretty(&coverage_rows()).expect("rows serialize");
    text.push('\n');
    text
}

/// The checked-in Markdown report (docs/metal-ligand-routing-coverage-20261027.md).
pub fn coverage_markdown() -> String {
    let rows = coverage_rows();
    let deciding: Vec<&CoverageRow> = rows.iter().filter(|r| r.route_deciding).collect();
    let count =
        |pred: &dyn Fn(&CoverageRow) -> bool| deciding.iter().copied().filter(|r| pred(r)).count();
    let mut out = String::new();
    out.push_str("# Metal-ligand routing coverage (2026-10-27)\n\n");
    out.push_str("Generated by `kerotakis_phreeqc::ligand_routing::coverage_markdown()` from the routed databases as the engine runs them (wateq4f and minteq.v4 with their reviewed extensions, pitzer). Do not edit by hand: `cargo test -p kerotakis-phreeqc --test metal_ligand_routing` fails when this file is stale; regenerate with `KEROTAKIS_BLESS_GOLDEN=1`.\n\n");
    out.push_str("A complex is any `SOLUTION_SPECIES` reaction whose left-hand master elements are exactly one metal and one ligand (protons and hydroxide aside); ternary species are not counted. Route-deciding families pair a d-/p-block metal with an ammine, thiosulfate, cyanide, thiocyanate, oxalate, citrate, EDTA or acetate ligand. The selector picks the first of wateq4f, minteq.v4 that defines every present family and holds every element; it refuses when a present family is defined nowhere (reviewed known gaps below) or on no single route.\n\n");
    out.push_str("## Summary\n\n");
    out.push_str(&format!(
        "- enumerated (metal, ligand) families: {}\n",
        rows.len()
    ));
    out.push_str(&format!("- route-deciding families: {}\n", deciding.len()));
    out.push_str(&format!(
        "- route-deciding, defined on wateq4f: {}\n",
        count(&|r| r.complexes.contains_key("wateq4f"))
    ));
    out.push_str(&format!(
        "- route-deciding, defined only on minteq.v4: {}\n",
        count(&|r| r.complexes.contains_key("minteq.v4") && !r.complexes.contains_key("wateq4f"))
    ));
    out.push_str(&format!(
        "- route-deciding, missing on every routed database (refused): {}\n",
        count(&|r| r.complexes.is_empty())
    ));
    out.push_str(&format!(
        "- route-deciding, metal suppliable from the registry: {}\n\n",
        count(&|r| r.registry_supplies_metal)
    ));
    out.push_str("## Route-deciding families\n\n");
    out.push_str("| metal | ligand | family | registry | wateq4f | minteq.v4 | pitzer | decision | source |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    let cell = |r: &CoverageRow, tag: &str| {
        r.complexes
            .get(tag)
            .map(|names| names.join(", "))
            .unwrap_or_else(|| "-".to_string())
    };
    for r in deciding.iter().copied() {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            r.metal,
            r.ligand,
            r.family,
            if r.registry_supplies_metal {
                "yes"
            } else {
                "no"
            },
            cell(r, "wateq4f"),
            cell(r, "minteq.v4"),
            cell(r, "pitzer"),
            r.decision,
            r.source.as_deref().unwrap_or("-"),
        ));
    }
    out.push_str("\n## Reported, not route-deciding\n\n");
    out.push_str("Counts of complex species per database.\n\n");
    out.push_str("| metal | ligand | family | wateq4f | minteq.v4 | pitzer |\n");
    out.push_str("|---|---|---|---|---|---|\n");
    for r in rows.iter().filter(|r| !r.route_deciding) {
        let n = |tag: &str| r.complexes.get(tag).map_or(0, Vec::len);
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            r.metal,
            r.ligand,
            r.family,
            n("wateq4f"),
            n("minteq.v4"),
            n("pitzer"),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn els(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn coefficients_and_secondary_species_reduce_to_master_elements() {
        let mut masters = BTreeMap::new();
        masters.insert("Ag+".to_string(), "Ag".to_string());
        masters.insert("NH4+".to_string(), "N(-3)".to_string());
        masters.insert("H+".to_string(), "H(1)".to_string());
        masters.insert("H2O".to_string(), "O(-2)".to_string());
        let db = b"SOLUTION_SPECIES\nNH4+ = NH3 + H+\n log_k -9.2\nAg+ + 2NH3 = Ag(NH3)2+\n log_k 7\nAg+ + H2O = AgOH + H+\nEND\n";
        let species = species_elements(db, &masters);
        assert_eq!(
            species["Ag(NH3)2+"].0,
            BTreeSet::from(["Ag".to_string(), "N(-3)".to_string()])
        );
        assert!(species["AgOH"].1);
    }

    #[test]
    fn s_block_and_nonmetals_never_decide_a_route() {
        assert!(is_route_deciding_metal("Ni"));
        assert!(!is_route_deciding_metal("Ca"));
        assert!(!is_route_deciding_metal("Na"));
        assert!(!is_metal("Cyanide"));
        assert_eq!(
            select(&els(&["Ca", "N(-3)", "Cl"])),
            LigandRoute::Unconstrained
        );
        assert_eq!(select(&els(&["Ag", "N(5)"])), LigandRoute::Unconstrained);
    }
}

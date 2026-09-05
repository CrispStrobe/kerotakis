//! Derived knowledge: everything this crate needs to know about a
//! thermodynamic database is parsed from the database itself — master
//! species per element (with formula weights), phase dissolution equations
//! (stoichiometry, hydrate waters, gas-ness, log K), and element coverage.
//! No hand-maintained tables of what the databases already state
//! ("derived, not hardcoded" applies to our own glue too).

use std::collections::BTreeMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MasterSpecies {
    /// The element name as the database writes it (e.g. "C(4)", "S(6)").
    pub element: String,
    /// The master species (e.g. "CO3-2", "SO4-2").
    pub species: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PhaseInfo {
    /// The solid/gas formula on the left of the dissolution equation,
    /// e.g. "CaSO4:2H2O", "NaCl", "CO2".
    pub formula: String,
    /// Anhydrous element counts of the formula (charge ignored, hydrate
    /// waters excluded), e.g. Gypsum → {Ca:1, S:1, O:4}.
    pub composition: BTreeMap<String, f64>,
    /// Waters of crystallisation per formula unit (the ":nH2O" suffix).
    pub waters: f64,
    /// Canonical elements the dissolution reaction moves into solution,
    /// with stoichiometry, e.g. Gypsum → [(Ca,1),(S(6),1)].
    pub elements: Vec<(String, f64)>,
    /// log K of the dissolution reaction (for polymorph choice: the stable
    /// polymorph has the lowest solubility, i.e. lowest log_k).
    pub log_k: Option<f64>,
    /// Enthalpy of the dissolution reaction AS WRITTEN, kJ/mol, from the
    /// database's own `delta_h` line — normalised out of whatever unit it
    /// was written in (minteq.v4 writes kJ, wateq4f writes kcal).
    ///
    /// `None` means the entry states no enthalpy, which is not the same as
    /// stating zero: a phase without one has no temperature dependence in
    /// this database and nothing may be derived from it.
    #[serde(default)]
    pub delta_h_kj: Option<f64>,
    /// Name ends in "(g)".
    pub is_gas: bool,
}

/// The activity model a database declares — derived from its own content,
/// not asserted by us: a `PITZER` block means the specific-ion-interaction
/// model; per-species `-gamma` parameters mean the WATEQ Debye-Hückel
/// extension; otherwise PHREEQC's Davies default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ActivityModel {
    /// Specific-ion interaction (Pitzer): valid to high ionic strength.
    Pitzer,
    /// WATEQ Debye-Hückel extension: reliable to roughly I = 1 mol/kgw.
    WateqDebyeHuckel,
    /// Davies equation: PHREEQC's default, roughly I < 0.5 mol/kgw.
    #[default]
    Davies,
}

impl ActivityModel {
    pub fn describe(self) -> &'static str {
        match self {
            ActivityModel::Pitzer => {
                "Pitzer specific-ion-interaction model (valid at high ionic strength)"
            }
            ActivityModel::WateqDebyeHuckel => {
                "WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)"
            }
            ActivityModel::Davies => "Davies equation (reliable to about I = 0.5 mol/kgw)",
        }
    }
}

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbIndex {
    /// Canonical element name → master species.
    pub masters: BTreeMap<String, MasterSpecies>,
    /// Master species name → canonical element (reverse lookup).
    pub species_element: BTreeMap<String, String>,
    /// Phase name → info.
    pub phases: BTreeMap<String, PhaseInfo>,
    /// Aqueous species name → the enthalpy of the `SOLUTION_SPECIES`
    /// reaction that DEFINES it, kJ/mol, as the database writes it
    /// (`H+ + CO3-2 = HCO3-` → the enthalpy of that association).
    ///
    /// This is the same quantity PHREEQC's `GetSpeciesDeltaH` returns, read
    /// from the file instead of from a running engine — which is what lets
    /// a reaction enthalpy be combined with a PHASES one, since phases have
    /// no engine accessor at all.
    #[serde(default)]
    pub species_delta_h_kj: BTreeMap<String, f64>,
    /// Activity model, derived from the file's own declarations.
    pub activity_model: ActivityModel,
    /// Literature citations found in the file's comments, in file order.
    /// These are the database's own record of where its numbers came from.
    pub citations: Vec<String>,
    /// Elements this database gives more than one oxidation state.
    ///
    /// Derived from the master-species block, which names them outright:
    /// `Fe(+2)`, `Fe(+3)`, `Mn(7)`, `N(-3)`. It is the difference between a
    /// solution whose redox state is *determined* by what is dissolved in
    /// it and one where pe is simply the value PHREEQC was handed — and
    /// reporting the second as a result would be reporting a default as a
    /// measurement.
    pub redox_elements: std::collections::BTreeSet<String>,
}

/// A comment that names a year and an author-ish token is the database
/// recording where a number came from ("Bénézeth et al., 2018, GCA 224").
fn looks_like_citation(c: &str) -> bool {
    let has_year = c
        .split(|ch: char| !ch.is_ascii_digit())
        .any(|t| t.len() == 4 && t.starts_with("19") || t.len() == 4 && t.starts_with("20"));
    let has_name = c.contains("et al") || c.contains(", 19") || c.contains(", 20");
    has_year && has_name
}

/// Canonicalise an element/valence name across databases: "C(+4)" and
/// "C(4)" are the same; carbon valence states collapse to "C" (we book
/// total dissolved carbonate). Others keep their valence tag.
pub fn canon_element(el: &str) -> String {
    let el = el.replace("(+", "(");
    match el.as_str() {
        "C(4)" => "C".to_string(),
        _ => el,
    }
}

impl DbIndex {
    pub fn parse(db: &[u8]) -> DbIndex {
        let text = String::from_utf8_lossy(db);
        let mut idx = DbIndex::default();
        let mut valences: BTreeMap<String, usize> = BTreeMap::new();
        let mut section = "";
        let mut pending_phase: Option<String> = None;
        let mut pending_species: Option<String> = None;
        let mut last_inserted: Option<String> = None;

        // Activity model: derived from what the file declares about itself.
        idx.activity_model = if text.contains("\nPITZER") {
            ActivityModel::Pitzer
        } else if text.contains("-gamma") {
            ActivityModel::WateqDebyeHuckel
        } else {
            ActivityModel::Davies
        };

        for raw in text.lines() {
            // A comment carrying a year is the database citing its source;
            // keep it as provenance rather than discarding it.
            if let Some((_, comment)) = raw.split_once('#') {
                let c = comment.trim();
                if c.len() > 12 && looks_like_citation(c) && !idx.citations.iter().any(|e| e == c) {
                    idx.citations.push(c.to_string());
                }
            }
            // Strip comments.
            let line = raw.split('#').next().unwrap_or("");
            if line.trim().is_empty() {
                continue;
            }
            let first = line.split_whitespace().next().unwrap_or("");
            // Keyword lines are upper-case at column 0.
            if !line.starts_with([' ', '\t'])
                && first.len() > 3
                && first.chars().all(|c| c.is_ascii_uppercase() || c == '_')
            {
                section = match first {
                    "SOLUTION_MASTER_SPECIES" => "masters",
                    "SOLUTION_SPECIES" => "species",
                    "PHASES" => "phases",
                    _ => "",
                };
                pending_phase = None;
                pending_species = None;
                continue;
            }

            match section {
                "masters" => {
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if tokens.len() < 2 {
                        continue;
                    }
                    let element = canon_element(tokens[0]);
                    if element == "E" {
                        continue;
                    }
                    // A valence-tagged master line — `Fe(+2)`, `Mn(7)` — is
                    // the database declaring that this element has more than
                    // one oxidation state to be in.
                    if tokens[0].contains('(') && tokens[0].contains(')') {
                        // Keyed on the *base* element. `canon_element` keeps
                        // the valence tag ("Mn(7)" stays "Mn(7)") because
                        // element totals are queried by it, so counting on
                        // that gave every oxidation state its own bucket and
                        // nothing ever reached two.
                        let base = tokens[0].split('(').next().unwrap_or(tokens[0]).to_string();
                        valences
                            .entry(base)
                            .and_modify(|n| *n += 1)
                            .or_insert(1usize);
                    }
                    if element == "Alkalinity" {
                        // Alkalinity's master species (HCO3-) belongs to the
                        // carbonate system: register the species→element
                        // mapping without inventing an "Alkalinity" element.
                        idx.species_element
                            .entry(tokens[1].to_string())
                            .or_insert_with(|| "C".to_string());
                        continue;
                    }
                    let species = tokens[1].to_string();
                    // A valence-specific element (S(6)) overrides the plain
                    // one (S) as the species' canonical element — the
                    // valence-tagged name is what totals are queried by.
                    match idx.species_element.get(&species) {
                        Some(existing) if existing.contains('(') => {}
                        _ => {
                            idx.species_element.insert(species.clone(), element.clone());
                        }
                    }
                    idx.masters
                        .entry(element.clone())
                        .or_insert(MasterSpecies { element, species });
                }
                "species" => {
                    // Indentation cannot be the signal here. minteq.v4
                    // writes its species equations at column 0; wateq4f
                    // indents EVERY line, equations included. Keying on
                    // column 0 parsed wateq4f's species enthalpies as an
                    // empty set — and an empty set is not an error, it is
                    // silence: every lookup then fell through to "it must
                    // be a master species, so it is zero", and every heat
                    // drawn from that database would have quietly gone to
                    // zero with no refusal anywhere.
                    //
                    // So split on content: a line with an `=` is a
                    // reaction and names the species it defines (the first
                    // product); anything else is an attribute of the one
                    // most recently named.
                    if line.contains('=') {
                        pending_species = line
                            .split('=')
                            .nth(1)
                            .and_then(|rhs| rhs.split_whitespace().next())
                            .map(|first| first.to_string())
                            .filter(|first| !first.is_empty());
                    } else if let Some(name) = &pending_species {
                        let tokens: Vec<&str> = line.split_whitespace().collect();
                        if tokens.first().is_some_and(|t| {
                            t.trim_start_matches('-').eq_ignore_ascii_case("delta_h")
                        }) {
                            if let Some(kj) = parse_delta_h(&tokens) {
                                idx.species_delta_h_kj.entry(name.clone()).or_insert(kj);
                            }
                        }
                    }
                }
                "phases" => {
                    if !line.starts_with([' ', '\t']) {
                        // Phase name line (column 0).
                        pending_phase = Some(first.to_string());
                        last_inserted = None;
                    } else if line.contains('=') {
                        if let Some(name) = pending_phase.take() {
                            if let Some(info) = parse_phase_equation(line, &idx, &name) {
                                idx.phases.insert(name.clone(), info);
                                last_inserted = Some(name);
                            }
                        }
                    } else if let Some(name) = &last_inserted {
                        // log_k line for the phase just inserted.
                        let tokens: Vec<&str> = line.split_whitespace().collect();
                        let keyword = tokens
                            .first()
                            .map(|t| t.trim_start_matches('-').to_ascii_lowercase());
                        match keyword.as_deref() {
                            Some("log_k") => {
                                if let Some(k) = tokens.get(1).and_then(|t| t.parse::<f64>().ok()) {
                                    if let Some(p) = idx.phases.get_mut(name) {
                                        if p.log_k.is_none() {
                                            p.log_k = Some(k);
                                        }
                                    }
                                }
                            }
                            Some("delta_h") => {
                                if let Some(kj) = parse_delta_h(&tokens) {
                                    if let Some(p) = idx.phases.get_mut(name) {
                                        if p.delta_h_kj.is_none() {
                                            p.delta_h_kj = Some(kj);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        // Two or more valence-tagged master lines means the element really
        // has a redox chemistry here; one alone is just a naming choice.
        idx.redox_elements = valences
            .into_iter()
            .filter(|(_, n)| *n >= 2)
            .map(|(el, _)| el)
            .collect();
        idx
    }

    /// OPT-6: Load a pre-parsed index from JSON (generated by generate-dbindex).
    /// This skips the ~50 ms of text parsing per database load.
    pub fn from_json(json: &str) -> Result<DbIndex, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// OPT-6: Serialize this index to JSON for caching.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn has_element(&self, canon: &str) -> bool {
        self.masters.contains_key(canon)
    }

    pub fn has_phase(&self, name: &str) -> bool {
        self.phases.contains_key(name)
    }
}

/// A PHREEQC `delta_h` line's value in kJ/mol.
///
/// The unit is optional and the databases disagree: minteq.v4 writes
/// `delta_h 4.06 kJ`, wateq4f writes `delta_h 3.72 kcal`. PHREEQC's own
/// default when none is given is kilojoules, so that is what an absent
/// unit means here too — getting this wrong is a silent factor of 4.184,
/// which is exactly the size of error that looks like a plausible number.
fn parse_delta_h(tokens: &[&str]) -> Option<f64> {
    let value: f64 = tokens.get(1)?.parse().ok()?;
    let unit = tokens
        .get(2)
        .map(|u| u.trim_end_matches("/mol").to_ascii_lowercase());
    let kj = match unit.as_deref() {
        None | Some("") | Some("kj") => value,
        Some("kcal") => value * 4.184,
        Some("cal") => value * 4.184e-3,
        Some("j") => value * 1e-3,
        // A unit we do not know is not a unit we may guess at.
        Some(_) => return None,
    };
    kj.is_finite().then_some(kj)
}

fn parse_phase_equation(line: &str, idx: &DbIndex, name: &str) -> Option<PhaseInfo> {
    let (lhs, rhs) = line.split_once('=')?;
    // The formula is the first LHS term (coefficient 1 by convention).
    // Terms are separated by " + " — a bare '+' split would break charged
    // species like Na+.
    let formula = lhs.split(" + ").next()?.trim().to_string();
    if formula.is_empty() {
        return None;
    }
    let (base, waters) = split_hydrate(&formula);
    let composition = parse_formula(&base)?;

    // Elements moved into solution: read the RHS master species.
    let mut elements: Vec<(String, f64)> = Vec::new();
    for term in rhs.split(" + ") {
        let term = term.trim();
        if term.is_empty() {
            continue;
        }
        let mut parts = term.split_whitespace();
        let (coeff, species) = match (parts.next(), parts.next()) {
            (Some(c), Some(s)) => match c.parse::<f64>() {
                Ok(n) => (n, s),
                Err(_) => (1.0, c),
            },
            (Some(s), None) => (1.0, s),
            _ => continue,
        };
        if matches!(species, "H2O" | "H+" | "OH-" | "e-") {
            continue;
        }
        if let Some(el) = idx.species_element.get(species) {
            if let Some(entry) = elements.iter_mut().find(|(e, _)| e == el) {
                entry.1 += coeff;
            } else {
                elements.push((el.clone(), coeff));
            }
        }
    }

    Some(PhaseInfo {
        formula,
        composition,
        waters,
        elements,
        log_k: None,
        delta_h_kj: None,
        is_gas: name.ends_with("(g)"),
    })
}

/// Split "CaSO4:2H2O" → ("CaSO4", 2.0). Also accepts "·" as used in
/// registry formulas.
pub fn split_hydrate(formula: &str) -> (String, f64) {
    for sep in [':', '·'] {
        if let Some((base, hydrate)) = formula.split_once(sep) {
            let digits: String = hydrate.chars().take_while(|c| c.is_ascii_digit()).collect();
            if hydrate.ends_with("H2O") {
                let n = if digits.is_empty() {
                    1.0
                } else {
                    digits.parse().unwrap_or(1.0)
                };
                return (base.to_string(), n);
            }
        }
    }
    (formula.to_string(), 0.0)
}

/// Element counts of a simple formula (charge suffixes ignored, one level
/// of parentheses supported). Returns None on anything unparseable.
pub fn parse_formula(formula: &str) -> Option<BTreeMap<String, f64>> {
    // OPT-8: this used to be a second, independent formula parser. The
    // 2026-08-23 differential over all 641 formulas in the shipped
    // databases found zero numeric disagreements with stoich's parser
    // and exactly one dialect difference — PHREEQC's pseudo-element
    // master species — so the second implementation died and this is
    // now an adapter over the one parser, asked in the PhreeqcMaster
    // dialect. tests/formula_parser_diff.rs holds the corpus.
    let f = kerotakis_core::stoich::parse_formula_with(
        formula,
        kerotakis_core::stoich::FormulaDialect::PhreeqcMaster,
    )
    .ok()?;
    if f.counts.is_empty() {
        return None;
    }
    Some(f.counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::databases;

    #[test]
    fn parses_the_three_databases() {
        let wateq = DbIndex::parse(databases::WATEQ4F);
        let minteq = DbIndex::parse(databases::MINTEQ_V4);
        let pitzer = DbIndex::parse(databases::PITZER);

        // Element coverage drives routing — derived, not listed.
        assert!(pitzer.has_element("Na") && pitzer.has_element("Ca"));
        assert!(!pitzer.has_element("N(5)") && !pitzer.has_element("Ag"));
        assert!(minteq.has_element("Acetate"));
        assert!(!wateq.has_element("Acetate"));
        assert!(wateq.has_element("P"));

        // Phase availability — derived.
        assert!(pitzer.has_phase("Sylvite"));
        assert!(!wateq.has_phase("Sylvite"));
        assert!(wateq.has_phase("Calcite") && minteq.has_phase("Calcite"));

        // Stoichiometry from the dissolution equations.
        let gypsum = &wateq.phases["Gypsum"];
        assert_eq!(gypsum.waters, 2.0, "CaSO4:2H2O carries two waters");
        assert!(gypsum.elements.iter().any(|(e, n)| e == "Ca" && *n == 1.0));
        assert!(gypsum
            .elements
            .iter()
            .any(|(e, n)| e == "S(6)" && *n == 1.0));

        let halite = &wateq.phases["Halite"];
        assert_eq!(halite.formula, "NaCl");

        // Polymorph choice: the stable form has the lower log_k.
        let calcite = wateq.phases["Calcite"].log_k.expect("calcite log_k");
        let aragonite = wateq.phases["Aragonite"].log_k.expect("aragonite log_k");
        assert!(
            calcite < aragonite,
            "calcite ({calcite}) is less soluble than aragonite ({aragonite})"
        );

        // Gas phases identified.
        assert!(wateq.phases["CO2(g)"].is_gas);
    }

    #[test]
    fn formula_parser_handles_the_registry() {
        let f = parse_formula("CaCO3").unwrap();
        assert_eq!(f["Ca"], 1.0);
        assert_eq!(f["C"], 1.0);
        assert_eq!(f["O"], 3.0);
        let (base, waters) = split_hydrate("CaSO4·2H2O");
        assert_eq!(base, "CaSO4");
        assert_eq!(waters, 2.0);
        let f = parse_formula("Ca(NO3)2").unwrap();
        assert_eq!(f["N"], 2.0);
        assert_eq!(f["O"], 6.0);
        // Charge and state suffixes ignored.
        assert_eq!(parse_formula("SO4-2").unwrap()["S"], 1.0);
        assert_eq!(parse_formula("NH3(aq)").unwrap()["N"], 1.0);
    }
}

#[cfg(test)]
mod delta_h_tests {
    use super::*;

    fn minteq() -> DbIndex {
        DbIndex::parse(crate::databases::MINTEQ_V4)
    }

    /// The unit is optional and the two databases disagree about it, so
    /// the conversion is the part worth pinning: a missed `kcal` is a
    /// silent factor of 4.184 and lands in the range a real answer would.
    #[test]
    fn delta_h_is_normalised_to_kilojoules() {
        assert_eq!(parse_delta_h(&["delta_h", "4.06", "kJ"]), Some(4.06));
        assert_eq!(parse_delta_h(&["delta_h", "4.06"]), Some(4.06));
        let kcal = parse_delta_h(&["delta_h", "3.72", "kcal"]).expect("kcal");
        assert!((kcal - 15.56448).abs() < 1e-6, "{kcal}");
        assert_eq!(parse_delta_h(&["delta_h", "1000", "J"]), Some(1.0));
        // A unit we do not know is not a unit we may guess at.
        assert_eq!(parse_delta_h(&["delta_h", "4.06", "furlongs"]), None);
        assert_eq!(parse_delta_h(&["delta_h", "not-a-number"]), None);
    }

    #[test]
    fn phase_and_species_enthalpies_come_off_the_shipped_files() {
        let m = minteq();
        let co2 = m.phases["CO2(g)"].delta_h_kj.expect("CO2(g) delta_h");
        assert!((co2 - 4.06).abs() < 1e-9, "{co2}");
        let hco3 = m.species_delta_h_kj["HCO3-"];
        assert!((hco3 + 14.6).abs() < 1e-9, "{hco3}");
        // Water's dissociation is written with TWO products, `H2O = OH- +
        // H+`. The species being defined is the first of them.
        let oh = m.species_delta_h_kj["OH-"];
        assert!((oh - 55.81).abs() < 1e-9, "{oh}");

        // EVERY shipped database must yield species enthalpies, not just
        // the one this module's carbonate cycle happens to use. wateq4f
        // indents its equations where minteq.v4 does not, and keying on
        // indentation parsed it as an empty set — which is silent, because
        // every lookup then falls through to "master species, therefore
        // zero". A corpus proved for one source says nothing about another.
        for (tag, bytes) in [
            ("wateq4f", crate::databases::WATEQ4F),
            ("minteq.v4", crate::databases::MINTEQ_V4),
        ] {
            let parsed = DbIndex::parse(bytes);
            let nonzero = parsed
                .species_delta_h_kj
                .values()
                .filter(|v| **v != 0.0)
                .count();
            assert!(
                nonzero > 50,
                "{tag}: only {nonzero} non-zero species enthalpies — suspect the parser"
            );
            let oh = parsed
                .species_delta_h_kj
                .get("OH-")
                .unwrap_or_else(|| panic!("{tag} defines no hydroxide enthalpy"));
            assert!(*oh > 40.0 && *oh < 70.0, "{tag}: OH- at {oh} kJ/mol");
        }

        // pitzer is the deliberate exception and is pinned as one rather
        // than left to look like a parser failure: it states its equilibria
        // as `-analytic` temperature expressions, not `delta_h`, and gives
        // no enthalpy for hydroxide at all. `enthalpy` refuses a heat
        // balance on that database by name instead of pricing whatever
        // happens to be a master species at zero and calling the rest of
        // the sum an answer.
        let pitzer = DbIndex::parse(crate::databases::PITZER);
        assert!(
            !pitzer.species_delta_h_kj.contains_key("OH-"),
            "pitzer now states a hydroxide enthalpy — the refusal in `enthalpy` can be lifted"
        );

        // wateq4f writes kcal; the same field must come back in kJ.
        let w = DbIndex::parse(crate::databases::WATEQ4F);
        let co2 = w.phases["CO2(g)"].delta_h_kj.expect("CO2(g) delta_h");
        assert!((co2 - (-4.776 * 4.184)).abs() < 1e-6, "{co2}");

        // An entry that states no enthalpy must stay None rather than
        // becoming a confident zero. wateq4f has ~100 such phases; minteq.v4
        // states one for very nearly everything, which is itself the reason
        // the carbonate cycle above is derivable there and not here.
        assert!(
            w.phases
                .values()
                .any(|p| p.delta_h_kj.is_none() && p.log_k.is_some()),
            "no wateq4f phase lacks an enthalpy — suspect the parser, not the file"
        );
    }

    /// The Hess cycle in `derived::carbonate_acid_enthalpy_kj` depends on
    /// the SHAPE of these two rows, not only on their numbers. If a
    /// database update rewrites either, the algebra silently becomes a
    /// different reaction — so pin the shapes here and fail loudly.
    #[test]
    fn carbonate_rows_are_the_shape_the_algebra_assumes() {
        let text = String::from_utf8_lossy(crate::databases::MINTEQ_V4);
        assert!(
            text.contains("H+ + CO3-2 = HCO3-"),
            "minteq.v4 no longer defines HCO3- by protonating the carbonate master species"
        );
        assert!(
            text.contains("CO2 + H2O = 2 H+ + CO3-2"),
            "minteq.v4's CO2(g) dissolution is no longer written to the carbonate master species"
        );
    }
}

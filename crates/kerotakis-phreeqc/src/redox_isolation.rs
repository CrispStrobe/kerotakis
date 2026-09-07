//! Native slow-redox isolation by separate conserved master components.
//!
//! USGS PHREEQC Example 4 and the PHREEQC FAQ require separate elements to
//! prevent homogeneous redox equilibration during a batch reaction or MIX.
//! This transformation changes the component namespace, not equilibrium
//! constants for acid/base or complex formation. Public names remain physical.
//!
//! Primary documentation (no numeric constants imported):
//! <https://water.usgs.gov/water-resources/software/PHREEQC/documentation/phreeqc3-html/phreeqc3-66.htm>
//! <https://wwwbrr.cr.usgs.gov/projects/GWC_coupled/phreeqc.v1/faq.html>
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub struct IsolatedDatabase {
    pub database: Vec<u8>,
    pub totals: BTreeMap<String, String>,
    pub species: BTreeMap<String, String>,
}

type Counts = BTreeMap<String, f64>;

#[derive(Debug)]
struct Reaction {
    line: usize,
    section: String,
    lhs: Vec<(String, f64)>,
    rhs: Vec<(String, f64)>,
}

fn terms(side: &str) -> Result<Vec<(String, f64)>, String> {
    let mut result = Vec::new();
    let mut coefficient = 1.0;
    for token in side.split_whitespace() {
        if token == "+" {
            continue;
        }
        if token == "-" {
            coefficient = -1.0;
            continue;
        }
        if let Ok(number) = token.parse::<f64>() {
            coefficient *= number;
            continue;
        }
        let split = token
            .find(|c: char| c.is_ascii_alphabetic() || c == '[' || c == '(')
            .unwrap_or(0);
        let (number, name) = token.split_at(split);
        if !number.is_empty() {
            coefficient *= number
                .parse::<f64>()
                .map_err(|_| format!("unsupported reaction term {token}"))?;
        }
        result.push((name.to_string(), coefficient));
        coefficient = 1.0;
    }
    Ok(result)
}

fn pseudo(element: &str, state: i32) -> String {
    let digits = [
        "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    let suffix = state
        .unsigned_abs()
        .to_string()
        .bytes()
        .map(|b| digits[(b - b'0') as usize])
        .collect::<String>();
    format!("{element}{}{suffix}", if state < 0 { "red" } else { "ox" })
}

fn add(into: &mut Counts, from: &Counts, scale: f64) {
    for (key, value) in from {
        *into.entry(key.clone()).or_default() += scale * value;
    }
}

fn composition(name: &str) -> Option<Counts> {
    if let Some((base, hydrate)) = name.split_once(':') {
        let mut result = composition(base)?;
        let end = hydrate
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(0);
        let n = if end == 0 {
            1.0
        } else {
            hydrate[..end].parse::<f64>().ok()?
        };
        add(&mut result, &composition(&hydrate[end..])?, n);
        return Some(result);
    }
    kerotakis_core::stoich::parse_formula_with(
        name,
        kerotakis_core::stoich::FormulaDialect::PhreeqcMaster,
    )
    .ok()
    .map(|f| f.counts)
}

fn slow_count(name: &str) -> f64 {
    element_tokens(name)
        .iter()
        .filter(|e| matches!(e.as_str(), "N" | "S"))
        .count() as f64
}

fn element_tokens(name: &str) -> Vec<String> {
    let bytes = name.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_uppercase() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_lowercase() {
                i += 1;
            }
            result.push(name[start..i].to_string());
        } else {
            i += 1;
        }
    }
    result
}

fn inferred(reaction: &Reaction, known: &BTreeMap<String, Counts>) -> Option<Counts> {
    let mut result = Counts::new();
    for (name, coefficient) in &reaction.lhs {
        add(&mut result, known.get(name)?, *coefficient);
    }
    for (name, coefficient) in reaction.rhs.iter().skip(1) {
        add(&mut result, known.get(name)?, -*coefficient);
    }
    let coefficient = reaction.rhs.first()?.1;
    for value in result.values_mut() {
        *value /= coefficient;
    }
    result.retain(|_, value| value.abs() > 1e-10);
    (result.values().all(|n| *n >= 0.0)).then_some(result)
}

fn rename_formula(name: &str, states: &Counts) -> Result<String, String> {
    if states.is_empty() {
        return Ok(name.to_string());
    }
    let elements = element_tokens(name);
    let mut replacements = BTreeMap::new();
    for element in ["N", "S"] {
        if !elements.iter().any(|e| e == element) {
            continue;
        }
        let components: Vec<_> = states
            .iter()
            .filter(|(key, _)| key.starts_with(element))
            .collect();
        if components.len() != 1 {
            return Err(format!(
                "mixed or unresolved {element} states within species {name}: {states:?}"
            ));
        }
        replacements.insert(element, components[0].0.as_str());
    }
    let bytes = name.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_uppercase() {
            let start = i;
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_lowercase() {
                i += 1;
            }
            let element = &name[start..i];
            out.push_str(replacements.get(element).copied().unwrap_or(element));
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    Ok(out)
}

fn replace_tokens(line: &str, aliases: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    let mut start = 0;
    for (i, ch) in line.char_indices() {
        if ch.is_whitespace() {
            if start < i {
                let token = &line[start..i];
                out.push_str(aliases.get(token).map(String::as_str).unwrap_or(token));
            }
            out.push(ch);
            start = i + ch.len_utf8();
        }
    }
    if start < line.len() {
        let token = &line[start..];
        out.push_str(aliases.get(token).map(String::as_str).unwrap_or(token));
    }
    out
}

fn render_terms(terms: &[(String, f64)], aliases: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (name, n) in terms {
        if !out.is_empty() {
            out.push_str(if *n < 0.0 { " - " } else { " + " });
        } else if *n < 0.0 {
            out.push('-');
        }
        if n.abs() != 1.0 {
            out.push_str(&format!("{} ", n.abs()));
        }
        out.push_str(aliases.get(name).unwrap_or(name));
    }
    out
}

fn inline_parameters(line: &str) -> String {
    line.split('#')
        .next()
        .unwrap_or("")
        .split_once(';')
        .map(|(_, rest)| format!(";{rest}"))
        .unwrap_or_default()
}

/// Transform the selected database, including reviewed appended extensions.
/// Unsupported unresolved state-bearing reactions are errors, not a fallback
/// to the original redox-equilibrating database.
pub fn transform(bytes: &[u8]) -> Result<IsolatedDatabase, String> {
    // PHREEQC databases are byte-oriented, with Latin-1 in comments.
    let text: String = bytes.iter().map(|b| char::from(*b)).collect();
    let lines: Vec<_> = text.lines().collect();
    let mut section = String::new();
    let mut master_rows = Vec::new();
    let mut phase_headers = BTreeSet::new();
    let mut reactions = Vec::new();
    let mut masses = BTreeMap::new();
    for (line, source) in lines.iter().enumerate() {
        let content = source.split('#').next().unwrap_or("").trim();
        if content.is_empty() {
            continue;
        }
        if matches!(
            content,
            "SOLUTION_MASTER_SPECIES"
                | "SOLUTION_SPECIES"
                | "PHASES"
                | "EXCHANGE_MASTER_SPECIES"
                | "EXCHANGE_SPECIES"
                | "SURFACE_MASTER_SPECIES"
                | "SURFACE_SPECIES"
                | "RATES"
                | "PITZER"
                | "SIT"
                | "NAMED_EXPRESSIONS"
                | "END"
        ) {
            section = content.to_string();
            continue;
        }
        if section == "SOLUTION_MASTER_SPECIES" {
            let fields: Vec<_> = content.split_whitespace().collect();
            if fields.len() >= 4 && matches!(fields[0].split('(').next(), Some("N" | "S")) {
                if !fields[0].contains('(') && fields.len() >= 5 {
                    masses.insert(fields[0].to_string(), fields[4].to_string());
                }
                master_rows.push((
                    line,
                    fields.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                ));
            }
        }
        if matches!(
            section.as_str(),
            "SOLUTION_SPECIES" | "PHASES" | "EXCHANGE_SPECIES" | "SURFACE_SPECIES"
        ) {
            if let Some((lhs, rhs)) = content.split_once('=') {
                if section == "PHASES" {
                    // PHREEQC permits indentation and a trailing reference
                    // number on phase names (for example `MnSO4 182`). The
                    // last meaningful line before its equation is the name;
                    // lexical resemblance to an aqueous formula is irrelevant.
                    let header = (0..line)
                        .rev()
                        .find(|i| !lines[*i].split('#').next().unwrap_or("").trim().is_empty())
                        .ok_or_else(|| "phase equation without a phase name".to_string())?;
                    phase_headers.insert(header);
                }
                let rhs = rhs.split(';').next().unwrap_or(rhs);
                reactions.push(Reaction {
                    line,
                    section: section.clone(),
                    lhs: terms(lhs)?,
                    rhs: terms(rhs)?,
                });
            }
        }
    }
    let mut totals = BTreeMap::new();
    let mut known = BTreeMap::new();
    let mut promoted = BTreeSet::new();
    let mut edits = BTreeMap::new();
    for (line, fields) in &master_rows {
        let Some((element, oxidation)) = fields[0].split_once('(') else {
            edits.insert(*line, String::new());
            continue;
        };
        let state = oxidation
            .trim_end_matches(')')
            .parse::<i32>()
            .map_err(|_| format!("invalid state {}", fields[0]))?;
        let component = pseudo(element, state);
        let counts =
            composition(&fields[1]).ok_or_else(|| format!("master formula {}", fields[1]))?;
        let count = *counts
            .get(element)
            .ok_or_else(|| format!("master missing element {}", fields[1]))?;
        known.insert(
            fields[1].clone(),
            BTreeMap::from([(component.clone(), count)]),
        );
        promoted.insert(fields[1].clone());
        totals.insert(format!("{element}({state})"), component.clone());
        totals.insert(format!("{element}({state:+})"), component.clone());
        let mass = masses
            .get(element)
            .ok_or_else(|| format!("missing atomic weight {element}"))?;
        edits.insert(
            *line,
            format!("{component} {} {} {mass} {mass}", fields[1], fields[2]),
        );
    }
    for (_, fields) in &master_rows {
        if !fields[0].contains('(') {
            let key = known
                .get(&fields[1])
                .and_then(|counts| counts.keys().next())
                .ok_or_else(|| {
                    format!(
                        "primary master has no declared oxidation state {}",
                        fields[1]
                    )
                })?;
            totals.insert(fields[0].clone(), key.clone());
        }
    }
    for reaction in &reactions {
        for (name, _) in reaction.lhs.iter().chain(&reaction.rhs) {
            if slow_count(name) == 0.0 {
                known.entry(name.clone()).or_default();
            }
        }
    }
    loop {
        let mut progress = false;
        for reaction in reactions.iter().filter(|r| r.section != "PHASES") {
            let Some((name, _)) = reaction.rhs.first() else {
                continue;
            };
            if known.contains_key(name) {
                continue;
            }
            if let Some(mut states) = inferred(reaction, &known) {
                // Native polysulfide mass action can consume one HS- while
                // its explicit mass balance carries several sulfur atoms.
                // Membership follows the reaction; multiplicity follows the
                // species formula, not a fictitious one-sulfur molecule.
                if let Some(counts) = composition(name) {
                    for element in ["N", "S"] {
                        let components: Vec<_> = states
                            .keys()
                            .filter(|k| k.starts_with(element))
                            .cloned()
                            .collect();
                        if components.len() == 1 {
                            if let Some(count) = counts.get(element) {
                                states.insert(components[0].clone(), *count);
                            }
                        }
                    }
                }
                known.insert(name.clone(), states);
                progress = true;
            }
        }
        if !progress {
            break;
        }
    }
    let mut species = BTreeMap::new();
    for reaction in &reactions {
        for (name, _) in reaction.lhs.iter().chain(&reaction.rhs) {
            if let Some(states) = known.get(name) {
                let renamed = rename_formula(name, states)?;
                if renamed != *name {
                    species.insert(name.clone(), renamed);
                }
            } else if reaction.section == "SOLUTION_SPECIES" {
                return Err(format!("unresolved oxidation-state membership of {name}"));
            }
        }
    }
    let mut reverse = BTreeMap::new();
    for (physical, internal) in &species {
        if let Some(previous) = reverse.insert(internal, physical) {
            return Err(format!("internal species alias collision: {previous} and {physical} both map to {internal}"));
        }
    }
    // Mineral and gas formula membership comes from its dissolution products,
    // not nominal elemental charge. Use the native reaction's stoichiometry.
    for reaction in reactions.iter().filter(|r| r.section == "PHASES") {
        let Some((formula, coefficient)) = reaction.lhs.first() else {
            continue;
        };
        if slow_count(formula) == 0.0 {
            continue;
        }
        let mut states = Counts::new();
        for (name, n) in &reaction.rhs {
            add(
                &mut states,
                known
                    .get(name)
                    .ok_or_else(|| format!("unresolved phase product {name}"))?,
                *n / coefficient,
            );
        }
        for (name, n) in reaction.lhs.iter().skip(1) {
            add(
                &mut states,
                known
                    .get(name)
                    .ok_or_else(|| format!("unresolved phase reactant {name}"))?,
                -*n / coefficient,
            );
        }
        if states.values().any(|n| !n.is_finite() || *n < -1e-10) {
            return Err(format!(
                "invalid isolated phase composition for {formula}: {states:?}"
            ));
        }
        // A phase formula may contain multiple oxidation states (NH4NO3).
        // Serialize physical non-N/S counts and the separately conserved pools.
        let mut formula_counts =
            composition(formula).ok_or_else(|| format!("phase formula {formula}"))?;
        formula_counts.remove("N");
        formula_counts.remove("S");
        add(&mut formula_counts, &states, 1.0);
        let renamed = formula_counts
            .iter()
            .filter(|(_, n)| **n > 1e-10)
            .map(|(element, n)| {
                if *n == 1.0 {
                    element.clone()
                } else {
                    format!("{element}{n}")
                }
            })
            .collect::<String>();
        let mut lhs = reaction.lhs.clone();
        lhs[0].0 = renamed;
        let render = |terms: &[(String, f64)]| render_terms(terms, &species);
        edits.insert(
            reaction.line,
            format!(
                "    {} = {}{}",
                render(&lhs),
                render(&reaction.rhs),
                inline_parameters(lines[reaction.line])
            ),
        );
    }
    for reaction in reactions.iter().filter(|r| r.section == "SOLUTION_SPECIES") {
        let name = &reaction.rhs[0].0;
        if promoted.contains(name) {
            let alias = species.get(name).unwrap_or(name);
            edits.insert(reaction.line, format!("{alias} = {alias}\n log_k 0"));
            let end = reactions
                .iter()
                .find(|r| r.line > reaction.line)
                .map_or(lines.len(), |r| r.line);
            for (i, line) in lines.iter().enumerate().take(end).skip(reaction.line + 1) {
                let parameter = line
                    .trim()
                    .trim_start_matches('-')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if matches!(
                    parameter.as_str(),
                    "log_k"
                        | "logk"
                        | "delta_h"
                        | "deltah"
                        | "analytical_expression"
                        | "analytic"
                        | "a_e"
                        | "add_logk"
                        | "add_log_k"
                ) {
                    edits.insert(i, String::new());
                }
            }
        }
    }
    for reaction in &reactions {
        if edits.contains_key(&reaction.line) {
            continue;
        }
        let render = |terms: &[(String, f64)]| render_terms(terms, &species);
        let suffix = inline_parameters(lines[reaction.line]);
        edits.insert(
            reaction.line,
            format!(
                "    {} = {}{suffix}",
                render(&reaction.lhs),
                render(&reaction.rhs)
            ),
        );
    }
    let mut output = String::new();
    for (i, line) in lines.iter().enumerate() {
        let line = edits.get(&i).map(String::as_str).unwrap_or(line);
        let line = if phase_headers.contains(&i) {
            line.to_string()
        } else {
            replace_tokens(line, &species)
        };
        // Explicit valence mass balances (polysulfides) also use isolated pools.
        let mut rewritten = line;
        for (physical, internal) in &totals {
            if physical.contains('(') {
                rewritten = rewritten.replace(physical, internal);
            }
        }
        output.push_str(&rewritten);
        output.push('\n');
    }
    Ok(IsolatedDatabase {
        database: output.chars().map(|c| c as u8).collect(),
        totals,
        species,
    })
}

#[cfg(test)]
mod tests {
    use super::transform;

    #[test]
    fn isolated_phase_keeps_inline_thermodynamic_parameters_exactly() {
        for suffix in [
            "; log_k -2.125",
            "; log_k -2.125; delta_h 3.75 kJ",
            "; -log_k -2.125; -delta_h -4.5 kcal; -analytical_expression 1 2 3 4 5 6",
        ] {
            let source = format!(
                "SOLUTION_MASTER_SPECIES\nN NO3- 0 N 14.0067\nN(5) NO3- 0 N\n\
                 SOLUTION_SPECIES\nNO3- = NO3-\n log_k 0\n\
                 PHASES\nTestNitrate\n NaNO3 = Na+ + NO3-{suffix}\nEND\n"
            );
            let transformed = transform(source.as_bytes()).expect("isolate nitrate phase");
            let result = String::from_utf8(transformed.database).unwrap();
            assert!(result.contains("\nTestNitrate\n"));
            let equation = result.lines().find(|line| line.contains("Na+")).unwrap();
            assert!(equation.contains("Noxfive"), "{equation}");
            assert_eq!(
                equation.split_once(';').map(|(_, rest)| rest),
                suffix.strip_prefix(';'),
                "inline phase properties changed: {equation}"
            );
        }
    }
}

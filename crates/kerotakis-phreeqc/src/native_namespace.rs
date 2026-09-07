//! Translate the adapter's physical names at the native component boundary.
use crate::redox_isolation::IsolatedDatabase;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

pub(crate) fn database(tag: &str) -> Result<&'static IsolatedDatabase, String> {
    static WATEQ: OnceLock<Result<IsolatedDatabase, String>> = OnceLock::new();
    static MINTEQ: OnceLock<Result<IsolatedDatabase, String>> = OnceLock::new();
    static PITZER: OnceLock<Result<IsolatedDatabase, String>> = OnceLock::new();
    let (slot, bytes) = match tag {
        "wateq4f" => (&WATEQ, crate::databases::wateq4f()),
        "minteq.v4" => (&MINTEQ, crate::databases::minteq_v4()),
        "pitzer" => (&PITZER, crate::databases::pitzer()),
        _ => return Err(format!("unknown native database {tag}")),
    };
    slot.get_or_init(|| crate::redox_isolation::transform(bytes))
        .as_ref()
        .map_err(Clone::clone)
}

pub(crate) fn fingerprint(tag: &str) -> Result<&'static str, String> {
    static WATEQ: OnceLock<Result<String, String>> = OnceLock::new();
    static MINTEQ: OnceLock<Result<String, String>> = OnceLock::new();
    static PITZER: OnceLock<Result<String, String>> = OnceLock::new();
    let slot = match tag {
        "wateq4f" => &WATEQ,
        "minteq.v4" => &MINTEQ,
        "pitzer" => &PITZER,
        _ => return Err(format!("unknown native database {tag}")),
    };
    slot.get_or_init(|| database(tag).map(|db| kerotakis_data::snapshot_sha256(&db.database)))
        .as_ref()
        .map(String::as_str)
        .map_err(Clone::clone)
}

fn words(text: &str, mut replace: impl FnMut(&str) -> String) -> String {
    let mut out = String::new();
    for part in text.split_inclusive(char::is_whitespace) {
        let word = part.trim_end_matches(char::is_whitespace);
        out.push_str(&replace(word));
        out.push_str(&part[word.len()..]);
    }
    out
}

pub(crate) fn input(db: &IsolatedDatabase, source: &str) -> String {
    let mut result = String::new();
    let mut section = "";
    for line in source.lines() {
        let mut tokens = line.split_whitespace();
        let first = tokens.next().unwrap_or("");
        if matches!(
            first,
            "SOLUTION"
                | "SOLUTION_SPECIES"
                | "PHASES"
                | "SELECTED_OUTPUT"
                | "EQUILIBRIUM_PHASES"
                | "GAS_PHASE"
                | "SOLID_SOLUTIONS"
                | "SURFACE"
                | "EXCHANGE"
                | "MIX"
                | "REACTION"
                | "SAVE"
                | "END"
                | "TITLE"
        ) {
            section = first;
        }
        if first == "-totals" {
            let mut selected = BTreeSet::new();
            for token in tokens {
                if matches!(token, "N" | "S") && db.totals.contains_key(token) {
                    for (key, internal) in &db.totals {
                        if key.starts_with(&format!("{token}(")) {
                            selected.insert(internal.clone());
                        }
                    }
                } else {
                    selected.insert(
                        db.totals
                            .get(token)
                            .cloned()
                            .unwrap_or_else(|| token.into()),
                    );
                }
            }
            result.push_str("    -totals ");
            result.push_str(&selected.into_iter().collect::<Vec<_>>().join(" "));
        } else if section == "SOLUTION" {
            // Only the first field is a component name; following fields
            // may name a gas/phase buffer and must retain their public names.
            let mut first_word = true;
            result.push_str(&words(line, |word| {
                if word.is_empty() {
                    return String::new();
                }
                let replace = first_word;
                first_word = false;
                if replace {
                    db.totals.get(word).cloned().unwrap_or_else(|| word.into())
                } else {
                    word.into()
                }
            }));
        } else if (section == "SOLUTION_SPECIES" && line.contains('='))
            || (section == "PHASES" && line.contains('='))
            || matches!(section, "EXCHANGE" | "SURFACE")
            || matches!(first, "-molalities" | "-activities")
        {
            result.push_str(&words(line, |word| {
                db.species.get(word).cloned().unwrap_or_else(|| word.into())
            }));
        } else {
            // A phase name can also be an aqueous species name (CuSO4).
            // Phase selection/output and solid-solution component names are
            // not chemical formula fields and must never be renamed.
            result.push_str(line);
        }
        result.push('\n');
    }
    result
}

/// Restore exact species tokens, never substrings (N is not Na or Nitrate).
pub(crate) fn report(db: &IsolatedDatabase, native: &str) -> String {
    let reverse: BTreeMap<_, _> = db
        .species
        .iter()
        .map(|(p, n)| (n.as_str(), p.as_str()))
        .collect();
    words(native, |word| {
        crate::complexation::native_to_physical(reverse.get(word).copied().unwrap_or(word)).into()
    })
}

pub(crate) fn selected(db: &IsolatedDatabase, mut rows: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let Some(native_header) = rows.first().cloned() else {
        return rows;
    };
    let mut physical = BTreeMap::new();
    for (name, internal) in &db.totals {
        if let Some((element, state)) = name.split_once('(') {
            if let Ok(state) = state.trim_end_matches(')').parse::<i32>() {
                physical.insert(internal.as_str(), format!("{element}({state})"));
            }
        }
    }
    let species: BTreeMap<_, _> = db
        .species
        .iter()
        .map(|(p, n)| (n.as_str(), p.as_str()))
        .collect();
    for header in &mut rows[0] {
        let trimmed = header.trim();
        *header = if let Some(name) = physical.get(trimmed) {
            name.clone()
        } else if let Some((prefix, native)) = trimmed.split_once('_') {
            if matches!(prefix, "m" | "la" | "a") {
                format!(
                    "{prefix}_{}",
                    crate::complexation::native_to_physical(
                        species.get(native).copied().unwrap_or(native)
                    )
                )
            } else {
                trimmed.into()
            }
        } else {
            trimmed.into()
        };
    }
    for element in ["N", "S"] {
        let expected: BTreeSet<_> = db
            .totals
            .iter()
            .filter(|(key, _)| key.starts_with(&format!("{element}(")))
            .map(|(_, internal)| internal.as_str())
            .collect();
        if expected.is_empty()
            || !expected
                .iter()
                .all(|name| native_header.iter().any(|column| column.trim() == *name))
        {
            // A requested oxidation state is not necessarily the complete
            // element. Never label a partial column sum as an element total.
            continue;
        }
        let columns: Vec<_> = native_header
            .iter()
            .enumerate()
            .filter_map(|(i, name)| {
                physical
                    .get(name.trim())
                    .filter(|key| key.starts_with(&format!("{element}(")))
                    .map(|_| i)
            })
            .collect();
        if columns.is_empty() {
            continue;
        }
        rows[0].push(element.into());
        for row in rows.iter_mut().skip(1) {
            let sum: Option<f64> = columns
                .iter()
                .map(|&i| row.get(i)?.trim().parse::<f64>().ok())
                .sum();
            row.push(
                sum.map(|n| format!("{n:.16e}"))
                    .unwrap_or_else(|| "NaN".into()),
            );
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> IsolatedDatabase {
        IsolatedDatabase {
            database: vec![],
            totals: BTreeMap::from([
                ("N".into(), "Noxfive".into()),
                ("N(5)".into(), "Noxfive".into()),
                ("N(-3)".into(), "Nredthree".into()),
            ]),
            species: BTreeMap::from([
                ("NH4+".into(), "NredthreeH4+".into()),
                ("NH3".into(), "NredthreeH3".into()),
            ]),
        }
    }
    #[test]
    fn totals_species_and_similar_names_keep_distinct_meanings() {
        let db = fixture();
        let translated = input(&db, "SOLUTION 1\n N(-3) 0.01\n Na 0.02\nSELECTED_OUTPUT\n -totals N N(-3) N(5) Na\n -molalities NH3 NH4+\nEND\n");
        assert!(translated.contains("Nredthree 0.01"));
        assert!(translated.contains("Na 0.02"));
        assert!(translated.contains("-totals Na Noxfive Nredthree"));
        assert!(translated.contains("-molalities NredthreeH3 NredthreeH4+"));
        assert_eq!(
            report(&db, " NredthreeH3 0.01\n Na+ 0.02\n"),
            " NH3 0.01\n Na+ 0.02\n"
        );
        let rows = selected(
            &db,
            vec![
                vec!["Noxfive".into(), "Nredthree".into(), "m_NredthreeH3".into()],
                vec!["0.002".into(), "0.003".into(), "0.001".into()],
            ],
        );
        assert_eq!(rows[0], ["N(5)", "N(-3)", "m_NH3", "N"]);
        assert!((rows[1][3].parse::<f64>().unwrap() - 0.005).abs() < 1e-14);
    }

    #[test]
    fn a_phase_name_is_not_an_aqueous_species_formula() {
        let mut db = fixture();
        db.species.insert("CuSO4".into(), "CuSoxsixO4".into());
        let source = "EQUILIBRIUM_PHASES 1\n CuSO4 0 0.01\nGAS_PHASE 1\n N2(g) 0.8\nSOLID_SOLUTIONS 1\n mix\n -comp1 CuSO4 0.01\nSELECTED_OUTPUT\n -equilibrium_phases CuSO4\n -saturation_indices CuSO4\n -molalities CuSO4\nEND\n";
        let translated = input(&db, source);
        assert_eq!(
            translated,
            source.replace("-molalities CuSO4", "-molalities CuSoxsixO4")
        );
    }

    #[test]
    fn a_partial_state_selection_does_not_invent_an_element_total() {
        let rows = selected(
            &fixture(),
            vec![vec!["Nredthree".into()], vec!["0.003".into()]],
        );
        assert_eq!(rows[0], ["N(-3)"]);
        assert_eq!(rows[1], ["0.003"]);
    }
}

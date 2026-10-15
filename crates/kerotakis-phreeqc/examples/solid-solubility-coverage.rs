//! Generated role/phase admission inventory, not a solubility calculation.
//! See docs/kno3-finite-solubility-lane.md for the distinct validity questions.

use kerotakis_core::species::{self, Phase};
use kerotakis_phreeqc::derived::{self, DerivedRole, DB_TAGS};
use serde_json::{json, Value};

fn inventory() -> Value {
    let crosswalk = derived::mineral_crosswalk();
    let mut solids: Vec<_> = species::REGISTRY
        .iter()
        .filter(|species| species.standard_phase == Phase::Solid)
        .collect();
    solids.sort_by_key(|species| species.key);
    let rows: Vec<Value> = solids
        .into_iter()
        .map(|species| {
            let (role, elements, mineral) = match derived::role(species.key) {
                Some(DerivedRole::Dissolves(elements)) => {
                    ("phase_blind_dissolves", elements.clone(), None)
                }
                Some(DerivedRole::Mineral { phase, elements }) => {
                    ("mineral", elements.clone(), Some(phase.as_str()))
                }
                Some(DerivedRole::Solvent) => ("solvent", Vec::new(), None),
                None => ("no_aqueous_role", Vec::new(), None),
            };
            let matched = crosswalk.iter().find(|row| row.species == species.key);
            let candidates: Vec<_> = derived::candidate_phases()
                .iter()
                .filter(|phase| phase.species == species.key)
                .map(|phase| phase.name.as_str())
                .collect();
            let reviewed_limit = species.aqueous_solubility_g_per_100_ml;
            let databases: Vec<_> = DB_TAGS
                .iter()
                .map(|tag| {
                    let index = derived::index_for(tag);
                    let mut matched_phases: Vec<_> = matched
                        .into_iter()
                        .flat_map(|row| row.phases.iter())
                        .filter(|(database, _)| database == tag)
                        .map(|(_, phase)| phase.as_str())
                        .collect();
                    matched_phases.sort_unstable();
                    let missing_elements: Vec<_> = elements
                        .iter()
                        .filter(|(element, _)| !index.has_element(element))
                        .map(|(element, _)| element.as_str())
                        .collect();
                    let native_phase = mineral.and_then(|phase| derived::phase_in_db(phase, tag));
                    json!({
                        "database": tag,
                        "composition_matched_phases": matched_phases,
                        "native_equilibrium_phase": native_phase,
                        "mineral_role_without_native_phase": mineral.is_some() && native_phase.is_none(),
                        "element_admission": if elements.is_empty() { None } else { Some(missing_elements.is_empty()) },
                        "missing_elements": missing_elements,
                    })
                })
                .collect();
            json!({
                "species": species.key,
                "formula": species.formula,
                "role": role,
                "elements": elements,
                "candidate_phases": candidates,
                "reviewed_solubility_g_per_100_ml": reviewed_limit,
                "reviewed_solubility_g_per_100_ml_at_100c": species.aqueous_solubility_g_per_100_ml_at_100c,
                "dissolves_without_speciation": species.dissolves_without_speciation,
                "elemental_metal": kerotakis_core::displacement::is_elemental_metal(species.key),
                "condensed_gas": kerotakis_core::phase_route::is_condensed_gas(species.key),
                "phase_blind_input": role == "phase_blind_dissolves",
                "finite_solubility_unreviewed": role == "phase_blind_dissolves" && reviewed_limit.is_none() && candidates.is_empty(),
                "databases": databases,
            })
        })
        .collect();
    json!({
        "schema_version": 1,
        "kind": "compiled_solid_role_and_phase_inventory",
        "scope": "All registry standard-phase solids, including unmapped and intentionally excluded solids",
        "limits": [
            "Generated role and element admission are not physical solubility or activity-model validation",
            "A bookkeeping flag does not establish finite dissolution support",
            "finite_solubility_unreviewed=false does not establish support; the flag identifies one specific fallback mechanism",
            "Missing native phases require review of borrowed-phase, coupled, surface and fallback paths; no solve is performed",
            "Bind source, database inputs, dependency lock and executable separately before publishing counts",
        ],
        "rows": rows,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer_pretty(std::io::stdout().lock(), &inventory())?;
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(report: &Value, key: &str) -> Value {
        report["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["species"] == key)
            .unwrap_or_else(|| panic!("missing solid {key}"))
            .clone()
    }

    #[test]
    fn every_registry_solid_is_present_once_in_stable_order() {
        let report = inventory();
        let keys: Vec<_> = report["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["species"].as_str().unwrap())
            .collect();
        let mut expected: Vec<_> = species::REGISTRY
            .iter()
            .filter(|species| species.standard_phase == Phase::Solid)
            .map(|species| species.key)
            .collect();
        expected.sort_unstable();
        assert_eq!(keys, expected);
        assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn nitrate_gap_is_visible_without_calling_element_admission_solubility() {
        let nitrate = row(&inventory(), "KNO3");
        assert_eq!(nitrate["role"], "phase_blind_dissolves");
        assert_eq!(nitrate["finite_solubility_unreviewed"], true);
        assert_eq!(nitrate["candidate_phases"], json!([]));
        let pitzer = nitrate["databases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|database| database["database"] == "pitzer")
            .unwrap();
        assert_eq!(pitzer["element_admission"], false);
        assert_eq!(pitzer["missing_elements"], json!(["N(5)"]));
    }

    #[test]
    fn native_mineral_and_bounded_borax_are_distinct_from_unbounded_solid() {
        let report = inventory();
        let salt = row(&report, "NaCl");
        assert_eq!(salt["role"], "mineral");
        assert_eq!(salt["phase_blind_input"], false);
        assert_eq!(salt["finite_solubility_unreviewed"], false);
        assert!(salt["databases"]
            .as_array()
            .unwrap()
            .iter()
            .all(|database| database["native_equilibrium_phase"].is_string()));
        let borax = row(&report, "Na2B4O7");
        assert!(borax["reviewed_solubility_g_per_100_ml"].is_number());
        assert_eq!(borax["role"], "no_aqueous_role");
        assert_eq!(borax["phase_blind_input"], false);
        assert_eq!(borax["finite_solubility_unreviewed"], false);
    }
}

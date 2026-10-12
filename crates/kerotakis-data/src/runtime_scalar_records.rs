//! Selection of the three mandatory constant records for runtime construction.
//!
//! A unique molar-mass record supplies the runtime standard-phase anchor.
//! Cp and density must each have one record in that phase. Other phases remain
//! valid source records and are neither selected nor prohibited here.

use serde_json::Value;

/// The records used to construct one runtime species' constant properties.
/// Unit normalization and numeric bounds are checked by their existing callers.
pub struct RuntimeScalarRecords<'a> {
    pub molar_mass: &'a Value,
    pub heat_capacity: &'a Value,
    pub density: &'a Value,
}

fn unique_record<'a>(
    records: &'a [Value],
    key: &str,
    property: &str,
    phase: Option<&str>,
) -> Result<&'a Value, String> {
    let mut matching = records.iter().filter(|row| {
        row["species_id"] == key
            && row["property"] == property
            && phase.is_none_or(|phase| row["phase"].as_str() == Some(phase))
    });
    let first = matching.next().ok_or_else(|| match phase {
        Some(phase) => format!("{key}: no {property} record for runtime phase '{phase}'"),
        None => format!("{key}: no {property} record"),
    })?;
    if matching.next().is_some() {
        return Err(match phase {
            Some(phase) => {
                format!("{key}: ambiguous {property} records for runtime phase '{phase}'")
            }
            None => format!("{key}: ambiguous {property} records for runtime phase anchor"),
        });
    }
    Ok(first)
}

/// Select runtime fallback constants without depending on document order.
/// This is not a schema-wide ban on multiple phases, and it does not select
/// optional thermodynamic properties or polynomial curves.
pub fn select_runtime_scalar_records<'a>(
    records: &'a [Value],
    key: &str,
) -> Result<RuntimeScalarRecords<'a>, String> {
    let molar_mass = unique_record(records, key, "molar_mass", None)?;
    let phase = molar_mass["phase"]
        .as_str()
        .ok_or_else(|| format!("{key}: molar mass has no phase"))?;
    Ok(RuntimeScalarRecords {
        molar_mass,
        heat_capacity: unique_record(records, key, "molar_heat_capacity", Some(phase))?,
        density: unique_record(records, key, "mass_density", Some(phase))?,
    })
}

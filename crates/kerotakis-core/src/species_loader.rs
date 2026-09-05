//! DATA-010: species from a loaded pack, at runtime.
//!
//! This is build.rs's document→`SpeciesData` join, mirrored for runtime:
//! the same field sourcing over the same registry-document shape, but
//! producing leaked `'static` values instead of generated code. Drift
//! between the two joins is pinned by `tests/loader_fidelity.rs`, which
//! parses the SAME source document this build compiled and demands
//! field-for-field equality with `REGISTRY`.
//!
//!
//! Leaking is deliberate and bounded: packs load at most once per
//! session per pack, and the alternative (owned strings) would fork
//! `SpeciesData` into two types across the whole engine.

use crate::species::{Colour, Phase, Resistivity, SpeciesData};

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn phase_of(p: &str) -> Result<Phase, String> {
    Ok(match p {
        "solid" => Phase::Solid,
        "liquid" => Phase::Liquid,
        "aqueous" => Phase::Aqueous,
        "gas" => Phase::Gas,
        other => return Err(format!("phase '{other}' has no runtime Phase variant")),
    })
}

/// A loaded species' absorption spectrum: the optical record's sixteen
/// bands, leaked once. DATA-011 removed the v1 fn-pointer limitation —
/// pack species now colour their solutions exactly like built-ins.
fn spectrum_of(
    key: &str,
    opt: Option<&serde_json::Value>,
) -> Result<Option<&'static crate::spectrum::Spectrum>, String> {
    let Some(samples) = opt
        .and_then(|o| o["spectrum"].as_array())
        .filter(|s| !s.is_empty())
    else {
        return Ok(None);
    };
    if samples.len() != crate::spectrum::BANDS {
        return Err(format!(
            "{key}: spectrum must carry {} bands, has {}",
            crate::spectrum::BANDS,
            samples.len()
        ));
    }
    let mut bands = [0.0f64; crate::spectrum::BANDS];
    for (b, s) in bands.iter_mut().zip(samples) {
        *b = s["molar_absorptivity"]["value"]
            .as_f64()
            .ok_or_else(|| format!("{key}: spectrum band without a value"))?;
    }
    Ok(Some(Box::leak(Box::new(bands))))
}

/// Parse a registry document (the JSON shape of
/// `kerotakis-data`'s `RegistryDocument` / the DATA-002 export) into
/// runtime species. Every error names its species and field — a pack
/// with one bad record refuses as a whole rather than half-loading.
pub fn parse_document(doc: &serde_json::Value) -> Result<Vec<SpeciesData>, String> {
    let arr = |k: &str| -> Result<&Vec<serde_json::Value>, String> {
        doc[k]
            .as_array()
            .ok_or_else(|| format!("document has no '{k}' array"))
    };
    let identities = arr("identities")?;
    let compositions = arr("compositions")?;
    let thermo = arr("phase_thermodynamics")?;
    let optical = arr("optical")?;
    let params = arr("model_parameters")?;
    let sources = arr("sources")?;

    let find_source = |id: &str| -> Result<&str, String> {
        sources
            .iter()
            .find(|s| s["id"] == id)
            .and_then(|s| s["citation"].as_str())
            .ok_or_else(|| format!("no source citation for {id}"))
    };
    let thermo_for = |key: &str, prop: &str| -> Option<&serde_json::Value> {
        thermo
            .iter()
            .find(|t| t["species_id"] == key && t["property"] == prop)
    };
    // EXP-33: sublimation, decomposition and dehydration have no typed
    // `PhaseProperty`, so they arrive under the schema's `Other(String)`
    // escape — an object, not a bare string.
    let thermo_other = |key: &str, name: &str| -> Option<&serde_json::Value> {
        thermo
            .iter()
            .find(|t| t["species_id"] == key && t["property"]["other"] == name)
    };
    let param_for = |key: &str, parameter: &str| -> Option<f64> {
        params
            .iter()
            .find(|p| {
                p["parameter"] == parameter
                    && p["subject"]["kind"] == "species"
                    && p["subject"]["id"] == key
            })
            .and_then(|p| p["quantity"]["value"].as_f64())
    };

    let mut out = Vec::with_capacity(identities.len());
    for identity in identities {
        let key = identity["id"]
            .as_str()
            .ok_or_else(|| "identity without id".to_string())?;
        let fail = |what: &str| format!("{key}: {what}");

        let comp = compositions
            .iter()
            .find(|c| c["species_id"] == key)
            .ok_or_else(|| fail("no composition"))?;
        let mm = thermo_for(key, "molar_mass").ok_or_else(|| fail("no molar mass"))?;
        let molar_mass = mm["quantity"]["value"]
            .as_f64()
            .ok_or_else(|| fail("molar mass has no value"))?;
        let standard_phase = phase_of(
            mm["phase"]
                .as_str()
                .ok_or_else(|| fail("molar mass has no phase"))?,
        )
        .map_err(|e| fail(&e))?;
        let heat_capacity = thermo_for(key, "molar_heat_capacity")
            .and_then(|t| t["quantity"]["value"].as_f64())
            .ok_or_else(|| fail("no heat capacity"))?;
        let density = thermo_for(key, "mass_density")
            .and_then(|t| t["quantity"]["value"].as_f64())
            .ok_or_else(|| fail("no density"))?;
        let dissolution = thermo_for(key, "enthalpy_of_dissolution")
            .and_then(|t| t["quantity"]["value"].as_f64());

        let opt = optical.iter().find(|o| o["species_id"] == key);
        let appearance = opt
            .and_then(|o| o["appearance"].as_str())
            .map(leak)
            .map(|s| s as &'static str);
        let flame_colour = opt.and_then(|o| o["flame_colour"].as_str()).map(leak);
        let colour = match opt.and_then(|o| o["reflective_srgb"].as_str()) {
            Some(hex) => {
                let h = hex.trim_start_matches('#');
                let byte = |r: std::ops::Range<usize>| {
                    u8::from_str_radix(h.get(r).unwrap_or(""), 16)
                        .map_err(|_| fail(&format!("bad srgb hex '{hex}'")))
                };
                let strength =
                    param_for(key, "strength").ok_or_else(|| fail("srgb without tint strength"))?;
                Some(Colour {
                    r: byte(0..2)?,
                    g: byte(2..4)?,
                    b: byte(4..6)?,
                    strength,
                })
            }
            None => None,
        };

        // EXP-33, mirroring build.rs: one citation stands behind the row,
        // taken from the transition records' own `source_id` rather than
        // the species' general provenance line.
        let transitions = {
            let rows: [(&str, Option<&serde_json::Value>); 5] = [
                ("melting point", thermo_for(key, "melting_temperature")),
                ("boiling point", thermo_for(key, "boiling_temperature")),
                (
                    "sublimation point",
                    thermo_other(key, "sublimation_temperature"),
                ),
                (
                    "decomposition point",
                    thermo_other(key, "decomposition_temperature"),
                ),
                (
                    "dehydration point",
                    thermo_other(key, "dehydration_temperature"),
                ),
            ];
            if rows.iter().all(|(_, r)| r.is_none()) {
                None
            } else {
                let mut values: [Option<f64>; 5] = [None; 5];
                let mut source: Option<&'static str> = None;
                let mut boundary: Option<&'static str> = None;
                for (slot, (what, row)) in rows.iter().enumerate() {
                    let Some(r) = row else { continue };
                    let v = r["quantity"]["value"]
                        .as_f64()
                        .ok_or_else(|| fail(&format!("{what} has no value")))?;
                    if r["quantity"]["unit"]["symbol"].as_str() != Some("K") {
                        return Err(fail(&format!("{what} must be given in kelvin")));
                    }
                    values[slot] = Some(v);
                    let citation = leak(find_source(
                        r["quantity"]["source_id"]
                            .as_str()
                            .ok_or_else(|| fail(&format!("{what} has no source")))?,
                    )?);
                    match source {
                        None => source = Some(citation),
                        Some(seen) if seen != citation => {
                            return Err(fail(
                                "transition records disagree about their source; one \
                                 citation must stand behind the row the apparatus prints",
                            ))
                        }
                        Some(_) => {}
                    }
                    if boundary.is_none() {
                        boundary = r["quantity"]["conditions"]["notes"].as_str().map(leak);
                    }
                }
                Some(crate::species::PhaseTransitions {
                    melting_k: values[0],
                    boiling_k: values[1],
                    sublimation_k: values[2],
                    decomposition_k: values[3],
                    dehydration_k: values[4],
                    source: source.ok_or_else(|| fail("no transition source"))?,
                    boundary,
                })
            }
        };

        // Mirrors build.rs: the resistivity rides the schema's `Other`
        // escape and carries its own citation, not the species' general
        // provenance line.
        let electrical_resistivity = match thermo_other(key, "electrical_resistivity") {
            Some(r) => {
                let ohm_m = r["quantity"]["value"]
                    .as_f64()
                    .ok_or_else(|| fail("electrical resistivity has no value"))?;
                if r["quantity"]["unit"]["symbol"].as_str() != Some("Ohm.m") {
                    return Err(fail("electrical resistivity must be given in Ohm.m"));
                }
                Some(Resistivity {
                    ohm_m,
                    source: leak(find_source(
                        r["quantity"]["source_id"]
                            .as_str()
                            .ok_or_else(|| fail("electrical resistivity has no source"))?,
                    )?),
                    boundary: r["quantity"]["conditions"]["notes"].as_str().map(leak),
                })
            }
            None => None,
        };

        out.push(SpeciesData {
            key: leak(key),
            name: leak(identity["name"].as_str().ok_or_else(|| fail("no name"))?),
            formula: leak(comp["formula"].as_str().ok_or_else(|| fail("no formula"))?),
            inchikey: leak(identity["identifiers"]["inchikey"].as_str().unwrap_or("")),
            molar_mass,
            heat_capacity,
            density,
            standard_phase,
            appearance,
            flame_colour,
            colour,
            spectrum: spectrum_of(key, opt)?,
            dissolution_enthalpy_kj: dissolution,
            dissolves_without_speciation: param_for(key, "dissolves-without-speciation")
                .unwrap_or(0.0)
                != 0.0,
            aqueous_solubility_g_per_100_ml: param_for(key, "aqueous-solubility-g-per-100-ml"),
            aqueous_solubility_g_per_100_ml_at_100c: param_for(
                key,
                "aqueous-solubility-g-per-100-ml-at-100c",
            ),
            forms_only_above_k: param_for(key, "forms-only-above"),
            magnetic: param_for(key, "magnetic").unwrap_or(0.0) != 0.0,
            transitions,
            electrical_resistivity,
            provenance: leak(find_source(
                identity["evidence"]["source_id"]
                    .as_str()
                    .ok_or_else(|| fail("no source id"))?,
            )?),
        });
    }
    Ok(out)
}

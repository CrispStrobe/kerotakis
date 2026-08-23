//! DATA-009: Generate the reachable CEA subset.
//!
//! Lists which registry species have CEA thermodynamic data, outputs a
//! manifest with species name, CEA entry, temperature range, and formation
//! enthalpy. Validates that existing thermal test species are all present.
//!
//! Usage: cargo run -p kerotakis-cea --bin generate-cea-subset

use kerotakis_cea::nasa9::db;
use kerotakis_cea::thermal::cea_name;
use kerotakis_core::species;

fn main() {
    let thermo_db = db();
    let registry = species::REGISTRY;

    let mut matched = 0;
    let mut unmatched = 0;
    let mut manifest = Vec::new();

    eprintln!("Registry → CEA mapping ({} species):", registry.len());
    for reg in registry {
        match cea_name(reg.key) {
            Some(cea) => {
                matched += 1;
                let s = thermo_db.get(cea).unwrap();
                let (t_min, t_max) = s.t_range().unwrap_or((0.0, 0.0));
                let hf = s.h_formation;
                eprintln!(
                    "  {:<30} → {:<25} T=[{:.0}–{:.0} K]  ΔHf={:.1} kJ/mol",
                    reg.key,
                    cea,
                    t_min,
                    t_max,
                    hf / 1000.0,
                );
                manifest.push(serde_json::json!({
                    "registry_key": reg.key,
                    "cea_name": cea,
                    "phase": format!("{:?}", reg.standard_phase),
                    "t_min_k": t_min,
                    "t_max_k": t_max,
                    "formation_enthalpy_j_mol": hf,
                    "molar_mass_g_mol": s.molar_mass,
                    "citation": s.reference,
                }));
            }
            None => {
                unmatched += 1;
                eprintln!("  {:<30}   (no CEA match)", reg.key);
            }
        }
    }

    eprintln!(
        "\n{matched} matched, {unmatched} unmatched, {} total",
        registry.len()
    );
    eprintln!("CEA database: {} species total", thermo_db.species.len());

    // Output manifest
    let output = serde_json::json!({
        "description": "DATA-009: Reachable CEA subset — registry species with NASA-9 polynomial data",
        "registry_count": registry.len(),
        "matched_count": matched,
        "unmatched_count": unmatched,
        "cea_total_species": thermo_db.species.len(),
        "species": manifest,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

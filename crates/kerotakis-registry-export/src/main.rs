use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: kerotakis-registry-export OUTPUT.json")?;
    let document = kerotakis_registry_export::export_current_registry()?;
    let mut json = serde_json::to_string_pretty(&document)?;
    json.push('\n');
    fs::write(output, json)?;
    Ok(())
}

//! DATA-003: Compile a deterministic runtime pack from the source registry.
//!
//! Reads the JSON source registry, validates it, serializes to compact JSON
//! with a content-addressed header, and writes a `.pack` file.
//!
//! Usage: cargo run -p kerotakis-data --bin compile-registry \
//!            -- data/registry/registry-source-v1.json data/registry/registry.pack

use kerotakis_data::{
    serialize_pack_payload, RegistryDocument, ValidationError, PACK_MAGIC, PACK_VERSION,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: compile-registry <source.json> <output.pack> [--inspect]");
        std::process::exit(2);
    }
    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);
    let inspect = args.iter().any(|a| a == "--inspect");

    // Read and parse the source registry.
    let json = fs::read_to_string(&input_path).unwrap_or_else(|e| {
        eprintln!("could not read {}: {e}", input_path.display());
        std::process::exit(1);
    });
    let document: RegistryDocument = serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("could not parse {}: {e}", input_path.display());
        std::process::exit(1);
    });

    // Validate.
    match document.validate() {
        Ok(()) => {}
        Err(ValidationError { issues }) => {
            eprintln!("validation failed:");
            for issue in &issues {
                eprintln!("  - {}: {}", issue.path, issue.detail);
            }
            std::process::exit(1);
        }
    }

    // Serialize to bincode.
    let payload = serialize_pack_payload(&document);

    // Content hash.
    let hash = Sha256::digest(&payload);

    // Write the pack file: magic + version(u32 LE) + hash(32 bytes) + payload.
    let mut out = fs::File::create(&output_path).unwrap_or_else(|e| {
        eprintln!("could not create {}: {e}", output_path.display());
        std::process::exit(1);
    });
    out.write_all(PACK_MAGIC).unwrap();
    out.write_all(&PACK_VERSION.to_le_bytes()).unwrap();
    out.write_all(&hash).unwrap();
    out.write_all(&payload).unwrap();

    let total = 4 + 4 + 32 + payload.len();
    eprintln!(
        "compiled {} → {} ({total} bytes, {payload_kb:.1} KiB payload)",
        input_path.display(),
        output_path.display(),
        payload_kb = payload.len() as f64 / 1024.0,
    );
    eprintln!("  schema:      {}", document.schema);
    eprintln!("  sources:     {}", document.sources.len());
    eprintln!("  identities:  {}", document.identities.len());
    eprintln!("  materials:   {}", document.material_recipes.len());
    eprintln!("  compositions:{}", document.compositions.len());
    eprintln!("  phase_thermo:{}", document.phase_thermodynamics.len());
    eprintln!("  optical:     {}", document.optical.len());
    eprintln!("  model_params:{}", document.model_parameters.len());
    eprintln!("  SHA-256:     {}", hex(&hash));

    if inspect {
        println!("{}", serde_json::to_string_pretty(&document).unwrap());
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

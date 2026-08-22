//! DATA-008: Generate serialized PHREEQC database indexes at build time.
//!
//! Parses each embedded database and writes its DbIndex as JSON. The runtime
//! can load these pre-computed indexes instead of re-parsing the raw database
//! files on every startup.
//!
//! Usage: cargo run -p kerotakis-phreeqc --bin generate-dbindex -- <output-dir>

use kerotakis_phreeqc::dbindex::DbIndex;
use std::fs;
use std::path::PathBuf;

fn main() {
    let output_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "data/dbindex".into()),
    );
    fs::create_dir_all(&output_dir).unwrap();

    let databases = [
        ("phreeqc", include_bytes!("../../../../vendor/iphreeqc/database/phreeqc.dat").as_slice()),
        ("wateq4f", include_bytes!("../../../../vendor/iphreeqc/database/wateq4f.dat").as_slice()),
        ("minteq_v4", include_bytes!("../../../../vendor/iphreeqc/database/minteq.v4.dat").as_slice()),
        ("pitzer", include_bytes!("../../../../vendor/iphreeqc/database/pitzer.dat").as_slice()),
    ];

    for (name, data) in &databases {
        let index = DbIndex::parse(data);
        let json = serde_json::to_string_pretty(&index).unwrap();
        let path = output_dir.join(format!("{name}.json"));
        fs::write(&path, &json).unwrap();
        eprintln!(
            "{name}: {} masters, {} phases, {} citations → {}",
            index.masters.len(),
            index.phases.len(),
            index.citations.len(),
            path.display(),
        );
    }
    eprintln!("Done. Generated {} database indexes.", databases.len());
}

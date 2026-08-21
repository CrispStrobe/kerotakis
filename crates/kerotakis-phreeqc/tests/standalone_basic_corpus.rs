#![cfg(all(
    feature = "engine",
    any(feature = "legacy-basic-oracle", feature = "my-basic-preview")
))]

use kerotakis_phreeqc::{databases, Phreeqc};

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn normalized_user_output(output: &str) -> String {
    output
        .split("----------------------------------User print-----------------------------------")
        .nth(1)
        .unwrap_or(output)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_standalone_basic_file_matches_its_backend_oracle() {
    let programs = [
        (
            "iso.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso.bas"),
        ),
        (
            "iso1.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso1.bas"),
        ),
        (
            "iso1revised.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso1revised.bas"),
        ),
        (
            "iso2.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso2.bas"),
        ),
        (
            "iso2revised.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso2revised.bas"),
        ),
        (
            "iso3.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso3.bas"),
        ),
        (
            "iso3revised.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso3revised.bas"),
        ),
        (
            "iso4.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso4.bas"),
        ),
        (
            "iso4revised.bas",
            include_str!("../../../vendor/iphreeqc/database/isotopes/basic/iso4revised.bas"),
        ),
    ];

    let preview_expected = [
        (21, 0xe7e4ec4fee4e6f95),
        (6, 0x7abcd8d539780975),
        (72, 0x90fb594757a27dab),
        (21, 0xa88de414828188d5),
        (5, 0x52d88b33e35182ba),
        (56, 0xe190c011847dbe15),
        (134, 0x14a4d93b194b0ec4),
        (126, 0x3490192127692d7d),
        (430, 0x954fd6e1ffe321cb),
    ];
    let legacy_expected = [
        (27, 0xf5dbfb20b1affffd),
        (6, 0x7abcd8d539780975),
        (72, 0x90fb594757a27dab),
        (27, 0x3484d653182c4435),
        (5, 0x14b37be633dfcc59),
        (92, 0x5f4562b9fbc5281f),
        (134, 0x4fa8f410be42515d),
        (237, 0xd19457a55c83510f),
        (430, 0x9274fdadd32499d0),
    ];

    for (index, (name, program)) in programs.into_iter().enumerate() {
        if let Ok(filter) = std::env::var("BASIC_PROBE_FILTER") {
            if name != filter {
                continue;
            }
        }
        let mut engine = Phreeqc::with_database(databases::PHREEQC).unwrap();
        let input = format!(
            "SOLUTION 1\n    pH 7\nPRINT\n    -reset false\n    -user_print true\nUSER_PRINT\n{program}\nEND\n"
        );
        match engine.run(&input) {
            Ok(()) => {
                let output = engine.output_string();
                let normalized = normalized_user_output(&output);
                eprintln!(
                    "BASIC_CORPUS name={name} status=ok bytes={} lines={} fnv={:016x} normalized_bytes={} normalized_lines={} normalized_fnv={:016x}",
                    output.len(),
                    output.lines().count(),
                    digest(output.as_bytes()),
                    normalized.len(),
                    normalized.lines().count(),
                    digest(normalized.as_bytes()),
                );
                if std::env::var_os("BASIC_PROBE_SHOW_OUTPUT").is_some() {
                    eprintln!("BASIC_OUTPUT name={name} value={output:?}");
                }
                let expected = if cfg!(feature = "legacy-basic-oracle") {
                    legacy_expected[index]
                } else {
                    preview_expected[index]
                };
                assert_eq!(normalized.lines().count(), expected.0, "{name} line count");
                assert_eq!(
                    digest(normalized.as_bytes()),
                    expected.1,
                    "{name} output digest"
                );
            }
            Err(error) => {
                let output = engine.output_string();
                let one_line = error.to_string().replace('\n', " | ");
                eprintln!(
                    "BASIC_CORPUS name={name} status=error bytes={} lines={} fnv={:016x} message={one_line}",
                    output.len(),
                    output.lines().count(),
                    digest(output.as_bytes()),
                );
                panic!("{name} failed: {one_line}");
            }
        }
    }
}

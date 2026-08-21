#![cfg(all(
    feature = "engine",
    any(feature = "legacy-basic-oracle", feature = "my-basic-preview")
))]

use kerotakis_phreeqc::{databases, Phreeqc};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DifferenceClass {
    FormattingOnly,
    ExpectedSemanticDifference,
}

struct DifferenceFixture {
    name: &'static str,
    class: DifferenceClass,
    observable_tolerance: usize,
    preview_lines: usize,
    legacy_lines: usize,
    explanation: &'static str,
}

// These are all seven standalone entries whose preview digest differs from
// the retained development oracle. String observables use exact comparison,
// hence a zero line-count tolerance. The classification is deliberately kept
// beside the executable corpus instead of hiding backend differences in two
// unexplained digest tables.
const DIFFERENCES: &[DifferenceFixture] = &[
    DifferenceFixture {
        name: "iso.bas",
        class: DifferenceClass::ExpectedSemanticDifference,
        observable_tolerance: 0,
        preview_lines: 21,
        legacy_lines: 27,
        explanation: "MY-BASIC applies logical NOT to the parenthesized equality, so the same/different branches are exclusive; the legacy oracle emits six extra equations",
    },
    DifferenceFixture {
        name: "iso2.bas",
        class: DifferenceClass::ExpectedSemanticDifference,
        observable_tolerance: 0,
        preview_lines: 21,
        legacy_lines: 27,
        explanation: "MY-BASIC applies logical NOT to the parenthesized equality, so the same/different branches are exclusive; the legacy oracle emits six extra equations",
    },
    DifferenceFixture {
        name: "iso3.bas",
        class: DifferenceClass::ExpectedSemanticDifference,
        observable_tolerance: 0,
        preview_lines: 56,
        legacy_lines: 92,
        explanation: "the source's mutually exclusive equality branches follow standard parenthesized NOT semantics; the legacy oracle emits 36 extra equations",
    },
    DifferenceFixture {
        name: "iso4.bas",
        class: DifferenceClass::ExpectedSemanticDifference,
        observable_tolerance: 0,
        preview_lines: 126,
        legacy_lines: 237,
        explanation: "the source's mutually exclusive equality branches follow standard parenthesized NOT semantics; the legacy oracle emits 111 extra equations",
    },
    DifferenceFixture {
        name: "iso2revised.bas",
        class: DifferenceClass::FormattingOnly,
        observable_tolerance: 0,
        preview_lines: 5,
        legacy_lines: 5,
        explanation: "the four equations and final 4/4 counters agree; only comma-PRINT field spacing differs",
    },
    DifferenceFixture {
        name: "iso3revised.bas",
        class: DifferenceClass::FormattingOnly,
        observable_tolerance: 0,
        preview_lines: 134,
        legacy_lines: 134,
        explanation: "the 133 equations and final 133/216 counters agree; only comma-PRINT field spacing differs",
    },
    DifferenceFixture {
        name: "iso4revised.bas",
        class: DifferenceClass::FormattingOnly,
        observable_tolerance: 0,
        preview_lines: 430,
        legacy_lines: 430,
        explanation: "the 429 equations and final 429/1296 counters agree; only comma-PRINT field spacing differs",
    },
];

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

fn integer_fields(line: &str) -> Vec<usize> {
    line.split(|character: char| !character.is_ascii_digit())
        .filter(|field| !field.is_empty())
        .map(|field| field.parse().expect("integer output field"))
        .collect()
}

#[test]
fn every_digest_difference_has_an_explicit_classification() {
    assert_eq!(DIFFERENCES.len(), 7);
    for fixture in DIFFERENCES {
        assert_eq!(fixture.observable_tolerance, 0, "{}", fixture.name);
        assert!(!fixture.explanation.is_empty(), "{}", fixture.name);
    }
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

                if let Some(fixture) = DIFFERENCES.iter().find(|fixture| fixture.name == name) {
                    match fixture.class {
                        DifferenceClass::ExpectedSemanticDifference => {
                            let expected_equations = if cfg!(feature = "legacy-basic-oracle") {
                                fixture.legacy_lines
                            } else {
                                fixture.preview_lines
                            };
                            assert_eq!(
                                normalized.lines().count().abs_diff(expected_equations),
                                fixture.observable_tolerance,
                                "{}: {}",
                                fixture.name,
                                fixture.explanation
                            );
                        }
                        DifferenceClass::FormattingOnly => {
                            let (equations, total) = match name {
                                "iso2revised.bas" => (4, 4),
                                "iso3revised.bas" => (133, 216),
                                "iso4revised.bas" => (429, 1296),
                                _ => unreachable!(),
                            };
                            let lines = normalized.lines().collect::<Vec<_>>();
                            let expected_lines = if cfg!(feature = "legacy-basic-oracle") {
                                fixture.legacy_lines
                            } else {
                                fixture.preview_lines
                            };
                            assert_eq!(
                                lines.len().abs_diff(expected_lines),
                                fixture.observable_tolerance,
                                "{}: {}",
                                fixture.name,
                                fixture.explanation
                            );
                            assert_eq!(
                                integer_fields(lines.last().expect("counter line")),
                                [equations, total],
                                "{}: {}",
                                fixture.name,
                                fixture.explanation
                            );
                        }
                    }
                }
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

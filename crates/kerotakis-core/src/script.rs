//! The `.lab` script grammar: one line, one bench command.
//!
//! This lives in the core so that every client runs the *same* lessons —
//! the CLI, the wasm build, and anything later. A lesson is data, and its
//! grammar is part of the engine rather than of one front end.

use crate::ops::{Instrument, Operator};
use crate::species::{self, SpeciesData, SpeciesId};
use crate::units::{Grams, Joules, Kelvin, Liters, Moles, Pascal};
use crate::vessel::VesselId;

/// Parse one bench command into an operator. Meta commands (register,
/// inspect) return `None` — they are session state, not bench state.
/// The grammar's public inventory: every verb `parse_op` accepts, with a
/// canonical example line (GUI-029). A UI's affordance manifest is checked
/// against this list by the protocol conformance suite, and the test at
/// the bottom of this file keeps each example honest against the parser.
/// Aliases share their canonical verb's row.
pub const VERBS: &[(&str, &str)] = &[
    ("new", "new"),
    ("remove", "remove v1"),
    ("add", "add v1 water 100mL"),
    ("heat", "heat v1 10kJ"),
    ("cool", "cool v1 10kJ"),
    ("wait", "wait 30s"),
    ("ignite", "ignite v1"),
    ("stir", "stir v1"),
    ("seal", "seal v1 500mL"),
    ("regulate", "regulate v1 1.5bar 500mL"),
    ("sweep", "sweep v1 1bar"),
    ("open", "open v1"),
    ("filter", "filter v1 v2"),
    ("evaporate", "evaporate v1 0.5"),
    ("decant", "decant v1 v2 0.5"),
    ("drain", "drain v1 v2"),
    ("distil", "distil v1 v2 0.5"),
    ("measure", "measure v1 ph"),
    ("chromatograph", "chromatograph v1"),
    ("electrolyse", "electrolyse v1 0.5A 30min"),
    ("cell", "cell v1 v2"),
    ("grind", "grind v1 NaCl 50um"),
    ("irradiate", "irradiate v1 254nm 10W/m2"),
    ("dilute", "dilute v1 100mL"),
    ("titrate", "titrate v1 NaOH 1M 1mL until ph 7"),
    ("magnet", "magnet v1 v2"),
    ("transport", "transport v1 v2 v3 from v4 to v5 steps 3"),
    ("react", "react v1 esterification"),
    ("test", "test v1 pop"),
];

/// Stable parse failure classes for corpus coverage and clients. The legacy
/// `parse_op` API remains source-compatible; new callers should prefer
/// `parse_op_typed` when the reason is part of their data contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseErrorKind {
    UnknownSpecies,
    UnknownReaction,
    InvalidSyntax,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{detail}")]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub detail: String,
}

pub fn parse_op_typed(line: &str) -> Result<Option<Operator>, ParseError> {
    let words = line.split_whitespace().collect::<Vec<_>>();
    let kind = match words.as_slice() {
        ["add", _, species, ..]
            if species::lookup_key(species).is_none()
                && crate::nuclide::lookup_notation(species).is_none() =>
        {
            ParseErrorKind::UnknownSpecies
        }
        ["react", _, reaction, ..]
            if !crate::curated::ORG_REACTIONS
                .iter()
                .any(|candidate| candidate.name == *reaction) =>
        {
            ParseErrorKind::UnknownReaction
        }
        _ => ParseErrorKind::InvalidSyntax,
    };
    parse_op_untyped(line).map_err(|detail| ParseError { kind, detail })
}

/// Compatibility parser. Prefer [`parse_op_typed`] when callers must retain a
/// machine-readable distinction between an unknown identity and bad grammar.
pub fn parse_op(line: &str) -> Result<Option<Operator>, String> {
    parse_op_typed(line).map_err(|error| error.detail)
}

fn parse_op_untyped(line: &str) -> Result<Option<Operator>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    let op = match words[0] {
        "register" | "inspect" | "explain" | "species" | "help" | "particles" | "zoom"
        | "structure" | "identify" | "coverage" => return Ok(None),
        // `react v1 esterification` — apply a named curated organic
        // transformation. The name is checked here so a typo fails at
        // parse time, with the shelf listed.
        "react" => {
            if words.len() < 3 {
                return Err("usage: react <vessel> <reaction> (see curated::ORG_REACTIONS)".into());
            }
            let vessel = parse_vessel(words[1])?;
            let name = words[2];
            if !crate::curated::ORG_REACTIONS.iter().any(|r| r.name == name) {
                let known: Vec<&str> = crate::curated::ORG_REACTIONS
                    .iter()
                    .map(|r| r.name)
                    .collect();
                return Err(format!(
                    "unknown reaction '{name}' — curated: {}",
                    known.join(", ")
                ));
            }
            Operator::React {
                vessel,
                reaction: name.to_string(),
            }
        }
        "new" => match words.get(1) {
            None => Operator::NewVessel { kind: None },
            Some(kind) => {
                if !crate::vessel::VESSEL_KINDS.iter().any(|(k, _)| k == kind) {
                    let known: Vec<&str> = crate::vessel::VESSEL_KINDS
                        .iter()
                        .map(|(k, _)| *k)
                        .collect();
                    return Err(format!(
                        "unknown vessel kind '{kind}' — known: {}",
                        known.join(", ")
                    ));
                }
                Operator::NewVessel {
                    kind: Some((*kind).to_string()),
                }
            }
        },
        "remove" => {
            if words.len() != 2 {
                return Err("usage: remove <vessel>".into());
            }
            Operator::RemoveVessel {
                vessel: parse_vessel(words[1])?,
            }
        }
        "add" => {
            if words.len() < 4 {
                return Err("usage: add <vessel> <species> <amount><mol|g|mL> [@ <T>C]".into());
            }
            let vessel = parse_vessel(words[1])?;
            // EXP-49: El-A notation with a curated nuclide entry routes
            // to the tracer ledger, not the chemical registry.
            if crate::nuclide::lookup_notation(words[2]).is_some() {
                let amount = words[3];
                let moles = amount
                    .strip_suffix("mol")
                    .and_then(|v| v.parse::<f64>().ok())
                    .ok_or_else(|| {
                        format!(
                            "nuclide amounts are stated in moles (got '{amount}') — \
                             tracer scale, e.g. 1e-9mol"
                        )
                    })?;
                return Ok(Some(Operator::SpikeNuclide {
                    vessel,
                    nuclide: words[2].to_string(),
                    moles: Moles(moles),
                }));
            }
            let data = species::lookup_key(words[2])
                .ok_or_else(|| format!("unknown species '{}' (see 'species')", words[2]))?;
            Operator::Add {
                vessel,
                species: SpeciesId::new(words[2]),
                moles: parse_amount(words[3], data)?,
                at: parse_at(&words[4..])?,
            }
        }
        "heat" | "cool" => {
            if words.len() < 3 {
                return Err(format!("usage: {} <vessel> <energy><J|kJ>", words[0]));
            }
            let vessel = parse_vessel(words[1])?;
            let energy = parse_energy(words[2])?;
            if words[0] == "heat" {
                Operator::Heat { vessel, energy }
            } else {
                Operator::Cool { vessel, energy }
            }
        }
        "wait" => {
            // `wait 30s` — the clock the rate experiments need.
            let raw = words.get(1).ok_or("usage: wait <n><s|min|h>")?;
            let digits: String = raw
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let value: f64 = digits
                .parse()
                .map_err(|_| format!("bad duration '{raw}'"))?;
            let seconds = match raw[digits.len()..].trim() {
                "" | "s" | "sec" | "secs" | "seconds" => value,
                "min" | "mins" | "minutes" => value * 60.0,
                "h" | "hr" | "hours" => value * 3600.0,
                other => return Err(format!("unknown time unit '{other}'")),
            };
            Operator::Wait { seconds }
        }
        "ignite" => Operator::Ignite {
            vessel: parse_vessel(words.get(1).ok_or("usage: ignite <vessel>")?)?,
        },
        "stir" => Operator::Stir {
            vessel: parse_vessel(words.get(1).ok_or("usage: stir <vessel>")?)?,
        },
        "seal" => {
            if words.len() != 3 {
                return Err("usage: seal <vessel> <headspace-volume><mL|L>".into());
            }
            Operator::Seal {
                vessel: parse_vessel(words[1])?,
                headspace_volume: parse_volume(words[2])?,
            }
        }
        "regulate" => {
            if words.len() != 4 {
                return Err(
                    "usage: regulate <vessel> <pressure><Pa|kPa|bar|atm> <initial-volume><mL|L>"
                        .into(),
                );
            }
            Operator::Regulate {
                vessel: parse_vessel(words[1])?,
                pressure: parse_pressure(words[2])?,
                initial_volume: parse_volume(words[3])?,
            }
        }
        "sweep" => {
            if words.len() != 3 {
                return Err("usage: sweep <vessel> <pressure><Pa|kPa|bar|atm>".into());
            }
            Operator::Sweep {
                vessel: parse_vessel(words[1])?,
                pressure: parse_pressure(words[2])?,
            }
        }
        "open" => Operator::Open {
            vessel: parse_vessel(words.get(1).ok_or("usage: open <vessel>")?)?,
        },
        "filter" => {
            if words.len() < 3 {
                return Err("usage: filter <from> <to>".into());
            }
            Operator::Filter {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
            }
        }
        "magnet" => {
            if words.len() < 3 {
                return Err("usage: magnet <from> <to>".into());
            }
            Operator::Magnet {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
            }
        }
        "evaporate" => {
            if words.len() < 3 {
                return Err("usage: evaporate <vessel> <fraction>".into());
            }
            Operator::Evaporate {
                vessel: parse_vessel(words[1])?,
                fraction: words[2]
                    .parse()
                    .map_err(|_| format!("bad fraction '{}'", words[2]))?,
            }
        }
        "decant" => {
            if words.len() < 4 {
                return Err("usage: decant <from> <to> <fraction>".into());
            }
            Operator::Decant {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
                fraction: words[3]
                    .parse()
                    .map_err(|_| format!("bad fraction '{}'", words[3]))?,
            }
        }
        "drain" => {
            if words.len() < 3 {
                return Err("usage: drain <from> <to>".into());
            }
            Operator::Drain {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
            }
        }
        "distil" | "distill" => {
            if words.len() < 4 {
                return Err(
                    "usage: distil <from> <to> <fraction | energy J|kJ> [stages <n>]".into(),
                );
            }
            let (fraction, energy) = if let Some(kj) = words[3].strip_suffix("kJ") {
                let v: f64 = kj
                    .parse()
                    .map_err(|_| format!("bad energy '{}'", words[3]))?;
                (None, Some(Joules(v * 1000.0)))
            } else if let Some(j) = words[3].strip_suffix('J') {
                let v: f64 = j
                    .parse()
                    .map_err(|_| format!("bad energy '{}'", words[3]))?;
                (None, Some(Joules(v)))
            } else {
                let f: f64 = words[3]
                    .parse()
                    .map_err(|_| format!("bad fraction '{}'", words[3]))?;
                (Some(f), None)
            };
            let stages = match (words.get(4), words.get(5)) {
                (Some(&"stages"), Some(n)) => {
                    n.parse().map_err(|_| format!("bad stage count '{n}'"))?
                }
                (None, _) => 1,
                _ => return Err("after the amount, only `stages <n>` may follow".into()),
            };
            Operator::Distil {
                from: parse_vessel(words[1])?,
                to: parse_vessel(words[2])?,
                fraction,
                energy,
                stages,
            }
        }
        // `look v1` — the youngest interaction there is.
        "look" | "observe" => Operator::Measure {
            vessel: parse_vessel(words.get(1).copied().unwrap_or("v1"))?,
            instrument: Instrument::Eyes,
        },
        "measure" => {
            if words.len() < 3 {
                return Err("usage: measure <vessel> <thermometer|balance|ph>".into());
            }
            Operator::Measure {
                vessel: parse_vessel(words[1])?,
                instrument: match words[2] {
                    "thermometer" | "temp" => Instrument::Thermometer,
                    "balance" | "mass" => Instrument::Balance,
                    "ph" | "phmeter" => Instrument::PhMeter,
                    "eyes" | "look" => Instrument::Eyes,
                    "pressure" | "gauge" => Instrument::PressureGauge,
                    "volume" => Instrument::VolumeMeter,
                    "conductivity" => Instrument::ConductivityMeter,
                    "spectrophotometer" | "uvvis" => Instrument::Spectrophotometer,
                    "calorimeter" => Instrument::Calorimeter,
                    "chromatograph" | "column" => Instrument::Chromatograph,
                    "geiger" => Instrument::GeigerCounter,
                    other => return Err(format!("unknown instrument '{other}'")),
                },
            }
        }
        // `smell v1` — waft, never huff. The taught technique is the verb.
        "smell" | "waft" => Operator::Smell {
            vessel: parse_vessel(words.get(1).copied().unwrap_or("v1"))?,
        },
        // `test v1 pop` — apply a classical gas test to the headspace.
        "test" => {
            let vessel = parse_vessel(words.get(1).copied().unwrap_or("v1"))?;
            let test_name = words
                .get(2)
                .copied()
                .ok_or("usage: test <vessel> pop|splint|limewater|litmus")?;
            let test = match test_name {
                "pop" => crate::gas_tests::GasTest::Pop,
                "splint" => crate::gas_tests::GasTest::GlowingSplint,
                "limewater" => crate::gas_tests::GasTest::Limewater,
                "litmus" => crate::gas_tests::GasTest::DampLitmus,
                _ => {
                    return Err(format!(
                        "unknown gas test '{test_name}' — options: pop, splint, limewater, litmus"
                    ));
                }
            };
            Operator::TestGas { vessel, test }
        }
        // `chromatograph v1` — inject the solution onto the column and
        // read the peak table. Sugar for `measure v1 chromatograph`,
        // first-class because running a separation is a verb in any lab.
        "chromatograph" => Operator::Measure {
            vessel: parse_vessel(words.get(1).copied().unwrap_or("v1"))?,
            instrument: Instrument::Chromatograph,
        },
        // `cell v1 v2` — touch the wires of two half-cells together and
        // read the voltmeter. Nothing flows; the reading is the prediction.
        "electrolyse" | "electrolyze" => {
            // `electrolyse v1 0.5A 600s` — a current and a clock, which is
            // exactly what the practical gives you.
            if words.len() < 4 {
                return Err("usage: electrolyse <vessel> <current>A <time><s|min|h>".into());
            }
            let vessel = parse_vessel(words[1])?;
            let amps = parse_suffixed(words[2], &[("a", 1.0), ("ma", 1e-3), ("", 1.0)], "current")?;
            let seconds = parse_suffixed(
                words[3],
                &[
                    ("s", 1.0),
                    ("sec", 1.0),
                    ("secs", 1.0),
                    ("seconds", 1.0),
                    ("min", 60.0),
                    ("mins", 60.0),
                    ("minutes", 60.0),
                    ("h", 3600.0),
                    ("hr", 3600.0),
                    ("hours", 3600.0),
                    ("", 1.0),
                ],
                "time",
            )?;
            Operator::Electrolyse {
                vessel,
                amps,
                seconds,
            }
        }
        "cell" | "voltmeter" => {
            if words.len() < 3 {
                return Err("usage: cell <vessel> <vessel>".into());
            }
            Operator::Cell {
                a: parse_vessel(words[1])?,
                b: parse_vessel(words[2])?,
            }
        }
        "grind" => {
            // `grind v1 NaCl 50um` — set particle size for heterogeneous rates
            if words.len() < 4 {
                return Err("usage: grind <vessel> <species> <diameter>um".into());
            }
            let vessel = parse_vessel(words[1])?;
            let species_key = words[2];
            let _ = species::lookup_key(species_key)
                .ok_or_else(|| format!("unknown species '{species_key}'"))?;
            let diameter = parse_suffixed(
                words[3],
                &[("um", 1.0), ("μm", 1.0), ("mm", 1000.0), ("", 1.0)],
                "diameter",
            )?;
            Operator::Grind {
                vessel,
                species: SpeciesId::new(species_key),
                diameter_um: diameter,
            }
        }
        "irradiate" => {
            // `irradiate v1 254nm 10W/m2` — turn on UV lamp
            if words.len() < 4 {
                return Err("usage: irradiate <vessel> <wavelength>nm <irradiance>W/m2".into());
            }
            let vessel = parse_vessel(words[1])?;
            let wavelength = parse_suffixed(words[2], &[("nm", 1.0), ("", 1.0)], "wavelength")?;
            let irradiance = parse_suffixed(words[3], &[("w/m2", 1.0), ("", 1.0)], "irradiance")?;
            Operator::Irradiate {
                vessel,
                wavelength_nm: wavelength,
                irradiance_w_m2: irradiance,
            }
        }
        "dilute" => {
            if words.len() < 3 {
                return Err("usage: dilute <vessel> <volume><mL|L>".into());
            }
            Operator::Dilute {
                vessel: parse_vessel(words[1])?,
                volume: parse_volume(words[2])?,
            }
        }
        "titrate" => {
            // titrate v1 NaOH 1mL until ph 7          (1 mol/L standard)
            // titrate v1 NaOH 0.1M 1mL until ph 7 max 200
            //
            // The burette holds a *standard solution*, not the pure
            // substance: `<c>M` states its concentration, defaulting to
            // 1 mol/L — the convention every titration practical prints
            // on the bottle. (Delivering pure titrant by volume would
            // dose ~50× per mL for NaOH and leap the whole curve in one
            // step, which is what this grammar replaced.)
            if words.len() < 7 {
                return Err(
                    "usage: titrate <vessel> <titrant> [<c>M] <step><mL|L> until ph <target> [max <n>]"
                        .into(),
                );
            }
            let vessel = parse_vessel(words[1])?;
            let titrant_key = words[2];
            let _ = species::lookup_key(titrant_key)
                .ok_or_else(|| format!("unknown species '{titrant_key}' (see 'species')"))?;
            let (concentration, rest) = match words[3].strip_suffix(['M', 'm']) {
                Some(c) if c.parse::<f64>().is_ok() => (c.parse::<f64>().unwrap(), &words[4..]),
                _ => (1.0, &words[3..]),
            };
            if concentration <= 0.0 {
                return Err("titrant concentration must be positive".into());
            }
            if rest.len() < 4 {
                return Err(
                    "usage: titrate <vessel> <titrant> [<c>M] <step><mL|L> until ph <target> [max <n>]"
                        .into(),
                );
            }
            let step = parse_volume(rest[0])?;
            if rest[1] != "until" || rest[2] != "ph" {
                return Err(
                    "usage: titrate <vessel> <titrant> [<c>M] <step> until ph <target> [max <n>]"
                        .into(),
                );
            }
            let target_ph: f64 = rest[3]
                .parse()
                .map_err(|_| format!("bad pH target '{}'", rest[3]))?;
            let max_steps = match (rest.get(4), rest.get(5)) {
                (Some(&"max"), Some(n)) => {
                    n.parse().map_err(|_| format!("bad max step count '{n}'"))?
                }
                (None, _) => 100,
                _ => return Err("after the pH target, only `max <n>` may follow".into()),
            };
            Operator::Titrate {
                vessel,
                titrant: SpeciesId::new(titrant_key),
                concentration,
                step,
                target_ph,
                max_steps,
            }
        }
        "mix" => {
            // mix v1 0.5 v2 0.5 into v3
            if words.len() < 7 {
                return Err(
                    "usage: mix <vessel-a> <frac-a> <vessel-b> <frac-b> into <target>".into(),
                );
            }
            let a = parse_vessel(words[1])?;
            let fraction_a: f64 = words[2]
                .parse()
                .map_err(|_| format!("bad fraction '{}'", words[2]))?;
            let b = parse_vessel(words[3])?;
            let fraction_b: f64 = words[4]
                .parse()
                .map_err(|_| format!("bad fraction '{}'", words[4]))?;
            if words[5] != "into" {
                return Err(
                    "usage: mix <vessel-a> <frac-a> <vessel-b> <frac-b> into <target>".into(),
                );
            }
            let into = parse_vessel(words[6])?;
            Operator::Mix {
                a,
                b,
                into,
                fraction_a,
                fraction_b,
            }
        }
        "transport" => {
            // transport v1 v2 v3 from v4 to v5 steps 5 [courant 0.5]
            let from_pos = words.iter().position(|&w| w == "from");
            let to_pos = words.iter().position(|&w| w == "to");
            let steps_pos = words.iter().position(|&w| w == "steps");
            let (from_pos, to_pos, steps_pos) = match (from_pos, to_pos, steps_pos) {
                (Some(f), Some(t), Some(s)) => (f, t, s),
                _ => {
                    return Err(
                        "usage: transport <v1> [v2 ...] from <inlet> to <receiver> steps <n> [courant <f>]"
                            .into(),
                    )
                }
            };
            if from_pos < 2 {
                return Err("transport needs at least one cell vessel before 'from'".into());
            }
            let chain: Vec<VesselId> = words[1..from_pos]
                .iter()
                .map(|w| parse_vessel(w))
                .collect::<Result<_, _>>()?;
            let inlet = parse_vessel(
                words
                    .get(from_pos + 1)
                    .ok_or("expected inlet vessel after 'from'")?,
            )?;
            let receiver = parse_vessel(
                words
                    .get(to_pos + 1)
                    .ok_or("expected receiver vessel after 'to'")?,
            )?;
            let steps: u32 = words
                .get(steps_pos + 1)
                .ok_or("expected step count after 'steps'")?
                .parse()
                .map_err(|_| {
                    format!(
                        "bad step count '{}'",
                        words.get(steps_pos + 1).unwrap_or(&"")
                    )
                })?;
            let courant_pos = words.iter().position(|&w| w == "courant");
            let courant: f64 = match courant_pos {
                Some(cp) => words
                    .get(cp + 1)
                    .ok_or("expected Courant fraction after 'courant'")?
                    .parse()
                    .map_err(|_| {
                        format!(
                            "bad Courant fraction '{}'",
                            words.get(cp + 1).unwrap_or(&"")
                        )
                    })?,
                None => 1.0,
            };
            Operator::Transport {
                chain,
                inlet,
                receiver,
                steps,
                courant,
            }
        }
        other => return Err(format!("unknown command '{other}' (try 'help')")),
    };
    Ok(Some(op))
}

pub fn parse_vessel(word: &str) -> Result<VesselId, String> {
    let digits = word.trim_start_matches('v');
    let n: usize = digits
        .parse()
        .map_err(|_| format!("bad vessel '{word}' (use v1, v2, …)"))?;
    if n == 0 {
        return Err("vessels are numbered from v1".into());
    }
    Ok(VesselId(n - 1))
}

/// `0.5mol`, `10g`, `100mL` (unit required, so units are never guessed).
pub fn parse_amount(word: &str, data: &SpeciesData) -> Result<Moles, String> {
    let (value, unit) = split_unit(word)?;
    match unit {
        "mol" => Ok(Moles(value)),
        // Household amounts. A child does not weigh things in grams, and
        // demanding they do is the fastest way to lose them. These are
        // ordinary kitchen measures, stated as such.
        "spoon" | "spoons" | "tsp" => Ok(data.moles_from_grams(Grams(value * 5.0))),
        "pinch" | "pinches" => Ok(data.moles_from_grams(Grams(value * 0.3))),
        "cup" | "cups" => Ok(data.moles_from_liters(Liters(value * 0.25))),
        "splash" | "splashes" => Ok(data.moles_from_liters(Liters(value * 0.02))),
        "drop" | "drops" => Ok(data.moles_from_liters(Liters(value * 0.00005))),
        "g" => Ok(data.moles_from_grams(Grams(value))),
        "mL" | "ml" => Ok(data.moles_from_liters(Liters(value / 1000.0))),
        "L" | "l" => Ok(data.moles_from_liters(Liters(value))),
        other => Err(format!(
            "unknown amount '{other}' — try g, mL, L, mol, or a kitchen measure: spoon, pinch, cup, splash, drop"
        )),
    }
}

pub fn parse_energy(word: &str) -> Result<Joules, String> {
    let (value, unit) = split_unit(word)?;
    match unit {
        "J" | "j" => Ok(Joules(value)),
        "kJ" | "kj" => Ok(Joules(value * 1000.0)),
        other => Err(format!("unknown energy unit '{other}' (J, kJ)")),
    }
}

pub fn parse_volume(word: &str) -> Result<Liters, String> {
    let (value, unit) = split_unit(word)?;
    if value <= 0.0 {
        return Err("headspace volume must be positive".into());
    }
    match unit {
        "mL" | "ml" => Ok(Liters(value / 1000.0)),
        "L" | "l" => Ok(Liters(value)),
        other => Err(format!("unknown volume unit '{other}' (mL, L)")),
    }
}

pub fn parse_pressure(word: &str) -> Result<Pascal, String> {
    let (value, unit) = split_unit(word)?;
    if value <= 0.0 {
        return Err("pressure must be positive".into());
    }
    match unit {
        "Pa" | "pa" => Ok(Pascal(value)),
        "kPa" | "kpa" => Ok(Pascal(value * 1_000.0)),
        "bar" => Ok(Pascal(value * 100_000.0)),
        "atm" => Ok(Pascal(value * Pascal::ATMOSPHERIC.0)),
        other => Err(format!(
            "unknown pressure unit '{other}' (Pa, kPa, bar, atm)"
        )),
    }
}

/// Optional trailing `@ 60C` / `@ 333K` on `add`.
pub fn parse_at(words: &[&str]) -> Result<Option<Kelvin>, String> {
    match words {
        [] => Ok(None),
        ["@", t] => {
            let (value, unit) = split_unit(t)?;
            match unit {
                "C" | "c" => Ok(Some(Kelvin::from_celsius(value))),
                "K" | "k" => Ok(Some(Kelvin(value))),
                other => Err(format!("unknown temperature unit '{other}' (C, K)")),
            }
        }
        _ => Err("temperature goes last: … @ 60C".into()),
    }
}

fn split_unit(word: &str) -> Result<(f64, &str), String> {
    let split = word
        .find(|c: char| c.is_ascii_alphabetic())
        .ok_or_else(|| format!("'{word}' needs a unit suffix"))?;
    let value: f64 = word[..split]
        .parse()
        .map_err(|_| format!("bad number in '{word}'"))?;
    Ok((value, &word[split..]))
}

/// A number with a unit suffix, matched longest-first so `ms` cannot be
/// read as `m`. Shared by the operators that take a physical quantity.
fn parse_suffixed(raw: &str, units: &[(&str, f64)], what: &str) -> Result<f64, String> {
    let digits: String = raw
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let value: f64 = digits.parse().map_err(|_| format!("bad {what} '{raw}'"))?;
    let suffix = raw[digits.len()..].trim().to_ascii_lowercase();
    let mut best: Option<f64> = None;
    for (name, scale) in units {
        if suffix == *name {
            best = Some(*scale);
            break;
        }
    }
    match best {
        Some(scale) if value > 0.0 => Ok(value * scale),
        Some(_) => Err(format!("{what} must be positive")),
        None => Err(format!("unknown {what} unit '{suffix}'")),
    }
}

#[cfg(test)]
mod grammar_inventory {
    use super::*;

    /// Every inventory row's example must parse to an operator, and its
    /// first word must be the row's verb — the inventory cannot claim a
    /// grammar the parser does not have.
    #[test]
    fn every_verb_example_parses() {
        for (verb, example) in VERBS {
            assert_eq!(
                example.split_whitespace().next(),
                Some(*verb),
                "inventory row '{verb}' must exemplify its own verb"
            );
            match parse_op(example) {
                Ok(Some(_)) => {}
                other => panic!("VERBS example '{example}' did not parse: {other:?}"),
            }
        }
    }

    /// Glassware kinds parse into the vessel label, and an unknown kind
    /// is refused with the list.
    #[test]
    fn vessel_kinds_parse_and_unknowns_refuse() {
        match parse_op("new tube") {
            Ok(Some(Operator::NewVessel { kind: Some(k) })) => assert_eq!(k, "tube"),
            other => panic!("new tube: {other:?}"),
        }
        let err = parse_op("new saucepan").unwrap_err();
        assert!(err.contains("beaker"), "refusal lists kinds: {err}");
    }

    /// The inventory is unique and non-trivial.
    #[test]
    fn the_inventory_is_well_formed() {
        let mut seen = std::collections::HashSet::new();
        for (verb, _) in VERBS {
            assert!(seen.insert(verb), "duplicate inventory verb '{verb}'");
        }
        assert!(
            VERBS.len() >= 25,
            "the inventory lost verbs: {}",
            VERBS.len()
        );
    }

    #[test]
    fn typed_errors_distinguish_identity_reaction_and_grammar_gaps() {
        assert_eq!(
            parse_op_typed("add v1 dragon-slime 1g").unwrap_err().kind,
            ParseErrorKind::UnknownSpecies
        );
        assert_eq!(
            parse_op_typed("react v1 transmutation").unwrap_err().kind,
            ParseErrorKind::UnknownReaction
        );
        assert_eq!(
            parse_op_typed("heat v1 eventually").unwrap_err().kind,
            ParseErrorKind::InvalidSyntax
        );
        assert!(matches!(
            parse_op_typed("add v1 water 10mL"),
            Ok(Some(Operator::Add { .. }))
        ));
    }
}

//! The `.lab` script grammar: one line, one bench command.
//!
//! This lives in the core so that every client runs the *same* lessons —
//! the CLI, the wasm build, and anything later. A lesson is data, and its
//! grammar is part of the engine rather than of one front end.

use crate::ops::{Instrument, Operator};
use crate::species::{self, SpeciesData, SpeciesId};
use crate::units::{Grams, Joules, Kelvin, Liters, Moles};
use crate::vessel::VesselId;

/// Parse one bench command into an operator. Meta commands (register,
/// inspect) return `None` — they are session state, not bench state.
pub fn parse_op(line: &str) -> Result<Option<Operator>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    let op = match words[0] {
        "register" | "inspect" | "explain" | "species" | "help" | "particles" | "zoom" => {
            return Ok(None)
        }
        "new" => Operator::NewVessel,
        "add" => {
            if words.len() < 4 {
                return Err("usage: add <vessel> <species> <amount><mol|g|mL> [@ <T>C]".into());
            }
            let vessel = parse_vessel(words[1])?;
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
        "filter" => {
            if words.len() < 3 {
                return Err("usage: filter <from> <to>".into());
            }
            Operator::Filter {
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
                    other => return Err(format!("unknown instrument '{other}'")),
                },
            }
        }
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

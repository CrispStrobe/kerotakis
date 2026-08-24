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
pub fn parse_op(line: &str) -> Result<Option<Operator>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    let op = match words[0] {
        "register" | "inspect" | "explain" | "species" | "help" | "particles" | "zoom"
        | "structure" | "identify" | "react" | "coverage" => return Ok(None),
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
                    other => return Err(format!("unknown instrument '{other}'")),
                },
            }
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
            // titrate v1 NaOH 1mL until ph 7
            // titrate v1 NaOH 1mL until ph 7 max 200
            if words.len() < 7 {
                return Err(
                    "usage: titrate <vessel> <titrant> <step><mL|L> until ph <target> [max <n>]"
                        .into(),
                );
            }
            let vessel = parse_vessel(words[1])?;
            let titrant_key = words[2];
            let _ = species::lookup_key(titrant_key)
                .ok_or_else(|| format!("unknown species '{titrant_key}' (see 'species')"))?;
            let step = parse_volume(words[3])?;
            if words[4] != "until" || words[5] != "ph" {
                return Err(
                    "usage: titrate <vessel> <titrant> <step> until ph <target> [max <n>]".into(),
                );
            }
            let target_ph: f64 = words[6]
                .parse()
                .map_err(|_| format!("bad pH target '{}'", words[6]))?;
            let max_steps = match (words.get(7), words.get(8)) {
                (Some(&"max"), Some(n)) => {
                    n.parse().map_err(|_| format!("bad max step count '{n}'"))?
                }
                (None, _) => 100,
                _ => return Err("after the pH target, only `max <n>` may follow".into()),
            };
            Operator::Titrate {
                vessel,
                titrant: SpeciesId::new(titrant_key),
                step,
                target_ph,
                max_steps,
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

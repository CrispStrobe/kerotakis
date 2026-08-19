//! `kero` — the Kerotakis bench as a REPL, batch runner and JSON interface.
//!
//! The CLI consumes `kerotakis-core` only through its public API; `--json`
//! output is the API contract shared with the future wasm/mobile clients
//! (PLAN.md, "CLI first").
//!
//! Usage:
//!   kero                      interactive session
//!   kero run FILE.lab         replay a command script
//!   kero run FILE.lab --json  replay, one JSON object per step on stdout
//!   kero species              list the registry

use std::io::{BufRead, Write};

use kerotakis_core::*;

struct Session {
    bench: Bench,
    register: Register,
    json: bool,
    stack: SolverStack,
}

/// Physics + aqueous chemistry + honesty. If the PHREEQC engine cannot be
/// initialised the session still works, honestly degraded.
fn build_stack() -> SolverStack {
    let mut solvers: Vec<Box<dyn Equilibrator>> =
        vec![Box::new(MixingEquilibrator), Box::new(CuratedEquilibrator)];
    match kerotakis_phreeqc::PhreeqcEquilibrator::new() {
        Ok(aqueous) => solvers.push(Box::new(aqueous)),
        Err(e) => eprintln!("kero: aqueous engine unavailable ({e}); running without it"),
    }
    solvers.push(Box::new(HonestyEquilibrator));
    SolverStack::new(solvers)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") => {
            let path = args.get(1).unwrap_or_else(|| usage());
            let json = args.iter().any(|a| a == "--json");
            let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("kero: cannot read {path}: {e}");
                std::process::exit(1);
            });
            let mut session = Session {
                bench: Bench::new(),
                register: Register::Student,
                json,
                stack: build_stack(),
            };
            for (lineno, line) in text.lines().enumerate() {
                if let Err(e) = session.exec_line(line) {
                    eprintln!("kero: {path}:{}: {e}", lineno + 1);
                    std::process::exit(1);
                }
            }
        }
        Some("prewarm") => {
            // Build-time: replay lesson scripts through the real engine and
            // export every solver result, so guided content never waits for
            // an engine on device (PLAN.md: pre-warmed cache).
            let out = args
                .iter()
                .position(|a| a == "-o" || a == "--out")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "cache.postcard".to_string());
            let files: Vec<&String> = args[1..].iter().filter(|a| a.ends_with(".lab")).collect();
            if files.is_empty() {
                eprintln!("kero prewarm: no .lab files given");
                std::process::exit(2);
            }
            let mut engine = kerotakis_phreeqc::PhreeqcEquilibrator::new().unwrap_or_else(|e| {
                eprintln!("kero prewarm: aqueous engine unavailable: {e}");
                std::process::exit(1);
            });
            let mut steps = 0usize;
            for file in &files {
                let text = std::fs::read_to_string(file).unwrap_or_else(|e| {
                    eprintln!("kero prewarm: cannot read {file}: {e}");
                    std::process::exit(1);
                });
                let mut bench = Bench::new();
                for line in text.lines() {
                    match parse_op(line) {
                        Ok(Some(op)) => {
                            bench
                                .step_with(op, &mut engine, &kerotakis_safety::ReactiveGroupScreen)
                                .ok();
                            steps += 1;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("kero prewarm: {file}: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
            let data = engine.export_cache();
            let bytes = postcard::to_allocvec(&data).expect("serialise cache");
            std::fs::write(&out, &bytes).unwrap_or_else(|e| {
                eprintln!("kero prewarm: cannot write {out}: {e}");
                std::process::exit(1);
            });
            println!(
                "pre-warmed {} solver results from {steps} steps across {} lessons → {out} ({} bytes)",
                data.entries.len(),
                files.len(),
                bytes.len()
            );
        }
        Some("species") => {
            for s in species::REGISTRY {
                println!(
                    "{:<10} {:<18} {:<8} M={:>8.3} g/mol   [{}]",
                    s.key, s.name, s.formula, s.molar_mass, s.provenance
                );
            }
        }
        Some("help") | Some("--help") | Some("-h") => {
            usage();
        }
        Some(other) => {
            eprintln!("kero: unknown command '{other}' (try 'kero help')");
            std::process::exit(2);
        }
        None => repl(),
    }
}

fn usage() -> ! {
    eprintln!(
        "kerotakis — a virtual laboratory that computes real chemistry\n\
         \n\
         usage:\n\
         \x20 kero                       interactive bench\n\
         \x20 kero run FILE.lab [--json] replay a command script\n\
         \x20 kero species               list known species\n\
         \n\
         bench commands (REPL and .lab files):\n\
         \x20 add <vessel> <species> <amount><mol|g|mL> [@ <T>C]\n\
         \x20 heat <vessel> <energy><J|kJ>\n\
         \x20 cool <vessel> <energy><J|kJ>\n\
         \x20 stir <vessel>\n\
         \x20 decant <from> <to> <fraction>\n\
         \x20 filter <from> <to>         solids stay, liquid passes\n\
         \x20 evaporate <vessel> <fraction>\n\
         \x20 measure <vessel> <thermometer|balance|ph>\n\
         \x20 new                        create a vessel\n\
         \x20 inspect [vessel]           show state\n\
         \x20 register <9|15|expert>     switch rendering register\n\
         \x20 quit"
    );
    std::process::exit(2);
}

fn repl() {
    println!("kerotakis 0.0.1 — the bench is ready. 'help' lists commands.");
    let mut session = Session {
        bench: Bench::new(),
        register: Register::Student,
        json: false,
        stack: build_stack(),
    };
    let stdin = std::io::stdin();
    loop {
        print!("kero> ");
        std::io::stdout().flush().ok();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let line = line.trim();
        if line == "quit" || line == "exit" {
            break;
        }
        if line == "help" {
            println!(
                "add <v> <species> <amount><mol|g|mL> [@ <T>C] · heat/cool <v> <E><J|kJ>\n\
                 stir <v> · decant/filter <from> <to> · evaporate <v> <frac> · measure <v> <thermometer|balance|ph>\n\
                 new · inspect [v] · register <9|15|expert> · species · quit"
            );
            continue;
        }
        if line == "species" {
            for s in species::REGISTRY {
                println!("  {:<10} {} ({})", s.key, s.name, s.formula);
            }
            continue;
        }
        if let Err(e) = session.exec_line(line) {
            println!("  ! {e}");
        }
    }
}

impl Session {
    fn exec_line(&mut self, line: &str) -> Result<(), String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        match words[0] {
            "register" => {
                self.register = match words.get(1).copied() {
                    Some("9") | Some("child") => Register::Child,
                    Some("15") | Some("student") => Register::Student,
                    Some("expert") => Register::Expert,
                    other => return Err(format!("unknown register {other:?}")),
                };
                Ok(())
            }
            "inspect" => {
                let target = words.get(1).map(|w| parse_vessel(w)).transpose()?;
                for v in &self.bench.vessels {
                    if target.is_some() && target != Some(v.id) {
                        continue;
                    }
                    self.print_vessel(v);
                }
                Ok(())
            }
            _ => match parse_op(trimmed)? {
                Some(op) => self.run_op(op),
                None => Ok(()),
            },
        }
    }

    fn run_op(&mut self, op: Operator) -> Result<(), String> {
        let events = self
            .bench
            .step_with(
                op.clone(),
                &mut self.stack,
                &kerotakis_safety::ReactiveGroupScreen,
            )
            .map_err(|e| e.to_string())?;
        if self.json {
            let step = serde_json::json!({
                "step": self.bench.log.len() - 1,
                "operator": op,
                "events": events,
                "bench": { "vessels": self.bench.vessels },
            });
            println!("{step}");
        } else {
            for event in &events {
                println!("  {}", render_event(event, self.register));
            }
        }
        Ok(())
    }

    fn print_vessel(&self, v: &Vessel) {
        let solution = v
            .solution
            .as_ref()
            .map(|s| format!(", pH {:.2}, I = {:.4} m", s.ph, s.ionic_strength))
            .unwrap_or_default();
        println!(
            "  {} ({}) — {:.2} °C, {:.1} g, {:.1} mL liquid{solution}",
            v.id,
            v.label,
            v.temperature.to_celsius(),
            v.mass().0 + 0.0, // + 0.0 normalises negative zero
            v.liquid_volume().0 * 1000.0 + 0.0
        );
        for p in &v.contents {
            let name = species::lookup(&p.species)
                .map(|d| d.name)
                .unwrap_or(p.species.0.as_str());
            println!("      {:>10.4} mol  {:<18} {:?}", p.moles.0, name, p.phase);
        }
        if v.is_empty() {
            println!("      (empty)");
        }
        // Expert register: the true equilibrium speciation.
        if self.register == Register::Expert {
            if let Some(info) = &v.solution {
                if !info.species.is_empty() {
                    println!("      speciation (mol/kgw · activity · γ):");
                    for sp in &info.species {
                        let gamma = if sp.molality > 0.0 {
                            sp.activity / sp.molality
                        } else {
                            0.0
                        };
                        println!(
                            "        {:<12} {:>12.4e} {:>12.4e}   γ={:.3}",
                            sp.name, sp.molality, sp.activity, gamma
                        );
                    }
                }
            }
        }
    }
}

/// Parse one bench command into an operator. Meta commands (register,
/// inspect) return `None` — they are session state, not bench state. This
/// is the single source of truth for the `.lab` grammar, shared by the
/// REPL, the batch runner and the pre-warmer.
fn parse_op(line: &str) -> Result<Option<Operator>, String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let words: Vec<&str> = line.split_whitespace().collect();
    let op = match words[0] {
        "register" | "inspect" | "species" | "help" => return Ok(None),
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
                    other => return Err(format!("unknown instrument '{other}'")),
                },
            }
        }
        other => return Err(format!("unknown command '{other}' (try 'help')")),
    };
    Ok(Some(op))
}

fn parse_vessel(word: &str) -> Result<VesselId, String> {
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
fn parse_amount(word: &str, data: &species::SpeciesData) -> Result<Moles, String> {
    let (value, unit) = split_unit(word)?;
    match unit {
        "mol" => Ok(Moles(value)),
        "g" => Ok(data.moles_from_grams(Grams(value))),
        "mL" | "ml" => Ok(data.moles_from_liters(Liters(value / 1000.0))),
        "L" | "l" => Ok(data.moles_from_liters(Liters(value))),
        other => Err(format!("unknown amount unit '{other}' (mol, g, mL, L)")),
    }
}

fn parse_energy(word: &str) -> Result<Joules, String> {
    let (value, unit) = split_unit(word)?;
    match unit {
        "J" | "j" => Ok(Joules(value)),
        "kJ" | "kj" => Ok(Joules(value * 1000.0)),
        other => Err(format!("unknown energy unit '{other}' (J, kJ)")),
    }
}

/// Optional trailing `@ 60C` / `@ 333K` on `add`.
fn parse_at(words: &[&str]) -> Result<Option<Kelvin>, String> {
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

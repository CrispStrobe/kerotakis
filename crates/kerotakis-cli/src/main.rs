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

use kerotakis_core::script::{parse_op, parse_vessel};
use kerotakis_core::*;

struct Session {
    bench: Bench,
    register: Register,
    json: bool,
    stack: SolverStack,
    /// A second engine instance used only for `explain`'s path comparison,
    /// so comparing never disturbs the session's own solver state.
    paths: Option<kerotakis_phreeqc::PhreeqcEquilibrator>,
}

/// Physics + aqueous chemistry + honesty. If the PHREEQC engine cannot be
/// initialised the session still works, honestly degraded.
fn build_stack() -> SolverStack {
    let mut solvers: Vec<Box<dyn Equilibrator>> = vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_cea::ThermalEquilibrator),
    ];
    match kerotakis_phreeqc::PhreeqcEquilibrator::new() {
        Ok(aqueous) => solvers.push(Box::new(aqueous)),
        Err(e) => eprintln!("kero: aqueous engine unavailable ({e}); running without it"),
    }
    // After the aqueous engine: where a solution freezes depends on how
    // many particles are dissolved in it, and only the speciation knows.
    solvers.push(Box::new(kerotakis_core::StateEquilibrator));
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
                register: Register::default(),
                json,
                stack: build_stack(),
                paths: kerotakis_phreeqc::PhreeqcEquilibrator::new().ok(),
            };
            for (lineno, line) in text.lines().enumerate() {
                if let Err(e) = session.exec_line(line) {
                    eprintln!("kero: {path}:{}: {e}", lineno + 1);
                    std::process::exit(1);
                }
            }
        }
        Some("codex") => {
            let sub = args.get(1).map(String::as_str).unwrap_or("lint");
            let dir = args
                .iter()
                .position(|a| a == "--dir")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "codex".to_string());
            match sub {
                "lint" => codex_lint(&dir),
                "concepts" => codex_concepts(&dir),
                "gaps" => codex_gaps(&dir),
                other => {
                    eprintln!("kero codex: unknown subcommand '{other}' (lint, concepts, gaps)");
                    std::process::exit(2);
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
        Some("balance") => {
            // Balancing is the null space of the element-count matrix, so
            // the lab can *do* it rather than check a memorised answer —
            // which makes "balance this equation" an exercise with the
            // engine as the arbiter.
            let Some(equation) = args.get(1) else {
                eprintln!("usage: kero balance \"Mg + O2 -> MgO\"");
                std::process::exit(2);
            };
            let arrows = ["->", "→", "=", "⇌"];
            let Some((l, r)) = arrows.iter().find_map(|a| equation.split_once(a)) else {
                eprintln!("kero balance: no reaction arrow in '{equation}'");
                std::process::exit(2);
            };
            let strip = |t: &str| -> String {
                let t = t.trim();
                let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
                if digits.is_empty() || digits.len() == t.len() {
                    t.to_string()
                } else {
                    t[digits.len()..].trim().to_string()
                }
            };
            // A spaced plus: a bare one is also the charge on `Ca+2`.
            let lhs: Vec<String> = l.split(" + ").map(strip).collect();
            let rhs: Vec<String> = r.split(" + ").map(strip).collect();
            let lref: Vec<&str> = lhs.iter().map(String::as_str).collect();
            let rref: Vec<&str> = rhs.iter().map(String::as_str).collect();

            if let Ok(eq) = kerotakis_core::stoich::parse_equation(equation) {
                if eq.is_balanced() {
                    println!("already balanced");
                    std::process::exit(0);
                }
                for (el, d) in eq.element_imbalance() {
                    println!("  {el}: {d:+} on the right as written");
                }
                let c = eq.charge_imbalance();
                if c.abs() > 1e-6 {
                    println!("  charge: {c:+} on the right as written");
                }
            }
            match kerotakis_core::stoich::balance(&lref, &rref) {
                Ok(n) => {
                    let show = |names: &[&str], coeffs: &[i64]| -> String {
                        names
                            .iter()
                            .zip(coeffs)
                            .map(|(s, c)| {
                                if *c == 1 {
                                    (*s).to_string()
                                } else {
                                    format!("{c} {s}")
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" + ")
                    };
                    println!(
                        "{} → {}",
                        show(&lref, &n[..lref.len()]),
                        show(&rref, &n[lref.len()..])
                    );
                }
                Err(e) => {
                    eprintln!("kero balance: {e}");
                    std::process::exit(1);
                }
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

/// Load every codex file in a directory.
fn load_codex(dir: &str) -> kerotakis_codex::Codex {
    let mut all = kerotakis_codex::Codex::default();
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
        eprintln!("kero codex: cannot read {dir}: {e}");
        std::process::exit(1);
    });
    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    files.sort();
    for file in files {
        let text = std::fs::read_to_string(&file).unwrap_or_else(|e| {
            eprintln!("kero codex: cannot read {}: {e}", file.display());
            std::process::exit(1);
        });
        match kerotakis_codex::Codex::parse(&text) {
            Ok(mut c) => {
                all.reactions.append(&mut c.reactions);
                all.models.append(&mut c.models);
            }
            Err(e) => {
                eprintln!("kero codex: {}: {e}", file.display());
                std::process::exit(1);
            }
        }
    }
    all
}

/// Validate the codex — structurally, and by replaying every entry through
/// the real solvers. A claim the chemistry no longer supports is an error,
/// which is the whole point of the format.
fn codex_lint(dir: &str) -> ! {
    let codex = load_codex(dir);
    let vocabulary = load_vocabulary(dir);
    let mut problems = codex.structural_problems();

    for entry in &codex.reactions {
        let mut bench = Bench::new();
        let mut stack = build_stack();
        let mut observed: Vec<Event> = Vec::new();
        let mut failed = false;
        for line in entry.setup.script.lines() {
            match kerotakis_core::script::parse_op(line) {
                Ok(None) => {}
                Ok(Some(op)) => {
                    match bench.step_with(op, &mut stack, &kerotakis_safety::ReactiveGroupScreen) {
                        Ok(mut events) => observed.append(&mut events),
                        Err(e) => {
                            problems.push(format!("{}: setup failed: {e}", entry.id));
                            failed = true;
                            break;
                        }
                    }
                }
                Err(e) => {
                    problems.push(format!("{}: bad setup script: {e}", entry.id));
                    failed = true;
                    break;
                }
            }
        }
        if failed {
            continue;
        }
        if observed
            .iter()
            .any(|e| matches!(e, Event::SolverFailed { .. }))
        {
            problems.push(format!(
                "{}: a solver could not answer during the setup",
                entry.id
            ));
        }
        for anchor in &entry.spine {
            if !vocabulary.concepts.is_empty() && vocabulary.get(anchor).is_none() {
                problems.push(format!(
                    "{}: spine anchor '{anchor}' is not a topic in concepts.toml",
                    entry.id
                ));
            }
        }
        for claim in &entry.expect.events {
            if !observed
                .iter()
                .any(|e| kerotakis_codex::event_matches(e, claim))
            {
                problems.push(format!(
                    "{}: claims '{claim}', which the solvers did not produce",
                    entry.id
                ));
            }
        }
        for claim in &entry.expect.absent {
            if observed
                .iter()
                .any(|e| kerotakis_codex::event_matches(e, claim))
            {
                problems.push(format!(
                    "{}: claims '{claim}' does NOT happen, but it did",
                    entry.id
                ));
            }
        }
        let vessel = bench.vessel(VesselId(0)).expect("first vessel");
        if let Some(range) = entry.expect.ph {
            match vessel.solution.as_ref().map(|s| s.ph) {
                Some(ph) if range.contains(ph) => {}
                Some(ph) => problems.push(format!(
                    "{}: claims pH {}–{}, computed {ph:.2}",
                    entry.id, range.min, range.max
                )),
                None => problems.push(format!(
                    "{}: claims a pH, but no solver characterised the solution",
                    entry.id
                )),
            }
        }
        if let Some(range) = entry.expect.temperature_c {
            let t = vessel.temperature.to_celsius();
            if !range.contains(t) {
                problems.push(format!(
                    "{}: claims {}–{} °C, computed {t:.1} °C",
                    entry.id, range.min, range.max
                ));
            }
        }
    }

    if problems.is_empty() {
        println!(
            "codex ok: {} entries, {} concepts, {} models ({} progressions) — every claim replayed through the solvers",
            codex.reactions.len(),
            codex.concept_index().len(),
            codex.models.len(),
            codex.model_chains().len(),
        );
        // Say what is checked and what is declared not to be. With
        // `equation` and `summary` separate, an entry with no equation is
        // making a statement rather than leaving a hole.
        let audit = codex.equation_audit();
        println!(
            "  equations: {} balanced (atoms and charge); {} entries describe something that is not a reaction",
            audit.balanced, audit.summary_only
        );
        std::process::exit(0);
    }
    eprintln!("codex: {} problem(s)", problems.len());
    for p in &problems {
        eprintln!("  · {p}");
    }
    std::process::exit(1);
}

/// Load the curriculum spine, if the directory carries one.
fn load_vocabulary(dir: &str) -> kerotakis_codex::Vocabulary {
    let path = std::path::Path::new(dir).join("concepts.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => kerotakis_codex::Vocabulary::parse(&text).unwrap_or_else(|e| {
            eprintln!("kero codex: {}: {e}", path.display());
            std::process::exit(1);
        }),
        Err(_) => kerotakis_codex::Vocabulary::default(),
    }
}

/// What the spine says a chemistry curriculum contains that we do not
/// teach yet. This is the codex's work list, and it comes from someone
/// else's published taxonomy rather than from our own imagination.
fn codex_gaps(dir: &str) -> ! {
    let codex = load_codex(dir);
    let vocab = load_vocabulary(dir);
    if vocab.concepts.is_empty() {
        eprintln!("kero codex: no spine at {dir}/concepts.toml");
        std::process::exit(1);
    }
    let gaps = vocab.gaps(&codex);
    let covered = vocab.concepts.len() - gaps.len();
    println!(
        "spine: {} topics · covered {covered} · remaining {}",
        vocab.concepts.len(),
        gaps.len()
    );
    // Group by their parent topic so the list reads as areas of work.
    let mut by_area: std::collections::BTreeMap<String, Vec<&str>> = Default::default();
    for c in &gaps {
        let area = c
            .broader
            .as_deref()
            .and_then(|b| vocab.get(b).map(|p| p.label_de.clone()))
            .unwrap_or_else(|| "—".to_string());
        by_area.entry(area).or_default().push(&c.label_de);
    }
    for (area, mut topics) in by_area {
        topics.sort_unstable();
        println!("\n{area} ({}):", topics.len());
        for t in topics {
            println!("  · {t}");
        }
    }
    std::process::exit(0);
}

fn codex_concepts(dir: &str) -> ! {
    let codex = load_codex(dir);
    for (concept, entries) in codex.concept_index() {
        println!("{concept:<28} {}", entries.join(", "));
    }
    std::process::exit(0);
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
         \x20 ignite <vessel>            hold a flame to it\n\
         \x20 decant <from> <to> <fraction>\n\
         \x20 filter <from> <to>         solids stay, liquid passes\n\
         \x20 evaporate <vessel> <fraction>\n\
         \x20 measure <vessel> <thermometer|balance|ph>\n\
         \x20 new                        create a vessel\n\
         \x20 inspect [vessel]           show state\n\
         \x20 explain [vessel]           where the answer came from, and\n\
         \x20                            what every other dataset says\n\
         \x20 register <lv1|lv2|lv3>     how much detail to show\n\
         \x20 quit"
    );
    std::process::exit(2);
}

fn repl() {
    println!("kerotakis 0.0.1 — the bench is ready. 'help' lists commands.");
    let mut session = Session {
        bench: Bench::new(),
        register: Register::default(),
        json: false,
        stack: build_stack(),
        paths: kerotakis_phreeqc::PhreeqcEquilibrator::new().ok(),
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
                 new · inspect [v] · register <lv1|lv2|lv3> · species · quit"
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
                self.register = match words.get(1).and_then(|w| Register::parse(w)) {
                    Some(r) => r,
                    None => {
                        return Err(format!(
                            "unknown level {:?} — use lv1 (what you see), lv2 (equations), lv3 (full detail)",
                            words.get(1)
                        ))
                    }
                };
                Ok(())
            }
            "explain" => {
                let target = words
                    .get(1)
                    .map(|w| parse_vessel(w))
                    .transpose()?
                    .unwrap_or(VesselId(0));
                let vessel = self.bench.vessel(target).map_err(|e| e.to_string())?;
                // Where the standing answer came from.
                match vessel.solution.as_ref().and_then(|s| s.provenance.as_ref()) {
                    Some(p) => {
                        println!("  {target}: answered by {} using {}", p.engine, p.dataset);
                        println!("    model:   {}", p.model);
                        println!("    routing: {}", p.routing);
                        if !p.dataset_sources.is_empty() {
                            println!("    the dataset records its own sources, e.g.:");
                            for src in &p.dataset_sources {
                                println!("      · {src}");
                            }
                        }
                    }
                    None => println!("  {target}: no aqueous solver has characterised this vessel"),
                }
                // What every other dataset says about the same vessel.
                let vessel = vessel.clone();
                match self.paths.as_mut() {
                    None => println!("    (no engine available to compare paths)"),
                    Some(engine) => {
                        let paths = engine.compare_paths(&vessel);
                        if paths.is_empty() {
                            println!("    (nothing aqueous to compare)");
                        } else {
                            println!("  the same question, asked of every dataset:");
                            for path in paths {
                                match path.outcome {
                                    kerotakis_phreeqc::PathOutcome::Solved {
                                        ph,
                                        ionic_strength,
                                        phases,
                                    } => {
                                        let solids: String = phases
                                            .iter()
                                            .filter(|(_, m)| *m > 1e-9)
                                            .map(|(n, m)| format!(" · {n} {m:.4} mol"))
                                            .collect();
                                        println!(
                                            "    {:<14} pH {ph:.3} · I = {ionic_strength:.4} m{solids}",
                                            path.dataset
                                        );
                                        println!("      {}", path.model);
                                    }
                                    kerotakis_phreeqc::PathOutcome::CannotExpress {
                                        missing_elements,
                                    } => println!(
                                        "    {:<14} cannot express this problem (no {})",
                                        path.dataset,
                                        missing_elements.join(", ")
                                    ),
                                    kerotakis_phreeqc::PathOutcome::Failed { detail } => {
                                        let short: String =
                                            detail.lines().next().unwrap_or("").into();
                                        println!(
                                            "    {:<14} could not solve it: {short}",
                                            path.dataset
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            "particles" | "zoom" => {
                // The submicroscopic vertex of Johnstone's triangle. The
                // ratios are solved, not drawn — which is the whole reason
                // this is worth showing rather than illustrating.
                let target = words.get(1).map(|w| parse_vessel(w)).transpose()?;
                let vessels: Vec<&Vessel> = self
                    .bench
                    .vessels
                    .iter()
                    .filter(|v| target.is_none() || target == Some(v.id))
                    .collect();
                for v in vessels {
                    let census = kerotakis_core::particles::census(v, 30);
                    if self.json {
                        let doc = serde_json::json!({
                            "step": self.bench.log.len(),
                            "operator": { "op": "particles", "vessel": v.id },
                            "events": [],
                            "particles": census,
                        });
                        println!("{doc}");
                    } else {
                        println!("  {} — what the particles are doing:", v.id);
                        print!("{}", census.render(self.register));
                    }
                }
                Ok(())
            }
            "inspect" => {
                let target = words.get(1).map(|w| parse_vessel(w)).transpose()?;
                let vessels: Vec<&Vessel> = self
                    .bench
                    .vessels
                    .iter()
                    .filter(|v| target.is_none() || target == Some(v.id))
                    .collect();
                if self.json {
                    // The --json stream is the API contract: every line is a
                    // JSON object, inspect included.
                    let doc = serde_json::json!({
                        "step": self.bench.log.len(),
                        "operator": { "op": "inspect" },
                        "events": [],
                        "bench": { "vessels": vessels },
                    });
                    println!("{doc}");
                } else {
                    for v in vessels {
                        self.print_vessel(v);
                    }
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
        if self.register >= Register::LV3 {
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

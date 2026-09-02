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
//!   kero materials            list the named household and school bottles
//!   kero find <word>          search species and materials together

mod balance_exercise;
mod chart_svg;
mod coverage;
mod diagram;
mod fit;
mod mcp;
mod provenance;
mod study;
mod sweep;

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
    /// EXP-0: loaded quest specs (lazy, from ./quests) and live states.
    quests: Vec<kerotakis_codex::quest::QuestSpec>,
    quest_states: std::collections::BTreeMap<String, kerotakis_codex::quest::QuestState>,
    /// Sealed-unknown display layer: alias → real species key. Chemistry
    /// is never touched — input words are unmasked before parsing, and
    /// rendered lines are re-masked before printing.
    aliases: std::collections::BTreeMap<String, String>,
    /// The display mask, one (real, alias) pair per sealed unknown. Kept
    /// longest-real-first so `sodium chloride` is rewritten before any
    /// fragment of it can shadow the replacement.
    masks: Vec<(String, String)>,
    /// A quest's `covers` list: the unknown's dissociation ions, which
    /// have no alias of their own to type. Applied only to lines about
    /// vessels the unknown has actually touched — chloride from a bottle
    /// of acid the learner poured themselves is not the sample's, and
    /// must not be dressed as it.
    cover_masks: Vec<(String, String)>,
    /// The vessels a sealed unknown has touched: sealed on `add`, and
    /// spread by every `Transferred` event out of a sealed vessel.
    sealed_vessels: std::collections::HashSet<VesselId>,
}

/// Physics + aqueous chemistry + honesty. If the PHREEQC engine cannot be
/// initialised the session still works, honestly degraded.
fn build_stack() -> SolverStack {
    // The order is kerotakis-stack's, shared with the shell and the wasm
    // bench — chemistry must not depend on which host ran it. Only the
    // aqueous tail is this host's to choose.
    let tail: Vec<Box<dyn Equilibrator>> = match kerotakis_phreeqc::PhreeqcEquilibrator::new() {
        // The metallic state rides on top of the aqueous solve: the series
        // moves electrons over the activities PHREEQC reports, and the
        // products go back through it.
        Ok(aqueous) => vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
            kerotakis_core::DisplacementEquilibrator::wrapping(Box::new(aqueous)),
        )))],
        Err(e) => {
            eprintln!("kero: aqueous engine unavailable ({e}); running without it");
            // Pure-water phase changes still work in the honestly degraded
            // stack; only brine re-speciation is unavailable.
            vec![Box::new(StateEquilibrator)]
        }
    };
    let solvers = kerotakis_stack::standard_solvers(tail);
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
                quests: Vec::new(),
                quest_states: Default::default(),
                aliases: Default::default(),
                masks: Vec::new(),
                cover_masks: Vec::new(),
                sealed_vessels: Default::default(),
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
                "export" => {
                    let out_path = args.get(2).unwrap_or_else(|| {
                        eprintln!("usage: kero codex export <out.json>");
                        std::process::exit(2);
                    });
                    codex_export(&dir, out_path);
                }
                other => {
                    eprintln!(
                        "kero codex: unknown subcommand '{other}' (lint, concepts, gaps, export)"
                    );
                    std::process::exit(2);
                }
            }
        }
        Some("coverage") => coverage::command(&args[1..], build_stack),
        Some("pack") => {
            // DATA-010: compile a registry document into a .pack for
            // independent delivery. Default source: the checked-in
            // registry; --from for arbitrary documents (tests, future
            // pack authors).
            let sub = args.get(1).map(String::as_str).unwrap_or("");
            if sub != "export" {
                eprintln!("kero pack: usage: pack export [--from doc.json] OUT.pack");
                std::process::exit(2);
            }
            let from = args
                .iter()
                .position(|a| a == "--from")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "data/registry/registry-source-v1.json".to_string());
            let out = args
                .iter()
                .skip(2)
                .find(|a| *a != "--from" && !from.ends_with(a.as_str()))
                .cloned()
                .unwrap_or_else(|| "registry.pack".to_string());
            let text = std::fs::read_to_string(&from).unwrap_or_else(|e| {
                eprintln!("kero pack export: reading {from}: {e}");
                std::process::exit(2);
            });
            let doc: kerotakis_data::RegistryDocument =
                serde_json::from_str(&text).unwrap_or_else(|e| {
                    eprintln!("kero pack export: {from} is not a registry document: {e}");
                    std::process::exit(2);
                });
            let pack = kerotakis_data::build_pack(&doc);
            use sha2::{Digest, Sha256};
            let hash = format!("{:x}", Sha256::digest(&pack));
            std::fs::write(&out, &pack).unwrap_or_else(|e| {
                eprintln!("kero pack export: writing {out}: {e}");
                std::process::exit(2);
            });
            println!(
                "pack: {} species + {} material recipes → {out} ({} bytes, sha256 {hash})",
                doc.identities.len(),
                doc.material_recipes.len(),
                pack.len()
            );
        }
        Some("provenance") => {
            let sub = args.get(1).map(String::as_str).unwrap_or("lint");
            if sub != "lint" {
                eprintln!("kero provenance: unknown subcommand '{sub}' (lint)");
                std::process::exit(2);
            }
            let manifest = args
                .iter()
                .position(|a| a == "--manifest")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "provenance/sources.toml".to_string());
            let root = args
                .iter()
                .position(|a| a == "--root")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| ".".to_string());
            provenance::lint_command(&manifest, &root);
        }
        Some("quest") => {
            let sub = args.get(1).map(String::as_str).unwrap_or("lint");
            if sub == "export" {
                // GUI-066: the quest specs as one JSON document, shipped
                // beside the web payload like the codex export.
                let out = args.get(2).map(String::as_str).unwrap_or("quests.json");
                let specs = kerotakis_codex::quest::load_dir(std::path::Path::new("quests"))
                    .unwrap_or_else(|e| {
                        eprintln!("kero quest export: {e}");
                        std::process::exit(2);
                    });
                let doc = serde_json::json!({ "quests": specs });
                std::fs::write(out, serde_json::to_string_pretty(&doc).unwrap()).unwrap_or_else(
                    |e| {
                        eprintln!("kero quest export: writing {out}: {e}");
                        std::process::exit(2);
                    },
                );
                println!("quest: exported {} quests → {out}", specs.len());
                return;
            }
            if sub != "lint" {
                eprintln!(
                    "kero quest: only 'lint' and 'export' work outside the REPL (quests are interactive)"
                );
                std::process::exit(2);
            }
            let dir = args
                .iter()
                .position(|a| a == "--dir")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "quests".to_string());
            match kerotakis_codex::quest::load_dir(std::path::Path::new(&dir)) {
                Ok(specs) => {
                    let problems = kerotakis_codex::quest::lint(&specs);
                    if problems.is_empty() {
                        println!("quests: {} spec(s), all sound", specs.len());
                    } else {
                        for p in &problems {
                            eprintln!("quest lint: {p}");
                        }
                        eprintln!("quests: {} problem(s)", problems.len());
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("kero quest lint: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("study") => {
            study::study_command(&args[1..]);
        }
        Some("fit") => fit::fit_command(&args[1..]),
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
            // The shipped cache is also the offline R1 contract. Keep these
            // application scenarios beside the lesson states so a browser
            // with no attached engine can prove the same five outcomes.
            let r1 = kerotakis_phreeqc::acceptance::run_r1_acceptance(&mut engine);
            if !r1.passed() {
                eprintln!(
                    "kero prewarm: R1 acceptance failed:\n{}",
                    serde_json::to_string_pretty(&r1).expect("serialise R1 report")
                );
                std::process::exit(1);
            }
            let data = engine.export_cache();
            let bytes = postcard::to_allocvec(&data).expect("serialise cache");
            std::fs::write(&out, &bytes).unwrap_or_else(|e| {
                eprintln!("kero prewarm: cannot write {out}: {e}");
                std::process::exit(1);
            });
            println!(
                "pre-warmed {} solver results from {steps} lesson steps and 5 R1 scenarios across {} lessons → {out} ({} bytes)",
                data.entries.len(),
                files.len(),
                bytes.len()
            );
        }
        Some("species") => {
            for s in species::REGISTRY {
                let verified = kerotakis_org::inchi_validate::CURATED_STRUCTURES
                    .iter()
                    .any(|(id, _)| *id == s.key);
                let mark = if verified { "✓" } else { " " };
                let hazards = kerotakis_safety::hazard_labels(s.key);
                let hz = if hazards.is_empty() {
                    String::new()
                } else {
                    format!("  ⚠ {}", hazards.join(", "))
                };
                println!(
                    "{:<10} {mark} {:<18} {:<8} M={:>8.3} g/mol   [{}]{hz}",
                    s.key, s.name, s.formula, s.molar_mass, s.provenance
                );
            }
            println!(
                "
✓ = identity verified: curated structure recomputed by the                  official IUPAC InChI library (1.07.5) matches the registry key"
            );
            println!(
                "{} named household and school bottles share this shelf — `kero materials` lists them, `kero find <word>` searches both halves.",
                kerotakis_core::material::all().len()
            );
        }
        // KID-1: the other half of the shelf, outside the REPL.
        //
        // `BRD-002` landed `find` as a REPL line, which is the right search
        // but the wrong reach: a script, a pipe, or anyone reading
        // `kero --help` never meets it. The audit in KIDS.md lost twelve of
        // thirty children's activities to names that were in the registry
        // the whole time, so both the listing and the search now exist as
        // subcommands too.
        Some("materials") => print_materials(),
        // KID-17: thirty-eight lessons ship and no command named one, so
        // `kero run lessons/fizz.lab` was reachable only by reading the
        // repository.
        Some("lessons") => print_lessons(),
        Some("find") => match args.get(1) {
            Some(query) => print_cabinet_search(query, &Bench::new()),
            None => {
                eprintln!(
                    "usage: kero find <word> — searches species keys, names, \
                     formulas and material aliases"
                );
                std::process::exit(2);
            }
        },
        Some("mechanism") => mechanism_command(&args[1..]),
        Some("sweep") => {
            // Drive a matrix of states through the whole stack and check
            // every invariant the engine claims about itself. Checking a
            // claim is cheaper than believing it.
            run_sweep(args.get(1).map(String::as_str));
        }
        Some("chart") => {
            // The universal outlet: any chart-contract JSON becomes SVG.
            // Producers write the contract; this renders it — the study
            // runner and the titration curve plug in here the day they
            // exist.
            let Some(input) = args.get(1) else {
                eprintln!("usage: kero chart <chart.json> [-o out.svg]");
                std::process::exit(2);
            };
            let out = match (args.get(2).map(String::as_str), args.get(3)) {
                (Some("-o") | Some("--out"), Some(p)) => p.clone(),
                _ => format!("{}.svg", input.trim_end_matches(".json")),
            };
            let text = match std::fs::read_to_string(input) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("kero chart: cannot read {input}: {e}");
                    std::process::exit(2);
                }
            };
            let chart: kerotakis_core::chart::Chart = match serde_json::from_str(&text) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("kero chart: {input} is not chart-contract JSON: {e}");
                    std::process::exit(2);
                }
            };
            if let Err(e) = std::fs::write(&out, chart_svg::render(&chart)) {
                eprintln!("kero chart: cannot write {out}: {e}");
                std::process::exit(2);
            }
            eprintln!("wrote {out}");
        }
        Some("diagram") => {
            // The workbench-class artefact, computed: `diagram pourbaix Fe`
            // solves a pe-pH grid cell by cell and draws what the
            // thermodynamics says, with refusals kept visible.
            match args.get(1).map(String::as_str) {
                Some("pourbaix") => {
                    if let Err(e) = diagram::run(&args[2..]) {
                        eprintln!("kero diagram: {e}");
                        std::process::exit(2);
                    }
                }
                Some("txy") => {
                    if let Err(e) = diagram::run_txy(&args[2..]) {
                        eprintln!("kero diagram: {e}");
                        std::process::exit(2);
                    }
                }
                _ => {
                    eprintln!("usage: kero diagram pourbaix <element> [--grid NxM] [--out FILE.svg] [--json]");
                    std::process::exit(2);
                }
            }
        }
        Some("calc") => {
            if args.len() < 2 {
                calc_usage();
            }
            let name = &args[1];
            if name == "help" || name == "--help" {
                calc_usage();
            }
            let relation_args: Vec<String> = args[2..].to_vec();
            match kerotakis_core::relations::evaluate(name, &relation_args) {
                Ok(result) => {
                    if args.iter().any(|a| a == "--json") {
                        let relation_args_clean: Vec<&String> =
                            relation_args.iter().filter(|a| *a != "--json").collect();
                        println!(
                            "{}",
                            serde_json::json!({
                                "relation": name,
                                "args": relation_args_clean,
                                "value": result.value,
                                "unit": result.unit,
                                "provenance": result.provenance,
                                "lv1": result.lv1,
                                "lv2": result.lv2,
                                "lv3": result.lv3,
                            })
                        );
                    } else {
                        println!("{}", result.lv3);
                    }
                }
                Err(e) => {
                    eprintln!("kero calc: {e}");
                    std::process::exit(2);
                }
            }
        }
        Some("properties") => {
            if args.len() < 2 {
                properties_usage();
            }
            let name = &args[1];
            if name == "help" || name == "--help" {
                properties_usage();
            }
            if *name == "water" {
                let t_k = args
                    .iter()
                    .position(|a| a == "--at")
                    .and_then(|i| args.get(i + 1))
                    .map(|s| s.as_str())
                    .or_else(|| args.iter().find_map(|a| a.strip_prefix("--at=")))
                    .map(|val| {
                        if val.ends_with('C') {
                            val.trim_end_matches('C').parse::<f64>().unwrap_or(25.0) + 273.15
                        } else {
                            val.trim_end_matches('K').parse::<f64>().unwrap_or(298.15)
                        }
                    })
                    .unwrap_or(298.15);
                let json = args.iter().any(|a| a == "--json");
                let table = kerotakis_core::properties::water_table(t_k);
                if json {
                    let entries: Vec<serde_json::Value> = table
                        .iter()
                        .map(|(name, r)| match r {
                            Ok(r) => serde_json::json!({
                                "property": name,
                                "value": r.value,
                                "unit": r.unit,
                                "provenance": r.provenance,
                            }),
                            Err(e) => serde_json::json!({
                                "property": name,
                                "error": e,
                            }),
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({
                            "species": "water",
                            "T_K": t_k,
                            "T_C": t_k - 273.15,
                            "properties": entries,
                        })
                    );
                } else {
                    println!("water at {:.2} K ({:.1} °C)\n", t_k, t_k - 273.15);
                    for (name, r) in &table {
                        match r {
                            Ok(r) => println!(
                                "  {:<16} {:.6} {:<16} {}",
                                name, r.value, r.unit, r.provenance
                            ),
                            Err(e) => println!("  {:<16} {}", name, e),
                        }
                    }
                    // Henry coefficients for all gases
                    println!("\nHenry's constants at {:.2} K:\n", t_k);
                    for c in kerotakis_core::properties::HENRY_COEFFICIENTS {
                        let h = kerotakis_core::properties::henry_at_t(c, t_k);
                        println!(
                            "  {:<6} ({:<16}) H = {:.4e} {}",
                            c.formula, c.gas, h.value, h.unit
                        );
                    }
                    println!(
                        "\n  {}",
                        kerotakis_core::properties::HENRY_COEFFICIENTS[0].provenance
                    );
                }
            } else {
                let prop_args: Vec<String> = args[2..].to_vec();
                match kerotakis_core::properties::evaluate(name, &prop_args) {
                    Ok(result) => {
                        if args.iter().any(|a| a == "--json") {
                            println!(
                                "{}",
                                serde_json::json!({
                                    "property": name,
                                    "value": result.value,
                                    "unit": result.unit,
                                    "provenance": result.provenance,
                                })
                            );
                        } else {
                            println!(
                                "{:.6} {} — {}",
                                result.value, result.unit, result.provenance
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("kero properties: {e}");
                        std::process::exit(2);
                    }
                }
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
            // GUI-095: the same solver posed as a drill. An equation is
            // never the bare word "exercise" — it has no arrow — so the
            // two readings of `args[1]` cannot collide.
            if equation == "exercise" {
                match balance_exercise_text(&args[2..]) {
                    Ok(text) => {
                        print!("{text}");
                        return;
                    }
                    Err(e) => {
                        eprintln!("kero balance exercise: {e}");
                        std::process::exit(2);
                    }
                }
            }
            if !ARROWS.iter().any(|a| equation.contains(a)) {
                eprintln!("kero balance: no reaction arrow in '{equation}'");
                std::process::exit(2);
            }
            match balance_text(equation) {
                Ok(text) => print!("{text}"),
                Err(e) => {
                    eprintln!("kero balance: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("serve") => {
            // The bench as an MCP server over stdio — the same public API
            // and the same `--json` contract, so a drafting agent gets
            // exactly the answers the CLI gets (PLAN.md, "Curation is
            // verifiable, so drafting can be assisted").
            if !args.iter().any(|a| a == "--mcp") {
                eprintln!("kero serve: only --mcp is available (kero serve --mcp)");
                std::process::exit(2);
            }
            let dir = args
                .iter()
                .position(|a| a == "--dir")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .unwrap_or_else(|| "codex".to_string());
            mcp::serve(dir);
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

// ---------------------------------------------------------------------
// The `--json` contract, built in exactly one place. The CLI's `--json`
// stream and the MCP server both call these, so the two can never drift —
// which is the point of having a contract (PLAN.md, "CLI first").

fn json_step(
    step: usize,
    op: &Operator,
    events: &[Event],
    vessels: &[Vessel],
) -> serde_json::Value {
    serde_json::json!({
        "step": step,
        "operator": op,
        "events": events,
        // The render model (PROTOCOL.md, GUI-003): the same object the wasm
        // step() carries, so every host repaints from one round trip.
        "scene": kerotakis_core::scene_of(vessels),
        "bench": { "vessels": vessels },
    })
}

/// The vessels as they stand, unmasked. Every `--json` line goes through
/// [`Repl::mask_json`] before it is printed, which is where a sealed
/// unknown stops being nameable — see the doc there.
fn json_inspect(step: usize, vessels: &[&Vessel]) -> serde_json::Value {
    serde_json::json!({
        "step": step,
        "operator": { "op": "inspect" },
        "events": [],
        "bench": { "vessels": vessels },
    })
}

fn json_particles(step: usize, v: &Vessel) -> serde_json::Value {
    serde_json::json!({
        "step": step,
        "operator": { "op": "particles", "vessel": v.id },
        "events": [],
        "particles": kerotakis_core::particles::census(v, 30),
    })
}

/// `explain` in the JSON stream: provenance is an answer too, and prose
/// printed into an NDJSON stream is the same defect `inspect` once had.
fn json_explain(step: usize, vessel: VesselId, text: &str) -> serde_json::Value {
    serde_json::json!({
        "step": step,
        "operator": { "op": "explain", "vessel": vessel },
        "events": [],
        "text": text,
    })
}

const ARROWS: [&str; 4] = ["->", "→", "=", "⇌"];

/// Everything `kero balance` says, as a string — shared by the CLI arm and
/// the MCP `balance` tool so the two cannot disagree.
fn balance_text(equation: &str) -> Result<String, String> {
    use std::fmt::Write as _;
    let Some((l, r)) = ARROWS.iter().find_map(|a| equation.split_once(a)) else {
        return Err(format!("no reaction arrow in '{equation}'"));
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

    let mut out = String::new();
    if let Ok(eq) = kerotakis_core::stoich::parse_equation(equation) {
        if eq.is_balanced() {
            return Ok("already balanced\n".into());
        }
        for (el, d) in eq.element_imbalance() {
            writeln!(out, "  {el}: {d:+} on the right as written").unwrap();
        }
        let c = eq.charge_imbalance();
        if c.abs() > 1e-6 {
            writeln!(out, "  charge: {c:+} on the right as written").unwrap();
        }
    }
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
    let show_signed = |names: &[&str], coeffs: &[i64]| -> String {
        names
            .iter()
            .zip(coeffs)
            .map(|(s, c)| {
                if *c == 0 {
                    String::new()
                } else if *c == 1 {
                    format!("+{s}")
                } else if *c == -1 {
                    format!("-{s}")
                } else {
                    format!("{c:+} {s}")
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    };
    match kerotakis_core::stoich::balance(&lref, &rref) {
        Ok(kerotakis_core::stoich::BalanceResult::Unique(n)) => {
            writeln!(
                out,
                "{} → {}",
                show(&lref, &n[..lref.len()]),
                show(&rref, &n[lref.len()..])
            )
            .unwrap();
            Ok(out)
        }
        Ok(kerotakis_core::stoich::BalanceResult::Family { particular, basis }) => {
            let all: Vec<&str> = lref.iter().chain(rref.iter()).copied().collect();
            writeln!(
                out,
                "under-determined: {} independent reactions\n",
                basis.len() + 1
            )
            .unwrap();
            writeln!(
                out,
                "particular solution:\n  {} → {}",
                show(&lref, &particular[..lref.len()]),
                show(&rref, &particular[lref.len()..])
            )
            .unwrap();
            for (i, bv) in basis.iter().enumerate() {
                writeln!(
                    out,
                    "\nbasis vector {}:\n  {}",
                    i + 1,
                    show_signed(&all, bv)
                )
                .unwrap();
            }
            writeln!(
                out,
                "\nany non-negative integer combination (particular + k₁·v₁ + …) \
                 with all coefficients > 0 is a valid balanced equation."
            )
            .unwrap();
            Ok(out)
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Everything `kero balance exercise` says, as a string (GUI-095).
///
/// The headless half of the balancing drill: the browser panel and this
/// share the engine's `balance_report` and the marking rule it makes
/// possible, so a reaction that drills one way drills the other.
fn balance_exercise_text(args: &[String]) -> Result<String, String> {
    use crate::balance_exercise::{blank_equation, mark, pool, write_equation, Verdict};
    use std::fmt::Write as _;

    let flag = |name: &str| -> Option<&str> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
    };
    // Positionals: everything that is neither a flag nor a flag's value.
    let mut positional: Vec<&str> = Vec::new();
    let mut skip = false;
    for a in args {
        if skip {
            skip = false;
            continue;
        }
        if a.starts_with("--") {
            skip = true;
            continue;
        }
        positional.push(a.as_str());
    }

    let usage = "usage: kero balance exercise list|unusable|show|check|answer \
                 [--dir codex] [--limit N]";
    let Some(sub) = positional.first().copied() else {
        return Err(usage.to_string());
    };
    let dir = flag("--dir").unwrap_or("codex");
    let limit: usize = match flag("--limit") {
        Some(n) => n
            .parse()
            .map_err(|_| format!("--limit {n:?} is not a number"))?,
        None => 20,
    };

    let p = pool(dir);
    let find = |id: &str| {
        p.exercises
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| format!("no exercise {id:?} — try `kero balance exercise list`"))
    };

    let mut out = String::new();
    match sub {
        "list" => {
            for e in p.exercises.iter().take(limit) {
                writeln!(out, "{}\t{}", e.id, blank_equation(&e.report)).expect("string");
            }
            writeln!(
                out,
                "\n{} exercises, {} shown; {} equations could not be drilled",
                p.exercises.len(),
                p.exercises.len().min(limit),
                p.unusable.len()
            )
            .expect("string");
        }
        "unusable" => {
            // The other half of the audit: what the field carries that is
            // not an equation is counted, never quietly skipped.
            for (id, why) in p.unusable.iter().take(limit) {
                writeln!(out, "{id}\t{why}").expect("string");
            }
            writeln!(out, "\n{} could not be drilled", p.unusable.len()).expect("string");
        }
        "show" => {
            let id = positional
                .get(1)
                .copied()
                .ok_or("usage: kero balance exercise show <id>")?;
            let e = find(id)?;
            writeln!(out, "{}", e.id).expect("string");
            writeln!(out, "  as shipped: {}", e.source).expect("string");
            writeln!(out, "  skeleton:   {}", blank_equation(&e.report)).expect("string");
            // The answer's positions, so a coefficient list is unambiguous
            // — and no more than that: the answer itself is not shown.
            let order: Vec<String> = e
                .report
                .species
                .iter()
                .enumerate()
                .map(|(i, name)| format!("{}={name}", i + 1))
                .collect();
            writeln!(out, "  order:      {}", order.join("  ")).expect("string");
            if !e.report.basis.is_empty() {
                writeln!(
                    out,
                    "  note:       underdetermined — {} independent reactions share these species",
                    e.report.basis.len() + 1
                )
                .expect("string");
            }
            writeln!(
                out,
                "  answer with: kero balance exercise check {id} {}",
                vec!["1"; e.report.species.len()].join(",")
            )
            .expect("string");
        }
        "answer" => {
            let id = positional
                .get(1)
                .copied()
                .ok_or("usage: kero balance exercise answer <id>")?;
            let e = find(id)?;
            writeln!(out, "{}", write_equation(&e.report, &e.report.coefficients)).expect("string");
        }
        "check" => {
            let id = positional
                .get(1)
                .copied()
                .ok_or("usage: kero balance exercise check <id> <c1,c2,…>")?;
            let e = find(id)?;
            let text = positional
                .get(2)
                .copied()
                .ok_or("usage: kero balance exercise check <id> <c1,c2,…>")?;
            let answer: Vec<i64> = text
                .split([',', ' '])
                .filter(|t| !t.is_empty())
                .map(|t| {
                    t.parse::<i64>()
                        .map_err(|_| format!("{t:?} is not a whole number"))
                })
                .collect::<Result<_, _>>()?;
            let m = mark(&e.report, &answer);
            writeln!(out, "{}", m.verdict.tag()).expect("string");
            match m.verdict {
                Verdict::Correct => {
                    writeln!(out, "  {}", write_equation(&e.report, &answer)).expect("string");
                    writeln!(
                        out,
                        "  every element and the charge cancel, in the smallest whole numbers"
                    )
                    .expect("string");
                }
                Verdict::Multiple => {
                    let simplest: Vec<i64> = answer.iter().map(|c| c / m.factor).collect();
                    writeln!(
                        out,
                        "  balanced, but every coefficient is {} times larger than it needs to be",
                        m.factor
                    )
                    .expect("string");
                    writeln!(
                        out,
                        "  divide through: {}",
                        write_equation(&e.report, &simplest)
                    )
                    .expect("string");
                }
                Verdict::Unbalanced => {
                    // `amount` is the surplus on the LEFT: the report
                    // negates right-hand species so a balanced answer is
                    // exactly one the matrix sends to zero.
                    for miss in &m.misses {
                        let side = if miss.amount > 0.0 { "left" } else { "right" };
                        // The magnitude only: "too many on the right"
                        // already carries the direction, and a `+` in
                        // front of it reads as a second, contradictory one.
                        writeln!(
                            out,
                            "  {}: {} too many on the {side}",
                            miss.element,
                            miss.amount.abs()
                        )
                        .expect("string");
                    }
                }
                Verdict::Incomplete => {
                    writeln!(
                        out,
                        "  this skeleton takes {} whole coefficients of at least 1; got {}",
                        e.report.species.len(),
                        answer.len()
                    )
                    .expect("string");
                }
            }
            if m.family {
                writeln!(
                    out,
                    "  (underdetermined: more than one independent reaction balances this)"
                )
                .expect("string");
            }
        }
        other => return Err(format!("unknown subcommand {other:?}\n{usage}")),
    }
    Ok(out)
}

/// Everything `explain` says, as a string — the REPL prints it, the MCP
/// server returns it, and building it in one place keeps them identical.
/// BRD-002: what `find` prints.
///
/// Species and materials in one list, because `add` takes them the same
/// way and a search that separated them would teach a distinction the
/// grammar does not make. The shelf level rides along where a bottle has
/// been stocked — the question "can I still use this?" is the same
/// question as "what is it called?", asked half a step later.
/// KID-1: the fifty named bottles, in the spelling `add` takes.
///
/// `species` answers "what pure substances does the registry know"; this
/// answers "what is on the shelf a child would recognise" — vinegar, milk,
/// dish soap, yeast, cornstarch, steel wool. Each row leads with the key
/// you type, because the audit's most common failure was a learner knowing
/// exactly what they wanted and not knowing what to call it.
///
/// What a recipe cannot resolve is printed rather than dropped: milk is
/// 87% water and 13% conserved-but-unresolved milk solids, and a listing
/// that showed only the water would be claiming the rest is not there.
/// KID-17: the lessons, listed.
///
/// Thirty-eight `.lab` files ship with the bench and no command named one,
/// so `kero run lessons/fizz.lab` was reachable only by reading the
/// repository. The first line of a lesson is its title by convention; that
/// convention is now load-bearing enough to be checked by a test.
fn print_lessons() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lessons");
    let dir = if dir.is_dir() {
        dir
    } else {
        std::path::PathBuf::from("lessons")
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        eprintln!(
            "kero lessons: no lessons directory here (looked in {}) — run from a checkout, \
             or pass a path to `kero run`",
            dir.display()
        );
        std::process::exit(2);
    };
    let mut lessons: Vec<(String, String)> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "lab"))
        .filter_map(|path| {
            let name = path.file_stem()?.to_str()?.to_string();
            let text = std::fs::read_to_string(&path).ok()?;
            let title = text
                .lines()
                .find_map(|line| line.strip_prefix('#').map(|rest| rest.trim().to_string()))
                .unwrap_or_default();
            Some((name, title))
        })
        .collect();
    lessons.sort();
    for (name, title) in &lessons {
        println!("{name:<34} {title}");
    }
    println!(
        "— {} lessons. Run one with `kero run lessons/<name>.lab`; \
         add `--json` for one JSON object per step.",
        lessons.len()
    );
}

fn print_materials() {
    let recipes = kerotakis_core::material::all();
    for recipe in &recipes {
        let form = match &recipe.physical_form {
            kerotakis_data::MaterialPhysicalForm::HomogeneousLiquid => "liquid",
            kerotakis_data::MaterialPhysicalForm::Suspension => "suspension",
            kerotakis_data::MaterialPhysicalForm::Powder => "powder",
            kerotakis_data::MaterialPhysicalForm::Granules => "granules",
            kerotakis_data::MaterialPhysicalForm::BulkSolid => "solid",
            kerotakis_data::MaterialPhysicalForm::GasMixture => "gas",
            kerotakis_data::MaterialPhysicalForm::CompositeObject { .. } => "object",
            kerotakis_data::MaterialPhysicalForm::Other { .. } => "other",
        };
        let unit = match recipe.basis {
            kerotakis_core::material::MaterialBasis::MassFraction => "g",
            kerotakis_core::material::MaterialBasis::MoleFraction => "mol",
            kerotakis_core::material::MaterialBasis::VolumeFraction => "mL",
        };
        println!(
            "{:<30} {:<10} {:<3} {}",
            recipe.canonical_key, form, unit, recipe.name
        );
        // A trace component rounds to "0%" at whole percent, and a shelf
        // row that says a substance is 0% of the bottle is worse than one
        // that omits it. Small fractions keep the digits that make them
        // true.
        let percent = |value: f64| -> String {
            let pc = 100.0 * value;
            if pc >= 10.0 {
                format!("{pc:.0}%")
            } else if pc >= 1.0 {
                format!("{pc:.1}%")
            } else if pc >= 0.001 {
                format!("{pc:.3}%")
            } else {
                // A catalase surrogate is parts per million of the bottle.
                // "0.000%" would read as absent, which is the one thing it
                // is not.
                format!("{pc:.1e}%")
            }
        };
        let mut resolves: Vec<String> = recipe
            .components
            .iter()
            .map(|component| {
                format!(
                    "{} {}",
                    component.species_id,
                    percent(0.5 * (component.fraction.lower + component.fraction.upper))
                )
            })
            .collect();
        if let Some(unresolved) = &recipe.unresolved_fraction {
            let mean = 0.5 * (unresolved.lower + unresolved.upper);
            if mean > 0.0 {
                resolves.push(format!("{} conserved but unresolved", percent(mean)));
            }
        }
        if !resolves.is_empty() {
            println!("      → {}", resolves.join(", "));
        }
        let mut also: Vec<String> = Vec::new();
        for (tag, aliases) in &recipe.aliases {
            for alias in aliases {
                // Print the spelling that can actually be typed: the
                // grammar splits on whitespace, and KID-1 made the
                // underscore form of every alias resolve. Several recipes
                // carry both the spaced and the underscored spelling of one
                // name, which is now the same name — print it once.
                let writable = format!("{}[{tag}]", alias.replace(char::is_whitespace, "_"));
                if !also.contains(&writable) {
                    also.push(writable);
                }
            }
        }
        if !also.is_empty() {
            println!("      also: {}", also.join(", "));
        }
    }
    println!(
        "— {} named bottles. Add one with `add v1 <key> <amount><mol|g|mL>`; \
         `kero find <word>` searches these and the pure species together.",
        recipes.len()
    );
}

fn print_cabinet_search(query: &str, bench: &Bench) {
    use kerotakis_core::cabinet::{self, CabinetKind, CabinetMatch};
    let hits = cabinet::search(query, 30);
    if hits.is_empty() {
        println!("  nothing on the shelf matches '{query}'");
        return;
    }
    for hit in &hits {
        let kind = match hit.kind {
            CabinetKind::Species => "species",
            CabinetKind::Material => "material",
        };
        let level = match bench.stock.remaining(&hit.key) {
            Some(amount) => format!("  — {:.4} {} left", amount.amount, amount.unit),
            // An unstocked key is an unlimited supply, which is the
            // sandbox every script written before BRD-002 assumed. Say so
            // rather than printing a blank and letting it read as empty.
            None => "  — unstocked (unlimited)".to_string(),
        };
        let via = match (&hit.via, hit.matched) {
            (Some(alias), _) => format!("  [{alias}]"),
            (None, CabinetMatch::Substring) => String::new(),
            (None, _) => String::new(),
        };
        println!(
            "  {:<14} {:<8} {:<26} {} in {}{via}{level}",
            hit.key, kind, hit.name, hit.detail, hit.unit
        );
    }
    println!("  — add one with `add v1 <key> <amount><mol|g|mL>`");
}

/// The recipes this vessel was built from, as provenance.
///
/// Read off the bench log rather than the vessel, because the vessel keeps
/// only what a recipe could *not* resolve. A recipe that expanded cleanly
/// leaves no trace in the contents — its acetic acid is indistinguishable
/// from acetic acid poured from a bottle — and that is exactly the case
/// worth reporting: the number in front of you rests on a reviewed
/// estimate of a composition, and nothing else on screen says so.
fn material_provenance(bench: &Bench, target: VesselId) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let mut seen: Vec<(String, u32)> = Vec::new();
    for entry in &bench.log {
        let kerotakis_core::ops::Operator::AddMaterial {
            vessel,
            material,
            recipe_id,
            recipe_version,
            total_amount,
            ..
        } = &entry.operator
        else {
            continue;
        };
        if *vessel != target {
            continue;
        }
        // One report per recipe version, however many times it was poured:
        // the provenance of a substance does not change with the dose.
        if seen.contains(&(recipe_id.clone(), *recipe_version)) {
            continue;
        }
        seen.push((recipe_id.clone(), *recipe_version));
        let Some(recipe) = kerotakis_core::material::lookup_versioned(recipe_id, *recipe_version)
        else {
            writeln!(
                out,
                "  {target}: {material} came from recipe {recipe_id} v{recipe_version}, which this build no longer carries"
            )
            .unwrap();
            continue;
        };
        let unit = kerotakis_core::stock::stock_unit(&recipe.canonical_key)
            .map(|u| u.label())
            .unwrap_or("");
        writeln!(
            out,
            "  {target}: {} came from the recipe {}@v{} ({}), dispensed {:.4}{unit}",
            material, recipe.canonical_key, recipe.version, recipe.name, total_amount
        )
        .unwrap();
        // A confidence is a claim about how far to trust the composition,
        // so it is spelled out. `{:?}` would print "Surrogate", which is
        // an enum variant rather than a sentence and tells a learner
        // nothing about what it means for the number in front of them.
        let confidence = match recipe.confidence {
            kerotakis_core::material::MaterialConfidence::Measured => {
                "measured — the composition was determined, not assumed"
            }
            kerotakis_core::material::MaterialConfidence::Curated => {
                "curated — read off a specification or label and reviewed"
            }
            kerotakis_core::material::MaterialConfidence::Estimated => {
                "estimated — a reviewed estimate, within the stated ranges"
            }
            kerotakis_core::material::MaterialConfidence::Surrogate => {
                "surrogate — a stand-in composition that behaves like the real thing for the chemistry modelled here, and is not a claim about any particular product"
            }
        };
        writeln!(out, "    confidence: {confidence}").unwrap();
        for component in &recipe.components {
            let range = if component.fraction.lower == component.fraction.upper {
                format!("{:.4}", component.fraction.lower)
            } else {
                format!(
                    "{:.4}–{:.4}",
                    component.fraction.lower, component.fraction.upper
                )
            };
            writeln!(out, "      · {} {range}", component.species_id).unwrap();
        }
        // The honest remainder. A recipe that resolves 97% of itself is
        // making a claim about 97% of itself, and the other 3% is not
        // nothing — it is the part the review could not name.
        if let Some(unresolved) = &recipe.unresolved_fraction {
            writeln!(
                out,
                "      · unresolved {:.4}–{:.4} — real matter this recipe does not name",
                unresolved.lower, unresolved.upper
            )
            .unwrap();
        }
        for assumption in &recipe.lot_assumptions {
            writeln!(out, "    assumes: {assumption}").unwrap();
        }
    }
    out
}

fn explain_text(
    bench: &Bench,
    paths: &mut Option<kerotakis_phreeqc::PhreeqcEquilibrator>,
    target: VesselId,
) -> Result<String, String> {
    use std::fmt::Write as _;
    let vessel = bench.vessel(target).map_err(|e| e.to_string())?;
    let mut out = String::new();
    // Where the standing answer came from.
    match vessel.solution.as_ref().and_then(|s| s.provenance.as_ref()) {
        Some(p) => {
            writeln!(
                out,
                "  {target}: answered by {} using {}",
                p.engine, p.dataset
            )
            .unwrap();
            writeln!(out, "    model:   {}", p.model).unwrap();
            writeln!(out, "    routing: {}", p.routing).unwrap();
            if !p.dataset_sources.is_empty() {
                writeln!(out, "    the dataset records its own sources, e.g.:").unwrap();
                for src in &p.dataset_sources {
                    writeln!(out, "      · {src}").unwrap();
                }
            }
        }
        None => writeln!(
            out,
            "  {target}: no aqueous solver has characterised this vessel"
        )
        .unwrap(),
    }
    // BRD-002: where the *contents* came from, when some of them came
    // from a recipe rather than a bottle.
    //
    // A named material is a reviewed estimate of a composition — "vinegar"
    // is 5% acetic acid within a stated range, with lot assumptions and an
    // evidence record behind it. `explain` reported the solver's
    // provenance and said nothing about that, so a vessel whose answer
    // rests half on a recipe looked exactly like one that did not. The
    // dispense is in the log, pinned to a recipe id and version precisely
    // so a replay cannot drift, which is also what makes it reportable.
    out.push_str(&material_provenance(bench, target));
    // What every other dataset says about the same vessel.
    let vessel = vessel.clone();
    match paths.as_mut() {
        None => writeln!(out, "    (no engine available to compare paths)").unwrap(),
        Some(engine) => {
            let compared = engine.compare_paths(&vessel);
            if compared.is_empty() {
                writeln!(out, "    (nothing aqueous to compare)").unwrap();
            } else {
                writeln!(out, "  the same question, asked of every dataset:").unwrap();
                for path in compared {
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
                            writeln!(
                                out,
                                "    {:<14} pH {ph:.3} · I = {ionic_strength:.4} m{solids}",
                                path.dataset
                            )
                            .unwrap();
                            writeln!(out, "      {}", path.model).unwrap();
                        }
                        kerotakis_phreeqc::PathOutcome::CannotExpress { missing_elements } => {
                            writeln!(
                                out,
                                "    {:<14} cannot express this problem (no {})",
                                path.dataset,
                                missing_elements.join(", ")
                            )
                            .unwrap()
                        }
                        kerotakis_phreeqc::PathOutcome::Failed { detail } => {
                            let short: String = detail.lines().next().unwrap_or("").into();
                            writeln!(out, "    {:<14} could not solve it: {short}", path.dataset)
                                .unwrap()
                        }
                    }
                }
                // Three answers are partly answers about different
                // admissible solids, not three opinions about one activity
                // model — say so rather than leave the reader to infer it.
                let coverage = kerotakis_phreeqc::derived::phase_coverage();
                writeln!(
                    out,
                    "  note: only {} of {} mineral phases exist in every dataset, so the answers",
                    coverage.shared, coverage.total
                )
                .unwrap();
                writeln!(
                    out,
                    "  can differ in which solids they may form, not only in activity model"
                )
                .unwrap();
            }
        }
    }
    Ok(out)
}

/// The sweep: a matrix of vessel states through the real solver stack.
///
/// Cases are built from a small alphabet — a solvent, things to dissolve,
/// acids and bases, an oxidant and a reductant, heat, cold, time — because
/// the interesting failures have all come from *combinations* nobody
/// thought to try, not from any one operator.
fn run_sweep(filter: Option<&str>) -> ! {
    let solvents = ["water 100mL", "water 250mL"];
    let solutes = [
        "NaCl 0.1mol",
        "KCl 0.05mol",
        "CaCl2 0.05mol",
        "CuSO4 0.01mol",
        "AgNO3 0.01mol",
        "Na2CO3 0.02mol",
        "NaHCO3 0.05mol",
        "CaCO3 2g",
        "MgSO4 0.02mol",
        "KMnO4 0.001mol",
        "FeSO4 0.005mol",
        "Na2S2O3 0.5g",
        "H2O2 0.05mol",
        "gypsum 1g",
        "CaO 1g",
        "MnO2 0.2g",
    ];
    let reagents = [
        "",
        "HCl 0.02mol",
        "NaOH 0.02mol",
        "CH3COOH 0.05mol",
        "H3PO4 0.02mol",
        "AgNO3 0.005mol",
        "KMnO4 0.0005mol",
        "MnO2 0.1g",
    ];
    let finishers = [
        "",
        "heat v1 5kJ",
        "cool v1 20kJ",
        "wait 30s",
        "stir v1",
        "evaporate v1 0.3",
    ];

    let mut cases: Vec<(String, String)> = Vec::new();
    for solvent in solvents {
        for solute in solutes {
            for reagent in reagents {
                for finish in finishers {
                    let mut script = format!("add v1 {solvent}\nadd v1 {solute}\n");
                    if !reagent.is_empty() {
                        script.push_str(&format!("add v1 {reagent}\n"));
                    }
                    if !finish.is_empty() {
                        script.push_str(finish);
                        script.push('\n');
                    }
                    let name = format!("{solvent} + {solute} + [{reagent}] + [{finish}]");
                    cases.push((name, script));
                }
            }
        }
    }
    if let Some(f) = filter {
        cases.retain(|(name, _)| name.contains(f));
    }

    let total = cases.len();
    eprintln!("sweeping {total} states…");
    let mut findings: Vec<sweep::Finding> = Vec::new();
    let mut refused = 0usize;
    let mut solved = 0usize;

    for (i, (name, script)) in cases.iter().enumerate() {
        if i % 200 == 0 && i > 0 {
            eprintln!("  {i}/{total}");
        }
        let mut bench = Bench::new();
        let mut stack = build_stack();
        let mut ok = true;
        for line in script.lines() {
            let Ok(Some(op)) = parse_op(line) else {
                continue;
            };
            let before = bench.vessel(VesselId(0)).cloned().ok();
            match bench.step_with(op, &mut stack, &kerotakis_safety::ReactiveGroupScreen) {
                Ok(events) => {
                    // A stated refusal is a correct outcome, not a failure.
                    if events
                        .iter()
                        .any(|e| matches!(e, Event::SolverFailed { .. }))
                    {
                        refused += 1;
                    }
                    if let (Some(before), Ok(after)) = (before, bench.vessel(VesselId(0))) {
                        findings.extend(sweep::check(name, &before, after, &events));
                    }
                }
                Err(_) => ok = false,
            }
        }
        if ok {
            solved += 1;
        }
    }

    println!("\nswept {total} states — {solved} ran, {refused} step(s) stated a refusal");
    if findings.is_empty() {
        println!("no invariant was violated.");
        std::process::exit(0);
    }
    let mut by_kind: std::collections::BTreeMap<String, Vec<&sweep::Finding>> = Default::default();
    for f in &findings {
        by_kind
            .entry(format!("{:?}", f.invariant))
            .or_default()
            .push(f);
    }
    println!("{} violation(s):", findings.len());
    for (kind, group) in &by_kind {
        println!("\n  {kind} — {} case(s)", group.len());
        for f in group.iter().take(4) {
            println!("    · {}", f.detail);
            println!("      in: {}", f.case);
        }
        if group.len() > 4 {
            println!("    · … and {} more", group.len() - 4);
        }
    }
    std::process::exit(1);
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
    // load_dir finds codex/i18n/*.toml as well as the English source. A
    // loader that walks the directory itself sees only English, and
    // nothing would fail — the catalogue would simply stop being German.
    let _ = files;
    let loaded = kerotakis_codex::Codex::load_dir(std::path::Path::new(dir)).unwrap_or_else(|e| {
        eprintln!("kero codex: {dir}: {e}");
        std::process::exit(1);
    });
    all.reactions.extend(loaded.reactions);
    all.models.extend(loaded.models);
    all
}

/// Validate the codex — structurally, and by replaying every entry through
/// the real solvers. A claim the chemistry no longer supports is an error,
/// which is the whole point of the format.
fn codex_lint(dir: &str) -> ! {
    let codex = load_codex(dir);
    let vocabulary = load_vocabulary(dir);
    let mut problems = codex.structural_problems();
    let mut stale: Vec<String> = Vec::new();

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
        for event in &observed {
            if let Event::SolverFailed { solver, detail, .. } = event {
                problems.push(format!(
                    "{}: solver '{solver}' could not answer during the setup: {detail}",
                    entry.id
                ));
            }
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
        // Collect what the engine actually produced, so the prose can be
        // checked against it — see `kerotakis_codex::prose`.
        let mut values = kerotakis_codex::prose::EngineValues::default();
        for e in &observed {
            match e {
                Event::Added { moles, .. }
                | Event::Dissolved { moles, .. }
                | Event::Precipitated { moles, .. }
                | Event::GasEvolved { moles, .. }
                | Event::GasAbsorbed { moles, .. }
                | Event::Consumed { moles, .. }
                | Event::Plated { moles, .. }
                | Event::Electrolysed { moles, .. }
                | Event::Reacted { moles, .. } => values.moles.push(moles.0),
                _ => {}
            }
            // The mass equivalent of whatever moved. An entry that says
            // "weigh out 5.844 g" is quoting an amount the bench knows,
            // but once the salt dissolves there is no portion left to
            // compute a mass from, so it has to be caught at the event.
            if let Event::Added { species, moles, .. }
            | Event::Dissolved { species, moles, .. }
            | Event::Precipitated { species, moles, .. }
            | Event::Consumed { species, moles, .. }
            | Event::Plated { species, moles, .. }
            | Event::Electrolysed { species, moles, .. }
            | Event::GasEvolved { species, moles, .. }
            | Event::GasAbsorbed { species, moles, .. } = e
            {
                if let Some(d) = kerotakis_core::species::lookup(species) {
                    values.grams.push(moles.0 * d.molar_mass);
                }
            }
            match e {
                Event::TemperatureChanged { from, to, .. } => {
                    values.celsius.push(from.to_celsius());
                    values.celsius.push(to.to_celsius());
                    // Prose usually quotes the *rise*, not the reading: "a
                    // rise of roughly 10 °C" is the sentence an author
                    // writes, and it is as much a result as either endpoint.
                    values.celsius.push((to.0 - from.0).abs());
                }
                Event::ThermalEquilibrium { temperature, .. } => {
                    values.celsius.push(temperature.to_celsius())
                }
                Event::SolutionCharacterized { ph, .. } => values.ph.push(*ph),
                _ => {}
            }
            if let Event::Reacted { seconds, .. } = e {
                values.seconds.push(*seconds);
            }
        }
        // Molar masses are engine-known quantities and get quoted as such
        // — "172 g of gypsum is 36 g of water" is arithmetic on the
        // registry, not a number from thin air.
        for d in kerotakis_core::species::REGISTRY {
            values.grams.push(d.molar_mass);
        }
        for v in &bench.vessels {
            values.celsius.push(v.temperature.to_celsius());
            values.grams.push(v.mass().0);
            values.seconds.push(v.elapsed_seconds);
            if let Some(info) = &v.solution {
                values.ph.push(info.ph);
            }
            for p in &v.contents {
                values.moles.push(p.moles.0);
                if let Some(d) = kerotakis_core::species::lookup(&p.species) {
                    values.grams.push(p.moles.0 * d.molar_mass);
                }
            }
        }
        stale.extend(
            kerotakis_codex::prose::unsupported(entry, &values)
                .into_iter()
                .map(|q| format!("{}: {}", entry.id, q.describe())),
        );

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
        if !stale.is_empty() {
            println!(
                "  prose: {} number(s) in register text that this replay does not account for —",
                stale.len()
            );
            for s in stale.iter().take(20) {
                println!("    · {s}");
            }
            if stale.len() > 20 {
                println!("    · … and {} more", stale.len() - 20);
            }
        }
        let pred = codex.prediction_audit();
        println!(
            "  predictions: {} questions, {}/{} wrong answers say what believing them reveals",
            pred.predictions, pred.diagnosed, pred.distractors
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

fn codex_export(dir: &str, out_path: &str) -> ! {
    let codex = load_codex(dir);
    let vocabulary = load_vocabulary(dir);
    let export = kerotakis_codex::CodexExport::build(&codex, &vocabulary);
    let json = serde_json::to_string(&export).unwrap_or_else(|e| {
        eprintln!("kero codex export: serialization failed: {e}");
        std::process::exit(1);
    });
    if let Some(parent) = std::path::Path::new(out_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).unwrap_or_else(|e| {
                eprintln!("kero codex export: cannot create {}: {e}", parent.display());
                std::process::exit(1);
            });
        }
    }
    std::fs::write(out_path, &json).unwrap_or_else(|e| {
        eprintln!("kero codex export: cannot write {out_path}: {e}");
        std::process::exit(1);
    });
    eprintln!(
        "codex: exported {} reactions, {} models, {} concepts → {out_path}",
        export.reactions.len(),
        export.models.len(),
        export.concepts.len(),
    );
    std::process::exit(0);
}

fn codex_concepts(dir: &str) -> ! {
    let codex = load_codex(dir);
    for (concept, entries) in codex.concept_index() {
        println!("{concept:<28} {}", entries.join(", "));
    }
    std::process::exit(0);
}

fn properties_usage() -> ! {
    eprint!("kero properties — temperature-dependent property correlations\n\nusage:\n  kero properties water [--at 25C] [--json]   full water property table\n  kero properties <property> <arg>=<value>...  single property lookup\n\nproperties:\n");
    for p in kerotakis_core::properties::PROPERTIES {
        eprintln!("  {:<24} {}", p.name, p.description);
    }
    eprintln!("\nexamples:\n  kero properties water\n  kero properties water --at 50C\n  kero properties water --at 310K --json\n  kero properties water-density T=298.15\n  kero properties henry gas=CO2 T=298.15");
    std::process::exit(2);
}

fn calc_usage() -> ! {
    eprint!("kero calc — evaluate a named physical relation\n\nusage: kero calc <relation> <arg>=<value>... [--json]\n\nrelations:\n");
    for r in kerotakis_core::relations::RELATIONS {
        eprintln!(
            "  {:<24} {}\n{:>28}{}\n{:>28}{}",
            r.name, r.equation, "", r.args, "", r.purpose
        );
    }
    eprintln!("\nexamples:\n  kero calc nernst e0=0.3419 n=2 a=0.01 T=298.15\n  kero calc arrhenius A=1e10 Ea=50000 T=298.15\n  kero calc henderson-hasselbalch pKa=4.76 cA=0.1 cB=0.01\n  kero calc debye-huckel z=2 I=0.01\n  kero calc ionic-strength 1:0.1 -1:0.1 2:0.05 -2:0.1\n  kero calc van-t-hoff dH=-57000 K1=1e14 T1=298.15 T2=373.15\n  kero calc eyring dG=65000 T=298.15");
    std::process::exit(2);
}

fn usage() -> ! {
    eprintln!(
        "kerotakis — a virtual laboratory that computes real chemistry\n\
         \n\
         usage:\n\
         \x20 kero                       interactive bench\n\
         \x20 kero run FILE.lab [--json] replay a command script\n\
         \x20 kero study FILE.lab --vary add:v1:HCl=0.005..0.02:4\n\
         \x20        --collect ph@v1[,…] [--csv]   run it varied over a parameter\n\
         \x20 kero fit FILE.lab --param rate:REACTION:pre_exponential\n\
         \x20        --data observations.csv --observe amount:SPECIES@vN\n\
         \x20        --bounds LO..HI --loss sse    fit one curated rate constant\n\
         \x20 kero serve --mcp           the bench as an MCP server (stdio)\n\
         \x20 kero species               list known species\n\
         \x20 kero materials             list the named household and school bottles\n\
         \x20 kero find <word>           search both halves of the shelf\n\
         \x20 kero lessons               list the shipped .lab lessons\n\
         \x20 kero coverage curiosity [--smoke] [--json]\n\
         \x20 kero calc <relation> ...   evaluate a named physical relation\n\
         \x20 kero properties water     temperature-dependent property table\n\
         \x20 kero balance \"Mg + O2 -> MgO\"   find the coefficients\n\
         \x20 kero balance exercise list|unusable [--dir codex] [--limit N]\n\
         \x20 kero balance exercise show <id>\n\
         \x20 kero balance exercise check <id> <c1,c2,…>\n\
         \x20 kero balance exercise answer <id>\n\
         \x20 kero provenance lint       validate source/distribution policy\n\
         \x20 kero mechanism inspect FILE.yaml [--json]\n\
         \x20 kero mechanism rates FILE.yaml --volume-l L --temperature-k K\n\
         \x20        --feed SPECIES=MOLES [--feed ...] [--json]\n\
         \x20 kero mechanism simulate FILE.yaml --seconds S --volume-l L\n\
         \x20        --temperature-k K --feed SPECIES=MOLES [--samples N] [--json]\n\
         \n\
         bench commands (REPL and .lab files):\n\
         \x20 add <vessel> <species> <amount><mol|g|mL> [@ <T>C]\n\
         \x20 heat <vessel> <energy><J|kJ>\n\
         \x20 cool <vessel> <energy><J|kJ>\n\
         \x20 stir <vessel>\n\
         \x20 wait <duration><s|min|h>\n\
         \x20 seal <vessel> <volume><mL|L>          close over a finite headspace\n\
         \x20 regulate <vessel> <pressure> <volume>  hold gas at fixed pressure\n\
         \x20 sweep <vessel> <pressure>              purge with nitrogen\n\
         \x20 open <vessel>                          vent headspace to the room\n\
         \x20 ignite <vessel>                        hold a flame to it\n\
         \x20 decant <from> <to> <fraction>\n\
         \x20 filter <from> <to>                     solids stay, liquid passes\n\
         \x20 evaporate <vessel> <fraction>\n\
         \x20 dilute <vessel> <volume><mL|L>         add water by volume\n\
         \x20 distil <from> <to> <frac|energy> [stages <n>]\n\
         \x20 drain <from> <to>                      lower layer through stopcock\n\
         \x20 transport <v..> from <inlet> to <recv> steps <n> [courant <f>]\n\
         \x20 titrate <v> <titrant> [<c>M] <step><mL|L> until <endpoint> [max <n>]\n\
         \x20   endpoint: ph <target> | pe <op> <value> | colour persists\n\
         \x20 measure <vessel> <thermometer|balance|ph|pressure|conductivity|uvvis|calorimeter>\n\
         \x20 look <vessel>                          observe with your eyes\n\
         \x20 cell <vessel> <vessel>                 wire two half-cells\n\
         \x20 electrolyse <vessel> <current>A <time><s|min|h>\n\
         \x20 grind <vessel> <species> <diameter>um\n\
         \x20 irradiate <vessel> <wavelength>nm <irradiance>W/m2\n\
         \x20 new                                    create a vessel\n\
         \x20 inspect [vessel]                       show state\n\
         \x20 explain [vessel]                       provenance\n\
         \x20 register <lv1|lv2|lv3>                 detail level\n\
         \x20 species                                list available species\n\
         \x20 quit"
    );
    std::process::exit(2);
}

fn mechanism_command(args: &[String]) -> ! {
    match args.first().map(String::as_str) {
        Some("inspect") => inspect_mechanism(&args[1..]),
        Some("rates") => mechanism_rates(&args[1..]),
        Some("simulate") => simulate_mechanism(&args[1..]),
        _ => mechanism_usage(),
    }
}

fn mechanism_usage() -> ! {
    eprintln!(
        "usage:\n\
         \x20 kero mechanism inspect FILE.yaml [--json]\n\
         \x20 kero mechanism rates FILE.yaml --volume-l L --temperature-k K \\\n\
         \x20      --feed SPECIES=MOLES [--feed ...] [--json]\n\
         \x20 kero mechanism simulate FILE.yaml --seconds S --volume-l L \\\n\
         \x20      --temperature-k K --feed SPECIES=MOLES [--feed ...] [--samples N] [--json]"
    );
    std::process::exit(2);
}

fn inspect_mechanism(args: &[String]) -> ! {
    let Some(path) = args.first() else {
        mechanism_usage();
    };
    let text = std::fs::read_to_string(path).unwrap_or_else(|error| {
        eprintln!("kero mechanism: cannot read {path}: {error}");
        std::process::exit(1);
    });
    let mechanism =
        kerotakis_core::kinetics::mechanism::parse_yaml(&text).unwrap_or_else(|error| {
            eprintln!("kero mechanism: {path}: {error}");
            std::process::exit(1);
        });
    // Compilation is part of inspection: a file that parses but cannot lower
    // into the runtime evaluator is not a usable mechanism.
    let arena = kerotakis_core::kinetics::mechanism::MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    debug_assert_eq!(network.reactions.len(), mechanism.summary().reactions);
    let summary = mechanism.summary();
    if args.iter().any(|arg| arg == "--json") {
        println!(
            "{}",
            serde_json::to_string(&summary).expect("mechanism summary is serializable")
        );
    } else {
        println!(
            "{}: {} species, {} reactions; elements {}",
            summary.name,
            summary.species,
            summary.reactions,
            summary.elements.join(", ")
        );
        for reaction in summary.reaction_details {
            println!(
                "  {}  {}  [{}{}] order {:.3}; A={:.6e}, b={:.6}, Ea={:.6} J/mol{}{}",
                reaction.id,
                reaction.equation,
                reaction.rate_model,
                if reaction.reversible {
                    ", reversible"
                } else {
                    ""
                },
                reaction.total_order,
                reaction.pre_exponential,
                reaction.temperature_exponent,
                reaction.activation_energy_j_per_mol,
                reaction
                    .low_pressure_pre_exponential
                    .map_or_else(String::new, |value| format!(", A0={value:.6e}")),
                if reaction.pressure_points_pa.is_empty() {
                    String::new()
                } else {
                    format!(", pressure grid={:?} Pa", reaction.pressure_points_pa)
                }
            );
        }
    }
    std::process::exit(0);
}

#[derive(Debug)]
struct MechanismRatesArgs {
    path: String,
    volume_litres: f64,
    temperature_k: f64,
    feeds: Vec<(String, f64)>,
    json: bool,
}

#[derive(serde::Serialize)]
struct MechanismRatesOutput<'a> {
    mechanism: &'a str,
    volume_litres: f64,
    temperature_k: f64,
    pressure_pa: f64,
    reaction_rates: Vec<MechanismReactionRates<'a>>,
    species_rates: Vec<MechanismSpeciesRate<'a>>,
    rate_determining_step: Option<RateDeterminingStep<'a>>,
    rate_determining_criterion: &'static str,
}

#[derive(serde::Serialize)]
struct MechanismReactionRates<'a> {
    reaction: &'a str,
    equation: &'a str,
    forward_moles_per_litre_second: f64,
    reverse_moles_per_litre_second: f64,
    net_moles_per_litre_second: f64,
}

#[derive(serde::Serialize)]
struct MechanismSpeciesRate<'a> {
    species: &'a str,
    net_production_moles_per_litre_second: f64,
}

#[derive(serde::Serialize)]
struct RateDeterminingStep<'a> {
    reaction: &'a str,
    equation: &'a str,
    absolute_net_moles_per_litre_second: f64,
}

#[derive(Debug)]
struct MechanismSimulationArgs {
    path: String,
    seconds: f64,
    volume_litres: f64,
    temperature_k: f64,
    feeds: Vec<(String, f64)>,
    sample_intervals: usize,
    json: bool,
}

#[derive(serde::Serialize)]
struct MechanismSimulationOutput<'a> {
    mechanism: &'a str,
    duration_seconds: f64,
    volume_litres: f64,
    temperature_k: f64,
    initial_pressure_pa: f64,
    final_pressure_pa: f64,
    initial_moles: Vec<MechanismSpeciesAmount<'a>>,
    final_moles: Vec<MechanismSpeciesAmount<'a>>,
    sample_intervals: usize,
    samples: Vec<MechanismTrajectoryPoint<'a>>,
    extents: Vec<MechanismReactionExtent<'a>>,
    statistics: MechanismSimulationStatistics,
}

#[derive(serde::Serialize)]
struct MechanismSpeciesAmount<'a> {
    species: &'a str,
    moles: f64,
}

#[derive(serde::Serialize)]
struct MechanismTrajectoryPoint<'a> {
    elapsed_seconds: f64,
    pressure_pa: f64,
    moles: Vec<MechanismSpeciesAmount<'a>>,
}

#[derive(serde::Serialize)]
struct MechanismReactionExtent<'a> {
    reaction: &'a str,
    equation: &'a str,
    moles: f64,
}

#[derive(Default, serde::Serialize)]
struct MechanismSimulationStatistics {
    accepted_steps: usize,
    rejected_steps: usize,
    nonlinear_iterations: usize,
    nonlinear_failures: usize,
    depletion_events: usize,
    constrained_commits: usize,
}

impl MechanismSimulationStatistics {
    fn include(&mut self, statistics: kerotakis_core::kinetics::IntegrationStatistics) {
        self.accepted_steps += statistics.accepted_steps;
        self.rejected_steps += statistics.rejected_steps;
        self.nonlinear_iterations += statistics.nonlinear_iterations;
        self.nonlinear_failures += statistics.nonlinear_failures;
        self.depletion_events += statistics.depletion_events;
        self.constrained_commits += statistics.constrained_commits;
    }
}

fn simulation_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn simulation_number(text: &str, flag: &str, allow_zero: bool) -> Result<f64, String> {
    let value = text
        .parse::<f64>()
        .map_err(|_| format!("{flag} requires a number, got '{text}'"))?;
    if !value.is_finite() || value < 0.0 || (!allow_zero && value == 0.0) {
        let range = if allow_zero {
            "non-negative"
        } else {
            "positive"
        };
        return Err(format!("{flag} must be finite and {range}, got {value}"));
    }
    Ok(value)
}

fn parse_mechanism_rates_args(args: &[String]) -> Result<MechanismRatesArgs, String> {
    let path = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| "rates requires a mechanism file".to_string())?;
    let mut volume_litres = None;
    let mut temperature_k = None;
    let mut feeds = Vec::new();
    let mut json = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--volume-l" => {
                let text = simulation_value(args, &mut index, "--volume-l")?;
                volume_litres = Some(simulation_number(&text, "--volume-l", false)?);
            }
            "--temperature-k" => {
                let text = simulation_value(args, &mut index, "--temperature-k")?;
                temperature_k = Some(simulation_number(&text, "--temperature-k", false)?);
            }
            "--feed" => {
                let text = simulation_value(args, &mut index, "--feed")?;
                let (species, amount) = text
                    .split_once('=')
                    .ok_or_else(|| format!("--feed expects SPECIES=MOLES, got '{text}'"))?;
                if species.is_empty() {
                    return Err("--feed species cannot be empty".to_string());
                }
                feeds.push((
                    species.to_string(),
                    simulation_number(amount, "--feed amount", false)?,
                ));
            }
            "--json" => json = true,
            option => return Err(format!("unknown rates option '{option}'")),
        }
        index += 1;
    }
    if feeds.is_empty() {
        return Err("rates requires at least one --feed SPECIES=MOLES".to_string());
    }
    Ok(MechanismRatesArgs {
        path,
        volume_litres: volume_litres.ok_or_else(|| "rates requires --volume-l".to_string())?,
        temperature_k: temperature_k.ok_or_else(|| "rates requires --temperature-k".to_string())?,
        feeds,
        json,
    })
}

fn validate_mechanism_feeds(
    mechanism: &kerotakis_core::kinetics::mechanism::ParsedMechanism,
    feeds: &[(String, f64)],
    command: &str,
) {
    for (species, _) in feeds {
        if !mechanism
            .species_names()
            .any(|candidate| candidate == species)
        {
            eprintln!(
                "kero mechanism {command}: feed species '{species}' is not declared by the mechanism"
            );
            std::process::exit(2);
        }
    }
}

fn mechanism_vessel(volume_litres: f64, temperature_k: f64, feeds: &[(String, f64)]) -> Vessel {
    let mut vessel = Vessel::new(VesselId(0), "mechanism reactor");
    vessel.temperature = Kelvin(temperature_k);
    vessel.headspace = Headspace::Sealed {
        volume: Liters(volume_litres),
    };
    for (species, moles) in feeds {
        vessel.deposit(SpeciesId::new(species), Moles(*moles), Phase::Gas);
    }
    vessel.refresh_pressure();
    vessel
}

fn mechanism_rates(args: &[String]) -> ! {
    const RDS_CRITERION: &str =
        "smallest non-zero absolute net progress rate among currently active reactions";
    let args = parse_mechanism_rates_args(args).unwrap_or_else(|error| {
        eprintln!("kero mechanism rates: {error}");
        mechanism_usage();
    });
    let text = std::fs::read_to_string(&args.path).unwrap_or_else(|error| {
        eprintln!("kero mechanism: cannot read {}: {error}", args.path);
        std::process::exit(1);
    });
    let mechanism =
        kerotakis_core::kinetics::mechanism::parse_yaml(&text).unwrap_or_else(|error| {
            eprintln!("kero mechanism: {}: {error}", args.path);
            std::process::exit(1);
        });
    validate_mechanism_feeds(&mechanism, &args.feeds, "rates");
    let arena = kerotakis_core::kinetics::mechanism::MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    let vessel = mechanism_vessel(args.volume_litres, args.temperature_k, &args.feeds);

    let mut species_rates = mechanism
        .species_names()
        .map(|species| (species, 0.0))
        .collect::<Vec<_>>();
    let reaction_rates = network
        .reactions
        .iter()
        .map(|reaction| {
            let rates = reaction.rates_now(&vessel);
            for term in reaction.stoichiometry {
                let (_, rate) = species_rates
                    .iter_mut()
                    .find(|(species, _)| *species == term.species)
                    .expect("compiled reaction species is declared by its mechanism");
                *rate += term.coefficient * rates.net;
            }
            MechanismReactionRates {
                reaction: reaction.id,
                equation: reaction.equation,
                forward_moles_per_litre_second: rates.forward,
                reverse_moles_per_litre_second: rates.reverse,
                net_moles_per_litre_second: rates.net,
            }
        })
        .collect::<Vec<_>>();
    let rate_determining_step = reaction_rates
        .iter()
        .filter(|rates| {
            rates.net_moles_per_litre_second.abs()
                > 1e-12
                    * (rates.forward_moles_per_litre_second + rates.reverse_moles_per_litre_second)
                        .max(f64::MIN_POSITIVE)
        })
        .min_by(|left, right| {
            left.net_moles_per_litre_second
                .abs()
                .total_cmp(&right.net_moles_per_litre_second.abs())
        })
        .map(|rates| RateDeterminingStep {
            reaction: rates.reaction,
            equation: rates.equation,
            absolute_net_moles_per_litre_second: rates.net_moles_per_litre_second.abs(),
        });
    let output = MechanismRatesOutput {
        mechanism: network.id,
        volume_litres: args.volume_litres,
        temperature_k: args.temperature_k,
        pressure_pa: vessel.pressure.0,
        reaction_rates,
        species_rates: species_rates
            .into_iter()
            .map(|(species, rate)| MechanismSpeciesRate {
                species,
                net_production_moles_per_litre_second: rate,
            })
            .collect(),
        rate_determining_step,
        rate_determining_criterion: RDS_CRITERION,
    };
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&output).expect("mechanism rates are serializable")
        );
    } else {
        println!(
            "{}: instantaneous rates at {:.3} K, {:.6e} Pa",
            output.mechanism, output.temperature_k, output.pressure_pa
        );
        for rates in &output.reaction_rates {
            println!(
                "  {}: forward {:.9e}, reverse {:.9e}, net {:+.9e} mol/(L s)",
                rates.reaction,
                rates.forward_moles_per_litre_second,
                rates.reverse_moles_per_litre_second,
                rates.net_moles_per_litre_second
            );
        }
        for rate in &output.species_rates {
            println!(
                "  {}: net production {:+.9e} mol/(L s)",
                rate.species, rate.net_production_moles_per_litre_second
            );
        }
        if let Some(step) = output.rate_determining_step {
            println!(
                "  instantaneous rate-determining candidate: {} ({:.9e} mol/(L s)); {}",
                step.reaction, step.absolute_net_moles_per_litre_second, RDS_CRITERION
            );
        } else {
            println!("  no active net reaction; no rate-determining candidate");
        }
    }
    std::process::exit(0);
}

fn parse_mechanism_simulation_args(args: &[String]) -> Result<MechanismSimulationArgs, String> {
    const MAX_SAMPLE_INTERVALS: usize = 100_000;
    let path = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| "simulate requires a mechanism file".to_string())?;
    let mut seconds = None;
    let mut volume_litres = None;
    let mut temperature_k = None;
    let mut feeds = Vec::new();
    let mut sample_intervals = 1;
    let mut json = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--seconds" => {
                let text = simulation_value(args, &mut index, "--seconds")?;
                seconds = Some(simulation_number(&text, "--seconds", true)?);
            }
            "--volume-l" => {
                let text = simulation_value(args, &mut index, "--volume-l")?;
                volume_litres = Some(simulation_number(&text, "--volume-l", false)?);
            }
            "--temperature-k" => {
                let text = simulation_value(args, &mut index, "--temperature-k")?;
                temperature_k = Some(simulation_number(&text, "--temperature-k", false)?);
            }
            "--feed" => {
                let text = simulation_value(args, &mut index, "--feed")?;
                let (species, amount) = text
                    .split_once('=')
                    .ok_or_else(|| format!("--feed expects SPECIES=MOLES, got '{text}'"))?;
                if species.is_empty() {
                    return Err("--feed species cannot be empty".to_string());
                }
                feeds.push((
                    species.to_string(),
                    simulation_number(amount, "--feed amount", false)?,
                ));
            }
            "--samples" => {
                let text = simulation_value(args, &mut index, "--samples")?;
                sample_intervals = text
                    .parse::<usize>()
                    .map_err(|_| format!("--samples requires a positive integer, got '{text}'"))?;
                if sample_intervals == 0 {
                    return Err("--samples must be positive".to_string());
                }
                if sample_intervals > MAX_SAMPLE_INTERVALS {
                    return Err(format!(
                        "--samples cannot exceed {MAX_SAMPLE_INTERVALS} intervals"
                    ));
                }
            }
            "--json" => json = true,
            option => return Err(format!("unknown simulate option '{option}'")),
        }
        index += 1;
    }
    if feeds.is_empty() {
        return Err("simulate requires at least one --feed SPECIES=MOLES".to_string());
    }
    Ok(MechanismSimulationArgs {
        path,
        seconds: seconds.ok_or_else(|| "simulate requires --seconds".to_string())?,
        volume_litres: volume_litres.ok_or_else(|| "simulate requires --volume-l".to_string())?,
        temperature_k: temperature_k
            .ok_or_else(|| "simulate requires --temperature-k".to_string())?,
        feeds,
        sample_intervals,
        json,
    })
}

fn mechanism_amounts<'a>(
    mechanism: &'a kerotakis_core::kinetics::mechanism::ParsedMechanism,
    vessel: &Vessel,
) -> Vec<MechanismSpeciesAmount<'a>> {
    mechanism
        .species_names()
        .map(|species| MechanismSpeciesAmount {
            species,
            moles: vessel.moles_of(&SpeciesId::new(species)).0,
        })
        .collect()
}

fn simulate_mechanism(args: &[String]) -> ! {
    let args = parse_mechanism_simulation_args(args).unwrap_or_else(|error| {
        eprintln!("kero mechanism simulate: {error}");
        mechanism_usage();
    });
    let text = std::fs::read_to_string(&args.path).unwrap_or_else(|error| {
        eprintln!("kero mechanism: cannot read {}: {error}", args.path);
        std::process::exit(1);
    });
    let mechanism =
        kerotakis_core::kinetics::mechanism::parse_yaml(&text).unwrap_or_else(|error| {
            eprintln!("kero mechanism: {}: {error}", args.path);
            std::process::exit(1);
        });
    validate_mechanism_feeds(&mechanism, &args.feeds, "simulate");

    let arena = kerotakis_core::kinetics::mechanism::MechanismArena::default();
    let network = mechanism.compile_in(&arena);
    let mut vessel = mechanism_vessel(args.volume_litres, args.temperature_k, &args.feeds);
    let initial_pressure_pa = vessel.pressure.0;
    let initial_moles = mechanism_amounts(&mechanism, &vessel);
    let mut samples = vec![MechanismTrajectoryPoint {
        elapsed_seconds: 0.0,
        pressure_pa: initial_pressure_pa,
        moles: mechanism_amounts(&mechanism, &vessel),
    }];
    let mut total_extents = vec![0.0; network.reactions.len()];
    let mut statistics = MechanismSimulationStatistics::default();
    let interval_seconds = args.seconds / args.sample_intervals as f64;
    for sample in 1..=args.sample_intervals {
        let report = kerotakis_core::kinetics::advance_network_with_options(
            &mut vessel,
            interval_seconds,
            &network,
            kerotakis_core::kinetics::IntegrationOptions::default(),
        )
        .unwrap_or_else(|error| {
            eprintln!("kero mechanism simulate: {error}");
            std::process::exit(1);
        });
        for (reaction, extent) in &report.extents {
            let index = network
                .reactions
                .iter()
                .position(|candidate| candidate.id == reaction.id)
                .expect("integration reports a reaction from its input network");
            total_extents[index] += extent.0;
        }
        statistics.include(report.statistics);
        samples.push(MechanismTrajectoryPoint {
            elapsed_seconds: args.seconds * sample as f64 / args.sample_intervals as f64,
            pressure_pa: vessel.pressure.0,
            moles: mechanism_amounts(&mechanism, &vessel),
        });
    }
    let final_moles = mechanism_amounts(&mechanism, &vessel);
    let extents = network
        .reactions
        .iter()
        .zip(total_extents)
        .filter(|(_, extent)| extent.abs() > 0.0)
        .map(|(reaction, extent)| MechanismReactionExtent {
            reaction: reaction.id,
            equation: reaction.equation,
            moles: extent,
        })
        .collect();
    let output = MechanismSimulationOutput {
        mechanism: network.id,
        duration_seconds: args.seconds,
        volume_litres: args.volume_litres,
        temperature_k: args.temperature_k,
        initial_pressure_pa,
        final_pressure_pa: vessel.pressure.0,
        initial_moles,
        final_moles,
        sample_intervals: args.sample_intervals,
        samples,
        extents,
        statistics,
    };
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&output).expect("mechanism simulation is serializable")
        );
    } else {
        println!(
            "{}: simulated {:.6} s at {:.3} K in {:.6} L; pressure {:.6e} -> {:.6e} Pa",
            output.mechanism,
            output.duration_seconds,
            output.temperature_k,
            output.volume_litres,
            output.initial_pressure_pa,
            output.final_pressure_pa
        );
        for amount in &output.final_moles {
            println!("  {}: {:.9e} mol", amount.species, amount.moles);
        }
        for sample in &output.samples {
            println!(
                "  t={:.9e} s: pressure {:.9e} Pa",
                sample.elapsed_seconds, sample.pressure_pa
            );
        }
        for extent in &output.extents {
            println!("  {}: extent {:.9e} mol", extent.reaction, extent.moles);
        }
        println!(
            "  solver: {} accepted, {} rejected, {} nonlinear iterations",
            output.statistics.accepted_steps,
            output.statistics.rejected_steps,
            output.statistics.nonlinear_iterations
        );
    }
    std::process::exit(0);
}

fn repl() {
    println!("kerotakis 0.0.1 — the bench is ready. 'help' lists commands.");
    let mut session = Session {
        bench: Bench::new(),
        register: Register::default(),
        json: false,
        stack: build_stack(),
        paths: kerotakis_phreeqc::PhreeqcEquilibrator::new().ok(),
        quests: Vec::new(),
        quest_states: Default::default(),
        aliases: Default::default(),
        masks: Vec::new(),
        cover_masks: Vec::new(),
        sealed_vessels: Default::default(),
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
        // KID-17: a help text that names two thirds of the grammar teaches
        // that the other third does not exist. `magnet`, `smell`, `test`,
        // `chromatograph` and `react` were all landed, all working, and all
        // absent from every surface a reader has — the children's corpus in
        // KIDS.md lost four experiments to verbs that were already there.
        // The invariant is held by a test: every verb in `script::VERBS`
        // appears here.
        if line == "help" {
            println!(
                "put things in    add <v> <name> <amount><mol|g|mL> [@ <T>C] · stock <name> <amount>\n\
                 energy           heat/cool <v> <E><J|kJ> · ignite <v> · irradiate <v> <nm> <W/m2>\n\
                 time and mixing  wait <t><s|min|h> · stir <v> [<rpm> <s>] · grind <v> <name> <um>\n\
                 gas boundary     seal <v> <vol> · regulate <v> <bar> <vol> · sweep <v> <bar> · open <v>\n\
                 move things      decant/filter/drain <from> <to> · dilute <v> <vol> · evaporate <v> <frac>\n\
                 \x20                distil <from> <to> <frac|energy> [stages <n>] · magnet <from> <to>\n\
                 \x20                centrifuge <v> <g> <s> · transport <v..> from <in> to <out> steps <n>\n\
                 look and measure look <v> · smell <v> · measure <v> <thermometer|balance|ph|…>\n\
                 \x20                test <v> <splint|limewater|…> · chromatograph <v> · particles [v]\n\
                 electrochemistry cell <v> <v> · electrolyse <v> <A> <t>   (each half-cell wants its metal)\n\
                 analysis         titrate <v> <name> [<c>M] <step><mL|L> until <ph <t>|pe <op> <v>|colour persists>\n\
                 named reactions  react <v> <esterification|saponification>\n\
                 the bench        new [beaker|flask|tube|cylinder|crucible] · remove <v> · inspect [v]\n\
                 \x20                register <lv1|lv2|lv3> · explain [v] · quest · quit\n\
                 what is here     species (pure substances) · materials (household bottles) · find <word>"
            );
            continue;
        }
        if line == "species" {
            for s in species::REGISTRY {
                println!("  {:<10} {} ({})", s.key, s.name, s.formula);
            }
            println!(
                "  — {} species. `find <word>` searches these and the named materials.",
                species::REGISTRY.len()
            );
            continue;
        }
        // BRD-002: the cabinet is searchable. `species` lists several
        // hundred rows and no materials at all, which is a catalogue a
        // learner can scroll past rather than one they can use.
        if let Some(query) = line.strip_prefix("find ") {
            print_cabinet_search(query, &session.bench);
            continue;
        }
        if line == "find" {
            println!("  usage: find <word> — searches species keys, names, formulas and material aliases");
            continue;
        }
        if line == "materials" {
            print_materials();
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
                let text = explain_text(&self.bench, &mut self.paths, target)?;
                if self.json {
                    println!(
                        "{}",
                        self.mask_json(json_explain(self.bench.log.len(), target, &text))
                    );
                } else {
                    // Provenance prose names species too; sealed unknowns
                    // keep their mask on in `explain` like everywhere else.
                    print!("{}", self.mask_for(target, &text));
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
                    if self.json {
                        println!(
                            "{}",
                            self.mask_json(json_particles(self.bench.log.len(), v))
                        );
                    } else {
                        println!("  {} — what the particles are doing:", v.id);
                        // The census names ions the vessel line never
                        // shows; a sealed unknown's `Na+` row is the
                        // answer in a different font, so it wears the
                        // mask like every other rendered line.
                        print!(
                            "{}",
                            self.mask_for(
                                v.id,
                                &kerotakis_core::particles::census(v, 30).render(self.register)
                            )
                        );
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
                    println!(
                        "{}",
                        self.mask_json(json_inspect(self.bench.log.len(), &vessels))
                    );
                } else {
                    for v in vessels {
                        self.print_vessel(v);
                    }
                }
                Ok(())
            }
            "quest" => self.quest_command(&words[1..]),
            _ => {
                // Sealed unknowns: the learner types the alias; the parser
                // gets the truth. Whole-word substitution only.
                let mut used_alias = false;
                let unmasked = if self.aliases.is_empty() {
                    trimmed.to_string()
                } else {
                    trimmed
                        .split_whitespace()
                        .map(|w| match self.aliases.get(w) {
                            Some(real) => {
                                used_alias = true;
                                real.as_str()
                            }
                            None => w,
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                };
                match parse_op(&unmasked)? {
                    Some(op) => {
                        // Sealing keys on the alias being typed, not on the
                        // species: `add v2 NaCl` from the learner's own
                        // shelf must not put v2 behind the covers just
                        // because NaCl is also this quest's secret.
                        if used_alias {
                            if let Operator::Add { vessel, .. } = &op {
                                self.sealed_vessels.insert(*vessel);
                            }
                        }
                        self.run_op(op)
                    }
                    None => Ok(()),
                }
            }
        }
    }

    /// Re-mask a rendered line for sealed unknowns: each masked species'
    /// key and display name both become the alias. Display-layer only.
    fn mask(&self, line: &str) -> String {
        Self::apply_masks(&self.masks, line)
    }

    /// The mask plus the covers — for lines about a vessel the unknown
    /// has touched, where an ion row is the answer in a different font.
    fn mask_covered(&self, line: &str) -> String {
        Self::apply_masks(&self.cover_masks, &Self::apply_masks(&self.masks, line))
    }

    /// The right mask for a line about one specific vessel.
    fn mask_for(&self, vessel: VesselId, line: &str) -> String {
        if self.sealed_vessels.contains(&vessel) {
            self.mask_covered(line)
        } else {
            self.mask(line)
        }
    }

    /// The `--json` stream with sealed unknowns masked, and a declaration
    /// of what the placeholders mean.
    ///
    /// This was a KNOWN LIMIT rather than an oversight, and the reasoning
    /// against fixing it was sound as far as it went: hosts key rendering,
    /// colours and spectra off species ids, so rewriting those ids is a
    /// change hosts have to be told about. The conclusion — leave the true
    /// keys in — did not follow. The mask is the whole point of a sealed
    /// unknown, and a guarantee that holds in the REPL and not on the wire
    /// is not a guarantee; it is a guarantee plus a way around it, and
    /// `--json` is the easier of the two to read.
    ///
    /// So the ids *are* rewritten, and the change is additive in the way
    /// that matters: a `sealed` object on every line declares which
    /// vessels the unknown has touched and which ids in this line are
    /// placeholders rather than species. A host that ignores it sees ids
    /// it cannot look up and renders them as the unknowns they are, which
    /// is the correct behaviour; a host that reads it can say so
    /// deliberately. Neither is shown the answer.
    ///
    /// The rule is the text layer's rule, applied to a tree instead of a
    /// line: the unknown's own alias masks everywhere, because that
    /// species *is* the sealed sample wherever it turns up; the `covers`
    /// (its dissociation ions, which have no alias to type) join in only
    /// once some vessel is sealed, because chloride from a bottle of acid
    /// the learner poured themselves is not the sample's and must not be
    /// dressed as it. Object keys are masked as well as values — a
    /// speciation map keyed by species id would otherwise carry the answer
    /// in its keys.
    fn mask_json(&self, value: serde_json::Value) -> serde_json::Value {
        if self.masks.is_empty() {
            return value;
        }
        const NONE: &[(String, String)] = &[];
        let covers: &[(String, String)] = if self.sealed_vessels.is_empty() {
            NONE
        } else {
            &self.cover_masks
        };
        let mut masked = Self::mask_json_value(&self.masks, covers, value);
        if let Some(map) = masked.as_object_mut() {
            map.insert("sealed".into(), self.sealed_declaration());
        }
        masked
    }

    fn mask_json_value(
        masks: &[(String, String)],
        covers: &[(String, String)],
        value: serde_json::Value,
    ) -> serde_json::Value {
        let mask = |text: &str| Self::apply_masks(covers, &Self::apply_masks(masks, text));
        match value {
            serde_json::Value::String(text) => serde_json::Value::String(mask(&text)),
            serde_json::Value::Array(items) => serde_json::Value::Array(
                items
                    .into_iter()
                    .map(|item| Self::mask_json_value(masks, covers, item))
                    .collect(),
            ),
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.into_iter()
                    .map(|(key, item)| (mask(&key), Self::mask_json_value(masks, covers, item)))
                    .collect(),
            ),
            other => other,
        }
    }

    /// What the placeholders in this line mean — the additive field.
    ///
    /// Deliberately not the mapping. Telling a host that `sample A` is
    /// really sodium chloride would be the leak with an extra step; what
    /// it is told is which ids are placeholders and which vessels the
    /// unknown has reached, which is everything it needs to render
    /// honestly and nothing it needs to give the game away.
    fn sealed_declaration(&self) -> serde_json::Value {
        let mut vessels: Vec<usize> = self.sealed_vessels.iter().map(|v| v.0).collect();
        vessels.sort_unstable();
        let mut placeholders: Vec<&str> = self
            .masks
            .iter()
            .chain(self.cover_masks.iter())
            .map(|(_, alias)| alias.as_str())
            .collect();
        placeholders.sort_unstable();
        placeholders.dedup();
        serde_json::json!({ "vessels": vessels, "placeholders": placeholders })
    }

    fn apply_masks(masks: &[(String, String)], line: &str) -> String {
        let mut out = line.to_string();
        for (real, alias) in masks {
            out = out.replace(real, alias);
            // Every spelling the registry knows for this species: the
            // level-1 census names ions by formula lookup, so the mask
            // matches by key OR formula and covers name and formula both.
            for d in kerotakis_core::species::registry()
                .iter()
                .filter(|d| d.key == real.as_str() || d.formula == real.as_str())
            {
                out = out.replace(d.name, alias);
                out = out.replace(d.formula, alias);
            }
        }
        out
    }

    /// One string a mask table must cover, as (real, alias). Tables stay
    /// longest-real-first so a compound's full name is rewritten before
    /// any of its fragments can shadow it.
    fn add_mask(table: &mut Vec<(String, String)>, real: &str, alias: &str) {
        if table.iter().any(|(r, _)| r == real) {
            return;
        }
        table.push((real.to_string(), alias.to_string()));
        table.sort_by_key(|entry| std::cmp::Reverse(entry.0.len()));
    }

    fn quest_command(&mut self, words: &[&str]) -> Result<(), String> {
        use kerotakis_codex::quest;
        if self.quests.is_empty() {
            if let Ok(specs) = quest::load_dir(std::path::Path::new("quests")) {
                self.quests = specs;
            }
        }
        match words.first().copied() {
            Some("list") | None => {
                if self.quests.is_empty() {
                    println!("  no quests found (looked in ./quests)");
                }
                for spec in &self.quests {
                    let state = self.quest_states.get(&spec.id);
                    let mark = match state {
                        Some(st) if st.complete => "done",
                        Some(_) => "active",
                        None => "     ",
                    };
                    println!(
                        "  {:6} {} — {}",
                        mark,
                        spec.id,
                        spec.title.at(self.register.level())
                    );
                }
                Ok(())
            }
            Some("start") => {
                let id = words.get(1).ok_or("usage: quest start <id>")?;
                let spec = self
                    .quests
                    .iter()
                    .find(|s| s.id == *id)
                    .ok_or_else(|| format!("no quest '{id}' — `quest list`"))?
                    .clone();
                self.quest_states.entry(spec.id.clone()).or_default();
                for (alias, real) in &spec.unknowns {
                    self.aliases.insert(alias.clone(), real.clone());
                    Self::add_mask(&mut self.masks, real, alias);
                }
                for (alias, keys) in &spec.covers {
                    for key in keys {
                        Self::add_mask(&mut self.cover_masks, key, alias);
                    }
                }
                println!("  quest started: {}", spec.title.at(self.register.level()));
                println!("  {}", spec.goal.at(self.register.level()));
                if !spec.unknowns.is_empty() {
                    let names: Vec<&str> = spec.unknowns.keys().map(String::as_str).collect();
                    println!(
                        "  sealed on your shelf: {} — add them like any reagent;                          name one with `quest answer <alias> <species>`",
                        names.join(", ")
                    );
                }
                Ok(())
            }
            Some("status") => {
                for spec in &self.quests {
                    let Some(state) = self.quest_states.get(&spec.id) else {
                        continue;
                    };
                    let done = spec
                        .claims
                        .iter()
                        .filter(|c| state.satisfied.contains(&c.id))
                        .count();
                    println!(
                        "  {} — {}/{} claims{}",
                        spec.id,
                        done,
                        spec.claims.len(),
                        if state.complete { " — COMPLETE" } else { "" }
                    );
                    for claim in &spec.claims {
                        let mark = if state.satisfied.contains(&claim.id) {
                            "✓"
                        } else {
                            "·"
                        };
                        println!("    {mark} {}", claim.title.at(self.register.level()));
                    }
                }
                Ok(())
            }
            Some("answer") => {
                let alias = words
                    .get(1)
                    .ok_or("usage: quest answer <alias> <species>")?;
                let guess = words
                    .get(2)
                    .ok_or("usage: quest answer <alias> <species>")?;
                match quest::answer(&self.quests, &mut self.quest_states, alias, guess) {
                    Ok(outputs) => {
                        self.print_quest_outputs(&outputs);
                        Ok(())
                    }
                    Err(msg) => {
                        println!("  {msg}");
                        Ok(())
                    }
                }
            }
            Some(other) => Err(format!(
                "unknown quest command '{other}' (list, start, status, answer)"
            )),
        }
    }

    fn print_quest_outputs(&self, outputs: &[kerotakis_codex::quest::QuestOutput]) {
        use kerotakis_codex::quest::QuestOutput as Q;
        for o in outputs {
            match o {
                Q::Nudge { say, .. } => {
                    println!("  ❯ {}", self.mask(say.at(self.register.level())))
                }
                // Said, not enforced: the mistake is the lesson.
                Q::ConstraintViolated { say, .. } => {
                    println!("  ⚠ {}", self.mask(say.at(self.register.level())))
                }
                Q::ClaimSatisfied { title, .. } => {
                    println!("  ✓ {}", self.mask(title.at(self.register.level())))
                }
                Q::Completed { title, .. } => println!(
                    "  ★ quest complete: {}",
                    self.mask(title.at(self.register.level()))
                ),
            }
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
        // The sealed set follows the matter: `exec_line` seals the vessel
        // an alias was added to, and every transfer out of a sealed
        // vessel seals the destination. Chemistry is untouched — this
        // only decides which lines the covers apply to.
        for event in &events {
            if let kerotakis_core::ops::Event::Transferred { from, to, .. } = event {
                if self.sealed_vessels.contains(from) {
                    self.sealed_vessels.insert(*to);
                }
            }
        }
        if self.json {
            println!(
                "{}",
                self.mask_json(json_step(
                    self.bench.log.len() - 1,
                    &op,
                    &events,
                    &self.bench.vessels
                ))
            );
        } else {
            // The ledger records everything; a person is shown what they
            // could notice, once each. Event lines are not scoped to one
            // vessel, so the covers join the mask as soon as any vessel
            // is sealed — conservative on purpose.
            let masker = if self.sealed_vessels.is_empty() {
                Self::mask
            } else {
                Self::mask_covered
            };
            for line in render_events(&events, self.register) {
                println!("  {}", masker(self, &line));
            }
            // GUI-092: the equation the beaker actually ran, derived from
            // the solved speciation. Silent at lv1 and silent wherever no
            // solver characterised the solution.
            for line in kerotakis_core::render_ionic_for(
                &events,
                &self.bench.vessels,
                self.register,
                kerotakis_core::Locale::EN,
            ) {
                println!("  {}", self.mask(&line));
            }
        }
        if !self.quest_states.is_empty() {
            let outputs = kerotakis_codex::quest::observe(
                &self.quests,
                &mut self.quest_states,
                &events,
                &self.bench,
            );
            self.print_quest_outputs(&outputs);
        }
        Ok(())
    }

    fn print_vessel(&self, v: &Vessel) {
        // Through the mask: `inspect` on a sealed unknown was the one
        // window that printed the vessel's truth unmasked, which made
        // the identification quest a reading exercise.
        for line in render_vessel(v, self.register) {
            println!("  {}", self.mask_for(v.id, &line));
        }
    }
}

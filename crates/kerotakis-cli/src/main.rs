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

mod chart_svg;
mod diagram;
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
        // The metallic state rides on top of the aqueous solve: the series
        // moves electrons over the activities PHREEQC reports, and the
        // products go back through it.
        Ok(aqueous) => solvers.push(Box::new(PhaseEquilibrator::wrapping(Box::new(
            kerotakis_core::DisplacementEquilibrator::wrapping(Box::new(aqueous)),
        )))),
        Err(e) => {
            eprintln!("kero: aqueous engine unavailable ({e}); running without it");
            // Pure-water phase changes still work in the honestly degraded
            // stack; only brine re-speciation is unavailable.
            solvers.push(Box::new(StateEquilibrator));
        }
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
        Some("study") => {
            study::study_command(&args[1..]);
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
                // ✓ marks a verified identity: this species has a curated
                // SMILES whose recomputation by the official IUPAC InChI
                // library (v1.07.5, vendored in inchi-sys) must reproduce
                // the registry InChIKey — enforced in the gate.
                let verified = kerotakis_org::inchi_validate::CURATED_STRUCTURES
                    .iter()
                    .any(|(id, _)| *id == s.key);
                let mark = if verified { "✓" } else { " " };
                println!(
                    "{:<10} {mark} {:<18} {:<8} M={:>8.3} g/mol   [{}]",
                    s.key, s.name, s.formula, s.molar_mass, s.provenance
                );
            }
            println!(
                "
✓ = identity verified: curated structure recomputed by the                  official IUPAC InChI library (1.07.5) matches the registry key"
            );
        }
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
        "bench": { "vessels": vessels },
    })
}

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

/// Everything `explain` says, as a string — the REPL prints it, the MCP
/// server returns it, and building it in one place keeps them identical.
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
        eprintln!("  {:<24} {}\n{:>28}{}", r.name, r.equation, "", r.args);
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
         \x20 kero serve --mcp           the bench as an MCP server (stdio)\n\
         \x20 kero species               list known species\n\
         \x20 kero calc <relation> ...   evaluate a named physical relation\n\
         \x20 kero properties water     temperature-dependent property table\n\
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
         \x20 titrate <v> <titrant> <step><mL|L> until ph <target> [max <n>]\n\
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
                 stir <v> · wait <t><s|min|h> · seal/open <v> · ignite <v>\n\
                 decant/filter <from> <to> · evaporate <v> <frac> · dilute <v> <vol><mL|L>\n\
                 distil <from> <to> <frac|energy> · drain <from> <to>\n\
                 titrate <v> <species> <step><mL|L> until ph <target>\n\
                 measure <v> <thermometer|balance|ph|…> · look <v> · cell <v> <v>\n\
                 electrolyse <v> <A> <t> · grind <v> <species> <um>\n\
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
                let text = explain_text(&self.bench, &mut self.paths, target)?;
                if self.json {
                    println!("{}", json_explain(self.bench.log.len(), target, &text));
                } else {
                    print!("{text}");
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
                        println!("{}", json_particles(self.bench.log.len(), v));
                    } else {
                        println!("  {} — what the particles are doing:", v.id);
                        print!(
                            "{}",
                            kerotakis_core::particles::census(v, 30).render(self.register)
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
                    println!("{}", json_inspect(self.bench.log.len(), &vessels));
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
            println!(
                "{}",
                json_step(self.bench.log.len() - 1, &op, &events, &self.bench.vessels)
            );
        } else {
            // The ledger records everything; a person is shown what they
            // could notice, once each.
            for line in render_events(&events, self.register) {
                println!("  {line}");
            }
        }
        Ok(())
    }

    fn print_vessel(&self, v: &Vessel) {
        for line in render_vessel(v, self.register) {
            println!("  {line}");
        }
    }
}

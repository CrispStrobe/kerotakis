//! `kero serve --mcp` — the bench as a Model Context Protocol server.
//!
//! JSON-RPC 2.0 over stdio, newline-delimited, per the MCP specification.
//! The server exists for one reason (PLAN.md, "Curation is verifiable, so
//! drafting can be assisted"): a drafting agent should be able to *run its
//! own claims* — execute a setup, read the computed numbers, compare them
//! with its prose — before a human reads the draft.
//!
//! Two rules keep it honest:
//! - Bench tool output IS the `--json` contract, built by the same
//!   functions the CLI's `--json` mode uses, so the two cannot drift.
//! - stdout carries protocol messages only; everything else is stderr.

use std::io::{BufRead, Write};

use kerotakis_core::script::{parse_op, parse_vessel};
use kerotakis_core::*;

use crate::{balance_text, build_stack, explain_text};
use crate::{json_explain, json_inspect, json_particles, json_step};

/// Newest first; the newest is also the answer to a version we don't know.
const SUPPORTED_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "Kerotakis is a virtual chemistry laboratory that computes real \
chemistry: PHREEQC for aqueous equilibrium, a Gibbs minimiser over NASA CEA data for heat \
and fire. Drive the bench with `bench_exec`; its state persists across calls until \
`bench_reset`. Every answer carries provenance (`explain`), and a stated refusal is a \
result, not a malfunction — this lab says what it cannot compute rather than guessing. \
Start with `species` to learn what the bench can name.";

/// One persistent bench per server process: an agent's conversation is a
/// session, exactly as a REPL is.
struct BenchSession {
    bench: Bench,
    stack: SolverStack,
    /// A second engine used only for `explain`'s path comparison, so
    /// comparing never disturbs the session's own solver state (the same
    /// split the REPL makes).
    paths: Option<kerotakis_phreeqc::PhreeqcEquilibrator>,
}

impl BenchSession {
    fn new() -> Self {
        BenchSession {
            bench: Bench::new(),
            stack: build_stack(),
            paths: kerotakis_phreeqc::PhreeqcEquilibrator::new().ok(),
        }
    }

    /// Run bench commands, one per line, collecting one contract object
    /// per step. On a bad line the error names it — and the steps already
    /// executed stay executed, which the error also says, because a bench
    /// is not a transaction.
    fn exec_script(&mut self, script: &str) -> Result<String, String> {
        let mut docs: Vec<serde_json::Value> = Vec::new();
        for (lineno, line) in script.lines().enumerate() {
            if let Err(e) = self.exec_line(line, &mut docs) {
                let done: String = docs.iter().map(|d| format!("{d}\n")).collect();
                return Err(format!(
                    "line {}: {e}\n({} step(s) before this line were executed and remain on the bench)\n{done}",
                    lineno + 1,
                    docs.len(),
                ));
            }
        }
        Ok(docs.iter().map(|d| format!("{d}\n")).collect())
    }

    /// One line of the `.lab` grammar — the same dispatch as the REPL's
    /// JSON mode, emitting the same objects via the same builders.
    fn exec_line(&mut self, line: &str, out: &mut Vec<serde_json::Value>) -> Result<(), String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return Ok(());
        }
        let words: Vec<&str> = trimmed.split_whitespace().collect();
        match words[0] {
            // Presentation only: the JSON contract always carries full data.
            "register" => Ok(()),
            "explain" => {
                let target = words
                    .get(1)
                    .map(|w| parse_vessel(w))
                    .transpose()?
                    .unwrap_or(VesselId(0));
                let text = explain_text(&self.bench, &mut self.paths, target)?;
                out.push(json_explain(self.bench.log.len(), target, &text));
                Ok(())
            }
            "particles" | "zoom" => {
                let target = words.get(1).map(|w| parse_vessel(w)).transpose()?;
                for v in self
                    .bench
                    .vessels
                    .iter()
                    .filter(|v| target.is_none() || target == Some(v.id))
                {
                    out.push(json_particles(self.bench.log.len(), v));
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
                out.push(json_inspect(self.bench.log.len(), &vessels));
                Ok(())
            }
            _ => match parse_op(trimmed)? {
                Some(op) => {
                    let events = self
                        .bench
                        .step_with(
                            op.clone(),
                            &mut self.stack,
                            &kerotakis_safety::ReactiveGroupScreen,
                        )
                        .map_err(|e| e.to_string())?;
                    out.push(json_step(
                        self.bench.log.len() - 1,
                        &op,
                        &events,
                        &self.bench.vessels,
                    ));
                    Ok(())
                }
                None => Ok(()),
            },
        }
    }
}

enum ToolError {
    /// Not a tool we serve — a protocol error, not a tool result.
    Unknown,
    /// The tool ran (or could not run) and this is what it has to say.
    Failed(String),
}

pub fn serve(codex_dir: String) -> ! {
    let stdin = std::io::stdin();
    let mut session = BenchSession::new();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let msg: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                reply_error(
                    &serde_json::Value::Null,
                    -32700,
                    &format!("parse error: {e}"),
                );
                continue;
            }
        };
        // No method: a response to a request we never sent. No id: a
        // notification (e.g. notifications/initialized) — never replied to.
        let Some(method) = msg.get("method").and_then(|m| m.as_str()) else {
            continue;
        };
        let Some(id) = msg.get("id").filter(|i| !i.is_null()).cloned() else {
            continue;
        };
        let params = msg
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        match method {
            "initialize" => {
                let requested = params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let version = if SUPPORTED_VERSIONS.contains(&requested) {
                    requested
                } else {
                    SUPPORTED_VERSIONS[0]
                };
                reply(
                    &id,
                    serde_json::json!({
                        "protocolVersion": version,
                        "capabilities": { "tools": {} },
                        "serverInfo": {
                            "name": "kerotakis",
                            "version": env!("CARGO_PKG_VERSION"),
                        },
                        "instructions": INSTRUCTIONS,
                    }),
                );
            }
            "ping" => reply(&id, serde_json::json!({})),
            "tools/list" => reply(&id, serde_json::json!({ "tools": tool_definitions() })),
            "tools/call" => {
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                match call_tool(&mut session, &codex_dir, name, &args) {
                    Ok(text) => reply(&id, tool_result(&text, false)),
                    Err(ToolError::Unknown) => {
                        reply_error(&id, -32602, &format!("unknown tool '{name}'"))
                    }
                    Err(ToolError::Failed(text)) => reply(&id, tool_result(&text, true)),
                }
            }
            other => reply_error(&id, -32601, &format!("method '{other}' is not supported")),
        }
    }
    std::process::exit(0);
}

fn call_tool(
    session: &mut BenchSession,
    codex_dir: &str,
    name: &str,
    args: &serde_json::Value,
) -> Result<String, ToolError> {
    let str_arg = |key: &str| args.get(key).and_then(|v| v.as_str());
    let need = |key: &str| {
        str_arg(key).ok_or_else(|| ToolError::Failed(format!("'{name}' needs a '{key}' string")))
    };
    match name {
        "bench_exec" => session
            .exec_script(need("script")?)
            .map_err(ToolError::Failed),
        "bench_reset" => {
            *session = BenchSession::new();
            Ok("bench reset — a fresh session with one empty vessel (v1)".into())
        }
        "run_lab" => BenchSession::new()
            .exec_script(need("script")?)
            .map_err(ToolError::Failed),
        "explain" => {
            let target = match str_arg("vessel") {
                Some(w) => parse_vessel(w).map_err(ToolError::Failed)?,
                None => VesselId(0),
            };
            explain_text(&session.bench, &mut session.paths, target).map_err(ToolError::Failed)
        }
        "balance" => balance_text(need("equation")?).map_err(ToolError::Failed),
        "species" => {
            use std::fmt::Write as _;
            let mut out = String::new();
            for s in species::REGISTRY {
                writeln!(
                    out,
                    "{:<10} {:<18} {:<8} M={:>8.3} g/mol   [{}]",
                    s.key, s.name, s.formula, s.molar_mass, s.provenance
                )
                .unwrap();
            }
            Ok(out)
        }
        "codex_lint" => {
            // Spawn ourselves: the tool runs the *same* `kero codex lint`
            // the CI runs, so an agent and the CI can never disagree.
            let dir = str_arg("dir").unwrap_or(codex_dir);
            let exe = std::env::current_exe()
                .map_err(|e| ToolError::Failed(format!("cannot locate kero: {e}")))?;
            let out = std::process::Command::new(exe)
                .args(["codex", "lint", "--dir", dir])
                .output()
                .map_err(|e| ToolError::Failed(format!("cannot run codex lint: {e}")))?;
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            if out.status.success() {
                Ok(text)
            } else {
                Err(ToolError::Failed(text))
            }
        }
        _ => Err(ToolError::Unknown),
    }
}

fn tool_definitions() -> serde_json::Value {
    let script_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "script": {
                "type": "string",
                "description": "Bench commands, one per line (the .lab grammar)",
            },
        },
        "required": ["script"],
    });
    serde_json::json!([
        {
            "name": "bench_exec",
            "description": "Execute bench commands, one per line, against this session's \
                persistent bench. Returns one JSON object per step — the same contract as \
                `kero run --json`. A stated refusal (solver_failed / not_yet_modeled) is a \
                result, not a malfunction. The event stream is deliberately complete: events \
                below the human-observability floor are included so the books always balance \
                — presentation filtering (Event::is_observable) is a rendering concern and \
                agents get the ledger. Commands: add <v> <species> <amount><mol|g|mL> \
                [@ <T>C] · heat/cool <v> <E><J|kJ> · stir <v> · wait <n>s · ignite <v> · \
                filter <from> <to> · decant <from> <to> <fraction> · evaporate <v> <fraction> \
                · measure <v> <thermometer|balance|ph> · cell <v> <v> · look <v> · new · inspect [v] · \
                particles [v] · explain [v]. Species names come from the `species` tool.",
            "inputSchema": script_schema,
        },
        {
            "name": "bench_reset",
            "description": "Discard the session bench and start fresh: one empty vessel, \
                an empty operator log.",
            "inputSchema": { "type": "object", "properties": {} },
        },
        {
            "name": "run_lab",
            "description": "Run a complete .lab script on a fresh, throwaway bench. The \
                session bench is not touched. Same commands and same output contract as \
                bench_exec.",
            "inputSchema": script_schema,
        },
        {
            "name": "explain",
            "description": "Where a vessel's standing answer came from: engine, dataset, \
                model, why that path was routed — and the same question asked of every \
                other dataset, disagreement shown rather than hidden.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "vessel": { "type": "string", "description": "v1, v2, … (default v1)" },
                },
            },
        },
        {
            "name": "balance",
            "description": "Balance a chemical equation by the null space of the \
                element-count matrix, charge included. Exact where the system determines \
                an answer; refuses under-determined skeletons rather than guessing. \
                Example: 'Cr2O7-2 + Fe+2 + H+ -> Cr+3 + Fe+3 + H2O'.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "equation": { "type": "string", "description": "Skeleton equation with a -> arrow" },
                },
                "required": ["equation"],
            },
        },
        {
            "name": "species",
            "description": "List every species the bench can name — key, name, formula, \
                molar mass, provenance. `add` only accepts these keys.",
            "inputSchema": { "type": "object", "properties": {} },
        },
        {
            "name": "codex_lint",
            "description": "Replay every codex claim through the real solvers — the same \
                `kero codex lint` the CI runs. Use it to verify a drafted entry before \
                submitting it.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "Codex directory (default: the server's --dir)" },
                },
            },
        },
    ])
}

fn tool_result(text: &str, is_error: bool) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error,
    })
}

fn reply(id: &serde_json::Value, result: serde_json::Value) {
    emit(serde_json::json!({ "jsonrpc": "2.0", "id": id, "result": result }));
}

fn reply_error(id: &serde_json::Value, code: i64, message: &str) {
    emit(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    }));
}

fn emit(msg: serde_json::Value) {
    let mut out = std::io::stdout().lock();
    writeln!(out, "{msg}").ok();
    out.flush().ok();
}

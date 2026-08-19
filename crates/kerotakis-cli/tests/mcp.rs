//! The MCP server speaks JSON-RPC 2.0 over stdio, and its bench output must
//! be the same `--json` contract the CLI emits — verified here by asking
//! both the same question and requiring identical answers, not merely
//! plausible ones (the same discipline as the wasm-bridge CI check).

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Server {
    child: Child,
    stdin: ChildStdin,
    out: BufReader<ChildStdout>,
    next_id: i64,
}

impl Server {
    /// Spawn `kero serve --mcp` and complete the MCP handshake.
    fn start() -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_kero"))
            .args(["serve", "--mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("kero serve --mcp starts");
        let stdin = child.stdin.take().unwrap();
        let out = BufReader::new(child.stdout.take().unwrap());
        let mut s = Server {
            child,
            stdin,
            out,
            next_id: 0,
        };
        let init = s.result(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "kerotakis-test", "version": "0" },
            }),
        );
        assert_eq!(init["protocolVersion"], "2025-06-18", "{init}");
        assert!(init["capabilities"]["tools"].is_object(), "{init}");
        assert_eq!(init["serverInfo"]["name"], "kerotakis", "{init}");
        s.notify("notifications/initialized");
        s
    }

    fn request(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        self.next_id += 1;
        let msg = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        self.out.read_line(&mut line).unwrap();
        let reply: serde_json::Value = serde_json::from_str(&line).expect("reply is JSON");
        assert_eq!(reply["id"], self.next_id, "replies match requests: {reply}");
        reply
    }

    fn result(&mut self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let reply = self.request(method, params);
        assert!(reply.get("error").is_none(), "unexpected error: {reply}");
        reply["result"].clone()
    }

    fn notify(&mut self, method: &str) {
        let msg = serde_json::json!({ "jsonrpc": "2.0", "method": method });
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
    }

    /// Call a tool; return its text content and whether it flagged an error.
    fn call(&mut self, tool: &str, args: serde_json::Value) -> (String, bool) {
        let r = self.result(
            "tools/call",
            serde_json::json!({ "name": tool, "arguments": args }),
        );
        let text = r["content"][0]["text"]
            .as_str()
            .expect("text content")
            .to_string();
        (text, r["isError"].as_bool().unwrap_or(false))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

/// The balance reading from the first step object in a tool's NDJSON output.
fn measured_grams(ndjson: &str) -> f64 {
    let step: serde_json::Value = serde_json::from_str(ndjson.lines().next().unwrap()).unwrap();
    step["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event"] == "measured" && e["unit"] == "g")
        .expect("a mass measurement")["value"]
        .as_f64()
        .unwrap()
}

#[test]
fn handshake_and_tool_listing() {
    let mut s = Server::start();
    let listed = s.result("tools/list", serde_json::json!({}));
    let tools = listed["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    for expected in [
        "bench_exec",
        "bench_reset",
        "run_lab",
        "explain",
        "balance",
        "species",
        "codex_lint",
    ] {
        assert!(
            names.contains(&expected),
            "missing tool {expected}: {names:?}"
        );
    }
    for t in tools {
        assert_eq!(
            t["inputSchema"]["type"], "object",
            "schema for {}",
            t["name"]
        );
        assert!(
            t["description"].as_str().unwrap().len() > 20,
            "a description an agent can act on: {}",
            t["name"]
        );
    }
    // An unknown method is a JSON-RPC error, not silence or a crash.
    let reply = s.request("resources/list", serde_json::json!({}));
    assert_eq!(reply["error"]["code"], -32601, "{reply}");
}

#[test]
fn bench_output_is_the_cli_json_contract() {
    let script = "add v1 water 100mL\nadd v1 NaCl 0.1mol\nnew\nadd v2 water 100mL\n\
                  add v2 AgNO3 0.01mol\ndecant v2 v1 1.0\nmeasure v1 ph\ninspect v1";

    // The CLI's answer to the script.
    let dir = std::env::temp_dir().join(format!("kero-mcp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("mcp.lab");
    std::fs::write(&lab, script).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let cli: Vec<serde_json::Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    // The MCP server's answer to the same script.
    let mut s = Server::start();
    let (text, is_error) = s.call("bench_exec", serde_json::json!({ "script": script }));
    assert!(!is_error, "{text}");
    let mcp: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).expect("every line is JSON"))
        .collect();

    // Identical, step for step — same code path, so exactly, not roughly.
    assert_eq!(cli.len(), mcp.len(), "same number of steps");
    for (c, m) in cli.iter().zip(&mcp) {
        assert_eq!(c, m, "the MCP server and the CLI must answer identically");
    }

    // And the chemistry is the marquee result: silver chloride precipitates.
    assert!(
        mcp.iter().any(|step| step["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event"] == "precipitated" && e["species"] == "AgCl")),
        "expected AgCl to precipitate"
    );

    // The answer explains itself: which engine, which dataset.
    let (text, is_error) = s.call("explain", serde_json::json!({ "vessel": "v1" }));
    assert!(!is_error, "{text}");
    assert!(text.contains("answered by"), "{text}");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_bench_persists_until_reset() {
    let mut s = Server::start();
    let (t, e) = s.call(
        "bench_exec",
        serde_json::json!({ "script": "add v1 water 100mL" }),
    );
    assert!(!e, "{t}");

    // A later call sees the same bench: the balance reads the water.
    let (t, e) = s.call(
        "bench_exec",
        serde_json::json!({ "script": "measure v1 balance" }),
    );
    assert!(!e, "{t}");
    let step: serde_json::Value = serde_json::from_str(t.lines().next().unwrap()).unwrap();
    assert_eq!(step["step"], 1, "the operator log continues across calls");
    let g = measured_grams(&t);
    assert!((g - 100.0).abs() < 1.0, "the water is still there: {g} g");

    // run_lab computes on a throwaway bench and must not touch the session…
    let (t, e) = s.call(
        "run_lab",
        serde_json::json!({ "script": "add v1 water 50mL\nadd v1 NaCl 1g" }),
    );
    assert!(!e, "{t}");
    let (t, _) = s.call(
        "bench_exec",
        serde_json::json!({ "script": "measure v1 balance" }),
    );
    let g = measured_grams(&t);
    assert!(
        (g - 100.0).abs() < 1.0,
        "run_lab leaked into the session: {g} g"
    );

    // …and reset really is a fresh start.
    let (_, e) = s.call("bench_reset", serde_json::json!({}));
    assert!(!e);
    let (t, _) = s.call(
        "bench_exec",
        serde_json::json!({ "script": "measure v1 balance" }),
    );
    let step: serde_json::Value = serde_json::from_str(t.lines().next().unwrap()).unwrap();
    assert_eq!(step["step"], 0, "the log starts over");
    let g = measured_grams(&t);
    assert!(g.abs() < 1e-9, "a reset bench weighs nothing: {g} g");
}

#[test]
fn refusals_and_errors_are_stated() {
    let mut s = Server::start();

    // A bad line is a tool error that names the line — and says what was
    // already executed, because a bench is not a transaction.
    let (text, is_error) = s.call(
        "bench_exec",
        serde_json::json!({ "script": "add v1 water 100mL\nadd v1 unobtainium 1mol" }),
    );
    assert!(is_error, "{text}");
    assert!(
        text.contains("line 2") && text.contains("unknown species"),
        "{text}"
    );
    assert!(text.contains("1 step(s)"), "{text}");

    // An unknown tool is a protocol error, not a guessed answer.
    let reply = s.request(
        "tools/call",
        serde_json::json!({ "name": "transmute", "arguments": {} }),
    );
    assert_eq!(reply["error"]["code"], -32602, "{reply}");

    // balance: exact where the system determines an answer…
    let (text, is_error) = s.call(
        "balance",
        serde_json::json!({ "equation": "Mg + O2 -> MgO" }),
    );
    assert!(!is_error, "{text}");
    assert!(text.contains("2 Mg + O2 → 2 MgO"), "{text}");

    // …and a refusal where it does not (two independent reactions).
    let (text, is_error) = s.call(
        "balance",
        serde_json::json!({ "equation": "C + O2 -> CO + CO2" }),
    );
    assert!(
        is_error,
        "an under-determined skeleton must be refused, got: {text}"
    );
}

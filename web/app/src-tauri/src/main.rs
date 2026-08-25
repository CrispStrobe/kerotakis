//! The Tauri shell: TauriHost's native side (GUI-030).
//!
//! One command, `engine_request`, speaks the same WEB-002 envelope the web
//! worker speaks — the same `{cmd, ...} → result_json` table as
//! `web/app/src/lib/host/engineWorker.ts`, dispatching to the same core.
//! The UI cannot tell the transports apart, which is the EngineHost
//! contract (PROTOCOL.md).
//!
//! The engine here is fully native: IPhreeqc linked in-process, the same
//! solver stack the CLI builds, owned by ONE dedicated engine thread and
//! reached over a channel — the same actor shape as the web worker, and
//! required besides: the solver stack is deliberately !Send (Rc-shared
//! caches), so it must live where it was born.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{mpsc, Mutex};

use kerotakis_core::{
    render_events, render_vessel, Bench, Equilibrator, Operator, PhaseEquilibrator, Register,
    SolverStack, StateEquilibrator, VesselId,
};
use serde_json::{json, Value};

struct NativeLab {
    bench: Bench,
    stack: SolverStack,
    register: Register,
    can_solve: bool,
}

/// Physics + aqueous chemistry + honesty — the CLI's `build_stack`,
/// verbatim in structure. If PHREEQC cannot initialise, the shell still
/// runs, honestly degraded, and `hello` says so.
fn build_stack() -> (SolverStack, bool) {
    // The shared standard order (kerotakis-stack); only the aqueous tail
    // is this host's to choose — the same wrapping as the CLI's.
    let mut can_solve = true;
    let tail: Vec<Box<dyn Equilibrator>> = match kerotakis_phreeqc::PhreeqcEquilibrator::new() {
        Ok(aqueous) => vec![Box::new(PhaseEquilibrator::wrapping(Box::new(
            kerotakis_core::DisplacementEquilibrator::wrapping(Box::new(aqueous)),
        )))],
        Err(_) => {
            can_solve = false;
            vec![Box::new(StateEquilibrator)]
        }
    };
    (kerotakis_stack::standard_stack(tail), can_solve)
}

impl NativeLab {
    fn new() -> Self {
        let (stack, can_solve) = build_stack();
        NativeLab {
            bench: Bench::new(),
            stack,
            register: Register::default(),
            can_solve,
        }
    }

    fn run(&mut self, op: Operator) -> Result<Vec<kerotakis_core::Event>, String> {
        self.bench
            .step_with(op, &mut self.stack, &kerotakis_safety::ReactiveGroupScreen)
            .map_err(|e| e.to_string())
    }

    fn vessel(&self, index: usize) -> Result<&kerotakis_core::Vessel, String> {
        self.bench
            .vessel(VesselId(index))
            .map_err(|e| e.to_string())
    }
}

const PROTOCOL: u32 = 1;

fn dispatch(lab: &mut NativeLab, req: &Value) -> Result<String, String> {
    let cmd = req
        .get("cmd")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing cmd".to_string())?;
    let field = |name: &str| -> Result<&str, String> {
        req.get(name)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing {name}"))
    };
    match cmd {
        "hello" => Ok(json!({
            "protocol": PROTOCOL,
            "can_solve": lab.can_solve,
            "engine_loaded": true,
            "engine_version": env!("CARGO_PKG_VERSION"),
            "git_rev": option_env!("KEROTAKIS_GIT_REV"),
            "registers": ["lv1", "lv2", "lv3"],
            "packs": kerotakis_core::packs_manifest::core_packs(),
        })
        .to_string()),
        "step" => {
            let op: Operator =
                serde_json::from_str(field("operator_json")?).map_err(|e| e.to_string())?;
            let events = lab.run(op)?;
            Ok(json!({
                "events": events,
                "rendered": render_events(&events, lab.register),
                "charts": kerotakis_core::chart::charts_for_events(&events),
                "scene": kerotakis_core::scene(&lab.bench),
                "bench": { "vessels": lab.bench.vessels },
            })
            .to_string())
        }
        "run_script" => {
            let mut steps = Vec::new();
            for (lineno, line) in field("script")?.lines().enumerate() {
                let trimmed = line.trim();
                if let Some(reg) = trimmed.strip_prefix("register ") {
                    lab.register = Register::parse(reg.trim())
                        .ok_or_else(|| format!("unknown level {reg:?}"))?;
                    continue;
                }
                match kerotakis_core::script::parse_op(line) {
                    Ok(None) => {}
                    Ok(Some(op)) => {
                        let events = lab.run(op.clone())?;
                        steps.push(json!({
                            "operator": op,
                            "events": events,
                            "rendered": render_events(&events, lab.register),
                            "charts": kerotakis_core::chart::charts_for_events(&events),
                        }));
                    }
                    Err(e) => return Err(format!("line {}: {e}", lineno + 1)),
                }
            }
            Ok(json!({
                "steps": steps,
                "scene": kerotakis_core::scene(&lab.bench),
                "bench": { "vessels": lab.bench.vessels },
            })
            .to_string())
        }
        "grammar" => {
            let list: Vec<Value> = kerotakis_core::script::VERBS
                .iter()
                .map(|(verb, example)| {
                    if *verb == "react" {
                        let names: Vec<&str> = kerotakis_core::curated::ORG_REACTIONS
                            .iter()
                            .map(|r| r.name)
                            .collect();
                        json!({ "verb": verb, "example": example, "options": names })
                    } else {
                        json!({ "verb": verb, "example": example })
                    }
                })
                .collect();
            Ok(Value::Array(list).to_string())
        }
        "parse" => Ok(match kerotakis_core::script::parse_op(field("line")?) {
            Ok(None) => json!({ "ok": true }).to_string(),
            Ok(Some(op)) => json!({ "ok": true, "operator": op }).to_string(),
            Err(e) => json!({ "ok": false, "error": e }).to_string(),
        }),
        "set_register" => {
            let level = field("level")?;
            lab.register =
                Register::parse(level).ok_or_else(|| format!("unknown level {level:?}"))?;
            Ok("{}".to_string())
        }
        "scene" => {
            serde_json::to_string(&kerotakis_core::scene(&lab.bench)).map_err(|e| e.to_string())
        }
        "state" => Ok(json!({
            "vessels": lab.bench.vessels,
            "steps": lab.bench.log.len(),
        })
        .to_string()),
        "species" => {
            let list: Vec<Value> = kerotakis_core::species::REGISTRY
                .iter()
                .map(|s| {
                    let (hazards, assessed) = kerotakis_safety::hazard_assessment(s.key);
                    let (srgb, solution_srgb) = kerotakis_core::species::shelf_swatch(s);
                    json!({
                        "key": s.key,
                        "name": s.name,
                        "formula": s.formula,
                        "phase": s.standard_phase,
                        "appearance": s.appearance,
                        "srgb": srgb,
                        "solution_srgb": solution_srgb,
                        "flame": s.flame_colour,
                        "density": s.density,
                        "provenance": s.provenance,
                        "hazards": hazards,
                        "hazard_assessed": assessed,
                    })
                })
                .collect();
            Ok(Value::Array(list).to_string())
        }
        "inspect" => {
            let index = req.get("vessel").and_then(Value::as_u64).unwrap_or(0) as usize;
            let v = lab.vessel(index)?;
            Ok(json!({
                "rendered": render_vessel(v, lab.register),
                "vessel": v,
            })
            .to_string())
        }
        "particles" => {
            let index = req.get("vessel").and_then(Value::as_u64).unwrap_or(0) as usize;
            let v = lab.vessel(index)?;
            let census = kerotakis_core::particles::census(v, 30);
            Ok(json!({
                "census": census,
                "rendered": census.render(lab.register),
            })
            .to_string())
        }
        "snapshot" => {
            let snap = serde_json::to_string(&lab.bench).map_err(|e| e.to_string())?;
            Ok(json!({ "snapshot": snap }).to_string())
        }
        "restore" => {
            lab.bench = serde_json::from_str(field("snapshot")?)
                .map_err(|e| format!("the snapshot did not parse: {e}"))?;
            Ok("{}".into())
        }
        "relations" => {
            let list: Vec<Value> = kerotakis_core::relations::RELATIONS
                .iter()
                .map(|r| json!({ "name": r.name, "equation": r.equation, "args": r.args }))
                .collect();
            Ok(Value::Array(list).to_string())
        }
        "calc" => {
            let name = field("name")?;
            let args: Vec<String> = req
                .get("args")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(match kerotakis_core::relations::evaluate(name, &args) {
                Ok(r) => json!({
                    "ok": true, "value": r.value, "unit": r.unit,
                    "provenance": r.provenance,
                    "lv1": r.lv1, "lv2": r.lv2, "lv3": r.lv3,
                })
                .to_string(),
                Err(e) => json!({ "ok": false, "error": e.to_string() }).to_string(),
            })
        }
        "reset" => {
            lab.bench = Bench::new();
            Ok("{}".to_string())
        }
        other => Err(format!("unknown command '{other}'")),
    }
}

type Reply = mpsc::Sender<Result<String, String>>;

/// The engine actor: one thread owns the (!Send) lab for the process's
/// lifetime; commands arrive as (request, reply) pairs and are answered
/// strictly in order — the same serialization the web worker provides.
struct EngineHandle {
    tx: Mutex<mpsc::Sender<(Value, Reply)>>,
}

fn spawn_engine() -> EngineHandle {
    let (tx, rx) = mpsc::channel::<(Value, Reply)>();
    std::thread::spawn(move || {
        let mut lab = NativeLab::new();
        for (req, reply) in rx {
            let _ = reply.send(dispatch(&mut lab, &req));
        }
    });
    EngineHandle { tx: Mutex::new(tx) }
}

#[tauri::command]
fn engine_request(state: tauri::State<'_, EngineHandle>, req: Value) -> Result<String, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    state
        .tx
        .lock()
        .map_err(|_| "the engine handle is poisoned".to_string())?
        .send((req, reply_tx))
        .map_err(|_| "the engine thread is gone".to_string())?;
    reply_rx
        .recv()
        .map_err(|_| "the engine thread is gone".to_string())?
}

fn main() {
    tauri::Builder::default()
        .manage(spawn_engine())
        .invoke_handler(tauri::generate_handler![engine_request])
        .run(tauri::generate_context!())
        .expect("error while running the Kerotakis shell");
}

// The shell half of PROTOCOL.md conformance: the same dispatch the GUI
// reaches through engine_request, exercised without the webview. The
// wasm host's suite (tools/test-protocol-conformance.mjs) checks the
// other transport; a command that answers differently here is a
// protocol bug even if both GUIs happen to work.
#[cfg(test)]
mod protocol_conformance {
    use super::*;
    use serde_json::json;

    fn ask(lab: &mut NativeLab, req: serde_json::Value) -> serde_json::Value {
        let text = dispatch(lab, &req).expect("dispatch failed");
        serde_json::from_str(&text).expect("result_json did not parse")
    }

    #[test]
    fn hello_carries_identity_and_registers() {
        let mut lab = NativeLab::new();
        let doc = ask(&mut lab, json!({"cmd": "hello"}));
        assert_eq!(doc["protocol"], 1);
        assert!(doc["engine_version"].is_string());
        assert_eq!(doc["registers"], json!(["lv1", "lv2", "lv3"]));
    }

    #[test]
    fn run_script_returns_steps_and_scene() {
        let mut lab = NativeLab::new();
        let doc = ask(
            &mut lab,
            json!({"cmd": "run_script", "script": "new\nadd v1 water 100mL"}),
        );
        assert_eq!(doc["steps"].as_array().map(Vec::len), Some(2));
        assert_eq!(doc["scene"]["scene"], 1);
        // The bench opens with one beaker; `new` stood a second one up —
        // the wasm host answers exactly the same (checked live).
        assert_eq!(doc["scene"]["vessels"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn parse_never_mutates() {
        let mut lab = NativeLab::new();
        let before = dispatch(&mut lab, &json!({"cmd": "state"})).unwrap();
        let ok = ask(
            &mut lab,
            json!({"cmd": "parse", "line": "add v1 water 100mL"}),
        );
        assert_eq!(ok["ok"], true);
        let bad = ask(
            &mut lab,
            json!({"cmd": "parse", "line": "utter nonsense &&&"}),
        );
        assert_eq!(bad["ok"], false);
        assert_eq!(
            dispatch(&mut lab, &json!({"cmd": "state"})).unwrap(),
            before
        );
    }

    #[test]
    fn relations_catalogue_and_calc_agree_with_the_engine() {
        let mut lab = NativeLab::new();
        let list = ask(&mut lab, json!({"cmd": "relations"}));
        assert_eq!(
            list.as_array().map(Vec::len),
            Some(kerotakis_core::relations::RELATIONS.len())
        );
        let doc = ask(
            &mut lab,
            json!({"cmd": "calc", "name": "henderson-hasselbalch",
                   "args": ["pKa=4.76", "cA=0.1", "cB=0.1"]}),
        );
        assert_eq!(doc["ok"], true);
        assert!((doc["value"].as_f64().unwrap() - 4.76).abs() < 1e-9);
        assert!(doc["provenance"].is_string());
        let bad = ask(
            &mut lab,
            json!({"cmd": "calc", "name": "no-such", "args": []}),
        );
        assert_eq!(bad["ok"], false);
    }

    #[test]
    fn snapshot_restores_the_exact_state_and_refuses_garbage() {
        let mut lab = NativeLab::new();
        dispatch(
            &mut lab,
            &json!({"cmd": "run_script", "script": "new\nadd v1 water 100mL"}),
        )
        .unwrap();
        let snap = ask(&mut lab, json!({"cmd": "snapshot"}))["snapshot"]
            .as_str()
            .unwrap()
            .to_string();
        let state_at = dispatch(&mut lab, &json!({"cmd": "state"})).unwrap();
        dispatch(
            &mut lab,
            &json!({"cmd": "run_script", "script": "new flask"}),
        )
        .unwrap();
        assert_ne!(
            dispatch(&mut lab, &json!({"cmd": "state"})).unwrap(),
            state_at
        );
        dispatch(&mut lab, &json!({"cmd": "restore", "snapshot": snap})).unwrap();
        assert_eq!(
            dispatch(&mut lab, &json!({"cmd": "state"})).unwrap(),
            state_at
        );
        assert!(dispatch(
            &mut lab,
            &json!({"cmd": "restore", "snapshot": "{ not json"})
        )
        .is_err());
        // And the bench still answers after the refusal.
        dispatch(&mut lab, &json!({"cmd": "scene"})).unwrap();
    }

    #[test]
    fn unknown_commands_refuse_by_name() {
        let mut lab = NativeLab::new();
        let err = dispatch(&mut lab, &json!({"cmd": "no_such_command"})).unwrap_err();
        assert!(err.contains("no_such_command"), "error was: {err}");
    }
}

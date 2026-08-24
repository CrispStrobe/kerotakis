//! The Tauri shell: TauriHost's native side (GUI-030).
//!
//! One command, `engine_request`, speaks the same WEB-002 envelope the web
//! worker speaks — the same `{cmd, ...} → result_json` table as
//! `web/app/src/lib/host/engineWorker.ts`, dispatching to the same core.
//! The UI cannot tell the transports apart, which is the EngineHost
//! contract (PROTOCOL.md).
//!
//! The engine here is fully native: IPhreeqc linked in-process, the same
//! solver stack the CLI builds, on a background thread via Tauri's command
//! pool. No wasm, no marshalling across a JS boundary.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use kerotakis_core::{
    render_events, render_vessel, Bench, CuratedEquilibrator, Equilibrator, HonestyEquilibrator,
    MixingEquilibrator, Operator, PhaseEquilibrator, Register, SolverStack, StateEquilibrator,
    VesselId,
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
    let mut solvers: Vec<Box<dyn Equilibrator>> = vec![
        Box::new(MixingEquilibrator),
        Box::new(CuratedEquilibrator),
        Box::new(kerotakis_cea::ThermalEquilibrator),
    ];
    let mut can_solve = true;
    match kerotakis_phreeqc::PhreeqcEquilibrator::new() {
        Ok(aqueous) => solvers.push(Box::new(PhaseEquilibrator::wrapping(Box::new(
            kerotakis_core::DisplacementEquilibrator::wrapping(Box::new(aqueous)),
        )))),
        Err(_) => {
            can_solve = false;
            solvers.push(Box::new(StateEquilibrator));
        }
    }
    solvers.push(Box::new(HonestyEquilibrator));
    (SolverStack::new(solvers), can_solve)
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
        })
        .to_string()),
        "step" => {
            let op: Operator =
                serde_json::from_str(field("operator_json")?).map_err(|e| e.to_string())?;
            let events = lab.run(op)?;
            Ok(json!({
                "events": events,
                "rendered": render_events(&events, lab.register),
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
                    json!({
                        "key": s.key,
                        "name": s.name,
                        "formula": s.formula,
                        "phase": s.standard_phase,
                        "appearance": s.appearance,
                        "provenance": s.provenance,
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
        "reset" => {
            lab.bench = Bench::new();
            Ok("{}".to_string())
        }
        other => Err(format!("unknown command '{other}'")),
    }
}

#[tauri::command]
fn engine_request(state: tauri::State<'_, Mutex<NativeLab>>, req: Value) -> Result<String, String> {
    let mut lab = state
        .lock()
        .map_err(|_| "the engine state is poisoned".to_string())?;
    dispatch(&mut lab, &req)
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(NativeLab::new()))
        .invoke_handler(tauri::generate_handler![engine_request])
        .run(tauri::generate_context!())
        .expect("error while running the Kerotakis shell");
}

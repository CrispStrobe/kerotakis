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

/// Minimal base64 (standard alphabet, padding) — the shell's transport
/// is JSON, and a dependency for one decoder would be noise.
fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let val = |c: u8| -> Result<u32, String> {
        ALPHA
            .iter()
            .position(|&a| a == c)
            .map(|p| p as u32)
            .ok_or_else(|| format!("bad base64 byte {c}"))
    };
    let s: Vec<u8> = s.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for chunk in s.chunks(4) {
        if chunk.len() < 2 {
            return Err("truncated base64".into());
        }
        let pads = chunk.iter().filter(|&&c| c == b'=').count();
        let mut acc = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            acc = (acc << 6) | if c == b'=' { 0 } else { val(c)? << 0 };
            let _ = i;
        }
        acc <<= 6 * (4 - chunk.len()) as u32;
        let bytes = [(acc >> 16) as u8, (acc >> 8) as u8, acc as u8];
        let keep = 3usize.saturating_sub(pads + (4 - chunk.len()));
        out.extend_from_slice(&bytes[..keep]);
    }
    Ok(out)
}

pub(crate) fn dispatch(lab: &mut NativeLab, req: &Value) -> Result<String, String> {
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
            let mut list: Vec<Value> = kerotakis_core::species::all_species()
                .into_iter()
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
            list.extend(kerotakis_core::material::all().into_iter().map(|recipe| {
                let pigment_swatch = kerotakis_core::material::pigment_swatch(&recipe)
                    .map(|rgb| [rgb.r, rgb.g, rgb.b]);
                let phase = match recipe.physical_form {
                    kerotakis_data::MaterialPhysicalForm::HomogeneousLiquid
                    | kerotakis_data::MaterialPhysicalForm::Suspension => "liquid",
                    kerotakis_data::MaterialPhysicalForm::GasMixture => "gas",
                    _ => "solid",
                };
                let component_species = recipe
                    .components
                    .iter()
                    .filter_map(|component| {
                        kerotakis_core::species::lookup(&kerotakis_core::SpeciesId::new(
                            &component.species_id,
                        ))
                    })
                    .collect::<Vec<_>>();
                let formula = component_species
                    .iter()
                    .map(|species| species.formula)
                    .collect::<Vec<_>>()
                    .join(" + ");
                let swatch = component_species.iter().find_map(|species| {
                    let (reflective, solution) = kerotakis_core::species::shelf_swatch(species);
                    solution.or(reflective)
                });
                let mut hazards = std::collections::BTreeSet::new();
                let mut assessed = recipe.unresolved_fraction.is_none();
                for species in &component_species {
                    let (component_hazards, component_assessed) =
                        kerotakis_safety::hazard_assessment(species.key);
                    hazards.extend(component_hazards.into_iter().map(str::to_string));
                    assessed &= component_assessed;
                }
                json!({
                    "key": recipe.canonical_key,
                    "name": recipe.name,
                    "formula": formula,
                    "phase": phase,
                    "appearance": recipe.preparation,
                    "srgb": pigment_swatch,
                    "solution_srgb": if pigment_swatch.is_none() { swatch } else { None },
                    "flame": Value::Null,
                    "density": recipe.bulk_density.map(|record| record.value),
                    "provenance": recipe.evidence.source_id,
                    "hazards": hazards,
                    "hazard_assessed": assessed,
                    "material": true,
                })
            }));
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
        "load_pack" => {
            let b64 = field("bytes_b64")?;
            let bytes = base64_decode(b64).map_err(|e| format!("bytes_b64: {e}"))?;
            let doc = kerotakis_data::load_pack(&bytes).map_err(|e| e.to_string())?;
            let recipes = doc.material_recipes.clone();
            let value = serde_json::to_value(&doc).map_err(|e| e.to_string())?;
            let species = kerotakis_core::species_loader::parse_document(&value)?;
            let (added, skipped) = kerotakis_core::species::register_loaded(species);
            let (materials_added, materials_skipped) =
                kerotakis_core::material::register_loaded(recipes);
            Ok(json!({
                "added": added,
                "skipped": skipped,
                "loaded_total": kerotakis_core::species::loaded_count(),
                "materials_added": materials_added,
                "materials_skipped": materials_skipped,
                "materials_loaded_total": kerotakis_core::material::all().len(),
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
                .map(|r| {
                    json!({
                        "name": r.name, "equation": r.equation, "args": r.args,
                        "purpose": r.purpose, "purpose_de": r.purpose_de,
                        "validity": r.validity, "validity_de": r.validity_de,
                    })
                })
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

/// The shell, started.
///
/// Both hosts come through here. On desktop `main.rs` calls it; on iOS
/// there is no Rust binary at all — cargo emits a staticlib, Xcode links
/// it, and the generated `main.mm` calls `ffi::start_app()`, which is the
/// extern "C" entry point `tauri::mobile_entry_point` writes from this
/// function. That is the whole reason the shell is a library.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
    fn base64_decoder_round_trips_reference_vectors() {
        // RFC 4648 vectors + a binary pack-like blob.
        for (plain, enc) in [
            (&b""[..], ""),
            (&b"f"[..], "Zg=="),
            (&b"fo"[..], "Zm8="),
            (&b"foo"[..], "Zm9v"),
            (&b"foob"[..], "Zm9vYg=="),
            (&b"fooba"[..], "Zm9vYmE="),
            (&b"foobar"[..], "Zm9vYmFy"),
        ] {
            assert_eq!(base64_decode(enc).unwrap(), plain, "vector {enc:?}");
        }
        let blob: Vec<u8> = (0u16..=255).map(|b| b as u8).collect();
        // Encode with a tiny reference encoder to close the loop.
        const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut enc = String::new();
        for c in blob.chunks(3) {
            let n = ((c[0] as u32) << 16)
                | ((*c.get(1).unwrap_or(&0) as u32) << 8)
                | (*c.get(2).unwrap_or(&0) as u32);
            let chars = [
                A[(n >> 18) as usize & 63],
                A[(n >> 12) as usize & 63],
                A[(n >> 6) as usize & 63],
                A[n as usize & 63],
            ];
            let keep = 1 + c.len();
            for (i, ch) in chars.iter().enumerate() {
                enc.push(if i < keep { *ch as char } else { '=' });
            }
        }
        assert_eq!(base64_decode(&enc).unwrap(), blob, "binary blob");
    }

    #[test]
    fn unknown_commands_refuse_by_name() {
        let mut lab = NativeLab::new();
        let err = dispatch(&mut lab, &json!({"cmd": "no_such_command"})).unwrap_err();
        assert!(err.contains("no_such_command"), "error was: {err}");
    }
}

//! The bench, in a browser.
//!
//! This is Track A of the plan's wasm strategy: one Rust source compiled to
//! `wasm32-unknown-unknown`, exposing the same `kerotakis-core` API the CLI
//! drives. Two things differ from a native build, both deliberate:
//!
//! * **The aqueous engine is not linked.** A browser cannot load IPhreeqc's
//!   C++ without a separate Emscripten module, so this build carries the
//!   *pre-warmed results* instead. Guided content answers instantly; a
//!   state nobody computed at build time is reported as a stated miss, not
//!   a guess. (Track B — the Emscripten side module — restores full
//!   solving on the web when it lands.)
//! * **Thermal chemistry is fully live.** The Gibbs minimiser is pure Rust,
//!   so heating, calcining and burning are computed in the browser.

use kerotakis_core::{
    render_event, Bench, Equilibrator, Event, HonestyEquilibrator, MixingEquilibrator, Operator,
    Register, SolverStack,
};
use wasm_bindgen::prelude::*;

/// A bench session: state, solvers and the register it speaks in.
#[wasm_bindgen]
pub struct Lab {
    bench: Bench,
    stack: SolverStack,
    aqueous: kerotakis_phreeqc::PhreeqcEquilibrator,
    register: Register,
}

#[wasm_bindgen]
impl Lab {
    /// Open a fresh bench.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Lab, JsError> {
        let aqueous = kerotakis_phreeqc::PhreeqcEquilibrator::new()
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Lab {
            bench: Bench::new(),
            stack: SolverStack::new(vec![
                Box::new(MixingEquilibrator),
                Box::new(kerotakis_core::CuratedEquilibrator),
                Box::new(kerotakis_cea::ThermalEquilibrator),
                Box::new(HonestyEquilibrator),
            ]),
            aqueous,
            register: Register::default(),
        })
    }

    /// Load pre-warmed solver results (postcard bytes, as produced by
    /// `kero prewarm`). Returns how many were added.
    #[wasm_bindgen(js_name = loadResults)]
    pub fn load_results(&mut self, bytes: &[u8]) -> Result<usize, JsError> {
        let data: kerotakis_phreeqc::CacheData =
            postcard::from_bytes(bytes).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(self.aqueous.import_cache(data))
    }

    /// How much detail to render: `lv1` (what you see), `lv2` (equations
    /// and quantities), `lv3` (full numeric detail). More levels can be
    /// added without changing this call.
    #[wasm_bindgen(js_name = setRegister)]
    pub fn set_register(&mut self, register: &str) -> Result<(), JsError> {
        self.register = Register::parse(register).ok_or_else(|| {
            JsError::new(&format!("unknown level '{register}' (try lv1, lv2, lv3)"))
        })?;
        Ok(())
    }

    /// Apply one operator, given as the same JSON the CLI's `--json` mode
    /// emits. Returns `{ events, rendered, bench }`.
    pub fn step(&mut self, operator_json: &str) -> Result<String, JsError> {
        let op: Operator =
            serde_json::from_str(operator_json).map_err(|e| JsError::new(&e.to_string()))?;
        let events = self.run(op)?;
        let rendered: Vec<String> = events
            .iter()
            .map(|e| render_event(e, self.register))
            .collect();
        let doc = serde_json::json!({
            "events": events,
            "rendered": rendered,
            "bench": { "vessels": self.bench.vessels },
        });
        Ok(doc.to_string())
    }

    /// Run a `.lab` lesson script — the same grammar the CLI reads, parsed
    /// by the same code in the core, so a lesson behaves identically in a
    /// browser and on a terminal (and its pre-warmed results match).
    /// Returns one step object per command.
    #[wasm_bindgen(js_name = runScript)]
    pub fn run_script(&mut self, text: &str) -> Result<String, JsError> {
        let mut steps = Vec::new();
        for (lineno, line) in text.lines().enumerate() {
            // Register directives are session state, not chemistry.
            let trimmed = line.trim();
            if let Some(reg) = trimmed.strip_prefix("register ") {
                self.set_register(reg.trim())?;
                continue;
            }
            match kerotakis_core::script::parse_op(line) {
                Ok(None) => {}
                Ok(Some(op)) => {
                    let events = self.run(op.clone())?;
                    let rendered: Vec<String> = events
                        .iter()
                        .map(|e| render_event(e, self.register))
                        .collect();
                    steps.push(serde_json::json!({
                        "operator": op,
                        "events": events,
                        "rendered": rendered,
                    }));
                }
                Err(e) => {
                    return Err(JsError::new(&format!("line {}: {e}", lineno + 1)));
                }
            }
        }
        Ok(serde_json::json!({
            "steps": steps,
            "bench": { "vessels": self.bench.vessels },
        })
        .to_string())
    }

    /// The bench state as JSON.
    pub fn state(&self) -> String {
        serde_json::json!({ "vessels": self.bench.vessels, "steps": self.bench.log.len() })
            .to_string()
    }

    /// Every species the lab knows, as JSON — what a UI offers on a shelf.
    pub fn species(&self) -> String {
        let list: Vec<serde_json::Value> = kerotakis_core::species::REGISTRY
            .iter()
            .map(|s| {
                serde_json::json!({
                    "key": s.key,
                    "name": s.name,
                    "formula": s.formula,
                    "phase": s.standard_phase,
                    "appearance": s.appearance,
                    "provenance": s.provenance,
                })
            })
            .collect();
        serde_json::Value::Array(list).to_string()
    }

    fn run(&mut self, op: Operator) -> Result<Vec<Event>, JsError> {
        // The aqueous solver runs from shipped results; everything else is
        // computed here in the browser.
        let mut stack = SolverStack::new(vec![]);
        std::mem::swap(&mut stack, &mut self.stack);
        let result = self.bench.step_with(
            op,
            &mut CombinedSolver {
                stack: &mut stack,
                aqueous: &mut self.aqueous,
            },
            &kerotakis_safety::ReactiveGroupScreen,
        );
        std::mem::swap(&mut stack, &mut self.stack);
        result.map_err(|e| JsError::new(&e.to_string()))
    }
}

impl Default for Lab {
    fn default() -> Self {
        Self::new().expect("a cache-only lab always constructs")
    }
}

/// Runs the pure-Rust stack and the cache-backed aqueous solver together.
struct CombinedSolver<'a> {
    stack: &'a mut SolverStack,
    aqueous: &'a mut kerotakis_phreeqc::PhreeqcEquilibrator,
}

impl Equilibrator for CombinedSolver<'_> {
    fn name(&self) -> &'static str {
        "wasm-stack"
    }

    fn equilibrate(
        &mut self,
        vessel: &mut kerotakis_core::Vessel,
    ) -> Result<Vec<Event>, kerotakis_core::SolveError> {
        let mut events = Vec::new();
        if self.aqueous.applies(vessel) {
            match self.aqueous.equilibrate(vessel) {
                Ok(mut more) => events.append(&mut more),
                // A cache miss is honest news, not a failure to hide.
                Err(e) => events.push(Event::SolverFailed {
                    vessel: vessel.id,
                    solver: "phreeqc-aqueous (shipped results)".to_string(),
                    detail: e.to_string(),
                }),
            }
        }
        events.append(&mut self.stack.equilibrate(vessel)?);
        Ok(events)
    }
}

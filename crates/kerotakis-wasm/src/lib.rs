//! The bench, in a browser.
//!
//! This is Track A of the plan's wasm strategy: one Rust source compiled to
//! `wasm32-unknown-unknown`, exposing the same `kerotakis-core` API the CLI
//! drives. Two things differ from a native build, both deliberate:
//!
//! * **The aqueous engine is not linked, it is *attached*.** A browser
//!   cannot load IPhreeqc's C++ into `wasm32-unknown-unknown`, so this
//!   build carries pre-warmed results — and, once `setSolver` is called,
//!   reaches the real engine through JavaScript to the Emscripten module
//!   (Track B). Everything above that hook is unchanged: routing, cache,
//!   parsers, the temperature fixed point. Without a solver attached the
//!   bench answers only from shipped results and says so; `canSolve()`
//!   reports which of the two it is, because the difference between a
//!   laboratory and a lesson player is worth surfacing rather than hiding.
//! * **Thermal chemistry is fully live.** The Gibbs minimiser is pure Rust,
//!   so heating, calcining and burning are computed in the browser.

use kerotakis_core::{
    render_event, Bench, Equilibrator, Event, HonestyEquilibrator, MixingEquilibrator, Operator,
    Register, SolverStack,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
                Box::new(kerotakis_core::StateEquilibrator),
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

    /// Hand the bench a real aqueous solver.
    ///
    /// `fn` is called as `fn(databaseTag, phreeqcInput)` and must return a
    /// JSON string `{ "selected": [[...]], "report": "..." }` — exactly
    /// what the linked engine produces. The intended supplier is the
    /// Emscripten build of IPhreeqc (`tools/build-iphreeqc-wasm.sh`), and
    /// `web/kerotakis.mjs` wires the two together.
    ///
    /// Until this is called the browser bench answers only from pre-warmed
    /// results, which is honest and is also not a laboratory. With it, a
    /// state nobody thought to pre-compute is solved rather than refused,
    /// on the same routing and through the same cache as the desktop
    /// build — so the web gets the same answers by the same path instead
    /// of a second implementation that could drift.
    #[wasm_bindgen(js_name = setSolver)]
    pub fn set_solver(&mut self, f: js_sys::Function) {
        self.aqueous.set_hook(Box::new(move |db_tag, input| {
            let out = f
                .call2(
                    &JsValue::NULL,
                    &JsValue::from_str(db_tag),
                    &JsValue::from_str(input),
                )
                .map_err(|e| {
                    // A thrown Error is not a string, and reporting it as
                    // "(no message)" hides exactly the thing worth reading.
                    let detail = e
                        .as_string()
                        .or_else(|| {
                            e.dyn_ref::<js_sys::Error>()
                                .map(|err| String::from(err.message()))
                        })
                        .unwrap_or_else(|| format!("{e:?}"));
                    format!("the JavaScript solver threw: {detail}")
                })?;
            let text = out
                .as_string()
                .ok_or_else(|| "the solver must return a JSON string".to_string())?;
            serde_json::from_str::<kerotakis_phreeqc::SolveOutput>(&text)
                .map_err(|e| format!("the solver's JSON did not parse: {e}"))
        }));
    }

    /// Whether this bench can compute a state nobody pre-computed, or is
    /// limited to shipped results. Worth surfacing rather than hiding: it
    /// is the difference between a laboratory and a lesson player.
    #[wasm_bindgen(js_name = canSolve)]
    pub fn can_solve(&self) -> bool {
        self.aqueous.can_solve()
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

    /// Empty the bench and start again.
    ///
    /// Without this, every experiment ran into whatever the last one left
    /// behind: the freezing demonstration was cooling a beaker that already
    /// held 74 mol of water and a silver chloride precipitate, so nothing
    /// froze and the lesson looked broken rather than crowded. The solver
    /// stack and the register survive — they are the session, not the
    /// chemistry — and so does the pre-warmed cache, which is expensive to
    /// rebuild and correct regardless of what is in the glassware.
    pub fn reset(&mut self) {
        self.bench = Bench::new();
    }

    /// The submicroscopic view of one vessel, as JSON: the census plus the
    /// text a given register would show. Drawn at solved ratios, so it is a
    /// rendering of the answer rather than an illustration of it.
    pub fn particles(&self, vessel: usize) -> Result<String, JsError> {
        let v = self
            .bench
            .vessel(kerotakis_core::VesselId(vessel))
            .map_err(|e| JsError::new(&e.to_string()))?;
        let census = kerotakis_core::particles::census(v, 30);
        let doc = serde_json::json!({
            "census": census,
            "rendered": census.render(self.register),
        });
        Ok(doc.to_string())
    }

    /// What the eye would report about one vessel.
    pub fn look(&self, vessel: usize) -> Result<String, JsError> {
        let v = self
            .bench
            .vessel(kerotakis_core::VesselId(vessel))
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(serde_json::to_string(&kerotakis_core::observe(v))
            .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}")))
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

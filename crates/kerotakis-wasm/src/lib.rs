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

pub mod worker;

use kerotakis_core::{
    localize_events, render_events_in, render_vessel_in, Bench, Equilibrator, Event, Locale,
    Operator, Register, SolverStack,
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
    /// The active quest, if one is running (GUI-066): the ENGINE
    /// evaluates nudges/claims/completion over its own events and
    /// vessel state — a client cannot be trusted to grade itself.
    quest: Option<kerotakis_codex::quest::QuestSpec>,
    quest_states: std::collections::BTreeMap<String, kerotakis_codex::quest::QuestState>,
    /// The language the engine renders its prose in (I18N-5).
    locale: Locale,
}

/// QuestOutput, serialized for the wire: {kind, quest, say|title} with
/// the register texts spelled out — consumers pick their level.
fn quest_outputs_json(outputs: &[kerotakis_codex::quest::QuestOutput]) -> Vec<serde_json::Value> {
    use kerotakis_codex::quest::QuestOutput as Q;
    outputs
        .iter()
        .map(|o| match o {
            Q::Nudge { quest, say } => serde_json::json!({
                "kind": "nudge", "quest": quest,
                "say": { "lv1": say.at(1), "lv2": say.at(2), "lv3": say.at(3) },
            }),
            Q::ConstraintViolated { quest, say } => serde_json::json!({
                "kind": "constraint_violated", "quest": quest,
                "say": { "lv1": say.at(1), "lv2": say.at(2), "lv3": say.at(3) },
            }),
            Q::ClaimSatisfied {
                claim,
                quest,
                title,
            } => serde_json::json!({
                "kind": "claim_satisfied", "quest": quest, "claim": claim,
                "title": { "lv1": title.at(1), "lv2": title.at(2), "lv3": title.at(3) },
            }),
            Q::Completed { quest, title } => serde_json::json!({
                "kind": "completed", "quest": quest,
                "title": { "lv1": title.at(1), "lv2": title.at(2), "lv3": title.at(3) },
            }),
        })
        .collect()
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
            // The shared standard order (kerotakis-stack); the aqueous
            // engine is attached through the JS hook, not in the stack,
            // so the tail is empty here.
            stack: kerotakis_stack::standard_stack(vec![]),
            aqueous,
            register: Register::default(),
            quest: None,
            quest_states: std::collections::BTreeMap::new(),
            locale: Locale::default(),
        })
    }

    /// Start a quest from its exported spec JSON (GUI-066). Replaces any
    /// running quest. The engine owns evaluation from here.
    #[wasm_bindgen(js_name = questStart)]
    pub fn quest_start(&mut self, spec_json: &str) -> Result<(), JsError> {
        let spec: kerotakis_codex::quest::QuestSpec =
            serde_json::from_str(spec_json).map_err(|e| JsError::new(&e.to_string()))?;
        self.quest_states.clear();
        self.quest_states.insert(
            spec.id.clone(),
            kerotakis_codex::quest::QuestState::default(),
        );
        self.quest = Some(spec);
        Ok(())
    }

    /// Abandon the running quest.
    #[wasm_bindgen(js_name = questStop)]
    pub fn quest_stop(&mut self) {
        self.quest = None;
        self.quest_states.clear();
    }

    /// Name a sealed unknown. Correct answers satisfy Identify claims;
    /// wrong ones come back as spoken guidance, never a block.
    #[wasm_bindgen(js_name = questAnswer)]
    pub fn quest_answer(&mut self, alias: &str, guess: &str) -> Result<String, JsError> {
        let Some(spec) = self.quest.clone() else {
            return Err(JsError::new("no quest is running"));
        };
        // A wrong guess is spoken guidance, never a block — so it comes back
        // as a RESULT carrying a stable refusal id, not as an exception
        // carrying an English sentence. "No quest is running" above stays an
        // error, because that one really is one.
        match kerotakis_codex::quest::answer_typed(&[spec], &mut self.quest_states, alias, guess) {
            Ok(outputs) => Ok(serde_json::json!({
                "outputs": quest_outputs_json(&outputs),
            })
            .to_string()),
            Err(refusal) => Ok(serde_json::json!({
                "outputs": [],
                "refusal": refusal,
            })
            .to_string()),
        }
    }

    /// Run the quest evaluator over freshly produced events.
    fn quest_observe(&mut self, events: &[Event]) -> Vec<serde_json::Value> {
        let Some(spec) = self.quest.clone() else {
            return Vec::new();
        };
        let outputs =
            kerotakis_codex::quest::observe(&[spec], &mut self.quest_states, events, &self.bench);
        quest_outputs_json(&outputs)
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

    /// Engine identity for `hello` (GUI-001): version, build revision,
    /// and the registers this engine renders at. The hosts merge this
    /// into their hello answer so a client can pin what it talked to.
    pub fn meta(&self) -> String {
        serde_json::json!({
            "engine_version": env!("CARGO_PKG_VERSION"),
            "git_rev": option_env!("KEROTAKIS_GIT_REV"),
            "registers": ["lv1", "lv2", "lv3"],
            // WEB-003: the model-pack inventory. content_hash is empty
            // until the pack build pipeline stamps it — an HONEST
            // "declared, not yet independently deliverable" state; a
            // client must treat empty-hash packs as built in.
            "packs": kerotakis_core::packs_manifest::core_packs(),
        })
        .to_string()
    }

    /// Run the five release-one chemistry scenarios through this bench's
    /// aqueous path. With a hook this is live IPhreeqc; without one it is an
    /// exact replay from the shipped cache, and any missing state is reported
    /// as a failed case rather than approximated.
    #[wasm_bindgen(js_name = r1Acceptance)]
    pub fn r1_acceptance(&mut self) -> String {
        serde_json::to_string(&kerotakis_phreeqc::acceptance::run_r1_acceptance(
            &mut self.aqueous,
        ))
        .expect("the R1 report is serialisable")
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

    /// Choose the language the engine renders its own prose in.
    ///
    /// Unlike `setRegister` this cannot fail: an unknown tag falls back to
    /// English inside the engine. Someone whose system is set to a
    /// language nobody has translated should see the language we do have,
    /// not an error where the bench used to be.
    #[wasm_bindgen(js_name = setLocale)]
    pub fn set_locale(&mut self, locale: &str) {
        self.locale = Locale::parse(locale);
    }

    /// Apply one operator, given as the same JSON the CLI's `--json` mode
    /// emits. Returns `{ events, rendered, scene, bench }` — `scene` is the
    /// render model (PROTOCOL.md, GUI-003), so one round trip repaints a
    /// bench canvas without a second call.
    pub fn step(&mut self, operator_json: &str) -> Result<String, JsError> {
        let op: Operator =
            serde_json::from_str(operator_json).map_err(|e| JsError::new(&e.to_string()))?;
        // Localised before serialising: the shell reads `hazard` and
        // `real_world` straight off the event rather than through
        // `rendered`, so a German session showed a German frame around
        // English safety prose. This is the first point that knows the
        // language — `bench.rs` and the safety screen do not.
        let events = localize_events(&self.run(op)?, self.locale);
        let rendered = render_events_in(&events, self.register, self.locale);
        let charts = kerotakis_core::chart::charts_for_events(&events);
        // GUI-092: the net ionic equation, where the solved speciation
        // supports one. Empty is the common and honest case.
        let ionic = kerotakis_core::ionic::net_ionic_for(&events, &self.bench.vessels);
        let quest = self.quest_observe(&events);
        let doc = serde_json::json!({
            "events": events,
            "rendered": rendered,
            "charts": charts,
            "ionic": ionic,
            "quest": quest,
            "scene": kerotakis_core::scene(&self.bench),
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
            // I18N: the line may be typed in the session's language, and
            // what comes back is the canonical English the shell must log
            // — a session typed in German has to replay on a bench that
            // never heard of German.
            match kerotakis_core::script::parse_command(line, self.locale) {
                Ok(kerotakis_core::script::Command { operator: None, .. }) => {}
                Ok(kerotakis_core::script::Command {
                    canonical,
                    operator: Some(op),
                }) => {
                    // Localised here too — see `run_operator`. Two call
                    // sites, and a hazard note that reached the reader
                    // through only one of them would be worse than neither.
                    let events = localize_events(&self.run(op.clone())?, self.locale);
                    let rendered = render_events_in(&events, self.register, self.locale);
                    let charts = kerotakis_core::chart::charts_for_events(&events);
                    let ionic = kerotakis_core::ionic::net_ionic_for(&events, &self.bench.vessels);
                    let quest = self.quest_observe(&events);
                    steps.push(serde_json::json!({
                        "canonical": canonical,
                        "operator": op,
                        "events": events,
                        "rendered": rendered,
                        "charts": charts,
                        "ionic": ionic,
                        "quest": quest,
                    }));
                }
                Err(e) => {
                    return Err(JsError::new(&format!("line {}: {e}", lineno + 1)));
                }
            }
        }
        Ok(serde_json::json!({
            "steps": steps,
            "scene": kerotakis_core::scene(&self.bench),
            "bench": { "vessels": self.bench.vessels },
        })
        .to_string())
    }

    /// The render model of the whole bench (PROTOCOL.md, GUI-003):
    /// everything a bench canvas needs, nothing it must derive.
    pub fn scene(&self) -> String {
        serde_json::to_string(&kerotakis_core::scene(&self.bench))
            .expect("the scene is serialisable")
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

    /// The full state of one vessel, rendered for a person. The JSON state
    /// remains available separately as the machine contract.
    pub fn inspect(&self, vessel: usize) -> Result<String, JsError> {
        let v = self
            .bench
            .vessel(kerotakis_core::VesselId(vessel))
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(serde_json::json!({
            "rendered": render_vessel_in(v, self.register, self.locale),
            "vessel": v,
        })
        .to_string())
    }

    /// Validate a single line without executing it.
    ///
    /// Returns `{ ok, operator?, error? }` — the same grammar `runScript`
    /// parses, but the bench is never touched.
    pub fn parse(&self, line: &str) -> String {
        match kerotakis_core::script::parse_command(line, self.locale) {
            Ok(kerotakis_core::script::Command {
                canonical,
                operator,
            }) => serde_json::json!({
                "ok": true,
                "operator": operator,
                "canonical": canonical,
            })
            .to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// The grammar's verb inventory with canonical examples (GUI-029) —
    /// the list a UI's affordance manifest is conformance-checked against.
    pub fn grammar(&self) -> String {
        let list: Vec<serde_json::Value> = kerotakis_core::script::VERBS
            .iter()
            // I18N: `typed` is the same line as a learner of this session's
            // language would write it, for a command bar to offer. Null
            // where the line already is what they would type.
            .map(|(verb, example)| {
                let typed = |line: &str| kerotakis_core::script::example_in(line, self.locale);
                if *verb == "react" {
                    let mut names: Vec<&str> = kerotakis_core::curated::ORG_REACTIONS
                        .iter()
                        .map(|r| r.name)
                        .collect();
                    names.push(kerotakis_core::selectivity::VERB_NAME);
                    serde_json::json!({ "verb": verb, "example": example, "options": names, "typed": typed(example) })
                } else {
                    serde_json::json!({ "verb": verb, "example": example, "typed": typed(example) })
                }
            })
            .collect();
        serde_json::Value::Array(list).to_string()
    }

    /// The named-relations catalogue (CAP-5): name, equation, arg spec,
    /// and (GUI-087/GUI-096) what each one answers, where it holds and
    /// where it came from, in every language the engine ships.
    pub fn relations(&self) -> String {
        let list: Vec<serde_json::Value> = kerotakis_core::relations::RELATIONS
            .iter()
            .map(|r| {
                serde_json::json!({
                    "name": r.name,
                    "equation": r.equation,
                    "args": r.args,
                    "purpose": r.purpose,
                    "purpose_de": r.purpose_de,
                    "validity": r.validity,
                    "validity_de": r.validity_de,
                    "source": r.source,
                    "source_de": r.source_de,
                })
            })
            .collect();
        serde_json::Value::Array(list).to_string()
    }

    /// Evaluate one named relation with `k=v` arguments. The result
    /// carries the value, unit, provenance, and the explanation at every
    /// register — a calculator whose answers say where they came from.
    pub fn calc(&self, name: &str, args_json: &str) -> String {
        let args: Vec<String> = serde_json::from_str(args_json).unwrap_or_default();
        match kerotakis_core::relations::evaluate(name, &args) {
            Ok(r) => serde_json::json!({
                "ok": true,
                "value": r.value,
                "unit": r.unit,
                "provenance": r.provenance,
                "lv1": r.lv1,
                "lv2": r.lv2,
                "lv3": r.lv3,
            })
            .to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// WORLD-003: what this learner can reach, and why.
    ///
    /// The request carries what the engine cannot know — the mode, the
    /// learner's progress, permanent awards, and the active mission's kit —
    /// and the engine answers with stable ids and tagged reasons. Sandbox
    /// availability is DERIVED as full from the installed inventory rather
    /// than read from anything a save could hold stale.
    pub fn catalog(&self, request_json: &str) -> String {
        let request: kerotakis_core::catalog::CatalogRequest =
            serde_json::from_str(request_json).unwrap_or_default();
        let species = kerotakis_core::species::all_species();
        let assessed: Vec<(&str, Vec<&str>, bool)> = species
            .iter()
            .map(|s| {
                let (hazards, assessed) = kerotakis_safety::hazard_assessment(s.key);
                (s.key, hazards, assessed)
            })
            .collect();
        let reagents: Vec<kerotakis_core::catalog::ReagentFacts<'_>> = assessed
            .iter()
            .map(
                |(key, hazards, assessed)| kerotakis_core::catalog::ReagentFacts {
                    key,
                    hazards,
                    assessed: *assessed,
                },
            )
            .collect();
        let packs: Vec<String> = kerotakis_core::packs_manifest::core_packs()
            .into_iter()
            .map(|pack| pack.pack_id)
            .collect();
        serde_json::to_string(&kerotakis_core::catalog::catalog(
            &request, &reagents, &packs,
        ))
        .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }

    /// GUI-095: the balancing exercise — the question, and nothing that
    /// answers it.
    ///
    /// This used to be `balance`, and it returned the whole
    /// `BalanceReport`: the solver's coefficients *and* the composition
    /// matrix. Both are answers. The coefficients are the answer written
    /// down; the matrix is the answer one null space away, and a browser
    /// is a place where anyone can open the network pane. The client
    /// renders the exercise; `balanceMark` marks it and `balanceReveal`
    /// gives it up when the learner asks for it.
    #[wasm_bindgen(js_name = balanceExercise)]
    pub fn balance_exercise(&self, equation: &str) -> String {
        match kerotakis_core::stoich::balance_exercise(equation) {
            Ok(exercise) => match serde_json::to_value(&exercise) {
                Ok(mut value) => {
                    if let Some(map) = value.as_object_mut() {
                        map.insert("ok".into(), serde_json::Value::Bool(true));
                    }
                    value.to_string()
                }
                Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            },
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// GUI-095: mark one answer, engine-side.
    ///
    /// `answer` is a JSON array of integers, one per species in the order
    /// `balanceExercise` listed them. The verdict distinguishes a wrong
    /// answer from a *correct multiple* and names the element that does
    /// not cancel, so a host that was told nothing can still say
    /// precisely what is wrong.
    #[wasm_bindgen(js_name = balanceMark)]
    pub fn balance_mark(&self, equation: &str, answer: &str) -> String {
        let parsed: Vec<i64> = match serde_json::from_str(answer) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::json!({ "ok": false, "error": e.to_string() }).to_string()
            }
        };
        match kerotakis_core::stoich::mark_answer(equation, &parsed) {
            Ok(mark) => match serde_json::to_value(&mark) {
                Ok(mut value) => {
                    if let Some(map) = value.as_object_mut() {
                        map.insert("ok".into(), serde_json::Value::Bool(true));
                    }
                    value.to_string()
                }
                Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
            },
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// GUI-095: the answer, on request — the one call that gives it up.
    ///
    /// Written out as a sentence rather than handed over as a coefficient
    /// vector, so a "show me" for this question cannot be quietly reused
    /// as the marking key for the next one.
    #[wasm_bindgen(js_name = balanceReveal)]
    pub fn balance_reveal(&self, equation: &str) -> String {
        match kerotakis_core::stoich::reveal_answer(equation) {
            Ok(answer) => serde_json::json!({ "ok": true, "equation": answer }).to_string(),
            Err(e) => serde_json::json!({ "ok": false, "error": e.to_string() }).to_string(),
        }
    }

    /// DATA-010: load a species pack (.pack bytes — magic, version,
    /// sha256-verified payload). New species join the shelf and every
    /// lookup; built-ins are never shadowed. Returns the honest count:
    /// { added, skipped, loaded_total }.
    #[wasm_bindgen(js_name = loadPack)]
    pub fn load_pack(&mut self, bytes: &[u8]) -> Result<String, JsError> {
        let doc = kerotakis_data::load_pack(bytes).map_err(|e| JsError::new(&e.to_string()))?;
        let recipes = doc.material_recipes.clone();
        let value = serde_json::to_value(&doc).map_err(|e| JsError::new(&e.to_string()))?;
        let species =
            kerotakis_core::species_loader::parse_document(&value).map_err(|e| JsError::new(&e))?;
        let (added, skipped) = kerotakis_core::species::register_loaded(species);
        let (materials_added, materials_skipped) =
            kerotakis_core::material::register_loaded(recipes);
        Ok(serde_json::json!({
            "added": added,
            "skipped": skipped,
            "loaded_total": kerotakis_core::species::loaded_count(),
            "materials_added": materials_added,
            "materials_skipped": materials_skipped,
            "materials_loaded_total": kerotakis_core::material::all().len(),
        })
        .to_string())
    }

    /// The whole bench as a restorable snapshot (serde round-trip of
    /// `Bench`). The GUI keeps one per log position so undo/scrub is a
    /// restore instead of a reset-and-replay — same determinism, O(1).
    pub fn snapshot(&self) -> Result<String, JsError> {
        serde_json::to_string(&self.bench).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Replace the bench with a snapshot taken by `snapshot()`. The
    /// session (register, solver, caches) survives — this is bench state
    /// only, exactly like `reset`.
    pub fn restore(&mut self, snapshot: &str) -> Result<(), JsError> {
        self.bench = serde_json::from_str(snapshot)
            .map_err(|e| JsError::new(&format!("the snapshot did not parse: {e}")))?;
        Ok(())
    }

    /// The bench state as JSON.
    pub fn state(&self) -> String {
        serde_json::json!({ "vessels": self.bench.vessels, "steps": self.bench.log.len() })
            .to_string()
    }

    /// Every species the lab knows, as JSON — what a UI offers on a shelf.
    ///
    /// GUI-093: the shelf groups by chemical role, and it derives those
    /// roles rather than carrying a second species list of its own. The
    /// four fields below are the engine facts that derivation needs and
    /// the hazard labels lose: `hazard_assessment` collapses `AcidStrong`
    /// and `BaseStrong` into one "corrosive", so the reactive groups
    /// travel unflattened; the element counts come from the engine's own
    /// formula parser rather than a second one written in TypeScript; and
    /// the indicator and solvent flags are membership in the tables that
    /// already decide those behaviours. All four are additive — an older
    /// bridge simply omits them and the client falls back to what the
    /// hazard labels still say.
    pub fn species(&self) -> String {
        let mut list: Vec<serde_json::Value> = kerotakis_core::species::all_species()
            .into_iter()
            .map(|s| {
                let (hazards, assessed) = kerotakis_safety::hazard_assessment(s.key);
                let (srgb, solution_srgb) = kerotakis_core::species::shelf_swatch(s);
                let composition = kerotakis_core::stoich::parse_formula(s.formula).ok();
                serde_json::json!({
                    "key": s.key,
                    "name": s.name,
                    "formula": s.formula,
                    "phase": s.standard_phase,
                    "appearance": s.appearance,
                    "srgb": srgb,
                    "solution_srgb": solution_srgb,
                    "flame": s.flame_colour,
                        "density": s.density,
                    // GUI-099: the room one mole of the substance takes up,
                    // derived here from the same registry mass and density
                    // the bench uses rather than recomputed in TypeScript,
                    // so a deposit can be sized by molar volume even after
                    // the solid has left the vessel and its scene row with
                    // it. Additive: an older bridge omits it and the client
                    // keeps its typical-ionic-solid fallback.
                    "molar_volume_l_per_mol": s.molar_volume_l_per_mol(),
                    "provenance": s.provenance,
                    "hazards": hazards,
                    "hazard_assessed": assessed,
                    "reactive_groups": kerotakis_safety::groups(s.key),
                    "elements": composition.as_ref().map(|f| &f.counts),
                    "charge": composition.as_ref().map(|f| f.charge),
                    // KID-8: a pH-dependent pigment is an indicator to the
                    // shelf's role derivation, however many forms it has.
                    "indicator": kerotakis_core::indicator::is_ph_dependent(s.key),
                    // Water is the solvent the aqueous engine is written
                    // around, which is why it is not in the organic list.
                    "solvent": kerotakis_core::nonaqueous::KNOWN_SOLVENTS.contains(&s.key)
                        || s.key == "water",
                    "enzyme_family": kerotakis_core::enzyme::profile(s.key).map(|profile| match profile.family {
                        kerotakis_core::enzyme::EnzymeFamily::Lactase => "lactase",
                        kerotakis_core::enzyme::EnzymeFamily::Protease => "protease",
                        kerotakis_core::enzyme::EnzymeFamily::Lipase => "lipase",
                        kerotakis_core::enzyme::EnzymeFamily::Catalase => "catalase",
                        kerotakis_core::enzyme::EnzymeFamily::Pepsin => "pepsin",
                        kerotakis_core::enzyme::EnzymeFamily::Bromelain => "bromelain",
                    }),
                    // Catalase owns a stoichiometric kinetic reaction. The
                    // other families expose bounded activity inside conserved
                    // food material without inventing product inventories.
                    "capability": kerotakis_core::enzyme::profile(s.key).map(|_| {
                        if s.key == "catalase" { "modeled_reaction" } else { "modeled_activity" }
                    }),
                })
            })
            .collect();
        list.extend(kerotakis_core::material::all().into_iter().map(|recipe| {
            let pigment_swatch =
                kerotakis_core::material::pigment_swatch(&recipe).map(|rgb| [rgb.r, rgb.g, rgb.b]);
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
            serde_json::json!({
                "key": recipe.canonical_key,
                "name": recipe.name,
                "formula": formula,
                "phase": phase,
                "appearance": recipe.preparation,
                "srgb": pigment_swatch,
                "solution_srgb": if pigment_swatch.is_none() { swatch } else { None },
                "flame": serde_json::Value::Null,
                "density": recipe.bulk_density.map(|record| record.value),
                "provenance": recipe.evidence.source_id,
                "hazards": hazards,
                "hazard_assessed": assessed,
                "material": true,
                // A mixture has no formula of its own to parse, so it
                // carries the keys of what is in it and the shelf takes
                // the roles of those. Component keys, not a second
                // classification: iron filings are iron.
                "components": component_species
                    .iter()
                    .map(|species| species.key)
                    .collect::<Vec<_>>(),
                "protein": kerotakis_core::protein::is_protein_recipe(&recipe.id),
                "capability": if kerotakis_core::protein::is_protein_recipe(&recipe.id) {
                    Some("modeled_observation")
                } else {
                    None
                },
            })
        }));
        serde_json::Value::Array(list).to_string()
    }

    /// Deterministic element-to-shelf coverage generated by the core registry.
    pub fn element_coverage(&self) -> Result<String, JsError> {
        kerotakis_core::element_coverage_json().map_err(|error| JsError::new(&error))
    }

    /// Parse a SMILES string and return molecular identity data.
    pub fn structure(&self, smiles: &str) -> Result<String, JsError> {
        let mol = kerotakis_org::parse_smiles(smiles).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(serde_json::to_string(&mol).unwrap())
    }

    /// Identify functional groups in a SMILES string.
    #[wasm_bindgen(js_name = identifyGroups)]
    pub fn identify_groups(&self, smiles: &str) -> String {
        let groups = kerotakis_org::groups::perceive_groups(smiles);
        serde_json::to_string(&groups).unwrap()
    }

    /// Coverage report: which solvers apply to a vessel's current state.
    pub fn coverage(&self, vessel: usize) -> Result<String, JsError> {
        let v = self
            .bench
            .vessel(kerotakis_core::VesselId(vessel))
            .map_err(|e| JsError::new(&e.to_string()))?;
        let mixing = kerotakis_core::MixingEquilibrator;
        let curated = kerotakis_core::CuratedEquilibrator;
        let thermal = kerotakis_cea::ThermalEquilibrator;
        let honesty = kerotakis_core::HonestyEquilibrator;
        let solvers: Vec<&dyn kerotakis_core::Equilibrator> =
            vec![&mixing, &curated, &thermal, &honesty];
        let report = kerotakis_core::coverage::coverage_manifest(&solvers, v);
        Ok(serde_json::to_string(&report).unwrap())
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

    fn chemistry_applies(&self, vessel: &kerotakis_core::Vessel) -> bool {
        self.aqueous.chemistry_applies(vessel) || self.stack.chemistry_applies(vessel)
    }

    fn equilibrate(
        &mut self,
        vessel: &mut kerotakis_core::Vessel,
    ) -> Result<Vec<Event>, kerotakis_core::SolveError> {
        let mut events = Vec::new();

        // Keep the browser's routing identical to the native stack: physical
        // mixing, curated and thermal chemistry first; aqueous/ice phase
        // coupling next; the honesty pass last. Running the whole
        // Rust stack after the cached aqueous answer let the thermal pass
        // overwrite the heat of precipitation. The next lesson step then
        // described a state that had never existed during pre-warming, so a
        // perfectly curated result looked like a cache miss on device.
        let aqueous_at = self
            .stack
            .solvers
            .iter()
            .position(|solver| solver.name() == "honesty")
            .unwrap_or(self.stack.solvers.len());
        run_solvers(&mut self.stack.solvers[..aqueous_at], vessel, &mut events);
        let mut aqueous = BrowserAqueous {
            inner: &mut *self.aqueous,
        };
        let mut more = kerotakis_core::equilibrate_phase_coupled(&mut aqueous, vessel)?;
        events.append(&mut more);
        run_solvers(&mut self.stack.solvers[aqueous_at..], vessel, &mut events);
        Ok(events)
    }
}

/// Borrow the browser's cache/hook-backed aqueous engine while preserving the
/// same displacement-over-speciation layer used by the native stack.
struct BrowserAqueous<'a> {
    inner: &'a mut kerotakis_phreeqc::PhreeqcEquilibrator,
}

impl Equilibrator for BrowserAqueous<'_> {
    fn name(&self) -> &'static str {
        "phreeqc-aqueous (shipped results)"
    }

    fn applies(&self, vessel: &kerotakis_core::Vessel) -> bool {
        self.inner.applies(vessel)
    }

    fn chemistry_applies(&self, vessel: &kerotakis_core::Vessel) -> bool {
        self.inner.chemistry_applies(vessel)
    }

    fn equilibrate(
        &mut self,
        vessel: &mut kerotakis_core::Vessel,
    ) -> Result<Vec<Event>, kerotakis_core::SolveError> {
        kerotakis_core::displacement::over(self.inner, vessel)
    }
}

fn run_solvers(
    solvers: &mut [Box<dyn Equilibrator>],
    vessel: &mut kerotakis_core::Vessel,
    events: &mut Vec<Event>,
) {
    for solver in solvers {
        if !solver.applies(vessel) {
            continue;
        }
        match solver.equilibrate(vessel) {
            Ok(mut more) => events.append(&mut more),
            Err(error) => events.push(Event::SolverFailed {
                vessel: vessel.id,
                solver: solver.name().to_string(),
                detail: error.to_string(),
            }),
        }
    }
}

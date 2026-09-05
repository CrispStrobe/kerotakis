use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kerotakis_codex::curiosity::{
    load_manifest, ActionFamily, AgeBand, CuriosityPrompt, Disposition,
};
use kerotakis_core::script::{parse_op_typed, ParseErrorKind};
use kerotakis_core::{
    Bench, Event, PermissiveScreen, SolverRoute, SolverRouteKind, SolverRouteOutcome, SolverStack,
};
use serde::{Deserialize, Serialize};

const DEFAULT_MANIFEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/coverage/curiosity-v1/manifest.toml"
);
const DEFAULT_BASELINE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/coverage/curiosity-v1/baseline.toml"
);
const BASELINE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub(crate) struct CuriosityReport {
    schema_version: u32,
    corpus: String,
    smoke_only: bool,
    prompts: Vec<PromptResult>,
    by_observed: BTreeMap<Disposition, usize>,
    by_action: BTreeMap<ActionFamily, BTreeMap<Disposition, usize>>,
    by_material_class: BTreeMap<String, BTreeMap<Disposition, usize>>,
    by_age_band: BTreeMap<AgeBand, BTreeMap<Disposition, usize>>,
    by_owning_task: BTreeMap<String, BTreeMap<Disposition, usize>>,
    expectation_mismatches: usize,
    /// WORLD/coverage: the mismatch count split into the three populations
    /// it conflates. One number cannot be acted on; these three have
    /// different owners and opposite meanings.
    expectation_split: ExpectationSplit,
    failures: Vec<PromptFailure>,
    baseline_drift: Vec<BaselineDrift>,
}

/// "Expectation mismatch" is three different things wearing one label.
///
/// A prompt's `expected` is a REQUIREMENT on the engine: what it must
/// eventually answer, and by which route. Two populations can fail one, and
/// they are not the same event, so they are never summed.
///
/// There used to be a third — `engine_gained`, for a corpus that said
/// `missing` where the engine now answers. It is gone because its cause is
/// gone: `expected` was doing double duty as a prediction, and `missing`
/// only ever made sense in that reading. As a requirement it says the
/// engine must stay silent, which nothing does. `lint` rejects it at load
/// now, so this bucket cannot be reached; what the engine actually does is
/// the baseline's job, and the baseline is drift-gated.
#[derive(Debug, Default, Serialize)]
struct ExpectationSplit {
    /// Corpus claimed an answer; the engine stood aside. Named for what was
    /// OBSERVED, not for a cause: the reason is not established here, and
    /// early evidence says these are not one thing either. A script may
    /// never reach the capability it names (four gas-test prompts test an
    /// open vessel, so the gas has left before the test runs), the
    /// capability may genuinely be absent, or the honest answer may be a
    /// negative that the engine is right to give and wrong to call
    /// `not-yet-modeled`. This is the tail worth WORKING, not a count of
    /// missing features.
    engine_stood_aside: usize,
    /// Both answer, by different routes. Neither is missing; the author
    /// predicted one road and the engine took another.
    route_differs: usize,
}

impl ExpectationSplit {
    /// Record one unmet requirement. Which population it belongs to is
    /// decided by where `Missing` sits: the engine stood aside if the
    /// requirement got nothing, and otherwise both answered by different
    /// roads.
    fn record(&mut self, required: Disposition, observed: Disposition) {
        debug_assert_ne!(
            required,
            Disposition::Missing,
            "`expected` is a requirement and cannot require silence; lint rejects it at load"
        );
        match (required, observed) {
            (_, Disposition::Missing) => self.engine_stood_aside += 1,
            _ => self.route_differs += 1,
        }
    }
}

#[derive(Debug, Serialize)]
struct PromptResult {
    id: String,
    owning_task: String,
    expected: Option<Disposition>,
    observed: Disposition,
    reason_code: String,
    routes: Vec<SolverRoute>,
}

#[derive(Debug, Clone, Serialize)]
struct PromptFailure {
    id: String,
    owning_task: String,
    outcome: BaselineOutcome,
    reason_code: String,
    detail: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BaselineOutcome {
    Computed,
    Curated,
    Qualitative,
    Boundary,
    Missing,
    SolverFailure,
    ExecutionFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BaselineObservation {
    id: String,
    owning_task: String,
    outcome: BaselineOutcome,
    reason_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CuriosityBaseline {
    schema_version: u32,
    corpus: String,
    engine_profile: String,
    observation: Vec<BaselineObservation>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum BaselineDriftKind {
    Added,
    Removed,
    Changed,
}

#[derive(Debug, Clone, Serialize)]
struct BaselineDrift {
    id: String,
    kind: BaselineDriftKind,
    baseline: Option<BaselineObservation>,
    observed: Option<BaselineObservation>,
}

pub(crate) fn command(args: &[String], build_stack: fn() -> SolverStack) {
    if args.first().map(String::as_str) != Some("curiosity") {
        eprintln!(
            "usage: kero coverage curiosity [--json] [--smoke] [--check] \
             [--manifest FILE] [--baseline FILE] [--emit-baseline]"
        );
        std::process::exit(2);
    }
    let json = args.iter().any(|arg| arg == "--json");
    let smoke_only = args.iter().any(|arg| arg == "--smoke");
    let check = args.iter().any(|arg| arg == "--check");
    let emit_baseline = args.iter().any(|arg| arg == "--emit-baseline");
    let manifest = flag_value(args, "--manifest")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MANIFEST));
    let baseline_path = flag_value(args, "--baseline")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BASELINE));
    let mut report = run(&manifest, smoke_only, build_stack()).unwrap_or_else(|error| {
        eprintln!("kero coverage curiosity: {error}");
        std::process::exit(1);
    });

    if emit_baseline {
        if smoke_only {
            eprintln!("kero coverage curiosity: --emit-baseline requires the full corpus");
            std::process::exit(2);
        }
        println!(
            "{}",
            toml::to_string_pretty(&report.baseline()).expect("baseline serializes")
        );
        return;
    }

    match load_baseline(&baseline_path) {
        Ok(baseline) => match compare_baseline(&baseline, &report) {
            Ok(drift) => report.baseline_drift = drift,
            Err(error) => {
                eprintln!("kero coverage curiosity: {error}");
                std::process::exit(1);
            }
        },
        Err(error) if check => {
            eprintln!("kero coverage curiosity: {error}");
            std::process::exit(1);
        }
        Err(_) => {}
    }

    if json {
        println!(
            "{}",
            serde_json::to_string(&report).expect("coverage report serializes")
        );
    } else {
        println!(
            "curiosity {}: {} prompts{}",
            report.corpus,
            report.prompts.len(),
            if smoke_only { " (smoke)" } else { "" }
        );
        for disposition in Disposition::ALL {
            println!(
                "  {:<12} {}",
                disposition_name(disposition),
                report.by_observed[&disposition]
            );
        }
        println!(
            "  expectation mismatches: {}",
            report.expectation_mismatches
        );
        // Split, because the two are not the same event: one is a
        // capability the corpus asserts and the engine does not have, the
        // other is both answering by different roads.
        println!(
            "    engine stood aside (corpus claimed it): {}",
            report.expectation_split.engine_stood_aside
        );
        println!(
            "    route differs (both answer):           {}",
            report.expectation_split.route_differs
        );
        // The stood-aside column is the only one with work in it, so it is
        // the only one worth naming row by row.
        for result in report
            .prompts
            .iter()
            .filter(|r| r.observed == Disposition::Missing && r.expected.is_some())
        {
            println!(
                "      {} [{}] expected {}",
                result.id,
                result.owning_task,
                result.expected.map(disposition_name).unwrap_or("nothing")
            );
        }
        println!("  solver/runtime failures: {}", report.failures.len());
        for failure in &report.failures {
            println!(
                "    {} [{}]: {}",
                failure.id, failure.reason_code, failure.detail
            );
        }
        println!("  baseline drift: {}", report.baseline_drift.len());
        for drift in &report.baseline_drift {
            println!("    {}: {:?}", drift.id, drift.kind);
            // Both rows in full, because the reader of this line is
            // usually a CI log: the box that ran the corpus is not the
            // box the maintainer is sitting at, and a drift line that
            // names only the id sends them off to re-run 500 prompts to
            // learn what changed.
            if let Some(baseline) = &drift.baseline {
                println!(
                    "      baseline: {}",
                    serde_json::to_string(baseline).unwrap_or_default()
                );
            }
            if let Some(observed) = &drift.observed {
                println!(
                    "      observed: {}",
                    serde_json::to_string(observed).unwrap_or_default()
                );
            }
        }
    }
    if check && !report.baseline_drift.is_empty() {
        std::process::exit(1);
    }
}

pub(crate) fn run(
    manifest_path: &Path,
    smoke_only: bool,
    mut stack: SolverStack,
) -> Result<CuriosityReport, String> {
    let corpus = load_manifest(manifest_path)?;
    let problems = corpus.lint();
    if !problems.is_empty() {
        return Err(format!("corpus lint failed:\n{}", problems.join("\n")));
    }

    let mut results = Vec::new();
    let mut by_observed = Disposition::ALL
        .into_iter()
        .map(|disposition| (disposition, 0))
        .collect::<BTreeMap<_, _>>();
    let mut expectation_mismatches = 0;
    let mut expectation_split = ExpectationSplit::default();
    let mut failures = Vec::new();
    let mut by_action = BTreeMap::new();
    let mut by_material_class = BTreeMap::new();
    let mut by_age_band = BTreeMap::new();
    let mut by_owning_task = BTreeMap::new();
    for prompt in corpus
        .prompts
        .iter()
        .filter(|prompt| !smoke_only || corpus.is_smoke(prompt))
    {
        match execute_prompt(prompt, &mut stack) {
            Ok(result) => {
                *by_observed.entry(result.observed).or_default() += 1;
                increment_group(&mut by_action, prompt.action, result.observed);
                increment_group(
                    &mut by_material_class,
                    prompt.material_class.clone(),
                    result.observed,
                );
                increment_group(&mut by_age_band, prompt.age_band, result.observed);
                increment_group(
                    &mut by_owning_task,
                    prompt.owning_task.clone(),
                    result.observed,
                );
                // A prompt that states no requirement cannot fail to meet
                // one. Before `expected` became prescriptive, 202 prompts
                // carried `missing` here and 64 of them counted as
                // mismatches against a requirement nobody had made.
                if let Some(required) = result.expected {
                    if required != result.observed {
                        expectation_mismatches += 1;
                        expectation_split.record(required, result.observed);
                    }
                }
                results.push(result);
            }
            Err(error) => failures.push(error),
        }
    }

    Ok(CuriosityReport {
        schema_version: corpus.manifest.schema_version,
        corpus: corpus.manifest.id,
        smoke_only,
        prompts: results,
        by_observed,
        by_action,
        by_material_class,
        by_age_band,
        by_owning_task,
        expectation_mismatches,
        expectation_split,
        failures,
        baseline_drift: Vec::new(),
    })
}

impl CuriosityReport {
    fn baseline(&self) -> CuriosityBaseline {
        let mut observations = BTreeMap::new();
        for prompt in &self.prompts {
            observations.insert(
                prompt.id.clone(),
                BaselineObservation {
                    id: prompt.id.clone(),
                    owning_task: prompt.owning_task.clone(),
                    outcome: prompt.observed.into(),
                    reason_code: prompt.reason_code.clone(),
                },
            );
        }
        for failure in &self.failures {
            observations.insert(
                failure.id.clone(),
                BaselineObservation {
                    id: failure.id.clone(),
                    owning_task: failure.owning_task.clone(),
                    outcome: failure.outcome,
                    reason_code: failure.reason_code.clone(),
                },
            );
        }
        CuriosityBaseline {
            schema_version: BASELINE_SCHEMA_VERSION,
            corpus: self.corpus.clone(),
            engine_profile: "kerotakis-native-v1".to_string(),
            observation: observations.into_values().collect(),
        }
    }
}

impl From<Disposition> for BaselineOutcome {
    fn from(value: Disposition) -> Self {
        match value {
            Disposition::Computed => Self::Computed,
            Disposition::Curated => Self::Curated,
            Disposition::Qualitative => Self::Qualitative,
            Disposition::Boundary => Self::Boundary,
            Disposition::Missing => Self::Missing,
        }
    }
}

fn load_baseline(path: &Path) -> Result<CuriosityBaseline, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("reading baseline {}: {error}", path.display()))?;
    let baseline: CuriosityBaseline = toml::from_str(&text)
        .map_err(|error| format!("parsing baseline {}: {error}", path.display()))?;
    if baseline.schema_version != BASELINE_SCHEMA_VERSION {
        return Err(format!(
            "baseline schema {} is unsupported (expected {BASELINE_SCHEMA_VERSION})",
            baseline.schema_version
        ));
    }
    if baseline.observation.len() != 500 {
        return Err(format!(
            "baseline has {} observations; expected exactly 500",
            baseline.observation.len()
        ));
    }
    let ids = baseline
        .observation
        .iter()
        .map(|observation| &observation.id)
        .collect::<std::collections::BTreeSet<_>>();
    if ids.len() != baseline.observation.len() {
        return Err("baseline contains duplicate prompt ids".to_string());
    }
    Ok(baseline)
}

fn compare_baseline(
    baseline: &CuriosityBaseline,
    report: &CuriosityReport,
) -> Result<Vec<BaselineDrift>, String> {
    if baseline.corpus != report.corpus {
        return Err(format!(
            "baseline corpus '{}' does not match report '{}'",
            baseline.corpus, report.corpus
        ));
    }
    let baseline_by_id = baseline
        .observation
        .iter()
        .map(|observation| (observation.id.as_str(), observation))
        .collect::<BTreeMap<_, _>>();
    let observed = report.baseline();
    let observed_by_id = observed
        .observation
        .iter()
        .map(|observation| (observation.id.as_str(), observation))
        .collect::<BTreeMap<_, _>>();
    let mut drift = Vec::new();
    for (id, observation) in &observed_by_id {
        match baseline_by_id.get(id) {
            None => drift.push(BaselineDrift {
                id: (*id).to_string(),
                kind: BaselineDriftKind::Added,
                baseline: None,
                observed: Some((*observation).clone()),
            }),
            Some(expected) if *expected != *observation => drift.push(BaselineDrift {
                id: (*id).to_string(),
                kind: BaselineDriftKind::Changed,
                baseline: Some((*expected).clone()),
                observed: Some((*observation).clone()),
            }),
            Some(_) => {}
        }
    }
    if !report.smoke_only {
        for (id, observation) in baseline_by_id {
            if !observed_by_id.contains_key(id) {
                drift.push(BaselineDrift {
                    id: id.to_string(),
                    kind: BaselineDriftKind::Removed,
                    baseline: Some(observation.clone()),
                    observed: None,
                });
            }
        }
    }
    drift.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(drift)
}

fn increment_group<K: Ord>(
    groups: &mut BTreeMap<K, BTreeMap<Disposition, usize>>,
    key: K,
    disposition: Disposition,
) {
    *groups
        .entry(key)
        .or_insert_with(|| Disposition::ALL.into_iter().map(|kind| (kind, 0)).collect())
        .entry(disposition)
        .or_default() += 1;
}

fn execute_prompt(
    prompt: &CuriosityPrompt,
    stack: &mut SolverStack,
) -> Result<PromptResult, PromptFailure> {
    if prompt.expected == Some(Disposition::Boundary) {
        return Ok(result(
            prompt,
            Disposition::Boundary,
            prompt.boundary.as_deref().unwrap_or("declared-boundary"),
            Vec::new(),
        ));
    }

    let mut bench = Bench::new();
    let mut events = Vec::new();
    let mut all_events = Vec::new();
    let mut routes = Vec::new();
    for (index, line) in prompt.script.iter().enumerate() {
        let op = match parse_op_typed(line) {
            Ok(Some(op)) => op,
            Ok(None) => {
                return Err(execution_failure(
                    prompt,
                    "session-command",
                    format!("line {} is not an operator", index + 1),
                ));
            }
            Err(error) => {
                return Ok(result(
                    prompt,
                    Disposition::Missing,
                    parse_reason(error.kind),
                    routes,
                ));
            }
        };
        // `last_routes` is cleared by `SolverStack::equilibrate`, so a step
        // that never equilibrates — `new`, and any other operator that only
        // touches bookkeeping — leaves the PREVIOUS step's routes standing.
        // Across prompts that is the previous PROMPT's routes, and this loop
        // then attributes them to this one. `aq-091` was classified `curated`
        // in a smoke run and `computed` in a full one, same script, because
        // the prompt that happened to run before it differed; the route it
        // was judged on belonged to its neighbour. Clearing here makes a
        // prompt's routes its own.
        stack.last_routes.clear();
        let step_events = bench
            .step_with(op, stack, &PermissiveScreen)
            .map_err(|error| {
                execution_failure(
                    prompt,
                    "bench-error",
                    format!("line {}: {error}", index + 1),
                )
            })?;
        // A prompt describes the state reached by the whole script. Earlier
        // additions may honestly be uncovered until a later command supplies
        // the solvent/reactant that makes the final state routable.
        all_events.extend(step_events.iter().cloned());
        events = step_events;
        routes.extend(stack.last_routes.iter().cloned());
    }

    if let Some(Event::SolverFailed { solver, detail, .. }) = all_events
        .iter()
        .find(|event| matches!(event, Event::SolverFailed { .. }))
    {
        return Err(PromptFailure {
            id: prompt.id.clone(),
            owning_task: prompt.owning_task.clone(),
            outcome: BaselineOutcome::SolverFailure,
            reason_code: format!("solver-failed-{solver}"),
            detail: detail.clone(),
        });
    }
    if all_events
        .iter()
        .any(|event| matches!(event, Event::SafetyVeto { .. }))
    {
        return Ok(result(prompt, Disposition::Boundary, "safety-veto", routes));
    }
    if events
        .iter()
        .any(|event| matches!(event, Event::NotYetModeled { .. }))
        && !all_events.iter().any(|event| {
            matches!(
                event,
                Event::Smelled { .. }
                    | Event::GasTested { .. }
                    | Event::FlameTest { .. }
                    | Event::DidNotIgnite { .. }
                    // KID-12: a smothered flame is an answer with a
                    // number in it, not a gap in the model.
                    | Event::FlameStarved { .. }
                    // BRD-023: so is a corrosion verdict. It is listed
                    // HERE and deliberately not in `typed_observation`
                    // below: it should stop a prompt being called
                    // `missing` when the corrosion route answered it, and
                    // it should not outrank a computed or curated route
                    // that was the real answer, because a corrosion
                    // verdict beside those is an aside about a spectator
                    // metal.
                    | Event::Corroded { .. }
                    // BRD-023: and so is a verdict about what heat has
                    // done to a plastic. Same placement and same reason
                    // as the corrosion verdict directly above: it should
                    // stop a prompt being called `missing` when the
                    // polymer route answered it, and it should not
                    // outrank a computed or curated route that was the
                    // real answer, so it is deliberately absent from
                    // `typed_observation` below.
                    | Event::PolymerHeated { .. }
                    // BRD-032: so is an adsorption split. Listed HERE
                    // and deliberately not in `typed_observation`
                    // below, for the same reason the corrosion verdict
                    // is: it should stop a prompt being called
                    // `missing` when the isotherm answered it, and it
                    // should not outrank a computed route that was the
                    // real answer.
                    | Event::Adsorbed { .. }
                    // BRD-014: and so is what a sealed cell says about
                    // its own insides. Same placement, same reason.
                    | Event::SealedCell { .. }
                    // BRD-041: "warm, with oxygen, and not burning" is an
                    // answer about a fuel, in the same class as `Inert`.
                    | Event::BelowAutoignition { .. }
                    // BRD-014.S05: a transmitted fraction is a computed
                    // answer about a material, in the same class.
                    | Event::UvAttenuated { .. }
                    // CAP-25: a seal that failed at a stated pressure is
                    // the answer to "can a sealed vessel burst?", with a
                    // number in it — not a gap beside the safety line.
                    | Event::Burst { .. }
                    | Event::Inert { .. }
                    | Event::InertInSolvent { .. }
            )
        })
    {
        return Ok(result(
            prompt,
            Disposition::Missing,
            "not-yet-modeled",
            routes,
        ));
    }
    let succeeded = |kind| {
        routes.iter().any(|route| {
            route.kind == kind
                && matches!(
                    route.outcome,
                    SolverRouteOutcome::Succeeded { event_count } if event_count > 0
                )
        })
    };
    let computed_chemistry = routes.iter().any(|route| {
        route.kind == SolverRouteKind::Computed
            && route.chemistry
            && matches!(route.outcome, SolverRouteOutcome::Succeeded { .. })
    });

    let typed_observation = all_events.iter().any(|event| {
        matches!(
            event,
            Event::Smelled { .. }
                | Event::GasTested { .. }
                | Event::FlameTest { .. }
                | Event::DidNotIgnite { .. }
                | Event::Inert { .. }
                | Event::InertInSolvent { .. }
        )
    })
        // KID-12: a flame that never caught is a typed observation. One
        // that burned first and then ran out of air is a computed
        // result, and must not be demoted past the computed branch
        // below — this check is on `burned`, not on the event.
        || all_events.iter().any(|event| {
            matches!(event, Event::FlameStarved { burned, .. } if burned.0 <= 0.0)
        });

    // A typed observation BESIDE a computed result does not make the row a
    // typed observation. That is KID-12's rule above, generalised from the
    // single event it was written for: a beaker that plates 0.0099 mol of
    // copper onto iron and also says, truly, that the iron itself is
    // kinetically blocked has computed something. The aside is extra, not
    // instead.
    //
    // It arrived as a REGRESSION FROM AN IMPROVEMENT, which is what makes
    // it worth fixing rather than tolerating. K40 added three copper
    // hydroxy-sulfate phases; precipitating them releases protons; the
    // solution is a little more acid; and that is enough for the
    // displacement model to add one true sentence about the overpotential
    // on iron. Same copper plated, better pH, one more honest sentence,
    // and `aq-123`, `mat-057` and `th-082` fell from computed to
    // qualitative. The only way to have kept the score would have been not
    // to model the phases. (Found by kerotakis-5f, who declined to change
    // the ordering in a commit that added species — correctly, and this is
    // that change made separately.)
    //
    // Note what this does NOT do. It does not decide whether the PROMPT's
    // question was answered; that needs the question, which lives in the
    // prompt and not in the engine, and guessing it from event kinds is
    // exactly what #362 got wrong. It decides which ROUTE produced the
    // answer, which is a fact about the engine that `routes` states
    // directly.
    //
    // Two events, and the second was found by the first not being enough.
    // `mat-057` asks which metal pair gives the largest cell voltage and
    // never plates anything — its answer is a `CellVoltage`, beside the
    // same kind of true aside about a metal. Fixing `Plated` alone moved
    // two of the three rows, and the third was the reason to look rather
    // than to declare the job done.
    //
    // And it is deliberately NARROW. The first attempt guarded the whole
    // branch with "a computed route succeeded", which moved fifteen rows
    // and raised the mismatch count — because for a smell or a gas test
    // the typed observation IS the answer, even in a beaker that also ran
    // an aqueous solve. Only the metal-plating case has an event that is
    // unambiguously the result beside an aside that is unambiguously not,
    // so only that case is claimed. The measurement said the guard was
    // wider than the evidence, which is the whole reason to measure.
    let plated_beside_an_aside = all_events.iter().any(|event| {
        matches!(
            event,
            Event::Plated { .. }
                | Event::CellVoltage { .. }
                | Event::AcidMetalCellVoltage { .. }
                // Electrolytic plating is plating. `mat-064` asks why
                // copper plates onto one electrode and the bench plates
                // 0.0047 mol of it, but the solvent-electrolysis path
                // reports that as `Electrolysed` rather than `Plated`.
                | Event::Electrolysed { .. }
        )
    }) && all_events.iter().all(|event| {
        !matches!(
            event,
            Event::Smelled { .. }
                | Event::GasTested { .. }
                | Event::FlameTest { .. }
                | Event::DidNotIgnite { .. }
                | Event::FlameStarved { .. }
        )
    });
    // The same "aside, not instead" rule, for the one other event that
    // is a remark rather than a result. An `Inert` fires at the step that
    // adds the substance, when it is TRUE: starch really does not dissolve
    // in cold water. The reagent arrives on a later line, and by the end of
    // the run a curated reaction has digested it. The engine cannot see the
    // future at the step it speaks, so nothing it says there is wrong — but
    // the row is graded on the whole transcript, where a curated reaction
    // ran and the remark about the starting material is no longer the
    // result.
    //
    // Narrow on purpose, in the same way and for the same reason as the
    // guard above: only a SUCCEEDED CURATED route overrides it. A computed
    // route is not enough — `bio-042` (starch + HCl + heat) and `mat-029`
    // (PET + NaOH + heat) have no curated hydrolysis, so their computed
    // route is the acid or the base speciating, not the polymer doing
    // anything, and there "the polymer is unchanged" IS the answer. Those
    // two stay qualitative, which is what they are.
    // The same "aside, not instead" rule again, and this time the aside
    // and the answer are about the SAME solid. A gram of activated
    // charcoal in dye solution truly does not dissolve, and the honesty
    // pass says so; then the isotherm computes how much of the dye it
    // holds. The remark is about the carbon's solubility and the answer
    // is about the dye's, so the first must not be allowed to file the
    // row as a typed observation and hide the second. Narrow on purpose:
    // an `Adsorbed` event is the only thing that lifts it, exactly as a
    // succeeded curated route is the only thing that lifts the clause
    // above.
    let inert_beside_curated = all_events
        .iter()
        .any(|event| matches!(event, Event::Inert { .. } | Event::InertInSolvent { .. }))
        && (succeeded(SolverRouteKind::Curated)
            || all_events
                .iter()
                .any(|event| matches!(event, Event::Adsorbed { .. })));
    if typed_observation && !plated_beside_an_aside && !inert_beside_curated {
        return Ok(result(
            prompt,
            Disposition::Qualitative,
            "typed-observation",
            routes,
        ));
    }
    if prompt.action == ActionFamily::HandleAndInspect
        && all_events
            .iter()
            .any(|event| matches!(event, Event::Observed { .. } | Event::Measured { .. }))
    {
        return Ok(result(
            prompt,
            Disposition::Qualitative,
            "typed-observation",
            routes,
        ));
    }

    let (observed, reason) = if succeeded(SolverRouteKind::Curated) {
        (Disposition::Curated, "curated-route")
    } else if computed_chemistry || succeeded(SolverRouteKind::Computed) {
        (Disposition::Computed, "computed-route")
    } else if succeeded(SolverRouteKind::Qualitative) {
        (Disposition::Qualitative, "qualitative-route")
    } else if !all_events.is_empty() {
        (Disposition::Computed, "typed-engine-event")
    } else {
        (Disposition::Missing, "no-applicable-model")
    };
    Ok(result(prompt, observed, reason, routes))
}

fn result(
    prompt: &CuriosityPrompt,
    observed: Disposition,
    reason_code: &str,
    routes: Vec<SolverRoute>,
) -> PromptResult {
    PromptResult {
        id: prompt.id.clone(),
        owning_task: prompt.owning_task.clone(),
        expected: prompt.expected,
        observed,
        reason_code: reason_code.to_string(),
        routes,
    }
}

fn execution_failure(prompt: &CuriosityPrompt, reason_code: &str, detail: String) -> PromptFailure {
    PromptFailure {
        id: prompt.id.clone(),
        owning_task: prompt.owning_task.clone(),
        outcome: BaselineOutcome::ExecutionFailure,
        reason_code: reason_code.to_string(),
        detail,
    }
}

fn parse_reason(kind: ParseErrorKind) -> &'static str {
    match kind {
        ParseErrorKind::UnknownSpecies => "unknown-species",
        ParseErrorKind::UnknownReaction => "unknown-reaction",
        ParseErrorKind::InvalidSyntax => "invalid-syntax",
    }
}

fn disposition_name(disposition: Disposition) -> &'static str {
    match disposition {
        Disposition::Computed => "computed",
        Disposition::Curated => "curated",
        Disposition::Qualitative => "qualitative",
        Disposition::Boundary => "boundary",
        Disposition::Missing => "missing",
    }
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_unmet_requirement_is_sorted_by_where_missing_sits() {
        use Disposition::*;
        let mut split = ExpectationSplit::default();

        // The corpus required an answer and the engine stands aside. This
        // is the tail worth working.
        split.record(Computed, Missing);
        split.record(Curated, Missing);

        // Both answered, by different roads.
        split.record(Computed, Qualitative);

        assert_eq!(split.engine_stood_aside, 2);
        assert_eq!(split.route_differs, 1);
    }

    use super::*;

    fn observation(id: &str, outcome: BaselineOutcome) -> BaselineObservation {
        BaselineObservation {
            id: id.to_string(),
            owning_task: "BRD-test".to_string(),
            outcome,
            reason_code: "reason".to_string(),
        }
    }

    fn report(smoke_only: bool, observations: &[BaselineObservation]) -> CuriosityReport {
        let prompts = observations
            .iter()
            .map(|observation| PromptResult {
                id: observation.id.clone(),
                owning_task: observation.owning_task.clone(),
                // These fixtures exercise baseline drift, which does not
                // consult `expected` at all — the two records are
                // independent, which is the whole point of the split.
                expected: None,
                observed: match observation.outcome {
                    BaselineOutcome::Computed => Disposition::Computed,
                    BaselineOutcome::Curated => Disposition::Curated,
                    BaselineOutcome::Qualitative => Disposition::Qualitative,
                    BaselineOutcome::Boundary => Disposition::Boundary,
                    BaselineOutcome::Missing => Disposition::Missing,
                    BaselineOutcome::SolverFailure | BaselineOutcome::ExecutionFailure => {
                        panic!("test helper expects a disposition")
                    }
                },
                reason_code: observation.reason_code.clone(),
                routes: Vec::new(),
            })
            .collect();
        CuriosityReport {
            schema_version: 1,
            corpus: "curiosity-v1".to_string(),
            smoke_only,
            prompts,
            by_observed: BTreeMap::new(),
            by_action: BTreeMap::new(),
            by_material_class: BTreeMap::new(),
            by_age_band: BTreeMap::new(),
            by_owning_task: BTreeMap::new(),
            expectation_mismatches: 0,
            expectation_split: ExpectationSplit::default(),
            failures: Vec::new(),
            baseline_drift: Vec::new(),
        }
    }

    #[test]
    fn baseline_reports_changed_added_and_removed_prompts() {
        let baseline = CuriosityBaseline {
            schema_version: 1,
            corpus: "curiosity-v1".to_string(),
            engine_profile: "test".to_string(),
            observation: vec![
                observation("changed", BaselineOutcome::Computed),
                observation("removed", BaselineOutcome::Missing),
            ],
        };
        let current = report(
            false,
            &[
                observation("changed", BaselineOutcome::Boundary),
                observation("added", BaselineOutcome::Curated),
            ],
        );
        let drift = compare_baseline(&baseline, &current).expect("compare baseline");
        assert_eq!(drift.len(), 3);
        assert!(matches!(drift[0].kind, BaselineDriftKind::Added));
        assert!(matches!(drift[1].kind, BaselineDriftKind::Changed));
        assert!(matches!(drift[2].kind, BaselineDriftKind::Removed));
    }

    #[test]
    fn smoke_comparison_ignores_baseline_entries_outside_the_subset() {
        let baseline = CuriosityBaseline {
            schema_version: 1,
            corpus: "curiosity-v1".to_string(),
            engine_profile: "test".to_string(),
            observation: vec![
                observation("smoke", BaselineOutcome::Computed),
                observation("full-only", BaselineOutcome::Missing),
            ],
        };
        let current = report(true, &[observation("smoke", BaselineOutcome::Computed)]);
        assert!(compare_baseline(&baseline, &current)
            .expect("compare smoke subset")
            .is_empty());
    }
}

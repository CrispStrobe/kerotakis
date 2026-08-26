use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use kerotakis_codex::curiosity::{
    load_manifest, ActionFamily, AgeBand, CuriosityPrompt, Disposition,
};
use kerotakis_core::script::{parse_op_typed, ParseErrorKind};
use kerotakis_core::{
    Bench, Event, PermissiveScreen, SolverRoute, SolverRouteKind, SolverRouteOutcome, SolverStack,
};
use serde::Serialize;

const DEFAULT_MANIFEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/coverage/curiosity-v1/manifest.toml"
);

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
    failures: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PromptResult {
    id: String,
    expected: Disposition,
    observed: Disposition,
    reason_code: String,
    routes: Vec<SolverRoute>,
}

pub(crate) fn command(args: &[String], build_stack: fn() -> SolverStack) {
    if args.first().map(String::as_str) != Some("curiosity") {
        eprintln!("usage: kero coverage curiosity [--json] [--smoke] [--check] [--manifest FILE]");
        std::process::exit(2);
    }
    let json = args.iter().any(|arg| arg == "--json");
    let smoke_only = args.iter().any(|arg| arg == "--smoke");
    let check = args.iter().any(|arg| arg == "--check");
    let manifest = flag_value(args, "--manifest")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MANIFEST));
    let report = run(&manifest, smoke_only, build_stack()).unwrap_or_else(|error| {
        eprintln!("kero coverage curiosity: {error}");
        std::process::exit(1);
    });

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
        println!("  solver/runtime failures: {}", report.failures.len());
        for failure in &report.failures {
            println!("    {failure}");
        }
    }
    if check && (!report.failures.is_empty() || report.expectation_mismatches > 0) {
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
                expectation_mismatches += usize::from(result.expected != result.observed);
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
        failures,
    })
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
) -> Result<PromptResult, String> {
    if prompt.expected == Disposition::Boundary {
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
            Ok(None) => return Err(format!("{}:{} is not an operator", prompt.id, index + 1)),
            Err(error) => {
                return Ok(result(
                    prompt,
                    Disposition::Missing,
                    parse_reason(error.kind),
                    routes,
                ));
            }
        };
        let step_events = bench
            .step_with(op, stack, &PermissiveScreen)
            .map_err(|error| format!("{}:{}: {error}", prompt.id, index + 1))?;
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
        return Err(format!("{}: solver {solver} failed: {detail}", prompt.id));
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
    if all_events.iter().any(|event| {
        matches!(
            event,
            Event::Smelled { .. }
                | Event::GasTested { .. }
                | Event::FlameTest { .. }
                | Event::DidNotIgnite { .. }
                | Event::Inert { .. }
                | Event::InertInSolvent { .. }
        )
    }) {
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
        expected: prompt.expected,
        observed,
        reason_code: reason_code.to_string(),
        routes,
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

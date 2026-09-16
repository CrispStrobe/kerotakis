//! **Perturbation cases generated from the curiosity corpus's own scripts.**
//!
//! `tests/coverage/curiosity-v1/*.toml` holds five hundred authored prompts.
//! Each carries a `question`, a `script` and — for 307 of them — an
//! `expected`. The `expected` vocabulary is
//! `computed | curated | qualitative | boundary`: four ROUTES, not four
//! outcomes. What the corpus asserts is therefore that the engine answered
//! by the route somebody intended, and nothing whatever about the
//! chemistry.
//!
//! `aq-003` is the illustration. The question asks whether a beaker
//! *cools*; the script measures a thermometer; nothing anywhere records
//! that the temperature should fall. The engine could report forty degrees
//! of warming and that row stays green for as long as the corpus exists.
//!
//! **What survives is the scripts.** Each is a real experiment with
//! reagents, quantities and an instrument, and that is the expensive half
//! of a test case. This file spends it, by asking the one question that
//! needs no reference value, no licence and no external data:
//!
//! > does this number move, in the right direction, when its cause moves?
//!
//! It is the mechanisation of `perturbation.rs`, whose seven hand-written
//! cases are the model, over `metamorphic.rs`'s relations, applied to
//! scripts nobody here wrote.
//!
//! # How a perturbation is chosen
//!
//! Each script is parsed into steps and offered to five rules in a fixed
//! order. A rule is admissible when the script's *shape* supports it; the
//! census below records, for every one of the five hundred, which rules it
//! admits and — when it admits none — why.
//!
//! | rule | perturbation | claim | shape |
//! |---|---|---|---|
//! | `Order` | reverse a contiguous run of adds | every final quantity is unchanged | invariance |
//! | `Scale` | double every quantity in the script | intensive unchanged, extensive doubles | invariance + direction |
//! | `Solvent` | double the solvent only | amounts unchanged, the solution dilutes | invariance + direction |
//! | `Ablation` | delete one reagent | the observation must move | causality |
//! | `Dose` | multiply one reagent | the observation must move monotonically | direction |
//! | `Sibling` | run an authored dose-sibling | two loadings, two responses, ordered | **differential** |
//!
//! The order is not arbitrary. It runs from the sharpest shape to the
//! bluntest, in exactly the sense the hand-written suite argues for:
//!
//! * An **invariance** survives an engine improvement. `Order` and `Scale`
//!   assert that two runs agree, so no number in them has to be re-blessed
//!   when the solver gets better, and neither can be satisfied by a
//!   constant.
//! * A **causal** claim (`Ablation`) is the direct cure for the disease
//!   this corpus has: it asserts that the answer depends on the reagent the
//!   question is about. `aq-003` cannot pass it while reporting a
//!   temperature that ignores the potassium chloride.
//! * A **directional** claim (`Dose`) is the weakest, because a quantity
//!   the engine simply multiplies through always moves. It is generated,
//!   but it is generated last and it is reported as weak.
//! * A **differential** claim (`Sibling`) is the strongest, and the corpus
//!   turns out to contain twenty-four groups of it already: scripts
//!   identical but for one quantity, authored independently. A global error
//!   cannot fake a ratio between two responses.
//!
//! # What this file deliberately does not touch
//!
//! `contents["OH-"]` is the solution's residual cation charge rather than
//! hydroxide — recorded, unfixed, and wrong by six orders of magnitude in a
//! buffered solution (`perturbation.rs`, case 7/7). Two generated rules
//! would otherwise have read it: `Ablation`'s observation vector and
//! `Order`'s per-species comparison both walk `contents`. `Order` keeps it,
//! because an invariance over a wrong number is still a true statement
//! about path independence; `Ablation` and `Dose` exclude it by name, so
//! that no generated case can ever be *satisfied* by that defect moving.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use kerotakis_codex::curiosity::{load_manifest, CuriosityPrompt};

static CASE: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn corpus() -> Vec<CuriosityPrompt> {
    load_manifest(&repo_root().join("tests/coverage/curiosity-v1/manifest.toml"))
        .expect("the corpus loads")
        .prompts
}

// ===================================================================
// The script model.
// ===================================================================

#[derive(Clone, Debug, PartialEq)]
struct Add {
    vessel: String,
    species: String,
    quantity: f64,
    unit: String,
    temperature: Option<String>,
}

impl Add {
    fn render(&self) -> String {
        let mut line = format!(
            "add {} {} {}{}",
            self.vessel,
            self.species,
            trim(self.quantity),
            self.unit
        );
        if let Some(temperature) = &self.temperature {
            line.push_str(&format!(" @ {temperature}C"));
        }
        line
    }
}

/// Render a quantity without an exponent or a trailing `.0`, because the
/// bench parser reads `5` and `5.5` but is not obliged to read `5e-3`.
fn trim(value: f64) -> String {
    let mut text = format!("{value:.10}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

#[derive(Clone, Debug, PartialEq)]
enum Step {
    Add(Add),
    Other { verb: String, line: String },
}

impl Step {
    fn render(&self) -> String {
        match self {
            Step::Add(add) => add.render(),
            Step::Other { line, .. } => line.clone(),
        }
    }
    fn verb(&self) -> &str {
        match self {
            Step::Add(_) => "add",
            Step::Other { verb, .. } => verb,
        }
    }
    fn as_add(&self) -> Option<&Add> {
        match self {
            Step::Add(add) => Some(add),
            Step::Other { .. } => None,
        }
    }
}

/// `add <vessel> <species> <quantity><unit> [@ <T>C]`, and everything else
/// verbatim. Returns `None` for an `add` this parser does not understand,
/// which is how a corpus edit that outgrows the generator announces itself
/// rather than being silently reclassified as "admits nothing".
fn parse(script: &[String]) -> Option<Vec<Step>> {
    let mut steps = Vec::new();
    for line in script {
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some(verb) = words.first() else {
            return None;
        };
        if *verb != "add" {
            steps.push(Step::Other {
                verb: (*verb).to_string(),
                line: line.clone(),
            });
            continue;
        }
        if words.len() != 4 && !(words.len() == 6 && words[4] == "@") {
            return None;
        }
        let amount = words[3];
        let split = amount.find(|c: char| c.is_alphabetic())?;
        let (quantity, unit) = amount.split_at(split);
        if !matches!(unit, "mol" | "g" | "mL" | "L") {
            return None;
        }
        let temperature = if words.len() == 6 {
            Some(words[5].strip_suffix('C')?.to_string())
        } else {
            None
        };
        steps.push(Step::Add(Add {
            vessel: words[1].to_string(),
            species: words[2].to_string(),
            quantity: quantity.parse().ok()?,
            unit: unit.to_string(),
            temperature,
        }));
    }
    Some(steps)
}

fn render(steps: &[Step]) -> String {
    let mut text = steps
        .iter()
        .map(Step::render)
        .collect::<Vec<_>>()
        .join("\n");
    text.push('\n');
    text
}

/// Poured as a medium rather than as the thing under study. Not a claim
/// about chemistry — a claim about what the ABLATION rule may delete, and
/// about which line the SOLVENT rule doubles.
const SOLVENTS: &[&str] = &["water", "ethanol", "hexane", "propanone", "vegetable_oil"];

fn is_solvent(species: &str) -> bool {
    SOLVENTS.contains(&species)
}

/// Verbs that fix an absolute size, so doubling every quantity around them
/// is not the same experiment twice as big. `ignite` holds a flame of a
/// size the script does not state; `irradiate` is a flux over an unstated
/// area; `titrate` searches in discrete steps; `transport` and `cell` have
/// a geometry.
const SIZE_BOUND: &[&str] = &[
    "ignite",
    "irradiate",
    "transport",
    "titrate",
    "cell",
    "chromatograph",
    "particles",
    "magnet",
    "sweep",
    "regulate",
    "electrolyse",
];

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Rule {
    Order,
    Scale,
    Solvent,
    Ablation,
    Dose,
}

impl Rule {
    fn name(self) -> &'static str {
        match self {
            Rule::Order => "order",
            Rule::Scale => "scale",
            Rule::Solvent => "solvent",
            Rule::Ablation => "ablation",
            Rule::Dose => "dose",
        }
    }
}

/// The longest contiguous run of `add` steps addressed to one vessel at one
/// temperature. Returns the half-open range into `steps`.
fn interchangeable_run(steps: &[Step]) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut start = None;
    for index in 0..=steps.len() {
        let extends = index < steps.len()
            && match (steps[index].as_add(), start.map(|s| steps[s].as_add())) {
                (Some(_), None) => true,
                (Some(add), Some(Some(first))) => {
                    add.vessel == first.vessel && add.temperature == first.temperature
                }
                _ => false,
            };
        match (extends, start) {
            (true, None) => start = Some(index),
            (true, Some(_)) => {}
            (false, Some(from)) => {
                let span = (from, index);
                if best.is_none_or(|b| span.1 - span.0 > b.1 - b.0) {
                    best = Some(span);
                }
                start = if steps.get(index).and_then(Step::as_add).is_some() {
                    Some(index)
                } else {
                    None
                };
            }
            (false, None) => {}
        }
    }
    best.filter(|(from, to)| to - from >= 2)
}

/// Index of the reagent an ablation deletes: the LAST non-solvent add, which
/// is overwhelmingly the thing the question is about. Corpus scripts read
/// "set the scene, then do the interesting thing".
fn ablation_target(steps: &[Step]) -> Option<usize> {
    steps
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            step.as_add()
                .is_some_and(|add| !is_solvent(&add.species))
        })
        .map(|(index, _)| index)
        .next_back()
}

/// Which rules this script's shape admits, and — when none — why not.
fn admits(steps: &[Step]) -> (Vec<Rule>, Option<&'static str>) {
    let adds: Vec<&Add> = steps.iter().filter_map(Step::as_add).collect();
    if adds.is_empty() {
        return (
            Vec::new(),
            Some("no reagent: the script only observes or manipulates"),
        );
    }
    let reagents = adds.iter().filter(|add| !is_solvent(&add.species)).count();
    let solvents = adds.len() - reagents;

    let mut rules = Vec::new();
    if interchangeable_run(steps).is_some() {
        rules.push(Rule::Order);
    }
    if !steps.iter().any(|step| SIZE_BOUND.contains(&step.verb())) {
        rules.push(Rule::Scale);
    }
    if solvents > 0 && reagents > 0 {
        rules.push(Rule::Solvent);
    }
    if reagents > 0 && adds.len() >= 2 {
        rules.push(Rule::Ablation);
    }
    if reagents > 0 {
        rules.push(Rule::Dose);
    }
    if rules.is_empty() {
        return (
            rules,
            Some("a single reagent with no solvent and a size-bound verb: \
                  nothing can be held fixed against the perturbation"),
        );
    }
    (rules, None)
}

/// The perturbed script a rule produces, or `None` when the rule does not
/// apply to this shape.
fn perturb(rule: Rule, steps: &[Step]) -> Option<String> {
    let mut perturbed = steps.to_vec();
    match rule {
        Rule::Order => {
            let (from, to) = interchangeable_run(steps)?;
            perturbed[from..to].reverse();
        }
        Rule::Scale => {
            for step in &mut perturbed {
                if let Step::Add(add) = step {
                    add.quantity *= 2.0;
                } else if let Step::Other { verb, line } = step {
                    // Energy and headspace are quantities too. A vessel
                    // twice the size warmed by the same joules is a
                    // DIFFERENT experiment, not the same one scaled.
                    if matches!(verb.as_str(), "heat" | "cool" | "seal" | "dilute") {
                        *line = double_last_quantity(line)?;
                    }
                }
            }
        }
        Rule::Solvent => {
            let mut touched = false;
            for step in &mut perturbed {
                if let Step::Add(add) = step {
                    if is_solvent(&add.species) {
                        add.quantity *= 2.0;
                        touched = true;
                    }
                }
            }
            if !touched {
                return None;
            }
        }
        Rule::Ablation => {
            perturbed.remove(ablation_target(steps)?);
        }
        Rule::Dose => {
            let target = ablation_target(steps)?;
            if let Step::Add(add) = &mut perturbed[target] {
                add.quantity *= 2.0;
            }
        }
    }
    let perturbed = render(&perturbed);
    (perturbed != render(steps)).then_some(perturbed)
}

/// `heat v1 20kJ` -> `heat v1 40kJ`. The quantity is the last word with a
/// leading number.
fn double_last_quantity(line: &str) -> Option<String> {
    let mut words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
    let last = words.last_mut()?;
    let split = last
        .find(|c: char| c.is_alphabetic())
        .unwrap_or(last.len());
    let (quantity, unit) = last.split_at(split);
    let doubled: f64 = quantity.parse::<f64>().ok()? * 2.0;
    *last = format!("{}{unit}", trim(doubled));
    Some(words.join(" "))
}

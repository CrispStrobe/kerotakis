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

// ===================================================================
// The observation surface: what a perturbed run is compared on.
// ===================================================================

/// Run a script through `kero run --json`. `Err` carries stderr, because a
/// script the generator mangled and a script the engine cannot run are
/// different findings and the census has to tell them apart.
fn run(script: &str) -> Result<Vec<serde_json::Value>, String> {
    let dir = std::env::temp_dir().join(format!(
        "kero-corpus-perturb-{}-{}",
        std::process::id(),
        CASE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let lab = dir.join("case.lab");
    std::fs::write(&lab, script).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_kero"))
        .args(["run", lab.to_str().unwrap(), "--json"])
        .output()
        .expect("kero runs");
    std::fs::remove_dir_all(&dir).ok();
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).map_err(|error| error.to_string()))
        .collect()
}

/// The `contents` slot the aqueous tail fills with the solution's residual
/// cation charge under hydroxide's name. Measured 1.1e6 high in an acetate
/// buffer (`perturbation.rs`, 7/7); recorded and unfixed. A generated case
/// must never be *satisfied* by this number moving, so the rules whose
/// claim is "something moved" refuse to look at it.
const NOT_WHAT_IT_SAYS: &[&str] = &["OH-"];

/// A flattened, comparable picture of the whole bench after the last step:
/// every vessel's intensive properties, every species' amount, and the
/// words the scene shows a person. Keys are stable strings so two runs
/// that disagree about which species EXIST are comparable — an appearing
/// or vanishing species is a difference, not a panic.
fn observe(steps: &[serde_json::Value]) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let Some(last) = steps.last() else {
        return out;
    };
    let vessels = last["bench"]["vessels"].as_array().cloned().unwrap_or_default();
    for (index, vessel) in vessels.iter().enumerate() {
        let mut put = |field: &str, value: Option<f64>| {
            if let Some(value) = value {
                out.insert(format!("v{index}.{field}"), value);
            }
        };
        put("ph", vessel["solution"]["ph"].as_f64());
        put("ionic_strength", vessel["solution"]["ionic_strength"].as_f64());
        put("pe", vessel["solution"]["pe"].as_f64());
        put("temperature_k", vessel["temperature_k"].as_f64());
        put("volume_l", vessel["volume_l"].as_f64());
        put("free_hydroxide", vessel["free_hydroxide"].as_f64());
        put(
            "co2_partial_pressure_atm",
            vessel["co2_partial_pressure_atm"].as_f64(),
        );
        for portion in vessel["contents"].as_array().cloned().unwrap_or_default() {
            let (Some(species), Some(moles)) =
                (portion["species"].as_str(), portion["moles"].as_f64())
            else {
                continue;
            };
            let phase = portion["phase"].as_str().unwrap_or("");
            *out.entry(format!("v{index}.n[{species}|{phase}]"))
                .or_insert(0.0) += moles;
        }
        for reported in vessel["solution"]["species"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            let (Some(name), Some(molality)) =
                (reported["name"].as_str(), reported["molality"].as_f64())
            else {
                continue;
            };
            *out.entry(format!("v{index}.m[{name}]")).or_insert(0.0) += molality;
        }
    }
    // The scene is a surface of its own, and the hand-written suite found a
    // defect that lived only there: a buoyancy verdict hard-coded against
    // water left `position: floating` beside the words "is at the bottom".
    // Words are not numbers, so they enter the picture as a hash — enough
    // to see that they MOVED, which is all a causal claim needs.
    for (index, vessel) in last["scene"]["vessels"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        if let Some(words) = vessel["words"].as_str() {
            out.insert(format!("s{index}.words"), stable_hash(words));
        }
        for (object, bulk) in vessel["bulk_objects"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            if let Some(position) = bulk["position"].as_str() {
                out.insert(format!("s{index}.o{object}.position"), stable_hash(position));
            }
        }
    }
    out
}

fn stable_hash(text: &str) -> f64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // Small enough to compare exactly in an f64, large enough not to collide.
    (hash % 1_000_000_007) as f64
}

/// Keys present in both pictures whose values differ by more than
/// `tolerance` relative, plus every key present in only one of them.
fn differences(
    before: &BTreeMap<String, f64>,
    after: &BTreeMap<String, f64>,
    tolerance: f64,
) -> Vec<String> {
    let keys: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    keys.into_iter()
        .filter(|key| match (before.get(*key), after.get(*key)) {
            (Some(a), Some(b)) => {
                let scale = a.abs().max(b.abs()).max(1e-12);
                (a - b).abs() / scale > tolerance
            }
            _ => true,
        })
        .cloned()
        .collect()
}

/// The subset of the picture a causal claim is allowed to be satisfied by:
/// no slot whose meaning is known to be wrong, and nothing that is
/// identically zero in both runs.
fn honest_differences(
    before: &BTreeMap<String, f64>,
    after: &BTreeMap<String, f64>,
    tolerance: f64,
) -> Vec<String> {
    differences(before, after, tolerance)
        .into_iter()
        .filter(|key| {
            !NOT_WHAT_IT_SAYS
                .iter()
                .any(|slot| key.contains(&format!("[{slot}|")) || key.contains(&format!("[{slot}]")))
        })
        .collect()
}

// ===================================================================
// The census: what the corpus admits, over all five hundred.
// ===================================================================

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct Census {
    prompts: usize,
    admitting: usize,
    admitting_none: usize,
    by_rule: BTreeMap<String, usize>,
    by_reason: BTreeMap<String, usize>,
    dose_sibling_groups: usize,
    dose_sibling_prompts: usize,
}

fn census() -> (Census, BTreeMap<String, Vec<Rule>>) {
    let prompts = corpus();
    let mut by_rule: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_prompt = BTreeMap::new();
    let (mut admitting, mut none) = (0, 0);
    for prompt in &prompts {
        let rules = if prompt.script.is_empty() {
            *by_reason
                .entry("no script: an explicit product boundary, nothing to run".into())
                .or_default() += 1;
            Vec::new()
        } else {
            let steps = parse(&prompt.script).unwrap_or_else(|| {
                panic!(
                    "{}: the generator cannot parse its own corpus: {:?}",
                    prompt.id, prompt.script
                )
            });
            let (rules, reason) = admits(&steps);
            if let Some(reason) = reason {
                *by_reason.entry(reason.to_string()).or_default() += 1;
            }
            // A rule is only admitted if it actually produces a different
            // script. `Order` over two identical adds does not.
            rules
                .into_iter()
                .filter(|rule| perturb(*rule, &steps).is_some())
                .collect()
        };
        if rules.is_empty() {
            none += 1;
        } else {
            admitting += 1;
        }
        for rule in &rules {
            *by_rule.entry(rule.name().to_string()).or_default() += 1;
        }
        per_prompt.insert(prompt.id.clone(), rules);
    }
    let siblings = dose_siblings();
    (
        Census {
            prompts: prompts.len(),
            admitting,
            admitting_none: none,
            by_rule,
            by_reason,
            dose_sibling_groups: siblings.len(),
            dose_sibling_prompts: siblings.iter().map(Vec::len).sum(),
        },
        per_prompt,
    )
}

/// Groups of prompts whose scripts are identical but for one or more
/// quantities: authored dose pairs, written independently by whoever wrote
/// the corpus, and the raw material of the only DIFFERENTIAL shape a
/// generator can reach without inventing chemistry. `aq-003` (10 g of KCl)
/// and `aq-107` (50 g) are one of them — the very row whose deadness is the
/// reason this file exists.
fn dose_siblings() -> Vec<Vec<CuriosityPrompt>> {
    let mut groups: BTreeMap<String, Vec<CuriosityPrompt>> = BTreeMap::new();
    for prompt in corpus() {
        if prompt.script.is_empty() {
            continue;
        }
        let Some(steps) = parse(&prompt.script) else {
            continue;
        };
        // The shape with every quantity erased. Observations are kept,
        // because `look` and `measure` do not change the state and two
        // scripts that differ only in how they are watched are still the
        // same experiment at two loadings.
        let shape = steps
            .iter()
            .map(|step| match step {
                Step::Add(add) => format!(
                    "add {} {} #{} @{:?}",
                    add.vessel, add.species, add.unit, add.temperature
                ),
                Step::Other { verb, line } => match verb.as_str() {
                    "look" | "measure" | "smell" | "inspect" | "test" => verb.clone(),
                    _ => line.clone(),
                },
            })
            .collect::<Vec<_>>()
            .join(";");
        groups.entry(shape).or_default().push(prompt);
    }
    groups
        .into_values()
        .filter(|group| {
            let loadings: BTreeSet<String> = group
                .iter()
                .map(|prompt| {
                    parse(&prompt.script)
                        .unwrap()
                        .iter()
                        .filter_map(Step::as_add)
                        .map(|add| trim(add.quantity))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .collect();
            loadings.len() > 1
        })
        .collect()
}

/// The census is checked in, so that a corpus edit which puts a script
/// beyond the generator's reach shows up as a diff with a number on it
/// rather than as silence.
#[test]
fn the_corpus_census_is_what_is_recorded() {
    let (measured, _) = census();
    let path = repo_root().join("tests/coverage/curiosity-v1/perturbation-census.json");
    let rendered = format!("{}\n", serde_json::to_string_pretty(&measured).unwrap());
    if std::env::var_os("KERO_BLESS_PERTURBATION_CENSUS").is_some() {
        std::fs::write(&path, &rendered).unwrap();
        return;
    }
    let recorded = std::fs::read_to_string(&path).expect("the census is checked in");
    assert_eq!(
        recorded.trim(),
        rendered.trim(),
        "the corpus moved under the generator. Re-run with \
         KERO_BLESS_PERTURBATION_CENSUS=1 and explain the diff in the commit."
    );
}

// ===================================================================
// The sweep: measure first, then assert. Every threshold in this file
// is derived from a number this harness printed, in the manner of
// `metamorphic.rs`, rather than guessed.
// ===================================================================

#[derive(Debug, serde::Serialize)]
struct Measurement {
    id: String,
    rule: String,
    baseline_error: Option<String>,
    perturbed_error: Option<String>,
    /// Largest relative disagreement between the two pictures, and where.
    worst: f64,
    worst_key: String,
    /// How many slots moved at all, ignoring the ones known to lie.
    moved: usize,
    /// Slots that exist in one run and not the other.
    appeared: usize,
    keys: usize,
}

fn measure(id: &str, rule: Rule, steps: &[Step]) -> Option<Measurement> {
    let perturbed_script = perturb(rule, steps)?;
    let baseline = run(&render(steps));
    let perturbed = run(&perturbed_script);
    let mut measurement = Measurement {
        id: id.to_string(),
        rule: rule.name().to_string(),
        baseline_error: baseline.as_ref().err().cloned(),
        perturbed_error: perturbed.as_ref().err().cloned(),
        worst: 0.0,
        worst_key: String::new(),
        moved: 0,
        appeared: 0,
        keys: 0,
    };
    let (Ok(baseline), Ok(perturbed)) = (baseline, perturbed) else {
        return Some(measurement);
    };
    let (before, after) = (observe(&baseline), observe(&perturbed));
    measurement.keys = before.len();
    measurement.moved = honest_differences(&before, &after, 1e-9).len();
    let keys: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    for key in keys {
        match (before.get(key), after.get(key)) {
            (Some(a), Some(b)) => {
                let scale = a.abs().max(b.abs()).max(1e-12);
                let relative = (a - b).abs() / scale;
                if relative > measurement.worst {
                    measurement.worst = relative;
                    measurement.worst_key.clone_from(key);
                }
            }
            _ => measurement.appeared += 1,
        }
    }
    Some(measurement)
}

/// Not part of the gate. `KERO_PERTURBATION_SWEEP=<path> cargo test -p
/// kerotakis-cli --test corpus_perturbation -- --ignored sweep` writes one
/// JSON line per generated case, which is how every threshold below was
/// chosen and how the reach was counted.
#[test]
#[ignore = "measurement harness: writes a report, asserts nothing"]
fn sweep() {
    let Some(path) = std::env::var_os("KERO_PERTURBATION_SWEEP") else {
        panic!("set KERO_PERTURBATION_SWEEP to the report path");
    };
    let only: Option<String> = std::env::var("KERO_PERTURBATION_ONLY").ok();
    let mut report = Vec::new();
    for prompt in corpus() {
        if prompt.script.is_empty() {
            continue;
        }
        let Some(steps) = parse(&prompt.script) else {
            continue;
        };
        let (rules, _) = admits(&steps);
        for rule in rules {
            if only.as_deref().is_some_and(|name| name != rule.name()) {
                continue;
            }
            if let Some(measurement) = measure(&prompt.id, rule, &steps) {
                println!("{}", serde_json::to_string(&measurement).unwrap());
                report.push(measurement);
            }
        }
    }
    let lines: Vec<String> = report
        .iter()
        .map(|m| serde_json::to_string(m).unwrap())
        .collect();
    std::fs::write(path, lines.join("\n") + "\n").unwrap();
}

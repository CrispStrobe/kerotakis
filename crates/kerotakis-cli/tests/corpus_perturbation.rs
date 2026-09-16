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
    let mut index = 0;
    while index < steps.len() {
        let Some(first) = steps[index].as_add() else {
            index += 1;
            continue;
        };
        let mut end = index + 1;
        while let Some(next) = steps.get(end).and_then(Step::as_add) {
            if next.vessel != first.vessel || next.temperature != first.temperature {
                break;
            }
            end += 1;
        }
        if best.is_none_or(|(from, to)| end - index > to - from) {
            best = Some((index, end));
        }
        index = end.max(index + 1);
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

/// A flattened, comparable picture of a run. Keys are stable strings so two
/// runs that disagree about which species EXIST are still comparable — an
/// appearing or vanishing species is a difference, not a panic.
///
/// The prefix on every key is load-bearing, because the rules do not all
/// read the same picture:
///
/// * `r.` — a READOUT: what somebody watching the experiment would come
///   away with. Instrument readings taken from the step stream's own
///   `measured` events, the derived properties of the solution, the
///   temperature, and the words and verdicts of the scene.
/// * `n.` — an amount, in moles, per species and phase.
/// * `m.` — a molality, out of the solver's own reported speciation.
///
/// The causal rules read only `r.`, and that restriction is the whole
/// difference between a real claim and a vacuous one. Comparing the full
/// inventory after deleting a reagent is trivially satisfied by the deleted
/// reagent no longer being in it; comparing the READOUT asks the question
/// the corpus row actually poses, which is whether the thermometer knew.
fn observe(steps: &[serde_json::Value]) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let Some(last) = steps.last() else {
        return out;
    };
    // Every instrument reading the run produced, in order, keyed by
    // instrument and occurrence. This is the surface the corpus scripts
    // were written to exercise: `measure v1 thermometer` is the whole
    // point of aq-003, and nothing until now compared what it said.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for step in steps {
        for event in step["events"].as_array().cloned().unwrap_or_default() {
            if event["event"] != "measured" {
                continue;
            }
            let instrument = event["instrument"].as_str().unwrap_or("instrument");
            let Some(value) = event["value"].as_f64() else {
                continue;
            };
            let count = seen.entry(instrument.to_string()).or_default();
            out.insert(format!("r.{instrument}#{count}"), value);
            *count += 1;
        }
    }
    let vessels = last["bench"]["vessels"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for (index, vessel) in vessels.iter().enumerate() {
        let mut put = |field: &str, value: Option<f64>| {
            if let Some(value) = value {
                out.insert(format!("r.v{index}.{field}"), value);
            }
        };
        put("ph", vessel["solution"]["ph"].as_f64());
        put("ionic_strength", vessel["solution"]["ionic_strength"].as_f64());
        put("pe", vessel["solution"]["pe"].as_f64());
        put("temperature", vessel["temperature"].as_f64());
        put("pressure", vessel["pressure"].as_f64());
        put("free_proton", vessel["free_proton"].as_f64());
        put(
            "co2_partial_pressure_atm",
            vessel["co2_partial_pressure_atm"].as_f64(),
        );
        // The solvent mass is what an intensive property is computed
        // against, so it belongs to the picture even though nothing reads
        // it directly: it is the quantity the SOLVENT rule moves.
        if let Some(kilograms) = vessel["solution"]["solvent_kg"].as_f64() {
            out.insert(format!("n.v{index}.solvent_kg"), kilograms);
        }
        for portion in vessel["contents"].as_array().cloned().unwrap_or_default() {
            let (Some(species), Some(moles)) =
                (portion["species"].as_str(), portion["moles"].as_f64())
            else {
                continue;
            };
            let phase = portion["phase"].as_str().unwrap_or("");
            *out.entry(format!("n.v{index}[{species}|{phase}]"))
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
            *out.entry(format!("m.v{index}[{name}]")).or_insert(0.0) += molality;
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
            out.insert(format!("r.s{index}.words"), stable_hash(words));
        }
        if let Some(mass) = vessel["mass_g"].as_f64() {
            out.insert(format!("n.s{index}.mass_g"), mass);
        }
        if let Some(kelvin) = vessel["temperature_k"].as_f64() {
            out.insert(format!("r.s{index}.temperature_k"), kelvin);
        }
        for (object, bulk) in vessel["bulk_objects"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .enumerate()
        {
            if let Some(position) = bulk["position"].as_str() {
                out.insert(
                    format!("r.s{index}.o{object}.position"),
                    stable_hash(position),
                );
            }
        }
    }
    out
}

/// Instruments whose reading counts the beaker rather than describing the
/// solution, so doubling the experiment doubles them.
fn extensive_instrument(key: &str) -> bool {
    key.starts_with("r.balance#") || key.starts_with("r.calorimeter#")
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

/// **The baseline run IS the corpus script.** Every generated case compares
/// a perturbed script against a baseline the generator re-rendered from its
/// own parse, so if that render were not byte-identical to the authored
/// line, the whole file would be measuring something nobody wrote. All 943
/// `add` lines in the corpus round-trip exactly; this is the assertion that
/// keeps it that way.
#[test]
fn the_generator_renders_the_corpus_back_exactly_as_written() {
    let mut mangled = Vec::new();
    for prompt in corpus() {
        if prompt.script.is_empty() {
            continue;
        }
        let steps = parse(&prompt.script).expect("every corpus script parses");
        let rendered = render(&steps);
        let original = format!("{}\n", prompt.script.join("\n"));
        if rendered != original {
            mangled.push(format!("{}\n  wrote: {original:?}\n  read:  {rendered:?}", prompt.id));
        }
    }
    assert!(
        mangled.is_empty(),
        "the generator does not render {} scripts back as they were \
         written, so their baselines are not the corpus:\n{}",
        mangled.len(),
        mangled.join("\n")
    );
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
// Running a case: two runs, two pictures.
// ===================================================================

/// One case in `density` is generated, in id order, per rule. The corpus is
/// not run whole: five hundred scripts against five rules is two and a half
/// thousand invocations of a solver at three and a half seconds each, which
/// is a nightly job and not a gate. What a gate needs is a sample that
/// cannot be gamed by the corpus growing, so selection is by a hash of the
/// id — stable under insertion, spread across all four shards, and
/// independent of anything the generator itself computed. Measured over the
/// five hundred ids, an FNV-1a taken modulo these divisors lands within a
/// point or two of the ideal fraction.
///
/// The densities differ by rule because the rules differ in worth. The
/// causal claim is sampled hardest and takes every prompt in the manifest's
/// smoke set besides, so the rows the rest of CI already watches are the
/// rows this watches too. `Dose`, which this file's own doc comment calls
/// the weakest shape, is sampled thinnest.
///
/// The whole corpus is still reachable: `KERO_PERTURBATION_ALL=1` on the
/// `sweep` harness ignores the subset entirely.
fn density(rule: Rule) -> u64 {
    match rule {
        Rule::Order => 23,
        Rule::Scale => 29,
        Rule::Solvent => 23,
        Rule::Ablation => 17,
        Rule::Dose => 61,
    }
}

/// A 64-bit FNV-1a, used to pick the subset. Separate from `stable_hash`,
/// which has to survive a round trip through an `f64` and is therefore
/// truncated — a truncation that costs it its uniformity modulo a small
/// divisor, which is exactly what this needs.
fn selection_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn selected(rule: Rule) -> Vec<(CuriosityPrompt, Vec<Step>)> {
    // The manifest's smoke set, NOT the shard's `smoke` field: 150 prompts
    // carry the field and 16 are in the set, and reading the wrong one
    // inflated an early sweep from 75 cases to 648.
    let smoke: BTreeSet<String> =
        load_manifest(&repo_root().join("tests/coverage/curiosity-v1/manifest.toml"))
            .expect("the corpus loads")
            .manifest
            .smoke_prompts
            .into_iter()
            .collect();
    corpus()
        .into_iter()
        .filter(|prompt| !prompt.script.is_empty())
        .filter_map(|prompt| {
            let steps = parse(&prompt.script)?;
            admits(&steps).0.contains(&rule).then_some((prompt, steps))
        })
        .filter(|(prompt, steps)| {
            perturb(rule, steps).is_some()
                && ((rule == Rule::Ablation && smoke.contains(&prompt.id))
                    || selection_hash(&prompt.id) % density(rule) == 0)
        })
        .collect()
}

struct Pair {
    before: BTreeMap<String, f64>,
    after: BTreeMap<String, f64>,
    error: Option<String>,
    seconds: f64,
}

fn pair(rule: Rule, steps: &[Step]) -> Option<Pair> {
    let perturbed_script = perturb(rule, steps)?;
    let started = std::time::Instant::now();
    let baseline = run(&render(steps));
    let perturbed = run(&perturbed_script);
    let seconds = started.elapsed().as_secs_f64();
    let error = match (&baseline, &perturbed) {
        (Err(error), _) => Some(format!("the corpus script itself failed: {error}")),
        (_, Err(error)) => Some(format!("the perturbed script failed: {error}")),
        _ => None,
    };
    Some(Pair {
        before: baseline.map(|steps| observe(&steps)).unwrap_or_default(),
        after: perturbed.map(|steps| observe(&steps)).unwrap_or_default(),
        error,
        seconds,
    })
}

/// A slot that describes the SOLUTION rather than the beaker, so doubling
/// the beaker must leave it alone.
fn is_intensive(key: &str) -> bool {
    if extensive_instrument(key) {
        return false;
    }
    key.starts_with("m.")
        || key.starts_with("r.") && !key.ends_with(".words") && !key.ends_with(".position")
}

/// A slot counted in moles or litres, which doubles with the beaker.
fn is_extensive(key: &str) -> bool {
    key.starts_with("n.") || extensive_instrument(key)
}

/// A reading the script itself took, as against a property the bench
/// happens to expose. `measure v1 thermometer` produces one of these and
/// `look v1` does not.
fn is_instrument_reading(key: &str) -> bool {
    key.starts_with("r.") && key.contains('#')
}

/// An amount of something that was DISSOLVED rather than something that
/// dissolved it. Twice the water holds the first fixed and doubles the
/// second, so only the first is the SOLVENT rule's invariant.
fn is_solute_amount(key: &str) -> bool {
    if key.ends_with(".solvent_kg") || key.ends_with(".mass_g") {
        return false;
    }
    !SOLVENTS
        .iter()
        .chain(["H2O", "H+", "OH-"].iter())
        .any(|name| key.contains(&format!("[{name}|")))
}

/// The readout: what somebody watching the experiment comes away with, as
/// against the inventory underneath it. The causal rules read this and
/// nothing else.
fn is_readout(key: &str) -> bool {
    key.starts_with("r.")
}

/// Whether a key is one a claim of the form "something moved" may be
/// satisfied by. `contents[OH-]` is not: it is the solution's residual
/// cation charge under hydroxide's name, recorded and unfixed, and a
/// generated case that counted it would be resting on a defect.
fn trustworthy(key: &str) -> bool {
    !NOT_WHAT_IT_SAYS
        .iter()
        .any(|slot| key.contains(&format!("[{slot}|")) || key.contains(&format!("[{slot}]")))
}

fn relative(a: f64, b: f64) -> f64 {
    let scale = a.abs().max(b.abs()).max(1e-12);
    (a - b).abs() / scale
}

/// The worst offender against a per-key expectation, as prose, or `None`
/// when every key met it. `expect` returns the value the perturbed run
/// should show, or `None` for a key this claim says nothing about.
fn worst_against(
    before: &BTreeMap<String, f64>,
    after: &BTreeMap<String, f64>,
    tolerance: f64,
    expect: impl Fn(&str, f64) -> Option<f64>,
) -> Option<String> {
    let mut worst: Option<(f64, String)> = None;
    for (key, a) in before {
        let Some(wanted) = expect(key, *a) else {
            continue;
        };
        let Some(b) = after.get(key) else {
            return Some(format!("{key} exists in one run and not the other"));
        };
        let deviation = relative(wanted, *b);
        if deviation > tolerance && worst.as_ref().is_none_or(|(w, _)| deviation > *w) {
            worst = Some((
                deviation,
                format!("{key}: expected {wanted:.6e}, got {b:.6e} ({deviation:.2e} relative)"),
            ));
        }
    }
    worst.map(|(_, prose)| prose)
}

/// Keys that moved, ignoring the ones known to lie and the ones that are
/// zero in both runs.
fn moved(before: &BTreeMap<String, f64>, after: &BTreeMap<String, f64>, tolerance: f64) -> Vec<String> {
    let keys: BTreeSet<&String> = before.keys().chain(after.keys()).collect();
    keys.into_iter()
        .filter(|key| trustworthy(key))
        .filter(|key| match (before.get(*key), after.get(*key)) {
            (Some(a), Some(b)) => relative(*a, *b) > tolerance,
            _ => true,
        })
        .cloned()
        .collect()
}

/// What the rule claims of one case, as a verdict: `None` when the claim
/// held, `Some(prose)` when it did not.
fn verdict(rule: Rule, case: &Pair) -> Option<String> {
    if let Some(error) = &case.error {
        return Some(error.clone());
    }
    let (before, after) = (&case.before, &case.after);
    match rule {
        // Everything comes back the same.
        Rule::Order => worst_against(before, after, ORDER_TOLERANCE, |_, value| Some(value)),
        // Intensive alone; extensive doubled.
        Rule::Scale => worst_against(before, after, SCALE_TOLERANCE, |key, value| {
            if is_intensive(key) {
                Some(value)
            } else if is_extensive(key) {
                Some(value * 2.0)
            } else {
                None
            }
        }),
        // Twice the solvent: every amount of every SOLUTE is untouched,
        // and the solution is more dilute than it was. The second half is
        // the one a stored-concentration bug fails, and it is skipped for
        // a vessel with no ions in it at all, where "more dilute" has no
        // referent.
        Rule::Solvent => {
            let amounts = worst_against(before, after, SOLVENT_TOLERANCE, |key, value| {
                (key.starts_with("n.") && is_solute_amount(key)).then_some(value)
            });
            if amounts.is_some() {
                return amounts;
            }
            let strength: Vec<(&String, f64, f64)> = before
                .iter()
                .filter(|(key, value)| key.ends_with(".ionic_strength") && **value > 1e-6)
                .filter_map(|(key, value)| after.get(key).map(|now| (key, *value, *now)))
                .collect();
            strength
                .iter()
                .find(|(_, was, now)| *now >= *was * (1.0 - SOLVENT_TOLERANCE))
                .map(|(key, was, now)| {
                    format!("{key}: twice the solvent left it at {now:.6e}, was {was:.6e}")
                })
        }
        // The answer has to depend on the reagent the question is about.
        // READOUT only: comparing the inventory would be satisfied by the
        // ablated reagent's own absence, which is not a claim about
        // anything.
        Rule::Ablation | Rule::Dose => {
            let readout: Vec<String> = moved(before, after, CAUSAL_TOLERANCE)
                .into_iter()
                .filter(|key| is_readout(key))
                .collect();
            // When the script picked up an instrument, THAT is the number
            // the question was asked about, and it is the one that has to
            // answer. `aq-003` measures a thermometer; a bench that reports
            // the room's temperature whatever is in the beaker passes
            // "something moved" on the pH alone, and must not.
            let instruments: Vec<&String> = before
                .keys()
                .filter(|key| is_instrument_reading(key))
                .collect();
            if !instruments.is_empty() {
                let moved_instrument = readout.iter().any(|key| is_instrument_reading(key));
                return (!moved_instrument).then(|| {
                    format!(
                        "the instrument the script picked up did not move: {}",
                        instruments
                            .iter()
                            .map(|key| format!("{key}={:.6}", before[*key]))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                });
            }
            readout.is_empty().then(|| {
                format!(
                    "the readout did not move: {} readings, every one unchanged",
                    before.keys().filter(|key| is_readout(key)).count(),
                )
            })
        }
    }
}

/// Relative. These compare hundreds of unrelated quantities across
/// hundreds of unrelated recipes, so they are looser than the hand-tuned
/// absolute tolerances in `metamorphic.rs` — which sets pH at 1e-5
/// absolute against a measured 1.9e-6 floor. Each was set from the sweep
/// report; see `tests/coverage/curiosity-v1/perturbation-report.md`.
const ORDER_TOLERANCE: f64 = 1e-6;
const SCALE_TOLERANCE: f64 = 1e-6;
const SOLVENT_TOLERANCE: f64 = 1e-6;
/// Deliberately blunt: a causal claim asks whether a number moved AT ALL,
/// and a threshold near the solver's noise floor would let a rounding
/// difference pass as a dependency.
const CAUSAL_TOLERANCE: f64 = 1e-6;

/// Run a rule over its subset, and compare the rows that departed from its
/// claim against the rows recorded as departing. A gate that merely skipped
/// them would be the corpus's own disease over again, so the comparison is
/// an equality: a new departure fails, and a departure that heals fails too
/// until somebody deletes it and says what fixed it.
fn gate(rule: Rule) -> (usize, Vec<(String, String)>) {
    let subset = selected(rule);
    let mut departed = Vec::new();
    for (prompt, steps) in &subset {
        let Some(case) = pair(rule, steps) else {
            continue;
        };
        if let Some(why) = verdict(rule, &case) {
            departed.push((prompt.id.clone(), why));
        }
    }
    let recorded = recorded_departures(rule);
    let unexplained: Vec<String> = departed
        .iter()
        .filter(|(id, _)| !recorded.contains_key(id.as_str()))
        .map(|(id, why)| format!("  {id}: {why}"))
        .collect();
    assert!(
        unexplained.is_empty(),
        "{} of {} generated {} cases departed with no recorded reason:\n{}",
        unexplained.len(),
        subset.len(),
        rule.name(),
        unexplained.join("\n")
    );
    let healed: Vec<&str> = recorded
        .keys()
        .copied()
        .filter(|id| subset.iter().any(|(prompt, _)| prompt.id == *id))
        .filter(|id| !departed.iter().any(|(found, _)| found == id))
        .collect();
    assert!(
        healed.is_empty(),
        "{healed:?} no longer depart from the {} claim; delete them from the \
         recorded list and say what fixed them",
        rule.name()
    );
    (subset.len(), departed)
}

// Recorded departures. Each entry is a row and the reason it departs, both
// filled from the sweep rather than from an argument.
const ORDER_DEPARTURES: &[(&str, &str)] = &[];
const SCALE_DEPARTURES: &[(&str, &str)] = &[];
const SOLVENT_DEPARTURES: &[(&str, &str)] = &[];
const ABLATION_INERT: &[(&str, &str)] = &[];
const DOSE_INERT: &[(&str, &str)] = &[];

fn recorded_departures(rule: Rule) -> BTreeMap<&'static str, &'static str> {
    match rule {
        Rule::Order => ORDER_DEPARTURES,
        Rule::Scale => SCALE_DEPARTURES,
        Rule::Solvent => SOLVENT_DEPARTURES,
        Rule::Ablation => ABLATION_INERT,
        Rule::Dose => DOSE_INERT,
    }
    .iter()
    .copied()
    .collect()
}

// ===================================================================
// The measurement harness. Not a gate: it writes a report, and every
// threshold and every recorded departure above came out of it.
// ===================================================================

/// `KERO_PERTURBATION_SWEEP=<path> cargo test -p kerotakis-cli --test
/// corpus_perturbation -- --ignored --nocapture sweep`, optionally with
/// `KERO_PERTURBATION_ONLY=<rule>` and `KERO_PERTURBATION_ALL=1` to leave
/// the subset behind and run the whole corpus.
#[test]
#[ignore = "measurement harness: writes a report, asserts nothing"]
fn sweep() {
    let path = std::env::var_os("KERO_PERTURBATION_SWEEP")
        .expect("set KERO_PERTURBATION_SWEEP to the report path");
    let only = std::env::var("KERO_PERTURBATION_ONLY").ok();
    let whole = std::env::var_os("KERO_PERTURBATION_ALL").is_some();
    let mut lines = Vec::new();
    for rule in [
        Rule::Order,
        Rule::Scale,
        Rule::Solvent,
        Rule::Ablation,
        Rule::Dose,
    ] {
        if only.as_deref().is_some_and(|name| name != rule.name()) {
            continue;
        }
        let subset = if whole {
            corpus()
                .into_iter()
                .filter(|prompt| !prompt.script.is_empty())
                .filter_map(|prompt| {
                    let steps = parse(&prompt.script)?;
                    admits(&steps).0.contains(&rule).then_some((prompt, steps))
                })
                .collect()
        } else {
            selected(rule)
        };
        for (prompt, steps) in subset {
            let Some(case) = pair(rule, &steps) else {
                continue;
            };
            let moved = moved(&case.before, &case.after, CAUSAL_TOLERANCE);
            let line = serde_json::json!({
                "id": prompt.id,
                "rule": rule.name(),
                "verdict": verdict(rule, &case),
                "moved": moved.len(),
                "moved_keys": moved.iter().take(6).collect::<Vec<_>>(),
                "keys": case.before.len(),
                "seconds": case.seconds,
                "question": prompt.question,
                "script": prompt.script,
            });
            println!("{line}");
            lines.push(line.to_string());
        }
    }
    std::fs::write(path, lines.join("\n") + "\n").unwrap();
}

// ===================================================================
// The generated cases.
// ===================================================================

/// **Equilibrium has no memory, over recipes nobody wrote for this test.**
/// Reverse a contiguous run of additions to one vessel; every number on the
/// bench must come back the same.
///
/// `metamorphic.rs` makes this claim once, for silver chloride, and its own
/// doc comment records what it cost: six rounds of diagnosis across two
/// sessions, ending in two real engine bugs — dissolution enthalpy
/// unrecorded for phases the routed database cannot name, and sensible heat
/// destroyed by `t0 + q/cp` whenever speciation shrank the vessel's heat
/// capacity. One pair of scripts found both. This generates the same pair
/// from every corpus script whose shape admits it.
///
/// **What this establishes:** that the state a recipe reaches does not
/// depend on the sequence it was assembled in, across the span of chemistry
/// the corpus actually covers rather than one precipitation. Path
/// dependence is the signature of state carried forward instead of
/// re-solved, and no single run can see it.
///
/// **What it cannot establish:** that the state is right. Both orders could
/// be equally wrong, and this would pass.
///
/// **Weakness: low.** Nothing here is a quantity the engine multiplies
/// through — the two runs are the same arithmetic in a different sequence,
/// which a solver cannot satisfy by construction. It is also the generated
/// rule least likely to be vacuous, because the comparison covers every
/// slot: a script with nothing interesting in it still has its temperature,
/// its volume and its scene compared.
#[test]
fn addition_order_does_not_change_where_the_corpus_ends_up() {
    let (ran, departed) = gate(Rule::Order);
    assert!(ran >= 10, "the order subset shrank to {ran} cases");
    assert_eq!(departed.len(), ORDER_DEPARTURES.len());
}

/// **The same experiment twice as big is the same experiment.** Double
/// every quantity the script states — amounts, the joules of a `heat`, the
/// headspace of a `seal` — and no intensive property may move, while every
/// amount must double.
///
/// **What this establishes:** that no absolute size leaks into a quantity
/// which describes the solution rather than the beaker. A concentration
/// computed against a hard-coded litre, a rate law that lost its per-volume
/// normalisation, an enthalpy divided by the wrong mass — each of them
/// produces a pH or a temperature that moves when only the beaker did, and
/// each is invisible in a single run.
///
/// **What it cannot establish:** any value at all. It is a statement about
/// homogeneity, degree zero and degree one, and nothing else.
///
/// **Weakness: low, with one caveat the rule takes seriously.** A script
/// containing a verb whose size the script does not state is EXCLUDED by
/// `SIZE_BOUND` rather than asserted loosely: a flame, a flux over an
/// unstated area, a discrete titration step and an electrode current are
/// not quantities the script controls, so doubling around them is a
/// different experiment and a failure there would mean nothing. That
/// exclusion is why this rule reaches fewer scripts than `Dose` does.
#[test]
fn doubling_the_corpus_leaves_every_intensive_property_alone() {
    let (ran, departed) = gate(Rule::Scale);
    assert!(ran >= 10, "the scale subset shrank to {ran} cases");
    assert_eq!(departed.len(), SCALE_DEPARTURES.len());
}

/// **Twice the water, the same amount of everything else.** The count you
/// weighed is not negotiable by the solvent, and the solution it is in must
/// get more dilute.
///
/// The pairing is the whole test, exactly as in `perturbation.rs` 6/7. A
/// quantity asserted only to be constant is satisfied by a constant; a
/// quantity asserted only to move is satisfied by anything that moves. The
/// two together pin down which of them the engine believes depends on the
/// beaker, and the classic bug they catch is `n = c · V` recovered on
/// readback, which makes the count follow the water.
///
/// **What this establishes:** amounts of substance are stored rather than
/// recomputed from a concentration, and the ionic strength is computed from
/// a solvent mass that the script can actually move.
///
/// **What it cannot establish:** that either number is right, or that the
/// dilution has the right functional form. Only that one is held and the
/// other is not.
///
/// **Weakness: moderate on the invariance half, low on the direction
/// half.** Where the engine stores moles, "the moles did not change" is
/// close to the path it tests — the same criticism `perturbation.rs` makes
/// of its own case 5. The falling ionic strength is not: it is recomputed
/// from a solvent mass, through speciation, on every run.
///
/// **The interesting exception, which is chemistry rather than noise:** a
/// SATURATED solution does not dilute. Adding water to a beaker with
/// undissolved salt at the bottom dissolves more salt and the ionic
/// strength comes back where it was. Any corpus row that departs here for
/// that reason is a row where the generator has accidentally found the
/// solubility limit, and it is recorded with that reason rather than
/// excused.
#[test]
fn twice_the_solvent_dilutes_the_corpus_without_moving_the_amounts() {
    let (ran, departed) = gate(Rule::Solvent);
    assert!(ran >= 8, "the solvent subset shrank to {ran} cases");
    assert_eq!(departed.len(), SOLVENT_DEPARTURES.len());
}

/// **The answer must depend on the reagent the question is about.** This is
/// the rule that exists because of `aq-003`: delete the last non-solvent
/// reagent from the script and something on the bench must move.
///
/// It is the cheapest claim in the file and the one most directly aimed at
/// what is wrong with the corpus it is generated from. A row whose
/// `expected` records only a route is green whatever number comes back; a
/// row that survives this one has at least established that the number is
/// downstream of the chemistry the question names. `aq-003` asks whether
/// potassium chloride cools a beaker. This does not know that it should
/// cool. It knows that if the potassium chloride is not there, the
/// thermometer must read something else — and an engine that answers that
/// question without consulting the salt fails, which is precisely the
/// failure the corpus could not see.
///
/// **What this establishes:** that the observation is causally downstream
/// of the reagent. Where the script picked up an instrument, that
/// instrument's own reading is what must move — not merely something on the
/// bench. Without that restriction the rule would be nearly vacuous, twice
/// over: deleting a reagent removes it from the inventory, and pH moves
/// when almost anything moves. The claim as written is the one `aq-003`
/// poses and could not check.
///
/// **What it cannot establish:** direction, magnitude, or correctness. A
/// beaker that warmed when it should have cooled passes this happily. It is
/// a floor, and it is worth having only because the floor was previously at
/// zero.
///
/// **Weakness: the claim is weak; the perturbation is not.** Deleting an
/// input is as far from "a quantity the engine multiplies through" as a
/// perturbation gets — there is no path by which an engine passes this
/// without reading the reagent. What is weak is the conclusion: `≠` is the
/// least you can ask. The rows that FAIL it are the valuable output, and
/// they are recorded by name in `ABLATION_INERT`.
///
/// `contents["OH-"]` cannot satisfy this test. That slot carries the
/// solution's residual cation charge under hydroxide's name — recorded,
/// unfixed, wrong by six orders of magnitude in a buffer — so a case that
/// passed only because that number moved would be resting on a defect. It
/// is excluded by name in `trustworthy`, and without that exclusion this
/// rule would have counted it.
#[test]
fn the_corpus_answer_depends_on_the_reagent_the_question_is_about() {
    let (ran, departed) = gate(Rule::Ablation);
    assert!(ran >= 20, "the ablation subset shrank to {ran} cases");
    assert_eq!(departed.len(), ABLATION_INERT.len());
}

/// **Twice the reagent, a different answer.** The weakest generated shape,
/// and generated anyway because the rows it cannot reach are informative.
///
/// **Weakness: high, and for exactly the reason the brief warns about.** A
/// perturbation that varies a quantity the code multiplies through always
/// passes. Doubling a solute doubles its moles, which doubles its molality,
/// which moves the ionic strength: that chain is arithmetic and this test
/// will ride it every time. It is kept for the rows where the chain BREAKS
/// — a saturated solution, a reagent already in excess, a limiting-reagent
/// situation where the second half does nothing — because those are the
/// rows where a dose response genuinely tells you something, and they are
/// exactly the rows the corpus's `expected` field could never distinguish.
///
/// It is sampled at a quarter the density of the others and must not be
/// counted as evidence about dose response.
#[test]
fn twice_the_reagent_moves_the_corpus_answer() {
    let (ran, departed) = gate(Rule::Dose);
    assert!(ran >= 5, "the dose subset shrank to {ran} cases");
    assert_eq!(departed.len(), DOSE_INERT.len());
}

// ===================================================================
// The differential case, built out of the corpus's own authored pairs.
// ===================================================================

/// **The same recipe at two loadings, authored independently, must not be
/// answered alike — and the corpus already contains twenty-four groups of
/// them.**
///
/// This is the strongest shape a generator can reach without inventing
/// chemistry, and it costs nothing, because both halves of the pair were
/// written by hand by whoever wrote the corpus. `aq-003` puts 10 g of
/// potassium chloride into 100 mL of water and reads a thermometer;
/// `aq-107` puts in 50 g and reads the same thermometer. Neither row can
/// say anything about the chemistry on its own — that is the whole
/// complaint against the corpus — but the PAIR can, and nobody had to
/// decide what the right temperature is to make it say it.
///
/// The claim is a ratio between two responses rather than either response's
/// value, which is the property `perturbation.rs` 2/7 argues for: a global
/// error cannot fake it, because a global error moves both members equally
/// and this asserts they move differently.
///
/// **What this establishes:** that the engine distinguishes two loadings of
/// the same recipe, on the surface the script's own instrument reads. It is
/// the perturbation-in-time twin of `tools/curiosity-answer-invariance.py`,
/// which asks the same question across vessels and caught `mat-012` — a row
/// that passed for as long as the corpus existed while weighing five grams
/// of three different metals and reading the same number three times.
///
/// **What it cannot establish:** which of the two answers is right, or that
/// the difference has the right size or even the right sign.
///
/// **Weakness: low as a shape, moderate as a claim.** The shape is
/// differential and cannot be satisfied by a constant or by an engine that
/// ignores the input. The claim is still only `≠`. Where a group has three
/// or more loadings the test also asserts that the responses are ORDERED by
/// the loading, which two points cannot check and which a merely
/// discriminating engine can still fail.
///
/// **The rows it cannot reach are the interesting ones.** A group whose
/// members differ only in a quantity the engine has already saturated —
/// 50 g of salt in 100 mL of water is past solubility and so is 30 g — is
/// entitled to answer both alike, and does. Those are recorded by name.
#[test]
fn authored_dose_siblings_are_not_answered_alike() {
    let groups = dose_siblings();
    assert!(
        groups.len() >= 12,
        "the corpus lost its authored dose pairs: {} groups",
        groups.len()
    );
    let recorded: BTreeMap<&str, &str> = SIBLING_ALIKE.iter().copied().collect();
    let mut alike = Vec::new();
    let mut discriminated = 0usize;
    for group in &groups {
        let mut members: Vec<(String, f64, BTreeMap<String, f64>)> = Vec::new();
        for prompt in group {
            let steps = parse(&prompt.script).expect("a sibling parses");
            let loading = steps
                .iter()
                .filter_map(Step::as_add)
                .filter(|add| !is_solvent(&add.species))
                .map(|add| add.quantity)
                .next_back()
                .unwrap_or_default();
            match run(&render(&steps)) {
                Ok(steps) => members.push((prompt.id.clone(), loading, observe(&steps))),
                Err(error) => members.push((
                    prompt.id.clone(),
                    loading,
                    BTreeMap::from([(format!("failed: {error}"), 0.0)]),
                )),
            }
        }
        members.sort_by(|a, b| a.1.total_cmp(&b.1));
        let name = members
            .iter()
            .map(|(id, ..)| id.as_str())
            .collect::<Vec<_>>()
            .join("+");
        let (first, last) = (members.first().unwrap(), members.last().unwrap());
        if first.1 == last.1 {
            continue; // the varying quantity was the solvent's, not a reagent's
        }
        let separated = moved(&first.2, &last.2, CAUSAL_TOLERANCE);
        if separated.is_empty() {
            alike.push((name.clone(), first.1, last.1));
            continue;
        }
        discriminated += 1;
        // Three loadings or more: the responses must be ORDERED, not merely
        // different. Checked on the key that separates the extremes most,
        // which is the engine's own choice of what this pair is about.
        if members.len() >= 3 {
            let key = separated
                .iter()
                .max_by(|a, b| {
                    relative(
                        first.2.get(*a).copied().unwrap_or(0.0),
                        last.2.get(*a).copied().unwrap_or(0.0),
                    )
                    .total_cmp(&relative(
                        first.2.get(*b).copied().unwrap_or(0.0),
                        last.2.get(*b).copied().unwrap_or(0.0),
                    ))
                })
                .expect("a separating key");
            let track: Vec<f64> = members
                .iter()
                .map(|(_, _, picture)| picture.get(key).copied().unwrap_or(0.0))
                .collect();
            let rising = track.windows(2).all(|pair| pair[1] >= pair[0]);
            let falling = track.windows(2).all(|pair| pair[1] <= pair[0]);
            assert!(
                rising || falling,
                "{name}: {key} is not ordered by the loading: {track:?} at \
                 loadings {:?}",
                members.iter().map(|(_, q, _)| *q).collect::<Vec<_>>()
            );
        }
    }
    let unexplained: Vec<&(String, f64, f64)> = alike
        .iter()
        .filter(|(name, ..)| !recorded.contains_key(name.as_str()))
        .collect();
    assert!(
        unexplained.is_empty(),
        "{} authored dose groups were answered identically with no recorded \
         reason:\n{}",
        unexplained.len(),
        unexplained
            .iter()
            .map(|(name, low, high)| format!("  {name}: {low} and {high} agree everywhere"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        discriminated >= 8,
        "only {discriminated} authored dose groups were told apart"
    );
}

/// Sibling groups the engine answers identically, and why each is entitled
/// to. Filled from the sweep.
const SIBLING_ALIKE: &[(&str, &str)] = &[];

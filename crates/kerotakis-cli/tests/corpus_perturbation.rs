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

    let mut rules = Vec::new();
    if interchangeable_run(steps).is_some() {
        rules.push(Rule::Order);
    }
    if !steps.iter().any(|step| SIZE_BOUND.contains(&step.verb())) {
        rules.push(Rule::Scale);
    }
    if adds.iter().any(|add| add.species == "water") && reagents > 0 {
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
            // Reverse the REAGENTS inside the run and leave every solvent
            // line where it is. Reversing the whole run would move the
            // water to the end, and a reagent poured into an empty vessel
            // is a different experiment rather than the same one in a
            // different sequence — the version that put the water last
            // agreed with the original to 1.3% on a limewater script whose
            // two reagent orders differ by 2.55 pH units.
            let (from, to) = interchangeable_run(steps)?;
            let mut reagents: Vec<usize> = (from..to)
                .filter(|index| {
                    steps[*index]
                        .as_add()
                        .is_some_and(|add| !is_solvent(&add.species))
                })
                .collect();
            if reagents.len() < 2 {
                return None;
            }
            let originals: Vec<Step> = reagents.iter().map(|index| steps[*index].clone()).collect();
            reagents.reverse();
            for (slot, step) in reagents.into_iter().zip(originals) {
                perturbed[slot] = step;
            }
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
            // WATER only. Doubling any solvent looked like the general
            // rule until `bio-017` — oil, vinegar and mustard — doubled its
            // vegetable oil and left the vinegar's ionic strength exactly
            // where it was, correctly. Ionic strength is computed against
            // the mass of WATER, so that is the quantity whose doubling is
            // a claim about anything.
            let mut touched = false;
            for step in &mut perturbed {
                if let Step::Add(add) = step {
                    if add.species == "water" {
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
    if steps.is_empty() {
        return out;
    }
    // NOT simply the last step. `particles v1` emits a step carrying its own
    // populations and NO `bench`, so reading `steps.last()` for the bench
    // found nothing and `aq-049` — "what dissolved ions are present in a
    // sodium chloride solution?" — came back with zero readouts and an
    // ablation case that could not fail. Each surface is taken from the
    // last step that actually carries it.
    let latest = |field: &str| -> serde_json::Value {
        steps
            .iter()
            .rev()
            .find(|step| !step[field].is_null())
            .map(|step| step[field].clone())
            .unwrap_or(serde_json::Value::Null)
    };
    let bench = latest("bench");
    let last = latest("scene");
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
    let vessels = bench["vessels"].as_array().cloned().unwrap_or_default();
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
        // Moles, not a concentration: `free_proton` is the proton activity
        // already multiplied by the solvent mass, so it is extensive and
        // belongs with the amounts. Filing it as a readout made every
        // generated scale case fail at exactly 0.5 relative, which is what
        // an extensive quantity looks like when it is asked to hold still.
        if let Some(protons) = vessel["free_proton"].as_f64() {
            out.insert(format!("n.v{index}.free_proton"), protons);
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
    for (index, vessel) in last["vessels"]
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
    // `particles v1` is an observation like any other, and it is the only
    // thing some corpus scripts do. Its populations are what the script
    // asked to see, so they are a readout.
    for step in steps {
        let particles = &step["particles"];
        if particles.is_null() {
            continue;
        }
        for population in particles["populations"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            let (Some(label), Some(amount)) = (
                population["label"].as_str(),
                population["amount"].as_f64(),
            ) else {
                continue;
            };
            out.insert(format!("r.particles#[{label}]"), amount);
        }
        for rare in particles["too_rare"].as_array().cloned().unwrap_or_default() {
            let (Some(label), Some(amount)) = (rare[0].as_str(), rare[1].as_f64()) else {
                continue;
            };
            out.insert(format!("r.particles#[{label}]"), amount);
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
            // script. `Order` over two identical adds does not, and if that
            // leaves nothing at all the prompt needs its own reason, or the
            // reasons stop adding up to the count.
            let kept: Vec<Rule> = rules
                .iter()
                .copied()
                .filter(|rule| perturb(*rule, &steps).is_some())
                .collect();
            if kept.is_empty() && !rules.is_empty() {
                *by_reason
                    .entry(
                        "the script's shape admits a rule but applying it changes \
                         nothing: the same reagent twice, or one add"
                            .into(),
                    )
                    .or_default() += 1;
            }
            kept
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
        Rule::Order => 13,
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

/// Whether two readings of the same slot are distinguishable at all, or
/// whether the difference between them is the wire, the solver's
/// convergence, or arithmetic residue. Three floors, each set from a
/// measured departure rather than from an argument:
///
/// * **Print precision.** A reported molality is four significant figures —
///   `"molality": 55.51` — so two runs agreeing to eight digits can still
///   print a part in a thousand apart. Found on `aq-023`: 1.516e-3 against
///   1.517e-3, one unit in the last printed place. `metamorphic.rs`
///   excluded print precision by hand and said so, "so nobody re-litigates
///   them"; a generator has to exclude it by rule, because it compares
///   hundreds of speciation entries it never chose.
/// * **The solver's own floor on an amount.** `metamorphic.rs` derives
///   1e-9 mol absolute for element totals from a measured 6e-11 post-fix
///   residual, and that is the right tier here too: `aq-023` again, with
///   7.736798e-7 against 7.735965e-7 mol of calcium — 1.1e-4 relative, and
///   8.3e-11 mol absolute, an order of magnitude under the repo's own
///   derived tolerance.
/// * **Dust.** Below 1e-9 mol a quantity is residue rather than an amount
///   of anything. `aq-055` carried 1e-11 mol of hydrogen peroxide after its
///   catalase had eaten the rest, and that dust failed a scale case at
///   exactly 0.5 relative, because dust does not double.
fn indistinguishable(key: &str, a: f64, b: f64, tolerance: f64) -> bool {
    let (floor, absolute) = if key.starts_with("m.") {
        (tolerance.max(1e-3), 1e-12)
    } else if key.starts_with("n.") {
        (tolerance, 1e-9)
    } else {
        (tolerance, 0.0)
    };
    (a.abs() < absolute && b.abs() < absolute)
        || (a - b).abs() < absolute
        || relative(a, b) <= floor
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
            if indistinguishable(key, *a, 0.0, tolerance) {
                continue;
            }
            return Some(format!("{key} exists in one run and not the other"));
        };
        if indistinguishable(key, wanted, *b, tolerance) {
            continue;
        }
        let deviation = relative(wanted, *b);
        if worst.as_ref().is_none_or(|(w, _)| deviation > *w) {
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
            (Some(a), Some(b)) => !indistinguishable(key, *a, *b, tolerance),
            (Some(a), None) | (None, Some(a)) => !indistinguishable(key, *a, 0.0, tolerance),
            (None, None) => false,
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
        // Twice the solvent must DO something, and there are exactly two
        // things it can do: dilute what is dissolved, or dissolve more of
        // what is not. A beaker that answers "nothing changed" to twice
        // the water fails, whichever branch it was entitled to take.
        //
        // The first draft of this rule also held every solute amount
        // fixed, and the corpus refuted it in three rows: `aq-006` moved
        // 2% of its antlerite, `aq-057` reshuffled hypochlorous acid
        // against hypochlorite, and `aq-107` — 50 g of potassium chloride
        // in 100 mL — stopped having any solid at all. All three are
        // right. Dilution shifts speciation and dissolves solid, so the
        // conserved quantity is the ELEMENT total, which the `--json`
        // contract does not expose for a named material like `antlerite`
        // or `milk`. Asserting a species amount instead would have been a
        // false claim mechanically generated 259 times.
        Rule::Solvent => {
            let strengths: Vec<(&String, f64, f64)> = before
                .iter()
                .filter(|(key, value)| key.ends_with(".ionic_strength") && **value > 1e-6)
                .filter_map(|(key, value)| after.get(key).map(|now| (key, *value, *now)))
                .collect();
            let solid = |picture: &BTreeMap<String, f64>| -> f64 {
                picture
                    .iter()
                    .filter(|(key, _)| key.starts_with("n.") && key.ends_with("|solid]"))
                    .map(|(_, moles)| *moles)
                    .sum()
            };
            let (solid_before, solid_after) = (solid(before), solid(after));
            let dissolved_more = solid_before > 1e-9
                && solid_after < solid_before * (1.0 - SOLVENT_TOLERANCE);
            if dissolved_more {
                return None;
            }
            if strengths.is_empty() {
                // Nothing dissolved and nothing to dissolve: the vessel has
                // no characterised solution, so twice the water is not a
                // claim about anything.
                return Some(
                    "no solution and no solid: the vessel cannot be diluted".to_string(),
                );
            }
            strengths
                .iter()
                .find(|(_, was, now)| *now >= *was * (1.0 - SOLVENT_TOLERANCE))
                .map(|(key, was, now)| {
                    format!(
                        "{key}: twice the solvent left it at {now:.6e}, was {was:.6e}, \
                         and no solid dissolved either ({solid_before:.3e} -> \
                         {solid_after:.3e} mol)"
                    )
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
/// Measured: every generated order case that is not an open vessel losing a
/// gas agreed to better than 1e-9 relative, so this has three orders of
/// headroom.
const ORDER_TOLERANCE: f64 = 1e-6;
/// Looser, because `pe` is the worst-conditioned number in the picture:
/// `aq-032` and `th-099`, both sealed carbonate systems, reproduce it to
/// 9.2e-6 and 8.1e-5 relative across a doubling that leaves every amount
/// exact. The substantive scale departures are all above 1e-2, so nothing
/// interesting hides under this.
const SCALE_TOLERANCE: f64 = 3e-4;
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
const ORDER_DEPARTURES: &[(&str, &str)] = &[
    // LIVE DEFECT. Bicarbonate, acid, a sealed 200 mL headspace and a
    // pressure gauge. Swapping the two reagents leaves pH agreeing to four
    // decimals (5.555778 / 5.555338), ionic strength to six (0.497017 both
    // ways) and the gauge to a part in a million — and moves `pe` from
    // 12.780243 to -0.055944, about 760 mV. Neither vessel holds a redox
    // couple: `solution.redox` is `[]` in both. The number is
    // unconstrained, the solver returns whatever its path left behind, and
    // the `--json` contract publishes it as the vessel's pe with nothing
    // to say it means nothing. Same shape as the recorded
    // `contents["OH-"]` defect, found the same way.
    (
        "th-100",
        "solution.pe is path-dependent by 12.84 where no redox couple \
         constrains it, while every other surface agrees",
    ),
    // LIVE DEFECT, small. Calcium chloride and powdered detergent. The
    // inventory's water agrees between the two orders to one part in 4e8
    // (5.5339424445 against 5.5339424570 mol); `solution.solvent_kg`
    // disagrees by one part in 1e4 (0.0997010580 against 0.0996909590 kg),
    // about 10 mg in 100 g. Every molality is divided by that number, and
    // the wire's four significant figures hide the result.
    (
        "aq-023",
        "solution.solvent_kg carries an order-dependent residue that \
         contents[water] does not",
    ),
];

const SCALE_DEPARTURES: &[(&str, &str)] = &[
    // Water, 40 kJ, one minute: the beaker is AT its boiling point, and
    // the doubled one is too. A vessel sitting on a phase boundary
    // amplifies any difference in where exactly it landed, and 1.8% in
    // m[H+] is 0.008 in pH. Not excused — recorded, because a scale
    // departure at a phase boundary is a different claim from one in a
    // homogeneous solution, and the next person should know which they are
    // looking at.
    (
        "aq-102",
        "boiling: both vessels sit on the liquid/vapour boundary, where \
         m[H+] differs by 1.77e-2",
    ),
    // Peroxide eaten by catalase, thirty seconds. 1.245025e-7 against
    // 1.246619e-7 is two units in the last printed place of a
    // four-significant-figure molality, i.e. at the edge of what the wire
    // can express. Recorded rather than absorbed by widening the floor,
    // because widening a floor to make a row green is how a suite stops
    // measuring anything.
    (
        "aq-055",
        "m[H+] differs by 1.28e-3, which is two units in the last printed \
         place of a four-figure molality",
    ),
];

const SOLVENT_DEPARTURES: &[(&str, &str)] = &[
    // "Why do old copper contacts turn green?" — copper, 20 mL of water,
    // oxygen, an hour. Nothing corrodes: the copper is 0.03147326 mol of
    // solid before and after, the oxygen is 0.01 mol of gas before and
    // after, and the ionic strength is 1.006e-7, which is water's own
    // autoprotolysis. Twice the water therefore changes nothing, correctly
    // — and the row cannot answer its own question, which is the same
    // disease as `aq-018` in a different organ.
    (
        "mat-069",
        "the copper never corrodes, so there is nothing dissolved to dilute \
         and nothing undissolved to dissolve",
    ),
    // Starch and amylase at 37 C. Named biological materials with no
    // characterised solution behind them: the ionic strength never rises
    // above the 1e-6 that separates a solution from wet nothing, and no
    // solid phase is reported either.
    (
        "bio-029",
        "starch and amylase are not characterised as a solution, so neither \
         branch of the claim has a referent",
    ),
];

const ABLATION_INERT: &[(&str, &str)] = &[
    // THE FINDING THIS RULE EXISTS FOR. "Can a spoonful of sugar disappear
    // into water?" — water, 10 g of sucrose, stir. Delete the sucrose and
    // NOT ONE readout moves: not pH, not ionic strength, not pe, not the
    // temperature, not the words the scene shows. The bench cannot tell
    // sugar-water from water on any surface it exposes.
    //
    // The row is tagged `substance-gap`, so the gap is known. What was not
    // known is that the row is green anyway: it is a SMOKE prompt with
    // `expected = "computed"`, and it has passed for as long as the corpus
    // has existed, because what the corpus checks is the route the answer
    // came by and not whether the answer depends on the sugar.
    (
        "aq-018",
        "10 g of sucrose in 100 mL of water is indistinguishable from the \
         water on every surface the bench exposes (tagged substance-gap)",
    ),
];

const DOSE_INERT: &[(&str, &str)] = &[
    // The same gap as above, seen from the other side: twice the sucrose is
    // also indistinguishable. Worth keeping separately, because a substance
    // the bench cannot see at all and a substance whose dose it cannot
    // resolve are different failures, and this row happens to be both.
    (
        "aq-018",
        "twice a sucrose the bench cannot see at all is still a sucrose it \
         cannot see",
    ),
];

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

/// **Twice the water has to DO something.** There are exactly two things it
/// can do — dilute what is dissolved, or dissolve more of what is not — and
/// a beaker that answers "nothing changed" fails whichever branch it was
/// entitled to take.
///
/// **This rule is the one the corpus corrected.** Its first draft also held
/// every solute amount fixed, on the reasoning of `perturbation.rs` 6/7:
/// the count you weighed is not negotiable by water. Generated over 259
/// real recipes, that claim was refuted in three of the first seven rows it
/// reached. `aq-006` moved 2% of its antlerite; `aq-057` reshuffled
/// hypochlorous acid against hypochlorite; `aq-107` — 50 g of potassium
/// chloride in 100 mL of water — stopped having any solid at all, because
/// the second 100 mL dissolved it.
///
/// All three are right, and the hand-written case is right too: they are
/// different claims. A *species* amount is not conserved under dilution,
/// because dilution shifts speciation and dissolves solid. What is
/// conserved is the ELEMENT total, and the `--json` contract does not
/// expose it for a named material — there is no formula for `antlerite`,
/// `milk` or `starch` on the wire. Asserting the species amount instead
/// would have been a false claim, mechanically generated 259 times, and
/// finding that out cost three runs.
///
/// **What this establishes:** that the solvent is a real quantity the
/// engine computes intensive properties against, and that the solubility
/// limit is live — a saturated beaker takes the second branch and an
/// undersaturated one takes the first, which is `aq-002`'s question
/// ("does a larger spoonful leave crystals at the bottom?") asked as a
/// perturbation rather than as a route.
///
/// **What it cannot establish:** any value, any functional form, or which
/// branch was the right one for a given beaker. Only that one of them
/// happened.
///
/// **Weakness: low.** There is no path by which an engine satisfies this
/// without recomputing something from the solvent mass. The disjunction is
/// not a loophole: failing both branches means the water went in and
/// nothing about the solution changed.
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

/// **Closing the lid restores order-independence, and that is the proof
/// that the open one losing it is chemistry rather than a bug.**
///
/// The generated `Order` claim fails on exactly two of the thirteen rows it
/// reaches, and both are carbonate systems in an open beaker: `mat-086`
/// (limewater and carbon dioxide) and `mat-124` (bicarbonate and vinegar).
/// Measured on `mat-086` — the same three lines, the two reagents swapped:
///
/// ```text
///                        pH      CaCO3 (mol)   Ca(OH)2 left
///   lime, then CO2     9.898      0.009988        none
///   CO2, then lime    12.452      0.003394       0.004605
/// ```
///
/// Two and a half pH units and a factor of three in the precipitate, from
/// nothing but the order of two bottles. That is not a small residual to be
/// absorbed by a tolerance; it is a different answer to "can carbon dioxide
/// turn limewater cloudy?".
///
/// It is also correct. An open beaker is not a closed system: the carbon
/// dioxide that goes in first has somewhere to go before the lime arrives,
/// and the engine lets it. Seal the same vessel and the two orders agree —
/// pH 9.109228 against 9.108648, and the precipitate to one part in 1e9.
///
/// This test asserts that, and it is a DIFFERENTIAL: the same perturbation
/// (swap the reagents) into two systems (open, sealed), with responses that
/// must differ by a large factor in a stated direction. A global error
/// cannot fake it, because a global error moves both.
///
/// **What this establishes:** that path dependence on this bench is
/// attributable to the boundary rather than to memory in the solver — the
/// question `metamorphic.rs`'s order-independence case cannot ask, because
/// it uses one closed recipe. If the engine ever grows a genuine path
/// memory, this test separates it from the open-vessel effect instead of
/// blaming the atmosphere for it.
///
/// **What it cannot establish:** that the open-vessel answer is right, or
/// that the amount lost to the room is right. It establishes where the
/// dependence comes FROM.
///
/// **Weakness: low.** Nothing here is multiplied through, and the assertion
/// is a ratio between two responses rather than either response.
#[test]
fn closing_the_vessel_restores_order_independence() {
    let orders = |lid: &str, first: &str, second: &str| {
        let script = format!("add v1 water 100mL\n{lid}add v1 {first}\nadd v1 {second}\n");
        let steps = run(&script).unwrap_or_else(|error| panic!("{script}\n{error}"));
        observe(&steps)
    };
    let spread = |lid: &str| {
        let (a, b) = (
            orders(lid, "Ca(OH)2 0.01mol", "CO2 0.01mol"),
            orders(lid, "CO2 0.01mol", "Ca(OH)2 0.01mol"),
        );
        let ph = |picture: &BTreeMap<String, f64>| picture["r.v0.ph"];
        let calcite = |picture: &BTreeMap<String, f64>| {
            picture
                .get("n.v0[CaCO3|solid]")
                .copied()
                .unwrap_or_default()
        };
        (
            (ph(&a) - ph(&b)).abs(),
            relative(calcite(&a), calcite(&b)),
            calcite(&a).max(calcite(&b)),
        )
    };

    let (open_ph, open_calcite, open_most) = spread("");
    let (sealed_ph, sealed_calcite, sealed_most) = spread("seal v1 1L\n");

    assert!(
        open_most > 1e-3 && sealed_most > 1e-3,
        "both vessels have to precipitate something, or there is nothing to \
         compare: {open_most} and {sealed_most} mol"
    );
    assert!(
        open_ph > 1.0,
        "an open beaker loses the CO2 poured in before the lime arrives, so \
         the two orders must land far apart: {open_ph} pH units"
    );
    assert!(
        sealed_ph < 0.01,
        "a sealed vessel has nowhere to lose it, so the two orders must \
         agree: {sealed_ph} pH units"
    );
    assert!(
        open_ph > 100.0 * sealed_ph,
        "the open beaker's path dependence must be orders of magnitude \
         larger than the sealed one's, or it is not the boundary doing it: \
         {open_ph} against {sealed_ph} pH units"
    );
    assert!(
        open_calcite > 0.1,
        "the precipitate must depend on the order too, in the open vessel: \
         {open_calcite} relative"
    );
    assert!(
        sealed_calcite < 1e-6,
        "and must not, in the sealed one: {sealed_calcite} relative"
    );
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

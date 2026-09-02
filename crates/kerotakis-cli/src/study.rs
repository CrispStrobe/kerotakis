//! CAP-2: `kero study` — run one lesson many times, varied over one
//! parameter, and collect what the instruments read. The workbench
//! class's core workflow, and what half the curriculum's practicals
//! are: the model is the same deterministic replay `prewarm` uses, so
//! a study is a fair test by construction — one variable moves, the
//! script stays frozen.
//!
//! Determinism survives parallelism: runs execute on a rayon pool with
//! one engine instance per thread (IPhreeqc instances are per-object),
//! and rows are emitted strictly in run-index order, never in
//! completion order.

use kerotakis_core::script::parse_op;
use kerotakis_core::*;
use rayon::prelude::*;

/// Which operator argument the study varies.
enum Selector {
    /// The amount of the unique `add <vessel> <species> …` line.
    Add {
        vessel: VesselId,
        species: SpeciesId,
    },
    /// The amount of the operator on this 1-based script line.
    Line(usize),
}

/// How the varied quantity sweeps: a deterministic linear grid, or a
/// seeded distribution for Monte Carlo (CAP-8).
enum Sweep {
    Linear { from: f64, to: f64, steps: usize },
    Normal { mean: f64, sd: f64 },
    Uniform { lo: f64, hi: f64 },
}

struct Vary {
    selector: Selector,
    /// The selector exactly as the user wrote it, echoed in every row.
    spoken: String,
    /// The range exactly as written, echoed in the provenance.
    range_spoken: String,
    sweep: Sweep,
}

/// One thing to read after each run.
#[derive(Clone)]
enum Probe {
    Ph(VesselId),
    Temperature(VesselId),
    Mass(VesselId),
    /// Total titrant volume delivered by the last titration in the run.
    TitrantVolume(VesselId),
    /// KID-5: how much of one species the vessel holds, in moles.
    ///
    /// `EXP-34`'s acceptance is that `kero study` sweeps the rusting
    /// conditions, and the only answer worth sweeping for is *how much
    /// rust*. Every existing probe reads an instrument, and none of them
    /// sees a solid: a sealed vessel's mass does not move when its oxygen
    /// becomes iron oxide inside it, and pH barely does. `kero fit` already
    /// takes `amount:SPECIES@vN` for exactly this reason; this is the same
    /// selector on the study side.
    Amount(VesselId, String),
}

impl Probe {
    fn parse(word: &str) -> Result<(String, Probe), String> {
        let (name, vessel) = match word.split_once('@') {
            Some((n, v)) => (n, v),
            None => (word, "v1"),
        };
        let vessel = kerotakis_core::script::parse_vessel(vessel)?;
        let probe = match name {
            "ph" => Probe::Ph(vessel),
            "temp" | "temperature" => Probe::Temperature(vessel),
            "mass" | "balance" => Probe::Mass(vessel),
            "titrant_volume" => Probe::TitrantVolume(vessel),
            other => match other.strip_prefix("amount:") {
                Some(species) if !species.is_empty() => {
                    if kerotakis_core::species::lookup_key(species).is_none() {
                        return Err(kerotakis_core::script::unknown_ingredient(species));
                    }
                    Probe::Amount(vessel, species.to_string())
                }
                _ => {
                    return Err(format!(
                        "unknown probe '{other}' (ph, temp, mass, titrant_volume, \
                         amount:<species>; append @vN for a vessel other than v1)"
                    ))
                }
            },
        };
        Ok((word.to_string(), probe))
    }

    fn unit(&self) -> &'static str {
        match self {
            Probe::Ph(_) => "pH",
            Probe::Temperature(_) => "°C",
            Probe::Mass(_) => "g",
            Probe::TitrantVolume(_) => "L",
            Probe::Amount(..) => "mol",
        }
    }

    /// Read the probe from the finished bench. `None` is an honest
    /// answer (no solution to have a pH, no titration ran) and is
    /// emitted as JSON null / an empty CSV cell, never as a made-up 0.
    fn read(&self, bench: &Bench, events: &[Event]) -> Option<f64> {
        match self {
            Probe::Ph(v) => bench
                .vessel(*v)
                .ok()
                .and_then(|v| v.solution.as_ref())
                .map(|s| s.ph),
            Probe::Temperature(v) => bench.vessel(*v).ok().map(|v| v.temperature.to_celsius()),
            Probe::Mass(v) => bench.vessel(*v).ok().and_then(|v| {
                use kerotakis_core::instrument::InstrumentContract;
                kerotakis_core::instrument::Balance
                    .measure(v)
                    .map(|r| r.value)
            }),
            Probe::TitrantVolume(v) => events.iter().rev().find_map(|e| match e {
                Event::Titrated {
                    vessel,
                    total_volume,
                    ..
                } if vessel == v => Some(total_volume.0),
                _ => None,
            }),
            // Zero is a real reading here, not a missing one: a vessel that
            // made no rust holds no rust, and that is the control arm's
            // whole answer. A vessel that does not exist still reads None.
            Probe::Amount(v, species) => bench.vessel(*v).ok().map(|vessel| {
                vessel
                    .moles_of(&kerotakis_core::species::SpeciesId::new(species))
                    .0
            }),
        }
    }
}

fn parse_two(args: &str, what: &str) -> Result<(f64, f64), String> {
    let (a, b) = args.split_once(',').ok_or(format!(
        "{what} takes two numbers, e.g. {what}(0.01,0.0001)"
    ))?;
    let a: f64 = a.trim().parse().map_err(|_| format!("bad number '{a}'"))?;
    let b: f64 = b.trim().parse().map_err(|_| format!("bad number '{b}'"))?;
    Ok((a, b))
}

fn parse_sweep(range: &str) -> Result<Sweep, String> {
    if let Some(rest) = range
        .strip_prefix("normal(")
        .and_then(|r| r.strip_suffix(')'))
    {
        let (mean, sd) = parse_two(rest, "normal")?;
        if sd <= 0.0 {
            return Err("normal needs a positive standard deviation".into());
        }
        return Ok(Sweep::Normal { mean, sd });
    }
    if let Some(rest) = range
        .strip_prefix("uniform(")
        .and_then(|r| r.strip_suffix(')'))
    {
        let (lo, hi) = parse_two(rest, "uniform")?;
        if hi <= lo {
            return Err("uniform needs lo < hi".into());
        }
        return Ok(Sweep::Uniform { lo, hi });
    }
    let (span, steps) = match range.rsplit_once(':') {
        Some((span, n)) => (
            span,
            n.parse::<usize>()
                .map_err(|_| format!("bad step count '{n}'"))?,
        ),
        None => (range, 11),
    };
    if steps < 2 {
        return Err("a study needs at least 2 steps".into());
    }
    let (from, to) = span
        .split_once("..")
        .ok_or("expected <from>..<to> (e.g. 0.005..0.02) or normal(μ,σ) / uniform(a,b)")?;
    let from: f64 = from.parse().map_err(|_| format!("bad number '{from}'"))?;
    let to: f64 = to.parse().map_err(|_| format!("bad number '{to}'"))?;
    Ok(Sweep::Linear { from, to, steps })
}

fn parse_vary(spec: &str) -> Result<Vary, String> {
    let (sel, range) = spec
        .split_once('=')
        .ok_or("expected <selector>=<from>..<to>[:steps] or <selector>=normal(μ,σ)")?;
    let sweep = parse_sweep(range)?;
    let selector = match sel.split(':').collect::<Vec<_>>().as_slice() {
        ["add", vessel, species] => Selector::Add {
            vessel: kerotakis_core::script::parse_vessel(vessel)?,
            species: SpeciesId::new(species),
        },
        ["line", n] => Selector::Line(
            n.parse::<usize>()
                .map_err(|_| format!("bad line number '{n}'"))?,
        ),
        _ => {
            return Err(format!(
                "unknown selector '{sel}' (add:<vessel>:<species> or line:<N>)"
            ))
        }
    };
    Ok(Vary {
        selector,
        spoken: sel.to_string(),
        range_spoken: range.to_string(),
        sweep,
    })
}

/// The parsed script: operators with the 1-based line they came from.
fn parse_script(text: &str) -> Result<Vec<(usize, Operator)>, String> {
    let mut ops = Vec::new();
    for (i, line) in text.lines().enumerate() {
        match parse_op(line) {
            Ok(Some(op)) => ops.push((i + 1, op)),
            Ok(None) => {}
            Err(e) => return Err(format!("line {}: {e}", i + 1)),
        }
    }
    Ok(ops)
}

/// Find the one operator the selector names; error out loud on zero or
/// several matches — a study that silently picked a line would be
/// varying something the user did not ask for.
fn resolve(vary: &Vary, ops: &[(usize, Operator)]) -> Result<usize, String> {
    match &vary.selector {
        Selector::Add { vessel, species } => {
            let hits: Vec<usize> = ops
                .iter()
                .enumerate()
                .filter(|(_, (_, op))| {
                    matches!(op, Operator::Add { vessel: v, species: s, .. }
                        if v == vessel && s == species)
                })
                .map(|(i, _)| i)
                .collect();
            match hits.as_slice() {
                [one] => Ok(*one),
                [] => Err(format!("no add line matches '{}'", vary.spoken)),
                many => Err(format!(
                    "{} add lines match '{}' (script lines {:?}) — use line:<N>",
                    many.len(),
                    vary.spoken,
                    many.iter().map(|&i| ops[i].0).collect::<Vec<_>>()
                )),
            }
        }
        Selector::Line(n) => ops
            .iter()
            .position(|(line, _)| line == n)
            .ok_or(format!("script line {n} holds no operator"))
            .and_then(|i| match &ops[i].1 {
                Operator::Add { .. } => Ok(i),
                _ => Err(format!(
                    "line {n} is not an add line; only amounts can vary in v1"
                )),
            }),
    }
}

struct Row {
    run: usize,
    value: f64,
    probes: Vec<Option<f64>>,
}

pub fn study_command(args: &[String]) {
    let lab = args
        .iter()
        .find(|a| a.ends_with(".lab"))
        .unwrap_or_else(|| die("kero study: no .lab file given"));
    let vary_spec = flag_value(args, "--vary")
        .unwrap_or_else(|| die("kero study: --vary <selector>=<from>..<to>[:steps] required"));
    let collect_spec = flag_value(args, "--collect")
        .unwrap_or_else(|| die("kero study: --collect <probe>[,…] required"));
    let csv = args.iter().any(|a| a == "--csv");

    let vary = parse_vary(&vary_spec).unwrap_or_else(|e| die(&format!("kero study: {e}")));
    let mc: Option<usize> = flag_value(args, "--mc").map(|n| {
        n.parse()
            .unwrap_or_else(|_| die(&format!("kero study: bad --mc count '{n}'")))
    });
    let seed: Option<u64> = flag_value(args, "--seed").map(|n| {
        n.parse()
            .unwrap_or_else(|_| die(&format!("kero study: bad --seed '{n}'")))
    });
    match (&vary.sweep, mc, seed) {
        (Sweep::Linear { .. }, Some(_), _) => die("kero study: --mc needs a distribution — write \
             --vary <sel>=normal(μ,σ) or uniform(a,b), not a linear range"),
        (Sweep::Normal { .. } | Sweep::Uniform { .. }, None, _) => {
            die("kero study: a distribution needs --mc <N>")
        }
        (Sweep::Normal { .. } | Sweep::Uniform { .. }, Some(_), None) => die(
            "kero study: --mc needs --seed <S> — determinism is the contract, \
             so the seed is spoken, never invented",
        ),
        _ => {}
    }
    let probes: Vec<(String, Probe)> = collect_spec
        .split(',')
        .map(|w| Probe::parse(w.trim()).unwrap_or_else(|e| die(&format!("kero study: {e}"))))
        .collect();

    let text = std::fs::read_to_string(lab)
        .unwrap_or_else(|e| die(&format!("kero study: cannot read {lab}: {e}")));
    let ops = parse_script(&text).unwrap_or_else(|e| die(&format!("kero study: {lab}: {e}")));
    let target = resolve(&vary, &ops).unwrap_or_else(|e| die(&format!("kero study: {e}")));

    // Fail before spawning anything if no engine can exist at all.
    kerotakis_phreeqc::PhreeqcEquilibrator::new()
        .unwrap_or_else(|e| die(&format!("kero study: aqueous engine unavailable: {e}")));

    let values: Vec<f64> = match (&vary.sweep, mc) {
        (Sweep::Linear { from, to, steps }, _) => (0..*steps)
            .map(|i| from + (to - from) * (i as f64) / ((steps - 1) as f64))
            .collect(),
        (Sweep::Normal { mean, sd }, Some(n)) => {
            kerotakis_core::statistics::Experiment::new(seed.expect("checked above"))
                .normal_samples(*mean, *sd, n)
        }
        (Sweep::Uniform { lo, hi }, Some(n)) => {
            kerotakis_core::statistics::Experiment::new(seed.expect("checked above"))
                .uniform_samples(*lo, *hi, n)
        }
        _ => unreachable!("validated above"),
    };

    let mut rows: Vec<Row> = values
        .par_iter()
        .enumerate()
        .map_init(
            || {
                kerotakis_phreeqc::PhreeqcEquilibrator::new()
                    .expect("engine construction succeeded once above")
            },
            |engine, (run, &value)| {
                let mut bench = Bench::new();
                let mut events: Vec<Event> = Vec::new();
                for (i, (_, op)) in ops.iter().enumerate() {
                    let mut op = op.clone();
                    if i == target {
                        if let Operator::Add { moles, .. } = &mut op {
                            *moles = Moles(value);
                        }
                    }
                    // A refused or failed step is part of the result, not
                    // a reason to kill the whole study: the probes read
                    // whatever state the bench honestly reached.
                    if let Ok(mut more) =
                        bench.step_with(op, engine, &kerotakis_safety::ReactiveGroupScreen)
                    {
                        events.append(&mut more);
                    }
                }
                Row {
                    run,
                    value,
                    probes: probes
                        .iter()
                        .map(|(_, p)| p.read(&bench, &events))
                        .collect(),
                }
            },
        )
        .collect();
    rows.sort_by_key(|r| r.run);

    let sweep_said = match (&vary.sweep, mc, seed) {
        (Sweep::Linear { .. }, ..) => format!("{} over {}", vary.spoken, vary.range_spoken),
        (_, Some(n), Some(sd)) => format!(
            "{} ~ {} sampled {n} times (ChaCha20, seed {sd})",
            vary.spoken, vary.range_spoken
        ),
        _ => unreachable!("validated above"),
    };
    // KID-5: the sweep is always in moles, whatever the line said.
    //
    // `*moles = Moles(value)` replaces the parsed amount, so
    // `--vary add:v1:Fe=1..2` on a line reading `add v1 Fe 1g` quietly
    // dosed one *mole* of iron — 55.8 g — and the resulting curve looked
    // like a rate law that had stopped responding. It cost this author an
    // hour of blaming the rate law. The unit is now in the provenance every
    // run carries, and a line written in grams or millilitres says so on
    // stderr before the first replay.
    let swept_line = ops
        .get(target)
        .and_then(|(number, _)| text.lines().nth(number.saturating_sub(1)))
        .unwrap_or_default()
        .trim()
        .to_string();
    if let Some(unit) = swept_line
        .split_whitespace()
        .last()
        .and_then(|amount| ["mL", "L", "g"].into_iter().find(|u| amount.ends_with(u)))
    {
        eprintln!(
            "kero study: '{swept_line}' is written in {unit}, but --vary always \
             sweeps moles — the range {} is read as moles, not {unit}",
            vary.range_spoken
        );
    }
    let provenance = format!(
        "computed replay of {lab}; varied {sweep_said} (in moles); \
         solver: kerotakis PHREEQC stack; probes read from solved state and events"
    );

    let mut out = std::io::stdout().lock();
    use std::io::Write;
    if csv {
        let header: Vec<String> = ["run", "value"]
            .iter()
            .map(|s| s.to_string())
            .chain(probes.iter().map(|(n, _)| n.clone()))
            .collect();
        writeln!(out, "{}", header.join(",")).unwrap();
        for r in &rows {
            let cells: Vec<String> = [r.run.to_string(), format!("{}", r.value)]
                .into_iter()
                .chain(r.probes.iter().map(|p| match p {
                    Some(v) => format!("{v}"),
                    None => String::new(),
                }))
                .collect();
            writeln!(out, "{}", cells.join(",")).unwrap();
        }
        writeln!(out, "# {provenance}").unwrap();
        if mc.is_some() {
            for (pi, (name, _)) in probes.iter().enumerate() {
                if let Some(sm) = summarize(&rows, pi) {
                    writeln!(
                        out,
                        "# {name}: p5={} p50={} p95={} mean={} sd={} (n={})",
                        sm.p5, sm.p50, sm.p95, sm.mean, sm.sd, sm.n
                    )
                    .unwrap();
                }
            }
        }
    } else {
        for r in &rows {
            let mut probe_obj = serde_json::Map::new();
            for ((name, p), v) in probes.iter().zip(&r.probes) {
                probe_obj.insert(
                    name.clone(),
                    serde_json::json!({
                        "value": v,
                        "unit": p.unit(),
                    }),
                );
            }
            let row = serde_json::json!({
                "run": r.run,
                "selector": vary.spoken,
                "value": r.value,
                "probes": probe_obj,
                "provenance": provenance,
            });
            writeln!(out, "{row}").unwrap();
        }
        if mc.is_some() {
            let mut sums = serde_json::Map::new();
            for (pi, (name, _)) in probes.iter().enumerate() {
                if let Some(sm) = summarize(&rows, pi) {
                    sums.insert(
                        name.clone(),
                        serde_json::json!({
                            "p5": sm.p5, "p50": sm.p50, "p95": sm.p95,
                            "mean": sm.mean, "sd": sm.sd, "n": sm.n,
                        }),
                    );
                }
            }
            writeln!(
                out,
                "{}",
                serde_json::json!({ "summary": sums, "provenance": provenance })
            )
            .unwrap();
        }
    }
}

struct Summary {
    p5: f64,
    p50: f64,
    p95: f64,
    mean: f64,
    sd: f64,
    n: usize,
}

/// Percentiles over the non-null readings of one probe. `None` when no
/// run produced a reading — an absent summary, not a fabricated one.
fn summarize(rows: &[Row], probe_index: usize) -> Option<Summary> {
    let vals: Vec<f64> = rows.iter().filter_map(|r| r.probes[probe_index]).collect();
    if vals.is_empty() {
        return None;
    }
    let ps = kerotakis_core::statistics::Experiment::percentiles(&vals, &[5.0, 50.0, 95.0]);
    let (mean, sd) = kerotakis_core::statistics::Experiment::mean_std(&vals);
    Some(Summary {
        p5: ps[0],
        p50: ps[1],
        p95: ps[2],
        mean,
        sd,
        n: vals.len(),
    })
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn die(msg: &str) -> ! {
    eprintln!("{msg}");
    std::process::exit(2);
}

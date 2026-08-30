//! CAP-9: fit one curated rate-law constant by replaying a lesson.
use kerotakis_core::chart::{Axis, Chart, Series};
use kerotakis_core::script::{parse_op, parse_vessel};
use kerotakis_core::*;

#[derive(Clone, Copy)]
struct Datum {
    t: f64,
    y: f64,
}

pub fn fit_command(args: &[String]) {
    let lab = args
        .iter()
        .find(|a| a.ends_with(".lab"))
        .unwrap_or_else(|| die("kero fit: no .lab file given"));
    let selector = flag(args, "--param").unwrap_or_else(|| die("kero fit: --param required"));
    let data_path = flag(args, "--data").unwrap_or_else(|| die("kero fit: --data required"));
    if flag(args, "--loss").as_deref() != Some("sse") {
        die("kero fit: --loss sse required (v1 supports SSE only)");
    }
    let observe = flag(args, "--observe")
        .unwrap_or_else(|| die("kero fit: --observe amount:<species>@vN required"));
    let bounds =
        flag(args, "--bounds").unwrap_or_else(|| die("kero fit: --bounds <lo>..<hi> required"));
    let id = selector
        .strip_prefix("rate:")
        .and_then(|s| s.strip_suffix(":pre_exponential"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| die("kero fit: selector must be rate:<reaction-id>:pre_exponential"));
    let reaction = kerotakis_core::kinetics::lookup(id).unwrap_or_else(|| {
        die(&format!(
            "kero fit: unknown curated kinetic reaction '{id}'"
        ))
    });
    let (species, vessel) = parse_observe(&observe);
    if !reaction
        .stoichiometry
        .iter()
        .any(|term| term.species == species.as_str())
    {
        die(&format!(
            "kero fit: observed species '{species}' is not in the '{id}' rate-law stoichiometry"
        ));
    }
    let (lo, hi) = parse_bounds(&bounds);
    let text = std::fs::read_to_string(lab)
        .unwrap_or_else(|e| die(&format!("kero fit: cannot read {lab}: {e}")));
    let mut ops = Vec::new();
    for (i, line) in text.lines().enumerate() {
        match parse_op(line) {
            Ok(Some(Operator::Wait { .. })) => die(&format!(
                "kero fit: {lab}:{}: setup lessons must not contain wait",
                i + 1
            )),
            Ok(Some(op)) => ops.push(op),
            Ok(None) => {}
            Err(e) => die(&format!("kero fit: {lab}:{}: {e}", i + 1)),
        }
    }
    let data = read_data(&data_path);
    if data.len() < 2 {
        die("kero fit: data needs at least two rows");
    }
    let endpoint_sse = |candidate: f64| {
        predict(&ops, &data, id, &species, vessel, candidate)
            .iter()
            .zip(&data)
            .map(|(prediction, datum)| (datum.y - prediction).powi(2))
            .sum::<f64>()
    };
    let lo_sse = endpoint_sse(lo);
    let hi_sse = endpoint_sse(hi);
    if !lo_sse.is_finite() || !hi_sse.is_finite() {
        die("kero fit: squared residuals overflow; scale or correct the observations");
    }
    if (lo_sse - hi_sse).abs() <= f64::EPSILON * lo_sse.abs().max(hi_sse.abs()).max(1.0) {
        die("kero fit: observations do not identify this parameter over the supplied bounds");
    }
    let mut evals = 0usize;
    let mut objective = |x: f64| {
        evals += 1;
        let sse = predict(&ops, &data, id, &species, vessel, x.exp())
            .iter()
            .zip(&data)
            .map(|(p, d)| (d.y - p).powi(2))
            .sum::<f64>();
        if !sse.is_finite() {
            die("kero fit: squared residuals overflow; scale or correct the observations");
        }
        sse
    };
    let (log_a, sse) = golden(lo.ln(), hi.ln(), &mut objective);
    let fitted = log_a.exp();
    let predictions = predict(&ops, &data, id, &species, vessel, fitted);
    let residuals: Vec<[f64; 2]> = data
        .iter()
        .zip(&predictions)
        .map(|(d, p)| [d.t, d.y - p])
        .collect();
    let zero = vec![
        [data.iter().map(|d| d.t).fold(f64::INFINITY, f64::min), 0.0],
        [
            data.iter().map(|d| d.t).fold(f64::NEG_INFINITY, f64::max),
            0.0,
        ],
    ];
    let at_boundary = (fitted / lo - 1.0).abs() < 1e-6 || (fitted / hi - 1.0).abs() < 1e-6;
    let provenance = format!(
        "residual = observed - predicted; measurements: {data_path}; each prediction: fresh \
         replay of {lab}, then wait(t), with only {selector} overridden; search bounds \
         {lo}..{hi} are user-supplied optimizer bounds, not parameter uncertainty; {}",
        reaction.provenance
    );
    let chart = Chart {
        title: format!("residuals for {selector}"),
        x: Axis {
            label: "time".into(),
            unit: Some("s".into()),
        },
        y: Axis {
            label: format!("observed − predicted {species}"),
            unit: Some("mol".into()),
        },
        series: vec![
            Series::Scatter {
                name: "residual".into(),
                points: residuals,
            },
            Series::Line {
                name: "zero".into(),
                points: zero,
            },
        ],
        provenance: provenance.clone(),
    };
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
      "parameter":{"selector":selector,"fitted_value":fitted,"unit":"rate-law dependent","bounds":[lo,hi],"at_boundary":at_boundary,
        "curated_value":reaction.forward.arrhenius.pre_exponential,"provenance":reaction.provenance,"source_ids":reaction.source_ids,
        "validity":{"note":reaction.validity.note},"uncertainty":{"relative":reaction.uncertainty.relative,"note":reaction.uncertainty.note}},
      "observation":{"selector":observe,"unit":"mol"},"loss":{"name":"sse","value":sse,"n":data.len()},
      "convergence":{"method":"golden_section_log_parameter","iterations":32,"evaluations":evals},"chart":chart,"provenance":provenance
    })).unwrap());
}

fn predict(
    ops: &[Operator],
    data: &[Datum],
    id: &str,
    species: &str,
    vessel: VesselId,
    a: f64,
) -> Vec<f64> {
    kerotakis_core::kinetics::with_pre_exponential_override(id, a, || {
        let mut engine = kerotakis_phreeqc::PhreeqcEquilibrator::new()
            .unwrap_or_else(|e| die(&format!("kero fit: aqueous engine unavailable: {e}")));
        data.iter()
            .map(|d| {
                let mut bench = Bench::new();
                for op in ops
                    .iter()
                    .cloned()
                    .chain(std::iter::once(Operator::Wait { seconds: d.t }))
                {
                    bench
                        .step_with(op, &mut engine, &kerotakis_safety::ReactiveGroupScreen)
                        .unwrap_or_else(|e| die(&format!("kero fit: replay failed: {e}")));
                }
                bench
                    .vessel(vessel)
                    .unwrap_or_else(|e| die(&format!("kero fit: observation vessel: {e}")))
                    .contents
                    .iter()
                    .filter(|p| p.species.0 == species)
                    .map(|p| p.moles.0)
                    .sum()
            })
            .collect()
    })
    .expect("validated override")
}
fn read_data(path: &str) -> Vec<Datum> {
    let mut r = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .unwrap_or_else(|e| die(&format!("kero fit: cannot read {path}: {e}")));
    let h = r.headers().unwrap_or_else(|e| die(&e.to_string())).clone();
    if h.len() != 2 || h.get(0) != Some("t") || h.get(1) != Some("observation") {
        die("kero fit: CSV header must be exactly t,observation");
    }
    r.records()
        .enumerate()
        .map(|(i, row)| {
            let row = row.unwrap_or_else(|e| die(&e.to_string()));
            let num = |j| {
                row[j]
                    .parse::<f64>()
                    .unwrap_or_else(|_| die(&format!("kero fit: {path}:{}: bad number", i + 2)))
            };
            let d = Datum {
                t: num(0),
                y: num(1),
            };
            if !d.t.is_finite() || d.t < 0.0 || !d.y.is_finite() || d.y < 0.0 {
                die("kero fit: observations must be finite and nonnegative");
            }
            d
        })
        .collect()
}
fn parse_observe(s: &str) -> (String, VesselId) {
    let rest = s
        .strip_prefix("amount:")
        .unwrap_or_else(|| die("kero fit: --observe must be amount:<species>@vN"));
    let (sp, v) = rest
        .rsplit_once('@')
        .unwrap_or_else(|| die("kero fit: --observe must include @vN"));
    if sp.is_empty() {
        die("kero fit: empty observed species");
    }
    (
        sp.into(),
        parse_vessel(v).unwrap_or_else(|e| die(&format!("kero fit: {e}"))),
    )
}
fn parse_bounds(s: &str) -> (f64, f64) {
    let (a, b) = s
        .split_once("..")
        .unwrap_or_else(|| die("kero fit: --bounds must be lo..hi"));
    let lo = a
        .parse::<f64>()
        .unwrap_or_else(|_| die("kero fit: bad lower bound"));
    let hi = b
        .parse::<f64>()
        .unwrap_or_else(|_| die("kero fit: bad upper bound"));
    if !lo.is_finite() || !hi.is_finite() || lo <= 0.0 || hi <= lo {
        die("kero fit: bounds must be finite, positive, and lo < hi");
    }
    (lo, hi)
}
fn golden(mut a: f64, mut b: f64, f: &mut impl FnMut(f64) -> f64) -> (f64, f64) {
    let q = (5f64.sqrt() - 1.0) / 2.0;
    let (mut c, mut d) = (b - q * (b - a), a + q * (b - a));
    let (mut fc, mut fd) = (f(c), f(d));
    for _ in 0..32 {
        if fc <= fd {
            b = d;
            d = c;
            fd = fc;
            c = b - q * (b - a);
            fc = f(c)
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + q * (b - a);
            fd = f(d)
        }
    }
    if fc <= fd {
        (c, fc)
    } else {
        (d, fd)
    }
}
fn flag(a: &[String], f: &str) -> Option<String> {
    a.iter()
        .position(|x| x == f)
        .and_then(|i| a.get(i + 1))
        .cloned()
}
fn die(s: &str) -> ! {
    eprintln!("{s}");
    std::process::exit(2)
}

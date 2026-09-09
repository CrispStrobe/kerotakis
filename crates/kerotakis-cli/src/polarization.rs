use kerotakis_core::polarization::{
    fit_polarization_curve, FitOptions, FitParameter, PolarizationModel, PolarizationObservation,
    PotentialReference,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct FitInput {
    model: PolarizationModel,
    parameters: Vec<FitParameter>,
    #[serde(default)]
    options: Option<FitOptions>,
}

pub fn polarization_command(args: &[String]) {
    let operation = args
        .first()
        .map(String::as_str)
        .unwrap_or_else(|| die("expected 'predict' or 'fit'"));
    let model_path = required_flag(args, "--model");
    let data_path = required_flag(args, "--data");
    let model_text = std::fs::read_to_string(&model_path)
        .unwrap_or_else(|error| die(&format!("cannot read {model_path}: {error}")));
    let (model, parameters, options) = match operation {
        "predict" => (
            serde_json::from_str::<PolarizationModel>(&model_text)
                .unwrap_or_else(|error| die(&format!("invalid model JSON: {error}"))),
            None,
            FitOptions::default(),
        ),
        "fit" => {
            let input: FitInput = serde_json::from_str(&model_text)
                .unwrap_or_else(|error| die(&format!("invalid fit JSON: {error}")));
            (
                input.model,
                Some(input.parameters),
                input.options.unwrap_or_default(),
            )
        }
        _ => die("expected 'predict' or 'fit'"),
    };
    let observations = read_observations(args, &data_path, &model);
    match parameters {
        None => {
            let predictions: Vec<_> = observations
                .iter()
                .map(|observation| {
                    model
                        .predict_current_density(observation)
                        .unwrap_or_else(|error| die(&format!("prediction refused: {error:?}")))
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "model": model,
                    "observations": observations,
                    "predictions": predictions,
                }))
                .expect("serializable polarization prediction")
            );
        }
        Some(parameters) => {
            let fit = fit_polarization_curve(&model, &observations, &parameters, options)
                .unwrap_or_else(|error| die(&format!("fit refused: {error:?}")));
            println!(
                "{}",
                serde_json::to_string_pretty(&fit).expect("serializable polarization fit")
            );
        }
    }
}

fn read_observations(
    args: &[String],
    path: &str,
    model: &PolarizationModel,
) -> Vec<PolarizationObservation> {
    let derive_sweep = args
        .iter()
        .any(|argument| argument == "--derive-sweep-rate");
    let allow_time_segments = args
        .iter()
        .any(|argument| argument == "--allow-time-segments");
    let current_sign = match required_flag(args, "--current-sign").as_str() {
        "anodic-positive" => 1.0,
        "cathodic-positive" => -1.0,
        _ => die("--current-sign must be anodic-positive or cathodic-positive"),
    };
    let area = parse_positive(&required_flag(args, "--area-m2"), "--area-m2");
    let temperature = flag(args, "--temperature-k")
        .map(|value| parse_positive(&value, "--temperature-k"))
        .unwrap_or(298.15);
    let reference = match flag(args, "--reference").as_deref() {
        Some("she") => {
            if flag(args, "--reference-offset-v").is_some()
                || flag(args, "--reference-label").is_some()
            {
                die("SHE reference must not also declare an offset or label");
            }
            PotentialReference::StandardHydrogen
        }
        Some(_) => {
            die("--reference currently accepts only 'she'; use an explicit offset otherwise")
        }
        None => PotentialReference::DeclaredOffset {
            label: required_flag(args, "--reference-label"),
            volts_vs_she: parse_finite(
                &required_flag(args, "--reference-offset-v"),
                "--reference-offset-v",
            ),
        },
    };
    let mut observations = match flag(args, "--format").as_deref() {
        None | Some("delimited") => {
            read_delimited(args, path, area, temperature, current_sign, &reference)
        }
        Some("whitespace") => {
            read_whitespace(args, path, area, temperature, current_sign, &reference)
        }
        Some(_) => die("--format must be delimited or whitespace"),
    };
    if derive_sweep {
        derive_sweep_rates(&mut observations, allow_time_segments);
    }
    if model.double_layer_capacitance_f_per_m2 > 0.0
        && !derive_sweep
        && !args
            .iter()
            .any(|argument| argument == "--sweep-rate-column" || argument == "--sweep-rate-index")
    {
        die("a model with double-layer capacitance requires a sweep-rate column/index or --derive-sweep-rate");
    }
    if let Some(value) = flag(args, "--min-abs-sweep-rate") {
        let minimum = parse_positive(&value, "--min-abs-sweep-rate");
        observations.retain(|observation| observation.sweep_rate_v_per_s.abs() >= minimum);
    }
    if let Some(value) = flag(args, "--max-abs-sweep-rate") {
        let maximum = parse_positive(&value, "--max-abs-sweep-rate");
        observations.retain(|observation| observation.sweep_rate_v_per_s.abs() <= maximum);
    }
    if observations.is_empty() {
        die("curve contains no observations after filtering");
    }
    observations
}

fn read_delimited(
    args: &[String],
    path: &str,
    area: f64,
    temperature: f64,
    current_sign: f64,
    reference: &PotentialReference,
) -> Vec<PolarizationObservation> {
    let potential_column = required_flag(args, "--potential-column");
    let current_column = required_flag(args, "--current-column");
    let time_column = flag(args, "--time-column");
    let sweep_column = flag(args, "--sweep-rate-column");
    let delimiter = match flag(args, "--delimiter").as_deref() {
        None | Some("comma") => b',',
        Some("semicolon") => b';',
        Some("tab") => b'\t',
        Some(_) => die("--delimiter must be comma, semicolon, or tab"),
    };
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .trim(csv::Trim::All)
        .from_path(path)
        .unwrap_or_else(|error| die(&format!("cannot read {path}: {error}")));
    let headers = reader
        .headers()
        .unwrap_or_else(|error| die(&format!("cannot read {path} header: {error}")))
        .clone();
    let column = |name: &str| {
        headers
            .iter()
            .position(|header| header == name)
            .unwrap_or_else(|| die(&format!("{path}: missing declared column '{name}'")))
    };
    let potential_index = column(&potential_column);
    let current_index = column(&current_column);
    let time_index = time_column.as_deref().map(column);
    let sweep_index = sweep_column.as_deref().map(column);
    reader
        .records()
        .enumerate()
        .map(|(row_index, record)| {
            let record =
                record.unwrap_or_else(|error| die(&format!("{path}:{}: {error}", row_index + 2)));
            let number = |index: usize, name: &str| {
                record
                    .get(index)
                    .and_then(|value| value.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .unwrap_or_else(|| {
                        die(&format!(
                            "{path}:{}: '{name}' must be a finite number",
                            row_index + 2
                        ))
                    })
            };
            PolarizationObservation {
                measured_potential_v: number(potential_index, &potential_column),
                measured_current_a: current_sign * number(current_index, &current_column),
                geometric_area_m2: area,
                temperature_k: temperature,
                reference: reference.clone(),
                sweep_rate_v_per_s: sweep_index
                    .map(|index| number(index, sweep_column.as_deref().unwrap()))
                    .unwrap_or(0.0),
                time_s: time_index.map(|index| number(index, time_column.as_deref().unwrap())),
            }
        })
        .collect()
}

fn read_whitespace(
    args: &[String],
    path: &str,
    area: f64,
    temperature: f64,
    current_sign: f64,
    reference: &PotentialReference,
) -> Vec<PolarizationObservation> {
    let potential_index = parse_index(
        &required_flag(args, "--potential-index"),
        "--potential-index",
    );
    let current_index = parse_index(&required_flag(args, "--current-index"), "--current-index");
    let time_index = flag(args, "--time-index").map(|value| parse_index(&value, "--time-index"));
    let sweep_index =
        flag(args, "--sweep-rate-index").map(|value| parse_index(&value, "--sweep-rate-index"));
    let skip_lines = flag(args, "--skip-lines")
        .map(|value| parse_index(&value, "--skip-lines"))
        .unwrap_or(0);
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| die(&format!("cannot read {path}: {error}")));
    if let Some(value) = flag(args, "--header-line") {
        let header_line = parse_index(&value, "--header-line");
        let header: Vec<_> = text
            .lines()
            .nth(header_line)
            .unwrap_or_else(|| die("--header-line is outside the input"))
            .split_whitespace()
            .collect();
        validate_header_token(
            &header,
            potential_index,
            &required_flag(args, "--potential-header-token"),
            "potential",
        );
        validate_header_token(
            &header,
            current_index,
            &required_flag(args, "--current-header-token"),
            "current",
        );
    }
    text.lines()
        .skip(skip_lines)
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(row, line)| {
            let fields: Vec<_> = line.split_whitespace().collect();
            let number = |index: usize, name: &str| {
                fields
                    .get(index)
                    .and_then(|value| value.parse::<f64>().ok())
                    .filter(|value| value.is_finite())
                    .unwrap_or_else(|| {
                        die(&format!(
                            "{path}:{}: zero-based {name} {index} must select a finite number",
                            skip_lines + row + 1
                        ))
                    })
            };
            PolarizationObservation {
                measured_potential_v: number(potential_index, "--potential-index"),
                measured_current_a: current_sign * number(current_index, "--current-index"),
                geometric_area_m2: area,
                temperature_k: temperature,
                reference: reference.clone(),
                sweep_rate_v_per_s: sweep_index
                    .map(|index| number(index, "--sweep-rate-index"))
                    .unwrap_or(0.0),
                time_s: time_index.map(|index| number(index, "--time-index")),
            }
        })
        .collect()
}

fn validate_header_token(header: &[&str], index: usize, expected: &str, quantity: &str) {
    if header.get(index).copied() != Some(expected) {
        die(&format!(
            "declared {quantity} index {index} does not select header token '{expected}'"
        ));
    }
}

fn derive_sweep_rates(observations: &mut [PolarizationObservation], allow_time_segments: bool) {
    if observations.len() < 2
        || observations
            .iter()
            .any(|observation| observation.time_s.is_none())
    {
        die("--derive-sweep-rate requires at least two rows and an explicit time column/index");
    }
    let rates: Vec<f64> = (0..observations.len())
        .map(|index| {
            let (left, right) = if index == 0 {
                (&observations[0], &observations[1])
            } else {
                (&observations[index - 1], &observations[index])
            };
            let dt = right.time_s.unwrap() - left.time_s.unwrap();
            if !dt.is_finite() || dt <= 0.0 {
                if allow_time_segments {
                    return 0.0;
                }
                die("time must increase strictly when deriving sweep rate; pass --allow-time-segments only for declared acquisition seams");
            }
            (right.measured_potential_v - left.measured_potential_v) / dt
        })
        .collect();
    for (observation, rate) in observations.iter_mut().zip(rates) {
        observation.sweep_rate_v_per_s = rate;
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|argument| argument == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn required_flag(args: &[String], name: &str) -> String {
    flag(args, name).unwrap_or_else(|| die(&format!("{name} is required")))
}

fn parse_finite(value: &str, name: &str) -> f64 {
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .unwrap_or_else(|| die(&format!("{name} must be finite")))
}

fn parse_positive(value: &str, name: &str) -> f64 {
    let number = parse_finite(value, name);
    if number <= 0.0 {
        die(&format!("{name} must be positive"));
    }
    number
}

fn parse_index(value: &str, name: &str) -> usize {
    value
        .parse::<usize>()
        .unwrap_or_else(|_| die(&format!("{name} must be a zero-based non-negative integer")))
}

fn die(message: &str) -> ! {
    eprintln!("kero polarization: {message}");
    std::process::exit(2)
}

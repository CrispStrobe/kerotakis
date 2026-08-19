//! Checking the numbers that live in sentences.
//!
//! `codex lint` replays every entry and verifies its events, its pH and its
//! temperature. It does not read the prose — and the prose is full of
//! numbers. When the hydrogen-peroxide rate constant was recalibrated the
//! lint stayed green while five entries went on quoting half-lives and
//! extents the engine no longer produced. Ranges were verified; sentences
//! were not, and a sentence is what the learner actually reads.
//!
//! This closes the gap the only way that does not require authors to write
//! every number twice: pull the numbers back *out* of the prose, and ask
//! whether the engine produced anything like them.
//!
//! **It is deliberately advisory.** Plenty of numbers in a good entry come
//! from somewhere other than this run — activation energies from the
//! literature, a molar mass, a ratio, a figure quoted from a different
//! experiment to compare against. Flagging those as errors would train
//! authors to strip real content out of their writing, which is the
//! opposite of what is wanted. So the outcome is a list to look at, in the
//! same spirit as the equation and prediction audits: a work list, not a
//! pass mark. What it is good at is the thing it was built for — telling
//! you which sentences went stale when a constant moved.

use crate::Entry;

/// The units worth chasing: the ones the bench actually reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Moles,
    Celsius,
    Grams,
    Seconds,
    Ph,
}

impl Unit {
    fn label(self) -> &'static str {
        match self {
            Unit::Moles => "mol",
            Unit::Celsius => "°C",
            Unit::Grams => "g",
            Unit::Seconds => "s",
            Unit::Ph => "pH",
        }
    }
}

/// Everything this replay produced, by unit, to check prose against.
#[derive(Debug, Clone, Default)]
pub struct EngineValues {
    pub moles: Vec<f64>,
    pub celsius: Vec<f64>,
    pub grams: Vec<f64>,
    pub seconds: Vec<f64>,
    pub ph: Vec<f64>,
}

impl EngineValues {
    fn bucket(&self, unit: Unit) -> &[f64] {
        match unit {
            Unit::Moles => &self.moles,
            Unit::Celsius => &self.celsius,
            Unit::Grams => &self.grams,
            Unit::Seconds => &self.seconds,
            Unit::Ph => &self.ph,
        }
    }

    /// Whether the engine produced something this number could be quoting.
    ///
    /// Generous on purpose. Prose rounds — "about 90 s", "0.0499 mol" for
    /// 0.049938 — and an author may quote a derived figure such as a
    /// difference or a doubled amount. The tolerance is therefore 2%, and
    /// values are also matched at twice and half the engine's, because
    /// extent and amount-consumed differ by a stoichiometric coefficient
    /// and both are legitimately quotable.
    fn supports(&self, value: f64, unit: Unit, decimals: u32) -> bool {
        let v = value.abs();
        // Half a unit in the last place the author wrote: the interval any
        // correctly-rounded quotation of an engine value must fall in.
        let rounding = 0.5 * 10f64.powi(-(decimals as i32));
        self.bucket(unit).iter().any(|e| {
            let e = e.abs();
            [1.0, 2.0, 0.5].iter().any(|k| {
                let target = e * k;
                let slack = rounding.max(0.02 * target).max(1e-9);
                (v - target).abs() <= slack
            })
        })
    }
}

/// A number found in prose, with what it was measuring.
#[derive(Debug, Clone, PartialEq)]
pub struct Quoted {
    pub value: f64,
    pub unit: Unit,
    /// Decimal places the author actually wrote. Prose rounds, so a number
    /// is checked at the precision it was quoted to: "near pH 1.6" is a
    /// correct report of 1.64, and a fixed percentage tolerance calls it
    /// stale.
    pub decimals: u32,
    /// A short window of the sentence, so a report can be acted on.
    pub context: String,
}

fn superscript_digit(c: char) -> Option<char> {
    Some(match c {
        '⁰' => '0',
        '¹' => '1',
        '²' => '2',
        '³' => '3',
        '⁴' => '4',
        '⁵' => '5',
        '⁶' => '6',
        '⁷' => '7',
        '⁸' => '8',
        '⁹' => '9',
        _ => return None,
    })
}

/// Pull every unit-carrying number out of a passage of prose.
///
/// Handles the two ways a chemist writes a small number: `4.06e-7` and
/// `4.06 × 10⁻⁷`. The second is the one that appears in the codex, and
/// missing it would have meant the checker quietly passed the entries most
/// likely to be wrong.
pub fn quoted_numbers(text: &str) -> Vec<Quoted> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            // "pH 7.05" puts the unit first.
            if chars[i] == 'p' && chars.get(i + 1) == Some(&'H') {
                let rest: String = chars[i + 2..].iter().take(12).collect();
                if let Some(v) = leading_number(rest.trim_start()) {
                    let decimals = rest
                        .trim_start()
                        .split_once('.')
                        .map(|(_, f)| f.chars().take_while(|c| c.is_ascii_digit()).count() as u32)
                        .unwrap_or(0);
                    out.push(Quoted {
                        value: v,
                        unit: Unit::Ph,
                        decimals,
                        context: window(&chars, i),
                    });
                }
            }
            i += 1;
            continue;
        }
        // A number, possibly with a decimal part.
        let start = i;
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
            i += 1;
        }
        let mantissa: String = chars[start..i].iter().collect();
        let mantissa = mantissa.trim_end_matches('.');
        let Ok(mut value) = mantissa.parse::<f64>() else {
            continue;
        };
        let mut decimals = mantissa
            .split_once('.')
            .map(|(_, frac)| frac.len() as u32)
            .unwrap_or(0);
        // Optional × 10^n, in either notation.
        let mut j = i;
        while j < chars.len() && chars[j] == ' ' {
            j += 1;
        }
        if chars.get(j) == Some(&'×') || chars.get(j) == Some(&'*') {
            let mut k = j + 1;
            while k < chars.len() && chars[k] == ' ' {
                k += 1;
            }
            if chars.get(k) == Some(&'1') && chars.get(k + 1) == Some(&'0') {
                let mut e = k + 2;
                let mut sign = 1.0;
                let mut digits = String::new();
                if chars.get(e) == Some(&'⁻') || chars.get(e) == Some(&'-') {
                    sign = -1.0;
                    e += 1;
                }
                while e < chars.len() {
                    match superscript_digit(chars[e]) {
                        Some(d) => {
                            digits.push(d);
                            e += 1;
                        }
                        None if chars[e].is_ascii_digit() => {
                            digits.push(chars[e]);
                            e += 1;
                        }
                        None => break,
                    }
                }
                if let Ok(exp) = digits.parse::<f64>() {
                    value *= 10f64.powf(sign * exp);
                    // The exponent moves the last written place with it.
                    decimals = (decimals as i32 - (sign * exp) as i32).max(0) as u32;
                    i = e;
                }
            }
        }
        // Now the unit, which must be a standalone token.
        let mut u = i;
        while u < chars.len() && chars[u] == ' ' {
            u += 1;
        }
        let tail: String = chars[u..].iter().take(4).collect();
        let unit = if tail.starts_with("mol") && !tail.starts_with("mol/") {
            Some(Unit::Moles)
        } else if tail.starts_with("°C") {
            Some(Unit::Celsius)
        } else if starts_word(&tail, "g") {
            Some(Unit::Grams)
        } else if starts_word(&tail, "s") {
            Some(Unit::Seconds)
        } else {
            None
        };
        if let Some(unit) = unit {
            out.push(Quoted {
                value,
                unit,
                decimals,
                context: window(&chars, start),
            });
        }
    }
    out
}

/// A unit token has to end there: "g" in "0.5 g" is grams, "g" in "0.5
/// grams of gas" is still grams, but "s" in "30 seconds" is seconds while
/// the "s" of "40 species" is not a unit at all.
fn starts_word(tail: &str, unit: &str) -> bool {
    let Some(rest) = tail.strip_prefix(unit) else {
        return false;
    };
    match rest.chars().next() {
        None => true,
        Some(c) => !c.is_alphanumeric() || rest.starts_with("econd") || rest.starts_with("ram"),
    }
}

fn leading_number(s: &str) -> Option<f64> {
    let digits: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    digits.trim_end_matches('.').parse().ok()
}

fn window(chars: &[char], at: usize) -> String {
    let from = at.saturating_sub(28);
    let to = (at + 30).min(chars.len());
    let text: String = chars[from..to].iter().collect();
    text.replace('\n', " ").trim().to_string()
}

/// Every prose number in this entry that the replay does not account for.
pub fn unsupported(entry: &Entry, values: &EngineValues) -> Vec<Quoted> {
    let mut text = String::new();
    for level in entry.registers.0.values() {
        text.push_str(level);
        text.push('\n');
    }
    if let Some(p) = &entry.expect.predict {
        if let Some(m) = &p.misconception {
            text.push_str(m);
            text.push('\n');
        }
        for d in &p.diagnosis {
            text.push_str(&d.reveals);
            text.push('\n');
            if let Some(n) = &d.next {
                text.push_str(n);
                text.push('\n');
            }
        }
    }
    quoted_numbers(&text)
        .into_iter()
        .filter(|q| !values.supports(q.value, q.unit, q.decimals))
        .collect()
}

impl Quoted {
    pub fn describe(&self) -> String {
        format!(
            "{} {} — \"…{}…\"",
            self.value,
            self.unit.label(),
            self.context
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_numbers_with_units() {
        let q = quoted_numbers("consumes 0.0501 mol and warms to 44.9 °C over 90 s");
        assert_eq!(q.len(), 3, "{q:?}");
        assert_eq!(q[0].unit, Unit::Moles);
        assert!((q[0].value - 0.0501).abs() < 1e-9);
        assert_eq!(q[1].unit, Unit::Celsius);
        assert_eq!(q[2].unit, Unit::Seconds);
        assert!((q[2].value - 90.0).abs() < 1e-9);
    }

    #[test]
    fn scientific_notation_the_way_chemists_write_it() {
        // The form that actually appears in the codex. Missing it would
        // have meant passing exactly the entries most likely to be stale.
        let q = quoted_numbers("the bench reports 5.0000 × 10⁻² mol of reaction");
        assert_eq!(q.len(), 1, "{q:?}");
        assert!((q[0].value - 0.05).abs() < 1e-12, "{:?}", q[0]);
    }

    #[test]
    fn ph_is_written_unit_first() {
        let q = quoted_numbers("the meter reads pH 1.75 afterwards");
        assert_eq!(q.len(), 1, "{q:?}");
        assert_eq!(q[0].unit, Unit::Ph);
        assert!((q[0].value - 1.75).abs() < 1e-9);
    }

    #[test]
    fn a_letter_that_is_not_a_unit_is_not_a_unit() {
        // "species" must not be read as seconds, and kJ/mol is not moles.
        let q = quoted_numbers("75 kJ/mol across 40 species and 0.5 mol/kgw");
        assert!(q.is_empty(), "{q:?}");
    }

    #[test]
    fn a_number_the_engine_produced_is_supported() {
        let v = EngineValues {
            moles: vec![0.049938],
            ..Default::default()
        };
        assert!(v.supports(0.0499, Unit::Moles, 4), "rounding is fine");
        // Twice the extent is the amount consumed, and both get quoted.
        assert!(v.supports(0.0999, Unit::Moles, 4));
        assert!(!v.supports(0.31, Unit::Moles, 2), "unrelated number");
    }

    #[test]
    fn an_empty_bucket_supports_nothing() {
        let v = EngineValues::default();
        assert!(!v.supports(1.0, Unit::Moles, 1));
    }

    #[test]
    fn a_rounded_quotation_is_still_a_correct_one() {
        // "near pH 1.6" is an honest report of a computed 1.64, and a flat
        // percentage tolerance calls it stale. Precision is the author's.
        let v = EngineValues {
            ph: vec![1.64],
            ..Default::default()
        };
        assert!(v.supports(1.6, Unit::Ph, 1));
        assert!(!v.supports(1.6, Unit::Ph, 2), "1.60 would be a real claim");
    }
}

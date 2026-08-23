//! Formulas, equations, and whether they balance.
//!
//! The codex carries an `equation` on every entry, documented as "balanced
//! equation", and until now nothing checked that claim — the lint verified
//! only that the string was not empty. Sixty-six entries, every one of them
//! asserting a balanced equation on the authority of whoever typed it.
//!
//! Balancing is not a lookup, it is linear algebra: the coefficients that
//! balance a reaction are the null space of the element-count matrix. So we
//! can *compute* the answer rather than trust the string, and the same
//! machinery turns "balance this equation" into an exercise the engine can
//! mark.
//!
//! Two design points worth stating.
//!
//! **Charge is balanced too.** `Ag⁺ + Cl⁻ → AgCl` is only right because the
//! charges cancel, and an ionic equation that conserves atoms while
//! inventing charge is a common student error. Both are checked.
//!
//! **What cannot be parsed is counted, never skipped.** Some entries use
//! this field for prose — "CH₃COOH / CH₃COO⁻ buffer" — and a checker that
//! quietly ignores what it does not understand reports a clean bill of
//! health it has not earned. That is the same silent-filter defect found
//! elsewhere in this engine, so the outcome here is three-valued: balanced,
//! unbalanced, or *not verifiable*, with the last one visible.

use num_rational::Rational64;
use std::collections::BTreeMap;

/// The element symbols. A parser that accepts any capital letter will
/// happily "balance" `A + B → C`, so a symbol that is not an element means
/// this string is not a formula — which is exactly the answer wanted for
/// the prose that also lives in the codex's equation field.
const ELEMENTS: &[&str] = &[
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

/// Whether this is a real element symbol.
pub fn is_element(symbol: &str) -> bool {
    ELEMENTS.contains(&symbol)
}

/// A parsed formula: element counts and net charge.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Formula {
    pub counts: BTreeMap<String, f64>,
    pub charge: f64,
}

/// One side of an equation: coefficient-weighted formulas.
#[derive(Debug, Clone, Default)]
pub struct Side(pub Vec<(f64, Formula)>);

/// A parsed chemical equation.
#[derive(Debug, Clone)]
pub struct Equation {
    pub lhs: Side,
    pub rhs: Side,
    /// True for ⇌ — still balanced, but worth keeping for rendering.
    pub reversible: bool,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("no reaction arrow")]
    NoArrow,
    #[error("empty side")]
    EmptySide,
    #[error("'{0}' is not a formula")]
    NotAFormula(String),
}

impl Side {
    fn totals(&self) -> (BTreeMap<String, f64>, f64) {
        let mut counts: BTreeMap<String, f64> = BTreeMap::new();
        let mut charge = 0.0;
        for (n, f) in &self.0 {
            for (el, c) in &f.counts {
                *counts.entry(el.clone()).or_insert(0.0) += n * c;
            }
            charge += n * f.charge;
        }
        (counts, charge)
    }
}

impl Equation {
    /// Elements that do not balance, as (element, right − left).
    pub fn element_imbalance(&self) -> Vec<(String, f64)> {
        let (l, _) = self.lhs.totals();
        let (r, _) = self.rhs.totals();
        let mut keys: Vec<&String> = l.keys().chain(r.keys()).collect();
        keys.sort();
        keys.dedup();
        keys.into_iter()
            .filter_map(|k| {
                let d = r.get(k).copied().unwrap_or(0.0) - l.get(k).copied().unwrap_or(0.0);
                (d.abs() > 1e-6).then(|| (k.clone(), d))
            })
            .collect()
    }

    /// Net charge on the right minus the left.
    pub fn charge_imbalance(&self) -> f64 {
        self.rhs.totals().1 - self.lhs.totals().1
    }

    pub fn is_balanced(&self) -> bool {
        self.element_imbalance().is_empty() && self.charge_imbalance().abs() < 1e-6
    }
}

/// Normalise the typography chemists actually write: Unicode subscripts and
/// superscripts, the various arrows, and the decorations (↓ ↑) and state
/// labels that carry no stoichiometry.
fn normalise(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut supers = String::new();
    let flush = |supers: &mut String, out: &mut String| {
        if !supers.is_empty() {
            out.push('^');
            out.push_str(supers);
            supers.clear();
        }
    };
    for ch in s.chars() {
        // Superscripts form the charge; they are collected as a run so that
        // "²⁻" becomes one charge of −2 rather than two separate marks.
        let sup = match ch {
            '⁰' => Some('0'),
            '¹' => Some('1'),
            '²' => Some('2'),
            '³' => Some('3'),
            '⁴' => Some('4'),
            '⁵' => Some('5'),
            '⁶' => Some('6'),
            '⁷' => Some('7'),
            '⁸' => Some('8'),
            '⁹' => Some('9'),
            '⁺' => Some('+'),
            '⁻' => Some('-'),
            _ => None,
        };
        if let Some(c) = sup {
            supers.push(c);
            continue;
        }
        flush(&mut supers, &mut out);
        let sub = match ch {
            '₀' => Some('0'),
            '₁' => Some('1'),
            '₂' => Some('2'),
            '₃' => Some('3'),
            '₄' => Some('4'),
            '₅' => Some('5'),
            '₆' => Some('6'),
            '₇' => Some('7'),
            '₈' => Some('8'),
            '₉' => Some('9'),
            _ => None,
        };
        match sub {
            Some(c) => out.push(c),
            // Precipitate and gas arrows are notation, not stoichiometry.
            None if ch == '↓' || ch == '↑' => {}
            None => out.push(ch),
        }
    }
    flush(&mut supers, &mut out);
    out
}

/// Strip the state labels that carry no stoichiometry. Done before group
/// parsing, or `(aq)` would be read as a parenthesised group of the
/// elements "a" and "q".
fn strip_states(s: &str) -> String {
    let mut out = s.to_string();
    for state in ["(aq)", "(s)", "(l)", "(g)", "(cr)", "(am)", "(soln)"] {
        out = out.replace(state, "");
    }
    out
}

/// Parse a formula: element groups, parentheses, hydrate dots, charge.
/// Which language a formula is written in.
///
/// The lexer is shared; the dialects differ in exactly one judgement:
/// what counts as an element. `Textbook` admits the periodic table and
/// nothing else — a learner typing "Unicorn2O" deserves a refusal.
/// `PhreeqcMaster` additionally admits PHREEQC's pseudo-elements: the
/// databases define organic ligands (Cyanide, Edta, Butylamine…),
/// exchanger sites (X) and DOM fragments (Hdg, Mtg…) as master-species
/// symbols, and a database parser that refused them would go blind to
/// minteq.v4. The 2026-08-23 differential over all 641 shipped database
/// formulas found the two old independent parsers disagreed on exactly
/// this class and on nothing numeric — which is why this is one parser
/// with a switch, not two parsers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaDialect {
    Textbook,
    PhreeqcMaster,
}

pub fn parse_formula(input: &str) -> Result<Formula, ParseError> {
    parse_formula_with(input, FormulaDialect::Textbook)
}

pub fn parse_formula_with(input: &str, dialect: FormulaDialect) -> Result<Formula, ParseError> {
    let cleaned = strip_states(&normalise(input));
    let body = cleaned.trim();
    if body.is_empty() {
        return Err(ParseError::NotAFormula(input.to_string()));
    }
    // A hydrate dot binds two formulas: CaSO4·2H2O. It is only a hydrate
    // dot when it sits flush against the formula; spaced, the same
    // character is used in this codex as a prose separator.
    if let Some((a, b)) = body.split_once('·').or_else(|| body.split_once('*')) {
        if !a.ends_with(' ') && !b.starts_with(' ') {
            let mut left = parse_formula_with(a, dialect)?;
            let (mult, rest) = leading_number(b);
            let right = parse_formula_with(rest, dialect)?;
            for (el, n) in right.counts {
                *left.counts.entry(el).or_insert(0.0) += n * mult;
            }
            left.charge += right.charge * mult;
            return Ok(left);
        }
    }
    let (body, charge) = split_charge(body);
    let counts = parse_groups(&body, input, dialect)?;
    if counts.is_empty() {
        return Err(ParseError::NotAFormula(input.to_string()));
    }
    Ok(Formula { counts, charge })
}

/// Split a trailing charge from a formula body.
///
/// Three notations have to coexist, and two of them genuinely conflict.
/// PHREEQC and our registry put the magnitude *after* the sign — `Ag+`,
/// `SO4-2`, `Cu+2`. Chemists superscript it, which normalisation turns
/// into `SO4^2-`. Both are unambiguous. The third, `Ca2+`, is not: the
/// same characters appear in `MnO4-`, where the 4 is a subscript and the
/// charge is −1.
///
/// **Digits before a trailing sign are read as a subscript.** That makes
/// `MnO4-`, `NO3-`, `HCO3-`, `CrO4-2` and every other oxyanion right, at
/// the cost of reading `Ca2+` as "Ca₂ with charge +1". Write `Ca+2`,
/// `Ca²⁺` or `Ca++` for that, all three of which work. The alternative
/// convention was tried first and quietly broke permanganate, which is a
/// much more common thing to write than a bare `Ca2+`.
fn split_charge(s: &str) -> (String, f64) {
    if let Some((body, sup)) = s.split_once('^') {
        return (body.to_string(), read_charge(sup));
    }
    let t = s.trim_end();
    let chars: Vec<char> = t.chars().collect();

    // Shape 1: ends with one or more signs. Magnitude is how many, so that
    // `Ca++` is a charge of two; any digits before them stay subscripts.
    let signs = chars
        .iter()
        .rev()
        .take_while(|c| **c == '+' || **c == '-')
        .count();
    if signs > 0 {
        let body: String = chars[..chars.len() - signs].iter().collect();
        let spec: String = chars[chars.len() - signs..].iter().collect();
        return (body, read_charge(&spec));
    }

    // Shape 2: a sign followed only by digits — the PHREEQC form.
    if let Some(pos) = t.rfind(['+', '-']) {
        if !t[pos + 1..].is_empty() && t[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
            return (t[..pos].to_string(), read_charge(&t[pos..]));
        }
    }
    (t.to_string(), 0.0)
}

fn read_charge(spec: &str) -> f64 {
    let sign = if spec.contains('-') { -1.0 } else { 1.0 };
    let digits: String = spec.chars().filter(|c| c.is_ascii_digit()).collect();
    let magnitude = if digits.is_empty() {
        // `Ca++` is a charge of two, written by repetition.
        spec.chars().filter(|c| *c == '+' || *c == '-').count() as f64
    } else {
        digits.parse().unwrap_or(1.0)
    };
    sign * magnitude.max(1.0)
}

fn leading_number(s: &str) -> (f64, &str) {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        (1.0, s)
    } else {
        (digits.parse().unwrap_or(1.0), &s[digits.len()..])
    }
}

fn parse_groups(
    s: &str,
    original: &str,
    dialect: FormulaDialect,
) -> Result<BTreeMap<String, f64>, ParseError> {
    let chars: Vec<char> = s.chars().collect();
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '(' | '[' => {
                let open = chars[i];
                let close = if open == '(' { ')' } else { ']' };
                let mut depth = 1;
                let mut j = i + 1;
                while j < chars.len() && depth > 0 {
                    if chars[j] == open {
                        depth += 1;
                    } else if chars[j] == close {
                        depth -= 1;
                    }
                    j += 1;
                }
                if depth != 0 {
                    return Err(ParseError::NotAFormula(original.to_string()));
                }
                let inner: String = chars[i + 1..j - 1].iter().collect();
                let digits: String = chars[j..]
                    .iter()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                let mult: f64 = if digits.is_empty() {
                    1.0
                } else {
                    digits.parse().unwrap_or(1.0)
                };
                for (el, n) in parse_groups(&inner, original, dialect)? {
                    *counts.entry(el).or_insert(0.0) += n * mult;
                }
                i = j + digits.len();
            }
            c if c.is_ascii_uppercase() => {
                let mut sym = c.to_string();
                i += 1;
                while i < chars.len() && chars[i].is_ascii_lowercase() {
                    sym.push(chars[i]);
                    i += 1;
                }
                let digits: String = chars[i..]
                    .iter()
                    .take_while(|c| c.is_ascii_digit() || **c == '.')
                    .collect();
                let n: f64 = if digits.is_empty() {
                    1.0
                } else {
                    digits.parse().unwrap_or(1.0)
                };
                i += digits.len();
                if dialect == FormulaDialect::Textbook && !is_element(&sym) {
                    return Err(ParseError::NotAFormula(original.to_string()));
                }
                *counts.entry(sym).or_insert(0.0) += n;
            }
            // Anything else — prose, punctuation, an unexpected symbol —
            // means this is not a formula, and saying so is the point.
            _ => return Err(ParseError::NotAFormula(original.to_string())),
        }
    }
    Ok(counts)
}

/// The reaction arrows used in practice, longest first so that `<=>` is not
/// mistaken for `<`.
const ARROWS: &[&str] = &["⇌", "⟶", "→", "->", "<=>", "=>", "⇄", "↔"];

fn parse_side(s: &str) -> Result<Side, ParseError> {
    let mut terms = Vec::new();
    // A spaced plus, because a bare one is also the charge on `Ag+`.
    for raw in s.split(" + ") {
        let term = raw.trim();
        if term.is_empty() {
            return Err(ParseError::EmptySide);
        }
        // A leading stoichiometric coefficient, integer or decimal.
        let digits: String = term
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let (coefficient, rest) = if digits.is_empty() || digits.len() == term.len() {
            (1.0, term)
        } else {
            (
                digits.parse().unwrap_or(1.0),
                term[digits.len()..].trim_start(),
            )
        };
        terms.push((coefficient, parse_formula(rest)?));
    }
    if terms.is_empty() {
        return Err(ParseError::EmptySide);
    }
    Ok(Side(terms))
}

/// Parse a full equation. Anything that is not one — prose, arithmetic, a
/// description — comes back as an error rather than as a pass.
pub fn parse_equation(input: &str) -> Result<Equation, ParseError> {
    let text = normalise(input);
    let Some(arrow) = ARROWS.iter().find(|a| text.contains(**a)) else {
        return Err(ParseError::NoArrow);
    };
    let reversible = *arrow == "⇌" || *arrow == "⇄" || *arrow == "<=>" || *arrow == "↔";
    let (l, r) = text.split_once(arrow).expect("arrow present");
    Ok(Equation {
        lhs: parse_side(l)?,
        rhs: parse_side(r)?,
        reversible,
    })
}

/// The result of balancing a reaction skeleton.
#[derive(Debug, Clone, PartialEq)]
pub enum BalanceResult {
    /// A unique set of smallest positive integer coefficients.
    Unique(Vec<i64>),
    /// The skeleton admits more than one independent reaction. The result
    /// is one smallest-integer particular solution plus the remaining
    /// null-space basis vectors (each also in smallest integers).
    Family {
        particular: Vec<i64>,
        basis: Vec<Vec<i64>>,
    },
}

/// Balance a skeleton reaction: given formulas on each side, find the
/// smallest positive integer coefficients that conserve every element and
/// the charge.
///
/// This is the null space of the composition matrix, computed with exact
/// rational arithmetic (`Rational64`) so integer families never pass
/// through floating point. A balanced reaction is a vector **n** with
/// A·**n** = 0, where A has one row per element (plus one for charge) and
/// one column per species, right-hand species entering negated.
///
/// Returns `BalanceResult::Unique` for a one-dimensional null space, or
/// `BalanceResult::Family` when the skeleton is underdetermined.
pub fn balance(lhs: &[&str], rhs: &[&str]) -> Result<BalanceResult, BalanceError> {
    let species: Vec<Formula> = lhs
        .iter()
        .chain(rhs.iter())
        .map(|s| parse_formula(s))
        .collect::<Result<_, _>>()
        .map_err(BalanceError::Parse)?;
    if species.len() < 2 {
        return Err(BalanceError::TooFewSpecies);
    }
    let mut elements: Vec<String> = species
        .iter()
        .flat_map(|f| f.counts.keys().cloned())
        .collect();
    elements.sort();
    elements.dedup();

    let n = species.len();
    let zero = Rational64::from_integer(0);
    let one = Rational64::from_integer(1);

    let to_r = |v: f64| -> Result<Rational64, BalanceError> {
        let rounded = v.round();
        if (v - rounded).abs() > 1e-6 || !rounded.is_finite() || rounded.abs() > 1e15 {
            return Err(BalanceError::Impossible);
        }
        Ok(Rational64::from_integer(rounded as i64))
    };

    let mut rows: Vec<Vec<Rational64>> = Vec::new();
    for el in &elements {
        let mut row_vec = Vec::with_capacity(n);
        for (i, f) in species.iter().enumerate() {
            let sign = if i < lhs.len() { one } else { -one };
            let count = to_r(f.counts.get(el).copied().unwrap_or(0.0))?;
            row_vec.push(sign * count);
        }
        rows.push(row_vec);
    }
    {
        let mut charge_row = Vec::with_capacity(n);
        for (i, f) in species.iter().enumerate() {
            let sign = if i < lhs.len() { one } else { -one };
            let charge = to_r(f.charge)?;
            charge_row.push(sign * charge);
        }
        rows.push(charge_row);
    }

    // Gaussian elimination to reduced row echelon form, exact arithmetic.
    let mut pivots: Vec<usize> = Vec::new();
    let mut cur_row = 0;
    for col in 0..n {
        let sel = (cur_row..rows.len()).find(|&r| rows[r][col] != zero);
        let Some(sel) = sel else { continue };
        rows.swap(cur_row, sel);
        let d = rows[cur_row][col];
        for v in rows[cur_row].iter_mut() {
            *v /= d;
        }
        for r in 0..rows.len() {
            if r != cur_row && rows[r][col] != zero {
                let f = rows[r][col];
                let pivot_row = rows[cur_row].clone();
                for (target, p) in rows[r].iter_mut().zip(&pivot_row) {
                    *target -= f * p;
                }
            }
        }
        pivots.push(col);
        cur_row += 1;
        if cur_row == rows.len() {
            break;
        }
    }

    let free: Vec<usize> = (0..n).filter(|c| !pivots.contains(c)).collect();
    if free.is_empty() {
        return Err(BalanceError::Impossible);
    }

    // Extract one null-space basis vector per free variable.
    let mut basis_vecs: Vec<Vec<Rational64>> = Vec::with_capacity(free.len());
    for &fv in &free {
        let mut v = vec![zero; n];
        v[fv] = one;
        for (i, &p) in pivots.iter().enumerate() {
            v[p] = -rows[i][fv];
        }
        basis_vecs.push(v);
    }

    // Convert a rational vector to smallest positive integers.
    let to_ints = |v: &[Rational64]| -> Result<Vec<i64>, BalanceError> {
        // Find the LCM of all denominators.
        let mut lcm_d: i64 = 1;
        for r in v {
            if *r != zero {
                let d = *r.denom();
                lcm_d = lcm(lcm_d, d);
            }
        }
        let mut ints: Vec<i64> = v
            .iter()
            .map(|r| {
                let scaled = *r * Rational64::from_integer(lcm_d);
                *scaled.numer()
            })
            .collect();
        let g = ints.iter().copied().fold(0i64, gcd);
        if g > 1 {
            for c in ints.iter_mut() {
                *c /= g;
            }
        }
        // Ensure first nonzero is positive.
        if ints.iter().find(|&&c| c != 0).copied().unwrap_or(1) < 0 {
            for c in ints.iter_mut() {
                *c = -*c;
            }
        }
        Ok(ints)
    };

    if free.len() == 1 {
        let ints = to_ints(&basis_vecs[0])?;
        if ints.iter().any(|&c| c <= 0) {
            return Err(BalanceError::Impossible);
        }
        return Ok(BalanceResult::Unique(ints));
    }

    // Underdetermined: find one particular all-positive solution, then
    // return the full basis alongside it. Try each basis vector, their
    // sum, and small positive linear combinations.
    let find_positive = |v: &[Rational64]| -> bool {
        match to_ints(v) {
            Ok(ints) => ints.iter().all(|&c| c > 0),
            Err(_) => false,
        }
    };

    let mut particular: Option<Vec<Rational64>> = None;
    // Try each basis vector individually.
    for bv in &basis_vecs {
        if find_positive(bv) {
            particular = Some(bv.clone());
            break;
        }
    }
    // Try small positive linear combinations.
    if particular.is_none() && basis_vecs.len() == 2 {
        'search: for a in 1i64..=5 {
            for b in 1i64..=5 {
                let ra = Rational64::from_integer(a);
                let rb = Rational64::from_integer(b);
                let combo: Vec<Rational64> = (0..n)
                    .map(|j| ra * basis_vecs[0][j] + rb * basis_vecs[1][j])
                    .collect();
                if find_positive(&combo) {
                    particular = Some(combo);
                    break 'search;
                }
            }
        }
    }
    if particular.is_none() {
        // General fallback: sum all basis vectors.
        let mut sum = vec![zero; n];
        for bv in &basis_vecs {
            for (j, val) in bv.iter().enumerate() {
                sum[j] += *val;
            }
        }
        if find_positive(&sum) {
            particular = Some(sum);
        }
    }
    let particular = particular.ok_or(BalanceError::Impossible)?;
    let part_ints = to_ints(&particular)?;

    let basis_ints: Vec<Vec<i64>> = basis_vecs
        .iter()
        .map(|bv| to_ints(bv))
        .collect::<Result<_, _>>()?;

    Ok(BalanceResult::Family {
        particular: part_ints,
        basis: basis_ints,
    })
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd(a, b)) * b
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BalanceError {
    #[error("{0}")]
    Parse(ParseError),
    #[error("a reaction needs at least two species")]
    TooFewSpecies,
    #[error("no set of positive coefficients balances this")]
    Impossible,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fuzzer's crash, kept as a test.
    ///
    /// Found by the peer session's `stoich` target in under two minutes,
    /// and reachable from `kero balance` and the MCP balance tool, so it
    /// was a user-facing panic on crafted input rather than a curiosity.
    ///
    /// The skeleton is nonsense, which is the point: a degenerate element
    /// matrix has a null space that can throw out numbers of any size, and
    /// `as i64` saturates instead of wrapping, so one arriving at i64::MIN
    /// made the sign flip panic with "attempt to negate with overflow".
    ///
    /// Both harvested artifacts dedupe to this one site. The bytes below
    /// are the valid UTF-8 prefix of the smaller one — which is all
    /// libfuzzer's `&str` ever handed the parser.
    #[test]
    fn a_degenerate_skeleton_is_refused_rather_than_overflowing() {
        let data = "HHI  + H+=B0I  + IH\n*2I44444444444444444444";
        let (l, r) = data
            .split_once('=')
            .expect("the artifact contains an arrow");
        let lhs: Vec<&str> = l.split(" + ").map(str::trim).collect();
        let rhs: Vec<&str> = r.split(" + ").map(str::trim).collect();
        // The assertion is that this returns at all.
        assert!(
            balance(&lhs, &rhs).is_err(),
            "a skeleton with no sane coefficients must be refused, not balanced"
        );
    }

    fn f(s: &str) -> Formula {
        parse_formula(s).unwrap_or_else(|e| panic!("{s}: {e}"))
    }

    #[test]
    fn plain_formulas() {
        assert_eq!(f("H2O").counts["H"], 2.0);
        assert_eq!(f("H2O").counts["O"], 1.0);
        assert_eq!(f("CaCO3").counts["Ca"], 1.0);
        assert_eq!(f("CaCO3").counts["O"], 3.0);
    }

    #[test]
    fn unicode_subscripts_and_charges() {
        let s = f("SO₄²⁻");
        assert_eq!(s.counts["S"], 1.0);
        assert_eq!(s.counts["O"], 4.0);
        assert_eq!(s.charge, -2.0);
        assert_eq!(f("Ag⁺").charge, 1.0);
        assert_eq!(f("Ca²⁺").charge, 2.0);
        assert_eq!(f("NO₃⁻").charge, -1.0);
    }

    #[test]
    fn registry_style_charges_too() {
        assert_eq!(f("Ag+").charge, 1.0);
        assert_eq!(f("SO4-2").charge, -2.0);
        assert_eq!(f("Cu+2").charge, 2.0);
    }

    #[test]
    fn a_digit_before_a_trailing_sign_is_a_subscript() {
        // The conflict that matters: in `MnO4-` the 4 is a subscript and
        // the charge is -1. Reading it as the charge magnitude gave "MnO
        // with charge -4", and permanganate then failed to balance against
        // anything.
        let p = f("MnO4-");
        assert_eq!(p.counts["O"], 4.0);
        assert_eq!(p.counts["Mn"], 1.0);
        assert_eq!(p.charge, -1.0);
        for (formula, o, q) in [
            ("NO3-", 3.0, -1.0),
            ("HCO3-", 3.0, -1.0),
            ("CrO4-2", 4.0, -2.0),
        ] {
            let x = f(formula);
            assert_eq!(x.counts["O"], o, "{formula}");
            assert_eq!(x.charge, q, "{formula}");
        }
        // Repeated signs still carry magnitude, which is how to write a
        // divalent cation without the PHREEQC suffix.
        assert_eq!(f("Ca++").charge, 2.0);
    }

    fn unique(r: Result<BalanceResult, BalanceError>) -> Vec<i64> {
        match r {
            Ok(BalanceResult::Unique(v)) => v,
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn permanganate_half_reaction_balances() {
        // The case the notation bug broke. Textbook answer:
        // MnO4- + 5 Fe2+ + 8 H+ -> Mn2+ + 5 Fe3+ + 4 H2O
        let n = unique(balance(&["MnO4-", "Fe+2", "H+"], &["Mn+2", "Fe+3", "H2O"]));
        assert_eq!(n, vec![1, 5, 8, 1, 5, 4]);
    }

    #[test]
    fn parenthesised_groups() {
        let c = f("Ca(OH)2");
        assert_eq!(c.counts["Ca"], 1.0);
        assert_eq!(c.counts["O"], 2.0);
        assert_eq!(c.counts["H"], 2.0);
    }

    #[test]
    fn hydrates() {
        let g = f("CaSO₄·2H₂O");
        assert_eq!(g.counts["Ca"], 1.0);
        assert_eq!(g.counts["S"], 1.0);
        assert_eq!(g.counts["O"], 6.0);
        assert_eq!(g.counts["H"], 4.0);
    }

    #[test]
    fn state_labels_are_not_elements() {
        // "(aq)" must not parse as a group of "a" and "q".
        let a = f("AgCl(s)");
        assert_eq!(a.counts.len(), 2);
        assert!(a.counts.contains_key("Ag") && a.counts.contains_key("Cl"));
    }

    #[test]
    fn prose_is_not_a_formula() {
        assert!(parse_formula("buffer").is_err());
        assert!(parse_formula("1.43 g theoretical").is_err());
        assert!(parse_formula("no precipitate").is_err());
    }

    #[test]
    fn balanced_equations_are_recognised() {
        let e = parse_equation("2 Mg + O₂ → 2 MgO").unwrap();
        assert!(e.is_balanced(), "{:?}", e.element_imbalance());
        let ionic = parse_equation("Ag⁺ + Cl⁻ → AgCl(s)").unwrap();
        assert!(ionic.is_balanced(), "{:?}", ionic.element_imbalance());
    }

    #[test]
    fn unbalanced_equations_are_caught() {
        let e = parse_equation("Mg + O₂ → MgO").unwrap();
        assert!(!e.is_balanced());
        assert_eq!(e.element_imbalance(), vec![("O".to_string(), -1.0)]);
    }

    #[test]
    fn charge_is_checked_not_just_atoms() {
        // Atoms conserve; charge does not. A common student error, and the
        // reason charge gets its own row in the matrix.
        let e = parse_equation("Fe²⁺ → Fe³⁺").unwrap();
        assert!(e.element_imbalance().is_empty());
        assert_eq!(e.charge_imbalance(), 1.0);
        assert!(!e.is_balanced());
    }

    #[test]
    fn balancing_finds_the_coefficients() {
        assert_eq!(unique(balance(&["Mg", "O2"], &["MgO"])), vec![2, 1, 2]);
        assert_eq!(
            unique(balance(&["CH4", "O2"], &["CO2", "H2O"])),
            vec![1, 2, 1, 2]
        );
        assert_eq!(
            unique(balance(&["Fe2O3", "C"], &["Fe", "CO2"])),
            vec![2, 3, 4, 3]
        );
    }

    #[test]
    fn balancing_respects_charge() {
        assert_eq!(unique(balance(&["Ag+", "Cl-"], &["AgCl"])), vec![1, 1, 1]);
        assert_eq!(
            unique(balance(&["Ca+2", "PO4-3"], &["Ca3(PO4)2"])),
            vec![3, 2, 1]
        );
    }

    #[test]
    fn an_impossible_skeleton_is_refused() {
        assert!(matches!(
            balance(&["H2O"], &["NaCl"]),
            Err(BalanceError::Impossible)
        ));
    }

    #[test]
    fn underdetermined_carbon_oxidation_returns_family() {
        // C + O₂ → CO + CO₂ admits two independent reactions:
        //   2C + O₂ → 2CO     (partial oxidation)
        //   C + O₂ → CO₂      (complete oxidation)
        let r = balance(&["C", "O2"], &["CO", "CO2"]).unwrap();
        match r {
            BalanceResult::Family {
                particular, basis, ..
            } => {
                assert!(
                    particular.iter().all(|&c| c > 0),
                    "particular solution must be all-positive: {particular:?}"
                );
                assert!(
                    !basis.is_empty(),
                    "underdetermined system must have at least one basis vector"
                );
                // Verify the particular solution actually balances.
                let lhs_formulas = ["C", "O2"];
                let rhs_formulas = ["CO", "CO2"];
                verify_balances(&lhs_formulas, &rhs_formulas, &particular);
            }
            BalanceResult::Unique(_) => {
                panic!("C + O₂ → CO + CO₂ is underdetermined, expected Family");
            }
        }
    }

    #[test]
    fn underdetermined_permanganate_peroxide() {
        // MnO₄⁻ + H₂O₂ + H⁺ → Mn²⁺ + O₂ + H₂O
        // Underdetermined because both MnO₄⁻ and H₂O₂ can provide oxygen.
        let r = balance(&["MnO4-", "H2O2", "H+"], &["Mn+2", "O2", "H2O"]).unwrap();
        match r {
            BalanceResult::Family {
                particular, basis, ..
            } => {
                assert!(
                    particular.iter().all(|&c| c > 0),
                    "particular solution must be all-positive: {particular:?}"
                );
                assert!(!basis.is_empty());
                let lhs_formulas = ["MnO4-", "H2O2", "H+"];
                let rhs_formulas = ["Mn+2", "O2", "H2O"];
                verify_balances(&lhs_formulas, &rhs_formulas, &particular);
            }
            BalanceResult::Unique(_) => {
                panic!("expected underdetermined");
            }
        }
    }

    fn verify_balances(lhs: &[&str], rhs: &[&str], coeffs: &[i64]) {
        let all_formulas: Vec<Formula> = lhs
            .iter()
            .chain(rhs.iter())
            .map(|s| parse_formula(s).unwrap())
            .collect();
        let mut elements: Vec<String> = all_formulas
            .iter()
            .flat_map(|f| f.counts.keys().cloned())
            .collect();
        elements.sort();
        elements.dedup();
        for el in &elements {
            let mut sum = 0i64;
            for (i, f) in all_formulas.iter().enumerate() {
                let sign = if i < lhs.len() { 1 } else { -1 };
                let count = f.counts.get(el).copied().unwrap_or(0.0) as i64;
                sum += sign * coeffs[i] * count;
            }
            assert_eq!(sum, 0, "element {el} does not balance");
        }
        let mut charge_sum = 0i64;
        for (i, f) in all_formulas.iter().enumerate() {
            let sign = if i < lhs.len() { 1 } else { -1 };
            charge_sum += sign * coeffs[i] * f.charge as i64;
        }
        assert_eq!(charge_sum, 0, "charge does not balance");
    }
}

//! `kero balance exercise` — GUI-095's drill, headless.
//!
//! The engine half already exists: `stoich::balance_report` strips an
//! equation's coefficients and returns the answer *together with the
//! composition matrix*, which is what lets a host mark an answer the
//! solver never produced. The browser drill marks with that matrix in
//! TypeScript (`web/app/src/lib/balancing.ts`). This is the same marking
//! rule for hosts that have no browser — the CLI today, the MCP server
//! next — so the drill can be exercised, scripted and regression-tested
//! without one.
//!
//! **One definition of "balanced", two implementations.** The check here
//! is deliberately not a second opinion: it is the same dot product
//! against the same `report.matrix` the engine emitted, with the same
//! four verdicts and the same names as `markBalance`. Nothing in this
//! file decides what balances — the matrix does, and the matrix comes
//! from the solver. If that rule ever changes it changes in one place,
//! and both surfaces follow.
//!
//! The pool is the other gap. The app builds its list of drillable
//! reactions in the client, out of the codex export it has already
//! fetched; a headless host has no such list, so this reads the codex
//! from disk and offers the same field — `equation`, the one the codex
//! lint already proves balanced — with the curated cascade the engine
//! compiles in behind it.

use kerotakis_core::stoich::{self, BalanceReport};

/// The verdicts, spelled exactly as `BalanceVerdict` in the app's
/// `balancing.ts`. Two surfaces, one vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Balances, in the smallest whole-number ratio.
    Correct,
    /// Balances, but every coefficient shares a factor. The actual lesson.
    Multiple,
    /// Does not conserve some element, or the charge.
    Unbalanced,
    /// Not yet an answer: a blank, a zero, a fraction, a negative.
    Incomplete,
}

impl Verdict {
    pub fn tag(self) -> &'static str {
        match self {
            Verdict::Correct => "correct",
            Verdict::Multiple => "multiple",
            Verdict::Unbalanced => "unbalanced",
            Verdict::Incomplete => "incomplete",
        }
    }
}

/// One row of the composition matrix that does not cancel. `amount` is
/// the surplus on the LEFT, because the report negates right-hand species.
#[derive(Debug, Clone, PartialEq)]
pub struct Miss {
    pub element: String,
    pub amount: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub verdict: Verdict,
    pub misses: Vec<Miss>,
    /// The shared factor, when the answer is a correct multiple.
    pub factor: i64,
    /// True when the skeleton admits more than one independent reaction.
    pub family: bool,
}

const TOLERANCE: f64 = 1e-6;

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Mark an answer against the matrix the engine reported.
///
/// A transliteration of `markBalance` in the app, kept deliberately close
/// to it: same order of tests, same tolerance, same verdicts. A coefficient
/// vector balances precisely when every row's dot product with it is zero.
pub fn mark(report: &BalanceReport, answer: &[i64]) -> Mark {
    let family = !report.basis.is_empty();
    if answer.len() != report.species.len() || answer.iter().any(|v| *v <= 0) {
        return Mark {
            verdict: Verdict::Incomplete,
            misses: Vec::new(),
            factor: 0,
            family,
        };
    }
    let mut misses: Vec<Miss> = Vec::new();
    for (index, row) in report.matrix.iter().enumerate() {
        let surplus: f64 = row
            .iter()
            .zip(answer)
            .map(|(count, n)| count * *n as f64)
            .sum();
        if surplus.abs() > TOLERANCE {
            misses.push(Miss {
                element: report
                    .elements
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| format!("row {index}")),
                amount: surplus,
            });
        }
    }
    if !misses.is_empty() {
        misses.sort_by(|a, b| {
            b.amount
                .abs()
                .partial_cmp(&a.amount.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        return Mark {
            verdict: Verdict::Unbalanced,
            misses,
            factor: 0,
            family,
        };
    }
    let factor = answer.iter().fold(0i64, |a, b| gcd(a, *b));
    Mark {
        verdict: if factor > 1 {
            Verdict::Multiple
        } else {
            Verdict::Correct
        },
        misses: Vec::new(),
        factor,
        family,
    }
}

/// The equation written out with these coefficients, a bare 1 left
/// implicit the way it is written by hand.
pub fn write_equation(report: &BalanceReport, coefficients: &[i64]) -> String {
    let term = |i: usize| -> String {
        let name = &report.species[i];
        match coefficients.get(i) {
            Some(1) | None => name.clone(),
            Some(n) => format!("{n} {name}"),
        }
    };
    let left: Vec<String> = (0..report.reactants).map(term).collect();
    let right: Vec<String> = (report.reactants..report.species.len()).map(term).collect();
    format!(
        "{} {} {}",
        left.join(" + "),
        if report.reversible { "⇌" } else { "→" },
        right.join(" + ")
    )
}

/// The skeleton as a question: no coefficients at all.
pub fn blank_equation(report: &BalanceReport) -> String {
    write_equation(report, &vec![1; report.species.len()])
}

/// One drillable reaction and where it came from.
pub struct Exercise {
    pub id: String,
    pub source: String,
    pub report: BalanceReport,
}

/// The pool, and what could not be drilled.
///
/// Both halves returned, because a generator that silently drops what it
/// cannot parse reports a coverage it has not earned — the codex's
/// `equation` field carries prose and annotated arrows as well as
/// equations, and `equation_audit` already counts them that way.
#[derive(Default)]
pub struct Pool {
    pub exercises: Vec<Exercise>,
    pub unusable: Vec<(String, String)>,
}

/// Build the pool: the codex on disk first, then the curated cascade the
/// engine compiles in.
pub fn pool(dir: &str) -> Pool {
    // A free function rather than a closure: a closure capturing `pool`
    // holds the borrow for its whole lifetime, and the codex's own load
    // failure needs to push to `unusable` too.
    fn offer(pool: &mut Pool, id: String, equation: &str) {
        match stoich::balance_report(equation) {
            Ok(report) => pool.exercises.push(Exercise {
                id,
                source: equation.to_string(),
                report,
            }),
            Err(e) => pool.unusable.push((id, e.to_string())),
        }
    }

    let mut pool = Pool::default();
    match kerotakis_codex::Codex::load_dir(std::path::Path::new(dir)) {
        Ok(codex) => {
            for entry in &codex.reactions {
                let Some(equation) = &entry.equation else {
                    continue;
                };
                // A chain of equilibria is several equations sharing their
                // intermediate terms; the codex already knows how to split
                // its own field, so ask it rather than re-deriving the rule.
                for (i, clause) in kerotakis_codex::equation_clauses(equation)
                    .into_iter()
                    .enumerate()
                {
                    let id = if i == 0 {
                        entry.id.clone()
                    } else {
                        format!("{}#{}", entry.id, i + 1)
                    };
                    offer(&mut pool, id, &clause);
                }
            }
        }
        Err(e) => pool.unusable.push((dir.to_string(), e.to_string())),
    }

    for (i, reaction) in kerotakis_core::curated::REACTIONS.iter().enumerate() {
        offer(&mut pool, format!("curated/{i}"), reaction.equation);
    }
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(equation: &str) -> BalanceReport {
        stoich::balance_report(equation).expect("balances")
    }

    #[test]
    fn the_solvers_own_answer_is_correct_and_simplest() {
        let r = report("H₂ + O₂ → H₂O");
        assert_eq!(r.coefficients, vec![2, 1, 2]);
        assert_eq!(mark(&r, &r.coefficients).verdict, Verdict::Correct);
    }

    #[test]
    fn a_correct_multiple_is_balanced_and_named_as_a_multiple() {
        // The lesson GUI-095 exists for: 4/2/4 conserves everything, it is
        // simply not the smallest whole-number ratio.
        let r = report("H₂ + O₂ → H₂O");
        let m = mark(&r, &[4, 2, 4]);
        assert_eq!(m.verdict, Verdict::Multiple);
        assert_eq!(m.factor, 2);
    }

    #[test]
    fn charge_is_marked_as_well_as_the_atoms() {
        // The matrix carries a charge row, so an answer that conserves the
        // atoms while inventing charge is still refused.
        let r = report("Ag⁺ + Cl⁻ → AgCl");
        let m = mark(&r, &[2, 1, 1]);
        assert_eq!(m.verdict, Verdict::Unbalanced);
        assert!(m.misses.iter().any(|miss| miss.element == "charge"));
    }

    #[test]
    fn a_blank_or_impossible_coefficient_is_not_an_answer() {
        let r = report("H₂ + O₂ → H₂O");
        assert_eq!(mark(&r, &[2, 1]).verdict, Verdict::Incomplete);
        assert_eq!(mark(&r, &[2, 0, 2]).verdict, Verdict::Incomplete);
        assert_eq!(mark(&r, &[2, -1, 2]).verdict, Verdict::Incomplete);
    }

    /// The property the drill rests on, over every reaction the shipped
    /// codex and the curated cascade can pose: the stored answer is marked
    /// correct, doubling it is marked a multiple, and bumping any single
    /// coefficient breaks it.
    ///
    /// Run against the real `codex/` directory, because that is the pool a
    /// learner meets — a marker that is right about the seventeen curated
    /// reactions and wrong about the hundreds in the catalogue has proved
    /// nothing.
    #[test]
    fn every_shipped_exercise_accepts_its_answer_and_refuses_a_perturbation() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../codex");
        let p = pool(dir);
        assert!(
            p.exercises.len() > 50,
            "expected a real pool from the shipped codex, got {}",
            p.exercises.len()
        );
        // The floor above would still pass on the curated cascade alone,
        // which is the failure that matters: a codex that stops loading
        // leaves a pool that looks plausible and teaches almost nothing.
        assert!(
            p.exercises.iter().any(|e| !e.id.starts_with("curated/")),
            "the codex contributed nothing — only the compiled-in reactions are here"
        );
        for exercise in &p.exercises {
            let r = &exercise.report;
            // An underdetermined skeleton has a whole family of answers, so
            // "the" answer and a perturbation of it are not meaningful.
            if !r.basis.is_empty() {
                continue;
            }
            assert_eq!(
                mark(r, &r.coefficients).verdict,
                Verdict::Correct,
                "{}: the solver's own answer {:?} was not marked correct\n  {}",
                exercise.id,
                r.coefficients,
                exercise.source
            );
            let doubled: Vec<i64> = r.coefficients.iter().map(|c| c * 2).collect();
            let m = mark(r, &doubled);
            assert_eq!(
                m.verdict,
                Verdict::Multiple,
                "{}: a doubled answer was not marked a multiple",
                exercise.id
            );
            assert_eq!(m.factor % 2, 0, "{}: doubling lost its factor", exercise.id);
            for i in 0..r.coefficients.len() {
                let mut wrong = r.coefficients.clone();
                wrong[i] += 1;
                assert_ne!(
                    mark(r, &wrong).verdict,
                    Verdict::Correct,
                    "{}: {:?} still marked correct with position {i} bumped\n  {}",
                    exercise.id,
                    wrong,
                    exercise.source
                );
            }
        }
    }

    #[test]
    fn what_cannot_be_drilled_is_reported_rather_than_dropped() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../codex");
        let p = pool(dir);
        assert!(
            !p.unusable.is_empty(),
            "every equation parsed — check the reporting, not the luck"
        );
        for (id, why) in &p.unusable {
            assert!(!why.is_empty(), "{id}: refused without saying why");
        }
    }
}

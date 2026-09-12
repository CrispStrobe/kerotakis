//! Does a sentence about a reaction name the species the reaction has?
//!
//! `quest lint` proves a quest cannot lie about being ACHIEVABLE — its goal
//! is reachable and it cannot block. It said nothing at all about whether
//! the prose beside that goal describes the mechanism the engine runs. On
//! 2026-09-08 the iodine-clock quest told a learner that the Landolt rate
//! law was consuming `NaHSO3`; the law had been re-keyed onto `HSO3-`
//! because a solved vessel holds the ion and not the bottle; and the lint
//! passed on the false sentence. It was fixed by inspection, not by a gate.
//!
//! That is one instance of a pattern this repository has now met on four
//! surfaces: **rename a reactant and everything keyed to the old name
//! follows silently.** Curated reactions were guarded after the vinegar and
//! baking-soda bug, rate laws by `kinetic_reactants_survive_a_solve`, and
//! hand-built test vessels by their own assertions. Prose was the only one
//! of the four with no guard at all. This module is that guard.
//!
//! ## What makes it tractable
//!
//! Only sentences about what the ENGINE DOES go stale. "The bottle is on
//! the shelf" stays true, because the learner really does add the bottle;
//! "the rate law is consuming the bottle" does not, because the law
//! consumes the ion. A lint that flagged every mention of a species would
//! be noise, so this one flags a species only where the sentence puts it in
//! a MECHANISM POSITION — inside a concentration bracket, as the object of
//! a consuming/producing verb, or as the thing a law is "keyed on".
//!
//! ## What it catches, and what it cannot
//!
//! It catches: a chemical formula, in a mechanism position, in prose bound
//! to a named reaction, that the reaction does not name — and a rate
//! expression whose bracketed species are the order set of no rate law in
//! the registry.
//!
//! It does NOT catch, and no passing run should be read as saying
//! otherwise:
//!
//! - **Common names.** "the rate law is consuming bisulfite" is invisible;
//!   only `HSO3-`, `NaHSO3` and their display spellings are matched. The
//!   repository's own codex prose is written largely in common names and
//!   in `c(...)`/`a(...)` notation, so this guard reaches very little of
//!   it.
//! - **Unbound prose.** A sentence is checked against a reaction only when
//!   its quest nudge or codex entry declares `reacted:<id>`. Prose that
//!   describes a reaction it does not name is checked only by the
//!   rate-expression rule.
//! - **Everything that is not a participant list.** Coefficients, reaction
//!   order, rate-determining step, colour, temperature, safety, and any
//!   claim about WHY. A sentence can name only real participants and still
//!   be wrong about all of those.
//! - **Translated prose, except its rate expressions.** The verb list is
//!   English. German was tried and declined rather than half-done: German
//!   puts the object before the verb in a subordinate clause and after it
//!   in a main clause, so a list of German verbs that looked forward for
//!   its object would read the wrong noun about half the time — and on
//!   this corpus it found nothing it was right about, so it would have
//!   been risk with no yield. `codex/i18n/de.toml` is therefore covered
//!   only by the rate-expression rule, which is language-independent
//!   because a bracket is.
//! - **Prose outside quests and the codex** — lesson step prose, UI
//!   strings, `PLAN.md`. Step prose (`data/steps/step-prose-v1.json`) is
//!   bound to a script LINE rather than to a reaction, and is written in
//!   common names — "the same salt, ten times the concentration" — so it
//!   has the same exposure by a different route and would need a
//!   different rule.
//!
//! In short: a passing lint means no bound sentence names a formula the
//! reaction lacks. It does not mean the prose is true.
//!
//! ## Why this one fails and [`crate::prose`] only advises
//!
//! [`crate::prose`] checks the NUMBERS in an entry against the run, and is
//! deliberately advisory: a good sentence may quote an activation energy,
//! a molar mass or a figure from a different experiment, and flagging
//! those would train authors to strip real content out. The set a
//! mechanism claim draws from is not open in that way. A reaction's
//! participants are a closed list the engine can state exactly, so naming
//! something outside it is not a judgement call, and this check fails the
//! lint rather than filing a work item.

use std::collections::BTreeSet;

use kerotakis_core::{kinetics, species, SpeciesId};

/// Is this the id of a reaction a `reacted:<id>` trigger can name?
///
/// `Event::Reacted` is emitted by the kinetics network and carries a rate
/// law's id, so this is the registry that can answer it. Curated reactions
/// have no id of their own — they are keyed by equation — and report
/// `Event::Reaction`, which is a different trigger.
pub fn reaction_exists(id: &str) -> bool {
    kinetics::lookup(id).is_some()
}

/// If this trigger binds prose to a reaction, which reaction.
pub fn bound_reaction(trigger: &str) -> Option<&str> {
    let id = trigger.strip_prefix("reacted:")?;
    reaction_exists(id).then_some(id)
}

/// Everything a run of text may say a reaction does.
struct Vocabulary {
    /// Every spelling of every species the reaction names anywhere.
    named: BTreeSet<String>,
    /// Every spelling of every species its rate law is keyed on.
    orders: BTreeSet<String>,
}

fn spellings(key: &str, into: &mut BTreeSet<String>) {
    into.insert(normalise(key));
    if let Some(data) = species::lookup(&SpeciesId::new(key)) {
        into.insert(normalise(data.formula));
        into.insert(normalise(data.key));
    }
}

fn vocabulary(id: &str) -> Option<Vocabulary> {
    let reaction = kinetics::lookup(id)?;
    let mut named = BTreeSet::new();
    let mut orders = BTreeSet::new();
    for term in reaction.stoichiometry {
        spellings(term.species, &mut named);
    }
    for term in reaction.forward.orders {
        spellings(term.species, &mut named);
        spellings(term.species, &mut orders);
    }
    if let Some(reverse) = &reaction.reverse {
        for term in reverse.orders {
            spellings(term.species, &mut named);
            spellings(term.species, &mut orders);
        }
    }
    for catalyst in reaction.catalysts {
        spellings(catalyst.species, &mut named);
    }
    // The equation is the reaction's own display spelling, so anything
    // written in it is by definition something the reaction names.
    for word in reaction.equation.split_whitespace() {
        if let Some(token) = candidate(word) {
            named.insert(normalise(&token));
        }
    }
    Some(Vocabulary { named, orders })
}

/// The registry key spelling of a formula as prose writes it.
///
/// `HSO₃⁻` and `HSO3-` are the same species; so are `S₂O₃²⁻` and `S2O3-2`,
/// where the charge moves from after the count to before it. Peel the
/// trailing SUPERSCRIPT charge first — otherwise a subscript count and a
/// superscript charge both flatten to bare digits and the two can no longer
/// be told apart.
pub fn normalise(token: &str) -> String {
    let chars: Vec<char> = token
        .chars()
        .filter(|c| *c != '·' && !c.is_whitespace())
        .collect();
    let mut end = chars.len();
    let mut charge = String::new();
    if let Some(sign) = chars.last().copied().and_then(|c| match c {
        '⁺' => Some('+'),
        '⁻' => Some('-'),
        _ => None,
    }) {
        let mut cut = end - 1;
        let mut digits = String::new();
        while cut > 0 {
            match superscript_digit(chars[cut - 1]) {
                Some(d) => {
                    digits.insert(0, d);
                    cut -= 1;
                }
                None => break,
            }
        }
        // A lone `⁻` is punctuation, not a charge: leave such a token alone.
        if cut > 0 {
            end = cut;
            charge.push(sign);
            if digits != "1" {
                charge.push_str(&digits);
            }
        }
    }
    let mut out = String::with_capacity(chars.len());
    for c in &chars[..end] {
        out.push(plain_digit(*c).unwrap_or(*c));
    }
    out.push_str(&charge);
    out
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

fn plain_digit(c: char) -> Option<char> {
    if let Some(d) = superscript_digit(c) {
        return Some(d);
    }
    Some(match c {
        '₀' => '0',
        '₁' => '1',
        '₂' => '2',
        '₃' => '3',
        '₄' => '4',
        '₅' => '5',
        '₆' => '6',
        '₇' => '7',
        '₈' => '8',
        '₉' => '9',
        '⁺' => '+',
        '⁻' => '-',
        _ => return None,
    })
}

const EDGE: &[char] = &[
    '.', ',', ';', ':', '!', '?', '"', '\'', '(', ')', '[', ']', '—', '–', '…', '“', '”', '„', '‘',
    '’', '«', '»',
];

fn token_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, '(' | ')' | '+' | '-' | '⁺' | '⁻')
        || plain_digit(c).is_some()
}

/// The chemical token a word could be, with sentence punctuation removed.
fn candidate(word: &str) -> Option<String> {
    let trimmed = word.trim_matches(|c: char| EDGE.contains(&c));
    if trimmed.is_empty() || !trimmed.chars().all(token_char) {
        return None;
    }
    Some(trimmed.to_string())
}

/// Whether a token is chemistry at all, so that "produces heat" is quiet.
///
/// Deliberately two tests rather than one. A registry species is chemistry
/// whatever it looks like (`Fe`, `water`); and a formula-SHAPED token is
/// chemistry even when the registry has never heard of it, which is what
/// catches an ion the bench does not carry (`I⁻`) being claimed as a rate
/// law's input.
fn chemical(normalised: &str) -> bool {
    if species::lookup(&SpeciesId::new(normalised)).is_some() {
        return true;
    }
    if species::registry()
        .iter()
        .any(|s| normalise(s.formula) == normalised)
    {
        return true;
    }
    formula_shaped(normalised)
}

fn formula_shaped(n: &str) -> bool {
    if !n.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if !n
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '(' | ')' | '+' | '-'))
    {
        return false;
    }
    // A capitalised English word at the start of a sentence is not a
    // formula; a digit, a charge or a second element symbol says it is.
    n.chars().any(|c| c.is_ascii_digit())
        || n.contains('+')
        || n.contains('-')
        || n.chars().filter(|c| c.is_ascii_uppercase()).count() > 1
}

const ACTIVE: &[&str] = &[
    "consume",
    "consumes",
    "consuming",
    "produce",
    "produces",
    "producing",
    "release",
    "releases",
    "releasing",
    "oxidise",
    "oxidises",
    "oxidising",
    "oxidize",
    "oxidizes",
    "oxidizing",
    "reduce",
    "reduces",
    "reducing",
    "forms",
    "forming",
];

const PASSIVE: &[&str] = &[
    "consumed", "produced", "released", "oxidised", "oxidized", "reduced", "formed",
];

const ARTICLES: &[&str] = &[
    "the",
    "a",
    "an",
    "any",
    "its",
    "that",
    "this",
    "some",
    "more",
    "free",
    "dissolved",
    "aqueous",
    "solid",
    "every",
    "all",
    "both",
];

fn lower(word: &str) -> String {
    word.trim_matches(|c: char| EDGE.contains(&c))
        .to_lowercase()
}

fn next_candidate(words: &[&str], from: usize) -> Option<String> {
    let mut i = from;
    while i < words.len() && ARTICLES.contains(&lower(words[i]).as_str()) {
        i += 1;
    }
    candidate(words.get(i)?)
}

/// The species this text puts in a mechanism position, and how.
fn mechanism_claims(text: &str) -> Vec<(&'static str, String)> {
    let mut found = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        let w = lower(word);
        if ACTIVE.contains(&w.as_str()) {
            if let Some(token) = next_candidate(&words, i + 1) {
                found.push(("names", token));
            }
        } else if w == "keyed" {
            let mut j = i + 1;
            while j < words.len() && matches!(lower(words[j]).as_str(), "on" | "onto" | "to") {
                j += 1;
            }
            if let Some(token) = next_candidate(&words, j) {
                found.push(("names", token));
            }
        } else if PASSIVE.contains(&w.as_str()) {
            let mut j = i;
            while j > 0 && matches!(lower(words[j - 1]).as_str(), "is" | "are" | "being") {
                j -= 1;
            }
            if j < i && j > 0 {
                if let Some(token) = candidate(words[j - 1]) {
                    found.push(("names", token));
                }
            }
        }
    }
    for inner in bracketed(text) {
        if let Some(token) = candidate(inner.trim()) {
            found.push(("is keyed on", token));
        }
    }
    found
}

fn bracketed(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        match after.find(']') {
            Some(close) => {
                out.push(&after[..close]);
                rest = &after[close + 1..];
            }
            None => break,
        }
    }
    out
}

fn list(set: &BTreeSet<String>) -> String {
    let mut shown: Vec<&str> = set
        .iter()
        .filter(|s| chemical(s))
        .map(String::as_str)
        .collect();
    shown.sort_unstable();
    shown.join(", ")
}

/// Every mechanism claim in `text` that the engine does not make.
///
/// `bound` is the reaction the text is attached to, when its trigger names
/// one. Unbound text is still checked for a rate expression, because a
/// sentence that writes out a rate law is claiming a rate law exists.
pub fn problems(bound: Option<&str>, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(id) = bound {
        if let Some(vocabulary) = vocabulary(id) {
            for (position, token) in mechanism_claims(text) {
                let n = normalise(&token);
                let allowed = if position == "is keyed on" {
                    &vocabulary.orders
                } else {
                    &vocabulary.named
                };
                if allowed.contains(&n) || !chemical(&n) {
                    continue;
                }
                problems.push(format!(
                    "says `{token}`, which `{id}` {position}: {}",
                    list(allowed)
                ));
            }
        }
    }
    problems.extend(rate_expression_problems(bound, text));
    problems
}

/// Whether this sentence is about a RATE, as a word.
///
/// A substring test is not good enough and the test below is why: a
/// sentence about a satu-RATE-d solution quoting `[Na⁺]` is a
/// concentration, not a rate law, and matching inside a word turned the
/// narrowest rule here into a false alarm.
fn states_a_rate(sentence: &str) -> bool {
    sentence
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word.eq_ignore_ascii_case("rate") || word.eq_ignore_ascii_case("rates"))
}

/// A written-out rate law must be a rate law the registry runs.
///
/// The bracket is what makes this narrow enough to be worth having: `[X]`
/// in a sentence about a rate is a claim that the law depends on X, and
/// nothing else in the prose is. `[Na⁺]` in a sentence about a saturated
/// solution is a concentration and is left alone, which is why the trigger
/// is the word `rate` and not the bracket.
fn rate_expression_problems(bound: Option<&str>, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for sentence in text.split(['.', ';']) {
        if !states_a_rate(sentence) {
            continue;
        }
        let written: Vec<String> = bracketed(sentence)
            .into_iter()
            .filter_map(|inner| candidate(inner.trim()))
            .map(|token| normalise(&token))
            .filter(|n| chemical(n))
            .collect();
        if written.is_empty() {
            continue;
        }
        let mut matched = false;
        for reaction in kinetics::REGISTRY {
            if bound.is_some_and(|id| id != reaction.id) {
                continue;
            }
            let Some(vocabulary) = vocabulary(reaction.id) else {
                continue;
            };
            if written.iter().all(|n| vocabulary.orders.contains(n)) {
                matched = true;
                break;
            }
        }
        if !matched {
            problems.push(match bound {
                Some(id) => format!(
                    "writes a rate law in `{}`, which `{id}` is not keyed on",
                    written.join(", ")
                ),
                None => format!(
                    "writes a rate law in `{}`, which is the order set of no \
                     reaction in the kinetics registry",
                    written.join(", ")
                ),
            });
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_spellings_normalise_to_registry_keys() {
        assert_eq!(normalise("HSO₃⁻"), "HSO3-");
        assert_eq!(normalise("S₂O₃²⁻"), "S2O3-2");
        assert_eq!(normalise("H⁺"), "H+");
        assert_eq!(normalise("I⁻"), "I-");
        assert_eq!(normalise("H₂O₂"), "H2O2");
        assert_eq!(normalise("KIO₃"), "KIO3");
        // Already in key spelling, and left alone.
        assert_eq!(normalise("NaHSO3"), "NaHSO3");
        assert_eq!(normalise("Mn+2"), "Mn+2");
    }

    /// The sentence that was shipped, and the sentence that replaced it.
    #[test]
    fn the_iodine_clock_sentence_that_was_false_is_caught() {
        let bad = "The iodate–bisulfite rate law is consuming NaHSO₃ — the \
                   clock endpoint is bisulfite exhaustion.";
        let found = problems(Some("iodate-bisulfite-clock"), bad);
        assert!(found.iter().any(|p| p.contains("NaHSO₃")), "{found:?}");

        let good = "The iodate–bisulfite rate law is consuming HSO₃⁻ — the \
                    bottle dissolves to the ion, and the law is keyed on the \
                    ion.";
        assert!(
            problems(Some("iodate-bisulfite-clock"), good).is_empty(),
            "{:?}",
            problems(Some("iodate-bisulfite-clock"), good)
        );
    }

    #[test]
    fn a_sentence_about_the_shelf_is_not_a_mechanism_claim() {
        // The learner really does add the bottle, so this stays true even
        // though the law is keyed on the ion. Flagging it would be the
        // noise that makes a lint worthless.
        let shelf = "NaHSO₃ is on the shelf; add NaHSO₃ and wait.";
        assert!(
            problems(Some("iodate-bisulfite-clock"), shelf).is_empty(),
            "{:?}",
            problems(Some("iodate-bisulfite-clock"), shelf)
        );
    }

    #[test]
    fn a_product_the_reaction_does_not_make_is_caught() {
        let claim = "the reaction produces Na₂SO₄";
        assert!(!problems(Some("iodate-bisulfite-clock"), claim).is_empty());
        let truth = "the reaction produces KI";
        assert!(
            problems(Some("iodate-bisulfite-clock"), truth).is_empty(),
            "{:?}",
            problems(Some("iodate-bisulfite-clock"), truth)
        );
    }

    #[test]
    fn a_rate_expression_is_checked_even_without_a_binding() {
        let wrong = "the rate law is rate = k·[H₂O₂]·[I⁻]·[H⁺]";
        assert!(!problems(None, wrong).is_empty());
        let right = "the rate law is rate = k·[H₂O₂]·[KI]·[H⁺]";
        assert!(
            problems(None, right).is_empty(),
            "{:?}",
            problems(None, right)
        );
    }

    #[test]
    fn a_concentration_that_is_not_a_rate_law_is_left_alone() {
        // `[Na⁺]` in a saturation claim is a concentration, not a claim
        // about a rate law's inputs, and the first draft of this guard
        // failed three real quest lines by not making that distinction.
        //
        // The word `saturated` CONTAINS `rate`, which is how the second
        // draft failed: the marker has to be the word, not the substring.
        let saturation = "the saturated solution reaches [Na⁺] of about 5.4 mol/L";
        assert!(
            problems(None, saturation).is_empty(),
            "{:?}",
            problems(None, saturation)
        );
    }

    #[test]
    fn a_common_name_is_outside_the_reach_and_says_so() {
        // Declared limitation, pinned so nobody later reads a passing lint
        // as a guarantee it does not give.
        let common = "the rate law is consuming bisulfite";
        assert!(problems(Some("iodate-bisulfite-clock"), common).is_empty());
    }

    #[test]
    fn a_trigger_naming_no_reaction_is_not_a_binding() {
        assert!(bound_reaction("reacted:iodate-bisulfite-clock").is_some());
        assert!(bound_reaction("reacted:landolt").is_none());
        assert!(bound_reaction("added:KI").is_none());
    }
}

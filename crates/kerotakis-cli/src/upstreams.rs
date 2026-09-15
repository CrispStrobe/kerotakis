//! Did a number in this repository come from a source we refused?
//!
//! `PLAN.md`'s "Data provenance, verified" section ends by admitting that
//! everything in it "is enforced by prose and care, which is how the codex
//! worked before `codex lint` existed — and the fix is the same fix". It then
//! names three fixes. This module is the third: the table becomes
//! `provenance/upstreams.toml`, and a lint reads it.
//!
//! The rule the same section states is narrow and worth quoting, because the
//! whole design turns on it: for a source marked avoid, **the claims stop, the
//! prose stays as commentary.** A licence does not forbid saying "NIST
//! disagrees with us by 2%". It forbids shipping NIST's number.
//!
//! ## Two lints, two questions
//!
//! `provenance/sources.toml` and [`crate::provenance`] already ask "may these
//! BYTES enter a Kerotakis artifact, and what must travel with them?" It is
//! keyed by vendored path, and it is about redistribution.
//!
//! This asks a different question: "may a NUMBER here have come from there?"
//! It is keyed by the name a provenance string uses, and it is about sourcing.
//! A value can fail this and pass that — nothing is vendored when someone
//! types a boiling point in by hand — which is exactly how seventy-nine CRC
//! Handbook citations reached `main` under a green `provenance lint`.
//!
//! ## The hard part: a claim and a comment look the same
//!
//! "CRC Handbook, 97th ed." is the same seventeen characters whether it is the
//! source of a number or a remark about one. The lint has to separate them,
//! and it cannot do it by reading the text.
//!
//! It separates them by **syntactic position**, and the repository makes the
//! case better than an argument could. In `kerotakis-core/src/nonaqueous.rs`:
//!
//! ```text
//! line  46:  source: "CRC Handbook, 97th ed.: NaCl in ethanol 0.065 g/100 mL"
//! line 597:  // CRC Handbook / Tanaka 2001 Table 1
//! ```
//!
//! Same phrase, same file. The first is a struct field sitting beside the
//! number it accounts for: it says *this value is that source's*. The second
//! is a comment: it says *here is where a reader might look*. Only the first
//! is a claim. So:
//!
//! > **A provenance string is VALUE-BOUND if it is the right-hand side of a
//! > field named `provenance`, `source`, `citation`, `attribution`,
//! > `reference` or `uncertainty` in Rust, or a `sources[].citation` in the
//! > registry export. Everything else — `//` comments, `///` doc comments,
//! > module docs, `PLAN.md`, this paragraph — is commentary and is not read.**
//!
//! That is a position test, not a content test, so it does not care how the
//! sentence is phrased and cannot be talked out of a finding by rewording.
//!
//! ### "Agrees with" is a claim
//!
//! One phrasing looked like it might be an exception and is not. Twenty-two
//! rows in `phase_route.rs` name the CRC Handbook and then add: "PENDING
//! REVIEW: the printed edition was not opened for this row, so no page-level
//! provenance is claimed; the value is the standard tabulated one and agrees
//! with NIST Chemistry WebBook SRD 69 phase-change data where that carries the
//! substance."
//!
//! The owner has ruled on this, and the ruling is the strict one: **"agrees
//! with NIST SRD 69" is a claim, not a citation, and it has to stop.** A
//! sentence in a value-bound field saying our number matches theirs is a
//! sentence asserting knowledge of their number. The lint therefore reports
//! those rows for `nist-webbook` as well as for `crc-handbook`, and it is not
//! double-counting: one string can make two claims, and both have to go.
//! Thirteen of the twenty-two were re-sourced to the vendored Apache-2.0 NASA
//! CEA file while this was being written, and the count moved by exactly that
//! much — which is the one property a figure like this has to have.
//!
//! The same sentence in a `//` comment, or in `PLAN.md`, or here, stays. That
//! is the "prose stays as commentary" half, and it is not a loophole — it is
//! the difference between discussing a source and sourcing from it.
//!
//! ## The named exception
//!
//! One case defeats the position test honestly: a value-bound string that
//! names a refused source as the LINEAGE of bytes we are separately licensed
//! to ship. `us-federal/nasa-cea-thermo-inp-v1` describes the vendored,
//! Apache-2.0 NASA CEA `thermo.inp`, and notes that CEA's own records carry
//! their literature references verbatim — several of which are JANAF. Those
//! thirty-seven heat-capacity polynomials are clean.
//!
//! Rather than weaken the rule, `upstreams.toml` carries an `[[excuse]]` list:
//! subject, upstream, and a written reason. Every excuse must MATCH something;
//! an excuse that covers nothing is reported as a problem, so a stale one
//! cannot sit there looking like diligence after the string it covered is
//! gone. The excuse list is exactly as strong as its review — it is here
//! because it is visible, counted and expiring, which prose was not.
//!
//! ## What it catches
//!
//! A value-bound provenance string, anywhere in `crates/**/*.rs` or in the
//! registry export's source citations, that names an upstream whose verdict is
//! `avoid` or `permission-required`, and that no excuse covers.
//!
//! ## What it does NOT catch
//!
//! Stated in full, because the two gates this repository shipped before this
//! one both read wider than they were — a test grid that stopped where its
//! solver stopped converging, and a prose lint whose headline rule reached one
//! file in thirty-six.
//!
//! - **A value with no provenance string at all.** This is the big one and it
//!   is unbounded. The lint reads attributions; it cannot see a constant that
//!   was never attributed. It therefore proves nothing about the numbers it
//!   does not find, and a drop in the count is as easily a deleted citation as
//!   a re-sourced value. Pair it with `kerotakis-data`'s rule that every
//!   numeric record names a resolvable source id — that rule covers the
//!   registry, and nothing yet covers the Rust constants.
//! - **A value attributed to the wrong source.** If a number really came from
//!   the WebBook and the string says "standard reference values", the lint
//!   passes and is wrong. It checks what a string SAYS, never where a number
//!   came from. A laundered citation is invisible to it by construction, and
//!   no lint of this shape can fix that.
//! - **Which value in a shared citation is the offending one.** The registry
//!   surface is coarse: `legacy/HBr` reads "M from IUPAC/CIAAW 2021 atomic
//!   weights; Cp(g) NIST-JANAF; density at STP, CRC Handbook 97th ed." — three
//!   sources, one string, and fifteen numeric records hanging off it. The lint
//!   condemns the citation, which is right, and cannot say that the molar mass
//!   is fine, which it is. Counting source citations rather than records is
//!   the deliberate consequence: one citation is one finding.
//! - **Spellings not in `names`.** "the Handbook", "the standard tables",
//!   "Lide", an ISBN, a bare DOI — all invisible. `names` is a list somebody
//!   maintains; it is not a classifier, and every row is one unlisted synonym
//!   away from a false negative. Two live consequences, both deliberate:
//!   - `unifac-consortium` lists only consortium spellings (`DDBST`, `Dortmund
//!     Data Bank`), because PLAN.md refuses the consortium tables and PERMITS
//!     the original journal tables, and a bare "UNIFAC" does not say which.
//!     `cli/src/diagram.rs` cites "UNIFAC γ(x, T) (Fredenslund 1975
//!     parameters)" — the journal route, correctly not flagged. A string that
//!     really did come from the consortium tables and said only "UNIFAC" would
//!     also not be flagged. The narrowness cuts both ways and is a judgement,
//!     not an oversight.
//!   - `thermo-python` lists only `CalebBell/thermo`, because a bare "thermo"
//!     matches half the vocabulary of thermodynamics. The consequence is that
//!     the oracle-only tally below currently reads zero while `diagram.rs`
//!     does name the Python `thermo` package as its cross-check. A zero there
//!     means "nothing matched", not "nothing happened".
//! - **Strings built by `concat!`, `format!` or constants.** The scanner reads
//!   one literal after a field name. A citation assembled at run time is not
//!   seen. (Adjacent-literal continuation with a trailing `\` IS handled.)
//! - **Non-Rust, non-registry surfaces.** Codex TOML, quest files, lesson step
//!   prose, the web payload, `PLAN.md` itself. Adding a surface is adding a
//!   scanner; the verdict table does not change.
//! - **Whether the verdicts are right.** `upstreams.toml` records a reading of
//!   terms pages on a date. It is not legal advice and it goes stale; the
//!   `retrieved` field is there so that staleness is visible rather than
//!   assumed away.
//! - **Whether an excuse is honest.** The lint checks that an excuse still
//!   MATCHES a real string and reports it if it does not. It cannot check that
//!   the reason is true. An excuse is a human judgement recorded in data, and
//!   its whole advantage over the prose it replaces is that it is named,
//!   counted, printed on every run, and expires loudly when the string it
//!   covered changes. That is a smaller claim than "reviewed", and it is the
//!   one being made.
//! - **The difference between a withdrawal and a claim.** The nine excused
//!   registry matches are the sharp case. A citation that says "the claim
//!   resting on the CRC Handbook has been withdrawn" is commentary, and a
//!   citation that says "from the CRC Handbook" is a claim, and both are the
//!   value of the same `citation` field. The position test cannot separate
//!   them; only the excuse list does, one reviewed row at a time. If
//!   withdrawal notices become common this does not scale, and the fix is a
//!   structured field — see below.
//!
//! ## The field that would make this exact, which does not exist
//!
//! Every limit above with real teeth comes from the same root: a registry
//! source has ONE free-text `citation` and many numeric records hang off it,
//! so the lint reasons about sentences where it should be reasoning about
//! links. `legacy/HBr` reads "M from IUPAC/CIAAW 2021 atomic weights; Cp(g)
//! NIST-JANAF; density at STP, CRC Handbook 97th ed." — the molar mass is
//! clean, the heat capacity is not, and nothing in the data says which is
//! which.
//!
//! The fix is a structured field on the source record, proposed here rather
//! than added, because `data/registry/registry-source-v1.json` and its
//! byte-exact golden mirror belong to another change in flight:
//!
//! ```text
//! "upstreams": [
//!   { "id": "ciawa-2021",   "role": "claims", "covers": ["molar_mass"] },
//!   { "id": "nist-janaf",   "role": "mentioned" },
//!   { "id": "crc-handbook", "role": "withdrawn", "on": "2026-09-13" }
//! ]
//! ```
//!
//! With `role` present the lint stops guessing: `claims` is a claim,
//! `mentioned` and `withdrawn` are not, the excuse list disappears, and the
//! finding moves from the citation to the individual quantity — which is also
//! the unit the accuracy work wants to count. Until then the excuse list is
//! the honest stand-in, and this paragraph is the record of what it stands in
//! for.
//!
//! ## What must be true before this becomes a gate
//!
//! Four things, in order:
//!
//! 1. **The count reaches zero** on both surfaces, by re-sourcing or by
//!    withdrawal, not by deleting citations — watch the denominators.
//! 2. **The 105 unjudged registry citations shrink.** A gate over a surface
//!    where more than half the rows name no audited source at all is a gate
//!    with a hole bigger than itself.
//! 3. **`names` lists are reviewed once more against the tree**, since a
//!    forward guard that matches nothing is indistinguishable from one that
//!    works. Seven of the eleven refused rows currently match nothing.
//! 4. **`--fail` is added in `tools/preflight.sh`** and the CI job that runs
//!    it, in a pull request that does nothing else, so that the promotion is
//!    revertible on its own.
//!
//! ## How much it actually reaches, in numbers
//!
//! On `main` at 8c2d03f6, the day this landed:
//!
//! - **210** value-bound provenance strings exist in `crates/**/*.rs`. That is
//!   the entire Rust surface — the denominator, not a sample. **98** of them
//!   name a refused upstream: 79 the CRC Handbook, 18 the NIST WebBook, 1
//!   JANAF. They sit in five files, and 65 of the 79 are one table in
//!   `nonaqueous.rs`.
//! - **185** source citations exist in the registry export. **59** name a
//!   refused upstream: 46 CRC, 6 Merck, 4 WebBook, 3 JANAF.
//! - **10** further matches are excused by a reviewed `[[excuse]]` row and
//!   printed separately: one NASA CEA lineage mention, and nine across the
//!   three tranches whose claims were withdrawn on 2026-09-13, whose citations
//!   name the refused sources in order to say the claim has stopped.
//! - The two surfaces are reported apart and never added into a claim about
//!   distinct values, because `kerotakis-registry-export` generates part of
//!   the registry from the same Rust constants: one sourcing error can appear
//!   on both. **157** is the number of findings; it is not 157 different
//!   numbers.
//! - **105** of the 185 registry citations name NO audited upstream at all,
//!   refused or cleared. They are not clean; they are unjudged, most saying
//!   "standard reference values" without saying whose. The lint prints that
//!   number every run, because it is the size of its own blind spot and it is
//!   nearly twice its finding.
//! - **11** of the 22 audited sources are refused and **2** carry an open
//!   licence question. Only four of the eleven are named anywhere in the tree
//!   (CRC, Merck, WebBook, JANAF). The other seven — Cantera, CAS Common
//!   Chemistry, CAMEO/CRW4, ECHA, Burcat, `chemicals`, the UNIFAC consortium —
//!   match nothing. Those rows are a forward guard, not a finding, and a
//!   forward guard is only as good as its `names` list.
//!
//! ### Addendum, 2026-09-15: the audit table was repaired, and what moved
//!
//! The figures above are left as they were measured, because they are dated.
//! What changed is the TABLE, not the tree: `docs/registry-unattributed-census.md`
//! separated the 105 and found that 100 of the 487 numeric records under them
//! name a real source this file could not spell. Eight rows were added and one
//! `names` list widened. Measured on the same two surfaces afterwards:
//!
//! - **Blind spot 105 -> 6** of 185 registry citations. Item 2 of the
//!   preconditions below is met. The six are the floor the census predicted:
//!   one citation that is derived by construction, three bleaching products
//!   defined by stoichiometry from a parent, and two awaiting a source that
//!   does not exist yet.
//! - **Findings 59 -> 60** on the registry. The one new finding is
//!   `legacy/amylase` under a textbook row; that is the count getting more
//!   honest rather than worse, and the census predicted it exactly. The Rust
//!   surface reads 20 of 210 today and this repair moved neither number - the
//!   98 above is from before the Rust re-sourcing sweep landed, not a
//!   disagreement with it.
//! - **Open questions 0 -> 152** on the registry and 2 on Rust, the first
//!   non-zero this column has ever printed. 152 citations name the atomic-weight
//!   body, whose terms grant educational reuse and reserve commercial use, and
//!   nobody has asked the question that settles it. Nothing got worse; a
//!   dependence became visible.
//! - **22 -> 30 audited sources**, 12 refused, 5 carrying an open question.
//!
//! Two defects this file still has, both found while measuring and neither
//! fixed here, because each is a row somebody has to judge:
//!
//! - **`nist-janaf` condemns the public-domain 1971 edition.** Two Rust
//!   findings, `phase_route.rs` NaCl and KCl, cite NSRDS-NBS 37 (1971), read
//!   from nvlpubs.nist.gov, bearing no copyright notice. The `nist-janaf` note
//!   says in terms that this edition "ARE public domain but dated" and "needs
//!   its own row, not this one" - and until it gets one, the lint reports two
//!   clean values as offences.
//! - **Majer & Svoboda still has no row**, and `phase_route.rs`'s ethanol
//!   citation names it. See `PLAN.md`'s scoped task: its terms cannot be read.
//!
//! ### Why the figure is a count and not a rate
//!
//! The headline is an absolute number of findings, never "N% clean". A rate
//! falls when somebody adds ten easy well-sourced rows, and nothing has been
//! fixed; a count falls only when a finding is removed. The denominators are
//! printed beside it for exactly one reason: the count can ALSO be driven down
//! by deleting a citation rather than re-sourcing a value, and a denominator
//! that falls alongside the count is what that looks like. Watch both.
//!
//! ## Reporting, not failing
//!
//! It exits zero. There are ninety-four findings on the Rust surface and
//! another agent is removing them; a gate that failed today would block every
//! unrelated change and land on that agent's head. `--fail` (or
//! `KERO_PROVENANCE_UPSTREAMS_FAIL=1`) makes it exit non-zero, and that is the
//! switch the promotion PR flips once the count is zero. Problems with
//! `upstreams.toml` ITSELF always fail — that file is not in anyone's way.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;

const SCHEMA_VERSION: u32 = 1;
const POLICY: &str = "scientific-upstream-audit-v1";

/// The field names whose string value is a claim about where a number came
/// from. A `//` comment carrying the same words is commentary and is skipped;
/// see the module docs for why that line is where it is.
const VALUE_BOUND_FIELDS: &[&str] = &[
    "provenance",
    "source",
    "citation",
    "attribution",
    "reference",
    "uncertainty",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Audit {
    schema: u32,
    policy: String,
    #[serde(default)]
    checked: String,
    #[serde(default, rename = "upstream")]
    upstreams: Vec<Upstream>,
    #[serde(default, rename = "excuse")]
    excuses: Vec<Excuse>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upstream {
    id: String,
    name: String,
    licence: String,
    terms: String,
    #[serde(default)]
    retrieved: String,
    verdict: Verdict,
    #[serde(default)]
    may_touch: Vec<String>,
    names: Vec<String>,
    #[serde(default)]
    in_plan_table: bool,
    #[serde(default)]
    note: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Excuse {
    surface: Surface,
    /// A registry source id, or a repository-relative path for `rust`.
    subject: String,
    /// Every refused upstream this one string names as commentary. A
    /// withdrawal notice typically names all of them at once, because it is
    /// quoting the rule it is obeying.
    upstreams: Vec<String>,
    reason: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "kebab-case")]
enum Surface {
    Rust,
    Registry,
}

impl Surface {
    fn label(self) -> &'static str {
        match self {
            Surface::Rust => "rust",
            Surface::Registry => "registry",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Verdict {
    Primary,
    Supplement,
    OracleOnly,
    Avoid,
    PermissionRequired,
    DecisionRequired,
}

impl Verdict {
    /// A verdict under which a value may NOT be claimed. `permission-required`
    /// joins `avoid` because "we could ask" is not "we asked and they said
    /// yes", and the value is in the binary either way.
    fn refuses_claims(self) -> bool {
        matches!(self, Verdict::Avoid | Verdict::PermissionRequired)
    }

    /// May a SHIPPED number cite this source?
    ///
    /// The vocabulary is deliberately three-way, because the repository's
    /// sourcing policy is: individually cited measurements first, build-time
    /// oracles as a fallback. Those are different permissions and a two-state
    /// allow/deny cannot hold both.
    ///
    /// - `primary`, `supplement` — yes. A shipped value may cite this.
    /// - `oracle-only` — no. It may generate a test fixture or check a number
    ///   we derived ourselves; it may not BE the number. `thermo` (Python) and
    ///   the CRD reaction corpus sit here.
    /// - `avoid`, `permission-required`, `decision-required` — no, and the
    ///   source may only be MENTIONED. That is the whole rule: the claims
    ///   stop, the prose stays as commentary.
    fn may_support_a_shipped_claim(self) -> bool {
        matches!(self, Verdict::Primary | Verdict::Supplement)
    }

    /// May the source be named at all in a value-bound field, even if the
    /// number itself came from somewhere else? Only for build-time oracles,
    /// where naming it is how a fixture records what checked the value.
    fn is_oracle_only(self) -> bool {
        matches!(self, Verdict::OracleOnly)
    }

    fn label(self) -> &'static str {
        match self {
            Verdict::Primary => "primary",
            Verdict::Supplement => "supplement",
            Verdict::OracleOnly => "oracle-only",
            Verdict::Avoid => "avoid",
            Verdict::PermissionRequired => "permission-required",
            Verdict::DecisionRequired => "decision-required",
        }
    }
}

/// One value-bound provenance string that names a refused upstream.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Finding {
    surface: Surface,
    /// File path for `rust`, source id for `registry`.
    subject: String,
    /// Line number, or 0 for the registry surface.
    line: usize,
    upstream: String,
    verdict: &'static str,
    excerpt: String,
}

/// What a scan found, per surface.
#[derive(Debug, Default)]
pub(crate) struct Report {
    pub findings: Vec<Finding>,
    /// Value-bound strings examined, per surface. The denominator.
    pub scanned: BTreeMap<&'static str, usize>,
    /// Registry citations naming no audited upstream at all.
    pub unjudged: usize,
    /// Value-bound strings naming a build-time oracle. Not an offence — an
    /// oracle may be named as what CHECKED a number — but counted, because an
    /// oracle quietly becoming the number's source is the fallback path going
    /// wrong and nothing else would see it.
    pub oracle_mentions: BTreeMap<String, usize>,
    /// Value-bound strings naming a source with an open licence question.
    /// Reported apart from the headline: a count that mixed "refused" with
    /// "not yet asked" would read stronger than it is.
    pub open_questions: BTreeMap<String, usize>,
    /// Excuses that matched a real string, by index into `Audit::excuses`.
    used_excuses: BTreeSet<usize>,
    /// How many findings each excuse silenced, so that a reader of the report
    /// can see what was excused rather than only what was counted.
    pub excused: BTreeMap<String, usize>,
}

// ---------------------------------------------------------------- matching --

/// Case-insensitive, whole-word-anchored containment.
///
/// The anchoring is not decoration. An unanchored `"ECHA"` matches
/// `"mechanism"`, and it did: the first draft of this lint reported
/// `legacy/CO` and `legacy/Fe+2` as ECHA offenders on the strength of the
/// phrases "the gas mechanism packs" and "its mechanism". Both are clean.
///
/// A boundary is any position where the neighbouring character is not
/// alphanumeric. `.`, `-`, `/` and `(` are therefore boundaries, which is what
/// lets `webbook.nist.gov` match inside a URL and `CRC Handbook` match before
/// a comma.
fn names_source(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let hay: Vec<char> = haystack.chars().collect();
    let pin: Vec<char> = needle.chars().collect();
    if pin.len() > hay.len() {
        return false;
    }
    let lower = |c: char| c.to_lowercase().next().unwrap_or(c);
    for start in 0..=(hay.len() - pin.len()) {
        if !(0..pin.len()).all(|i| lower(hay[start + i]) == lower(pin[i])) {
            continue;
        }
        let before_ok = start == 0 || !hay[start - 1].is_alphanumeric();
        let end = start + pin.len();
        let after_ok = end == hay.len() || !hay[end].is_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// Collapse whitespace so a citation wrapped across source lines still reads
/// as one sentence. Rust's `\`-at-end-of-line continuation leaves the newline
/// and the following indentation inside the literal.
fn flatten(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            space = true;
        } else {
            if space && !out.is_empty() {
                out.push(' ');
            }
            space = false;
            out.push(ch);
        }
    }
    out
}

fn excerpt(text: &str) -> String {
    let flat = flatten(text);
    let chars: Vec<char> = flat.chars().collect();
    if chars.len() <= 96 {
        flat
    } else {
        format!("{}…", chars[..96].iter().collect::<String>())
    }
}

// ---------------------------------------------------------------- scanning --

/// A value-bound provenance string found in Rust source.
#[derive(Debug)]
struct Bound {
    line: usize,
    text: String,
}

/// Find every `<field>: "<literal>"` in Rust source, skipping comments.
///
/// Deliberately a scanner and not a parser. It tracks line comments, block
/// comments and string literals well enough to know which is which, which is
/// the only structural fact the rule needs. It does not resolve `concat!`,
/// `format!` or constants — see the module docs.
fn scan_rust(text: &str) -> Vec<Bound> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let mut line = 1usize;
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        // Comments: skipped entirely. This is the claim/commentary split.
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if ch == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1usize;
            while i < chars.len() && depth > 0 {
                if chars[i] == '\n' {
                    line += 1;
                }
                if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                    continue;
                }
                if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                    continue;
                }
                i += 1;
            }
            continue;
        }
        if ch == '"' {
            // A string literal not preceded by a field name: skip it whole, so
            // that a citation quoted INSIDE another string cannot be read as a
            // field of its own.
            let (end, lines, _) = read_string(&chars, i);
            line += lines;
            i = end;
            continue;
        }
        if !ch.is_alphabetic() && ch != '_' {
            i += 1;
            continue;
        }
        // An identifier. Is it one of our field names, followed by `:` then a
        // string literal?
        let start = i;
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }
        let ident: String = chars[start..i].iter().collect();
        if !VALUE_BOUND_FIELDS.contains(&ident.as_str()) {
            continue;
        }
        let mut j = i;
        while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t') {
            j += 1;
        }
        // `source_id:` is a different field and was already excluded by the
        // identifier scan; `source :` with a space is accepted.
        if j >= chars.len() || chars[j] != ':' {
            continue;
        }
        j += 1;
        let mut newlines = 0usize;
        while j < chars.len() && chars[j].is_whitespace() {
            if chars[j] == '\n' {
                newlines += 1;
            }
            j += 1;
        }
        if j >= chars.len() || chars[j] != '"' {
            continue;
        }
        let (end, lines, body) = read_string(&chars, j);
        found.push(Bound {
            line: line + newlines,
            text: body,
        });
        line += newlines + lines;
        i = end;
    }
    found
}

/// Read a `"`-delimited literal starting at `chars[at]`.
///
/// Returns the index just past the closing quote, the number of newlines
/// consumed, and the body with `\`-escapes resolved well enough to match on.
fn read_string(chars: &[char], at: usize) -> (usize, usize, String) {
    let mut i = at + 1;
    let mut lines = 0usize;
    let mut body = String::new();
    while i < chars.len() {
        let ch = chars[i];
        if ch == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            match next {
                'n' | 't' | 'r' => body.push(' '),
                '\n' => {
                    lines += 1;
                    body.push(' ');
                }
                other => body.push(other),
            }
            i += 2;
            continue;
        }
        if ch == '"' {
            return (i + 1, lines, body);
        }
        if ch == '\n' {
            lines += 1;
        }
        body.push(ch);
        i += 1;
    }
    (i, lines, body)
}

// ------------------------------------------------------------------- audit --

impl Audit {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|error| format!("could not parse audit: {error}"))
    }

    /// Problems with the audit FILE. These always fail, reporting mode or not:
    /// the file is this change's own, and nothing else is waiting on it.
    pub(crate) fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.schema != SCHEMA_VERSION {
            problems.push(format!(
                "schema: expected {SCHEMA_VERSION}, found {}",
                self.schema
            ));
        }
        if self.policy != POLICY {
            problems.push(format!(
                "policy: expected '{POLICY}', found '{}'",
                self.policy
            ));
        }
        if self.checked.trim().is_empty() {
            problems.push("checked: the audit must say when it was checked".to_string());
        }
        let mut ids = BTreeSet::new();
        for upstream in &self.upstreams {
            let id = &upstream.id;
            if id.trim().is_empty() {
                problems.push("upstream: empty id".to_string());
                continue;
            }
            if !ids.insert(id.clone()) {
                problems.push(format!("upstream '{id}': duplicate id"));
            }
            for (field, value) in [
                ("name", &upstream.name),
                ("licence", &upstream.licence),
                ("terms", &upstream.terms),
                ("note", &upstream.note),
            ] {
                if value.trim().is_empty() {
                    problems.push(format!("upstream '{id}': {field} must not be empty"));
                }
            }
            if upstream.names.iter().all(|n| n.trim().is_empty()) {
                problems.push(format!(
                    "upstream '{id}': names must carry at least one spelling, or the lint cannot see it"
                ));
            }
            // The rule, as a structural invariant: a refused row may not have a
            // surface it is allowed to touch.
            if upstream.verdict.refuses_claims() && !upstream.may_touch.is_empty() {
                problems.push(format!(
                    "upstream '{id}': verdict is {} but may_touch lists {:?}",
                    upstream.verdict.label(),
                    upstream.may_touch
                ));
            }
            if (upstream.verdict.may_support_a_shipped_claim() || upstream.verdict.is_oracle_only())
                && upstream.may_touch.is_empty()
            {
                problems.push(format!(
                    "upstream '{id}': verdict is {} but may_touch is empty — say what it may touch",
                    upstream.verdict.label()
                ));
            }
            // A build-time oracle may check a number; it may not BE one. If a
            // row says oracle-only and then lists a shipping surface, the
            // verdict and the permission disagree and the verdict is the one
            // that was reviewed.
            if upstream.verdict.is_oracle_only()
                && upstream
                    .may_touch
                    .iter()
                    .any(|surface| !matches!(surface.as_str(), "build-oracle" | "template-mining"))
            {
                problems.push(format!(
                    "upstream '{id}': oracle-only may touch only build-oracle/template-mining, not {:?}",
                    upstream.may_touch
                ));
            }
            // A row inherited from PLAN.md's table asserts a date the terms
            // were read. A row proposed HERE deliberately has none, and says so.
            if upstream.in_plan_table && upstream.retrieved.trim().is_empty() {
                problems.push(format!(
                    "upstream '{id}': in_plan_table rows must carry the date their terms were read"
                ));
            }
        }
        for (index, excuse) in self.excuses.iter().enumerate() {
            if excuse.upstreams.is_empty() {
                problems.push(format!(
                    "excuse[{index}] for '{}': lists no upstream",
                    excuse.subject
                ));
            }
            for named in &excuse.upstreams {
                if !ids.contains(named) {
                    problems.push(format!("excuse[{index}]: unknown upstream '{named}'"));
                } else if let Some(up) = self.upstreams.iter().find(|u| &u.id == named) {
                    if !up.verdict.refuses_claims() {
                        problems.push(format!(
                            "excuse[{index}]: upstream '{}' is {}, which refuses nothing — the excuse excuses nothing",
                            up.id,
                            up.verdict.label()
                        ));
                    }
                }
            }
            if excuse.reason.trim().len() < 40 {
                problems.push(format!(
                    "excuse[{index}] for '{}': reason must say WHY the mention is commentary, in a sentence",
                    excuse.subject
                ));
            }
        }
        problems
    }

    fn excuse_for(&self, surface: Surface, subject: &str, upstream: &str) -> Option<usize> {
        self.excuses.iter().position(|e| {
            e.surface == surface
                && e.subject == subject
                && e.upstreams.iter().any(|u| u == upstream)
        })
    }

    /// Every upstream that `text` names, refused or not.
    fn named_by(&self, text: &str) -> Vec<&Upstream> {
        self.upstreams
            .iter()
            .filter(|u| u.names.iter().any(|n| names_source(text, n)))
            .collect()
    }

    /// Scan the Rust surface and the registry export.
    pub(crate) fn scan(&self, root: &Path) -> Report {
        let mut report = Report::default();
        let mut rust_files: Vec<std::path::PathBuf> = Vec::new();
        collect_rust(&root.join("crates"), &mut rust_files);
        rust_files.sort();
        let mut rust_scanned = 0usize;
        for path in rust_files {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            for bound in scan_rust(&text) {
                rust_scanned += 1;
                let flat = flatten(&bound.text);
                for upstream in self.named_by(&flat) {
                    if !upstream.verdict.refuses_claims() {
                        if upstream.verdict.is_oracle_only() {
                            *report
                                .oracle_mentions
                                .entry(upstream.id.clone())
                                .or_default() += 1;
                        } else if upstream.verdict == Verdict::DecisionRequired {
                            *report
                                .open_questions
                                .entry(upstream.id.clone())
                                .or_default() += 1;
                        }
                        continue;
                    }
                    if let Some(index) = self.excuse_for(Surface::Rust, &relative, &upstream.id) {
                        report.used_excuses.insert(index);
                        *report
                            .excused
                            .entry(format!("{relative} / {}", upstream.id))
                            .or_default() += 1;
                        continue;
                    }
                    report.findings.push(Finding {
                        surface: Surface::Rust,
                        subject: relative.clone(),
                        line: bound.line,
                        upstream: upstream.id.clone(),
                        verdict: upstream.verdict.label(),
                        excerpt: excerpt(&bound.text),
                    });
                }
            }
        }
        report.scanned.insert("rust", rust_scanned);

        let registry = root.join("data/registry/registry-source-v1.json");
        let mut registry_scanned = 0usize;
        if let Ok(text) = std::fs::read_to_string(&registry) {
            if let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(sources) = doc.get("sources").and_then(|s| s.as_array()) {
                    for source in sources {
                        let id = source.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let citation = source
                            .get("citation")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        registry_scanned += 1;
                        let flat = flatten(citation);
                        let named = self.named_by(&flat);
                        if named.is_empty() {
                            report.unjudged += 1;
                        }
                        for upstream in named {
                            if !upstream.verdict.refuses_claims() {
                                if upstream.verdict.is_oracle_only() {
                                    *report
                                        .oracle_mentions
                                        .entry(upstream.id.clone())
                                        .or_default() += 1;
                                } else if upstream.verdict == Verdict::DecisionRequired {
                                    *report
                                        .open_questions
                                        .entry(upstream.id.clone())
                                        .or_default() += 1;
                                }
                                continue;
                            }
                            if let Some(index) =
                                self.excuse_for(Surface::Registry, id, &upstream.id)
                            {
                                report.used_excuses.insert(index);
                                *report
                                    .excused
                                    .entry(format!("{id} / {}", upstream.id))
                                    .or_default() += 1;
                                continue;
                            }
                            report.findings.push(Finding {
                                surface: Surface::Registry,
                                subject: id.to_string(),
                                line: 0,
                                upstream: upstream.id.clone(),
                                verdict: upstream.verdict.label(),
                                excerpt: excerpt(citation),
                            });
                        }
                    }
                }
            }
        }
        report.scanned.insert("registry", registry_scanned);
        report.findings.sort();
        report
    }

    /// An excuse that covers nothing is a claim of diligence with nothing
    /// behind it, so it expires loudly rather than quietly.
    fn stale_excuses(&self, report: &Report) -> Vec<String> {
        self.excuses
            .iter()
            .enumerate()
            .filter(|(index, _)| !report.used_excuses.contains(index))
            .map(|(index, excuse)| {
                format!(
                    "excuse[{index}]: '{}' / {:?} on the {} surface matched nothing — delete it or fix its subject",
                    excuse.subject,
                    excuse.upstreams,
                    excuse.surface.label()
                )
            })
            .collect()
    }
}

fn collect_rust(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "target" || name == "vendor" || name.starts_with('.') {
                continue;
            }
            collect_rust(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

// ----------------------------------------------------------------- command --

pub(crate) fn upstreams_command(audit_path: &str, root: &str, fail: bool) -> ! {
    let text = std::fs::read_to_string(audit_path).unwrap_or_else(|error| {
        eprintln!("kero provenance upstreams: cannot read {audit_path}: {error}");
        std::process::exit(1);
    });
    let audit = Audit::parse(&text).unwrap_or_else(|error| {
        eprintln!("kero provenance upstreams: {audit_path}: {error}");
        std::process::exit(1);
    });
    let problems = audit.problems();
    if !problems.is_empty() {
        for problem in &problems {
            eprintln!("kero provenance upstreams: {audit_path}: {problem}");
        }
        eprintln!(
            "kero provenance upstreams: {} problem{} in the audit itself",
            problems.len(),
            if problems.len() == 1 { "" } else { "s" }
        );
        std::process::exit(1);
    }

    let report = audit.scan(Path::new(root));
    let stale = audit.stale_excuses(&report);
    for problem in &stale {
        eprintln!("kero provenance upstreams: {audit_path}: {problem}");
    }

    let refused: Vec<&Upstream> = audit
        .upstreams
        .iter()
        .filter(|u| u.verdict.refuses_claims())
        .collect();
    let open: Vec<&Upstream> = audit
        .upstreams
        .iter()
        .filter(|u| u.verdict == Verdict::DecisionRequired)
        .collect();

    println!(
        "provenance upstreams: {} audited sources ({} refused, {} open questions), checked {}",
        audit.upstreams.len(),
        refused.len(),
        open.len(),
        audit.checked
    );

    // Per surface, then per upstream, then the strings. A count with no names
    // beside it cannot be worked down, and working it down is the point.
    for surface in [Surface::Rust, Surface::Registry] {
        let here: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.surface == surface)
            .collect();
        let scanned = report.scanned.get(surface.label()).copied().unwrap_or(0);
        println!(
            "\n  {} surface: {} of {} value-bound provenance strings name a refused upstream",
            surface.label(),
            here.len(),
            scanned
        );
        let mut by_upstream: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
        for finding in &here {
            by_upstream
                .entry(finding.upstream.as_str())
                .or_default()
                .push(finding);
        }
        for (upstream, findings) in &by_upstream {
            println!(
                "    {upstream} ({}): {}",
                findings[0].verdict,
                findings.len()
            );
            let mut by_subject: BTreeMap<&str, usize> = BTreeMap::new();
            for finding in findings {
                *by_subject.entry(finding.subject.as_str()).or_default() += 1;
            }
            for (subject, count) in &by_subject {
                println!("      {count:4}  {subject}");
            }
        }
        if surface == Surface::Registry && report.unjudged > 0 {
            println!(
                "    (blind spot: {} of {scanned} citations name no audited source at all — \
                 unjudged, not clean)",
                report.unjudged
            );
        }
    }

    if !report.excused.is_empty() {
        let total: usize = report.excused.values().sum();
        println!(
            "\n  excused by a reviewed [[excuse]] row ({total}) — named as commentary, not claimed:"
        );
        for (what, count) in &report.excused {
            println!("    {count:4}  {what}");
        }
    }

    if !report.oracle_mentions.is_empty() || !report.open_questions.is_empty() {
        println!("\n  named but not refused (reported apart from the headline):");
        for (id, count) in &report.oracle_mentions {
            println!("    {id} (oracle-only): {count} — may CHECK a number, never BE one");
        }
        for (id, count) in &report.open_questions {
            println!("    {id} (decision-required): {count} — licence question still open");
        }
    }

    let total = report.findings.len();
    println!();
    if total == 0 && stale.is_empty() {
        println!(
            "provenance upstreams ok: no value-bound provenance string names a refused source"
        );
        std::process::exit(0);
    }
    let denominator: usize = report.scanned.values().sum();
    println!(
        "provenance upstreams: {total} finding{} over {denominator} value-bound provenance \
         strings examined.",
        if total == 1 { "" } else { "s" }
    );
    println!(
        "provenance upstreams: the figure to watch to zero is {total}. It is a COUNT, not a \
         rate, so adding well-sourced rows cannot move it; the surfaces overlap, so it is a \
         number of findings and not of distinct values; and if {denominator} falls alongside \
         it, citations are being deleted rather than values re-sourced."
    );
    let hard_fail = fail || std::env::var("KERO_PROVENANCE_UPSTREAMS_FAIL").is_ok_and(|v| v == "1");
    if hard_fail {
        eprintln!("provenance upstreams: failing, because --fail was asked for");
        std::process::exit(1);
    }
    println!(
        "provenance upstreams: REPORTING ONLY (exit 0). Pass --fail, or set \
         KERO_PROVENANCE_UPSTREAMS_FAIL=1, to make this a gate."
    );
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures build their field-and-literal text with `format!` rather
    // than writing it out, so that running this lint over the repository does
    // not count its own tests as findings. The lint reading itself would be
    // the exact "reads stronger than it is" failure the module docs are about.
    fn bound(field: &str, body: &str) -> String {
        format!("Row {{ value: 1.0, {field}: \"{body}\" }}")
    }

    fn audit() -> Audit {
        Audit::parse(
            r#"
schema = 1
policy = "scientific-upstream-audit-v1"
checked = "2026-08-18"

[[upstream]]
id = "nist-webbook"
name = "NIST Chemistry WebBook"
licence = "LicenseRef-NIST-SRD-Permission-Required"
terms = "https://webbook.nist.gov/chemistry/copyrght/"
retrieved = "2026-08-18"
verdict = "permission-required"
may_touch = []
names = ["NIST Chemistry WebBook", "webbook.nist.gov"]
in_plan_table = true
note = "NIST SRD, permission required."

[[upstream]]
id = "echa-cl"
name = "ECHA C&L exports"
licence = "LicenseRef-IP-Encumbered"
terms = "https://echa.europa.eu/legal-notice"
retrieved = "2026-08-18"
verdict = "avoid"
may_touch = []
names = ["ECHA"]
in_plan_table = true
note = "IP-encumbered."

[[upstream]]
id = "pubchem"
name = "PubChem"
licence = "LicenseRef-US-Public-Domain"
terms = "https://www.ncbi.nlm.nih.gov/home/about/policies/"
retrieved = "2026-08-18"
verdict = "primary"
may_touch = ["identity"]
names = ["PubChem"]
in_plan_table = true
note = "Cleared."
"#,
        )
        .expect("fixture parses")
    }

    #[test]
    fn the_shipped_audit_is_well_formed() {
        let text = std::fs::read_to_string("../../provenance/upstreams.toml")
            .expect("provenance/upstreams.toml is present");
        let audit = Audit::parse(&text).expect("the shipped audit parses");
        assert_eq!(audit.problems(), Vec::<String>::new());
        // The rows the rule exists for must actually be refused, or the lint
        // is a no-op that looks like a gate.
        for id in ["nist-webbook", "nist-janaf", "cameo-crw4", "cantera-data"] {
            let row = audit
                .upstreams
                .iter()
                .find(|u| u.id == id)
                .unwrap_or_else(|| panic!("{id} is audited"));
            assert!(row.verdict.refuses_claims(), "{id} must refuse claims");
            assert!(row.may_touch.is_empty(), "{id} may touch nothing");
        }
    }

    #[test]
    fn a_word_boundary_is_what_separates_echa_from_mechanism() {
        // The regression this whole matcher exists for: an unanchored "ECHA"
        // reported `legacy/CO` because its citation says "the gas mechanism
        // packs deposit CO by name".
        assert!(!names_source("the gas mechanism packs deposit CO", "ECHA"));
        assert!(!names_source(
            "colour and its mechanism - a weak field",
            "ECHA"
        ));
        assert!(names_source("ECHA C&L exports are IP-encumbered", "ECHA"));
        assert!(names_source("avoid ECHA, use EUR-Lex", "ECHA"));
        // Punctuation is a boundary, so URLs and comma-terminated phrases work.
        assert!(names_source(
            "phase-change data, https://webbook.nist.gov/cgi/cbook.cgi?ID=C64175",
            "webbook.nist.gov"
        ));
        assert!(names_source("CRC Handbook, 97th ed.", "CRC Handbook"));
        // Case-insensitive, but still anchored.
        assert!(names_source("crc handbook of chemistry", "CRC Handbook"));
        assert!(!names_source("NONECHAFOO", "ECHA"));
    }

    #[test]
    fn a_comment_is_commentary_and_a_field_is_a_claim() {
        // This is the rule, stated as a test. Both lines below carry the same
        // phrase; `nonaqueous.rs` really does carry both.
        let claim = bound(
            "source",
            "CRC Handbook, 97th ed.: NaCl in ethanol 0.065 g/100 mL",
        );
        let comment = "// CRC Handbook / Tanaka 2001 Table 1\nlet x = 1.0;";
        assert_eq!(scan_rust(&claim).len(), 1);
        assert_eq!(scan_rust(comment).len(), 0);
        // Doc comments and module docs are commentary too.
        assert_eq!(
            scan_rust("/// Fitted against the CRC Handbook.\nfn f() {}").len(),
            0
        );
        assert_eq!(scan_rust("/* CRC Handbook, block comment */").len(), 0);
    }

    #[test]
    fn a_refused_upstream_in_a_value_bound_field_is_a_finding() {
        let audit = audit();
        let text = bound(
            "provenance",
            "Ethanol enthalpy of fusion 4.93 kJ/mol: NIST Chemistry WebBook, SRD 69",
        );
        let bounds = scan_rust(&text);
        assert_eq!(bounds.len(), 1);
        let named = audit.named_by(&flatten(&bounds[0].text));
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].id, "nist-webbook");
        assert!(named[0].verdict.refuses_claims());
    }

    #[test]
    fn a_cleared_upstream_in_the_same_position_is_not() {
        let audit = audit();
        let text = bound("provenance", "PubChem CID 702 identity crosswalk");
        let bounds = scan_rust(&text);
        let named = audit.named_by(&flatten(&bounds[0].text));
        assert_eq!(named.len(), 1);
        assert!(!named[0].verdict.refuses_claims());
    }

    #[test]
    fn source_id_is_not_a_provenance_field() {
        // `source_id: "legacy/HBr"` must not be read as `source: ...`, or every
        // registry-shaped struct in the tree becomes a false finding.
        let text = format!("Evidence {{ {}: \"NIST Chemistry WebBook\" }}", "source_id");
        assert_eq!(scan_rust(&text).len(), 0);
    }

    #[test]
    fn a_citation_wrapped_across_lines_still_reads_as_one_sentence() {
        let text = format!(
            "Row {{ {}: \"CRC \\\n        Handbook, 97th ed.\" }}",
            "source"
        );
        let bounds = scan_rust(&text);
        assert_eq!(bounds.len(), 1);
        assert!(names_source(&flatten(&bounds[0].text), "CRC Handbook"));
    }

    #[test]
    fn an_excuse_silences_exactly_one_subject_and_upstream() {
        let mut audit = audit();
        audit.excuses.push(Excuse {
            surface: Surface::Registry,
            subject: "us-federal/nasa-cea-thermo-inp-v1".to_string(),
            upstreams: vec!["nist-webbook".to_string()],
            reason: "Lineage of Apache-2.0 bytes we are separately licensed to redistribute, not a transcription."
                .to_string(),
        });
        assert_eq!(audit.problems(), Vec::<String>::new());
        assert_eq!(
            audit.excuse_for(
                Surface::Registry,
                "us-federal/nasa-cea-thermo-inp-v1",
                "nist-webbook"
            ),
            Some(0)
        );
        // Different subject, different upstream, different surface: no cover.
        assert_eq!(
            audit.excuse_for(Surface::Registry, "legacy/I2", "nist-webbook"),
            None
        );
        assert_eq!(
            audit.excuse_for(
                Surface::Registry,
                "us-federal/nasa-cea-thermo-inp-v1",
                "echa-cl"
            ),
            None
        );
        assert_eq!(
            audit.excuse_for(
                Surface::Rust,
                "us-federal/nasa-cea-thermo-inp-v1",
                "nist-webbook"
            ),
            None
        );
    }

    #[test]
    fn an_excuse_that_covers_nothing_is_reported() {
        let mut audit = audit();
        audit.excuses.push(Excuse {
            surface: Surface::Registry,
            subject: "a/source/that/does/not/exist".to_string(),
            upstreams: vec!["nist-webbook".to_string()],
            reason: "A reason long enough to satisfy the minimum length rule for reasons."
                .to_string(),
        });
        let report = Report::default();
        let stale = audit.stale_excuses(&report);
        assert_eq!(stale.len(), 1);
        assert!(stale[0].contains("matched nothing"), "{stale:?}");
    }

    #[test]
    fn an_excuse_for_a_cleared_upstream_excuses_nothing() {
        let mut audit = audit();
        audit.excuses.push(Excuse {
            surface: Surface::Registry,
            subject: "legacy/water".to_string(),
            upstreams: vec!["pubchem".to_string()],
            reason: "A reason long enough to satisfy the minimum length rule for reasons."
                .to_string(),
        });
        let problems = audit.problems();
        assert!(
            problems.iter().any(|p| p.contains("excuses nothing")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_refused_row_may_not_be_allowed_to_touch_anything() {
        // The rule as an invariant on the table, so that a future edit cannot
        // clear a source by widening `may_touch` and leaving the verdict.
        let mut audit = audit();
        audit.upstreams[0].may_touch = vec!["property".to_string()];
        let problems = audit.problems();
        assert!(
            problems.iter().any(|p| p.contains("but may_touch lists")),
            "{problems:?}"
        );
    }

    #[test]
    fn an_inherited_row_must_say_when_its_terms_were_read() {
        let mut audit = audit();
        audit.upstreams[0].retrieved = String::new();
        let problems = audit.problems();
        assert!(
            problems.iter().any(|p| p.contains("terms were read")),
            "{problems:?}"
        );
        // A row PROPOSED here, not inherited, may say it has no date — that is
        // the honest state of `crc-handbook` and `merck-index`.
        audit.upstreams[0].in_plan_table = false;
        assert_eq!(audit.problems(), Vec::<String>::new());
    }

    #[test]
    fn scanning_the_repository_finds_the_known_offenders() {
        // A reach test with real numbers in it, so that a scanner that
        // silently stops finding things fails instead of passing.
        let text = std::fs::read_to_string("../../provenance/upstreams.toml")
            .expect("provenance/upstreams.toml is present");
        let audit = Audit::parse(&text).expect("parses");
        let report = audit.scan(Path::new("../.."));
        let rust_scanned = report.scanned.get("rust").copied().unwrap_or(0);
        let registry_scanned = report.scanned.get("registry").copied().unwrap_or(0);
        // Denominators: if the scanner breaks, these collapse and the test
        // fails rather than reporting a clean tree.
        assert!(
            rust_scanned > 150,
            "the Rust surface should carry ~210 value-bound strings, found {rust_scanned}"
        );
        assert!(
            registry_scanned > 150,
            "the registry carries ~185 citations, found {registry_scanned}"
        );
        // `phase_route.rs` cites the NIST WebBook for enthalpies of fusion and
        // vaporisation. If that stops being found, the scanner is broken — or
        // the values were re-sourced, and this assertion is the place to say so.
        let webbook: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.upstream == "nist-webbook" && f.surface == Surface::Rust)
            .collect();
        assert!(
            !webbook.is_empty(),
            "expected NIST WebBook findings on the Rust surface; if they are gone, \
             lower this assertion and raise the gate"
        );
        // Every excuse must still be earning its place. A stale one is a
        // claim of diligence with nothing behind it, and this is the assertion
        // that stops one being left behind after a citation is rewritten.
        assert_eq!(
            audit.stale_excuses(&report),
            Vec::<String>::new(),
            "an excuse matched nothing"
        );
        assert!(
            report.excused.values().sum::<usize>() >= 4,
            "expected the NASA CEA lineage excuse and the three 2026-09-13 \
             withdrawals to be silencing real matches, found {:?}",
            report.excused
        );
    }
}

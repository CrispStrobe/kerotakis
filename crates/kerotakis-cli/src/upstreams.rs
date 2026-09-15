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
//! Rather than weaken the rule, `upstreams.toml` carries a `[[citation]]`
//! list: subject, upstream, a `role` and a written reason. (It was an
//! `[[excuse]]` list until 2026-09-15, which could say only one of the three
//! roles; see "Citing a book is not leaning on one" below.) Every row must
//! MATCH something; one that covers nothing is reported as a problem, so a
//! stale one cannot sit there looking like diligence after the string it
//! covered is gone. The list is exactly as strong as its review — it is here
//! because it is visible, counted and expiring, which prose was not.
//!
//! ## What it catches
//!
//! A value-bound provenance string, anywhere in `crates/**/*.rs` or in the
//! registry export's source citations, that names an upstream whose verdict is
//! `avoid` or `permission-required`, and that no reviewed `[[citation]]` row
//! clears — by judging it `mentioned`, or by judging it an attributed
//! `claims` within the ceiling the upstream declares.
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
//! - **Whether a judgement is honest.** The lint checks that a `[[citation]]`
//!   row still MATCHES a real string and reports it if it does not. It cannot
//!   check that the reason is true, or that the role is the right one. A role
//!   is a human judgement recorded in data, and
//!   its whole advantage over the prose it replaces is that it is named,
//!   counted, printed on every run, and expires loudly when the string it
//!   covered changes. That is a smaller claim than "reviewed", and it is the
//!   one being made.
//! - **The difference between a withdrawal and a claim.** A citation that
//!   says "the claim resting on the CRC Handbook has been withdrawn" is
//!   commentary, and a citation that says "from the CRC Handbook" is a claim,
//!   and both are the value of the same `citation` field. The position test
//!   cannot separate them. *Partly answered on 2026-09-15* by the `role`
//!   vocabulary below: the separation is still one reviewed row at a time,
//!   but the row can now say which of three things it is, and `matching` can
//!   key it to a string rather than a file. What has NOT changed is the thing
//!   that matters — the lint checks that a role still matches a real string
//!   and never that it is true.
//! - **Whether a page number is real.** A `locator` is checked for presence
//!   and never for truth. It establishes that the citation names a place a
//!   reader could go; it cannot establish that the value is on that page,
//!   that anyone opened the book, or that the number did not come from
//!   somewhere else entirely. This tree proves the gap rather than supposing
//!   it: three rows in `phase_route.rs` name a CRC table and admit in the
//!   same sentence that "no positively identified copy was opened for this
//!   row". A fabricated locator would read exactly like a diligent one.
//!
//! ## Citing a book is not leaning on one
//!
//! `avoid` is a property of a SOURCE. The rule the owner gave on 2026-09-14
//! is a property of a CITATION:
//!
//! > We should be able to cite any book. Only not harvest the books by
//! > systematic scraping. And we should be able to trace original sources for
//! > almost all values, and cite those.
//!
//! Until 2026-09-15 this file could not say that. `avoid` was source-level
//! and every match under it was counted, so one properly attributed citation
//! produced a finding for exactly the practice the rule calls ordinary.
//! `upstreams.toml`'s own `voet-biochemistry` note says so in terms: "THE
//! VOCABULARY IS MISSING A VERDICT AND THIS IS WHERE IT SHOWS."
//!
//! ### What a citation is DOING: `role`
//!
//! `[[excuse]]` became `[[citation]]` with a `role`, because an excuse could
//! say only one of the three things a citation can be doing with a refused
//! work, and the tree exhibits all three:
//!
//! - **`mentioned`** — names it and claims nothing from it. Four shapes occur
//!   here: saying the cited source is NOT that one (`phase_route.rs` cites
//!   NBS Circular 500 and names the WebBook only to say the Standard
//!   Reference Data Act notice is absent from Circular 500), saying that
//!   source is WRONG (the naphthalene row: the WebBook prints a digit
//!   transposition), saying a RETIRED value used to rest on it, and recording
//!   a WITHDRAWAL. Not a claim, so not a finding.
//! - **`via`** — the value is a cleared primary's, read through this source's
//!   RENDERING rather than off the publication. Always a finding. Naming the
//!   right paper is good practice and does not undo having read the number
//!   off Standard Reference Data. What the role buys is that the debt is a
//!   *verification*, not a re-sourcing, which is a much cheaper repair.
//! - **`claims`** — the value IS this source's. Ordinary practice when it
//!   carries a `locator` and the work is refused for bulk; a finding
//!   otherwise. Its ABSENCE is meaningful too: three rows declare `claims`
//!   with no locator because their own text admits no copy was opened.
//!
//! ### Two clauses, two counters, and the unit each one wants
//!
//! The rule has two halves and they do not share a unit. Collapsing them is
//! what broke the instrument, so they are counted apart:
//!
//! - **Attribution is per CITATION**, because attribution is a property of a
//!   string. `locator` is the test.
//! - **Dependence is per SOURCE**, because harvesting is a property of a
//!   relationship to a work. `cite_at_most` is the ceiling, and going over it
//!   is ONE finding for the work rather than one per citation — the offence
//!   is the leaning, not each lean. Forty properly cited values out of one
//!   book is not forty offences; it is one decision, and a count that read
//!   forty would make the decision unreviewable.
//!
//! **Neither is the honest unit, and the honest unit is the QUANTITY.** A
//! citation is not a number: `legacy/HBr` is one string over fifteen numeric
//! records, and `kerotakis/material-recipes-v1` is one string over 493
//! evidence links. Counting citations therefore UNDER-counts dependence, in
//! one direction, always. Both figures are lower bounds and neither should
//! ever be quoted as a number of values.
//!
//! **No default ceiling is invented.** `cite_at_most` is zero on every row
//! until somebody declares otherwise with a written reason, because the rule
//! names no number and a lint picking one would be a lint deciding a policy
//! question. Deciding to depend on a copyrighted compilation should cost a
//! reviewed line.
//!
//! ### Where the carve-out stops: `avoid`, never `permission-required`
//!
//! The two refused verdicts refuse for different reasons and only one of them
//! is answerable by attribution. `avoid` carries the 2026-09-14 rule and its
//! objection is compilation copyright — one value out of a book is a fact and
//! not the compilation, which is the *Feist* argument PLAN.md already makes.
//! `permission-required` says something else: refused until a written grant
//! exists. A perfect citation does not create a grant.
//!
//! The repository settles it better than the argument does.
//! `legacy/liquid_nitrogen` cites "NIST Chemistry WebBook SRD 69 nitrogen
//! (CAS 7727-37-9)" with a deep link carrying the record id and the mask — a
//! better locator than any CRC citation in this tree — and it is still a
//! transcription out of Standard Reference Data. If attribution cleared
//! `permission-required`, that row would clear. Setting a `locator` on such a
//! row is therefore a problem in the audit file, not a silent no-op.
//!
//! ### What stops this being an amnesty
//!
//! A vocabulary that can clear findings can clear the wrong ones. Four things
//! hold it, and the first is the one that matters:
//!
//! 1. **It clears nothing by itself.** The default for every match is
//!    `undeclared`, which is a finding. A match changes side only when
//!    somebody writes a row, and every row in the shipped file quotes the
//!    sentence in the citation that establishes its role, so the judgement
//!    can be checked against the text rather than against an assertion.
//! 2. **A row that matches nothing is reported**, exactly as a stale excuse
//!    was.
//! 3. **A ceiling is legal only on `avoid`**, and only with a reason — the
//!    `may_touch` invariant wearing a number.
//! 4. **The registry denominator is asserted in a test.** The 46 CRC
//!    citations that name an edition and no page are where the bulk
//!    dependence actually lives, and a change that made them fall would have
//!    to lower that assertion and say why.
//!
//! ## The field that would STILL make this exact, which does not exist
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
//! **Half of this was built on 2026-09-15 and half was not.** `role` exists,
//! and it lives in `provenance/upstreams.toml` as `[[citation]]` rather than
//! on the registry source record, because the export and its byte-exact
//! golden mirror still belong to another change in flight. What that costs is
//! `covers`: the role is declared per (citation, upstream) and NOT per
//! QUANTITY, so `legacy/HBr` can be judged only as a whole and cannot yet say
//! that its molar mass is clean while its heat capacity is not. The finding
//! therefore still sits on the citation, and the sentence about it moving to
//! the individual quantity remains the thing to build. When the registry
//! field lands, these rows are what it should be seeded from.
//!
//! ## What must be true before this becomes a gate
//!
//! Four things, in order:
//!
//! 1. **The count reaches zero** on both surfaces, by re-sourcing or by
//!    withdrawal, not by deleting citations — watch the denominators.
//!    *Made satisfiable on 2026-09-15, and no closer to satisfied.* Before
//!    the `role` vocabulary this item could not be met honestly at all: a
//!    correctly attributed citation of a book was a permanent finding, so
//!    zero was reachable only by deleting the citation, which is the one
//!    route this item forbids in its own sentence. That contradiction is
//!    gone. The count is 67 and the work to move it is unchanged.
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
//! - **10** further matches are cleared by a reviewed row (an `[[excuse]]`
//!   then, a `[[citation]]` with `role = "mentioned"` now) and
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
//! - **Open questions 0 -> 154** on the registry and 2 on Rust, the first
//!   non-zero this column has ever printed: `ciaaw` 152, `acs-education` 1 and
//!   `fao` 1 on the registry, the last two on Rust as well. The 152 is the one
//!   that matters - that many citations name the atomic-weight body, whose
//!   terms grant educational reuse and reserve commercial use, and nobody has
//!   asked the question that settles it. Nothing got worse; a dependence
//!   became visible.
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
//! ### Addendum, 2026-09-15 (second): the vocabulary, and what moved
//!
//! Measured with the scanner reimplemented against the same two surfaces, no
//! citation edited and nothing re-sourced. This is a change to the
//! instrument, like the table repair before it.
//!
//! | | before | after |
//! |---|---|---|
//! | Rust findings | 18 of 210 | **7 of 210** |
//! | registry findings | 60 of 185 | **60 of 185** |
//! | blind spot | 6 of 185 | 6 of 185 |
//! | open questions | 152 / 2 / 2 | unchanged |
//!
//! The 18 is measured at `33c34631`, after #609 landed the `nsrds-nbs-37`
//! row and the `phase_route.rs` JANAF excuse. The "20 on Rust" quoted in
//! the brief for this change is the figure from before that excuse existed,
//! and the two rows are the whole difference.
//!
//! **Eleven Rust findings changed side, all to `mentioned`, and every one of
//! them was the lint reporting a source the string names in order to REJECT
//! it.** Named, because a count that moves without names is not reviewable:
//! `curated.rs` glucose (CRC and JANAF, "it is not a transcription ... no
//! edition-level provenance is claimed"); `phase_route.rs` methanol fusion,
//! acetic acid fusion, ethanol vaporisation and acetic acid vaporisation
//! (WebBook, named only in the clause distinguishing a Government work from
//! Standard Reference Data); naphthalene fusion (WebBook, named to report its
//! digit transposition); nitrogen vaporisation (WebBook, named for the
//! RETIRED figure); methanol vaporisation and acetic acid vaporisation (CRC,
//! named to trace a retired value and to refuse a PubChem route that would
//! have laundered it); acetone vaporisation (WebBook, named to report that
//! its pointer is self-inconsistent).
//!
//! **Seven stayed, and they are the ones that should.** Three `claims` with
//! no locator — `phase_route.rs` magnesium, copper and acetone on the CRC
//! Handbook, each of which says in its own words that no positively
//! identified copy was opened. Four `via` — propan-2-ol, ethyl acetate,
//! `pack.rs` and `vle.rs`, all four on the WebBook, all four already saying
//! that the value was read through its rendering of a correctly named paper.
//!
//! **The registry did not move at all, and that is the result, not a
//! shortfall.** The 46 CRC citations there read "CRC Handbook, 97th ed."
//! beside a quantity and nothing else; 58 of the 60 carry no locator of any
//! kind. This is what bulk dependence looks like and none of it clears.
//! `legacy/amylase`, the row this change was pointed at, stays a finding for
//! the reason the rule itself gives: no page, and "typical" by its own word.
//! What has changed is that a citation carrying author, title, edition and
//! page would no longer fail identically to it, which was the defect.
//!
//! **Nothing cleared through the attribution carve-out.** No row declares a
//! `cite_at_most`, so `attributed` is empty and a test asserts it stays that
//! way. Declaring a ceiling is a judgement about how much of a book this
//! project accepts leaning on, and it is the owner's to give.
//!
//! **Every excuse became a `[[citation]]` and none became unnecessary.** All
//! five were `role = "mentioned"` wearing a different name; converting them
//! is a rename, not a repair. One got strictly better: the `phase_route.rs`
//! JANAF row was keyed to the FILE and said so against itself — "this excuse
//! is keyed to the file, not to the two strings ... Narrow it if that becomes
//! likely" — and `matching = "NSRDS-NBS 37"` now keys it to the two strings,
//! so a genuine JANAF-online citation added to that file is reported.
//!
//! ### Two citations found while measuring, recorded and not touched
//!
//! Both are judgements somebody should make deliberately, and neither is
//! made here.
//!
//! - **`legacy/Fe+2` would be defensible as `mentioned` and is left
//!   undeclared.** The CRC is named inside a parenthesis supporting a
//!   QUALITATIVE fact — that iron(II) sulfate and its solutions are pale
//!   green — alongside Greenwood and Earnshaw, and the citation states that
//!   "no edition of any handbook was opened for a per-wavelength epsilon and
//!   none is claimed", the sixteen band values being a declared curated
//!   teaching spectrum. So no shipped NUMBER rests on the Handbook. A
//!   reviewer could equally say the colour claim does rest on it. That is a
//!   judgement about a citation, not about an instrument.
//! - **`legacy/amylase` is the live instance the vocabulary was built for and
//!   it does not move.** Worth stating plainly, because "the rule says this
//!   should be allowed" is not the same as "this particular row qualifies".
//!   It does not: no page, and "typical".
//!
//! And one thing that got WORSE in a way worth writing down rather than
//! celebrating: clearing the `curated.rs` glucose row removes a finding from
//! this lint and gives it to nothing. That value is "recorded AS COMMONLY
//! TABULATED and ITS PROVENANCE LANE IS PENDING REVIEW" — it has no source at
//! all. It was never a refused-source offence and is now correctly not
//! counted as one; it is an unattributed value, which is the older and larger
//! problem this file's own limits say it cannot see. The row adds that
//! "Nothing in the engine consumes it", which is the only reason this is
//! small.
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
//! It exits zero. There are 67 findings across the two surfaces and other
//! agents are working them down; a gate that failed today would block every
//! unrelated change and land on their heads. `--fail` (or
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
    #[serde(default, rename = "citation")]
    citations: Vec<Citation>,
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
    /// How many attributed claims this project has decided it will rest on
    /// this work. Zero by default and on every row, including refused ones:
    /// no number is invented here, because the rule names none. Raising it is
    /// a reviewed edit that must say why, and it is legal only on `avoid` —
    /// see `Role::Claims` and `Verdict::refuses_for_bulk`.
    #[serde(default)]
    cite_at_most: usize,
    #[serde(default)]
    cite_at_most_reason: String,
    #[serde(default)]
    note: String,
}

/// What a named refused source is DOING in one value-bound string.
///
/// This is the vocabulary the file did not have. `avoid` is a property of a
/// SOURCE; the owner's 2026-09-14 rule is about a property of a CITATION, and
/// until there was a word for the second the lint could only report the first.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Role {
    /// The string NAMES the source and claims no value from it — to say the
    /// cited source is not that one, to say that source is wrong, to say a
    /// retired value used to rest on it, or to record a withdrawal. Not a
    /// claim, so not a finding. This is the old `[[excuse]]`, renamed to sit
    /// beside its siblings.
    Mentioned,
    /// The value is a CLEARED primary's, and was read through this refused
    /// source's rendering of it rather than off the publication. A finding,
    /// always: the citation names the right work, and the bytes still came
    /// through the refused one. What it buys is naming the debt correctly —
    /// this wants a verification, not a re-sourcing.
    Via,
    /// The value IS this source's. Ordinary practice under the 2026-09-14
    /// rule when it carries a `locator` and the work is refused for bulk
    /// dependence; a finding otherwise. See `Verdict::refuses_for_bulk` for
    /// why `permission-required` is excluded.
    Claims,
}

impl Role {
    fn label(self) -> &'static str {
        match self {
            Role::Mentioned => "mentioned",
            Role::Via => "via",
            Role::Claims => "claims",
        }
    }
}

/// A reviewed judgement about ONE citation's relationship to ONE refused
/// source. The unit is deliberately the citation and not the source: a
/// verdict is a property of a work's terms and must read the same on every
/// day, which is the argument `voet-biochemistry`'s own note makes.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Citation {
    surface: Surface,
    /// A registry source id, or a repository-relative path for `rust`.
    subject: String,
    /// Optional: narrow this row to value-bound strings CONTAINING this text.
    /// A Rust subject is a whole file, so without it one judgement covers
    /// every string in that file — a limit the `phase_route.rs` excuse stated
    /// against itself and asked to have closed.
    #[serde(default)]
    matching: String,
    /// Every refused upstream this one string names in this role. A
    /// withdrawal notice typically names all of them at once, because it is
    /// quoting the rule it is obeying.
    upstreams: Vec<String>,
    role: Role,
    /// Where in the work the value is: the page, table or entry. Only
    /// meaningful on `claims`, and only on an `avoid` upstream. Its ABSENCE
    /// is meaningful too, and three rows in this tree use it that way: they
    /// declare `claims` with no locator, because their own text admits no
    /// copy was opened.
    #[serde(default)]
    locator: String,
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

    /// Is this row refused because depending on it IN BULK is the offence,
    /// rather than because nobody granted permission at all?
    ///
    /// This is the line the whole carve-out turns on, and the two verdicts
    /// define it themselves. `avoid` carries the owner's 2026-09-14 rule
    /// verbatim: the objection is compilation copyright, one value out of a
    /// book is a fact and not the compilation, and the cure is attribution.
    /// `permission-required` says something different — "refused until a
    /// written grant exists" — and a perfect citation does not create a
    /// grant. So an attributed claim can be ordinary practice under the first
    /// and never under the second.
    ///
    /// The repository makes the case better than the argument does.
    /// `legacy/liquid_nitrogen` cites "NIST Chemistry WebBook SRD 69 nitrogen
    /// (CAS 7727-37-9)" with a deep link carrying the record id and the mask
    /// — a BETTER locator than any CRC citation in the tree — and it is still
    /// a transcription out of Standard Reference Data. If attribution cleared
    /// `permission-required`, that row would clear, and it must not.
    fn refuses_for_bulk(self) -> bool {
        matches!(self, Verdict::Avoid)
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

/// Why a match is being reported. The headline is the sum of all four, but
/// they are different offences wanting different repairs, and a single number
/// that mixed them is what this change exists to stop.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Kind {
    /// No `[[citation]]` row judges this string. The default, and the state
    /// every match in the tree was in before this vocabulary existed.
    Undeclared,
    /// Declared `claims`, and it does not clear: no locator, or the upstream
    /// is refused for permission rather than for bulk.
    Claimed,
    /// Declared `via` — the primary is cited, the bytes came through the
    /// refused rendering.
    Via,
    /// Attributed dependence past the ceiling the upstream row declares. One
    /// per (surface, source), not one per citation: the offence is the
    /// leaning, not each lean.
    Bulk,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Undeclared => "undeclared",
            Kind::Claimed => "claimed",
            Kind::Via => "via",
            Kind::Bulk => "bulk dependence",
        }
    }
}

/// One value-bound provenance string that names a refused upstream.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Finding {
    kind: Kind,
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
    /// Citation rows that matched a real string, by index into
    /// `Audit::citations`.
    used_citations: BTreeSet<usize>,
    /// Matches cleared as `mentioned`, so that a reader of the report can see
    /// what was set aside rather than only what was counted.
    pub mentioned: BTreeMap<String, usize>,
    /// Attributed claims that cleared, keyed "surface / upstream", each entry
    /// naming the subject and the locator it rests on. Not an offence, and
    /// printed in full anyway: this IS the dependence, and a dependence that
    /// is not itemised cannot be judged.
    pub attributed: BTreeMap<String, Vec<String>>,
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
            // A declared ceiling is legal ONLY where the refusal is about
            // bulk dependence. Otherwise a row could be cleared by deciding
            // to accept N claims on a source nobody has permission to use at
            // all, which is the `may_touch` failure mode wearing a number.
            if upstream.cite_at_most > 0 && !upstream.verdict.refuses_for_bulk() {
                problems.push(format!(
                    "upstream '{id}': cite_at_most is {} but the verdict is {} — a ceiling is \
                     only meaningful where the refusal is bulk dependence, and attribution \
                     cannot create a permission",
                    upstream.cite_at_most,
                    upstream.verdict.label()
                ));
            }
            if upstream.cite_at_most > 0 && upstream.cite_at_most_reason.trim().len() < 40 {
                problems.push(format!(
                    "upstream '{id}': cite_at_most is {} and must say, in a sentence, how many \
                     claims this project accepts on this work and why",
                    upstream.cite_at_most
                ));
            }
            if upstream.cite_at_most == 0 && !upstream.cite_at_most_reason.trim().is_empty() {
                problems.push(format!(
                    "upstream '{id}': cite_at_most_reason is set but cite_at_most is 0 — a \
                     reason for a ceiling nobody declared"
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
        for (index, citation) in self.citations.iter().enumerate() {
            if citation.upstreams.is_empty() {
                problems.push(format!(
                    "citation[{index}] for '{}': lists no upstream",
                    citation.subject
                ));
            }
            for named in &citation.upstreams {
                if !ids.contains(named) {
                    problems.push(format!("citation[{index}]: unknown upstream '{named}'"));
                } else if let Some(up) = self.upstreams.iter().find(|u| &u.id == named) {
                    if !up.verdict.refuses_claims() {
                        problems.push(format!(
                            "citation[{index}]: upstream '{}' is {}, which refuses nothing — the judgement judges nothing",
                            up.id,
                            up.verdict.label()
                        ));
                    }
                    // A locator is the instrument that makes an attributed
                    // claim ordinary practice, and it only does that where
                    // the refusal is about bulk. Set anywhere else it is
                    // inert, and inert-looking-like-diligence is the thing
                    // this file refuses everywhere else.
                    if !citation.locator.trim().is_empty() && !up.verdict.refuses_for_bulk() {
                        problems.push(format!(
                            "citation[{index}] for '{}': locator is set for '{}', which is {} — \
                             a page cannot cure a permission, so the locator would do nothing",
                            citation.subject,
                            up.id,
                            up.verdict.label()
                        ));
                    }
                }
            }
            if citation.role != Role::Claims && !citation.locator.trim().is_empty() {
                problems.push(format!(
                    "citation[{index}] for '{}': locator is only meaningful on role = \"claims\", \
                     not on \"{}\"",
                    citation.subject,
                    citation.role.label()
                ));
            }
            if citation.reason.trim().len() < 40 {
                problems.push(format!(
                    "citation[{index}] for '{}': reason must say WHY this role is the right one, \
                     in a sentence",
                    citation.subject
                ));
            }
        }
        problems
    }

    /// The reviewed judgement covering this (surface, subject, upstream), if
    /// one exists. `matching`, when set, narrows the row to strings that
    /// contain it — which is how a Rust judgement stops covering a whole file.
    fn judgement_for(
        &self,
        surface: Surface,
        subject: &str,
        upstream: &str,
        text: &str,
    ) -> Option<usize> {
        self.citations.iter().position(|c| {
            c.surface == surface
                && c.subject == subject
                && c.upstreams.iter().any(|u| u == upstream)
                && (c.matching.trim().is_empty() || text.contains(c.matching.trim()))
        })
    }

    /// Every upstream that `text` names, refused or not.
    fn named_by(&self, text: &str) -> Vec<&Upstream> {
        self.upstreams
            .iter()
            .filter(|u| u.names.iter().any(|n| names_source(text, n)))
            .collect()
    }

    /// Apply the reviewed judgement, if any, to one match. `None` means the
    /// match is cleared and is not a finding.
    ///
    /// Everything the 2026-09-14 rule added to this lint is in these fifteen
    /// lines, and it is worth saying what they do NOT do: nothing here reads
    /// the citation. A role is a human judgement recorded in data, exactly as
    /// an excuse was, and the lint can check that it still matches a real
    /// string and never that it is true.
    fn judge(
        &self,
        report: &mut Report,
        surface: Surface,
        subject: &str,
        upstream: &Upstream,
        text: &str,
    ) -> Option<Kind> {
        let Some(index) = self.judgement_for(surface, subject, &upstream.id, text) else {
            return Some(Kind::Undeclared);
        };
        report.used_citations.insert(index);
        let citation = &self.citations[index];
        match citation.role {
            Role::Mentioned => {
                *report
                    .mentioned
                    .entry(format!("{subject} / {}", upstream.id))
                    .or_default() += 1;
                None
            }
            Role::Via => Some(Kind::Via),
            Role::Claims => {
                // Two ways an attributed claim fails to be ordinary practice:
                // the refusal is not about bulk, so no page can cure it; or
                // there is no page, so it is not attribution.
                if !upstream.verdict.refuses_for_bulk() || citation.locator.trim().is_empty() {
                    return Some(Kind::Claimed);
                }
                report
                    .attributed
                    .entry(format!("{} / {}", surface.label(), upstream.id))
                    .or_default()
                    .push(format!("{subject} — {}", citation.locator.trim()));
                None
            }
        }
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
                    let Some(kind) =
                        self.judge(&mut report, Surface::Rust, &relative, upstream, &flat)
                    else {
                        continue;
                    };
                    report.findings.push(Finding {
                        kind,
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
                            let Some(kind) =
                                self.judge(&mut report, Surface::Registry, id, upstream, &flat)
                            else {
                                continue;
                            };
                            report.findings.push(Finding {
                                kind,
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

        // Bulk dependence, computed last because it is a property of the
        // whole surface rather than of any one string. ONE finding per
        // (surface, source) that is over its declared ceiling, not one per
        // citation: the offence is the leaning, and counting each lean would
        // make a single decision read as twenty.
        for surface in [Surface::Rust, Surface::Registry] {
            for upstream in &self.upstreams {
                let key = format!("{} / {}", surface.label(), upstream.id);
                // Take the count, not the Vec: the push below needs `report`
                // mutably and a live borrow of `attributed` would forbid it.
                let Some(count) = report.attributed.get(&key).map(Vec::len) else {
                    continue;
                };
                if count <= upstream.cite_at_most {
                    continue;
                }
                report.findings.push(Finding {
                    kind: Kind::Bulk,
                    surface,
                    subject: format!("({count} attributed claims on this work)"),
                    line: 0,
                    upstream: upstream.id.clone(),
                    verdict: upstream.verdict.label(),
                    excerpt: format!(
                        "{count} attributed claims rest on this work; the row declares \
                         cite_at_most = {}",
                        upstream.cite_at_most
                    ),
                });
            }
        }

        report.findings.sort();
        report
    }

    /// A judgement that covers nothing is a claim of diligence with nothing
    /// behind it, so it expires loudly rather than quietly.
    fn stale_judgements(&self, report: &Report) -> Vec<String> {
        self.citations
            .iter()
            .enumerate()
            .filter(|(index, _)| !report.used_citations.contains(index))
            .map(|(index, citation)| {
                format!(
                    "citation[{index}]: '{}' / {:?} as {} on the {} surface matched nothing — \
                     delete it or fix its subject",
                    citation.subject,
                    citation.upstreams,
                    citation.role.label(),
                    citation.surface.label()
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
    let stale = audit.stale_judgements(&report);
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
            let mut by_subject: BTreeMap<(&str, &str), usize> = BTreeMap::new();
            for finding in findings {
                *by_subject
                    .entry((finding.kind.label(), finding.subject.as_str()))
                    .or_default() += 1;
            }
            for ((kind, subject), count) in &by_subject {
                println!("      {count:4}  [{kind}] {subject}");
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

    if !report.mentioned.is_empty() {
        let total: usize = report.mentioned.values().sum();
        println!(
            "\n  cleared as role = \"mentioned\" by a reviewed [[citation]] row ({total}) — \
             named as commentary, not claimed:"
        );
        for (what, count) in &report.mentioned {
            println!("    {count:4}  {what}");
        }
    }

    if !report.attributed.is_empty() {
        let total: usize = report.attributed.values().map(Vec::len).sum();
        println!(
            "\n  attributed dependence ({total}) — role = \"claims\" with a locator, on a work \
             refused for BULK. Not an offence under the 2026-09-14 rule; itemised anyway, \
             because this is what the project leans on:"
        );
        for (key, rows) in &report.attributed {
            let ceiling = key
                .split_once(" / ")
                .and_then(|(_, id)| audit.upstreams.iter().find(|u| u.id == id))
                .map_or(0, |u| u.cite_at_most);
            println!(
                "    {key}: {} of a declared cite_at_most = {ceiling}",
                rows.len()
            );
            for row in rows {
                println!("      - {row}");
            }
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
    if total > 0 {
        let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
        for finding in &report.findings {
            *by_kind.entry(finding.kind.label()).or_default() += 1;
        }
        let parts: Vec<String> = by_kind
            .iter()
            .map(|(kind, count)| format!("{count} {kind}"))
            .collect();
        println!(
            "provenance upstreams: findings by kind — {}",
            parts.join(", ")
        );
    }
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

[[upstream]]
id = "a-textbook"
name = "A copyrighted textbook"
licence = "LicenseRef-No-Licence-Granted"
terms = "https://example.invalid/book"
retrieved = "2026-09-15"
verdict = "avoid"
may_touch = []
cite_at_most = 1
cite_at_most_reason = "One value with a page is citing a book; a second would be the start of leaning on it."
names = ["A Textbook"]
in_plan_table = true
note = "Refused for bulk dependence, citable once with a page."
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
        // The two NIST SRD rows are refused for PERMISSION, not for bulk, so
        // no amount of attribution may clear them and no ceiling is legal.
        for id in ["nist-webbook", "nist-janaf"] {
            let row = audit
                .upstreams
                .iter()
                .find(|u| u.id == id)
                .unwrap_or_else(|| panic!("{id} is audited"));
            assert!(
                !row.verdict.refuses_for_bulk(),
                "{id} is refused until a written grant exists; a page cannot cure that"
            );
            assert_eq!(row.cite_at_most, 0, "{id} may declare no ceiling");
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

    fn judgement(
        surface: Surface,
        subject: &str,
        upstream: &str,
        role: Role,
        locator: &str,
    ) -> Citation {
        Citation {
            surface,
            subject: subject.to_string(),
            matching: String::new(),
            upstreams: vec![upstream.to_string()],
            role,
            locator: locator.to_string(),
            reason: "A reason long enough to satisfy the minimum length rule for reasons."
                .to_string(),
        }
    }

    #[test]
    fn a_judgement_covers_exactly_one_subject_and_upstream() {
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Registry,
            "us-federal/nasa-cea-thermo-inp-v1",
            "nist-webbook",
            Role::Mentioned,
            "",
        ));
        assert_eq!(audit.problems(), Vec::<String>::new());
        assert_eq!(
            audit.judgement_for(
                Surface::Registry,
                "us-federal/nasa-cea-thermo-inp-v1",
                "nist-webbook",
                "any text at all"
            ),
            Some(0)
        );
        // Different subject, different upstream, different surface: no cover.
        assert_eq!(
            audit.judgement_for(Surface::Registry, "legacy/I2", "nist-webbook", ""),
            None
        );
        assert_eq!(
            audit.judgement_for(
                Surface::Registry,
                "us-federal/nasa-cea-thermo-inp-v1",
                "echa-cl",
                ""
            ),
            None
        );
        assert_eq!(
            audit.judgement_for(
                Surface::Rust,
                "us-federal/nasa-cea-thermo-inp-v1",
                "nist-webbook",
                ""
            ),
            None
        );
    }

    #[test]
    fn matching_narrows_a_judgement_from_a_file_to_a_string() {
        // The limit `phase_route.rs`'s own excuse stated against itself: a
        // Rust subject is a whole FILE, so one judgement covered every string
        // in it. `matching` is how that is closed.
        let mut audit = audit();
        let mut row = judgement(
            Surface::Rust,
            "crates/kerotakis-core/src/phase_route.rs",
            "nist-webbook",
            Role::Mentioned,
            "",
        );
        row.matching = "NSRDS-NBS 37".to_string();
        audit.citations.push(row);
        assert_eq!(
            audit.judgement_for(
                Surface::Rust,
                "crates/kerotakis-core/src/phase_route.rs",
                "nist-webbook",
                "JANAF Thermochemical Tables, second edition, NSRDS-NBS 37"
            ),
            Some(0)
        );
        // A genuinely different citation in the SAME file is not covered.
        assert_eq!(
            audit.judgement_for(
                Surface::Rust,
                "crates/kerotakis-core/src/phase_route.rs",
                "nist-webbook",
                "Benzene enthalpy of fusion from the NIST Chemistry WebBook"
            ),
            None
        );
    }

    #[test]
    fn a_judgement_that_covers_nothing_is_reported() {
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Registry,
            "a/source/that/does/not/exist",
            "nist-webbook",
            Role::Mentioned,
            "",
        ));
        let report = Report::default();
        let stale = audit.stale_judgements(&report);
        assert_eq!(stale.len(), 1);
        assert!(stale[0].contains("matched nothing"), "{stale:?}");
    }

    #[test]
    fn a_judgement_for_a_cleared_upstream_judges_nothing() {
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Registry,
            "legacy/water",
            "pubchem",
            Role::Mentioned,
            "",
        ));
        let problems = audit.problems();
        assert!(
            problems.iter().any(|p| p.contains("judges nothing")),
            "{problems:?}"
        );
    }

    // ------------------------------------------------ the 2026-09-14 rule --

    #[test]
    fn an_attributed_claim_on_a_book_is_ordinary_practice() {
        // The whole point. One value, one page, a work refused for BULK
        // dependence: the owner's rule calls this ordinary and the lint now
        // agrees. Before this vocabulary existed it was a finding.
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Registry,
            "legacy/thing",
            "a-textbook",
            Role::Claims,
            "4th ed., Table 3-2, p. 118",
        ));
        assert_eq!(audit.problems(), Vec::<String>::new());
        let mut report = Report::default();
        let book = audit
            .upstreams
            .iter()
            .find(|u| u.id == "a-textbook")
            .expect("fixture row");
        assert_eq!(
            audit.judge(&mut report, Surface::Registry, "legacy/thing", book, ""),
            None,
            "an attributed claim within the ceiling is not a finding"
        );
        assert_eq!(report.attributed["registry / a-textbook"].len(), 1);
    }

    #[test]
    fn the_same_claim_without_a_page_is_still_a_finding() {
        // "Citing a book" means author, title, edition and page. A bare
        // "CRC Handbook, 97th ed." names a work and no place in it, which is
        // the state 58 of the 60 registry findings are in.
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Registry,
            "legacy/thing",
            "a-textbook",
            Role::Claims,
            "",
        ));
        let mut report = Report::default();
        let book = audit
            .upstreams
            .iter()
            .find(|u| u.id == "a-textbook")
            .expect("fixture row");
        assert_eq!(
            audit.judge(&mut report, Surface::Registry, "legacy/thing", book, ""),
            Some(Kind::Claimed)
        );
    }

    #[test]
    fn a_page_cannot_cure_a_permission() {
        // `legacy/liquid_nitrogen` cites the WebBook with a CAS number and a
        // deep link carrying the record id — a better locator than any CRC
        // citation in the tree — and it is still a transcription out of
        // Standard Reference Data. Attribution is the answer to compilation
        // copyright, never to "nobody granted permission".
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Registry,
            "legacy/liquid_nitrogen",
            "nist-webbook",
            Role::Claims,
            "SRD 69 nitrogen (CAS 7727-37-9)",
        ));
        let problems = audit.problems();
        assert!(
            problems
                .iter()
                .any(|p| p.contains("cannot cure a permission")),
            "a locator on a permission-required row must be refused outright: {problems:?}"
        );
        // And even if the file were somehow accepted, the match is a finding.
        let mut report = Report::default();
        let webbook = audit
            .upstreams
            .iter()
            .find(|u| u.id == "nist-webbook")
            .expect("fixture row");
        assert_eq!(
            audit.judge(
                &mut report,
                Surface::Registry,
                "legacy/liquid_nitrogen",
                webbook,
                ""
            ),
            Some(Kind::Claimed)
        );
    }

    #[test]
    fn via_names_the_right_work_and_is_still_a_finding() {
        // Four strings in the tree say the value was read through the
        // WebBook's RENDERING of a clean primary. The citation names the
        // right publication; the bytes still came through the refused source.
        let mut audit = audit();
        audit.citations.push(judgement(
            Surface::Rust,
            "crates/kerotakis-thermo/src/vle.rs",
            "nist-webbook",
            Role::Via,
            "",
        ));
        let mut report = Report::default();
        let webbook = audit
            .upstreams
            .iter()
            .find(|u| u.id == "nist-webbook")
            .expect("fixture row");
        assert_eq!(
            audit.judge(
                &mut report,
                Surface::Rust,
                "crates/kerotakis-thermo/src/vle.rs",
                webbook,
                ""
            ),
            Some(Kind::Via)
        );
    }

    #[test]
    fn attributed_claims_past_the_ceiling_are_one_finding_not_many() {
        // Forty properly cited values out of one book is not forty offences;
        // it is one decision to depend on a book, and the count should say so.
        let mut audit = audit();
        for subject in ["legacy/a", "legacy/b", "legacy/c"] {
            audit.citations.push(judgement(
                Surface::Registry,
                subject,
                "a-textbook",
                Role::Claims,
                "4th ed., p. 118",
            ));
        }
        let mut report = Report::default();
        let book = audit
            .upstreams
            .iter()
            .find(|u| u.id == "a-textbook")
            .expect("fixture row");
        for subject in ["legacy/a", "legacy/b", "legacy/c"] {
            assert_eq!(
                audit.judge(&mut report, Surface::Registry, subject, book, ""),
                None
            );
        }
        assert_eq!(report.attributed["registry / a-textbook"].len(), 3);
        // The fixture declares cite_at_most = 1, so three is over it. The
        // bulk pass in `scan` turns that into exactly one finding; here the
        // shape of the decision is what is asserted.
        assert!(3 > book.cite_at_most);
    }

    #[test]
    fn a_ceiling_is_only_legal_where_the_refusal_is_about_bulk() {
        // The `may_touch` failure mode wearing a number: a future edit must
        // not be able to clear a permission-required source by deciding to
        // accept N claims on it.
        let mut audit = audit();
        let index = audit
            .upstreams
            .iter()
            .position(|u| u.id == "nist-webbook")
            .expect("fixture row");
        audit.upstreams[index].cite_at_most = 5;
        audit.upstreams[index].cite_at_most_reason =
            "A reason long enough to satisfy the minimum length rule for reasons.".to_string();
        let problems = audit.problems();
        assert!(
            problems.iter().any(|p| p.contains("only meaningful where")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_ceiling_must_say_why_it_is_that_number() {
        let mut audit = audit();
        let index = audit
            .upstreams
            .iter()
            .position(|u| u.id == "a-textbook")
            .expect("fixture row");
        audit.upstreams[index].cite_at_most_reason = String::new();
        let problems = audit.problems();
        assert!(
            problems.iter().any(|p| p.contains("how many")),
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
        // Every judgement must still be earning its place. A stale one is a
        // claim of diligence with nothing behind it, and this is the assertion
        // that stops one being left behind after a citation is rewritten.
        assert_eq!(
            audit.stale_judgements(&report),
            Vec::<String>::new(),
            "a [[citation]] judgement matched nothing"
        );
        assert!(
            report.mentioned.values().sum::<usize>() >= 4,
            "expected the NASA CEA lineage row and the three 2026-09-13 \
             withdrawals to be clearing real matches, found {:?}",
            report.mentioned
        );
        // The registry surface is where the bulk dependence lives and this
        // change deliberately does not touch it: 46 CRC citations naming an
        // edition and no page stay exactly as they were. If this number
        // falls, either somebody re-sourced them — in which case lower it and
        // say so — or a role was declared that should not have been.
        let registry: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.surface == Surface::Registry)
            .collect();
        assert!(
            registry.len() >= 60,
            "the registry surface carried 60 findings when the role vocabulary \
             landed and this change was not supposed to move it, found {}",
            registry.len()
        );
        // Nothing in the shipped tree has been declared an attributed claim
        // that CLEARS, because no refused row declares a ceiling. Populating
        // that is a judgement, not an instrument change; if this fires, read
        // the `cite_at_most` rows before lowering it.
        assert!(
            report.attributed.is_empty(),
            "no work has a declared ceiling yet, so nothing should clear as \
             attributed dependence, found {:?}",
            report.attributed
        );
    }
}

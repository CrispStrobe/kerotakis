# Contributing to Kerotakis

Thank you for considering a contribution. Two things in this document are
load-bearing and non-negotiable; everything else is ordinary courtesy.

## 1. Licensing of contributions (please read before your first PR)

Kerotakis is licensed **AGPL-3.0-or-later** with an **additional permission**
(GNU AGPL v3, section 7) that allows the project's copyright holders to publish
official binaries through app stores (Apple App Store, Google Play) under those
stores' terms. The permission text is in [NOTICE](NOTICE).

Under AGPL §7, only copyright holders can grant additional permissions. So that
the app-store builds can keep including community code, we require:

> **By submitting a contribution to this repository, you license it under
> AGPL-3.0-or-later and additionally grant, for your contribution, the same
> app-store additional permission set out in NOTICE, exercisable by the
> project's copyright holders.**

This is inbound = outbound, including the exception (the model used by the
Nextcloud mobile apps). It takes none of your AGPL rights away — your code
remains AGPL for everyone, including us; the grant only keeps the official
store binaries legal. If you cannot or do not wish to grant this, please open
an issue instead of a PR so we can discuss.

Sign your commits with a `Signed-off-by:` line (`git commit -s`), which we read
as your assent to the above and to the
[Developer Certificate of Origin](https://developercertificate.org/).

## 2. Data provenance

Code and data have different rules here.

**Curated data shipped in official binaries is CC BY 4.0 or CC0 1.0**, separately
from the AGPL-3.0 code. Only CC BY or CC0 data enters the app store payload.
CC BY-SA material (including anything derived from BY-SA sources such as the
Open Reaction Database) is published separately on Hugging Face or a companion
GitHub repository and is not bundled in store builds. This separation keeps the
educational chemistry commons open while ensuring store-distribution
compatibility.

**Every dataset, parameter table, or constant must carry a provenance record**:
source, licence, retrieval date — and, where the source itself cites
literature, that citation too. This is not paperwork; it is a product feature.
`kero explain` shows users which engine, which dataset, which model, and what
that dataset cites, and codex entries are expected to do the same. Data
arriving without provenance cannot be displayed honestly, so it cannot be
merged. (Provenance records are moving to machine-readable TOML checked by
a `kero provenance lint` — see PLAN.md; until that lands, the reviewer is
the lint.)

**Model-assisted drafts are welcome, disclosed.** Codex content drafted
with an LLM is acceptable — the lint replays every claim through the real
solvers regardless of who typed it — but say so in the entry's provenance,
and expect the same editorial review as any draft. A model's confidence is
not a citation: misconception distractors still cite the literature or are
marked `Editorial judgement (Kerotakis)`.

**Honour the original terms, not the convenient label.** Several upstream
datasets relabel ShareAlike-derived data as CC BY or MIT without addressing
the inconsistency (see the data table in [PLAN.md](PLAN.md)). Where a source's
licence claim conflicts with its own stated provenance, we follow the original
source and record why.

PRs that add data without provenance, or from sources on the "avoid" list in
[PLAN.md](PLAN.md) (NIST SRD/WebBook, CAS Common Chemistry, CAMEO database
exports, ECHA dumps, Burcat, UNIFAC Consortium tables), will be declined
regardless of code quality — that list is a legal constraint, not a style
preference.

## 3. Dependency and data PR checklist

Every PR that adds a new Cargo dependency, vendored source, data import, or
external data file must include:

- [ ] **Source record** in `provenance/sources.toml` with id, licence, origin,
      retrieval date, and SHA-256 checksums for vendored files.
- [ ] **Lane assignment**: `runtime` (ships in app), `build_oracle` (dev only),
      or `external_oracle` (never fetched in CI).
- [ ] **Licence on the allowlist**: MIT, Apache-2.0, BSD-2/3-Clause, CC0, ISC,
      Zlib, or USGS User Rights Notice for runtime sources. LGPL and copyleft
      sources are build-only.
- [ ] **`cargo deny check` passes** after the addition.
- [ ] **`tools/provenance-lint.sh` passes** with the new checksums.
- [ ] **Attribution text** in NOTICE if the source ships in binaries.
- [ ] **No oracle output** in `crates/`, `web/`, or `data/` paths.

For data imports specifically:
- [ ] **Per-field compatibility review** — incompatible upstream fields are
      rejected with an explicit reason (see DATA-007 pattern).
- [ ] **Per-record provenance** — every `NumericRecord` carries its own
      `source_id` and `method`.

## 4. Ordinary courtesy

- Open an issue before large changes; PLAN.md is the source of truth for
  architecture and build order.
- New solver code needs the conservation-invariant property tests and at least
  one golden test against a textbook value. If it produces an answer a user
  can see, it must populate `Provenance` — an unattributable number is a bug.
- New parsers or grammars get a fuzz target (`cargo-fuzz`), and new solver
  paths should hold the metamorphic invariants (order-independence,
  dilution monotonicity, scale invariance) described in PLAN.md's testing
  section, not only conservation.
- New lessons live in `lessons/` and are replayed by CI; they must compute,
  not narrate.
- `kerotakis-core` must keep compiling to `wasm32-unknown-unknown` and the five
  native targets; CI enforces this.
- Honesty is a feature: solver failures are surfaced, predictions are labelled
  as predictions, and the stage of the L4 cascade that produced an answer is
  shown to the user. Contributions that hardcode "expected" results defeat the
  project's premise.

## 5. Picking up breadth tasks

The agent-sized breadth roadmap is [BREADTH.md](BREADTH.md). Before claiming a
`BRD-*` task, verify that every listed prerequisite is merged, read the owning
CAP/EXP/apparatus/GUI document, and keep the PR within that task's scope. A
decision-gate task produces evidence and a go/no-go record; it must not quietly
add the candidate dependency. Completion requires the task's acceptance tests,
an updated status, and—once BRD-001 exists—an updated curiosity-corpus baseline.

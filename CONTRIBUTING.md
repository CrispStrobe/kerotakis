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

Code and data have different rules here. Every dataset, parameter table, or
constant that enters `kerotakis-data` (or any crate) must carry a provenance
record: source, licence, retrieval date. PRs that add data without provenance,
or from sources on the "avoid" list in [PLAN.md](PLAN.md) (NIST SRD/WebBook,
CAS Common Chemistry, CAMEO database exports, ECHA dumps, Burcat, UNIFAC
Consortium tables), will be declined regardless of code quality — this is a
legal constraint, not a style preference.

## 3. Ordinary courtesy

- Open an issue before large changes; PLAN.md is the source of truth for
  architecture and build order.
- New solver code needs the conservation-invariant property tests and at least
  one golden test against a textbook value.
- `kerotakis-core` must keep compiling to `wasm32-unknown-unknown` and the five
  native targets; CI enforces this.
- Honesty is a feature: solver failures are surfaced, predictions are labelled
  as predictions, and the stage of the L4 cascade that produced an answer is
  shown to the user. Contributions that hardcode "expected" results defeat the
  project's premise.

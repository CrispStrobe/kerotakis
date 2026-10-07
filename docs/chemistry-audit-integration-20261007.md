# Audit integration manifest, 2026-10-07

Documentation delivery [PR #755](https://github.com/CrispStrobe/kerotakis/pull/755)
merged as `e9b2c842e8c8c41275c88baa5f3063bf6b499da9`. That merge contains
documentation only. Accepted engine source remains
[`8cdab1f6`](https://github.com/CrispStrobe/kerotakis/commit/8cdab1f6ecd0b1132c4008014a5a0c5476429d17);
[the checkpoint](chemistry-audit-status-20261005.md) describes its evidence.

## Ordered implementation slices

These are candidate dependency groups, not claims that every group already
compiles independently. Check public API and test dependencies before opening
each PR. Compare against the exact accepted source; do not cherry-pick late
repairs without their foundations.

| Order | Scope and original source | Prerequisites and acceptance |
| --- | --- | --- |
| 1 | Amount foundation: `core/src/amount.rs`; original introduction `fe574d02`, frozen contracts `9b32b696` | Main-based primitive and public-module export only. Preserve original test bytes and execute adapted imports. [Port receipt](../audits/integration-20261007/amount-port.json); hosted validation pending. |
| 1b | Compensated stock: `core/src/stock.rs` and minimal Bench adapters | Depends on PR #756. Carries 25 frozen stock contracts; [port mapping](../audits/integration-20261007/stock-port.json) and [ownership decision](chemistry-amount-ownership-decision-20261007.md). Hosted validation pending. |
| 2 | Thermodynamic kernels: `kerotakis-thermo/src/{batch,pack,unifac,vle}.rs` and continuity/still/trace controls | Inspect changed diagnostics/error APIs against existing core callers. Carry all prerequisite positive and refusal controls; no core still rewrite until these APIs pass. |
| 3 | Equilibrium/thermal kernels: `kerotakis-cea/src/{gibbs,thermal,carbonate,closed}.rs` | Inspect callers of thermal and closed-gas APIs. Carry heat ceiling, vented products, finite boundaries and closed-gas controls together. |
| 4 | Core ownership and transactions: `core/src/{delta,required_conservation,stock,bench,solve,vessel}.rs` | Depends on Amount and the kernel APIs actually used. Transfer, spill, extraction, receiver/donor acceptance, safety and titration share ownership seams; split only at a compiling conservative boundary. |
| 5 | Native reconstruction: `kerotakis-phreeqc/src/{aqueous,inventory,enthalpy}.rs` and crystal controls | Coordinate core conservation APIs and typed phase ownership. Port accepted behavior before applying AUD-01/02; preserve all raw and live controls. |
| 6 | Observable/data/presentation seams: coverage, sparse optics, CLI provenance, registry export and public DTOs | Carry the same bounded coverage claims across Rust, CLI and web consumers. Keep reviewed scalar provenance; no unreviewed corpus import. |
| 7 | Harness and exact-source acceptance | All implementation slices and their prerequisite tests; hosted full audit, third/fourth replays and artifact integrity against the integrated executable. |

The inventory contains 55 changed source paths and 110 changed test paths across
the relevant crates (169 paths including workflow changes). The implementation
checkpoint was 168 commits ahead of its original main base. Early commits mix
large evidence inventories with code, so small final-source ports with explicit
provenance are preferable where cherry-picks cannot stand alone.

## Completion rules and next work

Every slice needs a PR link, its original-source mapping, exact destination
commit, required PR gates and any test import/format adaptations. Keep original
forecasts and receipts immutable. Existing selected audit acceptance does not
replace full repository gates. Do not merge broken intermediate states or
silently remove the four qualified legacy exclusions.

AUD-00 remains open until the integrated executable has new acceptance receipts.
AUD-01 must establish a nonduplicated Ca/Sr/C allocation budget before clamping.
AUD-02 must bound raw aqueous reconciliation using same-solve witnesses.
NUM-01 needs a public API/save decision and one authoritative migrated owner;
exporting Amount is only its prerequisite. See [the executable lane
descriptions](chemistry-audit-next-lanes.md) for fixtures and completion criteria.

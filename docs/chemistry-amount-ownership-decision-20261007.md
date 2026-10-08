# Authoritative amount ownership decision, 2026-10-07

Status: implementation decision for the narrow stock port; proposed vessel API
migration, not a completed vessel migration. Prerequisite:
[Amount foundation PR #756](https://github.com/CrispStrobe/kerotakis/pull/756).

## Choice and compatibility

Keep `Moles(f64)` as a scalar unit/model/display DTO. Compensated ownership
belongs inside inventory owners, with private `Amount` fields and checked
mutation methods. A scalar projection is explicitly lossy and must never be
written back as an ownership update. Do not add a residual map keyed by
species/phase alongside public mutable portion vectors.

First owner: `StockLedger`. Its bottle map is already private; replacement,
draw and unlimit exhaust its material write paths. Port accepted
`OwnedStockAmount { amount: Amount, unit: StockUnit }` directly. Default
conservative draws keep the audit's precision-refusal behavior. Compensated mode
is opt-in, with real overdraw refusal and a before/after debit certificate.
`remaining` and `entries` are projections; `remaining_exact` exposes the pair.
This does not make a downstream scalar vessel deposit compensated.

Changing public `Moles` would break tuple construction and every `.0`
consumer, including quantities that are not persistent owners. Avoid that
workspace-wide unit change. A future vessel migration should instead replace
public mutable `Vessel.contents` authority with a private inventory container,
checked mutation APIs and immutable projected `Portion` snapshots. This is a
deliberate Rust API break for struct literals, direct vector writes and mutable
entry access. It needs a separately reviewed migration; no hidden compatibility
sidecar may pretend those writes still preserve residuals.

Duplicate species/phase entries need stable entry identity or a certified
coalescing operation. Lots are provenance subdivisions of physical inventory,
not additional matter. Their split/debit/merge policy must be migrated together
with vessel authority; scalar lots must not erase or duplicate compensated
balances. Surface, electrode, headspace, material-object and crystal owners
remain distinct inventories until their elemental ownership is certified.

## Persistence and model boundaries

Stock uses the accepted tagged `kerotakis-stock/2` wrapper for compensated
balances; legacy map saves load with zero residual in conservative mode.
Legacy readers reject the wrapper, including empty opted-in shelves. Malformed,
noncanonical, negative and nonfinite states refuse. Bench must omit only empty
default-mode shelves, preserving empty compensated mode on save/reload.

Future vessel saves need an explicit schema/capability boundary before any
residual-bearing state is serialized. Declare the tag and old-reader rejection
contract in frozen tests before selecting a format. Merely defaulting a new
field is insufficient: an old reader can silently discard it. Preserve all
owned quantities and mode on roundtrip; scalar export is a labeled projection,
not a reusable authoritative snapshot.

Solvers receive scalar model inputs plus a conservation certificate for the
authoritative owned amounts. Their proposals must reconstruct conserved
elemental/charge ownership within declared independent error budgets.
Residuals cannot remain attached to pre-reaction species after reconstruction.
Until NUM-03 defines this, compensated vessel states must refuse solver paths
that cannot preserve them atomically. Existing default scalar behavior remains
a separately tested policy.

## Writer/reader migration checklist

Use the accepted source revision above when reviewing these seams, then refresh
the inventory against each integration PR. This is a reviewed starting map;
an exhaustive semantic writer/reader inventory remains an explicit NUM-01
deliverable before enabling compensated vessels.

| Seam | Writes and acceptance needed | Readers/projections to audit |
| --- | --- | --- |
| `core/src/stock.rs` | stock replacement, draw, unlimit, deserialization | remaining, entries, remaining_exact, save mode |
| `core/src/vessel.rs` | deposit, withdraw, withdraw_phase, lots, public contents replacement | aggregate amount, volume, mass, heat capacity, phase identity |
| `core/src/bench.rs` | add/recipe, transfer/decant/filter/magnet, extraction, titration, still, spill, headspace, snapshots | requested/debited/received amounts, energy and narration |
| `core/src/delta.rs` | direct debit/deposit, full replacement, adsorbed/electrode changes | candidate acceptance and rollback |
| `core/src/solve.rs` | solver routing, phase redistribution, lot changes and rebuilt portions | conserved ownership and projection certificates |
| `phreeqc/src/{aqueous,inventory,acceptance}.rs` | native replacement, phase/site/gas reconstruction and cache restoration | raw totals, witnesses and proposed owner budgets |
| `cea/src/{thermal,closed}.rs`, `sundials/src/lib.rs` | thermal/gas and kinetic proposals | scalar solver inputs and inventory/energy closure |
| Core reaction modules | adsorption, combustion, corrosion, curated, displacement, fermentation, gas tests, kinetics, nonaqueous, phase routing, starch/iodine, swelling, transport | every direct contents/moles write; proposed-state conservation |
| `core/src/{ledger,compartment,spill,nuclide}.rs` | secondary owner changes, spill recovery/discard, separate tracer updates | double counting, ownership and numerical reduction |
| `core/src/{scene,script,apparatus}.rs` and CLI adapters | import, setup, cloning, script persistence and conversion | cache keys, saved schemas and DTO boundaries |
| Render/appearance/optics/selectivity/ionic/safety/quest and bindings | no implicit writes through scalar views | support status, measurements, safety and UI projections |

Acceptance must cover tiny changes against bulk, widely separated scales,
duplicate entries, cancellation, exact exhaustion, malformed saves, rollback,
cross-owner transfer and resumed operation after reload. Still's existing
`1e-14` cuts and latent energy contracts remain binding; a scalar-relative
debit guard cannot replace a complete donor/receiver/energy certificate.

## What remains open

The stock port is the bounded first owner, subject to hosted integration gates.
NUM-01 remains open for the exhaustive semantic inventory and frozen vessel
API/save contracts. NUM-02 migrates all mechanical writers; NUM-03 certifies
chemical reconstruction; NUM-04 closes still donor/receiver/energy precision.
Do not describe primitive or stock tests as closure of those vessel gaps.

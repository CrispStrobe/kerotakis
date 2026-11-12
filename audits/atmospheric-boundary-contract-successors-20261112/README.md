# Atmospheric boundary contract successors — 2026-11-12

These are proposed source contracts, **not executed acceptance**. The current
hosted candidate predates them. No build, engine, API execution, commit or push
was performed while preparing these successors.

`*.original` preserves the complete three original test files; `original-files.json`
records their SHA256 identities. The unchanged first producer ran source
`583a681dc2f477f7bda054f3d00ed11ccfd2698b` with `cargo test --locked --workspace
--no-fail-fast`, exit101. Its full stdout SHA256 is
`34a5e134718f238fbcd6f4f0c1f5d2ac4fd69230160430877f70a53f17cdf4d1`;
stderr SHA256 is `7e625309d2bd01c1bca8a77ce002da2f2d13d6bd3343bea26628cc098f205353`.
`first-failure-excerpts.txt` copies the three actual failed assertions without
rewriting their messages. Full raw evidence and genuine producer lock remain
with that producer; these excerpts do not claim a complete evidence capsule.

The approved boundary separates zero-time addition, the bounded curated
acid/carbonate endpoint and finite-time film transfer. See
[the repair scope](../../docs/carbonate-boundaries-20261112.md),
`GasExchangeClock::advance` in `crates/kerotakis-core/src/clock.rs` and the
native clock owner in `crates/kerotakis-phreeqc/tests/clock_co2_boundary.rs`.
The coefficient is a lake-derived analogy, not an empirically calibrated
beaker yield.

The carbonate test previously required an atmospheric-equilibrium escape
fraction after only one hour. Its successor requires a positive export smaller
than the actual pre-interval neutral CO2, plus independent Na/C/H/O and charge
closure at1e-10. Its zero-time and original8.0–9.7pH controls remain unchanged.
The headspace test retains all original sealed-carbon, inert-gas, escaped-event
and pressure bounds. Its unsupported sealed/open pH ordering becomes independent
open-carbon closure against actual escaped gas at the original2e-7bound.
The acid perturbation retains every original alkalinity, sodium, pH-direction
and carbon-loss bound. Its obsolete room-pCO2 equality becomes finite positive
liquid supersaturation relative to the declared room boundary for these exact
one-hour cases; no equality or calibrated magnitude is claimed.

Three companion tests in `clock_co2_boundary.rs` retain the exact original
litre bicarbonate and HCl-dose inputs. They independently reconcile every
represented element including H/O and charge at1e-10, preserve spectators,
exclude zero-time replay and sealed atmospheric transfer, and distinguish
acid-addition exports from subsequent film exports. They also require current
complete native answers and fail on solver errors. The sealed companion keeps
actual trapped-air carbon, never an assumed zero atmosphere.

Required next acceptance: execute these exact-source affected tests plus the
existing clock, carbonate, headspace and perturbation targets using a genuine
producer lock/binary binding. Check original retained invariants and all new
closure bounds. A new failure remains a failure; do not bless goldens, broaden
tolerances or reuse the older candidate's acceptance. Full applicable source,
platform and own-fleet requirements remain with the containing PR.

## First successor execution: pure-water test precondition

The actual `b2598f4411c47969cd7250f2bd4eab9ada403006` full-workspace
producer exposed a test-authoring error before two successor protocols reached
any bicarbonate or acid addition. `original_boundary_step` demanded `Complete`
for their initial open pure-water setup, but the analytic water-autoionisation
owner correctly reports `SolventOnly`. The sealed companion passes: its trapped
gas makes the water step a different native question. Verbatim raw failures and
original source/command/output hashes are preserved in
`solvent-only-first-failure.txt` and `solvent-only-first-producer.json`.

The corrected precondition applies only to an actual water-add operator targeting
vessel0 while that vessel is open, empty across every represented inventory and
has no pending CO2 carrier. The returned stock must remain exclusively liquid
water and must explicitly report `SolventOnly` with provenance. Finite pH and ionic
strength checks remain. Every subsequent solute or transfer step and the sealed
setup still requires `Complete` and the current numerical particle census.
All gas, element, H/O, charge, spectator and alkalinity assertions are unchanged.
This correction has only formatting and diff validation; its own exact-source
runtime acceptance remains pending.

The first source-bound solvent setup correction at `0b3baf11` did not compile: `Bench::vessel` returns `Result`, so `is_some_and` was invalid. The [first compiler failure](solvent-only-compile-first-failure.txt) and [producer command bindings](solvent-only-compile-first-producer.json) are preserved. The successor uses `is_ok_and` with the identical predicate; its changed-source execution remains required.

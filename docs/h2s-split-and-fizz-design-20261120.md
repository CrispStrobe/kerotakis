# H2S / S(-2) gaps — agent report (2026-11-20)

Source-only review of branch fix/round7-gaps-20261120 at 7015a9974; nothing was changed or compiled.

## A. H2S/HS-/S-2 protonation split — BLOCKED (not by data)

Verified (pinned iphreeqc 47b459c4, read from root checkout; worktree submodule is uninitialised):
- wateq4f: master `S(-2) H2S`; `H2S = HS- + H+` (l.533); `HS- = S-2 + H+` (l.540, -12.918).
- minteq.v4: `H+ + HS- = H2S` (l.2670); `HS- = S-2 + H+` log_k -17.3 (LMa1987), delta_h 49.4 kJ (l.486).
- pitzer: no S(-2) -> all-or-nothing readback (aqueous.rs ~9920) falls back to booking ion `HS-`. Fine.
- Namespace: S and N are redox-isolated (`redox_isolation::transform`); `native_namespace::input` renames
  `-molalities` words through `db.species`, the same path the N(-3) NH3/NH4+ split already uses. So
  asking for `m_H2S`, `m_HS-`, `m_S-2` is mechanically supported (engine-unverified).
- Registry `S-2` entry is NOT a data blocker: aqueous-basis ions use convention placeholders (Cp 0,
  density 1.0, source `kerotakis/aqueous-basis-v1`, as `HS-`/`SCN-`); molar mass 32.06 (CIAAW 2021).
  No cited value would need inventing. The 4.4-log-unit wateq4f/minteq disagreement is also not a
  blocker by the module's own rule ("the split is the solve's own"): each route uses its own K.
  It should be documented as route-dependent, not reconciled.

The actual blocker: the neutral share. The split row would be
`("S(-2)", [("H2S","hydrogen_sulfide"), ("HS-","HS-"), ("S-2","S-2")])`, booking H2S(aq) under the
gas key (SO2 precedent). But `derived::role("hydrogen_sulfide")` is `None` (asserted by
`sodium_sulfide_books_reduced_sulfur_and_nothing_else_does`, derived.rs ~2370). An aqueous
`hydrogen_sulfide` portion produced by the readback would therefore be unposable on the next step
(lost/refused element). Giving it a `Dissolves([S(-2)])` role is exactly what the fourth pass
(docs/sulfide-coverage-20261119.md) deliberately deferred: that role also applies to a gas dose,
and the finite-headspace path does not admit `H2S(g)` (`gas_component_elements` returns bare `S`,
no S(-2) special case like NH3/N(-3)), and no H2S clock exists, so a dose would enter and never leave.
Alternative (book H2S(aq) under a new `H2S(aq)` basis key) avoids touching the gas role but adds a
second new registry key and needs a phase-aware decision on how `hydrogen_sulfide` gas and `H2S(aq)`
relate in safety/strong_reserve (`("HS-","sulfide")` row) — a design decision for the owner.

Second practical blocker under the no-compile rule: a new registry key regenerates many tool-generated
coverage docs (75ec26875 touched 36 files: golden registry.json, registry-source-v1.json, exporter,
legacy_diff, loader count contracts, input-alias fixture, chromophore/stocking/transition/ion-partition/
iodine/crystallisation coverage JSON+MD, locales core+web, safety, strong_reserve, inchi_validate).
Those are produced/checked by running code; hand-editing them blind is likely to fail `--check` gates.

Exact prerequisite list to land A:
1. Decide neutral key: (a) role for `hydrogen_sulfide` = Dissolves S(-2) PLUS S(-2)-gated finite-headspace
   admission (`gas_component_elements` override + supplied-gas and empty-component special cases as N(-3))
   and an H2S clock or explicit zero-time-only offer; or (b) new aqueous-basis key `H2S(aq)`.
2. Add registry `S-2` (and `H2S(aq)` if 2b) with aqueous-basis placeholders; regenerate golden/exported/
   coverage files by running the generators (needs a machine that may build).
3. Add the PROTONATION_SPLITS row; update the derived.rs S(-2) comment, keep `("S(-2)","HS-")` booking ion.
4. Safety/strong_reserve rows for `S-2` (Sulfide, ReducingAgent, ToxicSoluble; BaseStrong? — needs review)
   and en/de/fr names.
5. Engine tests: Na2S solution on wateq4f shows ~28 % S-2 near pH 12.5, minteq ~0.002 %; acidified
   sulfide books mostly H2S(aq); pitzer falls back to HS-; conservation of S(-2) across two steps
   (the round-trip that blocks today).

## B. Sub-saturation H2S fizz — design only (blocked on A)

Today the parcel is CO2-specific end to end: `carbonate_endpoint::{prepare,Dose::extent,run}` reads
`CO2(aq)` pool, compares `co2_partial_pressure_atm` to `clock::ATMOSPHERIC_CO2_ATM`, and spends via
`Vessel::pending_co2_transfer_mol` (41 uses in phreeqc aqueous.rs, 31 in clock.rs, plus inventory.rs,
delta.rs, vessel.rs, aqueous_continuation.rs, named_alkaline.rs). The neutral H2S pool cannot be read
until A books H2S(aq) under its own key, so B cannot start before A.

Proposed generalisation:
- `enum EndpointGas { Co2, H2s }` with a static descriptor: neutral pool key (`CO2(aq)` / H2S key from A),
  gas registry key (`CO2` / `hydrogen_sulfide`), source pools (carbonates / Na2S,HS-,S-2), domain
  whitelist for `medium()`, atmospheric threshold (CO2 constant / H2S log p -12 from derived ATMOSPHERIC),
  pKa gate (carbonic_acid_pk1 / H2S pK1 — must come from the route's solve, not a constant).
- `Dose { gas, neutral_before, capacity }`; `prepare` returns `Vec<Dose>` (at most one per gas).
- Replace `pending_co2_transfer_mol: f64` by `pending_gas_transfer: BTreeMap<GasKey, f64>` (or add a
  sibling `pending_h2s_transfer_mol` first, to keep the CO2 path byte-identical — preferred for review).
  The phreeqc adapter must consume it by fixing the H2S(g) export exactly like CO2 (GAS_PHASE/
  EQUILIBRIUM_PHASES offer gated on S(-2), never S(6)).
- `run` per dose on one clone, sequentially; the whole candidate commits only if every dose's
  `GasEvolved` equals its amount within 1e-12, no matching `GasAbsorbed`, pending all zero, fresh
  receipt certifies, `validate_conservation` 1e-10, charge 1e-10; else original vessel untouched.
- Typed note analogous to `notice()`: new key `not-modeled.sulfide-endpoint` (en/de/fr), not a
  calibrated yield.
- bench.rs ~1011: rename `committed_carbonate_endpoint` to a generic flag; `coalesce_external_gas_crossings`
  must coalesce H2S crossings too.

Invariants: no CO2 behaviour change; capacity = min(acid added, sulfide present) on strong acids only;
redox-active pairs (nitrate/halide, any oxidant with S(-2)) excluded from the domain; pitzer refuses.

## Tests hosted CI must run (nothing here was compiled)
No source changed, so nothing new to validate from this agent. For the existing sulfide work on the
branch (and as regression set for any later A/B change):
- kerotakis-phreeqc: `derived::tests::sodium_sulfide_books_reduced_sulfur_and_nothing_else_does`,
  `monosulfides_map_to_s_minus_2_and_nothing_else_does`, `no_registry_role_books_sulfide_yet`,
  `hydrogen_sulfide_room_sink_is_gated_on_reduced_sulfur_only`; tests/sulfide_family.rs;
  tests/sulfite_speciation.rs; tests/redox_isolation_probe.rs; tests/gas_phase coverage;
  tests/clock_co2_boundary.rs; tests/curated_reactants_survive_a_solve.rs.
- kerotakis-core: carbonate_endpoint unit tests (5); tests/aqueous_continuation.rs,
  solver_transactions.rs, solver_snapshot_contracts.rs, ordinary_delta_state_contracts.rs,
  composed_locale.rs, golden registry tests, registry_loader_* contracts.
- kerotakis-safety (sulfide tests), kerotakis-registry-export legacy_diff, kerotakis-org native_identity.

## Status of B (2026-11-20)

Implemented as the preferred sibling, keeping the CO2 path unchanged:
`Vessel::pending_h2s_transfer_mol` (omitted from saved state while zero) is
written only by the acid/sulfide arm of `carbonate_endpoint` (`EndpointGas::H2s`)
and spent by the PHREEQC adapter against the S(-2) total, beside the clock CO2
parcel: `Problem::h2s_parcel_applied_mol`, `ClockCo2Boundary::h2s_applied_mol`
in the inventory certificates, the whole-step heat gas ledger, the canonical
candidate (spent once, announced once as `GasEvolved hydrogen_sulfide`), MIX and
continuation exclusions, and `validate_clock_boundary`. The domain is sodium
sulfide with HCl/HBr/HI only (no nitrate or other oxidant, no carbonate, no weak
buffer), ionic strength at most 1, no pK gate. Typed note
`not-modeled.sulfide-fast-endpoint` (en/de/fr). The half-neutralised round-trip
test now seals its vessel, since an open beaker vents the newly formed H2S(aq).

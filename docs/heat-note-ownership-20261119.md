# Heat-note ownership: metal + acid (2026-11-19)

## Defect

`water 100 mL / HCl 0.1 mol / Zn 1 g` (or Mg) emitted
`not-modeled.aqueous-heat-unpriced` ("the reaction heat of this step is not
modelled ... the temperature stays where the step began") while the
temperature rose by exactly the displacement heat (Mg 25.00 -> 36.05 C,
Zn 25.00 -> 30.64 C).

## Root cause

`DisplacementEquilibrator` (`crates/kerotakis-core/src/displacement.rs`)
charges `-dH * xi` from formation enthalpies and moves the vessel
temperature itself. The aqueous tail (`crates/kerotakis-phreeqc/src/aqueous.rs`,
heat quote ~line 8404) prices the WHOLE step from `Vessel::step_start`, so it
sees the same metal solid leave. Elemental metals are deliberately unpriced
by `enthalpy::solid_route` ("owned by displacement"), so the quote declines,
`HeatMode::Unpriced` holds the post-displacement temperature, and the refusal
note is emitted (~line 4590). The temperature was right only because the
decline happened to prevent a double charge; the note was false.

## Change

- `StepStart::heat_paid` (core `vessel.rs`): signed inventory changes whose
  heat an earlier solver of this step already charged, with
  `note_heat_paid` / `contents_after_paid`.
- Displacement books its swap (inventory diff, plus evolved H2 as gas out)
  on the step snapshot.
- When the tail's quote declines, it re-quotes from
  `contents_after_paid()`. If that remainder prices completely, the refusal
  note is suppressed (`last_heat_paid_elsewhere`). Heat mode, `q`, and the
  temperature path are unchanged — no temperature result moves.
- A decline with nothing paid elsewhere (or an unpriced remainder) keeps the
  note.

## Other reported cases

- Fe + CuSO4: same displacement route; expected fixed by the same booking
  (Fe solid is the unpriced mover). Not executed.
- Pb(NO3)2 + KI (PbI2): no other owner charges this heat (no displacement,
  no curated precipitation heat in core). PbI2 has an LLNL dissolution
  reference, so the declining mover is something else; the note is
  presumably genuine and is kept. Needs a run with `KERO_BAL=1` to name it.

## Not done / known limits

- The aqueous remainder after a displacement (e.g. ZnCl+ complexation,
  dilution of the remaining acid) is still not charged; only the false
  refusal is removed. Charging it would change temperatures.
- Tests added in `crates/kerotakis-phreeqc/tests/displacement.rs`, not run.

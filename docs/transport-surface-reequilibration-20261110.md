# Transport re-equilibrates bound pools every step

## The defect

A charcoal cell loaded with methyl orange (about 0.86 mg on 0.02 g in a
20 mL cell) kept its bound amount unchanged through a three-pore-volume
clean-water flush, although the model's own Langmuir isotherm put that
loading far off equilibrium at the flushed concentration.

The transport loop already re-ran the full solver stack in every cell after
every step. The adsorption rung skipped the move: it compared each step's
deficit (about 0.08 mg) with the 0.1 mg threshold for reporting an event,
and that threshold was also deciding whether any matter moved. Each step's
deficit was under the threshold, so the bound pool never changed.

## The fix

`kerotakis_core::adsorption` now decides whether to move matter with a
numerical tolerance (1e-9 of the pair's total sorbate, floor 1e-15 mol),
in both `equilibrate` and `applies`. The 0.1 mg threshold only decides
whether an `Adsorbed` event is reported. Moves stay exact transfers between
the dissolved portion and `Vessel::adsorbed` (withdraw then bind, or release
then deposit). Dissolved plus bound plus transferred plus inflow therefore
closes to floating-point rounding.

The transport contract is unchanged. The inlet is a constant external
reservoir by default; `drain` is opt-in; `inflow_moles` is recorded; and
every cell re-equilibrates the full stack after every step, followed by the
ordinary post-operator pass.

## Coverage

| Mechanism | Adsorbent / solute pairs | Transport behaviour |
|---|---|---|
| Curated Langmuir isotherm (`adsorption::ISOTHERMS`) | activated charcoal / methyl orange | Covered: adsorbs on loading and desorbs on flush, every step |
| PHREEQC interface (`Vessel::surfaces`, hydrous ferric oxide) | Zn+2, SO4-2 (closed `SurfaceSorbate` enum) | Covered: the aqueous tail applies whenever surfaces exist and recomputes occupancy from the current pore water each step; no event threshold is involved |
| Any other solid / solute (other dyes, other carbons, clays, resins, ion exchange) | none | Not modelled: there is no bound pool, and the solute travels as a non-sorbing species. Unknown pairs are never given a borrowed isotherm |

Not claimed: adsorption or desorption rates (each step is an equilibrium,
so desorption reaches the isotherm within one step), hysteresis, pH or
temperature dependence, and competition between sorbates.

## Tests

`crates/kerotakis-core/tests/transport_surface_reequilibration.rs`:

- a load-then-flush column desorbs toward the isotherm, every cell ends on
  its isotherm, and the dye ledger closes to 1e-12 relative
- a deficit below the event threshold still moves matter, and the release
  is exact
- a dye front is adsorbed as it arrives, and the open-column ledger
  (before + inflow) closes
- a column without a sorbent is identical to one run with a no-op solver

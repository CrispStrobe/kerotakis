# Gas-phase coverage across routed databases (2026-10-27)

Generated from `kerotakis_phreeqc::gas_phases::coverage()` and pinned by
`crates/kerotakis-phreeqc/tests/gas_phase_coverage.rs` (fixture
`crates/kerotakis-phreeqc/tests/fixtures/gas_phase_coverage.json`). The test fails
when either table below no longer matches the code; regenerate the fixture with
`KEROTAKIS_BLESS_GAS_COVERAGE=1 cargo test -p kerotakis-phreeqc --test gas_phase_coverage`
and paste `markdown_table` / `coupling_table` output here.

## Mechanism

Candidates are enumerated, not hand-listed: every `PHASES` entry ending in `(g)` in
any routed database (wateq4f, minteq.v4 as extended, pitzer) or donor, every
registry species whose standard phase is gas (`{formula}(g)`), `EQUILIBRIUM_GASES`
and `NH3(g)`.

A route lacking a gas imports the donor's phase block verbatim (donor priority:
vendored USGS `wateq4f.dat`, `phreeqc.dat`, `minteq.v4.dat`, `pitzer.dat`; all
U.S. Government public domain under the USGS User Rights Notice) only if:

1. the donor phase states log K and a temperature dependence (`delta_h` or `-analytic`);
2. every linking aqueous species exists on the route;
3. each linking species, expanded recursively to identity-defined master species,
   has the same stoichiometry on both databases, log K within 0.02 and ΔH within
   4 kJ/mol at 25 °C;
4. a redox-coupled dissolution is only imported where the route speciates every
   non-H/O element of the gas in more than one oxidation state.

Otherwise the cell is a typed refusal, and the aqueous adapter refuses any headspace
that would pose that gas (`solver-reason.aqueous.gas-phase-unavailable`, EN/DE/FR;
it replaces the NH3-only `ammonia-gas-unavailable` guard and the silent
`retain` that dropped unknown gas components).

Counts: before this change minteq.v4 lacked NH3(g) (one-off extension on this branch)
and H2(g). After: 33 native cells, 2 imported (NH3(g) and H2(g) onto minteq.v4,
both from wateq4f; NH3 link Δlog K 2.2e-4, ΔΔH 0.21 kJ/mol; H2 link identical),
67 refused cells.

## Coverage

| gas | wateq4f | minteq.v4 | pitzer |
|---|---|---|---|
| `C2H4(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `C3H8(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `C4H10(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `C4H8(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `CH4(g)` | native | native | refused (wateq4f: linking-species-absent `CH4`; phreeqc: linking-species-absent `CH4`; minteq.v4: redox-state-not-modelled-on-route `CO3-2`) |
| `CO(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `CO2(g)` | native | native | native |
| `Cl2(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `H2(g)` | native | **imported** from wateq4f | refused (wateq4f: linking-species-absent `H2`; phreeqc: linking-species-absent `H2`) |
| `H2O(g)` | native | refused (wateq4f: no-linking-aqueous-species; phreeqc: no-linking-aqueous-species; pitzer: no-linking-aqueous-species) | native |
| `H2S(g)` | native | native | refused (wateq4f: linking-species-absent `H2S`; phreeqc: linking-species-absent `HS-`; minteq.v4: linking-species-absent `HS-`) |
| `H2Se(g)` | refused (minteq.v4: linking-log-k-outside-tolerance `HSe-`) | native | refused (minteq.v4: linking-species-absent `HSe-`) |
| `H2Sg(g)` | refused (phreeqc: linking-species-absent `HSg-`; pitzer: linking-species-absent `HSg-`) | refused (phreeqc: linking-species-absent `HSg-`; pitzer: linking-species-absent `HSg-`) | native |
| `HBr(g)` | native | native | native |
| `Hdg(g)` | refused (phreeqc: linking-species-absent `Hdg`; pitzer: linking-species-absent `Hdg`) | refused (phreeqc: linking-species-absent `Hdg`; pitzer: linking-species-absent `Hdg`) | native |
| `He(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `Hg(CH3)2(g)` | refused (minteq.v4: linking-species-absent `Hg(OH)2`) | native | refused (minteq.v4: linking-species-absent `Hg(OH)2`) |
| `Hg(g)` | refused (minteq.v4: linking-species-absent `Hg2+2`) | native | refused (minteq.v4: linking-species-absent `Hg2+2`) |
| `Hg2(g)` | refused (minteq.v4: linking-species-absent `Hg2+2`) | native | refused (minteq.v4: linking-species-absent `Hg2+2`) |
| `HgBr(g)` | refused (minteq.v4: linking-species-absent `Hg2+2`) | native | refused (minteq.v4: linking-species-absent `Hg2+2`) |
| `HgBr2(g)` | refused (minteq.v4: linking-species-absent `Hg(OH)2`) | native | refused (minteq.v4: linking-species-absent `Hg(OH)2`) |
| `HgCl(g)` | refused (minteq.v4: linking-species-absent `Hg2+2`) | native | refused (minteq.v4: linking-species-absent `Hg2+2`) |
| `HgF(g)` | refused (minteq.v4: linking-species-absent `Hg2+2`) | native | refused (minteq.v4: linking-species-absent `F-`) |
| `HgF2(g)` | refused (minteq.v4: linking-species-absent `Hg(OH)2`) | native | refused (minteq.v4: linking-species-absent `F-`) |
| `HgI(g)` | refused (minteq.v4: linking-species-absent `Hg2+2`) | native | refused (minteq.v4: linking-species-absent `Hg2+2`) |
| `HgI2(g)` | refused (minteq.v4: linking-species-absent `Hg(OH)2`) | native | refused (minteq.v4: linking-species-absent `Hg(OH)2`) |
| `Mtg(g)` | refused (phreeqc: linking-species-absent `Mtg`; pitzer: linking-species-absent `Mtg`) | refused (phreeqc: linking-species-absent `Mtg`; pitzer: linking-species-absent `Mtg`) | native |
| `N2(g)` | native | refused (wateq4f: linking-species-absent `N2`; phreeqc: linking-species-absent `N2`) | refused (wateq4f: linking-species-absent `N2`; phreeqc: linking-species-absent `N2`) |
| `NH2Cl(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |
| `NH3(g)` | native | **imported** from wateq4f | refused (wateq4f: linking-species-absent `NH3`; phreeqc: linking-species-absent `NH3`) |
| `Ntg(g)` | refused (phreeqc: linking-species-absent `Ntg`; pitzer: linking-species-absent `Ntg`) | refused (phreeqc: linking-species-absent `Ntg`; pitzer: linking-species-absent `Ntg`) | native |
| `O2(g)` | native | native | refused (wateq4f: linking-species-absent `O2`; phreeqc: linking-species-absent `O2`; minteq.v4: no-linking-aqueous-species) |
| `Oxg(g)` | refused (phreeqc: linking-species-absent `Oxg`; pitzer: linking-species-absent `Oxg`) | refused (phreeqc: linking-species-absent `Oxg`; pitzer: linking-species-absent `Oxg`) | native |
| `SO2(g)` | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) | refused (no-reviewed-donor-definition) |

## Finite headspace coupling to aqueous acid/base (#834 siblings)

Only gases posed to the native solve (`EQUILIBRIUM_GASES` plus the NH3 coupling of
#834) equilibrate with protonation in the same solve. The others are carried as inert
headspace inventory: no Henry exchange and no acid/base suppression.

| gas | coupled to aqueous acid/base in finite headspace | wateq4f | minteq.v4 | pitzer |
|---|---|---|---|---|
| `NH3(g)` | yes | coupled | coupled | refused (gas-phase-unavailable) |
| `CO2(g)` | yes | coupled | coupled | coupled |
| `HBr(g)` | yes | coupled | coupled | coupled |
| `H2S(g)` | no (inert inventory) | phase defined, not posed | phase defined, not posed | no phase |
| `SO2(g)` | no (inert inventory) | no phase | no phase | no phase |
| `HCl(g)` | no (inert inventory) | no phase | no phase | no phase |
| `Cl2(g)` | no (inert inventory) | no phase | no phase | no phase |

Remaining gaps (not changed here; each needs reviewed data and closure controls):

- `H2S(g)`: phases exist on wateq4f and minteq.v4, but posing it couples reduced
  sulfur to pe (sulfate ⇄ sulfide) the same way unrestricted N2/O2 would nitrate room
  air; it needs a redox-isolated S(-2) capability before being added to
  `EQUILIBRIUM_GASES`. Pitzer has no reduced sulfur outside the uncoupled `H2Sg`.
- `SO2(g)`, `HCl(g)`, `Cl2(g)`: no vendored USGS database defines the phase. They need a
  reviewed Henry-law source (as HBr used Sander 2023) with an explicit standard-state
  conversion; `Cl2` also needs Cl(0)/hypochlorite redox ownership.
- `NH3(g)` on pitzer: pitzer.dat carries no nitrogen; the refusal is moot because
  N(-3) never routes there.

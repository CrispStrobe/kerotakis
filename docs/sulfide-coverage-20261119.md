# Reduced-sulfur (sulfide) family coverage, 2026-11-19

Scope: how reduced sulfur, S(-2), may enter an aqueous problem. Implemented in this lane:
the explicit mapping (step 1) only. Steps 2-5 are assessed below and deliberately NOT
implemented, because each depends on engine behaviour that cannot be verified without
running PHREEQC (this lane ran no compiler, test or engine).

## Mechanism (implemented)

`extract_sulfide` in `crates/kerotakis-phreeqc/src/derived.rs`: a formula books `S(-2)`
only when its own cations and protons pay every sulfur's two negative charges,
`sum(booked cation charge) + H == 2 * S`, with no O, C, N or halide present. Bare `S`
is not added to `RESIDUE_OK`, and `oxyanion_groups()` gains no `S` row.

| formula | result | reason |
|---|---|---|
| `H2S` | `S(-2)` 1 | 2 protons pay 2 |
| `NaHS` | `S(-2)` 1, `Na` 1 | 1 + 1 = 2 |
| `Na2S`, `K2S`, `Ag2S` | `S(-2)` 1 + cation | 2 x 1 = 2 |
| `FeS` | `S(-2)` 1, `Fe(2)` 1 | Fe booked Fe+2 (BOOKING_OVERRIDES) |
| `ZnS`, `PbS`, `CuS`, `MnS` | `S(-2)` 1 + cation | divalent booking |
| `S`, `S8` | refused | no charge paid (elemental sulfur) |
| `FeS2` (pyrite), `Na2S2` | refused | disulfide: half paid |
| `Cu2S` (chalcocite) | refused | Cu books as Cu(2); no Cu(1) aqueous booking |
| `Na2SO4`, `Na2SO3`, `Na2S2O3`, `KSCN` | unchanged | owned by SO4 / sulfite / thiosulfate / SCN rules |
| ions (`HS-`, `S-2`) | not by this rule | parser drops charge (`S-2` == `S` by counts); a registry ion would map through the database-master branch |

Route coverage of the `S(-2)` element (vendored USGS databases, public domain):

| route | master | gas phase | monosulfide minerals |
|---|---|---|---|
| wateq4f | `S(-2) H2S` (line 62) | `H2S(g)`: `H2S = H2S`, log_k -0.997, delta_h -4.57 kcal | Mackinawite, Sphalerite, Galena, Covellite (also Pyrite, Chalcocite, Sulfur - refused/unmatched) |
| minteq.v4 | `S(-2) HS-` (line 70) | `H2S(g)`: `H2S = H+ + HS-`, log_k -8.01 | Galena, Sphalerite, Covellite, Mackinawite (also Pyrite, Chalcocite, Sulfur) |
| pitzer | none | only the pseudo-element `H2Sg(g)`, not coupled to S | none |

Effect today: none on any registry role. The shelf's only oxygen-free sulfur formulas are
`S` (refused), `hydrogen_sulfide` (a gas with no `EQUILIBRIUM_GASES` entry, so no role) and
`KSCN`/`SCN-` (own rule). Database monosulfide phases now derive contributions but are
only candidates when a registry solid matches them, and none exists. Pinned by
`monosulfides_map_to_s_minus_2_and_nothing_else_does` and
`no_registry_role_books_sulfide_yet`.

## Not implemented, and why

1. **Redox ownership is the blocker, not data.** An open vessel poses
   `pe 4 O2(g) -0.68` (`aqueous.rs`, `ATMOSPHERIC_OXYGEN`). PHREEQC at that pe converts
   S(-2) to sulfate: thermodynamically true, kinetically wrong (sulfide air-oxidation is
   slow on bench time). Iron and nitrogen avoid this by valence-tagged totals, but whether a
   tagged `S(-2)` total survives the reaction step (EQUILIBRIUM_PHASES / gas exchange) with
   S(6) also present cannot be established without running the engine. The 2026-10-27 gas
   coverage note already records that posing `H2S(g)` "couples reduced sulfur to pe".
   Needed: a redox-isolated S(-2) capability (e.g. a borrowed pseudo-element as pitzer's
   `Sg`, or a decoupled master such as the sulfite/thiosulfate extensions), with an
   open-beaker Na2S control that keeps its sulfide.
2. **Registry Na2S** needs: identity, composition, phase_thermodynamics (molar mass
   78.04 g/mol computes from IUPAC atomic weights; density, solubility and appearance need
   a primary source - NBS/CRC not consulted in this lane, so no numbers were written),
   golden `registry.json`, exporter legacy diff, locales en/de/fr, safety row, solid
   coverage docs, plus a booking ion for `S(-2)` (`HS-` registry ion or a
   `BOOKING_OVERRIDES` row) or `bookings_cover_every_derivable_element` panics. The
   precedent commit 378761da (Cu2O) touched 21 files; hand-writing that without the
   exporter and tests is unsafe.
3. **Henry constant**: wateq4f and minteq.v4 already carry native `H2S(g)` phases (table
   above), so a Sander-compilation value is not needed for those routes; pitzer has no
   S(-2) and must refuse (`gas-phase-unavailable`).
4. **ATMOSPHERIC H2S(g) at log p -12** (room air holds none at bench resolution): blocked
   by item 1. The `all_present` gate in `aqueous.rs` keys on element names; it must key on
   `S(-2)` specifically, never on `S(6)`.
5. **Fizz** (`carbonate_endpoint.rs` sibling for neutral H2S): blocked by items 1-2.

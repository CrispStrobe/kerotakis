# Reduced-sulfur (sulfide) family coverage, 2026-11-19

Scope: how reduced sulfur, S(-2), may enter an aqueous problem. Implemented in this lane:
the explicit mapping (step 1) and, in a second pass, the gated H2S(g) boundary row plus its
typed note (see "Implemented in the second pass"; the redox blocker below was already solved). Steps 2-5 are assessed below and deliberately NOT
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

## Redox ownership: already solved by the native namespace (second pass, 2026-11-19)

The first pass of this lane named redox the blocker. Reading the solve path shows it is not:
every native run goes through `AqueousEquilibrator::run_raw`, which translates the input with
`native_namespace::input` against `redox_isolation::transform(db)`. That transform (USGS PHREEQC
Example 4 / FAQ technique) gives each N and S oxidation state its own conserved master element
(`S(-2)` -> `Sredtwo`, `S(6)` -> `Soxsix`), rewriting species, gas and mineral equations from
their native stoichiometry. No reaction then links `Sredtwo` to O2, so `pe 4 O2(g) -0.68`
cannot turn sulfide into sulfate within a step, at any pe. This is the same mechanism that
keeps ammonium beside room air and nitrate (the Fe/Mn/Cu/Cr held-split respeciation is a
different, metal-only device). Engine evidence that already exists (not rerun here):
`tests/redox_isolation_probe.rs::isolated_totals_survive_native_batch_and_mix_at_arbitrary_pe`
holds 1e-4..0.1 mol/kgw `S(-2)` beside equal `S(6)` at pe -2, 4 and 10 through REACTION and
MIX to 1e-7 relative. The 2026-10-27 note that posing H2S(g) "couples reduced sulfur to pe"
predates or ignores this transform: `H2S(g)` is rewritten onto `Sredtwo` and couples only to it.

## Implemented in the second pass (untested: no compiler or engine was run)

1. **ATMOSPHERIC H2S(g) sink.** `derived::ATMOSPHERIC` gains `("H2S(g)", "hydrogen_sulfide",
   -12.0)`; log p -12 is a numerical stand-in for "room air holds none", not a measured value.
   `derived::atmospheric_gate` / `atmospheric_row_applies` admit it only when the partition's
   elements include `S(-2)` (sulfate's `S(6)`, `Sulfite`, `Thiosulfate` and bare `S` never do);
   CO2's ungated row is unchanged. `aqueous.rs` uses the helper in the open/swept reservoir
   loop, with one further restriction: a gated row is offered only where the step's target is
   NOT the -12 sink, i.e. a zero-time step (any add: effervescence threshold log10(P/atm)) or
   an inert sweep. No clock owns H2S, so on a timed open step (Wait > 0 s) the -12 sink would
   strip essentially all sulfide even from alkaline Na2S (HS- + H2O -> H2S + OH- driven to
   completion) in one step; there the sulfide is retained instead. The -12 value is therefore
   currently a recorded constant, not a reached target.
2. **Typed note** `not-modeled.sulfide-open-air` (en inline, de/fr in `i18n/*.toml`), cause
   `RateNotModelled`, emitted on every open or swept solve whose totals hold `S(-2)`: sulfide
   air-oxidation (hours to days) is not modelled, none is converted; H2S leaves only as bubbles
   above saturation or into a sweep, slow escape to room air is not modelled.
3. **Tests (not run).** `derived::tests::hydrogen_sulfide_room_sink_is_gated_on_reduced_sulfur_only`;
   engine test `open_beaker_oxygen_does_not_oxidise_sulfide_and_the_room_sink_strips_only_h2s`
   (alkaline Na2S keeps 0.001 mol/kgw sulfide to 1e-10 with sulfate < 1e-15 under
   `pe 4 O2(g) -0.68`; acidified 0.005 mol/kgw sulfide with `H2S(g) -12 0` is left < 1e-10
   while 0.001 mol/kgw sulfate beside it is unchanged).

Effect today: none. No registry role books `S(-2)` (pinned), so the gate never opens and no
current run changes. The H2S(g) phase exists on wateq4f and minteq.v4; pitzer has no `S(-2)`
master, so a sulfide solution cannot route there (route selection requires every element).

## Still not implemented, and why

1. **Fizz analogous to `carbonate_endpoint.rs`.** Partially covered by the reservoir itself:
   an add step is zero-time, so the H2S(g) row targets the effervescence threshold and any
   H2S(aq) above saturation at the vessel pressure leaves in that step (K_H from wateq4f
   `H2S(g)` log_k -0.997, about 0.10 mol/kgw/atm). A sub-saturation "newly formed neutral
   parcel" endpoint as for CO2 was NOT written: the CO2 endpoint rides on CO2-only plumbing
   (`pending_co2_transfer_mol`, `co2_film_boundary`, the film clock, `medium()` whitelist,
   `already_exported` matched on species "CO2"). A sibling needs a generic pending-gas parcel
   in `Vessel` and adapter, which cannot be made correct without running the engine. Also,
   unlike CO2, no clock owns H2S, so timed open steps retain dissolved H2S (see item 1 above);
   a finite-time H2S film clock would be the correct owner and is not written.
2. **Registry Na2S: ADDED in the third pass (see below).** `hydrogen_sulfide` still has no
   dissolution role: adding it to `EQUILIBRIUM_GASES` would note bare `S`, which the isolated
   namespace maps to the primary master (S(6) on wateq4f); it needs an NH3-style `S(-2)` gas
   path with its own readback key, which is not clearly analogous (NH3 is a liquid registry
   row with an aqueous protonation split) and was not written.
3. **Pitzer**: no S(-2); must refuse typed (`gas-phase-unavailable`) if ever reached.

## Validator expectations once Na2S exists (by reading, not run)

- 0.005 mol Na2S + 0.02 mol HCl in 100 mL water, open beaker, add step: all sulfide is H2S(aq)
  (pH about 1, excess 0.01 mol H+), 0.05 mol/kgw, below about 0.10 mol/kgw saturation at
  1 atm, so H2S evolved in the add step is about 0 (below trace); the note
  `not-modeled.sulfide-open-air` appears; sulfate stays 0. A following `wait` evolves nothing
  (no H2S clock; sulfide retained, note repeated). Under an inert sweep the same solution
  evolves about 0.005 mol `hydrogen_sulfide`.
- 0.05 mol Na2S + 0.2 mol HCl in 100 mL: about 0.5 mol/kgw H2S formed; the add step evolves
  about (0.5 - 0.10) x 0.1 kg = 0.040 mol H2S (activity-coefficient and salting effects at
  I about 1.5 shift this by a few percent).
- 0.005 mol Na2S alone in 100 mL open beaker, add then wait 1 h: pH about 12.5-13, sulfide
  stays 0.005 mol as HS-/S-2 in both steps, sulfate 0, no gas event, note present.

## Third pass: sodium sulfide on the shelf (2026-11-19, untested)

Registry `Na2S` represents the ANHYDROUS salt (standard phase solid). Reason: every sourced
number describes it (NASA CEA `Na2S(cr)`, density 1.856), and this registry keys hydrates only
under their own names; the nonahydrate `Na2S.9H2O` that the trade sells (CAS 1313-84-4) is a
different solid and is not added.

| value | number | source |
|---|---|---|
| identity | InChIKey GRVFOGOEDUUMBP-UHFFFAOYSA-N, disodium;sulfide | PubChem PUG REST CID 14804, read 2026-11-19 |
| molar mass | 78.0395 g/mol | 2 x 22.98976928 + 32.06, IUPAC/CIAAW 2021 (S conventional) |
| Cp (scalar) | 82.80052821809429 J/(mol.K) | NASA CEA thermo.inp `Na2S(cr)` (Chase 1998, NIST-JANAF) at 298.15 K, R = 8.31446261815324 |
| density | 1.856 g/mL at 68 F | USCG CHRIS (1999, US Government work) as quoted by PubChem HSDB; Merck Index 1996 p. 1483 gives 1.856 at 14 C |
| appearance | colourless | Merck Index "cubic crystals or granules", NFPA 1978 "clear crystals" (both via HSDB); technical grades yellow/brick-red |
| solubility (documented, NOT shipped) | 12.4 (0 C), 18.6 (20 C), 29.0 (40 C), 39.0 (50 C) g/100 g water | Merck Index via HSDB; not shipped because the saturating solid below about 48 C is a hydrate (the Na2SO4 hydrate trap) |
| hazards | corrosive base, toxic (skin), reducing agent | ILO/WHO ICSC 1047 (GHS H314, H311) |
| not claimed | melting point (NASA liquid boundary 1445 K), dissolution enthalpy, flame colour, tint, Cp curve | - |

Booking: `S(-2)` books as a new aqueous-basis ion `HS-` ("hydrosulfide ion", 33.068 g/mol
= 1.008 + 32.06, Cp 0, density placeholder 1.0, source `kerotakis/aqueous-basis-v1` as `SCN-`),
via `BOOKING_OVERRIDES` `("S(-2)", "HS-")`. No protonation split (thiosulfate precedent); the
acid-side H2S(aq) share is booked under `HS-` by element and its proportion is carried by pH.
Dissolving Na2S has no reviewed enthalpy on any route, so a step that dissolves it emits
`not-modeled.aqueous-heat-unpriced` (chemistry and pH stand; heat withheld), as Na2SO3 does.

Safety: `Na2S` -> BaseStrong, ReducingAgent, ToxicSoluble; `HS-` -> ReducingAgent,
ToxicSoluble, and both also carry the `Sulfide` group. The fourth pass (below) closes the
former gap with the `acid-sulfide-h2s` rule.

Thermal: Na2S(cr)/Na2S(L) now enter CEA thermal pools for Na+S charges (the registry names
them). No melting transition or fusion enthalpy was added, so a hot Na2S run may need the
Cu2O-style transition and fusion-ledger work.

Validator tests (not run): `crates/kerotakis-phreeqc/tests/sulfide_family.rs`, see the
expectations above; `derived::tests::sodium_sulfide_books_reduced_sulfur_and_nothing_else_does`;
`kerotakis-safety` `sodium_sulfide_and_its_booking_ion_are_screened`.

## Fourth pass: acid + sulfide rule, SO2 headspace note, H2S booking decision (2026-11-19, untested)

**Safety.** New reactive group `Sulfide` on `Na2S` and `HS-` (hazard label "toxic"), and a
Danger row `acid-sulfide-h2s` (AcidStrong x Sulfide), citing ILO/WHO ICSC 0165 (hydrogen
sulfide: acutely toxic by inhalation, smell fatigue at harmful levels, extremely flammable).
Joins the reviewed strong-acid final-projection set beside `acid-carbonate-co2`, so a solved
vessel whose HCl is booked as `H+` charge does not warn a second time. de/fr in core i18n and
web locales. Tests: `acid_and_sulfide_warns_of_hydrogen_sulfide`,
`sulfide_alone_or_sulfate_with_acid_does_not_raise_h2s`, rule-id contract row `HCl + Na2S`.
The frozen production contract tests were not touched; they enumerate only their three rules.

**SO2 note.** `not-modeled.so2-finite-headspace` (aqueous.rs
`sulfur_dioxide_headspace_boundary`), emitted whenever `problem.gases` holds `SO2(g)`, i.e.
the minteq.v4 finite-headspace coupling. It reuses the HBr temperature clauses (same
constant-enthalpy van't Hoff convention). Unit tests in `sulfur_dioxide_boundary_tests`.

**H2S gas -> dissolved S(-2): NOT implemented.** Evaluated against the NH3 -> N(-3)
precedent and found not clearly correct:

1. NH3 works because one registry key serves both sides: `NH3` is the gas AND the neutral
   aqueous half of the `N(-3)` protonation split (`("N(-3)", [NH3, NH4+])`), so gas and
   dissolved ammonia read back under the same name. `S(-2)` has no split: it books wholly as
   the ion `HS-`. A `hydrogen_sulfide` Dissolves role would turn a neutral gas dose into an
   anion with no counter-cation; its proton would ride as negative `solute_charge`, which
   `kerotakis-safety` reads as dissociated acidity and `strong_reserve` would have to
   reconcile against `hydrogen_sulfide` = "sulfide". Clean booking needs a neutral aqueous
   key (e.g. an `H2S(aq)` basis row) and an `("S(-2)", [H2S(aq), HS-])` split first.
2. A finite headspace never admits `H2S(g)`: `finite_headspace_gases` gates on
   `gas_component_elements`, which for `H2S(g)` returns bare `S`, never the valence-tagged
   `S(-2)` the partition notes. The NH3 path has a dedicated `N(-3)` special case at both
   the supplied-gas site and the empty-component site; H2S would need the same pair plus a
   `gas_component_elements` override (as SO2 has) and evidence that the `redox_isolation`
   rewrite onto `Sredtwo` holds through a GAS_PHASE block, which only an engine run shows.
3. With no H2S clock (first-pass item 1), a dissolving role would let an open-beaker dose
   enter solution but never leave on timed steps, which inverts the physical expectation.

Prerequisites, in order: neutral aqueous H2S key + split; S(-2)-gated finite headspace
admission with readback to `hydrogen_sulfide`; engine tests for sealed partition and
conservation (pattern: `tests/reactive_gas.rs`) on wateq4f and minteq.v4; pitzer refusal.

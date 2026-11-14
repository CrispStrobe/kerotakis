# Cuprite registry and perturbation follow-up — 2026-11-14

The hosted workspace on `6e93cc8f` preserves the first failures caused by the new
Cu2O registry identity. These repairs do not alter energy or localization logic.
They are not a new chemistry qualification or an acceptance claim.

## Causal perturbation

The two new ablation departures (`th-062`, `mat-004`) remove the final explicit O2
addition from dry, open heating/ignition scripts. That does not remove their oxygen
source: the open thermal route still admits room air. An unchanged response to that
operation is not evidence that PP or Cu was ignored.

The generic ablation selector therefore removes the non-atmospheric specimen in
such scripts. Wet media, finite/swept/regulated boundaries, and heating in another
vessel retain the previous oxygen target. Dose perturbations retain their previous
last-reagent target. No original curiosity script, scientific forecast, perturbation
readout, tolerance, recorded departure list or minimum case count is changed.
Selector controls distinguish open/dry specimen ablation from these negative cases.
The original failed oxygen-ablation readbacks remain first-failure evidence; the
complete generated perturbation gate must run on the successor.

## Registry coverage and scientific boundaries

The Python chromophore generator adds Cu2O to `missing_absorptivity`: its red solid
appearance supplies no dissolved extinction coefficient. It raises that count from
38 to 39 without inventing a spectrum. The hosted element summary adds precisely
one Cu example and one O example (Cu 12→13, O 225→226); its other counts are unchanged.

The hosted dissolution readback reports the new identity as
`reference:wateq4f:Cuprite`, `reference:minteq.v4:Cuprite`, and refused on pitzer,
whose routed reaction species is absent. The maintained Markdown adds that actual
row and updates priced totals to 49/102, 51/102, 23/102. The native-generated JSON
fixtures must still be produced and reviewed on the hosted successor; these are
registry coverage consequences, not an expanded thermodynamic domain claim.

Cu2O receives an explicit compatibility-matrix row and coverage-key entry. The insoluble oxide
is not assigned to `ToxicSoluble`, nor treated as an active metal.
An empty compatibility row does not establish harmlessness or complete handling
coverage. [ILO/WHO ICSC 0421](https://inchem.org/documents/icsc/icsc/eics0421.htm)
identifies dust/ingestion, serious eye, aquatic and aluminium-powder incompatibility
hazards that this matrix does not encode. The source comment records that limit;
no unsourced hazard family or false complete GHS classification is introduced.

## Hosted fixture generation still required

Run only the existing narrow generators on the candidate, then preserve their diffs
for review and rerun the same tests with blessing absent:

```sh
KEROTAKIS_BLESS_GOLDEN=1 cargo test --locked -p kerotakis-core --test registry_snapshot registry_matches_the_golden_snapshot
KEROTAKIS_BLESS_ALIAS_COVERAGE=1 cargo test --locked -p kerotakis-core --test input_alias_coverage generated_alias_table_matches_checked_in_fixture_and_doc
KERO_BLESS_COVERAGE=1 cargo test --locked -p kerotakis-phreeqc --test crystallisation_route_coverage crystallisation_route_coverage_is_current
KERO_BLESS_COVERAGE=1 cargo test --locked -p kerotakis-phreeqc --test dissolution_enthalpy_coverage
```

Expected generated paths:

- `crates/kerotakis-core/tests/golden/registry.json`: exactly the sourced Cu2O row,
  alongside any independently justified current differences; no removed identities.
- Input-alias JSON fixture and Markdown: Cu2O's names, including the finalized locale
  inventory. Generation must follow localization repairs rather than guess aliases.
- Crystallisation-route JSON fixture and its generated Markdown table: one Cu2O
  row per routed database, preserving priced/solubility-only/refused distinctions.
- LLNL dissolution-reference JSON: the actual extracted Cuprite row and any required
  species dependencies, retaining exact vendored reaction and enthalpy authority.
- Dissolution-enthalpy JSON fixture: the three source-bound routes above. Its existing
  generator prints the Markdown table; the corresponding row/count update is prepared.

No Rust build, native test or fixture blessing ran locally. The Python chromophore
generator ran directly from repository source data; formatting was applied. Exact
hosted unblessed controls and broader workspace/fleet acceptance remain required.

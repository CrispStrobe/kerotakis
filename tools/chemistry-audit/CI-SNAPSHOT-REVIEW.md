# CI snapshot review — merge build 38d44ac

Read-only artifact/source comparison on 2026-09-07. Compared the Linux and
macOS `38d44ac53342c215f56d9d12db4135ee3a51224d` artifacts in
`/tmp/kero-audit-ci-snapshots-Kke0xc` against branch goldens at HEAD
`2c260cb7a3fd022a4cc5d6e7f9c16296fdf26721`. No build or golden update.

## Platform agreement

Both platforms produced byte-identical files:

| Artifact | SHA-256 on both platforms |
| --- | --- |
| `codex-export.actual.json` | `fe866c21cfea22f8a09df3981ff86ca2e0c86bd0aed792f4d7a8c4069a4dea3a` |
| `registry.actual.json` | `4e729ccda14a608956849962fb0baa57cc03cc4ce6f40d7f39c969a46b8946fc` |

This is deterministic source/schema drift, not a Linux/macOS discrepancy.

## Registry

All 176 identities and their order are unchanged. The **only** changed field
is `heat_capacity_polys`, newly serialized on every record. Deleting this
field from the actual artifact gives exact JSON equality with the branch
golden, including all existing values, provenance and aqueous-basis records.

There are 35 species with nonempty curves: `water`, `NaCl`, `S`, `SO2`,
`Ca(OH)2`, `CuO`, `NaOH`, `Cl2`, `Na2CO3`, `CO2`, `KCl`, `CaCO3`, `CaO`,
`Mg`, `Cu`, `Zn`, `Ag`, `Fe`, `MgO`, `C`, `O2`, `N2`, `H2`, `Pb`, `Al`,
`Fe(OH)3`, `Fe(OH)2`, `Mg(OH)2`, `Fe2O3`, `SiO2`, `Na2SO4`, `methane`,
`propane`, `butane`, `CO`. Water has solid/liquid/gas curves; each other
listed species has one phase record. The other 141 records have `[]`.
No new heat-capacity curve is asserted for KSCN or SCN-.

Cause: upstream `523b93ad` (*data: heat capacity stops being one number*)
adds the `SpeciesData.heat_capacity_polys` field, generation and NASA data.
Its serde annotation skips deserialization but **not serialization**.
Every nonempty curve in the CI artifact exactly matches the corresponding
curve in that upstream commit's registry golden.

## Codex

Counts/order unchanged: 189 concepts, 28 models, 110 reactions. Concepts
are identical. Removing all recursively nested `_de` fields makes the
entire actual export exactly equal to the branch export.

All 28 models gain `name_de`, `power_de`, `explains_de` and `fails_at_de`.
Additionally, models `enthalpie` and `gibbs-energie` change only
`registers.lv3_de` within their existing register text. All actual model
objects exactly match upstream `40422612` (*feat(i18n): the whole catalogue
speaks German, and the gate can tell*).

Only four reactions differ:

| Reaction ID | Changed fields |
| --- | --- |
| `dilution-by-ten` | `summary_de`: German decimal commas replace decimal points |
| `sugar-syrup-by-density` | Added `summary_de`, `registers.lv1_de/lv2_de/lv3_de`, and German prediction question/options/misconception plus two diagnosis next/reveals pairs |
| `warm-water-climbs-less` | Same German field groups, with two diagnosis pairs |
| `catalyst-area-and-stirring-change-the-rate` | Same German field groups, with three diagnosis pairs |

All four actual reaction objects exactly match upstream `40422612`.
The remaining 106 reaction objects match the audit branch, so this artifact
does not erase the audit's reaction corrections or additions.

## Integration consequence

The differences are accounted for by identifiable upstream changes. Sync
their production source first, preserving audit content, then update only
the corresponding expected snapshots from the integrated revision. Do not
copy the upstream codex wholesale: some other reaction texts intentionally
differ because of the audit's corrections. This review establishes snapshot
causality and platform agreement, not independent validation of NASA curves
or permission to skip the integrated test gates.

# Audit integration requests — reserved files not edited

## 2026-09-07 continuation

Local audit catalog now has 113 entries (the previous 110 plus three original
lessons), 28 models and 189 defined vocabulary concepts. Count used concepts
separately when reconciling README; do not substitute the defined count.
The three additions are `three-components-one-cut`,
`equal-charge-different-clocks`, and `grouping-does-not-change-water-heat`.
Native selected-catalog replay: 13 entries / 37 checks pass. Focused app tests:
199/199 pass; German slug labels and generated export include all three.
Core snapshot verification and combined integration remain pending at this
checkpoint. The recommendations below are historical, not current authoring state.

Fifth fleet passes 36 runs / 119 checks on recorded pre-diagnostic-fix CLI.
Generic no-op diagnostic repair and regressions are being validated; sixth
fleet 24 inputs / 66 checks is frozen and requires the rebuilt CLI. No source
research inventories or copied source content may be published in shared docs.

The parallel integrator owns main sync, shared planning reconciliation and live
catalog/discovery work. The audit branch has not edited those reserved files
following coordination. These are concrete remaining integration needs from
completed CI run 34096877766 (merge source
`38d44ac53342c215f56d9d12db4135ee3a51224d`):

- The combined catalog contains 110 entries, 28 models and 167 concepts at that
  tested source. `README.md` still claims 108 entries/166 concepts. Its existing
  `readme_counts_match_the_codex` test correctly rejects this. Please reconcile
  the prose from the actual combined catalog when integrating; newer work may
  change the totals again.
- Incoming `web/app/src/lib/codexProse.test.ts` expects 108 entries, but the
  audit adds two entries. That file is absent from the audit's current local
  base and present in CI's combined source. Please use the current combined
  corpus count while preserving the non-vacuous translation test.
- Computed codex/registry snapshot differences are under review in the audit.
  They must be reconciled by field against the exact combined source, not by
  copying unrelated newer catalog/discovery changes into this branch.

Promising third-fleet catalog candidates, not yet added to the app:

1. Three volatile solvents in one computed still cut: conservation, methanol
   enrichment and amount/order controls; explicitly ideal, latent-only where
   applicable, not a prediction of azeotropes or apparatus efficiency.
2. Equal total charge delivered at different currents or in separate steps:
   Faraday product ratios plus a full gas/material ledger.
3. An unsupported still feed must leave the vessel unchanged: best preserved
   as a regression test and a clear capability-boundary example, not a claim
   that ammonia distillation itself is modeled.

The audit has already preserved endpoint refinement and reverse equilibrium
as app lessons, with their translations, guidance and core snapshots. Further
app catalog changes need ownership coordination and clean validation first.

# Reaction repair checkpoint — 2026-10-16

All 200 supplied protocols have CLI/API baseline results and a maintained
[per-record repair queue](../audits/reaction-dataset-200-20261015/repair-progress.json).
The original screen remains 13 principal observations, 11 partial, 16 target
not demonstrated, 20 model limits, 32 inconclusive and 108 identity refusals.
These are qualitative screening categories, not calibrated scientific passes.
All 200 are not repaired. Original forecasts and first failures remain immutable.

| Repair | Actual integration and acceptance | Remaining scope |
| --- | --- | --- |
| Incomplete ionic-input reporting and electrolysis bookkeeping | PR796, `21d2cca8`; native, browser and full fleets passed. | Unsupported bare coordinates and the original pure-water electrode domain remain explicit. |
| Silver ammine routing | [PR798 receipt](../audits/reaction-dataset-200-20261015/pr798-integration.json), `5b7cc5ee`; all 58 applicable checks passed, one skip. | Canonical free/complex-bound ammonia ownership is still a material gap. |
| Finite-gas serialization precision | [PR801 receipt](../audits/reaction-dataset-200-20261015/pr801-integration.json), `6fffb455`; strict gas/filtration controls and all 58 checks passed, one skip. | No changed constants, forecasts, conservation tolerances or expanded chemistry claim. |
| Shared fresh aqueous continuation and unknown-heat ownership | [PR799 receipt](../audits/reaction-dataset-200-20261015/pr799-integration.json), `79f9e5f8`; native, actual browser, Wasm, bridge and all own fleets passed, 58 checks/one skip. | No new reaction endpoint was installed by this prerequisite. |
| Historical-lot false settling | [PR803 receipt](../audits/reaction-dataset-200-20261015/pr803-integration.json), `92e14021`; core/native/browser and all own fleets passed, 58 checks/one skip. | Historical grain-size allocation and metadata for new precipitates remain separate. |

The actual parents/tree and preservation of each accepted patch on preceding
integrated runtime repairs are recorded. Reviewed combined trees are not
misrepresented as earlier CI merge trees. Earlier documentation PR802's
[integration receipt](../audits/reaction-dataset-200-20261015/pr802-integration.json)
remains preserved. No merge by itself establishes deployment or calibration.

The separate [rebuilt-engine REST controls](../audits/reaction-dataset-200-20261015/solid-settling-rest-acceptance-20261016/completion.json)
passed four of four: absent CuSO4/NaCl do not settle, actual residual AgCl still
settles, and conducting-stock electrolysis preserves its 2:1 gas/water balance.
Source `4abb4d79`, binary `2d83c77b`, raw requests/responses and the frozen
[engine binding](../audits/reaction-dataset-200-20261015/solid-settling-rest-acceptance-20261016/execution-binding.json)
remain separate. Cargo.lock identity is producer-reported because its bytes
were absent from the artifact. This isolated validation branch must never merge.

The [paired rank-1 control](../audits/reaction-dataset-200-20261015/case001-repair-20261016/assessment.json)
verifies supported HCl/NaOH neutralisation and honest bare-input refusal.
[Rank 7's complete conducting stock](../audits/reaction-dataset-200-20261015/case007-repair-20261016/assessment.json)
verifies electrolysis products; [rank 14's complete stock](../audits/reaction-dataset-200-20261015/case014-repair-20261016/assessment.json)
first exposed the now-repaired phantom settling. Preserve that first defect
assessment alongside the later repaired-engine results. These separate protocols
do not rewrite the original experiments.

## Next executable lanes

1. **Finish the bounded rank-119 acidic endpoint.** Source implementation and
   independent review are active from integrated continuation main. Follow the
   [full lane contract](acid-permanganate-peroxide-lane.md). Require fresh complete
   acidity plus actual aqueous H+ ownership, phase-specific debits, sulfate/K
   medium, open/fixed-sealed oxygen ownership, rollback and persistent unknown
   heat. Do not turn partial unsupported aqueous answers into a new solver error.
   Freeze positive, limiting-reactant, wrong-phase, proxy-only, unsupported-medium,
   native/cache/browser and reentry controls; execute focused/full hosted checks,
   then normal exact-head protected auto-merge. No endpoint runtime pass is claimed.
2. **Finish source-bound REST delivery.** PR795 is refreshed as `74a030ec` onto
   accepted main `79f9e5f8`; fresh CI/fleets and preview run 38862025694 are pending.
   The [actual browser receipt](../audits/reaction-dataset-200-20261015/rest-browser-acceptance-20261016/review.json)
   accepts fifteen unchanged-client privacy/network controls on production's old
   engine. Actual Pages URL returns 404; controlled Pages-origin CORS passed
   separately. After accepted source integration, verify actual Pages publication
   and exact-source Vercel delivery. Preserve deployment protection and personal
   authentication. Production remains API `9bee95d9` / engine `d791d306`.
3. **Resolve identities individually.** The [first fifteen token reviews](../audits/reaction-dataset-200-20261015/identity-first15-review-20261016.json)
   identify only two proven aliases: `CuSO4_anhydrous` to `CuSO4` and explicitly
   singly charged `acetate` to `CH3COO-`. Add paired parse/finite-stock controls.
   The other thirteen require identities, physical domains or schematic definition;
   never substitute an ion, hydrate or isomer to silence a refusal.
4. **Continue remaining protocols and model gaps in order.** Use the
   [ordered repair contracts](../audits/reaction-dataset-200-20261015/CHEMISTRY-RESULTS.md#ordered-repairs)
   and per-rank queue. Complete conditions before assigning model fault. Prioritize
   canonical ligand ownership, complete-stock conservation/observable comparisons,
   finite KNO3, missing precipitate domains and explicit activation boundaries.
   Preserve unsupported chemistry as a coverage limit, not a physical pass.

All validation must retain source/binary/lock associations, original raw failures
and separately frozen follow-ups. Check load, memory and disk before local heavy
work or agents; use bounded hosted validation when local gates are closed.
Private host details stay in the private operating handoff.

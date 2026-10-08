# Integration checkpoint — 2026-10-08 09:52 UTC

Main is `0fafbf5e3c71841f83832dd61a0c7f4561204e63`. Amount #756, stock #757, thermo #758, native #760, safety #762 and binary still diagnostics #759 are merged. Each implementation passed its required matching-head gates; the [merge receipt](merged-port-gates.json) binds those checks and merge commits. The dated checkpoint and task-lane docs in #761 are also merged. Their pending-state rows reflect the earlier snapshot; this checkpoint updates them.

These merges do not replace the full audit on the combined source. The last full-audit accepted source remains `8cdab1f6ecd0b1132c4008014a5a0c5476429d17`.

## Pending implementation reviews

| Review | Current head | Remaining work |
| --- | --- | --- |
| [#763 refusal/preflight atomicity](https://github.com/CrispStrobe/kerotakis/pull/763) | `d30f0c4cd7ba24f06004f062ebdb26bf179ec212` | Refreshed against main; require fresh matching-head gates. Preserve the 50 targeted controls, original baseline failures and disclosed legacy Drain fixture adaptations. |
| [#764 additional-solvent still diagnostics](https://github.com/CrispStrobe/kerotakis/pull/764) | `ab644c6c818ce3bd4be2ec50c4f9fde649d0e244` | Binary prerequisite merged; retargeted to main and refreshed. Require fresh gates, preserving original kernel/operator/portion contracts and adaptations. Earlier-head passes do not validate this head. |
| [#765 withdrawal trace preservation](https://github.com/CrispStrobe/kerotakis/pull/765) | `3d2c6b3a744cf3993ddcc036b135371fa8d6e1d2` | Refreshed against main. Consume the identical-fixture baseline/repair run, distinguish compilation from behavioral results, then require fresh gates. |

## Execution lanes

1. **Consume the fifth independent fifty.** [Run 37751230970](https://github.com/CrispStrobe/kerotakis/actions/runs/37751230970) built the CLI successfully; experiment and profiling jobs remain queued. Forecast, source, binary and generated-lock bindings are preserved in [the campaign checkpoint](https://github.com/CrispStrobe/kerotakis/blob/81240dca/audits/fifth-independent-50-20261008/RUNS.md). The source is frozen at `7bc43e08`, before the later safety/binary merges. Preserve raw failures and original bounds; classify author errors, model boundaries, engine errors and missing observability separately. Revalidate native profiling samples with the archived-output reviewer before accepting timing summaries. No chemistry result or speedup is claimed yet.
2. **Validate the bounded trace repair.** [Run 37756438042](https://github.com/CrispStrobe/kerotakis/actions/runs/37756438042) has queued baseline and repair jobs for ten source-informed controls. Preserve both artifacts and each actually reached outcome. The repair pin `b6306db9` predates the later main refresh; it cannot replace required gates on #765's current head. [The repair checkpoint](https://github.com/CrispStrobe/kerotakis/blob/645abc98/audits/withdrawal-trace-20261008/STATUS.md) scopes the remaining mechanical and chemical cutoff review. These controls are separate from the independently forecast fifty.
3. **Merge validated implementations, then audit the combined source.** Inspect #763–765 at their exact heads, resolve actual failures without fitting expectations, and merge after required gates pass. Then run the full audit and preserved third/fourth replays against one exact executable. Bind source, binary, generated lock, toolchain and submodules; retain the four legacy exclusions and original qualifications. Only accepted, output-equivalent workloads can support a measured optimization claim.

Branch-only diagnostic workflows must never merge over main's audit fleet workflow. The [22-lane roadmap](../../docs/chemistry-audit-next-lanes.md) remains open beyond these bounded repairs, especially authoritative vessel ownership, chemical reconstruction and still donor/receiver/energy closure.

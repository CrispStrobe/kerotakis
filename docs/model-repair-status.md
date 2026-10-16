# Model repair status — 2026-10-16

Provider: togetherai via https://api.together.xyz/v1 (launcher --provider togetherai)
Model: meta-models/Muse-Spark-Open-816B-A42B-promo
Scope: Independent model-repair lane — solver routing, chemical/phase mechanisms, canonical inventory ownership,
  material/charge/gas/energy accounting, thermodynamic/activity data, rates and observable models.
  Protocol agent owns corrected .lab scripts and input aliases; no overlapping edits.

## Claimed issue: MODEL-001 — acid permanganate endpoint PR805

- Branch: fix/acid-permanganate-endpoint-20261016
- Head: d66b08119c75b0b30e22550c5951f85c7801fe93 (previous 3f84e59f38098c4e4e995c885659544c54eb47c2, merge candidate e7e0c4e05a0907e4c3c5b4136160c7d094605bca)
- Previous CI: https://github.com/CrispStrobe/kerotakis/actions/runs/38865460559 (failed, preserved)
- Current CI: https://github.com/CrispStrobe/kerotakis/actions/runs/38870449486 (queued, includes locale fix)
- Fleet: https://github.com/CrispStrobe/kerotakis/actions/runs/38870449501 (queued)
- Status: Open, not merged; awaiting exact-head acceptance

## Current actual failures (CI 38865460559) — preserved before source edits

- Native Ubuntu https://github.com/CrispStrobe/kerotakis/actions/runs/38865460559/job/116857537390 : FAILED — composed_locale::unknown_composers_are_declared
  Panic: these files compose sentences but are not in COMPOSERS, so nothing checks that their keys are translated: ["acid_endpoint.rs"]
  Log hash: d5e09750e33fc56f9fa38676b4dc15bb844b9c42d72963f3c31ec35e549dcdae
- Native macOS https://github.com/CrispStrobe/kerotakis/actions/runs/38865460559/job/116857537436 : FAILED — same unknown_composers (hash 868505c5b14c6b5e5a0d5fb8a41ed8fc1a9943027526502a9781c192e611a255)
- Preflight https://github.com/CrispStrobe/kerotakis/actions/runs/38865460559/job/116857537354 : FAILED — same unknown_composers (hash 122225f1c9c007544a12bd49f8100eaa077a8a3e50b2627f7d9f4ebb01332366)
- Viewer spike BRD-080 https://github.com/CrispStrobe/kerotakis/actions/runs/38865460559/job/116857537402 : FAILED — 3dmol/water: expected semantic rows and exactly one canvas, got 3/0 (hash c6f57451f9031401e3028eb64d3cc1fa6cb5fbc787dacaa36875e9e4af31af1c)

All four first-failure raw logs preserved with hashes; original failure evidence retained before any retry.
New head d66b0811 pushed with one-line COMPOSERS admission; previous CI cancelled, new CI queued.

Diagnosis:
- Native/preflight failures are production localization acceptance coverage defects, NOT runtime chemistry defects.
  The new file crates/kerotakis-core/src/acid_endpoint.rs composes two Phrase::bare sentences
  (not-modeled.acid-permanganate-endpoint, not-modeled.acid-permanganate-domain) with real
  crates/kerotakis-core/i18n/de.toml and fr.toml translations already added, but was not admitted to
  COMPOSERS in crates/kerotakis-core/tests/composed_locale.rs. The scanner test correctly rejects
  invisible composers — this is not a flaky harness and tolerances are unchanged.
- Chemistry runtime: All 8 acid_permanganate_endpoint tests passed on both platforms before the
  composed_locale gate (610 passed before the one failure). The pre-existing F12 replay on
  source a03113a2 remains valid; no new runtime chemistry failure is shown in these logs. The 10
  strict endpoint fixtures stay intact.
- Viewer: separate decision-spike failure; concrete evidence shows same spike passed on
  2026-08-31 commit 5d123880 (BRD-080 success), but fails on 2026-10-16 runner image
  20261012.334 with 3dmol/water 3/0 canvases. Likely environment WebGL/SwiftShader
  regression, not application change from acid branch (spike code unchanged, build
  identical). No canvas assertion weakened; awaiting new CI 38870449486 viewer job
  116873033113 to confirm before any flag change. Preserved first log c6f57451.

## Next fix (pending push)

- Add proper composer admission for acid_endpoint.rs to COMPOSERS with include_str
  and ensure all phrase keys have real locale coverage (no scanner bypass).
- Also verify tools/engine-locale-lint.py composer list and existing real translations.
- Retain existing de/fr translations verbatim; no weakened tolerances.
- Verify native/preflight and viewer before claiming readiness; preserve first-failure logs.

## Resource gate

- Source/network-only validation at this time; no duplicate workflow dispatch or active-check cancellation.

## Related

- Root contract/evidence committed a3835db9 (read-only) on audit/systematic-chemistry-20261002.
- Production https://github.com/CrispStrobe/kerotakis/actions/runs/38868092504 in_progress — not duplicated.
- MODEL-005 intake available; will review after PR805 stabilizes.

# EngineHost protocol v1 (GUI-001)

The contract between any Kerotakis UI and any engine transport. Two hosts
implement it:

- **WorkerHost** — web: `kerotakis-wasm` (+ IPhreeQC) in one module Web
  Worker (OPT-7 / GUI-004);
- **TauriHost** — desktop/mobile: native `kerotakis-core` on a background
  thread behind `tauri::command` (GUI-030).

The UI must not be able to tell them apart. One conformance suite runs
against both (GUI-001 acceptance).

This document canonizes what already exists — the wasm `Lab` API, the
WEB-002 `WorkerCommand`/`WorkerResponse` enums in
`crates/kerotakis-wasm/src/worker.rs`, and the serde shapes of `Operator`
and `Event` — and names the gaps with their GUI task ids. Rust remains the
source of truth: the protocol types are the serde serializations of core
types, never hand-maintained TypeScript mirrors (a generated `.d.ts` is
fine; a hand-written one is drift waiting to happen).

## Versioning and evolution

- The `hello` response carries `protocol: 1` plus engine build info. The UI
  refuses a major version it does not know.
- Evolution is additive within a major version, following the discipline the
  `Event` enum already practices (`Consumed.remaining`): new fields are
  `#[serde(default)]` + `skip_serializing_if`, new event/command variants may
  appear, and every consumer must ignore unknown fields and unknown variants
  rather than erroring.
- Field removal or meaning change = major bump. Expect never to need one.

## Transport envelope

All calls are asynchronous request/response with correlation ids; some calls
additionally stream interim messages under the same id.

```json
→ { "id": 7, "cmd": "step", "operator_json": "…" }
← { "id": 7, "type": "progress", "fraction": 0.4, "message": "equilibrating v1" }
← { "id": 7, "type": "done", "result_json": "…" }
```

- `cmd` / `type` are the existing WEB-002 tag names (`snake_case`).
- WorkerHost: `postMessage` both ways. TauriHost: `invoke` for
  request/response, a Tauri event channel for `progress`. The `id` is
  assigned by the UI and echoed verbatim.
- Exactly one terminal message per id: `done`, `error`, or `cancelled`.
- `cancel` addresses a specific in-flight `id`
  (today's `Cancel` has no target — extend it: `{ "cmd": "cancel", "target": 7 }`).
- Errors: `{ "type": "error", "message": … }` gains a `kind` field —
  `parse` (bad command), `refused` (model says no — carries the rendered
  explanation; this is a *result*, the UI shows it as chemistry, not as a
  failure), `engine` (solver error), `internal`. Until `kind` lands, all
  errors are `internal`.

## Commands

Existing = serves today's wasm/worker surface. Gap = named task.

| `cmd` | Status | Request → `result_json` |
|---|---|---|
| `hello` | **gap (GUI-001)** | `{}` → `{ protocol, engine_version, git_rev, can_solve, packs: [ModelPackManifest], registers: ["lv1","lv2","lv3"] }`. Must be answerable before any pack loads. |
| `step` | existing | `{ operator_json }` → `{ events, rendered, bench }` — `events` is the serde `Vec<Event>`, `rendered` the prose at the current register. GUI-002 adds `scene` (below) to this response so one round trip repaints the bench. |
| `run_script` | existing | `{ script }` → `{ steps: [{operator, events, rendered}], bench }`; grows `scene` per step (GUI-002). |
| `parse` | **gap (GUI-005)** | `{ line }` → `{ ok, operator?, error?, span? }`. Validate-only, never executes. Powers live input validation, drag-legality preview, and command-bar completion diagnostics. |
| `set_register` | existing | `{ level }` → `{}`. Presentation only; never re-solves. |
| `state` | existing | `{}` → `{ vessels, steps }` (full serde `Vessel`s — the lv3/machine contract). |
| `scene` | **gap (GUI-003)** | `{}` → Scene JSON v1 (below). The render model; everything a bench canvas needs, nothing it must derive. |
| `species` | existing (`Lab::species`, not yet a WorkerCommand) | `{}` → shelf list: key, name, formula, phase, appearance, provenance. |
| `look` / `inspect` / `particles` | existing (`Lab` methods, not yet WorkerCommands) | `{ vessel }` → observation / `{rendered, vessel}` / `{census, rendered}`. |
| `reset` | existing | `{}` → `{}`. Bench only; session (register, packs, cache) survives. |
| `load_cache` / `load_pack` | existing | per WEB-002/WEB-003; pack manifests are signed per LIC-009. |
| `cancel` | existing (needs `target`) | terminal `cancelled` for the target id. |

Promoting the `Lab`-only methods to `WorkerCommand` variants is part of
GUI-004 (the UI talks only to the host, never to `Lab` directly).

## Events

`Event` serializes as `{ "event": "<snake_case_tag>", …fields }` with typed
quantities (`VesselId`, `SpeciesId`, `Moles`, `Kelvin`). Rules:

- Tags and field names are API. Renaming one is a protocol break.
- `rendered` prose always accompanies `events` — clients that cannot render
  a variant fall back to prose. This is also the localization seam
  (GUI-002 decides locale-pack shape): prose moves per-locale, tags do not.
- The `Confidence` vocabulary (`computed`/`modeled`/`template_match`/
  `curated`/`unknown`) rides on events/results wherever a number is claimed
  and gets one fixed visual encoding in every UI (GUI-023).

## Scene JSON v1 (GUI-003 — to implement)

A versioned, per-vessel *render model*, derived engine-side from state +
`appearance`/`spectrum` so native and web paint identically and golden tests
can pin frames. Sketch (authoritative shape lands with GUI-003 as serde
types + golden files over replayed lessons):

```json
{
  "scene": 1,
  "vessels": [{
    "id": 1, "kind": "beaker",
    "boundary": "open|sealed|regulated|swept",
    "liquid": { "volume_ml": 200.0, "srgb": [0.83,0.41,0.72],
                 "path_length_basis_cm": 4.0, "turbidity": 0.0 },
    "solids": [{ "species": "AgCl", "moles": 0.0099, "texture": "fine",
                  "srgb": [0.96,0.96,0.94] }],
    "headspace": { "volume_ml": 300.0, "pressure_kpa": 101.3 },
    "temperature_k": 298.15,
    "effects": [{ "fx": "bubbling", "species": "CO2", "rate": 0.3 }],
    "apparatus": [{ "kind": "hotplate", "power_w": 120.0 }],
    "badges": [{ "key": "ph", "value": 7.10, "confidence": "computed" }]
  }]
}
```

Effects are event-derived and never fire without a computed event behind
them. Colors are the engine's computed sRGB (Beer–Lambert over the drawn
path length), paired with the engine's color *names* for accessibility.

## Conformance suite (GUI-001 acceptance)

`validation/protocol/` holds a corpus of `.lab` scripts plus, per script,
the expected sequence of `(cmd, result)` shapes — structural goldens
(field presence/types/tags), with numerics tolerance-checked the way lesson
replay already does. A host passes by replaying the corpus byte-identical in
structure. CI runs it against WorkerHost (wasm) and, once it exists,
TauriHost. Determinism of the engine makes this flake-free.

## Non-goals

- No streaming partial chemistry (a step commits atomically or not at all —
  the roadmap's transactional rule); `progress` is cosmetic.
- No UI state in the protocol: selection, pan/zoom, open panels are the
  client's business. The engine is the only chemistry state holder.

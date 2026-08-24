# EngineHost protocol v1 (GUI-001)

The contract between any Kerotakis UI and any engine transport. Two hosts
implement it:

- **WorkerHost** — web: `kerotakis-wasm` (+ IPhreeQC) in one module Web
  Worker (OPT-11 / GUI-004);
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
| `hello` | done except `packs` | `{}` → `{ protocol, can_solve, engine_loaded, load_failure, aqueous_note, engine_version, git_rev, registers }`. `git_rev` is stamped by the build (`KEROTAKIS_GIT_REV`; null in unstamped dev builds). Still to grow: `packs: [ModelPackManifest]` when pack loading lands. Must be answerable before any pack loads. |
| `step` | done | `{ operator_json }` → `{ events, rendered, scene, bench }` — `events` is the serde `Vec<Event>`, `rendered` the prose at the current register, `scene` the render model (one round trip repaints the bench). |
| `run_script` | done | `{ script }` → `{ steps: [{operator, events, rendered}], scene, bench }`. |
| `parse` | done (GUI-005, 9a9c744) | `{ line }` → `{ ok, operator?, error? }`. Validate-only, never executes. Powers the command bar's live validation; `span` remains a candidate additive field. |
| `relations` | done (GUI-027) | `{}` → `[{ name, equation, args }]` — the CAP-5 named-relations catalogue. `args` is the CLI arg-spec string (`k=<hint>`, brackets for optional); clients build forms from it rather than hard-coding fields. |
| `calc` | done (GUI-027) | `{ name, args: ["k=v", …] }` → `{ ok, value, unit, provenance, lv1, lv2, lv3 }` or `{ ok: false, error }`. One evaluation of a named relation; the result explains itself at every register and names its source. Same argument grammar as `kero calc`. |
| `set_register` | existing | `{ level }` → `{}`. Presentation only; never re-solves. |
| `state` | existing | `{}` → `{ vessels, steps }` (full serde `Vessel`s — the lv3/machine contract). |
| `scene` | done (GUI-003) | `{}` → Scene JSON v1 (below). The render model; everything a bench canvas needs, nothing it must derive. |
| `species` | existing | `{}` → shelf list: key, name, formula, phase, appearance, provenance — plus the visual fields (additive, 2026-08-24): `srgb` (reflective colour), `solution_srgb` (computed 0.1 M / 1 cm transmitted tint), `flame` (characteristic flame-colour word). Hazard classes join when the safety-matrix export lands. |
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

## Scene JSON v1 (GUI-003 — implemented in `kerotakis-core/src/scene.rs`)

A versioned, per-vessel *render model*, derived engine-side from state +
`appearance`/`spectrum` so native and web paint identically and golden tests
can pin frames. The serde types in `scene.rs` are authoritative; the shape
(pinned by `the_scene_shape_is_pinned`):

```json
{
  "scene": 1,
  "vessels": [{
    "id": 0, "label": "beaker",
    "boundary": "open|sealed|pressure_controlled|swept",
    "liquid": { "volume_l": 0.2, "srgb": [212,105,183], "colour_word": "pink",
                 "cloudiness": 0.0, "path_length_cm": 4.0 },
    "solids": [{ "species": "AgCl", "name": "silver chloride",
                  "moles": 0.0099, "srgb": [245,245,240],
                  "colour_word": "white", "metallic": false }],
    "bubbling": false,
    "temperature_k": 298.15, "pressure_pa": 101325.0, "elapsed_s": 0.0,
    "words": "The liquid is colourless and cloudy, …",
    "badges": [{ "key": "ph", "value": 7.10, "confidence": "computed" }]
  }]
}
```

The `boundary` tag and its fields are the existing `Headspace` serde enum,
flattened. `words` is the lv1 observation sentence — the accessibility text
for the drawn vessel. Colors are the engine's computed sRGB (Beer–Lambert
over `path_length_cm`), always paired with the color *word*; `metallic`
separates a plated coating from a suspending precipitate (texture, and
turbidity physics). Not yet in v1, arriving additively with their state:
`effects` (event-derived — an effect never fires without a computed event
behind it) and `apparatus`. `step`/`run_script` responses now carry `scene`,
and `Lab::scene()` serves it standalone.

## The chart contract (CAP-3; authoritative in `kerotakis-core/src/chart.rs`)

One JSON contract, every renderer consumes it — the CLI's `chart_svg`, and
the web app (`web/app/src/lib/components/Chart.svelte`, consumer types in
`chart.ts`). The serde shape:

```json
{
  "title": "titration of 25 mL 0.1 M HCl with 0.1 M NaOH",
  "x": { "label": "volume added", "unit": "mL" },
  "y": { "label": "pH" },
  "series": [
    { "kind": "line",    "name": "pH", "points": [[0.0, 1.0], [25.0, 7.0]] },
    { "kind": "scatter", "name": "samples", "points": [[5.0, 1.2]] },
    { "kind": "band",    "name": "±σ",
      "lower": [[0.0, 0.9]], "upper": [[0.0, 1.1]] }
  ],
  "provenance": "PHREEQC (IPhreeqc) · wateq4f.dat"
}
```

- `provenance` is required: a chart without it is a picture, not a result.
- `band` is CAP-8's uncertainty envelope (two polylines sharing x values).
- Transport into a UI session: a step object may carry
  `charts: [Chart]`; the web feed renders them inline (hook live today).
- Renderer duties: nice 1/2/5 ticks, responsive SVG, the same data as a
  table for screen readers, SVG export. Numbers arrive in data units —
  the renderer never converts; the emitter labels.
- Proposed additive extensions (not yet in the Rust contract; do not emit
  until they land there): a per-series `confidence` field rendered via
  GUI-023's stroke encoding, and x-axis `markers` (e.g. "equivalence").

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

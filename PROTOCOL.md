# EngineHost protocol v1 (GUI-001)

The contract between any Kerotakis UI and any engine transport. Two hosts
implement it:

- **WorkerHost** — web: `kerotakis-wasm` (+ IPhreeQC) in one module Web
  Worker (OPT-11 / GUI-004);
- **TauriHost** — desktop/mobile: native `kerotakis-core` on a background
  thread behind `tauri::command` (GUI-030).

The UI must not be able to tell them apart. Conformance is checked per
transport: the wasm host by `tools/test-protocol-conformance.mjs` (CI's
wasm job), the shell by `cargo test` in `web/app/src-tauri` — its
`protocol_conformance` module exercises the same `dispatch` the GUI
reaches through `engine_request`, no webview needed (GUI-001
acceptance). A command answering differently across the two is a
protocol bug even when both GUIs happen to work.

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
| `hello` | done | `{}` → `{ protocol, can_solve, engine_loaded, load_failure, aqueous_note, engine_version, git_rev, registers }`. `git_rev` is stamped by the build (`KEROTAKIS_GIT_REV`; null in unstamped dev builds). `packs` carries the WEB-003 inventory (kerotakis_core::packs_manifest; empty content_hash = built in, not yet independently deliverable — the honest pre-pipeline state); `load_pack` itself remains the open half. Must be answerable before any pack loads. |
| `step` | done | `{ operator_json }` → `{ events, rendered, charts, ionic, scene, bench }` — `events` is the serde `Vec<Event>`, `rendered` the prose at the current register, `charts` the CAP-3 `Chart[]` the step's events earned (empty when none; first producer: the titration curve), `ionic` the GUI-092 `NetIonic[]` derived from the solved speciation (empty when none, which is the common case), `scene` the render model (one round trip repaints the bench). |
| `run_script` | done | `{ script }` → `{ steps: [{operator, events, rendered, charts, ionic}], scene, bench }`. |
| `parse` | done (GUI-005, 9a9c744) | `{ line }` → `{ ok, operator?, error? }`. Validate-only, never executes. Powers the command bar's live validation; `span` remains a candidate additive field. |
| `relations` | done (GUI-027) | `{}` → `[{ name, equation, args, purpose, validity, source, …_<locale> }]` — the CAP-5 named-relations catalogue. `args` is the CLI arg-spec string (`k=<hint>`, brackets for optional); clients build forms from it rather than hard-coding fields. Additive 2026-08-25 (GUI-087): `purpose` (what question it answers) and `validity` (where it stops being true). Additive 2026-08-29 (GUI-096): `source` — who published it and when, the leading clause of the same provenance line `calc` returns, so the catalogue and the computed result cannot cite different papers. Each prose field carries a `_<locale>` sibling per shipped language (`purpose_de`, `validity_de`, `source_de`); the unsuffixed field is English and is the per-string fallback, so a client selects `field_<locale> ?? field` and never a blank. Every one is non-empty for every row — a relation whose validity range is unstated teaches a learner to apply it outside that range. |
| `calc` | done (GUI-027) | `{ name, args: ["k=v", …] }` → `{ ok, value, unit, provenance, lv1, lv2, lv3 }` or `{ ok: false, error }`. One evaluation of a named relation; the result explains itself at every register and names its source. Same argument grammar as `kero calc`. |
| `balance_exercise` | done (GUI-095) | `{ equation }` → `{ ok: true, species, reactants, reversible, trivial, family, skeleton }` or `{ ok: false, error }`. **The question, and nothing that answers it.** The null-space solve behind it is `kerotakis-core::stoich::balance_report`, the same machinery `kero balance` prints, but the reply is a projection of that report rather than the report: no `coefficients`, no `matrix`, no `basis`. Both of the omitted ones are answers — the coefficients written down, and the matrix they are the null space of — and a browser is a place where anyone can read the reply. `species` is the formulas AS WRITTEN, reactants first; `reactants` is how many are left of the arrow; `skeleton` is the question on one line with every coefficient stripped, so a balanced equation and its bare skeleton set the identical question. `trivial` and `family` are facts *about* the answer that give none of it away: `trivial` says every coefficient is 1, so a drill can prefer a question worth asking; `family` says the skeleton admits more than one independent reaction, which a learner is entitled to know up front. Prose in the equation field is refused rather than balanced. |
| `balance_mark` | done (GUI-095) | `{ equation, answer }` → `{ ok: true, verdict, misses, factor, family }` or `{ ok: false, error }`. Marks one answer engine-side (`stoich::mark_answer`); `answer` is a JSON array of one integer per species, in the order `balance_exercise` listed them. `verdict` is `correct`, `multiple`, `unbalanced` or `incomplete` — `multiple` is the lesson GUI-095 exists for: `4 Mg + 2 O₂ → 4 MgO` conserves everything and is simply not the smallest ratio, so `factor` says what to divide by. `misses` names what does not cancel, worst first, with `amount` the signed surplus on the left. Nothing in the marking consults the solver's own coefficients, which is what lets an answer the engine never produced still be marked precisely. Stateless: the host sends back the equation it was asked, so nothing has to be remembered between drawing a question and marking it. |
| `balance_reveal` | done (GUI-095) | `{ equation }` → `{ ok: true, equation }` or `{ ok: false, error }`. **The one call that gives the answer up**, made when the learner asks for it. Written out as a sentence (`2 H₂ + O₂ → 2 H₂O`) rather than handed back as a coefficient vector, so one reveal cannot be quietly reused as the marking key for the next question. |
| `catalog` | done (WORLD-003) | `{ request: { mode, completed, awarded, mission_kit } }` → `{ mode, completed, items, packs }`. One joined answer to "what can this learner reach, and why": every apparatus verb, instrument (`measure:<token>`) and registry species, each with `minimum_completed`, `available`, and a `reason`. The rules live in `kerotakis_core::catalog` so every host answers identically — the browser owning them meant the desktop shell duplicated them or did without. **Sandbox availability is DERIVED as full** from the installed inventory rather than serialized as thousands of `unlocked = true` flags that go stale when packs change. `reason` is a stable tag with parameters, never prose: `sandbox`, `earned{minimum_completed}`, `awarded` (a closed case granted it permanently), `loaned` (the active mission supplies it), `locked{minimum_completed}`. The most durable reason wins, so a learner who owns a thing is told they earned it rather than that a mission lent it. An absent or unparsable `request` is a Story request at zero progress. `tests/contract/catalog-milestones-v1.json` pins the progression tiers, and both the engine and the not-yet-migrated client check themselves against it. |
| `particles` (operator) | done (BRD-001) | Not a command of its own: the `particles <vessel>` / `zoom <vessel>` OPERATOR draws the vessel's particle census and emits `ParticlesCounted { vessel, census }` into the ordinary event stream. It was a session command until 2026-09-03, which meant no script or corpus prompt could pose it — the engine could answer "what dissolved ions are present?" and the script surface could not ask. The event carries the whole `Census` (populations with labels, kinds and glyph counts; `per_glyph`; `too_rare`; and `source`) rather than a rendered string, because the picture is the same claim at every register and a host that draws rather than prints needs the populations. `source` distinguishes `speciation` (ratios from the aqueous engine's species distribution) from `inventory` (no solution was characterised, so ion pairs and complexes are unresolved and the picture is coarser than it looks) — a viewer is entitled to know which they are looking at. Reads state and changes none, like `smell`. The CLI's interactive `particles` and the MCP `json_particles` shape are unchanged; both call the same `particles::census`. |
| `sealed` (field) | done (EXP-30) | Not a command: an **additive object on every `--json` line** while a sealed unknown is on the bench, `{ vessels, placeholders }`. Sealed-unknown masking used to be a text-REPL guarantee only — the stream carried true species keys, on the reasoning that hosts key rendering, colours and spectra off those ids and rewriting them is a change hosts must be told about. The reasoning was sound; the conclusion was not. A mask that holds on the screen and not on the wire is a mask plus a way around it, and the wire is the easier of the two to read. So the ids are rewritten to the learner-facing alias throughout the line — contents, events, scene, speciation, provenance prose, and object keys as well as values — and this field is how a host is told. `vessels` is the vessels the unknown has reached (sealed on `add`, spread by every transfer out of a sealed vessel); `placeholders` is the alias strings appearing in this line in place of species ids. **It is deliberately not the mapping**: telling a host that `unknown-a` is really NaCl would be the leak with an extra step. A host that ignores the field sees ids it cannot look up and renders them as the unknowns they are, which is the correct behaviour; a host that reads it can say so deliberately. Absent — not empty — when no unknown is sealed. |
| `set_register` | existing | `{ level }` → `{}`. Presentation only; never re-solves. |
| `state` | existing | `{}` → `{ vessels, steps }` (full serde `Vessel`s — the lv3/machine contract). |
| `scene` | done (GUI-003) | `{}` → Scene JSON v1 (below). The render model; everything a bench canvas needs, nothing it must derive. Additive 2026-08-25 (GUI-058): SceneVessel carries `layers` — the liquid as visible layers, bottom first, one for a mixed solution, two when computed LLE splits the phases; volumes sum to `liquid.volume_l`. |
| `species` | existing | `{}` → shelf list: key, name, formula, phase, appearance, provenance, hazards, hazard_assessed — plus visual fields: `srgb` (reflective colour), `solution_srgb` (computed 0.1 M / 1 cm transmitted tint), `flame` (characteristic flame-colour word). `hazards` is a string array of GHS-style labels from the CAP-11 safety matrix; `[]` = no hazard classification (inert species). `hazard_assessed` is a boolean: `true` when the species has an explicit safety-matrix row (including explicitly inert), `false` when unassessed — clients should show "unassessed" rather than "safe". |
| `look` / `inspect` / `particles` | existing (`Lab` methods, not yet WorkerCommands) | `{ vessel }` → observation / `{rendered, vessel}` / `{census, rendered}`. |
| `reset` | existing | `{}` → `{}`. Bench only; session (register, packs, cache) survives. |
| `snapshot` | done (O(1) undo) | `{}` → `{ snapshot }` — the bench as an OPAQUE token (today: `Bench` serde JSON; clients must not parse it). Session state is not in it. |
| `restore` | done (O(1) undo) | `{ snapshot }` → `{}`. Replace the bench with a `snapshot` token; must be indistinguishable from replaying the prefix the snapshot was taken after. Session survives, exactly like `reset`. |
| `load_pack` | done (DATA-010) | wasm: `loadPack(bytes)`; shell: `{ bytes_b64 }` → `{ added, skipped, loaded_total }`. A `.pack` (KREG magic, version, embedded sha256, registry-document payload; `kero pack export`, shipped as `packs/*.pack` + hashed `packs/index.json`) adds species to the shelf AND every lookup at runtime; built-ins are never shadowed; corruption refuses by hash; one bad record refuses the whole pack. Spectra are data since DATA-011: a loaded species' 16-band spectrum colours its solutions exactly like a built-in's (conformance-pinned with a pack dye). `load_cache` per WEB-002. |
| `cancel` | existing (needs `target`) | terminal `cancelled` for the target id. |

Promoting the `Lab`-only methods to `WorkerCommand` variants is part of
GUI-004 (the UI talks only to the host, never to `Lab` directly).

Quest outputs (`quest_start`/`step`) serialize as `{ kind, quest, … }`.
`claim_satisfied` additionally carries `claim` — the claim's stable id
(additive 2026-09-02). Clients must key on it: recognising a claim by
comparing its rendered title is prose matching, and two claims sharing a
title satisfy the wrong one. `constraint_violated` (WORLD-004) is spoken,
never blocking.

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
- `not_yet_modeled` carries a `cause` alongside its prose (added
  2026-09-04, `#[serde(default)]`, so older logs load unchanged). Values
  are kebab-case and stable: `no-solution`, `nothing-to-act-on`,
  `no-solver`, `rate-not-modelled`, `model-boundary`, `no-reviewed-datum`,
  `phase-not-in-registry`, `not-speciated`, `no-transport-path`,
  `boundary-mismatch`, `not-parameterised`, `not-in-any-database`, and
  `unclassified` for records written before the field existed.

  The `what` prose is unchanged and remains what a reader sees; the cause
  sits beside it so a client can GROUP refusals — "how many of these are
  the same gap?" — without matching sentences. The distinction clients will
  care about most is `phase-not-in-registry` (a phase the databases know
  and this lab does not carry: fixable here) against `not-in-any-database`
  (no shipped database defines it at all: not fixable anywhere).

  **It is for grouping and diagnosis, never for scoring.** A UI may sort,
  filter or count by it; nothing should derive a verdict about whether a
  question was answered from it, because that needs the question and the
  cause does not carry one. The reasoning is in `NotModelledCause`'s doc
  comment, at length, with the PR that got it wrong.

## Scene JSON v1 (GUI-003 — implemented in `kerotakis-core/src/scene.rs`)

### Scene/chemistry authority (BRD-070)

The scene is a one-way projection. Client physics may send the typed
`authority::SceneProposal` contract, but may never mutate serialized `Bench`
state or emit chemistry events. A vessel transfer is a cumulative fraction of
the interaction's initial charge plus a replay seed; `TransferReconciler`
compiles only the uncommitted remainder to `Operator::Decant` and advances only
from its returned `Transferred` receipt. Frame cadence is therefore
non-authoritative. Reduced-motion and headless hosts submit the same endpoint;
background hosts may suspend painting and polling but must let an accepted
atomic step finish.

Destinations are explicit (`Vessel` or a typed bench/tray/floor spill). Spill
compartments and accepted broken-container/spill events deliberately remain
BRD-073: collision impulse and destination can be proposed now, but cannot
discard matter or claim breakage before that chemistry-owned operator/event
semantics lands. Replay persists the proposal seed; visual randomness may use
it, chemistry amounts may not.

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

Additive 2026-08-29 (BRD-002): the scene carries a top-level `stock` array —
the shelf's *finite* bottles, in stable key order:

```json
"stock": [{ "key": "white_vinegar_5_percent", "remaining": 40.0, "unit": "g" }]
```

`unit` is the one the `add` grammar takes for that key: `mol` for a registry
species, and the recipe's own basis for a named material (`g` for a mass
basis, `mL` for a volume basis). The array is **omitted when empty**, and a
key that is absent from it is an *unlimited* supply, never an empty one — so
a host written before this field sees byte-for-byte what it saw before, and
the sandbox default is unchanged. Bottles are filled by the `StockShelf`
operator (`stock <species|material> <amount><mol|g|mL>`) and drawn down by
`Add`/`AddMaterial`; a draw against an empty bottle mutates nothing and
reports `Event::StockExhausted` with both numbers. The level lives on
`Bench`, so the opaque `snapshot`/`restore` token round-trips it with
everything else.

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
- One `titrate` step may now earn **two** charts (EXP-39). The pH curve
  is unchanged. A redox titration additionally emits a `pe` chart —
  y label `pe`, same x axis — as a separate chart rather than a second
  series, because pH and pe do not share a y axis. Its series is
  `scatter`, not `line`, and deliberately: the curve carries no point
  for a step where the engine declined to pin a potential (at
  equivalence the electron balance has no root), and joining across
  that gap would draw a line through the one point that has no value.
  Consumers that render only the first chart are unaffected.
- Renderer duties: nice 1/2/5 ticks, responsive SVG, the same data as a
  table for screen readers, SVG export. Numbers arrive in data units —
  the renderer never converts; the emitter labels.
- Proposed additive extensions (not yet in the Rust contract; do not emit
  until they land there): a per-series `confidence` field rendered via
  GUI-023's stroke encoding, and x-axis `markers` (e.g. "equivalence").

## The ionic contract (GUI-092; authoritative in `kerotakis-core/src/ionic.rs`)

A step object may carry `ionic: [NetIonic]` — the net ionic equation the
step's chemistry earned, derived from the solved speciation rather than
stored per reaction. Empty is the common and honest case.

```json
{
  "vessel": 0,
  "basis": "precipitation",
  "reactants": [
    { "species": "Ag+", "label": "Ag⁺", "coefficient": 1, "charge":  1, "phase": "aqueous" },
    { "species": "Cl-", "label": "Cl⁻", "coefficient": 1, "charge": -1, "phase": "aqueous" }
  ],
  "products": [
    { "species": "AgCl", "label": "AgCl", "coefficient": 1, "charge": 0, "phase": "solid" }
  ],
  "spectators": [
    { "species": "Na+",  "label": "Na⁺",  "coefficient": 1, "charge":  1, "phase": "aqueous" },
    { "species": "NO3-", "label": "NO₃⁻", "coefficient": 1, "charge": -1, "phase": "aqueous" }
  ],
  "equation": "Ag⁺(aq) + Cl⁻(aq) → AgCl(s)",
  "provenance": "PHREEQC (IPhreeqc) · wateq4f.dat · Debye–Hückel"
}
```

- `basis` is `precipitation` or `neutralisation` — the two engine results
  that carry their own participants. It is not a reaction-class guess: a
  step whose participants are unknown produces no entry at all.
- `species` is the engine's own name (PHREEQC notation); `label` is the
  same thing typeset for a reader. Clients display `label` and match on
  `species`. `equation` is the assembled line, so a client that only wants
  to print one needs no term logic.
- `spectators` are the charged species the solver left in solution taking
  no part, most abundant first. Empty is a real answer.
- `provenance` is absent where the vessel records no solver.
- Register: hosts show this at lv2 and above. At lv1 an equation is not
  the register's business.

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

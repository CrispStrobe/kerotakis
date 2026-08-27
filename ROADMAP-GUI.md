# Kerotakis — GUI roadmap: one bench, one dial, five platforms

Status: 2026-08-27. Companion to `ROADMAP-Webapp.md` (backend/science),
`CAPABILITIES.md` (CAP tasks), and `BREADTH.md` (`BRD-*` data, physics and
scientific-view prerequisites). This document decides the client architecture
and the user experience for the real virtual lab: Web, Windows, macOS, iOS,
Android (Linux desktop falls out for free). It is subordinate to the
distribution and licensing invariant in `ROADMAP-Webapp.md`: every UI
dependency must pass the permissive allowlist (MIT / Apache-2.0 / BSD /
ISC / Zlib / CC0 and legal equivalents; no GPL, LGPL, MPL, or ShareAlike
anywhere in a shipping payload).

## Executive decision

**Build one UI, in web standards, and ship it five ways.**

- The UI is a single TypeScript + Svelte application rendering the **bench** as
  SVG with a Canvas2D effects layer. No 3-D game world and no game engine.
  Scoped scientific viewers for molecules, crystals, orbitals or proteins may
  use WebGL only behind the `BRD-080/081` contract; they never replace the
  accessible 2-D bench or become authoritative simulation state.
- **Web**: the existing PWA grows into it. Engine = `kerotakis-wasm` +
  IPhreeQC in one module Web Worker (this is OPT-11; the GUI depends on it).
- **Desktop (Windows/macOS/Linux)** and **mobile (iOS/Android)**: the same
  UI in a **Tauri v2** shell (MIT/Apache-2.0), with the Rust core compiled
  **natively** and run in-process on a background thread — no wasm penalty,
  real threads for CAP-2 sweeps and CAP-8 Monte Carlo, memory-mapped data
  packs.
- The UI never talks to the engine directly. It talks to an **`EngineHost`
  protocol** — the same JSON contract in both transports. The web worker and
  the Tauri command layer are two ~200-line implementations of one interface.

**The UX thesis: the register dial is the product.** The engine already
renders the same computation at lv1 ("what you see"), lv2 ("equations and
amounts"), and lv3 ("everything"). The GUI extends that dial from prose to
the entire interface — layout, vocabulary, numerals, chart density — so a
nine-year-old and a graduate student use *the same app on the same bench*,
one turn of a dial apart. We do not build a kids' app and a pro app. A child
grows up inside the same application, and the dial is mid-session switchable
on a running experiment: that demonstration ("turn the dial and the cloudy
beaker becomes a speciation table") is the product's signature move.

**The product form is now an exploration game, not a text adventure.** The
bench is a place the learner inhabits and manipulates. Story mode gives that
place characters, changing problems, discoveries, resources, and an expanding
equipment cabinet. Sandbox exposes the complete laboratory immediately. Both
modes use the same computed chemistry and direct-manipulation UI; neither is a
sequence of dialogue choices wrapped around commands.

This is a game loop without a game engine. DOM/SVG/Canvas remains the correct
technical choice: game structure comes from persistent world state, spatial
interaction, missions, progression, feedback, and authored consequences—not
from 3-D rendering.

### Product precedents and what we take from them

- **Kerbal Space Program:** clearly separate directed progression from a true
  all-parts-unlocked sandbox; let knowledge and capability grow together; make
  failure informative and experimentation the main verb. We do not copy its
  simulation domain, economy, or visual language.
- **Scratch:** one inviting creative surface works for children and adults;
  saturated colors identify actions against a calm workspace; high-contrast
  and localization modes are product features, not afterthoughts. We do not
  make chemistry look like programming blocks.
- **BASF's Virtual Lab:** begin an experiment by choosing tangible materials
  from a cabinet and make apparatus visible throughout the procedure. We keep
  that immediacy but remove the corridor: goals are validated against the
  simulated world and may be solved in different ways.

References checked 2026-08-26: [KSP game modes](https://privatedivision.com/games/kerbal-space-program),
[KSP manual](https://www.kerbalspaceprogram.com/files/KSPedia-XB1.pdf),
[Scratch about](https://scratch.mit.edu/help/about),
[Scratch getting-started guide](https://resources.scratch.mit.edu/www/guides/en/scratch-getting-started-guide.pdf),
and [BASF experiment catalog](https://basf.kids-interactive.de/experimente).

## Why this stack and not the alternatives

The decisive constraints:

1. **Five targets, one small team.** Any per-platform UI multiplies work 5×.
2. **Schools run Chromebooks and old Androids.** The web target is not a
   checkbox; for the largest classroom population it *is* the product. That
   disqualifies any framework whose web story is second-class.
3. **Licence allowlist.** No LGPL (Qt out), no GPL-or-commercial dual
   (Slint out), no MPL.
4. **The engine's outputs are text-, table-, and chart-shaped.** A lab app
   is 80 % documents, tables, forms, and typography, 20 % scene. That favors
   DOM/SVG over immediate-mode or game-engine rendering.
5. **Accessibility is non-negotiable for a school product.** A real DOM
   gives screen readers, keyboard nav, IME, localization, and font scaling
   for free; every canvas-first framework reimplements them poorly.

| Candidate | Licence | Verdict |
|---|---|---|
| **Web UI + Tauri v2 shells** | MIT/Apache throughout | **Chosen.** Reuses the shipped PWA and wasm path; system webviews keep binaries ~5 MB; native Rust core on installed targets erases the wasm→JS marshalling cost (OPT-11/OPT-9 remain web-only concerns). Tauri v2 mobile is stable since late 2024. |
| Flutter | BSD-3 | Strong mobile, but web is CanvasKit: multi-MB runtime, weak a11y/SEO, poor text selection — unacceptable for the Chromebook population. Adds Dart beside Rust. Duplicates the existing web investment. |
| egui | MIT/Apache | Right for an expert instrument panel, wrong for a child-facing product: immediate-mode text layout, IME, and a11y (AccessKit helps but trails DOM) are weak; mobile is DIY. Rejected as the app, fine for internal debug tools. |
| Dioxus / Leptos (Rust UI) | MIT/Apache | Keeps one language, but mobile is immature and the ecosystem (charts, testing, a11y tooling) is a fraction of TS's. The JSON `EngineHost` boundary means the UI language choice is not load-bearing; choose the ecosystem. |
| Bevy / Godot | MIT | A virtual lab sounds like a game, but the content is tables, equations, provenance, and prose. Game engines make the 20 % scene easy and the 80 % document layer hard, ship large web builds, and have weak a11y. Rejected. |
| Qt, Slint, Compose Multiplatform | LGPL / GPL-dual / Apache | Qt and Slint fail the allowlist outright. Compose's web target is alpha and it adds Kotlin. Rejected. |

**Hedge:** because the UI is a standards-compliant web app first, the shells
are replaceable. If Tauri mobile disappoints on old school Androids, the
identical UI ships in a thin hand-rolled WKWebView/Android WebView wrapper,
or stays a PWA on that platform. No UI work is hostage to the shell choice.

### Frontend dependency allowlist (all require LIC source records before use)

| Dependency | Licence | Role |
|---|---|---|
| Tauri v2 (+ wry, tao) | MIT OR Apache-2.0 | Desktop + mobile shells |
| TypeScript | Apache-2.0 | UI language |
| Svelte 5 + Vite | MIT | UI framework + build (compiles away; smallest runtime of the mainstream options — this is the low-end-device pick; React/Solid acceptable substitutes if contributor pool demands) |
| d3-scale, d3-shape, d3-interpolate | ISC | Chart primitives under the CAP-3 renderer (no d3 megabundle) |
| Lucide icons | ISC | Iconography |
| vitest + Playwright | MIT | Test harness |
| System font stack | — | No shipped fonts: smaller payload, native feel, no OFL review needed. Revisit only if typography demands it. |

Explicitly avoided: chart libraries with MPL/proprietary tiers (Highcharts,
AG-Grid), moment/luxon-style locale megabundles, any CSS framework — the
design system is bespoke and tiny, themed by CSS custom properties (the
current PWA palette is the seed).

## The UX

### One bench, three registers, one dial

The register dial is permanent UI — a three-position control, always visible,
top-level. It changes *presentation*, never *simulation*: same vessels, same
ledger, same events.

| | lv1 — Look | lv2 — Measure | lv3 — Model |
|---|---|---|---|
| Audience center | ~9–13, curious adults | secondary / intro college | experts, teachers, developers |
| Bench | full-bleed, big targets, playful apparatus | same scene + numeric badges (T, pH, V) on vessels | same scene + gauges, saturation chips, boundary annotations |
| Amounts | "a pinch, a spoonful, a cup" (the engine already parses these) | grams, moles, mL with unit-aware input | full quantity grammar, tolerances, uncertainty |
| Events | narrated observations: "It went cloudy!" | + balanced equations, quantities | + speciation deltas, model provenance, routing reasons |
| Numbers | hidden until asked | shown | shown with uncertainty and confidence class |
| Instruments | qualitative ("warm", "very acidic") | numeric readouts, live charts | + calibration, detection limits, noise model (R6) |
| Extra surfaces | lesson story cards | notebook with tables, chart export | command bar, provenance drawer, studies, diagrams |

Vocabulary, sentence length, and layout density are register functions. The
engine already owns the prose problem per register; the GUI extends the same
switch to chrome and controls.

### The five surfaces

All responsive states of one component set — desktop shows them side by
side, a phone shows bench + bottom sheets.

**1. The Bench** (center). Vessels drawn as 2-D *cross-sections* — a
deliberate choice over 3-D: a cross-section shows what the chemistry teaches
(liquid color and depth, precipitate layers, headspace volume and pressure,
electrode surfaces, condenser path) and what a photorealistic render hides.
Liquid color comes from the engine's computed spectra (`appearance.rs` /
`spectrum.rs`) — Beer–Lambert over the actual path length of the drawn
vessel, so a wider beaker really reads darker. Precipitates accumulate as
textured layers (never color-only encoding). Bubbles, venting, and steam are
a Canvas2D effects layer driven by event data, decorative in style but never
in *fact* — no effect fires without a computed event behind it.

Apparatus snaps to sockets: hotplate under a vessel, burette above, condenser
between two, electrodes + supply + wires (R4), separatory funnel (R2). The
apparatus palette grows exactly as the roadmap's apparatus models land;
nothing appears on the shelf before it has a model behind it.

**2. The Shelf** (left / bottom drawer). The species registry, searchable,
with formula, computed appearance swatch, phase icon, and hazard chips —
served by the existing `species()` JSON, whose doc comment already says
"what a UI offers on a shelf". Drag onto a vessel (or tap-tap on touch),
then an amount picker whose units follow the register. Model packs appear
here as installable shelf sections.

**3. The Feed / Notebook** (right / bottom sheet). The rendered event
stream — the same lines the CLI prints, at the current register. This
surface is triple-duty:

- it is the **lab notebook**: autosaved, timestamped, exportable to
  Markdown/PDF with charts inline;
- it is the **accessibility surface**: the entire lab is, by construction,
  a legible text document; a screen-reader user runs the whole bench from
  the feed plus the command bar and loses nothing that matters;
- it is the **teaching mirror**: every GUI gesture prints the operator
  command it compiled to (`add v1 NaCl 5.8g`) alongside its narrated result,
  so the GUI continuously teaches the scripting language.

**4. The Inspector** (tap a vessel). Register-dependent detail: lv1 gets a
picture and one sentence; lv2 gets contents, amounts, and measurements; lv3
gets the full `inspect` tables — speciation, activities, γ, SI, the
conserved ledger, and a **provenance drawer**: which model computed this,
from which dataset, why routing chose it, and where its validity ends
(rendered from the R0 capability/validity reports when they land).

**5. The Command Bar** (⌘K / `/`). The full command grammar with completion,
inline validation (via a parse-only engine endpoint), and history. It is the
keyboard path for accessibility, the expert's fast path, and the CLI-parity
guarantee. The current terminal PWA is this surface's ancestor and remains
available as a "console" view.

### Interaction principles

1. **Direct manipulation compiles to operators.** Every drag, pour, and knob
   emits a command from the *same grammar* the CLI parses; the GUI holds no
   chemistry state of its own. One source of truth; CLI/TUI/GUI/MCP parity
   is structural, not maintained; every session **is** a `.lab` script.
2. **Undo is replay.** The engine is deterministic and the operator log is
   authoritative, so undo/redo = replay a log prefix (server-side snapshot
   caching keeps it O(1) in practice). The same machinery gives a **timeline
   scrubber** — drag back through an experiment — and lesson playback.
3. **Observation before explanation.** lv1 shows what you would see;
   the equation, the mechanism, and the numbers are each one tap deeper,
   never forced on the learner and never withheld from them.
4. **Hazards teach.** A hazard verdict interrupts with a card that warns
   precisely, then offers *show me why* — running the chloramine formation
   the warning describes. Prohibition without demonstration teaches nothing
   (the engine's own philosophy; the UI must not flatten it into a toast).
5. **Honest boundaries are first-class UI.** `unsupported` / outside-validity
   outcomes render as a distinct "the model can't say" panel — visually
   unmistakable from an error and from a null result. The confidence
   vocabulary (`computed` / `curated-family` / `curated-instance` /
   `estimated` / `qualitative` / `unsupported`) gets one fixed visual
   encoding (badge + border treatment) used everywhere a number appears.
6. **Everything leaves the app.** Notebook → Markdown/PDF; charts → SVG/PNG
   (the CAP-3 contract renders identically in-app and to file); session →
   `.lab`; state → JSON. Nothing a learner makes is trapped.

### Modes over the same bench

The start screen has two equally durable doors, with **Story** presented first
to new users and **Sandbox** always visible beside it. Switching mode never
destroys or rewrites the other mode's world.

- **Story** — a persistent laboratory and surrounding world. Missions arrive
  through characters, places, samples, and incidents. Solving them earns lab
  access, instruments, trusted suppliers, research knowledge, and new regions.
  The learner is free to explore, run unrelated experiments, and keep several
  missions active. Progress gates equipment and situations, never scientific
  truth or the register dial.
- **Sandbox** — every released apparatus, reagent, operation, study tool, and
  model pack is available from the first frame, with replenishable supplies
  and no progression economy. Missions can be loaded as optional scenarios,
  but rewards do not leak into Story. Sandbox is the teacher, expert, content
  author, and pure-play surface—not a lesser "creative mode."
- **Studies** (lv3, available in both worlds, rides CAP-2/3/4/8) — parameter
  sweeps, live-drawn titration curves, predominance/Pourbaix diagrams, and
  Monte Carlo bands. The GUI is a thin form plus the CAP-3 chart renderer; the
  engine does everything.

Lessons become replayable demonstrations in the notebook and mission hints;
Challenges become mission objective types. Neither remains a separate top-level
mode. The old `.lab` scripts remain importable in either world.

The launch hub must not confuse content formats with the product's modes. Its
four doors are one connected body of work: **Story missions** currently expose
the 27 shipped `.lab` investigations as guided, replayable starting points;
the **Experiment Library** exposes the 103 exported Codex reactions with the
predict → theory/procedure → run/check loop; the **Concept Map** connects those
experiments through the 189 concepts and 28 models in the same export; and
**Sandbox** opens the unrestricted bench. `EXPERIMENTS.md` is the audited
sixteen-experiment content/mission plan, while source examples and test fixtures
are engineering evidence—not a hidden fifth user catalog. Counts are rendered
from shipped indexes, never copied into UI strings.

Mode invariants:

1. Story and Sandbox have separate, versioned save namespaces and independent
   undo histories.
2. Mode switching is available from the pause/home surface at all times and
   confirms only when an operation is actively running.
3. Sandbox always reflects the complete installed registry, including newly
   installed packs; no unlock flag may hide an item there.
4. Story restrictions live in orchestration/catalog policy. The chemistry
   engine receives the same operation and computes the same result in both.
5. Story rewards may reduce friction or open possibilities; they never sell
   answers, improve physical laws, or introduce paid/artificial scarcity.

### Classroom and privacy posture

No account required, no telemetry by default, fully offline after install
(PWA service worker already exists; Tauri bundles everything). Sessions save
locally; a teacher collects exported notebooks/`.lab` files. Multi-user sync,
classrooms-as-a-service, and cloud saves are **non-goals** for this roadmap.

### Accessibility and localization

- The feed + command bar make the app operable and legible without the
  pointer or the canvas; the bench SVG additionally carries ARIA labels and
  keyboard focus for every socket and vessel.
- Color never carries meaning alone: computed colors are paired with the
  engine's color names; precipitates and phases get texture and shape.
- Touch targets ≥ 44 px; the whole lv1 register is one-hand phone operable.
- UI chrome is locale-keyed and English/German ship together. Structured engine
  events remain the long-term requirement for fully localized scientific prose:
  every new event crosses the protocol as an id plus typed parameters, never as
  an English-only sentence. German expansion is part of each feature's DoD,
  including mission text, equipment metadata, errors, ARIA labels, and export
  templates; screenshots are not accepted as evidence of coverage.

### Immediate experience target: the laboratory as a place

The first UX release does not wait for the whole campaign. It transforms the
current control-dense page into one readable room with a strong spatial model:

```text
┌ world / mission ─────────────── register · language · mode · pause ┐
│ ┌ equipment cabinet ┐  ┌──────── active bench ────────┐ ┌ journal ┐│
│ │ search / category │  │                               │ │ goal    ││
│ │ rendered objects  │  │  glassware, tools, effects   │ │ clues   ││
│ │ supplies / kit    │  │  sockets and work zones      │ │ evidence││
│ └───────────────────┘  └───────────────────────────────┘ └─────────┘│
│ selected object · contextual physical actions · undo / time          │
└──────────────────────────────────────────────────────────────────────┘
```

- **The bench owns the screen.** At desktop widths it receives at least 60% of
  the usable area; cabinet and journal collapse to labeled drawers. On phones,
  the bench remains visible behind one bottom sheet at a time.
- **Objects, not forms, are the primary controls.** Drag a beaker to the bench,
  place it on a hotplate, insert a probe, turn a stopcock, and pour by dragging
  source toward target. A selected object exposes only actions valid now.
- **The cabinet replaces the undifferentiated shelf.** Its tabs are Glassware,
  Instruments, Materials, Reagents, and Saved Kits. It shows a rendered object,
  name, one-line purpose, compatibility, availability, and modeled-confidence
  marker. "Store" is a supply-room metaphor only—there is no real-money shop.
- **Mission UI stays peripheral.** The current objective is one compact card;
  evidence and hints live in the journal. Dialogue never covers the apparatus
  while the user is acting.
- **Feedback is local and physical.** Valid sockets glow, liquid follows the
  pour, instrument readings appear on the instrument, and results enter the
  notebook. Toasts are reserved for global state such as unlocks or save errors.
- **The command bar becomes an expert accelerator.** It remains complete and
  accessible behind `/` or Command/Ctrl-K, but is no longer the visual center.

### Visual system: bright scientific workshop

The visual character is optimistic, precise, and a little adventurous. Large
silhouettes, rounded geometry, tactile controls, expressive motion, and concise
labels make it approachable; disciplined spacing, real notation, restrained
chrome, and high information density at lv3 keep it credible for professionals.
No mascot is required to make the interface friendly.

The default light palette below replaces brown/steampunk chrome. Exact values
remain design tokens and must pass automated contrast tests before shipping.

| Token | Light | Dark / pro | Use |
|---|---:|---:|---|
| `--lab-bg` | `#F4FAFF` | `#101D2B` | room/canvas outside the bench |
| `--surface` | `#FFFFFF` | `#17283A` | panels and cards |
| `--surface-raised` | `#EAF4FB` | `#20364B` | drawers, selected work zones |
| `--bench` | `#D8E9F2` | `#29475A` | physical work surface |
| `--ink` | `#17324D` | `#EAF5FC` | primary text and structural lines |
| `--ink-muted` | `#5D7488` | `#A9BECE` | secondary text |
| `--edge` | `#B7CEDD` | `#45647A` | dividers and inactive outlines |
| `--primary` | `#2F70E8` | `#78AAFF` | navigation, focus, selected object |
| `--instrument` | `#0E7180` | `#55D0D8` | measurement and instrument state |
| `--action` | `#A94F00` | `#FFB35C` | primary physical action / start |
| `--discovery` | `#7656D6` | `#A98CFF` | research, story clue, unlock |
| `--success` | `#247A50` | `#57CE91` | verified result / mission complete |
| `--warning` | `#A65A00` | `#FFC15A` | caution and recoverable hazard |
| `--danger` | `#C63F4A` | `#FF7B84` | stop, injury/equipment danger |

Color semantics are fixed across registers. Blue means selection/navigation,
cyan measurement, orange physical action, violet discovery, green verified,
amber caution, and red danger. Chemical appearance colors come from the engine
and are **never remapped to theme colors**; neutral bench and vessel surrounds
protect their readability. Every semantic color also has an icon, label, shape,
or border treatment.

Register changes density, not brand:

- **lv1 Look:** larger apparatus, 52 px minimum targets, short labels, warmer
  surfaces, more motion and illustration, one emphasized action at a time.
- **lv2 Measure:** 44 px targets, permanent units/readings, two-panel layout,
  subtler motion, numeric comparison close to the apparatus.
- **lv3 Model:** compact 36–40 px controls where pointer input permits, dark
  theme offered by default, dense tables/charts/provenance, full keyboard path.

Required variants: light, dark, high contrast, reduced motion, 200% text zoom,
and color-vision-safe hazard/phase encodings. Use a 4 px spacing grid, an 8 px
base radius (16–24 px for drawers/cards), firm 2 px interactive outlines, and
short 120–220 ms state transitions; continuous chemistry animations may be
longer because their duration conveys computed magnitude.

## Architecture

```text
┌─────────────────────────────────────────────────────┐
│  UI (TypeScript + Svelte)                            │
│  bench SVG · shelf · feed · inspector · command bar  │
│  CAP-3 chart renderer · lesson player                │
└──────────────────────┬──────────────────────────────┘
                       │  EngineHost protocol (JSON, versioned)
                       │  step · runScript · state · scene · events
                       │  species · parse(validate-only) · chart · packs
        ┌──────────────┴───────────────┐
┌───────┴────────────┐      ┌──────────┴──────────────┐
│ WorkerHost (web)   │      │ TauriHost (native)      │
│ kerotakis-wasm +   │      │ kerotakis-core native,  │
│ IPhreeQC in ONE    │      │ background thread(s),   │
│ module worker      │      │ rayon for CAP-2/CAP-8,  │
│ (OPT-11)           │      │ mmap'd data packs       │
└────────────────────┘      └─────────────────────────┘
```

- The UI is a pure projection of `(scene JSON, events JSON, state JSON)`;
  it holds interaction state only. All engine calls are async in both hosts,
  so the UI code cannot tell the transports apart and never blocks.
- **Scene JSON** is the one new engine contract of consequence: a versioned,
  per-vessel render model — liquid volume + sRGB color (+ path-length
  basis), precipitate layers, bubbles/venting flags, headspace P/V, T,
  attached apparatus, boundary condition. Most fields exist in `state()` and
  `appearance` today; this formalizes them so native and web render
  identically and golden tests can pin them.
- **Events as data**: `step()` returns the typed events as JSON alongside
  (eventually instead of) prose; the register renderer becomes callable on
  both sides of the boundary during migration.
- **Determinism is a test asset**: scene JSON over replayed `.lab` scripts
  gives golden-file UI tests (DOM snapshot per step per register) with no
  flakiness; Playwright drives the five canonical lessons per release.

## Performance budgets (CI-gated, per the backend roadmap's low-end mandate)

| Budget | Target device: 2018-class Chromebook / Android 10 WebView |
|---|---|
| Cold start to interactive bench (web, cached SW) | < 3 s |
| Core payload (UI + wasm + aqueous pack, gzipped) | < 6 MB; further packs on demand |
| Engine call round trip (UI-visible) | UI paints an optimistic "working" state < 100 ms, always |
| Main-thread long tasks during a step | 0 (engine fully off-thread in both hosts) |
| Bench frame budget with effects | 60 fps desktop, 30 fps floor on target device; effects degrade first, data never |
| Tauri bundle (desktop) | < 15 MB installed |

## Execution

Ordered, dependency-honest, one reviewable change each. `LIC` gates apply to
every new dependency before its first import.

### Phase G0 — Contracts (no visible UI change)

- [ ] **GUI-001 — `EngineHost` protocol v1.** Specify the JSON contract
  (step/runScript/state/scene/events/species/parse/chart) as a versioned
  document + conformance test that both hosts must pass. The current wasm
  API is the seed.
  *Status 2026-08-24 (2nd pass): spec written — [PROTOCOL.md](PROTOCOL.md).
  Conformance runs against BOTH shipping hosts: the CLI/MCP surface
  (`crates/kerotakis-cli/tests/protocol_conformance.rs`, in the test
  suite) and the wasm host (`tools/test-protocol-conformance.mjs`,
  919 structural checks over the lesson corpus, wired into the CI wasm
  job) — one shape, drift fails before a client sees it. Open for the
  checkbox: the same suite against TauriHost once the shell builds
  (GUI-030), and hello's remaining fields.*
- [ ] **GUI-002 — Events as structured JSON.** Expose the typed `Event`
  enum over the boundary with stable ids + params; decide engine-side vs
  client-side register rendering and the locale-pack shape.
- [ ] **GUI-003 — Scene JSON v1.** Per-vessel render model derived from
  existing state + appearance; golden-file tests over replayed lessons.
  *Status 2026-08-24: implemented — `kerotakis-core/src/scene.rs` (liquid
  colour+word, solids with metallic/precipitate split, headspace, badges,
  lv1 words), shape-pinning + behaviour tests in-module, wired into the
  wasm `step`/`run_script` responses and `Lab::scene()`. Open for the
  checkbox: goldens over the replayed lesson corpus (folds into GUI-001's
  conformance suite).*
- [ ] **GUI-004 — One-worker web engine.** Land OPT-11 (lab + IPhreeQC in a
  single module worker) behind `WorkerHost`; the current PWA runs on it
  unchanged. Measure; OPT-9 only if numbers demand.
- [ ] **GUI-005 — Parse-only endpoint.** Validate a command without
  executing (for live input validation and drag-legality preview).

### Phase G1 — The bench, on the web

- [ ] **GUI-010 — App shell.** Svelte + Vite + TS scaffold in `web/`,
  PWA/service worker carried over; the terminal view survives as "console".
  *Status 2026-08-24: scaffold landed in `web/app/` — EngineHost client
  layer (`RequestChannel` correlation, `WorkerHost`, engine worker over
  the WEB-002 envelope; 6 vitest tests green), Session store, first
  Bench/Vessel SVG painting Scene JSON v1, Feed (aria-live notebook),
  CommandBar, RegisterDial; npm licence lint (73 packages allowlisted);
  production build 19 KB gzip. Open for the checkbox: PWA/service-worker
  carry-over, build-web.sh integration, and the console view.*
- [ ] **GUI-011 — Bench canvas v1.** SVG vessels with computed color,
  precipitate layers, headspace, temperature badges; scene-JSON-driven;
  register dial wired end to end.
- [ ] **GUI-012 — Shelf v1.** Registry-driven, searchable, drag/tap-to-add,
  register-aware amount picker (pinch/cup ↔ g/mol/mL).
  *Status 2026-08-24 (2nd pass): tap-to-add and drag-to-vessel landed —
  registry-fed via `species`, search over name/formula/key, register-aware
  quick amounts. Open: hazard chips, appearance swatches — and SCALE: the
  shelf is structurally complete (it lists whatever the registry serves,
  79 today) but its flat list stops working past ~150 species. As the
  generated-registry tranches (CAP-21/CAP-23 pipeline, 23→65 landed
  2026-08-24) climb toward the hundreds, the shelf needs grouping
  (phase/family sections, codex-topic cross-links) with search staying
  primary. The species COUNT is data-side work and is deliberately
  tranche-gated: every entry arrives with molar wiring, appearance,
  safety row, provenance, and InChI identity — a shelf of hundreds of
  unverified names is the lookup-table failure this project exists to
  avoid.*
- [ ] **GUI-013 — Feed v1.** Event stream at current register with command
  echo; this is also the a11y acceptance gate: run a full lesson from
  keyboard + screen reader.
- [ ] **GUI-014 — Inspector v1 + command bar.** Register-dependent detail;
  ⌘K bar with completion from the grammar + parse-only validation.
  *Status 2026-08-24: Inspector v1 landed — tap a vessel (vessels are
  buttons, keyboard-operable) for `inspect` detail at the current
  register, refreshed after every step and register switch, with a
  particles append. Open: completion + parse-only validation in the bar
  (waits on GUI-005).*
- [x] **GUI-015 — Undo/replay + timeline.** Log-prefix replay with snapshot
  cache; timeline scrubber; session autosave/restore; `.lab` import/export.
  *Status 2026-08-24 (2nd pass): undo/redo/scrubber are one cursor over
  the replayed log (jumpTo = reset + prefix replay; range-input timeline
  in the header; mid-history commands truncate the future); autosave to
  localStorage with replay-based restore, corrupt saves dropped; `.lab`
  export AND import (import composes onto the current bench, stops at
  the first rejected line naming file:line, fully undoable); `clear`
  distinct from jumpTo(0). All vitest-pinned.*
  *3rd pass (same day): snapshot cache landed — `snapshot`/`restore`
  protocol commands (opaque token, `Bench` serde round-trip; conformance
  proves restore ≡ replay and that garbage refuses cleanly), Session
  keeps one per log position (cap 40, truncation/clear invalidate) so
  undo/scrub is O(1); replay stays the fallback and the semantics.*

- [ ] **GUI-016 — Test deploy.** One payload from `tools/build-web.sh` —
  console at `/`, app at `/app/`, cross-linked, one shared engine — pushed
  to a Vercel preview (token + team scope per the machine's
  vercel-deploys.md, prebuilt static, no build on Vercel's side) and to
  GitHub Pages. "Switchable" is two links, not a framework; when the app
  reaches parity (end of G1) the surfaces swap places: app at `/`, console
  at `/console/`. Blocked only on a local `wasm-bindgen-cli 0.2.127`
  install + emsdk sourcing after the cargo lock frees.
- [ ] **GUI-017 — Continuous deploy.** `.github/workflows/pages.yml`
  (added: builds the payload with the same recipe ci.yml already uses and
  deploys to Pages on web-affecting pushes + manual dispatch); the Vercel
  production deploy stays a deliberate manual step until the register dial
  UX is demo-ready. The service worker now precaches the app's hashed
  assets (stamped by build-web.sh), so both surfaces are offline-first
  from one install; `web/vercel.json` pins immutable caching for hashed
  assets.

### Phase G2 — Learning surfaces

- [x] **GUI-020 — Lesson player.** *Done 2026-08-24:* lessons walk as
  guided overlays (LessonBar; free commands never move the cursor);
  deviation is counted and NAMED ("off the script by N — exploring is
  allowed"), and "return to the script" rewinds it as an undo to the
  state after the lesson's last own step (snapshot-fast, never an
  erasure — the wandering stays in the undone future). The map screen
  is GUI-053's. Vitest-pinned.
- [ ] **GUI-021 — Charts.** CAP-3 contract renderer (d3 primitives); first
  chart is the live titration curve (with CAP-12); SVG/PNG export.
  *Status 2026-08-24: renderer half done, dependency-free (hand-rolled
  scales/ticks beat adding d3 for one curve): Chart JSON v1 is specified
  in PROTOCOL.md, `Chart.svelte` renders it (nice ticks, uncertainty
  bands for CAP-8, confidence-encoded strokes, screen-reader data table,
  SVG export; core vitest-covered), and the feed renders any step
  carrying `charts` today. Export pass (same day): the SVG export now
  inlines each element's COMPUTED style plus a theme background —
  serializing the live node had silently dropped the scoped CSS and
  theme variables, saving an invisible file — and PNG export renders
  that styled clone at 2× through a canvas. Open: the engine emitting
  the contract (CAP-3's other half, with CAP-12's titrate verb).*
- [ ] **GUI-022 — Notebook export.** Markdown/PDF with inline charts and
  provenance footer.
  *Status 2026-08-24: Markdown half done (`notebook.ts` + "save notes") —
  commands as code, observations as prose, hazards as severity-labelled
  quotes, charts as data tables with provenance. PDF half (same day):
  a print stylesheet strips the chrome so the feed prints AS the
  notebook, charts drawn (not tabled) with their provenance lines; the
  header's "print" button opens the dialog and the browser's save-as-PDF
  is the PDF export — no dependency earns its keep against that. Open:
  inline chart images in the Markdown export.*
- [ ] **GUI-023 — Hazard cards + honest-boundary panels.** The fixed visual
  encoding for the confidence vocabulary, everywhere.
- [ ] **GUI-024 — Challenges v1.** Equation balancing, endpoint, buffer
  hold; engine-marked.

- [ ] **GUI-025 — The equation strip.** Reactions as balanced equations
  pinned beside the bench at lv2+, live as they happen. We already
  compute them; today they are buried in the feed.
- [x] **GUI-026 — Pour and stir.** *Shipped 2026-08-24 (SVG, not
  Canvas2D — the vessels' own layer was enough):* drop-to-add splashes a
  ripple at the surface; `gas_evolved` vents wisps from an open mouth
  (sealed vessels honestly show nothing); `titrated` drips from above;
  `mixed`/`diluted` swirl a dashed eddy one revolution; `flame_test`
  reuses the flame. Every one fires only on its typed engine event
  (session map is unit-tested). Decorative in style, never in fact.
- [x] **GUI-027 — Utilities drawer.** *Toolbox shipped 2026-08-24:*
  `relations`/`calc` are protocol commands on all three hosts (PROTOCOL.md;
  conformance checks the catalogue shape, one known-good evaluation, and
  an honest refusal). The drawer builds its forms from the engine's own
  arg-spec strings (`relationArgs.ts`, tested) and shows the
  RelationResult verbatim — value, unit, provenance, and the explanation
  at the bench's current register, so the dial reaches the calculator
  too. Remaining for this item: property correlations (CAP-6) and unit
  conversion join the same drawer when their engine surfaces exist.
- [x] **GUI-028 — Voice input.** *Shipped 2026-08-24:* a microphone
  button on the command bar (drawn only where the Web Speech API
  exists). The transcript lands IN the input — read it, correct it,
  then run it; nothing executes straight from the microphone, and the
  live parse validation judges spoken lines exactly like typed ones.
  Speaker's locale, lowercased into the grammar's case.

### The sandbox completeness invariant

**Every registry species, every apparatus, every engine verb is reachable
from the GUI — in sandbox mode, without the command bar.** The engine
already exposes ~25 verbs and the full registry; the gap is graphical
affordance, and it is checkable, so it becomes an invariant with a test
rather than an aspiration:

- [ ] **GUI-029 — The affordance manifest.** A `grammar` protocol command
  (engine-side: the verb list with argument shapes, from the one parser)
  plus a client-side manifest mapping every verb to the component that
  invokes it; a conformance test fails when a verb lacks an affordance or
  an affordance invents a verb. Registry coverage is already structural
  (the shelf lists the registry); this makes verb coverage structural too.
- [ ] **GUI-033 — Apparatus palette and instrument panel.** Graphical form
  for the rest of the verb set, driven by the codex's own apparatus
  vocabulary: hotplate/bunsen (heat, ignite), fridge coil (cool), clock
  (wait), lids (seal/regulate/sweep/open), funnel+paper (filter),
  separating funnel (drain), still (distil), burette (titrate), column
  (chromatograph, transport), lamp (irradiate), mortar (grind), electrodes
  + supply (cell, wire, electrolyze), and an instrument tray for the eight
  measure targets. Vessel context ring for per-vessel actions; everything
  emits the same command lines.

### The codex is the content engine (apply it, then expand it)

Each codex entry already carries a runnable `setup`, checkable `expect`
predictions, per-register prose, an `apparatus` list ("drives what a UI
puts on the bench"), concept/prerequisite edges into 189 defined concepts,
calculation and model taxonomies, and curriculum placements. The GUI has
been ignoring all of it:

- [x] **GUI-053 — The concept map.** *Shipped 2026-08-24, armed on the
  export:* the map screen draws the concept DAG layered by longest
  prerequisite chain (edges = each entry's `requires` → its `concepts`;
  pure, tested layering in codex.ts with a cycle guard). Below lv3 it
  reads as a skill tree — edges only for the concept in hand; at lv3 the
  full DAG shows. A concept fills when the learner ran an entry teaching
  it to a green check on this device (progress in localStorage, separate
  from the bench save; nothing is met by reading). Tapping a concept
  lists its entries ready-first with the missing prerequisites named,
  and hands an entry straight to the experiment page.
- [ ] **GUI-054 — The experiment page.** Every codex entry as a one-tap
  experiment rendered in the shape the national school-lab platforms
  proved on millions of students — one experiment, tabbed:
  **Theory** (the entry's register prose, concepts and models, at the
  dial's level), **Procedure** (setup narrated + apparatus list docking
  GUI-033's palette + the kit), **Bench** (the live simulator — where we
  beat that genre: computed, not scripted), and **Check** (the `expect`
  predictions as engine-marked viva questions, usable before, during and
  after). 103 catalogued experiments on day one without writing a
  lesson; every new codex entry is a new page for free. The guided
  what-do-I-do-next instruction panel those platforms center on is our
  LessonBar, extended with the entry's own narration.
  *Status 2026-08-24: the client side is BUILT and waiting — catalog +
  tabbed page (theory at the dial's register, procedure,
  predict-observe-explain with committed predictions and per-wrong-
  answer diagnoses, and an honest checker that COMPARES the engine's
  actual events and final state against the entry's expect claims,
  never recomputing chemistry; vitest-pinned). Lights up the moment the
  codex export ships `codex/index.json` in the payload (kero-basic's
  task); quiet absence until then.*
- [x] **GUI-055 — The curriculum browser.** *Client shipped 2026-08-24,
  armed on the codex export:* the experiments dialog gained three doors —
  all (with a filter), by concept (chips with counts; a selected concept
  filters the list and names its co-occurring neighbours, the GUI-053
  down payment), by curriculum (system → stage ordered by age band,
  placement citations shown, entries launch straight into the tabbed
  page). Grouping is pure, tested code in codex.ts; the export must
  carry `concepts` and `curriculum` per entry for the doors to light up.
- [ ] **Codex expansion (engine/content side, tracked here for the GUI's
  sake):** more entries toward 200+, more curriculum spines beyond the two
  German systems, apparatus vocabulary kept in lockstep with GUI-033, and
  registry growth (CAP-21's pack-generated registry is the mechanism)
  toward the reagent breadth the genre's benches offer.

- [ ] **GUI-057 — The elements, wired to the lab.** An interactive
  periodic table (all 118, structural facts only: number, symbol, name,
  group/period/block, category — category tinted AND worded) whose
  detail panel answers "what does the lab have of this element": shelf
  species containing it (parsed from their formulas), their chips and
  flame tests, tap-to-add to the bench. Deliberately NO transcribed
  numeric properties: masses and measured values come from the
  registry's provenance-carrying records, and broader element data
  (electronegativity, radii, configurations) arrives only via the
  licence-clean data ETL (Wikidata CC0 per the data roadmap) — the
  standalone-table genre's breadth, with our sourcing discipline.
  *Status 2026-08-24: v1 landed as described; ETL-fed properties open.*

### The visual bar (2026-08-24 addendum)

The national school-lab platforms set the visual/UX bar the user holds
us to: illustrated 2-D benches with believable drawn glassware on a
bench surface, guided next-step instruction, zoomed insets for
readings, and the tabbed experiment page above. Our structural answer
is GUI-054; the ART answer is a polish pass on the canvas — a drawn
bench surface and stand, glass with highlights and shadow, meniscus
curves, reading insets (zoom on the badge tap) — tracked as the
GUI-033/026 finishing pass. The physics stays ours; the drawing must
stop looking like wireframes.

The 2026-08-26 direction raises that bar from "illustrated experiment page"
to **coherent explorable laboratory**. A collection of good apparatus drawings
is not enough: objects need scale, stable locations, sockets, selection states,
local controls, storage homes, and continuity between visits. The room may use
subtle parallax and depth cues, but remains a readable 2.5-D stage rather than a
free-camera 3-D environment.

### What the virtual-lab genre teaches (survey 2026-08-24, contenders
deliberately unnamed here)

The genre spans touch-first mobile benches (tactile pours, live
equation displays, outcome-pair chemistry), the freeware worktable
generation (drag-drop plus a utilities drawer), VR platform vendors
(assessment analytics as the selling point), scripted 3D academic labs
(catalogs of 100+ canned experiments, animated tutor characters), and
database-driven browser simulators (thousands of reaction rows, voice
input, procedure training). Every one of them runs on a script or a
lookup table, and each plateaus exactly at its catalog — the limitation
lists read "these procedures are not in the database." None combines
computed chemistry + one app from child to expert + web/offline-first +
an open licence; that intersection is ours. Adopted from the genre:
GUI-025..028 above. Rejected as recorded decisions: 3D environment
fidelity, VR-first, and an always-on animated tutor that competes with the
apparatus. Story characters may appear through brief in-world messages,
portraits, and visits, but the lab remains the agent of learning. After the
immediate UX foundation, the honest competitive lever is the chemistry-backed
mission corpus rather than ornamental scene volume.

### Phase G2.5 — UX first, then the world

This phase is the immediate priority. GUI-070 through GUI-075 make the current
app delightful before campaign breadth is built; GUI-076 through GUI-080 turn
that foundation into the first playable story slice.

**Owner priority reset, 2026-08-27:** stop expanding or redesigning missions,
examples, and progression until the shared laboratory is a convincing place to
work. Existing content must flow through the improved laboratory unchanged.
The order is now: cabinet/search/quantities → free spatial bench and object
controls → apparatus with computed motion → journal/environment polish → only
then more campaign breadth. A mission may suggest Prepare/React/Analyse, but it
must never own the physical layout; those guides are optional and Sandbox may
hide them completely.

- [ ] **GUI-070 — Recompose the app shell around the bench.** Implement the
  laboratory-place wireframe above; collapse global chrome; make cabinet and
  journal drawers; move mode, register, locale, accessibility, and save controls
  into a stable top rail; retain a persistent compact current-goal card. DoD:
  first-time users can place a vessel, add a reagent, run an action, inspect the
  result, undo it, and find the notebook without opening the command bar; the
  bench occupies ≥60% of a 1366×768 content area; no control overlaps at 320 px
  or 200% text zoom.
  *Status 2026-08-26: first shell pass landed in the Svelte app—stable top rail,
  explicit Sandbox badge, bench-first three-surface layout, labeled cabinet and
  journal, one utility drawer replacing the wrapped button wall, and mobile
  workspace/cabinet/journal tabs. Live-engine task walkthrough and 320 px/zoom
  audit remain before completion.*
- [ ] **GUI-071 — Ship the bright scientific-workshop token system.** Replace
  component-local colors with the tokens above; add light/dark/high-contrast
  themes, visible focus, semantic icon/shape pairings, reduced motion, and
  screenshot stories for all three registers. DoD: automated WCAG contrast
  checks for text/control pairs, color-blind snapshots for hazards and phases,
  and zero hard-coded UI colors outside the token/theme files. Engine-computed
  material colors are explicitly exempt and tested against their surrounds.
  *Status 2026-08-26: semantic tokens, bright-default light theme, explicit
  dark/pro and high-contrast themes, system typography, focus treatment,
  reduced-motion clamp, and refreshed shell/bench/cabinet/journal/command styles
  landed. Automated contrast and visual-regression gates remain.*
- [ ] **GUI-072 — Equipment cabinet v1.** Replace the flat apparatus palette
  and reagent shelf with one searchable, categorized supply room. Cards and
  detail sheets render catalog metadata; drag/tap places a real scene object;
  filters include compatible-with-selection, owned/available (Story), and all
  (Sandbox). Saved kits can repopulate an empty bench. DoD: every GUI-033 item
  has a visible catalog disposition (modeled, decorative, locked, or not yet
  available), never a dead card.
  *Status 2026-08-26: cabinet shell landed with separate Reagents and Equipment
  tabs; the equipment tab is now a visual instrument wall with illustrated,
  categorized cards for the burette, every parameterized apparatus, reaction
  studio, column train, and two-vessel transfer/separation tools. Choosing a
  card deploys it to the active work area. Full item metadata, compatibility
  filters, Story availability states, and saved kits remain.*
  *Core-control slice in progress 2026-08-27: reagent search matches the
  localized display name as well as canonical name, formula, and key, using
  case/diacritic-insensitive substring matching. Dispensing exposes a numeric
  amount and unit selector at every register; its initial unit/value derives
  from the selected vessel's capacity (mL for current glassware, L for large
  future vessels) rather than two fixed global buttons. Story now uses one
  persistent three-scope selector in both cabinet tabs: Mission set means
  temporary case-supplied stock, Unlocked means permanently earned stock, and
  All previews the complete catalog without bypassing locks. Sandbox omits the
  selector because its catalog is already wholly available.*
- [ ] **GUI-073 — Spatial bench and assembly grammar.** Give work zones,
  apparatus footprints, ports, sockets, layering, collision rules, and
  keyboard-equivalent placement to the scene. Valid destinations preview before
  commit; invalid drops explain why and leave state untouched. Arrangement is
  persisted and replayable but chemistry state remains engine-owned.
  *Status 2026-08-26: the visual interaction foundation landed—named
  Prepare/React/Analyse zones are now real vessel destinations: drag a vessel
  between them or use the selected vessel's translated left/right controls.
  Valid destinations preview before drop, the arrangement survives reload,
  and visible input/output ports anchor engine-confirmed connected rigs.
  Fine-grained free placement, collision/capacity rules, and arrangement
  replay inside exported `.lab` files remain.*
  *Direction correction 2026-08-27: the three named zones are workflow hints,
  not partitions. The decorative overhead rail/lights/sockets are removed to
  reclaim work area; Sandbox defaults to one continuous surface and users can
  persistently show or hide the guides in either mode. Next, replace zone-only
  movement with fine placement and make real services/stands explicit objects.*
  *Free-placement slice in progress 2026-08-27: the continuous surface now owns
  normalized x/y coordinates rather than three hidden flex columns. Mouse,
  pen, and touch use one Pointer Events drag path; compact four-way controls
  provide the keyboard-equivalent move; coordinates persist separately per
  mode. Version-1 zone-only saves migrate to their former zone centre, and the
  optional Prepare/React/Analyse overlay derives its counts from x without
  constraining placement. Collision/footprint rules and exported `.lab`
  arrangement replay remain.*
- [ ] **GUI-074 — Direct manipulation pass.** Implement contextual object
  selection and the highest-frequency physical gestures: place/remove, pour,
  dose, stir, heat/cool, seal/open, connect, insert/read probe, and start/stop.
  Each gesture must map through the affordance manifest to one existing engine
  operator and echo that operator to the notebook. No generic form is the main
  path for these verbs.
  *Status 2026-08-26: selected vessels now keep the user on the bench and open a
  local action dock backed by real grammar lines for stir, heat, cool, visual
  observation, temperature, pH, and seal/open; Details and More tools provide
  explicit routes to the journal and equipment cabinet. A direct Pour action
  now keeps the selected source fixed, exposes fraction choices, highlights
  valid target vessels, and compiles the result through the shared two-vessel
  grammar helper. Engine-confirmed transfers now draw a spatial stream between
  the actual source and receiver, scaled by the transferred fraction;
  filter, still, drain, and galvanic-cell operations now assemble transient
  connected rigs between their engine-reported source and target. Dedicated
  start/stop controls remain.*
  *Immediate remainder: selected-object controls must be icon-sized previous /
  next / remove controls. Removal is an engine-owned, undoable operation with
  an explicit contents-disposal decision; the UI must not silently delete
  chemistry state.*
  *Core-control slice in progress 2026-08-27: previous/next are compact icon
  controls and an engine-owned `remove vN` operator removes only empty vessels,
  never the last vessel, and participates in undo/replay. Apparatus deployment
  now replaces the generic vessel-action dock so unrelated vessel actions do
  not visually compete with the apparatus controls.*

- [ ] **GUI-081 — Learner-authored laboratory journal.** Keep engine output,
  mission evidence, and learner notes visibly distinct. Users can add, edit,
  and remove timestamped text notes; notes persist with the mode-specific lab,
  survive reload, and export in Markdown beside measurements and charts. Notes
  never become engine evidence merely because they are in the same journal.
  *Journal-note slice in progress 2026-08-27: add, edit, remove, and persist
  timestamped notes and include them in notebook export. Chronological replay
  placement remains.*
  *Observation-first slice in progress 2026-08-27: the journal now defaults to
  chemical observations and evidence instead of interleaving every operator
  command. A persistent Full trace switch reveals the replayable commands and
  their count. Common computed additions, dissolution, stirring, grinding,
  centrifuging, settling, and vessel-state prose is localized at the UI
  boundary and changes live with the locale; complete structured localization
  of the remaining long-tail engine events is still required.*

- [ ] **GUI-082 — Explorable laboratory room.** Build a colourful 2.5-D room
  around the continuous bench: cupboards and drawers are storage homes; racks,
  holders, utilities, sinks, and waste stations are interactive objects; the
  periodic table and scenario posters can be clicked and zoomed for relevant
  information. Rooms may change with context without changing chemistry state.
  Use warm, saturated workshop accents and object silhouettes inspired by good
  school-lab interfaces, while retaining professional density, dark/high-
  contrast themes, keyboard access, and no always-on mascot obstruction.
  *Interactive-wall slice in progress 2026-08-27: the bench backdrop now has
  three useful, keyboard-accessible destinations rather than decorative bars:
  a shelf-connected periodic table, an instrument cabinet, and a colourful
  zoomable safety station with concise real-lab rules and an explicit
  simulation boundary. Workflow-zone guides remain a separate persisted
  control. Cupboards, utilities, contextual scenario posters, room variants,
  and true free spatial placement remain.*

- [ ] **GUI-083 — Physical apparatus and computed motion.** Promote apparatus
  from forms/tool verbs to placeable assemblies with visible controls and
  engine-owned operating state. First families:
  - retort stands, bosses, clamps, rings, holders, hoses and cables whose
    connection graph constrains what can be operated;
  - magnetic stirrer/hotplate with power or target temperature, RPM, stir-bar
    coupling, ramp time, and start/stop;
  - mini centrifuge with rotor slots, tube compatibility, balance/imbalance,
    RPM, duration, acceleration, coast-down, lid interlock, and separation;
  - thermometer/pH probes, balances, burners, baths, cooling/freezing devices,
    filtration and distillation rigs at bench scale.
  Motion is sampled from computed state: stir vortex from RPM, viscosity and
  fill; heat shimmer/boiling/flame from power, temperature, phase and reaction
  energy; centrifuge blur and settling from angular speed, radius, particle
  size/density and medium viscosity. Reduced-motion changes presentation, not
  elapsed process or results. No canned loop may imply an effect the solver did
  not produce.
  *Initial placement correction 2026-08-27: grinding renders a standalone
  mortar and pestle beside its target vessel, with work-state motion, rather
  than drawing a mortar inside the vessel. The general assembly/footprint
  system and computed grind state remain.*
  *Magnetic-stirrer slice in progress 2026-08-27: the equipment wall exposes
  RPM and duration, the public grammar carries both, and the engine computes
  25 mm stir-bar tip speed. The bench draws the plate, rotating bar, and
  vortex with speed scaled from that emitted physical value. The companion
  hotplate exposes power and duration and compiles their product to the
  engine's delivered energy; engine-computed temperature then controls the
  persistent vessel heat presentation. Closed-loop target-temperature control
  and persistent start/stop state remain.*
  *Computed-mortar slice in progress 2026-08-27: reagent additions now retain
  material-lot provenance; grinding persists the requested mean particle
  diameter and emits spherical-particle surface area from actual solid moles,
  molar mass, and registry density. Mortar motion scales from that emitted
  area. Heterogeneous kinetic rate coupling remains explicitly false until a
  rate law consumes the surface-area state.*
  *Computed-centrifuge slice in progress 2026-08-27: the equipment wall now
  exposes RPM, duration, and rotor radius; the core derives angular speed and
  RCF, then applies Stokes settling per solid from particle size, registry
  density, computed liquid density, temperature-dependent water viscosity,
  and tube path length. The standalone rotor's speed follows emitted RCF.
  The follow-up state slice persists each tracked lot's suspended fraction:
  centrifuging transfers the computed portion into the visible bottom deposit,
  while magnetic stirring resuspends non-metal solids by computed bar-tip
  travel (speed × duration). Ordinary `wait` applies the same Stokes travel
  model at 1 g, so a suspension can settle, be resuspended, and be separated
  faster in the centrifuge. The centrifuge form preloads an opposing tube
  from engine-reported sample mass, visualizes imbalance, disables Start
  beyond 0.10 g, and the core independently refuses unsafe commands. Tube
  tare is declared excluded because equal tubes cancel it.*
  *Control-and-duration slice in progress 2026-08-27: stir, heat, and cool open
  compact apparatus controls with continuous numeric input and physical dials
  instead of firing unexplained fixed doses. Heating accepts power and time;
  cooling exposes bath temperature; stirring accepts RPM and time. Their bench
  animations use the requested or emitted duration and stop reactively rather
  than leaving an infinite decorative loop behind. Apparatus controls dock
  with the bench and replace unrelated vessel actions while deployed.*

- [ ] **GUI-084 — Mixing and transport state.** Replace bare `stir vN` with a
  parameterized, time-bearing operation and authoritative mixing state. Model
  the effects that matter to existing solvers first: concentration
  homogenisation, solid suspension/settling, surface-area/mass-transfer limits,
  heat transfer, and rate coupling. Until an effect is modeled, the UI may show
  the apparatus turning but must label the scientific boundary; it must not
  claim that stirring changed a reaction or solubility equilibrium. Repeated
  additions must update both inventory and visible amount, and the journal must
  report current totals rather than confusing a new dose with total material.
  *Feedback slice in progress 2026-08-27: authoritative inventory already
  accumulated repeated doses; repeated-add events now also carry and render the
  post-dose total. Stirring now carries RPM/duration and exposes explicitly
  uncoupled rate physics; scaling visible solid volume and coupling mixing to
  transport/rates remain part of this item.*
- [ ] **GUI-075 — Observe five users before adding campaign breadth.** Test with
  at least two children/novices, one teacher, and two experienced science users;
  use tasks, not preference questions. Record time-to-first-result, wrong turns,
  drawer/gesture discovery, mode comprehension, and register usage. Fix P0/P1
  findings and document the resulting interaction changes before GUI-076.
- [x] **GUI-076 — World/home shell and separated saves.** Add the start screen,
  Story/Sandbox doors, persistent lab identity, mode badge, safe switching, and
  versioned independent saves. DoD: starting or resetting Sandbox cannot mutate
  Story progress, and a newly installed catalog item appears immediately in
  Sandbox.
  *Shipped 2026-08-26: the first-run Research Campus is now the game's home,
  with distinct Discovery Wing (Story), Open Bench (Sandbox), Mission Board,
  and Research Library destinations plus a renameable persistent lab identity.
  Story and Sandbox scope their command log, snapshots, Codex progress, and
  bench arrangement independently; a one-time, non-destructive migration puts
  pre-mode installs into Sandbox. The live mode badge opens the campus, mission
  launches cross the Story door safely, and all new visible/ARIA copy ships in
  English and German.*
- [ ] **GUI-077 — Story progression and research map.** Render locations,
  contacts, lab areas, and equipment families as an explorable map—not a linear
  level list. Unlocks are previewable with understandable prerequisites; at
  least three useful missions are available whenever the chapter permits.
  *Progression-map slice shipped 2026-08-26: the flat mission grid is now a
  five-district campus map. The opening Discovery Hall offers four real shipped
  investigations; one completed mission opens both Matter Gardens and Energy
  Yard, so the story branches instead of becoming a corridor. Finishing the
  final engine-backed lesson command persists its stable file id, completed
  missions and districts are visibly marked, and every locked district states
  its exact requirement. Progress uses the existing Story-scoped storage;
  leaving a mission does not complete it and Sandbox progress cannot unlock the
  campus. The Electron Works, Systems Dock, contacts, equipment-family rewards,
  and engine-evaluated outcome transactions remain in this item.*
- [ ] **GUI-078 — Mission journal and in-world delivery.** Evolve QuestBar into
  active-mission cards, evidence ledger, optional hints, messages, and result
  debriefs. Dialogue pauses only itself, never silently the chemistry. All copy,
  including generated parameters and ARIA text, ships in English and German.
  *Journal slice shipped 2026-08-26: the active overlay now states a human
  objective beside the exact operator instruction, offers optional procedural
  hints for every verb used by the 27 shipped lessons, and expands into a
  mission-only ledger of engine-rendered observations, measurements, hazards,
  and charts. Successful completion leaves a non-blocking debrief over the live
  bench with evidence review, persistent completion totals, and newly opened
  campus routes; replay and first-discovery outcomes are distinct. A rejected
  operator cannot advance the objective or trigger the debrief. In-world
  contacts/messages and typed multi-objective evidence remain in this item.
  Copy direction is Mission Control: case files, briefings, observations, and
  evidence—not teacher-facing “showpiece experiment” language.*
- [x] **GUI-079 — Progression-aware catalog.** Story availability and quantity
  constraints decorate the shared apparatus/reagent catalog; Sandbox bypasses
  them. Rewards animate once, explain what changed, and offer "place on bench."
  No mystery currencies, daily streaks, loot boxes, or real-money storefront.
  *Access-policy slice shipped 2026-08-26: Story now previews instrument and
  reagent availability in the existing cabinet rather than a separate shop.
  Permanent apparatus families unlock at investigation milestones; locked cards
  state the exact completed-mission requirement, while Sandbox bypasses every
  gate. The stockroom starts with common materials, expands according to hazard
  handling progression, and temporarily loans any otherwise-locked substance
  required by the active mission kit. First completions reveal one permanent
  instrument reward in the debrief and can place it directly on the live bench.
  This has no currency, streak, purchase, or randomized reward. The same
  persistent Mission set / Unlocked / All scope selector now controls both
  reagent and equipment tabs; mission equipment is loaned under the same rule
  as mission reagents, and All previews locks instead of bypassing them.*
  *Finite-stock slice shipped 2026-08-26: unlocked Story substances now expose
  a persistent count of labelled dispenses. Only an engine-accepted `add` or
  `titrate` transaction consumes one; rejection consumes nothing, and bench
  undo/reset does not pretend used material returned to its bottle. The exact
  physical amount remains visible in the engine command and notebook, avoiding
  invented cross-unit mass/volume conversions. Active investigations supply
  their entire reagent kit independently of permanent stock, and each first
  discovery replenishes the permanent cabinet with an explicit debrief notice.
  Sandbox remains unlimited. This closes GUI-079.*
- [ ] **GUI-080 — First vertical slice: the contaminated sample.** Ship one
  compact district/lab with onboarding, free bench time, three concurrent
  missions, one optional discovery, a material cabinet, a permanent instrument
  unlock, and a debrief. At least one mission must accept two materially
  different valid solutions. Completion is engine-evaluated; closing/reopening,
  switching register, switching locale, and visiting Sandbox all preserve the
  right state.
  *Case-board slice shipped 2026-08-26: Discovery Hall is now presented as
  "The contaminated sample," with a one-time persisted briefing from a campus
  chemist, an animated physical sample, three engine-backed core leads available
  concurrently in any order, and the existing safety investigation called out
  as an optional discovery. Core and optional completion derive from the stable
  mission ids already stored in the Story save; returning to the board shows
  secured evidence, an active lead, and 0/3–3/3 case progress without touching
  Sandbox. The desktop and mobile board, onboarding, statuses, actions, and ARIA
  labels ship in English and German.*
  *Outcome-contract slice shipped 2026-08-26: “Trace the mineral
  contamination” now states a result rather than exposing the `.lab` recipe.
  Its evidence check is secured only by the engine's typed `precipitated:AgCl`
  event at or above the observable-moles threshold; direct AgCl placement, a
  different precipitate, and sub-visible traces do not pass. Both NaCl and KCl
  routes are supplied and complete the same contract, the board identifies the
  lead as solver-assessed, and the live goal, hint, evidence state, debrief,
  ARIA copy, and German translations share the existing mission UI. The other
  two core leads and optional safety audit remain procedural, so GUI-080 stays
  open until their outcomes and case-level transaction are typed.*
  *Thermal-outcome slice shipped 2026-08-26: “Establish the thermal baseline”
  is now an open investigation with no exposed recipe. A new graphical mixer
  turns three vessel taps into the public `mix` operation, with selectable
  source fractions and English/German guidance. The engine's typed `mixed`
  event now records both source temperatures and the computed receiver
  temperature. Assessment requires two materially different source
  temperatures, meaningful contributions from both streams, and a result
  strictly between them; heating alone, isothermal mixing, endpoint results,
  and trace contributions cannot pass. One core separation lead and the
  optional safety audit remain procedural, so GUI-080 stays open.*

### Phase G3 — Desktop

- [ ] **GUI-030 — TauriHost.** Native core behind the protocol; conformance
  suite green on Win/macOS/Linux.
- [ ] **GUI-031 — Desktop shell.** `.lab` file association, native
  open/save, menu/shortcuts, offline pack manager (signed manifests per
  LIC-009).
- [ ] **GUI-032 — Payload audit.** SBOM + notices for the Tauri bundles
  (extends LIC-008/LIC-012).

### Phase G4 — Mobile

- [ ] **GUI-040 — Android first** (the school-device population), then iOS:
  Tauri v2 mobile builds, touch polish pass, one-hand lv1 audit,
  WebView-version floor decided from telemetry-free field testing.
- [ ] **GUI-041 — Store payloads.** Official binaries under the Section 7
  permission (LIC-001/002 already resolved); store-review dry run.
- [ ] **GUI-042 — Fallback decision point.** If Tauri mobile misses the
  budgets on target hardware, wrap the identical UI in a plain
  WKWebView/WebView shell; the protocol makes this a shell swap.

### Phase G5 — Expert surfaces

- [ ] **GUI-050 — Studies UI** over CAP-2 (sweeps) with CAP-3 rendering.
- [ ] **GUI-051 — Diagrams** (CAP-4 predominance/Pourbaix) and **Monte
  Carlo bands** (CAP-8) in the chart renderer.
- [ ] **GUI-052 — Provenance drawer** rendering the R0 capability/validity
  reports and the property-resolution ladder rung per number.

Continuous, all phases: a11y audit per surface; perf budgets in CI on a
throttled target profile; golden scene/DOM tests over the lesson corpus;
every new dependency through the LIC checklist before first import.

## Non-goals

- Free-camera 3-D, photorealism, or adopting a game engine. The product **is a
  game**; its visual implementation is an accessible 2.5-D DOM/SVG/Canvas
  laboratory because the cross-section teaches more and keeps every control
  keyboard-, touch-, and screen-reader-operable.
- Accounts, cloud sync, classroom management, telemetry.
- A separate kids' edition — the dial is the product; forking the audience
  forks the codebase and betrays the register idea.
- Native per-platform UI rewrites (SwiftUI/Compose) — revisit only if a
  platform's webview floor proves untenable *and* the platform matters
  commercially.
- Investing in the TUI beyond debugging parity — it remains a developer
  surface.

### The realism bar (2026-08-25 addendum, owner-directed)

The reference quality is the established national school-lab platforms'
simulation pages: rendered instruments, task animations, effects whose
LOOK carries the MAGNITUDE. Our standing rule keeps us honest where
those platforms are theatrical: every visual quantity below traces to a
computed number — realism here means *rendering the simulation*, never
decorating it.

- [x] **GUI-058 — Liquid layers in the Scene (engine + client).** *Shipped 2026-08-25 (PR #30).* The
  engine already computes liquid–liquid equilibrium (water–hexane LLE,
  `solve::layered_pair`) but Scene JSON collapses everything into one
  liquid block. Additive `layers: [SceneLayer]` on SceneVessel — per
  layer: species, name, volume_l (from moles × molar volume via the
  registry's density), srgb, colour_word, stacking order by density.
  Client draws stacked fills with distinct menisci. One layer = today's
  render, so the change is invisible until chemistry splits. DoD:
  hexane-on-water renders two layers with correct proportions; scene
  conformance checks the layer shape; single-phase vessels byte-identical.
- [ ] **GUI-059 — Effect magnitudes.** Events carry amounts; visuals must
  scale by them: bubble count/rate from moles of gas evolved, flame size
  from energy/rate and COLOUR from the FlameTest event's computed colour
  word, stir vigour from the operator, precipitate fall density from
  moles. DoD: doubling the chemistry visibly doubles the effect; every
  scale factor names its event field in a comment.
  *Status 2026-08-26: the typed-event mapper now covers gas, precipitate,
  evaporation/distillation, electrolysis, mixing/dilution, transfer, thermal
  change, phase change, heat of mixing, plating, flame tests, and glass burst.
  Transfer fraction sets stream width/speed; mixing fractions set vortex size
  and stir speed; temperature/delta-T set heat shimmer and frost coverage;
  distilled/evolved/precipitated/electrolysed moles set effect density; flame
  colour is the engine's named result; `burst.at_pa / burst.rating_pa` sets the
  pressure-wave and shard distance. A hazard warning alone never explodes.
  Reaction energy from the CEA/NASA enthalpy solve now travels on ignition
  events and linearly controls flame size and flicker speed; an unquantified
  ignition uses a restrained fallback rather than maximum drama. Remaining
  DoD: screenshot regression cases.*
- [x] **GUI-065 — Fluid dynamics as the transport layer (supersedes
  GUI-060's scripted plume).** Owner question answered 2026-08-25:
  SPH-Lagrangian, VOF-Eulerian, or both → BOTH, split by phenomenon,
  never touching the chemistry.
  **065a, Eulerian (first):** a per-active-vessel stable-fluids grid
  (~64×96; semi-Lagrangian advection, Jacobi pressure projection,
  SDF boundary from the glass `inner` path) advecting per-species
  CONCENTRATION fields — dye plumes, stirring vortices, miscible
  diffusion, and density-buoyant layer separation all emerge; full
  VOF/PLIC rejected as overkill because the REST interface truth is
  GUI-058's static layer render, cross-faded to on settle.
  **065b, Lagrangian (second):** particles only above/at the surface —
  pour stream breaking into droplets, burette drips, splash ejecta —
  handing mass+momentum into the grid on entry (mini-FLIP handoff).
  Ground rules: plain TypeScript + typed arrays (no deps; wasm only if
  profiling demands, and never inside the engine module); sim runs in
  activity windows then freezes to the static render;
  prefers-reduced-motion skips straight to settled. THE HONESTY GATE,
  unit-tested: settled concentrations must converge to the engine's
  layer volumes and colours — the sim animates TOWARD the solver's
  answer, never past it. DoD 065a: KMnO4 into water shows violet
  tendrils dispersing to exactly the scene srgb; hexane onto water
  visibly separates into GUI-058's two layers; stir drives a vortex
  that decays; all kernels/projection/settle covered by vitest.
- [x] **GUI-061 — Volume-true fills.** *Shipped 2026-08-25 (kero-basic, PR #36).* Fill height must come from the
  vessel kind's real capacity and geometry (a conical flask's height vs
  volume is not linear). Per-kind capacity_ml + a volume→height profile;
  additions raise the level by exactly what was added. DoD: 50 mL into a
  100 mL beaker reads half; the same 50 mL in the flask reads correctly
  non-linear; cylinder graduations line up with real volumes.
- [ ] **GUI-062 — Instruments on the bench.** The burette clamps OVER the
  vessel on the bench (drawn, with stopcock and falling drops during
  titration), thermometer and pH probe render in-vessel when measuring,
  the still connects two vessels visibly. Portraits (ToolIcon) grow into
  bench-scale drawings as each tool earns it.
  *Status 2026-08-26: the cabinet's burette, wash bottle, evaporating dish and
  hotplate, electrodes and supply, mortar and pestle, wavelength lamp, piston
  lid/gauge, and carrier-gas line now deploy as SVG apparatus around the active
  vessel. Live form values drive wavelength colour, current pulse rate, and
  pressure gauge position; running state drives tool motion. Filter, still,
  drain, and cell events connect the actual source/receiver pair across the
  bench through visible vessel ports. Vessel work-zone arrangement persists
  across reload. The mortar and mini-centrifuge are now freestanding,
  target-labelled workstation cards placed in the clearest nearby bench space;
  they no longer render as contents inside the selected vessel. Remaining:
  user-positionable instrument stations and bench-scale analytical instruments
  beyond thermometer/pH.*
- [x] **GUI-063 — In-experiment visual shelves.** *Shipped 2026-08-25 (kero-basic, PR #36).* Lessons and codex
  experiments present their kit as a RENDERED shelf strip (SpeciesChip
  visuals, tap-to-add) directly in the LessonBar / experiment page —
  the pick-what-you-need surface the reference platforms open with.
- [x] **GUI-064 — Animation of running tasks.** *Shipped 2026-08-25: chart self-draw (#35), titration playback (#37), distillation + transport pacing (#45) — one clamped, cancellable, reduced-motion-honest scheduler.* Multi-step operations
  (titrate, distil, electrolyse, transport) animate over their duration
  instead of jumping to the result: the burette's meniscus falls per
  increment, the still's receiver fills, electrode gas accumulates.
  Driven by the per-step data the engine already returns (titration
  curve points, transported fractions).

- [x] **GUI-074 — Bench focus controls.** *Shipped 2026-08-27.* On wide
  screens, the material cabinet and laboratory journal collapse independently
  to narrow edge rails; opening tools, details, or a target panel expands the
  relevant rail automatically. Choices persist separately in Story and
  Sandbox. The existing three-pane tab bar remains the touch-first navigation
  on narrow screens.

- [x] **GUI-075 — Unobstructed vessel controls.** *Shipped 2026-08-27.*
  Free-placement nudge and removal controls sit as one compact, readable row
  on the selected card's upper corner. They no longer cover the vessel name,
  liquid, or action dock; direct pointer/touch dragging remains the primary
  spatial interaction.

Split: GUI-058 + 060 + 064 are architecture/engine-coupled (fable);
GUI-061 + 063 are self-contained client work (kero-basic);
GUI-059 + 062 are client work gated on no engine change (kero1, after
the KLU fix). Magnitude scaling rules (GUI-059) and layer rendering
(GUI-058) meet in Vessel.svelte — coordinate before touching it in
parallel.


### Shipped addendum (2026-08-25, realism + depth day)
- GUI-065a/b/c complete: MAC stable-fluids core (emergent Rayleigh-
  Taylor pinned), pour with a ledger-exact splash handoff, true-glass
  wall masks (canvas-free rasterizer) + frame governor. The honesty
  gate throughout: sims relax to exactly the engine's layers.
- GUI-066 quests: engine-evaluated (observe/answer in the protocol),
  17 quests exported beside the payload, QuestBar + nudge/claim cards.
- GUI-067: instant restore (snapshot-token autosave, triple-fallback).
- DATA-010: load_pack end to end — the registry is open-ended; a
  hash-verified .pack adds species to shelf AND chemistry at runtime;
  drift-pinned runtime join; packs/ shipped with hashed manifest.
- WEB-003 inventory in hello; PROTOCOL load_pack row done.

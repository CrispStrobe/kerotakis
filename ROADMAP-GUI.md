# Kerotakis — GUI roadmap: one bench, one dial, five platforms

> Finished work is not listed here. What landed, and what it taught us, is in
> [HISTORY.md](HISTORY.md). Task numbers are never renumbered and never reused.

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
- *Relation-toolbox audit shipped 2026-08-27:* canonical relation ids and
  argument symbols remain language-neutral for computation, while relation
  names, human-readable field labels, units, errors, register explanations,
  and provenance now cross the German presentation boundary. Inputs name the
  physical quantity prominently and retain the exact solver symbol beneath it.
- *Notebook-export audit shipped 2026-08-27:* Markdown exports now use the same
  locale boundary as the live journal for engine observations, hazards,
  refusals, chart titles, axes, units, and provenance. User-entered commands
  and notes remain verbatim. Rare particle populations now localize their
  species names instead of leaking canonical English labels.
- *Localized-discovery audit shipped 2026-08-27:* experiment search now indexes
  both canonical codex data and the current locale's visible title, summary,
  concepts, apparatus, models, and register prose with case/accent-insensitive
  substring matching. German queries can discover German-labelled experiments
  without sacrificing formula or stable-id search; curriculum stages and
  observed-event verdicts are localized as well.
- [ ] **Codex content localization.** The current golden export contains 103
  investigations whose long-form registers, prediction questions, diagnoses,
  and provenance are predominantly authored in English. Routing through
  `t()` and localized search is infrastructure, not German coverage. Add an
  explicit locale sidecar/schema plus an export-time English/German completeness
  gate; do not claim full-app i18n until that gate passes the production codex.

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
- [ ] **GUI-012 — Shelf v1.** Registry-driven, searchable, drag/tap-to-add,
  register-aware amount picker (pinch/cup ↔ g/mol/mL).
  *Status 2026-08-24: tap-to-add, drag-to-vessel, registry-fed search and
  register-aware quick amounts landed. Open: hazard chips, appearance
  swatches, and SCALE — the flat list stops working past ~150 species and
  needs phase/family grouping with search staying primary. The species
  COUNT is tranche-gated data work: every entry arrives with molar wiring,
  appearance, safety row, provenance and InChI identity, because a shelf of
  hundreds of unverified names is the lookup-table failure this project
  exists to avoid.*
- [ ] **GUI-013 — Feed v1.** Event stream at current register with command
  echo; this is also the a11y acceptance gate: run a full lesson from
  keyboard + screen reader.
- [ ] **GUI-014 — Inspector v1 + command bar.** Register-dependent detail;
  ⌘K bar with completion from the grammar + parse-only validation.
  *Status 2026-08-24: Inspector v1 landed (see `HISTORY.md`). Open:
  completion + parse-only validation in the bar.*
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

### The sandbox completeness invariant

**Every registry species, every apparatus, every engine verb is reachable
from the GUI — in sandbox mode, without the command bar.** The engine
already exposes ~25 verbs and the full registry; the gap is graphical
affordance, and it is checkable, so it becomes an invariant with a test
rather than an aspiration:

### The codex is the content engine (apply it, then expand it)

Each codex entry already carries a runnable `setup`, checkable `expect`
predictions, per-register prose, an `apparatus` list ("drives what a UI
puts on the bench"), concept/prerequisite edges into 189 defined concepts,
calculation and model taxonomies, and curriculum placements. The GUI has
been ignoring all of it:

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

  *Correction 2026-09-05 — the Bench tab was the missing one, and the
  two catalogues are now ONE module.* The run was never fake: it always
  went through `session.submit()`, so the chemistry, the feed, the log
  and undo were all real. What was missing was that anyone could SEE it.
  Every line of a script was fired in a single tick, so ten commands'
  worth of animation collapsed into one frame, and a full-screen modal
  sat over the stage while it happened. A learner opening
  "Löslichkeitsprodukt von Silberchlorid" got a verdict and no
  experiment, which is exactly the text-only genre GUI-054 was written
  to beat.

  The walk moved out of the component into `lib/catalogRunner.ts`, which
  paces the steps and announces each line before it goes; the panel docks
  to a one-line strip at the foot of the screen while it runs, and the
  scrim stops taking pointer events, so the bench is watched rather than
  covered. A bench that already holds the learner's work is ASKED about
  first — clear it, run beside it in fresh glassware (the script's `vN`
  tokens shifted past what is there), or run on it as it is — and is
  never silently wiped.

  `ExperimentCatalog.svelte` and `KidsCatalog.svelte` became one
  `Catalog.svelte` in the same change, because the Kids tier's cards were
  offering a different meaning of "run this" than the experiment tier's,
  and reconciling two runners is how they drift apart again. Tier is now
  presentation — Kids cards stay wordier and bigger-buttoned, and gain a
  direct "run it on the bench" wherever the KIDS entry names a codex
  entry we actually ship — over one search box, one progress record
  (`session.completedExperiments`), one close affordance and one runner.
  `kidsOpen`/`catalogOpen` survive as the tier selector so every existing
  door (home screen, story map, periodic table, concept map) still opens
  the panel on the tier it meant.
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
  *Responsive-quality gate in progress 2026-08-28:* the real built PWA now
  runs in desktop, German desktop and phone viewports in CI. It fails on
  page-level overflow, overlapping desktop panes, unnamed controls, duplicate
  ids, undersized primary vessel actions or phone tabs, and live bench motion
  under `prefers-reduced-motion`.
  *Contrast gate landed 2026-08-28:* every normal and muted text token is now
  checked against every workspace surface, every semantic accent has a
  theme-specific foreground, and light, dark/pro, and high-contrast palettes
  must all meet WCAG AA in a fast CI job. Dialog overlays, room environments,
  and periodic-table category colours now also belong to the shared theme; the
  gate rejects new component-local UI colours except in named physical scene
  renderers. Screenshot and color-blind diffs remain.*
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
  *Instrument-search slice shipped 2026-08-28:* the equipment wall now has
  the same case/diacritic-insensitive substring search as the reagent cabinet.
  It searches localized card names and descriptions as well as canonical
  apparatus verbs and English source vocabulary, keeps Story availability
  scope intact, and gives an explicit empty result instead of a blank wall.*
  *Measurement-catalog slice shipped 2026-08-28:* all twelve released
  measurement instruments now live in the same searchable wall instead of
  being discoverable only through vessel details. Each card names its purpose,
  targets the active vessel, compiles to the existing public operator, and
  participates in Mission set / Unlocked / All access. The compact inspector
  tray consumes the same registry, preventing the two surfaces from drifting.*
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
  arrangement replay remain. *Footprint slice shipped 2026-08-27:* vessel and
  freestanding-instrument destinations are checked before commit. Occupied
  destinations preview in red, explain the conflict through the live region,
  and preserve the previous arrangement for pointer, touch, and keyboard moves.*
  *Free-slot slice shipped 2026-08-27: engine-confirmed new glassware now
  selects itself and occupies the first stable open footprint, accounting for
  persisted instrument stations. Manually occupied legacy defaults no longer
  cause newly created vessels to appear inside existing objects.*
  *Arrangement-replay slice shipped 2026-08-27:* saved `.lab` files carry the
  normalized vessel and freestanding-instrument layout in a versioned comment
  that older CLI/core readers safely ignore. The web app validates and restores
  it after replay; legacy or malformed files continue with normal stable
  placement rather than disturbing chemistry import.
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
  not visually compete with the apparatus controls. *Safe-removal slice shipped
  2026-08-27:* the compact × opens an explicit vessel decision instead of
  blindly issuing a command. Empty removal is confirmed and undoable; occupied
  vessels offer a 100% liquid transfer or the waste station, and no material is
  silently deleted. The final workspace vessel remains protected.*
  *Destination-label slice shipped 2026-08-27: the selected-vessel dock no
  longer ends in ambiguous Details / More tools buttons. Explicit Measurement
  tools and Equipment cabinet routes, with distinct icons and localized
  tooltips, now name the panels they actually open.*

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
  *Vessel-context slice shipped 2026-08-27: engine lines carry a compact vessel
  chip instead of burying their target in prose. A persistent Whole lab /
  selected vessel scope lets users follow one sample without hiding global
  safety notices, charts, or their own notes.*

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
  control. *Interactive-wall slice shipped 2026-08-27:* an active Story
  investigation now appears as a colourful in-world mission briefing with its
  live step/evidence count; opening it reveals the same engine-backed mission
  journal instead of a duplicate panel. *Room-identity slice shipped
  2026-08-27:* Discovery Studio, Research Laboratory, and Orbital Laboratory
  give the same live chemistry three colourful, persisted environments; the
  choice is explicitly visual and never mutates vessels or evidence.
  *Utility-station slice shipped 2026-08-27:* a compact wall supply point
  opens a real utility panel for the selected vessel. Water routes to the
  exact amount-aware water card, power routes to equipment, and the waste
  station states why chemical contents are never discarded silently.
  Cupboards and true free spatial placement remain.*

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
  *Assembly-graph slice in progress 2026-08-28:* every parameterized
  workstation now exposes a compact physical setup alongside its controls:
  drive/plate/bath to sample, balanced tube–rotor–tube, supply–lead–electrode,
  lamp, sealed piston, and complete carrier-gas routes. The shared typed graph
  marks missing solid samples and unsafe counterbalances without inventing a
  chemical result. Direct manipulation of individual clamps, hoses and ports
  remains.*
  *Initial placement correction 2026-08-27: grinding renders a standalone
  mortar and pestle beside its target vessel, with work-state motion, rather
  than drawing a mortar inside the vessel. The general assembly/footprint
  system and computed grind state remain.*
  *Retort-stand slice shipped 2026-08-27: deploying a burette now places a
  freestanding, movable stand, boss, clamp, graduated column, stopcock, and
  tip beside its target. A visible route identifies the receiving vessel;
  during titration the column level follows delivered/total engine playback
  and the drop moves only while the operation runs.*
  *Measurement-probe slice shipped 2026-08-27: temperature and pH actions now
  lower distinct physical probes into the selected sample and show the
  engine-backed reading on a localized digital meter. The probes remain
  transient measurement tools rather than permanent decorations.*
  *Physical-balance slice shipped 2026-08-27: a balance measurement now slides
  a scale beneath the selected vessel, settles once, and shows the exact mass
  emitted by the engine. Temperature and pH meters likewise read their scalar
  measurement events directly rather than reconstructing the displayed value.*
  *Pressure-gauge slice shipped 2026-08-27: measuring headspace pressure now
  connects a dial to the selected vessel. Its localized digital value and
  needle angle come from the engine-emitted kPa reading, with a distinct upper
  warning arc and a reduced-motion-stable endpoint.*
  *Volume-and-conductivity slice shipped 2026-08-27: gas-volume measurement
  connects a graduated syringe whose piston follows the engine-emitted mL
  reading. Conductivity lowers a two-electrode probe and labels the displayed
  µS/cm value as the engine's current ionic-strength estimate; signal styling
  scales logarithmically without overstating the model as a full Kohlrausch
  calculation.*
  *Calorimeter-and-UV-Vis slice shipped 2026-08-27: calorimetry surrounds the
  sample with an insulated jacket and displays signed engine enthalpy relative
  to 25 °C. UV-Vis places a cuvette in a benchtop spectrophotometer, derives
  peak wavelength from the engine observable, and attenuates the output beam
  with T = 10^-A from the emitted Beer-Lambert absorbance.*
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
  area. The peroxide/MnO₂ law now consumes this area together with suspension
  state and a bounded stir-tip-speed correction; catalyst `Ground` and
  `Stirred` events therefore expose active rate coupling. Pore/BET area,
  adsorption and diffusion remain outside the reduced model.*
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
  with the bench and replace unrelated vessel actions while deployed. The
  desktop workstation is now a non-overlapping bottom dock, so it cannot cover
  the periodic table, instrument wall, safety station, vessels, or notebook;
  its fields reflow horizontally while narrow screens retain a touch sheet.*
  *Physical-readout slice in progress 2026-08-27: control changes immediately
  expose the quantities they imply before Start — delivered or removed energy
  from power × time, the 25 mm stir-bar tip speed, and centrifuge RCF from RPM
  and rotor radius; current × time exposes charge and electron amount, while
  lamp wavelength exposes single-photon energy. These use the same formulae
  and constants as the engine; chemical outcomes remain engine-owned.*
  *Installed-state slice in progress 2026-08-27: vessel-mounted apparatus now
  keeps a compact named status lamp on its target card. Configured equipment
  reads as ready; the same computed-operation window that drives its physical
  motion changes the label and lamp to running. The accessible vessel name
  carries the identical tool and state. A short-lived reactive clock now ends
  every transient vessel window at its declared duration even when no unrelated
  UI state happens to change at that instant.*
  *Installation-lifecycle correction shipped 2026-08-28:* the installed tool,
  its stable target and its physical control values now persist per Story or
  Sandbox lab. Reloading no longer puts configured hardware away or resets its
  RPM, duration or power. Explicit Put away and removal of an orphaned target
  clear the installation; only a matching live command/event can still mark it
  running.*
  *Cooling-bath correction shipped 2026-08-27:* cooling no longer borrows
  the magnetic-hotplate silhouette. A separate coolant bath surrounds the
  vessel, shows configured heat-removal power, and only animates ice/frost
  during the bounded operation; the vessel's computed temperature remains
  the authority for persistent cold and phase visuals.
  *Flame-test correction shipped 2026-08-27:* `FlameTest` no longer makes
  the whole sample vessel appear to combust. It deploys a Bunsen burner and
  wire loop, and colours that burner flame from the engine event; genuine
  `Ignited` events retain the energy-scaled vessel flame.
  *Irradiation-honesty slice shipped 2026-08-27:* operating the lamp now emits
  a typed physical event carrying the applied wavelength and irradiance. The
  lamp colour, brightness, readouts, and bounded operating window consume that
  event rather than merely echoing its form. A localized badge explicitly says
  that light was applied while photolysis remains uncoupled; no chemical change
  or reaction animation is implied until kinetics owns that state transition.
  *Electrolysis-playback slice shipped 2026-08-27:* the deposition event now
  retains the applied current and duration alongside charge, electron amount,
  product amount, and mass. The physical supply remains energized for the
  bounded playback, its charge pulses and gas bubbles scale from that event,
  and an engine-scaled coating grows on the cathode with the computed deposited
  mass shown on the supply. Requested controls no longer masquerade as results.
  *Thermal-delivery slice shipped 2026-08-27:* heat and cooling steps now emit
  requested versus physically delivered energy. Hotplate and bath readouts and
  effect strength use the delivered value, including cooling clamped by the
  vessel's available heat. Both the apparatus and localized journal explicitly
  identify the current instantaneous-energy boundary instead of pretending the
  form's power and duration are an engine-coupled time simulation.
  *Operating-ownership correction shipped 2026-08-27:* deployed hardware no
  longer consumes the session-wide busy flag. The in-flight command must match
  both the apparatus verb and its exact target vessel before that machine can
  enter its running state; typed result events then own the bounded playback.
  Adding a reagent, measuring, or operating the same tool on another vessel can
  no longer animate an unrelated lamp, rotor, stirrer, mortar, bath, or supply.
  *Stable-installation correction shipped 2026-08-27:* selecting another vessel
  no longer teleports deployed equipment or silently rewrites its command form.
  Apparatus and burettes retain the vessel they were installed for; choosing
  the same equipment card while another vessel is selected explicitly moves
  the installation, while choosing it again on its current target puts it away.
  Removing the target vessel safely removes its orphaned hardware as well. The
  dock now also exposes a compact localized move-to-selection control whenever
  selection and installation differ, without requiring a cabinet round-trip.
  *Apparatus-i18n audit shipped 2026-08-27:* titrant and curated-reaction
  selectors now render localized display names while preserving canonical
  command values. Computed chart titles, axes, series kinds, provenance, SVG
  accessibility titles, and gas-test tooltips all pass through the same German
  presentation boundary; numeric evidence and chemical formulae remain intact.
  *Live spatial-effects correction shipped 2026-08-27:* transfers and
  between-vessel rigs now reconnect their SVG path whenever either vessel is
  dragged, and also respond to bench resizing. Their visibility and motion use
  the engine event's declared duration instead of an unrelated 3.5-second UI
  timeout, so a long computed operation neither detaches nor disappears early.
  *Reduced-motion correction shipped 2026-08-27:* the accessibility preference
  now freezes moving streams, particles, droplets, cables, and valves while
  retaining the static filter, condenser, separator, magnet, meter, and
  source-to-target route. Reduced motion no longer erases the apparatus or its
  spatial meaning.
  *Workstation-routing correction shipped 2026-08-27:* a freestanding machine
  no longer points through the centres of itself and its sample with an
  arbitrary straight UI line. Edge-anchored lifted routes avoid the object
  silhouettes, carry a stable `vN` target badge, distinguish a physical
  burette connection from a sample/workstation association, and follow both
  objects through free placement. The route animates only when that exact
  workstation is operating.
  *Target-state correction shipped 2026-08-27:* freestanding workstations now
  mark their sample vessel with the same named ready/running status as mounted
  apparatus, without drawing the machine inside the glassware. Apparatus-active
  styling is scoped at the Bench/Vessel boundary to the exact target, so a
  running centrifuge or mortar cannot make every vessel appear active.
  *Bounded-motion correction shipped 2026-08-27:* a retained result no longer
  restarts or freezes an operation after its declared playback window. Mortar
  motion, rotor spin, evaporation steam, and wash-bottle flow exist only while
  the exact computed event is active; settled pellets and numeric result
  readouts remain afterward. Reduced motion follows the same time boundary.
  *Static operating-state correction shipped 2026-08-27:* reduced-motion users
  now retain a visible steady heat plume, frost, light cone, wash jet, steam,
  charge/gas markers, and burette drop while the corresponding computed
  operation is active. Freestanding workstations also announce localized
  ready/running state in addition to their tool and target vessel.

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
  *Solid-volume slice shipped 2026-08-27:* the scene now derives each solid
  population's additive pure volume from engine-owned moles, molar mass, and
  registry density. Vessel deposits scale monotonically from that volume and
  settled fraction, with a capacity-aware perceptual magnifier for sub-pixel
  traces and the exact mL value attached to the rendered layer. Repeated solid
  doses therefore grow the deposit without an arbitrary moles-to-pixels rule;
  multi-species deposits divide that height in proportion to each population's
  settled computed volume rather than equal decorative bands. Rate coupling
  remains open.*
  *Mixing/transport evidence slice shipped 2026-08-28:* the pinned result now
  exposes engine-emitted RPM, duration and resuspended fraction for stirring,
  both source fractions for mixing, and the delivered fraction for transfer.
  It explicitly separates the physical suspension change from the still-open
  reaction-rate coupling, so visible motion is useful without claiming kinetic
  chemistry that has not landed.*
- [ ] **GUI-075 — Observe five users before adding campaign breadth.** Test with
  at least two children/novices, one teacher, and two experienced science users;
  use tasks, not preference questions. Record time-to-first-result, wrong turns,
  drawer/gesture discovery, mode comprehension, and register usage. Fix P0/P1
  findings and document the resulting interaction changes before GUI-076.
- [ ] **GUI-077 — Story progression and research map.** Render locations,
  contacts, lab areas, and equipment families as an explorable map—not a linear
  level list. Unlocks are previewable with understandable prerequisites; at
  least three useful missions are available whenever the chapter permits.
  *Progression-map slice shipped 2026-08-26 (see `HISTORY.md`). Open: the
  Electron Works, Systems Dock, contacts, equipment-family rewards, and
  engine-evaluated outcome transactions.*
- [ ] **GUI-078 — Mission journal and in-world delivery.** Evolve QuestBar into
  active-mission cards, evidence ledger, optional hints, messages, and result
  debriefs. Dialogue pauses only itself, never silently the chemistry. All copy,
  including generated parameters and ARIA text, ships in English and German.
  *Journal slice shipped 2026-08-26 (see `HISTORY.md`). Open: in-world
  contacts/messages and typed multi-objective evidence. Copy direction is
  Mission Control — case files, briefings, observations, evidence.*

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

- [ ] **GUI-059 — Effect magnitudes.** Events carry amounts; visuals must
  scale by them: bubble count/rate from moles of gas evolved, flame size
  from energy/rate and COLOUR from the FlameTest event's computed colour
  word, stir vigour from the operator, precipitate fall density from
  moles. DoD: doubling the chemistry visibly doubles the effect; every
  scale factor names its event field in a comment.
  *Status 2026-08-26: the typed-event mapper now covers gas, precipitate,
  *Status 2026-08-26: the typed-event mapper covers gas, precipitate,
  evaporation/distillation, electrolysis, mixing/dilution, transfer, thermal
  change, phase change, heat of mixing, plating, flame tests and glass burst;
  every scale factor names its event field. A hazard warning alone never
  explodes, and an unquantified ignition uses a restrained fallback rather
  than maximum drama.*
  *Fourteen apparatus slices shipped 2026-08-27 — computed transfer colour,
  direct-pour motion, physical filtration, the computed still, the separatory
  funnel, magnetic separation, gravity settling, the centrifuge, computed
  stirring, the four gas tests, safe waft, pressure control, freestanding
  evaporation and dilution, and the computed gas sweep — are recorded in
  `HISTORY.md`. Remaining DoD: screenshot regression cases.*
Split: GUI-059 (magnitude scaling rules) is the remaining client work in
this group and meets layer rendering in Vessel.svelte — coordinate before
touching it in parallel.

## Making a computed result legible (GUI-090 … GUI-097)

The engine already produces every number and classification these tasks
display. Not one of them needs solver work; they are about a result being
*read* rather than merely emitted. The feed is a scroll of prose in which
the latest answer has no more prominence than the twentieth, the reaction
class we compute is never named, and a temperature change we calculate from
enthalpy is a clause in a sentence. A bench that computes real chemistry and
then buries it reads as less capable than one that fakes forty reactions
and presents them well.

- [ ] **GUI-090 — The result card.** The newest result gets a card above the
  feed rather than another line in it. Collapsed: the reaction-class badge,
  the equation, one sentence of observation, and an affordance to expand.
  Expanded: equation, ionic equation, reactant chips, observation,
  before/after temperature, and the concept/safety note. The feed stays as
  the notebook and the transcript; the card is what you look at while you
  work. Acceptance: every field comes from the existing `step` response, no
  new engine call, and the card is a `<details>`-shaped disclosure that
  degrades to the current feed when JavaScript is off or the register is at
  lv3 machine view.
  *Slices shipped 2026-08-28 and 2026-08-30 (see `HISTORY.md`): the
  `<details>` card, badge, equation, observation, quantities, temperature
  delta, reactant chips, concept and safety notes and the lv3 drop rule
  all land without a second engine call. Only the ionic equation remains,
  and it is GUI-092's work — deliberately not faked here.*

- [ ] **GUI-092 — The ionic equation, derived.** Beside the molecular
  equation, the ionic one — built from the solved speciation rather than
  stored. This is a thing only a computing bench can do honestly: the
  spectator ions are the ones the solver actually left in solution, at the
  actual concentration, and the neutral complexes that a memorised ionic
  equation omits (AgCl(aq) beside Ag⁺ and Cl⁻) appear because they are
  present. Where speciation is unavailable, show nothing.
  *First slice landed 2026-08-29 (`kerotakis-core/src/ionic.rs`): for a
  precipitation the partner for each element of the solid is the most
  abundant dissolved species carrying it, taken from the solver's own
  species distribution, and the coefficients are solved as a linear system
  over the elements and the charge and then verified — so the spectators
  fall out by never being selected, and nothing that fails to balance is
  shown. Neutralisation rides on a new `Neutralised` event: the aqueous
  solver has been computing the extent (from the change in the solutes'
  net charge) to get the heat right and discarding the number. Carried on
  the wire as an additive `ionic` field (PROTOCOL.md), rendered at lv2 and
  above, with the spectators named at lv3. Still open: showing the neutral
  complexes beside the free ions, and any basis beyond these two — redox
  and organic steps carry no participant list yet and are not guessed at.*

- [ ] **GUI-093 — Shelf by chemical role.** Acid, base, salt, metal, oxide,
  indicator, gas. We filter by phase — aqueous/liquid/gas/solid — which is a
  physics axis while the learner is thinking on a chemistry one. Phase stays
  as a secondary filter. The role comes from the registry, not a hand list,
  and a species with no assigned role appears under "other" rather than being
  hidden. Pairs with putting the **hazard chip on the reagent tile itself**,
  so the warning arrives at choosing time rather than at pouring time —
  including the honest "unassessed" state, which must remain visually
  distinct from "safe".

- [ ] **GUI-094 — The vessel deserves the room.** One vessel, large, central,
  when only one is on the bench; the wide empty expanse around a small beaker
  is the strongest signal we send that nothing much is happening. With it,
  quick-action chips within reach of the vessel for the two or three things
  every experiment needs — water, heat, and the reagent last used — so the
  common path does not cross the whole screen.

- [ ] **GUI-098 — Optional WebGPU presentation tier (BRD-094).** Keep the
  authoritative engine and the shipped deterministic `fluidScene` unchanged.
  Add a renderer-local capability boundary that may select a project-owned
  WGSL effect only when WebGPU exists, the effect has been explicitly enabled,
  motion is allowed and execution is neither headless nor backgrounded. The
  accelerator consumes bounded Scene/Event data and returns no chemistry,
  transfer, phase or temperature state. It performs no per-frame Tauri IPC or
  GPU readback; device loss immediately returns to the lightweight renderer.
  First implementation candidate must name one missing observable (smoke,
  flame, foam or a genuinely 3-D pour) and beat BRD-072's baseline on the
  low-end target. DoD: deterministic selection tests, WebGPU-absent/device-loss
  fallback, reduced-motion/headless equivalence, shader licence/provenance,
  payload and frame-budget measurements on web, Android, iOS, macOS and
  Windows. Taichi and NanoVDB are not dependencies of this task.

  **Owned execution tasklist (estimated 7–10 focused hours; each tranche is
  independently mergeable):**

  GPU-1 through GPU-5a are done (2026-08-30); see `HISTORY.md`. Open:

  7. [ ] **GPU-5b — Release instrumentation and host matrix (2–3 h plus device
     lab).** Measure payload, startup, first-frame time and p95 frame time
     against BRD-072's 9 ms governor. **DoD:** full Vitest/Vite/preflight gates;
     WebGPU absent, compile failure, device loss, headless, background and
     reduced-motion cases retain the SVG endpoint; web/Android/iOS/macOS/Windows
     results recorded; shader similarity/provenance review complete. GUI-098
     remains open until this cross-host matrix passes.
     - [x] Code checkpoint: telemetry, evaluator, asset measurement,
       evidence schema and browser probe are implemented (see `HISTORY.md`).
     - [ ] Lab checkpoint: collect and independently review physical web,
       Android, iOS, macOS and Windows artifacts; the committed template and
       matrix deliberately remain PENDING until those runs exist.
  8. [ ] **GPU-6 — Runtime-to-release evidence pipeline (10–13 h).** Close the
     gap between the renderer's bounded telemetry and the physical release
     artifacts. This is four independently mergeable checkpoints; none may
     turn CI, a simulator or an unavailable adapter into physical evidence.
     GPU-6a (runtime metrics ownership), GPU-6b (probe contract hardening),
     GPU-6c (offline release-tool CI gate) and GPU-6d (end-to-end evidence
     manifest) are done 2026-08-31; see `HISTORY.md`. What remains is the
     still-PENDING physical device matrix, not the pipeline.

## The stage must render the computation (GUI-099)

- [ ] **GUI-099 — Animations that follow the computed numbers.** The owner's
  brief, from the German live deploy: *"we need way better and more complete
  animations for what happens. they must render what actually goes on.
  rendering must follow actual physical computed parameters where possible."*
  The standard this sets is narrow and testable: a visual counts only when its
  size, count, colour, tempo or position is a **function of an
  engine-computed quantity**. A picture of the verb, drawn at a constant, does
  not count — however good it looks.

  `docs/ANIMATION-AUDIT.md` is the walk: every event kind that changes a
  vessel's visible state, what the engine computes for it, what the stage drew,
  what it should draw, and the source (event, scene, or nothing). 73 rows,
  scored done / partial / missing, with the numbers the engine should add
  listed at the end so the engine lane can pick them up.

  Starting score: **32 done, 18 partial, 23 missing.** Finishing score across
  the three PRs: **45 done, 11 partial, 17 missing.** The worst finding was
  not an absence but a constant: `steaming` gated on `temperature_k >= 368`, a
  number that is wrong under a partial vacuum, wrong in a pressurised vessel,
  wrong for a salted solvent and wrong for every solvent that is not water —
  while `state_changed.at` has been carrying the plateau the solver actually
  held the vessel at all along. Two more of the same kind: `dissolved` had its
  magnitude hard-coded to `1`, so a speck and a spoonful of salt dissolved with
  the same picture; and the pressure-controlled piston was drawn at a fixed
  `y`, so squeezing a gas moved nothing. Three events changed a vessel's state
  with no mapping at all (`state_changed` to anything but solid produced a
  `phase-change` effect kind that no component rendered), and
  `SceneVessel.emulsion` is read by no component in the app.

  ANIM-1 (thermal truth), ANIM-2 (matter and pressure) and ANIM-3 (the
  three events that drew nothing) shipped across three PRs and took the
  audit from 32/18/23 to **45 done, 11 partial, 17 missing**; see
  `HISTORY.md`. The remaining 11 partial and 17 missing rows are the open
  half of this item.

  DoD: mappings unit-tested in `magnitudes.test.ts` for monotonicity in the
  driving quantity and for bounds; every new visual reachable from the DOM by
  a `data-*` attribute carrying the number that drives it; `prefers-reduced-
  motion` still stops the motion; no new per-frame JS loop, and the GPU path
  untouched except where `visualBackend.ts` already says WebGPU is on.

## Three instrument surfaces must become one (GUI-100 … GUI-102)

The owner, from the German live deploy: *"we must consolidate Messgeräte and
Geräteschrank and Instrumentenwand. we have 2/3 surfaces and need them into
one."* Design: `docs/INSTRUMENTS-ONE-SURFACE.md`.

The inventory is the argument. Every one of the 12 instruments is listed
twice — once in the `MESSEN` strip (`InstrumentTray.svelte`, inside the
inspector) and once in the equipment cabinet (`EquipmentCabinet.svelte`) —
and `eyes`, `thermometer` and `ph` are listed a third time in the vessel
dock, whose `stir`/`heat`/`cool` are in turn a second copy of three
apparatus. `chromatograph` appears three times, because a kids' kit is a
fourth vocabulary over tools that are already on the wall. The three
surfaces also teach three different mental models — take a reading now /
install on the bench / do a thing to this vessel — and which one a tool
lives in is not predictable from what the tool is.

- [x] **GUI-100 — The design.** `docs/INSTRUMENTS-ONE-SURFACE.md`: what each
  of the three surfaces lists, how it is opened, what state it holds, what
  it emits into the engine grammar; the duplication tally; the target — one
  cupboard modal, opened from one small button at the right end of the
  MESSEN row, built from one merged model, with items on shelves grouped by
  what they do and an `(i)` per item saying what it models and what it does
  not; the migration in three PRs, and the open questions.

- [x] **GUI-101 — The cupboard, from one model.** `equipmentCatalogue.ts`
  merges `INSTRUMENTS`, `APPARATUS`, the transfer verbs (lifted out of
  `EquipmentCabinet.svelte`, where six verbs live in a component array with
  no module and no test), `burette`/`mix`/`transport`/`react` and
  `KIDS_EQUIPMENT` into one list keyed by the catalog id space
  `equipmentAccess()` already uses. `InstrumentCupboard.svelte` renders it on
  six shelves; the five action kinds map one to one onto the handlers
  `App.svelte` already passes down, so no new path into the engine is
  created. The MESSEN strip becomes the ~6 most recently used instruments,
  from the same model, so it never scrolls. DoD: every entry appears exactly
  once with a group and an action; quick-access ordering and its default seed
  unit-tested; availability answered by the engine's catalog for every entry,
  including the ungated-verb case.

- [x] **GUI-102 — Delete the duplicates.** `EquipmentCabinet.svelte` and
  `InstrumentTray.svelte` go; the shelf's *equipment* tab, the dock's single
  cupboard button and `UtilityStation`'s *power and apparatus* all open the
  one cupboard. `tools/test-ux-quality.mjs` gains a cupboard assertion.

  Done 2026-09-06. The shelf pane's *equipment* tab is gone rather than
  rewired: it was a second view of one pane, and the pane it competed with
  is the reagent shelf. The tab row keeps two buttons — the pane you are in,
  and the door to the cupboard — so nothing is now reachable in two shapes.
  `cabinetTab` is deleted, and with it five places that switched a tab in
  order to show equipment: the bench's cabinet button, the mission debrief's
  *place it on the bench*, the Kids Lab brief, the water supply and the
  utility station.

  What did NOT change, deliberately: the vessel dock still offers `look`,
  `temperature` and `pH` from `directActions.ts`. Those three are the last
  hard-coded instrument list in the app, and whether they become the first
  three of the quick-access row is open question 2 in
  `docs/INSTRUMENTS-ONE-SURFACE.md` — a question for the owner, not a thing
  to decide inside a deletion PR.

## Localisation is not finished (I18N-1 … I18N-4)

The shell is locale-keyed and English and German ship together. The *content*
is not, and the gap is now the largest single obstacle to the app being usable
in a German classroom — which is the audience the curriculum mapping in
`codex/` is explicitly aimed at.

- [ ] **I18N-1 — The experiment catalog (Forschungsbibliothek).** 103
  reactions carry English `question`, `misconception`, `reveals`, `next`,
  `lv1`/`lv2`/`lv3` prose and `summary`. The concept topics beside them are
  already German-only (`label_de`, `definition_de`, from the CC0 oehTopics
  set), so the catalog currently mixes languages within one screen. Roughly
  **84,000 words** of pedagogical prose; this is a sustained editorial task,
  not a build step. Structure it as `*_de` fields beside the English ones —
  the convention `label_de`/`definition_de` already establishes — so a
  partially translated catalog degrades to English per field rather than
  failing, and add a lint that reports coverage per file so progress is
  measurable. Machine translation is not acceptable unreviewed here: a
  misconception diagnosis that misstates the misconception is worse than an
  English one.

- I18N-2 (map-screen vocabulary, 2026-08-30) and I18N-3 (engine
  vocabulary coverage, 2026-08-30) are done; see `HISTORY.md`. The
  durable half of I18N-2 is `tools/i18n-slug-lint.py`, in `preflight.sh`.

- [ ] **I18N-4 — Locale-complete store presence.** German App Store and Play
  listings, German "what to test", and the German privacy policy already at
  `privacy.de.html` wired into both manifests.

## Completed GUI tasks

Numbers are never renumbered and never reused. Each of these landed; the detail
and the lessons are in `HISTORY.md`.

GUI-002, GUI-005 · GUI-011, GUI-015 · GUI-020, GUI-025, GUI-026, GUI-027,
GUI-028, GUI-029, GUI-033, GUI-053, GUI-055 · GUI-076, GUI-079, GUI-080
(Phase G2.5 numbering) · GUI-058, GUI-061, GUI-062, GUI-063, GUI-064, GUI-065,
GUI-066, GUI-067, GUI-074, GUI-075, GUI-077, GUI-078, GUI-079, GUI-080,
GUI-083a, GUI-083b (realism-bar numbering) · GUI-087, GUI-091, GUI-095,
GUI-096, GUI-097 · GPU-1 … GPU-5a, GPU-6a … GPU-6d · ANIM-1, ANIM-2,
ANIM-3 · I18N-2, I18N-3 · DATA-010, WEB-003. GUI-060 was superseded by
GUI-065 rather than built; its number stays retired.

Note the deliberate collision: GUI-074/075/077/078/079/080 were issued twice,
once in Phase G2.5 and once in the 2026-08-25 realism-bar addendum. Both sets
are recorded under their own dates in `HISTORY.md`; neither is renumbered.

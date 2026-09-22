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
  flakiness; dependency-free headless Chrome drives the five canonical
  lessons per release through the same DevTools harness as the PWA gate.

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

- [x] **GUI-001 — `EngineHost` protocol v1.** Specify the JSON contract
  (step/runScript/state/scene/events/species/parse/chart) as a versioned
  document + conformance test that both hosts must pass. The current wasm
  API is the seed.
  *Status 2026-09-06: complete — spec written in [PROTOCOL.md](PROTOCOL.md).
  Conformance runs against the CLI/MCP surface
  (`crates/kerotakis-cli/tests/protocol_conformance.rs`, in the test
  suite), the wasm host (`tools/test-protocol-conformance.mjs`, wired into
  CI over the lesson corpus), and Tauri's native dispatch
  (`web/app/src-tauri/src/lib.rs`) — one shape, drift fails before a client
  sees it. The versioned `hello` identity fields are shipped.*
- [x] **GUI-003 — Scene JSON v1.** Per-vessel render model derived from
  existing state + appearance; golden-file tests over replayed lessons.
  *Status 2026-09-06: complete — `kerotakis-core/src/scene.rs` (liquid
  colour+word, solids with metallic/precipitate split, headspace, badges,
  lv1 words), shape-pinning + behaviour tests in-module, wired into the
  wasm `step`/`run_script` responses and `Lab::scene()`. The representative
  cabbage, boiling, filtration, polymer and corrosion lessons now have a
  normalized numeric scene golden after every executable line, plus a
  semantic real-browser DOM golden that pins the quantities and accessible
  words actually rendered. Whole-corpus host conformance continues to pin
  the protocol structure independently.*
- [x] **GUI-004 — One-worker web engine.** Land OPT-11 (lab + IPhreeQC in a
  single module worker) behind `WorkerHost`; the current PWA runs on it
  unchanged. Measure; OPT-9 only if numbers demand.
  *Status 2026-09-06: complete — `WorkerHost.ts` owns transport and
  `engineWorker.ts` loads the bench wasm and IPhreeQC together off-thread.
  OPT-11 measured the crossings; OPT-9 recorded the evidence-based no-build
  decision for additional marshalling work.*

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
- [x] **GUI-017 — Continuous deploy.** `.github/workflows/ci.yml` builds and
  browser-tests the complete payload on pull requests and pushes, then deploys
  that artifact to GitHub Pages after the demo job on `main`; the Vercel
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

*Liquid-nitrogen learning slice (2026-09-06):* the existing lesson and mission
surfaces carry the coupled phase-change investigation. Story reach follows
completed investigations through Energy Yard, never learner age; the mission
temporarily loans the cryogen and declares no permanent reagent reward.

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

  *Correction 2026-09-06 — the tiers are gone, not merely merged.* One
  card design and one horizontally scrolling filter rail (level, topic,
  duration, "only what is on my shelf", done/not yet, plus concept and
  curriculum as selects where the old tabs were) now serve all 168
  entries; `lib/catalogEntry.ts` reads an authored learning-progress band,
  derives duration and maps both corpora into a shared topic vocabulary, so every filter is
  answerable for every entry, and no surface names a reader by age.

  *Progress-authority correction 2026-09-06.* All 108 Codex reactions now
  require an explicit `starter`, `intermediate` or `advanced` value in the
  Rust-owned schema, just as all 60 guided entries do. The reviewed assignment
  follows prerequisite depth and the number of concepts, calculations and
  models demanded; CI rejects missing/unknown values and any prerequisite for
  which every teaching route is later than its consumer. Curriculum ages stay
  available for syllabus browsing but no longer classify cards. Codex entries
  also make no invented safety claim where their source carries none.

  *Update 2026-09-06 — two mechanism-backed entries are promoted.* Activated-
  charcoal adsorption and the thermoplastic/thermoset heat comparison are
  searchable, filterable and directly runnable. Their copy states the curated
  parameter domains and safety limits; neither claims a general-purpose model.

  *Discovery tranche 2026-09-06:* cards and details now say whether the
  current shelf and the engine's catalog answer make an experiment ready,
  name exact missing reagent ids, and render engine-owned locked, loaned and
  mission-only reasons. A ready/missing facet composes with the existing rail.
  Continue-with links use only authored Codex, lesson and capability ids;
  titles are never treated as relations. Until the catalog answer arrives,
  readiness says it is checking and belongs to neither ready nor missing;
  authored instrument tokens resolve through the single equipment catalogue.
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
  *Disposal slice shipped 2026-09-07: the waste station now disposes. A
  selected vessel holding anything is emptied into the bench's shared waste
  container by the engine's own `discard vN` verb — asked once, weighed,
  logged, replayable and undoable — and the station shows back what the
  container took; an empty selection keeps the older bench-clearing
  meaning. The remove-vessel dialog's "open waste station" signpost now
  arrives with that vessel selected. Listing what is standing in the
  container remains: `Bench.spills` is not on the scene wire.*

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
  *Discovery tranche 2026-09-06:* the live Story Map now states the exact
  number of further completed investigations each locked district needs,
  both on its map node and in its lock panel. The retired, unmounted
  `MissionControl.svelte` is deliberately not treated as a product surface;
  the larger open map work above remains.
  Concept Map mission links consume that same district result, remain disabled
  with the exact remaining count while locked, and therefore cannot bypass the
  live mission surface's progression gate.
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
- [x] **GUI-052 — Provenance drawer** rendering the R0 capability/validity
  reports and the property-resolution ladder rung per number. The routing
  record (`SolverStack::last_routes`) reaches the wire as `step.routes`,
  native and wasm alike, with the declining solver's own sentence beside it;
  `provenance.ts` builds the drawer's model from a step's events and routes,
  and `ProvenanceDrawer.svelte` shows the solver chain in order, the datasets
  and models any `provenance` names, the validity notes riding a reading, and
  the honesty pass's refusals — each in the engine's own words. `ValidityBounds`
  is deliberately not rendered: no solver in the tree populates it, so a box
  for it would imply a check nobody ran (PR #512).

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
  above, with the spectators named at lv3.*
  *Closed 2026-09-21 by the second slice: the COMPLETE ionic equation, with
  the spectators on both sides at solved coefficients and flagged so the
  shell strikes them through. There is no stored molecular equation to take
  those coefficients from — the bench never believed in one — so they come
  from the fact that the reagents arrived in electrically neutral bottles,
  which gives `2 Na⁺` and `2 Cl⁻` around a barium sulfate without anything
  in the beaker saying "two". Where that cannot be solved and verified —
  two cations and no way to say which salt was opened, or no spectator of
  the needed sign — nothing is drawn and the net line stands, which is the
  honest fallback rather than a failure. Still open: showing the neutral
  complexes beside the free ions, and any basis beyond these two — redox
  and organic steps carry no participant list yet and are not guessed at.*

- [x] **GUI-093 — Shelf by chemical role. DONE 2026-09-18.** Acid, base,
  salt, metal, oxide, indicator, gas. We filtered by phase —
  aqueous/liquid/gas/solid — which is a physics axis while the learner is
  thinking on a chemistry one. Phase stays as a secondary filter.

  *The role axis arrived first as a chip rail (`reagentRoles.ts`): every
  role is DERIVED — from the NOAA-style reactive-group rows CI already
  forces to be total over the registry, from the engine's own formula
  parser, from the indicator and solvent tables that actually compute the
  behaviour — so nothing here names a species and a species added to the
  data pack cannot arrive silently unclassified. Where the inputs decide
  nothing the species is `unsorted` and the shelf says so in as many words,
  which is the honest answer for elemental oxygen and for the enzymes'
  stand-in bare "C".*

  **What 2026-09-18 added, and it is the half that reaches a learner who
  does not yet know what to ask for:** the shelf is now LAID OUT by role,
  not merely filterable by one. Sticky headings per group, in
  `REAGENT_ROLES`' pedagogical order rather than the alphabet — so the
  acids do not migrate down a French shelf because *acides* sorts
  differently from *Säuren*. One heading per bottle, never several: a
  species holding two roles (citric acid is an acid and an organic) is
  filed under the first in that order, because filing it under both would
  make the shelf longer than the cabinet and the tally beneath it a lie.
  The other roles are not lost — the chips still find it and the (i) panel
  names every one under "chemical family". Headings appear only when there
  is more than one group and no role chip is pressed.

  **And the hazard mark on the tile itself**, so the warning arrives at
  choosing time rather than at pouring time. Three states and two glyphs:
  an assessed hazard wears ⚠, a species nobody has assessed wears **?** in
  dim ink rather than alarming ink, and a species assessed as carrying no
  hazard wears **nothing**. Silence has to mean "checked, clean" for either
  mark to mean anything, which is exactly why "unassessed" may not be
  allowed to look like "safe" — there is a test whose only job is to fail
  the moment it does. A glyph and not a word because this row is ninety
  rows long and was already "mostly badge" once; the sentence it stands for
  is the label a screen reader hears, the tooltip a pointer finds, and the
  full row under the (i).

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

- [x] **GUI-099 — Animations that follow the computed numbers.** The owner's
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

  Starting score: **32 done, 18 partial, 23 missing.** The first three visual
  PRs reached **45 done, 11 partial, 17 missing**; the persistent corrosion
  extent tranche then reached **46 done, 11 partial, 16 missing.** The worst finding was
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

  ANIM-1 (thermal truth, #450), ANIM-2 (matter and pressure, #454) and
  ANIM-3 (the three events that drew nothing, #458) shipped across three PRs
  and took the audit from 32/18/23 to **45 done, 11 partial, 17 missing**;
  the six quantities the audit asked the engine for went onto the wire in
  #462. See `HISTORY.md`. Persistent corrosion extent moved one further
  missing row to
  done. The computed-motion tranche then made gas production cadence follow
  `rate_moles_per_second` and foam collapse follow `half_life_seconds`, moving
  two partial rows to done, reaching **48 done, 9 partial, 16 missing**.

  ANIM-5 through ANIM-9 (#490, #492, #493, #494, #495) then closed the rest,
  five slices of at most six rows each: 48/9/16 → 52/9/12 → 58/9/6 →
  64/9/0 → 70/3/0 → **73 done, 0
  partial, 0 missing.** The last two slices are the ones worth naming here,
  because they were not absences but *constants*: `plated`'s magnitude was a
  literal `1`, so a copper blush and a nail gone orange drew the same
  shimmer; `vessel_swept` drew two static arrows whatever the sweep;
  `enzyme_hydrolysed` had a caption percentage beside a liquid that never
  changed; `mixed` threw away all three of the temperatures its adiabatic
  balance had computed. Every one of the 73 rows is now a function of an
  engine number carrying a `data-*` attribute that names it.

  ANIM-10 adds a standing counterpart to the already-complete transient
  partition row. While water and hexane coexist, the scene recomputes each
  supported neutral solute's lower/upper equilibrium share from current matter
  and the same UNIFAC calculation used by `drain`; the accessible bars survive
  between events and disappear when the layers separate. This improves
  persistence without changing the closed 73-row event score.

  ANIM-11 adds a standing counterpart to the transient osmosis row. The
  prepared object's durable cumulative water exchange is exposed as signed
  moles and grams in the existing vessel, surviving between events and reloads.
  It does not resize the object or claim membrane mechanics or final
  equilibrium, and therefore does not change the closed 73-row event score.

  **This item is closed and the question it asks is not.** One gap outlives
  the row count, recorded in the audit. (The other, `Event::DidNotIgnite`
  carrying nothing but a vessel id, is now closed in #501: the event names
  *which*
  absence it is — `no_fuel`, `no_oxygen`, `below_autoignition`,
  `not_modelled` — and carries the candidate fuel, its moles, the oxygen
  fraction and the gap to the autoignition point, so a wisp scaled by the
  fuel is drawn for the one of the four that is drawable and nothing at
  all for the other three.) And a row score cannot
  see the difference between a visual that is right *at the instant of its
  event* and one that stays right *between* events; that was the whole
  subject of the scene-numbers PR, and it is the standing risk in every
  transient effect this item shipped.

  DoD: mappings unit-tested in `magnitudes.test.ts` for monotonicity in the
  driving quantity and for bounds; every new visual reachable from the DOM by
  a `data-*` attribute carrying the number that drives it; `prefers-reduced-
  motion` still stops the motion; no new per-frame JS loop, and the GPU path
  untouched except where `visualBackend.ts` already says WebGPU is on.

## Three instrument surfaces must become one (GUI-100 … GUI-103)

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
  not; the migration in three PRs, and the open questions. #463

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
  including the ungated-verb case. #466

- [x] **GUI-102 — Delete the duplicates.** `EquipmentCabinet.svelte` and
  `InstrumentTray.svelte` go; the shelf's *equipment* tab, the dock's single
  cupboard button and `UtilityStation`'s *power and apparatus* all open the
  one cupboard. `tools/test-ux-quality.mjs` gains a cupboard assertion.

  Done 2026-09-06 (#469). The shelf pane's *equipment* tab is gone rather
  than
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

- [x] **GUI-103 — The owner's five answers.** All five open questions in
  `docs/INSTRUMENTS-ONE-SURFACE.md` §5, in one PR: the MESSEN strip survives
  at **four** recents plus the cupboard door and excludes the dock's
  `look`/`thermometer`/`pH` (`DOCK_INSTRUMENTS`), so nothing is offered twice
  on one screen; the dock keeps those three as fixed landmarks; the kits
  become a **header chip** that renames the tools they stand for rather than a
  sixth shelf that showed the candle twice; **five shelves** — *antreiben*
  folded into *vorbereiten* (stirrer, mortar, centrifuge, lamp, dilute,
  curated reaction) and *verbinden* (electrodes, half-cell, tubing, gas line,
  burette, mixer) — each with one sentence saying what lives on it; and both
  defects fixed: `directActions.ts` names the heat source explicitly
  (`heat v1 10kJ on burner`) and the cupboard's denominator is a constant over
  every tool a learner can ever have, printed only while something is locked.
  Done 2026-09-06 (#475); §5 of the design note is now *Decisions* and
  carries the
  reason for each. Per the convention at the top of this file, the detail and
  the lessons live in `HISTORY.md`.

## The cupboard, after the owner used it (GUI-109 … GUI-111)

Three of the five things the owner hit in the German deploy landed on this
cupboard, so they are recorded here rather than in a section of their own.
The other two are with the journal and with the result card.

- [x] **GUI-109 — The cabinet's symbols are not drawings of anything.** The
  owner: *"most symbols are not even intuitive. we should have much better
  icons/symbols/drawings of the devices"*.

  Done. Ten of the twelve readings are inline SVG portraits in
  `ToolIcon.svelte` now — the same 18×18 box, the same stroke-only
  `currentColor` hand the apparatus have had since GUI-033 — and they are
  drawn in the cupboard and in the MESSEN strip, which are two doors onto
  one instrument and were showing two vocabularies.

  **Two are deliberately still letters, and this is the finding worth
  keeping:** `pH` and `Bq`. Both are the quantity's or the unit's own
  notation, translated into no language the app ships, so neither is the
  untranslatable text the no-text rule is about. And neither instrument has
  a silhouette: a pH probe is a rod, a Geiger counter is a box with a cable
  and a tube, and at 24 px both are grey rectangles. The radiation trefoil
  would read — but it marks RADIOACTIVITY, not a counter, and dressing a
  hazard symbol as an instrument is exactly the kind of shape that means
  nothing. A letter is honest about being a label; a bad drawing is not.
  `ToolIcon.test.ts` pins the pair so that giving one of them an icon is a
  decision somebody makes on purpose.

  **A defect the guard found on its first run:** `cool` — the cooling bath
  — named an icon `ToolIcon` had no path for. It renders `{#if d}`, so the
  tile had simply been drawing NOTHING since the portraits landed, and an
  empty box on a shelf looks like a design choice from the outside. Nothing
  threw, nothing logged, and no test could see it because every component
  test renders through `svelte/server` and would have produced the same
  empty markup. It is drawn now, as the deliberate opposite of `heat`: the
  same dish, a snowflake instead of rising heat. The test that caught it
  holds the whole catalogue against the paths the component declares.

- [x] **GUI-110 — The (i) buttons eat the cabinet.** The owner: *"the (i)
  buttons there should be more tiny and e.g. in upper right corner of the
  drawings of the devices, not occupy that much room"*.

  Done: `InfoToggle` gained `placement="corner"`, and the cupboard uses it.

  **How the 44 px floor survived the shrink.** By separating the two things
  a button conflates — what you see and what you can hit. The button
  element is still a 44 × 44 square, anchored to the tile's top-right
  corner and transparent; the thing painted inside it is a 19 px circled
  *i* in that corner. Nothing about the hit area was reduced.

  That is only honest if the square is genuinely spare, and it was not: a
  44 px corner on the old 83 px phone tile would have sat over the drawing,
  so a finger aimed at the tool would have opened its explanation instead.
  So the tile changed with it. It stands its contents on the LEFT rather
  than centred — 34 px drawing from the left padding, name underneath — and
  the grid's column minimum went 6.4 rem → 6.8 rem on a desktop and 5.2 rem
  → 5.8 rem on a phone. At 5.8 rem (93 px) the drawing runs 7–41 px and the
  (i) square runs 49–93 px: they do not meet, at the narrowest width the
  app is audited at. The browser guard asserts both halves — every toggle's
  rectangle is at least 44 × 44, and no toggle's rectangle intersects its
  tile's drawing — and then presses the square's outer CORNER and checks
  that `elementFromPoint` there is the toggle and not the tile beneath it.

  The deployed badge moved to the tile's bottom-right, because the top
  right now belongs to the (i).

- [x] **GUI-111 — "Experimentierkästen" earns nothing.** The owner: *"it
  makes no sense that '◆Experimentierkästen' changes a little bit like
  Chromatograph => Papierchromatograph. probably just remove that button
  (while of course keeping all features)"*.

  Done: the chip, `asShown()`, `instrumentSurface.sets`, `showSets()` and
  the stored `kerotakis.equipment.sets` key are all gone.

  **What the work found — what the button actually did.** It swapped five
  shelf slots for the `aliasOf` entry that skins them, which carried five
  differences: the name, the icon, the parts list, the modelling caveat,
  and — on one of the five — a `preset`. Four of those are description and
  one was behaviour.

  Where each went:

  * **the name** — into the tool's own (i) panel as an *Auch genannt* row,
    and into the search haystack. It was already half-there: the filter has
    searched both vocabularies since GUI-103. It searched them only in the
    ACTIVE language, though, so a German reader could find the burner by
    typing "Kerze" and an English reader could not find it by typing
    "candle and wick". Both vocabularies in both languages now.
  * **the parts list** — into the same (i) panel. A tool a kit names wears
    that kit's inventory, which is the list a learner setting it up needs.
  * **the caveat and the icon** — dropped. The tool's own boundary sentence
    is the more accurate of the two (the kit's said "uses the existing
    two-vessel filter"; the tool's says what the filter models and does
    not), and the tool's own drawing is of the apparatus.
  * **the preset** — `candle-kit` carried `preset: { source: "candle" }`.
    This is the one that was real: the engine caps a candle 100 °C below a
    laboratory burner, so it is a physical claim and not a label. It
    survives because the flame panel's `source` field has always offered
    `candle` alongside `burner` — see the `bunsen` spec in `apparatus.ts`.
    The capability is reachable in full; what is gone is a shortcut that
    pre-selected it. That is the better home for the choice anyway: which
    flame you are holding is a property of the flame, not a different
    device.

  **Which label stays, and why.** The laboratory name. It is the name the
  engine grammar, the catalog ids and the codex already use; it is the
  general tool, and naming a general operator after one classroom special
  case understates it — the chromatography operator does more than paper;
  and the kit name now has a better home than a label that half the
  sessions never saw.

  `setSkinOf()` and the `aliasOf` entries stay, because two things still
  need them: the search haystack and the (i) panel. Deleting them would
  have made the kit vocabulary unreachable, which is the one thing the
  owner ruled out.

## Localisation is not finished (I18N-1 … I18N-4)

The shell is locale-keyed and English and German ship together. The *content*
is not, and the gap is now the largest single obstacle to the app being usable
in a German classroom — which is the audience the curriculum mapping in
`codex/` is explicitly aimed at.

- [x] **I18N-1 — The experiment catalog (Forschungsbibliothek).** German for
  every authored string the catalogue carries: **1255 of 1255**, up from
  1108 of a denominator that was itself wrong. Translations stay one file
  per language (`codex/i18n/<code>.toml`, keyed by the path to the English
  field), folded into the `_de` siblings the shell already reads, so adding
  French remains one new data file and no code.
  `tools/codex-locale-lint.py` is promoted from a report to a gate in
  `preflight.sh`: a stale key, a positional list whose length changed, an
  age band (GUI-470 — this prose bypasses the locale bundles, so
  `learnerWording.test.ts` cannot see it) and, for a language in `COMPLETE`,
  a missing string all fail. A language NOT in `COMPLETE` is only reported
  on, so a translation in progress stays committable.
  `codexProse.test.ts` gates the same claim from the exported document the
  app parses. The two things this uncovered are the point: `models.toml`
  reported **100%** German while 325 of its 409 strings were English,
  because the lint's field list omitted `name`/`power`/`explains`/`fails_at`
  and `Model` had no `_de` fields for serde to keep — a coverage number is
  only as honest as its denominator. #505

- I18N-2 (map-screen vocabulary, 2026-08-30) and I18N-3 (engine
  vocabulary coverage, 2026-08-30) are done; see `HISTORY.md`. The
  durable half of I18N-2 is `tools/i18n-slug-lint.py`, in `preflight.sh`.

- [ ] **I18N-4 — Locale-complete store presence.** German App Store and Play
  listings, German "what to test", and the German privacy policy already at
  `privacy.de.html` wired into both manifests.

## The library understates the product by 44 lessons (GUI-104)

The owner asked, 2026-09-16, why the app shows only 208 experiments. The
number is honest arithmetic and it is `App.svelte:1577`:

```
experiments={codexEntries.length + kidsExperiments.length}
```

131 runnable codex reaction routes plus 77 guided experiments. What it omits
is the lesson shelf. **113 `.lab` lessons ship. Only 69 are referenced by a
catalogue entry.** The other 44 — among them `electrolysis`, `calorimetry`,
`fire`, `buffer`, `conductivity`, `density`, `boiling-curve`,
`current-time-and-electrolysis-yield` — exist only in the lesson picker,
because `tools/lessons-index.py` files anything absent from its `TOPICS`
grouping into a `"more"` bucket. They run. They are correct. The Research
Library does not know they exist, and neither does anyone browsing it.

So the truthful inventory is 131 + 77 + 44 = **252**, and the shortfall was
17% of the product, invisible. Closed 2026-09-16.

- [x] **GUI-104 — Give every shipped lesson a catalogue entry.** Landed as
  K78–K121, one row per unlisted `.lab`. The headline the app computes moves
  from 208 to **252** without touching `App.svelte`, because it was always
  derived. Every row was written from *running* the lesson, which is how four
  of them came back `partial` instead of `computed`: `hard-water-soap-boundary`
  (no fatty-acid soap species, so the hard and soft vessels look identical),
  `there-and-back` (the hydroxide is spent neutralising free acetic acid, so
  the requested saponification reports no conversion), and
  `track-precipitate-and-filtrate-through-drying` (a dried filtrate still
  reported as dissolved ions, which the bench refuses to resolve into a solid
  rather than guessing one). German is authored for all 44. The grouping in
  `lessons-index.py` absorbed the 13 of them that were in the `"more"` bucket.
  The invariant is now a test — every `.lab` on disk is reachable from the
  catalogue, with the unreferenced files named in the failure message — so the
  next lesson to land cannot go missing the same way.

  Two things the work turned up and deliberately did not fix:

  - [x] **GUI-104a — `starch-iodine-test.lab` shows its colour. DONE 2026-09-17.**
    The blue-black was computed all along; the script asked `inspect`, so the
    run printed an inventory and the phenomenon the lesson is named for never
    appeared. Three `inspect` lines became `look`, and the transcript now
    reads control **brown**, test **brown**, then test **blue-black** after
    the one thing that changed — which is the comparison the lesson is built
    on. The golden was re-blessed: three lines added, no number moved. K111
    is promoted `partial` → `computed`, and its boundary note now states the
    limit that actually remains — the colour is a **visible state, not a
    measured absorbance**, so the lesson cannot say how much starch is
    present from how dark it went.
  - [x] **GUI-106 — the engine says nothing happened, then describes what
    happened. DONE 2026-09-17.** Running the fixed lesson:

        You add cornstarch to v2.
        Hmm — nothing visible happens in v2 (this part of the lab isn't awake yet).
        You look closely at v2. The liquid is blue-black and so cloudy you cannot see through it.

    Same family as the two defects #626 found — a filtered beaker of sand
    and a sealed gas flask that both described themselves as `"."` — where
    the words contradict the scene the reader is looking at.

    **The gap is real and the sentence carrying it is not.** No wired
    solver speciates starch, so the note is true; the lv1 register says it
    in a sentence that makes a SECOND claim, about the beaker, which is
    not this event's to make and which the next line refutes.

    So the bench measures the thing the sentence claims. It reads
    `liquid_colour_word_of` before the operator and again after every
    solver — the same before/after shape it already uses for swelling,
    curdling and the luminol glow, and its own existing answer to "the one
    word a person would use for the liquid in this vessel", which EXP-39's
    self-indicating endpoint reads and nothing else — and sets
    `NotYetModeled.beside_a_visible_change` where the word moved. lv1 then
    says *"Something did change in v2 — but part of what happened isn't
    modelled yet."*

    **Nothing is suppressed, which is what makes it safe.** At lv2 and lv3
    the reason is word for word what it was, and at lv1 the note is still
    said. There is no path by which a genuine gap goes unreported, so the
    guard is not a judgement call about which gaps matter.

    The colour WORD and not the whole observation, and that difference
    decides two cases that look alike. Dropping iron into water changes
    what is IN the beaker and changes nothing about how the water looks —
    that is K17, the experiment the lv1 sentence was written for, and it
    keeps its sentence unchanged. Stirring cornstarch into Lugol takes the
    liquid from brown to blue-black.

    Four tests, two of them the "both ways" pair: the contradiction is
    gone and the reason still prints at lv2; a gap over an unchanged
    liquid keeps "nothing visible happens"; the flag belongs to the VESSEL
    and not to the step; and the real lesson on the shipped binary reads
    what a learner reads.

    That last one asserts three things and needs all three. The absence of
    the wrong sentence is worth almost nothing alone: it is satisfied just
    as well by a lesson that has stopped computing, and by a gap that
    stopped being reported — the same defect wearing silence instead of a
    wrong sentence. So it pins the colour line (**blue-black** in the test
    vessel, **brown** in the control, which is the comparison the lesson
    IS) and the replacement note beside it. Together they say: the gap
    still fires for that step, the defect was real, and what stands there
    now is true.

    *The limit, written down rather than glossed:* a liquid that goes
    cloudy without changing colour word does not trip this, and neither
    does a change that is only a new solid at the bottom. Both are
    reachable from the same measurement if a later reader wants them; the
    colour word is where the evidence was.
  - [x] **GUI-104b — the picker's `"more"` bucket is empty. DONE 2026-09-17.**
    47 of 113 lessons sat in it. Every one carried authored `topics` in the
    catalogue, so the grouping already existed and was simply written down
    somewhere the picker could not see. `lessons-index.py` now derives a
    shelf from those topics through one ORDERED list, `CATALOGUE_TOPIC`, and
    the order is the editorial judgement: `proteins` outranks `heat`, so
    heating egg white is food rather than thermochemistry, while
    `invisible-ink-boundary` carries heat and food with no protein and the
    same list sends it the other way. Two shelves were added because the
    catalogue sends lessons to them — **food & life** (8) and **everyday
    materials** (2, both non-Newtonian). `TOPICS` keeps its curated members
    and its order, and still wins where it has an opinion. Three tests: no
    shipped lesson lands in `"more"`, a lesson with no catalogue row still
    does (the payload builds must work from a lessons directory alone), and
    the ordering itself is pinned by the two lessons that demonstrate it.

## German stops where the engine starts talking (I18N-5 … I18N-9)

The shell is fully German. The *lesson* is not, and a single playback shows
exactly where the boundary falls — owner report, 2026-09-16:

> Du gibst Citronensäure in v1. / Du gibst baking_soda in v1. / Du siehst dir
> v1 genau an. **There is white baking soda (sodium bicarbonate) and white
> citric acid in the beaker.**

Every gap below is a *missing mechanism*, not a missing string, which is why
none of them was caught by a translation count. The requirement for all five
is the standing one: **adding French must stay one core `.toml` plus one web
`.json`, and no code.** Any fix that puts a German word in a match arm, or a
display name in the registry, is the wrong fix.

- [x] **I18N-5 — Material names are never localised.** `Event::Added` renders
  through `species_name`, which is why *Citronensäure* and *Wasser* arrive in
  German; every event naming a **material** printed the key the script typed,
  so the next line read `baking_soda` and a vinegar bottle announced itself as
  "white vinegar 5 percent". Fixed by `material_name` in `render.rs` across all
  15 sites, against a new `[material]` section carrying all 132 recipes. **No
  translation was authored**: each value is the recipe's own first German
  alias, which the *parser* has accepted since BRD-002 — the renderer simply
  never asked. Display belongs to the catalogue so that French stays one file.
- [x] **I18N-6 — `Detailstufe lv1`.** `lv1` is a protocol token, not a word,
  and reached the notebook verbatim. It now goes through `t()` like everything
  else, with a fallback to itself for a register a locale has not named.
- [x] **I18N-7 — `appearance.rs` has no locale at all.** Not a missing key: the
  file never took a `Locale` and could not have used one — it reached the
  reader through an EVENT, and by the time anything knew who was reading, the
  words had been chosen and the values baked in. Fixed by making the engine
  emit the RECIPE and the host cook it: `crate::phrase::Phrase` is a key, its
  English source, and typed slots, and `Appearance` now carries the clause
  list beside the English `words` that every existing consumer still reads.
  `Slot::Term` is what a flat `fill` could not express — the composer knows
  that *silver chloride* is a species and *white* is an appearance word, and
  by the time a sentence is a string that knowledge is gone. The list grammar
  (`", "` and `" and "`) and even the full stop are catalogue rows, because
  they are grammar rather than punctuation the moment the language is not
  English. 22 new German rows under `[look]`. Note `look.coloured`: English
  writes "white silver chloride", French writes it the other way round, and
  German would have to inflect the colour to the noun's gender — so the
  German construction puts the colour beside the name, a decision that lives
  in the data and not in the Rust.
- [x] **I18N-8 — Inert-reason prose is English.** `v1 Zink reagiert nicht —
  zinc should dissolve in this acid by the series (driving force +0.62 V), but
  hydrogen has to form on zinc…` The refusal's *name* was translated and its
  *reason* was not. Six composed verdicts in `displacement.rs`, `solve.rs` and
  `nonaqueous.rs` now carry a `Phrase` in the event beside the English, and
  the English is **generated from that phrase** rather than written next to
  it, so there is one sentence and not two to drift — the codex entry that
  quotes `inert.hydrogen-overpotential` verbatim still matches. The 25
  curated organic-solvent verdicts are keyed by their row of
  `INERT_IN_SOLVENT`, not by their English, so rewording one does not orphan
  its translation. Numbers go through `Slot::Number`, which is why a German
  reader sees +0,62 V.
- [x] **I18N-9 — Lesson prose had no translation mechanism.** The title,
  description, section comments and boundary note were the `.lab` file's own
  comments, rendered verbatim; only the *slug* was translated, which is why
  "Trocken, dann nass: Brausen" sat above six lines of English. **All 113
  lessons are migrated**, 413 labels, every one of them with authored German.
  Built as scoped, with four differences worth naming:

  * **A labelled comment**, as designed: `#@part.displacement Part 1 —
    displacement: …`, and `spannungsreihe.lab` carries exactly that label. It
    is still a `#` comment, so `kero run lessons/x.lab` is unchanged — proven
    by replaying every one of the 113 migrated files against its pre-migration
    self, output identical in all 113.
  * **The unit is the PARAGRAPH, not the line.** A labelled comment owns the
    plain comment lines under it. A `.lab` wraps at 78 columns for the
    terminal and that wrap is typography, not grammar — I18N-7 already paid
    for the alternative once. The one place where the line break IS the
    meaning, `electrolysis.lab`'s Faraday arithmetic, is labelled per line
    (`calc.charge`, `calc.electrons`, …) rather than run together.
  * **`lessons/prose/en.toml` is GENERATED** from the `.lab` files rather
    than maintained beside them. The roadmap called it the source; the `.lab`
    is the one source of truth for a lesson (GUI-020), and two hand-kept
    copies of the same English is a drift the lint would then have to police
    anyway. `--write` regenerates it, `--check` fails when it no longer
    matches the words the lesson says — which is how a reworded sentence
    forces its translations to be looked at again.
  * **A dotted label nests in TOML**, so `intro` and `intro.2` cannot both be
    keys of one lesson: the catalogue simply stops parsing.
    `lesson_prose.clashes` names the two labels and the file instead.

  `tools/lessons-index.py` stays the single source both payloads read: it
  emits `blurb_key` beside today's `blurb` and compiles the catalogue into
  `lessons/prose.json` (91 KB of German) beside `lessons/index.json`. English
  is deliberately absent from that file — the `.lab` carries it inline and the
  player falls back to it, so shipping it twice would be one more copy to
  drift. **Adding French is `lessons/prose/fr.toml` and no code**, discovered
  by filename the way `src/locales/*.json` already is.

  Two gates, because the lint and the screen can disagree:

  * `tools/lesson-prose-lint.py` — denominator is every label a `.lab`
    **references**, read from the lessons. It fails on a missing German row,
    an orphan no lesson asks for, one label holding two sentences, a
    malformed `#@` line, and a stale `en.toml`; it reports, without failing,
    how many lessons are migrated and how many comment lines still render
    verbatim (now 113 and 0), and which rows are identical to their English
    (one: a formula line with no words in it).
  * `web/app/src/lib/lessonProseParity.test.ts` — the PLAYER's parser walked
    over all 113 real lessons and held against the same catalogue. A payload
    keyed by one parser and read by another is green on both sides while the
    reader meets English, which is #505's shape one layer along. The lint
    also pins the player's two regexes, so a change there fails loudly
    rather than silently making every key a guess.

  One thing scoped and *not* done: folding `web/app/src/locales/de.json`'s
  lesson slugs into `lessons/prose/de.toml`. They turn out not to be the same
  data said twice — the slug is the lesson's short NAME ("Trocken, dann nass:
  Brausen"), the `title` label is its opening sentence ("Trocken, dann nass:
  warum Brausepulver und Badebomben auf Wasser warten"), and the picker shows
  both. Folding them would need a key no `.lab` references, which the lint
  would rightly call an orphan.
- [x] **I18N-10 — `NotYetModeled.what` is the same defect one event along.
  DONE 2026-09-17.** *82 of 82 sites carry a recipe; the lint reports zero
  still holding a finished English sentence, and 553 of 553 reachable keys
  carry German.* `what` was a finished English sentence for exactly the
  reason `Inert.why` was, and two of them sat in `displacement.rs` beside
  the verdicts that now speak German, so a reader of the zinc-in-vinegar
  lesson met one English paragraph in a German transcript. Both of those
  are done.

  The count in the earlier note, 93, was wrong — it counted patterns as
  well as constructions. `tools/engine-locale-lint.py` reads it out of the
  source now and reports **82**, of which a `matches!` arm is none.

  * `Event::NotYetModeled` carries `reason: Option<Phrase>` beside `what`,
    exactly as `Inert` does.
  * **`Event::not_modeled(vessel, cause, reason)` generates `what` from
    the recipe**, for the reason `Event::state_changed` exists: written by
    hand at eighty-two call sites, the English and the translation are two
    copies that drift. Converting a site is now one call, which is what
    makes the remaining tranches cheap.
  * `reason: None` is the honest unconverted state and still falls back to
    `localize_refusal`'s English-keyed `[refusal]` table.
  * The eighteen FIXED gap reasons that `[refusal]` could reach are
    `[not-modeled]` rows now, keyed by their place. **The German was moved,
    not rewritten.** `i18n_coverage.rs`'s scraper has nothing left to
    scrape and has been turned around: writing a gap reason as a finished
    English sentence again is now the failure.

  **Tranche three, the last twenty-six.** Every one outside `aqueous.rs`
  passed a sentence through from a helper, so the work was in the helper
  and the call sites followed: `SolventActivity::out_of_range_reason`
  (two keys, because the two routes say different things and not the same
  thing about a different route), `solve::stranded_solutes`,
  `SolventState::boundary`, `volatility::additional_solvent_cut`,
  `family::outcome_extent` and `KineticReaction::proton_consumption_
  boundary` all return a `Phrase` now. The curated
  `UNAVAILABLE_SOLID_PHASES` verdict is keyed by its ROW, the
  `INERT_IN_SOLVENT` treatment #626 chose, and so are
  `derived::UNSPECIATED_ACIDS` and the sentence quoted out of the whole
  milk recipe's `lot_assumptions`.

  Three things fell out that were not translation:

  * **`StructureOracle::apply` errs with a `Phrase`, and its KEY is what
    the router files the refusal's cause on.** It read
    `why.contains("cannot name")` — an English sentence doing structural
    work, so rewording the refusal would have silently refiled a registry
    gap as a model boundary. The key is a `pub const` in
    `kerotakis-core::family` because two crates share it.
  * **`unspeciated_acid_notes` carried the reason out of
    `UNSPECIATED_ACIDS` and threw the key away** — and the key is the only
    thing a curated row can be translated BY.
  * **`localize_refusal` rescues two gap reasons by STRIPPING the species
    name off the end of the English** and refilling a template with what
    is left. Both sites emit a recipe now, so nothing the engine emits
    reaches it — it is a REPLAY shim, for a session saved before today
    whose events hold `reason: None` and the English, and a reader
    opening that save is owed the German it had. It was deleted first and
    put back, because deleting it silently un-translates an old save.
    What DID change is the key it fills: the row it used to own was
    moved, so it now fills the key the live site composes and one German
    row serves both paths. Finding the noun by looking at the end of the
    sentence works only because English puts it first, which is the whole
    reason the event carries a recipe now.

  `phrase::sentence_pair` is new: the space between two whole sentences is
  `look.sentence-join` in the catalogue, for the reason `look.full-stop`
  already is. It replaced a `push_str` and a `.trim()` in the aqueous
  crate's reference-complex boundary, where two optional caveats made
  three shapes out of two booleans.

  **Three holes in the lint's own denominator,** each of which would have
  let a number go green by leaving work outside it. #505's scar again, and
  worth writing down because none of them looked like a denominator bug:

  * It cut a file at its FIRST `#[cfg(test)]`. `kinetics.rs` has a test
    module at line 1294 and a thousand lines of engine after it, so
    `proton_consumption_boundary` was invisible in both directions at
    once — its key reported as an orphan, its row outside the total. Test
    modules are brace-matched and removed now, wherever they sit.
  * A comment between `Phrase::new(` and its key lost the call.
  * A key named by a `const`, or one whose English is DATA rather than a
    literal, was not read at all.
  * `phrase.rs` was read for `locale.t` calls only, so `look.sentence-join`
    — the one clause that file composes itself — sat outside the
    denominator and its German was never asked for.

  `states.rs`, `volatility.rs`, `kinetics.rs`, `aqueous.rs`,
  `phase_diagnostics.rs` and `family_oracle.rs` joined the composer list,
  and `Phrase::bare(&format!("prefix.{…}"))` registers its prefix as a
  dynamic section the way `locale.lookup` already did, so the next curated
  table keyed by its row is not silently orphaned.

  **The follow-up that is still not a tranche.** NINE converted sites hold
  a `", "`-joined list inside a `Slot::Text` — the seven already named,
  plus `stranded_solutes`' solute names and `phase_diagnostics`' phase
  list (which also carries an English `", and {n} more"` tail).
  `Slot::List` would join them in the reader's grammar, which is the whole
  reason the slot type exists, and it renders *a, b and c* where the
  `join` renders *a, b, c* — so converting them CHANGES the English and
  needs a golden pass of its own.
- [x] **I18N-11 — `scene_vessel` appended English sentences to the
  observation.** Not eleven: **twenty**. `appearance::observe` composes
  translatable clauses, and `scene.rs` then took the finished English
  `words` off it and pushed its own `format!`s on — osmosis (three
  directions), the gel, both coating films, corrosion, swelling, the
  luminol glow, enzyme conversion, both food-colour states, adsorption,
  partition, emulsion, the material layer, "the vessel contains", the
  curds, and both foam states. The `look` line does not go through
  `scene_vessel`, so the owner's quoted defect was already fixed; the WEB
  bench paints its caption and its accessibility text from the SCENE
  (`t(vessel.words)` in `Vessel.svelte`), so a German web reader met every
  one of those twenty in English.
  `SceneVessel` now carries `clauses` (the observation's, passed through)
  and `notes` (its own); `SceneVessel::say(locale)` recomposes both, and
  `words` is **generated** from them rather than written beside them.
  `scene::localize(&scene, locale)` is to the scene what
  `render::localize_events` is to the events, and the wasm boundary calls
  it — the only place that knows who is reading. `SceneCoating` and
  `SceneCorrosion` carry their own `phrase` because the web draws each as
  a separate SVG `<title>`.
  Two of the twenty already had German in `web/app/src/locales/de.json`
  (the coating films have no holes, so a whole-sentence lookup could reach
  them); the engine's rows take that German word for word rather than
  inventing a second one. `scene.material-layer` is the `look.coloured`
  decision again — *gelb* declines to *Schicht* and *orange* does not
  decline at all — so the German row puts the colour in brackets beside
  the noun, in the data. The scene golden was re-blessed and the diff is
  **11777 insertions, zero deletions**: every `words` string and every
  number in it is byte-for-byte what it was.

### Found while finishing I18N-10, and fixed in #642

- [x] **`Provenance.routing` was a finished English paragraph.** Done in
  #642. It is a `Phrase` now, in all four files that compose one: the
  aqueous router's three dataset choices and its activity-model caveat,
  the redox note and the second-speciation note in
  `finalize_solution_info`, the electrode pass in `displacement.rs`
  (which NESTS the aqueous routing inside its own clause instead of
  pushing a string onto it), and the two combustion routes. The field
  keeps its English — `routing` is the recipe rendered in the source
  language, byte for byte what it was — because it has consumers that are
  not readers, which is what made this one different from `Inert.why` and
  `NotYetModeled.what`:

  | consumer | what it needs | matches on English? |
  |---|---|---|
  | `tools/chemistry-audit/analyse.py` | files the string verbatim as a `routing_claim` beside the numbers it checked | no — it records, it does not branch |
  | `kero explain` (`kerotakis-cli`) | prints it under `routing:` | no; and see below |
  | the provenance drawer (`ProvenanceDrawer.svelte`) | prints `source.routing` verbatim | no |
  | `phreeqc/tests/provenance.rs`, `displacement.rs`, `ligand_reference.rs` | `contains("concentrated")`, `contains("Nernst")`, `contains("not a validated concentrated-mixture prediction")` | **yes, three of them** |
  | `solve.rs`'s ion-interaction check | `provenance.model.starts_with(ION_INTERACTION_MODEL_PREFIX)` | no — and this is the healthy shape: the structural decision reads `model`, never the prose |

  So nothing ROUTES on the sentence the way `StructureOracle::apply` did
  in #632 — the one place in the engine that decides something from a
  provenance reads `model`, which is a dataset's name and not prose. The
  three tests do, and they are why `routing` still carries English: a
  German `routing` would have broken them, and they are right to want the
  source language.

  **Two surface facts that are worth writing down, because the entry
  above was wrong about one of them.** The aqueous routing paragraph is
  *not* on the drawer today: the drawer reads `event.provenance`, and the
  only event that carries a `vessel::Provenance` is
  `ThermalEquilibrium` — the combustion/CEA one. The aqueous routing
  lives on `vessel.solution.provenance`, which reaches `kero explain`
  and the `inspect` machine contract and nothing a reader reads in
  German. So #642 translates what is reachable (`localize_event` now
  renders `ThermalEquilibrium`'s routing, which IS beside the numbers in
  the drawer) and makes the rest translatable the moment somebody wires
  it.

- [x] **`kero explain` answers in the reader's language. DONE 2026-09-18.**
  Seven labels through the catalogue under `[explain]`, in the ENGINE's
  `i18n/<lang>.toml` rather than a CLI one — a third file per language would
  break the rule that adding French is one core `.toml` and one web `.json`.

  **It also settled a contradiction between two records here.** `vessel.rs`
  listed `kero explain` among the consumers that "want the SOURCE language
  and would be wrong to get German"; this entry said `routing_in` was
  waiting for it. The owner ruled that `explain` is user-facing and answers
  in the reader's language, so it calls `routing_in(locale)` and the
  `vessel.rs` comment is corrected at its source. The machine consumers are
  untouched and do not read this function: the audit files the `routing`
  FIELD from JSON and the three tests assert on that field, while
  `routing_in(Locale::EN)` is byte-identical to `routing` by construction.

  `mcp.rs` passes `Locale::EN` explicitly, with the reason in the code: MCP
  is a machine protocol whose consumer is a tool that may parse the output,
  so its language must not change under it.

- [x] **`Provenance.dataset` and `.model` carry English PROSE, not names.
  DONE 2026-09-18, in #655.** The fifth instance of the family
  after `Inert.why`, `NotYetModeled.what`, `scene_vessel` and `routing`
  — and it was called the last one at the time, which the bullet below
  corrects: `ionic.rs` was still reading both of these fields in English.
  Visible the moment `explain` spoke German:

      Modell:  WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)
      ... mit wateq4f.dat plus USBM IC 9429 reference-temperature complexes,
          with the reviewed Sander HBr gas-uptake slice

  `wateq4f.dat` and `pitzer.dat` are NAMES and must not be translated. What
  is welded to them is a sentence — the reliability range in a parenthesis,
  the "plus …" and "with the reviewed …" clauses that say what was added to
  the dataset and why.

  **The same split `routing` got, for the same reason.** Both fields keep
  their English, because both have consumers that are not readers;
  `dataset_phrase` and `model_phrase` carry the sentence as a recipe, and
  `dataset_in`/`model_in` answer in the reader's language.
  `dataset_in(Locale::EN)` is byte-identical to `dataset` by construction:
  the field is FILLED by rendering the recipe in the source language, so
  there is one sentence and not two.

  **The consumer that made the byte-identity load-bearing rather than
  tidy.** `solve.rs::solvent_activity_of` routes the colligative answer on
  `provenance.model.starts_with(states::ION_INTERACTION_MODEL_PREFIX)` —
  the literal string `"Pitzer"`. It is not prose to that caller; it is a
  boolean spelled in English. A reworded model label sends every brine back
  to the ideal route and puts one molal salt water at −3.61 °C instead of
  the −3.44 °C the ion-interaction route gives, with nothing failing. The
  seam is pinned from both sides now:
  `ActivityModel::phrase` renders back to `describe()` exactly, and
  `describe()` still starts with the prefix.

  **A `Claim` rather than a `String`.** `Provenance::new` takes
  `impl Into<Claim>` for both fields, where `Claim::Name` is a name nothing
  translates (`NASA CEA thermo.inp`,
  `kerotakis:combustion:curated-fuels-v1`) and `Claim::Said` is a sentence.
  The old signature could not tell the two apart, which is how the defect
  lasted; a `&str` call site still compiles unchanged.

  **`dataset_file()` asks the recipe now.** It was a whitespace scan
  (#653), a guess that worked only because English puts the noun first. The
  file travels in a named slot, so the name comes back because it was put
  there — and the water route can say *vendored USGS phreeqc.dat* without
  LV1 announcing "vendored". The scan stays as the fallback for a bare name
  and for a session saved before the recipes existed.

  Eleven new catalogue rows, German authored; the engine lint stays at
  100% with no orphans, with `dbindex.rs` added to its composer list —
  `ActivityModel::phrase` is the only place the three activity-model
  descriptions exist, and a composer the lint cannot see is a blind spot of
  exactly the shape three were found in yesterday.

- [x] **`ionic.rs::provenance_of` builds the ionic drawer's source line in
  English. DONE 2026-09-18, ruled by the owner.** The SIXTH and last
  member of the family, and the one the five before it kept out of sight:
  `NetIonic.provenance` is `"{engine} · {dataset} · {model}"`, and the two
  halves that are sentences were read off the ENGLISH fields. A German
  drawer said

      Herkunft: PHREEQC (IPhreeqc, USGS) · llnl.dat · Pitzer
                specific-ion-interaction model (valid at high ionic strength)

  where it now says *… · Pitzer-Modell der spezifischen Ionenwechselwirkung
  (gültig bei hoher Ionenstärke)*.

  **The recipes were already there; the locale was not.** #655 put
  `dataset_phrase` and `model_phrase` on the provenance and gave it
  `dataset_in`/`model_in` to render them — and this function could reach
  neither, because nothing in `ionic.rs` knew who was reading.
  `net_ionic_for` and `net_ionic` now take a `Locale`. Emitting a `Phrase`
  and letting each host compose the line was the alternative, and was
  offered and not chosen: the line is a producer's, every host prints it
  verbatim, and a second composition surface is a second place to drift.

  **The separator is not translated and gets no catalogue row.** ` · `
  holds no word for a translator to change, and a row over punctuation is
  a row inviting one. The three parts are names or recipes; the join is
  neither. So this fix adds ZERO catalogue rows, which is also why the
  engine locale lint is unmoved by it.

  **The callers, named because #645 broke `main` by missing one.**
  `kerotakis-wasm/src/lib.rs` calls it twice (`step` and `runScript`) and
  passes `self.locale`, which is the session's; `render_ionic_for` in
  `render.rs` already had a locale and passes it through;
  `kerotakis-phreeqc/tests/equilibrator.rs` asserts on English and says so
  by passing `Locale::EN`. Clippy was run `--workspace --all-targets`, not
  `-p kerotakis-core`, which is the check #645 skipped.

  **`ION_INTERACTION_MODEL_PREFIX` is deliberately left alone**, and this
  is not the "second English-shaped boolean" the family hunts. It is a
  DECISION read off `Provenance.model`, never prose shown to a reader, and
  `states.rs` documents at length why it stays a prefix test: `kerotakis-
  core` sits below the crate that knows which dataset is which, and
  inverting that dependency to type one boolean is the larger mistake. The
  seam is pinned from the other side instead —
  `only_the_ion_interaction_model_carries_the_prefix_core_matches_on` in
  `kerotakis-phreeqc` fails if the Pitzer description is reworded. A typed
  activity basis on the wire would be an additive protocol field and a
  ruling of its own, not a rider on a locale thread.

- [x] **Wire the vessel's own provenance to the drawer. DONE 2026-09-18,
  in #653.** The aqueous routing — which dataset answered this beaker and
  why — was the one a learner would most want and the one the drawer could
  not see. `Event::SolutionRouted` carries it now, and the drawer prints it
  through the reader it already had.

  **The owner settled the shape: an event at characterisation time.** It
  appears when the vessel is characterised and replays from the log like
  every other event, which is how everything else reaches the web — the
  engine emits, the host renders, and the sealed-unknown mask applies to it
  like any other line. The alternative, the drawer reading the inspected
  vessel, would have needed a second path that no replayed transcript could
  reproduce.

  **The crux was the emission rule, not the wire format.** The aqueous
  solver characterises far more often than a reader would want an event:
  the three commands of `aq-023` reach `finalize_solution_info` FIVE times,
  because the thermal fixed point in `equilibrate` re-solves a CLONE of the
  vessel until the temperature settles and keeps only the last pass's
  events. An event per solve would bury the log and tell a learner nothing:
  the news is never *the solver ran*, it is *the answer to this beaker now
  comes from somewhere else*.

  So the rule is: **fire when the engine, the dataset, the model, or the
  SHAPE of the routing recipe differs from what this vessel was last
  announced as.**

  **What it costs, measured** — by building the same binary with the rule
  flipped to fire on every characterisation and running both:

  | script | naive (one per characterisation) | on change |
  |---|---|---|
  | `aq-023` (3 commands, 1 beaker) | 2 | **1** |
  | `cabbage-rainbow.lab` (59 lines, 5 beakers) | 10 | **6** |
  | `rusting.lab` (55 lines) | 6 | **2** |
  | `buffer.lab` | 4 | **2** |
  | `electrolysis.lab` | 2 | **1** |
  | `yeast-fermentation.lab` (59 lines) | 1 | **1** |
  | **every lesson in `lessons/`** | **452** | **190** |

  Two numbers here are worth reading carefully, because they correct the
  note above. **Five CALLS into `finalize_solution_info` is not five events
  even under the naive rule**: the fixed point keeps only the surviving
  pass's events, so three of aq-023's five calls are discarded trials and
  the naive rule puts two on the wire. And `yeast-fermentation` shows the
  floor: a lesson with one beaker on one dataset states its routing once
  and says nothing more, under either rule.

  The six in `cabbage-rainbow` are the case that makes the rule worth
  having — **five beakers and six statements, every one of them news**:

      v1  minteq.v4.dat   the problem needs chemistry the default dataset lacks
      v2  wateq4f.dat     the default inorganic aqueous dataset
      v3  wateq4f.dat     …and a SECOND dataset was asked for the solvent's activity
      v4  pitzer.dat      concentrated (~1.1 mol/kgw), where ion-interaction is valid
      v5  pitzer.dat      concentrated (~1.0 mol/kgw)
      v5  wateq4f.dat     …and back, once it was no longer concentrated

  The four the naive rule adds are four repetitions of a sentence that was
  already on the screen.

  Over the whole corpus the naive rule puts **452** routing lines on the
  wire — one per characterisation, which is why it tracks
  `solution_characterized`'s 445 so closely — and the change rule puts
  **190**. Fifty-eight per cent of them said nothing that was not already
  true, and the ones that remain are about three per lesson.

  Three things are worth writing down, because each of them is a place a
  simpler rule is wrong.

  * **The shape, not the sentence.** `Phrase::shape` is the recipe's keys,
    its nesting and every name in it, with the measurements taken out. The
    concentrated-brine route says *chosen because the solution is
    concentrated (~16.0 mol/kgw)*, and that number moves with every
    spoonful of salt. Comparing rendered text would announce a routing
    change on every step of exactly the lesson where a learner is adding
    salt. A number moving inside a reason is not a new reason. Text slots
    are kept, though — the second-speciation clause names the FILE it asked
    for the solvent's activity, and a different file is a different answer
    however alike the two sentences read.

  * **Compared against what was SAID, not against `solution.provenance`.**
    The live provenance is not a record of what a reader was told: the
    electrode pass in `displacement.rs` nests its own clause into it after
    the aqueous solver has finished, with a computed activity in the
    sentence that moves every step. Comparing against it would have fired
    on every step of every metal lesson. `Vessel::aqueous_routing_said`
    holds the key of the last ANNOUNCED routing and only
    `finalize_solution_info` writes it. It is `#[serde(skip)]` — narration
    state, not chemistry — so no saved session and no vessel dump changes
    shape for it, and the one visible cost is that resuming a save
    re-states the routing once, which is the right thing to say to a reader
    who has just opened the file.

  * **On the vessel, not in the solver.** The fixed point solves a clone up
    to eight times and keeps the LAST pass's events. Memory held in the
    equilibrator would have announced on the first pass and been silent on
    the pass whose events actually survive — the event would have vanished
    entirely. Cloned with the vessel, every trial reaches the same verdict
    and the surviving one carries it.

  **The web needed the other half of "on change".** `provenance.ts` already
  lifted `event.provenance` off ANY event, so the new one reaches
  `report.sources` with no new reader — the wiring was already there and
  waiting for an event to exist. But a step that solved and said nothing
  about its routing has not LOST one, and a drawer that showed a dataset on
  the step that changed it and nothing on the nine after would read the
  engine's quietness as an absence of provenance. So the session carries
  the last routing per vessel and the drawer marks it *stated on an earlier
  step, still standing* rather than passing it off as this step's work. It
  is cleared at exactly the two places `latestStep` is cleared, and for the
  reason written there: a routing read against a bench state that a
  different script produced is the corpus route-leak bug on a screen.

  **One fixture was found to be fiction.** `provenance.test.ts` hung its
  aqueous provenance on a `precipitated` event, and `Event::Precipitated`
  has no provenance field and never had one. The drawer's aqueous source
  existed only in that file. It is a `solution_routed` fixture now, which
  is a shape the engine emits.

  **No golden moved, and that is a fact rather than luck.** The three
  checked-in engine goldens — `lessons.json` (ARCH-001), `scene-five.json`
  (GUI-003) and the codex export — all replay on `Bench::default()`, the
  deliberately engine-free core bench, where `finalize_solution_info` never
  runs and so no routing is ever announced. `kero coverage curiosity
  --check` over all 500 corpus rows reports **baseline drift: 0**: the new
  event moved no disposition and no reason code, which is the gate that
  would have caught it if the extra event had tipped a row from `missing`
  to `computed` by giving an otherwise silent solve something to count.

  Not done here, and deliberately: the MIX path
  (`routing.mix-by-fraction`) and the solvent-only analytic path
  (`routing.solvent-relation-only`) still write a provenance without
  announcing one. Both are reachable and neither is wrong to leave — the
  solvent-only path returns an empty event list ON PURPOSE, so that routine
  water setup is not filed as a computed answer, and giving it an event
  would move coverage rows for a reason that has nothing to do with
  provenance. Neither disturbs the memory, so a later ordinary solve is
  still compared against the last thing a reader was actually told.
  **Ruled back in by the owner on 2026-09-18; see the bullet below for
  what the coverage objection turned out to be.**

- [x] **MIX and solvent-only characterisation announce their provenance
  too. DONE 2026-09-18, ruled by the owner.** Both paths wrote a
  `Provenance` into `vessel.solution` that no event carried, so a beaker
  filled by combining two others — or one holding only solvent — held
  provenance no reader could reach. Announcing on MIX only was offered and
  not chosen.

  **What reaches the user.** A pour of two solutions narrates, after the
  line saying they were combined:

      v3: berechnet mit wateq4f.dat, ergänzt um Referenztemperatur-Komplexe
          aus USBM IC 9429, … — MIX: zwei gelöste Lösungen anteilig gemischt

  and a beaker of plain water, once:

      v1: berechnet mit phreeqc.dat, wie vom USGS mitgeliefert — keine
          dargestellte Säure, Base, kein Salz, … die Lösungsmittelbeziehung
          wurde ohne Aufruf von IPhreeqc ausgewertet

  Neither is a repeat of anything: the MIX routing is a sentence no other
  path composes, and the water relation is answered by a different ENGINE,
  a different dataset and a different model from any solve — it never
  invokes IPhreeqc at all.

  **One rule, one implementation.** `aqueous.rs::announce_routing` is the
  fire-on-change test above, lifted out of `finalize_solution_info` and
  shared by all three paths. Because they share
  `Vessel::aqueous_routing_said`, a beaker that leaves one path for another
  is told about the MOVE rather than told the same thing twice.

  **Solvent-only is most characterisations, which is what made the rule
  load-bearing rather than tidy.** The water relation answers a beaker
  every time the stack runs over it, with the same engine, the same file
  and the same reason every time. It is news exactly once — at the step the
  water is first characterised — and silent for the rest of the lesson.

  **The coverage objection was real, and it was about a PROXY.** The bullet
  above is right that an event here would have moved curiosity rows, and
  right about why: a route's `event_count` was the raw length of the
  solver's event list, and `kero coverage curiosity` reads a Computed route
  with events as *this solver produced an answer*. A beaker of plain water
  has `chemistry_applies == false`, so it would have been filed as a
  computed result **for saying which relation answered it** — a claim that
  is simply false, and the baseline gate would have failed on it.

  What was wrong was not the line; it was the proxy.
  `solve::answer_event_count` counts events that are an ANSWER, and a
  routing announcement says WHERE an answer came from rather than being
  one. It is a no-op on everything that shipped before it: the only
  producer of `SolutionRouted` is the aqueous solver, which until now
  emitted one only on the path where `chemistry_applies` is already true,
  and coverage's chemistry branch does not look at the count at all.

  **The goldens cannot see this, and that is the same fact as above.** All
  three checked-in engine goldens replay on `Bench::default()`, the
  engine-free core bench, where the aqueous solver — the only producer of
  the event — is not wired. `lessons.json` contains zero routing lines
  today despite #653 announcing on the direct path for every salted beaker,
  which is the measurement, not a hope.

- [x] **The `", "`-joined lists are `Slot::List` now.** Done in #642.
  **The count, settled from the source: eight sites**, and the two
  numbers on record were each half right. A paren-matching scan over
  every `crates/*/src/**/*.rs` with test modules removed — the shape
  `tools/engine-locale-lint.py` uses, because a line-window scan is what
  produced the disagreement — finds exactly eight `Slot::text(…join…)`:

  | site | list | items |
  |---|---|---|
  | `bench.rs` co-evaporation | `other_liquids` | display names → `Slot::terms("species", …)` |
  | `bench.rs` no distribution coefficient | `outside` | `SpeciesId` → `Slot::texts` |
  | `bench.rs` extraction without a coefficient | `outside` | `SpeciesId` → `Slot::texts` |
  | `bench.rs` incomplete absorbance | `gaps` | `Slot::terms("species", …)`, matching `appearance.rs`'s `look.spectral-gap`, which already rendered the same list that way |
  | `bench.rs` no group decomposition | `names` | `SpeciesId` → `Slot::texts` |
  | `bench.rs` no curated nuclide | `known` | notation → `Slot::texts` |
  | `bench.rs` no curated reaction | `ORG_REACTIONS` | row names → `Slot::texts` |
  | `solve.rs` `stranded_solutes` | `names` | display names → `Slot::terms("species", …)` |

  The roadmap's earlier scan named five and flagged two more as needing
  individual reading; both of those were real, and it missed two others
  outright (`outside` in the no-distribution-coefficient branch, and the
  curated organic reactions) because in both the `.join(", ")` sits
  several lines below the `Slot::text(` that contains it. **The I18N-10
  report's nine was the honest number** — it is these eight plus the
  `particles.rs` tail, which is the one item on the old list that is not
  a `Slot` at all. So: nine things named, eight of them `Slot::text`
  sites, and the disagreement was entirely about whether the tail counts.

  Three constructors carry the pattern now — `Slot::list`, `Slot::texts`,
  `Slot::terms` — so the next call site is one line and cannot reach for
  a `join` by accident. Only one golden line moved:
  `lessons.json`'s stranded-solute sentence gained the word *and*.

- [x] **`particles.rs` speaks the reader's language. DONE 2026-09-18.**
  The entry below was right that the tail could not be converted alone, and
  right about what it would cost. `Census::render` takes a `Locale` now, and
  the whole drawing moved with it: the two elision captions, the scale line,
  the water aside, the inventory footnote, `Kind::describe`'s six words in
  parentheses beside every row, and the species names — those reuse the
  `species.*` rows the engine already ships rather than a second copy of
  them. The tail renders through `census.and-more`, so its separator and its
  count phrase are the language's business.

  **It also predicted the lint failure, and it happened twice.** Adding
  `particles.rs` to the composer list was not enough: the composer scan
  looked only for `Phrase::new(…)`, so six `locale.t` keys came back as
  ORPHANS; and once those were seen, the six `census.kind.*` chosen by a
  match arm came back as orphans too, because `MENTIONED` — the mechanism
  for exactly that shape — was applied to `render.rs` alone. Both widened.
  Reachable keys 570 → 576, de 100.0%, match-arm keys 12 → 18.

  The test is in `kerotakis-phreeqc`, not core, and that is the other
  finding: a `Bench` with no equilibrator never speciates, so trace salts
  stay solid, never enter the census, and **nothing is elided** — a
  core-only test would have asserted the captions of a picture the product
  never draws.

### Found while resolving the bench gate against GUI-105

- [x] **Curated scripts ask for a fresh bench. DONE 2026-09-17.** Two
  pieces of work converged on the same answer for a dirty bench — the
  lesson gate (#641) and GUI-105's replacement for the capability explorer
  (#638) — and did not converge on the same *test*: the lesson path asked
  `benchDiffersFromFresh`, the catalogue path `runGate`, which was
  `benchOccupied` alone.

  Measured before changing it. The engine **refuses** `add v2` on a bench
  with no `v2` (*"no vessel v2 — make it first with `new`"*), so no script
  can reach past what it allocated. What it can do is land in the wrong
  vessel: **117 codex routes allocate with a bare `new` and then name the
  result absolutely as `v2`**, so with a leftover EMPTY `v2` present, `new`
  hands back `v3` and `add v2 water` fills the previous run's glassware —
  right number, wrong vessel, of whatever type that run left behind. All
  500 corpus prompts open `add v1 …` and have the same exposure.

  `runGate` asks `benchDiffersFromFresh` now, because **both of its callers
  run curated scripts**. `benchOccupied` stays, and its doc says what it is
  still the right question for — a script that only writes into glassware
  it can see, of which this app currently has none.

### Two defects found in the same playback, neither of them i18n

- [x] **The bench is not reset between lessons.** "Lektion begonnen: Elektrode"
  is followed by water going into `v1` and the *previous* lesson's Natron and
  Citronensäure dissolving in it. The electrode lesson then computes its pH and
  its driving force from a vessel that is holding another lesson's reagents.
  Every number it reports after that point is wrong, and it will reproduce
  whenever two lessons are played in sequence.
  *Closed. Every `.lab` in `lessons/` numbers its glassware from `v1` with no
  gaps — there is a test for that now — so all 113 are written for the bench
  the engine hands over, one empty vessel, and `startLesson` never checked.
  The bench is a precondition now, and an unmet precondition is a QUESTION:
  `requestLesson` holds the lesson at the door and asks, in the same shape the
  catalogue's own run gate has always used. Clearing goes through the confirmed
  `clear()` and leaves its note in the feed; keeping starts anyway and records
  that the readings are not the lesson's alone; cancelling touches nothing.
  Nothing is discarded silently, which is the principle the disposal station
  and the remove-vessel dialog exist for. The condition is* not *`benchOccupied`
  — a leftover EMPTY beaker is unoccupied and still wrong, because `new` then
  returns `v3` where the lesson says `v2`. "Fresh glassware beside it", the
  catalogue's third option, is scoped out rather than forgotten:
  `canUseFreshVessels` already refuses any script that allocates its own
  vessels, which is 94 of the 113, and the renumbering prelude would shift the
  `#@part.*` prose labels off the steps they annotate. The same gap was open in
  `CapabilityExplorer.run()`, where all 500 corpus scripts open `add v1 …`
  against an assumed empty beaker and were fired at the bench with no gate at
  all; it uses the same dialog now. `importLab` is left alone deliberately —
  its contract is that an import COMPOSES onto the bench you can see, and the
  file is one the learner chose themselves.*
- [ ] **Two locale keys disagree across `terms` and `messages`.**
  *This entry corrects an earlier claim of mine, made 2026-09-16 and wrong:
  I reported "59 duplicate keys" in `de.json` by counting key/value pairs
  across the whole file. Neither section contains a duplicate key. The 59 are
  keys carried in BOTH `terms` and `messages`, and the merge in
  `i18n.svelte.ts` lets `messages` win on purpose.* What is real is smaller
  and more specific: of those 59, **two carry different German**, so one
  translation does silently replace another —
  `limewater` is *Kalkwasser* as a term and *Kalkwasserprobe* as a message,
  the substance against the test for it, and `invisible ink boundary` is
  *Grenze der unsichtbaren Tinte* against *Unsichtbare Tinte*, which drops
  the boundary the lesson is about. Decide which each should be, then keep
  one. **The lint half is DONE 2026-09-18**: `tools/locale-collision-lint.py`
  counts the real collisions and fails when one carries two different
  translations, wired into `preflight.sh` with a self-test. The two above are
  recorded in it with what each side means, so a THIRD fails the build while
  the wording decision stays open — and a recorded one that heals fails too,
  because a stale exemption is a lint that has quietly stopped checking. The
  merge comment said "six keys actually collide" and now says 59 and why.
  **What is left is the decision**: which of *Kalkwasser* / *Kalkwasserprobe*
  `limewater` is, and whether `invisible ink boundary` keeps the boundary in
  its name.

## Re-measuring the catalogue window (#650)

The virtualisation (#650) was justified by numbers, and the harness that
produced them is left on disk so the next person can reproduce rather than
trust them:

    /mnt/volume1/tmp-overflow/kero-build/catwin-site   (13 MB)

    CHROME_PATH=~/.cache/ms-playwright/chromium-1234/chrome-linux64/chrome \
      node --experimental-websocket tools/test-ux-quality.mjs <payload>

**It needs no cargo** — it is the checked-in golden codex export plus the
python index tools (`curiosity-index.py`, `kids-catalog.py`,
`lessons-index.py`, `step-prose.py`) and a prebuilt `kerotakis_wasm`. That
matters because the box it was built on takes ~40 minutes for a cold Rust
build and the measurement takes seconds.

What it measured, for comparison if anything regresses: 730 cards in the
payload (230 experiments + 500 questions; production ~752), **730 → 12**
`<article>` elements in the DOM, 12,191 → 1,125 elements in the dialog, and
open-to-visible **3,921 ms → 911 ms** at a 4× CPU throttle. The trade, also
measured: a synthetic fling at 646 px/frame is *worse* windowed on a
throttled CPU, 52.7 against 39.4 ms.

`/mnt/volume1/tmp-overflow` is scratch and may be swept; if it is gone, the
recipe above rebuilds it.

## Two doors onto one question (GUI-105) — landed 2026-09-17

The app had two catalogues and the reader had to know which was which. The
split is real — an experiment is something you *run*, a corpus row is a
question with a *reviewed answer*, and flattening them would claim the
engine can run 500 experiments it cannot — but it was an author's
distinction, not a reader's. Nobody arrives asking which kind their
question is; they ask **"can it do this?"**, and that had to be asked in two
places or it got a wrong "no".

One index, typed facets: 131 codex routes, 121 guided experiments and the
500 reviewed corpus questions in one list, one query, one matcher, with the
kind as a badge and a facet chip. The counts stay separate and are derived
from the rows, so the headline reads **"252 experiments and 500 answered
questions"** rather than a false "752 experiments" or the old "208". The
detail, and the three defects the move exposed, are in `HISTORY.md`.

### The list that index produced is now a window — 2026-09-18

One index of 752 rows drew 752 cards. Measured in Chromium over the built
payload (730 rows there: 230 experiments, 500 questions): **12,191
elements in one dialog**, and **3.9 s** from pressing *experiments* to the
list being on screen with the CPU throttled 4× — the 2018-class target the
budgets above are written for. The dozen cards that fit a viewport cost
**1,125 elements** and **0.9 s**.

So the list draws a window: the grid rows near the viewport plus three
rows of overscan, absolutely placed inside a container stretched to the
library's height. Rows are measured as they are drawn rather than assumed,
because a refused question is three lines and a guided experiment with a
kit is twenty; a row nobody has visited uses the running mean, so the
scrollbar is up to ~28% long ahead of the reader and exact to 4 px in
90,000 behind them. Wheel scrolling stays at one frame a step; a fling at
646 px per frame costs more on a throttled CPU than the old list, which is
the trade.

Windowing is only safe because the in-app search greps the DESCRIPTIONS —
and the audit that established it found **three places where it did not**:
the safety rationale 58 guided cards print, the German procedure and
observation lists 22 of them ship, and the refusal reason all 500
questions draw through `t()`. All three are in the haystack now and
`catalogEntry.test.ts` fails loudly if any is taken back out, because
`Strg+F` is no longer there to cover for it.

## The journal, after the owner used it (GUI-107)

One of five things the owner hit in the German deploy. They are two
different complaints wearing five hats — *the chrome is eating the
content* (GUI-107, GUI-108, GUI-110) and *the labels are not telling me
anything* (GUI-109, GUI-111) — and each is recorded with the surface it
belongs to rather than in a section of its own: GUI-108 beside the result
card, GUI-109 … GUI-111 with the cupboard.

- [x] **GUI-107 — The Laborbuch spends two rows saying who it is.** The
  owner: *"the '≡ / >_ / ›' in Laborbuch can we move up into the top '≡
  Laborbuch 2 ›' row to save screenspace"*.

  Done. The pane heading and the journal's control row were two separate
  components' idea of the same thing: `App.svelte` drew `≡ Laborbuch 2 ›`
  and `Feed.svelte` drew a second row inside the log with the view toggle
  and the note chevron. They are one row now — `JournalHeader.svelte` —
  ordered identity, then what to look at, then what to add, then the pane's
  own collapse, which is the reading order and the tab order both.

  **What the work found:** the two rows could not simply be concatenated,
  because the controls and the state they drive live in different
  components and Svelte scopes styles to whoever wrote the markup. Three
  consequences worth recording. (1) `showTrace` and `composing` are
  `$bindable` props of `Feed.svelte` now, held in the shell: the button is
  one component up from the thing it switches, and two siblings can only
  share a value through the parent that holds both. (2) The pinned-tip
  behaviour — a tap pins the label, because a tap is the only "hover" a
  touch screen has — had to move WITH the buttons, which is the reason this
  is a component and not ten lines of markup in a 2800-line `App.svelte`.
  A `title=` alone would have been a silent accessibility regression on
  every phone. (3) The composer FORM stayed in the feed while its chevron
  went up: the textarea is part of the notebook and belongs beside the
  entries it is about to join.

  At 320 px the row is icon + name + count + two view buttons + chevron and
  nothing else — the collapse chevron is already `display: none` below 980
  px, because a tab bar gives the journal the whole screen there and a
  collapse control that cannot do anything is not worth its width. The name
  is the only elastic cell, so when the pane narrows it is the WORD that
  ellipsises and never the controls.

## Asked for, and deliberately not built here (GUI-112, GUI-113)

Both are the owner's, both are larger than the five above, and neither is
started. Recorded so they are not lost.

- [x] **GUI-112 — A command prompt bar in the GUI, with context-aware
  autocomplete.** First slice done: verbs, vessels and chemicals, each in
  the reader's own language, with the context read out of the grammar.
  `completions.ts` is the model, `CommandBar.svelte` is now an ARIA
  combobox over it.

  **What the work found — the completion model was already in the engine,
  in a place nobody had read it as one.** Every source turned out to
  exist, and none of them had to be hand-written:

  - **The verbs** are `kerotakis_core::script::VERBS`, reaching the app
    through `Lab::grammar()` — the same inventory the protocol
    conformance suite already checks `affordances.json` against, so a verb
    the bar offers is a verb the parser has. Each row carries a canonical
    example and (I18N) a `typed` line spelled by the engine's own alias
    layer. **The first word of `typed` is that language's word for the
    verb.** Nothing in the web app knows the word *erhitze*; it reads it
    off the engine's own example. A language added later brings its verbs
    with it, which keeps "one core toml + one web json, no code" true.
  - **The vessels** are the scene's `vessels[].id`, which is what `vN`
    counts, with the scene's own `label` as the hint so a suggestion says
    *Becherglas* and not only *v1*.
  - **The chemicals** are `session.shelf` — the registry the cabinet
    already has. Matched on the German name *and* the key, inserted as the
    **key**, because a species alias may or may not exist in every
    language and a suggestion that does not parse is worse than none.

  **And the context — "insofar as possible" — is the example line itself,
  read as a positional schema.** `VERBS` says `add` is
  `add v1 water 100mL`: position 1 is a vessel because the example has a
  vessel there, and position 2 is a chemical because the example has a
  shelf species there. `stock NaCl 0.5mol` puts its species at position 1
  instead, and nothing in the app knows that either. This is the one idea
  in `completions.ts`; it needed no new engine surface, it cannot drift
  from the grammar it is read out of, and a verb added tomorrow arrives
  with its own schema. `affordances.json` was examined and is not a
  completion source — it is English prose about which GUI surface invokes
  each verb — but it is the file that proves the verb list is total.

  **What is deliberately left for a second slice**, and why:

  - **Arguments that are amounts.** Where the example has `100mL`,
    `0.5mol`, `500rpm` or `254nm`, the bar says nothing. Offering a guess
    at *how much* somebody meant is a claim about their experiment, not a
    completion of their typing; `amounts.ts` already knows register-aware
    quick amounts and could supply them, but only against a vessel and a
    phase the bar does not yet track.
  - **Joining words** — `until`, `stages`, `from`, `to`, `on` — which the
    schema can see but which are worth offering only together with the
    clause they open.
  - **`react`'s curated reaction names**, which `session.reactOptions`
    already carries from the same grammar call: the example is
    `react v1 esterification`, so the slot is a name from a list rather
    than a shelf species, and it needs a fourth completion kind.
  - **Fuzzy ranking and history-based prediction**, which were out of
    scope by instruction and are worse than prefix matching for a reader
    learning a vocabulary: a list that reorders itself is a list that
    cannot be learned.

  The popup is an ARIA combobox — `aria-expanded`, `aria-controls`,
  `aria-activedescendant` resolving to an element that exists, arrows,
  Escape, Enter — with every row over 44 px, because this repo audits
  both. It starts closed: a wall of every verb the moment the console
  opens is not a feature. `verbExamples` is gone; it was a flat list of
  finished lines for a `<datalist>`, and it threw away the two thirds of
  each grammar row the model needs.

- [x] **GUI-113 — Set the energy amount directly for `Erhitzen`.** Done. The
  flame panel now has an *Energie aus* switch: **Flamme und Einwirkzeit**,
  which is what it always did, or **einer Zahl, die ich eingebe**, which
  hides the flame/collar/exposure controls and takes the kilojoules
  directly. The readout labelled *zugeführte Energie* was already there —
  this makes that same number an input.

  **What the work found.** Three things the one-line estimate did not
  anticipate.

  (1) **The German decimal comma is a real defect and it is invisible to
  every unit test in `web/app`.** A `<input type="number">` is parsed by
  the BROWSER against the BROWSER's locale, which is not the locale this
  app is speaking. A German reader on an English-locale Chrome who writes
  `40,5` hands the app the empty string: the field looks like it holds a
  number, and the run button greys out saying nothing. Nothing in this app
  parsed typed decimals before — `amounts.ts` and `stepAmount.ts` only
  ever *produce* numbers, and every displayed value goes out through
  `Intl.NumberFormat` — so there was no house parser to follow. The field
  is therefore `type="text" inputmode="decimal"` and `num()` in
  `apparatus.ts` reads both separators, which makes every apparatus field
  comma-tolerant rather than just this one. A lone comma is always the
  decimal point (`1,234` is 1.234), which is only safe to decide because
  this field's ceiling is 150 — there is nothing here for a thousands
  separator to group; a four-digit field would have to ask. Two separators
  is a typo and is refused. Whatever is typed, the grammar is sent a
  POINT.

  (2) **A form of fields that are all always shown can only offer one way
  of saying a thing.** `FormField` gained `when?(values)`, so a mode's
  controls hide when the other mode owns the answer — a flame slider
  sitting under a typed energy reads as if it still did something. Hidden,
  never dropped: `values` keeps every field, so switching back finds the
  flame where it was left.

  (3) **The ceiling had to come from the panel's own bounds, not from a
  round number.** `BUNSEN_MAX_KJ` is `0.005 × 100 × 300 × 1` — full flame,
  open collar, the longest exposure the exposure field accepts — so the
  limit on a typed energy cannot drift away from the fields it is the
  limit of. A test asserts the derived path at those settings produces
  exactly that number, and that the refusal sentence quotes it.

  The refusals say which thing is wrong (unreadable, not an amount, more
  than this flame has) rather than greying the button in silence, and an
  empty field is not yet a mistake. The browser-level half — that a comma
  survives a real keystroke in a real Chrome — is in
  `tools/test-ux-quality.mjs`, because that is the only level at which it
  is demonstrable.

- [x] **GUI-108 — The latest-result card takes the journal it summarises.**
  The owner: *"the 'Neuestes berechnetes Ergebnis · v1 / Temperaturänderung
  / ΔT +25,97 K / ⌖ ⤓ ×' modal overlays over the text in Laborbuch"*.

  Done. `.result-card` is `flex: 0 1 auto; min-height: 0` and its body is a
  capped scroll region, so the card takes what is left of the pane and
  never what is under it. The disclosure is remembered per browser, so a
  reader who wants the headline and not the detail says so once.

  **What the work found:** it is not an overlay, and looking for one is the
  wrong search. The card has no `position`, no z-index above its own export
  menu, and no scrim — which is exactly why it never appears in
  `overlayStacking.test.ts`, whose contract is about `position: fixed;
  inset: 0` surfaces and the Escape chain. **No z-index was changed and no
  recorded exception was added or healed.**

  The real mechanism is a flex column. `.pane-body` stacks the vessel
  inspector, this card and the feed; the feed is `flex: 1; min-height: 0`
  and the card was `flex: none`. One item that would not give way, beside
  one that would give way entirely — so an `<details open>` carrying an
  equation, a reactant list, an observation, a thermal row, a quantity
  table, a boundary note and a safety note could squeeze the log to zero
  height. To the reader that is indistinguishable from being covered.

  `Inspector.svelte` is the precedent, one component earlier in the same
  column, and its comment already said the whole thing: *"The instrument
  and action rows can be taller than their share of a narrow journal. Keep
  them in their own scroll region so they never paint over (or steal
  pointer events from) the notebook below."* The card now does the same.

  Two constraints shaped the implementation. The cap is `30vh` on the BODY
  rather than a percentage on the card, because `<details>` cannot safely
  be made a flex or grid container — the closed state depends on the UA's
  own display handling — and the journal pane is very nearly viewport
  height in every layout the app offers. And the summary row is left out of
  the scroll region so the export menu, which hangs off it, is not clipped.

  Known and deliberate: with the vessel inspector open as well (capped at
  55% of the pane) the log is thin. Both blocks scroll and neither can
  reach zero, and a reader in the inspector is not reading the log — but it
  is a compound case worth naming rather than claiming it is solved.

## The bench must look like a bench (GUI-114 … GUI-116)

Stated by the owner on 2026-09-20, in one breath, after using the app:

> it is counterintuitive that we can have devices floating not on a desk.
> we have not realistic rendered animations for most things that go on. for
> kids, it would be cool to have really impressive animations for some of
> the most standard experiments like soda vulcano etc. lage amounts of
> foam, steam, explosions, etc, visibly rendered, but their extent relative
> to modelled parameters. really drawn devices not text or schematic
> symbols no one understands. a real cupboard of devices not a wall of
> text.

**The clause that governs all three of these is "but their extent relative
to modelled parameters".** It is the same rule the 2026-08-27 product
directive already states as its sixth point — motion and visible effects
driven by computed power, temperature, RPM, viscosity, fill, particle
settling, reaction energy, gas/solid amount and time, *never* generic
animation — and it is what separates this work from decoration. A foam
column that is always the same height is a lie told beautifully, and this
engine's whole claim is that it does not tell those. Half a litre of
evolved CO₂ and half a mole of it must not look the same.

- [x] **GUI-114 — Apparatus stands on something.** *Floating was a missing
  surface, not a missing relationship.* What the work found, in the order
  it found it:

  - `.work-surface` names a surface and paints nothing: it is a bare
    `position: relative` box whose only job is to be the coordinate space
    `.vessel-position` is absolutely positioned in. `.bench` *did* paint
    "the counter the glassware stands on" — as a 2.6 rem lip along the
    very bottom edge of the scroll area, under a wall gradient covering
    the top 58%, with **nothing in between**. Vessels live at y 0.28–0.84
    of the work surface, which is exactly that nothing. Hence: devices,
    floating.
  - The scene says *where* a thing is and *what it is attached to*, and it
    has said both all along. `benchLayout.ts` stores a free (x, y) per
    vessel and per workstation; `apparatusTarget.ts` and the
    `deployedTool`/`deployedTarget` pair say what is installed on which
    vessel. Neither of them knew anything about "on".
  - The heater's relationship to its vessel was **already drawn**.
    `DeployedApparatus` renders a hotplate, stirrer, cooling bath and
    burner inside the vessel's own `0 0 100 140` viewBox with their base
    below the glass, and `ApparatusAssembly` names the parts on top of it.
    Freestanding workstations (mortar, centrifuge, burette, wash bottle,
    evaporating dish) stand beside their vessel with a drawn route and a
    `v1` badge. Nothing there needed inventing.

  So the change is a bench top and a set of feet. `BENCH_DECK_TOP` is the
  counter's far edge; `.bench-deck` draws the counter from there down to
  the front lip, behind everything that stands on it. `footingY` says
  where each stack *touches* that counter — the glass base normally, the
  appliance's base when a hotplate, stirrer, bath or burner is carrying
  the glassware — and a `.bench-footing` contact patch is drawn there. The
  mortar and the centrifuge had no contact patch at all and now have one;
  the evaporating dish, the wash bottle and the retort stand already drew
  theirs and now share the name. A workstation could be parked at y 0.12,
  which is up the wall: `APPARATUS_Y_MIN` is the deck edge now.

  Proved by measurement, in `tools/test-ux-quality.mjs` at 1440 px and at
  320 px: the deck exists inside the work surface, runs its full width (at
  320 px that is the surface's own 42 rem scroll width, not the viewport)
  and reaches the front; there is wall above it; and every vessel's
  contact patch lies inside the deck, at the foot of its own drawing
  (below 78% of the drawing's height) and horizontally under it.

  Left standing, deliberately, because the scene cannot yet say it: a
  burner is drawn *behind* a beaker whose base is at y 127 of a 140-unit
  box, not underneath it. Putting it truly below what it heats means
  lifting the glassware onto a tripod, which means making room in the
  vessel's own viewBox — a change to every drawn effect's coordinates, not
  a change to this layer. And the browser audit does not press a hotplate
  into place, because the only route to a deployed appliance is
  `ApparatusForm`, which GUI-112/113 is rewriting as this lands; the
  carrying rule is held by `benchFooting.test.ts` until that settles.

- [x] **GUI-115 — A cupboard of devices, not a wall of text.** *The
  picture was there, correctly drawn, and 11.7% of its own tile.* What the
  work found, in the order it found it:

  - **There were two small boxes, not one.** The tile was 118 x 84 and the
    portrait inside it 34 x 34, which is the 11.7% the item was opened
    for — but the `<svg>` inside THAT was 24 x 24, so the drawing itself
    was 5.8% of the tile. A padded container around a small picture is the
    same defect twice, and a share measured off the container would have
    flattered the fix. So the compartment is square and the drawing fills
    it: `.item-render` IS the picture's box now, and the number below is
    the picture.
  - **The 44 px (i) was the binding constraint, not the name.** GUI-110
    put a 44 px hit square in the tile's top-right and audits that the
    drawing never overlaps it. On a 118 px tile that is a quarter of the
    width, and a drawing that fills the width cannot clear it — so
    "make the drawing bigger" and "keep the touch floor" looked like a
    choice between two things the repo had already decided.

    They are not in conflict once the tile stops being a column and
    becomes a compartment. The name moved UP into a plate across the top
    whose `min-height` is exactly the 44 px the (i) needs, and the device
    stands below it. The (i) still costs a corner — it costs the PLATE's
    corner, which was going to be empty. `infoOverDrawing` is still zero
    and now says something true rather than something arranged, and
    `infoOverName` is new beside it because the plate's right padding is
    the only thing now keeping the mark off the caption.
  - **The stroke was the thing that did not survive the scale.**
    `ToolIcon` draws at `stroke-width: 1.3` in an 18-unit viewBox, which
    is 1.3/18 of whatever the box is: a hairline in the 16 px inline icon
    and a 7.9 px slab at 120 px. Held at 2.4 CSS px with
    `non-scaling-stroke` it reads as the same hand at every size the grid
    produces. Nothing about the paths changed.
  - **`pH` and `Bq` stay letters** — GUI-109's ruling stands, neither has
    a silhouette — but they had to grow with everything else or two tiles
    in thirty-four would read as a mistake. Sized in `cqw` off the
    compartment rather than in `rem` off the root, because the compartment
    is 117 px on the desktop grid and 144 px on a mid-size phone. They
    come out at 40% of its height, and `letterShare` pins that.
  - **The check found something before the layout shipped.** With the
    "after 2 missions" and "mission kit" tickets left in the flow AFTER
    the drawing, a locked tile's device sits higher than its neighbours'
    while the row stretches around it, and a shelf row's feet disagree by
    the height of a ticket. `unlevelRows` fails on that. The tickets moved
    above the device, between the plate and the compartment, so the
    drawing stays the last item with `margin-top: auto` and its foot is
    the tile's bottom edge whatever else the tile carries. With a locked
    and a loaned tile in every row the share settles at 52.5–54.3% rather
    than 58.1–60.5%, because those rows are taller.
  - **The phone needed no rule of its own.** The 5.8rem column override
    GUI-110 added is gone: the tile's own 7.6rem minimum yields exactly
    two columns at 320 px, and the device is BIGGER there (125 px) than on
    the desktop grid (123 px), which is the right way round for the
    surface with the least room for prose.

  Measured at four widths, by the harness's own audit code run against a
  standalone page serving this component's real CSS, the real catalogue
  and the real `ToolIcon` paths — it reproduces the deployed cupboard's
  118 x 84 / 34 x 34 / 11.7% exactly, which is what makes it worth
  trusting, and the assertion itself runs in CI against the built payload:

  | viewport | tile | drawing | share | was |
  | --- | --- | --- | --- | --- |
  | 1440 x 900 | 136 x 185 | 123 x 123 | **60.1%** | 11.7% |
  | 896 x 920 | 130 x 179 | 117 x 117 | **58.8%** | 12.2% |
  | 390 x 844 | 173 x 206 | 144 x 144 | **58.1%** | 12.0% |
  | 320 x 640 | 138 x 187 | 125 x 125 | **60.5%** | 9.8% |

  `tools/test-ux-quality.mjs` holds it, one named check per precondition:
  the share is at least 45%; the drawing fills at least 96% of its
  compartment; no device is under 72 px on a side; the letters take at
  least a quarter of their compartment's height; and every device on a
  shelf row has its feet on the same line, however many lines its name ran
  to. Run against the old cupboard those report 11.7%, 32 drawings loose
  in their box and 34 under 72 px, so the check fails on the defect it was
  written for rather than only passing on the fix.

  **The price, stated rather than hidden: scroll.** The cupboard went from
  1.2 screens to 2.6 at 1440, and from 4.3 to 8.7 at 320 x 640. Thirty-four
  devices drawn at 120 px cannot also fit on one screen, and the item asked
  for the devices. The filter box and the five shelves are the navigation
  that makes that survivable, and both were already there.

  Left standing, and it is the honest answer to "does anything still read
  as a list": the three rows of chrome ABOVE the first shelf. The target
  card, the filter and the progress line are a form, and on a 320 px phone
  they take a third of the height before a single device is visible. The
  shelves below them are now a cupboard; the top of the dialog is still a
  dialog.

- [x] **GUI-116 — The showpiece reactions, rendered, and scaled by the
  numbers.** *All three candidate causes named below were measured and
  closed: amplitude (#688), size (#689) and lifetime.*

  A soda volcano should look like one: foam that climbs and
  spills, steam that rises and thins, a flame that flares. The owner named
  foam, steam and explosions. Each needs a quantity to ride on, and the
  engine already computes them — `GasEvolved` carries moles and a rate,
  `peroxide-decomposition` reports joules released, boiling reports the
  water that left the vessel, `Sedimentation` reports a settled fraction.

  **The contract, and it is not negotiable:** nothing may be drawn that is
  not read off a computed quantity, and the mapping from quantity to
  appearance must be visible in the code and testable. A learner who doses
  twice the vinegar must see a visibly bigger eruption, and a test must
  fail if they do not. `BenchEffect.svelte`, `FluidOverlay.svelte`,
  `ParticleView.svelte` and `IgnitionFlameCanvas.svelte` are the existing
  surfaces; `fluidScene` is the deterministic scene the engine already
  ships, and GUI-098's WebGPU tier is a *presentation* option on top of it,
  never a second source of truth.

  **Most of the contract above already exists, and the next person to work
  on this should build on it rather than start over.** `magnitudes.ts` is
  GUI-059: 2600 lines that map engine event amounts onto visual scale
  factors in [0, 1], under its own stated rule — *"Every factor names its
  source event field so the link is auditable."* `BenchEffect.svelte`
  already drives `--condense-rate` and `--drain-rate` from
  `benchEffect.magnitude`, and already prints the quantity beside the
  drawing (`… kJ`, `… mmol`). So the mapping layer is built and audited;
  what is thin is the DRAWING on the far end of it. A soda volcano needs a
  foam effect that reads `gasMag` the way the condenser reads its
  magnitude — not a new parameter system.

  **And the drawings are thinner than "missing" too — read this before
  building anything.** Forty-odd effect kinds are already drawn, `foam`
  among them: `Vessel.svelte` renders a foam block whose height comes from
  `vessel.foam.height_cm`, whose bubble count scales with
  `vessel.foam.volume_liters`, and whose collapse runs on the model's own
  `halfLifeSeconds`. The engine models the foam and the app draws it to
  scale, today.

  **Nor is it confinement — I claimed that next and it is also wrong.**
  `Vessel.svelte` already draws `foam-overflow`: an ellipse at the rim and
  two paths running down the outside of the glass, scaled by
  `overflow_liters / FULL_AT_L` off the engine's own `foam.overflow_liters`.
  Foam already climbs, spills, and is measured while it does.

  **So this entry has been wrong twice, and the remaining question is
  empirical rather than architectural.** Everything the item asked for is
  built: the quantities, the mapping, the drawing, the spill. The owner
  used the app and it did not read that way. Before anyone writes code
  here, someone has to *look at the running bench* and say which of these
  it is:

  - **Size.** The vessel SVG is `width: clamp(64px, 14vw, …)` in a 100×140
    viewBox. A spill that runs to y≈68 of 140 is a third of a small
    picture. This is GUI-094 — *the vessel deserves the room* — wearing a
    different hat, and if it is the cause then GUI-116 is largely blocked
    on it rather than on any new drawing.
  - **Duration.** `latestEffect("foam", 3000)` — three seconds. An
    eruption a learner looks away from is an eruption that did not happen.
  - **Amplitude.** `spillScale` clamps at 1 at one vessel-full of
    overflow, so a tenfold dose past that looks identical to the first.
    That one is a real defect against the rule at the head of this
    section, and it is findable without looking at anything.

  The third was fixed on sight (#688: a log window in `magnitudes.ts`, so
  ten times the overflow moves the drawing by a third of its range instead
  of not at all). **The first was then measured on the deployed bench, and
  it is the answer:**

      vessel SVG     150 x 210
      bench pane    1024 x 779
      vessel share   3.9% of the pane's area

  One beaker, alone on the bench, drawn across under four per cent of the
  room it has. A spill scaled perfectly inside a 150 x 210 picture cannot
  look like an eruption, because the picture is not the size of an
  eruption. **So GUI-116 is mostly GUI-094 — *the vessel deserves the
  room* — and should be sequenced after it rather than beside it.** That
  item already says the same thing in words: "the wide empty expanse
  around a small beaker is the strongest signal we send that nothing much
  is happening."

  What was left of GUI-116 once the vessel was large: **duration.** That
  half is done, and it corrects this entry for the fourth time, because
  `latestEffect("foam", 3000)` is not what governs a foam.

  **`withinMs` was never the window.** `effectAlive` is
  `at - effect.at < (effect.durationMs ?? fallbackMs)`, so every constant
  in `Vessel.svelte` is a fallback that an engine-derived duration
  overrides. `foam_changed` has set `durationMs = half_life_seconds *
  1000` since it was mapped, with nothing on either end — and a foam
  event only fires when a stabiliser is present, so that product was
  never zero and never small. The seven stabiliser half-lives the
  registry ships are 90, 100, 120, 180, 300, 1800 and 3600 s. The foam
  drawing, and its `rising` class, therefore stayed live for between a
  minute and a half and **a full hour** after the foam was gone, and its
  collapse animation ran at `--foam-half-life: 3600s`, which is motion no
  one can see. Three seconds was the diagnosis; an hour was the defect.
  The 3000 was unreachable code.

  Inventory first: of the 54 `latestEffect(kind, ms)` windows,
  **eight kinds had a model quantity to ride on** — foam
  (`half_life_seconds`), `gas_produced` (`moles / rate_moles_per_second`,
  which is how long the gas takes to come off), `reacted` (`seconds`,
  a field whose own comment already said a mole in a second and a mole in
  an hour are different observations), plus settling, stirring,
  electrolysis, emulsion and fermentation, which already rode theirs
  through four *different* hand-written `Math.min(a, Math.max(b, …))`
  clamps. **The other forty-odd have nothing**, and that is the honest
  answer for them: an `osmosis` or a `gas_test` event carries no duration
  and no rate, so it keeps its constant.

  The decision now lives in `magnitudes.ts` as `modelledLifetimeMs`,
  beside the factors, under the same auditable rule — and it is bounded,
  by measurement rather than by taste. **What the model produces:**
  `apparatus.ts` accepts `seconds` in 1 … 3600 for stir, heat, cool and
  centrifuge (1 … 300 for a burner exposure), and the shipped foam
  half-lives top out at 3600 s — three and a half decades. **What the
  screen can spend:** the effect clock ticks at 100 ms and the vessel's
  looping effects run cycles of 0.8 s (`drip-fall`) to 4 s
  (`bubble-ride`), so below about a second a learner sees a fragment of
  one loop; the shortest window already in the file is 1200 ms, and that
  is the floor. **Where real time stops:** 12 000 ms, the longest window
  the app already used (`ferment`, "hours of bench time compressed to a
  watchable window").

  So: real time while real time is watchable — a four-second settling
  takes four seconds and the drawing is not lying about a duration it
  could have honoured — then a log tail to 18 000 ms at one hour, because
  past the ceiling the job of the number is *order*, not duration. An
  hour outlasts a minute by 1.5x instead of by 60x, and the old clamps'
  worst property goes with it: `min(8000, …)` gave a two-minute
  electrolysis and a half-hour one the same 8000 ms, which is #688's
  saturation defect in the time axis.

  The test is the claim, not the refactor: two phenomena whose modelled
  durations differ produce different visible lifetimes with the ratio in
  the right direction — inside the band, where the ratio is exactly the
  model's, and above it, where the old clamps made every long run
  identical.

  **And the larger vessel needed nothing retuned.** Checked rather than
  assumed: every stroke width, radius, particle count and font-size in
  the vessel drawing is in the units of a fixed `0 0 100 140` viewBox, so
  #689's 2.6x scales all of them together and the composition inside the
  picture is by definition unchanged — there is nothing expressed in
  device pixels to tune. The one thing #689 did change is a *premise*:
  the instrument readout's comment justified a duplicate HTML badge by
  "4.5-unit screen type lands at under 3 real pixels", and at 257 px that
  type is 11.6 px. The badge stays — the moment a second vessel arrives
  the clamp drops back to 150 px and the glyph is 6.8 px again, and on a
  phone it is the original 2.9 — but the comment now records both
  numbers instead of the one that stopped being true.

  Scope note: "explosions" in a school-chemistry bench means a flare, a
  bang, a lid lifting, a flask venting — the engine models energy release
  and gas production, and the drawing must not promise more than the model
  computed. See also the standing rule that this laboratory shows what a
  bench would really do, including refusing to show what it has not
  modelled.

## Glass and liquid at the new size (GUI-118)

- [x] **GUI-118 — The optics of a vessel three times the area.** #689
  (GUI-094) took a lone vessel from 150x210 to 257x360, from 3.9% of the
  bench pane to 11.6%. Everything inside that picture had been tuned for
  the small one. This item is the paint, not the layout.

  **What the work found, before changing anything.** An audit of what the
  engine *computes* against what the app *draws*:

  | Engine value | Where it is computed | Drawn? |
  | --- | --- | --- |
  | `SceneLiquid.srgb` (Beer–Lambert over the CIE 1931 observer) | `appearance.rs` → `scene.rs` | yes, per layer, with `liquidOpacity` mapping tint to alpha |
  | `SceneLayer.srgb` per phase (LLE split) | `scene.rs` | yes, bottom-up |
  | `SceneLiquid.cloudiness` | `appearance.rs:267` — max of particle, emulsion, colloid and protein turbidity | yes, but only over the liquid — see below |
  | `SceneSolid.srgb` + `metallic` | `scene.rs` | yes, metals get their own stroke texture |
  | `SceneSolid.settled_fraction`, `volume_l` | `scene.rs` | yes, via `depositDisplayHeight` |
  | `SceneFoam.srgb` | `scene.rs` | yes, through `--foam-colour` |
  | `bubbling` | `appearance.rs` | yes |
  | `SceneChemiluminescence.relative_intensity` | `scene.rs:455` | intensity yes; the **hue is invented** — see the note |
  | `SceneLiquid.path_length_cm` | `scene.rs` | **no consumer in the app** |
  | `Appearance.spectral_gaps` | `appearance.rs:35` | **never reaches the scene at all** |

  Everything else in the vessel's glass — the wall tint, the vertical
  depth shading, the meniscus — is *drawn*, computed by nobody, and that
  is the correct side of the line: it is presentation of a shape the scene
  already fixes, the same way a drawn beaker is not a claim about
  chemistry.

  **The one computed thing that was not reaching the picture.** The
  suspension wash is painted inside the liquid block, and the settled
  deposit is painted after it. So a milky vessel showed its deposit in
  full, unveiled colour *through* cloudy liquid. Light leaving that
  deposit has crossed the same suspension the engine already computed, so
  the same `cloudiness` is now applied over the deposit band as well —
  once, not twice: the wash below it is covered by the deposit itself.

  **Made theme-aware.** The glass gradients carried `#bcd6e4`, `#eaf5fb`,
  `#ffffff` and `#000000` fixed in the markup. They are now the tokens
  `--glass-wall`, `--glass-core`, `--glass-specular` and `--glass-depth`,
  defined for the light, dark and high-contrast benches. SVG presentation
  attributes cannot hold `var()`, so the stops carry classes and the
  component's stylesheet resolves them.

  **Retuned for the size.** Five stops across the wall falloff read as
  flat bands once the vessel is three times the area, so the falloff is
  now sampled closely enough to stay a curve, with a direction to the
  light: a specular streak at the left quarter, a turned-away wall on the
  right. The meniscus gained a shaded underside; a single 1-unit stroke
  was a scratch at 150 px and a longer scratch at 257 px. Nothing
  geometric changed — the drawing is one fixed `viewBox` scaled by CSS, so
  every stop and stroke is in the same user units at 64 px as at 257 px.

  **Deliberately not done.** `path_length_cm` is shipped with a doc
  comment inviting a renderer to rescale absorbance against it. The drawn
  vessel width is not a physical path length, so rescaling the colour by
  it would put a fabricated optical claim on screen; it stays unused and
  is recorded here instead. `spectral_gaps` — the species whose optical
  contribution the model could not compute — is an honesty signal the
  engine produces and `SceneVessel` does not carry, so the app paints a
  colour without ever admitting the gap. Surfacing it needs an engine
  change and belongs to its own item. The chemiluminescence glow is a
  fixed cyan over a computed intensity: the hue is invented, the engine
  models no emitter, and it was left alone only because it is another
  item's live surface.

  **Guarded.** `VesselGlassOptics.test.ts` renders the component and
  asserts the computed `cloudiness` reaches the drawing, scales it, and
  veils the deposit — and that a clear liquid draws neither. Named
  preconditions in `tools/test-ux-quality.mjs` check in real Chrome that
  the glass gradient is actually painted, that every stop resolves to a
  colour (a missing token paints glass flat black), that each of the three
  themes defines its own glass, and that the phone vessel is still the
  same gradient-painted glass at the small size.

## The colour the model could not finish (GUI-119)

- [x] **GUI-119 — A vessel that admits its colour is incomplete.**
  #691 (GUI-118) ended with a list of computed values the picture was not
  showing. `Appearance.spectral_gaps` was the one that had *no path to the
  picture at all*: `SceneVessel` did not carry it, so the drawn beaker
  painted a confident colour over a gap the engine had already admitted in
  words. A learner who ran `look` and read the note learned the colour was
  partial; a learner who looked at the beaker did not. That is the wrong
  way round for a laboratory whose standing rule is that it refuses to
  show what it has not modelled.

  **What the work found.**

  - The *prose* half was already complete, and complete in German: the
    `look.spectral-gap` Phrase has a catalogue row, `SceneVessel.notes`
    carries it, and `scene::localize` recomposes it — so the vessel's SVG
    `<title>` and the live observation line under the drawing have been
    saying "Die Farbe ist unvollständig …" for as long as the note has
    existed. Nothing needed inventing; the sentence needed a *picture* to
    stand next to.
  - The gap list is produced by `solution_optics::spectral_gaps`, which
    walks `vessel.solution.species` — the native speciation. **In the
    browser, `kerotakis-phreeqc` is built cache-only** (a browser cannot
    link the C++ engine), so an aqueous answer is a shipped pre-warmed
    result or a stated cache miss. The gap therefore appears in the app
    only where the shipped cache carries a solution whose native complexes
    have no registered spectrum. Whether any *lesson* currently reaches
    that state was not measured here and is worth its own item — the
    signal is now wired end to end either way, and the native bench and
    the conformance corpus reach it directly.
  - `bench.rs` and `instrument.rs` both consult the same list for the
    spectrophotometer. Three consumers, one source; the scene is the
    fourth and was the only one missing.

  **What is on the scene.** `SceneLiquid.spectral_gaps: Vec<String>` —
  the *names*, not a flag. Names cost a few short strings per frame and
  buy the only question the mark provokes ("what is missing?"); a bare
  boolean would also have put the picture and the prose beside it in
  disagreement about how much the model is willing to say. Empty lists
  are omitted from the wire, so a host written before the field sees
  byte-for-byte what it saw before. Recorded in `PROTOCOL.md`.

  **What is drawn.** A diagonal hatch over the liquid column, and nothing
  else in the vessel — the glass, the foam and the deposit are not what is
  in doubt. A hatch because hatching is what a chart draws over a region
  it has no data for: it says *this is not fully known* without adding one
  fact about the liquid. It carries **no hue at all**, which is the whole
  point — the content of this signal is that no colour could be computed,
  and tinting the gap would invert it. It is not a warning either: no
  red, no icon, no animation.

  Two strokes per tile, one `--glass-depth` and one `--glass-specular`,
  half a pitch apart. The liquid underneath is an arbitrary engine colour
  on an arbitrary bench, and a single-tone hatch vanishes against half of
  them; a light/dark pair cannot. Rendered at five liquid colours from
  near-white to near-black, at 64 px and 257 px, on all three benches: the
  texture reads in every one of the thirty.

  `patternUnits="userSpaceOnUse"` inside the same fixed `viewBox` as
  everything else, so an 8-unit tile is 5 px at the 64 px vessel and 20 px
  at the 257 px one — one drawing scaled, never a re-tiling that turns to
  noise at one end.

  **What is said.** A `<title>` on the hatch, so pointing at the texture
  answers what it is; and a standing readout in the caption row the reader
  already reads the vessel's numbers from — `Farbe unvollständig` with the
  species named — so the admission is visible text and not only a hover.
  Three new rows in `de.json` and three empty ones in `_template.json`;
  adding a language stays one core toml and one web json, with no code.

  **Deliberately not done.** No tint was invented for the missing species,
  for the same reason `path_length_cm` stayed unwired in #691: the model
  could not compute it, and a renderer that guesses has made the claim the
  engine refused to. The hatch does not extend over the deposit or the
  foam, which the gap says nothing about.

  **Guarded.** `VesselSpectralGap.test.ts` runs it in both directions — a
  whole colour draws no mark, a gapped one draws the hatch and names the
  species; a scene from an older engine reads as "nothing known to be
  missing"; the liquid's own fill is byte-identical with the mark and
  without it. Named preconditions in `tools/test-ux-quality.mjs` measure
  the real thing in real Chrome at the real drawn scale: both strokes
  resolve, they differ from each other, **neither carries a hue**, they
  are faint enough to leave the computed colour legible, and the tile is
  still a hatch rather than a flat wash at the smallest size the bench
  ever draws.

## The result card's header was the journal pane (GUI-120)

- [x] **GUI-120 — The sequel GUI-108 named at `.result-card` and did not
  write.** GUI-108 capped the latest-result card so it could no longer
  squeeze the log to nothing, and left the arithmetic of the squeeze in a
  comment: at 371 px of journal pane, 88 px of journal chrome, **104 px of
  card summary**, 44 px of card body and 135 px of log. Its own verdict was
  that *the room is not really the body's to give* — the header is where
  the pane went. This item is the header.

  **What the work found, before changing anything.** Three things, and the
  first invalidates the premise of the brief.

  1. **The 104 px is not a wrapped header. It is `min-height: 3.25rem` read
     at a 32 px root.** The check that reports the decomposition lives
     between `#ux-text-zoom`'s injection and its removal in
     `tools/test-ux-quality.mjs` — over 300 lines apart — so it runs under
     **200% text zoom** and every number in GUI-108's table is a zoomed
     number. 3.25rem × 32 px = 104 px exactly. Measuring a replica of the
     journal pane with the real component and the real stylesheet, the
     summary is one row at every width from 160 px to 400 px: four explicit
     grid tracks cannot wrap.

  2. **What the four tracks did instead is worse than wrapping.** The
     journal is 288 px wide — `min(18rem, 23vw)` from the *later*,
     unconditional `aside` rule in `App.svelte`, not the `min(24rem, 34vw)`
     earlier in the same file — which leaves the summary 263 px. A 32 px
     tick, an 84 px ΔT badge and 90 px of icons, plus 26 px of rem-sized
     gaps, take 232 of them, and `minmax(0, 1fr)` gave the operation name
     the remaining **10 px**. Under the audit's own text zoom the ΔT badge
     alone is 165 px and the name resolves to **0 px**: the operation name
     and the reaction class — the card's entire answer to *what just
     happened* — were not painted at all, at the accessibility setting the
     suite exists to test. No assertion could see it, because a card with
     no legible content is exactly as tall as a card with some.

  3. **The cap was clipping the body, not sizing it.** Chrome wraps
     everything after a `<summary>` in `::details-content`, so
     `.result-body` is a grandchild of the card and its `flex: 1 1 auto;
     min-height: 0` was being read by a block box outside the card's flex
     line. At GUI-108's cap the card was a 146 px content box holding
     180 px: the bottom 34 px of the scroll region lay outside the clipped
     card where no scrollbar reached it.

  **What changed.** The header's furniture is px, not rem — `min-height`
  44 px (the audited touch floor; the summary is a real press target),
  6 px gaps, 6/8 px padding, a 20 px tick instead of 30 px — so it stops
  doubling with the type. The visible eyebrow *"Neuestes berechnetes
  Ergebnis"*, which was the second line of the name column and said what
  the card's own green frame already says, becomes the disclosure's
  accessible name; the vessel stays visible as a `v1` chip, because which
  vessel is the one fact in that line not deducible from the rest of the
  card. The ΔT badge moves to a second grid row and is hidden while the
  card is **open**, where the body's thermal row states it exactly with the
  before and after temperatures the header cannot fit; a closed card has no
  body, and is only its header, so there it shows and the second row costs
  the log nothing. `::details-content` is given the flex rules the body was
  written for. The body's left indent is 34 px rather than 2.85rem, which
  at 200% zoom had been spending a third of the body's width on an indent.

  **The same four numbers, at the same 371 px of pane:**

  | | before | after |
  | --- | --- | --- |
  | journal chrome | 88 px | 88 px |
  | card summary | 104 px | **44 px** |
  | card body | 44 px | **88 px** |
  | log | 135 px | **149 px** |

  The cap comes down from 40% to 36% *because* the header gave 60 px back:
  the log and the body both gain, which is what GUI-108 said a compact
  summary would buy. The body is four or five lines rather than two, and
  nothing of it is clipped. The name column is 108 px at 100% zoom and
  114 px at 200% — ellipsised for a long German term such as
  *Löslichkeitsgleichgewicht*, but painted, where it was 10 px and 0 px.
  At 320 px the journal is the whole screen and the name gets 124 px.

  **Where it is asserted.** The GUI-108 block in `tools/test-ux-quality.mjs`
  is extended rather than duplicated, and each precondition is its own
  named check: the header stays chrome (≤ 64 px — the defect reads 104),
  the header still paints the name (≥ 60 px — the defect reads 0), the cap
  sizes the body rather than slicing it (0 px clipped — the defect reads
  34), and the log keeps more than a third. The component test can only say
  what is in the markup, so it says that dropping the eyebrow did not drop
  the card's accessible name, its vessel, or the operation.


## Text painted into a zero box (GUI-121)

- [x] **GUI-121 — The class GUI-120 turned out to be an instance of.**
  GUI-120 found that the latest-result card, at the 200% text zoom this
  suite injects, gave its name column **0 px**: a 32 px tick, a 165 px ΔT
  badge and 90 px of icons filled a 263 px grid row, `minmax(0, 1fr)`
  resolved the name to zero, and `.operation`'s own `max-width: 100%;
  overflow: hidden` clipped the operation name and the reaction class out
  of existence. The card's entire answer to *what just happened* was not
  painted, at the accessibility setting the suite exists to test — and
  **no assertion could see it**, because a card with no legible content is
  exactly as tall as a card with some. Every height, overlap and share
  check in `tools/test-ux-quality.mjs` passed on that card.

  That is a class rather than an instance: **content squeezed to nothing
  by rem-sized furniture beside it, in a container that is itself
  perfectly healthy**, invisible to every geometric assertion already
  written. This item is the sweep for it, the general assertion it
  produced, and the three live instances it found.

  ### The assertion

  > An element that carries text a reader is meant to read must have a box
  > that text can be painted in — at every regime this suite drives.

  *Meant to be read* is an element with non-whitespace text in its **own
  child text nodes**. Own text, not `textContent`: otherwise every
  ancestor up to `<body>` carries the same string, one defect is reported
  at thirty boxes, and only one of them is the box that failed.

  *The visible box* is the element's own rect, cut down by every ancestor
  that clips with `overflow: hidden` or `clip` **between it and the first
  ancestor that scrolls**. `auto` and `scroll` are not clips — what lies
  outside them is one gesture from the reader — and the per-axis stop at a
  scroller is the whole difference between a defect and a list. Without
  it, a bottle below the fold of the cabinet's own `.groups` scroller is
  cut to zero height by `.shelf-pane`'s `overflow: hidden` three levels
  up: the first run reported **1117 of the 1194 elements on one surface**
  that way, and every one of them was a row the reader had simply not
  scrolled to yet.

  The viewport is not a clip either: an element pushed off the page is
  what `bodyOverflow` and `viewportOverflow` already measure, and counting
  it here would report the `position: absolute; left: -9999px` idiom as a
  defect every time it appeared.

  Two tiers, because *zero* and *unreadable* are different failures.
  **blank** — the visible box is zero in an axis and nothing of the text
  reaches the screen; this is GUI-120's `.operation`. **squeezed** — the
  box survives but paints less than the text needs, measured as the
  *smaller* of one em and the element's own `scrollWidth`. That second
  clause matters: a 22 px box holding a two-letter element symbol at a
  23 px font paints all of it and is fine, while a 20 px box holding
  200 px of German compound noun paints not one character and is not. An
  absolute floor reported the periodic table's `Li` and `Ir` as defects;
  the content-relative one does not.

  ### What it deliberately excludes

  Each exclusion is counted and printed with every reading, so that a
  shrinking sample cannot quietly empty the check:

  | exclusion | why |
  | --- | --- |
  | `aria-hidden="true"`, on the element or any ancestor | the app has already said this text is not to be read: a decorative tick, a `·` separator, a duplicated glyph |
  | `.sr-only` / `.visually-hidden`, self or ancestor | being a 1 px box with `clip-path: inset(50%)` is the *point* of that class — it is what GUI-120 turned the card's eyebrow into |
  | the same idiom spelled without the class | a ≤ 1 px absolutely positioned box with `clip-path` or `overflow: hidden`. `Vessel.svelte`'s `.observation-status` is one: a `role="status"` live region, announced and never painted, exactly as designed |
  | no box at all (`getClientRects()` empty) | `display: none`, `[hidden]`, a **closed `<details>`**, an unopened `<dialog>`: not painted, deliberately. This is why the sweep *opens* each surface before measuring it rather than trawling the document once |
  | `checkVisibility()` says no | `visibility: hidden`, `opacity: 0`, `content-visibility: hidden` |
  | a descendant of something already reported | the outermost offender is the one that describes the defect |

  ### Where it measures, and in which regime

  Sixteen readings, each **labelled with its regime** rather than
  inheriting it silently. That is GUI-120's other finding made
  procedural: `#ux-text-zoom` is injected at one line of
  `tools/test-ux-quality.mjs` and removed **369 lines below it**, and
  GUI-108's entire table of measurements sat inside that bracket without
  saying so. For the record, in the file **as GUI-120 left it** the
  regimes were 1440 px (lines 808–1088, 1118–1401, 1423–1478, 1782–2019
  and from 2388), 390 px (1088–1118, 1412–1478, 1478–1579), 320 px
  (1579–1782) and **200% text zoom at 1440 px (2019–2388)** — the whole
  of the GUI-108 and GUI-120 blocks.

  The surfaces are the bench, the cabinet, the journal and its
  latest-result card, the equipment cupboard, the catalogue, the periodic
  table, the remove-vessel dialog, the utility drawer and an instrument
  panel — at 1440 px, at 320 px and at 200% text zoom. The cupboard, the
  catalogue and the table are dialogs, so the zoomed readings of them are
  *driven* open, and the open is a named check of its own; the
  catalogue's zoom bracket is five lines long, which is the point.

  Three preconditions are named checks: both regimes were visited, every
  surface offered text before it was measured, and the sweep saw the app
  rather than a fragment (~15 000 readable elements across the sixteen
  readings). An assertion that passes on an empty sample is the exact
  failure mode this one is written against.

  ### What the sweep found

  **The first instance is GUI-120's defect one component over.**
  At 200% text zoom the journal's own heading row gave its title nothing:
  `strong.pane-title` measured **0 × 33 px** with *"Laborbuch"* in it, and
  `.panel-collapse` was pushed outside the `aside` and clipped away by its
  `overflow: hidden`. The CSS comment above the rule had already predicted
  it — *"when the pane narrows it is the WORD that gives way"* — without
  noticing that text zoom narrows it without narrowing the pane. Every gap,
  pad and hit-area minimum in that row was in rem, so all of them doubled
  while the journal stayed at `min(18rem, 23vw)` = 331 px; the doubled
  furniture wanted 451 px of it. It reproduced in six of the sixteen
  readings — every zoomed one, because the journal is mounted behind every
  dialog — and in none of the 1440 px or 320 px readings.

  **The fix is GUI-120's rule: the chrome is px and the prose is rem.**
  `min-height` was already px; the gaps, the padding, the hit-area
  minimums and the *icon glyph* sizes join it, because an icon is
  furniture and does not grow with the reader's type. What stays in rem is
  everything with a word or a number in it — the title, and the digits in
  `.entry-count` and `.count`, which are information a reader reads. Each
  px value is its rem value at a 16 px root, rounded, so **nothing about
  the row changes at 100%**; at 200% the furniture comes to about 247 px
  of 331 and the title is painted again.

  **Two more instances, of a second shape of the same mechanism.** Once
  the scroller rule was in, the sweep read clean at 1440 px and at 320 px
  and still reported eight elements at zero in *every* 200% text-zoom
  reading. Neither is reachable by scrolling: both lie below the bottom
  of a pane whose overflow is hidden.

  - **The bench's control strip.** `.bench` had `min-height: 24rem`,
    which is **768 px** at a 32 px root. The bench pane has about 860, so
    roughly thirty were left for everything below the stage and the
    `VesselActionDock` was pushed clean out of `.bench-pane` and clipped
    away by its `overflow: hidden`. The selected vessel's name, its
    volume and temperature, its contents, and the *show all*,
    *measurement tools* and *equipment cabinet* buttons were not painted
    at all. That is a **layout** minimum — how much counter you need in
    order to stand glassware on it — not prose, so it is 384 px, which is
    what 24rem is at a 16 px root. Nothing changes at 100%.
  - **The cabinet's count.** `.tally` was a shrinkable flex item beside
    the group list, and at 200% zoom *"323 von 323 Stoffen"* was squeezed
    to **286 × 0** below the bottom of `.shelf-pane`. `flex: none`: the
    list beside it is the thing with a scroller and `min-height: 0`, so
    it is the one that should absorb the squeeze.

  Neither is fully closed, and the part that is left is **GUI-122** below
  rather than more of this item: the bench stage's minimum bought the
  dock about 45 px and recovered its first button, and `flex: none`
  stopped the tally being the item that gives way, but both panes are
  still taller than themselves at 200% zoom and the arithmetic that
  remains is a design decision rather than a rule.

  Nothing else was blank or squeezed. Every other candidate the first run
  produced was a scrolled-out row, the inline visually-hidden idiom, or a
  short string measured against an absolute floor.

  ### It fails on the defect, not merely passes on the fix

  Verified, rather than asserted, on a branch carrying this tree with
  `LatestResultCard.svelte` reverted to `aefc0502^` — the card exactly as
  GUI-120 found it. The sweep reports:

  ```
  strong.operation "Temperaturänderung" visible 0x41.8 of own 0x41.8 (em 28.8) in span. 0x71.9
  button.icon-provenance "⌖" visible 0x28 of own 28x28 clipped by details.result-card
  button.icon-export "⤓"    visible 0x28 of own 28x28 clipped by details.result-card
  button.icon-close "×"     visible 0x28 of own 28x28 clipped by details.result-card
  ```

  — the operation name at zero width inside a `minmax(0, 1fr)` track that
  is itself at zero, in a summary that is the right size, on a card that
  is the right size, in a pane that is the right size. GUI-120's own four
  checks fail beside it with the numbers they were written for (*summary
  104px of 64px*, *0px painting "Temperaturänderung"*, *214px of the card
  clipped*). The sweep also finds three things GUI-120's checks did not:
  the card's own export, provenance and close icons, pushed outside the
  card and clipped away.

  ### Known limit, stated rather than discovered later

  The sweep reads text *nodes*. The value and the placeholder of an
  `<input>` are neither, so a form field squeezed to nothing is not in
  this net — `Shelf.svelte`'s `.stepper` carries the standing comment
  about a number field once measured at 33 px, and that is the shape of
  thing this assertion does not yet see. Nor does it model `clip-path`,
  transforms or a parent's `text-overflow` beyond the box arithmetic
  above.

## Two more rulings, 2026-09-21

- [ ] **GUI-123 — the legibility guard reaches form controls.** GUI-121's
      assertion reads text *nodes*, so an `<input>`'s value and its
      placeholder are outside its net — a limit it recorded at the check
      rather than leaving to be discovered. `Shelf.svelte`'s `.stepper`
      already carries a standing comment about a number field measured at
      33 px, which is the defect in the wild.

      **RULED: extend it.** A value a reader typed, painted too small to
      read, is the same defect as a heading painted at zero. The ruling
      carries its own warning: **placeholders legitimately truncate**, so a
      threshold copied from the text-node rule will cry wolf. If the two
      cannot share a rule, give the value a strict one and the placeholder
      a loose one and say so at the check — the distinction between "the
      answer" and "a hint" is one a reader would agree with.

- [x] **GUI-092 — the ionic equation, derived: solve the spectator
      coefficients. DONE 2026-09-21.** The first slice shipped in August: the net ionic
      line, with spectators named at lv3. The complete equation with the
      spectators struck through was offered to the owner and could not
      honestly be drawn, and the reason is worth stating where the next
      person will look — **`ionic.rs::spectators` selects by abundance and
      constructs every term with `coefficient: 1`**. They are never
      stoichiometrically solved, so placing them on both sides of a
      complete equation would assert a balance nothing computed.

      **RULED: solve them.** Balance the spectators against the molecular
      equation the same way the net participants already are — `ionic.rs`
      has a small linear solve (`balance_against`) that pins free
      variables to zero and then *verifies every row*, returning `None`
      rather than a least-squares fiction. That refusal is the model: a
      complete equation that cannot be balanced must not be drawn at all,
      and the net line that ships today is the honest fallback.

      *What the work found, 2026-09-21.*

      **There is no molecular equation to balance against, and that turned
      out to be the useful discovery.** The ruling says "balance the
      spectators against the molecular equation", and the first hour went
      looking for it: `render.rs` composes equation LINES, but for a
      precipitation the string is never built — `Event::Precipitated`
      carries a solid and a quantity, and the bench has never believed in
      `AgNO3 + NaCl → AgCl + NaNO3` in any form a coefficient could be read
      off. The equation a textbook would print does not exist in this
      engine, by design.

      What does exist is the same stoichiometry in the only form a bench
      can hold it without remembering a reaction: **the reagents came out
      of bottles, and a bottle is electrically neutral.** Every ion the net
      equation consumes arrived beside a counter-ion, in the number that
      made its salt neutral. That is the molecular stoichiometry, derived
      rather than looked up, and it gives the same answer a molecular
      equation would: one chloride beside one silver, *two* beside one
      barium. It is one linear row per sign — the spectator cations account
      for the charge the anionic net reactants brought in and vice versa —
      solved with the module's own `gauss_jordan` and verified against
      every row, which is `balance_against`'s model precisely.

      `BaCl₂ + Na₂SO₄` is the worked case and the one a suite of 1:1 salts
      would have passed on: `Ba²⁺(aq) + SO₄²⁻(aq) + 2 Na⁺(aq) + 2 Cl⁻(aq) →
      BaSO₄(s) + 2 Na⁺(aq) + 2 Cl⁻(aq)`. Nothing in the beaker says "two";
      it falls out of barium carrying twice the charge sodium does.

      **The refusal got one guard the ruling did not anticipate, and it is
      the one that fires most.** A linear solve that pins free variables to
      zero is right for the net participants, where the free variables are
      candidate balancing partners and zero means "not needed". It is
      *wrong* for spectators, because there the free variables are ions
      that are physically in the beaker and the zero would be an assertion
      about which bottle was opened. With Na⁺ and K⁺ both in solution and
      cation counter-charge to account for, the system does not determine
      the answer — and it would verify anyway. So the ambiguity guard sits
      BEFORE the solve, and a beaker with two cations gets no complete
      equation. `an_ambiguous_counter_ion_draws_no_complete_equation`
      exercises it; `nothing_spectating_means_nothing_to_write_out`
      exercises the other refusal, a demand no spectator of that sign can
      meet. Both assert the net line still ships, because that is the
      point: falling back is a correct outcome.

      The verification is stronger than element-and-charge, and had to be:
      the spectators stand on both sides, so they cancel out of the element
      rows and out of total charge, which makes those rows nearly free. The
      row with teeth is that **each side is electrically neutral** — a side
      carrying net charge describes a beaker that would have electrocuted
      somebody. That is asserted in the engine tests directly and
      re-checked on the wire in `test-protocol-conformance.mjs`.

      On the wire the flag is `spectator: true` per term and no markup: the
      engine emits structure, the shell draws the line through it. Omitted
      when false, so every term outside `complete` is byte-identical to the
      August contract.

      **What stays open, and neither is a shortcut not taken:**

      - *The neutral complexes beside the free ions* — AgCl(aq) is present
        at fifty times the free silver and appears in neither line. The
        first slice deliberately ordered charge above abundance to keep
        `AgCl(aq) → AgCl(s)` out of the net equation; showing the complex
        *beside* the free ions is a different display question (a third
        row, not a fourth term) and nothing here decided it.
      - *Bases beyond precipitation and neutralisation.* Unchanged:
        `IonicBasis` still has two variants because those are the two
        engine results that carry their own participants. Redox and organic
        steps carry no participant list and are still not guessed at.
      - *Multi-salt beakers.* The ambiguity guard means a solution holding
        two cations or two anions above the naming threshold gets the net
        line only. That is honest and it is also a real limit: a learner
        who adds a third salt loses the complete equation. Narrowing it
        needs a way to say which counter-ion arrived with which
        participant, which the speciation does not record.

## A pane taller than itself at 200% text zoom (GUI-122)

**RULED 2026-09-21: the pane scrolls.** Put a scroller on the bench pane,
as the journal and the cabinet already have. Nothing is lost and nothing
is redesigned; a zoomed reader scrolls the bench the way they scroll
everything else.

Two alternatives were offered and declined, and the reasons are worth
keeping because they constrain the implementation:

- *Let the stage give up its share* — shrink the vessel at high zoom so
  the furniture fits. Declined, and it would have undone work from the
  same week: GUI-094 has just taken a lone vessel from 3.9% to 11.6% of
  the pane **because it was too small**, and a reader at 200% zoom is
  often zoomed precisely because they need things larger. Making the
  picture smaller for the readers who most need it bigger is the wrong
  trade.
- *Turn the dock into a pull-up sheet below a height threshold.* Declined:
  a second interaction model for the same controls, and a sheet that can
  cover the very vessel you are pouring into.

**The cost this ruling accepts, stated plainly:** the bench pane gains a
scrollbar it does not have at 100%, and the stage stops being guaranteed
wholly on screen — which is the one thing that pane has always promised.
Whoever implements it should make the stage the *last* thing to scroll
out of view, not the first.


- [x] **GUI-122 — What falls out of the bottom of a pane is not below the
  fold; it is gone.** GUI-121's sweep left two instances standing, and
  they are one finding wearing two hats. It is a *different shape* from
  the rem-furniture defects GUI-121 closed: there, a fixed box beside the
  text took the room; here, a pane's whole column of children is taller
  than the pane at 200% text zoom, the pane clips with `overflow: hidden`,
  and the part that falls out sits under **no scroller at all**. Not one
  gesture away — unreachable.

  | what is not painted | under | selector |
  | --- | --- | --- |
  | the selected vessel's name, volume, temperature and contents | `div.bench-pane` | `span.selection-copy` |
  | *measurement tools*, *equipment cabinet* | `div.bench-pane` | `div.more-actions` |
  | *"323 von 323 Stoffen"* | `nav.shelf-pane` | `p.tally` |

  GUI-121 took the two rules that were plainly wrong: the bench stage's
  `min-height` came down from 24rem (**768 px** at a 32 px root) to
  384 px, which bought the dock about 45 px and recovered its first
  button, and `.tally` became `flex: none` so it is no longer the item
  that gives way. Roughly **120 px** is still missing from the bench
  pane's column, and it is spread across four components — the equation
  block, the stage, the `VesselActionDock` and the command bar — none of
  which is obviously the one that should yield.

  **That is the question this item exists to answer, and it is a design
  decision rather than a rule**: does the pane itself become a scroller,
  does the stage collapse below its minimum when the column is short,
  does the dock become a sheet that the bench can cover? Whichever, it
  changes how the bench reads at every size, which is why GUI-121 did not
  take it.

  **It is guarded in the meantime, from both directions.** `KNOWN_OPEN`
  in `tools/test-ux-quality.mjs` names these three by precise selector
  and regime — never by their text — so a *new* zero anywhere else still
  fails the sweep. And a second check asserts that every entry in the
  list **still reproduces**, so the day this item lands the suite fails
  until the list is deleted. An exception cannot outlive the defect it
  names.

  ### What the work found

  **`KNOWN_OPEN` is empty.** All three entries stopped reproducing and all
  three were deleted; the list and both of its checks stay, so an empty
  list is now the asserted claim that this sweep carries no standing
  exceptions at all.

  **A scroller is not a clip, and the sweep already knew it.** The one
  thing that made all three instances *defects* rather than *lists* is the
  rule `visibleBox` has carried since GUI-121: the walk up the ancestors
  stops cutting once an axis is reachable, which is what keeps the
  cabinet's 1117 unscrolled bottles from being reported invisible. So the
  fix and the assertion are the same change of state — `overflow: hidden`
  to `overflow: hidden auto` on `div.bench-pane` and on `nav.shelf-pane`
  — and nothing in the sweep had to be taught about it.

  **The cabinet's count was not the bench's problem wearing a hat; it was
  a sibling problem.** The ruling says the cabinet "already has a
  scroller", and it does — on the *list*. `p.tally` is the list's
  **sibling**, so no amount of scrolling the bottles could ever reach it,
  and GUI-121's `flex: none` only stopped it being the item that gave way.
  The pane itself had to become the fallback scroller. One consequence had
  to be answered with it: with the pane scrolling, an unfloored list
  shrinks to nothing and the cabinet becomes a column of rails over no
  bottles, so the list took a **120 px** floor — a layout minimum, in px
  like the stage's 384 px, so it does not double with the reader's type.

  **"The stage scrolls last" is three rules, not one.** The *order* — the
  two pieces of chrome at the two ends of the column with the stage
  between them, so a reader scrolling down loses the equation block and
  gains the dock and passes over the stage either way. `flex: none` on
  both ends — GUI-121's `.tally` lesson said again, that the ends of a
  column must not be the items that give way; what gives way is the
  stage, down to its own 384 px floor, and after that the pane scrolls.
  And a *cap*: neither end may fill the pane, `max-height: 66%` with its
  own scroller inside the cap, so at either end of the scroll the stage
  keeps at least a third of the pane. The cap is a percentage of the
  **pane** and not of the viewport, because the pane is what the reader is
  scrolling — an earlier draft of this used `45vh`, and the measurement
  below is why that was wrong: at 200% the pane is 371 px, so 45vh is
  *405 px*, a bound larger than the thing it was supposed to bound.

  **What a reader actually loses, measured in the app rather than
  reasoned about.** At 1440x900, 100%: the pane's column is 820 px in an
  820 px pane — **no overflow, no scrollbar, and the scroll cannot move**
  — with the stage at 691, the vessel at 424 and the dock at 113, all
  whole. At 320x700: 582 px in a 582 px pane, the same, with the whole
  vessel on screen before anything is scrolled. At 200% text zoom the
  pane is only **371 px** and its column is **623** — 252 px of overflow,
  a quarter more than the ~120 px GUI-121 estimated, because the dock
  stops being squeezed once it is `flex: none`. With the pane untouched
  the stage shows **372 of its 384** and the dock is cut to **0 of 207**;
  run to the bottom, the dock is whole at 207 and the stage still shows
  **133**. That is the ruling's sentence as numbers: the reader reaches
  the dock, and never loses the counter.

  **The vessel is reachable, and it is not wholly on screen — and that is
  GUI-114's geometry rather than this ruling's.** At 200% the vessel is
  **479 px** of glassware on a work surface inside a 384 px stage that
  has carried its own scroller since GUI-114, so it cannot be wholly on
  screen at any scroll position of any pane, and it could not before this
  change either. The first draft of the assertion asked for it anyway and
  failed in CI on exactly that — 479 px tall, 0 px of it in the pane —
  which is the useful half of the finding. What the ruling actually owes
  is that it is never clipped away with nothing to reach it by, so that
  is what is asserted, twice: the stage's own scroller brings **364 of its
  479 px** into the pane's view with the pane untouched, and the pane's
  new scroll brings **more** of it into view rather than less — 0 px at
  the top, 239 at the bottom.

  **The cabinet's numbers.** 172 px of overflow at 200%, the count whole
  at 64 px once the pane is scrolled to it, and 120 px of shelf still
  showing under the rails.

  **The sweep itself is clean.** 1194 readable elements on the bench,
  cabinet and journal at a 32 px root: **0 blank, 0 squeezed**. Same for
  the cupboard (1243), the periodic table (1435), the latest-result card
  (1216), the remove-vessel dialog (1231) and the utility drawer (1229).

  **Every reading says which regime it was taken in.** `benchScrollAudit`
  takes `regime` as an argument and returns the root font size with the
  measurement, because `#ux-text-zoom` is injected on one line of that
  file and removed hundreds of lines below it and #697's whole table sat
  inside the bracket without saying so. The three readings are labelled
  `1440 px`, `320 px` and `200% text zoom`, and each block leads with its
  own preconditions as named checks — a pane with no dock in it, a pane
  that does not actually overflow, or a pane whose scroll never moves all
  make every claim after them vacuous.

  **One thing is asserted on one axis only, and says so.** The vessel
  check measures *vertical* overlap with the pane. Sideways is the work
  surface's own scroller — a 42 rem minimum inside a narrower pane, and
  that has been true since GUI-114 — so what this ruling changed, and
  therefore all this asserts, is what the pane's new vertical scroll can
  take away.


## The verdict list was the last English thing on a German screen (GUI-124)

Reported 2026-09-22, in a paste of a German bench transcript: *"in the
experiment picker/runner, we see 'added' not 'hinzugefügt'."*

- [x] **GUI-124 — an expectation is a verb and a name, and each is
  translated for what it is.** The rows under a finished catalogue run
  are not authored strings. They are wire tokens out of `codex/*.toml` —
  `added:phenolphthalein`, `gas_evolved:CO2`, `not_yet_modelled` — and
  `Catalog.svelte` rendered each one as `t(want.replace(/_/g, " "))`.

  **Two independent reasons, and either alone was fatal.** The compound
  token went to the dictionary as ONE key, and no bundle has ever carried
  a key of that shape, so `t()` fell back to its key: the colon was not
  even spaced. And the dictionary held no entry for the verbs regardless
  — **26 of the 29** verbs the catalogue uses had no German at all. The
  three that resolved (`boiled`, `froze`, `measured`) did so by colliding
  with words translated for something else, which is worse than missing:
  it is a verdict row that looks translated.

  **The fix splits the token, because the two halves are different kinds
  of thing.** The verb is prose and gets a per-verb template with `{what}`
  in it, since word order is not shared between languages — *"phenolphthalein
  added"* is *"Phenolphthalein hinzugefügt"*, and the German reader meets
  the operand first. A verb beside a colon cannot express that, which is
  why these are templates. The operand is a NAME and goes through the
  same term table the rest of the bench uses, with underscores and hyphens
  normalised to spaces because the catalogue writes the same name three
  ways (`methyl_orange`, `peroxide-decomposition`, `thermoplastic sheet`).

  **A formula is excluded on purpose, not by omission.** `NaCl` and `CO2`
  have no dictionary entry and therefore render as themselves. Sending a
  formula to a translator is how a bundle acquires a German "NaCl" that is
  "NaCl" today and is something else after a well-meaning edit.

  **The gate reads the catalogue, not the tables.** `i18n.test.ts` walks
  `codex/*.toml`, collects every `events`/`forbidden` token, and fails on
  a verb neither table names, on a verb phrase with no German, and on a
  lowercase operand with no German. The tables are only right for as long
  as they cover the data, and the failure this replaces is a codex author
  adding one more verb that nobody translates. Proven non-vacuous by
  removing one German value: the gate names it.

  **What it cost:** `expectationLabel.ts` (two tables, 15 bare verbs and
  14 with an operand, a partition asserted in its own test), 32 new
  strings in `de.json` and `_template.json` — 29 verb phrases in
  `messages`, and `gypsum`, `peroxide decomposition`, `thiosulfate acid`
  in `terms`, the only three operands the catalogue names in words that
  German had not already been taught. No new collisions: the two the
  collision lint records are the two it recorded before.


## Forty-one arrows, one of them chemistry (GUI-125)

Found while measuring the runner for the 2026-09-22 report. The REAKTION
rail above the bench read, in a German session:

```
REAKTION   Route → Kerotakis analytic equilibrium evaluator · phreeqc
```

- [x] **GUI-125 — the equation comes off the event, never out of the
  prose.** `equationFromRenderedLine` took any rendered line carrying a
  `→`, cut it at the last colon before the arrow and the first full stop
  after it, and pinned the result as the reaction — onto the rail, into
  `benchEquations`, and into the **balancing drill's question pool**.

  **The arrow is not the reaction's private punctuation.** 41 of the
  engine's rendered lines carry one and exactly one of them is an
  equation. Two were caught live:

  - `v1: Route → Kerotakis analytic equilibrium evaluator · phreeqc.dat…`
    — the aqueous routing announcement, whose arrow separates a label from
    a solver name.
  - `v1: T 298,150 K → 299,356 K (ΔT = +1,206 K)` — a temperature change.
    In German there is not even a full stop to cut it at, because the
    decimal separator is a comma, so the **whole line** pinned.

  No heuristic separates those from chemistry, because they are the same
  shape: a thing, an arrow, another thing. So the scrape is gone.
  `equationsFromEvents` reads `Event::ReactionOccurred { vessel, equation }`
  — the event the prose was rendered FROM — which is the move GUI-092 made
  for the ionic form and for the same reason: the structured claim is the
  claim, and its prose is one rendering of it. It also drops the vessel
  prefix for free, since the event never carried one.

  **The register gate moved with it.** Reading the event means the
  equation is available at lv1 too, where the engine deliberately renders
  "the mixture changes — something new is forming!" and no equation. The
  rail already hid itself at lv1; the PIN now respects lv1 as well, which
  is the half that was keeping the drill's pool honest.

  **Verified in Chrome-for-Testing against the deployed engine payload**
  (`kero-5522ab0`) with the app built from this branch, German, lv3,
  `vinegar-and-baking-soda` run from the catalogue: the rail reads
  `HCO₃⁻ + CH₃COOH → CH₃COO⁻ + H₂O + CO₂↑`, and at the step where the
  routing announcement and two temperature changes land it reads nothing
  at all — where before the change it read the routing sentence.


## The caption that became a lid (GUI-126)

Owner, 2026-09-22: *"the 'next step' dialog is in the way, overlaying the
complete bench."*

- [x] **GUI-126 — a caption takes a third of the screen, and what gives
  way inside it is the account.** The design was already right in
  principle — `.scrim.running` goes transparent and drops pointer events,
  and the panel becomes a strip at the foot of the screen — and it was
  right at one size only. `.panel.running` carried `max-height: none`.

  **Measured in Chrome against the deployed engine payload
  (`kero-5522ab0`), German, lv3, step-by-step, before the change:**

  | regime | caption | of viewport | stage covered | vessel covered |
  |---|---:|---:|---:|---:|
  | 1440×900 | 250 px | 28% | 19% | 11% |
  | 390×844 | 390 px | 46% | 39% | 31% |
  | 1440×900 @ 200% text | **713 px** | **79%** | **100%** | 21% |

  The third row is the report, in numbers: at 200% text zoom there was no
  bench on the screen at all.

  **And after:**

  | regime | caption | of viewport | stage covered | vessel covered |
  |---|---:|---:|---:|---:|
  | 1440×900 | 270 px | 30% | 21% | 12% |
  | 390×844 | 253 px | 30% | 17% | **0%** |
  | 1440×900 @ 200% text | 288 px | 32% | 49% | 21% |

  The desktop caption is **20 px taller** than it was, and that is the
  cost of the column layout below; in exchange its account scrolls rather
  than deciding the height. Everywhere else the caption roughly halves.

  **Three rules, and the third is the one that is easy to lose.** A cap
  in `vh`, because the panel is a fixed overlay whose container IS the
  viewport — the mirror of GUI-122's `45vh` mistake, where the container
  was a pane and `vh` bounded the wrong box. **The account is what gives
  way**: `.dock-account` scrolls, the panel itself does not, and
  `.dock-controls` is `flex: none`, because a capped panel that scrolls
  its own "next step" button away is a run nobody can continue. That is
  GUI-121 and GUI-122's lesson a third time — what gives way is the
  middle, never the ends.

  **The cap carries a floor in `rem`, and the measurement is why.** At a
  flat `30vh` under 200% text zoom the controls took 190 of 270 px and
  the account was one clipped line. The floor is 9 rem — nine lines of
  the reader's own type — which is 144 px at rest (under the cap, so the
  ordinary case is untouched) and 288 px zoomed. An 11 rem floor was
  tried and rejected on its number: a third line of account cost **29
  more points of covered stage**, 66% against 37%. The bench is the
  experiment, so the bench won.

  **What actually fixes the 200% case is not geometry.** Of that step's
  550 characters of account, **454 are one lv3 routing paragraph**. The
  cap is a bound on the damage; the verbosity switch is the fix.

  **`.dock-produced` lost its own `max-height: 5.5rem`** — 88 px at rest
  and 176 px at 200%, a nested scroller that grew in the one regime where
  the box containing it needed to shrink.

  **Guarded twice.** `tools/test-ux-quality.mjs` walks the catalogue,
  starts a step-by-step run and measures the caption in both regimes —
  the caption at most 40% of the screen, the stage never wholly behind
  it, the glass keeping the half its chemistry is drawn in, the controls
  on screen and inside the caption, and the account able to scroll.
  `runningCaption.test.ts` reads the rule out of the component, because a
  `max-height` deleted in a refactor reads as tidying up and no
  behavioural test can see a stylesheet.


## Routing is its own switch, not lv3's tax (GUI-127)

Owner, 2026-09-22: *"we need a toggle for verbosity: display 'Route ->'
info or not."*

- [x] **GUI-127 — the register is how much chemistry; this is which kinds
  of line at all.** The aqueous routing announcement is a paragraph at
  lv3 — engine · dataset · activity model · the clause explaining why
  that dataset was chosen. On the German bench measured for GUI-126 it
  was **454 of one step's 550 rendered characters**, and there was
  exactly one way to be rid of it: leave lv3, and give up every number
  lv3 had been turned on for. Two different questions had one control.

  **The engine decides, not the shell.** `Narration { routing }` in
  `kerotakis-core::render` filters the EVENT — `Event::SolutionRouted` —
  and `render_events_narrated` is what both hosts now call.
  `render_events_in` stays, delegating with `Narration::FULL`, so every
  other caller renders exactly what it rendered before. A shell-side
  filter was considered and rejected on this codebase's own scars: the
  only handle a shell has is the WORDS, and "the line that starts with
  Route" is a fact about one language's rendering at one register, not
  about the step. `rendered` is not positionally aligned with `events`
  either — it is filtered by `is_observable()` and deduped at lv1 — so
  a client cannot even find the line reliably without reimplementing the
  engine's own rules in TypeScript — one value derived in two places, which
  is the split the native/wasm divergence has already cost this project
  twice.

  **Only the prose is suppressed.** The event still travels in `events`
  and its provenance still reaches `routes`, so the provenance drawer —
  the surface that exists for exactly this — answers the same either way.
  A reader who turns the announcement off has said *not in the log*, not
  *do not tell me*. The switch's two titles say so in one sentence each,
  and the "off" one names where the fact went.

  **Both bindings, because the native one is the one that gets
  forgotten.** `set_announce_routing` is answered by the wasm host and by
  `NativeLab`, and `every_command_the_shell_sends_is_answered` scrapes
  `TauriHost.ts`, so it cannot be added to the browser alone — which is
  precisely what happened to `set_locale` for as long as the engine had a
  German catalogue.

  **An older engine keeps the switch where it is.** The command is
  refused by name, the session catches it, and the control does NOT move:
  a switch reading "off" over a log that still announces is worse than a
  switch that did not move. `hello.narration` (`["routing"]`) is how a
  shell can know before it offers the control. A save written before this
  carries no answer, and that absence IS the default.

  **Verified in Chrome** against the deployed payload: the control
  renders beside the dial as a 40x40 target, inside the viewport, with
  its German sentence as the accessible name, and clicking it against an
  engine that does not answer the command leaves it pressed — the honest
  degradation, exercised for real rather than reasoned about. **What is
  NOT verified in a browser here is the suppression itself**: that needs
  a wasm build of this branch, and building one on this box is what the
  memory of near-OOM preflights is about. It is covered by
  `narration_tests` in `render.rs` — full narration byte-identical to
  `render_events_in`, the announcement gone and the chemistry kept, the
  event itself untouched, at all three registers — and by CI.


## The volcano that drew no bubbles (GUI-128)

Owner, 2026-09-22: *"we need real animations. foam/explosions must be
visually rendered, parametrised to computed values."* The machinery was
already there and parametrised; three separate things kept it off the
screen, and the first was found by tracing the bench at 100 ms through a
whole catalogue run rather than by reading the code.

- [x] **GUI-128a — the fizz is drawn from the EVENT, not from the steady
  state.** `{#if vessel.bubbling}` gated the gas bubbles, and
  `vessel.bubbling` is a state read off the scene AFTER the step settles.
  An open beaker of vinegar and baking soda evolves 32 mmol of CO₂ and
  then the gas is **gone**, out of the vessel, so the flag is false by the
  time anything is drawn.

  Traced in Chrome over the whole of `vinegar-and-baking-soda`: the bench
  drew dissolving grains, then a heater, and **not one bubble** — while
  the journal beside it reported the carbon dioxide twice. The condition
  is now `vessel.bubbling || active("vent", 4000)`: the effect already
  carried the magnitude and the engine's own production rate, and was
  simply not allowed to draw unless the steady state agreed. Same trace
  after: **10 bubbles for 1.4 s of a 2.5 s run.**

- [x] **GUI-128b — the runner paces to what the step put on the stage.**
  `paceMs` was a flat **420 ms** and every visible effect outlives it — a
  burst is drawn for 1800 ms, a foam head for 3000, a bubble ride for
  9000. Ten lines therefore fired ten animations inside four seconds,
  each wiped by the next before it had drawn: *the original defect this
  runner was written to fix*, surviving in the one number nobody had
  measured against the thing it paces.

  The BENCH answers now — `settleMs()` — because the bench is what knows
  whether that line put anything on the stage. It reports the remainder
  of a 1400 ms window since the newest effect, so a line that started one
  asks for the rest of it and a line that only moved a number asks for
  nothing. The runner caps what it will wait at 1800 ms: a twelve-line
  script honouring a nine-second bubble ride in full would take two
  minutes, and what a learner needs is to see that something happened. A
  bench that cannot answer keeps the flat pace, which is the run that
  shipped before.

- [x] **GUI-128c — the foam head is a foam and the burst is as big as the
  burst.** The head was a coloured rectangle with 5–16 cells on a modulo
  lattice (`(i * 17) % width`), so every foam in the app had the same
  bubbles in the same places, in rows, and none of them moved. Three
  things are read off the engine now: **how much** foam decides the count
  (8 cells at a trace, 42 at a head that fills the glass) and the density;
  a foam **coarsens upward**, so the radius scales with the cell's own
  height in the head rather than with `i % 3`; and a foam that dies in two
  seconds **churns** while one that stands barely moves, so the pop cycle
  is the engine's own half-life divided down — the same number that
  drives `foam-collapse`, said as motion instead of as height. Plus a
  crown of larger bubbles proud of the fill, because the ruled line across
  the top of a rectangle was the single most artificial thing in the
  drawing.

  The burst threw eight identical shards at eight fixed angles however
  hard the seal failed; only the distance and the ring radius moved. The
  **count** is the magnitude now (6 to 20), every shard has its own angle,
  length, size, spin and delay, and there are two staggered rings and a
  flash.

  **The scatter is the index, never `Math.random()`**: a random scatter
  re-rolls on every reactive redraw and the foam twitches, and no
  server-rendered test could assert anything about a picture that is
  different every time.

  **And the scatter had to be two-dimensional, which a photograph caught
  and the first test did not.** The first draft salted ONE golden-ratio
  sequence with an additive offset per axis — and an additive offset of a
  sequence is the same sequence, so x and y were perfectly correlated and
  every bubble sat on a diagonal band through the middle of the head. A
  scatter that is a line is a lattice wearing a different hat. Each axis
  now has its own irrational, the first two being the R2 pair. Measured:
  the correlated version reaches **6 of the 9 cells** of a 3×3 grid, the
  R2 pair reaches **all 9**, and the assertion is 8.

  That assertion also had to exclude the crown, which sits in a row of its
  own along the top and filled buckets the head did not — which is how the
  first draft of it passed against the very scatter it was written to
  reject. The rule both times: a test proven to fail on the defect, not a
  test that merely passes on the fix.


## Completed GUI tasks

Numbers are never renumbered and never reused. Each of these landed; the detail
and the lessons are in `HISTORY.md`.

GUI-002, GUI-005 · GUI-011, GUI-015 · GUI-020, GUI-025, GUI-026, GUI-027,
GUI-028, GUI-029, GUI-033, GUI-053, GUI-055 · GUI-076, GUI-079, GUI-080
(Phase G2.5 numbering) · GUI-058, GUI-061, GUI-062, GUI-063, GUI-064, GUI-065,
GUI-066, GUI-067, GUI-074, GUI-075, GUI-077, GUI-078, GUI-079, GUI-080,
GUI-083a, GUI-083b (realism-bar numbering) · GUI-087, GUI-091, GUI-095,
GUI-096, GUI-097, GUI-105 · GPU-1 … GPU-5a, GPU-6a … GPU-6d · ANIM-1, ANIM-2,
ANIM-3 · I18N-2, I18N-3 · DATA-010, WEB-003. GUI-060 was superseded by
GUI-065 rather than built; its number stays retired.

Note the deliberate collision: GUI-074/075/077/078/079/080 were issued twice,
once in Phase G2.5 and once in the 2026-08-25 realism-bar addendum. Both sets
are recorded under their own dates in `HISTORY.md`; neither is renumbered.

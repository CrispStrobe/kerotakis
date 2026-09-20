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
  above, with the spectators named at lv3. Still open: showing the neutral
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

- [ ] **GUI-112 — A command prompt bar in the GUI, with context-aware
  autocomplete.** The owner wants to type at the bench. `CommandBar.svelte`
  exists and is opt-in from the utilities menu, and `session.parse(line)`
  already validates a line without running it — so the parse-only door the
  autocomplete would need is open. What is missing is the completion model:
  what can follow this verb, in this scene, for this vessel. That is an
  engine-side question about affordances before it is a widget.

- [ ] **GUI-113 — Set the energy amount directly for `Erhitzen`.** Today the
  heat panel takes its energy through the apparatus form's own controls.
  The owner wants to type the number. Related to GUI-112 but separable: it
  is one form field and one validation rule, against a verb that already
  accepts `heat v1 40kJ on burner`.

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

- [ ] **GUI-114 — Apparatus stands on something.** Instruments and
  glassware currently float in the bench pane with nothing under them. A
  bench has a work surface, a stand takes a clamp, a hotplate sits *under*
  a beaker rather than beside it, and a burner belongs below what it heats.
  The physical relationship is already in the model — `ApparatusAssembly`
  and the transport/connection ports know what is attached to what — so
  this is about drawing the relationship that exists rather than inventing
  one. Watch: the bench already has a `work-surface` element and a
  `vessel-position`; start by finding out what they currently do.

- [ ] **GUI-115 — A cupboard of devices, not a wall of text.** GUI-109
  drew ten instruments as inline SVG and left `pH` and `Bq` as letters
  because neither has an honest silhouette. That was the icons. This is the
  cupboard itself: it is still a scrolling list of rows with a drawing at
  the left, which reads as a menu rather than as a place things are kept.
  Shelves, depth, things standing where you would reach for them. The same
  honesty rule applies as for the icons — where a device cannot be drawn
  recognisably, say so rather than ship a shape that means nothing.

- [ ] **GUI-116 — The showpiece reactions, rendered, and scaled by the
  numbers.** A soda volcano should look like one: foam that climbs and
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

  So the gap is **confinement, not absence**. Every effect is drawn inside
  the vessel's own rectangle, and a soda volcano is precisely the case
  where the interesting part does not fit inside the glass: foam that
  climbs past the rim and spills down the outside, steam that rises above
  the beaker, a flare that reaches beyond it. The honest version of this
  item is "let an effect leave the vessel's box when the modelled quantity
  exceeds the vessel, and keep the scaling while it does" — which is a
  much smaller and much better-defined change than a new effect system,
  and it keeps every existing audit of the magnitude link intact.

  Scope note: "explosions" in a school-chemistry bench means a flare, a
  bang, a lid lifting, a flask venting — the engine models energy release
  and gas production, and the drawing must not promise more than the model
  computed. See also the standing rule that this laboratory shows what a
  bench would really do, including refusing to show what it has not
  modelled.

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

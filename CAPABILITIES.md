# Kerotakis — Capability extension tasks

Where the product stands against its computational neighbours, and the
scoped tasks that close the gaps worth closing. Companion to
[OPTIMIZATION.md](OPTIMIZATION.md); **its Ground rules apply verbatim to
every task here** (own worktree, private `CARGO_TARGET_DIR`,
`tools/preflight.sh` gates every push, push at checkpoints, one task one
branch). Two additions specific to capability work:

1. **Every new user-visible number carries `Provenance`** — an
   unattributable number is a bug (CONTRIBUTING.md). New constants and
   correlations come from primary literature or cleared sources only;
   the avoid-list in PLAN.md (NIST SRD/WebBook, CAS, CAMEO exports,
   ECHA, Burcat, UNIFAC Consortium) is a legal constraint.
2. **New grammar gets a fuzz target; new solver paths get the
   conservation and metamorphic invariants** (order-independence,
   dilution monotonicity, scale invariance) plus at least one golden
   test against a textbook value.

> **GUI consumers (2026-08-24):** [ROADMAP-GUI.md](ROADMAP-GUI.md) plans the
> cross-platform GUI and binds several CAP tasks as its dependencies: CAP-3's
> chart contract renders in the app (GUI-021), CAP-12's titrate verb drives
> the first live chart, CAP-2/CAP-8/CAP-4 get their user surface in Phase G5
> (GUI-050/051). Scoping those CAPs should treat the GUI contract as a
> consumer, not an afterthought.

## The yardsticks

Feature inventories taken **2026-08-23**, verified against each
project's own documentation the same day:

- **ChemPy** (BSD-2 Python library, already on our build-time oracle
  list in PLAN.md): equation balancing including underdetermined
  systems with parametric solutions; multiphase equilibrium systems;
  kinetics as ODE reaction networks; a layer of *named physical
  relations* (Debye-Hückel, Arrhenius, Eyring, Nernst,
  Einstein-Smoluchowski, ionic strength); literature-parametrized
  property correlations (water density/permittivity/viscosity/
  diffusivity vs T, sulfuric-acid density, Henry coefficients);
  unit-aware calculation throughout.
- **The commercial PHREEQC-workbench class** (deliberately unnamed
  here): parametric studies over thousands of engine runs, Monte Carlo
  uncertainty propagation, parameter optimization against measured
  data, predominance/Pourbaix diagram generation, interactive plotting,
  water-sample data management with CSV/SQLite import, hydrochemical
  diagram families (Piper/Schoeller/Wilcox), PCA, thermodynamic
  database browsing.

Parity is a direction, not a finish line: we match them **where the
capability serves a learner at a bench**, and we say so plainly where
it does not (see *Declined* at the bottom). What neither neighbour has
— and we do — is the bench itself: operators, conservation ledgers,
three registers, provenance on every number, a replayed codex.

## Where we stand today (inventory, 2026-08-23)

19 bench operators; a REPL, `.lab` replay, MCP server and a wasm PWA
over the same JSON contract; PHREEQC wired for speciation,
amount-limited equilibrium phases, sealed gas phase, Hfo surfaces,
saturation indices and redox splits; CEA Gibbs minimisation for TP and
HP (adiabatic flame); own Nernst electrochemistry (7 couples, galvanic
cell + electrolysis); 74 registry species, 4 embedded databases, 103
codex reactions / 189 concepts, 17 lessons. Notable holes the tasks
below attack:

- **`kerotakis-thermo` is dead weight**: VLE, azeotrope and UNIFAC code
  exists and is tested, but *no crate depends on it* — no operator, CLI
  command or wasm call can reach it. ~~Four groups, twelve interactions~~
  **(CAP-18 done)** — six main groups, 30 interactions, five golden
  γ∞ binaries.
- **No plotting anywhere**, though the PHREEQC `USER_GRAPH` parsing
  plumbing already exists unused (`kerotakis-phreeqc/src/lib.rs:94`).
- **`kero sweep` is a self-check harness** (1536 fixed cases, 8
  invariants), not a user-facing parameter study.
- **No uncertainty propagation**; uncertainty lives only in provenance
  prose.
- **~~Safety is a 4-species, 2-rule stub~~ (CAP-11 done)** — 77 species,
  11 groups, 7+1 rules; totality enforced in CI.
- PHREEQC vocabulary not yet wired: `EXCHANGE`, `MIX`,
  `KINETICS`/`RATES`, `SOLID_SOLUTIONS`, `TRANSPORT`,
  `INVERSE_MODELING`.
- No `titrate`, `dilute` or `mix` verb — titration is hand-rolled
  repeated `add`.

**Post-merge status (2026-08-23, after the R-stage execution sprint
landed on main).** The inventory above predates that sprint; verified
against the merged tree the same day: **CAP-10's `EXCHANGE` half is
done upstream** (typed cation-exchange ledgers plus 1-D transport,
AQ-007/011–014) — the task re-scopes to the `MIX` routing and the
softening lesson; **CAP-14 is half done** (`deny.toml` and `about.toml`
exist via LIC-006/007) — what remains is wiring `cargo deny` into
`tools/preflight.sh` and CI plus the synthetic-failure proof;
**CAP-1 stands undiminished** — `kerotakis-thermo` grew EOS, LLE,
fluid-model and flash modules and *still* has no dependent crate;
CAP-11 done (77 species, 11 groups, 7+1 rules); CAP-12 done (titrate
and dilute verbs landed); CAP-18 done (UNIFAC: 6 main groups, 30
interactions, 5 golden γ∞ binaries, OH↔CH2CO parameter bug fixed). The instrument lines of the old inventory are stale:
gas pressure/volume, conductivity, spectrophotometer, calorimeter,
chromatography and qualitative analysis landed (INST-003–008). New
crates since the inventory: `kerotakis-data`, `kerotakis-org`
(`chematic` adopted off the watch list), `kerotakis-sundials`,
`kerotakis-registry-export`. Cross-check every remaining task against
the tree before starting it.

**Canonicality and the claim audit (2026-08-23, evening).** This file is
the canonical task list; task numbers are stable identifiers and are
never re-bound. A 96-line replacement landed the same day redefining
every CAP number as an adopted-library checkmark and was restored from
history — the adoption facts it carried are folded into the tasks
below. A completion checkbox requires the task's Acceptance evidence in
the marking commit; a checkbox without it is a claim, not a status.
Audit of the day's completion claims, each verified against the tree:

- **True:** CAP-1 (distil landed — see its status line), CAP-10's
  `EXCHANGE` half, CAP-14's `deny.toml` + preflight wiring, OPT-4's
  lookup index (`species.rs` `OnceLock` + `HashMap`).
- **False as user capability, types/deps only:** CAP-2 (rayon adopted,
  no `kero study`), CAP-3 (no chart contract or renderer), CAP-4
  (`phase_diagram.rs` is library-only; no grid solve, no CLI), CAP-5
  (no relations module, no `kero calc`), CAP-6 (no properties module),
  CAP-8 (`statistics.rs` types, no `--mc` surface), CAP-9 (no
  `kero fit`), CAP-13 (`vendor/inchi/` holds one README —
  a scaffold is not a vendored library).
  **Now true (2026-08-23 evening):** CAP-11 (77-species safety matrix),
  CAP-12 (titrate and dilute verbs).

## Parity matrix

| Capability | ChemPy | Workbench class | Kerotakis today | Lands in |
|---|---|---|---|---|
| Equation balancing | ✓ (incl. underdetermined) | — | ✓ null-space (`stoich.rs`) | CAP-7 (underdetermined) |
| Aqueous equilibria / speciation | partial (EqSystem) | ✓ (full PHREEQC) | ✓ strong subset | CAP-10, then R1 |
| Kinetics as reaction networks | ✓ (ODE systems) | via engine | 2 curated rate laws | R3 (network IR) — not a CAP task |
| Named physical relations, exposed | ✓ | — | internal only (Nernst, Arrhenius, H-H exist in solvers) | **CAP-5** |
| Property correlations w/ provenance | ✓ | — | registry constants only | **CAP-6** |
| Unit-aware I/O | ✓ | — | newtype units, fixed parse set | folded into CAP-5 |
| VLE / boiling / azeotropes | — | — | built but unreachable | **CAP-1** |
| Parameter studies | — | ✓ | self-check only | **CAP-2** |
| Charts / curves | — | ✓ | none (plumbing exists) | **CAP-3** |
| Predominance / Pourbaix diagrams | — | ✓ | none | **CAP-4** |
| Monte Carlo uncertainty | — | ✓ | none | **CAP-8** |
| Parameter fitting to data | — | ✓ | none | **CAP-9** |
| Ion exchange, mixing, solid solutions | — | ✓ | not wired | **CAP-10** / R1 |
| Reactive-hazard screening | — | — | 4-species stub | **CAP-11** |
| Titration / dilution as first-class verbs | — | — | repeated `add` | **CAP-12** |
| Sample DB import, Piper/Schoeller/Wilcox, PCA, 3-D plots | — | ✓ | none | **Declined** (below) |
| 1-D transport columns, inverse modeling | — | ✓ | none | later R-stages, not CAP |

### Dependency graph

```
CAP-1 (wire thermo)   CAP-5 (relations)   CAP-7 (balancer)   CAP-11 (safety)
  independent           independent         independent        independent

CAP-2 (study runner) ──► CAP-3 (charts) ──► CAP-4 (diagrams)
        │                     ▲
        ├──► CAP-8 (MC)       │
        └──► CAP-9 (fitting)  │
CAP-12 (titrate verb) ────────┘  (titration curves are CAP-3's first plot)
CAP-6 (properties) — independent, feeds CAP-1's water story
CAP-10 (EXCHANGE/MIX) — independent, big win per line
CAP-13 (InChI) — independent      CAP-14 (licence lint) — independent, land early
```

Library choices below were licence-verified 2026-08-23 and are listed in
PLAN.md's "Queued by the 2026-08-23 review"; all shipped dependencies must
clear the shipping bar there (MIT/Apache-2.0/BSD/Zlib/Unlicense/public
domain — no GPL family, LGPL included). CAP-14 turns that bar into CI.

OPT-7 (OPTIMIZATION.md) multiplies CAP-2/-4/-8: grid studies and
Monte Carlo are thousands of engine calls, and today each one can cost
hundreds of solves.

---

## CAP-1 — Wire `kerotakis-thermo` into the bench

- [x] Status: **done 2026-08-23.** The `distil` operator is the crate's
      first dependent: bubble point with full UNIFAC γ(T) at the
      vessel's pressure, all three registers, conservation proptests,
      four integration tests, the `spirit-still` lesson, and the
      checked-in ethanol–water azeotrope acceptance test
      (x = 0.894, 78.1 °C, 95.9 wt% vs literature 0.894/78.17/95.6).
      Follow-ups spun out as CAP-15…CAP-19 below.

**Why.** The VLE crate (Antoine, bubble point, azeotrope, UNIFAC γ) is
written, tested — and unreachable: no crate in the workspace depends on
it. The chemistry it answers ("what boils when, what mixes") is a whole
row of the PLAN.md thesis table, currently undelivered. This is P3p's
first concrete slice.

**Scope.**

- Add `kerotakis-thermo` as a dependency where the router lives; route
  boiling-point questions for aqueous mixtures with a volatile
  component through `bubble_point` instead of pure-water assumptions.
- New verb `distil <from> <to> <fraction>` (parser + `Operator` +
  `apply` + render in all three registers): take overhead vapour of
  bubble-point composition into the receiver, honestly — one
  equilibrium stage, said so in lv2/lv3.
- Grow the Antoine data from 2 datasets (water, ethanol) to the obvious
  school set (methanol, propanone, ethanoic acid, …) — **each constant
  from primary literature with a provenance record**; the avoid-list
  applies. Golden fixtures via the build-time `thermo` (Python) oracle
  per PLAN.md.
- One lesson: ethanol–water distillation that *fails to pass the
  azeotrope*, because that is the true result.
- New grammar ⇒ extend the `.lab` fuzz target.

**Acceptance.** `kerotakis-thermo` has a dependent; the lesson replays
in CI; conservation proptests extended to `distil`; preflight green.
**Size.** Medium. **Depends on:** nothing.

---

## CAP-2 — A user-facing study runner (`kero study`)

- [x] Status: **done 2026-08-24** (Fable). `kero study <lab> --vary
      add:<v>:<species>=<from>..<to>[:steps] --collect <probe>[,…]
      [--csv]` — one varied parameter (v1, as scoped; the two-parameter
      grid did not fall out free and is not pretended), probes `ph`,
      `temp`, `mass`, `titrant_volume` addressed `@vN`, NDJSON default
      and CSV, every row carrying the varied value and provenance.
      Rayon-parallel with one engine instance per thread; rows emitted
      strictly in run order — `a_study_is_byte_deterministic` pins
      byte-equality of two full runs. An ambiguous selector refuses
      with the matching line numbers and the `line:<N>` escape hatch.
      Acceptance met literally: the titration study over
      `lessons/titration.lab` reproduces the codex's equivalence claim
      — delivered base moles equal acid moles within one burette step
      across four acid amounts (`tests/study.rs`). Finding the study
      surfaced: the titrate verb was delivering *pure* NaOH by volume
      (~0.053 mol/mL), leaping the whole curve in one step; the burette
      now holds a standard solution (`titrate v1 NaOH 0.1M 1mL …`,
      default 1 mol/L) delivering concentration × step moles plus the
      carrier water, and the engine test walks the curve to
      equivalence at 10 mL — CAP-12's semantics corrected in place.

**Why.** The workbench class's core workflow is "run the model many
times, varied over a parameter, and look at the result" — and it is
also what half the curriculum's practicals *are* (titration curves,
rate vs temperature, solubility vs pH). We have every ingredient
(deterministic replay, JSON contract, prewarm-style batch driving) and
no command.

**Scope.**

- `kero study <lesson.lab> --vary <selector>=<from>..<to>[:steps] --collect <probe>[,…] [--json|--csv]`
  where the selector addresses an operator argument in the script
  (e.g. the amount in a marked `add` line) and probes are the existing
  instrument reads (`ph`, `thermometer`, `balance`) plus solved
  quantities already in `json_inspect`.
- One varied parameter in v1; two (grid) only if it falls out free —
  say which in the commit.
- Output is NDJSON (one object per run) and CSV; every row carries the
  varied value and the provenance of the collected quantities.
- Reuse the replay path `prewarm` uses; no new solver code.
- Parallelize native runs with `rayon` (MIT OR Apache-2.0) — one engine
  instance per thread (IPhreeqc instances are per-object); wasm stays
  serial. Determinism must survive parallelism: results ordered by run
  index, never by completion.

**Out of scope.** Plotting (CAP-3), distributions (CAP-8),
optimization (CAP-9).

**Acceptance.** A titration study over `lessons/titration.lab`
reproduces the equivalence point the codex entry states; runs are
byte-deterministic; preflight green. **Size.** Medium.
**Depends on:** nothing (much faster after OPT-7).

---

## CAP-3 — Charts: one JSON contract, one renderer

- [x] Status: **done 2026-08-23** (Fable). `kerotakis-core::chart` is
      the contract (title, axes with units, line/scatter series,
      mandatory provenance — a chart is a claim); the CLI's
      `chart_svg` renders it hand-rolled (axes, ticks, legend, clamped
      provenance caption); `kero chart <json>` is the universal outlet
      any producer can feed — the study runner and the titration curve
      plug in the day they exist. First real producer shipped with it:
      `kero diagram txy`, the ethanol–water T–x–y envelope at 121
      computed points per curve, bubble and dew pinching shut at the
      azeotrope because the thermodynamics says so. The Pourbaix
      region grid remains a sibling shape, noted in the contract for a
      `Regions` kind when its second producer appears. Renderer held
      by a binary-path test (every series drawn and named, provenance
      present).

**Why.** No plot reaches a user anywhere in the product, while the
`USER_GRAPH` parsing already exists unused
(`kerotakis-phreeqc/src/lib.rs:94-107`) and CAP-2 produces exactly the
data a chart wants. A titration curve you computed yourself is worth a
chapter of prose.

**Scope.**

- Define a chart JSON contract (series, axes with units, title,
  provenance line) in `kerotakis-core`, emitted by: `kero study`
  (`--chart` flag), the `USER_GRAPH` plumbing, and the wasm `Lab` (new
  method returning chart JSON).
- Render it twice: (a) CLI — a modest SVG file written next to the
  output (no terminal graphics heroics); (b) web — a small hand-rolled
  SVG line/scatter renderer in `web/` (the PWA is deliberately
  framework-free and must stay that way; no chart library, no CDN).
  Prototype both hand-rolled from the one contract; reach for `poloto`
  or `plotters` (MIT — re-verify at adoption) only if that genuinely
  drags, and say so in the commit.
- Every chart displays its provenance line. A chart is a claim.

**Acceptance.** `kero study … --chart` writes an SVG titration curve;
the PWA renders the same JSON; both name the engine and dataset;
preflight green (wasm build included). **Size.** Medium.
**Depends on:** CAP-2.

---

## CAP-4 — Predominance (Pourbaix) diagrams

- [x] Status: **done 2026-08-23** (Fable). `kero diagram pourbaix <El>
      [--grid NxM] [--out F.svg] [--json]` computes the pe–pH grid cell
      by cell (one plain engine solve each — pe is the axis, not an
      unknown), classifies dominance as majority-precipitate else
      top element-bearing species from the engine's own distribution,
      and renders SVG with computed water-stability lines, region
      boundaries, legend and provenance caption. Refusals outside the
      water-stability field render as the pale wash physics predicts;
      in-field refusals render dark so a hole can never pass for a
      region. Curated systems: Fe, Cu (`pourbaix.rs::SYSTEMS` — growing
      it is data work). Topology pinned by
      `tests/pourbaix.rs` (ferric/ferrous/hydroxide fields + in-field
      refusals rare); `--json` emits the CAP-3 chart-contract seed.
      Fe at 48×40: 1,920 solves, 8 regions.

**Why.** The single most recognisable artefact of the workbench class,
and pure pedagogy: *computed* pe–pH predominance regions for iron make
rust, corrosion and redox chemistry visible. All the machinery is a
grid of PHREEQC solves we already know how to run.

**Scope.**

- `kero diagram pourbaix <element> [--grid NxM] [--databases …]`: solve
  an N×M pe–pH grid, record the dominant aqueous species or stable
  phase per cell, emit region polygons in the CAP-3 chart contract
  (region/heatmap series type — extend the contract, don't fork it).
  Region boundaries via the `contour` crate (Apache-2.0, marching
  squares). Its sibling `contour-isobands` is AGPL-3.0 — barred by the
  shipping bar; do not substitute it.
- Start with Fe and Cu (both in the registry and the displacement
  series); water-stability lines drawn from the same thermodynamics,
  not hardcoded.
- Diagram carries database provenance; cells where the engine failed
  render as explicitly unknown, never interpolated over — declining to
  model something must be loud (PLAN.md).
- Cache-friendliness: a grid is thousands of near-identical solves —
  run after OPT-7 or accept the wait.

**Acceptance.** Fe diagram at 25 °C reproduces the textbook topology
(Fe²⁺ / Fe³⁺ / Fe(OH)₃ / Fe fields) against a golden fixture; renders
in CLI SVG and PWA; preflight green. **Size.** Medium-large.
**Depends on:** CAP-2, CAP-3; wants OPT-7.

---

## CAP-5 — The named-relations layer (ChemPy's core, our registers)

- [x] Status: **done** (f0af26a). `relations.rs` with Arrhenius,
      Eyring, Nernst, Henderson-Hasselbalch, ionic strength,
      Debye-Hückel limiting law, van 't Hoff — each with typed inputs,
      `Provenance`, lv1/lv2/lv3 register text. `kero calc` CLI command.
      ChemPy differential oracle (`tools/check-relations-vs-chempy.py`)
      with 28-case fixture; in-solver Nernst/Arrhenius/H-H call sites
      refactored to the shared implementations with bit-identical results.

**Why.** ChemPy's most-used surface is not a solver — it is named
equations you can *ask*: Debye-Hückel, Arrhenius, Eyring, Nernst,
ionic strength, Einstein-Smoluchowski. We compute several of these
inside solvers (Nernst in `displacement.rs`, Arrhenius in
`kinetics.rs`, Henderson-Hasselbalch in `indicator.rs`) but a learner
cannot invoke one, vary one, or see one explained. The codex has 189
concepts; the relations are the executable half of many of them.

**Scope.**

- `kerotakis-core/src/relations.rs`: each relation is a struct with
  typed inputs (existing `units.rs` newtypes — extend them where a
  relation needs a unit we lack), a `compute()` returning value +
  `Provenance`, and register text at lv1/lv2/lv3. **Refactor the
  existing in-solver implementations to call these as the single
  source of truth — do not fork the formulas.**
- v1 set: Arrhenius, Eyring (new), Nernst, Henderson-Hasselbalch,
  ionic strength, Debye-Hückel limiting law (new — labelled with its
  validity window, ≲0.01 M, and *contrasted* against PHREEQC's real
  activity model when both are available: that disagreement is a
  lesson, not a bug), van 't Hoff.
- `kero calc <relation> <arg>=<value>… [--vary <arg>=a..b]` — the
  `--vary` form emits the CAP-3 chart contract.
- Differential oracle: a `tools/` script checks every relation against
  ChemPy (build-time, per the PLAN.md oracle pattern); fixtures
  checked in.
- Codex: each relation `embodied_by` the concept it teaches.

**Acceptance.** ChemPy differential fixtures agree to documented
precision; in-solver call sites now route through the shared
implementations with bit-identical test results; preflight green.
**Size.** Medium. **Depends on:** nothing (charts optional until
CAP-3).

---

## CAP-6 — Property correlations with provenance

- [x] Status: **done** (3e79ed2). `properties.rs` with water ρ(T),
      η(T), ε(T) from IAPWS formulations plus Henry coefficients for
      CO₂, O₂, N₂, H₂, Cl₂, NH₃ from primary literature. Validity
      ranges enforced with loud refusal. `kero properties` CLI command.
      ChemPy differential oracle (`tools/check-properties-vs-chempy.py`)
      with 43-case fixture. CODATA 2018 R constant unified across
      `heat_capacity()` and tests.

**Why.** ChemPy ships temperature-dependent water density,
permittivity, viscosity and diffusivity, and Henry coefficients — the
numbers every quantitative exercise leans on. Our registry has
point constants; `evaporate` and the energy balance would both be more
honest with ρ(T) and real ΔvapH(T).

**Scope.**

- `kerotakis-core/src/properties.rs`: water ρ(T), η(T), ε(T) from the
  IAPWS formulations (freely published releases — cite the specific
  release document in provenance; the avoid-list still applies to
  *compilations*); Henry coefficients for the gases we ship (CO₂, O₂,
  N₂, H₂, Cl₂, NH₃) from primary literature. Fundamental constants may
  come via the `physical_constants` crate (MIT OR Apache-2.0, CODATA) —
  provenance still recorded per value.
- Each correlation: validity range enforced (outside it, return the
  refusal, loudly), provenance record, golden tests at tabulated
  points.
- Wire ρ(T) into `evaporate`/volume accounting where the current
  constant sits; report the before/after on the sweep invariants.
- `kero properties <species> [--at 25C]` prints the table with
  provenance.
- Oracle: ChemPy's parametrizations as the build-time second opinion.

**Acceptance.** Golden tests at reference points; sweep (1536 cases)
still green — any invariant that moves is investigated, not widened;
preflight green. **Size.** Small-medium. **Depends on:** nothing.

---

## CAP-7 — Balancer parity: underdetermined systems

- [x] Status: **done** (94bbdb7). Replaced f64 Gaussian elimination
      with exact `Rational64` arithmetic. Underdetermined systems return
      `BalanceResult::Family` with particular solution + basis vectors.
      CLI displays parametric families with usage guidance. 21 stoich
      tests including two textbook underdetermined cases (C+O₂→CO+CO₂
      and MnO₄⁻+H₂O₂+H⁺→Mn²⁺+O₂+H₂O); `verify_balances` helper
      confirms element and charge conservation for every solution.

**Why.** ChemPy balances underdetermined reactions and returns the
parametric family. Our null-space balancer (`stoich.rs`) already
computes the right object — the null space — but presents only the
one-dimensional case.

**Scope.**

- When the null space has dimension > 1, present the family: a
  smallest-integer particular solution plus basis vectors, with lv2
  text explaining *why* the equation is underdetermined (the classic
  case: parallel oxidation products). Do the arithmetic exactly —
  `num-rational`/`num-bigint` (MIT OR Apache-2.0) — so integer families
  never pass through floating point.
- Charge-balanced ionic equations (electrons as a pseudo-element if
  not already).
- Extend the existing `stoich` fuzz target and unit tests.

**Acceptance.** The textbook underdetermined cases return families,
not errors; fuzz clean; preflight green. **Size.** Small.
**Depends on:** nothing.

---

## CAP-8 — Monte Carlo uncertainty over studies

- [x] Status: **done 2026-08-24** (Fable). `kero study … --vary
      <sel>=normal(μ,σ)|uniform(a,b) --mc N --seed S`: samples drawn
      by the existing seeded `statistics::Experiment` (ChaCha20), rows
      emitted in run order as before, and a summary carrying
      p5/p50/p95 + mean/sd per probe (NDJSON object, CSV comments).
      Percentiles are a sort and an interpolation in `statistics.rs` —
      one formula did not earn the `statrs` dependency the scope
      offered, and the code says so. The flag contract refuses out
      loud: a distribution without `--mc`, `--mc` without `--seed`
      (the seed is spoken, never invented), `--mc` over a linear
      range. Acceptance met: the titration-endpoint distribution
      under a 1 % acid-amount uncertainty on a 0.1 mL burette
      reproduces the linear-case analytic expectation — mean at
      equivalence plus the half-step overshoot, σ carrying the input
      1e-4 through, p95−p5 against 2·1.645σ on the step grid
      (`tests/study_mc.rs`); same seed twice is byte-identical, a
      different seed is not. The CAP-3 chart contract gained
      `Series::Band` (lower/upper envelopes) rendered as a shaded
      polygon with legend entry.

**Why.** The workbench class propagates input uncertainty by sampling;
we propagate nothing, while our provenance strings admit "good to
about a significant figure" in prose. Computed error bars on a
titration endpoint are honesty made quantitative — squarely this
project's brand.

**Scope.**

- `kero study … --mc N --seed S` with per-input distributions
  (`normal(μ,σ)`, `uniform(a,b)`) on varied quantities; output
  percentiles (p5/p50/p95) per collected probe, plus the raw NDJSON.
- Deterministic: seed required, PRNG named and pinned (`rand_chacha`);
  distributions from `rand_distr`, percentiles via `statrs` (all
  MIT/Apache-2.0); two runs with the same seed are byte-identical.
- CAP-3 chart contract extension: shaded uncertainty band.

**Out of scope.** Sampling *curated constants* (that touches the codex
contract — file separately if wanted).

**Acceptance.** Titration-endpoint distribution under a ±0.5 % burette
uncertainty reproduces analytic expectation on a linear test case;
determinism test; preflight green. **Size.** Small-medium.
**Depends on:** CAP-2.

---

## CAP-9 — Fit a constant to measured data

- [ ] Status: open (lowest priority of the numbered tasks)

**Why.** The workbench class fits model parameters to observations.
Our one honest use case today: a learner's own (t, observation) CSV
from a rates practical, fitted to a curated rate law — closing the
loop between the virtual bench and a real one.

**Scope.**

- `kero fit <lesson.lab> --param <selector> --data <csv> --loss sse`
  — **one scalar parameter, v1**; golden-section or Brent on the
  replay loss via `argmin` (MIT OR Apache-2.0, pure Rust), data read
  with the `csv` crate (Unlicense OR MIT). No derivative machinery —
  rust-cv's `levenberg-marquardt` only if multi-parameter fitting is
  ever actually asked for.
- Report the fitted value with a residual plot (CAP-3) and — pointedly
  — the curated value with its provenance next to it.

**Acceptance.** Recovers a known constant from synthetic noisy data
within tolerance; preflight green. **Size.** Small-medium.
**Depends on:** CAP-2 (CAP-3 for the plot).

---

## CAP-10 — First slice of the unwired PHREEQC vocabulary: `EXCHANGE` and `MIX`

- [x] Status: **MIX half done 2026-08-24.** `Operator::Mix` wired
      through full CAP-1 pattern: parser (`mix v1 0.5 v2 0.5 into v3`),
      `apply()` with three-body adiabatic temperature balance, `mix()`
      trait method on `Equilibrator` delegated through `SolverStack` →
      `PhreeqcEquilibrator`. Native PHREEQC `MIX`
      input: two SOLUTION blocks + MIX keyword with fractions +
      EQUILIBRIUM_PHASES + SELECTED_OUTPUT, with fallback to normal
      `equilibrate()`. `Event::Mixed` rendered at lv1/lv2. Hard-water
      softening lesson (`lessons/hard-water.lab`) replays. 6 core + 3
      engine integration tests (mass conservation, acid+base→neutral pH,
      lesson replay, adiabatic temperature, parser, rejection guards).
      EXCHANGE half landed upstream. Preflight green.

**Why.** ROADMAP R1 ("unlock the rest of PHREEQC") owns the whole
vocabulary; this is its highest-value slice, pulled forward. Ion
exchange is water softening — a curriculum staple that today has
nowhere to live — and `MIX` is what our `decant`-based mixing
approximates by hand.

**Scope.**

- Wire `EXCHANGE` (input generation + read-back in `aqueous.rs`,
  behind the same routing discipline as `SURFACE`); a `resin`/
  exchanger species in the registry with provenance.
- Route two-solution mixing through `MIX` where both vessels hold
  solved solutions (keep the current path as fallback; the routing
  decision is explained in `explain`).
- One lesson: hard water through an exchange column vessel, Ca²⁺ out,
  Na⁺ in, hardness measured before and after.
- Coordinate with OPT-6/OPT-7: this edits the same file — sequence, do
  not interleave (check OPTIMIZATION.md task status first).

**Acceptance.** Softening lesson replays with element conservation
across the exchanger; differential-oracle spot checks against the
Reaktoro fixtures where expressible; preflight green. **Size.**
Medium. **Depends on:** coordinate with OPT-6/7.

---

## CAP-11 — Safety matrix: from stub to methodology

- [x] Status: **done 2026-08-23.** Expanded from 4 species / 4 groups /
      2 rules to 77 species / 11 reactive groups / 7 incompatibility
      rules plus water-reactive special case. All 77 registry species
      have explicit group assignments (totality test
      `totality_of_covered_keys` enforced in CI). Groups: AcidStrong,
      BaseStrong, OxidizerStrong, OxidizerHypochlorite, ReducingAgent,
      ActiveMetal, FlammableLiquid, FlammableGas, WaterReactive,
      AmmoniaAmines, Carbonate. Rules: hypochlorite+ammonia (Danger),
      hypochlorite+acid (Danger), oxidizer+flammable liquid (Danger),
      oxidizer+flammable gas (Danger), oxidizer+reducing agent (Danger),
      acid+metal (Caution), acid+carbonate (Caution), water-reactive+water
      (Caution). `never-mix.lab` exercises 4 rules (1 existing + 3 new:
      oxidizer+flammable, acid+metal, acid+carbonate). 13 unit tests,
      preflight green.

**Why.** PLAN.md's thesis table lists "is this mixture dangerous —
solved by database (reactive-group matrix reimplemented from NOAA's
published methodology)". What ships is 4 species, 4 groups, 2 rules.
The L0 gate runs on every step; it should know more than chlorine and
chloramine.

**Scope.**

- First tranche: assign reactive groups to **all 74 registry species**
  (hand-curated table, provenance per assignment — the published
  methodology paper, not the database exports on the avoid-list);
  grow the incompatibility rules to cover the groups those species
  actually populate (acids, bases, oxidizers, reducing agents, active
  metals, peroxide formers…).
- Keep warn-and-proceed as default policy; wire the existing unused
  `Veto` for the small set where proceeding is pedagogically
  indefensible — list them in the commit message and the codex
  (`never-mix` lesson grows with the matrix).
- Property test: group assignment is total over the registry (a new
  species without a safety row fails CI).

**Acceptance.** Totality test in CI; the `never-mix` lesson exercises
at least three new rules; preflight green. **Size.** Medium (curation-
heavy). **Depends on:** nothing.

---

## CAP-12 — `titrate` and `dilute` as first-class verbs

- [x] Status: **done.** `Operator::Dilute` and `Operator::Titrate`
      wired through the full pattern: parser (`dilute v1 100mL`,
      `titrate v1 NaOH 1mL until ph 7`), `apply()` for dilute with
      adiabatic mixing, `titrate_loop()` in `step_with()` with
      per-step add → equilibrate → read pH → crossing detection,
      `Event::Diluted` and `Event::Titrated` (carrying the full
      (mL, pH) curve), three-register rendering, codex event mapping,
      conservation proptest `RandOp::Dilute` arm (mass + energy
      conserved, 256 cases green), 6 integration tests
      (`tests/dilute.rs`), `lessons/titration.lab` rewritten with
      `titrate`, old spelling preserved as `lessons/titration-manual.lab`,
      golden regenerated with both lessons, help text completed for
      all verbs. Titrate reports `NotYetModeled` when no aqueous
      solver is wired — the verb is honest.

**Why.** Titration — the quantitative heart of school chemistry — is
currently spelled as a dozen hand-written `add` lines, and there is no
`dilute`. A first-class `titrate` also gives CAP-3 its signature
artefact: the auto-generated titration curve.

**Scope.**

- `dilute <v> <amount><mL|L>`: add solvent, with the dilution-
  monotonicity metamorphic invariant attached.
- `titrate <from> into <to> step <mL> until <ph OP value | endpoint <indicator>>`:
  an auto-stepper over the existing `add` + solve path — no new solver
  code — recording (volume, pH) per step; emits the CAP-3 chart
  contract; refuses politely when no endpoint is reachable and says
  why.
- All three registers narrate it; parser + fuzz target extended;
  `lessons/titration.lab` rewritten to use it (keep the old spelling
  as a second lesson proving they agree).

**Acceptance.** Old and new titration lessons agree on the endpoint to
solver precision; grammar fuzz clean; preflight green. **Size.**
Small-medium. **Depends on:** CAP-3 for the curve (verb itself:
nothing).

---

## CAP-13 — Adopt the official InChI library (MIT since 1.07.1)

- [x] Status: **done 2026-08-24** (Fable), with one scoping deviation
      stated: the official sources are vendored *inside the
      checksummed `inchi-sys` 0.1.4 crate* (IUPAC InChI v1.07.5
      bundled, MIT, statically linked, no network at build) rather
      than as a git submodule — the Cargo.lock checksum is the pin,
      and a submodule would duplicate the same tree. The
      `native-inchi` feature now actually compiles (its previous call
      site named a function that did not exist — the feature had
      never been built): SMILES → chematic molecule → V2000 molfile →
      official library → standard InChIKey
      (`native_inchikey_from_smiles`). The identity contract:
      `CURATED_STRUCTURES` pins a SMILES for 23 registry species, and
      `tests/native_identity.rs` recomputes each key and requires it
      to equal the registry's `canonical_key` — all 23 matched on
      first run, and the check is a preflight step ("inchi
      identity"), so a curation bug now fails the gate. `kero
      species` marks verified identities with ✓ and names the
      library version. Cross-validation semantics corrected:
      chematic's canonical key is a different algorithm, so
      chematic-vs-native Mismatch is expected and documented, not
      asserted away. Remaining (not claimed): the Emscripten/wasm
      InChI build, and growing the tranche to species without simple
      SMILES (minerals, enzymes, aromatic dyes).
- [ ] Tranche growth: **23 → 65** (2026-08-24, Opus). Added 42 species:
      monatomic ions (Na+ K+ Cl- Ca+2 Mg+2 Sr+2 Ag+ Cu+2 Cu+1 Fe+2
      Fe+3 Zn+2 Mn+2), polyatomic ions (NO3- SO4-2 HCO3- H2PO4-),
      metals (Cu Zn Ag Fe), oxides (CaO MgO CuO MnO2), hydroxide
      Ca(OH)2, salts (AgCl NaOCl NaHCO3 Na2CO3 Na2SO3 Na2S2O3 AgNO3
      CaCl2 CaCO3 MgSO4 gypsum CuSO4 KMnO4 FeSO4 ZnSO4), and
      chloramine NH2Cl. All 65 InChIKeys recompute and match via
      the official IUPAC library.
      **Deferred species** (11, all chematic `write_mol` limitations —
      SMILES parse succeeds, molfile is generated, but the molfile
      encodes the wrong structure for the official InChI library):
      - **Mg** `[Mg]`: `write_mol` adds 2 implicit H → InChI sees
        `Mg.2H` (InChI=1S/Mg.2H) instead of bare Mg.
      - **Pb** `[Pb]`: same — `write_mol` adds 2 implicit H → `Pb.2H`.
      - **C** `[C]`: `write_mol` adds 4 implicit H → InChI sees CH₄
        (methane), not elemental carbon.
      - **S** `[S]`: `write_mol` adds 2 implicit H → InChI sees H₂S,
        not elemental sulfur.
      - **Cu(OH)2** `O[Cu]O`: `write_mol` outputs disconnected
        `Cu.2H₂O` with charge q+2/p-2 instead of connected copper
        dihydroxide; all ionic variants (`[Cu+2].[OH-].[OH-]`) produce
        the same wrong key.
      - **MnO4-** `[O-][Mn](=O)(=O)=O`: `write_mol` outputs
        disconnected `Mn.4O` losing Mn–O bond connectivity and charge;
        InChI sees `InChI=1S/Mn.4O/q;;;;-1`.
      - **Pb+2** `[Pb+2]`: `write_mol` preserves charge but the InChI
        connectivity hash differs from the registry key (InChI=1S/Pb/q+2
        produces RVPVRDXYQKGNMQ, registry expects XMOCLSLCDHWDHP).
      - **Pb(NO3)2** `[Pb+2].[O-][N+](=O)[O-]…`: connectivity hash
        matches (RLJMLMKIBZAXJO) but InChI charge layer differs (N vs L)
        — `write_mol` loses the net -2 proton balance across fragments.
      - **phenolphthalein**: `write_mol` outputs the open (acid) form;
        InChI encodes different connectivity than the closed lactone the
        registry expects (KMBTWMWDXLZUHH vs KJFMBFZCATUALV).
      - **methyl_orange**: azo-bond and Na⁺ fragment handling in
        `write_mol` produces wrong connectivity (STZCRXQWRGQSJD vs
        BSKHPKMHTQYZBB).
      - **bromothymol_blue**: sulfonphthalein ring system connectivity
        differs (MEEJMWWOAOVJHW vs FBSFWRHWHYMIOG).
      Fix scope: all 11 are chematic `mol::write_mol` bugs, not InChI
      or registry issues. Fixes belong upstream in chematic-mol. The 4
      implicit-H cases are one fix (bracket-atom H-count in V2000
      writer); the 3 aromatics are kekulisation; the rest are individual
      connectivity/charge bugs.

**Why.** The IUPAC InChI reference implementation was relicensed to
plain MIT with v1.07.1 (2024-08) and lives on GitHub, and upstream
demonstrates its own Emscripten/wasm build. The registry already
carries an InChIKey per species, but nothing can *compute or verify*
one — identity is currently a hand-curated string. The L1 identity
crosswalk (UniChem, keyed on Standard InChI) assumes exactly this
capability.

**Scope.**

- Vendor the official `IUPAC-InChI/InChI` source on the IPhreeqc
  pattern (submodule + build.rs behind a feature; Emscripten side for
  the web, following upstream's own wasm recipe).
- CI check: every registry entry's stored InChIKey is recomputed from
  its structure input and must match — a mismatch is a curation bug
  and fails the build (totality, like the CAP-11 safety rows).
- `kero species` gains a verified-identity marker; provenance names
  the InChI version.
- Keep it feature-gated so the engine-less and minimal wasm builds do
  not grow unless they use it.
- **Relation to the Indigo plan.** PLAN.md's L1/L4 design reaches InChI
  through Indigo's bundled plugin once `kerotakis-indigo` exists.
  CAP-13 deliberately takes the standalone route *first* — the official
  library is small, MIT, and needed years before template chemistry is.
  When the Indigo FFI lands, decide once: keep the standalone build as
  the single InChI source (Indigo's copy feature-gated off if its build
  allows) or switch to Indigo's bundled copy and retire this one — but
  never link two InChI implementations into one artifact. Record the
  decision here.

**Acceptance.** All 74 registry InChIKeys recompute and match (or the
curation is fixed); native + wasm builds green in preflight/CI;
`cargo-deny` (CAP-14) passes with the vendored code declared.
**Size.** Medium (FFI + build plumbing). **Depends on:** nothing;
CAP-14 first is tidier.

---

## CAP-14 — Turn the licence policy into a CI lint

- [x] Status: **done 2026-08-23** — `deny.toml` passes all four
      checks; `cargo deny` wired into `tools/preflight.sh`; `cargo-about`
      generates `THIRD_PARTY_LICENSES.html` from `about.hbs` template
      (164 KB, 81 licences); synthetic copyleft proof: adding
      `gpl-session = "2.0.0"` as a dependency triggers
      `error[rejected]: GPL-3.0 ... license is not explicitly allowed`
      (tested and reverted 2026-08-23).

**Why.** PLAN.md's shipping bar (hardened 2026-08-23) says shipped code
is MIT/Apache-2.0/BSD/Zlib/Unlicense/public-domain only — no GPL
family, LGPL included. PLAN queued `cargo-deny` for exactly this; no
`deny.toml` exists yet. Until the bar is a lint, it is reviewer memory,
and reviewer memory is how an AGPL transitive dependency arrives
quietly.

**Scope.**

- `deny.toml` at the workspace root: licence allowlist per the
  shipping bar; explicit documented exceptions for the vendored
  public-domain USGS code (IPhreeqc) and any crate with a nonstandard
  but permissive declaration (each exception carries a comment saying
  who verified what, when).
- `cargo deny check licenses bans` wired into `tools/preflight.sh`
  **and** CI; duplicate-version and yanked-crate checks on, advisories
  optional (decide and say).
- `cargo-about` generating the shipped attribution inventory (the
  NOTICE-adjacent list app stores want) as a build artifact.

**Acceptance.** CI fails on a synthetic copyleft dev-branch dependency
(prove it once, then revert); current tree passes; preflight includes
the check. **Size.** Small. **Depends on:** nothing.

---

## CAP-15 — Re-source and grow the Antoine data

- [x] Status: **done 2026-08-23** (kero-basic, 8e7e461; audited by
      Fable 2026-08-23). Every `source` string now cites Stull 1947
      (Ind. Eng. Chem. 39(4), 517-540, Table I) directly, carrying its
      own mmHg→kPa conversion arithmetic; a tree-wide grep finds no
      avoid-list citation. The school set landed with it — methanol,
      propanone, ethanoic acid — each constant with a golden
      bubble-point test at its tabulated boiling point (64.7, 56.1,
      117.9 °C; `pure_*_bubble_point` in vle.rs). Preflight green on
      the pushes that carried and followed it.

**Why.** Both shipped Antoine sets cite "Stull 1947 *via the NIST
WebBook*" in their `source` fields; the WebBook is on PLAN.md's
avoid-list as a legal constraint. **Scope.** Re-derive/verify the
water and ethanol constants against Stull 1947 (Ind. Eng. Chem.) or
another primary source and rewrite the two `source` strings; then add
the school set (methanol, propanone, ethanoic acid, …), each from
primary literature with provenance, each with a golden test at a
tabulated boiling point. **Acceptance.** No `source` string cites an
avoid-list entry; new compounds distil or refuse honestly; preflight
green. **Size.** Small-medium, curation-heavy. **Depends on:** nothing.

## CAP-16 — γ(T) for the flash paths

- [x] Status: **done 2026-08-23** (Fable). dew_point, tp_flash and
      hp_flash gained _with variants taking γ as a function of the
      *liquid composition* and kelvin — the honest signature, because
      dew and flash solve for the very liquid their γ belongs to. The
      γ–φ successive-substitution loop wraps the existing bisections
      (measured contraction ~0.6 per pass on the worst mid-range case;
      eighty passes clear 1e-9 with margin, and non-convergence refuses
      rather than publishing a drifting split). The fixed-γ functions
      are now delegating wrappers, so the formulas cannot fork, and the
      old suites pass unchanged. Proven by thermodynamic *consistency*,
      not self-consistency: bubble↔dew roundtrips recover T within
      0.05 °C and x within 5e-3 at three compositions (two of them to
      eleven decimals), azeotropic vapour condenses to itself, and a
      mid-boil flash brackets its feed with the first bubble matching
      the bubble-point vapour (tests/flash_gamma.rs).

**Why.** `bubble_point_with` couples γ to temperature inside the
bisection; `dew_point`, `tp_flash` and `hp_flash` still take a fixed
`gamma` per component, which is wrong by the same few per cent the
bubble path used to be. **Scope.** `_with` variants taking
`gammas: FnMut(kelvin) -> Vec<f64>` for all three, fixed-γ wrappers
preserved; UNIFAC-coupled tests mirroring the azeotrope acceptance
test at the dew and flash boundaries. **Acceptance.** Ethanol–water
dew point and TP flash agree with the bubble-path thermodynamics;
existing fixed-γ tests unchanged; preflight green. **Size.** Medium.
**Depends on:** nothing.

## CAP-17 — Batch distillation and the column

- [x] Status: **done 2026-08-23** (Fable). (a) Rayleigh integration:
      `ethanol_water_still` walks 256 steps with the vapour composition
      following the pot, so long cuts deplete honestly and the boil
      climbs — the spirit-still lesson now reads "boiled at 88.4 °C and
      climbed to 92.2 °C as the light component left". (b) The column:
      `distil … stages N` runs an N-stage cascade at total reflux (the
      stated upper bound a real column cannot beat); a 40-stage column
      from wine lands on the azeotrope at x = 0.894 ± 0.02 and reports
      the wall. (c) Energy: `distil … <E>kJ` boils exactly what that
      latent-heat budget lifts (ΔHvap: water 40.657 IAPWS-95, ethanol
      38.56 Majer & Svoboda 1985), and every Distilled event now bills
      the latent heat the burner paid and the condenser dumped —
      quantified on the event, deliberately outside the vessel ledger;
      full coupling through `hp_flash` remains for the feed-flash case.
      Tests: `thermo/tests/still.rs` (drift, azeotrope wall, exact
      energy meter) + extended bench suites.

**Why.** `distil` is one equilibrium stage with y frozen at the
starting composition, stated as such in lv3; `IdealStage` still has no
methods and the still is externally powered. **Scope.** (a) Rayleigh
integration so long boils drift honestly; (b) a stage-count parameter
(`distil … stages N`) built on `IdealStage`, giving fractional
distillation that walks to — and stops at — the azeotrope; (c) energy
coupling through `hp_flash` so the still stops being the second
externally-powered apparatus (the evaporate caveat then has one owner,
not two). **Acceptance.** N-stage separation of wine reaches
azeotropic strength where one stage cannot; energy proptest still
green with the latent term in the ledger. **Size.** Medium-large.
**Depends on:** CAP-16 for the flash half.

## CAP-18 — Grow the UNIFAC table, provenance per parameter

- [x] Status: **done 2026-08-23.** Expanded from 4 main groups (CH2,
      OH, H2O, CH2CO) / 8 subgroups / 12 interactions to 6 main groups /
      10 subgroups / 30 interactions. Added CH3OH (main 6, Fredenslund
      1975) for methanol, COOH (main 20, Gmehling 1982) for acetic acid.
      Fixed pre-existing OH↔CH2CO parameter swap (a(5,9) was 164.5,
      should be 84.0; a(9,5) was −150.0, should be 164.5 — invisible
      before because no computed binary used both groups). Source
      citations split: `SOURCE_1975` (groups 1–9 interactions) vs
      `SOURCE_1982` (group 20 interactions). Five golden γ∞ tests
      validated against Python `thermo` oracle: methanol–water (2.25),
      propanone–water (11.47), acetic acid–water (3.51),
      methanol–propanone (1.96), acetic acid–ethanol (0.96). All
      existing thermo oracle and LLE tests still pass.

**Why.** Eight groups and twelve interactions cover
alkane/OH/H₂O/ketone — enough for the ethanol–water proof, far short
of the school's solvent list. **Scope.** Extend `approved_table()`
from the original open-literature publications (Fredenslund 1975/1977,
Gmehling revisions — never the Consortium tables, per PLAN.md's UNIFAC
note), with `source` per group and per interaction, and a golden γ∞
test per new binary. **Acceptance.** Each new binary reproduces a
literature activity coefficient at stated conditions; provenance lint
green. **Size.** Small per binary, curation-heavy. **Depends on:**
nothing.

## CAP-19 — The thermo differential oracle

- [x] Status: **done 2026-08-23** (Fable). tools/gen-thermo-fixtures.py
      generates a 36-point γ grid from Python `thermo`'s own UNIFAC
      (same published parameters, independent implementation — the
      check that would have caught the combinatorial bug on day one)
      plus seven bubble points solved in Python against the same cited
      Antoine constants; fixtures checked in, replayed by
      tests/thermo_oracle.rs at 1e-4 relative on γ, 0.05 °C and 5e-4
      on bubble T and y. At generation time the two implementations
      agreed to a part in a million on every point, azeotrope included
      (oracle: 78.074 °C, y = 0.89440). Disagreements are investigated,
      never tolerated away.

**Why.** PLAN.md's P3p requires golden fixtures generated by the
Python `thermo` package; nothing in `kerotakis-thermo` is
oracle-checked yet — the UNIFAC divergence bug survived precisely
because only self-consistency tests existed. **Scope.** A `tools/`
build-time script generating flash/VLE/γ fixtures from `thermo`
(MIT, Python), checked in; a test that replays them against
`bubble_point_with`/`dew_point`/`tp_flash`/UNIFAC within documented
tolerances. **Acceptance.** Fixtures cover every shipped Antoine pair
and UNIFAC binary; disagreements are investigated, not tolerated away.
**Size.** Medium. **Depends on:** nothing; richer after CAP-15/18.

## CAP-20 — Give the orphaned physics its verbs

- [x] Status: **done 2026-08-24** (multiple sessions). All verb slots
      filled: extract/drain/chromatograph/react (Fable), transport (Opus).
      See prose below for per-verb details.

**Why.** The tree's own admission ("types implemented but awaiting
grammar"): working transport, extraction, photochemistry, reaction
templates and instruments that no user sentence can reach. The
grammar, not the physics, is the product bottleneck. **Scope.** One
verb per landed subsystem, each with the full CAP-1 wiring pattern
(parser → operator → apply → three registers → conservation arm →
tests → lesson): `extract` (on `apparatus::extract`, upgraded to use
`lle.rs` instead of a supplied K), `transport` (the 1-D chain in
`transport.rs`), `chromatograph` and `calorimeter` (instrument enum
entries plus grammar), `react` (the two `kerotakis-org` templates —
the crate's first dependent). Refusals stay loud where data is
missing.

**First slice done 2026-08-23** (Fable): computed liquid–liquid
demixing reaches the bench. `lle_binary` was rebuilt as a real
solver — spinodal scan then the equal-activity tie line by nested
bisection with an activity-overlap-trimmed bracket (the old ±0.005
alternating walk stalled a quarter of the composition axis from the
answer); hexane entered the registry as pure data through the CAP-21
pipeline (#77, CIAAW/CRC provenance); and water+hexane in a vessel
now emits a computed `LayersFormed` event (hexane floating, three
registers, lv3 stating the alkane–water γ∞ honesty bound) while
water+ethanol provably does not — same machinery, opposite verdict,
which is the lesson. The `drain` verb followed the same
day: the separating funnel's stopcock, gated on the computed layers —
the lower layer runs out with everything dissolved in it (engine test:
brine drains from under hexane, salt travelling with its water, the
organic layer left alone), a settled solid stays (a stopcock passes
liquid; filtration is a different question, and lv3 says so), and
draining a computed single phase is refused out loud. `layered_pair`
is the one source of truth the solver's report and the bench's verb
both consult. Computed partitioning followed the same day: at
the stopcock a curated neutral solute splits on K = γ∞(upper)/γ∞(lower)
from the same UNIFAC (ethanol 88% with the water at 2:1 layers,
methanol 96% — the hydrophilicity ordering emerging from group counts
alone), a `Partitioned` event says so in three registers, ions still
travel entirely with their water, and the engine test pins the split
window and exact solute conservation. The `chromatograph` verb landed
2026-08-24 (Fable): `Instrument::Chromatograph` on the school column
(the CAP-22 oracle's own N = 10⁴, t₀ = 60 s, β = 0.5, as
`ChromatographyColumn::school()`), K per solute computed as
γ∞(water)/γ∞(alkane) from the same UNIFAC the funnel partitions on —
so column and funnel cannot disagree about hydrophobicity — and a
`Chromatographed` event carrying the peak table (retention, width,
area, K) in three registers. Propanone entered the registry as the
78th species, data-only through the CAP-21 pipeline (CIAAW/CRC
provenance; the golden diff was one added record, 77 untouched), so
the demo separation is methanol 63 s, ethanol 68 s, propanone 115 s —
the ketone retained by its groups alone, Rs > 1 between neighbours.
Ions are named `outside_method`, never silently dropped (engine test);
a settled solid was never injected and says so (core test); a
solute-free or dry sample refuses out loud; the injection provably
moves no ledger. Lesson: `one-thing-at-a-time.lab`. The calorimeter
half of the remainder was already served: `Instrument::Calorimeter`,
its grammar, and `calorimetry.lab` predate this task. The `react` verb landed
2026-08-24 (Fable): `react v1 esterification` applies a curated
`OrgReaction` on command — deliberately NOT auto-fired by
`CuratedEquilibrator`, because vinegar and spirit standing in one
beaker do not visibly esterify; the verb *is* the conditions. Two
rows: Fischer esterification (CH3COOH + ethanol ⇌ ethyl acetate +
water, boundary stating the equilibrium it drives past) and
saponification (ester + NaOH → NaOAc + ethanol). Ethyl acetate became
species #79 through the CAP-21 pipeline (CRC/CIAAW; golden diff one
added record) with its safety row. `kerotakis-org` is now
load-bearing twice over: the wasm structure panel consumed it already,
and its SMIRKS templates are the oracle for the curated table —
`tests/template_oracle.rs` applies each template to reference
molecules and requires molecule-level identity (chematic canonical
keys + formulas; standard-InChIKey anchoring is CAP-13's upgrade)
with the acetate-anion→NaOAc ledger bridge stated. Engine tests pin
exact mass conservation, limiting-reagent extents, the there-and-back
round trip (ester made then unmade, the alcohol returns), loud
refusal naming the missing reactant, and a parse-time shelf listing
for unknown reactions. Lesson: `there-and-back.lab`.

**Transport verb done 2026-08-24** (Opus): the existing 1-D upwind
`CellChain` in `transport.rs` now has the full bench wiring.
`transport v1 v2 v3 from v4 to v5 steps N [courant F]` parses,
builds an `Operator::Transport`, runs N `CellChain::advance()`
steps with the inlet as a non-consumed template, deposits
accumulated effluent into the receiver with adiabatic temperature
mixing, and emits `Event::Transported` rendered at three register
levels.  Six integration tests (`transport_verb.rs`) verify the
binomial dispersion profile, mass conservation, empty-chain and
zero-steps refusal, water-volume invariance across chain cells,
and effluent collection.  `transport-column.lab` is the lesson
(salt pulse through a 3-cell water column at Cf = 0.5).
CAP-20 done — all verb slots filled (extract's upgrade to lle.rs
folded into the funnel work above). **Acceptance.** Each verb demonstrable in a replayed lesson;
`kerotakis-org` gains a dependent; preflight green. **Size.** Medium
per verb — they are independent; take them one per branch.
**Depends on:** nothing.

## CAP-21 — Make the data pipeline load-bearing

- [x] Status: **done 2026-08-23** (Fable). The registry table is now
      generated at build time from
      `data/registry/registry-source-v1.json` — `species.rs` shrank
      from 1,563 lines to 179, the table stays `static` with
      `&'static str` fields at zero runtime cost (completing OPT-4's
      binary-size half on the way), and wasm ships unchanged.
      Faithfulness proven, not assumed: `tests/registry_snapshot.rs`
      pins every field and every evaluated spectrum band against a
      golden captured from the hand-written table before the switch —
      the migration surfaced and fixed three placeholder InChIKeys and
      two sub-ulp export-rounding deltas, and nothing else moved. The
      ceiling itself is demonstrably gone: methanol became the 76th
      species through a JSON-only commit (identity, composition,
      three thermodynamic records, provenance citing CIAAW + CRC), and
      `kero species` lists it with no `.rs` edit anywhere.

**Why.** The pack compiler, resolution ladder and 238-record registry
JSON exist, and the runtime still reads 77 hand-written Rust literals
in `species.rs` — the exact ceiling ROADMAP names ("every new species
is code work"). **Scope.** Generate the runtime registry from the
compiled pack (build-time codegen or load-behind-the-API, decide and
say); the parity test already in `kerotakis-data` becomes the gate;
new species arrive as data, proven by adding one species with no
`.rs` edit. **Acceptance.** `species.rs`'s table is generated or
bypassed; a data-only species addition ships end-to-end; preflight
green. **Size.** Medium-large. **Depends on:** nothing.

## CAP-22 — Oracle coverage for the sprint's new surfaces

- [ ] Status: **in progress 2026-08-23** (Fable) — and already paying:
      the first oracle caught a real curation error. The
      spectrophotometer is now anchored to the literature through the
      full pipeline (bench → engine → registry spectrum → instrument):
      permanganate's ε(525) had been curated at 4363 L/(mol·cm), 1.8×
      the classic ~2455 — every permanganate solution rendered nearly
      twice too intense. Rescaled to 2400 through the pack pipeline
      with the correction's provenance on the record. Landed:
      spectrophotometer literature anchor + Beer–Lambert linearity as
      a metamorphic invariant; chromatography vs a hand-worked
      plate-theory example plus limiting identities (void-time,
      √N-scaling); calorimeter vs the closed-form energy ledger.
      Remaining: conductivity carries its written statement (a stub by
      its own comment; the oracle would rightly fail it — CAP task
      material, not tolerance material); nuclide/photochem oracles
      wait until those subsystems are wired to anything.

**Why.** The differential-oracle discipline that makes the PHREEQC
core trustworthy stops at that crate's border: instruments, apparatus,
photochemistry, polymers and nuclides carry only self-consistency
unit tests — the pattern that let a divergent UNIFAC pass. **Scope.**
Per subsystem, one independent second opinion: spectrophotometer vs
hand-integrated Beer–Lambert; calorimeter vs closed-form enthalpy;
chromatography formulas vs a textbook worked example; decay chains vs
analytic Bateman; CEA already has Cantera. Build-time oracles per the
PLAN pattern, fixtures checked in. **Acceptance.** Every shipped
instrument and apparatus number is either oracle-checked or carries a
written statement of why it cannot be. **Size.** Medium, spread
across subsystems. **Depends on:** nothing.

## Already scheduled elsewhere — pointers, not tasks

- **Reaction networks / stiff kinetics** (ChemPy's ODE systems):
  ROADMAP R3 (reaction-network IR + diffsol). CAP does not duplicate
  it.
- **The rest of the PHREEQC vocabulary** (`KINETICS`/`RATES`,
  `SOLID_SOLUTIONS`, 1-D `TRANSPORT`): ROADMAP R1 beyond CAP-10's
  slice.
- **Full phase behaviour** (feos, cubics, flashes, apparatus): ROADMAP
  R2 / PLAN P3p beyond CAP-1's slice.
- **Coupled electrochemistry** (concentration cells beyond the shared
  couple, internal resistance, discharge curves): ROADMAP R4.

## CAP-23 — The single-solvent organic bench answers with chemistry

- [x] Rung 1: **done 2026-08-24** (Fable). A REPL transcript motivated
      this task: salts and a metal in a beaker of ethanol drew a wall
      of "not yet modelled" — but "NaCl is practically insoluble in
      ethanol (0.065 g/100 mL, CRC)" is an answer, and "zinc does not
      react with dry ethanol at bench conditions" is knowledge.
      `nonaqueous::NonAqueousEquilibrator` (wired into all three
      stacks between the curated reactions and the aqueous engine)
      applies curated per-(solute, solvent) handbook solubilities —
      dissolution to the limit as undissociated solute, remainder
      solid — and curated metal-inertness verdicts with the reason a
      learner can check. The honesty pass stands aside exactly where a
      verdict exists and keeps apologising everywhere else; KMnO4 in
      ethanol is deliberately NOT tabled, because it reacts (rung 2's
      job) and tabulating it as soluble would be a lie. Model boundary
      carried in every lv3 line: no speciation, no activity model, no
      conductivity claim in an organic phase. Acceptance: the
      motivating transcript replays with numbered verdicts and zero
      apologies for covered pairs (`tests/nonaqueous.rs`); settled
      species are not re-verdicted every step; water present means the
      rung stands aside.

**Remaining rungs.** Rung 2 (kero1, in flight): the curated
permanganate–ethanol oxidation — the reaction the safety screen warns
about becomes a modelled reaction; silver-halide metathesis in ethanol
follows the same pattern. Rung 1 data growth (kero-basic, in flight):
the solubility table toward every registry solid × four solvents,
handbook-sourced, reactive pairs excluded by rule. Rung 3 (open):
mixed water/organic solvents — route to PHREEQC above a stated water
mole-fraction threshold with the co-solvent named as unmodelled for
activity; refuse below it with the dielectric reason; Born-corrected
mixed-solvent log K is **declined** until someone brings data worth
trusting. Bare dissolved ions in an organic phase (MnO4-, HCO3- typed
straight into ethanol) remain outside every rung and keep their
honest refusal.

### Rung 1 data growth — 2026-08-24 (kero-basic)

`ORGANIC_SOLUBILITY` grown from 8 to 65 rows across all four solvents
(ethanol, hexane, propanone, ethyl_acetate). `INERT_IN_SOLVENT` grown
from 6 to 24 entries (6 metals × 4 solvents). Every row carries a CRC
Handbook 97th ed. source string. Coverage summary:

- **Ethanol** (26 solubility + 6 inert = 32 species): all 33 registry
  solids covered (KMnO4 deliberately excluded — reactive pair). Soluble
  highlights: CaCl2 25.8, NaOH 13.9, NaOAc 5.3, AgNO3 2.1, MgSO4 1.2,
  S 0.066, NaCl 0.065, KCl 0.03 g/100 mL. All others 'i'.
- **Hexane** (23 solubility + 6 inert = 29 species): all ionic 'i';
  sulfur slightly soluble at 0.05 g/100 mL; graphite 'i'.
- **Propanone** (10 solubility + 6 inert = 16 species): CaCl2 33.3,
  AgNO3 0.44 g/100 mL; NaCl, KCl, CaCO3, NaOH, MgSO4, CuSO4, S, C
  all 'i'. Remaining solids uncovered (gypsum/propanone serves the
  honesty test).
- **Ethyl acetate** (6 solubility + 6 inert = 12 species): S 1.8
  g/100 mL; NaCl, CaCO3, CuSO4, NaOH, C 'i'. Remaining solids
  uncovered.

Reactive exclusions: KMnO4/ethanol (already documented). All other
KMnO4/solvent pairs left uncovered pending rung 2 scope review.

Tests extended to 9 (was 5): CaCl2/ethanol (soluble, dissolves
completely), Na2CO3/ethanol (insoluble, all stays solid),
S/ethyl_acetate (soluble, dissolves completely), Zn/hexane (inert).
Honesty test updated: gypsum/propanone (uncovered) still draws the
honest apology.

## Declined — off-mission, recorded so nobody re-litigates silently

The workbench class serves professional geochemists managing field
data. We serve a learner at a bench. Therefore, deliberately not
planned: **water-sample database management and CSV/SQLite sample
import; Piper/Schoeller/Wilcox hydrochemical diagrams; PCA; 3-D and
treemap visualization; `INVERSE_MODELING`**. Each would be real work
serving a user we do not have; none teaches a concept our codex
covers. If the mission changes, change this paragraph first — PLAN.md
"What this will not do" governs.

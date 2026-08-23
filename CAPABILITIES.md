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
  command or wasm call can reach it. Two Antoine datasets (water,
  ethanol).
- **No plotting anywhere**, though the PHREEQC `USER_GRAPH` parsing
  plumbing already exists unused (`kerotakis-phreeqc/src/lib.rs:94`).
- **`kero sweep` is a self-check harness** (1536 fixed cases, 8
  invariants), not a user-facing parameter study.
- **No uncertainty propagation**; uncertainty lives only in provenance
  prose.
- **Safety is a 4-species, 2-rule stub** despite being the L0 gate on
  every step.
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
CAP-11 stands (safety is still the 140-line stub); CAP-12 stands (no
titration verbs). The instrument lines of the old inventory are stale:
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
  `kero fit`), CAP-11 (safety still the 140-line stub), CAP-12 (no
  titrate/dilute verbs), CAP-13 (`vendor/inchi/` holds one README —
  a scaffold is not a vendored library).

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open — `vendor/inchi/` currently holds a README scaffold
      only; the task starts when sources with checksums land

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

- [ ] Status: open — **a standing avoid-list violation until done**

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

- [ ] Status: open

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

- [ ] Status: open

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
window and exact solute conservation. Remaining in this task: the
transport / chromatograph / calorimeter / react verbs. **Acceptance.** Each verb demonstrable in a replayed lesson;
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

## Declined — off-mission, recorded so nobody re-litigates silently

The workbench class serves professional geochemists managing field
data. We serve a learner at a bench. Therefore, deliberately not
planned: **water-sample database management and CSV/SQLite sample
import; Piper/Schoeller/Wilcox hydrochemical diagrams; PCA; 3-D and
treemap visualization; `INVERSE_MODELING`**. Each would be real work
serving a user we do not have; none teaches a concept our codex
covers. If the mission changes, change this paragraph first — PLAN.md
"What this will not do" governs.

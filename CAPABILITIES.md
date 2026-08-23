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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

- [ ] Status: open

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

**Acceptance.** All 74 registry InChIKeys recompute and match (or the
curation is fixed); native + wasm builds green in preflight/CI;
`cargo-deny` (CAP-14) passes with the vendored code declared.
**Size.** Medium (FFI + build plumbing). **Depends on:** nothing;
CAP-14 first is tidier.

---

## CAP-14 — Turn the licence policy into a CI lint

- [ ] Status: open — **land early; it guards every other task here**

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

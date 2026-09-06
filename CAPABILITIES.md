# Kerotakis — Capability extension tasks

> Finished work is not listed here. What landed, and what it taught us, is in
> [HISTORY.md](HISTORY.md). Task numbers are never renumbered and never reused.

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

## Where we stand today

19 bench operators; a REPL, `.lab` replay, MCP server and a wasm PWA
over the same JSON contract; PHREEQC wired for speciation,
amount-limited equilibrium phases, sealed gas phase, Hfo surfaces,
saturation indices and redox splits; CEA Gibbs minimisation for TP and
HP (adiabatic flame); own Nernst electrochemistry; 142+ registry
species, 4 embedded databases, 100+ codex reactions, 17+ lessons. Most
of the 2026-08-23 inventory's "notable holes" (no plotting, `kero
sweep` self-check only, no uncertainty propagation, 4-species safety
stub, no titrate/dilute/mix verb, `kerotakis-thermo` unreachable) are
now closed by the completed CAP tasks — see `## Completed CAP tasks`
below and `HISTORY.md`. Still open in the PHREEQC vocabulary:
`EXCHANGE` (CAP-10, remaining half); `KINETICS`/`RATES`,
`SOLID_SOLUTIONS`, `TRANSPORT`, `INVERSE_MODELING` (ROADMAP R1, beyond
CAP-10's slice — see `Already scheduled elsewhere` below).

**Post-merge note (2026-08-23).** The instrument lines of the original
inventory were stale even at the time: gas pressure/volume,
conductivity, spectrophotometer, calorimeter, chromatography and
qualitative analysis had landed (INST-003–008). New crates since the
inventory: `kerotakis-data`, `kerotakis-org` (`chematic` adopted off
the watch list), `kerotakis-sundials`, `kerotakis-registry-export`.
Cross-check every remaining task against the tree before starting it.

**Canonicality (2026-08-23).** This file is the canonical task list;
task numbers are stable identifiers and are never re-bound. A
completion checkbox requires the task's Acceptance evidence in the
marking commit; a checkbox without it is a claim, not a status. (A
same-day 96-line replacement mis-stated several tasks' status and was
restored from history — see `HISTORY.md` for the record if it matters
again.)

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
| Parameter fitting to data | — | ✓ | ✓ bounded one-parameter `kero fit` with residual chart/provenance | **CAP-9** |
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

CAP-26 (breadth programme) ──► BREADTH.md BRD-000…BRD-100
  measurement/data contracts first; substance/material packs before reaction
  families; engine adapters before UI; one final curiosity-corpus release gate
```

Library choices below were licence-verified 2026-08-23 and are listed in
PLAN.md's "Queued by the 2026-08-23 review"; all shipped dependencies must
clear the shipping bar there (MIT/Apache-2.0/BSD/Zlib/Unlicense/public
domain — no GPL family, LGPL included). CAP-14 turns that bar into CI
(done — see `HISTORY.md`).

OPT-7 (OPTIMIZATION.md) multiplies CAP-2/-4/-8: grid studies and
Monte Carlo are thousands of engine calls — done, see `HISTORY.md`
(OPT-7: 272 → 20 engine calls on the worst coupled case).

Note: essentially all CAP tasks above the matrix references (CAP-1
through CAP-9, CAP-11, CAP-12, CAP-14 through CAP-21) are now done —
see `## Completed CAP tasks` below. The matrix and dependency graph are
kept as originally written for parity-comparison context.

---

## CAP-10 — First slice of the unwired PHREEQC vocabulary: `EXCHANGE` and `MIX`

- [x] **MIX done 2026-08-24** (see `HISTORY.md`, CAP-10): `Operator::Mix`
      wired through the full CAP-1 pattern (parser, `apply()` with
      three-body adiabatic temperature balance, `mix()` trait method,
      native PHREEQC `MIX` input with fallback), `Event::Mixed` at
      lv1/lv2, hard-water softening lesson (`lessons/hard-water.lab`),
      6 core + 3 engine tests, preflight green.
- [ ] **EXCHANGE remains open.**

**Why.** Ion exchange (water softening) is a curriculum staple with
nowhere to live. `MIX` (now done) is what `decant`-based mixing used to
approximate by hand.

**Scope (remaining).** Wire `EXCHANGE` (input generation + read-back in
`aqueous.rs`, behind the same routing discipline as `SURFACE`); a
`resin`/exchanger species in the registry with provenance. One lesson:
hard water through an exchange column vessel, Ca²⁺ out, Na⁺ in,
hardness measured before and after. (Note: typed cation-exchange
ledgers plus 1-D transport landed upstream via AQ-007/011–014 before
this task was scoped — re-verify against the current tree whether that
already satisfies this line before doing the work twice.)

**Acceptance.** Softening lesson replays with element conservation
across the exchanger; differential-oracle spot checks against the
Reaktoro fixtures where expressible; preflight green. **Size.**
Medium. **Depends on:** OPT-6/OPT-7 coordination no longer applies —
both are done (see `HISTORY.md`).

---

## CAP-13 — Adopt the official InChI library (MIT since 1.07.1)

- [x] **Core adoption, tranche growth (23→102) and the chematic molfile
      spike are all done** (see `HISTORY.md`, CAP-13, and
      [`provenance/cap-13-chematic-molfile-spike.md`](provenance/cap-13-chematic-molfile-spike.md)
      for full evidence). Key facts kept: the identity bridge in
      `crates/kerotakis-org/src/native_inchi.rs` builds the InChI
      reference implementation's own 0D input structure directly —
      bypassing the V2000 molfile format entirely, which cannot express
      stereochemistry or isotopes regardless of chematic's version. Six
      species remain deferred pending BRD-010's external identity
      source (Cu(OH)2, MnO4-, Pb+2, Pb(NO3)2, methyl orange,
      bromothymol blue — both routes agree on these, so it is a
      curation disagreement, not a writer bug). `Al`'s registry key was
      corrected to `XAGFODPZIPBFFR-UHFFFAOYSA-N` (PubChem CID 5359268)
      after the old value was found certified only by two independently
      wrong computations agreeing (it was alumane, AlH3).

- [ ] **Dependency routing decision — open, needs the owner's call.**

      **(a) Upstream PR to chematic's V2000 writer** (valence field from
      explicit H count, `M  ISO` isotope tag) is prepared in-branch
      (`provenance/chematic/0001-mol2000-*.patch`) but not sent —
      patching a third party's project under our name needs sign-off.
      **(b) Vendoring/forking chematic — recommended against.**
      `deny.toml`'s `unknown-git = "deny"` blocks a
      `[patch.crates-io]` fork until licence policy is amended (an
      owner decision), and vendoring costs ~28 kLOC across chematic's
      dependency tree plus re-basing at every bump. **(c) Recommendation:**
      keep the pinned crates.io `chematic 0.18` unchanged and use the
      0D-structure route (already landed) — even a fixed V2000 writer
      cannot express E/Z geometry without 2D coordinates, so there is no
      version of this where forking chematic is the answer.

**Why.** The registry carries an InChIKey per species but nothing could
*compute or verify* one; identity was a hand-curated string. The L1
identity crosswalk (UniChem, keyed on Standard InChI) assumes exactly
this capability.

**Acceptance.** All registry InChIKeys recompute and match (or the
curation is fixed); native + wasm builds green in preflight/CI;
`cargo-deny` (CAP-14, done) passes with the vendored code declared.
**Size.** Medium. **Depends on:** the owner's call on (a)/(b)/(c) above;
nothing else blocks continued use of the already-landed 0D route.

---

## CAP-22 — Oracle coverage for the sprint's new surfaces

- [ ] Status: **in progress** (Fable and others). Landed increments
      (see `HISTORY.md`, CAP-22, for full detail): spectrophotometer
      literature anchor + Beer–Lambert linearity invariant (caught a
      real curation bug: permanganate ε(525) was 1.8× the literature
      value, 4363 → 2400); chromatography vs plate-theory + √N-scaling;
      calorimeter vs closed-form energy ledger; conductivity graduated
      2026-08-30 (Kohlrausch sum over solved speciation, KCl 1413 µS/cm
      calibration pinned, declares itself out-of-calibration above
      I≈0.1 mol/kgw); dry-solid conductance 2026-09-05
      (`electrical_resistivity` registry column, six pinned tests).
- [ ] **Remaining:** nuclide/photochem oracles wait until those
      subsystems are wired to anything; the `measure` verb's dispatch
      in `bench.rs` does not yet reach the dry-solid conductance path.

**Why.** The differential-oracle discipline that makes the PHREEQC core
trustworthy stops at that crate's border: instruments, apparatus,
photochemistry, polymers and nuclides carry only self-consistency unit
tests — the pattern that once let a divergent UNIFAC pass unnoticed.

**Scope.** Per subsystem, one independent second opinion (build-time
oracles, fixtures checked in). Decay chains vs analytic Bateman is
still open; CEA already has Cantera.

**Acceptance.** Every shipped instrument and apparatus number is either
oracle-checked or carries a written statement of why it cannot be.
**Size.** Medium, spread across subsystems. **Depends on:** nothing.

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

## CAP-23 — The single-solvent organic bench answers with chemistry

- [x] **Rungs 1, 2, 2b done** (see `HISTORY.md`, CAP-23): curated
      per-(solute, solvent) solubility + metal-inertness verdicts
      (`NonAqueousEquilibrator`, standing aside exactly where a verdict
      exists); permanganate–ethanol oxidation curated reaction
      (`4 KMnO4 + 3 C2H5OH → 4 MnO2↓ + 3 CH3COOH + 4 KOH + H2O`, MnO2
      deposits as a solid); silver metathesis
      (`AgNO3 + NaCl/KCl → AgCl↓ + NaNO3/KNO3`) gated on the dissolved
      fraction only. Rung 1 data growth (kero-basic): `ORGANIC_SOLUBILITY`
      grown 8 → 65 rows, `INERT_IN_SOLVENT` 6 → 24, across
      ethanol/hexane/propanone/ethyl_acetate, every row CRC Handbook
      97th ed. sourced. Reactive pairs (e.g. KMnO4/ethanol) excluded by
      rule, not silently tabled.

**Remaining rungs (open).** Rung 1 data growth continues toward every
registry solid × four solvents (kero-basic, in flight). **Rung 3
(open):** mixed water/organic solvents — route to PHREEQC above a
stated water mole-fraction threshold, with the co-solvent named as
unmodelled for activity; refuse below it with the dielectric reason.
Born-corrected mixed-solvent log K is **declined** until data worth
trusting exists. Bare dissolved ions in an organic phase (MnO4-, HCO3-
typed straight into ethanol) remain outside every rung and keep their
honest refusal.

**Depends on:** nothing new; Rung 3 wants a stated water mole-fraction
threshold decision.

## CAP-24 — Sixteen experiments, one open world

- [ ] Status: **scoped 2026-08-24** — the full audit, verdict matrix,
      quest design and ownership live in **EXPERIMENTS.md** (kept
      separate: it is a product map, not a single task). Summary: 4
      experiments runnable on today's chemistry (quest authoring only),
      7 within curated-data reach, 4 need one small physics model each,
      2 are declared boundaries (photovoltaics, acoustics). The
      load-bearing new piece is the quest engine: goal + free bench +
      event-driven nudges + any-order completion claims. **Part 2:** a
      ~40-problem aqueous virtual-lab corpus audited in EXPERIMENTS.md
      (capability classes only, source deliberately not named) —
      ~18 problem-classes NOW, the rest NEAR/HARDER, zero out of scope.
- [x] **EXP-2, EXP-13, EXP-14, EXP-43 landed 2026-08-24** (see
      `HISTORY.md`, CAP-24): thermal decomposition of NaHCO3
      (temperature-gated at 353 K); iodine decolorisation by vitamin C
      (titration endpoint persists past I2 consumption); enzymatic
      hydrolysis of starch by amylase (catalyst-gated, not consumed);
      clock kinetics (iodide–peroxide and iodate–bisulfite Landolt rate
      laws).

## CAP-25 — The senses of the bench (the visuals' honest sources)

- [x] **Slice 1 done 2026-08-24** (Fable; see `HISTORY.md`, CAP-25):
      smell (`smell`/`waft` verb, curated odour rows in `senses.rs`,
      an empty answer is spoken because "odourless" is data) and Burst
      (sealed glassware has a teaching overpressure limit, ~4 atm
      editorial constant; a sealed gas-maker fails as a Danger event,
      ledger exact through the bang).
- [ ] **Remaining:** the apparatus catalog (APPARATUS.md, ~70 items) —
      ~45 are drawable today over landed verbs; the rest map to
      EXP-8/18/30/31/32/33/34 plus three new small behaviours
      (reduced-pressure boiling, programmed dosing, accelerated
      settling). GUI workline: the catalog is the requirements list;
      visuals are our own.

## CAP-26 — Breadth: familiar matter, reusable reactions, honest reach

- [ ] Status: **scoped 2026-08-27.** The executable task graph lives in
      **[BREADTH.md](BREADTH.md)** as `BRD-000…BRD-100`; this CAP is its stable
      capability-level owner.

**Why.** Solver depth has outpaced reachable matter. The generated source pack
currently has 105 identities and several specialist modules are already built,
but a learner asks for vinegar, milk, paper, steel, soap, cola, soil, wax,
plastic and batteries. Those are often mixtures or objects, not pure species.
Adding another solver without supplying identities, material recipes,
parameters and bounded reaction families does little for that experience.

**Delivery order.** The programme is deliberately progressive:

1. `BRD-000…003` establish the curiosity corpus, typed coverage report,
   `MaterialRecipe` schema and quarantined import path.
2. `BRD-010…014` add reviewed PubChem/ChEBI identities, USDA-derived generic
   food compositions and at least 75 familiar material recipes.
3. `BRD-020…023` establish a curated reaction-family IR, choose Indigo or
   RDKit by compile/chemistry evidence, and ship the first organic family pack.
4. `BRD-030…042` evaluate direct feos and broader Cantera mechanisms, curate
   their separately licensed parameters, then wire only the routes that pass.
5. `BRD-050…062` add bounded Rhea/ChEBI biochemistry and COD/spglib crystal
   structure support.
6. `BRD-070…082` add physics-authority contracts, optional Rapier/Salva tactile
   behavior, a scientific viewer and optionally Ketcher authoring.
7. `BRD-090…100` add build-time validation and make the 500-prompt curiosity
   corpus a release gate.

**Acceptance.** `BRD-100` requires every curiosity prompt to end as computed,
curated, qualitative, explicit boundary, or an owned missing task; no silent
outcomes, unknown reachable safety rows, unattributed numbers, or host drift.
Coverage floors are set from the measured `BRD-001` baseline, not invented in
advance. **Out of scope:** claiming universality, bulk-importing unreviewed
databases, unrestricted synthesis prediction, or letting scene physics become
a second chemistry ledger.

---

## Completed CAP tasks

- **CAP-1** — wired `kerotakis-thermo` via the `distil` operator with full
  UNIFAC bubble point and a checked-in ethanol–water azeotrope acceptance
  test. Done 2026-08-23. See `HISTORY.md`.
- **CAP-2** — shipped `kero study`, a one-parameter sweep runner,
  rayon-parallel and byte-deterministic. Done 2026-08-24. (Fable) See
  `HISTORY.md`.
- **CAP-3** — defined the chart JSON contract and shipped a hand-rolled
  CLI/PWA SVG renderer plus `kero diagram txy`. Done 2026-08-23. (Fable)
  See `HISTORY.md`.
- **CAP-4** — computed Pourbaix pe–pH predominance diagrams for Fe and
  Cu with water-stability lines. Done 2026-08-23. (Fable) See
  `HISTORY.md`.
- **CAP-5** — added the named-relations layer (Arrhenius, Eyring,
  Nernst, Henderson-Hasselbalch, Debye-Hückel, van 't Hoff) with `kero
  calc`. Done. `f0af26a`. See `HISTORY.md`.
- **CAP-6** — added water/gas property correlations with provenance and
  `kero properties`. Done. `3e79ed2`. See `HISTORY.md`.
- **CAP-7** — replaced f64 Gaussian elimination with exact
  `Rational64` arithmetic for underdetermined balance families. Done.
  `94bbdb7`. See `HISTORY.md`.
- **CAP-8** — added Monte Carlo sampling to `kero study` with seeded
  percentiles and a chart band series. Done 2026-08-24. (Fable) See
  `HISTORY.md`.
- **CAP-9** — shipped `kero fit`, a bounded golden-section parameter fit
  recovering a rate constant within 3%. Done 2026-08-30. See
  `HISTORY.md`.
- **CAP-11** — expanded the reactive-hazard safety matrix from 4 to 142
  species with a totality test enforced in CI. Done 2026-08-23. See
  `HISTORY.md`.
- **CAP-12** — added `titrate` and `dilute` as first-class verbs with an
  auto-stepped titration curve; endpoint grammar extended to pe and
  colour-persists in EXP-39 (2026-08-30). Done 2026-08-23 (named-indicator
  spelling still unwired to `titrate`, tracked informally, not a
  numbered task). See `HISTORY.md`.
- **CAP-14** — turned the licence policy into a `cargo-deny` CI lint
  with a synthetic copyleft proof. Done 2026-08-23. See `HISTORY.md`.
- **CAP-15** — resourced Antoine constants to Stull 1947 and added
  methanol/propanone/acetic-acid data. Done 2026-08-23. `8e7e461`
  (kero-basic; audited by Fable). See `HISTORY.md`.
- **CAP-16** — added temperature-coupled γ to the dew-point and flash
  solvers, proven by bubble↔dew consistency. Done 2026-08-23. (Fable)
  See `HISTORY.md`.
- **CAP-17** — added Rayleigh batch distillation and an N-stage column
  with energy coupling through `hp_flash`. Done 2026-08-23. (Fable)
  See `HISTORY.md`.
- **CAP-18** — grew the UNIFAC table to 6 main groups / 30 interactions
  and fixed the OH↔CH2CO parameter swap. Done 2026-08-23. See
  `HISTORY.md`.
- **CAP-19** — built the Python-`thermo` differential oracle for
  UNIFAC γ and bubble points. Done 2026-08-23. (Fable) See `HISTORY.md`.
- **CAP-20** — wired extract/drain/chromatograph/react/transport verbs
  onto existing but ungrammared physics. Done 2026-08-24. (Fable;
  transport by Opus) See `HISTORY.md`.
- **CAP-21** — generated the species registry at build time from JSON,
  shrinking `species.rs` from 1,563 to 179 lines. Done 2026-08-23.
  (Fable) See `HISTORY.md`.

## Declined — off-mission, recorded so nobody re-litigates silently

The workbench class serves professional geochemists managing field
data. We serve a learner at a bench. Therefore, deliberately not
planned: **water-sample database management and CSV/SQLite sample
import; Piper/Schoeller/Wilcox hydrochemical diagrams; PCA; 3-D and
treemap visualization; `INVERSE_MODELING`**. Each would be real work
serving a user we do not have; none teaches a concept our codex
covers. If the mission changes, change this paragraph first — PLAN.md
"What this will not do" governs.

# Kerotakis — Development Plan

A virtual chemistry laboratory that computes real chemistry. Offline-first, no
Python at runtime, one simulation served at every register from nine-year-old to
expert.

Named for the sealed reflux vessel invented by Maria the Jewess in Alexandria,
1st–3rd century CE — the first named alchemist in recorded history. A sealed dome
with a sample suspended above a heated solvent, so the vapours act on it. She also
gave us the bain-marie; the airtight seal her apparatus needed is where
"hermetically sealed" comes from; and von Soxhlet's 1879 modernisation of it is
still in working labs today. The name describes the architecture: a sealed vessel
you put things into, and reactions happen.

---

## The thesis

There is no engine that takes arbitrary reagents and computes what happens. But
the problem decomposes into five sub-problems and **four of them are solved** by
mature tools with real thermodynamic databases. The product's job is to be
truthful about which is which.

| Sub-problem | Status | Engine |
|---|---|---|
| pH, precipitation, dissolution, titration, redox, buffers | solved exactly | PHREEQC |
| What boils when, what mixes, azeotropes, distillation | solved predictively | UNIFAC + CoolProp |
| Is this mixture dangerous | solved by database | CAMEO reactive groups |
| How fast, concentrations over time | solved | diffsol |
| Does an arbitrary organic reaction happen | **unsolved** | curate it |

PHREEQC is the highest-value engine here and almost no educational app uses it.
It solves speciation, mineral saturation, gas partitioning, redox and ionic
strength *simultaneously* from thermodynamic data. Mix silver nitrate and table
salt and it returns AgCl precipitate, how many moles, what ions remain, and the
saturation index — derived, not hardcoded.

---

## Architecture

### Don't build a simulator, build a vessel

Model a container in a state and the operations a person performs on it.

```
Vessel = { species: [(InChIKey, moles, phase)], T, P, V, solvent, container }

Operators: add · heat · cool · stir · wait(t) · filter · decant
           distil · evaporate · titrate · electrolyse · ignite
```

Every user action is one turn of a loop: operator → L0 safety pass → solver
router → new state + explanation. Between operators the vessel re-equilibrates.
This is what makes it a *lab* rather than a reaction calculator — the pedagogy
lives in the sequence, and the sequence is free once state is explicit.

### The solver stack

Numbering is real: **L0 runs first and can veto**, and each layer depends on what
the ones below resolve.

| Layer | Role | Engine | Licence |
|---|---|---|---|
| **L0** | Safety & reactivity screen — runs first, can veto | CAMEO / CRW 43×43 matrix, ~6,000 chemicals | US Gov work |
| **L1** | Species & property registry, canonical InChIKey identity | SQLite export + CoolProp + Indigo | MIT / Apache-2.0 |
| **L2** | Aqueous equilibrium — **the workhorse** | IPhreeqc | public domain |
| **L3** | Phase behaviour — boiling, miscibility, azeotropes | UNIFAC + cubic EOS + CoolProp | MIT |
| **L4** | Reaction — propose → filter → rank → verify | curated + Indigo templates | Apache-2.0 |
| **L4′** | QM enrichment — **build time only, never in the app** | xtb / CREST | LGPL-3.0 |
| **L5** | Kinetics & time evolution | diffsol | MIT |

### L4 is a cascade, not a choice

The stage that produced an answer is **shown to the user**.

1. **Propose** — curated library first. A few thousand hand-verified reactions
   with conditions, ΔH and observations covers all of school and most of
   undergraduate chemistry, and it is *correct*, which matters more than coverage
   in education. Indigo's `indigoReactionProductEnumerate` / `indigoTransform`
   generalise templates across homologues.
2. **Filter** — our own SMARTS incompatibility rules, plus the L0 pass. Note that
   RDKit's shipped `FilterCatalog` (PAINS, BRENK, NIH, ChEMBL) is a set of
   *medicinal-chemistry alerts*, not reaction-feasibility rules. The rules are ours.
3. **Rank** — surface confidence, never present a prediction as a fact.
4. **Verify** — L4′, offline.

### Why QM is build-time only

GFN2-xTB gives real ΔG_rxn for structures handed to it. It is a *verifier*, not a
generator — something else must propose the mechanism first. Barriers are harder
still: CREST samples conformers, it does not find transition states, and a real
ΔG‡ needs a supervised saddle-point search plus frequency and IRC confirmation.
That pipeline fails often and is normally human-supervised, so it cannot sit
behind a user tapping "mix".

Batching it on the build machine also dissolves the LGPL-3.0 relinking conflict
with App Store distribution, because the binary never ships. The results ship as
numbers in the curated library.

---

## Why Rust, and why offline works

### The deciding fact

`dart:ffi` **cannot be imported when compiling to Wasm**, and there is no unified
API for driving one native library through FFI on mobile and JS interop on web.
A Flutter app targeting web must therefore write every native integration twice.
With PHREEQC, Indigo and CoolProp in the stack that doubles the hardest code in
the project. Rust compiles one source to `wasm32` and to all five native targets.

### PHREEQC runs on a phone

The layer that could have killed offline turns out to be the most portable thing
in the stack. IPhreeqc's C API has a complete string-in / value-out path that
**never touches the filesystem**:

```c
LoadDatabaseString(id, db)          // thermodynamic DB from a string
RunString(id, input)                // input from a string
SetSelectedOutputStringOn(id, 1)    // results to memory
GetSelectedOutputValue(id, r, c, &v)
SetOutputFileOn(id, 0)              // all file I/O off
```

It builds with **CMake**, so Android NDK, iOS and Emscripten all get proper
toolchain files. And the databases are small enough to compile into the binary:

| Database | Size | Covers |
|---|---|---|
| `phreeqc.dat` | 52 KB | Core aqueous set — most teaching chemistry |
| `wateq4f.dat` | 88 KB | Extended natural-water species |
| `minteq.v4.dat` | 316 KB | Metals, complexation, sorption |
| **all three** | **451 KB** (77 KB gzipped) | an `include_str!`, not an asset pipeline |

### Workspace layout

```
kerotakis/
├── crates/
│   ├── kerotakis-core/       vessel state machine, operators, solver router
│   ├── kerotakis-phreeqc/    IPhreeqc FFI + embedded databases
│   ├── kerotakis-indigo/     Indigo FFI — structures, templates, products
│   ├── kerotakis-thermo/     UNIFAC, cubic EOS, flash; CoolProp via coolprop-sys
│   ├── kerotakis-kinetics/   diffsol wrapper
│   ├── kerotakis-data/       curated library + CAMEO matrix, embedded SQLite
│   └── kerotakis-wasm/       wasm-bindgen surface for web
├── tools/                    build-time data pipelines (xtb batches, DB exports)
└── app/                      UI — see the open decision below
```

`kerotakis-core` is the invariant. It compiles to `wasm32-unknown-unknown` and to
aarch64-apple-ios, aarch64-linux-android, x86_64-pc-windows-msvc and
aarch64-apple-darwin from one source.

### Dropped by going offline

- **Cantera** — SCons-only build, no WASM. diffsol plus our own mechanism data
  covers the educational need.
- **Molecular Transformer on-device** — 50–100 MB quantised; tolerable as an
  optional mobile download, brutal for web. It is also trained on USPTO patent
  reactions, so it is weakest exactly where our users are: ask it about vinegar
  and baking soda and it is far off-distribution with no signal that it is.
- **Python entirely** — `thermo`'s role moves into `kerotakis-thermo`;
  `chemicals` becomes a build-time export to SQLite.

---

## Nine to expert, one simulation

Never dumb down the model, only the view. One PHREEQC result, rendered at
whatever register the reader is in. The child and the postdoc see the same numbers.

| Register | Output |
|---|---|
| Age 9 | "It went cloudy! A white solid appeared — that's a *precipitate*." |
| Age 15 | `AgNO₃ + NaCl → AgCl↓ + NaNO₃` · 0.010 mol · Ksp = 1.77 × 10⁻¹⁰ |
| Expert | SI(AgCl) = +2.41 · I = 0.021 m · γ(Ag⁺) = 0.857 · full selected-output |

Registers are a presentation concern and live entirely in the UI. The solver has
no idea who is asking.

### The alchemical layer earns its keep

The twelve classical operations map almost directly onto our operator list, so
the naming system *is* the difficulty ladder rather than decoration:

| Child | Modern | The Work |
|---|---|---|
| Heat it up | Thermal decomposition | Calcination |
| Let it settle | Precipitation | Coagulation |
| Boil it off | Fractional distillation | Distillation |

The four stages of the magnum opus — nigredo, albedo, citrinitas, rubedo — are a
ready-made progression system.

---

## What this will not do

Worth writing down before starting, because each is where an ambitious version
quietly fails.

- **Predict arbitrary organic reactions.** Genuinely unsolved. Curate, and be
  visibly honest where we are predicting rather than knowing.
- **Mechanisms and transition states.** Quantum chemistry, build-time at best.
- **Extremes.** Plasmas, exotic organometallics, solid-state, high pressure.
- **Biochemistry.** A different stack; a later module, not an extension.

A general-purpose engine that computes any reaction from first principles is also
a synthesis oracle for things we do not want it computing. Curated-first gives us
an explicit, auditable boundary — a product-safety property, much easier to
defend than a filter bolted onto a general predictor.

---

## Open TODOs

### P0 — Feasibility spike

The single highest-information task. Everything else is downstream of it.

- [ ] Build IPhreeqc for `wasm32-unknown-emscripten` and one mobile target
- [ ] Drive it through `LoadDatabaseString` / `RunString` with no filesystem
- [ ] Embed `phreeqc.dat` via `include_str!`, confirm binary size
- [ ] One end-to-end case: AgNO₃ + NaCl → saturation index out
- [ ] **Gate:** if PHREEQC cross-compiles clean to both, the offline premise holds

### P1 — Vessel state machine + L0 safety

- [ ] `Vessel`, operators, re-equilibration between steps
- [ ] Extract the CAMEO reactive-group matrix into embedded SQLite; confirm
      redistribution terms for the dataset specifically
- [ ] Wire L0 as a veto that runs before any chemistry
- [ ] Do this *first*. Retrofitting a safety layer into a shipped app is where
      products get hurt.

### P2 — PHREEQC, shippable on its own

- [ ] `kerotakis-phreeqc` FFI surface (~15 functions matter)
- [ ] Acid–base, precipitation, titration curves, solubility, buffers
- [ ] Content-addressed result cache — same species set, T and P is the same answer
- [ ] This alone is a strong product

### P3 — Phase behaviour

- [ ] UNIFAC against published group-interaction tables. **Note:** the `unifac`
      crate reports a non-standard licence string on crates.io — verify or
      reimplement. UNIFAC is a few hundred lines; reimplementing beats
      discovering the problem at legal review.
- [ ] Evaluate `vle-thermo` (MIT) and `KiThe` (MIT) before writing our own EOS
- [ ] `coolprop-sys` (MIT) for pure components
- [ ] Rachford-Rice flash; target the ethanol–water azeotrope at 95.6% as the
      acceptance test — it is a genuine teaching moment most simulators miss

### P4 — Curated reaction library

The slow, expensive, valuable part. This is the moat: nobody can scrape a
well-curated pedagogical reaction set with observations attached.

- [ ] Schema: balanced equation, conditions, ΔH, observations (colour, gas,
      precipitate, heat), provenance, register-specific copy
- [ ] Indigo template application over homologues
- [ ] Our own SMARTS incompatibility rules
- [ ] Budget this as a chemistry-editorial hire, not an engineering task

### P5 — Kinetics

- [ ] diffsol integration. Reaction networks are **stiff** — explicit
      Runge–Kutta will not integrate them, so the solver choice is not free.
      diffsol's default features are pure Rust (nalgebra/faer); the LLVM and
      Cranelift JIT paths are opt-in and must stay off for iOS and wasm.

### P6 — Build-time QM enrichment

- [ ] `tools/` pipeline batching xtb over the curated library
- [ ] Supervised TS searches only where a barrier genuinely matters
- [ ] Output is data; no xtb binary or library ships

---

## Open decisions

### UI framework

`kerotakis-core` is the invariant either way. This is a second-order choice.

| | Tauri + React | Flutter |
|---|---|---|
| Web | same Rust → wasm | separate JS-interop path |
| Mobile UI maturity | Tauri v2 mobile is newer | battle-tested |
| Core integration | native | `dart:ffi` on 5 platforms |

If web is a real target → Tauri. If mobile UI polish outranks web → Flutter,
accepting a thinner web story or shipping web later as a small app over the same
wasm build.

### Data provenance

The two traps are both about data, not code.

- [ ] `chemicals` is MIT *code* aggregating property data from CRC, NIST WebBook,
      Yaws, Common Chemistry and Wikidata, and does not exhaustively enumerate
      redistribution terms per source. Conservative fallback: Wikidata (CC0) plus
      properly cited primary sources plus our own curation.
- [ ] The Open Reaction Database is **CC-BY-SA 4.0** — ShareAlike could reach our
      curated library. Keep it out unless a licence review says otherwise.
- [ ] Both want a real legal read before launch.

---

## Name & trademark status

Cleared 2026-08-18 via TMview (aggregates USPTO, EUIPO and 70+ national registries).

- **One** KEROTAKIS mark worldwide: Argentina only, Nice class 34
  (tobacco/smokers' articles), owner HELMFELT, reg. 3470789, expires 2033-11-21.
  No conflict with software.
- **Zero** hits at USPTO, zero at EUIPO, zero in classes 9 / 41 / 42 worldwide.
- Zero hits for phonetic variants (`cerotakis`, `kerotaki`, `kerotakys`,
  `kerotaxis`, `keratakis`, stem `kerotak`) in those classes.
- All of `.com` `.app` `.dev` `.io` `.org` `.net` were unregistered; crates.io,
  npm and PyPI all free.

Outstanding:

- [ ] Register the domains — the only item here with a race condition
- [ ] Claim `kerotakis` on crates.io, npm, PyPI
- [ ] File classes 9 / 41 / 42 through an attorney nearer launch, once the
      goods-and-services wording is settled. What was done is a screen, not a
      clearance opinion.

---

## Licence

AGPL-3.0-or-later, with an App Store / Google Play additional permission for
binaries published by the copyright holder. See `LICENSE` and `NOTICE`.

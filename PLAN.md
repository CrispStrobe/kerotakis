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

> Every licence, build flag and API claim in this document was verified against
> upstream source, package metadata or primary legal text on **2026-08-18**.
> Items marked **wasm ✓** were compile-tested locally against
> `wasm32-unknown-unknown` on that date, not read off a README.

---

## The thesis

There is no engine that takes arbitrary reagents and computes what happens. But
the problem decomposes into six sub-problems and **five of them are solved** by
mature tools with real thermodynamic databases. The product's job is to be
truthful about which is which.

| Sub-problem | Status | Engine |
|---|---|---|
| pH, precipitation, dissolution, titration, redox, buffers | solved exactly | PHREEQC |
| Heating & igniting solids and gases — decomposition, combustion, flame T | solved exactly | Gibbs minimisation over NASA CEA data |
| What boils when, what mixes, azeotropes, distillation | solved predictively | feos + UNIFAC + cubic EOS |
| Is this mixture dangerous | solved by database | reactive-group matrix (reimplemented from NOAA's published methodology) |
| How fast, concentrations over time | solved | diffsol |
| Does an arbitrary organic reaction happen | **unsolved** | curate it |

PHREEQC is the highest-value engine here and almost no educational app uses it.
It solves speciation, mineral saturation, gas partitioning, redox and ionic
strength *simultaneously* from thermodynamic data. Mix silver nitrate and table
salt and it returns AgCl precipitate, how many moles, what ions remain, and the
saturation index — derived, not hardcoded.

The second row is new, and it matters: heating and burning things is half of
school chemistry (CaCO₃ → CaO + CO₂, decomposing KMnO₄, dehydrating copper
sulfate, magnesium in a flame) and neither PHREEQC (aqueous) nor VLE (liquids)
touches it. NASA open-sourced CEA under **Apache-2.0 in 2026, including its
thermodynamic database** (`data/thermo.inp`) — so a small Gibbs-energy minimiser
over NASA polynomials with pure condensed phases turns "heat it" and "ignite it"
into computed chemistry, adiabatic flame temperature included.

---

## Architecture

### Don't build a simulator, build a bench

Model containers in states and the operations a person performs on and between
them. `filter` and `decant` produce a *second* container, so the unit of state is
a bench, not a vessel:

```
Bench  = { vessels: [Vessel], log: [Op] }
Vessel = { species: [(InChIKey, moles, phase)], T, P, V, solvent, container }

Mutating ops:   add · heat · cool · stir · wait(t) · filter → v2 · decant → v2
                distil → v2 · evaporate · titrate · electrolyse · ignite
Measuring ops:  pH probe · thermometer · balance · conductivity meter ·
                litmus / indicator paper · spectrophotometer
```

Every user action is one turn of a loop: operator → L0 safety pass → solver
router → new state + explanation. Between operators the bench re-equilibrates.
This is what makes it a *lab* rather than a reaction calculator — the pedagogy
lives in the sequence, and the sequence is free once state is explicit.

**Measuring ops are first-class and read-only.** They cost almost nothing (they
read existing solver output), they give register-appropriate precision naturally
— litmus for the nine-year-old, the same pH to three decimals for the expert —
and "measure before and after" *is* the scientific method being taught.

**The operator log is the save file.** Bench state is tiny; persisting the full
operator sequence gives undo, replay, sharing experiments as scripts, and golden
tests for free. It is also the substrate for the lesson layer (below).

**The vessel has an energy balance.** PHREEQC is isothermal — you *tell* it T.
So the loop does its own enthalpy bookkeeping: reaction ΔH (curated data or
formation enthalpies) + mixture heat capacity → new T → re-equilibrate at the
new T, iterated to convergence. Each vessel is adiabatic, isothermal or
thermostatted by mode. Without this, exothermic mixing cannot warm the beaker,
and "it got hot!" is one of the most important observables at every register.

**Solver failure is a first-class result.** PHREEQC convergence failures on odd
inputs are routine, not exceptional. The router surfaces an honest "couldn't
compute this" state rather than a wrong answer or a crash — the same honesty
rule that governs L4.

### The solver stack

Numbering is real: **L0 runs first and can veto**, and each layer depends on what
the ones below resolve.

| Layer | Role | Engine | Licence |
|---|---|---|---|
| **L0** | Safety & reactivity screen — runs first, can veto | Our reimplementation of the NOAA 43×43 reactive-group matrix (see below) | ours, from public-domain methodology |
| **L1** | Species & property registry, canonical InChIKey identity | SQLite/static data + Indigo (bundles the InChI plugin) | Apache-2.0 / MIT |
| **L2** | Aqueous equilibrium — **the workhorse** | IPhreeqc + phreeqc.dat, wateq4f.dat, minteq.v4.dat, **pitzer.dat** | USGS, public domain |
| **L2g** | Gas + condensed-phase equilibrium — heat, ignite, decompose, burn | Gibbs minimiser over NASA CEA data (adopt/extend `cea-rs`, or write it — see adopt-and-extend policy) | Apache-2.0 data |
| **L3** | Phase behaviour — boiling, miscibility, azeotropes | `feos` (SAFT family, flash) + own UNIFAC + `vle-thermo` (cubics, NRTL/Wilson) + `seuif97` (water) | MIT / Apache-2.0 |
| **L3e** | Electrolysis | Faraday's law + standard-potential ordering, own module; PHREEQC supplies speciation and Eh | ours |
| **L4** | Reaction — propose → filter → rank → verify | curated + Indigo templates | Apache-2.0 |
| **L4′** | QM enrichment — **build time only, never in the app** | xtb / CREST | LGPL-3.0, never shipped |
| **L5** | Kinetics & time evolution | diffsol | MIT |
| **L6** | Appearance — colour, cloudiness, flames | curated colour data + Beer–Lambert over ε(λ) + CIE colour math via `palette` | ours / MIT |

Notes per layer:

- **L0** — the CAMEO/CRW4 *database* is not redistributable: its terms prohibit
  duplicating contributed data, explicitly naming CAS registry numbers and
  formulas (ACS property), NFPA ratings, AEGLs and ERPGs. But the reactive-group
  **methodology** — the 43×43 compatibility chart and the group-classification
  logic — is published in open-access NOAA papers, and NOAA-authored technical
  prose is US-government public domain. So: our own SMARTS rules (run by Indigo)
  assign compounds to the 43 groups; we ship the matrix and our assignments,
  never their database. More work than an extract, but it is the only defensible
  version, and the assignment rules become an asset we own.
  Primary sources: NOAA Institutional Repository CRW4 paper
  (repository.library.noaa.gov/view/noaa/61941); Gorman et al., *Process Safety
  Progress* 2014.
- **L1** — InChI/InChIKey has exactly one implementation, the IUPAC C library,
  **relicensed MIT with v1.07** (plain C, current 1.07.5). We get it through
  Indigo's bundled InChI plugin via the same FFI; a standalone `kerotakis-inchi`
  binding is the fallback (wasm precedent: cheminfo's `inchi-js` npm package).
- **L2** — `pitzer.dat` is only **37 KB** and public-domain like the rest; it
  unlocks brines and high ionic strength (seawater evaporation is a beautiful
  teaching sequence). Embed it. **Do not embed `sit.dat`**: it is generated from
  ANDRA's ThermoChimie database — non-USGS provenance; revisit only after a
  terms check. Upstream is now the actively maintained `phreeqc-dev` GitHub org
  (CMake, C++14).
- **L2g** — a Gibbs minimiser over NASA polynomials is a well-understood,
  few-hundred-line solver. `cea-rs` (MIT OR Apache-2.0, wasm ✓) appeared on
  crates.io in Aug 2026 — embryonic, but proof the port is tractable and a
  candidate to adopt and extend rather than start from zero.
- **L3** — see the crate table below for what each piece covers and why
  CoolProp was demoted.
- **L6** — most of the age-9 register is *observations*, and they need a
  computation path: species/precipitate colours and flame colours are curated
  data; indicator colours follow from indicator pKa via PHREEQC; solution colour
  comes from Beer–Lambert over per-species ε(λ). `palette` (wasm-fine) has no
  spectral module, so we integrate CIE 1931 colour-matching functions against
  the transmittance spectrum ourselves — small, well-documented math — and hand
  the XYZ result to `palette` for Lab/sRGB.

### L4 is a cascade, not a choice

The stage that produced an answer is **shown to the user**.

1. **Propose** — curated library first. A few thousand hand-verified reactions
   with conditions, ΔH and observations covers all of school and most of
   undergraduate chemistry, and it is *correct*, which matters more than coverage
   in education. Indigo's `indigoReactionProductEnumerate` / `indigoTransform`
   (both verified present in the current flat C API) generalise templates across
   homologues.
2. **Filter** — our own SMARTS incompatibility rules, plus the L0 pass. RDKit's
   shipped `FilterCatalog` (PAINS, BRENK, NIH, ChEMBL) is a set of
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

**Bonus from the same pipeline:** xtb computes vibrational frequencies, so the
build machine can emit **synthetic IR spectra** for every curated compound —
licence-clean spectra (computed, not scraped from NIST) that power the
spectrophotometer instrument and an "identify the unknown" lesson mechanic no
competitor has.

### The lesson layer is the product

The bench is the engine; guided sequences are what people buy. Lessons are
declarative data driving the same operator loop: a scenario file names starting
vessels, allowed operators, goals ("identify the unknown solution", "make 100 mL
of pH 7 buffer") and register-specific narration hooks. The alchemical
progression — nigredo, albedo, citrinitas, rubedo — is the difficulty ladder.
Because lessons are data over the operator log, they are also replayable tests.

---

## The crate stack, verified

| Crate | Licence | Status (2026-08) | Role | wasm |
|---|---|---|---|---|
| `feos` | MIT OR Apache-2.0 | active, v0.10.1 | L3 core: PC-SAFT, gc-PC-SAFT, multiparameter Helmholtz, full flash machinery. **No UNIFAC** | ✓ |
| `num-dual` | MIT OR Apache-2.0 | active | AD backbone of feos; exact fugacity/enthalpy derivatives | ✓ |
| `vle-thermo` | MIT | active, very young (May 2026) | 22+ cubic EOS, NRTL/Wilson/van Laar, Rachford–Rice flash, phase envelopes | ✓ |
| `seuif97` | MIT | active | IAPWS-IF97 water/steam — most of our solvent story | ✓ |
| `diffsol` | MIT | active | L5 stiff ODE/DAE (BDF), pure-Rust nalgebra/faer backends; keep `diffsl` JIT **off** for iOS/wasm | ✓ (JIT off) |
| `cea-rs` | MIT OR Apache-2.0 | embryonic (Aug 2026) | L2g seed — adopt/extend or rewrite over the same Apache-2.0 CEA data | ✓ |
| `nalgebra` | Apache-2.0 | active | Stoichiometric null-space balancing, linear algebra | ✓ |
| `petgraph` | MIT OR Apache-2.0 | active | Reaction-network DAGs, cascade routing | ✓ |
| `uom` | Apache-2.0 OR MIT | alive, ~1 release/yr | Compile-time units at `kerotakis-core` API boundaries — kills molarity-vs-molality bugs | ✓ |
| `palette` | MIT OR Apache-2.0 | active | L6 colour math (XYZ/Lab/sRGB); we supply the spectral→XYZ integration | ✓ |
| `rusqlite` ≥ 0.38 | MIT | active | Curated library + registry. wasm works via `sqlite-wasm-rs` (also Diesel's official wasm backend) | ✓ |
| `postcard`/`rkyv` | MIT etc. | active | Alternative for read-only bundled data: `include_bytes!`, zero-copy, smaller than SQLite | ✓ |

**Evaluated and set aside** (with the reason, so we don't re-litigate):

- `purr`, `gamma`, `chemcore` — frozen at 2021 proof-of-concept state; no SMARTS
  matching, no canonicalisation, no InChI. Their author, Rich Apodaca, died in
  2024; the successor `balsa` is also dormant. Not a base to build on.
- `sundials-sys` — dormant ~20 months; no evidence anyone has ever built
  SUNDIALS for wasm or iOS. diffsol already covers L5 in pure Rust.
- `coolprop-sys` — actively maintained but bundles **prebuilt desktop dylibs
  only**: no wasm, no iOS/Android. CoolProp itself has an official Emscripten/JS
  build, so it remains available as an optional *desktop/web extra*, not core.
  `feos` multiparameter + `seuif97` cover the pure-fluid need.
- `KiThe` — hard, non-optional `reqwest`/`tokio` deps (its NIST WebBook scraper)
  fail on wasm ✗, and scraping WebBook is an SRD-licensing problem anyway (see
  data provenance). Fork candidate only if its NASA-polynomial equilibrium code
  outperforms our L2g — reassess once L2g exists.
- `ort` — iOS/Android static binaries exist, but the browser path is `ort-web`
  bridging Microsoft's onnxruntime-web: two incompatible wasm contexts with
  tensor sync. If the ML tier ever ships, `candle` (proven wasm demos, quantised
  GGUF) or `burn` (working in-browser WebGPU) fit this stack better. Deferred
  with the ML tier itself.
- `rdkit-rs` — dormant ~20 months, needs native C++ RDKit, no wasm story; RDKit
  MinimalLib's long-time maintainer stepped down 2026-04. Indigo is primary.

**Watch list** (young or needs-work, but filling real gaps):

- `chematic` — pure-Rust cheminformatics with real SMARTS + VF2 substructure
  matching, canonical SMILES, 2D depiction, first-class wasm npm build. Three
  months old, bus-factor 1, self-reported RDKit parity. If it matures it
  replaces the Indigo FFI for everything except InChI; re-evaluate quarterly.
- `teqp` (NIST) — **public-domain** multiparameter/GERG/SAFT EOS in C++. No
  official wasm artifact, but CMake + a changelog hint at JS support make an
  Emscripten side-module feasible. The option if L3 ever needs
  reference-quality multiparameter mixtures beyond feos.
- `GEMS3K` (PSI) — LGPL-3.0 Gibbs minimiser, markedly better than PHREEQC for
  non-ideal solid solutions and melts, designed for embedding. LGPL is
  manageable for desktop/server builds (dynamic linking) but awkward for a
  static wasm bundle and App Store. The option if L2 ever hits the
  solid-solution wall.
- the alpha `phreeqc` npm package (Emscripten, MIT + USGS notice, Jan 2026) —
  single-maintainer alpha; we do not depend on it, but it is the existence proof
  for our own P0 build, and worth reading before writing ours.

### Adopt-and-extend policy

A tool is not disqualified because it needs work from us — forking, wasm
compiles, FFI bindings, feature-gating out bad deps — **if it fills a gap no
equally good, licence-compatible tool fills**. `cea-rs`, `chematic`, `teqp`, a
KiThe fork and the `unifac` crate's algorithm are all in that category. A tool
*is* disqualified by: incompatible licence on code we'd ship (GPL-only, NC),
non-redistributable embedded data, or a dead upstream *plus* an equally good
maintained alternative. When we extend, we upstream patches where the project is
alive and fork visibly where it is not.

### The UNIFAC question, precisely

No clean-licensed, maintained Rust UNIFAC exists. The `unifac` crate (frozen
2021, wasm ✓) is MIT **code** with a warning clause about its embedded
parameters — and that problem attaches to *any* implementation, including one we
write: the maintained UNIFAC Consortium tables are proprietary; the original
open-literature tables (Fredenslund, Gmehling et al., 1970s–90s journals) are
usable. So: reimplement the ~300-line algorithm (or fork the crate), source
parameters from the original publications, and record provenance per parameter.
Budget it as data curation, not coding. Acceptance test unchanged: the
ethanol–water azeotrope at 95.6% — a genuine teaching moment most simulators miss.

---

## Why Rust, and why offline works

### The deciding fact

`dart:ffi` **cannot be imported when compiling to Wasm**, and there is no unified
API for driving one native library through FFI on mobile and JS interop on web.
A Flutter app targeting web must therefore write every native integration twice.
With PHREEQC and Indigo in the stack that doubles the hardest code in the
project. Rust compiles one source to `wasm32` and to all five native targets.

### The two-track wasm/FFI strategy

**Track A — pure Rust → `wasm32-unknown-unknown` + wasm-bindgen.**
`kerotakis-core`, feos, vle-thermo, seuif97, diffsol, nalgebra, petgraph, uom,
palette, sqlite-wasm-rs. All compile-verified or wasm-proven. No experiments
needed; the same source serves all five native targets.

**Track B — the C/C++ engines (IPhreeqc, Indigo+InChI) → Emscripten side
modules with a thin JS bridge on web; ordinary cargo + CMake FFI on native.**
Three existence proofs de-risk the web side: the alpha `phreeqc` npm package,
EPAM's official `indigo-ketcher` wasm, and CoolProp's official Emscripten build.
Because IPhreeqc's API is string-in/value-out, the JS bridge between the wasm
modules is trivial. On native, a shipping third-party Android PHREEQC app
already proves cross-compilation. One caveat: `indigo-ketcher` exports a
*subset* of the Indigo API and may not include `indigoReactionProductEnumerate`
/ `indigoTransform` — plan on building Indigo's wasm target ourselves with those
symbols exported (the Emscripten toolchain is in their repo).

**Stretch experiment — single-module linking via wasi-sdk.** Since wasi-sdk 33,
C++ exceptions are supported opt-in (`-fwasm-exceptions`, dual libc++ builds),
and Rust's `wasm32-wasip1` officially supports linking wasi-sdk-built C/C++
static libraries: one wasm module, no JS bridge, no Emscripten. Nobody has done
it with PHREEQC (which uses exceptions internally), so it is time-boxed and
never on the critical path — Track B is already proven. Either way, exceptions
stay caught on the C++ side of the boundary; IPhreeqc's API already does.

### PHREEQC runs on a phone

IPhreeqc's C API has a complete string-in / value-out path that **never touches
the filesystem**:

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
| `pitzer.dat` | 37 KB | Brines, high ionic strength |
| **all four** | **~490 KB** (~80 KB gzipped) | an `include_str!`, not an asset pipeline |

### Workspace layout

```
kerotakis/
├── crates/
│   ├── kerotakis-core/       bench + vessel state machine, operators, energy
│   │                         balance, solver router, measurement ops
│   ├── kerotakis-phreeqc/    IPhreeqc FFI + embedded databases (L2)
│   ├── kerotakis-cea/        Gibbs minimiser over NASA CEA data (L2g)
│   ├── kerotakis-indigo/     Indigo FFI — structures, InChI, templates (L1/L4)
│   ├── kerotakis-thermo/     feos + own UNIFAC + vle-thermo + seuif97 (L3)
│   ├── kerotakis-electro/    electrolysis — Faraday + potential ordering (L3e)
│   ├── kerotakis-kinetics/   diffsol wrapper (L5)
│   ├── kerotakis-appearance/ colour: curated data + Beer–Lambert + CIE (L6)
│   ├── kerotakis-safety/     reimplemented reactive-group matrix + rules (L0)
│   ├── kerotakis-data/       curated library + registry, embedded
│   └── kerotakis-wasm/       wasm-bindgen surface for web (Track A)
├── tools/                    build-time pipelines: xtb batches, IR spectra,
│                             PubChem/Wikidata/CEA data exports, Indigo wasm build
├── lessons/                  declarative scenario files
└── app/                      UI — see the open decision below
```

`kerotakis-core` is the invariant. It compiles to `wasm32-unknown-unknown` and to
aarch64-apple-ios, aarch64-linux-android, x86_64-pc-windows-msvc and
aarch64-apple-darwin from one source.

### Testing is part of the architecture

- **Conservation invariants** — property tests asserting mass and charge balance
  across *every* operator, on random benches. This catches whole classes of bugs
  no example test finds, and it is a moat: competitors with lookup tables cannot
  even state the invariant.
- **Golden tests** — textbook values: acetic-acid titration curve, AgCl Ksp,
  ethanol–water azeotrope, CaCO₃ decomposition temperature, adiabatic flame T.
- **Fuzzing PHREEQC** — random vessel states in, no crash and honest failure out.
- **Lessons as tests** — every scenario file replays in CI via the operator log.

---

## Data provenance, verified

The traps are all about data, not code. Checked against primary sources
2026-08-18; the conclusions changed the plan.

| Source | Terms | Verdict for us |
|---|---|---|
| PubChem (NCBI bulk) | No NCBI restrictions, commercial OK; per-annotation source attribution expected | **Primary property + GHS source.** Keep attribution per record |
| Wikidata | CC0 | Clean supplement; coverage is thin (≈2k boiling points, ≈310 pKa) — cannot carry the load |
| NASA CEA (`github.com/nasa/cea`) | **Apache-2.0** incl. `data/thermo.inp` | **Primary thermochemistry source** for L2g and formation enthalpies |
| CLP Annex VI via EUR-Lex | EU legislation, reuse with acknowledgment | Harmonised GHS/CLP hazard classes — take from EUR-Lex, not ECHA dumps |
| PHREEQC databases | USGS User Rights Notice (public-domain-like, attribution) | Embed (except `sit.dat` — ThermoChimie provenance, needs a terms check) |
| CAS Common Chemistry | **CC BY-NC 4.0** | Unusable commercially. Also: never present CAS RNs as licensed-from-CAS data; identifiers come from PubChem/Wikidata |
| NIST WebBook / JANAF-online | **NIST SRD — copyrighted**, permission required | Do not scrape or redistribute. (The 1971 NSRDS-NBS 37 JANAF tables are public domain but dated) |
| CAMEO / CRW4 database | Contributed fields explicitly non-duplicable (CAS RNs, NFPA, AEGL, ERPG) | Never ship the database; reimplement the published methodology (L0 note above) |
| ECHA C&L exports | IP-encumbered (CAS data named) | Avoid; use EUR-Lex / PubChem routes |
| Burcat (Third Millennium) | Free non-commercial only; commercial inclusion forbidden without written permission | Skip, or write for permission if CEA coverage falls short |
| Open Reaction Database | **CC-BY-SA 4.0** on data; ShareAlike propagates to merged datasets under the mainstream reading | Keep out of the curated library, or accept BY-SA on the whole dataset — decide before the first record is ingested |
| `chemicals` (Python) | MIT code aggregating CRC/NIST/Yaws/Common Chemistry data | **Dropped as a build-time source** — it launders the SRD and NC problems above into our binary |
| UNIFAC parameters | Consortium tables proprietary; original journal tables usable | Source from the original publications, provenance per parameter |

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
no idea who is asking. Register copy is generated by deterministic templates
over solver output ("SI > 0 and new solid phase → 'went cloudy'"), never by a
language model — offline, reproducible, trustworthy.

### The alchemical layer earns its keep

The twelve classical operations map almost directly onto our operator list, so
the naming system *is* the difficulty ladder rather than decoration:

| Child | Modern | The Work |
|---|---|---|
| Heat it up | Thermal decomposition | Calcination |
| Let it settle | Precipitation | Coagulation |
| Boil it off | Fractional distillation | Distillation |

The four stages of the magnum opus — nigredo, albedo, citrinitas, rubedo — are a
ready-made progression system, realised as lesson-file tiers.

---

## What this will not do

Worth writing down before starting, because each is where an ambitious version
quietly fails.

- **Predict arbitrary organic reactions.** Genuinely unsolved. Curate, and be
  visibly honest where we are predicting rather than knowing.
- **Mechanisms and transition states.** Quantum chemistry, build-time at best.
- **Extremes.** Plasmas, exotic organometallics, solid-state band structure,
  high pressure. (L2g's CEA data does extend T range honestly for gases and
  simple condensed phases; the databases' validity ranges are surfaced, not
  hidden.)
- **Biochemistry.** A different stack; a later module, not an extension.

A general-purpose engine that computes any reaction from first principles is also
a synthesis oracle for things we do not want it computing. Curated-first gives us
an explicit, auditable boundary — a product-safety property, much easier to
defend than a filter bolted onto a general predictor.

Optional later module, cheap and on-theme: radioactive decay chains (Bateman
equations — trivial math, public-domain nuclide data, and half-lives are a
curriculum staple).

---

## Build order

Genuinely sequential — each phase is shippable on its own, and each one depends
on the state model the previous one hardened.

### P0 — Feasibility spike

The single highest-information task. Everything else is downstream of it.

- [ ] Build IPhreeqc with **Emscripten** (primary — three existence proofs) and
      one mobile target; time-box a **wasi-sdk / `wasm32-wasip1` single-module**
      attempt as a stretch, never on the critical path
- [ ] Drive it through `LoadDatabaseString` / `RunString` with no filesystem
- [ ] Embed all four databases via `include_str!`, confirm binary size
- [ ] One end-to-end case: AgNO₃ + NaCl → saturation index out
- [ ] Fuzz it: random inputs → no crash, honest failure state
- [ ] **Gate:** if PHREEQC cross-compiles clean to web + one mobile target, the
      offline premise holds

### P1 — Bench state machine + energy balance + L0

- [ ] `Bench`/`Vessel`, mutating + measuring operators, operator log,
      re-equilibration between steps
- [ ] Enthalpy bookkeeping loop (ΔH + Cp → ΔT, iterate), vessel thermal modes
- [ ] **Reimplement** the 43×43 reactive-group matrix from the published NOAA
      methodology: our SMARTS group-assignment rules, our matrix encoding; strip
      every third-party field. Wire it as a veto that runs before any chemistry
- [ ] Conservation-invariant property tests from the first operator
- [ ] Do this *first*. Retrofitting a safety layer into a shipped app is where
      products get hurt.

### P2 — PHREEQC, shippable on its own

- [ ] `kerotakis-phreeqc` FFI surface (~15 functions matter)
- [ ] Acid–base, precipitation, titration curves, solubility, buffers, brines
      (pitzer.dat)
- [ ] Content-addressed result cache — same species set, T and P is the same answer
- [ ] This alone is a strong product

### P2g — Heat and fire

- [ ] Gibbs minimiser over NASA CEA `thermo.inp` (Apache-2.0): gas equilibrium +
      pure condensed phases. Evaluate adopting/extending `cea-rs` first
- [ ] Acceptance: CaCO₃ decomposition vs temperature; CH₄/air adiabatic flame T
- [ ] This is what makes `heat` and `ignite` computed chemistry rather than
      curated lookups — and it feeds ΔH into the P1 energy balance

### P3 — Phase behaviour

- [ ] `feos` integration (SAFT family + flash); `vle-thermo` for cubics +
      classical activity models; `seuif97` for water
- [ ] Own UNIFAC (~300 lines) against original-literature parameter tables with
      per-parameter provenance (see the UNIFAC section)
- [ ] Acceptance: ethanol–water azeotrope at 95.6%

### P4 — Curated reaction library + appearance

The slow, expensive, valuable part. This is the moat: nobody can scrape a
well-curated pedagogical reaction set with observations attached.

- [ ] Schema: balanced equation, conditions, ΔH, observations (colour, gas,
      precipitate, heat, smell), provenance, register-specific copy
- [ ] Indigo template application over homologues; our SMARTS incompatibility rules
- [ ] Colour data: species/precipitate/flame colours, indicator ε(λ) sets;
      Beer–Lambert + CIE integration in `kerotakis-appearance`
- [ ] Budget this as a chemistry-editorial hire, not an engineering task
- [ ] ORD decision (in or out, per the BY-SA row above) **before** the first
      record is ingested

### P5 — Kinetics + electrolysis

- [ ] diffsol integration. Reaction networks are **stiff** — explicit
      Runge–Kutta will not integrate them. Pure-Rust backends only; `diffsl`
      JIT stays off for iOS and wasm
- [ ] `kerotakis-electro`: Faraday's law + standard-potential ordering over
      PHREEQC speciation

### P6 — Build-time QM enrichment

- [ ] `tools/` pipeline batching xtb over the curated library
- [ ] Vibrational frequencies → synthetic IR spectra per compound
- [ ] Supervised TS searches only where a barrier genuinely matters
- [ ] Output is data; no xtb binary or library ships

### P7 — Lessons

- [ ] Declarative scenario format over the operator log; register narration hooks
- [ ] The nigredo → rubedo tier ladder
- [ ] Every lesson replays in CI

### ML tier — last or never

Molecular Transformer is trained on USPTO patent reactions and is weakest
exactly where our users are. If it ever ships: `candle` or `burn` (pure Rust,
real wasm stories), not `ort` (browser path is a JS bridge). Optional download,
never web-bundled, confidence always surfaced.

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

### Registry storage

SQLite (via `rusqlite`/`sqlite-wasm-rs`, wasm-proven) if the registry wants real
queries; `postcard`/`rkyv` + `include_bytes!` if it is read-only lookup. Decide
when L1 is built; both are wasm-clean.

---

## Governance

- **Licence:** AGPL-3.0-or-later, with an App Store / Google Play additional
  permission for binaries published by the copyright holder. See `LICENSE` and
  `NOTICE`.
- **The §7 trap, closed:** under GPLv3/AGPLv3 §7 only copyright holders can
  grant additional permissions. The moment an outside contribution is merged
  without a grant, the combined work can no longer ship under the store
  exception (VLC had to chase every contributor to fix exactly this).
  Therefore `CONTRIBUTING.md` requires, from the first PR, that all
  contributions are licensed **AGPL-3.0-or-later + the store exception**
  (inbound = outbound including the exception — the Nextcloud model; Signal's
  CLA is the heavier alternative if needed later).
- **Data licences are tracked separately from code** — per-source provenance
  files in `kerotakis-data`, reproduced in the app's about screen.

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

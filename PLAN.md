# Kerotakis — Development Plan

A virtual chemistry laboratory that computes real chemistry, accompanied by a
codex of a few hundred curriculum reactions and every concept that explains
them — from nine-year-old to expert, one simulation, rendered at every
register. Offline-first, no Python at runtime, the complete apparatus of
experimental chemistry as first-class operators.

Named for the sealed reflux vessel invented by Maria the Jewess in Alexandria,
1st–3rd century CE — the first named alchemist in recorded history. A sealed dome
with a sample suspended above a heated solvent, so the vapours act on it. She also
gave us the bain-marie; the airtight seal her apparatus needed is where
"hermetically sealed" comes from; and von Soxhlet's 1879 modernisation of it is
still in working labs today — and is itself an apparatus this lab will model.
The name describes the architecture: a sealed vessel you put things into, and
reactions happen. The product's voice — steampunk brass to necromantic glow —
lives entirely in data (operation names, narration templates, codex copy), never
in the solvers.

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
| How fast, concentrations over time | solved | diffsol (+ Cantera-format mechanisms) |
| Does an arbitrary organic reaction happen | **unsolved** | curate it |

PHREEQC is the highest-value engine here and almost no educational app uses it.
It solves speciation, mineral saturation, gas partitioning, redox and ionic
strength *simultaneously* from thermodynamic data. Mix silver nitrate and table
salt and it returns AgCl precipitate, how many moles, what ions remain, and the
saturation index — derived, not hardcoded.

The second row matters: heating and burning things is half of school chemistry
(CaCO₃ → CaO + CO₂, decomposing KMnO₄, dehydrating copper sulfate, magnesium in
a flame) and neither PHREEQC (aqueous) nor VLE (liquids) touches it. NASA
open-sourced CEA under **Apache-2.0 in 2026, including its thermodynamic
database** (`data/thermo.inp`) — so a small Gibbs-energy minimiser over NASA
polynomials with pure condensed phases turns "heat it" and "ignite it" into
computed chemistry, adiabatic flame temperature included.

### The build-time principle

**"No Python" is a runtime constraint. The build machine runs anything.**
Every heavyweight scientific tool becomes a build-time oracle, exporter or
verifier in `tools/`, and only *data* ships:

| Build-time tool | Role |
|---|---|
| xtb / CREST (LGPL) | Reaction ΔG, Fukui indices, MO data, approximate reaction paths, vibrational frequencies → IR spectra |
| PySCF (Apache-2.0) | Proper DFT wavefunctions for showcase FMO reactions; cube generation |
| Multiwfn (custom, verified commercial-OK) | Wavefunction analysis where PySCF needs help; cite both required papers, pin the version |
| `thermo` (MIT, Python) | Golden-test fixture generation for `kerotakis-thermo` — thousands of reference flash/VLE results |
| Cantera (desktop Python) | Reference solutions for combustion golden tests |
| RDKit (Python) | Cross-validation of Indigo template applications and canonicalisations during curation — two independent toolkits agreeing is real QA |
| RMG (MIT — verbatim MIT text under a custom header; GitHub shows NOASSERTION) | Benson group-additivity ΔHf/S/Cp(T) for arbitrary organics — the property gap PubChem/Wikidata cannot fill for the energy balance. **Blocker verified 2026-08-19: RMG-database has no LICENSE file.** Note `JacksonBurns/rmgdb` (active RMG developer, Aug 2026): vendors the database + SQLite/YAML repackaging under MIT — but with a *personal* copyright line, and one contributor cannot relicense a ~25-year collective work; no team licensing decision is on record (searched). It does make the upstream ask concrete: "affirm the licence upstream or correct rmgdb's copyright line to the RMG Team". Until then `thermo`'s Joback estimators (MIT) carry this role and no RMG parameter ships |
| ORDerly / ORD | Validation oracle only, never ingestion: check curated conditions against literature without touching ORD's CC-BY-SA (the same oracle pattern as `thermo` and Cantera). Patent-chemistry distribution → low relevance to a school codex; third-tier |
| UniChem (EMBL-EBI) | L1 identity crosswalk keyed on Standard InChI across 25 sources. EBI adds no restrictions beyond the original owners' — but no per-source licence field in the API, and a full dump includes DrugBank/CCDC accession lists (encumbered). **Use restricted to a cleared source whitelist** (PubChem, CompTox, Rhea) |
| DeepChem (MIT, active) | Build-time featurisation/toolbox if ever needed; its bundled datasets carry their own upstream terms |
| **Reaction-QM** (code BSD-3; **data CC BY 4.0**, Zenodo DOI-pinned, *Sci Data* 2026) | **Precomputed reaction energetics** — 2.3M GFN2-xTB + 200k B3LYP-D3/TZVP reactions with IRC-validated transition states and **ΔE‡/ΔH‡/ΔG‡ already in plain CSV** (57–71 MB; the 35 GB is geometry files we can ignore). Clean provenance (PubChem-enumerated, no copyleft corpus). Cuts a large slice of the L4′ xtb pipeline: where a curated reaction appears here, its barrier is free and B3LYP-quality. Caveat: machine-enumerated, often exotic reactions, ≤10 heavy atoms, 9 elements — an *enrichment and validation* source, never a codex source; prefer the B3LYP subset for anything shown to learners |
| cclib (BSD-3, actively maintained 2026-08) | Parses QC logfiles (Gaussian, ORCA, **xtb**, +17 more; PySCF via an in-memory bridge, not a parser). `ccget`/`ccwrite` slot into a Makefile as "QC output → structured JSON". Pin 1.8.1 (2.0 is alpha). May prove unnecessary — Reaction-QM ships parsed data |
| ncsw-data (MIT, solo, untested) | **Download-only** fetcher for ~20 reaction/compound datasets — verified to ship no data and to fetch from original upstream URLs, so it launders nothing. Value is its **URL inventory**; treat as a reference, not a dependency. Warning: it is a URL map, **not a licence map** — it hands over CC-BY-SA data without a word |
| PyTorch | Weight export (safetensors/ONNX) if the ML tier ever ships |

Tool licences (incl. geomeTRIC's "BSD 3-clause Non-AI" clause and Sella's
LGPL) are recorded in `tools/` provenance files; none of it ships.
(Evaluated and parked: `rxnfp` — MIT but dormant since 2021 and USPTO-trained,
the same distribution mismatch as Molecular Transformer; at most an afternoon
of codex-lint clustering someday, not a dependency.)

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

**L0 grades, it rarely refuses.** This is a pedagogical tool: for known
hazards the screen emits a **strong, precise warning first** — what forms,
what it would mean outside the simulation — and then the chemistry *proceeds
and shows exactly what would happen* (curated outcome where no solver covers
it). Mixing bleach and ammonia warns, then evolves chloramine you can watch
leave the vessel on the balance. Prohibition teaches nothing; precision plus
consequence does, and the virtual lab is the only place that is safe. The
hard `Veto` verdict exists in the type but is reserved for the
product-safety boundary ("What this will not do" — anything shading into
synthesis-oracle territory), not for curriculum hazards.

**Apparatus are operators.** Burette, distillation head, separating funnel,
pneumatic trough, Soxhlet — each is an operator (or operator + constraint set)
in `kerotakis-core`, drivable textually from day one; graphics come later.
"Never dumb down the model, only the view" applies to equipment too.

**Measuring ops are first-class and read-only.** They cost almost nothing (they
read existing solver output), they give register-appropriate precision naturally
— litmus for the nine-year-old, the same pH to three decimals for the expert —
and "measure before and after" *is* the scientific method being taught.

**The operator log is the save file.** Bench state is tiny; persisting the full
operator sequence gives undo, replay, sharing experiments as scripts, and golden
tests for free. It is also the substrate for lessons and the API contract.

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

**The cache ships pre-warmed.** Content-addressed: canonical species set + T +
P (floats quantised) → result. Because lessons enumerate their reachable
states, every vessel state in the curated lessons is computed at build time and
shipped — most user actions in guided content never touch a solver even on
first run, on any device. Also the graceful-degradation story for the rare
convergence failure.

### CLI first

The engine is text-native by construction — the bench loop *is* a REPL — so the
first client is a thorough CLI, and it is not a detour:

- **It proves the thesis before any UI exists.** `--register 9|15|expert` on
  the same session output tests "one simulation, many views" in the cheapest
  possible medium. If the registers don't work in text, no UI will save them.
- **It is the test harness.** Golden tests, conservation-invariant property
  tests, fuzzing, lesson replays — all are `kero run experiment.lab --json`
  under CI, diffing outputs.
- **It is the curation tool.** `kero codex lint` validates every codex entry:
  balanced equations, provenance records, register copy at all levels, and —
  the killer check — that claimed observations match what the solvers actually
  compute.
- **It is shippable.** PHREEQC's power with a humane interface is a real
  product for the expert register, publishable as `kerotakis` on crates.io
  (claiming the name with substance). A `ratatui` TUI skin later is optional
  flavour — brass-and-verdigris terminal chemistry — but REPL + batch + JSON
  comes first.

Two guardrails: **(1)** `kerotakis-cli` consumes `kerotakis-core` through the
same public API `kerotakis-wasm` will — both are thin peers over one boundary,
and `--json` output is the API contract, snapshot-tested. The moment the CLI
reaches into internals, the boundary mobile and web depend on stops being
hardened. **(2)** CLI comfort must not defer the P0 portability spike — a CLI
on macOS proves the chemistry, not the premise that can kill the project.

### The solver stack

Numbering is real: **L0 runs first and can veto**, and each layer depends on what
the ones below resolve.

| Layer | Role | Engine | Licence |
|---|---|---|---|
| **L0** | Safety & reactivity screen — runs first, grades the state | Our reimplementation of the NOAA reactive-group matrix (see below) | ours, from public-domain methodology |
| **L1** | Species & property registry, canonical InChIKey identity | SQLite/static data + Indigo (bundles the InChI plugin) | Apache-2.0 / MIT |
| **L2** | Aqueous equilibrium — **the workhorse** | IPhreeqc + phreeqc.dat, wateq4f.dat, minteq.v4.dat, **pitzer.dat** | USGS, public domain |
| **L2g** | Gas + condensed-phase equilibrium — heat, ignite, decompose, burn | Gibbs minimiser over NASA CEA data (adopt/extend `cea-rs`, or write it) | Apache-2.0 data |
| **L3** | Phase behaviour — boiling, miscibility, azeotropes | `feos` (SAFT family, flash) + own UNIFAC + `vle-thermo` (cubics, NRTL/Wilson) + `seuif97` (water) | MIT / Apache-2.0 |
| **L3e** | Electrolysis | Faraday's law + standard-potential ordering, own module; PHREEQC supplies speciation and Eh | ours |
| **L4** | Reaction — propose → filter → rank → verify | curated + Indigo templates | Apache-2.0 |
| **L4′** | QM enrichment — **build time only, never in the app** | xtb / CREST / PySCF | LGPL / Apache-2.0, never shipped |
| **L5** | Kinetics & time evolution | diffsol + our rate evaluator over Cantera-format mechanisms | MIT / BSD-3 data |
| **L6** | Appearance — colour, cloudiness, flames, orbitals | curated colour data + Beer–Lambert over ε(λ) + CIE via `palette`; precomputed orbital meshes | ours / MIT |

Notes per layer:

- **L0** — no hardcoded compound pairs in the end state. The screen is a
  cascade (mirroring L4's), each layer labeled with what produced it:
  1. **Curated codex outcome** — shows exactly what forms (the current seed
     entries live here).
  2. **Computed outcome** where a solver genuinely covers it: chlorine
     evolution from hypochlorite + acid is redox speciation (PHREEQC,
     llnl.dat-class database — spike this; if it works the entry moves from
     curated to computed); an L2g Gibbs minimisation finding a strongly
     exothermic accessible state (high adiabatic flame T) *is* an
     energetic-mixture detector; build-time RMG/xtb ΔH_rxn plus classical
     indicators (oxygen balance, energetic functional groups) give
     CHETAH-style screening — the ASTM tool is proprietary, the methods are
     published.
  3. **Reactive-group matrix** — two computable stages: structure → groups
     is a SMARTS match (Indigo; the current key→group table is a placeholder
     for this), and group × group → **predicted consequence category**
     (toxic gas, heat, flammable gas, polymerisation — the published chart's
     cells carry these). ~70 groups squared covers millions of compound
     pairs; nothing per-pair is stored.
  4. **Per-substance GHS data** — H-codes/pictograms/signal words from
     **PubChem bulk** (no NCBI restrictions, per-record attribution) and
     **CLP Annex VI via EUR-Lex** (EU legislation, reusable), keyed by
     InChIKey into L1; concentration-awareness via the GHS's own published
     mixture-classification rules (additivity formulas, cut-offs) — a drop
     of bleach and a beaker of it get different verdicts, from arithmetic.

  **Deliberately not computed: predicted toxicity (QSAR).** Authoritative
  classification beats prediction on reliability and liability — a wrong
  "safe" is the one failure mode this product cannot have. Below all four
  layers the honest output is "outside the screened set", never a guess.

  Legal position, stated precisely: the CAMEO/CRW4 *database* is not
  redistributable (its terms fence off CAS RNs, NFPA ratings, AEGLs,
  ERPGs — none of which the design needs). The methodology's facts are not
  copyrightable under *Feist*, and the SMARTS assignment rules are ours —
  but Gorman et al. 2014 is a Wiley paper with CCPS (AIChE) co-authors, so
  **source the matrix from the NOAA Institutional Repository copy**
  (repository.library.noaa.gov/view/noaa/61941), record the derivation, and
  keep it on the counsel list. Verify the group-set generation before
  encoding (classic 43×43 vs the ~68 groups current CAMEO materials
  reference), and verify NITE/other national GHS sources' terms if used
  beyond PubChem's aggregation.
- **L1** — InChI/InChIKey has exactly one implementation, the IUPAC C library,
  **relicensed MIT with v1.07** (plain C, current 1.07.5). We get it through
  Indigo's bundled InChI plugin via the same FFI; a standalone `kerotakis-inchi`
  binding is the fallback (wasm precedent: cheminfo's `inchi-js` npm package).
- **L2** — `pitzer.dat` is only **37 KB** and public-domain like the rest; it
  unlocks brines and high ionic strength (seawater evaporation is a beautiful
  teaching sequence). Embed it. **Do not embed `sit.dat`**: generated from
  ANDRA's ThermoChimie database — non-USGS provenance; revisit only after a
  terms check. Upstream is the actively maintained `phreeqc-dev` GitHub org
  (CMake, C++14).
- **L2g** — a Gibbs minimiser over NASA polynomials is a well-understood,
  few-hundred-line solver. `cea-rs` (MIT OR Apache-2.0, wasm ✓) appeared on
  crates.io in Aug 2026 — embryonic, but proof the port is tractable and a
  candidate to adopt and extend rather than start from zero.
- **L5** — reaction networks are **stiff**; explicit Runge–Kutta will not
  integrate them. diffsol's default backends are pure Rust (nalgebra/faer);
  the `diffsl` JIT stays off for iOS and wasm. For gas kinetics we parse the
  **Cantera YAML mechanism format** (publicly documented; the shipped data
  files — gri30.yaml etc. — are part of Cantera's BSD-3 distribution and
  freely redistributable) and evaluate rates ourselves: GRI-Mech-class
  education mechanisms need only **Arrhenius + three-body + Troe falloff**.
  Precedent: KiThe and Fauconneau/combustion in Rust; Arrhenius.jl and
  ReactionMechanismSimulator.jl in Julia.
- **L6** — most of the age-9 register is *observations*, and they need a
  computation path — but sized to its data sourcing. **Default: curated sRGB
  per species + concentration-driven opacity** — 95% of the pedagogy at 5% of
  the cost, and the data is a colour word per species, not a spectrum. The
  full Beer–Lambert-over-ε(λ) + CIE path is reserved for **indicators only**,
  where the dataset is small, classical and openly published. (Full spectra
  for the interesting colours are the hard case: Cu²⁺/MnO₄⁻/Cr₂O₇²⁻ are d–d
  and charge-transfer bands; sTDA-xTB *can* compute UV-Vis at build time but
  is weakest exactly there, published spectra carry copyright, and NIST is
  off-limits per the data table.) `palette` (wasm-fine) handles the colour
  math either way; we supply the CIE integration for the indicator path.

### L4 is a cascade, not a choice

The stage that produced an answer is **shown to the user**.

1. **Propose** — curated library first. A few hundred hand-verified curriculum
   reactions with conditions, ΔH and observations covers all of school and most
   of undergraduate chemistry, and it is *correct*, which matters more than
   coverage in education. Indigo's `indigoReactionProductEnumerate` /
   `indigoTransform` (both verified present in the current flat C API)
   generalise templates across homologues.
2. **Filter** — our own SMARTS incompatibility rules, plus the L0 pass. RDKit's
   shipped `FilterCatalog` (PAINS, BRENK, NIH, ChEMBL) is a set of
   *medicinal-chemistry alerts*, not reaction-feasibility rules. The rules are ours.
3. **Rank** — surface confidence, never present a prediction as a fact.
4. **Verify** — L4′, offline.

### L4′: the QM pipeline, and what it ships

GFN2-xTB is a *verifier*, not a generator — something else proposes the
mechanism. Barriers need a supervised saddle-point search plus frequency and
IRC confirmation; that pipeline fails often and cannot sit behind a user
tapping "mix". So it runs on the build machine, and ships as data. Batching it
build-time also dissolves the LGPL-3.0 relinking conflict with App Store
distribution — the binary never ships.

What the pipeline emits per curated reaction/compound (capabilities verified
against xtb docs):

- **Energetics** — ΔG_rxn (GFN2-xTB), the honesty backbone of L4.
- **Fukui indices** — `xtb --vfukui`: per-atom condensed f⁺/f⁻/f⁰. Colouring
  atoms by the dual descriptor explains regioselectivity *before a bond forms*
  — tiny data, huge pedagogy.
- **Frontier orbitals** — HOMO/LUMO energies and MOs (`--molden`); proper DFT
  wavefunctions from **PySCF** for the showcase FMO set (Diels–Alder, SN2,
  carbonyl additions) where orbital *shape* is the lesson. Honest-labelling
  caveats: GFN2's minimal valence basis gives qualitatively sensible valence MO
  ordering/symmetry but compressed gaps and no anion/diffuse orbitals — the
  register system says which engine produced what.
- **Reaction-path frames** — `xtb --path` (RMSD-PP) gives approximate paths
  *without a transition state*: the SN2 umbrella inversion and carbonyl
  sp²→sp³ animations. True IRC only where a supervised TS search earned it.
- **IR spectra** — vibrational frequencies → synthetic spectra: licence-clean
  (computed, not scraped from NIST), powering the spectrophotometer instrument
  and "identify the unknown" lessons.

**Some of this is already computed.** Reaction-QM (CC BY 4.0, DOI-pinned) publishes IRC-validated transition states and ΔG‡ for 200k B3LYP-quality reactions in plain CSV — so for any curated reaction with a match there, the barrier is a lookup rather than a supervised saddle-point search. Our own xtb/CREST pipeline then covers what the dataset does not, which is most curriculum chemistry (it is machine-enumerated organic space, ≤10 heavy atoms). Check the dataset first, compute second.

**Orbitals ship as meshes, not cubes.** Build-time marching cubes → quantised
glTF keyframes (KHR_mesh_quantization; skip Draco — its decoder outweighs the
savings at these sizes). Verified ~60–300× smaller than cube files; the direct
ancestor is Jmol's JVXL format (400–1000× published). Two-colour signed lobes
give phase-matching visuals — [4+2]-allowed vs [2+2]-forbidden — as plain mesh
rendering, so **no chemistry viewer is required at runtime**: the app's own 3D
layer plays orbital animations like any other asset. For the web expert
register, where raw-cube interaction earns its place: **3Dmol.js** (BSD-3,
active, ~158 KB gzipped, native cube rendering with two-isovalue phase
colouring). Mol* (MIT, very active, ~10× heavier; its `alpha-orbitals`
extension computes MOs on-grid client-side) stays on the watch list; NGL is
dormant — skip.

**The runtime QM tier: Hückel, honestly labelled.** For molecules the *user*
draws, no precomputation exists. No pure-Rust Hückel implementation exists, and
tblite (LGPL Fortran) has no realistic wasm path (gfortran cannot target
wasm). Options, per the adopt-and-extend policy: port or FFI **YAeHMOP**
(extended Hückel, plain C, BSD-2-Clause, small, Landrum-maintained), or write
simple Hückel ourselves — it is essentially an eigensolver over a connectivity
matrix, which nalgebra/faer provide. Qualitative π-MO phases and energies,
labelled approximate; textbook FMO diagrams *are* Hückel theory, so the
approximation is literally the pedagogy.

### Every answer carries its provenance, and the paths are shown

An answer is not just a number. The bench records, for each result: which
**engine** produced it, which **dataset** it consulted, which **model** that
dataset applies (derived from the file's own declarations — a `PITZER`
block means specific-ion interaction; per-species `-gamma` means the WATEQ
Debye-Hückel extension), **why that path was routed** over the
alternatives, and the **literature the dataset itself cites** — those
citations sit in the data files as comments (minteq.v4 carries ~310) and we
capture them instead of stripping them.

And where more than one dataset can express the question, `kero explain`
asks **all of them** and shows the disagreement rather than asserting one
answer. Saturated brine, live:

```
v1: answered by PHREEQC (IPhreeqc, USGS) using pitzer.dat
  model:   Pitzer specific-ion-interaction model (valid at high ionic strength)
  routing: chosen because the solution is concentrated (~16 mol/kgw)
the same question, asked of every dataset:
  wateq4f.dat    pH 7.059 · I = 6.42 m · Halite 1.597 mol
    WATEQ Debye-Hückel extension (reliable to about I = 1 mol/kgw)
  minteq.v4.dat  pH 6.855 · I = 3.69 m · Halite 4.317 mol
  pitzer.dat     pH 6.469 · I = 6.11 m · Halite 1.908 mol
```

A dataset that *cannot* express the question (pitzer.dat has no silver)
says so and names what it lacks, rather than being skipped or answering
wrongly. This is the honesty rule taken to its conclusion: not "here is the
number" but "here is the number, here is what computed it, here is where
that came from, and here is what the alternatives say." It is also the
expert register's deepest layer and, for the codex, the model for how
curated entries cite their sources — DB value, computed value, and which
model produced each.

### The pedagogy is the architecture

School chemistry overfeeds facts and underteaches the models that make
facts predictable. That is not a complaint about teachers; it is what the
chemistry-education literature has documented for forty years, and this
app is unusually well placed to do something about it — because it
*computes* the level that textbooks can only assert.

**The core problem has a name.** Johnstone's triangle: chemistry lives at
three levels — the **macroscopic** (what you see), the **submicroscopic**
(particles, ions, species) and the **symbolic** (equations, formulae) —
and novices fail because instruction moves between all three at once
without saying so. A textbook shows a photograph, an equation and a
particle diagram on one page and leaves the learner to guess how they
relate.

**Our engine closes that gap structurally.** PHREEQC computes the
submicroscopic level for real: 3×10⁻⁷ mol/kgw of AgCl(aq) is not an
illustration, it is the answer. So the same vessel state can be rendered
at all three levels *simultaneously and consistently*, which is precisely
the move the literature says learners cannot make unaided.

**Two axes, not one — and we had this wrong.** An earlier version of this
plan mapped lv1/lv2/lv3 onto macroscopic/symbolic/submicroscopic. That is
an error, and worth recording as one: a speciation table with molalities
and activity coefficients is not the submicroscopic level, it is *deeper
symbolic*. It is still numbers and formulae. Under that mapping the
triangle was never closed — we had macroscopic, symbolic, and more
symbolic.

Representation and detail are **orthogonal**:

|              | **lv1** | **lv2** | **lv3** |
|---|---|---|---|
| **macroscopic** | "It went cloudy." | "A white precipitate, ~1.4 g." | mass, turbidity, computed sRGB from ε(λ) |
| **submicroscopic** | a few dots, two kinds | dots at computed ratios, pairs touching | hydration shells, ion pairs, γ as crowding |
| **symbolic** | `silver + salt → chalky stuff` | `Ag⁺ + Cl⁻ → AgCl↓`, Ksp | activities, log K, saturation indices |

Every cell is a rendering of one solved state. The **submicroscopic row is
the missing vertex**, and building it is not polish: the engine already
computes the census (`Ag⁺ 9.56e-6 · AgCl(aq) 3.21e-7`), so drawing dots at
solved ratios is honest in a way a textbook diagram is not — the picture
is the answer, not an artist's impression of it. It can start humble: 2-D
dots at representative ratios, no molecular dynamics. It belongs in the
v1.0 UI spec.

**Guided model-building, not discovery learning.** Pure discovery fails —
Kirschner, Sweller & Clark (2006) is the decisive review: minimally
guided instruction overloads working memory and novices flounder. This is
a real constraint on a virtual lab, because "here is a bench, try things"
is exactly the failure mode. What works is *structured* model-building:
a paradigm experiment, a model built from it, deployment on new cases,
a case that breaks it, revision — with the guidance heavy early and faded
as competence grows. Modeling Instruction (Hestenes) and POGIL are the
best-evidenced structures. The codex's `requires` graph and its
predict-then-run cycle are that shape; the free-form REPL is the *faded*
end of the scaffold, not the entry point.

**Simulation is the right medium for this specific job.** The PhET
programme's finding is that for *concept* building, good simulations often
beat real labs, because they make the invisible visible and strip the
extraneous load of glassware logistics. The real lab keeps what software
cannot honestly supply: measurement skill and the epistemic messiness of
real data. We should say that plainly rather than pretend to replace the
bench.

**Organise around a few load-bearing models, not many topics.** The
candidate spine: the particle model of matter; the mole as the bridge
between mass and count; energy in bonds and reactions; structure-property
relationships; the electron-shell/orbital family; and the acid-base and
redox model families. Everything else is an application of one of these.
Acid-base is the ideal nature-of-science vehicle because school chemistry
already contains three models of expanding scope (Arrhenius → Brønsted →
Lewis) — taught *as* successive models with explicit domains rather than
as three facts, it is philosophy of science for free. `codex/models.toml`
already carries that chain.

**Models are content, not background.** The codex carries `[[model]]`
entries beside its reactions, and the load-bearing field is `fails_at`.
A model presented without its boundary is presented as truth — which is
false, and is why the next model feels like an arbitrary replacement
rather than an answer to a problem the learner can already feel. The
particle model cannot say *why* sodium and chlorine react; knowing that is
what makes the electron model worth having.

**Nature of science has to be named out loud.** Lederman and
Abd-El-Khalick's finding is uncomfortable and load-bearing for us: simply
*doing* labs teaches essentially nothing about how science works. Students
learn what a model is, why models have domains, and how evidence
adjudicates between them only when it is taught **explicitly and
reflectively**. So the app must say the quiet part: "we are now comparing
two models of acids; each explains some things and fails on others." An
engine that silently routes between three thermodynamic databases teaches
nothing about models. One that *says* it is routing, and shows the three
answers disagreeing, teaches the central idea of the discipline.

**And we can show a model failing, live.** `kero explain` already asks
every dataset the same question and prints the disagreement: three
activity models give 1.60, 4.32 and 1.91 mol of undissolved salt, each
with its validity range stated. That is philosophy of science as a
computed result rather than a paragraph — the boundary of a model made
visible by driving the model past it. No textbook can do this; we got it
by accident, chasing honesty, and it turns out to be the pedagogical core.

**Prediction comes before observation.** Predict-observe-explain is the
best-evidenced sequence in science education, and it only works if the
prediction is *committed* first. Codex entries therefore carry an optional
`predict` block: a question, plausible options drawn from the mistakes
learners actually make, and the misconception each wrong answer reveals.
The engine is the arbiter — and because it computes rather than looks up,
a quantitative prediction can be checked, which makes calculation
load-bearing instead of ritual. Working out moles matters when the number
you derive is the number the beaker will show.

Two commitments follow. First, **distractors carry their own diagnosis**:
each wrong option names the misconception it reveals and what to do about
it, rather than the entry carrying one blanket note. Misconception
*prevalence findings* are research facts rather than copyrightable
expression, so we cite them and write our own options — AAAS Project 2061's
item bank, Taber's *Chemical Misconceptions* (RSC), Barke's *Misconceptions
in Chemistry* for the German line, Driver's *Making Sense of Secondary
Science* as the compendium, and Treagust's two-tier format. Where no
citation exists the entry says `Editorial judgement (Kerotakis)`. This is
also the *better* path, because a distractor has to match what our engine
actually computes, not what a textbook rounds to.

Second, **retrieval practice, spacing and interleaving are applied to
model-use, not trivia**. "Given these two elements, predict the bond type
and justify" is worth spacing; "what is the atomic number of vanadium" is
not. The unit of review is a prediction, which is what the `predict` block
already is.

**The order is the dependency structure, not the school year.** School
years are an artefact of national administration and differ by country;
the order in which the ideas depend on each other is not. `teaching_order`
is a topological sort over `requires`. Curriculum placements stay on each
entry so a learner who needs to find their syllabus topic still can — the
app follows the subject, and meets the school where it is.

**Antecedents we are consciously working in.** Martin Wagenschein's
*exemplarisches Lernen* (1956) — teach few phenomena genetically and
deeply rather than covering everything thinly — is this manifesto seventy
years early, and it is the German tradition the project is written from.
*Chemie im Kontext* (ChiK, IPN Kiel) is the modern programme that anchors
concepts in contexts and has real evaluation behind it. Neither is a
licence question; both are prior art we should be honest about rather than
present the approach as novel.

**Where we deviate from convention, we do it knowingly and say so.** The
deviation is not novelty for its own sake: it is subordinating facts to
models, making boundaries explicit, and asking for a prediction first.
Facts are not fewer — they are *organised*, which is the condition under
which they are retained at all.

### Stoichiometry: ours, and why not ChemicalFun

ChemicalFun (thermohub, LGPL-2.1) provides the stoichiometric layer we
lacked — formula parsing, composition matrices, automatic balancing. We
build it instead of linking it, for two reasons that point the same way.

**Licence.** LGPL-2.1-*only* is incompatible with AGPL-3.0; the *or-later*
form is compatible via LGPLv3. Even where compatible, LGPL's relink
requirement is awkward for statically linked iOS and wasm store binaries,
and our store exception grants nothing about third-party code. Linking is
the problem; *running* is not.

**Cost.** Balancing is the null space of the element-count matrix, with one
extra row for charge. That is a few hundred lines against machinery we
already had — a formula parser in two places and Gaussian elimination in
the Gibbs minimiser — so the dependency buys little and costs a licence
question on every target.

`crates/kerotakis-core/src/stoich.rs` therefore does it: Unicode
subscripts and superscripts, parenthesised groups, hydrate dots, state
labels, both charge notations, and `kero balance` for the exercise. It
balances dichromate against iron(II) — 14 H⁺ and 7 H₂O — which is the
university case, not a toy.

**They remain valuable as build-time oracles**, and that use is clean:
running a program over public data does not make the output a derivative
work, which is the same reasoning behind the Python `thermo` fixtures in
P3p. Two jobs worth doing:

- **Differential testing.** Run ChemicalFun/ThermoFun over a corpus at
  build time and diff against our Gibbs minimiser and PHREEQC, checked in
  as fixtures. The conservation bug found on 2026-08-19 would have been
  caught in a day rather than by accident.
- **Completeness checking.** ChemicalFun can enumerate every balanced
  reaction among a substance set; diffing that against the codex yields
  "reactions our registry could already teach but does not" — a coverage
  metric derived from chemistry rather than from a topic list.

Conditions: keep such a tool out of the build graph and out of `vendor/`
so nothing links it accidentally, and keep the *tool* licence separate from
the *dataset* licence — ThermoFun's data carries its own terms, some not
commercial-friendly, exactly as we already track for the PHREEQC databases.

ChemReaX is a closed web application. It is legitimate to consult by hand
as a chemist would; it is not a source we can automate against or
redistribute from.

### Declining to model something must be loud

Three bugs found on 2026-08-19 were the same bug wearing different clothes,
and the pattern is worth naming so it stops recurring.

- `ignite` on ethanol reported **"nothing ignited"**. Ethanol has no
  condensed form in the NASA data, so the thermal solver never engaged —
  the bench was reporting the absence of a model as an observation about
  the world.
- Copper sulfate plus lye reported **pH 9.88 holding 0.01 mol/L of Cu²⁺**.
  That solution cannot exist: it is grossly supersaturated against two
  solids (`Cu(OH)2`, `Tenorite`) that are *already in the databases we
  ship* but absent from our registry, so the phase could never be admitted.
- Cooling a beaker past 0 °C reported **liquid water at −7.95 °C, with a
  pH**, because nothing re-evaluates state.

In every case a *filter* — the honesty boundary that says only species we
can name may appear — behaved as a *fact*. The boundary itself is right and
should stay: a solver that reaches for an exotic carbide we cannot name
must either drop it (losing mass) or show a formula with no story attached.
The defect is that the filter was **silent**.

**The rule: every place the engine declines to model something, it says
so.** A state we cannot characterise is reported as uncharacterised, never
returned as the state. Concretely this means the honesty pass must read
back **saturation indices** rather than only compositions, so "the solution
you are looking at is supersaturated against a phase this lab cannot name"
becomes a sentence the user sees. That single change closes the general
class rather than the copper instance, and it surfaces SI — which the model
audit independently flagged as computed-but-never-displayed.

### Known gaps, written down where they can be found

- **Solid-to-solid conversion carries no heat.** The energy balance reads
  `Dissolved` and `Precipitated` events; a solid turning into a different
  solid emits neither, so slaking lime (CaO + H₂O → Ca(OH)₂, about
  −82 kJ/mol) shows the vessel *cooling*. A bucket of quicklime steams and
  is a burn hazard, and we get the sign wrong. The fix is an event for a
  consumed solid plus reaction enthalpies on phase conversions, not a
  number bolted to the registry.
- **Added gas does not dissolve.** `add v1 CO2` leaves the CO₂ sitting as a
  gas phase, so limewater cannot go milky — one of the commonest school
  tests. Open-vessel *venting* works (the fizz loses its CO₂ correctly);
  it is the inward direction that is missing.
- **Partial freezing is not modelled.** A freezing solution really gives ice
  plus an ever more concentrated brine down to a eutectic; we freeze the
  solvent and say the rest is not modelled.

### Thermodynamic product versus kinetic product

Copper turns out to be a *better* problem than a missing phase, and it earns
a place in the plan rather than a patch.

```text
Cu(OH)2  + 2H⁺ = Cu²⁺ + 2H₂O    log_k 8.674
Tenorite + 2H⁺ = Cu²⁺ +  H₂O    log_k 7.644   (CuO)
```

Tenorite is more stable by ~1.03 log units, so **equilibrium says black
CuO while the beaker shows pale blue Cu(OH)₂ gel**. That is Ostwald's rule
of stages: the metastable phase nucleates first because it is kinetically
accessible, and heating converts blue to black — which is the classic
school demonstration.

So a Gibbs-minimising engine is *structurally* unable to reproduce one of
the most-performed reactions in school chemistry, and the reason why is
itself first-rate content. The design consequence: phases may be marked
**metastable under stated conditions**, as *data with provenance*, not as
special cases in code — the same shape as the thermal solver's 500 K
stand-down, which is the same admission (equilibrium is not the whole
story) made once already. Done properly, "heat the blue precipitate and it
turns black" becomes a **computed prediction**: raise the temperature,
lift the suppression, tenorite wins.

### The codex

A few hundred curriculum reactions, and **every concept that explains them**,
as one dataset:

- **Concept graph** — a petgraph DAG: concepts → reactions they explain,
  prerequisite edges between concepts. The difficulty ladder made explicit and
  queryable; the nigredo → albedo → citrinitas → rubedo tiers are cuts through
  this graph.
- **Reaction entries** — balanced equation, conditions, ΔH, observations
  (colour, gas, precipitate, heat, smell), provenance, register copy at every
  level, and the L4′ enrichment block (ΔG, Fukui colouring, FMO pair + gap +
  phase-match verdict, path frames, IR spectrum).
- **Flavour is data.** Operation names, narration templates, codex voice — the
  steampunk-to-necromancer register lives in these files and survives any UI
  decision untouched.
- **Markup decided early.** The codex rendering convention (register copy,
  diagrams, concept pages) is chosen during authoring, not after hundreds of
  entries exist — the renderer can come late, the format cannot.
- `kero codex lint` enforces all of it mechanically, including that claimed
  observations match solver output.

Budget curation as a chemistry-editorial role, not an engineering task. This is
the moat: nobody can scrape a well-curated pedagogical reaction set with
observations, orbital stories and register copy attached.

---

## The crate stack, verified

| Crate | Licence | Status (2026-08) | Role | wasm |
|---|---|---|---|---|
| `feos` | MIT OR Apache-2.0 | active, v0.10.1 | L3 core: PC-SAFT, gc-PC-SAFT, multiparameter Helmholtz, full flash machinery. **No UNIFAC** | ✓ |
| `num-dual` | MIT OR Apache-2.0 | active | AD backbone of feos; exact fugacity/enthalpy derivatives | ✓ |
| `vle-thermo` | MIT | active, very young (May 2026) | 22+ cubic EOS, NRTL/Wilson/van Laar, Rachford–Rice flash, phase envelopes | ✓ |
| `seuif97` | MIT | active | IAPWS-IF97 water/steam — most of our solvent story | ✓ |
| `diffsol` | MIT | active | L5 stiff ODE/DAE (BDF), pure-Rust nalgebra/faer backends; `diffsl` JIT **off** for iOS/wasm | ✓ (JIT off) |
| `cea-rs` | MIT OR Apache-2.0 | embryonic (Aug 2026) | L2g seed — adopt/extend or rewrite over the same Apache-2.0 CEA data | ✓ |
| `nalgebra` | Apache-2.0 | active | Stoichiometric null-space balancing, eigensolver for Hückel, linear algebra | ✓ |
| `petgraph` | MIT OR Apache-2.0 | active | Concept graph, reaction-network DAGs, cascade routing | ✓ |
| `uom` | Apache-2.0 OR MIT | alive, ~1 release/yr | Compile-time units at `kerotakis-core` API boundaries — kills molarity-vs-molality bugs | ✓ |
| `palette` | MIT OR Apache-2.0 | active | L6 colour math (XYZ/Lab/sRGB); we supply the spectral→XYZ integration | ✓ |
| `rusqlite` ≥ 0.38 | MIT | active | Registry/codex if queryable storage wins. wasm via `sqlite-wasm-rs` (also Diesel's official wasm backend) | ✓ |
| `postcard`/`rkyv` | MIT etc. | active | Alternative for read-only bundled data: `include_bytes!`, zero-copy | ✓ |

**Evaluated and set aside** (with the reason, so we don't re-litigate):

- `purr`, `gamma`, `chemcore` — frozen at 2021 proof-of-concept state; no SMARTS
  matching, no canonicalisation, no InChI. Their author, Rich Apodaca, died in
  2024; the successor `balsa` is also dormant. Not a base to build on.
- `sundials-sys` — dormant ~20 months; no evidence anyone has ever built
  SUNDIALS for wasm or iOS. diffsol covers L5 in pure Rust.
- `coolprop-sys` — actively maintained but bundles **prebuilt desktop dylibs
  only**: no wasm, no iOS/Android. CoolProp's official Emscripten/JS build
  keeps it available as an optional *desktop/web extra*, not core.
- `KiThe` — hard, non-optional `reqwest`/`tokio` deps (its NIST WebBook
  scraper) fail on wasm ✗, and scraping WebBook is an SRD-licensing problem
  anyway. Fork candidate only if its equilibrium code outperforms our L2g.
- `ort` — iOS/Android static binaries exist, but the browser path is `ort-web`
  bridging onnxruntime-web: two incompatible wasm contexts. If the ML tier
  ever ships: `tract` (pure Rust, wasm-clean) for small models, `candle` /
  `burn` for larger. Deferred with the ML tier itself.
- `rdkit-rs` — dormant, needs native C++ RDKit, no wasm story; RDKit
  MinimalLib's maintainer stepped down 2026-04. Indigo is primary at runtime;
  RDKit serves at build time (oracle table above).
- NGL Viewer — MIT but dormant; its author moved to Mol*. Skip.

**Watch list** (young or needs-work, but filling real gaps — see policy below):

- `chematic` — pure-Rust cheminformatics with real SMARTS + VF2 substructure
  matching, canonical SMILES, 2D depiction, first-class wasm npm build. Three
  months old, bus-factor 1, self-reported RDKit parity. If it matures it
  replaces the Indigo FFI for everything except InChI; re-evaluate quarterly.
- **Cantera via its generated C API** — Cantera 3.2 ships a generated clib
  (handle-based, no C++ at the boundary) covering equilibrate, kinetics rates
  and reactor networks, and **Emscripten/wasm support was merged upstream**
  into the 4.0 dev branch (2026-03; lead maintainer: "essentially no issues
  compiling Cantera and its dependencies as a WASM library"). Still SCons-built,
  clib marked experimental, mobile unproven. Our slice reimplementation (L5)
  covers the educational need; full Cantera-by-FFI becomes a real option when
  4.0 ships — re-evaluate then.
- `teqp` (NIST) — **public-domain** multiparameter/GERG/SAFT EOS in C++;
  Emscripten side-module feasible. The option if L3 ever needs
  reference-quality multiparameter mixtures beyond feos.
- `GEMS3K` (PSI) — LGPL-3.0 Gibbs minimiser, markedly better than PHREEQC for
  non-ideal solid solutions and melts. LGPL manageable for desktop/server,
  awkward for static wasm/App Store. The option if L2 hits that wall.
- `mcubes` — MIT marching cubes written for electron-density meshing; young
  0.1.x. Build-time mesh pipeline first choice; vendoring the ~200-line
  algorithm is the fallback.
- **YAeHMOP** — extended Hückel, plain C, BSD-2-Clause, small,
  Landrum-maintained. Port-to-Rust or FFI candidate for the runtime Hückel tier.
- the alpha `phreeqc` npm package (Emscripten, MIT + USGS notice, Jan 2026) —
  single-maintainer alpha; not a dependency, but the existence proof for our
  P0 build and worth reading first.

### Adopt-and-extend policy

A tool is not disqualified because it needs work from us — forking, wasm
compiles, FFI bindings, feature-gating out bad deps — **if it fills a gap no
equally good, licence-compatible tool fills**. `cea-rs`, `chematic`, `teqp`,
YAeHMOP, the Cantera clib, a KiThe fork and the `unifac` crate's algorithm are
all in that category. A tool *is* disqualified by: incompatible licence on code
we'd ship (GPL-only, NC), non-redistributable embedded data, or a dead upstream
*plus* an equally good maintained alternative. When we extend, we upstream
patches where the project is alive and fork visibly where it is not.

### The UNIFAC question, precisely

No clean-licensed, maintained Rust UNIFAC exists. The `unifac` crate (frozen
2021, wasm ✓) is MIT **code** with a warning clause about its embedded
parameters — and that problem attaches to *any* implementation, including one we
write: the maintained UNIFAC Consortium tables are proprietary; the original
open-literature tables (Fredenslund, Gmehling et al., 1970s–90s journals) are
usable. So: reimplement the ~300-line algorithm (or fork the crate), source
parameters from the original publications, and record provenance per parameter.
Budget it as data curation, not coding. Acceptance test: the ethanol–water
azeotrope at 95.6% — a genuine teaching moment most simulators miss.

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

**Done and CI-enforced:** `kerotakis-wasm` runs the bench in a wasm runtime —
thermal chemistry computed live (magnesium ignites, burns to the oxide,
~3000 K, narrated for a nine-year-old), the species shelf exposed with
provenance, and **whole `.lab` lessons replayed** by the same grammar the CLI
uses (moved into the core, so a lesson behaves identically in both and its
pre-warmed results match exactly). Aqueous chemistry comes from the shipped
results, because a browser cannot link IPhreeqc's C++: the marquee lesson
replays with **no engine present**, and a state nobody pre-computed is
reported as a stated miss rather than guessed at. The engine is now a cargo
feature (`--no-default-features` gives the cache-only build), which is what
makes that split clean rather than a fork.

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
│   │                         balance, solver router, measurement ops, registers
│   ├── kerotakis-cli/        REPL + batch + --json; the harness, the curation
│   │                         tool, and the first shippable client
│   ├── kerotakis-phreeqc/    IPhreeqc FFI + embedded databases (L2)
│   ├── kerotakis-cea/        Gibbs minimiser over NASA CEA data (L2g)
│   ├── kerotakis-indigo/     Indigo FFI — structures, InChI, templates (L1/L4)
│   ├── kerotakis-thermo/     feos + own UNIFAC + vle-thermo + seuif97 (L3)
│   ├── kerotakis-electro/    electrolysis — Faraday + potential ordering (L3e)
│   ├── kerotakis-kinetics/   diffsol + Cantera-YAML mechanism parser (L5)
│   ├── kerotakis-appearance/ colour: curated data + Beer–Lambert + CIE (L6)
│   ├── kerotakis-safety/     reimplemented reactive-group matrix + rules (L0)
│   ├── kerotakis-huckel/     runtime qualitative MO tier (own / YAeHMOP)
│   ├── kerotakis-data/       codex + registry + pre-warmed cache, embedded
│   └── kerotakis-wasm/       wasm-bindgen surface for web (Track A)
├── tools/                    build-time pipelines: xtb/PySCF batches, orbital
│                             meshes, IR spectra, data exports, oracles,
│                             Indigo wasm build
├── lessons/                  declarative scenario files
└── app/                      UI — see the open decision below
```

`kerotakis-core` is the invariant. It compiles to `wasm32-unknown-unknown` and to
aarch64-apple-ios, aarch64-linux-android, x86_64-pc-windows-msvc and
aarch64-apple-darwin from one source.

### Testing is part of the architecture

- **Conservation invariants** — property tests asserting mass and charge balance
  across *every* operator, on random benches. Catches whole classes of bugs no
  example test finds, and it is a moat: lookup-table competitors cannot even
  state the invariant.
- **Golden tests** — textbook values: acetic-acid titration curve, AgCl Ksp,
  ethanol–water azeotrope, CaCO₃ decomposition temperature, adiabatic flame T —
  plus oracle-generated fixtures from `thermo` and Cantera (build-time Python).
- **Fuzzing PHREEQC** — random vessel states in, no crash and honest failure out.
- **Lessons as tests** — every scenario file replays in CI via the operator log: no lesson may go silent, hit a solver failure, or break the `--json` contract, and the pre-warmed cache must cover them. (This test immediately caught `inspect` printing prose into the JSON stream.)
- **Snapshot tests on `--json`** — the CLI's JSON output is the API contract.

---

## Data provenance, verified

The traps are all about data, not code. Checked against primary sources
2026-08-18; the conclusions changed the plan.

| Source | Terms | Verdict for us |
|---|---|---|
| PubChem (NCBI bulk) | No NCBI restrictions, commercial OK; per-annotation source attribution expected | **Primary property + GHS source.** Keep attribution per record |
| Wikidata | CC0 | Clean supplement; coverage is thin (≈2k boiling points, ≈310 pKa) — cannot carry the load |
| NASA CEA (`github.com/nasa/cea`) | **Apache-2.0** incl. `data/thermo.inp` | **Primary thermochemistry source** for L2g and formation enthalpies |
| Cantera data files (gri30.yaml etc.) | Part of Cantera's BSD-3 distribution | Redistributable mechanism data for L5. (GRI-Mech 3.0's own site imposes no restriction, but has no formal licence text — the BSD-3 Cantera copy is the clean channel) |
| CLP Annex VI via EUR-Lex | EU legislation, reuse with acknowledgment | Harmonised GHS/CLP hazard classes — take from EUR-Lex, not ECHA dumps |
| PHREEQC databases | USGS User Rights Notice (public-domain-like, attribution) | Embed (except `sit.dat` — ThermoChimie provenance, needs a terms check) |
| CAS Common Chemistry | **CC BY-NC 4.0** | Unusable commercially. Never present CAS RNs as licensed-from-CAS data; identifiers come from PubChem/Wikidata |
| NIST WebBook / JANAF-online | **NIST SRD — copyrighted**, permission required | Do not scrape or redistribute. (The 1971 NSRDS-NBS 37 JANAF tables are public domain but dated) |
| CAMEO / CRW4 database | Contributed fields explicitly non-duplicable (CAS RNs, NFPA, AEGL, ERPG) | Never ship the database; reimplement the published methodology (L0 note) |
| ECHA C&L exports | IP-encumbered (CAS data named) | Avoid; use EUR-Lex / PubChem routes |
| Burcat (Third Millennium) | Free non-commercial only | Skip, or write for permission if CEA coverage falls short |
| Open Reaction Database | **CC-BY-SA 4.0** on data; ShareAlike propagates to merged datasets | **Decided: BY-SA is acceptable.** The curated dataset is published BY-SA (code stays AGPL — separate works, separate licences); an educational commons staying open is a feature, not a cost. What we do *not* accept is a licence being quietly dropped: where an upstream relabels BY-SA data as CC BY or MIT (ORDerly's Figshare deposits; the CaCS SQLite), we honour the **original** terms and say so |
| `chemicals` (Python) | MIT code aggregating CRC/NIST/Yaws/Common Chemistry data | **Dropped as a data source** — it launders the SRD and NC problems into our binary. (Its sibling `thermo` remains a build-time *oracle*, generating test fixtures, not shipped data) |
| UNIFAC parameters | Consortium tables proprietary; original journal tables usable | Source from the original publications, provenance per parameter |
| USPTO reaction data (Lowe extraction) | **CC0 — verified via the figshare API**, patent text USPTO-confirmed copyright-free, reaction facts *Feist*-uncopyrightable | The clean chain under every USPTO-derived corpus/model; relevant only to the deferred ML tier and template tooling |
| CRD — Chemical Reaction Database (van der Lingen) | **CC BY 4.0** (Figshare, verified) — 1.37M reaction SMILES from US patents + literature (1.44M as of Jan 2026) | **The best-licensed bulk reaction corpus found** — attribution only, no ShareAlike. But SMILES only (no conditions/observations), patent-chemistry distribution, semi-automated with author-acknowledged errors. Build-time **template-mining** corpus at most; never codex content |
| ORDerly benchmarks (Figshare 23298467 / 23502372) | Deposits say **CC BY 4.0** — but the data is extracted from **ORD (CC-BY-SA-4.0)**, and neither the deposits nor the JCIM paper mention ShareAlike anywhere | ⚠️ **Decision required before any use.** The data itself is attractive (≈919k/939k/691k reactions with solvents, agents, **temperature**, time, yield). If ever used: treat it as **BY-SA regardless of the depositors' claim**, attribute both ORDerly and ORD — or stay out. Note `procedure_details` is verbatim patent prose, not pedagogy. Do **not** run their toolchain (dormant, CI red since 2024-06) |

---

## Nine to expert, one simulation

Never dumb down the model, only the view. One PHREEQC result, rendered at
whatever register the reader is in. The child and the postdoc see the same numbers.

| Level | Output |
|---|---|
| **lv1** | "It went cloudy! A white solid appeared — that's a *precipitate*." |
| **lv2** | `AgNO₃ + NaCl → AgCl↓ + NaNO₃` · 0.010 mol · Ksp = 1.77 × 10⁻¹⁰ |
| **lv3** | SI(AgCl) = +2.41 · I = 0.021 m · γ(Ag⁺) = 0.857 · full selected-output |

Levels are numbers, not audiences: naming them "age 9" or "child" bakes in
an assumption about who a level is for, and the levels will multiply (a
step between equations and full numerics, say). `Register` is therefore a
`u8` with named constants, unspecified levels inherit the nearest one
below, and the codex keys its copy by `lv1`/`lv2`/`lv3`/… so adding
granularity is a data change rather than a schema change.

Registers are a presentation concern and live entirely in the UI (and the
CLI's renderer). The solver has no idea who is asking. Register copy is
generated by deterministic templates over solver output ("SI > 0 and new solid
phase → 'went cloudy'"), never by a language model — offline, reproducible,
trustworthy. The same registers apply to orbitals: age 9 gets "the electron
clouds have to match up like puzzle pieces"; the expert gets the molden file.

### The alchemical layer earns its keep

The twelve classical operations map almost directly onto our operator list, so
the naming system *is* the difficulty ladder rather than decoration:

| Child | Modern | The Work |
|---|---|---|
| Heat it up | Thermal decomposition | Calcination |
| Let it settle | Precipitation | Coagulation |
| Boil it off | Fractional distillation | Distillation |

The four stages of the magnum opus — nigredo, albedo, citrinitas, rubedo — are
cuts through the codex concept graph, realised as lesson-file tiers.

---

## What this will not do

Worth writing down before starting, because each is where an ambitious version
quietly fails.

- **Predict arbitrary organic reactions.** Genuinely unsolved. Curate, and be
  visibly honest where we are predicting rather than knowing.
- **Mechanisms and transition states at runtime.** Quantum chemistry is
  build-time; the runtime ceiling is honestly-labelled Hückel.
- **Extremes.** Plasmas, exotic organometallics, solid-state band structure,
  high pressure. (L2g's CEA data does extend T range honestly for gases and
  simple condensed phases; database validity ranges are surfaced, not hidden.)
- **Biochemistry.** A different stack; a later module, not an extension.

A general-purpose engine that computes any reaction from first principles is also
a synthesis oracle for things we do not want it computing. Curated-first gives us
an explicit, auditable boundary — a product-safety property, much easier to
defend than a filter bolted onto a general predictor.

Optional later module, cheap and on-theme: radioactive decay chains (Bateman
equations — trivial math, public-domain nuclide data, half-lives are a
curriculum staple, and decay is the most necromantic chemistry there is).

---

## The v1.0 cut

Eight phases and thirteen crates have no floor without an explicit line.
**v1.0 in a store is:**

- P0 + P1 + P2 — the bench, the safety veto, and aqueous chemistry
  (acid–base, precipitation, titration, solubility, buffers, brines)
- a **~40-reaction codex slice** with full register copy, concept links and
  observations — curated to one curriculum's inorganic aqueous block, not
  breadth-first
- the **curated-colour sliver of L6** (colour word + sRGB per species,
  concentration-driven opacity): the age-9 register *is* observations, so
  v1.0 cannot ship without "it went cloudy" — but it needs no spectra
- **one UI** over the same `--json` contract the CLI already snapshot-tests
- all three registers — they are the product's identity, not a feature
- **the particle view**, added 2026-08-19: a submicroscopic renderer driven
  by computed speciation. Without it the product has macroscopic, symbolic
  and deeper-symbolic views and Johnstone's triangle is not closed — which
  is the one pedagogical claim the whole design rests on. It may be humble
  (2-D dots at solved ratios) but it may not be absent
- **the browser as a real bench, not a lesson player.** Schools reach this
  on locked-down devices with no installs, and today the web build serves
  aqueous chemistry from pre-warmed cache only — so "a lab: try things"
  silently degrades to "replay what we prepared" in exactly the channel
  that matters most. The Emscripten IPhreeqc module already builds and
  passes the AgCl gate; what is missing is the JS bridge in
  `kerotakis-wasm`. This is worth more than any solver work below it

Explicitly **not** in v1.0: P2g (v1.1, with `ignite`), P3p (v1.1, with
distillation), the QM/orbital layer, Hückel, lessons beyond the codex slice,
and everything ML. Each later phase extends the same bench; nothing in v1.0 is
scaffolding to be thrown away.

## Build order

Genuinely sequential — each phase is shippable on its own, each depends on the
state model the previous one hardened, and **from P1 on, the CLI is each
phase's acceptance demo**.

### P0 — Feasibility spike

The single highest-information task. Everything else is downstream of it.

- [x] Build IPhreeqc natively (macOS) and drive it through
      `LoadDatabaseString` / `RunString` from Rust FFI with no filesystem
      (`kerotakis-phreeqc`, vendored submodule + cmake build.rs; 4/4 tests)
- [x] Build IPhreeqc with **Emscripten** (`tools/build-iphreeqc-wasm.sh`,
      1.2 MB wasm module; needs `-fexceptions` at compile *and* link, plus
      `-sSTACK_SIZE=8MB` — found the hard way). wasi-sdk single-module attempt
      remains a time-boxed stretch, never on the critical path
- [x] Embed all four databases (`include_bytes!` — pitzer.dat has Latin-1
      comment bytes, so not `include_str!`)
- [x] One end-to-end case: AgNO₃ + NaCl → SI(cerargyrite) = 0 at equilibrium,
      0.0099848 mol AgCl precipitated — identical result native (Rust FFI) and
      wasm (Node, `tools/test-iphreeqc-wasm.mjs`), both in CI
- [x] Engine quirk, documented in the wrapper: **loading a database resets the
      selected-output string flag** — `SetSelectedOutputStringOn` must be
      called after `LoadDatabaseString` (and is re-asserted before every run)
- [ ] Fuzz it: random inputs → no crash, honest failure state (basic
      malformed-input test in place; real fuzzing pending)
- [x] **Gate passed 2026-08-19:** the offline premise holds. Web ✓ (Emscripten
      wasm, AgCl case green in Node, CI-enforced) · mobile ✓
      (`cargo build -p kerotakis-phreeqc --target aarch64-apple-ios` clean) ·
      native ✓ (macOS, 4/4 tests)

### P1 — Bench state machine + energy balance + L0 + CLI

- [x] `Bench`/`Vessel`, mutating + measuring operators (thermometer, balance,
      pH meter), operator log, solver stack (physics → chemistry → honesty),
      re-equilibration between steps
- [x] Enthalpy bookkeeping: thermal mixing on add/decant; curated dissolution
      enthalpies feed ΔT (NaOH warms the beaker ~+10.6 K, NaCl cools
      slightly — computed, tested). Solver↔T iteration still single-pass
- [x] `kerotakis-cli`: REPL + batch (`kero run x.lab`) + `--json` with a
      contract test pinning the JSON shape
- [x] L0 wired before any chemistry, on the *prospective* state, for `add`
      **and** `decant` (`kerotakis-safety`), with graded verdicts:
      Allow / **Warn-then-show** (the pedagogical default — warning always
      precedes the chemistry, tested) / Veto (reserved, product-safety
      boundary). Seed matrix: bleach×ammonia, bleach×acid — both with
      curated outcomes that actually run (chloramine/chlorine gas evolves,
      mass measurably leaves the open vessel, NaOH byproduct turns the
      solution basic)
- [ ] Grow the seed matrix to the full published NOAA group set with
      SMARTS-driven group assignment (needs Indigo; legal sourcing per the
      L0 note)
- [x] Conservation-invariant property tests from the first operator (256-case
      proptest: mass and energy)

### P2 — PHREEQC, shippable on its own

- [x] `kerotakis-phreeqc` FFI surface (string-in/value-out, ~15 functions)
- [x] `PhreeqcEquilibrator`: vessel → element totals + amount-limited
      `EQUILIBRIUM_PHASES` → back to inventory. Precipitation (AgCl marquee),
      **computed solubility limits** (8 mol NaCl/kgw leaves ~1.9 mol solid),
      acid–base via charge balance (0.001 m HCl → pH 3.0), **titration to
      equivalence** (pH 1.75 → 6.99 → 11.53), 200-seed fuzz with element
      conservation
- [x] Weak acids and buffers from the database's own equilibria: 0.1 m
      acetic acid pH 2.88, equimolar acetate buffer at pKa (4.63), buffer
      absorbs 0.01 mol HCl with Δ0.08 while plain water crashes to pH 2.0,
      half-neutralisation midpoint reads pKa — Henderson–Hasselbalch without
      ever writing it down
- [x] **Database routing by validity domain** (found the hard way: minteq.v4
      gives halite solubility 3.7 instead of ~6.1 mol/kgw — its activity
      model fails in brines): wateq4f for inorganic problems, minteq.v4 only
      when organics (acetate) are involved; pitzer.dat is the eventual right
      tool for real brines
- [x] Content-addressed result cache (keyed by database + canonical input,
      which is a deterministic function of species set, amounts and T);
      identical replays served bit-identically with zero engine calls
- [x] Carbonate chemistry with an **open vessel**: escaping gas phases
      (CO2(g) equilibrium phase pinned at atmospheric pCO2, one-way) —
      vinegar + baking soda fizzes ~77% of its carbonate out, plain
      bicarbonate degasses modestly and drifts basic (thermodynamic truth;
      bubble-vs-seep is L5 kinetics), the balance sees the mass leave, and
      the H2O co-product of HCO3- + H+ → CO2↑ + H2O keeps the ledger
      chemical. NaHCO3 endothermic dissolution cools the beaker
      (lessons/fizz.lab)
- [x] True speciation in the expert register: SolutionInfo carries the
      full equilibrium distribution (molality, activity → γ) parsed from
      the engine's own report — γ(Ag⁺)=0.78 at I=0.1 m, the AgCl(aq)
      neutral complex, dissolved O₂/N₂ from redox; deduplicated, cached
      with the result
- [x] Phosphate (minteq routing — wateq4f lacks free H3PO4): 0.1 m
      phosphoric acid pH 1.6, titration reads the *conditional* pKa2
      (~6.65 at I≈0.25, γ(HPO4²⁻)≈0.4) — textbook-constant vs pH-meter
      reality, an expert lesson in itself
- [x] pitzer.dat routing for concentrated major-ion brines: halite
      saturates at the textbook 6.13 mol/kgw (wateq4f: 6.50, minteq: 3.7 —
      three databases, three validity domains, routed honestly)
- [x] Hard-water chemistry (K, Ca, Mg, sulfate; Calcite/Gypsum/Sylvite
      phases with per-database availability): chalk barely dissolves and
      fizzes away in acid, hard water deposits limescale, gypsum's two
      waters of crystallisation move between liquid and crystal in the
      ledger (exact), CaCl2 is a +20 K hot pack and KCl a −4 K cold pack.
      Fixed en route: phase-delta baseline is the phase's input amount, not
      vessel solids (freely-soluble solids double-counted their heat)
- [x] **Derivation over tables**: the equilibrator's hand-maintained
      mapping tables are gone. `dbindex` parses the embedded databases
      (master species, phase dissolution equations → stoichiometry, hydrate
      waters, log K, element coverage); `derived` computes each registry
      species' aqueous role from its *formula* by oxyanion-group
      decomposition, matches mineral phases by composition, picks the
      stable polymorph by lowest log K, and derives routing capability and
      per-database phase availability. All 18 suites pass unchanged — the
      derivation reproduces the tables' chemistry exactly. What stays
      curated is documented and small: ~6 oxyanion groups, 3 booking
      overrides (protonation state at teaching pH), atmospheric partial
      pressures, and the safety layer
- [ ] Registry breadth continues with L1 (PubChem/Wikidata export)
- [x] Cache pre-warming: `kero prewarm lessons/*.lab -o cache.postcard`
      replays every lesson through the real engine and exports the results
      (9 lessons, 73 steps → 26 unique solver results, 20 KB). A cold
      engine that imports it serves the same lessons bit-identically with
      zero engine calls — tested end to end through the postcard
      round-trip. The `.lab` grammar is now one shared parser across REPL,
      batch runner and pre-warmer
- [ ] The P2 CLI **is** the "strong product on its own" claim, tested literally

### P2g — Heat and fire

- [x] NASA-9 thermochemistry from CEA's `thermo.inp` (Apache-2.0, vendored
      with NASA's own LICENSE/NOTICE): Cp(T), H(T), S(T), G(T), formation
      enthalpies, composition, phase — CO₂ reproduces ΔHf = −393.51 kJ/mol,
      Cp(298) = 37.13, S° = 213.8 from the coefficients. Species citations
      kept as provenance. (Found en route: CEA writes element symbols
      upper-case, "CA" — normalised, or every two-letter element breaks.)
- [x] **Gibbs minimiser** (Gordon & McBride formulation, Lagrange
      multipliers per element, damped Newton on ln n, condensed phases
      admitted/dropped by chemical potential): methane burns to CO₂ + H₂O
      with elements conserved to 1e-6; products visibly dissociate to
      CO/OH/H at 3000 K; **CH₄/air adiabatic flame temperature ≈ 2225 K**
      by bisection on enthalpy; chalk stable at 800 K and fully calcined by
      1400 K, with the **decomposition temperature computed** (~1170 K
      textbook window) rather than assumed
- [x] Two design notes worth keeping: convergence is measured on each
      species' *contribution*, not its log step (a trace radical doubling
      every iteration must not hold the mixture hostage — RP-1311 eq 3.14);
      and an element with no gaseous carrier (calcium) must enter through a
      condensed phase from the start or its balance row is all zeros. A
      vessel's **atmosphere is part of the problem**: with CO₂ as the only
      possible gas, calcite below its decomposition point has no gas phase
      at all — true, and numerically degenerate. Real beakers contain air
- [x] **L2g wired into the bench** (`ThermalEquilibrator`): heating chalk
      calcines it (quicklime left, CO₂ gone), burning magnesium consumes
      the ribbon into brilliant-white oxide and releases its enormous
      exotherm into the vessel temperature. Registry species map to CEA
      species **by composition** — nothing lists the pairs.
- [x] Three design decisions the wiring forced, all now explicit:
      **the atmosphere is a reservoir, not inventory** (a vessel stands
      open in air: oxygen is available without being weighed in, product
      gases leave — the same one-way exchange as the aqueous fizz);
      **the species pool is exactly what the registry can name**, so the
      minimiser can never reach for an exotic carbide we would have to drop
      (mass loss) or show without a story — widening what the lab can
      discover is a deliberate act of naming; and a **kinetic threshold**
      (500 K) below which the solver stands down, because equilibrium would
      otherwise oxidise every metal on the bench and only kinetics explains
      why the world is not like that. That gap is L5's, and it is stated
      rather than hidden.
- [x] `ignite` operator: a spark that brings the vessel to flame
      temperature and lets the solvers decide. Magnesium burns with a
      blinding white light, is consumed into brilliant-white oxide,
      reaches ~3040 K, and **gains mass** (1 g → 2 g) — the result that
      surprises every student, because the oxygen came from the air.
      A spark held to salt leaves *no trace*: the vessel goes back to
      where it was, and instead of burning it gives the **flame test**
      (sodium's bright yellow) — chemically right and better pedagogy
      than either burning it or saying nothing happened.
- [x] **Observation has a detection limit.** Bookkeeping stays exact, but
      user-visible events need `OBSERVABLE_MOLES` (1e-6): equilibrium put
      1.7 nanomoles of chlorine over the salt and the lab announced a
      poisonous gas cloud. Instruments have detection limits and so does
      this one; reporting a nanomole as a cloud is a lie of scale.
- [x] Adiabatic vessels solve **enthalpy-conserving**, not ΔH ÷ Cp: a gram
      of burning magnesium heats the air around it, not just the speck of
      oxide it leaves behind (the naive version hit the 6000 K clamp)
- [ ] Validate a wider set against build-time Cantera oracle runs

### P3 — Reordered by curriculum weight, 2026-08-19

The original P3 was VLE/UNIFAC. That is now **demoted below P3e and P3k**,
on a straightforward value-per-effort argument: distillation and azeotropes
are a sliver of school chemistry, while redox and rates are enormous
blocks, and both are *already most of the way there* underneath. Phase
behaviour returns when the school-facing layers are covered.

#### P3s — States, freezing and boiling  ← **first, it is a correctness bug**

- [ ] The bench happily reports **liquid water at −7.95 °C with a pH**. It
      has no model of state at all: `Phase` is assigned when matter is
      added and never reconsidered, so cooling a beaker past its freezing
      point changes nothing but the number on the thermometer.
- [ ] Melting/boiling points per species from the registry; the solvent
      state re-evaluated whenever temperature changes.
- [ ] **Colligative properties fall straight out** and are core curriculum:
      freezing-point depression and boiling-point elevation from the
      computed ionic strength — salt on icy roads, why seawater freezes
      below 0 °C. PHREEQC gives us the osmotic coefficient already.
- [ ] Honest boundary: a frozen or boiling vessel is a state the aqueous
      solver does not model, and must say so rather than keep answering.

#### P3e — Redox and electrochemistry  ← the biggest missing curriculum block

- [ ] **Surface pe/Eh, which PHREEQC already computes and we discard.** The
      model audit found no pe, no Eh, no saturation index reaching the
      user. Exposure first, new physics second.
- [ ] Standard-potential ordering → the activity series, displacement
      reactions, why zinc protects iron.
- [ ] Nernst equation over computed activities — an equation whose inputs
      the engine already has.
- [ ] Faraday's law for electrolysis: charge → moles → mass at an
      electrode.
- [ ] Corrosion and batteries as the contexts.

#### P3k — Rates, the cheap sliver

- [ ] Rate law + Arrhenius parameters as **codex data**, integrated by a
      bench clock. This buys the thiosulfate disappearing-cross, catalyst
      and temperature effects, and concentration dependence — the whole
      school treatment of kinetics — without a mechanism engine.
- [ ] Same reasoning as the L6 decision: ~95% of the pedagogy at ~5% of the
      cost, with P5's real mechanism chemistry still ahead of it.
- [ ] Honest flag: a clock makes **time a state dimension**, which is a
      state-machine change rather than a data addition, and the one part of
      this that is not cheap.
- [ ] It also fills a hole the codex already admits: the `collision-theory`
      model entry currently `embodied_by` nothing, and the 500 K thermal
      stand-down is a placeholder for exactly this.

#### P3p — Phase behaviour (was P3)

- [ ] `feos` integration (SAFT family + flash); `vle-thermo` for cubics +
      classical activity models; `seuif97` for water
- [ ] Own UNIFAC (~300 lines) against original-literature parameter tables with
      per-parameter provenance
- [ ] Golden fixtures generated by build-time `thermo` (Python oracle)
- [ ] Acceptance: ethanol–water azeotrope at 95.6%

### Curriculum sources — verified 2026-08-19

Four parallel surveys, every licence checked against the issuing body's own
page. The headline is that **the obvious sources moved against us this
year**, and the workable path is statutory rather than licensed.

**Relicensed to NonCommercial in 2026 — no longer usable:** OpenStax
Chemistry 2e and Atoms First 2e (March 2026; the pre-relicense commit
`51f80f1a` stays CC BY 4.0 *irrevocably* and is the escape hatch), PhET
(2026-03-29, 119/119 sims; pre-cutoff pinned builds remain CC BY, and
PhET's *teacher activities* are still CC BY 4.0), OpenSciEd High School —
which is where every chemistry unit lives.

**Never usable:** CK-12 (proprietary, revocable, claims rights in *your*
derivatives), MIT OCW, Khan Academy, ChemCollective (NC **and** ND),
LibreTexts except per-page tag-filtered, New Zealand's entire Ministry
estate (BY-NC), NSW NESA (all rights reserved), BC/Ontario/Alberta,
Singapore (and Cambridge co-owns the syllabuses), Ireland (no licence at
all), Switzerland's Lehrplan 21 (**not** CC — "keinerlei Rechte
übertragen"; note the constraint is *legal, not technical*: a maintained
JSON API and a ~20 MB XML export exist, but both sit behind a signed
Nutzungsvereinbarung with the D-EDK, and the only credential-free routes
are per-competence PDFs. The cantonal adoptions are arguably amtliche
Erlasse under Art. 5 URG, which is the only path worth pursuing),
European Schools, DDC (NC+ND, and dewey.info is dead).

**NGSS is worse than NC:** not NC, but the grant is *enumerated* to states,
districts, schools, teachers and non-profits — a commercial app is not on
the list, no CC licence exists, and ownership is genuinely unclear (site
says NAP, book says Achieve 2013, Achieve has wound down).

**Genuinely usable, ranked:**

| Source | Basis | Why |
|---|---|---|
| **Norway (Udir Grep API)** | `åndsverkloven §14` — public domain | Best in Europe: læreplaner *are* forskrifter, stable codes, explicit year bands, open unauthenticated API |
| **Austria (RIS OGD API)** | `UrhG §7 freie Werke` | Lehrpläne are Verordnungen in BGBl. II; **the cleanest German-language curriculum anywhere**, and no §5(2)-style Änderungsverbot. Documents: Mittelschule `Gesetzesnummer 20007850`/Anlage 1 (idF BGBl. II 178/2025), AHS `10008568`/Anlage A (idF BGBl. II 204/2024, Unterstufe and Oberstufe share one Anlage). API gotchas: `Titel` wants the plural official Kurztitel, `Fassung.FassungVom` rejects the literal `"Heute"`, and `GeltendeFassung.wxe` pages time out — use the API. First year-mapping fact banked: **Chemie is Schulstufe 8 only** (not 7–8), with a wirtschaftskundliches-Realgymnasium exception; Oberstufe runs Kompetenzmodul 5–8 |
| **Sweden (Skolverket API)** | `URL 1960:729 §9` | Live, unauthenticated; kursplaner are författningar |
| **England (DfE)** | Open Government Licence v3.0 | Richest chemistry content of any open source |
| **Australia (ACARA v9)** | CC BY 4.0 | Excellent RDF/JSON-LD/SPARQL — but §5 mandates a fixed citation *including* an app-specific offline-cache clause, and teacher-support resources are carved out NC |
| **Scotland** | OGL v3.0 | Take the Benchmarks, not just the E&Os |
| **France (Éduscol)** | **Licence Ouverte 2.0** | Switched *from* CC BY-NC around Dec 2025 — snapshot the terms page, it has moved once. Take from éduscol, **not** education.gouv.fr, which is still NC |
| **Netherlands** | `Auteurswet art. 11` | Take the examenprogramma from wetten.overheid.nl (no conditions), not Examenblad (ND) |
| **Serlo / ZUM** | CC BY-SA | German OER; **filter per item** — Serlo's licence registry includes CC BY-ND and six state-ministry exam licences |

**Germany: Bayern first, and never the KMK.**

Verified across five states. **Bayern is the only one that grants commercial
use in writing**: the ISB's Nutzungsbedingungen state *"Die Texte der
Lehrpläne unterliegen nicht dem Urheberrechtsschutz"* and expressly carve
the Lehrplan Originaltexte **out** of the site's NC clause (which governs
"alle weiteren Inhalte" — Servicematerialien, images). It also has the best
machine-readability: stable URLs (`/fachlehrplan/gymnasium/9/chemie/ch`)
and a working unauthenticated PDF export. Attribute the ISB; take text
only, never the figures.

Then **Baden-Württemberg** (published as Amtsblatt K.u.U. Ausgabe C; its
only copyright note is inside the PDF and restricts reproducing *"des
Satzes beziehungsweise der Satzordnung"* — the typesetting, not the text,
so extracting competency statements as data and re-rendering them is
outside it; best semantic IDs of the five, and verified: there is **no**
XML/ZIP export, six `requestMode` values all fall back to PDF).
**NRW and Niedersachsen** rest on the §5 argument alone — the underlying
facts are verified (NRW: RdErl. 23.06.2019 in Amtsblatt 07-08/19 under §29
SchulG), but neither state says so in writing. **Berlin-Brandenburg** is
the only state with a real CC licence (Sek II RLP is **CC BY-ND 4.0** — not
NC, so commercial use is fine, but ND plausibly forbids restructuring into
an app data model, which makes it *less* useful than Bayern's
public-domain status); its RLP 1–10 carries no licence at all.

🚫 **The KMK Bildungsstandards are the one hard NC blocker** — their terms
prohibit *"Einspeicherung, Verarbeitung bzw. Wiedergabe von Inhalten in
Datenbanken"*, which is precisely what this app does. Source every
competency statement from a **state** plan, never from a KMK document.

And don't wait for open data: **GovData holds zero curriculum datasets**,
and no Bundesland uses Datenlizenz Deutschland (all sixteen checked).

Operationally: `WebFetch` is domain-blocked for lehrplanplus.bayern.de,
schulentwicklung.nrw.de and bildungsplaene-bw.de, but `curl` works; and
NRW's site has moved to lehrplannavigator.nrw.de.

**Go to the Länder, not the KMK.**

The decisive finding is not legal but structural: **the KMK Bildungsstandards
contain no year-by-year sequencing at all.** In the 2024 Chemie MSA document
"Jahrgangsstufe" appears twice, both in the historical introduction; the
content is organised by Kompetenzbereiche and Basiskonzepte, and sequencing
is explicitly delegated downward ("landesspezifische Ergänzungen und
Präzisierungen können vorgenommen werden"). **The year axis exists only in
the 16 Länder Lehrpläne** — which is also where the §5(1) case is
*stronger*, since e.g. NRW's Kernlehrpläne are set by Runderlass under §29
SchulG and published in the Amtsblatt.

Legally: §5(1) UrhG puts Verordnungen and Erlasse outside copyright
entirely; §5(2) covers other official works but drags in an Änderungsverbot
that is functionally ND, and the courts read Abs. 2 *narrowly*. Crucially
**§5 draws no commercial/non-commercial distinction** — NC is simply not a
feature of it. kmk.org's own Impressum does carry an NC clause that names
database ingestion specifically, but it is self-limiting (it reserves only
what the UrhG does not already permit) and an Impressum cannot re-copyright
a §5 work.

Either way the *facts* are free: "Säure-Base-Reaktionen are taught in
Jahrgangsstufe 9" is a fact, not an authored expression, and *Football
Dataco* (C-604/10) confirms a constraint-dictated sequence attracts no
copyright. Two constraints shape how we take them:

- **§4 UrhG** protects a distinctive *Auswahl oder Anordnung* even when
  every entry is reworded. So we normalise into **our own** taxonomy —
  which the CC0 spine already does — rather than mirroring any document's
  architecture.
- **§87a Datenbankherstellerrecht** protects investment in *obtaining*
  data, not in creating it (the spin-off doctrine), so a ministry authoring
  its own curriculum has a weak claim — but §87b catches "wiederholte und
  systematische" extraction, which means **harvest from official ministry
  sites, never from aggregators**.

Caveat carried openly: **no court decision or commentary addresses Lehrpläne
under §5**, and the widely-repeated claim that curricula may be used
commercially traces to no primary source. This needs a German IP lawyer's
sign-off before launch, not a confident reading of statute.

**The topic spine, and it is CC0.** `oehTopics.ttl` from
`openeduhub/oeh-metadata-vocabs` — the WirLernenOnline curriculum-topic
taxonomy, CC0 1.0 verified, German, hierarchical, with definitions.
Extracted by `tools/extract-oeh-topics.py` into `codex/concepts.toml`:
**189 chemistry topics**. It carries no year mapping, which is the honest
division of labour — topics from a CC0 vocabulary, years from Norway and
Austria, and the mapping between them is ours.

Rejected as spines after checking: EuroVoc (clean licence, ~30 chemistry
concepts, three orders of magnitude too coarse), DBpedia (real granularity
but 3.5-year-stale dumps, self-contradicting licence metadata, and depth-3
is mostly Nobel laureates and trade associations), UDC Summary (CC BY-SA
but inorganic has 2 subclasses and organic 1), ChEBI and Wikidata (entity
ontologies, not teaching topics), IEEE LOM (paywalled).

### P4 — Codex + curated reaction library + appearance

- [x] Codex schema and content: TOML (a chemistry editor must be able to
      write it without a build step). **66 reaction entries** across
      `inorganic`, `aqueous` and `quantitative` — dissolving and
      saturation, precipitation and the common-ion effect, strong, weak and
      polyprotic acids, buffers and titration, salt hydrolysis, the fizz,
      hot and cold packs, limescale and hardness, separations, calcination,
      combustion and the flame tests — resting on **113 concepts** with
      prerequisite edges between them. Every entry carries register copy at
      all three levels and its own provenance (source, licence, and what
      computed the numbers), and most carry a prediction whose wrong
      answers are the mistakes learners actually make.
- [x] **27 model entries** (`codex/models.toml`) in eight supersession
      chains: particle → Dalton → Kern-Hülle → shell → charge-cloud →
      orbital; Arrhenius → Brønsted → Lewis; ionic/covalent/metallic →
      bond triangle. Every one names what it *lets you predict* and where
      it *breaks* — Bohr fails at helium, Lewis predicts a diamagnetic O₂
      that a magnet contradicts, the photoelectron spectrum of methane
      shows two valence ionisations rather than four equivalent bonds. Lint
      rejects a model with an empty `fails_at`, because a model shown
      without its boundary is shown as truth.
- [x] **Curriculum spine wired in**: `codex/concepts.toml` holds the 189
      CC0 topics; entries anchor to them with `spine = [...]`; lint rejects
      an anchor that is not a real topic; and **`kero codex gaps`** prints
      what the spine says a chemistry curriculum contains that we do not
      teach yet, grouped by area. Currently **33 of 189 covered** — that is
      the extension work list, and it comes from somebody else's published
      taxonomy rather than from our imagination
- [x] `kero codex lint` — **the check that makes the format worth having**:
      each entry's setup is a `.lab` script, so lint replays it through the
      real solvers and verifies the claimed observations, pH and
      temperature actually occur. Claiming a strong acid is neutral fails
      with "claims pH 6.8–7.2, computed 3.01". CI-enforced, so a curation
      error cannot merge and a solver change that breaks a lesson is caught
      at once. Also checks structure: duplicate ids, empty registers,
      missing provenance, dangling prerequisites, and entries that claim
      nothing checkable ("a story, not chemistry")
- [ ] Indigo template application over homologues; RDKit as build-time
      cross-validator; our SMARTS incompatibility rules
- [ ] Colour data: species/precipitate/flame colours, indicator ε(λ) sets;
      Beer–Lambert + CIE integration in `kerotakis-appearance`
- [ ] ORD decision (in or out) **before** the first record is ingested
- [ ] Chemistry-editorial hire

### P5 — Real mechanism kinetics

School-level rates and electrochemistry moved forward to P3k/P3e; what is
left here is the part that genuinely needs an engine.

- [ ] Cantera-YAML mechanism parser (Arrhenius + three-body + Troe covers
      GRI-Mech-class) + rate evaluator feeding diffsol
- [ ] Multi-step mechanisms, rate-determining steps, steady-state
      approximations — the university-level treatment that curated
      Arrhenius parameters cannot reach

### P6 — Build-time QM enrichment

- [ ] `tools/` pipeline batching xtb + PySCF over the codex: ΔG, Fukui
      (`--vfukui`), MOs (`--molden` / cubegen), path frames (`--path`),
      frequencies → IR spectra
- [ ] Marching cubes → quantised glTF orbital meshes (adopt `mcubes` or vendor)
- [ ] Supervised TS searches only where a barrier genuinely matters
- [ ] Output is data; no QM binary or library ships

### P7 — Lessons + runtime Hückel

- [ ] Declarative scenario format over the operator log; register narration
      hooks; nigredo → rubedo tiers as concept-graph cuts
- [ ] Every lesson replays in CI; lesson states feed the pre-warmed cache
- [ ] `kerotakis-huckel`: simple/extended Hückel for user-drawn molecules
      (own eigensolver or YAeHMOP port), always labelled approximate

### ML tier — last or never

USPTO-trained models are weakest exactly where our users are. If the tier
ever ships: **T5Chem** is the reference forward-predictor (MIT code,
**CC-BY-4.0 weights on Zenodo**, USPTO-trained on the verified-CC0 Lowe
chain, deps modernised Dec 2024) — superseding the Molecular Transformer
plan; runtime via `tract`/`candle`/`burn` (pure Rust, real wasm stories),
not `ort`. Optional download, never web-bundled, confidence always surfaced.

Evaluated and set aside (2026-08-19, licenses verified): **ReactionT5v2** —
capable and popular, but its MIT tag over ORD-trained weights is the
author's unilateral claim with the CC-BY-SA ShareAlike question simply
unaddressed (data pulled from a Drive mirror, no licence notice); not while
that stands. **mhn-react** — clean USPTO-50k template corpus, but its
LICENSE file is textually defective (mangled BSD-2 granting use but
omitting the redistribution conditions); get author confirmation before
vendoring; frozen 2023. **MolReactGen** — thesis artifact, dead 2024,
inherits mhn-react's data and its ambiguity. **The HF enzyme-interaction
model** — zero downloads, no paper/metrics/config, bare pickle checkpoint:
untouchable, and biochemistry is parked regardless.

---

## Open decisions

### UI framework

`kerotakis-core` is the invariant either way; the CLI defers the choice
harmlessly. If web is a real target → Tauri (same Rust → wasm). If mobile UI
polish outranks web → Flutter, accepting a thinner web story. The codex markup
convention must **not** wait on this decision (P4).

### Registry/codex storage

SQLite (via `rusqlite`/`sqlite-wasm-rs`, wasm-proven) if it wants real queries;
`postcard`/`rkyv` + `include_bytes!` if read-only lookup. Decide when L1 is
built; both are wasm-clean.

---

## Governance

- **Licence:** AGPL-3.0-or-later, with an App Store / Google Play additional
  permission for binaries published by the copyright holders. See `LICENSE`
  and `NOTICE`.
- **The §7 trap, closed:** under GPLv3/AGPLv3 §7 only copyright holders can
  grant additional permissions. `CONTRIBUTING.md` therefore requires, from the
  first PR, that all contributions are licensed **AGPL-3.0-or-later + the
  store exception** (inbound = outbound including the exception — the
  Nextcloud model; Signal's CLA is the heavier alternative if needed later).
- **Data licences are tracked separately from code** — per-source provenance
  files in `kerotakis-data` and `tools/`, reproduced in the app's about screen.

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
- [ ] Claim `kerotakis` on crates.io, npm, PyPI (the CLI publish does this with
      substance behind it)
- [ ] File classes 9 / 41 / 42 through an attorney nearer launch, once the
      goods-and-services wording is settled. What was done is a screen, not a
      clearance opinion.

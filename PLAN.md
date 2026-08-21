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
| **Reaktoro** (LGPL-2.1, verified 2026-08-19 via repo metadata) | **Differential oracle for the supported parts of L2** — the modern PHREEQC-class geochemical solver (Leal, ETH), which loads our exact PHREEQC databases natively: same pitzer.dat, independent solver. Diff a corpus against `PhreeqcEquilibrator`; every disagreement is our bug, its bug, or genuinely interesting chemistry. **Boundary found in AQ-006:** Reaktoro 2.13 does not implement PHREEQC surface complexation (the upstream request remains open), so it cannot validate HFO adsorption. Build-time only, never linked; persist only approved facts or aggregate metrics, never an unreviewed fixture export. |
| ChemPy (BSD-2-Clause, verified 2026-08-19) | Cheap second opinion for textbook-level aqueous fixtures where a full geochemical solver is overkill |
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
| **L3e** | Electrochemistry | Standard-potential ordering + Nernst over PHREEQC's activities, own module (`kerotakis-core/src/displacement.rs`, **built** for displacement and the activity series); Faraday's law for electrolysis still open | ours |
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
wrongly. The comparison should also say how much the datasets even
share: only 21 of 672 mineral phases exist in all three, so three
answers are partly answers about *different admissible solids*, not
three opinions about one activity model — a sentence the `explain`
rendering now carries, fed by `derived::phase_coverage()`, whose numbers
are pinned by a test so that a vendored-database bump forces the
sentence to be re-read rather than silently reprinted. This is the honesty rule taken to its conclusion: not "here is the
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

Every cell is a rendering of one solved state. The submicroscopic row was
the missing vertex and is now **built** (`kero particles`,
`kerotakis-core/src/particles.rs`): the engine already computes the census
(`Ag⁺ 9.56e-6 · AgCl(aq) 3.21e-7`), so drawing dots at solved ratios is
honest in a way a textbook diagram is not — the picture is the answer, not
an artist's impression of it.

Two things a particle picture can lie about, both handled explicitly:

- **Scale.** You cannot draw 10²³ of anything, so each glyph stands for an
  amount and the amount is printed. Water is drawn sparsely on purpose,
  because a picture that is 99.9 % solvent teaches nothing about the
  solute — and the renderer says that is what it has done.
- **Omission.** A species too dilute to earn a glyph is *named* rather than
  dropped. A picture that silently leaves out AgCl(aq) teaches that the
  neutral complex is not there, which is the same silent-filter defect
  found three times elsewhere in this engine.

It also distinguishes a census drawn from solved speciation from one drawn
off the raw inventory, and says which it is: without a characterised
solution, ion pairs and complexes are unresolved and the picture is coarser
than it looks.

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
each wrong option names the misconception it reveals *and the next move
that puts pressure on exactly that idea* — a diagnosis without a next move
is a label, not teaching. One blanket note per question cannot do this,
because a learner who picks option 2 rarely holds the same idea as one who
picks option 3, and the conceptual-change evidence is that instruction
works by eliciting the learner's own model and confronting it. The schema
carries it (`[[reaction.expect.predict.diagnosis]]`), the lint checks the
indices, and `codex lint` reports how many distractors are diagnosed — a
count this document deliberately does not carry, having watched its own
copy go stale (rates.toml is complete and is the pattern to match;
aqueous and quantitative are the work list). Misconception
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
already is. The scheduler is not ours to invent: **FSRS** is the current
state of the art, and `fsrs-rs` (BSD-3-Clause, verified 2026-08-19; the
implementation Anki ships) is pure Rust and wasm-clean — zero licence or
portability friction. Hand-rolling SM-2 would be re-solving a solved
sub-problem, which is against the thesis.

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

**What the balancer does and does not do.** It solves the linear system:
one row per element, one for charge, right-hand species negated, and the
answer is the null space. That makes it exact where the system determines
an answer, and it *refuses* where the system does not, which is the more
important half:

- **Under-determined skeletons are refused, not guessed.** `C + O₂ → CO +
  CO₂` admits two independent reactions, and so does `MnO₄⁻ + H₂O₂ + H⁺ →
  Mn²⁺ + O₂ + H₂O` — permanganate can take its oxygen from the peroxide or
  from itself. Resolving those needs oxidation-state bookkeeping, which is
  chemistry the linear system does not contain.
- **It balances a skeleton; it cannot complete one.** `MnO₄⁻ + Fe²⁺ → Mn²⁺
  + Fe³⁺` has nowhere to put the oxygen, and we report that rather than
  inventing the H⁺ and H₂O a half-reaction method would add. Given them, it
  is exact: `MnO₄⁻ + 5 Fe²⁺ + 8 H⁺ → Mn²⁺ + 5 Fe³⁺ + 4 H₂O`.
- **Notation limits, all deliberate.** Element symbols are validated
  against the periodic table, so `A + B → C` is not a formula rather than
  an unbalanced one. Structural formulas (`CH₃-CH₂-OH`), SMILES, isotopes,
  free electrons and organic placeholders (`R`, `Et`) are not formulas
  either. Terms separate on a *spaced* plus, because a bare one is the
  charge in `Ag+`.
- **One notation conflict, resolved and documented.** `Ca2+` and `MnO4-`
  are the same shape with different meanings. Digits before a trailing sign
  are read as a subscript, which makes every oxyanion right at the cost of
  `Ca2+`; write `Ca+2`, `Ca²⁺` or `Ca++`. The other convention was tried
  first and silently broke permanganate.

**They remain valuable as build-time oracles**, and that use is clean:
running a program over public data does not make the output a derivative
work, which is the same reasoning behind the Python `thermo` fixtures in
P3p. Two jobs worth doing:

- **Differential testing.** Run ChemicalFun/ThermoFun over a corpus at
  build time and diff against our Gibbs minimiser and PHREEQC, checked in
  as fixtures. The conservation bug found on 2026-08-19 would have been
  caught in a day rather than by accident. For the aqueous layer
  specifically, **Reaktoro** is the stronger oracle (see the build-time
  table): it speciates from the same PHREEQC databases with an
  independent solver, so a diff isolates *our* code from the data.
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

### `equation` and `summary` are different claims

An entry says what happens in one of two ways, and the schema now keeps
them apart.

`equation` is a **claim about chemistry** and is enforced: it must parse and
it must conserve atoms *and* charge. Putting prose there is an error that
names `summary` as the fix.

`summary` is for entries whose point is not a reaction — a yield
calculation, a measurement, a physical change, or a deliberate null result
where nothing happens at all. It is never parsed.

This was forced by evidence rather than taste. With one field doing both
jobs, 27 of 66 entries held prose in a field documented as "balanced
equation", and no checker can tell a deliberate summary from an equation
someone got wrong. Splitting them turned an ambiguous silence into two
explicit statements, and the count is now a fact rather than an apology:
**51 balanced equations, 19 entries that describe something else.**

One temptation worth naming: an entry whose point is that *nothing reacts*
can be given an identity equation that balances trivially. That inflates
the number and teaches nothing, so those keep a summary instead.

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
- **Added gas now dissolves (resolved in AQ-003).** `add v1 CO2` is an
  amount-limited dose through an external boundary: limewater goes milky,
  continued excess CO₂ clears it again, and the event ledger separates what
  entered the condensed inventory from what vented.
- **Partial freezing is not modelled.** A freezing solution really gives ice
  plus an ever more concentrated brine down to a eutectic; we freeze the
  solvent and say the rest is not modelled.
- **`evaporate` is externally powered, and this is a decision rather than an
  oversight.** Boiling off 5.5 mol of water costs about 223 kJ — vastly more
  than any chemistry in the beaker — and the operator charges nothing for
  it. That is deliberate: `evaporate` means *you put it on a hotplate*, and
  the energy comes from outside the ledger, exactly as the heat for it would
  in a real dish. Modelling the cost without modelling the burner would be
  worse than not modelling either, because a beaker that froze itself while
  evaporating is further from the truth than one that simply does not say.

  The consequence has to be stated plainly, because it is not obvious: **the
  thermometer after an `evaporate` is not a claim.** What it shows is only
  the chemistry that happened in the beaker — crystallisation heat, mostly —
  divided by a heat capacity that shrank as the water left. That is why
  brine boiled to 99% reads 65 °C: 0.0936 mol of salt crystallising into a
  gram of remaining water, with nothing to carry the heat away. The ionic
  strength and the solid amount from that beaker are trustworthy; its
  temperature is arithmetic about a system whose largest energy term has
  been deliberately left out. No codex entry quotes a temperature after
  evaporating, and none should.

  The costed version wants a heat source as a first-class thing — a hotplate
  with a power and a duration — at which point evaporation becomes a *rate*
  rather than a fraction, and the operator changes shape. That is a real
  feature and not a patch, and it is not scheduled.

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

### Curation is verifiable, so drafting can be assisted

The no-language-model rule governs the *runtime*: narration is
deterministic templates, and that stands untouched. Authoring is a
different regime, and this project is the rare place where model-drafted
content is safe to use — because it is **mechanically verifiable before a
human reads it**. `codex lint` replays every numeric claim through the
real solvers, equations must balance atoms and charge, spine anchors must
resolve, and a model entry with an empty `fails_at` is refused. A draft
that survives all of that is wrong in at most the ways a human draft is
wrong.

That matters because the two largest open work lists are editorial and
exactly draft-then-verify shaped: **most distractors carry no diagnosis**
(`codex lint` prints the live count) and **most spine topics are
uncovered** (`kero codex gaps` prints them). Budgeting those as pure
hand-curation prices the moat wrong — the moat is the verification
machinery and the editorial judgement, not the typing.

The pipeline, honestly bounded:

- A model drafts entries, distractor diagnoses, register copy,
  translations. The lint is the first reader; the chemistry editor is the
  second and remains the gate. Nothing merges on a model's say-so.
- Model-assisted entries **say so in their provenance** — the same rule
  as `Editorial judgement (Kerotakis)`. A model's confidence is not a
  citation: misconception distractors still cite Taber, Barke, Driver or
  the AAAS bank, or are marked editorial.
- `kero serve --mcp` exposes the bench as an **MCP server** over the same
  `--json` contract the CLI already snapshot-tests, so a drafting agent
  can *run its own claims* — execute the entry's `.lab` setup, compare
  the computed numbers against its prose, iterate — before a human sees
  the draft. The CLI is text-native by construction; this is a thin
  wrapper, not a new surface.

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

**Queued by the 2026-08-19 review** (adopted when their phase arrives;
licences verified via repository metadata that day):

- `fsrs-rs` (BSD-3-Clause) — spaced-retrieval scheduler; **ships**, so its
  licence matters, and it is clean. See the retrieval-practice note in the
  pedagogy section.
- `fluent-rs` (Apache-2.0) — register narration templates; **ships**. See
  the registers section.
- Dev-tools that never ship, so their licences never propagate: `insta`
  (snapshot-test the `--json` contract with a review workflow, replacing
  hand-pinned shape tests), `cargo-mutants` (mutation testing — which
  invariants are load-bearing and which decorative), `cargo-fuzz`,
  `cargo-deny`, `cargo-about`, `release-plz`, `taplo`.

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
  Still open from P0, and the plan is now concrete: `cargo-fuzz` with
  `arbitrary`-derived structured inputs over the `.lab` grammar, the
  Unicode formula parser in `stoich.rs` (subscripts, hydrate dots, two
  charge notations — a classic fuzz target), `dbindex`, and the
  vessel → PHREEQC round trip. Nightly CI job; apply to **OSS-Fuzz**
  (free continuous fuzzing, Rust supported — acceptance is not guaranteed
  for a young project, but the application costs an afternoon).
- **Metamorphic invariants** — properties conservation cannot see, each a
  router/cache bug detector: order-independence (`add A; add B` ≡
  `add B; add A` at equilibrium), dilution monotonicity (adding water
  moves pH toward 7), scale invariance (double everything → intensive
  properties unchanged), and — the thesis as a test — all three registers
  and all three representations render from **one** solved state with no
  re-solving. The kinetics work found four bugs the conservation proptest
  could not see; metamorphic relations hunt that class systematically
  instead of by accident. **First three built 2026-08-19**
  (`kerotakis-cli/tests/metamorphic.rs`, driven through the real binary and
  the `--json` contract), and the order-independence test earned its place
  on its first run: the same reagents added in a different order settled
  ~8e-5 apart in pH — and pulling that thread took five rounds of
  measurement across two sessions before the root cause fell out, every
  intermediate mechanism plausible and wrong (quantisation cancellation,
  falsified by a non-dyadic ×1.7 scaling holding at 2.5e-11; uniform
  solute rescaling, falsified by K+ carrying none of the excess; print
  precision, worth ~1e-7 against a 1.4e-2 symptom, which struck
  `GetSelectedOutputValue` from the work list; carrying moles forward,
  worth 2.8e-10). The real chain: dissolution enthalpy rode on the
  Dissolved event, no event was recorded for a phase the routed database
  cannot name — and that is not a corner: **651 of 672 mineral phases
  exist in only some of the three databases** (Sylvite is pitzer-only;
  the figures were themselves corrected once, when the word "mineral"
  in the rendered sentence exposed 24 gas phases hiding in the count) —
  so KCl cooled the beaker on one path and not the other, the two orders
  ended 0.82 K apart, and dpH/dT ≈ −0.0163/K made a temperature bug
  masquerade as a composition mystery. Enthalpy had stopped being a
  state function; fixed generically in 39c592e (the fix keys on absence
  from the routed database, not on any mineral). The residual left after
  that fix was then *rationalised* — "the solver's own 0.05 K convergence
  tolerance, worth ~8e-4 in pH" — and a tolerance was derived on top of
  the rationalisation, which is the sixth wrong turn and the most
  instructive: the residual was a third bug wearing an explanation small
  enough to be believed. Dissolved matter carries no heat capacity in
  this model, so `t0 + q/cp` destroyed sensible heat whenever speciation
  shrank the vessel's Cp; balancing enthalpy instead (c1d493c) took
  pure-salt order-independence to machine precision (2.3e-10 in pH) and
  the precipitating scenario to 1.9e-6 — and, en route, made Hess's law
  hold exactly, which turned a dead codex entry writable. The test's
  tiers are re-derived from post-fix measurements: element totals 1e-9,
  phase split 1e-8, pH and ionic strength 1e-5 — the pH tier deliberately
  below the smallest historical bug signal (7.9e-5), so a recurrence of
  any bug in this chronicle fails the suite.
- **Mutation testing** (`cargo-mutants`) — distinguishes load-bearing
  invariants from decorative ones, which is this project's epistemics
  applied to its own test suite.
- **Lessons as tests** — every scenario file replays in CI via the operator log: no lesson may go silent, hit a solver failure, or break the `--json` contract, and the pre-warmed cache must cover them. (This test immediately caught `inspect` printing prose into the JSON stream.)
- **Snapshot tests on `--json`** — the CLI's JSON output is the API
  contract. Migrate the hand-pinned shape tests to `insta` with
  redactions for volatile fields: `cargo insta review` makes an
  intentional contract change an audited event rather than a test edit.
- **CI must enforce what the plan claims — our own standard, applied to
  us.** Two claims currently rot: the **iOS gate** (passed 2026-08-19)
  has no CI step — add
  `cargo build -p kerotakis-phreeqc --target aarch64-apple-ios` on the
  macOS runner — and there is **no wasm size budget**, though the
  offline/mobile premise depends on one: assert a byte ceiling on the
  built module so a dependency cannot silently add megabytes. Smaller:
  install `wasm-bindgen-cli` from a prebuilt binary rather than
  `cargo install`, and let Dependabot watch the actions and the iphreeqc
  submodule.

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

### The licence discipline becomes a lint

Everything above is enforced by prose and care, which is how the codex
worked before `codex lint` existed — and the fix is the same fix:

- **`cargo-deny`** in CI: a licence allowlist (an LGPL-2.1-only crate is
  then blocked mechanically, not by someone remembering this document)
  plus RustSec advisories.
- **`cargo-about`** generates the attribution/about screen from crate
  metadata instead of a maintained page.
- **The provenance table above moves to machine-readable TOML**
  (per source: licence, terms URL, retrieval date, verdict, what it may
  touch) with a `kero provenance lint` that checks every entry in
  `Cargo.lock` and `vendor/` appears in the audit and nothing ships from
  an avoid-row source. The prose stays as commentary; the claims stop
  being able to go stale silently — the `equation`/`summary` move,
  applied to licensing.
- **SPDX headers** (REUSE) on data files; a CycloneDX SBOM then falls
  out for free.

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

The template format should be **Project Fluent** (`fluent-rs`, Apache-2.0,
verified 2026-08-19) rather than ad-hoc strings: plural, case and gender
rules live in the `.ftl` data, which is what the German line will need —
ad-hoc templates fight German grammar — and it is the flavour-is-data
commitment applied to grammar. Deterministic and offline either way;
Fluent changes where the rules live, not who writes them.

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
- **the browser as a real bench, not a lesson player** — **built and
  published**, at <https://crispstrobe.github.io/kerotakis/>. The two
  wasm halves are wired together: `Lab.setSolver()` takes a JavaScript
  function, `web/kerotakis.mjs` backs it with the Emscripten build of
  IPhreeqc, and everything above the hook is unchanged — same routing, same
  content cache, same temperature fixed point, same parsers. The web gets
  the same answers **by the same path** rather than a second implementation
  that could drift from the one the codex was linted against, and CI proves
  it by recording the desktop build's answer to a deliberately un-warmed
  question and requiring the browser to match it to 1e-6 in pH. A bench with
  no solver attached reports `canSolve() == false` and refuses rather than
  guessing, which is the honest version of what shipped before
- **the demo demonstrates the premise** — **built, 2026-08-20**: the page
  is a PWA (manifest, icon, versioned service worker precaching the
  shell, both engines and all three databases), and the headless CI test
  now *proves* the premise rather than assuming it — first load online,
  then the server is killed and the page must boot from the worker's
  cache with the engine live and the precipitate still forming. "Turn
  off wifi, it still solves" is a test assertion. The Web-Worker half is
  deliberately not done and recorded as its own piece of work: the
  solver hook is synchronous *by design* (Rust calls JavaScript and
  waits — that synchronicity is what makes the bridge possible at all),
  so moving it off the main thread means Asyncify or SharedArrayBuffer
  plumbing, not an afternoon

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
- [x] Fuzz it: four libFuzzer targets in `fuzz/` (2026-08-20) — the `.lab`
      grammar (4.4M runs clean), `dbindex` on corrupted database bytes
      (2.3M clean), arbitrary operator sequences through the bench with
      non-finite floats (3.2M clean), and the stoichiometry parser, which
      paid for the whole exercise in its first two minutes: a negation
      overflow when a balancing coefficient saturates to i64::MIN,
      reachable from `kero balance` (reported engine-side with minimised
      artifacts; regression test lands with the fix). Continuous fuzzing
      (OSS-Fuzz) remains open
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

**Settled by experiment, 2026-08-19.** The question "how do we solve
`MnO₄⁻ + Fe²⁺ → ?`" has an answer that needs no reaction rules at all:
*ask the thermodynamics*. Probing IPhreeqc directly:

```text
1 mmol MnO4- into 5 mmol Fe2+ at pH 1
  pe   = 18.345    "Adjusted to redox equilibrium"
  Mn+2 = 9.999e-4     all the manganese reduced
  Fe+3 = 4.827e-3 (+1.63e-4 as FeOH+2)   all the iron oxidised

the same, with half the oxidant
  pe   = 12.674
  Mn+2 = 5.000e-4
  Fe+3 = 2.418e-3     exactly half the iron oxidised
  Fe+2 = 2.500e-3     exactly half left
```

The 5:1 stoichiometry is not encoded anywhere — it falls out of free-energy
minimisation. And the half-oxidised case lands at pe 12.67, which is the
Fe³⁺/Fe²⁺ standard potential: **a half-titrated redox couple sits at its
own E°, exactly as a half-neutralised acid sits at its pKa.** That is one
of the best results this engine can produce, and it is computed.

Two mechanics matter, both learned the hard way:

- **Naming a valence state in `SOLUTION` decouples that element.** Entering
  `Mn(7) 1e-3` and `Fe(2) 5e-3` gives a solution where *nothing reacts*:
  PHREEQC holds each valence in its own mass balance and reports pe 4 with
  the permanganate untouched. Redox needs the elements *coupled*.
- **Adding a reagent by formula is the idiom that works**, and it is what
  the bench's `add` already means: `REACTION … KMnO4` supplies K, Mn and O
  together, and PHREEQC finds the pe satisfying the electron balance.

So the build order is:

- [x] **Surface pe and Eh**, which we computed and discarded.
- [x] **Report the redox distribution per element** — `redox — all Fe as
      Fe(II); all Mn as Mn(VII)`. PHREEQC will split an element across its
      oxidation states if you name them (`-totals Fe(2) Fe(3)`), and the
      states themselves are read from the database's master-species block
      rather than a list of ours. Iron enters the registry for it, since a
      redox couple with nothing to reduce is not a demonstration.
- [x] **Couple the elements, so an oxidant and a reductant actually react.**
      This is the substantive remainder and it is a design change, not a
      switch. Naming an oxidation state in a `SOLUTION` block *decouples*
      that element in PHREEQC: it gets its own mass balance and exchanges
      electrons with nothing. So the bench will currently show permanganate
      and iron(II) in one beaker — each state correct, their coexistence
      impossible — and it now *says so* in the routing rather than
      presenting it as an answer.

      Two routes, both real work:

      * **Reagents as `REACTION` blocks.** What the experiment proved:
        `SOLUTION` with `Fe 5e-3` (no valence) plus `REACTION … KMnO4`
        gives pe 12.674 and the exact 50/50 split. But it needs the vessel
        to remember *what was added* rather than only its element totals,
        which the bench does not currently do.
      * **Bisect pe on the electron balance.** The electrons that went in
        are fixed by what was added and its oxidation state; ask PHREEQC
        for the distribution at a trial pe, count the electrons, and
        bisect until it matches. Structurally the same move as
        `equilibrate_hp` bisecting temperature on enthalpy, and it needs
        no new state on the vessel.

      The second is the better fit for this engine and is the one that was
      built. `redox_coupling` computes Σ(oxidation state × moles) as added
      — conserved by any real redox reaction — and `solve_coupled` bisects
      pe until the distribution PHREEQC reports reproduces it. Permanganate
      into iron(II) sulfate now gives the 1:5 ratio the half-equations
      demand, as an answer rather than a rule: a fifth of an equivalent
      oxidises a fifth of the iron, and the manganese comes back as Mn(II).

      **Its boundary, which is the interesting part.** A narrowed bracket
      is not a struck balance. Past the equivalence point no pe in the
      stability field of water balances the books, because the excess
      oxidant would have to take its remaining electrons from the water or
      the chloride. PHREEQC will do exactly that if asked; the bench does
      not carry the oxygen it makes, so the ledger came out 12% short while
      the beaker reported every last manganese as Mn(II) — a colourless
      answer to the one titration whose entire point is that the excess
      stays purple. The balance is now checked rather than assumed, and an
      unbalanceable beaker is refused: the stack carries on, the elements
      are shown in the states they were added in, and the routing says so.

      **What the refusal is not.** It looked like the textbook reason
      permanganate titrations are run in sulfuric acid rather than
      hydrochloric — that the chloride gets oxidised. It is not: H₂SO₄ went
      into the registry to check, and the same beaker made up with sulfuric
      acid is refused with an identical 2.500e-3 residual. The electrons
      are owed by the *solvent*, and swapping the acid changes nothing.
      Chloride oxidation is real chemistry, but this bench does not model
      it either — Cl is not in `FAST_REDOX`, which is a curated claim about
      rates — so the HCl/H₂SO₄ distinction cannot be demonstrated here at
      all, in either direction. Worth stating because the plausible
      pedagogical story survived being written down and died on the first
      experiment that could have confirmed it.
- [x] **Nernst over computed activities; the standard-potential ordering
      (activity series), displacement, why zinc protects iron.** Built
      2026-08-20 as `kerotakis-core/src/displacement.rs`, a wrapper around
      the aqueous solver: solve → let the series move electrons over the
      activities that solve reported → solve the products again → pin the
      potential to whatever electrode is left standing.

      **The architecture question, and the experiment that decided it.**
      Two candidates were on the table: native-metal phases through
      `EQUILIBRIUM_PHASES`, or an own E° module. The first check was a
      grep of the shipped datasets for metal phases, and its first answer
      was wrong: searching for "Silver", "Copper" and bare element names
      found nothing and the design was nearly written on "no shipped
      dataset has them, only llnl.dat does". The truth, found by the
      engine session's conservation fuzz test going red the moment
      elemental silver entered the registry (silver precipitating out of
      silver nitrate with no reductant in the beaker — the phase matcher
      had paired the new solid with a database phase and PHREEQC walked
      down the redox path at whatever pe it was holding):

      ```text
      wateq4f.dat   AgMetal  Ag = Ag+  + e-    log_k -13.51
                    CuMetal  Cu = Cu+  + e-    log_k  -8.76   (the Cu⁺ couple)
                    ZnMetal  Zn = Zn+2 + 2e-   log_k  25.757
                    PbMetal, CdMetal; no Fe, no Mg
      minteq.v4, pitzer, phreeqc.dat:  no metal phase at all
      ```

      So `EQUILIBRIUM_PHASES` could carry three metals on one routing
      path, and not the magnesium ribbon the flagship reaction is made
      of, nor the iron that zinc protects; it would also need the
      electron balance in `solve_coupled` to count phase moles. The own
      module covers the whole series on every route and *states its
      model*. The database's metal phases then became the check on it
      rather than its engine: `−log_k · (RT ln10/F) / n` from wateq4f
      gives E°(Zn) = −0.7619 V and E°(Ag) = +0.7993 V against the CRC
      values the module carries, −0.7618 and +0.7996 — agreement to
      within a millivolt, pinned by a test. (llnl.dat, not shipped, gives
      the same ordering with values 40–70 mV lower on its own O₂
      convention.) Both `AgMetal`-class phases and the metals' cation
      booking are now excluded in `derived`, so a metal is inert to
      PHREEQC and only the series moves its electrons.

      **What it computes.** Couples carry E° (CRC) and ΔfH° of the ion
      (NBS); the extent is found by bisection on the cell potential, so a
      pair that sits close together stops at a real Nernst root and the
      school pairs run to the last ion (Mg/Cu²⁺ is 2.7 V apart, K ≈ 10⁹²;
      copper into silver nitrate leaves 6e-10 mol of Ag⁺, written with →
      because a learner cannot see it and ⇌ would teach a hesitation the
      beaker does not show). Heat is the difference of formation
      enthalpies, balanced as enthalpy across the inventory change rather
      than as `t0 + q/cp`. Engine-read, 100 mL of water:

      ```text
      CuSO4 0.01 + Mg 0.02    0.0100 mol Cu plated, 0.0100 mol Mg left, pH 6.56,
                              26.8 → 39.5 °C; identical either order
      CuSO4 0.05 + Zn 0.05    +26.2 K  (ΔH −218.7 kJ/mol from ΔfH°, textbook −217),
                              59.9 °C either order; T agrees to 1.7e-7 K, pH to 7e-10
      HCl 0.1 + Mg 0.02       0.02 mol H2 up, pH 0.23, +22.4 K, and NO neutralisation
                              heat — the acid the metal consumed is removed from
                              the charge ledger before the next solve sees it
      HCl 0.1 + Cu 0.02       "inert", with the reason (E° +0.342 V above 0.000 V)
      AgNO3 0.02 + Cu 0.05    0.02 mol Ag plated, the solution turns blue, +3.5 K
      AgNO3 0.01 + CuSO4 0.01 + Zn 0.005   all the silver, none of the copper
      ```

      **Its boundary, stated in the beaker.** Three different silences
      were distinguished because conflating them is the silent-filter
      fault in a new coat: copper in dilute acid is `Inert` — a computed
      result about copper, with the potentials; magnesium in brine is
      `NotYetModeled` — nothing to displace, and its slow reaction with
      water itself is a *rate* this lab does not compute; a metal plated
      out this step says nothing further. Kinetics, oxidising acids,
      overpotentials and air oxidising a metal that merely stands in
      solution are not modelled, and E° is used at the vessel temperature
      without dE°/dT.

      **The potential.** A metal in contact with its own ion is an
      electrode, and the reported pe is that electrode's by Nernst, with
      the provenance saying so — and saying, where the value lies below
      water's hydrogen line (magnesium at −2.43 V), that the metal is not
      at equilibrium with the water it stands in and a voltmeter would
      read a mixed potential. It is pinned only when both members are
      present in observable amounts: at exact Zn/Cu²⁺ equivalence the
      pin keyed on a 2e-10 mol trace of copper one way round and on
      nothing the other, and the answer depended on addition order
      (+0.02 V against the open-air +0.77 V). Same rule as the titration
      endpoint: no couple left, no potential published. Open follow-up:
      the speciation itself is still solved at the open-air pe, and
      feeding the electrode's pe into the solve is the next step — which
      waits on the initial-speciation-versus-air-equilibrium question the
      engine session opened the same day (the two disagree by 2.2 pH
      units on an iron beaker, and which side is right wants an oracle).

      **The cell.** `cell v1 v2` wires two vessels as a galvanic cell and
      reads the voltmeter: E = E(cathode) − E(anode) over each vessel's
      electrode (`displacement::electrode`, the accessor the electrolysis
      layer builds on), open circuit — no current, no internal resistance,
      ideal salt bridge, and the event says so, because a learner's next
      question after "1.10 V" is how long the torch runs, and that is
      Faraday's question with a different answer. Zinc in zinc sulfate
      against copper in copper sulfate, 1 mol/kgw each: **1.104 V**, zinc
      the anode whichever side it is wired to, nothing in either beaker
      changed. Copper against silver: 0.483 V against an E° of 0.458 —
      above it, because at 1 mol/kgw most copper is paired with sulfate
      and the free-ion activity is what Nernst sees. Diluting the copper
      side tenfold costs 17.6 mV, not the ideal 29.6, for the same reason;
      the test asserts that it is *less* than ideal, since exactly 29.6 mV
      would mean concentrations had reached the equation instead of
      activities. A copper strip in brine is refused as a half-cell with
      the reason (`NoCell`), not as a modelling gap.

      **The hydrogen overpotential** (proposed by the engine session when
      the user asked why real batteries exist; built 2026-08-20). Hydrogen
      has to *form* on the metal, and that costs an overpotential the
      thermodynamics knows nothing about — platinum ~0.02 V, iron 0.40,
      copper 0.60, zinc 0.72, lead 0.88 at a bench-scale current density.
      It enters `displace` as a gate on one pair only, the H⁺/H₂ couple:
      driving force = E_H(pH) − E(metal), Nernst as the beaker stands, and
      if it is under the barrier the reaction is refused *kinetically*, in
      a different sentence from copper's thermodynamic refusal, because a
      learner needs to know which. Under 0.1 V of margin it runs and the
      bench says the rate is not computed. Curated like `FAST_REDOX`, for
      the same reason: a claim about rates. Its limit: overpotential is
      current-density dependent and the bench has none, so one number per
      metal can say "blocked on the timescale of a lesson" and "marginal",
      never "four hours". The table is *uncited and says so*: the values
      are electrochemistry-text folklore that neither session verified
      against a primary table, they spread 0.1–0.2 V between compilations,
      and magnesium's is an estimate (it corrodes too fast to hold a
      Tafel line). Nothing computed is sensitive to 0.1 V except lead,
      where the margin is sevenfold. A worse-looking provenance line than
      a citation, and the true one.

      Checked before building that it changes no outcome for the metals
      already on the bench, which is the point — the model earns its
      place by predicting the observed five and getting the margins
      right. Engine-read, 0.02 mol metal in 100 mL:

      ```text
      HCl 0.1 (pH −0.04)      Zn  reacts, margin 0.04 V  (marginal, said)
                              Fe  reacts, margin 0.05 V  (marginal, said)
      CH3COOH 0.1 (pH 2.38)   Zn  driving 0.62 V < 0.72: kinetically blocked
                              Fe  driving 0.31 V < 0.40: kinetically blocked
                              Mg  reacts; pH 2.38 → 4.27
      ```

      Zinc in vinegar really is an overnight job and zinc and iron really
      do fizz far less eagerly than magnesium; both were previously
      reported as plain reactions. Where it becomes decisive is lead:
      Pb²⁺/Pb at −0.126 V against 0.88 V of overpotential is blocked by a
      factor of seven, which is why a lead-acid accumulator can sit in a
      car full of sulfuric acid for years — and now computed: with lead
      in the registry (83ec2fb), `Pb` in molar hydrochloric acid is
      refused kinetically with the 0.88 V barrier named, while zinc
      plates lead out of lead nitrate without any such trouble, because
      no gas has to form for that.
- [ ] Faraday's law for electrolysis: charge → moles → mass at an electrode.

**Oxidation-state bookkeeping is the explanation layer, not the solver.**
It does not find the products — the free-energy minimisation does — but it
turns a computed distribution into the sentence a learner needs: manganese
fell from +7 to +2, five electrons each; iron rose from +2 to +3. It also
supplies half-reaction display and the electron count. Where the numbers
come from:

1. **Derived from the databases first.** PHREEQC's master-species block
   names oxidation states outright — `Mn(7) MnO4-`, `Fe(+3) Fe+3`,
   `N(-3) NH4+`, `C(-4) CH4` — so for aqueous species the state is *data
   with provenance*, not our opinion. `dbindex` already parses this block.
2. **Rules where the database is silent**, as a constraint solve rather
   than a lookup: F = −1, group 1 = +1, group 2 = +2, H = +1, O = −2, and
   the sum over a species equals its charge. Fix what is known, solve for
   the rest. One unknown element is then determined — possibly as a
   *fraction*, which is honest: Fe₃O₄ really is +8/3 on average, and an
   average is exactly what electron counting needs.
3. **Inconsistency is the detector, not a special case.** Applying O = −2
   to H₂O₂ gives a sum of −2 against a neutral species, and that failure
   *is* the signal that the oxygen is peroxidic. Metal hydrides announce
   themselves the same way. So the exceptions are found rather than listed.
4. **Refuse when two elements remain unknown** and no database entry
   settles it — the same discipline as the balancer refusing an
   under-determined skeleton.

What oxidation states will **not** do, stated so it is not promised: they
do not resolve a skeleton with two products of the same element in
different states. `C + O₂ → CO + CO₂` stays ambiguous however carefully the
electrons are counted, because both products are oxidations and nothing in
the bookkeeping picks a ratio. That needs either a stated ratio or, better,
the same treatment as above — let the thermodynamics decide.

#### P3k — Rates, the cheap sliver  ← **built**

- [x] `kerotakis-core/src/kinetics.rs`: rate = k·Π[Xᵢ]^nᵢ with k from
      Arrhenius, orders and activation energies curated with provenance,
      integrated by a bench clock. `wait 20s` in the `.lab` grammar.
- [x] **Time is a state dimension**, and shared: `wait` advances *every*
      vessel, because time is not something one beaker has more of than
      another. That is the only way a fair test means anything, and the
      disappearing-cross lesson is exactly two beakers, one variable and
      one clock.
- [x] Catalysis is modelled as **a lower activation energy**, not a factor
      on the rate — which is what a catalyst is. Manganese dioxide and
      catalase sit side by side on the same reaction so the enzyme's
      advantage is a computed consequence.
- [x] The ten-degree rule *falls out* rather than being applied: an
      activation energy near 50 kJ/mol gives a factor of about two per ten
      degrees at room temperature, and a steeper barrier visibly breaks the
      rule of thumb.
- [x] Codex entries for the practicals: `codex/rates.toml`, 14 entries —
      order by initial rates, the temperature series *and where the
      ten-degree rule frays* (×1.90, ×1.80, ×1.69 over successive steps),
      the three-beaker catalyst comparison, constant half-life, the fair
      test, and four entries that exist to state a limit rather than a
      result.
- [x] **pe and Eh surfaced** (P3e's first step): reported only when the
      beaker holds a redox couple the user put there, never when it is
      PHREEQC's default dressed as a measurement. Known limitation written
      into the source — the test is necessary but not sufficient, and the
      engine's own "Adjusted to redox equilibrium" marker turned out to fire
      on the water couple in plain brine, so it is parsed and cached against
      a better understanding rather than trusted now.

**Integration accuracy is checked against a closed form, not against
another guess.** A first-order decay has an exact solution, so the
integrator is compared with real arithmetic. That test earned its place
immediately: it caught a 0.7% drift over two minutes that nothing else
could see, which forced the switch from explicit Euler to the midpoint
method. `tools/kinetics-oracle.py` is the second opinion for cases with no
closed form — SciPy, out of the build graph, on the ChemicalFun terms.

Four bugs that only a rate model could have exposed, all fixed:

- **Neither curated rate law conserved mass**, and the conservation
  proptest could not see it because it never issues a `wait`. Peroxide
  destroyed the water its own equation string promised; thiosulfate
  destroyed Na₂O, 62 g/mol per extent, visible on the balance. A rate law
  is a reaction and has to balance like one — there is now a test that
  audits every entry in the registry, and a second that checks the declared
  equation string against the modelled stoichiometry, because when those
  two disagree it is usually the code that is lying.


- **One unknown species disabled the whole aqueous engine.** `partition`
  returned `None` if *any* species lacked a derived role, so adding
  thiosulfate — which is in no database we ship — silently withdrew the pH
  of the acid sitting beside it. The vessel simply stopped having a
  solution.
- **And then the solver deleted it.** The aqueous rebuild replaces the
  vessel's contents with the computed state, so a species with no role was
  not ignored but *destroyed*. It had been invisible only because the
  first bug meant the solver never ran on such a vessel at all.
- **The integrator could freeze.** The midpoint method evaluates on a copy;
  once a half-step pushed the copy's last reactant below the threshold at
  which `withdraw` discards a spent portion, the midpoint rate came back
  zero and the full step applied nothing — two million substeps to advance
  the clock by seven milliseconds.

**Prose numbers are now checked too, advisorily.** `codex lint` caught
nothing when the peroxide rate constant was recalibrated, while five
entries went on quoting half-lives and extents the engine no longer
produced: ranges were verified, sentences were not, and a sentence is what
the learner reads. The lint now pulls unit-carrying numbers back *out* of
the register text — handling `4.06 × 10⁻⁷` as well as `4.06e-7`, since the
first is what the codex actually uses — and asks whether the replay
produced anything like them, at the precision the author wrote. "Near
pH 1.6" is a correct report of a computed 1.64 and is not flagged; a
percentage tolerance would have called it stale.

It is **advisory and stays advisory**, because a good entry legitimately
quotes numbers this replay did not produce: an activation energy from the
literature, a stoichiometric coefficient, a textbook figure held up for
contrast (bicarbonate's famous pH 8.3 exists in the codex precisely to be
contradicted), or another entry's result quoted for comparison. Making
those errors would train authors to strip real content out of their
writing. Currently **43 flagged across 80 entries**, most of them of
exactly those kinds.

What it is genuinely good at is the job it was built for: after changing a
curated constant, the *new* entries in that list are the sentences that
went stale.

**Honest gaps.** A salt the aqueous engine cannot speciate now dissolves
rather than sitting at the bottom of the beaker (`dissolves_without_
speciation`), and the lab says exactly what that means: it contributes
nothing to pH or ionic strength. The thiosulfate reaction treats acid as a
rate influence read from the computed pH rather than as a consumed
reactant, because the practical runs with acid in large excess and the
vessel has no proton portion to draw down.

It also fills a hole the codex already admitted: `collision-theory`
`embodied_by` nothing, and the 500 K thermal stand-down is a placeholder
for exactly this.

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
      write it without a build step). Entries span `inorganic`, `aqueous`,
      `quantitative`, `rates`, `redox` and `states` — dissolving and
      saturation, precipitation and the common-ion effect, strong, weak and
      polyprotic acids, buffers and titration, salt hydrolysis, the fizz,
      hot and cold packs, limescale and hardness, separations, calcination,
      combustion and the flame tests, redox titration, autoprotolysis,
      freezing-point depression and Hess's law — resting on a concept
      graph with prerequisite edges. Every entry carries register copy at
      all three levels and its own provenance (source, licence, and what
      computed the numbers), and every prediction's wrong answers carry
      diagnoses. Live counts belong to `codex lint` and README (whose
      copies are pinned by tests); this document stopped carrying them
      after watching its own copies go stale.
- [x] **Model entries** (`codex/models.toml`) in eight supersession
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
      teach yet, grouped by area. The covered count is `gaps`'s to print
      and README's to quote (pinned by test) — the remainder is the
      extension work list, and it comes from somebody else's published
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
- [ ] **TOML schema served through `taplo`** (MIT, verified 2026-08-19):
      "a chemistry editor must write it without a build step" gets teeth —
      a JSON Schema in the taplo language server puts red squiggles in the
      editor *before* lint runs
- [x] `kero serve --mcp` — the bench as an MCP server over the `--json`
      contract, so drafting agents run their own claims before a human
      reads them (see "Curation is verifiable, so drafting can be
      assisted"). Built 2026-08-19: JSON-RPC over stdio; bench tools emit
      the contract through the *same* builders as the CLI's `--json` mode,
      and the integration test requires the two answers to be identical,
      not merely close; `codex_lint` spawns the same `kero codex lint` the
      CI runs
- [ ] Indigo template application over homologues; RDKit as build-time
      cross-validator; our SMARTS incompatibility rules
- [ ] Colour data: species/precipitate/flame colours, indicator ε(λ) sets;
      Beer–Lambert + CIE integration in `kerotakis-appearance`
- [ ] ORD decision (in or out) **before** the first record is ingested
- [ ] Chemistry-editorial hire

### P5 — Real mechanism kinetics

School-level rates and electrochemistry moved forward to P3k/P3e; what is
left here is the part that genuinely needs an engine.

**Completed session — `codex-kin` (2026-08-21).** This session delivered the
first generic-kinetics slice: a reaction-network IR, both current rate laws
compiled through it without changing lesson or JSON output, and
element/charge/site/electron conservation lint. Its implementation worktree is
`/Users/christianstrobele/code/kerotakis-codex-kin` on branch
`codex-kin/reaction-network`; its code boundary is
`crates/kerotakis-core/src/kinetics.rs` plus new kinetics-focused modules and
tests. The slice also executes reversible, consecutive, and competing reactions
with atomic, availability-scaled coupled extents. It did not modify the PHREEQC
BASIC runtime, its adapter, vendored sources, or compatibility corpus. CI run
`32481885344` passed native Ubuntu/macOS, Wasm, browser, and combined-solver
checks. Stiff integration, mechanism-file parsing, and external mechanism data
remain later, separately reviewed work.

**Completed session — `codex-kin` (2026-08-21, KIN-004/005).** This session
audited and added the approved implicit-solver dependency, then replaced the
explicit midpoint loop with adaptive BDF integration over reaction extents,
including positivity protection, step rejection/retry, depletion events,
diagnostics, propagated solver errors, and exact-solution tests. Work is isolated
in `/Users/christianstrobele/code/kerotakis-codex-kin-integrator` on branch
`codex-kin/adaptive-integrator`. It may modify kinetics modules, focused tests,
and dependency/audit metadata. It will not modify equilibrium coupling,
mechanism parsing or data, the PHREEQC BASIC replacement, vendored sources, or
compatibility fixtures.

KIN-004 audit checkpoint: `diffsol = =0.16.2` is MIT and is selected with
`default-features = false` plus `nalgebra`. Upstream 0.16.2 still enables its
pure-Rust `faer` implementation on the internal linear/nonlinear crates; that
resolved graph is accepted. `diffsl`, LLVM/Cranelift JIT, CUDA, SuiteSparse,
SUNDIALS, bindgen, and native C compilation are absent from the runtime graph.
The Wasm CI gate remains part of KIN-004 acceptance.
The resolved matrix graph also reaches `getrandom` through `rand`; the Wasm
target therefore selects its supported `wasm_js` backend explicitly. This is a
target adapter, not solver randomness, and native targets remain unchanged.
CI run `32484407871` passed strict lint, all native tests and claims on Ubuntu
and macOS, core and full-bench Wasm, browser, and combined-solver checks.

**Completed session — `codex-kin` (2026-08-21, KIN-006).** This session
delivered the first mechanism-file front-end slice: strict parsing and
validation of portable Cantera-YAML species composition plus elementary
Arrhenius reactions, lowering them into the existing reaction-network IR, and a
CLI inspection path with machine-readable output. Work is isolated on branch
`codex-kin/mechanism-yaml` in a fresh worktree. Its code boundary is new
kinetics-mechanism modules, focused CLI wiring/tests, and narrowly required
pure-Rust parsing dependencies. It will not modify equilibrium or surface
coupling, vessel state, VLE, the PHREEQC BASIC replacement, vendored sources,
or compatibility fixtures. Three-body and falloff/Troe evaluation remain a
separate follow-on slice after this schema and diagnostic boundary is proven.
KIN-006 dependency checkpoint: `serde_yaml_ng = =0.10.0` is MIT and resolves to
the MIT `unsafe-libyaml` Rust translation (no system libyaml link); `bumpalo`
`=3.20.3` is MIT OR Apache-2.0 with no default features. The arena gives runtime-owned
mechanisms a borrowed IR without leaking allocations. Native and Wasm CI gates
passed in run `32489657046`, including strict lint and all tests/claims on
Ubuntu and macOS, core and full-bench Wasm, browser, and combined-solver checks.

**Completed session — `codex-kin` (2026-08-21, KIN-007).** This session added
third-body concentrations and species efficiencies, Lindemann/Troe falloff
parsing and exact rate evaluation, and gas-network execution through the
implicit integrator using finite headspace volume. Mechanism inspection now
reports each rate model and normalized low-pressure prefactor. Exact tests cover
third-body efficiencies, closed-form Lindemann/Troe rates, schema failures,
finite-headspace advancement with pressure refresh, and the CLI JSON contract.
Native, WebAssembly, browser, and combined-solver CI gates passed in run
`32492646325`. Pressure-log interpolation and external mechanism data remain
future work.

**Completed session — `codex-kin` (2026-08-21, KIN-008).** This session added
CLI-first runtime gas-mechanism simulation: validated mechanism loading, an
explicit finite sealed headspace, temperature, duration, repeatable species
feeds, and implicit reaction-network advancement. Stable JSON reports complete
initial/final mechanism composition, initial/final pressure, reaction extents,
and solver diagnostics; human output exposes the same run. Exact CLI tests cover
the analytic first-order solution, pressure increase, JSON fields, and refusal
of undeclared feed species. All native, WebAssembly, browser, and combined
solver CI gates passed in run `32494325781`.

**Completed session — `codex-kin` (2026-08-21, KIN-009).** This session added
bounded sampled gas-mechanism trajectories to the CLI. The stable JSON contract
preserves all KIN-008 endpoint fields and adds the initial state plus exact
evenly spaced requested times, composition and pressure at every point,
cumulative reaction extents, and aggregate implicit-solver diagnostics. Exact
tests compare every sampled point with the analytic first-order solution, prove
monotonic depletion and pressure growth, and reject zero intervals. All native,
WebAssembly, browser, and combined-solver CI gates passed in run `32495335994`.

**Completed session — `codex-kin` (2026-08-21, KIN-010).** This session added
strict one- and two-region NASA7 thermochemistry, per-species reference-pressure
parsing, ideal-gas concentration equilibrium constants, and elementary
reversible detailed-balance execution in both direct and implicit evaluators.
Product-side reverse orders and shared thermochemistry validity ranges are
enforced. CLI inspection/simulation plus exact standard-state equilibrium,
direct-rate, equilibrium-convergence, out-of-range, and schema tests are
included. All native, WebAssembly, browser, and combined-solver CI gates passed
in run `32496871088`. Pressure-dependent reverse reactions remain future work.

**Completed session — `codex-kin` (2026-08-21, KIN-011).** This session added
instantaneous mechanism-rate diagnostics for university-level multi-step
analysis: per-reaction forward/reverse/net progress, net species production,
pressure, and an explicitly defined instantaneous rate-determining candidate in
stable human and JSON output. Reversible equilibrium reports equal directional
fluxes with no false limiting candidate; multistep tests prove stoichiometric
production accounting and step selection. All native, WebAssembly, browser, and
combined-solver CI gates passed in run `32498166044`.

**Completed session — `codex-kin` (2026-08-21, KIN-012).** This session added
pressure-dependent Arrhenius gas kinetics with pressure-unit parsing, strict
pressure-grid validation, same-pressure rate summation, logarithmic pressure
interpolation, and nearest-endpoint extrapolation shared by direct and implicit
evaluation. CLI inspection exposes normalized pressure points; CLI rates and
simulation tests exercise the same evaluator. Closed-form, analytic-decay, and
invalid-grid tests are included. All native, WebAssembly, browser, combined
solver, and BASIC transition CI gates passed in run `32499420214`.

**Completed concurrent session — `codex-AQ` (2026-08-21).** This session owned
AQ-004 in the shared main checkout and did not modify `kinetics.rs`,
reaction-network modules, or KIN-001–003 tests. Boundary-aware headspace energy
accounting and open/sealed/pressure-controlled checks reached DoD: core and
strict-lint checks are green, and hosted Ubuntu/macOS native, IPhreeQC, Wasm
runtime, browser, and combined solver checks passed in CI run `32481206425`.

**Completed concurrent session — `codex-AQ` (2026-08-21, AQ-005).** This
session added typed finite-capacity HFO surface interfaces, strong/weak site
ownership, zinc/sulfate occupancy and ligand-exchange water ledgers, PHREEQC
`SURFACE` compilation/readback, explicit refusal for untracked sorbates, and
focused conservation/re-equilibration/live-engine tests. It did not modify
kinetics modules, dependency metadata, the BASIC runtime, vendored sources, or
VLE work. Hosted Ubuntu/macOS native, IPhreeqc, Wasm runtime, browser, and
combined-solver gates passed in CI run `32491444035`.

**Completed session — `codex-AQ` (2026-08-21, AQ-006).** Work was isolated in
`/private/var/folders/53/8b_q74j10mv9xq84_j44tm1w0000gn/T/kerotakis-aq006.TwawJ86yJn.worktree`
on branch `codex-aq/aq-006-oracle`. It owned the pH-dependent HFO adsorption
benchmark, its PHREEQC-facing comparison test, and a development-only oracle
runner. Reaktoro remains an external `oracle-only` tool: it must not enter any
crate, app bundle, Wasm artifact, vendored directory, or required CI path. The
repository may persist only reviewed scalar benchmark facts and aggregate error
metrics with tool version, input/database identity, retrieval date, and an
explicit distributability decision. This session did not modify kinetics,
the BASIC runtime/vendor, VLE, exchange sites, or runtime dependency metadata.

AQ-006 checkpoint: the live engine now exercises three ordered acid-side
points while checking zinc conservation and finite site capacity. Reaktoro's
documented PHREEQC database loader was investigated, but its still-open surface
complexation gap makes it incapable of this benchmark. `tools/surface-oracle.py`
therefore independently solves the intrinsic acid/base, zinc, sulfate and
finite-site mass-action balances from the approved USGS `wateq4f.dat`
constants. It explicitly omits diffuse-layer electrostatics and full aqueous
side speciation. The live test can export per-case values only when given an
explicit path and revision/date environment variables; that ephemeral file is
fed to the tool, whose output contains aggregate errors and monotonic verdicts
only. The tool and its four stdlib unit checks do not enter any crate, app,
Wasm artifact or required CI path. Hosted audit run `32494550891` reviewed
three cases and recorded only the approved aggregate result: strict monotonic
agreement, mean absolute bound-fraction error 0.07210 and maximum error
0.21626. Those pass the executable limits of 0.10 mean and 0.25 maximum for
this deliberately reduced oracle. Native Ubuntu/macOS, strict lint, IPhreeqc
Wasm, core/full/combined Wasm and the real-browser demo all passed in the same
run; follow-up run `32495027486` executed and passed those now-enforced limits.
The temporary hosted audit hook was then removed; the development-only export
remains explicit and opt-in.

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
- [ ] Mechanise releases (`release-plz`, Apache-2.0, verified 2026-08-19)
      and give each codex release a **Zenodo DOI** — the CC BY-SA dataset
      becomes citable for the education-research audience we cite, and the
      DOI timestamps the licence grant
- [ ] File classes 9 / 41 / 42 through an attorney nearer launch, once the
      goods-and-services wording is settled. What was done is a screen, not a
      clearance opinion.

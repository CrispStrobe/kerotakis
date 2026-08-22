# Kerotakis — roadmap for a broader chemistry emulator

> The filename is retained from the original request. This is not primarily a
> web-frontend roadmap. It is a roadmap for the complete simulation backend,
> its scientific coverage, its data system, and the clients that deliver it.

Status: 2026-08-20. This document complements `PLAN.md`: the plan records the
project's history and detailed decisions; this roadmap asks what architectural
and scientific work creates the most *multiplicative* increase in chemistry
coverage from the system that exists now.

## Executive decision

Kerotakis should remain a federation of specialist models. There is no honest
universal solver for arbitrary chemistry, and trying to make one would weaken
the project's best property: it says what computed an answer and where that
answer stops being trustworthy.

The next large step is therefore not “add another solver to the end of the
stack.” It is:

1. make conserved material and energy the authoritative state;
2. represent phases, compartments, headspaces, surfaces, electrodes, and
   apparatus explicitly;
3. compile each user operation into a coupled problem graph;
4. let thermodynamics settle fast processes while kinetic and transport models
   advance slow ones;
5. commit one audited state transition only after every participating model has
   agreed on the shared quantities;
6. expand coverage through data and reusable reaction families rather than
   compound-pair special cases.

That substrate lets the existing engines do much more, makes additional engines
composable, and turns every curated property or reaction template into coverage
across many experiments.

## Distribution and licensing invariant

This roadmap is subordinate to the project's distribution model:

- Kerotakis code remains **AGPL-3.0-or-later**.
- Official App Store and Google Play binaries rely on the additional permission
  written under AGPLv3 Section 7.
- That permission is intended for official binaries published by the project's
  copyright holders. Contributions must carry the matching inbound grant.
- An exception granted by Kerotakis copyright holders cannot relax obligations
  attached to somebody else's code or data.
- Scientific capability is never a reason to weaken this rule.

This is an engineering policy, not a legal opinion. Ambiguous cases stop at a
review gate; they do not enter the build while somebody argues that they are
“probably fine.”

### Hard allowlist for direct inclusion

This is deliberately narrower than the set of licences that might be legally
compatible with an AGPL project in some distribution contexts. The question is
not merely “can these works coexist?” but “can the complete official payload be
distributed through the intended stores under Kerotakis' fixed terms?”

Only these families may enter an official app payload by default:

**Code**

- Kerotakis-owned AGPL code whose contributors granted the matching store
  permission;
- MIT;
- Apache-2.0;
- BSD-2-Clause or BSD-3-Clause;
- equally permissive forms such as ISC, Zlib, 0BSD, or Unlicense after the exact
  text and attribution requirements are recorded;
- verified public-domain code or government works with compatible notices.

**Data and media**

- CC0;
- CC BY, with attribution carried per record or asset;
- permissive data terms materially equivalent to those grants after review;
- verified public-domain facts/data, with source acknowledgment retained even
  where copyright does not require it;
- Kerotakis-owned material only under a contribution and distribution grant
  that expressly covers the official store path.

Everything else is denied direct inclusion by default. In particular, GPL,
LGPL, AGPL code without the matching store permission, MPL, CC BY-SA, any
NonCommercial licence, proprietary terms, field-of-use restrictions, unclear
database rights, and “free to use” without an actual grant do not enter the
official binary or its bundled assets.

“Similar” means legally similar, not merely open-looking. It requires a source
record and an explicit allowlist decision.

### The Section 7 permission is not contagious

The additional permission solves a restriction in Kerotakis' AGPL grant. It
does not:

- relicense third-party GPL/LGPL/AGPL code;
- waive a third party's anti-DRM, relinking, source, attribution, or ShareAlike
  terms;
- convert CC BY-SA data into CC BY;
- make a proprietary parameter table distributable;
- cleanse outputs copied or systematically extracted from a restricted
  database.

Consequently, a technically compatible library can still be ineligible for the
official app. A separate process or dynamic library does not automatically fix
the store-distribution problem, and WebAssembly static linking makes that
boundary even less plausible. The default is to keep copyleft scientific tools
outside the runtime.

### Artifact lanes

Every external input must be assigned one lane before it is downloaded by CI or
used by a build script.

| Lane | Distributed? | Rule |
|---|---:|---|
| `runtime-code` | Yes | Must pass the direct code allowlist, compile on every target, and appear in source/notice/SBOM output. |
| `runtime-data` | Yes | Must pass the direct data allowlist; each record keeps source, licence, attribution, and transformation history. |
| `optional-pack` | Yes | Downloading after installation is still distribution, not a loophole. The pack needs its own reviewed licence manifest and must pass the same store/channel analysis. |
| `generated-shipping-artifact` | Yes | The generator may stay outside the app, but its output and every input dataset must permit this artifact to be distributed. |
| `test-fixture` | Yes, in source | A committed golden file is distributed. Treat it like runtime data even if no production binary opens it. |
| `oracle-only` | No | The tool and restricted input stay outside the repository and release artifacts. Persist only a verdict or independently distributable factual result approved by provenance lint. |
| `development-tool` | Normally no | May run in development/CI. Vendoring, modifying, or redistributing it triggers a separate review. |
| `blocked` | No | No ingestion, scraping, fixture generation, template mining, or derived runtime data. |

### “Oracle-only” is a narrow permission

A build-time or validation oracle may answer “does our result agree?” without
entering the app. It may not become a laundering path.

- Do not vendor the executable, library, model weights, or restricted database.
- Do not copy expressive output, source records, parameter tables, or bulk
  excerpts into fixtures.
- A numerical output is not automatically safe: determine whether it is a
  protectable output, a database extraction, or subject to tool/input terms.
- Store the smallest useful result: often pass/fail, an error metric, or a few
  independently sourced benchmark numbers rather than an exported dataset.
- Record tool version, input source, command, retrieval date, and the reason the
  persisted output is distributable.
- CI must be able to run without the oracle. Oracle jobs are reproducible audit
  jobs, not hidden build dependencies required to make a release.

### A current policy mismatch must be resolved first

The repository currently says that curated data is CC BY-SA 4.0, separately
from AGPL code. The contribution grant and the store exception are written
primarily around AGPL-covered contributions. Separately licensed CC BY-SA data
does not automatically receive an AGPL Section 7 permission.

There are also two descriptions of who receives the store permission: the
additional-permission text appended to `LICENSE` is broad, while `NOTICE` says
it applies only to binaries published by the copyright holders. The intended
scope is clear from this roadmap and the user's instruction, but the operative
texts should not be left to implication.

Before adding another contributor-owned data record or shipping a store build,
obtain qualified review and make `LICENSE`, `NOTICE`, `CONTRIBUTING.md`, the
curated-data licence, and the in-app notices state one consistent result. Likely
implementation choices are:

1. keep the public curated dataset CC BY-SA, but obtain an explicit additional
   store-distribution grant for every contribution and upstream item included
   in official binaries;
2. publish only project-controlled/compatibly licensed CC BY/CC0 data inside the
   official app and keep BY-SA material out of that payload; or
3. distribute separately reviewed data packs through a channel whose terms are
   compatible with the pack licence.

The roadmap does not choose among these legal structures. It makes choosing one
the first release gate.

### Provisional source disposition

The exact source manifest will be authoritative; this table sets the intended
lane so engineering work does not outrun it.

| Source/tool | Intended lane | Constraint |
|---|---|---|
| Kerotakis Rust code | `runtime-code` | AGPL-3.0-or-later plus matching inbound store grant. |
| IPhreeQC and approved PHREEQC databases | `runtime-code` / `runtime-data` | Keep USGS notices; audit each newly added database separately. `sit.dat` remains blocked pending its ThermoChimie terms. |
| NASA CEA code/data already vendored | `runtime-data` | Apache-2.0 notices and source manifest remain with the compiled subset. |
| FeOS, DiffSol, Indigo, InChI, YAeHMOP, permissive Rust crates | candidate `runtime-code` | Accept only after exact-version code licence and separately bundled parameter/data licences pass lint. |
| Cantera code | `oracle-only` initially | BSD code could pass the allowlist, but runtime inclusion is unnecessary while Kerotakis evaluates its own compact IR. |
| Individual Cantera YAML mechanisms | candidate `runtime-data` | Audit mechanism-by-mechanism; Cantera's code licence does not prove every imported mechanism/data file is redistributable. |
| PubChem records | candidate `runtime-data` | Import only fields whose upstream annotation provenance is compatible; retain per-field attribution. |
| Wikidata | `runtime-data` | CC0; keep source identifiers for scientific traceability. |
| CLP Annex VI via EUR-Lex | review, then candidate `runtime-data` | Confirm exact reuse/acknowledgment terms fit the allowlist before ingestion. |
| Reaction-QM records | candidate `runtime-data` | CC BY; ship only selected records with required attribution and dataset version. |
| CRD | `oracle-only` / template-mining review | CC BY may permit derived artifacts, but attribution and database-extraction analysis precede committing mined templates. |
| Open Reaction Database / ORDerly | `oracle-only` | CC BY-SA does not pass the direct-inclusion rule. No merged runtime dataset or committed extracted corpus. |
| xTB and CREST | `oracle-only` | LGPL tools do not ship. Generated artifacts require an independent output/input rights review. |
| Reaktoro and GEMS3K | `oracle-only` | LGPL code does not enter official app binaries; use only for differential checks. Reaktoro 2.13 can load PHREEQC databases but does not implement PHREEQC surface complexation, so it is not an oracle for AQ-006. |
| RDKit and PySCF | `oracle-only` / `development-tool` | Even where code is permissive, runtime inclusion is unnecessary; generated artifacts still need source/input provenance. |
| Python `thermo` / `chemicals` aggregate data | restricted `oracle-only` | Never export its aggregated NIST/CRC/Yaws/CAS-derived tables or make fixtures from unclear records. |
| NIST WebBook/SRD, CAS Common Chemistry, CAMEO exports, ECHA dumps, Burcat, proprietary UNIFAC tables, unlicensed RMG database | `blocked` | No scraping, ingestion, generated fixtures, or derived shipping tables without a new explicit grant. |

## What exists now

The current backend is already unusually strong for an educational chemistry
application:

- a bench and vessel state machine with an operator log;
- conservation checks and an energy balance;
- aqueous speciation, acid/base chemistry, minerals, precipitation, redox, and
  activity models through IPhreeQC;
- gas/condensed equilibrium and adiabatic flame calculations over NASA-9 data;
- metal displacement, Nernst cell potentials, hydrogen-overpotential gates, and
  the arithmetic core of electrolysis;
- phase transitions and colligative effects for water;
- a first kinetic layer with Arrhenius rate laws and a shared bench clock;
- ideal vapour–liquid equilibrium foundations in `kerotakis-thermo`, with the
  non-ideal layer under development;
- computed colour, particles, indicators, and observation thresholds;
- 74 registered species, 103 codex reaction entries, 16 lessons, and a
  mechanically checked curriculum graph;
- native, WebAssembly, CLI, browser, and MCP delivery paths over the same core.

The limiting factor is no longer whether serious chemistry can run locally. It
is whether the state and orchestration model can combine more kinds of serious
chemistry without one model silently undoing another.

### Current ceilings

| Ceiling | Consequence |
|---|---|
| A vessel stores portions as both inventory and interpreted chemical state | Rebuilding aqueous speciation can lose or relabel material; reagent history and equilibrium identity are conflated. |
| Solvers mutate the vessel sequentially | More domains create order dependence: aqueous, thermal, phase, kinetic, and electrochemical models may each believe they own the final state. |
| A vessel has one bulk phase model and no explicit headspace or interface | Gas absorption, finite-pressure bottles, surface reactions, electrodes, extraction, and real distillation have nowhere to live. |
| Temperature is stored while enthalpy is reconstructed approximately | Phase change, reaction heat, pressure work, and externally powered apparatus become difficult to couple consistently. |
| Kinetics is a small static registry | It teaches rates well but does not yet express reaction networks, reversible elementary steps, stiff systems, or transport limitations. |
| The registry is hand-authored Rust data | Every new species is code work; property coverage cannot scale independently from releases. |
| Model validity is explained in prose but not fully machine-routable | The system cannot yet plan a multi-model solve or report a precise coverage boundary before attempting it. |
| Curriculum coverage counts topics, not scientific capability | “Covered” does not distinguish one scripted example from a reusable model that spans a whole family. |

### The highest-leverage moves

| Leverage | Why it multiplies coverage |
|---|---|
| Conserved ledger separate from resolved species | Every equilibrium, kinetic, transport, electrochemical, and nuclear model can share state without destroying another model's interpretation. |
| Explicit compartments and interfaces | One abstraction unlocks headspace chemistry, extraction, adsorption, electrodes, membranes, columns, and finite-rate phase transfer. |
| More of the already-shipped PHREEQC vocabulary | Gas, surface, exchange, solid-solution, kinetic, and 1-D transport chemistry arrive without inventing new numerical chemistry. |
| A generic reaction-network IR | Every validated mechanism becomes executable through one evaluator and one stiff integration layer. |
| Molecular graphs plus atom-mapped templates | One curated transformation covers a chemical family instead of one exact pair of registry keys. |
| Build-time mechanism reduction and property compilation | Desktop scientific tools and large datasets produce small, deterministic, offline runtime packs with quantified error. |
| Property-resolution ladder | Measured, correlated, group-contribution, and computed values can extend coverage without being presented as equally certain. |
| Instruments as models | The same hidden chemical state supports qualitative analysis, spectroscopy, calorimetry, electroanalysis, and assessment. |
| Capability manifests and explicit validity | New model packs compose safely because the router knows what each one can and cannot claim. |
| Differential oracles plus invariant tests | Coverage can grow rapidly without making correctness depend on one implementation or one dataset. |

## The target state model

The most important backend change is to separate what the user put into the
world from what a model currently says it has become.

```text
World
└── apparatus
    ├── compartments
    │   ├── conserved ledger       elements, charge, mass, energy
    │   ├── material lots          reagent identity and provenance
    │   ├── resolved phases        aqueous, organic, gas, solids
    │   └── process state          T, P, V, time, mixing regime
    ├── interfaces
    │   ├── gas ↔ liquid           dissolution, evaporation, kLa
    │   ├── liquid ↔ liquid        partitioning, extraction
    │   ├── solid ↔ liquid         dissolution, nucleation, adsorption
    │   └── electrode ↔ electrolyte charge transfer, plating, corrosion
    └── connections
        ├── material flow          pour, filter, column, condenser
        ├── heat flow              hotplate, bath, flame, insulation
        ├── electrical             wire, power supply, resistor, meter
        └── radiation              lamp spectrum and photon flux
```

### Three layers of state

**1. Conserved ledger — authoritative**

- element/isotope totals;
- net charge and electron bookkeeping where meaningful;
- total mass;
- enthalpy or internal energy, with the chosen constraint stated;
- compartment volume and mechanical constraints;
- material transferred to the environment, filter, electrode, or another
  compartment.

No solver directly rewrites this ledger. It proposes a balanced delta. The
orchestrator audits and commits it.

**2. Material lots — historical**

A lot records what was added: identity, amount, phase, temperature, source, and
possibly particle size or concentration. This is needed because kinetics and
safety depend on history even when equilibrium does not. “Iron powder” and an
iron nail have the same elements and different rates; freshly precipitated and
aged solids may have the same formula and different accessibility.

**3. Resolved chemical state — derived**

Species populations, activities, phase fractions, saturation indices,
potentials, colours, and spectra are model outputs. They may be invalidated and
recomputed without losing the ledger or the material history.

This separation fixes the current awkwardness around protons, unknown aqueous
species, metastable products, and phase relabelling. It also permits two models
to offer different resolved states over the same conserved ledger.

### First-class physical structure

Add the following concepts before broadening reaction coverage:

- `Compartment`: a homogeneous control volume with explicit constraints;
- `PhaseState`: phase kind, amount, composition, volume, and model provenance;
- `Headspace`: finite volume, pressure, gas inventory, and vent boundary;
- `Interface`: area, participating phases, transfer model, and optional sites;
- `SurfaceSites` and `ExchangeSites`: capacity and occupancy;
- `Electrode`: material, area, roughness, potential/current boundary, and
  deposited layer;
- `Environment`: ambient pressure/composition/temperature and permitted sinks;
- `Apparatus`: connections and boundary conditions, not decorative UI objects.

The geometry should stay deliberately low-dimensional: well-mixed compartments
and optional one-dimensional cells/columns. Full CFD would consume the project
without adding proportional curriculum value.

## Replace the solver stack with a model orchestrator

The existing `Equilibrator` trait remains a useful adapter during migration,
but it is too small as the permanent contract. A model should declare:

| Contract field | Purpose |
|---|---|
| capabilities | Questions it can answer: aqueous equilibrium, VLE, rate, charge transfer, observation, and so on. |
| required state | Phases, properties, geometry, and boundary conditions it needs. |
| validity domain | Temperature, pressure, concentration, elements, phase assumptions, and timescale. |
| conserved quantities | What its proposed transition promises to conserve or exchange through a named boundary. |
| timescale | Instantaneous equilibrium, resolved kinetic process, or externally imposed operation. |
| outputs | State variables, residuals, events, derivatives, and uncertainty. |
| provenance | Engine, dataset, parameter records, version, and routing reason. |
| failure semantics | Cannot express, outside validity, no convergence, or a computed null result. |

### One bench step

```text
operator
  → validate units and apparatus
  → run safety policy on the prospective material state
  → compile an active-domain problem graph
  → apply external transfers of matter, energy, charge, or photons
  → solve fast equilibria under explicit constraints
  → integrate slow kinetics and transport with adaptive time steps
  → re-equilibrate fast subdomains as required
  → audit mass, elements, charge, energy, positivity, and phase totals
  → atomically commit state + explanation + provenance
```

For time-dependent problems, use multi-rate operator splitting first rather than
attempting one monolithic nonlinear solve:

1. equilibrate the fast aqueous/phase subsystem;
2. evaluate kinetic, interfacial, and transport rates;
3. integrate a bounded step;
4. project back onto the fast-equilibrium manifold;
5. reduce the step and retry if conservation or convergence fails.

This is physically legible, debuggable, and compatible with the current
specialist engines. A fully coupled DAE is an optimization for the cases where
splitting error is demonstrated, not the starting architecture.

## Coverage roadmap

The stages below are ordered by leverage and dependency, not by UI visibility.

## R0 — Make coverage and coupling explicit

Build the substrate before adding another large domain.

### Deliverables

- Introduce the conserved ledger/material-lot/resolved-state separation.
- Add compartments, finite headspace, interfaces, environment boundaries, and
  transactional state deltas.
- Add typed quantities for power, current, potential, area, flow, photon flux,
  concentration, and amount density. Do not let bare `f64` values cross model
  boundaries.
- Replace `applies()` with a machine-readable capability and validity report.
- Make model routing produce a plan that `explain` can render before and after
  execution.
- Add rollback: a failed coupled solve must leave the pre-step state intact.
- Generate a coverage manifest from the registry, model capabilities, and
  acceptance corpus.

### Coverage should be measured on five axes

1. **identity** — which substances and phases are representable;
2. **transformations** — which reaction/process families are computable;
3. **conditions** — temperature, pressure, concentration, timescale, and
   apparatus domains;
4. **observables** — what a learner can measure or see;
5. **validation** — which outcomes have independent references or oracles.

A single curated lesson should not count as equivalent to a reusable model that
works over a parameter range.

### Exit gate

Run every current lesson through both the old adapter and new orchestrator,
obtain equivalent accepted outputs, and prove that every committed step balances
the ledger. Deliberately inject a solver failure and prove rollback.

## R1 — Unlock the rest of PHREEQC

The highest immediate scientific leverage is already vendored. The current
adapter uses only a fraction of PHREEQC's problem vocabulary. Add explicit
backend representations for these existing capabilities:

| PHREEQC capability | Kerotakis unlock | Example experiments |
|---|---|---|
| `GAS_PHASE` | Finite headspace and bidirectional gas–liquid equilibrium | CO₂ dissolving into limewater; opening a fizzy bottle; ammonia or oxygen absorption; pressure in a sealed vessel. |
| `SURFACE` | Surface complexation with finite site capacity | Adsorption onto oxides; pH-dependent surface charge; pollutant removal; why surface area matters. |
| `EXCHANGE` | Ion-exchange sites | Water softening; soil nutrient exchange; column breakthrough. |
| `SOLID_SOLUTIONS` | Non-ideal mixed solid phases | Mineral substitution and composition-dependent precipitation. |
| `KINETICS`/`RATES` | Rate-limited mineral and aqueous processes using the same thermodynamic state | Weathering, slow dissolution, carbonate formation, metastable persistence. |
| `TRANSPORT`/`ADVECTION` | One-dimensional reactive cells | Water-softener columns, contaminant fronts, diffusion plus reaction. |

Do not expose raw PHREEQC keywords as the public domain model. Compile typed
Kerotakis state into those keywords so native and attached-Wasm engines retain
the same contract.

### Specific fixes this stage absorbs

- Added gases can dissolve instead of only venting.
- Open, sealed, pressure-controlled, and gas-reservoir vessels become distinct
  boundary conditions.
- A precipitate can be metastable, age, transform, or dissolve kinetically.
- Adsorbates and exchanger occupancy remain on an interface/site ledger rather
  than masquerading as bulk solids.
- Partial freezing can reject solute into a shrinking liquid compartment and
  repeatedly re-equilibrate down to an explicit model boundary.
- Unknown dissolved material stays in the conserved ledger even if PHREEQC
  cannot speciate it.

### Exit demonstrations

- Bubble CO₂ into limewater, form carbonate, continue bubbling, and redissolve
  it under excess carbonic acid.
- Compare a sealed carbonated bottle, an opened bottle, and a continuously
  swept vessel using the same starting ledger.
- Run a hard-water stream through a finite ion-exchange column and obtain a
  breakthrough curve.
- Adsorb a dissolved ion onto a finite surface and release it by changing pH.
- Freeze salt water partially while conserving water and salt across ice and
  brine.

## R2 — Complete phase behaviour and separation apparatus

`kerotakis-thermo` should become the thermophysical layer for molecular fluids,
not just an isolated VLE calculator.

### Model ladder

1. pure-component vapour pressure and heat capacities;
2. ideal gas and ideal liquid models as explicit educational baselines;
3. activity-coefficient models for non-ideal liquids;
4. equations of state for vapour and dense-fluid non-ideality;
5. liquid–liquid and solid–liquid equilibrium where supported;
6. group-contribution estimates only when parameters and applicability are
   available, with confidence below measured/curated data.

FeOS is the strongest candidate runtime framework for equations of state and
phase equilibria because it is Rust-native and already covers more than one
model family. That is a technical recommendation, not an automatic data grant:
audit FeOS code and every parameter dataset independently before inclusion.
Keep a narrow Kerotakis trait in front of it so datasets and equations of state
remain replaceable.

### Required flashes

- TP, PH/HP, and UV flashes;
- bubble and dew points;
- vapour fraction and phase compositions;
- liquid–liquid split;
- repeated equilibrium stages;
- explicit “no supported model for this mixture” outcomes.

### Apparatus makes the chemistry reusable

- powered hotplate/burner: power and duration instead of free fractional
  evaporation;
- condenser and receiver;
- simple and fractional distillation with configurable ideal-stage count;
- reflux and reflux ratio;
- separatory funnel and repeated extraction stages;
- recrystallization through temperature-dependent solubility;
- drying and humidification;
- pressure vessel and vacuum operation within stated limits.

Equilibrium alone gives endpoints. Add small interfacial mass-transfer models
(`kLa`, evaporation area, mixing regime) so “wait” determines how fast a gas
absorbs or a volatile component leaves. The parameters must be curated or
measured; geometry must not be inferred from a substance name.

### Exit demonstrations

- Pure water and ethanol boil at the correct pressure-dependent temperatures.
- Ethanol–water shows the ideal prediction, the non-ideal azeotrope, and the
  failure of further fractional distillation.
- A two-phase extraction obeys mass balance and improves through repeated
  small washes.
- A closed heated vessel changes pressure while an open one vents.
- Recrystallization predicts both recovered crystal mass and solute left in
  the mother liquor.

## R3 — General reaction networks and stiff kinetics

The current rate-law module proves the time dimension. Generalize it into a
compiled reaction-network intermediate representation.

### Reaction-network IR

Each reaction carries:

- stoichiometric vectors keyed by canonical species or conserved components;
- reversible/irreversible direction and thermodynamic consistency information;
- rate expression and dimensional units;
- Arrhenius, pressure-dependent, third-body, and falloff parameters where
  relevant;
- catalysts and surface sites without consuming them stoichiometrically;
- phase/interface locality;
- parameter validity range, uncertainty, and provenance;
- optional photolysis cross section/quantum yield;
- observability hooks, never narration embedded in the solver.

Parse the relevant subset of Cantera YAML at build time and compile it into this
IR. Cantera remains an independent oracle; the runtime evaluates the compact
network itself. Start with elementary Arrhenius, three-body, and Troe falloff,
then add only rate forms demanded by accepted mechanisms. A mechanism enters a
shipping pack only after its own licence and upstream provenance pass the direct
data allowlist; the licence of Cantera's parser/runtime does not cover arbitrary
mechanism files.

Use a stiff integrator such as DiffSol's implicit methods. The integration layer
needs positivity guards, events, Jacobians or automatic sparsity, adaptive
steps, and equilibrium projection. Explicit midpoint remains useful for tiny,
non-stiff educational rate laws but must not be the universal integrator.

### Model heterogeneous kinetics honestly

Add interface area, particle-size distribution or effective radius, site
density, and mixing regime. Then catalyst amount and grinding can affect rates
for physical reasons. Without those fields, preserve the existing honest
refusal to invent a surface-area effect.

### Mechanism-reduction leverage

Large mechanisms should be reduced at build time against a declared envelope of
temperature, pressure, composition, and target observables. Ship the reduced
network plus an error report against the full Cantera oracle. This converts a
large desktop mechanism into a small offline educational model without silently
changing its domain.

### Exit demonstrations

- Reproduce current peroxide and thiosulfate lessons through the generic IR.
- Model a reversible reaction approaching equilibrium without crossing it.
- Show consecutive and competing reactions, including a changing product
  distribution with time and temperature.
- Simulate one validated gas mechanism in batch and plug-flow forms.
- Demonstrate that catalyst mass and particle size affect a heterogeneous rate
  only when interface data exists.

## R4 — Make electrochemistry a coupled domain

The current Nernst and Faraday work is the thermodynamic/arithmetic foundation.
The next layer requires explicit electrodes and electrical boundary conditions.

### Add in this order

1. electrode objects, area, material, deposits, and electrolyte interface;
2. equilibrium potentials from computed activities;
3. Butler–Volmer/Tafel charge-transfer kinetics with parameter provenance;
4. galvanostatic and potentiostatic operation;
5. solution resistance and simple circuit elements;
6. diffusion-limited current through a boundary layer;
7. competing electrode reactions and gas evolution;
8. passivation, corrosion, and deposit growth as stateful surface processes.

This separates four claims that are currently easy to conflate:

- thermodynamic direction;
- activation/overpotential barrier;
- current under a specified load;
- total converted matter over time.

### Exit demonstrations

- Predict electroplated mass and the voltage required to sustain the requested
  current, including current efficiency where side reactions are modelled.
- Produce a cell discharge curve rather than only open-circuit voltage.
- Show concentration polarization and recovery at open circuit.
- Reproduce galvanic corrosion and sacrificial protection with finite electrode
  inventories.
- Explain why a thermodynamically allowed hydrogen reaction can be kinetically
  blocked on one metal and active on another.

## R5 — Scale organic chemistry through structures and reaction families

Arbitrary organic reaction prediction remains outside the honest boundary. A
large, useful subset is nevertheless tractable if coverage is built from
verified families rather than named compound pairs.

### Structural foundation

- canonical molecule graph with bond order, formal charge, isotope, and
  stereochemistry;
- InChI/InChIKey identity plus an internal stable identifier;
- functional-group and reactive-site perception;
- protonation/tautomer states as explicit related species, not aliases;
- atom mapping for reactions;
- structure depiction and formula generation from the same graph;
- structure-to-property links carrying method, range, and uncertainty.

Use a mature cheminformatics toolkit at build time to generate and cross-check
artifacts. Keep the runtime representation compact and deterministic. Toolkit
output is reviewed as a `generated-shipping-artifact`; build-time use alone does
not authorize redistribution of its inputs, mined templates, or output files.

### Reaction-family cascade

For each family, store a balanced, atom-mapped transformation template plus:

- required functional groups and forbidden contexts;
- reagents/catalysts and broad condition class;
- competing pathways and selectivity rules;
- kinetic or equilibrium model where available;
- expected observations and analytical signatures;
- provenance and confidence;
- build-time energetic and structural checks.

Prioritize families by curriculum multiplication:

1. proton transfer and organic acid/base equilibria;
2. alkene addition and elimination;
3. nucleophilic substitution;
4. alcohol oxidation and carbonyl reduction;
5. carbonyl addition and condensation;
6. esterification, hydrolysis, and saponification;
7. electrophilic aromatic substitution at a qualitative/curated level;
8. radical addition/polymerization;
9. pericyclic showcase reactions where orbital symmetry is the lesson.

Templates propose. Filters reject impossible contexts. Curated rules rank.
Build-time xTB/DFT data can verify energetics and enrich explanation, but it
must not turn an unverified proposal into a runtime fact. Those tools remain
oracle-only unless their generated output and all input datasets have an
independent shipping approval.

### Confidence vocabulary

- `computed`: solved by a model within its declared domain;
- `curated-family`: a verified template and conditions match;
- `curated-instance`: only this exact reaction is claimed;
- `estimated`: property/group-contribution result with uncertainty;
- `qualitative`: direction or site only, no quantitative yield/rate;
- `unsupported`: no model claims the state.

### Exit demonstrations

- One template correctly spans a homologous series and preserves atom mapping.
- Competing substitution/elimination outcomes change with declared conditions
  without pretending to predict precise yields.
- Esterification couples equilibrium, kinetics, heat, and distillation/removal
  of a product in one bench history.
- Polymerization reports conversion and a molecular-weight distribution model,
  not one impossibly large explicit molecule.

## R6 — Analytical chemistry as a coverage multiplier

An instrument turns invisible state into an experiment. This expands usable
chemistry without requiring a new reaction predictor.

### Instrument contract

An instrument declares sampled compartment/phase, perturbation, detection
limit, calibration, resolution, noise model, and returned observable. Ideal
instruments can be offered explicitly beside realistic ones.

### Priority instruments

- pressure and gas-volume measurement;
- conductivity from ionic mobilities and activities;
- absorbance/UV–Vis from spectra and path length;
- IR from build-time frequencies or curated bands;
- emission/flame spectra;
- calorimetry with apparatus heat capacity and loss;
- potentiometry and reference electrodes;
- gravimetry and precipitate recovery;
- chromatography from partition/retention models and ideal plates;
- mass spectra as curated/computed fragmentation only where justified.

This supports qualitative analysis, unknown identification, calibration curves,
Beer–Lambert failure, endpoint choice, purity checks, and structure evidence.
The simulated observation must always derive from the state plus an instrument
model; it must not be copied from the lesson's expected answer.

### Exit demonstrations

- Identify ions through a sequence of computed wet tests with false-positive
  and detection-limit behavior.
- Produce a titration curve and realistic indicator/electrode readings from the
  same state.
- Separate a mixture chromatographically and connect peaks back to recovered
  material.
- Run a calorimetry experiment where vessel heat capacity and heat loss explain
  the difference from an ideal enthalpy.

## R7 — Materials, environmental, polymer, and nuclear modules

These should share the world model but remain separate scientific domains.

### Materials and metallurgy

- extend CEA-backed high-temperature reduction, oxidation, and slag/gas
  equilibria over a curated element set;
- add solid phase transitions and reaction enthalpies;
- represent alloys and binary phase diagrams only where redistribution rights
  for parameter data are clear;
- treat nucleation, grain size, and mechanical properties as separate models,
  not consequences of equilibrium composition.

Acceptance targets: iron extraction, copper smelting, lime/cement chemistry,
oxidation scales, and one explicitly scoped binary alloy diagram.

### Environmental and soil chemistry

Build on PHREEQC surface/exchange/transport support: alkalinity, hardness,
buffering, adsorption, mineral weathering, contaminant mobility, ocean
acidification, and treatment columns. This is one of the best breadth-to-engine
ratios in the roadmap.

### Polymers

Represent chain populations by moments or distributions: conversion,
number/weight-average molar mass, dispersity, branching, and crosslink fraction.
Do not instantiate millions of monomer nodes. Couple polymerization kinetics to
heat release, viscosity regimes, and phase separation only as supported.

### Photochemistry

Make the light source explicit: spectrum, flux, exposure, and geometry. A
photochemical rate requires absorption and a quantum yield or a curated rate;
“shine UV” must not be a magic reaction trigger. Begin with photolysis,
photoisomerization, and atmospheric ozone/NOx teaching networks.

### Nuclear chemistry

Keep this an optional, cleanly separated module: nuclides, decay modes,
branching ratios, Bateman chains, activity, absorbed dose only when geometry and
radiation type are stated. Nuclear transmutation must never pass through the
ordinary element-conservation auditor; it has its own nucleon/charge/energy
ledger.

Biochemistry remains out of scope until there is a separate solvent,
macromolecule, compartment, and enzyme-kinetic design. It is not “more organic
chemistry.”

## The data system is the real scaling engine

The runtime should consume compact, versioned, licence-audited artifacts. Raw
scientific datasets and heavyweight tools belong in reproducible build-time ETL.
Only allowlisted inputs may flow through ETL into a shipping artifact. Restricted
sources may be queried by a segregated oracle job, but they do not enter this
pipeline.

```text
upstream source
  → pinned fetch + licence record + checksum
  → parse into typed source records
  → normalize identity, units, phase, conditions
  → retain provenance and uncertainty per value
  → cross-check conflicts and applicability
  → compile target-specific runtime packs
  → replay acceptance corpus
  → publish a manifest of included records and sources
```

### Registry records

Split the present monolithic species entry into linked records:

- identity and molecular structure;
- elemental/isotopic composition;
- phase-specific thermodynamic properties;
- transport properties;
- optical/analytical properties;
- acid/base and microstate relationships;
- safety classifications;
- model parameters and validity ranges;
- literature and licence provenance;
- uncertainty or quality grade.

Every numeric field needs units, conditions, phase, source, and method. “Boiling
point = 78.4” without pressure and provenance is invalid data, not a convenient
default.

### Property resolution ladder

When a model asks for a property, resolve it explicitly:

1. measured curated value in range;
2. authoritative database value in range;
3. fitted correlation in range;
4. group-contribution estimate with applicability check;
5. build-time quantum estimate;
6. unavailable.

Never silently jump down the ladder. The chosen rung and uncertainty travel
with the result. Every rung is also constrained by the artifact lane: a better
scientific value from a blocked source remains unavailable to the shipping app.

### Compile, do not parse, large runtime datasets

The browser currently ships raw PHREEQC databases both as engine input and,
indirectly, inside Rust for derived indexes. The complete NASA CEA text is also
embedded and parsed even though the runtime admits only registry-named species.
Generate compact indexes and a reachable CEA subset at build time. Keep raw
files only where the external engine itself requires them.

This reduces download, startup parsing, memory, and the number of runtime parser
failure modes while preserving exact source provenance.

## Verification architecture

Broader coverage is valuable only if each new model makes stronger falsifiable
claims.

### Five levels of verification

1. **local invariants** — non-negative amounts; element, mass, charge, and
   energy balances; normalized phase fractions;
2. **metamorphic tests** — scaling amount, reordering independent additions,
   splitting transfers, and changing units must transform answers predictably;
3. **analytic cases** — closed-form kinetics, ideal flashes, Nernst limits,
   lever rule, and limiting-dilution behavior;
4. **differential oracles** — compare against independent engines such as
   Reaktoro, Cantera, trusted thermophysical packages, and build-time scientific
   Python tools;
5. **experimental benchmarks** — DOI-pinned, licence-audited measurements with
   conditions and tolerances.

Agreement between two paths using the same database is useful but is not an
independent validation of that database. Record shared ancestry in provenance.
Oracle comparisons persist only allowlisted benchmark facts or aggregate error
metrics; they do not commit restricted source exports.

### Validation corpus

Create `validation/cases/` separately from lessons. A validation case contains:

- exact initial ledger, apparatus, and boundary conditions;
- model path expected to claim it;
- quantities and tolerances;
- conservation expectations;
- source/oracle and licence;
- known disagreement or failure modes;
- the domain over which the case is allowed to generalize.

Lessons may reference validation cases, but prose and pedagogy must not become
the scientific golden file.

### Uncertainty and sensitivity

Add parameter uncertainty before adding probabilistic flourish. For a result,
report which input values dominate it and whether model disagreement is larger
than parameter uncertainty. DiffSol's sensitivity support or finite-difference
build-time sweeps can generate this initially. The expert register should be
able to answer “what assumption controls this result?”

## Performance and delivery

The backend architecture remains client-neutral. The web app is one client and
should not dictate the scientific interfaces.

Still, several delivery changes directly support broader models:

- run the Rust lab and IPhreeQC together in one module Web Worker; their internal
  synchronous hook can stay synchronous while UI messaging is asynchronous;
- ship and load the existing pre-warmed lesson cache before attaching the live
  engine;
- compile dataset subsets and derived indexes rather than parsing source text at
  runtime;
- cache individual problem nodes by canonical input, model version, and data
  manifest—not an opaque whole-vessel string alone;
- return state deltas/rendered events on the hot path and fetch full state only
  on demand;
- execute independent compartments in parallel where no interface couples them;
- establish performance budgets on a low-end school device, including solver
  latency, memory, startup, and long-task counts;
- make the service-worker installation atomic for required assets and optional
  for advanced model packs.

Model packs should be independently downloadable: aqueous core, advanced
organics, combustion, spectra, and so on. Offline-first does not require every
learner to download every domain before the first experiment. A download is
still distribution: every pack carries a signed manifest, licence, attribution,
source offer where applicable, and store/channel approval. Packs are a payload
and update boundary, never a way around the allowlist.

## Suggested code evolution

Prefer modules until a portability, dependency, or ownership boundary justifies
a crate. The likely eventual boundaries are:

```text
kerotakis-state          conserved ledger, lots, compartments, interfaces
kerotakis-registry       identity/property/reaction artifact readers
kerotakis-orchestrator   routing, coupling, transactions, audits
kerotakis-phreeqc        aqueous/gas/surface/exchange adapter
kerotakis-thermo         fluid properties and phase equilibria
kerotakis-cea            high-temperature gas/condensed equilibrium
kerotakis-kinetics       reaction-network IR and stiff integration
kerotakis-electro        electrodes, circuits, charge-transfer kinetics
kerotakis-organic        structure graph and verified reaction templates
kerotakis-instruments    measurement and detection models
kerotakis-safety         prospective-state policy and classifications
kerotakis-codex          curriculum graph and claims
kerotakis-core           public bench operations and compatibility facade
```

Do not split all of these immediately. First define `state`, `model`, and
`transition` modules inside the current crates, migrate one solver end to end,
and extract only after the interfaces survive that use.

## Recommended execution order

| Order | Work | Why it comes here |
|---:|---|---|
| 0 | Reconcile the store permission/data grant and automate the allowlist | No new source or dependency should enter through an ambiguous path. |
| 1 | Conserved ledger, lots, compartments, interfaces, transactional model contract | Every later domain depends on it; it prevents solver-order bugs from multiplying. |
| 2 | PHREEQC gas/surface/exchange/solid-solution adapters | Largest immediate coverage gain from an engine already shipped and tested. |
| 3 | Generated registry artifacts and the property-resolution service | Data can scale without becoming hand-written code or bypassing licence gates. |
| 4 | Complete phase-equilibrium layer and powered apparatus | Unlocks gases, mixtures, extraction, distillation, recrystallization, and honest energy use. |
| 5 | Generic reaction-network IR plus stiff integration | Turns time from two curated examples into a reusable domain. |
| 6 | Explicit electrodes and electrochemical kinetics | Builds naturally on activities, interfaces, and the time integrator. |
| 7 | Structural organic layer and verified reaction families | Large content multiplier once state, rates, and properties have stable homes. |
| 8 | Instrument framework | Converts the expanded hidden state into experiments and assessments. |
| 9 | Environmental, polymer, photochemical, materials, and optional nuclear packs | These become bounded applications of the common substrate rather than exceptions in core. |

Some work runs continuously across all stages:

- expand the validation corpus;
- record validity, uncertainty, data licence, and provenance;
- grow the registry through generated artifacts;
- turn every discovered limitation into a machine-readable boundary and a
  pedagogical opportunity;
- keep safety policy separate from scientific capability.

## Executable task list

The following list is dependency ordered. Each item is intended to be one
reviewable change or a short sequence with one observable acceptance result.
Do not begin a task that imports a new dependency or dataset until its `LIC`
gate is complete.

### Phase 0 — Licence and provenance rails

- [ ] **LIC-001 — Resolve the operative store-permission text.** Obtain
  qualified review of the scope difference between `LICENSE` and `NOTICE`, the
  `-or-later` wording, and the copyright-holder-only intent. Done when all
  operative files use one approved text and a test pins it.
- [ ] **LIC-002 — Resolve curated-data store distribution.** Choose one of the
  three structures described above and update the data contribution grant.
  Done when every data contribution has an unambiguous public licence and
  official-store grant or is excluded from the store payload.
- [x] **LIC-003 — Define `provenance/sources.toml`.** Add a schema containing
  source id, artifact lane, exact licence/SPDX id, terms URL, copyright holder,
  retrieval date, checksum, attribution, upstream inputs, allowed outputs,
  targets, reviewer, and decision.
- [ ] **LIC-004 — Inventory the current tree.** Add every `Cargo.lock` package,
  vendored source, PHREEQC database, NASA file, codex data file, lesson, image,
  and generated asset to the source manifest. Done when an unlisted file class
  fails lint.
- [ ] **LIC-005 — Implement `kero provenance lint`.** Reject missing records,
  non-allowlisted `runtime-*` licences, ambiguous `NOASSERTION`, missing
  attribution, stale checksums, and an oracle output with no shipping verdict.
- [ ] **LIC-006 — Add `cargo-deny`.** Configure an explicit runtime/development
  graph policy. Resolve dual-licensed crates through an approved permissive
  branch; do not allow an LGPL alternative merely because it appears in an
  `OR` expression.
- [ ] **LIC-007 — Generate notices.** Use `cargo-about` or an equivalent pinned
  tool to generate the in-app/source attribution bundle; compare it with the
  source manifest in CI.
- [ ] **LIC-008 — Generate an SBOM.** Produce CycloneDX/SPDX manifests for CLI,
  web, iOS, and Android release payloads. Done when CI can diff the actual
  payload against approved runtime sources.
- [ ] **LIC-009 — Define the model-pack manifest.** Include content hash,
  engine/data ABI, licence, attribution, source URL, signature, minimum app
  version, and lane. Reject unsigned or unapproved packs.
- [ ] **LIC-010 — Segregate oracle jobs.** Give oracle jobs separate caches and
  output directories; add a CI assertion that their raw inputs/outputs are not
  copied into release or source artifacts.
- [ ] **LIC-011 — Add the dependency/data PR checklist.** Every new source must
  answer: what is conveyed, under which exact terms, whether outputs ship,
  whether database rights exist, whether store terms are compatible, and how
  removal works.
- [ ] **LIC-012 — Audit current release artifacts.** Build every target, unpack
  it, and manually reconcile it once against the manifest. This becomes the
  golden baseline for automated payload audits.

No later phase may introduce external material before LIC-003 through LIC-006
are enforced in CI. Store publication additionally depends on LIC-001 and
LIC-002.

### Phase 1 — State and orchestration foundation

- [ ] **ARCH-001 — Freeze current behavior.** Snapshot the JSON contract and
  accepted outputs of every lesson at all registers. Record intentionally
  unstable numeric fields separately from structural compatibility.
- [ ] **ARCH-002 — Add typed quantity gaps.** Introduce types for power,
  current, potential, area, amount density, flow, and photon flux, with unit
  round-trip tests.
- [ ] **ARCH-003 — Introduce `ConservedLedger` in shadow mode.** Derive it from
  every current vessel without changing behavior. Assert element, mass, charge,
  and sensible-energy agreement after every existing operation.
- [ ] **ARCH-004 — Introduce `MaterialLot`.** Record additions and transfers
  independently of resolved species. Prove that two lots can merge physically
  without losing their provenance or particle-size metadata.
- [ ] **ARCH-005 — Introduce `ResolvedState`.** Move aqueous `SolutionInfo`,
  thermal equilibrium, saturation, and phase interpretation behind an
  invalidatable derived-state container.
- [ ] **ARCH-006 — Add `Compartment` and `Environment`.** Wrap the current
  vessel as one well-mixed liquid/solid compartment with the existing open-air
  behavior expressed as boundary conditions.
- [ ] **ARCH-007 — Add `Headspace` and `Interface` types.** Land data structures
  and serialization first, with no new chemistry. Preserve old save/log replay
  through migration defaults.
- [ ] **ARCH-008 — Define `StateDelta`.** Require models to propose ledger,
  phase, energy, and environment transfers rather than mutate a vessel.
- [ ] **ARCH-009 — Add transactional commit/rollback.** Validate positivity and
  conservation before commit. Inject failures at each stage and prove the
  bench remains byte-equivalent to its pre-step state.
- [ ] **ARCH-010 — Define capability/validity reports.** Replace the boolean
  concept of `applies` with a structured result while keeping an adapter for
  current equilibrators.
- [ ] **ARCH-011 — Build the first orchestrator path.** Route one simple water
  operation through problem planning, old-solver adaptation, audit, and atomic
  commit.
- [ ] **ARCH-012 — Migrate current solvers one at a time.** Suggested order:
  mixing → state transitions → aqueous → curated → kinetics → thermal →
  electrochemistry → honesty. Run the frozen corpus after each migration.
- [ ] **ARCH-013 — Remove sequential direct mutation.** Delete the compatibility
  path only when every solver returns deltas and an order-randomization test
  proves independent model ordering does not change accepted state.
- [ ] **ARCH-014 — Emit a coverage manifest.** For every registered operation
  and species family, report claimed models, validity, observables, validation
  cases, and unsupported dimensions.

### Phase 2 — Broader aqueous, gas, surface, and transport chemistry

This phase uses the already approved IPhreeQC source/database set. A new
PHREEQC database is a separate `LIC` task, not part of feature implementation.

- [x] **AQ-001 — Compile a finite `Headspace` to `GAS_PHASE`.** Start with a
  sealed CO₂/water system; round-trip gas moles, pressure, dissolved carbon, and
  water mass.
- [x] **AQ-002 — Add gas boundary modes.** Implement sealed, open reservoir,
  pressure-controlled, and swept headspaces as explicit constraints.
- [x] **AQ-003 — Support inward and outward gas transfer.** Replace the current
  one-way atmospheric treatment. Acceptance: the complete limewater/excess-CO₂
  sequence conserves carbon.
- [x] **AQ-004 — Add headspace energy accounting.** Include gas sensible
  enthalpy and pressure/volume constraints; test open versus sealed heating.
- [x] **AQ-005 — Add typed `SurfaceSites`.** Compile one oxide surface model to
  `SURFACE`; retain capacity and occupancy on the interface ledger.
- [x] **AQ-006 — Validate pH-dependent adsorption.** Add an independent oracle
  comparison that persists only approved benchmark values/error metrics.
  Reaktoro 2.13 is not eligible for this case because surface complexation is
  still an open upstream capability gap. The active implementation uses a
  project-owned, development-only intrinsic mass-action/site-balance oracle
  over the already-approved USGS constants; its stated omission of diffuse-
  layer electrostatics makes it an edge-direction/position check, not another
  full surface solver.
- [x] **AQ-007 — Add `ExchangeSites`.** Compile finite-capacity cation exchange
  and test a batch water-softening case.
- [x] **AQ-008 — Add `SOLID_SOLUTIONS`.** Begin with one approved mineral pair;
  prove component and phase conservation across precipitation/dissolution.
- [x] **AQ-009 — Spike PHREEQC `KINETICS` behind the new rate contract.** Compare
  its state trajectory with Kerotakis' integrator for one mineral-dissolution
  case; choose which layer owns time integration based on evidence.
  **Decision:** Kerotakis owns time integration and the vessel clock; PHREEQC
  remains the aqueous equilibrium/speciation engine plus an opt-in development
  comparator. A project-authored first-order calcite dissolution case now
  compares five ordered samples with the analytic solution and enforces a
  maximum cross-engine remaining-mineral error of `5e-5` relative, while Ca/C
  ledgers close independently. Both MY-BASIC preview platforms and the full
  native/Wasm/browser matrix passed in CI run `32503082107`. This is evidence
  about numerical ownership, not a physical calcite-rate claim: the chosen
  constant has zero uncertainty only because it is an exact test parameter.
- [x] **AQ-010 — Implement partial freezing as two compartments/phases.** Remove
  pure ice, re-equilibrate the residual brine, and stop at a stated eutectic or
  model boundary.
  **Complete 2026-08-21:** the native and browser application stacks now couple
  aqueous speciation to the water phase pass until liquid composition and the
  ice fraction agree within the aqueous engine's declared `0.05 K` resolution.
  PHREEQC receives only liquid solvent mass; solid water remains a pure ice
  ledger, and each solvent transfer invalidates and re-solves the residual
  brine. Core checks pin bounded convergence, stable repeat solves, water
  conservation, and the explicit `252 K` low-temperature boundary. A live NaCl
  check pins pure-ice exclusion, increased residual particle molality, sodium
  and water conservation, common liquidus temperature, and stable repetition.
  Below the boundary the app emits an explicit refusal because salt solids and
  a solute-specific phase diagram are required. No external dataset or runtime
  dependency was added. Native Ubuntu/macOS, strict codex lint, both MY-BASIC
  previews, core/IPhreeqc/full/combined Wasm, and the real-browser demo passed
  in CI run `32506920952`.
- [x] **AQ-011 — Add a 1-D cell chain.** Implement conservative transfer between
  cells before adding reaction; test a passive tracer.
  **Complete 2026-08-21:** `kerotakis-core` now provides a uniform chain of
  existing `Vessel` cells with simultaneous first-order upwind transfer of
  liquid and aqueous portions. Each step reports its injected and effluent
  boundary parcels, invalidates stale solution metadata, and leaves solids,
  surfaces, exchange sites, solid solutions, and headspaces owned by their
  original cells. Typed pre-mutation checks reject invalid Courant numbers,
  non-uniform or empty liquid geometry, incompatible inlet volume, invalid
  mobile state, and thermostatted cells whose environmental heat would make a
  hidden ledger term. The passive-tracer acceptance check pins the repeated
  binomial profile, invariant cell water volume, stationary inventories, and
  per-step species, analytical-charge, and sensible-energy closure. No
  reaction coupling, PHREEQC `TRANSPORT`, new data, dependency, or external
  source was added. Native Ubuntu/macOS, strict codex lint, both MY-BASIC
  previews, core/IPhreeqc/full/combined Wasm, and the real-browser demo passed
  in CI run `32508147378`.
- [x] **AQ-012 — Add exchange/transport coupling.** Produce a finite-column
  breakthrough curve while conserving each exchanged element.
  **Complete 2026-08-21:** `CellChain::advance_reactive` now performs one
  conservative transport step followed by inlet-to-outlet local equilibrium,
  returns indexed solver events, and restores the complete pre-step chain if
  any cell solve fails. A live four-cell sodium-form resin column uses the
  existing typed PHREEQC `EXCHANGE` adapter for 12 pore volumes. Its calcium
  effluent begins below `1e-8` of feed, exceeds 80% of feed by pore volume 12,
  and rises by more than 25 percentage points from the midpoint; dissolved plus
  exchanger-bound calcium and sodium close against inlet and effluent after
  every step within `2e-8 mol`, and every finite exchanger remains capacity
  balanced. Core also pins whole-chain rollback and the one-part-per-million
  hydraulic equality appropriate to the app's explicitly approximate
  water-only volume proxy. No PHREEQC `TRANSPORT`, new source, dataset,
  dependency, or vendored change was added. Native Ubuntu/macOS, strict codex
  lint, both MY-BASIC previews, core/IPhreeqc/full/combined Wasm, and the
  real-browser demo passed in CI run `32509744496`.
- [x] **AQ-013 — Add surface/transport coupling.** Produce one adsorption-front
  case and compare it with PHREEQC's own transport result.
  **Complete 2026-08-21:** a live four-cell HFO column now advances 20
  full-cell shifts through the project-authored reactive cell chain and the
  existing typed `SURFACE` adapter. Dissolved plus surface-bound zinc and
  sulfate close against inlet and effluent after every shift within
  `2e-8 mol`; finite strong/weak site capacities remain valid. The normalized outlet
  starts below `1e-8`, reaches at least 80% of feed, and agrees with a separate
  PHREEQC `TRANSPORT` calculation to within one shift at half-breakthrough,
  2.5% mean absolute curve error, and 25% at every individual grid sample.
  Pooled native/browser engines are reset between vessel solves, surface
  readback cannot create more bound sorbate than the vessel's analytical
  inventory, and reactive hydraulic drift is bounded by finite surface-site
  capacity rather than a global relaxed tolerance. Raw PHREEQC `TRANSPORT`
  remains an engine-gated development oracle only; the shipped app continues
  to use AGPL-owned transport and exposes no raw transport API. No external
  source, dataset, dependency, or vendored-source change was added. Native
  Ubuntu/macOS, strict codex lint, both MY-BASIC previews,
  core/IPhreeqc/combined Wasm, the Wasm bench, and the real-browser demo passed
  in CI run `32514634767`.
- [x] **AQ-014 — Publish the R1 acceptance suite.** Limewater, carbonated bottle,
  surface release, softener breakthrough, and partial freezing must work native,
  Wasm, cached replay, and offline.
  **Complete 2026-08-21:** one typed, serialisable R1 runner now exercises all
  five outcomes through the supplied `Equilibrator`; it contains no alternate
  chemistry implementation. A native integration test runs the suite against
  linked IPhreeqc, exports its content-addressed results, then proves exact
  replay without growing the cache. `kero prewarm` makes the same states part
  of the shipped postcard. Cache-only Wasm runs all five with no solver hook;
  combined Wasm runs them through live Emscripten IPhreeqc. The built web app
  loads and service-worker-caches the postcard, and its real-browser gate runs
  all five both online and again after the HTTP server is stopped. Any missing
  solver state becomes a named failed case rather than an approximation. No
  external source, dataset, dependency, database, species, kinetics,
  MY-BASIC, or vendored-source change was added. CI run `32516261610` passed
  native Ubuntu/macOS, strict codex lint, both MY-BASIC previews, core,
  IPhreeqc, cache-only and combined Wasm, and the online/offline browser demo.

### Phase 3 — Generated registry and property service

- [x] **DATA-001 — Define typed registry schemas.** Separate identity,
  composition, phase thermodynamics, transport, optical data, safety,
  microstates, and model parameters. Include units, conditions, uncertainty,
  source id, and method on every numeric record.
  **Complete 2026-08-21:** the new project-owned `kerotakis-data` crate keeps
  those eight independently reviewable record families in a versioned source
  document joined by stable species and source ids. Every numeric claim is a
  typed value/unit/dimension plus applicability, explicit uncertainty, source,
  and described method; qualitative claims carry the same evidence boundary.
  Validation rejects duplicate and dangling ids, empty evidence, non-finite
  values, invalid intervals/uncertainties, and known property/condition
  dimension mismatches, reporting all issues deterministically. Source lanes
  make the AGPL distribution boundary executable: only material already
  reviewed into `runtime` may enter a future pack, while build and external
  oracles remain non-distributable. No external data was imported, no current
  species or runtime behavior changed, and no third-party dependency was
  added. CI run `32518836256` passed schema round trips and negative validation
  tests, strict Clippy and native tests on Ubuntu/macOS, the new data-crate
  `wasm32` gate, both MY-BASIC previews, IPhreeqc/cache/combined Wasm, the Wasm
  bench, and the real-browser demo.
- [x] **DATA-002 — Export the current 75 species.** Generate the new source
  records from existing Rust declarations, diff every field, and keep runtime
  behavior unchanged.
  **Complete 2026-08-21:** the unpublished, build-only
  `kerotakis-registry-export` crate converts all 75 current declarations into
  the DATA-001 contract without becoming a dependency of the app or any
  simulation crate. The checked-in human-readable export contains 75 sources,
  identities, and compositions, 238 phase-property records, 64 optical
  records, and 103 legacy model parameters. Tests compare every old field:
  key, name, formula and parsed composition/charge, InChIKey, molar mass, heat
  capacity, density, phase, appearance, flame/reflective colour, tint strength,
  evaluated spectrum, dissolution enthalpy, both behavior flags, and verbatim
  provenance. A second gate regenerates the JSON byte for byte, so registry
  drift cannot silently pass. The roadmap's earlier count of 74 was corrected
  to the actual 75. All exported sources remain `build_oracle` with an explicit
  legacy-review-required license reference; zero are eligible for a runtime
  pack. No external data was imported, no third-party dependency was added,
  and lookup or simulation behavior did not change. CI run `32520759936`
  passed the export/diff gates, strict Clippy and native tests on Ubuntu/macOS,
  both MY-BASIC previews, core/data/IPhreeqc/cache/combined Wasm, the Wasm
  bench, and the real-browser demo.
- [x] **DATA-003 — Compile a deterministic runtime pack.** Use a versioned binary
  format with reproducible ordering and content hash; provide a human-readable
  inspection command.
  **Complete 2026-08-22:** `compile-registry` binary reads the JSON source
  registry, validates it, serializes to postcard binary with a KREG header
  (magic + version + SHA-256 content hash), and writes a `.pack` file.
  586 KB JSON → 116 KB binary (5x reduction). Deterministic: same input
  produces identical hash `cd14829b...`.
- [x] **DATA-004 — Load the pack behind the current registry API.** Keep static
  Rust data as a fallback until all tests pass on native and Wasm.
  **Complete 2026-08-22:** `load_pack()` in `kerotakis-data` reads a `.pack`
  file, verifies the KREG magic, version, and SHA-256 content hash, then
  deserializes the postcard payload. Three unit tests (round-trip, bad magic,
  hash mismatch). The 116 KB registry pack round-trips correctly.
- [ ] **DATA-005 — Implement the property-resolution ladder.** Return value,
  rung, uncertainty, validity, and provenance; return unavailable rather than a
  naked default.
- [ ] **DATA-006 — Add a tiny CC0 import.** Import a handful of Wikidata identity
  records end to end through fetch, normalize, review, compile, notice, and
  explain. This validates the legal/data pipeline before bulk scale.
- [ ] **DATA-007 — Add a tiny approved PubChem import.** Select fields whose
  upstream annotations are compatible, retain per-field provenance, and reject
  one deliberately incompatible annotation in a test.
- [ ] **DATA-008 — Generate PHREEQC derived indexes at build time.** Remove the
  duplicate runtime parsing copy while retaining the engine's approved raw
  databases where required.
- [ ] **DATA-009 — Generate the reachable CEA subset.** Include only admitted
  registry species plus citations/notices; compare all existing thermal cases
  bit-for-bit or within declared tolerances.
- [ ] **DATA-010 — Remove the hand-authored runtime registry.** Do this only
  after source-pack round trips, reproducibility, provenance lint, and all
  target builds pass.

### Phase 4 — Phase behavior and apparatus

- [ ] **THERMO-001 — Audit candidate code and parameter data separately.** Add
  source records for the exact FeOS/vle/water-property versions and every
  parameter file before adding dependencies.
- [ ] **THERMO-002 — Put the existing ideal VLE behind a `FluidModel` trait.**
  Preserve water/ethanol tests and expose validity/errors through the model
  contract.
- [ ] **THERMO-003 — Add phase-specific property records.** Heat capacities,
  vapour-pressure correlations, densities, and latent heats must carry ranges
  and sources.
- [ ] **THERMO-004 — Complete UNIFAC only from approved parameters.** Every
  group and interaction parameter points to an allowlisted source record; the
  proprietary consortium table is mechanically blocked.
- [ ] **THERMO-005 — Implement bubble/dew and TP flash.** Validate ideal limits,
  pure-component limits, and phase/material balance.
- [ ] **THERMO-006 — Add HP and UV flashes.** Couple energy and phase state;
  verify latent-heat plateaus and sealed-vessel pressure.
- [ ] **THERMO-007 — Integrate an approved equation-of-state backend.** Start
  with one model and a small cleared parameter set; do not bundle upstream
  databases wholesale.
- [ ] **THERMO-008 — Add liquid–liquid split.** Validate one binary/ternary
  extraction case from an allowlisted experimental source.
- [ ] **APP-001 — Add powered heat sources.** Replace “free” evaporation with
  power, duration, heat loss, and boundary conditions while retaining the old
  operator as a clearly external-powered shorthand.
- [ ] **APP-002 — Add condenser and receiver connections.** Prove matter and
  energy conservation in simple distillation.
- [ ] **APP-003 — Add repeated ideal stages and reflux.** Acceptance is the
  ethanol–water azeotrope plus the impossibility of crossing it under the
  selected model.
- [ ] **APP-004 — Add separatory-funnel stages.** Compare one large extraction
  with repeated small extractions at equal solvent total.
- [ ] **APP-005 — Add recrystallization.** Track recovered crystals, mother
  liquor, cooling energy, and an explicit solubility-model boundary.

### Phase 5 — Generic kinetics

- [ ] **KIN-001 — Define the reaction-network IR.** Include stoichiometry,
  locality, reversibility, dimensional rate law, catalysts/sites, validity,
  uncertainty, and source ids.
- [ ] **KIN-002 — Compile the two current rate laws into the IR.** Require
  identical lesson outputs before deleting their bespoke evaluator path.
- [ ] **KIN-003 — Add reaction-network conservation lint.** Balance elements,
  charge, sites, and declared electron transfer for every compiled reaction.
- [ ] **KIN-004 — Audit and add DiffSol.** Allow only the approved permissive
  feature graph; keep JIT/native extras off mobile and Wasm.
- [ ] **KIN-005 — Implement adaptive implicit integration.** Add positivity,
  event detection, rejection/retry, and exact-solution tests.
- [ ] **KIN-006 — Couple kinetics to fast equilibrium.** Advance one bounded
  kinetic step, re-equilibrate, measure splitting error, and reduce the step
  when needed.
- [ ] **KIN-007 — Implement the Cantera YAML parser without importing data.**
  Fuzz the supported schema and produce useful errors for unsupported rate
  forms.
- [ ] **KIN-008 — Audit one mechanism file.** Give it its own runtime-data
  record; if it fails, keep it oracle-only and choose another rather than
  weakening the allowlist.
- [ ] **KIN-009 — Compile and validate the first gas mechanism.** Compare batch
  trajectories with Cantera while persisting only approved benchmarks/errors.
- [ ] **KIN-010 — Add build-time mechanism reduction.** Emit the reduced
  network, declared envelope, source lineage, and maximum error against the
  full oracle.
- [ ] **KIN-011 — Add heterogeneous-rate inputs.** Surface area, effective
  particle radius, site density, and mixing regime must be explicit before
  catalyst amount affects rate.
- [ ] **KIN-012 — Add batch and plug-flow apparatus models.** Reuse one network
  and prove that residence-time behavior follows from apparatus, not different
  reaction data.

### Phase 6 — Electrochemistry

- [ ] **ELEC-001 — Add explicit electrode/interface state.** Material, area,
  roughness, deposits, and connected compartment must serialize and replay.
- [ ] **ELEC-002 — Move current Nernst/Faraday behavior onto electrodes.** Keep
  all existing cell and electrolysis tests unchanged.
- [ ] **ELEC-003 — Add reviewed kinetic parameter records.** No folklore table
  enters runtime; each exchange-current/Tafel/overpotential value needs an
  allowlisted source and validity conditions.
- [ ] **ELEC-004 — Implement Butler–Volmer/Tafel kinetics.** Test equilibrium,
  low-overpotential, and Tafel limits analytically.
- [ ] **ELEC-005 — Add galvanostatic and potentiostatic control.** Keep charge,
  electrical work, and chemical conversion in the ledger.
- [ ] **ELEC-006 — Add ohmic and diffusion limits.** Begin with a boundary-layer
  model; surface the geometry assumption.
- [ ] **ELEC-007 — Add competing electrode reactions.** Choose deposition versus
  gas evolution from thermodynamics, kinetics, activities, and available
  parameters; otherwise refuse quantitative efficiency.
- [ ] **ELEC-008 — Add deposit/passivation state.** Let surface coverage alter
  subsequent kinetics without changing elemental inventory.
- [ ] **ELEC-009 — Publish electrochemical acceptance cases.** Electroplating,
  cell discharge, concentration polarization, corrosion, and sacrificial
  protection must identify which of thermodynamics, kinetics, transport, or
  inventory limits each result.

### Phase 7 — Structural and organic chemistry

- [ ] **ORG-001 — Audit the exact structure toolkit path.** Approve Indigo/InChI
  versions and bundled data/notices for runtime, or keep them build-time and
  implement the minimal runtime graph in Kerotakis.
- [ ] **ORG-002 — Define the molecule graph.** Bond orders, formal charge,
  isotope, stereochemistry, atom ids, and serialization round-trip.
- [ ] **ORG-003 — Add canonical identity and formula derivation.** Cross-check a
  cleared corpus with two independent tools without importing either corpus.
- [ ] **ORG-004 — Add functional-group perception.** Start with the groups
  needed by one reaction family; fuzz SMARTS/graph matching.
- [ ] **ORG-005 — Define atom-mapped transformation templates.** Lint atom,
  charge, and stereochemical mapping before application.
- [ ] **ORG-006 — Implement one family end to end.** Choose esterification or
  saponification because it exercises structure, aqueous state, equilibrium,
  kinetics, heat, and separation.
- [ ] **ORG-007 — Cross-validate template application with RDKit oracle-only.**
  Persist discrepancies and approved small factual fixtures, not RDKit exports.
- [ ] **ORG-008 — Add conditions and incompatibility filters.** A template match
  is a proposal; conditions and forbidden context decide whether it is claimed.
- [ ] **ORG-009 — Add confidence labels to the public event contract.** Distinguish
  computed, curated-family, curated-instance, estimated, qualitative, and
  unsupported.
- [ ] **ORG-010 — Add families one at a time.** Each PR includes source audit,
  template tests, counterexamples, at least one lesson, and a declared
  selectivity/yield boundary.
- [ ] **ORG-011 — Add oracle enrichment as a separate pipeline.** xTB/CREST,
  PySCF, Reaction-QM, or CRD-derived artifacts enter only after an individual
  generated-output review and source-manifest record.
- [ ] **ORG-012 — Add polymer population state.** Implement conversion and
  molar-mass moments before any polymerization family claims chain-length
  distributions.

### Phase 8 — Instruments and observations

- [ ] **INST-001 — Define the instrument contract.** Sampling, perturbation,
  detection limit, calibration, resolution, uncertainty/noise, and provenance.
- [ ] **INST-002 — Migrate eyes, balance, thermometer, and pH meter.** Preserve
  deterministic ideal mode; add realistic mode only with parameters.
- [ ] **INST-003 — Add gas pressure/volume instruments.** Validate ideal and
  non-ideal model routing.
- [ ] **INST-004 — Add conductivity.** Use approved mobility/conductivity data;
  state concentration and temperature validity.
- [ ] **INST-005 — Complete UV–Vis/indicator measurements.** Every spectrum or
  coefficient must be CC BY/CC0/public-domain/project-cleared; published images
  and restricted spectral databases stay out.
- [ ] **INST-006 — Add calorimetry.** Model calorimeter heat capacity and loss;
  recover the ideal enthalpy in the zero-loss limit.
- [ ] **INST-007 — Add chromatography.** Begin with ideal plates and approved
  partition parameters; connect peak area to the conserved material recovered.
- [ ] **INST-008 — Add qualitative-analysis workflows.** Unknown identification
  must emerge from computed tests and detection limits, never from a scripted
  answer key.

### Phase 9 — Bounded advanced packs

- [ ] **ADV-001 — Environmental pack.** Assemble only approved PHREEQC data and
  project-authored scenarios for soils, treatment, weathering, and ocean
  acidification.
- [ ] **ADV-002 — Photochemistry IR.** Add light-source state and photolysis
  rates; admit a network only with approved spectra/cross sections and quantum
  yields.
- [ ] **ADV-003 — Materials/metallurgy pilot.** Expand the cleared CEA subset for
  one iron/copper process; audit every added thermodynamic record.
- [ ] **ADV-004 — Polymer kinetics pilot.** Couple one project-authored network
  to the population moments and heat ledger.
- [ ] **ADV-005 — Nuclear module design.** Define a separate nuclide ledger and
  identify a CC0/public-domain decay source before writing runtime code.
- [ ] **ADV-006 — Keep biochemistry parked.** Open it only with an approved data
  source and a separate solvent/macromolecule/enzyme-kinetics architecture.

### Phase 10 — Delivery and maintainability

- [ ] **WEB-001 — Ship the generated pre-warmed cache.** Give it a
  `generated-shipping-artifact` record and prove the cache contains no
  unapproved oracle-derived material.
- [ ] **WEB-002 — Move both Wasm engines into one module Worker.** Keep their
  synchronous internal bridge; expose asynchronous command/progress/cancel to
  clients.
- [ ] **WEB-003 — Split model packs.** Core aqueous, phase, combustion,
  structures, and spectra get independent signed manifests and payload audits.
- [ ] **WEB-004 — Make offline install atomic.** Required allowlisted assets
  must all cache or installation fails; optional packs fail independently.
- [ ] **PERF-001 — Add bundle/model-pack budgets.** Measure compressed size,
  parsed memory, initialization, and solve latency on a low-end reference
  device.
- [ ] **PERF-002 — Add node-level cache keys.** Include model version, dataset
  manifest hash, constraints, and canonical inputs; test invalidation on every
  component.
- [ ] **CI-001 — Build scientific artifacts once.** Reuse signed/payload-audited
  outputs across Wasm, bridge, browser, and publication jobs.
- [ ] **CI-002 — Separate fast, full, and oracle validation.** PRs run fast
  invariants; main runs all cleared acceptance cases; scheduled jobs run
  optional oracles without becoming a release dependency.
- [ ] **REL-001 — Add a release gate.** Refuse publication unless tests,
  provenance lint, dependency policy, notices, SBOM, source offer, pack
  signatures, and unpacked-payload reconciliation all pass.

## What not to optimize for

- **Raw species count.** Ten thousand names without compatible properties do
  not create ten thousand simulatable substances.
- **A single winner solver.** Specialist models with explicit coupling are more
  honest and more capable.
- **Equilibrium everywhere.** Metastability, barriers, transport, and apparatus
  are often the experiment.
- **Precise yields from reaction templates.** A transformation family is not a
  process model.
- **Runtime quantum chemistry.** Build-time enrichment gives more reliability,
  smaller binaries, and clearer provenance.
- **ML as the final authority.** It may propose candidates for build-time
  verification; it must not convert unsupported chemistry into a confident
  runtime result.
- **Three-dimensional realism.** Well-mixed compartments, interfaces, and 1-D
  transport cover far more chemistry per unit complexity than CFD or molecular
  dynamics.
- **Silent fallback.** An approximation is useful only when it identifies
  itself.

## The durable product idea

The moat is not a giant list of reactions and not one heroic numerical engine.
It is a system in which:

- a learner performs an operation on explicit apparatus;
- several specialist models cooperate over one conserved world;
- the same state yields macroscopic, particle, symbolic, and instrumental
  evidence;
- every answer identifies its model, data, validity, and uncertainty;
- model disagreement and model failure remain visible;
- curated chemistry multiplies through structures, families, and parameterized
  conditions;
- every scientific claim is replayable and independently testable.

That architecture can cover most school chemistry, a substantial part of
undergraduate physical/inorganic/analytical chemistry, and carefully bounded
families of organic and materials chemistry without ever claiming to be an
arbitrary synthesis oracle.

## Licence-policy references

- [`LICENSE`](LICENSE), [`NOTICE`](NOTICE), and
  [`CONTRIBUTING.md`](CONTRIBUTING.md) — the current repository terms whose
  store-permission and curated-data language must be reconciled in LIC-001 and
  LIC-002.
- [GNU Affero General Public License v3](https://www.gnu.org/licenses/agpl-3.0.html)
  — base code licence, including the framework for additional permissions.
- [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0),
  [BSD 3-Clause](https://opensource.org/license/bsd-3-clause), and
  [MIT](https://opensource.org/license/mit) — representative directly
  includable code terms; exact files and notices still require review.
- [CC BY 4.0 legal code](https://creativecommons.org/licenses/by/4.0/legalcode.en),
  [CC0 1.0 legal code](https://creativecommons.org/publicdomain/zero/1.0/legalcode.en),
  and [CC BY-SA 4.0 legal code](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en)
  — the direct-data allowlist and the existing ShareAlike-data mismatch.

## Primary technical references

- [USGS PHREEQC Version 3 manual](https://water.usgs.gov/water-resources/software/PHREEQC/documentation/phreeqc3-html/phreeqc3.htm) — equilibrium, gas phases, exchange, surfaces, solid solutions, kinetics, and transport.
- [Cantera reference](https://cantera.org/stable/reference/index.html) and [YAML input format](https://cantera.org/stable/yaml/index.html) — reaction mechanisms, rates, transport, reactors, and interchange format.
- [FeOS documentation](https://feos-org.github.io/feos/) — Rust equations of state, phase equilibria, and density-functional capabilities.
- [DiffSol documentation](https://martinjrobins.github.io/diffsol/) — implicit integration, events, DAEs, and sensitivity analysis in Rust.
- [Reaktoro documentation](https://reaktoro.org/) — independent equilibrium/kinetics reference and differential oracle for water–gas–rock systems.
- [InChI Trust](https://www.inchi-trust.org/) — canonical chemical identity infrastructure.
- [PubChem PUG REST](https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest) — one interface for build-time identity/property retrieval; every imported field still requires its own provenance and licence review.

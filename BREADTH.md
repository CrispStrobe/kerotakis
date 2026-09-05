# Kerotakis — Breadth programme

Dependency-ordered work for making the bench answer the ordinary questions a
curious child or teenager asks: what happens if I mix, heat, cool, burn, crush,
drop, dissolve, separate, or inspect familiar matter?

This document is the source of truth for `BRD-*` tasks. `CAPABILITIES.md` owns
solver parity, `EXPERIMENTS.md` owns experiment and quest content,
`APPARATUS.md` owns equipment affordances, and the GUI roadmaps own rendering.
Those documents link here rather than restating task scope.

The programme does **not** promise arbitrary chemistry. No available engine can
reliably decide what every arbitrary set of reagents will do, and the project
deliberately does not ship an unrestricted synthesis oracle. Breadth comes from
three honest layers:

1. familiar substances and named materials represented as reviewed data;
2. general solvers used where their model domain applies;
3. curated reaction families with explicit substrate and condition gates.

Every uncovered branch remains a typed, visible `NotYetModelled` result. A
larger catalog must never turn absence of a model into “nothing happened.”

## Rules every BRD task inherits

- Shipped code must satisfy the permissive-only bar in `PLAN.md` and
  `deny.toml`. Re-verify the exact release and transitive dependency graph at
  adoption time; a licence named here is not a substitute for that review.
- Shipped data is CC0 or CC BY 4.0 only. CC BY-SA data may be an external or
  build-time oracle but does not enter official app-store packs. Source,
  licence, retrieval date, checksum, original citation, and per-field
  provenance are mandatory under `CONTRIBUTING.md`.
- A code licence does not license bundled parameters, force fields, reaction
  mechanisms, structures, or database records. Audit these separately.
- Network access is build-time only. Every released pack and every runtime
  engine works offline.
- A new visible number carries `Provenance`; a new parser gets a fuzz target;
  a new solver route gets conservation and relevant metamorphic invariants plus
  an independent golden or differential oracle.
- Do not bulk-import “everything.” Importers first produce a quarantine report;
  only allowlisted fields and reviewed records graduate into runtime packs.
- Each task is one branch/PR unless its scope explicitly says otherwise. An
  agent claiming a task records its ID in the PR and does not silently absorb a
  dependent task.

Large data/content parents are intentionally sliceable. Claim them as
`BRD-012.S01`, `BRD-014.S01`, `BRD-023.S01`, `BRD-031.S01`, `BRD-041.S01`,
`BRD-052.S01`, or `BRD-060.S01`; add the next zero-padded slice to the parent
status before work starts. Every slice must be independently useful, pass all
parent invariants, name its exact records/families, and stay small enough for
one review. The parent closes only when its numeric/content acceptance floor is
met. Agents may work on independent slices concurrently.

A decision gate closes with either `go` or `no-go`. On `go`, its implementation
children become available. On `no-go`, those children are marked
`not-applicable` with the decision record and downstream milestones treat that
track as honestly closed, not missing. A no-go may not remove the user outcome:
the record must name the existing/fallback route that owns it.

## Target and measurement

“Everything a young person can think of” is not a finite acceptance criterion.
The programme therefore measures a versioned **curiosity corpus** of concrete
prompts. The initial target is 500 prompts spanning at least these action
families:

| Family | Examples |
|---|---|
| Mix and dissolve | salt + water, oil + water, cola + milk, soap + grease |
| Heat and cool | boil alcohol, freeze seawater, melt wax, bake soda |
| Burn and oxidise | candle, sugar, steel wool, alcohol, fuel with too little air |
| Acids, bases and gases | vinegar + chalk, fizzy drinks, bleach, ammonia cleaner |
| Separate | filter soil, distil colored water, chromatography, crystallise salt |
| Materials | paper, wood, plastic, glass, concrete, rust, batteries, alloys |
| Food and life | bread rising, digestion, enzymes, fermentation, respiration |
| Handle and inspect | pour, spill, drop, crush, smell, view particles/molecules/crystals |

The corpus is a regression suite, not a claim that every prompt must compute.
Every prompt must end in exactly one auditable disposition: computed, curated,
qualitative model, explicit boundary, or missing task. Silent fall-through is a
failure.

## Dependency graph and delivery stages

```text
Stage B0 — measure and define matter
  BRD-000 curiosity corpus ──→ BRD-001 coverage report
  BRD-002 MaterialRecipe IR ─→ BRD-003 import/quarantine framework

Stage B1 — stock the everyday shelf
  BRD-003 ─┬→ BRD-010 PubChem importer ─┐
           ├→ BRD-011 ChEBI importer ───┴→ BRD-012 familiar pure-substance pack
           └→ BRD-013 USDA importer
  BRD-002 + BRD-012 + BRD-013 ───────────→ BRD-014 household-material packs

Stage B2 — make structure and reaction families load-bearing
  BRD-012 ─→ BRD-020 reaction-family IR ─→ BRD-021 Indigo/RDKit spike
                                            └→ BRD-022 runtime executor
  BRD-014 + BRD-022 ───────────────────────→ BRD-023 first family pack

Stage B3 — broaden general solver domains (parallel after B1)
  BRD-012 ─→ BRD-030 feos spike ─→ BRD-031 fluid parameter pack ─→ BRD-032 routing
  BRD-012 ─→ BRD-040 Cantera audit ─→ BRD-041 mechanism packs ─→ BRD-042 FFI gate

Stage B4 — life and solids
  BRD-011 + BRD-020 ─→ BRD-050 bio IR ─→ BRD-051 Rhea importer ─→ BRD-052 bio pack
  BRD-003 ─→ BRD-060 COD importer ─→ BRD-061 spglib adapter
                 BRD-060 + BRD-061 + BRD-081 ─→ BRD-062 crystal experience

Stage B5 — tactile and visual reach
  BRD-070 authority contract ─→ BRD-071 Rapier ─→ BRD-073 spill/breakage
                           └──→ BRD-072 Salva ───┘
  BRD-014 + BRD-070 ──────────→ BRD-074 gas/foam observables
  BRD-014 + spectral optics ──→ BRD-075 dye/pigment mixing
  BRD-041 + BRD-070 + BRD-071 → BRD-076 movable heat/flame tools
  BRD-000 + BRD-012 + BRD-023 → BRD-077 element coverage/table modes
  BRD-012 + BRD-080 viewer spike ─→ BRD-081 molecular/crystal viewer
  BRD-022 ─────────────────────────→ BRD-082 Ketcher authoring surface

Stage B6 — validate and graduate
  BRD-032 ─→ BRD-090 pycalphad oracle where a cleared TDB exists
  all shipped tracks ─→ BRD-100 breadth release gate
```

Stages express release order, not a ban on parallel work. Tasks with all listed
dependencies complete may proceed concurrently. `BRD-042`, `BRD-082`, and
`BRD-090` are optional gates; they cannot hold the first household pack hostage.

## Stage B0 — measurement and shared data contracts

### BRD-000 — Curiosity corpus v1

- [x] **Status:** complete on `codex/brd-000-curiosity`. **Size:** medium.
  **Depends on:** nothing.
- **Outcome:** `tests/coverage/curiosity-v1/manifest.toml` indexes ordered TOML
  shards containing exactly 500 stable prompts,
  expected dispositions, age band, action family, and capability tags. The
  corpus contains harmless, hazardous, nonsensical, and intentionally
  unsupported questions; it is not a list of only happy paths.
- **Scope:** derive prompts from the already-audited experiment corpora, shipped
  lessons, common household/school materials, and editorially authored edge
  cases. Define stable normalization for aliases and quantities. Add a runner
  that executes runnable prompts and records typed routing outcomes without
  snapshotting prose.
- **Integration:** reuse `.lab`/replay and JSON event contracts. Map each prompt
  to `EXP-*`, `CAP-*`, and `BRD-*` tags where applicable.
- **Acceptance:** every prompt parses or has an explicit parse-boundary
  expectation; corpus lint catches duplicate normalized prompts, missing tags,
  and unowned gaps; CI can run a small smoke subset while the full report is a
  scheduled/native tier.
- **Out of scope:** claiming scientific support for all prompts or changing a
  solver to make the initial percentage look better.

### BRD-001 — Coverage classifier and report

- [x] **Status:** complete on `codex/brd-001-baseline`. **Size:** medium.
  **Depends on:** BRD-000 (complete).
- **Outcome:** `kero coverage curiosity` emits deterministic JSON and a compact
  human report with counts for `computed`, `curated`, `qualitative`,
  `boundary`, and `missing`, grouped by action, material class, age band, and
  owning task.
- **Scope:** classify from typed engine/router events, never output-string
  matching. Preserve the engine/model/dataset provenance for successful paths
  and the precise refusal reason for gaps. Add a checked-in baseline so a PR
  cannot silently turn a computation into a refusal or vice versa.
- **Acceptance:** byte-deterministic reports; a synthetic routing regression
  fails CI; the report distinguishes “substance unknown” from “substance known,
  reaction unknown.”
- **Out of scope:** a vanity percentage with unlike dispositions collapsed.
- **Implementation note (2026-08-27):** typed parser errors, solver-route
  evidence, the five-way runner, deterministic grouped reports, the
  cross-family smoke gate, and a 500-entry native baseline are shipped. The
  baseline pins owner/outcome/reason per prompt and treats seven initial solver
  failures as their own regression state; ownership and graduation rules live
  beside it in `tests/coverage/curiosity-v1/README.md`.
- **Coverage state (2026-09-04).** Against the 500-prompt corpus:

  | disposition | `origin/main` | now |
  |---|---|---|
  | computed | 229 | 269 |
  | curated | 20 | 20 |
  | qualitative | 47 | 70 |
  | boundary | 59 | 59 |
  | **missing** | **145** | **82** |

  Expectation mismatches held at 85 throughout, which is the number that
  catches a classifier guard written wider than its evidence.

  **What the remaining 82 need is not more of the same.** Sixty carry
  `unknown-species`, and after this branch's twenty-six additions the tail is
  no longer substances — it is `pepsin`, `lactase`, `pondweed`, `chlorophyll`,
  `acetobacter`, `sourdough_starter`: enzymes and organisms, which need a
  mechanism decision before a recipe can be written. The rest want hydrocarbon
  fuel species (`methane`, `propane`, `butane`, `petrol`, `diesel`), which is
  registry work of a different kind — species with CEA thermochemistry rather
  than mass-fraction recipes.

  **The single largest blocker is that no protein species is installed.** One
  gap wearing five names: egg white does not set at 65 °C, gelatine does not
  gel, cream does not whip, albumin does not denature by heat, acid, alcohol
  or salt, and onion does not sting the eyes. It gates most of the
  kitchen-chemistry half of the corpus, and no amount of recipe writing moves
  it.

  **Two measurement lessons are recorded in the corpus README** because they
  cost real time: a prompt was being classified on its *neighbour's* solver
  routes when its script began with a non-equilibrating operator, and diffing
  corpus tokens against `registry-source-v1.json` reports substances that
  already resolve (material aliases do not live where that walk looks — probe
  the binary instead).

### BRD-002 — `MaterialRecipe`: named mixtures and objects

- [ ] **Status:** in progress. **Size:** large.
  **Depends on:** current pack loader
  (`CAP-21`/`DATA-010`).
- **Outcome:** the data schema represents vinegar, bleach, air, milk, paper,
  steel, soil, and batteries without pretending each is a pure species.
- **Scope:** add a versioned `MaterialRecipe` containing identity/aliases,
  component amount ranges and basis, optional unresolved fractions, physical
  form, preparation/lot assumptions, allowed substitutions, model confidence,
  provenance per component, and an expansion policy. `add v1 vinegar 10mL`
  expands once into conserved vessel contents and records the recipe version
  in the event log. Solid objects may carry geometry/surface-area metadata but
  still expand into ledger-owned components.
- **Integration:** registry packs, story stock, cabinet search, safety screening,
  replay/cache keys, and `explain`. Built-in species always remain canonical
  identities; recipes cannot override them.
- **Acceptance:** exact component and mass ledgers across add/undo/replay;
  deterministic selection within declared ranges (fixed recipe or seeded
  sample, never ambient randomness); an unresolved fraction is displayed and
  conserved rather than discarded; English/German aliases resolve.
- **Out of scope:** reverse-engineering branded proprietary formulations or
  treating a nutrient panel as complete molecular composition.
- **Foundation checkpoint (2026-08-27):** the source/pack schema now carries
  versioned recipes, localized aliases, component ranges, explicit unresolved
  fractions, form/geometry, substitutions, confidence and fixed/seeded
  expansion. Validation prevents species shadowing and ambiguous material
  names; deterministic expansion conserves the requested basis amount. The
  checked-in pack contains initial 3% peroxide and 5% vinegar recipes. Runtime
  `add`, stock/safety/replay integration remains the next BRD-002 checkpoint.
- **Runtime checkpoint (2026-08-27):** `add` now resolves a material key/alias,
  converts a volume only through reviewed bulk density, pins recipe ID/version,
  basis and sample seed in the serialized operator, expands once, screens the
  complete prospective mixture, deposits canonical species, and retains an
  explicit unresolved-material ledger. Events keep both the familiar material
  identity and component amounts. Built-in and optional-pack recipes share the
  runtime registry without allowing shadowing. Remaining BRD-002 work is stock
  depletion, proportional transfer of unresolved portions, cabinet cards and
  full undo/UI coverage.
- **Stockroom checkpoint (2026-08-29):** the shelf now holds finite bottles.
  `Bench` owns a `StockLedger` (`crates/kerotakis-core/src/stock.rs`), a new
  `StockShelf` operator (`stock NaCl 0.5mol`) fills one, and `Add`/
  `AddMaterial` draw against it in the unit the dispense already carries —
  moles for a registry species, the recipe's own basis for a material. An
  untracked key stays an unlimited supply, so every existing script and the
  sandbox default are unchanged. Running out mutates nothing and reports
  `Event::StockExhausted` with both numbers, in all three registers. Because
  the ledger lives on `Bench`, the protocol's opaque snapshot token
  round-trips it — undo and scrub restore the bottle level for free, proved
  by `the_snapshot_token_round_trips_the_shelf_stock`. The scene carries the
  bottles additively (`stock`, omitted when empty) and the GUI shelf shows
  each level, dimming and refusing an emptied card. Proportional transfer was
  found **already correct**: `Decant` scales every liquid/aqueous portion and
  every unresolved liquid portion by one shared fraction, so solutes move with
  their solvent; `tests/proportional_transfer.rs` pins it against regression.
  Remaining BRD-002 work is cabinet search/`explain` integration for recipes.

### BRD-003 — Source adapter, quarantine, and promotion framework

- [x] **Status:** done. **Size:** large. **Depends on:** BRD-002 and the existing
  `kerotakis-data` pack compiler.
- **Outcome:** all external breadth sources use one auditable path:
  fetch/build-time snapshot → raw quarantine → field allowlist → normalized
  candidate → human-reviewed runtime pack.
- **Scope:** define adapter output, source/record/field provenance, licence lane,
  checksums, rejection reasons, identity conflict reports, units normalization,
  and reproducible snapshot manifests. No runtime HTTP. Add commands to diff an
  upstream refresh without automatically promoting changed records.
- **Acceptance:** synthetic tainted fields and incompatible licences cannot
  enter a runtime pack; two source records for one InChIKey produce a reviewable
  merge/conflict report; rebuilding from a pinned snapshot is byte-identical;
  parser fuzz target and provenance lint pass.
- **Out of scope:** a generic data lake or unattended periodic imports.
- **Foundation checkpoint (2026-08-28):** `kerotakis-data` now exposes the
  shared offline quarantine contract: versioned SHA-256 snapshot manifests,
  deterministic candidate serialization, exact per-field provenance/licence,
  explicit field-and-licence review policies, and same-identity conflict
  reports. Review returns eligible/rejected fields and cannot mutate the
  runtime registry.
- **Review-tooling checkpoint (2026-08-28):** the offline
  `quarantine-review` binary verifies snapshot manifests against raw bytes,
  canonicalizes candidate fixtures, applies review policies, and emits a
  deterministic record/identity/field-level refresh diff. A checked-in
  synthetic fixture pins the required directory and manifest shape. These
  commands only print review artifacts; none can write a runtime pack.
  Remaining BRD-003 work is units normalization, the parser fuzz target and a
  provenance lint consumable by BRD-010/011/013/060.
- **Gate checkpoint (2026-08-29):** the remainder lands and the task closes.
  Units normalization converges 201 reviewed upstream spellings — `g·cm⁻³`,
  `g/cc`, `℃`, `deg C`, `°F`, `kcal/mol`, `J mol-1 K-1`, `mg/L`, `wt%`, `ppm`,
  `Da`, `Å`, `M⁻¹cm⁻¹` — onto the `Dimension`/`Unit` vocabulary DATA-001
  already defines, with affine temperature scales rather than bare factors. A
  spelling fixes the physical quantity, never the semantic field: `g/L` and
  `J/(mol.K)` each serve two dimensions, so the target field's declared
  dimension picks the reading and a mismatch is refused. Everything outside
  the table — bare mass, bare energy, wavenumbers, `Mg`, `ppt` — is a typed
  rejection carrying the original string; nothing falls back to
  `Dimension::Other`. A 71-case checked-in fixture pins each spelling, and
  round-trip and idempotence hold over the whole table.
  A quarantined field now carries the source's unit spelling verbatim and a
  runtime field policy may declare its dimension, so review is where an
  external spelling becomes a `Unit` and records both.
  The `quarantine` fuzz target covers the external-bytes surface — snapshot
  manifests, candidate fixtures, promotion policies, unit spellings — and
  asserts both invariants the framework rests on: canonical quarantine bytes
  are stable across a re-parse, and an unpinned snapshot is always refused.
  The promotion lint is one function (`lint_promotion`) and one command
  (`quarantine-review lint`), so BRD-010/011/013/060 call the same gate from
  either side. It refuses missing per-field source or licence, a licence
  outside the runtime data lane (in a candidate *or* in the policy that would
  admit it), raw bytes that no longer hash to the pinned manifest, candidates
  claiming a snapshot they did not come from, and eligible-field lists naming
  fields the record does not carry. `tools/provenance-lint.sh` now runs that
  gate over the checked-in fixture in both directions: the clean flow passes,
  the tainted one is refused.

## Stage B1 — the everyday shelf

### BRD-010 — PubChem identity and approved-property adapter

- [x] **Status:** done on `brd010/pubchem-adapter`. **Size:** medium.
  **Depends on:** BRD-003.
- **Source/licence:** PubChem PUG REST and bulk records; US-government core is
  public-domain-like, but depositor annotations retain source-specific terms.
  Continue the existing per-field source allowlist; do not interpret “found in
  PubChem” as a licence. Primary docs: <https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest>
  and <https://pubchem.ncbi.nlm.nih.gov/docs/data-sources>.
- **Scope:** generalize the existing approved import into an adapter for CID,
  canonical/isomeric SMILES, Standard InChI/InChIKey, formula, charge, mass,
  depositor-neutral synonyms, and only explicitly cleared physical-property
  fields. Pin raw responses/bulk release identifiers and obey service limits.
- **Integration:** official InChI recomputes identity; `chematic` plus the
  selected BRD-022 engine cross-check structure; candidates enter quarantine,
  not `registry-source-v1.json` directly.
- **Acceptance:** a 100-record fixture covers salts, hydrates, isotopes,
  stereochemistry, mixtures incorrectly returned for names, and conflicting
  synonyms; no CAS-only or non-allowlisted annotation crosses the promotion
  boundary.
- **Adapter checkpoint (2026-08-29):** the adapter lands and the acceptance is
  met. `tools/fetch-pubchem-snapshot.py` pins one PUG REST/PUG View retrieval —
  101 seed names resolved to 100 distinct CIDs, 121 response bodies, SHA-256 in
  a BRD-003 `SnapshotManifest` — and `kerotakis-data`'s `pubchem` module turns
  those bytes into quarantine candidates and nothing else. The fixture is
  100 records: 70 single molecules, 20 salts, 8 hydrates and 2 mixtures, with
  6 isotopically labelled records, 6 stereochemistry pairs sharing an InChIKey
  skeleton, and 12 synonyms claimed by more than one record — `(+)-glucose` by
  three, `d(-)-tartaric acid` by the D-, L- *and* meso- records, `soda (van)`
  by both sodium carbonate and sodium bicarbonate. `brass` and `aqua regia`
  are reported as `MixtureRecord`, not taken as substances; `vinegar` and
  `acetic acid` landing on one CID is reported as `SharedNameResolution`.
  The promotion policy allowlists PubChem's own computed/curated core only.
  Everything else is refused by name through BRD-003's own review: CAS
  Registry Numbers (separated out of the depositor synonym list and validated
  by check digit so the refusal is exact), the depositor synonym list itself,
  other registry identifiers, `ExactMass` and the database descriptors, and
  **every** depositor annotation — see the finding below.
  The official IUPAC InChI library recomputes identity along **two independent
  routes** in `kerotakis-org`'s `native-inchi` gate, because one number would
  have conflated two different questions. Re-keying PubChem's own published
  Standard InChI puts nothing of ours in the path, so a disagreement there
  would be a statement about the source: it agrees on **100 of 100**, and the
  test asserts that exact count. Re-deriving the key from the structure
  exercises our own toolchain as well, and it now also agrees on **100 of
  100**.
  That second number is worth its history. On the molfile bridge this branch
  was first written against, the structure route agreed on only 73: 23 of the
  27 conflicts kept the record's connectivity block and returned the
  `UHFFFAOYSA` "no stereo, no isotope" hash, and the other four were `[Al]`,
  `[S]`, `[Mg]` and brass. Reported as one blended number that would have read
  as 27 defective PubChem records; split across the two routes it read
  correctly as a limitation of our own writer, on precisely the
  stereochemistry pairs, isotopologues and bare metals this fixture was built
  to contain. CAP-13 has since replaced the molfile detour with the library's
  own 0D input, and every one of those 27 disagreements went away — which is
  the confirmation that the split diagnosis was right. The per-record verdict is
  pinned beside the fixture so the dependency-free crate reads it in every
  build, and every disagreement surfaces as a BRD-003 `IdentityConflict` row
  naming its route rather than being resolved. The full dry run — snapshot → quarantine →
  allowlist → normalized candidates → `lint_promotion` — passes over 100
  records and 1297 policy-covered fields, and refuses each planted violation.
  Nothing was promoted: `registry-source-v1.json` is untouched, and
  `LicenseRef-PubChem-Public-Domain` is deliberately still absent from
  `default_runtime_data_licences()`, so shipping any of this remains a separate
  licence review.
- **Finding — PubChem supplies identity, not properties (2026-08-29).** This
  is load-bearing for BRD-012 and for anyone planning to source a physical
  property here, so it is recorded as a result rather than left implied.
  **No experimental physical property from PubChem is promotable.** Across the
  pinned snapshot's 84 depositor annotations from 9 upstream sources, every
  source that states a licence in the runtime data lane — ILO-WHO ICSC, whose
  licence note is exactly "Creative Commons CC BY 4.0" — supplies its boiling
  point as **prose** (`"78.29 °C @760 [mm Hg]"`, `"173 °F"`), and the **only**
  source that supplies a structured `Number` + `Unit` is DrugBank, which
  licences it **CC BY-NC 4.0** and therefore cannot enter a shipped pack at
  all. The two conditions never coincide on a single value. The adapter
  carries prose verbatim and refuses to parse it into a number: turning
  `"78.29 °C @760 [mm Hg]"` into a quantity is a guess about units, pressure
  reference and significant figures, and BRD-003's whole contract is that an
  importer never guesses. The consequence for BRD-012: the "available phases"
  and add-by-mass parts of its behavior matrix must come from a source with
  structured, runtime-licensed quantities, not from PubChem. PubChem's
  contribution to the everyday shelf is **identity** — CID, both SMILES
  flavours, Standard InChI/InChIKey, formula, formal charge, the masses,
  the IUPAC name and the record title — and that is what the promotion policy
  allowlists.

### BRD-011 — ChEBI identity and ontology adapter

- [x] **Status:** done. **Size:** medium. **Depends on:** BRD-003.
- **Source/licence:** ChEBI CC BY 4.0, monthly/nightly versioned dumps. Primary
  docs: <https://www.ebi.ac.uk/chebi/about> and
  <https://www.ebi.ac.uk/chebi/downloads>.
- **Scope:** ingest reviewed 3-star structures and the minimum useful ontology
  slice: identity, formula, charge, mass, names/synonyms, roles, and parent
  classes needed for biochemical/material search. Keep ChEBI IDs as external
  identifiers; Standard InChIKey remains the cross-source join.
- **Integration:** source attribution flows into NOTICE/data attribution and
  `explain`; ontology roles may seed search tags but never safety or reactivity
  without a separate reviewed rule.
- **Acceptance:** pinned-release reproducibility; tautomer/protonation conflicts
  are reported rather than merged; attribution survives pack compilation; no
  biological role is converted into a reaction rule.
- **Adapter checkpoint (2026-08-29):** `kerotakis-data::chebi` reads ChEBI
  release 253 (2026-07-07) from the immutable per-release archive, not the
  rolling `current/` directory, so the pin is a pin. The committed snapshot is
  87 curated reviewed three-star entities — common sugars, organic acids and
  their conjugate bases, amino-acid and nucleotide exemplars, salts, gases,
  polymers and familiar alkaloids — drawn from `compounds`, `names`,
  `chemical_data`, `structures` and `relation`, checksummed whole and pinned by
  the BRD-003 snapshot manifest. Ingestion is reviewed-only twice over: three
  stars *and* a curated status on the entity, and independently on every
  ontology relation, so ChEBI's `SUBMITTED` third-party deposits never become
  candidates.
  **Protonation and tautomer families are reported, never merged.** The join
  key is the full Standard InChIKey, so acetic acid and acetate stay two
  records with two keys and `identity_conflicts` finds nothing to merge across
  the whole snapshot. The relationship itself is published separately, from two
  independent signals: ChEBI's own `is_conjugate_acid_of` /
  `is_conjugate_base_of` / `is_tautomer_of` assertions, and shared InChIKey
  family membership. Where they disagree the report says which — 10 pairs
  corroborated both ways, 4 structural-only (citric acid to citrate(3−), which
  ChEBI links through the intermediate protonation states), 1 ontology-only
  (citrate(2−), an ontology class with no single defined structure). Family
  membership needs the skeleton *and* stereo blocks: maltose/lactose and
  cellulose/amylose share a skeleton block while being diastereomers, and a
  skeleton-only heuristic would have merged them.
  Identity is recomputed rather than believed. Mass is recalculated from
  ChEBI's own formula against a local IUPAC-2021 weight table — tolerance
  0.05 Da, ~7x the 0.007 Da atomic-weight-revision noise floor — and charge is
  cross-checked against the Standard InChI's `/q` and `/p` layers. Every
  disagreement, every polymer whose `(C6H10O5)n` formula admits no finite mass,
  and every entity with no InChIKey to join on, lands in the conflict report
  instead of being repaired or quietly promoted.
  **Roles are search tags and nothing else.** The firewall is default-deny in
  both directions: an ontology-derived field may only reach a tag target, no
  ChEBI field of any kind may reach a target whose name carries a
  safety/hazard/reactivity marker, and no non-ontology field may reach the tag
  lane. Planted violations — roles aimed at `safety_flags`, nine reserved
  target spellings, formula aimed at `reactivity_class` — are all refused, and
  the refusal stops the whole promotion.
  Per-record CC BY attribution travels as an ordinary promotable field, so it
  survives review into a compiled pack manifest with its source path intact
  rather than being reattached by hand. 31 tests; nothing enters the runtime
  registry.
  The `provenance/sources.toml` record sits in the `quarantine` lane, which
  reaches no runtime path and no release payload by construction, so it clears
  ChEBI for nothing: promotion into a shipping lane is its own reviewed record,
  written when a pack actually needs these fields.
  **Licence finding, reviewed and settled at CC BY 4.0 (2026-08-29):** ChEBI's
  flat-file `README` names the terms "CC Attribution-ShareAlike 4.0" and then
  points the reader at a `LICENSE` file that is verbatim CC BY 4.0 with no
  ShareAlike condition anywhere. Two authoritative sources settle it against
  that one line: the shipped licence text itself, and ChEBI's about page, which
  states "The data on this website is available under the Creative Commons
  License (CC BY 4.0)". The record in `provenance/sources.toml` is approved at
  `CC-BY-4.0` and keeps the conflicting line on file so a later reviewer meets
  the evidence instead of rediscovering the doubt. This mattered rather than
  being pedantry: under ROADMAP-Webapp.md's 2026-08-23 decision the README read
  literally would have excluded ChEBI from store builds altogether.
  **Follow-up owed upstream:** report the stale README line to EMBL-EBI so the
  flat-file README matches its own LICENSE file and about page.

### BRD-012 — Familiar pure-substance pack v1

- [ ] **Status:** open; slices in flight. **Size:** large/data-heavy.
  **Depends on:** BRD-010 and BRD-011.
  - `BRD-012.S02` — P0 school essentials plus the gated barium pair.
    Ammonium chloride, iron(III) chloride and sodium sulfate land as the
    three P0 salts named in the triage list below; barium chloride and
    barium hydroxide land as the P2 toxic virtual-only pair, gated behind
    a `ToxicSoluble` safety row and never entered as a household material
    recipe. Three supporting records ride with them because the engine
    needs them to speak at all: `NH4+` and `Ba+2` are the database master
    species dissolved ammonium and barium book back as, and `BaSO4` is the
    registry solid the Barite phase precipitates into — which is EXP-30's
    open "BaCl2 sulfate row". The litmus/indicator material of item 1 is
    NOT in this slice: it needs colour-state machinery.
  - `BRD-012.S03` — the food-chemistry identity tranche. The pure species
    the household-material recipes name and cannot resolve: glucose,
    fructose, malic acid, citric acid and cellulose, plus the citrate ion
    minteq.v4 books dissolved citrate back as. Glucose and fructose ride
    sucrose's finite neutral-solute rung with CRC room-temperature
    capacities, and are pinned distinct end to end — same C6H12O6,
    different InChIKeys, and a test that says so. Adding the first
    stereo-bearing SMILES this repo has ever curated also found a
    pipeline limit worth recording: the SMILES-to-molfile-to-InChI route
    does not preserve tetrahedral parity, so both sugars recompute to
    stereo-free skeletons and no registry key here distinguishes D from
    L. The sugars stay distinct because they are structural isomers, not
    because the identity carries chirality. The stereo SMILES are kept as
    a tripwire, and sucrose's asserted stereo key (never recomputed,
    because it is not in CURATED_STRUCTURES) is the open question this
    leaves behind. Cellulose follows
    starch's precedent exactly: a per-monomer anhydroglucose aggregate
    with no InChIKey asserted, because (C6H10O5)n is not a molecule.
    The two acids divide, and the division is the point of the slice:
    minteq.v4 is the only shipped database defining a Citrate master
    species, so citric acid computes its own pH from the database's three
    protonation constants, while malate is in none of the ~40 databases
    vendored with iphreeqc — so malic acid dissolves and the engine says
    out loud that its acidity is not in the pH, rather than publishing a
    neutral number that looks like an answer. Deliberately NOT in this
    slice: the BRD-014 juice/flour/paper recipes are not re-pointed at
    the new species, because that tranche was still unmerged when this
    one landed. That upgrade is the follow-up, and each recipe that gains
    a resolved fraction needs its own conservation test.
  - `BRD-012.S04` — six pure substances the curiosity corpus names and the
    parser could not resolve: the three fuel-gas alkanes (methane, propane,
    butane), helium, naphthalene and hydrogen sulfide. Chosen because each
    one already has an engine waiting for it — the NASA CEA database defines
    a gas record for every one of them, matched by composition, so the three
    alkanes reach the combustion solver on the same road sulfur and paper
    already take, and helium reaches the sealed-vessel pressure route with
    no chemistry underneath it at all. Naphthalene arrives with its melting
    and boiling points and an explicit boundary note saying that the slow
    room-temperature sublimation a mothball is known for is exactly what
    this bench does *not* model. Hydrogen sulfide arrives able to be named
    and weighed and nothing more: there is no sulfide ion on the shelf, so
    it forms no metal sulfide, and the row that asked for silver tarnish is
    closed as a typed observation rather than as chemistry. Six rows leave
    `unknown-species`, and the split matters more than the count: th-044,
    th-045 and th-046 reach `computed`/`computed-route` through the CEA
    equilibrium solve, th-095 reaches `qualitative`/`typed-observation` on
    a real computed pressure, and th-028 and th-070 reach
    `computed`/`typed-engine-event` — the classifier's weakest evidence,
    which here means the run happened rather than that the question was
    answered. Sublimation and metal-sulfide formation remain absent, and
    those two rows are closed against the parser, not against the science.
  - `BRD-012.S05` (2026-09-05) — one species and one energy: a block of
    uranium that warms itself. `th-122` asked whether radioactive decay
    heats the sample, and the bench had a decay ledger that advanced
    nuclides on the slow clock and deposited no energy at all, so the row
    could not be answered even in principle. Each radioactive row in
    `nuclide::TEACHING_NUCLIDES` now states what ONE of its decays leaves
    behind — `Deposited::Mev` with its own source, or `Deposited::NotCurated`
    with the reason there is no reviewed value — and `DecayClock` books that
    as heat into an adiabatic vessel with `ReactionHeatReleased`. Uranium
    joins the registry as a species so the block can be weighed, and
    `nuclide::BULK_RADIONUCLIDES` bridges the chemical contents to the
    nuclide table, because a tracer is spiked into the ledger and a block of
    metal is a portion, and those are deliberately separate stores.
    **`th-122` flips `missing`/`unknown-species` → `computed`/
    `typed-engine-event`.** THE NUMBERS ARE THE POINT: 1 g decays 1.075e9
    times a day (12 440 Bq/g, U-238's textbook specific activity), each α
    depositing 4.270 MeV, for 7.35e-4 J into 0.1162 J/K — 6.3 mK in
    twenty-four hours. A test that checked only the sign of ΔT would pass
    with the energy per decay wrong by any factor, so `tests/nuclear.rs`
    pins the energy per decay and derives the temperature from it. Three
    boundaries are declared rather than absorbed, each with a test: the
    ledger stops at Th-234, so this is the first α and NOT the ~51.7 MeV
    the whole series deposits in secular equilibrium (a factor of twelve);
    the block is modelled as pure U-238, where natural uranium's activity
    is about twice that because of U-234; and β rows book the MEAN electron
    energy with the neutrino's share and any penetrating γ excluded, which
    the table test enforces against the mass defect. Tc-99m books no heat
    at all, because an isomeric transition is almost all γ and no reviewed
    internal-conversion split is recorded — a refusal, not a zero.
- **Outcome:** at least 300 reviewed identities that a school-age user is likely
  to name, including common gases, acids/bases, salts, metals, minerals, fuels,
  solvents, sugars, fats, monomers, polymers-as-populations, pigments, and
  biological small molecules.
- **Scope:** select from curiosity-corpus demand, not database popularity. Every
  record needs the minimum viable behavior matrix: identity/composition,
  available phases, add-by-mass conversion, visible appearance or explicit
  unknown, safety coverage, supported solver routes, and honest refusals for
  unsupported routes. Add aliases in English and German.
- **Integration:** PHREEQC names and CEA/thermo/mechanism availability are
  resolved at build time. A registry identity does not imply that every engine
  supports it; coverage metadata makes that distinction queryable.
- **Acceptance:** `kero species` and the GUI catalog load records without code
  edits; every new record passes identity, molar-mass, safety-totality,
  provenance, locale, and route-coverage lints; curiosity report shows the
  identity-unknown bucket materially reduced with no false “inert” answers.
- **Out of scope:** importing hundreds of thousands of database entries or
  fabricating missing thermodynamic parameters.
- **Shelf screenshot triage (2026-08-27; formula is authoritative when labels
  are truncated):** 29 of the 39 visible entries already resolve. Add the ten
  real gaps progressively:
  1. **P0 school essentials:** ammonium chloride (`NH4Cl`), iron(III) chloride
     (`FeCl3`), sodium sulfate (`Na2SO4`), and a litmus/indicator material with
     acidic/basic colour-state data. These unlock common solubility, hydrolysis,
     crystallisation and indicator interactions using existing solver routes.
  2. **P1 constrained:** nitric acid (`HNO3`) as a clearly restricted lab stock;
     “carbonic acid” as carbonated water / dissolved `CO2(aq)`, not a stable
     neat-acid bottle. Land identity/safety first, then aqueous routing.
  3. **P2 virtual-only metals:** sodium (`Na`) and potassium (`K`), gated behind
     complete water/fire safety and qualitative reaction-family coverage before
     shelf exposure.
  4. **P2 toxic virtual-only barium salts:** barium chloride (`BaCl2`) and barium
     hydroxide (`Ba(OH)2`), gated behind soluble-barium safety and precipitation
     coverage. Never present these as household experiment supplies.
  UI acceptance for this batch includes full-name tooltips, formula-first search,
  and English/German aliases rather than duplicate identities.

### BRD-013 — USDA FoodData Central adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-003.
- **Source/licence:** FoodData Central CC0/public domain. Primary docs:
  <https://fdc.nal.usda.gov/api-guide/>. Branded formulations are volatile and
  incomplete; prefer Foundation Foods and stable generic records.
- **Scope:** import only components that can map honestly onto Kerotakis
  species or declared unresolved fractions: water, sugars, organic acids,
  salts/minerals, fat/protein/carbohydrate aggregate populations. Preserve the
  food description, basis, sample/release and analytical uncertainty.
- **Integration:** output `MaterialRecipe` candidates. Nutrients are not silently
  converted to unique molecules: “protein,” “fat,” “fiber,” and “ash” remain
  named aggregate components until a model explicitly handles them.
- **Acceptance:** fixtures for milk, egg, flour, juice, oil and sugar; component
  masses plus unresolved remainder reconcile to the declared serving/sample
  mass; API keys and network access never enter builds or runtime.
- **Out of scope:** nutrition advice, branded-product fidelity, flavor chemistry,
  or inferring pH/reactions from nutrient labels.
- **Adapter checkpoint (2026-08-30):** the adapter lands as
  `kerotakis_data::usda` over a pinned snapshot of fifteen Foundation Foods
  records — milk, egg, wheat flour, apple and orange juice, soybean oil,
  granulated sugar, butter, table salt, yogurt, oats, potato, carrot, white
  rice and dry cannellini beans. The pin is the versioned release archive
  (`FoodData_Central_foundation_food_json_2025-04-24.zip`, checksummed before
  it is read), not the REST API: the archive carries a release identity, needs
  no API key at all, and contains two records — whole milk (fdcId 746782) and
  whole egg (748967) — that `/food/{id}` returns 404 for and that the bulk
  endpoint silently omits from a fifteen-id request. Honey is absent because
  Foundation Foods carries no honey record.
  Mapping is deliberately narrow. Water, individually determined sugars,
  individually determined organic acids, starch and alcohol become proposed
  registry species; `protein`, `fat`, `ash`, `dietary fibre` and the
  carbohydrate no determination accounts for stay named, conserved, unresolved
  components. A sugar becomes a species only where USDA determined *that*
  sugar: wheat flour, oats, rice, carrots and beans report no individual
  sugar, so their whole carbohydrate stays unresolved, and `Sugars, Total` is
  refused as a restatement rather than read. Minerals are elemental totals —
  USDA measured how much sodium is in the food, never which salt it was in —
  so they are reported as an element inventory inside `ash` and never become
  an ion or a salt. Table salt is the sharpest case: its record states 38.7 g
  of sodium and no chlorine at all, so `NaCl` is an inference the adapter
  declines to make. Lactose, galactose, oxalic and quinic acid are determined
  but have no registry species; their mass stays named under its own compound
  name instead of vanishing into an anonymous remainder. Glucose, fructose,
  citric acid and malic acid are proposed species the registry does not carry
  yet, reported as a registry-gap list rather than silently dropped.
  Every amount is converted to the record's own per-100 g basis before any
  unit reaches a candidate, so a candidate quantity carries the reviewed
  `g/100g` spelling and normalizes onto `MassPerMass`; `kcal`, `kJ` and `IU`
  are typed rejections that keep their original spelling.
  Twelve of the fifteen foods reconcile — resolved plus named unresolved
  equals the declared 100 g within the record's own stated min/max spread —
  and three are reported conflicts rather than candidates: soybean oil,
  unsalted butter and table salt each leave a proximate unstated, and an
  unstated proximate is not a zero. `lint_promotion` passes the clean flow and
  refuses four planted violations plus a policy that would admit a ShareAlike
  licence. Nothing is promoted: the runtime registry and the material packs
  are untouched, and the snapshot sits in the new `quarantine` provenance lane
  with a CC0 licence and a `review-required` decision.

### BRD-014 — Household and school material packs

- [ ] **Status:** open; slices in flight. **Size:** large/data-heavy.
  **Depends on:** BRD-002, BRD-012, and BRD-013.
  - `BRD-014.S02` (2026-09-05) — thirteen materials the curiosity corpus names
    and the parser could not resolve: mayonnaise, prepared mustard, fruit jam,
    coconut fat, alcohol hand sanitiser, perfume, permanent marker ink, orange
    peel oil, kitchen grease, petrol, damp wood, sugar water and dried beans,
    plus one alias so `red_cabbage_extract` reaches the jar that already
    exists. The shelf reaches 102 recipes. Two of them exist to be acted on
    rather than to act: kitchen grease is the immiscible layer `dish_soap`'s
    emulsifier role had nothing to work on, and sugar water is the substrate
    the dry-yeast fermentation role was already able to eat. Three carry a
    bounded role of their own — mayonnaise as an opaque colloid, mustard as an
    emulsifier, orange peel oil as an immiscible liquid — and in every case
    the role is a declared observable rather than an interfacial model, which
    each recipe's own notes say. Deliberately NOT in this slice: activated
    charcoal and sunscreen. Both would parse and both would then answer their
    question WRONGLY — there is no adsorption model, so charcoal would leave
    the dye in the beaker, and the spectral bands stop at 405 nm, so a
    sunscreen could not absorb the 300 nm light bio-111 shines at it. A
    confident wrong answer is worse than `unknown-species`, and those two rows
    stay open until the models exist.
  - `BRD-014.S03` (2026-09-05) — the biology tranche: fourteen materials and
    two species. Chlorophyll and nylon 6,6 arrive as registry species;
    chlorophyll also gains a reviewed ethanol solubility so `bio-091`'s
    pigment extraction is a real dissolution rather than a shrug, and nylon
    deliberately gains NO aqueous solubility, because the reviewed zero the
    other polymers carry would assert that hot acid leaves it intact. Nine of
    the materials are plant and food matter whose rows the bench can run and
    cannot answer — the missing models are named in each recipe. Three carry
    something real: `food/meat` gains an enzyme-activity substrate profile,
    `laboratory/bile-salts` an emulsifier role whose mechanism is the one the
    question is about, and `household/alkaline-battery-electrolyte` a fully
    resolved 30% potassium hydroxide solution that is honestly not a battery.
  - `BRD-014.S04` (2026-09-05) — the electrical property the shelf did not
    have. Until now the registry carried no conductivity or resistivity for
    any solid, so `mat-011` ("why are wires copper rather than iron?") ran,
    reached the Kohlrausch solution meter, and got `NotYetModeled — no
    aqueous solution has been characterised`: true, and no answer, because a
    metal does not conduct the way a salt solution does. Seven dry solids
    (Ag, Cu, Al, Mg, Zn, Fe, graphite) now carry a curated
    `electrical_resistivity` in the registry with its own source record, and
    `conductivity::dry_solid_conductance` reads it — iron is 5.79 times
    copper's resistivity, which is the whole of the answer. THE ROW IS NOT
    FLIPPED. `measure <vessel> conductivity` is dispatched from
    `bench.rs`'s `Operator::Measure` arm, which this session was scoped out
    of; the model, its data and its tests are in place and the remaining
    change is one match arm delegating to `crate::conductivity` the way
    `Instrument::MeltingPointApparatus` already delegates to
    `crate::instrument::read_transition`. Deliberately NOT in this slice:
    elemental silicon. Adding it would answer no question — `mat-066` asks
    about `doped_silicon`, and a doped resistivity needs a carrier-density
    model this bench does not have. An intrinsic value alone would be a
    number nobody asked for; a doped one would be a confident invention.
    The tranche's provenance lane is PENDING REVIEW and its citation says so
    in its first sentence: the values are recorded as commonly tabulated,
    with the CRC Handbook's pure-metals table as the intended primary
    reference and every row flagged for reviewer confirmation against a
    positively identified copy, exactly as the phase-transition tranche is.
  - `BRD-014.S05` assessment (2026-09-05, no code) — **`bio-111` stays
    `missing`, and the BRD-014.S02 refusal above stands.** Extending the
    spectral table below 405 nm was assessed and rejected as neither small
    nor honest. `spectrum::BANDS` is a compile-time `16` behind 44 uses
    across ten files including the Kubelka–Munk pigment path and the
    registry validator, so the band count is not a local constant; six UV
    bands would also demand ε(λ) for zinc oxide, titanium dioxide and an
    organic absorber, each a new species with a safety-matrix row, and the
    two oxides work by scattering rather than absorption, so the
    Beer–Lambert path the question invites is the wrong physics for two of
    the three. No CC0/CC BY ε(λ) source for the organic absorber was found.
    The honest answer to "does sunscreen absorb UV" needs a UV model, not a
    wider table, and until there is one `unknown-species` remains the
    better answer.
- **Outcome:** versioned packs for at least 75 familiar named materials, selected
  by BRD-000 demand.
- **Scope:** begin with air, tap/seawater, vinegar, baking powder/soda, bleach,
  ammonia cleaner, hydrogen peroxide, soap/detergent surrogate, cola/fizzy
  drink, dry/wet yeast, dish soap, hand soap, pepper, isopropanol, food dyes,
  watercolor and acrylic-paint surrogates, milk, juice, flour/dough, vegetable
  oil, candle wax, paper, wood, common plastics, glass, soil/sand/clay,
  chalk/limestone, concrete surrogate, rusted/clean iron, steel/brass/bronze,
  and common battery chemistries. Each recipe states grade/concentration
  assumptions and unresolved material. Locale-sensitive ambiguous names such
  as English “soda” and German “Soda” must ask which material was meant rather
  than silently choosing baking soda, washing soda, or a fizzy drink.
- **Integration:** safety evaluates expanded components before chemistry;
  solver routing operates on those components; the UI and narration retain the
  material name; depletion and replay use the recipe version.
- **Acceptance:** at least 150 curiosity prompts become runnable or receive a
  more specific model boundary; every recipe has a conservation test and one
  characteristic behavior test; changing recipe version invalidates cache keys.
- **Out of scope:** product endorsements, clandestine composition guessing, or
  detailed toxicology from generic recipes.
- **Checkpoint 2026-09-05 — three recipes stopped describing bare iron.**
  The `metal/stainless-steel`, `metal/galvanized-steel` and
  `metal/painted-iron` entries each said in their own `lot_assumptions` that
  they resolve to iron the bench will attack, with no representation of the
  film or coat that is the whole point of the object. `corrosion::BARRIERS`
  (BRD-023's checkpoint above) is where the bench now keeps the first two of
  those sentences, keyed on the lot source the material route stamps, and it
  is enforced in the kinetics gate so a stainless spoon does not rust. The
  galvanised entry needs no barrier row at all: its zinc protects its iron
  by the ordinary galvanic rule, which is also the honest answer for a
  *scratched* sheet, since the zinc protects the iron it is merely next to.
  Closes curiosity row mat-014.

### BRD-020 — Reaction-family intermediate representation

- [ ] **Status:** open; phase 1 (IR + chematic oracle, #272), phase 2
  (conservation ledger + order independence, 2026-09-02) and phase 3 (the
  router in the standard stack, 2026-09-05) landed. **Size:**
  large. **Depends on:** BRD-012 and the landed kinetics/curated-reaction
  infrastructure.
  *Phase 2, 2026-09-02: the IR shipped contract-first — templates that could
  be applied, with nothing checking that applying one conserves matter. That
  check now exists and REFUSES: every application is weighed, atoms including
  implicit hydrogens and formal charge, in against out, and a set that does
  not balance is declined by name ("template 'x' does not conserve matter:
  C: 4 in, 5 out") rather than returned. A ledger counting only heavy atoms
  would have balanced while every hydrogen vanished, which is the error it
  exists to catch. Also landed: `apply_template_any_order`, because SMIRKS
  matching is positional and a bench does not know which vessel the learner
  poured first — permutations are tried in a fixed order so the same inputs
  always give the same products. Acceptance covered: atom/charge conservation
  on all four curated families, a deliberately inventing template refused by
  name, out-of-domain substrates declining rather than overgeneralising, and
  order independence. STILL OPEN and deliberately not attempted here: the
  router/equilibrator integration itself — where family matching sits in the
  route order, after safety and identity resolution and before the honesty
  fallback, is an owner-level decision about precedence, and this phase
  builds the guard that has to exist before products may enter the vessel
  ledger at all.*
  *Phase 3, 2026-09-05 — the router, decided and wired.* Precedence:
  `FamilyRouter` sits in `kerotakis-stack` immediately after
  `CuratedEquilibrator` and before every general engine. A curated pair is
  the more specific claim and answers first; the bench screens the operator
  for safety and resolves names at parse time before any solver sees the
  vessel, so the position is the IR's "after safety and identity, before
  honesty". The router (`kerotakis-core::family::FamilyRouter`, generic over
  `StructureOracle`) asks a record only when its pattern matches species in
  the vessel that have curated structures, in a fixed candidate order so
  pouring order cannot change the answer; the gates then decide and the first
  refusing gate names itself. Gate declines are quiet in the event stream
  (a lesson that never meant to esterify does not gain a line per step) and
  spoken through the solver's capability report; a product the registry
  cannot name is a typed `NotYetModeled` refusal where the gates admit
  (the vessel would run and the lab cannot name what forms), and a quiet
  decline behind a closed gate — citric acid beside a sugar's alcohol group
  matches the esterification pattern in every glass of lemonade, and six
  corpus rows said so before this rule. Outcomes:
  `to_completion` runs the limiting reagent; `equilibrium` solves the
  mole-basis quotient to K (bidirectional, so a mixture past K runs back —
  and added water pushes esterification back, computed); `kinetic_law`
  declines by name until BRD-050 admits a law. Records ship as data,
  `data/families/families-v1.toml`, linted on load; the chematic oracle
  learned to offer one fragment of a salt (`[Na+].[OH-]`) in a slot and carry
  the rest as named spectators, which is how the sodium survives
  saponification. Pack v1: Fischer esterification (H₂SO₄, ≥ 60 °C, K = 4)
  and alkaline ester hydrolysis (water-majority, ≥ 50 °C). The SN2/E2
  templates stay out of the pack until their haloalkanes have curated
  structures — a record that can never fire is documentation in a record's
  clothes. *Same day, found by the aqueous-tail tests (#395):* the tail keeps
  a strong base as `Na+` plus a positive `solute_charge`, never as an `NaOH`
  or `OH-` portion, so a record naming `NaOH` matched only on the step the
  base was poured. The router now carries two charge-backed keys — `OH-`
  from free alkalinity and `H+` from free strong acidity (the
  `unspent_acidity` convention) — as candidates and gate answers; consuming
  them moves no portion, the charge refresh after the deposited ions does
  the bookkeeping, and a balanced salt opens neither. The pack names `OH-`
  and accepts `H+` as the esterification catalyst. Spoken declines as events are the next slice, gated on being able
  to regenerate the lesson goldens.
- **Outcome:** one audited rule can apply a known transformation to structurally
  matching substrates without becoming an arbitrary predictor.
- **Scope:** versioned family records contain mapped reactant/product query,
  stoichiometry, required/forbidden functional groups, solvent/phase,
  temperature/pH/catalyst/light gates, competing-family priority, equilibrium
  or kinetic model reference, products/by-products, atom mapping, provenance,
  confidence and explicit refusal domain. Define deterministic conflict
  resolution and require the rule to name why it fired or declined.
- **Integration:** family matching occurs after safety and identity resolution,
  before generic honesty fallback. Products enter the normal vessel ledger and
  downstream PHREEQC/thermo/kinetics routes. Reuse exact stoichiometry and
  molecule conservation lint.
- **Acceptance:** serializer/schema tests, parser fuzzing, atom/charge/mass
  conservation, order independence, conflict fixtures, and a deliberately
  out-of-domain substrate that refuses rather than overgeneralizes.
- **Out of scope:** reaction planning, retrosynthesis, learned outcome
  prediction, or automatic extraction of rules from patents.

### BRD-021 — Indigo versus RDKit shipping spike

- [ ] **Status:** open/decision gate. **Size:** medium. **Depends on:** BRD-020.
- **Candidates:** Indigo (Apache-2.0) and RDKit (BSD-3-Clause). Both have
  browser-capable builds and reaction-template machinery; verify the chosen
  release, notices, wasm size, mobile builds and transitive assets. Primary
  docs: <https://lifescience.opensource.epam.com/indigo/> and
  <https://github.com/rdkit/rdkit/tree/master/Code/MinimalLib>.
- **Scope:** implement the same narrow C/wasm-facing spike for both: parse and
  canonicalize 100 structures, match 30 SMARTS queries, execute 20 mapped
  reactions, retain stereochemistry/isotopes/charges, serialize products, and
  survive malformed inputs/resource caps. Compare against current `chematic`.
- **Decision rule:** prefer the smallest engine that passes the chemistry
  corpus identically on native, browser, macOS and iOS. Keep the loser as a
  build-time differential oracle where useful. If neither passes, harden
  `chematic`; do not force an FFI adoption.
- **Acceptance:** checked-in benchmark/report with exact versions and licence
  inventory; no production dependency in this PR.
- **Out of scope:** GUI molecule drawing or general reaction prediction.

### BRD-022 — Runtime structure/reaction executor

- [ ] **Status:** blocked on decision. **Size:** large. **Depends on:** BRD-021.
- **Scope:** integrate the selected engine behind a Kerotakis-owned trait for
  canonicalization, substructure match, mapped transformation, depiction data,
  and stable error/resource-limit reporting. The trait prevents pack schemas
  from depending on toolkit-specific object formats. Cross-check official InChI
  where supported.
- **Integration:** native and wasm hosts expose identical result JSON; mobile
  builds link the same curated API surface; `kerotakis-org` retains ownership.
  No engine call may bypass reaction-family conditions or the safety pass.
- **Acceptance:** BRD-021 corpus plus differential tests against the non-selected
  toolkit; byte-stable canonical output where the format promises it; wasm and
  native limits reject adversarial structures deterministically; dependency and
  NOTICE lints pass.

### BRD-023 — Familiar organic reaction-family pack v1

- [ ] **Status:** open. **Size:** large/data-heavy. **Depends on:** BRD-014 and
  BRD-022.
- **Scope:** curate a first useful set driven by `EXP-36/41/42/46/50` and the
  curiosity corpus: acid/base behavior of functional groups, combustion,
  esterification/hydrolysis, alcohol oxidation, carbonyl tests/additions,
  carboxylate formation, amide hydrolysis at an honest level, addition and
  condensation polymerization exemplars, substitution/elimination only inside
  the documented selectivity matrix, and moisture-sensitive Grignard formation
  as a tightly bounded teaching case.
- **Integration:** thermodynamic/kinetic numbers use existing provenance and
  solver routes; template products are ordinary registered species or
  generated structures with an explicit property-coverage ceiling.
- **Acceptance:** at least 50 family/substrate cases and 25 negative/out-of-scope
  cases; every product is atom mapped and conserved; condition perturbation
  tests switch or suppress outcomes as documented; no unrestricted free-form
  synthesis endpoint appears in CLI, wasm, MCP or GUI.
- **Out of scope:** pharmaceuticals as a catalog objective, reaction-condition
  recommendation, yield optimization, and routes outside curriculum/household
  demand.
- **Checkpoint 2026-09-05 — the galvanic couple shipped.** The curiosity
  corpus books its rust prompts against this task even though the task text
  is about organic families, so the evidence is recorded here rather than
  renumbered. The bench could already rust iron (`kinetics::iron-corrosion`,
  KID-5, with a chloride catalyst so brine beats tap water); what it could
  not do was notice a lump of zinc lying against the nail, which that
  entry's own uncertainty note admits by saying the half-reactions and the
  cell that separates them are not resolved.
  `crates/kerotakis-core/src/corrosion.rs` resolves that much of it and no
  more: the lower-E° metal in contact is the anode, read off
  `displacement::SERIES` so the bench holds one activity series and not two;
  `BARRIERS` carries the stainless passive film and the paint film, keyed on
  the material recipe the lot came from; and both rules are enforced in
  `KineticReaction::can_run`, so a protected metal's corrosion reaction does
  not run rather than merely being described as protected. A companion
  kinetic entry `zinc-corrosion` (`2 Zn + O2 + 2 H2O -> 2 Zn(OH)2`) makes
  the sacrifice real. `crates/kerotakis-core/tests/corrosion.rs` pairs every
  protection assertion with the unprotected control that rusts under the
  same script. Deliberately no second rate model, no area ratio, and no
  atmospheric weathering — the copper patina is named as unmodelled.
  Rows answered by the new chemistry: mat-099 (the one that mattered — it
  used to rust its iron at full rate with untouched zinc lying against it),
  mat-020, mat-100, mat-105, mat-069 (this task), mat-014 (BRD-014) and
  mat-104 (BRD-070). mat-096 and mat-097 were already rusting and gain a
  verdict beside the extent. Four more rows move for a different and
  smaller reason — `Event::Corroded` joins the classifier's answering list,
  so aq-089, mat-006, mat-003 and mat-108 stop being called `missing` while
  printing an answer; mat-003 and mat-108 remain comparative questions their
  single-condition scripts cannot ask, which is an `expected` problem and not
  an engine one (see the triage in #389). `engine stood aside` falls 19 -> 5.
  Still open here: the whole organic family pack, which this checkpoint does
  not touch.

## Stage B3 — general thermodynamics and gas kinetics

### BRD-030 — Direct feos integration spike

- [x] **Status:** closed `go` (scoped) on `brd030/feos-spike` (2026-08-30).
  **Size:** medium. **Depends on:** BRD-012 and completed CAP-1 routing.
- **Checkpoint 2026-08-30 — decision: `go`, scoped and conditional.** Full
  report in `provenance/brd-030-feos-spike.md`; three-way fixtures and the
  disposable prototype in `spikes/brd-030-feos/` (its own `[workspace]`, not a
  member of this one). Nothing shipped: no workspace `Cargo.toml` change, no
  third-party parameter file committed, no `sources.toml` record. Five
  findings:
  1. **feos replaces nothing.** It ships no activity-coefficient model of any
     kind — no UNIFAC, NRTL, Wilson or UNIQUAC — so the Antoine + UNIFAC γ–φ
     route in `vle.rs`/`unifac.rs`, and everything built on it, stays. Its
     `feos_core::cubic` Peng-Robinson is a documented 234-line teaching
     example, not a replacement for THERMO-007.
  2. **What it adds is what `kerotakis-thermo` structurally cannot compute:**
     liquid density, enthalpy of vaporisation and critical points (no model
     exists for any fluid), the gases BRD-000/014 demand (CO₂, N₂, O₂, NH₃),
     the sixteen corpus fluids with no curated Antoine set, and the binaries
     outside the ten UNIFAC groups in `approved_table()` — including
     acetone–chloroform, the maximum-boiling azeotrope the bench cannot teach
     today.
  3. **wasm re-verified at 0.10.1: yes.** 55 pure-Rust crates with
     `default-features = false, features = ["pcsaft"]`; no pyo3, rayon,
     rusqlite, cc or libc. Upstream CI already ships a Pyodide/emscripten
     build of every model. One design constraint: `Parameters::from_json` and
     friends call `std::fs` unconditionally, so they compile for wasm and then
     fail at runtime — browser use must embed parameters with `include_str!`.
  4. **Parameter provenance is the real risk, not the code.** The published
     crate contains no `parameters/` directory, so BRD-031 must supply
     everything; the repository's `parameters/` tree carries **no licence
     statement at all**, so silence must not be read as clearance. DIPPR and
     UNIFAC are genuinely absent upstream (feos refuses to ship DIPPR by
     policy). Two files are excluded by name: `ideal_gas/poling2000.json`
     (transcribed from a McGraw-Hill book) and `multiparameter/coolprop.json`
     (CoolProp's MIT notice and per-fluid citations stripped — an upstream
     compliance defect worth reporting). One literature table *is* compiled
     into the crate: the Joback & Reid 1987 group coefficients in
     `src/ideal_gas/joback.rs`, ungated.
  5. **`FluidModel` must be fixed before BRD-032 routes anything.** The trait
     passes the Raoult model's own Antoine constants and γ through a
     model-agnostic seam, and its default `dew_point`/`tp_flash`/
     `saturation_pressure_kpa` bodies would make a feos backend answer those
     three questions with the ideal model *silently*. That is the
     fall-through this document forbids. A day's work in a 2 775-line crate.
  Conditions on the `go`, all owned by BRD-031: build for macOS and iOS (only
  native and `wasm32-unknown-unknown` were tested by the original spike); clear
  every parameter table independently; fix `FluidModel`; pin `=0.10.x` and
  embed parameters. PRs #274 and #279 subsequently discharged the silent
  inherited-method, exact-pin and target-build conditions. The model-neutral
  component-identity seam and independently cleared parameter-table condition
  remain open and continue to block runtime routing.
- **Candidate/licence:** `feos`, MIT OR Apache-2.0. It supplies PC-SAFT,
  ePC-SAFT, group-contribution/multiparameter models, phase equilibrium and
  transport calculations. Audit parameter-file provenance independently.
  Primary project: <https://github.com/feos-org/feos>.
- **Scope:** compare feos with `kerotakis-thermo` on 20 pure fluids and 20
  mixtures: density, vapor pressure, bubble/dew point, flash, enthalpy and
  critical point where applicable. Measure wasm size/time/memory and compile
  every release target. Prototype a Kerotakis adapter without changing routing.
- **Acceptance:** independent fixtures and discrepancy report; exact model and
  parameter source attached to every result; go/no-go decision names which
  existing models feos would replace, backstop, or leave alone.
- **Out of scope:** adopting feos merely because it has more models.

### BRD-031 — Cleared fluid parameter pack

- [ ] **Status:** in progress through independently reviewable checkpoints
  (2026-08-31), unblocked by the BRD-030 `go`; BRD-032 remains blocked on a
  cleared residual-fluid parameter pack.
  **Size:** large/data-heavy. **Depends on:** BRD-030.
- **Scope:** curate parameters for the fluids and mixtures actually demanded by
  BRD-000/014: water, common alcohols/ketones/esters/hydrocarbons, CO2, air
  gases, ammonia, light fuels and selected refrigerants. Every parameter set
  records its original publication/data licence and model validity range.
- **Carried in from BRD-030 (2026-08-30):** scope is limited to the properties
  `kerotakis-thermo` cannot compute at all — density, enthalpy of
  vaporisation, critical points, the air gases and CO2/NH3, and fluids or
  UNIFAC groups outside the curated tables. Extending `vle.rs`'s Antoine set
  and `unifac.rs`'s `approved_table()` by hand remains the cheaper answer for
  the *binary* gap and should be preferred where it suffices. Excluded by
  name: feos's `parameters/ideal_gas/poling2000.json` and
  `parameters/multiparameter/coolprop.json`. feos's `parameters/` tree carries
  no licence statement, so every table needs its own record reasoned from the
  primary publication; the Joback & Reid 1987 table compiled into
  `feos/src/ideal_gas/joback.rs` needs a NOTICE entry if the crate is adopted.
  PR #274 removed `FluidModel`'s silent inherited methods, made capabilities
  explicit, and fixed missing UNIFAC interactions and converged-domain checks.
  The trait still accepts Raoult-shaped `Volatile` values, so the historical
  spike's positional component-identity seam remains unresolved. PR #279 pins
  feos and feos-core exactly at `=0.10.1`, checks in the disposable adapter
  lockfile, keeps the browser-facing adapter library free of filesystem
  parameter loading, and compiles that library in CI for wasm32, Android,
  Windows, macOS and iOS. These close named fail-closed/pin/target hazards, not
  the identity seam or independent rights review for residual parameters.
- **Integration:** join by canonical species identity; model selection is
  explicit and inspectable. Missing binary parameters produce a named refusal
  or a labelled lower-fidelity route, never silent ideality.
- **Acceptance:** coverage matrix and one literature/oracle fixture per model
  family; no proprietary DIPPR, NIST SRD/WebBook, UNIFAC Consortium, or
  otherwise encumbered parameter enters the pack.
- **Checkpoint plan and DoDs (2026-08-31):** this is deliberately a sequence
  of independently reviewable deliveries, not one data dump. Estimates are
  engineering effort, not elapsed promises.
  1. **BRD-031a — fail-closed fluid contract (2–3 h).** Remove inherited
     calculations from `FluidModel`, declare capabilities, and distinguish a
     named unsupported operation from a supported calculation with no
     numerical solution. **DoD:** every operation is implemented explicitly;
     a deliberately partial trait object refuses by operation and model name;
     all thermo targets and clippy pass.
  2. **BRD-031b — current-solver domain safety (4–6 h).** Enforce the common
     Antoine fit interval in bubble, dew, TP and HP flash; reject nonfinite
     inputs/coefficients; make a missing directional UNIFAC interaction a
     checked, named error. **DoD:** endpoint, just-outside, disjoint-range,
     malformed-gamma, missing-forward and missing-reverse tests pass; existing
     literature fixtures remain within their stated domains; no missing
     interaction can become `psi = 1` silently.
  3. **BRD-031.S01 — six-fluid pilot pack (4–6 h).** Curate only water, CO2,
     N2, O2, NH3 and one common alcohol, using the smallest feos model family
     that answers a currently missing property. **DoD:** each field has a
     canonical species key, units, validity range, primary citation, licence,
     retrieval date and checksum; an importer emits a quarantine report and
     an allowlist emits deterministic embedded records; any unclear data right
     is a recorded rejection, never an inferred permission.
  4. **BRD-031d — disposable feos adapter evidence (3–5 h).** Consume the
     embedded pilot records with pinned feos `=0.10.x`; keep routing unchanged.
     **DoD:** native and wasm compile/run fixtures pass; macOS and iOS targets
     compile in CI or remain explicitly unproven (never inferred from Metal or
     WebGPU support); package size/dependency/licence reports are checked in;
     filesystem parameter loading is absent from browser/mobile paths.
  5. **BRD-031e — integration audit (2–4 h).** Join only by canonical species
     identity and compare the pilot against an independent fixture per model
     family. **DoD:** conservation/metamorphic checks, exact model/parameter
     provenance in every result, repository policy gates, replay determinism,
     and protected-main CI all pass. Only then may BRD-032 be unblocked.
- **S01 source gate (2026-08-31):** the initial rights audit is a runtime
  promotion `no-go`; see `provenance/brd-031-pilot-source-audit.md`. feos and
  CoolProp have technically adequate six-fluid candidates under permissive
  repository licences, but neither gives an explicit path-level assurance for
  the third-party-derived numerical parameter tables. Work therefore proceeds
  only on a quarantine importer with synthetic fixtures. No candidate value is
  copied, no pack is embedded, and BRD-032 remains blocked.
- **Delivered checkpoint ledger (2026-08-31):** PR #274 merged the fail-closed
  fluid contract and current-solver safety work; PR #279 merged the exact feos
  adapter pin/lock and five-target CI matrix; PR #278 merged the six-fluid
  quarantine importer, canonical identity join, licence/unit refusals and
  synthetic PC-SAFT-shaped fixture. PR #289 merged a dedicated
  `MolecularLength` dimension so reviewed segment diameters cannot be confused
  with optical wavelengths. PR #291 merged canonical access to the already
  vendored NASA-9 ideal-gas records for the six pilot identities. None promotes
  third-party residual parameters or changes runtime model selection.
- **Open checkpoint ledger (do not treat as shipped):** PR #290 proposes a
  piecewise ethanol vapour-pressure route, preserving the existing
  low-temperature correlation and adding an explicitly CC-BY-4.0 experimental
  fit for 79.65–151.95 °C.
  PR #293 proposes pinned-manifest/checksum verification, a deterministic
  offline importer CLI/report, and parser fuzzing. These are open review
  checkpoints. In particular, NASA ideal-gas heat-capacity/enthalpy/entropy
  data complement a residual EOS; they are not PC-SAFT segment, dispersion,
  association or binary-interaction parameters and therefore cannot unblock
  feos routing.
- **Permissive-source result (2026-08-31):** this repository records no exact
  source bytes and grant that clear a direct PC-SAFT parameter table covering
  all six pilot fluids. That is the auditable result here; it is not evidence
  of an exhaustive external search. NIST WebBook/ThermoML, journal tables, and
  permissively licensed software repositories are not relabelled as open
  numerical data. An agent may generate a table only from exact source bytes
  whose grant covers the numerical fields, with per-field provenance and a
  reproducible checksum; it may not reconstruct or transcribe the currently
  blocked candidates under a new label.
- **Identity-seam checkpoint (2026-09-05):** the model-neutral
  component-identity condition carried over from BRD-030 is discharged.
  `crates/kerotakis-thermo/src/pack.rs` holds the nine fluids the curiosity
  corpus and Kids Lab actually reach for — water, ethanol, methanol,
  propanone, isopropanol, ethanoic acid, ethyl acetate, hexane, CO2, N2, O2 —
  as rows keyed by **Standard InChIKey**, and `row_by_inchikey` is the only
  lookup the module offers, so the spike's `"WATER" => vle::WATER` name match
  has no successor. `crates/kerotakis-core/tests/fluid_pack_identity.rs`
  proves the join from the side that can see both halves: each row resolves
  to exactly one registry species, agrees with it in both directions, and
  neither the registry key nor the display name selects a row.
  Provenance is per *parameter*, not per row, and the ethanol row is why —
  its two correlation segments sit in two different rights lanes, which one
  `source` string on the row would have lost. `lint_row` refuses a row whose
  segment or provenance carries no source, no locator, no licence or no ISO
  date; whose open-lane licence is outside the audit's allowlist; whose
  provenance count does not match its segment count; or which leaves a
  parameter neither cleared nor declared a gap. It is run in both directions,
  the shipped pack passing and a deliberately sourceless row being refused.
  **Nothing is newly cleared, and the S01 `no-go` stands.** Liquid density
  and residual-EOS (PC-SAFT) parameters are absent for *every* fluid and each
  row says so by name, citing
  `provenance/brd-031-pilot-source-audit.md`; asking for either returns a
  typed refusal rather than a number. Five of the six shipped Antoine sets
  are recorded in the new
  `PrimaryLiteratureCoefficientsPendingReview` lane, which is a statement
  that an independent rights review still owes them an answer, not a
  clearance. One finding fell out of writing the records down: the
  isopropanol coefficients in `vle.rs` were transcribed from the NIST
  Chemistry WebBook's rendering of Stull 1947, and NIST WebBook is a rejected
  source class in this task's own audit. The row now carries the primary
  citation and the detour in writing.
- **Open-source search checkpoint (2026-09-05):** the "replacement dataset
  under an explicit open licence" route named in the S01 clearance list has
  been searched for the first time and the result is recorded in
  `provenance/brd-031-pilot-source-audit.md` § Addendum. Seven CC BY 4.0
  articles between them carry PC-SAFT parameters for all nine fluids,
  including the only clean sources found for oxygen (Staubach et al., IJT
  2023) and ethyl acetate (Molecules 2016). **They are candidates, not
  clearance, and nothing is promoted:** the numbers are second-hand, mostly
  republished from Gross & Sadowski's closed papers, and two of them
  disagree with each other on ethanol — which is itself the argument for
  putting them through the existing quarantine path rather than typing them
  in. Three named candidates were rejected on their licences (the Esper
  IECR database is closed; the FeOs paper is CC BY-NC-ND; arXiv 2309.12404
  is CC BY-NC-SA), and the ML-SAFT Zenodo deposit was rejected because its
  own README contradicts its CC BY stamp with a Dortmund Databank
  proprietary notice. A CC BY-stamped critical-constant compilation was
  likewise recorded and refused, because its README names Perry, Yaws, VDI,
  NIST WebBook and the CRC Handbook as its sources: a depositor's licence
  stamp does not launder upstream rights, and accepting it would make every
  rejection above meaningless. **Saturated liquid density remains
  unsourceable** under any accepted licence, which is why the pack refuses
  it for every fluid rather than shipping the registry's 25 °C constant
  dressed as a correlation.

### BRD-032 — feos-backed bench routing

- [ ] **Status:** first slice shipped (2026-09-05); the residual-EOS half
  remains blocked on BRD-031's uncleared parameter pack. **Size:** large.
  **Depends on:** BRD-031.
- **Pressure-dependent boiling checkpoint (2026-09-05):** the bench boiled
  water at 100 °C in a vacuum flask and at 100 °C in a pressure cooker,
  because `states::transitions` took a molality and nothing else. That is
  the defect shape this document warns about — a quantity claimed to depend
  on X that does not move when X does — and the corpus asks about it
  directly in th-019 and th-020.
  `states::transitions_at` now takes the vessel's own pressure and returns
  the boiling point **with the route that set it**. The route resolves the
  solvent's parameters through the BRD-031 pack **by InChIKey**, inverts the
  cleared saturation-pressure correlation by bisection inside its own fitted
  bracket, and composes the answer as
  `T_b(P) = T_b(1 atm) + [T_fit(P) − T_fit(1 atm)]`. The anchoring is the
  design decision worth reading: Stull's water fit reproduces the normal
  boiling point to 0.003 K rather than to zero, so anchoring keeps every open
  vessel — which reports exactly one atmosphere — bit-for-bit unchanged, and
  leaves the correlation doing the one job it is better at than a table.
  **No refusal is weakened and one is added.** Water's shipped fit spans
  0.65–101.34 kPa, so a vacuum flask routes and a pressure cooker does not;
  above that window the curated boiling point stands and a new
  `Event::BoilingPointRouted` *says so by name*, in all three registers and
  in German. A silent fall-through would have been indistinguishable from a
  modelled answer, which is precisely what BRD-030 finding 5 forbids.
  `Transitions` now reports the colligative and pressure shifts separately,
  because "higher than pure water because of what is dissolved in it" and
  "lower because the vessel is under vacuum" are different sentences and
  folding them together would make the first one lie.
  Proved by `crates/kerotakis-core/tests/pressure_boiling.rs` (open beaker
  unchanged to 1e-12 and emitting no routing event at all; 50 kPa boils at
  81.4 °C against the steam tables' 81.3; 200 kPa refuses by name;
  monotonicity across the routed window; scale invariance; the two shifts
  reported separately) and by `kerotakis-thermo`'s own pack tests
  (monotonicity and inverse-consistency for each of the six cleared fluids,
  1 atm within 0.4 K of each substance's normal boiling point, a
  pressure-shaped refusal outside each window).
  Still refused, and named: liquid density for every fluid; saturation
  pressure for CO2/N2/O2/hexane/ethyl acetate; every residual-EOS route.
  The `measure <vessel> boiling_point` apparatus is **not** routed — it lives
  in `crates/kerotakis-core/src/bench.rs`, outside this task's boundary — so
  the other five cleared fluids are answered by the pack API and not yet by
  a bench surface. That is the next BRD-032 slice and it is small.
- **Scope:** route pressure-dependent boiling/condensation, flash, phase split,
  density and transport-property requests through the adapter when the exact
  parameter/model domain is present. Preserve existing UNIFAC/cubic routes as
  named alternatives and expose model disagreement through `explain`.
- **Integration:** `heat`, `cool`, `distil`, `evaporate`, `drain`, sealed
  headspace, rotovap/reduced-pressure behavior, charts, CLI and wasm.
- **Acceptance:** conservation and scale invariance; pressure monotonicity for
  boiling where valid; azeotrope/phase-split goldens; identical host results;
  BRD-000 phase-change coverage increases without weakening honest refusals.

### BRD-040 — Cantera mechanism and API audit

- [x] **Status:** complete on `brd040/cantera-audit` (2026-08-29). **Size:**
  medium. **Depends on:** BRD-012 and current Cantera-YAML/kinetics support.
- **Checkpoint 2026-08-29:** full report in
  `provenance/brd-040-cantera-audit.md`; machine-readable licence verdicts in
  `provenance/sources.toml`; rejection matrix executed by
  `crates/kerotakis-core/tests/mechanism_cantera_audit.rs`. No runtime FFI and
  no mechanism file shipped, per the acceptance criterion. Four findings:
  1. **Parser bugs fixed.** Reaction orders were derived from *net*
     stoichiometry rather than each side of the equation, giving a wrong rate
     order and a mis-scaled pre-exponential wherever a species appears on both
     sides (6 of 29 reactions in Cantera's `h2o2.yaml`, 18 of 325 in
     `gri30.yaml`). The default activation-energy unit ignored the `energy` and
     `quantity` directives, misreading `units: {quantity: mol}` by 10³. Phase
     reaction selectors and named reaction sections were ignored. Unknown keys
     — including `orders`, `SRI`, `Tsang`, `negative-A` and nested `units` —
     were silently dropped; all six raw structures now refuse anything outside
     a documented allowlist.
  2. **Smallest additional subset for BRD-041 is three rate-law items** —
     reversible three-body reactions, reversible falloff (Troe and Lindemann),
     and negative activation energies — **plus one piece of document handling**:
     select a phase rather than validating every phase in the file, since real
     mechanism files pair an ideal-gas phase with a real-gas variant of the same
     species. With those, H₂/O₂, N₂/NOₓ and CH₄ + CO teaching mechanisms are
     fully expressible. PLOG is already supported and unused by them; Chebyshev,
     NASA9, explicit orders and plasma rates are not needed.
  3. **Licence verdict: every audited mechanism is oracle-only.** None of
     GRI-Mech 3.0, Ó Conaire, Boivin, the syngas sets, FFCM-1 or San Diego
     carries a redistribution grant, and Cantera states it "is not claiming to
     grant a license to" the mechanisms it ships. PLAN.md's claim that those
     files are BSD-3 redistributable is **wrong** and is corrected in this
     change. BRD-041 must author its own reduced mechanisms from primary
     literature, find a genuinely CC-licensed one, or obtain written permission.
  4. **C-API verdict: no gap; BRD-042 stays parked.** Nothing BRD-041 needs
     requires Cantera's C API, and linking it would not touch the licensing
     problem that actually blocks the mechanism packs.
- **Candidate/licence:** Cantera BSD-3-Clause **for its code only**; the shipped
  mechanism files carry no grant (audited 2026-08-29). Mechanism files and their
  original provenance/licences require separate review. Primary project and licence:
  <https://github.com/Cantera/cantera> and
  <https://github.com/Cantera/cantera/blob/main/License.txt>.
- **Scope:** inventory the current parser against Cantera YAML rate-law,
  thermo, transport and reactor features; identify the smallest additional
  subset needed for familiar fuels and atmospheric examples. Audit candidate
  mechanisms before any data import. Record gaps that truly require Cantera's
  C API rather than extending the portable slice.
- **Acceptance:** feature/fixture/licence matrix, parser rejection tests for
  unsupported YAML, and an ordered list of reduced mechanisms. No runtime FFI
  or new mechanism ships in this task.

### BRD-041 — Familiar gas/combustion mechanism packs

- [ ] **Status:** packs shipped 2026-09-05 (PRs #393, #399); three acceptance
  items remain, listed in the checkpoint below. **Size:** large/data-heavy.
  **Depends on:** BRD-040 (complete).
- **Checkpoint 2026-09-05 — three project-original packs, from primary
  literature.** BRD-040's route of last resort turned out to be the only one:
  the networks are written here, reaction by reaction, and every reaction
  records the recommendation it came from, the page it was read from, the range
  that recommendation covers, its stated uncertainty, and the date it was read.
  A test refuses any reaction missing one of those. `data/mechanisms/`:

  | File | Steps | Kind | Source |
  |---|---|---|---|
  | `h2-o2-skeletal-v1.yaml` | 16 | skeletal, elementary | Baulch et al., *J. Phys. Chem. Ref. Data* **34**(3), 757–1397 (2005) |
  | `co-h2-wet-v1.yaml` | 20 | skeletal, elementary | the same evaluation |
  | `hydrocarbon-global-v1.yaml` | 3 | **global**, not elementary | Westbrook & Dryer, *Combust. Sci. Technol.* **27**(1–2), 31–43 (1981), Table I |

  The two skeletal packs draw every coefficient from **one** evaluation, on
  purpose: a mechanism assembled from a single self-consistent source is a
  defensible object, one assembled from whichever paper gave the prettiest
  curve per step is not. Where that evaluation recommends nothing, the step is
  absent and the header says which and why. Every value was read from the
  published pages twice, independently, and cross-checked — 15 expressions, 15
  exact matches — and three publication errata in the source were identified
  and are not propagated, most consequentially the `OH + CO` low-pressure
  coefficients, printed as cm⁶ molecule⁻² s⁻¹ in two places when they are the
  bimolecular CO₂ + H branch in cm³ molecule⁻¹ s⁻¹. The packs are
  Kerotakis-authored data, CC BY 4.0, and carry their licence in-file;
  `provenance/sources.toml` is scoped to vendored external material and
  correctly has no record for them.
- **Two BRD-040 guards moved, and one BRD-040 conclusion was wrong.**
  Negative activation energies are accepted (§7 item 3, as recommended):
  `OH + OH → H₂O + O` and the slow `HO₂ + HO₂` term both have positive
  exponentials in the evaluation, and `CO + OH` is barrierless. A NaN `Ea` is
  still refused by name. Explicit reaction `orders` are accepted too — which
  §7 had listed as *not* needed. That was written assuming every fuel would be
  an elementary set; the light hydrocarbons have no licence-clean skeletal
  mechanism and no evaluation of their elementary steps at the level Baulch
  reaches for H₂/O₂ and CO, so they are global steps, and a global step's
  orders are measured separately from its stoichiometry. Methane's fitted fuel
  order is **negative** and its equation-derived order would be third where the
  fit is first. `nonreactant-orders` stays refused, which is why the two-step
  Westbrook–Dryer form is not shipped: its CO step depends on water it neither
  consumes nor produces. Corrections recorded in
  `provenance/brd-040-cantera-audit.md`; a `mechanism_yaml` fuzz target was
  added with the relaxations and is honestly marked *not yet run*.
- **Corpus evidence, and who actually earned it.** All 36 BRD-041-owned rows
  are out of `missing`: 29 `computed`, 7 `boundary` with typed reasons (four
  `weaponization`, `unsafe-toxic-combustion`, `unsafe-thermal-runaway`,
  `out-of-scope-scale`). The three that were missing — `th-044` methane,
  `th-045` propane, `th-046` butane — failed at `unknown-species`, a registry
  identity gap rather than a kinetics gap, and were closed by **PR #388**'s
  fuel-gas identities, after which CEA's Gibbs minimiser reaches them
  unaided. This task did not flip them and does not claim to;
  `tests/coverage/curiosity-v1/baseline.toml` is untouched by #393 and #399.
  BRD-012.S04's note below says the same thing from the other side and is
  right: three prompts reaching an equilibrium solver is not twenty-five
  prompts reaching a reviewed reduced mechanism. `th-047` petrol and `th-048`
  diesel remain `missing` and belong to BRD-014: they are mixtures, and no
  mechanism here or anywhere else in the tree names them.
- **Engine finding: the extent integrator cannot carry a radical chain through
  ignition.** `kinetics_integrator.rs` hands diffsol a matrix-free Jacobian
  whose finite-difference probe is one scalar for the whole extent vector,
  sized from `(1 + ‖x‖∞)`. A radical chain has extents spanning nine orders of
  magnitude at once, so a probe sized for the millimole extents linearises the
  nanomole ones across their entire range; the Newton iteration fails and the
  H₂/O₂ pack exhausts its failure budget at about 2.7 µs, right where the chain
  runs away. The `.max(0.0)` clamp on reconstructed amounts puts a second
  corner in the same right-hand side. The skeletal packs are therefore
  exercised over a bounded half-microsecond early-chain window and their
  endpoint claims are made against CEA thermodynamics instead of by
  integrating to one. Two exits for whoever takes it further: a
  component-scaled Jacobian probe, or the CVODE path that already exists in
  `kerotakis-sundials` and is API-compatible with
  `advance_network_with_options`. The three-reaction global pack is unaffected
  and integrates to exhaustion, which is where the endpoint oracle lives.
- **Acceptance, item by item.** Element and energy conservation: **met** —
  balance is enforced by the parser and re-checked on the compiled network, and
  every global step priced through CEA's formation enthalpies releases
  400–410 kJ per mole of O₂, the band all hydrocarbon oxidation shares.
  Equilibrium endpoints: **met** — lean methane ends within 2 % of CEA at
  1600 K, and where equilibrium is *not* complete combustion (rich methane at
  2000 K) the global step's inability to make CO is itself a passing test.
  Rich/lean and temperature/pressure metamorphic cases: **met** — more oxygen
  burns more hydrogen; doubling the oxygen speeds each global fuel up by
  exactly its own fitted power; doubling the methane makes it *slower*, by
  exactly 2⁻⁰·³. Still open: (1) **no Cantera differential oracle** for
  ignition delay or species traces — blocked behind the integrator finding
  above, not behind licensing, since running a mechanism outside the repository
  distributes nothing; (2) **wasm runtime is unbounded/unmeasured**; (3) the
  "at least 25 curiosity prompts graduate from missing to computed" floor is
  **not met, and cannot be met as counted** — only three BRD-041 rows were ever
  `missing`, and BRD-012.S04 closed those through CEA equilibrium. The
  criterion's real content is the clause after it, *through a reviewed reduced
  mechanism*, and that is item (4).
- **(4) Not routed into the bench, and the reason is not laziness.** The packs
  are reachable through `kero mechanism inspect|rates|simulate` and through the
  tests; nothing in `ignite`, `heat` or `wait` reaches them. The bench's
  kinetic route is `wait` over the static curated `REGISTRY`, and `ignite` goes
  through the equilibrator stack, so wiring a pack in means adding pack loading
  to the engine and touching `solve.rs` or `bench.rs` — both outside this
  round's boundary. But the deciding reason is the integrator finding above:
  a skeletal pack cannot currently be integrated through its own ignition
  transient, so routing `ignite` to it would replace an answer CEA already
  gets right with one that fails at 2.7 µs. **Fix the integrator first, then
  route.** Until then the honest arrangement is the one shipped: the packs are
  reviewed data with an oracle against them, and `ignite` keeps saying what
  Gibbs minimisation can prove.
- **Deliberately absent, and said so in each file.** Alcohol fuel exemplars.
  Nitrogen chemistry of any kind: N₂ is a diluent and third-body collider, and
  the absence of NO is not a claim that none forms. Soot, luminosity and flame
  colour. Falloff — every third-body step is a low-pressure limit, with the
  evaluation's k∞ and F_c unused. Transport, flame structure and flame speed.
  `OH + HO₂ → H₂O + O₂` between 400 and 1300 K, where the evaluation declines
  to recommend anything. And any hydrogen-free route from CO to CO₂, since the
  evaluation recommends none — a dry vessel does nothing in that pack, which
  understates a slow reaction rather than inventing a fast one, and must not be
  read as a claim that dry carbon monoxide cannot burn.
- **2026-09-05, BRD-012.S04:** the three fuel gases the corpus asks about —
  methane, propane and butane — are now registry species, so th-044, th-045
  and th-046 run instead of failing at the parser. What they run through is
  the **CEA equilibrium** path, not a mechanism: composition-matched NASA-9
  records give an equilibrium endpoint and an energy, and that is the whole
  of it. No rate, no ignition delay, no flame. The sourcing decision above
  is untouched by this, and the acceptance criterion — twenty-five prompts
  reaching *computed* through a reviewed reduced mechanism — is not met by
  three prompts reaching an equilibrium solver. Recorded here so the next
  reader does not mistake the closed rows for a mechanism pack.
- **BRD-040 finding (2026-08-29):** *no* audited mechanism may ship as
  runtime-data — not GRI-Mech 3.0, Ó Conaire, Boivin, the syngas sets, FFCM-1 or
  San Diego. None carries a redistribution grant, and Cantera states it "is not
  claiming to grant a license to" the mechanisms it ships. Three routes remain,
  in order of preference: author project-original reduced networks from
  primary-literature rate constants with per-reaction source records (the
  pattern KIN-001…003 already uses); find a mechanism under a real open licence;
  or obtain written permission recorded as a `LicenseRef-` grant. The parser
  work is small by comparison — three rate-law items and phase selection. Full
  reasoning and the ordered candidate list: `provenance/brd-040-cantera-audit.md`.
- **Scope:** add reviewed reduced mechanisms for hydrogen/oxygen, methane,
  carbon monoxide, selected light hydrocarbon/alcohol fuel exemplars, and
  nitrogen chemistry only where the mechanism licence and educational need are
  clear. Add soot/yellow-flame narration only when backed by an explicit model;
  otherwise state that luminosity/particles are outside the gas mechanism.
- **Integration:** `ignite`, sealed/open headspace, CEA equilibrium, diffsol
  kinetics, spectrophotometer/flame appearance where computed, heat ledger,
  emissions and safety events.
- **Acceptance:** Cantera differential oracle for ignition delay/species traces
  and equilibrium endpoints; element/energy conservation; rich/lean and
  temperature/pressure metamorphic cases; bounded runtime in wasm; at least 25
  curiosity prompts graduate from missing to computed.

- **Routing checkpoint (2026-09-05, Fable):** the three packs are compiled
  into the engine (`kinetics::packs`), renamed onto registry keys where the
  registry has one (`CH4`→`methane`, `H2O`→`water`), and run by the slow
  clock as `gas-mechanisms` on any vessel holding two or more of a pack's
  species as gas — with the heat of what burned applied from the vendored
  NASA formation enthalpies, cited per species. Two integrator defects had
  to go first (#404): a Jacobian probe sized by extents rather than by the
  species it moved, and a depletion gate that flipped a radical's reactions
  off at ten nanomoles. The hydrogen pack now integrates through ignition
  in one call. **Precedence, stated honestly:** `cea-thermal` still claims
  every gas vessel at or above 500 K on the equilibration step itself, so in
  the standard stack a fuel/air mixture is burned to equilibrium by CEA the
  moment it is hot, before any `wait` can hand it to a pack; the packs are
  reached today in CEA-less stacks and in the same `wait` step that CEA then
  finishes. Radicals and CO enter
  the ledger by name without a registry identity; CO is a registry gap.
- **Precedence checkpoint (2026-09-05, Fable):** `cea-thermal` no longer
  burns a warm fuel that has not been sparked. A table of autoignition
  temperatures in air (`combustion::GAS_AUTOIGNITION`, H₂, methane, propane,
  butane, CO, ethanol, methanol, propanone, hexane; Zabetakis 1965 as
  commonly tabulated, pending-review lane) gates the thermal solver: a fuel
  standing with oxygen below its autoignition temperature is answered with a
  typed `BelowAutoignition` event — "warm but not hot enough to catch; needs
  X °C or a spark" — and left alone. `ignite` takes a vessel to 1200 K, above
  every row, so a spark still burns; heating past the autoignition point
  burns too, which is what autoignition means. Below the point, the `wait`
  clock hands the mixture to the mechanism packs, whose rate laws are zero
  there without a radical source — the honest "nothing happens" a real bench
  shows. The classifier counts `BelowAutoignition` as an answer beside
  `Inert`, never above a computed or curated route.

### BRD-042 — Full Cantera C-API shipping gate

- [ ] **Status:** parked — BRD-040 recorded a **no-go** on 2026-08-29. **Size:**
  extra large. **Depends on:** BRD-040 (complete) and a stable upstream C API on
  all targets.
- **BRD-040 finding:** no BRD-041 need requires the C API. The portable parser,
  diffsol, the CEA equilibrium path and the existing apparatus models cover
  every item in BRD-041's acceptance criteria; the only capability the portable
  path lacks is mixture transport, which BRD-041 does not ask for and which is a
  self-contained kinetic-theory calculation rather than a reason to link a C++
  engine. Linking Cantera would also not touch what actually blocks BRD-041,
  which is that no candidate mechanism carries a redistribution grant. Re-open
  this task only on a required capability that is genuinely infeasible in Rust.
  Reasoning in `provenance/brd-040-cantera-audit.md` § 6.
- **Scope:** compile a minimal handle-based API for desktop, wasm and mobile;
  compare binary size, startup, determinism, resource limits and answers with
  the portable Kerotakis mechanism path. Keep one engine instance per worker.
- **Acceptance:** all release targets and offline packaging pass; no C++ types
  cross the boundary; exact licence/NOTICE/SBOM; measurable capability gain
  that BRD-041 cannot reasonably supply. Otherwise record a no-go and close.
- **Out of scope:** replacing the portable path solely for solver prestige.

## Stage B4 — biochemistry and crystalline matter

### BRD-050 — Bounded biochemical reaction IR and router

- [ ] **Status:** open, with a bounded first route shipped. **Size:** medium.
  **Depends on:** BRD-011, BRD-020 and existing reaction-network kinetics.
- **2026-09-05, the bounded biochemical route.** The acceptance line asked for
  a pH/T window, a documented denaturation model and an enzyme that is not
  consumed. Two of the three now exist for the food enzymes, and the eight
  curiosity rows that were blocked on them ran. **Evidence, row by row:**
  - **A pH window per catalyst.** `enzyme::FAMILIES` carries an optimum and a
    width in pH beside the temperature pair, read from the vessel's solved
    solution as a second Gaussian. Pepsin sits at pH 1.8, bromelain at 5.5,
    lactase at 6.5, lipase at 8, catalase and the generic protease near
    neutral. `bio-049` digests protein at pH 1 and `bio-050` does not at pH
    13 — the hydrolysed mass there falls below the observable floor, so no
    event fires at all and the answer is the absence.
  - **A denaturation model, and it is irreversible.** A material's carried
    enzyme held above the recipe's `denatures_above_k` is marked, and cooling
    does not bring it back. `bio-053` (cooked pineapple sets a jelly) is that
    one number.
  - **The enzyme is not consumed, and now a FOOD can carry one.**
    `MaterialRole::EnzymeSource` was the missing bridge: a recipe component
    must be a registry identity and an enzyme deliberately is not, so no food
    could bring its own catalyst. The role declares an ACTIVITY EQUIVALENT in
    the model's own dose units per gram of material, never a mass of enzyme
    in the food. `bio-052` runs on it.
  - **Substrate class, not enzyme name, joins the two tables.** Three
    proteases cut the same peptide bond and differ only in where; when more
    than one is in the beaker their rates ADD over one shared pool.
  - **Bounded fermentation gains three metabolisms.** Homolactic,
    heterolactic and acetic beside the alcoholic route, each one balanced
    aggregate equation conserving mass exactly, each with its own temperature
    envelope. `bio-071` makes vinegar from ethanol and oxygen, and stops when
    the oxygen does. `bio-073` makes acid and gas out of the same sugar.
  - **What is still missing, precisely.** There is no reaction IR and no
    `Biochemical` solver route: these are two hand-written models with typed
    parameters, not the composable representation this task names. No
    Michaelis–Menten, no cofactors, no inhibition, no compartments, no
    directionality. `bio-069`/`bio-070` (yoghurt) run a real fermentation and
    still cannot answer, because lactic acid cannot be speciated by any
    database this lab loads and milk resolves only water, so no solution is
    characterised and the pH meter reads nothing. And the lactic and acetic
    routes emit no typed event of their own: the clock arm builds `Fermented`
    from the sucrose/ethanol/CO2 fields, so they report through the vessel
    inventory. Closing those needs a lactate species in a loaded database (or
    milk's minerals resolved) and one more arm in `clock.rs`.
- **Outcome:** familiar biochemical reactions can be represented without
  pretending PHREEQC or the organic family router models a living cell.
- **Scope:** extend/compose the reaction-family IR with enzyme identity,
  compartment/environment, pH/T window, cofactors, directionality, kinetic law
  (`mass_action`, Michaelis–Menten and inhibition forms), aggregate
  macromolecule bookkeeping, and a declared abstraction level. Define a
  `Biochemical` solver route and explicit boundary against medical claims.
- **Acceptance:** catalase, amylase and fermentation exemplar networks conserve
  declared moieties/elements; enzyme is not consumed unless the model says so;
  outside pH/T/compartment refuses or applies a documented denaturation model;
  `explain` labels the curated biochemical abstraction.
- **Out of scope:** cell biology simulation, diagnosis, pharmacology, complete
  metabolism, or sequence-to-function prediction.

### BRD-051 — Rhea reaction adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-050 and BRD-003.
- **Source/licence:** Rhea CC BY 4.0; ChEBI supplies participant identities.
  Primary licence: <https://www.rhea-db.org/help/license-disclaimer>.
- **Scope:** ingest selected balanced reaction equations, directionality,
  participant ChEBI IDs and enzyme cross-references into quarantine. Map
  protonation/compartment assumptions explicitly. Do not infer rate constants,
  physiological occurrence, safety or lesson suitability from Rhea membership.
- **Acceptance:** pinned-release fixture, equation/charge conservation, stable
  ChEBI identity joins, attribution propagation, and rejection reports for
  polymers/generic participants or unsupported compartments.

### BRD-052 — Familiar biochemistry pack v1

- [ ] **Status:** open. **Size:** large/data/content. **Depends on:** BRD-051,
  BRD-012 and EXP-51's kinetic family.
- **2026-09-05, vocabulary only.** Ten of this task's curiosity rows stopped in
  the PARSER rather than in a solver, and BRD-014.S03 gave them their words:
  meat, bile salts, a green leaf, pondweed, a dry seed, a germinating seed, a
  celery stem, a wilted lettuce leaf and an onion epidermis. **Exactly one of
  them gained a mechanism.** `bio-051` now hydrolyses muscle protein, because
  `food/meat` carries an enzyme-activity profile and the prompt adds the
  protease itself; and `bio-085` emulsifies fat, because bile salts really are
  surfactants and the bounded emulsifier role is the right shape for them. The
  other eight run and answer nothing: **photosynthesis, respiration,
  transpiration, turgor and plasmolysis are all absent**, and each recipe's
  own `lot_assumptions` say which one it is missing. Those rows are closed
  against `unknown-species`, not against the question, and the corpus README
  records the distinction row by row. What they still need is listed there
  and is not a data problem.
- **2026-09-05, closed by BRD-050's bounded route.** The four rows this entry
  held open — `bio-052`/`bio-053` (pineapple and gelatine) and
  `bio-049`/`bio-050` (pepsin in acid and in base) — are answered. The
  recipe-to-catalyst bridge and the pH window are described under BRD-050
  above. Four more rows moved with them: `bio-071` (vinegar) answers,
  `bio-073` (sourdough) computes and is graded on a true remark about the
  inert flour beside it, and `bio-069`/`bio-070` (yoghurt) run a real lactic
  fermentation and stop at a pH no shipped database can compute. The corpus
  README records all eight row by row rather than counting eight closures.
- **Scope:** curate roughly 100 reactions/networks around starch/sugar
  digestion, catalase, lactase/protease exemplars, yeast fermentation, bread
  rising, respiration, photosynthesis as a bounded net model, acidification,
  food browning only where a defensible simplified family exists, and enzyme
  inhibition experiments. Supply parameters from primary/open sources or leave
  the reaction qualitative/equilibrium-only.
- **Integration:** material recipes from BRD-013/014, `wait`, `kero study`,
  calorimeter, gas/headspace, pH and spectrophotometer; EXP-9/14/40/47/51 quests.
- **Acceptance:** at least 30 replayable experiments with positive and negative
  controls; conservation and temperature/pH response tests; no medical or
  nutritional advice; every rate parameter is independently oracle-checked.

### BRD-060 — Crystallography Open Database adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-003.
- **Source/licence:** COD structures/data CC0. Primary source:
  <https://www.crystallography.net/cod/new.html>.
- **Scope:** ingest a small reviewed educational subset of CIF structures for
  registered substances: salts, sugar, ice polymorphs, graphite/diamond,
  metals/alloys, minerals, hydrates and selected molecular crystals. Preserve
  COD ID, original authors/citation, cell, coordinates, occupancy, temperature
  and disorder flags. Validate CIF and identity/composition before promotion.
- **Integration:** registry `structure` facet, precipitate/crystal inspection,
  codex and viewer payload. A crystal record does not supply thermodynamic
  stability or a reaction rule.
- **Acceptance:** at least 50 representative structures, deterministic
  normalized payloads, malformed/disordered fixtures, exact formula/charge
  checks where meaningful, CC0 plus scholarly attribution in provenance.

### BRD-061 — spglib symmetry adapter

- [ ] **Status:** open. **Size:** medium. **Depends on:** BRD-060.
- **Candidate/licence:** spglib BSD-3-Clause. Primary project:
  <https://github.com/spglib/spglib>.
- **Scope:** compile the narrow C API needed to standardize cells and report
  space group, equivalent atoms and symmetry operations. First target native;
  ship to wasm/mobile only after a compile/size spike. Define stable Rust-owned
  input/output types and resource limits.
- **Acceptance:** known NaCl, diamond, graphite, ice and calcite fixtures;
  tolerance sensitivity is surfaced; native results reproduce the upstream
  oracle; licence and target gates pass.
- **Out of scope:** predicting a crystal structure from formula.

### BRD-062 — Crystal inspection and growth experience

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-060,
  BRD-061 and BRD-081's selected viewer.
- **Scope:** add `inspect crystal`/GUI affordance showing unit cell, repeated
  lattice, coordination and symmetry at register-appropriate detail. Connect
  precipitation/recrystallisation events to the correct known structure where
  the solved phase has an exact mapping; otherwise show particles or an honest
  “structure not installed.” Crystal growth animation follows solved deposited
  amount, not a nucleation claim.
- **Acceptance:** native/web visual contract snapshots, exact phase-to-structure
  mapping, accessibility descriptions, and no structure shown for ambiguous
  polymorphs without the ambiguity being stated.

## Stage B5 — tactile physics and scientific views

### BRD-070 — Scene/chemistry authority contract

- [x] **Status:** complete (2026-08-30). **Size:** medium. **Depends on:** GUI scene graph and
  current operator/event contract.
- **Outcome:** physics can make the bench tactile without becoming a second,
  divergent chemistry simulation.
- **Scope:** document and type the one-way authority boundary. Chemistry owns
  amounts, phases, temperature, pressure and events. Scene physics proposes
  gestures/collisions/transfers; an accepted operator returns the authoritative
  state and visual target. Define spill destinations, broken-container events,
  transfer reconciliation, replay seeds, reduced-motion endpoints and
  background throttling.
- **Acceptance:** replay and host parity; visual frame rate cannot change
  transferred moles; interrupted pours reconcile exactly; reduced-motion and
  headless execution reach the same state; contract referenced from
  `ROADMAP-GUI.md` and `APPARATUS.md`.
- **Evidence:** `kerotakis_core::authority` provides serde-stable typed
  proposals, replay seeds, explicit vessel/bench/tray/floor destinations,
  chemistry-owned break/spill event shapes, presentation-only motion policies,
  and receipt-driven cumulative transfer reconciliation. The executable
  `scene_authority` tests pin host serialization, different frame cadences,
  reduced-motion/headless/background endpoints, exact interruption, and
  refusal/malformed-proposal non-advancement. BRD-073 still owns emitting the
  reserved break/spill events and creating material-holding spill state.

### BRD-071 — Rapier rigid-body integration

- [x] **Status:** complete. **Size:** medium-large. **Depends on:** BRD-070.
- **Candidate/licence:** Rapier Apache-2.0 with deterministic wasm builds.
  Primary project: <https://github.com/dimforge/rapier>.
- **Scope:** prototype glassware/apparatus collision, stacking, tipping and
  dropping against current 2-D bench needs before choosing 2-D or 3-D. Use
  catalog footprints/ports as collider sources; chemistry-breaking thresholds
  remain explicit apparatus data and engine events.
- **Acceptance:** deterministic replay on supported hosts, keyboard/touch
  equivalents, no tunnelling in the drop corpus, measured bundle/performance
  budget, and a go/no-go versus simpler local collision handling.
- **Delivered tasklist:**
  - [x] isolate Rapier 2-D from chemistry authority behind versioned,
    quantized replay inputs and bounded untrusted-input limits;
  - [x] validate a six-item prototype collider/port catalog and exercise
    stacking, tipping, dropping, collision proposals and an 18-case CCD corpus;
  - [x] compile mouse, pen, touch and keyboard endpoints to identical canonical
    intents while reduced-motion/headless/background modes remain visual only;
  - [x] pin a serialized replay SHA-256 golden for Linux/macOS CI host parity;
  - [x] measure release-native timing and a retained wasm payload, with stable
    determinism/payload gates and advisory shared-runner timing thresholds.
- **Decision/evidence:** **go with optional Rapier 2-D** for tactile bench
  collision, stacking, tipping and drop proposals; retain the simpler local
  path for deployments that omit the feature. The current bench has no depth
  interaction or rendering contract, so 3-D adds cost without an accepted user
  endpoint and is a no-go for this milestone. The 20-object/360-tick probe was
  byte-identical across three runs (`efb244de…ce0ea`), measured 0.100 ms p95
  and 0.113 ms maximum per step on the reference x86_64 Linux host, and the
  conservative standalone wasm upper bound was 392,897 gzip bytes (below the
  768,000-byte gate). `tools/brd071_evaluate.py` makes the reproducible gates
  executable; CI runs the golden on both supported desktop hosts.

### BRD-072 — Salva fluid-visual integration

- [x] **Status:** complete/no-go. **Size:** medium-large. **Depends on:**
  BRD-070; may run in parallel with BRD-071.
- **Candidate/licence:** Salva Apache-2.0, Rust SPH with viscosity, surface
  tension, multiphase fluids, wasm and optional Rapier coupling. Primary
  project: <https://github.com/dimforge/salva>.
- **Scope:** prototype three visuals: water pour, oil/water layers and viscous
  syrup. Feed density/viscosity/surface tension only from solved/provenanced
  state; map SPH particles to an already accepted transfer fraction. Compare
  with the existing lightweight `fluidScene` path on size, stability, mobile
  performance and reduced motion.
- **Acceptance:** no particle loss affects chemistry; visual phase ordering
  matches authoritative layers; 60/30 fps budgets are explicit; go/no-go
  report. If no-go, retain Salva as a reference and improve `fluidScene`.
- **Delivered tasklist:**
  - [x] build bounded Salva 2-D prototypes for water pouring, authoritative
    oil/water layers and a high-viscosity syrup;
  - [x] accept only provenanced density, viscosity and surface-tension values,
    while keeping all coefficients and particles presentation-only;
  - [x] map the accepted BRD-070 transfer fraction independently per phase,
    and prove deliberate render-particle loss cannot alter that endpoint;
  - [x] improve `fluidScene` with accepted-fraction scaling, viscosity damping,
    surface-tension droplet sizing, deterministic event seeding and a strict
    reduced-motion/no-animation path;
  - [x] measure deterministic replay, phase order, particle-loss isolation,
    standalone wasm payload and explicit 60/30 fps thresholds.
- **Decision/evidence:** **no-go for shipping Salva in the interactive path;
  retain Salva as a reference and ship the improved lightweight `fluidScene`
  path.** Three reference runs reproduced the exact
  visual trace (`a472b73…0aeaf8`), retained authoritative chemistry through a
  forced 50% particle decimation and preserved bottom-to-top phase order. The
  standalone Salva wasm upper bound was modest at 48,990 gzip bytes, but the
  96-particle, 120-step stress frame measured 35.99 ms p95 and missed the
  explicit 33.33 ms/30 fps reference budget (and therefore 16.67 ms/60 fps).
  More decisively, its dependency closure includes archived
  `generational-arena` (`RUSTSEC-2024-0014`) and MPL-2.0 code rejected by the
  shipping licence policy; the measured prototype was therefore removed from
  the product build graph instead of weakening either gate. The existing path
  already has a 9 ms governor, economy grid, static/reduced-motion endpoint and
  no extra runtime boundary. `tools/brd072_evaluate.py` keeps the stable gates
  and named-reference timing decision executable.

### BRD-073 — Spills, tipping, drops and breakage

- [x] **Status:** complete (2026-08-30). **Size:** large. **Depends on:**
  completed BRD-071 and the closed-no-go BRD-072 outcome.
- **Scope:** add operator/event semantics for controlled partial pours, bench
  spills, vessel tipping, collision damage and recovery/cleanup. A broken
  vessel creates recoverable consequences and transfers its contents to a
  typed spill compartment; safety reruns against exposed/combined material.
- **Integration:** undo/replay, story inventory, disposal quests, cabinet
  replacement, Burst, accessibility and notebook evidence.
- **Acceptance:** mass/element/energy ledgers close across every failure path;
  identical chemistry with and without animations; hazardous spills emit
  precise safety events; save/load migration and undo cannot duplicate stock.
- **Completed tasklist:**
  - [x] authoritative typed bench/tray/floor spill compartments and cumulative
    partial-pour reconciliation;
  - [x] deterministic collision thresholds, vessel breakage, full-content
    transfer, cleanup/recovery and stable replacement-vessel identities;
  - [x] combined exposed-material safety reruns with sorted contributor species
    plus cross-location safety findings;
  - [x] mass, element and energy conservation, zero-fraction/no-break no-ops,
    animation/reduced-motion/headless endpoint parity;
  - [x] legacy-save defaults, serialized spill state, exact replay and
    snapshot-undo recovery without stock duplication;
  - [x] Burst-style incident presentation, static reduced-motion equivalent,
    accessible live status, hazard feed cards and durable notebook evidence.

### BRD-074 — Gas-to-foam observable and elephant-toothpaste slice

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-014 and
  BRD-070; reuse the existing peroxide kinetics until BRD-050/052 supplies the
  richer enzyme model.
- **Outcome:** gas-forming reactions can drive bubbles or persistent foam when
  a recipe contains a declared surfactant, without treating foam as new matter.
- **Scope:** add typed gas-production-rate, trapped-gas, foam-volume/height,
  overflow, lifetime and warmth observables. Chemistry owns oxygen yield, rate
  and heat; a bounded drainage/coalescence model maps those values plus
  surfactant concentration, viscosity, vessel geometry and temperature onto a
  visual target. Reduced motion shows the same peak and final state. Food color
  is an optical passenger and may form user-chosen stripes without changing the
  rate. Ship an elephant-toothpaste experiment comparing no catalyst, hydrated
  yeast/catalase surrogate, manganese dioxide and potassium iodide where its
  distinct reaction path is installed.
- **Safety contract:** the child-facing real-world activity is 3% peroxide,
  adult supervision, fitted goggles and gloves. Concentrations above 3% remain
  explorable in simulation but are labelled restricted; 10% and above are
  never described as safe home practice. Closed-vessel and combustible-contact
  variants are vetoed or presented only as safety boundaries.
- **Acceptance:** conservation from 2 H2O2 to 2 H2O + O2; catalyst survives;
  oxygen/foam monotonicity across controlled concentration and catalyst tests;
  exothermic temperature response with an explicit enthalpy source; native/web
  event parity; visual snapshots for foam rise, overflow, color stripes and
  reduced motion; a no-soap control bubbles but does not build persistent foam.
- **Out of scope:** CFD-derived bubble-size distributions, ingestible advice,
  or claiming one yeast brand has a universal enzyme activity.
- **First implementation slice (2026-08-27):** dish soap and dry yeast are
  versioned material recipes with explicit unresolved blends/biomass. The
  existing peroxide rate law owns O2 yield and catalyst choice; its interval
  now emits gas rate and an explicit 98.2 kJ-per-stoichiometric-extent heat
  source. A recipe-declared, bounded drainage/coalescence model maps O2 to
  trapped gas, foam volume/height, half-life and overflow using deterministic
  vessel geometry. With no declared surfactant, the same chemistry bubbles but
  produces no persistent foam. Remaining work: hydration/activity dependence,
  KI's distinct path, color stripes and renderer snapshots. The first guided
  `elephant-toothpaste.lab` lesson compares equal 3% peroxide/yeast charges with
  and without dish soap on one shared clock, then waits one foam half-life.
- **Colored-foam checkpoint 1 implemented:** persistent foam now carries the
  liquid mixture's engine-computed spectral sRGB and plain-language colour into
  the additive scene contract. The web vessel lightens that physical tint into
  bubble-film and overflow visuals, while older hosts still fall back to white;
  arbitrary food-colour mixtures therefore change the foam without changing
  oxygen yield or rate. Spatially preserved, user-placed stripes still require
  a typed placement operator and are not inferred from a well-mixed vessel.
- **Quantitative-catalysis checkpoint implemented (2026-08-27):** catalyst
  selection is no longer a boolean presence test. Effective dissolved KI
  concentration enters with the measured first-order iodide dependence;
  catalase scales with effective enzyme loading and a Michaelis–Menten
  substrate-saturation correction; MnO₂ consumes material-lot mass, density,
  particle diameter and suspended fraction to obtain nominal exposed area.
  Magnetic-stirrer tip speed supplies a bounded external mass-transfer gain.
  Thus dose, grinding and stirring now change oxygen production, foam growth,
  overflow and reaction heat through one kinetics path. Regression tests cover
  twofold initial KI/enzyme dose response, fourfold household-dose ordering in
  integrated oxygen and foam, twofold MnO₂ loading, tenfold area gain from
  grinding, bounded mixing acceleration, catalyst retention, and the complete
  household peroxide + soap + yeast/KI visual outcome. The guided
  `elephant-toothpaste-catalyst-dose.lab` comparison gives two equal 3%
  peroxide/soap vessels 0.25 g and 1 g KI on one shared ten-second clock so the
  resulting foam/overflow difference is directly visible. The mixing pass now
  transfers declared solution catalysts such as KI from solid inventory into
  the aqueous phase, emits `Dissolved`, preserves their moles, and prevents a
  dissolved catalyst from being rendered or gravity-settled as sediment.
  The real-browser CI self-test now runs that two-dose KI experiment through
  the shipped worker and UI, then asserts two rendered foam columns, visible
  out-of-glass overflow, dose ordering, and the absence of the former
  unsupported-contact warning. This closes the core-to-Wasm-to-DOM path rather
  than accepting a scene value that the child cannot see.
  Shelf clicks, periodic-table additions and reagent drops now model the
  physical dispense as explicit, replayable one-second contact ticks. A
  nonreactive addition stops after its first tick; computed bubbling or growing
  foam keeps the gesture advancing for at most ten seconds, with short visual
  pacing between scene updates. Kinetic reactions therefore blubber, rise and
  overflow after the gesture instead of requiring a child to discover the
  textual `wait` command; authored lessons and command-line scripts retain
  complete control of time.
  Dry yeast recipe components now retain material-lot provenance and their
  first liquid-contact time. Only that reviewed surrogate receives a bounded
  hydration/activity ramp (warmer water shortens its teaching time constant);
  purified catalase remains immediately available, catalyst moles remain
  conserved, and old saves without lot provenance retain their prior result.
  The correlation is explicitly editorial and does not claim universal
  activity for a yeast brand, age or batch.
  Catalase now also has a smooth high-temperature activity envelope: moderate
  warmth still accelerates the curated pathway, while very hot water suppresses
  it instead of allowing Arrhenius extrapolation to grow without limit. This is
  an instantaneous teaching envelope, not yet irreversible denaturation memory;
  the latter requires explicit exposure history and remains a boundary.
  Remaining
  boundaries are yeast-brand/age calibration, irreversible denaturation
  history and inhibition, catalyst pore/BET area, adsorption and pore-scale
  diffusion.

### BRD-075 — Transparent dye and opaque-pigment mixing

- [ ] **Status:** open. **Size:** medium-large. **Depends on:** BRD-014 and the
  existing spectral/Beer–Lambert path; BRD-070 for the renderer contract.
- **Candidate/licence:** `palette` (MIT OR Apache-2.0) for audited color-space
  conversion/interpolation. It does not supply chemistry or pigment constants;
  keep the dependency only if it reduces tested conversion code and passes the
  wasm/mobile size gate.
- **Scope:** distinguish transparent food color/watercolor absorption from
  opaque acrylic pigment scattering. Dyes mix by concentration, path length
  and spectra through Beer–Lambert. Paint uses a bounded Kubelka–Munk K/S model
  with curated pigment coefficients, binder/white-substrate assumptions and an
  explicit “pigment data missing” result. Never average display RGB as though it
  were a physical mixture. Track dilution, opacity, staining and unresolved
  proprietary pigment/binder fractions through `MaterialRecipe` versions.
- **Interaction:** offer side-by-side swatches, droppers/brush amounts, undo,
  arbitrary user ratios and “what should I add to move toward this color?” only
  as bounded interpolation among installed materials—not general inverse
  formulation.
- **Acceptance:** primary/secondary transparent-dye fixtures, subtractive paint
  fixtures including white/black, concentration/intensity monotonicity, order
  independence, spectral-to-sRGB oracle tests, color-vision-safe descriptions,
  and identical numeric outcomes headless/native/web.
- **Out of scope:** branded paint matching, fluorescence, drying/polymerization
  in v1, or learned image-based color prediction.
- **Transparent-dye checkpoint 1 implemented:** explicit 0.1% w/w dropper
  surrogates use the already curated 16-band spectra for betanin red, curcumin
  yellow and indigo-carmine blue. Their absorbances add in the existing
  Beer–Lambert/CIE pipeline, so arbitrary ratios, dilution, vessel path length,
  intensity and order-independent subtractive mixing are computed rather than
  RGB-averaged. Generic “Lebensmittelfarbe/food colouring” stays ambiguous;
  watercolor and acrylic remain blocked on the distinct scattering/pigment
  model.
- **Opaque-pigment checkpoint 1 implemented:** the shared native/wasm core now
  has a deterministic, order-independent Kubelka–Munk `K/S` mixer for an
  optically thick, diffusely lit acrylic-paint surrogate. Curated absorption
  and scattering spectra mix by amount before conversion through the same CIE
  observer as solutions; white/black bounds, subtractive blue+yellow mixing,
  order independence and explicit missing-pigment-data refusal are tested.
  Installed pigment coefficient records, thin watercolor washes, UI
  droppers/brushes and substrate/coverage controls remain separate checkpoints.
- **Transparent-watercolor checkpoint 1 implemented:** red/betanin,
  yellow/curcumin and blue/indigo-carmine watercolor washes are versioned
  school-material surrogates at 0.02% w/w. They expand to water plus the same
  reviewed chromophores as the food-color droppers, so concentration, dilution,
  path length and arbitrary-ratio mixing remain Beer–Lambert/CIE calculations.
  Generic “Wasserfarbe/watercolor” stays unclaimed, and these transparent
  washes are not presented as opaque commercial pigment pans.
- **Acrylic-material checkpoint 1 implemented:** named red, yellow, blue,
  white and black waterborne acrylic teaching surrogates carry effective
  16-band absorption/scattering roles. Shelf swatches and arbitrary-ratio
  vessel mixtures run through the shared Kubelka–Munk/CIE core; white lightens,
  blue+yellow mixes subtractively, order does not matter and acrylic is visibly
  opaque. Water, pigment and binder fractions remain explicitly surrogate or
  unresolved, generic “Acrylfarbe/acrylic paint” stays ambiguous, and no result
  claims a brand, artist pigment, wet-film gloss or dried-film match.

### BRD-076 — Movable Bunsen burner and guided heat interactions

- [ ] **Status:** open. **Size:** large. **Depends on:** BRD-070, BRD-071 and
  BRD-041's combustion/oxidation mechanisms.
- **Outcome:** learners can place a burner, adjust gas and air, light/extinguish
  it, and heat or ignite nearby matter through the same authoritative engine
  used by scripts.
- **Scope:** typed place/move/valve/air-collar/ignite/extinguish operations;
  flame geometry and heat-flux field derived from fuel flow and entrained air;
  vessel/material exposure integrates energy over time. Installed combustion
  models decide ignition, sustained burning, oxygen-starved yellow flames,
  soot/CO boundaries and fuel depletion. Distance, shielding, vessel material,
  heat capacity and breakage thresholds matter. Scene motion proposes poses;
  chemistry accepts exposure and owns temperature/reaction events.
- **Guidance:** sandbox permits free placement with persistent hazard cues;
  lessons may highlight safe zones and controls but do not teleport tools or
  fake outcomes. Keyboard/touch controls and reduced-motion equivalents expose
  every operation.
- **Acceptance:** deterministic replay of pose and valve history; validated
  heat-flux tests within the declared near-field model; water heats without
  burning, ethanol/candle/paper ignite only after their installed gates,
  nonflammable controls refuse, fuel/oxygen/energy ledgers close, and moving the
  flame away stops heat transfer. Native/web parity and safety veto tests.
- **Out of scope:** using renderer pixels as collision/temperature truth, full
  turbulent flame CFD, or implying that unmodelled materials are nonflammable.
- **Guided-control checkpoint 1 implemented:** a Bunsen burner can be deployed
  at the selected vessel/work zone, moved by selecting another zone, and given
  a 5–100% flame setting plus bounded exposure time. The first reviewed
  near-field bridge delivers at most 500 W to the vessel and compiles the
  exposure to the authoritative replayable `heat` operator; a separate “touch
  flame to contents” control compiles to `ignite`, so the engine's installed
  combustion gates—not the animation—decide whether anything burns. Continuous
  free-space pose, fuel/air collar chemistry and distance-dependent heat flux
  remain for the typed apparatus-state tranche.
- **Air-collar checkpoint implemented:** the deployed burner now exposes a
  0–100% air collar beside its 0–100% gas/flame control. Zero flame is visibly
  extinguished and cannot heat or invoke ignition. Opening the collar moves a
  declared near-field teaching efficiency from 55% to 100%, changes the
  rendered low-air yellow flame to the open-air blue flame, and compiles the
  resulting energy to the same replayable `heat` operator. This does not claim
  fuel depletion, soot or CO; those remain dependent on typed burner fuel/air
  chemistry rather than renderer colour.
- **Liquid-fuel checkpoint 1 implemented:** touching the guided flame to an
  open vessel of ethanol now reaches CEA's separately parsed, feed-only liquid
  record, admits only the matching ethanol vapour plus named stable flame
  gases, and computes fuel depletion, CO2/water-vapour products and reaction
  energy from the bundled Apache-licensed NASA-9 data. HP remains preferred;
  when its liquid-feed bracket fails, a declared TP fallback uses the explicit
  ignition-zone temperature and says so in provenance. This does not yet claim
  sustained pool-fire geometry, sealed/oxygen-starved combustion, soot/CO, or
  isopropanol identity and volatility data.
- **Isopropanol checkpoint 1 implemented:** pure isopropanol is now a searchable
  registry identity with reviewed room-liquid properties, and household 70%
  rubbing alcohol is a localized, fixed 70/30 v/v recipe rather than a falsely
  relabelled mass mixture. The shared safety screen classifies the alcohol as a
  flammable liquid, and a NIST Antoine fit computes its volatility across the
  stated range around the normal boiling point. A flame held to the 70% aqueous
  mixture remains an explicit combustion-model boundary: the bundled CEA subset
  has no isopropanol feed thermochemistry, so this checkpoint does not fake fuel
  depletion, products, heat release or sustained burning.

### BRD-077 — Element coverage score and progressive periodic table

- [x] **Status:** complete (2026-08-30). **Size:** medium. **Depends on:** BRD-000, BRD-012 and
  BRD-023; reaction links deepen progressively as later family packs land.
- **Outcome:** selecting an element answers “what can I actually try with this?”
  while the default table stays inviting rather than presenting 118 equally
  actionable boxes.
- **Default view:** a curated **lab/curiosity table**, not the exact reduced
  main-group diagram used as inspiration. Keep familiar main-group elements and
  high-value transition metals such as Mn, Fe, Cu and Zn; omit obscure,
  synthetic and highly hazardous elements from the default even when they fit
  a neat block pattern. Preserve real group/period positions and visible gaps.
  Offer an explicit “full table” toggle with all 118 structural identities,
  remembered per user and keyboard/screen-reader accessible.
- **Coverage criterion:** compute an element-to-content index from parsed
  formulas across pure species and expanded material recipes. For each default
  element, aim for two meaningfully different familiar examples and at least
  one runnable educational interaction where chemistry supports it. One example
  may be elemental/simple and one a common compound/material; repeated salts or
  ubiquitous water do not satisfy diversity by themselves. Counts must expose
  capability level: identity-only, add/observe, property-backed, reacting, and
  lesson-backed.
- **Prioritization:** score gaps by child/teen curiosity demand, familiarity,
  solver readiness, visual/educational payoff, and safety burden. A high score
  advances a substance/reaction tranche; no quota may force an obscure,
  unsupported or dangerous bottle onto the shelf. Full-table-only elements may
  honestly say why no runnable example is installed and link to safe structural
  or nuclear context instead.
- **Interaction:** selecting a cell lists installed substances/materials that
  contain that element, then separately lists runnable reactions/lessons and
  their required co-materials. Search accepts symbol, localized element name,
  formula and common material name. Coverage badges and empty states come from
  generated registry/route data, never a parallel hand-maintained UI list.
- **Acceptance:** generated coverage report and regression fixture; default/full
  toggle snapshots at desktop/mobile/reduced motion; Fe/Cu/Zn remain reachable
  in the default view; Po/At/Fr/Ra and synthetic elements do not appear there;
  every displayed count round-trips to a real shelf key and every “runnable”
  link replays successfully through the engine.
- **Out of scope:** inventing a compound per element, implying every element is
  safe to handle, or using visual completeness as a substitute for model
  coverage.
- **First UI slice (2026-08-27):** the existing 118-element structural table is
  retained behind a remembered full-table toggle. The default curated lab table
  keeps high-value Mn/Fe/Cu/Zn and excludes Po/At/Fr/Ra/synthetic elements;
  cells show a count generated from the live shelf's parsed formulas and empty
  cells remain honest. The completed slice now includes expanded recipes,
  replay-proved lesson/codex routes, capability levels, a reviewed default-view
  artifact, localized search and browser-level desktop/mobile/reduced-motion
  accessibility coverage.
- **Completion tasklist and DoDs (2026-08-30):**
  - [x] Generate a deterministic, versioned 118-entry coverage report from
    pure species and expanded material recipes. **Done when:** its reviewed
    regression fixture is stable, every example resolves to a live shelf key,
    identity-only cells remain present, and native plus wasm boundaries agree.
  - [x] Derive runnable content links from shipped lesson/codex scripts.
    **Done when:** required co-materials all resolve to the shelf, lesson kits
    are generated from source, and every advertised source passes the existing
    real-engine replay/lint gates.
  - [x] Finish the progressive table interaction. **Done when:** coverage
    levels, substance/material search, honest empty states, direct lesson and
    experiment actions, remembered lab/full modes, keyboard names, mobile
    layout and reduced-motion behavior pass focused web tests and production
    build.
  - [x] Integrate and audit both host transports. **Done when:** native/wasm
    schemas match, Fe/Cu/Zn and Po/At/Fr/Ra/synthetic inclusion rules regress,
    formatting/clippy/focused suites/full preflight pass, the PR merges without
    unrelated work, and the resulting GitHub `main` workflow is green.

### BRD-080 — Molecular viewer selection spike

- [ ] **Status:** in progress; 3Dmol.js 2.5.5 is the provisional smaller-
  adequate selection, pending disposable Svelte and physical constrained-
  mobile acceptance. Do not ship both candidates. **Size:** small-medium. **Depends on:**
  BRD-012.
- **Candidates/licences:** 3Dmol.js (BSD) and Mol* (MIT). Primary projects:
  <https://github.com/3dmol/3Dmol.js> and
  <https://github.com/molstar/molstar>.
- **Scope:** render the same molecule, crystal/CIF, protein exemplar, cube
  orbital and short trajectory; test selection, labels, accessibility hooks,
  offline bundling, Svelte integration, mobile memory and bundle size.
- **Decision rule:** choose the smaller adequate viewer; Mol* wins only if
  macromolecular/volume capability justifies its complexity. Do not ship both.
- **Acceptance:** report and prototype behind a disposable route; exact licence
  inventory; no production dependency in the decision PR.
- **Claimed progression (minimum 2 h; merge each checkpoint independently):**
  1. [x] **BRD-080a — Reproducible candidate evidence (45–75 min).** Pin the
     exact 3Dmol.js and Mol* releases, licences and dependency closures; build
     the same molecule, CIF, protein, cube and trajectory fixture matrix with
     cold bundle-size and offline checks. **DoD:** a committed, deterministic
     evidence command refuses missing fixtures/unknown licences, records raw +
     gzip bytes and dependency counts, and tests malformed/incomplete reports;
     no runtime dependency enters the production app.
     *Done 2026-08-31:* the isolated exact lock pins 3Dmol 2.5.5 and Mol*
     5.11.0 without touching the app dependency graph. Five project-authored,
     hash-pinned format probes and a fail-closed Node test validate fixture
     integrity, exact top-level pins, every installed production licence and
     lock integrity. Real Vite entries measure 3Dmol at 586,724 raw / 168,749
     gzip bytes and Mol* at 5,345,087 raw / 1,968,375 gzip bytes; the evidence
     also exposes Mol*'s Node >=22 requirement and its 222-instance combined
     spike closure rather than silently accepting the host's Node 20 warning.
  2. [ ] **BRD-080b — Disposable comparison route (45–75 min).** Exercise both
     candidates behind one spike-only adapter contract with selection, labels,
     resize/dispose and accessible tabular fallback. **DoD:** identical fixture
     inputs and scripted interactions run for both viewers; teardown and
     unsupported-volume/crystal cases are explicit; keyboard, reduced-motion,
     offline and bounded-mobile-memory evidence is captured; full frontend
     tests/build/licence gates pass.
     *Code/browser checkpoint 2026-08-31:* the isolated route gives both candidates the same five
     local fixtures behind one bounded adapter contract and an always-present
     semantic atom table. SSR guards, exact kind/format gates, source/atom/bond/
     frame/grid/coordinate/unit-cell/canvas limits, labels, selection, reduced
     motion, stale-mount cleanup and idempotent disposal are tested. A dedicated
     Node 22 Chrome lane exercised all ten candidate/fixture paths, required one
     live canvas plus semantic rows after each replacement, and observed no
     request outside the locally served origin. The production app remains
     unchanged. *Svelte/deployment checkpoint:* the disposable route is now a
     strict Svelte 5 component, passes `svelte-check`, and its ten paths pass a
     keyboard-driven Playwright Pixel 7 profile against the isolated HTTPS
     Vercel deployment with zero external requests. **Pending acceptance:**
     physical constrained-mobile RAM/GPU evidence; Playwright emulation and
     SwiftShader are explicitly not substitutes for that check.
  3. [ ] **BRD-080c — Audited go/no-go record (30–60 min).** Independently
     review the measurements and choose exactly one candidate, or close no-go
     with the existing 2-D fallback named. **DoD:** primary-source citations,
     exact transitive licence inventory, target/bundle/memory table, decision
     rule application and reproducible artifact hashes are committed; BRD-080
     closes and BRD-081 is either unblocked with a bounded first slice or
     marked not-applicable without overstating capability.
     *Evidence checkpoint 2026-08-31:* two independent primary-source/package audits and the
     executable comparison show both candidates reaching ready state with one
     canvas and semantic rows for the bounded five-fixture matrix. The
     committed decision provisionally selects 3Dmol 2.5.5 under the
     smaller-adequate rule: 168,749 gzip bytes and six closure instances versus
     Mol*'s 1,968,375 bytes and 216, with no accepted capability requiring the
     larger viewer. Accessibility, UMD/eval, teardown, DPR and physical-mobile
     limitations remain explicit acceptance gates. The go/no-go remains
     provisional until disposable Svelte integration and physical constrained-
     mobile RAM/GPU checks pass.
  4. [ ] **BRD-081a — Renderer-neutral accessible core (60–90 min,
     conditional on go).** Land `ScientificView` data/view-state contracts and
     the plain-language/table alternative before the selected renderer.
     **DoD:** deterministic serialization, hostile/invalid input bounds,
     molecule + crystal + orbital fixture coverage, SSR/offline behavior and
     full workspace/preflight gates; no conformer, unit cell or surface is
     inferred.

### BRD-081 — Molecular/crystal viewer integration

- [ ] **Status:** blocked on the remaining BRD-080 acceptance checks; BRD-060
  also blocks the crystal slice. **Size:** medium-large. **Depends on:**
  BRD-080 and BRD-060 for the crystal slice.
- **Scope:** create a renderer-neutral `ScientificView` contract for atoms,
  bonds, unit cells, surfaces/volumes, annotations and provenance, then adapt
  the selected viewer. It consumes registry structures and future QM assets;
  it never invents a conformer or crystal. Add plain-language and tabular
  alternatives.
- **Acceptance:** offline PWA/native packaging, molecule/crystal/orbital
  fixtures, accessible fallback, theme independence from computed chemical
  appearance, and deterministic view-state serialization.

### BRD-082 — Ketcher structure/reaction authoring surface

- [ ] **Status:** optional after the organic executor. **Size:** medium-large.
  **Depends on:** BRD-022 and BRD-081.
- **Candidate/licence:** Ketcher Apache-2.0; its standalone mode uses Indigo
  wasm. Audit all npm packages/assets/notices and avoid bundling a second
  chemistry engine if BRD-022 selected RDKit without a justified boundary.
  Primary project: <https://github.com/epam/ketcher>.
- **Scope:** Sandbox/codex-author mode for drawing or importing a molecule or
  mapped reaction. Submission goes through Kerotakis identity, safety and
  reaction-family routing. The editor may validate/draw; it cannot authorize a
  transformation or populate missing thermodynamic properties.
- **Acceptance:** offline, keyboard/touch accessible, Svelte integration,
  round-trip MOL/SDF/SMILES/RXN fixtures, identity conflict handling, and an
  unmistakable response when a valid drawing has no installed behavior model.
- **Out of scope:** exposing unrestricted reaction enumeration to learners or
  treating a drawn arrow as evidence that chemistry occurs.

## Stage B6 — build-time validation and release gate

### BRD-090 — pycalphad solid/alloy oracle

- [ ] **Status:** optional build-time oracle. **Size:** medium. **Depends on:**
  a concrete EXP/materials task and BRD-032 if coupled fluid/solid behavior is
  being checked.
- **Candidate/licence:** pycalphad code MIT. Thermodynamic database (`.tdb`)
  licences are independent and often restrictive; no calculation begins until
  a specific cleared database is recorded. Primary project:
  <https://github.com/pycalphad/pycalphad>.
- **Scope:** validate a bounded school-relevant system such as a cleared binary
  alloy phase diagram or heat-treatment transition. Persist only reviewed
  aggregate/golden values with provenance; Python remains build-time.
- **Acceptance:** independent hand/literature check, pinned environment and TDB
  checksum/licence, deterministic fixture generator, written validity range.
- **Out of scope:** scraping proprietary CALPHAD databases, a general metallurgy
  engine, or shipping Python.

### BRD-091 — OpenMM decision record (parked)

- [ ] **Status:** parked; do not implement without a new concrete requirement.
  **Size:** small decision record. **Depends on:** BRD-052 or a future molecular
  motion experiment that cannot use the current particle/kinetics models.
- **Reason:** OpenMM simulates molecular dynamics, not bond-making/breaking in
  ordinary chemistry. Its Reference/CPU pieces are MIT, while GPU platform
  pieces have different licensing; it is heavy and lacks a natural offline
  browser/mobile fit for this product. Primary licence record:
  <https://github.com/openmm/openmm/blob/master/docs-source/licenses/Licenses.txt>.
- **Gate:** an agent may reopen this only with a named experiment, force field
  and data licence, target matrix, educational observable, and proof that a
  trajectory materially teaches something the lighter particle view cannot.
- **Acceptance:** either a justified scoped successor task or a dated no-go.
- **Out of scope:** adopting molecular dynamics as a proxy for chemical
  reactivity.

### BRD-092 — CoolProp decision record (oracle/extra only)

- [ ] **Status:** parked. **Size:** small decision record. **Depends on:** a
  BRD-030 discrepancy or fluid absent from a cleared feos parameter pack.
- **Candidate/licence:** CoolProp MIT, but wrapper/platform support and fluid
  data provenance must be checked separately. Primary project:
  <https://github.com/CoolProp/CoolProp>.
- **Reason:** CoolProp offers a broad reference-quality fluid-property surface,
  but duplicates much of the intended feos role and currently has a less clean
  cross-target integration path. It is valuable as a second opinion before it
  is valuable as shipped runtime weight.
- **Gate:** prefer it as a desktop/build-time differential oracle. Shipping is
  reconsidered only if it closes a named high-demand fluid gap on every target
  at acceptable size and no feos route exists.
- **Note from BRD-030 (2026-08-30):** do **not** reach CoolProp data by way of
  feos. `feos:parameters/multiparameter/coolprop.json` carries CoolProp's
  reference-EOS coefficients with the MIT notice and the per-fluid citations
  removed; if CoolProp data is ever wanted it must come from CoolProp itself,
  with its notice, or from the primary publications. See
  `provenance/brd-030-feos-spike.md` § 4.2.
- **Acceptance:** dated comparison and explicit oracle/runtime decision.

### BRD-093 — Permissive thermochemical-engine target gate

- [x] **Status:** closed no-go for universal runtime (2026-08-30); optional
  native/build-time oracle only. **Size:** small decision record. **Depends
  on:** a named high-temperature condensed-phase experiment that the existing
  CEA path cannot represent.
- **Candidate/licence:** Thermochimica code is BSD-3-Clause. Its thermodynamic
  databases and individual CALPHAD assessments are separate works and require
  record-by-record redistribution review. EQ3/6 and OpenGeoSys are also
  BSD-3-Clause; AqEquil wraps EQ3/6; ChemEQL is MIT. Code licences alone do not
  make their databases, packaged binaries, or dependency closures shippable.
- **Target verdict:** none is a new universal Kerotakis engine. Thermochimica
  requires a Fortran toolchain plus BLAS/LAPACK and has no demonstrated
  maintained Rust-to-wasm/iOS/Android distribution path. OpenGeoSys is a large
  native THMC application rather than a bench library. EQ3/6/AqEquil and
  ChemEQL duplicate the shipped IPhreeqc aqueous domain. PhreeqcRM is useful
  only after choosing multidimensional porous-media transport, which remains
  outside the product mission.
- **Portable rule:** a core runtime model must build behind the same Rust API
  for browser wasm, Android, iOS, macOS and Windows. A native-only backend may
  be an optional acceleration or oracle, but it may not own a learner-visible
  capability or produce a result unavailable in the PWA. Tauri does not make
  arbitrary native libraries web- or mobile-portable; installed shells use a
  native Rust core while the browser uses the wasm core. Native-only workspace
  adapters declare `[package.metadata.kerotakis] runtime = "native-only"`;
  `tools/portable-dependency-lint.py` rejects any such package in the
  `kerotakis-wasm` dependency closure and runs in preflight.
- **Reopen gate:** name the experiment and educational observable; identify a
  cleared database; prove deterministic C-ABI builds on Windows, macOS,
  Android and iOS; measure a browser-wasm build or specify a portable
  Kerotakis fallback with answer-level conformance fixtures. Until then,
  Thermochimica may generate reviewed fixtures externally, like pycalphad,
  but does not enter any shipped dependency graph.
- **Immediate path:** finish the already-claimed BRD-030 `feos` spike for
  portable fluid thermodynamics and use the completed BRD-040 verdict for gas
  kinetics: extend the portable Cantera-YAML/diffsol slice; keep full Cantera
  FFI parked unless a concrete capability gap survives BRD-042's gate.

### BRD-094 — GPU fluid and volumetric-rendering decision record

- [x] **Status:** frontend WebGPU spike only; Taichi/NanoVDB backend adoption
  closed no-go (2026-08-30). **Size:** small decision record. **Depends on:**
  completed BRD-070 and BRD-072; reopen implementation only for a named visual
  that the shipped lightweight `fluidScene` cannot express.
- **Authority and placement:** chemistry continues to own amounts, phase,
  temperature, pressure and accepted transfers. GPU state is disposable
  presentation state. Run an optional accelerator beside the renderer in the
  webview so particles/textures do not cross Tauri JSON IPC; feed it the same
  bounded scene/event contract in PWA and installed shells. A deterministic
  Canvas/WebGL/lightweight fallback remains the release baseline for old
  Android WebViews, reduced motion, headless tests and absent WebGPU.
- **Taichi verdict:** Apache-2.0 and useful for native research prototypes, but
  its AOT/C-API backend matrix is not a demonstrated browser + Android + iOS +
  macOS + Windows distribution. The official C-API tutorial currently lists
  Vulkan, OpenGL, x86 and CUDA and explicitly says Metal is unsupported; it
  does not provide the claimed transparent Metal/DX12 universal binary. Python
  may generate artifacts at build time, but no Taichi runtime enters a shipped
  Kerotakis target without passing BRD-093's target gate.
- **NanoVDB verdict:** current OpenVDB/NanoVDB is Apache-2.0, not BSD-3.
  NanoVDB is a compact GPU/CPU sparse-grid representation, principally for
  read access, rendering and collision queries; its topology is static at
  runtime. It neither calculates combustion chemistry nor supplies a fluid or
  smoke solver. Consider its C99 `CNanoVDB`/`PNanoVDB` layouts only after a
  measured sparse-volume transport bottleneck exists; do not stream NanoVDB
  buffers through ordinary Tauri IPC.
- **WebGPU candidate:** `jeantimex/fluid` is MIT and demonstrates browser SPH
  plus 2-D/3-D PIC/FLIP, marching cubes, raymarching and screen-space fluid.
  Treat it as algorithm/reference code, not a drop-in dependency: it is an
  application, requires a WebGPU-capable browser, and credits/ports earlier
  implementations whose exact copied-code provenance must be audited before
  reuse. Prefer a small project-owned WGSL effect scoped to one accepted
  observable over importing the whole demo.
- **Reopen/acceptance gate:** first name the missing visual—volumetric flame,
  smoke plume, foam or a genuinely 3-D pour. Then measure it against
  BRD-072's existing 9 ms governor on the low-end Chromebook/Android floor;
  require no chemistry/particle coupling, no readback per frame, graceful
  device-loss fallback, reduced-motion equivalence, deterministic endpoint
  snapshots, shader/source licence records and identical scene semantics on
  all hosts. Visual fidelity alone cannot make WebGPU mandatory.
- **First named candidate (2026-08-30):** a procedural envelope for a live
  vessel `ignite` event. Existing magnitude and curated flame-colour inputs
  make it bounded without inventing chemistry, while the current fallback is
  only a two-path SVG flame. Do not render generic evolved gas as smoke (there
  is no soot/particulate authority), infer burning from temperature alone, or
  copy `jeantimex/fluid` WGSL: its MIT repository identifies two earlier MIT
  ports but supplies no per-file lineage map. Implement project-owned WGSL
  from published fire-rendering ideas and record that provenance explicitly.

### BRD-100 — Breadth release gate v1

- [ ] **Status:** final integration task. **Size:** large. **Depends on:**
  BRD-001, BRD-014, BRD-023, BRD-032, BRD-041, BRD-052, BRD-062, BRD-073 and
  BRD-081. A track whose decision gate closed `no-go` satisfies this dependency
  only when its implementation children are marked `not-applicable` and the
  documented fallback is covered by the curiosity corpus. Optional decision
  tasks need only be closed with a go/no-go record.
- **Outcome:** the curiosity corpus becomes a release-quality capability
  contract rather than a one-time audit.
- **Scope:** run all prompts and publish the disposition matrix; require zero
  silent outcomes, zero unowned gaps, zero provenance-free visible numbers,
  zero unknown safety classifications for reachable stock, and host parity for
  the supported route subset. Set per-family coverage floors from the BRD-000
  baseline only after the classifier exists; do not invent percentages now.
- **Acceptance:** full preflight, licence/SBOM/provenance/locale lints, native
  and wasm smoke corpora, accessibility checks for new views, deterministic
  reports, and a human review of every disposition that changed since baseline.
- **Out of scope:** declaring Kerotakis universal. The release report names
  boundaries and the next highest-demand missing tasks.

## Agent pickup checklist

Before starting a `BRD-*` task:

1. Verify every listed dependency is merged, not merely in progress.
2. Read the owning integration document (`CAPABILITIES.md`, `EXPERIMENTS.md`,
   `APPARATUS.md`, `ROADMAP-GUI.md`) and name any required companion task.
3. Re-verify upstream version, source licence, bundled data/parameter licence,
   target support and maintenance status; update `provenance/sources.toml`.
4. Keep quarantined inputs and generated oracle outputs out of runtime data
   paths until promotion review.
5. Add the task's acceptance tests before marking it complete, run
   `tools/preflight.sh`, and update this status plus the BRD-001 baseline.

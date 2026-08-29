# BRD-040 — Cantera mechanism and API audit

**Date:** 2026-08-29 · **Branch:** `brd040/cantera-audit` · **Reference:** Cantera
3.2.0 YAML input specification.

This is an audit, not an import. No mechanism file entered the repository, no
FFI was added, and no dependency changed. What did change is the parser's
honesty: constructs it cannot model are now refused by name instead of being
dropped, and two arithmetic errors that produced wrong answers on files it
already accepted are fixed.

## 1. Method

The portable front end is
`crates/kerotakis-core/src/kinetics/mechanism.rs` (`parse_yaml` →
`ParsedMechanism` → `compile_in` → the borrowed `ReactionNetwork` IR). It was
read against the Cantera 3.2.0 input specification
(<https://cantera.org/stable/yaml/index.html>) and then exercised against real
mechanism files inspected outside the repository:

| File inspected | Source | Size |
| --- | --- | --- |
| `h2o2.yaml` | `Cantera/cantera@main:data/` | 10 species, 29 reactions |
| `gri30.yaml` | `Cantera/cantera@main:data/` | 53 species, 325 reactions |
| `air.yaml` | `Cantera/cantera@main:data/` | 8 species, 8 reactions |
| `nDodecane_Reitz.yaml` | `Cantera/cantera@main:data/` | 100 species, 553 reactions |
| `ptcombust.yaml`, `methane_pox_on_pt.yaml` | `Cantera/cantera@main:data/` | gas + surface phases |
| `ammonia-CO-H2-Alzueta-2023.yaml` | `Cantera/cantera-example-data@main` | 42 species, 54 reactions, three named reaction sections |

None of these files is in the diff and none is retained in the tree.

## 2. Feature matrix — portable subset vs Cantera YAML

Four dispositions are used. **Supported** means modelled and executed.
**Refused** means a typed `MechanismError` naming the construct. **Was
confusing** means the refusal existed but arrived as a serde type error or as an
unrelated downstream failure; it is now typed. **Was silent** means a valid file
parsed into a network that answered a different question; these were the bugs.

### 2.1 Rate-law families

| Cantera `type` | Disposition | Note |
| --- | --- | --- |
| `elementary` (irreversible) | Supported | |
| `elementary` (reversible) | Supported | reverse rate from NASA7 detailed balance |
| `three-body` (irreversible) | Supported | `efficiencies`, `default-efficiency` honoured |
| `three-body` (reversible) | **Refused** | the dominant real form — see §4 |
| `falloff` + `Troe` (irreversible) | Supported | `T2` optional |
| `falloff`, no sub-block (Lindemann, irreversible) | Supported | |
| `falloff` (reversible) | **Refused** | see §4 |
| `falloff` + `SRI` | **Was silent** → refused | dropped, reaction degraded to Lindemann |
| `falloff` + `Tsang` | **Was silent** → refused | same |
| `pressure-dependent-Arrhenius` (PLOG) | Supported | log-interpolated, duplicate pressures summed |
| `chemically-activated` | Refused | |
| `Chebyshev` | Refused | |
| `linear-Burke` (3.1) | Refused | present in `ammonia-CO-H2-Alzueta-2023.yaml` |
| `electron-collision-plasma` (3.1), `electron-collisions` (3.2) | Refused | |
| `two-temperature-plasma` | Refused | |
| `Blowers-Masel`, `three-body-Blowers-Masel` | Refused | |
| `interface-Arrhenius`, `interface-Blowers-Masel` | Refused | surface phases refused earlier anyway |
| `sticking-Arrhenius`, `sticking-Blowers-Masel` | Refused | |
| `electrochemical` | Refused | Kerotakis has its own Butler–Volmer path (ELEC-004) |
| Undocumented aliases `Arrhenius`, `three-body-Arrhenius`, `Troe`, `Lindemann`, `SRI`, `Tsang` | Refused | **deliberate divergence**: Cantera resolves these internally; the subset reads only canonical spellings |
| `(+M)` with a plain `rate-constant` and no `type` | Refused | Cantera silently reads this as an ordinary three-body reaction; refusing beats guessing |
| `M` with no `type` (3.0 auto-detection) | Refused | Cantera auto-detects three-body; the subset requires the explicit type |

### 2.2 Rate-constant parameters

| Construct | Disposition | Note |
| --- | --- | --- |
| `A`, `b`, `Ea` numeric | Supported | |
| `Ea` as a unit-bearing string (`Ea: 10 kcal/mol`) | Supported | |
| `A` as a unit-bearing string (`A: 1e12 cm^3/mol/s`) | **Was confusing** → refused | had surfaced as serde `invalid type: string` |
| Negative `Ea` | Refused | legal and common — 4× in `h2o2.yaml`, 32× in `gri30.yaml` |
| Negative `A` / `negative-A: true` | Refused | value guard already refused `A ≤ 0`; the flag is now refused by name too |
| `orders` (explicit reaction orders) | **Was silent** → refused | changes the concentration exponents *and* the units of `A` |
| `negative-orders`, `nonreactant-orders` | **Was silent** → refused | |
| Troe `A` outside [0, 1] | Refused | legal; `n-heptane-NUIG-2016.yaml` ships A from −73.91 to 2.545 |
| `duplicate: true` | Supported (ignored) | an assertion, not a rate modifier — Cantera keeps duplicates separate and sums their rates, which compiling each entry independently already does. The subset does not *enforce* the flag, so an unmarked duplicate is accepted where Cantera would error. |
| Any other reaction key | **Was silent** → refused | catch-all; future Cantera rate modifiers cannot be dropped unnoticed |

### 2.3 Units

| Construct | Disposition | Note |
| --- | --- | --- |
| `length` m/cm/mm, `quantity` mol/kmol, `time` s/ms/min, `pressure` Pa/kPa/MPa/bar/atm | Supported | |
| `activation-energy` J/mol, J/kmol, kJ/mol, cal/mol, kcal/mol, K | Supported | |
| `energy` J/kJ/cal/kcal | **Was silent** → supported | see §3.2 |
| `activation-energy` absent, derived from `energy`/`quantity` | **Was silent** → supported | see §3.2 |
| `mass`, `current` | Supported (ignored) | cannot affect a rate constant |
| `temperature` other than K | **Was silent** → refused | Cantera itself refuses non-unity temperature scales |
| Any other units key | **Was silent** → refused | |
| `units` nested in a reaction, species, or as a list's first item | **Was silent** → refused | legal Cantera scoping; the subset reads only the document-level mapping |

### 2.4 Thermo

| Construct | Disposition | Note |
| --- | --- | --- |
| `NASA7`, 2 or 3 temperature bounds | Supported | ascending rows, per the Cantera convention (the reverse of Chemkin) |
| `reference-pressure`, numeric or unit-bearing | Supported | |
| `NASA9`, `Shomate`, `constant-cp`, `piecewise-Gibbs` | Refused | error names the model *and* the species |
| Any other thermo key (e.g. `dimensionless`) | **Was silent** → refused | |
| Species with no thermo | Supported | only irreversible reactions may use it |

### 2.5 Species and phases

| Construct | Disposition | Note |
| --- | --- | --- |
| `composition` with positive counts | Supported | |
| Fractional counts | Supported | |
| Zero or negative counts (ions: cations carry `E: -1`) | Refused | ionised chemistry is out of the subset; the refusal names species and element |
| `transport` block | Supported (ignored) | never read by the kinetics path |
| `note`, `critical-parameters` | Supported (ignored) | |
| `equation-of-state` (real-gas parameters) | Supported (ignored) | Cantera ignores these for an `ideal-gas` phase, and only `ideal-gas` phases are accepted. `h2o2.yaml` and `nDodecane_Reitz.yaml` carry Redlich–Kwong coefficients on every species. |
| Any other species key | **Was silent** → refused | |
| `thermo: ideal-gas` phase | Supported | |
| Any other phase thermo (`ideal-surface`, `edge`, …) | Refused | |
| Phase `species` list | Supported | |
| Phase `species` absent, or `all` | **Was confusing** → supported | Cantera's default is every declared species; the key had been mandatory |
| Phase `species` cross-file (`[{gri30.yaml/species: all}]`) | **Was confusing** → refused | had surfaced as serde `invalid type: map` |
| Phase `reactions: all` | Supported | |
| Phase `reactions: none` / `declared-species` / section list | **Was silent** → refused | see §3.3 |
| Phase `kinetics: gas` / `bulk` | Supported | |
| Phase `kinetics: none` / `surface` / `edge` | **Was silent** → refused | `none` means Cantera builds no reactions at all |
| Phase `kinetics` absent | Supported | **deliberate divergence**: Cantera's default is `none`; a mechanism loader is asked for kinetics, so the subset assumes gas kinetics |
| Phase `transport`, `state`, `elements`, `note`, `adjacent-phases`, `skip-undeclared-*`, `explicit-third-body-duplicates` | Supported (ignored) | none can change a rate the subset evaluates |
| Any other phase key | **Was silent** → refused | |
| ck2yaml provenance keys (`generator`, `input-files`, `cantera-version`, `date`) | Supported (ignored) | |
| Any other named top-level section | **Was silent** → refused | see §3.3 |

### 2.6 Reactor features

Cantera's reactor networks, walls, valves, flow devices and 1-D flames have **no
representation in the YAML mechanism format at all** — they are constructed
through the API. There is therefore nothing to parse and nothing to refuse.
Kerotakis' apparatus models (KIN-012 batch and plug-flow, sealed/open headspace,
the heat ledger) already occupy that layer and are not fed from mechanism files.

## 3. Bugs found and fixed

Four defects let a valid file compile into a network that answered a different
question. Each now has a failing-input test in
`crates/kerotakis-core/tests/mechanism_cantera_audit.rs`.

### 3.1 Reaction orders were taken from the *net* stoichiometry

`H + 2 O2 <=> HO2 + O2` is second order in O₂ even though O₂'s net coefficient
is −1. Deriving orders from the net vector produced a second-order rate law
whose pre-exponential had been unit-converted as if it were third order —
wrong rate *and* wrong scale, silently. A species whose net coefficient is zero
disappeared from the rate law entirely.

This is not an unsupported feature; it is arithmetic, inside territory the
subset already claimed. Orders are now taken from the coefficients written on
each side of the equation, forward from the reactant side and reverse from the
product side, matching Cantera's mass-action definition.

**Prevalence in files that would otherwise be candidates:** 6 of 29 reactions in
`h2o2.yaml`, 18 of 325 in `gri30.yaml`.

### 3.2 The default activation-energy unit ignored `energy` and `quantity`

Cantera: *"Setting default units for `energy` and `quantity` will determine the
default units of `activation-energy`, which can be overridden by explicitly
giving the desired units of `activation-energy`."* The parser instead defaulted
to a fixed J/kmol. A file declaring `units: {length: cm, quantity: mol}` — a
very common ck2yaml output shape — had every activation energy misread by a
factor of one thousand, and `units: {energy: cal, quantity: mol}` by 4184.

The derived default is now implemented, `energy` is parsed, and the explicit
`activation-energy` key still overrides it.

### 3.3 Document structure was read without regard to what the phase selects

Two related silences:

- A phase may say `reactions: none`, `reactions: declared-species`, or name
  specific sections. The selector was dropped and every reaction in the file was
  compiled regardless.
- Reaction sections may carry arbitrary names. `ammonia-CO-H2-Alzueta-2023.yaml`
  is a live example: it has no top-level `reactions` key at all, but three named
  sections (`common-reactions`, `baseline-pdep-reactions`,
  `linear-Burke-reactions`), and its two phases differ *only* in which sections
  they take. The old parser reported "mechanism has no reactions" for a file
  containing 54 of them.

Both are now typed refusals. Only `reactions: all` — Cantera's default — is
honoured.

### 3.4 Unknown keys were dropped rather than refused

serde's default is to ignore what it does not recognise. That turned every
Cantera feature the subset does not model into a silent semantic change, and
would have done the same for every feature Cantera adds in future. All six raw
structures (document, units, phase, species, thermo, reaction) now capture
unmatched keys and refuse anything outside a short, documented allowlist of
provenance and annotation fields. `SRI`, `Tsang`, `orders`, `negative-orders`,
`nonreactant-orders`, `negative-A`, nested `units`, `equation-of-state` and
`coverage-dependencies` are all caught by this one change.

Three refusals that already existed were also made legible rather than left as
serde type errors: a unit-bearing `A`, a cross-file phase `species` selector,
and an absent phase `species` key (which is legal, and now means *all*).

Two keys were deliberately left ignorable after checking that they cannot reach
an ideal-gas rate: species `transport` blocks, and species `equation-of-state`
blocks — Cantera itself ignores real-gas parameters for an `ideal-gas` phase,
and every one of Cantera's own gas mechanism files carries them.

### 3.5 Every inspected file is now refused legibly

Running the audited parser over the six files in §1 produces, in each case, a
message that names the actual obstacle:

| File | Refusal |
| --- | --- |
| `gri30.yaml`, `air.yaml` | `reaction 1: reversible pressure-dependent reactions are not supported yet` — the §4 gap, and nothing else |
| `h2o2.yaml`, `nDodecane_Reitz.yaml` | `phase 'ohmech-RK' / 'nDodecane_RK': only ideal-gas kinetics are supported (got thermo 'Redlich-Kwong')` — see §3.6 |
| `ammonia-CO-H2-Alzueta-2023.yaml` | `unsupported Cantera document section 'baseline-pdep-reactions'` — previously "mechanism has no reactions", of a file with 54 |
| `ptcombust.yaml` | `phase 'gas': unsupported value '- gri30.yaml/reactions: declared-species' for Cantera field 'reactions'` |
| `methane_pox_on_pt.yaml` | `phase 'Pt_surf': only ideal-gas kinetics are supported (got thermo 'ideal-surface')` |

That `gri30.yaml` and `air.yaml` both stop at the same line is the evidence
behind §4: one gap stands between the subset and the teaching set.

### 3.6 A document is validated whole, not selected from

`h2o2.yaml` and `nDodecane_Reitz.yaml` each declare **two** phases over the same
species: an `ideal-gas` phase and a Redlich–Kwong real-gas variant of the same
mechanism. The parser requires *every* phase in the document to be ideal-gas, so
it refuses a file whose gas phase it fully supports.

This is a design limit, not a bug — the parser has no notion of "load this
phase" — but it is the second-most likely thing to block BRD-041, and it is
entangled with the phase selectors of §3.3: `DuplicatePhaseAssignment` would
fire on these files even if the real-gas phase were tolerated, because both
phases claim the same species. Fixing it means giving `parse_yaml` a phase to
select rather than a document to validate. Recorded here; not attempted in this
task.

## 4. Smallest additional subset for BRD-041

Measured, not estimated. Feature counts over the inspected files:

| File | reversible three-body | reversible falloff | negative `Ea` | spectator species | PLOG | Chebyshev | NASA9 | explicit `orders` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `h2o2.yaml` (H₂/O₂) | 5 | 1 | 4 | 6 | 0 | 0 | 0 | 0 |
| `gri30.yaml` (CH₄, CO) | 12 | 29 | 32 | 18 | 0 | 0 | 0 | 0 |
| `air.yaml` (N₂/NOₓ) | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| `nDodecane_Reitz.yaml` | 3 | 15 | 49 | 1 | 0 | 0 | 0 | 0 |

The spectator column is fixed by this task (§3.1). Everything else in the
teaching set reduces to **exactly three rate-law additions**:

1. **Reversible three-body reactions.** Equilibrium-constrained reverse rate
   with the third-body concentration factor applied to both directions. The
   forward machinery and the NASA7 detailed-balance path both already exist —
   this is a matter of allowing the two to combine, which the parser currently
   forbids outright.
2. **Reversible falloff reactions** (Troe and Lindemann). Same equilibrium
   reverse; the pressure-dependent correction multiplies both directions.
3. **Negative activation energies.** Drop the `Ea ≥ 0` guard. `Ea` in a fitted
   Arrhenius expression is a fitted parameter, not a barrier height, and
   negative values are ordinary. The guard should become a finiteness check.

Plus one piece of document handling that is not a rate law:

4. **Select a phase instead of validating the whole document** (§3.6). Multi-phase
   files pairing an ideal-gas mechanism with a real-gas variant of the same
   species are common enough that two of the four gas files inspected are built
   that way. This also subsumes the phase reaction selectors of §3.3.

With those and §3.1's fix, `h2o2.yaml`, `air.yaml` and `gri30.yaml` are fully
within the subset's expressive range — H₂/O₂, N₂/NOₓ, and the CH₄ + CO chemistry
that BRD-041 names. Every one of the three is a reversible or sign-relaxed form
of a rate law the subset already evaluates; not one is a new family.

**Not needed for BRD-041's scope:** PLOG (already supported, and unused by every
teaching file inspected), Chebyshev, `linear-Burke`, chemically-activated
reactions, NASA9, explicit reaction orders, negative `A`, sticking coefficients,
electron-collision or plasma rates, transport blocks, and surface phases.

**Needed only past the teaching set:** Troe `A` outside [0, 1] appears twice in
`nDodecane_Reitz.yaml` and across `n-heptane-NUIG-2016.yaml`. Widen that guard
when a large-hydrocarbon mechanism is actually a candidate, not before. Nitrogen
chemistry beyond `air.yaml`'s eight reactions (the ammonia/NOₓ sets) does reach
for `linear-Burke` and PLOG.

**Caveat on syngas.** The CO/H₂ evidence above comes from `gri30.yaml`'s C/O
subset. A standalone syngas set (Davis–Joshi–Wang–Egolfopoulos 2005, or the
Princeton Li/Burke models) is distributed in Chemkin format, not Cantera YAML;
converting one would produce the same three families, but that is inference, not
inspection. §5 makes the point moot — none of them is redistributable anyway.

## 5. Mechanism licence and provenance matrix

The allowlist question is not "is this free to download?" but "can the complete
official payload be distributed through the intended stores?" — CC0, CC BY, or
BSD-style data only, and **the mechanism file's own provenance decides, not
Cantera's**.

| Mechanism | Upstream origin | Licence found | Verdict |
| --- | --- | --- | --- |
| Cantera (software) | Caltech / Sandia / Cantera Developers | BSD-3-Clause | Permissive — but see below |
| Cantera's shipped data files (`gri30.yaml`, `h2o2.yaml`, `air.yaml`) | assembled by third parties | **none granted** | **No-go** as runtime-data; oracle-only |
| GRI-Mech 3.0 | Berkeley / Stanford / UT Austin / SRI, funded by the Gas Research Institute, 1999 | **none** — a warranty disclaimer plus an informal permission sentence scoped to the web documentation | **No-go**; oracle-only |
| Ó Conaire et al. 2004 H₂/O₂ | NUI Galway C3 + LLNL | **none**; both hosts' site-wide terms are restrictive by default | **No-go**; oracle-only |
| Boivin / Sánchez / Williams reduced H₂ | author's CNRS page | **none** — no licence text on the page or in the files | **No-go**; oracle-only |
| Davis–Joshi–Wang–Egolfopoulos 2005 H₂/CO | original host defunct; Stanford mirror | **none** — citation request only | **No-go**; oracle-only |
| Li 2007 / Burke 2012 (Princeton) | official pages now 403 | **none**, plus an express `(c) … Princeton University, 2011` in the file | **No-go**; weakest chain of title in the set |
| FFCM-1 | Stanford / SRI, 2016 | **none** — "How to Cite" plus a `Copyright © 2016` footer | **No-go**; oracle-only |
| San Diego mechanism | UCSD | **none** — citation request; UCSD site-wide terms restrictive | **No-go**; oracle-only |

Aggregators were checked for a cleaner channel and none exists. The ReSpecTh
OSF deposit is genuinely CC BY 4.0 but contains experimental datasets, not
mechanisms. The CRECK repository has no licence file. At least one GitHub
collection re-licenses several of these mechanisms as MIT without any evident
standing to do so — that is weaker evidence than Cantera's own honest
disclaimer, not stronger, and it must not be used as a channel.

**Ordered candidate list, with lanes.** Ordering is by educational value for
BRD-041; every entry lands in the same lane.

1. H₂/O₂ — `h2o2.yaml` (a GRI-Mech 3.0 derivative) or Ó Conaire — **oracle-only**
2. CH₄ — GRI-Mech 3.0 — **oracle-only**
3. CO/H₂ syngas — Davis 2005 or Li/Burke — **oracle-only**
4. N₂/NOₓ — `air.yaml` — **oracle-only**
5. Reduced H₂ — Boivin — **oracle-only**
6. FFCM-1, San Diego — **oracle-only**

No mechanism on the ordered list is a `runtime-data` candidate under the current
allowlist. Machine-readable records for each are in `provenance/sources.toml`
with `lane = "oracle-only"` and `decision = "rejected-runtime-data"`.

### 5.1 Correction to a standing roadmap claim

`PLAN.md` asserted in three places that Cantera's shipped mechanism files are
covered by its BSD-3 licence and are therefore freely redistributable — line 375
("the shipped data files — gri30.yaml etc. — are part of Cantera's BSD-3
distribution and freely redistributable"), line 1134 ("Mechanism YAML files are
part of the BSD-3 distribution"), and the artifact-lane table at line 1359 ("the
BSD-3 Cantera copy is the clean channel").

**This audit disagrees, and Cantera itself disagrees.** The `cantera-example-data`
repository states plainly that the project "is not the original author of the
reaction mechanisms included in this repository and is not claiming to grant a
license to them", and `data/README.md` in the main repository (added 2025-09-22)
says input files are "provided for illustration purposes only". The BSD-3 text
covers the software; there is no licence file in the example-data repository at
all. GRI-Mech's own site carries a liability disclaimer and a citation request,
never a redistribution grant.

ROADMAP-Webapp.md's artifact-lane table was already correct — "Cantera's code
licence does not prove every imported mechanism/data file is redistributable" —
and this audit confirms that line while contradicting PLAN.md's. PLAN.md is
corrected in this change.

PLAN.md line 1134 also claimed the GRI-Mech hydrogen subset was "already used
… (KIN-008)". No mechanism data exists anywhere in the tree and
`provenance/sources.toml` has never held a Cantera record. The claim was
unsupported in fact as well as in licence; it is removed.

### 5.2 What this means for BRD-041

BRD-041 cannot ship any of the audited mechanisms as runtime data. Three routes
remain open, in order of preference:

1. **Author project-original reduced mechanisms** from primary-literature rate
   constants, one reaction at a time, each with its own source record — the
   pattern curated kinetics already uses (KIN-001…003). Rate constants are
   facts, not expression; a Kerotakis-authored network citing measurements is
   not a derivative of anyone's mechanism file. Slowest, cleanest, and the only
   route that yields a licence Kerotakis controls.
2. **Find a mechanism with an actual open licence.** CC BY 4.0 mechanism
   deposits do exist on Zenodo. None was found for H₂/O₂, CH₄ or syngas at the
   teaching scale, but this is a search that can be repeated.
3. **Seek written permission** from the upstream authors for the specific
   mechanisms wanted, recorded as a `LicenseRef-` grant. Several of these are
   single-institution artefacts with reachable authors.

Every route keeps the audited files as **oracle-only** differential references,
which is exactly what BRD-041's acceptance criterion asks for ("Cantera
differential oracle for ignition delay/species traces"). The oracle role is
unaffected by any of this — running Cantera outside the repository to check
Kerotakis' answers ships nothing.

## 6. C-API gap list

**Verdict: no gap. BRD-042 stays parked.**

The question is whether any audited need requires Cantera's C API rather than
extending the portable parser. Working through what BRD-041 actually asks for:

| BRD-041 need | Portable path | C API required? |
| --- | --- | --- |
| Parse the mechanism format | `parse_yaml`, extended per §4 | No — the format is publicly specified, and §4's additions are arithmetic plus phase selection |
| Evaluate Arrhenius / three-body / Troe / PLOG rates | `RateExpression` + `PressureDependence`, already executing | No |
| Reverse rates from thermochemistry | NASA7 detailed balance, already executing | No |
| Stiff integration of the network | diffsol (KIN-004/005), already executing | No |
| Equilibrium endpoints | CEA path (`kerotakis-cea`), already shipping | No |
| Ignition delay, species traces | batch/plug-flow apparatus models (KIN-012) | No |
| Element and energy conservation | reaction-network conservation lint (KIN-003) | No |
| Differential validation against Cantera | oracle, outside the repository | No — an oracle is a subprocess, not a linked library |
| Transport properties (viscosity, diffusion, thermal conductivity) | **not implemented** | Would need new work — but BRD-041 does not ask for it, and it is a self-contained kinetic-theory calculation, not a reason to link a C++ engine |
| Surface/interface kinetics | KIN-011 heterogeneous rates has its own IR | No |

The one genuine capability the portable path lacks is mixture transport, and
BRD-041 does not require it. Even there the argument for the C API is weak:
mixture-averaged transport from Lennard-Jones parameters is a few hundred lines
against data the YAML files already carry.

Three further reasons the API is the wrong instrument here, all independent of
capability:

- **It would not solve the actual blocker.** §5 is a licensing problem, not an
  engineering one. Linking Cantera does not make GRI-Mech redistributable; the
  mechanism file's provenance is unchanged by what reads it.
- **Lane conflict.** ROADMAP-Webapp.md places Cantera code at `oracle-only`
  initially, and BRD-042's own acceptance demands "measurable capability gain
  that BRD-041 cannot reasonably supply". This audit finds none.
- **Target cost.** BRD-042 requires the binary to build for desktop, wasm, iOS
  and Android with no C++ types crossing the boundary, for a capability the
  portable path already has.

Record: **no-go for now.** The finding to re-open BRD-042 would be a required
capability that is genuinely infeasible in Rust — mixture transport is not that,
and neither is anything else BRD-041 needs.

## 7. Scope boundaries observed

- No FFI, no new dependency, no `Cargo.toml` change.
- No mechanism file in the diff or in the tree.
- No new rate-law family implemented. The reversible three-body and reversible
  falloff work identified in §4 belongs to BRD-041.
- Files fetched for inspection stayed outside the repository.
- **Not done, deliberately:** no numerical cross-check against a running
  Cantera. This audit compares the parser to the *specification* and to real
  files' structure; it does not claim rate-for-rate agreement. The differential
  oracle is BRD-041's acceptance criterion, and it needs the §4 additions before
  there is anything to compare. Where a claim here rests on Cantera's observed
  behaviour rather than its documentation — `(+M)` without a `type` being read
  as a three-body reaction, the undocumented `type` aliases, unbounded Troe `A`,
  legal negative element counts — that came from a live Cantera 3.2.0 install
  outside the repository, and is marked as such above.

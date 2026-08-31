# BRD-030 — direct feos integration spike

**Date:** 2026-08-30 · **Branch:** `brd030/feos-spike` · **Reference:** `feos`
0.10.1 / `feos-core` 0.10.1 (crates.io, 2026), MIT OR Apache-2.0.

This is a decision gate. No dependency shipped, the workspace `members` list
is untouched, and the prototype lives in `spikes/brd-030-feos/` behind its own
`[workspace]` so nothing in it can reach a production build, a lockfile, a
`cargo deny` run, or a NOTICE.

**Verdict: GO, scoped and conditional.** feos is adopted as an *additional*
engine for the fluids and properties `kerotakis-thermo` has no model for at
all, and as a build-time differential oracle for the ones it does. It replaces
nothing that works today.

The corpus is what scoped it. Of 550 comparison rows, **351 are a coverage gap
and every one runs the same way** — feos answers, `kerotakis-thermo` has no
model. But on the 180 rows where both engines speak, PC-SAFT is *less* accurate
than the existing Antoine + UNIFAC route on exactly the aqueous binaries the
teaching bench is built around (median 2.15 °C, worst 11.0 °C, § 3.4). The two
engines are good at disjoint things. Zero rows were classified `our-bug`, and
`kerotakis-thermo`'s Peng-Robinson matched feos's to 4.7 × 10⁻¹¹ (§ 3.3).

The four conditions, the reasoning, and the strongest argument against the
recommendation are in § 8.

## 1. Method

Three engines answer the same questions, defined once in
`spikes/brd-030-feos/corpus.json` and read by both the Rust driver and the
Python referee so no engine gets asked a slightly different question:

| engine | what it is |
| --- | --- |
| `kerotakis` | `kerotakis-thermo` today: Antoine vapour pressure × UNIFAC activity coefficients, bisected (`vle.rs`, `unifac.rs`) |
| `kerotakis-pr` | `kerotakis-thermo`'s Peng-Robinson (`eos.rs`, THERMO-007) |
| `feos-pcsaft` | feos PC-SAFT with the Esper et al. 2023 parameter set |
| `feos-pcsaft-gross2002` | the same feos code with the *original* Gross & Sadowski 2002 parameters — present only to separate a model difference from a parameter difference |
| `feos-pr` | `feos_core::cubic::PengRobinson`, fed byte-identical Tc/Pc/ω |
| `adapter` | the prototype adapter, feos dressed as `kerotakis_thermo::fluid::FluidModel` |
| `oracle-*` | the referee: the Python `thermo` package (MIT, Caleb Bell), extending the CAP-19 pattern already in `tools/gen-thermo-fixtures.py` |

The referee speaks in three voices on purpose, because "kerotakis and feos
disagree" is not a finding until you know which one moved:

* `oracle-unifac-kero` — `thermo`'s UNIFAC driven with **kerotakis-thermo's
  own Antoine constants**, duplicated into `oracle.py` exactly as the existing
  CAP-19 script duplicates them. Same model, same parameters, independent
  code: a gap here can only be an implementation difference.
* `oracle-unifac-corr` — `thermo`'s UNIFAC with `thermo`'s own vapour
  pressures. A fully independent answer, available for every binary in the
  corpus including the ones `kerotakis-thermo` cannot express.
* `oracle-corr` / `oracle-pr` — `thermo`'s pure-fluid correlations and its own
  Peng-Robinson, for the pure-fluid and cubic rows.

Every engine emits a row even when it cannot answer. A refusal carries its
reason, and `compare.py` classifies it rather than dropping it. That is the
whole design: **the corpus is mostly a coverage measurement, and averaging the
blanks away would have deleted the finding.**

Corpus size: **22 pure fluids** — water, four alcohols, two ketones, ethyl
ethanoate, ethanoic acid, six *n*-alkanes from propane to octane, cyclohexane,
toluene, chloroform, CO₂, N₂, O₂ and NH₃ — at three or four temperatures each,
and **20 binaries** at five liquid compositions each plus a TP flash, all at
101.325 kPa. Six of the 22 pure fluids and ten of the 20 binaries are ones
`kerotakis-thermo` can express at all; the rest are deliberately outside it,
because measuring the gap is half the point.

## 2. What feos 0.10.1 actually is

### 2.1 Shape

`feos` is a thin model crate over `feos-core`. Default features are **empty** —
no model compiles unless asked for. Building with `features = ["pcsaft"]` and
`default-features = false` pulls 55 crates in total:

```
feos 0.10.1, feos-core 0.10.1, nalgebra 0.35.0, num-dual 0.14.2,
quantity 0.14.1, simba 0.10.2, wide 1.7.0, indexmap 2.14.1,
itertools 0.15.0, csv 1.4.0, serde 1.0.229, serde_json 1.0.151,
thiserror 2.0, num-traits 0.2.19, libm 0.2.16
```

(The project does not track `Cargo.lock`, so the exact resolved versions the
measurements were taken against are written out here rather than left to a
lockfile.)

All pure Rust. No BLAS/LAPACK, no mmap, no `getrandom`, no `libc`. This
matters more than any benchmark: the reason the earlier roadmap note said
"wasm ✓" is that there is nothing in the default dependency graph that
*could* stop it.

Three specific things that were checked rather than assumed:

* **There is no `python` feature in 0.10.1 at all.** It existed through 0.8.0
  and was removed in 0.9.0; the PyO3 bindings now live in a separate `py-feos`
  crate that is not published to crates.io. There is no code path from the
  crates.io crate to `pyo3` or `numpy`.
* **`rusqlite` is the one C dependency in the tree**, and it is an *implicit*
  optional feature (`dep:rusqlite` is never declared in `[features]`, so Cargo
  auto-creates the name) pulling SQLite with `bundled`, i.e. a `cc` build. It
  is off by default and gated `#[cfg(feature = "rusqlite")]` in
  `feos-core/src/parameter/mod.rs`. **Never enable it**; doing so would put a
  bundled C SQLite into a wasm and mobile build for the sake of an optional
  parameter loader.
* **`rayon` likewise stays off.** It is the other thing upstream's own
  Pyodide build disables.

Edition 2024 needs Rust ≥ 1.85, but `quantity` and `num-dual` both declare
`rust-version = "1.89"`, so 1.89 is the effective floor. The workspace is on
1.96.

### 2.2 Model inventory, against what kerotakis-thermo already has

| feos model | feature | kerotakis-thermo counterpart | disposition |
| --- | --- | --- | --- |
| PC-SAFT (+ association, dipolar, quadrupolar) | `pcsaft` | none | **adds** |
| gc-PC-SAFT (homo/heterosegmented) | `gc_pcsaft` | none | adds (needs SMARTS/RDKit for group assignment — Python-only upstream) |
| ePC-SAFT (electrolytes) | `epcsaft` | none (PHREEQC covers aqueous ions by a different route) | leave alone |
| PeTS, UV-theory, SAFT-VR Mie, SAFT-VRQ Mie | various | none | not needed by BRD-000/014 |
| multiparameter (Helmholtz reference EOS) | `multiparameter` | none | adds, but see § 4 |
| Peng-Robinson | `feos-core::cubic`, ungated | `eos.rs` `PengRobinson` (THERMO-007) | **differential oracle only** — see the caveat below |
| Joback / DIPPR-form ideal gas | always | none | adds |
| **activity-coefficient models (UNIFAC, NRTL, Wilson, UNIQUAC)** | — | `unifac.rs`, and it is the heart of the crate | **feos has none. Nothing here is replaced.** |

Two caveats on that table.

**The cubic is a teaching example, not a product.** `feos_core::cubic` is
ungated — it compiles with zero features — but its own module doc says it
"acts as a reference on how a simple equation of state … can be implemented".
It is 234 lines, Peng-Robinson only, and there is no SRK. It is an excellent
independent second opinion on THERMO-007's arithmetic (§ 3 uses it as exactly
that) and a poor reason to depend on feos.

**gc-PC-SAFT's group assignment is Python-only upstream.** Building parameters
from a SMILES string goes through `sauer2014_smarts.json` and RDKit, in feos's
Python bindings. The Rust side takes group *counts*, not structures. If
Kerotakis ever wants gc-PC-SAFT it must do the group decomposition itself —
which, notably, `unifac.rs` already does for its own group set.

The activity-coefficient row is the single most consequential fact in this report and it is
easy to miss when reading feos's feature list, because the list is long. feos
is a *residual-Helmholtz-energy framework*: every model in it is an equation of
state. The γ–φ route — Antoine vapour pressures times UNIFAC activity
coefficients, which is what `kerotakis-thermo` is — is a different formalism,
and feos does not implement it. **Adopting feos cannot retire `unifac.rs`,
`vle.rs`, or the ethanol–water still.**

### 2.3 API surface used

`PhaseEquilibrium::{pure, bubble_point, dew_point, tp_flash}`,
`State::{new_npt, critical_point}`, `State::{ln_phi, compressibility,
residual_molar_enthalpy, density}`, `Parameters::from_json`. Units are carried
in the type by the `quantity` crate, which caught two unit errors in this
spike's own driver at compile time — a point in feos's favour given that
`vle.rs`'s own doc comment records boiling water at 51.9 °C from exactly that
class of mistake.

## 3. Results

Full three-way table: `spikes/brd-030-feos/fixtures/discrepancies.tsv`.
Raw engine output: `fixtures/engines.tsv`; referee output: `fixtures/oracle.tsv`.

### 3.0 The referee, checked first

A referee nobody has checked is not evidence. Before it was allowed to judge
anything, `oracle-unifac-corr` was run against six binaries whose azeotropes
are textbook numbers:

| system | referee | literature | |
| --- | --- | --- | --- |
| ethanol–water | 78.15 °C at x = 0.9, y₁ = 0.899 | 78.15 °C at x = 0.894 | ✓ |
| acetone–chloroform | **maximum** at 64.3 °C near x = 0.3 | maximum-boiling, 64.5 °C at x ≈ 0.34 | ✓ |
| methanol–cyclohexane | minimum 53.4 °C | ≈ 54 °C | ✓ |
| hexane–ethanol | minimum 58.6 °C | 58.7 °C | ✓ |
| ethyl ethanoate–ethanol | minimum 71.2 °C | 71.8 °C | ✓ |
| hexane–heptane | monotonic, no azeotrope | near-ideal, none | ✓ |
| methanol–water | monotonic to 66.0 °C | none, MeOH boils 64.7 °C | ✓ |

The acetone–chloroform row is the one worth pausing on: the referee reproduces
a *maximum*-boiling azeotrope — the negative-deviation case, where the mixture
boils higher than either pure component. It is also a binary
`kerotakis-thermo` cannot express at all, because UNIFAC main group 23 (CCl₃)
is not in `approved_table()`. The bench can currently teach the ethanol–water
azeotrope and not its opposite.

### 3.1 Three-way comparison

1 208 engine rows and 731 referee rows join into **550 comparison rows**.
Every row carries exactly one class; none is averaged away.

| class | rows | |
| --- | ---: | --- |
| `coverage-gap` | **351** | one engine has no model for the fluid or property at all |
| `agree` | 134 | every engine that answered agreed inside tolerance |
| `feos-difference` | 42 | feos differs from the UNIFAC consensus |
| `parameter-difference` | 12 | same feos code, two published parameter sets |
| `single-phase-refusal` | 8 | feos raises where kerotakis returns β = 0 or 1 |
| `oracle-limitation` | 2 | referee answered only from a Perry's/DIPPR correlation |
| `range-refusal` | 1 | kerotakis declines to extrapolate Antoine |
| **`our-bug`** | **0** | — |
| **total** | **550** | |

### 3.2 The headline: 351 of 550 rows are a coverage gap, and all of them the same way round

| quantity | answered by | rows |
| --- | --- | ---: |
| liquid density | feos only | 66 |
| enthalpy of vaporisation | feos only | 64 |
| vapour pressure | feos only | 51 |
| bubble temperature | feos only | 50 |
| vapour composition y₁ | feos only | 50 |
| critical pressure | feos only | 22 |
| critical temperature | feos only | 22 |
| normal boiling point | feos only | 16 |
| vapour fraction | neither | 10 |

`kerotakis-thermo` answered 134 of 496 questions put to it and refused 362 —
a **73 % refusal rate**. feos answered 478 of 496 and refused 18. Not one of
the 351 gaps runs the other way: there is no quantity in this corpus that
`kerotakis-thermo` can compute and feos cannot.

The refusals are honest and well-labelled, which is the crate working as
designed — 70 of them are "no liquid-density model", 70 "no enthalpy model",
67 "no Antoine constants curated", 44 "critical properties for 3 fluids only".
But a refusal is still a question the bench cannot answer.

### 3.3 `our-bug`: zero rows, and that is a real result

`oracle-unifac-kero` runs `thermo`'s UNIFAC against **kerotakis-thermo's own
Antoine constants**: same model, same published parameters, independently
written code. Across every binary where both could answer, `kerotakis` matched
it to six significant figures. Whatever else this spike found, it found no
arithmetic error in `vle.rs` or `unifac.rs`.

The Peng-Robinson row is stronger still. Fed byte-identical Tc/Pc/ω,
`kerotakis-thermo`'s `eos.rs` and `feos_core::cubic` agree across all 54
states to a **maximum relative deviation of 4.7 × 10⁻¹¹** — the same equation,
implemented twice, both right, including the compressed-liquid states where a
Newton iteration started from Z = 1 could plausibly have found the wrong root.
THERMO-007 is independently confirmed. (`thermo`'s own cubic solve is
converged only to ≈ 4 × 10⁻⁴, which is why the tolerance for these three
quantities had to be loosened from my initial 1 × 10⁻⁶ — the measurement
corrected the assumption rather than the other way round.)

### 3.4 `feos-difference`: PC-SAFT is *worse* than UNIFAC on the bench's own mixtures

42 rows, all binaries, median deviation **2.15 °C** in bubble temperature and
**0.037** in vapour composition, worst case **11.0 °C**.

| binary | x₁ | kerotakis + referee | feos PC-SAFT | Δ |
| --- | ---: | ---: | ---: | ---: |
| ethanoic acid–water | 0.9 | 111.67 °C | 100.66 °C | **11.0** |
| ethanoic acid–water | 0.7 | 104.36 °C | 97.54 °C | 6.8 |
| acetone–water | 0.1 | 67.73 °C | 71.81 °C | 4.1 |
| ethanol–water | 0.1 | 85.81 °C | 89.22 °C | 3.4 |
| acetone–ethanoic acid | 0.5 | 73.22 °C | 76.00 °C | 2.8 |

An honest caveat first: the two UNIFAC referee voices are **not two independent
votes**. They share a model and its parameters, so "kerotakis and the referee
agree" here means "two implementations of UNIFAC agree", which § 3.3 already
established. The question of which model is *right* has to be settled against
literature, not against the table.

It settles against UNIFAC. Ethanol–water at x = 0.1 boils at about 86 °C; feos
says 89.2 °C. Ethanoic acid–water at x = 0.9 must approach acetic acid's
117.9 °C boiling point; UNIFAC says 111.7 °C and feos says 100.66 °C — barely
above pure water, which is qualitatively wrong. Acetic acid dimerises in the
vapour, and PC-SAFT with these parameters and **no fitted binary interaction
parameter** (`rehner2023_binary.json` has no acetic acid–water record) does not
capture it.

This is the most important single result in the report, and it points the
opposite way from "adopt feos". **On the aqueous, strongly non-ideal binaries
the teaching bench is built around, the existing Antoine + UNIFAC route is more
accurate than PC-SAFT, and a missing k_ij degrades PC-SAFT silently.** feos
earns its place on the fluids and properties `kerotakis-thermo` cannot express
at all — not on the ones it already does well.

### 3.5 `parameter-difference`: the parameter set moves the answer more than the engine does

Twelve rows, same feos code, same model family, two published PC-SAFT sets:

| case | quantity | Esper 2023 | Gross 2002 | referee |
| --- | --- | ---: | ---: | ---: |
| water @ 25 °C | liquid density | 57 246 | 51 178 | 55 345 mol/m³ |
| water @ 100 °C | liquid density | 53 477 | 48 756 | 53 197 mol/m³ |
| methanol @ 25 °C | Δh_vap | 37.17 | 35.27 | 37.46 kJ/mol |
| ethanoic acid @ 25 °C | Δh_vap | 43.75 | 37.76 | 23.42 kJ/mol |

Water's liquid density differs by **12 %** between two peer-reviewed parameter
sets for the same equation — Esper 2023 lands +3.4 % from the referee, Gross
2002 −7.5 %. This is the concrete argument for BRD-031's existence: choosing
feos is not a decision, choosing *which published parameters* is, and the
second decision has the larger error bar.

(The acetic acid Δh_vap row is a physics subtlety, not a straight error: the
referee's 23.4 kJ/mol is the *apparent* enthalpy, depressed by vapour-phase
dimerisation, while PC-SAFT's association term reports the monomer value.
Comparing them directly is the mistake; the row is kept, and labelled, rather
than dropped.)

### 3.6 `single-phase-refusal`: an API contract difference worth knowing before BRD-032

feos's `tp_flash` raises `"No phase split according to stability analysis."` on
8 of the 20 flash cases. kerotakis returns β = 0 or β = 1. Both are correct;
feos's stability analysis is arguably the better answer and its error is
informative. But a routing layer that expects a number will treat the two
engines' single-phase behaviour differently, and that has to be handled
explicitly rather than discovered.

### 3.7 The adapter is transparent

Across all 50 binary bubble points, the `adapter` column and the direct
`feos-pcsaft` column differ by **exactly 0.000 °C**. The wrapper adds nothing
of its own — which is what a seam prototype is supposed to prove.


## 4. Parameter provenance

This is where the answer is least comfortable and most important.

### 4.1 The published crate ships almost no parameters — with one exception

`feos` 0.10.1 on crates.io contains `src/`, `tests/` and `benches/`. There is
no `parameters/` directory in the tarball; the doc example's
`"../../parameters/pcsaft/esper2023.json"` is a path into the **git
repository**, which is not published. The `.crate` carries `license-mit` and
`license-apache` at its root and no data-specific notice anywhere.

**The exception, and it is a real one:**
`feos-0.10.1/src/ideal_gas/joback.rs` compiles the Joback & Reid (1987)
group-contribution table straight into the crate as four `const [f64; 22]`
arrays (`A`, `B`, `C`, `D`) plus a `GROUPS` name array. That is the complete
published table — the same numbers as the repository's
`parameters/ideal_gas/joback1987.json` — shipped as Rust source under
MIT OR Apache-2.0, attributed only by a DOI in a doc comment
(doi:10.1080/00986448708960487). `pub mod ideal_gas;` is **not** feature-gated
in `lib.rs`, so it compiles in even with `default-features = false,
features = ["pcsaft"]`. A code-only adoption of feos therefore does ingest one
third-party literature table, whether or not anything calls it.

The table is 88 fitted constants from a 1987 journal paper, attributed, and
upstream has taken the position that it is MIT/Apache-licensable. That is a
defensible position for individually published scientific constants, and it is
a much smaller exposure than any of the JSON files. But it is not zero, and the
NOTICE/`sources.toml` record BRD-031 writes must name it rather than repeating
the "feos ships no data" line that a quick look at the tarball suggests.

Two further consequences:

* Nothing else about feos's licensing changes what Kerotakis would have to
  clear. Adding the crate adds MIT/Apache-2.0 Rust code, one small literature
  table, and no fluid parameters.
* A feos-backed route is useless until BRD-031 supplies parameters. **The code
  is the easy half.** Every model in § 2.2 marked "adds" is a promise that
  BRD-031 has to fund.

### 4.2 What is in the repository, and under what terms

`github.com/feos-org/feos` carries `parameters/` with per-directory READMEs.
The pure-component PC-SAFT sets each name one publication:

| file | content | source |
| --- | --- | --- |
| `gross2001.json` | 78 non-associating, non-polar substances | Gross & Sadowski, *Ind. Eng. Chem. Res.* 40(4) 1244 (2001), doi:10.1021/ie0003887 |
| `gross2002.json` | 18 associating substances (water, alcohols, acetic acid) | Gross & Sadowski, *IECR* 41(22) 5510 (2002), doi:10.1021/ie010954d |
| `gross2005_literature.json`, `gross2005_fit.json` | quadrupolar (CO₂, N₂, …) | Gross, *AIChE J.* 51(9) 2556 (2005), doi:10.1002/aic.10502 |
| `gross2006.json` | 24 dipolar (acetone, esters, ethers) | Gross & Vrabec, *AIChE J.* 52(3) 1194 (2006), doi:10.1002/aic.10683 |
| `esper2023.json` | 1842 substances — the set feos recommends | Esper et al., *IECR* (2023), doi:10.1021/acs.iecr.3c02255 |
| `rehner2023_binary.json` | 7848 binary interaction/cross-association records | Rehner et al., *Int. J. Thermophys.* (2023), doi:10.1007/s10765-023-03290-3 |
| `sauer2014_*.json`, `loetgeringlin*.json`, `rehner2020.json`, `eller2022.json` | group-contribution, viscosity, surface-tension, hydrogen sets | one DOI each |

Three findings about the terms:

1. **feos is licence-aware about DIPPR and excludes it — genuinely.**
   `parameters/ideal_gas/README.md` states plainly: *"The parameters published
   in the DIPPR database itself are not publicly available. If you have a valid
   license, contact us to obtain a compatible input file."* The Rust module
   `src/ideal_gas/dippr.rs` implements the DIPPR *equation forms* (`DIPPR100`,
   `DIPPR107`, `DIPPR127`) as an enum and carries **no coefficients**. No DIPPR
   table is in the repository. This is exactly the behaviour BRD-031 requires,
   and it is evidence that the upstream project thinks about this at all.
   Likewise: no UNIFAC anywhere — no model, no parameters, no mention — and no
   NIST SRD or REFPROP reference in the source. On the three sources this
   project forbids by name, feos is clean.

2. **One ideal-gas file is a transcription of a copyrighted book.**
   `parameters/ideal_gas/poling2000.json` (97 KB) is described as *"correlation
   parameters published in 'The Properties of Gases and Liquids, 5th edition'"*
   — Poling, Prausnitz & O'Connell, McGraw-Hill. There is no DOI, no permission
   note, and no separate licence. Individual fitted constants are facts, but a
   97 KB extraction of a book's appendix tables is the shape of thing an EU
   database right attaches to. **BRD-031 must not ingest `poling2000.json`.**
   Nothing in this spike used it; the enthalpy comparison here is built from
   residual enthalpies, which need no ideal-gas heat capacity at all (§ 3).

3. **`parameters/multiparameter/coolprop.json` carries a live upstream
   attribution defect, and it is the sharpest single finding in this report.**
   The file is 467 KB of *actual* Helmholtz-energy coefficients for 124 fluids
   taken from CoolProp — CO₂ as the 42-term Span–Wagner form, water as the
   56-term IAPWS-95 form, and so on. CoolProp is MIT, and MIT requires the
   copyright notice and permission text to travel with substantial portions of
   the work. In feos's copy the CoolProp notice is not retained, and CoolProp's
   per-fluid BibTeX references — the citations to the original reference-EOS
   papers — have been stripped. The `parameters/multiparameter/README.md`
   sentence "the list of pure fluids contained in the open-source software
   CoolProp" is the only acknowledgement, and it understates the file: this is
   the data, not a list.

   Kerotakis must not ingest `coolprop.json`, and the `multiparameter` feature
   must stay out of scope. This is not merely BRD-092's parked question about
   whether CoolProp is worth its runtime weight; it is a licence-compliance
   defect in the intermediary. If any feos parameter file is ever adopted, the
   reference-EOS coefficients must come from CoolProp directly, with CoolProp's
   own notice, or from the primary publications. **Owner:** whoever holds the
   BRD-031 licence decision; worth reporting upstream as an issue, since it is
   very likely an oversight rather than a policy.

4. **The `parameters/` tree carries no licence statement of any kind.** The
   full recursive repository tree contains exactly two kinds of licence file:
   the root `license-mit` / `license-apache` and per-crate copies under
   `crates/feos*/`. There is no LICENSE, NOTICE, COPYING, permission note, or
   disclaimer anywhere under `parameters/` or `parameters_old/`, and grepping
   every `parameters/*/README.md` for licence language returns exactly one hit
   — the DIPPR refusal quoted above. The root README says only that the
   repository "contains JSON files with previously published parameters for
   the different models."

   So the tables fall under the root dual licence *by omission, not by
   affirmative grant*. That is feos's grant of whatever rights it holds in the
   compilation; it is not, and cannot be, a grant of rights the original
   publishers hold. For a project whose provenance bar requires a positive
   licence answer, **silence is not a clearance**. BRD-031 must write its own
   record for any table it ingests, reasoning from the primary publication,
   rather than inheriting a claim from feos.

### 4.3 What this spike committed

Nothing third-party. `fetch-parameters.sh` downloads the parameter files into
a gitignored directory; only computed fixtures and this report are in the
diff. `provenance/sources.toml` therefore carries no new record, per the same
reasoning as BRD-040's closing section: the manifest inventories inputs the
tree actually uses, and this tree uses none.

Checksums of the files the measurements were taken against are in
`spikes/brd-030-feos/fixtures/parameters.sha256`.

### 4.4 A provenance finding about the *referee*

36 of the referee's pure-fluid points — liquid density and enthalpy of
vaporisation for propan-1-ol, propan-2-ol, butanone, ethyl ethanoate, acetic
acid and chloroform — came back from `thermo` with `method=DIPPR_PERRY_8E`,
i.e. Perry's Chemical Engineers' Handbook correlation coefficients. Those
rows are classified `oracle-limitation` and are *not* counted as evidence
either way. Every vapour pressure used IAPWS, a CoolProp-derived Helmholtz
fit, or the Wagner–McGarry correlation, all of which are open literature. This
is worth recording because the same trap is waiting for BRD-031: `thermo` is
MIT code wrapped around a data set that is not uniformly clear, and the
project's existing use of it as a build-time oracle (CAP-19) should stay
build-time.

## 5. Feasibility

Measured on the project VPS (4 cores, 8 GB, `CARGO_BUILD_JOBS=2`, shared with
other builds), release profile identical to the workspace's (`lto = "thin"`,
`codegen-units = 1`, `strip = "debuginfo"`). These are spike-grade
order-of-magnitude numbers, not benchmarks.

### 5.1 Compile time and memory

| build | wall | peak RSS |
| --- | ---: | ---: |
| cold `cargo build --release` — feos + 54 deps + the spike | **1 m 37 s** | 262 MiB |
| cold `--target wasm32-unknown-unknown --lib` — feos + adapter | **1 m 30 s** | 75 MiB |
| the two wasm size probes | 53 s | — |
| baseline: `cargo build --release -p kerotakis-thermo` alone | **17 s** | 262 MiB |

So feos costs roughly **+80 seconds of cold compile** against the crate it
would sit beside, on two cores. That is real but unremarkable: it is a fifth of
what `sundials-kinetics-rs` costs this workspace today, and it is fully cached
after the first build. Memory is a non-issue — the wasm build peaks at 75 MiB.

Build-directory cost is less comfortable: `378 MiB` of `target/` for the spike
against `44 MiB` for `kerotakis-thermo` alone. On a VPS that has twice hit
`No space left on device` during this spike, that is worth knowing.

### 5.2 Native artefact size

| artefact | size |
| --- | ---: |
| `libkerotakis_thermo.rlib` | 590 KB |
| `libbrd030_feos_spike.rlib` (adapter + feos linked in) | 2.9 MB |

### 5.3 Browser size — the number that decides it

Two `cdylib`s with the same exported signature, the same release profile, and
one difference: which engine computes the bubble point. Neither loads
parameters from disk; both build them in code, because that is what a browser
build has to do (§ 5.4).

| probe | raw `.wasm` | gzipped |
| --- | ---: | ---: |
| `wasm-probe-base` — `kerotakis-thermo` Antoine + UNIFAC | 68 537 B (67 KB) | **26 933 B (26 KB)** |
| `wasm-probe-feos` — feos PC-SAFT | 1 130 432 B (1.08 MB) | **372 461 B (364 KB)** |
| **delta attributable to feos** | +1 061 895 B (+1.01 MiB) | **+345 528 B (+337 KB)** |

The raw number looks alarming and the gzipped one is what counts:
`tools/bundle-budget.sh` sets this project's budget at **1 MiB gzipped** per
wasm module, and feos costs **+337 KB gzipped** — about a third of the budget
for one module, on top of a 26 KB baseline. That is a real cost and it is
affordable. I had expected the raw 1 MiB figure to be the argument that killed
the browser case; measuring it compressed is what stopped that from being
written down as a conclusion.

Three qualifications, in both directions:

* `wasm-opt -Oz` was **not available in this environment**, so these are
  unoptimised. `tools/build-web.sh` runs it on the real bundle, so the true
  delta is smaller than 337 KB, not larger.
* The probe links one model (`pcsaft`) and one calculation family. It is a
  floor for feos: `all_models` would be considerably more.
* It excludes parameters entirely, and § 5.4 explains why a browser build must
  embed them. `esper2023.json` is 872 KB raw — though a build-time step that
  emits only the fluids BRD-031 actually clears, as records rather than JSON
  text, would cost a small fraction of that.

### 5.3.1 Per-call speed

Order-of-magnitude only, release build, single-threaded:

| operation | time/call |
| --- | ---: |
| kerotakis Peng-Robinson Z | < 1 ns (inlined away) |
| feos Peng-Robinson Z | 3.7 µs |
| feos PC-SAFT vapour pressure | 81 µs |
| kerotakis Antoine + UNIFAC binary bubble point | 933 µs |
| feos PC-SAFT binary bubble point | 2.35 ms |
| feos parameter load, `esper2023.json` (1 component) | 17.6 ms |
| feos parameter load, esper2023 + rehner2023 binary | 92.5 ms |

feos's bubble point is about **2.5× slower** than the existing route — the same
order of magnitude, and irrelevant for a bench that solves one at a time. The
number that would matter in a browser is the 92 ms parameter load, which is
JSON parsing of a 4.2 MB binary-interaction file and is exactly what § 5.4's
`include_str!` constraint forces you to replace anyway.



### 5.4 The wasm verdict: **yes, with one design constraint**

The 2026-08 roadmap claim re-verifies at 0.10.1. Three independent
confirmations:

1. `cargo build --release --lib --target wasm32-unknown-unknown` on the spike's
   adapter — feos PC-SAFT plus `kerotakis-thermo` — compiles clean (§ 5.2).
2. The dependency graph for `default-features = false, features = ["pcsaft"]`
   contains no `pyo3`, `numpy`, `rusqlite`, `rayon`, `cc`, `libc`, `getrandom`
   or `openssl`. There is nothing present that could fail.
3. Upstream CI already builds feos for wasm. PR #368 ("Pyodide wheel", merged
   2026-07-22, shipped in 0.10.1) added an emscripten job to
   `.github/workflows/release.yml` building `wasm32-unknown-emscripten` with
   `all_models` plus AD — every model including DFT and multiparameter. The
   only things it turns off are exactly the two named above; `py-feos`'s
   manifest says so in comments: *"All parallelism is gated behind this
   feature. Not active for emscripten wheel."* and the same for SQLite. A
   GitHub search across `feos-org/feos`, `itt-ustutt/num-dual` and
   `itt-ustutt/quantity` for `wasm`/`WebAssembly`/`wasm32` returns zero
   issues — nobody has hit a problem, and nobody has had to ask.

**The design constraint, and it is the practical finding of this section:**
`feos-core`'s parameter loaders — `Parameters::from_json`,
`from_multiple_json`, `from_json_segments`, `PureRecord::from_json`, the CSV
paths — call `std::fs::File` unconditionally. That *compiles* for
`wasm32-unknown-unknown` and then fails at runtime in a browser, which is the
worst possible failure shape: green build, dead bench. Any browser-facing use
must bypass them entirely and go
`include_str!` → `serde_json::from_str` → `Parameters::new_pure` /
`new_binary` / `new`. The spike's `wasm-probe-feos` crate is written that way
on purpose, which is why its `.wasm` links and runs; the corpus driver, which
is native-only, uses `from_json` freely.

This is not a blocker, but it does mean BRD-031's parameter pack has to be
compiled *into* the wasm artefact rather than fetched — which in turn means the
size number in § 5.3 is a floor, not the whole cost.

## 6. The adapter: does the seam hold?

`spikes/brd-030-feos/src/adapter.rs` implements
`kerotakis_thermo::fluid::FluidModel` over feos PC-SAFT for bubble points.
It compiles, it runs on the same corpus as everything else, and its numbers
match the direct feos calls exactly — the `adapter` column in
`discrepancies.tsv` is there to prove the wrapper adds nothing of its own.

**The seam holds, but the trait leaks its own model.** `FluidModel::bubble_point`
takes `&[Volatile]`, and `Volatile` is `{ antoine, x, gamma }`: two of its
three fields are the Raoult model's own state. A SAFT backend has neither and
needs neither, so the adapter must ignore `antoine` and `gamma` and key on
position and mole fraction alone. That works — but a trait whose argument type
carries the *other* model's parameters will silently accept a mismatched
pairing, and nothing in the signature can catch it.

**The sharper problem is the trait's default methods, and this is the one
finding here that would have caused a wrong answer on a bench.**
`FluidModel` supplies bodies for `dew_point`, `tp_flash` and
`saturation_pressure_kpa` that call `crate::vle::*` — the Raoult
implementation — directly. A backend that overrides only `bubble_point`, as
any first feos integration naturally would, therefore answers dew points and
flashes *with the ideal model*, silently, using whatever Antoine constants
happen to be sitting in the `Volatile`s it was handed. Nothing in the
signature, and nothing at runtime, says the answer came from a different
engine than the one the caller selected.

BREADTH.md's own rule is that "silent fall-through is a failure". These three
default bodies are a fall-through waiting for its first caller. The spike's
adapter deliberately does not override them, so the hazard is demonstrable
rather than papered over; the corpus only ever asks it for bubble points.

The fixes are small and mechanical, and BRD-031 should do them before
BRD-032 exists:

1. reduce `FluidModel`'s component argument to (species identity, mole
   fraction), moving the Antoine constants inside the Raoult implementation
   where they belong;
2. delete the three default method bodies, so a backend that cannot do a dew
   point has to say so rather than inherit someone else's answer.

`kerotakis-thermo` is 2 775 lines with three dependencies. This is a day's
work, not an architecture change — but it must happen before any routing.

### 6.1 Two latent hazards in `kerotakis-thermo` that this corpus walked past

Neither fires on the corpus, and both will fire on BRD-031's first day.

**A missing UNIFAC interaction pair silently becomes ideal.**
`unifac.rs`'s inner `psi` closure ends `table.interaction(m, n).map_or(1.0, …)`
— so a group pair with no published `a_mn` in `approved_table()` contributes
ψ = 1, which is the ideal answer, with no refusal and no label. BRD-031's own
integration rule says missing binary parameters must "produce a named refusal
or a labelled lower-fidelity route, never silent ideality". The corpus never
triggers it because the six main groups the approved table carries (1, 5, 6, 7,
9, 20) form a **complete 30-entry matrix** — checked, not assumed. The moment
an ester (main group 11), a chlorinated solvent (23) or an aromatic (3/4) is
added without every row completed, mixtures involving it start quietly
behaving like ideal solutions. `map_or(1.0, …)` should become an `Option`
that propagates.

**The binary solver extrapolates Antoine where the pure-fluid path refuses.**
`Antoine::pressure_kpa` honours `valid_c` and returns `None` outside it — the
right behaviour, and visible in the corpus as `kerotakis` refusals for
propan-2-ol at 25 °C. But `bubble_point_with`, `dew_point_with` and
`tp_flash_with` all call `pressure_kpa_unchecked`, by design, so that the
bisection can walk through out-of-range temperatures on its way to an
in-range answer. The doc comment says the *answer* is range-checked by the
caller — but nothing in this corpus's call path does that check, and neither
does `fluid.rs`. Acetone–ethanoic acid bubbles between 56 °C and 118 °C at
1 atm while acetone's fit stops at 77 °C, and the number comes back unlabelled.
The fix is small: have the `*_with` solvers report whether the converged
temperature lies inside every component's fitted range.

A third, much smaller friction: `vle::AZEOTROPE_TOLERANCE` is private, so the
adapter restates the constant rather than sharing it. Another symptom of the
same thing — the module was built around one model, and the constants that
belong to the *calculation* were never given a home outside it.

## 7. Decision: replace, backstop, or leave alone

BRD-030's acceptance requires this to be named explicitly, model by model.

### Replaced: **nothing**

| kerotakis-thermo model | why it stays |
| --- | --- |
| `vle.rs` Antoine + Raoult | feos has no vapour-pressure correlation model. Its equivalent is "solve VLE from the EOS", which is a different and much more expensive answer to the same question |
| `unifac.rs` activity coefficients | feos ships no activity-coefficient model of any kind (§ 2.2). There is nothing to replace it with |
| `lle.rs`, `phase_diagram.rs`, `excess.rs`, the ethanol–water still | all built on the two above |
| `eos.rs` Peng-Robinson (THERMO-007) | feos's cubic is a 234-line documented *teaching example*. Swapping 175 lines of our own for a dependency, to get the same equation, buys nothing |

The γ–φ route is not a lesser version of the φ–φ route; it is the one that
makes the ethanol–water azeotrope legible at a bench, and it is the one whose
parameters this project has already cleared. Adopting feos does not retire a
single line of it.

### Backstopped: the differential-oracle role

* **Peng-Robinson against Peng-Robinson.** Two implementations fed
  byte-identical Tc/Pc/ω, plus `thermo`'s as a third — a build-time
  differential test in exactly the CAP-19 shape, with no runtime weight. See
  § 3 for what it caught.
* **VLE outside the UNIFAC domain.** Where `unifac.rs` has no interaction
  parameter, a feos PC-SAFT answer is a second opinion worth having at build
  time before it is worth shipping.

### Added: the actual case for feos

Everything `kerotakis-thermo` has no model for at all:

* **Liquid density and enthalpy of vaporisation.** `kerotakis-thermo` has
  neither, for any fluid. Antoine is a pressure correlation and nothing more.
* **Critical points** for more than the three fluids `eos.rs` hard-codes.
* **The gases.** CO₂, N₂, O₂, NH₃ — nothing in the crate speaks about any of
  them, and BRD-000/014 demand all four.
* **Fluids outside the six curated Antoine sets** — the alkanes, the esters,
  cyclohexane, toluene, chloroform.
* **Binaries outside the ten UNIFAC groups in `approved_table()`** — esters
  (main group 11), chlorinated solvents (23), aromatics (3/4). Acetone–
  chloroform, the textbook *negative*-deviation azeotrope, is unreachable
  today for exactly this reason.
* **Pressure ranges outside the Antoine fits.** `Antoine::pressure_kpa`
  correctly refuses outside `valid_c`, which is honest and is also why
  reduced-pressure distillation and sealed-headspace behaviour hit a wall.

### Left alone

| model | why |
| --- | --- |
| ePC-SAFT | aqueous ionic behaviour already routes through PHREEQC by a different and better-cleared path |
| `multiparameter` | the CoolProp attribution defect in § 4.2, plus BRD-092's parked question. Not to be enabled |
| gc-PC-SAFT | the SMARTS group assignment is Python/RDKit-side upstream; the Rust API takes group counts only |
| PeTS, uv-theory, SAFT-VR Mie, SAFT-VRQ Mie, DFT | no demand anywhere in BRD-000/014. "More models" is explicitly out of scope for this task |
| `rusqlite`, `rayon` features | one bundles C SQLite, the other breaks the browser target. Never enable |

## 8. Recommendation

**GO, scoped and conditional.** Open BRD-031, but with its scope cut to what
§ 7 says feos actually adds, and with four conditions attached.

BREADTH.md's decision rule is "prefer the smallest engine that passes the
chemistry corpus identically on native, browser, macOS, iOS". Applied here:

* **Smallest.** With `default-features = false, features = ["pcsaft"]` feos is
  55 crates of pure Rust and no C. That is not small in absolute terms, but it
  is the smallest thing that answers the questions in § 7 at all — and the
  measured browser cost (§ 5.3) is the number that decides it, not the crate
  count.
* **Passes the corpus.** § 3.
* **Native and browser** are demonstrated here. **macOS and iOS are not.**
  The dependency graph contains no platform-specific code, which makes them
  very likely, but "likely" is not the rule's word. **Condition 1: BRD-031
  does not close until feos has actually been built for the macOS and iOS
  targets the release gate names.**

The other three conditions:

* **Condition 2 — parameters are cleared independently, not inherited.**
  § 4.2 finding 4: the `parameters/` tree carries no licence statement at all.
  Each table BRD-031 ingests needs its own `sources.toml` record reasoning from
  the primary publication. `poling2000.json` and `coolprop.json` are excluded
  by name.
* **Condition 3 — `FluidModel` is fixed before anything is routed.** § 6. The
  trait passes the Raoult model's own parameters through a model-agnostic
  seam, and — worse — its default `dew_point` / `tp_flash` /
  `saturation_pressure_kpa` bodies would make any second backend answer those
  three questions with the ideal model without saying so. That is a day's work
  in a 2 775-line crate, and it must happen in BRD-031 rather than being
  discovered on a bench in BRD-032.
* **Condition 4 — the version is pinned and the parameters are embedded.**
  `=0.10.x`, in the style the workspace already uses for `diffsol` and
  `serde_yaml_ng`, and parameters compiled in with `include_str!` rather than
  loaded through the `std::fs` paths that die silently in a browser (§ 5.4).

### The strongest argument against this recommendation

*On the mixtures this bench exists to teach, feos is measurably worse than what
is already there — so adopting it adds a second engine that must never be
allowed near the first engine's territory, and that boundary is the thing
projects fail to hold.*

This is § 3.4, and it is not a small effect: median 2.15 °C and worst case
11.0 °C in bubble temperature on the ten aqueous/alcoholic binaries where both
engines can speak, with PC-SAFT on the wrong side of literature every time.
Ethanoic acid–water at x = 0.9 comes back at 100.7 °C when it must approach
117.9 °C. And the mechanism is worse than the magnitude: the degradation comes
from a **missing binary interaction parameter**, `rehner2023_binary.json` simply
has no acetic acid–water record, and PC-SAFT quietly carried on with k_ij = 0
rather than refusing. That is the same silent-ideality failure mode § 6.1 found
in `unifac.rs`, arriving from a new direction and with 7 848 binary records to
audit instead of thirty.

So the `go` is buying an engine that is excellent at the properties we lack a
model for and worse than what we have at the ones we do — while adding a
second, larger surface on which "missing parameter" can mean "wrong answer
instead of no answer". Every condition in this section is really about holding
that line, and conditions are the weakest form of engineering control there is.

A second, independent objection: **feos's Rust API is not stable, and
Kerotakis's value is that its numbers do not move.**

Four breaking minor releases in about two years, the most recent six weeks
before this spike: 0.7.0 removed the `HelmholtzEnergy` traits; 0.8.0 deleted
the `si` module; 0.9.0 changed the **parameter file format** incompatibly
(which is why the repository still carries a `parameters_old/` directory) and
moved the Python bindings out; 0.10.0 removed `StateBuilder` and made every
extensive property return a `Result`. Anything written against 0.8 or 0.9 does
not compile today. A pin protects the build but not the maintenance: each
upgrade is a rewrite of the adapter, and each one lands on an offline teaching
artefact whose whole promise is that the ethanol–water still gives the same
answer next year.

Against that, the cheaper alternative this spike cannot rule out: **extend the
tables by hand.** `vle.rs` has six Antoine sets and `unifac.rs` has ten groups.
Adding alkane, ester, aromatic and chloro groups to `approved_table()` and a
dozen more Antoine sets is literature transcription of exactly the kind BRD-031
must do anyway — and it would close most of the *binary* half of the coverage
gap with no dependency, no API churn, and no new licence surface at all.

Both objections are correct as far as they go, and together they should shrink
BRD-031's scope hard. What neither can reach is the 351-row coverage gap.
Hand-extending Antoine and UNIFAC produces no liquid density, no enthalpy of
vaporisation, no critical point, and nothing at all for CO₂, nitrogen, oxygen
or ammonia, because those are not quantities a vapour-pressure correlation and
an activity model can express — and every one of them is asked for by
BRD-000/014. That residue is the case for feos.

So the verdict is `go` **because** feos is worse where `kerotakis-thermo` is
good, not despite it. If PC-SAFT had matched UNIFAC on ethanol–water this
report would have had to argue about which engine should own the bench, and
that argument is how a project ends up with two half-maintained
thermodynamics stacks. The measurement removed the ambiguity: the two engines
are good at disjoint things, the boundary between them is a property list
rather than a judgement call, and § 7 can state it as a table. A `no-go` would
leave BRD-000/014's density, enthalpy and permanent-gas questions with no
route at all and no fallback to name — which the decision-gate rule does not
permit.

If the conditions in this section cannot be met — particularly the macOS/iOS
builds and an independently cleared parameter set — the correct outcome is not
an unconditional `go`. It is to reopen this record.

## 9. Scope boundaries observed

* No change to the workspace `Cargo.toml`, to any crate under `crates/`, or to
  `Cargo.lock`. The spike is its own workspace and depends on
  `kerotakis-thermo` read-only by path.
* No third-party parameter file is in the diff.
* No routing, CLI, wasm, MCP or GUI surface was touched. The adapter is not
  reachable from any of them.
* `provenance/sources.toml` gains no record, for the reason in § 4.3.
* Targets tested here are `x86_64-unknown-linux-gnu` and
  `wasm32-unknown-unknown` only. The decision rule in BREADTH.md asks for
  native, browser, macOS and iOS; **macOS and iOS were not tested** and this
  report does not claim them. The dependency graph is pure Rust with no
  platform-specific code, which makes them likely but not demonstrated, and
  BRD-031's acceptance should require an actual build on both before any
  routing lands.

# Chemistry audit — 2026-09-06

This is the **original, pre-fix audit**, preserved as evidence. Subsequent
implementation and verification are tracked in [IMPLEMENTATION.md](IMPLEMENTATION.md).
The statements below about unchanged production code describe that initial run.

Fifty independently conceived experiments were executed through the native
Kerotakis CLI: 42 initial experiments, eight diagnostic follow-ups, and three
repeat runs (53 process executions). **49 of the 50 distinct scripts completed;
one was rejected for missing KSCN. No `solver_failed` events, timeouts, malformed
JSON lines, or nonfinite JSON numbers were observed. This is not a 98% chemistry
pass rate.** Thirty-eight explicit physical/product checks give **28 passes and
10 unmet expectations**, with additional qualitative gaps below. Several
expectations exercise the same underlying defect.

The strongest finding is that aqueous inventories and material inventories can
produce wrong balance readings while pH, pressure, and reaction events appear
reasonable. Fix those ledgers before expanding the chemical catalogue. The
next priorities are phase identity, missing water characterization, and ligand
coverage/routing. None of those initial fixes needs another solver dependency.

## Scope and reproduction

- Worktree: `/mnt/volume1/kero-experiment-audit`.
- Branch: `audit/chemistry-experiments-20260906`.
- Audited application revision: `f511d533` (full SHA and binary SHA-256 in each
  `summary.json`). Existing worktrees, branches, and agents were left alone.
- Single agent; experiments run sequentially, each in a fresh CLI process and
  its own working directory. Build artifacts are local to this worktree, using
  the machine's two-job limit. No running app or external deployment was used.
- Inputs are original LLM-assisted audit designs, using the existing grammar
  and registry. They are simulation inputs, not physical-laboratory protocols.
- Only audit scripts, recorded outputs, a dependency snapshot, and this report
  were added. Production code and existing tests were not changed.

The first clean build attempt, `cargo build --locked -p kerotakis-cli`, failed:
the root `Cargo.lock` is ignored and untracked. `cargo build --offline -p
kerotakis-cli -j 2` then built successfully in 3m51s. Its resolved lockfile is
preserved as [dependency-lock.snapshot](dependency-lock.snapshot). This records
the actual build, rather than implying that the untracked lockfile in another
worktree was used. Submodules were initialized at their recorded revisions.

From a fresh checkout, initialize the submodules, copy this snapshot to the
checkout's root `Cargo.lock`, and build with `--locked`. Do not overwrite a
pre-existing lockfile in somebody else's working tree.

```sh
git submodule update --init --recursive
cp tools/chemistry-audit/dependency-lock.snapshot Cargo.lock
cargo build --locked --offline -p kerotakis-cli -j 2
python3 tools/chemistry-audit/run.py --out /tmp/kero-audit-replay
python3 tools/chemistry-audit/analyse.py /tmp/kero-audit-replay --out /tmp/kero-audit-checks.json
```

Use a fresh output directory. The runner requires only Python's standard
library; it retains every `.lab`, stdout, stderr, exit status, runtime, binary
hash, lockfile hash, and submodule revision. The analysis exits **1** when
expectations remain unmet; this is intentional, not an infrastructure failure.
The original evidence is in [results](results), [followups](followups), and
[followups-50](followups-50). [checks.json](checks.json) contains check-level
evidence. The repeat outputs for cases 40–42 are byte-identical to the initial
JSON outputs; this verifies only those three replays, not all state-leak paths.

This audit tested native CLI behavior at one pinned revision. It did not test
browser rendering, mobile/wasm parity, all reaction families, experimental
kinetic accuracy, or all possible input failures. “Supported” below means the
particular question's main observation was delivered, not that every quantity
in that run was independently validated. Assertions use conservation laws,
qualitative chemistry, broad dilute-solution bounds, and paired experiments;
they are not fitted goldens of the engine's own answers.

## Experiment inventory

Every row has an executable script and complete output under the matching
numbered directory. Temperatures below are Celsius unless explicitly marked K.

| ID | Experiment | What Kerotakis delivered | Assessment |
|---|---|---|---|
| 01 | Pure-water pH | No characterized solution; no pH | Missing baseline |
| 02 | Tenfold HCl dilution | pH 3.01346 → 4.00366 | Supported |
| 03 | Tenfold NaOH dilution | pH 10.98527 → 9.99575 | Supported |
| 04 | Acid-first neutralization | pH 6.96939; 26.20469 °C | Supported; warm neutral water need not be pH 7 |
| 05 | Base-first neutralization | Same pH; temperature agrees to floating-point precision | Strong order-invariance result |
| 06 | Acetate buffer + acid | pH 4.69372 → 4.60677 | Supported |
| 07 | Water + same acid dose | Final pH 2.71784; missing initial water pH | Comparison partly delivered |
| 08 | Phosphate base additions | pH 2.61824, 4.70523, 6.98208, 9.31274 | Distinct titration regions; no exact pKa claim at these concentrations |
| 09 | Nanomolar HCl | pH 6.99759 | Autoionization handled once route engages |
| 10 | AgNO3 into NaCl | 0.0019969623 mol AgCl from 0.002 mol each | Supported |
| 11 | Reversed silver precipitation | AgCl yield differs by 1.27e-9 mol | Yield agrees; 0.00931 K thermal residual worth tracking, not established as a major defect |
| 12 | Barium sulfate | 0.0019963420 mol BaSO4 | Supported |
| 13 | Copper hydroxide | 0.001996949 mol Cu(OH)2 plus trace brochantite | Main precipitation supported; withheld-phase tag misleading |
| 14 | Excess salt | 0.19074954 mol NaCl remains; Pitzer route | Supported saturation/model selection |
| 15 | Dilute saturated salt | All salt redissolves | Supported |
| 16 | Calcium chloride hot pack | 25 → 28.97378 °C | Correct heat sign |
| 17 | Potassium chloride cold pack | 25 → 24.17453 °C | Correct heat sign |
| 18 | Copper sulfate heat/cool | Hydrate dissolves/reforms; return temperature within 0.00064 K | Good thermal cycle; mass drifts by 0.00212 g |
| 19 | Open bicarbonate/acid | Total 0.01 mol CO2 evolved, partly before acid arrives | Gas accounting observable; balance inherits aqueous ledger errors |
| 20 | Seal after bicarbonate | Final pressure 145.123 kPa | Gas behavior supported; added acid mass off by about 5.28 mg |
| 21 | Pressure-controlled fizz | 120 kPa retained; headspace expands to 354.194 mL | Supported boundary response |
| 22 | Limewater, excess CO2 | Calcite 0.00493879 → 0.000347777 mol | Substantial redissolution, not complete clearing at this dose |
| 23 | Zn into CuSO4 | 0.00298960 mol Cu plated; residual Zn remains | Supported displacement; useful kinetic explanation |
| 24 | Cu into ZnSO4 | No Zn plated; Cu retained | Correct reverse control |
| 25 | Mg into HCl | 0.002 mol H2 from 0.002 mol Mg | Supported stoichiometry; hazard event emitted |
| 26 | Cu into HCl | Explicit electrochemical refusal | Correct control |
| 27 | Daniell cell | 1.10420572 V | Supported voltage; slow Zn/water boundary disclosed |
| 28 | Conductive-water electrolysis | 60 C; H2 0.0003109281 mol, O2 0.0001554640 mol | Faraday amount and 2:1 ratio supported |
| 29 | Pure-water electrolysis | Refused: says no metal/electrode | Refusal acceptable; explanation names the wrong limitation |
| 30 | Permanganate dilution | Purple → pink; MnO4 inventory retained | Supported simple-ion optics |
| 31 | Filter AgCl | Solid remains in source; dissolved traces in filtrate; Ag conserved | Supported separation |
| 32 | Evaporate brine | 0.07756345 mol NaCl crystallizes | Supported equilibrium endpoint; no physical drying-rate validation |
| 33 | Distil aqueous ethanol | Receiver ethanol mole fraction 0.38506 vs feed 0.09091; ethanol conserved | Supported separation; reported boil/energy is a process calculation, vessels remain at 25 °C |
| 34 | Passive acid/alcohol, 1 h | No organic transformation and no missing-kinetics explanation | Silent boundary |
| 35 | Explicit esterification | Extent 0.01980477 mol, roughly 99% of acid charge | Curated forced conversion; boundary clearly says catalyst/heat assumed and no yield prediction |
| 36 | Sugar, no yeast | No fermentation over 600 s | Correct control |
| 37 | Sugar + yeast | 0.002518854 mol each ethanol and CO2; all CO2 absorbed | Useful bounded result; mass increases by 0.0426492 g without addition |
| 38 | Mg ignition | 0.24305 → 0.40304 g; 3213.7909 K | Oxygen mass gain correct; retained MgO incorrectly solid despite liquid-phase solver result |
| 39 | NaCl ignition control | Yellow flame test, unchanged salt | Supported |
| 40 | Fe(III)/thiocyanate | KSCN unknown; aborts at that addition | Registry/reaction/optics gap, not solver convergence failure |
| 41 | CuSO4 + excess NH3 | Mostly Cu(OH)2, pale blue; no tetraammine response | Ligand coverage gap; unrelated sulfide saturation warnings |
| 42 | Automated neutralization | Crosses target after 21 mL (20 mL stoichiometric); final pH 10.60465 | Correct increment crossing, poor “endpoint pH” experience |
| 43 | Weigh HCl addition | 0.1 mol adds 3.5453 g vs 3.6461 g | Missing 0.1008 g proton mass |
| 44 | Weigh NaOH addition | 0.1 mol adds 2.2990 g vs 3.9997 g | Missing 1.7007 g hydroxide mass |
| 45 | Weigh yeast addition | 0.5 g dose adds 0.0000005 g | Unresolved solid mass omitted |
| 46 | Cu/NH3 with trace acetate | 1e-9 mol acetate switches to minteq; almost all Cu precipitates; 1.677e-5 mol N(3) omitted with warning | Routing discontinuity and admitted lost inventory |
| 47 | Wait after limewater dose | Calcite returns to 0.00473608 mol; pH 8.21879 | Instant ambient re-equilibration; gas-transfer time not resolved |
| 48 | Seal before both fizz reagents | Added mass 1.55838345 g vs 1.560694 g | Closed-system mass discrepancy persists |
| 49 | Wait before pure-water pH | Still no pH | Confirms routing gap |
| 50 | Sugar-water pH | Sugar dissolves, no pH | Nonionic-solution characterization gap |

## Findings and proposed fixes

### A. First priority: one physically conserved inventory

**A1 — Acid/base mass is missing, not merely rounded.** Cases 43 and 44 retain
water plus Cl− or Na+ in `contents`; the measured proton/hydroxide quantities
live elsewhere. [`Vessel::mass`](../../crates/kerotakis-core/src/vessel.rs)
sums portion masses and compartments but does not include those fields.
[`aqueous.rs`](../../crates/kerotakis-phreeqc/src/aqueous.rs) separately writes
`free_proton` and `free_hydroxide`. pH can therefore look correct while a scale
loses essentially all of the added hydroxide's mass.

Do not simply add `solute_charge × molar_mass` to the scale: alkalinity is not
free hydroxide. Define a conserved elemental/H/O inventory with an explicit
projection into solvent, actual protonation states, and residual compartments.
Make the scale and conservation checks consume that inventory. A local free
H+/OH− term may repair the simple probes, but needs a water-reaction accounting
proof before it is a general fix. Test acid, base, neutralization, dilution,
carbonate, phosphate, and both addition orders across the **full CLI stack**.
Start with tolerances tied to declared molar-mass precision, not percentage
of the large water mass that hides a missing small reagent.

**A2 — Carbon readback changes mass across phases.** Fermentation case 37 gains
42.65 mg despite all generated CO2 staying in the vessel. The repository already
documents the underlying carbon-basis issue in
[`sealed_mass.rs`](../../crates/kerotakis-phreeqc/tests/sealed_mass.rs): dissolved
CO2 is booked as HCO3− without the corresponding solvent debit. I executed its
ignored regression explicitly: a sealed vessel gains **0.01085 g** on cooling
from 298.23 K to 283.93 K. The companion volcano test passes its 5 mg tolerance.
The exact command returned exit 101, one pass and one failure (retained
[test output](sealed-mass-test.txt)):

```sh
cargo test --offline -p kerotakis-phreeqc --test sealed_mass -- --include-ignored
```

Implement the C(4) distribution as actual CO2(aq)/HCO3−/CO3²−, including complex
stoichiometry and H/O exchange with water, while keeping element totals
conserved. Extend the existing protonation mechanism rather than special-case
fermentation or compensate a balance display. Then unignore the cooling test
and add open/sealed/regulated and biological-produced-CO2 checks. This defect
was already assigned in source comments to the aqueous work; coordinate with
that owner before modifying the same code.

**A3 — Unresolved solids still have mass.** Case 45's 0.4999995 g unresolved yeast
portion exists in state, but
[`unresolved_material_mass_g`](../../crates/kerotakis-core/src/material.rs)
filters for `HomogeneousLiquid` before counting it. Count mass-basis portions
regardless of physical form. For volume-basis portions require a reviewed bulk
density; keep genuinely unknown conversions explicit. Audit transfer, filter,
spill, and consumption ownership to prevent double counting. Acceptance: dose
mass appears on the scale and is conserved across those operations for yeast,
powders, opaque liquids, and prepared objects.

**A4 — A warning must not authorize losing an element.** Case 46 explicitly
reports that 1.677e-5 mol N(3) was omitted because no inventory species names it.
Retain an unresolved elemental/protonation compartment, or reject the proposed
solve transaction while keeping its diagnostics. Do not commit a smaller
inventory. This should share A1's accounting design, not add a per-nitrogen
exception. This warning is honest about the defect but does not make the
result physically acceptable.

### B. Preserve the state the solver actually calculated

**B1 — Hot-phase identity is lost at readback.** Case 38's provenance explicitly
names NASA `MgO(L)`, yet its final portion is `phase: solid` at 3213.79 K. The
registry itself declares a 3098.15 K melting temperature. In
[`thermal.rs`](../../crates/kerotakis-cea/src/thermal.rs), retained amounts are
assigned `reg.standard_phase`, losing the selected NASA phase. Preserve the
solver phase as part of each result key, permit simultaneous phases when
appropriate, and emit matching liquid/solid observations. Validate heating and
cooling across a transition, mass, latent heat, and the scene together. No
additional thermodynamic dataset is needed to fix this demonstrated mismatch.

**B2 — Pure-water and sugar-water pH fall through routing.** Cases 01/49/50
cannot read pH, while case 09 proves autoionization works once the aqueous
solver runs. `partition` rejects `solutes == 0`, and the `applies` gate does
not cover these neutral baselines. Allow a valid water-only equilibrium problem
within the aqueous domain; retain explicit handling for unsupported acids and
mixed solvents. Calculate temperature-dependent neutrality through the solver,
rather than hardcode 7. Verify pure water, diluted neutral solute, temperature
changes, and sealed/open boundaries.

### C. Better coverage requires species and ligands, not just elements

**C1 — Cu/ammonia is not covered by merely recognizing Cu and N.** The default
route selects wateq4f using element availability; its species set lacks the
needed copper ammines. Adding just 1e-9 mol acetate chooses minteq instead and
changes the result materially (case 46). Inspection finds a monoammine row in
minteq, not a complete reviewed tetraammine ladder. Neither this route switch
nor a new colour tint is a complete repair.

The intended qualitative demonstration is precipitation followed by
complexation/redissolution in excess ammonia, with a deep-blue complex, as
described by [Purdue's teaching demonstration](https://chemed.chem.purdue.edu/demos/demosheets/18.8.html).
That source establishes the intended phenomenon, **not an exact equilibrium
yield for our dose**. Route on required species/reaction coverage as well as
ionic-strength validity, and report missing ligand equilibria explicitly.
Add a reviewed stepwise complexation dataset only after its redistribution
terms are cleared. Compute colour from actual speciated complexes; current
[`appearance.rs`](../../crates/kerotakis-core/src/appearance.rs) primarily uses
the aggregate `contents` portions and skips missing spectra. Validate low/high
NH3, acid reversal, and the negligible-ligand route perturbation.

**C2 — KSCN is not reachable.** Case 40 aborts before thiocyanate can be tested.
The Fe(III)/SCN− colour equilibrium is a real teaching target documented by
[Harvard's demonstration](https://sciencedemonstrations.fas.harvard.edu/presentations/equilibrium-iron-iii-thiocyanate).
Add identity, reaction/speciation coverage, and reviewed optical data as one
slice, with mass/charge balance. Adding a bottle alone would turn the explicit
missing-input failure into a potentially misleading successful run. Neither
teaching webpage is treated as a reusable numeric dataset or imported here.

### D. Model boundaries and instrument semantics

**D1 — Successful no-op versus unavailable kinetics.** Case 34 silently leaves
acid/alcohol unchanged after an hour. Case 35 honestly says its reaction verb
forces an extent and assumes conditions; its approximately 99% conversion
must not be presented as predicted equilibrium yield. Emit a scoped missing
kinetics explanation for recognized candidate chemistry during `wait`. Keep
ordinary inert controls quiet; this is not a request to warn about every
nonreaction. An equilibrium esterification model is a later, separately
parameterized improvement.

**D2 — Wrong reason for pure-water electrolysis refusal.** Case 29 says there
is no metal to be an electrode. The adjacent sulfate case uses inert-electrode
water splitting. Explain the actual apparatus/conductivity boundary and its
assumptions, using the same electrode model in both cases. A refusal is not
evidence that water lacks electrode reactions.

**D3 — Equilibrium gas transfer is being read as a clock.** Case 47 restores
most chalk after a one-second wait: the finite CO2 dose has ended and the open
boundary re-equilibrates with ambient gas. This direction is chemically
reasonable, but these runs do not validate a one-second mass-transfer rate.
State that boundary reset as instantaneous equilibrium. Add kinetics only
with reviewed transfer coefficients, geometry, and a need for real time.
Do not force the earlier dose's clear state to persist without gas support.

**D4 — “Reached pH 7” actually means crossed it.** Case 42 is internally
consistent: warmed neutral solution is at pH 6.99347 at 20 mL; the next 1 mL
increment jumps to pH 10.60465. Label the result as a crossing bracket and
overshoot. If the user asks to approach a target, replay/bracket/refine the
increment while conserving titrant and carrier water. Do not falsify the
measured final pH or declare this a stoichiometry failure.

**D5 — Typed diagnostics conflate different boundaries.** Deliberately withheld
Tenorite has cause `phase-not-in-registry` in cases 13/41/46 even though the
prose correctly explains a kinetic choice. Add a distinct withheld-metastable
cause or use an appropriate existing model-boundary tag. Case 41's sulfide/N2
saturation warnings also need review against constrained redox chemistry:
thermodynamic saturation alone does not establish what appears on a classroom
timescale. I did not establish a quantitative rate for those phases.

**D6 — `step` is not a unique output-row identifier.** An `inspect` record uses
`bench.log.len()` and the next mutation uses `bench.log.len()-1` after adding a
log entry. Case 18 therefore emits successive records both called step 2,
then both called step 3. This follows the current implementation; do not assume
uniqueness in a consumer. Document it as a log-position field, and add a
separate monotonic output/request sequence if clients need unique identities.

### E. Reproducibility and license boundaries

Track a root lockfile for the executable workspace, retain `--locked` in
reproducible builds, and stamp binary revision/features/dependency identity.
The audit snapshot is evidence, not a production lockfile-policy change.

`cargo deny --offline check licenses` passed (only unused-allowance warnings).
`bash tools/provenance-lint.sh` passed, including its quarantine promotion
checks. These are the repository's current checks, **not a claim that every
possible future dependency or numeric record is cleared**. The global
`deny.toml` AGPL allowance is broader than a workspace-only exception; scope
that exception to project-owned crates so a future external AGPL dependency
cannot pass merely by naming the same license. Keep external GPL/LGPL/AGPL and
NC inputs out of the proposed fixes, including any development oracle selected
for this work.

Kerotakis itself declares AGPL-3.0-or-later, with its own contribution terms;
this audit has not changed that. Your permissive-only constraint is applied to
**new external code and data used for fixes**. If it was intended to require a
permissively licensed Kerotakis as a whole, that is a separate copyright-holder
relicensing decision and has not been accomplished here.

All priority-A/B work can use existing code and already-vendored material.
PHREEQC supports ion-association, Pitzer, and SIT models, so the proposed
coverage-aware routing builds on the existing aqueous architecture rather
than replacing it ([USGS documentation](https://pubs.usgs.gov/publication/tm6A43)).
For future differential kinetics checks, Cantera's code is BSD-3-Clause
([Cantera community/license statement](https://www.cantera.org/community.html)),
but that does not clear arbitrary mechanisms or their parameter sources.
The existing BRD-040/042 decision already parks runtime Cantera integration;
this audit supplies no reason to reverse that decision.

For ligand constants, spectra, or kinetics that are missing, accept only a
positively reviewed permissive/public-domain grant with per-field provenance,
conditions, units, uncertainty, and retained attribution. An accessible PDF,
MIT wrapper, or chemistry database download is not a grant for its contents.
In particular, minteq comments cite NIST46 for some constants; do not infer
permission to scrape more NIST data or promote unrelated records from that.
No GPL-family/NC package or new external numerical dataset was added here.

## Recommended implementation order and acceptance

1. **Shared mass/element ledger:** A1–A4 together, with separate small patches
   for unresolved-material mass and carbonate/protonation readback. Coordinate
   with the existing aqueous owner. Require the current ignored cooling test
   and new acid/base/material full-stack assertions to pass.
2. **Preserve authoritative state:** B1 phase mapping, B2 neutral-water route.
   Verify scene/measurements against the same solver state on native and wasm.
3. **Tell the truth about existing boundaries:** D1–D6. These mostly need
   semantic/event/render changes and targeted contract checks, not new data.
4. **Add complete ligand slices:** C1 then C2, after per-record license review.
   Require equilibrium reversibility, conserved inventories, confidence and
   provenance, and spectra driven by the actual complex populations.
5. **Broaden validation:** reuse these paired cases in the full-stack suite,
   then test wasm/mobile transport parity and domain-specific differential
   oracles where both code and data satisfy the requested license range.

The targeted existing `sealed_mass` tests were run with ignored tests enabled;
the full workspace test suite was not run for this audit-only change. Existing
goldens and a clean solver exit cannot replace the unmet physical checks
recorded here. No production fix, merge, PR, or deployment was performed.

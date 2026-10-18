# Saved64 review and repair lanes — 2026-10-18

Finish accepted-source integration and the full frontend/native deployment in
[the handover](next-agent-handoff.md) before starting new model work. These lanes
turn the [saved64 first screen](https://github.com/CrispStrobe/kerotakis/blob/7c14f3966d40f7c771bc90a31f497a1a72971320/audits/target468-20261016/saved64-first-screen/README.md)
into concrete follow-ups. They preserve original forecasts and first outcomes;
none authorizes treating unsupported chemistry, exit0 or a diagnostic warning
as a scientific pass. The catalog remains200 / fixed468 selected196.

## A. Finalize bounded selections from existing evidence

**Input:** all64 records and original criteria in `first-screen.json`, original
producer archive and pinned source, material projection and supplemental metrics.
There are44 first-screen candidates and20 holds; candidate is not a final pass.

**Work:** review each original criterion against the actual timepoint and owned
inventory. Check source contracts for bounded stirring, enzyme activity, ideal
organic templates, chromatography and thermal properties. For thermal selections,
compute sensible/latent energy from pinned properties; for separation, verify
per-vessel ownership and declared energy boundaries. Reproduce the independent
material reader. Keep rounding, missing phase domains and property provenance
explicit. Check the source's additive liquid volume, including peroxide, for
KN009; the saved concentration ratio18.9029 exceeds consumption ratio8.47084.

**Done:** append a case-level final assessment and independent calculation review;
add only supported IDs once with exact source/script association. Preserve all
200 existing catalog entries and original first failures. Record before/after
hashes, counts, exclusions and an immutable selection receipt. No unchanged668
replay. The20 holds below remain excluded unless separate evidence closes them.

## B. Verify current carbon and charge ownership, then repair if reproduced

**Cases:** GS010, with GS003/006/008/009 as related finite-gas controls.
**Source:** `crates/kerotakis-phreeqc/src/aqueous.rs`, its finite-gas input,
readback, phase pinning and charge reconstruction, and `finite_gas_temperature.rs`.

**Work:** the old salt/CO2 output adds2.7674e-7mol carbon and1.1069e-6eq charge
at CO2 addition. First inspect pinned old/current changes. Freeze a minimal
changed-source diagnostic with solvent-only and matched salt controls. Distinguish
actual PHREEQC element totals, gas portions, oxidation-state pinning, reconstructed
acid/base equivalents and rounded public species. The saved brine actually routes through Pitzer; this is not evidence that a
salinity-capable model was never used. Do not tune the solver or subtract the
observed residual merely to pass this point.

**Done:** identify the responsible writer, retain its first failing current-source
result if reproduced, and fix ownership across direct/comparison/repeated solve,
opening and retained-gas routes. Require represented C/H/O/Na/Cl/charge and
boundary ledgers; preserve honest dataset/activity limits. A salinity comparison
is not accepted until conservation and actual activity ownership are established.

## C. Preserve latent phase fractions through downstream metal writers

**Case:** TH016. A1kJ pulse emits a Zn melt event for0.07395386mol, but the old
result labels all0.76475987mol as liquid at692.68K. No retained solid remains.
**Source:** `crates/kerotakis-core/src/{phase_route,displacement,bench,vessel}.rs`,
`crates/kerotakis-phreeqc/src/aqueous.rs`,
`crates/kerotakis-cea/src/thermal.rs` and the standard solver stack.
The CEA route is a specific source-review lead: `Charge.mapped` stores species
and amount without the owned phase, incoming enthalpy resolves a registry-mapped
CEA record, and final condensed contents are assigned even when no reaction
event marks a change. Establish whether that path overwrites the paid partial
melt; a source-reading lead is not yet a current-source reproduction.

**Work:** locate every post-transition metal phase writer. Reproduce against
current source before changing it. Trace the event, deposited phases and energy
pool through each solver. Compare fusion energy and integrated sensible heat
against the requested pulse; do not infer energy closure from atomic closure.

**Done:** partial melt retains both finite phases and event amounts agree with
owned phase changes. Cover below/at/above transition, repeated unchanged solve,
reverse cooling, transfer and a second metal plus a molecular-solid control.
No new oxide may appear without an oxygen source. Preserve original TH016 and
freeze repaired-source evidence separately.

## D. Share authoritative aqueous base with named ester hydrolysis

**Cases:** OR006–009. Explicit `react ... saponification` leaves ester unchanged
although positive base equivalents are owned; downstream protonation and carbon
cycle prerequisites therefore also fail.
**Source:** `crates/kerotakis-org/src/lib.rs`, core family routing and aqueous
transfer/readback; `crates/kerotakis-phreeqc/tests/family_aqueous_tail.rs`.

**Work:** distinguish the explicit requested stoichiometric template from the
warm kinetic family and inspect their hydroxide-capacity owners. Use actual
full-precision analytical base, not rounded displayed OH activity or residual
NaOH molecules. Deduct residual acetic-acid neutralization before ester cleavage.
Do not add naming aliases for an inventory-ownership failure. Keep the
counterion ledger explicit: consuming hydroxide equivalents while depositing
neutral sodium acetate can double count sodium if the spectator cation is also
retained. Test that path and either support other alkali counterions with a
balanced net ionic reaction or state their precise admission boundary.

**Done:** base-limited and ester-limited requests form matched acetate/ethanol,
retain the correct excess, and close C/H/O/Na/charge. Include prior aqueous solve,
changed pouring order, repeated request, transfer/save/cache and atomic refusals.
Keep heat, rate and warm-family limitations separate. Reassess OR008/009 only
after their hydrolysis prerequisites actually occur.

## E. Repair analytical coverage reporting and staged separation

**Cases:** SP012 and SP003, separately; do not combine their runtime repairs.

For **SP012**, inspect chromatography method admission and peak generation. The
saved mixture contains ethanol, ethyl acetate and glucose, but only ethanol is
reported and `outside_method` is empty. Identify supported parameters; explicitly
report missing-method species rather than silently implying absence. Require
actual differentiated peaks only where parameters exist, unchanged sample
inventory, response-factor qualification and CLI/browser parity. Unsupported
parameters are a coverage limit, not a license to invent retention constants.

For **SP003**, inspect distillation's accumulated-condensate positivity refusal.
One stage delivers product; four stages refuse atomically. Freeze matched feed,
cut, stage count and energy controls. Determine the admitted algorithmic domain
before implementing a numerical repair; require every positive component retained,
all source/receiver balances, finite energy and no invented ethanol. An honest
unsupported domain remains held until the requested comparison is supported.

## F. Close thermal and phase-model prerequisites

**Cases:** TH007–010 and TH015, using separate narrow repairs.

- **TH007:** use the existing [KNO3 lane](kno3-finite-solubility-lane.md). Cooling
  leaves all K/nitrate dissolved and no crystals. Phase admission, temperature
  data and concentrated nitrate activities are distinct prerequisites.
- **TH008:** five hydration waters and Cu/S balance are positive; determine
  whether hydrate dissolution heat is sourced or absent before comparative heat
  selection. Test declared reference states and avoid counting hydration twice.
- **TH009:** CaO hydration is represented but its exotherm is not established;
  old output cools. Review independent reaction enthalpy/reference states and
  wire an owned heat contribution only with reviewed provenance and units.
- **TH010:** bicarbonate decomposition creates carbonate/CO2, but the aqueous
  tail fails on the tiny generated water pool. Review valid aqueous-domain
  admission and atomic failure behavior. Do not convert exit0 into convergence.
- **TH015:** compute the no-reaction heat-capacity baseline from both prepared
  solutions. Determine where mixing neutralization heat is omitted or recorded;
  do not charge prior acid/base dissolution again. Require proton/base, matter
  and energy ledgers across direct addition, finite MIX and normal fallback.

**Done:** each scoped repair has actual source-bound positive, reverse and
rejected-domain controls with independent property provenance. No aggregate
thermal pass is earned by conservation of atoms alone.

## G. Protocol and coupled-chemistry holds

- **GS005:** connect to authoritative free/bound ligand ownership in handover
  lane5. Aqueous protonation is strong but gas NH3 is essentially unchanged;
  inspect Henry transfer and iteration before claiming a validated acid trap.
- **GS007:** connect to ordered-agenda boundary semantics. CO2-free infinite
  purge is not finite degassing. Review swept exports and carbonate equilibrium;
  preserve the first scale forecast and freeze distinct finite/reservoir tests.
- **KN010:** saved samples miss an iodine-clock onset comparison. Review iodine/
  iodide/thiosulfate coupling, then freeze an informative sampling schedule.
- **KN011:** the weak-acid rate branch is explicitly unsupported. A zero rate
  from a withheld pathway is not proof of relative chemical kinetics.
- **KN013:** the pre-replenishment acid reserve is not exhausted at the saved
  timepoint. A new longer-time protocol needs separate frozen forecasts and
  coupled SO2/bisulfite accounting; do not rewrite the original outcome.
- **SP011:** iodine reduction never occurred before extraction; both organic
  receivers contain equal I2. Fix or explicitly scope the redox prerequisite
  before interpreting partitioning, with complete iodine/sulfur/charge ledgers.
- **KN007:** old filtering loses O2; the later integrated transfer repair retains
  donor gas. Preserve the old failure and the separate existing source-bound
  repair acceptance. Do not duplicate unchanged tests or select the old output.

Keep host operation details, tokens, private inventories and local resource
thresholds in the ignored operating handoff. Use the normal protected PR workflow;
never merge the broad audit branch to import this evidence.

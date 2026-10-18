# Saved64 review and repair lanes — 2026-10-18

Finish accepted-source integration and the full frontend/native deployment in
[the handover](next-agent-handoff.md) before starting new model work. These lanes
turn the [saved64 first screen](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/saved64-first-screen/README.md)
into concrete follow-ups. They preserve original forecasts and first outcomes;
none authorizes treating unsupported chemistry, exit0 or a diagnostic warning
as a scientific pass. The latest catalog is204 / fixed468 selected200 after the separate deployed-source KN-007 replay and final review of three original saved gas cases; the original screen and first failures remain unchanged.

## A. Finalize bounded selections from existing evidence

**Input:** all64 records and original criteria in `first-screen.json`, original
producer archive and pinned source, material projection and supplemental metrics.
There are44 first-screen candidates and20 holds; candidate is not a final pass.

The [complete reproducible preparation index](https://github.com/CrispStrobe/kerotakis/blob/f865e4c23566043f3224598d466a4459757cfe62/audits/target468-20261016/saved64-review-preparation/README.md) covers all64 cases:10 gas,14 kinetics,12 organic,12 separation and16 thermal. It includes actual owned phase/species timelines, original criteria and explicit scope limits. No simulation or API replay and no selection was added by these readers.

**Work:** review each original criterion against the actual timepoint and owned
inventory. Check source contracts for bounded stirring, enzyme activity, ideal
organic templates, chromatography and thermal properties. For thermal selections,
compute sensible/latent energy from pinned properties; for separation, verify
per-vessel ownership and declared energy boundaries. Reproduce the independent
material reader. Keep rounding, missing phase domains and property provenance
explicit. Check the source's additive liquid volume, including peroxide, for
KN009; the saved concentration ratio18.9029 exceeds consumption ratio8.47084.

The [detailed kinetics preparation](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/saved64-kinetics-preparation/README.md)
reproduces all14 kinetics timelines, source mixing bounds and original model
limits without assigning new passes. Use it to complete that group's review.

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
result labels all0.76475987mol as liquid at692.68K. No retained solid remains. The [independent saved thermal calculation](https://github.com/CrispStrobe/kerotakis/blob/f865e4c23566043f3224598d466a4459757cfe62/audits/target468-20261016/saved64-thermal-preparation/README.md) accounts for458.65774450J sensible plus5598.04221474J latent for that full liquid fraction:6056.69995924J against the1000J pulse. Keep the5056.69995924J mismatch and original phase/event disagreement; do not fit a property to erase it.
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

Prepared [PR828](https://github.com/CrispStrobe/kerotakis/pull/828) addresses the destructive unchanged-chemistry readback boundary, with a separate smaller native first failure. It is normally integrated as `0a5a92f1` after25 CI, both native platforms and33 original logical fleet jobs; actual changed-source artifact,11 API,37 browser and original TH016 replay acceptance now pass. The guard is restricted to supported paid coexistence plateaus; it does not implement general phase-aware CEA input enthalpy or energy closure across reacting HP phase transitions. Keep those algorithmic tasks separate.

## D. Share authoritative aqueous base with named ester hydrolysis

**Cases:** OR006–009. Explicit `react ... saponification` leaves ester unchanged
although positive base equivalents are owned; downstream protonation and carbon
cycle prerequisites therefore also fail.
**Source:** `crates/kerotakis-org/src/lib.rs`, core family routing and aqueous
transfer/readback; `crates/kerotakis-phreeqc/tests/family_aqueous_tail.rs`.

**Work:** distinguish the explicit requested stoichiometric template from the
warm kinetic family and inspect their hydroxide-capacity owners. Use actual
full-precision usable strong-base capacity, not rounded displayed OH activity or
residual NaOH molecules. The `base_equivalents` inventory coordinate is not that
capacity: an acetate buffer can own positive basis equivalents while having
negligible free hydroxide. Establish the authoritative capacity/admission
contract before consumption; if retained state is insufficient, repair its
ownership rather than infer caustic capacity from the coordinate. Instantaneous
measured `free_hydroxide` is not, by itself, proof of the total stoichiometric
reserve either: re-equilibration can recruit weak acid/base reservoirs. An
existing kinetic-family measurement contract must not silently become the
named to-completion verb's capacity contract. Establish measured-versus-literal
hydroxide ownership before adding those amounts, and verify numerical negative
controls against the actual admission threshold. Account for
weak-acid/bound-proton reservoirs and prior neutralization before ester cleavage.
Do not add naming aliases for an inventory-ownership failure. Keep the
counterion ledger explicit: consuming hydroxide equivalents while depositing
neutral sodium acetate can double count sodium if the spectator cation is also
retained. Test that path and either support other alkali counterions with a
balanced net ionic reaction or state their precise admission boundary.

**Done:** base-limited and ester-limited requests form matched acetate/ethanol,
retain the correct excess, and close C/H/O/Na/charge. Include prior aqueous solve,
changed pouring order, repeated request, transfer/save/cache and atomic refusals.
Include acetate-buffer, carbonate and ammonia negative controls, unsolved
molecular base, literal hydroxide and a non-sodium alkali spectator. Refresh the
solute-charge cache after the complete atomic ionic inventory change, as the
shared family path does. A successful base-only control must not cause false
cleavage in buffered samples. Keep heat, rate and warm-family limitations
separate. Reassess OR008/009 only
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

Prepared [PR829](https://github.com/CrispStrobe/kerotakis/pull/829) corrects the lost missing-model warning beside supported peaks. `outside_method` should remain the ionic-method exclusion list; an unsupported neutral belongs in a separate parameter-gap warning. The new smaller native failure confirms this distinction. Actual8 core and3 native controls pass at corrected head8e2fe429; all25 CI and33 logical fleet jobs now pass and normal integration is `b3c18f5c`; SP012 stays held for missing retention coverage.

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

## H. Match the actual Pages payload to its reviewed interpreter profile

**Input:** `.github/workflows/ci.yml` jobs `demo`, `iphreeqc-wasm` and
`wasm-bridge`, plus `tools/build-{web,iphreeqc-wasm}.sh`. The two engine/bridge
jobs explicitly select `IPHREEQC_BASIC_MODE=my-basic`; the `demo` job that builds
and publishes Pages does not. The build helper defaults to `disabled`. Pages
serving engine files200 therefore does not establish interpreter capability.
This is a source/configuration finding; measure the actual published interpreter
before claiming its observed failure. Keep PR827's current acceptance unchanged.

**Work:** after the current Vercel release, use a fresh narrow branch from actual
main. Bind the actual Pages JS/Wasm bytes and source metadata, then run a real
`USER_PUNCH` arithmetic probe and an uncached Rust/aqueous solve. Preserve that
first observation. Select the already reviewed MY-BASIC profile in the Pages
payload job and test the generated site files themselves, rather than infer
capability from a separate engine build or `canSolve()` flag. Keep the existing
native/browser numerical comparison and interpreted/unsupported-mode tests.

**Done:** source-bound CI verifies the actual published-payload engine executes
BASIC and solves an uncached solution consistent with native. Complete applicable
CI/fleet acceptance through normal protection, publish Pages, and independently
verify the live bytes and capability. Keep REST-client/CORS acceptance separate
from full Pages chemistry acceptance. No new interpreter, parameter data, science
selection or calibration claim is needed for this build-profile repair.

## I. Make producer dependency resolution reproducible and explicit

**Input:** root `.gitignore`, `tools/build-{web,rest-native}.sh`, the chemistry
fleet's CLI artifact and release producer receipts. Root `Cargo.lock` is ignored
and absent from the reviewed Git revisions; it is generated during builds. The
fleet artifact preserves actual lock bytes, and PR827 adds them to the native
release artifact. A source checkout's local lock is not automatically the lock
that produced a saved binary.

**Work:** independently hash the actual producer lock alongside the binary,
source/tree, runtime libraries and build/toolchain metadata. Compare against a
Git lock blob only where one exists; never label a missing blob an engine or
experiment failure. Retain the generated lock and its exact binding under a
public evidence filename where publication is appropriate, and preserve private
producer artifacts separately. Qualify frontend dependency binding separately
if no stage-specific lock was captured.

Review the intended workspace lock policy and choose a reproducible contract:
either a reviewed source-tracked workspace lock or an explicit frozen producer
lock supplied to clean builds. Audit the frontend/native build order, source
export, all supported targets and dependency/license checks before changing the
policy. Do not silently reuse an unrelated local lock or run `cargo update`
against a historical experiment's binding.

**Done:** a clean checkout with the declared producer inputs can follow the
explicit resolution/build contract; actual generated/consumed lock bytes are
recorded and independently verifiable. Native and frontend claims distinguish
what was captured. Existing source-qualified results, original locks, first
failures and scientific counts remain unchanged. Byte-for-byte rebuild or
empirical-calibration claims require their own evidence.

## Actual deployed-source follow-up

The [four unchanged diagnostic protocols](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/deployed-shared-diagnostics-20261018/README.md)
now have source/binary/lock-bound API results on `ed7c452d`, after actual release
acceptance. KN-007 passes both original criteria and whole H/O/Mn/charge/filter
oxygen checks; its separate selection does not rewrite the old oxygen-loss
outcome. GS-010, OR-006 and TH-016 remain excluded. PR828 `5b054e6c` passes111
focused tests but is not deployed; do not overwrite TH-016's production failure.
Actual Pages source `76fe144` fails interpreter arithmetic with BASIC disabled,
confirming lane H's configuration gap. No empirical calibration is inferred.


## Current-source brine cause and finite molar-input repair

The original GS-010 deployed result remains a failure. A separately frozen
smaller CLI control and real solver comparison identify an input-boundary
mismatch: Kerotakis supplies ideal pressure, while Pitzer CO2 initializes moles
through its real-gas EOS. Supplied0.001mol returns0.0010003390707956424mol.
The native raw component-moles format instead returns0.000999999999999987mol
without changing the database or EOS. This is an initialization conservation
control, not empirical salting-out calibration.

[PR833](https://github.com/CrispStrobe/kerotakis/pull/833), exact `8eece0b1`,
prepares the generic finite-gas boundary repair and native controls across
loading, temperature, dilute/brine routes, repeated settlement and initially
empty gas phases. Read its `docs/finite-gas-molar-initialization.md` and
`audits/target468-20261016/finite-gas-eos-first-control/`. Combined source `e9cc8782` now passes Clippy,604 PHREEQC/CEA tests,31 core controls and all33 original logical fleet jobs. Current protected/platform checks and normal integration remain pending.
Preserve the original failure, use normal exact-head integration, then bind a
separate actual-production GS-010 replay before any working-catalog selection.
Do not rescale readback or replace the reviewed thermodynamic constants.


## Final saved-gas review completed for three candidates

GS001, GS003 and GS009 now satisfy all six original criteria within their original
finite ideal-carrier/carbonate domains. The [independent reader and assessment](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/saved-gas-final-review-20261018/README.md)
retain actual temperature, trapped-air baselines, pressure increments, carbon
fractions, per-arm additions and numerical residuals. No engine was rerun.
The previous201 catalog records remain unchanged; the new complete catalog has
204 entries, including200 fixed468 selections. The other41 candidates still
need final review;20 original holds remain historical. GS008's explicit
nonideal-correction criterion remains pending, and GS010 is still excluded.
Do not interpret internal ideal-gas closure as quantitative solubility or
real-gas calibration. The receipt preserves the first assessment-tool failure
and its correction to the existing molar inventory resolution.

## J. Give coupled ammonia protonation and headspace exchange one owner

**Observed hold:** original GS005 changes free aqueous NH3 from about0.000947mol
to6.35e-11mol on acidification, yet gas NH3 stays near3.57198e-6mol in both
arms. Nitrogen closes; the tiny gas difference does not establish acid trapping.
A [separate smaller accepted-equivalent control](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/ammonia-coupled-first-control-20261018/README.md) now reproduces this failure on runtime identical to accepted `0a5a92f1`, binary `6c617d2b`. All three arms retain nitrogen/chloride; both acidic arms suppress free NH3 but retain over90% of neutral gas. Raw observations and the first saved-reader defect are preserved.

**Source lead:** `crates/kerotakis-core/src/volatility.rs` partitions neutral
NH3 with a standalone Henry law, while
`crates/kerotakis-phreeqc/src/derived.rs` currently grants native finite gas
ownership only to CO2/HBr. Aqueous NH3/NH4+ protonation and gas exchange must
converge together. WATEQ already carries an NH3 gas phase; inspect the actual
selected databases and their capabilities rather than assuming every route has
that phase or compatible thermochemistry.

**Work:** freeze matched neutral/acid controls with finite N/H/O/charge ledgers,
actual temperatures, aqueous free/bound nitrogen and headspace NH3. Trace all
writers and their order in the standard solver stack. Define explicit coupled
gas capability/ownership so the authoritative aqueous solver can handle its
supported wet domain while core-only, native-disabled and pure-liquid Henry
fallbacks retain their existing scoped behavior. Preserve inert N2/O2 and any
retained-gas ledger. Do not merely add another independent gas writer or disable
all NH3 partitioning globally. Dataset fallback or borrowed gas data needs its
own declared provenance, temperature range and source-bound acceptance.

**Done:** acid trapping changes actual owned headspace NH3 while whole N/H/O and
charge remain closed, diagnostics agree with owned phases, repeated unchanged
settlement is stable, and neutral, acidic, buffered, initially empty gas,
pure-liquid and unavailable-backend controls remain honest. Keep the original
GS005 failure/hold and add a separately bound replay only after normal protected
integration and actual delivery. A model gap or unavailable gas capability is
not a scientific pass.

PR834 initial `2750df2d` implements the bounded finite-only ammonia coupling proposal. First native validation fails all four new tests (523 others pass); diagnostic follow-up `4d8e1705` retains the chemical assertions. Missing minteq NH3(g) is an explicit unchanged-state refusal; core-only/dry fallback and existing open/swept behavior remain. Final combined-source acceptance must include the separately accepted finite-gas initialization repair. Original GS005 stays excluded until its own separately bound repaired-source replay passes.

Current delivery: PR831 merged normally as `ac1f9993` with verified parents/tree. The phase-source `0a5a92f1` artifact and11 API controls are accepted, binary `e01b96e0`; unchanged37-check actual browser validation is running with corrected engine-only asset binding; its first preflight failure is preserved, after which only the changed-source TH016 replay is authorized by its frozen plan. Last fully accepted API/browser release remains `ed7c452d` until that review completes. [First ammonia native failure](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/ammonia-native-first-failure-20261018/README.md) remains preserved; no new scientific selection.

[Current phase-source artifact/API evidence](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/phase-release-api-20261018/README.md) records actual delivered bytes and11 passing controls. Browser and TH016 acceptance remain separate.

## Current phase and ammonia follow-up

[Changed-source TH016](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/phase-release-browser-thermal-20261018/README.md) now independently passes both original criteria after full artifact/API/browser acceptance. The old failure and every previous catalog entry remain preserved. Complete catalog205 / fixed468201 needs200 further selections for401. This does not implement general reacting multiphase HP energy.

PR834 current `6dee9254` replaces constructed display-formula gas names with explicit native capability identities. [The actual failed admission trace](https://github.com/CrispStrobe/kerotakis/blob/a1162023a4c44045ba6edfcfd2e0ddb9fabb4bd1/audits/target468-20261016/ammonia-native-admission-trace-20261018/README.md) records `NH3(aq)(g)` being dropped, explaining failed gas absorption/trapping and missing-route refusal. All original assertions remain; full native acceptance is running. Finish combined-source acceptance with the initializer prerequisite and normal protected integration before separately bound GS005 replay. No ammonia scientific selection has been made.

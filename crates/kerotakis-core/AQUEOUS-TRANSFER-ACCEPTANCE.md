# Aqueous transfer ownership acceptance

The original AB-029 script is preserved byte for byte in
`crates/kerotakis-phreeqc/tests/fixtures/ab029-filter-suspension.lab`
(SHA-256 `cf7a5085df974af447178e328821b2ec41c883e27092929b790d25d670e12c9c`).
The hosted c446 result completed with a `solver_failed` event after filtering;
its first output and physical forecast remain unchanged.

Filter and proportional Decant move the existing measured hydroxide/proton and
solute-charge baseline with the aqueous aliquot, once. These fields are not new
ions or a newly computed solution. The receiver still needs ordinary fresh
characterization. Dry or organic-only transfers and zero-fraction decants do not migrate old
aqueous diagnostic coordinates without a positive liquid-water carrier.
Copying a solution cache, inferring measured hydroxide from
analytical base equivalents or adding compensating ions is forbidden.

The patch also retains gas when filtering: only selected liquid/aqueous matter
moves. Existing admission and headspace mechanics apply; this change does not
introduce a new closed-flow, exhaust, pressure-work or bubble-residence model.
Mix, Drain, Extract and transport retain their separate existing contracts.

## Required hosted checks

- `kerotakis-core/tests/aqueous_transfer_snapshot.rs`: full and proportional
  transfers, empty/new/nonempty receivers, acid/basic/neutral measured snapshots,
  signed charge, no-carrier/zero-fraction handling, no additional matter, retained solids/interface/gas, current
  StepStart baseline, invalidated caches, safety refusal, numerical-overflow
  rollback and failed solver-trial rollback.
- `kerotakis-phreeqc/tests/aqueous_transfer_snapshot.rs`: run the unchanged
  AB-029 protocol with the standard native stack. The filtrate must obtain a
  fresh pH; an unchanged aliquot must not acquire a transport-only temperature
  shift. Preserve the suspension-versus-filtrate capacity contrast after the
  same acid dose. Repeated pours of already solved caustic solution must not
  repeatedly charge its hydroxide reference enthalpy.
- A separately bound hosted CLI replay of the unchanged script remains required
  for delivery acceptance; the native unit fixture replays its bench operators
  and does not establish CLI output/exit behavior by itself.
- Preserve existing proportional-transfer, native order-invariance, continuation,
  cache/Wasm and full applicable fleet checks. Native acceptance is pending;
  formatting/source review alone does not establish chemistry success.

The saved source carried 2.5382005404967627e-5 mol measured hydroxide. Its missing
55.81 kJ/mol reference term is about 1.41657 J, or 0.003399 K on the local water
heat-capacity scale. That resembles the reported final iteration residual but is
not proof of the exact 64-pass recurrence. Do not change thermal residual/cycle
thresholds, iteration limits or numerical brackets to obtain this acceptance.
If unchanged AB-029 still fails after ownership is corrected, preserve that
result and collect a separately authorized hosted temperature/heat/OH trace.
No empirical reaction-heat or solubility calibration is claimed.

# Current source acceptance: detergent order and heat

Source `3d21174a87938b4054ddd9ba7ca189d4749a9199`, tree `7be22fc0c4d41f0ba5cd3af76c962d494554c28c`, passes all twelve newly added controls in [full preflight](https://github.com/CrispStrobe/kerotakis/actions/runs/40224476658/job/121264675995): five library controls, four native opaque-clock controls, two native chalk controls and the native three-dose exhaustion/zero-time control. This does not constitute full source or product acceptance, or a new original-experiment qualification.

The same preflight then fails `the_same_beaker_reads_the_same_solvent_mass_in_either_order` in `crates/kerotakis-phreeqc/tests/order_invariance.rs:229`. Original full log SHA-256: `9ab30c8577f09718d043dbf8ecee2e9ee69793ae83edcef8f191b9042097a223`. The reported readings are:

| Order | Solvent kg | Ionic strength | pH | Liquid water mol |
| --- | ---: | ---: | ---: | ---: |
| Salt first | 0.0996939731 | 0.2759702852 | 10.9521300644 | 5.5339424445 |
| Powder first | 0.0996940239 | 0.2761694037 | 10.9579919224 | 5.5339452597 |

Only the preceding solvent-to-water checks have passed when the ionic-strength assertion fails. The later cross-order pH/water comparisons and gas-adjusted H/O/element/charge assertions are **unreached**, not passes. Both exact histories have now been captured once in the [immutable measured capsule](../audits/order-heat-current3d-20261025/SOURCE-ADMISSION.md). Salt-first ends at300.1316387105029K and powder-first at299.9119677430168K, with no gas transfers or SolverFailed events. A separate known-formula receipt calculation closes H/O/C/Ca/Na/Cl/charge within1.4e-14mol/eq and preserves the3.5g unresolved remainder; this does not make the original unreached assertions passes. Heat-mode tracing has not been executed. Full CI is terminal: both native platforms and preflight fail the same order test; remaining checks and all33 fleet jobs pass.

## Lane A: exact histories captured — do not replay unchanged

The two protocols below have each run once on the authenticated current-source CLI. Their complete per-operation JSON, stderr, exact scripts, source/tree, executable hash and actual producer lock are preserved in the capsule. Do not repeat them unchanged. A changed source requires its own successor and fresh resource checks.

```text
add v1 water 100mL
add v1 CaCl2 0.01mol
add v1 laundry_detergent 5g
```

```text
add v1 water 100mL
add v1 laundry_detergent 5g
add v1 CaCl2 0.01mol
```

The CLI emits successive JSON objects; decode every operation, not just the final object. Check SolverFailed and unsupported-state events first. Compare actual temperature and thermal boundary, signed gas by species, carbonate/calcium phases, current aqueous route/scope, liquid water and known stock. Recompute known inputs from Added/MaterialAdded receipts; retain the detergent's exact recipe/version and unresolved 3.5 g without inventing atoms for it. Count physical contents once; lot and free-ion diagnostic views are not additional matter.

## Lane B: identify and repair the heat ownership or data gap

Inspect `enthalpy::DISSOCIATION`, `species_enthalpy`, `cumulative_dissolution_fallback`, the `heat_released_j` caller and `HeatMode`, and the original StepStart/adiabatic mixing boundary. Anhydrous Na2CO3 has no entry in the curated dissociation table or registry dissolution-heat parameter; the routed WATEQ carbonate phases are hydrates. The whole-step quote can therefore decline and fall back to dissolution events, omitting other known heat. This is a source-backed hypothesis for the measured order difference, not an established causal diagnosis.

If confirmed, price the exact anhydrous salt through sourced reference data and a formula/charge-balanced dissociation. Do not borrow a hydrated phase or invent an enthalpy. The [OECD/NEA evaluation, section V.7.1.3](https://www.oecd-nea.org/upload/docs/application/pdf/2024-06/cts15-selected_ancillary_compounds-r-final-rev.pdf), provides a candidate standard dissolution reference at 298.15 K and infinite dilution; its standard-state scope must be retained. It is not an empirical calibration of this concentrated mixture. Check every consumed registry-data binding when changing the pack.

Preserve the existing ionic-strength, pH, solvent and physical-ledger limits. Do not add a thermostat merely to hide the original failure. If the histories actually have different boundary flux or inlet energy, demonstrate that independently and retain the original outcomes before defining a separate, physically matched comparison. Every source change needs its own actual execution and retained first failures.

## Lane C: finish the existing ordered task

Complete full exact-head native/platform/preflight and all33 fleet acceptance, then normal reviewed-head PR859 integration and artifact/deployment verification. Assess the retained15 API/86 browser controls with explicit scientific domain holds. Next qualify unchanged originals under `docs/current-continuation-20261025.md`, then implement and deliver owned runtime diagnostics in EN/DE/FR with raw technical detail and old event JSON preserved. Close the task only after that last item. Current qualification remains338/468; no new production release is claimed here.

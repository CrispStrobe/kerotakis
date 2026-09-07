# Sixth fleet contract: cases 195–218

Status: frozen, not validated on the repaired PR head. Do not edit expectations
after execution. Run only after PR #504 CI passes and a fresh CLI is tied to its
source revision and SHA-256.

## Coverage

| Cases | Family | Independent question |
| --- | --- | --- |
| 195–200 | Descending acid titration | Endpoint capacity, concentration, predose, solvent and extensive controls |
| 201–206 | Conductivity | Blank, charged/neutral solute, dose and intensive scaling |
| 207–212 | Dry sealed gas energy | Owned gas, headspace, partitioned dose, extensive scale and energy round trip |
| 213–218 | Explicit organic equilibrium no-op | Forward/reverse approach, scale, low water and repeated requests |

All 24 scripts must produce complete JSON, a final bench, finite nonnegative
registered inventory and no solver failure. Missing output fails all applicable
checks. No exact experiment ID may influence runtime behavior.

## Frozen checks

- Titration: delivered HCl equals initial NaOH minus predose within `2e-7 mol`
  plus `0.002` relative; endpoint pH is within `0.02` of 7. Na and Cl close
  within `1e-8 mol` plus `1e-6` relative.
- Conductivity: report finite nonnegative aqueous µS/cm. Electrolytes exceed the
  blank by `0.01 µS/cm`; dose ordering is qualitative. Extensive scaling agrees
  within `0.01 µS/cm` plus `1e-4` relative. Neutral glucose changes the blank by
  less than 1% of the matched nitrate excess signal.
- Dry gas: use actual retained gas, temperature and headspace in `P=nRT/V`, with
  `R=8314.46261815324 Pa L mol⁻¹ K⁻¹`; pressure tolerance is `0.001 Pa` plus
  `1e-6` relative. N/O/H inventory closes within `1e-10 mol` plus `1e-6`
  relative. Temperature controls use `0.002 K` plus `1e-6` relative. Sealing
  traps atmosphere, so total gas is not the supplied nitrogen alone.
- Ester equilibrium: apply signed extent to the molecular inventory immediately
  before `react`; require `|Q-4| < 1e-4`. Each supported equilibrium request
  emits `org_reacted` with a nonempty boundary. Later extents are below `1e-8
  mol`; post-first and final phase/species inventories agree within `1e-8 mol`
  plus `1e-6` relative, with whole-vessel C/H/O conservation.

These checks cover declared model behavior, not thermostat accuracy, nonideal
mixed-solvent activity, catalyst kinetics, electrode behavior, heat loss or
instrument calibration.

## Execution

Use a new output directory. The analyzer refuses to overwrite its report.
Do not use the existing `target/debug/kero`: it predates the current repairs.
Set `AUDIT_REBUILT_CLI` to the absolute path of the explicitly supplied rebuilt
binary. Before execution, record its source revision and SHA-256 and separately
record SHA-256 values for `sixth_batch.py`, `analyse_sixth.py` and this contract
in the new run's preflight manifest.

```sh
python tools/chemistry-audit/analyse_sixth.py --self-test
python tools/chemistry-audit/sixth_batch.py \
  --binary "$AUDIT_REBUILT_CLI" \
  --out tools/chemistry-audit/sixth-batch-1
python tools/chemistry-audit/analyse_sixth.py \
  tools/chemistry-audit/sixth-batch-1 \
  --out tools/chemistry-audit/sixth-batch-1/law-checks.json
```

Before interpreting output, verify every preflight hash and the binary's source
revision. Preserve every failure and use a different directory for any replay.

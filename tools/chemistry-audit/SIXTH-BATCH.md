# Sixth original fleet: 195–218

Twenty-four independently authored virtual controls, frozen before execution. No source-manual recipes, copied empirical outputs or experiment-specific production behavior. Execute only with the rebuilt CLI containing the explicit equilibrium no-op diagnostic. The shared recorder still starts one isolated process per case; do not concatenate experiments into a shared bench to improve timing.

## Preservation review of the fifth fleet

The existing `limiting-reagent` entry in codex/quantitative.toml already contrasts unequal silver/chloride feeds, and `silver-chloride-precipitation` plus the aqueous solubility/complexation entries cover related silver chemistry. Therefore fifth cases 159–164 are valuable regressions, not a compelling duplicate lesson. Existing `equal-charge-different-clocks` makes fifth equal-charge electrolysis variants principally additional sealed-boundary verification.

Two stronger candidates merit original app authoring after root's integration review:

- Diprotic capacity and known predose, fifth 171–176: acid molecules are not acid equivalents; a measured prior neutralization subtracts from remaining demand. Distinct from existing `endpoint-is-not-a-full-drop`, which teaches numerical final-step refinement. Prefer combining baseline and predose, not another coarse-step demonstration.
- Hydroxide-limited zinc precipitation and acid reversal, fifth 165–170: the base supplies two equivalents per precipitated metal, and later acidity can undo the solid. This offers a different limiting-capacity lesson from existing 1:1 silver precipitation. Describe total solid metal rather than hardcoding a hydroxide polymorph. Root should check its in-progress entries before adding another ID.

Repeated ester equilibrium (192) is chiefly a regression and diagnostic improvement; existing `equilibrium-can-run-backward` already provides the stronger conceptual introduction. Three-salt grouping adds multicomponent native coverage but overlaps `grouping-does-not-change-water-heat` in pedagogical structure.

## New coverage

| Cases | Question / distinct control |
|---|---|
|195–200|Descending titration with acid: trial refinement, concentration, predose, solvent and extensive controls|
|201–206|An actual conductivity readout: explicit blank, charged versus neutral solute, dose and intensive scaling|
|207–212|Small-energy dry sealed gas: headspace, partitioned dose, extensive scale and reversible energy round trip|
|213–218|Explicit no-op diagnostics after forward/reverse equilibrium, extensive scale, low water and repeated requests|

These families test boundaries not established by merely rerunning the fifth input values. The balanced-quotient case has nominal Q=4, but checks use the actual molecular inventory immediately before `react`, not an assumption that analytical feed equals free molecular amount. It is valid for the first equilibrium solve to adjust a changed molecular state. Every later request must explicitly report the supported family with negligible extent and preserve material state. No check matches exact explanatory prose.

## Frozen independent checks and tolerances

- All 24 cases require successful complete JSON observations, finite nonnegative formula-mapped inventory, matching final state and no solver failure. Missing streams cannot pass.
- Descending pH endpoint: delivered HCl equals initial NaOH minus HCl predose, tolerance `2e-7 mol + 0.002 relative`; endpoint pH within 0.02 of 7. This is dilute strong-acid/base capacity, not a source burette calibration or exact neutral-temperature claim. Na/Cl conservation independently includes titrant delivery; default inventory tolerance `1e-8 mol + 1e-6 relative`.
- Conductivity must explicitly use the aqueous µS/cm unit and finite nonnegative values. Electrolyte contrasts exceed blank by 0.01 µS/cm; dose ordering is qualitative, not linear concentration fitting. Extensive scaling tolerance `0.01 µS/cm + 1e-4 relative`. Neutral glucose's blank shift must be less than 1% of the matched dilute nitrate excess signal. Domain: dilute neutral glucose and dilute 1:1 salts, same initial temperature; no concentrated-solution accuracy claim. The existing model includes a concentration correction; this is not a claim of a pure limiting-law implementation.
- Dry gas: actual retained gas amount, pressure, temperature and headspace must obey `P=nRT/V`, R=8314.46261815324 Pa L mol⁻¹ K⁻¹; pressure tolerance `0.001 Pa + 1e-6 relative`. N/O/H inventory after the gas loading but before heating is retained, `1e-10 mol + 1e-6 relative`. Sealing introduces atmosphere, so do not equate total gas with the supplied nitrogen alone. Small-dose temperature partition/extensive controls allow 0.002 K + 1e-6 relative; round trip allows 0.002 K plus default relative tolerance. No heat-capacity table is imported, and no real heat-loss or sensor-rate claim is made.
- Ester: signed reaction extents applied to the actual pre-react molecular inventory must give Q=4 within `1e-4`. Every requested equilibrium operation must emit an `org_reacted` result with a nonempty boundary; subsequent extents below `1e-8 mol`. Post-first-reaction versus final species/phase inventories agree within `1e-8 mol + 1e-6 relative`, with whole-vessel C/H/O conservation. These are ideal-equilibrium checks, not catalyst kinetics, reaction-time predictions or real nonideal activity validation.

No output-fitted bounds. Each failed check remains unmet with evidence, including unsupported/refused descending behavior if discovered. Model limitations must not be relabeled scientific passes. Bench log counters are excluded from no-op physics comparison; temperature and pressure are also checked with 0.002 K / 0.001 Pa absolute plus 1e-6 relative tolerances.

## Commands

```sh
python tools/chemistry-audit/analyse_sixth.py --self-test
python tools/chemistry-audit/sixth_batch.py --out tools/chemistry-audit/sixth-batch-1
python tools/chemistry-audit/analyse_sixth.py tools/chemistry-audit/sixth-batch-1 --out tools/chemistry-audit/sixth-batch-1/law-checks.json
```

Authoring verification: 24 unique scripts; empty-output negative control rejects all 66 checks. No CLI experiment execution or builds during authoring.

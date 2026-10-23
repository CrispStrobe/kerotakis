# Native mixing: whole reaction-heat and aqueous-coordinate ownership

This is the next thermal lane after PR859 acceptance. It makes no new experiment qualification claim. Preserve the completed51 fusion controls and finish source/full/platform/product acceptance without optional changes to that reviewed head.

## Refreshed source finding

The [prospective contract](../audits/target468-20261016/mix-thermal-prospective-contract-20261023/contract.json) pins actual main`4eee0d63`. In `crates/kerotakis-phreeqc/src/aqueous.rs::PhreeqcEquilibrator::mix`, the native route rebuilds composition, writes measured hydroxide/proton and characterizes pH, then returns. Its guard comment claims whole-step thermal bookkeeping, but this path contains no call to the ordinary enthalpy-state balance. In `crates/kerotakis-core/src/bench.rs`, the successful native return also clears StepStart and bypasses ordinary equilibration. Separately, the Mix operator scales liquid/aqueous portions without transporting/scaling the measured free-hydroxide/free-proton/solute-charge coordinates already handled for Filter and Decant. These are concrete source paths to test, not an executed current-runtime failure or proof that the historical TH015 outcome persists.

## Freeze the baseline before repair

Prepare the five declared controls without modifying runtime. Capture every source and receiver before and after each operation, actual route/events and phase inventories. Run against the exact accepted source and retain first outcomes. Require that the intended native path actually runs; a silent advisory fallback cannot prove native parity. Keep actual donor dissolution temperatures and energy, including residual source fractions. Do not reset temperatures or count earlier NaOH dissolution again as neutralization.

An independent ledger must integrate each phase's published Cp from one declared reference and price every changed chemical state on one explicit routed basis. Compare final sensible energy with both actual pre-mix sensible energies plus the modeled reaction heat; derive the no-reaction temperature from the captured pre-mix inventories. Account all H/O analytical coordinates, counterions, water, gas, spills, reservoir/excess enthalpy and remaining donors. Preserve0.001J/1e−10mol budgets and refuse missing authority instead of inserting a heat sink.

Read the actual routed database. Native MIX selects wateq4f for these simple reagents: its water reaction prints13.362kcal/mol, or55906.608J/mol. The0.005mol limiting forecast is279.53304J, not an observed result. The ordinary phreeqc dataset prints a different reference; a generic55.81kJ/mol source comment is not authority for all routes. Exact expectations must use actual measured hydroxide consumption and all changed priced components, with reference/domain limits stated. Do not claim physical calibration or globally identical databases.

## Repair and acceptance

1. Transport measured aqueous coordinates only with their real water-bearing liquid/aqueous aliquot. Preserve zero-fraction/dry behavior, scale remaining donor fields and add to existing receiver fields before StepStart captures them. Do not treat measured free ions as additional matter or move interface/gas/solid reservoirs by a liquid fraction.
2. Give native MIX and ordinary fallback one complete, consistently referenced thermal account. Never add a second independent neutralization formula to the native path. Preserve source snapshots until the reaction-heat owner has priced the change; route-incompatible or incomplete inventory must decline before mutation. Native MIX must include an existing receiver or explicitly yield to the ordinary complete-target path.
3. Check equal/reversed source order, unequal fractions, inert salt mixing, remaining donor reuse and nonempty receiver/fallback. No heat may appear in an emptied source or be charged twice. Require real hydroxide/proton consumption and correct total counterions, not only neutral pH or temperature direction.
4. After baseline/fixed controls and review, complete full own-fleet/platform/protected and changed-release acceptance, normal exact-head integration and only then the unchanged original TH015 follow-up. Qualification requires its two original calorimetry criteria and complete ownership. Keep original requests/forecasts and historical failures immutable.

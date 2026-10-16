# Finite carbon ownership and initializer precision

## Demonstrated cause

The frozen carbonated-water opening screen supplied only neutral water, room
air and CO2, but accumulated about -3.9e-9 equivalents of formal charge in the
analytical inventory. The residual persisted after neutral CO2 vented. Its
carbon inventory drift was one quarter as large because carbon(IV) species obey
`charge = 4 C + H - 2 O`; the existing H/O basis conserves the latter terms.
This is an inventory/serialization issue, not evidence of missing counterions.

The native gas initializer receives partial pressures computed using the
unrounded vessel temperature. Previously its `-temperature` was printed to
four decimal Celsius places. In the saved sealed state, 298.1500375477956 K
became 298.15 K in native input. The resulting `T_raw/T_written - 1` is
1.259359e-7, matching the relative carbon discrepancy. The existing conversion
constant already matches IPhreeqc's `R_LITER_ATM = 0.0820597`.

The saved diagnostic execution used engine
`d791d3062886e4a06dbc7c9d1635383da3e05359`, binary SHA-256
`e837e75f6e1c2c8d3f86e56144a1cebd8e2a6edacc3a264e94ed1117a006c21c`
and lock SHA-256
`12d9a91b9348a577cb83f1f16db0d89c4994ea79d1f6967a611b20e8a71a78dc`.
Its unchanged script was `add v1 water 100mL`, `seal v1 100mL`,
`add v1 CO2 0.02mol`, then `open v1` (script SHA-256
`fff535eb1ebb3a50b8e4702746f3eea722bb68d7cb58104e5f43505646f97f04`).
The post-dose temperature, 298.5820854655798 K, rounded to 298.5821 K;
the corresponding -4.8678e-8 relative initializer error also matches the
approximately -9.7338e-10 mol carbon discrepancy in that step. These are saved
first-failure observations, not executions of the corrected source.

The causal change writes only the GAS_PHASE initializer temperature with
twelve decimal places in scientific notation. IPhreeqc's `tidy.cpp` computes
initial gas moles from the gas phase's own temperature, independently of the
SOLUTION temperature. The established direct/MIX SOLUTION precision is retained;
the change does not alter every aqueous equilibrium input. It does not change
the gas constant, equilibrium constants or physical forecasts. The finite-input
solvent precision repair is described separately below. A deterministic input round-trip control uses
the actual written gas pressure/temperature to recover supplied moles at that
fractional temperature.

A second deterministic input check identifies a separate finite-owner
quantization: totals are written as `n / kgw`, but the old SOLUTION water field
rounded `kgw` to nine decimal places. The native supplied aqueous carbon was
therefore `C_aqueous * W_written / W_raw` before equilibrium. A fractional
0.099699987654321 kg solvent fixture demonstrates the error and its formula
from serialized text. Owned finite-gas inputs now write water with the same
twelve-decimal scientific precision as concentrations. Open aqueous and MIX
water formatting retain their existing precision, so this does not perturb
unrelated filtration equilibria. The native regulated residual may have further
contributors; this independently proven input repair is not a passing native
validation claim.

## Scope of this repair

This change fixes the proven finite-input temperature and solvent-amount
serialization mismatches. It does not
project analytical carbon, change H/O closure, alter global tolerances or
introduce a new finite-gas owner. An unexecuted draft of bounded carbon
reconciliation is retained privately for follow-up if corrected native evidence
still demonstrates a separate inventory residual. It is excluded from this
patch until such evidence establishes the need and its applicable domain.

## Acceptance and exact-source limits

Deterministic controls cover initializer round trips at all three recorded
fractional temperatures, total carbon from fractional solvent plus owned gas,
the old solvent-error formula, and retention of the existing open-water format.
Native controls reproduce the actual sealed/open protocol
and repeated waits, plus a pressure-controlled finite volume. They require
carbon, H/O and formal-charge conservation and unchanged inert N2/O2; they are
engineering ownership checks, not independent Henry-law calibration.

The first native attempt at `727966a5` passed the sealed/open control, but the
regulated control retained a -2.815e-12 mol carbon residual against its 1e-12 mol
assertion. The existing filtration control also found a solid in the filtrate
under the initially broader SOLUTION precision change. Both first failures are
preserved. The present revision restores SOLUTION/MIX temperature precision, preserves
open/MIX water precision, repairs the independently demonstrated finite-gas
solvent input error and adds state/amount/ledger context to failures; no
assertion tolerance has been changed.
Corrected native acceptance remains pending.

Preparation base: integrated main `e027e964`. Formatting and source diff checks
have passed. Native execution has not been performed locally because resource
prerequisites were not satisfied. Focused native tests and the applicable full
chemistry/own-fleet checks remain pending; no passing outcome is claimed.

Serialized native inputs are content-addressed cache keys. The precision change
therefore creates new keys for affected states. Existing caches must be
regenerated and accepted against this exact source, rather than relabeled or
patched. Preserve old inputs, first results and evidence; compare scientific
outcomes independently from process completion and cache regeneration.

# Fourth discovery fleet: cases 123–158

Status: authored before execution; **no results claimed**. Root owns running
these scripts against the fresh verified CLI and preserving new evidence.
This set contains 36 distinct virtual experiments, not cached outputs or
production case branches. No new dependency, dataset or licensing exception
is introduced; arithmetic uses Python's standard library and existing
reviewed runtime property inputs. Existing project licensing is unchanged.

## Scope

| Cases | Question | Independent test |
| --- | --- | --- |
| 123–128 | Water/ethanol/methanol cuts, small scale, feed order, two latent budgets, then a four-component cut | Per-component conservation, amount fraction, sum of condensate amount times latent enthalpy, intensive scale/order, monotone component withdrawal |
| 129–134 | Unequal acid/base doses with nitrate spectator, reversed order, extensive scale, dilution before/after, completing the endpoint | Remaining equivalents divided by solvent volume, logarithmic dilution, matched pH/temperature, neutral endpoint |
| 135–140 | Sulfate-limited barium precipitation and magnesium precipitation/acid reversal/half-base control | Metal atoms in all solid phases, limiting-reagent capacity, order/scale, acid removal and hydroxide dependence |
| 141–146 | Finite sealed CO2 at varied headspace/dose/water, then one versus two heat pulses in dry sealed nitrogen/air | Actual initial sealed inventory plus supplied atoms, ideal-gas pressure, uptake direction, dry-gas thermal partition |
| 147–152 | Asymmetric ester mixtures, product perturbation, reverse direction, scale, exact Q=K, water-limited hydrolysis | Q from actual pre-request molecular inventory and signed extent, atom conservation, zero-progress control, direction and stoichiometric bounds |
| 153–158 | Acid-rich acetate buffer, dilution, inverted ratio, acid/base pulses and scale | Henderson–Hasselbalch ratio differences without assuming a pKa, matched perturbations and intensive scale |

The inventory-derived inputs to checks are not answer goldens: for example,
the ester check substitutes the reported signed extent into the independent
mass-action equation rather than copying the production root-finder. It
uses the actual molecular state immediately before `react`, avoiding a
false failure if an earlier owner changed a feed's protonation or phase.
The no-progress equilibrium control does not require an `org_reacted` event
for zero extent, but does require Q=K and unchanged molecular amounts.

## Model boundaries and accounting

- Distillation tests concern the reviewed ideal Raoult/constant-latent
  Clausius–Clapeyron approximation, not experimental nonideal yields or
  azeotropes. They require the model's explicit approximation disclosure.
  Reported energy is withdrawn condensate latent duty, excluding sensible
  heating and reflux/reboiler circulation. The reviewed USCG input rows and
  existing water/ethanol latent constants supply the independent energy sum.
- Precipitation counts metal atoms in **all** solid phases and reports phase
  identities; a valid stable oxide/hydroxide polymorph is not rejected merely
  for having a different formula label. The barium reagent amounts are well
  above trace-solubility scale; its 90% floor is a coarse precipitation
  capacity screen, not a claimed measured yield.
- Sealing traps air, and water may already contain atmospheric solutes.
  The closed-gas check starts from the bench **after sealing**, then adds the
  finite gas dose; it does not assume an initially empty headspace or zero
  carbon inventory. All contents phases count, including retained gas.
  External gas exchange after sealing is unexpected in these probes and
  must be explained if reported. No evaporation, purge or pressure-regulated
  reservoir is requested. Comparing inventory changes prevents bulk water
  from hiding a missing small gas dose in a relative tolerance.
- Dry-gas heat partition tests run far below chemical reaction temperatures;
  they test a closed, constant-volume gas model, not an apparatus heat-loss
  model. The ideal-gas equation uses total owned gas, not only the added N2.
- Ester mass action is the existing ideal-mixture K=4 computation, not
  uncatalyzed kinetics or a real synthesis yield. These organic-rich initial
  states lie below the water-dominant aqueous capability threshold.
- Buffer ratio laws are approximate dilute-solution controls; different
  activity coefficients and small speciation corrections are allowed.
  No universal experimental pKa is asserted or newly imported.
- Whole-fleet supplied-element checks cover Na/K/Cl/Ba/Mg, which have no
  requested gas outlet in these inputs; they do not pretend that every open
  vessel closes H/O/C/N/S automatically. Selected closed gas and organic
  checks add their explicitly scoped atom balances. These probes introduce
  no interfaces, adsorbed pools or material objects.

No standing adsorption, partition, osmosis, StoryMap, KIDS or GUI experiment
is included. These files do not modify the app catalog or its ownership.

## Tolerances frozen before output

- General amount/atom comparisons: 1e-8 mol absolute plus 1e-6 relative.
- Distillation scale/order: 1e-8 mol plus 1e-5 relative; latent identities use
  the general comparison in kJ, with amounts well above numerical trace size.
- Residual strong-acid ideal pH: 0.08; matched path/scale pH: 0.02;
  acid/base-order temperature: 0.02 K; fourfold dilution shift: 0.08 pH.
  Completed endpoint: 6.5 < pH < 7.5, allowing thermal neutral-pH shift.
- Barium precipitate: more than 90% of limiting sulfate, bounded above by
  its amount plus 1e-8 mol; order/scale agreement 1e-4 relative plus 1e-8 mol.
  Magnesium: more than half the supplied metal in the full-base control,
  acid reversal leaves less than 10% of that solid, and half-base gives a
  smaller positive amount bounded by its base-equivalent capacity.
- Sealed-gas P=nRT/V: 0.001 Pa plus 1e-6 relative, using SI R; finite CO2
  uptake monotonicity allows 1e-8 mol for the non-strict capacity comparisons.
  Dry-gas heat partition: 1e-5 K absolute and 0.01 Pa plus 1e-6 relative.
- Ester |Q−4| < 1e-4; reverse scaling 1e-4 relative plus 1e-8 mol;
  already-equilibrated control has |extent| < 1e-10 mol.
- Buffer dilution: 0.1 pH; inverted ratio: 0.08 pH; acid/base pulses:
  0.06 pH about the independently computed ratio change, with correct sign;
  extensive scale: 0.01 pH.

Failures retain the original evidence and call for diagnosis, not automatic
tolerance relaxation. Execution success alone is not a scientific pass.
Malformed output, missing required observations, nonfinite amounts, unknown
species compositions or solver failures produce unmet checks.

## Run when root confirms the CLI is ready

```sh
python3 tools/chemistry-audit/fourth_batch.py --binary target/debug/kero --out tools/chemistry-audit/fourth-batch-1
python3 tools/chemistry-audit/analyse_fourth.py tools/chemistry-audit/fourth-batch-1 --out tools/chemistry-audit/fourth-batch-1-checks.json
python3 tools/chemistry-audit/check_native_inventory.py tools/chemistry-audit/fourth-batch-1 --out tools/chemistry-audit/fourth-batch-1-native-inventory.json
```

Use fresh directories/files; existing evidence must never be overwritten.
The recorder keeps exact scripts, stdout/stderr, source patch, untracked
source content, runtime hash and execution metadata. Run the independent
native inventory checker as an additional check, not a substitute for these
law and control checks; zero applicable comparisons are not a pass.

## Third-fleet preservation candidates (recommendations only)

Subject to the third fleet's final evidence and editorial review:

1. **Three components, one conserved cut** (87–90): a compact ternary
   comparison can show every component moving and the latent-duty identity,
   while openly distinguishing ideal-model accounting from a real azeotrope.
2. **Equal charge, different clocks** (105–108): half-current/double-time and
   split-current controls teach Faraday's law without turning the lesson
   into a fixed gas-volume recipe; retain the external-flow atom check.
3. **Does the grouping change the heat?** (117–118): three-water mixing
   order visualizes a state-function balance with a clean independent energy
   derivation, avoiding the added ambiguities of reactive mixtures.

These are recommendations, not added catalog entries or claims that the
third-fleet outputs have been examined here. Avoid duplicating existing
electrolysis or heat lessons if a control extension conveys the same idea.

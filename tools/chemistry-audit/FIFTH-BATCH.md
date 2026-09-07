# Fifth computed experiment fleet: 159–194

This is a pre-execution protocol, not a results report. The 36 fresh inputs
and 119 checks were authored before running this fleet. They use the existing
native CLI and shared evidence recorder. No runtime code recognizes these IDs;
the audit checks conservation, equations and matched inputs, not golden answers.
No dependency or external dataset is added. Existing source licences remain
unchanged; this work does not introduce GPL/NC data or dependencies.

## Families and declared expectations

| Cases | New variation family | Independent checks |
| --- | --- | --- |
| 159–164 | Unequal silver/chloride feeds, inverted limiting reagent, order, extensive scale and solvent dilution | Solid silver stays within the smaller Ag/Cl feed, exceeds 90% of that capacity at these concentrations; dilution cannot increase solid; order/scale controls |
| 165–170 | Substoichiometric zinc/KOH, quarter/double base, order, scale and acid reversal | Positive precipitation below half the hydroxide feed; monotone base response; excess acid removes at least 90% of baseline solid |
| 171–176 | Diprotic sulfuric acid titrated with KOH; coarse/dilute burette, extensive scale, extra solvent, partial predose | Remaining capacity is twice the acid feed minus the KOH predose; measured titrant moles and pH endpoint agree |
| 177–182 | Sealed electrolyte-supported water electrolysis; headspace, equal/split/double charge and extensive scale | Independently calculate both Faraday products from current and duration; compare closed H/O/N/K inventories including initial trapped air and dissolved gases; calculate pressure from actual gas inventory and headspace |
| 183–188 | Three fractional calcium/magnesium/spectator salt feeds; regrouping, operand order, scale, dilution and half withdrawals | Receiver atom amounts from each specified transfer fraction; total atoms across every source and receiver; matched receiver pH |
| 189–194 | Alcohol-limited four-component organic equilibrium; feed order, scale, repeated request, extra alcohol and initial water | Signed mass-action extent, atom conservation, scale/order/idempotence, opposite reactant/product perturbations |

The second fleet already tested two-feed magnesium MIX and the third fleet
tested open electrolysis. These new families exercise three unequal feed
fractions and a sealed finite gas boundary respectively. Earlier silver tests
used equal larger feeds, and earlier zinc tests used stoichiometric hydroxide.
All 36 scripts must be pairwise distinct and not exact duplicates of fleets
one through four. Inputs use only registered species and existing CLI syntax.

## Frozen tolerances and model boundaries

- Default atom/amount comparisons: absolute `1e-8 mol` plus relative `1e-6`.
  These are absolute **and** relative tolerances, not a relaxed percentage-only
  test for depleted components. Nonvolatile feed accounting includes KOH added
  by the recorded titration, independently constrained by its capacity check.
- Silver capacity upper margin: `1e-8 mol`; lower margin: 90% of limiting feed.
  Silver and zinc matched order/scale comparisons use `1e-8 mol + 1e-4 relative`.
  Zinc dose checks assert direction and stoichiometric bounds, not a guessed
  equilibrium yield or known nucleation rate. Registered solids may compete;
  solid metal atoms are summed across phases rather than assuming one name.
- Titration: final pH within `0.02` of 7, explicit endpoint reached, KOH demand
  within `2e-7 mol + 0.002 relative` of remaining two-proton capacity. The endpoint
  is in dilute water near room temperature, where bisulfate is substantially
  dissociated. This is not a general claim that sulfuric acid has two equally
  strong dissociation steps at arbitrary concentrations.
- Faraday constant is independently `N_A e`, using exact SI definitions
  `6.02214076e23 mol^-1` and `1.602176634e-19 C`. Generated H2 and O2 tolerances
  are `1e-10 mol + 1e-6 relative`. Water is in large excess and KNO3 supplies
  conductivity. This validates the declared ideal charge-yield model, not
  electrode overpotentials, gas bubble dynamics or measured current efficiency.
- Sealed pressure: use actual reported gas moles, temperature and headspace,
  with `R = 8314.46261815324 Pa L mol^-1 K^-1`; tolerance `0.001 Pa + 1e-6 relative`.
  Equal/split-charge and extensive-scale pressure comparisons allow
  `0.1 Pa + 1e-5 relative`. Generated gas events are **not** added to final gas
  inventories: that would double-count internal reaction products. Closed atom
  checks compare the entire vessel immediately before electrolysis with its
  final state; external gas flow above `1e-8 mol` is a failure. No fixed H2/O2
  headspace ratio is assumed because dissolved gas partitioning is computed.
- Three-feed matched pH: `0.03 pH` across equal-concentration regrouping,
  order, scale and withdrawal controls. The doubled-solvent case is checked
  for receiver atoms, not forced to have identical activity-corrected pH.
- Organic equilibrium uses the existing declared ideal-mixture `K = 4`,
  with quotient tolerance `1e-4` and extent comparisons
  `1e-9 mol + 1e-4 relative`. Infer mass action from current pre-request amounts
  and signed event extents, independently of native aqueous projection aliases;
  final H/C/O atoms must still close. Every reaction event must disclose its
  model boundary. A repeated request must preserve the same total extent, not
  apply the old yield again. These are equilibrium controls, not rate/yield
  predictions for a timed synthesis.

Every case also requires successful process exit, nonempty well-formed output,
a final bench matching the last recorded bench, finite nonnegative inventories,
registered output compositions and no `solver_failed` event. Missing files,
fields or observations fail their checks; they are not silently interpreted as
zero material or skipped. A synthetic empty-output negative control exercises
all 119 checks and must reject every one. A failed model check remains an open
finding; do not widen these tolerances after looking at results.

## Run and preserve

From the isolated audit worktree, with a fresh output directory:

```sh
python tools/chemistry-audit/analyse_fifth.py --self-test
python tools/chemistry-audit/fifth_batch.py --binary target/debug/kero --out tools/chemistry-audit/fifth-batch-1
python tools/chemistry-audit/analyse_fifth.py tools/chemistry-audit/fifth-batch-1 --out tools/chemistry-audit/fifth-batch-1/law-checks.json
```

The recorder retains input scripts, raw output, stderr, source evidence and
binary fingerprint. The analyser refuses to overwrite an existing report.
Preserve original failures and use a fresh directory for any repaired replay.
Passing this fleet would support these specific laws and variation controls;
it would not establish that arbitrary chemistry is completely modeled.

# Third discovery fleet: cases 87–122

Status: **designed before execution; not yet run**. This fleet must wait for
the previous workspace gate and a confirmed fresh CLI. It is a new set of
virtual questions, not a replay of the first 86 inputs. It imports no new
data, changes no production chemistry and adds no runtime case dispatch.

The project code's existing licensing is unchanged. The audit harnesses are
original authored checks using Python's standard library, existing reviewed
property inputs and arithmetic. No new dependency or external dataset enters
the runtime. The few model constants used by checks are existing input
parameters, not retained simulator outputs.

## Coverage and controls

| Cases | New question | Independent checks |
| --- | --- | --- |
| 87–92 | Three-solvent cuts, molar scale/order, finite latent budget, aqueous-volatile refusal, pure-solvent limit | Component conservation, fixed fraction, intensive composition, sum(n ΔH), normal-boiling anchor, atomic unsupported-component refusal |
| 93–98 | HBr/HCl equivalence, closed HBr uptake at two headspace volumes, reversed finite-gas neutralization | Dilute strong-acid law, bromine atoms, matched gas-capacity trend, neutral endpoint and thermal path independence |
| 99–104 | Zinc and ferric hydroxide reversibility; carbonate precipitation from dissolved feeds | Metal capacity and conservation, sufficient-acid reversal, solvent dilution control |
| 105–110 | Nitrate-supported electrolysis and automatic electrode orientation | Q=It, n(H2)=Q/(2F), n(O2)=Q/(4F), equal/split/doubled charge, physical electrode identity under operand reversal, identical-half-cell zero driving force |
| 111–116 | Finite-acid kinetics under scale, time partition and donor perturbations | Acid/2 capacity, actual progress, sulfur product, intensive scale, predeclared time-partition screen, weak-buffer refusal and glucose positive control |
| 117–122 | Three-way heat/MIX, serial aliquots and four-component equilibrium | Sensible-energy balance, mixing associativity, residual acid equivalents, multiplicative aliquots, mass action and extensive extent scaling |

The cell command automatically chooses anode/cathode; it is **not** an
ordered voltmeter-lead command. Therefore case 109 expects the same physical
orientation and positive emf after swapping operands, not a sign reversal.
This distinction was checked against the command contract before execution.
Canonical registry identifiers are used throughout (no `acetone` alias).

Precipitation checks sum the supplied metal's atoms over **all solid phases**
and separately report each phase identity. They do not assume a hydroxide
must prevail over a stable oxide/oxyhydroxide or select a polymorph by name.
The acid-reversal and dilution comparisons therefore test material behavior
without turning an unverified phase-label expectation into a false failure.

## Supported laws versus capability boundaries

Cases 91 and 115 deliberately expect explicit, non-destructive model
boundaries, not invented predictions. A recognized aqueous volatile without
still parameters must not be silently excluded. A weak-acid buffer must not
be treated as a fixed free-proton reservoir by uncoupled consuming kinetics.
The glucose control ensures that the conservative donor guard does not
withhold every organic solute.

The additional-solvent still is explicitly an ideal-liquid approximation:
no quantitative real IPA azeotrope, activity coefficients or apparatus duty
are claimed. Its reported energy includes withdrawn condensate latent heat
only, not reflux circulation or sensible heating. The pure-methanol check
tests reduction to the model's reviewed anchor, not independent experimental
validation of the entire pressure curve.

Electrolysis accounting sums **newly generated electrode-product events**
once. Subsequent dissolution, headspace redistribution or open-boundary escape
does not erase production. Adding those event amounts to final inventory
would double-count the same material and is deliberately not done. Separate
nonvolatile-element checks follow inventory across every vessel.

## Tolerances declared before seeing output

- Inventory comparisons: normally 1e-8 mol absolute plus 1e-6 relative; sealed
  bromine and aliquot controls use 1e-9 mol absolute.
- Dilute strong-acid pH: 0.08 from ideal concentration, allowing dilute
  activity coefficients; matched acid identity 0.03 pH. Three-feed residual
  acid allows 0.1 pH for activity and small thermal effects.
- Neutralization path: 0.01 pH and 0.01 K, alongside an independently broad
  near-neutral check; no expected output temperature is supplied.
- Ternary scale/order: 1e-5 relative and 1e-8 mol absolute. Amount-based
  Rayleigh integration and common composition should preserve intensives.
- Faraday amounts: 1e-10 mol absolute plus 1e-6 relative, permitting rounding
  of the existing Faraday constant relative to exact SI N_A times e.
- Kinetic scale: 0.1% relative plus 1e-8 mol. Time partition: **1% relative
  plus 1e-8 mol**, a diagnostic stability budget, not a claimed exact
  semigroup law. These dilute runs exchange consumed monovalent H+ for Na+
  beside the same chloride inventory, so ionic-strength changes should be
  small; thiosulfate reaction heat is not priced in the existing rate model.
  Nevertheless each full-stack step refreshes activity/temperature, unlike
  the isolated frozen-coefficient kernel. A failure calls for analysis of
  approximation sensitivity, not automatic adjustment of this tolerance.
- Three-water energy balance: 0.001 K; associativity 1e-5 K. Both are wider
  than floating-point summation error and small native-water reconstruction
  effects, but narrow relative to the deliberately different initial baths.
- Four-component mass action: |Q−4| < 1e-4 and extensive scaling at 1e-4
  relative plus 1e-8 mol. The quotient is computed from authored feed amounts
  and the actual signed extent, before interpreting the downstream aqueous
  display. If dissociation/phase ownership changes that basis materially,
  preserve the unmet check and investigate the basis rather than fitting a
  golden answer.

Every case must execute, produce a final state and have no solver failures or
malformed JSON. Every final amount must be finite/nonnegative and every
reported species must have a formula. Missing measurements/events fail their
own checks rather than turning an absent prediction into a pass.

## Reproduction after the prerequisite gate

Run from the worktree using a **fresh** output directory:

```sh
python3 tools/chemistry-audit/third_batch.py --binary target/debug/kero --out tools/chemistry-audit/third-batch-1
python3 tools/chemistry-audit/analyse_third.py tools/chemistry-audit/third-batch-1 --out tools/chemistry-audit/third-batch-1-checks.json
python3 tools/chemistry-audit/check_native_inventory.py tools/chemistry-audit/third-batch-1 --out tools/chemistry-audit/third-batch-1-native-inventory.json
```

The shared recorder retains exact inputs, raw stdout/stderr, process status,
full source diff, untracked runtime/data source contents and binary hash.
Earlier failed results must never be overwritten. A discovery failure is
evidence to diagnose, not permission to weaken an assertion or script a
production answer.

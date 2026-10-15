# Charged inputs and physical reagent protocols

The existing aqueous adapter accepts analytical element components. Its native
PHREEQC input uses `pH 7 charge`: pH is adjusted to impose electroneutrality on
the supplied non-H/O totals. For example, an isolated chloride component is
interpreted as an acid analytical problem; an isolated sodium component is a
base analytical problem. This is an existing component convention, not an
implicit sodium/chloride bottle or a prediction for an isolated macroscopic ion.
See the [USGS SOLUTION documentation](https://water.usgs.gov/water-resources/software/PHREEQC/documentation/phreeqc3-html/phreeqc3-48.htm).

`contents` also includes analytical H/O closure coordinates. `H+` and
`base_equivalents` in a solved inventory are not measured free-ion populations.
Use `solution.species`, pH, `free_proton` and `free_hydroxide` for the computed
distribution. Feeding those closure coordinates back as independently measured
strong acid or hydroxide would change the established analytical problem.

For homogeneous inputs with net uncompensated formula charge, a named
`not-modeled.analytical-component-charge` diagnostic now explains the limited
meaning of the analytical result. The legacy calculation remains available.
The diagnostic considers the whole bulk inventory, including H/O coordinates,
and uses a declared canonical-readback allowance of 1e-8 mol of charge plus
1e-7 of the absolute charged inventory. This is the reporting resolution of
existing native element/gas readback, not a solution-accuracy or physical net-charge
claim. Native carbon convergence can leave about 3.9e-9 mol of charge in a neutral
carbonated-water trajectory; opening it removes neutral CO2 and preserves that
residual. Comparing only with the remaining ions falsely diagnosed missing
counterions. No inventory is zeroed and no conservation check is relaxed.
Uncompensated doses below this reporting resolution remain unresolved by this
diagnostic; its absence does not certify a complete physical mixture. The separate
bare H/O refusal below has no dose floor. Balanced stocks and balanced explicit
ion components do not acquire this limitation. Typed surfaces,
exchangers and electrodes retain their own reviewed charge models; the generic
bulk diagnostic does not infer their interfacial countercharge.

The solvent-only fast path has a narrower refusal. Water plus explicit acid/base
coordinates cannot be described as plain-water autoionisation. It retains its
inventory, clears solution readings and emits
`not-modeled.bare-acid-base-basis`. This includes equal bare H+/OH- doses: neutral
pH alone would not establish correct water conversion or reaction heat. Later
addition of actual counterions can make a supported analytical problem.
When `solution` is absent, zeroed `free_proton` and `free_hydroxide` fields are
unavailable-state sentinels, not measurements that no free ions exist.

For physical experiments, supply complete reagents such as HCl, NaOH, NH4Cl or
Na2CO3, or explicitly supply their actual charge-balanced components. Do not add
arbitrary spectator ions to a frozen failed protocol. Freeze each corrected
physical protocol separately and retain the original outcome.

The focused controls in
[`tests/charged_component_inputs.rs`](tests/charged_component_inputs.rs) check
withheld solvent-only readings, retained inventory, explicit counterion recovery,
legacy analytical reporting, complete-stock/component equivalence and repeated
equilibration. They are engineering admission controls, not empirical calibration.

Further work remains: a versioned physical ion-input convention, complete
H+/OH- neutralisation water/energy accounting, and condition-complete physical
replays of the original source-expectation gaps. The new warning does not supply
missing chemistry or convert a completed script into a scientific pass.


The [first hosted macOS job](https://github.com/CrispStrobe/kerotakis/actions/runs/38798035314/job/116639494243)
at PR head `9f0ba203247cc15a120b8ef9c620fa6e5dbe5ae1` (tested merge
`ca878ed732e4356182ef8a1b0c877e9b3c69d4ec`) passed the eight original
focused groups and the complete Rust suite. Curiosity acceptance then failed
`aq-063`, `aq-065` and `mat-063`: their frozen `computed` expectations became
`missing`. These are retained first failures, not changed forecasts.
The failures exposed two false gas-path diagnoses
and a real chloralkali bookkeeping defect: the no-metal hydrogen cathode was
counted as zero water-reduction electrons when deriving water and hydroxide.
The correction preserves original question forecasts and coverage classification.
New controls exercise the actual standard stack, retained charge, water use and
both sides of the diagnostic resolution. Six additional core regression groups
cover chloralkali, sulfate water splitting, absent/partial/full metal plating,
and finite-water admission. Both electrode routes reject a dose before mutation
if it would consume the last observable liquid water; dry-cell chemistry is
outside this aqueous model. Corrected-source execution is pending.

Reviewing successful doses in the separate existing-metal half-cell found a
second defect: metal-ion withdrawal and oxygen evolution omitted the remaining
anodic proton inventory. That route now books H+ equal to the metal-deposition
electron share, with hydrogen co-evolution cancelling only its own share.
Successful full-plating and metal-depletion controls include the pre-existing
solid electrode and close hydrogen, oxygen and formal charge before any native
aqueous tail. A pH adjustment is not a substitute for those missing atoms.

The carbonated-water residual remains a separate element/readback accuracy
gap. Its diagnostic allowance does not reconcile dissolved carbon with retained
gas, and does not establish finite-gas numerical or empirical accuracy.

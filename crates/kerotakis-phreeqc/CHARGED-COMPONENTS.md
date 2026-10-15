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
and allows rounding relative to total charged inventory. Balanced stocks and
balanced explicit ion components do not acquire this limitation. Typed surfaces,
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

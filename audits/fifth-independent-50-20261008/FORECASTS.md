# Fifth independently authored fifty — frozen expectations

Source: `7bc43e089753483710507133ccc8377591bb3490`. Forecast SHA-256: `4041d15bc70b108437bf6df909cbbf892f79d737b6ef6faae7f99bbde21a15f5`.

Authored from general expectations and Kero verb/argument syntax; no prior experiment files or outputs consulted. An introductory README excerpt was viewed for CLI discovery and included existing examples; those examples were not used as this batch design. This is a new independently forecast batch, not an assertion of experimental blinding. No new scripts executed before freeze.

Record pass, unmet expectation, explicit model refusal, author/syntax error, harness failure and timeout separately. A refusal does not satisfy a supported-chemistry expectation. Do not fit bounds to observations. Any adaptation gets a separate receipt; preserve original forecast bytes.

## Expectations

- **F01 — An aliquot returns home:** A supported out-and-back water transfer conserves the original donor inventory.
- **F02 — Two quarter withdrawals:** Two successive quarter withdrawals leave the same water as one 7/16 withdrawal.
- **F03 — Three-way split and reunion:** Water can travel through two receivers and return without loss.
- **F04 — Millilitres versus litres:** Equivalent water volume units produce equivalent mole inventory.
- **F05 — Dose partitioning:** Four independent 0.125 mol doses equal a single 0.5 mol water dose.
- **F06 — Zero aliquot:** A zero-fraction decant transfers no water.
- **F07 — Exhaust the donor:** A complete water decant empties the donor and credits one mole to the receiver.
- **F08 — Filter a homogeneous liquid:** Filtering pure water transfers the liquid without creating or losing water.
- **F09 — Stirring is not matter creation:** Stirring homogeneous water preserves its molecular inventory.
- **F10 — Partition the clock:** Splitting a wait into two pieces preserves equilibrium water inventory.
- **F11 — Larger thermal ballast:** The same 1 kJ heats 50 mL of water more than 200 mL, below phase transitions.
- **F12 — Energy additivity:** Two 0.5 kJ additions and one 1 kJ addition reach the same single-phase temperature.
- **F13 — Thermal round trip:** Below phase transitions, adding and removing 0.5 kJ restores the initial temperature.
- **F14 — Mix hot and cold portions:** A receiver mixing equal portions of hot and cold water ends between their temperatures.
- **F15 — Molar heat capacity contrast:** Equal-mole water warms more than ethanol under the same small heat input.
- **F16 — Zero heat request:** Zero added energy preserves temperature and water inventory.
- **F17 — Heat a rigid headspace:** Heating a sealed nitrogen/water system increases its temperature and pressure.
- **F18 — Give gas more room:** A fixed nitrogen dose has lower pressure in a larger rigid headspace.
- **F19 — Dilute a strong acid:** Tenfold dilution raises dilute HCl pH by approximately one unit.
- **F20 — Dilute a strong base:** Tenfold dilution lowers dilute NaOH pH by approximately one unit.
- **F21 — Neutralization equivalence:** Equal dilute strong acid/base doses yield a nearly neutral solution.
- **F22 — Acid beyond equivalence:** A two-to-one strong acid/base dose ratio leaves an acidic solution.
- **F23 — Base beyond equivalence:** A two-to-one strong base/acid dose ratio leaves a basic solution.
- **F24 — Neutralization order:** Reversing dilute HCl/NaOH addition order reaches equivalent final pH.
- **F25 — Weak versus strong acid:** An equal analytical dose of acetic acid has higher pH than HCl.
- **F26 — Buffer resists acid:** A small HCl challenge lowers acetate-buffer pH by less than 0.2 units.
- **F27 — Buffer resists base:** A small NaOH challenge raises acetate-buffer pH by less than 0.2 units.
- **F28 — Dilute both buffer partners:** Tenfold dilution of both buffer components changes pH by less than 0.25 units.
- **F29 — A neutral salt is not a strong acid or base:** Moderate NaCl in water remains broadly neutral, allowing an air boundary.
- **F30 — Increase strong acid concentration:** A tenfold analytical HCl increase lowers pH by approximately one.
- **F31 — Increase strong base concentration:** A tenfold analytical NaOH increase raises pH by approximately one.
- **F32 — An aliquot retains intensive pH:** A homogeneous acid aliquot has the same pH as its remaining donor.
- **F33 — Retained CO2 changes pressure:** A sealed finite CO2 dose raises pressure compared with the same sealed water blank.
- **F34 — CO2 acidifies water:** A finite CO2 dose lowers water pH compared with an otherwise identical blank.
- **F35 — Generate gas inside a seal:** Acid plus bicarbonate raises sealed pressure compared with acid alone.
- **F36 — Open and closed gas generation:** A closed bicarbonate reaction retains greater pressure than an open one.
- **F37 — Opening releases excess pressure:** Opening a pressurized nitrogen vessel returns it to ambient pressure.
- **F38 — Rigid headspace preserves dose dependence:** A larger nitrogen dose raises rigid-headspace pressure.
- **F39 — Volatility enriches the first cut:** The early water/ethanol condensate is richer in ethanol than the 0.2 mole-fraction feed.
- **F40 — Distillation closes both species:** A supported binary cut conserves total water and ethanol separately.
- **F41 — Analytic pure-water cut:** A 20 percent pure-water cut moves 0.2 mol and leaves 0.8 mol.
- **F42 — More latent energy makes more condensate:** Below exhaustion, an 8 kJ pure-water cut collects more than a 4 kJ cut.
- **F43 — Compose pure cuts:** Two 10 percent residual cuts collect the same pure water as a single 19 percent cut.
- **F44 — A filter separates insoluble solid:** Filtering a calcium-carbonate slurry leaves more explicit solid in the donor than receiver.
- **F45 — Drain the denser liquid layer:** Water/hexane drainage selects the lower aqueous layer rather than the upper hexane inventory.
- **F46 — Nonvolatile sugar stays in the pot:** A small sucrose addition stays behind when a supported water cut distils.
- **F47 — Evaporation removes the requested solvent fraction:** Half evaporation of pure water leaves half its starting molecular inventory.
- **F48 — A dry salt cannot supply liquid distillate:** Dry nonvolatile salt distillation refuses visibly and preserves the donor state.
- **F49 — Reject a negative transfer fraction:** A negative decant fraction is an explicit CLI failure, rather than a fabricated successful transfer.
- **F50 — Reject an excessive stage count:** A request for 129 ideal stages refuses visibly and preserves pure-water ownership.

## Provisional performance protocol

Use the same source-bound release executable for behavior and timing. Preserve compiler, generated dependency lock, submodules and executable hash. Measure whole CLI processes, including startup and serialization; do not call that solver-only speed. For F19, F26, F34, F39 and F45, warm once and collect seven independent runs per variant, reporting median, range and p95. Timeouts and refused workloads remain labeled. Record process wall time and resource usage separately from build time. Profile selected successful workloads with Callgrind on the hosted runner; instrumented timings are not native performance measurements. This establishes a provisional baseline, not a speedup or cross-host comparison.

No local build or runtime execution is authorized by this harness design; execution is hosted. Later optimizations require output equivalence on these workloads and a new paired timing measurement.

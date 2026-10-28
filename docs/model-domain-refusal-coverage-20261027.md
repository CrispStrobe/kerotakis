# Refuse, not clamp: capped-model coverage (2026-10-27)

Pattern from #857/#759/#786: a model evaluated outside its stated domain
refuses before mutation with a typed, localized reason instead of clamping.

| Model | Site | Before | After | Feeds |
|---|---|---|---|---|
| Brine partial freezing (#857) | `core/src/solve.rs`, `states.rs` `BRINE_MODEL_MIN_K` | refuses (`not-modeled.partial-freezing-boundary`) | unchanged | energy, phase |
| Ion-interaction (Pitzer-style) solvent activity, I <= 20 | `states.rs` `ION_INTERACTION_MAX_IONIC_STRENGTH` | refuses (`not-modeled.activity-range-ion-interaction`) | unchanged | energy, phase |
| Antoine vapour pressure | `kerotakis-thermo/src/vle.rs` `pressure_kpa`, `common_valid_range` | refuses (None / typed still refusal); unchecked form only inside range-checked flash | unchanged | VLE, still cuts |
| CEA open-air route temperature (200-6000 K) | `kerotakis-cea/src/thermal.rs` (was `clamp(200, 6000)`) | **clamped** | **refuses** `not-modeled.cea-temperature-range` (EN/DE/FR), inventory unchanged | energy |
| CEA feed/condensed record enthalpy | `thermal.rs::enthalpy_within_record` (was `clamp(low, high)`) | **clamped to record endpoint** | **refuses** a ranged record only BELOW its range (the reagent persists: true extrapolation), via existing typed `solver-reason.cea.<context>.data-out-of-range`; above the range decomposition/phase change is owned by the solve (calcination, reheating unchanged) | energy (h_before) |
| Luminol glow Arrhenius ratio | `core/src/chemiluminescence.rs` `clamp(273.15, 333.15)` | clamped | unchanged: the 0-60 C bound is editorial, not a cited domain (an ice bath at 273.1 K is a lesson); deferred | display only |
| Debye-Hueckel conductivity (0.1 / 2.0 mol/kgw) | `core/src/conductivity.rs` | extrapolates, flagged; bench warns "ABOVE ... extrapolation" | unchanged (disclosed, display only) | display |

## Deferred (listed, not changed)

* CEA gas-record polynomials: `kerotakis-cea/src/nasa9.rs::Species::interval_for`
  evaluates the nearest interval outside the record (used throughout the Gibbs
  solve); changing it alters every flame calculation. Degenerate point records
  (assigned enthalpy at one reference temperature) keep that enthalpy.
* NASA/Shomate Cp polynomials (`core/src/heat_capacity.rs`): endpoint Cp is
  held deliberately for heat ledgers; refusing would change every heating
  step. Needs a disclosure flag first.
* UNIFAC / `hmix`: no published temperature domain is declared for the
  parameter set (UNIFAC refuses only nonfinite/nonpositive T; `hmix` uses a
  fixed T_REF by design). Refusing would require cited ranges: not invented.
* PHREEQC Debye-Hueckel databases: routing by ionic strength, no explicit
  per-database I-max is declared; needs cited limits.
* Henry's law van 't Hoff (`core/src/properties.rs::henry_at_t`): no T range in
  the data.
* Yeast/fermentation tau clamps (`kinetics.rs`, `fermentation.rs`): editorial
  teaching bounds, not fitted domains.
* CEA route temperature below 200 K is unreachable through `applies`
  (kinetic threshold), so only the upper bound is exercised.

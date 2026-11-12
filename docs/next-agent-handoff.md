# Fresh-agent handoff — 2026-11-12

Read [STATUS](STATUS.md), [the ordered agenda](improvement-agenda-20261014.md), [PR policy](PR-WORKFLOW.md) and [the systematic-lanes index](systematic-lanes-20261027.md). The ignored operating handoff holds private inputs (API key location, runner script, raw-output directories); never publish those details.

## 0. State and open tasks after round 4 (2026-11-12)

State: production `731931f0` until round 4 deploys; qualification **402/468** (see [STATUS](STATUS.md)).
Round 4 (`integrate/round4-20261111`) is merged to main after hosted CPU validation, deployed once,
and the affected originals (KI-011, SP-011, EN-001, PO-009, TH-004, AB-035, GS-013, plus any whose
output changed) re-run once, then strict and adversarial review and a new catalog version.

Every lane below is family-wide: one generic mechanism, a coverage table (value + primary citation, or
no-data with the attempts listed), typed EN/DE/FR refusals and tests. Sources: primary, or "as quoted by"
Wikipedia/NIST with the primary named; NIST SRD single facts only; the MPI-Mainz UV/VIS atlas is
CC BY 4.0 (Zenodo doi:10.5281/zenodo.6951) and may be used with attribution.

1. **Volatile acid–base gases beyond CO2.** Assessed 2026-11-12, not implemented, because each gas lacks a
   prerequisite rather than a rule:
   - H2S: no reduced sulfur ever reaches solution. `derived.rs` has no S(-2) group or residue rule, the
     registry has no sulfide salt (no Na2S, FeS, ZnS) and `hydrogen_sulfide` has no dissolution path.
     A naive "S without O is sulfide" rule would mis-book elemental sulfur and pyrite. Needed: a
     reduced-sulfur family (explicit H2S/HS-/S2- and sulfide-salt groups, registry salts, H2S Henry
     constant), then an `ATMOSPHERIC` H2S(g) entry offered only where `S(-2)` is present (never to
     sulfate, `S(6)`), log p -12 (room air holds none at bench resolution; registry key
     `hydrogen_sulfide`). Pitzer names the phase `H2Sg(g)`; the gas-coverage importer must cover it.
   - SO2: SO2(g) is defined in none of the routed databases; needs a sourced gas-phase slice (as
     `aqueous_gases.rs` does for HBr: Sander Henry constant with standard-state conversion).
   - NH3: far too soluble to bubble, so no fizz rule applies; its open-vessel escape is lane 3's rate
     (`volatility.rs` is deliberately silent over an open vessel).
   The fizz test itself can then generalise from carbonic acid's pK1 to "the molecular gas is the
   dominant dissolved form" read from the native speciation (H2S vs HS-), keeping CO2's behaviour.
2. **Headspace gas colour: done 2026-11-12** (`gas_colour.rs`, `docs/gas-colour-coverage-20261112.md`):
   Cl2, Br2, I2 vapour, NO2, O3 from the MPI-Mainz atlas (CC BY 4.0); SO2, H2S, NH3, CO2, O2 cited
   colourless. Open: NO2 at 730-750 nm (outside the 16 bands), N2O4 and NO2 are not registry species.
3. **Open-vessel seeping over `wait`.** The two-film lane was withdrawn (it removed fizz). Re-add a sourced
   k_L seeping rate *under* the zero-time/fizz rule; carbonate, foam and codex tests must stay green.
4. **Unsourced data.** FeCl2+ absorptivity (figure-only), starch–iodine blue value, maltose/maltotriose
   split, PVC and unsintered PTFE latent heat, salt/oxide boiling and decomposition points (NaOH, CaCl2,
   BaCl2, KCl, CaO, MgO, SiO2, Pb(NO3)2, AgNO3; NBS Circular 500 page images).
5. **Typical-experiment sweep.** Run classroom reactions through `kero run` (neutralisation heat, hot/cold
   packs, metals in acid, displacement, AgCl/BaSO4/PbI2, limewater, vinegar + soda, chalk + acid,
   permanganate colour, ethanol burning, boiling) and fix what is off, generically.
6. **Typical-experiment sweep, findings 2026-11-12** (15 classroom reactions through `kero run`, Kaggle cx):
   neutralisation (+5.3 K, +6.6 K, pH 6.75), hot pack (+19.9 K), cold pack (-6.2 K), AgCl/BaSO4/PbI2,
   limewater, chalk + acid, permanganate (A525 = 0.334) and Zn/Mg + HCl H2 yields are right. Gaps:
   Round 5 fixed (b). (a) was a misreading: the displacement solver already charges metal + acid and
   displacement heat from formation enthalpies (`metal-acid-race` 25.0 -> 36.2 C); the aqueous
   tail's "no reviewed enthalpy" note concerns its own balance, and a second pricing route
   double-counted (Kaggle cz) and was withdrawn. Original text:
   (a) metal + acid and metal displacement reaction heats are unpriced (Zn, Mg, Fe + Cu2+: about
   +5 to +10 K missing) because a solid metal has no path into the PHREEQC master-species enthalpy
   basis (`kerotakis-phreeqc/src/enthalpy.rs`); price M(s) via the cited ΔfH of M^z+(aq) (NBS
   Circular 500, public domain) with H2 and electrons on the same basis. (b) `ignite` on a liquid
   fuel in an open dish boils it off: `ignite` sets the vessel to ignition temperature and the phase
   route (before CEA and the curated combustion table) vents the fuel vapour, so nothing burns; a
   fuel at or above its autoignition row with air available must burn, not boil, and liquid fuels
   need curated rows (ΔcH from Circular 500 formation enthalpies) for an open dish. (c) A 1 h `wait`
   relaxes an open bicarbonate to room air at once (pH 9.7); that is lane 3's seeping rate.
   (d) The zero-time material addition in aq-023 is not reproducible run to run beyond ~1e-4
   (particle molality); find the source of the non-determinism.
7. TR-006's verbatim script asks a 20 mL feed for 60 mL and is correctly refused; leave it.

Earlier handoffs (2026-10-27 successor lanes 1–8) moved to [HISTORY.md](../HISTORY.md#archived-status-and-handoff-entries-moved-2026-11-12).

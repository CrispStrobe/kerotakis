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

1. **Volatile acid–base gases beyond CO2.** `derived::ATMOSPHERIC` offers an open beaker only CO2(g), and
   the fizz rule (`acid_carbonate_effervesces`, bench.rs) only knows carbonic acid. Generalise both to
   a table of volatile conjugate pairs with their own pKa and room-air level: SO2/HSO3- (sulfite + acid),
   H2S/HS- (sulfide + acid), NH3/NH4+ (ammonium + base, fizz when pH > pKa). Acceptance: acid on Na2SO3
   or Na2S releases the gas in the step; neutral solutions keep it until time passes.
   Findings 2026-11-12: SO2(g) is defined in none of the routed databases (phreeqc, wateq4f, minteq.v4,
   pitzer), so SO2 needs a sourced gas-phase slice (as `aqueous_gases.rs` does for HBr) or stays with
   `sulfur_iv`/`volatility`; H2S(g) exists natively (pitzer: `H2Sg(g)`), but the registry has no Na2S
   and keys the gas as `hydrogen_sulfide`, so gas events must map to that key; ammonia has its own
   Henry-law pass (`volatility.rs`), deliberately silent over an open vessel. A generic fizz test can
   read dominance of the molecular form from the native speciation instead of a per-gas pKa table.
2. **Headspace gas colour from cross sections.** The registry has gas entries (Cl2, SO2, H2S) without a
   spectrum, and no Br2 vapour or NO2. Build one mechanism mapping cited gas-phase cross sections
   (atlas) to the 16-band colour of a headspace at its partial pressure and path; cover Cl2, Br2, NO2,
   I2 vapour, O3; refuse typed where no cross section is cited.
3. **Open-vessel seeping over `wait`.** The two-film lane was withdrawn (it removed fizz). Re-add a sourced
   k_L seeping rate *under* the zero-time/fizz rule; carbonate, foam and codex tests must stay green.
4. **Unsourced data.** FeCl2+ absorptivity (figure-only), starch–iodine blue value, maltose/maltotriose
   split, PVC and unsintered PTFE latent heat, salt/oxide boiling and decomposition points (NaOH, CaCl2,
   BaCl2, KCl, CaO, MgO, SiO2, Pb(NO3)2, AgNO3; NBS Circular 500 page images).
5. **Typical-experiment sweep.** Run classroom reactions through `kero run` (neutralisation heat, hot/cold
   packs, metals in acid, displacement, AgCl/BaSO4/PbI2, limewater, vinegar + soda, chalk + acid,
   permanganate colour, ethanol burning, boiling) and fix what is off, generically.
6. TR-006's verbatim script asks a 20 mL feed for 60 mL and is correctly refused; leave it.

Earlier handoffs (2026-10-27 successor lanes 1–8) moved to [HISTORY.md](../HISTORY.md#archived-status-and-handoff-entries-moved-2026-11-12).

# Kerotakis current state — 2026-11-12

Older checkpoints (2026-10-27/28 delivery, release acceptance, snapshot cleanup, earlier coverage
counts) moved to [HISTORY.md](../HISTORY.md#archived-status-and-handoff-entries-moved-2026-11-12).

## Production and qualification

Production is `731931f090e3bc41059d0aef98ae64ca0223138b`. Qualification is **402/468** unique
originals (406 entries; catalog `audits/working-experiments-20261111-reviewed-production-round3-29.json`
on the audit branch). The goal of ≥ 401 is met; further qualifications continue the same way.

## Round 4 (integration branch `integrate/round4-20261111`, merging to main 2026-11-12)

Validated on a hosted CPU runner, then merged once, deployed once, and the affected originals re-run once:
- S(IV) couples to iodine, peroxide and the other sourced oxidants (Yiin & Margerum; Hoffmann & Calvert;
  HOBr/OBr-; MnO4-/Cr(VI)/Br2 acid-limited); KI-011.
- Iodine in hexane is labelled a non-aqueous solution and counted by `inspect`; hexane/water K_D = 26.0
  from Hildebrand & Scott saturation data (the exam value 85 is gone); SP-011.
- Polymer and wax latent heat ΔHf = Xc·ΔHf(100 %) (Wunderlich via TA TN048); PO-009, TH-004.
- Englyst RDS/SDS/RS starch digestion across seven starch sources and states; EN-001.
- 27 NBS Circular 500 and 9 WebBook-quoted transition points; NaHSO4, H2O2 and NH3-solution density corrected.
- UV-vis cited molar absorptivities: I2, I3-, MnO4- (3340), MnO4^2-, Cr2O7^2-/HCrO4-, CrO4^2- (Haupt 1952,
  NBS RP2331), Co(H2O)6^2+ (Davis & Gibson, NBS MP114), Cl2(aq) (Fergusson 1936 via the MPI-Mainz atlas,
  CC BY 4.0), HOCl/OCl-, Br2/Br3-, FeSCN2+, Cu(NH3)4^2+, Ni2+. Only FeCl2+ is still refused.
- Iodide-catalysed peroxide decomposition uses the measured k0 = 0.0115 L/mol/s (15x slower than before).
- **Zero-time additions and the fizz rule.** Any addition (or `wait 0s`) takes no time, so an open beaker
  releases only gas above the bubble threshold; when the zero-time answer shows carbonic acid dominant
  (pH < pK1(T), Harned & Davis 1943) with CO2 above room air, the acid has made CO2 and the step fizzes to
  room air. Baking soda in water keeps its carbon until time passes (AB-035, GS-013). The two-film
  seeping lane was withdrawn: a 1 atm threshold alone stops acid on carbonate from fizzing.

## Open work (detail and acceptance in the [handoff](next-agent-handoff.md))

1. Volatile acid–base gases beyond CO2: open-boundary release and the fizz rule for SO2 (sulfite + acid),
   H2S (sulfide + acid) and NH3 (ammonium + base).
2. Headspace gas colour from cited cross sections (MPI-Mainz atlas, CC BY 4.0): Cl2, Br2, NO2, I2 vapour.
3. Open-vessel seeping as a rate over `wait` (sourced k_L), on top of the zero-time/fizz rule.
4. Remaining unsourced data: FeCl2+ absorptivity, starch–iodine blue value, maltose/maltotriose split,
   PVC latent heat, salt/oxide boiling and decomposition points (NaOH, CaCl2, BaCl2, Pb(NO3)2, …).
5. Typical-experiment sweep through the CLI (15 classroom reactions) and fixes for anything off.
6. Re-run only originals whose blocker a merged lane removed, once per deployed sha; strict and
   adversarial review; new catalog version. TR-006 stays correctly refused.

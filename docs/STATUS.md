# Kerotakis current state — 2026-11-12

Older checkpoints (2026-10-27/28 delivery, release acceptance, snapshot cleanup, earlier coverage
counts) moved to [HISTORY.md](../HISTORY.md#archived-status-and-handoff-entries-moved-2026-11-12).

## Production and qualification

Production is `731931f090e3bc41059d0aef98ae64ca0223138b`. Qualification is **402/468** unique
originals (406 entries; catalog `audits/working-experiments-20261111-reviewed-production-round3-29.json`
on the audit branch). The goal of ≥ 401 is met; further qualifications continue the same way.

## Rounds 4 and 5 — PR881 pending acceptance

[PR881](https://github.com/CrispStrobe/kerotakis/pull/881) is a draft while source review and
acceptance remain incomplete. It has not merged or deployed; the production identity above remains
current. Its first failing head is `bb168622`: six starch display names lack German translations,
the bicarbonate demo lacks step prose, and carbonate/gas/selectivity fleet contracts fail. Repair
work preserves those first failures and distinguishes physical source defects from invalid oracles.
Normal exact-head auto-merge and one bound deployment follow acceptance. Version/tag publication
and original qualifications are not yet earned.

[Authenticated focused source results](../audits/carbonate-focused-source-20261112/README.md)
show72 internal cases/92 declared checks passing on `32f3aed5`, plus core/Clippy/type checks and
eight identical-process aq-023 readbacks. Two native controls and two translator-template checks
failed on that producer; later source-bound runs passed their corrections. This adds no original
qualification.

The subsequent hosted workspace run on `583a681d` exposed additional carbonate ownership,
locale inventory and Clippy failures. Candidate `e862b601` addresses the reported calcite and
molecular-reagent ownership gaps and the two inventory/lint issues. Original heat and chalk
assertions remain in place. Three atmospheric-equilibrium contracts were independently reviewed
against the finite-time gas model, with their originals and first failures preserved. No release
acceptance is claimed for this candidate.

The `e862b601` workspace run subsequently passed the original chalk, pouring-order heat,
gas-event and transfer controls and Clippy. Its remaining failures and three independently
audited readbacks are preserved in [the native carbonate contract correction](../audits/carbonate-native-contract-correction-20261112/README.md).
Concentrated mineral and gas yields remain model-domain holds. The successor controls and the
full fleet were subsequently executed on the candidate described below.

Candidate `7bd82c80` passed its [hosted workspace, frontend and 702-case own-fleet checks](../audits/source-acceptance-7bd-20261112/README.md).
Its [macOS CI first failure](../audits/macos-clippy-1100-first-failure-20261112/README.md)
is a Clippy 1.100 iterator lint in solver-owner discovery; the successor must validate the
same solver order and ownership on the newer toolchain before this PR can be accepted.
The same CI run also exposed a [browser foam overflow failure](../audits/browser-foam-clock-first-failure-20261112/README.md). The successor advances an explicit observation interval appropriate to the revised iodide rate while retaining the browser assertions. The repaired overflow checks passed in Chromium on `66d0d970`; the same job then identified a [carbonate lesson snapshot change](../audits/gui003-carbonate-model-adjudication-20261112/README.md). The original snapshot, browser readback and independent native evidence are preserved; a real-browser run on the reviewed successor is still required. The earlier preflight also found [implicit Python text encodings](../audits/python-encoding-first-failure-20261112/README.md); the successor fixes them and still requires the complete preflight. Both native jobs also found three [outdated carbonate codex claims](../audits/codex-carbonate-claims-adjudication-20261112/README.md). The original forecasts and source-bound readbacks are preserved; the revised English, German and French lessons pass a bounded whole-codex check. The following CI run found the corresponding [codex export snapshot drift](../audits/codex-export-a4-first-failure-20261112/README.md); its reviewed update retains the original and first actual exports. Exact-successor CI remains required. The next Linux native run exposed [curiosity coverage failures](../audits/curiosity-cbab-first-failure-20261113/README.md) affecting finite CO2 doses and high-temperature silica, plus three classification changes; the original baseline and refusal controls remain in force pending source-level review.

Round 4:
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
- **Carbonate boundaries under repair.** Instantaneous operations now share the zero-time boundary;
  signed finite-time CO2 film transfer has a separate owner. The old bulk-pH/atmospheric-sink fizz
  fallback is replaced by a bounded curated parcel with an explicit EN/DE/FR modelling limit.
  [Design, first failures and pending acceptance](carbonate-boundaries-20261112.md) distinguish
  transport, new reaction gas and old stored supersaturation. The versioned independent NaHCO3
  law preserves the original failed ordering and all original scripts.

Round 5 (from the typical-experiment sweep): `ignite` burns liquid ethanol, methanol, propanone and
n-hexane in an open dish (curated rows from Circular 500 / NIST-quoted formation enthalpies) instead
of boiling them off; headspace gas colour (Cl2, Br2, I2, NO2, O3) from the MPI-Mainz atlas. Metal +
acid and displacement heats were already charged by the displacement solver from formation
enthalpies. A second route was withdrawn after double counting: zinc in copper sulfate warmed
51.8 K instead of about 25 K, and the metal-acid lesson lost its 25.0 to 36.2 °C reading.

## Open work (detail and acceptance in the [handoff](next-agent-handoff.md))

1. Volatile acid–base gases beyond CO2: H2S needs a reduced-sulfur family first, SO2 a sourced gas-phase
   slice; NH3 needs no fizz rule (see the handoff).
2. Headspace gas colour: done (Cl2, Br2, I2 vapour, NO2, O3 cited; `gas_colour.rs`).
3. Complete hosted acceptance of the new finite-time CO2 owner and bounded endpoint; broader
   gas-transfer and nucleation domains remain scoped separately.
4. Remaining unsourced data: FeCl2+ absorptivity, starch–iodine blue value, maltose/maltotriose split,
   PVC latent heat, salt/oxide boiling and decomposition points (NaOH, CaCl2, BaCl2, Pb(NO3)2, …).
5. Typical-experiment sweep through the CLI (15 classroom reactions) and fixes for anything off.
6. Re-run only originals whose blocker a merged lane removed, once per deployed sha; strict and
   adversarial review; new catalog version. TR-006 stays correctly refused.

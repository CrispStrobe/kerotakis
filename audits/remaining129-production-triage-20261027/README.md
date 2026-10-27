# Remaining-129 originals: one production run each and blocker triage (2026-10-27)

All 129 unqualified originals of the fixed 468 target were sent **once, unchanged** to the production REST API
on accepted release `a7640a07` (native `5427d0ee`, lock `568f0a32`). No adapter, retry or protocol change was
used. [production-run-manifest.json](production-run-manifest.json) binds each original script hash to the API's
echoed hash, status and response-body hash: **126 completed, 3 refused** (CO-005 counterbalance, TR-003/TR-007
non-uniform transport cells). Clean exits are not passes; `physical_verdict` stays `not_assigned`.

[triage.json](triage.json) classifies every case against its **original** forecast and criteria, using the current
output as primary evidence. It was produced by read-only model-assisted review in four disjoint parts and
spot-checked by the parent against raw output (AB-017, TH-007, TH-009 and TH-015 findings below were confirmed). It is a
prioritisation map, **not criterion qualification**: no catalog entry is added and the count stays **339/468**.

## Results

| Current outcome | Cases |
| --- | ---: |
| Likely pass, needs formal criterion review | 32 |
| Partial | 50 |
| Fails forecast | 34 |
| Refused | 9 |
| Unsupported | 4 |

| Primary blocker | Cases |
| --- | ---: |
| None (formal review only) | 30 |
| Data gap | 32 |
| Model gap | 21 |
| Instrument/observable | 15 |
| Ownership/ledger defect | 15 |
| Forecast identifiability | 7 |
| Protocol/syntax | 7 |
| Out of scope | 2 |

**Formal-review queue (30):** AD-001..004, BD-003, CO-006, EN-010, FE-001/002/004/005/007/008, MI-003/004,
PO-003/004, SU-001/004, TH-006 (batch3), TR-001/002/004/005/006/008, AB-018, RX-007, SP-004, TH-010 (first120).
Even if all of them qualify, the count reaches 369; at least 32 more must come from source repairs.

## Shared source lanes (approximate, overlapping keyword grouping of non-ready cases)

1. **Iodide/iodine/thiosulfate ownership (~19):** KI is not mapped to dissolved ions and I2 + S2O3²⁻ never
   reacts (EC-008, KI-011, PQ-004, QA-007/008, SE-001, KN-010/011, PR-017, RX-004/005/006, SP-011, AN-003 …).
   The largest single generic lane.
2. **Reaction/solution heat ownership (~19, overlapping):** parent-confirmed: CaO slaking *cools* 0.32 K
   (TH-009); adding KNO₃ at 90 °C warms 363 → 372 K although its dissolution is endothermic (TH-007); mixing
   HCl and NaOH vessels yields their average temperature without neutralisation heat (TH-015). To review:
   TH-010 reports 111.7 J sensible heat for a 100 J pulse.
3. **Optics/absorbance (~18):** starch–iodine, copper complexes and pigments lack chromophore data; now
   refused rather than reading 0.
4. **Missing registered solids (~7):** Witherite, Ag₂CO₃, Fe(III) hydroxide routing (PR-005 vs AB-018).
5. Chromatography retention (6), volume/conductivity readouts (6), partition data (5), milk matrix (4),
   transport-cell volume admission (2).

## Confirmed defects or regressions to fix at source

- **AB-017 regression:** `phreeqc-aqueous` now fails because NH3(g) is undefined in routed minteq.v4; sodium
  acetate also reports no wired dissolution solver. The earlier review recorded pH 6.875.
- **TH-009:** CaO + water precipitates Ca(OH)₂ while temperature falls 298.15 → 297.83 K.
- Reported for follow-up (unconfirmed by parent): SO-005 NaCl in ethanol recorded as liquid with a false freezing
  event; AN-002 molten salts liquid at 298 K; FE-003 open-vessel CO₂ `gas_absorbed`; MI-007 langite/brochantite
  water count; PO-008 PVA dissolved yet solid; FO-001 baking powder lacks its acid component.

## Would a new engine help?

Engine candidates named by the triage: none needed for 72 cases, own code 24, curated property data
(thermo/chemicals-style, licence-checked) 16, existing PHREEQC databases 14, **FeOs 2** (salts in ethanol),
**teqp/CoolProp 1** (pressure-aware boiling). The remaining originals are dominated by ownership, data and
observable gaps inside existing solvers, so integrating a new equation-of-state or kinetics engine is not the
next lever for the 401 goal. FeOs remains a reasonable later option for the mixed-solvent electrolyte lane.

Raw responses are retained privately; the hashes above identify them. No production state was changed.

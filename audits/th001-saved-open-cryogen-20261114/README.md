# TH001: unchanged open-boundary cold-addition comparison

The saved original meets **3/3 original criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`, as an open-boundary cooling/phase/vent and explicit coverage comparison. **No retained-CO2 acidification or bubble-to-liquid transfer kinetics is qualified.** The original instruction is conditional: any CO2 retained in water can participate in carbonate chemistry; equal cryogen masses do not imply equal cooling or dissolved-gas amounts.

The actual events establish the input-to-gas identities and their complete exit:

| Arm | Input | Phase event | Explicit gas exit |
| --- | --- | --- | --- |
| v1 | 0.1 g dry ice | sublimation, 0.0022722624917630484 mol | same amount CO2 |
| v2 | 0.1 g liquid nitrogen | boiling, 0.003569643749553795 mol | same amount N2 |

Both vessels remain open. No cryogen, dissolved carbon/nitrogen or headspace gas remains, and no pending CO2 transfer is recorded. Independent H/O/C/N/charge accounting adds every original input and subtracts each explicit gas boundary term; all checkpoint residuals are zero within the unchanged `1e-10 mol/eq` bound. The source registry assigns the input aliases the same atom/charge compositions as their reported output gases. Solvent water is retained without loss.

The immediate thermometers report **24.540430143916865 °C** and **24.840472082524002 °C**. Reported pH values before/after the original 120 s wait are **7.005003068057578 → 7.003833388698753** and **7.000019390544955 → 6.99961450222844**. Every characterised solution explicitly has `solvent_only` scope, with matched H+/OH- molalities. These are solvent-relation outputs at the reported temperatures, not evidence of retained-CO2 acid chemistry. The immediate phase-to-open-vent model does not establish gas residence, real cooling efficiency, subsequent gas warming, dissolution kinetics or handling behaviour.

Nine controls reject missing carbon/nitrogen vent terms, wrong gas identity, wrong phase amount, invented retained carbon, lost water, invented acidic pH, removal of the existing cryogen warning and injected solver failure. The raw hazard notice and all admission/boundary behaviour are preserved. Original script, forecast, criteria and limitations remain unchanged.

`binding.json` authenticates original/output, source/tree, genuine producer lock and successful locked-release build receipt. The historical producer captured **no executable checksum**; the limit remains explicit, with no later binary/hash substitution. Other experiments' producer failures remain separate. No new build or experimental replay was performed.

```sh
python audits/th001-saved-open-cryogen-20261114/review_original.py audits/th001-saved-open-cryogen-20261114
```

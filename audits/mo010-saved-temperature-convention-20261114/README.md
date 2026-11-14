# MO010: unchanged salt-inventory and conductivity-convention comparison

The saved original meets **2/2 original criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`. The qualification is the explicitly requested inventory/convention review. **It does not qualify an empirical conductivity-versus-temperature law.** The original limitation explicitly says fixed reference mobilities make a flat signal a coverage limitation, not evidence that real conductivity is temperature independent.

C1: every snapshot retains **0.005 mol Na+**, **0.005 mol Cl-**, and **99.7 g water** after the original salt addition. The reported liquid volume remains **0.1 L** throughout the 10 kJ heating and cooling steps. Independent H/O/Na/Cl/charge accounting from the actual additions gives zero residual at every checkpoint, within the unchanged `1e-10 mol/eq` bound. There is no unresolved material, gas crossing or solver failure. The three open-vessel headspace-volume-meter refusals are retained; actual liquid volumes are taken from the saved vessel records, with no successful headspace reading invented.

C2: thermometers read **19.9534685448 → 43.9353618157 → 19.9534685447 °C**. Conductivities read **5678.763936808268 → 5678.82842953843 → 5678.763936808268 µS/cm**. The pinned `specific_conductance` function uses fixed **25 °C limiting ionic mobilities** and a fixed ionic-strength correction. Its inputs contain solved speciation and ionic strength, with no mobility-temperature term or empirical automatic temperature-compensation step. It uses the stated dilute molality-as-molarity approximation; each ionic strength is about 0.05015 mol/kgw and below the source's 0.1 dilute-limit threshold.

An independent reader reconstructs all three readings from those source coefficients and each saved speciation record. The small **0.064492730162 µS/cm** heating difference arises from the solved H+/OH- and ionic-strength changes. It supplies no measured ion-mobility temperature coefficient. The restored final signal and constant salt/water distinguish this convention limitation from solute consumption or evaporation.

Eight mutations reject lost salt, lost water, changed liquid volume, an invented mobility-driven signal increase, a mismatched thermometer, a hidden volume refusal, an invented gas crossing and a solver failure. Original script, forecast, criterion wording and raw observations remain unchanged.

`binding.json` authenticates script/output, historical source/tree, genuine producer lock and locked release-build receipt. `source-conductivity.rs` is the exact file from that source, not a later implementation; `source-properties.json` extracts its required registry compositions and water properties. The historical producer did not record an executable checksum: it remains explicitly null, with no later hash substitution. Other originals' aggregate producer failures remain separate from this criterion review.

```sh
python audits/mo010-saved-temperature-convention-20261114/review_original.py audits/mo010-saved-temperature-convention-20261114
```

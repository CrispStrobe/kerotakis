# LI003: unchanged neutral-pigment and solvent/base blank comparison

The saved original meets **4/4 original criteria** at historical source `4bd93add656c936b0724673c7ea8332cf21e906f`, within its stated pigment/blank and coverage-review scope. **No curcumin acid–base spectral shift, mixed-solvent dissolution validation or optical calibration is qualified.** The unchanged original explicitly requires interpreting the pigment with measured pH and coverage rather than assuming a fully parameterised spectrum after a pH shift.

The pigment arm's reported maximum is **1.1 absorbance at 425 nm** before carbonate and **1.1000023941694355 at 425 nm** afterward. The solvent/base blank reports a **zero maximum**, labelled at the instrument's default **405 nm**, at both observations. These are not claimed to be two direct readings at a common wavelength. The pinned instrument reports the maximum of its sampled visible spectrum; its zero maximum establishes zero across those bands, including 425 nm. This supports the unchanged pigment-versus-blank question/control: the blank does not acquire the pigment band after base addition.

Both arms report the same post-carbonate **pH 10.11477227605**. The neutral curcumin pool remains **0.000001 mol** only in the pigment arm; each arm retains **0.08563242093381666 mol ethanol** and **0.00002 mol sodium**. Their final nonpigment inventories are identical. Independent per-vessel H/O/C/Na/charge accounting from all explicit additions closes below **1.8e-16 mol/eq**, within the unchanged `1e-10` bound. No unresolved material, gas crossing or solver failure appears.

The source registry declares a fixed neutral-curcumin visible spectrum with an unestablished legacy absorptivity and no pH or mixed-solvent parameters. Its 425 nm coefficient and the instrument's 1 cm path reconstruct both pigment readings from the saved actual liquid volumes. The tiny change is explained by the model's solvent-volume change; it is not evidence of a new acid–base band or chemical stability. Reported aqueous pH is retained as model output, without claiming mixed-solvent pH calibration. An unchanged neutral pool does not establish long-term alkaline stability or a validated dissolution equilibrium.

Nine controls reject lost pigment, lost ethanol spectator, lost carbonate carbon, lost solvent, an invented blank band, mismatched paired pH, an invented base-induced peak shift, partial final scope and solver failure. The original script, forecast, limitations, criterion wording and raw observations remain unchanged.

`binding.json` authenticates script/output, historical source/tree, actual producer lock and locked-release build receipt. `source-properties.json` extracts used compositions and the complete neutral-curcumin optical declaration from that source; `source-instrument.rs` preserves its actual peak-reporting implementation. The producer recorded **no executable checksum**: the identity limit remains explicit and no later binary hash is substituted. The aggregate producer failures for other originals are not erased or used to qualify this target. No new build or experimental replay was performed.

```sh
python audits/li003-saved-pigment-blank-20261114/review_original.py audits/li003-saved-pigment-blank-20261114
```

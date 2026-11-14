# BD003: unchanged light/dark boundary comparison

The saved original meets **4/4 original criteria** on historical source `4bd93add656c936b0724673c7ea8332cf21e906f`. This qualifies the explicitly designed known-boundary comparison: illumination is stored, while peroxide conversion is the same thermal reaction in the light and dark arms. **No light-dependent peroxide pathway is qualified.** The unchanged forecast itself requires a supported light-dependent route beyond the dark control before claiming photochemistry.

Both irradiated vessels report `photolysis_coupled=false`. After 600 seconds, each peroxide arm consumes `4.8627577126687e-7 mol` H2O2, with the matching half-amount of O2 production. The peroxide-free irradiated water control has no peroxide reaction. Light and dark peroxide pressures are both `101.32336084217407 kPa`; the water control reads `101.32205833455458 kPa`. The pressure difference demonstrates thermal peroxide conversion, not a UV effect.

The independent reader counts every explicit dose and the seal's room-air admission (N2/O2/CO2 fractions 0.7901/0.2095/0.0004, as declared at this source). Hydrogen, oxygen, carbon, nitrogen and charge close at every checkpoint within the unchanged `1e-10 mol/eq` bound; the maximum residual is below `3.1e-16`. Inventories remain sealed, with no unresolved material or solver failure. The reader rejects invented light coupling, missing oxygen/carbon/nitrogen, unequal dark conversion, a missing water control and injected solver failure. Original script, criteria and raw observations remain unchanged.

`binding.json` authenticates the collected script/output, producer build receipt, source/tree and genuine producer lock. The historical producer did not record an executable checksum; that limitation is explicit and no later binary is substituted. The extracted compositions come from that historical source, not the current registry. The parent batch's aggregate failure flag remains preserved; it does not erase or qualify individual scientific observations.

Reproduce the saved-output assessment without rebuilding or replaying:

```sh
python audits/bd003-saved-light-boundary-20261114/review_original.py audits/bd003-saved-light-boundary-20261114 .
```

This is a teaching-model coverage comparison. It supplies no quantum yield, radical concentration, optical geometry or UV safety claim.

# BD001: unchanged salt-insensitive teaching-model boundary

The already-collected original meets **4/4 original criteria** at source `58497904f44ce3f20fd82a8b216784a88c0bb732`, after that source's shared retained-material mass correction. No new build or replay was performed. The original script, forecast and criteria remain unchanged.

Both arms contain **0.2 g** unresolved instant-snow polymer and **30 g water**. The treated arm additionally retains **1 g NaCl**, as `0.01711068904744794 mol` each Na+ and Cl-. Every post-salt scene reports the same **20 g retained water / 100 g per g dry polymer** for both arms. That water is a subset of the existing solvent, not an extra water source. The intentionally designed boundary question is therefore answered: this fixed-capacity teaching model does not couple salinity to retention. **No real-gel salt-insensitivity, calibrated swelling capacity or salinity response is qualified.**

Independent per-vessel H/O/Na/Cl/charge accounting closes with zero residual from explicit additions. The opaque polymer is conserved separately, with exact recipe/version/basis and 0.2 g in every subsequent snapshot; no invented polymer atom formula is used. Scene masses now include it: **30.2 g** versus **31.2 g**. The older `4bd93add` output is preserved separately as `first-4bd.stdout`: its scene mass omits 0.2 g per arm and it is not qualified.

Conductivities are **0.05508761762446267** versus **52722.58979476864 µS/cm**, confirming distinct ionic environments. The saline arm's `measured.conductivity-past-the-dilute-limit` notice remains required; no calibrated concentrated-electrolyte reading is claimed. No solver failure, gas crossing or hidden material loss appears. Seven controls reject the actual first mass failure, lost polymer, lost salt, extra water, an invented salinity response, removed conductivity boundary and solver failure.

`binding.json` authenticates collected source/tree, exact script/output, genuine producer lock and successful locked-release build receipt. The producer's separate FE003 solver failures are not erased by its CLI exits or this target's success. The historical producer recorded **no executable checksum**; it remains explicitly null, with no later hash substitution. The source's 71 focused material/fermentation/mass tests had passed; the original BD001 qualification here rests on its saved observations and independent inventory/criterion review.

```sh
python audits/bd001-saved-mass-boundary-20261114/review_original.py audits/bd001-saved-mass-boundary-20261114
```

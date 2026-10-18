# Saved finite-gas selection preparation

This independent raw-output reader reproduces all10 original gas-case timelines:
62 owned observations with actual pressure, gas-phase moles, temperature,
headspace volume, aqueous diagnostics and explicit solver/coverage limits.
No engine or API execution and no new catalog selection is involved.

Read [metrics](metrics.json), the [verification reader](verify_metrics.py) and
[original criteria and material ledgers](../saved64-first-screen/README.md).
The source is `c44692053a94c59dcf62e7fb67549f84c7d18f83`; its vessel file
and the original archive/scripts/outputs are independently hashed. The reader
uses the source's declared ideal-gas constant,8314.462618Pa L mol^-1 K^-1.
It retains numerical pressure residuals without inventing a fitted pass band.

From a checkout containing the immutable original raw archive, run:

```console
python3 audits/target468-20261016/saved64-gas-preparation/verify_metrics.py
```

Include trapped room air and its carbon; subtract the appropriate finite baseline
when comparing added doses. Nondissolved and aqueous portions are owned matter;
solution diagnostics are not additional matter. Ideal-gas agreement alone proves
neither correct partition chemistry nor salinity, acid-trap or energy behavior.
GS005/007/010 remain held. This prepares the final criterion review; it does not
certify a current source/deployment or empirical calibration.

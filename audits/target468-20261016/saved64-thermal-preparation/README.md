# Saved thermal selection preparation

The [reader](verify_metrics.py) independently reproduces [all16 thermal timelines](metrics.json)
and eight bounded heat calculations plus the hot-copper/cold-water mixing
balance. It authenticates the pinned registry, phase-route table, original
archive, scripts and raw outputs. No simulation/API execution, selection,
empirical calibration or current-source reproduction is claimed.

```console
python3 audits/target468-20261016/saved64-thermal-preparation/verify_metrics.py
```

Integration uses the actual pinned NASA-9/Shomate Cp curves, including their
declared endpoint clamping; legacy constant Cp is used only where no curve is
present. Composite Simpson integration independently reproduces the sensible
heat rather than copying the engine's antiderivative. The water5kJ result changes
by only7.2e-11J when the quadrature step count doubles. This numerical convergence
does not establish property uncertainty or measured accuracy.

- TH001: the water/ethanol5kJ balances close to below3e-9J.
- TH002: actual mixture296.71744434K; water gains1491.94386717J and copper
  loses1491.94386734J, residual−1.72e-7J. This uses variable Cp, not a
  mass-weighted or constant-Cp mixing estimate.
- TH003: the actual liquid/ice fraction changes account for−10kJ then+5kJ.
- TH011: naphthalene stores339.35314494J sensible and660.64685506J latent.
- TH012: acetic acid loses696.96263239J sensible and1303.03736761J latent.
- TH013: ethanol uses394.89895591J sensible and4605.10104409J on the actual
  exported vapour. Its declining retained amount is not chemical combustion.
- TH016 remains held: actual ownership labels all0.7647598654mol Zn liquid.
  The pinned sensible integral is458.65774450J and that full liquid fraction
  costs5598.04221474J latent, totaling6056.69995924J against a1000J pulse.
  The5056.69995924J mismatch is preserved; no property was fitted to hide it.
  The recorded melt event accounts for only the partial fraction. Test current
  source separately before declaring the present engine defective.

Other mixture, dissolution, hydrate, reaction and open-boundary energy domains
are retained as observations, without pretending these eight calculations prove
them. Source provenance and unsupported-property qualifications remain binding.
Formal criterion review is separate; the catalog stays200 / fixed468 selected196.

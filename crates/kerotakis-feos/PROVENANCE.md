# Provenance: kerotakis-feos

## Upstream engine

- Library: FeOs, "A framework for equations of state and classical density
  functional theory" (crate `feos`, with `feos-core` and `quantity`).
- Repository: https://github.com/feos-org/feos
- Version: `feos = "=0.10.1"` (crates.io release 2026-07-24), with
  `quantity = "=0.14.0"`. The exact resolved `feos-core` / `num-dual` /
  `nalgebra` versions are fixed by Cargo.lock on the hosted build.
- Licence: MIT OR Apache-2.0 (dual; `license-mit`, `license-apache` in the
  published crate). Compatible with distribution inside this
  AGPL-3.0-or-later workspace; see NOTICE.
- Reference: Rehner, P., Bauer, G., Gross, J. (2023), "FeOs: An Open-Source
  Framework for Equations of State and Classical Density Functional Theory",
  Ind. Eng. Chem. Res. 62, 5347-5357, doi:10.1021/acs.iecr.2c04561.
- Used surface: `feos::pcsaft::PcSaftBinary<f64, 8>` (residual Helmholtz
  energy of PC-SAFT with 2B association, binary-optimised) and
  `feos_core::PhaseEquilibrium::{bubble_point, dew_point}`.

## Model parameters

PC-SAFT pure-component parameters (non-polar dispersion + 2B association,
mu = 0) are typed from:

  Gross, J., Sadowski, G. (2002), "Application of the Perturbed-Chain SAFT
  Equation of State to Associating Systems", Ind. Eng. Chem. Res. 41,
  5510-5515, doi:10.1021/ie010954d, Table 1.

| species  | m      | sigma / A | eps/k / K | kappa_AB | eps_AB/k / K |
|----------|--------|-----------|-----------|----------|--------------|
| methanol | 1.5255 | 3.2300    | 188.90    | 0.035176 | 2899.5       |
| ethanol  | 2.3827 | 3.1771    | 198.24    | 0.032384 | 2653.4       |
| water    | 1.0656 | 3.0007    | 366.51    | 0.034868 | 2500.7       |

Parameter licence: these are published numerical model constants (facts
reported in a journal article), restated with citation; no table, file or
database from the paper or from FeOs's own parameter files is copied.
Binary interaction k_ij = 0 (pure prediction, no fitted binary parameter);
this is a deliberate choice so the comparison against UNIFAC is not tuned.
The parameter values above MUST be re-checked against the printed table by
a reviewer before this crate leaves draft.

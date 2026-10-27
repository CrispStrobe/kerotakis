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
Verified 2026-10-27 field-by-field (m, sigma, epsilon_k, kappa_ab,
epsilon_k_ab, na = nb = 1) against FeOs's own transcription of the same paper:
https://github.com/feos-org/feos/blob/aabc02bbf6c469cf702206cfa68bbb7b2105a324/parameters/pcsaft/gross2002.json
(feos-org/feos main at commit aabc02bbf6c469cf702206cfa68bbb7b2105a324,
entries `methanol` CAS 67-56-1, `ethanol` CAS 64-17-5, `water` CAS
7732-18-5): all 15 values identical, no discrepancy. The companion
`gross2002_binary.json` at the same commit has no methanol-water or
ethanol-water k_ij entry, which supports the unfitted k_ij = 0 choice.

## Vendored parameter file (since this commit)

`data/gross2002.json` is a byte-for-byte copy of feos-org/feos
`parameters/pcsaft/gross2002.json` at commit
aabc02bbf6c469cf702206cfa68bbb7b2105a324, sha256
9680ae8998565feabf6b3808bc2f11df288f55db049677ccebe08a937332c491, licensed
MIT OR Apache-2.0 with FeOs. It is embedded with `include_str!` and parsed
with FeOs's own `PureRecord<PcSaftRecord, PcSaftAssociationRecord>` and
`PcSaftParameters::new_pure/new_binary`, replacing the hand-typed arrays
fed to `PcSaftBinary`. The table above is kept as documentation only.

## Binary interaction and measured model differences

- k_ij = 0 for both alcohol-water pairs. No k_ij consistent with the
  Gross & Sadowski (2002) pure set is published in FeOs: `gross2002_binary.json`
  @ aabc02bb has no alcohol-water entry. `rehner2023_binary.json` @ aabc02bb
  has ethanol-water (cross-association epsilon_k_ab 2324.05 K, kappa_ab
  0.13979) and methanol-water (k_ij -0.01595), but these belong to the
  Rehner et al. (2023) pure-parameter set and are not transferable. Adopting
  that whole set is a possible follow-up.
- Measured (hosted run 670ee975): ethanol-water bubble point at x = 0.05,
  101.325 kPa: PC-SAFT 83.39 °C (y 0.4779) vs Kerotakis UNIFAC 89.47 °C
  (y 0.3503), a 6.08 K difference. Pure PC-SAFT: methanol T_b 338.50 K,
  ethanol 351.66 K, water 373.27 K; p_sat(330 K) 71.81 / 40.58 / 17.36 kPa.
  The full sweep is printed by `ethanol_water_against_experiment_and_unifac`.
- The asserted reference is the ethanol-water azeotrope, 78.2 °C at
  x = 0.894, within 3 K. It is taken from the existing reviewed repository
  control `crates/kerotakis-thermo/tests/vle.rs::unifac_finds_the_ethanol_water_azeotrope`
  (same values in `tests/still.rs`), which has no primary citation of its
  own. Per the PLAN.md provenance audit, commercial handbooks (CRC) and NIST
  WebBook/JANAF-online are not cited for any asserted number here. The
  candidate open primary source is Kamihama et al., J. Chem. Eng. Data 57
  (2012) 339, doi:10.1021/je2008704; still to be confirmed against its
  tables before this PR leaves draft.
- The pure boiling-point and 330 K vapour-pressure references in the tests
  are loose sanity bounds only. They are consistent with the DOI-cited Antoine
  fits already in `crates/kerotakis-thermo/src/vle.rs` (Stull 1947,
  doi:10.1021/ie50448a022; Susial Badajoz et al., doi:10.1021/acsomega.6c04827).

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

## Parameter set replaced: Esper 2023 pure + Rehner 2023 binary (user decision, 2026-10-27)

The Gross & Sadowski (2002) file is removed. With k_ij = 0 it sat 5-6 K below
UNIFAC at x(EtOH) = 0.05-0.1 (hosted run d3d1a79) while reproducing the
azeotrope (78.15 °C). The adapter now uses one consistent published set:

| upstream file (NOT vendored; see subsets below) | upstream path @ feos-org/feos aabc02bbf6c469cf702206cfa68bbb7b2105a324 | sha256 | paper |
|---|---|---|---|
| `esper2023.json` | `parameters/pcsaft/esper2023.json` | 87fdcd8733245d90cd6ad9e9d7d0583bd3f7ffb3c4d67a25197190f289e2a679 | Esper et al. (2023), Ind. Eng. Chem. Res., doi:10.1021/acs.iecr.3c02255 |
| `rehner2023_binary.json` | `parameters/pcsaft/rehner2023_binary.json` | 8b7d008b2bdf7ec49542eb443e36078fbe46e2737775126589c1718dfa13c335 | Rehner et al. (2023), Int. J. Thermophys., doi:10.1007/s10765-023-03290-3 |

The FeOs parameter README pairs these two files.

- Licence: both files are distributed in the FeOs repository under
  MIT OR Apache-2.0. The numbers are published model constants from the two
  papers. I did not verify the papers' own article licences (possibly open
  access); a reviewer should confirm them.
- Records used at this commit:
  - methanol: m 2.25965, sigma 2.83016 A, eps/k 183.58634 K, kappa_ab 0.08716, eps_ab/k 2465.13545 K
  - ethanol: m 2.8866, sigma 2.95772 A, eps/k 187.26028 K, kappa_ab 0.05533, eps_ab/k 2462.30577 K
  - water: m 2.36948, sigma 2.15072 A, eps/k 230.71557 K, kappa_ab 0.35319, eps_ab/k 2195.10176 K
  - All three use 2B sites (na = nb = 1).
  - ethanol-water binary: cross-association kappa_ab 0.13979271332941498, epsilon_k_ab 2324.047279847392 K, no k_ij.
  - methanol-water binary: k_ij -0.0159473671673194.
- Loading: parsed with FeOs `PureRecord<PcSaftRecord, PcSaftAssociationRecord>`
  and `BinaryRecord<Identifier, PcSaftBinaryRecord, PcSaftAssociationRecord>`,
  then `PcSaftParameters::new_binary(pures, model_record, association_sites)`.
  A missing binary record is a typed refusal, never a silent k_ij = 0.
- Size: the full files (about 5.2 MB) are not vendored. Only the records used
  are kept, in subset files extracted by `tools/extract_feos_params.py`. That
  script downloads the files, or reads them from `--upstream-dir`, at the
  pinned commit, and refuses unless the full-file sha256 values above match.
  It copies the records unchanged into `{"_source": {upstream, upstream_sha256,
  extracted_by}, "records": [...]}`:

  | subset | records | sha256 |
  |---|---|---|
  | `data/esper2023.subset.json` | methanol, ethanol, water | bd6e2381dce31a14688844ec5f8ea97c80292f83e2262ccda0fc2998450a1a5f |
  | `data/rehner2023_binary.subset.json` | methanol-water, ethanol-water | ddb62c57b54e3453d88c1d9a921e77820c330c35afa31c913c3fc7ac50aee565 |

  `tests/subset_matches_upstream.rs` checks the headers and record sets. When
  `KEROTAKIS_FEOS_UPSTREAM_DIR` points at the full upstream files, it also
  asserts that each subset record is JSON-identical to its upstream record.
  Otherwise it prints SKIPPED. The subsets are embedded only with the `feos` feature.
- The earlier sections of this file describe the superseded Gross 2002 set
  and are kept as history.

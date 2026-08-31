# BRD-031.S01 — six-fluid parameter source audit

Date: 2026-08-31

## Decision

**No-go for runtime promotion; go for a quarantine-only importer.** The two
best technical sources cover the pilot fluids and live in permissively licensed
repositories, but neither supplies an explicit path-level statement that the
third-party-derived numerical parameter tables may be redistributed under the
repository licence. Kerotakis does not turn that silence into clearance.

No numerical parameter from either candidate is copied into this repository by
this checkpoint. The importer is tested with synthetic records. Runtime routing
remains unchanged and BRD-032 remains blocked.

Subsequent engineering checkpoints do not change that decision. PR #278 merged
the quarantine-only importer with synthetic PC-SAFT-shaped fixtures; PR #289
merged molecular-length typing, and PR #291 merged scoped access to the already
vendored NASA-9 ideal-gas records. None contains a promoted residual parameter
pack. PR #290 (a CC-BY-4.0 ethanol vapour-pressure fit) and PR #293 (verified
offline snapshot tooling) remain open review checkpoints as of this audit date
and must not be described as shipped.

## Pilot identities

The proposed slice is water, carbon dioxide, nitrogen, oxygen, ammonia, and
ethanol. A future promoted record must join the registry by full Standard
InChIKey and separately retain the current runtime species ID; positional or
display-name joins are forbidden.

## Candidate A — feos PC-SAFT parameter repository

- Repository revision:
  `f1fd55d8a3cd7a254cd8721619e01820a453234e`.
- Code/repository licence evidence:
  <https://github.com/feos-org/feos/blob/f1fd55d8a3cd7a254cd8721619e01820a453234e/license-mit>
  and the sibling Apache-2.0 licence.
- Parameter catalogue and publication mapping:
  <https://github.com/feos-org/feos/blob/f1fd55d8a3cd7a254cd8721619e01820a453234e/parameters/pcsaft/README.md>.
- Candidate files (not copied): `gross2002.json` covers water and ethanol;
  `gross2005_literature.json` covers carbon dioxide and nitrogen;
  `esper2023.json` covers all six, including oxygen and ammonia.
- Technical fields: identity metadata, molar weight, PC-SAFT segment number,
  segment diameter, dispersion energy, and association-site data where used.
- Verdict: `candidate_permissive_repository__parameter_rights_pending`.
  A repository-wide licence is useful evidence, but it is not the explicit
  third-party data-right assurance this project's release policy requires.

## Candidate B — CoolProp fluid JSON

- Repository revision:
  `61b616edfbb49f32633b21d1f901bdba1002340a`.
- Repository licence evidence:
  <https://github.com/CoolProp/CoolProp/blob/61b616edfbb49f32633b21d1f901bdba1002340a/LICENSE>.
- Candidate directory (not copied):
  <https://github.com/CoolProp/CoolProp/tree/61b616edfbb49f32633b21d1f901bdba1002340a/dev/fluids>.
- Technical fields: multiparameter Helmholtz coefficients, reducing/critical
  state, saturation ancillaries, and per-correlation bibliography keys for all
  six pilot fluids.
- Verdict: `candidate_permissive_repository__parameter_rights_pending`, for
  the same reason as candidate A. The stripped `coolprop.json` formerly found
  in feos remains excluded; if this route is ever cleared, it must start from
  the original CoolProp files with notices and citations intact.

## Rejected source classes

- NIST WebBook/SRD is not a blanket U.S.-public-domain source. NIST's own
  licensing page directs users to product-specific rights:
  <https://www.nist.gov/open/copyright-fair-use-and-licensing-statements-srd-data-software-and-technical-series-publications>.
- Numerical tables transcribed directly from journal articles are rejected
  unless the table itself carries an allowed data licence. Access to or
  citation of a paper is not redistribution permission.
- feos `parameters/ideal_gas/poling2000.json` and
  `parameters/multiparameter/coolprop.json` remain excluded as already decided
  in BRD-030.

## Clearance evidence still required

Promotion needs one of: an explicit upstream statement covering the tracked
parameter files and their third-party-derived values; permission from the
relevant rights holder; or a replacement dataset released explicitly under
CC0, CC BY 4.0, MIT, BSD, Apache-2.0, or U.S. public domain terms. The evidence
must identify the exact revision/files and be recorded in `sources.toml` before
any candidate bytes or generated runtime pack are committed.

## Recorded permissive-source status

This repository contains no exact source bytes and grant clearing a direct
six-fluid PC-SAFT table under CC0, CC BY 4.0, or U.S. public-domain terms. No
reproducible search log accompanies this checkpoint, so it does not claim an
exhaustive or independently repeatable external search. In particular:

- a permissive code licence does not automatically license third-party-derived
  numerical values stored beside the code;
- NIST WebBook/ThermoML records and journal tables are not assumed public
  domain merely because they are publicly accessible or citable; and
- NASA CEA's cleared NASA-9 records provide ideal-gas heat capacity, enthalpy
  and entropy. They do not provide PC-SAFT segment count/diameter, dispersion
  energy, association parameters or binary interactions.

Therefore an agent can safely write generated tables only after the exact
source bytes and grant have passed the existing snapshot, checksum and
per-field promotion gate. The NASA ideal-gas slice can complement a future
residual model, but it cannot satisfy or bypass the residual-parameter gate.

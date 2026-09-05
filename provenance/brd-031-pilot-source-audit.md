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

---

## Addendum, 2026-09-05 — candidate open-licensed PC-SAFT sources

**Status: candidates, not clearance. Nothing below is promoted, transcribed
into the tree, or read by any code.** This section exists because the
"Clearance evidence still required" list above named a third route —
*"a replacement dataset released explicitly under CC0, CC BY 4.0, MIT, BSD,
Apache-2.0, or U.S. public domain terms"* — and until now this repository
had no record of anyone having looked for one. It now has one.

### What was searched, and what the search does not claim

An offline agent ran roughly a dozen distinct searches and fetched a dozen
candidate documents, checking each one's licence statement at its source
rather than inferring it. This is **not** an exhaustive external search and
does not claim to be. Two further caveats are recorded because they change
what the candidates are worth:

1. **Numbers are not independently verified.** The values below were read
   out of the cited documents by that agent and have not been checked
   against the publications a second time by a human or against an
   independent calculation. They are therefore **not promotable as they
   stand**; a promotion needs the pinned bytes, a checksum, and the
   BRD-003 quarantine path that already exists in
   `crates/kerotakis-data/src/fluid_parameters.rs`.
2. **Most of these are secondary republications.** The parameters largely
   originate with Gross & Sadowski's closed-access papers and are reprinted
   in later open-access articles. A CC BY 4.0 article licenses that
   article's content; whether that reaches numbers the authors took from a
   prior closed paper is a question this repository has not answered and
   should answer deliberately rather than by accident.

### Candidates that met the stated licence test

| # | Source | Licence evidence | Fluids covered |
|---|---|---|---|
| A | Staubach, Schwarz, Möbius, Hasse, Stephan, *Int. J. Thermophys.* **44**:182 (2023), DOI `10.1007/s10765-023-03297-w`, SI Tables S1–S3 | CC BY 4.0 (Springer open access, Crossref `vor`) | CO2, **O2** — the only clean O2 source found; also T_c/p_c/ω for both |
| B | *ChemistryOpen* **10** (2021), DOI `10.1002/open.202000258`, PMC7874510 | CC BY 4.0, quoted verbatim in the article XML | water, CO2, methanol, ethanol, hexane |
| C | *Molecules* **21**(5):593 (2016), DOI `10.3390/molecules21050593` | CC BY 4.0 (MDPI standard statement) | water, ethanol, **ethyl acetate** — the only clean ethyl acetate source found |
| D | *Int. J. Pharmaceutics: X* **3** (2021), DOI `10.1016/j.ijpx.2021.100072` | CC BY 4.0 (Elsevier open access) | **propanone**, ethanol, **nitrogen** |
| E | *J. Phys. Chem. B*, DOI `10.1021/acs.jpcb.6c00492` | "This article is licensed under CC-BY 4.0" | methanol (4C), CO2 |
| F | *Sci. Rep.* **11** (2021), DOI `10.1038/s41598-021-03643-8` | CC BY 4.0 | nitrogen, with M and critical constants |
| G | *Front. Chem.* (2022), DOI `10.3389/fchem.2022.909485` | CC BY | CO2, including a 2B associating variant |

Between them these cover all nine fluids the corpus and Kids Lab reach for.
**Sources B and D disagree on ethanol** — 3.1752/2.8283/170.287 against the
mainstream 2.3827/3.1771/198.24 — which is itself useful evidence: two
incompatible parameter sets circulate under the same substance name, and a
pack that took the first one it found would be picking blind.

### Candidates checked and rejected

* **Esper, Bauer, Rehner, Gross, IECR 62:15300 (2023)**, DOI
  `10.1021/acs.iecr.3c02255` — the actual PCP-SAFT database behind
  `esper2023.json`. Crossref reports ACS `stm-asf` only: closed. Rejected.
* **Rehner, Bardow, Gross, IJT (2023)**, DOI `10.1007/s10765-023-03290-3` —
  genuinely CC BY 4.0, and its ESM was retrieved and checksummed
  (`MOESM1_ESM.pdf`, 1 595 409 B, sha256
  `ad29b0953db9ffcc16e52779eee731db8dcef5a4d2bcc1e17225f71ba86f7ee0`;
  `MOESM2_ESM.csv`, 951 165 B, sha256
  `ebbbf57c6e491ac7fa3307f3308e3a1a305ea6cc911e216724e36741edba06c2`).
  It contains **binary interaction parameters only** and sources its
  pure-component parameters from the closed Esper paper above. Useful later
  for mixtures; useless for the pure-fluid gap.
* **The FeOs paper** (DOI `10.1021/acs.iecr.2c04561`) — CC BY-**NC-ND**.
  Rejected: the licence list above is not satisfied by a non-commercial or
  no-derivatives grant.
* **arXiv 2309.12404** (SMILES→PC-SAFT) — CC BY-**NC-SA**. Rejected.
* **ML-SAFT / `dl4thermo` (Zenodo 12737308)**, record licence `cc-by-4.0`,
  324 577 832 B, sha256
  `edec00a22c3f7d0c0c073f3b07ddc6e65522ae7487d23bbe1ec5497b2f639481`.
  Rejected on two independent grounds. The archive's own `README.md`
  contradicts the deposit's licence stamp — *"Dortmund Databank … A
  proprietary dataset … PLEASE NOTE THAT DORTMUND DATABANK CAN ONLY BE USED
  IN THE COURSE OF THIS PROJECT AND MUST BE DELETED AT ITS TERMINATION"* —
  and the values themselves are not bench-grade: CO2 lands at
  ε/k = 376.29 K and propanone's fitted dipole at 0.737 D against a physical
  ≈2.9 D, both sitting in the authors' own outlier file.
* **feos `parameters/`** and **CoolProp `dev/fluids`** — re-checked at
  current upstream; still no path-level statement. The verdicts above stand.

### Secondary question: critical constants and liquid density

* **Critical constants (T_c, p_c, ρ_c, ω) for all nine fluids** exist in
  Zenodo record 8072892 (`CritProp_v1.1.0.zip`, 15 832 626 B, sha256
  `14c1f8e20a7cbe41ab235f3ece98608b43936f591bc487c635a72443e9d65bd9`) under
  a record-level `cc-by-4.0`. **Recorded and not used.** Its own README
  names the underlying sources as Perry's Handbook, Yaws, the VDI Heat
  Atlas, the NIST Chemistry WebBook and the CRC Handbook — a CC BY stamp
  placed over a compilation of precisely the sources this audit already
  rejects. Depositor licence statements do not launder upstream rights, and
  treating this one as clearance would make the rejections above
  meaningless. Where individual critical constants are needed, sources A and
  F above supply clean ones for CO2, O2 and nitrogen.
* **Saturated liquid density: nothing clears the bar.** No CC0/CC BY/MIT/
  BSD/Apache dataset of saturated liquid densities for these fluids was
  found. NIST's ThermoML archive (DOI `10.18434/mds2-2422`) declares
  `https://www.nist.gov/open/license`, which contains *both* the 17 USC 105
  public-domain clause *and* an assertion that "copyright protection on this
  compilation of data has been secured by the Secretary of the U.S.
  Department of Commerce … pursuant to Section 290(e) of Title 15". Which
  applies to ThermoML is not stated, so it is ambiguous and therefore fails.
  Wikidata is genuinely CC0 and unusable on quality: `P2107` returns 2200
  for water's critical temperature.

  **This is why `kerotakis_thermo::pack` refuses liquid density for every
  fluid rather than shipping a placeholder.** The refusal is the finding.

### What would discharge this

One of the candidates A–G taken through the existing quarantine path:
pinned bytes with a `SnapshotManifest` checksum, per-field provenance and
licence, a second reading of each number against the publication, and a
deliberate answer to caveat 2 above. Until then BRD-032's residual-EOS
routing stays blocked and the pack keeps saying so by name.


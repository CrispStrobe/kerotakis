# USBM IC 9429 ligand slice review

Reviewed 2026-09-06; model-assisted transcription and review disclosed.

Source: Flynn, C. M., Jr. and Haslem, S. M. (1995), *Cyanide Chemistry —
Precious Metals Processing and Waste Treatment*, U.S. Bureau of Mines IC 9429.
[CDC metadata](https://stacks.cdc.gov/view/cdc/206360) explicitly marks this
specific report Public Domain, not merely publicly accessible.
[Original PDF](https://stacks.cdc.gov/view/cdc/206360/cdc_206360_DS1.pdf)
SHA-256: `9e37d39dcf66f7ab6da0e6fbeafdd06622e415a434643c0ceabe1b874a2424a2`.
The full PDF is not distributed; only six manually reviewed scalar constants
and original stoichiometric PHREEQC definitions enter the runtime payload.

## Accepted fields

| Reaction, cumulative formation | log10 beta | Reference |
| --- | ---: | --- |
| Cu²⁺ + NH3 → Cu(NH3)²⁺ | 4.0 | D-7, printed p.149 / PDF index 166 |
| Cu²⁺ + 2 NH3 → Cu(NH3)2²⁺ | 7.5 | same |
| Cu²⁺ + 3 NH3 → Cu(NH3)3²⁺ | 10.3 | same |
| Cu²⁺ + 4 NH3 → Cu(NH3)4²⁺ | 11.8 | same |
| Fe³⁺ + SCN⁻ → Fe(SCN)²⁺ | 3.0 | D-15, printed p.165 / PDF index 182 |
| Fe³⁺ + 2 SCN⁻ → Fe(SCN)2⁺ | 3.6 | same |

Both tables explicitly specify 25 °C and distinguish ionic strengths 0.0,
0.1, 1.0 and 3.0 M. Only the zero-ionic-strength column is transcribed into
the activity-based native problem. These are cumulative, not stepwise,
constants. All selected cells were visually checked against PDF renderings.
Appendix D cites Smith and Martell's *Critical Stability Constants*, volumes
4 (1976), 5 (1982) and 6 (1989), among the compilation's general sources.
These six rows provide no additional reference or uncertainty. Their displayed
precision is preserved; no greater experimental accuracy is claimed.

The native species use a conserved `Thiocyanate` pseudo-element, not free
carbon/nitrogen/sulfur redox pools. The adapter maps that inventory to SCN⁻
formula stoichiometry. Masses use the existing registry atomic-weight basis:
SCN = 32.06 + 12.011 + 14.007 = 58.078 g/mol; KSCN adds K=39.0983.
KSCN is an **aqueous analytical reagent**, not a solid-property model. Its
zero aqueous partial heat capacity and placeholder density are explicitly
unused: solvent owns heat capacity/volume. No solid density, solubility or
heat capacity is invented.

## Explicit omissions and rejected fields

- No complex absorption spectrum, reaction enthalpy, temperature derivative,
  rate coefficient, higher complex or mixed-ligand constant was inferred.
- Temperature is a reference-condition limitation: holding these constants at
  25 °C elsewhere is a labelled approximation, not temperature coverage.
- Table B-2 p.124 prints thiocyanic-acid pKa=+1.6, but its cited primary
  [Morgan, Stedman and Whincup (1965)](https://doi.org/10.1039/JR9650004813)
  reports negative pKa estimates. That internally suspect row is **rejected**,
  not silently sign-flipped. Strongly acidic thiocyanate protonation remains
  outside this slice pending a cleared consistent source.
- The 2022 copper–ammonia review DOI 10.1016/j.clet.2022.100515 is not used;
  its advertised open-access status is not a permissive-data clearance.
- EPA report EPA/600/R-95/085 gives condition-defined constants, but the EPA
  [copyright statement](https://www.epa.gov/web-policies-and-procedures/epa-disclaimers)
  does not generally clear commercial reuse of hosted third-party material.
  That report is not the runtime source.

This review clears only the six named equilibrium fields from this explicitly
public-domain report, not its cited commercial compilations or unrelated data.

## Optical follow-up and verification

No quantitative optical field cleared this review. The primary 2002 study
[Visible spectrophotometric determination of metal ions](https://doi.org/10.1016/S0003267002001356)
reports a monochromatic tetraammine measurement, but its reuse licence was not
cleared. [UCRL-2008](https://escholarship.org/content/qt0pz4s65m/qt0pz4s65m_noSplash_34fc01acd8a619bbaf99f5ce89f44a83.pdf)
contains an aqueous-ammonia copper spectrum, but the plotted concentrations
differ and it is not a species-resolved molar spectrum; governmental sponsorship
alone also does not establish reuse rights. Fe-thiocyanate spectra found for
solvent extraction cannot be substituted for aqueous complex spectra. These
are research leads, not runtime data or claims of quantitative colour coverage.

`ligand_reference.rs` checks 48 native states (two databases, four solvent
scales, three ligand concentrations, two metal systems), asserting the six
activity mass-action identities and native thiocyanate total. This passes;
it tests correct integration, not independent experimental agreement.
The database-index regression also passes: redefining a species clears an
earlier enthalpy/analytic temperature expression just as native PHREEQC does.
Registry re-export and both legacy-difference tests pass, including source
contract byte identity. Cargo licence checking and static provenance checks
pass; the latter deliberately skipped cargo-based quarantine fixtures while
the team's native test slot was occupied. The final workspace/provenance gate
must still be run against the integrated revision.

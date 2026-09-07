# USCG CHRIS: bounded additional-solvent distillation

Reviewed 2026-09-06. This is a narrow original-manual physical-property slice,
not a CAMEO database export, a NIST WebBook import, or permission to reuse
other fields in either compilation.

## Origin and terms

NOAA identifies the linked CHRIS manuals as a separate U.S. Coast Guard
source: https://cameochemicals.noaa.gov/about . The USCG ownership notice
permits copying and distribution of its public information:
https://www.uscg.mil/disclaim/ . The physical-property rows below carry no
third-party copyright marking. Review clears these USCG-authored manual
fields as `LicenseRef-US-Coast-Guard-Public-Domain`, not the entire PDF.
CAMEO's terms specifically preserve third-party restrictions:
https://cameochemicals.noaa.gov/help/reference/terms_and_conditions.htm .
No CAS identity, NFPA, DuPont, AEGL, ERPG, toxicology, response instructions,
or other restricted-category fields are imported. Existing Kerotakis species
identifiers join the two rows; no CAS field supplies that join. No endorsement
or fitness for real distillation apparatus is implied.

## Per-field selection

Both manuals are dated June 1999. Only sections 9.2, 9.3 and 9.12 enter the
runtime TSV. Units are explicit; latent J/kg times g/mol divided by 1,000,000
gives kJ/mol.

| Manual | Normal boiling K | Latent J/kg | Molar mass g/mol |
| --- | ---: | ---: | ---: |
| [MAL](https://cameochemicals.noaa.gov/chris/MAL.pdf) | 337.7 | 1,100,000 | 32.04 |
| [IPA](https://cameochemicals.noaa.gov/chris/IPA.pdf) | 355.5 | 666,000 | 60.10 |

MAL prints **11.00 × 10⁵**, consistent with its other unit representations;
the search-engine snippet's 110.0 × 10⁵ is incorrect and was rejected.
The original PDFs are not runtime payloads. Download SHA-256:

- MAL: `4bc208e525c012db4dc93d1eee5cd558f0f191156142927b1f5ff57f8844e038`
- IPA: `e6cc82c528177f1bdbf6ff6299e4fb9447e841f142bc51a03d474b2cb7ac3991`

The runtime TSV is pinned separately in `sources.toml`. The pre-existing
still's water/ethanol latent-reference temperatures and latent heats remain
their existing data owner's responsibility; this review does not broaden
their provenance clearance.

## What the computation claims

The multicomponent kernel integrates ideal-liquid Raoult equilibrium with
constant-latent Clausius–Clapeyron pressure anchored at normal boiling. A local
domain of ±40 K around each boiling point is an explicitly editorial model
boundary, **not a fitted accuracy guarantee** or permission to extrapolate an
Antoine fit. The intersection of all active component domains must contain
every pot and stage bubble point; otherwise the entire cut is refused before
any inventory changes. No quarantined Antoine coefficients are consulted.

Each step is bounded by every component's inventory, molar cut fraction, and
optional latent-energy budget. More than two components use the same solver.
Nonideality, azeotropes, finite reflux, solute boiling shifts, and apparatus
heat loss are not predicted. Stage enrichment uses a total-reflux composition
cascade as a separation approximation, not a literal continuously operating
column: literal total reflux has no net product. Reported energy counts only
the withdrawn condensate's latent heat, not reflux/reboiler circulation or
sensible heating. In particular, an ideal IPA/water cut is not a
quantitative real IPA/water separation prediction. That limitation accompanies
every result in both structured output and all rendered registers. The
existing water/ethanol UNIFAC route is retained rather than downgraded.

Tests use pure-component boiling, Rayleigh's independent constant-relative-
volatility identity, scale/permutation invariance, finite inventory and energy
conservation, and atomic refusal of missing competing-solvent data.

# Goldberg and Parker (1985) SO2 slice: per-field review

Source: R. N. Goldberg and V. B. Parker, "Thermodynamics of solution of SO2(g)
in water and of aqueous sulfur dioxide solutions", J. Res. Natl. Bur. Stand. 90
(1985) 341-358, doi:10.6028/jres.090.024. NBS authors (Gaithersburg); a U.S.
Government work. PDF from nvlpubs.nist.gov, SHA-256
b8904c624e19e8dc15cdc555fe648a72b4f8dec0bd6c63fc0ec511d184ebacd1. Values read
from the rendered page image of p. 353 (the text layer garbles subscripts).

Table 9, 298.15 K, standard state 1 bar:

| process | dG kJ/mol | dH kJ/mol | used |
|---|---|---|---|
| (A) SO2(aq)+H2O(l) = H+ + HSO3- | 10.60 +- 0.10 | -17.80 +- 0.40 | yes |
| (B) HSO3- = H+ + SO3-2 | 40.94 +- 0.20 | -3.65 +- 0.10 | check only |
| (D) SO2(g) = SO2(aq) | -0.51 +- 0.10 | -26.97 +- 0.30 | yes |
| (E) SO2(g)+H2O(l) = H+ + HSO3- | 10.09 +- 0.14 | -44.77 +- 0.40 | check: (A)+(D) |

Conversions: log K = -dG/(RT ln 10) with R = 8.31446261815324 J/(mol K):
(A) -1.857040, (D) 0.089348 per bar, +log10(1.01325) = 0.095065 per atm
(molality basis already; no density conversion). The abstract's K(D) =
1.23 +- 0.05 mol kg-1 bar-1 agrees (10^0.0893 = 1.228).

(B) gives 7.1724 against the routed llnl 7.2054 (0.033 log units, inside the
0.035 implied by +-0.20 kJ/mol); the routed constant is kept so the existing
sulfite speciation does not move. Its dH (-3.65) also differs from the routed
9.33032 kJ/mol for the reverse association (i.e. +3.65 vs 9.33): a known,
unchanged discrepancy in the existing llnl borrow, recorded, not fixed here.

Omitted: dCp and dCp/dT (constant-enthalpy van't Hoff only), pyrosulfite (C),
activity/osmotic data, Tables 10-11. Written on the minteq.v4 `Sulfite`
pseudo-element as `Sulfite-2 + 2 H+ = H2(Sulfite)` (log K 9.0624397, dH
27.13032) and phase `SO2(g)` = `H2(Sulfite)` (log K 0.0950647, dH -26.97);
`H2(Sulfite)` is H2SO3 = SO2(aq) + H2O per Table 10 note c.

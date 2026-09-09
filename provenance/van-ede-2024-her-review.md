# Stainless-steel hydrogen-evolution and oxygen-reduction parameters

Reviewed 2026-09-08. M. C. van Ede and U. Angst, “Tafel slopes and
exchange current densities of oxygen reduction and hydrogen evolution on
steel,” *Corrosion Engineering, Science and Technology* (2024),
[doi:10.1177/1478422X241227829](https://doi.org/10.1177/1478422X241227829).
The article and supplement are CC BY 4.0. Supplement SHA-256:
`d58dd9c5f98fc8ba0e492d4eeb0f1b765022da91501399da8118016cc755ad28`.

Only Supplementary Table B1's three repetitions at the same 1200 rpm,
0.50 mV/s, initial upward-scan condition are admitted. Their cathodic HER
values are `j0 = 0.011, 0.017, 0.017 A/m2` and
`|b| = 0.16, 0.14, 0.16 V/dec`. Runtime nominal values are the arithmetic
means, `0.015 A/m2` and `0.15333333333333332 V/dec`; the typed envelope is
the observed min/max. Paired rows remain paired source observations rather
than independent probability distributions.

The domain is the reported X5CrNi18-10 disk, 1 µm diamond polish, ethanol
degreasing, three-minute ultrasound cleaning, five-minute cathodic
polarisation at -1.5 V versus Ag/AgCl/saturated KCl, 0.5 h immersion,
N2-deaerated 0.1 M boric-acid/borax buffer at pH 7.5 with 0.027 M chloride,
oxygen below 0.30 ppm, 16 °C and 1200 rpm. Other steels, preparations,
temperatures, pH values, flow or scan protocols do not match.

Supplementary Table C1 supplies a second compatible ensemble for oxygen
reduction on the same stainless-steel preparation at 1200 rpm and 0.50 mV/s
in the initial upward scan. The aerated experiment was at room temperature,
reported as around 20 °C. The three rows give
`i0,O2 = 3.5e-7, 1.9e-7, 4.2e-7 A/m2`,
`|b_cath| = 0.18, 0.17, 0.18 V/dec`, and measured limiting currents
`2.2, 1.6, 2.2 A/m2`. Runtime stores the first two arithmetic means and their
observed envelopes. The measured limiting currents are independent validation
evidence: production transport limits remain computed from the available O₂,
stoichiometric electron count and diffusion model rather than frozen to an RDE
apparatus value.

The article writes the neutral ORR as
`O2 + 2 H2O + 4 e- -> 4 OH-`, uses `Erev = 0.79 V vs SHE` at pH 7.5, and
fits the exchange current by extrapolation to that reversible potential. The
record therefore uses four electrons per reaction extent. The separate
Levich-derived effective electron counts in Table C1 are observations about
the experimental pathway and do not redefine the balanced inventory equation.

Both admitted measurements are cathodic Tafel branches. They supply no
corresponding anodic transfer coefficient, so Kerotakis stores directional
laws and does not manufacture full Butler–Volmer couples. No metal-dissolution
parameter is imported. Consequently the records can model HER/ORR competition
on the reviewed stainless surface, but cannot yet replace a complete iron or
zinc corrosion clock.

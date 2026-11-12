# Headspace gas colour coverage (2026-11-12)

Mechanism: `crates/kerotakis-core/src/gas_colour.rs`. A vessel that owns its gas (sealed or
pressure-controlled) gets `headspace_colour` in its scene: Beer–Lambert over every gas portion,
A(λ) = Σ ε(λ)·(n/V_headspace)·l with l = 4 cm (`BEAKER_PATH_CM`, the liquid's viewing depth), and
the existing transmitted-colour conversion. ε = σ / 3.8235e-21 (L mol⁻¹ cm⁻¹ from cm² molecule⁻¹),
averaged over each band centre ± 10 nm (405–705 nm, 16 bands).

Data: MPI-Mainz UV/VIS Spectral Atlas files (Keller-Rudek et al., ESSD 5 (2013) 365,
doi:10.5194/essd-5-365-2013; frozen archive doi:10.5281/zenodo.6951, **CC BY 4.0**), each with its
primary source.

| Gas | Colour | ε max in band (L mol⁻¹ cm⁻¹) | Source |
|---|---|---|---|
| Cl2 | yellow-green | 4.08 at 405 nm | JPL 10-6 (2011) rec., Maric et al. 1993; beyond 550 nm (data end) taken as 0 |
| Br2 | red-brown | 162 at 425 nm | JPL 10-6 (2011) rec., Maric et al. 1994; 665–705 nm Passchier et al. 1967 |
| I2 vapour | violet | 828 at 525 nm | Saiz-Lopez et al. 2004 (band means over the banded region) |
| NO2 | red-brown | 158 at 405 nm | Vandaele et al. 2002 (FT, 294 K) |
| O3 | pale blue | 1.27 at 605 nm | JPL 10-6 (2011) rec. (Reims group) |

Cited colourless (no visible absorption at bench amounts): SO2 (Hermans et al. 2009), H2S
(Grosch et al. 2015), NH3 (Cheng et al. 2006), CO2 (Shemansky 1972), O2 (He et al. 2021).
N2O4 is colourless at 405 nm (Holmes & Daniels 1934) but is not a registry species.

Any other gas counts as colourless only when its registry appearance says so; otherwise the
colour is withheld (`srgb` absent) and the gas is named in `uncited`. Open and swept vessels have
no headspace of their own and carry no colour. The web stage fills the headspace band with the
colour at 0.85 opacity. Sourcing notes: private round-4 research file `gas-cross-sections-sourcing.md`.

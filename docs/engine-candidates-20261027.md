# External engine candidates — licence and fit audit (2026-10-27)

Kerotakis already bundles PHREEQC (aqueous speciation), NASA CEA (gas/combustion equilibrium), SUNDIALS
(kinetics ODEs), its own UNIFAC and chematic (cheminformatics). Licences below were read from each repository's
licence file or Cargo manifest on 2026-10-27, not from memory. Parameter *data* can carry separate terms from
the code: verify the provenance of every imported parameter set before shipping it.

| Candidate | Licence (verified) | Language | What it adds | Fit |
| --- | --- | --- | --- | --- |
| **CoolProp** | MIT | C++ (C API, official JS/Wasm builds) | Reference equations of state for water/steam (IAPWS-95) and ~120 pure fluids; saturation, density, heat capacity, viscosity, conductivity | **Recommended first.** Used wherever water or common solvents are heated, boiled, condensed or pressurised |
| **FeOs** | MIT OR Apache-2.0 | Rust | PC-SAFT, ePC-SAFT (electrolytes), gc-PC-SAFT; mixture phase equilibria (VLE/LLE), critical points, DFT | **Recommended second.** Native Rust; mixture distillation, extraction and salts in mixed solvents |
| teqp (NIST) | NIST public-domain notice (US); may be under foreign copyright | C++ header-only | Fast multiparameter, cubic and SAFT EOS with automatic differentiation | Alternative to CoolProp for mixtures; smaller ecosystem |
| Cantera | BSD-3-Clause (Caltech/Sandia text) | C++/Python | Detailed gas kinetics mechanisms, flames, transport, surface chemistry | Overlaps CEA+SUNDIALS; Wasm build is heavy. Only for flame/ignition or surface-catalysis work |
| thermopack (SINTEF) | Apache-2.0 / MIT | Fortran/C++ | Cubic and SAFT EOS, phase envelopes | Overlaps FeOs; Fortran complicates Wasm |
| thermo / chemicals | MIT | Python | Large property correlations and data | Port reviewed data with provenance, not the code |
| Clapeyron.jl | MIT | Julia | Broad EOS collection | Julia runtime makes bridging impractical |
| PySCF | Apache-2.0 | Python/C | Quantum chemistry | Too heavy for interactive use; possible offline data generation only |
| RDKit | BSD-3-Clause | C++ | Cheminformatics | Already covered by chematic's RDKit-parity port |
| Reaktoro | LGPL-2.1 | C++ | Multiphase reactive equilibrium | Excluded by licence policy |
| xtb, tblite, Psi4 | LGPL-3.0 | Fortran/C++ | Semi-empirical / ab initio | Excluded by licence policy |
| Open Babel | GPL-2.0 | C++ | Format conversion | Excluded by licence policy |

## Recommendation

**Independent of the current gaps:**
- **CoolProp** brings the largest general accuracy gain for the least risk. It improves boiling, steam,
  vapour pressure, density and transport properties for water and common lab fluids across the whole
  bench, it has a stable C API and existing Wasm builds, and the MIT licence is unambiguous.
- **FeOs** is the natural second engine. It is Rust like the rest of the codebase, so it needs no FFI,
  and it opens nonideal mixture VLE/LLE for distillation and extraction, plus ePC-SAFT for electrolytes
  in mixed solvents.

**For the 401 goal:** neither engine is the next lever. In the
[remaining-129 triage](../audits/remaining129-production-triage-20261027/README.md), only 3 cases name
FeOs or teqp/CoolProp; the rest need in-house ownership, data or observable repairs.

**Integration shape:** use a separate adapter crate per engine with an explicit domain (species,
temperature and pressure range). Out-of-domain states must be refused, never extrapolated silently.
Use parity fixtures against the existing PHREEQC/UNIFAC/CEA results where domains overlap, and enforce
element, charge and energy closure across engine handoffs.

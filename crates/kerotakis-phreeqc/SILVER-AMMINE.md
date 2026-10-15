# Finite silver–ammonia equilibrium routing

## Problem and change

An aqueous AgCl/NH3 input previously selected `wateq4f.dat` because that dataset
contains both silver and reduced nitrogen as elements. It lacks the silver
ammine solution species, so elemental coverage did not provide ligand coverage.
The saved physical stock follow-up retained almost all AgCl and reported no
silver ammines. Its raw outcome remains historical evidence.

The new capability gate in `src/aqueous.rs` selects the existing `minteq.v4.dat`
route when the represented elements include both Ag and N(-3). This includes
ammonia and ammonium, whose protonation equilibrium determines available NH3.
Nitrate N(5), including AgNO3 alone, does not satisfy the gate. No sodium,
chloride, ligand reservoir, equilibrium constant or reaction yield is invented.
The ordinary concentrated-load caveat and typed-interface admission remain.
The native MIX shortcut declines when the combined source elements require
silver ammines, including when neither source requires them alone. The caller
then uses the ordinary target solve and its capability routing/provenance.

## Existing data and convention

The vendored `vendor/iphreeqc/database/minteq.v4.dat` already supplies:

| Reaction | log K | Reaction enthalpy | Cited entry |
| --- | ---: | ---: | --- |
| Ag+ + NH4+ = AgNH3+ + H+ | -5.934 | -72 kJ/mol | NIST46.4, 204901 |
| Ag+ + 2 NH4+ = Ag(NH3)2+ + 2 H+ | -11.268 | -160 kJ/mol | NIST46.4, 204902 |

These constants belong to the written NH4-based equations. They must not be
inserted unchanged into NH3-based formation equations. The same database
supplies NH4+/NH3 protonation and the Cerargyrite AgCl solid. Existing phase-name
reconciliation, native mass action, activities and finite element balances
compute dissolution and complexation. This is not a new thermodynamic data slice.

## Acceptance and limits

`tests/silver_ammine_route.rs` exercises finite NH3 supply, dissolution enhancement
relative to AgCl without ammonia, native activity mass action, dissolved/solid
silver and nitrogen recovery, analytical ledger conservation and repeated
characterization. It also tests acidic ammonia protonation, nitrate exclusion
and ammonium admission. A separate-source mixing control checks ordinary-target
fallback against direct preparation and verifies source inventory is unchanged.
The acid control adds HCl: both ligand protonation and
chloride common-ion effects are present, so it is not an isolated measurement
of either contribution. No acceptance requires complete AgCl dissolution at a
finite two-equivalent NH3 dose.

Public speciation is rounded to four significant figures. Tests comparing sums
of those reported species use a 5e-4 relative rounding allowance plus a 1e-12 mol
trace floor. The public species list omits molalities at or below 1e-9; the
alkaline control therefore obtains hydrogen activity from `10^(-pH)` rather
than requiring a filtered-out H+ row. Its mass-action check still requires the
silver and ammonium activities and both NIST equations. The first native test
attempt's missing-activity lookup failure remains preserved. A fresh native run
must confirm the corrected assertion and the remaining finite-balance controls;
that failed lookup alone did not establish a chemistry failure. The first
[native failure](https://github.com/CrispStrobe/kerotakis/actions/runs/38799616390/job/116644921313)
at `c32a608febdca6e506528d364e2da7ada1cc7e67` passed three other focused
groups and failed this lookup before its mass-action assertions.
Analytical-ledger charge and element conservation are checked
separately, over the union of pre/post elements so newly created elements are
also detected; the public-speciation allowance does not relax that ledger.

### Analytical ligand ownership remains limited

Silver ammines and their free-ligand distribution are reported in
`solution.species`. The canonical NH3/NH4+ portions remain analytical total-N
carriers: readback distributes total N(-3), including complex-bound ligand,
using the free NH3/NH4+ weights. Those portions are not measurements of free NH3
when silver ammines bind nitrogen. This routing change does not repair that
broader amount-ownership contract.
A positive silver-ammine solution population emits the structured
`not-modeled.silver-ammine-ligand-inventory` diagnostic explaining this specific
cross-route limitation. It preserves the valid equilibrium distribution and its
provenance; it does not claim that silver complexation is absent. Both ordinary
readback and the native MIX owner attach the diagnostic. Focused controls check
its presence with ammines and absence without ammonia or with nitrate alone.

In particular, `kerotakis-core/src/volatility.rs` currently partitions and
withdraws canonical NH3 portions for finite headspace, and the curated
NH3/hypochlorite rows match those portions. Their joint behavior with
complex-bound ammonia is not validated by this static equilibrium lane.
Do not infer validated ammonia volatilisation, reagent eligibility or reactive
free-ligand consumption from these canonical quantities. A separately reviewed
owner/reconstruction change is needed before such coupled claims.

The work is an equilibrium capability repair, not experimental calibration,
reaction kinetics, optical validation or a deployment claim. The legacy
analytical-component input convention remains separate. Higher-concentration
activity-model limits and applicable typed-interface restrictions still apply.

## Source preparation receipt

- Base: integrated main `4bf2b973`.
- Data: existing vendored `minteq.v4.dat`, unchanged; IPhreeqc gitlink
  `47b459c4a5e3e20b9990fa304e207eca0dcc6ff1`, database SHA-256
  `4a48bf5c357b3da3084606a5e322426c3ab1e969dd1c86f9c62a3a7995836ca3`.
- Validation performed at preparation: Rust formatting and source diff checks.
- Native focused tests, broader chemistry regression and own-fleet acceptance
  remain pending. No local compilation or native execution was performed during
  source preparation; its resource prerequisites were not satisfied.

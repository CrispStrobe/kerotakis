# Native MIX finite-inventory boundary

The optimized aqueous `mix` implementation serializes source dissolved totals
and zero-amount precipitation candidates. It creates empty gas, surface,
exchange and solid-solution ledgers for readback. This is insufficient when a
source or target carries positive solid/gas inventory or a typed interface.
The ordinary equilibrium path owns those inventories.

The candidate returns `None` before an engine call or target mutation for these
cases. `Bench` already treats `None` as an instruction to use ordinary target
equilibrium. This preserves the solution-only MIX optimization and avoids
representing omitted inventory as a successful native mixture. It does not add
a multiphase MIX model or change transfer fractions and physical entrainment.

## Accepted regression scope

Native tests must verify every source/target position declines without an engine
call or any serialized target mutation for solids, gas, surfaces, exchangers and
solid solutions. A real ordinary fallback checks finite sodium and residual salt,
sealed inert oxygen and a finite solution result. The existing uncoupled native
MIX positive control must continue to pass. Exact-head native CI and all33 own-fleet jobs passed before integration as
`d6020643`; the [receipt](../audits/target468-20261016/accepted-pr-integration/pr823-integration.json)
verifies actual parents/tree. Formatting alone is not acceptance.

After integration, preserve source-bound observations for representative pour
and decant operations, including settled solids left behind and gas not
transferred with liquid. A shortcut declining its domain is not proof that every
transfer and ordinary-equilibrium domain is scientifically validated. No working
catalog increment follows from these boundary regressions.

# Compiled solid dissolution inventory

The finite-phase crosswalk only contains solids matched to a database phase.
It therefore omits the no-phase KNO3 failure entirely. This diagnostic enumerates
every standard-phase solid in the actual compiled registry and asks the same
generated role mapping used by the aqueous solver.

```sh
cargo run -p kerotakis-phreeqc --no-default-features \
  --example solid-solubility-coverage > solid-solubility-inventory.json
cargo test -p kerotakis-phreeqc --no-default-features \
  --example solid-solubility-coverage
```

Compilation still requires resource headroom. These commands do not request the
native PHREEQC engine and perform no equilibrium solves. Bind the actual source,
registry/database inputs, dependency lock, executable and raw output before
publishing counts. No inventory count or executed outcome is claimed by this
source-only description.

Each row retains canonical identity, generated role and element contributions,
reviewed finite-limit fields, bookkeeping/intentional-exclusion flags, offered
candidate phases, and every database's composition matches, native phase and
missing element masters. Rows are sorted by canonical key. Unmapped solids are
included; an empty element list has unknown admission rather than vacuous success.

`finite_solubility_unreviewed` identifies a specific mechanism: a solid assigned
phase-blind `Dissolves`, without a reviewed limit or active offered phase.
`false` does not establish support. Element admission and bookkeeping flags do
not validate saturation, activities or temperature dependence.

For mineral rows, missing native phases expose a separate review queue. They do
not prove that a runtime fallback dissolved a solid: borrowed phases, coupled
chemistry, surfaces, hydrates and actual operator paths need their own review.

Three test groups require complete unique solid membership and stable ordering,
the known KNO3 role/nitrate-admission gap, and distinct native NaCl versus bounded
borax behavior. The KNO3 assertion records a known gap, not a passing physical
forecast; update it deliberately when a reviewed chemistry repair changes the
mapping. Do not silently remove KNO3 from the inventory to obtain green tests.

Use the output to complete the [finite-solubility repair lanes](https://github.com/CrispStrobe/kerotakis/blob/5808123a346ced48e407ddcd903ed4f252a7c7c2/docs/kno3-finite-solubility-lane.md).
This tooling changes no dissolution, amount, phase, activity or temperature model.

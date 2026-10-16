# Two reviewed input spellings

The case-sensitive input spellings below resolve through the existing shared
species boundary. Canonical stored identities, formulas, charges, phases,
properties and finite bottles are unchanged.

| Input | Canonical identity | Meaning |
| --- | --- | --- |
| `CuSO4_anhydrous` | `CuSO4` | Existing neutral, solid anhydrous copper sulfate. |
| `acetate` | `CH3COO-` | Existing aqueous C2H3O2 ion with charge −1. |

This implements only the two alias candidates from the separately frozen
[identity review](https://github.com/CrispStrobe/kerotakis/blob/f317153b20460a20970bbce51704f986c8caf8eb/audits/reaction-dataset-200-20261015/identity-first15-review-20261016.json). The original supplied scripts, forecasts and first refusals remain
historical evidence. The registry and thermodynamic/property data are unchanged.

`chalcanthite` remains CuSO4·5H2O with its separate hydrate identity and molar
mass. `acetate` supplies no sodium: acetic acid (`CH3COOH`), sodium acetate
(`NaOAc`) and ethyl acetate remain different substances. Neutral `Na` is not
aliased to `Na+`. No hydrate, counterion, phase or reaction is inferred.
Bare acetate input retains the existing analytical-component charge convention
and its physical-mixture limits; accepting its spelling establishes no
hydrolysis or neutralization model.

`tests/proven_input_alias.rs` checks text/direct/JSON admission, canonical
serialization and legacy restore, mixed-spelling finite molar stock, exact
compensated amounts and restored-bottle collision refusal. It also checks
identity negatives and loaded-pack collision protection. The controls use
explicit mole amounts; they do not invent density-based volume conversions.

Source formatting and static review are the preparation checks. Focused native
tests and applicable full CI/own-fleet acceptance remain pending until they run
against this exact candidate. This naming change assigns no scientific passes.

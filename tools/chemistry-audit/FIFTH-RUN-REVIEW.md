# Fifth-fleet execution review

The completed recorder output is [fifth-batch-1](fifth-batch-1/).
All **36 cases (159–194) exited successfully**. The unchanged predeclared
analyser reports **119 checks passed, 0 unmet**: 36 execution checks,
42 conservation checks and 41 model-law/control checks. This is not a claim
that all diagnostics are correct or all chemistry is modeled.

The recorder completed normally; no partial summary was reconstructed and no
case was rerun. Recorded per-case process times sum to 477.525 seconds. Timing
does not establish chemical accuracy and includes process/engine startup.

## Evidence and unchanged expectations

- [Preflight hashes](fifth-batch-1/preflight.json)
- [Completed recorder summary](fifth-batch-1/summary.json)
- [Frozen law-check results](fifth-batch-1/law-checks.json)
- [Predeclared protocol and model domains](FIFTH-BATCH.md)

The input-generator, analyser and protocol hashes were verified against
preflight before analysis. The completed summary's binary fingerprint matches
preflight: `43f041c05fbdc795b5b2ac77b214a74dfd7fbae2b8983ddeefa3e1c023414bcd`.
No check or tolerance was edited after seeing output. The analyser command was:

```sh
python tools/chemistry-audit/analyse_fifth.py tools/chemistry-audit/fifth-batch-1 --out tools/chemistry-audit/fifth-batch-1/law-checks.json
```

The preserved checks cover limiting precipitation and acid reversal,
diprotic titration capacities, charge-derived electrode products, sealed
gas pressure and element closure, three-feed fractional MIX, and ideal
organic mass action with order/scale/repetition controls. Every execution
check found finite nonnegative registered inventory and no `solver_failed`.

## Confirmed diagnostic defect outside the frozen numerical checks

Case 192's second equilibrium request leaves the physically correct equilibrium
inventory unchanged, but emits `not_yet_modeled` with cause
`nothing-to-act-on`, incorrectly stating that the vessel needs acid and alcohol
together. Both reactants are present:

| Species | Moles before and after the second request |
| --- | ---: |
| CH3COOH | 0.010276049844845335 |
| ethanol | 0.002276049844845333 |
| ethyl_acetate | 0.008723950155154666 |
| water | 0.010723950155154666 |

The first request has computed extent `0.006723950155154666 mol`; the second
requires no further extent. The mass-action and idempotence checks pass.
Raw evidence is in
[case 192 output](fifth-batch-1/192-ester-repeat-equilibrium/stdout.ndjson).

The reviewed `Operator::React` branch in
`crates/kerotakis-core/src/bench.rs` tests whether the computed extent is finite
and its absolute value exceeds `1e-12`. Failure of that test is interpreted as
missing reactants, although a valid equilibrium root can be zero. The same
branch also converts a model error into zero before reaching this diagnostic.
These are distinct states, not one generic absence of reagents.

Minimum generic repair proposal: distinguish a valid zero equilibrium extent
from absent reaction capacity and from a model-computation error. A valid
zero-extent result should state that the current mixture is already at the
declared equilibrium, while preserving the model boundary and unchanged
physical inventory. Regressions should cover nonempty equilibrium mixtures,
genuinely absent substrates, valid reverse reactions, and model errors. This
review makes **no production fix** and does not rewrite the original evidence.

Thus the broader diagnostic review has **one confirmed correctness gap** even
though no frozen numerical assertion failed. A future expanded analyser should
test truthful status diagnostics separately from equilibrium idempotence.

## Explicit limitations, not numerical failures

The recorder retained 13 `not_yet_modeled` diagnostics: 12 mixed-organic-solvent
ionic-domain warnings and the one misleading equilibrium diagnostic above.
The organic-rich vessels are explicitly uncharacterized for ionic speciation;
an ideal esterification extent does not supply a valid mixed-solvent ionic
activity model. Their reaction events also state that rates, catalyst effects,
activity corrections, temperature dependence and reaction heat are not modeled.

The electrolysis checks validate the declared ideal Faraday route and actual
finite-headspace gas law, not electrode overpotentials or bubble dynamics.
Precipitation controls do not establish nucleation rates or universal access
to every possible solid phase. Passing these 36 cases supports their stated
domains and controls only.

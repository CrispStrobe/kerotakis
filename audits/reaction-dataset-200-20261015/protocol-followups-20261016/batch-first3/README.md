# First three protocol follow-ups — recorded once

These three prospectively frozen requests completed serially on the already
bound **preview source74a030ec / binary30f32713**, with HTTP200, zero process
exits and matching source/binary/script identities. Fresh health was checked
before the requests. No local CLI/build, deployment, model change or retry was
performed. This evidence does not describe the newer production deployment.

[The design](design.json) retains original rank17/32 records, script hashes and
source forecasts separately in [originals.json](originals.json). These are
source-informed protocol exercises, not blind experiments or empirical
calibration. The reused artifact's lock hash is producer-reported; raw build
lock bytes were unavailable, as retained in [the API binding](api-binding.json).

| Follow-up | First saved result and scope |
| --- | --- |
| Rank17 explicit ideal equilibrium | `react v1 esterification` gives extent0.0006666666666666663mol from neat equimolar0.001mol stocks. Reactant debits, ester/water products and C/H/O/charge close; temperature stays298.15K. Its event explicitly withholds catalyst, activity corrections, temperature dependence and reaction heat. This demonstrates the requested ideal-K4 model exercise, **not closure of the original timed/catalyzed Fischer case**. Ionic speciation of the mostly organic mixture remains explicitly uncharacterized. |
| Rank32 with MnO2 | The10second teaching-model interval consumes2.8010421088975e-6mol peroxide and produces1.4005210544488067e-6mol O2. MnO2 remains0.0001mol. GasProduced and Reacted events identify the target and catalyst. |
| Rank32 without catalyst | The matched interval consumes8.12397908576e-8mol peroxide and produces4.0619895428817714e-8mol O2. No catalyst is inferred. This background amount is below the visible kinetic narration threshold. |

The virtual catalyst/no-catalyst oxygen ratio is about34.48. Water/gas
stoichiometry and full canonical material/charge closure pass **including
retained gas**. Both open vessels nevertheless retain new bulk Phase::Gas O2
without GasEvolved/GasContained transfer events. The frozen open-export
expectation therefore **fails in both controls**; a green HTTP/process exit
cannot mask it. This is a boundary-contract/residence-accounting investigation,
not evidence of missing mass and not yet a judgment that transient bubble
residence was unintended. [MODEL-005](../../model-issues-20261016/MODEL-005.json)
retains the first observations and defines independent follow-up work. Neither
rank32 nor any catalogue case is closed by this batch.

[Assessment](assessment.json) separates every check from raw observations.
[Execution metadata](execution.json), [completion](completed.json), and
[pre-request health](health-validation.json) retain exact bindings. Per-case
`.lab`, `.request.json`, `.response.json` and client stderr files preserve the
first inputs and results. Returned streams were checked for owner/deployment
credential reflection and private host paths before publication.

`review_saved.py` reads only these saved files and independently rechecks
formula, amount, charge, event and identity bindings. It cannot send a request.
Use a new `--report` filename for a subsequent review; preserve the original
assessment and forecasts. The separate model agent must investigate MODEL-005
under current source/resource/acceptance rules before changing model behavior.

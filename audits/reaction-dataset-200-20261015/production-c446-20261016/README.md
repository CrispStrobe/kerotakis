# Production REST delivery acceptance — c446

The production alias passed the bounded delivery checks and both complete
chemistry controls. This confirms this deployment's two exercised procedures,
not the entire supplied datasets, new chemistry coverage or empirical calibration.

The [prospective design](design.json) was frozen before any requests.
[Artifact binding](binding.json) authenticates successful
[production run 38868092504](https://github.com/CrispStrobe/kerotakis/actions/runs/38868092504),
artifact 12033256299 and its ZIP digest, exact source/tree, rehashed native binary
and five libraries, matching snapshot engine, deny-all configuration and actual
producer smoke/isolation results. [Native manifest](native-manifest.json),
[producer smoke](producer-smoke.jsonl) and
[isolation observation](producer-isolation-observation.json) preserve the evidence.
The binary is `670bd3d8a5f1bf157d2f0e90e11597135183b906af48f7a0e6cdabe4cd26a54f`.
Raw producer Cargo.lock bytes were absent: its reported hash is bound through
manifest, snapshot and live health, but was not independently rehashed.

## Actual bounded requests

Nine requests were issued once each, serially at least 3.1 seconds apart: health,
client GET, missing/bad authentication, disallowed command, denied/allowed CORS
preflights and two chemistry POSTs. All returned their expected statuses:
200, 200, 401, 401, 422, 400, 200, 200 and 200. There were no retries, quota tests,
deployment changes, local builds or local CLI executions.

The live health matched source `c44692053a94c59dcf62e7fb67549f84c7d18f83`,
engine and producer lock. Served `/rest.html` bytes exactly match source
`web/rest.html`. Pages remains separately reported as 404; it was not retested.
No new browser execution was needed for this unchanged client.

- Water baseline: 5.534276991396059 mol water at 298.15 K, solvent-only pH 6.99738.
- Unchanged complete-stock rank-001 control: acid pH 2.03811, neutralized pH 6.99269,
  after ten seconds 6.99271. Final inventory retains 0.001 mol each Na+ and Cl-
  and gains 0.001 mol water. H/O/Na/Cl and charge residuals are zero at the saved
  precision. The modeled base-addition temperature rise is 0.240731 K.

These meet the separately frozen material, pH, thermal-response and stability
checks. Older source/binary responses are context rather than an exact byte-equality
oracle. Conservation and a modeled temperature response do not establish measured
reaction heat or an independently calibrated temperature prediction.

The [assessment](assessment.json), [execution](execution.json),
[completion](completed.json) and [verification](verification.json) retain hashes,
statuses, source references and scope. Every response body, header block and client
stderr is saved separately. The [saved reader](review_saved.py) never requests
execution; use a new `--report` destination when rereviewing.

## Isolation, credentials and cleanup

The producer's actual same-snapshot isolation probe passed its input-sentinel,
file creation/read, environment, unprivileged identity and process/network checks.
Exact attempted operations and their scope are listed in `verification.json`;
this is not an exhaustive sandbox-policy test.
Live request completion agrees with source's bounded nonpersistent sandbox context
and admission release. These facts **do not independently prove VM deletion**;
no deletion receipt or extra VM management request was obtained.

The client loaded the owner key privately into a temporary mode-0600 curl config,
never arguments, environment or public output. Both authorization configs were
removed in `finally`. Returned bodies, headers and stderr were scanned for owner
and bad-key reflection and private host paths before publication. No credentials
were echoed. No model source or shared catalogue/status was changed.

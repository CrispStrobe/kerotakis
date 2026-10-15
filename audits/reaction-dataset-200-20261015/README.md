# CLI probes for all 200 supplied reaction records

The [chemistry results and ordered repair lanes](CHEMISTRY-RESULTS.md) now review
all 200 saved responses and twelve separately frozen complete-reagent controls.
Execution success and expected chemical behavior are reported separately.

Both supplied JSON files were read. The 200-record dataset validates against
its supplied JSON Schema; the schema itself is not a chemical reaction.
An independent [atom/charge check](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/input-balance-check.json) confirms 186 exact
or overall equations; the 14 schematic records remain outside that balance claim.
[Input hashes](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/input-inventory.json), [the frozen manifest](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/design/manifest.json)
and [all 200 scripts](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/design/scripts) retain exact record accounting.

Execution completed in [run 38763662200](https://github.com/CrispStrobe/kerotakis/actions/runs/38763662200).
All 200 records were attempted: 92 zero exits and 108 nonzero exits, with no
timeouts or invalid JSON. All additions were reached in 92 scripts.
[Authenticated completion](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/completion.json), [case report](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/execution-assessment.json)
and [CSV](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/execution-summary.csv) preserve outcomes. Do not duplicate the batch. Its isolated branch
`audit/reaction-dataset-200-20261015` must never merge into main. Only the explicit
200-record lane is eligible; all nine ordinary fleet jobs were verified skipped.
No new binary build is performed.

[Execution binding](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/execution-binding.json) identifies preserved main source
`d791d306`, its executable, dependency lock and authenticated producer artifact.
This source was main at the earlier build; later merges do not silently change
the binding. The hosted producer authenticates the API association, ZIP checksum,
safe member set, source identity, binary and lock before starting the runner.

Every record states that no procedure is provided. These are declared
contact/activation probes, with one millimole per numeric stoichiometric unit,
source ordering, a ten-second wait and per-case inspections. Hydronium is
represented explicitly as H+ plus its associated water. Charged notation and
name mappings require matching elemental composition and charge; molecular
formula alone is not used to select an isomer. Unknown identities are submitted
unchanged, not omitted. Symbolic coefficients use one solely for a finite probe.

Water is added for aqueous phase labels, and gas cases use a sealed one-litre
headspace with an actual initial-air baseline. Activation doses for ignition,
heat, electrolysis or light are declared screening assumptions. Dataset phase
labels cannot override registry phases. Missing catalysts and multistage
procedures remain missing. Read the independent [protocol review](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/protocol-review.json):
some organic water setups are incompatible or incomplete, and starch amounts
use a repeat-unit surrogate. Negative results there do not establish a chemistry
defect.

The runner gives every record its own CLI process and preserves stdout, stderr,
exit status, timeout, timing and hashes. Each child has a 30-second timeout and
a one-GiB address-space ceiling. It continues after individual refusals and
timeouts; a resource stop preserves partial results and stops the batch. A CLI
script itself stops at its first refusal, so attempting 200 scripts does not
mean every reactant or activation was reached.

After completion, authenticate the run/artifact and then use
`assess_execution.py EXECUTIONS_DIRECTORY NEW_REPORT_DIRECTORY` to verify all
200 records and streams. The report distinguishes execution coverage and
commands reached. It assigns no physical passes. Preserve first outputs and
freeze any corrected or condition-specific follow-ups separately.

`monitor.py NEW_EVIDENCE_DIRECTORY` watches only the already-dispatched run,
without dispatching or executing chemistry. It freezes its ingestion inputs,
authenticates the eventual artifact and automatically invokes the execution
reader when all 200 records are present. It preserves partial artifacts and
terminal failures rather than rerunning cases. Its bounded polling interval is
30 seconds, for at most 480 polls; inspect its state before restarting collection.

A separately bound VPS run attempted ranks 1–13 on 2026-10-15 in 3.61 seconds
combined case time: twelve zero exits and one identity refusal (`Na`, rank 11).
The memory gate stopped execution before rank 14. All thirteen saved stream
hashes were verified. See the [partial receipt](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/vps-partial-execution.json).
Ranks 14–200 remain unexecuted locally; resume only unfinished cases under fresh
resource headroom. The independent hosted run subsequently completed all 200 attempts. These execution
results assign no physical passes and do not validate the generated procedures.

## Explicitly requested production API comparison

**Complete:** all 200 unchanged scripts were sent through the public production
API, one request each. All 200 returned HTTP200: **92 completed, 108 refused**.
Execution status and full parsed observations match the verified CLI baseline
for **all 200**. There were no transport/server errors or rate-limit retries.
Only the 92 completed scripts reached every addition. Median request time was
3.27 seconds; the paced batch took 12 minutes 38 seconds. This establishes API/CLI
agreement for the pinned engine, not scientific validation of the reactions.

[Verified completion](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/api-comparison/completion.json),
[case-level assessment](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/api-comparison/assessment.json),
[CSV](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/api-comparison/summary.csv) and [binding](https://github.com/CrispStrobe/kerotakis/blob/51beacc0b1bf018b8f3bd0a4c7cdc70b13d1cca6/audits/reaction-dataset-200-20261015/api-comparison/binding.json)
preserve exact API/source/binary/script identities and response hashes. A
separate reader rechecked every saved HTTP attempt and both CLI streams.
The API comparison is finished; do not run it again without a new purpose.


The user subsequently requested all 200 protocols through the deployed API.
`run_api.py PRIVATE_KEY_JSON NEW_OUTPUT_DIRECTORY VERIFIED_CLI_EXECUTIONS`
submits the exact frozen script bytes to the public production endpoint. This is
a separately bound transport/runtime comparison, not a repeat native CLI batch.
The initial health identity must match the earlier CLI binary; later health and
response checks reject version drift. Both current source JSON hashes match the
frozen input inventory. No protocol or physical forecast is rewritten.

Requests are serial, at least 3.1 seconds apart, respecting both API admission
and the initial Vercel Sandbox allocation quota. Only explicit 429 admission
refusals may retry; ambiguous failures retain their evidence without blind
re-execution. Every attempt preserves its body, selected non-secret headers,
status, timing and hashes. Credentials never enter output or Git. Large evidence
belongs in private storage; the client performs no native chemistry locally.
The final report compares execution status and full parsed observations with the
already verified CLI streams. API completion is not a scientific pass.

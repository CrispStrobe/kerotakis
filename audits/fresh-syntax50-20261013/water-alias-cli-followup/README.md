# Corrected alias CLI follow-ups

Two new source-informed controls replace only the invalid final `balance v1`
command with the documented `measure v1 balance`. Original scripts, predictions,
process failures and reader crash remain [preserved](../water-alias-cli-first-outcomes.json).
Quantitative expectations are unchanged. These controls are not new blind designs.

The saved-output reader now checks the named stock creation/exhaustion events;
CLI bench JSON exposes vessels, not the stock map. Shared finite balance is
observable in those events; map cardinality is independently covered by native
regressions. See [protocol](protocol.json), [expectations](expectations.json) and
[consumer](analyse.py). Current state: **2/2 executed and passed**. The [outcome receipt](../water-alias-cli-followup-outcomes.json)
preserves actual streams/report hashes and the unchanged numerical checks.

[Hosted follow-up](../water-alias-cli-followup-dispatch.json) runs the
[pinned helper](../hosted-water-alias-cli-followup.py) on audit source `3632f581`.
Run `38614113822`, job `116024670167`, completed successfully; its other 25 jobs are skipped.
It reuses the verified CLI from core source `e4fb5e33`, verifies bytes/source,
and checks fresh worker resource gates before each case. No app is rebuilt.
On completion, preserve raw streams and process records before judging the
unchanged forecasts. Record producer/consumer/protocol hashes and actual source.
Any first failure must remain intact before a new version is prepared.

Both processes exited zero with empty stderr. Final canonical water inventory
is 0.5 mol, and the shared bottle reports 0.5 mol remaining when refusing the
0.75 mol overdraw. Later observations preserve inventory; both semantic output
streams are equal. The saved-data reader independently reproduced the archived
worker report. Selected ZIP members have verified CRCs; raw stream hashes and
the report hash match producer records and the actual job marker. The full result
ZIP has not been downloaded or byte-verified. No new CLI execution was needed.

These passes apply to unchanged core source `e4fb5e33`; they do not establish
staged app acceptance. The original pair remains two failed controls, and all
blind fifty forecasts and normalized forty remain unchanged.

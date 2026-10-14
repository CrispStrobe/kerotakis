# Corrected alias CLI follow-ups

Two new source-informed controls replace only the invalid final `balance v1`
command with the documented `measure v1 balance`. Original scripts, predictions,
process failures and reader crash remain [preserved](../water-alias-cli-first-outcomes.json).
Quantitative expectations are unchanged. These controls are not new blind designs.

The saved-output reader now checks the named stock creation/exhaustion events;
CLI bench JSON exposes vessels, not the stock map. Shared finite balance is
observable in those events; map cardinality is independently covered by native
regressions. See [protocol](protocol.json), [expectations](expectations.json) and
[consumer](analyse.py). Current state: 0/2 executed. Preserve any first failure.

[Hosted follow-up](../water-alias-cli-followup-dispatch.json) runs the
[pinned helper](../hosted-water-alias-cli-followup.py) on audit source `3632f581`.
Run `38614113822`, job `116024670167`, is queued; its other 25 jobs are skipped.
It reuses the verified CLI from core source `e4fb5e33`, verifies bytes/source,
and checks fresh worker resource gates before each case. No app is rebuilt.
On completion, preserve raw streams and process records before judging the
unchanged forecasts. Record producer/consumer/protocol hashes and actual source.
Any first failure must remain intact before a new version is prepared.

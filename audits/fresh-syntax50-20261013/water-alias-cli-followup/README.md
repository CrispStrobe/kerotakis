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

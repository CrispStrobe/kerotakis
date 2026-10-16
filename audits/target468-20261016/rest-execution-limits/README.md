# Native execution limit diagnosis

The [first API failure](../rest-api-all668/first-api-failure.json) is preserved:
PR-005 received HTTP503 with native SIGKILL, while its separately recorded CLI
execution completed. The cause is not yet established. QA-010 later also
received HTTP503; it remains a separate first API outcome.

The prospective [diagnostic design](design.json) freezes the exact original
script, engine and supervisor hashes. The [observer](vm_job_observer.py) changes
only trusted post-reap reporting of child CPU user/system time and maximum RSS.
The [runner](run_diagnostic.py) uses a fresh copy of the same Vercel snapshot,
with unchanged isolation, eight CPU seconds, ten wall seconds and one GiB child
address-space limits. It receives no API or admission-store secrets. There is
no blind API retry, engine replacement, limit relaxation or scientific verdict.

Execute this separately bound diagnostic after the668-case API batch finishes,
retaining its issued marker and raw result. CPU near the configured ceiling
would support CPU exhaustion for the reproduction; completion or an ambiguous
kill would not establish the original failure's cause.

## Diagnostic results and bounded repair

The [PR005 diagnostic](pr005-diagnostic-result.json) completed under unchanged
limits:6.37CPU seconds and99024KiB maximum RSS, with exact saved native JSON.
It does not establish why the original API request received SIGKILL. Its first
launcher failed before any VM was created because its interpreter lacked the
SDK; the existing API environment supplied that prerequisite. The first captured
traceback was not retained, and that evidence limit is recorded explicitly.

The separately frozen [QA010 diagnostic](qa010-design.json)
[reproduced SIGKILL](qa010-diagnostic-result.json) at8.012676CPU seconds and38152KiB
maximum RSS under the eight-second hard ceiling. This supports CPU exhaustion
for that reproduction, not an OOM claim or a retroactive cause for every kill.

[PR817](https://github.com/CrispStrobe/kerotakis/pull/817) at266121dd proposes a
finite12CPU/18wall budget. Child memory, descriptors/tasks, output, authentication,
shared admission and network/filesystem/privilege isolation stay bounded. Trusted
post-reap CPU accounting identifies a hard-CPU timeout; other signals retain
an undetermined cause. Total Redis3s and forwarding50s deadlines prevent slow
streams from exceeding nominal inactivity timeouts. SDK/process/VM deadlines,
the60s platform deadline and65s browser timeout include admission and cleanup.

Twenty-three lightweight API/accounting/admission/packaging groups and maintained
REST-client/service-worker controls pass. Independent
[source review](budget-independent-review.md) preserves both timeout findings
and their corrections. A new immutable snapshot has
[passed smoke and strengthened actual isolation](budget-snapshot-acceptance.json),
including the12CPU/1GiB child limits. Its supervisor hash matches the final API
source; the accepted native binary remains c446/670. There is no new Rust engine
or latest-main chemistry claim. Full native CI and the exact-head own fleet remain
required. The [two changed-policy API follow-ups](budget-api-followup-design.json)
are separately frozen; original668 first outcomes remain unchanged.

The [changed-policy API results](budget-api-followup-results.json) now complete
both original failures once on gateway266121dd, with the same scripts and native
engine. Both exit0 and exactly match their saved native JSON observations. Four
live over-budget/session controls still refuse. These separately bound results
close the two measured delivery failures; they do not replace the original
555/111/2 first outcomes or establish668 executions on the new gateway. The111
CLI refusals and scientific/model coverage limits remain separate work.

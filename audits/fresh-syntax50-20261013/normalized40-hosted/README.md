# Original forty spelling follow-ups — hosted execution

The [dispatch](../normalized40-hosted-dispatch.json) selects one bounded worker
and excludes the other 26 jobs. It verifies the original release artifact,
source/tree association, full ZIP/member/binary/lock checksums and every frozen
input before invoking the unchanged runner. Fresh worker resource gates apply
before metadata/download and every case. No application is rebuilt.

The old-source binding remains main `30fd09eb`, tree `25d1dd49`, executable
`b290b436`. This isolates spelling before subsequent repairs. See the
[protocol](protocol.json) and [component hashes](hosted-binding.json).
Preserve first raw streams, stderr, exits, timeouts and resource refusals.
The worker records process outcomes; those are not physical pass/fail verdicts.

Current state: dispatched, no completed cases yet verified. Do not duplicate
this run. Review saved observations against the unchanged physical forecasts
before classifying. A current-main replay needs its own new source binding.

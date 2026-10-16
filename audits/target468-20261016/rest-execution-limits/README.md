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

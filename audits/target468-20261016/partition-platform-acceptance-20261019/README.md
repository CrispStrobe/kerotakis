# PR841 protected/platform acceptance and normal integration

All25 CI jobs pass for exact head `d6720509`. Both actual native-platform logs
contain every one of the23 required named regression controls, including invalid
coordinates, atomic drain/extraction refusal, the ordinary two-phase positive,
new interaction-factor underflow/overflow boundaries and peroxide-medium controls.
[Raw-log hashes and exact names](acceptance.json), [actual CI](exact-head-ci.json)
and the [reviewed candidate/protection snapshot](reviewed-candidate.json) are bound.

The platform logs were read with bounded streaming buffers and a strict2MiB limit
per log. Actual logs are approximately600KiB each. No test assertion, native
execution, protected status or applicable own-fleet acceptance was substituted.
The prior accepted33-job CPU/source proof remains separately bound. Large source,
archive, build and browser work keeps its existing resource gates.

PR841 merged through normal exact-head auto-merge as `dacf76513328c2226e2f5133a79fb9af84343431`.
[Actual parents and reconstructed tree](../accepted-pr-integration/pr841-integration.json)
verify parents `0b579699` and `d6720509`, with actual tree `76820b9c` exactly equal
to the validated candidate. No admin bypass, force push or test waiver was used.

Changed production artifact/API/browser acceptance remains separate. The
engineering checks add no scientific-catalog selection and do not resolve the
missing ester-data or unknown-neutral transfer domains.

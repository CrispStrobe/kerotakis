# Independent source review: bounded REST runtime budget

Reviewed initial head `8cd77370a2608488aae7fc7303e2b9be68db3a59` against
`828c09b9a32b5ae1e22b9f782920a33ec299a6d0`, then the required forwarding correction
at **`6a0e182cad0c63687a150664cd73c4dfb1dceaf5`** against that initial head.
No remaining material source blocker found at the corrected head. This is a
source-only review, not activation or live acceptance. No engine, API, browser,
C compilation, tests or builds were run for this review. No product files changed.

## Evidence and finite resource policy

The separately preserved QA-010 diagnostic records SIGKILL with
7.928582 user plus0.084094 system CPU seconds, total8.012676s, under the old8s
hard ceiling; reported peak RSS is38152KiB. This supports CPU exhaustion for
that changed-observer reproduction. PR-005's diagnostic completed with
6.271279+0.099941=6.371220CPU seconds and99024KiB RSS; it does not establish the
cause of PR-005's original SIGKILL. Neither diagnosis supplies a chemistry pass
or permission to overwrite the first API outcomes.

The repair raises the finite child budget to12CPU/18wall seconds in the direct
worker and microVM supervisor, without changing the1GiB child address-space cap,
2GiB VM allocation,64 descriptor/task caps, output budgets, network denial,
filesystem/privilege restrictions, auth, global slot or request quotas. The
source exposes the new CPU/wall constants through capabilities. The immutable
snapshot must contain the matching supervisor; changing the API alone cannot
update an old snapshot's limits.

`worker.py` installs the12s soft and hard CPU limits. The microVM child installs
matching CPU limits and unchanged address-space/descriptor/task restrictions
before dropping privileges and executing the engine. The C isolation probe now
requires both12s CPU ceilings and both1GiB address-space ceilings, in addition
to its existing isolation assertions. The fake-engine test checks actual inherited
CPU/memory limits through both existing direct-worker paths. Its separate signal
check uses direct execution: it correctly avoids assuming Bubblewrap propagates
the same negative return code as its child.

## CPU accounting and cleanup

`vm_job.py` is launched as a fresh supervisor process for one child in a fresh
nonpersistent VM. It does not launch earlier helper children; hash verification
is ordinary parent-process file IO. The engine child cannot fork a new process;
permitted clone flags are restricted to its thread group. Thus post-reap
`RUSAGE_CHILDREN` identifies the one execution child's lifetime user/system CPU
and peak RSS under this architecture. It must not be treated as per-request
accounting if this supervisor is later reused across jobs or given other helper
children. Peak RSS is Linux child-lifetime RSS, not a complete VM/parent-memory
measurement or a proof of OOM cause.

The CPU classification requires all of: supervisor status `completed`, native
code `-SIGKILL`, and trusted reaped user+system CPU at least12s. Wall/output-limit
statuses, other signals and kills below that ceiling keep their distinct paths.
No empirical tolerance is added and SIGKILL alone remains undetermined. The
result records that the configured CPU budget was exhausted; these observations
need not establish the identity of an external signal sender. The unit matrix
covers threshold crossing, below-threshold kills, other signals, normal exits
and supervisor-enforced limit statuses.

The existing cleanup still kills the process group, cancels/drains pipe tasks,
reaps the child and closes the VM context. Resource usage is read after reap.
CPU exhaustion becomes timeout with no fabricated partial scientific result;
other signal failures still reach PR816's undetermined-cause503 branch. Existing
cancellation/output cleanup contracts are retained. Source review of those paths
does not prove actual remote VM deletion or a completed live isolation run.

## Deadline finding and reviewed correction

The initial head's `httpx.AsyncClient(timeout=40)` was an inactivity/per-operation
timeout, **not a total40s forwarding deadline**. A slowly streamed response could
exceed the advertised forwarding/browser ordering. This was a concrete gap in
the proposed deadline contract, even though the child and VM limits were finite.

The corrected head wraps the complete client/stream/JSON-validation operation
in `asyncio.timeout(40)` and maps its `TimeoutError` to the existing nonreflecting
502 failure. Response/client context managers unwind on cancellation. Its new
mocked drip-response regression accelerates only the40s outer deadline, asserts
502 and verifies stream closure. No credentials, redirect policy or output-size
limits are weakened. That correction changes only `app.py` and its test module
relative to the initially reviewed head.

The resulting ordering is child18s wall; supervisor-process kill21s; SDK
process deadline23s; VM lifetime30s; complete VM lifecycle35s; total optional
forwarding40s; browser abort45s; function allocation60s. Packaging source sets the
60s function allocation. These nested deadlines leave nominal room for cleanup;
platform startup/teardown delays can still produce honest execution-unavailable
responses. No guarantee that every valid script completes follows from the
budget increase.

## Pins and acceptance still required

| File | SHA256 at corrected head |
| --- | --- |
| `services/rest_api/app.py` | `57b13b6afa9e010bd52daa3773aacab0e4af8558eb35d59b422d5a27d4f12712` |
| `services/rest_api/test_api.py` | `ad2ebd9db29e9ecd2b218589a82df2885b7def9d3854803c9cf4c369b55cae55` |
| `services/rest_api/vm_job.py` | `299e61560ed46b465b666a276a78a3664e6d1dbed1e681382544419e1b2cab70` |
| `services/rest_api/worker.py` | `6598ce05fc94cf6d11e9458b3896196e840467e85e707ed2ce37bc2c4715ae07` |
| `services/rest_api/probe.c` | `c4a29b3cf066b0c5f014bacbd3e7e771204654726abe304604d990762ab9429c` |
| `web/rest.html` | `aeff83ae7d1a67d57418ca534eb83b84a1a14dcd92862aef8c57aa5488beb71d` |
| `tools/test-rest-client.mjs` | `38aed5c864d8684e537e5a36586d7cd4d2f86973d2578cee52a22788753ba3c9` |

Both reviewed diffs pass whitespace checking. The snapshot supervisor policy is
unchanged by the final forwarding-only correction. Normal exact-head hosted
checks, actual compiled isolation probes, a new immutable snapshot and bound
live isolation/delivery acceptance are required before activation. Neither this
review nor earlier limits diagnosis establishes production readiness, repairs
unsupported chemistry or changes a frozen scientific forecast. Original failed
protocols remain separate from corrected controls; no last-vessel engine invariant
change is reviewed or implied here.

## Deadline-margin supplement: head2b9600f9

Reviewed `2b9600f9143f50c0e5cc0d2e8063a690bdd4c9d3` against the previously
reviewed `6a0e182cad0c63687a150664cd73c4dfb1dceaf5`. The five-file delta changes
total forwarding40→50s and browser45→65s, with matching tests and README. It
retains the actual outer forwarding deadline. No child, supervisor or snapshot
policy changes occur. `vm_job.py` remains SHA256
`299e61560ed46b465b666a276a78a3664e6d1dbed1e681382544419e1b2cab70`.

The intended nominal budget is35s VM lifecycle +5s body +3s admission +3s
release =46s, followed by forwarding50s + gateway body5s =55s under the60s
function limit, with browser65s outside it. The forwarding/browser changes are
coherent, but **one claimed upper bound remains unestablished at this head**:
`RedisAdmission.command` uses `httpx.AsyncClient(timeout=3)` without an outer
`asyncio.timeout`. This bounds inactivity/individual IO operations, not total
admission/release time. A drip-fed bounded response can therefore exceed3s.
The README's “up to3s” cannot yet establish that46s worker upper bound.

The narrow repair is an outer3s deadline around the complete Redis command,
stream consumption and decoding, mapping `TimeoutError` to the existing503
admission-unavailable contract. Release should retain its existing fail-closed
lease-expiry behavior. A mocked drip-response/stream-close test can verify the
contract without an engine. Alternatively the documentation must qualify these
as nominal IO budgets, but then46s is not a proven total maximum. This finding
does not weaken the independently finite child/VM/function limits.

At2b9600f9, `app.py` SHA256 is
`0c7c87ac965c5632e16585239a05ce3e47792529bee40984ff5663ead1435067`,
`test_api.py` is `e6362e134ce43e959e012be509b755710f8eaac41daee3d6144156138ea32296`,
`web/rest.html` is `d4d3fbfded5a3d404f91e3dea707a4cc197a9dfd8cb3d64f2cd6f604b87f4a21`,
and `tools/test-rest-client.mjs` is
`ad058105fda86baf543eeae9c0321392787b8a2d775a83f2f90bbb4c98d0de33`.
The diff passes whitespace checking. Source-only review did not run tests,
probes, API calls or snapshot creation. Separately reported snapshot smoke/live
isolation acceptance from6a must retain its own receipt and qualify the final
API head separately; unchanged supervisor bytes do not imply equality of all
API source bytes. First lifecycle-protocol failure remains historical and is
not repaired by changing the engine's last-vessel invariant.

## Redis deadline resolution and final network audit: head266121dd

Reviewed exact final correction
`266121dd5605a439a495c85f2401fbeacd6272f4` against
`2b9600f9143f50c0e5cc0d2e8063a690bdd4c9d3`. The only changed files are the
Redis command implementation in `app.py` and its test module. The entire
Redis client, streamed response and JSON decoding now sit inside
`asyncio.timeout(3)`; `TimeoutError` becomes the existing nonreflecting503
shared-admission-unavailable response. Acquire cannot fall back to local state;
release keeps its existing expiring-lease fail-closed behavior. Its mocked drip
regression accelerates only that3s timer, checks acquire503 and release behavior,
and requires both streams to close. No child or snapshot policy changes occur.
The remaining margin-contract finding at2b9600f9 is resolved.

An independent source search of all network call classes in `app.py` found:

| Network path | Total bound and scope |
| --- | --- |
| Shared Redis admission/release | Outer3s includes client/request, response stream and decoding;8192-byte response cap; redirects and environment proxy configuration disabled. |
| Optional upstream forwarding | Outer50s includes client/request, full response stream and result validation; existing2MB response cap, fixed destination and redirect refusal retained. |
| Vercel SDK session/create/write/process/lifecycle | Outer35s encloses the session and VM context, including script upload and process/result handling; process call also has23s timer and21s kill-after; VM lifetime30s remains independent. |
| Incoming ASGI request body | Outer5s encloses stream consumption with the existing20KB cap. Authentication is local; no added network authentication lookup. |

No additional outbound HTTP/socket request class lacking a total deadline was
found in this module. Network-denied engine execution and subprocess limits are
separate from those parent-side network timers. These cooperative application
deadlines do not replace the60s platform deadline, prove actual remote deletion,
or guarantee scheduling/teardown latency. The browser65s deadline permits the
bounded worker/function response to finish before the browser aborts under the
intended deployment policy.

Final `app.py` SHA256:
`f1a41301384ae3987d7e2d387b3134e6a8a0cfe64771bc3058b562e3edf8e8ce`.
Final `test_api.py` SHA256:
`9265db915ab663e260a32c1b81d547687d776286b7e2132d7d638129a62d017d`.
`vm_job.py` remains the previously reviewed
`299e61560ed46b465b666a276a78a3664e6d1dbed1e681382544419e1b2cab70`.
The correction passes diff whitespace checks. **No remaining material source
blocker found at266121dd.** Original findings and their source revisions above
remain preserved. This final supplement is source-only: no tests, network/API
requests, builds, probes or snapshot actions were performed by this reviewer.
Normal exact-head CI and separately bound snapshot/live acceptance still own
activation; none of this changes chemistry forecasts or first failures.

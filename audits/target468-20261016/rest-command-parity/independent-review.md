# Independent source review: REST PR815 and PR816

No material source blocker found in the requested diffs. This review supports
source readiness only. It does not claim tests, official parser execution,
native/API/browser acceptance, deployment or chemical validation. No product
files, tests, catalogues or worktrees were changed; no engine, API, build or
browser was executed.

Reviewed exact revisions:

- PR815: `c96c821a54c53b3167ede3a2b5cb6000253d5ae2` →
  `735c523d415e1e8ab1dddf58efbbf6d7bc53266e`.
- PR816: `735c523d415e1e8ab1dddf58efbbf6d7bc53266e` →
  `0651e64e0e4df4b4128b2031ac82febed0ba6007`.

## PR815: command admission and capabilities

Static extraction of `script.rs` and the Python AST confirms **35 canonical
verbs plus eight synonyms/spellings**, all admitted. The policy has46 words in
all; its three additional words are the existing JSON-capable shell commands
`register`, `inspect` and `explain`. Actual CLI source distinguishes JSON
`explain` from the excluded plain-text/no-observation shell facilities.

The additions `stock`, `remove`/`discard`, `smell`/`waft`, `zoom` and `voltmeter`
match the engine inventory. The small structural guards for their arguments do
not introduce an alternative physics implementation. Malformed engine syntax
can still receive an engine refusal; API admission is not the complete Rust
parser. The new tests cover every inventory example and all alias words against
that source inventory rather than merely comparing two copied lists.

`MULTIWORD_INGREDIENT_ALIASES` contains exactly the engine's two multiword
identities, `carbon dioxide` and `sodium chloride`. Its exact case-sensitive
matching agrees with `canonical_ingredient_line`, not the separate
case-insensitive reservation check used to prevent pack identity collisions.
Positions match stock1, add/grind/titrate2 and extract3. Collapsing these aliases
only for admission accounting leaves the submitted script unchanged. Other
single-token aliases cannot move parameter positions. Both current multiword
aliases have equal width and different prefixes, so the policy's longest-first
selection agrees with the engine's longest-match rule.

Loop admission continues to consume the complete normalised suffix. Extraction
and both distillation spellings admit only an optional bounded `stages` pair;
titration recognises the engine's concentration/default, pH/pe/colour endpoints
and optional bounded `max` pair; transport uses a full canonical-chain match
with bounded cells, steps and Courant value. Their aggregate work is checked
before execution. Existing duration and child-process limits remain in force.
The added aliases do not bypass those positions or budgets. Tests include
multiword titration/extraction, excessive counts, repeated suffixes and aggregate
work. Language directives remain excluded, preventing an English-only admission
parser from being paired with later localised execution.

`/capabilities` reports the static admission policy and the same request/loop/
duration constants. Its eight CPU seconds and ten wall seconds agree with the
existing direct worker and microVM supervisor. It includes neither credentials
nor private configuration. The README correctly qualifies this as admission
policy, with `physical_verdict: not_assigned`, rather than promising model
coverage or runtime readiness. Unavailable session commands receive explicit
non-reflecting refusal. The existing test module is extended, so no new test-file
discovery change is required.

## PR816: signal diagnostics

The added branch applies only to a completed supervisor result with a negative
child exit code. It runs **before JSON decoding**, so a truncated final line
cannot replace an authenticated negative-code diagnosis with a generic malformed
output error. It returns HTTP503 with `engine_terminated`, the positive signal
number and `cause: not_determined`. The source does not infer CPU exhaustion,
OOM or another resource cause from SIGKILL.

Only the numeric signal is logged; script, stderr and private input path are not
reflected in this branch. Ordinary positive CLI refusal, malformed output,
sandbox/bootstrap failure and supervisor-enforced timeout/output-limit retain
their existing paths. The tests cover signals9/11/24 with deliberately partial
JSON and secret-like stderr/path inputs, plus ordinary refusal, malformed output,
bootstrap failure and timeout. Both native-worker and microVM execution converge
on `build_result`; no new child limit, retry or solver behaviour is installed.
This classification does not establish which instruction or startup stage the
child had reached before termination.

## Source pins and validation limits

| Revision | `services/rest_api/app.py` SHA256 | `services/rest_api/test_api.py` SHA256 |
| --- | --- | --- |
| PR815 head | `5e7b415900d577f30cc4f1299c8945785eb66391325f5457f9710cebd88c5dbc` | `bb3d57957a5ec1290ae1005d481c6bd822a80aa971721f98654885467177482a` |
| PR816 head | `c658bba168bee7b0f49a31b02110fe450d97e7dc15c8912eac64c801e41c2b27` | `cb8abfdf3142caac70a50a2b08c1dd76819bf7c1fa8901946d31edd9011a29ba` |

Shared source SHA256 pins:

- `crates/kerotakis-core/src/script.rs`: `279036582bf9270055ab8b96f2ef3f5c9644ce3a58916bef8daf584a3e22bca2`.
- `crates/kerotakis-core/src/species.rs`: `af0992794956454224d6cb176c3805cc5fa91421c266bb63b3953fafda9b7595`.
- `services/rest_api/worker.py`: `3d41012867eb01c317171447116d93d4c53c3cf0d853635e41ee5f73823abcea`.
- `services/rest_api/vm_job.py`: `bb00548fb24e62192fc39a4f633981dceabea17d8542a8f5f3244b6c7e7b429f`.

Diff whitespace checks pass for both requested comparisons. Source/AST inventory
inspection is not a substitute for the normal test suites or bounded hosted
API acceptance. Existing first failures and the running668-case delivery batch
are untouched. Root owns exact-head acceptance, integration and any deployment.

## Workflow supplement: PR816 head828c09b9

Subsequently reviewed exact head `828c09b9a32b5ae1e22b9f782920a33ec299a6d0`
against `0651e64e0e4df4b4128b2031ac82febed0ba6007`. The only changed file is
`.github/workflows/rest-api.yml`: it adds `crates/kerotakis-core/src/script.rs`
and `crates/kerotakis-core/src/species.rs` to **both** pull-request and main-push
path filters. This closes an important wiring gap: future engine grammar or
identity changes now trigger the existing REST acceptance job, including its
source-inventory parity assertions. The job already invokes
`python -B -m unittest services.rest_api.test_api -v` from the checkout root.
No job command, permissions, sandbox, protection, resources or product source
changes in this supplement. Workflow SHA256:
`bb0c1ebc4be2a6f72356714b1c8f8cb6a1c6b5e2d7b65097edcd033025e3ee17`.
The four-line diff passes whitespace checking. No additional blocker found;
normal exact-head CI and bounded delivery acceptance remain required.

# Canonical water input alias — implementation lane

Active after thermal boundary PR786 integrated. Draft PR787 now contains the
core and app follow-up at `98051f7b` on main `8a6aa68e`. The
[application receipt](../audits/fresh-syntax50-20261013/water-alias-app-followup-application.json)
records eleven files and new exact-source CI `38627469049` / own fleet
`38627469041`. [Current macOS native validation passed](../audits/fresh-syntax50-20261013/water-alias-app-candidate-native-validation.json),
including all nine alias regressions, workspace/CODEX/coverage. The
[new own build is source-associated](../audits/fresh-syntax50-20261013/water-alias-app-candidate-own-build-metadata.json);
archive/binary bytes remain unverified. Remaining CI and full fleet are pending.
The [first app suite](../audits/fresh-syntax50-20261013/water-alias-app-first-failure.json)
ran 1,637 tests: 1,636 passed and one visualization coverage guard failed.
The seven new regressions were included; individual passing names are not printed
by the dot reporter. A correction at `d5c6cecf` is staged and unexecuted.

The previous core candidate `e4fb5e33` passed all nine native regressions,
workspace/CODEX/coverage checks, full CI and all 33 own-fleet jobs. Its corrected
CLI pair also passed. Core/CLI/WASM source is unchanged in the app descendant;
retain the previous receipts' source bindings. See the
[native receipt](../audits/fresh-syntax50-20261013/water-alias-native-validation.json),
[fleet completion](../audits/fresh-syntax50-20261013/water-alias-core-fleet-validation.json)
and [CLI passes](../audits/fresh-syntax50-20261013/water-alias-cli-followup-outcomes.json).
Current app acceptance and integration remain pending.
Recheck these call sites against actual integrated main before editing.
The audit branch's bench, vessel and operator implementations include unrelated
changes. Use integrated main as the implementation base; do not transplant those
files from the audit checkout.

The desired behavior is one canonical `water` identity and one stock balance
when users supply either `water` or `H2O`. A lookup-only alias is insufficient:
the parser currently builds addition IDs from the input spelling, while stock
parsing already takes the resolved registry key. The bench debits stock using
the addition ID. Resolving properties without canonicalizing that ID could
create a separate inventory or bypass the finite canonical stock bottle.

Inspect these repository entry points together:

| Entry point | Responsibility and required review |
| --- | --- |
| `crates/kerotakis-core/src/species.rs` | `SpeciesId::new`, transparent serde representation, and `lookup_key`. Resolve input spelling without introducing a second registry species. Constructors alone do not cover deserialized IDs or direct tuple construction. |
| `crates/kerotakis-core/src/script.rs` | `parse_op_typed`, `add`, `stock`, and every ingredient-bearing command, including extraction, grinding and titrant selection. Use canonical identities after resolution; retain unknown-key refusals. |
| `crates/kerotakis-core/src/bench.rs` | Direct `Operator` callers bypass the script parser. Addition deposits, prospective safety state, and stock debits must use the same canonical identity. Review extraction and other identity-bearing operations too. |
| `crates/kerotakis-core/src/stock.rs` | `stock_unit`, stock/unlimit/read/draw operations, and both legacy and compensated ledger deserialization. Canonical reads must not expose an unlimited alias for a finite bottle. Preserve amount precision and units. |
| `crates/kerotakis-core/src/vessel.rs` | Deposit, withdrawal and inventory queries. Verify input normalization reaches these paths without changing solver-generated canonical species. |
| `crates/kerotakis-wasm/src/lib.rs` | JSON `Operator` deserialization and bench snapshot restoration bypass text parsing. Cover browser-facing operations and save/replay. |
| `web/app/src/lib/session.svelte.ts`, `web/app/src/lib/storyStock.ts` | Review shelf identification and persisted story stock against canonical engine output. Determine whether these accept ingredient text independently. |

Implement the smallest shared input-resolution boundary that covers text and
direct operators. Inventory all writers before choosing it; do not assume that
changing `lookup_key` or `SpeciesId::new` covers serde and string-keyed ledgers.
Limit the accepted new spelling to exact `H2O`; broader formula normalization
needs a separate reviewed contract.

The source review suggests one exact-spelling `canonical_input_key` resolver,
used by lookup, species construction/deserialization and all stock key APIs.
Normalize owned operators before dispatch, safety assessment, application and
logging to cover direct tuple construction as well as JSON/text inputs. On the
reviewed production source, the identity-bearing operator fields are addition
species, extraction solvent, grinding species and titrant, plus the string key
for stocking. Recheck that inventory after integration. Direct vessel deposit,
withdrawal and query APIs also need consistent canonical identities.

For both legacy and compensated stock restore, reject a map containing both
`H2O` and `water` before constructing the ledger. Canonicalize a single alias
without changing its authoritative amount, unit or compensated components.
Inventory portions and provenance lots can retain separate entries under one
canonical species; preserve their quantities rather than silently dropping or
merging lots. A rejected WASM restore must preserve the live bench. Frontend review found that raw text is used before the canonical engine output
arrives. This bypasses Story access and usage gates for aliases. The staged
follow-up uses non-mutating typed engine parse output before those gates and
canonicalizes mission-kit keys through the same parser, without a second product
alias table. Only actual addition/material addition or finite positive-volume
`titrated` events consume a Story dispense; veto, exhaustion and zero-dose
endpoint responses must not consume one.

Acceptance must exercise equivalent `water`/`H2O` additions in moles, grams and
milliliters; mixed-spelling finite stock, exhaustion and unlimit operations;
direct operators; extraction/transfer; snapshots and restore/replay. Assert
canonical output identities and equal quantities, one inventory identity and
one bottle, unchanged safety-veto accounting, and unchanged unknown-key refusal.
If restoring a snapshot with both spellings is supported, define its collision
policy explicitly and preserve quantities and compensated precision; otherwise
reject it explicitly before mutating the bench. Silent overwrite is unacceptable.

Freeze the alias acceptance inputs before execution on the resulting source.
Keep the original blind fifty and original-source normalized forty unchanged.
Run bounded tests only when resource gates permit, then record actual source,
checks and limitations in the ordered agenda.

## Immediate acceptance lane

Read the [staged follow-up receipt](../audits/fresh-syntax50-20261013/water-alias-story-followup.json)
for source `98051f7b`, the four app files and seven new tests. The
[candidate acceptance binding](../audits/fresh-syntax50-20261013/water-alias-app-acceptance-binding.json)
records its exact tree, eleven-file combined scope, and confirms that core/CLI/WASM
source is unchanged from the earlier candidate. This supports the earlier kernel
evidence without claiming complete app acceptance. The seven tests ran in the
all-app suite; its sole failure is recorded separately in the visualization guard. Independent source review caught a titration regression in the first
staged predicate; the correction now uses the engine's `titrated.total_volume`
and the fake emits the real event shape. Positive, zero-volume and unsuccessful
responses have separate controls.

1. The previous-source CI and own fleet completed successfully, and PR787 was
   fast-forwarded to `98051f7b` after exact head/base/main guards. Keep those
   receipts and the [application record](../audits/fresh-syntax50-20261013/water-alias-app-followup-application.json);
   do not repeat that completed transition.
2. Preserve the [first app failure](../audits/fresh-syntax50-20261013/water-alias-app-first-failure.json)
   at `98051f7b`. Its visualization guard scanned non-visual stock event comparisons
   as drawing handlers. Correction `d5c6cecf` moves the predicate to `storyStock.ts`
   and adds a direct predicate test without changing the original guard or the
   seven prior regressions. The combined scope stays eleven files.
3. Let CI `38627469049` and own fleet `38627469041` finish. Then refresh
   head/base/main, fast-forward PR787 to the reviewed staged descendant and
   require fresh all-app/type/browser/native/full-fleet acceptance. The added
   eighth app regression is unexecuted. Do not cancel or duplicate active runs;
   do not merge the known-failing `98051f7b` candidate.
4. Review the [two corrected CLI passes](../audits/fresh-syntax50-20261013/water-alias-cli-followup-outcomes.json)
   and retain their raw stream/report bindings. These controls are complete;
   do not repeat them on unchanged source. Their source remains `e4fb5e33`;
   record that limit rather than relabel their binary as the app follow-up.
5. Integrate only after exact-head checks and current merge guards pass. Verify
   the integrated tree, then advance to the forty naming follow-ups in the
   [ordered agenda](improvement-agenda-20261014.md).

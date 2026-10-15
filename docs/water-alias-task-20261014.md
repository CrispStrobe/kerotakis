# Canonical water input alias — implementation lane

Active after thermal boundary PR786 integrated. Draft PR787 now contains the
corrected core and app follow-up at `d5c6cecf` on main `8a6aa68e`. The
[guarded application](../audits/fresh-syntax50-20261013/water-alias-accounting-followup-application.json)
records eleven files and fresh exact-source CI `38673902939` / own fleet
`38673902811`. Eight new app regressions and full corrected-source acceptance
remain pending.

Previous source `98051f7b` [completed all 33 own-fleet jobs](../audits/fresh-syntax50-20261013/water-alias-app-candidate-completion.json).
Its CI completed with 23 passes, two failures and one skip. Both failures were
the same visualization guard scanning non-visual stock accounting as drawing.
The first all-app suite had 1,636 passes and one failure, with the seven original
new regressions included. Passing names are not printed by the dot reporter.
The applied correction moves the predicate into `storyStock.ts`, keeps the
original guard and scene classifications, and adds one direct predicate test.
Preserve the [first failure](../audits/fresh-syntax50-20261013/water-alias-app-first-failure.json)
and both original CI job logs.

The previous source's [nine native regressions and workspace/CODEX/coverage passed](../audits/fresh-syntax50-20261013/water-alias-app-candidate-native-validation.json).
Its [own archive and binary are verified](../audits/fresh-syntax50-20261013/water-alias-app-candidate-artifact-review.json),
and executable/lock bytes equal the earlier verified CLI. Core/CLI/WASM source
is unchanged by the app accounting correction. Keep each receipt's actual source
binding; this supports kernel evidence without claiming corrected app acceptance.

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

1. Both previous runs finished, and PR787 was normally fast-forwarded from
   `98051f7b` to `d5c6cecf` after fresh head/base/main guards. Keep the
   [application record](../audits/fresh-syntax50-20261013/water-alias-accounting-followup-application.json)
   and previous completion/failure receipts; do not repeat that transition.
2. Monitor fresh CI `38673902939` and own fleet `38673902811`. Preserve any
   first failure and its exact source/log before fixing it. Do not cancel or
   duplicate active candidate runs.
3. Require actual corrected-source all-app Vitest, including all eight new app
   regressions and the unchanged visualization guard, plus type/browser/native
   checks and complete own-fleet acceptance. Confirm source/tree associations;
   previous native/CLI passes do not establish current app acceptance.
4. Review the [two corrected CLI passes](../audits/fresh-syntax50-20261013/water-alias-cli-followup-outcomes.json)
   and retain their raw stream/report bindings. These controls are complete;
   do not repeat them on unchanged source. Their source remains `e4fb5e33`;
   record that limit rather than relabel their binary as the app follow-up.
5. Integrate only after exact-head checks and current merge guards pass. Verify
   the integrated tree, then advance to the forty naming follow-ups in the
   [ordered agenda](improvement-agenda-20261014.md).

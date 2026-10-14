# Canonical water input alias — implementation lane

Queued after thermal boundary PR786 integrates. This is a source review and
task specification, not an implemented fix or executed acceptance result.
Recheck these call sites against actual integrated main before editing.

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

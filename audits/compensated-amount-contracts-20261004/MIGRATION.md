# Compensated amount migration boundary

The standalone `Amount` primitive is a foundational numerical tool. It does not alter any current `Moles`, `Portion`, vessel, stock, spill, solver, or still inventory. Its tests cannot establish closure of those production accounting gaps.

## Quantity contract

An amount is a finite nonnegative value represented by a normalized pair of binary64 components, `hi + lo`. `lo` can be signed. Scalar projections can round away `lo`; code must not infer exact inventory equality from projections. Two components provide bounded precision rather than arbitrary exact arithmetic. In particular, a correction beyond the low component's resolution can still be lost. Subnormal product residuals are not generally representable.

Before production adoption, specify the acceptance error of every operation, including scaling and multi-component reduction. Distinguish a requested transfer, its conserved compensated ledger amount, and its projected scalar amount. Existing `1e-14` still cuts and their latent-energy contract must remain supported. A strict scalar-relative debit guard would violate those frozen contracts.

## Ownership and API

Prefer authoritative compensated amounts within owned inventory entries, with scalar projections confined to model boundaries. A separate species/phase residual map is insufficient: existing code mutates or replaces inventory vectors. Such a sidecar can remain after the matter it described has changed. Duplicate species/phase portions also make sidecar identity ambiguous.

Adding a field to public `Portion` changes Rust struct-literal APIs. Replacing the existing public tuple `Moles(f64)` changes `.0` access and construction throughout the workspace. Choose and document that API migration explicitly. An additive serde default is only save compatibility, not Rust source compatibility.

Legacy scalar saves load with a zero low component. New saves must persist both components and validate normalization, finiteness, and nonnegative totals. Old readers must not silently discard nonzero residuals from new snapshots; use an explicit schema/capability boundary if old-reader behavior cannot be rejected reliably.

## Required mutation coverage

This is a map of known mechanisms, not an exhaustive claim. Search the complete workspace again when adoption begins.

- `vessel.rs`: `deposit`, `withdraw`, and `withdraw_phase`; aggregate readers, heat capacity, mass, volume, and quantity exhaustion. Lots contain separate provenance amounts and need a defined relationship to authoritative portions.
- `bench.rs`: checked transfer split/deposit; decant/mix/filter/magnetic transfer; spill creation/recovery/discard; titration quantities and water/titrant coalescing; extraction; distillation; inventory vector replacement; headspace exchanges.
- `delta.rs`: direct scalar subtraction and deposit in `StateDelta::apply`, replacement snapshots, candidate validation, adsorbed and electrode inventories.
- `solve.rs`: aqueous/solid redistribution, lot withdrawals, ownership copies, gas handling, and rebuilt portions. Compensated chemical totals must not be confused with labels for interchangeable phases.
- `kerotakis-phreeqc/src/aqueous.rs`: complete `contents` replacements near current lines 2808, 3121, and 3488, reconstructed portions near line 5008, scaled phase/site quantities, and scalar external-engine exchange.
- Other model integrations, UI/WASM bindings, persistence, cache keys, and stock units: identify every scalar write and projection before claiming integration.

## Solver boundary policy

External solvers return approximate scalar chemical states. Retaining an old per-species residual after reactions can attach matter to the wrong species. Dropping it silently breaks inventory. Before integration, define conserved elemental/charge ownership and a reconciliation tolerance, along with an explicit refusal policy for unsupported reconstruction. Independent custom equilibrators can replace or modify quantities; acceptance must validate their proposed authoritative state. No primitive alone proves chemical conservation.

## Acceptance sequence

1. Validate the isolated primitive, including malformed saves and underflow controls.
2. Introduce authoritative ownership for one narrow nonreacting inventory path. Freeze old-save and new-save contracts before changing it.
3. Test repeated tiny transfers, cancellation, widely separated scales, exact exhaustion, duplicate portions, save/reload, and rollback.
4. Migrate all mutations of that inventory, including reconstruction, before allowing compensated state through those paths.
5. Expand to chemical/solver-owned quantities only after the conservation and reconciliation policy has independent tests.
6. Re-run the preserved still controls plus new requested/debited/receiver/energy contracts. Only then describe the corresponding production precision gap as closed.

## Narrow opt-in stock adoption

Stock is a useful first production owner because its bottle map is private, its quantities do not react, and its write paths are confined to stock replacement, draw, and unlimit. Internal balances can therefore own `Amount` directly. `StockAmount` remains a projected display DTO; `remaining_exact` exposes the authoritative pair separately.

The existing default ledger deliberately refuses scalar-inaccurate withdrawals. Those frozen contracts remain in force. Compensated mode must be selected explicitly through `StockLedger::compensated()`; no existing script automatically opts in. Its stricter exhaustion policy rejects genuine compensated overdraw, including the representational difference in the decimal sequence `0.3 - 0.1 - 0.2`. The legacy mode retains its prior relative last-bit allowance.

Compensated ledgers serialize using the tagged `kerotakis-stock/2` schema, including both amount components. Legacy scalar ledger readers reject the wrapper instead of silently dropping the low component. Legacy map snapshots load in conservative mode and continue serializing in their original shape. An empty compensated ledger also needs persistence; a Bench serializer must omit only empty default-mode ledgers, not every empty shelf.

Compensated draws require an actual authoritative balance change and a before/after debit certificate accurate within `1e-8` relative to the requested amount. Once both components are too coarse for a withdrawal, the draw refuses atomically. This is bounded support, not arbitrarily small debit support. This adoption does not change vessel transfers, still donor quantities, or the default stock precision contract.

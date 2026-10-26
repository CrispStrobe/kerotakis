# Owned runtime diagnostics: implementation scope and acceptance

This is preparation for the last ordered lane in [the current continuation](current-continuation-20261025.md). Finish accepted source/product delivery and unchanged-original review first. Refresh this inventory on the actual implementation head; do not reuse an old commit label as execution evidence.

## Current source and compatibility boundary

Candidate `44552bb153cd97b232145619688431eede419600`, tree `b6ca6e9ab7a518a2d4d9bc2a9b1926576e30f02c`, has the same crate sources as focused source `aafdcd5ad2fb0e212a2dc6c36e49d02b9b5ba8eb`. The aafd syntax inventory has79 NotConverged references (71 constructors/eight patterns) and121 SolverFailed references (13 constructors/108 patterns). These are source-reference counts, not executed failures. A later source change requires a new census.

Keep `SolveError::NotConverged { solver, detail }` and its raw Display; add optional boxed Phrase metadata. Add corresponding optional metadata to `Event::SolverFailed`, with `serde(default, skip_serializing_if = "Option::is_none")`. Every literal Rust constructor still needs the new field. Current error patterns already use `..`; the exhaustive SolverFailed pattern in `crates/kerotakis-core/src/render.rs` needs to bind the field or accept additional fields. Preserve raw third-party/native detail separately from the localized learner reason.

Old event JSON without metadata must deserialize and reserialize to the same object. None must omit the field; Some must round-trip its typed phrase. This does not promise old positional binary Event compatibility. Current tracked Postcard uses concern native cache data rather than Event transport; search again before making any binary-compatibility claim. Preserve the versioned cache layout and existing actual-Wasm controls.

## Implementation lanes

1. **Metadata and propagation.** Update core error/event definitions, all literal constructors and exhaustive patterns across core, PHREEQC, CEA, CLI, Wasm and tests. Add a typed error accessor; copy its phrase at the eight core error-to-event conversion sites in `bench.rs`, `clock.rs`, `orchestrator.rs` and `solve.rs`. Preserve each existing `error.to_string()` detail, solver identifier, severity, rollback and next-route behavior. Give locally generated overlap/invalid-event/conservation/commit failures a reason at their actual typed branch. Test raw upstream failures with metadata absent.
2. **Core reasons.** Convert `acid_endpoint::refused` and `aqueous_continuation::refused` at rejecting call sites. Endpoint domain, reactant debit and proton debit are different reasons; use only the actual species/finite amounts already present. Continuation reasons must distinguish adapter capability, trusted answer, identity, boundary, state/material, charge, sealed crossings, contained-gas mismatch, snapshots, receipt ownership and withheld thermal mutation. Existing NotYetModeled coverage phrases remain coverage phrases.
3. **PHREEQC reasons.** Convert canonical, component-projection, crystal-budget and inventory helpers to typed reason input. Their current English string arguments cannot determine semantic identity. Distinguish the source classes below; keep complete raw detail and every scientific threshold. A projection returning `Ok(false)` is non-applicability, not an invented error and never a scientific pass.
4. **Catalog and display.** Add complete EN/DE/FR entries with equal placeholder sets. LV1 remains the generic failure sentence; LV2/LV3 render a present owned reason and retain separate raw technical detail as appropriate. Use finite numeric slots, registered-species terms and technical text identifiers; do not preformat localized numbers into English strings. Computed/native readbacks are results, not empirical measurements. Unknown user labels and native error payloads must remain identifiable without inventing translations or chemical ownership.

Independent file edits can be delegated after the prerequisites, but the parent must review their combined API and catalogs and run the actual acceptance. No weaker tests, English parsing, generic catch-all reason that erases typed distinctions, or source-calibration claims.

## PHREEQC reason families to cover

| Existing source owner | Required distinct branches and earned slots |
| --- | --- |
| Canonical driver | Pass cap; forced route missing/outside domain; secondary skip/probe/replay transition; nested failure; incomplete current answer; clock parcel empty/mismatched/nonfinite/overdrawn/ambiguous; changed certificate/reservoir/model/heat owner. Pass counts are installed constants; amounts and ionic strength require finite values. |
| Physical comparison | Missing complete state/provenance; changed route; invalid redox/particle/solvent/activity/free-ion coordinate; missing coordinate versus nonfinite coordinate; nonfinite stock/heat; changed immutable opaque or spectator stock. Name a coordinate only where its typed branch identifies it. |
| Event consolidation | Wrong event vessel, invalid gas amount, lost provenance, nonfinite cumulative mineral transfer or neutralization. Preserve signed cumulative crossings and gas-containing narration distinctions. |
| Component projection | Invalid raw total or gas column; absent/invalid protonation split; missing phase/gas readback; invalid initial/returned amount; negative mineral amount; exhausted reservoir endpoint residual; target/readback outside the existing bound. Keep finite numeric residuals and exact existing bounds; never manufacture missing numbers. |
| Inventory certificate | Vessel identity, unresolved formula/amount, unowned/invalid gas crossing, unknown reservoir formula, per-element ledger and charge ledger. Keep clock and reservoir-plus-opaque policies separate. Changed recipe identity/progress or immutable reporter state needs its own reason, not an element-balance label. |
| Native/capability boundary | Cache-only miss without live engine; unavailable dataset/phase/species/model; missing selected-output column versus malformed/nonfinite/negative numerical value; redox balance/search/missing totals; undefined water relation; surface/exchanger/mixed-crystal ownership and capacity; thermal cap. Native engine and injected-hook error bodies remain raw. |

For dynamic helpers, attach an enum/context at the rejecting branch and construct the Phrase from that typed context. Composite predicates may keep an honest composite reason; do not claim one particular failed coordinate without splitting or inspecting the typed predicate. Every value slot must come from the rejected state or an existing constant, and remain finite.

## Completion and delivery

Require exact old/new event JSON controls, unchanged error Display and raw event detail, real error-to-event propagation, None fallback, distinct typed reasons, catalog placeholder coverage and actual German/French failure/capability/unknown-label transcripts. Re-run the appropriate compiler, native, Wasm/browser and own-fleet acceptance on the exact final source. Localization must not change chemistry, routing, numerical budgets or severity.

After independent review and applicable non-protected acceptance, use normal reviewed-head auto-merge under [PR policy](PR-WORKFLOW.md). Authenticate the actual merged tree and changed release, complete its applicable deployed checks, and reconcile maintained status/handoff documents. Only then close the ordered user task. No implementation, new original qualification or deployment is claimed by this preparation.

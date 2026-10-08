# Mechanical withdrawal trace preservation

These ten source-informed controls were written before changing withdrawal code or running them. They are separate from the fifth independent CLI batch and do not count as another blind experiment batch. Starting production source: `605c37366416d23c804ae831ae684833aa390897`.

Observed source issue: `Vessel::withdraw` and `Vessel::withdraw_phase` retain only portions above `1e-15` mol after every call. That globally drops unrelated positive inventory, including during a zero request or a missing species/phase request. A direct mechanical withdrawal must debit the selected owner and preserve other positive owners.

Frozen controls in `crates/kerotakis-core/tests/withdrawal_trace.rs`:

1. Zero species withdrawal preserves the complete vessel.
2. Absent species withdrawal preserves the complete vessel.
3. Zero phase withdrawal preserves the complete vessel.
4. Absent phase withdrawal preserves the complete vessel.
5. A bulk debit preserves unrelated trace salt exactly.
6. A phase debit preserves trace of the same species in an excluded phase.
7. A partial tiny debit preserves its positive remainder.
8. Exact exhaustion removes only the exhausted owner.
9. A tiny owned dose remains fully withdrawable (positive compatibility control).
10. Repeated ordinary debits preserve unrelated trace inventory.

Trace controls span `1e-12`, `1e-15`, `5e-16`, `1e-18` and the smallest positive binary64 value. Assertions use exact equality for mechanically untouched owners and deliberately exactly representable debit relationships. No equilibrium, bulk-plus-tiny subtraction accuracy, compensated inventory migration, provenance-lot reconciliation, malformed-state handling, save schema or whole-engine chemistry guarantee is implied.

The proposed repair removes the global positive-amount cutoff in these two helpers and retains positive portions until actual exhaustion. Existing scalar arithmetic and public APIs remain. Hosted baseline and repair execution plus full required gates must validate compatibility; source inspection is not runtime acceptance.

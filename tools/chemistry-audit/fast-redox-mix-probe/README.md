# Fast-redox mixing boundary probes

Both scripts were run using the existing audit CLI, without rebuilding:

```sh
KERO_DUMP_INPUT=all target/debug/kero run tools/chemistry-audit/fast-redox-mix-probe/direct.lab --json
KERO_DUMP_INPUT=all target/debug/kero run tools/chemistry-audit/fast-redox-mix-probe/mixed.lab --json
```

CLI SHA256: `e3418a89a552c6eca38aa6179d2b6bd71b963167c923862b0fc8d198c611e8b7`.
The respective `.ndjson` and `.stderr` files preserve results and native inputs.
Both exited successfully. Final pH was 1.328841401558; native Fe(II), Fe(III),
and Mn(II) inventories were approximately 0.00400111, 0.000998886, and
0.000200000 mol in both cases.

These are **not native MIX electron-budget validation**. The CLI stack's
displacement wrapper does not delegate `mix`; both experiments therefore
reached ordinary equilibrium. Its normal direct solver balances the closed
fast-redox electron ledger. The library's independently callable native MIX
path formerly omitted that root solve when fast-redox components were coupled.
Reporting no pe did not constrain the resulting composition.

The bounded correction is to decline native MIX before target mutation when
the merged inventory requires coupled redox; the bench then uses its ordinary
equilibrium owner. Direct trait regression tests (including an uncoupled
positive control) live in
`crates/kerotakis-phreeqc/tests/mix_redox_boundary.rs`. Delegating MIX through
the displacement wrapper without reviewing the remaining stack passes is
outside this correction's scope.

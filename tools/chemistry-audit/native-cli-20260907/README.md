# Native CLI checkpoint, 2026-09-07

The user requested compiling only the CLI to obtain chemistry results without
waiting for the browser/mobile/full-workspace CI matrix. The native CLI built
with `cargo build --locked --offline -p kerotakis-cli --bin kero -j 1` in 8m32s.
No reduced chemistry feature set was used. Each recorder summary preserves the
binary digest and source/lock metadata. The executable remained unchanged
through all of the recorded runs below; intervening test/docs changes did not
change the executable. Later implementation changes require a new binary and
a separate output directory.

Results from that executable:

- Original fleet: 50 runs, 43/43 scientific checks, no solver failures.
- Second fleet: 36 runs, 121/121 checks, no solver failures.
- Native inventory for those fleets: 150/150 comparisons across 457 rows;
  zero invalid comparisons or internal-alias leaks.
- Selected catalog replay: 10 runs, 30/30 checks.
- Third fleet: 36 runs, 173/174 checks, no solver failures.
- Third-fleet native inventory: 69/69 comparisons across 224 rows;
  zero invalid comparisons or internal-alias leaks.

The failed third-fleet assertion is retained, not waived: unsupported ammonia
distillation transferred no component but still caused post-operation aqueous
re-equilibration, changing the source's contents, temperature and solute charge.
The generic runner must honor an explicit unchanged/refused operation result
before invoking solvers or downstream physical-state mutations. A new focused
test validates full-state preservation at three scales and in fraction/energy
modes; both it and the successful-distillation control pass locally. The CLI
repair still requires rebuilding and replay in a fresh directory.

Additional local checks (not embedded as experiment output): all 1,125 web
tests across 94 files passed with one worker; three pure-water phase fallback
tests passed; the reviewed lesson snapshot passed. The retained
`lessons.actual.json` added exactly two lessons, with zero changes to existing
lessons. Full CLI codex lint replayed all 110 entries successfully, but reported
237 prose numbers outside its automated accounting; those are not certified.

This checkpoint is not a full CI pass or merge approval. Model limitations in
the audit documentation and third-fleet predeclared bounds still apply.
